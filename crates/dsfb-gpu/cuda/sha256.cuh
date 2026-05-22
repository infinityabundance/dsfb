// cuda/sha256.cuh
//
// `__device__` SHA-256, mirroring `dsfb-gpu-debug-core/src/hash.rs`
// line-for-line. The two implementations must produce byte-identical
// digests for byte-identical inputs; this is a load-bearing property of
// the Tier 3B path and is pinned by acceptance tests
// `device_sha256_self_test` (three known-vector inputs) and
// `throughput_device_digests_match_host_digests` (canonical-fixture
// per-stage hashes match the host Tier 3A path exactly).
//
// Why a single-thread serial implementation: SHA-256 is inherently
// serial across its 64-byte compression blocks — each block's compress
// step depends on the prior block's state. There is no correctness-
// preserving way to parallelise the standard hash across a single
// stream, and the win of Tier 3B is *not* faster SHA-256 in isolation
// (one GPU thread is slower at SHA-256 than one CPU core). The win is
// keeping ~160 KB of per-stage cell bytes on the device and freeing the
// host CPU to do other host-side work (case-file assembly, the next
// batch's `compute_features`, the bank stage). The kernels are launched
// independently per stage and may run in parallel on the device, but
// each kernel is one block of one thread.
//
// Numerics and endianness mirror the Rust side exactly:
//   * K[] / H_INIT[] copied verbatim from FIPS 180-4
//   * big-endian word load on every 4-byte slot of the input block
//   * big-endian message-length suffix at finalize
//   * big-endian state-to-digest emission
//
// Independence from CUDA intrinsics: nothing in this header uses
// `__shfl_*`, FMA, fast-math, atomics, or warp shuffles. The build
// is configured (build.rs) with `--fmad=false` and without
// `--use_fast_math`, both of which are irrelevant to integer SHA-256
// anyway. The deterministic numeric contract pins this posture.

#pragma once

#include <cstdint>

namespace dsfb {

// SHA-256 round constants (FIPS 180-4 §4.2.2). First 32 bits of the
// fractional parts of the cube roots of the first 64 primes.
__device__ __constant__ const uint32_t SHA256_K[64] = {
    0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u,
    0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
    0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u,
    0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
    0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu,
    0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
    0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u,
    0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
    0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u,
    0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
    0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u,
    0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
    0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u,
    0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
    0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u,
    0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u,
};

// Initial hash values (FIPS 180-4 §5.3.3). First 32 bits of the
// fractional parts of the square roots of the first 8 primes.
__device__ __constant__ const uint32_t SHA256_H_INIT[8] = {
    0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
    0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u,
};

// Helpers — direct transcription of the Rust counterparts.

__device__ __forceinline__ uint32_t rotr32(uint32_t x, uint32_t n) {
    return (x >> n) | (x << (32 - n));
}

__device__ __forceinline__ uint32_t load_be_u32(const uint8_t* p) {
    return (static_cast<uint32_t>(p[0]) << 24)
         | (static_cast<uint32_t>(p[1]) << 16)
         | (static_cast<uint32_t>(p[2]) << 8)
         | static_cast<uint32_t>(p[3]);
}

__device__ __forceinline__ void store_be_u32(uint8_t* p, uint32_t v) {
    p[0] = static_cast<uint8_t>((v >> 24) & 0xff);
    p[1] = static_cast<uint8_t>((v >> 16) & 0xff);
    p[2] = static_cast<uint8_t>((v >> 8) & 0xff);
    p[3] = static_cast<uint8_t>(v & 0xff);
}

__device__ __forceinline__ void store_be_u64(uint8_t* p, uint64_t v) {
    for (int i = 0; i < 8; ++i) {
        p[i] = static_cast<uint8_t>((v >> ((7 - i) * 8)) & 0xff);
    }
}

// Compression function — one 64-byte block, in place on the 8-word
// state. Direct transcription of `Sha256::compress` in hash.rs.
__device__ __forceinline__ void sha256_compress(uint32_t state[8], const uint8_t block[64]) {
    uint32_t w[64];
    for (int i = 0; i < 16; ++i) {
        w[i] = load_be_u32(block + i * 4);
    }
    for (int i = 16; i < 64; ++i) {
        uint32_t s0 = rotr32(w[i - 15], 7) ^ rotr32(w[i - 15], 18) ^ (w[i - 15] >> 3);
        uint32_t s1 = rotr32(w[i - 2], 17) ^ rotr32(w[i - 2], 19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16] + s0 + w[i - 7] + s1;
    }

    uint32_t a = state[0], b = state[1], c = state[2], d = state[3];
    uint32_t e = state[4], f = state[5], g = state[6], h = state[7];

    for (int i = 0; i < 64; ++i) {
        uint32_t big_sigma1 = rotr32(e, 6) ^ rotr32(e, 11) ^ rotr32(e, 25);
        uint32_t ch = (e & f) ^ ((~e) & g);
        uint32_t t1 = h + big_sigma1 + ch + SHA256_K[i] + w[i];
        uint32_t big_sigma0 = rotr32(a, 2) ^ rotr32(a, 13) ^ rotr32(a, 22);
        uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
        uint32_t t2 = big_sigma0 + maj;

        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    state[0] += a; state[1] += b; state[2] += c; state[3] += d;
    state[4] += e; state[5] += f; state[6] += g; state[7] += h;
}

// One-shot device SHA-256 over `len` bytes at `data`. Writes 32-byte
// big-endian digest to `out`. Designed for single-thread invocation;
// no internal parallelism. Memory traffic: reads `data` sequentially
// in 64-byte blocks (good for L1/L2 cache utilisation), writes 32 bytes
// at the end.
__device__ inline void dsfb_sha256_device(const uint8_t* data, uint64_t len, uint8_t out[32]) {
    uint32_t state[8];
    for (int i = 0; i < 8; ++i) state[i] = SHA256_H_INIT[i];

    uint64_t full_blocks = len / 64;
    for (uint64_t i = 0; i < full_blocks; ++i) {
        sha256_compress(state, data + i * 64);
    }
    uint64_t tail_start = full_blocks * 64;
    uint64_t tail_len = len - tail_start;

    // Build the trailing one or two padded blocks. `buf` holds bytes
    // beyond the last full block; `buf_len` is the live byte count.
    uint8_t buf[128];
    for (int i = 0; i < 128; ++i) buf[i] = 0;
    for (uint64_t i = 0; i < tail_len; ++i) {
        buf[i] = data[tail_start + i];
    }
    buf[tail_len] = 0x80;

    uint64_t bit_len = len * 8;
    // If the tail+0x80 cannot fit the 8-byte length in the same 64-byte
    // block (i.e., tail_len > 55), we need a second padded block.
    if (tail_len > 55) {
        store_be_u64(buf + 120, bit_len);
        sha256_compress(state, buf);
        sha256_compress(state, buf + 64);
    } else {
        store_be_u64(buf + 56, bit_len);
        sha256_compress(state, buf);
    }

    for (int i = 0; i < 8; ++i) {
        store_be_u32(out + i * 4, state[i]);
    }
}

// ============================================================
// S-PERF.14b.1 — streaming SHA-256 primitives (init / update /
// finalize). Required by the panel-locked Path 1b root-kernel
// rewrite: streaming SHA absorbs the canonical
// CompactDensorDigestV1 byte stream (44 B header + n_chunks ×
// 32 B leaves) directly from global memory WITHOUT staging
// through a per-block scratch buffer.
//
// **Byte-stream contract (panel-locked, MUST hold)**: for any
// input byte sequence, the streaming sequence
// `init → update(chunk_1) → ... → update(chunk_k) → finalize`
// MUST produce a 32-byte digest byte-identical to
// `dsfb_sha256_device(concat(chunk_1..chunk_k))`. Cross-
// validated by `dsfb_gpu_sha256_streaming_self_test` FFI on
// three known-vector inputs (empty / 55 B / 64 KB tail-padding
// edge cases).
//
// **Design**: ctx holds the running 8×u32 SHA state + a 64-byte
// staging buffer + live byte count + total bit-length accumulator.
// `update` appends bytes one at a time into the staging buffer
// and compresses 64-byte blocks as they fill (avoiding the
// requirement that caller chunks be 64-byte aligned). `finalize`
// applies the canonical 1-or-2-block padding + length encoding
// from `dsfb_sha256_device` (same tail-padding edge case at
// buf_len > 55).
//
// Single-thread invocation only (SHA-256 is inherently serial
// within a stream). No internal parallelism.
// ============================================================

struct Sha256Ctx {
    uint32_t state[8];     // running SHA state (8 × u32 BE words)
    uint8_t  buf[64];      // staging buffer for partial-block bytes
    uint32_t buf_len;      // live byte count in `buf` (0..64)
    uint64_t total_len;    // total bytes absorbed so far (drives length encoding)
};

// Initialise a streaming SHA-256 context. Zero out staging
// buffer + total-length accumulator; seed state from
// SHA256_H_INIT. Equivalent to entering the one-shot path with
// zero bytes absorbed.
__device__ __forceinline__ void dsfb_sha256_init(Sha256Ctx* ctx) {
    for (int i = 0; i < 8; ++i) ctx->state[i] = SHA256_H_INIT[i];
    for (int i = 0; i < 64; ++i) ctx->buf[i] = 0;
    ctx->buf_len = 0;
    ctx->total_len = 0;
}

// Absorb `len` bytes from `data` into the streaming SHA state.
// Appends bytes one-at-a-time into the 64-byte staging buffer
// and compresses as each block fills. Caller may call `update`
// any number of times with any chunk size; the produced digest
// only depends on the concatenated byte stream, not the chunking.
//
// Memory traffic: reads `data` sequentially (good for L1/L2
// utilisation), writes 0 bytes to global. The 64-byte staging
// buffer lives in the per-thread register / local-memory file —
// NO global-memory scratch buffer, NO L2-resident scratch
// round-trip (the elimination that drives S-PERF.14b.1's win).
__device__ __forceinline__ void dsfb_sha256_update(Sha256Ctx* ctx,
                                                   const uint8_t* data,
                                                   uint64_t len) {
    // Aligned-bulk fast path is the load-bearing optimisation for
    // S-PERF.14b.1 v2: when the staging buffer is empty and at
    // least one full 64-byte block is available, compress directly
    // from `data` without a byte-by-byte copy through `ctx->buf`.
    // Byte-identical to the slow path (cross-validated by the
    // dsfb_gpu_sha256_streaming_self_test known-vector harness on
    // empty / 55-byte / 64-KiB inputs) but ~64× fewer per-byte
    // memory ops per fully-aligned chunk.
    ctx->total_len += len;
    uint64_t i = 0;

    // Slow-path drain: fill any partial staging buffer first so
    // the fast path can start at a clean 64-byte boundary.
    while (ctx->buf_len != 0 && i < len) {
        ctx->buf[ctx->buf_len++] = data[i++];
        if (ctx->buf_len == 64) {
            sha256_compress(ctx->state, ctx->buf);
            ctx->buf_len = 0;
        }
    }

    // Fast path: compress whole 64-byte blocks directly from `data`
    // without staging. Only fires when buf_len == 0 (drain above
    // guarantees that condition before each iteration).
    while (len - i >= 64) {
        sha256_compress(ctx->state, data + i);
        i += 64;
    }

    // Slow-path tail: stage the trailing < 64 bytes for the next
    // call (or for finalize).
    for (; i < len; ++i) {
        ctx->buf[ctx->buf_len++] = data[i];
        if (ctx->buf_len == 64) {
            sha256_compress(ctx->state, ctx->buf);
            ctx->buf_len = 0;
        }
    }
}

// Finalise the streaming SHA-256 context and write the 32-byte
// big-endian digest to `out`. Applies the canonical 1-or-2-block
// padding + length encoding (same algorithm as
// `dsfb_sha256_device`'s tail-block handling — verified
// byte-identical by `dsfb_gpu_sha256_streaming_self_test`).
//
// After finalize the context state is undefined; do not call
// `update` or `finalize` again without re-initialising.
__device__ __forceinline__ void dsfb_sha256_finalize(Sha256Ctx* ctx, uint8_t out[32]) {
    // Append the 0x80 padding byte and zero-fill the rest of the
    // staging buffer.
    uint32_t tail_len = ctx->buf_len;
    uint8_t pad[128];
    for (int i = 0; i < 128; ++i) pad[i] = 0;
    for (uint32_t i = 0; i < tail_len; ++i) pad[i] = ctx->buf[i];
    pad[tail_len] = 0x80;

    uint64_t bit_len = ctx->total_len * 8;

    // Tail-padding edge case: if tail+0x80 doesn't fit the 8-byte
    // length in the same 64-byte block (tail_len > 55), need a
    // second padded block. Same condition + same length-position
    // as dsfb_sha256_device.
    if (tail_len > 55) {
        store_be_u64(pad + 120, bit_len);
        sha256_compress(ctx->state, pad);
        sha256_compress(ctx->state, pad + 64);
    } else {
        store_be_u64(pad + 56, bit_len);
        sha256_compress(ctx->state, pad);
    }

    for (int i = 0; i < 8; ++i) {
        store_be_u32(out + i * 4, ctx->state[i]);
    }
}

} // namespace dsfb
