//! R.6c acceptance tests: opt-in CUDA Graph capture for the
//! Throughput-digests pipeline.
//!
//! The graph capture wrapper is **opt-in by design**. On hosts where
//! the driver / device refuses graph capture (older drivers, locked
//! down contexts, certain virtualised environments), the dispatch
//! wrapper returns `GraphCaptureStatus::Demoted { reason }` and falls
//! back to the pinned/async (R.6b) path. Every test in this file is
//! written to tolerate that demotion: the test either asserts a
//! property that holds across both branches, or branches on the
//! returned status and verifies the appropriate invariant.
//!
//! The graph is not semantic. It only records the launch plan
//! against the workspace's pinned shadows and stream. The CPU bank
//! still admits episodes; the GPU still emits evidence; per-stage
//! hash chain semantics are unchanged. These tests pin **byte
//! equivalence** between the captured-graph path and the R.6b
//! reference, and pin the **deterministic launch-plan hash** the
//! case file records when capture succeeds.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_graph_or_demote, build_gpu_throughput_pinned_async_on_workspace,
    GpuWorkspace, GraphCaptureStatus,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

fn scaled_contract(n_entities: u32, n_windows: u32) -> Contract {
    let mut c = Contract::scaled(n_entities, n_windows);
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn graph_or_demote_succeeds() {
    // Load-bearing minimum: the dispatch wrapper completes Ok on
    // any machine, whether or not graph capture is supported. The
    // returned case file is always usable; the GraphCaptureStatus
    // surfaces which branch produced it.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case, status) = build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws).unwrap();

    // The case file's final verdict is the canonical
    // `ReplayAdmissible`; both branches must produce a valid
    // case file because both run the same pipeline math.
    assert!(
        !case.final_case_file_hash.iter().all(|b| *b == 0),
        "case file hash must be non-zero on both Captured and Demoted branches"
    );
    match status {
        GraphCaptureStatus::Captured { plan_hash } => {
            assert!(
                !plan_hash.iter().all(|b| *b == 0),
                "captured plan hash must be non-zero"
            );
            assert!(
                ws.has_graph(),
                "Captured status implies workspace.has_graph()"
            );
            assert_eq!(ws.graph_plan_hash(), Some(plan_hash));
        }
        GraphCaptureStatus::Demoted { reason } => {
            assert!(
                !reason.is_empty(),
                "Demoted status must carry a non-empty reason for audit logs"
            );
            assert!(
                !ws.has_graph(),
                "Demoted status implies no graph captured on the workspace"
            );
        }
    }
}

#[test]
fn graph_path_case_hash_matches_pinned_async_if_captured() {
    // When capture succeeds, the graph replay must produce a case
    // file byte-identical to the R.6b pinned/async reference. The
    // graph is a launch-plan rearrangement; it does not change
    // kernel math.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    // Reference: R.6b pinned/async path on its own workspace.
    let mut ws_ref = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let ref_case =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_ref).unwrap();

    // Subject: graph-or-demote on a fresh workspace.
    let mut ws_graph = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (graph_case, status) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_graph).unwrap();

    match status {
        GraphCaptureStatus::Captured { .. } => {
            assert_eq!(
                graph_case.hashes, ref_case.hashes,
                "captured-graph chain hashes diverge from pinned/async reference"
            );
            assert_eq!(graph_case.episodes, ref_case.episodes);
            assert_eq!(
                graph_case.final_case_file_hash, ref_case.final_case_file_hash,
                "captured-graph final case-file hash diverges from pinned/async reference"
            );
            assert_eq!(graph_case.final_verdict, ref_case.final_verdict);
        }
        GraphCaptureStatus::Demoted { .. } => {
            // Tolerated: this machine cannot capture. The demoted
            // branch is covered by the next test; here we just
            // verify the demoted path also matches the reference
            // (since it IS the reference path).
            assert_eq!(graph_case.hashes, ref_case.hashes);
            assert_eq!(
                graph_case.final_case_file_hash, ref_case.final_case_file_hash,
                "demoted path must match the pinned/async reference (it is the reference)"
            );
        }
    }
}

#[test]
fn demoted_path_case_hash_matches_pinned_async_if_not_captured() {
    // Symmetric to the previous test, written for the failure
    // mode. On hosts where capture refuses, the dispatch wrapper
    // falls back to R.6b, which is the reference path. The case
    // file must therefore match the reference exactly. This test
    // would naturally skip on a machine that captures, by
    // returning early.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_ref = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let ref_case =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_ref).unwrap();

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case, status) = build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws).unwrap();

    if matches!(status, GraphCaptureStatus::Captured { .. }) {
        // Tolerated: nothing to assert about the demoted path on
        // this machine because it never demoted. The previous
        // test covers the captured path's byte equality.
        return;
    }

    assert_eq!(
        case.hashes, ref_case.hashes,
        "demoted-path chain hashes must match the pinned/async reference"
    );
    assert_eq!(case.episodes, ref_case.episodes);
    assert_eq!(
        case.final_case_file_hash, ref_case.final_case_file_hash,
        "demoted-path final case-file hash must match the pinned/async reference"
    );
}

#[test]
fn graph_plan_hash_stable_for_same_topology() {
    // Two captures on the same contract scale must produce the
    // same graph plan hash. The hash is computed from contract
    // metadata only (no pointer addresses, no graph handles, no
    // wall-clock), so it must be byte-deterministic.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_a = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let mut ws_b = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (_, status_a) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_a).unwrap();
    let (_, status_b) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_b).unwrap();

    // Tolerated: if either capture refused, there is nothing to
    // pin about plan-hash stability on this host.
    if let (
        GraphCaptureStatus::Captured { plan_hash: ha },
        GraphCaptureStatus::Captured { plan_hash: hb },
    ) = (status_a, status_b)
    {
        assert_eq!(
            ha, hb,
            "graph_plan_hash must be stable across two captures at the same scale"
        );
    }
}

#[test]
fn graph_plan_hash_changes_when_scale_changes() {
    // Different (n_entities, n_windows) ⇒ different captured
    // graph plan hash. The plan hash records the scale; it must
    // distinguish topologies that would route through different
    // launch geometries.
    let contract_small = canonical_contract();
    let contract_big = scaled_contract(32, 256);

    let events_small = synthesize(DEFAULT_SEED);
    let events_big = synthesize(DEFAULT_SEED);

    let mut ws_small = GpuWorkspace::new_with_pinned_async(&contract_small).unwrap();
    let mut ws_big = GpuWorkspace::new_with_pinned_async(&contract_big).unwrap();
    let (_, status_small) =
        build_gpu_throughput_graph_or_demote(&events_small, &contract_small, &mut ws_small)
            .unwrap();
    let (_, status_big) =
        build_gpu_throughput_graph_or_demote(&events_big, &contract_big, &mut ws_big).unwrap();

    // Tolerated: capture refused on at least one scale ⇒
    // nothing to pin about cross-scale distinction.
    if let (
        GraphCaptureStatus::Captured { plan_hash: small },
        GraphCaptureStatus::Captured { plan_hash: big },
    ) = (status_small, status_big)
    {
        assert_ne!(
            small, big,
            "graph_plan_hash must differ when (n_entities, n_windows) differ"
        );
    }
}
