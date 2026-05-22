//! S-PERF.14b.1 Step 1 — streaming SHA-256 vs one-shot
//! byte-equivalence self-test on three known-vector inputs.
//!
//! **Purpose (panel-locked, 2026-05-18)**:
//!
//!   Before the rewritten
//!   `compact_densor_digest_v1_root_kernel_streaming` is allowed
//!   to consume the streaming SHA-256 init/update/finalize
//!   helpers, those helpers MUST produce byte-identical 32-byte
//!   digests to the one-shot `dsfb_sha256_device` for any input
//!   byte sequence. This test exercises three known-vector
//!   inputs that cover the SHA-256 tail-padding edge cases:
//!
//!     1. empty (0 bytes)        — single padding block, length=0
//!     2. 55 bytes               — single padding block, length fits
//!                                  in the same 64-byte block as
//!                                  the 0x80 marker (tail_len ≤ 55)
//!     3. 64 KiB (= 65 536 B)    — multi-block + clean tail
//!                                  (1024 full blocks + 0 tail)
//!
//!   These cover the three branches of the tail-padding logic in
//!   `dsfb_sha256_finalize` (and the equivalent logic in
//!   `dsfb_sha256_device`'s one-shot path). If streaming ≠ one-
//!   shot for any input, the rewrite is blocked until the
//!   divergence is fixed — the streaming helpers are not safe to
//!   consume in production until this test passes.
//!
//!   Cross-validates against the host SHA-256 in
//!   `dsfb_gpu_debug_core::hash::sha256` as a third independent
//!   reference (so a bug in BOTH device implementations would be
//!   surfaced).

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::hash::sha256 as host_sha256;
use dsfb_gpu_debug_cuda::{sha256_device, sha256_device_streaming};

/// 64 KiB = 1024 full 64-byte SHA blocks with 0-byte tail.
const SIZE_64K: usize = 64 * 1024;

/// Generate the 64-KiB known-vector input. Deterministic
/// byte pattern (`(i % 256) as u8`) so the input is
/// reproducible across machines without depending on RNG
/// state or fixture files.
fn build_64k_input() -> Vec<u8> {
    (0..SIZE_64K).map(|i| (i % 256) as u8).collect()
}

#[test]
fn s_perf_14b_1_streaming_equals_one_shot_empty() {
    let input: &[u8] = &[];
    let streaming = sha256_device_streaming(input).expect("streaming sha self-test");
    let one_shot = sha256_device(input).expect("one-shot sha self-test");
    let host = host_sha256(input);
    assert_eq!(
        streaming, one_shot,
        "streaming SHA-256 must byte-equal one-shot on empty input \
         (SHA-256 single padding block, length=0)"
    );
    assert_eq!(
        streaming, host,
        "streaming SHA-256 must byte-equal host SHA-256 on empty input \
         (third-party reference cross-check)"
    );
}

#[test]
fn s_perf_14b_1_streaming_equals_one_shot_55_bytes() {
    let input: Vec<u8> = (0..55u8).collect();
    let streaming = sha256_device_streaming(&input).expect("streaming sha self-test");
    let one_shot = sha256_device(&input).expect("one-shot sha self-test");
    let host = host_sha256(&input);
    assert_eq!(
        streaming, one_shot,
        "streaming SHA-256 must byte-equal one-shot on 55-byte input \
         (tail_len = 55 ≤ 55 edge case — length encodes in the same 64-byte block)"
    );
    assert_eq!(
        streaming, host,
        "streaming SHA-256 must byte-equal host reference at 55 bytes"
    );
}

#[test]
fn s_perf_14b_1_streaming_equals_one_shot_64_kib() {
    let input = build_64k_input();
    let streaming = sha256_device_streaming(&input).expect("streaming sha self-test");
    let one_shot = sha256_device(&input).expect("one-shot sha self-test");
    let host = host_sha256(&input);
    assert_eq!(
        streaming, one_shot,
        "streaming SHA-256 must byte-equal one-shot on 64-KiB input \
         (1024 full blocks + 0 tail — multi-block clean-tail edge case)"
    );
    assert_eq!(
        streaming, host,
        "streaming SHA-256 must byte-equal host reference at 64 KiB"
    );
}

#[test]
fn s_perf_14b_1_streaming_is_deterministic_across_two_runs() {
    let input = build_64k_input();
    let a = sha256_device_streaming(&input).expect("streaming sha self-test run a");
    let b = sha256_device_streaming(&input).expect("streaming sha self-test run b");
    assert_eq!(
        a, b,
        "streaming SHA-256 must be deterministic across two runs"
    );
}
