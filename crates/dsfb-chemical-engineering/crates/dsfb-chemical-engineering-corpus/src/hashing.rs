//! Deterministic SHA-256 helpers for the soft-sensor data corpus (own copy; no dependency on edge).

use alloc::format;
use alloc::string::String;
use sha2::{Digest, Sha256};

/// SHA-256 of bytes as a raw 32-byte digest.
pub fn sha256_32(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

/// SHA-256 of bytes as a 64-character lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex32(&sha256_32(bytes))
}

/// Render a 32-byte digest as a 64-character lowercase hex string.
pub fn hex32(d: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Streaming canonical hasher: `len(label) LE | label | len(value) LE | value` per field, fixed order.
#[derive(Default)]
pub struct CanonicalHasher {
    inner: Sha256,
}

impl CanonicalHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(&mut self, label: &str, bytes: &[u8]) -> &mut Self {
        self.inner.update((label.len() as u64).to_le_bytes());
        self.inner.update(label.as_bytes());
        self.inner.update((bytes.len() as u64).to_le_bytes());
        self.inner.update(bytes);
        self
    }

    pub fn u64(&mut self, label: &str, v: u64) -> &mut Self {
        self.field(label, &v.to_le_bytes())
    }

    pub fn finalize_hex(self) -> String {
        hex32(&self.inner.finalize().into())
    }
}
