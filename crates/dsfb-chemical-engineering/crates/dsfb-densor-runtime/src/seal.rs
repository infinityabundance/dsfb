//! Deterministic, tamper-evident sealing — a self-contained copy of the DSFB `CanonicalHasher` discipline.
//!
//! This crate keeps its own copy (like `dsfb-chemical-engineering-atlas` does) so the runtime substrate has no
//! dependency on any chemical crate — it is a generic mechanism. The protocol is byte-identical to the edge
//! crate's: each field is encoded as `len(label) LE | label | len(value) LE | value`, so field order and content
//! are unambiguous and two runs over the same inputs produce the same digest.

use sha2::{Digest, Sha256};

/// A canonical, order-stable SHA-256 sealer. Build a preimage from labelled fields, then `finalize`.
pub struct CanonicalHasher {
    inner: Sha256,
}

impl Default for CanonicalHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalHasher {
    pub fn new() -> Self {
        CanonicalHasher {
            inner: Sha256::new(),
        }
    }

    /// Absorb a labelled byte field. Both label and value are length-prefixed (little-endian `u64`) so no two
    /// distinct (label, value) sequences can collide by concatenation.
    pub fn field(&mut self, label: &str, bytes: &[u8]) -> &mut Self {
        self.inner.update((label.len() as u64).to_le_bytes());
        self.inner.update(label.as_bytes());
        self.inner.update((bytes.len() as u64).to_le_bytes());
        self.inner.update(bytes);
        self
    }

    /// Absorb a labelled `u64` (8 bytes little-endian).
    pub fn u64(&mut self, label: &str, v: u64) -> &mut Self {
        self.field(label, &v.to_le_bytes())
    }

    /// Absorb a labelled 32-byte digest.
    pub fn hash32(&mut self, label: &str, h: &[u8; 32]) -> &mut Self {
        self.field(label, h)
    }

    /// Finalise to the raw 32-byte digest.
    pub fn finalize(self) -> [u8; 32] {
        self.inner.finalize().into()
    }

    /// Finalise to a 64-char lowercase hex digest.
    pub fn finalize_hex(self) -> String {
        to_hex(&self.finalize())
    }
}

/// One-shot raw-bytes SHA-256 → 32 bytes.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

/// Lowercase hex of a 32-byte digest.
pub fn to_hex(h: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in h {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_order_and_length_prefix_prevent_collisions() {
        // ("ab","c") vs ("a","bc") must NOT collide thanks to length prefixing.
        let mut a = CanonicalHasher::new();
        a.field("ab", b"c");
        let mut b = CanonicalHasher::new();
        b.field("a", b"bc");
        assert_ne!(a.finalize(), b.finalize());
    }

    #[test]
    fn seal_is_deterministic() {
        let mk = || {
            let mut h = CanonicalHasher::new();
            h.field("schema", b"x")
                .u64("n", 3)
                .hash32("auth", &sha256(b"policy"));
            h.finalize()
        };
        assert_eq!(mk(), mk());
        assert_eq!(to_hex(&mk()).len(), 64);
    }
}
