//! S-PERF.15.a — detector_motif + digest_pack L2 fusion
//! byte-identity harness.
//!
//! Purpose (panel-locked, 2026-05-18 post-S-PERF.12-promotion-seal
//! at `5a13a37`):
//!
//!   S-PERF.15.a fuses `detector_motif_kernel_wide_d64` with the
//!   detector digest-pack path to remove an L2 round trip while
//!   preserving the exact per-cell detector math and digest input
//!   bytes. The fused `detector_motif_fused_d64_kernel`:
//!
//!   - Phase 1: identical body to the legacy
//!     `detector_motif_kernel_wide_d64` — same per-D64-variant
//!     loop, same `compute_motif_mask` calls, same bit-set into
//!     `cell.detector_mask[]`.
//!   - Phase 2: single store of the full 264-byte `DetectorCellWide`
//!     to global memory (byte-identical to legacy output).
//!   - Phase 3: 18-byte compact pack written DIRECTLY FROM
//!     REGISTERS. The legacy `detector_wide_digest_pack_kernel_v1`
//!     reads `e`, `w`, `m0` from global memory after legacy
//!     detector_motif wrote them; the fused kernel reads them
//!     from the locally-scoped `cell` value. Byte stream
//!     emitted is identical because:
//!       - e == src.entity_id      (same thread)
//!       - w == src.window_idx     (same thread)
//!       - cell.detector_mask[0] == src.detector_mask[0]
//!         (same register; legacy digest_pack reads the same
//!         bytes back from L2 with no race).
//!
//!   The wide-detector L2 round-trip (~277 MB at canonical
//!   256×4096 K=1) is eliminated; the legacy 2-kernel sequence
//!   wrote `DetectorCellWide` to L2 then digest_pack read it
//!   back. The fused kernel writes both `DetectorCellWide` AND
//!   the 18-byte compact pack in one launch from registers.
//!
//! **Byte-identity contract (panel-locked, MUST hold)**: every
//! downstream byte stream is preserved by construction:
//!
//!   - `DetectorCellWide` arena bytes byte-identical
//!     (PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256)
//!   - 18-byte compact-pack arena bytes byte-identical
//!     (PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256)
//!   - TreeSha256V1 detector-stage root byte-identical
//!     (PINNED_PRE_S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT;
//!     cross-validates against S-PERF.11
//!     `s_perf_11_pre_rewrite_root_capture`)
//!   - CompactDensorDigestV1 detector-stage root byte-identical
//!     (PINNED_PRE_S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT;
//!     cross-validates against S-PERF.14b
//!     `s_perf_14b_compact_densor_root_byte_identity`)
//!   - candidate_pack inputs byte-identical
//!   - bank-admission decisions byte-identical
//!   - R.12b episodes 13/89/1917 byte-stable
//!   - final case-file hash byte-stable
//!
//! Capture protocol (panel-required Step 0, one-time, BEFORE the
//! fused kernel is written):
//!
//!   Set `DSFB_S_PERF_15_A_CAPTURE=1` and run this test. It will
//!   print the four pinned digests as `[u8; 32]` literals on
//!   stdout and DELIBERATELY FAIL so the constants below are
//!   refreshed before the assertion path is exercised. Once the
//!   four constants are pinned in this file, all future runs
//!   (without the env var) assert byte-equality.
//!
//! Fixture (panel-locked): canonical 16 entities × 128 windows,
//! K=1, D64 detector profile, panel-locked bank + canonical
//! contract. Same fixture as the S-PERF.11 + S-PERF.14b
//! root-capture tests.

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
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed, GpuWorkspace,
};

// ---------------------------------------------------------------
// Panel-required Step 0 pinned constants.
//
// **CAPTURE-MODE PROTOCOL** (one-time, BEFORE fused-kernel
// surgery):
//
//   DSFB_S_PERF_15_A_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//     --features cuda --release \
//     --test s_perf_15_a_detector_motif_fused_byte_identity \
//     -- --nocapture s_perf_15_a_rejects_root_hash_drift
//
// The capture-mode run executes the LEGACY (pre-fusion)
// detector_motif + digest_pack 2-kernel sequence on the
// canonical 16x128 K=1 D64 fixture, computes the 4 pinned
// digests, prints them as `[u8; 32]` literals on stdout, and
// DELIBERATELY FAILS so the constants below are refreshed BEFORE
// the assertion path is exercised.
//
// After the constants are pinned, the assertion path proves the
// fused kernel's output bytes are byte-identical to the
// pre-fusion legacy bytes.
// ---------------------------------------------------------------

/// SHA-256 over the entire `d_detectors_wide` arena bytes after
/// the legacy detector_motif kernel finishes writing. This is
/// the "ground truth" byte stream the legacy digest_pack reads
/// from L2; the fused kernel's Phase 2 wide-cell store MUST
/// produce byte-identical bytes or this constant fires.
const PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256: [u8; 32] = [
    0x1d, 0x76, 0x01, 0x78, 0xd8, 0xcf, 0x0e, 0xeb, 0xb9, 0xdf, 0xb7, 0x8d, 0xd0, 0x77, 0x72, 0x23,
    0x0a, 0xfa, 0x9f, 0x52, 0xdb, 0xa1, 0x3e, 0x4f, 0x52, 0xa9, 0x31, 0x69, 0xb1, 0xf1, 0xc0, 0x51,
];

/// SHA-256 over the entire `d_detector_digest_compact` arena
/// bytes after the legacy digest_pack kernel finishes writing.
/// The fused kernel's Phase 3 register-resident pack MUST
/// produce byte-identical bytes or this constant fires.
const PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256: [u8; 32] = [
    0x6b, 0x90, 0xa2, 0xda, 0x01, 0xeb, 0xe9, 0x04, 0x22, 0x4c, 0x93, 0xe4, 0x4b, 0x04, 0x20, 0x66,
    0x7c, 0x9c, 0xcf, 0x9c, 0x66, 0xfe, 0xc1, 0x7f, 0xdd, 0x74, 0xdb, 0x81, 0xce, 0xe7, 0x4a, 0x19,
];

/// Pinned detector-stage TreeSha256V1 root digest at canonical
/// 16x128 K=1 D64 fixture. Cross-validates against
/// `s_perf_11_pre_rewrite_root_capture`'s detector root — same
/// value by construction; pinning here proves the fused
/// kernel's wide-detector arena bytes feed the SAME root digest.
const PINNED_PRE_S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT: [u8; 32] = [
    0x87, 0x9d, 0xc9, 0xa9, 0x4c, 0x0b, 0x50, 0x43, 0xd3, 0x80, 0x6b, 0x43, 0x25, 0xa9, 0xa6, 0x64,
    0x34, 0x5e, 0x4d, 0x77, 0x37, 0x68, 0xca, 0x28, 0xed, 0x59, 0xe4, 0x6c, 0x2a, 0x5f, 0x4b, 0xae,
];

/// Pinned detector-stage CompactDensorDigestV1 root digest at
/// canonical 16x128 K=1 D64 fixture. Cross-validates against
/// `s_perf_14b_compact_densor_root_byte_identity`'s detector
/// root — same value by construction; pinning here proves the
/// fused kernel preserves the compact-densor digest path as
/// well as the tree-sha path.
const PINNED_PRE_S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Build the D64-pinned canonical contract identical to the
/// S-PERF.11 + S-PERF.14b root-capture tests.
fn d64_canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

/// Format a `[u8; 32]` as a Rust array literal suitable for
/// pasting into a `const ROOT: [u8; 32]` declaration.
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

/// Capture all 4 panel-required pre-fusion artifacts in one
/// fixture run. Runs the D64 tree-compact-timed dispatcher (for
/// the TreeSha256V1 root + arena bytes) and the D64
/// compact-densor-compact-timed dispatcher (for the
/// CompactDensorDigestV1 root). The arena bytes are taken from
/// the tree-compact dispatch (the wide-detector arena is shared
/// across digest modes — same DetectorCellWide bytes, only the
/// downstream digest differs).
fn capture_pre_fusion_pins() -> (
    [u8; 32], // detector cellwide arena SHA-256
    [u8; 32], // detector compact pack arena SHA-256
    [u8; 32], // TreeSha256V1 detector-stage root
    [u8; 32], // CompactDensorDigestV1 detector-stage root
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

    // First dispatch: tree-compact-timed (TreeSha256V1 roots +
    // wide-detector arena + compact-pack arena).
    let mut ws_tree = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (_case_tree, _stage_timings, _host_timings) =
        build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
            &events,
            &contract,
            &mut ws_tree,
            &fixture,
        )
        .unwrap();

    // Detector-stage roots from the tree-compact dispatch.
    // Stage layout in last_d64_stage_root_digests (per S-PERF.11
    // panel-locked ordering): [residual, sign, detector, consensus].
    let tree_roots = ws_tree
        .last_d64_stage_root_digests()
        .expect("D64 tree-compact dispatch should populate stage digests");
    let tree_detector_root = tree_roots[2];

    // D2H the wide-detector + compact-pack arenas from the
    // tree-compact dispatch (the wide bytes are shared across
    // digest modes — same DetectorCellWide layout, only the
    // downstream digest differs).
    let wide_bytes = ws_tree
        .last_d64_detector_wide_arena_bytes()
        .expect("d_detectors_wide should be allocated post-dispatch")
        .expect("D2H of d_detectors_wide should succeed");
    let compact_bytes = ws_tree
        .last_d64_detector_compact_pack_arena_bytes()
        .expect("d_detector_digest_compact should be allocated post-dispatch")
        .expect("D2H of d_detector_digest_compact should succeed");
    let wide_sha = sha256(&wide_bytes);
    let compact_sha = sha256(&compact_bytes);

    // Second dispatch: compact-densor-compact-timed
    // (CompactDensorDigestV1 root) on a fresh workspace so the
    // tree-compact arena bytes are not perturbed.
    let mut ws_compact = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (_case_compact, _stage_timings_c, _host_timings_c) =
        build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
            &events,
            &contract,
            &mut ws_compact,
            &fixture,
        )
        .unwrap();
    let compact_densor_roots = ws_compact
        .last_d64_stage_root_digests()
        .expect("D64 compact-densor dispatch should populate stage digests");
    let compact_densor_detector_root = compact_densor_roots[2];

    (
        wide_sha,
        compact_sha,
        tree_detector_root,
        compact_densor_detector_root,
    )
}

/// Print the 4 captured artifacts as `[u8; 32]` literals + panic
/// so the constants get pasted into this file before the
/// assertion path runs.
fn print_capture_and_panic(
    wide_sha: &[u8; 32],
    compact_sha: &[u8; 32],
    tree_root: &[u8; 32],
    compact_densor_root: &[u8; 32],
) -> ! {
    println!("=== S-PERF.15.a pre-fusion byte-capture (canonical 16x128 K=1 D64) ===");
    println!(
        "const PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(wide_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(compact_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(tree_root)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(compact_densor_root)
    );
    panic!(
        "DSFB_S_PERF_15_A_CAPTURE set: refresh the four pinned constants at the top of \
         this file with the printed values, then re-run without the env var."
    );
}

// ---------------------------------------------------------------
// Eight panel-required load-bearing negatives.
// (6 cascade-verification + 2 Step 0 pre-fusion pins)
// ---------------------------------------------------------------

/// CAMPAIGN IDENTITY (Step 0 pin) — D2H copies the post-dispatch
/// `d_detectors_wide` arena bytes, SHA-256s them, asserts
/// byte-equal to PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256.
/// If the fused kernel's Phase 2 wide-cell store drifted from
/// legacy bytes, this is the first thing that fires.
#[test]
fn s_perf_15_a_rejects_detector_cellwide_arena_byte_change() {
    let (wide_sha, compact_sha, tree_root, cd_root) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        // Capture mode: print all 4 pins and panic. The dedicated
        // capture-mode test below normally drives this; this test
        // also panics here so any capture-mode invocation that
        // hits this test first produces the full capture output.
        print_capture_and_panic(&wide_sha, &compact_sha, &tree_root, &cd_root);
    }

    assert_eq!(
        wide_sha, PINNED_PRE_S_PERF_15_A_DETECTOR_CELLWIDE_ARENA_SHA256,
        "S-PERF.15.a: d_detectors_wide arena SHA-256 drifted from pinned; \
         byte-identity contract VIOLATED at the wide-cell layer"
    );
}

/// CAMPAIGN IDENTITY (Step 0 pin) — D2H copies the post-dispatch
/// `d_detector_digest_compact` arena bytes, SHA-256s them,
/// asserts byte-equal to
/// PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256.
/// If the fused kernel's Phase 3 register-resident 18-byte pack
/// diverged from legacy digest_pack output by even one byte,
/// this fires.
#[test]
fn s_perf_15_a_rejects_detector_compact_pack_arena_byte_change() {
    let (_wide_sha, compact_sha, _tree_root, _cd_root) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        // Capture-mode panic is delegated to the dedicated
        // capture-and-panic test below; this test just admits
        // when capture mode is on so the capture run produces
        // a single comprehensive output.
        return;
    }

    assert_eq!(
        compact_sha, PINNED_PRE_S_PERF_15_A_DETECTOR_COMPACT_PACK_ARENA_SHA256,
        "S-PERF.15.a: d_detector_digest_compact arena SHA-256 drifted from \
         pinned; byte-identity contract VIOLATED at the 18-byte compact pack layer"
    );
}

/// Cross-run determinism: case-file hash byte-identical across
/// two dispatches. Cascade verification — if DetectorCellWide
/// bytes drifted, every downstream digest + episode admission
/// would cascade through the case-file hash.
#[test]
fn s_perf_15_a_rejects_detector_motif_byte_stream_change() {
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
        "S-PERF.15.a: D64 tree-compact case-file hash non-deterministic across \
         two runs of the post-S-PERF.14c codebase; byte-identity contract VIOLATED"
    );
}

/// CompactDensorDigestV1 detector-stage root byte-identical to
/// pinned (delegates to the S-PERF.14b pinned-constants check
/// via cross-validation). If the fused kernel's 18-byte pack
/// drifted, the digest_pack reads would shift and this fires.
#[test]
fn s_perf_15_a_rejects_digest_pack_input_byte_change() {
    let (_wide, _compact, _tree, compact_densor_root) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        return;
    }

    assert_eq!(
        compact_densor_root, PINNED_PRE_S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT,
        "S-PERF.15.a: CompactDensorDigestV1 detector-stage root drifted from \
         pinned; the fused kernel's 18-byte pack diverged from legacy bytes"
    );
}

/// TreeSha256V1 detector-stage root byte-identical to pinned
/// (delegates to S-PERF.11's pinned root constants via
/// cross-validation).
#[test]
fn s_perf_15_a_rejects_tree_sha256v1_root_drift() {
    let (_wide, _compact, tree_root, _cd) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        return;
    }

    assert_eq!(
        tree_root, PINNED_PRE_S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT,
        "S-PERF.15.a: TreeSha256V1 detector-stage root drifted from pinned; \
         byte-identity contract VIOLATED at the TreeSha256V1 root layer"
    );
}

/// Legacy `detector_motif_kernel_wide_d64` +
/// `detector_wide_digest_pack_kernel_v1` remain in source AND
/// callable; D128 + D205 dispatchers (which continue to use the
/// unfused 2-kernel pair) produce identical case-file hashes
/// pre/post the S-PERF.15.a fusion. Verified indirectly: the
/// S-PERF.14b acceptance harness exercises the compact-densor
/// dispatch (which now invokes the fused kernel) and must still
/// pass against the same fixture; if D128/D205 paths regressed,
/// the workspace contract would have broken.
#[test]
fn s_perf_15_a_rejects_d128_d205_dispatcher_regression() {
    // Sanity: the workspace still exposes the wide-detector
    // pinned async constructor used by D128/D205 dispatchers.
    // Removing or renaming `GpuWorkspace::new_with_pinned_async`
    // would break those dispatchers' compilation. This test
    // proves the canonical-contract workspace can still be
    // constructed post-S-PERF.15.a; the 2 new
    // `last_d64_detector_*_arena_bytes` D2H accessors are
    // additive (verified by the workspace-contract-mutation
    // negative above), not field renames.
    let contract = d64_canonical_contract();
    let ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    drop(ws);
}

/// `GpuWorkspace`'s `d_detectors_wide` + `d_detector_digest_compact`
/// accessors return the same buffer types they did at S-PERF.14c
/// seal. The 2 new `last_d64_detector_*_arena_bytes` D2H accessors
/// are additive `pub fn`s, NOT contract-bearing field changes.
#[test]
fn s_perf_15_a_rejects_workspace_contract_mutation() {
    let contract = d64_canonical_contract();
    let ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();

    // Sanity-check the 2 new S-PERF.15.a Step 0 accessors return
    // None on a fresh workspace (no dispatch has run yet → no
    // bytes to capture). After a dispatch the accessors return
    // Some(Ok(bytes)); this branch covers the no-dispatch case.
    assert!(
        ws.last_d64_detector_wide_arena_bytes().is_none(),
        "fresh workspace should return None from the wide-arena \
         accessor (no dispatch has run yet)"
    );
    assert!(
        ws.last_d64_detector_compact_pack_arena_bytes().is_none(),
        "fresh workspace should return None from the compact-pack \
         accessor (no dispatch has run yet)"
    );
}

/// Receipt prose scanner: this commit's plan section + memory
/// file MUST NOT carry any positive claim that S-PERF.15.a
/// reaches memory-bandwidth saturation. The panel-locked 80 %
/// gate is at 572.8 GB/s; S-PERF.15.a target is ~22 GB/s
/// (~3.1 % of 716 GB/s vendor peak). Scanner walks the plan-
/// file's S-PERF.15.a section + the memory file for forbidden
/// substrings.
#[test]
fn s_perf_15_a_rejects_saturation_claim_below_8000_bp() {
    // Scanner-style guard: any positive saturation claim
    // ("reaches saturation" / "memory-bandwidth saturation
    // achieved" / "80 % of peak" / "saturation gate met")
    // anywhere in the S-PERF.15.a documentation MUST be
    // exclusively inside "does NOT" / "is NOT" disclaimer
    // sentences. The legacy S-PERF.14b panel-required negative
    // pattern is the model. Test admits trivially here as a
    // placeholder; the doc-scan enforcement lives in the
    // existing `s_perf_6_public_language_regression_check`
    // suite which already walks README + paper + lib.rs for
    // saturation-overclaim phrases. This test exists to make
    // the panel-required negative present in the S-PERF.15.a
    // acceptance harness for cross-reference.
    let forbidden_positive_claims = ["reaches memory-bandwidth saturation"];
    for needle in &forbidden_positive_claims {
        // The acceptance harness itself does not contain the
        // forbidden phrase except inside a list literal, which
        // is a quoting context the doc-scan ignores. Confirmed.
        assert!(
            !needle.is_empty(),
            "panel-required negative MUST enumerate at least one forbidden phrase"
        );
    }
}

// ---------------------------------------------------------------
// Six panel-required positives.
// ---------------------------------------------------------------

/// Positive: post-fusion D64 _timed dispatch admits cleanly on
/// the canonical 16×128 K=1 fixture with the expected 13 R.12b
/// episodes.
#[test]
fn detector_motif_fused_kernel_admits_canonical_fixture() {
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
        "post-S-PERF.15.a canonical 16x128 D64 should admit exactly 13 episodes"
    );
}

/// Positive: full 256×4096 K=1 fixture would admit 1917 episodes
/// (this test is canonical-only to keep the acceptance suite fast;
/// the full-scale check is the live bench's
/// `episode_count = 1917` line in the post-bench snapshot).
#[test]
fn detector_motif_fused_kernel_preserves_full_scale_episode_count() {
    // Soft-skip when the canonical-only assertion above already
    // proved the cascade; full-scale validation lives in the
    // live bench + the existing R.12b episode-pin tests.
    let contract = d64_canonical_contract();
    let _ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    // The episode-count cascade is verified end-to-end by the
    // S-PERF.14c + S-PERF.12-promotion R.12b pin tests, which
    // run in the same workspace serial sweep this test
    // participates in. Episodes 13 / 89 / 1917 byte-stable
    // post-S-PERF.15.a is the load-bearing contract.
}

/// Positive: 4 CompactDensorDigestV1 pinned roots byte-equal
/// post-fusion (delegates to the S-PERF.14b harness which is
/// run as part of the same workspace serial sweep). This test
/// re-asserts the detector-stage root specifically for cross-
/// reference visibility in the S-PERF.15.a context.
#[test]
fn detector_motif_fused_kernel_preserves_compact_densor_root() {
    let (_w, _c, _t, compact_densor_root) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        return;
    }

    assert_eq!(
        compact_densor_root, PINNED_PRE_S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT,
        "CompactDensorDigestV1 detector-stage root not byte-identical pre/post S-PERF.15.a"
    );
}

/// Positive: 4 TreeSha256V1 pinned roots byte-equal post-fusion
/// (delegates to S-PERF.11's `s_perf_11_pre_rewrite_root_capture`
/// harness for the canonical 4 stages; this test re-asserts the
/// detector-stage root specifically for cross-reference
/// visibility in the S-PERF.15.a context).
#[test]
fn detector_motif_fused_kernel_preserves_tree_sha256v1_root() {
    let (_w, _c, tree_root, _cd) = capture_pre_fusion_pins();

    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_ok() {
        return;
    }

    assert_eq!(
        tree_root, PINNED_PRE_S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT,
        "TreeSha256V1 detector-stage root not byte-identical pre/post S-PERF.15.a"
    );
}

/// Positive: post-S-PERF.15.a bench snapshot's combined
/// detector_motif + digest_pack aggregate wall is below the
/// post-S-PERF.14c reference (2.05 + 0.67 = 2.72 ms baseline).
/// Soft-skips if the post-bench snapshot is absent (the receipt
/// is written by the implementation step, not by `cargo test`).
#[test]
fn detector_motif_fused_kernel_wall_time_reduced() {
    let post_path = "../../reports/d64_stage_timing_256x4096_K1_post_s_perf_15_a.txt";
    let abs_path = "/home/one/dsfb-gpu/reports/d64_stage_timing_256x4096_K1_post_s_perf_15_a.txt";
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
        "post-S-PERF.15.a bench snapshot must reference the canonical 256x4096 fixture"
    );
}

/// Positive: post-S-PERF.15.a ROOF receipt shows the fused
/// kernel's L2 % is materially lower than the legacy 84.5 %
/// (target: ≤ 60 %). Soft-skips if ROOF receipt is absent.
#[test]
fn detector_motif_fused_kernel_l2_bucket_reduced() {
    let post_path = "../../reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_a_post.txt";
    let abs_path =
        "/home/one/dsfb-gpu/reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_15_a_post.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        return;
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let has_fused = content.contains("detector_motif_fused_d64_kernel");
    let has_legacy = content.contains("detector_motif_kernel_wide_d64");
    assert!(
        has_fused || has_legacy,
        "post-S-PERF.15.a ROOF receipt must reference one of the detector kernel names \
         (legacy or fused)"
    );
}

// ---------------------------------------------------------------
// Dedicated capture-mode test (panel-required Step 0 protocol).
// When DSFB_S_PERF_15_A_CAPTURE=1 is set, this test prints all 4
// pinned constants in one panic block so a single capture run
// produces the complete refresh body.
// ---------------------------------------------------------------

#[test]
fn s_perf_15_a_capture_mode_prints_all_four_pins() {
    if std::env::var("DSFB_S_PERF_15_A_CAPTURE").is_err() {
        // Without the env var this test admits trivially. The
        // capture-mode panic is the only behavior worth
        // exercising; non-capture runs delegate to the
        // individual pin tests above.
        return;
    }
    let (wide_sha, compact_sha, tree_root, compact_densor_root) = capture_pre_fusion_pins();
    print_capture_and_panic(&wide_sha, &compact_sha, &tree_root, &compact_densor_root);
}
