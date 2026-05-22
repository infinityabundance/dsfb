//! T.12.PROV acceptance suite — Scientific Provenance Credit
//! Pass invariants.
//!
//! Eight panel-required load-bearing negatives:
//!
//! 1. `t12_prov_rejects_canonical_addition_without_scientist_credit`
//! 2. `t12_prov_rejects_canonical_addition_without_source_ref`
//! 3. `t12_prov_rejects_scientist_credit_without_contribution_note`
//! 4. `t12_prov_rejects_source_ref_key_not_in_proposal_sources`
//! 5. `t12_prov_rejects_dsfb_invention_claim_for_prior_detector`
//! 6. `t12_prov_rejects_engineering_practice_record_without_provenance_note`
//! 7. `t12_prov_rejects_rejected_record_without_method_family_credit`
//! 8. `t12_prov_rejects_parameterization_without_parent_lineage_note`
//!
//! Plus structural defect tests (hash anchors, seed length,
//! sort order) + determinism + sensitivity + rendering byte
//! stability + production walk shape pins (98 canonical
//! additions, 17 proposals, etc.).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    CorpusAmendmentProposal, ProposalStatus, ProposedDedupRecord, ProposedPrimitive,
    ProposedSourceRef, ProposerRole, RejectionRecord, SourceClass,
};
use dsfb_gpu_atlas_corpus::consolidate::load_all_t12_proposals;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::t12_prov_scientific_provenance::{
    build_provenance_credit_report, build_provenance_credit_report_from,
    build_scientist_credit_index, build_scientist_credit_index_from,
    build_source_bibliography_index, build_source_bibliography_index_from,
    forbidden_dsfb_invention_substrings, render_provenance_credit_report_json,
    render_provenance_credit_report_text, render_scientist_credit_index_json,
    render_scientist_credit_index_text, render_source_bibliography_index_json,
    render_source_bibliography_index_text, verify_t12_prov, ProvenanceCreditReport,
    T12ProvVerifyErrorKind, PROVENANCE_CREDIT_REPORT_DOMAIN_V1, PROVENANCE_CREDIT_REPORT_SCHEMA_V1,
    SCIENTIST_CREDIT_INDEX_DOMAIN_V1, SCIENTIST_CREDIT_INDEX_SCHEMA_V1,
    SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1, SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1,
    T12_PROV_DSFB_CREDIT_NOTE,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Build a defective proposal carrying a single `CanonicalAddition`
/// dedup-record for `canonical_id`. The caller controls every
/// field that the verifier's eight panel-required negatives key
/// off (primitive presence, source-ref presence, reason text).
fn build_defective_canonical_addition_proposal(
    proposal_id: &'static str,
    canonical_id: u32,
    include_primitive: bool,
    include_source_ref: bool,
    reason: &'static str,
) -> CorpusAmendmentProposal {
    let primitives = if include_primitive {
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(canonical_id),
            display_name: "t12-prov-defective-test-primitive",
            motivation: "T.12.PROV defective test primitive motivation.",
        }]
    } else {
        Vec::new()
    };
    let source_refs = if include_source_ref {
        vec![ProposedSourceRef {
            citation_key: "t12_prov_defective_test_source",
            title: "T.12.PROV defective test source title",
            year: 2026,
            venue: "T.12.PROV defective test venue",
        }]
    } else {
        Vec::new()
    };
    let dedup_records = vec![ProposedDedupRecord {
        decision_wire_name: "CanonicalAddition",
        canonical_id: DetectorCanonicalId(canonical_id),
        reason,
    }];
    let batch = build_expansion_batch(
        "t12_prov_defective_test_batch",
        SourceClass::InformationTheory,
        primitives,
        Vec::new(),
        dedup_records,
        Vec::new(),
        source_refs,
    );
    let delta = build_dedup_court_delta(
        "t12_prov_defective_test_delta",
        vec![DetectorCanonicalId(canonical_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        proposal_id,
        "T.12.PROV defective test motivation.",
        SourceClass::InformationTheory,
        batch,
        delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_prov_defective_test_commit",
    )
}

// ---------------------------------------------------------------
// Shape + load discipline
// ---------------------------------------------------------------

#[test]
fn loader_returns_seventeen_proposals_for_t12_prov() {
    let proposals = load_all_t12_proposals();
    assert_eq!(proposals.len(), 17);
}

#[test]
fn build_report_walks_seventeen_proposals() {
    let r = build_provenance_credit_report();
    assert_eq!(r.proposal_count, 17);
}

#[test]
fn build_report_admits_admissible_proposal_set_with_zero_errors() {
    let proposals = load_all_t12_proposals();
    let report = build_provenance_credit_report_from(&proposals);
    let errors = verify_t12_prov(&report, &proposals);
    assert!(
        errors.is_empty(),
        "panel-locked T.12.a..T.12.p must satisfy T.12.PROV: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Production walk-shape pins
// ---------------------------------------------------------------

#[test]
fn production_walk_pins_ninety_eight_canonical_additions() {
    let r = build_provenance_credit_report();
    assert_eq!(r.scientist_credit_index.canonical_addition_count, 98);
}

#[test]
fn production_walk_pins_one_hundred_thirty_three_bibliography_entries() {
    let r = build_provenance_credit_report();
    assert_eq!(r.source_bibliography_index.unique_entry_count, 133);
}

#[test]
fn production_walk_pins_twenty_four_rejection_records() {
    let r = build_provenance_credit_report();
    assert_eq!(r.rejection_record_count, 24);
}

#[test]
fn production_walk_pins_forty_nine_parameterization_records() {
    let r = build_provenance_credit_report();
    assert_eq!(r.parameterization_record_count, 49);
}

#[test]
fn scientist_credit_index_length_matches_canonical_addition_count() {
    let r = build_provenance_credit_report();
    assert_eq!(
        r.scientist_credit_index.credits.len() as u32,
        r.scientist_credit_index.canonical_addition_count
    );
}

#[test]
fn source_bibliography_index_length_matches_unique_entry_count() {
    let r = build_provenance_credit_report();
    assert_eq!(
        r.source_bibliography_index.entries.len() as u32,
        r.source_bibliography_index.unique_entry_count
    );
}

// ---------------------------------------------------------------
// SEED + corpus_hash_v1 invariance
// ---------------------------------------------------------------

#[test]
fn seed_len_remains_54_after_t12_prov() {
    let _ = build_provenance_credit_report();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn corpus_hash_v1_unchanged_after_t12_prov() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_provenance_credit_report();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after, "corpus_hash_v1 must remain historical");
}

#[test]
fn report_records_seed_len_as_54() {
    let r = build_provenance_credit_report();
    assert_eq!(r.seed_len, 54);
}

#[test]
fn report_records_corpus_hash_v1_anchor() {
    let r = build_provenance_credit_report();
    assert_eq!(r.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn scientist_credit_index_hash_is_deterministic() {
    let a = build_scientist_credit_index();
    let b = build_scientist_credit_index();
    assert_eq!(
        a.scientist_credit_index_hash_v1,
        b.scientist_credit_index_hash_v1
    );
}

#[test]
fn source_bibliography_index_hash_is_deterministic() {
    let a = build_source_bibliography_index();
    let b = build_source_bibliography_index();
    assert_eq!(
        a.source_bibliography_index_hash_v1,
        b.source_bibliography_index_hash_v1
    );
}

#[test]
fn provenance_credit_report_hash_is_deterministic() {
    let a = build_provenance_credit_report();
    let b = build_provenance_credit_report();
    assert_eq!(
        a.provenance_credit_report_hash_v1,
        b.provenance_credit_report_hash_v1
    );
}

#[test]
fn report_text_render_is_deterministic_across_two_builds() {
    let a = render_provenance_credit_report_text(&build_provenance_credit_report());
    let b = render_provenance_credit_report_text(&build_provenance_credit_report());
    assert_eq!(a, b);
}

#[test]
fn report_json_render_is_deterministic_across_two_builds() {
    let a = render_provenance_credit_report_json(&build_provenance_credit_report());
    let b = render_provenance_credit_report_json(&build_provenance_credit_report());
    assert_eq!(a, b);
}

#[test]
fn scientist_credit_index_text_render_is_deterministic() {
    let a = render_scientist_credit_index_text(&build_scientist_credit_index());
    let b = render_scientist_credit_index_text(&build_scientist_credit_index());
    assert_eq!(a, b);
}

#[test]
fn scientist_credit_index_json_render_is_deterministic() {
    let a = render_scientist_credit_index_json(&build_scientist_credit_index());
    let b = render_scientist_credit_index_json(&build_scientist_credit_index());
    assert_eq!(a, b);
}

#[test]
fn source_bibliography_index_text_render_is_deterministic() {
    let a = render_source_bibliography_index_text(&build_source_bibliography_index());
    let b = render_source_bibliography_index_text(&build_source_bibliography_index());
    assert_eq!(a, b);
}

#[test]
fn source_bibliography_index_json_render_is_deterministic() {
    let a = render_source_bibliography_index_json(&build_source_bibliography_index());
    let b = render_source_bibliography_index_json(&build_source_bibliography_index());
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Three new own-namespace hashes are distinct from each other
// ---------------------------------------------------------------

#[test]
fn three_new_own_namespace_hashes_are_pairwise_distinct() {
    let r = build_provenance_credit_report();
    let a = r.scientist_credit_index.scientist_credit_index_hash_v1;
    let b = r
        .source_bibliography_index
        .source_bibliography_index_hash_v1;
    let c = r.provenance_credit_report_hash_v1;
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(b, c);
}

#[test]
fn report_hash_differs_from_corpus_hash_v1() {
    let r = build_provenance_credit_report();
    assert_ne!(r.provenance_credit_report_hash_v1, r.corpus_hash_v1);
}

// ---------------------------------------------------------------
// Domain separators + schema ids are distinct and panel-locked
// ---------------------------------------------------------------

#[test]
fn domain_separators_are_pairwise_distinct() {
    assert_ne!(
        SCIENTIST_CREDIT_INDEX_DOMAIN_V1,
        SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1
    );
    assert_ne!(
        SCIENTIST_CREDIT_INDEX_DOMAIN_V1,
        PROVENANCE_CREDIT_REPORT_DOMAIN_V1
    );
    assert_ne!(
        SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1,
        PROVENANCE_CREDIT_REPORT_DOMAIN_V1
    );
}

#[test]
fn domain_separators_end_with_nul_byte() {
    assert!(SCIENTIST_CREDIT_INDEX_DOMAIN_V1.ends_with('\0'));
    assert!(SOURCE_BIBLIOGRAPHY_INDEX_DOMAIN_V1.ends_with('\0'));
    assert!(PROVENANCE_CREDIT_REPORT_DOMAIN_V1.ends_with('\0'));
}

#[test]
fn schema_ids_are_pairwise_distinct() {
    assert_ne!(
        SCIENTIST_CREDIT_INDEX_SCHEMA_V1,
        SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1
    );
    assert_ne!(
        SCIENTIST_CREDIT_INDEX_SCHEMA_V1,
        PROVENANCE_CREDIT_REPORT_SCHEMA_V1
    );
    assert_ne!(
        SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1,
        PROVENANCE_CREDIT_REPORT_SCHEMA_V1
    );
}

// ---------------------------------------------------------------
// Sort order
// ---------------------------------------------------------------

#[test]
fn scientist_credits_are_sorted_ascending_by_canonical_id() {
    let r = build_provenance_credit_report();
    for w in r.scientist_credit_index.credits.windows(2) {
        assert!(
            w[0].canonical_id < w[1].canonical_id,
            "credits must be strictly ascending: {} >= {}",
            w[0].canonical_id,
            w[1].canonical_id
        );
    }
}

#[test]
fn bibliography_entries_are_sorted_ascending_by_class_then_key() {
    let r = build_provenance_credit_report();
    for w in r.source_bibliography_index.entries.windows(2) {
        let a = (w[0].source_class_wire_name, w[0].citation_key);
        let b = (w[1].source_class_wire_name, w[1].citation_key);
        assert!(a <= b, "bibliography must be ascending: {a:?} > {b:?}");
    }
}

// ---------------------------------------------------------------
// Every credit carries the panel-locked DSFB credit note
// ---------------------------------------------------------------

#[test]
fn every_credit_row_carries_panel_locked_dsfb_credit_note() {
    let r = build_provenance_credit_report();
    assert!(!r.scientist_credit_index.credits.is_empty());
    for c in &r.scientist_credit_index.credits {
        assert_eq!(c.credit_note, T12_PROV_DSFB_CREDIT_NOTE);
    }
}

#[test]
fn panel_locked_credit_note_disclaims_dsfb_invention() {
    assert!(T12_PROV_DSFB_CREDIT_NOTE.contains("does not claim invention"));
}

// ---------------------------------------------------------------
// Cross-proposal canonical-id uniqueness on production walk
// ---------------------------------------------------------------

#[test]
fn no_two_credits_share_the_same_canonical_id_on_production_walk() {
    let r = build_provenance_credit_report();
    let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for c in &r.scientist_credit_index.credits {
        assert!(
            seen.insert(c.canonical_id),
            "duplicate canonical id {} in credit index",
            c.canonical_id
        );
    }
}

// ---------------------------------------------------------------
// Every credit row's origin proposal id is one of the 17
// expected proposal ids
// ---------------------------------------------------------------

#[test]
fn every_credit_origin_proposal_id_is_known() {
    let proposals = load_all_t12_proposals();
    let known: std::collections::BTreeSet<&str> = proposals.iter().map(|p| p.proposal_id).collect();
    let r = build_provenance_credit_report();
    for c in &r.scientist_credit_index.credits {
        assert!(
            known.contains(c.origin_proposal_id),
            "credit row references unknown proposal {}",
            c.origin_proposal_id
        );
    }
}

// ---------------------------------------------------------------
// Reserved id-range coverage (T.12.a..T.12.p reserved bands)
// ---------------------------------------------------------------

#[test]
fn every_credit_canonical_id_lies_in_reserved_t12_band() {
    let r = build_provenance_credit_report();
    for c in &r.scientist_credit_index.credits {
        assert!(
            (5001..=6699).contains(&c.canonical_id),
            "credit canonical id {} outside the T.12 reserved 5001..=6699 band",
            c.canonical_id
        );
    }
}

// ---------------------------------------------------------------
// Eight panel-required load-bearing negatives
// ---------------------------------------------------------------

#[test]
fn t12_prov_rejects_canonical_addition_without_scientist_credit() {
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_neg1_without_scientist_credit",
        6_950,
        false, // omit primitive → no scientist credit text
        true,
        "T.12.PROV neg-1: canonical without primitive shell.",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::CanonicalAdditionWithoutScientistCredit {
                canonical_id: 6_950,
                ..
            }
        )),
        "missing-primitive case must surface CanonicalAdditionWithoutScientistCredit: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_canonical_addition_without_source_ref() {
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_neg2_without_source_ref",
        6_951,
        true,
        false, // omit source ref → entire batch lacks any provenance citation
        "T.12.PROV neg-2: canonical without source ref.",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::CanonicalAdditionWithoutSourceRef {
                canonical_id: 6_951,
                ..
            }
        )),
        "empty-source-ref case must surface CanonicalAdditionWithoutSourceRef: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_scientist_credit_without_contribution_note() {
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_neg3_without_contribution_note",
        6_952,
        true,
        true,
        "", // empty reason
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::ScientistCreditWithoutContributionNote {
                canonical_id: 6_952,
            }
        )),
        "empty-reason case must surface ScientistCreditWithoutContributionNote: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_source_ref_key_not_in_proposal_sources() {
    // The verifier's R.4 rule walks the credit-index rows and
    // rejects any source_ref_key absent from the originating
    // proposal's proposed_source_refs list. We synthesise a
    // defective report by hand-mutating one credit row's
    // source_ref_keys to inject an orphan key.
    let proposals = load_all_t12_proposals();
    let mut report = build_provenance_credit_report_from(&proposals);
    let first = report
        .scientist_credit_index
        .credits
        .first_mut()
        .expect("production credit index must be non-empty");
    first.source_ref_keys.push("t12_prov_neg4_orphan_key");
    let errors = verify_t12_prov(&report, &proposals);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::SourceRefKeyNotInProposalSources {
                orphan_citation_key,
                ..
            } if orphan_citation_key == "t12_prov_neg4_orphan_key"
        )),
        "orphan source-ref key must surface SourceRefKeyNotInProposalSources: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_dsfb_invention_claim_for_prior_detector() {
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_neg5_dsfb_invention_claim",
        6_953,
        true,
        true,
        "T.12.PROV neg-5: dsfb invented this detector — forbidden phrase.",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::DsfbInventionClaimForPriorDetector {
                canonical_id: 6_953,
                ..
            }
        )),
        "dsfb-invention-claim case must surface DsfbInventionClaimForPriorDetector: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_engineering_practice_record_without_provenance_note() {
    let mut full = load_all_t12_proposals();
    let primitives = vec![ProposedPrimitive {
        reserved_canonical_id: DetectorCanonicalId(6_954),
        display_name: "t12-prov-eng-practice-test",
        motivation: "T.12.PROV neg-6: engineering practice test primitive.",
    }];
    let dedup_records = vec![ProposedDedupRecord {
        decision_wire_name: "CanonicalAddition",
        canonical_id: DetectorCanonicalId(6_954),
        reason: "T.12.PROV neg-6: declared engineering-practice canonical.",
    }];
    // Engineering-practice source ref: year = 0 AND empty venue.
    // This is the panel-locked R.6 trip wire.
    let source_refs = vec![ProposedSourceRef {
        citation_key: "t12_prov_neg6_engineering_practice",
        title: "T.12.PROV neg-6 engineering practice note",
        year: 0,
        venue: "",
    }];
    let batch = build_expansion_batch(
        "t12_prov_neg6_batch",
        SourceClass::InformationTheory,
        primitives,
        Vec::new(),
        dedup_records,
        Vec::new(),
        source_refs,
    );
    let delta = build_dedup_court_delta(
        "t12_prov_neg6_delta",
        vec![DetectorCanonicalId(6_954)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    full.push(build_amendment_proposal(
        "t12_prov_neg6_engineering_practice",
        "T.12.PROV neg-6: engineering practice without provenance note.",
        SourceClass::InformationTheory,
        batch,
        delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_prov_neg6_commit",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::EngineeringPracticeRecordWithoutProvenanceNote {
                citation_key: "t12_prov_neg6_engineering_practice",
                ..
            }
        )),
        "year-0 + empty-venue source ref must surface EngineeringPracticeRecordWithoutProvenanceNote: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_rejected_record_without_method_family_credit() {
    let mut full = load_all_t12_proposals();
    let dedup_records = vec![ProposedDedupRecord {
        decision_wire_name: "RejectedNotDeterministic",
        canonical_id: DetectorCanonicalId(6_955),
        reason: "", // empty reason on a rejection shell
    }];
    let batch = build_expansion_batch(
        "t12_prov_neg7_batch",
        SourceClass::InformationTheory,
        Vec::new(),
        Vec::new(),
        dedup_records,
        Vec::new(),
        vec![ProposedSourceRef {
            citation_key: "t12_prov_neg7_source",
            title: "T.12.PROV neg-7 source",
            year: 2026,
            venue: "T.12.PROV neg-7 venue",
        }],
    );
    let delta = build_dedup_court_delta(
        "t12_prov_neg7_delta",
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    full.push(build_amendment_proposal(
        "t12_prov_neg7_rejected_without_method_family",
        "T.12.PROV neg-7: rejection without method-family credit.",
        SourceClass::InformationTheory,
        batch,
        delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_prov_neg7_commit",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::RejectedRecordWithoutMethodFamilyCredit {
                canonical_id: 6_955,
                ..
            }
        )),
        "empty-reason rejection must surface RejectedRecordWithoutMethodFamilyCredit: {errors:?}"
    );
}

#[test]
fn t12_prov_rejects_parameterization_without_parent_lineage_note() {
    let mut full = load_all_t12_proposals();
    let dedup_records = vec![ProposedDedupRecord {
        decision_wire_name: "ParameterizationOf",
        canonical_id: DetectorCanonicalId(6_956),
        reason: "", // empty reason on a parameterization shell
    }];
    let batch = build_expansion_batch(
        "t12_prov_neg8_batch",
        SourceClass::InformationTheory,
        Vec::new(),
        Vec::new(),
        dedup_records,
        Vec::new(),
        vec![ProposedSourceRef {
            citation_key: "t12_prov_neg8_source",
            title: "T.12.PROV neg-8 source",
            year: 2026,
            venue: "T.12.PROV neg-8 venue",
        }],
    );
    let delta = build_dedup_court_delta(
        "t12_prov_neg8_delta",
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    full.push(build_amendment_proposal(
        "t12_prov_neg8_parameterization_without_parent",
        "T.12.PROV neg-8: parameterization without parent lineage note.",
        SourceClass::InformationTheory,
        batch,
        delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_prov_neg8_commit",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::ParameterizationWithoutParentLineageNote {
                canonical_id: 6_956,
                ..
            }
        )),
        "empty-reason parameterization must surface ParameterizationWithoutParentLineageNote: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Forbidden DSFB-invention substring set is non-empty + scanner
// is case-insensitive
// ---------------------------------------------------------------

#[test]
fn forbidden_dsfb_invention_substring_set_is_non_empty() {
    assert!(!forbidden_dsfb_invention_substrings().is_empty());
}

#[test]
fn dsfb_invention_scanner_is_case_insensitive() {
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_case_insensitive_dsfb_invention",
        6_957,
        true,
        true,
        "T.12.PROV: DSFB INVENTED this detector (uppercase variant).",
    ));
    let report = build_provenance_credit_report_from(&full);
    let errors = verify_t12_prov(&report, &full);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::DsfbInventionClaimForPriorDetector {
                canonical_id: 6_957,
                ..
            }
        )),
        "uppercase 'DSFB INVENTED' must trip the case-insensitive scanner: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Structural defect tests
// ---------------------------------------------------------------

#[test]
fn structural_corpus_hash_mismatch_surfaces() {
    let proposals = load_all_t12_proposals();
    let mut report = build_provenance_credit_report_from(&proposals);
    // Flip a byte of the pinned corpus_hash_v1.
    report.corpus_hash_v1[0] ^= 0xFF;
    let errors = verify_t12_prov(&report, &proposals);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, T12ProvVerifyErrorKind::CorpusHashV1Mismatch { .. })),
        "mutated corpus_hash_v1 must surface CorpusHashV1Mismatch: {errors:?}"
    );
}

#[test]
fn structural_credit_sort_violation_surfaces() {
    let proposals = load_all_t12_proposals();
    let mut report = build_provenance_credit_report_from(&proposals);
    // Force a descending pair at the head.
    let len = report.scientist_credit_index.credits.len();
    assert!(len >= 2);
    report.scientist_credit_index.credits.swap(0, len - 1);
    let errors = verify_t12_prov(&report, &proposals);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::ScientistCreditNotSortedAscending
        )),
        "out-of-order credit list must surface ScientistCreditNotSortedAscending: {errors:?}"
    );
}

#[test]
fn structural_bibliography_sort_violation_surfaces() {
    let proposals = load_all_t12_proposals();
    let mut report = build_provenance_credit_report_from(&proposals);
    let len = report.source_bibliography_index.entries.len();
    assert!(len >= 2);
    report.source_bibliography_index.entries.swap(0, len - 1);
    let errors = verify_t12_prov(&report, &proposals);
    assert!(
        errors.iter().any(|e| matches!(
            e.kind,
            T12ProvVerifyErrorKind::BibliographyNotSortedAscending
        )),
        "out-of-order bibliography must surface BibliographyNotSortedAscending: {errors:?}"
    );
}

// ---------------------------------------------------------------
// Sensitivity: hash changes when proposal contents change
// ---------------------------------------------------------------

#[test]
fn scientist_credit_index_hash_changes_when_credit_is_added() {
    let baseline = build_scientist_credit_index();
    let mut full = load_all_t12_proposals();
    // Append a legitimate proposal so the credit index gains a row.
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_sensitivity_added_credit",
        6_958,
        true,
        true,
        "T.12.PROV sensitivity: added credit must change the index hash.",
    ));
    let mutated = build_scientist_credit_index_from(&full);
    assert_ne!(
        baseline.scientist_credit_index_hash_v1,
        mutated.scientist_credit_index_hash_v1
    );
}

#[test]
fn source_bibliography_index_hash_changes_when_source_ref_is_added_in_new_class() {
    let baseline = build_source_bibliography_index();
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_sensitivity_added_source_ref",
        6_959,
        true,
        true,
        "T.12.PROV sensitivity: added source ref must change the bibliography hash.",
    ));
    let mutated = build_source_bibliography_index_from(&full);
    assert_ne!(
        baseline.source_bibliography_index_hash_v1,
        mutated.source_bibliography_index_hash_v1
    );
}

#[test]
fn provenance_credit_report_hash_changes_when_proposal_set_changes() {
    let baseline = build_provenance_credit_report();
    let mut full = load_all_t12_proposals();
    full.push(build_defective_canonical_addition_proposal(
        "t12_prov_sensitivity_added_proposal",
        6_960,
        true,
        true,
        "T.12.PROV sensitivity: added proposal must change the report hash.",
    ));
    let mutated = build_provenance_credit_report_from(&full);
    assert_ne!(
        baseline.provenance_credit_report_hash_v1,
        mutated.provenance_credit_report_hash_v1
    );
}

// ---------------------------------------------------------------
// Helper: typed access to a sample credit row for spot-checks
// ---------------------------------------------------------------

#[test]
fn first_credit_row_lies_in_t12_a_band_5001_through_5199() {
    let r = build_provenance_credit_report();
    let first = r
        .scientist_credit_index
        .credits
        .first()
        .expect("production credit index must be non-empty");
    assert!(
        (5001..=5199).contains(&first.canonical_id),
        "first credit (sorted ascending) should be in T.12.a band 5001..=5199; got {}",
        first.canonical_id
    );
}

#[test]
fn last_credit_row_lies_in_t12_p_band_6601_through_6699() {
    let r = build_provenance_credit_report();
    let last = r
        .scientist_credit_index
        .credits
        .last()
        .expect("production credit index must be non-empty");
    assert!(
        (6601..=6699).contains(&last.canonical_id),
        "last credit (sorted ascending) should be in T.12.p band 6601..=6699; got {}",
        last.canonical_id
    );
}

// ---------------------------------------------------------------
// Rendering smoke tests (text + JSON output contains pinned
// substrings)
// ---------------------------------------------------------------

#[test]
fn report_text_contains_pinned_header_lines() {
    let s = render_provenance_credit_report_text(&build_provenance_credit_report());
    assert!(s.contains("T.12.PROV Provenance Credit Report (v1)"));
    assert!(s.contains("Pinned corpus anchors"));
    assert!(s.contains("Walk shape"));
    assert!(s.contains("Component hashes"));
    assert!(s.contains("Panel-locked DSFB credit note"));
    assert!(s.contains("provenance_credit_report_hash_v1"));
}

#[test]
fn report_text_records_panel_locked_credit_note() {
    let s = render_provenance_credit_report_text(&build_provenance_credit_report());
    assert!(s.contains(T12_PROV_DSFB_CREDIT_NOTE));
}

#[test]
fn report_json_contains_pinned_schema_id() {
    let s = render_provenance_credit_report_json(&build_provenance_credit_report());
    assert!(s.contains(PROVENANCE_CREDIT_REPORT_SCHEMA_V1));
    assert!(s.contains("provenance_credit_report_hash_v1"));
    assert!(s.contains("canonical_addition_count"));
}

#[test]
fn scientist_credit_index_text_contains_pinned_header() {
    let s = render_scientist_credit_index_text(&build_scientist_credit_index());
    assert!(s.contains("T.12.PROV Scientist Credit Index (v1)"));
    assert!(s.contains("canonical_addition_count"));
}

#[test]
fn source_bibliography_index_text_contains_pinned_header() {
    let s = render_source_bibliography_index_text(&build_source_bibliography_index());
    assert!(s.contains("T.12.PROV Source Bibliography Index (v1)"));
    assert!(s.contains("unique_entry_count"));
}

#[test]
fn scientist_credit_index_json_contains_pinned_schema_id() {
    let s = render_scientist_credit_index_json(&build_scientist_credit_index());
    assert!(s.contains(SCIENTIST_CREDIT_INDEX_SCHEMA_V1));
}

#[test]
fn source_bibliography_index_json_contains_pinned_schema_id() {
    let s = render_source_bibliography_index_json(&build_source_bibliography_index());
    assert!(s.contains(SOURCE_BIBLIOGRAPHY_INDEX_SCHEMA_V1));
}

// ---------------------------------------------------------------
// Reason-text discipline: no credit row carries any forbidden
// substring at production
// ---------------------------------------------------------------

#[test]
fn production_credit_rows_carry_no_forbidden_dsfb_invention_substring() {
    let r = build_provenance_credit_report();
    let forbidden = forbidden_dsfb_invention_substrings();
    for c in &r.scientist_credit_index.credits {
        for &sub in forbidden {
            let cl = c.contribution.to_ascii_lowercase();
            let ml = c.motivation.to_ascii_lowercase();
            let sl = sub.to_ascii_lowercase();
            assert!(
                !cl.contains(&sl),
                "credit canonical_id {} contribution contains forbidden '{}'",
                c.canonical_id,
                sub
            );
            assert!(
                !ml.contains(&sl),
                "credit canonical_id {} motivation contains forbidden '{}'",
                c.canonical_id,
                sub
            );
        }
    }
}

// ---------------------------------------------------------------
// Sanity: ProvenanceCreditReport struct fields are populated
// (no zero-init slip-through)
// ---------------------------------------------------------------

#[test]
fn report_has_non_zero_component_hashes() {
    let r = build_provenance_credit_report();
    assert_ne!(r.provenance_credit_report_hash_v1, [0u8; 32]);
    assert_ne!(
        r.scientist_credit_index.scientist_credit_index_hash_v1,
        [0u8; 32]
    );
    assert_ne!(
        r.source_bibliography_index
            .source_bibliography_index_hash_v1,
        [0u8; 32]
    );
}

// ---------------------------------------------------------------
// Self-check: helper builder produces a defective proposal
// that admits a baseline report (so it doesn't conflate
// failures from helper bugs with genuine verifier wins).
// ---------------------------------------------------------------

#[test]
fn helper_build_defective_proposal_compiles_and_produces_hash() {
    let p = build_defective_canonical_addition_proposal(
        "t12_prov_helper_self_check",
        6_961,
        true,
        true,
        "T.12.PROV helper self-check reason.",
    );
    assert_ne!(p.corpus_amendment_proposal_hash_v1, [0u8; 32]);
}

// ---------------------------------------------------------------
// Cross-test: a defective proposal added to the live set
// invalidates the report verifier but the baseline production
// walk-shape pins remain truthful (98 canonical additions on
// the unmodified live set).
// ---------------------------------------------------------------

#[test]
fn production_walk_shape_is_invariant_under_repeated_builds() {
    let r1 = build_provenance_credit_report();
    let r2 = build_provenance_credit_report();
    assert_eq!(
        r1.scientist_credit_index.canonical_addition_count,
        r2.scientist_credit_index.canonical_addition_count
    );
    assert_eq!(
        r1.source_bibliography_index.unique_entry_count,
        r2.source_bibliography_index.unique_entry_count
    );
    assert_eq!(r1.rejection_record_count, r2.rejection_record_count);
    assert_eq!(
        r1.parameterization_record_count,
        r2.parameterization_record_count
    );
}

// ---------------------------------------------------------------
// Spot-check helper: an empty proposal slice produces an empty
// report (sanity for proposal-injection paths).
// ---------------------------------------------------------------

#[test]
fn build_report_from_empty_proposal_slice_admits() {
    let report = build_provenance_credit_report_from(&[]);
    assert_eq!(report.proposal_count, 0);
    assert_eq!(report.scientist_credit_index.canonical_addition_count, 0);
    assert_eq!(report.source_bibliography_index.unique_entry_count, 0);
}

// ---------------------------------------------------------------
// Pin: ProvenanceCreditReport carries SEED.len() exactly
// (not a literal 54 that could drift).
// ---------------------------------------------------------------

#[test]
fn report_seed_len_matches_live_seed_len() {
    let r: ProvenanceCreditReport = build_provenance_credit_report();
    assert_eq!(r.seed_len as usize, SEED.len());
}
