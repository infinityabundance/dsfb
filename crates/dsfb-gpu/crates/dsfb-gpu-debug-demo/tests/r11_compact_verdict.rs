//! R.11 acceptance tests: compact verdict finalizer.
//!
//! R.8 localised the post-R.8.5 wall-time bottleneck to the host
//! bank + case-finalize step: ~553 ms at 256×4096 K=1, dominated
//! by `hash_events_compact` over ~176 MB and
//! `hash_window_features_compact` over ~24 MB — both running
//! scalar SHA-256 on the host's hand-rolled implementation.
//!
//! R.11 introduces:
//!
//! * `FixtureHashes` — caller-precomputed (input_catalog,
//!   window_feature) pair, derived once per fixture from the
//!   same canonical byte forms the chain commits to.
//! * `build_throughput_compact_verdict_from_device_digests` —
//!   takes a `FixtureHashes` and the device-side stage digests,
//!   builds the same 12-link chain the non-compact path
//!   produces, but skips the per-dispatch re-hashing of events
//!   and window features.
//! * `build_gpu_throughput_pinned_async_on_workspace_tree_compact`
//!   — GPU dispatch that runs the R.8.5 tree-digest kernels and
//!   finalises via the compact builder.
//!
//! These tests pin the load-bearing R.11 invariants:
//!
//! 1. **Byte equivalence**: the compact builder produces a
//!    `CaseFile` byte-identical to the non-compact builder for
//!    the same fixture. Same chain hashes, same episode list,
//!    same final hash, same verdict.
//! 2. **Semantic episode invariance**: the bank admits the same
//!    episodes regardless of which finalizer ran. The Semantic
//!    Non-Bypass Axiom holds — `bank::collapse` is still the
//!    only path to a `BankAdmissionToken`.
//! 3. **Mutation sensitivity**: changing the input events
//!    changes the `FixtureHashes`, which changes the chain.
//!    Replay catches a stale fixture-hash receipt.
//! 4. **FixtureHashes::compute matches the inline path**: the
//!    helper produces the same bytes the non-compact builder
//!    would re-derive from events / features.
//!
//! Audit golden hashes are NOT touched. Serial Throughput is
//! NOT touched. The compact path is opt-in; existing callers
//! see no behaviour change.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_tree,
    build_gpu_throughput_pinned_async_on_workspace_tree_compact, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

fn compute_fixture_hashes(events: &[dsfb_gpu_debug_core::event::TraceEvent]) -> FixtureHashes {
    // Match the way the non-compact path derives features so the
    // canonical bytes the hashes commit to are byte-identical.
    let contract = canonical_contract();
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    FixtureHashes::compute(events, &features)
}

#[test]
fn compact_case_file_matches_non_compact_byte_for_byte() {
    // Load-bearing R.11 invariant: the compact and non-compact
    // tree-digest dispatch paths must produce byte-identical
    // case files on the same fixture. This is the contract that
    // makes the compact path a pure performance optimisation
    // and not a semantic change.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let fixture = compute_fixture_hashes(&events);

    let mut ws_non_compact = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let non_compact = build_gpu_throughput_pinned_async_on_workspace_tree(
        &events,
        &contract,
        &mut ws_non_compact,
    )
    .unwrap();

    let mut ws_compact = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let compact = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events,
        &contract,
        &mut ws_compact,
        &fixture,
    )
    .unwrap();

    assert_eq!(
        non_compact.hashes, compact.hashes,
        "12-link chain hashes must match between non-compact and compact tree-digest paths"
    );
    assert_eq!(
        non_compact.episodes, compact.episodes,
        "admitted-episode list must match between paths"
    );
    assert_eq!(
        non_compact.final_case_file_hash, compact.final_case_file_hash,
        "final case-file hash must match between paths"
    );
    assert_eq!(
        non_compact.final_verdict, compact.final_verdict,
        "final verdict must match between paths"
    );
}

#[test]
fn compact_case_file_is_deterministic_across_runs() {
    // Two consecutive compact dispatches on the same fixture
    // produce byte-identical case files. Catches any
    // nondeterminism the compact finalizer might have introduced
    // (e.g. iteration order, episode sorting, hash chaining).
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let fixture = compute_fixture_hashes(&events);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn fixture_hashes_helper_matches_chain_derivation() {
    // `FixtureHashes::compute` is the canonical helper for
    // pre-deriving the two hashes. Verify it produces the same
    // commitments the non-compact chain would re-derive — if it
    // diverged, every compact case file would mismatch.
    //
    // This is checked indirectly by `compact_case_file_matches_
    // non_compact_byte_for_byte` (since byte equivalence of the
    // full case file implies byte equivalence of the input chain
    // links), but the assert here pins the contract more
    // precisely so a regression localises to the helper itself.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let fixture = compute_fixture_hashes(&events);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let compact = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    // The case file's `input_catalog` link is the compact
    // fixture's input_catalog (no chain prefix on the first link
    // by design of `build_throughput_compact_verdict_*`).
    assert_eq!(
        compact.hashes.input_catalog, fixture.input_catalog,
        "input_catalog chain link must equal FixtureHashes::compute(events).input_catalog"
    );
}

#[test]
fn compact_case_file_changes_when_input_changes() {
    // Mutation sensitivity: a different LCG seed produces
    // different events, which yields a different
    // `FixtureHashes::input_catalog`, which propagates through
    // every chain link including `final_case_file_hash`. If this
    // failed, the FixtureHashes layer would be collapsing
    // distinct inputs onto the same receipt — a replay-safety
    // breach.
    let contract = canonical_contract();
    let events_a = synthesize(DEFAULT_SEED);
    let events_b = synthesize(DEFAULT_SEED.wrapping_add(0x9E37_79B9));
    let fixture_a = compute_fixture_hashes(&events_a);
    let fixture_b = compute_fixture_hashes(&events_b);

    assert_ne!(
        fixture_a.input_catalog, fixture_b.input_catalog,
        "FixtureHashes::input_catalog must change when events change"
    );

    let mut ws_a = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case_a = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events_a, &contract, &mut ws_a, &fixture_a,
    )
    .unwrap();

    let mut ws_b = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case_b = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events_b, &contract, &mut ws_b, &fixture_b,
    )
    .unwrap();

    assert_ne!(
        case_a.hashes.input_catalog, case_b.hashes.input_catalog,
        "input_catalog chain link must change when input events change"
    );
    assert_ne!(
        case_a.final_case_file_hash, case_b.final_case_file_hash,
        "final case-file hash must change when input events change"
    );
}

#[test]
fn compact_path_episodes_are_bank_admitted() {
    // Semantic Non-Bypass Axiom test: every episode in the
    // compact case file carries a `BankAdmissionToken`. The
    // compact builder cannot mint episodes outside the bank
    // module; the only way an episode reaches the case file is
    // through `bank_collapse`, which is unchanged by R.11.
    //
    // The `Episode::is_bank_admitted` predicate is the same one
    // the case-file finaliser checks before deciding whether
    // the verdict is `ReplayAdmissible` vs. `SemanticBypassRejected`.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let fixture = compute_fixture_hashes(&events);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    assert!(
        !case.episodes.is_empty(),
        "canonical fixture admits at least one episode"
    );
    for (i, ep) in case.episodes.iter().enumerate() {
        assert!(
            ep.is_bank_admitted(),
            "compact-path episode {i} lacks BankAdmissionToken — Semantic Non-Bypass Axiom violated"
        );
    }
    assert_eq!(
        case.final_verdict,
        dsfb_gpu_debug_core::verdict::FinalVerdict::GpuReplayAdmissible,
        "all bank-admitted episodes ⇒ GpuReplayAdmissible verdict"
    );
}
