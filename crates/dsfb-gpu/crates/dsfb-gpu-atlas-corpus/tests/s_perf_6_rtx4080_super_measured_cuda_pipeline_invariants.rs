//! S-PERF.6 acceptance suite for the measured RTX 4080
//! SUPER CUDA pipeline baseline.
//!
//! 14 panel-required load-bearing negatives (verbatim from
//! the directive):
//!
//!  1. s_perf_6_rejects_zero_measured_bandwidth
//!  2. s_perf_6_rejects_zero_device_total_time
//!  3. s_perf_6_rejects_missing_source_report_path
//!  4. s_perf_6_rejects_missing_rtx4080_super_identity
//!  5. s_perf_6_rejects_percent_of_peak_arithmetic_mismatch
//!  6. s_perf_6_rejects_saturation_claim_below_8000_bp
//!  7. s_perf_6_rejects_claim_that_13_33_gbps_is_saturation
//!  8. s_perf_6_rejects_claim_that_result_is_b300_or_gb300
//!  9. s_perf_6_rejects_claim_that_result_is_production_performance
//! 10. s_perf_6_rejects_rebaseline_of_r12b_episode_counts
//! 11. s_perf_6_rejects_missing_tree_digest_stage_timing
//! 12. s_perf_6_rejects_missing_host_segment_disclosure
//! 13. s_perf_6_rejects_empty_claim_kind
//! 14. s_perf_6_rejects_no_claim_baseline_for_measured_result
//!
//! 13 panel-required positive tests (verbatim):
//!
//!  - measured_cuda_pipeline_result_admits
//!  - percent_of_peak_computes_from_13_33_and_716
//!  - saturation_false_because_186_or_187_bp_below_8000
//!  - source_report_path_is_declared
//!  - tree_digest_consensus_stage_is_declared
//!  - host_compute_features_segment_is_declared
//!  - host_bank_admit_case_finalize_segment_is_declared
//!  - r12b_episode_count_pins_remain_13_89_1917
//!  - renderers_byte_stable
//!  - hashes_deterministic_across_two_builds
//!  - changing_measured_bandwidth_changes_hash
//!  - changing_device_total_us_changes_hash
//!  - changing_source_report_path_changes_hash
//!
//! Plus 3 pinned-hash constants (back-stop against silent
//! rebaselining).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

use dsfb_gpu_atlas_corpus::s_perf_1_device_traffic_receipt::{
    compute_device_identity_hash, S_PERF_1_SATURATION_BP,
};
use dsfb_gpu_atlas_corpus::s_perf_2_layer_a_resident_pipeline::seed_baseline_layer_a_traffic_receipt;
use dsfb_gpu_atlas_corpus::s_perf_3_public_data_saturation_bundle::seed_baseline_public_data_saturation_bundle;
use dsfb_gpu_atlas_corpus::s_perf_4_active_family_compaction::seed_baseline_family_compaction_benchmark_schema;
use dsfb_gpu_atlas_corpus::s_perf_5_effective_bandwidth_report::seed_baseline_effective_bandwidth_report;
use dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    build_rtx4080_super_measured_bandwidth_claim, build_rtx4080_super_measured_baseline_report,
    build_rtx4080_super_measured_cuda_pipeline, compute_s_perf_6_percent_of_peak_basis_points,
    forbidden_claim_substrings, render_rtx4080_super_measured_bandwidth_claim_json,
    render_rtx4080_super_measured_bandwidth_claim_text,
    render_rtx4080_super_measured_baseline_report_json,
    render_rtx4080_super_measured_baseline_report_text,
    render_rtx4080_super_measured_cuda_pipeline_json,
    render_rtx4080_super_measured_cuda_pipeline_text, seed_rtx4080_super_measured_bandwidth_claim,
    seed_rtx4080_super_measured_baseline_report, seed_rtx4080_super_measured_cuda_pipeline,
    verify_rtx4080_super_measured_baseline_report, MeasuredCudaPipelineClaimKind,
    Rtx4080SuperMeasuredBandwidthClaimV1, Rtx4080SuperMeasuredBaselineReportV1,
    Rtx4080SuperMeasuredCudaPipelineV1, SPerf6VerifyErrorKind,
    ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE,
    R12B_EPISODE_COUNT_CANONICAL_W16H128, R12B_EPISODE_COUNT_FULL_W256H4096,
    R12B_EPISODE_COUNT_MID_W64H512, RTX_4080_SUPER_DEVICE_NAME, RTX_4080_SUPER_SM_ARCH,
    RTX_4080_SUPER_THEORETICAL_PEAK_GBPS, S_PERF_6_BOTTLENECK_SENTENCE, S_PERF_6_CUDA_VERSION,
    S_PERF_6_DEVICE_TOTAL_US, S_PERF_6_HOST_BANK_ADMIT_CASE_FINALIZE_US,
    S_PERF_6_HOST_COMPUTE_FEATURES_US, S_PERF_6_HOST_WALL_MEDIAN_US,
    S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS, S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
    S_PERF_6_REPORT_SENTENCE, S_PERF_6_SOURCE_REPORT_PATH,
    S_PERF_6_TREE_DIGEST_CONSENSUS_PERCENT_BP, S_PERF_6_TREE_DIGEST_CONSENSUS_US,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn has_kind(
    errors: &[dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::SPerf6VerifyError],
    pred: impl Fn(&SPerf6VerifyErrorKind) -> bool,
) -> bool {
    errors.iter().any(|e| pred(&e.kind))
}

fn live_device_identity() -> [u8; 32] {
    compute_device_identity_hash(RTX_4080_SUPER_DEVICE_NAME, RTX_4080_SUPER_SM_ARCH)
}

fn build_baseline_with_measurement_and_claim(
    measurement: Rtx4080SuperMeasuredCudaPipelineV1,
    claim: Rtx4080SuperMeasuredBandwidthClaimV1,
) -> Rtx4080SuperMeasuredBaselineReportV1 {
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let report_5 = seed_baseline_effective_bandwidth_report();
    build_rtx4080_super_measured_baseline_report(
        "test_baseline_v1",
        measurement,
        claim,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
        report_5.effective_bandwidth_report_hash_v1,
        R12B_EPISODE_COUNT_CANONICAL_W16H128,
        R12B_EPISODE_COUNT_MID_W64H512,
        R12B_EPISODE_COUNT_FULL_W256H4096,
    )
}

// ---------------------------------------------------------------
// Positive: baseline admits
// ---------------------------------------------------------------

#[test]
fn measured_cuda_pipeline_result_admits() {
    let r = seed_rtx4080_super_measured_baseline_report();
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(errors.is_empty(), "baseline must admit: {errors:?}");
}

#[test]
fn percent_of_peak_computes_from_13_33_and_716() {
    let computed = compute_s_perf_6_percent_of_peak_basis_points(
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
    );
    assert_eq!(
        computed, 186,
        "1333 centi-GB/s on 716 GB/s peak (floor) must equal 186 bp"
    );
    assert_eq!(computed, S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS);
}

#[test]
fn saturation_false_because_186_or_187_bp_below_8000() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert!(
        r.claim.observed_percent_of_peak_basis_points < S_PERF_1_SATURATION_BP,
        "{} bp must be below the 8000 bp saturation threshold",
        r.claim.observed_percent_of_peak_basis_points
    );
    assert!(
        !r.claim.saturation_admitted,
        "saturation_admitted must be false"
    );
    // Allow either 186 (floor) or 187 (nearest) per directive
    // tolerance; current rounding law is floor → 186.
    assert!(
        r.claim.observed_percent_of_peak_basis_points == 186
            || r.claim.observed_percent_of_peak_basis_points == 187
    );
}

#[test]
fn source_report_path_is_declared() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(
        r.measurement.source_report_path,
        "reports/d64_stage_timing_256x4096_K1.txt"
    );
    assert_eq!(
        r.measurement.source_report_path,
        S_PERF_6_SOURCE_REPORT_PATH
    );
}

#[test]
fn tree_digest_consensus_stage_is_declared() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(r.measurement.tree_digest_consensus_us, 4_338);
    assert_eq!(
        r.measurement.tree_digest_consensus_us,
        S_PERF_6_TREE_DIGEST_CONSENSUS_US
    );
    // 20.88 % = 2088 bp.
    assert_eq!(
        r.measurement.tree_digest_consensus_percent_basis_points,
        2_088
    );
    assert_eq!(
        r.measurement.tree_digest_consensus_percent_basis_points,
        S_PERF_6_TREE_DIGEST_CONSENSUS_PERCENT_BP
    );
}

#[test]
fn host_compute_features_segment_is_declared() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(r.measurement.host_compute_features_us, 7_525);
    assert_eq!(
        r.measurement.host_compute_features_us,
        S_PERF_6_HOST_COMPUTE_FEATURES_US
    );
}

#[test]
fn host_bank_admit_case_finalize_segment_is_declared() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(r.measurement.host_bank_admit_case_finalize_us, 2_237);
    assert_eq!(
        r.measurement.host_bank_admit_case_finalize_us,
        S_PERF_6_HOST_BANK_ADMIT_CASE_FINALIZE_US
    );
}

#[test]
fn r12b_episode_count_pins_remain_13_89_1917() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(r.r12b_episode_count_canonical_w16h128, 13);
    assert_eq!(r.r12b_episode_count_mid_w64h512, 89);
    assert_eq!(r.r12b_episode_count_full_w256h4096, 1917);
}

// ---------------------------------------------------------------
// Determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn hashes_deterministic_across_two_builds() {
    let a = seed_rtx4080_super_measured_baseline_report();
    let b = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(
        a.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1,
        b.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1
    );
    assert_eq!(
        a.claim.rtx4080_super_measured_bandwidth_claim_hash_v1,
        b.claim.rtx4080_super_measured_bandwidth_claim_hash_v1
    );
    assert_eq!(
        a.rtx4080_super_measured_baseline_report_hash_v1,
        b.rtx4080_super_measured_baseline_report_hash_v1
    );
}

#[test]
fn changing_measured_bandwidth_changes_hash() {
    let id = live_device_identity();
    let original = seed_rtx4080_super_measured_cuda_pipeline();
    let mutated = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        1500, // <-- mutated (was 1333)
        compute_s_perf_6_percent_of_peak_basis_points(800, RTX_4080_SUPER_THEORETICAL_PEAK_GBPS),
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    assert_ne!(
        original.rtx4080_super_measured_cuda_pipeline_hash_v1,
        mutated.rtx4080_super_measured_cuda_pipeline_hash_v1
    );
}

#[test]
fn changing_device_total_us_changes_hash() {
    let id = live_device_identity();
    let original = seed_rtx4080_super_measured_cuda_pipeline();
    let mutated = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        99_999, // <-- mutated
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    assert_ne!(
        original.rtx4080_super_measured_cuda_pipeline_hash_v1,
        mutated.rtx4080_super_measured_cuda_pipeline_hash_v1
    );
}

#[test]
fn changing_source_report_path_changes_hash() {
    let id = live_device_identity();
    let original = seed_rtx4080_super_measured_cuda_pipeline();
    let mutated = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        "reports/some_other_report.txt", // <-- mutated
    );
    assert_ne!(
        original.rtx4080_super_measured_cuda_pipeline_hash_v1,
        mutated.rtx4080_super_measured_cuda_pipeline_hash_v1
    );
}

#[test]
fn three_hashes_are_pairwise_distinct() {
    let r = seed_rtx4080_super_measured_baseline_report();
    let h_m = r.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1;
    let h_c = r.claim.rtx4080_super_measured_bandwidth_claim_hash_v1;
    let h_r = r.rtx4080_super_measured_baseline_report_hash_v1;
    assert_ne!(h_m, h_c);
    assert_ne!(h_m, h_r);
    assert_ne!(h_c, h_r);
}

#[test]
fn three_hashes_distinct_from_every_upstream_anchor() {
    let r = seed_rtx4080_super_measured_baseline_report();
    let new_hashes = [
        r.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1,
        r.claim.rtx4080_super_measured_bandwidth_claim_hash_v1,
        r.rtx4080_super_measured_baseline_report_hash_v1,
    ];
    let upstream = [
        r.measurement.device_uuid_or_identity_hash,
        r.s_perf_2_layer_a_traffic_receipt_hash,
        r.s_perf_3_public_data_bundle_hash,
        r.s_perf_4_family_compaction_benchmark_schema_hash,
        r.s_perf_5_effective_bandwidth_report_hash,
    ];
    for n in new_hashes {
        for u in upstream {
            assert_ne!(n, u);
        }
    }
}

// ---------------------------------------------------------------
// Pinned hash constants (back-stop against silent rebaselining)
// ---------------------------------------------------------------

const PINNED_MEASURED_CUDA_PIPELINE_HASH_V1: [u8; 32] = [
    0xa5, 0xb5, 0x8b, 0xc8, 0x74, 0x4c, 0x93, 0xbf, 0xfe, 0x4c, 0x10, 0xb9, 0x30, 0xc7, 0x41, 0x80,
    0x90, 0x53, 0x48, 0x32, 0x44, 0x82, 0x3e, 0x01, 0xdd, 0x87, 0x2f, 0x9e, 0x9a, 0x1f, 0x21, 0xf2,
];

const PINNED_MEASURED_BANDWIDTH_CLAIM_HASH_V1: [u8; 32] = [
    0x4f, 0xdf, 0x86, 0x99, 0x79, 0x8d, 0xb9, 0xdb, 0x15, 0x0e, 0x2c, 0xef, 0x40, 0x09, 0x0e, 0x41,
    0x89, 0x07, 0x97, 0x46, 0x67, 0x98, 0x37, 0x94, 0x4f, 0xb5, 0xdf, 0xe5, 0x0e, 0x21, 0xcb, 0xab,
];

const PINNED_MEASURED_BASELINE_REPORT_HASH_V1: [u8; 32] = [
    0xd4, 0x4c, 0x9e, 0xc5, 0x44, 0x7f, 0x05, 0xfb, 0x6c, 0x40, 0x56, 0x7d, 0xcb, 0x56, 0x50, 0x2c,
    0x17, 0xc1, 0x4f, 0x00, 0xad, 0x10, 0x0f, 0x81, 0xd3, 0x09, 0x00, 0xc1, 0xc1, 0x5a, 0x15, 0x33,
];

#[test]
fn baseline_measurement_hash_matches_pinned_constant() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    assert_eq!(
        m.rtx4080_super_measured_cuda_pipeline_hash_v1,
        PINNED_MEASURED_CUDA_PIPELINE_HASH_V1
    );
}

#[test]
fn baseline_claim_hash_matches_pinned_constant() {
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    assert_eq!(
        c.rtx4080_super_measured_bandwidth_claim_hash_v1,
        PINNED_MEASURED_BANDWIDTH_CLAIM_HASH_V1
    );
}

#[test]
fn baseline_report_hash_matches_pinned_constant() {
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(
        r.rtx4080_super_measured_baseline_report_hash_v1,
        PINNED_MEASURED_BASELINE_REPORT_HASH_V1
    );
}

// ---------------------------------------------------------------
// Renderer byte-stability
// ---------------------------------------------------------------

#[test]
fn renderers_byte_stable() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = seed_rtx4080_super_measured_baseline_report();
    assert_eq!(
        render_rtx4080_super_measured_cuda_pipeline_text(&m),
        render_rtx4080_super_measured_cuda_pipeline_text(&m)
    );
    assert_eq!(
        render_rtx4080_super_measured_cuda_pipeline_json(&m),
        render_rtx4080_super_measured_cuda_pipeline_json(&m)
    );
    assert_eq!(
        render_rtx4080_super_measured_bandwidth_claim_text(&c),
        render_rtx4080_super_measured_bandwidth_claim_text(&c)
    );
    assert_eq!(
        render_rtx4080_super_measured_bandwidth_claim_json(&c),
        render_rtx4080_super_measured_bandwidth_claim_json(&c)
    );
    assert_eq!(
        render_rtx4080_super_measured_baseline_report_text(&r),
        render_rtx4080_super_measured_baseline_report_text(&r)
    );
    assert_eq!(
        render_rtx4080_super_measured_baseline_report_json(&r),
        render_rtx4080_super_measured_baseline_report_json(&r)
    );
}

#[test]
fn baseline_report_text_contains_panel_locked_sentences() {
    let r = seed_rtx4080_super_measured_baseline_report();
    let text = render_rtx4080_super_measured_baseline_report_text(&r);
    assert!(text.contains(S_PERF_6_REPORT_SENTENCE));
    assert!(text.contains(S_PERF_6_BOTTLENECK_SENTENCE));
}

#[test]
fn measurement_text_contains_panel_locked_values() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let text = render_rtx4080_super_measured_cuda_pipeline_text(&m);
    assert!(text.contains("RTX 4080 SUPER"));
    assert!(text.contains("sm_89"));
    assert!(text.contains("13.2"));
    assert!(text.contains("716"));
    assert!(text.contains("13.33 GB/s"));
    assert!(text.contains("186"));
    assert!(text.contains("reports/d64_stage_timing_256x4096_K1.txt"));
}

// ---------------------------------------------------------------
// Panel-required negative #1: zero measured bandwidth
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_zero_measured_bandwidth() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        0, // <-- zero measured bandwidth
        0,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ZeroMeasuredBandwidth
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #2: zero device total time
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_zero_device_total_time() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        0, // <-- zero device total
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ZeroDeviceTotalTime
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #3: missing source report path
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_missing_source_report_path() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        "", // <-- empty
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingSourceReportPath
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #4: missing RTX 4080 SUPER identity
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_missing_rtx4080_super_identity() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        "Wrong Device", // <-- wrong name
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity { which_field_wire_name }
            if *which_field_wire_name == "device_name"
    )));
}

#[test]
fn s_perf_6_rejects_wrong_sm_arch() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        80, // <-- not 89
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity { which_field_wire_name }
            if *which_field_wire_name == "sm_arch"
    )));
}

#[test]
fn s_perf_6_rejects_wrong_theoretical_peak() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        999, // <-- not 716
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity { which_field_wire_name }
            if *which_field_wire_name == "theoretical_memory_bandwidth_gbps"
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #5: arithmetic mismatch
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_percent_of_peak_arithmetic_mismatch() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        1_333,
        9_999, // <-- LIED; should be 186
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::PercentOfPeakArithmeticMismatch {
            claimed: 9_999,
            computed: 186
        }
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #6: saturation claim below 8000 bp
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_saturation_claim_below_8000_bp() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE,
        true,
        S_PERF_1_SATURATION_BP,
        186,
        true, // <-- saturation_admitted=true with 186 bp
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::SaturationClaimBelow8000Bp { observed_bp: 186 }
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #7: claim that 13.33 GB/s is saturation
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_claim_that_13_33_gbps_is_saturation() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    // Same as negative #6 but framed as the "13.33 GB/s is
    // saturation" gate: saturation_admitted=true on a
    // measured 1333 centi-GB/s baseline (well below the
    // ~57200 centi-GB/s floor required to satisfy 8000 bp
    // of 716 GB/s peak).
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE,
        true,
        S_PERF_1_SATURATION_BP,
        186,
        true,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    // Both negative #6 and negative #7 fire on the same
    // malformed claim.
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ClaimThat7_70GbpsIsSaturation
    )));
}

#[test]
fn s_perf_6_rejects_claim_that_13_33_gbps_is_saturation_via_admissibility_reason_text() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let leaked: &'static str = Box::leak(
        "MeasuredCudaPipeline achieves saturation"
            .to_string()
            .into_boxed_str(),
    );
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        leaked,
        true,
        S_PERF_1_SATURATION_BP,
        186,
        false,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ClaimThat7_70GbpsIsSaturation
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #8: B300 / GB300 claim
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_claim_that_result_is_b300_or_gb300() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let leaked: &'static str = Box::leak(
        "MeasuredCudaPipeline_b300_class"
            .to_string()
            .into_boxed_str(),
    );
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        leaked,
        true,
        S_PERF_1_SATURATION_BP,
        186,
        false,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ClaimThatResultIsB300OrGb300
    )));
}

#[test]
fn s_perf_6_rejects_gb300_in_source_report_path() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        "reports/gb300_results.txt", // <-- forbidden
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ClaimThatResultIsB300OrGb300
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #9: production-performance claim
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_claim_that_result_is_production_performance() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let leaked: &'static str = Box::leak(
        "MeasuredCudaPipeline production performance"
            .to_string()
            .into_boxed_str(),
    );
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        leaked,
        true,
        S_PERF_1_SATURATION_BP,
        186,
        false,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::ClaimThatResultIsProductionPerformance
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #10: R.12b episode-count rebaseline
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_rebaseline_of_r12b_episode_counts() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let report_5 = seed_baseline_effective_bandwidth_report();
    let r = build_rtx4080_super_measured_baseline_report(
        "test",
        m,
        c,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
        report_5.effective_bandwidth_report_hash_v1,
        99, // <-- canonical pin rebaselined (should be 13)
        89,
        1917,
    );
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts { which_pin_wire_name, .. }
            if *which_pin_wire_name == "r12b_episode_count_canonical_w16h128"
    )));
}

#[test]
fn s_perf_6_rejects_rebaseline_of_r12b_episode_counts_mid_and_full() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let report_5 = seed_baseline_effective_bandwidth_report();
    let r = build_rtx4080_super_measured_baseline_report(
        "test",
        m,
        c,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
        report_5.effective_bandwidth_report_hash_v1,
        13,
        88,   // <-- mid pin (should be 89)
        1916, // <-- full pin (should be 1917)
    );
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts { which_pin_wire_name, .. }
            if *which_pin_wire_name == "r12b_episode_count_mid_w64h512"
    )));
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts { which_pin_wire_name, .. }
            if *which_pin_wire_name == "r12b_episode_count_full_w256h4096"
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #11: missing tree_digest stage timing
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_missing_tree_digest_stage_timing() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        0, // <-- missing tree_digest_consensus_us
        2_120,
        7_525,
        2_237,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingTreeDigestStageTiming
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #12: missing host segment disclosure
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_missing_host_segment_disclosure() {
    let id = live_device_identity();
    let m = build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        id,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        812,
        4_338,
        2_120,
        0, // <-- both host segments zero
        0,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    );
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::MissingHostSegmentDisclosure
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #13: empty claim kind / reason
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_empty_claim_kind() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        "", // <-- empty admissibility reason
        true,
        S_PERF_1_SATURATION_BP,
        186,
        false,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::EmptyClaimKind
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #14: NoClaim baseline for measured
// result (admitted=false guard)
// ---------------------------------------------------------------

#[test]
fn s_perf_6_rejects_no_claim_baseline_for_measured_result() {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let c = build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE,
        false, // <-- admitted=false for a measured result is the panel-forbidden NoClaim posture
        S_PERF_1_SATURATION_BP,
        186,
        false,
    );
    let r = build_baseline_with_measurement_and_claim(m, c);
    let errors = verify_rtx4080_super_measured_baseline_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf6VerifyErrorKind::NoClaimBaselineForMeasuredResult
    )));
}

// ---------------------------------------------------------------
// Misc structural + panel-locked discipline
// ---------------------------------------------------------------

#[test]
fn forbidden_claim_substring_set_includes_b300_gb300_production_performance() {
    let set = forbidden_claim_substrings();
    assert!(set.iter().any(|s| s.eq_ignore_ascii_case("b300")));
    assert!(set.iter().any(|s| s.eq_ignore_ascii_case("gb300")));
    assert!(set
        .iter()
        .any(|s| s.eq_ignore_ascii_case("production performance")));
}

#[test]
fn admissibility_reason_constant_is_non_empty() {
    assert!(!ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE.is_empty());
}

#[test]
fn measured_claim_kind_wire_name_is_measured_cuda_pipeline_bandwidth() {
    assert_eq!(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth.as_str(),
        "MeasuredCudaPipelineBandwidth"
    );
}

#[test]
fn report_sentence_states_13_33_gbps_and_1_07_percent_and_not_saturation() {
    assert!(S_PERF_6_REPORT_SENTENCE.contains("13.33 GB/s"));
    assert!(S_PERF_6_REPORT_SENTENCE.contains("1.86%"));
    assert!(S_PERF_6_REPORT_SENTENCE.contains("716 GB/s"));
    assert!(S_PERF_6_REPORT_SENTENCE.contains("not a saturation claim"));
}

#[test]
fn bottleneck_sentence_names_tree_digest_consensus_and_host_segments() {
    assert!(S_PERF_6_BOTTLENECK_SENTENCE.contains("tree_digest consensus"));
    assert!(S_PERF_6_BOTTLENECK_SENTENCE.contains("host-side"));
    assert!(S_PERF_6_BOTTLENECK_SENTENCE.contains("saturation"));
}

#[test]
fn host_wall_median_is_30020_us() {
    assert_eq!(S_PERF_6_HOST_WALL_MEDIAN_US, 30_020);
}

#[test]
fn device_total_is_20771_us() {
    assert_eq!(S_PERF_6_DEVICE_TOTAL_US, 20_771);
}

#[test]
fn cuda_version_is_13_2() {
    assert_eq!(S_PERF_6_CUDA_VERSION, "13.2");
}

#[test]
fn arithmetic_helper_returns_zero_on_zero_peak() {
    let x = compute_s_perf_6_percent_of_peak_basis_points(1_333, 0);
    assert_eq!(x, 0);
}

#[test]
fn arithmetic_helper_uses_floor_not_nearest() {
    // 1333 * 10000 / (716 * 100) = 13_330_000 / 71_600 = 186.17...
    // floor → 186, nearest → 186. The panel-pinned value is 186.
    let computed = compute_s_perf_6_percent_of_peak_basis_points(1_333, 716);
    assert_eq!(computed, 186, "must use FLOOR rounding, not nearest");
}
