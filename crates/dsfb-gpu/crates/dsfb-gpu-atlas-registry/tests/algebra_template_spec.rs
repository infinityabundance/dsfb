//! S1.1.1 acceptance tests: template + spec shape with the
//! post-T.10 cross-field rule
//! (`HashFrozenT10 ⇔ source_corpus_hash != [0; 32]`).
//!
//! Panel-required tests in this file:
//!
//! - `detector_template_requires_primitive_id`
//! - `detector_spec_requires_parameter_hash`
//! - `detector_spec_requires_corpus_binding_status`
//! - `detector_spec_rejects_hash_frozen_without_source_corpus_hash`
//! - `detector_spec_admits_hash_frozen_t10_with_non_zero_corpus_hash`
//! - `detector_spec_rejects_pre_hash_with_non_zero_source_corpus_hash`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;
use dsfb_gpu_atlas_registry::canonical_name::CanonicalDetectorName;
use dsfb_gpu_atlas_registry::{
    verify_spec, verify_template, AxisBinding, Comparator, CorpusBindingStatus, CostClass,
    DetectorFamily, DetectorId, DetectorParamSet, DetectorSpec, DetectorTemplate, DomainTag,
    DomainTagSet, Gate, ImplementationKind, NumericMode, ParameterizationId, Statistic, Transform,
    VerifyErrorKind, WindowSpec,
};

fn sample_template() -> DetectorTemplate {
    DetectorTemplate::minimal(
        DetectorFamily::RobustZMad,
        DetectorCanonicalId(101),
        Transform::Residual,
        Statistic::Mad,
        Comparator::TwoSided,
        WindowSpec::W64,
        3,
        AxisBinding::single(AxisBinding::AXIS_1_RESIDUAL_MAGNITUDE),
        DomainTagSet::EMPTY.with(DomainTag::TimeSeries),
    )
}

fn sample_spec() -> DetectorSpec {
    let params = DetectorParamSet::new(64, 3, 1 << 16, 0);
    let canonical_name = CanonicalDetectorName::build(
        DetectorFamily::RobustZMad,
        Transform::Residual,
        Statistic::Mad,
        Comparator::TwoSided,
        params,
    );
    DetectorSpec {
        detector_id: DetectorId(0),
        parameterization_id: ParameterizationId(0),
        family: DetectorFamily::RobustZMad,
        transform: Transform::Residual,
        window: WindowSpec::W64,
        statistic: Statistic::Mad,
        comparator: Comparator::TwoSided,
        gate: Gate::Persistence,
        persistence_windows: 3,
        axis_binding: AxisBinding::single(AxisBinding::AXIS_1_RESIDUAL_MAGNITUDE),
        domain_tags: DomainTagSet::EMPTY.with(DomainTag::TimeSeries),
        cost_class: CostClass::Light,
        numeric_mode: NumericMode::Q16_16,
        implementation_kind: ImplementationKind::ScalarCpu,
        parameter_hash: [0xAB; 32],
        primitive_id: Some(DetectorCanonicalId(101)),
        corpus_binding_status: CorpusBindingStatus::PreHashT9InternalAudit,
        source_corpus_hash: [0u8; 32],
        canonical_name,
    }
}

#[test]
fn detector_template_requires_primitive_id() {
    let clean = sample_template();
    let errors = verify_template(&clean);
    assert!(
        errors.is_empty(),
        "sample template should verify clean; got {errors:?}"
    );

    let mut bad = sample_template();
    bad.primitive_id = None;
    let errors = verify_template(&bad);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::TemplateMissingPrimitiveId),
        "template with primitive_id = None must trigger TemplateMissingPrimitiveId; got {errors:?}"
    );
}

#[test]
fn detector_spec_requires_parameter_hash() {
    let clean = sample_spec();
    assert!(
        verify_spec(&clean).is_empty(),
        "sample spec should verify clean"
    );

    let mut bad = sample_spec();
    bad.parameter_hash = [0u8; 32];
    let errors = verify_spec(&bad);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecMissingParameterHash),
        "spec with all-zero parameter_hash must trigger SpecMissingParameterHash; got {errors:?}"
    );
}

#[test]
fn detector_spec_requires_corpus_binding_status() {
    // Every spec MUST carry a `corpus_binding_status` field.
    // The sample fixture uses `PreHashT9InternalAudit` (the
    // algebra-only fixture variant); the matching
    // `source_corpus_hash = [0; 32]` keeps the cross-field rule
    // satisfied. The historical S1.1 default helper is preserved
    // for fixtures that pre-date the post-T.10 cross-field rule.
    assert_eq!(
        CorpusBindingStatus::S1_1_DEFAULT,
        CorpusBindingStatus::PreHashT9InternalAudit,
        "S1.1 default corpus_binding_status historical helper must remain PreHashT9InternalAudit"
    );
    let clean = sample_spec();
    assert_eq!(
        clean.corpus_binding_status,
        CorpusBindingStatus::PreHashT9InternalAudit
    );
    assert_eq!(
        clean.source_corpus_hash, [0u8; 32],
        "PreHashT9 fixture must carry source_corpus_hash = [0; 32]"
    );
    assert!(verify_spec(&clean).is_empty());
}

#[test]
fn detector_spec_rejects_hash_frozen_without_source_corpus_hash() {
    // Post-T.10 cross-field rule: a spec claiming
    // `HashFrozenT10` MUST carry a non-zero `source_corpus_hash`.
    // The S1.1-era variant kind `SpecHashFrozenWithoutT10` is
    // reused; its message distinguishes the two failure modes.
    let mut forged = sample_spec();
    forged.corpus_binding_status = CorpusBindingStatus::HashFrozenT10;
    // source_corpus_hash stays at the all-zero sentinel — invalid
    // for a HashFrozenT10 spec post-T.10.
    let errors = verify_spec(&forged);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecHashFrozenWithoutT10),
        "spec with HashFrozenT10 + zero source_corpus_hash must be rejected; got {errors:?}"
    );
}

#[test]
fn detector_spec_admits_hash_frozen_t10_with_non_zero_corpus_hash() {
    // Post-T.10 positive direction: a spec carrying
    // HashFrozenT10 AND a non-zero source_corpus_hash verifies
    // clean. T.10's `corpus_hash_v1` freeze is the event that
    // made this gate admissible.
    let mut spec = sample_spec();
    spec.corpus_binding_status = CorpusBindingStatus::HashFrozenT10;
    spec.source_corpus_hash = [0xCD; 32];
    let errors = verify_spec(&spec);
    assert!(
        errors.is_empty(),
        "HashFrozenT10 + non-zero source_corpus_hash must verify clean post-T.10; got {errors:?}"
    );
}

#[test]
fn detector_spec_rejects_pre_hash_with_non_zero_source_corpus_hash() {
    // The reverse half of the cross-field invariant:
    // PreHashT9InternalAudit MUST carry source_corpus_hash =
    // [0; 32]. A pre-freeze spec that claims a non-zero
    // corpus hash is making a binding it has no right to.
    let mut forged = sample_spec();
    forged.corpus_binding_status = CorpusBindingStatus::PreHashT9InternalAudit;
    forged.source_corpus_hash = [0xCD; 32];
    let errors = verify_spec(&forged);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecHashFrozenWithoutT10),
        "PreHashT9InternalAudit + non-zero source_corpus_hash must be rejected; got {errors:?}"
    );
}

#[test]
fn sample_template_verifies() {
    let t = sample_template();
    let errors = verify_template(&t);
    assert!(
        errors.is_empty(),
        "the minimal sample template must verify clean; got {errors:?}"
    );
}

#[test]
fn malformed_template_fails_verify() {
    let mut t = sample_template();
    t.default_window = WindowSpec { cells: 0 };
    t.default_axis_binding = AxisBinding(0);
    t.domain_tags = DomainTagSet::EMPTY;
    t.primitive_id = None;
    let errors = verify_template(&t);
    let kinds: Vec<VerifyErrorKind> = errors.iter().map(|e| e.kind.clone()).collect();
    assert!(kinds.contains(&VerifyErrorKind::TemplateMissingPrimitiveId));
    assert!(kinds.contains(&VerifyErrorKind::InvalidWindowCells));
    assert!(kinds.contains(&VerifyErrorKind::EmptyAxisBinding));
    assert!(kinds.contains(&VerifyErrorKind::EmptyDomainTagSet));
    assert_eq!(
        errors.len(),
        4,
        "all four invariants should fire; got {kinds:?}"
    );
}
