//! Deterministic SHA-256 hashing for the atlas authority gates.
//!
//! This is a verbatim copy of the `edge` crate's hashing protocol (`sha256_hex` + `CanonicalHasher`).
//! The authority crate owns its own copy so it stays free of any dependency on `edge`; the two
//! implementations are byte-identical by construction and each has its own determinism test.
//!
//! `CanonicalHasher` uses an explicit field protocol: every field is `len(label) LE | label |
//! len(value) LE | value`, integers are little-endian, and floats are quantised to a 1 ns grid
//! (`round(v*1e9)` as `i64`) so benign last-bit platform noise cannot change a digest.

use alloc::format;
use alloc::string::String;
use sha2::{Digest, Sha256};

/// SHA-256 of arbitrary bytes as a 64-character lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Streaming SHA-256 hasher accepting named, typed fields in a caller-defined fixed order.
///
/// The caller must feed fields in a fixed, documented order; reordering is a breaking change that
/// produces a different digest (field order is part of the canonical contract).
#[derive(Default)]
pub struct CanonicalHasher {
    inner: Sha256,
}

impl CanonicalHasher {
    pub fn new() -> Self {
        Self::default()
    }

    /// One labelled byte-slice field (label and value both length-prefixed).
    pub fn field(&mut self, label: &str, bytes: &[u8]) -> &mut Self {
        self.inner.update((label.len() as u64).to_le_bytes());
        self.inner.update(label.as_bytes());
        self.inner.update((bytes.len() as u64).to_le_bytes());
        self.inner.update(bytes);
        self
    }

    /// A `u64` field encoded as 8 bytes little-endian.
    pub fn u64(&mut self, label: &str, v: u64) -> &mut Self {
        self.field(label, &v.to_le_bytes())
    }

    /// Consume the hasher and return the digest as a 64-character lowercase hex string.
    pub fn finalize_hex(self) -> String {
        let d = self.inner.finalize();
        d.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
