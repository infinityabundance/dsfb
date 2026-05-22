//! S-PERF.15.c — candidate_pack launch-geometry repair
//! byte-identity harness.
//!
//! Purpose (panel-locked 2026-05-18 post-S-PERF.15.b ROOF;
//! retargeted from the original residual_field design):
//!
//!   The post-S-PERF.15.b ROOF receipt
//!   (`reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_b_post.txt`)
//!   surfaced `candidate_pack_kernel_wide` at **873 µs @ 5.7 %
//!   occupancy** as the largest unaddressed fixable wall. The
//!   panel's rule "do not obey the old plan when ROOF reveals a
//!   sharper wall" overrides the prior residual_field design;
//!   S-PERF.15.c attacks candidate_pack with launch-geometry
//!   repair, mirroring the successful S-PERF.14a / 14c
//!   Pre-Alpha + wide-emit pattern.
//!
//! Repair design (panel-locked direct rewrite; NOT a fusion):
//!
//!   `candidate_pack_kernel_wide_blockcoop` — one block per
//!   `(slot, entity, catalog)` (block count: 256 -> 4 096 at
//!   canonical, 16× block-count increase), 32 threads per block
//!   (one warp). Block-level early-return if
//!   `slot >= local_count`.
//!
//!     Phase 1: per-thread partials. Thread `tid` walks windows
//!              {start_w + tid, start_w + tid + 32, ...}
//!              accumulating 5 peaks (max-if-greater),
//!              union_mask (OR), entity_sum + grid_sum (i64).
//!     Phase 2: block-cooperative pairwise reduction in shared
//!              memory. All reductions are
//!              associative + commutative (max + OR + i64-add)
//!              so the tree reduction produces byte-identical
//!              output regardless of intra-tree order.
//!     Phase 3: thread 0 derives entity_avg + grid_avg via
//!              integer division of the same numerators +
//!              denominators as the legacy serial loop, then
//!              writes the final `CandidateInterval` to the
//!              canonical-indexed output slot.
//!
//! **Byte-identity contract (panel-locked, MUST hold)**:
//!
//!   - `d_candidates` arena bytes byte-identical
//!     (PINNED_PRE_S_PERF_15_C_CANDIDATE_PACK_BYTES)
//!   - `d_candidate_count` arena bytes byte-identical
//!     (PINNED_PRE_S_PERF_15_C_CANDIDATE_COUNT_BYTES) —
//!     defense-in-depth that the upstream cascade
//!     (candidate_boundary + candidate_fired) is unchanged
//!   - final case-file hash byte-stable
//!     (PINNED_PRE_S_PERF_15_C_CASEFILE_FINAL)
//!   - admitted episode list byte-stable
//!     (PINNED_PRE_S_PERF_15_C_EPISODE_SUMMARY)
//!
//! Capture protocol (panel-required Step 0, one-time, BEFORE
//! the blockcoop kernel is written):
//!
//!   DSFB_S_PERF_15_C_CAPTURE=1 cargo test \
//!     -p dsfb-gpu-debug-demo --features cuda --release \
//!     --test s_perf_15_c_candidate_pack_byte_identity \
//!     -- --nocapture s_perf_15_c_capture_mode_prints_all_four_pins
//!
//! Fixture (panel-locked): canonical 16 entities × 128 windows,
//! K=1, D64 detector profile, panel-locked bank + canonical
//! contract. Same fixture as the S-PERF.11 + S-PERF.14b +
//! S-PERF.15.a + S-PERF.15.b root-capture tests.

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
// CAPTURE-MODE PROTOCOL (one-time, BEFORE kernel surgery):
//
//   DSFB_S_PERF_15_C_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//     --features cuda --release \
//     --test s_perf_15_c_candidate_pack_byte_identity \
//     -- --nocapture s_perf_15_c_capture_mode_prints_all_four_pins
//
// Capture-mode run executes the LEGACY (pre-rewrite)
// candidate_pack_kernel_wide on the canonical 16x128 K=1 D64
// fixture, computes 4 pinned digests, prints them as `[u8; 32]`
// literals on stdout, and DELIBERATELY FAILS so the constants
// below are refreshed BEFORE the assertion path is exercised.
// ---------------------------------------------------------------

/// SHA-256 over the entire `d_candidates` arena bytes
/// (`n_entities × MAX_CANDIDATES_PER_ENTITY × sizeof::<CandidateInterval>`)
/// after the legacy `candidate_pack_kernel_wide` finishes
/// writing. The post-S-PERF.15.c blockcoop kernel MUST produce
/// byte-identical bytes here or this fires.
const PINNED_PRE_S_PERF_15_C_CANDIDATE_PACK_BYTES: [u8; 32] = [
    0xbb, 0x2f, 0x37, 0x80, 0x61, 0x75, 0x12, 0x40, 0x64, 0x04, 0xfb, 0xda, 0xdb, 0x48, 0x30, 0xb3,
    0x75, 0x2c, 0xcb, 0xa2, 0x63, 0x85, 0xaf, 0x31, 0x34, 0x00, 0xf4, 0xcb, 0x57, 0x9b, 0x9d, 0x54,
];

/// SHA-256 over the entire `d_candidate_count` arena bytes
/// (`n_entities × sizeof::<i32>`). The pack kernel READS this
/// but does NOT write it; pinning is defense-in-depth that the
/// upstream cascade (candidate_boundary + candidate_fired)
/// remains byte-stable across the S-PERF.15.c swap.
const PINNED_PRE_S_PERF_15_C_CANDIDATE_COUNT_BYTES: [u8; 32] = [
    0x3a, 0xf2, 0x67, 0xbd, 0x2e, 0xa9, 0x1a, 0xfb, 0x03, 0xd1, 0x92, 0x6c, 0x9d, 0x6b, 0x9b, 0x00,
    0xcc, 0xe7, 0x24, 0x85, 0x08, 0xbf, 0xc2, 0x3e, 0xb7, 0xa5, 0x0e, 0x63, 0x25, 0xb1, 0xac, 0x83,
];

/// Final case-file hash from the canonical 16x128 K=1 D64
/// dispatch. End-to-end cascade verification: if any layer
/// upstream of the case file drifted, this hash shifts.
const PINNED_PRE_S_PERF_15_C_CASEFILE_FINAL: [u8; 32] = [
    0x98, 0xd0, 0x69, 0x67, 0x01, 0xf7, 0x6a, 0x81, 0xb3, 0x8e, 0x18, 0xe0, 0xf5, 0xb0, 0x2e, 0x86,
    0x75, 0x3f, 0xa0, 0xfd, 0xdb, 0xd1, 0x58, 0xd2, 0xe2, 0xd8, 0xb8, 0xb4, 0x00, 0x26, 0xf3, 0xa6,
];

/// SHA-256 over the canonical bytes of the admitted episode
/// list: a u32 LE count prefix, then per-episode canonical
/// bytes (entity_id LE + start_window LE + end_window LE +
/// motif as u32 LE), the list pre-sorted by
/// (entity_id, start_window). Defense-in-depth pin that the
/// bank's admission decisions are unchanged across the
/// S-PERF.15.c swap.
const PINNED_PRE_S_PERF_15_C_EPISODE_SUMMARY: [u8; 32] = [
    0xd5, 0x21, 0x28, 0x3b, 0xc8, 0xd8, 0xd0, 0xc4, 0x3e, 0x7a, 0xdc, 0x73, 0x1e, 0x63, 0x79, 0xc3,
    0x0d, 0x7e, 0xc0, 0x45, 0x2f, 0x55, 0x66, 0x37, 0xbd, 0x99, 0x56, 0x16, 0xdc, 0x40, 0x7e, 0xeb,
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

/// Compute the canonical episode-summary digest from a case
/// file. Format: `u32 LE count || per-episode bytes` where
/// each episode contributes `entity_id LE || start_window LE
/// || end_window LE || (motif as u32) LE`. Episodes are
/// already in canonical (entity_id, start_window) order in the
/// case file by construction (the bank emits them sorted).
fn episode_summary_digest(case: &dsfb_gpu_debug_core::casefile::CaseFile) -> [u8; 32] {
    let count: u32 = case.episodes.len() as u32;
    let mut bytes: Vec<u8> = Vec::with_capacity(4 + case.episodes.len() * 16);
    bytes.extend_from_slice(&count.to_le_bytes());
    for ep in &case.episodes {
        bytes.extend_from_slice(&ep.entity_id.to_le_bytes());
        bytes.extend_from_slice(&ep.start_window.to_le_bytes());
        bytes.extend_from_slice(&ep.end_window.to_le_bytes());
        // `motif` is a `BankMotif` (panel-locked u8 enum); cast
        // to u32 LE for fixed 4-byte canonical encoding so the
        // digest layout is independent of any future enum
        // discriminant-size change.
        let motif: u32 = ep.motif as u32;
        bytes.extend_from_slice(&motif.to_le_bytes());
    }
    sha256(&bytes)
}

/// Capture all 4 panel-required artifacts in one fixture run
/// (D64 tree-compact-timed dispatch on canonical 16x128 K=1).
fn capture_pre_rewrite_pins() -> (
    [u8; 32], // d_candidates arena SHA-256
    [u8; 32], // d_candidate_count arena SHA-256
    [u8; 32], // casefile final hash
    [u8; 32], // episode summary SHA-256
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

    let candidates_bytes = ws
        .last_d64_candidates_arena_bytes()
        .expect("d_candidates should be allocated post-dispatch")
        .expect("D2H of d_candidates should succeed");
    let count_bytes = ws
        .last_d64_candidate_count_arena_bytes()
        .expect("d_candidate_count should be allocated post-dispatch")
        .expect("D2H of d_candidate_count should succeed");

    let candidates_sha = sha256(&candidates_bytes);
    let count_sha = sha256(&count_bytes);
    let casefile_hash = case.final_case_file_hash;
    let episode_sha = episode_summary_digest(&case);

    (candidates_sha, count_sha, casefile_hash, episode_sha)
}

fn print_capture_and_panic(
    candidates_sha: &[u8; 32],
    count_sha: &[u8; 32],
    casefile_hash: &[u8; 32],
    episode_sha: &[u8; 32],
) -> ! {
    println!("=== S-PERF.15.c pre-rewrite byte-capture (canonical 16x128 K=1 D64) ===");
    println!(
        "const PINNED_PRE_S_PERF_15_C_CANDIDATE_PACK_BYTES: [u8; 32] = [\n    {}\n];",
        hex_u8_array(candidates_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_C_CANDIDATE_COUNT_BYTES: [u8; 32] = [\n    {}\n];",
        hex_u8_array(count_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_C_CASEFILE_FINAL: [u8; 32] = [\n    {}\n];",
        hex_u8_array(casefile_hash)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_C_EPISODE_SUMMARY: [u8; 32] = [\n    {}\n];",
        hex_u8_array(episode_sha)
    );
    panic!(
        "DSFB_S_PERF_15_C_CAPTURE set: refresh the four pinned constants at the top of \
         this file with the printed values, then re-run without the env var."
    );
}

// ---------------------------------------------------------------
// Step 0 capture-mode test (single panic point for one
// comprehensive capture run).
// ---------------------------------------------------------------

#[test]
fn s_perf_15_c_capture_mode_prints_all_four_pins() {
    if std::env::var("DSFB_S_PERF_15_C_CAPTURE").is_err() {
        return;
    }
    let (candidates_sha, count_sha, casefile_hash, episode_sha) = capture_pre_rewrite_pins();
    print_capture_and_panic(&candidates_sha, &count_sha, &casefile_hash, &episode_sha);
}

// ---------------------------------------------------------------
// 4 panel-required Step 0 pre-rewrite-pin negatives.
// ---------------------------------------------------------------

/// CAMPAIGN IDENTITY — `d_candidates` arena bytes byte-equal
/// to pinned. If the blockcoop kernel's per-block reduction
/// drifts from the legacy serial walk (e.g., non-associative
/// op accidentally introduced, wrong canonical slot indexing),
/// this is the first thing that fires.
#[test]
fn s_perf_15_c_rejects_candidate_pack_byte_change() {
    if std::env::var("DSFB_S_PERF_15_C_CAPTURE").is_ok() {
        return;
    }
    let (candidates_sha, _, _, _) = capture_pre_rewrite_pins();
    assert_eq!(
        candidates_sha, PINNED_PRE_S_PERF_15_C_CANDIDATE_PACK_BYTES,
        "S-PERF.15.c: d_candidates arena SHA-256 drifted from pinned; byte-identity \
         contract VIOLATED at the CandidateInterval layer"
    );
}

/// `d_candidate_count` byte-equal to pinned. Defense-in-depth:
/// the blockcoop kernel READS but does not write this buffer,
/// so a drift here indicates upstream cascade (candidate_boundary
/// / candidate_fired) regression rather than a S-PERF.15.c kernel
/// bug.
#[test]
fn s_perf_15_c_rejects_candidate_count_byte_change() {
    if std::env::var("DSFB_S_PERF_15_C_CAPTURE").is_ok() {
        return;
    }
    let (_, count_sha, _, _) = capture_pre_rewrite_pins();
    assert_eq!(
        count_sha, PINNED_PRE_S_PERF_15_C_CANDIDATE_COUNT_BYTES,
        "S-PERF.15.c: d_candidate_count arena SHA-256 drifted from pinned; upstream \
         cascade integrity violated (this kernel does not write d_candidate_count)"
    );
}

/// End-to-end case-file hash byte-equal to pinned. Final
/// cascade verification — if any earlier layer drifted, this
/// hash would shift.
#[test]
fn s_perf_15_c_rejects_casefile_final_hash_change() {
    if std::env::var("DSFB_S_PERF_15_C_CAPTURE").is_ok() {
        return;
    }
    let (_, _, casefile_hash, _) = capture_pre_rewrite_pins();
    assert_eq!(
        casefile_hash, PINNED_PRE_S_PERF_15_C_CASEFILE_FINAL,
        "S-PERF.15.c: case-file final hash drifted from pinned; end-to-end cascade \
         broken somewhere in the post-S-PERF.15.c path"
    );
}

/// Admitted episode list byte-equal to pinned. Defense-in-depth
/// pin proving the bank's admission decisions are unchanged
/// across the S-PERF.15.c swap. If the CandidateInterval bytes
/// drifted, the bank would admit a different episode set and
/// this would fire even if the case-file final hash happened
/// to collide.
#[test]
fn s_perf_15_c_rejects_episode_summary_change() {
    if std::env::var("DSFB_S_PERF_15_C_CAPTURE").is_ok() {
        return;
    }
    let (_, _, _, episode_sha) = capture_pre_rewrite_pins();
    assert_eq!(
        episode_sha, PINNED_PRE_S_PERF_15_C_EPISODE_SUMMARY,
        "S-PERF.15.c: admitted episode list digest drifted from pinned; bank admission \
         decisions have shifted post-S-PERF.15.c"
    );
}

// ---------------------------------------------------------------
// 4 panel-required positives.
// ---------------------------------------------------------------

/// Canonical 16x128 K=1 D64 fixture admits 13 episodes
/// post-S-PERF.15.c.
#[test]
fn candidate_pack_blockcoop_admits_canonical_fixture() {
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
        "canonical 16x128 D64 should admit 13 episodes post-S-PERF.15.c"
    );
}

/// Cross-run determinism: two dispatches produce byte-identical
/// case-file hashes. Defense-in-depth against any residual race
/// in the block-cooperative reduction.
#[test]
fn candidate_pack_blockcoop_cross_run_determinism() {
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
        "S-PERF.15.c: blockcoop kernel non-deterministic across two runs"
    );
}

/// post-S-PERF.15.c bench snapshot exists + references the
/// canonical fixture (soft-skip if the post-bench is not yet
/// captured).
#[test]
fn candidate_pack_blockcoop_wall_time_recorded() {
    let post_path = "../../reports/d64_stage_timing_256x4096_K1_post_s_perf_15_c.txt";
    let abs_path = "/home/one/dsfb-gpu/reports/d64_stage_timing_256x4096_K1_post_s_perf_15_c.txt";
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
        "post-S-PERF.15.c bench snapshot must reference the canonical 256x4096 fixture"
    );
}

/// post-S-PERF.15.c ROOF receipt exists + references either the
/// new blockcoop kernel or the legacy kernel name (soft-skip if
/// the ROOF receipt is not yet captured).
#[test]
fn candidate_pack_blockcoop_roof_receipt_present() {
    let post_path = "../../reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_c_post.txt";
    let abs_path =
        "/home/one/dsfb-gpu/reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_c_post.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        return;
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let has_blockcoop = content.contains("candidate_pack_kernel_wide_blockcoop");
    let has_legacy = content.contains("candidate_pack_kernel_wide");
    assert!(
        has_blockcoop || has_legacy,
        "post-S-PERF.15.c ROOF receipt must reference the S-PERF.15.c blockcoop kernel \
         (candidate_pack_kernel_wide_blockcoop) or the legacy kernel name if the post-ROOF \
         run captured the legacy event-slot label"
    );
}
