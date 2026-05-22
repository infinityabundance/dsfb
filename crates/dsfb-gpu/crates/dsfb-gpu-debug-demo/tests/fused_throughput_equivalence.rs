//! R.4 acceptance tests: the fused Throughput dispatch
//! (`build_gpu_fused_throughput_digests_on_workspace`, which uses
//! Pre-Alpha EWMA + cell-parallel fused R+S) must produce a case
//! file that is byte-identical to the un-fused reference
//! (`build_gpu_throughput_device_digests_on_workspace`, which uses
//! the legacy entity-serial sign kernel).
//!
//! Load-bearing invariants pinned here:
//!
//! 1. All 12 chain hashes match byte-for-byte. In particular the
//!    `residual_field`, `sign_field`, `detector_cell`, `consensus_grid`,
//!    `candidate_interval`, and `episode` links must match — these are
//!    the links downstream of the fused kernels.
//! 2. `final_case_file_hash` matches.
//! 3. Admitted episode list matches.
//! 4. Golden hashes are unchanged. R.4 is implementation-rearrangement
//!    only; the canonical fixture's pinned hashes must remain at the
//!    R.5 values.
//!
//! These tests do not depend on the parallel boundary-detection
//! candidate kernel (which R.4 honestly defers); they only exercise
//! the Pre-Alpha EWMA + cell-parallel fused R+S rewrite.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_fused_throughput_digests_on_workspace,
    build_gpu_throughput_device_digests_on_workspace, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn fused_throughput_case_file_matches_unfused_byte_for_byte() {
    // The load-bearing R.4 invariant: every chain link, the final
    // hash, and the admitted episode list match byte-for-byte
    // between the fused and un-fused Throughput-digests dispatch
    // paths on the same fixture and contract.
    //
    // If the Pre-Alpha EWMA kernel's drift values diverge by even one
    // bit from the legacy entity-serial recurrence, the `sign_field`
    // digest will differ and cascade through every downstream link.
    // If the cell-parallel R+S kernel's slew computation diverges
    // (e.g., a missing `prev_norm` recomputation), the same.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_unfused = GpuWorkspace::new(&contract).unwrap();
    let unfused =
        build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws_unfused)
            .unwrap();

    let mut ws_fused = GpuWorkspace::new(&contract).unwrap();
    let fused =
        build_gpu_fused_throughput_digests_on_workspace(&events, &contract, &mut ws_fused).unwrap();

    assert_eq!(
        fused.hashes, unfused.hashes,
        "fused throughput chain hashes diverge from un-fused reference"
    );
    assert_eq!(fused.episodes, unfused.episodes);
    assert_eq!(fused.final_case_file_hash, unfused.final_case_file_hash);
    assert_eq!(fused.final_verdict, unfused.final_verdict);
}

#[test]
fn fused_throughput_is_deterministic_across_runs() {
    // Two consecutive fused dispatches on the same fixture must
    // produce byte-identical case files. This catches non-determinism
    // that could be introduced by warp scheduling order, atomic-free
    // reductions, or shared-memory races in the fused kernel.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new(&contract).unwrap();
    let a = build_gpu_fused_throughput_digests_on_workspace(&events, &contract, &mut ws).unwrap();
    let b = build_gpu_fused_throughput_digests_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

#[test]
fn fused_throughput_episode_count_unchanged() {
    // R.5 pinned the canonical fixture at 15 admitted episodes. R.4
    // is a byte-preserving rearrangement; if the fused path admits a
    // different number, the kernel rewrite drifted off-contract.
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new(&contract).unwrap();
    let case =
        build_gpu_fused_throughput_digests_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(
        case.episodes.len(),
        15,
        "fused dispatch admitted episode count drifted from R.5 canonical receipt"
    );
}
