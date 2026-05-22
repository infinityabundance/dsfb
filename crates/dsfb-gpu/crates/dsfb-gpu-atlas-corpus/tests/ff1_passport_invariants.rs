//! FF.1 acceptance suite — DetectorPassport materialisation
//! invariants for corpus_hash_v2-ratified entries.
//!
//! Ten panel-required load-bearing negatives pin the contract
//! discipline FF.1 exists to prove:
//!
//! * `ff1_rejects_passport_for_non_ratified_canonical_id`
//! * `ff1_rejects_passport_if_corpus_hash_v2_mismatch`
//! * `ff1_rejects_passport_materialisation_that_mutates_t12_proposal_hash`
//! * `ff1_rejects_passport_materialisation_that_mutates_corpus_hash_v2`
//! * `ff1_rejects_duplicate_passport_for_same_canonical_id`
//! * `ff1_rejects_missing_source_lineage_for_literature_passport`
//! * `ff1_rejects_missing_gpu_family_mapping`
//! * `ff1_rejects_missing_activation_applicability_tags`
//! * `ff1_rejects_missing_contraindication_linkage_stub`
//! * `ff1_rejects_missing_challenge_surface_stub`
//!
//! Panel-locked non-claim (verbatim):
//!
//! > FF.1 materializes DetectorPassport records for Accepted
//! > T.12 expansion entries ratified by corpus_hash_v2. It does
//! > not reopen T.12 dedup decisions, add new literature
//! > primitives, alter corpus_hash_v1, alter corpus_hash_v2, or
//! > rewrite historical proposal hashes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::amendment::SourceClass;
use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::{
    build_ff1_materialisation_report, build_ff1_passport_index, build_ff1_passport_index_from,
    render_ff1_materialisation_report_json, render_ff1_materialisation_report_text,
    render_ff1_passport_index_json, render_ff1_passport_index_text,
    source_class_to_activation_tags, source_class_to_gpu_family, verify_ff1, ChallengeStub,
    ContraindicationStub, Ff1VerifyErrorKind, T12RatifiedPassport,
    FF1_MATERIALISATION_REPORT_DOMAIN_V1, FF1_PASSPORT_INDEX_DOMAIN_V1,
    FF1_T12_RATIFIED_PASSPORT_DOMAIN_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::GpuFamilyKernel;

// ---------------------------------------------------------------
// Shape + materialisation discipline
// ---------------------------------------------------------------

#[test]
fn ff1_materialises_one_passport_per_ratified_canonical_addition() {
    let report = build_consolidation_report();
    let idx = build_ff1_passport_index();
    assert_eq!(idx.passports.len(), report.expansion_index.len());
}

#[test]
fn ff1_total_passport_count_is_98() {
    let r = build_ff1_materialisation_report();
    assert_eq!(r.total_passport_count, 98);
}

#[test]
fn ff1_passports_sorted_ascending_by_canonical_id() {
    let idx = build_ff1_passport_index();
    let mut prev: i64 = -1;
    for p in &idx.passports {
        assert!((p.canonical_id as i64) > prev);
        prev = p.canonical_id as i64;
    }
}

#[test]
fn ff1_admissible_set_produces_zero_verify_errors() {
    let report = build_consolidation_report();
    let idx = build_ff1_passport_index();
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.is_empty(),
        "FF.1 admissible set must produce zero errors: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Hash invariance (FF.1 does NOT mutate any upstream anchor)
// ---------------------------------------------------------------

#[test]
fn ff1_does_not_mutate_corpus_hash_v1() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_ff1_passport_index();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

#[test]
fn ff1_does_not_mutate_corpus_hash_v2() {
    let report_before = build_consolidation_report();
    let _ = build_ff1_passport_index();
    let report_after = build_consolidation_report();
    assert_eq!(report_before.corpus_hash_v2, report_after.corpus_hash_v2);
}

#[test]
fn ff1_does_not_mutate_t12_expansion_index_hash_v1() {
    let report_before = build_consolidation_report();
    let _ = build_ff1_passport_index();
    let report_after = build_consolidation_report();
    assert_eq!(
        report_before.t12_expansion_index_hash_v1,
        report_after.t12_expansion_index_hash_v1
    );
}

#[test]
fn ff1_does_not_mutate_consolidation_report_hash_v1() {
    let report_before = build_consolidation_report();
    let _ = build_ff1_passport_index();
    let report_after = build_consolidation_report();
    assert_eq!(
        report_before.consolidation_report_hash_v1,
        report_after.consolidation_report_hash_v1
    );
}

#[test]
fn ff1_does_not_mutate_t12_proposal_hashes() {
    let report_before = build_consolidation_report();
    let _ = build_ff1_passport_index();
    let report_after = build_consolidation_report();
    for (b, a) in report_before
        .proposals
        .iter()
        .zip(report_after.proposals.iter())
    {
        assert_eq!(b.proposal_hash, a.proposal_hash);
        assert_eq!(b.batch_hash, a.batch_hash);
        assert_eq!(b.dedup_delta_hash, a.dedup_delta_hash);
    }
}

#[test]
fn ff1_seed_len_remains_54() {
    let _ = build_ff1_passport_index();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn ff1_passport_index_pins_all_four_anchors() {
    let report = build_consolidation_report();
    let idx = build_ff1_passport_index();
    assert_eq!(idx.corpus_hash_v1, report.corpus_hash_v1);
    assert_eq!(idx.corpus_hash_v2, report.corpus_hash_v2);
    assert_eq!(
        idx.t12_expansion_index_hash_v1,
        report.t12_expansion_index_hash_v1
    );
    assert_eq!(
        idx.consolidation_report_hash_v1,
        report.consolidation_report_hash_v1
    );
    assert_eq!(idx.seed_len, 54);
}

// ---------------------------------------------------------------
// FF.1 new own-namespace hash discipline
// ---------------------------------------------------------------

#[test]
fn ff1_passport_index_hash_is_deterministic() {
    let a = build_ff1_passport_index();
    let b = build_ff1_passport_index();
    assert_eq!(a.ff1_passport_index_hash_v1, b.ff1_passport_index_hash_v1);
}

#[test]
fn ff1_materialisation_report_hash_is_deterministic() {
    let a = build_ff1_materialisation_report();
    let b = build_ff1_materialisation_report();
    assert_eq!(
        a.ff1_materialisation_report_hash_v1,
        b.ff1_materialisation_report_hash_v1
    );
}

#[test]
fn ff1_passport_index_hash_is_nonzero() {
    let idx = build_ff1_passport_index();
    assert!(idx.ff1_passport_index_hash_v1.iter().any(|b| *b != 0));
}

#[test]
fn ff1_materialisation_report_hash_is_nonzero() {
    let r = build_ff1_materialisation_report();
    assert!(r.ff1_materialisation_report_hash_v1.iter().any(|b| *b != 0));
}

#[test]
fn ff1_new_hashes_distinct_from_upstream_anchors() {
    let r = build_ff1_materialisation_report();
    let idx = &r.passport_index;
    assert_ne!(idx.ff1_passport_index_hash_v1, idx.corpus_hash_v1);
    assert_ne!(idx.ff1_passport_index_hash_v1, idx.corpus_hash_v2);
    assert_ne!(
        idx.ff1_passport_index_hash_v1,
        idx.t12_expansion_index_hash_v1
    );
    assert_ne!(
        idx.ff1_passport_index_hash_v1,
        idx.consolidation_report_hash_v1
    );
    assert_ne!(
        r.ff1_materialisation_report_hash_v1,
        idx.ff1_passport_index_hash_v1
    );
}

#[test]
fn every_passport_carries_unique_passport_hash_v1() {
    let idx = build_ff1_passport_index();
    let mut seen: std::collections::BTreeSet<[u8; 32]> = std::collections::BTreeSet::new();
    for p in &idx.passports {
        assert!(
            seen.insert(p.passport_hash_v1),
            "passport_hash_v1 collision at canonical_id {}",
            p.canonical_id
        );
    }
}

// ---------------------------------------------------------------
// Domain-separator pins
// ---------------------------------------------------------------

#[test]
fn ff1_domain_separators_are_panel_locked() {
    assert_eq!(
        FF1_T12_RATIFIED_PASSPORT_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0"
    );
    assert_eq!(
        FF1_PASSPORT_INDEX_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0"
    );
    assert_eq!(
        FF1_MATERIALISATION_REPORT_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0"
    );
}

// ---------------------------------------------------------------
// Per-passport field invariants
// ---------------------------------------------------------------

#[test]
fn every_passport_has_canonical_id_in_t12_reserved_bands() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(
            (5001..=6699).contains(&p.canonical_id),
            "passport canonical_id {} outside T.12.a..T.12.p reserved bands",
            p.canonical_id
        );
    }
}

#[test]
fn no_passport_collides_with_seed_canonical_ids() {
    let idx = build_ff1_passport_index();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for p in &idx.passports {
        assert!(!seed_ids.contains(&p.canonical_id));
    }
}

#[test]
fn every_passport_has_nonempty_display_name() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(!p.display_name.is_empty());
        assert_ne!(p.display_name, "(uncredited)");
    }
}

#[test]
fn every_passport_has_nonempty_source_class() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(!p.source_class_wire_name.is_empty());
    }
}

#[test]
fn every_passport_has_nonempty_origin_proposal_id() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(!p.origin_proposal_id.is_empty());
    }
}

#[test]
fn every_passport_has_nonempty_gpu_family_wire_name() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(!p.gpu_family_wire_name.is_empty());
    }
}

#[test]
fn every_passport_has_nonempty_activation_tags() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(!p.activation_applicability_tags.is_empty());
    }
}

#[test]
fn every_passport_contraindication_stub_is_declared() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(p.contraindication_linkage_stub.stub_declared);
        assert_eq!(
            p.contraindication_linkage_stub
                .linked_contraindication_ids
                .len(),
            0
        );
    }
}

#[test]
fn every_passport_challenge_stub_is_declared() {
    let idx = build_ff1_passport_index();
    for p in &idx.passports {
        assert!(p.challenge_surface_stub.stub_declared);
        assert_eq!(p.challenge_surface_stub.linked_challenge_ids.len(), 0);
    }
}

// ---------------------------------------------------------------
// SourceClass mapping discipline
// ---------------------------------------------------------------

#[test]
fn source_class_to_gpu_family_is_total() {
    // Spot-check every panel-locked SourceClass variant returns
    // a non-NegativeWitnessFamily GpuFamilyKernel.
    let classes = [
        SourceClass::StatisticalProcessControl,
        SourceClass::SequentialChangeDetection,
        SourceClass::DriftDetection,
        SourceClass::RobustStatistics,
        SourceClass::DistributionDistance,
        SourceClass::InformationTheory,
        SourceClass::SignalProcessing,
        SourceClass::SpectralAndWavelet,
        SourceClass::TimeSeriesStructure,
        SourceClass::ControlResiduals,
        SourceClass::FaultDetectionDiagnostics,
        SourceClass::ConditionMonitoring,
        SourceClass::IndustrialProcessMonitoring,
        SourceClass::GraphAnomalyDetection,
        SourceClass::StreamingSketches,
        SourceClass::DataQualityRules,
        SourceClass::DatabaseIntegrityConstraints,
        SourceClass::ObservabilityDebugging,
        SourceClass::MedicalBiosignal,
        SourceClass::RfCommunications,
        SourceClass::Chemometrics,
        SourceClass::Econometrics,
        SourceClass::ReliabilitySurvival,
    ];
    for class in classes {
        let family = source_class_to_gpu_family(class);
        assert!(!matches!(family, GpuFamilyKernel::NegativeWitnessFamily));
        assert!(!family.as_str().is_empty());
    }
}

#[test]
fn source_class_to_activation_tags_is_total_and_nonempty() {
    let classes = [
        SourceClass::StatisticalProcessControl,
        SourceClass::InformationTheory,
        SourceClass::StreamingSketches,
        SourceClass::GraphAnomalyDetection,
        SourceClass::MedicalBiosignal,
    ];
    for class in classes {
        let tags = source_class_to_activation_tags(class);
        assert!(!tags.is_empty());
        for t in tags {
            assert!(!t.is_empty());
        }
    }
}

// ---------------------------------------------------------------
// Aggregate per-source-class count regression sentinel
// ---------------------------------------------------------------

#[test]
fn ff1_per_source_class_counts_sum_to_total() {
    let r = build_ff1_materialisation_report();
    let sum: u32 = r
        .per_source_class_counts
        .iter()
        .map(|c| c.passport_count)
        .sum();
    assert_eq!(sum, r.total_passport_count);
}

#[test]
fn ff1_per_source_class_counts_sorted_by_wire_name() {
    let r = build_ff1_materialisation_report();
    let mut prev: Option<&str> = None;
    for c in &r.per_source_class_counts {
        if let Some(p) = prev {
            assert!(p < c.source_class_wire_name);
        }
        prev = Some(c.source_class_wire_name);
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #1
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_passport_for_non_ratified_canonical_id() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    // Inject a passport for canonical_id 9999 — NOT in the
    // ratified expansion index.
    let bad_passport = T12RatifiedPassport {
        canonical_id: 9999,
        display_name: "defective-non-ratified-passport",
        source_class_wire_name: "InformationTheory",
        origin_proposal_id: "ff1_test_non_ratified",
        gpu_family_wire_name: "DistributionDistanceFamily",
        activation_applicability_tags: &["DataDrift"],
        contraindication_linkage_stub: ContraindicationStub {
            stub_declared: true,
            linked_contraindication_ids: &[],
        },
        challenge_surface_stub: ChallengeStub {
            stub_declared: true,
            linked_challenge_ids: &[],
        },
        passport_hash_v1: [0xCDu8; 32],
    };
    idx.passports.push(bad_passport);
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::PassportForNonRatifiedCanonicalId { canonical_id: 9999 }
        )),
        "non-ratified canonical_id 9999 must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #2
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_passport_if_corpus_hash_v2_mismatch() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    // Mutate the pinned corpus_hash_v2 to a non-matching value.
    idx.corpus_hash_v2 = [0xAAu8; 32];
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::PassportIfCorpusHashV2Mismatch { .. }
        )),
        "mutated corpus_hash_v2 must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #3
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_passport_materialisation_that_mutates_t12_proposal_hash() {
    let mut report = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&report);
    // Mutate the first proposal_summary's proposal_hash to the
    // all-zero sentinel (the test-mutation case that surfaces
    // the rule).
    report.proposals[0].proposal_hash = [0u8; 32];
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::PassportMaterialisationMutatedT12ProposalHash { .. }
        )),
        "all-zero proposal_hash must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #4
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_passport_materialisation_that_mutates_corpus_hash_v2() {
    let mut report = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&report);
    // Mutate the report's corpus_hash_v2 to the all-zero
    // sentinel.
    report.corpus_hash_v2 = [0u8; 32];
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::PassportMaterialisationMutatedCorpusHashV2
        )),
        "all-zero corpus_hash_v2 must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #5
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_duplicate_passport_for_same_canonical_id() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    // Duplicate the first passport.
    let dup = idx.passports[0].clone();
    idx.passports.push(dup);
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::DuplicatePassportForSameCanonicalId { .. }
        )),
        "duplicate passport must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #6
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_missing_source_lineage_for_literature_passport() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    // Mutate the first passport to clear its source_class_wire_name.
    idx.passports[0].source_class_wire_name = "";
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::MissingSourceLineageForLiteraturePassport { .. }
        )),
        "missing source lineage must surface: {errors:?}"
    );
    // Also test with empty origin_proposal_id.
    let mut idx2 = build_ff1_passport_index();
    idx2.passports[0].origin_proposal_id = "";
    let errors2 = verify_ff1(&idx2, &report);
    assert!(
        errors2.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::MissingSourceLineageForLiteraturePassport { .. }
        )),
        "missing origin_proposal_id must surface: {errors2:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #7
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_missing_gpu_family_mapping() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    idx.passports[0].gpu_family_wire_name = "";
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, Ff1VerifyErrorKind::MissingGpuFamilyMapping { .. })),
        "missing GPU family must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #8
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_missing_activation_applicability_tags() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    idx.passports[0].activation_applicability_tags = &[];
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::MissingActivationApplicabilityTags { .. }
        )),
        "missing activation tags must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #9
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_missing_contraindication_linkage_stub() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    idx.passports[0].contraindication_linkage_stub.stub_declared = false;
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::MissingContraindicationLinkageStub { .. }
        )),
        "undeclared contraindication stub must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #10
// ---------------------------------------------------------------

#[test]
fn ff1_rejects_missing_challenge_surface_stub() {
    let report = build_consolidation_report();
    let mut idx = build_ff1_passport_index();
    idx.passports[0].challenge_surface_stub.stub_declared = false;
    let errors = verify_ff1(&idx, &report);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            Ff1VerifyErrorKind::MissingChallengeSurfaceStub { .. }
        )),
        "undeclared challenge stub must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn ff1_passport_index_text_rendering_byte_stable() {
    let idx = build_ff1_passport_index();
    assert_eq!(
        render_ff1_passport_index_text(&idx),
        render_ff1_passport_index_text(&idx)
    );
}

#[test]
fn ff1_passport_index_json_rendering_byte_stable() {
    let idx = build_ff1_passport_index();
    assert_eq!(
        render_ff1_passport_index_json(&idx),
        render_ff1_passport_index_json(&idx)
    );
}

#[test]
fn ff1_materialisation_report_text_rendering_byte_stable() {
    let r = build_ff1_materialisation_report();
    assert_eq!(
        render_ff1_materialisation_report_text(&r),
        render_ff1_materialisation_report_text(&r)
    );
}

#[test]
fn ff1_materialisation_report_json_rendering_byte_stable() {
    let r = build_ff1_materialisation_report();
    assert_eq!(
        render_ff1_materialisation_report_json(&r),
        render_ff1_materialisation_report_json(&r)
    );
}

#[test]
fn ff1_materialisation_report_text_carries_panel_locked_non_claims() {
    let r = build_ff1_materialisation_report();
    let text = render_ff1_materialisation_report_text(&r);
    assert!(text.contains("FF.1 does NOT reopen T.12 dedup decisions"));
    assert!(text.contains("FF.1 does NOT add new literature primitives"));
    assert!(text.contains("FF.1 does NOT alter corpus_hash_v1"));
    assert!(text.contains("FF.1 does NOT alter corpus_hash_v2"));
    assert!(text.contains("FF.1 does NOT rewrite historical T.12 proposal hashes"));
    assert!(text.contains("FF.1 does NOT rewrite any T.12.consolidate hash"));
    assert!(text.contains("FF.1 does NOT mutate SEED.len()"));
    assert!(text.contains("FF.1 does NOT activate any detector"));
    assert!(text.contains("FF.1 does NOT decide contraindications or challenges"));
    assert!(text.contains("FF.1 does NOT generate CUDA kernels"));
}
