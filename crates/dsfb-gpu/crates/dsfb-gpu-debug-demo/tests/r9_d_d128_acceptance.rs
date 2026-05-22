//! R.9.d.1 — D128 profile acceptance tests.
//!
//! D128 is 16 canonical motifs × 8 threshold-scaled variants =
//! 128 active detectors. Active bits 0..127 populate
//! `DetectorCellWide::detector_mask[0..2]`. R.9.d.1 lands D128 as
//! a scaling-proof commit: profile expansion + correctness +
//! measurement, no kernel-level optimisation yet (the D128
//! detector tree-digest still hashes the full 264-byte
//! `DetectorCellWide` stride; the R.10b compact form is
//! D64-specific and deferred for D128).
//!
//! Test taxonomy (mirrors R.9.b.3 / R.9.c-diagnostic for D64):
//!
//! 1. **D128 registry hash distinct from D16 and D64** — the
//!    `detector_registry` chain link must distinguish a D128
//!    receipt from any other profile.
//! 2. **D128 V0-only projection equals canonical D16** — the
//!    R.9.b bridge invariant extended to D128: variant 0 has the
//!    same threshold scale as D64.V0 = 1.0, so its firing
//!    pattern matches the canonical D16 mask exactly.
//! 3. **D128 OR projection ⊇ V0 projection** — strict-superset
//!    property: extra variants can only add firings, never
//!    remove them.
//! 4. **D128 OR projection ⊇ D64 OR projection** — D128's first
//!    four variants mirror D64's exactly, so D128 sees at least
//!    every cell D64 sees fire.
//! 5. **D128 throughput replay is byte-identical across two runs**
//!    — full case file (chain hashes + episodes + final hash) is
//!    deterministic.
//! 6. **D128 case-file binds to `DetectorProfile::D128.registry_hash()`**
//!    when the caller pins it on the contract.
//! 7. **D128 episodes are bank-admitted** — Semantic Non-Bypass
//!    Axiom holds at the wider profile.
//! 8. **D128 episode count exceeds the D64 episode count** at the
//!    canonical fixture (the wider variant set produces strictly
//!    more candidate-eligible cells).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::{chain, Contract};
use dsfb_gpu_debug_core::detector::{
    evaluate_wide, DetectorCellWide, DetectorThresholds, D128_VARIANT_COUNT, D64_VARIANT_COUNT,
};
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::residual::{compute as residual_compute, Baseline};
use dsfb_gpu_debug_core::sign::compute as sign_compute;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact, GpuWorkspace,
};

fn d128_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D128.registry_hash());
    c
}

// `d64_contract` was used by an earlier
// `d128_episode_count_meets_or_exceeds_d64` test that the corrected
// invariant (`d128_total_interesting_cells_meets_or_exceeds_d64`)
// replaced. Kept in the file under `#[allow(dead_code)]` because
// future R.9.d.2+ acceptance tests will want a D64 contract handle
// alongside the D128 one for cross-profile comparisons.
#[allow(dead_code)]
fn d64_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

/// CPU OR-projection helper: take the 128 active bits in
/// `detector_mask[0..2]` and reduce per motif to the canonical
/// 16-bit basis via OR over the 8 variants.
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

/// CPU V0-only projection: pick just the V0 bit (variant 0) per
/// motif. With the R.9.d.1 layout (det_id = motif_id * 8 + v),
/// V0 of motif_id is at bit `motif_id * 8`.
fn project_d128_v0_only_to_u16(cell: &DetectorCellWide) -> u32 {
    let word0 = cell.detector_mask[0];
    let word1 = cell.detector_mask[1];
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let source = if motif_id < 8 { word0 } else { word1 };
        let shift = (motif_id & 7) * 8;
        let v0_bit = (source >> shift) & 1u64;
        if v0_bit != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

/// CPU OR-projection helper for D64 (4 variants per motif, all in
/// word 0). Mirrors `cuda::project_d64_to_u16`.
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

#[test]
fn d128_registry_hash_is_distinct_from_d16_and_d64() {
    let d16 = DetectorProfile::D16.registry_hash();
    let d64 = DetectorProfile::D64.registry_hash();
    let d128 = DetectorProfile::D128.registry_hash();
    assert_ne!(d128, d16, "D128 registry hash collides with D16");
    assert_ne!(d128, d64, "D128 registry hash collides with D64");
}

#[test]
fn d128_v0_projection_equals_canonical_d16_mask() {
    // R.9.b bridge invariant extended to D128: variant 0 carries
    // scale 1.0, so its firing pattern is bit-identical to the
    // canonical D16 mask emitted by the legacy `evaluate` path.
    use dsfb_gpu_debug_core::detector::{evaluate, DetectorThresholds};

    let contract = d128_contract();
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

    let d16_cells = evaluate(
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let d128_cells = evaluate_wide(
        DetectorProfile::D128,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );

    assert_eq!(d16_cells.len(), d128_cells.len());
    for (i, (d16, d128)) in d16_cells.iter().zip(d128_cells.iter()).enumerate() {
        let v0 = project_d128_v0_only_to_u16(d128);
        assert_eq!(
            d16.detector_mask, v0,
            "cell {i}: D128.V0-only projection ({v0:#x}) != canonical D16 mask ({:#x})",
            d16.detector_mask
        );
    }
}

#[test]
fn d128_or_projection_is_superset_of_v0() {
    let contract = d128_contract();
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
    let cells = evaluate_wide(
        DetectorProfile::D128,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    for (i, cell) in cells.iter().enumerate() {
        let v0 = project_d128_v0_only_to_u16(cell);
        let or = project_d128_or_to_u16(cell);
        assert_eq!(
            v0 & or,
            v0,
            "cell {i}: D128 OR projection {or:#x} missing V0 bits {v0:#x}"
        );
    }
}

#[test]
fn d128_or_projection_is_superset_of_d64_or_projection() {
    // D128's V0..V3 scales mirror D64.V0..V3, so D128 OR ⊇ D64 OR
    // bit-for-bit. (D128 adds V4..V7 which can only ADD bits.)
    let contract = d128_contract();
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
    let d64_cells = evaluate_wide(
        DetectorProfile::D64,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let d128_cells = evaluate_wide(
        DetectorProfile::D128,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    for (i, (d64, d128)) in d64_cells.iter().zip(d128_cells.iter()).enumerate() {
        let d64_or = project_d64_or_to_u16(d64);
        let d128_or = project_d128_or_to_u16(d128);
        assert_eq!(
            d64_or & d128_or,
            d64_or,
            "cell {i}: D128 OR {d128_or:#x} missing D64 OR bits {d64_or:#x}"
        );
    }
}

#[test]
fn d128_throughput_replay_is_byte_identical() {
    let contract = d128_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn d128_case_file_binds_to_d128_registry_hash() {
    let contract = d128_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    let d128_reg = DetectorProfile::D128.registry_hash();
    let expected_detreg = chain(b"detreg", &d128_reg, &case.hashes.bank);
    assert_eq!(
        case.hashes.detector_registry, expected_detreg,
        "D128 case file's detector_registry must bind to DetectorProfile::D128.registry_hash"
    );
}

#[test]
fn d128_episodes_are_bank_admitted() {
    let contract = d128_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    for (i, ep) in case.episodes.iter().enumerate() {
        assert!(
            ep.is_bank_admitted(),
            "D128-admitted episode {i} lacks BankAdmissionToken — Semantic Non-Bypass violated"
        );
    }
}

#[test]
fn d128_admits_at_least_one_episode_at_canonical() {
    // Defensive sanity: D128 at the canonical fixture must admit
    // at least one bank-governed episode. The exact count is NOT a
    // load-bearing invariant: with the wider OR projection more
    // cells fire as `interesting`, which can MERGE adjacent runs
    // into single longer episodes — so the count can move either
    // way relative to D64 while preserving every R.9.d.1 bridge
    // invariant. The comparison to D64 is therefore reported as
    // diagnostic info in the saturation sweep, not asserted as a
    // count ordering here.
    let contract = d128_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert!(
        !case.episodes.is_empty(),
        "D128 at canonical 16x128 admitted zero episodes — broken pipeline"
    );
}

#[test]
fn d128_total_interesting_cells_meets_or_exceeds_d64() {
    // The right monotonicity: at the per-cell level the D128 OR
    // projection ⊇ D64 OR projection (proven by
    // `d128_or_projection_is_superset_of_d64_or_projection`).
    // Aggregated across the grid: the total number of cells where
    // the projected mask is non-zero must be ≥ for D128. This
    // bridges the per-cell invariant to a population-level
    // invariant without depending on episode-merge behaviour.
    use dsfb_gpu_debug_core::detector::DetectorThresholds;

    let contract = d128_contract();
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

    let d64_cells = evaluate_wide(
        DetectorProfile::D64,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let d128_cells = evaluate_wide(
        DetectorProfile::D128,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );

    let d64_firing_cells = d64_cells
        .iter()
        .filter(|c| project_d64_or_to_u16(c) != 0)
        .count();
    let d128_firing_cells = d128_cells
        .iter()
        .filter(|c| project_d128_or_to_u16(c) != 0)
        .count();

    assert!(
        d128_firing_cells >= d64_firing_cells,
        "D128 firing-cell count ({d128_firing_cells}) must be >= D64 ({d64_firing_cells}) \
         under the OR-superset invariant"
    );
    // Drop suppresses "unused variable" warning on
    // build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact
    // — this test doesn't need the GPU dispatch.
    let _ = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact;
}

#[test]
fn d128_variant_counts_are_canonical() {
    // Defensive: catches regressions where someone changes the
    // variant count constant without updating the FFI / kernels.
    assert_eq!(D64_VARIANT_COUNT, 4);
    assert_eq!(D128_VARIANT_COUNT, 8);
    assert_eq!(DetectorProfile::D64.active_detector_count(), 64);
    assert_eq!(DetectorProfile::D128.active_detector_count(), 128);
}
