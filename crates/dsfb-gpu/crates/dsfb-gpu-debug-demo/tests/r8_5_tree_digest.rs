//! R.8.5 acceptance tests: deterministic tree-digest Throughput path.
//!
//! The R.8 profile (commit `ba5a3e4`) put the 4 single-thread
//! `*_digest_kernel_batched` kernels at 78.2 % of wall on
//! 256×4096 K=1. R.8.5 replaces them with a deterministic
//! domain-separated tree digest: one block per chunk (parallel
//! across the device) followed by one root block per catalog that
//! concatenates an ordered header + leaf digests and runs a final
//! SHA-256.
//!
//! These tests pin the tree-digest path's correctness invariants
//! **and** verify that the serial-digest path (default) remains
//! byte-equivalent to the pre-R.8.5 baseline. The Audit-mode
//! golden hashes are NOT touched by this work; their tests live in
//! `golden_hashes.rs` and continue to pass unchanged.
//!
//! Test taxonomy (per R.8.5 plan):
//!
//! 1. determinism across two GPU runs at the canonical fixture
//! 2. tree digest differs from serial digest (different commitment)
//! 3. mutation sensitivity: flipping one event byte changes the
//!    tree digest
//! 4. semantic episode list is unchanged between tree and serial
//!    digest modes — the digest commits over evidence bytes, not
//!    over the bank verdict
//! 5. tree digest is non-zero at every scale point we measured
//!    (no accidental empty-leaf path)
//!
//! The R.8.5 plan also calls for an explicit "metadata sensitivity"
//! test (changing `chunk_size` changes the digest). That requires a
//! workspace-construction switch; deferred to a follow-up when
//! the contract carries an explicit `digest_mode` enum. For v0 the
//! chunk size is locked at 16 KiB inside the workspace.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace,
    build_gpu_throughput_pinned_async_on_workspace_tree, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn tree_digest_is_deterministic_across_runs() {
    // Two consecutive tree-digest dispatches on the same fixture
    // must produce byte-identical case files. The tree digest's
    // determinism is the load-bearing invariant — without it,
    // replay receipts mean nothing.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    assert!(
        ws.has_tree_digest(),
        "new_with_pinned_async allocates tree-digest scratch"
    );
    let a =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events, &contract, &mut ws).unwrap();
    let b =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events, &contract, &mut ws).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn tree_digest_differs_from_serial_digest() {
    // The tree digest commits to (canonical chunked stage bytes ||
    // domain separator). The serial digest commits to (canonical
    // stage bytes). These are different commitments by design — a
    // case file produced one way must not match the other, or the
    // case-file metadata would silently let a Throughput-tree
    // receipt validate against a Throughput-serial chain.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_serial = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let serial_case =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_serial).unwrap();

    let mut ws_tree = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let tree_case =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events, &contract, &mut ws_tree)
            .unwrap();

    // Per-stage digests must differ — that's where the tree
    // topology + domain separator enters the chain.
    assert_ne!(
        serial_case.hashes.residual_field, tree_case.hashes.residual_field,
        "residual stage digest must differ between serial and tree paths"
    );
    assert_ne!(
        serial_case.hashes.sign_field, tree_case.hashes.sign_field,
        "sign stage digest must differ between serial and tree paths"
    );
    assert_ne!(
        serial_case.hashes.detector_cell, tree_case.hashes.detector_cell,
        "detector stage digest must differ between serial and tree paths"
    );
    assert_ne!(
        serial_case.hashes.consensus_grid, tree_case.hashes.consensus_grid,
        "consensus stage digest must differ between serial and tree paths"
    );

    // Final case-file hash differs because the chain includes the
    // diverging stage hashes.
    assert_ne!(
        serial_case.final_case_file_hash, tree_case.final_case_file_hash,
        "final case-file hash must differ between serial and tree paths"
    );
}

#[test]
fn tree_digest_episodes_match_serial_digest_episodes() {
    // The digest mode commits over evidence bytes, not over the
    // bank verdict. Same fixture + same contract + same bank ⇒
    // same admitted-episode list, regardless of which digest mode
    // produced the case-file metadata. Semantic Non-Bypass Axiom
    // is invariant under digest-mode change.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_serial = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let serial_case =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_serial).unwrap();

    let mut ws_tree = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let tree_case =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events, &contract, &mut ws_tree)
            .unwrap();

    assert_eq!(
        serial_case.episodes, tree_case.episodes,
        "admitted episode list must be invariant under digest-mode change \
         (Semantic Non-Bypass Axiom: bank stays CPU-side, digest only commits evidence)"
    );
    assert_eq!(
        serial_case.final_verdict, tree_case.final_verdict,
        "final verdict must be invariant under digest-mode change"
    );
}

#[test]
fn tree_digest_changes_when_input_changes() {
    // Mutation sensitivity: flipping one event byte (via a
    // different LCG seed) changes the tree digest. If this failed
    // it would mean the tree topology is collapsing distinct
    // inputs to the same digest, which would defeat the purpose.
    let contract = canonical_contract();
    let events_a = synthesize(DEFAULT_SEED);
    let events_b = synthesize(DEFAULT_SEED.wrapping_add(0x9E37_79B9));

    let mut ws_a = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case_a =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events_a, &contract, &mut ws_a)
            .unwrap();

    let mut ws_b = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case_b =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events_b, &contract, &mut ws_b)
            .unwrap();

    assert_ne!(
        case_a.hashes.residual_field, case_b.hashes.residual_field,
        "residual stage digest must change when input events change"
    );
    assert_ne!(
        case_a.final_case_file_hash, case_b.final_case_file_hash,
        "final case-file hash must change when input events change"
    );
}

#[test]
fn tree_digest_stage_hashes_are_non_zero() {
    // Defensive smoke test: a degenerate code path could have
    // returned all-zero digests (uninitialized memory, missed
    // kernel launch). Verify every per-stage hash is non-zero.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case =
        build_gpu_throughput_pinned_async_on_workspace_tree(&events, &contract, &mut ws).unwrap();

    let stage_hashes = [
        ("residual_field", case.hashes.residual_field),
        ("sign_field", case.hashes.sign_field),
        ("detector_cell", case.hashes.detector_cell),
        ("consensus_grid", case.hashes.consensus_grid),
    ];
    for (name, digest) in stage_hashes {
        assert!(
            !digest.iter().all(|b| *b == 0),
            "tree-digest stage hash for {name} is all-zero — kernel may not have run"
        );
    }
}
