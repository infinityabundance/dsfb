//! S1.1.1 acceptance tests: verifier integration with the
//! post-T.10 cross-field rule and the registry-level verifier.
//!
//! Panel-required tests in this file:
//!
//! - `sample_template_verifies` (also covered in algebra_template_spec)
//! - `malformed_template_fails_verify` (also covered there)
//!
//! Additional verifier coverage:
//!
//! - `spec_with_zero_window_cells_is_rejected`
//! - `spec_with_zero_persistence_is_rejected`
//! - `spec_with_empty_axis_binding_is_rejected`
//! - `spec_with_empty_domain_tags_is_rejected`
//! - `spec_with_malformed_canonical_name_is_rejected`
//! - `verifier_is_deterministic_across_two_runs`
//!
//! S1.1.1 registry-verifier tests:
//!
//! - `verify_registry_spec_admits_hash_frozen_t10_with_live_corpus_hash`
//! - `verify_registry_spec_rejects_stale_source_corpus_hash`
//! - `verify_registry_spec_rejects_unknown_primitive_id`
//! - `verify_registry_spec_rejects_pre_hash_t9_status`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;
use dsfb_gpu_atlas_corpus::SEED;
use dsfb_gpu_atlas_registry::canonical_name::CanonicalDetectorName;
use dsfb_gpu_atlas_registry::{
    verify_registry_spec, verify_spec, AxisBinding, Comparator, CorpusBindingStatus, CostClass,
    DetectorFamily, DetectorId, DetectorParamSet, DetectorSpec, DomainTag, DomainTagSet, Gate,
    ImplementationKind, NumericMode, ParameterizationId, Statistic, Transform, VerifyErrorKind,
    WindowSpec,
};

fn clean_spec() -> DetectorSpec {
    let params = DetectorParamSet::new(64, 3, 1 << 16, 0);
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
        canonical_name: CanonicalDetectorName::build(
            DetectorFamily::RobustZMad,
            Transform::Residual,
            Statistic::Mad,
            Comparator::TwoSided,
            params,
        ),
    }
}

#[test]
fn clean_spec_verifies() {
    let spec = clean_spec();
    let errors = verify_spec(&spec);
    assert!(
        errors.is_empty(),
        "clean spec must verify with no errors; got {errors:?}"
    );
}

#[test]
fn spec_with_zero_window_cells_is_rejected() {
    let mut spec = clean_spec();
    spec.window = WindowSpec { cells: 0 };
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::InvalidWindowCells),
        "spec with window.cells = 0 must be rejected; got {errors:?}"
    );
}

#[test]
fn spec_with_zero_persistence_is_rejected() {
    let mut spec = clean_spec();
    spec.persistence_windows = 0;
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::InvalidPersistenceWindows),
        "spec with persistence_windows = 0 must be rejected; got {errors:?}"
    );
}

#[test]
fn spec_with_empty_axis_binding_is_rejected() {
    let mut spec = clean_spec();
    spec.axis_binding = AxisBinding(0);
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::EmptyAxisBinding),
        "spec with empty axis_binding must be rejected; got {errors:?}"
    );
}

#[test]
fn spec_with_empty_domain_tags_is_rejected() {
    let mut spec = clean_spec();
    spec.domain_tags = DomainTagSet::EMPTY;
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::EmptyDomainTagSet),
        "spec with empty domain_tags must be rejected; got {errors:?}"
    );
}

#[test]
fn spec_with_malformed_canonical_name_is_rejected() {
    let mut spec = clean_spec();
    // Forge a 5-token name (missing the persistence token).
    spec.canonical_name = CanonicalDetectorName::from_raw_for_test(
        "ROBUST_Z_MAD__RESIDUAL__W64__MAD__TWO_SIDED".to_string(),
    );
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::CanonicalNameWrongTokenCount),
        "spec with 5-token canonical_name must be rejected; got {errors:?}"
    );
}

#[test]
fn spec_with_empty_token_in_canonical_name_is_rejected() {
    let mut spec = clean_spec();
    // Forge a name with an empty middle token (i.e. four
    // underscores between two tokens). Six `__`-delimited tokens
    // total but one of them is empty.
    spec.canonical_name = CanonicalDetectorName::from_raw_for_test(
        "ROBUST_Z_MAD____W64__MAD__TWO_SIDED__P3".to_string(),
    );
    let errors = verify_spec(&spec);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::CanonicalNameHasEmptyToken),
        "spec with empty middle token in canonical_name must be rejected; got {errors:?}"
    );
}

#[test]
fn verifier_is_deterministic_across_two_runs() {
    let mut spec = clean_spec();
    spec.parameter_hash = [0u8; 32];
    spec.axis_binding = AxisBinding(0);
    spec.window = WindowSpec { cells: 0 };
    let a = verify_spec(&spec);
    let b = verify_spec(&spec);
    assert_eq!(a.len(), b.len(), "verifier must be deterministic");
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.kind, y.kind);
    }
}

fn registry_grade_spec() -> DetectorSpec {
    let mut spec = clean_spec();
    let hash = compute_corpus_hash_v1();
    spec.corpus_binding_status = CorpusBindingStatus::HashFrozenT10;
    spec.source_corpus_hash = hash.bytes;
    // Use a real corpus canonical_id (SEED's first record) so
    // the verify_registry_spec id lookup passes by default.
    spec.primitive_id = Some(SEED[0].canonical_id);
    spec
}

fn corpus_canonical_ids() -> Vec<DetectorCanonicalId> {
    SEED.iter().map(|r| r.canonical_id).collect()
}

#[test]
fn verify_registry_spec_admits_hash_frozen_t10_with_live_corpus_hash() {
    let spec = registry_grade_spec();
    let hash = compute_corpus_hash_v1();
    let ids = corpus_canonical_ids();
    let errors = verify_registry_spec(&spec, &hash.bytes, &ids);
    assert!(
        errors.is_empty(),
        "HashFrozenT10 + live corpus_hash_v1 + known primitive_id must verify clean; got {errors:?}"
    );
}

#[test]
fn verify_registry_spec_rejects_stale_source_corpus_hash() {
    let mut spec = registry_grade_spec();
    // Mutate one byte; the registry verifier must reject.
    spec.source_corpus_hash[0] ^= 0xFF;
    let live = compute_corpus_hash_v1();
    let ids = corpus_canonical_ids();
    let errors = verify_registry_spec(&spec, &live.bytes, &ids);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecSourceCorpusHashStale),
        "stale source_corpus_hash must trigger SpecSourceCorpusHashStale; got {errors:?}"
    );
}

#[test]
fn verify_registry_spec_rejects_unknown_primitive_id() {
    let mut spec = registry_grade_spec();
    // 9999 is outside the 1..=54 SEED range; not a known id.
    spec.primitive_id = Some(DetectorCanonicalId(9999));
    let hash = compute_corpus_hash_v1();
    let ids = corpus_canonical_ids();
    let errors = verify_registry_spec(&spec, &hash.bytes, &ids);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecPrimitiveIdUnknown),
        "unknown primitive_id must trigger SpecPrimitiveIdUnknown; got {errors:?}"
    );
}

#[test]
fn verify_registry_spec_rejects_pre_hash_t9_status() {
    let mut spec = registry_grade_spec();
    // Drop back to pre-hash; the source_corpus_hash is now
    // inconsistent (non-zero) AND the registry verifier
    // additionally requires HashFrozenT10.
    spec.corpus_binding_status = CorpusBindingStatus::PreHashT9InternalAudit;
    let hash = compute_corpus_hash_v1();
    let ids = corpus_canonical_ids();
    let errors = verify_registry_spec(&spec, &hash.bytes, &ids);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == VerifyErrorKind::SpecMustBeHashFrozenAtS12),
        "PreHashT9InternalAudit at registry level must trigger SpecMustBeHashFrozenAtS12; got {errors:?}"
    );
}
