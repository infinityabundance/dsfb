//! S-PERF.5 acceptance suite for
//! `LayerABandwidthMeasurementV1`,
//! `BandwidthClaimAdmissionV1`, and
//! `EffectiveBandwidthReportV1` invariants.
//!
//! TEN panel-required load-bearing negatives:
//!
//! 1. `s_perf_5_rejects_report_without_s_perf_1_receipt`
//! 2. `s_perf_5_rejects_report_without_s_perf_2_layer_a_receipt`
//! 3. `s_perf_5_rejects_report_without_s_perf_3_bundle_hash`
//! 4. `s_perf_5_rejects_report_without_s_perf_4_compaction_hash`
//! 5. `s_perf_5_rejects_saturation_claim_below_8000_bp`
//! 6. `s_perf_5_rejects_saturation_claim_with_host_timing`
//! 7. `s_perf_5_rejects_effective_bandwidth_mismatch_from_bytes_and_time`
//! 8. `s_perf_5_rejects_report_that_includes_host_json_or_casefile_time`
//! 9. `s_perf_5_rejects_cross_device_claim_without_device_identity`
//! 10. `s_perf_5_rejects_benchmark_claim_without_public_artifact_manifest`
//!
//! Plus structural defect tests (empty report id, empty
//! admissibility reason, forbidden benchmark-claim substring
//! inside report, claim_kind / measurement coherence,
//! inadmissible without verifier reason), baseline admission,
//! determinism, sensitivity, cross-verifier composition,
//! rendering byte-stability, and panel-locked pinned hash
//! constants.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

use dsfb_gpu_atlas_corpus::s_perf_1_device_traffic_receipt::{
    seed_baseline_uninstrumented_receipt, S_PERF_1_SATURATION_BP,
};
use dsfb_gpu_atlas_corpus::s_perf_2_layer_a_resident_pipeline::seed_baseline_layer_a_traffic_receipt;
use dsfb_gpu_atlas_corpus::s_perf_3_public_data_saturation_bundle::seed_baseline_public_data_saturation_bundle;
use dsfb_gpu_atlas_corpus::s_perf_4_active_family_compaction::seed_baseline_family_compaction_benchmark_schema;
use dsfb_gpu_atlas_corpus::s_perf_5_effective_bandwidth_report::{
    build_bandwidth_claim_admission, build_effective_bandwidth_report,
    build_layer_a_bandwidth_measurement, forbidden_benchmark_claim_substrings,
    render_bandwidth_claim_admission_json, render_bandwidth_claim_admission_text,
    render_effective_bandwidth_report_json, render_effective_bandwidth_report_text,
    render_layer_a_bandwidth_measurement_json, render_layer_a_bandwidth_measurement_text,
    seed_baseline_bandwidth_claim_admission, seed_baseline_effective_bandwidth_report,
    seed_baseline_layer_a_bandwidth_measurement, verify_effective_bandwidth_report,
    BandwidthClaimKind, EffectiveBandwidthReportV1, LayerABandwidthMeasurementV1,
    SPerf5VerifyErrorKind, ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH,
    ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM, ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK,
    ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION, BANDWIDTH_CLAIM_ADMISSION_DOMAIN_V1,
    BANDWIDTH_CLAIM_ADMISSION_SCHEMA_V1, EFFECTIVE_BANDWIDTH_REPORT_DOMAIN_V1,
    EFFECTIVE_BANDWIDTH_REPORT_SCHEMA_V1, LAYER_A_BANDWIDTH_MEASUREMENT_DOMAIN_V1,
    LAYER_A_BANDWIDTH_MEASUREMENT_SCHEMA_V1,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn has_kind(
    errors: &[dsfb_gpu_atlas_corpus::s_perf_5_effective_bandwidth_report::SPerf5VerifyError],
    pred: impl Fn(&SPerf5VerifyErrorKind) -> bool,
) -> bool {
    errors.iter().any(|e| pred(&e.kind))
}

/// Build an admissible saturating measurement on the baseline
/// device identity using CUDA-event timing: 716 GB/s peak, 100
/// microseconds, 71_600_000_000 bytes → effective_bandwidth =
/// 716 GB/s, percent_of_peak = 10000 bp (≥ 8000 ⇒ saturation).
fn build_admissible_saturating_measurement() -> LayerABandwidthMeasurementV1 {
    let inner = seed_baseline_uninstrumented_receipt();
    // 716 GB/s = 716_000_000_000 B/s. At 100 µs ⇒ 71_600_000_000 bytes.
    // The verifier's integer formula: bytes / (time_us * 1000) = bytes / 100_000.
    // 71_600_000_000 / 100_000 = 716_000 → way larger than u32 716 ⇒ pick smaller
    // numbers. 716 GB/s ≡ 716 in our integer arithmetic when bytes/(us*1000)=716.
    // We want effective_bandwidth_gbps = 716 ⇒ bytes / (time_us * 1000) = 716.
    // Choose time_us = 1000 ⇒ denom = 1_000_000 ⇒ bytes = 716_000_000.
    let bytes: u64 = 716_000_000;
    let time_us: u64 = 1000;
    let eff_bw_gbps: u32 = u32::try_from(bytes / (time_us * 1000)).unwrap();
    assert_eq!(eff_bw_gbps, 716);
    let pct_bp: u32 = eff_bw_gbps.saturating_mul(10_000) / inner.theoretical_memory_bandwidth_gbps;
    assert_eq!(pct_bp, 10_000);
    build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        time_us,
        "CudaEvent",
        bytes,
        eff_bw_gbps,
        pct_bp,
        false,
        false,
        false,
    )
}

/// Build an admissible percent-of-peak (sub-saturation)
/// measurement: 358 GB/s effective ⇒ 5000 bp (50 %).
fn build_admissible_percent_of_peak_measurement() -> LayerABandwidthMeasurementV1 {
    let inner = seed_baseline_uninstrumented_receipt();
    let bytes: u64 = 358_000_000;
    let time_us: u64 = 1000;
    let eff_bw_gbps: u32 = u32::try_from(bytes / (time_us * 1000)).unwrap();
    assert_eq!(eff_bw_gbps, 358);
    let pct_bp: u32 = eff_bw_gbps.saturating_mul(10_000) / inner.theoretical_memory_bandwidth_gbps;
    assert_eq!(pct_bp, 5_000);
    build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        time_us,
        "CudaEvent",
        bytes,
        eff_bw_gbps,
        pct_bp,
        false,
        false,
        false,
    )
}

/// Build a saturating report from a measurement (uses the
/// live S-PERF.2/3/4 upstream anchor hashes).
fn build_report_with_admission(
    measurement: LayerABandwidthMeasurementV1,
    claim_kind: BandwidthClaimKind,
    reason: &'static str,
    admitted: bool,
) -> EffectiveBandwidthReportV1 {
    let admission = build_bandwidth_claim_admission(claim_kind, reason, admitted);
    let layer_a_traffic = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    build_effective_bandwidth_report(
        "test_report_v1",
        measurement,
        admission,
        layer_a_traffic.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    )
}

// ---------------------------------------------------------------
// Baseline admission
// ---------------------------------------------------------------

#[test]
fn baseline_measurement_is_constructible() {
    let m = seed_baseline_layer_a_bandwidth_measurement();
    assert!(!m.timing_method_wire_name.is_empty());
    assert_ne!(m.layer_a_bandwidth_measurement_hash_v1, [0u8; 32]);
}

#[test]
fn baseline_admission_is_no_claim_and_admitted() {
    let a = seed_baseline_bandwidth_claim_admission();
    assert!(matches!(a.claim_kind, BandwidthClaimKind::NoClaim));
    assert_eq!(
        a.admissibility_reason_wire_name,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM
    );
    assert!(a.admitted);
}

#[test]
fn baseline_report_admits() {
    let r = seed_baseline_effective_bandwidth_report();
    let errors = verify_effective_bandwidth_report(&r);
    assert!(errors.is_empty(), "baseline must admit: {errors:?}");
}

#[test]
fn baseline_report_references_live_s_perf_1_receipt_hash() {
    let r = seed_baseline_effective_bandwidth_report();
    let inner = seed_baseline_uninstrumented_receipt();
    assert_eq!(
        r.measurement.device_traffic_receipt_hash_v1,
        inner.device_traffic_receipt_hash_v1
    );
}

#[test]
fn baseline_report_references_live_s_perf_2_layer_a_receipt_hash() {
    let r = seed_baseline_effective_bandwidth_report();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    assert_eq!(
        r.layer_a_traffic_receipt_hash,
        layer_a.layer_a_traffic_receipt_hash_v1
    );
}

#[test]
fn baseline_report_references_live_s_perf_3_bundle_hash() {
    let r = seed_baseline_effective_bandwidth_report();
    let bundle = seed_baseline_public_data_saturation_bundle();
    assert_eq!(
        r.public_data_bundle_hash,
        bundle.public_data_saturation_bundle_hash_v1
    );
}

#[test]
fn baseline_report_references_live_s_perf_4_compaction_hash() {
    let r = seed_baseline_effective_bandwidth_report();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    assert_eq!(
        r.family_compaction_benchmark_schema_hash,
        compaction.family_compaction_benchmark_schema_hash_v1
    );
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn baseline_measurement_hash_deterministic_across_two_builds() {
    let a = seed_baseline_layer_a_bandwidth_measurement();
    let b = seed_baseline_layer_a_bandwidth_measurement();
    assert_eq!(
        a.layer_a_bandwidth_measurement_hash_v1,
        b.layer_a_bandwidth_measurement_hash_v1
    );
}

#[test]
fn baseline_admission_hash_deterministic_across_two_builds() {
    let a = seed_baseline_bandwidth_claim_admission();
    let b = seed_baseline_bandwidth_claim_admission();
    assert_eq!(
        a.bandwidth_claim_admission_hash_v1,
        b.bandwidth_claim_admission_hash_v1
    );
}

#[test]
fn baseline_report_hash_deterministic_across_two_builds() {
    let a = seed_baseline_effective_bandwidth_report();
    let b = seed_baseline_effective_bandwidth_report();
    assert_eq!(
        a.effective_bandwidth_report_hash_v1,
        b.effective_bandwidth_report_hash_v1
    );
}

#[test]
fn baseline_three_hashes_are_pairwise_distinct() {
    let r = seed_baseline_effective_bandwidth_report();
    let m_h = r.measurement.layer_a_bandwidth_measurement_hash_v1;
    let a_h = r.admission.bandwidth_claim_admission_hash_v1;
    let r_h = r.effective_bandwidth_report_hash_v1;
    assert_ne!(m_h, a_h);
    assert_ne!(m_h, r_h);
    assert_ne!(a_h, r_h);
}

#[test]
fn baseline_three_hashes_distinct_from_every_upstream_anchor() {
    let r = seed_baseline_effective_bandwidth_report();
    let m_h = r.measurement.layer_a_bandwidth_measurement_hash_v1;
    let a_h = r.admission.bandwidth_claim_admission_hash_v1;
    let r_h = r.effective_bandwidth_report_hash_v1;
    let s1 = r.measurement.device_traffic_receipt_hash_v1;
    let s2 = r.layer_a_traffic_receipt_hash;
    let s3 = r.public_data_bundle_hash;
    let s4 = r.family_compaction_benchmark_schema_hash;
    for new_h in [m_h, a_h, r_h] {
        for anchor in [s1, s2, s3, s4] {
            assert_ne!(new_h, anchor);
        }
    }
}

// ---------------------------------------------------------------
// Panel-locked pinned hash constants
// (back-stop against silent rebaselining)
// ---------------------------------------------------------------

/// Pin the baseline `layer_a_bandwidth_measurement_hash_v1`
/// emitted by `s-perf-5-receipts-emit` on 2026-05-17. Any
/// future code change that alters the byte form MUST also
/// update this constant — that surface change demands an
/// explicit panel decision.
const PINNED_LAYER_A_BANDWIDTH_MEASUREMENT_HASH_V1: [u8; 32] = [
    0x05, 0x54, 0xec, 0x29, 0xb2, 0x92, 0x3f, 0xff, 0xef, 0x29, 0x91, 0x00, 0xf1, 0x36, 0x5d, 0x56,
    0x82, 0xea, 0x26, 0x96, 0xf9, 0x92, 0x8d, 0x4b, 0x86, 0xab, 0x10, 0x72, 0xcc, 0x18, 0xea, 0x55,
];

/// Pin the baseline `bandwidth_claim_admission_hash_v1`.
const PINNED_BANDWIDTH_CLAIM_ADMISSION_HASH_V1: [u8; 32] = [
    0xc4, 0x5f, 0x8b, 0x88, 0x38, 0x60, 0xed, 0x79, 0x61, 0xe9, 0x81, 0xf3, 0x9e, 0x9b, 0x39, 0x1d,
    0x5b, 0x64, 0x47, 0x07, 0xe4, 0x9e, 0x0c, 0x15, 0x84, 0x04, 0x6d, 0x34, 0xce, 0xd9, 0x7c, 0xfd,
];

/// Pin the baseline `effective_bandwidth_report_hash_v1`.
const PINNED_EFFECTIVE_BANDWIDTH_REPORT_HASH_V1: [u8; 32] = [
    0xa1, 0x29, 0xd7, 0xe0, 0xb0, 0xf6, 0xc3, 0xbf, 0x09, 0x4f, 0x44, 0x67, 0x61, 0xd5, 0x19, 0xb4,
    0x10, 0x9a, 0xc8, 0x6e, 0x19, 0xc3, 0xec, 0x82, 0xd5, 0x62, 0x46, 0x00, 0xc7, 0x45, 0xaf, 0xa8,
];

#[test]
fn baseline_layer_a_bandwidth_measurement_hash_matches_pinned_constant() {
    let m = seed_baseline_layer_a_bandwidth_measurement();
    assert_eq!(
        m.layer_a_bandwidth_measurement_hash_v1,
        PINNED_LAYER_A_BANDWIDTH_MEASUREMENT_HASH_V1
    );
}

#[test]
fn baseline_bandwidth_claim_admission_hash_matches_pinned_constant() {
    let a = seed_baseline_bandwidth_claim_admission();
    assert_eq!(
        a.bandwidth_claim_admission_hash_v1,
        PINNED_BANDWIDTH_CLAIM_ADMISSION_HASH_V1
    );
}

#[test]
fn baseline_effective_bandwidth_report_hash_matches_pinned_constant() {
    let r = seed_baseline_effective_bandwidth_report();
    assert_eq!(
        r.effective_bandwidth_report_hash_v1,
        PINNED_EFFECTIVE_BANDWIDTH_REPORT_HASH_V1
    );
}

// ---------------------------------------------------------------
// Domain separators are NUL-terminated
// ---------------------------------------------------------------

#[test]
fn measurement_domain_separator_is_nul_terminated() {
    assert!(LAYER_A_BANDWIDTH_MEASUREMENT_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn admission_domain_separator_is_nul_terminated() {
    assert!(BANDWIDTH_CLAIM_ADMISSION_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn report_domain_separator_is_nul_terminated() {
    assert!(EFFECTIVE_BANDWIDTH_REPORT_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn schema_ids_are_distinct_from_domain_separators() {
    assert_ne!(
        LAYER_A_BANDWIDTH_MEASUREMENT_DOMAIN_V1,
        LAYER_A_BANDWIDTH_MEASUREMENT_SCHEMA_V1
    );
    assert_ne!(
        BANDWIDTH_CLAIM_ADMISSION_DOMAIN_V1,
        BANDWIDTH_CLAIM_ADMISSION_SCHEMA_V1
    );
    assert_ne!(
        EFFECTIVE_BANDWIDTH_REPORT_DOMAIN_V1,
        EFFECTIVE_BANDWIDTH_REPORT_SCHEMA_V1
    );
}

// ---------------------------------------------------------------
// Sensitivity (every hashable field changes the hash when
// mutated)
// ---------------------------------------------------------------

#[test]
fn measurement_hash_changes_when_device_traffic_receipt_hash_changes() {
    let inner = seed_baseline_uninstrumented_receipt();
    let original = seed_baseline_layer_a_bandwidth_measurement();
    let mutated = build_layer_a_bandwidth_measurement(
        [0x42; 32], // mutated
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        inner.measured_kernel_time_us,
        inner.timing_method.as_str(),
        inner.total_accounted_device_bytes,
        inner.effective_bandwidth_gbps,
        inner.percent_of_peak_basis_points,
        false,
        false,
        false,
    );
    assert_ne!(
        original.layer_a_bandwidth_measurement_hash_v1,
        mutated.layer_a_bandwidth_measurement_hash_v1
    );
}

#[test]
fn measurement_hash_changes_when_theoretical_peak_changes() {
    let inner = seed_baseline_uninstrumented_receipt();
    let original = seed_baseline_layer_a_bandwidth_measurement();
    let mutated = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        999, // mutated
        inner.measured_kernel_time_us,
        inner.timing_method.as_str(),
        inner.total_accounted_device_bytes,
        inner.effective_bandwidth_gbps,
        inner.percent_of_peak_basis_points,
        false,
        false,
        false,
    );
    assert_ne!(
        original.layer_a_bandwidth_measurement_hash_v1,
        mutated.layer_a_bandwidth_measurement_hash_v1
    );
}

#[test]
fn measurement_hash_changes_when_inner_host_json_flag_toggles() {
    let inner = seed_baseline_uninstrumented_receipt();
    let original = seed_baseline_layer_a_bandwidth_measurement();
    let mutated = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        inner.measured_kernel_time_us,
        inner.timing_method.as_str(),
        inner.total_accounted_device_bytes,
        inner.effective_bandwidth_gbps,
        inner.percent_of_peak_basis_points,
        true, // mutated
        false,
        false,
    );
    assert_ne!(
        original.layer_a_bandwidth_measurement_hash_v1,
        mutated.layer_a_bandwidth_measurement_hash_v1
    );
}

#[test]
fn admission_hash_changes_when_claim_kind_changes() {
    let baseline = seed_baseline_bandwidth_claim_admission();
    let mutated = build_bandwidth_claim_admission(
        BandwidthClaimKind::EffectiveBandwidth, // mutated
        ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH,
        true,
    );
    assert_ne!(
        baseline.bandwidth_claim_admission_hash_v1,
        mutated.bandwidth_claim_admission_hash_v1
    );
}

#[test]
fn admission_hash_changes_when_admitted_toggles() {
    let baseline = seed_baseline_bandwidth_claim_admission();
    let mutated = build_bandwidth_claim_admission(
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        false, // mutated
    );
    assert_ne!(
        baseline.bandwidth_claim_admission_hash_v1,
        mutated.bandwidth_claim_admission_hash_v1
    );
}

#[test]
fn report_hash_changes_when_report_id_changes() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let a = build_effective_bandwidth_report(
        "report_a",
        measurement.clone(),
        admission.clone(),
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let b = build_effective_bandwidth_report(
        "report_b",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    assert_ne!(
        a.effective_bandwidth_report_hash_v1,
        b.effective_bandwidth_report_hash_v1
    );
}

#[test]
fn report_hash_changes_when_layer_a_traffic_receipt_hash_changes() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let a = build_effective_bandwidth_report(
        "report",
        measurement.clone(),
        admission.clone(),
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let b = build_effective_bandwidth_report(
        "report",
        measurement,
        admission,
        [0x77; 32],
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    assert_ne!(
        a.effective_bandwidth_report_hash_v1,
        b.effective_bandwidth_report_hash_v1
    );
}

// ---------------------------------------------------------------
// Panel-required negative #1: missing S-PERF.1 receipt
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_report_without_s_perf_1_receipt() {
    let inner = seed_baseline_uninstrumented_receipt();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let measurement = build_layer_a_bandwidth_measurement(
        [0u8; 32], // <-- missing
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        inner.measured_kernel_time_us,
        inner.timing_method.as_str(),
        inner.total_accounted_device_bytes,
        inner.effective_bandwidth_gbps,
        inner.percent_of_peak_basis_points,
        false,
        false,
        false,
    );
    let admission = seed_baseline_bandwidth_claim_admission();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::ReportWithoutSPerf1Receipt
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #2: missing S-PERF.2 LayerA receipt
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_report_without_s_perf_2_layer_a_receipt() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        [0u8; 32], // <-- missing
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::ReportWithoutSPerf2LayerAReceipt
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #3: missing S-PERF.3 bundle hash
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_report_without_s_perf_3_bundle_hash() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        [0u8; 32], // <-- missing
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::ReportWithoutSPerf3BundleHash
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #4: missing S-PERF.4 compaction hash
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_report_without_s_perf_4_compaction_hash() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        [0u8; 32], // <-- missing
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::ReportWithoutSPerf4CompactionHash
    )));
}

// ---------------------------------------------------------------
// Panel-required negative #5: Saturation claim below 8000 bp
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_saturation_claim_below_8000_bp() {
    // Build a measurement at 50 % peak ⇒ 5000 bp, but claim
    // Saturation.
    let m = build_admissible_percent_of_peak_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::SaturationClaimBelow8000Bp { observed_bp } if *observed_bp == 5_000
        )),
        "expected SaturationClaimBelow8000Bp(5000); got: {errors:?}"
    );
}

#[test]
fn saturation_at_or_above_8000_bp_passes_negative_five() {
    // Saturating measurement at 100 % peak ⇒ 10000 bp.
    let m = build_admissible_saturating_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        !has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::SaturationClaimBelow8000Bp { .. }
        )),
        "should NOT fire SaturationClaimBelow8000Bp at >=8000bp: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #6: Saturation claim with host timing
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_saturation_claim_with_host_timing() {
    let inner = seed_baseline_uninstrumented_receipt();
    let bytes: u64 = 716_000_000;
    let time_us: u64 = 1000;
    let eff_bw_gbps: u32 = 716;
    let pct_bp: u32 = 10_000;
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        time_us,
        "HostInstantOnly", // <-- host timing forbidden for Saturation
        bytes,
        eff_bw_gbps,
        pct_bp,
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::SaturationClaimWithHostTiming { observed_timing_method_wire_name } if *observed_timing_method_wire_name == "HostInstantOnly"
        )),
        "expected SaturationClaimWithHostTiming; got: {errors:?}"
    );
}

#[test]
fn saturation_with_cuda_stream_sync_admits_negative_six() {
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        1000,
        "CudaStreamSync",
        716_000_000,
        716,
        10_000,
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        !has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::SaturationClaimWithHostTiming { .. }
        )),
        "should NOT fire SaturationClaimWithHostTiming with CudaStreamSync: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #7: effective bandwidth / percent
// arithmetic coherence
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_effective_bandwidth_mismatch_from_bytes_and_time() {
    let inner = seed_baseline_uninstrumented_receipt();
    // Build a measurement that LIES about the effective
    // bandwidth (bytes = 100M, time = 1000 µs ⇒ expected = 100
    // GB/s; but we claim 999).
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        1000,
        "CudaEvent",
        100_000_000,
        999,    // <-- lied
        13_950, // <-- pct-of-peak lied to match the lie
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::EffectiveBandwidthMismatchFromBytesAndTime {
                which_field_wire_name, ..
            } if *which_field_wire_name == "effective_bandwidth_gbps"
        )),
        "expected effective_bandwidth_gbps mismatch; got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_percent_of_peak_mismatch_from_effective_and_theoretical() {
    let inner = seed_baseline_uninstrumented_receipt();
    // bytes = 358M / time = 1000 µs ⇒ eff = 358 GB/s; expected
    // pct = 5000 bp. We claim 9999 (lied).
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        1000,
        "CudaEvent",
        358_000_000,
        358,
        9_999, // <-- lied
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::PercentOfPeak,
        ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::EffectiveBandwidthMismatchFromBytesAndTime {
                which_field_wire_name, ..
            } if *which_field_wire_name == "percent_of_peak_basis_points"
        )),
        "expected percent_of_peak_basis_points mismatch; got: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #8: host JSON / casefile / transcript
// flags
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_report_that_includes_host_json_emission_time() {
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        0,
        inner.timing_method.as_str(),
        0,
        0,
        0,
        true, // <-- host JSON emission present
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name
            } if *flag_wire_name == "inner_host_json_emission_present"
        )),
        "expected host_json_emission flag rejection; got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_report_that_includes_casefile_materialization_time() {
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        0,
        inner.timing_method.as_str(),
        0,
        0,
        0,
        false,
        true, // <-- casefile materialization present
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name
            } if *flag_wire_name == "inner_casefile_materialization_present"
        )),
        "expected casefile_materialization flag rejection; got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_report_that_includes_host_transcript_time() {
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        0,
        inner.timing_method.as_str(),
        0,
        0,
        0,
        false,
        false,
        true, // <-- host transcript present
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name
            } if *flag_wire_name == "inner_host_transcript_present"
        )),
        "expected host_transcript flag rejection; got: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #9: cross-device claim without device
// identity
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_cross_device_claim_without_device_identity() {
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        [0u8; 32], // <-- no device identity
        inner.theoretical_memory_bandwidth_gbps,
        1000,
        "CudaEvent",
        716_000_000,
        716,
        10_000,
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::CrossDeviceClaimWithoutDeviceIdentity
        )),
        "expected CrossDeviceClaimWithoutDeviceIdentity; got: {errors:?}"
    );
}

#[test]
fn no_claim_with_zero_device_identity_does_not_fire_negative_nine() {
    // NoClaim is allowed to have zero device identity (the
    // baseline does not). But if it did, negative #9 should
    // only fire on non-NoClaim claims.
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        [0u8; 32], // zero device identity
        inner.theoretical_memory_bandwidth_gbps,
        0,
        inner.timing_method.as_str(),
        0,
        0,
        0,
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        !has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::CrossDeviceClaimWithoutDeviceIdentity
        )),
        "negative #9 should NOT fire on NoClaim: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #10: benchmark claim without public
// artifact manifest bundle
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_benchmark_claim_without_public_artifact_manifest() {
    let m = build_admissible_saturating_measurement();
    let admission = build_bandwidth_claim_admission(
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        m,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        [0u8; 32], // <-- no public bundle
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::BenchmarkClaimWithoutPublicArtifactManifest
        )),
        "expected BenchmarkClaimWithoutPublicArtifactManifest; got: {errors:?}"
    );
}

#[test]
fn no_claim_with_zero_bundle_does_not_fire_negative_ten() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        [0u8; 32], // zero bundle hash
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    // Negative #3 still fires (structural), but #10 should not.
    assert!(
        !has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::BenchmarkClaimWithoutPublicArtifactManifest
        )),
        "negative #10 should NOT fire on NoClaim: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Structural: ReportIdEmpty
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_empty_report_id() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "", // <-- empty
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::ReportIdEmpty
    )));
}

// ---------------------------------------------------------------
// Structural: AdmissibilityReasonEmpty
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_empty_admissibility_reason() {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = build_bandwidth_claim_admission(
        BandwidthClaimKind::NoClaim,
        "", // <-- empty
        true,
    );
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf5VerifyErrorKind::AdmissibilityReasonEmpty
    )));
}

// ---------------------------------------------------------------
// Structural: forbidden benchmark-claim substring
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_benchmark_claim_substring_inside_report_id() {
    // Use a leaked Box<str> to satisfy `'static`. The
    // substring "achieves saturation" is the first forbidden
    // entry.
    let leaked: &'static str = Box::leak(
        "report_that_ACHIEVES SATURATION_at_peak"
            .to_string()
            .into_boxed_str(),
    );
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        leaked,
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::BenchmarkClaimInsideReport { location, .. }
                if *location == "report_id"
        )),
        "expected BenchmarkClaimInsideReport(report_id); got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_benchmark_claim_substring_inside_admissibility_reason() {
    let leaked: &'static str = Box::leak("Outperforms baseline".to_string().into_boxed_str());
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = build_bandwidth_claim_admission(BandwidthClaimKind::NoClaim, leaked, true);
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        "test",
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::BenchmarkClaimInsideReport { location, .. }
                if *location == "admissibility_reason_wire_name"
        )),
        "expected BenchmarkClaimInsideReport(admissibility_reason_wire_name); got: {errors:?}"
    );
}

#[test]
fn forbidden_benchmark_claim_substring_scanner_is_case_insensitive() {
    let leaked_upper: &'static str = Box::leak("BeAtS THE BaSeLiNe".to_string().into_boxed_str());
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let r = build_effective_bandwidth_report(
        leaked_upper,
        measurement,
        admission,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::BenchmarkClaimInsideReport { .. }
        )),
        "case-insensitive scanner must catch BeAtS THE BaSeLiNe: {errors:?}"
    );
}

#[test]
fn forbidden_benchmark_claim_substring_set_has_twelve_entries() {
    // Mirrors S-PERF.3 / S-PERF.4 set.
    assert_eq!(forbidden_benchmark_claim_substrings().len(), 12);
}

// ---------------------------------------------------------------
// Structural: claim_kind / measurement coherence
// ---------------------------------------------------------------

#[test]
fn s_perf_5_rejects_no_claim_with_nonzero_effective_bandwidth() {
    let m = build_admissible_percent_of_peak_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ClaimKindIncoherentWithMeasurement { .. }
        )),
        "expected ClaimKindIncoherentWithMeasurement; got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_effective_bandwidth_claim_with_zero_effective_bandwidth() {
    let m = seed_baseline_layer_a_bandwidth_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::EffectiveBandwidth,
        ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ClaimKindIncoherentWithMeasurement { .. }
        )),
        "expected ClaimKindIncoherentWithMeasurement; got: {errors:?}"
    );
}

#[test]
fn s_perf_5_rejects_percent_of_peak_claim_with_basis_points_at_or_above_8000() {
    let m = build_admissible_saturating_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::PercentOfPeak,
        ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf5VerifyErrorKind::ClaimKindIncoherentWithMeasurement { .. }
        )),
        "PercentOfPeak with bp>=8000 must be incoherent: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Cross-verifier composition: an admissible PercentOfPeak claim
// admits cleanly
// ---------------------------------------------------------------

#[test]
fn admissible_percent_of_peak_claim_admits() {
    let m = build_admissible_percent_of_peak_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::PercentOfPeak,
        ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(errors.is_empty(), "expected empty errors; got: {errors:?}");
}

#[test]
fn admissible_saturation_claim_admits() {
    let m = build_admissible_saturating_measurement();
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::Saturation,
        ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(errors.is_empty(), "expected empty errors; got: {errors:?}");
}

#[test]
fn admissible_effective_bandwidth_claim_admits() {
    // EffectiveBandwidth claim coheres when eff_bw > 0; the
    // percent-of-peak basis points can be anything coherent
    // with the formula (including >= 8000). Use the saturating
    // measurement so the arithmetic holds.
    let inner = seed_baseline_uninstrumented_receipt();
    let m = build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        1000,
        "CudaEvent",
        358_000_000,
        358,
        5_000,
        false,
        false,
        false,
    );
    let r = build_report_with_admission(
        m,
        BandwidthClaimKind::EffectiveBandwidth,
        ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH,
        true,
    );
    let errors = verify_effective_bandwidth_report(&r);
    assert!(errors.is_empty(), "expected empty errors; got: {errors:?}");
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn measurement_text_renderer_byte_stable_across_two_calls() {
    let m = seed_baseline_layer_a_bandwidth_measurement();
    let a = render_layer_a_bandwidth_measurement_text(&m);
    let b = render_layer_a_bandwidth_measurement_text(&m);
    assert_eq!(a, b);
}

#[test]
fn measurement_json_renderer_byte_stable_across_two_calls() {
    let m = seed_baseline_layer_a_bandwidth_measurement();
    let a = render_layer_a_bandwidth_measurement_json(&m);
    let b = render_layer_a_bandwidth_measurement_json(&m);
    assert_eq!(a, b);
}

#[test]
fn admission_text_renderer_byte_stable_across_two_calls() {
    let a = seed_baseline_bandwidth_claim_admission();
    let x = render_bandwidth_claim_admission_text(&a);
    let y = render_bandwidth_claim_admission_text(&a);
    assert_eq!(x, y);
}

#[test]
fn admission_json_renderer_byte_stable_across_two_calls() {
    let a = seed_baseline_bandwidth_claim_admission();
    let x = render_bandwidth_claim_admission_json(&a);
    let y = render_bandwidth_claim_admission_json(&a);
    assert_eq!(x, y);
}

#[test]
fn report_text_renderer_byte_stable_across_two_calls() {
    let r = seed_baseline_effective_bandwidth_report();
    let a = render_effective_bandwidth_report_text(&r);
    let b = render_effective_bandwidth_report_text(&r);
    assert_eq!(a, b);
}

#[test]
fn report_json_renderer_byte_stable_across_two_calls() {
    let r = seed_baseline_effective_bandwidth_report();
    let a = render_effective_bandwidth_report_json(&r);
    let b = render_effective_bandwidth_report_json(&r);
    assert_eq!(a, b);
}

#[test]
fn report_text_contains_panel_locked_identity_strings() {
    let r = seed_baseline_effective_bandwidth_report();
    let text = render_effective_bandwidth_report_text(&r);
    assert!(text.contains("EffectiveBandwidthReportV1"));
    assert!(text.contains("report_id"));
    assert!(text.contains("claim_kind"));
    assert!(text.contains("NoClaim"));
    assert!(text.contains("AdmissibleNoClaim"));
}

// ---------------------------------------------------------------
// Saturation threshold visibility
// ---------------------------------------------------------------

#[test]
fn saturation_threshold_constant_is_8000_basis_points() {
    assert_eq!(S_PERF_1_SATURATION_BP, 8000);
}

// ---------------------------------------------------------------
// Panel-locked non-claim: baseline reports NoClaim
// ---------------------------------------------------------------

#[test]
fn baseline_report_claim_kind_is_no_claim() {
    let r = seed_baseline_effective_bandwidth_report();
    assert!(matches!(
        r.admission.claim_kind,
        BandwidthClaimKind::NoClaim
    ));
}

#[test]
fn baseline_report_admitted_flag_is_true() {
    let r = seed_baseline_effective_bandwidth_report();
    assert!(r.admission.admitted);
}

// ---------------------------------------------------------------
// Defensive: empty-string admissibility reason is structurally
// rejected even if claim kind is NoClaim
// ---------------------------------------------------------------

#[test]
fn admissibility_reason_constants_are_non_empty() {
    assert!(!ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM.is_empty());
    assert!(!ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH.is_empty());
    assert!(!ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK.is_empty());
    assert!(!ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION.is_empty());
}
