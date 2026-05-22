//! FF.3 acceptance suite — registry-generation gate invariants
//! for `corpus_hash_v2`-ratified + FF.1-passported source
//! authority.
//!
//! Eight panel-required load-bearing negatives pin the contract
//! discipline FF.3 exists to prove:
//!
//! * `ff3_rejects_detector_spec_for_unratified_proposal`
//! * `ff3_rejects_detector_spec_for_missing_ff1_passport`
//! * `ff3_rejects_detector_spec_when_corpus_hash_v2_mismatch`
//! * `ff3_rejects_detector_spec_when_passport_index_hash_mismatch`
//! * `ff3_rejects_detector_spec_from_ad_hoc_record`
//! * `ff3_rejects_detector_spec_with_unknown_source_authority`
//! * `ff3_rejects_registry_generation_that_skips_ff2_ratification_gate`
//! * `ff3_rejects_registry_generation_that_mutates_existing_registry_hash`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > FF.2 blocks unratified activation;
//! > FF.3 blocks unratified registry generation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
};
use dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::{
    build_ff3_registry_generation_gate, build_ff3_registry_generation_gate_from,
    build_ff3_registry_generation_gate_summary,
    build_ff3_registry_generation_gate_summary_from_gate, default_registry_generation_candidates,
    render_ff3_gate_json, render_ff3_gate_summary_json, render_ff3_gate_summary_text,
    render_ff3_gate_text, verify_ff3, Ff3RegistryGenerationCandidate,
    Ff3RegistryGenerationEligibility, Ff3SourceAuthority, Ff3VerifyError, Ff3VerifyErrorKind,
    FF3_NON_CLAIM_LINES, FF3_REGISTRY_GENERATION_GATE_DOMAIN_V1,
    FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1, FF3_REGISTRY_GENERATION_GATE_SUMMARY_DOMAIN_V1,
    FF3_REGISTRY_GENERATION_GATE_SUMMARY_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

// Helper that builds the standard (report, passport_index,
// ff2_gate) triple used by most tests.
fn live_triple() -> (
    dsfb_gpu_atlas_corpus::consolidate::ConsolidationReport,
    dsfb_gpu_atlas_corpus::ff1_passport_materialisation::Ff1PassportIndex,
    dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::Ff2ActivationRatificationGate,
) {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let activation_candidate_ids = default_candidate_ids(&passport_index);
    let ff2_gate = build_ff2_activation_ratification_gate_from(
        &report,
        &passport_index,
        &activation_candidate_ids,
    );
    (report, passport_index, ff2_gate)
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #1
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_for_unratified_proposal() {
    let (report, passport_index, ff2_gate) = live_triple();
    // Inject a synthetic id outside SEED and outside ratified-
    // expansion, claiming T12RatifiedAndPassported. The
    // classifier MUST emit RejectedUnratifiedProposal.
    let bogus_id: u32 = 9_999_999;
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: bogus_id,
        claimed_source_authority: Ff3SourceAuthority::T12RatifiedAndPassported,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let gate =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == bogus_id)
        .expect("bogus id must produce a decision");
    assert_eq!(
        d.eligibility,
        Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal
    );
    assert_eq!(
        d.rejection_reason_wire_name,
        Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal.as_str()
    );

    // Verifier admits when classifier correctly emitted
    // RejectedUnratifiedProposal.
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.is_empty(), "verifier should admit: {errs:?}");

    // Mutate the decision to claim Eligible anyway — verifier
    // MUST reject.
    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == bogus_id)
        .unwrap();
    mutated.decisions[pos].eligibility = Ff3RegistryGenerationEligibility::Eligible;
    mutated.decisions[pos].eligibility_wire_name =
        Ff3RegistryGenerationEligibility::Eligible.as_str();
    mutated.decisions[pos].rejection_reason_wire_name = "";
    let errs = verify_ff3(&mutated, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecForUnratifiedProposal { canonical_id, .. }
            if canonical_id == bogus_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #2
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_for_missing_ff1_passport() {
    let report = build_consolidation_report();
    let live_index = build_ff1_passport_index_from(&report);
    // Drop one passport so the dropped id becomes "ratified
    // but no passport" under the shrunk index.
    let mut shrunk = live_index.clone();
    let dropped_id = shrunk.passports[0].canonical_id;
    shrunk.passports.remove(0);

    let activation_candidate_ids = default_candidate_ids(&shrunk);
    let ff2_gate =
        build_ff2_activation_ratification_gate_from(&report, &shrunk, &activation_candidate_ids);

    let mut candidates = default_registry_generation_candidates(&shrunk);
    // dropped_id is no longer in the default candidates
    // (because default uses the shrunk passport index); inject
    // it explicitly with the T12RatifiedAndPassported claim.
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: dropped_id,
        claimed_source_authority: Ff3SourceAuthority::T12RatifiedAndPassported,
    });
    candidates.sort_by_key(|c| c.canonical_id);

    let gate = build_ff3_registry_generation_gate_from(&report, &shrunk, &ff2_gate, &candidates);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == dropped_id)
        .expect("dropped id must produce a decision");
    assert_eq!(
        d.eligibility,
        Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport
    );

    // Mutate to claim Eligible — verifier rejects.
    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == dropped_id)
        .unwrap();
    mutated.decisions[pos].eligibility = Ff3RegistryGenerationEligibility::Eligible;
    mutated.decisions[pos].eligibility_wire_name =
        Ff3RegistryGenerationEligibility::Eligible.as_str();
    mutated.decisions[pos].rejection_reason_wire_name = "";
    let errs = verify_ff3(&mutated, &report, &shrunk, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecForMissingFf1Passport { canonical_id }
            if canonical_id == dropped_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #3
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_when_corpus_hash_v2_mismatch() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.corpus_hash_v2 = [0xff; 32];
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecWhenCorpusHashV2Mismatch { claimed, actual }
            if claimed == [0xff; 32] && actual == report.corpus_hash_v2
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #4
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_when_passport_index_hash_mismatch() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.ff1_passport_index_hash_v1 = [0xee; 32];
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecWhenPassportIndexHashMismatch { claimed, actual }
            if claimed == [0xee; 32] && actual == passport_index.ff1_passport_index_hash_v1
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #5
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_from_ad_hoc_record() {
    let (report, passport_index, ff2_gate) = live_triple();
    let bogus_id: u32 = 7_777_777;
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: bogus_id,
        claimed_source_authority: Ff3SourceAuthority::AdHocUnsanctioned,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let gate =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == bogus_id)
        .unwrap();
    assert_eq!(
        d.eligibility,
        Ff3RegistryGenerationEligibility::RejectedAdHocRecord
    );

    // Mutate to claim Eligible — verifier rejects under R.5.
    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == bogus_id)
        .unwrap();
    mutated.decisions[pos].eligibility = Ff3RegistryGenerationEligibility::Eligible;
    mutated.decisions[pos].eligibility_wire_name =
        Ff3RegistryGenerationEligibility::Eligible.as_str();
    mutated.decisions[pos].rejection_reason_wire_name = "";
    let errs = verify_ff3(&mutated, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecFromAdHocRecord { canonical_id }
            if canonical_id == bogus_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #6
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_detector_spec_with_unknown_source_authority() {
    let (report, passport_index, ff2_gate) = live_triple();
    let bogus_id: u32 = 6_666_666;
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: bogus_id,
        claimed_source_authority: Ff3SourceAuthority::UnknownExternal,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let gate =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    let d = gate
        .decisions
        .iter()
        .find(|d| d.canonical_id == bogus_id)
        .unwrap();
    assert_eq!(
        d.eligibility,
        Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority
    );

    let mut mutated = gate.clone();
    let pos = mutated
        .decisions
        .iter()
        .position(|d| d.canonical_id == bogus_id)
        .unwrap();
    mutated.decisions[pos].eligibility = Ff3RegistryGenerationEligibility::Eligible;
    mutated.decisions[pos].eligibility_wire_name =
        Ff3RegistryGenerationEligibility::Eligible.as_str();
    mutated.decisions[pos].rejection_reason_wire_name = "";
    let errs = verify_ff3(&mutated, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DetectorSpecWithUnknownSourceAuthority { canonical_id }
            if canonical_id == bogus_id
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #7
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_registry_generation_that_skips_ff2_ratification_gate() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.ff2_activation_ratification_gate_hash_v1 = [0xab; 32];
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::RegistryGenerationThatSkipsFf2RatificationGate { claimed, actual }
            if claimed == [0xab; 32]
                && actual == ff2_gate.ff2_activation_ratification_gate_hash_v1
    )));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative #8
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_registry_generation_that_mutates_existing_registry_hash() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    // Inflate eligible_count beyond the FF.2 eligible total.
    // The verifier's R.8 rule rejects when FF.3 claims more
    // eligibles than FF.2 admits.
    let ff2_eligible = ff2_gate.seed_historical_count + ff2_gate.t12_ratified_and_passported_count;
    gate.eligible_count = ff2_eligible + 1;
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::RegistryGenerationThatMutatesExistingRegistryHash {
            ff3_eligible_count,
            ff2_eligible_count,
        }
            if ff3_eligible_count == ff2_eligible + 1
                && ff2_eligible_count == ff2_eligible
    )));
}

// ---------------------------------------------------------------
// Default-build invariants
// ---------------------------------------------------------------

#[test]
fn ff3_default_build_is_admissible_under_verifier() {
    let (report, passport_index, ff2_gate) = live_triple();
    let gate = build_ff3_registry_generation_gate();
    let errs: Vec<Ff3VerifyError> = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(
        errs.is_empty(),
        "default FF.3 gate must verify cleanly: {errs:?}"
    );
}

#[test]
fn ff3_default_build_has_zero_rejected_decisions() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(gate.rejected_unratified_proposal_count, 0);
    assert_eq!(gate.rejected_missing_ff1_passport_count, 0);
    assert_eq!(gate.rejected_corpus_hash_v2_mismatch_count, 0);
    assert_eq!(gate.rejected_passport_index_hash_mismatch_count, 0);
    assert_eq!(gate.rejected_ad_hoc_record_count, 0);
    assert_eq!(gate.rejected_unknown_source_authority_count, 0);
}

#[test]
fn ff3_default_build_eligible_count_is_one_hundred_fifty_two() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(gate.eligible_count, 54 + 98);
}

#[test]
fn ff3_default_build_total_equals_eligible_under_default_candidates() {
    let gate = build_ff3_registry_generation_gate();
    let total = gate.eligible_count
        + gate.rejected_unratified_proposal_count
        + gate.rejected_missing_ff1_passport_count
        + gate.rejected_corpus_hash_v2_mismatch_count
        + gate.rejected_passport_index_hash_mismatch_count
        + gate.rejected_ad_hoc_record_count
        + gate.rejected_unknown_source_authority_count;
    assert_eq!(total, u32::try_from(gate.decisions.len()).unwrap());
}

#[test]
fn ff3_default_build_eligible_count_matches_ff2_eligible_count() {
    let (_report, _passport_index, ff2_gate) = live_triple();
    let gate = build_ff3_registry_generation_gate();
    let ff2_eligible = ff2_gate.seed_historical_count + ff2_gate.t12_ratified_and_passported_count;
    assert_eq!(gate.eligible_count, ff2_eligible);
}

// ---------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------

#[test]
fn ff3_gate_hash_is_deterministic_across_two_builds() {
    let g1 = build_ff3_registry_generation_gate();
    let g2 = build_ff3_registry_generation_gate();
    assert_eq!(
        g1.ff3_registry_generation_gate_hash_v1,
        g2.ff3_registry_generation_gate_hash_v1
    );
}

#[test]
fn ff3_gate_summary_hash_is_deterministic_across_two_builds() {
    let s1 = build_ff3_registry_generation_gate_summary();
    let s2 = build_ff3_registry_generation_gate_summary();
    assert_eq!(
        s1.ff3_registry_generation_gate_summary_hash_v1,
        s2.ff3_registry_generation_gate_summary_hash_v1
    );
}

#[test]
fn ff3_gate_decisions_sorted_ascending() {
    let gate = build_ff3_registry_generation_gate();
    for w in gate.decisions.windows(2) {
        assert!(w[0].canonical_id < w[1].canonical_id);
    }
}

#[test]
fn ff3_text_render_byte_stable() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(render_ff3_gate_text(&gate), render_ff3_gate_text(&gate));
}

#[test]
fn ff3_json_render_byte_stable() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(render_ff3_gate_json(&gate), render_ff3_gate_json(&gate));
}

#[test]
fn ff3_summary_text_render_byte_stable() {
    let s = build_ff3_registry_generation_gate_summary();
    assert_eq!(
        render_ff3_gate_summary_text(&s),
        render_ff3_gate_summary_text(&s)
    );
}

#[test]
fn ff3_summary_json_render_byte_stable() {
    let s = build_ff3_registry_generation_gate_summary();
    assert_eq!(
        render_ff3_gate_summary_json(&s),
        render_ff3_gate_summary_json(&s)
    );
}

// ---------------------------------------------------------------
// Sensitivity
// ---------------------------------------------------------------

#[test]
fn ff3_gate_hash_changes_when_candidate_set_changes() {
    let (report, passport_index, ff2_gate) = live_triple();
    let baseline = build_ff3_registry_generation_gate();
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: 9_999_999,
        claimed_source_authority: Ff3SourceAuthority::T12RatifiedAndPassported,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let altered =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    assert_ne!(
        baseline.ff3_registry_generation_gate_hash_v1,
        altered.ff3_registry_generation_gate_hash_v1
    );
}

#[test]
fn ff3_summary_hash_changes_when_gate_hash_changes() {
    let (report, passport_index, ff2_gate) = live_triple();
    let baseline = build_ff3_registry_generation_gate_summary();
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: 5_555_555,
        claimed_source_authority: Ff3SourceAuthority::T12RatifiedAndPassported,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let altered_gate =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    let altered_summary = build_ff3_registry_generation_gate_summary_from_gate(altered_gate);
    assert_ne!(
        baseline.ff3_registry_generation_gate_summary_hash_v1,
        altered_summary.ff3_registry_generation_gate_summary_hash_v1
    );
}

// ---------------------------------------------------------------
// Upstream-anchor invariance
// ---------------------------------------------------------------

#[test]
fn ff3_does_not_mutate_corpus_hash_v1() {
    let before = compute_corpus_hash_v1().bytes;
    let _ = build_ff3_registry_generation_gate();
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

#[test]
fn ff3_does_not_mutate_corpus_hash_v2() {
    let before = build_consolidation_report().corpus_hash_v2;
    let _ = build_ff3_registry_generation_gate();
    let after = build_consolidation_report().corpus_hash_v2;
    assert_eq!(before, after);
}

#[test]
fn ff3_does_not_mutate_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let before = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    let _ = build_ff3_registry_generation_gate();
    let after = build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff3_does_not_mutate_ff2_activation_ratification_gate_hash_v1() {
    let (_report, _passport_index, before_ff2) = live_triple();
    let _ = build_ff3_registry_generation_gate();
    let (_, _, after_ff2) = live_triple();
    assert_eq!(
        before_ff2.ff2_activation_ratification_gate_hash_v1,
        after_ff2.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn ff3_does_not_mutate_consolidation_report_hash_v1() {
    let before = build_consolidation_report().consolidation_report_hash_v1;
    let _ = build_ff3_registry_generation_gate();
    let after = build_consolidation_report().consolidation_report_hash_v1;
    assert_eq!(before, after);
}

#[test]
fn ff3_does_not_mutate_seed_len() {
    let before = SEED.len();
    let _ = build_ff3_registry_generation_gate();
    let after = SEED.len();
    assert_eq!(before, 54);
    assert_eq!(after, 54);
}

// ---------------------------------------------------------------
// Pinned-anchor cross-check invariants
// ---------------------------------------------------------------

#[test]
fn ff3_gate_pins_live_corpus_hash_v1() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(gate.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn ff3_gate_pins_live_corpus_hash_v2() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(
        gate.corpus_hash_v2,
        build_consolidation_report().corpus_hash_v2
    );
}

#[test]
fn ff3_gate_pins_live_consolidation_report_hash_v1() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(
        gate.consolidation_report_hash_v1,
        build_consolidation_report().consolidation_report_hash_v1
    );
}

#[test]
fn ff3_gate_pins_live_ff1_passport_index_hash_v1() {
    let r = build_consolidation_report();
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(
        gate.ff1_passport_index_hash_v1,
        build_ff1_passport_index_from(&r).ff1_passport_index_hash_v1
    );
}

#[test]
fn ff3_gate_pins_live_ff2_activation_ratification_gate_hash_v1() {
    let (_, _, ff2_gate) = live_triple();
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(
        gate.ff2_activation_ratification_gate_hash_v1,
        ff2_gate.ff2_activation_ratification_gate_hash_v1
    );
}

// ---------------------------------------------------------------
// Wire-name / domain-separator / non-claim pins
// ---------------------------------------------------------------

#[test]
fn ff3_gate_domain_separator_pin() {
    assert_eq!(
        FF3_REGISTRY_GENERATION_GATE_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0"
    );
}

#[test]
fn ff3_gate_schema_pin() {
    assert_eq!(
        FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1,
        "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1"
    );
}

#[test]
fn ff3_gate_summary_domain_separator_pin() {
    assert_eq!(
        FF3_REGISTRY_GENERATION_GATE_SUMMARY_DOMAIN_V1,
        "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0"
    );
}

#[test]
fn ff3_gate_summary_schema_pin() {
    assert_eq!(
        FF3_REGISTRY_GENERATION_GATE_SUMMARY_SCHEMA_V1,
        "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1"
    );
}

#[test]
fn ff3_source_authority_wire_names_pin() {
    assert_eq!(
        Ff3SourceAuthority::SeedHistorical.as_str(),
        "SeedHistorical"
    );
    assert_eq!(
        Ff3SourceAuthority::T12RatifiedAndPassported.as_str(),
        "T12RatifiedAndPassported"
    );
    assert_eq!(
        Ff3SourceAuthority::AdHocUnsanctioned.as_str(),
        "AdHocUnsanctioned"
    );
    assert_eq!(
        Ff3SourceAuthority::UnknownExternal.as_str(),
        "UnknownExternal"
    );
}

#[test]
fn ff3_eligibility_wire_names_pin() {
    assert_eq!(
        Ff3RegistryGenerationEligibility::Eligible.as_str(),
        "Eligible"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal.as_str(),
        "RejectedUnratifiedProposal"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport.as_str(),
        "RejectedMissingFf1Passport"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedCorpusHashV2Mismatch.as_str(),
        "RejectedCorpusHashV2Mismatch"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedPassportIndexHashMismatch.as_str(),
        "RejectedPassportIndexHashMismatch"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedAdHocRecord.as_str(),
        "RejectedAdHocRecord"
    );
    assert_eq!(
        Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority.as_str(),
        "RejectedUnknownSourceAuthority"
    );
}

#[test]
fn ff3_passes_gate_only_for_eligible() {
    assert!(Ff3RegistryGenerationEligibility::Eligible.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedCorpusHashV2Mismatch.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedPassportIndexHashMismatch.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedAdHocRecord.passes_gate());
    assert!(!Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority.passes_gate());
}

#[test]
fn ff3_non_claim_lines_are_non_empty() {
    assert!(!FF3_NON_CLAIM_LINES.is_empty());
    for line in FF3_NON_CLAIM_LINES {
        assert!(!line.is_empty());
    }
}

#[test]
fn ff3_summary_carries_canonical_non_claim_lines() {
    let s = build_ff3_registry_generation_gate_summary();
    assert_eq!(s.non_claim_lines, FF3_NON_CLAIM_LINES);
}

// ---------------------------------------------------------------
// Field-level invariants
// ---------------------------------------------------------------

#[test]
fn ff3_eligible_decisions_carry_empty_rejection_reason() {
    let gate = build_ff3_registry_generation_gate();
    for d in gate
        .decisions
        .iter()
        .filter(|d| d.eligibility == Ff3RegistryGenerationEligibility::Eligible)
    {
        assert!(d.rejection_reason_wire_name.is_empty());
    }
}

#[test]
fn ff3_eligible_seed_decisions_carry_zero_passport_hash() {
    let gate = build_ff3_registry_generation_gate();
    for d in gate.decisions.iter().filter(|d| {
        d.eligibility == Ff3RegistryGenerationEligibility::Eligible
            && d.claimed_source_authority_wire_name == Ff3SourceAuthority::SeedHistorical.as_str()
    }) {
        assert_eq!(d.cited_passport_hash, [0u8; 32]);
    }
}

#[test]
fn ff3_eligible_t12_decisions_carry_non_zero_passport_hash() {
    let gate = build_ff3_registry_generation_gate();
    for d in gate.decisions.iter().filter(|d| {
        d.eligibility == Ff3RegistryGenerationEligibility::Eligible
            && d.claimed_source_authority_wire_name
                == Ff3SourceAuthority::T12RatifiedAndPassported.as_str()
    }) {
        assert_ne!(d.cited_passport_hash, [0u8; 32]);
    }
}

#[test]
fn ff3_seed_decisions_cover_ids_one_through_fifty_four() {
    let gate = build_ff3_registry_generation_gate();
    let seed_ids: Vec<u32> = gate
        .decisions
        .iter()
        .filter(|d| {
            d.claimed_source_authority_wire_name == Ff3SourceAuthority::SeedHistorical.as_str()
        })
        .map(|d| d.canonical_id)
        .collect();
    let expected: Vec<u32> = (1..=54).collect();
    assert_eq!(seed_ids, expected);
}

#[test]
fn ff3_t12_decisions_only_in_reserved_band() {
    let gate = build_ff3_registry_generation_gate();
    for d in gate.decisions.iter().filter(|d| {
        d.claimed_source_authority_wire_name
            == Ff3SourceAuthority::T12RatifiedAndPassported.as_str()
    }) {
        assert!((5001..=6699).contains(&d.canonical_id));
    }
}

// ---------------------------------------------------------------
// Structural defect rules
// ---------------------------------------------------------------

#[test]
fn ff3_rejects_duplicate_gate_decision_for_same_canonical_id() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    let dup = gate.decisions[0].clone();
    gate.decisions.push(dup);
    gate.decisions.sort_by_key(|d| d.canonical_id);
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DuplicateGateDecisionForSameCanonicalId { .. }
    )));
}

#[test]
fn ff3_rejects_gate_decisions_not_sorted_ascending() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.decisions.reverse();
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff3VerifyErrorKind::GateDecisionsNotSortedAscending)));
}

#[test]
fn ff3_rejects_corpus_hash_v1_mismatch() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.corpus_hash_v1 = [0xaa; 32];
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs
        .iter()
        .any(|e| matches!(e.kind, Ff3VerifyErrorKind::CorpusHashV1Mismatch { .. })));
}

#[test]
fn ff3_rejects_consolidation_report_hash_v1_mismatch() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    gate.consolidation_report_hash_v1 = [0xbb; 32];
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::ConsolidationReportHashV1Mismatch { .. }
    )));
}

#[test]
fn ff3_rejects_eligible_decision_with_nonempty_rejection_reason() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    // Inject a non-empty rejection reason on an Eligible decision.
    let pos = gate
        .decisions
        .iter()
        .position(|d| d.eligibility == Ff3RegistryGenerationEligibility::Eligible)
        .unwrap();
    gate.decisions[pos].rejection_reason_wire_name = "BogusReason";
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::EligibleDecisionCarriesNonEmptyRejectionReason { .. }
    )));
}

#[test]
fn ff3_rejects_rejection_decision_with_empty_rejection_reason() {
    let (report, passport_index, ff2_gate) = live_triple();
    // Build a gate with an AdHocUnsanctioned record.
    let bogus_id: u32 = 5_000_000;
    let mut candidates = default_registry_generation_candidates(&passport_index);
    candidates.push(Ff3RegistryGenerationCandidate {
        canonical_id: bogus_id,
        claimed_source_authority: Ff3SourceAuthority::AdHocUnsanctioned,
    });
    candidates.sort_by_key(|c| c.canonical_id);
    let mut gate =
        build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates);
    let pos = gate
        .decisions
        .iter()
        .position(|d| d.canonical_id == bogus_id)
        .unwrap();
    gate.decisions[pos].rejection_reason_wire_name = "";
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::RejectionDecisionCarriesEmptyRejectionReason { .. }
    )));
}

#[test]
fn ff3_rejects_decision_classification_inconsistent_with_claim() {
    let (report, passport_index, ff2_gate) = live_triple();
    let mut gate = build_ff3_registry_generation_gate();
    // Reclassify the first SeedHistorical decision as
    // RejectedUnratifiedProposal — that combination is forbidden
    // because Unratified rejections apply only to ratified-claim
    // candidates.
    let pos = gate
        .decisions
        .iter()
        .position(|d| {
            d.claimed_source_authority_wire_name == Ff3SourceAuthority::SeedHistorical.as_str()
                && d.eligibility == Ff3RegistryGenerationEligibility::Eligible
        })
        .unwrap();
    gate.decisions[pos].eligibility = Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal;
    gate.decisions[pos].eligibility_wire_name =
        Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal.as_str();
    gate.decisions[pos].rejection_reason_wire_name =
        Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal.as_str();
    let errs = verify_ff3(&gate, &report, &passport_index, &ff2_gate);
    assert!(errs.iter().any(|e| matches!(
        e.kind,
        Ff3VerifyErrorKind::DecisionClassificationInconsistentWithClaim { .. }
    )));
}

// ---------------------------------------------------------------
// Count-shape regression sentinels
// ---------------------------------------------------------------

#[test]
fn ff3_eligible_count_is_one_hundred_fifty_two() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(gate.eligible_count, 152);
}

#[test]
fn ff3_total_decision_count_is_one_hundred_fifty_two() {
    let gate = build_ff3_registry_generation_gate();
    assert_eq!(gate.decisions.len(), 152);
}

// ---------------------------------------------------------------
// Hash-namespace distinctness
// ---------------------------------------------------------------

#[test]
fn ff3_gate_hash_distinct_from_summary_hash() {
    let s = build_ff3_registry_generation_gate_summary();
    assert_ne!(
        s.gate.ff3_registry_generation_gate_hash_v1,
        s.ff3_registry_generation_gate_summary_hash_v1
    );
}

#[test]
fn ff3_gate_hash_distinct_from_corpus_hash_v1() {
    let gate = build_ff3_registry_generation_gate();
    assert_ne!(
        gate.ff3_registry_generation_gate_hash_v1,
        gate.corpus_hash_v1
    );
}

#[test]
fn ff3_gate_hash_distinct_from_corpus_hash_v2() {
    let gate = build_ff3_registry_generation_gate();
    assert_ne!(
        gate.ff3_registry_generation_gate_hash_v1,
        gate.corpus_hash_v2
    );
}

#[test]
fn ff3_gate_hash_distinct_from_consolidation_report_hash() {
    let gate = build_ff3_registry_generation_gate();
    assert_ne!(
        gate.ff3_registry_generation_gate_hash_v1,
        gate.consolidation_report_hash_v1
    );
}

#[test]
fn ff3_gate_hash_distinct_from_ff1_passport_index_hash() {
    let gate = build_ff3_registry_generation_gate();
    assert_ne!(
        gate.ff3_registry_generation_gate_hash_v1,
        gate.ff1_passport_index_hash_v1
    );
}

#[test]
fn ff3_gate_hash_distinct_from_ff2_gate_hash() {
    let gate = build_ff3_registry_generation_gate();
    assert_ne!(
        gate.ff3_registry_generation_gate_hash_v1,
        gate.ff2_activation_ratification_gate_hash_v1
    );
}

// ---------------------------------------------------------------
// Render coverage
// ---------------------------------------------------------------

#[test]
fn ff3_render_text_contains_pinned_anchors_and_counts() {
    let gate = build_ff3_registry_generation_gate();
    let text = render_ff3_gate_text(&gate);
    assert!(text.contains("FF.3 Registry Generation Gate"));
    assert!(text.contains("corpus_hash_v1"));
    assert!(text.contains("corpus_hash_v2"));
    assert!(text.contains("consolidation_report_hash_v1"));
    assert!(text.contains("ff1_passport_index_hash_v1"));
    assert!(text.contains("ff2_activation_ratification_gate_hash_v1"));
    assert!(text.contains("Eligible"));
    assert!(text.contains("RejectedUnratifiedProposal"));
    assert!(text.contains("RejectedMissingFf1Passport"));
    assert!(text.contains("RejectedCorpusHashV2Mismatch"));
    assert!(text.contains("RejectedPassportIndexHashMismatch"));
    assert!(text.contains("RejectedAdHocRecord"));
    assert!(text.contains("RejectedUnknownSourceAuthority"));
    assert!(text.contains("ff3_registry_generation_gate_hash_v1"));
}

#[test]
fn ff3_render_json_contains_schema_field() {
    let gate = build_ff3_registry_generation_gate();
    let json = render_ff3_gate_json(&gate);
    assert!(json.contains(FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1));
}

#[test]
fn ff3_summary_render_text_contains_non_claim_lines() {
    let s = build_ff3_registry_generation_gate_summary();
    let text = render_ff3_gate_summary_text(&s);
    for line in FF3_NON_CLAIM_LINES {
        assert!(text.contains(line));
    }
}

#[test]
fn ff3_summary_render_json_contains_non_claim_array() {
    let s = build_ff3_registry_generation_gate_summary();
    let json = render_ff3_gate_summary_json(&s);
    assert!(json.contains("non_claim_lines"));
    for line in FF3_NON_CLAIM_LINES {
        assert!(json.contains(line));
    }
}

#[test]
fn ff3_render_text_has_deterministic_count_row_order() {
    let gate = build_ff3_registry_generation_gate();
    let text = render_ff3_gate_text(&gate);
    let positions = [
        ("Eligible", text.find("Eligible").unwrap()),
        (
            "RejectedUnratifiedProposal",
            text.find("RejectedUnratifiedProposal").unwrap(),
        ),
        (
            "RejectedMissingFf1Passport",
            text.find("RejectedMissingFf1Passport").unwrap(),
        ),
        (
            "RejectedCorpusHashV2Mismatch",
            text.find("RejectedCorpusHashV2Mismatch").unwrap(),
        ),
        (
            "RejectedPassportIndexHashMismatch",
            text.find("RejectedPassportIndexHashMismatch").unwrap(),
        ),
        (
            "RejectedAdHocRecord",
            text.find("RejectedAdHocRecord").unwrap(),
        ),
        (
            "RejectedUnknownSourceAuthority",
            text.find("RejectedUnknownSourceAuthority").unwrap(),
        ),
    ];
    for w in positions.windows(2) {
        assert!(
            w[0].1 < w[1].1,
            "expected `{}` to appear before `{}` in render",
            w[0].0,
            w[1].0
        );
    }
}
