//! T.12.j acceptance suite — Medical / Biosignal expansion
//! proposal invariants.
//!
//! Thirteen panel-required load-bearing negatives pin the
//! signal-source + sampling + filtering + morphology / interval
//! / spectral measurement-law + artifact-confuser-profile +
//! decision-functional discipline T.12.j exists to prove
//! ("count signal witnesses, not diagnoses; no sampling /
//! filtering / morphology law, no canonical admission").
//!
//! Most-important load-bearing negative:
//! `t12_j_rejects_diagnostic_claim_language` — every
//! CanonicalAddition reason text must describe its record as a
//! signal / morphology / interval / envelope / artifact /
//! spectral measurement witness, NEVER as a clinical diagnosis.
//! The court does NOT issue diagnostic verdicts.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    compute_corpus_amendment_proposal_hash_v1, render_amendment_proposal_json,
    render_amendment_proposal_text, seed_proof_of_life_proposal, verify_amendment_proposal,
    AmendmentVerifyErrorKind, ProposalStatus, ProposedDedupRecord, ProposedPrimitive, ProposerRole,
    RejectionRecord, SourceClass,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::t12_a_spc::seed_t12_a_spc_proposal;
use dsfb_gpu_atlas_corpus::t12_b_scd::seed_t12_b_scd_proposal;
use dsfb_gpu_atlas_corpus::t12_c_drift::seed_t12_c_drift_proposal;
use dsfb_gpu_atlas_corpus::t12_d_robust::seed_t12_d_robust_proposal;
use dsfb_gpu_atlas_corpus::t12_e_spectral::seed_t12_e_spectral_proposal;
use dsfb_gpu_atlas_corpus::t12_f_timeseries::seed_t12_f_timeseries_proposal;
use dsfb_gpu_atlas_corpus::t12_g_graph::seed_t12_g_graph_proposal;
use dsfb_gpu_atlas_corpus::t12_h_dataquality::seed_t12_h_dataquality_proposal;
use dsfb_gpu_atlas_corpus::t12_i_observability::seed_t12_i_observability_proposal;
use dsfb_gpu_atlas_corpus::t12_j_biosignal::{
    seed_t12_j_biosignal_proposal, BASELINE_WANDER_RESERVED_CANONICAL_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID,
    CLIPPING_RESERVED_CANONICAL_ID, FFT_BAND_ENERGY_SEED_ID, HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID,
    HRV_TIME_DOMAIN_SHIFT_SEED_ID, HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID,
    LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID, LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID,
    MOTION_ARTIFACT_RESERVED_CANONICAL_ID, PR_INTERVAL_RESERVED_CANONICAL_ID,
    P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID, QRS_WIDTH_SEED_ID, QT_INTERVAL_RESERVED_CANONICAL_ID,
    RESIDUAL_ENVELOPE_EXIT_SEED_ID, RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID,
    R_PEAK_INTERVAL_SEED_ID, SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID,
    ST_SEGMENT_DEVIATION_SEED_ID, T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12J_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (R_PEAK_INTERVAL_SEED_ID, "R-peak interval anomaly"),
    (HRV_TIME_DOMAIN_SHIFT_SEED_ID, "HRV time-domain shift"),
    (QRS_WIDTH_SEED_ID, "QRS width anomaly"),
    (ST_SEGMENT_DEVIATION_SEED_ID, "ST-segment deviation proxy"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn bio_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_j_biosignal_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Biosignal proposal failed verifier: {errors:?}"
    );
}

#[test]
fn bio_proposal_has_open_status() {
    let p = seed_t12_j_biosignal_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn bio_proposal_targets_medical_biosignal() {
    let p = seed_t12_j_biosignal_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::MedicalBiosignal
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_j_does_not_mutate_seed_len`.
#[test]
fn t12_j_does_not_mutate_seed_len() {
    let _ = seed_t12_j_biosignal_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn bio_proposal_proposes_fourteen_primitives() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 14);
}

#[test]
fn bio_proposal_proposes_zero_aliases() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn bio_proposal_proposes_twenty_dedup_records() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 20);
}

#[test]
fn bio_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn bio_proposal_proposes_nine_source_refs() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn bio_delta_has_eight_new_canonical_records() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn bio_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_j_biosignal_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn bio_proposal_carries_two_rejection_records() {
    let p = seed_t12_j_biosignal_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.j must carry TWO RejectedNotDeterministic records \
        (fourth T.12.x with two, following T.12.g / T.12.h / T.12.i)"
    );
}

#[test]
fn bio_proposal_court_delta_category_counts() {
    let p = seed_t12_j_biosignal_proposal();
    let mut canonical = 0;
    let mut existing = 0;
    let mut transfer = 0;
    let mut paramof = 0;
    let mut rejected = 0;
    for r in &p.body.proposed_dedup_records {
        match r.decision_wire_name {
            CATEGORY_CANONICAL_ADDITION => canonical += 1,
            CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION => existing += 1,
            CATEGORY_DOMAIN_TRANSFER_OF => transfer += 1,
            CATEGORY_PARAMETERIZATION_OF => paramof += 1,
            CATEGORY_REJECTED_NOT_DETERMINISTIC => rejected += 1,
            other => panic!("unexpected wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 8);
    assert_eq!(existing, 4);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn bio_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_j_biosignal_proposal();
    let b = seed_t12_j_biosignal_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_bio_proposal_hash_matches_stored() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn bio_proposal_hash_is_distinct_from_all_prior_t12x() {
    let j = seed_t12_j_biosignal_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    let ts = seed_t12_f_timeseries_proposal();
    let graph = seed_t12_g_graph_proposal();
    let dq = seed_t12_h_dataquality_proposal();
    let obs = seed_t12_i_observability_proposal();
    for other in [pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs] {
        assert_ne!(
            j.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.j hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative #12 (panel-required):
/// `t12_j_hash_changes_when_sampling_or_filter_law_changes`.
#[test]
fn t12_j_hash_changes_when_sampling_or_filter_law_changes() {
    let p_a = seed_t12_j_biosignal_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BASELINE_WANDER_RESERVED_CANONICAL_ID
        })
        .expect("Baseline wander CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED sampling / filter law for hash-sensitivity test",
    };
    let new_batch = build_expansion_batch(
        p_a.body.batch_id,
        p_a.body.source_class,
        p_a.body.proposed_primitives.clone(),
        p_a.body.proposed_aliases.clone(),
        records,
        p_a.body.proposed_genealogy_edges.clone(),
        p_a.body.proposed_source_refs.clone(),
    );
    let p_b = build_amendment_proposal(
        p_a.proposal_id,
        p_a.motivation,
        p_a.target_source_class,
        new_batch,
        p_a.dedup_court_delta.clone(),
        p_a.status,
        p_a.proposer_role,
        p_a.created_at_commit,
    );
    assert_ne!(
        p_a.corpus_amendment_proposal_hash_v1,
        p_b.corpus_amendment_proposal_hash_v1
    );
}

// ---------------------------------------------------------------
// SEED collision load-bearing negative #13
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_j_collision_batch",
        SourceClass::MedicalBiosignal,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "biosignal collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_j_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_j_collision_proposal",
        "Defective T.12.j proposal duplicating an existing SEED canonical.",
        SourceClass::MedicalBiosignal,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_j_test",
    )
}

/// Load-bearing negative #13 (panel-required):
/// `t12_j_existing_seed_collision_requires_authority_resolution`.
#[test]
fn t12_j_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12J_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id);
        let errors = verify_amendment_proposal(&p);
        assert!(
            errors.iter().any(|e| matches!(
                e.kind,
                AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id
                } if canonical_id.0 == *seed_id
            )),
            "expected collision rule fire for SEED id {seed_id} ({label})"
        );
    }
}

/// Load-bearing negative #2 (panel-required):
/// `t12_j_rejects_r_peak_duplicate_without_existing_authority_resolution`.
#[test]
fn t12_j_rejects_r_peak_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == R_PEAK_INTERVAL_SEED_ID
        })
        .expect("R-peak interval must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("signal source") || r.contains("ecg lead"));
    assert!(r.contains("sampling rate"));
    assert!(r.contains("filtering law"));
    assert!(r.contains("r-peak fiducial-detection law"));
    assert!(r.contains("artifact exclusion"));
    // Panel-locked non-claim must appear.
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #3 (panel-required):
/// `t12_j_rejects_qrs_width_without_sampling_and_qrs_detection_law`.
#[test]
fn t12_j_rejects_qrs_width_without_sampling_and_qrs_detection_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == QRS_WIDTH_SEED_ID
        })
        .expect("QRS width must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("sampling rate"));
    assert!(r.contains("qrs-detection law"));
    assert!(r.contains("pan-tompkins"));
    assert!(r.contains("width measurement law"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #4 (panel-required):
/// `t12_j_rejects_hrv_without_rr_interval_and_artifact_exclusion_law`.
#[test]
fn t12_j_rejects_hrv_without_rr_interval_and_artifact_exclusion_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == HRV_TIME_DOMAIN_SHIFT_SEED_ID
        })
        .expect("HRV time-domain shift must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("rr-interval extraction law"));
    assert!(r.contains("beat inclusion") || r.contains("beat exclusion"));
    assert!(r.contains("artifact correction"));
    assert!(r.contains("task force 1996"));
    // Rust string-literal `\\\n` continuation joins with the
    // trailing space, so "time-\n     domain statistic" becomes
    // "time- domain statistic". Match both halves.
    assert!(r.contains("time-") && r.contains("domain statistic"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #5 (panel-required):
/// `t12_j_rejects_st_deviation_without_lead_and_baseline_law`.
#[test]
fn t12_j_rejects_st_deviation_without_lead_and_baseline_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == ST_SEGMENT_DEVIATION_SEED_ID
        })
        .expect("ST-segment deviation must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("lead / channel identity"));
    assert!(r.contains("isoelectric baseline definition"));
    assert!(r.contains("j-point detection law"));
    assert!(r.contains("st-deviation measurement law"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #6 (panel-required):
/// `t12_j_rejects_spectral_hrv_without_band_and_resampling_law`.
#[test]
fn t12_j_rejects_spectral_hrv_without_band_and_resampling_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("Spectral HRV band shift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("rr-interval extraction law"));
    assert!(r.contains("resampling law"));
    assert!(r.contains("spectral-estimation method"));
    assert!(r.contains("welch") || r.contains("lomb-scargle"));
    // Rust string-literal `\\\n` continuation joins with the
    // trailing space, so "frequency-\n     band definitions"
    // becomes "frequency- band definitions". Match both halves.
    assert!(r.contains("frequency-") && r.contains("band definitions"));
    assert!(r.contains("vlf / lf / hf"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #7 (panel-required):
/// `t12_j_rejects_baseline_wander_without_filter_and_frequency_law`.
#[test]
fn t12_j_rejects_baseline_wander_without_filter_and_frequency_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BASELINE_WANDER_RESERVED_CANONICAL_ID
        })
        .expect("Baseline wander must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("high-pass filter cutoff frequency"));
    assert!(r.contains("wander-band frequency law"));
    assert!(r.contains("below 0.5 hz"));
    assert!(r.contains("signal type"));
    assert!(r.contains("ecg / ppg / emg"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #8 (panel-required):
/// `t12_j_rejects_motion_artifact_without_artifact_signal_definition`.
#[test]
fn t12_j_rejects_motion_artifact_without_artifact_signal_definition() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == MOTION_ARTIFACT_RESERVED_CANONICAL_ID
        })
        .expect("Motion artifact must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("artifact signal definition"));
    assert!(
        r.contains("accelerometer")
            || r.contains("amplitude-saturation")
            || r.contains("baseline jump")
    );
    assert!(r.contains("sensor source"));
    assert!(r.contains("decision threshold"));
    assert!(r.contains("confuser handling"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #9 (panel-required):
/// `t12_j_rejects_clipping_without_adc_or_saturation_boundary_law`.
#[test]
fn t12_j_rejects_clipping_without_adc_or_saturation_boundary_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CLIPPING_RESERVED_CANONICAL_ID
        })
        .expect("Clipping detector must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("adc bit-depth") || r.contains("saturation boundary"));
    assert!(r.contains("consecutive-samples-at-boundary threshold"));
    assert!(r.contains("max / min observable sample value"));
    assert!(r.contains("signal witness, not a medical diagnosis"));
}

/// Load-bearing negative #10 (panel-required):
/// `t12_j_rejects_learned_arrhythmia_classifier_without_deterministic_formula`.
#[test]
fn t12_j_rejects_learned_arrhythmia_classifier_without_deterministic_formula() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Learned arrhythmia classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned arrhythmia classifier must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("hannun") || r.contains("deep-learning"));
    // Rust string-literal `\\\n` continuation joins with the
    // trailing space before `\\`, so "model-\n     identification
    // seed" becomes "model- identification seed". Match both
    // halves; same gotcha for training-data anchor / pinned
    // PhysioNet record-hash where the continuation lands.
    assert!(r.contains("model-") && r.contains("identification seed"));
    assert!(
        r.contains("training-data anchor")
            || (r.contains("training-") && r.contains("data anchor"))
    );
    assert!(r.contains("pinned physionet"));
    assert!(r.contains("label schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    // The rejection reason must explicitly disavow diagnostic
    // verdicts.
    assert!(r.contains("does not issue diagnostic verdicts"));
    assert!(r.contains("only to describe what is not admitted"));
}

/// Load-bearing negative #11 (panel-required, MOST IMPORTANT):
/// `t12_j_rejects_diagnostic_claim_language`. Every
/// `CanonicalAddition` AND `ExistingCanonicalAuthorityResolution`
/// reason text must consistently describe its record as a signal
/// witness, NEVER as a clinical diagnosis. Forbidden clinical
/// terms are only allowed inside RejectedNotDeterministic reasons
/// (where they appear to describe what is NOT admitted) or inside
/// the rejection-shell display names.
/// Forbidden positive-diagnostic terms when used as a detector
/// claim. These are commonly-overclaimed clinical diagnoses.
/// Hoisted out of the test body so clippy's
/// `items_after_statements` lint stays clean.
const T12J_FORBIDDEN_DIAGNOSTIC_TERMS: &[&str] = &[
    "arrhythmia",
    "fibrillation",
    "infarction",
    "ischemia",
    "ischaemia",
    "tachycardia",
    "bradycardia",
    "diagnoses ",
    "diagnostic verdict",
];

#[test]
fn t12_j_rejects_diagnostic_claim_language() {
    let p = seed_t12_j_biosignal_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            // Rejection reason text describes what is NOT admitted;
            // skip the forbidden-term scan.
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12J_FORBIDDEN_DIAGNOSTIC_TERMS {
            assert!(
                !lower.contains(term),
                "Record with decision={} canonical_id={} reason text contains \
                forbidden diagnostic term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
        // Every CanonicalAddition / ExistingCanonicalAuthority
        // Resolution must end its scope sentence with the panel-
        // locked "signal witness, not a medical diagnosis" non-
        // claim. (DomainTransferOf reason describes structural
        // relationships, not detector behaviour; ParameterizationOf
        // reason describes the parameterization, not the detector
        // behaviour; so we only require the non-claim on
        // CanonicalAddition and ExistingCanonicalAuthorityResolution
        // reasons.)
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
            || r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
        {
            assert!(
                lower.contains("signal witness, not a medical diagnosis"),
                "Record with decision={} canonical_id={} must declare 'signal \
                witness, not a medical diagnosis': {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                r.reason,
            );
        }
    }
}

/// Rejection contract test #2: clinician-label-only diagnostic
/// rule rejection.
#[test]
fn t12_j_rejects_clinician_label_only_diagnostic_rule() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Clinician-label-only diagnostic rule must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID
        })
        .expect("Clinician-label-only diagnostic rule must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("clinical-labeller-specific judgement"));
    assert!(r.contains("morphology") && r.contains("interval") && r.contains("rhythm law"));
    assert!(r.contains("does not issue diagnostic verdicts"));
    assert!(r.contains("only to describe what is not admitted"));
}

// ---------------------------------------------------------------
// Per-canonical / additional contract assertions
// ---------------------------------------------------------------

#[test]
fn t12_j_p_wave_morphology_declares_full_contract() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID
        })
        .expect("P-wave morphology must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("lead / channel identity"));
    assert!(r.contains("sampling rate"));
    assert!(r.contains("filtering law"));
    // Rust string-literal `\\\n` continuation joins with the
    // trailing space before `\\`, so "P-wave fiducial-\n
    // detection law" becomes "p-wave fiducial- detection law".
    // Match both halves.
    assert!(r.contains("p-wave fiducial-") && r.contains("detection law"));
    assert!(r.contains("amplitude") && r.contains("duration") && r.contains("polarity"));
}

#[test]
fn t12_j_t_wave_morphology_declares_full_contract() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID
        })
        .expect("T-wave morphology must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("t-wave fiducial-detection law"));
    assert!(r.contains("amplitude") && r.contains("duration"));
    assert!(r.contains("polarity") || r.contains("inversion"));
}

#[test]
fn t12_j_qt_interval_declares_extraction_and_rate_correction() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == QT_INTERVAL_RESERVED_CANONICAL_ID
        })
        .expect("QT interval must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("qt extraction law"));
    assert!(r.contains("q-onset"));
    assert!(r.contains("t-offset"));
    // Optional rate-correction formula name must appear.
    let formulas = ["bazett", "fridericia", "framingham", "hodges"];
    let has_formula = formulas.iter().any(|f| r.contains(f));
    assert!(
        has_formula,
        "QT interval reason must name at least one rate-correction formula"
    );
}

#[test]
fn t12_j_pr_interval_declares_extraction_law() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == PR_INTERVAL_RESERVED_CANONICAL_ID
        })
        .expect("PR interval must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("pr extraction law"));
    assert!(r.contains("p-onset"));
    assert!(r.contains("r-onset"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_j_rr_interval_irregularity_is_parameterizationof_r_peak() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID
        })
        .expect("RR-interval irregularity must have ParameterizationOf record");
    assert!(rec.reason.contains("R-peak interval anomaly"));
}

#[test]
fn t12_j_hrv_time_domain_variant_is_parameterizationof_hrv_time_domain_shift() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID
        })
        .expect("HRV time-domain variant must have ParameterizationOf record");
    assert!(rec.reason.contains("HRV time-domain shift"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("sdnn") && r.contains("rmssd") && r.contains("pnn50"));
}

#[test]
fn t12_j_hrv_lf_hf_band_is_parameterizationof_spectral_hrv() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID
        })
        .expect("HRV LF / HF band must have ParameterizationOf record");
    assert!(rec.reason.contains("Spectral HRV band shift"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("lf") && r.contains("hf"));
}

#[test]
fn t12_j_lead_specific_st_is_parameterizationof_st_segment() {
    let p = seed_t12_j_biosignal_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID
        })
        .expect("Lead-specific ST must have ParameterizationOf record");
    assert!(rec.reason.contains("ST-segment deviation proxy"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("anterior") || r.contains("inferior") || r.contains("lateral"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_j_fft_band_energy_domain_transfer_to_biosignal_exists() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == FFT_BAND_ENERGY_SEED_ID
        })
        .expect("FFT band-energy must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared spectral ancestor"));
    assert!(r.contains("medicalbiosignal"));
}

#[test]
fn t12_j_residual_envelope_domain_transfer_to_biosignal_exists() {
    let p = seed_t12_j_biosignal_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == RESIDUAL_ENVELOPE_EXIT_SEED_ID
        })
        .expect("Residual envelope exit must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    // Rust string-literal `\\\n` continuation joins with the
    // trailing space, so "shared envelope-\n     boundary
    // ancestor" becomes "shared envelope- boundary ancestor".
    // Match both halves.
    assert!(r.contains("shared envelope-") && r.contains("boundary ancestor"));
    assert!(r.contains("medicalbiosignal"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_j_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_j_biosignal_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "ExistingCanonicalAuthorityResolution target {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_j_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_j_biosignal_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "DomainTransferOf target {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_j_authority_resolution_covers_all_seed_biosignal_ids() {
    // SEED biosignal IDs are 49, 50, 51, 52. T.12.j MUST
    // authority-resolve every one of them.
    let p = seed_t12_j_biosignal_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12J_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.j"
        );
    }
}

#[test]
fn t12_j_every_dedup_record_has_reason() {
    let p = seed_t12_j_biosignal_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_j_canonical_addition_ids_are_in_6001_to_6008_range() {
    let p = seed_t12_j_biosignal_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6001..=6008).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.j reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_j_parameterization_ids_are_in_6009_to_6012_range() {
    let p = seed_t12_j_biosignal_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6009..=6012).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.j reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_j_rejection_ids_are_in_6013_to_6014_range() {
    let p = seed_t12_j_biosignal_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6013..=6014).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.j reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn bio_proposal_text_rendering_byte_stable() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn bio_proposal_json_rendering_byte_stable() {
    let p = seed_t12_j_biosignal_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
