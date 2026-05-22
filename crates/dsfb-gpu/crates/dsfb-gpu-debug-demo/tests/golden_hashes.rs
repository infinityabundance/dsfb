//! Golden-hash pinning test.
//!
//! Every stage hash plus the final case-file hash is recorded here as a
//! literal byte array. Any change to the pipeline that alters one of
//! these — a detector threshold tweak, a bank entry rename, a fixture
//! seed change, a Q16 rounding adjustment — triggers a hard failure
//! with a precise pointer to the diverging stage.
//!
//! This is the strongest single test the prior-art repository carries:
//! it locks the deterministic output bit-for-bit against a hand-recorded
//! reference. Future contract revisions that intend to change behaviour
//! must update both the relevant module *and* these constants together,
//! which makes drift impossible to merge accidentally.
//!
//! The reference values were captured from the canonical fixture
//! (`fixtures/synthetic_trace.json`, default LCG seed) using the
//! `Contract::canonical()` contract with bank and detector-registry
//! hashes pinned to the in-repo canonical values.

#![allow(clippy::unwrap_used)]

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::build_cpu;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;

const GOLDEN_BANK: &str = "497a40c495b7c70e65086770a3575569c31c28249e400f6cd3787e8d97216506";
// R.5 refresh: CandidateInterval grew by 8 bytes (entity_avg_q + grid_avg_q
// for the axis-5 on-device move). The new canonical bytes re-pin
// candidate_interval, which cascades into episode and final.
const GOLDEN_CANDIDATE: &str = "ae6036982dd46a90b210fee29aed121bf6b05ce5bd90fe3480cc5e7f025a7296";
const GOLDEN_CONSENSUS: &str = "4b5f3ba7229bcb9c8c677c3228799ddb48b170f3032d6129fb8200f71662a9c8";
const GOLDEN_CONTRACT: &str = "4c2b473e660d5034b39eb68be28353ec04e31b4b002d354541d12546ca3566b5";
const GOLDEN_DETECTOR: &str = "c4fbabbd807574bd22ae9bcfef6e776a1cb8071194c2f6f3983df79dda4b7d26";
const GOLDEN_DETREG: &str = "72c0cde2732087a89d7369825780a90bfb3278a8f055122ac5a500752a05535d";
const GOLDEN_EPISODE: &str = "5498bb237244c161867a2883eb6b02001536727b41b8a4396c05a79a04c3958a";
const GOLDEN_INPUT: &str = "1eeefffa2a1029672a9e6a55e575c928fca926e8077e6784a392597ccd640487";
const GOLDEN_KSEQ: &str = "77117e5d74f1b45168be90159a517d9629fa324c0c97bc902865bfd903041960";
const GOLDEN_RESIDUAL: &str = "52578099ee75e1213372d9128402401433a063e5576a533ee605abe128247ef7";
const GOLDEN_SIGN: &str = "e2609c77de5f1b16378a9c6953735111ae8469877932fd5ebb370f6339b234df";
const GOLDEN_WINDOW: &str = "4104ba351e5e23aef861b4cc1e32b4208feee378111682a69caf0044213e67f9";
// Final case-file hash now also covers the `mode` field added in the
// Audit-vs-Throughput refactor and (post-R.5) the new entity_avg_q +
// grid_avg_q candidate fields. The audit-mode case file's final hash
// is the value pinned here.
const GOLDEN_FINAL: &str = "e895db09b77189e0b362c7647638cf79b1ebc53c82dca238aed2755a6c45272c";
const GOLDEN_EPISODE_COUNT: usize = 15;

fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(64);
    for b in bytes {
        write!(s, "{b:02x}").unwrap();
    }
    s
}

#[test]
fn golden_hashes_match_canonical_pipeline_output() {
    let events = synthesize(DEFAULT_SEED);
    let mut contract = Contract::canonical();
    contract.pin_bank_hash(bank_hash());
    contract.pin_detector_registry_hash(registry_hash());
    let case = build_cpu(&events, &contract);

    let h = &case.hashes;
    let pairs = [
        ("bank", hex(&h.bank), GOLDEN_BANK),
        (
            "candidate_interval",
            hex(&h.candidate_interval),
            GOLDEN_CANDIDATE,
        ),
        ("consensus_grid", hex(&h.consensus_grid), GOLDEN_CONSENSUS),
        ("contract", hex(&h.contract), GOLDEN_CONTRACT),
        ("detector_cell", hex(&h.detector_cell), GOLDEN_DETECTOR),
        (
            "detector_registry",
            hex(&h.detector_registry),
            GOLDEN_DETREG,
        ),
        ("episode", hex(&h.episode), GOLDEN_EPISODE),
        ("input_catalog", hex(&h.input_catalog), GOLDEN_INPUT),
        ("kernel_sequence", hex(&h.kernel_sequence), GOLDEN_KSEQ),
        ("residual_field", hex(&h.residual_field), GOLDEN_RESIDUAL),
        ("sign_field", hex(&h.sign_field), GOLDEN_SIGN),
        ("window_feature", hex(&h.window_feature), GOLDEN_WINDOW),
    ];

    let mut divergences: Vec<String> = Vec::new();
    for (name, actual, golden) in &pairs {
        if actual != golden {
            divergences.push(format!("{name}: expected {golden}, got {actual}"));
        }
    }
    assert!(
        divergences.is_empty(),
        "golden hash divergence:\n{}",
        divergences.join("\n")
    );

    assert_eq!(
        hex(&case.final_case_file_hash),
        GOLDEN_FINAL,
        "final case-file hash drifted from the recorded canonical value"
    );
    assert_eq!(case.episodes.len(), GOLDEN_EPISODE_COUNT);
}
