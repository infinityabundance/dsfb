//! T.12.n — Econometrics + Reliability / Survival: the
//! fourteenth real literature expansion proposal filed through
//! the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.n files the Econometrics + Reliability / Survival
//! > amendment proposal. It admits only deterministic
//! > econometric, reliability, survival, and degradation
//! > witnesses whose stationarity contract, window contract,
//! > regression / hazard model, censoring law, time-origin
//! > law, residual definition, decision functional, confuser
//! > profile, and numeric mode are declared; resolves SEED
//! > collisions with CUSUM / Page-Hinkley / Mann-Kendall /
//! > Residual envelope exit; classifies variants as
//! > parameterizations or domain transfers; rejects learned
//! > market predictors / black-box financial forecasters and
//! > learned RUL classifiers / black-box predictive-maintenance
//! > scores; and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"An econometric or
//! reliability / survival witness is admissible only when the
//! stationarity contract, window contract, regression / hazard
//! model, censoring law, time-origin law, residual definition,
//! decision functional, confuser profile, and numeric mode are
//! declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.n admits deterministic econometric, reliability,
//! > survival, and degradation witnesses. It does not admit
//! > market prediction, investment advice, credit-decision
//! > authority, actuarial pricing authority, causal economic
//! > certainty, RUL certainty, maintenance recommendations,
//! > or failure-time prediction.
//!
//! ## Combined-campaign rationale
//!
//! The campaign combines `Econometrics` and `ReliabilitySurvival`
//! into a single proposal because the two domains share
//! structural-break / CUSUM / envelope-residual ancestry: SEED
//! 3 CUSUM, SEED 4 Page-Hinkley, SEED 11 Mann-Kendall, and
//! SEED 22 Residual envelope exit serve as common ancestors
//! for both econometric structural-break detection and
//! reliability / survival hazard / failure-rate envelope
//! exit detection. Filing them under one proposal keeps the
//! cross-class dedup discipline tight and avoids re-resolving
//! the same SEED collisions twice. The proposal's
//! `target_source_class` is `Econometrics` (per the canonical
//! T.12 sequence); the reliability / survival primitives are
//! admitted in the same body and tagged via display names.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.n's design began with a grep of [`crate::seed::SEED`]
//! for every econometric / reliability / survival candidate.
//! The walk found **four** T.12.n-relevant primitives already
//! canonical:
//!
//! * **CUSUM (cumulative sum) chart** at SEED id 3 — the
//!   shared structural-change ancestor (econometric CUSUM-of-
//!   recursive-residuals per Brown-Durbin-Evans 1975 reduces
//!   to a CUSUM parameterization; reliability cumulative-
//!   damage residual per Palmgren-Miner 1924/1945 is a
//!   CUSUM parameterization).
//! * **Page-Hinkley test** at SEED id 4 — econometric
//!   sequential change detection; structural-break F-test
//!   parameterization (Quandt-Andrews / Chow) collapses here.
//! * **Mann-Kendall trend test** at SEED id 11 — econometric
//!   trend detection; serves as the shared trend ancestor
//!   for reliability degradation trends.
//! * **Residual envelope exit** at SEED id 22 — shared
//!   envelope-boundary ancestor for reliability failure-rate
//!   envelope exit + hazard-rate-change detection.
//!
//! All four become `ExistingCanonicalAuthorityResolution`
//! records. **Panel-locked success-shape** (mirroring T.12.k /
//! T.12.l / T.12.m): the campaign's strength comes from
//! cross-class dedup discipline (4 authority resolutions over
//! the structural-change + envelope SEED family that
//! econometric / reliability / survival methods heavily
//! reuse), not detector count.
//!
//! Eight genuinely new canonicals at reserved ids 6401..=6408
//! survived the SEED-walk as structurally distinct decision
//! functionals:
//!
//! * **GARCH volatility residual anomaly witness** (6401;
//!   Bollerslev 1986) — declared GARCH(p,q) model order,
//!   estimation procedure (MLE with fixed-seed convergence
//!   tolerances), standardised-residual computation, decision
//!   law (per-window standardised-residual exceeds threshold
//!   relative to nominal conditional variance). Distinct from
//!   any SEED because the residual is against a CONDITIONAL
//!   VARIANCE model, not a level model or a raw signal
//!   baseline. Econometric signal witness, not market
//!   prediction.
//! * **Cointegration-break detector** (6402; Hansen 1992;
//!   Quintos & Phillips 1993) — declared cointegrating-vector
//!   estimation procedure, cointegration-regression residual,
//!   CUSUM-of-squared-residuals law against fixed nominal
//!   bound. Distinct from SEED 3 CUSUM because the residual
//!   source is a cointegration regression, not the raw signal,
//!   and the decision functional is on SQUARED residuals.
//! * **Hausman-test residual witness** (6403; Hausman 1978) —
//!   declared two-estimator framework (e.g., fixed-effects
//!   vs random-effects, OLS vs IV), parameter-difference
//!   vector, asymptotic-covariance estimation, chi-squared
//!   test law. Distinct from any SEED because the decision
//!   functional is on a PARAMETER-DIFFERENCE chi-squared
//!   statistic, not on a residual sequence.
//! * **Bai-Perron multiple-break detector** (6404; Bai-Perron
//!   1998 / 2003) — declared maximum-break-count + minimum-
//!   regime-length + information-criterion (BIC / LWZ) +
//!   F-statistic-based break-test law (Quandt-Andrews
//!   supremum F). Distinct from SEED 4 Page-Hinkley
//!   (sequential single-shift) and T.12.b binary segmentation
//!   (cost-minimization without F-statistic) because it
//!   admits MULTIPLE structural breaks under an information-
//!   criterion-selected count.
//! * **Kaplan-Meier survival-residual witness** (6405;
//!   Kaplan & Meier 1958 product-limit estimator) — declared
//!   time-origin law, censoring contract (right-censoring /
//!   left-truncation / interval-censoring; censoring
//!   independence assumption), product-limit construction,
//!   per-window residual against expected KM curve, decision
//!   law (per-window deviation exceeds threshold). Survival
//!   signal witness, not failure-time prediction.
//! * **Cox proportional-hazards / Schoenfeld residual witness**
//!   (6406; Cox 1972 proportional hazards regression;
//!   Schoenfeld 1982 Schoenfeld residual) — declared covariate
//!   set, proportional-hazards assumption, Schoenfeld
//!   residual computation per covariate and per event time,
//!   decision law (per-covariate Schoenfeld residual departs
//!   from zero correlation with time beyond threshold).
//!   Survival signal witness; the court does NOT issue
//!   failure-time prediction or RUL certainty.
//! * **Weibull failure-rate envelope exit witness** (6407;
//!   Weibull 1951 statistical distribution) — declared
//!   Weibull shape and scale parameters, MLE estimation with
//!   fixed-seed convergence tolerances, parametric failure-
//!   rate envelope, decision law (observed failure-rate
//!   departs from declared envelope beyond threshold).
//!   Reliability signal witness, not RUL certainty or
//!   warranty-extension recommendation.
//! * **Crack-growth law residual witness** (6408; Paris &
//!   Erdogan 1963 crack-growth law) — declared stress-
//!   intensity-range model, Paris-Erdogan parameters (C and
//!   m), per-cycle expected crack-increment, residual against
//!   measured crack-growth, decision law (per-window crack-
//!   growth residual exceeds tolerance). Reliability /
//!   degradation signal witness, not failure-time prediction.
//!
//! Two domain transfers (panel-locked):
//!
//! * **CUSUM (cumulative sum) chart** (SEED 3) →
//!   `DomainTransferOf` as the shared structural-change
//!   ancestor for `Econometrics` AND `ReliabilitySurvival`
//!   (CUSUM-of-recursive-residuals 6409 and cumulative-damage
//!   residual 6412 are descendants).
//! * **Residual envelope exit** (SEED 22) → `DomainTransferOf`
//!   as the shared envelope-boundary ancestor for
//!   `ReliabilitySurvival` (hazard-rate-change 6411 is the
//!   reliability descendant; Weibull failure-rate envelope
//!   exit 6407 inherits the envelope-exit semantic).
//!
//! Four parameterizations (panel-candidate canonicals that
//! collapsed on closer inspection):
//!
//! * **CUSUM-of-recursive-residuals** (6409; Brown / Durbin /
//!   Evans 1975 CUSUM-of-recursive-residuals structural-
//!   stability test) → `ParameterizationOf(CUSUM, SEED 3)`
//!   with declared OLS recursive-residual computation +
//!   normalised CUSUM-of-recursive-residuals path + boundary-
//!   crossing decision law.
//! * **Quandt-Andrews / Chow structural-break F-test**
//!   (6410; Quandt 1960 / Chow 1960 / Andrews 1993) →
//!   `ParameterizationOf(Page-Hinkley, SEED 4)` with declared
//!   structural-break F-statistic law + supremum / average /
//!   exponential aggregation rule over candidate break dates.
//! * **Hazard-rate change** (6411) → `ParameterizationOf
//!   (Residual envelope exit, SEED 22)` with declared
//!   piecewise-constant hazard model + hazard-rate envelope
//!   bounds + envelope-exit decision law.
//! * **Cumulative damage residual** (6412; Palmgren 1924 /
//!   Miner 1945 linear cumulative damage rule) →
//!   `ParameterizationOf(CUSUM, SEED 3)` with declared
//!   per-cycle damage-increment law + cumulative-damage
//!   decision threshold.
//!
//! Two rejections (eighth T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, h,
//! i, j, k, l, m):
//!
//! * **Learned market predictor / black-box financial
//!   forecaster** (6413) — `RejectedNotDeterministic`.
//!   Learned market prediction pipelines (Bloomberg AIM
//!   alpha capture, AlphaSense AI signals, Kavout K Score,
//!   Goldman Sachs SecDB ML, JP Morgan COIN / LOXM) expose
//!   market-prediction / investment-recommendation scores
//!   from opaque learned embeddings. Admission requires a
//!   future T.12.x to admit a
//!   `Deterministic_Market_Forecast_Proxy` canonical with
//!   deterministic feature-extraction law, declared formula,
//!   declared training-data anchor, feature schema, tie-break,
//!   numeric mode, no market-prediction claim, no investment-
//!   recommendation claim, and no credit-decision claim. The
//!   court does NOT issue market prediction, investment
//!   advice, credit-decision authority, or actuarial-pricing
//!   authority.
//! * **Learned RUL classifier / black-box predictive-
//!   maintenance score** (6414) — `RejectedNotDeterministic`.
//!   Learned RUL prediction pipelines (Uptake AI, C3.ai
//!   predictive maintenance, Senseye Predictive Maintenance,
//!   IBM Maximo RUL, Siemens MindSphere Asset Analytics)
//!   expose remaining-useful-life and failure-time
//!   prediction scores from learned embeddings. Admission
//!   requires a future T.12.x to admit a
//!   `Deterministic_RUL_Proxy` canonical with deterministic
//!   feature-extraction law + declared formula + declared
//!   training-data anchor + feature schema + tie-break +
//!   numeric mode + NO RUL-certainty claim + NO failure-
//!   time-prediction claim + NO maintenance-recommendation
//!   claim. The court does NOT issue RUL certainty, failure-
//!   time prediction, or maintenance recommendations.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×8 (6401..=6408).
//! * `ExistingCanonicalAuthorityResolution` ×4 — SEED 3, 4,
//!   11, 22.
//! * `DomainTransferOf` ×2 — SEED 3 (structural-change
//!   ancestor) + SEED 22 (envelope-boundary ancestor).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 8 + 4 + 2 + 4 + 2 = **20 dedup-court records**.
//!
//! ## Market-prediction / credit-decision / RUL-certainty discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND
//! ExistingCanonicalAuthorityResolution reason text MUST
//! describe its record as an "econometric signal witness" /
//! "structural-break signal witness" / "survival signal
//! witness" / "reliability signal witness" / "degradation
//! signal witness" — NEVER as a market-prediction, investment-
//! advice, credit-decision, actuarial-pricing, RUL-certainty,
//! failure-time-prediction, or maintenance-recommendation
//! claim. The dedicated load-bearing negatives scan every such
//! reason for forbidden terms across three parametric
//! scanners:
//!
//! - market-prediction terms (stock price prediction, market
//!   return forecast, investment recommendation, trading
//!   signal, buy signal, sell signal, alpha generation);
//! - investment / credit-decision terms (credit approval
//!   verdict, credit denial verdict, loan approval verdict,
//!   investment advice verdict, fiduciary recommendation,
//!   actuarial pricing authority);
//! - RUL / failure-time terms (rul prediction, remaining
//!   useful life certainty, failure time prediction,
//!   predicted failure date, guaranteed lifetime, warranty
//!   extension recommendation, maintenance recommendation).
//!
//! Forbidden terms appear ONLY in `RejectedNotDeterministic`
//! reason text (where they describe what is NOT admitted).
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.n econometrics + reliability
//!   / survival `corpus_amendment_proposal_hash_v1` distinct
//!   from every prior T.12.x proposal hash.
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
// Reserved id constants (panel-locked, 6401..=6414 used;
// 6415..=6499 reserved for future Econometrics + Reliability /
// Survival proposals)
// ---------------------------------------------------------------

/// Reserved canonical id for GARCH volatility residual anomaly
/// witness. Bollerslev 1986 canonical reference. Residual is
/// against a CONDITIONAL VARIANCE model, not a level model.
pub const GARCH_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 6401;

/// Reserved canonical id for Cointegration-break detector.
/// Hansen 1992 / Quintos & Phillips 1993. Residual source is
/// a cointegration regression; decision functional is on
/// SQUARED residuals (distinct from SEED 3 raw CUSUM).
pub const COINTEGRATION_BREAK_RESERVED_CANONICAL_ID: u32 = 6402;

/// Reserved canonical id for Hausman-test residual witness.
/// Hausman 1978 canonical reference. Decision functional is on
/// a PARAMETER-DIFFERENCE chi-squared statistic, not a
/// residual sequence.
pub const HAUSMAN_RESERVED_CANONICAL_ID: u32 = 6403;

/// Reserved canonical id for Bai-Perron multiple-break
/// detector. Bai-Perron 1998 / 2003. Admits MULTIPLE
/// structural breaks under information-criterion-selected
/// count + F-statistic-based break-test law.
pub const BAI_PERRON_RESERVED_CANONICAL_ID: u32 = 6404;

/// Reserved canonical id for Kaplan-Meier survival residual
/// witness. Kaplan & Meier 1958 product-limit estimator.
/// Declared time-origin + censoring contract.
pub const KM_SURVIVAL_RESERVED_CANONICAL_ID: u32 = 6405;

/// Reserved canonical id for Cox proportional-hazards /
/// Schoenfeld residual witness. Cox 1972 PH regression;
/// Schoenfeld 1982 Schoenfeld residual. Declared covariate
/// set + PH assumption + Schoenfeld residual computation.
pub const COX_SCHOENFELD_RESERVED_CANONICAL_ID: u32 = 6406;

/// Reserved canonical id for Weibull failure-rate envelope
/// exit witness. Weibull 1951 statistical distribution.
/// Declared Weibull shape + scale + MLE estimation +
/// parametric failure-rate envelope.
pub const WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID: u32 = 6407;

/// Reserved canonical id for Crack-growth law residual
/// witness. Paris & Erdogan 1963 crack-growth law. Declared
/// stress-intensity-range model + Paris-Erdogan parameters.
pub const PARIS_ERDOGAN_RESERVED_CANONICAL_ID: u32 = 6408;

/// Reserved id for CUSUM-of-recursive-residuals.
/// `ParameterizationOf(CUSUM, SEED 3)`.
pub const CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID: u32 = 6409;

/// Reserved id for Quandt-Andrews / Chow structural-break
/// F-test. `ParameterizationOf(Page-Hinkley, SEED 4)`.
pub const QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID: u32 = 6410;

/// Reserved id for Hazard-rate change. `ParameterizationOf
/// (Residual envelope exit, SEED 22)`.
pub const HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID: u32 = 6411;

/// Reserved id for Cumulative damage residual (Palmgren-Miner
/// linear cumulative damage rule). `ParameterizationOf(CUSUM,
/// SEED 3)`.
pub const CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID: u32 = 6412;

/// Reserved id for Learned market predictor / black-box
/// financial forecaster. `RejectedNotDeterministic`.
pub const LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID: u32 = 6413;

/// Reserved id for Learned RUL classifier / black-box
/// predictive-maintenance score. `RejectedNotDeterministic`.
pub const LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 6414;

// Existing SEED canonical ids referenced by T.12.n.

/// CUSUM (cumulative sum) chart — SEED canonical id 3.
/// Shared structural-change ancestor for Econometrics +
/// ReliabilitySurvival.
pub const CUSUM_SEED_ID: u32 = 3;

/// Page-Hinkley test — SEED canonical id 4. Sequential
/// change-detection ancestor; structural-break F-test
/// parameterizes here.
pub const PAGE_HINKLEY_SEED_ID: u32 = 4;

/// Mann-Kendall trend test — SEED canonical id 11. Trend-
/// detection ancestor for econometric / reliability trend
/// witnesses.
pub const MANN_KENDALL_SEED_ID: u32 = 11;

/// Residual envelope exit — SEED canonical id 22. Shared
/// envelope-boundary ancestor for ReliabilitySurvival
/// failure-rate envelope + hazard-rate-change witnesses.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

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
// Builders for the econometrics + reliability / survival batch
// ---------------------------------------------------------------

/// Build the econometrics + reliability / survival
/// `CorpusExpansionBatch` body.
fn build_econ_reliability_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_n_econometrics_reliability_first_proposal",
        SourceClass::Econometrics,
        econ_reliability_proposed_primitives(),
        econ_reliability_proposed_aliases(),
        econ_reliability_proposed_dedup_records(),
        econ_reliability_proposed_genealogy_edges(),
        econ_reliability_proposed_source_refs(),
    )
}

/// Fourteen proposed primitives: 8 canonical (4 econometric +
/// 4 reliability/survival) + 4 parameterization shells + 2
/// rejection shells. The "tight canonical set, heavy
/// structural-change + envelope authority resolution, clear
/// rejection of learned market predictors and learned RUL
/// classifiers" shape applies the panel-locked T.12.k /
/// T.12.l / T.12.m success posture to combined econometrics +
/// reliability / survival.
fn econ_reliability_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(GARCH_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "GARCH volatility residual anomaly witness",
            motivation: "GARCH volatility residual anomaly econometric signal \
                 witness (Bollerslev 1986). Required contract: stationarity \
                 contract (declared GARCH(p,q) stationarity / co-stationarity \
                 conditions), window contract (declared rolling window length + \
                 minimum sample size), regression / hazard model (declared \
                 GARCH(p,q) order + MLE estimation procedure with fixed-seed \
                 convergence tolerances + fixed iteration cap), residual \
                 definition (per-window standardised residual = observed \
                 squared-return divided by GARCH conditional-variance estimate), \
                 decision functional (per-window standardised residual exceeds \
                 declared threshold relative to nominal conditional variance), \
                 confuser profile (regime-switching volatility, outlier \
                 contamination, structural break in mean), numeric mode. \
                 Structurally distinct from any SEED because the residual is \
                 against a CONDITIONAL VARIANCE model, not a level model or a \
                 raw signal baseline. Econometric signal witness, not market \
                 prediction or investment recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COINTEGRATION_BREAK_RESERVED_CANONICAL_ID),
            display_name: "Cointegration-break detector",
            motivation: "Cointegration-break econometric signal witness (Hansen \
                 1992; Quintos & Phillips 1993). Required contract: stationarity \
                 contract (declared series-pair I(1) integration order + Engle-\
                 Granger or Johansen cointegration estimation), window contract, \
                 regression / hazard model (declared cointegrating-vector \
                 estimation procedure with fixed-seed initial conditions), \
                 residual definition (cointegration-regression residual per \
                 observation), decision functional (CUSUM-of-squared-residuals \
                 of the cointegration regression exceeds Hansen 1992 fixed \
                 nominal bound), confuser profile (changing cointegrating \
                 rank, structural break in long-run relation, near-unit-root \
                 false signal), numeric mode. Structurally distinct from SEED 3 \
                 CUSUM because the residual source is a cointegration \
                 regression, not a raw signal, and the decision functional is \
                 on SQUARED residuals. Econometric signal witness, not market \
                 prediction or investment recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HAUSMAN_RESERVED_CANONICAL_ID),
            display_name: "Hausman-test residual witness",
            motivation: "Hausman-test residual econometric signal witness \
                 (Hausman 1978). Required contract: stationarity contract \
                 (declared two-estimator framework e.g. fixed-effects vs \
                 random-effects or OLS vs IV; consistency-under-null \
                 assumption), window contract (declared sample size + degrees-\
                 of-freedom budget), regression / hazard model (declared \
                 estimator pair + estimation procedure with fixed-seed initial \
                 conditions), residual definition (parameter-difference vector \
                 = estimator_A - estimator_B), decision functional (chi-squared \
                 test law on parameter-difference vector weighted by inverse \
                 of declared asymptotic-covariance-matrix difference exceeds \
                 chi-squared quantile threshold), confuser profile (weak \
                 instruments, small-sample bias, near-singular covariance), \
                 numeric mode. Structurally distinct from any SEED because the \
                 decision functional is on a PARAMETER-DIFFERENCE chi-squared \
                 statistic, not on a residual sequence. Econometric signal \
                 witness, not credit-decision authority.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BAI_PERRON_RESERVED_CANONICAL_ID),
            display_name: "Bai-Perron multiple-break detector",
            motivation: "Bai-Perron multiple-structural-break econometric \
                 signal witness (Bai-Perron 1998 / 2003). Required contract: \
                 stationarity contract (declared piecewise-stationarity within \
                 regimes), window contract (declared minimum-regime-length \
                 epsilon trimming + maximum-break-count M), regression / hazard \
                 model (declared base regression model whose coefficients shift \
                 across regimes), residual definition (per-regime residual sum-\
                 of-squares), decision functional (information-criterion \
                 (BIC / LWZ) over candidate break counts AND Quandt-Andrews \
                 supremum-F structural-break test per candidate break date), \
                 confuser profile (gradual drift mistaken for break, \
                 heteroscedasticity, autocorrelated errors), numeric mode. \
                 Structurally distinct from SEED 4 Page-Hinkley (sequential \
                 single-shift detection) and from T.12.b binary segmentation \
                 (cost-minimization without F-statistic) because Bai-Perron \
                 admits MULTIPLE breaks under an information-criterion-selected \
                 count. Econometric structural-break signal witness, not market \
                 prediction.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(KM_SURVIVAL_RESERVED_CANONICAL_ID),
            display_name: "Kaplan-Meier survival-residual witness",
            motivation: "Kaplan-Meier survival-residual signal witness (Kaplan \
                 & Meier 1958 product-limit estimator). Required contract: \
                 stationarity contract (declared exchangeability assumption \
                 across survival cohort), window contract (declared cohort + \
                 observation period), regression / hazard model (declared \
                 product-limit construction of the KM survival curve), \
                 censoring law (right-censoring / left-truncation / interval-\
                 censoring contract; censoring independence assumption), time-\
                 origin law (declared event-time-zero anchor — birth, \
                 enrollment, surgery, equipment-commissioning), residual \
                 definition (per-window survival-residual = observed empirical \
                 survival minus baseline KM curve), decision functional (per-\
                 window survival-residual exceeds declared threshold relative \
                 to nominal KM curve), confuser profile (informative \
                 censoring, cohort heterogeneity, late entry), numeric mode. \
                 Survival signal witness, not failure-time prediction or RUL \
                 certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COX_SCHOENFELD_RESERVED_CANONICAL_ID),
            display_name: "Cox proportional-hazards / Schoenfeld residual witness",
            motivation: "Cox proportional-hazards / Schoenfeld residual signal \
                 witness (Cox 1972 proportional hazards regression; Schoenfeld \
                 1982 Schoenfeld residual). Required contract: stationarity \
                 contract (declared proportional-hazards assumption), window \
                 contract (declared event-time grid + cohort), regression / \
                 hazard model (declared covariate set + partial-likelihood \
                 estimation procedure with fixed-seed convergence), censoring \
                 law (right-censoring contract; censoring independence), time-\
                 origin law (declared event-time-zero anchor), residual \
                 definition (per-covariate per-event-time Schoenfeld residual \
                 = observed covariate minus expected covariate under partial-\
                 likelihood weights), decision functional (per-covariate \
                 Schoenfeld residual departs from zero correlation with time \
                 beyond threshold — Grambsch-Therneau 1994 test law), \
                 confuser profile (time-varying covariate misspecification, \
                 informative censoring), numeric mode. Survival signal \
                 witness; the court does NOT issue failure-time prediction or \
                 RUL certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID),
            display_name: "Weibull failure-rate envelope exit witness",
            motivation: "Weibull failure-rate envelope exit reliability signal \
                 witness (Weibull 1951 statistical distribution). Required \
                 contract: stationarity contract (declared population \
                 stationarity within Weibull regime), window contract \
                 (declared cohort + observation interval), regression / hazard \
                 model (declared Weibull shape and scale parameters with MLE \
                 estimation procedure + fixed-seed convergence tolerances + \
                 fixed iteration cap), censoring law (right-censoring contract \
                 for life-test observations), time-origin law (declared \
                 component-commissioning anchor), residual definition (observed \
                 per-window failure rate minus expected Weibull failure rate), \
                 decision functional (observed failure-rate residual exits \
                 parametric envelope bounds derived from declared Weibull \
                 confidence interval), confuser profile (cohort heterogeneity, \
                 mixed Weibull populations, infant mortality vs wearout regime \
                 confusion), numeric mode. Reliability signal witness, not RUL \
                 certainty or warranty-extension recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PARIS_ERDOGAN_RESERVED_CANONICAL_ID),
            display_name: "Crack-growth law residual witness",
            motivation: "Crack-growth law residual reliability / degradation \
                 signal witness (Paris & Erdogan 1963 crack-growth law). \
                 Required contract: stationarity contract (declared loading \
                 spectrum stationarity), window contract (declared load-cycle \
                 grid), regression / hazard model (declared Paris-Erdogan \
                 parameters C and m + stress-intensity-range model + crack-\
                 length measurement schedule), residual definition (per-cycle \
                 expected crack-increment da/dN = C * (delta_K)^m minus \
                 measured per-cycle increment), decision functional (per-window \
                 crack-growth residual exceeds tolerance derived from \
                 measurement-uncertainty envelope), confuser profile (variable-\
                 amplitude loading, environment effects, multi-mode crack \
                 propagation), numeric mode. Reliability / degradation signal \
                 witness, not failure-time prediction or maintenance \
                 recommendation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "CUSUM-of-recursive-residuals - parameterization shell",
            motivation: "Recursive-residual parameterization of CUSUM (SEED id \
                 3) with declared OLS recursive-residual computation per \
                 observation + normalised CUSUM-of-recursive-residuals path + \
                 boundary-crossing decision law (Brown / Durbin / Evans 1975 \
                 structural-stability test). The court rules: CUSUM-of-\
                 recursive-residuals is ParameterizationOf(CUSUM, SEED 3), NOT \
                 a new canonical primitive. Appears in proposed_primitives but \
                 NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID),
            display_name: "Quandt-Andrews / Chow structural-break F-test - parameterization shell",
            motivation: "F-statistic parameterization of Page-Hinkley (SEED id \
                 4) with declared structural-break F-statistic law (Quandt \
                 1960 likelihood-ratio for break; Chow 1960 F-test for break; \
                 Andrews 1993 supremum-F over candidate dates) + supremum / \
                 average / exponential aggregation rule over candidate break \
                 dates + asymptotic critical-value table. The court rules: \
                 Quandt-Andrews / Chow structural-break F-test is \
                 ParameterizationOf(Page-Hinkley, SEED 4), NOT a new canonical \
                 primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID),
            display_name: "Hazard-rate change - parameterization shell",
            motivation: "Hazard-rate-envelope parameterization of Residual \
                 envelope exit (SEED id 22) with declared piecewise-constant \
                 hazard model + hazard-rate envelope bounds + envelope-exit \
                 decision law. The court rules: hazard-rate change is \
                 ParameterizationOf(Residual envelope exit, SEED 22), NOT a \
                 new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID),
            display_name: "Cumulative damage residual - parameterization shell",
            motivation: "Cumulative-damage parameterization of CUSUM (SEED id \
                 3) with declared per-cycle damage-increment law (Palmgren \
                 1924 / Miner 1945 linear cumulative damage rule) + \
                 cumulative-damage decision threshold + S-N curve reference. \
                 The court rules: cumulative damage residual is \
                 ParameterizationOf(CUSUM, SEED 3), NOT a new canonical \
                 primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID,
            ),
            display_name:
                "Learned market predictor / black-box financial forecaster - rejected shell",
            motivation: "Learned market-prediction pipelines (Bloomberg AIM \
                 alpha capture, AlphaSense AI signals, Kavout K Score, \
                 Goldman Sachs SecDB ML, JP Morgan COIN / LOXM execution AI) \
                 expose market-prediction / investment-recommendation scores \
                 from opaque learned embeddings without a deterministic \
                 feature-extraction law, declared formula, declared training-\
                 data anchor, declared tie-break law, or declared numeric \
                 mode. The court does NOT admit these to the dedup-court \
                 delta's new_canonical_records. A future T.12.x may admit a \
                 Deterministic_Market_Forecast_Proxy canonical only if a \
                 deterministic feature-extraction law, declared formula, \
                 declared training-data anchor, feature schema, tie-break, \
                 numeric mode, and NO market-prediction claim AND NO \
                 investment-recommendation claim AND NO credit-decision \
                 claim AND NO actuarial-pricing claim are all brutally \
                 explicit. The court does NOT issue market prediction, \
                 investment advice, credit-decision authority, or actuarial-\
                 pricing authority; those terms appear here only to describe \
                 what is NOT admitted.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name:
                "Learned RUL classifier / black-box predictive-maintenance score - rejected shell",
            motivation: "Learned remaining-useful-life prediction pipelines \
                 (Uptake AI, C3.ai predictive maintenance, Senseye Predictive \
                 Maintenance, IBM Maximo RUL, Siemens MindSphere Asset \
                 Analytics) expose RUL and failure-time prediction scores \
                 from learned embeddings without a deterministic feature-\
                 extraction law, declared formula, declared training-data \
                 anchor, declared tie-break law, or declared numeric mode. \
                 The court does NOT admit these to new_canonical_records. A \
                 future T.12.x may admit a Deterministic_RUL_Proxy canonical \
                 only if a deterministic feature-extraction law + declared \
                 formula + declared training-data anchor + feature schema + \
                 tie-break + numeric mode + NO RUL-certainty claim AND NO \
                 failure-time-prediction claim AND NO maintenance-recommendation \
                 claim are all brutally explicit. The court does NOT issue \
                 RUL certainty, failure-time prediction, or maintenance \
                 recommendations; those terms appear here only to describe \
                 what is NOT admitted.",
        },
    ]
}

/// Zero alias claims (T.12.n routes everything through dedup
/// records and existing-canonical authority resolutions).
fn econ_reliability_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty dedup-court decisions on the econometrics +
/// reliability / survival batch.
fn econ_reliability_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(GARCH_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "GARCH volatility residual anomaly witness (Bollerslev \
                 1986): declared stationarity contract + window contract + \
                 regression model (GARCH(p,q) order + MLE estimation with \
                 fixed-seed convergence) + residual definition (per-window \
                 standardised residual) + decision functional (per-window \
                 standardised residual exceeds threshold relative to nominal \
                 conditional variance) + confuser profile + numeric mode. \
                 Structurally distinct from any SEED because the residual is \
                 against a CONDITIONAL VARIANCE model, not a level model. \
                 Econometric signal witness, not market prediction or \
                 investment recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(COINTEGRATION_BREAK_RESERVED_CANONICAL_ID),
            reason: "Cointegration-break detector (Hansen 1992; Quintos & \
                 Phillips 1993): declared stationarity contract (series-pair \
                 I(1) integration + Engle-Granger or Johansen cointegration) \
                 + window contract + regression model (cointegrating-vector \
                 estimation with fixed-seed initial conditions) + residual \
                 definition (cointegration-regression residual per \
                 observation) + decision functional (CUSUM-of-squared-\
                 residuals exceeds Hansen 1992 fixed nominal bound) + \
                 confuser profile + numeric mode. Structurally distinct from \
                 SEED 3 CUSUM because the residual source is a cointegration \
                 regression, not a raw signal, and the decision functional \
                 is on SQUARED residuals. Econometric structural-break signal \
                 witness, not market prediction.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(HAUSMAN_RESERVED_CANONICAL_ID),
            reason: "Hausman-test residual witness (Hausman 1978): declared \
                 stationarity contract (declared two-estimator framework + \
                 consistency-under-null assumption) + window contract \
                 (declared sample size + degrees-of-freedom budget) + \
                 regression model (declared estimator pair + estimation \
                 procedure with fixed-seed initial conditions) + residual \
                 definition (parameter-difference vector) + decision \
                 functional (chi-squared test law on parameter-difference \
                 vector weighted by inverse asymptotic-covariance-matrix \
                 difference exceeds chi-squared quantile threshold) + \
                 confuser profile + numeric mode. Structurally distinct from \
                 any SEED because the decision functional is on a PARAMETER-\
                 DIFFERENCE chi-squared statistic, not on a residual \
                 sequence. Econometric signal witness, not credit-decision \
                 authority.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BAI_PERRON_RESERVED_CANONICAL_ID),
            reason: "Bai-Perron multiple-break detector (Bai-Perron 1998 / \
                 2003): declared stationarity contract (piecewise-stationarity \
                 within regimes) + window contract (minimum-regime-length \
                 epsilon trimming + maximum-break-count M) + regression model \
                 (declared base regression whose coefficients shift across \
                 regimes) + residual definition (per-regime residual sum-of-\
                 squares) + decision functional (information criterion (BIC \
                 / LWZ) over candidate break counts AND Quandt-Andrews \
                 supremum-F structural-break test per candidate break date) \
                 + confuser profile + numeric mode. Structurally distinct \
                 from SEED 4 Page-Hinkley because Bai-Perron admits MULTIPLE \
                 breaks under information-criterion-selected count. \
                 Econometric structural-break signal witness, not market \
                 prediction.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(KM_SURVIVAL_RESERVED_CANONICAL_ID),
            reason: "Kaplan-Meier survival-residual witness (Kaplan & Meier \
                 1958): declared stationarity contract (exchangeability \
                 across survival cohort) + window contract (cohort + \
                 observation period) + regression model (product-limit \
                 construction of KM survival curve) + censoring law (right-\
                 censoring / left-truncation / interval-censoring contract; \
                 censoring independence assumption) + time-origin law \
                 (declared event-time-zero anchor — birth, enrollment, \
                 surgery, equipment-commissioning) + residual definition \
                 (per-window survival-residual = observed empirical survival \
                 minus baseline KM curve) + decision functional (per-window \
                 survival-residual exceeds threshold relative to nominal KM \
                 curve) + confuser profile + numeric mode. Survival signal \
                 witness, not failure-time prediction or RUL certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(COX_SCHOENFELD_RESERVED_CANONICAL_ID),
            reason: "Cox proportional-hazards / Schoenfeld residual witness \
                 (Cox 1972; Schoenfeld 1982): declared stationarity contract \
                 (proportional-hazards assumption) + window contract (event-\
                 time grid + cohort) + regression model (declared covariate \
                 set + partial-likelihood estimation with fixed-seed \
                 convergence) + censoring law (right-censoring contract; \
                 censoring independence) + time-origin law (declared event-\
                 time-zero anchor) + residual definition (per-covariate per-\
                 event-time Schoenfeld residual = observed covariate minus \
                 expected covariate under partial-likelihood weights) + \
                 decision functional (per-covariate Schoenfeld residual \
                 departs from zero correlation with time beyond threshold — \
                 Grambsch-Therneau 1994 test law) + confuser profile + \
                 numeric mode. Survival signal witness; the court does NOT \
                 issue failure-time prediction or RUL certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID),
            reason: "Weibull failure-rate envelope exit witness (Weibull \
                 1951): declared stationarity contract (population \
                 stationarity within Weibull regime) + window contract \
                 (cohort + observation interval) + regression model (declared \
                 Weibull shape and scale parameters with MLE estimation + \
                 fixed-seed convergence tolerances + fixed iteration cap) + \
                 censoring law (right-censoring contract for life-test \
                 observations) + time-origin law (declared component-\
                 commissioning anchor) + residual definition (observed per-\
                 window failure rate minus expected Weibull failure rate) + \
                 decision functional (observed failure-rate residual exits \
                 parametric envelope bounds derived from declared Weibull \
                 confidence interval) + confuser profile + numeric mode. \
                 Reliability signal witness, not RUL certainty or warranty-\
                 extension recommendation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PARIS_ERDOGAN_RESERVED_CANONICAL_ID),
            reason: "Crack-growth law residual witness (Paris & Erdogan \
                 1963): declared stationarity contract (loading-spectrum \
                 stationarity) + window contract (load-cycle grid) + \
                 regression model (declared Paris-Erdogan parameters C and m \
                 + stress-intensity-range model + crack-length measurement \
                 schedule) + residual definition (per-cycle expected crack-\
                 increment da/dN = C * (delta_K)^m minus measured per-cycle \
                 increment) + decision functional (per-window crack-growth \
                 residual exceeds tolerance derived from measurement-\
                 uncertainty envelope) + confuser profile + numeric mode. \
                 Reliability / degradation signal witness, not failure-time \
                 prediction or maintenance recommendation.",
        },
        // -- 4 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            reason: "CUSUM (cumulative sum) chart stays canonical at SEED id \
                 3 under Econometrics + ReliabilitySurvival. Econometric \
                 CUSUM-of-recursive-residuals (Brown / Durbin / Evans 1975) \
                 and reliability cumulative-damage residual (Palmgren 1924 / \
                 Miner 1945) BOTH reduce to a CUSUM parameterization. \
                 Declared stationarity contract + window contract + per-\
                 observation residual + cumulative-sum aggregation + decision \
                 law (cumulative sum exceeds upper or lower control limit) + \
                 unit law + numeric mode. No duplicate admitted; CUSUM-of-\
                 recursive-residuals (6409) and cumulative damage residual \
                 (6412) collapse here as ParameterizationOf. Econometric / \
                 reliability structural-change signal witness, not market \
                 prediction or RUL certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PAGE_HINKLEY_SEED_ID),
            reason: "Page-Hinkley test stays canonical at SEED id 4 under \
                 Econometrics + ReliabilitySurvival. Econometric structural-\
                 break F-test (Quandt 1960; Chow 1960; Andrews 1993) \
                 parameterizes here. Declared stationarity contract + window \
                 contract + per-observation deviation + sequential aggregation \
                 + decision law (Page-Hinkley statistic exceeds threshold) + \
                 unit law + numeric mode. No duplicate admitted; Quandt-\
                 Andrews / Chow structural-break F-test (6410) collapses \
                 here as ParameterizationOf. Econometric change-detection \
                 signal witness, not market prediction.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MANN_KENDALL_SEED_ID),
            reason: "Mann-Kendall trend test stays canonical at SEED id 11 \
                 under Econometrics + ReliabilitySurvival. Econometric trend \
                 detection AND reliability degradation trend detection both \
                 reduce to non-parametric trend testing. Declared stationarity \
                 contract (declared trend-test null hypothesis) + window \
                 contract + per-observation rank + Kendall tau aggregation + \
                 decision law (Mann-Kendall S statistic exceeds normal-\
                 approximation quantile) + unit law + numeric mode. No \
                 duplicate admitted. Econometric / reliability trend signal \
                 witness, not market prediction or failure-time prediction.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22 \
                 under Econometrics + ReliabilitySurvival. Reliability \
                 failure-rate envelope exit (Weibull 6407) and hazard-rate \
                 change (6411) inherit the envelope-exit semantic. Declared \
                 stationarity contract + window contract + residual definition \
                 (observed minus model-predicted) + nominal envelope bounds + \
                 envelope-exit decision law + unit law + numeric mode. No \
                 duplicate admitted; hazard-rate change (6411) collapses here \
                 as ParameterizationOf. Reliability envelope signal witness, \
                 not RUL certainty or failure-time prediction.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            reason: "CUSUM (SEED id 3) is the shared structural-change \
                 ancestor for the Econometrics and ReliabilitySurvival \
                 source classes. CUSUM-of-recursive-residuals (6409; Brown / \
                 Durbin / Evans 1975) and cumulative-damage residual (6412; \
                 Palmgren 1924 / Miner 1945) are descendants. The court \
                 records the domain transfer without re-canonicalising CUSUM.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the shared \
                 envelope-boundary ancestor for the ReliabilitySurvival \
                 source class. Weibull failure-rate envelope exit (6407) and \
                 hazard-rate change (6411) inherit the envelope-exit \
                 semantic without re-canonicalisation. The court records \
                 the domain transfer without re-canonicalising Residual \
                 envelope exit.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID),
            reason: "CUSUM-of-recursive-residuals is ParameterizationOf \
                 (CUSUM, SEED id 3). Recursive-residual parameterization \
                 with declared OLS recursive-residual computation per \
                 observation + normalised CUSUM-of-recursive-residuals path \
                 + boundary-crossing decision law (Brown / Durbin / Evans \
                 1975 structural-stability test). The court declines to \
                 admit CUSUM-of-recursive-residuals as a new canonical \
                 primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID),
            reason: "Quandt-Andrews / Chow structural-break F-test is \
                 ParameterizationOf(Page-Hinkley, SEED id 4). F-statistic \
                 parameterization with declared structural-break F-statistic \
                 law (Quandt 1960; Chow 1960; Andrews 1993) + supremum / \
                 average / exponential aggregation rule over candidate break \
                 dates. The court declines to admit Quandt-Andrews / Chow \
                 structural-break F-test as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID),
            reason: "Hazard-rate change is ParameterizationOf(Residual \
                 envelope exit, SEED id 22). Hazard-rate-envelope \
                 parameterization with declared piecewise-constant hazard \
                 model + hazard-rate envelope bounds + envelope-exit \
                 decision law. The court declines to admit hazard-rate \
                 change as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID),
            reason: "Cumulative damage residual is ParameterizationOf(CUSUM, \
                 SEED id 3). Cumulative-damage parameterization with \
                 declared per-cycle damage-increment law (Palmgren 1924 / \
                 Miner 1945 linear cumulative damage rule) + cumulative-\
                 damage decision threshold + S-N curve reference. The court \
                 declines to admit cumulative damage residual as a new \
                 canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID),
            reason: "Learned market predictor / black-box financial forecaster \
                 (Bloomberg AIM alpha capture, AlphaSense AI signals, Kavout \
                 K Score, Goldman Sachs SecDB ML, JP Morgan COIN / LOXM \
                 execution AI) exposes market-prediction / investment-\
                 recommendation scores from opaque learned embeddings \
                 without a deterministic feature-extraction law, declared \
                 formula, declared training-data anchor, declared tie-break \
                 law, or declared numeric mode. Rejected unless reduced to a \
                 Deterministic_Market_Forecast_Proxy with deterministic \
                 feature-extraction law + declared formula + declared \
                 training-data anchor + feature schema + tie-break + numeric \
                 mode + no market-prediction claim + no investment-\
                 recommendation claim + no credit-decision claim + no \
                 actuarial-pricing claim, all brutally explicit in a later \
                 T.12.x. The court does NOT issue market prediction, \
                 investment advice, credit-decision authority, or actuarial-\
                 pricing authority; those terms appear here only to describe \
                 what is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID),
            reason: "Learned RUL classifier / black-box predictive-maintenance \
                 score (Uptake AI, C3.ai predictive maintenance, Senseye \
                 Predictive Maintenance, IBM Maximo RUL, Siemens MindSphere \
                 Asset Analytics) exposes remaining-useful-life and failure-\
                 time prediction scores from learned embeddings without a \
                 deterministic feature-extraction law, declared formula, \
                 declared training-data anchor, declared tie-break law, or \
                 declared numeric mode. Rejected unless reduced to a \
                 Deterministic_RUL_Proxy with deterministic feature-\
                 extraction law + declared formula + declared training-data \
                 anchor + feature schema + tie-break + numeric mode + no \
                 RUL-certainty claim + no failure-time-prediction claim + no \
                 maintenance-recommendation claim, all brutally explicit in \
                 a later T.12.x. The court does NOT issue RUL certainty, \
                 failure-time prediction, or maintenance recommendations; \
                 those terms appear here only to describe what is NOT \
                 admitted.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn econ_reliability_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(GARCH_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COINTEGRATION_BREAK_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HAUSMAN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(MANN_KENDALL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BAI_PERRON_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PAGE_HINKLEY_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(KM_SURVIVAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COX_SCHOENFELD_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(KM_SURVIVAL_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PARIS_ERDOGAN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(MANN_KENDALL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(PAGE_HINKLEY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Eleven source refs supporting the econometrics + reliability
/// / survival expansion.
fn econ_reliability_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "brown_durbin_evans_1975",
            title: "Techniques for Testing the Constancy of Regression Relationships \
                Over Time",
            year: 1975,
            venue: "Journal of the Royal Statistical Society Series B 37(2) \
                (CUSUM-of-recursive-residuals canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "bollerslev_garch_1986",
            title: "Generalized Autoregressive Conditional Heteroskedasticity",
            year: 1986,
            venue: "Journal of Econometrics 31(3) (GARCH canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "hansen_cointegration_1992",
            title: "Tests for Parameter Instability in Regressions with I(1) \
                Processes",
            year: 1992,
            venue: "Journal of Business and Economic Statistics 10(3) (cointegration-\
                break canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "hausman_test_1978",
            title: "Specification Tests in Econometrics",
            year: 1978,
            venue: "Econometrica 46(6) (Hausman-test canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "bai_perron_1998",
            title: "Estimating and Testing Linear Models with Multiple Structural \
                Changes",
            year: 1998,
            venue: "Econometrica 66(1) (Bai-Perron canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "kaplan_meier_1958",
            title: "Nonparametric Estimation from Incomplete Observations",
            year: 1958,
            venue: "Journal of the American Statistical Association 53(282) \
                (product-limit estimator canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "cox_proportional_hazards_1972",
            title: "Regression Models and Life-Tables",
            year: 1972,
            venue: "Journal of the Royal Statistical Society Series B 34(2) \
                (Cox proportional-hazards canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "schoenfeld_residuals_1982",
            title: "Partial Residuals for the Proportional Hazards Regression Model",
            year: 1982,
            venue: "Biometrika 69(1) (Schoenfeld residual canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "weibull_distribution_1951",
            title: "A Statistical Distribution Function of Wide Applicability",
            year: 1951,
            venue: "Journal of Applied Mechanics 18 (Weibull distribution canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "paris_erdogan_1963",
            title: "A Critical Analysis of Crack Propagation Laws",
            year: 1963,
            venue: "Journal of Basic Engineering 85(4) (Paris-Erdogan crack-growth \
                canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "vendor_market_and_rul_refs",
            title: "Vendor Market-Prediction Pipelines (Bloomberg AIM, AlphaSense, \
                Kavout, Goldman SecDB ML, JP Morgan COIN / LOXM) + Vendor RUL \
                Pipelines (Uptake AI, C3.ai PdM, Senseye PdM, IBM Maximo RUL, \
                Siemens MindSphere Asset Analytics)",
            year: 2023,
            venue: "Vendor documentation (rejection-shell reference; vendor scores \
                lack public deterministic feature-extraction law and embed market-\
                prediction / RUL-certainty claims)",
        },
    ]
}

/// Build the T.12.n econometrics + reliability / survival
/// `DedupCourtDelta`. The delta names EIGHT new canonicals at
/// 6401..=6408 (4 econometric + 4 reliability / survival).
fn build_econ_reliability_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_n_econometrics_reliability_delta",
        vec![
            DetectorCanonicalId(GARCH_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(COINTEGRATION_BREAK_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(HAUSMAN_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BAI_PERRON_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(KM_SURVIVAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(COX_SCHOENFELD_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PARIS_ERDOGAN_RESERVED_CANONICAL_ID),
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

/// Build the T.12.n Econometrics + Reliability / Survival
/// `CorpusAmendmentProposal`. Two builds against this static
/// seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_n_econometrics_reliability_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_n_econometrics_reliability_first_proposal",
        "T.12.n files the Econometrics + Reliability / Survival amendment \
         proposal. Combines two source-class domains under one proposal because \
         they share structural-break / CUSUM / envelope-residual ancestry; the \
         proposal's target_source_class is Econometrics (per the canonical T.12 \
         sequence) and the reliability / survival primitives are admitted in \
         the same body. Adds EIGHT genuinely new canonicals at reserved \
         canonical ids 6401..=6408: four econometric (GARCH volatility residual \
         per Bollerslev 1986, cointegration-break per Hansen 1992 / Quintos & \
         Phillips 1993, Hausman-test residual per Hausman 1978, Bai-Perron \
         multiple-break detector per Bai-Perron 1998 / 2003) plus four \
         reliability / survival (Kaplan-Meier survival residual per Kaplan & \
         Meier 1958, Cox proportional-hazards / Schoenfeld residual per Cox \
         1972 / Schoenfeld 1982, Weibull failure-rate envelope exit per Weibull \
         1951, Crack-growth law residual per Paris & Erdogan 1963). Each \
         declares stationarity contract + window contract + regression / hazard \
         model + censoring law (where applicable) + time-origin law (where \
         applicable) + residual definition + decision functional + confuser \
         profile + numeric mode. Records FOUR ExistingCanonicalAuthorityResolution \
         decisions keeping CUSUM (SEED 3), Page-Hinkley (4), Mann-Kendall (11), \
         Residual envelope exit (22) canonical under Econometrics + \
         ReliabilitySurvival. Records TWO DomainTransferOf decisions: SEED 3 \
         CUSUM as shared structural-change ancestor; SEED 22 Residual envelope \
         exit as shared envelope-boundary ancestor. Records FOUR \
         ParameterizationOf decisions (panel-candidate canonicals that \
         collapsed on closer inspection): CUSUM-of-recursive-residuals (6409; \
         Brown / Durbin / Evans 1975) is ParameterizationOf(CUSUM, SEED 3); \
         Quandt-Andrews / Chow structural-break F-test (6410; Quandt 1960 / \
         Chow 1960 / Andrews 1993) is ParameterizationOf(Page-Hinkley, SEED \
         4); hazard-rate change (6411) is ParameterizationOf(Residual envelope \
         exit, SEED 22); cumulative damage residual (6412; Palmgren 1924 / \
         Miner 1945) is ParameterizationOf(CUSUM, SEED 3). Rejects TWO records \
         as RejectedNotDeterministic (eighth T.12.x with two rejections, \
         following T.12.g / h / i / j / k / l / m): learned market predictor / \
         black-box financial forecaster (6413; Bloomberg AIM alpha capture, \
         AlphaSense AI signals, Kavout K Score, Goldman Sachs SecDB ML, JP \
         Morgan COIN / LOXM) and learned RUL classifier / black-box predictive-\
         maintenance score (6414; Uptake AI, C3.ai PdM, Senseye PdM, IBM Maximo \
         RUL, Siemens MindSphere Asset Analytics). Panel-locked non-claim: \
         T.12.n admits deterministic econometric, reliability, survival, and \
         degradation witnesses. It does not admit market prediction, investment \
         advice, credit-decision authority, actuarial pricing authority, causal \
         economic certainty, RUL certainty, maintenance recommendations, or \
         failure-time prediction. Every CanonicalAddition / \
         ExistingCanonicalAuthorityResolution reason text declares the full \
         contract AND avoids the panel-locked forbidden terms (pinned by \
         t12_n_rejects_market_prediction_claim_language, \
         t12_n_rejects_investment_or_credit_decision_claim_language, and \
         t12_n_rejects_rul_or_failure_time_certainty_claim_language scanners; \
         plus survival-without-censoring and econometric-without-stationarity \
         and black-box-forecaster-without-formula contract guards). Does NOT \
         mutate SEED (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::Econometrics,
        build_econ_reliability_expansion_batch(),
        build_econ_reliability_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_n_econometrics_reliability",
    )
}
