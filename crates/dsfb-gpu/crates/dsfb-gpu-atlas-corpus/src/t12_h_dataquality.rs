//! T.12.h — Data Quality / Tabular / Database Integrity
//! Constraints: the eighth real literature expansion proposal
//! filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.h files the Data Quality / Tabular / Database
//! > Integrity amendment proposal. It admits only deterministic
//! > table, schema, integrity, and column-structure witnesses
//! > whose scope, baseline, null semantics, cardinality law,
//! > dependency law, type law, range law, and decision
//! > functional are declared; resolves SEED collisions;
//! > classifies variants as parameterizations or domain
//! > transfers; rejects underspecified leakage or learned
//! > data-quality claims; and preserves the frozen T.10 corpus
//! > hash.**
//!
//! **Main panel warning (panel-locked)**: *"A validation rule
//! is not a detector until scope, baseline, null / type / key
//! semantics, and decision law are declared."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.h's design began with a grep of [`crate::seed::SEED`]
//! for every data-quality candidate. The walk found **five**
//! T.12.h-relevant primitives already canonical:
//!
//! * **Missingness spike** at SEED id 13 — shared data-quality
//!   ancestor recognised by the `DataQualityRules` source class.
//! * **Missingness coupling** at SEED id 44 — multi-column
//!   missingness coupling.
//! * **Schema drift** at SEED id 45 — schema-level structural
//!   change.
//! * **Cardinality drift** at SEED id 46 — per-column
//!   cardinality temporal change.
//! * **Uniqueness violation** at SEED id 47 — primary-key /
//!   candidate-key uniqueness breach.
//!
//! All five become `ExistingCanonicalAuthorityResolution`
//! records with declared scope + baseline + null-semantics +
//! key-scope contracts. EIGHT genuinely new canonicals at
//! reserved ids 5801..=5808 are admitted with declared
//! contracts:
//!
//! * **Functional dependency violation** (5801) — declared
//!   determinant columns + dependent columns + null handling +
//!   duplicate-row handling + violation count law + minimum
//!   support.
//! * **Type instability** (5802) — declared type system
//!   (SQL types / Arrow types / Parquet types) + observed-vs-
//!   expected mismatch law + scope + threshold.
//! * **Target-leakage candidate** (5803) — declared target
//!   column + time / order law + feature-availability time +
//!   correlation / mutual-information / association law +
//!   train / test split or temporal holdout. Panel-locked
//!   non-claim: this is a CANDIDATE witness, not proof of
//!   leakage.
//! * **Correlation break** (5804) — declared correlation
//!   convention (Pearson / Spearman / Kendall) + window-pair +
//!   normalization + threshold.
//! * **Covariance shift** (5805) — declared multivariate
//!   covariance-matrix change + scope + comparison law +
//!   threshold (generalises correlation break to the full
//!   covariance structure).
//! * **Null-run anomaly** (5806) — declared null semantics +
//!   consecutive-null-run law + threshold. Structurally
//!   distinct from Missingness spike (which counts missing
//!   values in a window) and Run-length anomaly (T.12.f 5606,
//!   which tracks consecutive-event runs over a general event
//!   class).
//! * **Range envelope exit (tabular)** (5807) — declared
//!   per-column min / max bounds + unit + inclusive / exclusive
//!   boundary semantics + null handling. Structurally distinct
//!   from SEED 22 Residual envelope exit (residual-magnitude-
//!   based) — tabular range envelope is per-column bound-based.
//! * **Category emergence** (5808) — declared reference-
//!   category set + new-category-appearance law + case
//!   sensitivity + unknown-category handling.
//!
//! Three parameterizations:
//!
//! * **Per-column missingness** (5809) →
//!   `ParameterizationOf(Missingness spike, SEED 13)` — scope
//!   parameterized to a single column.
//! * **Composite-key uniqueness** (5810) →
//!   `ParameterizationOf(Uniqueness violation, SEED 47)` —
//!   key-scope parameterized to a column-tuple.
//! * **Category collapse** (5811) →
//!   `ParameterizationOf(Cardinality drift, SEED 46)` — the
//!   direction-specific variant tracking category disappearance.
//!
//! Two rejections (second T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g):
//!
//! * **Learned data-quality anomaly score** (5812) —
//!   `RejectedNotDeterministic`. Autoencoder-reconstruction
//!   anomaly, Mahalanobis with learned covariance, ML-based
//!   tabular outlier scores (Isolation Forest with random seed
//!   / random feature sample, LOF with learned k-NN graph) are
//!   randomized / learned in origin. Admission requires
//!   model-identification seed + training-data anchor + feature
//!   schema + tie-break + numeric mode declared.
//! * **Auto-schema inference anomaly** (5813) —
//!   `RejectedNotDeterministic`. Auto-detected schema +
//!   constraint-inference (TFDV-style or Great Expectations
//!   profiler) produces different rules per run. Admission
//!   requires inference algorithm + sample seed + sampling
//!   schedule + schema-version anchor + tie-break declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×8.
//! * `ExistingCanonicalAuthorityResolution` ×5.
//! * `DomainTransferOf` ×1 — Missingness spike (SEED 13) as
//!   shared data-quality ancestor for `DataQualityRules`.
//! * `ParameterizationOf` ×3.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 8 + 5 + 1 + 3 + 2 = **19 dedup-court records**.
//!
//! ## Target-leakage non-claim (panel-locked)
//!
//! Target-leakage candidate (5803) is admitted explicitly as a
//! CANDIDATE witness, not proof of leakage. Its reason text
//! carries the panel-locked phrasing "candidate, not proof"
//! so a future activation planner / case-file emitter does
//! NOT promote candidate signals into ratified leakage
//! verdicts.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11/S1.3/T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.h data-quality
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

/// Reserved canonical id for Functional dependency violation.
/// 5801..5813 is the T.12.h bucket.
pub const FD_VIOLATION_RESERVED_CANONICAL_ID: u32 = 5801;

/// Reserved canonical id for Type instability.
pub const TYPE_INSTABILITY_RESERVED_CANONICAL_ID: u32 = 5802;

/// Reserved canonical id for Target-leakage candidate.
/// Panel-locked non-claim: this is a CANDIDATE witness, not
/// proof of leakage.
pub const TARGET_LEAKAGE_RESERVED_CANONICAL_ID: u32 = 5803;

/// Reserved canonical id for Correlation break.
pub const CORRELATION_BREAK_RESERVED_CANONICAL_ID: u32 = 5804;

/// Reserved canonical id for Covariance shift (multivariate
/// generalisation of correlation break).
pub const COVARIANCE_SHIFT_RESERVED_CANONICAL_ID: u32 = 5805;

/// Reserved canonical id for Null-run anomaly. Distinct from
/// Missingness spike (window-count) and Run-length anomaly
/// (T.12.f 5606; general event class).
pub const NULL_RUN_RESERVED_CANONICAL_ID: u32 = 5806;

/// Reserved canonical id for Range envelope exit (tabular).
/// Per-column min/max bounds. Distinct from SEED 22 Residual
/// envelope exit (residual-magnitude-based).
pub const RANGE_ENVELOPE_RESERVED_CANONICAL_ID: u32 = 5807;

/// Reserved canonical id for Category emergence.
pub const CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID: u32 = 5808;

/// Reserved id for Per-column missingness.
/// `ParameterizationOf(Missingness spike, SEED 13)`.
pub const PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID: u32 = 5809;

/// Reserved id for Composite-key uniqueness.
/// `ParameterizationOf(Uniqueness violation, SEED 47)`.
pub const COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID: u32 = 5810;

/// Reserved id for Category collapse.
/// `ParameterizationOf(Cardinality drift, SEED 46)`.
pub const CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID: u32 = 5811;

/// Reserved id for Learned data-quality anomaly score
/// (autoencoder-reconstruction, Mahalanobis with learned
/// covariance, Isolation Forest with random seed, LOF with
/// learned k-NN graph). `RejectedNotDeterministic`.
pub const LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID: u32 = 5812;

/// Reserved id for Auto-schema inference anomaly (TFDV-style
/// auto-detected schema + constraint-inference; Great
/// Expectations profiler with random sampling).
/// `RejectedNotDeterministic`.
pub const AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID: u32 = 5813;

// Existing SEED canonical ids referenced by T.12.h.

/// Residual envelope exit — SEED canonical id 22 (genealogy
/// ancestor for tabular range envelope exit).
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// Autocorrelation-coefficient break — SEED canonical id 40
/// (genealogy ancestor for correlation break).
pub const AUTOCORRELATION_BREAK_SEED_ID: u32 = 40;

/// Missingness spike — SEED canonical id 13. Shared data-
/// quality ancestor for the DataQualityRules source class.
pub const MISSINGNESS_SPIKE_SEED_ID: u32 = 13;

/// Missingness coupling — SEED canonical id 44.
pub const MISSINGNESS_COUPLING_SEED_ID: u32 = 44;

/// Schema drift — SEED canonical id 45.
pub const SCHEMA_DRIFT_SEED_ID: u32 = 45;

/// Cardinality drift — SEED canonical id 46.
pub const CARDINALITY_DRIFT_SEED_ID: u32 = 46;

/// Uniqueness violation — SEED canonical id 47.
pub const UNIQUENESS_VIOLATION_SEED_ID: u32 = 47;

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
// Builders for the data-quality expansion batch
// ---------------------------------------------------------------

/// Build the data-quality `CorpusExpansionBatch` body.
fn build_dataquality_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_h_dataquality_first_proposal",
        SourceClass::DataQualityRules,
        dataquality_proposed_primitives(),
        dataquality_proposed_aliases(),
        dataquality_proposed_dedup_records(),
        dataquality_proposed_genealogy_edges(),
        dataquality_proposed_source_refs(),
    )
}

/// Thirteen proposed primitives: 8 canonical + 3 parameterization
/// + 2 rejection shells.
fn dataquality_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(FD_VIOLATION_RESERVED_CANONICAL_ID),
            display_name: "Functional dependency violation",
            motivation: "Functional-dependency violation detector (Codd 1972 relational \
                 model). Required scope + decision law: determinant columns (the LHS \
                 of the FD A -> B), dependent columns (the RHS), null handling (count \
                 / treat-as-distinct / ignore), duplicate-row handling, violation- \
                 count law, minimum support (min row count for the FD to apply). \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TYPE_INSTABILITY_RESERVED_CANONICAL_ID),
            display_name: "Type instability",
            motivation: "Per-column type instability detector: observed vs expected \
                 type mismatch over time. Required scope + decision law: type system \
                 (SQL types / Arrow types / Parquet types / Python runtime types), \
                 observed-vs-expected mismatch law, scope (per-column / per-row), \
                 threshold (single violation / rate / proportion). Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TARGET_LEAKAGE_RESERVED_CANONICAL_ID),
            display_name: "Target-leakage candidate",
            motivation: "Target-leakage candidate detector: panel-locked non-claim - \
                 this is a CANDIDATE witness, not proof of leakage. Required scope + \
                 decision law: target column declaration, time / order law, feature- \
                 availability time (when each feature becomes observable relative to \
                 the target), association law (correlation / mutual-information / \
                 conditional-mutual-information), train / test split or temporal \
                 holdout, threshold. Deterministic given the train / test split. The \
                 court emits a candidate witness; a future activation planner / \
                 case-file emitter must NOT promote this to a ratified leakage \
                 verdict without additional human review.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            display_name: "Correlation break",
            motivation: "Pairwise-column correlation temporal break detector. Required \
                 scope + decision law: correlation convention (Pearson / Spearman / \
                 Kendall tau), column pair (or set of pairs), window pair (baseline + \
                 active), normalization, threshold. Distinct from SEED 40 \
                 Autocorrelation-coefficient break (which is intra-column lag-based) \
                 — correlation break is INTER-column. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COVARIANCE_SHIFT_RESERVED_CANONICAL_ID),
            display_name: "Covariance shift (multivariate)",
            motivation: "Multivariate covariance-matrix temporal shift. Required scope \
                 + decision law: column set, covariance estimator (sample / shrinkage / \
                 robust), comparison law (Frobenius distance / log-determinant ratio / \
                 affine-invariant Riemannian), threshold. Generalises Correlation \
                 break to the full multivariate covariance structure. Deterministic \
                 when estimator + comparison law are pinned.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(NULL_RUN_RESERVED_CANONICAL_ID),
            display_name: "Null-run anomaly",
            motivation: "Consecutive-null-run detector for tabular columns. Required \
                 scope + decision law: null semantics (NULL / NaN / empty-string / \
                 sentinel-value), consecutive-null-run law (max-run / current-run / \
                 percentile-run), threshold. Structurally distinct from Missingness \
                 spike (which counts missing values in a window, not consecutive \
                 runs) and from Run-length anomaly (T.12.f 5606; general event \
                 class). Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RANGE_ENVELOPE_RESERVED_CANONICAL_ID),
            display_name: "Range envelope exit (tabular)",
            motivation: "Per-column tabular min / max bound exit detector. Required \
                 scope + decision law: per-column bounds, unit, inclusive vs exclusive \
                 boundary semantics, null handling (NULL = pass / NULL = fail / NULL \
                 = quarantine), tie-break at boundary. Structurally distinct from \
                 SEED 22 Residual envelope exit (which is residual-magnitude-based) \
                 - tabular range envelope is per-column bound-based against declared \
                 contract values. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID),
            display_name: "Category emergence",
            motivation: "New-category appearance detector: a categorical value appears \
                 in the active window that was not present in the reference category \
                 set. Required scope + decision law: reference-category set (with \
                 declared anchor: snapshot-hash / pinned-fixture / sliding-window), \
                 new-category-appearance law (first-seen / first-N-occurrences), \
                 case sensitivity, unknown-category handling (treat-as-emergence / \
                 quarantine / pass-through), threshold. Distinct from SEED 46 \
                 Cardinality drift (count-of-distinct-categories shift) - category \
                 emergence tracks the IDENTITY of new categories. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Per-column missingness - parameterization shell",
            motivation: "Per-column scope parameterization of Missingness spike (SEED \
                 id 13). The court rules: per-column missingness is \
                 ParameterizationOf(Missingness spike, SEED 13), NOT a new canonical \
                 primitive. Appears in proposed_primitives but NOT in \
                 new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Composite-key uniqueness - parameterization shell",
            motivation: "Column-tuple key-scope parameterization of Uniqueness \
                 violation (SEED id 47). The court rules: composite-key uniqueness \
                 is ParameterizationOf(Uniqueness violation, SEED 47) with declared \
                 column-tuple as composite key, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID),
            display_name: "Category collapse - parameterization shell",
            motivation: "Direction-specific parameterization of Cardinality drift \
                 (SEED id 46): tracks category disappearance (count-of-distinct- \
                 categories DECREASES rather than changes). The court rules: \
                 category collapse is ParameterizationOf(Cardinality drift, SEED 46) \
                 with declared direction-of-change predicate, NOT a new canonical \
                 primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID),
            display_name: "Learned data-quality anomaly score - rejected shell",
            motivation: "Autoencoder-reconstruction-error anomaly score, Mahalanobis \
                 distance with LEARNED covariance, ML-based tabular outlier scores \
                 (Isolation Forest with random seed / random feature sample, LOF \
                 with learned k-NN graph) are randomized / learned in origin. The \
                 court does NOT admit learned-DQ-score to the dedup-court delta's \
                 new_canonical_records. A future T.12.x proposal may admit a \
                 Deterministic_Learned_DQ_Score_Proxy canonical only if the model- \
                 identification seed, training-data anchor (pinned-fixture-hash), \
                 feature schema (pinned), tie-break law, and numeric mode are all \
                 brutally explicit.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID),
            display_name: "Auto-schema inference anomaly - rejected shell",
            motivation: "Auto-detected schema + constraint-inference (TensorFlow Data \
                 Validation TFDV-style, Great Expectations profiler with random \
                 sampling) produces different rules per run when the inference is \
                 unspecified. The court does NOT admit auto-schema inference anomaly \
                 to the dedup-court delta's new_canonical_records. A future T.12.x \
                 proposal may admit a Deterministic_Schema_Inference_Proxy canonical \
                 only if the inference algorithm, sample seed, sampling schedule, \
                 schema-version anchor, and tie-break law are all brutally explicit.",
        },
    ]
}

/// Zero alias claims.
fn dataquality_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Nineteen dedup-court decisions on the data-quality batch.
fn dataquality_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(FD_VIOLATION_RESERVED_CANONICAL_ID),
            reason: "Functional dependency violation: declared determinant columns + \
                 dependent columns + null handling + duplicate-row handling + \
                 violation count law + minimum support. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(TYPE_INSTABILITY_RESERVED_CANONICAL_ID),
            reason: "Type instability: declared type system (SQL / Arrow / Parquet / \
                 runtime types) + observed-vs-expected mismatch law + scope + \
                 threshold. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(TARGET_LEAKAGE_RESERVED_CANONICAL_ID),
            reason: "Target-leakage candidate: declared target column + time / order \
                 law + feature-availability time + association law (correlation / \
                 mutual-information / conditional-mutual-information) + train / test \
                 split or temporal holdout + threshold. Panel-locked non-claim: \
                 candidate, not proof. Future activation planner / case-file emitter \
                 must NOT promote this to a ratified leakage verdict.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            reason: "Correlation break (inter-column): declared correlation convention \
                 (Pearson / Spearman / Kendall tau) + column pair (or pair set) + \
                 window pair + normalization + threshold. Distinct from SEED 40 \
                 Autocorrelation-coefficient break (intra-column lag-based).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(COVARIANCE_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Covariance shift (multivariate): declared column set + \
                 covariance estimator (sample / shrinkage / robust) + comparison law \
                 (Frobenius distance / log-determinant ratio / affine-invariant \
                 Riemannian) + threshold. Generalises Correlation break to the full \
                 multivariate covariance structure.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(NULL_RUN_RESERVED_CANONICAL_ID),
            reason: "Null-run anomaly: declared null semantics (NULL / NaN / empty- \
                 string / sentinel-value) + consecutive-null-run law (max-run / \
                 current-run / percentile-run) + threshold. Structurally distinct \
                 from Missingness spike (window-count) and from Run-length anomaly \
                 (T.12.f 5606; general event class).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(RANGE_ENVELOPE_RESERVED_CANONICAL_ID),
            reason: "Range envelope exit (tabular): declared per-column min / max \
                 bounds + unit + inclusive vs exclusive boundary semantics + null \
                 handling + tie-break at boundary. Structurally distinct from SEED \
                 22 Residual envelope exit (residual-magnitude-based) - tabular \
                 range envelope is per-column bound-based against declared contract \
                 values.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID),
            reason: "Category emergence: declared reference-category set (anchor: \
                 snapshot-hash / pinned-fixture / sliding-window) + new-category- \
                 appearance law (first-seen / first-N-occurrences) + case \
                 sensitivity + unknown-category handling + threshold. Distinct from \
                 SEED 46 Cardinality drift (count-of-distinct shift) - category \
                 emergence tracks IDENTITY of new categories.",
        },
        // -- 5 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            reason: "Missingness spike stays canonical at SEED id 13. Declared null \
                 semantics (NULL / NaN / empty-string / sentinel) + per-column or \
                 row-wise scope + baseline window + threshold + comparison law. No \
                 duplicate admitted; per-column missingness (record 5809 below) \
                 collapses here as ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MISSINGNESS_COUPLING_SEED_ID),
            reason: "Missingness coupling stays canonical at SEED id 44. Declared \
                 multi-column null-pattern coupling + scope + baseline + threshold. \
                 No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SCHEMA_DRIFT_SEED_ID),
            reason: "Schema drift stays canonical at SEED id 45. Declared schema- \
                 version or column-identity law (added / removed / renamed / type \
                 change / unit change / nullability change subkinds) + scope + \
                 threshold. No duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            reason: "Cardinality drift stays canonical at SEED id 46. Declared \
                 category-identity law (case sensitivity + unknown-category \
                 handling) + counting law + reference window + threshold. No \
                 duplicate admitted; category collapse (record 5811 below) \
                 collapses here as ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(UNIQUENESS_VIOLATION_SEED_ID),
            reason: "Uniqueness violation stays canonical at SEED id 47. Declared \
                 key scope (single-column or composite-key tuple) + null handling \
                 (NULL = distinct / NULL = same / quarantine) + duplicate-row \
                 handling + violation-count law. No duplicate admitted; composite- \
                 key uniqueness (record 5810 below) collapses here as \
                 ParameterizationOf.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            reason: "Missingness spike (SEED id 13) is the shared data-quality \
                 ancestor for the DataQualityRules source class. The court records \
                 the domain transfer without re-canonicalising Missingness spike.",
        },
        // -- 3 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID),
            reason: "Per-column missingness is ParameterizationOf(Missingness spike, \
                 SEED id 13). Scope parameterized to a single declared column. The \
                 court declines to admit per-column missingness as a new canonical \
                 primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID),
            reason: "Composite-key uniqueness is ParameterizationOf(Uniqueness \
                 violation, SEED id 47). Key scope parameterized to a declared \
                 column-tuple. The court declines to admit composite-key uniqueness \
                 as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID),
            reason: "Category collapse is ParameterizationOf(Cardinality drift, \
                 SEED id 46). Direction-specific parameterization tracking category \
                 disappearance (count-of-distinct DECREASES). The court declines to \
                 admit category collapse as a new canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID),
            reason: "Learned data-quality anomaly score (autoencoder reconstruction, \
                 Mahalanobis with learned covariance, Isolation Forest with random \
                 seed / random feature sample, LOF with learned k-NN graph) is \
                 randomized / learned in origin. Rejected unless reduced to a \
                 declared deterministic proxy (model-identification seed + training- \
                 data anchor pinned-fixture-hash + feature schema pinned + tie-break \
                 law + numeric mode all brutally explicit) in a later T.12.x.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID),
            reason: "Auto-schema inference anomaly (TFDV-style auto-detected schema + \
                 constraint-inference, Great Expectations profiler with random \
                 sampling) produces different rules per run when inference is \
                 unspecified. Rejected unless reduced to a declared deterministic \
                 proxy (inference algorithm + sample seed + sampling schedule + \
                 schema-version anchor + tie-break law all brutally explicit) in a \
                 later T.12.x.",
        },
    ]
}

/// Ten genealogy edges proposed for the post-freeze graph.
fn dataquality_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FD_VIOLATION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(UNIQUENESS_VIOLATION_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(TYPE_INSTABILITY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SCHEMA_DRIFT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COVARIANCE_SHIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(NULL_RUN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RANGE_ENVELOPE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(UNIQUENESS_VIOLATION_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the data-quality expansion.
fn dataquality_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "codd_relational_1972",
            title: "Further Normalization of the Data Base Relational Model",
            year: 1972,
            venue: "IBM Research Report RJ909 (functional-dependency origin)",
        },
        ProposedSourceRef {
            citation_key: "date_database_textbook_2003",
            title: "An Introduction to Database Systems (8th ed.)",
            year: 2003,
            venue: "Addison-Wesley (FD / key / integrity-constraint reference)",
        },
        ProposedSourceRef {
            citation_key: "pearson_correlation_1895",
            title: "Notes on Regression and Inheritance in the Case of Two Parents",
            year: 1895,
            venue: "Proceedings of the Royal Society 58 (Pearson correlation)",
        },
        ProposedSourceRef {
            citation_key: "spearman_rank_correlation_1904",
            title: "The Proof and Measurement of Association between Two Things",
            year: 1904,
            venue: "American Journal of Psychology 15(1) (Spearman rank correlation)",
        },
        ProposedSourceRef {
            citation_key: "mahalanobis_covariance_1936",
            title: "On the Generalised Distance in Statistics",
            year: 1936,
            venue: "Proceedings of the National Institute of Sciences of India 2(1)",
        },
        ProposedSourceRef {
            citation_key: "kaufman_target_leakage_2012",
            title: "Leakage in Data Mining: Formulation, Detection, and Avoidance",
            year: 2012,
            venue: "ACM TKDD 6(4) (target-leakage candidate framework)",
        },
        ProposedSourceRef {
            citation_key: "breck_tfdv_2019",
            title: "Data Validation for Machine Learning",
            year: 2019,
            venue: "MLSys 2019 (TFDV; rejection-shell reference for auto-schema inference)",
        },
        ProposedSourceRef {
            citation_key: "great_expectations_2018",
            title: "Great Expectations: An Open-Source Tool for Data Pipeline Testing",
            year: 2018,
            venue: "Project website (data-quality rule profiler)",
        },
        ProposedSourceRef {
            citation_key: "liu_isolation_forest_2008",
            title: "Isolation Forest (rejection-shell reference; randomized in origin)",
            year: 2008,
            venue: "ICDM 2008",
        },
    ]
}

/// Build the data-quality `DedupCourtDelta`.
fn build_dataquality_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_h_dataquality_delta",
        vec![
            DetectorCanonicalId(FD_VIOLATION_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(TYPE_INSTABILITY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(TARGET_LEAKAGE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CORRELATION_BREAK_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(COVARIANCE_SHIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(NULL_RUN_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(RANGE_ENVELOPE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID),
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

/// Build the T.12.h data-quality `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_h_dataquality_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_h_dataquality_first_proposal",
        "T.12.h files the Data Quality / Tabular / Database Integrity amendment \
         proposal. Adds eight genuinely new canonical data-quality primitives \
         (functional dependency violation, type instability, target-leakage \
         candidate, correlation break, covariance shift, null-run anomaly, range \
         envelope exit tabular, category emergence) at reserved canonical ids \
         5801..=5808 with declared scope + baseline + null-semantics + key-scope + \
         type-system + range / unit + association-law + decision-law contracts. \
         Records five ExistingCanonicalAuthorityResolution decisions keeping \
         Missingness spike (SEED id 13), Missingness coupling (id 44), Schema drift \
         (id 45), Cardinality drift (id 46), Uniqueness violation (id 47) \
         canonical under the DataQualityRules source class without duplication. \
         Records one DomainTransferOf decision naming Missingness spike as the \
         shared data-quality ancestor. Records three ParameterizationOf decisions: \
         per-column missingness is ParameterizationOf(Missingness spike); \
         composite-key uniqueness is ParameterizationOf(Uniqueness violation); \
         category collapse is ParameterizationOf(Cardinality drift). Rejects TWO \
         data-quality literature records as RejectedNotDeterministic (second \
         T.12.x proposal with two rejection records in one commit, following \
         T.12.g): learned data-quality anomaly score (5812 - autoencoder / \
         Mahalanobis-with-learned-cov / Isolation Forest / LOF) and auto-schema \
         inference anomaly (5813 - TFDV-style auto-detection, Great Expectations \
         profiler with random sampling). Target-leakage candidate (5803) is \
         admitted under the panel-locked non-claim: candidate witness, not proof \
         of leakage. Every record's reason text declares its specific scope + \
         decision-law contract - the panel-locked warning was 'a validation rule \
         is not a detector until scope, baseline, null / type / key semantics, \
         and decision law are declared'. Does NOT mutate SEED (SEED.len() stays \
         at 54); status = Open pending review.",
        SourceClass::DataQualityRules,
        build_dataquality_expansion_batch(),
        build_dataquality_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_h_dataquality",
    )
}
