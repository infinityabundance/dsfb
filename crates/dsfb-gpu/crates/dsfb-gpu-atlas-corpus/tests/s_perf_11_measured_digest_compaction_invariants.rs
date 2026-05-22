//! S-PERF.11 acceptance suite (panel-locked 2026-05-18).
//!
//! Twelve panel-required positive tests (verbatim names) +
//! eight panel-required campaign-identity negatives (verbatim
//! names) + five structural defect rules + four defense-in-
//! depth structural extras + measured-shape pins + hash
//! determinism / sensitivity + renderer byte-stability +
//! cross-anchor distinctness + pinned-hash back-stop.
//!
//! The matching kernel-side byte-equivalence safety harness
//! lives at `tests/s_perf_11_pre_rewrite_root_capture.rs` and
//! pins the four pre-rewrite `TreeSha256V1` root digests as
//! `[u8; 32]` constants; together the two test files
//! exercise S-PERF.11's same-mode digest-root preservation
//! end-to-end.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::{
    parse_digest_stage_costs, S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS,
    S_PERF_10_TREE_DIGEST_STAGE_DETECTOR, S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
    S_PERF_10_TREE_DIGEST_STAGE_SIGN,
};
use dsfb_gpu_atlas_corpus::s_perf_11_measured_digest_compaction::{
    bandwidth_basis_points, build_digest_compaction_measurement, build_digest_root_equivalence,
    compute_bandwidth_delta_report_hash, compute_digest_compaction_measurement_hash,
    compute_digest_root_equivalence_hash, render_bandwidth_delta_report_json,
    render_bandwidth_delta_report_text, seed_bandwidth_delta_report_from_disk,
    verify_bandwidth_delta_report, BandwidthDeltaReportV1, PrePostRootPairV1, PrePostStageTimingV1,
    SPerf11VerifyErrorKind, S_PERF_11_CHUNK_SIZE_BYTES, S_PERF_11_DIGEST_MODE_LABEL,
    S_PERF_11_LEAVES_PER_BLOCK, S_PERF_11_POST_SOURCE_REPORT_PATH,
    S_PERF_11_PRE_BANDWIDTH_CENTI_GBPS, S_PERF_11_PRE_DIGEST_TOTAL_US,
    S_PERF_11_PRE_ROOT_CONSENSUS, S_PERF_11_PRE_ROOT_DETECTOR, S_PERF_11_PRE_ROOT_RESIDUAL,
    S_PERF_11_PRE_ROOT_SIGN, S_PERF_11_PRE_SOURCE_REPORT_PATH,
    S_PERF_11_PRE_TREE_DIGEST_CONSENSUS_US, S_PERF_11_PRE_TREE_DIGEST_DETECTOR_US,
    S_PERF_11_PRE_TREE_DIGEST_RESIDUAL_US, S_PERF_11_PRE_TREE_DIGEST_SIGN_US,
    S_PERF_11_REWRITE_KERNEL_NAME, S_PERF_11_REWRITE_KIND_LABEL,
    S_PERF_11_SATURATION_THRESHOLD_BASIS_POINTS,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn live_report() -> BandwidthDeltaReportV1 {
    seed_bandwidth_delta_report_from_disk(&repo_root())
        .expect("live disk seed must succeed; post file is whitelisted")
}

// ===============================================================
// Panel-locked twelve positive tests (verbatim names)
// ===============================================================

#[test]
fn admits_measured_digest_compaction_result() {
    let r = live_report();
    let errs = verify_bandwidth_delta_report(&r);
    assert!(
        errs.is_empty(),
        "S-PERF.11 live disk seed must admit cleanly; errors: {errs:?}"
    );
}

#[test]
fn computes_digest_speedup_1_39x() {
    let r = live_report();
    // pre = 11895, post = 8556 → 11895 * 100 / 8556 = 139 centi-x (1.39x)
    assert_eq!(r.measurement.digest_speedup_x_centi, 139);
}

#[test]
fn computes_bandwidth_delta_3_05_gbps() {
    let r = live_report();
    // pre = 1333, post = 1638 → delta = 305 centi-GB/s = 3.05 GB/s
    assert_eq!(r.bandwidth_delta_centi_gbps, 305);
}

#[test]
fn computes_bandwidth_gain_22_9_percent() {
    let r = live_report();
    // delta_bp = 305 * 10000 / 1333 = 2288 bp = 22.88% (panel-pinned ~22.9%)
    assert_eq!(r.bandwidth_delta_basis_points, 2288);
}

#[test]
fn preserves_four_tree_sha256v1_roots() {
    let r = live_report();
    assert!(r.root_equivalence.four_roots_byte_identical);
    for p in &r.root_equivalence.root_pairs {
        assert!(p.byte_identical);
        assert_eq!(p.pre_root, p.post_root);
    }
}

#[test]
fn preserves_r12b_episode_counts_13_89_1917() {
    let r = live_report();
    assert_eq!(r.r12b_episode_count_canonical_w16h128, 13);
    assert_eq!(r.r12b_episode_count_mid_w64h512, 89);
    assert_eq!(r.r12b_episode_count_full_w256h4096, 1917);
}

#[test]
fn binds_s_perf_10_digest_lane_plan_hash() {
    let r = live_report();
    assert_ne!(r.s_perf_10_digest_lane_plan_hash_v1, [0u8; 32]);
    // Sanity: the pinned post-clarification S-PERF.10 plan hash
    // starts with 0xe9 (= `e9cf5c34…`).
    assert_eq!(r.s_perf_10_digest_lane_plan_hash_v1[0], 0xe9);
}

#[test]
fn renderers_are_byte_stable() {
    let r = live_report();
    let a = render_bandwidth_delta_report_text(&r);
    let b = render_bandwidth_delta_report_text(&r);
    assert_eq!(a, b);
    let ja = render_bandwidth_delta_report_json(&r);
    let jb = render_bandwidth_delta_report_json(&r);
    assert_eq!(ja, jb);
}

#[test]
fn hashes_are_deterministic() {
    let a = live_report();
    let b = live_report();
    assert_eq!(
        a.measurement
            .s_perf_11_digest_compaction_measurement_hash_v1,
        b.measurement
            .s_perf_11_digest_compaction_measurement_hash_v1
    );
    assert_eq!(
        a.root_equivalence.s_perf_11_digest_root_equivalence_hash_v1,
        b.root_equivalence.s_perf_11_digest_root_equivalence_hash_v1
    );
    assert_eq!(
        a.s_perf_11_bandwidth_delta_report_hash_v1,
        b.s_perf_11_bandwidth_delta_report_hash_v1
    );
}

#[test]
fn changing_post_digest_total_changes_hash() {
    let post_text =
        std::fs::read_to_string(repo_root().join(S_PERF_11_POST_SOURCE_REPORT_PATH)).unwrap();
    let post_audit = parse_digest_stage_costs(&post_text).unwrap();
    let m1 = build_digest_compaction_measurement(&post_audit);
    let mut shifted = post_audit.clone();
    shifted.consensus.us += 1;
    shifted.digest_total_us += 1;
    let m2 = build_digest_compaction_measurement(&shifted);
    assert_ne!(
        m1.s_perf_11_digest_compaction_measurement_hash_v1,
        m2.s_perf_11_digest_compaction_measurement_hash_v1
    );
}

#[test]
fn changing_post_bandwidth_changes_hash() {
    let mut r = live_report();
    let baseline = r.s_perf_11_bandwidth_delta_report_hash_v1;
    r.post_bandwidth_centi_gbps += 1;
    r.bandwidth_delta_centi_gbps =
        (r.post_bandwidth_centi_gbps as i32) - (r.pre_bandwidth_centi_gbps as i32);
    r.bandwidth_delta_basis_points = ((i64::from(r.bandwidth_delta_centi_gbps)
        .saturating_mul(10_000))
        / i64::from(r.pre_bandwidth_centi_gbps)) as i32;
    let mutated_hash = compute_bandwidth_delta_report_hash(&r);
    assert_ne!(baseline, mutated_hash);
}

#[test]
fn changing_any_digest_root_changes_equivalence_hash() {
    let baseline = build_digest_root_equivalence();
    let mut mutated = baseline.clone();
    mutated.root_pairs[0].post_root[0] ^= 0x01;
    mutated.root_pairs[0].byte_identical = false;
    mutated.four_roots_byte_identical = false;
    let mutated_hash = compute_digest_root_equivalence_hash(&mutated);
    assert_ne!(
        baseline.s_perf_11_digest_root_equivalence_hash_v1,
        mutated_hash
    );
}

// ===============================================================
// Panel-locked eight campaign-identity negatives (verbatim names)
// ===============================================================

#[test]
fn s_perf_11_rejects_speedup_without_digest_root_equivalence() {
    let mut r = live_report();
    r.root_equivalence.four_roots_byte_identical = false;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::SpeedupWithoutDigestRootEquivalence));
}

#[test]
fn s_perf_11_rejects_speedup_without_r12b_episode_stability() {
    let mut r = live_report();
    r.r12b_episode_count_full_w256h4096 = 1918;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::SpeedupWithoutR12bEpisodeStability));
}

#[test]
fn s_perf_11_rejects_digest_total_not_reduced() {
    let mut r = live_report();
    r.measurement.post_digest_total_us = r.measurement.pre_digest_total_us;
    for st in &mut r.measurement.stages {
        st.post_us = st.pre_us;
    }
    r.measurement.digest_delta_us = 0;
    r.measurement.digest_speedup_x_centi = 100;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::DigestTotalNotReduced));
}

#[test]
fn s_perf_11_rejects_bandwidth_not_improved() {
    let mut r = live_report();
    r.post_bandwidth_centi_gbps = r.pre_bandwidth_centi_gbps;
    r.bandwidth_delta_centi_gbps = 0;
    r.bandwidth_delta_basis_points = 0;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::BandwidthNotImproved));
}

#[test]
fn s_perf_11_rejects_missing_s_perf_10_digest_lane_plan_hash() {
    let mut r = live_report();
    r.s_perf_10_digest_lane_plan_hash_v1 = [0u8; 32];
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::MissingSPerf10DigestLanePlanHash));
}

#[test]
fn s_perf_11_rejects_tree_sha256v1_root_drift() {
    let mut r = live_report();
    r.root_equivalence.root_pairs[0].post_root[0] ^= 0x01;
    r.root_equivalence.root_pairs[0].byte_identical = false;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::TreeSha256V1RootDrift));
}

#[test]
fn s_perf_11_rejects_saturation_claim_below_8000_bp() {
    let mut r = live_report();
    r.saturation_admitted = true;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::SaturationClaimBelow8000Bp));
}

#[test]
fn s_perf_11_rejects_claim_that_16_38_gbps_is_memory_saturation() {
    let mut r = live_report();
    // Inject a forbidden saturation phrasing into the
    // rewrite_kind_label.
    r.measurement.rewrite_kind_label = "LeafBatchingV1 saturated the bandwidth";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::ClaimThat1638GbpsIsMemorySaturation));
}

// ===============================================================
// Five structural defect rules
// ===============================================================

#[test]
fn structural_rejects_empty_report_id() {
    let mut r = live_report();
    r.report_id = "";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::ReportIdEmpty));
}

#[test]
fn structural_rejects_empty_pre_source_report_path() {
    let mut r = live_report();
    r.measurement.pre_source_report_path = "";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::PreSourceReportPathEmpty));
}

#[test]
fn structural_rejects_empty_post_source_report_path() {
    let mut r = live_report();
    r.measurement.post_source_report_path = "";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::PostSourceReportPathEmpty));
}

#[test]
fn structural_rejects_digest_speedup_arithmetic_mismatch() {
    let mut r = live_report();
    r.measurement.digest_speedup_x_centi += 1;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::DigestSpeedupArithmeticMismatch));
}

#[test]
fn structural_rejects_bandwidth_delta_arithmetic_mismatch() {
    let mut r = live_report();
    r.bandwidth_delta_basis_points += 1;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::BandwidthDeltaArithmeticMismatch));
}

// ===============================================================
// Four panel-acknowledged defense-in-depth structural extras
// ===============================================================

#[test]
fn structural_rejects_completion_order_fragment_merge() {
    let mut r = live_report();
    r.measurement.rewrite_kind_label = "LeafBatchingCompletionOrderV1";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::StructuralCompletionOrderFragmentMerge));
}

#[test]
fn structural_rejects_casefile_chain_drift() {
    let mut r = live_report();
    r.measurement.digest_mode_label = "CompactDensorDigestV1";
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::StructuralCasefileChainDrift));
}

#[test]
fn structural_rejects_missing_pre_post_digest_table() {
    let mut r = live_report();
    r.measurement.stages[2].post_us = 0;
    r.measurement.post_digest_total_us = r.measurement.stages.iter().map(|s| s.post_us).sum();
    r.measurement.digest_delta_us = i64::try_from(r.measurement.pre_digest_total_us).unwrap()
        - i64::try_from(r.measurement.post_digest_total_us).unwrap();
    r.measurement.digest_speedup_x_centi = if r.measurement.post_digest_total_us == 0 {
        0
    } else {
        ((r.measurement.pre_digest_total_us.saturating_mul(100))
            / r.measurement.post_digest_total_us) as u32
    };
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::StructuralMissingPrePostDigestTable));
}

#[test]
fn structural_rejects_k_amortisation_overclaim() {
    let mut r = live_report();
    r.post_bandwidth_centi_gbps = r.pre_bandwidth_centi_gbps * 5;
    r.bandwidth_delta_centi_gbps =
        (r.post_bandwidth_centi_gbps as i32) - (r.pre_bandwidth_centi_gbps as i32);
    r.bandwidth_delta_basis_points = ((i64::from(r.bandwidth_delta_centi_gbps)
        .saturating_mul(10_000))
        / i64::from(r.pre_bandwidth_centi_gbps)) as i32;
    let errs = verify_bandwidth_delta_report(&r);
    assert!(errs
        .iter()
        .any(|e| e.kind == SPerf11VerifyErrorKind::StructuralKAmortisationOverclaim));
}

// ===============================================================
// Measured-shape pins
// ===============================================================

#[test]
fn pre_residual_us_panel_pin() {
    assert_eq!(S_PERF_11_PRE_TREE_DIGEST_RESIDUAL_US, 2364);
}

#[test]
fn pre_sign_us_panel_pin() {
    assert_eq!(S_PERF_11_PRE_TREE_DIGEST_SIGN_US, 2684);
}

#[test]
fn pre_detector_us_panel_pin() {
    assert_eq!(S_PERF_11_PRE_TREE_DIGEST_DETECTOR_US, 2509);
}

#[test]
fn pre_consensus_us_panel_pin() {
    assert_eq!(S_PERF_11_PRE_TREE_DIGEST_CONSENSUS_US, 4338);
}

#[test]
fn pre_digest_total_us_panel_pin() {
    assert_eq!(S_PERF_11_PRE_DIGEST_TOTAL_US, 11895);
    assert_eq!(
        S_PERF_11_PRE_DIGEST_TOTAL_US,
        S_PERF_11_PRE_TREE_DIGEST_RESIDUAL_US
            + S_PERF_11_PRE_TREE_DIGEST_SIGN_US
            + S_PERF_11_PRE_TREE_DIGEST_DETECTOR_US
            + S_PERF_11_PRE_TREE_DIGEST_CONSENSUS_US
    );
}

#[test]
fn pre_bandwidth_centi_gbps_panel_pin() {
    assert_eq!(S_PERF_11_PRE_BANDWIDTH_CENTI_GBPS, 1333);
}

#[test]
fn rewrite_kernel_name_pin() {
    assert_eq!(S_PERF_11_REWRITE_KERNEL_NAME, "tree_digest_leaf_kernel_v2");
}

#[test]
fn rewrite_kind_label_pin() {
    assert_eq!(S_PERF_11_REWRITE_KIND_LABEL, "LeafBatchingV1");
}

#[test]
fn rewrite_leaves_per_block_pin() {
    assert_eq!(S_PERF_11_LEAVES_PER_BLOCK, 32);
}

#[test]
fn rewrite_digest_mode_label_pin() {
    assert_eq!(S_PERF_11_DIGEST_MODE_LABEL, "TreeSha256V1");
}

#[test]
fn rewrite_chunk_size_bytes_pin() {
    assert_eq!(S_PERF_11_CHUNK_SIZE_BYTES, 16384);
}

#[test]
fn live_seed_records_pre_paths() {
    let r = live_report();
    assert_eq!(
        r.measurement.pre_source_report_path,
        S_PERF_11_PRE_SOURCE_REPORT_PATH
    );
    assert_eq!(
        r.measurement.post_source_report_path,
        S_PERF_11_POST_SOURCE_REPORT_PATH
    );
}

#[test]
fn live_seed_records_canonical_stage_labels() {
    let r = live_report();
    assert_eq!(
        r.measurement.stages[0].stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL
    );
    assert_eq!(
        r.measurement.stages[1].stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_SIGN
    );
    assert_eq!(
        r.measurement.stages[2].stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_DETECTOR
    );
    assert_eq!(
        r.measurement.stages[3].stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS
    );
}

#[test]
fn live_seed_records_panel_pinned_pre_roots() {
    let r = live_report();
    assert_eq!(
        r.root_equivalence.root_pairs[0].pre_root,
        S_PERF_11_PRE_ROOT_RESIDUAL
    );
    assert_eq!(
        r.root_equivalence.root_pairs[1].pre_root,
        S_PERF_11_PRE_ROOT_SIGN
    );
    assert_eq!(
        r.root_equivalence.root_pairs[2].pre_root,
        S_PERF_11_PRE_ROOT_DETECTOR
    );
    assert_eq!(
        r.root_equivalence.root_pairs[3].pre_root,
        S_PERF_11_PRE_ROOT_CONSENSUS
    );
}

#[test]
fn post_digest_total_us_below_pre() {
    let r = live_report();
    assert!(r.measurement.post_digest_total_us < S_PERF_11_PRE_DIGEST_TOTAL_US);
}

#[test]
fn post_bandwidth_above_pre() {
    let r = live_report();
    assert!(r.post_bandwidth_centi_gbps > S_PERF_11_PRE_BANDWIDTH_CENTI_GBPS);
}

// ===============================================================
// Cross-anchor distinctness + sensitivity
// ===============================================================

#[test]
fn three_s_perf_11_hashes_pairwise_distinct() {
    let r = live_report();
    let h1 = r
        .measurement
        .s_perf_11_digest_compaction_measurement_hash_v1;
    let h2 = r.root_equivalence.s_perf_11_digest_root_equivalence_hash_v1;
    let h3 = r.s_perf_11_bandwidth_delta_report_hash_v1;
    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_ne!(h1, h3);
}

#[test]
fn s_perf_11_hashes_distinct_from_upstream_anchors() {
    let r = live_report();
    let top = r.s_perf_11_bandwidth_delta_report_hash_v1;
    assert_ne!(top, r.s_perf_6_baseline_report_hash_v1);
    assert_ne!(top, r.s_perf_7_source_report_import_verifier_hash_v1);
    assert_ne!(top, r.s_perf_8_batched_k_saturation_receipt_hash_v1);
    assert_ne!(top, r.s_perf_10_digest_lane_plan_hash_v1);
}

#[test]
fn changing_rewrite_label_changes_measurement_hash() {
    let post_text =
        std::fs::read_to_string(repo_root().join(S_PERF_11_POST_SOURCE_REPORT_PATH)).unwrap();
    let post_audit = parse_digest_stage_costs(&post_text).unwrap();
    let baseline = build_digest_compaction_measurement(&post_audit);
    let mut mutated = baseline.clone();
    mutated.rewrite_kind_label = "LeafBatchingV2";
    let mutated_hash = compute_digest_compaction_measurement_hash(&mutated);
    assert_ne!(
        baseline.s_perf_11_digest_compaction_measurement_hash_v1,
        mutated_hash
    );
}

#[test]
fn measurement_hash_helper_matches_recompute() {
    let r = live_report();
    let recomputed = compute_digest_compaction_measurement_hash(&r.measurement);
    assert_eq!(
        recomputed,
        r.measurement
            .s_perf_11_digest_compaction_measurement_hash_v1
    );
}

#[test]
fn root_equivalence_hash_helper_matches_recompute() {
    let r = live_report();
    let recomputed = compute_digest_root_equivalence_hash(&r.root_equivalence);
    assert_eq!(
        recomputed,
        r.root_equivalence.s_perf_11_digest_root_equivalence_hash_v1
    );
}

#[test]
fn top_hash_helper_matches_recompute() {
    let r = live_report();
    let recomputed = compute_bandwidth_delta_report_hash(&r);
    assert_eq!(recomputed, r.s_perf_11_bandwidth_delta_report_hash_v1);
}

// ===============================================================
// Saturation arithmetic
// ===============================================================

#[test]
fn saturation_admitted_false_at_post_below_threshold() {
    let r = live_report();
    let post_bp = bandwidth_basis_points(r.post_bandwidth_centi_gbps);
    assert!(post_bp < S_PERF_11_SATURATION_THRESHOLD_BASIS_POINTS);
    assert!(!r.saturation_admitted);
}

#[test]
fn bandwidth_basis_points_floor_law() {
    // 16.38 GB/s → 1638 centi → 1638 * 10000 / 71600 =
    // 228.77 → floor 228.
    assert_eq!(bandwidth_basis_points(1638), 228);
    // 13.33 GB/s → 1333 * 10000 / 71600 = 186.17 → floor 186.
    assert_eq!(bandwidth_basis_points(1333), 186);
    // 80% gate boundary: 572.8 GB/s → 57280 centi → 57280 *
    // 10000 / 71600 = 8000 bp exact.
    assert_eq!(bandwidth_basis_points(57280), 8000);
}

// ===============================================================
// Renderer content checks
// ===============================================================

#[test]
fn text_renderer_contains_panel_locked_phrases() {
    let r = live_report();
    let a = render_bandwidth_delta_report_text(&r);
    assert!(a.contains("tree_digest residual"));
    assert!(a.contains("tree_digest sign"));
    assert!(a.contains("tree_digest detector (wide cells)"));
    assert!(a.contains("tree_digest consensus"));
    assert!(a.contains("Digest-root equivalence"));
    assert!(a.contains("Panel-locked report wording"));
    assert!(a.contains("Panel-locked one-line verdict"));
    assert!(a.contains("saturation_admitted"));
}

#[test]
fn json_renderer_contains_panel_locked_field_names() {
    let r = live_report();
    let a = render_bandwidth_delta_report_json(&r);
    assert!(a.contains("\"rewrite_kernel_name\": \"tree_digest_leaf_kernel_v2\""));
    assert!(a.contains("\"digest_mode_label\": \"TreeSha256V1\""));
    assert!(a.contains("\"four_roots_byte_identical\": true"));
    assert!(a.contains("\"s_perf_11_digest_compaction_measurement_hash_v1\""));
    assert!(a.contains("\"s_perf_11_digest_root_equivalence_hash_v1\""));
    assert!(a.contains("\"s_perf_11_bandwidth_delta_report_hash_v1\""));
}

// ===============================================================
// Pinned-hash back-stop (refreshed after the panel-locked
// rename + new root-equivalence hash; capture mode prints the
// live hash so the operator can refresh the pin on first run).
// ===============================================================

const PINNED_S_PERF_11_BANDWIDTH_DELTA_REPORT_HASH_V1: [u8; 32] = [
    0x1a, 0x27, 0x15, 0x4e, 0x33, 0x5c, 0x27, 0xdf, 0x6d, 0xb9, 0x39, 0xd4, 0xc8, 0xff, 0x0f, 0x36,
    0xf8, 0xba, 0xf7, 0x58, 0x71, 0xbe, 0x06, 0xc7, 0x50, 0xf0, 0x85, 0x3f, 0x22, 0x68, 0xad, 0xc8,
];

#[test]
fn pinned_top_level_hash_matches_live_disk_seed() {
    let r = live_report();
    if PINNED_S_PERF_11_BANDWIDTH_DELTA_REPORT_HASH_V1 == [0u8; 32] {
        use std::fmt::Write;
        let mut hex = String::with_capacity(96);
        for (i, b) in (0..).zip(&r.s_perf_11_bandwidth_delta_report_hash_v1) {
            if i > 0 {
                hex.push_str(", ");
            }
            let _ = write!(hex, "0x{b:02x}");
        }
        panic!(
            "PINNED_S_PERF_11_BANDWIDTH_DELTA_REPORT_HASH_V1 still all zeros; \
             refresh with live hash: [\n    {hex}\n]"
        );
    }
    assert_eq!(
        r.s_perf_11_bandwidth_delta_report_hash_v1,
        PINNED_S_PERF_11_BANDWIDTH_DELTA_REPORT_HASH_V1
    );
}

#[test]
fn pre_post_root_pair_round_trip() {
    let p = PrePostRootPairV1 {
        stage_label: S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
        pre_root: [1u8; 32],
        post_root: [1u8; 32],
        byte_identical: true,
    };
    assert!(p.byte_identical);
    assert_eq!(p.pre_root, p.post_root);
}

#[test]
fn pre_post_stage_timing_round_trip() {
    let t = PrePostStageTimingV1 {
        stage_label: S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
        pre_us: 1234,
        post_us: 567,
    };
    assert_eq!(t.pre_us, 1234);
    assert_eq!(t.post_us, 567);
}

#[test]
fn live_report_satisfies_panel_locked_post_values() {
    let r = live_report();
    assert_eq!(r.measurement.post_digest_total_us, 8556);
    assert_eq!(r.post_bandwidth_centi_gbps, 1638);
    assert_eq!(r.measurement.stages[0].post_us, 1685); // residual
    assert_eq!(r.measurement.stages[1].post_us, 1929); // sign
    assert_eq!(r.measurement.stages[2].post_us, 2052); // detector
    assert_eq!(r.measurement.stages[3].post_us, 2890); // consensus
}

#[test]
fn live_report_records_panel_locked_measurement_total() {
    let r = live_report();
    let sum: u64 = r.measurement.stages.iter().map(|s| s.post_us).sum();
    assert_eq!(sum, r.measurement.post_digest_total_us);
}

#[test]
fn changing_episode_count_changes_top_level_hash() {
    let mut r = live_report();
    let baseline = r.s_perf_11_bandwidth_delta_report_hash_v1;
    r.r12b_episode_count_canonical_w16h128 = 14;
    let mutated = compute_bandwidth_delta_report_hash(&r);
    assert_ne!(baseline, mutated);
}
