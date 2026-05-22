//! S-PERF.7 acceptance suite for the source-report import
//! verifier.
//!
//! 4 panel-required load-bearing negatives (verbatim from
//! the directive):
//!
//!  1. `rejects_receipt_if_source_report_bandwidth_differs`
//!  2. `rejects_receipt_if_source_report_device_total_differs`
//!  3. `rejects_receipt_if_source_report_host_segment_differs`
//!  4. `rejects_receipt_if_r12b_episode_pins_differ`
//!
//! Plus structural defect tests, parser tests, hash
//! determinism, sensitivity, renderer byte-stability, and a
//! pinned-hash back-stop against silent rebaselining.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, seed_rtx4080_super_measured_cuda_pipeline,
    R12B_EPISODE_COUNT_CANONICAL_W16H128, R12B_EPISODE_COUNT_FULL_W256H4096,
    R12B_EPISODE_COUNT_MID_W64H512,
};
use dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::{
    build_source_report_import_verifier_report, parse_d64_stage_timing, parse_r12b_d64_saturation,
    render_source_report_import_verifier_report_json,
    render_source_report_import_verifier_report_text,
    seed_source_report_import_verifier_report_from_disk,
    verify_source_reports_match_s_perf_6_baseline, ParseError, ParsedD64StageTimingV1,
    ParsedR12bSaturationV1, SPerf7VerifyErrorKind, S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
};

// ---------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

/// Build a `ParsedD64StageTimingV1` matching the panel-pinned
/// S-PERF.6 receipt values so tests can mutate ONE field to
/// exercise individual verifier rules without disk I/O.
fn parsed_d64_matching_receipt() -> ParsedD64StageTimingV1 {
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    ParsedD64StageTimingV1 {
        host_wall_median_us: m.host_wall_median_us,
        device_total_us: m.device_total_us,
        consensus_grid_kernel_wide_us: m.consensus_grid_kernel_wide_us,
        tree_digest_consensus_us: m.tree_digest_consensus_us,
        host_compute_features_us: m.host_compute_features_us,
        host_bank_admit_case_finalize_us: m.host_bank_admit_case_finalize_us,
        measured_wide_bandwidth_centi_gbps: m.measured_wide_bandwidth_centi_gbps,
        episode_count_full_256x4096: R12B_EPISODE_COUNT_FULL_W256H4096,
    }
}

fn parsed_r12b_matching_pins() -> ParsedR12bSaturationV1 {
    ParsedR12bSaturationV1 {
        episode_count_canonical_w16h128: R12B_EPISODE_COUNT_CANONICAL_W16H128,
        episode_count_mid_w64h512: R12B_EPISODE_COUNT_MID_W64H512,
        episode_count_full_w256h4096: R12B_EPISODE_COUNT_FULL_W256H4096,
    }
}

fn build_report(
    d64: ParsedD64StageTimingV1,
    r12b: ParsedR12bSaturationV1,
) -> dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::SourceReportImportVerifierReportV1
{
    let baseline = seed_rtx4080_super_measured_baseline_report();
    build_source_report_import_verifier_report(
        "test_verifier_v1",
        "reports/d64_stage_timing_256x4096_K1.txt",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        d64,
        r12b,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
    )
}

fn has_kind(
    errors: &[dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::SPerf7VerifyError],
    pred: impl Fn(&SPerf7VerifyErrorKind) -> bool,
) -> bool {
    errors.iter().any(|e| pred(&e.kind))
}

// ---------------------------------------------------------------
// Positive: live disk seed admits
// ---------------------------------------------------------------

#[test]
fn live_disk_seed_admits_against_s_perf_6_baseline() {
    let report =
        seed_source_report_import_verifier_report_from_disk(&repo_root()).expect("seed must read");
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &baseline.measurement);
    assert!(
        errors.is_empty(),
        "live disk source reports must match S-PERF.6 baseline; drift: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #1
// ---------------------------------------------------------------

#[test]
fn s_perf_7_rejects_receipt_if_source_report_bandwidth_differs() {
    let mut d64 = parsed_d64_matching_receipt();
    d64.measured_wide_bandwidth_centi_gbps = 2_000; // 20.00 GB/s, mutated
    let report = build_report(d64, parsed_r12b_matching_pins());
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::SourceReportBandwidthDiffers {
                source_centi_gbps: 2_000,
                ..
            }
        )),
        "must fire SourceReportBandwidthDiffers; got {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #2
// ---------------------------------------------------------------

#[test]
fn s_perf_7_rejects_receipt_if_source_report_device_total_differs() {
    let mut d64 = parsed_d64_matching_receipt();
    d64.device_total_us = 99_999; // mutated
    let report = build_report(d64, parsed_r12b_matching_pins());
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::SourceReportDeviceTotalDiffers {
                source_us: 99_999,
                ..
            }
        )),
        "must fire SourceReportDeviceTotalDiffers; got {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #3 (fires on either host segment)
// ---------------------------------------------------------------

#[test]
fn s_perf_7_rejects_receipt_if_source_report_host_segment_differs_compute_features() {
    let mut d64 = parsed_d64_matching_receipt();
    d64.host_compute_features_us = 12_345; // mutated
    let report = build_report(d64, parsed_r12b_matching_pins());
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::SourceReportHostSegmentDiffers {
                which: "host_compute_features_us",
                source_us: 12_345,
                ..
            }
        )),
        "must fire SourceReportHostSegmentDiffers on compute_features; got {errors:?}"
    );
}

#[test]
fn s_perf_7_rejects_receipt_if_source_report_host_segment_differs_bank_admit() {
    let mut d64 = parsed_d64_matching_receipt();
    d64.host_bank_admit_case_finalize_us = 9_999; // mutated
    let report = build_report(d64, parsed_r12b_matching_pins());
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::SourceReportHostSegmentDiffers {
                which: "host_bank_admit_case_finalize_us",
                source_us: 9_999,
                ..
            }
        )),
        "must fire SourceReportHostSegmentDiffers on bank-admit; got {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required negative #4 (fires on any of three pins)
// ---------------------------------------------------------------

#[test]
fn s_perf_7_rejects_receipt_if_r12b_episode_pins_differ_canonical() {
    let mut r12b = parsed_r12b_matching_pins();
    r12b.episode_count_canonical_w16h128 = 99; // mutated
    let report = build_report(parsed_d64_matching_receipt(), r12b);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_canonical_w16h128",
                source_count: 99,
                panel_locked: 13,
            }
        )),
        "must fire R12bEpisodePinsDiffer on canonical; got {errors:?}"
    );
}

#[test]
fn s_perf_7_rejects_receipt_if_r12b_episode_pins_differ_mid() {
    let mut r12b = parsed_r12b_matching_pins();
    r12b.episode_count_mid_w64h512 = 88;
    let report = build_report(parsed_d64_matching_receipt(), r12b);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_mid_w64h512",
                source_count: 88,
                panel_locked: 89,
            }
        )),
        "must fire R12bEpisodePinsDiffer on mid; got {errors:?}"
    );
}

#[test]
fn s_perf_7_rejects_receipt_if_r12b_episode_pins_differ_full() {
    let mut r12b = parsed_r12b_matching_pins();
    r12b.episode_count_full_w256h4096 = 1_900;
    // Keep the d64 episode_count consistent with the (mutated)
    // r12b pin so we only fire the R.12b pin negative, not the
    // cross-report inconsistency one.
    let mut d64 = parsed_d64_matching_receipt();
    d64.episode_count_full_256x4096 = 1_900;
    let report = build_report(d64, r12b);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_full_w256h4096",
                source_count: 1_900,
                panel_locked: 1_917,
            }
        )),
        "must fire R12bEpisodePinsDiffer on full; got {errors:?}"
    );
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn s_perf_7_rejects_empty_verifier_id() {
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let report = build_source_report_import_verifier_report(
        "",
        "reports/d64_stage_timing_256x4096_K1.txt",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        parsed_d64_matching_receipt(),
        parsed_r12b_matching_pins(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
    );
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf7VerifyErrorKind::VerifierIdEmpty
    )));
}

#[test]
fn s_perf_7_rejects_cross_report_episode_count_inconsistency() {
    let mut d64 = parsed_d64_matching_receipt();
    let mut r12b = parsed_r12b_matching_pins();
    // Mutate BOTH so the panel-locked R.12b pins still pass
    // but the cross-report consistency fails.
    d64.episode_count_full_256x4096 = 1_000;
    r12b.episode_count_full_w256h4096 = 2_000;
    let report = build_report(d64, r12b);
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf7VerifyErrorKind::CrossReportEpisodeCountInconsistent {
                d64_source_count: 1_000,
                r12b_source_count: 2_000,
            }
        )),
        "must fire CrossReportEpisodeCountInconsistent; got {errors:?}"
    );
}

#[test]
fn s_perf_7_rejects_tree_digest_drift() {
    let mut d64 = parsed_d64_matching_receipt();
    d64.tree_digest_consensus_us = 999;
    let report = build_report(d64, parsed_r12b_matching_pins());
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &m);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf7VerifyErrorKind::SourceReportTreeDigestConsensusDiffers { source_us: 999, .. }
    )));
}

// ---------------------------------------------------------------
// Parser tests
// ---------------------------------------------------------------

#[test]
fn parser_reads_live_d64_source_report() {
    let path = repo_root().join("reports/d64_stage_timing_256x4096_K1.txt");
    let text = std::fs::read_to_string(&path).expect("d64 source report must exist");
    let parsed = parse_d64_stage_timing(&text).expect("d64 parser must accept live report");
    let m = seed_rtx4080_super_measured_cuda_pipeline();
    assert_eq!(parsed.host_wall_median_us, m.host_wall_median_us);
    assert_eq!(parsed.device_total_us, m.device_total_us);
    assert_eq!(
        parsed.consensus_grid_kernel_wide_us,
        m.consensus_grid_kernel_wide_us
    );
    assert_eq!(parsed.tree_digest_consensus_us, m.tree_digest_consensus_us);
    assert_eq!(parsed.host_compute_features_us, m.host_compute_features_us);
    assert_eq!(
        parsed.host_bank_admit_case_finalize_us,
        m.host_bank_admit_case_finalize_us
    );
    assert_eq!(
        parsed.measured_wide_bandwidth_centi_gbps,
        m.measured_wide_bandwidth_centi_gbps
    );
    assert_eq!(
        parsed.episode_count_full_256x4096,
        R12B_EPISODE_COUNT_FULL_W256H4096
    );
}

#[test]
fn parser_reads_live_r12b_source_report() {
    let path = repo_root().join("reports/r12_d64_saturation.txt");
    let text = std::fs::read_to_string(&path).expect("R.12b source report must exist");
    let parsed = parse_r12b_d64_saturation(&text).expect("R.12b parser must accept live report");
    assert_eq!(
        parsed.episode_count_canonical_w16h128,
        R12B_EPISODE_COUNT_CANONICAL_W16H128
    );
    assert_eq!(
        parsed.episode_count_mid_w64h512,
        R12B_EPISODE_COUNT_MID_W64H512
    );
    assert_eq!(
        parsed.episode_count_full_w256h4096,
        R12B_EPISODE_COUNT_FULL_W256H4096
    );
}

#[test]
fn parser_rejects_d64_missing_bandwidth_line() {
    let text = "Host wall median (incl. host segments): 30020 us\n\
                Device total_device_us (median): 20771 us\n\
                consensus_grid_kernel_wide | 382 | 1.8\n\
                tree_digest consensus | 4338 | 20.9\n\
                host: compute_features | 7525 us\n\
                host: bank admit + case finalize | 2237 us\n\
                episode_count : 1917\n";
    let err = parse_d64_stage_timing(text).unwrap_err();
    assert!(matches!(err, ParseError::MissingWideBandwidth));
}

#[test]
fn parser_rejects_d64_with_unparseable_us() {
    let text = "Host wall median (incl. host segments): NOTANUMBER us\n\
                Device total_device_us (median): 20771 us\n\
                consensus_grid_kernel_wide | 382 | 1.8\n\
                tree_digest consensus | 4338 | 20.9\n\
                host: compute_features | 7525 us\n\
                host: bank admit + case finalize | 2237 us\n\
                wide bytes/sec (264) : 13.33 GB/s\n\
                episode_count : 1917\n";
    let err = parse_d64_stage_timing(text).unwrap_err();
    assert!(matches!(
        err,
        ParseError::MalformedNumber {
            field: "host_wall_median_us"
        }
    ));
}

#[test]
fn parser_rejects_bandwidth_with_one_decimal_digit() {
    // The grammar is "X.YY" with exactly two decimal digits;
    // "7.7" must be rejected so silent rounding cannot hide
    // a precision drop.
    let text = "Host wall median (incl. host segments): 30020 us\n\
                Device total_device_us (median): 20771 us\n\
                consensus_grid_kernel_wide | 382 | 1.8\n\
                tree_digest consensus | 4338 | 20.9\n\
                host: compute_features | 7525 us\n\
                host: bank admit + case finalize | 2237 us\n\
                wide bytes/sec (264) : 7.7 GB/s\n\
                episode_count : 1917\n";
    let err = parse_d64_stage_timing(text).unwrap_err();
    assert!(matches!(
        err,
        ParseError::MalformedNumber {
            field: "measured_wide_bandwidth_centi_gbps"
        }
    ));
}

#[test]
fn parser_rejects_r12b_missing_canonical_pin() {
    let text = "Detailed throughput:\n\
                mid 64x512             K=  1 : cells/sec=1.0e6  det_evals/sec=1.0e8  episodes/cat=89\n\
                full 256x4096          K=  1 : cells/sec=1.0e6  det_evals/sec=1.0e8  episodes/cat=1917\n";
    let err = parse_r12b_d64_saturation(text).unwrap_err();
    assert!(matches!(err, ParseError::MissingEpisodesCanonicalW16H128));
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn verifier_hash_is_deterministic_across_two_builds() {
    let a = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    let b = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    assert_eq!(
        a.source_report_import_verifier_hash_v1,
        b.source_report_import_verifier_hash_v1
    );
}

#[test]
fn verifier_hash_changes_when_bandwidth_changes() {
    let a = build_report(parsed_d64_matching_receipt(), parsed_r12b_matching_pins());
    let mut d64 = parsed_d64_matching_receipt();
    d64.measured_wide_bandwidth_centi_gbps = 1_500;
    let b = build_report(d64, parsed_r12b_matching_pins());
    assert_ne!(
        a.source_report_import_verifier_hash_v1,
        b.source_report_import_verifier_hash_v1
    );
}

#[test]
fn verifier_hash_changes_when_r12b_pin_changes() {
    let a = build_report(parsed_d64_matching_receipt(), parsed_r12b_matching_pins());
    let mut r12b = parsed_r12b_matching_pins();
    r12b.episode_count_mid_w64h512 = 90;
    let b = build_report(parsed_d64_matching_receipt(), r12b);
    assert_ne!(
        a.source_report_import_verifier_hash_v1,
        b.source_report_import_verifier_hash_v1
    );
}

#[test]
fn verifier_hash_changes_when_verifier_id_changes() {
    let a = build_report(parsed_d64_matching_receipt(), parsed_r12b_matching_pins());
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let b = build_source_report_import_verifier_report(
        "different_verifier_v1",
        "reports/d64_stage_timing_256x4096_K1.txt",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        parsed_d64_matching_receipt(),
        parsed_r12b_matching_pins(),
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
    );
    assert_ne!(
        a.source_report_import_verifier_hash_v1,
        b.source_report_import_verifier_hash_v1
    );
}

#[test]
fn verifier_hash_changes_when_upstream_s_perf_6_baseline_hash_changes() {
    let a = build_report(parsed_d64_matching_receipt(), parsed_r12b_matching_pins());
    let mut anchor = a.s_perf_6_baseline_report_hash;
    anchor[0] ^= 0xff;
    let b = build_source_report_import_verifier_report(
        "test_verifier_v1",
        "reports/d64_stage_timing_256x4096_K1.txt",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        parsed_d64_matching_receipt(),
        parsed_r12b_matching_pins(),
        anchor,
    );
    assert_ne!(
        a.source_report_import_verifier_hash_v1,
        b.source_report_import_verifier_hash_v1
    );
}

// ---------------------------------------------------------------
// Distinctness from prior anchors
// ---------------------------------------------------------------

#[test]
fn verifier_hash_distinct_from_s_perf_6_anchors() {
    let r = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let h_self = r.source_report_import_verifier_hash_v1;
    assert_ne!(
        h_self,
        baseline.rtx4080_super_measured_baseline_report_hash_v1
    );
    assert_ne!(
        h_self,
        baseline
            .measurement
            .rtx4080_super_measured_cuda_pipeline_hash_v1
    );
    assert_ne!(
        h_self,
        baseline
            .claim
            .rtx4080_super_measured_bandwidth_claim_hash_v1
    );
}

// ---------------------------------------------------------------
// Renderer byte-stability
// ---------------------------------------------------------------

#[test]
fn renderers_are_byte_stable_across_two_calls() {
    let r = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    assert_eq!(
        render_source_report_import_verifier_report_text(&r),
        render_source_report_import_verifier_report_text(&r)
    );
    assert_eq!(
        render_source_report_import_verifier_report_json(&r),
        render_source_report_import_verifier_report_json(&r)
    );
}

#[test]
fn text_renderer_contains_verifier_id_and_provenance() {
    let r = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    let text = render_source_report_import_verifier_report_text(&r);
    assert!(text.contains("S-PERF.7"));
    assert!(text.contains("reports/d64_stage_timing_256x4096_K1.txt"));
    assert!(text.contains("reports/r12_d64_saturation.txt"));
    assert!(text.contains("source_report_import_verifier_hash_v1"));
}

// ---------------------------------------------------------------
// Pinned-hash back-stop
// ---------------------------------------------------------------

const PINNED_SOURCE_REPORT_IMPORT_VERIFIER_HASH_V1: [u8; 32] = [
    0x99, 0xcc, 0x8a, 0x71, 0xcd, 0xce, 0xb7, 0xdb, 0x7c, 0x37, 0x75, 0x4f, 0xc4, 0x94, 0xb5, 0x1f,
    0x33, 0xa7, 0xcc, 0xef, 0xee, 0xab, 0x95, 0x17, 0x50, 0x95, 0xbe, 0xe7, 0x38, 0x83, 0xd5, 0xc9,
];

#[test]
fn pinned_verifier_hash_matches_live_disk_seed() {
    let r = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    assert_eq!(
        r.source_report_import_verifier_hash_v1, PINNED_SOURCE_REPORT_IMPORT_VERIFIER_HASH_V1,
        "verifier hash drifted; refresh the pinned constant if the bench source report changed"
    );
}
