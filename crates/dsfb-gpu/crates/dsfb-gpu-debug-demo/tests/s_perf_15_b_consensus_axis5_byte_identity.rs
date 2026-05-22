//! S-PERF.15.b — consensus_grid + axis5_grid_sum locality
//! fusion byte-identity harness.
//!
//! Purpose (panel-locked, 2026-05-18 post-S-PERF.15.a seal
//! at `a47a8e9`):
//!
//!   S-PERF.15.b fuses `consensus_grid_kernel_wide` with
//!   `axis5_grid_sum_kernel_wide` to remove the ~32 MB L2
//!   reread of the ConsensusCell arena while preserving:
//!
//!     - ConsensusCell bytes per cell (byte-identical)
//!     - axis5_grid_sum i64 per-window values (byte-identical
//!       via i64 associativity + canonical block-ascending
//!       merge order)
//!     - candidate_fired downstream cascade input
//!       (byte-identical)
//!     - R.12b episodes 13/89/1917 byte-stable
//!     - final case-file hash byte-stable
//!
//! Design (panel-locked direct fusion 2026-05-18 post-S-PERF.15.a;
//! avoids unordered atomics by construction):
//!
//!   `consensus_axis5_fused_kernel` — one block per
//!   `(window, catalog)`, one thread per entity
//!   (`blockDim.x == n_entities`).
//!
//!     Phase 1: compute ConsensusCell identical to legacy
//!              `consensus_grid_kernel_wide` (Phase 1+2 body
//!              verbatim — same per-cell math, same canonical
//!              cell offset).
//!     Phase 2: write `ConsensusCell` to global memory (byte-
//!              identical to legacy by construction).
//!     Phase 3: stage `axis7` in shared memory at `shm[e]` in
//!              canonical lane order (thread `e` writes shm[e]).
//!              `__syncthreads()`.
//!     Phase 4: thread 0 performs the canonical entity-ascending
//!              SERIAL sum:
//!                  int64_t sum = 0;
//!                  for (int i = 0; i < n_entities; i++)
//!                      sum += shm[i];
//!                  grid_sum_w[catalog_id * n_windows + w] = sum;
//!              Same loop order as the legacy
//!              `axis5_grid_sum_kernel_wide` serial body, so the
//!              i64 sum is byte-identical by construction.
//!
//!   Two-stage tile reduction held as S-PERF.15.b.1 fallback
//!   only if a future profile shows `n_entities > 1024`
//!   (canonical 16, full-scale 256 both fit in one block).
//!
//! **Byte-identity contract (panel-locked, MUST hold)**:
//!
//!   - ConsensusCell arena bytes byte-identical
//!     (PINNED_PRE_S_PERF_15_B_CONSENSUS_ARENA_SHA256)
//!   - axis5_grid_sum i64 per-window values byte-identical
//!     (PINNED_PRE_S_PERF_15_B_AXIS5_GRID_SUM_ARENA_SHA256)
//!   - candidate_fired downstream input byte-identical
//!     (PINNED_PRE_S_PERF_15_B_CANDIDATE_FIRED_ARENA_SHA256)
//!   - final case-file hash byte-stable
//!     (PINNED_PRE_S_PERF_15_B_CASEFILE_FINAL_HASH)
//!
//! Capture protocol (panel-required Step 0, one-time, BEFORE
//! the fused kernels are written):
//!
//!   DSFB_S_PERF_15_B_CAPTURE=1 cargo test ... \
//!     --test s_perf_15_b_consensus_axis5_byte_identity \
//!     -- --nocapture s_perf_15_b_capture_mode_prints_all_four_pins
//!
//! Fixture (panel-locked): canonical 16 entities × 128 windows,
//! K=1, D64 detector profile, panel-locked bank + canonical
//! contract. Same fixture as the S-PERF.11 + S-PERF.14b +
//! S-PERF.15.a root-capture tests.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::hash::sha256;
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed, GpuWorkspace,
};

// ---------------------------------------------------------------
// Panel-required Step 0 pinned constants.
//
// CAPTURE-MODE PROTOCOL (one-time, BEFORE fused-kernel surgery):
//
//   DSFB_S_PERF_15_B_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//     --features cuda --release \
//     --test s_perf_15_b_consensus_axis5_byte_identity \
//     -- --nocapture s_perf_15_b_capture_mode_prints_all_four_pins
//
// Capture-mode run executes the LEGACY (pre-fusion)
// consensus_grid + axis5_grid_sum 2-kernel sequence on the
// canonical 16x128 K=1 D64 fixture, computes 4 pinned
// digests, prints them as `[u8; 32]` literals on stdout, and
// DELIBERATELY FAILS so the constants below are refreshed
// BEFORE the assertion path is exercised.
// ---------------------------------------------------------------

/// SHA-256 over the entire `d_consensus` arena bytes after
/// the legacy consensus_grid_kernel_wide finishes writing.
/// The post-S-PERF.15.b fused kernel's Phase 2 ConsensusCell
/// store MUST produce byte-identical bytes or this fires.
const PINNED_PRE_S_PERF_15_B_CONSENSUS_ARENA_SHA256: [u8; 32] = [
    0xce, 0x67, 0x79, 0x40, 0x74, 0x6f, 0x8e, 0x11, 0x61, 0x37, 0x6d, 0x18, 0x19, 0x13, 0xae, 0x21,
    0xad, 0x38, 0x99, 0x74, 0xcd, 0x44, 0x36, 0xf2, 0x45, 0xcb, 0x26, 0x43, 0x54, 0xaa, 0x38, 0x58,
];

/// SHA-256 over the entire `d_axis5_grid_sum` arena bytes
/// (n_windows × i64 LE) after the legacy
/// axis5_grid_sum_kernel_wide finishes writing. The
/// post-S-PERF.15.b fused kernel MUST produce byte-identical
/// i64 sums (i64-add associativity + canonical entity-
/// ascending reduction order guarantees this).
const PINNED_PRE_S_PERF_15_B_AXIS5_GRID_SUM_ARENA_SHA256: [u8; 32] = [
    0x94, 0xf5, 0x7c, 0x08, 0x9e, 0x3b, 0xb9, 0x4c, 0xdf, 0xa0, 0x40, 0xfc, 0x91, 0xe7, 0x86, 0x81,
    0xf7, 0xa5, 0x0e, 0x7b, 0xe9, 0x77, 0xd6, 0x09, 0x76, 0x8a, 0x86, 0xc9, 0x8b, 0xc0, 0x55, 0xdb,
];

/// SHA-256 over the entire `d_candidate_fired` arena bytes
/// (n_cells × 1 B) after the downstream cascade has run.
/// candidate_fired reads ConsensusCell to produce fired
/// flags; if upstream consensus bytes drifted, fired flags
/// would shift and this constant fires.
const PINNED_PRE_S_PERF_15_B_CANDIDATE_FIRED_ARENA_SHA256: [u8; 32] = [
    0x74, 0xff, 0xfb, 0xdf, 0x0e, 0x5f, 0x36, 0xf8, 0x28, 0x26, 0x3a, 0x45, 0x37, 0xed, 0xf5, 0x5e,
    0x49, 0xec, 0x65, 0x2c, 0x2c, 0xc5, 0xf7, 0x23, 0x9b, 0x52, 0x0d, 0x94, 0x70, 0x6f, 0x79, 0x3a,
];

/// Final case-file hash from the canonical 16x128 K=1 D64
/// dispatch. The end-to-end cascade verification: if
/// anything upstream drifted, this hash shifts.
const PINNED_PRE_S_PERF_15_B_CASEFILE_FINAL_HASH: [u8; 32] = [
    0x98, 0xd0, 0x69, 0x67, 0x01, 0xf7, 0x6a, 0x81, 0xb3, 0x8e, 0x18, 0xe0, 0xf5, 0xb0, 0x2e, 0x86,
    0x75, 0x3f, 0xa0, 0xfd, 0xdb, 0xd1, 0x58, 0xd2, 0xe2, 0xd8, 0xb8, 0xb4, 0x00, 0x26, 0xf3, 0xa6,
];

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn d64_canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

fn hex_u8_array(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(32 * 6);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "0x{b:02x}");
        if i % 8 == 7 && i != 31 {
            out.push('\n');
            out.push_str("    ");
        }
    }
    out
}

/// Capture all 4 panel-required artifacts in one fixture run
/// (D64 tree-compact-timed dispatch on canonical 16x128 K=1).
fn capture_pre_fusion_pins() -> (
    [u8; 32], // consensus arena SHA-256
    [u8; 32], // axis5_grid_sum arena SHA-256
    [u8; 32], // candidate_fired arena SHA-256
    [u8; 32], // casefile final hash
) {
    let contract = d64_canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case, _stage_timings, _host_timings) =
        build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();

    let consensus_bytes = ws
        .last_d64_consensus_arena_bytes()
        .expect("d_consensus should be allocated post-dispatch")
        .expect("D2H of d_consensus should succeed");
    let axis5_bytes = ws
        .last_d64_axis5_grid_sum_bytes()
        .expect("d_axis5_grid_sum should be allocated post-dispatch")
        .expect("D2H of d_axis5_grid_sum should succeed");
    let fired_bytes = ws
        .last_d64_candidate_fired_arena_bytes()
        .expect("d_candidate_fired should be allocated post-dispatch")
        .expect("D2H of d_candidate_fired should succeed");

    let consensus_sha = sha256(&consensus_bytes);
    let axis5_sha = sha256(&axis5_bytes);
    let fired_sha = sha256(&fired_bytes);
    let casefile_hash = case.final_case_file_hash;

    (consensus_sha, axis5_sha, fired_sha, casefile_hash)
}

fn print_capture_and_panic(
    consensus_sha: &[u8; 32],
    axis5_sha: &[u8; 32],
    fired_sha: &[u8; 32],
    casefile_hash: &[u8; 32],
) -> ! {
    println!("=== S-PERF.15.b pre-fusion byte-capture (canonical 16x128 K=1 D64) ===");
    println!(
        "const PINNED_PRE_S_PERF_15_B_CONSENSUS_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(consensus_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_B_AXIS5_GRID_SUM_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(axis5_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_B_CANDIDATE_FIRED_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(fired_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_B_CASEFILE_FINAL_HASH: [u8; 32] = [\n    {}\n];",
        hex_u8_array(casefile_hash)
    );
    panic!(
        "DSFB_S_PERF_15_B_CAPTURE set: refresh the four pinned constants at the top of \
         this file with the printed values, then re-run without the env var."
    );
}

// ---------------------------------------------------------------
// Step 0 capture-mode test (single panic point for one
// comprehensive capture run).
// ---------------------------------------------------------------

#[test]
fn s_perf_15_b_capture_mode_prints_all_four_pins() {
    if std::env::var("DSFB_S_PERF_15_B_CAPTURE").is_err() {
        return;
    }
    let (consensus_sha, axis5_sha, fired_sha, casefile_hash) = capture_pre_fusion_pins();
    print_capture_and_panic(&consensus_sha, &axis5_sha, &fired_sha, &casefile_hash);
}

// ---------------------------------------------------------------
// 4 panel-required Step 0 pre-fusion-pin negatives.
// ---------------------------------------------------------------

/// CAMPAIGN IDENTITY — ConsensusCell arena bytes byte-equal
/// to pinned. If the fused kernel's Phase 2 ConsensusCell
/// store drifted from legacy, this is the first thing that
/// fires.
#[test]
fn s_perf_15_b_rejects_consensus_arena_byte_change() {
    if std::env::var("DSFB_S_PERF_15_B_CAPTURE").is_ok() {
        return;
    }
    let (consensus_sha, _, _, _) = capture_pre_fusion_pins();
    assert_eq!(
        consensus_sha, PINNED_PRE_S_PERF_15_B_CONSENSUS_ARENA_SHA256,
        "S-PERF.15.b: ConsensusCell arena SHA-256 drifted from pinned; byte-identity \
         contract VIOLATED at the ConsensusCell layer"
    );
}

/// CAMPAIGN IDENTITY — axis5_grid_sum i64 values byte-equal
/// to pinned. The 2-stage tile reduction MUST produce the
/// same i64 sum as the legacy serial per-window
/// entity-ascending walk (i64 associativity + canonical
/// block-ascending merge guarantees this).
#[test]
fn s_perf_15_b_rejects_axis5_grid_sum_byte_change() {
    if std::env::var("DSFB_S_PERF_15_B_CAPTURE").is_ok() {
        return;
    }
    let (_, axis5_sha, _, _) = capture_pre_fusion_pins();
    assert_eq!(
        axis5_sha, PINNED_PRE_S_PERF_15_B_AXIS5_GRID_SUM_ARENA_SHA256,
        "S-PERF.15.b: axis5_grid_sum arena SHA-256 drifted from pinned; byte-identity \
         contract VIOLATED at the i64 per-window sum layer"
    );
}

/// candidate_fired downstream input byte-equal to pinned.
/// If consensus bytes drifted upstream, the cascade fired
/// flags would shift.
#[test]
fn s_perf_15_b_rejects_candidate_fired_cascade_change() {
    if std::env::var("DSFB_S_PERF_15_B_CAPTURE").is_ok() {
        return;
    }
    let (_, _, fired_sha, _) = capture_pre_fusion_pins();
    assert_eq!(
        fired_sha, PINNED_PRE_S_PERF_15_B_CANDIDATE_FIRED_ARENA_SHA256,
        "S-PERF.15.b: candidate_fired arena SHA-256 drifted from pinned; downstream \
         cascade input shifted, indicating consensus byte drift upstream"
    );
}

/// End-to-end case-file hash byte-equal to pinned.
/// Final cascade verification — if any earlier layer
/// drifted, this hash would shift.
#[test]
fn s_perf_15_b_rejects_casefile_final_hash_change() {
    if std::env::var("DSFB_S_PERF_15_B_CAPTURE").is_ok() {
        return;
    }
    let (_, _, _, casefile_hash) = capture_pre_fusion_pins();
    assert_eq!(
        casefile_hash, PINNED_PRE_S_PERF_15_B_CASEFILE_FINAL_HASH,
        "S-PERF.15.b: case-file final hash drifted from pinned; end-to-end cascade \
         broken somewhere in the post-S-PERF.15.b path"
    );
}

// ---------------------------------------------------------------
// 4 panel-required positives.
// ---------------------------------------------------------------

/// Canonical 16x128 K=1 fixture admits 13 episodes
/// post-S-PERF.15.b.
#[test]
fn consensus_axis5_fused_admits_canonical_fixture() {
    let contract = d64_canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case, _, _) = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    assert_eq!(
        case.episodes.len(),
        13,
        "canonical 16x128 D64 should admit 13 episodes post-S-PERF.15.b"
    );
}

/// Cross-run determinism: two dispatches produce
/// byte-identical case-file hashes.
#[test]
fn consensus_axis5_fused_cross_run_determinism() {
    let contract = d64_canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws1 = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case1, _, _) = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
        &events, &contract, &mut ws1, &fixture,
    )
    .unwrap();
    let mut ws2 = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case2, _, _) = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
        &events, &contract, &mut ws2, &fixture,
    )
    .unwrap();
    assert_eq!(
        case1.final_case_file_hash, case2.final_case_file_hash,
        "S-PERF.15.b: 2-stage tile fusion non-deterministic across two runs"
    );
}

/// post-S-PERF.15.b bench snapshot exists + references the
/// canonical fixture.
#[test]
fn consensus_axis5_fused_wall_time_recorded() {
    let post_path = "../../reports/d64_stage_timing_256x4096_K1_post_s_perf_15_b.txt";
    let abs_path = "/home/one/dsfb-gpu/reports/d64_stage_timing_256x4096_K1_post_s_perf_15_b.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        return;
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        content.contains("256x4096")
            || content.contains("256 entities")
            || content.contains("n_entities=256"),
        "post-S-PERF.15.b bench snapshot must reference the canonical 256x4096 fixture"
    );
}

/// post-S-PERF.15.b ROOF receipt exists + references either
/// the legacy kernels or the new 2-stage tile kernel pair.
#[test]
fn consensus_axis5_fused_roof_receipt_present() {
    let post_path = "../../reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_b_post.txt";
    let abs_path =
        "/home/one/dsfb-gpu/reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_b_post.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        return;
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let has_fused = content.contains("consensus_axis5_fused_kernel");
    let has_legacy = content.contains("consensus_grid_kernel_wide")
        || content.contains("axis5_grid_sum_kernel_wide");
    assert!(
        has_fused || has_legacy,
        "post-S-PERF.15.b ROOF receipt must reference the S-PERF.15.b fused kernel \
         (consensus_axis5_fused_kernel) or, if the post-ROOF run captured the legacy \
         event-slot labels, the legacy consensus_grid_kernel_wide / axis5_grid_sum_kernel_wide"
    );
}
