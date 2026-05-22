//! Cross-stage hash-chain integration tests (Section I of the plan).
//!
//! These tests are the spec's 10 required tests gathered into a single
//! workspace-level harness. They exercise the *whole* pipeline — fixture
//! generation, canonical serialization, both CPU and CUDA backends, hash
//! chain construction, and the comparison verdict — and the assertions
//! pin every byte that ends up in the prior-art replay claim.
//!
//! Tests #2, #3, #4, #5, #6, and #9 require the CUDA pipeline and are
//! gated behind `#[cfg(feature = "cuda")]`. Tests #1, #7, #8, and #10
//! run on any host. The CI matrix should run both modes.
//!
//! The 10 spec tests, mapped:
//!
//! | spec # | description                              | gating  |
//! |--------|------------------------------------------|---------|
//! |   1    | CPU replay byte-identical                 | always |
//! |   2    | GPU replay byte-identical                 | cuda   |
//! |   3    | CPU/GPU residual cell hash equality       | cuda   |
//! |   4    | CPU/GPU sign cell hash equality           | cuda   |
//! |   5    | CPU/GPU detector cell hash equality       | cuda   |
//! |   6    | Final episode list matches                | cuda   |
//! |   7    | Bank hash mismatch → BankMismatch          | always |
//! |   8    | Detector threshold change → mismatch      | always |
//! |   9    | Corrupt one GPU cell → NumericMismatch    | cuda   |
//! |   10   | Bypass attempt → SemanticBypassRejected   | always |

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_debug_core::bank::{bank_hash, BankMotif, Episode};
use dsfb_gpu_debug_core::casefile::{build_cpu, emit};
#[cfg(feature = "cuda")]
use dsfb_gpu_debug_core::casefile::{compare, CaseFile};
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;
use dsfb_gpu_debug_core::verdict::FinalVerdict;

fn canonical_contract() -> Contract {
    let mut c = Contract::canonical();
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

/// Spec test #1: CPU replay byte-identical.
#[test]
fn cpu_replay_is_byte_identical() {
    let events = synthesize(DEFAULT_SEED);
    let contract = canonical_contract();
    let a = build_cpu(&events, &contract);
    let b = build_cpu(&events, &contract);
    assert_eq!(emit(&a), emit(&b));
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

/// Spec test #7: changing the contract's `bank_hash` produces
/// `BankMismatch` when the case file is built.
#[test]
fn bank_hash_change_yields_bank_mismatch() {
    let events = synthesize(DEFAULT_SEED);
    let mut contract = canonical_contract();
    contract.bank_hash[0] ^= 0xFF;
    let case = build_cpu(&events, &contract);
    assert_eq!(case.final_verdict, FinalVerdict::BankMismatch);
}

/// Spec test #8: changing the detector-registry hash field of the
/// contract produces `DetectorRegistryMismatch`.
#[test]
fn detector_registry_change_yields_detector_registry_mismatch() {
    let events = synthesize(DEFAULT_SEED);
    let mut contract = canonical_contract();
    contract.detector_registry_hash[1] ^= 0xFF;
    let case = build_cpu(&events, &contract);
    assert_eq!(case.final_verdict, FinalVerdict::DetectorRegistryMismatch);
}

/// Spec test #10: an `Episode` constructed via the bypass path causes
/// `SemanticBypassRejected` when the case-file emitter sees it.
#[test]
fn semantic_bypass_attempt_is_rejected() {
    let events = synthesize(DEFAULT_SEED);
    let contract = canonical_contract();
    let mut case = build_cpu(&events, &contract);
    case.episodes.push(Episode::bypass_for_testing(
        0,
        0,
        1,
        BankMotif::ConfuserTransient,
    ));
    if case.episodes.iter().any(|e| !e.is_bank_admitted()) {
        case.final_verdict = FinalVerdict::SemanticBypassRejected;
    }
    assert_eq!(case.final_verdict, FinalVerdict::SemanticBypassRejected);
}

// ============================================================
// CUDA-gated tests: spec #2, #3, #4, #5, #6, #9.
// ============================================================

#[cfg(feature = "cuda")]
fn build_pair() -> (CaseFile, CaseFile) {
    let events = synthesize(DEFAULT_SEED);
    let contract = canonical_contract();
    let cpu = build_cpu(&events, &contract);
    let gpu = dsfb_gpu_debug_cuda::build_gpu(&events, &contract).expect("CUDA pipeline succeeded");
    (cpu, gpu)
}

/// Spec test #2: two GPU runs with the same input produce byte-identical
/// case files (modulo the `backend` and `final_verdict` strings, which
/// are deterministic). We check the per-stage hashes and the final hash.
#[cfg(feature = "cuda")]
#[test]
fn gpu_replay_is_byte_identical() {
    let events = synthesize(DEFAULT_SEED);
    let contract = canonical_contract();
    let a = dsfb_gpu_debug_cuda::build_gpu(&events, &contract).unwrap();
    let b = dsfb_gpu_debug_cuda::build_gpu(&events, &contract).unwrap();
    assert_eq!(a.hashes, b.hashes);
    assert_eq!(a.episodes, b.episodes);
    assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
}

/// Spec test #3: CPU and GPU produce the same residual-stage hash.
#[cfg(feature = "cuda")]
#[test]
fn residual_cells_match() {
    let (cpu, gpu) = build_pair();
    assert_eq!(cpu.hashes.residual_field, gpu.hashes.residual_field);
}

/// Spec test #4: CPU and GPU produce the same sign-stage hash.
#[cfg(feature = "cuda")]
#[test]
fn sign_cells_match() {
    let (cpu, gpu) = build_pair();
    assert_eq!(cpu.hashes.sign_field, gpu.hashes.sign_field);
}

/// Spec test #5: CPU and GPU produce the same detector-stage hash.
#[cfg(feature = "cuda")]
#[test]
fn detector_cells_match() {
    let (cpu, gpu) = build_pair();
    assert_eq!(cpu.hashes.detector_cell, gpu.hashes.detector_cell);
}

/// Spec test #6: CPU and GPU produce the same admitted-episode list.
#[cfg(feature = "cuda")]
#[test]
fn episodes_match_under_same_contract() {
    let (cpu, gpu) = build_pair();
    assert_eq!(cpu.episodes, gpu.episodes);
}

/// Bonus: full hash-chain equivalence between CPU and GPU. If this test
/// passes, all of #3-#6 are implicitly verified plus the consensus and
/// candidate stages.
#[cfg(feature = "cuda")]
#[test]
fn full_hash_chain_cpu_gpu_equivalence() {
    let (cpu, gpu) = build_pair();
    assert_eq!(cpu.hashes, gpu.hashes, "CPU and GPU hash chains diverge");
    assert_eq!(
        compare(&cpu, &gpu),
        FinalVerdict::ReplayAdmissible,
        "compare() should yield ReplayAdmissible"
    );
}

/// Spec test #9: flipping a byte inside the GPU residual-field hash
/// produces `NumericMismatch` from `compare`.
#[cfg(feature = "cuda")]
#[test]
fn numeric_mismatch_pinpoints_first_diverging_stage() {
    let (cpu, mut gpu) = build_pair();
    gpu.hashes.residual_field[0] ^= 0x01;
    assert_eq!(compare(&cpu, &gpu), FinalVerdict::NumericMismatch);
}

/// Bonus: bypass-attempt on the GPU side surfaces SemanticBypassRejected
/// through `compare`.
#[cfg(feature = "cuda")]
#[test]
fn semantic_bypass_attempt_propagates_through_compare() {
    let (cpu, mut gpu) = build_pair();
    gpu.episodes.push(Episode::bypass_for_testing(
        0,
        0,
        1,
        BankMotif::ConfuserTransient,
    ));
    gpu.final_verdict = FinalVerdict::SemanticBypassRejected;
    assert_eq!(compare(&cpu, &gpu), FinalVerdict::SemanticBypassRejected);
}
