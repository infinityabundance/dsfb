//! R.6b acceptance test: the pinned/async Throughput-digests
//! dispatch produces a case file byte-identical to the pageable/sync
//! reference on the same fixture and contract.
//!
//! R.6b is a mechanical sync→async conversion (cudaMemcpy →
//! cudaMemcpyAsync, default-stream → explicit single stream, final
//! cudaStreamSynchronize). Single CUDA streams execute their work in
//! program order, so the output bytes must match the sync wrapper's
//! exactly. Any divergence here means R.6b accidentally changed
//! kernel execution ordering or memcpy semantics — by the
//! publishability gate, the async path should then be demoted.
//!
//! These tests do not exercise async-overlap performance; that is
//! R.6c (CUDA Graphs) and beyond. R.6b only pins correctness.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_device_digests_on_workspace,
    build_gpu_throughput_pinned_async_on_workspace, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn pinned_async_case_file_matches_pageable_sync_byte_for_byte() {
    // Load-bearing R.6b invariant: every chain hash, the final
    // case-file hash, and the admitted-episode list must match
    // between the async path and the sync reference.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_sync = GpuWorkspace::new(&contract).unwrap();
    let sync_case =
        build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws_sync).unwrap();

    let mut ws_async = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    assert!(
        ws_async.has_pinned_async(),
        "new_with_pinned_async must populate the pinned shadows + stream"
    );
    let async_case =
        build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws_async).unwrap();

    assert_eq!(
        async_case.hashes, sync_case.hashes,
        "pinned/async chain hashes diverge from pageable/sync reference"
    );
    assert_eq!(async_case.episodes, sync_case.episodes);
    assert_eq!(
        async_case.final_case_file_hash, sync_case.final_case_file_hash,
        "pinned/async final case-file hash diverges from pageable/sync reference"
    );
    assert_eq!(async_case.final_verdict, sync_case.final_verdict);
}

#[test]
fn pinned_async_is_deterministic_across_runs() {
    // Two consecutive async dispatches on the same workspace must
    // produce byte-identical case files. Catches any nondeterminism
    // introduced by stream scheduling order or pinned-buffer reuse.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let a = build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws).unwrap();
    let b = build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn pinned_async_rejects_unpinned_workspace() {
    // The async dispatch must refuse a workspace that wasn't built
    // with pinned shadows + stream. Returning InvalidInput is the
    // documented contract; silently falling back to the sync path
    // would hide a mis-configuration.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new(&contract).unwrap();
    assert!(!ws.has_pinned_async());
    let result = build_gpu_throughput_pinned_async_on_workspace(&events, &contract, &mut ws);
    assert!(
        result.is_err(),
        "async dispatch must reject a workspace built with the legacy `new` constructor"
    );
}
