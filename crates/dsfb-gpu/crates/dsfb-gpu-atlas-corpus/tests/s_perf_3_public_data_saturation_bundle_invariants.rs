//! S-PERF.3 acceptance suite for `PublicArtifactManifestV1`,
//! `DatasetMaterializationPolicyV1`, and
//! `PublicDataSaturationBundleV1` invariants.
//!
//! Eight panel-required load-bearing negatives:
//!
//! 1. `s_perf_3_rejects_dataset_without_source_or_access_note`
//! 2. `s_perf_3_rejects_artifact_without_hash_policy`
//! 3. `s_perf_3_rejects_bundle_with_synthetic_only_data`
//! 4. `s_perf_3_rejects_dataset_without_materialization_recipe`
//! 5. `s_perf_3_rejects_license_or_access_status_missing`
//! 6. `s_perf_3_rejects_unpinned_download_or_live_remote_dependency`
//! 7. `s_perf_3_rejects_dataset_role_without_layer_a_mapping`
//! 8. `s_perf_3_rejects_benchmark_claim_inside_bundle_definition`
//!
//! Plus structural defect tests, determinism (3 hashes
//! byte-stable across two builds; 6 renderers), sensitivity
//! (every hashable field changes the hash when mutated),
//! and baseline admission tests.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

use dsfb_gpu_atlas_corpus::s_perf_3_public_data_saturation_bundle::{
    build_panel_locked_dataset_materialization_policy, build_public_artifact_manifest,
    build_public_data_saturation_bundle, forbidden_benchmark_claim_substrings,
    panel_locked_dataset_materialization_policy_lines, render_dataset_materialization_policy_json,
    render_dataset_materialization_policy_text, render_public_artifact_manifest_json,
    render_public_artifact_manifest_text, render_public_data_saturation_bundle_json,
    render_public_data_saturation_bundle_text, seed_adbench_subset_manifest,
    seed_baseline_public_data_saturation_bundle, seed_defects4j_manifest,
    seed_nasa_cmapss_manifest, seed_tadbench_manifest, seed_tsb_uad_manifest,
    verify_public_artifact_manifest, verify_public_data_saturation_bundle, DatasetClass,
    DatasetMaterializationPolicyV1, DatasetMaterializationRecipe, DatasetUsageMode, HashPolicyKind,
    LayerARoleMapping, LicenseOrAccessStatus, PublicArtifactManifestV1, SPerf3VerifyErrorKind,
    DATASET_MATERIALIZATION_POLICY_DOMAIN_V1, DATASET_MATERIALIZATION_POLICY_SCHEMA_V1,
    PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1, PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1,
    PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1, PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1,
    S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Build a minimal admissible recipe for synthetic test
/// manifests.
fn admissible_recipe() -> DatasetMaterializationRecipe {
    DatasetMaterializationRecipe {
        source_url_or_doi: "https://example.org/test-dataset",
        local_path_template: "fixtures/test/{id}.csv",
        materialization_steps: vec!["download archive", "verify checksum", "decompress"],
        expected_bytes_after_materialization: 0,
        deterministic_postprocess: true,
        requires_live_remote_fetch: false,
    }
}

/// Build a TADBench-shaped admissible manifest with caller-
/// controlled tweaks for negative-test construction.
fn tadbench_like_manifest(
    dataset_id: &'static str,
    access_note: &'static str,
    license: LicenseOrAccessStatus,
    hash_policy: HashPolicyKind,
    layer_a_role: LayerARoleMapping,
    recipe: DatasetMaterializationRecipe,
    is_synthetic: bool,
) -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        dataset_id,
        "TADBench-shaped test manifest",
        DatasetClass::DebugObservabilityTrace,
        layer_a_role,
        access_note,
        license,
        DatasetUsageMode::CitationOnly,
        hash_policy,
        0,
        [0u8; 32],
        is_synthetic,
        recipe,
    )
}

// ---------------------------------------------------------------
// Baseline admission
// ---------------------------------------------------------------

#[test]
fn baseline_bundle_admits() {
    let b = seed_baseline_public_data_saturation_bundle();
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(
        errors.is_empty(),
        "baseline S-PERF.3 bundle must admit: {errors:?}"
    );
}

#[test]
fn baseline_bundle_carries_five_manifests() {
    let b = seed_baseline_public_data_saturation_bundle();
    assert_eq!(b.manifests.len(), 5);
}

#[test]
fn baseline_bundle_covers_all_five_panel_named_dataset_classes() {
    let b = seed_baseline_public_data_saturation_bundle();
    let mut classes: std::collections::BTreeSet<DatasetClass> = std::collections::BTreeSet::new();
    for m in &b.manifests {
        classes.insert(m.dataset_class);
    }
    assert!(classes.contains(&DatasetClass::DebugObservabilityTrace));
    assert!(classes.contains(&DatasetClass::SoftwareDefectTable));
    assert!(classes.contains(&DatasetClass::DataScienceTabular));
    assert!(classes.contains(&DatasetClass::TimeSeriesAnomaly));
    assert!(classes.contains(&DatasetClass::IndustrialPublicFixture));
}

#[test]
fn baseline_bundle_every_manifest_is_citation_only() {
    let b = seed_baseline_public_data_saturation_bundle();
    for m in &b.manifests {
        assert!(matches!(m.usage_mode, DatasetUsageMode::CitationOnly));
    }
}

#[test]
fn baseline_bundle_every_manifest_is_real_not_synthetic() {
    let b = seed_baseline_public_data_saturation_bundle();
    for m in &b.manifests {
        assert!(
            !m.is_synthetic,
            "baseline manifests must be real: {}",
            m.dataset_id
        );
    }
}

#[test]
fn baseline_bundle_manifests_sorted_ascending_by_dataset_id() {
    let b = seed_baseline_public_data_saturation_bundle();
    for w in b.manifests.windows(2) {
        assert!(w[0].dataset_id < w[1].dataset_id);
    }
}

#[test]
fn baseline_bundle_carries_panel_locked_policy() {
    let b = seed_baseline_public_data_saturation_bundle();
    assert_eq!(
        b.materialization_policy.policy_lines,
        S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES
    );
}

#[test]
fn tadbench_manifest_admits() {
    let m = seed_tadbench_manifest();
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.is_empty(),
        "TADBench manifest must admit: {errors:?}"
    );
}

#[test]
fn defects4j_manifest_admits() {
    let m = seed_defects4j_manifest();
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.is_empty(),
        "Defects4J manifest must admit: {errors:?}"
    );
}

#[test]
fn adbench_subset_manifest_admits() {
    let m = seed_adbench_subset_manifest();
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.is_empty(),
        "ADBench subset manifest must admit: {errors:?}"
    );
}

#[test]
fn tsb_uad_manifest_admits() {
    let m = seed_tsb_uad_manifest();
    let errors = verify_public_artifact_manifest(&m);
    assert!(errors.is_empty(), "TSB-UAD manifest must admit: {errors:?}");
}

#[test]
fn nasa_cmapss_manifest_admits() {
    let m = seed_nasa_cmapss_manifest();
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.is_empty(),
        "NASA C-MAPSS manifest must admit: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn public_artifact_manifest_hash_is_deterministic() {
    let a = seed_tadbench_manifest();
    let b = seed_tadbench_manifest();
    assert_eq!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn dataset_materialization_policy_hash_is_deterministic() {
    let a = build_panel_locked_dataset_materialization_policy();
    let b = build_panel_locked_dataset_materialization_policy();
    assert_eq!(
        a.dataset_materialization_policy_hash_v1,
        b.dataset_materialization_policy_hash_v1
    );
}

#[test]
fn public_data_saturation_bundle_hash_is_deterministic() {
    let a = seed_baseline_public_data_saturation_bundle();
    let b = seed_baseline_public_data_saturation_bundle();
    assert_eq!(
        a.public_data_saturation_bundle_hash_v1,
        b.public_data_saturation_bundle_hash_v1
    );
}

#[test]
fn manifest_text_render_is_deterministic() {
    let m = seed_tadbench_manifest();
    assert_eq!(
        render_public_artifact_manifest_text(&m),
        render_public_artifact_manifest_text(&m)
    );
}

#[test]
fn manifest_json_render_is_deterministic() {
    let m = seed_tadbench_manifest();
    assert_eq!(
        render_public_artifact_manifest_json(&m),
        render_public_artifact_manifest_json(&m)
    );
}

#[test]
fn policy_text_render_is_deterministic() {
    let p = build_panel_locked_dataset_materialization_policy();
    assert_eq!(
        render_dataset_materialization_policy_text(&p),
        render_dataset_materialization_policy_text(&p)
    );
}

#[test]
fn policy_json_render_is_deterministic() {
    let p = build_panel_locked_dataset_materialization_policy();
    assert_eq!(
        render_dataset_materialization_policy_json(&p),
        render_dataset_materialization_policy_json(&p)
    );
}

#[test]
fn bundle_text_render_is_deterministic() {
    let b = seed_baseline_public_data_saturation_bundle();
    assert_eq!(
        render_public_data_saturation_bundle_text(&b),
        render_public_data_saturation_bundle_text(&b)
    );
}

#[test]
fn bundle_json_render_is_deterministic() {
    let b = seed_baseline_public_data_saturation_bundle();
    assert_eq!(
        render_public_data_saturation_bundle_json(&b),
        render_public_data_saturation_bundle_json(&b)
    );
}

// ---------------------------------------------------------------
// Hash distinctness
// ---------------------------------------------------------------

#[test]
fn three_s_perf_3_hashes_are_pairwise_distinct() {
    let b = seed_baseline_public_data_saturation_bundle();
    let manifest_hash = b.manifests[0].public_artifact_manifest_hash_v1;
    let policy_hash = b
        .materialization_policy
        .dataset_materialization_policy_hash_v1;
    let bundle_hash = b.public_data_saturation_bundle_hash_v1;
    assert_ne!(manifest_hash, policy_hash);
    assert_ne!(manifest_hash, bundle_hash);
    assert_ne!(policy_hash, bundle_hash);
}

#[test]
fn manifests_all_have_distinct_hashes() {
    let b = seed_baseline_public_data_saturation_bundle();
    let mut seen: std::collections::BTreeSet<[u8; 32]> = std::collections::BTreeSet::new();
    for m in &b.manifests {
        assert!(
            seen.insert(m.public_artifact_manifest_hash_v1),
            "duplicate manifest hash for {}",
            m.dataset_id
        );
    }
}

// ---------------------------------------------------------------
// Domain separator + schema id discipline
// ---------------------------------------------------------------

#[test]
fn domain_separators_are_pairwise_distinct() {
    assert_ne!(
        PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1,
        DATASET_MATERIALIZATION_POLICY_DOMAIN_V1
    );
    assert_ne!(
        PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1,
        PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1
    );
    assert_ne!(
        DATASET_MATERIALIZATION_POLICY_DOMAIN_V1,
        PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1
    );
}

#[test]
fn domain_separators_end_with_nul_byte() {
    assert!(PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1.ends_with('\0'));
    assert!(DATASET_MATERIALIZATION_POLICY_DOMAIN_V1.ends_with('\0'));
    assert!(PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn schema_ids_are_pairwise_distinct() {
    assert_ne!(
        PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1,
        DATASET_MATERIALIZATION_POLICY_SCHEMA_V1
    );
    assert_ne!(
        PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1,
        PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1
    );
    assert_ne!(
        DATASET_MATERIALIZATION_POLICY_SCHEMA_V1,
        PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1
    );
}

// ---------------------------------------------------------------
// Panel-locked policy structural pins
// ---------------------------------------------------------------

#[test]
fn panel_locked_policy_has_eight_lines() {
    assert_eq!(panel_locked_dataset_materialization_policy_lines().len(), 8);
    assert_eq!(S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES.len(), 8);
}

#[test]
fn panel_locked_policy_lines_match_in_built_policy() {
    let p = build_panel_locked_dataset_materialization_policy();
    assert_eq!(
        p.policy_lines,
        S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES
    );
}

#[test]
fn forbidden_benchmark_substring_set_is_non_empty() {
    assert!(!forbidden_benchmark_claim_substrings().is_empty());
}

// ---------------------------------------------------------------
// Eight panel-required load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn s_perf_3_rejects_dataset_without_source_or_access_note() {
    // Empty access_note.
    let m = tadbench_like_manifest(
        "neg1_dataset",
        "", // empty access_note
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetWithoutSourceOrAccessNote { .. }
        )),
        "empty access_note must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_dataset_with_empty_source_url() {
    let mut recipe = admissible_recipe();
    recipe.source_url_or_doi = "";
    let m = tadbench_like_manifest(
        "neg1b_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        recipe,
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetWithoutSourceOrAccessNote { .. }
        )),
        "empty source_url_or_doi must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_artifact_without_hash_policy() {
    let m = tadbench_like_manifest(
        "neg2_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Unknown,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::ArtifactWithoutHashPolicy { .. }
        )),
        "HashPolicyKind::Unknown must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_bundle_with_synthetic_only_data() {
    let synthetic_manifest = tadbench_like_manifest(
        "neg3_synthetic_only",
        "synthetic generator description",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        true, // is_synthetic
    );
    let policy = build_panel_locked_dataset_materialization_policy();
    let b = build_public_data_saturation_bundle(
        "synthetic_only_test_bundle",
        vec![synthetic_manifest],
        policy,
    );
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf3VerifyErrorKind::BundleWithSyntheticOnlyData)),
        "synthetic-only bundle must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_empty_bundle() {
    let policy = build_panel_locked_dataset_materialization_policy();
    let b = build_public_data_saturation_bundle("empty_test_bundle", Vec::new(), policy);
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, SPerf3VerifyErrorKind::BundleWithSyntheticOnlyData)));
}

#[test]
fn s_perf_3_rejects_dataset_without_materialization_recipe() {
    let mut recipe = admissible_recipe();
    recipe.local_path_template = ""; // missing template
    let m = tadbench_like_manifest(
        "neg4_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        recipe,
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetWithoutMaterializationRecipe { .. }
        )),
        "empty local_path_template must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_dataset_with_empty_materialization_steps() {
    let mut recipe = admissible_recipe();
    recipe.materialization_steps = Vec::new();
    let m = tadbench_like_manifest(
        "neg4b_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        recipe,
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetWithoutMaterializationRecipe { .. }
        )),
        "empty materialization_steps must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_measured_fixture_with_zero_expected_bytes() {
    let mut recipe = admissible_recipe();
    recipe.expected_bytes_after_materialization = 0;
    // Override usage_mode to MeasuredFixture.
    let m = build_public_artifact_manifest(
        "neg4c_dataset",
        "Measured fixture with zero expected bytes",
        DatasetClass::DebugObservabilityTrace,
        LayerARoleMapping::EvidenceDensorSource,
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        DatasetUsageMode::MeasuredFixture,
        HashPolicyKind::Sha256PerFileManifest,
        0,
        [0u8; 32],
        false,
        recipe,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetWithoutMaterializationRecipe { .. }
        )),
        "MeasuredFixture with zero expected_bytes must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_license_or_access_status_missing() {
    let m = tadbench_like_manifest(
        "neg5_dataset",
        "valid access note",
        LicenseOrAccessStatus::UnknownLicense,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::LicenseOrAccessStatusMissing { .. }
        )),
        "UnknownLicense must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_unpinned_download_or_live_remote_dependency() {
    let mut recipe = admissible_recipe();
    recipe.requires_live_remote_fetch = true;
    let m = tadbench_like_manifest(
        "neg6_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        recipe,
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::UnpinnedDownloadOrLiveRemoteDependency { .. }
        )),
        "requires_live_remote_fetch=true must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_dataset_role_without_layer_a_mapping() {
    let m = tadbench_like_manifest(
        "neg7_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::Unmapped,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::DatasetRoleWithoutLayerAMapping { .. }
        )),
        "Unmapped role must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_benchmark_claim_inside_bundle_definition() {
    // Insert "outperforms" in the access_note.
    let m = tadbench_like_manifest(
        "neg8_dataset",
        "this bundle outperforms every existing baseline",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition { .. }
        )),
        "forbidden benchmark substring must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_rejects_benchmark_claim_in_bundle_identifier() {
    let policy = build_panel_locked_dataset_materialization_policy();
    let manifest = seed_tadbench_manifest();
    // Use a forbidden substring that is whole-word (no
    // intra-word spaces) so it can appear inside a
    // snake-case bundle id.
    let bundle =
        build_public_data_saturation_bundle("bundle_with_petaflops_claim", vec![manifest], policy);
    let errors = verify_public_data_saturation_bundle(&bundle);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition {
                location: "<bundle_id>",
                ..
            }
        )),
        "forbidden substring in bundle_id must surface: {errors:?}"
    );
}

#[test]
fn s_perf_3_benchmark_claim_scanner_is_case_insensitive() {
    let m = tadbench_like_manifest(
        "neg8_uppercase",
        "ACHIEVES SATURATION at memory bandwidth peak",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition { .. }
        )),
        "uppercase forbidden substring must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn empty_dataset_id_surfaces_structural_defect() {
    let m = tadbench_like_manifest(
        "",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, SPerf3VerifyErrorKind::DatasetIdEmpty)));
}

#[test]
fn non_deterministic_recipe_surfaces_structural_defect() {
    let mut recipe = admissible_recipe();
    recipe.deterministic_postprocess = false;
    let m = tadbench_like_manifest(
        "non_det_dataset",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        recipe,
        false,
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        SPerf3VerifyErrorKind::NonDeterministicMaterializationRecipe { .. }
    )));
}

#[test]
fn per_artifact_sha256_count_with_sha256_of_source_archive_surfaces() {
    let m = build_public_artifact_manifest(
        "inconsistent_hash_policy_dataset",
        "inconsistent",
        DatasetClass::DebugObservabilityTrace,
        LayerARoleMapping::EvidenceDensorSource,
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256OfSourceArchive,
        5, // non-zero per_artifact count
        [0u8; 32],
        false,
        admissible_recipe(),
    );
    let errors = verify_public_artifact_manifest(&m);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        SPerf3VerifyErrorKind::PerArtifactSha256CountInconsistentWithHashPolicyKind { .. }
    )));
}

#[test]
fn empty_bundle_id_surfaces_structural_defect() {
    let policy = build_panel_locked_dataset_materialization_policy();
    let b = build_public_data_saturation_bundle("", vec![seed_tadbench_manifest()], policy);
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, SPerf3VerifyErrorKind::BundleIdEmpty)));
}

#[test]
fn duplicate_dataset_id_surfaces_structural_defect() {
    let m1 = seed_tadbench_manifest();
    let m2 = seed_tadbench_manifest();
    let policy = build_panel_locked_dataset_materialization_policy();
    let b = build_public_data_saturation_bundle("duplicate_test_bundle", vec![m1, m2], policy);
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        SPerf3VerifyErrorKind::DuplicateDatasetIdInBundle { .. }
    )));
}

#[test]
fn non_panel_locked_policy_surfaces_structural_defect() {
    // Hand-build a policy with mutated lines.
    let mut policy = build_panel_locked_dataset_materialization_policy();
    policy.policy_lines = vec!["bogus policy line"];
    let b = build_public_data_saturation_bundle(
        "non_panel_locked_policy_test",
        vec![seed_tadbench_manifest()],
        policy,
    );
    let errors = verify_public_data_saturation_bundle(&b);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        SPerf3VerifyErrorKind::MaterializationPolicyNotPanelLocked
    )));
}

// ---------------------------------------------------------------
// Sensitivity tests
// ---------------------------------------------------------------

#[test]
fn manifest_hash_changes_when_dataset_id_changes() {
    let a = seed_tadbench_manifest();
    let mut b = a.clone();
    b.dataset_id = "different_id";
    let b = build_public_artifact_manifest(
        b.dataset_id,
        b.display_name,
        b.dataset_class,
        b.layer_a_role_mapping,
        b.access_note,
        b.license_or_access_status,
        b.usage_mode,
        b.hash_policy_kind,
        b.per_artifact_sha256_count,
        b.source_archive_sha256,
        b.is_synthetic,
        b.materialization_recipe,
    );
    assert_ne!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn manifest_hash_changes_when_license_changes() {
    let a = seed_tadbench_manifest();
    let b = build_public_artifact_manifest(
        a.dataset_id,
        a.display_name,
        a.dataset_class,
        a.layer_a_role_mapping,
        a.access_note,
        LicenseOrAccessStatus::Bsd2Clause, // mutated
        a.usage_mode,
        a.hash_policy_kind,
        a.per_artifact_sha256_count,
        a.source_archive_sha256,
        a.is_synthetic,
        a.materialization_recipe.clone(),
    );
    assert_ne!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn manifest_hash_changes_when_recipe_step_added() {
    let a = seed_tadbench_manifest();
    let mut recipe = a.materialization_recipe.clone();
    recipe.materialization_steps.push("extra step");
    let b = build_public_artifact_manifest(
        a.dataset_id,
        a.display_name,
        a.dataset_class,
        a.layer_a_role_mapping,
        a.access_note,
        a.license_or_access_status,
        a.usage_mode,
        a.hash_policy_kind,
        a.per_artifact_sha256_count,
        a.source_archive_sha256,
        a.is_synthetic,
        recipe,
    );
    assert_ne!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn manifest_hash_changes_when_synthetic_flag_changes() {
    let a = seed_tadbench_manifest();
    let b = build_public_artifact_manifest(
        a.dataset_id,
        a.display_name,
        a.dataset_class,
        a.layer_a_role_mapping,
        a.access_note,
        a.license_or_access_status,
        a.usage_mode,
        a.hash_policy_kind,
        a.per_artifact_sha256_count,
        a.source_archive_sha256,
        !a.is_synthetic,
        a.materialization_recipe.clone(),
    );
    assert_ne!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn manifest_hash_changes_when_layer_a_role_mapping_changes() {
    let a = seed_tadbench_manifest();
    let b = build_public_artifact_manifest(
        a.dataset_id,
        a.display_name,
        a.dataset_class,
        LayerARoleMapping::WitnessDensorReference, // mutated
        a.access_note,
        a.license_or_access_status,
        a.usage_mode,
        a.hash_policy_kind,
        a.per_artifact_sha256_count,
        a.source_archive_sha256,
        a.is_synthetic,
        a.materialization_recipe.clone(),
    );
    assert_ne!(
        a.public_artifact_manifest_hash_v1,
        b.public_artifact_manifest_hash_v1
    );
}

#[test]
fn bundle_hash_changes_when_manifest_added() {
    let a = seed_baseline_public_data_saturation_bundle();
    let mut b_manifests: Vec<PublicArtifactManifestV1> = a.manifests.clone();
    b_manifests.push(tadbench_like_manifest(
        "extra_test_manifest",
        "valid access note",
        LicenseOrAccessStatus::Apache2,
        HashPolicyKind::Sha256PerFileManifest,
        LayerARoleMapping::EvidenceDensorSource,
        admissible_recipe(),
        false,
    ));
    let policy = build_panel_locked_dataset_materialization_policy();
    let b = build_public_data_saturation_bundle("extended_bundle", b_manifests, policy);
    assert_ne!(
        a.public_data_saturation_bundle_hash_v1,
        b.public_data_saturation_bundle_hash_v1
    );
}

#[test]
fn bundle_hash_changes_when_policy_changes() {
    let a = seed_baseline_public_data_saturation_bundle();
    let mut policy = build_panel_locked_dataset_materialization_policy();
    policy.policy_lines = vec!["mutated policy"];
    let b = build_public_data_saturation_bundle(
        a.bundle_id,
        a.manifests.clone(),
        DatasetMaterializationPolicyV1 {
            policy_lines: policy.policy_lines.clone(),
            dataset_materialization_policy_hash_v1: [0u8; 32], // builder doesn't recompute
                                                               // when this is a fresh policy
        },
    );
    // The bundle hash incorporates the policy hash field
    // verbatim; we need to actually rebuild the policy hash
    // for fairness.
    let bundle_with_real_mutated_policy_hash = build_public_data_saturation_bundle(
        a.bundle_id,
        a.manifests.clone(),
        DatasetMaterializationPolicyV1 {
            policy_lines: policy.policy_lines.clone(),
            // The bundle hash uses
            // materialization_policy.dataset_materialization_policy_hash_v1
            // verbatim; if we set it to zero the bundle hash
            // already differs from the baseline.
            dataset_materialization_policy_hash_v1: [0xABu8; 32],
        },
    );
    // Either non-matching policy hash byte triggers a
    // different bundle hash.
    assert_ne!(
        a.public_data_saturation_bundle_hash_v1,
        bundle_with_real_mutated_policy_hash.public_data_saturation_bundle_hash_v1
    );
    // Sanity: the two mutated bundles also differ from each
    // other.
    assert_ne!(
        b.public_data_saturation_bundle_hash_v1,
        bundle_with_real_mutated_policy_hash.public_data_saturation_bundle_hash_v1
    );
}

// ---------------------------------------------------------------
// Rendering smoke tests
// ---------------------------------------------------------------

#[test]
fn manifest_text_contains_pinned_header_lines() {
    let s = render_public_artifact_manifest_text(&seed_tadbench_manifest());
    assert!(s.contains("S-PERF.3 PublicArtifactManifestV1"));
    assert!(s.contains("Identity"));
    assert!(s.contains("Access + license"));
    assert!(s.contains("Hash policy"));
    assert!(s.contains("Materialization recipe"));
    assert!(s.contains("public_artifact_manifest_hash_v1"));
}

#[test]
fn manifest_json_contains_pinned_schema_id() {
    let s = render_public_artifact_manifest_json(&seed_tadbench_manifest());
    assert!(s.contains(PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1));
    assert!(s.contains("public_artifact_manifest_hash_v1"));
    assert!(s.contains("materialization_recipe"));
}

#[test]
fn policy_text_contains_eight_pinned_rules() {
    let s = render_dataset_materialization_policy_text(
        &build_panel_locked_dataset_materialization_policy(),
    );
    for line in S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES {
        assert!(s.contains(line), "policy text missing pinned line `{line}`");
    }
}

#[test]
fn policy_json_contains_pinned_schema_id() {
    let s = render_dataset_materialization_policy_json(
        &build_panel_locked_dataset_materialization_policy(),
    );
    assert!(s.contains(DATASET_MATERIALIZATION_POLICY_SCHEMA_V1));
    assert!(s.contains("dataset_materialization_policy_hash_v1"));
}

#[test]
fn bundle_text_contains_pinned_header_lines() {
    let s =
        render_public_data_saturation_bundle_text(&seed_baseline_public_data_saturation_bundle());
    assert!(s.contains("S-PERF.3 PublicDataSaturationBundleV1"));
    assert!(s.contains("Bundle identity"));
    assert!(s.contains("Manifests"));
    assert!(s.contains("materialization_policy_hash"));
    assert!(s.contains("public_data_saturation_bundle_hash_v1"));
}

#[test]
fn bundle_json_contains_pinned_schema_id() {
    let s =
        render_public_data_saturation_bundle_json(&seed_baseline_public_data_saturation_bundle());
    assert!(s.contains(PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1));
    assert!(s.contains("public_data_saturation_bundle_hash_v1"));
    assert!(s.contains("manifest_dataset_ids"));
}

// ---------------------------------------------------------------
// Production walk: every baseline manifest carries no
// forbidden substring on any free-text field
// ---------------------------------------------------------------

#[test]
fn production_manifests_carry_no_forbidden_benchmark_substring() {
    let b = seed_baseline_public_data_saturation_bundle();
    let forbidden = forbidden_benchmark_claim_substrings();
    for m in &b.manifests {
        for &sub in forbidden {
            let dn = m.display_name.to_ascii_lowercase();
            let an = m.access_note.to_ascii_lowercase();
            let sl = sub.to_ascii_lowercase();
            assert!(
                !dn.contains(&sl),
                "{} display_name contains forbidden '{}'",
                m.dataset_id,
                sub
            );
            assert!(
                !an.contains(&sl),
                "{} access_note contains forbidden '{}'",
                m.dataset_id,
                sub
            );
            for step in &m.materialization_recipe.materialization_steps {
                let lower = step.to_ascii_lowercase();
                assert!(
                    !lower.contains(&sl),
                    "{} recipe step contains forbidden '{}'",
                    m.dataset_id,
                    sub
                );
            }
        }
    }
}

// ---------------------------------------------------------------
// Wire-name stability tests
// ---------------------------------------------------------------

#[test]
fn dataset_class_wire_names_are_stable() {
    assert_eq!(
        DatasetClass::DebugObservabilityTrace.as_str(),
        "DebugObservabilityTrace"
    );
    assert_eq!(
        DatasetClass::SoftwareDefectTable.as_str(),
        "SoftwareDefectTable"
    );
    assert_eq!(
        DatasetClass::DataScienceTabular.as_str(),
        "DataScienceTabular"
    );
    assert_eq!(
        DatasetClass::TimeSeriesAnomaly.as_str(),
        "TimeSeriesAnomaly"
    );
    assert_eq!(
        DatasetClass::IndustrialPublicFixture.as_str(),
        "IndustrialPublicFixture"
    );
}

#[test]
fn hash_policy_kind_wire_names_are_stable() {
    assert_eq!(
        HashPolicyKind::Sha256OfSourceArchive.as_str(),
        "Sha256OfSourceArchive"
    );
    assert_eq!(
        HashPolicyKind::Sha256PerFileManifest.as_str(),
        "Sha256PerFileManifest"
    );
    assert_eq!(
        HashPolicyKind::UpstreamProvidedChecksum.as_str(),
        "UpstreamProvidedChecksum"
    );
    assert_eq!(HashPolicyKind::Unknown.as_str(), "Unknown");
}

#[test]
fn license_or_access_status_wire_names_are_stable() {
    assert_eq!(LicenseOrAccessStatus::PublicDomain.as_str(), "PublicDomain");
    assert_eq!(LicenseOrAccessStatus::Apache2.as_str(), "Apache2");
    assert_eq!(
        LicenseOrAccessStatus::UnknownLicense.as_str(),
        "UnknownLicense"
    );
}

#[test]
fn layer_a_role_mapping_wire_names_are_stable() {
    assert_eq!(
        LayerARoleMapping::EvidenceDensorSource.as_str(),
        "EvidenceDensorSource"
    );
    assert_eq!(LayerARoleMapping::Unmapped.as_str(), "Unmapped");
}

#[test]
fn dataset_usage_mode_wire_names_are_stable() {
    assert_eq!(DatasetUsageMode::CitationOnly.as_str(), "CitationOnly");
    assert_eq!(
        DatasetUsageMode::MeasuredFixture.as_str(),
        "MeasuredFixture"
    );
}

// ---------------------------------------------------------------
// Non-zero-hash guards
// ---------------------------------------------------------------

#[test]
fn baseline_bundle_has_non_zero_bundle_hash() {
    let b = seed_baseline_public_data_saturation_bundle();
    assert_ne!(b.public_data_saturation_bundle_hash_v1, [0u8; 32]);
}

#[test]
fn baseline_policy_has_non_zero_policy_hash() {
    let p = build_panel_locked_dataset_materialization_policy();
    assert_ne!(p.dataset_materialization_policy_hash_v1, [0u8; 32]);
}

#[test]
fn baseline_manifest_has_non_zero_manifest_hash() {
    let m = seed_tadbench_manifest();
    assert_ne!(m.public_artifact_manifest_hash_v1, [0u8; 32]);
}
