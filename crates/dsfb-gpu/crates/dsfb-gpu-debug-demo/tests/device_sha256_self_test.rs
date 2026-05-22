//! Tier 3B foundation: assert byte-equality between the host SHA-256
//! (`dsfb_gpu_debug_core::hash::sha256`) and the `__device__` SHA-256
//! exported by `cuda/sha256.cuh`.
//!
//! Three known-vector inputs cover the padding edge cases that bit the
//! host implementation when it was first ported:
//!
//! 1. **empty input** — exercises the all-padding path where the entire
//!    final block is `[0x80, 0..., len_be_u64]`.
//! 2. **55-byte input** — exactly fills the single-padded-block budget
//!    (`tail_len <= 55`); one more byte would force the two-block path.
//! 3. **64 KiB input** — covers the multi-block compression loop at a
//!    size larger than any per-stage buffer we will hash in Tier 3B.
//!
//! If any of these fails, **stop**. The Tier 3B digest kernels read the
//! same `dsfb_sha256_device` routine; a divergence here would surface
//! as a silent hash mismatch in the throughput pipeline.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::hash::sha256 as host_sha256;
use dsfb_gpu_debug_cuda::sha256_device;

#[test]
fn empty_input_matches_host() {
    let bytes: &[u8] = b"";
    let host = host_sha256(bytes);
    let device = sha256_device(bytes).expect("device sha256 self-test");
    assert_eq!(
        host, device,
        "device SHA-256 of empty input must match host (FIPS 180-4 \
         e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855)"
    );
}

#[test]
fn fifty_five_byte_input_matches_host() {
    let msg: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnop";
    assert_eq!(msg.len(), 55);
    let host = host_sha256(msg);
    let device = sha256_device(msg).expect("device sha256 self-test");
    assert_eq!(
        host, device,
        "device SHA-256 of 55-byte input must match host (single-block padding)"
    );
}

#[test]
fn sixty_four_kib_input_matches_host() {
    // Deterministic 64 KiB byte stream — a simple LCG so the bytes are
    // not all-zero (all-zero inputs miss the compression-loop bit
    // mixing) and easy to regenerate if the test ever needs auditing.
    let mut data = vec![0u8; 65_536];
    let mut s: u32 = 0x1234_5678;
    for byte in &mut data {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        *byte = (s >> 16) as u8;
    }
    let host = host_sha256(&data);
    let device = sha256_device(&data).expect("device sha256 self-test");
    assert_eq!(
        host, device,
        "device SHA-256 of 64 KiB pseudo-random input must match host (multi-block path)"
    );
}
