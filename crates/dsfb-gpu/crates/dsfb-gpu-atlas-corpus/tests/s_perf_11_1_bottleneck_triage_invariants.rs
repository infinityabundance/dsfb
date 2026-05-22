//! S-PERF.11.1 post-S-PERF.11 bottleneck triage acceptance
//! tests. Twenty-four tests pin the six panel-required
//! load-bearing negatives, the four structural defect rules,
//! the classifier behaviour against three synthetic profiles
//! (digest-dominant + host-compute-features-surfaced +
//! detector-motif-surfaced), determinism + sensitivity of the
//! hash chain, renderer byte-stability, distinctness from
//! every prior anchor, and the pinned-hash back-stop against
//! silent rebaselining.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::s_perf_11_1_post_rewrite_bottleneck_triage::{
    build_post_rewrite_bottleneck_triage_report, classify_dominant_stage,
    compute_post_rewrite_bottleneck_triage_hash, parse_post_rewrite_d64_stage_profile,
    recommend_next_strike, render_post_rewrite_bottleneck_triage_report_json,
    render_post_rewrite_bottleneck_triage_report_text,
    seed_post_rewrite_bottleneck_triage_report_from_disk,
    verify_post_rewrite_bottleneck_triage_report, BottleneckCategory, NextStrikeRecommendation,
    ParsedTriageProfile, SPerf11_1VerifyError, TriageStageTimingV1,
    S_PERF_11_1_DEVICE_STAGE_LABELS, S_PERF_11_1_R12B_EPISODES_CANONICAL,
    S_PERF_11_1_R12B_EPISODES_FULL, S_PERF_11_1_R12B_EPISODES_MID, S_PERF_11_1_SOURCE_REPORT_PATH,
    S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS, S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS,
};

/// Pinned top-level hash captured from the live disk seed at
/// S-PERF.11.1 seal time. Back-stop against silent rebaselining
/// of the triage receipt or its upstream S-PERF.11 anchor.
const PINNED_S_PERF_11_1_BOTTLENECK_TRIAGE_HASH_V1: [u8; 32] = [
    0x70, 0xdd, 0x96, 0x7b, 0x47, 0xbb, 0x08, 0x30, 0x24, 0x9f, 0x1e, 0x51, 0x22, 0x97, 0x5b, 0xf7,
    0xab, 0x53, 0xa7, 0x64, 0x73, 0xfa, 0x52, 0x41, 0x45, 0x1b, 0x01, 0xe9, 0x30, 0x14, 0xa4, 0xfa,
];

/// Pinned S-PERF.11 anchor hash (S-PERF.11 sealed at 3e67cb4).
const S_PERF_11_PINNED_ANCHOR: [u8; 32] = [
    0x1a, 0x27, 0x15, 0x4e, 0x33, 0x5c, 0x27, 0xdf, 0x6d, 0xb9, 0x39, 0xd4, 0xc8, 0xff, 0x0f, 0x36,
    0xf8, 0xba, 0xf7, 0x58, 0x71, 0xbe, 0x06, 0xc7, 0x50, 0xf0, 0x85, 0x3f, 0x22, 0x68, 0xad, 0xc8,
];

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        )
}

fn fixture_stages(values: [u64; 12]) -> [TriageStageTimingV1; 12] {
    let mut out = [TriageStageTimingV1 {
        stage_label: "",
        us: 0,
    }; 12];
    for (idx, label) in S_PERF_11_1_DEVICE_STAGE_LABELS.iter().enumerate() {
        out[idx] = TriageStageTimingV1 {
            stage_label: label,
            us: values[idx],
        };
    }
    out
}

/// Digest-dominant fixture: 4 digest stages sum to 15334 (the
/// live triage observation); other stages small.
fn digest_dominant_profile() -> ParsedTriageProfile {
    // [h2d, residual_field, drift_slew_sign, detector_motif,
    //  consensus, axis5, candidate_collapse,
    //  tree_digest_residual, _sign, _detector, _consensus, d2h]
    let stages = fixture_stages([
        2520, 549, 2281, 6077, 657, 83, 1712, 3208, 3650, 3426, 5050, 25,
    ]);
    ParsedTriageProfile {
        device_stages: stages,
        host_compute_features_us: 7821,
        host_bank_admit_case_finalize_us: 2262,
        host_wall_median_us: 40418,
        device_total_us: 30283,
        triage_run_bandwidth_centi_gbps: 914,
    }
}

/// Host-compute-features-surfaced fixture: tiny digest +
/// big host compute_features.
fn host_compute_features_surfaced_profile() -> ParsedTriageProfile {
    let stages = fixture_stages([500, 500, 500, 1000, 500, 100, 500, 200, 200, 200, 200, 50]);
    ParsedTriageProfile {
        device_stages: stages,
        host_compute_features_us: 20000,
        host_bank_admit_case_finalize_us: 1000,
        host_wall_median_us: 25000,
        device_total_us: 4450,
        triage_run_bandwidth_centi_gbps: 1500,
    }
}

/// Detector-motif-surfaced fixture: digest small, detector
/// motif big.
fn detector_motif_surfaced_profile() -> ParsedTriageProfile {
    let stages = fixture_stages([500, 200, 500, 10000, 500, 100, 500, 200, 200, 200, 200, 50]);
    ParsedTriageProfile {
        device_stages: stages,
        host_compute_features_us: 1000,
        host_bank_admit_case_finalize_us: 500,
        host_wall_median_us: 14000,
        device_total_us: 13150,
        triage_run_bandwidth_centi_gbps: 1800,
    }
}

// ===============================================================
// Panel-required load-bearing negatives (6)
// ===============================================================

/// CAMPAIGN IDENTITY: triage receipt MUST cite the rebaselined
/// S-PERF.11 `s_perf_11_bandwidth_delta_report_hash_v1`; a zero
/// anchor is forbidden.
#[test]
fn s_perf_11_1_rejects_triage_without_post_s_perf_11_anchor() {
    let profile = digest_dominant_profile();
    let report = build_post_rewrite_bottleneck_triage_report("trial", profile, [0u8; 32]);
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::TriageWithoutPostSPerf11Anchor));
}

/// Reject when the report's claimed `bottleneck_category` does
/// not match what the classifier would produce against its own
/// parsed timings.
#[test]
fn s_perf_11_1_rejects_decision_without_dominant_stage_evidence() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.bottleneck_category = BottleneckCategory::HostComputeFeaturesSurfaced;
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::DecisionWithoutDominantStageEvidence));
}

/// Reject when the recommendation does not match the panel-
/// locked decision tree applied to the (claimed) bottleneck
/// category.
#[test]
fn s_perf_11_1_rejects_decision_inconsistent_with_panel_locked_rule() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    // DigestStillDominant must map to SPerf12CompactDensorDigestV1;
    // force a mismatch.
    report.next_strike_recommendation = NextStrikeRecommendation::ReRankBeforeNextStrike;
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::DecisionInconsistentWithPanelLockedRule));
}

/// Reject when the R.12b episode pins drift from 13/89/1917.
#[test]
fn s_perf_11_1_rejects_triage_with_r12b_episode_drift() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.r12b_episode_count_full_w256h4096 = 1916;
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::TriageWithR12bEpisodeDrift));
}

/// Reject when the pinned post-S-PERF.11 bandwidth value drifts
/// from the panel-locked 1638 centi-GB/s (16.38 GB/s).
#[test]
fn s_perf_11_1_rejects_triage_with_pinned_post_s_perf_11_bandwidth_drift() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.post_s_perf_11_pinned_bandwidth_centi_gbps = 1639;
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::TriageWithPinnedPostSPerf11BandwidthDrift));
}

/// Reject when any label/note field contains a forbidden
/// bandwidth-claim phrase (case-insensitive scan).
#[test]
fn s_perf_11_1_rejects_triage_that_claims_bandwidth_improvement() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    // Smuggle a forbidden phrase into report_id (the panel-
    // required scanner reads the metadata fields).
    report.report_id = "triage improves bandwidth";
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::TriageThatClaimsBandwidthImprovement));
}

// ===============================================================
// Structural defect rules (4)
// ===============================================================

#[test]
fn structural_rejects_empty_report_id() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.report_id = "";
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::ReportIdEmpty));
}

#[test]
fn structural_rejects_empty_source_report_path() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.source_report_path = "";
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::SourceReportPathEmpty));
}

#[test]
fn structural_rejects_dominant_stage_pct_arithmetic_mismatch() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.dominant_stage_pct_basis_points_of_device_total += 7;
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.contains(&SPerf11_1VerifyError::DominantStagePctArithmeticMismatch));
}

#[test]
fn structural_rejects_per_stage_label_mismatch() {
    let profile = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", profile, S_PERF_11_PINNED_ANCHOR);
    report.device_stages[3].stage_label = "wrong_label_for_detector_motif";
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs
        .iter()
        .any(|e| matches!(e, SPerf11_1VerifyError::PerStageLabelMismatch { at: 3, .. })));
}

// ===============================================================
// Classifier behaviour against synthetic fixtures
// ===============================================================

#[test]
fn classifier_picks_digest_when_digest_aggregate_dominates() {
    let profile = digest_dominant_profile();
    let (cat, label, us) = classify_dominant_stage(&profile);
    assert_eq!(cat, BottleneckCategory::DigestStillDominant);
    assert_eq!(label, "tree_digest (4-stage aggregate)");
    assert_eq!(us, 3208 + 3650 + 3426 + 5050);
    assert_eq!(
        recommend_next_strike(cat),
        NextStrikeRecommendation::SPerf12CompactDensorDigestV1
    );
}

#[test]
fn classifier_picks_host_compute_features_when_it_surfaces() {
    let profile = host_compute_features_surfaced_profile();
    let (cat, _label, _us) = classify_dominant_stage(&profile);
    assert_eq!(cat, BottleneckCategory::HostComputeFeaturesSurfaced);
    assert_eq!(
        recommend_next_strike(cat),
        NextStrikeRecommendation::SPerf13DeviceSideFeatureConstruction
    );
}

#[test]
fn classifier_picks_detector_motif_when_it_surfaces() {
    let profile = detector_motif_surfaced_profile();
    let (cat, label, _us) = classify_dominant_stage(&profile);
    assert_eq!(cat, BottleneckCategory::DetectorMotifSurfaced);
    assert_eq!(label, "detector_motif_kernel_wide_d64");
    assert_eq!(
        recommend_next_strike(cat),
        NextStrikeRecommendation::ReRankBeforeNextStrike
    );
}

// ===============================================================
// Hash determinism + sensitivity
// ===============================================================

#[test]
fn hash_is_deterministic_across_two_builds() {
    let p = digest_dominant_profile();
    let a = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    let p2 = digest_dominant_profile();
    let b = build_post_rewrite_bottleneck_triage_report("trial", p2, S_PERF_11_PINNED_ANCHOR);
    assert_eq!(
        a.s_perf_11_1_bottleneck_triage_hash_v1,
        b.s_perf_11_1_bottleneck_triage_hash_v1
    );
}

#[test]
fn hash_changes_when_triage_run_bandwidth_changes() {
    let mut p = digest_dominant_profile();
    let a = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    p.triage_run_bandwidth_centi_gbps += 1;
    let b = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    assert_ne!(
        a.s_perf_11_1_bottleneck_triage_hash_v1,
        b.s_perf_11_1_bottleneck_triage_hash_v1
    );
}

#[test]
fn hash_changes_when_dominant_stage_us_changes() {
    let p = digest_dominant_profile();
    let mut report =
        build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    let original = report.s_perf_11_1_bottleneck_triage_hash_v1;
    report.dominant_stage_us += 1;
    let new_hash = compute_post_rewrite_bottleneck_triage_hash(&report);
    assert_ne!(original, new_hash);
}

#[test]
fn hash_changes_when_s_perf_11_anchor_changes() {
    let p = digest_dominant_profile();
    let a = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    let p2 = digest_dominant_profile();
    let mut altered = S_PERF_11_PINNED_ANCHOR;
    altered[0] ^= 0xff;
    let b = build_post_rewrite_bottleneck_triage_report("trial", p2, altered);
    assert_ne!(
        a.s_perf_11_1_bottleneck_triage_hash_v1,
        b.s_perf_11_1_bottleneck_triage_hash_v1
    );
}

// ===============================================================
// Renderer byte-stability (2)
// ===============================================================

#[test]
fn text_renderer_is_byte_stable() {
    let p = digest_dominant_profile();
    let r = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    let a = render_post_rewrite_bottleneck_triage_report_text(&r);
    let b = render_post_rewrite_bottleneck_triage_report_text(&r);
    assert_eq!(a, b);
    assert!(a.contains("DigestStillDominant"));
    assert!(a.contains("SPerf12CompactDensorDigestV1"));
}

#[test]
fn json_renderer_is_byte_stable() {
    let p = digest_dominant_profile();
    let r = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    let a = render_post_rewrite_bottleneck_triage_report_json(&r);
    let b = render_post_rewrite_bottleneck_triage_report_json(&r);
    assert_eq!(a, b);
    assert!(a.contains("\"bottleneck_category\":\"DigestStillDominant\""));
    assert!(a.contains("\"next_strike_recommendation\":\"SPerf12CompactDensorDigestV1\""));
}

// ===============================================================
// Cross-anchor distinctness + pinned-hash back-stop
// ===============================================================

#[test]
fn triage_hash_is_distinct_from_s_perf_11_anchor() {
    let p = digest_dominant_profile();
    let r = build_post_rewrite_bottleneck_triage_report("trial", p, S_PERF_11_PINNED_ANCHOR);
    assert_ne!(
        r.s_perf_11_1_bottleneck_triage_hash_v1,
        r.s_perf_11_bandwidth_delta_report_hash_v1
    );
}

#[test]
fn pinned_triage_hash_matches_live_disk_seed() {
    let report =
        seed_post_rewrite_bottleneck_triage_report_from_disk(&repo_root()).expect("seed succeeds");
    assert_eq!(
        report.s_perf_11_1_bottleneck_triage_hash_v1, PINNED_S_PERF_11_1_BOTTLENECK_TRIAGE_HASH_V1,
        "S-PERF.11.1 triage hash drifted; rebaseline if the change is panel-acknowledged"
    );
    assert_eq!(
        report.s_perf_11_bandwidth_delta_report_hash_v1,
        S_PERF_11_PINNED_ANCHOR
    );
    assert_eq!(
        report.bottleneck_category,
        BottleneckCategory::DigestStillDominant
    );
    assert_eq!(
        report.next_strike_recommendation,
        NextStrikeRecommendation::SPerf12CompactDensorDigestV1
    );
    assert!(verify_post_rewrite_bottleneck_triage_report(&report).is_empty());
}

// ===============================================================
// Parser smoke + panel-locked constant pins
// ===============================================================

#[test]
fn parser_admits_live_triage_source_report() {
    let path = repo_root().join(S_PERF_11_1_SOURCE_REPORT_PATH);
    let text = std::fs::read_to_string(&path).expect("read triage source");
    let profile = parse_post_rewrite_d64_stage_profile(&text).expect("parse triage source");
    assert_eq!(profile.device_stages.len(), 12);
    assert_eq!(profile.host_compute_features_us, 7821);
    assert_eq!(profile.host_bank_admit_case_finalize_us, 2262);
    assert_eq!(profile.host_wall_median_us, 40418);
    assert_eq!(profile.device_total_us, 30283);
    assert_eq!(profile.triage_run_bandwidth_centi_gbps, 914);
}

#[test]
fn panel_locked_constants_pin_panel_verbatim_values() {
    assert_eq!(S_PERF_11_1_R12B_EPISODES_CANONICAL, 13);
    assert_eq!(S_PERF_11_1_R12B_EPISODES_MID, 89);
    assert_eq!(S_PERF_11_1_R12B_EPISODES_FULL, 1917);
    assert_eq!(S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS, 1333);
    assert_eq!(S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS, 1638);
    assert_eq!(
        S_PERF_11_1_SOURCE_REPORT_PATH,
        "reports/d64_stage_timing_256x4096_K1_post_s_perf_11_triage.txt"
    );
}

#[test]
fn live_seed_admission_path_is_clean() {
    let report =
        seed_post_rewrite_bottleneck_triage_report_from_disk(&repo_root()).expect("seed succeeds");
    let errs = verify_post_rewrite_bottleneck_triage_report(&report);
    assert!(errs.is_empty(), "live seed verifier errors: {errs:?}");
    // Triage-run bandwidth is the live bench measurement; pinned
    // post-S-PERF.11 is the panel-locked 16.38 GB/s reference;
    // pinned-pre is 13.33 GB/s. The verifier checks pinned post +
    // pre are present; triage-run bandwidth is free to drift.
    assert_eq!(
        report.post_s_perf_11_pinned_bandwidth_centi_gbps,
        S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS
    );
    assert_eq!(
        report.pre_s_perf_11_pinned_bandwidth_centi_gbps,
        S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS
    );
}
