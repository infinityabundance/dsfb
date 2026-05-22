//! T.12.f — Time-Series Structure / Control Residuals: the
//! sixth real literature expansion proposal filed through the
//! T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.f files the Time-Series Structure / Control
//! > Residuals amendment proposal. It admits only deterministic
//! > time-structure and residual-observer witnesses whose model,
//! > lag, residual, envelope, innovation, sampling, and decision
//! > laws are declared; resolves SEED collisions; classifies
//! > model variants as parameterizations; rejects stochastic or
//! > unidentified model-fitting claims unless deterministically
//! > reduced; and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"A model is not a
//! detector until the residual and decision law are declared."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.f's design began with a grep of [`crate::seed::SEED`]
//! for every time-series / control-residual candidate. The walk
//! found **four** primitives already canonical:
//!
//! * **Sensor bias detector** at SEED id 23 — control-residual
//!   primitive. Catches "sensor-bias" / "offset-fault" aliases.
//! * **Actuator stiction detector** at SEED id 24 — control-
//!   residual primitive. Catches "stick-slip" / "actuator-
//!   stiction" aliases.
//! * **Valve hunting (control-loop oscillation) detector** at
//!   SEED id 25 — control-residual primitive. Catches "limit-
//!   cycle" / "valve-hunting" aliases.
//! * **Error burst** at SEED id 41 — debug / observability
//!   primitive. Catches "burstiness" / "burst-rate" aliases via
//!   ParameterizationOf record below.
//!
//! Two additional SEED records, already recognised in T.12.e
//! under `SignalProcessing`, are recognised again here under
//! `TimeSeriesStructure` (different source class; same
//! authoritative record):
//!
//! * **Residual envelope exit** at SEED id 22 — fundamental
//!   residual-witness primitive applicable across time-series
//!   AND control-residual contexts.
//! * **Autocorrelation-coefficient break** at SEED id 40 —
//!   fundamental time-series structural witness.
//!
//! All six become `ExistingCanonicalAuthorityResolution`
//! records. Of the remaining panel candidates the court ruled:
//!
//! * **AR residual anomaly** is structurally distinct: declared
//!   AR(p) model order + fit law (closed-form Yule-Walker /
//!   Burg / fixed coefficients) + residual definition (one-step-
//!   ahead prediction error) + threshold + sampling law.
//!   `CanonicalAddition` at reserved id 5601.
//! * **ARIMA residual anomaly** is structurally distinct from
//!   AR: declared order (p, d, q) + integration + moving-average
//!   term + fit law + residual + threshold + sampling.
//!   `CanonicalAddition` at reserved id 5602.
//! * **STL residual anomaly** (Cleveland 1990) is structurally
//!   distinct: seasonal-trend decomposition by LOESS; declared
//!   seasonality period + LOESS smoother bandwidth + iteration
//!   count + residual definition + threshold + sampling law.
//!   `CanonicalAddition` at reserved id 5603.
//! * **Lag-correlation break** is structurally distinct from
//!   the SEED autocorrelation-coefficient break (40): multi-lag
//!   correlation matrix break rather than single-lag. Declared
//!   lag range + autocorrelation convention + normalization +
//!   threshold + window. `CanonicalAddition` at reserved id
//!   5604.
//! * **Variance-ratio shift** (Lo & MacKinlay 1988) is
//!   structurally distinct: ratio of long-window variance to
//!   short-window variance as a stationarity / random-walk
//!   test. Declared window-pair law + ratio definition +
//!   threshold. `CanonicalAddition` at reserved id 5605.
//! * **Run-length anomaly** is structurally distinct from
//!   error-burst (which counts events in a window): tracks
//!   consecutive-event-run lengths. Declared event definition +
//!   run-length law + threshold. `CanonicalAddition` at reserved
//!   id 5606.
//! * **Observer residual** is structurally distinct: state-
//!   estimator residual y_measured - y_estimated. Declared
//!   state model + measurement model + observer gain law +
//!   residual definition + envelope law + threshold. The most
//!   general control-residual primitive. `CanonicalAddition` at
//!   reserved id 5607.
//! * **Parity-space residual** is structurally distinct from
//!   observer residual: algebraic redundancy equation residual
//!   rather than state-estimator residual. Declared parity
//!   equations + residual definition + threshold. `CanonicalAddition`
//!   at reserved id 5608.
//! * **Innovation sequence anomaly** is the Kalman-specific
//!   parameterization of observer residual: declared filter
//!   (Kalman / extended Kalman / unscented Kalman) + state-
//!   noise covariance Q + measurement-noise covariance R +
//!   innovation definition + threshold. `ParameterizationOf
//!   (Observer residual, 5607)` at reserved id 5609.
//! * **Periodicity break** is the period-extraction
//!   parameterization of lag-correlation break: declared peak
//!   selection law + period candidate set + tie handling.
//!   `ParameterizationOf(Lag-correlation break, 5604)` at
//!   reserved id 5610.
//! * **Burstiness index** is the general-event-count
//!   parameterization of Error burst (SEED 41): declared event
//!   definition + window length + threshold. `ParameterizationOf
//!   (Error burst, SEED 41)` at reserved id 5611.
//! * **Unidentified-model anomaly** (any "ARIMA with auto-
//!   determined order via random search", "Kalman without
//!   declared Q / R", "STL with adaptive seasonality") is
//!   randomized / unidentified in origin. Acknowledged at
//!   reserved id 5612 but `RejectedNotDeterministic` — admitted
//!   neither to SEED nor to `new_canonical_records` unless a
//!   future T.12.x proposal admits a
//!   `Deterministic_Model_Identification_Proxy` canonical with
//!   the model-order-search seed, identification algorithm,
//!   fit-data anchor, tie-break law, and numeric mode declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! T.12.f exercises all five panel-locked court-delta categories
//! (the wire-name set closed at T.12.d remains closed):
//!
//! * `CanonicalAddition` ×8 — AR / ARIMA / STL residual,
//!   lag-correlation break, variance-ratio shift, run-length
//!   anomaly, observer residual, parity-space residual.
//! * `ExistingCanonicalAuthorityResolution` ×6 — Residual
//!   envelope exit (22), Sensor bias detector (23), Actuator
//!   stiction (24), Valve hunting (25), Autocorrelation-
//!   coefficient break (40), Error burst (41).
//! * `DomainTransferOf` ×1 — Residual envelope exit (22) as
//!   the shared residual-witness ancestor for the
//!   `TimeSeriesStructure` source class (recognised by both
//!   time-series-residual AND control-residual sub-families).
//! * `ParameterizationOf` ×3 — Innovation sequence
//!   (ParameterizationOf observer residual 5607); Periodicity
//!   break (ParameterizationOf lag-correlation break 5604);
//!   Burstiness index (ParameterizationOf Error burst SEED 41).
//! * `RejectedNotDeterministic` ×1 — Unidentified-model
//!   anomaly (5612).
//!
//! Total: 8 + 6 + 1 + 3 + 1 = **19 dedup-court records**.
//!
//! ## Residual-and-decision-law discipline (panel-required)
//!
//! Every CanonicalAddition + ExistingCanonicalAuthorityResolution
//! record's reason text declares its model, residual, envelope,
//! and decision laws — without which a model is not a detector.
//! Each declaration is pinned by a dedicated panel-required
//! load-bearing negative.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11/S1.3/T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.f time-series
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
// Reserved id constants (panel-locked)
// ---------------------------------------------------------------

/// Reserved canonical id for AR residual anomaly. 5601..5612 is
/// the T.12.f bucket (T.12.a 5001+, T.12.b 5201+, T.12.c 5301+,
/// T.12.d 5401+, T.12.e 5501+).
pub const AR_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5601;

/// Reserved canonical id for ARIMA residual anomaly (Box &
/// Jenkins 1970).
pub const ARIMA_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5602;

/// Reserved canonical id for STL residual anomaly (Cleveland,
/// Cleveland, McRae & Terpenning 1990 --- Seasonal-Trend
/// decomposition by LOESS).
pub const STL_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5603;

/// Reserved canonical id for lag-correlation break (multi-lag
/// correlation matrix break, distinct from the single-lag SEED
/// autocorrelation-coefficient break at id 40).
pub const LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID: u32 = 5604;

/// Reserved canonical id for variance-ratio shift (Lo &
/// MacKinlay 1988 --- random-walk / stationarity test).
pub const VARIANCE_RATIO_RESERVED_CANONICAL_ID: u32 = 5605;

/// Reserved canonical id for run-length anomaly (tracks
/// consecutive-event-run lengths, distinct from Error burst's
/// window-count functional).
pub const RUN_LENGTH_RESERVED_CANONICAL_ID: u32 = 5606;

/// Reserved canonical id for observer residual (general state-
/// estimator residual y_measured - y_estimated; the most
/// general control-residual primitive).
pub const OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5607;

/// Reserved canonical id for parity-space residual (Chow &
/// Willsky 1984 --- algebraic redundancy equation residual,
/// structurally distinct from state-estimator-based observer
/// residual).
pub const PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5608;

/// Reserved id for innovation sequence anomaly. Kalman-
/// specific parameterization of observer residual:
/// `ParameterizationOf(Observer residual, 5607)`.
pub const INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID: u32 = 5609;

/// Reserved id for periodicity break. Period-extraction
/// parameterization of lag-correlation break:
/// `ParameterizationOf(Lag-correlation break, 5604)`.
pub const PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID: u32 = 5610;

/// Reserved id for burstiness index. General-event-count
/// parameterization of Error burst:
/// `ParameterizationOf(Error burst, SEED 41)`.
pub const BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID: u32 = 5611;

/// Reserved id for unidentified-model anomaly. Any literature
/// claim like "ARIMA with auto-determined order via random
/// search", "Kalman without declared Q / R", or "STL with
/// adaptive seasonality" is randomized / unidentified in origin.
/// `RejectedNotDeterministic` --- admitted neither to SEED nor
/// to `new_canonical_records` unless a future T.12.x proposal
/// admits a `Deterministic_Model_Identification_Proxy`
/// canonical with the model-order-search seed, identification
/// algorithm, fit-data anchor, tie-break law, and numeric mode
/// declared.
pub const UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID: u32 = 5612;

// Existing SEED canonical ids referenced by T.12.f.

/// Residual envelope exit --- SEED canonical id 22. Recognised
/// by T.12.e under SignalProcessing; T.12.f recognises it again
/// under TimeSeriesStructure as the shared residual-witness
/// ancestor.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// Sensor bias detector --- SEED canonical id 23 (control-
/// residual primitive).
pub const SENSOR_BIAS_SEED_ID: u32 = 23;

/// Actuator stiction detector --- SEED canonical id 24
/// (control-residual primitive).
pub const ACTUATOR_STICTION_SEED_ID: u32 = 24;

/// Valve hunting detector --- SEED canonical id 25 (control-
/// residual primitive).
pub const VALVE_HUNTING_SEED_ID: u32 = 25;

/// Autocorrelation-coefficient break --- SEED canonical id 40.
/// Recognised by T.12.e under SignalProcessing; T.12.f
/// recognises it again under TimeSeriesStructure.
pub const AUTOCORRELATION_BREAK_SEED_ID: u32 = 40;

/// Error burst --- SEED canonical id 41 (debug / observability
/// primitive; burstiness index ParameterizationOf parent).
pub const ERROR_BURST_SEED_ID: u32 = 41;

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
// Builders for the time-series expansion batch
// ---------------------------------------------------------------

/// Build the time-series `CorpusExpansionBatch` body.
fn build_timeseries_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_f_timeseries_first_proposal",
        SourceClass::TimeSeriesStructure,
        timeseries_proposed_primitives(),
        timeseries_proposed_aliases(),
        timeseries_proposed_dedup_records(),
        timeseries_proposed_genealogy_edges(),
        timeseries_proposed_source_refs(),
    )
}

/// Twelve proposed primitives: 8 genuinely new canonicals + 3
/// parameterization shells + 1 rejection shell.
fn timeseries_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(AR_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "AR residual anomaly",
            motivation: "AR(p) autoregressive-model residual-anomaly detector. The \
                 detector is the one-step-ahead prediction error of an AR(p) model. \
                 Required model-and-decision-law declarations: AR order p, fit law \
                 (closed-form Yule-Walker / Burg / fixed-coefficient table), residual \
                 definition (signed / absolute / squared), threshold law, and \
                 sampling law. Deterministic when the AR coefficients are pinned or \
                 the fit law is closed-form.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(ARIMA_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "ARIMA residual anomaly",
            motivation: "ARIMA(p, d, q) residual-anomaly detector (Box & Jenkins 1970). \
                 Structurally distinct from AR residual: integration order d > 0 and \
                 moving-average order q > 0 introduce different decision functionals. \
                 Required model-and-decision-law declarations: ARIMA order (p, d, q), \
                 fit law, residual definition, forecast horizon, threshold law, and \
                 sampling law.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(STL_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "STL residual anomaly",
            motivation: "STL (Cleveland, Cleveland, McRae & Terpenning 1990) Seasonal-\
                 Trend decomposition by LOESS residual-anomaly detector. \
                 Structurally distinct from AR / ARIMA: nonparametric LOESS smoother \
                 rather than ARMA model. Required model-and-decision-law \
                 declarations: seasonality period, LOESS smoother bandwidth, \
                 inner / outer iteration count, residual definition (observed - \
                 (trend + seasonal)), threshold law, and sampling law.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            display_name: "Lag-correlation break (multi-lag)",
            motivation: "Multi-lag autocorrelation-matrix-break detector. \
                 Structurally distinct from the single-lag autocorrelation-coefficient \
                 break at SEED id 40: the decision functional is a break across a \
                 declared lag range, not a single lag. Required decision-law \
                 declarations: lag range, autocorrelation convention (biased / \
                 unbiased), normalization, threshold, and window.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(VARIANCE_RATIO_RESERVED_CANONICAL_ID),
            display_name: "Variance-ratio shift (Lo & MacKinlay)",
            motivation: "Lo & MacKinlay (1988) variance-ratio test: ratio of long-\
                 window variance to short-window variance (q-lag VR statistic) used \
                 as a random-walk / stationarity test. Required decision-law \
                 declarations: window-pair law (short window size, long window size), \
                 ratio definition (homoskedasticity-corrected vs heteroskedasticity-\
                 robust), threshold law, and sampling law.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RUN_LENGTH_RESERVED_CANONICAL_ID),
            display_name: "Run-length anomaly",
            motivation: "Tracks consecutive-event-run lengths (e.g. consecutive \
                 above-threshold values, consecutive same-sign residuals). \
                 Structurally distinct from Error burst at SEED id 41 (which counts \
                 events in a sliding window): the decision functional is the maximum \
                 / current run length, not the count. Required decision-law \
                 declarations: event definition, run-length law (max / current / \
                 percentile), threshold, and sampling law.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "Observer residual (general state-estimator residual)",
            motivation: "State-estimator residual y_measured - y_estimated for an \
                 explicitly declared observer (Luenberger / Kalman / nonlinear \
                 estimator). The MOST GENERAL control-residual primitive. Required \
                 model-and-decision-law declarations: state model (A, B matrices \
                 / nonlinear f), measurement model (C matrix / nonlinear h), \
                 observer gain law (L or K), residual definition, envelope law, \
                 threshold, and sampling law. Deterministic when matrices / gain \
                 are pinned. Innovation sequence (record 5609 below) is the \
                 Kalman-specific parameterization of this general primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "Parity-space residual (Chow & Willsky 1984)",
            motivation: "Algebraic-redundancy parity-equation residual. \
                 Structurally distinct from observer residual: uses linear-algebraic \
                 parity relations between sensor outputs (over a sliding window) \
                 rather than a state-estimator. Required model-and-decision-law \
                 declarations: parity equations (W matrix), residual definition, \
                 envelope law, threshold, and sampling law.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID),
            display_name: "Innovation sequence anomaly - parameterization shell",
            motivation: "Kalman-specific parameterization of observer residual \
                 (5607). The innovation sequence is the residual under a Kalman \
                 filter with declared Q + R covariances; whitening / normalization \
                 properties follow from the filter law. The court rules: innovation \
                 sequence is ParameterizationOf(Observer residual, 5607), NOT a \
                 new canonical primitive. Appears in proposed_primitives but NOT \
                 in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID),
            display_name: "Periodicity break - parameterization shell",
            motivation: "Period-extraction parameterization of lag-correlation \
                 break (5604). Adds a declared peak-selection law over the \
                 candidate-lag range; the family-level decision functional is the \
                 lag-correlation break's. The court rules: periodicity break is \
                 ParameterizationOf(Lag-correlation break, 5604), NOT a new \
                 canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID),
            display_name: "Burstiness index - parameterization shell",
            motivation: "General-event-count parameterization of Error burst (SEED \
                 id 41). Same window-count decision functional as Error burst; the \
                 parameterization generalises the event type from 'error' to any \
                 declared event class. The court rules: burstiness index is \
                 ParameterizationOf(Error burst, SEED 41), NOT a new canonical \
                 primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID),
            display_name: "Unidentified-model anomaly - rejected shell",
            motivation: "Any literature claim of the form 'ARIMA with auto-determined \
                 order via random search', 'Kalman filter without declared Q / R \
                 covariances', 'STL with adaptive seasonality', or 'observer with \
                 fit-during-deployment' is randomized / unidentified in origin: the \
                 model parameters change per invocation. The court does NOT admit \
                 unidentified-model anomaly to the dedup-court delta's \
                 new_canonical_records. A future T.12.x proposal may admit a \
                 Deterministic_Model_Identification_Proxy canonical only if the \
                 model-order-search seed, identification algorithm (closed-form / \
                 pinned-fixture), fit-data anchor (pinned-fixture-hash), tie-break \
                 law, and numeric mode are all brutally explicit.",
        },
    ]
}

/// Zero alias claims.
fn timeseries_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Nineteen dedup-court decisions on the time-series batch.
fn timeseries_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(AR_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "AR residual anomaly: AR(p) one-step-ahead prediction-error \
                 detector. Declared model + decision law: AR order p, fit law \
                 (closed-form Yule-Walker / Burg / fixed coefficients), residual \
                 definition, threshold, sampling law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(ARIMA_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "ARIMA residual anomaly: ARIMA(p, d, q) prediction-error \
                 detector. Declared model + decision law: ARIMA order (p, d, q), \
                 fit law, residual definition, forecast horizon, threshold, \
                 sampling law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(STL_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "STL residual anomaly: Cleveland 1990 LOESS-based seasonal-trend \
                 decomposition residual. Declared model + decision law: seasonality \
                 period, LOESS smoother bandwidth, decomposition-method iteration \
                 count, residual definition (observed minus trend plus seasonal), \
                 threshold, sampling law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            reason: "Lag-correlation break (multi-lag): autocorrelation-matrix \
                 break across a declared lag range. Structurally distinct from the \
                 single-lag autocorrelation-coefficient break at SEED id 40. \
                 Declared decision law: lag range, autocorrelation convention, \
                 normalization, threshold, window.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(VARIANCE_RATIO_RESERVED_CANONICAL_ID),
            reason: "Variance-ratio shift (Lo & MacKinlay 1988): q-lag variance- \
                 ratio random-walk test. Declared decision law: window-pair law \
                 (short window + long window), ratio definition (homoskedasticity- \
                 corrected vs heteroskedasticity-robust), threshold, sampling law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(RUN_LENGTH_RESERVED_CANONICAL_ID),
            reason: "Run-length anomaly: consecutive-event-run length detector. \
                 Structurally distinct from Error burst (SEED id 41) which counts \
                 events in a sliding window. Declared decision law: event \
                 definition, run-length law (max / current / percentile), \
                 threshold, sampling law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "Observer residual: general state-estimator residual y_measured \
                 - y_estimated. Most general control-residual primitive. Declared \
                 model + decision law: plant or observer contract (state model + \
                 measurement model + observer gain law), residual definition, \
                 envelope law, threshold, sampling law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "Parity-space residual (Chow & Willsky 1984): algebraic- \
                 redundancy parity-equation residual. Structurally distinct from \
                 observer residual (state-estimator vs algebraic). Declared model \
                 + decision law: plant or observer contract (parity equations W \
                 matrix), residual definition, envelope law, threshold, sampling \
                 law.",
        },
        // -- 6 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22. \
                 Recognised in T.12.e under SignalProcessing; recognised again \
                 here under TimeSeriesStructure as the fundamental residual- \
                 witness primitive applicable to both time-series and control- \
                 residual contexts. Declared model + decision law: residual \
                 envelope, threshold, sampling law. No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SENSOR_BIAS_SEED_ID),
            reason: "Sensor bias detector stays canonical at SEED id 23. \
                 Cross-class adjacency: ControlResiduals source class. Declared \
                 plant or observer contract: residual sustained-offset detection \
                 against a declared baseline / sliding median, threshold, sampling \
                 law. No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ACTUATOR_STICTION_SEED_ID),
            reason: "Actuator stiction detector stays canonical at SEED id 24. \
                 Cross-class adjacency: ControlResiduals source class. Declared \
                 plant or observer contract: input / output signal relation, \
                 stick-slip oscillation signature, sampling law, minimum cycle \
                 support, confuser notes. No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(VALVE_HUNTING_SEED_ID),
            reason: "Valve hunting (control-loop oscillation) detector stays \
                 canonical at SEED id 25. Cross-class adjacency: ControlResiduals \
                 source class. Declared plant or observer contract: control-loop \
                 limit-cycle oscillation signature, sampling law, minimum cycle \
                 support, confuser notes. No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            reason: "Autocorrelation-coefficient break stays canonical at SEED id \
                 40. Recognised in T.12.e under SignalProcessing; recognised again \
                 here under TimeSeriesStructure. Declared decision law: lag, \
                 normalization, window. Single-lag scope; the multi-lag form is \
                 admitted as a new canonical at 5604 (Lag-correlation break).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            reason: "Error burst stays canonical at SEED id 41. Window-count event \
                 detector. Burstiness-index aliases collapse to ParameterizationOf \
                 (record 5611 below).",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the fundamental \
                 residual-witness primitive and the shared ancestor recognised \
                 by the TimeSeriesStructure source class for both time-series \
                 residual detectors (AR / ARIMA / STL) and control-residual \
                 detectors (observer / parity-space / innovation). The court \
                 records the domain transfer without re-canonicalising residual \
                 envelope exit.",
        },
        // -- 3 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID),
            reason: "Innovation sequence is ParameterizationOf(Observer residual, \
                 5607). Kalman-filter-specific parameterization: state-noise \
                 covariance Q + measurement-noise covariance R + filter gain K + \
                 innovation definition + whitening test. The family-level decision \
                 functional is observer residual's. The court declines to admit \
                 innovation sequence as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID),
            reason: "Periodicity break is ParameterizationOf(Lag-correlation break, \
                 5604). Adds a declared peak-selection law over the candidate-lag \
                 range; the family-level decision functional is the lag-correlation \
                 break's. The court declines to admit periodicity break as a new \
                 canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID),
            reason: "Burstiness index is ParameterizationOf(Error burst, SEED id \
                 41). Same window-count decision functional as Error burst; \
                 parameterization generalises the event type. The court declines \
                 to admit burstiness index as a new canonical primitive.",
        },
        // -- 1 RejectedNotDeterministic record ---------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID),
            reason: "Unidentified-model anomaly (ARIMA with auto-determined order \
                 via random search; Kalman filter without declared Q / R \
                 covariances; STL with adaptive seasonality; observer with \
                 fit-during-deployment) is randomized / unidentified in origin: \
                 model parameters change per invocation. Rejected as a literature- \
                 original canonical primitive for this deterministic corpus unless \
                 reduced to a declared deterministic proxy (model-order-search \
                 seed, identification algorithm closed-form OR pinned-fixture-hash, \
                 fit-data anchor, tie-break law, and numeric mode all brutally \
                 explicit) in a later T.12.x proposal. Deterministic stance: the \
                 rejection is on the random / adaptive parameter search alone; the \
                 underlying residual / prediction-error functional is deterministic \
                 given a fixed model identification.",
        },
    ]
}

/// Ten genealogy edges proposed for the post-freeze graph.
fn timeseries_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(AR_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(ARIMA_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AR_RESIDUAL_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(STL_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(VARIANCE_RATIO_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RUN_LENGTH_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Eight source refs supporting the time-series expansion.
fn timeseries_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "yule_ar_1927",
            title: "On a Method of Investigating Periodicities in Disturbed Series",
            year: 1927,
            venue: "Philosophical Transactions of the Royal Society A 226 (AR model origin)",
        },
        ProposedSourceRef {
            citation_key: "box_jenkins_arima_1970",
            title: "Time Series Analysis: Forecasting and Control",
            year: 1970,
            venue: "Holden-Day (ARIMA(p, d, q) framework)",
        },
        ProposedSourceRef {
            citation_key: "cleveland_stl_1990",
            title: "STL: A Seasonal-Trend Decomposition Procedure Based on Loess",
            year: 1990,
            venue: "Journal of Official Statistics 6(1)",
        },
        ProposedSourceRef {
            citation_key: "lo_mackinlay_variance_ratio_1988",
            title: "Stock Market Prices Do Not Follow Random Walks: Evidence from a Simple Specification Test",
            year: 1988,
            venue: "Review of Financial Studies 1(1) (variance-ratio test)",
        },
        ProposedSourceRef {
            citation_key: "kalman_innovation_1960",
            title: "A New Approach to Linear Filtering and Prediction Problems",
            year: 1960,
            venue: "Journal of Basic Engineering 82(1) (Kalman innovation sequence)",
        },
        ProposedSourceRef {
            citation_key: "luenberger_observer_1971",
            title: "An Introduction to Observers",
            year: 1971,
            venue: "IEEE Transactions on Automatic Control 16(6) (Luenberger observer)",
        },
        ProposedSourceRef {
            citation_key: "chow_willsky_parity_space_1984",
            title: "Analytical Redundancy and the Design of Robust Failure Detection Systems",
            year: 1984,
            venue: "IEEE Transactions on Automatic Control 29(7) (parity-space residual)",
        },
        ProposedSourceRef {
            citation_key: "isermann_fdd_textbook_2006",
            title: "Fault-Diagnosis Systems: An Introduction from Fault Detection to Fault Tolerance",
            year: 2006,
            venue: "Springer (observer / parity-space / signature-based FDD textbook)",
        },
    ]
}

/// Build the time-series `DedupCourtDelta`.
fn build_timeseries_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_f_timeseries_delta",
        vec![
            DetectorCanonicalId(AR_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(ARIMA_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(STL_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(VARIANCE_RATIO_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(RUN_LENGTH_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID),
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

/// Build the T.12.f time-series `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_f_timeseries_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_f_timeseries_first_proposal",
        "T.12.f files the Time-Series Structure / Control Residuals amendment \
         proposal. Adds eight genuinely new canonical primitives (AR residual \
         anomaly, ARIMA residual anomaly, STL residual anomaly, lag-correlation \
         break, variance-ratio shift, run-length anomaly, observer residual, \
         parity-space residual) at reserved canonical ids 5601..=5608 with \
         declared model-and-decision-law contracts (model order + fit law + \
         residual definition + threshold + sampling law / window-pair / lag-range \
         / parity equations / state model + observer gain). Records six \
         ExistingCanonicalAuthorityResolution decisions keeping residual \
         envelope exit (SEED id 22), sensor bias detector (id 23), actuator \
         stiction detector (id 24), valve hunting detector (id 25), \
         autocorrelation-coefficient break (id 40), Error burst (id 41) \
         canonical without duplication. Records one DomainTransferOf decision \
         naming residual envelope exit as the shared residual-witness ancestor \
         for the TimeSeriesStructure source class (recognised by both time- \
         series-residual AND control-residual sub-families). Records three \
         ParameterizationOf decisions: innovation sequence anomaly is \
         ParameterizationOf(Observer residual); periodicity break is \
         ParameterizationOf(Lag-correlation break); burstiness index is \
         ParameterizationOf(Error burst). Rejects unidentified-model anomaly \
         as RejectedNotDeterministic at reserved id 5612 - any 'ARIMA with \
         auto-determined order', 'Kalman without declared Q / R', or 'STL with \
         adaptive seasonality' is randomized / unidentified in origin and is \
         admitted neither to SEED nor to new_canonical_records unless a future \
         T.12.x proposal admits a Deterministic_Model_Identification_Proxy \
         with the model-order-search seed, identification algorithm, fit-data \
         anchor, tie-break law, and numeric mode declared. Every record's \
         reason text declares its specific model-and-decision-law contract - \
         the panel-locked warning was 'a model is not a detector until the \
         residual and decision law are declared'. Does NOT mutate SEED \
         (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::TimeSeriesStructure,
        build_timeseries_expansion_batch(),
        build_timeseries_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_f_timeseries",
    )
}
