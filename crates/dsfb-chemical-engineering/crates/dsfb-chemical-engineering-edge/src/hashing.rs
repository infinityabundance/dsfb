//! Deterministic hashing for replay verification and dataset provenance gates.
//!
//! ## Purpose
//!
//! Replay hashes are computed over a canonical, byte-stable serialisation of the grammar-state
//! stream and the fused episodes, so that re-running the identical pipeline on identical inputs
//! yields an identical digest. Dataset gates hash the raw bytes of a vendored slice to confirm it
//! matches the value recorded in the provenance manifest.
//!
//! ## `CanonicalHasher` design
//!
//! Rather than relying on a serialisation library (which could change behaviour across versions),
//! `CanonicalHasher` uses a simple, explicit protocol:
//! - Every field is prefixed by its label length (8 bytes LE) + label bytes, then value length
//!   (8 bytes LE) + value bytes. This makes the encoding unambiguous: different field names or
//!   ordering produce different digests even if the values happen to be the same.
//! - Integers are fed as little-endian bytes (fixed-width), avoiding any decimal formatting.
//! - Floats are quantised to a 1 ns grid (`round(v * 1e9)` as `i64`) before hashing, so that
//!   benign platform floating-point differences (last-bit noise) do not break replay.
//!
//! ## SHA-256
//!
//! SHA-256 is used because it is:
//! - Cryptographically collision-resistant (important for forensic evidence chains).
//! - Available via the `sha2` crate with no unsafe code.
//! - Standard enough that external auditors can independently verify digests.

use sha2::{Digest, Sha256};

/// Compute the SHA-256 digest of arbitrary bytes and return it as a 64-character lowercase hex string.
///
/// Used for dataset provenance gates: the caller computes `sha256_hex` over the raw bytes of a
/// committed CSV slice and compares against the value in `data/MANIFEST.toml`. This confirms the
/// slice has not been modified since the manifest was written.
///
/// # Examples
/// ```
/// use dsfb_chemical_engineering_edge::hashing::sha256_hex;
/// let d = sha256_hex(b"residual slice");
/// assert_eq!(d.len(), 64);                       // 32 bytes as lowercase hex
/// assert_eq!(d, sha256_hex(b"residual slice"));  // deterministic
/// assert_ne!(d, sha256_hex(b"residual slicE"));  // sensitive to any byte change
/// ```
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    d.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Streaming SHA-256 hasher that accepts named, typed fields in a caller-defined fixed order.
///
/// The field protocol is:
/// ```text
/// SHA256.update( len(label)  as u64 LE )
/// SHA256.update( label bytes            )
/// SHA256.update( len(value)  as u64 LE )
/// SHA256.update( value bytes            )
/// ```
/// The length prefixes on both label and value prevent collisions between:
/// - fields with different names but the same value bytes, and
/// - adjacent fields whose concatenations happen to match a longer field.
///
/// The caller is responsible for feeding fields in a **fixed, documented order** on every
/// invocation. Reordering fields is a breaking change that produces a different digest —
/// this is intentional (field order is part of the canonical contract).
///
/// # Examples
/// ```
/// use dsfb_chemical_engineering_edge::hashing::CanonicalHasher;
/// // Seal the same labelled fields in the same order → the same digest; change any
/// // value (or label, or order) → a different digest.
/// let seal = |n: u64| {
///     let mut h = CanonicalHasher::new();
///     h.field("schema", b"demo_v1").u64("count", n);
///     h.finalize_hex()
/// };
/// assert_eq!(seal(3), seal(3)); // deterministic
/// assert_ne!(seal(3), seal(4)); // a real change moves the seal
/// assert_eq!(seal(3).len(), 64);
/// ```
#[derive(Default)]
pub struct CanonicalHasher {
    inner: Sha256,
}

impl CanonicalHasher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the digest with one labelled byte-slice field.
    ///
    /// Both the label and the value are length-prefixed so that the encoding is unambiguous.
    /// All higher-level helpers (`u64`, `f64q`) delegate to this method.
    pub fn field(&mut self, label: &str, bytes: &[u8]) -> &mut Self {
        self.inner.update((label.len() as u64).to_le_bytes());
        self.inner.update(label.as_bytes());
        self.inner.update((bytes.len() as u64).to_le_bytes());
        self.inner.update(bytes);
        self
    }

    /// Update with a `u64` field encoded as 8 bytes little-endian.
    pub fn u64(&mut self, label: &str, v: u64) -> &mut Self {
        self.field(label, &v.to_le_bytes())
    }

    /// Update with an `f64` field, first quantised to a 1e-9 grid.
    ///
    /// Quantisation: `round(v * 1e9)` as `i64`, little-endian. This collapses differences smaller than
    /// 1e-9 (within platform float noise) while remaining sensitive to any difference ≥ 1e-9 (a real
    /// numerical change), **for the magnitude range where the 1e-9 grid is representable as `i64`**:
    /// `|v| < i64::MAX / 1e9 ≈ 9.2e9`. Detector margins and residuals are far inside this band.
    ///
    /// Two edge cases are handled deterministically so the digest is total and collision-free:
    /// - Non-finite (`NaN`, `±∞`) → `i64::MIN`. All non-finite inputs **deliberately collapse to this one
    ///   sentinel**: the grammar treats any non-finite residual as an out-of-band sensor-fault token, so the
    ///   distinction between `NaN`, `+∞`, and `−∞` carries no admissible meaning and is intentionally not
    ///   preserved in the seal.
    /// - Finite but out of grid range (`|v| ≳ 9.2e9`, where `f64` precision is already coarser than
    ///   1e-9 anyway): the saturating `as i64` cast would map *every* such value to `i64::MAX`/`MIN`
    ///   and silently collide, so we hash the raw IEEE-754 bit pattern instead — distinct large values
    ///   stay distinct. The in-range encoding is unchanged, so all frozen digests are unaffected.
    pub fn f64q(&mut self, label: &str, v: f64) -> &mut Self {
        let scaled = v * 1e9;
        let q = if !v.is_finite() {
            i64::MIN
        } else if scaled.abs() < i64::MAX as f64 {
            scaled.round() as i64 // 1e-9 grid (unchanged from the original encoding)
        } else {
            v.to_bits() as i64 // out-of-grid magnitude: raw bits, so large values never collide
        };
        self.field(label, &q.to_le_bytes())
    }

    /// Consume the hasher and return the final digest as a 64-character lowercase hex string.
    pub fn finalize_hex(self) -> String {
        let d = self.inner.finalize();
        d.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hash a single `f64q` field in isolation, for behavioural pinning.
    fn h(v: f64) -> String {
        let mut c = CanonicalHasher::new();
        c.f64q("x", v);
        c.finalize_hex()
    }

    /// The 1e-9 grid: differences below it collapse (platform float noise is absorbed); differences at or
    /// above it are preserved (a real numerical change must move the seal).
    #[test]
    fn f64q_grid_absorbs_subgrid_noise_but_keeps_real_changes() {
        assert_eq!(
            h(1.0),
            h(1.0 + 4e-10),
            "a <1e-9 difference must collapse to the same grid point"
        );
        assert_ne!(
            h(1.0),
            h(1.0 + 2e-9),
            "a >=1e-9 difference must change the seal"
        );
    }

    /// All non-finite inputs deliberately collapse to the single `i64::MIN` sentinel (any non-finite residual
    /// is an out-of-band sensor-fault token; the NaN/±∞ distinction carries no admissible meaning), and they
    /// are always distinct from a finite value.
    #[test]
    fn f64q_nonfinite_all_map_to_one_sentinel_distinct_from_finite() {
        let nan = h(f64::NAN);
        assert_eq!(nan, h(f64::INFINITY));
        assert_eq!(nan, h(f64::NEG_INFINITY));
        assert_ne!(
            nan,
            h(0.0),
            "the non-finite sentinel must differ from a finite value"
        );
    }

    /// Out-of-grid magnitudes (|v| ≳ 9.2e9) fall back to the raw IEEE-754 bit pattern, so distinct large
    /// values stay distinct instead of all saturating to one `i64` extremum and silently colliding. This is
    /// the anti-collision property the boundary exists to guarantee.
    #[test]
    fn f64q_large_values_do_not_collide_via_saturation() {
        let big = 1e10_f64; // beyond the ~9.2e9 grid limit
        assert_ne!(
            h(big),
            h(big + 1.0),
            "distinct out-of-grid values must not collide"
        );
        assert_ne!(h(big), h(2.0 * big));
        assert_ne!(
            h(big),
            h(1.0),
            "an out-of-grid value must differ from an in-grid one"
        );
        // Symmetric on the negative side.
        assert_ne!(h(-big), h(-big - 1.0));
        assert_ne!(h(big), h(-big));
    }
}
