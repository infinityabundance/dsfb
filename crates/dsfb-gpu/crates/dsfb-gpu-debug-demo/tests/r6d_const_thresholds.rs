//! R.6d acceptance tests: opt-in constant-memory detector thresholds.
//!
//! The R.6d optimization hoists the canonical `DetectorThresholds`
//! struct from a per-launch kernel argument into the device's
//! `__constant__ c_detector_thresholds` symbol. The dispatch wrappers
//! prefer the `detector_motif_kernel_const` variant when the upload
//! at workspace construction succeeded, and gracefully fall back to
//! the param-passing `detector_motif_kernel` when it failed.
//!
//! These tests pin **byte equivalence** between the two paths, the
//! **fallback** path's correctness (via the `force_…_for_test`
//! method), and the **graph_plan_hash** reflecting the const-or-not
//! topology choice. Audit mode and golden hashes are not touched
//! by R.6d; their tests live elsewhere and continue to pass
//! unchanged.

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

#[test]
fn const_path_case_hash_matches_param_path() {
    // Byte-equivalence: the constant-memory detector kernel variant
    // produces a case file identical to the param-passing variant.
    // R.6d is correctness-first; this is the load-bearing invariant.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    // Const-memory path (default on a working CUDA context).
    let mut ws_const = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    assert!(
        ws_const.has_const_thresholds(),
        "fresh new_with_pinned_async workspace should have const-thresholds uploaded on a working CUDA context"
    );
    let case_const =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_const).unwrap();

    // Forced fallback path (param-passing).
    let mut ws_param = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    ws_param.force_const_thresholds_uploaded_for_test(false);
    assert!(!ws_param.has_const_thresholds());
    let case_param =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_param).unwrap();

    assert_eq!(
        case_const.hashes, case_param.hashes,
        "const-memory and param-passing detector kernel variants must produce byte-identical chain hashes"
    );
    assert_eq!(case_const.episodes, case_param.episodes);
    assert_eq!(
        case_const.final_case_file_hash, case_param.final_case_file_hash,
        "const-memory and param-passing final case-file hash must match"
    );
    assert_eq!(case_const.final_verdict, case_param.final_verdict);
}

#[test]
fn const_path_intermediate_hashes_unchanged() {
    // Per-stage hash invariance: each named link in the chain
    // must be byte-identical between the const and param paths.
    // Mostly redundant with the previous test's struct equality,
    // but localises a regression to the specific stage if one
    // ever appears.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_const = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let case_const =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_const).unwrap();

    let mut ws_param = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    ws_param.force_const_thresholds_uploaded_for_test(false);
    let case_param =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_param).unwrap();

    let lhs = &case_const.hashes;
    let rhs = &case_param.hashes;
    assert_eq!(
        lhs.input_catalog, rhs.input_catalog,
        "input_catalog mismatch"
    );
    assert_eq!(lhs.contract, rhs.contract, "contract mismatch");
    assert_eq!(lhs.bank, rhs.bank, "bank mismatch");
    assert_eq!(
        lhs.detector_registry, rhs.detector_registry,
        "detector_registry mismatch"
    );
    assert_eq!(
        lhs.kernel_sequence, rhs.kernel_sequence,
        "kernel_sequence mismatch"
    );
    assert_eq!(
        lhs.window_feature, rhs.window_feature,
        "window_feature mismatch"
    );
    assert_eq!(
        lhs.residual_field, rhs.residual_field,
        "residual_field mismatch"
    );
    assert_eq!(lhs.sign_field, rhs.sign_field, "sign_field mismatch");
    assert_eq!(
        lhs.detector_cell, rhs.detector_cell,
        "detector_cell mismatch (R.6d const vs param)"
    );
    assert_eq!(
        lhs.consensus_grid, rhs.consensus_grid,
        "consensus_grid mismatch"
    );
    assert_eq!(
        lhs.candidate_interval, rhs.candidate_interval,
        "candidate_interval mismatch"
    );
    assert_eq!(lhs.episode, rhs.episode, "episode mismatch");
}

#[test]
fn graph_capture_succeeds_under_const_and_param_paths() {
    // The R.6c graph capture path must work under both R.6d
    // branches. Const-path captures by default; param-path
    // captures via the test forcer. Either way, capture either
    // succeeds with a non-zero plan_hash or honestly demotes —
    // both outcomes are acceptable.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_const = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case_const, status_const) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_const).unwrap();
    assert!(
        !case_const.final_case_file_hash.iter().all(|b| *b == 0),
        "const-path case file hash must be non-zero"
    );
    match status_const {
        GraphCaptureStatus::Captured { plan_hash } => {
            assert!(plan_hash.iter().any(|b| *b != 0));
        }
        GraphCaptureStatus::Demoted { reason } => {
            assert!(!reason.is_empty(), "demoted status carries a reason");
        }
    }

    let mut ws_param = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    ws_param.force_const_thresholds_uploaded_for_test(false);
    let (case_param, status_param) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_param).unwrap();
    assert!(
        !case_param.final_case_file_hash.iter().all(|b| *b == 0),
        "param-path case file hash must be non-zero"
    );
    match status_param {
        GraphCaptureStatus::Captured { plan_hash } => {
            assert!(plan_hash.iter().any(|b| *b != 0));
        }
        GraphCaptureStatus::Demoted { reason } => {
            assert!(!reason.is_empty(), "demoted status carries a reason");
        }
    }

    // Bytes must match across const and param paths whichever
    // graph branch each one took.
    assert_eq!(
        case_const.final_case_file_hash, case_param.final_case_file_hash,
        "const vs param graph-or-demote case file hash must match"
    );
}

#[test]
fn graph_plan_hash_differs_when_const_thresholds_differs() {
    // R.6c plan_hash includes `uses_const_thresholds=...` in its
    // canonical metadata. Two workspaces with different upload
    // outcomes therefore produce different plan hashes — the
    // const path is a topology variant and the case file's
    // launch-plan provenance records that variant.
    //
    // Tolerated: if either workspace's capture demotes (e.g. on a
    // graph-incapable host), there is nothing to compare. Skip
    // in that case.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_const = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (_, status_const) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_const).unwrap();

    let mut ws_param = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    ws_param.force_const_thresholds_uploaded_for_test(false);
    let (_, status_param) =
        build_gpu_throughput_graph_or_demote(&events, &contract, &mut ws_param).unwrap();

    if let (
        GraphCaptureStatus::Captured { plan_hash: hc },
        GraphCaptureStatus::Captured { plan_hash: hp },
    ) = (status_const, status_param)
    {
        assert_ne!(
            hc, hp,
            "graph_plan_hash must differ when uses_const_thresholds differs"
        );
    }
}

#[test]
fn const_path_is_deterministic_across_runs() {
    // Two consecutive dispatches on the const path produce
    // byte-identical case files. Catches any non-determinism the
    // const-memory upload or kernel variant might introduce.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws).unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}
