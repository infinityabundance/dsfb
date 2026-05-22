//! S-PERF.10 acceptance suite (DigestLanePlanV1 /
//! digest-cost audit, receipt-only).
//!
//! Test groups:
//!
//!   - parser tests
//!   - verifier negatives (8 panel-required + structural)
//!   - measured-shape pins (per-stage us + total +
//!     pct-band)
//!   - hash determinism + sensitivity
//!   - renderer byte-stability + panel-pinned summary
//!   - upstream-anchor binding + distinctness
//!   - pinned-hash back-stop
//!
//! The CAMPAIGN IDENTITY negative is
//! `s_perf_10_rejects_digest_optimisation_claim_without_byte_identical_digest_roots`
//! because the dangerous overclaim S-PERF.10 must
//! mechanically forbid is *"the digest is optimized but
//! the digest roots changed"*. The contract law text is
//! folded into the hash so any rewrite that weakens it
//! surfaces as both a verifier rejection AND a hash
//! change.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::{
    build_digest_compaction_contract, build_digest_lane_plan, compute_digest_lane_plan_hash,
    parse_digest_stage_costs, render_digest_lane_plan_json, render_digest_lane_plan_text,
    seed_digest_lane_plan_from_disk, verify_digest_lane_plan, DigestLanePlanV1, ParseError,
    SPerf10VerifyErrorKind, S_PERF_10_CANONICAL_FRAGMENT_MERGE_ORDER_LAW,
    S_PERF_10_CASEFILE_CHAIN_PRESERVATION_LAW, S_PERF_10_DIGEST_MODE_NON_ALIASING_LAW,
    S_PERF_10_SAME_MODE_DIGEST_ROOT_LAW, S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS,
    S_PERF_10_TREE_DIGEST_STAGE_DETECTOR, S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
    S_PERF_10_TREE_DIGEST_STAGE_SIGN,
};
use dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, R12B_EPISODE_COUNT_FULL_W256H4096,
};
use dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::seed_source_report_import_verifier_report_from_disk;
use dsfb_gpu_atlas_corpus::s_perf_8_batched_k_saturation_receipt::seed_batched_k_saturation_receipt_from_disk;

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
    errors: &[dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::SPerf10VerifyError],
    pred: impl Fn(&SPerf10VerifyErrorKind) -> bool,
) -> bool {
    errors.iter().any(|e| pred(&e.kind))
}

fn live_plan() -> DigestLanePlanV1 {
    seed_digest_lane_plan_from_disk(&repo_root()).unwrap()
}

// ---------------------------------------------------------------
// Parser tests
// ---------------------------------------------------------------

#[test]
fn parses_live_digest_stage_rows_from_disk() {
    let path = repo_root().join("reports/d64_stage_timing_256x4096_K1.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let audit = parse_digest_stage_costs(&text).unwrap();
    assert_eq!(
        audit.residual.stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL
    );
    assert_eq!(audit.sign.stage_label, S_PERF_10_TREE_DIGEST_STAGE_SIGN);
    assert_eq!(
        audit.detector.stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_DETECTOR
    );
    assert_eq!(
        audit.consensus.stage_label,
        S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS
    );
}

#[test]
fn parser_rejects_text_missing_consensus_row() {
    let bad = "tree_digest residual | 2364 | 11.4\ntree_digest sign | 2684 | 12.9\ntree_digest detector (wide cells) | 2509 | 12.1\n";
    let err = parse_digest_stage_costs(bad).unwrap_err();
    assert!(matches!(
        err,
        ParseError::MissingTreeDigestRow {
            stage: S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS
        }
    ));
}

#[test]
fn parser_rejects_text_missing_residual_row() {
    let bad = "tree_digest sign | 2684 | 12.9\ntree_digest detector (wide cells) | 2509 | 12.1\ntree_digest consensus | 4338 | 20.9\n";
    let err = parse_digest_stage_costs(bad).unwrap_err();
    assert!(matches!(
        err,
        ParseError::MissingTreeDigestRow {
            stage: S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL
        }
    ));
}

#[test]
fn parser_total_us_equals_sum_of_four_stages() {
    let path = repo_root().join("reports/d64_stage_timing_256x4096_K1.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let audit = parse_digest_stage_costs(&text).unwrap();
    assert_eq!(
        audit.digest_total_us,
        audit.residual.us + audit.sign.us + audit.detector.us + audit.consensus.us
    );
}

// ---------------------------------------------------------------
// Measured-shape pins
// ---------------------------------------------------------------

#[test]
fn pins_tree_digest_residual_us() {
    let plan = live_plan();
    // Panel-pinned band: residual in 2000..3000 us
    // (live measurement 2364 us with thermal tolerance).
    assert!(
        (2000..=3000).contains(&plan.audit.residual.us),
        "tree_digest residual us out of band: got {}",
        plan.audit.residual.us
    );
}

#[test]
fn pins_tree_digest_sign_us() {
    let plan = live_plan();
    // Panel-pinned band: sign in 2200..3200 us (live 2684).
    assert!(
        (2200..=3200).contains(&plan.audit.sign.us),
        "tree_digest sign us out of band: got {}",
        plan.audit.sign.us
    );
}

#[test]
fn pins_tree_digest_detector_us() {
    let plan = live_plan();
    // Panel-pinned band: detector in 2000..3100 us (live 2509).
    assert!(
        (2000..=3100).contains(&plan.audit.detector.us),
        "tree_digest detector us out of band: got {}",
        plan.audit.detector.us
    );
}

#[test]
fn pins_tree_digest_consensus_us() {
    let plan = live_plan();
    // Panel-pinned band: consensus in 3800..4900 us (live 4338).
    assert!(
        (3800..=4900).contains(&plan.audit.consensus.us),
        "tree_digest consensus us out of band: got {}",
        plan.audit.consensus.us
    );
}

#[test]
fn pins_digest_total_us_band() {
    let plan = live_plan();
    // Panel-pinned tolerance band: 11000..13000 us
    // (live sum 11895).
    assert!(
        (11000..=13000).contains(&plan.audit.digest_total_us),
        "digest_total_us out of band: got {}",
        plan.audit.digest_total_us
    );
}

#[test]
fn pins_digest_total_pct_in_panel_band() {
    let plan = live_plan();
    // Panel-locked band: 50%..65% (5000..=6500 bp).
    assert!(
        (5000..=6500).contains(&plan.audit.digest_total_pct_basis_points),
        "digest_total_pct must be in [50%, 65%]; got {} bp",
        plan.audit.digest_total_pct_basis_points
    );
}

#[test]
fn pins_full_scale_episode_count_at_1917() {
    let plan = live_plan();
    assert_eq!(plan.r12b_episode_count_full_w256h4096, 1917);
}

#[test]
fn verifier_admits_live_disk_seed() {
    let plan = live_plan();
    let errors = verify_digest_lane_plan(&plan);
    assert!(
        errors.is_empty(),
        "verifier should admit the live disk seed but got: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Panel-required load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn s_perf_10_rejects_digest_optimisation_claim_without_byte_identical_digest_roots() {
    // CAMPAIGN IDENTITY: weaken the same_mode_digest_root_law text;
    // verifier must reject (the panel-locked law is folded
    // into the hash AND verified literally).
    let mut plan = live_plan();
    plan.contract.same_mode_digest_root_law = "Digest roots may differ if the rewrite is fast.";
    plan.contract.digest_compaction_contract_hash_v1 = [0u8; 32];
    plan.digest_lane_plan_hash_v1 = compute_digest_lane_plan_hash(&plan);
    let errors = verify_digest_lane_plan(&plan);
    assert!(
        has_kind(&errors, |k| matches!(
            k,
            SPerf10VerifyErrorKind::DigestOptimisationClaimWithoutByteIdenticalDigestRoots
        )),
        "expected DigestOptimisationClaimWithoutByteIdenticalDigestRoots; got {errors:?}"
    );
}

#[test]
fn s_perf_10_rejects_digest_plan_without_four_tree_digest_stage_timings() {
    let mut plan = live_plan();
    plan.audit.residual.us = 0;
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutFourTreeDigestStageTimings
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_without_total_digest_share() {
    let mut plan = live_plan();
    // Push digest_total_pct outside the panel band.
    plan.audit.digest_total_pct_basis_points = 100; // 1%
    plan.audit.residual.pct_basis_points = 25;
    plan.audit.sign.pct_basis_points = 25;
    plan.audit.detector.pct_basis_points = 25;
    plan.audit.consensus.pct_basis_points = 25;
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutTotalDigestShare
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_without_s_perf_8_1_anchor() {
    let mut plan = live_plan();
    plan.s_perf_8_batched_k_saturation_receipt_hash_v1 = [0u8; 32];
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutSPerf81Anchor
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_without_s_perf_6_measured_baseline_anchor() {
    let mut plan = live_plan();
    plan.s_perf_6_baseline_report_hash_v1 = [0u8; 32];
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutSPerf6MeasuredBaselineAnchor
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_that_claims_bandwidth_improvement() {
    let mut plan = live_plan();
    plan.contract.casefile_chain_preservation_law =
        "Digest compaction achieves saturation; bandwidth improvement is locked in.";
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanThatClaimsBandwidthImprovement
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_without_future_rewrite_contract() {
    let mut plan = live_plan();
    plan.contract.canonical_fragment_merge_order_law = "";
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutFutureRewriteContract
    )));
}

#[test]
fn s_perf_10_rejects_digest_plan_with_episode_count_drift() {
    let mut plan = live_plan();
    plan.r12b_episode_count_full_w256h4096 = 1916;
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithEpisodeCountDrift
    )));
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn structural_rejects_empty_plan_id() {
    let mut plan = live_plan();
    plan.plan_id = "";
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::PlanIdEmpty
    )));
}

#[test]
fn structural_rejects_empty_source_report_path() {
    let mut plan = live_plan();
    plan.audit.source_report_path = "";
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::SourceReportPathEmpty
    )));
}

#[test]
fn structural_rejects_digest_total_us_arithmetic_mismatch() {
    let mut plan = live_plan();
    plan.audit.digest_total_us += 1;
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestTotalUsArithmeticMismatch
    )));
}

#[test]
fn structural_rejects_digest_total_pct_arithmetic_mismatch() {
    let mut plan = live_plan();
    plan.audit.digest_total_pct_basis_points += 100;
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestTotalPctArithmeticMismatch
    )));
}

#[test]
fn structural_rejects_missing_s_perf_7_anchor() {
    let mut plan = live_plan();
    plan.s_perf_7_source_report_import_verifier_hash_v1 = [0u8; 32];
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::DigestPlanWithoutSPerf7VerifierAnchor
    )));
}

#[test]
fn structural_rejects_stage_label_mismatch() {
    let mut plan = live_plan();
    plan.audit.residual.stage_label = "tree_digest residual_renamed";
    let errors = verify_digest_lane_plan(&plan);
    assert!(has_kind(&errors, |k| matches!(
        k,
        SPerf10VerifyErrorKind::StageLabelMismatch
    )));
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn hash_is_deterministic_across_two_seeds() {
    let a = live_plan();
    let b = live_plan();
    assert_eq!(a.digest_lane_plan_hash_v1, b.digest_lane_plan_hash_v1);
    assert_eq!(
        a.audit.digest_stage_cost_audit_hash_v1,
        b.audit.digest_stage_cost_audit_hash_v1
    );
    assert_eq!(
        a.contract.digest_compaction_contract_hash_v1,
        b.contract.digest_compaction_contract_hash_v1
    );
}

#[test]
fn changing_consensus_us_changes_hash() {
    let mut plan = live_plan();
    let before = plan.digest_lane_plan_hash_v1;
    plan.audit.consensus.us += 1;
    plan.audit.digest_total_us += 1;
    plan.audit.digest_stage_cost_audit_hash_v1 =
        dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::compute_digest_stage_cost_audit_hash(
            &plan.audit,
        );
    let after = compute_digest_lane_plan_hash(&plan);
    assert_ne!(before, after);
}

#[test]
fn changing_digest_root_law_changes_contract_hash() {
    let mut c = build_digest_compaction_contract();
    let before = c.digest_compaction_contract_hash_v1;
    c.same_mode_digest_root_law = "Different law text.";
    c.digest_compaction_contract_hash_v1 =
        dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::compute_digest_compaction_contract_hash(
            &c,
        );
    assert_ne!(before, c.digest_compaction_contract_hash_v1);
}

#[test]
fn changing_episode_count_changes_plan_hash() {
    let mut plan = live_plan();
    let before = plan.digest_lane_plan_hash_v1;
    plan.r12b_episode_count_full_w256h4096 = 1918;
    let after = compute_digest_lane_plan_hash(&plan);
    assert_ne!(before, after);
}

#[test]
fn changing_s_perf_8_anchor_changes_plan_hash() {
    let mut plan = live_plan();
    let before = plan.digest_lane_plan_hash_v1;
    plan.s_perf_8_batched_k_saturation_receipt_hash_v1[0] ^= 0xff;
    let after = compute_digest_lane_plan_hash(&plan);
    assert_ne!(before, after);
}

#[test]
fn changing_source_report_path_changes_audit_hash() {
    let mut plan = live_plan();
    let before = plan.audit.digest_stage_cost_audit_hash_v1;
    plan.audit.source_report_path = "reports/different.txt";
    let after =
        dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::compute_digest_stage_cost_audit_hash(
            &plan.audit,
        );
    assert_ne!(before, after);
}

// ---------------------------------------------------------------
// Renderer byte-stability + summary text
// ---------------------------------------------------------------

#[test]
fn text_renderer_is_byte_stable() {
    let plan = live_plan();
    assert_eq!(
        render_digest_lane_plan_text(&plan),
        render_digest_lane_plan_text(&plan)
    );
}

#[test]
fn json_renderer_is_byte_stable() {
    let plan = live_plan();
    assert_eq!(
        render_digest_lane_plan_json(&plan),
        render_digest_lane_plan_json(&plan)
    );
}

#[test]
fn text_renderer_contains_panel_pinned_summary_prose() {
    let plan = live_plan();
    let text = render_digest_lane_plan_text(&plan);
    assert!(text.contains("audits the measured digest-lane bottleneck"));
    assert!(text.contains("DigestLanePlanV1"));
    assert!(text.contains("does not claim bandwidth improvement"));
    assert!(text.contains("preservation contract"));
    assert!(text.contains("same_mode_digest_root_law:"));
    assert!(text.contains("canonical_fragment_merge_order_law:"));
    assert!(text.contains("digest_mode_non_aliasing_law:"));
    assert!(text.contains("casefile_chain_preservation_law:"));
}

#[test]
fn text_renderer_lists_all_four_tree_digest_stages() {
    let plan = live_plan();
    let text = render_digest_lane_plan_text(&plan);
    assert!(text.contains(S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL));
    assert!(text.contains(S_PERF_10_TREE_DIGEST_STAGE_SIGN));
    assert!(text.contains(S_PERF_10_TREE_DIGEST_STAGE_DETECTOR));
    assert!(text.contains(S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS));
}

// ---------------------------------------------------------------
// Upstream anchor distinctness
// ---------------------------------------------------------------

#[test]
fn three_hashes_pairwise_distinct() {
    let plan = live_plan();
    assert_ne!(
        plan.audit.digest_stage_cost_audit_hash_v1,
        plan.contract.digest_compaction_contract_hash_v1
    );
    assert_ne!(
        plan.audit.digest_stage_cost_audit_hash_v1,
        plan.digest_lane_plan_hash_v1
    );
    assert_ne!(
        plan.contract.digest_compaction_contract_hash_v1,
        plan.digest_lane_plan_hash_v1
    );
}

#[test]
fn plan_hash_distinct_from_s_perf_6_7_8_anchors() {
    let plan = live_plan();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let s_perf_7 = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    let s_perf_8 = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    assert_ne!(
        plan.digest_lane_plan_hash_v1,
        baseline.rtx4080_super_measured_baseline_report_hash_v1
    );
    assert_ne!(
        plan.digest_lane_plan_hash_v1,
        s_perf_7.source_report_import_verifier_hash_v1
    );
    assert_ne!(
        plan.digest_lane_plan_hash_v1,
        s_perf_8.batched_k_saturation_receipt_hash_v1
    );
}

#[test]
fn build_lane_plan_binds_all_three_upstream_anchors() {
    let audit = parse_digest_stage_costs(
        &std::fs::read_to_string(repo_root().join("reports/d64_stage_timing_256x4096_K1.txt"))
            .unwrap(),
    )
    .unwrap();
    let contract = build_digest_compaction_contract();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let s_perf_7 = seed_source_report_import_verifier_report_from_disk(&repo_root()).unwrap();
    let s_perf_8 = seed_batched_k_saturation_receipt_from_disk(&repo_root()).unwrap();
    let plan = build_digest_lane_plan(
        "s_perf_10_digest_lane_plan_v1",
        audit,
        contract,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        s_perf_7.source_report_import_verifier_hash_v1,
        s_perf_8.batched_k_saturation_receipt_hash_v1,
        R12B_EPISODE_COUNT_FULL_W256H4096,
    );
    assert_eq!(
        plan.s_perf_6_baseline_report_hash_v1,
        baseline.rtx4080_super_measured_baseline_report_hash_v1
    );
    assert_eq!(
        plan.s_perf_7_source_report_import_verifier_hash_v1,
        s_perf_7.source_report_import_verifier_hash_v1
    );
    assert_eq!(
        plan.s_perf_8_batched_k_saturation_receipt_hash_v1,
        s_perf_8.batched_k_saturation_receipt_hash_v1
    );
}

// ---------------------------------------------------------------
// Contract construction
// ---------------------------------------------------------------

#[test]
fn contract_law_text_matches_panel_locked_constants() {
    let c = build_digest_compaction_contract();
    assert_eq!(
        c.same_mode_digest_root_law,
        S_PERF_10_SAME_MODE_DIGEST_ROOT_LAW
    );
    assert_eq!(
        c.canonical_fragment_merge_order_law,
        S_PERF_10_CANONICAL_FRAGMENT_MERGE_ORDER_LAW
    );
    assert_eq!(
        c.digest_mode_non_aliasing_law,
        S_PERF_10_DIGEST_MODE_NON_ALIASING_LAW
    );
    assert_eq!(
        c.casefile_chain_preservation_law,
        S_PERF_10_CASEFILE_CHAIN_PRESERVATION_LAW
    );
}

#[test]
fn contract_hash_changes_when_any_law_changes() {
    let baseline = build_digest_compaction_contract();
    let mut c = baseline.clone();
    c.canonical_fragment_merge_order_law = "Different order law.";
    c.digest_compaction_contract_hash_v1 =
        dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::compute_digest_compaction_contract_hash(
            &c,
        );
    assert_ne!(
        c.digest_compaction_contract_hash_v1,
        baseline.digest_compaction_contract_hash_v1
    );
}

// ---------------------------------------------------------------
// Pinned-hash back-stop
// ---------------------------------------------------------------

/// Pinned `digest_lane_plan_hash_v1` for the live disk
/// seed. Any silent rebaseline fails this test loudly;
/// refresh ONLY when the panel approves a schema or
/// upstream-anchor change.
///
/// S-PERF.11 Phase 0b refresh (2026-05-18): panel-acknowledged
/// contract wording clarification — 4 law fields renamed +
/// rewritten to remove the ambiguous "cross-mode root
/// identity" reading. The audit content is unchanged; the
/// contract hash + plan hash rebaseline accordingly.
const PINNED_DIGEST_LANE_PLAN_HASH_V1: [u8; 32] = [
    0xe9, 0xcf, 0x5c, 0x34, 0x18, 0x01, 0x50, 0x44, 0xa0, 0x3e, 0x4b, 0x36, 0x9f, 0xef, 0xdf, 0x28,
    0x49, 0x08, 0xf4, 0xb6, 0x34, 0x12, 0xfe, 0x37, 0x21, 0x87, 0xde, 0x67, 0x0d, 0xb0, 0x78, 0x11,
];

#[test]
fn pinned_plan_hash_matches_live_disk_seed() {
    let plan = live_plan();
    assert_eq!(
        plan.digest_lane_plan_hash_v1, PINNED_DIGEST_LANE_PLAN_HASH_V1,
        "S-PERF.10 plan hash drifted; refresh the pinned constant if the source report changed"
    );
}
