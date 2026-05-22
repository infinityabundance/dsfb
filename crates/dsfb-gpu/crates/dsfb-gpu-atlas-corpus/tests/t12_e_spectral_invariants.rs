//! T.12.e acceptance suite — Signal Processing / Spectral /
//! Wavelet expansion proposal invariants.
//!
//! Nine panel-required load-bearing negatives pin the
//! transform-law discipline T.12.e exists to prove ("in spectral
//! detectors, the transform law is the detector"). Plus
//! per-SEED-record collision tests for every existing
//! spectral / signal canonical and per-CanonicalAddition
//! transform-law-declaration tests.

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
use dsfb_gpu_atlas_corpus::t12_e_spectral::{
    seed_t12_e_spectral_proposal, AUTOCORRELATION_BREAK_SEED_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_DOMAIN_TRANSFER_OF, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_PARAMETERIZATION_OF, CATEGORY_REJECTED_NOT_DETERMINISTIC,
    CEPSTRAL_RESERVED_CANONICAL_ID, FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID,
    FFT_BAND_ENERGY_SEED_ID, HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID,
    MATCHED_FILTER_RESERVED_CANONICAL_ID, RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID,
    RESIDUAL_ENVELOPE_EXIT_SEED_ID, SPECTRAL_CENTROID_RESERVED_CANONICAL_ID,
    SPECTRAL_ENTROPY_SEED_ID, STFT_RIDGE_RESERVED_CANONICAL_ID,
    STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID, WAVELET_COEFFICIENT_ENERGY_SEED_ID,
    WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID, WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

/// Every existing SEED canonical id T.12.e recognises via an
/// `ExistingCanonicalAuthorityResolution` record.
const T12E_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (FFT_BAND_ENERGY_SEED_ID, "FFT band-energy anomaly"),
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
    (SPECTRAL_ENTROPY_SEED_ID, "Spectral entropy"),
    (
        WAVELET_COEFFICIENT_ENERGY_SEED_ID,
        "Wavelet coefficient energy",
    ),
    (
        AUTOCORRELATION_BREAK_SEED_ID,
        "Autocorrelation-coefficient break",
    ),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn spectral_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_e_spectral_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Spectral proposal failed verifier: {errors:?}"
    );
}

#[test]
fn spectral_proposal_has_open_status() {
    let p = seed_t12_e_spectral_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn spectral_proposal_targets_signal_processing() {
    let p = seed_t12_e_spectral_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::SignalProcessing
    ));
}

/// Load-bearing negative #1 (panel-required).
#[test]
fn t12_e_does_not_mutate_seed_len() {
    let _ = seed_t12_e_spectral_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn spectral_proposal_proposes_ten_primitives() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 10);
}

#[test]
fn spectral_proposal_proposes_zero_aliases() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn spectral_proposal_proposes_sixteen_dedup_records() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 16);
}

#[test]
fn spectral_proposal_proposes_nine_genealogy_edges() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 9);
}

#[test]
fn spectral_proposal_proposes_seven_source_refs() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 7);
}

#[test]
fn spectral_delta_has_six_new_canonical_records() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 6);
}

#[test]
fn spectral_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_e_spectral_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert!(categories.contains(CATEGORY_CANONICAL_ADDITION));
    assert!(categories.contains(CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION));
    assert!(categories.contains(CATEGORY_DOMAIN_TRANSFER_OF));
    assert!(categories.contains(CATEGORY_PARAMETERIZATION_OF));
    assert!(categories.contains(CATEGORY_REJECTED_NOT_DETERMINISTIC));
    assert_eq!(categories.len(), 5);
}

#[test]
fn spectral_proposal_court_delta_category_counts() {
    let p = seed_t12_e_spectral_proposal();
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
            other => panic!("unexpected category wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 6);
    assert_eq!(existing, 5);
    assert_eq!(transfer, 1);
    assert_eq!(paramof, 3);
    assert_eq!(rejected, 1);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn spectral_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_e_spectral_proposal();
    let b = seed_t12_e_spectral_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_spectral_proposal_hash_matches_stored() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn spectral_proposal_hash_is_distinct_from_t12_0_a_b_c_d() {
    let spectral = seed_t12_e_spectral_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    assert_ne!(
        spectral.corpus_amendment_proposal_hash_v1,
        pol.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        spectral.corpus_amendment_proposal_hash_v1,
        spc.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        spectral.corpus_amendment_proposal_hash_v1,
        scd.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        spectral.corpus_amendment_proposal_hash_v1,
        drift.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        spectral.corpus_amendment_proposal_hash_v1,
        robust.corpus_amendment_proposal_hash_v1
    );
}

/// Load-bearing negative #9 (panel-required):
/// `t12_e_hash_changes_when_transform_law_changes`. Mutating one
/// CanonicalAddition record's transform-law declaration changes
/// the proposal hash.
#[test]
fn t12_e_hash_changes_when_transform_law_changes() {
    let p_a = seed_t12_e_spectral_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == STFT_RIDGE_RESERVED_CANONICAL_ID
        })
        .expect("STFT ridge shift CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED transform-law declaration for hash-sensitivity test",
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
// SEED collision load-bearing negative + per-id tests
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
    test_label: &'static str,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_e_collision_batch",
        SourceClass::SignalProcessing,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: test_label,
            motivation: "Should be rejected — id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_e_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_e_collision_proposal",
        "Defective T.12.e-style proposal duplicating an existing SEED canonical.",
        SourceClass::SignalProcessing,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_e_test",
    )
}

/// Load-bearing negative (panel-required):
/// `t12_e_existing_seed_collision_requires_authority_resolution`.
/// Parametric loop over all 5 SEED ids T.12.e recognises.
#[test]
fn t12_e_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12E_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id, label);
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

#[test]
fn t12_e_rejects_duplicate_fft_band_energy() {
    let p = build_defective_collision_proposal(FFT_BAND_ENERGY_SEED_ID, "FFT band-energy");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == FFT_BAND_ENERGY_SEED_ID
    )));
}

#[test]
fn t12_e_rejects_duplicate_spectral_entropy() {
    let p = build_defective_collision_proposal(SPECTRAL_ENTROPY_SEED_ID, "Spectral entropy");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == SPECTRAL_ENTROPY_SEED_ID
    )));
}

#[test]
fn t12_e_rejects_duplicate_wavelet_coefficient_energy() {
    let p = build_defective_collision_proposal(
        WAVELET_COEFFICIENT_ENERGY_SEED_ID,
        "Wavelet coefficient energy",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == WAVELET_COEFFICIENT_ENERGY_SEED_ID
    )));
}

// ---------------------------------------------------------------
// Transform-law contract load-bearing negatives (panel-required)
// ---------------------------------------------------------------

/// Load-bearing negative #2 (panel-required, MOST IMPORTANT for
/// spectral): `t12_e_rejects_fft_variant_without_window_and_normalization_law`.
#[test]
fn t12_e_rejects_fft_variant_without_window_and_normalization_law() {
    let p = seed_t12_e_spectral_proposal();
    let fft_seed = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == FFT_BAND_ENERGY_SEED_ID
        })
        .expect("FFT band-energy must have ExistingCanonicalAuthorityResolution record");
    let r = fft_seed.reason.to_lowercase();
    assert!(
        r.contains("window function") || r.contains("window"),
        "FFT band-energy record must declare window function: {}",
        fft_seed.reason
    );
    assert!(
        r.contains("normalization") || r.contains("fft normalization"),
        "FFT band-energy record must declare normalization: {}",
        fft_seed.reason
    );
    let fft_param = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID
        })
        .expect("FFT bandpower variant must have ParameterizationOf record");
    let r2 = fft_param.reason.to_lowercase();
    assert!(
        r2.contains("window function") || r2.contains("window"),
        "FFT bandpower variant must declare window function: {}",
        fft_param.reason
    );
    assert!(
        r2.contains("normalization"),
        "FFT bandpower variant must declare normalization: {}",
        fft_param.reason
    );
}

/// Load-bearing negative #3 (panel-required):
/// `t12_e_rejects_spectral_entropy_without_bin_or_power_normalization_law`.
#[test]
fn t12_e_rejects_spectral_entropy_without_bin_or_power_normalization_law() {
    let p = seed_t12_e_spectral_proposal();
    let entropy = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == SPECTRAL_ENTROPY_SEED_ID
        })
        .expect("Spectral entropy must have ExistingCanonicalAuthorityResolution record");
    let r = entropy.reason.to_lowercase();
    assert!(
        r.contains("bin definition") || r.contains("bin"),
        "Spectral entropy record must declare bin definition: {}",
        entropy.reason
    );
    assert!(
        r.contains("power-mass normalization")
            || r.contains("power mass")
            || r.contains("normalization"),
        "Spectral entropy record must declare power-mass normalization: {}",
        entropy.reason
    );
    assert!(
        r.contains("log base") || r.contains("log"),
        "Spectral entropy record must declare log base: {}",
        entropy.reason
    );
}

/// Load-bearing negative #4 (panel-required):
/// `t12_e_rejects_wavelet_detector_without_wavelet_family_and_boundary_law`.
#[test]
fn t12_e_rejects_wavelet_detector_without_wavelet_family_and_boundary_law() {
    let p = seed_t12_e_spectral_proposal();
    let wavelet_seed = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == WAVELET_COEFFICIENT_ENERGY_SEED_ID
        })
        .expect("Wavelet coefficient energy must have ExistingCanonicalAuthorityResolution record");
    let r = wavelet_seed.reason.to_lowercase();
    assert!(
        r.contains("wavelet family"),
        "Wavelet coefficient energy must declare wavelet family: {}",
        wavelet_seed.reason
    );
    assert!(
        r.contains("boundary handling") || r.contains("boundary"),
        "Wavelet coefficient energy must declare boundary handling: {}",
        wavelet_seed.reason
    );
    let wavelet_packet = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID
        })
        .expect("Wavelet packet energy must have CanonicalAddition record");
    let r2 = wavelet_packet.reason.to_lowercase();
    assert!(
        r2.contains("wavelet family"),
        "Wavelet packet energy must declare wavelet family: {}",
        wavelet_packet.reason
    );
    assert!(
        r2.contains("boundary handling") || r2.contains("boundary"),
        "Wavelet packet energy must declare boundary handling: {}",
        wavelet_packet.reason
    );
    assert!(
        r2.contains("packet-tree depth") || r2.contains("packet tree depth"),
        "Wavelet packet energy must declare packet-tree depth: {}",
        wavelet_packet.reason
    );
}

/// Load-bearing negative #5 (panel-required):
/// `t12_e_rejects_stft_ridge_without_window_hop_and_ridge_selection_law`.
#[test]
fn t12_e_rejects_stft_ridge_without_window_hop_and_ridge_selection_law() {
    let p = seed_t12_e_spectral_proposal();
    let stft = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == STFT_RIDGE_RESERVED_CANONICAL_ID
        })
        .expect("STFT ridge shift must have CanonicalAddition record");
    let r = stft.reason.to_lowercase();
    assert!(
        r.contains("window function") || r.contains("window"),
        "STFT ridge must declare window function: {}",
        stft.reason
    );
    assert!(
        r.contains("window length"),
        "STFT ridge must declare window length: {}",
        stft.reason
    );
    assert!(
        r.contains("hop") || r.contains("overlap"),
        "STFT ridge must declare hop / overlap law: {}",
        stft.reason
    );
    assert!(
        r.contains("ridge selection") || r.contains("ridge"),
        "STFT ridge must declare ridge selection law: {}",
        stft.reason
    );
}

/// Load-bearing negative #6 (panel-required):
/// `t12_e_rejects_matched_filter_without_template_provenance`.
#[test]
fn t12_e_rejects_matched_filter_without_template_provenance() {
    let p = seed_t12_e_spectral_proposal();
    let matched = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == MATCHED_FILTER_RESERVED_CANONICAL_ID
        })
        .expect("Matched filter must have CanonicalAddition record");
    let r = matched.reason.to_lowercase();
    assert!(
        r.contains("template provenance"),
        "Matched filter must declare template provenance: {}",
        matched.reason
    );
    assert!(
        r.contains("sampling-rate match") || r.contains("sampling rate"),
        "Matched filter must declare sampling-rate match: {}",
        matched.reason
    );
    assert!(
        r.contains("normalization"),
        "Matched filter must declare normalization: {}",
        matched.reason
    );
}

/// Load-bearing negative #7 (panel-required):
/// `t12_e_rejects_hilbert_amplitude_without_sampling_law`.
#[test]
fn t12_e_rejects_hilbert_amplitude_without_sampling_law() {
    let p = seed_t12_e_spectral_proposal();
    let hilbert = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID
        })
        .expect("Hilbert amplitude must have CanonicalAddition record");
    let r = hilbert.reason.to_lowercase();
    assert!(
        r.contains("sampling law"),
        "Hilbert amplitude must declare sampling law: {}",
        hilbert.reason
    );
    assert!(
        r.contains("analytic-signal extraction") || r.contains("analytic signal"),
        "Hilbert amplitude must declare analytic-signal extraction method: {}",
        hilbert.reason
    );
}

/// Load-bearing negative #8 (panel-required):
/// `t12_e_rejects_randomized_spectral_projection_without_deterministic_reduction`.
#[test]
fn t12_e_rejects_randomized_spectral_projection_without_deterministic_reduction() {
    let p = seed_t12_e_spectral_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Randomized spectral projection (id {RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let rejection = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID
        })
        .expect("Randomized spectral projection must have RejectedNotDeterministic record");
    let r = rejection.reason.to_lowercase();
    assert!(
        r.contains("seed"),
        "Randomized spectral projection rejection must declare seed requirement: {}",
        rejection.reason
    );
    assert!(
        r.contains("projection matrix"),
        "Randomized spectral projection rejection must declare projection matrix requirement: {}",
        rejection.reason
    );
    assert!(
        r.contains("dimension"),
        "Randomized spectral projection rejection must declare dimension requirement: {}",
        rejection.reason
    );
    assert!(
        r.contains("numeric mode"),
        "Randomized spectral projection rejection must declare numeric mode requirement: {}",
        rejection.reason
    );
}

// ---------------------------------------------------------------
// Per-CanonicalAddition transform-law declarations
// ---------------------------------------------------------------

#[test]
fn t12_e_spectral_centroid_declares_first_moment_law() {
    let p = seed_t12_e_spectral_proposal();
    let centroid = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SPECTRAL_CENTROID_RESERVED_CANONICAL_ID
        })
        .expect("Spectral centroid must have CanonicalAddition record");
    let r = centroid.reason.to_lowercase();
    assert!(r.contains("first-moment") || r.contains("first moment"));
    assert!(r.contains("power-spectrum") || r.contains("power spectrum"));
    assert!(r.contains("sampling law"));
}

#[test]
fn t12_e_cepstral_declares_fft_convention_and_log_base() {
    let p = seed_t12_e_spectral_proposal();
    let cepstral = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CEPSTRAL_RESERVED_CANONICAL_ID
        })
        .expect("Cepstral anomaly must have CanonicalAddition record");
    let r = cepstral.reason.to_lowercase();
    assert!(r.contains("fft convention") || r.contains("fft"));
    assert!(r.contains("log base"));
    assert!(r.contains("real-cepstrum") || r.contains("complex-cepstrum"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_e_fft_bandpower_variant_is_parameterizationof_fft_band_energy() {
    let p = seed_t12_e_spectral_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let param = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID
        })
        .expect("FFT bandpower variant must have ParameterizationOf record");
    assert!(
        param.reason.contains("FFT band-energy"),
        "FFT bandpower variant must reference parent: {}",
        param.reason
    );
}

#[test]
fn t12_e_wavelet_family_variant_is_parameterizationof_wavelet_coefficient_energy() {
    let p = seed_t12_e_spectral_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let param = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID
        })
        .expect("Wavelet family variant must have ParameterizationOf record");
    assert!(
        param.reason.contains("wavelet coefficient energy"),
        "Wavelet family variant must reference parent: {}",
        param.reason
    );
}

#[test]
fn t12_e_stft_window_hop_variant_is_parameterizationof_stft_ridge_shift() {
    let p = seed_t12_e_spectral_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let param = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID
        })
        .expect("STFT window/hop variant must have ParameterizationOf record");
    assert!(
        param.reason.contains("STFT ridge shift"),
        "STFT window/hop variant must reference parent: {}",
        param.reason
    );
}

#[test]
fn t12_e_randomized_spectral_projection_present_in_primitives_but_not_in_canonical() {
    let p = seed_t12_e_spectral_proposal();
    let in_primitives = p.body.proposed_primitives.iter().any(|pr| {
        pr.reserved_canonical_id.0 == RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID
    });
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID);
    assert!(in_primitives);
    assert!(!in_canonical);
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_e_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_e_spectral_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_e_domain_transfer_target_must_be_in_seed() {
    let p = seed_t12_e_spectral_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_e_every_dedup_record_has_reason() {
    let p = seed_t12_e_spectral_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_e_canonical_addition_ids_are_in_5501_to_5506_range() {
    let p = seed_t12_e_spectral_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5501..=5506).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.e reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn spectral_proposal_text_rendering_byte_stable() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn spectral_proposal_json_rendering_byte_stable() {
    let p = seed_t12_e_spectral_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
