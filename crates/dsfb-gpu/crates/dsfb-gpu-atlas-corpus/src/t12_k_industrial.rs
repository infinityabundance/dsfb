//! T.12.k — Industrial / Fault Detection and Diagnostics /
//! Condition Monitoring: the eleventh real literature expansion
//! proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.k files the Industrial / FDD / Condition Monitoring
//! > amendment proposal. It admits only deterministic condition-
//! > monitoring / FDD witnesses whose plant or sensor model,
//! > physical quantity, unit law, sampling law, operating
//! > regime, baseline / nominal envelope, residual definition,
//! > fault-signature decision law, and confuser / nuisance-
//! > process profile are declared; resolves SEED collisions;
//! > classifies variants as parameterizations or domain
//! > transfers; rejects proprietary PdM black-box scores and
//! > learned fault classifiers without training-artifact
//! > anchors; and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"An industrial fault
//! witness is not a diagnosis of machine cause unless the plant
//! model, residual law, sensor law, operating regime, and
//! confuser profile are declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.k admits deterministic condition-monitoring / FDD
//! > witnesses, not root-cause certainty and not maintenance
//! > recommendations.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.k's design began with a grep of [`crate::seed::SEED`]
//! for every industrial / FDD candidate. The walk found
//! **eight** T.12.k-relevant primitives already canonical —
//! the largest SEED-collision set of any T.12.x to date:
//!
//! * **FFT band-energy anomaly** at SEED id 12 (shared spectral
//!   ancestor for industrial vibration / motor-current /
//!   acoustic detectors).
//! * **PCA T² on score vector** at SEED id 19.
//! * **PCA SPE / Q residual** at SEED id 20.
//! * **PLS residual / Q on PLS** at SEED id 21.
//! * **Residual envelope exit** at SEED id 22 (shared envelope-
//!   boundary ancestor for industrial envelope detectors).
//! * **Sensor bias detector** at SEED id 23.
//! * **Actuator stiction detector** at SEED id 24.
//! * **Valve hunting (control-loop oscillation) detector** at
//!   SEED id 25.
//!
//! All eight become `ExistingCanonicalAuthorityResolution`
//! records under the `FaultDetectionDiagnostics` source class,
//! each with its specific sensor / quantity / unit / sampling /
//! baseline / residual / decision-law contract declared. **This
//! is the core T.12.k discipline: ratify the existing FDD
//! surface rather than inflate detector count.**
//!
//! Six genuinely new canonicals at reserved ids 6101..=6106
//! survived the SEED-walk as structurally distinct decision
//! functionals:
//!
//! * **Kalman innovation whiteness witness** (6101) — Mehra &
//!   Peschon 1971 whiteness-of-innovations test on Kalman
//!   filter residuals. Decision functional: autocorrelation-of-
//!   innovations equals zero (whiteness test). Structurally
//!   distinct from T.12.f 5609 innovation-sequence anomaly
//!   which uses innovation-magnitude threshold.
//! * **Operating-regime transition witness** (6102) — process
//!   state-machine baseline switch (start-up / steady-state /
//!   shutdown / fault-recovery regimes). No existing SEED or
//!   T.12.f ancestor; structurally novel.
//! * **Condition-indicator drift** (6103) — derived condition-
//!   indicator rate-of-change witness. Structurally distinct
//!   from Sensor bias (SEED 23) because CI is a derived /
//!   computed quantity, not a raw sensor signal.
//! * **Fault signature angle** (6104) — angular direction of
//!   the fault vector in PCA score space (or in a declared
//!   latent space). Structurally distinct from PCA T² (SEED 19,
//!   magnitude in score space) and SPE (SEED 20, magnitude in
//!   residual space) — angle vs magnitude is a different
//!   decision functional.
//! * **Contribution-plot spike** (6105) — per-variable
//!   contribution to T² or SPE crossing a threshold. Structurally
//!   distinct decision functional — operates on per-variable
//!   contribution series rather than aggregate T² / SPE scalar.
//! * **Spectral kurtosis** (6106) — Antoni 2006 kurtosis-in-
//!   frequency-bands detector for transient mechanical faults.
//!   Structurally distinct from SEED 12 FFT band-energy because
//!   the decision functional is fourth-moment shape (kurtosis),
//!   not energy magnitude.
//!
//! Two domain transfers (panel-locked):
//!
//! * **FFT band-energy anomaly** (SEED 12) → `DomainTransferOf`
//!   for `FaultDetectionDiagnostics` as the shared spectral
//!   ancestor (bearing vibration 6107 and motor current
//!   signature 6108 are the FDD descendants).
//! * **Residual envelope exit** (SEED 22) → `DomainTransferOf`
//!   for `FaultDetectionDiagnostics` as the shared envelope-
//!   boundary ancestor (temperature envelope excursion 6109 is
//!   the FDD descendant).
//!
//! Four parameterizations (panel-candidate canonicals that
//! collapsed on closer inspection — the strength of T.12.k is
//! refusing to admit these as new):
//!
//! * **Bearing vibration band-energy** (6107) →
//!   `ParameterizationOf(FFT band-energy, SEED 12)` with
//!   bearing-defect-frequency parameterization (BPFO / BPFI /
//!   BSF / FTF defect frequencies per McFadden & Smith 1984).
//! * **Motor current signature anomaly (MCSA)** (6108) →
//!   `ParameterizationOf(FFT band-energy, SEED 12)` with motor-
//!   current parameterization (broken-rotor-bar sidebands,
//!   eccentricity harmonics per Thomson 2001).
//! * **Temperature envelope excursion** (6109) →
//!   `ParameterizationOf(Residual envelope exit, SEED 22)` with
//!   thermal-time-constant + per-channel-temperature
//!   parameterization.
//! * **Pressure transient witness** (6110) →
//!   `ParameterizationOf(Slew shock, SEED 42)` with pressure-
//!   physics + derivative-of-pressure parameterization.
//!
//! Two rejections (fifth T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, T.12.h,
//! T.12.i, T.12.j):
//!
//! * **Proprietary PdM black-box score** (6111) —
//!   `RejectedNotDeterministic`. Vendor predictive-maintenance
//!   scores (GE Predix, Siemens MindSphere, IBM Maximo Predict,
//!   Honeywell Forge, Aspen Mtell) expose numeric anomaly /
//!   "asset health" scores without a stable public decision
//!   functional. Admission requires a future T.12.x to admit a
//!   `Deterministic_PdM_Score_Proxy` canonical with deterministic
//!   formula + model-identification anchor + training-data
//!   anchor + feature schema + tie-break + numeric mode all
//!   brutally explicit. The court does NOT issue maintenance
//!   recommendations or remaining-useful-life predictions.
//! * **Learned fault classifier without fixed training
//!   artifact** (6112) — `RejectedNotDeterministic`. Deep-
//!   learning fault classifiers (Wen et al.\ 2017 CNN-based
//!   bearing classifier; Khan & Yairi 2018 deep-learning fault
//!   classifier review; commercial equivalents) classify by
//!   learned representations on labelled fault corpora.
//!   Admission requires model-identification seed + training-
//!   data anchor (pinned dataset record-hash, e.g.\ CWRU
//!   bearing dataset) + label schema + tie-break + numeric mode
//!   declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×6.
//! * `ExistingCanonicalAuthorityResolution` ×8 — the largest
//!   SEED-collision ratification of any T.12.x to date.
//! * `DomainTransferOf` ×2 — FFT band-energy (SEED 12) and
//!   Residual envelope exit (SEED 22).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 6 + 8 + 2 + 4 + 2 = **22 dedup-court records**.
//!
//! ## Root-cause-claim discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND ExistingCanonicalAuthorityResolution
//! reason text MUST describe its record as a "condition-
//! monitoring witness" / "FDD witness" / "fault-signature
//! witness" / "envelope-exit witness" / "spectral witness" —
//! NEVER as a root-cause claim, maintenance recommendation,
//! remaining-useful-life prediction, or failure-mode
//! classification. The dedicated load-bearing negative
//! `t12_k_rejects_root_cause_claim_language` scans every such
//! reason for forbidden terms (`root cause`, `diagnosis of
//! machine cause`, `maintenance recommendation`, `remaining
//! useful life`, `predicted RUL`, `failure mode classification`)
//! and asserts every qualifying reason ends with the panel-
//! locked non-claim "condition-monitoring witness, not a
//! maintenance recommendation". Forbidden terms appear ONLY in
//! `RejectedNotDeterministic` reason text (where they describe
//! what is NOT admitted).
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.k industrial
//!   `corpus_amendment_proposal_hash_v1` distinct from every
//!   prior T.12.x proposal hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior T.12.x;
//! every `pub` item AND every private helper carries a doc
//! comment whose first sentence states the WHY for a future
//! engineer; 10-step ritual; no `--no-verify`.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    CorpusAmendmentProposal, ProposalStatus, ProposedAliasClaim, ProposedDedupRecord,
    ProposedGenealogyEdge, ProposedPrimitive, ProposedSourceRef, ProposerRole, RejectionRecord,
    SourceClass,
};
use crate::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Reserved id constants (panel-locked, 6101..=6112 used; 6113..=6199 reserved)
// ---------------------------------------------------------------

/// Reserved canonical id for Kalman innovation whiteness witness.
/// Distinct from T.12.f 5609 innovation-sequence anomaly: this
/// is a whiteness test, not a magnitude test.
pub const KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID: u32 = 6101;

/// Reserved canonical id for Operating-regime transition witness.
pub const OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID: u32 = 6102;

/// Reserved canonical id for Condition-indicator drift. Distinct
/// from SEED 23 Sensor bias because CI is a derived / computed
/// quantity, not a raw sensor signal.
pub const CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID: u32 = 6103;

/// Reserved canonical id for Fault signature angle. Angular
/// direction of fault vector in PCA score space; distinct from
/// SEED 19 PCA T² magnitude.
pub const FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID: u32 = 6104;

/// Reserved canonical id for Contribution-plot spike. Per-
/// variable contribution to T² or SPE; distinct from aggregate
/// SEED 19 / SEED 20 scalars.
pub const CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID: u32 = 6105;

/// Reserved canonical id for Spectral kurtosis (Antoni 2006).
/// Fourth-moment shape, not energy magnitude; distinct from
/// SEED 12 FFT band-energy.
pub const SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID: u32 = 6106;

/// Reserved id for Bearing vibration band-energy.
/// `ParameterizationOf(FFT band-energy, SEED 12)`.
pub const BEARING_VIBRATION_RESERVED_PRIMITIVE_ID: u32 = 6107;

/// Reserved id for Motor current signature anomaly (MCSA).
/// `ParameterizationOf(FFT band-energy, SEED 12)`.
pub const MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID: u32 = 6108;

/// Reserved id for Temperature envelope excursion.
/// `ParameterizationOf(Residual envelope exit, SEED 22)`.
pub const TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID: u32 = 6109;

/// Reserved id for Pressure transient witness.
/// `ParameterizationOf(Slew shock, SEED 42)`.
pub const PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID: u32 = 6110;

/// Reserved id for Proprietary PdM black-box score.
/// `RejectedNotDeterministic`.
pub const PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID: u32 = 6111;

/// Reserved id for Learned fault classifier.
/// `RejectedNotDeterministic`.
pub const LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 6112;

// Existing SEED canonical ids referenced by T.12.k.

/// FFT band-energy anomaly — SEED canonical id 12. Shared
/// spectral ancestor for industrial vibration / motor-current /
/// acoustic detectors.
pub const FFT_BAND_ENERGY_SEED_ID: u32 = 12;

/// PCA T-squared on score vector — SEED canonical id 19.
pub const PCA_T2_SEED_ID: u32 = 19;

/// PCA SPE / Q residual — SEED canonical id 20.
pub const PCA_SPE_Q_SEED_ID: u32 = 20;

/// PLS residual / Q on PLS — SEED canonical id 21.
pub const PLS_RESIDUAL_SEED_ID: u32 = 21;

/// Residual envelope exit — SEED canonical id 22. Shared
/// envelope-boundary ancestor for industrial envelope detectors.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// Sensor bias detector — SEED canonical id 23.
pub const SENSOR_BIAS_SEED_ID: u32 = 23;

/// Actuator stiction detector — SEED canonical id 24.
pub const ACTUATOR_STICTION_SEED_ID: u32 = 24;

/// Valve hunting (control-loop oscillation) detector — SEED
/// canonical id 25.
pub const VALVE_HUNTING_SEED_ID: u32 = 25;

/// Slew shock — SEED canonical id 42 (ancestor for pressure
/// transient witness parameterization).
pub const SLEW_SHOCK_SEED_ID: u32 = 42;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------

/// `CanonicalAddition`.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution`.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf`.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf`.
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

/// `RejectedNotDeterministic`.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

// ---------------------------------------------------------------
// Builders for the industrial expansion batch
// ---------------------------------------------------------------

/// Build the industrial `CorpusExpansionBatch` body.
fn build_industrial_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_k_industrial_first_proposal",
        SourceClass::FaultDetectionDiagnostics,
        industrial_proposed_primitives(),
        industrial_proposed_aliases(),
        industrial_proposed_dedup_records(),
        industrial_proposed_genealogy_edges(),
        industrial_proposed_source_refs(),
    )
}

/// Twelve proposed primitives: 6 canonical + 4 parameterization
/// shells + 2 rejection shells. The "few new canonicals, many
/// authority resolutions" shape is the panel-locked success
/// posture for T.12.k.
fn industrial_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID,
            ),
            display_name: "Kalman innovation whiteness witness",
            motivation: "Mehra & Peschon 1971 whiteness-of-innovations test on \
                 Kalman filter residuals. Required FDD contract: sensor identity, \
                 physical quantity, unit, sampling law, operating regime, plant / \
                 observer model (Kalman state-space declared with Q / R covariances \
                 + filter gain), nominal innovation-sequence baseline, whiteness \
                 decision law (autocorrelation-of-innovations equals zero test - \
                 NOT magnitude threshold), confuser / nuisance-process profile \
                 (regime-transition residual transient). Condition-monitoring \
                 witness, not a maintenance recommendation. Structurally distinct \
                 from T.12.f 5609 innovation-sequence anomaly which uses \
                 magnitude.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID,
            ),
            display_name: "Operating-regime transition witness",
            motivation: "Process state-machine baseline-switch witness. Required \
                 FDD contract: sensor identity, physical quantity, unit, sampling \
                 law, declared operating-regime set (start-up / steady-state / \
                 shutdown / fault-recovery / cleaning regimes), regime-membership \
                 decision law (state-machine transition predicate), baseline-switch \
                 law on regime change, confuser / nuisance-process profile \
                 (legitimate regime change vs. fault-induced transition). \
                 Condition-monitoring witness, not a maintenance recommendation. \
                 No existing SEED or T.12.f ancestor.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID,
            ),
            display_name: "Condition-indicator drift",
            motivation: "Derived condition-indicator rate-of-change witness. \
                 Required FDD contract: condition-indicator computation law (CI = \
                 declared function of raw sensors), physical quantity, unit, \
                 sampling law, operating regime, nominal CI envelope, rate-of- \
                 change decision law, confuser / nuisance-process profile \
                 (regime-induced CI shift vs. drift). Distinct from SEED 23 Sensor \
                 bias because CI is a derived / computed quantity, not a raw \
                 sensor signal. Condition-monitoring witness, not a maintenance \
                 recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID),
            display_name: "Fault signature angle",
            motivation: "Angular direction of fault vector in PCA score space (or \
                 in a declared latent space). Required FDD contract: latent-space \
                 model (PCA / PLS / declared subspace), physical quantity, unit, \
                 sampling law, operating regime, nominal score-space angular \
                 distribution, angular decision law (cosine-distance from nominal \
                 direction crosses threshold - NOT magnitude), confuser / nuisance- \
                 process profile (regime change moves the nominal direction). \
                 Structurally distinct from SEED 19 PCA T² (magnitude in score \
                 space) and SEED 20 SPE (magnitude in residual space). Condition- \
                 monitoring witness, not a maintenance recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID,
            ),
            display_name: "Contribution-plot spike",
            motivation: "Per-variable contribution to T² or SPE crossing a \
                 declared threshold. Required FDD contract: latent-space model \
                 (PCA / PLS), physical quantity per variable, unit per variable, \
                 sampling law, operating regime, nominal per-variable contribution \
                 envelope, decision law (per-variable contribution crosses \
                 threshold - operates on contribution SERIES, not aggregate \
                 T² / SPE scalar), confuser / nuisance-process profile. \
                 Structurally distinct decision functional from SEED 19 / SEED 20 \
                 aggregate scalars. Condition-monitoring witness, not a \
                 maintenance recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID),
            display_name: "Spectral kurtosis",
            motivation: "Antoni 2006 spectral kurtosis transient-mechanical-fault \
                 witness. Required FDD contract: sensor identity (vibration \
                 accelerometer), physical quantity, unit, sampling law (with \
                 Nyquist), STFT window + hop, spectral-kurtosis estimator \
                 (kurtogram band selection or fixed band declared), operating \
                 regime, nominal spectral-kurtosis envelope, decision law \
                 (kurtosis-in-band crosses threshold - fourth-moment shape, NOT \
                 energy magnitude), confuser / nuisance-process profile (regime- \
                 induced transients). Structurally distinct from SEED 12 FFT \
                 band-energy. Condition-monitoring witness, not a maintenance \
                 recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BEARING_VIBRATION_RESERVED_PRIMITIVE_ID),
            display_name: "Bearing vibration band-energy - parameterization shell",
            motivation: "Bearing-defect-frequency parameterization of FFT band- \
                 energy (SEED id 12) with declared bearing-defect-frequency set \
                 (BPFO / BPFI / BSF / FTF per McFadden & Smith 1984). The court \
                 rules: bearing vibration band-energy is ParameterizationOf(FFT \
                 band-energy, SEED 12) with bearing-domain parameterization, NOT \
                 a new canonical primitive. Appears in proposed_primitives but \
                 NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Motor current signature anomaly - parameterization shell",
            motivation: "Motor-current parameterization of FFT band-energy (SEED \
                 id 12) per Thomson 2001 MCSA with declared spectral bands (broken- \
                 rotor-bar sidebands at (1 +/- 2s)f_s; eccentricity harmonics). \
                 The court rules: motor current signature anomaly is \
                 ParameterizationOf(FFT band-energy, SEED 12) with motor-current- \
                 domain parameterization, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID),
            display_name: "Temperature envelope excursion - parameterization shell",
            motivation: "Thermal-time-constant parameterization of Residual \
                 envelope exit (SEED id 22) with declared per-channel temperature \
                 envelope + thermal-time-constant + cooling / heating rate law. \
                 The court rules: temperature envelope excursion is \
                 ParameterizationOf(Residual envelope exit, SEED 22) with thermal- \
                 physics parameterization, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID),
            display_name: "Pressure transient witness - parameterization shell",
            motivation: "Pressure-physics parameterization of Slew shock (SEED id \
                 42) with declared derivative-of-pressure decision law + transient \
                 magnitude + transient-duration thresholds + pressure-sensor \
                 dynamic-range bounds. The court rules: pressure transient witness \
                 is ParameterizationOf(Slew shock, SEED 42) with pressure-physics \
                 parameterization, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID),
            display_name: "Proprietary PdM black-box score - rejected shell",
            motivation: "Vendor predictive-maintenance scores (GE Predix, Siemens \
                 MindSphere, IBM Maximo Predict, Honeywell Forge, Aspen Mtell) \
                 expose numeric anomaly / asset-health scores without a stable \
                 public decision functional, declared training-data anchor, or \
                 model-identification anchor. The court does NOT admit proprietary \
                 PdM scores to the dedup-court delta's new_canonical_records. A \
                 future T.12.x may admit a Deterministic_PdM_Score_Proxy canonical \
                 only if a deterministic formula, model-identification anchor, \
                 training-data anchor, feature schema, tie-break, and numeric \
                 mode are all brutally explicit. The court does NOT issue \
                 maintenance recommendations or remaining useful life \
                 predictions.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Learned fault classifier - rejected shell",
            motivation: "Deep-learning fault classifiers (Wen et al.\\ 2017 CNN- \
                 based bearing fault classifier; Khan & Yairi 2018 deep-learning \
                 fault classifier review; commercial deep-learning equivalents) \
                 classify rotating-machinery / process faults by learned \
                 representations trained on labelled fault corpora. The court \
                 does NOT admit learned fault classifiers to the dedup-court \
                 delta's new_canonical_records. A future T.12.x may admit a \
                 Deterministic_Fault_Classifier_Proxy canonical only if the \
                 model-identification seed, training-data anchor (pinned dataset \
                 record-hash, e.g.\\ CWRU bearing dataset record-hash), label \
                 schema (pinned failure-mode taxonomy), tie-break law, and \
                 numeric mode are all brutally explicit.",
        },
    ]
}

/// Zero alias claims (T.12.k routes everything through dedup
/// records and existing-canonical authority resolutions).
fn industrial_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty-two dedup-court decisions on the industrial batch.
fn industrial_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 6 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID),
            reason: "Kalman innovation whiteness witness (Mehra & Peschon 1971): \
                 declared sensor identity + physical quantity + unit + sampling \
                 law + operating regime + plant / observer model (Kalman state- \
                 space with Q / R covariances + filter gain) + nominal innovation- \
                 sequence baseline + whiteness decision law (autocorrelation- \
                 of-innovations equals zero - NOT magnitude threshold) + confuser \
                 / nuisance-process profile. Distinct from T.12.f 5609 innovation- \
                 sequence anomaly. Condition-monitoring witness, not a \
                 maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID),
            reason: "Operating-regime transition witness: declared sensor \
                 identity + physical quantity + unit + sampling law + operating- \
                 regime set (start-up / steady-state / shutdown / fault-recovery \
                 / cleaning) + regime-membership decision law (state-machine \
                 transition predicate) + baseline-switch law on regime change + \
                 confuser / nuisance-process profile (legitimate regime change \
                 vs. fault-induced transition). Condition-monitoring witness, \
                 not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID),
            reason: "Condition-indicator drift: declared condition-indicator \
                 computation law (CI = declared function of raw sensors) + \
                 physical quantity + unit + sampling law + operating regime + \
                 nominal CI envelope + rate-of-change decision law + confuser / \
                 nuisance-process profile. Distinct from SEED 23 Sensor bias \
                 because CI is a derived / computed quantity. Condition- \
                 monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID),
            reason: "Fault signature angle: declared latent-space model (PCA / \
                 PLS / declared subspace) + physical quantity + unit + sampling \
                 law + operating regime + nominal score-space angular \
                 distribution + angular decision law (cosine-distance from nominal \
                 direction crosses threshold - NOT magnitude) + confuser / \
                 nuisance-process profile. Distinct from SEED 19 PCA T² magnitude \
                 and SEED 20 SPE magnitude. Condition-monitoring witness, not a \
                 maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID),
            reason: "Contribution-plot spike: declared latent-space model + \
                 physical quantity per variable + unit per variable + sampling \
                 law + operating regime + nominal per-variable contribution \
                 envelope + decision law (per-variable contribution crosses \
                 threshold - operates on contribution SERIES, not aggregate \
                 T² / SPE scalar) + confuser / nuisance-process profile. Distinct \
                 decision functional from SEED 19 / SEED 20 aggregate scalars. \
                 Condition-monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID),
            reason: "Spectral kurtosis (Antoni 2006): declared sensor identity \
                 (vibration accelerometer) + physical quantity + unit + sampling \
                 law (with Nyquist) + STFT window + hop + spectral-kurtosis \
                 estimator (kurtogram band selection or fixed band declared) + \
                 operating regime + nominal spectral-kurtosis envelope + decision \
                 law (kurtosis-in-band crosses threshold - fourth-moment shape, \
                 NOT energy magnitude) + confuser / nuisance-process profile. \
                 Distinct from SEED 12 FFT band-energy. Condition-monitoring \
                 witness, not a maintenance recommendation.",
        },
        // -- 8 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly stays canonical at SEED id 12. \
                 Declared sensor identity (vibration / acoustic / current sensor) \
                 + physical quantity + unit + sampling law (with Nyquist) + \
                 windowing law + per-band frequency definitions + power-mass \
                 normalization + baseline-band-energy envelope + decision law \
                 (per-band energy crosses threshold) + confuser / nuisance-process \
                 profile. No duplicate admitted; bearing vibration (6107) and \
                 motor current signature (6108) collapse here as \
                 ParameterizationOf. Condition-monitoring witness, not a \
                 maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            reason: "PCA T-squared on score vector stays canonical at SEED id 19. \
                 Declared sensor set + physical quantity per sensor + unit per \
                 sensor + sampling law + operating regime + PCA model (pinned \
                 score-space loadings + nominal score-space covariance) + T² \
                 decision law (Mahalanobis distance in score space crosses \
                 declared control limit) + confuser / nuisance-process profile \
                 (regime-induced score shift). No duplicate admitted. Condition- \
                 monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            reason: "PCA SPE / Q residual stays canonical at SEED id 20. \
                 Declared sensor set + PCA model (pinned loadings) + per-variable \
                 residual computation + SPE decision law (sum-of-squared-residuals \
                 crosses declared control limit) + confuser / nuisance-process \
                 profile (regime-induced residual shift). No duplicate admitted. \
                 Condition-monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PLS_RESIDUAL_SEED_ID),
            reason: "PLS residual / Q on PLS stays canonical at SEED id 21. \
                 Declared input + output sensor sets + PLS model (pinned X- and \
                 Y-loadings + inner-relation regression) + per-variable residual \
                 computation + Q decision law + confuser / nuisance-process \
                 profile. No duplicate admitted. Condition-monitoring witness, \
                 not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22. \
                 Declared sensor identity + physical quantity + unit + sampling \
                 law + operating regime + residual definition (observed minus \
                 model-predicted) + nominal envelope bounds + envelope-exit \
                 decision law (residual crosses envelope) + confuser / nuisance- \
                 process profile. No duplicate admitted; temperature envelope \
                 excursion (6109) collapses here as ParameterizationOf. \
                 Condition-monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SENSOR_BIAS_SEED_ID),
            reason: "Sensor bias detector stays canonical at SEED id 23. \
                 Declared sensor identity + physical quantity + unit + sampling \
                 law + operating regime + nominal sensor-mean baseline + bias \
                 decision law (raw sensor mean deviates from nominal over \
                 declared window) + confuser / nuisance-process profile \
                 (regime-induced mean shift). No duplicate admitted; this is \
                 RAW-sensor bias, distinct from 6103 condition-indicator drift. \
                 Condition-monitoring witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ACTUATOR_STICTION_SEED_ID),
            reason: "Actuator stiction detector stays canonical at SEED id 24. \
                 Declared actuator identity + command / output signal pair + \
                 physical quantity + unit + sampling law + operating regime + \
                 stiction-signature decision law (command-vs-output deadband / \
                 jump pattern crosses declared threshold) + confuser / nuisance- \
                 process profile. No duplicate admitted. Condition-monitoring \
                 witness, not a maintenance recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(VALVE_HUNTING_SEED_ID),
            reason: "Valve hunting (control-loop oscillation) detector stays \
                 canonical at SEED id 25. Declared control-loop identity + \
                 setpoint + manipulated-variable + controlled-variable + sampling \
                 law + operating regime + oscillation decision law (sustained \
                 oscillation magnitude + frequency-band cross declared envelope) \
                 + confuser / nuisance-process profile. No duplicate admitted. \
                 Condition-monitoring witness, not a maintenance recommendation.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly (SEED id 12) is the shared spectral \
                 ancestor for the FaultDetectionDiagnostics source class \
                 (bearing vibration band-energy 6107, motor current signature \
                 6108, and spectral kurtosis 6106 are FDD descendants - the \
                 first two as parameterizations, the third as a structurally- \
                 distinct canonical with a different decision functional). The \
                 court records the domain transfer without re-canonicalising \
                 FFT band-energy.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the shared envelope- \
                 boundary ancestor for the FaultDetectionDiagnostics source class \
                 (temperature envelope excursion 6109 is the FDD descendant as a \
                 thermal-physics parameterization). The court records the domain \
                 transfer without re-canonicalising Residual envelope exit.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(BEARING_VIBRATION_RESERVED_PRIMITIVE_ID),
            reason: "Bearing vibration band-energy is ParameterizationOf(FFT \
                 band-energy, SEED id 12). Bearing-defect-frequency \
                 parameterization (BPFO / BPFI / BSF / FTF per McFadden & Smith \
                 1984). The court declines to admit bearing vibration band- \
                 energy as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID),
            reason: "Motor current signature anomaly (MCSA) is ParameterizationOf \
                 (FFT band-energy, SEED id 12). Motor-current spectral-band \
                 parameterization (broken-rotor-bar sidebands at (1 +/- 2s)f_s; \
                 eccentricity harmonics per Thomson 2001). The court declines to \
                 admit MCSA as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID),
            reason: "Temperature envelope excursion is ParameterizationOf \
                 (Residual envelope exit, SEED id 22). Thermal-time-constant + \
                 per-channel-temperature parameterization with declared cooling \
                 / heating rate law. The court declines to admit temperature \
                 envelope excursion as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID),
            reason: "Pressure transient witness is ParameterizationOf(Slew shock, \
                 SEED id 42). Pressure-physics parameterization with declared \
                 derivative-of-pressure decision law + transient magnitude + \
                 transient-duration thresholds + pressure-sensor dynamic-range \
                 bounds. The court declines to admit pressure transient as a new \
                 canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID),
            reason: "Proprietary PdM black-box score (GE Predix, Siemens \
                 MindSphere, IBM Maximo Predict, Honeywell Forge, Aspen Mtell) \
                 exposes numeric anomaly / asset-health scores without a \
                 deterministic formula, model-identification anchor, training- \
                 data anchor, feature schema, tie-break, or numeric mode. \
                 Rejected unless reduced to a declared Deterministic_PdM_Score_ \
                 Proxy with all six contract fields brutally explicit in a later \
                 T.12.x. The court does NOT issue maintenance recommendations, \
                 root-cause certainty, or remaining-useful-life predictions; \
                 those terms appear here only to describe what is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID),
            reason: "Learned fault classifier (Wen et al.\\ 2017 CNN-based bearing \
                 fault classifier; Khan & Yairi 2018 deep-learning fault \
                 classifier review; commercial deep-learning equivalents) \
                 classifies rotating-machinery / process faults by learned \
                 representations trained on labelled fault corpora. Rejected \
                 unless reduced to a Deterministic_Fault_Classifier_Proxy with \
                 model-identification seed + training-data anchor (pinned \
                 dataset record-hash, e.g.\\ CWRU bearing dataset record-hash) + \
                 label schema (pinned failure-mode taxonomy) + tie-break law + \
                 numeric mode all brutally explicit in a later T.12.x. The court \
                 does NOT issue failure-mode classifications; the term appears \
                 here only to describe what is NOT admitted.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn industrial_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID,
            ),
            to_canonical_id: DetectorCanonicalId(SENSOR_BIAS_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID,
            ),
            to_canonical_id: DetectorCanonicalId(VALVE_HUNTING_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SENSOR_BIAS_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BEARING_VIBRATION_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(SLEW_SHOCK_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PLS_RESIDUAL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
    ]
}

/// Nine source refs supporting the industrial expansion.
fn industrial_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "kalman_1960",
            title: "A New Approach to Linear Filtering and Prediction Problems",
            year: 1960,
            venue: "Journal of Basic Engineering 82(1) (Kalman filter; canonical \
                state-space and innovation-sequence reference)",
        },
        ProposedSourceRef {
            citation_key: "mehra_peschon_1971",
            title: "An Innovations Approach to Fault Detection and Diagnosis in \
                Dynamic Systems",
            year: 1971,
            venue: "Automatica 7(5) (innovation-whiteness FDD reference)",
        },
        ProposedSourceRef {
            citation_key: "mcfadden_smith_1984",
            title: "Vibration Monitoring of Rolling Element Bearings by the High- \
                Frequency Resonance Technique - A Review",
            year: 1984,
            venue: "Tribology International 17(1) (bearing-defect-frequency \
                spectral reference)",
        },
        ProposedSourceRef {
            citation_key: "thomson_mcsa_2001",
            title: "On-line MCSA to diagnose shorted turns in low voltage stator \
                windings of 3-phase induction motors prior to failure",
            year: 2001,
            venue: "IEEE International Electric Machines and Drives Conference \
                (motor current signature analysis canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "antoni_spectral_kurtosis_2006",
            title: "The Spectral Kurtosis: A Useful Tool for Characterising Non- \
                Stationary Signals",
            year: 2006,
            venue: "Mechanical Systems and Signal Processing 20(2) (spectral- \
                kurtosis canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "isermann_fdd_textbook_2006",
            title: "Fault-Diagnosis Systems: An Introduction from Fault Detection \
                to Fault Tolerance",
            year: 2006,
            venue: "Springer (FDD textbook canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "iso_13373_vibration_condition_monitoring",
            title: "Condition Monitoring and Diagnostics of Machine Systems - \
                Vibration Condition Monitoring",
            year: 2002,
            venue: "ISO 13373 (vibration condition monitoring standard)",
        },
        ProposedSourceRef {
            citation_key: "iso_17359_condition_monitoring_framework",
            title: "Condition Monitoring and Diagnostics of Machines - General \
                Guidelines",
            year: 2018,
            venue: "ISO 17359 (general condition-monitoring framework)",
        },
        ProposedSourceRef {
            citation_key: "wen_khan_yairi_deep_learning_fault_2017_2018",
            title: "Deep-Learning Fault Classifier Reviews (Wen et al.\\ 2017 CNN- \
                based bearing classifier; Khan & Yairi 2018 deep-learning fault \
                classifier review)",
            year: 2018,
            venue: "Mechanical Systems and Signal Processing (rejection-shell \
                reference; learned classifier requires deterministic reduction)",
        },
    ]
}

/// Build the T.12.k industrial `DedupCourtDelta`.
fn build_industrial_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_k_industrial_delta",
        vec![
            DetectorCanonicalId(KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID),
        ],
        Vec::<DetectorAliasId>::new(),
        Vec::<DetectorCanonicalId>::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    )
}

// ---------------------------------------------------------------
// Public seed entry point
// ---------------------------------------------------------------

/// Build the T.12.k industrial `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_k_industrial_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_k_industrial_first_proposal",
        "T.12.k files the Industrial / Fault Detection and Diagnostics / \
         Condition Monitoring amendment proposal. Adds SIX genuinely new \
         canonical FDD primitives (Kalman innovation whiteness witness, \
         operating-regime transition witness, condition-indicator drift, fault \
         signature angle, contribution-plot spike, spectral kurtosis) at \
         reserved canonical ids 6101..=6106 with declared sensor + physical \
         quantity + unit + sampling + operating-regime + baseline + residual + \
         fault-signature-decision-law + confuser-nuisance-process contracts. \
         Records EIGHT ExistingCanonicalAuthorityResolution decisions (the \
         largest SEED-collision ratification of any T.12.x to date) keeping FFT \
         band-energy (SEED 12), PCA T² (19), PCA SPE / Q (20), PLS residual \
         (21), Residual envelope exit (22), Sensor bias detector (23), Actuator \
         stiction detector (24), Valve hunting (25) canonical without \
         duplication. Records TWO DomainTransferOf decisions: FFT band-energy \
         (SEED 12) as shared spectral ancestor for FaultDetectionDiagnostics; \
         Residual envelope exit (SEED 22) as shared envelope-boundary ancestor. \
         Records FOUR ParameterizationOf decisions (panel-candidate canonicals \
         that collapse on closer inspection - the strength of T.12.k): bearing \
         vibration band-energy is ParameterizationOf(FFT band-energy); motor \
         current signature anomaly is ParameterizationOf(FFT band-energy); \
         temperature envelope excursion is ParameterizationOf(Residual envelope \
         exit); pressure transient witness is ParameterizationOf(Slew shock). \
         Rejects TWO records as RejectedNotDeterministic (fifth T.12.x with two \
         rejections, following T.12.g / h / i / j): proprietary PdM black-box \
         score (6111 - GE Predix / Siemens MindSphere / IBM Maximo Predict / \
         Honeywell Forge / Aspen Mtell) and learned fault classifier (6112 - \
         Wen et al.\\ 2017, Khan & Yairi 2018). Panel-locked non-claim: T.12.k \
         admits deterministic condition-monitoring / FDD witnesses, not root- \
         cause certainty and not maintenance recommendations. Every \
         CanonicalAddition / ExistingCanonicalAuthorityResolution reason text \
         declares the full 9-field contract AND ends with the panel-locked non- \
         claim 'condition-monitoring witness, not a maintenance recommendation' \
         (pinned by t12_k_rejects_root_cause_claim_language). Does NOT mutate \
         SEED (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::FaultDetectionDiagnostics,
        build_industrial_expansion_batch(),
        build_industrial_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_k_industrial",
    )
}
