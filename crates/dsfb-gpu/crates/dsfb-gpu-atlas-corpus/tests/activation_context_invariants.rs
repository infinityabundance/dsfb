//! S1.3c acceptance suite — TaskManifestV1 / DatasetManifestV1
//! / ActivationContextV1 invariants.
//!
//! Every test states the WHY in a leading comment.
//! Four panel-required load-bearing negatives are marked with
//! the `_rejects_` prefix; they pin the verifier's blocking
//! rules.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::activation::KNOWN_S12_REGISTRY_HASH_V2;
use dsfb_gpu_atlas_corpus::activation_context::{
    build_activation_context, build_dataset_manifest, build_task_manifest,
    compute_activation_context_hash_v1, compute_dataset_manifest_hash_v1,
    compute_task_manifest_hash_v1, render_activation_context_json, render_activation_context_text,
    render_dataset_manifest_json, render_dataset_manifest_text, render_task_manifest_json,
    render_task_manifest_text, seed_dataset_manifest, seed_task_manifest,
    verify_activation_against_context, verify_activation_context, verify_dataset_manifest,
    verify_task_manifest, ActivationContextV1, ArtifactFixedness, ColumnKindSet,
    ContextVerifyErrorKind, MissingnessProfile, SamplingLaw, StrictnessLevel, TargetEpisodeKindSet,
    TaskKind, TimestampLaw, UnitSemantics, WitnessRoleSet, ACTIVATION_CONTEXT_DOMAIN,
    ACTIVATION_CONTEXT_SCHEMA_V1, DATASET_MANIFEST_DOMAIN, DATASET_MANIFEST_SCHEMA_V1,
    TASK_MANIFEST_DOMAIN, TASK_MANIFEST_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::challenge_docket::collect_challenge_docket;
use dsfb_gpu_atlas_corpus::contraindication::{
    collect_contraindications, compute_contraindication_hash_v1,
};
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::coverage_holes::{
    collect_coverage_holes, compute_coverage_hole_hash_v1,
};
use dsfb_gpu_atlas_corpus::types::{DetectorCanonicalId, DomainTagSet};

// ---------------------------------------------------------------
// Schema constants
// ---------------------------------------------------------------

#[test]
fn domain_separators_end_in_nul() {
    assert!(TASK_MANIFEST_DOMAIN.ends_with('\0'));
    assert!(DATASET_MANIFEST_DOMAIN.ends_with('\0'));
    assert!(ACTIVATION_CONTEXT_DOMAIN.ends_with('\0'));
}

#[test]
fn schema_wire_names_are_stable() {
    assert_eq!(TASK_MANIFEST_SCHEMA_V1, "TaskManifestV1");
    assert_eq!(DATASET_MANIFEST_SCHEMA_V1, "DatasetManifestV1");
    assert_eq!(ACTIVATION_CONTEXT_SCHEMA_V1, "ActivationContextV1");
}

// ---------------------------------------------------------------
// Seed shape
// ---------------------------------------------------------------

#[test]
fn seed_task_manifest_is_admissible() {
    let task = seed_task_manifest();
    let errors = verify_task_manifest(&task);
    assert!(errors.is_empty(), "seed task manifest errors: {errors:?}");
}

#[test]
fn seed_dataset_manifest_is_admissible() {
    let dataset = seed_dataset_manifest();
    let errors = verify_dataset_manifest(&dataset);
    assert!(
        errors.is_empty(),
        "seed dataset manifest errors: {errors:?}"
    );
}

#[test]
fn seed_activation_context_is_admissible() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    let errors = verify_activation_context(&c, &task, &dataset);
    assert!(errors.is_empty(), "seed context errors: {errors:?}");
}

#[test]
fn anchor_hashes_are_populated() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    assert_ne!(c.corpus_hash_v1, [0u8; 32]);
    assert_ne!(c.registry_hash_v2, [0u8; 32]);
    assert_ne!(c.task_manifest_hash_v1, [0u8; 32]);
    assert_ne!(c.dataset_manifest_hash_v1, [0u8; 32]);
    assert_ne!(c.coverage_hole_hash_v1, [0u8; 32]);
    assert_ne!(c.detector_contraindication_hash_v1, [0u8; 32]);
    assert_ne!(c.activation_context_hash_v1, [0u8; 32]);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn task_manifest_hash_is_deterministic() {
    assert_eq!(
        seed_task_manifest().task_manifest_hash_v1,
        seed_task_manifest().task_manifest_hash_v1
    );
}

#[test]
fn dataset_manifest_hash_is_deterministic() {
    assert_eq!(
        seed_dataset_manifest().dataset_manifest_hash_v1,
        seed_dataset_manifest().dataset_manifest_hash_v1
    );
}

#[test]
fn recomputed_task_manifest_hash_matches_stored() {
    let m = seed_task_manifest();
    assert_eq!(compute_task_manifest_hash_v1(&m), m.task_manifest_hash_v1);
}

#[test]
fn recomputed_dataset_manifest_hash_matches_stored() {
    let m = seed_dataset_manifest();
    assert_eq!(
        compute_dataset_manifest_hash_v1(&m),
        m.dataset_manifest_hash_v1
    );
}

#[test]
fn recomputed_context_hash_matches_stored() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    assert_eq!(
        compute_activation_context_hash_v1(&c),
        c.activation_context_hash_v1
    );
}

/// Load-bearing negative #4 (panel-required): changing the
/// dataset's sampling_law changes
/// activation_context_hash_v1.
#[test]
fn activation_context_hash_changes_when_sampling_law_changes() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let ch = compute_coverage_hole_hash_v1(&coverage);
    let ct = compute_contraindication_hash_v1(&contras);
    let a = build_activation_context(&task, &dataset, KNOWN_S12_REGISTRY_HASH_V2, ch, ct);

    let dataset_b = build_dataset_manifest(
        dataset.dataset_id,
        dataset.artifact_fixedness,
        dataset.schema_hash,
        dataset.column_kinds,
        dataset.unit_semantics,
        SamplingLaw::OrderedNonRegular, // mutation
        dataset.missingness_profile,
        dataset.timestamp_law,
        dataset.source_artifact_hash,
    );
    let b = build_activation_context(&task, &dataset_b, KNOWN_S12_REGISTRY_HASH_V2, ch, ct);
    assert_ne!(a.activation_context_hash_v1, b.activation_context_hash_v1);
}

#[test]
fn context_hash_changes_when_task_changes() {
    let task_a = seed_task_manifest();
    let task_b = build_task_manifest(
        task_a.task_id,
        TaskKind::TabularDatasetStructure,
        task_a.domain_tags,
        task_a.target_episode_kinds,
        task_a.required_witness_roles,
        task_a.forbidden_witness_roles,
        task_a.strictness_level,
    );
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let ch = compute_coverage_hole_hash_v1(&coverage);
    let ct = compute_contraindication_hash_v1(&contras);
    let a = build_activation_context(&task_a, &dataset, KNOWN_S12_REGISTRY_HASH_V2, ch, ct);
    let b = build_activation_context(&task_b, &dataset, KNOWN_S12_REGISTRY_HASH_V2, ch, ct);
    assert_ne!(a.activation_context_hash_v1, b.activation_context_hash_v1);
}

// ---------------------------------------------------------------
// Verifier — load-bearing negatives
// ---------------------------------------------------------------

/// Load-bearing negative #1 (panel-required): fixed-artifact
/// fixedness with all-zero source_artifact_hash MUST be
/// rejected.
#[test]
fn context_rejects_fixed_artifact_without_source_hash() {
    let bad = build_dataset_manifest(
        "synthetic_no_source",
        ArtifactFixedness::FixedBytes,
        [0xaa; 32], // non-zero schema
        ColumnKindSet(ColumnKindSet::NUMERIC_INTEGER),
        UnitSemantics::DimensionlessRatios,
        SamplingLaw::UnorderedRowSet,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::NoneDeclared,
        [0u8; 32], // missing source hash — the defect
    );
    let errors = verify_dataset_manifest(&bad);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::FixedArtifactMissingSourceHash
    )));
}

/// Load-bearing negative #2 (panel-required): time-series task
/// bound to a dataset with no timestamp law MUST be rejected.
#[test]
fn context_rejects_time_series_task_without_sampling_law() {
    let task = seed_task_manifest(); // DebugTraceResidualCourt is time-series
    let dataset_bad = build_dataset_manifest(
        "synthetic_no_timestamp",
        ArtifactFixedness::FixedEventCatalog,
        [0xab; 32],
        ColumnKindSet(ColumnKindSet::NUMERIC_INTEGER),
        UnitSemantics::LatencyMillisecondsAndErrorIndicator,
        SamplingLaw::OrderedRegularWindows,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::NoneDeclared, // missing — the defect
        [0xcd; 32],
    );
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset_bad,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    let errors = verify_activation_context(&c, &task, &dataset_bad);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::TimeSeriesTaskWithoutTimestampLaw
    )));
}

/// Load-bearing negative #3 (panel-required): activating a
/// unit-sensitive detector against a context whose dataset
/// declares NoUnitsDeclared MUST be rejected by the activation
/// crosscheck.
#[test]
fn activation_rejects_enabled_unit_sensitive_detector_when_context_has_no_units() {
    let dataset_no_units = build_dataset_manifest(
        "synthetic_no_units",
        ArtifactFixedness::FixedEventCatalog,
        [0xee; 32],
        ColumnKindSet(ColumnKindSet::CATEGORICAL),
        UnitSemantics::NoUnitsDeclared, // the defect
        SamplingLaw::UnorderedRowSet,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::Unordered,
        [0u8; 32],
    );
    // Pretend canonical_id 99 is unit-sensitive and enabled
    // by the planner.
    let id = DetectorCanonicalId(99);
    let enabled = [id];
    let spectral: [DetectorCanonicalId; 0] = [];
    let unit_sensitive = [id];
    let errors =
        verify_activation_against_context(&enabled, &spectral, &unit_sensitive, &dataset_no_units);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::UnitSensitiveDetectorWithoutUnits { canonical_id }
            if canonical_id == id
    )));
}

// ---------------------------------------------------------------
// Verifier — additional rules
// ---------------------------------------------------------------

#[test]
fn context_rejects_empty_task_id() {
    let bad = build_task_manifest(
        "",
        TaskKind::DebugTraceResidualCourt,
        DomainTagSet(DomainTagSet::DEBUG),
        TargetEpisodeKindSet(TargetEpisodeKindSet::PRIMARY),
        WitnessRoleSet(WitnessRoleSet::PRIMARY),
        WitnessRoleSet(0),
        StrictnessLevel::Phase5_6,
    );
    let errors = verify_task_manifest(&bad);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ContextVerifyErrorKind::TaskManifestMissingTaskId)));
}

#[test]
fn context_rejects_empty_dataset_id() {
    let bad = build_dataset_manifest(
        "",
        ArtifactFixedness::StreamingAppendOnly,
        [0x11; 32],
        ColumnKindSet(ColumnKindSet::NUMERIC_INTEGER),
        UnitSemantics::DimensionlessRatios,
        SamplingLaw::OrderedRegularWindows,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::MonotonicStrict,
        [0u8; 32],
    );
    let errors = verify_dataset_manifest(&bad);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::DatasetManifestMissingDatasetId
    )));
}

#[test]
fn context_rejects_empty_domain_tags() {
    let bad = build_task_manifest(
        "synthetic",
        TaskKind::TabularDatasetStructure,
        DomainTagSet(0),
        TargetEpisodeKindSet(TargetEpisodeKindSet::PRIMARY),
        WitnessRoleSet(WitnessRoleSet::PRIMARY),
        WitnessRoleSet(0),
        StrictnessLevel::Phase5_6,
    );
    let errors = verify_task_manifest(&bad);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ContextVerifyErrorKind::TaskManifestDomainTagsEmpty)));
}

#[test]
fn context_rejects_zero_schema_hash() {
    let bad = build_dataset_manifest(
        "synthetic",
        ArtifactFixedness::StreamingMutable,
        [0u8; 32],
        ColumnKindSet(ColumnKindSet::CATEGORICAL),
        UnitSemantics::CategoricalLabelsOnly,
        SamplingLaw::UnorderedRowSet,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::NoneDeclared,
        [0u8; 32],
    );
    let errors = verify_dataset_manifest(&bad);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::DatasetManifestSchemaHashZero
    )));
}

#[test]
fn context_rejects_missing_corpus_hash() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let mut c = ActivationContextV1 {
        corpus_hash_v1: [0u8; 32],
        registry_hash_v2: KNOWN_S12_REGISTRY_HASH_V2,
        task_manifest_hash_v1: task.task_manifest_hash_v1,
        dataset_manifest_hash_v1: dataset.dataset_manifest_hash_v1,
        coverage_hole_hash_v1: compute_coverage_hole_hash_v1(&collect_coverage_holes()),
        detector_contraindication_hash_v1: compute_contraindication_hash_v1(
            &collect_contraindications(),
        ),
        activation_context_hash_v1: [0u8; 32],
    };
    c.activation_context_hash_v1 = compute_activation_context_hash_v1(&c);
    let errors = verify_activation_context(&c, &task, &dataset);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ContextVerifyErrorKind::ContextMissingCorpusHash)));
}

#[test]
fn context_rejects_missing_registry_hash() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let mut c = ActivationContextV1 {
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        registry_hash_v2: [0u8; 32],
        task_manifest_hash_v1: task.task_manifest_hash_v1,
        dataset_manifest_hash_v1: dataset.dataset_manifest_hash_v1,
        coverage_hole_hash_v1: compute_coverage_hole_hash_v1(&collect_coverage_holes()),
        detector_contraindication_hash_v1: compute_contraindication_hash_v1(
            &collect_contraindications(),
        ),
        activation_context_hash_v1: [0u8; 32],
    };
    c.activation_context_hash_v1 = compute_activation_context_hash_v1(&c);
    let errors = verify_activation_context(&c, &task, &dataset);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ContextVerifyErrorKind::ContextMissingRegistryHash)));
}

#[test]
fn verifier_rejects_manifest_hash_mismatch() {
    let mut m = seed_task_manifest();
    // Mutate the stored hash so it no longer matches the body.
    m.task_manifest_hash_v1[0] ^= 0xff;
    let errors = verify_task_manifest(&m);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ContextVerifyErrorKind::ManifestHashMismatch)));
}

// ---------------------------------------------------------------
// Activation crosscheck — spectral detector path
// ---------------------------------------------------------------

#[test]
fn activation_rejects_enabled_spectral_detector_without_sampling_law() {
    let dataset_no_sampling = build_dataset_manifest(
        "synthetic_no_sampling",
        ArtifactFixedness::FixedEventCatalog,
        [0xa1; 32],
        ColumnKindSet(ColumnKindSet::NUMERIC_INTEGER | ColumnKindSet::LATENCY),
        UnitSemantics::LatencyMillisecondsAndErrorIndicator,
        SamplingLaw::NoSamplingLawDeclared, // the defect
        MissingnessProfile::NoneDeclared,
        TimestampLaw::MonotonicStrict,
        [0xa2; 32],
    );
    let id = DetectorCanonicalId(33);
    let enabled = [id];
    let spectral = [id];
    let unit_sensitive: [DetectorCanonicalId; 0] = [];
    let errors = verify_activation_against_context(
        &enabled,
        &spectral,
        &unit_sensitive,
        &dataset_no_sampling,
    );
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ContextVerifyErrorKind::SpectralDetectorWithoutSamplingLaw { canonical_id }
            if canonical_id == id
    )));
}

// ---------------------------------------------------------------
// Rendering determinism
// ---------------------------------------------------------------

#[test]
fn render_task_manifest_text_is_byte_stable() {
    let m = seed_task_manifest();
    assert_eq!(render_task_manifest_text(&m), render_task_manifest_text(&m));
}

#[test]
fn render_task_manifest_json_is_byte_stable() {
    let m = seed_task_manifest();
    assert_eq!(render_task_manifest_json(&m), render_task_manifest_json(&m));
}

#[test]
fn render_dataset_manifest_text_is_byte_stable() {
    let m = seed_dataset_manifest();
    assert_eq!(
        render_dataset_manifest_text(&m),
        render_dataset_manifest_text(&m)
    );
}

#[test]
fn render_dataset_manifest_json_is_byte_stable() {
    let m = seed_dataset_manifest();
    assert_eq!(
        render_dataset_manifest_json(&m),
        render_dataset_manifest_json(&m)
    );
}

#[test]
fn render_activation_context_text_is_byte_stable() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    assert_eq!(
        render_activation_context_text(&c),
        render_activation_context_text(&c)
    );
}

#[test]
fn render_activation_context_json_is_byte_stable() {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        compute_coverage_hole_hash_v1(&coverage),
        compute_contraindication_hash_v1(&contras),
    );
    assert_eq!(
        render_activation_context_json(&c),
        render_activation_context_json(&c)
    );
}

// ---------------------------------------------------------------
// Wire-name stability
// ---------------------------------------------------------------

#[test]
fn task_kind_wire_names_are_stable() {
    assert_eq!(
        TaskKind::DebugTraceResidualCourt.as_str(),
        "DebugTraceResidualCourt"
    );
    assert_eq!(
        TaskKind::TabularDatasetStructure.as_str(),
        "TabularDatasetStructure"
    );
    assert_eq!(
        TaskKind::TimeSeriesAnomalyCourt.as_str(),
        "TimeSeriesAnomalyCourt"
    );
    assert!(TaskKind::DebugTraceResidualCourt.is_time_series());
    assert!(!TaskKind::TabularDatasetStructure.is_time_series());
}

#[test]
fn artifact_fixedness_wire_names_are_stable() {
    assert_eq!(ArtifactFixedness::FixedBytes.as_str(), "FixedBytes");
    assert_eq!(
        ArtifactFixedness::FixedEventCatalog.as_str(),
        "FixedEventCatalog"
    );
    assert_eq!(
        ArtifactFixedness::UnfixedExternalReference.as_str(),
        "UnfixedExternalReference"
    );
    assert!(ArtifactFixedness::FixedBytes.requires_source_artifact_hash());
    assert!(!ArtifactFixedness::StreamingMutable.requires_source_artifact_hash());
}

#[test]
fn unit_semantics_wire_names_are_stable() {
    assert_eq!(UnitSemantics::NoUnitsDeclared.as_str(), "NoUnitsDeclared");
    assert_eq!(
        UnitSemantics::LatencyMillisecondsAndErrorIndicator.as_str(),
        "LatencyMillisecondsAndErrorIndicator"
    );
    assert!(!UnitSemantics::NoUnitsDeclared.declares_units());
    assert!(UnitSemantics::PerColumnPhysicalUnits.declares_units());
}

#[test]
fn sampling_law_wire_names_are_stable() {
    assert_eq!(
        SamplingLaw::NoSamplingLawDeclared.as_str(),
        "NoSamplingLawDeclared"
    );
    assert_eq!(
        SamplingLaw::OrderedRegularWindows.as_str(),
        "OrderedRegularWindows"
    );
    assert!(!SamplingLaw::NoSamplingLawDeclared.declares_sampling());
    assert!(SamplingLaw::OrderedRegularWindows.declares_sampling());
}

// ---------------------------------------------------------------
// Upstream-anchor preservation
// ---------------------------------------------------------------

/// S1.3c MUST NOT mutate any upstream hash anchor. Building
/// and verifying contexts leaves corpus / registry / coverage
/// / contraindication hashes byte-identical.
#[test]
fn building_contexts_does_not_mutate_upstream_anchors() {
    let corpus_before = compute_corpus_hash_v1().bytes;
    let coverage_before = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let contra_before = compute_contraindication_hash_v1(&collect_contraindications());
    let docket_before = collect_challenge_docket();
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let _c = build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        coverage_before,
        contra_before,
    );
    let corpus_after = compute_corpus_hash_v1().bytes;
    let coverage_after = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let contra_after = compute_contraindication_hash_v1(&collect_contraindications());
    let docket_after = collect_challenge_docket();
    assert_eq!(corpus_before, corpus_after);
    assert_eq!(coverage_before, coverage_after);
    assert_eq!(contra_before, contra_after);
    assert_eq!(
        docket_before.challenges.len(),
        docket_after.challenges.len()
    );
}
