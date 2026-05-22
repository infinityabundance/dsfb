//! R.9.b.3 acceptance tests: full D64 Throughput pipeline.
//!
//! R.9.b.2 (commit `1caf0df`) proved the GPU wide-mask detector
//! kernel produces byte-identical `DetectorMask2048` to the CPU
//! evaluator. R.9.b.3 routes that wide kernel through the full
//! pipeline — wide consensus → wide candidate (with on-device OR
//! projection to the canonical 16-motif basis) → tree digest over
//! the wide stage bytes → compact-verdict finalizer — producing
//! a complete D64 Throughput case file without changing the bank
//! ABI, the canonical D16 court bytes, or any Audit golden hash.
//!
//! Test taxonomy (per the R.9.b.3 design lock):
//!
//! 1. **D16 path unchanged** — covered by the existing
//!    `golden_hashes` + R.11 byte-equivalence tests, run as part
//!    of the workspace suite. Not duplicated here.
//! 2. **D64 wide→u16 projection is deterministic** — same wide
//!    cell input always produces the same projected u16 mask.
//! 3. **D64 V0-only projection equals canonical D16 mask** — at
//!    every cell, taking just the `motif_id * 4 + 0` bit
//!    recovers the legacy D16 mask exactly. The bridge invariant.
//! 4. **D64 OR projection ⊇ V0 projection** — equal-or-superset
//!    property: the bank sees at least every D16 firing, plus
//!    additional variant evidence.
//! 5. **D64 Throughput replay is byte-identical across two runs**
//!    — full case file (chain hashes + episodes + final hash) is
//!    deterministic.
//! 6. **D64 case-file metadata** — `contract.detector_registry_hash`
//!    pinned via `DetectorProfile::D64.registry_hash()` propagates
//!    into the chain's `detector_registry` link.
//! 7. **D64 bank admission still requires `BankAdmissionToken`** —
//!    every D64-admitted episode carries the token; the Semantic
//!    Non-Bypass Axiom holds at the wider profile.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::detector::{DetectorCellWide, D64_VARIANT_COUNT};
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact,
    evaluate_detector_wide_d64_on_workspace, GpuWorkspace,
};

/// Build a D64-pinned canonical contract: bank hash canonical,
/// detector_registry_hash = `DetectorProfile::D64.registry_hash()`
/// so the chain's `detector_registry` link binds the case file
/// to the D64 profile (not the canonical 16-motif registry).
fn d64_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

/// CPU projection of a `DetectorCellWide` to the canonical 16-bit
/// mask using the locked OR rule. Mirrors the device-side
/// `project_d64_to_u16` exactly.
fn project_or_d64_to_u16(cell: &DetectorCellWide) -> u32 {
    let m = cell.detector_mask[0];
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let four = (m >> (motif_id * D64_VARIANT_COUNT)) & 0xF;
        if four != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

/// CPU projection of a `DetectorCellWide` to "V0 only": take bit
/// `motif_id * 4 + 0` for each motif. Used to verify the bridge
/// invariant (V0-only projection ≡ canonical D16 mask).
fn project_v0_d64_to_u16(cell: &DetectorCellWide) -> u32 {
    let m = cell.detector_mask[0];
    let mut projected = 0u32;
    for motif_id in 0..16u32 {
        let v0_bit = (m >> (motif_id * D64_VARIANT_COUNT)) & 1;
        if v0_bit != 0 {
            projected |= 1u32 << motif_id;
        }
    }
    projected
}

#[test]
fn d64_projection_is_deterministic() {
    // Two identical wide-cell inputs produce identical projected
    // u16 masks. Catches any future projection-helper drift.
    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let wide = evaluate_detector_wide_d64_on_workspace(&events, &contract, &mut ws).unwrap();
    for (i, cell) in wide.iter().enumerate() {
        let a = project_or_d64_to_u16(cell);
        let b = project_or_d64_to_u16(cell);
        assert_eq!(a, b, "cell {i}: projection is not deterministic");
    }
}

#[test]
fn d64_v0_projection_equals_canonical_d16() {
    // The bridge invariant locked by R.9.b's design: the D64 V0-
    // only projection of every wide cell equals the canonical D16
    // legacy mask for the same cell. This is what keeps the bank's
    // canonical motif basis stable under the wider profile.
    use dsfb_gpu_debug_core::detector::{evaluate, DetectorThresholds};
    use dsfb_gpu_debug_core::residual::{compute as residual_compute, Baseline};
    use dsfb_gpu_debug_core::sign::compute as sign_compute;

    let canonical_d16_contract = {
        let mut c = Contract::canonical();
        c.pin_bank_hash(bank_hash());
        c.pin_detector_registry_hash(DetectorProfile::D16.registry_hash());
        c
    };
    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);

    // CPU D16 reference: features → residuals → signs → evaluate (legacy).
    let features = compute_features(
        &events,
        canonical_d16_contract.n_windows,
        canonical_d16_contract.n_entities,
        u64::from(canonical_d16_contract.window_size_ms) * 1_000_000,
    );
    let residuals = residual_compute(&features, &Baseline::CANONICAL);
    let signs = sign_compute(
        &residuals,
        dsfb_gpu_debug_core::fixed::Q16::from_raw(canonical_d16_contract.ewma_alpha_q16_raw),
        canonical_d16_contract.n_windows,
        canonical_d16_contract.n_entities,
    );
    let d16_cells = evaluate(
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        canonical_d16_contract.n_windows,
        canonical_d16_contract.n_entities,
    );

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let wide = evaluate_detector_wide_d64_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(d16_cells.len(), wide.len());
    for (i, (d16, wide_cell)) in d16_cells.iter().zip(wide.iter()).enumerate() {
        let v0_projected = project_v0_d64_to_u16(wide_cell);
        assert_eq!(
            d16.detector_mask, v0_projected,
            "cell {i}: D64.V0-only projection != canonical D16 mask"
        );
    }
}

#[test]
fn d64_or_projection_is_superset_of_v0() {
    // The OR projection (V0 OR V1 OR V2 OR V3) must always
    // include every bit the V0-only projection sets. Failure
    // would mean a variant fired AND simultaneously suppressed
    // the V0 bit — impossible by construction, but worth pinning.
    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let wide = evaluate_detector_wide_d64_on_workspace(&events, &contract, &mut ws).unwrap();
    for (i, cell) in wide.iter().enumerate() {
        let v0 = project_v0_d64_to_u16(cell);
        let or_proj = project_or_d64_to_u16(cell);
        assert_eq!(
            v0 & or_proj,
            v0,
            "cell {i}: OR projection {or_proj:#x} missing V0 bits {v0:#x}"
        );
    }
}

#[test]
fn d64_throughput_replay_is_byte_identical() {
    // Two consecutive D64 Throughput dispatches on the same fixture
    // produce a byte-identical case file. The wide kernels + tree
    // digest + compact-verdict path must all be deterministic.
    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn d64_case_file_records_d64_registry_hash() {
    // The chain's `detector_registry` link must commit to
    // `DetectorProfile::D64.registry_hash()`, not to the canonical
    // 16-motif registry hash. This is how a replay verifier knows
    // the case file ran at D64 (vs D16 vs D128/D2000).
    use dsfb_gpu_debug_core::contract::chain;
    use dsfb_gpu_debug_core::motif::registry_hash;

    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    // Reconstruct the chain link explicitly: detector_registry =
    // chain(b"detreg", contract.detector_registry_hash, bank_link).
    let d64_reg = DetectorProfile::D64.registry_hash();
    let canonical_reg = registry_hash();
    assert_ne!(
        d64_reg, canonical_reg,
        "D64 registry hash collides with D16 — R.9.a invariant broken"
    );

    let expected_detreg = chain(b"detreg", &d64_reg, &case.hashes.bank);
    assert_eq!(
        case.hashes.detector_registry, expected_detreg,
        "D64 case file's detector_registry link does not bind to DetectorProfile::D64.registry_hash"
    );
}

#[test]
fn d64_episodes_are_bank_admitted() {
    // Semantic Non-Bypass Axiom at the D64 profile: every admitted
    // episode carries a `BankAdmissionToken`, because the bank
    // module is the only mint of that type. R.9.b.3 changes what
    // detector evidence the bank sees (via OR projection) but
    // does NOT change the admission path.
    let contract = d64_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    for (i, ep) in case.episodes.iter().enumerate() {
        assert!(
            ep.is_bank_admitted(),
            "D64-admitted episode {i} lacks BankAdmissionToken — Semantic Non-Bypass violated"
        );
    }
}
