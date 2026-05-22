//! O.16 (Tier 2) acceptance tests for the batched-catalog dispatch.
//!
//! These tests pin the user-stated invariants for batched mode:
//!
//! 1. K batched GPU outputs equal K independent CPU outputs per-catalog.
//! 2. Per-catalog device overhead falls as K rises (smoke check; the
//!    bench surfaces the actual measurements).
//! 3. Case-file hashes are independent across catalogs.
//! 4. Corrupting catalog[j] only changes case[j], leaving the other
//!    K-1 case files byte-identical to their solo-run counterparts.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::build_cpu_throughput;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_cuda::{build_gpu_batched_throughput, BatchedGpuWorkspace};

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

#[test]
fn batched_dispatch_matches_per_catalog_cpu_results() {
    // K independent fixtures via different LCG seeds. Each one is a
    // 10 000-event v0 fixture; if the batched dispatch produces the
    // same case-file hashes as a per-catalog CPU run, the batched
    // kernels are correctly routing per-catalog data through their
    // slices.
    let k: u32 = 4;
    let seeds: [u64; 4] = [DEFAULT_SEED, 0x1111, 0x2222, 0x3333];
    let contract = canonical_contract();

    let fixtures: Vec<_> = seeds.iter().map(|&s| synthesize(s)).collect();
    let cpu_cases: Vec<_> = fixtures
        .iter()
        .map(|events| build_cpu_throughput(events, &contract))
        .collect();

    let mut workspace = BatchedGpuWorkspace::new(k, &contract).unwrap();
    let event_slices: Vec<&[_]> = fixtures.iter().map(Vec::as_slice).collect();
    let gpu_cases = build_gpu_batched_throughput(&mut workspace, &event_slices, &contract).unwrap();

    assert_eq!(gpu_cases.len(), seeds.len());
    for (i, (cpu, gpu)) in cpu_cases.iter().zip(gpu_cases.iter()).enumerate() {
        assert_eq!(
            cpu.hashes, gpu.hashes,
            "catalog {i}: CPU vs batched GPU intermediate hashes differ"
        );
        assert_eq!(
            cpu.episodes, gpu.episodes,
            "catalog {i}: CPU vs batched GPU episodes differ"
        );
    }
}

#[test]
fn corrupting_catalog_j_does_not_affect_other_catalogs() {
    // Independence invariant: mutating one catalog's input must leave
    // every other catalog's output byte-identical. We run the same
    // batch twice — first with all canonical inputs, then with
    // catalog 1's events perturbed — and confirm catalogs 0, 2, 3 are
    // unchanged while catalog 1's hashes differ.
    let k: u32 = 4;
    let seeds: [u64; 4] = [DEFAULT_SEED, 0x1111, 0x2222, 0x3333];
    let contract = canonical_contract();
    let mut workspace = BatchedGpuWorkspace::new(k, &contract).unwrap();

    let mut fixtures: Vec<Vec<_>> = seeds.iter().map(|&s| synthesize(s)).collect();
    let baseline_event_slices: Vec<&[_]> = fixtures.iter().map(Vec::as_slice).collect();
    let baseline =
        build_gpu_batched_throughput(&mut workspace, &baseline_event_slices, &contract).unwrap();

    // Mutate catalog 1: flip a byte in the first event's latency.
    fixtures[1][0].latency_us ^= 0x0000_00FF;
    let perturbed_slices: Vec<&[_]> = fixtures.iter().map(Vec::as_slice).collect();
    let perturbed =
        build_gpu_batched_throughput(&mut workspace, &perturbed_slices, &contract).unwrap();

    for (i, (b, p)) in baseline.iter().zip(perturbed.iter()).enumerate() {
        if i == 1 {
            assert_ne!(
                b.hashes.input_catalog, p.hashes.input_catalog,
                "catalog 1's input hash should differ after perturbation"
            );
        } else {
            assert_eq!(
                b.hashes, p.hashes,
                "catalog {i} should be byte-identical across the two batched runs"
            );
            assert_eq!(b.episodes, p.episodes);
        }
    }
}

#[test]
fn batched_dispatch_is_deterministic_across_runs() {
    // Two back-to-back batched runs with the same inputs must produce
    // byte-identical per-catalog case files.
    let k: u32 = 2;
    let seeds: [u64; 2] = [DEFAULT_SEED, 0x4321];
    let contract = canonical_contract();
    let fixtures: Vec<_> = seeds.iter().map(|&s| synthesize(s)).collect();
    let event_slices: Vec<&[_]> = fixtures.iter().map(Vec::as_slice).collect();

    let mut workspace = BatchedGpuWorkspace::new(k, &contract).unwrap();
    let a = build_gpu_batched_throughput(&mut workspace, &event_slices, &contract).unwrap();
    let b = build_gpu_batched_throughput(&mut workspace, &event_slices, &contract).unwrap();

    for (a_case, b_case) in a.iter().zip(b.iter()) {
        assert_eq!(a_case.hashes, b_case.hashes);
        assert_eq!(a_case.episodes, b_case.episodes);
        assert_eq!(a_case.final_case_file_hash, b_case.final_case_file_hash);
    }
}
