//! T.12.j — Medical / Biosignal: the tenth real literature
//! expansion proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.j files the Medical / Biosignal amendment proposal.
//! > It admits only deterministic biosignal witnesses whose
//! > signal source, sampling law, filtering law, morphology
//! > measurement law, baseline / noise handling, artifact
//! > confuser profile, and decision functional are declared;
//! > resolves SEED collisions; classifies measurement variants
//! > as parameterizations or domain transfers; rejects
//! > diagnostic classifiers and learned arrhythmia scores
//! > unless deterministically reduced; and preserves the frozen
//! > T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"Count signal
//! witnesses, not diagnoses. No sampling / filtering /
//! morphology law, no canonical admission."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.j does not admit medical diagnoses. It admits
//! > deterministic biosignal witnesses: morphology, interval,
//! > artifact, and spectral signal structures under declared
//! > sampling, filtering, and measurement laws. Clinical
//! > interpretation remains out of scope.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.j's design began with a grep of [`crate::seed::SEED`]
//! for every biosignal candidate. The walk found **four**
//! T.12.j-relevant primitives already canonical:
//!
//! * **R-peak interval anomaly (RR-interval)** at SEED id 49.
//! * **HRV time-domain shift** at SEED id 50.
//! * **QRS width anomaly** at SEED id 51.
//! * **ST-segment deviation proxy** at SEED id 52.
//!
//! All four become `ExistingCanonicalAuthorityResolution`
//! records under the `MedicalBiosignal` source class, each with
//! its specific signal-source + sampling-rate + filtering-law +
//! morphology-measurement-law + baseline-handling + artifact-
//! confuser-profile contract declared in the reason text.
//!
//! Eight genuinely new canonicals at reserved ids 6001..=6008
//! are admitted with declared biosignal-and-decision-law
//! contracts:
//!
//! * **P-wave morphology anomaly** (6001) — declared lead /
//!   channel identity, sampling rate, filtering law (bandpass
//!   for P-wave isolation), P-wave fiducial-detection law,
//!   morphology measurement (amplitude, duration, polarity),
//!   baseline definition, artifact exclusion law.
//! * **T-wave morphology anomaly** (6002) — declared lead /
//!   channel identity, sampling rate, T-wave fiducial-detection
//!   law (peak / onset / offset), morphology measurement
//!   (amplitude, duration, inversion polarity), baseline,
//!   artifact exclusion.
//! * **QT interval anomaly** (6003) — declared lead / channel
//!   identity, sampling rate, QT extraction law (Q-onset to
//!   T-offset detection), rate-correction formula (Bazett /
//!   Fridericia / Framingham / Hodges) optional, comparison law,
//!   threshold.
//! * **PR interval anomaly** (6004) — declared lead / channel
//!   identity, sampling rate, PR extraction law (P-onset to
//!   R-onset), comparison law, threshold.
//! * **Spectral HRV band shift** (6005) — declared RR-interval
//!   extraction law, beat inclusion / exclusion law, resampling
//!   law (cubic-spline at 4 Hz / Welch interpolation), spectral-
//!   estimation method (Welch periodogram / Lomb-Scargle / AR
//!   parametric), frequency-band definitions (VLF / LF / HF
//!   bounds), comparison law.
//! * **Baseline wander detector** (6006) — declared signal type
//!   (ECG / PPG / EMG), high-pass filter cutoff frequency,
//!   wander-band frequency law (typically below 0.5 Hz),
//!   minimum support, threshold.
//! * **Motion artifact detector** (6007) — declared artifact
//!   signal definition (accelerometer-corroborated motion /
//!   amplitude-saturation / sudden baseline jump), sensor
//!   source, duration / minimum support, decision threshold,
//!   confuser handling.
//! * **Saturation / clipping detector** (6008) — declared ADC
//!   bit-depth or saturation boundary (max / min observable
//!   sample value), duration / minimum support, clipping
//!   threshold (consecutive samples at boundary), decision law.
//!
//! Two domain transfers (panel-locked):
//!
//! * **FFT band-energy anomaly** (SEED 12) → `DomainTransferOf`
//!   for the `MedicalBiosignal` source class as the shared
//!   spectral ancestor (spectral HRV band shift 6005 is the
//!   biosignal-specific descendant with the RR-interval
//!   resampling contract added).
//! * **Residual envelope exit** (SEED 22) → `DomainTransferOf`
//!   for `MedicalBiosignal` as the shared envelope-boundary
//!   ancestor (saturation / clipping detector 6008 and motion
//!   artifact detector 6007 inherit the "exit-the-envelope"
//!   semantic; biosignal records add the ADC / sensor contract).
//!
//! Four parameterizations:
//!
//! * **RR-interval irregularity** (6009) → `ParameterizationOf
//!   (R-peak interval anomaly, SEED 49)` — beat-to-beat
//!   irregularity-of-irregularity variant of the RR-interval
//!   anomaly with declared irregularity metric (consecutive RR
//!   difference / variance over window).
//! * **HRV time-domain variant: SDNN / RMSSD / pNN50** (6010)
//!   → `ParameterizationOf(HRV time-domain shift, SEED 50)` —
//!   specific time-domain statistics (standard deviation of NN
//!   intervals / root-mean-square of successive differences /
//!   proportion of NN50) over the standard HRV time-domain
//!   primitive.
//! * **HRV band-specific LF / HF** (6011) → `ParameterizationOf
//!   (Spectral HRV band shift, 6005)` — frequency-band-specific
//!   variant (LF 0.04-0.15 Hz / HF 0.15-0.4 Hz / LF / HF ratio)
//!   over the general spectral HRV band shift.
//! * **Lead-specific ST deviation** (6012) → `ParameterizationOf
//!   (ST-segment deviation proxy, SEED 52)` — lead-specific
//!   parameterization (anterior / inferior / lateral lead
//!   groups).
//!
//! Two rejections (fourth T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, T.12.h,
//! T.12.i):
//!
//! * **Learned arrhythmia classifier** (6013) —
//!   `RejectedNotDeterministic`. Deep-learning ECG classifiers
//!   (Hannun et al.\ 2019; commercial arrhythmia detection
//!   models) classify rhythms by learned representations on
//!   labelled ECG corpora. Admission requires a future T.12.x
//!   proposal to admit a `Deterministic_Arrhythmia_Classifier
//!   _Proxy` canonical with model-identification seed,
//!   training-data anchor (pinned PhysioNet record-hash),
//!   label schema, tie-break, and numeric mode all brutally
//!   explicit. The court does NOT issue diagnostic verdicts.
//! * **Clinician-label-only diagnostic rule** (6014) —
//!   `RejectedNotDeterministic`. Rules of the form "if the
//!   clinician annotated the strip 'atrial fibrillation', mark
//!   it as a positive" depend on clinical-labeller-specific
//!   judgement, not a deterministic signal formula. Admission
//!   requires a deterministic signal-based reduction
//!   (morphology + interval + rhythm law) declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×8.
//! * `ExistingCanonicalAuthorityResolution` ×4.
//! * `DomainTransferOf` ×2 — FFT band-energy (SEED 12) and
//!   Residual envelope exit (SEED 22) as shared ancestors for
//!   `MedicalBiosignal`.
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 8 + 4 + 2 + 4 + 2 = **20 dedup-court records**.
//!
//! ## Diagnostic-claim discipline (panel-locked,
//! MOST IMPORTANT)
//!
//! Every CanonicalAddition reason text MUST describe its
//! record as a "signal witness" / "morphology measurement" /
//! "interval measurement" / "envelope detector" / "artifact
//! detector" / "spectral measurement" — NEVER as a clinical
//! diagnosis. The dedicated load-bearing negative
//! `t12_j_rejects_diagnostic_claim_language` scans every
//! CanonicalAddition reason for forbidden diagnostic terms
//! (arrhythmia, infarction, ischemia, fibrillation,
//! tachycardia, bradycardia, etc.) and fails if any appears
//! as a positive detector claim. Diagnostic terms may appear
//! only in REJECTION reason text (where they describe what is
//! NOT admitted) or in genealogy / motivation context.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.j biosignal
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
// Reserved id constants (panel-locked, 6001..=6014 bucket)
// ---------------------------------------------------------------

/// Reserved canonical id for P-wave morphology anomaly.
/// 6001..=6014 is the T.12.j bucket.
pub const P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID: u32 = 6001;

/// Reserved canonical id for T-wave morphology anomaly.
pub const T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID: u32 = 6002;

/// Reserved canonical id for QT interval anomaly.
pub const QT_INTERVAL_RESERVED_CANONICAL_ID: u32 = 6003;

/// Reserved canonical id for PR interval anomaly.
pub const PR_INTERVAL_RESERVED_CANONICAL_ID: u32 = 6004;

/// Reserved canonical id for Spectral HRV band shift.
pub const SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID: u32 = 6005;

/// Reserved canonical id for Baseline wander detector.
pub const BASELINE_WANDER_RESERVED_CANONICAL_ID: u32 = 6006;

/// Reserved canonical id for Motion artifact detector.
pub const MOTION_ARTIFACT_RESERVED_CANONICAL_ID: u32 = 6007;

/// Reserved canonical id for Saturation / clipping detector.
pub const CLIPPING_RESERVED_CANONICAL_ID: u32 = 6008;

/// Reserved id for RR-interval irregularity.
/// `ParameterizationOf(R-peak interval anomaly, SEED 49)`.
pub const RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID: u32 = 6009;

/// Reserved id for HRV time-domain variant (SDNN / RMSSD /
/// pNN50). `ParameterizationOf(HRV time-domain shift, SEED 50)`.
pub const HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID: u32 = 6010;

/// Reserved id for HRV band-specific LF / HF.
/// `ParameterizationOf(Spectral HRV band shift, 6005)`.
pub const HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID: u32 = 6011;

/// Reserved id for Lead-specific ST deviation.
/// `ParameterizationOf(ST-segment deviation proxy, SEED 52)`.
pub const LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID: u32 = 6012;

/// Reserved id for Learned arrhythmia classifier.
/// `RejectedNotDeterministic`. Reason text mentions clinical
/// terms only to describe what is NOT admitted.
pub const LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 6013;

/// Reserved id for Clinician-label-only diagnostic rule.
/// `RejectedNotDeterministic`. Reason text mentions clinical
/// terms only to describe what is NOT admitted.
pub const CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID: u32 = 6014;

// Existing SEED canonical ids referenced by T.12.j.

/// FFT band-energy anomaly — SEED canonical id 12. Shared
/// spectral ancestor for biosignal HRV band shifts.
pub const FFT_BAND_ENERGY_SEED_ID: u32 = 12;

/// Residual envelope exit — SEED canonical id 22. Shared
/// envelope-boundary ancestor for biosignal artifact / clipping
/// witnesses.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// R-peak interval anomaly (RR-interval) — SEED canonical id 49.
pub const R_PEAK_INTERVAL_SEED_ID: u32 = 49;

/// HRV time-domain shift — SEED canonical id 50.
pub const HRV_TIME_DOMAIN_SHIFT_SEED_ID: u32 = 50;

/// QRS width anomaly — SEED canonical id 51.
pub const QRS_WIDTH_SEED_ID: u32 = 51;

/// ST-segment deviation proxy — SEED canonical id 52.
pub const ST_SEGMENT_DEVIATION_SEED_ID: u32 = 52;

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
// Builders for the biosignal expansion batch
// ---------------------------------------------------------------

/// Build the biosignal `CorpusExpansionBatch` body.
fn build_biosignal_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_j_biosignal_first_proposal",
        SourceClass::MedicalBiosignal,
        biosignal_proposed_primitives(),
        biosignal_proposed_aliases(),
        biosignal_proposed_dedup_records(),
        biosignal_proposed_genealogy_edges(),
        biosignal_proposed_source_refs(),
    )
}

/// Fourteen proposed primitives: 8 canonical + 4 parameterization
/// shells + 2 rejection shells.
fn biosignal_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            display_name: "P-wave morphology anomaly",
            motivation: "P-wave morphology signal witness for ECG. Required \
                 contract: lead / channel identity, sampling rate, filtering law \
                 (bandpass to isolate P-wave content), P-wave fiducial-detection \
                 law (Pan-Tompkins-style preprocessing then peak / onset / offset \
                 location), morphology measurement (amplitude, duration, \
                 polarity), baseline definition, artifact exclusion law, \
                 minimum-beat support. Deterministic. Signal witness, not a \
                 medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            display_name: "T-wave morphology anomaly",
            motivation: "T-wave morphology signal witness for ECG. Required \
                 contract: lead / channel identity, sampling rate, filtering law, \
                 T-wave fiducial-detection law (peak / onset / offset), \
                 morphology measurement (amplitude, duration, polarity / \
                 inversion), baseline definition, artifact exclusion. \
                 Deterministic. Signal witness, not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(QT_INTERVAL_RESERVED_CANONICAL_ID),
            display_name: "QT interval anomaly",
            motivation: "QT interval measurement signal witness for ECG. Required \
                 contract: lead / channel identity, sampling rate, QT extraction \
                 law (Q-onset detection to T-offset detection), optional rate- \
                 correction formula (Bazett / Fridericia / Framingham / Hodges \
                 declared explicitly when used), comparison law, threshold, \
                 artifact exclusion. Deterministic. Interval signal witness, not \
                 a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PR_INTERVAL_RESERVED_CANONICAL_ID),
            display_name: "PR interval anomaly",
            motivation: "PR interval measurement signal witness for ECG. Required \
                 contract: lead / channel identity, sampling rate, PR extraction \
                 law (P-onset detection to R-onset detection), comparison law, \
                 threshold, artifact exclusion. Deterministic. Interval signal \
                 witness, not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID,
            ),
            display_name: "Spectral HRV band shift",
            motivation: "Spectral heart-rate-variability band-shift signal \
                 witness. Required contract: RR-interval extraction law, beat \
                 inclusion / exclusion law, artifact correction policy, \
                 resampling law (cubic-spline interpolation at 4 Hz / Welch \
                 interpolation / Lomb-Scargle on irregular samples), spectral- \
                 estimation method (Welch periodogram / Lomb-Scargle / AR \
                 parametric order), frequency-band definitions (VLF / LF / HF \
                 bounds), comparison law, threshold. Deterministic given the \
                 declared resampling + estimator choice. Spectral signal witness, \
                 not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BASELINE_WANDER_RESERVED_CANONICAL_ID),
            display_name: "Baseline wander detector",
            motivation: "Baseline-wander signal witness for ECG / PPG / EMG. \
                 Required contract: signal type (ECG / PPG / EMG channel), high- \
                 pass filter cutoff frequency declaration, wander-band frequency \
                 law (typically below 0.5 Hz for respiratory / motion baseline \
                 wander), minimum support, threshold. Deterministic. Artifact \
                 signal witness, not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MOTION_ARTIFACT_RESERVED_CANONICAL_ID),
            display_name: "Motion artifact detector",
            motivation: "Motion-artifact signal witness for biosignal channels. \
                 Required contract: artifact signal definition (accelerometer- \
                 corroborated motion event / amplitude-saturation pattern / \
                 sudden baseline jump exceeding declared threshold), sensor \
                 source, duration / minimum support, decision threshold, confuser \
                 handling (legitimate signal vs artifact). Deterministic. \
                 Artifact signal witness, not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CLIPPING_RESERVED_CANONICAL_ID),
            display_name: "Saturation / clipping detector",
            motivation: "ADC saturation / clipping signal witness for biosignal \
                 channels. Required contract: ADC bit-depth or explicit \
                 saturation boundary (max / min observable sample value declared), \
                 consecutive-samples-at-boundary threshold, duration / minimum \
                 support, decision law (clipping = N consecutive samples at the \
                 boundary value). Deterministic. Envelope-exit signal witness, \
                 not a medical diagnosis.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "RR-interval irregularity - parameterization shell",
            motivation: "Irregularity-of-irregularity parameterization of R-peak \
                 interval anomaly (SEED id 49). Required additional law: \
                 irregularity metric (consecutive RR difference / variance over \
                 window / Poincare SD1 / SD2). The court rules: RR-interval \
                 irregularity is ParameterizationOf(R-peak interval anomaly, \
                 SEED 49), NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "HRV time-domain SDNN / RMSSD / pNN50 - parameterization shell",
            motivation: "Specific time-domain statistic parameterization of HRV \
                 time-domain shift (SEED id 50): SDNN (standard deviation of NN \
                 intervals), RMSSD (root mean square of successive differences), \
                 pNN50 (proportion of consecutive NN intervals differing by more \
                 than 50 ms). The court rules: SDNN / RMSSD / pNN50 are \
                 ParameterizationOf(HRV time-domain shift, SEED 50), NOT new \
                 canonical primitives.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID),
            display_name: "HRV LF / HF band-specific - parameterization shell",
            motivation: "Frequency-band-specific parameterization of spectral \
                 HRV band shift (5005): LF (0.04 - 0.15 Hz), HF (0.15 - 0.4 Hz), \
                 LF / HF ratio. The court rules: HRV LF / HF band-specific \
                 measurements are ParameterizationOf(Spectral HRV band shift, \
                 6005), NOT new canonical primitives.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID),
            display_name: "Lead-specific ST deviation - parameterization shell",
            motivation: "Lead-group parameterization of ST-segment deviation \
                 proxy (SEED id 52): anterior leads (V1-V4), inferior leads \
                 (II, III, aVF), lateral leads (I, aVL, V5, V6). The court \
                 rules: lead-specific ST deviation is ParameterizationOf \
                 (ST-segment deviation proxy, SEED 52), NOT a new canonical \
                 primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Learned arrhythmia classifier - rejected shell",
            motivation: "Deep-learning ECG classifiers (Hannun et al.\\ 2019; \
                 commercial deep-learning rhythm detectors) classify ECG strips \
                 by learned representations trained on labelled corpora. The \
                 court does NOT admit learned classifiers to the dedup-court \
                 delta's new_canonical_records. A future T.12.x proposal may \
                 admit a Deterministic_Arrhythmia_Classifier_Proxy canonical \
                 only if the model-identification seed, training-data anchor \
                 (pinned PhysioNet record-hash), label schema (pinned), tie- \
                 break law, and numeric mode are all brutally explicit. The \
                 court does NOT issue diagnostic verdicts. Diagnostic terms \
                 such as arrhythmia / fibrillation appear in this reason text \
                 only to describe what is NOT admitted.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Clinician-label-only diagnostic rule - rejected shell",
            motivation: "Rules of the form 'if the clinician annotated the strip \
                 as positive for a diagnosis, mark it as a positive' depend on \
                 clinical-labeller-specific judgement rather than a deterministic \
                 signal formula. The court does NOT admit clinician-label-only \
                 diagnostic rules. Admission requires a deterministic signal- \
                 based reduction (morphology + interval + rhythm law all \
                 declared) in a later T.12.x. The court does NOT issue \
                 diagnostic verdicts. Diagnostic terms appear in this reason \
                 text only to describe what is NOT admitted.",
        },
    ]
}

/// Zero alias claims (T.12.j routes everything through dedup
/// records and existing-canonical authority resolutions).
fn biosignal_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty dedup-court decisions on the biosignal batch.
fn biosignal_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            reason: "P-wave morphology anomaly: declared lead / channel identity, \
                 sampling rate, filtering law (bandpass), P-wave fiducial- \
                 detection law (Pan-Tompkins preprocessing then peak / onset / \
                 offset), morphology measurement (amplitude, duration, polarity), \
                 baseline definition, artifact exclusion law, minimum beat \
                 support. Signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            reason: "T-wave morphology anomaly: declared lead / channel identity, \
                 sampling rate, filtering law, T-wave fiducial-detection law \
                 (peak / onset / offset), morphology measurement (amplitude, \
                 duration, polarity / inversion), baseline definition, artifact \
                 exclusion. Signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(QT_INTERVAL_RESERVED_CANONICAL_ID),
            reason: "QT interval anomaly: declared lead / channel identity, \
                 sampling rate, QT extraction law (Q-onset to T-offset \
                 detection), optional rate-correction formula (Bazett / \
                 Fridericia / Framingham / Hodges declared when used), \
                 comparison law, threshold, artifact exclusion. Interval signal \
                 witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PR_INTERVAL_RESERVED_CANONICAL_ID),
            reason: "PR interval anomaly: declared lead / channel identity, \
                 sampling rate, PR extraction law (P-onset to R-onset \
                 detection), comparison law, threshold, artifact exclusion. \
                 Interval signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Spectral HRV band shift: declared RR-interval extraction \
                 law, beat inclusion / exclusion law, artifact correction \
                 policy, resampling law (cubic-spline at 4 Hz / Welch / Lomb- \
                 Scargle on irregular samples), spectral-estimation method \
                 (Welch periodogram / Lomb-Scargle / AR parametric), frequency- \
                 band definitions (VLF / LF / HF bounds), comparison law, \
                 threshold. Spectral signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BASELINE_WANDER_RESERVED_CANONICAL_ID),
            reason: "Baseline wander detector: declared signal type (ECG / PPG / \
                 EMG channel), high-pass filter cutoff frequency, wander-band \
                 frequency law (typically below 0.5 Hz), minimum support, \
                 threshold. Artifact signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MOTION_ARTIFACT_RESERVED_CANONICAL_ID),
            reason: "Motion artifact detector: declared artifact signal \
                 definition (accelerometer-corroborated motion event / \
                 amplitude-saturation pattern / sudden baseline jump exceeding \
                 declared threshold), sensor source, duration / minimum support, \
                 decision threshold, confuser handling. Artifact signal witness, \
                 not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CLIPPING_RESERVED_CANONICAL_ID),
            reason: "Saturation / clipping detector: declared ADC bit-depth or \
                 explicit saturation boundary (max / min observable sample \
                 value), consecutive-samples-at-boundary threshold, duration / \
                 minimum support, decision law (clipping = N consecutive samples \
                 at the boundary value). Envelope-exit signal witness, not a \
                 medical diagnosis.",
        },
        // -- 4 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(R_PEAK_INTERVAL_SEED_ID),
            reason: "R-peak interval anomaly stays canonical at SEED id 49. \
                 Declared signal source (ECG lead / channel), sampling rate, \
                 filtering law, R-peak fiducial-detection law (Pan-Tompkins \
                 1985 or equivalent), RR-interval extraction, comparison law, \
                 artifact exclusion. No duplicate admitted; RR-interval \
                 irregularity (record 6009 below) collapses here as \
                 ParameterizationOf. Signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(HRV_TIME_DOMAIN_SHIFT_SEED_ID),
            reason: "HRV time-domain shift stays canonical at SEED id 50. \
                 Declared RR-interval extraction law (Pan-Tompkins or \
                 equivalent), beat inclusion / exclusion law (Task Force 1996 \
                 standards), artifact correction policy, window length, time- \
                 domain statistic family (SDNN / RMSSD / pNN50 / mean NN), \
                 comparison law. No duplicate admitted; SDNN / RMSSD / pNN50 \
                 variants (record 6010 below) collapse here as \
                 ParameterizationOf. Signal witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(QRS_WIDTH_SEED_ID),
            reason: "QRS width anomaly stays canonical at SEED id 51. Declared \
                 lead / channel identity, sampling rate, QRS-detection law \
                 (Pan-Tompkins 1985 or equivalent), QRS-onset / -offset \
                 detection law, width measurement law, comparison law, \
                 threshold, artifact exclusion. No duplicate admitted. Signal \
                 witness, not a medical diagnosis.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ST_SEGMENT_DEVIATION_SEED_ID),
            reason: "ST-segment deviation proxy stays canonical at SEED id 52. \
                 Declared lead / channel identity, sampling rate, J-point \
                 detection law, isoelectric baseline definition, ST-deviation \
                 measurement law (mV deviation at fixed offset from J-point), \
                 comparison law, threshold, artifact exclusion. No duplicate \
                 admitted; lead-specific ST deviation (record 6012 below) \
                 collapses here as ParameterizationOf. Signal witness, not a \
                 medical diagnosis.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly (SEED id 12) is the shared \
                 spectral ancestor for the MedicalBiosignal source class. \
                 Spectral HRV band shift (6005) is the biosignal-specific \
                 descendant that adds the RR-interval resampling contract on \
                 top of the FFT band-energy primitive. The court records the \
                 domain transfer without re-canonicalising FFT band-energy.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the shared envelope- \
                 boundary ancestor for the MedicalBiosignal source class. \
                 Saturation / clipping detector (6008) and motion artifact \
                 detector (6007) inherit the exit-the-envelope semantic; the \
                 biosignal descendants add ADC / sensor / consecutive-sample \
                 contracts on top. The court records the domain transfer \
                 without re-canonicalising Residual envelope exit.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID),
            reason: "RR-interval irregularity is ParameterizationOf(R-peak \
                 interval anomaly, SEED id 49). Irregularity-of-irregularity \
                 parameterization with declared irregularity metric \
                 (consecutive RR difference / variance over window / Poincare \
                 SD1 / SD2). The court declines to admit RR-interval \
                 irregularity as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID),
            reason: "HRV time-domain SDNN / RMSSD / pNN50 is ParameterizationOf \
                 (HRV time-domain shift, SEED id 50). Specific time-domain \
                 statistic parameterization (SDNN / RMSSD / pNN50 as named \
                 by Task Force 1996). The court declines to admit these as \
                 new canonical primitives.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID),
            reason: "HRV LF / HF band-specific is ParameterizationOf(Spectral \
                 HRV band shift, 6005). Frequency-band-specific parameterization \
                 (LF 0.04 - 0.15 Hz / HF 0.15 - 0.4 Hz / LF / HF ratio per \
                 Task Force 1996). The court declines to admit these as new \
                 canonical primitives.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID),
            reason: "Lead-specific ST deviation is ParameterizationOf(ST-segment \
                 deviation proxy, SEED id 52). Lead-group parameterization \
                 (anterior / inferior / lateral lead groups). The court \
                 declines to admit lead-specific ST deviation as a new \
                 canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_ARRHYTHMIA_CLASSIFIER_RESERVED_PRIMITIVE_ID),
            reason: "Learned arrhythmia classifier (Hannun et al.\\ 2019 deep- \
                 learning ECG classifier; commercial deep-learning rhythm \
                 detectors) classifies ECG strips by learned representations \
                 trained on labelled corpora. Rejected unless reduced to a \
                 Deterministic_Arrhythmia_Classifier_Proxy with model- \
                 identification seed + training-data anchor (pinned PhysioNet \
                 record-hash) + label schema pinned + tie-break law + numeric \
                 mode all brutally explicit in a later T.12.x. The court does \
                 NOT issue diagnostic verdicts; diagnostic terms appear here \
                 only to describe what is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(CLINICIAN_LABEL_DIAGNOSTIC_RESERVED_PRIMITIVE_ID),
            reason: "Clinician-label-only diagnostic rule (rules of the form 'if \
                 the clinician annotated the strip as positive for a diagnosis, \
                 mark it as a positive') depends on clinical-labeller-specific \
                 judgement rather than a deterministic signal formula. Rejected \
                 unless reduced to a deterministic signal-based reduction \
                 (morphology + interval + rhythm law all declared) in a later \
                 T.12.x. The court does NOT issue diagnostic verdicts; \
                 diagnostic terms appear here only to describe what is NOT \
                 admitted.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn biosignal_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(QRS_WIDTH_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(QRS_WIDTH_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(QT_INTERVAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(R_PEAK_INTERVAL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PR_INTERVAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(R_PEAK_INTERVAL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(HRV_TIME_DOMAIN_SHIFT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BASELINE_WANDER_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MOTION_ARTIFACT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CLIPPING_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RR_INTERVAL_IRREGULARITY_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(R_PEAK_INTERVAL_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HRV_TIME_DOMAIN_VARIANT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(HRV_TIME_DOMAIN_SHIFT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HRV_LF_HF_BAND_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(LEAD_SPECIFIC_ST_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(ST_SEGMENT_DEVIATION_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the biosignal expansion.
fn biosignal_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "pan_tompkins_qrs_1985",
            title: "A Real-Time QRS Detection Algorithm",
            year: 1985,
            venue: "IEEE Transactions on Biomedical Engineering BME-32(3) \
                doi:10.1109/TBME.1985.325532",
        },
        ProposedSourceRef {
            citation_key: "hrv_task_force_1996",
            title: "Heart Rate Variability: Standards of Measurement, Physiological \
                Interpretation and Clinical Use",
            year: 1996,
            venue: "Circulation 93(5) / European Heart Journal 17(3) (Task Force \
                of the European Society of Cardiology and the North American \
                Society of Pacing and Electrophysiology)",
        },
        ProposedSourceRef {
            citation_key: "moody_mark_mit_bih_2001",
            title: "The Impact of the MIT-BIH Arrhythmia Database",
            year: 2001,
            venue: "IEEE Engineering in Medicine and Biology Magazine 20(3); \
                PhysioNet record source",
        },
        ProposedSourceRef {
            citation_key: "aami_ec57_2012",
            title: "Testing and reporting performance results of cardiac rhythm \
                and ST segment measurement algorithms",
            year: 2012,
            venue: "ANSI / AAMI EC57:2012 (performance-test standard)",
        },
        ProposedSourceRef {
            citation_key: "welch_periodogram_1967",
            title: "The Use of Fast Fourier Transform for the Estimation of Power \
                Spectra",
            year: 1967,
            venue: "IEEE Transactions on Audio and Electroacoustics AU-15(2) \
                (Welch periodogram; spectral HRV reference)",
        },
        ProposedSourceRef {
            citation_key: "lomb_scargle_periodogram_1976",
            title: "Least-Squares Frequency Analysis of Unequally Spaced Data",
            year: 1976,
            venue: "Astrophysics and Space Science 39(2) (Lomb-Scargle on \
                irregular RR-interval samples)",
        },
        ProposedSourceRef {
            citation_key: "friesen_ecg_noise_1990",
            title: "A Comparison of the Noise Sensitivity of Nine QRS Detection \
                Algorithms",
            year: 1990,
            venue: "IEEE Transactions on Biomedical Engineering 37(1) \
                (baseline wander / motion artifact reference)",
        },
        ProposedSourceRef {
            citation_key: "ansi_aami_ec11",
            title: "Diagnostic Electrocardiographic Devices",
            year: 1991,
            venue: "ANSI / AAMI EC11 (ADC sample-rate / amplitude resolution / \
                clipping reference)",
        },
        ProposedSourceRef {
            citation_key: "hannun_ecg_deep_learning_2019",
            title: "Cardiologist-level Arrhythmia Detection and Classification \
                in Ambulatory Electrocardiograms Using a Deep Neural Network",
            year: 2019,
            venue: "Nature Medicine 25(1) (rejection-shell reference; learned \
                classifier requires deterministic reduction)",
        },
    ]
}

/// Build the T.12.j biosignal `DedupCourtDelta`.
fn build_biosignal_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_j_biosignal_delta",
        vec![
            DetectorCanonicalId(P_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(T_WAVE_MORPHOLOGY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(QT_INTERVAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PR_INTERVAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SPECTRAL_HRV_BAND_SHIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BASELINE_WANDER_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MOTION_ARTIFACT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CLIPPING_RESERVED_CANONICAL_ID),
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

/// Build the T.12.j biosignal `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_j_biosignal_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_j_biosignal_first_proposal",
        "T.12.j files the Medical / Biosignal amendment proposal. Adds eight \
         genuinely new canonical biosignal primitives (P-wave morphology anomaly, \
         T-wave morphology anomaly, QT interval anomaly, PR interval anomaly, \
         spectral HRV band shift, baseline wander detector, motion artifact \
         detector, saturation / clipping detector) at reserved canonical ids \
         6001..=6008 with declared signal-source + sampling-rate + filtering-law \
         + morphology-or-interval-measurement-law + baseline-handling + artifact- \
         confuser-profile + decision-functional contracts. Records four \
         ExistingCanonicalAuthorityResolution decisions keeping R-peak interval \
         anomaly (SEED id 49), HRV time-domain shift (id 50), QRS width anomaly \
         (id 51), and ST-segment deviation proxy (id 52) canonical under the \
         MedicalBiosignal source class without duplication. Records two \
         DomainTransferOf decisions: FFT band-energy anomaly (SEED 12) as the \
         shared spectral ancestor for biosignal HRV bands; Residual envelope exit \
         (SEED 22) as the shared envelope-boundary ancestor for biosignal \
         artifact / clipping witnesses. Records four ParameterizationOf \
         decisions: RR-interval irregularity is ParameterizationOf(R-peak \
         interval anomaly); HRV time-domain SDNN / RMSSD / pNN50 is \
         ParameterizationOf(HRV time-domain shift); HRV LF / HF band-specific is \
         ParameterizationOf(Spectral HRV band shift); lead-specific ST deviation \
         is ParameterizationOf(ST-segment deviation proxy). Rejects TWO biosignal \
         records as RejectedNotDeterministic (fourth T.12.x proposal with two \
         rejection records in one commit, following T.12.g / T.12.h / T.12.i): \
         learned arrhythmia classifier (6013 - Hannun et al.\\ 2019 deep-learning \
         ECG classifier and commercial equivalents) and clinician-label-only \
         diagnostic rule (6014). Panel-locked non-claim: T.12.j does not admit \
         medical diagnoses. It admits deterministic biosignal witnesses - \
         morphology, interval, artifact, and spectral signal structures under \
         declared sampling, filtering, and measurement laws. Clinical \
         interpretation remains out of scope. Every CanonicalAddition reason \
         text describes its record as a signal witness / morphology measurement \
         / interval measurement / envelope detector / artifact detector / \
         spectral measurement, NEVER as a clinical diagnosis (pinned by \
         t12_j_rejects_diagnostic_claim_language). Does NOT mutate SEED \
         (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::MedicalBiosignal,
        build_biosignal_expansion_batch(),
        build_biosignal_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_j_biosignal",
    )
}
