//! S-PERF.14b.1 Step 0 — streaming SHA-256 Path 1b
//! `compact_densor_digest_v1_root_kernel_blockcoop`
//! byte-identity harness.
//!
//! **Purpose (panel-locked, 2026-05-18 post-S-PERF.15.d seal
//! at `6233622`)**:
//!
//!   S-PERF.14b Path 1a (sealed at `e1dcf54`) cut the root
//!   kernel 2.38 → 0.82 ms via cooperative scratch staging
//!   while preserving the exact `CompactDensorDigestV1` byte
//!   stream. Post-S-PERF.15.d the root kernel is the largest
//!   remaining device wall (~925 µs per invocation × 4
//!   invocations per dispatch ≈ 3.7 ms total). S-PERF.14b.1
//!   replaces the scratch-staging path with streaming
//!   SHA-256 (init/update/finalize) that absorbs the 44 B
//!   header + n_chunks × 32 B leaves directly from global
//!   memory — eliminating the scratch round-trip while
//!   preserving the EXACT same byte stream (same final root
//!   bytes by construction).
//!
//! **Panel-required Step 0 (4 pinned constants — match
//! existing S-PERF.14b pins by construction; the codebase at
//! HEAD `6233622` IS post-Path-1a-seal)**:
//!
//!   1. `PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_RESIDUAL_ROOT`
//!   2. `PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_SIGN_ROOT`
//!   3. `PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_DETECTOR_ROOT`
//!   4. `PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_CONSENSUS_ROOT`
//!
//! These 4 pins MUST byte-equal the existing
//! `PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_*_ROOT` constants
//! (cross-validated by the
//! `s_perf_14b_1_roots_match_s_perf_14b_pinned_roots` test
//! below). Post-S-PERF.14b.1 the streaming path MUST produce
//! the same 4 roots — proving the byte stream consumed by
//! SHA-256 is unchanged.
//!
//! **Capture protocol** (panel-required Step 0; one-time,
//! BEFORE the streaming SHA-256 kernel surgery):
//!
//!   ```
//!   DSFB_S_PERF_14B_1_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//!     --features cuda --release \
//!     --test s_perf_14b_1_streaming_sha_byte_identity \
//!     -- --nocapture s_perf_14b_1_capture_mode_prints_all_four_pins
//!   ```
//!
//!   The capture-mode test runs the CURRENT
//!   `compact_densor_digest_v1_root_kernel_blockcoop` (Path
//!   1a cooperative-staging) on the canonical 16 × 128 K=1
//!   D64 fixture, computes the 4 pinned digests, prints them
//!   as `[u8; 32]` literals on stdout, and DELIBERATELY FAILS
//!   so the constants below are refreshed BEFORE the
//!   assertion path is exercised.
//!
//! **CompactDensorDigestV1 byte-stream metadata (Step 0
//! audit deliverable; sourced from `cuda/kernels.cu` at HEAD
//! `6233622`)**:
//!
//! ```text
//! Header (44 bytes):
//!   [ 0..28) "DSFB_STAGE_COMPACT_DENSOR_V1"  (28 B domain separator)
//!   [28..32) fold_factor    u32 LE = 256 (panel-locked S-PERF.12 FOLD_FACTOR)
//!   [32..36) stage_id       u32 LE ∈ {0=residual, 1=sign, 2=detector, 3=consensus}
//!   [36..40) chunk_size     u32 LE (per-stage; from contract / launch)
//!   [40..44) n_chunks       u32 LE (per-stage; from launch)
//!
//! Body (n_chunks × 32 bytes):
//!   leaf[0]  : 32 B SHA-256 of chunk 0
//!   leaf[1]  : 32 B SHA-256 of chunk 1
//!   ...
//!   leaf[n-1]: 32 B SHA-256 of chunk n-1
//!
//! Total: 44 + n_chunks × 32 bytes per root invocation
//! Canonical leaf ordering: catalog-major × chunk-index-minor
//! ```
//!
//! At canonical 256 × 4096 K=1 D64, the per-stage leaf counts
//! vary by stage payload size. The streaming SHA-256 path
//! MUST consume the bytes in this exact order to produce
//! byte-identical roots.
//!
//! Fixture (panel-locked): canonical 16 entities × 128
//! windows, K=1, D64 detector profile, panel-locked bank +
//! canonical contract. Same fixture as the S-PERF.11 /
//! S-PERF.14b / S-PERF.15.a/b/c/d root-capture tests.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed,
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed, GpuWorkspace,
};

/// PIN 5 — end-to-end case-file final hash on the canonical
/// 16x128 K=1 D64 fixture (TreeSha256V1 dispatch). Pinned by
/// the long-standing S-PERF.15.b casefile pin (sealed at
/// dc0feaf; preserved through 8b0db9b → 6233622). The
/// S-PERF.14b.1 streaming SHA-256 rewrite touches the
/// CompactDensorDigestV1 root path only; the TreeSha256V1
/// path + bank cascade + case-file emission are unaffected,
/// so this pin MUST stay byte-identical post-rewrite as
/// downstream-cascade defense-in-depth.
const PINNED_PRE_S_PERF_14B_1_CASEFILE_FINAL_HASH: [u8; 32] = [
    0x98, 0xd0, 0x69, 0x67, 0x01, 0xf7, 0x6a, 0x81, 0xb3, 0x8e, 0x18, 0xe0, 0xf5, 0xb0, 0x2e, 0x86,
    0x75, 0x3f, 0xa0, 0xfd, 0xdb, 0xd1, 0x58, 0xd2, 0xe2, 0xd8, 0xb8, 0xb4, 0x00, 0x26, 0xf3, 0xa6,
];

// ---------------------------------------------------------------
// Panel-required Step 0 pinned constants (4).
//
// CAPTURE-MODE PROTOCOL (one-time, BEFORE S-PERF.14b.1
// streaming-SHA kernel surgery):
//
//   DSFB_S_PERF_14B_1_CAPTURE=1 cargo test -p dsfb-gpu-debug-demo \
//     --features cuda --release \
//     --test s_perf_14b_1_streaming_sha_byte_identity \
//     -- --nocapture s_perf_14b_1_capture_mode_prints_all_four_pins
//
// Capture-mode run executes the CURRENT
// compact_densor_digest_v1_root_kernel_blockcoop (Path 1a
// cooperative-staging, sealed at e1dcf54) on the canonical
// 16x128 K=1 D64 fixture, computes 4 pinned root digests,
// prints them as `[u8; 32]` literals on stdout, and
// DELIBERATELY FAILS so the constants below are refreshed
// BEFORE the assertion path is exercised.
// ---------------------------------------------------------------

/// Pinned CompactDensorDigestV1 root digest for the `residual`
/// stage on the canonical 16×128 K=1 D64 fixture. The
/// S-PERF.14b.1 streaming SHA-256 rewrite MUST produce a
/// byte-identical 32-byte root because the byte stream
/// consumed by SHA-256 (44 B header + n_chunks × 32 B leaves
/// in canonical order) is unchanged — only the staging
/// mechanism (scratch buffer → streaming init/update/finalize)
/// changes.
const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [
    0xc7, 0x46, 0xed, 0x1c, 0x7e, 0xae, 0x75, 0xd6, 0xc8, 0xe3, 0xd9, 0x28, 0xfe, 0x56, 0x67, 0x64,
    0x9d, 0x96, 0xed, 0x4f, 0x9b, 0xf9, 0x3e, 0xa8, 0x37, 0x3f, 0x74, 0x87, 0xe2, 0x0b, 0xf3, 0x89,
];

/// Pinned CompactDensorDigestV1 root digest for the `sign`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [
    0xe4, 0x10, 0x31, 0x43, 0xa4, 0x6b, 0x94, 0x65, 0xf6, 0x0e, 0xf4, 0x9c, 0x20, 0x12, 0xc3, 0x96,
    0x89, 0x60, 0xc9, 0x0d, 0x8d, 0x79, 0x67, 0x4b, 0x70, 0x96, 0x0a, 0xf0, 0x09, 0xe5, 0x6f, 0x09,
];

/// Pinned CompactDensorDigestV1 root digest for the `detector`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

/// Pinned CompactDensorDigestV1 root digest for the `consensus`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [
    0x97, 0x87, 0xf8, 0x13, 0x89, 0xad, 0xf6, 0x38, 0x9b, 0xf0, 0xbd, 0x7e, 0x34, 0x2f, 0x37, 0xb9,
    0xe7, 0xf1, 0xb0, 0x6b, 0xd6, 0x9f, 0xa2, 0x75, 0x41, 0x67, 0x5a, 0x58, 0x4b, 0xcb, 0x5c, 0x15,
];

// ---------------------------------------------------------------
// S-PERF.14b's pinned root constants (sealed at e1dcf54;
// preserved through 8b0db9b → 6233622). The S-PERF.14b.1
// Step 0 pins MUST byte-equal these by construction: the
// codebase at HEAD `6233622` is post-Path-1a-seal, so the
// Path 1a roots ARE the post-S-PERF.14b roots.
//
// Hoisted to module scope so clippy's items_after_statements
// lint stays clean.
// ---------------------------------------------------------------

/// S-PERF.14b's pinned CompactDensorDigestV1 residual-stage
/// root (sealed at `e1dcf54`).
const S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [
    0xc7, 0x46, 0xed, 0x1c, 0x7e, 0xae, 0x75, 0xd6, 0xc8, 0xe3, 0xd9, 0x28, 0xfe, 0x56, 0x67, 0x64,
    0x9d, 0x96, 0xed, 0x4f, 0x9b, 0xf9, 0x3e, 0xa8, 0x37, 0x3f, 0x74, 0x87, 0xe2, 0x0b, 0xf3, 0x89,
];

/// S-PERF.14b's pinned CompactDensorDigestV1 sign-stage root.
const S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [
    0xe4, 0x10, 0x31, 0x43, 0xa4, 0x6b, 0x94, 0x65, 0xf6, 0x0e, 0xf4, 0x9c, 0x20, 0x12, 0xc3, 0x96,
    0x89, 0x60, 0xc9, 0x0d, 0x8d, 0x79, 0x67, 0x4b, 0x70, 0x96, 0x0a, 0xf0, 0x09, 0xe5, 0x6f, 0x09,
];

/// S-PERF.14b's pinned CompactDensorDigestV1 detector-stage
/// root (also matches S-PERF.15.a/d pinned detector root).
const S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

/// S-PERF.14b's pinned CompactDensorDigestV1 consensus-stage
/// root.
const S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [
    0x97, 0x87, 0xf8, 0x13, 0x89, 0xad, 0xf6, 0x38, 0x9b, 0xf0, 0xbd, 0x7e, 0x34, 0x2f, 0x37, 0xb9,
    0xe7, 0xf1, 0xb0, 0x6b, 0xd6, 0x9f, 0xa2, 0x75, 0x41, 0x67, 0x5a, 0x58, 0x4b, 0xcb, 0x5c, 0x15,
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

/// Capture the 4 CompactDensorDigestV1 stage roots produced
/// by the CURRENT codebase (Path 1a cooperative-staging at
/// HEAD `6233622`). Used for both Step 0 capture-mode AND the
/// post-rewrite byte-identity verification.
fn capture_compact_densor_roots() -> [[u8; 32]; 4] {
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
    let (_case, _stage_timings, _host_timings) =
        build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();
    ws.last_d64_stage_root_digests()
        .expect("D64 compact-densor dispatch should populate stage digests")
}

// ---------------------------------------------------------------
// Step 0 capture-mode test (single panic point for one
// comprehensive capture run).
// ---------------------------------------------------------------

#[test]
fn s_perf_14b_1_capture_mode_prints_all_four_pins() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_err() {
        return;
    }
    let roots = capture_compact_densor_roots();
    println!("=== S-PERF.14b.1 Step 0 pre-rewrite root-capture (canonical 16x128 K=1 D64) ===");
    println!(
        "const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(&roots[0])
    );
    println!(
        "const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(&roots[1])
    );
    println!(
        "const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(&roots[2])
    );
    println!(
        "const PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [\n    {}\n];",
        hex_u8_array(&roots[3])
    );
    panic!(
        "DSFB_S_PERF_14B_1_CAPTURE set: refresh the four pinned constants at the top of \
         this file with the printed values, then re-run without the env var."
    );
}

// ---------------------------------------------------------------
// 4 panel-required Step 0 pre-rewrite-pin negatives.
// ---------------------------------------------------------------

/// PIN 1 — CompactDensorDigestV1 residual-stage root
/// byte-equal to pinned. If the S-PERF.14b.1 streaming
/// SHA-256 rewrite consumed bytes in a different order or
/// dropped/duplicated any bytes from the canonical
/// (header || leaves) stream, this fires.
#[test]
fn s_perf_14b_1_rejects_compact_densor_residual_root_drift() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let roots = capture_compact_densor_roots();
    assert_eq!(
        roots[0], PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_RESIDUAL_ROOT,
        "S-PERF.14b.1: CompactDensorDigestV1 residual-stage root drifted from pinned; \
         the streaming SHA-256 rewrite changed the byte stream consumed by SHA-256 \
         (header layout, fold_factor, stage_id, chunk_size, n_chunks, or leaf order) — \
         CompactDensorDigestV1 byte-stream contract VIOLATED at the residual layer"
    );
}

/// PIN 2 — CompactDensorDigestV1 sign-stage root byte-equal
/// to pinned.
#[test]
fn s_perf_14b_1_rejects_compact_densor_sign_root_drift() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let roots = capture_compact_densor_roots();
    assert_eq!(
        roots[1], PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_SIGN_ROOT,
        "S-PERF.14b.1: CompactDensorDigestV1 sign-stage root drifted from pinned; \
         streaming SHA-256 byte-stream contract VIOLATED at the sign layer"
    );
}

/// PIN 3 — CompactDensorDigestV1 detector-stage root
/// byte-equal to pinned. Cross-validates against S-PERF.14b
/// + S-PERF.15.a/d's detector-root pins by construction.
#[test]
fn s_perf_14b_1_rejects_compact_densor_detector_root_drift() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let roots = capture_compact_densor_roots();
    assert_eq!(
        roots[2], PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_DETECTOR_ROOT,
        "S-PERF.14b.1: CompactDensorDigestV1 detector-stage root drifted from pinned; \
         streaming SHA-256 byte-stream contract VIOLATED at the detector layer"
    );
}

/// PIN 4 — CompactDensorDigestV1 consensus-stage root
/// byte-equal to pinned.
#[test]
fn s_perf_14b_1_rejects_compact_densor_consensus_root_drift() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let roots = capture_compact_densor_roots();
    assert_eq!(
        roots[3], PINNED_PRE_S_PERF_14B_1_COMPACT_DENSOR_CONSENSUS_ROOT,
        "S-PERF.14b.1: CompactDensorDigestV1 consensus-stage root drifted from pinned; \
         streaming SHA-256 byte-stream contract VIOLATED at the consensus layer"
    );
}

// ---------------------------------------------------------------
// Positives — cross-validation + determinism.
// ---------------------------------------------------------------

/// Cross-validation: the 4 freshly-captured CompactDensorDigestV1
/// stage roots MUST byte-equal the 4 S-PERF.14b pinned root
/// constants (sealed at `e1dcf54`). The codebase at HEAD
/// `6233622` is post-Path-1a-seal, so the Path 1a roots ARE
/// the post-S-PERF.14b roots; this test catches any silent
/// drift between `e1dcf54` and `6233622`.
#[test]
fn s_perf_14b_1_roots_match_s_perf_14b_pinned_roots() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let roots = capture_compact_densor_roots();
    assert_eq!(
        roots[0], S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
        "S-PERF.14b.1's CompactDensorDigestV1 residual root must match S-PERF.14b's \
         pinned root (both pin the same canonical 16x128 K=1 D64 fixture)"
    );
    assert_eq!(
        roots[1], S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT,
        "S-PERF.14b.1's CompactDensorDigestV1 sign root must match S-PERF.14b's pinned root"
    );
    assert_eq!(
        roots[2], S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT,
        "S-PERF.14b.1's CompactDensorDigestV1 detector root must match S-PERF.14b's pinned root"
    );
    assert_eq!(
        roots[3], S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
        "S-PERF.14b.1's CompactDensorDigestV1 consensus root must match S-PERF.14b's pinned root"
    );
}

/// Capture is deterministic across two runs (no hidden race
/// or non-determinism in the pre-rewrite path).
#[test]
fn s_perf_14b_1_capture_is_deterministic_across_two_runs() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
    let a = capture_compact_densor_roots();
    let b = capture_compact_densor_roots();
    assert_eq!(
        a, b,
        "4 CompactDensorDigestV1 stage roots must be deterministic across two runs"
    );
}

/// PIN 5 — end-to-end case-file final hash on the canonical
/// 16x128 K=1 D64 fixture (TreeSha256V1 dispatch). The S-PERF.14b.1
/// streaming SHA-256 rewrite touches the CompactDensorDigestV1
/// root path only; the TreeSha256V1 path + bank cascade +
/// case-file emission MUST stay byte-identical. Downstream-cascade
/// defense-in-depth: even if the streaming kernel were used in
/// production (which it isn't post-revert), this pin catches
/// any cascade regression that propagates beyond the digest
/// path.
#[test]
fn s_perf_14b_1_rejects_casefile_final_hash_change() {
    if std::env::var("DSFB_S_PERF_14B_1_CAPTURE").is_ok() {
        return;
    }
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
        case.final_case_file_hash, PINNED_PRE_S_PERF_14B_1_CASEFILE_FINAL_HASH,
        "S-PERF.14b.1: end-to-end case-file final hash drifted from pinned; downstream \
         cascade broken somewhere beyond the digest path"
    );
}
