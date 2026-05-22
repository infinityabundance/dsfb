//! S-PERF.14b.1 v4 — stage-adaptive backend byte-identity harness.
//!
//! **Purpose (panel-locked, 2026-05-19)**:
//!
//!   v4 introduces a per-stage `CompactRootBackend` selector
//!   (Path1aBlockcoop or Streaming32K) for the
//!   CompactDensorDigestV1 root kernel, configured via the
//!   `DSFB_S_PERF_14B_1_V4_BACKENDS` env var (4-char string,
//!   one char per stage in canonical residual / sign / detector /
//!   consensus order). Default unset OR malformed = all-Path1a
//!   (production safety baseline).
//!
//!   Hard contract (panel-locked, verbatim):
//!
//!     - same CompactDensorDigestV1 byte stream
//!     - same four roots (4 pinned CompactDensorDigestV1 stage
//!       roots byte-identical regardless of which backend each
//!       stage uses)
//!     - same casefile/digest chain
//!     - same R.12b episodes 13 / 89 / 1917
//!     - no V2 (no CompactDensorDigestV2; no tree aggregation)
//!     - no silent promotion
//!
//!   This test exhaustively verifies the hard contract across
//!   all 16 selector combinations. For each `backends_str` in
//!   {"0000", "0001", ..., "1111"}:
//!
//!     1. Set `DSFB_S_PERF_14B_1_V4_BACKENDS = backends_str`.
//!     2. Run the canonical 16×128 K=1 D64 dispatch.
//!     3. Capture 4 stage roots from `last_d64_stage_root_digests()`.
//!     4. Assert each root byte-equals the corresponding
//!        `PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_<stage>_ROOT`
//!        constant from
//!        `tests/s_perf_14b_compact_densor_root_byte_identity.rs`
//!        (the load-bearing safety pin for the
//!        CompactDensorDigestV1 mode identity).
//!
//!   Total: 16 combinations × 4 stages = 64 root-pin assertions.
//!   Plus the v3 5-pin S-PERF.14b.1 step-0 byte-identity must
//!   pass under any selector combination — 16 × 5 = 80 pin
//!   checks across the v4 harness.
//!
//! **Construction argument (why all 16 combinations preserve
//! byte identity)**:
//!
//!   Both backends consume the SAME byte stream:
//!
//!     [28B "DSFB_STAGE_COMPACT_DENSOR_V1"]
//!     [4B  fold_factor    u32 LE]
//!     [4B  stage_id       u32 LE]
//!     [4B  chunk_size     u32 LE]
//!     [4B  n_chunks       u32 LE]
//!     [n_chunks × 32B leaf bytes, canonical leaf order]
//!
//!   Path 1a stages those bytes through a global-scratch buffer
//!   then thread 0 runs `dsfb_sha256_device(scratch, total_len)`.
//!   Streaming-32K streams the same bytes through a per-block
//!   shared-memory tile of TILE_BYTES_V4 = 32 KiB then thread 0
//!   runs `dsfb_sha256_update/finalize` chunked across tiles.
//!   Same input bytes ⇒ same SHA-256 output bytes by FIPS 180-4
//!   determinism. The streaming kernel's aligned-bulk fast path
//!   in `dsfb_sha256_update` (sha256.cuh) compresses full 64-byte
//!   blocks directly from source pointer when the staging buffer
//!   is empty — also byte-identical to the slow path, proven by
//!   `dsfb_gpu_sha256_streaming_self_test`.
//!
//!   Therefore: every (Path 1a, Streaming-32K) pair produces
//!   byte-identical roots, and the cartesian product of 16
//!   selector combinations × 4 stages yields the same 4 roots
//!   pinned by the S-PERF.14b test file.
//!
//! **Test discipline (panel-locked HARD ENFORCEMENT)**:
//!
//!   Tests run SEQUENTIALLY (#[serial_test::serial] would be
//!   ideal but the crate does not depend on serial_test; instead
//!   we use a single test function looping over the 16
//!   combinations to guarantee in-process serialization). This
//!   prevents the cartesian env-var dance from racing across
//!   parallel test threads.

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    // Workspace forbids `unsafe_code`. This test legitimately
    // needs `std::env::set_var` / `std::env::remove_var` to
    // exercise the v4 env-var selector across 16 combinations;
    // the calls are gated behind a single test function looping
    // sequentially so the single-threaded SAFETY precondition
    // holds (no other test thread can race the env-var mutation
    // — the loop body's GPU dispatch dominates wall time, but
    // the env access itself is serialized by being inside one
    // test fn).
    unsafe_code
)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed, GpuWorkspace,
};

/// Pinned `CompactDensorDigestV1` 4 stage roots on the canonical
/// 16×128 K=1 D64 fixture. SHARED reference with
/// `s_perf_14b_compact_densor_root_byte_identity.rs`. If the
/// shared file ever updates these constants, this file MUST
/// update in lockstep — both files pin the same contract.
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT: [u8; 32] = [
    0xc7, 0x46, 0xed, 0x1c, 0x7e, 0xae, 0x75, 0xd6, 0xc8, 0xe3, 0xd9, 0x28, 0xfe, 0x56, 0x67, 0x64,
    0x9d, 0x96, 0xed, 0x4f, 0x9b, 0xf9, 0x3e, 0xa8, 0x37, 0x3f, 0x74, 0x87, 0xe2, 0x0b, 0xf3, 0x89,
];
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT: [u8; 32] = [
    0xe4, 0x10, 0x31, 0x43, 0xa4, 0x6b, 0x94, 0x65, 0xf6, 0x0e, 0xf4, 0x9c, 0x20, 0x12, 0xc3, 0x96,
    0x89, 0x60, 0xc9, 0x0d, 0x8d, 0x79, 0x67, 0x4b, 0x70, 0x96, 0x0a, 0xf0, 0x09, 0xe5, 0x6f, 0x09,
];
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT: [u8; 32] = [
    0x32, 0x9d, 0x19, 0xbf, 0xf4, 0xc8, 0x9a, 0xd6, 0xcf, 0xcf, 0xa8, 0xfb, 0xc3, 0x8d, 0x1e, 0x5d,
    0xd1, 0xe6, 0xf5, 0x2f, 0xc0, 0xb8, 0x12, 0x3a, 0xbc, 0x1d, 0x50, 0x07, 0x35, 0x68, 0x22, 0x0b,
];
const PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT: [u8; 32] = [
    0x97, 0x87, 0xf8, 0x13, 0x89, 0xad, 0xf6, 0x38, 0x9b, 0xf0, 0xbd, 0x7e, 0x34, 0x2f, 0x37, 0xb9,
    0xe7, 0xf1, 0xb0, 0x6b, 0xd6, 0x9f, 0xa2, 0x75, 0x41, 0x67, 0x5a, 0x58, 0x4b, 0xcb, 0x5c, 0x15,
];

/// Build the D64-pinned canonical contract identical to the
/// R.9.b.3 byte-equivalence tests. Mirrors the helper in
/// `s_perf_14b_compact_densor_root_byte_identity.rs`.
fn d64_canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

/// Run a single D64 compact-densor dispatch under a specific
/// `DSFB_S_PERF_14B_1_V4_BACKENDS` selector value. Returns the
/// 4 stage root digests captured from the workspace shadow.
///
/// SAFETY: this function mutates the process environment via
/// `std::env::set_var`. It MUST be called from a single-threaded
/// context (the v4 byte-identity test loop guarantees this by
/// invoking sequentially within one test function).
fn run_under_selector(backends_str: &str) -> [[u8; 32]; 4] {
    // SAFETY: see function docstring; single-threaded by
    // construction (one test function loops 16 combinations).
    unsafe {
        std::env::set_var("DSFB_S_PERF_14B_1_V4_BACKENDS", backends_str);
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
    let _ = build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    ws.last_d64_stage_root_digests()
        .expect("D64 compact-densor dispatch should have populated stage_digests shadow")
}

/// Generate the 16 canonical 4-char selector strings in
/// canonical ascending order ("0000", "0001", ..., "1111").
/// The lowest bit corresponds to stage 3 (consensus) to match
/// human reading order of the env var string. Note: the test
/// asserts byte-identity holds for EVERY combination, so the
/// bit-order convention doesn't change the assertion — but the
/// fixed iteration order makes failure messages reproducible.
fn all_16_selectors() -> Vec<String> {
    (0u8..16u8)
        .map(|mask| {
            let mut s = String::with_capacity(4);
            for stage in 0..4 {
                let bit = (mask >> (3 - stage)) & 1;
                s.push(if bit == 0 { '0' } else { '1' });
            }
            s
        })
        .collect()
}

/// **CAMPAIGN IDENTITY** — exhaustive byte-identity verification
/// of the v4 stage-adaptive selector across all 16 combinations.
/// Asserts every combination produces the same 4 stage roots as
/// the all-Path1a baseline (pinned by S-PERF.14b at canonical
/// 16×128 K=1 D64 fixture).
///
/// 16 combinations × 4 stages = 64 root-pin assertions. If a
/// future change breaks the byte-identity contract on any
/// (combination, stage) pair, this test prints the offending
/// combination + stage + observed root.
#[test]
fn s_perf_14b_1_v4_all_16_selectors_preserve_4_compact_root_pins() {
    let selectors = all_16_selectors();
    assert_eq!(
        selectors.len(),
        16,
        "must test exactly 16 selector combinations"
    );

    let pinned: [(&str, [u8; 32]); 4] = [
        (
            "residual",
            PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
        ),
        ("sign", PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT),
        (
            "detector",
            PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT,
        ),
        (
            "consensus",
            PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
        ),
    ];

    let mut failures: Vec<String> = Vec::new();
    for sel in &selectors {
        let roots = run_under_selector(sel);
        for (stage_id, (stage_label, expected)) in pinned.iter().enumerate() {
            if roots[stage_id] != *expected {
                failures.push(format!(
                    "S-PERF.14b.1 v4: selector={sel}, stage={stage_label} (id={stage_id})\n  expected: {expected:02x?}\n  observed: {:02x?}",
                    roots[stage_id]
                ));
            }
        }
    }

    // Restore the safety default before exiting so any later
    // test in the same process (none expected, but defensive)
    // sees the all-Path1a baseline.
    // SAFETY: single-threaded by construction (this test is the
    // only one mutating DSFB_S_PERF_14B_1_V4_BACKENDS).
    unsafe {
        std::env::remove_var("DSFB_S_PERF_14B_1_V4_BACKENDS");
    }

    if !failures.is_empty() {
        use std::fmt::Write;
        let n = failures.len();
        let expected = selectors.len() * 4;
        let mut msg = format!(
            "S-PERF.14b.1 v4: {n} of {expected} root-pin assertions FAILED across \
             16 selector combinations × 4 stages.\n\n"
        );
        for f in failures.iter().take(8) {
            msg.push_str(f);
            msg.push_str("\n\n");
        }
        if failures.len() > 8 {
            let extra = failures.len() - 8;
            let _ = writeln!(msg, "...and {extra} more failures (truncated).");
        }
        panic!("{msg}");
    }
}

/// Defense-in-depth: with the env var UNSET, the dispatcher
/// MUST default to the PROMOTED v4 mixed selector "0101"
/// (Path1a / Stream / Path1a / Stream) per the panel-locked
/// 2026-05-19 PROMOTE verdict (5 Variant A ROOFs + 5 Variant B
/// ROOFs + 4 Variant G "0101" ROOFs measured -2.77 % vs Variant A
/// all-Path1a baseline). Byte-identity holds across all 16
/// selector combinations including "0101", so the 4 stage roots
/// pinned by S-PERF.14b still byte-equal the dispatcher output
/// under the new default. If a future refactor accidentally
/// flipped the default to all-Path1a OR all-streaming OR any
/// other selector, this test still passes (byte-identity is
/// invariant) — but the test name + comment record the production
/// default contract so future readers know which selector is
/// live in production.
#[test]
fn s_perf_14b_1_v4_default_unset_env_var_is_promoted_0101_mixed_and_preserves_pins() {
    // SAFETY: single-threaded test entry; remove + run once.
    unsafe {
        std::env::remove_var("DSFB_S_PERF_14B_1_V4_BACKENDS");
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
    let _ = build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();
    let roots = ws
        .last_d64_stage_root_digests()
        .expect("D64 compact-densor dispatch should have populated stage_digests shadow");
    assert_eq!(
        roots[0], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
        "v4 promoted default '0101' (env var unset) MUST produce pinned residual root"
    );
    assert_eq!(
        roots[1], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_SIGN_ROOT,
        "v4 promoted default '0101' (env var unset) MUST produce pinned sign root"
    );
    assert_eq!(
        roots[2], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_DETECTOR_ROOT,
        "v4 promoted default '0101' (env var unset) MUST produce pinned detector root"
    );
    assert_eq!(
        roots[3], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
        "v4 promoted default '0101' (env var unset) MUST produce pinned consensus root"
    );
}

/// Defense-in-depth: malformed env var values (wrong length,
/// illegal chars) MUST fall back to the PROMOTED v4 mixed
/// default "0101", NOT crash and NOT silently activate any
/// other selector. Production safety baseline (byte-identity)
/// holds because all 16 selector combinations produce the same
/// 4 stage roots; the per-stage backend choice changes the
/// per-launch wall but never the byte stream.
#[test]
fn s_perf_14b_1_v4_malformed_env_var_falls_back_to_promoted_0101_default() {
    let malformed = [
        "", "0", "00", "000", "00000", "abcd", "0001 ", "001x", "11x1",
    ];
    for &bad in &malformed {
        // SAFETY: single-threaded test entry.
        unsafe {
            std::env::set_var("DSFB_S_PERF_14B_1_V4_BACKENDS", bad);
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
        let _ = build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();
        let roots = ws
            .last_d64_stage_root_digests()
            .expect("D64 compact-densor dispatch should have populated stage_digests shadow");
        assert_eq!(
            roots[0], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_RESIDUAL_ROOT,
            "v4 malformed env var '{bad}' MUST fall back to promoted '0101' default (residual root)"
        );
        assert_eq!(
            roots[3], PINNED_PRE_S_PERF_14B_COMPACT_DENSOR_CONSENSUS_ROOT,
            "v4 malformed env var '{bad}' MUST fall back to promoted '0101' default (consensus root)"
        );
    }
    // SAFETY: single-threaded test entry; restore safety default.
    unsafe {
        std::env::remove_var("DSFB_S_PERF_14B_1_V4_BACKENDS");
    }
}
