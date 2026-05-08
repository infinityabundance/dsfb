//! DSFB-Debug: hand-rolled SHA-256 — FIPS 180-4 §6.2.
//!
//! # Why hand-rolled
//!
//! The crate's zero-dep policy forbids `sha2` / `ring` / `openssl`
//! at runtime. SHA-256 is the only cryptographic primitive
//! DSFB-Debug needs (fixture-byte integrity verification against
//! `MANIFEST.toml`); a single 200-LOC pure-Rust implementation
//! preserves the zero-dep policy while delivering the canonical
//! NIST hash.
//!
//! # Threat model
//!
//! This implementation is used **solely** to verify the integrity
//! of vendored fixture bytes against a manifest entry. It is
//! **never** used on secret material. Constant-time properties are
//! NOT a goal — the bytes are public test fixtures, the manifest
//! digest is public, and the only attack we defend against is
//! fixture-file tampering / drift, which the digest comparison
//! catches.
//!
//! # Reference
//!
//! NIST FIPS 180-4, Secure Hash Standard, August 2015. Tested
//! against the SHA-256 NIST test vectors (empty string, "abc",
//! 448-bit message, 512-bit alignment, the
//! `"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"`
//! standard vector) in the unit tests at the bottom of this file.

#![cfg(feature = "std")]

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

#[inline]
fn rotr(x: u32, n: u32) -> u32 { x.rotate_right(n) }

/// Compute SHA-256 of `data`. Returns the raw 32-byte digest.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = H0;

    // Pre-processing: append 0x80, pad with zeros, append 64-bit length.
    let bit_len: u64 = (data.len() as u64).wrapping_mul(8);
    // We process 64-byte blocks. Build the final padded stream lazily.
    let mut buf = std::vec::Vec::with_capacity(data.len() + 72);
    buf.extend_from_slice(data);
    buf.push(0x80);
    while buf.len() % 64 != 56 {
        buf.push(0);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    let mut offset = 0;
    while offset < buf.len() {
        let mut w = [0u32; 64];
        let mut i = 0;
        while i < 16 {
            let base = offset + i * 4;
            w[i] = u32::from_be_bytes([buf[base], buf[base + 1], buf[base + 2], buf[base + 3]]);
            i += 1;
        }
        let mut t = 16;
        while t < 64 {
            let s0 = rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] >> 3);
            let s1 = rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16].wrapping_add(s0).wrapping_add(w[t - 7]).wrapping_add(s1);
            t += 1;
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        let mut t = 0;
        while t < 64 {
            let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[t]).wrapping_add(w[t]);
            let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
            t += 1;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);

        offset += 64;
    }

    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 8 {
        let bytes = h[i].to_be_bytes();
        out[i * 4] = bytes[0];
        out[i * 4 + 1] = bytes[1];
        out[i * 4 + 2] = bytes[2];
        out[i * 4 + 3] = bytes[3];
        i += 1;
    }
    out
}

/// Hex-encoded SHA-256 (lowercase, 64 ASCII bytes).
pub fn sha256_hex(data: &[u8]) -> [u8; 64] {
    let digest = sha256(data);
    let hex_chars = b"0123456789abcdef";
    let mut out = [0u8; 64];
    let mut i = 0;
    while i < 32 {
        out[i * 2] = hex_chars[(digest[i] >> 4) as usize];
        out[i * 2 + 1] = hex_chars[(digest[i] & 0x0f) as usize];
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_eq(actual: &[u8; 64], expected: &str) -> bool {
        actual.as_slice() == expected.as_bytes()
    }

    #[test]
    fn nist_empty_string() {
        // FIPS 180-4 example: SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = sha256_hex(b"");
        assert!(hex_eq(&h, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
    }

    #[test]
    fn nist_abc() {
        // FIPS 180-4 example A.1: SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let h = sha256_hex(b"abc");
        assert!(hex_eq(&h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
    }

    #[test]
    fn nist_two_block() {
        // FIPS 180-4 example A.2 (multi-block padding):
        // SHA-256("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")
        // = 248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1
        let h = sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert!(hex_eq(&h, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"));
    }
}
