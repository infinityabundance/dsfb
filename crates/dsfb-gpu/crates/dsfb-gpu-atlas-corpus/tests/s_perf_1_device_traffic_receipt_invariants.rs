//! S-PERF.1 acceptance suite — `DeviceTrafficReceiptV1` +
//! `DeviceBandwidthClaimPolicyV1` invariants.
//!
//! Eight panel-required load-bearing negatives:
//!
//! 1. `s_perf_1_rejects_bandwidth_claim_without_byte_accounting`
//! 2. `s_perf_1_rejects_peak_percentage_without_device_bandwidth_declared`
//! 3. `s_perf_1_rejects_layer_a_claim_when_host_json_time_included`
//! 4. `s_perf_1_rejects_saturation_claim_without_cuda_event_timing`
//! 5. `s_perf_1_rejects_cross_device_comparison_without_device_identity`
//! 6. `s_perf_1_rejects_effective_bandwidth_when_total_bytes_zero`
//! 7. `s_perf_1_rejects_percent_of_peak_above_100_without_explicit_error_flag`
//! 8. `s_perf_1_rejects_receipt_missing_contract_hashes`
//!
//! Plus structural defect tests (empty device name / driver /
//! cuda version / sm_arch=0 / TimingMethod::Unknown with
//! non-zero time / accounted-bytes mismatch sum of fields),
//! determinism (receipt + policy hash byte-stable across two
//! builds; renderers byte-stable), sensitivity (every receipt
//! field that participates in the hash changes the hash when
//! mutated), and rendering smoke tests.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::s_perf_1_device_traffic_receipt::{
    build_device_traffic_receipt, build_panel_locked_bandwidth_claim_policy,
    compute_device_identity_hash, panel_locked_bandwidth_claim_policy_lines,
    render_bandwidth_claim_policy_json, render_bandwidth_claim_policy_text,
    render_device_traffic_receipt_json, render_device_traffic_receipt_text,
    seed_baseline_uninstrumented_receipt, verify_cross_device_comparison,
    verify_device_traffic_receipt, DeviceBandwidthLayer, DeviceTrafficReceiptComparison,
    DeviceTrafficReceiptV1, SPerf1VerifyErrorKind, TimingMethod,
    DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1, DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1,
    DEVICE_IDENTITY_HASH_DOMAIN_V1, DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1,
    DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1, S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES,
    S_PERF_1_SATURATION_BP,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Build a receipt with caller-controlled bandwidth claim
/// fields. Holds device identity + sm_arch + driver / cuda
/// version + contract hashes constant so the test focuses on
/// the field under inspection.
fn build_test_receipt(
    layer: DeviceBandwidthLayer,
    timing_method: TimingMethod,
    theoretical_memory_bandwidth_gbps: u32,
    measured_kernel_time_us: u64,
    input_bytes: u64,
    evidence_bytes_read: u64,
    evidence_bytes_written: u64,
    witness_bytes_written: u64,
    fusion_bytes_read_written: u64,
    digest_bytes_read: u64,
    candidate_summary_bytes: u64,
    total_accounted_device_bytes: u64,
    effective_bandwidth_gbps: u32,
    percent_of_peak_basis_points: u32,
    accounting_overflow_acknowledged: bool,
    contract_hashes: Vec<[u8; 32]>,
) -> DeviceTrafficReceiptV1 {
    let corpus_anchor = compute_corpus_hash_v1().bytes;
    build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        89,
        "13.2.0",
        "13.2",
        theoretical_memory_bandwidth_gbps,
        measured_kernel_time_us,
        timing_method,
        layer,
        152,
        64,
        input_bytes,
        evidence_bytes_read,
        evidence_bytes_written,
        witness_bytes_written,
        fusion_bytes_read_written,
        digest_bytes_read,
        candidate_summary_bytes,
        total_accounted_device_bytes,
        effective_bandwidth_gbps,
        percent_of_peak_basis_points,
        accounting_overflow_acknowledged,
        vec![corpus_anchor],
        contract_hashes,
    )
}

fn one_contract_hash() -> Vec<[u8; 32]> {
    vec![compute_corpus_hash_v1().bytes]
}

// ---------------------------------------------------------------
// Baseline admission
// ---------------------------------------------------------------

#[test]
fn baseline_uninstrumented_receipt_admits() {
    let r = seed_baseline_uninstrumented_receipt();
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.is_empty(),
        "baseline uninstrumented receipt must admit: {errors:?}"
    );
}

#[test]
fn baseline_receipt_has_non_zero_device_identity() {
    let r = seed_baseline_uninstrumented_receipt();
    assert_ne!(r.device_uuid_or_identity_hash, [0u8; 32]);
}

#[test]
fn baseline_receipt_has_declared_panel_locked_peak_bandwidth() {
    // RTX 4080 SUPER vendor datasheet: 716 GB/s peak memory
    // bandwidth. The seed pins this value; rebaselining the
    // seed without panel approval changes the baseline
    // receipt hash.
    let r = seed_baseline_uninstrumented_receipt();
    assert_eq!(r.theoretical_memory_bandwidth_gbps, 716);
}

#[test]
fn baseline_receipt_declares_corpus_anchor() {
    let r = seed_baseline_uninstrumented_receipt();
    let corpus_anchor = compute_corpus_hash_v1().bytes;
    assert!(r.artifact_hashes.contains(&corpus_anchor));
    assert!(r.contract_hashes.contains(&corpus_anchor));
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn device_traffic_receipt_hash_is_deterministic() {
    let a = seed_baseline_uninstrumented_receipt();
    let b = seed_baseline_uninstrumented_receipt();
    assert_eq!(
        a.device_traffic_receipt_hash_v1,
        b.device_traffic_receipt_hash_v1
    );
}

#[test]
fn bandwidth_claim_policy_hash_is_deterministic() {
    let a = build_panel_locked_bandwidth_claim_policy();
    let b = build_panel_locked_bandwidth_claim_policy();
    assert_eq!(
        a.device_bandwidth_claim_policy_hash_v1,
        b.device_bandwidth_claim_policy_hash_v1
    );
}

#[test]
fn receipt_text_render_is_deterministic() {
    let r = seed_baseline_uninstrumented_receipt();
    let a = render_device_traffic_receipt_text(&r);
    let b = render_device_traffic_receipt_text(&r);
    assert_eq!(a, b);
}

#[test]
fn receipt_json_render_is_deterministic() {
    let r = seed_baseline_uninstrumented_receipt();
    let a = render_device_traffic_receipt_json(&r);
    let b = render_device_traffic_receipt_json(&r);
    assert_eq!(a, b);
}

#[test]
fn policy_text_render_is_deterministic() {
    let p = build_panel_locked_bandwidth_claim_policy();
    let a = render_bandwidth_claim_policy_text(&p);
    let b = render_bandwidth_claim_policy_text(&p);
    assert_eq!(a, b);
}

#[test]
fn policy_json_render_is_deterministic() {
    let p = build_panel_locked_bandwidth_claim_policy();
    let a = render_bandwidth_claim_policy_json(&p);
    let b = render_bandwidth_claim_policy_json(&p);
    assert_eq!(a, b);
}

#[test]
fn device_identity_hash_is_deterministic() {
    let a = compute_device_identity_hash("RTX 4080 SUPER", 89);
    let b = compute_device_identity_hash("RTX 4080 SUPER", 89);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Hash distinctness
// ---------------------------------------------------------------

#[test]
fn receipt_and_policy_hashes_are_distinct() {
    let r = seed_baseline_uninstrumented_receipt();
    let p = build_panel_locked_bandwidth_claim_policy();
    assert_ne!(
        r.device_traffic_receipt_hash_v1,
        p.device_bandwidth_claim_policy_hash_v1
    );
}

#[test]
fn receipt_hash_differs_from_corpus_hash_v1() {
    let r = seed_baseline_uninstrumented_receipt();
    assert_ne!(
        r.device_traffic_receipt_hash_v1,
        compute_corpus_hash_v1().bytes
    );
}

#[test]
fn device_identity_hash_changes_when_arch_changes() {
    let a = compute_device_identity_hash("RTX 4080 SUPER", 89);
    let b = compute_device_identity_hash("RTX 4080 SUPER", 90);
    assert_ne!(a, b);
}

#[test]
fn device_identity_hash_changes_when_name_changes() {
    let a = compute_device_identity_hash("RTX 4080 SUPER", 89);
    let b = compute_device_identity_hash("H100", 89);
    assert_ne!(a, b);
}

#[test]
fn device_identity_hash_is_non_zero() {
    let h = compute_device_identity_hash("RTX 4080 SUPER", 89);
    assert_ne!(h, [0u8; 32]);
}

// ---------------------------------------------------------------
// Domain separator + schema id discipline
// ---------------------------------------------------------------

#[test]
fn domain_separators_are_pairwise_distinct() {
    assert_ne!(
        DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1,
        DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1
    );
    assert_ne!(
        DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1,
        DEVICE_IDENTITY_HASH_DOMAIN_V1
    );
    assert_ne!(
        DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1,
        DEVICE_IDENTITY_HASH_DOMAIN_V1
    );
}

#[test]
fn domain_separators_end_with_nul_byte() {
    assert!(DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1.ends_with('\0'));
    assert!(DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1.ends_with('\0'));
    assert!(DEVICE_IDENTITY_HASH_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn schema_ids_are_pairwise_distinct() {
    assert_ne!(
        DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1,
        DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1
    );
}

// ---------------------------------------------------------------
// Policy structural pins
// ---------------------------------------------------------------

#[test]
fn panel_locked_policy_has_eight_lines() {
    assert_eq!(panel_locked_bandwidth_claim_policy_lines().len(), 8);
    assert_eq!(S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES.len(), 8);
}

#[test]
fn panel_locked_policy_lines_match_in_built_policy() {
    let p = build_panel_locked_bandwidth_claim_policy();
    assert_eq!(p.policy_lines, S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES);
}

#[test]
fn panel_locked_saturation_threshold_is_80_percent() {
    // 80.00 % = 8000 basis points.
    assert_eq!(S_PERF_1_SATURATION_BP, 8_000);
}

// ---------------------------------------------------------------
// Eight panel-required load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn s_perf_1_rejects_bandwidth_claim_without_byte_accounting() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        716,
        1_000_000,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,   // total_accounted_device_bytes
        100, // effective_bandwidth_gbps — claim without bytes
        500, // percent_of_peak_basis_points — claim without bytes
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::BandwidthClaimWithoutByteAccounting { .. }
        )),
        "bandwidth claim with zero bytes must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_peak_percentage_without_device_bandwidth_declared() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        0, // theoretical_memory_bandwidth_gbps — undeclared
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,  // total_accounted_device_bytes
        50,   // effective_bandwidth_gbps
        2000, // percent_of_peak_basis_points — 20 % of nothing
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::PeakPercentageWithoutDeviceBandwidthDeclared { .. }
        )),
        "percent-of-peak without theoretical bandwidth must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_layer_a_claim_when_host_json_time_included() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::HostJsonInclusiveTime,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700, // total_accounted_device_bytes
        50,
        500,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::LayerAClaimWithHostJsonInclusiveTime
        )),
        "Layer-A + HostJsonInclusiveTime must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_saturation_claim_without_cuda_event_timing() {
    // 90 % of peak (9000 basis points) — saturation claim;
    // backed only by HostInstantOnly — insufficient.
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        644,
        9_000,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::SaturationClaimWithoutCudaEventTiming { .. }
        )),
        "saturation claim with non-CUDA-event timing must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_cross_device_comparison_without_device_identity() {
    // Per-receipt failure: zero UUID surfaces the rule on
    // single-receipt verification.
    let r = build_device_traffic_receipt(
        "RTX 4080 SUPER",
        [0u8; 32], // zero device identity — panel-locked trip wire
        89,
        "13.2.0",
        "13.2",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::CrossDeviceComparisonWithoutDeviceIdentity
        )),
        "zero device identity must surface CrossDeviceComparisonWithoutDeviceIdentity: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_effective_bandwidth_when_total_bytes_zero() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        716,
        1_000_000,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,   // total_accounted_device_bytes
        100, // effective_bandwidth_gbps without bytes
        0,   // percent_of_peak_basis_points — only the eff path
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::EffectiveBandwidthWhenTotalBytesZero { .. }
        )),
        "effective_bandwidth_gbps>0 with total_bytes=0 must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_percent_of_peak_above_100_without_explicit_error_flag() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        644,
        12_500, // 125 % — over 100 without flag
        false,  // not acknowledged
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::PercentOfPeakAbove100WithoutErrorFlag { .. }
        )),
        "percent>100 without flag must surface: {errors:?}"
    );
}

#[test]
fn s_perf_1_rejects_receipt_missing_contract_hashes() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::HostInstantOnly,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        644,
        5_000,
        false,
        Vec::new(), // contract_hashes empty
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf1VerifyErrorKind::ReceiptMissingContractHashes)),
        "empty contract_hashes must surface ReceiptMissingContractHashes: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-locked negative #5 also surfaces from the comparison
// verifier
// ---------------------------------------------------------------

#[test]
fn cross_device_comparison_with_zero_uuid_surfaces_negative_5() {
    let r1 = seed_baseline_uninstrumented_receipt();
    let r2 = build_device_traffic_receipt(
        "Anonymous GPU",
        [0u8; 32],
        89,
        "13.2.0",
        "13.2",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let receipts = [r1, r2];
    let comparison = DeviceTrafficReceiptComparison {
        receipts: &receipts,
    };
    let errors = verify_cross_device_comparison(&comparison);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::CrossDeviceComparisonWithoutDeviceIdentity
        )),
        "comparison with zero-UUID receipt must surface: {errors:?}"
    );
}

#[test]
fn cross_device_comparison_requires_at_least_two_receipts() {
    let r = seed_baseline_uninstrumented_receipt();
    let receipts = [r];
    let comparison = DeviceTrafficReceiptComparison {
        receipts: &receipts,
    };
    let errors = verify_cross_device_comparison(&comparison);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::ComparisonRequiresAtLeastTwoReceipts { actual: 1 }
        )),
        "single-receipt comparison must surface ComparisonRequiresAtLeastTwoReceipts: {errors:?}"
    );
}

#[test]
fn cross_device_comparison_requires_distinct_device_identities() {
    // Two receipts with the SAME non-zero UUID — not a cross-
    // device comparison.
    let r1 = seed_baseline_uninstrumented_receipt();
    let r2 = seed_baseline_uninstrumented_receipt();
    let receipts = [r1, r2];
    let comparison = DeviceTrafficReceiptComparison {
        receipts: &receipts,
    };
    let errors = verify_cross_device_comparison(&comparison);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::ComparisonReceiptsShareDeviceIdentity
        )),
        "shared-UUID comparison must surface ComparisonReceiptsShareDeviceIdentity: {errors:?}"
    );
}

#[test]
fn cross_device_comparison_with_two_distinct_devices_admits() {
    let r1 = seed_baseline_uninstrumented_receipt();
    let r2 = build_device_traffic_receipt(
        "H100",
        compute_device_identity_hash("H100", 90),
        90,
        "13.2.0",
        "13.2",
        3_350,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let receipts = [r1, r2];
    let comparison = DeviceTrafficReceiptComparison {
        receipts: &receipts,
    };
    let errors = verify_cross_device_comparison(&comparison);
    assert!(
        errors.is_empty(),
        "two distinct-UUID receipts must admit cross-device comparison: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Saturation claim with CudaStreamSync admits
// ---------------------------------------------------------------

#[test]
fn saturation_claim_with_cuda_stream_sync_admits() {
    // 80 % of peak (8000 basis points) backed by
    // CudaStreamSync — admissible per is_device_resident.
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::CudaStreamSync,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        572,
        8_000,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        !errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::SaturationClaimWithoutCudaEventTiming { .. }
        )),
        "CudaStreamSync should admit a saturation claim: {errors:?}"
    );
}

#[test]
fn saturation_claim_with_cuda_event_admits() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::CudaEvent,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        572,
        8_000,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.is_empty(),
        "CudaEvent saturation claim with byte accounting must admit: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Percent-of-peak above 100 admitted WITH explicit flag
// ---------------------------------------------------------------

#[test]
fn percent_of_peak_above_100_admitted_with_explicit_flag() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::CudaEvent,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        644,
        12_500,
        true, // accounting_overflow_acknowledged
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        !errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::PercentOfPeakAbove100WithoutErrorFlag { .. }
        )),
        "percent>100 WITH flag should not surface negative #7: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn empty_device_name_surfaces_structural_defect() {
    let r = build_device_traffic_receipt(
        "", // empty device_name
        compute_device_identity_hash("anonymous", 89),
        89,
        "13.2.0",
        "13.2",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf1VerifyErrorKind::DeviceNameEmpty)),
        "empty device_name must surface DeviceNameEmpty: {errors:?}"
    );
}

#[test]
fn empty_driver_version_surfaces_structural_defect() {
    let r = build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        89,
        "",
        "13.2",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf1VerifyErrorKind::DriverVersionEmpty)),
        "empty driver_version must surface DriverVersionEmpty: {errors:?}"
    );
}

#[test]
fn empty_cuda_version_surfaces_structural_defect() {
    let r = build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        89,
        "13.2.0",
        "",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf1VerifyErrorKind::CudaVersionEmpty)),
        "empty cuda_version must surface CudaVersionEmpty: {errors:?}"
    );
}

#[test]
fn sm_arch_zero_surfaces_structural_defect() {
    let r = build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        0,
        "13.2.0",
        "13.2",
        716,
        0,
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, SPerf1VerifyErrorKind::SmArchZero)),
        "sm_arch=0 must surface SmArchZero: {errors:?}"
    );
}

#[test]
fn unknown_timing_method_with_non_zero_time_surfaces_structural_defect() {
    let r = build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        89,
        "13.2.0",
        "13.2",
        716,
        1_000, // non-zero time
        TimingMethod::Unknown,
        DeviceBandwidthLayer::LayerB,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        Vec::new(),
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::TimingMethodUnknownWithNonZeroTime
        )),
        "Unknown timing with non-zero time must surface TimingMethodUnknownWithNonZeroTime: {errors:?}"
    );
}

#[test]
fn accounted_bytes_mismatch_surfaces_structural_defect() {
    // Sum of byte fields = 700; total claimed = 999 → mismatch.
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerB,
        TimingMethod::CudaEvent,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        999,
        0,
        0,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            SPerf1VerifyErrorKind::AccountedBytesMismatchSumOfFields { .. }
        )),
        "total mismatch with sum of fields must surface: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Sensitivity: every hashable field changes the receipt hash
// ---------------------------------------------------------------

#[test]
fn receipt_hash_changes_when_sm_arch_changes() {
    let baseline = seed_baseline_uninstrumented_receipt();
    let mutated = build_device_traffic_receipt(
        baseline.device_name,
        baseline.device_uuid_or_identity_hash,
        90, // mutated
        baseline.driver_version,
        baseline.cuda_version,
        baseline.theoretical_memory_bandwidth_gbps,
        baseline.measured_kernel_time_us,
        baseline.timing_method,
        baseline.layer,
        baseline.detector_count,
        baseline.catalog_count,
        baseline.input_bytes,
        baseline.evidence_bytes_read,
        baseline.evidence_bytes_written,
        baseline.witness_bytes_written,
        baseline.fusion_bytes_read_written,
        baseline.digest_bytes_read,
        baseline.candidate_summary_bytes,
        baseline.total_accounted_device_bytes,
        baseline.effective_bandwidth_gbps,
        baseline.percent_of_peak_basis_points,
        baseline.accounting_overflow_acknowledged,
        baseline.artifact_hashes.clone(),
        baseline.contract_hashes.clone(),
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

#[test]
fn receipt_hash_changes_when_timing_method_changes() {
    let baseline = seed_baseline_uninstrumented_receipt();
    let mutated = build_device_traffic_receipt(
        baseline.device_name,
        baseline.device_uuid_or_identity_hash,
        baseline.sm_arch,
        baseline.driver_version,
        baseline.cuda_version,
        baseline.theoretical_memory_bandwidth_gbps,
        baseline.measured_kernel_time_us,
        TimingMethod::HostInstantOnly, // mutated
        baseline.layer,
        baseline.detector_count,
        baseline.catalog_count,
        baseline.input_bytes,
        baseline.evidence_bytes_read,
        baseline.evidence_bytes_written,
        baseline.witness_bytes_written,
        baseline.fusion_bytes_read_written,
        baseline.digest_bytes_read,
        baseline.candidate_summary_bytes,
        baseline.total_accounted_device_bytes,
        baseline.effective_bandwidth_gbps,
        baseline.percent_of_peak_basis_points,
        baseline.accounting_overflow_acknowledged,
        baseline.artifact_hashes.clone(),
        baseline.contract_hashes.clone(),
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

#[test]
fn receipt_hash_changes_when_layer_changes() {
    let baseline = seed_baseline_uninstrumented_receipt();
    let mutated = build_device_traffic_receipt(
        baseline.device_name,
        baseline.device_uuid_or_identity_hash,
        baseline.sm_arch,
        baseline.driver_version,
        baseline.cuda_version,
        baseline.theoretical_memory_bandwidth_gbps,
        baseline.measured_kernel_time_us,
        baseline.timing_method,
        DeviceBandwidthLayer::LayerC, // mutated
        baseline.detector_count,
        baseline.catalog_count,
        baseline.input_bytes,
        baseline.evidence_bytes_read,
        baseline.evidence_bytes_written,
        baseline.witness_bytes_written,
        baseline.fusion_bytes_read_written,
        baseline.digest_bytes_read,
        baseline.candidate_summary_bytes,
        baseline.total_accounted_device_bytes,
        baseline.effective_bandwidth_gbps,
        baseline.percent_of_peak_basis_points,
        baseline.accounting_overflow_acknowledged,
        baseline.artifact_hashes.clone(),
        baseline.contract_hashes.clone(),
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

#[test]
fn receipt_hash_changes_when_measured_kernel_time_changes() {
    let baseline = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::CudaEvent,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        0,
        0,
        false,
        one_contract_hash(),
    );
    let mutated = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::CudaEvent,
        716,
        2_000_000, // mutated
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        0,
        0,
        false,
        one_contract_hash(),
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

#[test]
fn receipt_hash_changes_when_accounting_overflow_flag_changes() {
    let baseline = seed_baseline_uninstrumented_receipt();
    let mutated = build_device_traffic_receipt(
        baseline.device_name,
        baseline.device_uuid_or_identity_hash,
        baseline.sm_arch,
        baseline.driver_version,
        baseline.cuda_version,
        baseline.theoretical_memory_bandwidth_gbps,
        baseline.measured_kernel_time_us,
        baseline.timing_method,
        baseline.layer,
        baseline.detector_count,
        baseline.catalog_count,
        baseline.input_bytes,
        baseline.evidence_bytes_read,
        baseline.evidence_bytes_written,
        baseline.witness_bytes_written,
        baseline.fusion_bytes_read_written,
        baseline.digest_bytes_read,
        baseline.candidate_summary_bytes,
        baseline.total_accounted_device_bytes,
        baseline.effective_bandwidth_gbps,
        baseline.percent_of_peak_basis_points,
        true, // mutated
        baseline.artifact_hashes.clone(),
        baseline.contract_hashes.clone(),
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

#[test]
fn receipt_hash_changes_when_contract_hashes_change() {
    let baseline = seed_baseline_uninstrumented_receipt();
    let mut mutated_hashes = baseline.contract_hashes.clone();
    mutated_hashes.push([0xAB; 32]);
    let mutated = build_device_traffic_receipt(
        baseline.device_name,
        baseline.device_uuid_or_identity_hash,
        baseline.sm_arch,
        baseline.driver_version,
        baseline.cuda_version,
        baseline.theoretical_memory_bandwidth_gbps,
        baseline.measured_kernel_time_us,
        baseline.timing_method,
        baseline.layer,
        baseline.detector_count,
        baseline.catalog_count,
        baseline.input_bytes,
        baseline.evidence_bytes_read,
        baseline.evidence_bytes_written,
        baseline.witness_bytes_written,
        baseline.fusion_bytes_read_written,
        baseline.digest_bytes_read,
        baseline.candidate_summary_bytes,
        baseline.total_accounted_device_bytes,
        baseline.effective_bandwidth_gbps,
        baseline.percent_of_peak_basis_points,
        baseline.accounting_overflow_acknowledged,
        baseline.artifact_hashes.clone(),
        mutated_hashes,
    );
    assert_ne!(
        baseline.device_traffic_receipt_hash_v1,
        mutated.device_traffic_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Rendering smoke tests
// ---------------------------------------------------------------

#[test]
fn receipt_text_contains_pinned_header_lines() {
    let s = render_device_traffic_receipt_text(&seed_baseline_uninstrumented_receipt());
    assert!(s.contains("S-PERF.1 DeviceTrafficReceiptV1"));
    assert!(s.contains("Device identity"));
    assert!(s.contains("Bandwidth posture"));
    assert!(s.contains("Workload"));
    assert!(s.contains("Byte accounting"));
    assert!(s.contains("Effective claim"));
    assert!(s.contains("Anchors"));
    assert!(s.contains("device_traffic_receipt_hash_v1"));
}

#[test]
fn receipt_json_contains_pinned_schema_id() {
    let s = render_device_traffic_receipt_json(&seed_baseline_uninstrumented_receipt());
    assert!(s.contains(DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1));
    assert!(s.contains("device_traffic_receipt_hash_v1"));
}

#[test]
fn policy_text_contains_eight_pinned_rules() {
    let s = render_bandwidth_claim_policy_text(&build_panel_locked_bandwidth_claim_policy());
    for line in S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES {
        assert!(s.contains(line), "policy text missing pinned line `{line}`");
    }
}

#[test]
fn policy_json_contains_pinned_schema_id() {
    let s = render_bandwidth_claim_policy_json(&build_panel_locked_bandwidth_claim_policy());
    assert!(s.contains(DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1));
    assert!(s.contains("device_bandwidth_claim_policy_hash_v1"));
    assert!(s.contains("policy_lines"));
}

#[test]
fn policy_text_records_panel_locked_eight_line_count() {
    let s = render_bandwidth_claim_policy_text(&build_panel_locked_bandwidth_claim_policy());
    assert!(s.contains("(8)"));
}

// ---------------------------------------------------------------
// Defensive admission cases: receipts that don't claim anything
// admit even with non-zero declared device bandwidth.
// ---------------------------------------------------------------

#[test]
fn receipt_with_no_bandwidth_claim_admits_even_with_declared_peak() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::CudaEvent,
        716,
        0, // no measured time
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.is_empty(),
        "all-zero claim with declared peak should admit: {errors:?}"
    );
}

#[test]
fn receipt_with_byte_accounting_and_modest_claim_admits() {
    let r = build_test_receipt(
        DeviceBandwidthLayer::LayerA,
        TimingMethod::CudaEvent,
        716,
        1_000_000,
        100,
        100,
        100,
        100,
        100,
        100,
        100,
        700,
        1,
        100, // 1.00 % of peak
        false,
        one_contract_hash(),
    );
    let errors = verify_device_traffic_receipt(&r);
    assert!(
        errors.is_empty(),
        "modest backed claim should admit: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Receipt struct fields are populated (no zero-init slip-through)
// ---------------------------------------------------------------

#[test]
fn baseline_receipt_has_non_zero_receipt_hash() {
    let r = seed_baseline_uninstrumented_receipt();
    assert_ne!(r.device_traffic_receipt_hash_v1, [0u8; 32]);
}

#[test]
fn policy_has_non_zero_policy_hash() {
    let p = build_panel_locked_bandwidth_claim_policy();
    assert_ne!(p.device_bandwidth_claim_policy_hash_v1, [0u8; 32]);
}

// ---------------------------------------------------------------
// TimingMethod::is_device_resident classifier
// ---------------------------------------------------------------

#[test]
fn timing_method_is_device_resident_classifier_is_correct() {
    assert!(TimingMethod::CudaEvent.is_device_resident());
    assert!(TimingMethod::CudaStreamSync.is_device_resident());
    assert!(!TimingMethod::HostInstantOnly.is_device_resident());
    assert!(!TimingMethod::HostJsonInclusiveTime.is_device_resident());
    assert!(!TimingMethod::Unknown.is_device_resident());
}

#[test]
fn timing_method_wire_names_are_stable() {
    assert_eq!(TimingMethod::CudaEvent.as_str(), "CudaEvent");
    assert_eq!(TimingMethod::CudaStreamSync.as_str(), "CudaStreamSync");
    assert_eq!(TimingMethod::HostInstantOnly.as_str(), "HostInstantOnly");
    assert_eq!(
        TimingMethod::HostJsonInclusiveTime.as_str(),
        "HostJsonInclusiveTime"
    );
    assert_eq!(TimingMethod::Unknown.as_str(), "Unknown");
}

#[test]
fn layer_wire_names_are_stable() {
    assert_eq!(DeviceBandwidthLayer::LayerA.as_str(), "LayerA");
    assert_eq!(DeviceBandwidthLayer::LayerB.as_str(), "LayerB");
    assert_eq!(DeviceBandwidthLayer::LayerC.as_str(), "LayerC");
}
