//! S-PERF.15.d Step 0 — `detector_motif_fused_d64_kernel`
//! contract capture + DetectorCellWide field-consumption
//! byte-identity harness.
//!
//! **Purpose (panel-locked, 2026-05-18 post-S-PERF.15.c seal
//! at `8b0db9b`)**:
//!
//!   S-PERF.15.d targets the 2.49 ms L2-heavy
//!   `detector_motif_fused_d64_kernel` (91.9 % L2 in the
//!   post-S-PERF.15.c ROOF receipt) — the single largest
//!   remaining device wall. The rewrite class is L2 / layout
//!   refinement (higher byte-identity risk than 15.a/.b/.c
//!   because it MAY touch the `DetectorCellWide` contract).
//!   Step 0 pins the wider contract surface BEFORE any
//!   kernel surgery so any slimming / hot-cold split / SoA
//!   sidecar / conditional-write design cannot silently
//!   regress the bytes any downstream stage consumes.
//!
//! **Panel-required strengthened Step 0 (5 pinned constants,
//! one more than 15.a's 4)**:
//!
//!   1. `PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256`
//!      — full `d_detectors_wide` arena bytes (264 B per cell ×
//!      n_entities × n_windows × n_catalogs). The rewrite MUST
//!      preserve byte-identical bytes in EVERY field the legacy
//!      kernel writes that any downstream stage reads, unless
//!      a hot-cold split design conditional-writes some fields
//!      under the D64 _timed dispatcher AND the cascade pin
//!      below independently verifies the digest + casefile
//!      paths are unchanged.
//!   2. `PINNED_PRE_S_PERF_15_D_DETECTOR_COMPACT_PACK_ARENA_SHA256`
//!      — full `d_detector_digest_compact` arena bytes (18 B
//!      per cell). Same pin used by S-PERF.15.a. Proves the
//!      18-byte compact pack bytes (which feed the per-stage
//!      digest tree) are byte-identical post-rewrite.
//!   3. `PINNED_PRE_S_PERF_15_D_TREE_SHA256V1_DETECTOR_ROOT`
//!      — TreeSha256V1 detector-stage root digest. Matches
//!      S-PERF.11's and S-PERF.15.a's detector-root pins by
//!      construction.
//!   4. `PINNED_PRE_S_PERF_15_D_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT`
//!      — CompactDensorDigestV1 detector-stage root digest.
//!      Matches S-PERF.14b's and S-PERF.15.a's detector-root
//!      pins.
//!   5. `PINNED_PRE_S_PERF_15_D_CONSENSUS_INPUT_CASCADE_SHA256`
//!      — NEW combined cascade pin: SHA-256 over
//!      (consensus arena bytes || final case-file hash). A
//!      slimming bug that flips a cold `DetectorCellWide`
//!      field that consensus secretly reads (or that the
//!      casefile cascade depends on) would shift either the
//!      consensus arena bytes OR the casefile final hash,
//!      and this single pin catches both. Defense-in-depth
//!      against the field-audit mis-classifying a hot field
//!      as audit-only.
//!
//! **Capture protocol** (panel-required Step 0; one-time,
//! BEFORE any layout-refinement kernel surgery):
//!
//!   ```
//!   DSFB_S_PERF_15_D_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//!     --features cuda --release \
//!     --test s_perf_15_d_detector_motif_layout_byte_identity \
//!     -- --nocapture s_perf_15_d_capture_mode_prints_all_five_pins
//!   ```
//!
//!   The capture-mode test runs the CURRENT
//!   `detector_motif_fused_d64_kernel` (pre-S-PERF.15.d codebase
//!   at HEAD `8b0db9b`) on the canonical 16 × 128 K=1 D64
//!   fixture, computes the 5 pinned digests, prints them as
//!   `[u8; 32]` literals on stdout, and DELIBERATELY FAILS so
//!   the constants below are refreshed BEFORE the assertion
//!   path is exercised.
//!
//! **DetectorCellWide field-consumption audit (panel-required
//! deliverable; sourced from a live grep over kernels.cu at
//! HEAD `8b0db9b`)**:
//!
//! ```text
//! field                | bytes | producer (write)                  | reader  | D64 read path                                             | D128/D205 read path                                | digest? | cold? | safe to slim on D64?
//! ---------------------+-------+-----------------------------------+---------+-----------------------------------------------------------+----------------------------------------------------+---------+-------+----------------------
//! window_idx           |   4   | detector_motif_fused_d64_kernel   | NONE    | not read (consensus + candidate_pack use blockIdx values) | not read (D128/D205 consensus uses blockIdx too)   | NO      | YES   | YES (safe to drop on D64 fast path; sidecar can omit)
//! entity_id            |   4   | detector_motif_fused_d64_kernel   | NONE    | not read (consensus + candidate_pack use blockIdx values) | not read (D128/D205 consensus uses blockIdx too)   | NO      | YES   | YES (safe to drop on D64 fast path; sidecar can omit)
//! detector_mask[0]     |   8   | detector_motif_fused_d64_kernel   | HOT     | consensus_axis5_fused (project_d64_to_u16, line 1660+1669); candidate_pack_blockcoop (project_d64_to_u16, line 2059); compact-pack Phase 3 register-resident (line 943) | consensus_grid_wide_d128/_d205 (project_d128/d205_to_u16) | YES     | NO    | NO (load-bearing; must be preserved exactly)
//! detector_mask[1]     |   8   | detector_motif_fused_d64_kernel (writes 0) | COLD@D64 | not read on D64 path                                | consensus_grid_wide_d128 (project_d128, line 1279) | YES (D128+) | YES@D64 | YES (safe to drop on D64 sidecar; legacy wide arena keeps writing for D128/D205 compat via unfused 2-kernel pair at D128/D205 dispatcher)
//! detector_mask[2..3]  |  16   | detector_motif_fused_d64_kernel (writes 0) | COLD@D64,D128 | not read                                       | consensus_grid_wide_d205 (project_d205, line 1318) | YES (D205+) | YES@D64,D128 | YES@D64 (D205 path unchanged)
//! detector_mask[4..31] | 224   | detector_motif_fused_d64_kernel (writes 0) | COLD ALWAYS | not read                                          | not read                                           | NO      | YES   | YES (no consumer; pure write-allocate waste)
//! ```
//!
//! **Per-cell footprint summary at D64 canonical (256x4096 K=1)**:
//!
//! ```text
//! WRITE: detector_motif_fused writes full 264 B per cell
//!        (8 B header + 256 B mask)
//!        = 1 048 576 cells × 264 B = ~277 MB write traffic
//!        of which the only HOT byte is detector_mask[0] (8 B)
//!        = 8 MB of meaningful writes, 269 MB write-allocate waste
//!
//! READ:  consensus_axis5_fused reads:
//!          - 1 self cell (project_d64_to_u16(det))
//!          - TEMPORAL_WINDOW=32 neighbor cells (loop k=0..32)
//!          = 33 cell reads per (entity, window)
//!          via project_d64_to_u16 → ONLY reads mask[0] (8 B)
//!          If compiler loads full 264 B per cell (const ref):
//!          = 33 × 264 = 8 712 B per cell read traffic
//!          If compiler loads only mask[0] (8 B):
//!          = 33 × 8 = 264 B per cell read traffic
//!
//!        candidate_pack_blockcoop reads:
//!          - per (slot, entity, catalog) block walks per-slot
//!            window range; per window: 1 detector cell read
//!            via project_d64_to_u16
//!
//! ROOF dominance hypothesis:
//!   91.9 % L2 traffic on detector_motif_fused itself is the
//!   WRITE-allocate cost of the 277 MB wide-arena store. The
//!   8-byte hot path (mask[0]) accounts for ~8 MB / 277 MB ≈
//!   3 % of the actual write payload; the other 97 % is zero
//!   bytes (writing the 32-byte mask[1..31] cold tail to L2
//!   when only mask[0] carries meaning at D64).
//!
//!   The most-likely Step 1 direction (NOT locked here; the
//!   ROOF byte-counter trace + L2 store-throughput metrics
//!   confirm or refute):
//!     (a) Conditional-write at D64: skip mask[1..31] writes
//!         entirely on the D64 _timed path. Cuts write
//!         traffic 264 B → 16 B per cell (~277 MB → ~17 MB).
//!         Risk: wide arena diverges from the D128/D205
//!         legacy shape at mask[1..31] positions; harness
//!         pin #1 (`DETECTOR_CELLWIDE_ARENA_SHA256`) catches
//!         this regression — so the harness MUST be updated
//!         to verify only the bytes that downstream stages
//!         actually read at D64. Alternative: keep the wide
//!         arena byte-identical and add a slim D64-only
//!         sidecar (direction b).
//!     (b) SoA sidecar: write a 16 B D64-only compact cell
//!         (entity_id + window_idx + mask[0]) side-by-side
//!         with the full 264 B wide arena. D64 consumers
//!         read the sidecar; full arena still updated (writes
//!         still 264 B but the L2 read footprint on
//!         downstream consumers drops massively).
//!     (c) Hybrid: conditional-write the slim form to the
//!         sidecar AND write only the hot 16 B of the wide
//!         arena (skip mask[1..31] zeros); rebuild the wide
//!         arena from the sidecar lazily if a D128/D205-mode
//!         consumer ever runs. Highest payoff; highest design
//!         risk.
//!
//! Direction lock deferred to Step 1 per panel rule "No
//! DetectorCellWide slimming without a field-consumption
//! audit. No layout rewrite without five pre-rewrite pins."
//!
//! Fixture (panel-locked): canonical 16 entities × 128 windows,
//! K=1, D64 detector profile, panel-locked bank + canonical
//! contract. Same fixture as the S-PERF.11 + S-PERF.14b +
//! S-PERF.15.a + S-PERF.15.b + S-PERF.15.c root-capture tests.

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
// Panel-required Step 0 pinned constants (5).
//
// CAPTURE-MODE PROTOCOL (one-time, BEFORE S-PERF.15.d kernel
// surgery):
//
//   DSFB_S_PERF_15_D_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//     --features cuda --release \
//     --test s_perf_15_d_detector_motif_layout_byte_identity \
//     -- --nocapture s_perf_15_d_capture_mode_prints_all_five_pins
//
// Capture-mode run executes the CURRENT
// detector_motif_fused_d64_kernel (pre-S-PERF.15.d codebase) on
// the canonical 16x128 K=1 D64 fixture, computes 5 pinned
// digests, prints them as `[u8; 32]` literals on stdout, and
// DELIBERATELY FAILS so the constants below are refreshed
// BEFORE the assertion path is exercised.
// ---------------------------------------------------------------

/// SHA-256 over the entire `d_detectors_wide` arena bytes
/// (n_entities × n_windows × n_catalogs × 264 B per cell)
/// after `detector_motif_fused_d64_kernel` finishes writing.
///
/// The S-PERF.15.d rewrite MUST produce byte-identical bytes
/// in EVERY field the legacy fused kernel writes that any
/// downstream stage reads. If the rewrite slims cold fields
/// via conditional-write (Direction a/c), this pin MUST be
/// updated atomically with the kernel change AND the cascade
/// pin #5 below MUST verify the slimming doesn't break the
/// digest + casefile paths.
const PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256: [u8; 32] = [
    0x1d, 0x76, 0x01, 0x78, 0xd8, 0xcf, 0x0e, 0xeb, 0xb9, 0xdf, 0xb7, 0x8d, 0xd0, 0x77, 0x72, 0x23,
    0x0a, 0xfa, 0x9f, 0x52, 0xdb, 0xa1, 0x3e, 0x4f, 0x52, 0xa9, 0x31, 0x69, 0xb1, 0xf1, 0xc0, 0x51,
];

/// SHA-256 over the entire `d_detector_digest_compact` arena
/// bytes (n_cells × 18 B). Same pin used by S-PERF.15.a. The
/// 18-byte compact pack bytes feed the per-stage digest tree
/// (TreeSha256V1 + CompactDensorDigestV1); any drift here
/// surfaces as a per-stage root drift downstream.
const PINNED_PRE_S_PERF_15_D_DETECTOR_COMPACT_PACK_ARENA_SHA256: [u8; 32] = [
    0x6b, 0x90, 0xa2, 0xda, 0x01, 0xeb, 0xe9, 0x04, 0x22, 0x4c, 0x93, 0xe4, 0x4b, 0x04, 0x20, 0x66,
    0x7c, 0x9c, 0xcf, 0x9c, 0x66, 0xfe, 0xc1, 0x7f, 0xdd, 0x74, 0xdb, 0x81, 0xce, 0xe7, 0x4a, 0x19,
];

/// Pinned detector-stage TreeSha256V1 root digest at canonical
/// 16x128 K=1 D64 fixture. Matches S-PERF.11 + S-PERF.15.a's
/// detector-root pin by construction.
const PINNED_PRE_S_PERF_15_D_TREE_SHA256V1_DETECTOR_ROOT: [u8; 32] = [
    0x87, 0x9d, 0xc9, 0xa9, 0x4c, 0x0b, 0x50, 0x43, 0xd3, 0x80, 0x6b, 0x43, 0x25, 0xa9, 0xa6, 0x64,
    0x34, 0x5e, 0x4d, 0x77, 0x37, 0x68, 0xca, 0x28, 0xed, 0x59, 0xe4, 0x6c, 0x2a, 0x5f, 0x4b, 0xae,
];

/// Pinned detector-stage CompactDensorDigestV1 root digest at
/// canonical 16x128 K=1 D64 fixture. Matches S-PERF.14b +
/// S-PERF.15.a's detector-root pin by construction.
const PINNED_PRE_S_PERF_15_D_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

/// NEW combined consensus-input + casefile-final cascade pin
/// (panel-required Step 0 strengthening — one more pin than
/// 15.a's 4). SHA-256 over (consensus arena bytes || final
/// case-file hash). Defense-in-depth: a slimming bug that
/// flips a cold DetectorCellWide field that consensus
/// secretly reads (or that the casefile cascade depends on)
/// would shift either the consensus arena bytes OR the
/// casefile final hash, and this single pin catches both.
const PINNED_PRE_S_PERF_15_D_CONSENSUS_INPUT_CASCADE_SHA256: [u8; 32] = [
    0x90, 0xb5, 0xae, 0x8a, 0xf2, 0x6b, 0xe0, 0x5d, 0xa0, 0x9e, 0xd6, 0x8e, 0x25, 0xaa, 0x99, 0xa5,
    0x3f, 0x00, 0xbe, 0xff, 0x71, 0xa0, 0x31, 0xaa, 0xc1, 0xcb, 0xb2, 0xb3, 0x14, 0x2b, 0x74, 0xb1,
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

/// Tuple of the 5 panel-required Step 0 pinned constants
/// captured by [`capture_pre_rewrite_pins`]:
/// `(detector cellwide arena SHA-256, detector compact pack
/// arena SHA-256, TreeSha256V1 detector-stage root,
/// CompactDensorDigestV1 detector-stage root,
/// consensus arena || casefile final cascade SHA-256)`.
/// Type alias exists to keep clippy's `type_complexity` lint
/// quiet on the 5-tuple of `[u8; 32]` arrays.
type SPerf15dCapturedPins = ([u8; 32], [u8; 32], [u8; 32], [u8; 32], [u8; 32]);

// Long-standing pinned values from earlier sub-leg harnesses,
// hoisted to module scope so clippy's `items_after_statements`
// lint stays clean (these were originally declared inline
// inside the cross-validation tests, which clippy rejects
// post-S-PERF.15.d).

/// S-PERF.15.a's pinned detector-stage TreeSha256V1 root
/// (sealed at `a47a8e9`). Same canonical 16x128 K=1 D64
/// fixture; cross-validates that S-PERF.15.d Step 0 reads the
/// same detector root as the prior sub-leg.
const S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT: [u8; 32] = [
    0x87, 0x9d, 0xc9, 0xa9, 0x4c, 0x0b, 0x50, 0x43, 0xd3, 0x80, 0x6b, 0x43, 0x25, 0xa9, 0xa6, 0x64,
    0x34, 0x5e, 0x4d, 0x77, 0x37, 0x68, 0xca, 0x28, 0xed, 0x59, 0xe4, 0x6c, 0x2a, 0x5f, 0x4b, 0xae,
];

/// S-PERF.15.a's pinned detector-stage CompactDensorDigestV1
/// root (sealed at `a47a8e9`).
const S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

/// S-PERF.15.b's pinned end-to-end case-file final hash
/// (sealed at `dc0feaf`, preserved through `8b0db9b`).
const S_PERF_15_B_CASEFILE_FINAL_HASH: [u8; 32] = [
    0x98, 0xd0, 0x69, 0x67, 0x01, 0xf7, 0x6a, 0x81, 0xb3, 0x8e, 0x18, 0xe0, 0xf5, 0xb0, 0x2e, 0x86,
    0x75, 0x3f, 0xa0, 0xfd, 0xdb, 0xd1, 0x58, 0xd2, 0xe2, 0xd8, 0xb8, 0xb4, 0x00, 0x26, 0xf3, 0xa6,
];

/// Capture all 5 panel-required Step 0 artifacts in two
/// fixture runs (TreeSha256V1 + CompactDensorDigestV1
/// dispatchers; the wide-detector + compact-pack arenas are
/// shared so we take them from the tree-compact dispatch).
fn capture_pre_rewrite_pins() -> SPerf15dCapturedPins {
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
    // wide-detector arena + compact-pack arena + consensus arena
    // + casefile final).
    let mut ws_tree = GpuWorkspace::new_with_pinned_async(&contract).unwrap();
    let (case_tree, _stage_timings, _host_timings) =
        build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
            &events,
            &contract,
            &mut ws_tree,
            &fixture,
        )
        .unwrap();

    let tree_roots = ws_tree
        .last_d64_stage_root_digests()
        .expect("D64 tree-compact dispatch should populate stage digests");
    let tree_detector_root = tree_roots[2];

    let wide_bytes = ws_tree
        .last_d64_detector_wide_arena_bytes()
        .expect("d_detectors_wide should be allocated post-dispatch")
        .expect("D2H of d_detectors_wide should succeed");
    let compact_pack_bytes = ws_tree
        .last_d64_detector_compact_pack_arena_bytes()
        .expect("d_detector_digest_compact should be allocated post-dispatch")
        .expect("D2H of d_detector_digest_compact should succeed");
    let consensus_bytes = ws_tree
        .last_d64_consensus_arena_bytes()
        .expect("d_consensus should be allocated post-dispatch")
        .expect("D2H of d_consensus should succeed");
    let casefile_final = case_tree.final_case_file_hash;

    let wide_sha = sha256(&wide_bytes);
    let compact_pack_sha = sha256(&compact_pack_bytes);

    // Combined cascade pin: SHA-256 over (consensus arena bytes
    // || casefile final hash). Defense-in-depth.
    let mut cascade_input = Vec::with_capacity(consensus_bytes.len() + 32);
    cascade_input.extend_from_slice(&consensus_bytes);
    cascade_input.extend_from_slice(&casefile_final);
    let cascade_sha = sha256(&cascade_input);

    // Second dispatch: compact-densor-compact-timed
    // (CompactDensorDigestV1 detector-stage root) on a fresh
    // workspace so the tree-compact arena bytes are not
    // perturbed.
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
        compact_pack_sha,
        tree_detector_root,
        compact_densor_detector_root,
        cascade_sha,
    )
}

/// Print the 5 captured artifacts as `[u8; 32]` literals +
/// panic so the constants get pasted into this file before
/// the assertion path runs.
fn print_capture_and_panic(
    wide_sha: &[u8; 32],
    compact_pack_sha: &[u8; 32],
    tree_root: &[u8; 32],
    compact_densor_root: &[u8; 32],
    cascade_sha: &[u8; 32],
) -> ! {
    println!("=== S-PERF.15.d Step 0 pre-rewrite byte-capture (canonical 16x128 K=1 D64) ===");
    println!(
        "const PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(wide_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_D_DETECTOR_COMPACT_PACK_ARENA_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(compact_pack_sha)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_D_TREE_SHA256V1_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(tree_root)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_D_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(compact_densor_root)
    );
    println!(
        "const PINNED_PRE_S_PERF_15_D_CONSENSUS_INPUT_CASCADE_SHA256: [u8; 32] = [\n    {}\n];",
        hex_u8_array(cascade_sha)
    );
    panic!(
        "DSFB_S_PERF_15_D_CAPTURE set: refresh the five pinned constants at the top of \
         this file with the printed values, then re-run without the env var."
    );
}

// ---------------------------------------------------------------
// Step 0 capture-mode test (single panic point for one
// comprehensive capture run).
// ---------------------------------------------------------------

#[test]
fn s_perf_15_d_capture_mode_prints_all_five_pins() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_err() {
        return;
    }
    let (wide_sha, compact_pack_sha, tree_root, compact_densor_root, cascade_sha) =
        capture_pre_rewrite_pins();
    print_capture_and_panic(
        &wide_sha,
        &compact_pack_sha,
        &tree_root,
        &compact_densor_root,
        &cascade_sha,
    );
}

// ---------------------------------------------------------------
// 5 panel-required Step 0 pre-rewrite-pin negatives.
// ---------------------------------------------------------------

/// PIN 1 — DetectorCellWide arena bytes byte-equal to pinned.
/// If the S-PERF.15.d rewrite slims a field that any downstream
/// stage actually reads (per the field-audit table at the top
/// of this file), this fires.
#[test]
fn s_perf_15_d_rejects_detector_cellwide_arena_byte_change() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let (wide_sha, _, _, _, _) = capture_pre_rewrite_pins();
    assert_eq!(
        wide_sha, PINNED_PRE_S_PERF_15_D_DETECTOR_CELLWIDE_ARENA_SHA256,
        "S-PERF.15.d: DetectorCellWide arena SHA-256 drifted from pinned; the rewrite \
         changed bytes that some downstream stage reads. If this is a deliberate \
         conditional-write/slimming design (Direction a/c), update the pin in the same \
         commit AND verify cascade pin #5 catches the digest+casefile invariant."
    );
}

/// PIN 2 — detector compact-pack arena bytes byte-equal to
/// pinned. The 18-byte per-cell compact pack feeds the per-
/// stage digest tree; any drift surfaces as a per-stage root
/// drift downstream.
#[test]
fn s_perf_15_d_rejects_detector_compact_pack_arena_byte_change() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let (_, compact_pack_sha, _, _, _) = capture_pre_rewrite_pins();
    assert_eq!(
        compact_pack_sha, PINNED_PRE_S_PERF_15_D_DETECTOR_COMPACT_PACK_ARENA_SHA256,
        "S-PERF.15.d: detector compact-pack arena SHA-256 drifted from pinned; the \
         18-byte compact pack write path (Phase 3 of detector_motif_fused) shifted, \
         which will propagate to TreeSha256V1 + CompactDensorDigestV1 detector-root \
         drift."
    );
}

/// PIN 3 — TreeSha256V1 detector-stage root byte-equal to
/// pinned. Cross-validates against S-PERF.11 + S-PERF.15.a.
#[test]
fn s_perf_15_d_rejects_tree_sha256v1_detector_root_drift() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let (_, _, tree_root, _, _) = capture_pre_rewrite_pins();
    assert_eq!(
        tree_root, PINNED_PRE_S_PERF_15_D_TREE_SHA256V1_DETECTOR_ROOT,
        "S-PERF.15.d: TreeSha256V1 detector-stage root drifted from pinned; the rewrite \
         broke the same_mode_digest_root_law for TreeSha256V1 — S-PERF.11's byte-\
         identity contract VIOLATED."
    );
}

/// PIN 4 — CompactDensorDigestV1 detector-stage root byte-
/// equal to pinned. Cross-validates against S-PERF.14b +
/// S-PERF.15.a.
#[test]
fn s_perf_15_d_rejects_compact_densor_digest_v1_detector_root_drift() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let (_, _, _, compact_densor_root, _) = capture_pre_rewrite_pins();
    assert_eq!(
        compact_densor_root, PINNED_PRE_S_PERF_15_D_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT,
        "S-PERF.15.d: CompactDensorDigestV1 detector-stage root drifted from pinned; \
         the rewrite broke the same_mode_digest_root_law for CompactDensorDigestV1 — \
         S-PERF.14b's byte-identity contract VIOLATED."
    );
}

/// PIN 5 — consensus arena || casefile final cascade
/// byte-equal to pinned. Defense-in-depth: catches a slimming
/// bug where the field-audit mis-classified a hot field as
/// audit-only, by independently verifying that consensus bytes
/// AND the end-to-end casefile cascade are both byte-stable.
#[test]
fn s_perf_15_d_rejects_consensus_input_cascade_byte_change() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let (_, _, _, _, cascade_sha) = capture_pre_rewrite_pins();
    assert_eq!(
        cascade_sha, PINNED_PRE_S_PERF_15_D_CONSENSUS_INPUT_CASCADE_SHA256,
        "S-PERF.15.d: consensus_input||casefile cascade SHA-256 drifted from pinned; \
         either consensus_axis5_fused read different DetectorCellWide bytes than the \
         field-audit predicted (slimming bug) OR the casefile final hash shifted from \
         a deeper cascade regression. Investigate before sealing."
    );
}

// ---------------------------------------------------------------
// Positives — ensure the harness produces deterministic
// captures across two runs (panel-required for any harness
// that pins bytes derived from a GPU dispatch).
// ---------------------------------------------------------------

/// Capture-mode helper is deterministic across two runs (no
/// hidden race or non-determinism in the pre-rewrite path).
#[test]
fn s_perf_15_d_capture_is_deterministic_across_two_runs() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    let a = capture_pre_rewrite_pins();
    let b = capture_pre_rewrite_pins();
    assert_eq!(
        a.0, b.0,
        "wide arena SHA-256 must be deterministic across two runs"
    );
    assert_eq!(a.1, b.1, "compact-pack arena SHA-256 must be deterministic");
    assert_eq!(a.2, b.2, "TreeSha256V1 detector root must be deterministic");
    assert_eq!(
        a.3, b.3,
        "CompactDensorDigestV1 detector root must be deterministic"
    );
    assert_eq!(
        a.4, b.4,
        "consensus_input||casefile cascade SHA-256 must be deterministic"
    );
}

/// Cross-validation against S-PERF.15.a's pinned detector
/// roots. Both 15.a and 15.d pin the SAME TreeSha256V1 +
/// CompactDensorDigestV1 detector-stage roots; if they
/// disagree, one harness is broken or the codebase drifted
/// between 15.a-seal and 15.d-Step-0.
#[test]
fn s_perf_15_d_detector_roots_match_s_perf_15_a_pinned_roots() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    // S-PERF.15.a's pinned detector roots (module-scope
    // constants S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT +
    // S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT;
    // sealed at a47a8e9).
    let (_, _, tree_root, compact_densor_root, _) = capture_pre_rewrite_pins();
    assert_eq!(
        tree_root, S_PERF_15_A_TREE_SHA256V1_DETECTOR_ROOT,
        "S-PERF.15.d's TreeSha256V1 detector root must match S-PERF.15.a's pinned root \
         (both pin the same canonical 16x128 K=1 D64 detector-stage TreeSha256V1 root)"
    );
    assert_eq!(
        compact_densor_root, S_PERF_15_A_COMPACT_DENSOR_DIGEST_V1_DETECTOR_ROOT,
        "S-PERF.15.d's CompactDensorDigestV1 detector root must match S-PERF.15.a's \
         pinned root"
    );
}

/// Cross-validation against S-PERF.15.b's pinned casefile
/// final hash. The cascade pin #5 is SHA-256 over
/// (consensus_arena || casefile_final), and the casefile
/// component must equal the long-standing canonical pin.
#[test]
fn s_perf_15_d_cascade_includes_s_perf_15_b_pinned_casefile_final() {
    if std::env::var("DSFB_S_PERF_15_D_CAPTURE").is_ok() {
        return;
    }
    // S-PERF.15.b's pinned end-to-end case-file final hash
    // (module-scope constant S_PERF_15_B_CASEFILE_FINAL_HASH;
    // sealed at dc0feaf, preserved through 8b0db9b).
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
        case.final_case_file_hash, S_PERF_15_B_CASEFILE_FINAL_HASH,
        "S-PERF.15.d Step 0 casefile final hash must match the long-standing pinned \
         value (sealed at S-PERF.15.b dc0feaf and S-PERF.15.c 8b0db9b); divergence \
         indicates a regression in the pre-rewrite codebase, not an S-PERF.15.d concern"
    );
}
