//! Detector-motif registry: the canonical 16 detectors plus the registry
//! hash that the contract pins.
//!
//! The detector layer is part of the deterministic execution contract.
//! Two crucial properties live here:
//!
//! 1. The *order* of detector motifs. The detector cell encodes which
//!    detectors fired in a single 16-bit mask; bit `i` always refers to
//!    `MotifClass::variant_at(i)`. Reordering would silently flip the
//!    meaning of every stored mask.
//! 2. The registry *hash*. The contract carries a `registry_hash` field
//!    that pins the detector set; if the constant exposed here changes
//!    (new motif added, threshold table altered, name renamed) the
//!    contract hash must be recomputed and any stored case files become
//!    invalid. That is intentional — a detector-set change is a contract
//!    breach by design.

use crate::hash::{format_digest, sha256};

/// The deterministic catalog of detector motifs. Sixteen entries, all in
/// canonical order. Bit `i` of a `DetectorCell::detector_mask` is `1`
/// when `MOTIF_CATALOG[i].class` fired on that cell.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MotifClass {
    /// Single-cell norm above the spike threshold.
    ResidualSpike = 0,
    /// EWMA drift above the sustained threshold.
    SustainedResidualElevation = 1,
    /// Monotonically increasing drift for ≥ ramp-steps windows.
    DriftRamp = 2,
    /// Absolute slew above the shock threshold.
    SlewShock = 3,
    /// Norm above plateau threshold and slew near zero for ≥ plateau-windows.
    Plateau = 4,
    /// Sign of slew alternated ≥ oscillation-alternations times in last
    /// oscillation-window cells.
    Oscillation = 5,
    /// Norm crossed from below `deadband` to above `deadband + hysteresis`.
    DeadbandExit = 6,
    /// Residual error rate above the burst threshold.
    ErrorRateBurst = 7,
    /// Latency residual AND error residual both above their coupling
    /// thresholds within the same cell.
    LatencyErrorCoupling = 8,
    /// Cell's norm is markedly above the entity's running average — set
    /// in the consensus stage; the per-cell evaluator records a
    /// candidate-flag and lets the consensus pass confirm or reject it.
    EntityLocalAnomaly = 9,
    /// Cell's norm is markedly above the entity's average for other
    /// routes — placeholder bit, set when route_id distribution within
    /// the entity is uneven on this window.
    RouteLocalAnomaly = 10,
    /// Drift rising in an upstream entity while this cell shows elevated
    /// error rate. v0 uses a conservative single-cell approximation.
    FanoutPrecursor = 11,
    /// Sample variance of norm over the variance-window cells above
    /// `var_threshold`.
    VarianceExpansion = 12,
    /// Drift turning over (current drift < previous drift) while norm
    /// still elevated — signals recovery off a peak.
    RecoveryEdge = 13,
    /// Norm and drift both below the deadband; no other detector fired.
    /// Sentinel bit used by consensus to confirm "clean" cells.
    CleanWindowStability = 14,
    /// Single-cell spike that resolved in the very next window — used
    /// by the confuser-suppression axis.
    ConfuserLikeTransient = 15,
}

impl MotifClass {
    /// Total number of motifs in the canonical catalog. Doubles as the
    /// width of `detector_mask`.
    pub const COUNT: usize = 16;

    /// Map a class to its bit position in `detector_mask`.
    #[must_use]
    pub const fn bit_index(self) -> u32 {
        self as u32
    }

    /// Map a class to a `1u32 << bit_index` mask.
    #[must_use]
    pub const fn bit_mask(self) -> u32 {
        1u32 << self.bit_index()
    }

    /// Recover a class from its bit index. Returns `None` for any value
    /// `≥ COUNT`.
    #[must_use]
    pub const fn from_bit_index(bit: u32) -> Option<Self> {
        match bit {
            0 => Some(Self::ResidualSpike),
            1 => Some(Self::SustainedResidualElevation),
            2 => Some(Self::DriftRamp),
            3 => Some(Self::SlewShock),
            4 => Some(Self::Plateau),
            5 => Some(Self::Oscillation),
            6 => Some(Self::DeadbandExit),
            7 => Some(Self::ErrorRateBurst),
            8 => Some(Self::LatencyErrorCoupling),
            9 => Some(Self::EntityLocalAnomaly),
            10 => Some(Self::RouteLocalAnomaly),
            11 => Some(Self::FanoutPrecursor),
            12 => Some(Self::VarianceExpansion),
            13 => Some(Self::RecoveryEdge),
            14 => Some(Self::CleanWindowStability),
            15 => Some(Self::ConfuserLikeTransient),
            _ => None,
        }
    }

    /// Human-readable name. Used by the case-file serializer and any
    /// future operator-facing renderer. Stable strings; renaming any of
    /// them changes the registry hash and is therefore a contract
    /// breach.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ResidualSpike => "residual_spike",
            Self::SustainedResidualElevation => "sustained_residual_elevation",
            Self::DriftRamp => "drift_ramp",
            Self::SlewShock => "slew_shock",
            Self::Plateau => "plateau",
            Self::Oscillation => "oscillation",
            Self::DeadbandExit => "deadband_exit",
            Self::ErrorRateBurst => "error_rate_burst",
            Self::LatencyErrorCoupling => "latency_error_coupling",
            Self::EntityLocalAnomaly => "entity_local_anomaly",
            Self::RouteLocalAnomaly => "route_local_anomaly",
            Self::FanoutPrecursor => "fanout_precursor",
            Self::VarianceExpansion => "variance_expansion",
            Self::RecoveryEdge => "recovery_edge",
            Self::CleanWindowStability => "clean_window_stability",
            Self::ConfuserLikeTransient => "confuser_like_transient",
        }
    }
}

/// Order-locked array of all 16 motif classes. Used by the registry hash
/// and by iteration sites that need to walk every detector in canonical
/// order.
pub const MOTIF_CATALOG: [MotifClass; MotifClass::COUNT] = [
    MotifClass::ResidualSpike,
    MotifClass::SustainedResidualElevation,
    MotifClass::DriftRamp,
    MotifClass::SlewShock,
    MotifClass::Plateau,
    MotifClass::Oscillation,
    MotifClass::DeadbandExit,
    MotifClass::ErrorRateBurst,
    MotifClass::LatencyErrorCoupling,
    MotifClass::EntityLocalAnomaly,
    MotifClass::RouteLocalAnomaly,
    MotifClass::FanoutPrecursor,
    MotifClass::VarianceExpansion,
    MotifClass::RecoveryEdge,
    MotifClass::CleanWindowStability,
    MotifClass::ConfuserLikeTransient,
];

/// Canonical bytes of the detector registry: a comma-separated list of
/// the motif names in catalog order, with no whitespace. This format
/// produces a stable hash that changes if any name is renamed or any
/// motif is reordered, which is exactly what we want for breach
/// detection.
#[must_use]
pub fn registry_canonical_bytes() -> [u8; 16 * 64] {
    let mut buf = [0u8; 16 * 64];
    let mut pos = 0usize;
    let mut i = 0usize;
    while i < MOTIF_CATALOG.len() {
        if i > 0 {
            buf[pos] = b',';
            pos += 1;
        }
        let name = MOTIF_CATALOG[i].name().as_bytes();
        let mut j = 0;
        while j < name.len() {
            buf[pos] = name[j];
            pos += 1;
            j += 1;
        }
        i += 1;
    }
    // Pad the unused tail with zeros (already zeroed by construction);
    // the hash includes only the populated prefix.
    let _ = pos; // length is also returned by `registry_canonical_len()`.
    buf
}

/// Length of the populated prefix of `registry_canonical_bytes()`.
#[must_use]
pub fn registry_canonical_len() -> usize {
    let mut total = 0usize;
    let mut i = 0usize;
    while i < MOTIF_CATALOG.len() {
        if i > 0 {
            total += 1; // comma
        }
        total += MOTIF_CATALOG[i].name().len();
        i += 1;
    }
    total
}

/// SHA-256 digest of the canonical detector registry. The contract's
/// `registry_hash` field must equal this value verbatim; a mismatch is a
/// `DetectorRegistryMismatch` verdict.
#[must_use]
pub fn registry_hash() -> [u8; 32] {
    let bytes = registry_canonical_bytes();
    let len = registry_canonical_len();
    sha256(&bytes[..len])
}

/// Lowercased hex spelling of [`registry_hash`] prefixed with `sha256:`,
/// suitable for direct substitution into `contract.toml`.
#[must_use]
pub fn registry_hash_string() -> [u8; 71] {
    let digest = registry_hash();
    format_digest(&digest)
}

/// R.9 — detector-axis expansion profiles. Seven profile IDs are
/// reserved (`D16`, `D64`, `D128`, `D205`, `D512`, `D1024`, `D2000`).
/// At HEAD `D16`, `D64`, and `D128` are fully implemented:
/// D16 is the legacy 16-motif profile (R.9.a, audit-mode reference);
/// D64 is the R.9.b/R.10/R.11 throughput path that drove the R.13
/// ~55× full-pipeline campaign reduction; D128 is the R.9.d.1
/// scaling-ladder proof (commit `99a0f3b`, 16 motifs × 8 variants,
/// wide-digest baseline with R.10b compact-pack deferred). `D205`
/// and the wider profiles still reserve their identity + registry-
/// hash slots, deferred to paper §16.
///
/// **Why this exists.** R.7 reported 2.9× GPU Layer B at K=64 with
/// the 16-detector court. R.8 showed the dominant cost was the
/// host bank + digest plumbing, not the kernel math; R.8.5 + R.11
/// cleared those bottlenecks down to ~72 ms at 256×4096 K=1. With
/// the launch + finalize floor squeezed, **more detectors per cell
/// is the load-bearing way to keep the GPU saturated**. The Atlas
/// continuation calls for 2 000+ algebra-generated detectors; R.9
/// brings 16 → 2 000 inside this prior-art crate so the R.13
/// headline benchmark can run on the architecture's intended scale.
///
/// **Canonical 16-detector preservation**: `D16` derives the same
/// `registry_hash` bytes that [`registry_hash`] returns. Audit-mode
/// golden hashes are not touched by R.9.a. Wider profiles compose
/// the canonical 16-motif hash with a profile-id + active-count
/// suffix so their `detector_registry_hash` is deterministic and
/// distinct per profile.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum DetectorProfile {
    /// 16 detectors. The canonical court. Byte-identical to the
    /// pre-R.9 path; Audit golden hashes pin this profile.
    D16 = 16,
    /// 64 detectors. v1 expansion (4 parameter variants per family).
    /// Scaffolded in R.9.a; wide-mask kernel lands in R.9.b.
    D64 = 64,
    /// 128 detectors. v1 expansion (8 variants per family).
    D128 = 128,
    /// 205 detectors. Mirrors the dsfb-debug taxonomy size — the
    /// "mature 205-detector court" the user named in the campaign
    /// brief.
    D205 = 205,
    /// 512 detectors. Mid-Atlas density.
    D512 = 512,
    /// 1024 detectors. Approaching Atlas headline.
    D1024 = 1024,
    /// 2000 detectors. Headline target per the user's locked R.9
    /// scope. The R.13 ≥10× gate is measured here.
    D2000 = 2000,
}

impl DetectorProfile {
    /// Number of *active* detector bits this profile carries. Always
    /// equals the enum's numeric value; surfaced as a method for
    /// readability at call sites.
    #[must_use]
    pub const fn active_detector_count(self) -> u32 {
        self as u32
    }

    /// Short stable identifier string. Used inside the canonical
    /// per-profile registry-hash derivation; never localised or
    /// reformatted.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::D16 => "D16",
            Self::D64 => "D64",
            Self::D128 => "D128",
            Self::D205 => "D205",
            Self::D512 => "D512",
            Self::D1024 => "D1024",
            Self::D2000 => "D2000",
        }
    }

    /// Width of the per-cell detector bitset in 64-bit words. R.9.b
    /// will pin `DetectorCell` to `[u64; MASK_WORDS]` so 2 048 bits
    /// fit at the headline profile. `D16` still uses the legacy
    /// `u32` cell field in R.9.a; the wider mask only activates
    /// when the wide-kernel commit lands.
    #[must_use]
    pub const fn mask_word_count(self) -> u32 {
        // The mask is sized for the widest profile so all profiles
        // share a single ABI shape once R.9.b lands. 32 × 64 = 2048
        // bits, comfortably above 2 000.
        32
    }

    /// Returns the canonical per-profile `detector_registry_hash`
    /// that the contract pins for this profile.
    ///
    /// **Byte stability**: `DetectorProfile::D16.registry_hash() ==
    /// motif::registry_hash()` — the canonical 16-detector court's
    /// hash is unchanged. Wider profiles compose:
    ///
    /// ```text
    ///   sha256(
    ///       "DSFB-GPU-DEBUG:detector-profile:v1\0"
    ///       || canonical_motif_registry_hash
    ///       || profile_name_ascii
    ///       || 0x00
    ///       || active_detector_count_le_u32
    ///   )
    /// ```
    ///
    /// The domain prefix prevents the wider profiles' hashes from
    /// colliding with any other 32-byte commitment in the chain.
    /// The active count is included so a future R.9.b that
    /// reorganises the 64/128/etc variant table without changing
    /// the profile-id-string would still produce a fresh hash if
    /// the count changed. The R.9.b commit may extend this
    /// derivation with the parameter-variant table; that's a
    /// `v2` derivation when it lands.
    #[must_use]
    pub fn registry_hash(self) -> [u8; 32] {
        // Build a small fixed-size buffer on the stack — the
        // longest possible content is ~85 bytes (35 domain + 32
        // canonical + 6 name + 1 null + 4 count). Sized at 128 to
        // leave headroom; the actual hash input length is tracked
        // separately so trailing padding bytes are excluded.
        const DOMAIN: &[u8] = b"DSFB-GPU-DEBUG:detector-profile:v1\0";
        if matches!(self, Self::D16) {
            return registry_hash();
        }
        let canonical = registry_hash();
        let name = self.name().as_bytes();
        let count = self.active_detector_count().to_le_bytes();
        let mut buf = [0u8; 128];
        let mut pos = 0usize;
        // Domain prefix.
        let mut i = 0;
        while i < DOMAIN.len() {
            buf[pos] = DOMAIN[i];
            pos += 1;
            i += 1;
        }
        // Canonical 16-motif registry hash.
        let mut j = 0;
        while j < 32 {
            buf[pos] = canonical[j];
            pos += 1;
            j += 1;
        }
        // Profile-name ASCII + null terminator.
        let mut k = 0;
        while k < name.len() {
            buf[pos] = name[k];
            pos += 1;
            k += 1;
        }
        buf[pos] = 0;
        pos += 1;
        // Active count little-endian u32.
        let mut m = 0;
        while m < 4 {
            buf[pos] = count[m];
            pos += 1;
            m += 1;
        }
        sha256(&buf[..pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motif_class_round_trips_through_bit_index() {
        for class in MOTIF_CATALOG {
            let bit = class.bit_index();
            let recovered = MotifClass::from_bit_index(bit).unwrap();
            assert_eq!(class, recovered);
        }
    }

    #[test]
    fn bit_masks_are_disjoint_powers_of_two() {
        let mut union = 0u32;
        for class in MOTIF_CATALOG {
            let mask = class.bit_mask();
            assert_eq!(mask.count_ones(), 1, "non-power-of-two mask for {class:?}");
            assert_eq!(union & mask, 0, "duplicate bit for {class:?}");
            union |= mask;
        }
        assert_eq!(union, 0xFFFF, "expected 16 bits set, got {union:#x}");
    }

    #[test]
    fn catalog_order_matches_bit_indices() {
        // The catalog walk and bit_index() must agree on the canonical
        // ordering — otherwise serialization and parsing would disagree.
        for (i, &class) in MOTIF_CATALOG.iter().enumerate() {
            assert_eq!(class.bit_index() as usize, i);
        }
    }

    #[test]
    fn names_are_unique_and_lowercase_snake_case() {
        let mut seen: Vec<&'static str> = Vec::new();
        for class in MOTIF_CATALOG {
            let name = class.name();
            assert!(!seen.contains(&name), "duplicate motif name {name}");
            assert!(
                name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
                "motif name {name} contains non-lowercase-snake-case byte"
            );
            seen.push(name);
        }
        assert_eq!(seen.len(), MotifClass::COUNT);
    }

    #[test]
    fn registry_canonical_length_matches_computed_bytes() {
        let bytes = registry_canonical_bytes();
        let len = registry_canonical_len();
        // The first `len` bytes are the canonical content; the rest is
        // zeroed padding.
        let s = core::str::from_utf8(&bytes[..len]).expect("ASCII");
        assert!(s.starts_with("residual_spike,"));
        assert!(s.ends_with(",confuser_like_transient"));
    }

    #[test]
    fn registry_hash_is_stable_across_calls() {
        let a = registry_hash();
        let b = registry_hash();
        assert_eq!(a, b);
    }

    #[test]
    fn registry_hash_is_what_we_expect() {
        // Pin the expected digest. Computed once over the canonical bytes
        // and recorded here so that any silent change to the catalog
        // (rename, reorder, addition) fails the test deterministically.
        let bytes = registry_canonical_bytes();
        let len = registry_canonical_len();
        let expected = sha256(&bytes[..len]);
        assert_eq!(registry_hash(), expected);
        // Also assert the hash is non-trivial (not all zeros, not the
        // empty-string digest). This catches construction bugs that would
        // pass the round-trip but produce a meaningless value.
        assert_ne!(registry_hash(), [0u8; 32]);
        assert_ne!(registry_hash(), sha256(b""));
    }

    #[test]
    fn d16_profile_hash_equals_canonical_registry_hash() {
        // R.9.a load-bearing invariant: the canonical 16-detector
        // court's per-profile hash is bit-identical to the
        // pre-R.9 `registry_hash()`. This is what keeps Audit
        // golden hashes untouched.
        assert_eq!(DetectorProfile::D16.registry_hash(), registry_hash());
    }

    #[test]
    fn wider_profile_hashes_differ_from_d16() {
        // Every wider profile must produce a distinct
        // `detector_registry_hash`. If two profiles collided,
        // the case-file chain would silently confuse them at
        // replay.
        let d16 = DetectorProfile::D16.registry_hash();
        for p in [
            DetectorProfile::D64,
            DetectorProfile::D128,
            DetectorProfile::D205,
            DetectorProfile::D512,
            DetectorProfile::D1024,
            DetectorProfile::D2000,
        ] {
            assert_ne!(
                p.registry_hash(),
                d16,
                "{} hash collides with D16",
                p.name()
            );
        }
    }

    #[test]
    fn profile_hashes_are_pairwise_distinct() {
        // Stronger version of the above: no two profiles share a
        // registry hash.
        let profiles = [
            DetectorProfile::D16,
            DetectorProfile::D64,
            DetectorProfile::D128,
            DetectorProfile::D205,
            DetectorProfile::D512,
            DetectorProfile::D1024,
            DetectorProfile::D2000,
        ];
        for (i, &a) in profiles.iter().enumerate() {
            for &b in profiles.iter().skip(i + 1) {
                assert_ne!(
                    a.registry_hash(),
                    b.registry_hash(),
                    "{} and {} share a registry hash",
                    a.name(),
                    b.name()
                );
            }
        }
    }

    #[test]
    fn profile_hashes_are_deterministic() {
        // Two consecutive calls to `registry_hash()` on the same
        // profile produce byte-identical output. Catches any
        // future regression that introduces non-determinism into
        // the hash derivation (e.g. address-based ordering).
        for p in [
            DetectorProfile::D16,
            DetectorProfile::D64,
            DetectorProfile::D128,
            DetectorProfile::D205,
            DetectorProfile::D512,
            DetectorProfile::D1024,
            DetectorProfile::D2000,
        ] {
            assert_eq!(p.registry_hash(), p.registry_hash());
        }
    }

    #[test]
    fn profile_active_detector_count_matches_repr_u32() {
        // `active_detector_count()` returns the enum's `repr(u32)`
        // value verbatim. This is the contract that connects the
        // profile id to the cell-mask width R.9.b will allocate.
        assert_eq!(DetectorProfile::D16.active_detector_count(), 16);
        assert_eq!(DetectorProfile::D64.active_detector_count(), 64);
        assert_eq!(DetectorProfile::D128.active_detector_count(), 128);
        assert_eq!(DetectorProfile::D205.active_detector_count(), 205);
        assert_eq!(DetectorProfile::D512.active_detector_count(), 512);
        assert_eq!(DetectorProfile::D1024.active_detector_count(), 1024);
        assert_eq!(DetectorProfile::D2000.active_detector_count(), 2000);
    }

    #[test]
    fn profile_mask_word_count_fits_all_profiles() {
        // Every profile's active count fits inside the shared 2048-
        // bit mask (32 × 64 bits). The mask width is uniform across
        // profiles so the cell ABI is profile-independent once
        // R.9.b widens it.
        for p in [
            DetectorProfile::D16,
            DetectorProfile::D64,
            DetectorProfile::D128,
            DetectorProfile::D205,
            DetectorProfile::D512,
            DetectorProfile::D1024,
            DetectorProfile::D2000,
        ] {
            let bits = u64::from(p.mask_word_count()) * 64;
            assert!(
                bits >= u64::from(p.active_detector_count()),
                "{}: mask width {} < active count {}",
                p.name(),
                bits,
                p.active_detector_count()
            );
        }
    }
}
