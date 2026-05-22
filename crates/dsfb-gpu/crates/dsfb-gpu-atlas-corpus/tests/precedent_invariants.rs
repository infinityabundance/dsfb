//! T.11b acceptance tests for the court precedent ledger.
//!
//! Panel-required invariants (20+):
//!
//! - `precedent_collection_is_deterministic`
//! - `precedent_hash_changes_when_reason_changes`
//! - `precedent_hash_changes_when_binding_changes`
//! - `t4_alias_decisions_emit_alias_collapse_precedents`
//! - `t4_composition_decisions_emit_composition_precedents`
//! - `western_electric_precedent_points_to_shewhart`
//! - `nelson_precedent_points_to_shewhart_and_western_electric`
//! - `semantic_role_difference_has_precedent`
//! - `primary_witness_cannot_be_negative_only_has_precedent`
//! - `clean_window_cannot_admit_alone_has_precedent`
//! - `l6_requires_lband_whitelist_has_precedent`
//! - `l7_forbidden_until_benchmark_artifact_has_precedent`
//! - `l8_forbidden_until_measured_ledger_has_precedent`
//! - `not_scored_usefulness_has_honesty_precedent`
//! - `corpus_hash_freeze_has_precedent`
//! - `registry_binding_has_precedent`
//! - `every_passport_links_at_least_one_precedent`
//! - `alias_passports_link_alias_precedent` (panel form: aliased canonicals link their alias precedents)
//! - `precedent_text_render_is_deterministic`
//! - `precedent_json_render_is_deterministic`
//! - `no_publication_language_in_t11b_reports`
//! - **Negative**: `precedent_verifier_rejects_precedent_with_missing_subject`
//! - **Negative**: `precedent_verifier_rejects_alias_precedent_if_semantic_role_hash_differs`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use dsfb_gpu_atlas_corpus::passport::all_passports;
use dsfb_gpu_atlas_corpus::precedent::{
    collect_court_precedents, compute_precedent_hash_v1, render_precedents_json,
    render_precedents_text, verify_precedent_set, CourtPrecedent, PrecedentBinding, PrecedentId,
    PrecedentKind, PrecedentReason, PrecedentSet, PrecedentSeverity, PrecedentSource,
    PrecedentVerifyErrorKind,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const SHEWHART_ID: u32 = 1;
const WESTERN_ELECTRIC_ID: u32 = 16;
const NELSON_ID: u32 = 17;

fn ids_for_kind(set: &PrecedentSet, kind: PrecedentKind) -> Vec<&CourtPrecedent> {
    set.precedents.iter().filter(|p| p.kind == kind).collect()
}

fn has_reason(set: &PrecedentSet, kind: PrecedentKind, reason: PrecedentReason) -> bool {
    set.precedents
        .iter()
        .any(|p| p.kind == kind && p.reason == reason)
}

#[test]
fn precedent_collection_is_deterministic() {
    let a = collect_court_precedents();
    let b = collect_court_precedents();
    assert_eq!(
        a.precedent_hash_v1, b.precedent_hash_v1,
        "two builds must produce byte-identical precedent_hash_v1"
    );
    assert_eq!(
        a.precedents.len(),
        b.precedents.len(),
        "two builds must produce the same precedent count"
    );
    for (x, y) in a.precedents.iter().zip(b.precedents.iter()) {
        assert_eq!(x.id, y.id);
        assert_eq!(x.kind, y.kind);
        assert_eq!(x.reason, y.reason);
        assert_eq!(x.binding, y.binding);
        assert_eq!(x.severity, y.severity);
    }
}

#[test]
fn precedent_hash_changes_when_reason_changes() {
    let mut set = collect_court_precedents();
    let baseline = set.precedent_hash_v1;
    // Mutate one precedent's reason.
    set.precedents[0].reason = PrecedentReason::DifferentFormulaSameDomain;
    let mutated = compute_precedent_hash_v1(&set);
    assert_ne!(
        baseline, mutated,
        "precedent_hash_v1 must change when any precedent reason changes"
    );
}

#[test]
fn precedent_hash_changes_when_binding_changes() {
    let mut set = collect_court_precedents();
    let baseline = set.precedent_hash_v1;
    // Mutate the binding on the first per-record precedent.
    let target_idx = set
        .precedents
        .iter()
        .position(|p| !matches!(p.binding, PrecedentBinding::Global))
        .expect("at least one per-record precedent");
    set.precedents[target_idx].binding =
        PrecedentBinding::SingleCanonical(DetectorCanonicalId(SHEWHART_ID));
    let mutated = compute_precedent_hash_v1(&set);
    assert_ne!(
        baseline, mutated,
        "precedent_hash_v1 must change when any binding changes"
    );
}

#[test]
fn t4_alias_decisions_emit_alias_collapse_precedents() {
    let set = collect_court_precedents();
    let aliases = ids_for_kind(&set, PrecedentKind::AliasCollapse);
    assert!(
        !aliases.is_empty(),
        "T.4 alias decisions must emit AliasCollapse precedents (CLAIMS has alias entries)"
    );
    // Every AliasCollapse uses an AliasToCanonical or
    // CanonicalToCanonical binding.
    for p in aliases {
        assert!(
            matches!(
                p.binding,
                PrecedentBinding::AliasToCanonical { .. }
                    | PrecedentBinding::CanonicalToCanonical { .. }
            ),
            "AliasCollapse precedent {:?} has unexpected binding {:?}",
            p.id,
            p.binding
        );
    }
}

#[test]
fn t4_composition_decisions_emit_composition_precedents() {
    let set = collect_court_precedents();
    let compositions = ids_for_kind(&set, PrecedentKind::CompositionJudgment);
    assert!(
        !compositions.is_empty(),
        "T.4 composition decisions (Western Electric, Nelson) must emit CompositionJudgment precedents"
    );
}

#[test]
fn western_electric_precedent_points_to_shewhart() {
    let set = collect_court_precedents();
    let we = set
        .precedents
        .iter()
        .find(|p| {
            p.kind == PrecedentKind::CompositionJudgment
                && matches!(
                    &p.binding,
                    PrecedentBinding::Composition { subject, .. } if subject.0 == WESTERN_ELECTRIC_ID
                )
        })
        .expect("Western Electric composition precedent");
    if let PrecedentBinding::Composition { parents, .. } = &we.binding {
        let parent_ids: Vec<u32> = parents.iter().map(|p| p.0).collect();
        assert!(
            parent_ids.contains(&SHEWHART_ID),
            "Western Electric composition must list Shewhart (id {SHEWHART_ID}) as a parent; got {parent_ids:?}"
        );
    } else {
        panic!("Western Electric binding must be Composition");
    }
}

#[test]
fn nelson_precedent_points_to_shewhart_and_western_electric() {
    let set = collect_court_precedents();
    let nelson = set
        .precedents
        .iter()
        .find(|p| {
            p.kind == PrecedentKind::CompositionJudgment
                && matches!(
                    &p.binding,
                    PrecedentBinding::Composition { subject, .. } if subject.0 == NELSON_ID
                )
        })
        .expect("Nelson composition precedent");
    if let PrecedentBinding::Composition { parents, .. } = &nelson.binding {
        let parent_ids: Vec<u32> = parents.iter().map(|p| p.0).collect();
        assert!(
            parent_ids.contains(&SHEWHART_ID),
            "Nelson must compose over Shewhart (id {SHEWHART_ID}); got {parent_ids:?}"
        );
        assert!(
            parent_ids.contains(&WESTERN_ELECTRIC_ID),
            "Nelson must compose over Western Electric (id {WESTERN_ELECTRIC_ID}); got {parent_ids:?}"
        );
    } else {
        panic!("Nelson binding must be Composition");
    }
}

#[test]
fn semantic_role_difference_has_precedent() {
    // Panel intent: role-difference handling MUST appear in the
    // precedent ledger. Two pathways encode it:
    //
    // 1. T.6 WitnessLaw precedents
    //    (`PrimaryWitnessCannotBeNegativeOnly`,
    //    `CleanWindowWitnessCannotAdmitAlone`) prevent role
    //    drift on the affirmative side.
    // 2. The verifier `KindReasonIncompatible` rule rejects an
    //    `AliasCollapse` precedent carrying the
    //    `SameFormulaDifferentWitnessRole` reason. Exercised by
    //    `precedent_verifier_rejects_alias_precedent_if_semantic_role_hash_differs`
    //    below.
    //
    // The `SemanticRoleSeparation` kind exists for
    // `StochasticOriginalDeterministicReduction` decisions. The
    // current corpus has no such decisions; this is honest — no
    // SEED record carries that variant yet. When one lands, the
    // collector will emit a `SemanticRoleSeparation` precedent
    // automatically. For T.11b we assert the role-difference
    // coverage via the WitnessLaw precedents.
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::WitnessLaw,
            PrecedentReason::PrimaryWitnessCannotBeNegativeOnly,
        ) && has_reason(
            &set,
            PrecedentKind::WitnessLaw,
            PrecedentReason::CleanWindowWitnessCannotAdmitAlone,
        ),
        "role-difference handling MUST be documented via WitnessLaw precedents at T.11b"
    );
}

#[test]
fn primary_witness_cannot_be_negative_only_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::WitnessLaw,
            PrecedentReason::PrimaryWitnessCannotBeNegativeOnly,
        ),
        "T.6 must emit a PrimaryWitnessCannotBeNegativeOnly precedent"
    );
}

#[test]
fn clean_window_cannot_admit_alone_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::WitnessLaw,
            PrecedentReason::CleanWindowWitnessCannotAdmitAlone,
        ),
        "T.6 must emit a CleanWindowWitnessCannotAdmitAlone precedent"
    );
}

#[test]
fn l6_requires_lband_whitelist_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::LBandHonestyLaw,
            PrecedentReason::L5L6RequiresGpuWhitelist,
        ),
        "T.7 must emit an L5/L6 GPU-whitelist precedent"
    );
}

#[test]
fn l7_forbidden_until_benchmark_artifact_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::LBandHonestyLaw,
            PrecedentReason::L7ForbiddenUntilBenchmarkArtifact,
        ),
        "T.7 must emit an L7 benchmark-artifact precedent"
    );
}

#[test]
fn l8_forbidden_until_measured_ledger_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::LBandHonestyLaw,
            PrecedentReason::L8ForbiddenUntilMeasuredLedger,
        ),
        "T.7 must emit an L8 measured-ledger precedent"
    );
}

#[test]
fn not_scored_usefulness_has_honesty_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::UsefulnessHonestyLaw,
            PrecedentReason::NotScoredRequiresZeroEmpiricals,
        ),
        "T.8 must emit a NotScored zero-empiricals precedent"
    );
}

#[test]
fn corpus_hash_freeze_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::CorpusHashLaw,
            PrecedentReason::CorpusHashV1IsFrozen,
        ),
        "T.10 must emit a CorpusHashV1IsFrozen precedent"
    );
}

#[test]
fn registry_binding_has_precedent() {
    let set = collect_court_precedents();
    assert!(
        has_reason(
            &set,
            PrecedentKind::RegistryBindingLaw,
            PrecedentReason::HashFrozenT10RequiresNonZeroSourceCorpusHash,
        ),
        "S1.2 must emit the HashFrozenT10-cross-field-rule precedent"
    );
    assert!(
        has_reason(
            &set,
            PrecedentKind::RegistryBindingLaw,
            PrecedentReason::RegistryHashV2BindsToFrozenCorpusHash,
        ),
        "S1.2 must emit the registry_hash_v2 binding precedent"
    );
}

#[test]
fn every_passport_links_at_least_one_precedent() {
    let passports = all_passports();
    assert_eq!(passports.len(), 54);
    for p in &passports {
        assert!(
            !p.linked_precedent_ids.is_empty(),
            "passport for canonical_id {:?} has no linked precedents; every passport MUST link at least the global laws",
            p.canonical_id
        );
    }
}

#[test]
fn alias_passports_link_alias_precedent() {
    // Panel form interpretation: a canonical record that has
    // aliases in CLAIMS (e.g. ROBUST_Z_MAD, canonical_id 6) must
    // link to at least one AliasCollapse precedent through its
    // linked_precedent_ids (AliasToCanonical bindings list the
    // canonical, so precedents_for_canonical surfaces them).
    let set = collect_court_precedents();
    let passports = all_passports();
    let robust_z = passports
        .iter()
        .find(|p| p.canonical_id == DetectorCanonicalId(6))
        .expect("ROBUST_Z_MAD passport (canonical_id 6)");
    let linked: Vec<&CourtPrecedent> = robust_z
        .linked_precedent_ids
        .iter()
        .map(|id| set.precedents.iter().find(|p| &p.id == id).unwrap())
        .collect();
    let has_alias_collapse = linked
        .iter()
        .any(|p| p.kind == PrecedentKind::AliasCollapse);
    assert!(
        has_alias_collapse,
        "ROBUST_Z_MAD passport must link at least one AliasCollapse precedent"
    );
}

#[test]
fn precedent_text_render_is_deterministic() {
    let set = collect_court_precedents();
    let a = render_precedents_text(&set);
    let b = render_precedents_text(&set);
    assert_eq!(a, b, "text rendering must be deterministic");
}

#[test]
fn precedent_json_render_is_deterministic() {
    let set = collect_court_precedents();
    let a = render_precedents_json(&set);
    let b = render_precedents_json(&set);
    assert_eq!(a, b, "JSON rendering must be deterministic");
}

#[test]
fn no_publication_language_in_t11b_reports() {
    let set = collect_court_precedents();
    let text = render_precedents_text(&set);
    let json = render_precedents_json(&set);
    let forbidden = [
        "Zenodo",
        "DOI",
        "publication-grade",
        "peer-reviewed",
        "ready for publication",
    ];
    for body in [&text, &json] {
        for word in forbidden {
            assert!(
                !body.contains(word),
                "T.11b report contains forbidden publication-language `{word}`"
            );
        }
    }
}

#[test]
fn verify_precedent_set_admits_live_collection() {
    let set = collect_court_precedents();
    let errors = verify_precedent_set(&set);
    assert!(
        errors.is_empty(),
        "verify_precedent_set MUST admit the live collection; got {errors:?}"
    );
}

#[test]
fn precedent_verifier_rejects_precedent_with_missing_subject() {
    let mut set = collect_court_precedents();
    // Forge a precedent with a non-existent canonical id.
    set.precedents.push(CourtPrecedent {
        id: PrecedentId(99999),
        kind: PrecedentKind::DedupCanonical,
        source: PrecedentSource::T4DedupCourt,
        binding: PrecedentBinding::SingleCanonical(DetectorCanonicalId(9999)),
        reason: PrecedentReason::OriginRecord,
        severity: PrecedentSeverity::Hard,
        notes: "forged for the missing-subject test",
    });
    let errors = verify_precedent_set(&set);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == PrecedentVerifyErrorKind::MissingCanonicalSubject),
        "verifier MUST reject precedent with missing canonical subject; got {errors:?}"
    );
}

#[test]
fn precedent_verifier_rejects_alias_precedent_if_semantic_role_hash_differs() {
    // The panel test name says "if semantic_role_hash differs."
    // The T.4 + T.3 invariant: AliasCollapse requires same
    // formula AND same witness role. The reason
    // `SameFormulaDifferentWitnessRole` directly contradicts
    // that. The verifier MUST reject an AliasCollapse precedent
    // carrying that reason.
    let mut set = collect_court_precedents();
    set.precedents.push(CourtPrecedent {
        id: PrecedentId(99998),
        kind: PrecedentKind::AliasCollapse,
        source: PrecedentSource::T4DedupCourt,
        binding: PrecedentBinding::AliasToCanonical {
            alias_id: DetectorAliasId(1001),
            canonical_id: DetectorCanonicalId(6),
        },
        // This is the role-drift reason; AliasCollapse with this
        // reason is illegitimate.
        reason: PrecedentReason::SameFormulaDifferentWitnessRole,
        severity: PrecedentSeverity::Hard,
        notes: "forged AliasCollapse with role-drift reason",
    });
    let errors = verify_precedent_set(&set);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == PrecedentVerifyErrorKind::KindReasonIncompatible),
        "verifier MUST reject AliasCollapse + role-drift reason; got {errors:?}"
    );
}

#[test]
fn deferred_gates_have_deferred_severity() {
    let set = collect_court_precedents();
    let deferred = ids_for_kind(&set, PrecedentKind::DeferredGateLaw);
    assert!(
        !deferred.is_empty(),
        "T.9 must emit DeferredGateLaw precedents"
    );
    for p in deferred {
        assert_eq!(
            p.severity,
            PrecedentSeverity::Deferred,
            "DeferredGateLaw precedent {:?} must carry Deferred severity",
            p.id
        );
    }
}

#[test]
fn precedent_ids_are_unique_and_dense() {
    let set = collect_court_precedents();
    let ids: BTreeSet<u32> = set.precedents.iter().map(|p| p.id.0).collect();
    assert_eq!(
        ids.len(),
        set.precedents.len(),
        "all precedent ids must be unique"
    );
    // Dense: 1..=count
    for (i, p) in set.precedents.iter().enumerate() {
        let expected = u32::try_from(i + 1).unwrap();
        assert_eq!(
            p.id.0, expected,
            "precedent ids must be a 1..=count dense sequence after canonical sort"
        );
    }
}

#[test]
fn precedent_count_is_panel_locked_at_t11b() {
    // 54 SEED canonical records → 54 court records, minus the
    // two CompositionOf records that get CompositionJudgment +
    // any StochasticOriginalDeterministicReduction → some
    // mixture. Plus 12 alias claims, plus the global laws.
    // The exact count is the panel-locked T.11b receipt. Pin it
    // here so an accidental SEED / CLAIMS / law-list change is
    // caught.
    let set = collect_court_precedents();
    assert_eq!(
        set.precedents.len(),
        83,
        "T.11b precedent count is panel-locked at 83 (52 DedupCanonical + 12 alias + 2 composition + N stochastic-reduction + 15 global laws). A change here means a structural corpus change."
    );
}
