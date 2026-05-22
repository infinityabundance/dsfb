// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.7 acceptance tests: implementation-status (L-band) honesty.
//!
//! Panel-locked invariants (each test pins one rule):
//!
//! - `every_detector_has_exactly_one_lband` — schema guarantees a
//!   single L-band; this test asserts the seed exercises more than
//!   one band (no all-L1 fallback).
//! - `lband_histogram_sums_to_seed_count` — the histogram is a
//!   partition: no record counted twice, no record missing.
//! - `l5_l6_records_match_gpu_implemented_whitelist` — every record
//!   at L5/L6 has its canonical_id in `GPU_IMPLEMENTED_CANONICAL_IDS`
//!   (the dsfb-gpu-debug-core bank surface). Inverse direction is
//!   NOT asserted: whitelist members may legitimately be at lower
//!   bands during a campaign.
//! - `no_record_claims_l7_at_t7` — L7 (`BenchmarkCharacterised`) is
//!   forbidden in the corpus at T.7 because no per-detector
//!   benchmark artifact exists yet.
//! - `no_record_claims_l8_at_t7` — L8 (`LedgerCharacterised`) is
//!   forbidden in the corpus at T.7 because the usefulness ledger
//!   (T.8) has not landed.
//! - `lband_verify_clean_on_static_seed` — the static seed passes
//!   the T.7 verifier with zero errors.
//! - `lband_verify_rejects_forged_l5_for_non_gpu_implemented` — a
//!   synthesised record at L5 whose canonical_id is NOT in the
//!   whitelist must be rejected.
//! - `lband_verify_rejects_forged_l7` — any L7 record is rejected
//!   at T.7 regardless of canonical_id.
//! - `lband_verify_rejects_forged_l8` — any L8 record is rejected
//!   at T.7 regardless of canonical_id.
//! - `gpu_implemented_whitelist_is_sorted_and_deduplicated` — the
//!   load-bearing whitelist is canonically ordered and contains no
//!   duplicates.
//! - `gpu_implemented_whitelist_canonical_ids_resolve_in_seed` —
//!   every whitelist entry is present in the static seed (no
//!   dangling whitelist members).
//! - `report_contains_lband_invariant_block` — the public dedup
//!   report renders the T.7 invariant section.

use dsfb_gpu_atlas_corpus::lband::{
    compute_histogram, verify_corpus_lband, verify_record_lband, LBandErrorKind,
    GPU_IMPLEMENTED_CANONICAL_IDS,
};
use dsfb_gpu_atlas_corpus::report::render_report;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{DetectorCanonicalId, ImplementationLevel, LiteratureDetector};

#[test]
fn every_detector_has_exactly_one_lband() {
    // The enum schema guarantees a single L-band per record; this
    // test asserts the seed actually exercises more than one band
    // so the histogram is not a degenerate all-L1 row.
    let mut bands: Vec<ImplementationLevel> =
        SEED.iter().map(|r| r.implementation_status).collect();
    bands.sort_unstable_by_key(|b| *b as u8);
    bands.dedup();
    assert!(
        bands.len() >= 2,
        "expected at least 2 distinct L-bands in the seed; got {}",
        bands.len()
    );
}

#[test]
fn lband_histogram_sums_to_seed_count() {
    let h = compute_histogram(SEED);
    assert_eq!(
        h.total(),
        SEED.len(),
        "L-band histogram total ({}) must equal SEED length ({})",
        h.total(),
        SEED.len()
    );
}

#[test]
fn l5_l6_records_match_gpu_implemented_whitelist() {
    // Every record at L5 or L6 must have its canonical_id in the
    // GPU_IMPLEMENTED_CANONICAL_IDS whitelist. The inverse
    // direction (whitelist member => L5/L6) is intentionally NOT
    // asserted: a whitelist entry may legitimately sit at L1
    // during a campaign before its corpus seed entry lands.
    for r in SEED {
        match r.implementation_status {
            ImplementationLevel::L5_GpuImplemented
            | ImplementationLevel::L6_CpuGpuByteEquivalent => {
                assert!(
                    GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id),
                    "record `{}` (id {}) claims L5/L6 but is not in GPU_IMPLEMENTED_CANONICAL_IDS",
                    r.display_name,
                    r.canonical_id.0
                );
            }
            _ => {}
        }
    }
}

#[test]
fn no_record_claims_l7_at_t7() {
    for r in SEED {
        assert_ne!(
            r.implementation_status,
            ImplementationLevel::L7_BenchmarkCharacterised,
            "record `{}` claims L7 but T.7 forbids it (no per-detector benchmark artifact yet)",
            r.display_name
        );
    }
}

#[test]
fn no_record_claims_l8_at_t7() {
    for r in SEED {
        assert_ne!(
            r.implementation_status,
            ImplementationLevel::L8_LedgerCharacterised,
            "record `{}` claims L8 but T.7 forbids it (usefulness ledger T.8 has not landed)",
            r.display_name
        );
    }
}

#[test]
fn lband_verify_clean_on_static_seed() {
    let v = verify_corpus_lband(SEED);
    assert_eq!(v.records_inspected, SEED.len());
    assert!(
        v.is_clean(),
        "T.7 verifier produced {} errors on the static seed: {:?}",
        v.errors.len(),
        v.errors
    );
}

#[test]
fn lband_verify_rejects_forged_l5_for_non_gpu_implemented() {
    // Pick any non-whitelisted record (the seed has many L1 records);
    // forge it to L5; the verifier must reject. This pins the
    // anti-inflation property: claiming GPU implementation status
    // without a real GPU kernel is structurally caught.
    let baseline = SEED
        .iter()
        .find(|r| !GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id))
        .expect("seed must contain at least one non-whitelisted record");
    let mut forged = *baseline;
    forged.implementation_status = ImplementationLevel::L5_GpuImplemented;
    let errs = verify_record_lband(&forged);
    assert_eq!(
        errs.len(),
        1,
        "forged L5 record on non-whitelisted id must produce exactly one error"
    );
    match errs[0].kind {
        LBandErrorKind::GpuImplementationClaimedWithoutGpuSurface { claimed } => {
            assert_eq!(claimed, ImplementationLevel::L5_GpuImplemented);
        }
        ref other => panic!("expected GpuImplementationClaimedWithoutGpuSurface, got {other:?}"),
    }
}

#[test]
fn lband_verify_rejects_forged_l6_for_non_gpu_implemented() {
    let baseline = SEED
        .iter()
        .find(|r| !GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id))
        .expect("seed must contain at least one non-whitelisted record");
    let mut forged = *baseline;
    forged.implementation_status = ImplementationLevel::L6_CpuGpuByteEquivalent;
    let errs = verify_record_lband(&forged);
    assert_eq!(errs.len(), 1);
    match errs[0].kind {
        LBandErrorKind::GpuImplementationClaimedWithoutGpuSurface { claimed } => {
            assert_eq!(claimed, ImplementationLevel::L6_CpuGpuByteEquivalent);
        }
        ref other => panic!("expected GpuImplementationClaimedWithoutGpuSurface, got {other:?}"),
    }
}

#[test]
fn lband_verify_rejects_forged_l7() {
    // Even a whitelisted canonical_id cannot claim L7 at T.7
    // because L7 requires a benchmark artifact that does not yet
    // exist. Forge a whitelisted record up to L7 and confirm
    // rejection.
    let baseline = SEED
        .iter()
        .find(|r| GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id))
        .expect("seed must contain at least one whitelisted record");
    let mut forged = *baseline;
    forged.implementation_status = ImplementationLevel::L7_BenchmarkCharacterised;
    let errs = verify_record_lband(&forged);
    assert_eq!(errs.len(), 1);
    assert!(matches!(
        errs[0].kind,
        LBandErrorKind::BenchmarkCharacterisedWithoutArtifact
    ));
}

#[test]
fn lband_verify_rejects_forged_l8() {
    let baseline = SEED
        .iter()
        .find(|r| GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id))
        .expect("seed must contain at least one whitelisted record");
    let mut forged = *baseline;
    forged.implementation_status = ImplementationLevel::L8_LedgerCharacterised;
    let errs = verify_record_lband(&forged);
    assert_eq!(errs.len(), 1);
    assert!(matches!(
        errs[0].kind,
        LBandErrorKind::LedgerCharacterisedWithoutLedger
    ));
}

#[test]
fn gpu_implemented_whitelist_is_sorted_and_deduplicated() {
    // The load-bearing whitelist must be canonically ordered with
    // no duplicates so the report's rendering is deterministic and
    // a reviewer's diff against future commits is trivial.
    let ids: Vec<DetectorCanonicalId> = GPU_IMPLEMENTED_CANONICAL_IDS.to_vec();
    let mut sorted = ids.clone();
    sorted.sort_unstable_by_key(|c| c.0);
    assert_eq!(
        ids, sorted,
        "GPU_IMPLEMENTED_CANONICAL_IDS must be in ascending canonical-id order"
    );
    let mut deduped = sorted.clone();
    deduped.dedup();
    assert_eq!(
        deduped.len(),
        sorted.len(),
        "GPU_IMPLEMENTED_CANONICAL_IDS has duplicates"
    );
}

#[test]
fn gpu_implemented_whitelist_canonical_ids_resolve_in_seed() {
    // Every whitelist entry must point to a real seed record;
    // dangling whitelist members would silently inflate the L5/L6
    // ceiling without any corpus-side honesty story.
    for id in GPU_IMPLEMENTED_CANONICAL_IDS {
        let found = SEED.iter().any(|r| r.canonical_id == *id);
        assert!(
            found,
            "GPU_IMPLEMENTED_CANONICAL_IDS contains canonical_id {} which is absent from SEED",
            id.0
        );
    }
}

#[test]
fn report_contains_lband_invariant_block() {
    let body = render_report(SEED);
    assert!(
        body.contains("(13) L-band honesty invariants (T.7)"),
        "report must include the T.7 invariant section header"
    );
    assert!(
        body.contains("L-band is an honesty marker, not a quality score."),
        "report must carry the panel-locked honesty-marker line"
    );
    assert!(
        body.contains("GPU-implemented canonical IDs (load-bearing whitelist):"),
        "report must include the GPU-implemented whitelist block"
    );
    assert!(
        body.contains("Histogram (records per L-band):"),
        "report must include the L-band histogram block"
    );
}

#[test]
fn report_l7_l8_marked_forbidden_at_t7() {
    let body = render_report(SEED);
    assert!(
        body.contains("L7_BenchmarkCharacterised") && body.contains("forbidden at T.7"),
        "report must label L7 as forbidden at T.7"
    );
    assert!(
        body.contains("L8_LedgerCharacterised") && body.contains("forbidden at T.7"),
        "report must label L8 as forbidden at T.7"
    );
}

#[test]
fn all_whitelisted_records_actually_carry_l5_or_l6() {
    // Stronger sibling of the asymmetric test above: at T.7 every
    // whitelisted canonical_id in the seed IS at L5 or L6 (because
    // the bank kernel exists today). This is a snapshot of T.7
    // state, not a permanent invariant — a future campaign may
    // legitimately drop a whitelisted record's L-band during a
    // refactor. The test pins the current state and fails loudly
    // if it changes silently.
    for id in GPU_IMPLEMENTED_CANONICAL_IDS {
        let rec = SEED
            .iter()
            .find(|r| r.canonical_id == *id)
            .expect("whitelist resolution checked elsewhere");
        let band = rec.implementation_status;
        assert!(
            matches!(
                band,
                ImplementationLevel::L5_GpuImplemented
                    | ImplementationLevel::L6_CpuGpuByteEquivalent
            ),
            "whitelisted record `{}` (id {}) is at {:?}; T.7 snapshot expected L5 or L6",
            rec.display_name,
            id.0,
            band
        );
    }
}

#[test]
fn lband_verify_record_returns_empty_for_lband_at_or_below_l4() {
    // L0..L4 must always pass the T.7 verifier regardless of
    // canonical_id, because those bands make no GPU/benchmark/ledger
    // claim.
    let baseline = SEED[0];
    for band in [
        ImplementationLevel::L0_CitedOnly,
        ImplementationLevel::L1_Canonicalised,
        ImplementationLevel::L2_DeterministicFormula,
        ImplementationLevel::L3_CpuImplemented,
        ImplementationLevel::L4_CpuVerified,
    ] {
        let mut forged = baseline;
        forged.implementation_status = band;
        let errs = verify_record_lband(&forged);
        assert!(
            errs.is_empty(),
            "L-band {band:?} should never produce a T.7 error; got {errs:?}"
        );
    }
}

#[test]
fn lband_verify_corpus_aggregates_errors_in_canonical_order() {
    // If multiple records each produce errors, verify_corpus_lband
    // collects them in the seed's iteration order (no shuffling).
    // Pin that by constructing a small synthesised slice.
    let baseline = SEED[0];
    let mut a = baseline;
    a.canonical_id = DetectorCanonicalId(9001);
    a.implementation_status = ImplementationLevel::L7_BenchmarkCharacterised;
    let mut b = baseline;
    b.canonical_id = DetectorCanonicalId(9002);
    b.implementation_status = ImplementationLevel::L8_LedgerCharacterised;
    let records: [LiteratureDetector; 2] = [a, b];
    let report = verify_corpus_lband(&records);
    assert_eq!(report.errors.len(), 2);
    assert_eq!(report.errors[0].canonical_id.0, 9001);
    assert_eq!(report.errors[1].canonical_id.0, 9002);
}
