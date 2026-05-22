//! S-PERF.8 acceptance suite (post-S-PERF.8.1 hardening).
//!
//! Test groups (per panel directive):
//!
//!   - parser tests
//!   - verifier negatives (panel-required + structural)
//!   - measured-shape pins (canonical 1.76x, full +3.4%)
//!   - determinism tests
//!   - sensitivity tests
//!   - renderer byte-stability
//!   - upstream-anchor invariance
//!   - R.12b episode pin coverage
//!
//! The CAMPAIGN IDENTITY negative is
//! `s_perf_8_rejects_canonical_launch_bound_gain_generalized_to_full_scale`
//! because the dangerous overclaim is
//! *"16x128 got 1.76x, therefore K batching solved full-scale."*
//! The court refuses.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, seed_rtx4080_super_measured_cuda_pipeline,
};
use dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::{
    seed_source_report_import_verifier_report_from_disk,
    S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
};
use dsfb_gpu_atlas_corpus::s_perf_8_batched_k_saturation_receipt::{
    build_batched_k_saturation_receipt, parse_batched_k_saturation_table,
    render_batched_k_saturation_receipt_json, render_batched_k_saturation_receipt_text,
    seed_batched_k_saturation_receipt_from_disk, summarise_per_scale,
    verify_batched_k_saturation_receipt, BatchedKResultInterpretation, BatchedKSaturationReceiptV1,
    ParseError, ParsedBatchedKCellV1, ParsedScaleSummaryV1, SPerf8VerifyErrorKind,
    S_PERF_8_CATALOG_ORDER_LABEL, S_PERF_8_CUDA_GRAPH_STATUS_LABEL, S_PERF_8_DISPATCH_MODE_LABEL,
    S_PERF_8_MERGE_POLICY_LABEL, S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
    S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096, S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn has_kind(
    errors: &[dsfb_gpu_atlas_corpus::s_perf_8_batched_k_saturation_receipt::SPerf8VerifyError],
    pred: impl Fn(&SPerf8VerifyErrorKind) -> bool,
) -> bool {
    errors.iter().any(|e| pred(&e.kind))
}

fn build_test_receipt_from_live(
    cells: Vec<ParsedBatchedKCellV1>,
    per_scale: Vec<ParsedScaleSummaryV1>,
) -> BatchedKSaturationReceiptV1 {
    let baseline = seed_rtx4080_super_measured_baseline_report();
    build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        cells,
        per_scale,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    )
}

fn live_pre_bandwidth_centi_gbps() -> u32 {
    seed_rtx4080_super_measured_cuda_pipeline().measured_wide_bandwidth_centi_gbps
}

// ---------------------------------------------------------------
// Parser tests
// ---------------------------------------------------------------

#[test]
fn parses_live_batched_k_report_from_disk() {
    let path = repo_root().join("reports/r12_d64_saturation.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let cells = parse_batched_k_saturation_table(&text).expect("parser must accept live report");
    assert_eq!(cells.len(), 18, "must parse 18 (scale, K) cells");
}

#[test]
fn parser_rejects_table_with_missing_header() {
    let err = parse_batched_k_saturation_table("nothing useful here").unwrap_err();
    assert!(matches!(err, ParseError::MissingHeader));
}

#[test]
fn parser_summary_per_scale_picks_best_k() {
    let path = repo_root().join("reports/r12_d64_saturation.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let cells = parse_batched_k_saturation_table(&text).unwrap();
    let pre = live_pre_bandwidth_centi_gbps();
    let summaries = summarise_per_scale(&cells, pre);
    assert_eq!(summaries.len(), 3);
    let canonical = summaries
        .iter()
        .find(|s| s.scale_label == "canonical 16x128")
        .unwrap();
    assert!(canonical.best_k == 64 || canonical.best_k == 32);
}

// ---------------------------------------------------------------
// Measured-shape pins (THE empirical conclusions)
// ---------------------------------------------------------------

#[test]
fn parses_canonical_16x128_speedup_as_launch_bound_gain() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let canonical = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "canonical 16x128")
        .unwrap();
    assert_eq!(
        canonical.interpretation,
        BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture,
        "canonical 16x128 must classify as LaunchBoundGainAtSmallFixture; got {:?}",
        canonical.interpretation
    );
}

#[test]
fn parses_full_256x4096_speedup_as_modest_gain() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let full = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "full 256x4096")
        .unwrap();
    assert_eq!(
        full.interpretation,
        BatchedKResultInterpretation::ModestFullScaleGain,
        "full 256x4096 must classify as ModestFullScaleGain; got {:?}",
        full.interpretation
    );
}

#[test]
fn full_scale_batched_k_delta_is_panel_pinned_approximately_3_4_percent() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let full = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "full 256x4096")
        .unwrap();
    // Panel-pinned band: full-scale delta is between
    // +0% and +10% (+0 bp to +1000 bp). Honest measurement is
    // ~+3.4%; the band leaves room for thermal/system load
    // variance without re-baselining each commit.
    assert!(
        full.delta_basis_points >= 0 && full.delta_basis_points <= 1000,
        "full-scale delta must be in [+0%, +10%] band; got {} bp",
        full.delta_basis_points
    );
}

#[test]
fn canonical_batched_k_speedup_is_panel_pinned_approximately_1_76x() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let canonical = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "canonical 16x128")
        .unwrap();
    // Panel-pinned band: canonical gain is between 1.5x
    // and 2.0x (15000 bp to 20000 bp). Honest measurement
    // is ~1.76x; the band leaves room for variance.
    assert!(
        (15_000..=20_000).contains(&canonical.best_k_gain_basis_points),
        "canonical gain must be in [1.5x, 2.0x] band; got {} bp",
        canonical.best_k_gain_basis_points
    );
}

#[test]
fn verifier_admits_live_batched_k_report() {
    let receipt = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(
        errors.is_empty(),
        "live disk K-saturation receipt must admit; drift: {errors:?}"
    );
}

#[test]
fn r12b_episode_pins_remain_13_89_1917() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_eq!(r.r12b_episode_count_canonical_w16h128, 13);
    assert_eq!(r.r12b_episode_count_mid_w64h512, 89);
    assert_eq!(r.r12b_episode_count_full_w256h4096, 1917);
}

// ---------------------------------------------------------------
// Panel-required negatives (S-PERF.8 + S-PERF.8.1)
// ---------------------------------------------------------------

#[test]
fn s_perf_8_rejects_host_loop_k_claimed_as_batched() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        "true batched K kernel with single launch", // <-- dishonest
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::HostLoopKClaimedAsBatched { .. }
    )));
}

#[test]
fn s_perf_8_rejects_missing_batched_k_source_report() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        "", // <-- empty path
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::MissingBatchedKSourceReport
    )));
}

#[test]
fn s_perf_8_rejects_missing_pre_post_bandwidth_delta() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            s.pre_bandwidth_centi_gbps = 0;
            s.post_bandwidth_centi_gbps = 0;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::MissingPrePostBandwidthDelta { .. }
    )));
}

#[test]
fn s_perf_8_rejects_full_scale_claim_above_measured_delta() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            // Forge a label that overstates the measured delta.
            // Real delta is ~+3.4% which is `ModestFullScaleGain`;
            // overclaim as `LaunchBoundGainAtSmallFixture` (which
            // requires >= +50% / 5000 bp). This ALSO triggers
            // the campaign-identity negative; assert the
            // overclaim-above-measured negative fires too.
            s.interpretation = BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::FullScaleClaimAboveMeasuredDelta { .. }
    )));
}

#[test]
fn s_perf_8_rejects_claim_that_full_scale_reached_25gbps_if_it_did_not() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            // Forge post bandwidth to 25.00 GB/s without the
            // measured gain to back it up. The live K=1
            // pre-bandwidth is ~1333 centi-GB/s; required gain
            // to reach 2500 centi-GB/s = 25 GB/s is
            // ~18750 bp; the live K-table best_k_gain is
            // ~10300 bp. The 25 GB/s claim is unbacked.
            s.post_bandwidth_centi_gbps = 2_500;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::ClaimFullScaleReached25GbpsWithoutMeasurement { .. }
    )));
}

#[test]
fn s_perf_8_rejects_saturation_claim_below_8000bp() {
    // Forge a full-scale row whose declared post bandwidth
    // computes to >= 8000 bp percent-of-peak (i.e. claims
    // saturation) while the gain-implied actual is
    // unchanged.
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            // 80% of 716 = 572.8 GB/s = 57280 centi-GB/s
            s.post_bandwidth_centi_gbps = 57_500;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::SaturationClaimBelow8000Bp { .. }
    )));
}

// CAMPAIGN IDENTITY negative
#[test]
fn s_perf_8_rejects_canonical_launch_bound_gain_generalized_to_full_scale() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            // The forbidden generalisation: applying
            // canonical's `LaunchBoundGainAtSmallFixture`
            // label to full-scale.
            s.interpretation = BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf8VerifyErrorKind::CanonicalLaunchBoundGainGeneralizedToFullScale { .. }
        )),
        "must fire the CAMPAIGN IDENTITY negative; got {errors:?}"
    );
}

#[test]
fn s_perf_8_rejects_episode_count_drift() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        14, // <-- drift
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::R12bEpisodePinsDrift {
            which: "r12b_episode_count_canonical_w16h128",
            declared: 14,
            ..
        }
    )));
}

#[test]
fn s_perf_8_rejects_catalog_order_drift() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        "randomised dispatch order", // <-- drift
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::CatalogOrderDrift { .. }
    )));
}

#[test]
fn s_perf_8_rejects_completion_order_merge() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        "completion-order merge (forbidden)", // <-- forbidden
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::CompletionOrderMergeRejected { .. }
    )));
}

#[test]
fn s_perf_8_rejects_cuda_graph_claim_without_replay_contract() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        "CUDA Graph capture engaged for K iterations", // <-- claim without replay contract
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::CudaGraphClaimWithoutReplayContract { .. }
    )));
}

#[test]
fn s_perf_8_rejects_missing_device_identity() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        [0u8; 32], // <-- missing device identity
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::MissingDeviceIdentity
    )));
}

#[test]
fn s_perf_8_rejects_percent_of_peak_arithmetic_mismatch() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            // Forge post-bandwidth to a value that doesn't
            // match pre * gain. pre*gain/10000 ≈ pre * 1.03
            // = 1373; forge to 9999 to force mismatch.
            s.post_bandwidth_centi_gbps = 9_999;
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::PercentOfPeakArithmeticMismatch { .. }
    )));
}

#[test]
fn s_perf_8_rejects_speedup_arithmetic_mismatch() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = live.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "canonical 16x128" {
            // Forge gain_bp away from best/k1 * 10000.
            s.best_k_gain_basis_points = s.best_k_gain_basis_points.saturating_add(500);
        }
    }
    let receipt = build_test_receipt_from_live(live.cells.clone(), summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::SpeedupArithmeticMismatch { .. }
    )));
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn report_requires_canonical_fixture_row() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut cells = live.cells.clone();
    cells.retain(|c| c.scale_label != "canonical 16x128");
    let pre = live_pre_bandwidth_centi_gbps();
    let summaries = summarise_per_scale(&cells, pre);
    let receipt = build_test_receipt_from_live(cells, summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::KMatrixIncomplete { .. }
    )));
}

#[test]
fn report_requires_full_scale_fixture_row() {
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut cells = live.cells.clone();
    cells.retain(|c| c.scale_label != "full 256x4096");
    let pre = live_pre_bandwidth_centi_gbps();
    let summaries = summarise_per_scale(&cells, pre);
    let receipt = build_test_receipt_from_live(cells, summaries);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::KMatrixIncomplete { .. }
    )));
}

#[test]
fn report_requires_pre_bandwidth_gbps() {
    // Full-scale pre-bandwidth must be non-zero in the
    // panel-pinned receipt (it comes from S-PERF.6).
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let full = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "full 256x4096")
        .unwrap();
    assert!(full.pre_bandwidth_centi_gbps > 0);
}

#[test]
fn report_requires_post_bandwidth_gbps() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let full = r
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "full 256x4096")
        .unwrap();
    assert!(full.post_bandwidth_centi_gbps > 0);
}

#[test]
fn report_requires_speedup_x() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    for s in &r.per_scale_summaries {
        assert!(s.best_k_gain_basis_points > 0);
    }
}

#[test]
fn report_requires_delta_percent() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    for s in &r.per_scale_summaries {
        // delta_bp is gain_bp - 10000; gain_bp must be > 0
        // so delta_bp is finite and meaningful.
        assert!(s.delta_basis_points >= -10_000);
    }
}

#[test]
fn report_requires_interpretation_label() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    for s in &r.per_scale_summaries {
        assert!(!s.interpretation.as_str().is_empty());
    }
}

#[test]
fn report_rejects_empty_source_report_path() {
    // Same as `s_perf_8_rejects_missing_batched_k_source_report`
    // but framed structurally.
    let live = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let receipt = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        "",
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        live.cells.clone(),
        live.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_batched_k_saturation_receipt(&receipt, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf8VerifyErrorKind::MissingBatchedKSourceReport
    )));
}

// ---------------------------------------------------------------
// Sensitivity tests (hash changes when any input changes)
// ---------------------------------------------------------------

#[test]
fn changing_full_scale_post_bandwidth_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = a.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            s.post_bandwidth_centi_gbps = s.post_bandwidth_centi_gbps.saturating_add(1);
        }
    }
    let b = build_test_receipt_from_live(a.cells.clone(), summaries);
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn changing_canonical_speedup_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = a.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "canonical 16x128" {
            s.best_k_gain_basis_points = s.best_k_gain_basis_points.saturating_add(1);
        }
    }
    let b = build_test_receipt_from_live(a.cells.clone(), summaries);
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn changing_delta_percent_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = a.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "full 256x4096" {
            s.delta_basis_points = s.delta_basis_points.saturating_add(1);
        }
    }
    let b = build_test_receipt_from_live(a.cells.clone(), summaries);
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn changing_interpretation_label_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let mut summaries = a.per_scale_summaries.clone();
    for s in &mut summaries {
        if s.scale_label == "mid 64x512" {
            s.interpretation = BatchedKResultInterpretation::Regressed;
        }
    }
    let b = build_test_receipt_from_live(a.cells.clone(), summaries);
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn changing_source_report_path_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let b = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        "reports/some_other_path.txt",
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        a.cells.clone(),
        a.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn changing_device_identity_changes_hash() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let mut bogus_identity = baseline.measurement.device_uuid_or_identity_hash;
    bogus_identity[0] ^= 0xff;
    let b = build_batched_k_saturation_receipt(
        "test_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        bogus_identity,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        a.cells.clone(),
        a.per_scale_summaries.clone(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        [0xaa; 32],
    );
    assert_ne!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Determinism + renderer + upstream-anchor + pinned-hash
// ---------------------------------------------------------------

#[test]
fn bandwidth_delta_report_is_deterministic_across_two_builds() {
    let a = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let b = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_eq!(
        a.batched_k_saturation_receipt_hash_v1,
        b.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn summary_renderer_is_byte_stable() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_eq!(
        render_batched_k_saturation_receipt_text(&r),
        render_batched_k_saturation_receipt_text(&r)
    );
}

#[test]
fn json_renderer_is_byte_stable() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_eq!(
        render_batched_k_saturation_receipt_json(&r),
        render_batched_k_saturation_receipt_json(&r)
    );
}

#[test]
fn text_renderer_contains_panel_pinned_summary_prose() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let text = render_batched_k_saturation_receipt_text(&r);
    assert!(text.contains("S-PERF.8 replaces K-as-host-loop with batched-K execution"));
    assert!(text.contains("canonical 16x128 improves by"));
    // The renderer line-wraps the prose, so the literal substring
    // "full 256x4096 improves only" can be split across two lines.
    // Check the surviving fragments.
    assert!(text.contains("while full"));
    assert!(text.contains("256x4096 improves only +3.4% / 1.03x"));
    assert!(text.contains("not a"));
    assert!(text.contains("saturation claim"));
}

#[test]
fn receipt_hash_distinct_from_s_perf_6_and_s_perf_7_anchors() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let verifier = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    assert_ne!(
        r.batched_k_saturation_receipt_hash_v1,
        baseline.rtx4080_super_measured_baseline_report_hash_v1
    );
    assert_ne!(
        r.batched_k_saturation_receipt_hash_v1,
        verifier.source_report_import_verifier_hash_v1
    );
}

// Pinned-hash back-stop --- refreshed for the S-PERF.8.1
// schema upgrade.
const PINNED_BATCHED_K_SATURATION_RECEIPT_HASH_V1: [u8; 32] = [
    // S-PERF.8.1 hardening seal (live R.12b D64 saturation source report,
    // panel-pinned labels, full-scale pre-bandwidth anchored to S-PERF.6
    // measurement 1333 centi-GB/s, R.12b episodes 13/89/1917, RTX 4080
    // SUPER device identity).
    0x37, 0x21, 0x2c, 0x42, 0xb4, 0xfd, 0xf0, 0x60, 0x69, 0xc0, 0x19, 0xe3, 0x44, 0x74, 0xcf, 0x0d,
    0x66, 0x60, 0x84, 0x3b, 0xad, 0x7c, 0xd4, 0xa9, 0xcb, 0xc9, 0x20, 0x20, 0xa7, 0xb9, 0xf2, 0x01,
];

#[test]
fn pinned_receipt_hash_matches_live_disk_seed() {
    let r = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_eq!(
        r.batched_k_saturation_receipt_hash_v1, PINNED_BATCHED_K_SATURATION_RECEIPT_HASH_V1,
        "S-PERF.8 receipt hash drifted; refresh the pinned constant if the R.12b source report changed"
    );
}
