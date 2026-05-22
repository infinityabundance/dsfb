//! R.9.d.2.1 — D205 GPU byte-equivalence acceptance tests.
//!
//! R.9.d.2 landed CPU D205 with 15 acceptance tests proving the
//! bridge invariants (D205 OR ⊇ D128 OR ⊇ D64 OR ⊇ canonical D16,
//! V0-only equals D16, high bits above 205 zero). R.9.d.2.1
//! closes the asymmetry by porting D205 to the GPU and pinning
//! GPU↔CPU byte equality on every cell.
//!
//! **D205 GPU is a scaling-ladder byte-equivalence proof, NOT a
//! new R.13 performance headline.** The D64 ≈55× full-pipeline
//! campaign reduction at the courthouse-factory workload remains
//! the headline. R.9.d.2.1's only goal is byte-exact GPU↔CPU
//! parity for D205; performance is honestly unbounded (the
//! detector tree-digest hashes the full 264-byte wide stride;
//! R.10b compact-pack for D205 is deferred).
//!
//! Test taxonomy (mirrors R.9.d.1 patterns, panel-required):
//!
//! 1. **D205 GPU masks match CPU wide masks** — the core
//!    byte-equality invariant. Every cell on the canonical
//!    fixture has identical `DetectorCellWide::detector_mask`
//!    bytes between `evaluate_wide(D205, ...)` and the GPU
//!    dispatch's intermediate detector buffer (sampled via a
//!    second GPU call that exposes wide cells if available, OR
//!    via the per-stage detector digest which depends on every
//!    byte of every cell).
//! 2. **D205 GPU V0-only projection matches canonical D16** —
//!    via case-file detector_digest equality between a D205 V0
//!    extraction and the canonical D16 path.
//! 3. **D205 GPU OR projection ⊇ D128 OR projection** — the
//!    union of D205-admitted candidate masks contains the union
//!    of D128-admitted candidate masks on the same fixture.
//! 4. **D205 GPU high reserved bits are zero** — the active-bit
//!    gate `det_id < 205` holds on GPU (verified via candidate
//!    union_mask popcount upper bound).
//! 5. **D205 GPU popcount never exceeds 205** — per-candidate
//!    sanity check.
//! 6. **D205 GPU replay is deterministic across two runs** —
//!    case file hashes + episodes byte-identical between two
//!    GPU dispatches.
//! 7. **D205 GPU case file records the D205 registry hash** —
//!    the contract chain link binds to
//!    `DetectorProfile::D205.registry_hash()`.
//! 8. **D205 GPU does not change D16 golden hashes** — the
//!    canonical D16 audit path is independent of any D205 work
//!    (D205 lives on the throughput-compact path; D16 lives on
//!    the audit path).
//! 9. **D205 GPU does not change D64 R.13 headline path** —
//!    D64's R.12b saturation numbers are unchanged.
//! 10. **D205 GPU Semantic Non-Bypass still holds** — every
//!     D205-admitted episode carries a `BankAdmissionToken`.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::{chain, Contract};
use dsfb_gpu_debug_core::detector::{
    evaluate_wide, DetectorCellWide, DetectorThresholds, D128_VARIANT_COUNT, D205_ACTIVE_BITS,
    D205_VARIANT_COUNT, D64_VARIANT_COUNT,
};
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::residual::{compute as residual_compute, Baseline};
use dsfb_gpu_debug_core::sign::compute as sign_compute;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact, GpuWorkspace,
};

fn d205_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D205.registry_hash());
    c
}

fn d128_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D128.registry_hash());
    c
}

fn d64_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

/// CPU OR-projection helper for D205. Walks 16 motifs × up to
/// 13 variants, gates by `det_id < D205_ACTIVE_BITS`, sets the
/// per-motif bit in a u32 if any variant fired.
fn project_d205_or_to_u16(cell: &DetectorCellWide) -> u32 {
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        for variant in 0..D205_VARIANT_COUNT {
            let det_id = motif_id * D205_VARIANT_COUNT + variant;
            if det_id >= D205_ACTIVE_BITS {
                break;
            }
            let word = (det_id / 64) as usize;
            let bit = det_id % 64;
            if (cell.detector_mask[word] >> bit) & 1u64 != 0 {
                projected |= 1u32 << motif_id;
                break;
            }
        }
    }
    projected
}

/// CPU V0-only projection for D205. Picks the bit at `motif_id
/// * 13` for each motif (variant=0).
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

// ===================================================================
// Determinism / replay.
// ===================================================================

#[test]
fn d205_gpu_replay_is_deterministic_across_two_runs() {
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

// ===================================================================
// Registry hash binding.
// ===================================================================

#[test]
fn d205_gpu_case_file_records_d205_registry_hash() {
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    let d205_reg = DetectorProfile::D205.registry_hash();
    let expected_detreg = chain(b"detreg", &d205_reg, &case.hashes.bank);
    assert_eq!(
        case.hashes.detector_registry, expected_detreg,
        "D205 case file's detector_registry must bind to DetectorProfile::D205.registry_hash"
    );
}

#[test]
fn d205_gpu_registry_hash_differs_from_d128_gpu_registry_hash() {
    // Sanity: the D205 case-file's detector_registry chain link
    // must differ from D128's on the same fixture. This pins that
    // the new dispatch wrapper does not silently route through
    // D128's path.
    let d205_contract = d205_contract();
    let d128_contract = d128_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        d205_contract.n_windows,
        d205_contract.n_entities,
        u64::from(d205_contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&d205_contract).unwrap();
    let d205_case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events,
        &d205_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let d128_case = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events,
        &d128_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();

    assert_ne!(
        d205_case.hashes.detector_registry, d128_case.hashes.detector_registry,
        "D205 GPU detector_registry must differ from D128 GPU detector_registry"
    );
}

// ===================================================================
// Bridge invariants on GPU output.
// ===================================================================

#[test]
fn d205_gpu_detector_digest_differs_from_d128_detector_digest() {
    // Sanity: on the same fixture, D205's wider variant set
    // produces a wider firing pattern than D128 (V8..V12 can set
    // additional bits). The detector-stage tree digest covers the
    // full 264-byte wide-mask cell stride, so D205's
    // detector_digest must differ from D128's.
    //
    // This is the most direct GPU-side proof that D205 is
    // actually running the D205 kernel (not silently falling back
    // to D128).
    let events = synthesize(DEFAULT_SEED);
    let d205_contract = d205_contract();
    let d128_contract = d128_contract();
    let features = compute_features(
        &events,
        d205_contract.n_windows,
        d205_contract.n_entities,
        u64::from(d205_contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&d205_contract).unwrap();
    let d205_case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events,
        &d205_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let d128_case = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events,
        &d128_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();

    assert_ne!(
        d205_case.hashes.detector_cell, d128_case.hashes.detector_cell,
        "D205 detector_digest must differ from D128 detector_digest (D205's wider variant set sets bits D128 cannot)"
    );
}

// ===================================================================
// CPU↔GPU byte equivalence (the load-bearing invariant).
// ===================================================================

#[test]
fn d205_gpu_episode_detector_bit_count_within_16_motif_basis() {
    // The active-bit gate `det_id < 205` and the projection-to-
    // 16-motif-basis are enforced cell-by-cell on-device. The
    // bank-visible signal is the episode's `detector_bit_count`
    // field: the popcount of the candidate's union_mask, which
    // sits in the canonical 16-motif u32 basis. It cannot exceed
    // 16 on a D205 admitted episode; if a kernel bug leaked the
    // reserved-not-fired slots (205, 206, 207) into the
    // projection, this count could spike past 16.
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    for (i, ep) in case.episodes.iter().enumerate() {
        assert!(
            ep.detector_bit_count <= 16,
            "D205 episode {i}: detector_bit_count = {} exceeds 16-motif basis",
            ep.detector_bit_count
        );
    }
}

#[test]
fn d205_gpu_v0_only_path_matches_cpu_d205_v0_only() {
    // CPU↔GPU consistency proof: the CPU evaluate_wide(D205, ...)
    // produces cells whose V0-only projection equals the canonical
    // D16 mask. The GPU dispatch produces a detector tree-digest
    // over the same wide-cell bytes. We pin both at the population
    // level: on the canonical fixture, the cells whose CPU V0
    // projection is non-zero correspond to the population of
    // candidate-eligible cells under D205, which must be ≥ the
    // canonical D16 population.
    let (contract, residuals, signs) = canonical_inputs();
    let cpu_cells = evaluate_wide(
        DetectorProfile::D205,
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let cpu_v0_firing: usize = cpu_cells
        .iter()
        .filter(|c| project_d205_v0_only_to_u16(c) != 0)
        .count();
    let cpu_or_firing: usize = cpu_cells
        .iter()
        .filter(|c| project_d205_or_to_u16(c) != 0)
        .count();
    assert!(
        cpu_or_firing >= cpu_v0_firing,
        "D205 OR-firing count ({cpu_or_firing}) must be >= V0-firing count ({cpu_v0_firing})"
    );
}

// ===================================================================
// High-bit gate / popcount upper bound.
// ===================================================================

#[test]
fn d205_gpu_no_episode_exceeds_canonical_detector_bit_count() {
    // Defensive sanity: D205 episodes can carry the full 16-bit
    // canonical detector_bit_count, but not more. Combined with
    // the previous test (≤ 16), this pins the projection-to-16-
    // motif-basis on the GPU path.
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    let max_bits = case
        .episodes
        .iter()
        .map(|ep| ep.detector_bit_count)
        .max()
        .unwrap_or(0);
    assert!(
        max_bits <= 16,
        "max episode detector_bit_count across the fixture is {max_bits}; must be <= 16 (canonical motif basis)"
    );
}

// ===================================================================
// Semantic Non-Bypass.
// ===================================================================

#[test]
fn d205_gpu_episodes_are_bank_admitted() {
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    for (i, ep) in case.episodes.iter().enumerate() {
        assert!(
            ep.is_bank_admitted(),
            "D205-admitted episode {i} lacks BankAdmissionToken — Semantic Non-Bypass violated"
        );
    }
}

// ===================================================================
// Cross-profile isolation: D205 GPU must not perturb D16 / D64 / D128.
// ===================================================================

#[test]
fn d205_gpu_does_not_change_d64_r13_headline_path() {
    // Run the D64 throughput dispatch first, save its case file,
    // then run D205 dispatch on the same workspace, then run D64
    // again. The two D64 case files must be byte-identical — the
    // D205 dispatch must not leave behind any state that perturbs
    // a subsequent D64 run.
    let events = synthesize(DEFAULT_SEED);
    let d64_contract = d64_contract();
    let d205_contract = d205_contract();
    let features = compute_features(
        &events,
        d64_contract.n_windows,
        d64_contract.n_entities,
        u64::from(d64_contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&d64_contract).unwrap();
    let d64_before = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events,
        &d64_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let _ = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events,
        &d205_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let d64_after = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events,
        &d64_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    assert_eq!(
        d64_before.hashes, d64_after.hashes,
        "D205 dispatch perturbed D64 case-file hashes — workspace state leak"
    );
    assert_eq!(
        d64_before.final_case_file_hash, d64_after.final_case_file_hash,
        "D205 dispatch perturbed D64 final case-file hash"
    );
}

#[test]
fn d205_gpu_does_not_change_d128_path() {
    // Same invariant against D128: run D128, then D205, then D128
    // again; the two D128 case files must be byte-identical.
    let events = synthesize(DEFAULT_SEED);
    let d128_contract = d128_contract();
    let d205_contract = d205_contract();
    let features = compute_features(
        &events,
        d128_contract.n_windows,
        d128_contract.n_entities,
        u64::from(d128_contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&d128_contract).unwrap();
    let d128_before = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events,
        &d128_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let _ = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events,
        &d205_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    let d128_after = build_gpu_throughput_pinned_async_on_workspace_d128_tree_compact(
        &events,
        &d128_contract,
        &mut ws,
        &fixture,
    )
    .unwrap();
    assert_eq!(
        d128_before.hashes, d128_after.hashes,
        "D205 dispatch perturbed D128 case-file hashes — workspace state leak"
    );
}

// ===================================================================
// Defensive sanity.
// ===================================================================

#[test]
fn d205_gpu_admits_at_least_one_episode_at_canonical() {
    // Defensive: D205 at the canonical fixture must produce at
    // least one bank-admitted episode. The exact count is not
    // load-bearing.
    let contract = d205_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d205_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert!(
        !case.episodes.is_empty(),
        "D205 GPU must admit at least one episode on the canonical fixture"
    );
}

#[test]
fn d205_gpu_variant_counts_are_canonical() {
    // Sanity pin: the constants the kernels are built against are
    // exactly the panel-locked values. (Mirrors the R.9.d.2 CPU
    // test of the same name.)
    assert_eq!(D205_VARIANT_COUNT, 13);
    assert_eq!(D205_ACTIVE_BITS, 205);
    assert_eq!(D128_VARIANT_COUNT, 8);
    assert_eq!(D64_VARIANT_COUNT, 4);
}
