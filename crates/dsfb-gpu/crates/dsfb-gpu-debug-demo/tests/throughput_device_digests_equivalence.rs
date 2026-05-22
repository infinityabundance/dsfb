//! Tier 3B acceptance test: the Throughput-mode case file produced
//! by the device-digest path (`build_gpu_throughput_device_digests_on_workspace`)
//! must equal, byte-for-byte, the Throughput-mode case file produced
//! by the host-digest path (`build_gpu_throughput_on_workspace`).
//!
//! Both paths run the same five CUDA pipeline kernels and feed the
//! same `EmissionMode::Throughput` casefile builder. The only difference
//! is where the four residual / sign / detector / consensus stage
//! digests are computed:
//!
//! * Host-digest path: 4 host SHA-256 calls over the D2H-copied cell
//!   buffers (Tier 3A).
//! * Device-digest path: 4 `__device__` SHA-256 calls hashing the
//!   `#[repr(C)]` cell buffers on-device; only the 4 × 32-byte
//!   digests come back to the host (Tier 3B).
//!
//! If this test fails, either (a) the device SHA-256 disagrees with
//! the host SHA-256 (covered separately by
//! `device_sha256_self_test`) or (b) the on-device cell-buffer
//! layout disagrees with the host's `hash_*_compact` byte form.
//! Either way the device-digest path is unsafe to enable.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{
    build_gpu_batched_throughput, build_gpu_batched_throughput_device_digests,
    build_gpu_throughput_device_digests_on_workspace, build_gpu_throughput_on_workspace,
    BatchedGpuWorkspace, GpuWorkspace,
};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn single_catalog_device_digests_match_host_digests() {
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);

    let mut ws_host = GpuWorkspace::new(&contract).unwrap();
    let host_case = build_gpu_throughput_on_workspace(&mut ws_host, &events, &contract).unwrap();

    let mut ws_dev = GpuWorkspace::new(&contract).unwrap();
    let dev_case =
        build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws_dev).unwrap();

    assert_eq!(
        host_case.hashes, dev_case.hashes,
        "Tier 3B device-digest case file diverges from Tier 3A host-digest case file at \
         the intermediate-hash level. Either the device SHA-256 or the on-device cell \
         layout disagrees with the host equivalents."
    );
    assert_eq!(host_case.episodes, dev_case.episodes);
    assert_eq!(
        host_case.final_case_file_hash,
        dev_case.final_case_file_hash
    );
    assert_eq!(host_case.final_verdict, dev_case.final_verdict);
}

#[test]
fn batched_device_digests_match_host_digests_for_each_catalog() {
    let k: u32 = 4;
    let seeds: [u64; 4] = [DEFAULT_SEED, 0x1111, 0x2222, 0x3333];
    let contract = canonical_contract();

    let fixtures: Vec<_> = seeds.iter().map(|&s| synthesize(s)).collect();
    let slices: Vec<&[_]> = fixtures.iter().map(Vec::as_slice).collect();

    let mut ws_host = BatchedGpuWorkspace::new(k, &contract).unwrap();
    let host_cases = build_gpu_batched_throughput(&mut ws_host, &slices, &contract).unwrap();

    let mut ws_dev = BatchedGpuWorkspace::new(k, &contract).unwrap();
    let dev_cases =
        build_gpu_batched_throughput_device_digests(&mut ws_dev, &slices, &contract).unwrap();

    assert_eq!(host_cases.len(), dev_cases.len());
    for (i, (h, d)) in host_cases.iter().zip(dev_cases.iter()).enumerate() {
        assert_eq!(
            h.hashes, d.hashes,
            "catalog {i}: device-digest hashes differ from host-digest"
        );
        assert_eq!(h.episodes, d.episodes);
        assert_eq!(h.final_case_file_hash, d.final_case_file_hash);
    }
}

#[test]
fn device_digest_path_is_deterministic_across_runs() {
    let contract = canonical_contract();
    let events = synthesize(DEFAULT_SEED);
    let mut ws = GpuWorkspace::new(&contract).unwrap();
    let a = build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws).unwrap();
    let b = build_gpu_throughput_device_digests_on_workspace(&events, &contract, &mut ws).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}
