//! R.9.d.2 — D205 profile CPU acceptance tests.
//!
//! D205 mirrors the dsfb-debug mature 205-detector taxonomy count.
//! The wide-mask layout is 16 canonical motifs × 13 threshold-scaled
//! variants = 208 candidate slots; firings are gated by
//! `det_id < D205_ACTIVE_BITS = 205`, leaving the top three slots
//! (205, 206, 207) deterministically held at zero.
//!
//! R.9.d.2 is a SCALING-LADDER proof, NOT a new R.13 performance
//! headline. The D64 ≈55× full-pipeline reduction at the
//! courthouse-factory workload remains the headline. D205 lands the
//! detector-count bridge to the dsfb-debug 205-count identity; it
//! does NOT require beating D64 performance.
//!
//! GPU dispatch for D205 (`detector_motif_kernel_wide_d205`,
//! `build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact`)
//! is honestly deferred to the R.9.d.2.1 follow-on commit. The
//! tests in this file are CPU-only so they run on any host.
//!
//! Test taxonomy (mirrors R.9.d.1):
//!
//! 1. **D205 registry hash distinct from D16 / D64 / D128**.
//! 2. **D205 V0-only projection equals canonical D16**.
//! 3. **D205 OR projection ⊇ V0 projection**.
//! 4. **D205 OR projection ⊇ D128 OR projection ⊇ D64 OR projection**.
//! 5. **D205 V0..V7 firings byte-identical to D128 V0..V7** (the
//!    canonical bridge invariant at the per-cell-per-variant level).
//! 6. **D205 CPU evaluation is deterministic across two runs**.
//! 7. **D205 active_detector_count equals 205**, total slots equal
//!    208, and the variant count equals 13.
//! 8. **High-bit slots above 205 remain zero** on every cell (the
//!    active-bit gate is enforced).
//! 9. **D205 V0 bits are at motif_id * 13**.
//! 10. **D205 total interesting cells ≥ D128 total interesting cells**
//!     (the wider variant set can only add firings).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::detector::{
    evaluate, evaluate_wide, DetectorCellWide, DetectorThresholds, D128_VARIANT_COUNT,
    D128_VARIANT_SCALES_Q16, D205_ACTIVE_BITS, D205_TOTAL_SLOTS, D205_VARIANT_COUNT,
    D205_VARIANT_SCALES_Q16,
};
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::residual::{compute as residual_compute, Baseline};
use dsfb_gpu_debug_core::sign::compute as sign_compute;
use dsfb_gpu_debug_core::window::compute_features;

// ---------------------------------------------------------------
// Projection helpers — extract per-motif firing patterns from the
// wide [u64; 32] mask for each profile.
// ---------------------------------------------------------------

/// OR-projection helper for D205: for each motif, OR over its 13
/// variants (slots `motif_id * 13 .. motif_id * 13 + 13`) capped
/// at `D205_ACTIVE_BITS`, and set the per-motif bit in the u32
/// output if any variant fired. Bits 208..2047 are not iterated;
/// bits 205..207 are required by the active-bit gate to stay zero.
fn project_d205_or_to_u16(cell: &DetectorCellWide) -> u32 {
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let mut fired = false;
        for variant in 0..D205_VARIANT_COUNT {
            let det_id = motif_id * D205_VARIANT_COUNT + variant;
            if det_id >= D205_ACTIVE_BITS {
                break;
            }
            let word = (det_id / 64) as usize;
            let bit = det_id % 64;
            if (cell.detector_mask[word] >> bit) & 1u64 != 0 {
                fired = true;
                break;
            }
        }
        if fired {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

/// V0-only projection helper for D205: for each motif, take the
/// bit at slot `motif_id * 13`. With V0 scale = 1.0 (canonical),
/// this projection must equal the canonical D16 mask.
fn project_d205_v0_only_to_u16(cell: &DetectorCellWide) -> u32 {
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let det_id = motif_id * D205_VARIANT_COUNT;
        let word = (det_id / 64) as usize;
        let bit = det_id % 64;
        if (cell.detector_mask[word] >> bit) & 1u64 != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

/// OR-projection helper for D128 (8 variants in words 0..1). Used
/// to compare D205 ⊇ D128.
fn project_d128_or_to_u16(cell: &DetectorCellWide) -> u32 {
    let word0 = cell.detector_mask[0];
    let word1 = cell.detector_mask[1];
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let source = if motif_id < 8 { word0 } else { word1 };
        let shift_in_source = (motif_id & 7) * 8;
        let eight_bits = (source >> shift_in_source) & 0xFFu64;
        if eight_bits != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

/// OR-projection helper for D64 (4 variants in word 0). Used
/// transitively to verify the full bridge chain.
fn project_d64_or_to_u16(cell: &DetectorCellWide) -> u32 {
    let word0 = cell.detector_mask[0];
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let four_bits = (word0 >> (motif_id * 4)) & 0xFu64;
        if four_bits != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

// ---------------------------------------------------------------
// Fixture helpers.
// ---------------------------------------------------------------

fn canonical_inputs() -> (
    Contract,
    Vec<dsfb_gpu_debug_core::residual::ResidualCell>,
    Vec<dsfb_gpu_debug_core::sign::SignCell>,
) {
    let contract = Contract::canonical();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let residuals = residual_compute(&features, &Baseline::CANONICAL);
    let signs = sign_compute(
        &residuals,
        dsfb_gpu_debug_core::fixed::Q16::from_raw(contract.ewma_alpha_q16_raw),
        contract.n_windows,
        contract.n_entities,
    );
    (contract, residuals, signs)
}

fn d205_cells() -> Vec<DetectorCellWide> {
    let (contract, residuals, signs) = canonical_inputs();
    evaluate_wide(
        DetectorProfile::D205,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    )
}

fn d128_cells() -> Vec<DetectorCellWide> {
    let (contract, residuals, signs) = canonical_inputs();
    evaluate_wide(
        DetectorProfile::D128,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    )
}

fn d64_cells() -> Vec<DetectorCellWide> {
    let (contract, residuals, signs) = canonical_inputs();
    evaluate_wide(
        DetectorProfile::D64,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    )
}

// ---------------------------------------------------------------
// Constants + registry hash.
// ---------------------------------------------------------------

#[test]
fn d205_variant_counts_are_canonical() {
    assert_eq!(
        D205_VARIANT_COUNT, 13,
        "D205_VARIANT_COUNT must be 13 (16 motifs × 13 = 208 slots; 205 active)"
    );
    assert_eq!(
        D205_ACTIVE_BITS, 205,
        "D205_ACTIVE_BITS must be 205 to match the dsfb-debug mature-taxonomy count"
    );
    assert_eq!(
        D205_TOTAL_SLOTS, 208,
        "D205_TOTAL_SLOTS must be 16 * 13 = 208"
    );
    assert_eq!(
        D205_VARIANT_SCALES_Q16.len(),
        D205_VARIANT_COUNT as usize,
        "D205_VARIANT_SCALES_Q16 must have D205_VARIANT_COUNT entries"
    );
    assert_eq!(
        DetectorProfile::D205.active_detector_count(),
        205,
        "DetectorProfile::D205.active_detector_count() must equal 205"
    );
}

#[test]
fn d205_variant_scales_v0_through_v7_match_d128() {
    // The bridge invariant rests on V0..V7 of D205 being byte-
    // identical to V0..V7 of D128. Pinning that here so a future
    // refactor cannot silently reorder D128's scales without
    // breaking this assertion.
    assert_eq!(D128_VARIANT_COUNT, 8);
    for v in 0..D128_VARIANT_COUNT as usize {
        assert_eq!(
            D205_VARIANT_SCALES_Q16[v], D128_VARIANT_SCALES_Q16[v],
            "D205.V{v} scale must equal D128.V{v} scale (bridge invariant)"
        );
    }
}

#[test]
fn d205_registry_hash_is_distinct_from_d16_d64_d128() {
    let d16 = DetectorProfile::D16.registry_hash();
    let d64 = DetectorProfile::D64.registry_hash();
    let d128 = DetectorProfile::D128.registry_hash();
    let d205 = DetectorProfile::D205.registry_hash();
    assert_ne!(d205, d16, "D205 registry hash must differ from D16");
    assert_ne!(d205, d64, "D205 registry hash must differ from D64");
    assert_ne!(d205, d128, "D205 registry hash must differ from D128");
}

// ---------------------------------------------------------------
// Bridge invariants.
// ---------------------------------------------------------------

#[test]
fn d205_v0_projection_equals_canonical_d16_mask() {
    let (contract, residuals, signs) = canonical_inputs();
    let d16 = evaluate(
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let d205 = d205_cells();
    assert_eq!(d16.len(), d205.len());
    for (i, (d16_cell, d205_cell)) in d16.iter().zip(d205.iter()).enumerate() {
        let v0 = project_d205_v0_only_to_u16(d205_cell);
        assert_eq!(
            d16_cell.detector_mask, v0,
            "cell {i}: D205.V0-only projection ({v0:#x}) != canonical D16 mask ({:#x})",
            d16_cell.detector_mask
        );
    }
}

#[test]
fn d205_or_projection_is_superset_of_v0() {
    let cells = d205_cells();
    for (i, cell) in cells.iter().enumerate() {
        let v0 = project_d205_v0_only_to_u16(cell);
        let or = project_d205_or_to_u16(cell);
        assert_eq!(
            v0 & or,
            v0,
            "cell {i}: D205 OR projection ({or:#x}) is not a superset of D205 V0 projection ({v0:#x})"
        );
    }
}

#[test]
fn d205_or_projection_is_superset_of_d128_or_projection() {
    let d128 = d128_cells();
    let d205 = d205_cells();
    assert_eq!(d128.len(), d205.len());
    for (i, (d128_cell, d205_cell)) in d128.iter().zip(d205.iter()).enumerate() {
        let d128_or = project_d128_or_to_u16(d128_cell);
        let d205_or = project_d205_or_to_u16(d205_cell);
        assert_eq!(
            d128_or & d205_or,
            d128_or,
            "cell {i}: D205 OR projection ({d205_or:#x}) is not a superset of D128 OR projection ({d128_or:#x})"
        );
    }
}

#[test]
fn d205_or_projection_is_superset_of_d64_or_projection() {
    // Transitive bridge: D205 ⊇ D128 ⊇ D64. The intermediate step
    // is already pinned; this test pins the end-to-end chain so a
    // future refactor of D128's variant scales (without
    // simultaneously refactoring D205's) cannot silently break the
    // long-range invariant.
    let d64 = d64_cells();
    let d205 = d205_cells();
    assert_eq!(d64.len(), d205.len());
    for (i, (d64_cell, d205_cell)) in d64.iter().zip(d205.iter()).enumerate() {
        let d64_or = project_d64_or_to_u16(d64_cell);
        let d205_or = project_d205_or_to_u16(d205_cell);
        assert_eq!(
            d64_or & d205_or,
            d64_or,
            "cell {i}: D205 OR projection ({d205_or:#x}) is not a superset of D64 OR projection ({d64_or:#x})"
        );
    }
}

#[test]
fn d205_v0_through_v7_firings_match_d128_byte_for_byte() {
    // The strongest bridge invariant: for every cell and every
    // variant v ∈ {0..7}, the firing pattern of D205 at variant
    // v equals the firing pattern of D128 at variant v. The bit
    // POSITIONS differ (D128 uses motif*8+v, D205 uses motif*13+v),
    // but the PREDICATE OUTPUT (16 motif bits per variant) must be
    // identical because the threshold scales are identical.
    let d128 = d128_cells();
    let d205 = d205_cells();
    assert_eq!(d128.len(), d205.len());
    for (i, (d128_cell, d205_cell)) in d128.iter().zip(d205.iter()).enumerate() {
        for v in 0..D128_VARIANT_COUNT {
            let mut d128_motifs = 0u32;
            let mut d205_motifs = 0u32;
            for motif_id in 0..16u32 {
                let d128_slot = motif_id * D128_VARIANT_COUNT + v;
                let d128_word = (d128_slot / 64) as usize;
                let d128_bit = d128_slot % 64;
                if (d128_cell.detector_mask[d128_word] >> d128_bit) & 1u64 != 0 {
                    d128_motifs |= 1u32 << motif_id;
                }
                let d205_slot = motif_id * D205_VARIANT_COUNT + v;
                let d205_word = (d205_slot / 64) as usize;
                let d205_bit = d205_slot % 64;
                if (d205_cell.detector_mask[d205_word] >> d205_bit) & 1u64 != 0 {
                    d205_motifs |= 1u32 << motif_id;
                }
            }
            assert_eq!(
                d128_motifs, d205_motifs,
                "cell {i}: D128.V{v} motif-mask ({d128_motifs:#x}) != D205.V{v} motif-mask ({d205_motifs:#x})"
            );
        }
    }
}

// ---------------------------------------------------------------
// Active-bit gate.
// ---------------------------------------------------------------

#[test]
fn d205_high_bits_above_205_remain_zero() {
    // The gate `det_id < D205_ACTIVE_BITS` must hold cell-by-cell:
    // bits 205, 206, 207 (the three reserved-not-fired slots) and
    // bits 208..2047 (never iterated) must all be zero.
    let cells = d205_cells();
    for (i, cell) in cells.iter().enumerate() {
        for bit_id in D205_ACTIVE_BITS..(32 * 64) {
            let word = (bit_id / 64) as usize;
            let bit = bit_id % 64;
            let set = (cell.detector_mask[word] >> bit) & 1u64;
            assert_eq!(
                set, 0,
                "cell {i}: bit {bit_id} is set but must be zero (above D205_ACTIVE_BITS = 205)"
            );
        }
    }
}

#[test]
fn d205_no_cell_has_active_count_above_205() {
    // A defensive cross-check: popcount over the full mask must
    // never exceed 205 on any cell.
    let cells = d205_cells();
    for (i, cell) in cells.iter().enumerate() {
        let popcount: u32 = cell.detector_mask.iter().map(|w| w.count_ones()).sum();
        assert!(
            popcount <= D205_ACTIVE_BITS,
            "cell {i}: popcount {popcount} > D205_ACTIVE_BITS {D205_ACTIVE_BITS}"
        );
    }
}

// ---------------------------------------------------------------
// Layout proof.
// ---------------------------------------------------------------

#[test]
fn d205_v0_bits_are_at_motif_times_13() {
    // Pin the bit-layout invariant for the V0 projection helper.
    // motif 0 V0 = bit 0 (word 0 bit 0)
    // motif 1 V0 = bit 13 (word 0 bit 13)
    // motif 4 V0 = bit 52 (word 0 bit 52)
    // motif 5 V0 = bit 65 (word 1 bit 1)
    // motif 15 V0 = bit 195 (word 3 bit 3)
    let layout: [(u32, usize, u32); 5] = [(0, 0, 0), (1, 0, 13), (4, 0, 52), (5, 1, 1), (15, 3, 3)];
    for (motif_id, expected_word, expected_bit) in layout {
        let det_id = motif_id * D205_VARIANT_COUNT;
        assert_eq!(
            (det_id / 64) as usize,
            expected_word,
            "motif {motif_id} V0 should be in word {expected_word}, got word {}",
            det_id / 64
        );
        assert_eq!(
            det_id % 64,
            expected_bit,
            "motif {motif_id} V0 should be at bit {expected_bit}, got bit {}",
            det_id % 64
        );
    }
}

// ---------------------------------------------------------------
// Determinism + population.
// ---------------------------------------------------------------

#[test]
fn d205_cpu_evaluation_is_deterministic_across_runs() {
    // Two evaluate_wide calls on the same fixture must produce
    // byte-identical DetectorCellWide buffers.
    let a = d205_cells();
    let b = d205_cells();
    assert_eq!(a.len(), b.len());
    for (i, (ca, cb)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            ca.detector_mask, cb.detector_mask,
            "cell {i}: D205 mask diverged between two CPU evaluations"
        );
        assert_eq!(
            ca.window_idx, cb.window_idx,
            "cell {i}: window_idx diverged"
        );
        assert_eq!(ca.entity_id, cb.entity_id, "cell {i}: entity_id diverged");
    }
}

#[test]
fn d205_total_interesting_cells_meets_or_exceeds_d128() {
    // Population-level invariant: the count of cells whose
    // OR-projection is non-zero must satisfy D205 ≥ D128 because
    // D205's variants V0..V7 match D128's V0..V7 byte-for-byte
    // and V8..V12 can only add cells, never remove them.
    let d128 = d128_cells();
    let d205 = d205_cells();
    assert_eq!(d128.len(), d205.len());
    let d128_firing: usize = d128
        .iter()
        .filter(|c| project_d128_or_to_u16(c) != 0)
        .count();
    let d205_firing: usize = d205
        .iter()
        .filter(|c| project_d205_or_to_u16(c) != 0)
        .count();
    assert!(
        d205_firing >= d128_firing,
        "D205 firing-cell count ({d205_firing}) must be >= D128 firing-cell count ({d128_firing})"
    );
}

#[test]
fn d205_admits_at_least_one_firing_at_canonical_fixture() {
    // Defensive: the canonical fixture has 3 injected episodes; at
    // least one cell must fire under D205 (otherwise the
    // implementation is silently empty).
    let cells = d205_cells();
    let firing = cells
        .iter()
        .filter(|c| c.detector_mask.iter().any(|w| *w != 0))
        .count();
    assert!(
        firing > 0,
        "D205 must admit at least one firing cell on the canonical fixture"
    );
}

#[test]
fn d205_silently_unused_for_d64_d128_paths() {
    // Cross-check: evaluating D64 and D128 must not touch any bit
    // at index >= 128, regardless of how D205's gate is implemented.
    // (Defensive — if a future refactor accidentally widens the gate
    // for D64/D128, this test surfaces it.)
    let d64 = d64_cells();
    let d128 = d128_cells();
    for (i, cell) in d64.iter().enumerate() {
        for word_idx in 1..32 {
            assert_eq!(
                cell.detector_mask[word_idx], 0,
                "D64 cell {i}: word {word_idx} must be zero (D64 spans word 0 only)"
            );
        }
    }
    for (i, cell) in d128.iter().enumerate() {
        for word_idx in 2..32 {
            assert_eq!(
                cell.detector_mask[word_idx], 0,
                "D128 cell {i}: word {word_idx} must be zero (D128 spans words 0..1)"
            );
        }
    }
    // Sanity: D205 uses words 0..3 (bit 195 is in word 3).
    let d205 = d205_cells();
    for (i, cell) in d205.iter().enumerate() {
        for word_idx in 4..32 {
            assert_eq!(
                cell.detector_mask[word_idx], 0,
                "D205 cell {i}: word {word_idx} must be zero (D205 spans words 0..3)"
            );
        }
    }
}
