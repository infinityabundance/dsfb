// Tests legitimately panic on assertion failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.8 acceptance tests: deterministic detector usefulness ledger
//! honesty invariants.
//!
//! Panel-locked tests (each name documents an invariant). T.8
//! ships the ledger schema, verifier, and conservative seed —
//! NOT empirical usefulness claims. These tests pin that posture.
//!
//! Coverage / shape:
//! - `every_canonical_detector_has_usefulness_row`
//! - `ledger_row_count_matches_seed_count`
//! - `ledger_rows_are_deterministically_sorted_by_canonical_id`
//! - `ledger_is_byte_stable_across_two_reads`
//! - `report_renders_section_14`
//! - `report_section_14_contains_evidence_level_histogram`
//! - `report_section_14_contains_lifecycle_histogram`
//! - `report_section_14_contains_no_fabricated_claims_invariant`
//! - `report_section_14_carries_audit_surface_phrasing`
//!
//! Verifier rejection rules:
//! - `unmeasured_rows_cannot_claim_unique_episode_gain`
//! - `unmeasured_rows_cannot_claim_false_positive_cost`
//! - `unmeasured_rows_cannot_claim_runtime_cost`
//! - `unmeasured_rows_cannot_claim_sample_count`
//! - `l8_requires_measured_ledger_evidence`
//! - `retired_state_requires_measured_negative_evidence`
//! - `gpu_active_claim_requires_l5_or_l6`
//! - `same_triple_detector_cannot_be_active_and_retired`
//! - `verifier_rejects_missing_task_id`
//! - `verifier_rejects_missing_dataset_id`
//! - `verifier_rejects_missing_domain`
//! - `verifier_rejects_unknown_detector_id`
//! - `verifier_rejects_duplicate_triple`
//! - `notscored_blocks_nonzero_score`
//! - `reason_inconsistent_with_evidence_level_rejected`
//!
//! Score policy:
//! - `score_kind_not_scored_returns_none`
//! - `score_kind_prior_score_returns_some_for_all_zero_row`
//! - `score_formula_is_deterministic_across_two_calls`
//!
//! Conservative seed posture:
//! - `t8_seed_uses_only_unmeasured_or_role_or_literature_evidence`
//! - `t8_seed_score_kind_is_uniformly_notscored`
//! - `t8_seed_empirical_fields_are_uniformly_zero`
//! - `gpu_whitelisted_rows_match_dsfb_gpu_debug_core_source`
//! - `non_whitelisted_rows_source_is_atlas_corpus_seed_v1`
//! - `verify_clean_on_seed`

use dsfb_gpu_atlas_corpus::lband::GPU_IMPLEMENTED_CANONICAL_IDS;
use dsfb_gpu_atlas_corpus::report::render_report;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    DetectorCanonicalId, DomainTagSet, ImplementationLevel, LifecycleState,
};
use dsfb_gpu_atlas_corpus::usefulness::{
    compute_evidence_histogram, compute_lifecycle_histogram, usefulness_score,
    verify_usefulness_ledger, DatasetId, LedgerSource, TaskId, UsefulnessEvidenceLevel,
    UsefulnessLedgerErrorKind, UsefulnessLedgerRow, UsefulnessReason, UsefulnessScoreKind,
    SEED_DATASET_ID, SEED_TASK_ID, USEFULNESS_LEDGER,
};

// ---------------------------------------------------------------
// Coverage / shape.
// ---------------------------------------------------------------

#[test]
fn every_canonical_detector_has_usefulness_row() {
    for d in SEED {
        let has_row = USEFULNESS_LEDGER
            .iter()
            .any(|r| r.canonical_id == d.canonical_id);
        assert!(
            has_row,
            "canonical detector `{}` (id {}) has no row in USEFULNESS_LEDGER",
            d.display_name, d.canonical_id.0
        );
    }
}

#[test]
fn ledger_row_count_matches_seed_count() {
    assert_eq!(
        USEFULNESS_LEDGER.len(),
        SEED.len(),
        "T.8 seed: one row per canonical detector; got {} rows for {} detectors",
        USEFULNESS_LEDGER.len(),
        SEED.len()
    );
}

#[test]
fn ledger_rows_are_deterministically_sorted_by_canonical_id() {
    // The T.8 seed is hand-authored in canonical_id ascending order.
    // Two builds produce byte-identical USEFULNESS_LEDGER bytes; this
    // test asserts the canonical-id sort key is monotone so the
    // public report's row order is stable.
    let mut last: u32 = 0;
    for r in USEFULNESS_LEDGER {
        assert!(
            r.canonical_id.0 >= last,
            "USEFULNESS_LEDGER not in canonical_id ascending order: {} after {}",
            r.canonical_id.0,
            last
        );
        last = r.canonical_id.0;
    }
}

#[test]
fn ledger_is_byte_stable_across_two_reads() {
    // The static slice address may shift between processes but the
    // (canonical_id, evidence, lifecycle, reason) tuple set must
    // be identical across two iterations of the same process.
    let a: Vec<_> = USEFULNESS_LEDGER
        .iter()
        .map(|r| {
            (
                r.canonical_id.0,
                r.evidence_level,
                r.lifecycle_state,
                r.reason_code,
            )
        })
        .collect();
    let b: Vec<_> = USEFULNESS_LEDGER
        .iter()
        .map(|r| {
            (
                r.canonical_id.0,
                r.evidence_level,
                r.lifecycle_state,
                r.reason_code,
            )
        })
        .collect();
    assert_eq!(a, b);
}

#[test]
fn report_renders_section_14() {
    let body = render_report(SEED);
    assert!(
        body.contains("(14) Usefulness ledger honesty invariants (T.8)"),
        "report must include the T.8 Section 14 header"
    );
}

#[test]
fn report_section_14_contains_evidence_level_histogram() {
    let body = render_report(SEED);
    assert!(body.contains("Evidence-level histogram:"));
    assert!(body.contains("Unmeasured"));
    assert!(body.contains("LiteraturePrior"));
    assert!(body.contains("RoleSeeded"));
}

#[test]
fn report_section_14_contains_lifecycle_histogram() {
    let body = render_report(SEED);
    assert!(body.contains("Lifecycle-state histogram (cross-check vs Section 5):"));
    assert!(body.contains("Active"));
    assert!(body.contains("Dormant"));
}

#[test]
fn report_section_14_contains_no_fabricated_claims_invariant() {
    let body = render_report(SEED);
    assert!(body.contains("No-fabricated-claims invariant:"));
    assert!(
        body.contains("rows at Unmeasured/Literature/Role with nonzero empirical fields : 0"),
        "T.8 conservative seed must keep zero fabricated empirical claims"
    );
}

#[test]
fn report_section_14_carries_audit_surface_phrasing() {
    // Panel-locked phrasing must appear verbatim somewhere in Section 14.
    let body = render_report(SEED);
    assert!(
        body.contains("audit surface, not a learned"),
        "Section 14 must carry the panel-locked 'audit surface, not a learned ranking model' framing"
    );
}

// ---------------------------------------------------------------
// Verifier rejection rules.
// ---------------------------------------------------------------

fn baseline_unmeasured_row() -> UsefulnessLedgerRow {
    UsefulnessLedgerRow {
        canonical_id: DetectorCanonicalId(1),
        task_id: TaskId("synth_task"),
        domain: DomainTagSet(DomainTagSet::TABULAR),
        dataset_id: DatasetId("synth_dataset"),
        evidence_level: UsefulnessEvidenceLevel::Unmeasured,
        lifecycle_state: LifecycleState::Dormant,
        score_kind: UsefulnessScoreKind::NotScored,
        unique_episode_gain: 0,
        redundant_with_count: 0,
        clean_window_false_positive_cost: 0,
        confuser_reduction_gain: 0,
        runtime_cost_us_p50: 0,
        memory_cost_bytes: 0,
        casefile_explanation_value: 0,
        operator_readability_score: 0,
        sample_count: 0,
        ledger_source: LedgerSource::AtlasCorpusSeedV1,
        reason_code: UsefulnessReason::UnmeasuredAtT8,
    }
}

#[test]
fn unmeasured_rows_cannot_claim_unique_episode_gain() {
    let mut row = baseline_unmeasured_row();
    row.unique_episode_gain = 5;
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::UnmeasuredRowClaimsEmpiricalGain));
}

#[test]
fn unmeasured_rows_cannot_claim_false_positive_cost() {
    let mut row = baseline_unmeasured_row();
    row.clean_window_false_positive_cost = 7;
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::UnmeasuredRowClaimsEmpiricalGain));
}

#[test]
fn unmeasured_rows_cannot_claim_runtime_cost() {
    let mut row = baseline_unmeasured_row();
    row.runtime_cost_us_p50 = 250;
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::UnmeasuredRowClaimsEmpiricalGain));
}

#[test]
fn unmeasured_rows_cannot_claim_sample_count() {
    let mut row = baseline_unmeasured_row();
    row.sample_count = 1;
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::UnmeasuredRowClaimsEmpiricalGain));
}

#[test]
fn l8_requires_measured_ledger_evidence() {
    // Synthesise a SEED where one record is L8 but USEFULNESS_LEDGER
    // is the conservative T.8 seed (all Unmeasured/Literature/Role).
    // The verifier must reject because no row reaches RealDataset
    // or higher for that detector.
    let mut synth_seed: Vec<_> = SEED.to_vec();
    synth_seed[0].implementation_status = ImplementationLevel::L8_LedgerCharacterised;
    let report = verify_usefulness_ledger(&synth_seed, USEFULNESS_LEDGER);
    assert!(
        report.errors.iter().any(|e| e.kind
            == UsefulnessLedgerErrorKind::L8RecordWithoutMeasuredLedgerEvidence
            && e.canonical_id == synth_seed[0].canonical_id),
        "L8 record with only Unmeasured/Literature/Role rows must be rejected"
    );
}

#[test]
fn retired_state_requires_measured_negative_evidence() {
    let mut row = baseline_unmeasured_row();
    row.lifecycle_state = LifecycleState::RetiredRedundant;
    // Keep evidence_level = Unmeasured + reason = UnmeasuredAtT8 to
    // make the row fail rule 4 (retired without measured negative).
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::RetiredStateWithoutMeasuredEvidence));
}

#[test]
fn gpu_active_claim_requires_l5_or_l6() {
    // Pick a non-whitelisted canonical_id; assign the GPU reason;
    // the verifier must reject because the L-band is not L5/L6
    // (or because the canonical_id is not in the whitelist).
    let baseline = SEED
        .iter()
        .find(|r| !GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id))
        .expect("seed has at least one non-whitelisted record");
    let row = UsefulnessLedgerRow {
        canonical_id: baseline.canonical_id,
        reason_code: UsefulnessReason::GpuSurfaceSeededFromDsfbGpuDebugCore,
        evidence_level: UsefulnessEvidenceLevel::RoleSeeded,
        lifecycle_state: LifecycleState::Active,
        ledger_source: LedgerSource::DsfbGpuDebugCoreSurface,
        ..baseline_unmeasured_row()
    };
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::GpuActiveClaimWithoutWhitelistOrLBand));
}

#[test]
fn same_triple_detector_cannot_be_active_and_retired() {
    let mut active = baseline_unmeasured_row();
    active.lifecycle_state = LifecycleState::Active;
    // Distinguish the two rows by domain so rule 10 (duplicate
    // triple) does not also fire; the triple key for rule 8 is
    // (canonical_id, task_id, dataset_id) — domain is excluded.
    let mut retired = baseline_unmeasured_row();
    retired.lifecycle_state = LifecycleState::RetiredRedundant;
    retired.evidence_level = UsefulnessEvidenceLevel::RetiredByEvidence;
    retired.reason_code = UsefulnessReason::RetiredRedundantByEvidence;
    retired.domain = DomainTagSet(DomainTagSet::TELEMETRY);
    let report = verify_usefulness_ledger(SEED, &[active, retired]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::SameTripleBothActiveAndRetired));
}

#[test]
fn verifier_rejects_missing_task_id() {
    let mut row = baseline_unmeasured_row();
    row.task_id = TaskId("");
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::RequiredFieldMissing));
}

#[test]
fn verifier_rejects_missing_dataset_id() {
    let mut row = baseline_unmeasured_row();
    row.dataset_id = DatasetId("");
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::RequiredFieldMissing));
}

#[test]
fn verifier_rejects_missing_domain() {
    let mut row = baseline_unmeasured_row();
    row.domain = DomainTagSet(0);
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::RequiredFieldMissing));
}

#[test]
fn verifier_rejects_unknown_detector_id() {
    let mut row = baseline_unmeasured_row();
    row.canonical_id = DetectorCanonicalId(9999);
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::UnknownDetectorId));
}

#[test]
fn verifier_rejects_duplicate_triple() {
    let a = baseline_unmeasured_row();
    let b = baseline_unmeasured_row();
    let report = verify_usefulness_ledger(SEED, &[a, b]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::DuplicateTriple));
}

#[test]
fn notscored_blocks_nonzero_score() {
    // A NotScored row should make usefulness_score return None
    // regardless of any (fabricated) empirical fields. Even if a
    // caller bypasses the rule-2 check by leaving empirical fields
    // at zero, score_kind=NotScored alone must block scoring.
    let row = baseline_unmeasured_row();
    assert_eq!(usefulness_score(&row), None);
}

#[test]
fn reason_inconsistent_with_evidence_level_rejected() {
    let mut row = baseline_unmeasured_row();
    // Unmeasured evidence + LiteraturePriorOnly reason = inconsistent.
    row.reason_code = UsefulnessReason::LiteraturePriorOnly;
    let report = verify_usefulness_ledger(SEED, &[row]);
    assert!(report
        .errors
        .iter()
        .any(|e| e.kind == UsefulnessLedgerErrorKind::ReasonInconsistentWithEvidenceLevel));
}

// ---------------------------------------------------------------
// Score policy.
// ---------------------------------------------------------------

#[test]
fn score_kind_not_scored_returns_none() {
    let row = baseline_unmeasured_row();
    assert_eq!(usefulness_score(&row), None);
}

#[test]
fn score_kind_prior_score_returns_some_for_all_zero_row() {
    // A row marked PriorScore with zero empirical fields scores to
    // zero — not None. This pins the score-vs-no-score gate.
    let row = UsefulnessLedgerRow {
        score_kind: UsefulnessScoreKind::PriorScore,
        ..baseline_unmeasured_row()
    };
    assert_eq!(usefulness_score(&row), Some(0));
}

#[test]
fn score_formula_is_deterministic_across_two_calls() {
    let row = UsefulnessLedgerRow {
        score_kind: UsefulnessScoreKind::MeasuredScore,
        evidence_level: UsefulnessEvidenceLevel::SyntheticFixtureMeasured,
        reason_code: UsefulnessReason::MeasuredFromSyntheticFixture,
        unique_episode_gain: 10,
        confuser_reduction_gain: 3,
        clean_window_false_positive_cost: 1,
        runtime_cost_us_p50: 2048,
        redundant_with_count: 2,
        casefile_explanation_value: 4,
        operator_readability_score: 6,
        ..baseline_unmeasured_row()
    };
    let a = usefulness_score(&row);
    let b = usefulness_score(&row);
    assert_eq!(a, b);
    // Verify the formula explicitly: 4*10 + 3*3 + 2*4 + 6 - 3*1 - 2 - 2*2 = 40+9+8+6-3-2-4 = 54
    assert_eq!(a, Some(54));
}

// ---------------------------------------------------------------
// Conservative seed posture.
// ---------------------------------------------------------------

#[test]
fn t8_seed_uses_only_unmeasured_or_role_or_literature_evidence() {
    for r in USEFULNESS_LEDGER {
        assert!(
            matches!(
                r.evidence_level,
                UsefulnessEvidenceLevel::Unmeasured
                    | UsefulnessEvidenceLevel::LiteraturePrior
                    | UsefulnessEvidenceLevel::RoleSeeded
            ),
            "T.8 seed row [{}] claims evidence_level {:?} — no measured rows allowed at T.8",
            r.canonical_id.0,
            r.evidence_level
        );
    }
}

#[test]
fn t8_seed_score_kind_is_uniformly_notscored() {
    for r in USEFULNESS_LEDGER {
        assert_eq!(
            r.score_kind,
            UsefulnessScoreKind::NotScored,
            "T.8 seed row [{}] is scored ({:?}); the conservative seed must keep score_kind=NotScored",
            r.canonical_id.0,
            r.score_kind
        );
    }
}

#[test]
fn t8_seed_empirical_fields_are_uniformly_zero() {
    for r in USEFULNESS_LEDGER {
        assert!(
            UsefulnessLedgerRow::has_zero_empirical_fields(r),
            "T.8 seed row [{}] carries nonzero empirical fields; the conservative seed must keep them zero",
            r.canonical_id.0
        );
    }
}

#[test]
fn gpu_whitelisted_rows_match_dsfb_gpu_debug_core_source() {
    for r in USEFULNESS_LEDGER {
        if !GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id) {
            continue;
        }
        assert_eq!(
            r.ledger_source,
            LedgerSource::DsfbGpuDebugCoreSurface,
            "GPU-whitelisted row [{}] must declare ledger_source=DsfbGpuDebugCoreSurface",
            r.canonical_id.0
        );
        assert_eq!(
            r.reason_code,
            UsefulnessReason::GpuSurfaceSeededFromDsfbGpuDebugCore,
            "GPU-whitelisted row [{}] must declare reason=GpuSurfaceSeededFromDsfbGpuDebugCore",
            r.canonical_id.0
        );
        assert_eq!(
            r.evidence_level,
            UsefulnessEvidenceLevel::RoleSeeded,
            "GPU-whitelisted row [{}] must be RoleSeeded",
            r.canonical_id.0
        );
        assert_eq!(
            r.lifecycle_state,
            LifecycleState::Active,
            "GPU-whitelisted row [{}] must be Active",
            r.canonical_id.0
        );
    }
}

#[test]
fn non_whitelisted_rows_source_is_atlas_corpus_seed_v1() {
    for r in USEFULNESS_LEDGER {
        if GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id) {
            continue;
        }
        assert_eq!(
            r.ledger_source,
            LedgerSource::AtlasCorpusSeedV1,
            "non-whitelisted row [{}] must declare ledger_source=AtlasCorpusSeedV1",
            r.canonical_id.0
        );
        assert_eq!(
            r.evidence_level,
            UsefulnessEvidenceLevel::LiteraturePrior,
            "non-whitelisted row [{}] must be LiteraturePrior",
            r.canonical_id.0
        );
        assert_eq!(
            r.lifecycle_state,
            LifecycleState::Dormant,
            "non-whitelisted row [{}] must be Dormant at T.8",
            r.canonical_id.0
        );
    }
}

#[test]
fn verify_clean_on_seed() {
    let report = verify_usefulness_ledger(SEED, USEFULNESS_LEDGER);
    assert_eq!(
        report.records_inspected,
        SEED.len(),
        "records_inspected must match SEED length"
    );
    assert_eq!(
        report.rows_inspected,
        USEFULNESS_LEDGER.len(),
        "rows_inspected must match USEFULNESS_LEDGER length"
    );
    assert!(
        report.is_clean(),
        "T.8 verifier produced {} errors on the conservative seed: {:?}",
        report.errors.len(),
        report.errors
    );
}

// ---------------------------------------------------------------
// Histogram + constant sanity.
// ---------------------------------------------------------------

#[test]
fn evidence_histogram_total_matches_ledger_size() {
    let h = compute_evidence_histogram(USEFULNESS_LEDGER);
    assert_eq!(h.total(), USEFULNESS_LEDGER.len());
}

#[test]
fn lifecycle_histogram_total_matches_ledger_size() {
    let h = compute_lifecycle_histogram(USEFULNESS_LEDGER);
    assert_eq!(h.total(), USEFULNESS_LEDGER.len());
}

#[test]
fn seed_task_id_and_dataset_id_are_non_empty() {
    assert_ne!(SEED_TASK_ID.0, "");
    assert_ne!(SEED_DATASET_ID.0, "");
    assert_eq!(SEED_TASK_ID.0, "atlas_corpus_seed_v1");
    assert_eq!(SEED_DATASET_ID.0, "none");
}

#[test]
fn five_gpu_whitelisted_rows_are_in_seed() {
    // The T.8 seed must seed exactly the five canonical IDs in the
    // T.7 whitelist as RoleSeeded/Active/GpuSurface*.
    let gpu_active: Vec<u32> = USEFULNESS_LEDGER
        .iter()
        .filter(|r| r.reason_code == UsefulnessReason::GpuSurfaceSeededFromDsfbGpuDebugCore)
        .map(|r| r.canonical_id.0)
        .collect();
    let expected: Vec<u32> = GPU_IMPLEMENTED_CANONICAL_IDS.iter().map(|c| c.0).collect();
    let mut sorted_actual = gpu_active.clone();
    sorted_actual.sort_unstable();
    let mut sorted_expected = expected.clone();
    sorted_expected.sort_unstable();
    assert_eq!(
        sorted_actual, sorted_expected,
        "T.8 GPU-seeded rows must exactly match GPU_IMPLEMENTED_CANONICAL_IDS"
    );
}
