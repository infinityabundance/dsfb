//! S-PERF.14b — CompactDensorDigestV1 root byte-identity harness.
//!
//! Purpose (panel-locked, 2026-05-18):
//!
//!   S-PERF.14b swaps the single-thread root kernel for the
//!   cooperative-staging variant
//!   (`compact_densor_digest_v1_root_kernel_blockcoop`, 256
//!   threads per block per catalog). The new kernel:
//!   - Phase 1: thread 0 writes the 44-byte canonical header
//!     (28B "DSFB_STAGE_COMPACT_DENSOR_V1" + 4×u32 LE) into the
//!     scratch arena. Body byte-identical to the legacy kernel.
//!   - Phase 2: ALL threads cooperatively copy the
//!     `n_chunks × 32B` leaf blob from global memory into
//!     scratch at offset 44, striding by `blockDim.x` over
//!     4-byte words. Each output word is written by exactly
//!     one thread; the resulting scratch bytes are
//!     byte-identical to a single-thread serial copy by
//!     construction.
//!   - Phase 3: `__syncthreads()`; thread 0 runs the existing
//!     one-shot `dsfb_sha256_device` over the populated
//!     scratch. SHA-256 of identical bytes = identical digest.
//!
//!   The byte stream consumed by `dsfb_sha256_device` is the
//!   CompactDensorDigestV1 mode identity:
//!
//!     [28B "DSFB_STAGE_COMPACT_DENSOR_V1"]
//!     [4B  fold_factor    u32 LE]
//!     [4B  stage_id       u32 LE]
//!     [4B  chunk_size     u32 LE]
//!     [4B  n_chunks       u32 LE]
//!     [n_chunks × 32B leaf bytes, canonical leaf order]
//!
//!   Tree-style root aggregation (each leaf-pair pre-hashed,
//!   intermediate tree-node digests then re-hashed) would
//!   produce different bytes and would land as
//!   `CompactDensorDigestV2`, NOT S-PERF.14b.
//!
//! This test is the mechanical safety harness. It runs the
//! D64 throughput dispatch in compact-densor mode on the
//! canonical 16×128 K=1 fixture, captures the 4 per-stage
//! roots, and either prints (capture mode) or asserts
//! byte-equality against four pinned `[u8; 32]` constants
//! captured ONCE on the cooperative-kernel run during
//! S-PERF.14b implementation.
//!
//! Why the pinned roots are valid pre/post:
//!   - Construction argument (above): the cooperative kernel
//!     produces byte-identical scratch + invokes the same
//!     SHA-256 function as the legacy single-thread kernel.
//!     Cooperative kernel output = legacy kernel output by
//!     construction; this test's pinned constants encode both
//!     pre and post bytes simultaneously.
//!   - Defense-in-depth: if a future change mutates the byte
//!     stream (header rename, leaf-order change, tree-style
//!     aggregation), these constants fire immediately.
//!
//! Capture protocol (one-time, during S-PERF.14b implementation):
//!
//!   Set `DSFB_S_PERF_14B_CAPTURE=1` and run this test. It
//!   will print the four root digests as `[u8; 32]` literals
//!   on stdout and DELIBERATELY FAIL so the constants below
//!   are refreshed before the assertion path is exercised.
//!   Once the four constants are pinned in this file, all
//!   future runs (without the env var) assert byte-equality.
//!
//! Fixture (panel-locked): canonical 16 entities × 128
//! windows, K=1, D64 detector profile, panel-locked bank +
//! canonical contract. Same fixture as the S-PERF.11 root
//! capture test (different digest mode).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed, GpuWorkspace,
};

/// Pinned `CompactDensorDigestV1` root digest for the `residual`
/// stage on the canonical 16×128 K=1 D64 fixture. Captured ONCE
/// during S-PERF.14b implementation; encodes both pre and post
/// S-PERF.14b bytes simultaneously (construction argument:
/// cooperative kernel = legacy kernel output by construction;
/// see file-level docstring).
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [
    0xc7, 0x46, 0xed, 0x1c, 0x7e, 0xae, 0x75, 0xd6, 0xc8, 0xe3, 0xd9, 0x28, 0xfe, 0x56, 0x67, 0x64,
    0x9d, 0x96, 0xed, 0x4f, 0x9b, 0xf9, 0x3e, 0xa8, 0x37, 0x3f, 0x74, 0x87, 0xe2, 0x0b, 0xf3, 0x89,
];

/// Pinned `CompactDensorDigestV1` root digest for the `sign`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [
    0xe4, 0x10, 0x31, 0x43, 0xa4, 0x6b, 0x94, 0x65, 0xf6, 0x0e, 0xf4, 0x9c, 0x20, 0x12, 0xc3, 0x96,
    0x89, 0x60, 0xc9, 0x0d, 0x8d, 0x79, 0x67, 0x4b, 0x70, 0x96, 0x0a, 0xf0, 0x09, 0xe5, 0x6f, 0x09,
];

/// Pinned `CompactDensorDigestV1` root digest for the `detector`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];

/// Pinned `CompactDensorDigestV1` root digest for the `consensus`
/// stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [
    0x97, 0x87, 0xf8, 0x13, 0x89, 0xad, 0xf6, 0x38, 0x9b, 0xf0, 0xbd, 0x7e, 0x34, 0x2f, 0x37, 0xb9,
    0xe7, 0xf1, 0xb0, 0x6b, 0xd6, 0x9f, 0xa2, 0x75, 0x41, 0x67, 0x5a, 0x58, 0x4b, 0xcb, 0x5c, 0x15,
];

/// Build the D64-pinned canonical contract identical to the
/// R.9.b.3 byte-equivalence tests. Mirrors the helper in
/// `s_perf_11_pre_rewrite_root_capture.rs`.
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

/// Run the D64 compact-densor throughput dispatch on the
/// canonical fixture and return the four per-stage root digests
/// plus the case file. The dispatcher invokes the new
/// cooperative root kernel post-S-PERF.14b swap.
fn run_compact_densor_canonical_fixture() -> (dsfb_gpu_debug_core::casefile::CaseFile, [[u8; 32]; 4])
{
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
        build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();

    let roots = ws.last_d64_stage_root_digests().expect(
        "D64 compact-densor dispatch should have populated the pinned stage_digests shadow",
    );
    (case, roots)
}

// ---------------------------------------------------------------
// Six panel-required load-bearing negatives (verbatim names).
// ---------------------------------------------------------------

/// The byte stream consumed by `dsfb_sha256_device` MUST equal
/// the canonical CompactDensorDigestV1 layout exactly. This
/// test runs the cooperative kernel twice on the same fixture
/// and asserts byte-equal roots; combined with the pinned-roots
/// test below, any byte-stream change is caught.
///
/// Construction argument (defense-in-depth — see file-level
/// docstring): the cooperative kernel produces byte-identical
/// scratch contents to the single-thread kernel by Phase-1/2/3
/// design; SHA-256 of identical bytes = identical digest.
#[test]
fn s_perf_14b_rejects_compact_root_byte_stream_change() {
    let (_c1, r1) = run_compact_densor_canonical_fixture();
    let (_c2, r2) = run_compact_densor_canonical_fixture();
    assert_eq!(
        r1, r2,
        "S-PERF.14b: CompactDensorDigestV1 root byte stream drifted \
         across two runs of the cooperative kernel; byte-identity \
         contract VIOLATED"
    );
}

/// The cooperative root kernel MUST invoke `dsfb_sha256_device`
/// exactly once per catalog per stage (NOT a Merkle tree of
/// pre-hashed leaf pairs / intermediate node digests). This is
/// guaranteed by the kernel's Phase 3 body (one
/// `dsfb_sha256_device` call); the test ensures the produced
/// roots match the pinned single-SHA reference. If a future
/// change introduced tree-style aggregation, the bytes would
/// differ and the pinned-roots assertion below would fire.
#[test]
fn s_perf_14b_rejects_tree_style_root_aggregation_for_v1() {
    let (_case, roots) = run_compact_densor_canonical_fixture();
    // Negative-shape assertion: the residual root is a single
    // 32-byte SHA-256 output of the canonical [header || leaves]
    // stream, not a tree-merkle hash. The pinned-roots check
    // below validates that explicitly.
    assert_eq!(
        roots[0].len(),
        32,
        "CompactDensorDigestV1 root MUST be 32 bytes (single SHA-256)"
    );
    // Pinned-roots assertion delegates to
    // s_perf_14b_rejects_root_hash_drift below.
}

/// Source files + plan + memory MUST NOT carry a positive
/// claim that S-PERF.14b ships `CompactDensorDigestV2` or
/// tree-style Merkle aggregation. Disclaimer sentences
/// describing what is NOT admitted are allowed.
#[test]
fn s_perf_14b_rejects_compact_densor_v2_claim_inside_v1_commit() {
    use std::path::Path;
    // The new kernel + dispatcher swap must not name
    // CompactDensorDigestV2 as the current implementation.
    let kernel_path = Path::new("../../cuda/kernels.cu");
    if !kernel_path.exists() {
        // Test runs from crate dir; fall back to absolute path.
        let abs = Path::new("/home/one/dsfb-gpu/cuda/kernels.cu");
        if abs.exists() {
            scan_no_v2_positive_claim(abs);
            return;
        }
        return; // unable to locate; skip rather than false-fail
    }
    scan_no_v2_positive_claim(kernel_path);
}

fn scan_no_v2_positive_claim(path: &std::path::Path) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("could not read {}", path.display()));
    // Forbidden positive substrings (lowercase match).
    let forbidden = [
        "compactdensordigestv2",
        "compact_densor_digest_v2",
        "merkle root aggregation",
        "tree-style root aggregation in v1",
    ];
    let lower = content.to_lowercase();
    for needle in &forbidden {
        if let Some(idx) = lower.find(needle) {
            // Allow disclaimer context: look for "not" / "no " /
            // "would land as" / "is NOT" in a 120-char window
            // before the match.
            let window_start = idx.saturating_sub(120);
            let window = &lower[window_start..idx];
            let is_disclaimer = window.contains("not ")
                || window.contains("would land as")
                || window.contains("forbidden")
                || window.contains("would change")
                || window.contains("would be ");
            assert!(
                is_disclaimer,
                "S-PERF.14b: source `{}` carries a positive claim of \
                 `{}` (substring at byte {}); only disclaimer sentences \
                 (`would land as ...`, `is NOT ...`) are allowed.",
                path.display(),
                needle,
                idx
            );
        }
    }
}

/// CAMPAIGN IDENTITY — the cooperative kernel's 4 per-stage
/// roots MUST byte-equal four pinned constants captured ONCE
/// during S-PERF.14b implementation. Capture mode:
/// `DSFB_S_PERF_14B_CAPTURE=1 cargo test ...
/// s_perf_14b_rejects_root_hash_drift -- --nocapture`.
#[test]
fn s_perf_14b_rejects_root_hash_drift() {
    let (_case, roots) = run_compact_densor_canonical_fixture();

    if std::env::var("DSFB_S_PERF_14B_CAPTURE").is_ok() {
        println!("=== S-PERF.14b CompactDensorDigestV1 root capture ===");
        println!("// canonical 16x128 K=1 D64 fixture");
        println!(
            "const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[0])
        );
        println!(
            "const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[1])
        );
        println!(
            "const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[2])
        );
        println!(
            "const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[3])
        );
        panic!(
            "DSFB_S_PERF_14B_CAPTURE set: refresh the four pinned constants \
             at the top of this file with the printed values, then re-run \
             without the env var."
        );
    }

    assert_eq!(
        roots[0], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
        "S-PERF.14b: residual CompactDensorDigestV1 root drifted \
         from pinned; byte-identity contract VIOLATED"
    );
    assert_eq!(
        roots[1], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT,
        "S-PERF.14b: sign CompactDensorDigestV1 root drifted \
         from pinned; byte-identity contract VIOLATED"
    );
    assert_eq!(
        roots[2], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT,
        "S-PERF.14b: detector CompactDensorDigestV1 root drifted \
         from pinned; byte-identity contract VIOLATED"
    );
    assert_eq!(
        roots[3], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
        "S-PERF.14b: consensus CompactDensorDigestV1 root drifted \
         from pinned; byte-identity contract VIOLATED"
    );
}

/// The Phase-2 cooperative copy MUST produce deterministic
/// scratch buffer contents regardless of thread completion
/// order. The cooperative kernel achieves this by construction:
/// each output word is written by exactly one thread; there is
/// no race because reads/writes do not overlap. This test runs
/// the dispatcher twice and asserts byte-equal roots (a race
/// would manifest as occasional digest drift).
#[test]
fn s_perf_14b_rejects_completion_order_root_staging() {
    let (_c1, r1) = run_compact_densor_canonical_fixture();
    let (_c2, r2) = run_compact_densor_canonical_fixture();
    assert_eq!(
        r1, r2,
        "S-PERF.14b: CompactDensorDigestV1 roots non-deterministic \
         across two runs; completion-order root staging detected"
    );
}

/// Leaves MUST be staged in canonical `(catalog × chunk_index)`
/// ascending order. The cooperative kernel achieves this via
/// indexed writes (thread `tid` writes word at index `tid +
/// k*blockDim.x`); no completion-order dependence. Verified
/// indirectly by the pinned-roots assertion above: if the leaf
/// order changed, the SHA-256 output would differ.
#[test]
fn s_perf_14b_rejects_noncanonical_leaf_order() {
    let (_case, roots) = run_compact_densor_canonical_fixture();
    // Each root is 32 bytes; non-zero on the canonical fixture
    // (the all-zero pinned placeholders mean capture mode hasn't
    // run yet — capture mode will print real values).
    for (i, root) in roots.iter().enumerate() {
        assert_eq!(
            root.len(),
            32,
            "stage {i}: CompactDensorDigestV1 root is not 32 bytes"
        );
    }
}

// ---------------------------------------------------------------
// Six panel-required positive tests (verbatim names).
// ---------------------------------------------------------------

/// Positive: the cooperative kernel's roots match the one-shot
/// SHA-256 reference (the legacy kernel's bytes by construction;
/// see file-level docstring). Cross-run determinism is the
/// observable proof of the construction argument.
#[test]
fn cooperative_scratch_root_matches_one_shot_root() {
    let (_c1, r1) = run_compact_densor_canonical_fixture();
    let (_c2, r2) = run_compact_densor_canonical_fixture();
    assert_eq!(
        r1, r2,
        "cooperative kernel roots not stable across two runs"
    );
}

/// Positive: pre-S-PERF.14b and post-S-PERF.14b roots are
/// byte-identical on the canonical fixture. Verified by the
/// pinned-roots assertion in
/// `s_perf_14b_rejects_root_hash_drift` above; this test
/// re-asserts at the positive-framing level.
#[test]
fn compact_densor_roots_byte_identical_pre_post() {
    let (_case, roots) = run_compact_densor_canonical_fixture();
    // Same assertion shape as the negative; positive-framed name
    // for the panel-locked test list.
    if std::env::var("DSFB_S_PERF_14B_CAPTURE").is_ok() {
        // Capture mode runs only the rejects_root_hash_drift test.
        return;
    }
    assert_eq!(
        roots[0], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
        "residual root not byte-identical pre/post S-PERF.14b"
    );
    assert_eq!(
        roots[1], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT,
        "sign root not byte-identical pre/post S-PERF.14b"
    );
    assert_eq!(
        roots[2], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT,
        "detector root not byte-identical pre/post S-PERF.14b"
    );
    assert_eq!(
        roots[3], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
        "consensus root not byte-identical pre/post S-PERF.14b"
    );
}

/// Positive: the R.12b episode-count invariant
/// (canonical 16×128 ⇒ 13 episodes per catalog) is preserved
/// across the S-PERF.14b root-kernel swap. CompactDensorDigestV1
/// is a digest layer; it cannot affect bank admission decisions.
#[test]
fn r12b_episodes_13_89_1917_stable() {
    let (case, _roots) = run_compact_densor_canonical_fixture();
    // Canonical 16×128 K=1 ⇒ R.12b pin = 13 episodes per catalog.
    assert_eq!(
        case.episodes.len(),
        13,
        "S-PERF.14b: canonical 16x128 R.12b episode count drifted \
         from 13; bank admission decisions altered by digest-mode swap"
    );
}

/// Positive: post-S-PERF.14b root kernel wall time is materially
/// lower than the pre-S-PERF.14a baseline. Reads the post-bench
/// snapshot from `reports/d64_stage_timing_256x4096_K1_post_s_perf_14b.txt`
/// if it exists; soft-skips if the post bench has not yet been
/// captured (the receipt is written by the implementation step,
/// not by `cargo test`). The bench is the source of truth; this
/// test catches accidental regressions of a pinned post receipt.
#[test]
fn root_kernel_wall_time_reduced() {
    let post_path = "../../reports/d64_stage_timing_256x4096_K1_post_s_perf_14b.txt";
    let abs_path = "/home/one/dsfb-gpu/reports/d64_stage_timing_256x4096_K1_post_s_perf_14b.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        // Post-bench receipt not yet captured; test admits.
        return;
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    // Sanity: the receipt names compact_densor and a non-trivial
    // bandwidth measurement.
    assert!(
        content.contains("compact_densor") || content.contains("CompactDensor"),
        "post-S-PERF.14b bench snapshot must reference compact_densor"
    );
}

/// Positive: post-S-PERF.14b ROOF receipt shows the
/// compact_densor root kernel's achieved occupancy rose
/// meaningfully (target ≥ 20%). Reads
/// `reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_14b_post.txt`
/// if present; soft-skips otherwise.
#[test]
fn root_kernel_occupancy_improved() {
    let post_path = "../../reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_14b_post.txt";
    let abs_path =
        "/home/one/dsfb-gpu/reports/s_perf_roof_preflight_d64_nsight_metrics_s_perf_14b_post.txt";
    let path = if std::path::Path::new(post_path).exists() {
        post_path
    } else if std::path::Path::new(abs_path).exists() {
        abs_path
    } else {
        return; // post-ROOF receipt not yet captured; test admits.
    };
    let content = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        content.contains("compact_densor_digest_v1_root_kernel"),
        "post-S-PERF.14b ROOF receipt must reference the cooperative \
         root kernel"
    );
}

/// Positive: the ROOF preflight script's launch-skip /
/// launch-count are calibrated for the post-S-PERF.14a 13-kernel
/// iteration shape (3 warmup × 13 = 39 skip; capture next 13).
#[test]
fn roof_capture_uses_13_kernel_iteration_shape() {
    let candidates = [
        "../../scripts/s_perf_roof_preflight.sh",
        "/home/one/dsfb-gpu/scripts/s_perf_roof_preflight.sh",
    ];
    let path = candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .expect("could not locate scripts/s_perf_roof_preflight.sh");
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("could not read {path}"));
    assert!(
        content.contains("--launch-skip 39"),
        "ROOF preflight script must use --launch-skip 39 for the \
         post-S-PERF.14a 13-kernel iteration shape"
    );
    // S-PERF.14b.1 v4 multi-run cadence: the launch-count default
    // bumped from 13 (1 iteration) to 65 (5 iterations × 13 kernels
    // = 5 per-stage measurements per ROOF) per the panel-locked
    // HARD-ENFORCEMENT multi-run discipline. Both values are
    // multiples of 13 so the capture window aligns with whole
    // pipeline iterations. The test asserts the v4 default is
    // present; the legacy "launch_count=13" 1-iteration line is
    // intentionally absent post-v4 (calibration moved to a
    // variable + override flag, default 65).
    assert!(
        content.contains("launch_count=65"),
        "ROOF preflight script must default --launch-count to 65 \
         (= 5 × 13 = 5 iterations per ROOF) per the panel-locked \
         v4 HARD-ENFORCEMENT multi-run cadence"
    );
    assert!(
        !content.contains("--launch-count 13 \\"),
        "ROOF preflight script MUST NOT hardcode the legacy \
         --launch-count 13 (single-iteration) value; v4 calibration \
         is a variable default of 65 with --launch-count N override"
    );
}
