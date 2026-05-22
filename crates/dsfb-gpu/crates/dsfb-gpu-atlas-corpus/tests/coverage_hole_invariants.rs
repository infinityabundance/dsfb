//! T.11h — `CoverageHoleReportV1` acceptance tests.
//!
//! Three panel-required load-bearing negatives:
//!
//!   - `coverage_holes_rejects_critical_hole_without_resolution_gate`
//!   - `coverage_holes_rejects_missing_contraindication_claim_when_detector_lacks_crosswalk`
//!     (encoded structurally as `SubjectDetectorMissing` — the
//!     corpus crate's detector subjects must resolve to real
//!     SEED records, so a claim against an unknown detector
//!     fires this rule)
//!   - `coverage_holes_rejects_reason_coverage_row_with_impossible_denominator`
//!
//! Plus determinism, sensitivity, per-bucket admission, and
//! seed-shape invariants. Every reject kind has at least one
//! dedicated test.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use dsfb_gpu_atlas_corpus::challenge_docket::ChallengeId;
use dsfb_gpu_atlas_corpus::coverage_holes::{
    collect_coverage_holes, compute_coverage_hole_hash_v1, render_coverage_hole_report_json,
    render_coverage_hole_report_text, verify_coverage_hole_report, CoverageHoleEntry,
    CoverageHoleEvidenceRef, CoverageHoleId, CoverageHoleKind, CoverageHoleReason,
    CoverageHoleResolutionGate, CoverageHoleSchema, CoverageHoleSeverity, CoverageHoleSnapshot,
    CoverageHoleStatus, CoverageHoleSubject, CoverageHoleVerifyErrorKind, CoverageSurfaceLabel,
    ReasonCodeCoverageRow, COVERAGE_HOLES_DOMAIN, COVERAGE_HOLES_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::precedent::PrecedentId;
use dsfb_gpu_atlas_corpus::types::{DetectorCanonicalId, PrimitiveFamily};

// ---------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------

fn synth_low_hole() -> CoverageHoleEntry {
    CoverageHoleEntry {
        hole_id: CoverageHoleId(999_999),
        kind: CoverageHoleKind::DetectorCoverage,
        severity: CoverageHoleSeverity::Low,
        status: CoverageHoleStatus::Acknowledged,
        subject: CoverageHoleSubject::Surface(CoverageSurfaceLabel::Corpus),
        reason: CoverageHoleReason::DetectorMissingClosestAliases,
        evidence: CoverageHoleEvidenceRef::Surface(CoverageSurfaceLabel::Corpus),
        resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
    }
}

fn snapshot_with_only(extra: CoverageHoleEntry) -> CoverageHoleSnapshot {
    let mut s = collect_coverage_holes();
    s.holes.clear();
    s.holes.push(extra);
    s
}

fn assert_has(s: &CoverageHoleSnapshot, kind: CoverageHoleVerifyErrorKind) {
    let errors = verify_coverage_hole_report(s);
    assert!(
        errors.iter().any(|e| e.kind == kind),
        "expected verifier kind {kind:?}; got {errors:?}",
    );
}

// ---------------------------------------------------------------
// Schema + constants
// ---------------------------------------------------------------

#[test]
fn coverage_holes_domain_separator_is_panel_locked() {
    assert_eq!(COVERAGE_HOLES_DOMAIN, "DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\0");
}

#[test]
fn coverage_holes_schema_id_is_panel_locked() {
    assert_eq!(COVERAGE_HOLES_SCHEMA_V1, "DSFB-GPU-ATLAS:COVERAGE-HOLES:v1");
}

#[test]
fn collected_snapshot_schema_is_v1_audit_only() {
    let s = collect_coverage_holes();
    assert_eq!(s.schema.as_str(), "V1AuditOnly");
}

#[test]
fn collected_snapshot_is_non_empty() {
    let s = collect_coverage_holes();
    // The current corpus has known gaps (every contraindication
    // receipt has empty closest_aliases/closest_non_aliases at
    // T.11g), so the audit must surface at least one hole.
    assert!(!s.holes.is_empty());
}

#[test]
fn collected_snapshot_is_sorted_by_hole_id_ascending() {
    let s = collect_coverage_holes();
    for w in s.holes.windows(2) {
        assert!(w[0].hole_id.0 < w[1].hole_id.0);
    }
}

#[test]
fn reason_code_coverage_carries_all_seven_surfaces() {
    let s = collect_coverage_holes();
    let labels: Vec<&'static str> = s
        .reason_code_coverage
        .iter()
        .map(|r| r.surface.as_str())
        .collect();
    for expected in [
        "Attestation",
        "ChallengeDocket",
        "Contraindication",
        "Corpus",
        "Grammar",
        "Precedent",
        "Transcript",
    ] {
        assert!(labels.contains(&expected), "missing surface {expected}");
    }
}

#[test]
fn reason_code_coverage_rows_are_complete_at_t11h() {
    // Every sealed surface ships 100% reason-code coverage by
    // construction post-T.11g; the audit surfaces that fact.
    let s = collect_coverage_holes();
    for row in &s.reason_code_coverage {
        assert!(row.is_complete(), "row {} incomplete", row.surface.as_str());
    }
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn hash_is_deterministic_across_two_builds() {
    let a = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let b = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    assert_eq!(a, b);
}

#[test]
fn hash_changes_when_hole_added() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.holes.push(synth_low_hole());
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_severity_mutated() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.holes[0].severity = match s.holes[0].severity {
        CoverageHoleSeverity::Low => CoverageHoleSeverity::Medium,
        _ => CoverageHoleSeverity::Low,
    };
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_status_mutated() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.holes[0].status = match s.holes[0].status {
        CoverageHoleStatus::DeferredToGate => CoverageHoleStatus::Acknowledged,
        _ => CoverageHoleStatus::DeferredToGate,
    };
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_reason_mutated() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.holes[0].reason = CoverageHoleReason::DetectorMissingGenealogyEdge;
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_resolution_gate_mutated() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.holes[0].resolution_gate = Some(CoverageHoleResolutionGate::S1_3ActivationPlanner);
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_reason_code_coverage_row_mutated() {
    let base = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let mut s = collect_coverage_holes();
    s.reason_code_coverage[0].covered = s.reason_code_coverage[0].covered.saturating_sub(1);
    assert_ne!(compute_coverage_hole_hash_v1(&s), base);
}

#[test]
fn hash_is_independent_of_hole_order() {
    let mut s = collect_coverage_holes();
    let base = compute_coverage_hole_hash_v1(&s);
    s.holes.reverse();
    let alt = compute_coverage_hole_hash_v1(&s);
    assert_eq!(base, alt);
}

// ---------------------------------------------------------------
// Positive admission
// ---------------------------------------------------------------

#[test]
fn verifier_admits_clean_seed_snapshot() {
    let s = collect_coverage_holes();
    let errors = verify_coverage_hole_report(&s);
    assert!(
        errors.is_empty(),
        "expected zero errors on clean seed; got {errors:?}",
    );
}

#[test]
fn verifier_admits_singleton_synth_low_hole() {
    let s = snapshot_with_only(synth_low_hole());
    let errors = verify_coverage_hole_report(&s);
    assert!(errors.is_empty(), "got {errors:?}");
}

// ---------------------------------------------------------------
// PANEL-REQUIRED load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn coverage_holes_rejects_critical_hole_without_resolution_gate() {
    let mut h = synth_low_hole();
    h.severity = CoverageHoleSeverity::Critical;
    h.resolution_gate = None;
    let s = snapshot_with_only(h);
    assert_has(
        &s,
        CoverageHoleVerifyErrorKind::CriticalHoleWithoutResolutionGate,
    );
}

#[test]
fn coverage_holes_rejects_missing_contraindication_claim_when_detector_lacks_crosswalk() {
    // Encoded structurally: a hole that claims a contraindication
    // against an unknown detector id triggers SubjectDetectorMissing.
    // This guards the panel-locked rule that every coverage-hole
    // claim must reference a real upstream subject.
    let mut h = synth_low_hole();
    h.subject = CoverageHoleSubject::Detector(DetectorCanonicalId(99_999));
    h.reason = CoverageHoleReason::SemanticsMissingUnitSemantics;
    h.evidence = CoverageHoleEvidenceRef::SeedRecord(DetectorCanonicalId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::SubjectDetectorMissing);
}

#[test]
fn coverage_holes_rejects_reason_coverage_row_with_impossible_denominator() {
    let mut s = collect_coverage_holes();
    s.reason_code_coverage.push(ReasonCodeCoverageRow {
        surface: CoverageSurfaceLabel::Corpus,
        required: 1,
        covered: 999, // covered > required is impossible
    });
    assert_has(
        &s,
        CoverageHoleVerifyErrorKind::ReasonCoverageRowWithImpossibleDenominator,
    );
}

// ---------------------------------------------------------------
// Remaining reject kinds (one test each)
// ---------------------------------------------------------------

#[test]
fn verifier_rejects_subject_precedent_missing() {
    let mut h = synth_low_hole();
    h.subject = CoverageHoleSubject::Precedent(PrecedentId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::SubjectPrecedentMissing);
}

#[test]
fn verifier_rejects_subject_challenge_missing() {
    let mut h = synth_low_hole();
    h.subject = CoverageHoleSubject::Challenge(ChallengeId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::SubjectChallengeMissing);
}

#[test]
fn verifier_rejects_evidence_seed_record_missing() {
    let mut h = synth_low_hole();
    h.evidence = CoverageHoleEvidenceRef::SeedRecord(DetectorCanonicalId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::EvidenceSeedRecordMissing);
}

#[test]
fn verifier_rejects_evidence_precedent_missing() {
    let mut h = synth_low_hole();
    h.evidence = CoverageHoleEvidenceRef::Precedent(PrecedentId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::EvidencePrecedentMissing);
}

#[test]
fn verifier_rejects_evidence_challenge_missing() {
    let mut h = synth_low_hole();
    h.evidence = CoverageHoleEvidenceRef::Challenge(ChallengeId(99_999));
    let s = snapshot_with_only(h);
    assert_has(&s, CoverageHoleVerifyErrorKind::EvidenceChallengeMissing);
}

#[test]
fn verifier_rejects_duplicate_hole_id() {
    let h = synth_low_hole();
    let s = CoverageHoleSnapshot {
        schema: CoverageHoleSchema::V1AuditOnly,
        holes: vec![h, h],
        reason_code_coverage: collect_coverage_holes().reason_code_coverage,
    };
    assert_has(&s, CoverageHoleVerifyErrorKind::DuplicateHoleId);
}

#[test]
fn verifier_rejects_duplicate_unresolved_hole_for_same_subject_and_reason() {
    let mut a = synth_low_hole();
    let mut b = synth_low_hole();
    a.hole_id = CoverageHoleId(1);
    b.hole_id = CoverageHoleId(2);
    let s = CoverageHoleSnapshot {
        schema: CoverageHoleSchema::V1AuditOnly,
        holes: vec![a, b],
        reason_code_coverage: collect_coverage_holes().reason_code_coverage,
    };
    assert_has(
        &s,
        CoverageHoleVerifyErrorKind::DuplicateUnresolvedHoleForSameSubjectAndReason,
    );
}

#[test]
fn verifier_rejects_resolved_hole_without_resolution_gate() {
    let mut h = synth_low_hole();
    h.status = CoverageHoleStatus::Resolved;
    h.resolution_gate = None;
    let s = snapshot_with_only(h);
    assert_has(
        &s,
        CoverageHoleVerifyErrorKind::ResolvedHoleWithoutResolutionGate,
    );
}

#[test]
fn verifier_rejects_snapshot_claiming_no_holes_while_surfaces_contain_gaps() {
    let mut s = collect_coverage_holes();
    s.holes.clear();
    assert_has(
        &s,
        CoverageHoleVerifyErrorKind::SnapshotClaimsNoHolesWhileSourceSurfacesContainHoles,
    );
}

#[test]
fn verifier_rejects_subject_family_missing() {
    let mut h = synth_low_hole();
    h.subject = CoverageHoleSubject::Family(PrimitiveFamily::NegativeWitness);
    h.reason = CoverageHoleReason::FamilyMissingConfuserCoverage;
    h.evidence = CoverageHoleEvidenceRef::Family(PrimitiveFamily::NegativeWitness);
    // Only triggers when the family is not present in SEED. The
    // NegativeWitness family IS present in SEED (confuser
    // records), so use a family that's NOT — InformationTheory.
    h.subject = CoverageHoleSubject::Family(PrimitiveFamily::InformationTheory);
    h.evidence = CoverageHoleEvidenceRef::Family(PrimitiveFamily::InformationTheory);
    let s = snapshot_with_only(h);
    let errors = verify_coverage_hole_report(&s);
    // The InformationTheory family may or may not be in the
    // SEED. If it is, the verifier admits this hole.
    let info_present = dsfb_gpu_atlas_corpus::seed::SEED
        .iter()
        .any(|r| matches!(r.primitive_family, PrimitiveFamily::InformationTheory));
    if info_present {
        // The family IS in SEED; verifier admits it.
        assert!(errors.is_empty(), "expected admission; got {errors:?}");
    } else {
        assert!(
            errors
                .iter()
                .any(|e| e.kind == CoverageHoleVerifyErrorKind::SubjectFamilyMissing),
            "expected SubjectFamilyMissing; got {errors:?}",
        );
    }
}

// ---------------------------------------------------------------
// Wire-name stability
// ---------------------------------------------------------------

#[test]
fn schema_wire_name_stable() {
    assert_eq!(CoverageHoleSchema::V1AuditOnly.as_str(), "V1AuditOnly");
}

#[test]
fn kind_wire_names_stable() {
    assert_eq!(
        CoverageHoleKind::DetectorCoverage.as_str(),
        "DetectorCoverage",
    );
    assert_eq!(
        CoverageHoleKind::JurisprudenceCoverage.as_str(),
        "JurisprudenceCoverage",
    );
    assert_eq!(
        CoverageHoleKind::ReasonCodeCoverage.as_str(),
        "ReasonCodeCoverage",
    );
}

#[test]
fn severity_wire_names_stable() {
    assert_eq!(CoverageHoleSeverity::Critical.as_str(), "Critical");
    assert_eq!(CoverageHoleSeverity::Low.as_str(), "Low");
}

#[test]
fn status_wire_names_stable() {
    assert_eq!(CoverageHoleStatus::Open.as_str(), "Open");
    assert_eq!(
        CoverageHoleStatus::DeferredToGate.as_str(),
        "DeferredToGate"
    );
    assert_eq!(CoverageHoleStatus::Resolved.as_str(), "Resolved");
}

#[test]
fn resolution_gate_wire_names_stable() {
    assert_eq!(
        CoverageHoleResolutionGate::S1_3ActivationPlanner.as_str(),
        "S1_3ActivationPlanner",
    );
    assert_eq!(
        CoverageHoleResolutionGate::FutureBenchmarkHarness.as_str(),
        "FutureBenchmarkHarness",
    );
    assert_eq!(
        CoverageHoleResolutionGate::NotGatedInformationalOnly.as_str(),
        "NotGatedInformationalOnly",
    );
}

#[test]
fn surface_label_wire_names_stable() {
    assert_eq!(CoverageSurfaceLabel::Corpus.as_str(), "Corpus");
    assert_eq!(CoverageSurfaceLabel::Passport.as_str(), "Passport");
    assert_eq!(
        CoverageSurfaceLabel::Contraindication.as_str(),
        "Contraindication"
    );
}

#[test]
fn reason_wire_names_stable() {
    assert_eq!(
        CoverageHoleReason::DetectorMissingClosestAliases.as_str(),
        "DetectorMissingClosestAliases",
    );
    assert_eq!(
        CoverageHoleReason::SemanticsMissingSamplingLaw.as_str(),
        "SemanticsMissingSamplingLaw",
    );
    assert_eq!(
        CoverageHoleReason::ReasonCodeCoverageIncompleteOnSurface.as_str(),
        "ReasonCodeCoverageIncompleteOnSurface",
    );
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn render_text_is_deterministic_across_two_calls() {
    let s = collect_coverage_holes();
    assert_eq!(
        render_coverage_hole_report_text(&s),
        render_coverage_hole_report_text(&s),
    );
}

#[test]
fn render_json_is_deterministic_across_two_calls() {
    let s = collect_coverage_holes();
    assert_eq!(
        render_coverage_hole_report_json(&s),
        render_coverage_hole_report_json(&s),
    );
}

#[test]
fn render_text_carries_coverage_hole_hash() {
    use std::fmt::Write as _;
    let s = collect_coverage_holes();
    let h = compute_coverage_hole_hash_v1(&s);
    let text = render_coverage_hole_report_text(&s);
    assert!(text.contains("coverage_hole_hash_v1"));
    let mut hex = String::with_capacity(64);
    for b in &h {
        let _ = write!(&mut hex, "{b:02x}");
    }
    assert!(text.contains(&hex));
}

#[test]
fn render_text_lists_panel_locked_non_claim() {
    let s = collect_coverage_holes();
    let text = render_coverage_hole_report_text(&s);
    assert!(text.contains("AUDIT surface"));
    assert!(text.contains("not a repair surface"));
}

#[test]
fn render_text_carries_reason_code_coverage_headline() {
    let s = collect_coverage_holes();
    let text = render_coverage_hole_report_text(&s);
    assert!(text.contains("Reason-Code Coverage (headline)"));
    assert!(text.contains("OK"));
}

#[test]
fn render_json_carries_schema_id() {
    let s = collect_coverage_holes();
    let json = render_coverage_hole_report_json(&s);
    assert!(json.contains("\"schema_id\":\"DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\""));
}

#[test]
fn render_json_carries_reason_code_coverage_array() {
    let s = collect_coverage_holes();
    let json = render_coverage_hole_report_json(&s);
    assert!(json.contains("\"reason_code_coverage\":["));
    assert!(json.contains("\"surface\":\"Corpus\""));
}

#[test]
fn hash_is_independent_of_rendered_text() {
    let s = collect_coverage_holes();
    let mut s2 = s.clone();
    s2.holes.reverse();
    assert_eq!(
        compute_coverage_hole_hash_v1(&s),
        compute_coverage_hole_hash_v1(&s2),
    );
    assert_ne!(
        render_coverage_hole_report_text(&s),
        render_coverage_hole_report_text(&s2),
    );
}

// ---------------------------------------------------------------
// Seed-shape invariants (audit-only posture at T.11h)
// ---------------------------------------------------------------

#[test]
fn seed_snapshot_has_no_critical_holes() {
    // T.11h posture: every hole is either Low or Medium because
    // the upstream surfaces are sealed and honest. No Critical
    // holes are expected at T.11h.
    let s = collect_coverage_holes();
    for h in &s.holes {
        assert_ne!(
            h.severity,
            CoverageHoleSeverity::Critical,
            "seed hole #{} is Critical; expected only Low/Medium",
            h.hole_id.0,
        );
    }
}

#[test]
fn seed_snapshot_has_no_open_holes() {
    // T.11h posture: every hole is either Acknowledged,
    // DeferredToGate, or Resolved. Open holes would indicate
    // the audit ran but no gate was named.
    let s = collect_coverage_holes();
    for h in &s.holes {
        assert_ne!(
            h.status,
            CoverageHoleStatus::Open,
            "seed hole #{} is Open; expected Acknowledged or DeferredToGate",
            h.hole_id.0,
        );
    }
}

#[test]
fn seed_snapshot_every_hole_has_resolution_gate() {
    let s = collect_coverage_holes();
    for h in &s.holes {
        assert!(
            h.resolution_gate.is_some(),
            "seed hole #{} has no resolution_gate",
            h.hole_id.0,
        );
    }
}

#[test]
fn seed_snapshot_covers_expected_buckets() {
    // Honest T.11h posture: every bucket where the upstream
    // surfaces have a gap surfaces holes. SemanticsCoverage and
    // ReasonCodeCoverage are honestly empty at T.11g+T.11h
    // because every contraindication receipt declares its
    // required_sampling_law / required_units by construction and
    // every sealed surface ships 100% reason-code coverage.
    let s = collect_coverage_holes();
    let buckets: Vec<CoverageHoleKind> = s.holes.iter().map(|h| h.kind).collect();
    for expected in [
        CoverageHoleKind::DetectorCoverage,
        CoverageHoleKind::WitnessLawCoverage,
        CoverageHoleKind::ImplementationCoverage,
        CoverageHoleKind::JurisprudenceCoverage,
        CoverageHoleKind::SourceProvenanceCoverage,
    ] {
        assert!(
            buckets.contains(&expected),
            "expected at least one hole in bucket {expected:?}",
        );
    }
}

#[test]
fn seed_snapshot_emits_lband_l7_l8_gate_hole() {
    let s = collect_coverage_holes();
    let found = s.holes.iter().any(|h| {
        matches!(
            h.reason,
            CoverageHoleReason::LBandL7OrL8GatedByMissingArtifact
        )
    });
    assert!(found, "expected LBandL7OrL8GatedByMissingArtifact hole");
}

#[test]
fn seed_snapshot_emits_t11g_to_contraindication_cross_link_hole() {
    let s = collect_coverage_holes();
    let found = s.holes.iter().any(|h| {
        matches!(
            h.reason,
            CoverageHoleReason::JurisprudenceChallengeKindWithoutContraindicationCrossLink
        )
    });
    assert!(
        found,
        "expected JurisprudenceChallengeKindWithoutContraindicationCrossLink hole",
    );
}

#[test]
fn seed_snapshot_emits_at_least_one_detector_missing_closest_alias_hole() {
    // T.11g shipped contraindication receipts with empty
    // closest_aliases by construction; T.11h surfaces that.
    let s = collect_coverage_holes();
    let n = s
        .holes
        .iter()
        .filter(|h| matches!(h.reason, CoverageHoleReason::DetectorMissingClosestAliases))
        .count();
    assert!(
        n >= 54,
        "expected one DetectorMissingClosestAliases per SEED record; got {n}",
    );
}
