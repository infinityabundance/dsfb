//! Integration tests for the Chemical Court Record v1 bundle (`court_record` module).
//!
//! These assert the canonical-artifact contract: the bundle contains *exactly* the documented files,
//! the manifest is well-formed and self-describing, the bundle hash is deterministic, every episode
//! carries a known claim-boundary badge, and the non-claims statement is present.

use dsfb_chemical_engineering_edge as edge;
use edge::pipeline::PipelineConfig;
use edge::{court_record, datasets, pipeline};
use std::path::{Path, PathBuf};

/// Path to a committed CSV slice (tests run with the crate dir as `CARGO_MANIFEST_DIR`).
fn slice_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("slices")
        .join(format!("{name}.csv"))
}

/// Analyse a committed dataset and write its court record into `out`; return the bundle root.
fn build_bundle(name: &str, out: &Path) -> String {
    let (m, n_base) = datasets::load_csv_slice(&slice_path(name), 0.4).expect("load slice");
    let cfg = PipelineConfig::default();
    let res = pipeline::analyze(name, "measured/slice", &m, n_base, cfg);
    let timelines = pipeline::timelines_for(name, &m, n_base, cfg);
    court_record::write_court_record(out, &res, &timelines, n_base, cfg.fusion)
        .expect("write bundle")
}

/// Unique, self-cleaning temp directory per test (cargo runs tests in one process, so the prefix
/// must differ per test to avoid collisions).
fn fresh(prefix: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("{prefix}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn bundle_has_exactly_the_canonical_files() {
    let out = fresh("dsfb_cr_files");
    build_bundle("tennessee_eastman_idv01", &out);
    let dir = out.join(court_record::CASEFILE_FORMAT);
    let mut got: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    got.sort();
    let mut want: Vec<String> = court_record::CONTENT_FILES
        .iter()
        .map(|s| s.to_string())
        .collect();
    want.push("casefile.json".into());
    want.sort();
    assert_eq!(
        got, want,
        "bundle must contain exactly the 11 canonical files"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn casefile_manifest_is_well_formed() {
    let out = fresh("dsfb_cr_manifest");
    build_bundle("wine_quality_red", &out);
    let dir = out.join(court_record::CASEFILE_FORMAT);
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("casefile.json")).unwrap()).unwrap();

    assert_eq!(json["format"], court_record::CASEFILE_FORMAT);
    assert_eq!(json["format_version"], 1);
    // The manifest hashes the ten content files (casefile.json excluded from its own list).
    assert_eq!(json["files"].as_array().unwrap().len(), 10);
    // evidence_root.txt mirrors the manifest evidence_root exactly.
    let er = std::fs::read_to_string(dir.join("evidence_root.txt")).unwrap();
    assert_eq!(er.trim(), json["evidence_root"].as_str().unwrap());
    assert_eq!(json["bundle_root"].as_str().unwrap().len(), 64);
    // The court never admits a root-cause claim — this global badge must always be present.
    assert!(json["global_badges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b == "ROOT_CAUSE_NOT_ADMITTED"));
    // Every per-episode badge must be a known, self-bounding token.
    let valid = [
        "STRUCTURE_ONLY",
        "CANDIDATE_FAULT",
        "NEAR_MISS",
        "SENSOR_QUALITY",
        "CONTROL_CONTEXT_REQUIRED",
        "PHYSICS_WITNESS_REQUIRED",
    ];
    // (P93) Every per-episode badge also carries a typed EvidenceKind tag (what kind of evidence backs it).
    let valid_kinds = [
        "physical_balance",
        "first_principles_equation",
        "instrumentation_health",
        "controller_context",
        "process_topology",
        "dataset_quality",
        "historian_import",
        "chemometric_detector",
        "heuristic_pattern",
        "precedent_similarity",
        "operator_annotation",
        "narrative_summary",
    ];
    for b in json["episode_badges"].as_array().unwrap() {
        let t = b["badge"].as_str().unwrap();
        assert!(valid.contains(&t), "unexpected episode badge: {t}");
        let k = b["evidence_kind"]
            .as_str()
            .expect("every badge carries an evidence_kind (P93)");
        assert!(valid_kinds.contains(&k), "unexpected evidence_kind: {k}");
    }
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn bundle_root_is_deterministic() {
    let a = fresh("dsfb_cr_det_a");
    let b = fresh("dsfb_cr_det_b");
    let ra = build_bundle("cstr_reactor", &a);
    let rb = build_bundle("cstr_reactor", &b);
    assert_eq!(
        ra, rb,
        "bundle_root must be identical across runs over the same analysis"
    );
    assert_eq!(ra.len(), 64);
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}

/// Frozen golden gate for the Chemical Court Record `bundle_root` — the cross-run analogue of the
/// `bundle_root_is_deterministic` smoke check. The bundle is a pure function of the analysis (no
/// timestamps, no paths — see the `court_record` module doc), so on the pinned toolchain its root is a
/// stable constant. Pinning it makes a cross-compiler / codegen drift in the court bundle (e.g. the
/// signed-zero CSV-formatting seam closed in P46) fail loudly, the same way `golden_replay.rs` pins the
/// `replay_hash` and `atlas_gates.rs` pins `atlas_hash_v1`. Re-freeze deliberately if the bundle format
/// or the committed `tennessee_eastman_idv01` slice changes.
// Governed re-freeze (Wave-1 A3, 2026-05-25): the operator_report.html now carries a top claim-boundary
// banner + claim-strength legend; the HTML is one of the ten hashed CONTENT_FILES, so every bundle_root
// shifted (the evidence_root / replay_hash did NOT — court fields are not in canonical_replay_hash).
// GOVERNED RE-FREEZE (Phase-C executable, 2026-05-26): the executed detector bank grew 14 → 18, changing
// the fused episodes / detector evidence in every casefile, so every bundle_root + evidence_root shifted.
// GOVERNED RE-FREEZE (P83 universal operator legend, 2026-05-26): operator_report.html gained the universal
// claim legend (claim tier · witness-strength ladder · evidence kind · unknown route) + a per-episode
// witness-strength column; the HTML is a hashed CONTENT_FILE, so every bundle_root shifted again. The
// evidence_root / replay_hash did NOT move (court fields are not in canonical_replay_hash) → verify-replay 6/6.
// GOVERNED RE-FREEZE (P85 per-episode EvidenceKind column, 2026-05-27): operator_report.html gained a typed
// per-episode "Evidence kind" column (EvidenceKind::from_witness_strength); the HTML is a hashed CONTENT_FILE,
// so every bundle_root shifted again. evidence_root / replay_hash did NOT move (court fields are not in
// canonical_replay_hash) → verify-replay 6/6; all 20 evidence_roots byte-identical (asserted by the regen).
// GOVERNED RE-FREEZE (P102 per-episode ClaimStrength + EvidenceAnchor columns, 2026-05-28): operator_report.html
// gained two more per-episode columns — the claim-strength tier (derived from the evidence kind) and a per-episode
// evidence-anchor digest. Same mechanism: the HTML is a hashed CONTENT_FILE, so every bundle_root shifted again
// while every evidence_root is byte-UNCHANGED (verified: 20/20 evidence_roots identical; verify-replay 6/6).
// NB: this golden is the TEST build_bundle path (load_csv_slice @ 0.4 baseline, "measured/slice") — distinct
// by construction from the demo/EXPECTED_BUNDLE_ROOTS idv01 value (different baseline fraction + kind), so the
// two idv01 bundle_roots have always differed; both are display-only re-mints under P102.
// GOVERNED RE-FREEZE (panel B6-v1 evidence_kind in admitted_episodes.csv, 2026-05-28): the claim-audit CSV gained a
// per-episode "evidence_kind" column (EvidenceKind::from_witness_strength, identical to the casefile.json EpisodeBadge);
// admitted_episodes.csv is one of the ten hashed CONTENT_FILES, so every bundle_root shifted again while every
// evidence_root is byte-UNCHANGED (20/20 evidence_roots identical; verify-replay 6/6).
const GOLDEN_BUNDLE_ROOT_IDV01: &str =
    "b71c1ad7f86008458fe23969c2f2ccbdc7c408403b9a00957916cc1f28f959db";

#[test]
fn bundle_root_matches_frozen_golden() {
    let out = fresh("dsfb_cr_golden");
    let root = build_bundle("tennessee_eastman_idv01", &out);
    assert_eq!(
        root, GOLDEN_BUNDLE_ROOT_IDV01,
        "bundle_root drift for tennessee_eastman_idv01 — a court-bundle regression (CSV formatting, file \
         set, or pipeline) OR an intended change (if intended, re-freeze the hex)"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn non_claims_carries_the_key_phrase() {
    let out = fresh("dsfb_cr_nc");
    build_bundle("tennessee_eastman_idv01", &out);
    let nc = std::fs::read_to_string(
        out.join(court_record::CASEFILE_FORMAT)
            .join("non_claims.md"),
    )
    .unwrap();
    assert!(
        nc.contains("does not emit an alarm"),
        "key framing phrase must be present"
    );
    assert!(nc.contains("ROOT_CAUSE_NOT_ADMITTED"));
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn rejection_vocabulary_grouping_is_explicit() {
    use court_record::RejectionReason as R;
    // The reserved (not-yet-emitted) group is exactly the seven non-quorum variants, and the one
    // emitted reason (QuorumNotMet) is deliberately excluded from it. This locks the documented
    // split so a future edit cannot silently move a variant between "emitted" and "reserved".
    assert_eq!(
        R::RESERVED.len(),
        7,
        "schema v1 reserves seven forthcoming rejection reasons"
    );
    assert!(
        !R::RESERVED.contains(&R::QuorumNotMet),
        "QuorumNotMet is the emitted reason, not reserved"
    );
    // Every variant — emitted or reserved — carries a stable, distinct, non-empty UPPER_SNAKE token.
    let mut tokens: Vec<&str> = std::iter::once(R::QuorumNotMet.token())
        .chain(R::RESERVED.iter().map(|r| r.token()))
        .collect();
    assert!(
        tokens
            .iter()
            .all(|t| !t.is_empty() && t == &t.to_uppercase()),
        "tokens are non-empty UPPER_SNAKE"
    );
    tokens.sort_unstable();
    let n = tokens.len();
    tokens.dedup();
    assert_eq!(tokens.len(), n, "every rejection-reason token is unique");
}
