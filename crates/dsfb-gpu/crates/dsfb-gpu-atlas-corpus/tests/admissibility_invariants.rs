//! T.11c acceptance tests for the admissibility grammar.
//!
//! Panel-required invariants (26+):
//!
//! - `grammar_snapshot_is_deterministic`
//! - `grammar_hash_is_deterministic`
//! - `grammar_hash_changes_when_rule_changes`
//! - `grammar_hash_changes_when_precedent_link_changes`
//! - `every_grammar_rule_links_precedent`
//! - `every_witness_law_precedent_is_consumed`
//! - `every_negative_witness_law_precedent_is_consumed`
//! - `primary_witness_rule_requires_positive_structural_support`
//! - `clean_window_witness_cannot_admit_alone`
//! - `confuser_witness_cannot_admit_alone`
//! - `boundary_witness_cannot_classify_episode_alone`
//! - `recovery_witness_cannot_originate_episode_alone`
//! - `negative_witness_blocks_or_downgrades_admission`
//! - `gpu_evidence_cannot_mint_admission`
//! - `semantic_non_bypass_rule_is_present`
//! - `bank_admission_token_rule_is_present`
//! - `unknown_or_deferred_outcome_is_explicit`
//! - `grammar_json_render_is_deterministic`
//! - `grammar_text_render_is_deterministic`
//! - `crosswalk_is_deterministic`
//! - `no_publication_language_in_t11c_reports`
//! - `corpus_hash_v1_unchanged`
//! - `registry_hash_v2_unchanged_does_not_apply_to_corpus_crate` (note)
//! - `precedent_hash_v1_unchanged`
//! - **Negative**: `grammar_verifier_rejects_rule_without_precedent_link`
//! - **Negative**: `grammar_verifier_rejects_episode_admission_rule_that_allows_confuser_only_admission`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::admissibility::{
    build_crosswalk, collect_admissibility_grammar, compute_admissibility_grammar_hash_v1,
    precedent_ids_by_kind, render_crosswalk_json, render_crosswalk_text, render_grammar_json,
    render_grammar_text, verify_grammar_snapshot, ConfuserEffect, EpisodeAdmissibilityRule,
    EvidenceRequirement, GrammarRuleId, GrammarRuleKind, GrammarRuleSeverity,
    GrammarVerifyErrorKind,
};
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::passport::all_passports;
use dsfb_gpu_atlas_corpus::precedent::{collect_court_precedents, PrecedentKind};

const T10_CORPUS_HASH: &str = "35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7";
const T11B_PRECEDENT_HASH: &str =
    "6721f511f1eb951ba7eff4fa36832f233331507f6e4208d4f97866afd984dd14";

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn grammar_snapshot_is_deterministic() {
    let a = collect_admissibility_grammar();
    let b = collect_admissibility_grammar();
    assert_eq!(
        a.admissibility_grammar_hash_v1, b.admissibility_grammar_hash_v1,
        "two builds must produce byte-identical admissibility_grammar_hash_v1"
    );
    assert_eq!(a.admission_rules.len(), b.admission_rules.len());
    assert_eq!(a.confuser_rules.len(), b.confuser_rules.len());
    for (x, y) in a.admission_rules.iter().zip(b.admission_rules.iter()) {
        assert_eq!(x.id, y.id);
        assert_eq!(x.kind, y.kind);
        assert_eq!(x.name, y.name);
        assert_eq!(x.linked_precedent_ids, y.linked_precedent_ids);
    }
}

#[test]
fn grammar_hash_is_deterministic() {
    let s = collect_admissibility_grammar();
    let h = compute_admissibility_grammar_hash_v1(&s);
    assert_eq!(
        h, s.admissibility_grammar_hash_v1,
        "compute_admissibility_grammar_hash_v1 must match the embedded hash"
    );
}

#[test]
fn grammar_hash_changes_when_rule_changes() {
    let mut s = collect_admissibility_grammar();
    let baseline = s.admissibility_grammar_hash_v1;
    s.admission_rules[0].witness_requirement.min_primary = 99;
    let mutated = compute_admissibility_grammar_hash_v1(&s);
    assert_ne!(
        baseline, mutated,
        "grammar hash must change when any rule field changes"
    );
}

#[test]
fn grammar_hash_changes_when_precedent_link_changes() {
    let mut s = collect_admissibility_grammar();
    let baseline = s.admissibility_grammar_hash_v1;
    // Replace the first rule's precedent links with a different
    // (still-valid) precedent id.
    let precedents = collect_court_precedents();
    let some_other_id = precedents
        .precedents
        .iter()
        .find(|p| !s.admission_rules[0].linked_precedent_ids.contains(&p.id))
        .map(|p| p.id)
        .expect("a precedent not currently linked exists");
    s.admission_rules[0].linked_precedent_ids = vec![some_other_id];
    let mutated = compute_admissibility_grammar_hash_v1(&s);
    assert_ne!(
        baseline, mutated,
        "grammar hash must change when precedent linkage changes"
    );
}

#[test]
fn every_grammar_rule_links_precedent() {
    let s = collect_admissibility_grammar();
    for r in &s.admission_rules {
        assert!(
            !r.linked_precedent_ids.is_empty(),
            "admission rule {:?} has no linked precedents",
            r.id
        );
    }
    for r in &s.confuser_rules {
        assert!(
            !r.linked_precedent_ids.is_empty(),
            "confuser rule {:?} has no linked precedents",
            r.id
        );
    }
}

fn rule_links_precedent(
    s: &dsfb_gpu_atlas_corpus::admissibility::AdmissibilityGrammarSnapshot,
    pid: dsfb_gpu_atlas_corpus::precedent::PrecedentId,
) -> bool {
    s.admission_rules
        .iter()
        .any(|r| r.linked_precedent_ids.contains(&pid))
        || s.confuser_rules
            .iter()
            .any(|r| r.linked_precedent_ids.contains(&pid))
}

#[test]
fn every_witness_law_precedent_is_consumed() {
    let p = collect_court_precedents();
    let s = collect_admissibility_grammar();
    let witness_law_ids = precedent_ids_by_kind(&p, PrecedentKind::WitnessLaw);
    assert!(
        !witness_law_ids.is_empty(),
        "T.11b must declare WitnessLaw precedents"
    );
    for id in witness_law_ids {
        assert!(
            rule_links_precedent(&s, id),
            "WitnessLaw precedent {id:?} is not consumed by any grammar rule"
        );
    }
}

#[test]
fn every_negative_witness_law_precedent_is_consumed() {
    let p = collect_court_precedents();
    let s = collect_admissibility_grammar();
    let neg_ids = precedent_ids_by_kind(&p, PrecedentKind::NegativeWitnessLaw);
    assert!(
        !neg_ids.is_empty(),
        "T.11b must declare NegativeWitnessLaw precedents"
    );
    for id in neg_ids {
        assert!(
            rule_links_precedent(&s, id),
            "NegativeWitnessLaw precedent {id:?} is not consumed by any grammar rule"
        );
    }
}

fn find_admission_by_name<'a>(
    s: &'a dsfb_gpu_atlas_corpus::admissibility::AdmissibilityGrammarSnapshot,
    name: &str,
) -> &'a EpisodeAdmissibilityRule {
    s.admission_rules
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("admission rule `{name}` must be present"))
}

#[test]
fn primary_witness_rule_requires_positive_structural_support() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "PrimaryWitnessRequiresPositiveSupport");
    assert!(r.witness_requirement.min_primary >= 1);
    assert!(r.witness_requirement.min_corroborating >= 1);
    assert!(r.witness_requirement.forbids_confuser_only);
    assert!(r
        .evidence_requirements
        .contains(&EvidenceRequirement::AtLeastOneCorroboratingWitness));
}

#[test]
fn clean_window_witness_cannot_admit_alone() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "CleanWindowWitnessCannotAdmitAlone");
    assert!(r.witness_requirement.forbids_clean_window_only);
    assert_eq!(r.kind, GrammarRuleKind::CleanWindowSupport);
}

#[test]
fn confuser_witness_cannot_admit_alone() {
    let s = collect_admissibility_grammar();
    // NegativeWitnessRejection rule pins this directly.
    let r = find_admission_by_name(&s, "NegativeWitnessBlocksAdmissionUnlessBankOverride");
    assert!(r.witness_requirement.forbids_confuser_only);
    assert!(r
        .evidence_requirements
        .contains(&EvidenceRequirement::NoConfuserOnlyAdmission));
}

#[test]
fn boundary_witness_cannot_classify_episode_alone() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "BoundaryWitnessCannotClassifyAlone");
    assert!(r.witness_requirement.forbids_boundary_only);
    assert_eq!(r.kind, GrammarRuleKind::BoundaryCondition);
}

#[test]
fn recovery_witness_cannot_originate_episode_alone() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "RecoveryWitnessCannotOriginateAlone");
    assert!(r.witness_requirement.forbids_recovery_only);
    assert_eq!(r.kind, GrammarRuleKind::RecoveryClosure);
}

#[test]
fn negative_witness_blocks_or_downgrades_admission() {
    let s = collect_admissibility_grammar();
    assert!(
        !s.confuser_rules.is_empty(),
        "T.11c must declare at least one confuser-suppression rule"
    );
    for r in &s.confuser_rules {
        assert!(
            matches!(
                r.effect,
                ConfuserEffect::BlockAdmission
                    | ConfuserEffect::DowngradeAdmission
                    | ConfuserEffect::QuarantineEpisode
            ),
            "confuser rule {:?} must declare Block / Downgrade / Quarantine effect",
            r.id
        );
    }
}

#[test]
fn gpu_evidence_cannot_mint_admission() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "GpuOutputIsEvidenceOnly");
    assert!(r.witness_requirement.gpu_output_is_evidence_only);
    assert!(r
        .evidence_requirements
        .contains(&EvidenceRequirement::GpuOutputIsEvidenceOnly));
}

#[test]
fn semantic_non_bypass_rule_is_present() {
    let s = collect_admissibility_grammar();
    let count = s
        .admission_rules
        .iter()
        .filter(|r| r.kind == GrammarRuleKind::SemanticNonBypass)
        .count();
    assert!(
        count >= 1,
        "T.11c must declare at least one SemanticNonBypass rule"
    );
}

#[test]
fn bank_admission_token_rule_is_present() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "BankAdmissionTokenIsTheOnlyAdmissionRoute");
    assert!(r.witness_requirement.requires_bank_admission_token);
    assert!(r
        .evidence_requirements
        .contains(&EvidenceRequirement::BankAdmissionTokenRequired));
}

#[test]
fn unknown_or_deferred_outcome_is_explicit() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "UnknownOrDeferredOutcomeIsExplicit");
    assert_eq!(r.kind, GrammarRuleKind::DeferredUnknown);
    assert_eq!(r.severity, GrammarRuleSeverity::Deferred);
    assert!(r
        .evidence_requirements
        .contains(&EvidenceRequirement::DeferredUnknownIsExplicit));
}

#[test]
fn grammar_text_render_is_deterministic() {
    let s = collect_admissibility_grammar();
    let a = render_grammar_text(&s);
    let b = render_grammar_text(&s);
    assert_eq!(a, b);
}

#[test]
fn grammar_json_render_is_deterministic() {
    let s = collect_admissibility_grammar();
    let a = render_grammar_json(&s);
    let b = render_grammar_json(&s);
    assert_eq!(a, b);
}

#[test]
fn crosswalk_is_deterministic() {
    let s = collect_admissibility_grammar();
    let a = build_crosswalk(&s);
    let b = build_crosswalk(&s);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.canonical_id, y.canonical_id);
        assert_eq!(x.linked_precedent_ids, y.linked_precedent_ids);
        assert_eq!(x.linked_grammar_rule_ids, y.linked_grammar_rule_ids);
    }
    let ta = render_crosswalk_text(&a);
    let tb = render_crosswalk_text(&a);
    assert_eq!(ta, tb);
    let ja = render_crosswalk_json(&a);
    let jb = render_crosswalk_json(&a);
    assert_eq!(ja, jb);
}

#[test]
fn crosswalk_covers_every_passport() {
    let s = collect_admissibility_grammar();
    let cw = build_crosswalk(&s);
    let passports = all_passports();
    assert_eq!(
        cw.len(),
        passports.len(),
        "crosswalk MUST have exactly one row per passport"
    );
    for row in &cw {
        assert!(
            !row.linked_grammar_rule_ids.is_empty(),
            "every passport's crosswalk row MUST link at least one grammar rule (via global laws); canonical_id {:?}",
            row.canonical_id
        );
    }
}

#[test]
fn no_publication_language_in_t11c_reports() {
    let s = collect_admissibility_grammar();
    let text = render_grammar_text(&s);
    let json = render_grammar_json(&s);
    let cw_text = render_crosswalk_text(&build_crosswalk(&s));
    let cw_json = render_crosswalk_json(&build_crosswalk(&s));
    let forbidden = [
        "Zenodo",
        "DOI",
        "publication-grade",
        "peer-reviewed",
        "ready for publication",
    ];
    for body in [&text, &json, &cw_text, &cw_json] {
        for word in forbidden {
            assert!(
                !body.contains(word),
                "T.11c artifact contains forbidden publication-language `{word}`"
            );
        }
    }
}

#[test]
fn corpus_hash_v1_unchanged() {
    let live = compute_corpus_hash_v1();
    assert_eq!(
        hex(&live.bytes),
        T10_CORPUS_HASH,
        "T.11c must NOT mutate corpus_hash_v1; T.10 freeze is permanent"
    );
}

#[test]
fn precedent_hash_v1_unchanged() {
    let p = collect_court_precedents();
    assert_eq!(
        hex(&p.precedent_hash_v1),
        T11B_PRECEDENT_HASH,
        "T.11c must NOT mutate precedent_hash_v1; T.11b freeze is permanent"
    );
}

#[test]
fn verify_grammar_snapshot_admits_live_collection() {
    let s = collect_admissibility_grammar();
    let p = collect_court_precedents();
    let errors = verify_grammar_snapshot(&s, &p);
    assert!(
        errors.is_empty(),
        "verify_grammar_snapshot MUST admit the live collection; got {errors:?}"
    );
}

#[test]
fn grammar_verifier_rejects_rule_without_precedent_link() {
    let mut s = collect_admissibility_grammar();
    let p = collect_court_precedents();
    // Forge a rule with empty linked_precedent_ids.
    s.admission_rules[0].linked_precedent_ids = Vec::new();
    let errors = verify_grammar_snapshot(&s, &p);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == GrammarVerifyErrorKind::RuleWithoutPrecedentLink),
        "verifier MUST reject a rule with no precedent link; got {errors:?}"
    );
}

#[test]
fn grammar_verifier_rejects_episode_admission_rule_that_allows_confuser_only_admission() {
    let mut s = collect_admissibility_grammar();
    let p = collect_court_precedents();
    // Find any EpisodeAdmission rule and forge it to allow
    // confuser-only admission by clearing both the witness flag
    // and the evidence-requirement clause.
    let idx = s
        .admission_rules
        .iter()
        .position(|r| r.kind == GrammarRuleKind::EpisodeAdmission)
        .expect("at least one EpisodeAdmission rule");
    s.admission_rules[idx]
        .witness_requirement
        .forbids_confuser_only = false;
    s.admission_rules[idx]
        .evidence_requirements
        .retain(|e| *e != EvidenceRequirement::NoConfuserOnlyAdmission);
    let errors = verify_grammar_snapshot(&s, &p);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == GrammarVerifyErrorKind::EpisodeAdmissionAllowsConfuserOnly),
        "verifier MUST reject an EpisodeAdmission rule that allows confuser-only admission; got {errors:?}"
    );
}

#[test]
fn grammar_panel_locked_counts_at_t11c() {
    // 9 admission rules + 9 confuser-suppression rules (one per
    // NegativeWitnessKind variant excluding NotANegativeWitness).
    let s = collect_admissibility_grammar();
    assert_eq!(
        s.admission_rules.len(),
        9,
        "T.11c admission-rule count is panel-locked at 9"
    );
    assert_eq!(
        s.confuser_rules.len(),
        9,
        "T.11c confuser-suppression-rule count is panel-locked at 9 (one per NegativeWitnessKind variant)"
    );
}

#[test]
fn grammar_rule_ids_are_unique_and_dense() {
    use std::collections::BTreeSet;
    let s = collect_admissibility_grammar();
    let mut all_ids: BTreeSet<u32> = BTreeSet::new();
    for r in &s.admission_rules {
        all_ids.insert(r.id.0);
    }
    for r in &s.confuser_rules {
        all_ids.insert(r.id.0);
    }
    assert_eq!(
        all_ids.len(),
        s.admission_rules.len() + s.confuser_rules.len(),
        "all grammar rule ids must be unique"
    );
    // Dense 1..=N
    let n = s.admission_rules.len() + s.confuser_rules.len();
    for i in 1..=n {
        assert!(
            all_ids.contains(&(i as u32)),
            "grammar rule ids must be dense 1..={n}; missing {i}"
        );
    }
}

#[test]
fn verifier_rejects_linked_precedent_outside_known_set() {
    use dsfb_gpu_atlas_corpus::precedent::PrecedentId;
    let mut s = collect_admissibility_grammar();
    let p = collect_court_precedents();
    // Cite a precedent id that does not exist in the live set.
    s.admission_rules[0]
        .linked_precedent_ids
        .push(PrecedentId(99999));
    let errors = verify_grammar_snapshot(&s, &p);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == GrammarVerifyErrorKind::LinkedPrecedentMissing),
        "verifier MUST reject a rule citing an unknown precedent id"
    );
}

#[test]
fn verifier_rejects_duplicate_rule_id() {
    let mut s = collect_admissibility_grammar();
    let p = collect_court_precedents();
    // Force two rules to share an id.
    let shared = s.admission_rules[0].id;
    s.admission_rules[1].id = shared;
    let errors = verify_grammar_snapshot(&s, &p);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == GrammarVerifyErrorKind::DuplicateRuleId),
        "verifier MUST reject duplicate rule ids"
    );
}

#[test]
fn confuser_suppression_rules_cover_every_named_negative_witness_kind() {
    use dsfb_gpu_atlas_corpus::types::NegativeWitnessKind;
    let s = collect_admissibility_grammar();
    let expected = [
        NegativeWitnessKind::SmallSampleConfuser,
        NegativeWitnessKind::SingleWindowSpikeConfuser,
        NegativeWitnessKind::PeriodicBoundaryConfuser,
        NegativeWitnessKind::MissingnessArtifactConfuser,
        NegativeWitnessKind::SchemaChangeConfuser,
        NegativeWitnessKind::UnitScaleChangeConfuser,
        NegativeWitnessKind::DeploymentMarkerConfuser,
        NegativeWitnessKind::ClockSkewConfuser,
        NegativeWitnessKind::BatchBoundaryConfuser,
    ];
    for kind in expected {
        assert!(
            s.confuser_rules.iter().any(|r| r.trigger_kind == kind),
            "T.11c must declare a ConfuserSuppressionRule for NegativeWitnessKind::{kind:?}"
        );
    }
}

#[test]
fn passport_hashes_unchanged_by_t11c() {
    // Panel-locked: T.11c MUST NOT churn passport hashes. The
    // passport-grammar linkage is carried by a separate crosswalk
    // artifact, not by extending the DetectorPassport struct.
    // Pin one canonical passport hash byte-for-byte: Shewhart
    // (canonical_id 1).
    use dsfb_gpu_atlas_corpus::passport::passport_for;
    use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;
    let shewhart =
        passport_for(DetectorCanonicalId(1)).expect("Shewhart passport (canonical_id 1)");
    assert_eq!(
        hex(&shewhart.passport_hash),
        "7bbca7282908d41e0a1bcb0a87bd16052b23c7beb5567f4db64c1a825d14c465",
        "T.11c must NOT mutate passport_hash; the T.11b passport bytes are frozen for T.11c"
    );
}

#[test]
fn deferred_unknown_rule_severity_is_deferred() {
    let s = collect_admissibility_grammar();
    let r = find_admission_by_name(&s, "UnknownOrDeferredOutcomeIsExplicit");
    assert_eq!(
        r.severity,
        GrammarRuleSeverity::Deferred,
        "DeferredUnknown rule MUST carry Deferred severity"
    );
}

#[test]
fn admission_rule_ids_precede_confuser_rule_ids() {
    let s = collect_admissibility_grammar();
    let max_admission_id = s.admission_rules.iter().map(|r| r.id.0).max().unwrap_or(0);
    let min_confuser_id = s
        .confuser_rules
        .iter()
        .map(|r| r.id.0)
        .min()
        .unwrap_or(u32::MAX);
    assert!(
        max_admission_id < min_confuser_id,
        "admission rule ids ({max_admission_id} max) must precede confuser rule ids ({min_confuser_id} min); IDs are assigned admission-first then confuser-second so the report and crosswalk are stable"
    );
}

#[test]
fn no_admission_rule_uses_global_witness_flags_alone() {
    // Belt-and-braces variant of the Semantic Non-Bypass test:
    // EVERY EpisodeAdmission rule MUST either set a witness-
    // requirement minimum AND/OR a forbids_* flag AND/OR an
    // explicit evidence requirement. The verifier already
    // enforces the confuser-only-blocked invariant; this test
    // makes sure no admission rule is structurally empty.
    let s = collect_admissibility_grammar();
    for r in &s.admission_rules {
        if r.kind != GrammarRuleKind::EpisodeAdmission {
            continue;
        }
        let w = r.witness_requirement;
        let has_min = w.min_primary >= 1
            || w.min_corroborating >= 1
            || w.min_boundary >= 1
            || w.min_recovery >= 1;
        let has_forbid = w.forbids_confuser_only
            || w.forbids_clean_window_only
            || w.forbids_boundary_only
            || w.forbids_recovery_only;
        let has_evidence = !r.evidence_requirements.is_empty();
        assert!(
            has_min || has_forbid || has_evidence,
            "EpisodeAdmission rule {:?} is structurally empty",
            r.id
        );
    }
}

#[test]
fn confuser_rule_for_single_window_spike_blocks_admission() {
    use dsfb_gpu_atlas_corpus::types::NegativeWitnessKind;
    let s = collect_admissibility_grammar();
    let r = s
        .confuser_rules
        .iter()
        .find(|r| r.trigger_kind == NegativeWitnessKind::SingleWindowSpikeConfuser)
        .expect("SingleWindowSpikeConfuser rule");
    assert_eq!(
        r.effect,
        ConfuserEffect::BlockAdmission,
        "single-window-spike confuser MUST block admission"
    );
}

#[test]
fn forge_unused_imports_to_keep_referenced() {
    // Import-only sanity: the imports listed in the test module
    // header are all referenced. (Cargo would otherwise warn,
    // and the workspace pedantic lints would fail clippy.)
    let _ = GrammarRuleId(0);
}
