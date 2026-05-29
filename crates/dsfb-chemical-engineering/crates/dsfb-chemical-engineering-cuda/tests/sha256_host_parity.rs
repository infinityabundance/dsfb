//! Host-side parity gate for the device SHA-256 padding logic — runs on EVERY machine, no GPU needed.
//!
//! The on-GPU `dsfb_sha256_*` routine in `cuda/sha256.cuh` had a two-block padding bug (P43): the length
//! field was inflated by 512 bits whenever the pre-pad buffer occupancy was 56..63 bytes, i.e. for any
//! evidence stream of `40*n_samples` bytes with `n_samples ≡ 3 (mod 8)`. The fix snapshots the true
//! message length *before* padding. The `#[cfg(feature="cuda")]` parity test (`gpu_cpu_parity.rs`) only
//! exercises that on a real CUDA host, so the regression is otherwise invisible on a CPU-only box.
//!
//! This test makes the proof permanent and machine-independent: it is a faithful Rust **port of the
//! exact (fixed) device padding algorithm** from `sha256.cuh` — the same `update` (which adds 512 bits
//! per completed block) and the same `final` (which encodes a pre-pad `total_bits` snapshot) — and it
//! asserts the port equals the canonical `sha2::Sha256` for the message lengths that triggered the bug.
//! It is an executable specification of the contract the device kernel must satisfy; keep it in sync
//! with `cuda/sha256.cuh` (any change there must keep this green, and a regression to the pre-fix
//! padding would make these assertions fail).

use sha2::{Digest, Sha256};

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Faithful Rust mirror of `dsfb_sha256_ctx` + its update/final, ported byte-for-byte from `sha256.cuh`.
struct DeviceSha {
    state: [u32; 8],
    bitlen: u64,
    buf: [u8; 64],
    buflen: usize,
}

impl DeviceSha {
    fn new() -> Self {
        // FIPS 180-4 §5.3.3 initial hash values (mirror dsfb_sha256_init).
        DeviceSha {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            bitlen: 0,
            buf: [0u8; 64],
            buflen: 0,
        }
    }

    /// Mirror of `dsfb_sha256_block`: the SHA-256 compression function (FIPS 180-4 §6.2.2).
    fn block(&mut self, p: &[u8; 64]) {
        // SHA-256 circular right-rotate. `(x >> n) | (x << (32 - n))` is exactly `rotate_right(n)` for
        // u32 (n is always in 1..=25 here, never 0), so this is bit-identical to the manual form it replaces.
        let rotr = |x: u32, n: u32| x.rotate_right(n);
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = ((p[i * 4] as u32) << 24)
                | ((p[i * 4 + 1] as u32) << 16)
                | ((p[i * 4 + 2] as u32) << 8)
                | (p[i * 4 + 3] as u32);
        }
        for i in 16..64 {
            let s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            let s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            self.state[0],
            self.state[1],
            self.state[2],
            self.state[3],
            self.state[4],
            self.state[5],
            self.state[6],
            self.state[7],
        );
        for i in 0..64 {
            let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (st, v) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *st = st.wrapping_add(v);
        }
    }

    /// Mirror of `dsfb_sha256_update`: buffer bytes; on a full 64-byte block, process it and add 512 bits.
    fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.buf[self.buflen] = byte;
            self.buflen += 1;
            if self.buflen == 64 {
                let b = self.buf;
                self.block(&b);
                self.bitlen += 512;
                self.buflen = 0;
            }
        }
    }

    /// Mirror of the FIXED `dsfb_sha256_final`: encode a PRE-PAD length snapshot (the P43 fix), so the
    /// `bitlen += 512` that `update` applies when the padding wraps a second block cannot corrupt the
    /// encoded length. Returns the 32-byte digest.
    fn finalize(mut self) -> [u8; 32] {
        let total_bits = self.bitlen + (self.buflen as u64) * 8; // snapshot BEFORE padding
        self.update(&[0x80]);
        while self.buflen != 56 {
            self.update(&[0x00]);
        }
        let lenbe = total_bits.to_be_bytes();
        for &b in &lenbe {
            self.buf[self.buflen] = b;
            self.buflen += 1;
        }
        let b = self.buf;
        self.block(&b);
        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i * 4..i * 4 + 4].copy_from_slice(&self.state[i].to_be_bytes());
        }
        out
    }
}

fn device_digest(msg: &[u8]) -> [u8; 32] {
    let mut c = DeviceSha::new();
    c.update(msg);
    c.finalize()
}

#[test]
fn device_sha256_matches_reference_across_padding_residues() {
    // Build the message-length set: FIPS edge cases, the block-boundary lengths, and — critically — the
    // evidence-stream sizes 40*n for the n_samples ≡ 3 (mod 8) class (40*n ≡ 56 mod 64), which is exactly
    // the previously-broken two-block padding case (3 -> 120 B, 11 -> 440 B, 27 -> 1080 B, 43 -> 1720 B).
    let mut lengths: Vec<usize> = vec![0, 3, 55, 56, 63, 64, 119, 120, 121, 184];
    for n in [3usize, 11, 27, 43] {
        lengths.push(40 * n);
    }
    for &len in &lengths {
        // Deterministic, content-varying message so the digest actually depends on the bytes.
        let msg: Vec<u8> = (0..len).map(|i| (i * 31 + 7) as u8).collect();
        let got = device_digest(&msg);
        let want = Sha256::digest(&msg);
        assert_eq!(
            got[..],
            want[..],
            "device SHA-256 port diverges from sha2 at len={len} (40*n two-block case if len%64==56: {}) \
             — the sha256.cuh padding fix has regressed",
            len % 64 == 56
        );
    }

    // Spot-check the canonical FIPS 180-4 vector "abc" explicitly.
    assert_eq!(
        device_digest(b"abc")[..],
        Sha256::digest(b"abc")[..],
        "FIPS-180-4 \"abc\" known-answer must match"
    );
}
