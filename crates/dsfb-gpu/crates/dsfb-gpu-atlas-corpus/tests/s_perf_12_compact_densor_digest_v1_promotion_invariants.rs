//! S-PERF.12 — CompactDensorDigestV1 throughput-mode
//! promotion-seal acceptance suite.
//!
//! Panel-locked rules (verbatim from the post-S-PERF.14c
//! panel directive). Eight campaign-identity negatives +
//! 4 structural-defect rules + 8 positives + determinism /
//! sensitivity / rendering invariants.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::s_perf_12_compact_densor_digest_v1_promotion::{
    build_promotion_fields, build_promotion_report, compute_promotion_fields_hash,
    compute_promotion_report_hash, render_promotion_report_json, render_promotion_report_text,
    seed_s_perf_12_promotion_report_from_disk, verify_promotion_report,
    SPerf12PromotionVerifyErrorKind, COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME,
    POST_S_PERF_11_BANDWIDTH_CENTI_GBPS, POST_S_PERF_14C_BANDWIDTH_BAND_MAX_CENTI_GBPS,
    POST_S_PERF_14C_BANDWIDTH_BAND_MEDIAN_CENTI_GBPS,
    POST_S_PERF_14C_BANDWIDTH_BAND_MIN_CENTI_GBPS, PRE_S_PERF_11_BANDWIDTH_CENTI_GBPS,
    R12B_EPISODE_COUNT_CANONICAL_W16H128, R12B_EPISODE_COUNT_FULL_W256H4096,
    R12B_EPISODE_COUNT_MID_W64H512, S_PERF_11_COMMIT_SHA,
    S_PERF_12A_CANDIDATE_BANDWIDTH_CENTI_GBPS, S_PERF_12_PROMOTION_GATE_CENTI_GBPS,
    S_PERF_14B_COMMIT_SHA, S_PERF_14C_COMMIT_SHA,
};

/// Build the canonical (admit-shaped) report using
/// non-zero stub anchor hashes. Acceptance suite reuses
/// this baseline for every test that wants to start from
/// the admit-shape and mutate one field.
fn canonical_report(
) -> dsfb_gpu_atlas_corpus::s_perf_12_compact_densor_digest_v1_promotion::SPerf12PromotionReportV1 {
    let stub_s_perf_11_hash: [u8; 32] = [1u8; 32];
    let stub_s_perf_11_1_hash: [u8; 32] = [2u8; 32];
    let stub_corpus_hash: [u8; 32] = [3u8; 32];
    build_promotion_report(stub_s_perf_11_hash, stub_s_perf_11_1_hash, stub_corpus_hash)
}

// ---------------------------------------------------------------
// 8 panel-required campaign-identity negatives (verbatim)
// ---------------------------------------------------------------

#[test]
fn s_perf_12_rejects_promotion_seal_when_bandwidth_below_gate() {
    let mut r = canonical_report();
    // Force the MIN of the band below the gate.
    r.fields.post_s_perf_14c_bandwidth_band_min_centi_gbps =
        S_PERF_12_PROMOTION_GATE_CENTI_GBPS - 1;
    r.fields.promotion_gate_passed = false;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWhenBandwidthBelowGate
        )),
        "verifier did NOT reject promotion seal with band_min below gate; \
         errs={errs:?}"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_with_saturation_claim() {
    let mut r = canonical_report();
    r.fields.saturation_admitted = true;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithSaturationClaim
        )),
        "verifier did NOT reject promotion seal with saturation_admitted=true"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_with_tree_sha256v1_aliasing_claim() {
    let mut r = canonical_report();
    r.fields.tree_sha256v1_root_aliasing = true;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithTreeSha256V1AliasingClaim
        )),
        "verifier did NOT reject promotion seal with tree_sha256v1_root_aliasing=true"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_with_r12b_episode_drift() {
    for (mut r, label) in [
        (canonical_report(), "canonical_w16h128"),
        (canonical_report(), "mid_w64h512"),
        (canonical_report(), "full_w256h4096"),
    ] {
        match label {
            "canonical_w16h128" => r.fields.r12b_episode_count_canonical_w16h128 = 12,
            "mid_w64h512" => r.fields.r12b_episode_count_mid_w64h512 = 88,
            "full_w256h4096" => r.fields.r12b_episode_count_full_w256h4096 = 1916,
            _ => unreachable!(),
        }
        let errs = verify_promotion_report(&r);
        assert!(
            errs.iter().any(|e| matches!(
                e.kind,
                SPerf12PromotionVerifyErrorKind::PromotionSealWithR12bEpisodeDrift
            )),
            "verifier did NOT reject promotion seal with R.12b drift on {label}"
        );
    }
}

#[test]
fn s_perf_12_rejects_promotion_seal_without_s_perf_11_anchor() {
    let mut r = canonical_report();
    r.s_perf_11_bandwidth_delta_report_hash_v1 = [0u8; 32];
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithoutSPerf11Anchor
        )),
        "verifier did NOT reject promotion seal with zero S-PERF.11 anchor"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_without_s_perf_11_1_anchor() {
    let mut r = canonical_report();
    r.s_perf_11_1_bottleneck_triage_hash_v1 = [0u8; 32];
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithoutSPerf11_1Anchor
        )),
        "verifier did NOT reject promotion seal with zero S-PERF.11.1 anchor"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_without_audit_mode_unchanged_flag() {
    let mut r = canonical_report();
    r.fields.audit_mode_unchanged = false;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithoutAuditModeUnchangedFlag
        )),
        "verifier did NOT reject promotion seal with audit_mode_unchanged=false"
    );
}

#[test]
fn s_perf_12_rejects_promotion_seal_without_compact_densor_digest_v1_mode_identity() {
    let mut r = canonical_report();
    r.fields.digest_mode_wire_name = "TreeSha256V1";
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionSealWithoutCompactDensorDigestV1ModeIdentity
        )),
        "verifier did NOT reject promotion seal with wrong digest_mode_wire_name"
    );
}

// ---------------------------------------------------------------
// 4 panel-required structural-defect rules
// ---------------------------------------------------------------

#[test]
fn structural_rejects_empty_report_id() {
    let mut r = canonical_report();
    r.report_id = "";
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter()
            .any(|e| matches!(e.kind, SPerf12PromotionVerifyErrorKind::ReportIdEmpty)),
        "verifier did NOT reject empty report_id"
    );
}

#[test]
fn structural_rejects_corpus_hash_missing() {
    let mut r = canonical_report();
    r.corpus_hash_v1 = [0u8; 32];
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter()
            .any(|e| matches!(e.kind, SPerf12PromotionVerifyErrorKind::CorpusHashMissing)),
        "verifier did NOT reject zero corpus_hash_v1"
    );
}

#[test]
fn structural_rejects_promotion_gate_value_drifted() {
    let mut r = canonical_report();
    r.fields.promotion_gate_centi_gbps = 1_900;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionGateValueDrifted
        )),
        "verifier did NOT reject promotion_gate_centi_gbps drift away from 2000"
    );
}

#[test]
fn structural_rejects_promotion_gate_passed_arithmetic_mismatch() {
    let mut r = canonical_report();
    // band_min = 2002 > gate = 2000, so gate_passed = true.
    // Force the flag to false to trigger the arithmetic-
    // mismatch rule.
    r.fields.promotion_gate_passed = false;
    let errs = verify_promotion_report(&r);
    assert!(
        errs.iter().any(|e| matches!(
            e.kind,
            SPerf12PromotionVerifyErrorKind::PromotionGatePassedArithmeticMismatch
        )),
        "verifier did NOT reject promotion_gate_passed flag inconsistent with arithmetic"
    );
}

// ---------------------------------------------------------------
// 8 panel-required positives
// ---------------------------------------------------------------

#[test]
fn admits_post_s_perf_14c_promotion_seal_with_2016_centi_gbps_median() {
    let r = canonical_report();
    let errs = verify_promotion_report(&r);
    assert!(
        errs.is_empty(),
        "canonical S-PERF.12 promotion report SHOULD admit; errs={errs:?}"
    );
    assert_eq!(
        r.fields.post_s_perf_14c_bandwidth_band_median_centi_gbps, 2_016,
        "canonical 3-run median MUST be panel-pinned 2016 (= 20.16 GB/s)"
    );
}

#[test]
fn records_pre_s_perf_11_bandwidth_1333_centi_gbps() {
    let f = build_promotion_fields();
    assert_eq!(f.pre_s_perf_11_bandwidth_centi_gbps, 1_333);
    assert_eq!(PRE_S_PERF_11_BANDWIDTH_CENTI_GBPS, 1_333);
}

#[test]
fn records_post_s_perf_11_bandwidth_1638_centi_gbps() {
    let f = build_promotion_fields();
    assert_eq!(f.post_s_perf_11_bandwidth_centi_gbps, 1_638);
    assert_eq!(POST_S_PERF_11_BANDWIDTH_CENTI_GBPS, 1_638);
}

#[test]
fn records_s_perf_12a_candidate_bandwidth_1872_centi_gbps() {
    let f = build_promotion_fields();
    assert_eq!(f.s_perf_12a_candidate_bandwidth_centi_gbps, 1_872);
    assert_eq!(S_PERF_12A_CANDIDATE_BANDWIDTH_CENTI_GBPS, 1_872);
}

#[test]
fn records_post_s_perf_14c_bandwidth_band_2002_to_2122_centi_gbps() {
    let f = build_promotion_fields();
    assert_eq!(f.post_s_perf_14c_bandwidth_band_min_centi_gbps, 2_002);
    assert_eq!(f.post_s_perf_14c_bandwidth_band_max_centi_gbps, 2_122);
    assert_eq!(f.post_s_perf_14c_bandwidth_band_median_centi_gbps, 2_016);
    assert_eq!(POST_S_PERF_14C_BANDWIDTH_BAND_MIN_CENTI_GBPS, 2_002);
    assert_eq!(POST_S_PERF_14C_BANDWIDTH_BAND_MAX_CENTI_GBPS, 2_122);
    assert_eq!(POST_S_PERF_14C_BANDWIDTH_BAND_MEDIAN_CENTI_GBPS, 2_016);
}

#[test]
fn binds_s_perf_11_bandwidth_delta_report_hash() {
    let r = canonical_report();
    assert_ne!(
        r.s_perf_11_bandwidth_delta_report_hash_v1, [0u8; 32],
        "promotion report MUST bind a non-zero S-PERF.11 anchor"
    );
}

#[test]
fn preserves_compact_densor_digest_v1_non_aliasing_law() {
    let f = build_promotion_fields();
    assert_eq!(
        f.digest_mode_wire_name,
        COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME
    );
    assert_eq!(f.digest_mode_wire_name, "CompactDensorDigestV1");
    assert!(
        !f.tree_sha256v1_root_aliasing,
        "panel-locked: TreeSha256V1 aliasing MUST be false per S-PERF.10 non-aliasing law"
    );
    assert!(
        f.audit_mode_unchanged,
        "panel-locked: Audit-mode (SerialSha256) MUST be byte-identical end-to-end"
    );
}

#[test]
fn hashes_are_deterministic_across_two_seeds() {
    let r1 = canonical_report();
    let r2 = canonical_report();
    assert_eq!(
        r1.fields.s_perf_12_promotion_fields_hash_v1,
        r2.fields.s_perf_12_promotion_fields_hash_v1
    );
    assert_eq!(
        r1.s_perf_12_promotion_report_hash_v1,
        r2.s_perf_12_promotion_report_hash_v1
    );
}

// ---------------------------------------------------------------
// Hash sensitivity tests (changing ONE field must change the
// META-hash — the promotion-report hash is load-bearing for
// the Track B completion seal).
// ---------------------------------------------------------------

#[test]
fn changing_post_s_perf_14c_band_min_changes_fields_hash() {
    let baseline = build_promotion_fields();
    let mut mutated = baseline.clone();
    mutated.post_s_perf_14c_bandwidth_band_min_centi_gbps = 2_003;
    let mutated_hash = compute_promotion_fields_hash(&mutated);
    assert_ne!(
        baseline.s_perf_12_promotion_fields_hash_v1, mutated_hash,
        "fields hash should change when band_min changes"
    );
}

#[test]
fn changing_digest_mode_wire_name_changes_fields_hash() {
    let baseline = build_promotion_fields();
    let mut mutated = baseline.clone();
    mutated.digest_mode_wire_name = "SerialSha256";
    let mutated_hash = compute_promotion_fields_hash(&mutated);
    assert_ne!(
        baseline.s_perf_12_promotion_fields_hash_v1, mutated_hash,
        "fields hash should change when digest_mode_wire_name changes"
    );
}

#[test]
fn changing_audit_mode_unchanged_flag_changes_fields_hash() {
    let baseline = build_promotion_fields();
    let mut mutated = baseline.clone();
    mutated.audit_mode_unchanged = false;
    let mutated_hash = compute_promotion_fields_hash(&mutated);
    assert_ne!(
        baseline.s_perf_12_promotion_fields_hash_v1, mutated_hash,
        "fields hash should change when audit_mode_unchanged flag flips"
    );
}

#[test]
fn changing_any_anchor_changes_report_hash() {
    let r = canonical_report();
    let mut mutated = r.clone();
    mutated.s_perf_11_bandwidth_delta_report_hash_v1 = [0xffu8; 32];
    let mutated_hash = compute_promotion_report_hash(&mutated);
    assert_ne!(
        r.s_perf_12_promotion_report_hash_v1, mutated_hash,
        "report hash should change when S-PERF.11 anchor changes"
    );

    let mut mutated = r.clone();
    mutated.s_perf_11_1_bottleneck_triage_hash_v1 = [0xeeu8; 32];
    let mutated_hash = compute_promotion_report_hash(&mutated);
    assert_ne!(
        r.s_perf_12_promotion_report_hash_v1, mutated_hash,
        "report hash should change when S-PERF.11.1 anchor changes"
    );

    let mut mutated = r.clone();
    mutated.corpus_hash_v1 = [0xddu8; 32];
    let mutated_hash = compute_promotion_report_hash(&mutated);
    assert_ne!(
        r.s_perf_12_promotion_report_hash_v1, mutated_hash,
        "report hash should change when corpus_hash_v1 changes"
    );
}

#[test]
fn changing_commit_sha_provenance_changes_report_hash() {
    let r = canonical_report();
    let mut mutated = r.clone();
    mutated.s_perf_14c_commit_sha = "deadbeef";
    let mutated_hash = compute_promotion_report_hash(&mutated);
    assert_ne!(
        r.s_perf_12_promotion_report_hash_v1, mutated_hash,
        "report hash should change when S-PERF.14c commit-sha provenance changes"
    );
}

// ---------------------------------------------------------------
// Render byte-stability + epis structural pin
// ---------------------------------------------------------------

#[test]
fn text_renderer_is_byte_stable() {
    let r = canonical_report();
    let t1 = render_promotion_report_text(&r);
    let t2 = render_promotion_report_text(&r);
    assert_eq!(t1, t2, "text renderer must be byte-stable across two calls");
}

#[test]
fn json_renderer_is_byte_stable() {
    let r = canonical_report();
    let j1 = render_promotion_report_json(&r);
    let j2 = render_promotion_report_json(&r);
    assert_eq!(j1, j2, "JSON renderer must be byte-stable across two calls");
}

#[test]
fn text_renderer_contains_panel_pinned_summary() {
    let r = canonical_report();
    let t = render_promotion_report_text(&r);
    assert!(t.contains("CompactDensorDigestV1 throughput-mode promotion receipt"));
    assert!(t.contains("20.02"));
    assert!(t.contains("20.16"));
    assert!(t.contains("21.22"));
    assert!(t.contains("13.33"));
    assert!(t.contains("16.38"));
    assert!(t.contains("18.72"));
    assert!(t.contains("CompactDensorDigestV1"));
    assert!(t.contains("digest_mode_non_aliasing_law"));
}

#[test]
fn report_pins_panel_locked_commit_sha_chain() {
    let r = canonical_report();
    assert_eq!(r.s_perf_11_commit_sha, S_PERF_11_COMMIT_SHA);
    assert_eq!(r.s_perf_11_commit_sha, "3e67cb4");
    assert_eq!(r.s_perf_14b_commit_sha, S_PERF_14B_COMMIT_SHA);
    assert_eq!(r.s_perf_14b_commit_sha, "e1dcf54");
    assert_eq!(r.s_perf_14c_commit_sha, S_PERF_14C_COMMIT_SHA);
    assert_eq!(r.s_perf_14c_commit_sha, "795d0f9");
}

#[test]
fn r12b_episode_pin_constants_match_panel_locked_triple() {
    assert_eq!(R12B_EPISODE_COUNT_CANONICAL_W16H128, 13);
    assert_eq!(R12B_EPISODE_COUNT_MID_W64H512, 89);
    assert_eq!(R12B_EPISODE_COUNT_FULL_W256H4096, 1_917);
}

// ---------------------------------------------------------------
// Live disk seed: pinned-hash back-stop. Reads the live
// pinned post-S-PERF.11 source-report file + the live
// post-S-PERF.11.1 triage file from the repo root and
// asserts the seeded promotion report admits and binds
// non-zero S-PERF.11 + S-PERF.11.1 + corpus anchors.
// ---------------------------------------------------------------

#[test]
fn seeds_from_disk_against_live_pinned_reports() {
    // The test crate's working directory at runtime is the
    // package root (`crates/dsfb-gpu-atlas-corpus/`); the
    // upstream seeds expect the repo root, so walk two
    // levels up.
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let r = seed_s_perf_12_promotion_report_from_disk(&repo_root)
        .expect("seed should succeed against live pinned reports");
    let errs = verify_promotion_report(&r);
    assert!(
        errs.is_empty(),
        "seeded S-PERF.12 promotion report should admit (errs={errs:?})"
    );
    assert_ne!(r.s_perf_11_bandwidth_delta_report_hash_v1, [0u8; 32]);
    assert_ne!(r.s_perf_11_1_bottleneck_triage_hash_v1, [0u8; 32]);
    assert_ne!(r.corpus_hash_v1, [0u8; 32]);
    assert_ne!(r.s_perf_12_promotion_report_hash_v1, [0u8; 32]);
}
