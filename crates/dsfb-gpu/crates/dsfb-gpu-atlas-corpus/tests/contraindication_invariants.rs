//! T.11g — `DetectorContraindicationReceiptV1` acceptance tests.
//!
//! Three panel-required load-bearing negatives:
//!
//!   - `contraindication_verifier_rejects_primary_without_known_confuser`
//!   - `contraindication_verifier_rejects_spectral_detector_without_sampling_law`
//!   - `contraindication_verifier_rejects_unit_sensitive_detector_without_unit_semantics`
//!
//! Plus determinism + sensitivity + reject-coverage + seed-shape
//! invariants. Every reject kind has at least one dedicated test.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use dsfb_gpu_atlas_corpus::contraindication::{
    build_passport_crosswalk, collect_contraindications, compute_contraindication_hash_v1,
    render_contraindications_json, render_contraindications_text, render_passport_crosswalk_json,
    render_passport_crosswalk_text, verify_contraindications, AliasSimilarityReason,
    ClosestAliasBinding, ClosestNonAliasBinding, ContraindicationSchema, ContraindicationSnapshot,
    ContraindicationVerifyErrorKind, DetectorContraindicationReceiptV1, DetectorTwinRelation,
    DoNotUseForReason, FailsWhenCondition, KnownConfuserBinding, MinimumSupport,
    NonAliasDistinctionReason, RequiredSamplingLaw, RequiredUnitSemantics, SamplingLawKind,
    SamplingRegularity, UnitResolution, UnitSemanticsKind, WorksBestWhenCondition,
    DETECTOR_CONTRAINDICATION_DOMAIN, DETECTOR_CONTRAINDICATION_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    DetectorCanonicalId, InputRequirementSet, NegativeWitnessKind, PrimitiveFamily, WitnessRole,
};

// ---------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------

fn first_seed_id() -> DetectorCanonicalId {
    SEED[0].canonical_id
}

fn baseline_receipt(canonical_id: DetectorCanonicalId) -> DetectorContraindicationReceiptV1 {
    let snap = collect_contraindications();
    snap.receipts
        .into_iter()
        .find(|r| r.canonical_id == canonical_id)
        .expect("baseline receipt for canonical id")
}

fn singleton(receipt: DetectorContraindicationReceiptV1) -> ContraindicationSnapshot {
    ContraindicationSnapshot {
        schema: ContraindicationSchema::V1DatasheetLike,
        receipts: vec![receipt],
    }
}

fn assert_has(s: &ContraindicationSnapshot, kind: ContraindicationVerifyErrorKind) {
    let errors = verify_contraindications(s);
    assert!(
        errors.iter().any(|e| e.kind == kind),
        "expected verifier kind {kind:?}; got {errors:?}",
    );
}

// ---------------------------------------------------------------
// Schema + constants
// ---------------------------------------------------------------

#[test]
fn contraindication_domain_separator_is_panel_locked() {
    assert_eq!(
        DETECTOR_CONTRAINDICATION_DOMAIN,
        "DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1\0"
    );
}

#[test]
fn contraindication_schema_id_is_panel_locked() {
    assert_eq!(
        DETECTOR_CONTRAINDICATION_SCHEMA_V1,
        "DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1"
    );
}

#[test]
fn snapshot_has_one_receipt_per_seed_record() {
    let s = collect_contraindications();
    assert_eq!(s.receipts.len(), SEED.len());
}

#[test]
fn snapshot_is_sorted_by_canonical_id_ascending() {
    let s = collect_contraindications();
    for w in s.receipts.windows(2) {
        assert!(w[0].canonical_id.0 < w[1].canonical_id.0);
    }
}

#[test]
fn snapshot_schema_is_v1_datasheet_like() {
    let s = collect_contraindications();
    assert_eq!(s.schema.as_str(), "V1DatasheetLike");
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn hash_is_deterministic_across_two_builds() {
    let a = compute_contraindication_hash_v1(&collect_contraindications());
    let b = compute_contraindication_hash_v1(&collect_contraindications());
    assert_eq!(a, b);
}

#[test]
fn hash_changes_when_receipt_added() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    let mut extra = baseline_receipt(first_seed_id());
    extra.canonical_id = DetectorCanonicalId(99_999);
    s.receipts.push(extra);
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_works_best_when_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0]
        .works_best_when
        .push(WorksBestWhenCondition::NonStationaryInputs);
    s.receipts[0].works_best_when.sort();
    s.receipts[0].works_best_when.dedup();
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_fails_when_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0]
        .fails_when
        .push(FailsWhenCondition::DeploymentMarkerArtifact);
    s.receipts[0].fails_when.sort();
    s.receipts[0].fails_when.dedup();
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_known_confuser_added() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0].known_confusers.push(KnownConfuserBinding {
        confuser: NegativeWitnessKind::UnitScaleChangeConfuser,
    });
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_required_sampling_law_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0].required_sampling_law = Some(RequiredSamplingLaw {
        kind: SamplingLawKind::GraphAdjacency,
        min_observations: 7,
        regularity: SamplingRegularity::IrregularAdmissible,
    });
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_required_units_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0].required_units = Some(RequiredUnitSemantics {
        kind: UnitSemanticsKind::PhysicalUnitsRequired,
        min_unit_resolution: UnitResolution::ExactDeclared,
    });
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_minimum_support_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0].minimum_support = MinimumSupport {
        min_baseline_observations: 999,
        min_active_observations: 2,
        min_distinct_entities: 1,
    };
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_do_not_use_for_changes() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0]
        .do_not_use_for
        .push(DoNotUseForReason::UnboundedHistory);
    s.receipts[0].do_not_use_for.sort();
    s.receipts[0].do_not_use_for.dedup();
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_closest_alias_added() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0].closest_aliases.push(ClosestAliasBinding {
        canonical_id: SEED[1].canonical_id,
        similarity_reason: AliasSimilarityReason::IdenticalFormula,
    });
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_closest_non_alias_added() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0]
        .closest_non_aliases
        .push(ClosestNonAliasBinding {
            canonical_id: SEED[1].canonical_id,
            distinction_reason: NonAliasDistinctionReason::DifferentDecisionFunctional,
        });
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_changes_when_adversarial_twin_added() {
    let base = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts[0]
        .adversarial_twins
        .push(DetectorTwinRelation::SameFormulaDifferentRole(
            SEED[1].canonical_id,
        ));
    assert_ne!(compute_contraindication_hash_v1(&s), base);
}

#[test]
fn hash_is_independent_of_receipt_order() {
    let a = compute_contraindication_hash_v1(&collect_contraindications());
    let mut s = collect_contraindications();
    s.receipts.reverse();
    let b = compute_contraindication_hash_v1(&s);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Positive admission paths
// ---------------------------------------------------------------

#[test]
fn verifier_admits_clean_seed_snapshot() {
    let s = collect_contraindications();
    let errors = verify_contraindications(&s);
    assert!(
        errors.is_empty(),
        "expected zero verifier errors on clean seed, got: {errors:?}",
    );
}

// ---------------------------------------------------------------
// PANEL-REQUIRED LOAD-BEARING NEGATIVES
// ---------------------------------------------------------------

#[test]
fn contraindication_verifier_rejects_primary_without_known_confuser() {
    // Find a Primary-witness seed record, strip its known_confusers,
    // expect PrimaryWithoutKnownConfuser.
    let primary_id = SEED
        .iter()
        .find(|r| matches!(r.witness_role, WitnessRole::Primary))
        .expect("at least one Primary witness in SEED")
        .canonical_id;
    let mut r = baseline_receipt(primary_id);
    r.known_confusers.clear();
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::PrimaryWithoutKnownConfuser,
    );
}

#[test]
fn contraindication_verifier_rejects_spectral_detector_without_sampling_law() {
    let spectral_id = SEED
        .iter()
        .find(|r| matches!(r.primitive_family, PrimitiveFamily::Spectral))
        .expect("at least one Spectral primitive in SEED")
        .canonical_id;
    let mut r = baseline_receipt(spectral_id);
    r.required_sampling_law = None;
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::SpectralWithoutSamplingLaw,
    );
}

#[test]
fn contraindication_verifier_rejects_unit_sensitive_detector_without_unit_semantics() {
    // "Unit-sensitive" covers two populations: `InputRequirementSet::UNITS`
    // bit set OR `PrimitiveFamily::{Spectral, Wavelet}` (intrinsically
    // frequency-resolved). The current SEED has no UNITS-bit records,
    // so the panel-required negative targets a Spectral detector and
    // strips its required_units.
    let id = SEED
        .iter()
        .find(|r| matches!(r.primitive_family, PrimitiveFamily::Spectral))
        .expect("at least one Spectral primitive in SEED")
        .canonical_id;
    let mut r = baseline_receipt(id);
    r.required_units = None;
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::UnitSensitiveWithoutUnitSemantics,
    );
}

// ---------------------------------------------------------------
// Verifier — remaining reject kinds (one test each)
// ---------------------------------------------------------------

#[test]
fn verifier_rejects_l5_or_l6_without_required_sampling_law() {
    // Canonical id 14 is L6 by construction.
    let mut r = baseline_receipt(DetectorCanonicalId(14));
    r.required_sampling_law = None;
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::LBandL5OrL6WithoutRequiredSamplingLaw,
    );
}

#[test]
fn verifier_rejects_closest_alias_missing() {
    let mut r = baseline_receipt(first_seed_id());
    r.closest_aliases.push(ClosestAliasBinding {
        canonical_id: DetectorCanonicalId(99_999),
        similarity_reason: AliasSimilarityReason::IdenticalFormula,
    });
    let s = singleton(r);
    assert_has(&s, ContraindicationVerifyErrorKind::ClosestAliasMissing);
}

#[test]
fn verifier_rejects_closest_non_alias_missing() {
    let mut r = baseline_receipt(first_seed_id());
    r.closest_non_aliases.push(ClosestNonAliasBinding {
        canonical_id: DetectorCanonicalId(99_999),
        distinction_reason: NonAliasDistinctionReason::DifferentDecisionFunctional,
    });
    let s = singleton(r);
    assert_has(&s, ContraindicationVerifyErrorKind::ClosestNonAliasMissing);
}

#[test]
fn verifier_rejects_unknown_detector() {
    let mut r = baseline_receipt(first_seed_id());
    r.canonical_id = DetectorCanonicalId(99_999);
    let s = singleton(r);
    assert_has(&s, ContraindicationVerifyErrorKind::UnknownDetector);
}

#[test]
fn verifier_rejects_duplicate_receipt() {
    let r = baseline_receipt(first_seed_id());
    let s = ContraindicationSnapshot {
        schema: ContraindicationSchema::V1DatasheetLike,
        receipts: vec![r.clone(), r],
    };
    assert_has(&s, ContraindicationVerifyErrorKind::DuplicateReceipt);
}

#[test]
fn verifier_rejects_adversarial_twin_missing() {
    let mut r = baseline_receipt(first_seed_id());
    r.adversarial_twins
        .push(DetectorTwinRelation::SameFormulaDifferentRole(
            DetectorCanonicalId(99_999),
        ));
    let s = singleton(r);
    assert_has(&s, ContraindicationVerifyErrorKind::AdversarialTwinMissing);
}

#[test]
fn verifier_rejects_adversarial_twin_self_reference() {
    let mut r = baseline_receipt(first_seed_id());
    r.adversarial_twins
        .push(DetectorTwinRelation::ConfuserOfPrimary(r.canonical_id));
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::AdversarialTwinSelfReference,
    );
}

#[test]
fn verifier_rejects_contraindication_without_cross_reference() {
    let mut r = baseline_receipt(first_seed_id());
    r.fails_when.clear();
    r.known_confusers.clear();
    r.do_not_use_for.clear();
    r.closest_non_aliases.clear();
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::ContraindicationWithoutCrossReference,
    );
}

#[test]
fn verifier_rejects_active_without_do_not_use_for() {
    // The first SEED record is Primary (Active); strip do_not_use_for
    // while keeping fails_when populated so the
    // ContraindicationWithoutCrossReference rule does not fire.
    let mut r = baseline_receipt(first_seed_id());
    r.do_not_use_for.clear();
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::ActiveWithoutDoNotUseFor,
    );
}

#[test]
fn verifier_rejects_distribution_without_reference_baseline() {
    let dist_id = SEED
        .iter()
        .find(|r| matches!(r.primitive_family, PrimitiveFamily::DistributionDistance))
        .expect("at least one DistributionDistance primitive in SEED")
        .canonical_id;
    let mut r = baseline_receipt(dist_id);
    r.works_best_when
        .retain(|c| !matches!(c, WorksBestWhenCondition::BaselineReferenceAvailable));
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::DistributionWithoutReferenceBaseline,
    );
}

#[test]
fn verifier_rejects_time_series_without_ordered_time_declaration() {
    let time_id = SEED
        .iter()
        .find(|r| r.input_requirements.0 & InputRequirementSet::ORDERED_TIME != 0)
        .expect("at least one time-series primitive in SEED")
        .canonical_id;
    let mut r = baseline_receipt(time_id);
    r.works_best_when
        .retain(|c| !matches!(c, WorksBestWhenCondition::TimeOrderedInput));
    r.required_sampling_law = None;
    let s = singleton(r);
    assert_has(
        &s,
        ContraindicationVerifyErrorKind::TimeSeriesWithoutOrderedTimeDeclaration,
    );
}

// ---------------------------------------------------------------
// Seed-shape invariants
// ---------------------------------------------------------------

#[test]
fn seed_every_primary_witness_carries_at_least_one_known_confuser() {
    let s = collect_contraindications();
    for r in &s.receipts {
        if matches!(r.witness_role, WitnessRole::Primary) {
            assert!(
                !r.known_confusers.is_empty(),
                "Primary detector #{} has no known_confusers",
                r.canonical_id.0,
            );
        }
    }
}

#[test]
fn seed_every_spectral_detector_has_required_sampling_law() {
    let s = collect_contraindications();
    for r in &s.receipts {
        if matches!(r.primitive_family, PrimitiveFamily::Spectral) {
            assert!(
                r.required_sampling_law.is_some(),
                "Spectral detector #{} missing required_sampling_law",
                r.canonical_id.0,
            );
        }
    }
}

#[test]
fn seed_every_unit_sensitive_detector_has_required_units() {
    let s = collect_contraindications();
    for r in &s.receipts {
        let req = SEED
            .iter()
            .find(|x| x.canonical_id == r.canonical_id)
            .map_or(InputRequirementSet(0), |x| x.input_requirements);
        if (req.0 & InputRequirementSet::UNITS) != 0 {
            assert!(
                r.required_units.is_some(),
                "Unit-sensitive detector #{} missing required_units",
                r.canonical_id.0,
            );
        }
    }
}

#[test]
fn seed_every_distribution_detector_has_baseline_reference() {
    let s = collect_contraindications();
    for r in &s.receipts {
        if matches!(r.primitive_family, PrimitiveFamily::DistributionDistance) {
            let has = r
                .works_best_when
                .iter()
                .any(|w| matches!(w, WorksBestWhenCondition::BaselineReferenceAvailable));
            assert!(
                has,
                "Distribution detector #{} missing BaselineReferenceAvailable",
                r.canonical_id.0,
            );
        }
    }
}

#[test]
fn seed_every_active_detector_declares_at_least_one_do_not_use_for() {
    let s = collect_contraindications();
    for r in &s.receipts {
        if !matches!(
            r.witness_role,
            WitnessRole::Confuser | WitnessRole::CleanWindow
        ) {
            assert!(
                !r.do_not_use_for.is_empty(),
                "Active detector #{} missing do_not_use_for",
                r.canonical_id.0,
            );
        }
    }
}

// ---------------------------------------------------------------
// Wire-name stability
// ---------------------------------------------------------------

#[test]
fn works_best_when_wire_names_are_stable() {
    assert_eq!(
        WorksBestWhenCondition::StableBaselineWindow.as_str(),
        "StableBaselineWindow",
    );
    assert_eq!(
        WorksBestWhenCondition::PersistentResidualElevation.as_str(),
        "PersistentResidualElevation",
    );
    assert_eq!(
        WorksBestWhenCondition::TimeOrderedInput.as_str(),
        "TimeOrderedInput",
    );
}

#[test]
fn fails_when_wire_names_are_stable() {
    assert_eq!(
        FailsWhenCondition::SmallSampleSize.as_str(),
        "SmallSampleSize",
    );
    assert_eq!(
        FailsWhenCondition::PeriodicBoundaryEffects.as_str(),
        "PeriodicBoundaryEffects",
    );
    assert_eq!(FailsWhenCondition::ClockSkew.as_str(), "ClockSkew");
}

#[test]
fn sampling_law_wire_names_are_stable() {
    assert_eq!(
        SamplingLawKind::RegularFixedRate.as_str(),
        "RegularFixedRate",
    );
    assert_eq!(SamplingLawKind::GraphAdjacency.as_str(), "GraphAdjacency");
    assert_eq!(
        SamplingRegularity::StrictlyRegular.as_str(),
        "StrictlyRegular",
    );
}

#[test]
fn unit_semantics_wire_names_are_stable() {
    assert_eq!(
        UnitSemanticsKind::PhysicalUnitsRequired.as_str(),
        "PhysicalUnitsRequired",
    );
    assert_eq!(
        UnitSemanticsKind::DimensionlessRatio.as_str(),
        "DimensionlessRatio",
    );
    assert_eq!(UnitResolution::ExactDeclared.as_str(), "ExactDeclared");
}

#[test]
fn do_not_use_for_wire_names_are_stable() {
    assert_eq!(
        DoNotUseForReason::ProbabilisticDecisionMaking.as_str(),
        "ProbabilisticDecisionMaking",
    );
    assert_eq!(
        DoNotUseForReason::BlackBoxRetrievalAugmentation.as_str(),
        "BlackBoxRetrievalAugmentation",
    );
}

#[test]
fn alias_distinction_wire_names_are_stable() {
    assert_eq!(
        AliasSimilarityReason::IdenticalFormula.as_str(),
        "IdenticalFormula",
    );
    assert_eq!(
        NonAliasDistinctionReason::DifferentDecisionFunctional.as_str(),
        "DifferentDecisionFunctional",
    );
}

#[test]
fn adversarial_twin_kind_strs_are_stable() {
    let target = first_seed_id();
    assert_eq!(
        DetectorTwinRelation::SameFormulaDifferentRole(target).kind_str(),
        "SameFormulaDifferentRole",
    );
    assert_eq!(
        DetectorTwinRelation::ConfuserOfPrimary(target).kind_str(),
        "ConfuserOfPrimary",
    );
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn render_text_is_deterministic_across_two_calls() {
    let s = collect_contraindications();
    assert_eq!(
        render_contraindications_text(&s),
        render_contraindications_text(&s),
    );
}

#[test]
fn render_json_is_deterministic_across_two_calls() {
    let s = collect_contraindications();
    assert_eq!(
        render_contraindications_json(&s),
        render_contraindications_json(&s),
    );
}

#[test]
fn render_text_carries_contraindication_hash() {
    use std::fmt::Write as _;
    let s = collect_contraindications();
    let h = compute_contraindication_hash_v1(&s);
    let text = render_contraindications_text(&s);
    assert!(text.contains("detector_contraindication_hash_v1"));
    let mut hex = String::with_capacity(64);
    for b in &h {
        let _ = write!(&mut hex, "{b:02x}");
    }
    assert!(text.contains(&hex));
}

#[test]
fn render_json_carries_schema_id() {
    let s = collect_contraindications();
    let json = render_contraindications_json(&s);
    assert!(json.contains("\"schema_id\":\"DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1\""));
}

#[test]
fn render_text_lists_panel_locked_non_claim() {
    let s = collect_contraindications();
    let text = render_contraindications_text(&s);
    assert!(text.contains("Panel-locked non-claim"));
    assert!(text.contains("NOT mutate"));
}

#[test]
fn render_json_contains_every_receipt() {
    let s = collect_contraindications();
    let json = render_contraindications_json(&s);
    for r in &s.receipts {
        let needle = format!("\"canonical_id\":{}", r.canonical_id.0);
        assert!(json.contains(&needle));
    }
}

#[test]
fn hash_is_independent_of_rendered_text() {
    let s = collect_contraindications();
    // Mutate the order of receipts (text-affecting) but keep the
    // content; the hash sorts by canonical_id so it must NOT change.
    let mut s2 = s.clone();
    s2.receipts.reverse();
    assert_eq!(
        compute_contraindication_hash_v1(&s),
        compute_contraindication_hash_v1(&s2),
    );
    assert_ne!(
        render_contraindications_text(&s),
        render_contraindications_text(&s2),
    );
}

// ---------------------------------------------------------------
// Passport-contraindication crosswalk
// ---------------------------------------------------------------

#[test]
fn passport_crosswalk_has_one_row_per_seed_record() {
    let s = collect_contraindications();
    let rows = build_passport_crosswalk(&s);
    assert_eq!(rows.len(), SEED.len());
}

#[test]
fn passport_crosswalk_is_sorted_by_canonical_id() {
    let s = collect_contraindications();
    let rows = build_passport_crosswalk(&s);
    for w in rows.windows(2) {
        assert!(w[0].canonical_id.0 < w[1].canonical_id.0);
    }
}

#[test]
fn passport_crosswalk_text_render_is_deterministic() {
    let s = collect_contraindications();
    assert_eq!(
        render_passport_crosswalk_text(&s),
        render_passport_crosswalk_text(&s),
    );
}

#[test]
fn passport_crosswalk_json_render_is_deterministic() {
    let s = collect_contraindications();
    assert_eq!(
        render_passport_crosswalk_json(&s),
        render_passport_crosswalk_json(&s),
    );
}

#[test]
fn passport_crosswalk_counts_known_confusers() {
    let s = collect_contraindications();
    let rows = build_passport_crosswalk(&s);
    for (row, receipt) in rows.iter().zip(s.receipts.iter()) {
        assert_eq!(row.canonical_id, receipt.canonical_id);
        assert_eq!(
            row.known_confuser_count as usize,
            receipt.known_confusers.len(),
        );
    }
}
