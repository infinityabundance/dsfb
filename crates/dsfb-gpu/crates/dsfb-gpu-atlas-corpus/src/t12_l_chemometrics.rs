//! T.12.l — Chemometrics: the twelfth real literature
//! expansion proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.l files the Chemometrics amendment proposal. It
//! > admits only deterministic chemometric residual / latent-
//! > space / calibration / concentration-structure witnesses
//! > whose sample matrix, preprocessing law, scaling law,
//! > latent-space model, calibration / residual law,
//! > validation split, unit semantics, and decision functional
//! > are declared; resolves SEED collisions; classifies
//! > variants as parameterizations or domain transfers;
//! > rejects black-box spectroscopy classifiers and adaptive-
//! > AutoML chemometric models without fixed component-
//! > selection law; and preserves the frozen T.10 corpus
//! > hash.**
//!
//! **Main panel warning (panel-locked)**: *"A chemometric
//! witness is admissible only when the sample matrix,
//! preprocessing law, scaling law, latent-space model,
//! calibration / residual law, validation split, unit
//! semantics, and decision functional are declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.l admits deterministic chemometric residual /
//! > latent-space / calibration / concentration-structure
//! > witnesses. It does not admit chemical causation, material
//! > identification certainty, regulatory compliance, lab
//! > diagnosis, or process-control authority.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.l's design began with a grep of [`crate::seed::SEED`]
//! for every chemometric candidate. The walk found **four**
//! T.12.l-relevant primitives already canonical (same latent-
//! space + envelope set T.12.k authority-resolved under
//! `FaultDetectionDiagnostics`, now re-resolved under
//! `Chemometrics`):
//!
//! * **PCA T² on score vector** at SEED id 19 — Hotelling
//!   T² score-space distance.
//! * **PCA SPE / Q residual** at SEED id 20 — residual-space
//!   sum-of-squared-residuals.
//! * **PLS residual / Q on PLS** at SEED id 21 — partial-
//!   least-squares regression residual.
//! * **Residual envelope exit** at SEED id 22 — envelope-
//!   boundary detection for spectral preprocessing artifacts.
//!
//! All four become `ExistingCanonicalAuthorityResolution`
//! records under the `Chemometrics` source class. **Panel-
//! locked success-shape** (mirroring T.12.k): the campaign's
//! strength comes from cross-class dedup discipline, not
//! detector count — only FIVE new canonicals admitted after
//! SEED-walk collapse.
//!
//! Five genuinely new canonicals at reserved ids 6201..=6205
//! survived the SEED-walk as structurally distinct decision
//! functionals:
//!
//! * **Calibration residual witness** (6201) — declared
//!   calibration dataset anchor, cross-validation procedure
//!   (k-fold / leave-one-out with fixed seed), and per-sample
//!   prediction residual definition vs reference value.
//!   Structurally distinct from SEED 20 PCA SPE / Q because
//!   the residual is against a REFERENCE value (chemistry
//!   ground truth), not against a PCA reconstruction.
//! * **Leverage outlier** (6202) — declared latent-space model
//!   and leverage decision law (diagonal of projection / hat
//!   matrix crosses threshold). Structurally distinct from
//!   SEED 19 PCA T² because leverage is the SAMPLE INFLUENCE
//!   in the projection, not the score-space Mahalanobis
//!   distance.
//! * **Concentration drift witness** (6203) — declared
//!   calibration model (PLS / inverse least squares /
//!   classical least squares), reference concentration anchor,
//!   and predicted-concentration shift law. Calibration-bound:
//!   refuses to fire without a declared calibration model.
//! * **SIMCA class-distance witness** (6204) — declared
//!   per-class PCA model (Wold & Sjöström 1977 Soft
//!   Independent Modeling of Class Analogies), per-class
//!   residual, and class-distance decision law. Structurally
//!   distinct from PCA T² because SIMCA has a CLASS MODEL
//!   (multiple PCA models, one per class), not a single
//!   global model.
//! * **Variable-importance (VIP) shift witness** (6205) —
//!   declared PLS model, per-variable VIP score (Wold 1995),
//!   and VIP shift decision law (rate-of-change over windows).
//!   Distinct from VIP magnitude — this is the temporal shift
//!   of per-variable importance, not the static score.
//!
//! Two domain transfers (panel-locked):
//!
//! * **PCA T² on score vector** (SEED 19) → `DomainTransferOf`
//!   for `Chemometrics` as the shared latent-space ancestor
//!   (leverage outlier 6202, PCA score outlier 6206,
//!   Mahalanobis-on-scores 6207, SIMCA class-distance 6204
//!   are descendants).
//! * **Residual envelope exit** (SEED 22) → `DomainTransferOf`
//!   for `Chemometrics` as the shared envelope-boundary
//!   ancestor (spectral preprocessing artifact 6209 is the
//!   chemometrics descendant).
//!
//! Four parameterizations (panel-candidate canonicals that
//! collapsed on closer inspection):
//!
//! * **PCA score outlier** (6206) → `ParameterizationOf(PCA
//!   T², SEED 19)` with outlier-flag parameterization on top
//!   of T² score-space distance.
//! * **Mahalanobis distance on PCA scores** (6207) →
//!   `ParameterizationOf(PCA T², SEED 19)` — Mahalanobis is
//!   the distance form underlying T²; explicit naming as a
//!   distinct distance metric collapses here.
//! * **Latent-variable control chart** (6208) →
//!   `ParameterizationOf(PCA SPE / Q residual, SEED 20)` with
//!   control-limit + windowed-monitoring parameterization on
//!   top of the SPE residual.
//! * **Spectral preprocessing artifact** (6209) →
//!   `ParameterizationOf(Residual envelope exit, SEED 22)`
//!   with declared spectral-preprocessing chain (SNV / MSC /
//!   Savitzky-Golay derivative / baseline correction) +
//!   artifact-detection decision law on top of envelope-exit
//!   semantics.
//!
//! Two rejections (sixth T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, h, i,
//! j, k):
//!
//! * **Black-box spectroscopy classifier** (6210) —
//!   `RejectedNotDeterministic`. Vendor spectroscopy AI
//!   (Bruker AI-IDENT, Mettler-Toledo Spectraline, Thermo
//!   Scientific OMNIC ML classifiers, Agilent MicroLab AI)
//!   exposes material-identification / spectral-classification
//!   scores without public decision functional. Admission
//!   requires a future T.12.x to admit a
//!   `Deterministic_Spectroscopy_Classifier_Proxy` canonical
//!   with deterministic formula, model-identification anchor,
//!   training-data anchor, feature schema, tie-break, and
//!   numeric mode all brutally explicit. The court does NOT
//!   issue material identification certainty.
//! * **Adaptive-AutoML / stochastic-CV chemometric model**
//!   (6211) — `RejectedNotDeterministic`. Chemometric models
//!   selected by undeclared adaptive AutoML (auto-sklearn,
//!   H2O AutoML, TPOT) or stochastic cross-validation
//!   (shuffled K-fold without fixed seed; bootstrap
//!   resampling without fixed sample schedule) produce
//!   different component selections per run. Admission
//!   requires fixed component-selection law, fixed CV seed,
//!   fixed train / test split, and fixed preprocessing chain
//!   all declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×5.
//! * `ExistingCanonicalAuthorityResolution` ×4 — SEED 19, 20,
//!   21, 22.
//! * `DomainTransferOf` ×2 — SEED 19 (latent-space ancestor)
//!   + SEED 22 (envelope ancestor).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 5 + 4 + 2 + 4 + 2 = **17 dedup-court records**.
//!
//! ## Material-identification / regulatory-compliance discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND
//! ExistingCanonicalAuthorityResolution reason text MUST
//! describe its record as a "chemometric signal witness" /
//! "latent-space witness" / "calibration residual witness" /
//! "concentration-structure witness" — NEVER as a material-
//! identification claim, regulatory-compliance certification,
//! lab-diagnosis verdict, or process-control authority. The
//! dedicated load-bearing negatives scan every such reason for
//! forbidden terms (material identification, compound
//! identification, regulatory compliance, FDA approved, USP
//! compliant, ICH compliant, lab diagnosis, process control
//! authority) and assert every qualifying reason ends with
//! the panel-locked non-claim "chemometric signal witness,
//! not material identification or regulatory compliance".
//! Forbidden terms appear ONLY in `RejectedNotDeterministic`
//! reason text.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.l chemometrics
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
// Reserved id constants (panel-locked, 6201..=6211 used; 6212..=6299 reserved)
// ---------------------------------------------------------------

/// Reserved canonical id for Calibration residual witness.
/// Distinct from SEED 20 PCA SPE/Q because residual is against
/// a chemistry REFERENCE value, not against a PCA reconstruction.
pub const CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 6201;

/// Reserved canonical id for Leverage outlier. Distinct from
/// SEED 19 PCA T² because leverage is sample INFLUENCE in the
/// projection (diagonal of hat matrix), not score-space
/// Mahalanobis distance.
pub const LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID: u32 = 6202;

/// Reserved canonical id for Concentration drift witness.
/// Calibration-bound; refuses to fire without declared
/// calibration model.
pub const CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID: u32 = 6203;

/// Reserved canonical id for SIMCA class-distance witness.
/// Wold & Sjöström 1977 Soft Independent Modeling of Class
/// Analogies; distinct from PCA T² because SIMCA has multiple
/// per-class PCA models, not a single global model.
pub const SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID: u32 = 6204;

/// Reserved canonical id for Variable-importance (VIP) shift
/// witness. Wold 1995 VIP score; this is the TEMPORAL SHIFT
/// of per-variable importance, not the static score.
pub const VIP_SHIFT_RESERVED_CANONICAL_ID: u32 = 6205;

/// Reserved id for PCA score outlier.
/// `ParameterizationOf(PCA T², SEED 19)`.
pub const PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID: u32 = 6206;

/// Reserved id for Mahalanobis distance on PCA scores.
/// `ParameterizationOf(PCA T², SEED 19)`.
pub const MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID: u32 = 6207;

/// Reserved id for Latent-variable control chart.
/// `ParameterizationOf(PCA SPE / Q residual, SEED 20)`.
pub const LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID: u32 = 6208;

/// Reserved id for Spectral preprocessing artifact witness.
/// `ParameterizationOf(Residual envelope exit, SEED 22)` with
/// declared SNV / MSC / Savitzky-Golay preprocessing chain.
pub const SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID: u32 = 6209;

/// Reserved id for Black-box spectroscopy classifier.
/// `RejectedNotDeterministic`.
pub const BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 6210;

/// Reserved id for Adaptive-AutoML / stochastic-CV chemometric
/// model. `RejectedNotDeterministic`.
pub const ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID: u32 = 6211;

// Existing SEED canonical ids referenced by T.12.l.

/// PCA T-squared on score vector — SEED canonical id 19.
/// Shared latent-space ancestor for chemometrics.
pub const PCA_T2_SEED_ID: u32 = 19;

/// PCA SPE / Q residual — SEED canonical id 20.
pub const PCA_SPE_Q_SEED_ID: u32 = 20;

/// PLS residual / Q on PLS — SEED canonical id 21.
pub const PLS_RESIDUAL_SEED_ID: u32 = 21;

/// Residual envelope exit — SEED canonical id 22. Shared
/// envelope-boundary ancestor for chemometric spectral
/// preprocessing artifacts.
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
// Builders for the chemometrics expansion batch
// ---------------------------------------------------------------

/// Build the chemometrics `CorpusExpansionBatch` body.
fn build_chemometrics_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_l_chemometrics_first_proposal",
        SourceClass::Chemometrics,
        chemometrics_proposed_primitives(),
        chemometrics_proposed_aliases(),
        chemometrics_proposed_dedup_records(),
        chemometrics_proposed_genealogy_edges(),
        chemometrics_proposed_source_refs(),
    )
}

/// Eleven proposed primitives: 5 canonical + 4 parameterization
/// shells + 2 rejection shells. The "few new canonicals, many
/// authority resolutions and parameterizations" shape mirrors
/// the panel-locked T.12.k success posture.
fn chemometrics_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "Calibration residual witness",
            motivation: "Calibration residual signal witness for chemometric \
                 regression / calibration. Required contract: sample matrix, \
                 preprocessing law (SNV / MSC / Savitzky-Golay derivative / \
                 baseline correction declared), scaling law (mean-centering / \
                 auto-scaling / pareto), calibration dataset anchor (pinned \
                 dataset record-hash), cross-validation procedure (k-fold or \
                 leave-one-out with fixed seed), per-sample prediction residual \
                 definition vs reference value, unit semantics, residual decision \
                 law (residual crosses calibration-derived threshold). \
                 Structurally distinct from SEED 20 PCA SPE / Q because residual \
                 is against a chemistry REFERENCE value, not a PCA \
                 reconstruction. Chemometric signal witness, not material \
                 identification or regulatory compliance.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID),
            display_name: "Leverage outlier",
            motivation: "Leverage outlier signal witness for chemometric \
                 regression. Required contract: sample matrix, preprocessing law, \
                 scaling law, latent-space model (PCA / PLS with declared \
                 component-selection law), leverage computation (diagonal of \
                 projection / hat matrix H = X(XᵀX)⁻¹Xᵀ), leverage decision \
                 law (per-sample leverage crosses threshold derived from p / n \
                 ratio or 2p/n / 3p/n rule), unit semantics. Structurally \
                 distinct from SEED 19 PCA T² because leverage is sample \
                 INFLUENCE in the projection, not score-space Mahalanobis \
                 distance. Chemometric signal witness, not material \
                 identification or regulatory compliance.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID),
            display_name: "Concentration drift witness",
            motivation: "Concentration drift signal witness. Required contract: \
                 sample matrix, preprocessing law, calibration model (PLS / \
                 inverse least squares / classical least squares with declared \
                 component-selection law), reference concentration anchor \
                 (pinned reference set), predicted-concentration shift law \
                 (concentration prediction relative to baseline window crosses \
                 threshold), unit semantics, validation split (declared train / \
                 test or temporal holdout). Calibration-bound: refuses to fire \
                 without a declared calibration model. Chemometric signal \
                 witness, not material identification or regulatory compliance.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID),
            display_name: "SIMCA class-distance witness",
            motivation: "SIMCA class-distance signal witness (Wold & Sjöström \
                 1977 Soft Independent Modeling of Class Analogies). Required \
                 contract: sample matrix, preprocessing law, scaling law, per- \
                 class PCA model (one PCA model per declared class with \
                 component-selection law declared per class), per-class residual \
                 computation, class-distance decision law (sample-to-class- \
                 model residual crosses class-specific threshold), unit \
                 semantics. Structurally distinct from SEED 19 PCA T² because \
                 SIMCA has multiple per-class PCA models, not a single global \
                 model. Chemometric signal witness, not material identification \
                 or regulatory compliance.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(VIP_SHIFT_RESERVED_CANONICAL_ID),
            display_name: "Variable-importance (VIP) shift witness",
            motivation: "Variable-Importance-in-Projection (VIP) temporal shift \
                 signal witness (Wold 1995). Required contract: sample matrix, \
                 preprocessing law, PLS model (with declared component-selection \
                 law), per-variable VIP score computation (VIP_j = sqrt(p * sum_a \
                 (q_a² * w_aj² * SS_a) / SS_total)), VIP shift decision law \
                 (rate-of-change of per-variable VIP over declared windows \
                 crosses threshold), unit semantics. Distinct from VIP MAGNITUDE \
                 — this is the TEMPORAL SHIFT of per-variable importance, not \
                 the static score. Chemometric signal witness, not material \
                 identification or regulatory compliance.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID),
            display_name: "PCA score outlier - parameterization shell",
            motivation: "Outlier-flag parameterization of PCA T² (SEED id 19) \
                 with declared T²-threshold-based outlier predicate on top of \
                 score-space Mahalanobis distance. The court rules: PCA score \
                 outlier is ParameterizationOf(PCA T², SEED 19), NOT a new \
                 canonical primitive. Appears in proposed_primitives but NOT in \
                 new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID),
            display_name: "Mahalanobis distance on PCA scores - parameterization shell",
            motivation: "Distance-metric parameterization of PCA T² (SEED id 19). \
                 Mahalanobis distance IS the distance form underlying T² — \
                 explicit naming as a distinct distance metric collapses here. \
                 The court rules: Mahalanobis distance on PCA scores is \
                 ParameterizationOf(PCA T², SEED 19) with explicit covariance- \
                 matrix declaration, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID),
            display_name: "Latent-variable control chart - parameterization shell",
            motivation: "Control-limit + windowed-monitoring parameterization of \
                 PCA SPE / Q residual (SEED id 20) with declared control limits \
                 + window length + sustained-violation rule on top of SPE \
                 residual semantics. The court rules: latent-variable control \
                 chart is ParameterizationOf(PCA SPE / Q residual, SEED 20), \
                 NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Spectral preprocessing artifact - parameterization shell",
            motivation: "Spectral-preprocessing parameterization of Residual \
                 envelope exit (SEED id 22) with declared preprocessing chain \
                 (SNV / MSC / Savitzky-Golay derivative / baseline correction) \
                 + artifact-detection decision law (preprocessing-induced \
                 residual crosses envelope bound). The court rules: spectral \
                 preprocessing artifact is ParameterizationOf(Residual envelope \
                 exit, SEED 22), NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Black-box spectroscopy classifier - rejected shell",
            motivation: "Vendor spectroscopy AI classifiers (Bruker AI-IDENT, \
                 Mettler-Toledo Spectraline, Thermo Scientific OMNIC ML \
                 classifiers, Agilent MicroLab AI) expose material-identification \
                 / spectral-classification scores without public decision \
                 functional, declared training-data anchor, or model- \
                 identification anchor. The court does NOT admit black-box \
                 spectroscopy classifiers to the dedup-court delta's \
                 new_canonical_records. A future T.12.x may admit a \
                 Deterministic_Spectroscopy_Classifier_Proxy canonical only if \
                 a deterministic formula, model-identification anchor, training- \
                 data anchor, feature schema, tie-break, and numeric mode are \
                 all brutally explicit. The court does NOT issue material \
                 identification certainty; the term appears here only to \
                 describe what is NOT admitted.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Adaptive-AutoML / stochastic-CV chemometric model - rejected shell",
            motivation: "Chemometric models selected by undeclared adaptive \
                 AutoML (auto-sklearn, H2O AutoML, TPOT) or stochastic cross- \
                 validation (shuffled K-fold without fixed seed; bootstrap \
                 resampling without fixed sample schedule) produce different \
                 component selections per run. The court does NOT admit such \
                 models. A future T.12.x proposal may admit a \
                 Deterministic_AutoML_Chemometric_Proxy canonical only if \
                 fixed component-selection law + fixed CV seed + fixed train / \
                 test split + fixed preprocessing chain are all declared.",
        },
    ]
}

/// Zero alias claims (T.12.l routes everything through dedup
/// records and existing-canonical authority resolutions).
fn chemometrics_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Seventeen dedup-court decisions on the chemometrics batch.
fn chemometrics_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 5 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "Calibration residual witness: declared sample matrix + \
                 preprocessing law (SNV / MSC / Savitzky-Golay derivative / \
                 baseline correction) + scaling law (mean-centering / auto- \
                 scaling / pareto) + calibration dataset anchor (pinned dataset \
                 record-hash) + cross-validation procedure (k-fold or leave-one- \
                 out with fixed seed) + per-sample prediction residual \
                 definition vs reference value + unit semantics + residual \
                 decision law. Structurally distinct from SEED 20 PCA SPE / Q \
                 because residual is against chemistry reference, not PCA \
                 reconstruction. Chemometric signal witness, not material \
                 identification or regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID),
            reason: "Leverage outlier: declared sample matrix + preprocessing \
                 law + scaling law + latent-space model (PCA / PLS with \
                 declared component-selection law) + leverage computation \
                 (diagonal of hat matrix H = X(XᵀX)⁻¹Xᵀ) + leverage decision \
                 law (per-sample leverage crosses 2p/n or 3p/n threshold) + \
                 unit semantics. Structurally distinct from SEED 19 PCA T² \
                 because leverage is sample INFLUENCE, not score-space \
                 Mahalanobis distance. Chemometric signal witness, not \
                 material identification or regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID),
            reason: "Concentration drift witness: declared sample matrix + \
                 preprocessing law + calibration model (PLS / ILS / CLS with \
                 declared component-selection law) + reference concentration \
                 anchor (pinned reference set) + predicted-concentration shift \
                 law (concentration prediction relative to baseline crosses \
                 threshold) + unit semantics + validation split. Calibration- \
                 bound: refuses to fire without a declared calibration model. \
                 Chemometric signal witness, not material identification or \
                 regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID),
            reason: "SIMCA class-distance witness (Wold & Sjöström 1977 Soft \
                 Independent Modeling of Class Analogies): declared sample \
                 matrix + preprocessing law + scaling law + per-class PCA \
                 model (one PCA per declared class with component-selection \
                 law declared per class) + per-class residual computation + \
                 class-distance decision law (sample-to-class-model residual \
                 crosses class-specific threshold) + unit semantics. \
                 Structurally distinct from SEED 19 PCA T² because SIMCA has \
                 multiple per-class PCA models, not a single global model. \
                 Chemometric signal witness, not material identification or \
                 regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(VIP_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Variable-importance (VIP) shift witness (Wold 1995): \
                 declared sample matrix + preprocessing law + PLS model (with \
                 declared component-selection law) + per-variable VIP score \
                 computation (VIP_j = sqrt(p * sum_a (q_a² * w_aj² * SS_a) / \
                 SS_total)) + VIP shift decision law (rate-of-change of per- \
                 variable VIP over windows crosses threshold) + unit \
                 semantics. Distinct from VIP MAGNITUDE — this is temporal \
                 shift, not static score. Chemometric signal witness, not \
                 material identification or regulatory compliance.",
        },
        // -- 4 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            reason: "PCA T-squared on score vector stays canonical at SEED id \
                 19 under Chemometrics (same authority T.12.k ratified under \
                 FaultDetectionDiagnostics, now ratified under Chemometrics). \
                 Declared sample matrix + preprocessing law + scaling law + \
                 PCA model (pinned loadings + nominal score-space covariance + \
                 declared component-selection law) + T² decision law \
                 (Mahalanobis distance in score space crosses control limit) \
                 + unit semantics. No duplicate admitted; PCA score outlier \
                 (6206), Mahalanobis-on-scores (6207) collapse here as \
                 ParameterizationOf. Chemometric signal witness, not material \
                 identification or regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            reason: "PCA SPE / Q residual stays canonical at SEED id 20 under \
                 Chemometrics. Declared sample matrix + preprocessing law + \
                 scaling law + PCA model (pinned loadings + declared component- \
                 selection law) + per-variable residual computation + SPE \
                 decision law (sum-of-squared-residuals crosses control limit) \
                 + unit semantics. No duplicate admitted; latent-variable \
                 control chart (6208) collapses here as ParameterizationOf. \
                 Chemometric signal witness, not material identification or \
                 regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PLS_RESIDUAL_SEED_ID),
            reason: "PLS residual / Q on PLS stays canonical at SEED id 21 \
                 under Chemometrics. Declared sample matrix + preprocessing \
                 law + scaling law + PLS model (pinned X- and Y-loadings + \
                 inner-relation regression + declared component-selection law) \
                 + per-variable residual computation + Q decision law + unit \
                 semantics + validation split. No duplicate admitted. \
                 Chemometric signal witness, not material identification or \
                 regulatory compliance.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22 \
                 under Chemometrics. Declared sample matrix + preprocessing \
                 law + residual definition (observed minus model-predicted) + \
                 nominal envelope bounds + envelope-exit decision law + unit \
                 semantics. No duplicate admitted; spectral preprocessing \
                 artifact (6209) collapses here as ParameterizationOf. \
                 Chemometric signal witness, not material identification or \
                 regulatory compliance.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            reason: "PCA T² on score vector (SEED id 19) is the shared latent- \
                 space ancestor for the Chemometrics source class. Leverage \
                 outlier (6202), SIMCA class-distance (6204), PCA score \
                 outlier (6206), and Mahalanobis-on-scores (6207) are \
                 chemometric descendants. The court records the domain \
                 transfer without re-canonicalising PCA T².",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the shared \
                 envelope-boundary ancestor for the Chemometrics source class \
                 (spectral preprocessing artifact 6209 is the chemometrics \
                 descendant with declared SNV / MSC / Savitzky-Golay / \
                 baseline-correction preprocessing chain). The court records \
                 the domain transfer without re-canonicalising Residual \
                 envelope exit.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID),
            reason: "PCA score outlier is ParameterizationOf(PCA T², SEED id \
                 19). Outlier-flag parameterization with declared T²-threshold- \
                 based outlier predicate on top of score-space Mahalanobis \
                 distance. The court declines to admit PCA score outlier as a \
                 new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID),
            reason: "Mahalanobis distance on PCA scores is ParameterizationOf \
                 (PCA T², SEED id 19). Distance-metric parameterization — \
                 Mahalanobis IS the distance form underlying T². The court \
                 declines to admit Mahalanobis-on-scores as a new canonical \
                 primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID),
            reason: "Latent-variable control chart is ParameterizationOf(PCA \
                 SPE / Q residual, SEED id 20). Control-limit + windowed- \
                 monitoring parameterization with declared control limits + \
                 window length + sustained-violation rule on top of SPE \
                 residual semantics. The court declines to admit latent- \
                 variable control chart as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(
                SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID,
            ),
            reason: "Spectral preprocessing artifact is ParameterizationOf \
                 (Residual envelope exit, SEED id 22). Declared preprocessing \
                 chain (SNV / MSC / Savitzky-Golay derivative / baseline \
                 correction) + artifact-detection decision law. The court \
                 declines to admit spectral preprocessing artifact as a new \
                 canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(
                BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            reason: "Black-box spectroscopy classifier (Bruker AI-IDENT, \
                 Mettler-Toledo Spectraline, Thermo Scientific OMNIC ML \
                 classifiers, Agilent MicroLab AI) exposes material- \
                 identification / spectral-classification scores without a \
                 deterministic formula, model-identification anchor, training- \
                 data anchor, feature schema, tie-break, or numeric mode. \
                 Rejected unless reduced to a declared \
                 Deterministic_Spectroscopy_Classifier_Proxy with all six \
                 contract fields brutally explicit in a later T.12.x. The \
                 court does NOT issue material identification certainty or \
                 regulatory compliance; those terms appear here only to \
                 describe what is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID),
            reason: "Adaptive-AutoML / stochastic-CV chemometric model (auto- \
                 sklearn, H2O AutoML, TPOT-style search; shuffled K-fold \
                 without fixed seed; bootstrap resampling without fixed \
                 sample schedule) produces different component selections per \
                 run. Rejected unless reduced to a \
                 Deterministic_AutoML_Chemometric_Proxy with fixed component- \
                 selection law + fixed CV seed + fixed train / test split + \
                 fixed preprocessing chain all declared in a later T.12.x.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn chemometrics_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PLS_RESIDUAL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(VIP_SHIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PLS_RESIDUAL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(VIP_SHIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(PCA_T2_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID,
            ),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the chemometrics expansion.
fn chemometrics_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "hotelling_t2_1931",
            title: "The Generalization of Student's Ratio",
            year: 1931,
            venue: "Annals of Mathematical Statistics 2(3) (canonical T² \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "wold_pls_1966",
            title: "Estimation of Principal Components and Related Models by \
                Iterative Least Squares",
            year: 1966,
            venue: "Multivariate Analysis (PLS canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "wold_sjostrom_simca_1977",
            title: "SIMCA: A Method for Analyzing Chemical Data in Terms of \
                Similarity and Analogy",
            year: 1977,
            venue: "ACS Symposium Series 52 (SIMCA canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "geladi_kowalski_pls_1986",
            title: "Partial Least-Squares Regression: A Tutorial",
            year: 1986,
            venue: "Analytica Chimica Acta 185 (PLS regression tutorial)",
        },
        ProposedSourceRef {
            citation_key: "wold_vip_1995",
            title: "PLS for Multivariate Linear Modeling",
            year: 1995,
            venue: "Chemometric Methods in Molecular Design (Variable \
                Importance in Projection VIP canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "brereton_chemometrics_textbook_2003",
            title: "Chemometrics: Data Analysis for the Laboratory and \
                Chemical Plant",
            year: 2003,
            venue: "Wiley (chemometrics textbook canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "astm_e1655_nir_calibration",
            title: "Standard Practices for Infrared Multivariate Quantitative \
                Analysis",
            year: 2017,
            venue: "ASTM E1655 (NIR spectroscopy multivariate calibration \
                standard)",
        },
        ProposedSourceRef {
            citation_key: "ich_q2_r1_analytical_validation",
            title: "Validation of Analytical Procedures: Text and Methodology",
            year: 2005,
            venue: "ICH Q2(R1) (analytical procedure validation standard; \
                cited only to anchor validation-split vocabulary, not to \
                claim ICH compliance)",
        },
        ProposedSourceRef {
            citation_key: "vendor_spectroscopy_classifier_refs",
            title: "Vendor Spectroscopy AI Classifiers (Bruker AI-IDENT, \
                Mettler-Toledo Spectraline, Thermo Scientific OMNIC ML, \
                Agilent MicroLab AI)",
            year: 2023,
            venue: "Vendor documentation (rejection-shell reference; vendor \
                scores lack public decision functional)",
        },
    ]
}

/// Build the T.12.l chemometrics `DedupCourtDelta`.
fn build_chemometrics_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_l_chemometrics_delta",
        vec![
            DetectorCanonicalId(CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(VIP_SHIFT_RESERVED_CANONICAL_ID),
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

/// Build the T.12.l chemometrics `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_l_chemometrics_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_l_chemometrics_first_proposal",
        "T.12.l files the Chemometrics amendment proposal. Adds FIVE genuinely \
         new canonical chemometric primitives (calibration residual witness, \
         leverage outlier, concentration drift witness, SIMCA class-distance \
         witness, variable-importance VIP shift witness) at reserved canonical \
         ids 6201..=6205 with declared sample matrix + preprocessing + scaling \
         + latent-space model + calibration / residual law + validation split + \
         unit semantics + decision functional contracts. Records FOUR \
         ExistingCanonicalAuthorityResolution decisions keeping PCA T² (SEED \
         19), PCA SPE / Q (20), PLS residual (21), Residual envelope exit (22) \
         canonical under Chemometrics — same latent-space + envelope set \
         T.12.k authority-resolved under FaultDetectionDiagnostics, now \
         re-resolved under Chemometrics. Records TWO DomainTransferOf \
         decisions: SEED 19 as shared latent-space ancestor; SEED 22 as shared \
         envelope-boundary ancestor. Records FOUR ParameterizationOf decisions \
         (panel-candidate canonicals that collapsed on closer inspection): \
         PCA score outlier (6206) is ParameterizationOf(PCA T², SEED 19); \
         Mahalanobis distance on PCA scores (6207) is ParameterizationOf(PCA \
         T², SEED 19); latent-variable control chart (6208) is \
         ParameterizationOf(PCA SPE / Q, SEED 20); spectral preprocessing \
         artifact (6209) is ParameterizationOf(Residual envelope exit, SEED \
         22). Rejects TWO chemometric records as RejectedNotDeterministic \
         (sixth T.12.x with two rejections, following T.12.g / h / i / j / k): \
         black-box spectroscopy classifier (6210; Bruker AI-IDENT, Mettler- \
         Toledo Spectraline, Thermo Scientific OMNIC ML, Agilent MicroLab AI) \
         and adaptive-AutoML / stochastic-CV chemometric model (6211; auto- \
         sklearn, H2O AutoML, TPOT-style search). Panel-locked non-claim: \
         T.12.l admits deterministic chemometric residual / latent-space / \
         calibration / concentration-structure witnesses. It does not admit \
         chemical causation, material identification certainty, regulatory \
         compliance, lab diagnosis, or process-control authority. Every \
         CanonicalAddition / ExistingCanonicalAuthorityResolution reason text \
         declares the full contract AND ends with the panel-locked non-claim \
         'chemometric signal witness, not material identification or \
         regulatory compliance' (pinned by \
         t12_l_rejects_material_identification_claim_language and \
         t12_l_rejects_regulatory_compliance_claim_language scanners). Does \
         NOT mutate SEED (SEED.len() stays at 54); status = Open pending \
         review.",
        SourceClass::Chemometrics,
        build_chemometrics_expansion_batch(),
        build_chemometrics_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_l_chemometrics",
    )
}
