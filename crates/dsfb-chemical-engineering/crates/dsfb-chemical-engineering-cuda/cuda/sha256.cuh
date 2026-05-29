// sha256.cuh — minimal device-side SHA-256 (FIPS 180-4), incremental update API.
//
// This is an on-GPU, single-threaded (per lane) implementation of SHA-256. Each lane's thread
// maintains its own private context in registers/local memory and hashes the full sample stream
// sequentially. The result is byte-for-byte identical to the Rust `sha2` crate running on the
// host over the same byte stream.
//
// Why on-GPU SHA-256 (not CPU post-processing)?
//   Computing the digest on the GPU avoids transferring the raw per-sample evidence records
//   (40 bytes/sample * n_samples * n_lanes) back to the host. The kernel transfers only 32 bytes
//   per lane (the digest) plus 40 bytes per lane (5 i64 summary fields), which is orders of
//   magnitude smaller. The GPU is therefore an acceleration + attestation layer: it produces the
//   same forensic record as the CPU reference, but does so without the large intermediate transfer.
//
// Correctness: the implementation follows FIPS 180-4 exactly. The round constants K[0..63] are
// the first 32 fractional bits of the cube roots of the first 64 primes. The initial hash values
// H[0..7] are the first 32 fractional bits of the square roots of the first 8 primes.
// Message schedule, compression function, padding, and big-endian length encoding match the spec.
#pragma once
#include <stdint.h>

// SHA-256 incremental context (per-lane, lives in thread-local storage / registers).
typedef struct {
    uint32_t state[8];   /* running hash state H[0..7], initialised by dsfb_sha256_init */
    uint64_t bitlen;     /* total bits hashed so far (updated at each completed 64-byte block) */
    uint8_t  buf[64];    /* partial block accumulator (0..63 bytes pending) */
    uint32_t buflen;     /* number of bytes currently in buf */
} dsfb_sha256_ctx;

// SHA-256 round constants K[0..63]: first 32 fractional bits of cbrt(prime_i) for i=0..63.
// Stored in __constant__ memory so all threads in a warp read from the same cache line.
__device__ __constant__ uint32_t DSFB_K256[64] = {
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};

// Right-rotate a 32-bit word by n bits (standard ROTR_n operation from FIPS 180-4 §3.2).
// `__funnelshift_r(x, x, n)` is the single-instruction hardware rotate (SHF.R) on the GPU and is
// bit-identical to `(x>>n)|(x<<(32-n))` for n in [1,31] — a digest-preserving speedup of the
// per-round Σ/σ rotates that dominate the SHA-256 inner loop.
__device__ __forceinline__ uint32_t dsfb_rotr(uint32_t x, uint32_t n) { return __funnelshift_r(x, x, n); }

// Initialise a SHA-256 context with the standard initial hash values H[0..7].
// H[i] = first 32 fractional bits of sqrt(prime_i) for i=0..7 (FIPS 180-4 §5.3.3).
__device__ void dsfb_sha256_init(dsfb_sha256_ctx* c) {
    c->state[0]=0x6a09e667; c->state[1]=0xbb67ae85; c->state[2]=0x3c6ef372; c->state[3]=0xa54ff53a;
    c->state[4]=0x510e527f; c->state[5]=0x9b05688c; c->state[6]=0x1f83d9ab; c->state[7]=0x5be0cd19;
    c->bitlen=0; c->buflen=0;
}

// Process one 64-byte (512-bit) SHA-256 message block in `p`, updating the running state in `c`.
// Implements the SHA-256 compression function from FIPS 180-4 §6.2.2.
__device__ void dsfb_sha256_block(dsfb_sha256_ctx* c, const uint8_t* p) {
    uint32_t w[64];
    // Message schedule W[0..15]: parse 16 big-endian 32-bit words from the 64-byte block.
    #pragma unroll
    for (int i=0;i<16;i++)
        w[i] = (p[i*4]<<24)|(p[i*4+1]<<16)|(p[i*4+2]<<8)|(p[i*4+3]);
    // Message schedule W[16..63]: extend using σ_0 and σ_1 (FIPS 180-4 §4.1.2). Unrolled so the
    // compiler keeps the window in registers and schedules the σ rotates without loop overhead.
    #pragma unroll
    for (int i=16;i<64;i++) {
        uint32_t s0 = dsfb_rotr(w[i-15],7)^dsfb_rotr(w[i-15],18)^(w[i-15]>>3);  /* σ_0 */
        uint32_t s1 = dsfb_rotr(w[i-2],17)^dsfb_rotr(w[i-2],19)^(w[i-2]>>10);   /* σ_1 */
        w[i]=w[i-16]+s0+w[i-7]+s1;
    }
    // Working variables initialised from the running state (FIPS 180-4 §6.2.2 step 2).
    uint32_t a=c->state[0],b=c->state[1],cc=c->state[2],d=c->state[3],
             e=c->state[4],f=c->state[5],g=c->state[6],h=c->state[7];
    // 64 compression rounds (FIPS 180-4 §6.2.2 step 3). Unrolled: the rotating assignment
    // (h=g; g=f; …) becomes pure register renaming when unrolled, removing 64 iterations of copies
    // and the loop branch — the single biggest scheduling win for the per-thread SHA inner loop.
    #pragma unroll
    for (int i=0;i<64;i++) {
        uint32_t S1=dsfb_rotr(e,6)^dsfb_rotr(e,11)^dsfb_rotr(e,25);  /* Σ_1(e) */
        uint32_t ch=(e&f)^((~e)&g);                                    /* Ch(e,f,g) */
        uint32_t t1=h+S1+ch+DSFB_K256[i]+w[i];
        uint32_t S0=dsfb_rotr(a,2)^dsfb_rotr(a,13)^dsfb_rotr(a,22);  /* Σ_0(a) */
        uint32_t maj=(a&b)^(a&cc)^(b&cc);                              /* Maj(a,b,c) */
        uint32_t t2=S0+maj;
        h=g; g=f; f=e; e=d+t1; d=cc; cc=b; b=a; a=t1+t2;
    }
    // Add compressed chunk to current hash value (FIPS 180-4 §6.2.2 step 4).
    c->state[0]+=a; c->state[1]+=b; c->state[2]+=cc; c->state[3]+=d;
    c->state[4]+=e; c->state[5]+=f; c->state[6]+=g; c->state[7]+=h;
}

// Absorb `len` bytes from `data` into the SHA-256 context.
// Bytes are buffered until a full 64-byte block is accumulated, then the block is processed.
// `bitlen` tracks the total bits processed in completed blocks (updated per block, not per byte).
//
// Chunked copy: fill up to the next 64-byte boundary in one inner loop (one boundary branch per
// chunk, not per byte), process the block, repeat. Byte-for-byte identical buffering to the naive
// per-byte version — same bytes land in `buf` in the same order and blocks fire at the same points —
// but it strips the per-byte branch that dominated the 40-bytes/sample evidence feed.
__device__ void dsfb_sha256_update(dsfb_sha256_ctx* c, const uint8_t* data, uint32_t len) {
    uint32_t i = 0;
    while (i < len) {
        uint32_t room = 64 - c->buflen;          /* space left in the current block buffer */
        uint32_t take = (len - i < room) ? (len - i) : room; /* bytes we can copy before a boundary */
        for (uint32_t k = 0; k < take; k++) c->buf[c->buflen + k] = data[i + k];
        c->buflen += take;
        i += take;
        if (c->buflen == 64) {                   /* boundary reached: process the full block */
            dsfb_sha256_block(c, c->buf);
            c->bitlen += 512;                     /* 64 bytes = 512 bits */
            c->buflen = 0;
        }
    }
}

// Append an 8-byte little-endian value to the SHA-256 context.
// Matches the byte order of Rust's `.to_le_bytes()` on u64/i64. Used for all per-sample fields
// in the canonical evidence record (raw bits, q, e, d, s).
__device__ __forceinline__ void dsfb_sha256_update_u64le(dsfb_sha256_ctx* c, uint64_t v) {
    uint8_t b[8];
    #pragma unroll
    for (int i=0;i<8;i++) b[i]=(uint8_t)(v>>(8*i)); /* extract byte i (little-end first) */
    dsfb_sha256_update(c, b, 8);
}
// Convenience wrapper: reinterpret a signed int64 as uint64 before serialising.
// The bit pattern is unchanged; this matches Rust's i64.to_le_bytes() which also reinterprets.
__device__ __forceinline__ void dsfb_sha256_update_i64le(dsfb_sha256_ctx* c, long long v) {
    dsfb_sha256_update_u64le(c, (uint64_t)v);
}

// Finalise the SHA-256 computation and write the 32-byte digest to `out`.
// Implements the FIPS 180-4 §5.1.1 padding scheme:
//   1. Append a 0x80 byte (the "1" bit followed by zeros).
//   2. Pad with 0x00 bytes until the buffer is 56 bytes long (leaving 8 bytes for the length).
//   3. Append the total message length in **big-endian** bits as a u64.
//   4. Process the final padded block.
//   5. Output the eight 32-bit state words in big-endian byte order (standard SHA-256 output).
__device__ void dsfb_sha256_final(dsfb_sha256_ctx* c, uint8_t out[32]) {
    // Snapshot the TRUE total message length in bits BEFORE padding, and encode that snapshot below.
    //
    // Subtlety (the reason this is a snapshot, not `c->bitlen` read at the end): the padding bytes are
    // fed through `dsfb_sha256_update`, which does `c->bitlen += 512` whenever it completes a 64-byte
    // block. In the two-block padding case — pre-pad `buflen` in 56..63, i.e. for the evidence stream
    // any lane whose 40*n_samples byte count is ≡ 56 (mod 64), which is n_samples ≡ 3 (mod 8) — the
    // 0x80 + zero padding fills and wraps the first block, so `update` spuriously adds 512 bits to
    // `c->bitlen`. Reading `c->bitlen` after padding would then encode (true_len + 512) and produce a
    // digest that diverges from the host `sha2` crate. Snapshotting the length up-front is the standard
    // fix and makes the device digest equal the CPU reference for ALL message lengths.
    uint64_t total_bits = c->bitlen + (uint64_t)c->buflen * 8;
    uint8_t pad = 0x80; /* mandatory leading 1-bit */
    dsfb_sha256_update(c, &pad, 1);
    uint8_t zero = 0x00;
    // Pad with zeros until the buffer position is at byte 56 (leaving 8 bytes for the length). In the
    // two-block case this loop carries across a block boundary (buflen wraps 64 -> 0) and continues.
    while (c->buflen != 56) dsfb_sha256_update(c, &zero, 1);
    // Encode the total bit length in big-endian byte order (FIPS 180-4 §5.1.1) from the pre-pad snapshot.
    uint8_t lenbe[8];
    #pragma unroll
    for (int i=0;i<8;i++) lenbe[i]=(uint8_t)(total_bits>>(56-8*i)); /* most-significant byte first */
    // Write the 8 length bytes directly into the buffer and process the final block.
    // We do not call dsfb_sha256_update here to avoid incorrectly incrementing bitlen a second time.
    for (int i=0;i<8;i++) { c->buf[c->buflen++]=lenbe[i]; }
    dsfb_sha256_block(c, c->buf);
    // Serialise the eight 32-bit state words in big-endian order to produce the 32-byte digest.
    #pragma unroll
    for (int i=0;i<8;i++) {
        out[i*4]   = (uint8_t)(c->state[i]>>24);
        out[i*4+1] = (uint8_t)(c->state[i]>>16);
        out[i*4+2] = (uint8_t)(c->state[i]>>8);
        out[i*4+3] = (uint8_t)(c->state[i]);
    }
}
