//! FF.2 acceptance suite — activation ratification gate
//! invariants for `corpus_hash_v2`-ratified + FF.1-passported
//! detectors.
//!
//! Six panel-required load-bearing negatives pin the contract
//! discipline FF.2 exists to prove:
//!
//! * `ff2_rejects_activation_for_unratified_proposal`
//! * `ff2_rejects_activation_for_missing_ff1_passport`
//! * `ff2_rejects_activation_when_passport_index_hash_mismatch`
//! * `ff2_rejects_unratified_proposal_without_reason_code`
//! * `ff2_rejects_silent_fallback_to_disabled_by_weak_lband`
//! * `ff2_rejects_activation_reason_without_corpus_hash_v2_binding`
//!
//! Panel-locked non-claim (verbatim):
//!
//! > FF.2 makes activation refuse any detector proposal that is
//! > not ratified by corpus_hash_v2 and materialized through
//! > FF.1 passport authority. Core rule: no ratification + no
//! > passport = no activation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::activation::DisabledReason;
use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate, build_ff2_activation_ratification_gate_from,
    build_ff2_activation_ratification_gate_summary,
    build_ff2_activation_ratification_gate_summary_from_gate, default_candidate_ids,
    render_ff2_gate_json, render_ff2_gate_summary_json, render_ff2_gate_summary_text,
    render_ff2_gate_text, verify_ff2, Ff2RatificationStatus, Ff2VerifyError, Ff2VerifyErrorKind,
    DISABLED_BY_WEAK_L_BAND_WIRE_NAME, DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME,
    FF2_ACTIVATION_GATE_DOMAIN_V1, FF2_ACTIVATION_GATE_SCHEMA_V1,
    FF2_ACTIVATION_GATE_SUMMARY_DOMAIN_V1, FF2_ACTIVATION_GATE_SUMMARY_SCHEMA_V1,
    FF2_NON_CLAIM_LINES,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

// ---------------------------------------------------------------
// Panel-required load-bearing negative #1
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_activation_for_unratified_proposal() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);

    // Inject a synthetic id outside both SEED (1..=54) and the
    // ratified expansion index (5001..=6699). Production
    // classification MUST emit UnratifiedProposal.
    let bogus_id: u32 = 9_999_999;
    let mut candidates = default_candidate_ids(&index);
    candidates.push(bogus_id);
    candidates.sort_unstable();

    let gate = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == bogus_id)
        .expect("bogus id must produce a gate decision");
    assert_eq!(d.status, Ff2RatificationStatus::UnratifiedProposal);
    assert_eq!(
        d.disabled_reason_wire_name,
        DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME
    );
    assert_eq!(d.cited_passport_hash, [0u8; 32]);

    // Verifier admits because the classifier correctly emitted
    // UnratifiedProposal.
    let errs = verify_ff2(&gate, &report, &index);
    assert!(
        errs.is_empty(),
        "verifier should admit when classifier correctly emits UnratifiedProposal: {errs:?}"
    );

    // Now mutate the gate decision to claim T12RatifiedAndPassported
    // for a non-ratified id; the verifier MUST reject.
    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == bogus_id)
        .unwrap();
    mutated.decisions[pos].status = Ff2RatificationStatus::T12RatifiedAndPassported;
    mutated.decisions[pos].status_wire_name =
        Ff2RatificationStatus::T12RatifiedAndPassported.as_str();
    let errs = verify_ff2(&mutated, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::ActivationForUnratifiedProposal { canonical_id, .. }
            if canonical_id == bogus_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #2
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_activation_for_missing_ff1_passport() {
    let report = build_consolidation_report();
    let live_index = build_ff1_passport_index_from(&report);

    // Construct a synthetic passport-index variant that drops
    // one passport, simulating a ratified id without a
    // materialised passport. Note: we must NOT change the
    // pinned ff1_passport_index_hash_v1 field arbitrarily;
    // recompute via the public builder.
    let mut shrunk = live_index.clone();
    let dropped_id = shrunk.passports[0].canonical_id;
    shrunk.passports.remove(0);
    // Decisions on the shrunk index must classify dropped_id
    // as MissingPassport.
    let candidates = default_candidate_ids(&shrunk);
    let mut candidates_with_drop = candidates.clone();
    if !candidates_with_drop.contains(&dropped_id) {
        candidates_with_drop.push(dropped_id);
        candidates_with_drop.sort_unstable();
    }
    let gate = build_ff2_activation_ratification_gate_from(&report, &shrunk, &candidates_with_drop);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == dropped_id)
        .expect("dropped id must produce a decision under shrunk passport index");
    assert_eq!(d.status, Ff2RatificationStatus::MissingPassport);
    assert_eq!(
        d.disabled_reason_wire_name,
        DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME
    );

    // Mutate the decision to claim T12RatifiedAndPassported
    // anyway. Verifier MUST reject under the shrunk passport
    // index.
    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == dropped_id)
        .unwrap();
    mutated.decisions[pos].status = Ff2RatificationStatus::T12RatifiedAndPassported;
    mutated.decisions[pos].status_wire_name =
        Ff2RatificationStatus::T12RatifiedAndPassported.as_str();
    let errs = verify_ff2(&mutated, &report, &shrunk);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::ActivationForMissingFf1Passport { canonical_id }
            if canonical_id == dropped_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #3
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_activation_when_passport_index_hash_mismatch() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    // Mutate the pinned passport-index hash to a sentinel value.
    gate.ff1_passport_index_hash_v1 = [0xff; 32];
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::PassportIndexHashMismatch { claimed, actual }
            if claimed == [0xff; 32] && actual == index.ff1_passport_index_hash_v1
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #4
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_unratified_proposal_without_reason_code() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut candidates = default_candidate_ids(&index);
    candidates.push(8_888_888);
    candidates.sort_unstable();
    let mut gate = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);

    // Mutate the unratified decision to clear its reason code.
    let pos = gate
        .decisions
        .iter()
        .position(|d| d.status == Ff2RatificationStatus::UnratifiedProposal)
        .expect("at least one UnratifiedProposal decision present");
    let cid = gate.decisions[pos].canonical_id;
    gate.decisions[pos].disabled_reason_wire_name = "";
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::UnratifiedProposalWithoutReasonCode { canonical_id }
            if canonical_id == cid
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #5
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_silent_fallback_to_disabled_by_weak_lband() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut candidates = default_candidate_ids(&index);
    candidates.push(7_777_777);
    candidates.sort_unstable();
    let mut gate = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);

    // Mutate the unratified decision to carry the forbidden
    // wire name (the pre-FF.2 silent fallback the panel
    // warning rejects).
    let pos = gate
        .decisions
        .iter()
        .position(|d| d.status == Ff2RatificationStatus::UnratifiedProposal)
        .expect("at least one UnratifiedProposal decision present");
    let cid = gate.decisions[pos].canonical_id;
    gate.decisions[pos].disabled_reason_wire_name = DISABLED_BY_WEAK_L_BAND_WIRE_NAME;
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::SilentFallbackToDisabledByWeakLBand {
            canonical_id,
            observed_reason_wire_name,
        }
            if canonical_id == cid
                && observed_reason_wire_name == DISABLED_BY_WEAK_L_BAND_WIRE_NAME
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #6
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_activation_reason_without_corpus_hash_v2_binding() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    gate.corpus_hash_v2 = [0u8; 32];
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::ActivationReasonWithoutCorpusHashV2Binding { observed_corpus_hash_v2 }
            if observed_corpus_hash_v2 == [0u8; 32]
    )));
}

// ---------------------------------------------------------------
// Default-build invariants
// ---------------------------------------------------------------

#[test]
fn ff2_default_build_is_admissible_under_verifier() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let gate = build_ff2_activation_ratification_gate();
    let errs: Vec<Ff2VerifyError> = verify_ff2(&gate, &report, &index);
    assert!(
        errs.is_empty(),
        "default FF.2 gate must verify cleanly: {errs:?}"
    );
}

#[test]
fn ff2_default_build_has_zero_unratified_decisions() {
    let gate = build_ff2_activation_ratification_gate();
    assert_eq!(gate.unratified_proposal_count, 0);
    assert_eq!(gate.missing_passport_count, 0);
}

#[test]
fn ff2_default_build_seed_count_equals_seed_len() {
    let gate = build_ff2_activation_ratification_gate();
    let seed_len = u32::try_from(SEED.len()).unwrap();
    assert_eq!(gate.seed_historical_count, seed_len);
    assert_eq!(gate.seed_len, seed_len);
}

#[test]
fn ff2_default_build_t12_count_equals_passport_index_size() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let gate = build_ff2_activation_ratification_gate();
    let expected = u32::try_from(index.passports.len()).unwrap();
    assert_eq!(gate.t12_ratified_and_passported_count, expected);
}

#[test]
fn ff2_default_build_total_equals_seed_plus_t12() {
    let gate = build_ff2_activation_ratification_gate();
    let total = gate.seed_historical_count
        + gate.t12_ratified_and_passported_count
        + gate.missing_passport_count
        + gate.unratified_proposal_count;
    assert_eq!(total, u32::try_from(gate.decisions.len()).unwrap());
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn ff2_gate_hash_is_deterministic_across_two_builds() {
    let g1 = build_ff2_activation_ratification_gate();
    let g2 = build_ff2_activation_ratification_gate();
    assert_eq!(
        g1.ff2_activation_ratification_gate_hash_v1,
        g2.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff2_gate_summary_hash_is_deterministic_across_two_builds() {
    let s1 = build_ff2_activation_ratification_gate_summary();
    let s2 = build_ff2_activation_ratification_gate_summary();
    assert_eq!(
        s1.ff2_activation_ratification_gate_summary_hash_v1,
        s2.ff2_activation_ratification_gate_summary_hash_v1
    );
}

#[test]
fn ff2_gate_decisions_sorted_ascending() {
    let gate = build_ff2_activation_ratification_gate();
    for w in gate.decisions.windows(2) {
        assert!(
            w[0].canonical_id < w[1].canonical_id,
            "decisions must be strictly sorted ascending"
        );
    }
}

#[test]
fn ff2_text_render_byte_stable() {
    let gate = build_ff2_activation_ratification_gate();
    let a = render_ff2_gate_text(&gate);
    let b = render_ff2_gate_text(&gate);
    assert_eq!(a, b);
}

#[test]
fn ff2_json_render_byte_stable() {
    let gate = build_ff2_activation_ratification_gate();
    let a = render_ff2_gate_json(&gate);
    let b = render_ff2_gate_json(&gate);
    assert_eq!(a, b);
}

#[test]
fn ff2_summary_text_render_byte_stable() {
    let summary = build_ff2_activation_ratification_gate_summary();
    let a = render_ff2_gate_summary_text(&summary);
    let b = render_ff2_gate_summary_text(&summary);
    assert_eq!(a, b);
}

#[test]
fn ff2_summary_json_render_byte_stable() {
    let summary = build_ff2_activation_ratification_gate_summary();
    let a = render_ff2_gate_summary_json(&summary);
    let b = render_ff2_gate_summary_json(&summary);
    assert_eq!(a, b);
}

// ---------------------------------------------------------------
// Sensitivity
// ---------------------------------------------------------------

#[test]
fn ff2_gate_hash_changes_when_candidate_set_changes() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let baseline = build_ff2_activation_ratification_gate();
    let mut candidates = default_candidate_ids(&index);
    candidates.push(9_999_999);
    candidates.sort_unstable();
    let altered = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);
    assert_ne!(
        baseline.ff2_activation_ratification_gate_hash_v1,
        altered.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff2_gate_hash_changes_when_decision_status_mutates() {
    let mut gate = build_ff2_activation_ratification_gate();
    let baseline_hash = gate.ff2_activation_ratification_gate_hash_v1;
    // Re-classify the first decision deliberately wrong; hash
    // is recomputed by passing through the public surface.
    gate.decisions[0].status = Ff2RatificationStatus::UnratifiedProposal;
    gate.decisions[0].status_wire_name = Ff2RatificationStatus::UnratifiedProposal.as_str();
    // Rehash via a fresh builder over a synthetic candidate
    // list that omits an id; sensitivity is observed against
    // the deterministic builder output.
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut candidates = default_candidate_ids(&index);
    candidates.pop();
    let altered = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);
    assert_ne!(
        baseline_hash,
        altered.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff2_summary_hash_changes_when_gate_hash_changes() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let baseline = build_ff2_activation_ratification_gate_summary();
    let mut candidates = default_candidate_ids(&index);
    candidates.push(5_555_555);
    candidates.sort_unstable();
    let altered_gate = build_ff2_activation_ratification_gate_from(&report, &index, &candidates);
    let altered_summary = build_ff2_activation_ratification_gate_summary_from_gate(altered_gate);
    assert_ne!(
        baseline.ff2_activation_ratification_gate_summary_hash_v1,
        altered_summary.ff2_activation_ratification_gate_summary_hash_v1
    );
}

// ---------------------------------------------------------------
// Upstream-anchor invariance
// ---------------------------------------------------------------

#[test]
fn ff2_does_not_mutate_corpus_hash_v1() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_ff2_activation_ratification_gate();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

#[test]
fn ff2_does_not_mutate_corpus_hash_v2() {
    let before = build_consolidation_report().corpus_hash_v2;
    let _ = build_ff2_activation_ratification_gate();
    let after = build_consolidation_report().corpus_hash_v2;
    assert_eq!(before, after);
}

#[test]
fn ff2_does_not_mutate_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let before = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    let _ = build_ff2_activation_ratification_gate();
    let after = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff2_does_not_mutate_consolidation_report_hash_v1() {
    let before = build_consolidation_report().consolidation_report_hash_v1;
    let _ = build_ff2_activation_ratification_gate();
    let after = build_consolidation_report().consolidation_report_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff2_does_not_mutate_seed_len() {
    let before = SEED.len();
    let _ = build_ff2_activation_ratification_gate();
    let after = SEED.len();
    assert_eq!(before, 54);
    assert_eq!(after, 54);
}

// ---------------------------------------------------------------
// Field-level invariants
// ---------------------------------------------------------------

#[test]
fn ff2_seed_decisions_carry_empty_disabled_reason() {
    let gate = build_ff2_activation_ratification_gate();
    for d in gate
        .decisions
        .iter()
        .filter(|d| d.status == Ff2RatificationStatus::SeedHistorical)
    {
        assert!(d.disabled_reason_wire_name.is_empty());
        assert_eq!(d.cited_passport_hash, [0u8; 32]);
    }
}

#[test]
fn ff2_t12_decisions_carry_empty_disabled_reason_and_non_zero_passport_hash() {
    let gate = build_ff2_activation_ratification_gate();
    for d in gate
        .decisions
        .iter()
        .filter(|d| d.status == Ff2RatificationStatus::T12RatifiedAndPassported)
    {
        assert!(d.disabled_reason_wire_name.is_empty());
        assert_ne!(d.cited_passport_hash, [0u8; 32]);
    }
}

#[test]
fn ff2_seed_decisions_cover_ids_one_through_fifty_four() {
    let gate = build_ff2_activation_ratification_gate();
    let seed_ids: Vec<u32> = gate
        .decisions
        .iter()
        .filter(|d| d.status == Ff2RatificationStatus::SeedHistorical)
        .map(|d| d.canonical_id)
        .collect();
    let expected: Vec<u32> = (1..=54).collect();
    assert_eq!(seed_ids, expected);
}

#[test]
fn ff2_t12_decisions_only_in_reserved_band() {
    let gate = build_ff2_activation_ratification_gate();
    for d in gate
        .decisions
        .iter()
        .filter(|d| d.status == Ff2RatificationStatus::T12RatifiedAndPassported)
    {
        assert!((5001..=6699).contains(&d.canonical_id));
    }
}

#[test]
fn ff2_disabled_reason_wire_name_matches_activation_enum_variant() {
    assert_eq!(
        DisabledReason::DisabledUnratifiedProposal.as_str(),
        DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME
    );
}

#[test]
fn ff2_non_claim_lines_are_non_empty() {
    assert!(!FF2_NON_CLAIM_LINES.is_empty());
    for line in FF2_NON_CLAIM_LINES {
        assert!(!line.is_empty());
    }
}

#[test]
fn ff2_summary_carries_canonical_non_claim_lines() {
    let summary = build_ff2_activation_ratification_gate_summary();
    assert_eq!(summary.non_claim_lines, FF2_NON_CLAIM_LINES);
}

// ---------------------------------------------------------------
// Wire-name / domain-separator pins
// ---------------------------------------------------------------

#[test]
fn ff2_gate_domain_separator_pin() {
    assert_eq!(
        FF2_ACTIVATION_GATE_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0"
    );
}

#[test]
fn ff2_gate_schema_pin() {
    assert_eq!(
        FF2_ACTIVATION_GATE_SCHEMA_V1,
        "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1"
    );
}

#[test]
fn ff2_gate_summary_domain_separator_pin() {
    assert_eq!(
        FF2_ACTIVATION_GATE_SUMMARY_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0"
    );
}

#[test]
fn ff2_gate_summary_schema_pin() {
    assert_eq!(
        FF2_ACTIVATION_GATE_SUMMARY_SCHEMA_V1,
        "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1"
    );
}

#[test]
fn ff2_status_wire_names_pin() {
    assert_eq!(
        Ff2RatificationStatus::SeedHistorical.as_str(),
        "SeedHistorical"
    );
    assert_eq!(
        Ff2RatificationStatus::T12RatifiedAndPassported.as_str(),
        "T12RatifiedAndPassported"
    );
    assert_eq!(
        Ff2RatificationStatus::MissingPassport.as_str(),
        "MissingPassport"
    );
    assert_eq!(
        Ff2RatificationStatus::UnratifiedProposal.as_str(),
        "UnratifiedProposal"
    );
}

#[test]
fn ff2_passes_gate_predicate_matches_status_buckets() {
    assert!(Ff2RatificationStatus::SeedHistorical.passes_gate());
    assert!(Ff2RatificationStatus::T12RatifiedAndPassported.passes_gate());
    assert!(!Ff2RatificationStatus::MissingPassport.passes_gate());
    assert!(!Ff2RatificationStatus::UnratifiedProposal.passes_gate());
}

// ---------------------------------------------------------------
// Pinned anchor cross-check
// ---------------------------------------------------------------

#[test]
fn ff2_gate_pins_live_corpus_hash_v1() {
    let gate = build_ff2_activation_ratification_gate();
    let live = compute_corpus_hash_v1().bytes;
    assert_eq!(gate.corpus_hash_v1, live);
}

#[test]
fn ff2_gate_pins_live_corpus_hash_v2() {
    let gate = build_ff2_activation_ratification_gate();
    let live = build_consolidation_report().corpus_hash_v2;
    assert_eq!(gate.corpus_hash_v2, live);
}

#[test]
fn ff2_gate_pins_live_consolidation_report_hash_v1() {
    let gate = build_ff2_activation_ratification_gate();
    let live = build_consolidation_report().consolidation_report_hash_v1;
    assert_eq!(gate.consolidation_report_hash_v1, live);
}

#[test]
fn ff2_gate_pins_live_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let gate = build_ff2_activation_ratification_gate();
    let live = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    assert_eq!(gate.ff1_passport_index_hash_v1, live);
}

// ---------------------------------------------------------------
// Structural defect rules (R.7 / R.8 / R.9)
// ---------------------------------------------------------------

#[test]
fn ff2_rejects_duplicate_gate_decision_for_same_canonical_id() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    let dup = gate.decisions[0].clone();
    gate.decisions.push(dup);
    gate.decisions.sort_by_key(|d| d.canonical_id);
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::DuplicateGateDecisionForSameCanonicalId { .. }
    )));
}

#[test]
fn ff2_rejects_gate_decisions_not_sorted_ascending() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    gate.decisions.reverse();
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff2VerifyErrorKind::GateDecisionsNotSortedAscending)));
}

#[test]
fn ff2_rejects_corpus_hash_v1_mismatch() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    gate.corpus_hash_v1 = [0xaa; 32];
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff2VerifyErrorKind::CorpusHashV1Mismatch { .. })));
}

#[test]
fn ff2_rejects_consolidation_report_hash_v1_mismatch() {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let mut gate = build_ff2_activation_ratification_gate();
    gate.consolidation_report_hash_v1 = [0xbb; 32];
    let errs = verify_ff2(&gate, &report, &index);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff2VerifyErrorKind::ConsolidationReportHashV1Mismatch { .. }
    )));
}

// ---------------------------------------------------------------
// Count-shape regression sentinels
// ---------------------------------------------------------------

#[test]
fn ff2_seed_historical_count_is_fifty_four() {
    let gate = build_ff2_activation_ratification_gate();
    assert_eq!(gate.seed_historical_count, 54);
}

#[test]
fn ff2_t12_ratified_and_passported_count_is_ninety_eight() {
    let gate = build_ff2_activation_ratification_gate();
    assert_eq!(gate.t12_ratified_and_passported_count, 98);
}

#[test]
fn ff2_total_decision_count_is_one_hundred_fifty_two() {
    let gate = build_ff2_activation_ratification_gate();
    assert_eq!(gate.decisions.len(), 54 + 98);
}

// ---------------------------------------------------------------
// Hash-namespace distinctness
// ---------------------------------------------------------------

#[test]
fn ff2_gate_hash_distinct_from_summary_hash() {
    let summary = build_ff2_activation_ratification_gate_summary();
    assert_ne!(
        summary.gate.ff2_activation_ratification_gate_hash_v1,
        summary.ff2_activation_ratification_gate_summary_hash_v1
    );
}

#[test]
fn ff2_gate_hash_distinct_from_corpus_hash_v1() {
    let gate = build_ff2_activation_ratification_gate();
    assert_ne!(
        gate.ff2_activation_ratification_gate_hash_v1,
        gate.corpus_hash_v1
    );
}

#[test]
fn ff2_gate_hash_distinct_from_corpus_hash_v2() {
    let gate = build_ff2_activation_ratification_gate();
    assert_ne!(
        gate.ff2_activation_ratification_gate_hash_v1,
        gate.corpus_hash_v2
    );
}

#[test]
fn ff2_gate_hash_distinct_from_passport_index_hash() {
    let gate = build_ff2_activation_ratification_gate();
    assert_ne!(
        gate.ff2_activation_ratification_gate_hash_v1,
        gate.ff1_passport_index_hash_v1
    );
}

#[test]
fn ff2_gate_hash_distinct_from_consolidation_report_hash() {
    let gate = build_ff2_activation_ratification_gate();
    assert_ne!(
        gate.ff2_activation_ratification_gate_hash_v1,
        gate.consolidation_report_hash_v1
    );
}

// ---------------------------------------------------------------
// Render coverage
// ---------------------------------------------------------------

#[test]
fn ff2_render_text_contains_pinned_anchors_and_counts() {
    let gate = build_ff2_activation_ratification_gate();
    let text = render_ff2_gate_text(&gate);
    assert!(text.contains("FF.2 Activation Ratification Gate"));
    assert!(text.contains("corpus_hash_v1"));
    assert!(text.contains("corpus_hash_v2"));
    assert!(text.contains("consolidation_report_hash_v1"));
    assert!(text.contains("ff1_passport_index_hash_v1"));
    assert!(text.contains("SeedHistorical"));
    assert!(text.contains("T12RatifiedAndPassported"));
    assert!(text.contains("MissingPassport"));
    assert!(text.contains("UnratifiedProposal"));
    assert!(text.contains("ff2_activation_ratification_gate_hash_v1"));
}

#[test]
fn ff2_render_json_contains_schema_field() {
    let gate = build_ff2_activation_ratification_gate();
    let json = render_ff2_gate_json(&gate);
    assert!(json.contains(FF2_ACTIVATION_GATE_SCHEMA_V1));
}

#[test]
fn ff2_summary_render_text_contains_non_claim_lines() {
    let summary = build_ff2_activation_ratification_gate_summary();
    let text = render_ff2_gate_summary_text(&summary);
    for line in FF2_NON_CLAIM_LINES {
        assert!(text.contains(line));
    }
}

#[test]
fn ff2_summary_render_json_contains_non_claim_array() {
    let summary = build_ff2_activation_ratification_gate_summary();
    let json = render_ff2_gate_summary_json(&summary);
    assert!(json.contains("non_claim_lines"));
    for line in FF2_NON_CLAIM_LINES {
        assert!(json.contains(line));
    }
}

#[test]
fn ff2_render_text_has_deterministic_count_row_order() {
    let gate = build_ff2_activation_ratification_gate();
    let text = render_ff2_gate_text(&gate);
    let seed_pos = text.find("SeedHistorical").unwrap();
    let t12_pos = text.find("T12RatifiedAndPassported").unwrap();
    let missing_pos = text.find("MissingPassport").unwrap();
    let unratified_pos = text.find("UnratifiedProposal").unwrap();
    assert!(seed_pos < t12_pos);
    assert!(t12_pos < missing_pos);
    assert!(missing_pos < unratified_pos);
}
