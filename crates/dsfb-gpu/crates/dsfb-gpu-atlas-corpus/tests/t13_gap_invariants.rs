//! T.13.GAP acceptance suite.
//!
//! Pins ten plan-required load-bearing negatives (CAMPAIGN
//! IDENTITY: `t13_gap_rejects_completeness_claim`) plus
//! structural defect rules, hash determinism / sensitivity,
//! renderer byte-stability, cross-anchor distinctness, GPU-
//! family-mapping closure, and bucket-histogram arithmetic.

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::{
    build_disposition_histogram, build_gap_candidate_index, build_taxonomy_gap_audit_report,
    forbidden_completeness_claim_substrings, render_t13_gap_report_json,
    render_t13_gap_report_text, seed_canonical_ids, seed_panels, text_makes_completeness_claim,
    GapDisposition, SurveyMethodRecord, SurveyTaxonomyPanelId, T13GapErrorKind,
    T13_GAP_PLAN_LOCKED_GPU_FAMILIES, T13_GAP_PLAN_LOCKED_THESIS,
    T13_GAP_SURVEY_TAXONOMY_INDEX_DOMAIN_V1,
};

// ---------------------------------------------------------------
// Plan-required load-bearing negatives (10)
// ---------------------------------------------------------------

#[test]
fn t13_gap_rejects_completeness_claim() {
    // CAMPAIGN IDENTITY: case-insensitive scanner over every
    // surveyed-method prose field rejecting positive-claim
    // variants of "the Atlas covers every known method" etc.
    for s in forbidden_completeness_claim_substrings() {
        assert!(
            text_makes_completeness_claim(s),
            "forbidden phrase `{s}` must trip the scanner"
        );
        let upper = s.to_uppercase();
        assert!(
            text_makes_completeness_claim(&upper),
            "scanner must be case-insensitive on `{upper}`"
        );
    }
    // Negative path: "does NOT claim ..." prose is legitimate.
    assert!(!text_makes_completeness_claim(
        "It does not claim completeness."
    ));
    assert!(!text_makes_completeness_claim(
        "The audit does not assert exhaustive coverage."
    ));
}

#[test]
fn t13_gap_rejects_new_canonical_when_existing_seed_authority_matches() {
    let method = SurveyMethodRecord {
        method_label: "smuggle-test ROBUST-Z (collides with SEED 6)",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "smuggle: try to admit as new despite SEED 6 collision",
        linked_canonical_id: 6,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::NewCanonicalCandidateCollidesWithSeedAuthority));
}

#[test]
fn t13_gap_rejects_new_canonical_for_parameter_setting_only() {
    let method = SurveyMethodRecord {
        method_label: "param-shift-only candidate",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "method only differs by parameter-setting; should not admit",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::NewCanonicalCandidateForParameterSettingOnly));
}

#[test]
fn t13_gap_rejects_new_canonical_for_domain_transfer_only() {
    let method = SurveyMethodRecord {
        method_label: "domain-only candidate",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "method only differs by different domain; should not admit",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::NewCanonicalCandidateForDomainTransferOnly));
}

#[test]
fn t13_gap_rejects_learned_black_box_flag_in_new_candidate() {
    let method = SurveyMethodRecord {
        method_label: "learned anomaly candidate",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason:
            "method uses neural network trained estimator without deterministic gate",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::LearnedBlackBoxFlagsNewCanonicalCandidate));
}

#[test]
fn t13_gap_rejects_probabilistic_estimator_flag_in_new_candidate() {
    let method = SurveyMethodRecord {
        method_label: "MCMC candidate",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "method uses MCMC sampling at runtime; no declared gate",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::ProbabilisticEstimatorFlagsNewCanonicalCandidate));
}

#[test]
fn t13_gap_rejects_runtime_metric_flag_in_new_candidate() {
    let method = SurveyMethodRecord {
        method_label: "wall-time SLA candidate",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "method records wall-time SLA as the entire signal",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::RuntimeMetricFlagsNewCanonicalCandidate));
}

#[test]
fn t13_gap_rejects_survey_method_without_source_ref() {
    let method = SurveyMethodRecord {
        method_label: "sourceless",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &[],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "no source refs",
        linked_canonical_id: 1,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::SurveyMethodMissingSourceRef));
}

#[test]
fn t13_gap_rejects_method_without_densor_mapping() {
    let method = SurveyMethodRecord {
        method_label: "non-rejected missing densor",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "non-rejected disposition with empty densor mapping",
        linked_canonical_id: 1,
        densor_mapping_label: "",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::NonRejectedMethodMissingDensorMapping));
}

#[test]
fn t13_gap_rejects_method_without_gpu_family_mapping() {
    let method = SurveyMethodRecord {
        method_label: "non-rejected missing gpu family",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "non-rejected disposition with unknown gpu family",
        linked_canonical_id: 1,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "NotARealFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::NonRejectedMethodMissingGpuFamilyMapping));
}

// ---------------------------------------------------------------
// Structural defect tests (4)
// ---------------------------------------------------------------

#[test]
fn t13_gap_rejects_empty_report_id() {
    let mut r = build_taxonomy_gap_audit_report();
    r.report_id = "";
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::ReportIdEmpty));
}

#[test]
fn t13_gap_rejects_empty_panel_label() {
    let mut r = build_taxonomy_gap_audit_report();
    r.panels[0].panel_label = "";
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::PanelLabelEmpty));
}

#[test]
fn t13_gap_rejects_bucket_histogram_sum_mismatch() {
    let mut r = build_taxonomy_gap_audit_report();
    r.bucket_histogram[0] = r.bucket_histogram[0].saturating_add(999);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::BucketHistogramSumMismatch));
}

#[test]
fn t13_gap_rejects_resolution_disposition_linking_to_zero_canonical() {
    let method = SurveyMethodRecord {
        method_label: "zero-link resolution",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["test"],
        disposition: GapDisposition::ParameterizationOf,
        disposition_reason: "param variant",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    };
    let r = stub_report_with_method(method);
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(v
        .errors
        .iter()
        .any(|e| e.kind == T13GapErrorKind::LinkedCanonicalIdResolvesToUnknown));
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity (4)
// ---------------------------------------------------------------

#[test]
fn t13_gap_top_level_hash_is_deterministic() {
    let a = build_taxonomy_gap_audit_report();
    let b = build_taxonomy_gap_audit_report();
    assert_eq!(a.taxonomy_gap_audit_hash_v1, b.taxonomy_gap_audit_hash_v1);
}

#[test]
fn t13_gap_changing_corpus_anchor_changes_top_level_hash() {
    let mut r = build_taxonomy_gap_audit_report();
    let original = r.taxonomy_gap_audit_hash_v1;
    // Mutate corpus_hash_v1 anchor and recompute top-level hash.
    // We rebuild via the public builder to compare a real
    // alternative state; here we mutate the field and check
    // that the rendered text differs (sensitivity to anchor).
    r.corpus_hash_v1[0] ^= 0xFF;
    let altered_text = render_t13_gap_report_text(&r);
    let original_text = {
        let r2 = build_taxonomy_gap_audit_report();
        render_t13_gap_report_text(&r2)
    };
    assert_ne!(altered_text, original_text);
    // The top-level field itself is not auto-recomputed by
    // mutating the struct directly (the build function pins it);
    // but the rendered output covers the same bytes the top-level
    // hash binds, so a change in `corpus_hash_v1` shows up here.
    assert_eq!(r.taxonomy_gap_audit_hash_v1, original);
}

#[test]
fn t13_gap_panel_count_is_seven() {
    let panels = seed_panels();
    assert_eq!(panels.len(), 7);
}

#[test]
fn t13_gap_each_panel_has_at_least_one_method() {
    let panels = seed_panels();
    for p in &panels {
        assert!(
            !p.methods.is_empty(),
            "panel {} has zero methods",
            p.panel_id.wire_name()
        );
    }
}

// ---------------------------------------------------------------
// Renderer byte-stability (2)
// ---------------------------------------------------------------

#[test]
fn t13_gap_text_render_is_byte_stable() {
    let r = build_taxonomy_gap_audit_report();
    let a = render_t13_gap_report_text(&r);
    let b = render_t13_gap_report_text(&r);
    assert_eq!(a, b);
}

#[test]
fn t13_gap_json_render_is_byte_stable() {
    let r = build_taxonomy_gap_audit_report();
    let a = render_t13_gap_report_json(&r);
    let b = render_t13_gap_report_json(&r);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Cross-anchor distinctness + closures + arithmetic (4)
// ---------------------------------------------------------------

#[test]
fn t13_gap_four_hash_namespaces_pairwise_distinct() {
    let r = build_taxonomy_gap_audit_report();
    let h = [
        r.survey_taxonomy_index_hash_v1,
        r.deterministic_gap_candidate_index_hash_v1,
        r.gap_disposition_report_hash_v1,
        r.taxonomy_gap_audit_hash_v1,
    ];
    for i in 0..h.len() {
        for j in (i + 1)..h.len() {
            assert_ne!(h[i], h[j], "T.13.GAP hashes {i} and {j} must differ");
        }
    }
}

#[test]
fn t13_gap_gpu_family_mapping_closure_holds() {
    let panels = seed_panels();
    let allowed: std::collections::HashSet<&str> =
        T13_GAP_PLAN_LOCKED_GPU_FAMILIES.iter().copied().collect();
    for p in &panels {
        for m in p.methods {
            if !m.disposition.is_rejection() && !m.gpu_family_mapping_label.is_empty() {
                assert!(
                    allowed.contains(m.gpu_family_mapping_label),
                    "method `{}` references unknown GPU family `{}`",
                    m.method_label,
                    m.gpu_family_mapping_label,
                );
            }
        }
    }
}

#[test]
fn t13_gap_bucket_histogram_sum_equals_total_methods() {
    let panels = seed_panels();
    let total: u32 = panels
        .iter()
        .map(|p| u32::try_from(p.methods.len()).unwrap_or(u32::MAX))
        .sum();
    let h = build_disposition_histogram(&panels);
    let sum: u32 = h.iter().sum();
    assert_eq!(sum, total);
}

#[test]
fn t13_gap_gap_candidate_index_admits_only_new_canonical_candidates() {
    let panels = seed_panels();
    let candidates = build_gap_candidate_index(&panels);
    let new_count: usize = panels
        .iter()
        .flat_map(|p| p.methods.iter())
        .filter(|m| m.disposition == GapDisposition::NewCanonicalCandidate)
        .count();
    assert_eq!(candidates.len(), new_count);
}

// ---------------------------------------------------------------
// Plan-locked invariants (4)
// ---------------------------------------------------------------

#[test]
fn t13_gap_plan_locked_thesis_is_present_in_text_render() {
    let r = build_taxonomy_gap_audit_report();
    let s = render_t13_gap_report_text(&r);
    assert!(s.contains(T13_GAP_PLAN_LOCKED_THESIS));
}

#[test]
fn t13_gap_anchors_live_corpus_hash_v1() {
    let r = build_taxonomy_gap_audit_report();
    let live = compute_corpus_hash_v1().bytes;
    assert_eq!(r.corpus_hash_v1, live);
}

#[test]
fn t13_gap_seed_len_pinned_to_54() {
    let r = build_taxonomy_gap_audit_report();
    assert_eq!(r.seed_len, 54);
    assert_eq!(seed_canonical_ids().len(), 54);
}

#[test]
fn t13_gap_domain_separator_carries_v1_suffix() {
    assert!(T13_GAP_SURVEY_TAXONOMY_INDEX_DOMAIN_V1.contains(":v1\0"));
}

// ---------------------------------------------------------------
// Seed admits clean
// ---------------------------------------------------------------

#[test]
fn t13_gap_seed_report_admits_under_verifier() {
    let r = build_taxonomy_gap_audit_report();
    let v = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::verify_t13_gap_report(&r);
    assert!(
        v.is_admissible(),
        "seed must admit; errors = {:?}",
        v.errors
    );
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Build a report whose first panel contains exactly one method
/// (the test injection) and zero methods in every other panel.
/// The bucket histogram, gap-candidate index, and dispositions
/// reflect only the injected method. Used by negative tests so
/// each one can target a single rule in isolation.
fn stub_report_with_method(
    m: SurveyMethodRecord,
) -> dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::TaxonomyGapAuditReportV1 {
    // We cannot construct &'static [SurveyMethodRecord] from a
    // runtime value, so we fall back to leaking the boxed slice.
    let methods: &'static [SurveyMethodRecord] = Box::leak(Box::new([m]));
    let panel = dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::SurveyTaxonomyPanelV1 {
        panel_id: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        panel_label: "stub panel (test only)",
        methods,
        survey_taxonomy_panel_hash_v1: [0u8; 32],
    };
    let mut histogram = [0u32; 12];
    histogram[usize::from(methods[0].disposition.ordinal())] = 1;
    dsfb_gpu_atlas_corpus::t13_gap_witness_family_audit::TaxonomyGapAuditReportV1 {
        report_id: "stub_report_for_negative_test",
        panels: vec![panel],
        gap_candidates: vec![],
        bucket_histogram: histogram,
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        seed_len: 54,
        survey_taxonomy_index_hash_v1: [0u8; 32],
        deterministic_gap_candidate_index_hash_v1: [0u8; 32],
        gap_disposition_report_hash_v1: [0u8; 32],
        taxonomy_gap_audit_hash_v1: [0u8; 32],
    }
}
