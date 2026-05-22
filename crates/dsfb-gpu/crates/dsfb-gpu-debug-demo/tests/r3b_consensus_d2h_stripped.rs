//! R.3b acceptance test: the consensus D2H is actually stripped from
//! the Tier 3B / Layer A / Layer B paths.
//!
//! This is the load-bearing observable for R.3b. After R.5 moved the
//! bank's axis-5 evidence into `CandidateInterval`, the consensus
//! grid no longer needs to come back to host. R.3b's job was to
//! actually remove that D2H. This test proves it: we deliberately
//! poison the workspace's host-side `consensus` buffer with a sentinel
//! pattern, then run the Tier 3B dispatch, then read the buffer back.
//! If the dispatch's `cudaMemcpy` overwrites the buffer, the
//! sentinels are gone (test fails). If R.3b actually skipped the
//! copy, the sentinels survive untouched.
//!
//! Audit-mode and non-digest Throughput paths still copy the
//! consensus grid back — they have to, for the canonical-JSON case
//! file. Those paths are not exercised here. R.3b only strips the
//! D2H from the digest-aware Throughput paths (Tier 3B + Layer A).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::consensus::ConsensusCell;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_core::Q16;
use dsfb_gpu_debug_cuda::{
    build_gpu_layer_a_on_workspace, build_gpu_throughput_device_digests_on_workspace, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

/// Poisoned sentinel value: a recognisable Q16.16 bit pattern that
/// is wildly out of range for any real consensus cell. If the
/// workspace's consensus buffer is overwritten by a D2H, this byte
/// pattern disappears.
fn sentinel_cell() -> ConsensusCell {
    ConsensusCell {
        window_idx: 0xDEAD_BEEF,
        entity_id: 0xCAFE_BABE,
        detector_count: 0xFEED_FACE,
        axis1_residual_q: Q16::from_raw(i32::MIN + 1),
        axis2_drift_q: Q16::from_raw(i32::MIN + 2),
        axis3_slew_q: Q16::from_raw(i32::MIN + 3),
        axis4_temporal_q: Q16::from_raw(i32::MIN + 4),
        axis7_consensus_q: Q16::from_raw(i32::MIN + 5),
    }
}

#[test]
fn tier3b_does_not_overwrite_workspace_consensus_buffer() {
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut workspace = GpuWorkspace::new(&contract).unwrap();

    // Poison every host-side consensus cell with the sentinel pattern.
    // Pre-R.3b this poisoning would have been silently wiped by the
    // FFI's cudaMemcpy on the next dispatch; post-R.3b the dispatch
    // passes null for h_consensus and the buffer stays intact.
    let sentinel = sentinel_cell();
    for cell in workspace.consensus_mut().iter_mut() {
        *cell = sentinel;
    }

    // Run the Tier 3B (Layer B) dispatch — this is the path that
    // R.3b changed. We do not consume the returned case file; we
    // only care whether the workspace's consensus buffer was touched.
    let _case =
        build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut workspace)
            .expect("Tier 3B dispatch must succeed");

    // The buffer must be byte-identical to the sentinel pattern.
    // Any divergence proves the consensus D2H is still happening
    // and R.3b regressed.
    for (i, cell) in workspace.consensus().iter().enumerate() {
        assert_eq!(
            *cell, sentinel,
            "consensus cell {i} was overwritten — Tier 3B D2H is NOT stripped"
        );
    }
}

#[test]
fn layer_a_does_not_overwrite_workspace_consensus_buffer() {
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut workspace = GpuWorkspace::new(&contract).unwrap();

    let sentinel = sentinel_cell();
    for cell in workspace.consensus_mut().iter_mut() {
        *cell = sentinel;
    }

    let _summary = build_gpu_layer_a_on_workspace(&events, &contract, &mut workspace)
        .expect("Layer A dispatch must succeed");

    for (i, cell) in workspace.consensus().iter().enumerate() {
        assert_eq!(
            *cell, sentinel,
            "consensus cell {i} was overwritten — Layer A D2H is NOT stripped"
        );
    }
}
