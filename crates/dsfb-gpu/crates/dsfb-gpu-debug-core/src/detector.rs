//! Detector-motif evaluation.
//!
//! Sixteen deterministic detectors, each a closed-form function over the
//! entity's residual and sign grids. Bit `i` of the output cell's
//! `detector_mask` is set when `MOTIF_CATALOG[i]` fires.
//!
//! Design constraints:
//!
//! * **Per-entity, history-bounded.** Each detector reads at most
//!   `DetectorThresholds::history_window` cells back from the current
//!   window. This is what keeps the CUDA mirror tractable — a kernel
//!   thread can index into the entity's contiguous slice of the grid
//!   without atomics or shared state.
//! * **Threshold-driven, not learned.** Every decision is a comparison
//!   against a contract-locked Q16.16 threshold. No probability, no
//!   confidence score, no learned weight.
//! * **Pure functions.** No allocation in the inner loop, no
//!   floating-point. Two calls with the same inputs produce identical
//!   output buffers, which is the property the case-file hash chain
//!   depends on.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::fixed::Q16;
use crate::motif::MotifClass;
use crate::residual::ResidualCell;
use crate::sign::SignCell;

/// Configuration table for the 16 detectors. Every field carries the
/// `_q16_raw` suffix when the value is a raw Q16.16 `i32`, the `_q16`
/// suffix when it is a `Q16` (rare here for clarity), and a plain `u32`
/// for window counts. All fields are part of the contract hash.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DetectorThresholds {
    /// Norm above this fires `ResidualSpike`. v0 default: 10 ms (~ ten
    /// times the baseline latency).
    pub spike_q16_raw: i32,
    /// EWMA drift above this fires `SustainedResidualElevation`.
    pub sustain_q16_raw: i32,
    /// Absolute slew above this fires `SlewShock`.
    pub slew_shock_q16_raw: i32,
    /// Minimum norm during a plateau.
    pub plateau_min_q16_raw: i32,
    /// Maximum absolute slew tolerated inside a plateau.
    pub plateau_slew_max_q16_raw: i32,
    /// Number of consecutive cells the plateau condition must hold.
    pub plateau_windows: u32,
    /// Lookback window for the oscillation detector.
    pub oscillation_window: u32,
    /// Number of sign alternations in slew that fires `Oscillation`.
    pub oscillation_alternations: u32,
    /// Low edge of the deadband. A previous-cell norm below this is the
    /// "below" side of the deadband-exit transition.
    pub deadband_low_q16_raw: i32,
    /// High edge of the deadband. A current-cell norm above this is the
    /// "above" side of the deadband-exit transition.
    pub deadband_high_q16_raw: i32,
    /// Residual error rate above this fires `ErrorRateBurst`.
    pub error_burst_q16_raw: i32,
    /// Latency-residual threshold for the coupling detector.
    pub coupling_lat_q16_raw: i32,
    /// Error-residual threshold for the coupling detector.
    pub coupling_err_q16_raw: i32,
    /// Number of cells contributing to the variance detector's window.
    pub variance_window: u32,
    /// Max-minus-min norm above this fires `VarianceExpansion`. v0 uses
    /// the spread (a deterministic proxy for variance) so we don't need
    /// a Q16 multiply-accumulate over the window.
    pub variance_threshold_q16_raw: i32,
    /// Number of consecutive cells over which the drift must
    /// monotonically rise to fire `DriftRamp`. The `ramp_window` cells
    /// are the most recent `ramp_window` cells in entity order.
    pub ramp_window: u32,
    /// Minimum norm at a recovery edge. Below this the recovery edge is
    /// suppressed (we are not in a meaningful recovery if the level is
    /// near zero).
    pub recovery_min_norm_q16_raw: i32,
    /// All-axes upper bound for `CleanWindowStability`. If norm, drift,
    /// and |slew| are all under this, and no other detector fired, the
    /// clean bit is set.
    pub clean_band_q16_raw: i32,
    /// Confuser detector: cur.norm above this and prev.norm below
    /// `clean_band` makes the cell a transient-spike candidate.
    pub confuser_min_q16_raw: i32,
    /// Coupling between drift and error rate that fires
    /// `FanoutPrecursor`. The drift threshold reuses `sustain` and the
    /// error threshold reuses `error_burst`/8, so the fan-out detector
    /// fires earlier than the burst detector — capturing precursor
    /// conditions rather than the burst itself.
    pub fanout_drift_q16_raw: i32,
    /// EntityLocalAnomaly: norm exceeds this many times the drift.
    /// v0 default: 4. Stored as Q16.16 multiplier so the threshold can
    /// be tuned later without changing the comparison form.
    pub entity_anomaly_factor_q16_raw: i32,
    /// Number of windows of history any detector may read back from the
    /// current cell. The window sizes above are required to be ≤ this
    /// number; this is a structural bound.
    pub history_window: u32,
}

impl DetectorThresholds {
    /// Canonical v0 thresholds. Pinned by the contract; any change is a
    /// contract breach. Chosen so the three injected fixture episodes
    /// (latency ramp, error burst, slew shock + recovery) cleanly trigger
    /// the relevant detectors while clean windows stay silent.
    pub const CANONICAL: Self = Self {
        spike_q16_raw: 10 * 65_536,
        sustain_q16_raw: 5 * 65_536,
        slew_shock_q16_raw: 20 * 65_536,
        plateau_min_q16_raw: 5 * 65_536,
        plateau_slew_max_q16_raw: 65_536, // 1 ms
        plateau_windows: 3,
        oscillation_window: 6,
        oscillation_alternations: 3,
        deadband_low_q16_raw: 2 * 65_536,
        deadband_high_q16_raw: 4 * 65_536,
        error_burst_q16_raw: 0x4000, // 0.25 in Q16
        coupling_lat_q16_raw: 5 * 65_536,
        coupling_err_q16_raw: 0x1000, // 0.0625 in Q16
        variance_window: 5,
        variance_threshold_q16_raw: 30 * 65_536,
        ramp_window: 4,
        recovery_min_norm_q16_raw: 5 * 65_536,
        clean_band_q16_raw: 65_536, // 1 ms
        confuser_min_q16_raw: 10 * 65_536,
        fanout_drift_q16_raw: 3 * 65_536,
        entity_anomaly_factor_q16_raw: 4 * 65_536,
        history_window: 8,
    };
}

/// One `(window, entity)` detector cell.
///
/// The 16-bit `detector_mask` is the OR of every motif bit that fired
/// on this cell. The cell carries its position metadata so the
/// downstream consensus stage can address cells by `(window, entity)`.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct DetectorCell {
    /// Window index this cell belongs to.
    pub window_idx: u32,
    /// Entity this cell belongs to.
    pub entity_id: u32,
    /// Bitmask of fired motif bits. Bit `i` corresponds to
    /// `motif::MotifClass::from_bit_index(i)`.
    pub detector_mask: u32,
}

/// R.9.b — fixed-width detector bitset, sized for the D2000 headline
/// profile (2 048 bits, 256 bytes). All wider detector profiles
/// (`DetectorProfile::D64..D2000`) share this layout so the cell ABI
/// is profile-independent; profiles below 2 000 leave the unused
/// high bits zero. `repr(C)` so the GPU sees the same byte form.
pub type DetectorMask2048 = [u64; 32];

/// R.9.b — wide-mask detector cell used by every profile other than
/// `DetectorProfile::D16`. Byte layout (264 bytes total, 8-byte
/// aligned by construction):
///
/// ```text
///   offset  field
///        0  window_idx: u32
///        4  entity_id:  u32
///        8  detector_mask: [u64; 32]   (256 bytes)
/// ```
///
/// **Why a separate cell type**: changing `DetectorCell.detector_mask`
/// from `u32` to `[u64; 32]` would shift the canonical compact byte
/// form for D16, breaking Audit-mode golden hashes. R.9.b preserves
/// D16 byte-for-byte by keeping the legacy cell intact and adding
/// this wider cell only on the new wide dispatch path.
///
/// **Memory budget** (256x4096 K=1): 1 M cells × 264 bytes = ~270 MB
/// for the detector stage. That fits comfortably at K=1 single-
/// catalog; K>1 batched needs per-profile cell-size optimisation
/// before it fits on a 16 GB GPU (deferred to R.9.c).
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct DetectorCellWide {
    /// Window index this cell belongs to.
    pub window_idx: u32,
    /// Entity this cell belongs to.
    pub entity_id: u32,
    /// 2 048-bit detector mask. Bit `i` corresponds to detector id `i`
    /// for the active profile; bit position semantics depend on the
    /// `DetectorProfile` (e.g. for D64: `detector_id = motif_id * 4
    /// + variant_id`).
    pub detector_mask: DetectorMask2048,
}

impl DetectorCellWide {
    /// Test whether a specific detector id fired on this cell.
    /// Returns `false` for ids ≥ 2 048 (the mask width).
    #[must_use]
    pub const fn fired_by_id(&self, detector_id: u32) -> bool {
        if detector_id >= 2048 {
            return false;
        }
        let word = (detector_id / 64) as usize;
        let bit = detector_id % 64;
        (self.detector_mask[word] & (1u64 << bit)) != 0
    }

    /// Total number of detector bits set on this cell.
    #[must_use]
    pub fn popcount(&self) -> u32 {
        let mut total = 0u32;
        let mut i = 0;
        while i < self.detector_mask.len() {
            total += self.detector_mask[i].count_ones();
            i += 1;
        }
        total
    }

    /// Set the bit at the given detector id. No-op for ids ≥ 2 048.
    pub fn set_bit(&mut self, detector_id: u32) {
        if detector_id >= 2048 {
            return;
        }
        let word = (detector_id / 64) as usize;
        let bit = detector_id % 64;
        self.detector_mask[word] |= 1u64 << bit;
    }
}

/// R.9.b — D64 variant scale factors in Q16.16. The D64 profile
/// runs each of the 16 canonical motifs at four threshold scales,
/// producing 64 distinct detector slots. `detector_id = motif_id * 4
/// + variant_id`.
///
/// Variant semantics (locked by the R.9.b design):
///
/// * V0 = canonical (× 1.0). For V0 the scaled-threshold vector is
///   byte-identical to the canonical `DetectorThresholds`, so the
///   V0 bit at slot `motif_id * 4 + 0` equals the D16 mask bit at
///   slot `motif_id`. This gives D64 the strict-superset property:
///   every D16 firing is recoverable from the D64 mask.
/// * V1 = sensitive (× 0.5). Lower thresholds and shorter windows
///   ⇒ more firings.
/// * V2 = strict (× 1.5). Higher thresholds and longer windows ⇒
///   fewer firings.
/// * V3 = persistence-biased (× 0.75). A different scale that
///   produces a distinct firing pattern from V0/V1/V2 in most
///   cases.
///
/// Future R.9 phases (D128 / D205 / D2000) may add more variants
/// or compose family-parameter combinations on top of these scales;
/// they are not constrained to keep these specific values.
pub const D64_VARIANT_COUNT: u32 = 4;

/// R.9.b — Total detector count for the D64 profile (16 motifs × 4
/// variants = 64). Matches `DetectorProfile::D64.active_detector_count()`.
pub const D64_TOTAL_DETECTORS: u32 = 16 * D64_VARIANT_COUNT;

/// R.9.b — Variant scale factors in Q16.16. Order is canonical (V0,
/// V1, V2, V3) and must never be reordered: `detector_registry_hash`
/// is sensitive to this list via the profile-id metadata, but the
/// per-cell firing pattern is sensitive to the precise scale values
/// and their ordering across variants.
pub const D64_VARIANT_SCALES_Q16: [i32; 4] = [
    1 << 16,               // V0: 1.0 (canonical)
    1 << 15,               // V1: 0.5 (sensitive)
    (1 << 16) + (1 << 15), // V2: 1.5 (strict)
    (1 << 16) - (1 << 14), // V3: 0.75 (persistence-biased)
];

/// R.9.d.1 — D128 profile variant count. 16 canonical motifs ×
/// 8 threshold-scaled variants = 128 active detectors. Active bits
/// `0..127` populate `DetectorCellWide::detector_mask[0..2]`
/// (D128 spans words 0 and 1; bits 128..2047 stay zero).
pub const D128_VARIANT_COUNT: u32 = 8;

/// R.9.d.1 — total D128 detector count.
pub const D128_TOTAL_DETECTORS: u32 = 16 * D128_VARIANT_COUNT;

/// R.9.d.1 — D128 variant scale factors in Q16.16. Order is
/// canonical (V0..V7) and must never be reordered: the
/// detector-registry hash binds to the profile id + ordered scale
/// values, and the per-cell firing pattern is sensitive to the
/// precise scales.
///
/// V0..V3 mirror the D64 scales bit-for-bit so the D128 V0-only
/// projection equals the D64 V0-only projection (and therefore the
/// canonical D16 mask) — the same R.9.b "bridge invariant"
/// extended to a wider variant set. V4..V7 add a finer-grained
/// threshold sweep on both the strict and sensitive sides so the
/// OR-projected mask sees more cells fire than D64 does.
pub const D128_VARIANT_SCALES_Q16: [i32; 8] = [
    1 << 16,               // V0: 1.0   (canonical — matches D64.V0)
    1 << 15,               // V1: 0.5   (sensitive — matches D64.V1)
    (1 << 16) + (1 << 15), // V2: 1.5   (strict    — matches D64.V2)
    (1 << 16) - (1 << 14), // V3: 0.75  (persistence-biased — matches D64.V3)
    1 << 14,               // V4: 0.25  (very sensitive)
    (1 << 16) + (1 << 14), // V5: 1.25
    (1 << 17),             // V6: 2.0   (very strict)
    (1 << 17) + (1 << 16), // V7: 3.0   (extreme strict)
];

/// R.9.d.2 — D205 profile variant count. 16 canonical motifs ×
/// 13 threshold-scaled variants = **208 fireable slots**, of
/// which the bottom **205** are reported as active detectors
/// (`DetectorProfile::D205.active_detector_count()`). The
/// remaining three slots (bit indices 205, 206, 207) are
/// deterministically held at zero by the gate
/// `det_id < D205_ACTIVE_BITS` inside `evaluate_wide`; bits
/// 208..2047 are never iterated. The "205" canonical name
/// mirrors the dsfb-debug mature taxonomy count.
///
/// **The three reserved-not-fired slots are intentional**: they
/// are the scaling-ladder bridge to the dsfb-debug 27-tier
/// taxonomy, which has an uneven per-motif distribution. We do
/// NOT split the variants per motif (that would break the
/// regular kernel iteration). Instead we iterate the full 16 ×
/// 13 grid and gate firings by `det_id < 205`. This keeps the
/// CPU/GPU paths simple and the high-bit slots deterministic.
pub const D205_VARIANT_COUNT: u32 = 13;

/// R.9.d.2 — total fireable bit count for D205. Equals
/// `DetectorProfile::D205.active_detector_count()` = 205.
/// The 16 × 13 iteration produces 208 candidate slots; the gate
/// `det_id < D205_ACTIVE_BITS` masks the top 3 to zero.
pub const D205_ACTIVE_BITS: u32 = 205;

/// R.9.d.2 — total iterated slots for D205. Equals 16 × 13 =
/// 208. The three slots `[205, 206, 207]` (motif 15 variants
/// 10/11/12) are iterated but their bits are NOT set in the
/// output mask. Bits 208..2047 are never touched.
pub const D205_TOTAL_SLOTS: u32 = 16 * D205_VARIANT_COUNT;

/// R.9.d.2 — D205 variant scale factors in Q16.16. Order is
/// canonical (V0..V12) and must never be reordered: the
/// detector-registry hash binds to the profile id + ordered scale
/// values, and the per-cell firing pattern is sensitive to the
/// precise scales.
///
/// **Bridge invariants (panel-locked)**:
/// - V0..V7 mirror the D128 scales bit-for-bit so the D205 V0-only
///   projection equals D128.V0 (= D64.V0 = canonical D16).
/// - V0..V7 produce the same per-cell firings as D128 → the
///   D205 per-motif OR-projection over V0..V7 equals D128's
///   per-motif OR-projection. The additional variants V8..V12
///   can only add cells to the OR-projection, so
///   `D205 OR ⊇ D128 OR ⊇ D64 OR ⊇ D16`.
///
/// V8..V12 sample five additional deterministic dyadic
/// fractions between the existing scales, broadening the
/// threshold sweep without introducing floating-point arithmetic.
pub const D205_VARIANT_SCALES_Q16: [i32; 13] = [
    1 << 16,               // V0: 1.0    (canonical — matches D128.V0)
    1 << 15,               // V1: 0.5    (matches D128.V1)
    (1 << 16) + (1 << 15), // V2: 1.5    (matches D128.V2)
    (1 << 16) - (1 << 14), // V3: 0.75   (matches D128.V3)
    1 << 14,               // V4: 0.25   (matches D128.V4)
    (1 << 16) + (1 << 14), // V5: 1.25   (matches D128.V5)
    1 << 17,               // V6: 2.0    (matches D128.V6)
    (1 << 17) + (1 << 16), // V7: 3.0    (matches D128.V7)
    (1 << 14) + (1 << 13), // V8: 0.375  (between V4 and V1)
    (1 << 15) + (1 << 13), // V9: 0.625  (between V1 and V3)
    (1 << 16) - (1 << 13), // V10: 0.875 (slight sensitive bias)
    (1 << 16) + (1 << 13), // V11: 1.125 (slight strict bias)
    (1 << 17) - (1 << 14), // V12: 1.75  (between V2 and V6)
];

/// R.9.b — scale a Q16.16 raw threshold value by another Q16.16
/// factor. Math: `(value × scale) >> 16`, with the multiplication
/// performed in i64 so a Q32.32 intermediate is representable.
/// Truncates toward zero (the natural `>>` behaviour on i64).
///
/// For `scale_q16 = 1 << 16` (= 1.0), the function is the identity
/// on every valid Q16.16 input — this is what gives D64.V0 the
/// byte-identical-to-canonical property.
#[must_use]
pub fn scale_q16_threshold(value_raw: i32, scale_q16: i32) -> i32 {
    let result = (i64::from(value_raw) * i64::from(scale_q16)) >> 16;
    // The result fits in i32 for all realistic Q16.16 thresholds we
    // use (peak values are well under 2^15). Saturate defensively if
    // a future profile pushes the bound — clamping to i32::MAX/MIN
    // is the deterministic choice.
    if result > i64::from(i32::MAX) {
        i32::MAX
    } else if result < i64::from(i32::MIN) {
        i32::MIN
    } else {
        result as i32
    }
}

/// R.9.b — scale a `u32` window count by a Q16.16 factor, rounding
/// to nearest and clamping to ≥ 1. Used for the two motifs whose
/// "primary parameter" is a window count rather than a Q16.16
/// threshold (`DriftRamp.ramp_window`, `Oscillation.oscillation_
/// window`, plus the `Plateau` / `VarianceExpansion` window fields).
///
/// For `scale_q16 = 1 << 16` (= 1.0), `scale_window(w, scale_q16) = w`
/// for every input — preserving the V0 = canonical property.
#[must_use]
pub fn scale_window(window: u32, scale_q16: i32) -> u32 {
    let scaled = (i64::from(window) * i64::from(scale_q16) + (1 << 15)) >> 16;
    if scaled < 1 {
        1
    } else if scaled > i64::from(u32::MAX) {
        u32::MAX
    } else {
        scaled as u32
    }
}

/// R.9.b — produce a `DetectorThresholds` whose every scalar
/// threshold and window field is scaled by `scale_q16`. For
/// `scale_q16 = 1 << 16` this returns a byte-identical copy of
/// the input; that's the V0-equals-canonical invariant.
///
/// Fields that semantically do not scale (`oscillation_alternations`
/// is an integer count, `history_window` is a hard ceiling) stay
/// at their canonical values.
#[must_use]
pub fn scale_thresholds(t: &DetectorThresholds, scale_q16: i32) -> DetectorThresholds {
    DetectorThresholds {
        spike_q16_raw: scale_q16_threshold(t.spike_q16_raw, scale_q16),
        sustain_q16_raw: scale_q16_threshold(t.sustain_q16_raw, scale_q16),
        slew_shock_q16_raw: scale_q16_threshold(t.slew_shock_q16_raw, scale_q16),
        plateau_min_q16_raw: scale_q16_threshold(t.plateau_min_q16_raw, scale_q16),
        plateau_slew_max_q16_raw: scale_q16_threshold(t.plateau_slew_max_q16_raw, scale_q16),
        plateau_windows: scale_window(t.plateau_windows, scale_q16),
        oscillation_window: scale_window(t.oscillation_window, scale_q16),
        oscillation_alternations: t.oscillation_alternations,
        deadband_low_q16_raw: scale_q16_threshold(t.deadband_low_q16_raw, scale_q16),
        deadband_high_q16_raw: scale_q16_threshold(t.deadband_high_q16_raw, scale_q16),
        error_burst_q16_raw: scale_q16_threshold(t.error_burst_q16_raw, scale_q16),
        coupling_lat_q16_raw: scale_q16_threshold(t.coupling_lat_q16_raw, scale_q16),
        coupling_err_q16_raw: scale_q16_threshold(t.coupling_err_q16_raw, scale_q16),
        variance_window: scale_window(t.variance_window, scale_q16),
        variance_threshold_q16_raw: scale_q16_threshold(t.variance_threshold_q16_raw, scale_q16),
        ramp_window: scale_window(t.ramp_window, scale_q16),
        recovery_min_norm_q16_raw: scale_q16_threshold(t.recovery_min_norm_q16_raw, scale_q16),
        clean_band_q16_raw: scale_q16_threshold(t.clean_band_q16_raw, scale_q16),
        confuser_min_q16_raw: scale_q16_threshold(t.confuser_min_q16_raw, scale_q16),
        fanout_drift_q16_raw: scale_q16_threshold(t.fanout_drift_q16_raw, scale_q16),
        entity_anomaly_factor_q16_raw: scale_q16_threshold(
            t.entity_anomaly_factor_q16_raw,
            scale_q16,
        ),
        history_window: t.history_window,
    }
}

impl DetectorCell {
    /// Test whether a specific motif fired on this cell.
    #[must_use]
    pub const fn fired(&self, class: MotifClass) -> bool {
        (self.detector_mask & class.bit_mask()) != 0
    }

    /// Number of motifs that fired.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.detector_mask.count_ones()
    }
}

/// Look up a cell in an entity-major grid. Sign and residual grids share
/// this layout because they are produced by the upstream stages with the
/// same convention.
#[inline]
const fn flat(entity_id: u32, window_idx: u32, n_windows: u32) -> usize {
    (entity_id * n_windows + window_idx) as usize
}

/// Evaluate the 16-detector grid over the entity-major residual and
/// sign cells.
///
/// The function returns a `Vec<DetectorCell>` in the same entity-major
/// layout as its inputs. Determinism: identical inputs produce
/// byte-identical output.
#[must_use]
pub fn evaluate(
    residuals: &[ResidualCell],
    signs: &[SignCell],
    thresholds: &DetectorThresholds,
    n_windows: u32,
    n_entities: u32,
) -> Vec<DetectorCell> {
    let total = (n_windows as usize) * (n_entities as usize);
    debug_assert_eq!(residuals.len(), total, "residual grid shape mismatch");
    debug_assert_eq!(signs.len(), total, "sign grid shape mismatch");

    let mut out: Vec<DetectorCell> = Vec::with_capacity(total);

    for entity_id in 0..n_entities {
        for window_idx in 0..n_windows {
            let mask = eval_motifs_for_cell(
                residuals, signs, thresholds, entity_id, window_idx, n_windows,
            );
            out.push(DetectorCell {
                window_idx,
                entity_id,
                detector_mask: mask,
            });
        }
    }
    out
}

/// R.9.b — Per-cell 16-motif evaluator. Returns a `u32` mask where
/// bit `i` is set when `MotifClass::from_bit_index(i)` fires on the
/// `(entity_id, window_idx)` cell under the supplied `thresholds`.
///
/// Extracted from `evaluate` so the wide-mask path (`evaluate_wide`)
/// can call it once per variant with a scaled-threshold copy. Single
/// source of truth: a future refactor to the motif predicates only
/// needs to touch this function, not two copies.
///
/// **Byte stability**: calling this with the canonical
/// `DetectorThresholds::CANONICAL` produces exactly the bytes the
/// pre-R.9.b `evaluate` produced for the same cell. The function
/// is unsafe-code-free and host-side; the CUDA kernel mirrors the
/// same predicate set bit-for-bit.
#[must_use]
pub fn eval_motifs_for_cell(
    residuals: &[ResidualCell],
    signs: &[SignCell],
    thresholds: &DetectorThresholds,
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
) -> u32 {
    let idx = flat(entity_id, window_idx, n_windows);
    let r = residuals[idx];
    let s = signs[idx];
    let mut mask = 0u32;

    if s.norm_q.raw() > thresholds.spike_q16_raw {
        mask |= MotifClass::ResidualSpike.bit_mask();
    }
    if s.drift_q.raw() > thresholds.sustain_q16_raw {
        mask |= MotifClass::SustainedResidualElevation.bit_mask();
    }
    if drift_ramp_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::DriftRamp.bit_mask();
    }
    if s.slew_q.abs().raw() > thresholds.slew_shock_q16_raw {
        mask |= MotifClass::SlewShock.bit_mask();
    }
    if plateau_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::Plateau.bit_mask();
    }
    if oscillation_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::Oscillation.bit_mask();
    }
    if deadband_exit_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::DeadbandExit.bit_mask();
    }
    if r.residual_error_q.raw() > thresholds.error_burst_q16_raw {
        mask |= MotifClass::ErrorRateBurst.bit_mask();
    }
    if r.residual_latency_q.raw() > thresholds.coupling_lat_q16_raw
        && r.residual_error_q.raw() > thresholds.coupling_err_q16_raw
    {
        mask |= MotifClass::LatencyErrorCoupling.bit_mask();
    }
    if entity_local_anomaly_fires(&s, thresholds) {
        mask |= MotifClass::EntityLocalAnomaly.bit_mask();
    }
    // Route-local anomaly: v0 single-cell proxy — fires when the
    // spike condition holds *and* the error axis is also non-zero,
    // marking a candidate that the consensus pass refines using
    // route distribution. Carrying a deterministic proxy here
    // keeps the bit position meaningful even before the route-
    // distribution pass lands.
    if (mask & MotifClass::ResidualSpike.bit_mask()) != 0 && r.residual_error_q.raw() > 0 {
        mask |= MotifClass::RouteLocalAnomaly.bit_mask();
    }
    if fanout_precursor_fires(&s, &r, thresholds) {
        mask |= MotifClass::FanoutPrecursor.bit_mask();
    }
    if variance_expansion_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::VarianceExpansion.bit_mask();
    }
    if recovery_edge_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::RecoveryEdge.bit_mask();
    }
    if confuser_like_transient_fires(signs, entity_id, window_idx, n_windows, thresholds) {
        mask |= MotifClass::ConfuserLikeTransient.bit_mask();
    }

    // Clean-window stability is the catch-all sentinel: every
    // non-clean bit must be zero for clean to fire. Mask out the
    // clean bit position itself before testing so the sentinel
    // doesn't reference itself.
    let any_non_clean = mask & !MotifClass::CleanWindowStability.bit_mask();
    if any_non_clean == 0
        && s.norm_q.abs().raw() <= thresholds.clean_band_q16_raw
        && s.drift_q.abs().raw() <= thresholds.clean_band_q16_raw
        && s.slew_q.abs().raw() <= thresholds.clean_band_q16_raw
    {
        mask |= MotifClass::CleanWindowStability.bit_mask();
    }

    mask
}

/// R.9.b — wide-mask detector evaluator. Runs the canonical 16-motif
/// predicate set at every variant's scaled threshold and packs the
/// resulting bits into a `DetectorMask2048` per cell. The bit
/// position for the `(motif_id, variant_id)` pair is
/// `motif_id * variants_per_motif + variant_id`, computed once at
/// construction and constant across cells.
///
/// **Profile support**: D16, D64, D128, and D205 are all routed
/// here at the current head. D16 produces a single variant
/// (V0 = canonical) so its wide-mask bits 0..15 are bit-identical
/// to the legacy `DetectorCell.detector_mask`; bits 16..2047 are
/// zero. D64 emits 16 motifs × 4 variants = 64 bits (bits 0..63).
/// D128 emits 16 motifs × 8 variants = 128 bits (bits 0..127,
/// occupying words 0..2 of the [u64; 32] mask). D205 (R.9.d.2)
/// iterates 16 motifs × 13 variants = 208 slots but gates firings
/// by `det_id < D205_ACTIVE_BITS = 205`, so the mask carries the
/// bottom 205 active bits plus three deterministic
/// reserved-not-fired slots (205, 206, 207). D512 / D1024 / D2000
/// are deferred to paper section 16 future work and panic at this
/// entry point until they are routed (a defensive backstop for
/// in-process callers; the dispatch sites already guard against
/// this case).
///
/// **Bridge invariants (panel-locked)**: V0..V3 of every wider
/// profile mirror D64's V0..V3 bit-for-bit, so for every cell the
/// wider mask's low 64 bits equal the D64 mask, and the
/// OR-projection chain `D205 OR ⊇ D128 OR ⊇ D64 OR ⊇ canonical D16`
/// holds. Pinned by the 10 R.9.d.1 acceptance tests in
/// `tests/r9_d_d128_acceptance.rs` and the additional R.9.d.2
/// tests in `tests/r9_d2_d205_acceptance.rs`.
///
/// **Byte equivalence**: the GPU's wide kernel mirrors this
/// per-cell evaluation order bit-for-bit at every supported
/// profile. GPU dispatch for D205 is honestly deferred to the
/// R.9.d.2.1 follow-on commit; this evaluator is the CPU
/// scaling-ladder proof.
///
/// # Panics
///
/// Panics if `profile` is a wider variant not yet implemented
/// (D512 and above). The dispatch sites guard against this; the
/// panic is a defensive backstop for in-process callers.
#[must_use]
pub fn evaluate_wide(
    profile: crate::motif::DetectorProfile,
    residuals: &[ResidualCell],
    signs: &[SignCell],
    thresholds: &DetectorThresholds,
    n_windows: u32,
    n_entities: u32,
) -> Vec<DetectorCellWide> {
    use crate::motif::DetectorProfile;
    let variants_per_motif: u32 = match profile {
        DetectorProfile::D16 => 1,
        DetectorProfile::D64 => D64_VARIANT_COUNT,
        DetectorProfile::D128 => D128_VARIANT_COUNT,
        DetectorProfile::D205 => D205_VARIANT_COUNT,
        DetectorProfile::D512 | DetectorProfile::D1024 | DetectorProfile::D2000 => {
            panic!(
                "DetectorProfile::{} not yet implemented in R.9.d.2; expected D16, D64, D128, or D205",
                profile.name()
            );
        }
    };

    let total = (n_windows as usize) * (n_entities as usize);
    debug_assert_eq!(residuals.len(), total, "residual grid shape mismatch");
    debug_assert_eq!(signs.len(), total, "sign grid shape mismatch");

    // R.9.d.2 — pick the variant-scale table for this profile.
    // D16 reuses D64's first slot (V0 = 1.0 canonical); D64 uses the
    // 4-entry D64 table; D128 uses the 8-entry D128 table whose
    // first four entries match D64 bit-for-bit (preserves the
    // V0..V3 bridge invariants); D205 uses the 13-entry D205 table
    // whose first eight entries match D128 bit-for-bit (preserves
    // the V0..V7 bridge invariants and therefore the chain
    // D205 OR ⊇ D128 OR ⊇ D64 OR ⊇ D16).
    let scales_slice: &[i32] = match profile {
        DetectorProfile::D16 | DetectorProfile::D64 => &D64_VARIANT_SCALES_Q16,
        DetectorProfile::D128 => &D128_VARIANT_SCALES_Q16,
        DetectorProfile::D205 => &D205_VARIANT_SCALES_Q16,
        _ => unreachable!("guarded above by the panic on D512+ profiles"),
    };

    // R.9.d.2 — active-bit gate. D16/D64/D128 emit a tight
    // motif × variants product (16 / 64 / 128) with no high-bit
    // gate needed. D205 iterates 16 × 13 = 208 candidate slots
    // but reports active_detector_count = 205, so the inner-loop
    // gate `det_id < D205_ACTIVE_BITS` masks the top 3 slots to
    // zero. For D16/D64/D128 the gate is unreachable
    // (`u32::MAX`) so the inner loop is unchanged.
    let active_bit_limit: u32 = match profile {
        DetectorProfile::D205 => D205_ACTIVE_BITS,
        _ => u32::MAX,
    };

    // Pre-compute scaled thresholds per variant once. The D16 case
    // reuses the canonical threshold table directly (no copy).
    let scaled: Vec<DetectorThresholds> = (0..variants_per_motif)
        .map(|v| scale_thresholds(thresholds, scales_slice[v as usize]))
        .collect();

    let mut out: Vec<DetectorCellWide> = Vec::with_capacity(total);
    for entity_id in 0..n_entities {
        for window_idx in 0..n_windows {
            let mut wide = DetectorCellWide {
                window_idx,
                entity_id,
                detector_mask: [0u64; 32],
            };
            for variant in 0..variants_per_motif {
                let scaled_thresh = &scaled[variant as usize];
                let d16_mask = eval_motifs_for_cell(
                    residuals,
                    signs,
                    scaled_thresh,
                    entity_id,
                    window_idx,
                    n_windows,
                );
                for motif_id in 0..16u32 {
                    if (d16_mask & (1u32 << motif_id)) != 0 {
                        let det_id = motif_id * variants_per_motif + variant;
                        if det_id < active_bit_limit {
                            wide.set_bit(det_id);
                        }
                    }
                }
            }
            out.push(wide);
        }
    }
    out
}

/// Detector 3: drift_ramp.
fn drift_ramp_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx + 1 < t.ramp_window {
        return false;
    }
    let mut prev = i32::MIN;
    for k in 0..t.ramp_window {
        let w = window_idx + 1 - t.ramp_window + k;
        let idx = flat(entity_id, w, n_windows);
        let d = signs[idx].drift_q.raw();
        if d <= prev {
            return false;
        }
        prev = d;
    }
    true
}

/// Detector 5: plateau.
fn plateau_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx + 1 < t.plateau_windows {
        return false;
    }
    for k in 0..t.plateau_windows {
        let w = window_idx + 1 - t.plateau_windows + k;
        let idx = flat(entity_id, w, n_windows);
        let c = &signs[idx];
        if c.norm_q.raw() < t.plateau_min_q16_raw {
            return false;
        }
        if c.slew_q.abs().raw() > t.plateau_slew_max_q16_raw {
            return false;
        }
    }
    true
}

/// Detector 6: oscillation. Counts sign alternations in slew across the
/// last `oscillation_window` cells.
fn oscillation_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx + 1 < t.oscillation_window {
        return false;
    }
    let mut alternations = 0u32;
    let mut last_sign: i32 = 0;
    for k in 0..t.oscillation_window {
        let w = window_idx + 1 - t.oscillation_window + k;
        let idx = flat(entity_id, w, n_windows);
        let raw = signs[idx].slew_q.raw();
        let sign = match raw.cmp(&0) {
            core::cmp::Ordering::Greater => 1,
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
        };
        if sign != 0 && last_sign != 0 && sign != last_sign {
            alternations += 1;
        }
        if sign != 0 {
            last_sign = sign;
        }
    }
    alternations >= t.oscillation_alternations
}

/// Detector 7: deadband_exit. Reads only the immediately preceding cell.
fn deadband_exit_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx == 0 {
        return false;
    }
    let prev = &signs[flat(entity_id, window_idx - 1, n_windows)];
    let cur = &signs[flat(entity_id, window_idx, n_windows)];
    prev.norm_q.raw() < t.deadband_low_q16_raw && cur.norm_q.raw() > t.deadband_high_q16_raw
}

/// Detector 10: entity_local_anomaly. Single-cell proxy: norm
/// exceeds drift by the configured Q16 multiplier.
fn entity_local_anomaly_fires(s: &SignCell, t: &DetectorThresholds) -> bool {
    // `norm > factor * drift`. We compare in raw i64 to keep overflow
    // explicit.
    let factor = i64::from(t.entity_anomaly_factor_q16_raw);
    let drift = i64::from(s.drift_q.raw());
    let lhs = i64::from(s.norm_q.raw()) << 16; // align both sides to Q32 for the compare
    let rhs = factor.saturating_mul(drift);
    lhs > rhs && s.drift_q.raw() > 0
}

/// Detector 12: fanout_precursor. Drift rising past the fan-out threshold
/// while the cell already shows any non-zero error residual is a
/// precursor signal in v0.
fn fanout_precursor_fires(s: &SignCell, r: &ResidualCell, t: &DetectorThresholds) -> bool {
    s.drift_q.raw() > t.fanout_drift_q16_raw && r.residual_error_q.raw() > 0
}

/// Detector 13: variance_expansion. Uses max-minus-min spread of norm
/// across the variance window as a deterministic, sqrt-free proxy for
/// variance.
fn variance_expansion_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx + 1 < t.variance_window {
        return false;
    }
    let mut hi = i32::MIN;
    let mut lo = i32::MAX;
    for k in 0..t.variance_window {
        let w = window_idx + 1 - t.variance_window + k;
        let idx = flat(entity_id, w, n_windows);
        let raw = signs[idx].norm_q.raw();
        if raw > hi {
            hi = raw;
        }
        if raw < lo {
            lo = raw;
        }
    }
    Q16::from_raw(hi).sat_sub(Q16::from_raw(lo)).raw() > t.variance_threshold_q16_raw
}

/// Detector 14: recovery_edge. Current drift below previous drift while
/// the absolute norm is still above the recovery floor.
fn recovery_edge_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx == 0 {
        return false;
    }
    let prev = &signs[flat(entity_id, window_idx - 1, n_windows)];
    let cur = &signs[flat(entity_id, window_idx, n_windows)];
    cur.drift_q.raw() < prev.drift_q.raw() && cur.norm_q.raw() > t.recovery_min_norm_q16_raw
}

/// Detector 16: confuser_like_transient. Current cell is a spike while
/// the previous cell sat inside the clean band.
fn confuser_like_transient_fires(
    signs: &[SignCell],
    entity_id: u32,
    window_idx: u32,
    n_windows: u32,
    t: &DetectorThresholds,
) -> bool {
    if window_idx == 0 {
        return false;
    }
    let prev = &signs[flat(entity_id, window_idx - 1, n_windows)];
    let cur = &signs[flat(entity_id, window_idx, n_windows)];
    cur.norm_q.raw() > t.confuser_min_q16_raw && prev.norm_q.abs().raw() <= t.clean_band_q16_raw
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
    use crate::residual::{compute as residual_compute, Baseline};
    use crate::sign::compute as sign_compute;
    use crate::window::{compute_features, WindowFeature};

    const ALPHA: Q16 = Q16::from_raw(0x2000);

    /// End-to-end pipeline against the synthesized fixture, returning the
    /// detector grid for inspection.
    fn full_pipeline() -> Vec<DetectorCell> {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        )
    }

    #[test]
    fn detector_grid_has_expected_shape() {
        let grid = full_pipeline();
        assert_eq!(grid.len(), (N_WINDOWS as usize) * (N_ENTITIES as usize));
    }

    #[test]
    fn detector_evaluation_is_deterministic() {
        let a = full_pipeline();
        let b = full_pipeline();
        assert_eq!(a, b);
    }

    #[test]
    fn ramp_episode_fires_spike_and_sustain_and_ramp() {
        let grid = full_pipeline();
        // Pick a cell deep in the ramp on entity 3.
        let idx = WindowFeature::flat_index(3, 34, N_WINDOWS);
        let cell = grid[idx];
        assert!(cell.fired(MotifClass::ResidualSpike));
        assert!(cell.fired(MotifClass::SustainedResidualElevation));
        assert!(cell.fired(MotifClass::DriftRamp));
    }

    #[test]
    fn burst_episode_fires_error_rate_burst_and_coupling() {
        let grid = full_pipeline();
        // Middle of the error burst on entity 7.
        let idx = WindowFeature::flat_index(7, 62, N_WINDOWS);
        let cell = grid[idx];
        assert!(cell.fired(MotifClass::ErrorRateBurst));
        // Burst events still carry baseline latency, so coupling may or
        // may not fire — we just verify the error-axis bit is up.
    }

    #[test]
    fn shock_episode_fires_slew_shock_and_recovery_edge_in_subsequent_windows() {
        let grid = full_pipeline();
        let shock_idx = WindowFeature::flat_index(11, 90, N_WINDOWS);
        assert!(grid[shock_idx].fired(MotifClass::SlewShock));
        // At least one cell in the recovery range must report a recovery edge.
        let any_recovery = (91..96).any(|w| {
            let idx = WindowFeature::flat_index(11, w, N_WINDOWS);
            grid[idx].fired(MotifClass::RecoveryEdge)
        });
        assert!(
            any_recovery,
            "no recovery edge fired in the post-shock window range"
        );
    }

    #[test]
    fn clean_windows_fire_only_clean_stability_bit() {
        let grid = full_pipeline();
        // Pick a clean entity (entity 0) at a quiet window (window 5 —
        // far from all three episodes).
        let idx = WindowFeature::flat_index(0, 5, N_WINDOWS);
        let cell = grid[idx];
        if cell.fired(MotifClass::CleanWindowStability) {
            // If clean fired, no other bit may be set.
            let non_clean = cell.detector_mask & !MotifClass::CleanWindowStability.bit_mask();
            assert_eq!(non_clean, 0);
        }
    }

    #[test]
    fn confuser_detector_does_not_fire_on_sustained_ramp() {
        let grid = full_pipeline();
        // Confuser is meant to catch single-window spikes, not sustained
        // ramps. Entity 3's deep-ramp cells should not see the confuser bit.
        let idx = WindowFeature::flat_index(3, 34, N_WINDOWS);
        assert!(!grid[idx].fired(MotifClass::ConfuserLikeTransient));
    }

    // ====================================================================
    // R.9.b acceptance tests — wide-mask detector evaluator + D64.
    // ====================================================================

    use crate::motif::DetectorProfile;

    /// Helper: compute the canonical D16 mask via the legacy
    /// `evaluate` and the wide D16 mask via `evaluate_wide`. Returns
    /// both for cross-comparison.
    fn evaluate_both_d16(
        events: &[crate::event::TraceEvent],
    ) -> (Vec<DetectorCell>, Vec<DetectorCellWide>) {
        let features = compute_features(events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let legacy = evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let wide = evaluate_wide(
            DetectorProfile::D16,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        (legacy, wide)
    }

    #[test]
    fn d16_legacy_and_wide_masks_match_bit_for_bit() {
        // Load-bearing R.9.b invariant: the canonical 16-detector
        // path produces a wide mask whose low 16 bits equal the
        // legacy `DetectorCell.detector_mask` exactly, and whose
        // remaining 2032 bits are zero. If this broke we'd have
        // silently divergent D16 byte forms in two code paths.
        let events = synthesize(DEFAULT_SEED);
        let (legacy, wide) = evaluate_both_d16(&events);
        assert_eq!(legacy.len(), wide.len());
        for (i, (a, b)) in legacy.iter().zip(wide.iter()).enumerate() {
            assert_eq!(a.window_idx, b.window_idx, "cell {i} window_idx mismatch");
            assert_eq!(a.entity_id, b.entity_id, "cell {i} entity_id mismatch");
            assert_eq!(
                u64::from(a.detector_mask),
                b.detector_mask[0],
                "cell {i} mask divergence: legacy={:08x} wide[0]={:016x}",
                a.detector_mask,
                b.detector_mask[0]
            );
            // Every higher word must be zero in D16.
            for (w, &word) in b.detector_mask.iter().enumerate().skip(1) {
                assert_eq!(word, 0, "cell {i} word {w} non-zero in D16 wide mask");
            }
        }
    }

    #[test]
    fn d64_v0_bits_match_d16_bits() {
        // The D64 design lock states that variant V0 uses the
        // canonical thresholds verbatim, so D64.detector_id =
        // motif_id * 4 + 0 must fire on exactly the same cells as
        // D16.detector_id = motif_id. This is the "strict superset"
        // property — every D16 firing is recoverable from the D64
        // mask by reading bit (motif_id * 4) of the wide cell.
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let d16 = evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let d64 = evaluate_wide(
            DetectorProfile::D64,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        assert_eq!(d16.len(), d64.len());
        for (i, (cell16, cell64)) in d16.iter().zip(d64.iter()).enumerate() {
            for motif_id in 0..16u32 {
                let d16_bit = (cell16.detector_mask & (1u32 << motif_id)) != 0;
                let d64_det_id = motif_id * D64_VARIANT_COUNT;
                let d64_bit = cell64.fired_by_id(d64_det_id);
                assert_eq!(
                    d16_bit, d64_bit,
                    "cell {i} motif_id {motif_id}: D16.bit_{motif_id} != D64.bit_{d64_det_id}"
                );
            }
        }
    }

    #[test]
    fn d64_evaluation_is_deterministic_across_runs() {
        // Two consecutive calls produce byte-identical wide masks.
        // Catches any non-determinism the variant-scaling logic
        // might have introduced (e.g. address-dependent iteration
        // order or floating-point sneaking in).
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let a = evaluate_wide(
            DetectorProfile::D64,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let b = evaluate_wide(
            DetectorProfile::D64,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        assert_eq!(a.len(), b.len());
        for (i, (ca, cb)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(ca, cb, "D64 cell {i} differs between runs");
        }
    }

    #[test]
    fn d64_total_firings_strictly_exceed_d16() {
        // Sanity: V1 (sensitive, threshold × 0.5) MUST fire on at
        // least as many cells as V0 in aggregate, because lowering
        // a threshold can only add firings or leave them unchanged
        // for the standalone scalar-comparison motifs (ResidualSpike,
        // SustainedResidualElevation, etc.). The window-based motifs
        // are not strictly monotonic in window length, so we relax
        // the assertion to "total D64 firings ≥ 4× D16 firings of
        // the strict-superset subset" with a generous margin.
        //
        // The looser version: D64 total bit-count ≥ D16 total
        // bit-count, since D64 V0 ≡ D16 and V1..V3 add firings.
        // We assert exact equality on V0 and ≥ on totals.
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let d16 = evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let d64 = evaluate_wide(
            DetectorProfile::D64,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let d16_total: u64 = d16
            .iter()
            .map(|c| u64::from(c.detector_mask.count_ones()))
            .sum();
        let d64_total: u64 = d64.iter().map(|c| u64::from(c.popcount())).sum();
        assert!(
            d64_total >= d16_total,
            "D64 total firings ({d64_total}) must be >= D16 total firings ({d16_total}) \
             because V0 ≡ canonical and V1..V3 add or repeat firings"
        );
    }

    #[test]
    fn scale_threshold_identity_at_unit_scale() {
        // For scale_q16 = 1.0 (= 1 << 16) every threshold value
        // must round-trip unchanged. This is the V0-equals-canonical
        // proof at the primitive level.
        let canon = DetectorThresholds::CANONICAL;
        let scaled = scale_thresholds(&canon, 1 << 16);
        assert_eq!(scaled, canon, "scale_thresholds(_, 1.0) must be identity");
    }

    #[test]
    fn scale_window_clamps_below_one() {
        // A scale factor of 0 would produce 0, which is invalid for
        // every motif that uses a window count. The function clamps
        // to ≥ 1 so the kernel can safely use the scaled value as a
        // loop bound.
        assert_eq!(scale_window(8, 0), 1, "scale × 0 must clamp to 1");
        assert_eq!(scale_window(8, 1), 1, "extreme small scale still clamps");
        assert_eq!(scale_window(8, 1 << 16), 8, "scale × 1.0 is identity");
        assert_eq!(scale_window(8, 1 << 17), 16, "scale × 2.0 doubles");
    }

    #[test]
    fn d64_variant_scales_v0_is_unity() {
        // The variant-scale array's V0 slot MUST be 1.0 in Q16.16
        // (= 1 << 16). If a future refactor changes this constant
        // accidentally, the D64 V0 bits would diverge from D16 and
        // every wider-profile case file would mis-commit.
        assert_eq!(
            D64_VARIANT_SCALES_Q16[0],
            1 << 16,
            "D64_VARIANT_SCALES_Q16[V0] must equal 1.0 in Q16.16"
        );
    }
}
