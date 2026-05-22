//! S-PERF.11 — pre-rewrite TreeSha256V1 root-equivalence capture.
//!
//! Purpose (panel-locked):
//!
//!   The S-PERF.11 commit batches `tree_digest_leaf_kernel` work
//!   into `tree_digest_leaf_kernel_v2` (32 chunks per block, one
//!   chunk per thread within a warp) WITHOUT changing the per-chunk
//!   SHA-256 input bytes. Per-chunk inputs unchanged → per-chunk
//!   leaf digests byte-identical → per-stage TreeSha256V1 root
//!   digests byte-identical → S-PERF.10's
//!   `same_mode_digest_root_law` is satisfied.
//!
//!   This test is the mechanical safety harness: it runs the D64
//!   throughput dispatch on the canonical 16×128 K=1 fixture and
//!   asserts that the four `TreeSha256V1` stage-root digests
//!   (residual / sign / detector / consensus) byte-equal four
//!   pinned `[u8; 32]` constants. Those four constants were
//!   captured ONCE on the pre-rewrite kernel; if the kernel rewrite
//!   ever changes the bytes hashed (not just the launch geometry),
//!   this test fires immediately and the commit is panel-locked
//!   inadmissible.
//!
//!   The test is also a backstop against silent rebaselining: a
//!   future commit that intentionally changes the digest mode
//!   (CompactDensorDigestV1 in S-PERF.12) does so under a separate
//!   digest-mode identifier per S-PERF.10's
//!   `digest_mode_non_aliasing_law`; this test continues to assert
//!   on TreeSha256V1 specifically.
//!
//! Capture protocol (one-time, during S-PERF.11 implementation):
//!
//!   Set DSFB_S_PERF_11_CAPTURE=1 in the environment and run this
//!   test. The test will print the four root digests as `[u8; 32]`
//!   literals on stdout and DELIBERATELY FAIL so the constants
//!   below are refreshed before the assertion path is exercised.
//!   Once the four constants are pinned in this file, all future
//!   runs (without the env var) assert byte-equality.
//!
//! Fixture (panel-locked): canonical 16 entities × 128 windows,
//! K=1, D64 detector profile, panel-locked bank + canonical
//! contract. Same fixture used by R.9.b.3 byte-equivalence tests
//! (`d64_throughput_replay_is_byte_identical`).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact, GpuWorkspace,
};

/// Pinned pre-rewrite `TreeSha256V1` root digest for the
/// `residual` stage on the canonical 16×128 K=1 D64 fixture.
/// Captured ONCE during S-PERF.11 implementation on the
/// pre-rewrite kernel; must remain byte-identical across the
/// kernel-rewrite commit (and forever after; the S-PERF.10
/// `same_mode_digest_root_law` is panel-locked).
const PINNED_PRE_REWRITE_RESIDUAL_ROOT: [u8; 32] = [
    0x89, 0xa1, 0x85, 0x26, 0xfc, 0xa3, 0x58, 0xc4, 0x55, 0xd7, 0x10, 0xeb, 0xe0, 0xf2, 0xad, 0x67,
    0xb2, 0x95, 0xc4, 0xc2, 0x65, 0xcf, 0xd6, 0xd2, 0xe9, 0x70, 0xc5, 0xc2, 0x9c, 0x49, 0xdc, 0x2e,
];

/// Pinned pre-rewrite `TreeSha256V1` root digest for the
/// `sign` stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_REWRITE_SIGN_ROOT: [u8; 32] = [
    0xac, 0xe5, 0xe6, 0x26, 0x5a, 0x3c, 0xda, 0x85, 0x14, 0xfc, 0xb6, 0xa9, 0xaa, 0xa7, 0xf0, 0x19,
    0xc3, 0xec, 0x1b, 0xa9, 0xb7, 0xd2, 0xce, 0x7e, 0x6e, 0x23, 0xfd, 0x38, 0x3c, 0x33, 0xda, 0x59,
];

/// Pinned pre-rewrite `TreeSha256V1` root digest for the
/// `detector` stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_REWRITE_DETECTOR_ROOT: [u8; 32] = [
    0x87, 0x9d, 0xc9, 0xa9, 0x4c, 0x0b, 0x50, 0x43, 0xd3, 0x80, 0x6b, 0x43, 0x25, 0xa9, 0xa6, 0x64,
    0x34, 0x5e, 0x4d, 0x77, 0x37, 0x68, 0xca, 0x28, 0xed, 0x59, 0xe4, 0x6c, 0x2a, 0x5f, 0x4b, 0xae,
];

/// Pinned pre-rewrite `TreeSha256V1` root digest for the
/// `consensus` stage on the canonical 16×128 K=1 D64 fixture.
const PINNED_PRE_REWRITE_CONSENSUS_ROOT: [u8; 32] = [
    0x0d, 0x89, 0x49, 0xd5, 0x2d, 0x1a, 0xea, 0xc7, 0xd0, 0xf0, 0x44, 0x18, 0x38, 0x21, 0x27, 0x6b,
    0x23, 0x0f, 0x6a, 0x87, 0x19, 0x57, 0xfd, 0x32, 0x0a, 0x0d, 0x66, 0xd4, 0x4d, 0x47, 0xce, 0xd6,
];

/// Build a D64-pinned canonical contract identical to the
/// R.9.b.3 byte-equivalence tests.
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

#[test]
fn s_perf_11_pre_rewrite_root_capture() {
    // CAMPAIGN-IDENTITY assertion for S-PERF.11. Runs the D64
    // throughput dispatch on the canonical 16×128 K=1 fixture and
    // either prints + fails (capture mode) or asserts byte-equality
    // against the pinned constants (verification mode).

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
    let _case = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    let roots = ws
        .last_d64_stage_root_digests()
        .expect("D64 dispatch should have populated the pinned stage_digests shadow");

    if std::env::var("DSFB_S_PERF_11_CAPTURE").is_ok() {
        println!("=== S-PERF.11 pre-rewrite TreeSha256V1 root capture ===");
        println!("// canonical 16x128 K=1 D64 fixture, pre-rewrite kernel");
        println!(
            "const PINNED_PRE_REWRITE_RESIDUAL_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[0])
        );
        println!(
            "const PINNED_PRE_REWRITE_SIGN_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[1])
        );
        println!(
            "const PINNED_PRE_REWRITE_DETECTOR_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[2])
        );
        println!(
            "const PINNED_PRE_REWRITE_CONSENSUS_ROOT: [u8; 32] = [\n    {}\n];",
            hex_u8_array(&roots[3])
        );
        panic!(
            "DSFB_S_PERF_11_CAPTURE set: refresh the four pinned constants \
             above with the printed values, then re-run without the env var."
        );
    }

    assert_eq!(
        roots[0], PINNED_PRE_REWRITE_RESIDUAL_ROOT,
        "S-PERF.11: residual TreeSha256V1 root drifted from pre-rewrite capture; \
         same_mode_digest_root_law VIOLATED"
    );
    assert_eq!(
        roots[1], PINNED_PRE_REWRITE_SIGN_ROOT,
        "S-PERF.11: sign TreeSha256V1 root drifted from pre-rewrite capture; \
         same_mode_digest_root_law VIOLATED"
    );
    assert_eq!(
        roots[2], PINNED_PRE_REWRITE_DETECTOR_ROOT,
        "S-PERF.11: detector TreeSha256V1 root drifted from pre-rewrite capture; \
         same_mode_digest_root_law VIOLATED"
    );
    assert_eq!(
        roots[3], PINNED_PRE_REWRITE_CONSENSUS_ROOT,
        "S-PERF.11: consensus TreeSha256V1 root drifted from pre-rewrite capture; \
         same_mode_digest_root_law VIOLATED"
    );
}
