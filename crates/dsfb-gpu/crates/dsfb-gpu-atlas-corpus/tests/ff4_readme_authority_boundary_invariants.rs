//! FF.4 acceptance suite — README authority-boundary policy
//! invariants for the post-T.12.consolidate / post-FF.1 /
//! post-FF.2 / post-FF.3 authority-state communication seal.
//!
//! Seven panel-required load-bearing negatives pin the contract
//! discipline FF.4 exists to prove:
//!
//! * `ff4_readme_rejects_stale_future_ratification_language`
//! * `ff4_readme_requires_corpus_hash_v1_historical_anchor_language`
//! * `ff4_readme_requires_corpus_hash_v2_ratified_authority_language`
//! * `ff4_readme_requires_ff1_passport_materialisation_language`
//! * `ff4_readme_requires_ff2_ff3_unratified_rejection_language`
//! * `ff4_readme_rejects_claim_that_t12_proposals_mutated_seed`
//! * `ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2`
//!
//! Plus a live README-content sweep that verifies the real
//! `README.md` on disk satisfies the policy. This is the
//! hygiene seal that prevents future commits from regressing
//! the front-door authority-state story.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
};
use dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use dsfb_gpu_atlas_corpus::ff4_readme_authority_boundary::{
    build_ff4_readme_authority_boundary_policy, render_ff4_authority_boundary_block,
    render_ff4_policy_json, render_ff4_policy_text, verify_ff4_readme, Ff4VerifyError,
    Ff4VerifyErrorKind, FF4_AUTHORITY_BOUNDARY_BLOCK_LINES, FF4_FORBIDDEN_SUBSTRINGS,
    FF4_README_AUTHORITY_BOUNDARY_POLICY_DOMAIN_V1, FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1,
    FF4_REQUIRED_SUBSTRINGS,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

const CANONICAL_GOOD_README: &str = "
This file is a fixture README satisfying the FF.4 policy.

## Authority boundary (post-T.12.consolidate + FF.1 + FF.2 + FF.3)

Important authority-state note. T.12.a..T.12.p were amendment proposals.
They did not mutate SEED, corpus_hash_v1, registry_hash_v2, historical
DetectorPassports, or activation outputs while they were filed.

T.12.consolidate ratified the accepted T.12 expansion set and froze
corpus_hash_v2 as the first post-amendment corpus authority.

FF.1 then materialized 98 ratified T.12 CanonicalAddition entries into
T12RatifiedPassport records under ff1_passport_index_hash_v1.

FF.2 and FF.3 now enforce that activation and registry generation consume
only SeedHistorical records or T12RatifiedAndPassported records. Unratified,
non-passported, ad-hoc, or unknown-source records are rejected by explicit
reason code (DisabledUnratifiedProposal at activation; RejectedUnratifiedProposal,
RejectedMissingFf1Passport, RejectedCorpusHashV2Mismatch, RejectedPassportIndexHashMismatch,
RejectedAdHocRecord, RejectedUnknownSourceAuthority at registry generation).

- SEED and corpus_hash_v1 remain the historical seed-corpus anchor.
- T.12 proposals did not mutate seed authority while filed.
- T.12.consolidate froze corpus_hash_v2 as ratified post-amendment authority.
- FF.1 materialized ratified T.12 additions into passports.
- FF.2 / FF.3 prevent unratified records from entering activation or registry generation.
";

// ---------------------------------------------------------------
// Panel-required load-bearing negative #1
// ---------------------------------------------------------------

#[test]
fn ff4_readme_rejects_stale_future_ratification_language() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let stale = format!(
        "{CANONICAL_GOOD_README}\nT.12.a..T.12.p are amendment proposals; they do not mutate SEED \
         until a future ratification / freeze campaign."
    );
    let errs = verify_ff4_readme(&policy, &stale);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff4VerifyErrorKind::StaleFutureRatificationLanguage {
            observed_forbidden_substring,
        }
            if observed_forbidden_substring == "future ratification / freeze campaign"
    )));
}

#[test]
fn ff4_readme_rejects_each_stale_phrase_variant() {
    let policy = build_ff4_readme_authority_boundary_policy();
    for phrase in [
        "until a future ratification campaign",
        "until a future freeze",
    ] {
        let stale = format!("{CANONICAL_GOOD_README}\nOlder doc said: {phrase}");
        let errs = verify_ff4_readme(&policy, &stale);
        assert!(
            errs.iter().any(|e| matches!(
                e.kind,
                Ff4VerifyErrorKind::StaleFutureRatificationLanguage { .. }
            )),
            "missed stale phrase: {phrase}"
        );
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #2
// ---------------------------------------------------------------

#[test]
fn ff4_readme_requires_corpus_hash_v1_historical_anchor_language() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let bad = CANONICAL_GOOD_README.replace("historical seed-corpus anchor", "anchor");
    let errs = verify_ff4_readme(&policy, &bad);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff4VerifyErrorKind::MissingCorpusHashV1HistoricalAnchorLanguage
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #3
// ---------------------------------------------------------------

#[test]
fn ff4_readme_requires_corpus_hash_v2_ratified_authority_language() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let bad = CANONICAL_GOOD_README.replace("ratified post-amendment authority", "authority");
    let errs = verify_ff4_readme(&policy, &bad);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff4VerifyErrorKind::MissingCorpusHashV2RatifiedAuthorityLanguage
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #4
// ---------------------------------------------------------------

#[test]
fn ff4_readme_requires_ff1_passport_materialisation_language() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let bad = CANONICAL_GOOD_README.replace(
        "FF.1 materialized ratified T.12 additions into passports.",
        "FF.1 happened.",
    );
    let errs = verify_ff4_readme(&policy, &bad);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff4VerifyErrorKind::MissingFf1PassportMaterialisationLanguage
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #5
// ---------------------------------------------------------------

#[test]
fn ff4_readme_requires_ff2_ff3_unratified_rejection_language() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let bad = CANONICAL_GOOD_README.replace(
        "FF.2 / FF.3 prevent unratified records from entering activation or registry generation.",
        "Some gates exist.",
    );
    let errs = verify_ff4_readme(&policy, &bad);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff4VerifyErrorKind::MissingFf2Ff3UnratifiedRejectionLanguage
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #6
// ---------------------------------------------------------------

#[test]
fn ff4_readme_rejects_claim_that_t12_proposals_mutated_seed() {
    let policy = build_ff4_readme_authority_boundary_policy();
    for phrase in ["T.12 proposals mutated SEED", "T.12 proposals mutate SEED"] {
        let bad = format!("{CANONICAL_GOOD_README}\nIncorrect claim: {phrase}.");
        let errs = verify_ff4_readme(&policy, &bad);
        assert!(
            errs.iter()
                .any(|e| matches!(e.kind, Ff4VerifyErrorKind::ClaimThatT12ProposalsMutatedSeed)),
            "missed mutation claim: {phrase}"
        );
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #7
// ---------------------------------------------------------------

#[test]
fn ff4_readme_rejects_claim_that_ff1_mutated_corpus_hash_v2() {
    let policy = build_ff4_readme_authority_boundary_policy();
    for phrase in ["FF.1 mutated corpus_hash_v2", "FF.1 mutates corpus_hash_v2"] {
        let bad = format!("{CANONICAL_GOOD_README}\nIncorrect claim: {phrase}.");
        let errs = verify_ff4_readme(&policy, &bad);
        assert!(
            errs.iter()
                .any(|e| matches!(e.kind, Ff4VerifyErrorKind::ClaimThatFf1MutatedCorpusHashV2)),
            "missed mutation claim: {phrase}"
        );
    }
}

// ---------------------------------------------------------------
// Live README content verification (the hygiene seal)
// ---------------------------------------------------------------

#[test]
fn ff4_live_readme_satisfies_policy() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let path = format!("{}/../../README.md", env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read README at {path}: {err}"));
    let errs: Vec<Ff4VerifyError> = verify_ff4_readme(&policy, &readme);
    assert!(
        errs.is_empty(),
        "live README MUST satisfy FF.4 policy:\n{errs:#?}"
    );
}

// ---------------------------------------------------------------
// Default policy + admissibility on canonical text
// ---------------------------------------------------------------

#[test]
fn ff4_policy_admits_canonical_good_readme() {
    let policy = build_ff4_readme_authority_boundary_policy();
    let errs = verify_ff4_readme(&policy, CANONICAL_GOOD_README);
    assert!(
        errs.is_empty(),
        "canonical good README must verify: {errs:?}"
    );
}

// ---------------------------------------------------------------
// Determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn ff4_policy_hash_is_deterministic_across_two_builds() {
    let p1 = build_ff4_readme_authority_boundary_policy();
    let p2 = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        p1.ff4_readme_authority_boundary_policy_hash_v1,
        p2.ff4_readme_authority_boundary_policy_hash_v1
    );
}

#[test]
fn ff4_policy_text_render_byte_stable() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(render_ff4_policy_text(&p), render_ff4_policy_text(&p));
}

#[test]
fn ff4_policy_json_render_byte_stable() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(render_ff4_policy_json(&p), render_ff4_policy_json(&p));
}

#[test]
fn ff4_authority_boundary_block_render_byte_stable() {
    assert_eq!(
        render_ff4_authority_boundary_block(),
        render_ff4_authority_boundary_block()
    );
}

// ---------------------------------------------------------------
// Upstream-anchor invariance
// ---------------------------------------------------------------

#[test]
fn ff4_does_not_mutate_corpus_hash_v1() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

#[test]
fn ff4_does_not_mutate_corpus_hash_v2() {
    let before = build_consolidation_report().corpus_hash_v2;
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = build_consolidation_report().corpus_hash_v2;
    assert_eq!(before, after);
}

#[test]
fn ff4_does_not_mutate_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let before = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff4_does_not_mutate_ff2_gate_hash() {
    let r = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&r);
    let ids = default_candidate_ids(&idx);
    let before = build_ff2_activation_ratification_gate_from(&r, &idx, &ids)
        .ff2_activation_ratification_gate_hash_v1;
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = build_ff2_activation_ratification_gate_from(&r, &idx, &ids)
        .ff2_activation_ratification_gate_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff4_does_not_mutate_ff3_gate_hash() {
    let before = build_ff3_registry_generation_gate().ff3_registry_generation_gate_hash_v1;
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = build_ff3_registry_generation_gate().ff3_registry_generation_gate_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff4_does_not_mutate_seed_len() {
    let before = SEED.len();
    let _ = build_ff4_readme_authority_boundary_policy();
    let after = SEED.len();
    assert_eq!(before, 54);
    assert_eq!(after, 54);
}

// ---------------------------------------------------------------
// Pinned anchor cross-checks
// ---------------------------------------------------------------

#[test]
fn ff4_policy_pins_live_corpus_hash_v1() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(p.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn ff4_policy_pins_live_corpus_hash_v2() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        p.corpus_hash_v2,
        build_consolidation_report().corpus_hash_v2
    );
}

#[test]
fn ff4_policy_pins_live_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        p.ff1_passport_index_hash_v1,
        build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1
    );
}

#[test]
fn ff4_policy_pins_live_ff2_gate_hash() {
    let r = build_consolidation_report();
    let idx = build_ff1_passport_index_from(&r);
    let ids = default_candidate_ids(&idx);
    let live_ff2 = build_ff2_activation_ratification_gate_from(&r, &idx, &ids);
    let p = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        p.ff2_activation_ratification_gate_hash_v1,
        live_ff2.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff4_policy_pins_live_ff3_gate_hash() {
    let p = build_ff4_readme_authority_boundary_policy();
    let live_ff3 = build_ff3_registry_generation_gate();
    assert_eq!(
        p.ff3_registry_generation_gate_hash_v1,
        live_ff3.ff3_registry_generation_gate_hash_v1
    );
}

// ---------------------------------------------------------------
// Pinned constants
// ---------------------------------------------------------------

#[test]
fn ff4_domain_separator_pin() {
    assert_eq!(
        FF4_README_AUTHORITY_BOUNDARY_POLICY_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1\0"
    );
}

#[test]
fn ff4_schema_pin() {
    assert_eq!(
        FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1,
        "DSFB-GPU-ATLAS:FF4-README-AUTHORITY-BOUNDARY-POLICY:v1"
    );
}

#[test]
fn ff4_block_lines_non_empty() {
    assert!(!FF4_AUTHORITY_BOUNDARY_BLOCK_LINES.is_empty());
    for line in FF4_AUTHORITY_BOUNDARY_BLOCK_LINES {
        assert!(!line.is_empty());
    }
}

#[test]
fn ff4_required_substrings_non_empty() {
    assert!(!FF4_REQUIRED_SUBSTRINGS.is_empty());
    for s in FF4_REQUIRED_SUBSTRINGS {
        assert!(!s.is_empty());
    }
}

#[test]
fn ff4_forbidden_substrings_non_empty() {
    assert!(!FF4_FORBIDDEN_SUBSTRINGS.is_empty());
    for s in FF4_FORBIDDEN_SUBSTRINGS {
        assert!(!s.is_empty());
    }
}

#[test]
fn ff4_required_and_forbidden_substring_sets_are_disjoint() {
    for r in FF4_REQUIRED_SUBSTRINGS {
        assert!(
            !FF4_FORBIDDEN_SUBSTRINGS.contains(r),
            "required substring `{r}` must not also be forbidden"
        );
    }
}

// ---------------------------------------------------------------
// Block-coverage invariants
// ---------------------------------------------------------------

#[test]
fn ff4_canonical_block_contains_all_required_substrings() {
    let block = render_ff4_authority_boundary_block();
    for s in FF4_REQUIRED_SUBSTRINGS {
        assert!(
            block.contains(s),
            "canonical block must contain required substring `{s}`"
        );
    }
}

#[test]
fn ff4_canonical_block_contains_no_forbidden_substring() {
    let block = render_ff4_authority_boundary_block();
    for s in FF4_FORBIDDEN_SUBSTRINGS {
        assert!(
            !block.contains(s),
            "canonical block must NOT contain forbidden substring `{s}`"
        );
    }
}

#[test]
fn ff4_block_lines_pin_first_line_is_header() {
    assert_eq!(
        FF4_AUTHORITY_BOUNDARY_BLOCK_LINES[0],
        "## Authority boundary (post-T.12.consolidate + FF.1 + FF.2 + FF.3)"
    );
}

#[test]
fn ff4_block_line_count_is_nineteen() {
    assert_eq!(FF4_AUTHORITY_BOUNDARY_BLOCK_LINES.len(), 19);
}

// ---------------------------------------------------------------
// Hash-namespace distinctness
// ---------------------------------------------------------------

#[test]
fn ff4_policy_hash_distinct_from_corpus_hash_v1() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_ne!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        p.corpus_hash_v1
    );
}

#[test]
fn ff4_policy_hash_distinct_from_corpus_hash_v2() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_ne!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        p.corpus_hash_v2
    );
}

#[test]
fn ff4_policy_hash_distinct_from_ff1_passport_index_hash() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_ne!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        p.ff1_passport_index_hash_v1
    );
}

#[test]
fn ff4_policy_hash_distinct_from_ff2_gate_hash() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_ne!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        p.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff4_policy_hash_distinct_from_ff3_gate_hash() {
    let p = build_ff4_readme_authority_boundary_policy();
    assert_ne!(
        p.ff4_readme_authority_boundary_policy_hash_v1,
        p.ff3_registry_generation_gate_hash_v1
    );
}

// ---------------------------------------------------------------
// Render coverage
// ---------------------------------------------------------------

#[test]
fn ff4_render_text_contains_pinned_anchors_and_block() {
    let p = build_ff4_readme_authority_boundary_policy();
    let text = render_ff4_policy_text(&p);
    assert!(text.contains("FF.4 README Authority-Boundary Policy"));
    assert!(text.contains("corpus_hash_v1"));
    assert!(text.contains("corpus_hash_v2"));
    assert!(text.contains("ff1_passport_index_hash_v1"));
    assert!(text.contains("ff2_activation_ratification_gate_hash_v1"));
    assert!(text.contains("ff3_registry_generation_gate_hash_v1"));
    assert!(text.contains("ff4_readme_authority_boundary_policy_hash_v1"));
    for s in FF4_REQUIRED_SUBSTRINGS {
        assert!(text.contains(s));
    }
}

#[test]
fn ff4_render_json_contains_schema_field() {
    let p = build_ff4_readme_authority_boundary_policy();
    let json = render_ff4_policy_json(&p);
    assert!(json.contains(FF4_README_AUTHORITY_BOUNDARY_POLICY_SCHEMA_V1));
}
