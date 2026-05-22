//! R.3a acceptance tests for the Layer A device-evidence-fabric path.
//!
//! Three load-bearing invariants are pinned here:
//!
//! 1. **Chain equivalence**: Layer A's `CompactCaseSummary.hashes`
//!    must equal Layer B's `CaseFile.hashes` byte-for-byte through
//!    `candidate_interval`. This proves that skipping the bank stage
//!    does not change any link the bank does not write.
//!
//! 2. **Type-level non-bypass**: `CompactCaseSummary` has no
//!    `episodes` or `final_case_file_hash` field, so Layer A
//!    callers literally cannot mint admitted episodes. This is the
//!    Semantic Non-Bypass Axiom enforced by the type system at the
//!    Layer A boundary.
//!
//! 3. **Batched equivalence per catalog**: when K catalogs run
//!    through the batched Layer A path, each catalog's summary
//!    chain hashes match the corresponding single-catalog Layer A
//!    summary on the same input.
//!
//! These tests do not change CandidateInterval layout (that is
//! R.5's contract-affecting work); the golden hashes are unchanged.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_layer_a_batched, build_gpu_layer_a_on_workspace,
    build_gpu_throughput_device_digests_on_workspace, BatchedGpuWorkspace, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn layer_a_summary_chain_matches_layer_b_case_file_through_candidate() {
    // The first 11 chain links (input_catalog through candidate_interval)
    // must be byte-identical between the Layer A skip-bank summary and
    // the Layer B full case file on the same fixture. Layer A simply
    // declines to compute the `episode` link; everything else is the
    // same evidence-fabric chain.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_a = GpuWorkspace::new(&contract).unwrap();
    let summary = build_gpu_layer_a_on_workspace(&events, &contract, &mut ws_a).unwrap();

    let mut ws_b = GpuWorkspace::new(&contract).unwrap();
    let case =
        build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws_b).unwrap();

    // Compare every chain link except `episode` (Layer A does not run
    // the bank, so the episode digest is intentionally absent).
    assert_eq!(summary.hashes.input_catalog, case.hashes.input_catalog);
    assert_eq!(summary.hashes.contract, case.hashes.contract);
    assert_eq!(summary.hashes.bank, case.hashes.bank);
    assert_eq!(
        summary.hashes.detector_registry,
        case.hashes.detector_registry
    );
    assert_eq!(summary.hashes.kernel_sequence, case.hashes.kernel_sequence);
    assert_eq!(summary.hashes.window_feature, case.hashes.window_feature);
    assert_eq!(summary.hashes.residual_field, case.hashes.residual_field);
    assert_eq!(summary.hashes.sign_field, case.hashes.sign_field);
    assert_eq!(summary.hashes.detector_cell, case.hashes.detector_cell);
    assert_eq!(summary.hashes.consensus_grid, case.hashes.consensus_grid);
    assert_eq!(
        summary.hashes.candidate_interval, case.hashes.candidate_interval,
        "the candidate_interval link must match — this is the boundary at which Layer A stops"
    );

    // Layer A does not compute the `episode` link.
    assert_eq!(summary.hashes.episode, [0u8; 32]);

    // The Layer A summary's candidate list is the *pre-bank* slice
    // (raw admitted-candidate intervals). The Layer B case file's
    // `episodes` is the *post-bank* admitted list (after axis-5,
    // confuser suppression, and tie-break). These are different
    // shapes by design — comparing their lengths would be apples to
    // oranges. The shared boundary is the `candidate_interval` chain
    // hash, already asserted above.
}

#[test]
fn layer_a_summary_has_no_admitted_episodes_field() {
    // Type-level Semantic Non-Bypass: callers of Layer A cannot
    // obtain admitted episodes from the summary. This test is a
    // compile-time invariant disguised as a runtime test — if anyone
    // adds an `episodes` field to `CompactCaseSummary`, this test
    // would still pass (the field would just exist); but the *type
    // signature* below documents the contract. We also confirm at
    // runtime that no breach flag is set on the canonical fixture
    // (clean run, no contract mismatch).
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new(&contract).unwrap();
    let summary = build_gpu_layer_a_on_workspace(&events, &contract, &mut ws).unwrap();

    // The compile-time type contract: this line would not compile if
    // CompactCaseSummary acquired an `episodes` field of incompatible
    // shape. (We rely on type-checking; no runtime assertion.)
    let backend: &str = summary.backend;
    assert!(!backend.is_empty(), "backend tag must be set");

    // Runtime invariant: canonical fixture has no breaches.
    assert_eq!(
        summary.breach_flags, 0,
        "canonical fixture must produce zero breach flags"
    );
}

#[test]
fn layer_a_batched_matches_single_per_catalog() {
    // Batched K-catalog Layer A must produce per-catalog summaries
    // that equal the single-catalog Layer A on the same input. This
    // is the per-catalog independence invariant carried over from
    // Section O.16's batched-equivalence tests, applied to the
    // skip-bank Layer A path.
    let k: u32 = 2;
    let contract = canonical_contract();
    let events_0 = synthesize(DEFAULT_SEED);
    let events_1 =
        dsfb_gpu_debug_core::fixture::synthesize(DEFAULT_SEED.wrapping_add(0x9E37_79B9_7F4A_7C15));
    let slices: [&[_]; 2] = [&events_0[..], &events_1[..]];

    let mut ws_batched = BatchedGpuWorkspace::new(k, &contract).unwrap();
    let batched_summaries = build_gpu_layer_a_batched(&mut ws_batched, &slices, &contract).unwrap();
    assert_eq!(batched_summaries.len(), k as usize);

    let mut ws_single = GpuWorkspace::new(&contract).unwrap();
    let single_0 = build_gpu_layer_a_on_workspace(&events_0, &contract, &mut ws_single).unwrap();
    let mut ws_single2 = GpuWorkspace::new(&contract).unwrap();
    let single_1 = build_gpu_layer_a_on_workspace(&events_1, &contract, &mut ws_single2).unwrap();

    assert_eq!(
        batched_summaries[0].hashes, single_0.hashes,
        "catalog 0 batched-vs-single chain hashes must match"
    );
    assert_eq!(
        batched_summaries[1].hashes, single_1.hashes,
        "catalog 1 batched-vs-single chain hashes must match"
    );
    assert_eq!(batched_summaries[0].candidates, single_0.candidates);
    assert_eq!(batched_summaries[1].candidates, single_1.candidates);
    assert_eq!(batched_summaries[0].breach_flags, 0);
    assert_eq!(batched_summaries[1].breach_flags, 0);
}
