//! S-PERF.2 acceptance suite for `LayerAResidentPipelineV1`,
//! `LayerADeviceResidencyReceiptV1`, and
//! `LayerATrafficReceiptV1` invariants.
//!
//! Eight panel-required load-bearing negatives:
//!
//! 1. `s_perf_2_rejects_layer_a_receipt_with_host_json_time`
//! 2. `s_perf_2_rejects_layer_a_receipt_with_casefile_materialization_time`
//! 3. `s_perf_2_rejects_pipeline_without_device_residency_declaration`
//! 4. `s_perf_2_rejects_d2h_full_witness_dump_when_summary_only_declared`
//! 5. `s_perf_2_rejects_missing_h2d_d2h_byte_accounting`
//! 6. `s_perf_2_rejects_cuda_timing_method_not_allowed_by_s_perf_1`
//! 7. `s_perf_2_rejects_layer_a_claim_without_device_traffic_receipt`
//! 8. `s_perf_2_rejects_pipeline_that_mutates_court_authority_hashes`
//!
//! Plus structural defect tests, determinism (3 hashes byte-
//! stable across two builds; 6 renderers), sensitivity
//! (every hashable field that the builder API exposes
//! changes the hash when mutated), and baseline admission
//! tests.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::fn_params_excessive_bools
)]

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::s_perf_1_device_traffic_receipt::{
    seed_baseline_uninstrumented_receipt, TimingMethod,
};
use dsfb_gpu_atlas_corpus::s_perf_2_layer_a_resident_pipeline::{
    build_layer_a_device_residency_receipt, build_layer_a_resident_pipeline,
    build_layer_a_traffic_receipt, panel_locked_device_identity_hash_helper,
    panel_locked_layer_a_canonical_stage_names, render_layer_a_device_residency_receipt_json,
    render_layer_a_device_residency_receipt_text, render_layer_a_resident_pipeline_json,
    render_layer_a_resident_pipeline_text, render_layer_a_traffic_receipt_json,
    render_layer_a_traffic_receipt_text, seed_baseline_layer_a_pipeline,
    seed_baseline_layer_a_residency_receipt, seed_baseline_layer_a_traffic_receipt,
    verify_layer_a_device_residency_receipt, verify_layer_a_resident_pipeline,
    verify_layer_a_traffic_receipt, DeviceResidencyClass, LayerADensorKind,
    LayerADensorResidencyDeclaration, SPerf2VerifyErrorKind, LAYER_A_CANONICAL_STAGE_NAMES,
    LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1, LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1,
    LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1, LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1,
    LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1, LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn corpus_anchor() -> [u8; 32] {
    compute_corpus_hash_v1().bytes
}

/// Build a Layer-A pipeline with caller-controlled forbidden
/// flags; everything else mirrors the panel-locked baseline.
fn build_test_pipeline(
    casefile_materialization_present: bool,
    host_transcript_present: bool,
    host_json_emission_present: bool,
    semantic_admission_present: bool,
    mutates_court_authority_hashes: bool,
) -> dsfb_gpu_atlas_corpus::s_perf_2_layer_a_resident_pipeline::LayerAResidentPipelineV1 {
    let baseline = seed_baseline_layer_a_pipeline();
    build_layer_a_resident_pipeline(
        baseline.pipeline_id,
        baseline.stage_names.clone(),
        baseline.residency_declarations.clone(),
        casefile_materialization_present,
        host_transcript_present,
        host_json_emission_present,
        semantic_admission_present,
        mutates_court_authority_hashes,
    )
}

// ---------------------------------------------------------------
// Baseline admission
// ---------------------------------------------------------------

#[test]
fn baseline_layer_a_pipeline_admits() {
    let p = seed_baseline_layer_a_pipeline();
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.is_empty(),
        "baseline Layer-A pipeline must admit: {errors:?}"
    );
}

#[test]
fn baseline_layer_a_residency_receipt_admits() {
    let p = seed_baseline_layer_a_pipeline();
    let r = seed_baseline_layer_a_residency_receipt();
    let errors = verify_layer_a_device_residency_receipt(&r, &p);
    assert!(
        errors.is_empty(),
        "baseline Layer-A residency receipt must admit: {errors:?}"
    );
}

#[test]
fn baseline_layer_a_traffic_receipt_admits() {
    let r = seed_baseline_layer_a_traffic_receipt();
    let errors = verify_layer_a_traffic_receipt(&r);
    assert!(
        errors.is_empty(),
        "baseline Layer-A traffic receipt must admit: {errors:?}"
    );
}

#[test]
fn baseline_pipeline_declares_panel_locked_five_canonical_stages() {
    let p = seed_baseline_layer_a_pipeline();
    assert_eq!(p.stage_count, 5);
    assert_eq!(p.stage_names, LAYER_A_CANONICAL_STAGE_NAMES);
}

#[test]
fn baseline_pipeline_declares_all_five_densor_kinds() {
    let p = seed_baseline_layer_a_pipeline();
    assert_eq!(p.residency_declarations.len(), 5);
}

#[test]
fn baseline_pipeline_evidence_witness_fusion_are_device_resident_only() {
    let p = seed_baseline_layer_a_pipeline();
    for d in &p.residency_declarations {
        match d.densor_kind {
            LayerADensorKind::Evidence | LayerADensorKind::Witness | LayerADensorKind::Fusion => {
                assert!(
                    matches!(d.residency_class, DeviceResidencyClass::DeviceResidentOnly),
                    "{} must be DeviceResidentOnly in baseline",
                    d.densor_kind.as_str()
                );
                assert_eq!(d.expected_max_d2h_bytes_per_catalog, 0);
            }
            LayerADensorKind::Candidate | LayerADensorKind::StageDigest => {
                assert!(matches!(
                    d.residency_class,
                    DeviceResidencyClass::DeviceResidentWithCompactD2H
                ));
                assert!(d.expected_max_d2h_bytes_per_catalog > 0);
            }
        }
    }
}

#[test]
fn baseline_pipeline_has_all_forbidden_flags_false() {
    let p = seed_baseline_layer_a_pipeline();
    assert!(!p.casefile_materialization_present);
    assert!(!p.host_transcript_present);
    assert!(!p.host_json_emission_present);
    assert!(!p.semantic_admission_present);
    assert!(!p.mutates_court_authority_hashes);
}

#[test]
fn baseline_traffic_receipt_references_s_perf_1_baseline() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let s_perf_1 = seed_baseline_uninstrumented_receipt();
    assert_eq!(
        traffic.device_traffic_receipt_hash_v1,
        s_perf_1.device_traffic_receipt_hash_v1
    );
    assert_eq!(traffic.inner_timing_method_wire_name, "CudaEvent");
}

#[test]
fn baseline_traffic_receipt_includes_corpus_anchor() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    assert!(traffic
        .court_authority_hash_anchors
        .contains(&corpus_anchor()));
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn layer_a_pipeline_hash_is_deterministic() {
    let a = seed_baseline_layer_a_pipeline();
    let b = seed_baseline_layer_a_pipeline();
    assert_eq!(
        a.layer_a_resident_pipeline_hash_v1,
        b.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn layer_a_residency_receipt_hash_is_deterministic() {
    let a = seed_baseline_layer_a_residency_receipt();
    let b = seed_baseline_layer_a_residency_receipt();
    assert_eq!(
        a.layer_a_device_residency_receipt_hash_v1,
        b.layer_a_device_residency_receipt_hash_v1
    );
}

#[test]
fn layer_a_traffic_receipt_hash_is_deterministic() {
    let a = seed_baseline_layer_a_traffic_receipt();
    let b = seed_baseline_layer_a_traffic_receipt();
    assert_eq!(
        a.layer_a_traffic_receipt_hash_v1,
        b.layer_a_traffic_receipt_hash_v1
    );
}

#[test]
fn pipeline_text_render_is_deterministic() {
    let p = seed_baseline_layer_a_pipeline();
    let a = render_layer_a_resident_pipeline_text(&p);
    let b = render_layer_a_resident_pipeline_text(&p);
    assert_eq!(a, b);
}

#[test]
fn pipeline_json_render_is_deterministic() {
    let p = seed_baseline_layer_a_pipeline();
    let a = render_layer_a_resident_pipeline_json(&p);
    let b = render_layer_a_resident_pipeline_json(&p);
    assert_eq!(a, b);
}

#[test]
fn residency_receipt_text_render_is_deterministic() {
    let r = seed_baseline_layer_a_residency_receipt();
    let a = render_layer_a_device_residency_receipt_text(&r);
    let b = render_layer_a_device_residency_receipt_text(&r);
    assert_eq!(a, b);
}

#[test]
fn residency_receipt_json_render_is_deterministic() {
    let r = seed_baseline_layer_a_residency_receipt();
    let a = render_layer_a_device_residency_receipt_json(&r);
    let b = render_layer_a_device_residency_receipt_json(&r);
    assert_eq!(a, b);
}

#[test]
fn traffic_receipt_text_render_is_deterministic() {
    let r = seed_baseline_layer_a_traffic_receipt();
    let a = render_layer_a_traffic_receipt_text(&r);
    let b = render_layer_a_traffic_receipt_text(&r);
    assert_eq!(a, b);
}

#[test]
fn traffic_receipt_json_render_is_deterministic() {
    let r = seed_baseline_layer_a_traffic_receipt();
    let a = render_layer_a_traffic_receipt_json(&r);
    let b = render_layer_a_traffic_receipt_json(&r);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Hash distinctness
// ---------------------------------------------------------------

#[test]
fn three_layer_a_hashes_are_pairwise_distinct() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let p_hash = traffic.pipeline.layer_a_resident_pipeline_hash_v1;
    let r_hash = traffic
        .residency_receipt
        .layer_a_device_residency_receipt_hash_v1;
    let t_hash = traffic.layer_a_traffic_receipt_hash_v1;
    assert_ne!(p_hash, r_hash);
    assert_ne!(p_hash, t_hash);
    assert_ne!(r_hash, t_hash);
}

#[test]
fn layer_a_hashes_differ_from_corpus_hash_v1() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    assert_ne!(
        traffic.pipeline.layer_a_resident_pipeline_hash_v1,
        corpus_anchor()
    );
    assert_ne!(traffic.layer_a_traffic_receipt_hash_v1, corpus_anchor());
}

#[test]
fn layer_a_pipeline_hash_differs_from_s_perf_1_device_traffic_receipt_hash() {
    let p = seed_baseline_layer_a_pipeline();
    let s_perf_1 = seed_baseline_uninstrumented_receipt();
    assert_ne!(
        p.layer_a_resident_pipeline_hash_v1,
        s_perf_1.device_traffic_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Domain separator + schema id discipline
// ---------------------------------------------------------------

#[test]
fn domain_separators_are_pairwise_distinct() {
    assert_ne!(
        LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1,
        LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1
    );
    assert_ne!(
        LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1,
        LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1
    );
    assert_ne!(
        LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1,
        LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1
    );
}

#[test]
fn domain_separators_end_with_nul_byte() {
    assert!(LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1.ends_with('\0'));
    assert!(LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1.ends_with('\0'));
    assert!(LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn schema_ids_are_pairwise_distinct() {
    assert_ne!(
        LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1,
        LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1
    );
    assert_ne!(
        LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1,
        LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1
    );
    assert_ne!(
        LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1,
        LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1
    );
}

// ---------------------------------------------------------------
// Panel-locked canonical stage names
// ---------------------------------------------------------------

#[test]
fn panel_locked_canonical_stage_names_match_in_baseline() {
    assert_eq!(
        panel_locked_layer_a_canonical_stage_names(),
        seed_baseline_layer_a_pipeline().stage_names.as_slice()
    );
}

#[test]
fn panel_locked_canonical_stage_names_count_is_five() {
    assert_eq!(panel_locked_layer_a_canonical_stage_names().len(), 5);
    assert_eq!(LAYER_A_CANONICAL_STAGE_NAMES.len(), 5);
}

#[test]
fn panel_locked_canonical_stage_names_in_order() {
    let names = panel_locked_layer_a_canonical_stage_names();
    assert_eq!(names[0], "EvidenceDensorProjection");
    assert_eq!(names[1], "WitnessDensorEvaluation");
    assert_eq!(names[2], "FusionDensorReduction");
    assert_eq!(names[3], "CandidateDensorCollapse");
    assert_eq!(names[4], "StageDigestEmission");
}

// ---------------------------------------------------------------
// Eight panel-required load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn s_perf_2_rejects_layer_a_receipt_with_host_json_time() {
    let p = build_test_pipeline(false, false, true, false, false);
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf2VerifyErrorKind::LayerAReceiptWithHostJsonTime)),
        "host_json_emission_present=true must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_layer_a_receipt_with_casefile_materialization_time() {
    let p = build_test_pipeline(true, false, false, false, false);
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::LayerAReceiptWithCasefileMaterializationTime
        )),
        "casefile_materialization_present=true must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_pipeline_without_device_residency_declaration() {
    // Hand-build a pipeline with empty residency_declarations.
    let p = build_layer_a_resident_pipeline(
        "test_empty_residency_pipeline",
        LAYER_A_CANONICAL_STAGE_NAMES.to_vec(),
        Vec::new(),
        false,
        false,
        false,
        false,
        false,
    );
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::PipelineWithoutDeviceResidencyDeclaration
        )),
        "empty residency_declarations must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_d2h_full_witness_dump_when_summary_only_declared() {
    let p = seed_baseline_layer_a_pipeline();
    // Construct a receipt with non-zero Witness D2H bytes —
    // Witness is panel-locked DeviceResidentOnly.
    let kinds = [
        LayerADensorKind::Evidence,
        LayerADensorKind::Witness,
        LayerADensorKind::Fusion,
        LayerADensorKind::Candidate,
        LayerADensorKind::StageDigest,
    ];
    let per_h2d: Vec<(LayerADensorKind, u64)> = kinds.iter().map(|k| (*k, 0u64)).collect();
    let per_d2h: Vec<(LayerADensorKind, u64)> = kinds
        .iter()
        .map(|k| {
            (
                *k,
                if matches!(k, LayerADensorKind::Witness) {
                    1_000_000u64
                } else {
                    0u64
                },
            )
        })
        .collect();
    let r = build_layer_a_device_residency_receipt(
        p.layer_a_resident_pipeline_hash_v1,
        per_h2d,
        per_d2h,
    );
    let errors = verify_layer_a_device_residency_receipt(&r, &p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::D2hFullWitnessDumpWhenSummaryOnlyDeclared {
                densor_kind_wire_name: "Witness",
                ..
            }
        )),
        "Witness D2H>0 with DeviceResidentOnly must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_missing_h2d_d2h_byte_accounting() {
    let p = seed_baseline_layer_a_pipeline();
    // Both per-densor lists empty → no byte accounting at all.
    let r = build_layer_a_device_residency_receipt(
        p.layer_a_resident_pipeline_hash_v1,
        Vec::new(),
        Vec::new(),
    );
    let errors = verify_layer_a_device_residency_receipt(&r, &p);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf2VerifyErrorKind::MissingH2dD2hByteAccounting)),
        "both empty per-densor lists must surface MissingH2dD2hByteAccounting: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_cuda_timing_method_not_allowed_by_s_perf_1() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    // Re-build with the inner timing method set to a forbidden value.
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        TimingMethod::HostJsonInclusiveTime.as_str(),
        baseline.court_authority_hash_anchors.clone(),
    );
    let errors = verify_layer_a_traffic_receipt(&mutated);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::CudaTimingMethodNotAllowedBySPerf1 { .. }
        )),
        "HostJsonInclusiveTime as inner timing method must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_layer_a_claim_without_device_traffic_receipt() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        [0u8; 32], // zero device_traffic_receipt reference
        baseline.inner_timing_method_wire_name,
        baseline.court_authority_hash_anchors.clone(),
    );
    let errors = verify_layer_a_traffic_receipt(&mutated);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::LayerAClaimWithoutDeviceTrafficReceipt
        )),
        "zero device_traffic_receipt_hash_v1 must surface: {errors:?}"
    );
}

#[test]
fn s_perf_2_rejects_pipeline_that_mutates_court_authority_hashes() {
    let p = build_test_pipeline(false, false, false, false, true);
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::PipelineThatMutatesCourtAuthorityHashes
        )),
        "mutates_court_authority_hashes=true must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Inner-receipt timing methods that ADMIT
// ---------------------------------------------------------------

#[test]
fn cuda_event_inner_timing_method_admits() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    // The baseline already uses CudaEvent; just confirm
    // verifier returns no CudaTimingMethodNotAllowedBySPerf1.
    let errors = verify_layer_a_traffic_receipt(&baseline);
    assert!(!errors.iter().any(|e| matches!(
        e.kind,
        SPerf2VerifyErrorKind::CudaTimingMethodNotAllowedBySPerf1 { .. }
    )));
}

#[test]
fn cuda_stream_sync_inner_timing_method_admits() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        TimingMethod::CudaStreamSync.as_str(),
        baseline.court_authority_hash_anchors.clone(),
    );
    let errors = verify_layer_a_traffic_receipt(&mutated);
    assert!(!errors.iter().any(|e| matches!(
        e.kind,
        SPerf2VerifyErrorKind::CudaTimingMethodNotAllowedBySPerf1 { .. }
    )));
}

#[test]
fn host_instant_only_inner_timing_method_rejected() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        TimingMethod::HostInstantOnly.as_str(),
        baseline.court_authority_hash_anchors.clone(),
    );
    let errors = verify_layer_a_traffic_receipt(&mutated);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::CudaTimingMethodNotAllowedBySPerf1 { .. }
        )),
        "HostInstantOnly should be rejected for Layer-A"
    );
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn empty_pipeline_id_surfaces_structural_defect() {
    let baseline = seed_baseline_layer_a_pipeline();
    let p = build_layer_a_resident_pipeline(
        "",
        baseline.stage_names.clone(),
        baseline.residency_declarations.clone(),
        false,
        false,
        false,
        false,
        false,
    );
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, SPerf2VerifyErrorKind::PipelineIdEmpty)));
}

#[test]
fn empty_stage_names_surfaces_structural_defect() {
    let baseline = seed_baseline_layer_a_pipeline();
    let p = build_layer_a_resident_pipeline(
        baseline.pipeline_id,
        Vec::new(),
        baseline.residency_declarations.clone(),
        false,
        false,
        false,
        false,
        false,
    );
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, SPerf2VerifyErrorKind::StageNamesEmpty)));
}

#[test]
fn duplicate_densor_kind_surfaces_structural_defect() {
    let mut decls = seed_baseline_layer_a_pipeline()
        .residency_declarations
        .clone();
    decls.push(LayerADensorResidencyDeclaration {
        densor_kind: LayerADensorKind::Witness,
        residency_class: DeviceResidencyClass::DeviceResidentOnly,
        expected_max_d2h_bytes_per_catalog: 0,
    });
    let p = build_layer_a_resident_pipeline(
        "test_duplicate",
        LAYER_A_CANONICAL_STAGE_NAMES.to_vec(),
        decls,
        false,
        false,
        false,
        false,
        false,
    );
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::DuplicateDensorKindInPipeline { .. }
        )),
        "duplicate densor kind must surface: {errors:?}"
    );
}

#[test]
fn host_materialized_densor_class_surfaces_structural_defect() {
    let mut decls = seed_baseline_layer_a_pipeline()
        .residency_declarations
        .clone();
    decls[0].residency_class = DeviceResidencyClass::HostMaterialized;
    let p = build_layer_a_resident_pipeline(
        "test_host_materialized",
        LAYER_A_CANONICAL_STAGE_NAMES.to_vec(),
        decls,
        false,
        false,
        false,
        false,
        false,
    );
    let errors = verify_layer_a_resident_pipeline(&p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::HostMaterializedDensorInLayerAPipeline { .. }
        )),
        "HostMaterialized class in Layer-A must surface: {errors:?}"
    );
}

#[test]
fn residency_receipt_pipeline_hash_mismatch_surfaces() {
    let p = seed_baseline_layer_a_pipeline();
    let bogus_hash = [0xFFu8; 32];
    let r = build_layer_a_device_residency_receipt(
        bogus_hash,
        vec![(LayerADensorKind::Evidence, 0)],
        vec![(LayerADensorKind::Evidence, 0)],
    );
    let errors = verify_layer_a_device_residency_receipt(&r, &p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::ResidencyReceiptPipelineHashMismatch { .. }
        )),
        "mismatched pipeline hash must surface: {errors:?}"
    );
}

#[test]
fn d2h_exceeds_declared_cap_surfaces_structural_defect() {
    let p = seed_baseline_layer_a_pipeline();
    // Candidate cap is 2048; push 5000 bytes through.
    let per_h2d = vec![(LayerADensorKind::Candidate, 0u64)];
    let per_d2h = vec![(LayerADensorKind::Candidate, 5_000u64)];
    let r = build_layer_a_device_residency_receipt(
        p.layer_a_resident_pipeline_hash_v1,
        per_h2d,
        per_d2h,
    );
    let errors = verify_layer_a_device_residency_receipt(&r, &p);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::D2hBytesExceedDeclaredCap {
                densor_kind_wire_name: "Candidate",
                ..
            }
        )),
        "Candidate D2H over cap must surface: {errors:?}"
    );
}

#[test]
fn court_authority_anchor_list_missing_corpus_hash_v1_surfaces() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        baseline.inner_timing_method_wire_name,
        Vec::new(), // missing corpus_hash_v1
    );
    let errors = verify_layer_a_traffic_receipt(&mutated);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf2VerifyErrorKind::CourtAuthorityAnchorListMissingCorpusHashV1
        )),
        "missing corpus_hash_v1 anchor must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Sensitivity
// ---------------------------------------------------------------

#[test]
fn pipeline_hash_changes_when_pipeline_id_changes() {
    let baseline = seed_baseline_layer_a_pipeline();
    let mutated = build_layer_a_resident_pipeline(
        "different_pipeline_id",
        baseline.stage_names.clone(),
        baseline.residency_declarations.clone(),
        false,
        false,
        false,
        false,
        false,
    );
    assert_ne!(
        baseline.layer_a_resident_pipeline_hash_v1,
        mutated.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn pipeline_hash_changes_when_stage_names_change() {
    let baseline = seed_baseline_layer_a_pipeline();
    let mut stages = baseline.stage_names.clone();
    stages.push("ExtraStage");
    let mutated = build_layer_a_resident_pipeline(
        baseline.pipeline_id,
        stages,
        baseline.residency_declarations.clone(),
        false,
        false,
        false,
        false,
        false,
    );
    assert_ne!(
        baseline.layer_a_resident_pipeline_hash_v1,
        mutated.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn pipeline_hash_changes_when_residency_class_changes() {
    let baseline = seed_baseline_layer_a_pipeline();
    let mut decls = baseline.residency_declarations.clone();
    decls[0].residency_class = DeviceResidencyClass::DeviceResidentWithCompactD2H;
    decls[0].expected_max_d2h_bytes_per_catalog = 64;
    let mutated = build_layer_a_resident_pipeline(
        baseline.pipeline_id,
        baseline.stage_names.clone(),
        decls,
        false,
        false,
        false,
        false,
        false,
    );
    assert_ne!(
        baseline.layer_a_resident_pipeline_hash_v1,
        mutated.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn pipeline_hash_changes_when_forbidden_flag_changes() {
    let baseline = seed_baseline_layer_a_pipeline();
    let mutated = build_layer_a_resident_pipeline(
        baseline.pipeline_id,
        baseline.stage_names.clone(),
        baseline.residency_declarations.clone(),
        true, // casefile_materialization_present
        false,
        false,
        false,
        false,
    );
    assert_ne!(
        baseline.layer_a_resident_pipeline_hash_v1,
        mutated.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn residency_receipt_hash_changes_when_h2d_bytes_change() {
    let baseline = seed_baseline_layer_a_residency_receipt();
    let pipeline_hash = baseline.pipeline_hash;
    let mut per_h2d = baseline.per_densor_h2d_bytes.clone();
    per_h2d[0].1 = 1024;
    let mutated = build_layer_a_device_residency_receipt(
        pipeline_hash,
        per_h2d,
        baseline.per_densor_d2h_bytes.clone(),
    );
    assert_ne!(
        baseline.layer_a_device_residency_receipt_hash_v1,
        mutated.layer_a_device_residency_receipt_hash_v1
    );
}

#[test]
fn residency_receipt_hash_changes_when_d2h_bytes_change() {
    let baseline = seed_baseline_layer_a_residency_receipt();
    let pipeline_hash = baseline.pipeline_hash;
    let mut per_d2h = baseline.per_densor_d2h_bytes.clone();
    // Boost candidate summary D2H (within cap).
    for entry in &mut per_d2h {
        if matches!(entry.0, LayerADensorKind::Candidate) {
            entry.1 = 512;
        }
    }
    let mutated = build_layer_a_device_residency_receipt(
        pipeline_hash,
        baseline.per_densor_h2d_bytes.clone(),
        per_d2h,
    );
    assert_ne!(
        baseline.layer_a_device_residency_receipt_hash_v1,
        mutated.layer_a_device_residency_receipt_hash_v1
    );
}

#[test]
fn traffic_receipt_hash_changes_when_anchor_list_changes() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mut anchors = baseline.court_authority_hash_anchors.clone();
    anchors.push([0xABu8; 32]);
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        baseline.inner_timing_method_wire_name,
        anchors,
    );
    assert_ne!(
        baseline.layer_a_traffic_receipt_hash_v1,
        mutated.layer_a_traffic_receipt_hash_v1
    );
}

#[test]
fn traffic_receipt_hash_changes_when_inner_timing_method_changes() {
    let baseline = seed_baseline_layer_a_traffic_receipt();
    let mutated = build_layer_a_traffic_receipt(
        baseline.pipeline.clone(),
        baseline.residency_receipt.clone(),
        baseline.device_traffic_receipt_hash_v1,
        TimingMethod::CudaStreamSync.as_str(),
        baseline.court_authority_hash_anchors.clone(),
    );
    assert_ne!(
        baseline.layer_a_traffic_receipt_hash_v1,
        mutated.layer_a_traffic_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Rendering smoke tests
// ---------------------------------------------------------------

#[test]
fn pipeline_text_contains_pinned_header_lines() {
    let s = render_layer_a_resident_pipeline_text(&seed_baseline_layer_a_pipeline());
    assert!(s.contains("S-PERF.2 LayerAResidentPipelineV1"));
    assert!(s.contains("Identity"));
    assert!(s.contains("Stages"));
    assert!(s.contains("Per-densor residency declarations"));
    assert!(s.contains("Forbidden host activities (all must be false)"));
    assert!(s.contains("layer_a_resident_pipeline_hash_v1"));
}

#[test]
fn pipeline_json_contains_pinned_schema_id() {
    let s = render_layer_a_resident_pipeline_json(&seed_baseline_layer_a_pipeline());
    assert!(s.contains(LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1));
    assert!(s.contains("layer_a_resident_pipeline_hash_v1"));
    assert!(s.contains("stage_names"));
    assert!(s.contains("residency_declarations"));
}

#[test]
fn residency_receipt_text_contains_pinned_header_lines() {
    let s =
        render_layer_a_device_residency_receipt_text(&seed_baseline_layer_a_residency_receipt());
    assert!(s.contains("S-PERF.2 LayerADeviceResidencyReceiptV1"));
    assert!(s.contains("Per-densor H2D bytes"));
    assert!(s.contains("Per-densor D2H bytes"));
    assert!(s.contains("layer_a_device_residency_receipt_hash_v1"));
}

#[test]
fn residency_receipt_json_contains_pinned_schema_id() {
    let s =
        render_layer_a_device_residency_receipt_json(&seed_baseline_layer_a_residency_receipt());
    assert!(s.contains(LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1));
    assert!(s.contains("layer_a_device_residency_receipt_hash_v1"));
    assert!(s.contains("per_densor_h2d_bytes"));
    assert!(s.contains("per_densor_d2h_bytes"));
}

#[test]
fn traffic_receipt_text_contains_pinned_header_lines() {
    let s = render_layer_a_traffic_receipt_text(&seed_baseline_layer_a_traffic_receipt());
    assert!(s.contains("S-PERF.2 LayerATrafficReceiptV1"));
    assert!(s.contains("Pipeline + residency"));
    assert!(s.contains("S-PERF.1 reference"));
    assert!(s.contains("Court-authority anchors"));
    assert!(s.contains("layer_a_traffic_receipt_hash_v1"));
    assert!(s.contains("CudaEvent"));
}

#[test]
fn traffic_receipt_json_contains_pinned_schema_id() {
    let s = render_layer_a_traffic_receipt_json(&seed_baseline_layer_a_traffic_receipt());
    assert!(s.contains(LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1));
    assert!(s.contains("layer_a_traffic_receipt_hash_v1"));
    assert!(s.contains("inner_timing_method_wire_name"));
    assert!(s.contains("court_authority_hash_anchors"));
}

// ---------------------------------------------------------------
// Sanity / non-zero hash guards
// ---------------------------------------------------------------

#[test]
fn baseline_pipeline_has_non_zero_pipeline_hash() {
    let p = seed_baseline_layer_a_pipeline();
    assert_ne!(p.layer_a_resident_pipeline_hash_v1, [0u8; 32]);
}

#[test]
fn baseline_residency_receipt_has_non_zero_receipt_hash() {
    let r = seed_baseline_layer_a_residency_receipt();
    assert_ne!(r.layer_a_device_residency_receipt_hash_v1, [0u8; 32]);
}

#[test]
fn baseline_traffic_receipt_has_non_zero_traffic_hash() {
    let r = seed_baseline_layer_a_traffic_receipt();
    assert_ne!(r.layer_a_traffic_receipt_hash_v1, [0u8; 32]);
}

#[test]
fn s_perf_1_device_identity_helper_re_export_is_non_zero() {
    let h = panel_locked_device_identity_hash_helper("RTX 4080 SUPER", 89);
    assert_ne!(h, [0u8; 32]);
}

// ---------------------------------------------------------------
// Densor kind wire-name stability
// ---------------------------------------------------------------

#[test]
fn densor_kind_wire_names_are_stable() {
    assert_eq!(LayerADensorKind::Evidence.as_str(), "Evidence");
    assert_eq!(LayerADensorKind::Witness.as_str(), "Witness");
    assert_eq!(LayerADensorKind::Fusion.as_str(), "Fusion");
    assert_eq!(LayerADensorKind::Candidate.as_str(), "Candidate");
    assert_eq!(LayerADensorKind::StageDigest.as_str(), "StageDigest");
}

#[test]
fn residency_class_wire_names_are_stable() {
    assert_eq!(
        DeviceResidencyClass::DeviceResidentOnly.as_str(),
        "DeviceResidentOnly"
    );
    assert_eq!(
        DeviceResidencyClass::DeviceResidentWithCompactD2H.as_str(),
        "DeviceResidentWithCompactD2H"
    );
    assert_eq!(
        DeviceResidencyClass::HostMaterialized.as_str(),
        "HostMaterialized"
    );
}

// ---------------------------------------------------------------
// Cross-test: baseline Layer-A traffic receipt's pipeline hash
// matches the baseline pipeline's hash (composition invariant)
// ---------------------------------------------------------------

#[test]
fn baseline_traffic_receipt_pipeline_hash_matches_baseline_pipeline_hash() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let pipeline = seed_baseline_layer_a_pipeline();
    assert_eq!(
        traffic.pipeline.layer_a_resident_pipeline_hash_v1,
        pipeline.layer_a_resident_pipeline_hash_v1
    );
}

#[test]
fn baseline_traffic_receipt_residency_receipt_hash_matches_baseline_residency_receipt() {
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let residency = seed_baseline_layer_a_residency_receipt();
    assert_eq!(
        traffic
            .residency_receipt
            .layer_a_device_residency_receipt_hash_v1,
        residency.layer_a_device_residency_receipt_hash_v1
    );
}
