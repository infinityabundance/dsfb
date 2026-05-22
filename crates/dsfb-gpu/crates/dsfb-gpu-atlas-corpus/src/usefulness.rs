//! T.8 — deterministic detector usefulness ledger shell.
//!
//! **Thesis (panel-locked)**:
//!
//! > The Atlas does not learn detector usefulness. It records
//! > detector usefulness as deterministic, source-bound,
//! > task-bound ledger evidence. Until a detector has measured
//! > task deltas, its usefulness row remains `Unmeasured` /
//! > `LiteraturePrior` / `RoleSeeded`.
//!
//! T.8 builds the **ledger court**, not its trial history. **Do
//! NOT fabricate empirical numbers.** Every empirical field on
//! every conservative-seed row is zero. The ledger gains
//! measured rows only when a real benchmark artifact backs them.
//!
//! **Schema-evolution decision (user-locked)**: T.8 is additive.
//!
//! - The existing per-detector summary
//!   [`crate::types::UsefulnessLedgerSnapshot`] (renamed from
//!   `UsefulnessLedgerRow` in T.8) stays embedded inside every
//!   [`crate::types::LiteratureDetector`] as a zero-init prior
//!   summary. Field layout and field types are byte-identical to
//!   T.1a-T.7.
//! - This module owns the richer
//!   [`UsefulnessLedgerRow`] keyed by
//!   `(canonical_id, task_id, domain, dataset_id)` plus the
//!   [`UsefulnessEvidenceLevel`] honesty ladder.
//! - [`USEFULNESS_LEDGER`] is a separate `pub static` collection
//!   alongside [`crate::seed::SEED`]; one row per canonical
//!   detector at T.8.
//!
//! **The ledger is an audit surface, not a learned ranking model.**
//! T.8 records declared evidence levels and conservative
//! contribution fields; empirical usefulness remains unclaimed
//! until a row is backed by a named benchmark artifact.
//!
//! T.8 explicitly does NOT do:
//!
//! - No empirical usefulness claims (every conservative seed row
//!   stays `Unmeasured` / `LiteraturePrior` / `RoleSeeded` with
//!   all empirical fields zero and `sample_count = 0`).
//! - No detector ranking (the deterministic
//!   [`usefulness_score`] policy returns `None` on every row
//!   carrying `UsefulnessScoreKind::NotScored`).
//! - No activation planner (that belongs to Section S Phase 1+).
//! - No `corpus_hash_v1` (T.10).
//! - No `CaseFileV2` integration (T.11).
//! - No L8 row in the seed (forbidden until a measured artifact
//!   exists; the verifier enforces this).
//! - No `Retired*` row in the seed (forbidden until measured
//!   negative evidence exists).
//! - No GPU code changes.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::lband::GPU_IMPLEMENTED_CANONICAL_IDS;
use crate::types::{
    DetectorCanonicalId, DomainTagSet, ImplementationLevel, LifecycleState, LiteratureDetector,
};

// ===================================================================
// Newtypes for task and dataset provenance.
// ===================================================================

/// Stable identifier for a task the ledger row applies to.
///
/// Tasks are coarse categories — "audit-replay determinism check",
/// "courthouse-factory throughput sweep", "external Adbench
/// fixture run" — so the same detector can have separate ledger
/// rows for separate tasks without losing identity.
///
/// At T.8 every seed row uses `"atlas_corpus_seed_v1"`. Later
/// commits introduce additional task IDs when a real benchmark
/// run populates measured rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub &'static str);

/// Stable identifier for the dataset a ledger row was measured on.
///
/// Datasets are independent of tasks — the same dataset can feed
/// multiple tasks. At T.8 every seed row uses `"none"` (no real
/// dataset has been measured against). Later commits introduce
/// real dataset IDs when an external fixture is hashed and pinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DatasetId(pub &'static str);

// ===================================================================
// The evidence-level ladder.
// ===================================================================

/// How well-justified the empirical fields in a ledger row are.
///
/// The ladder is load-bearing: it prevents fake empirical claims.
/// Rows at `Unmeasured` / `LiteraturePrior` / `RoleSeeded` MUST
/// keep every empirical field zero and `sample_count = 0`; the
/// verifier rejects any nonzero empirical fact below
/// `SyntheticFixtureMeasured`.
///
/// Variants (in ascending evidence-strength order):
///
/// - `Unmeasured` — no claim is being made about this detector
///   for this triple yet. The default conservative state.
/// - `LiteraturePrior` — citation-only evidence (the detector
///   appears in the literature under this domain). No
///   measurement.
/// - `RoleSeeded` — the detector is admitted on role / fusion-plane
///   grounds (confusers, GPU-bank-surface entries) without
///   empirical task evidence. No measurement.
/// - `SyntheticFixtureMeasured` — measured against a synthetic
///   fixture whose bytes are pinned and reproducible.
/// - `RealDatasetMeasured` — measured against a real-world
///   dataset whose bytes are hashed and pinned.
/// - `CrossDomainReplicated` — measured across two or more
///   independent domains with consistent results.
/// - `RetiredByEvidence` — measured negative evidence has retired
///   this detector for this triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsefulnessEvidenceLevel {
    /// No claim. Empirical fields MUST be zero.
    Unmeasured,
    /// Citation-only evidence. Empirical fields MUST be zero.
    LiteraturePrior,
    /// Role / fusion-plane admittance. Empirical fields MUST be zero.
    RoleSeeded,
    /// Measured against a pinned synthetic fixture.
    SyntheticFixtureMeasured,
    /// Measured against a hashed real-world dataset.
    RealDatasetMeasured,
    /// Measured across two or more independent domains.
    CrossDomainReplicated,
    /// Measured negative evidence retired this detector.
    RetiredByEvidence,
}

impl UsefulnessEvidenceLevel {
    /// True if this level forbids any nonzero empirical claim. The
    /// verifier uses this to reject Unmeasured/LiteraturePrior/
    /// RoleSeeded rows that carry fabricated numbers.
    #[must_use]
    pub const fn forbids_empirical_claims(self) -> bool {
        matches!(
            self,
            Self::Unmeasured | Self::LiteraturePrior | Self::RoleSeeded
        )
    }

    /// Canonical wire name for the report and for any future TOML
    /// ingestion path.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unmeasured => "Unmeasured",
            Self::LiteraturePrior => "LiteraturePrior",
            Self::RoleSeeded => "RoleSeeded",
            Self::SyntheticFixtureMeasured => "SyntheticFixtureMeasured",
            Self::RealDatasetMeasured => "RealDatasetMeasured",
            Self::CrossDomainReplicated => "CrossDomainReplicated",
            Self::RetiredByEvidence => "RetiredByEvidence",
        }
    }

    /// Canonical ordering for histogram display.
    #[must_use]
    pub const fn all() -> [UsefulnessEvidenceLevel; 7] {
        [
            Self::Unmeasured,
            Self::LiteraturePrior,
            Self::RoleSeeded,
            Self::SyntheticFixtureMeasured,
            Self::RealDatasetMeasured,
            Self::CrossDomainReplicated,
            Self::RetiredByEvidence,
        ]
    }
}

// ===================================================================
// Score-kind gate.
// ===================================================================

/// Whether a ledger row carries a usable usefulness score.
///
/// `NotScored` blocks [`usefulness_score`] from returning a
/// numeric score; a `PriorScore` row carries a deterministic
/// policy score derived from citation / role evidence only; a
/// `MeasuredScore` row carries a score backed by real
/// `evidence_level >= SyntheticFixtureMeasured` measurements.
///
/// The T.8 seed uses `NotScored` exclusively — no row scores
/// itself before evidence backs the fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsefulnessScoreKind {
    /// No score available. Score formula returns `None`.
    NotScored,
    /// Score derived from prior (citation / role) evidence only.
    PriorScore,
    /// Score derived from measured fields.
    MeasuredScore,
}

impl UsefulnessScoreKind {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotScored => "NotScored",
            Self::PriorScore => "PriorScore",
            Self::MeasuredScore => "MeasuredScore",
        }
    }
}

// ===================================================================
// Per-row reason code.
// ===================================================================

/// Why a ledger row is at its declared evidence level / lifecycle.
///
/// Every row carries exactly one. The verifier rejects any row
/// whose reason does not match its `evidence_level` (e.g.
/// `MeasuredFromSyntheticFixture` paired with
/// `evidence_level = Unmeasured` is incoherent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsefulnessReason {
    /// Conservative default: no claim, no measurement, no role.
    UnmeasuredAtT8,
    /// Citation-only evidence — the detector appears in the
    /// literature under this domain but has not been measured.
    LiteraturePriorOnly,
    /// Role-seeded — the detector is admitted on fusion-role
    /// grounds (e.g. confuser / negative witness) without
    /// empirical task evidence.
    RoleSeededOnly,
    /// Role-seeded specifically by the existing dsfb-gpu-debug-core
    /// kernel surface (canonical IDs 14, 15, 41, 42, 43).
    GpuSurfaceSeededFromDsfbGpuDebugCore,
    /// Measured against a pinned synthetic fixture.
    MeasuredFromSyntheticFixture,
    /// Measured against a hashed real-world dataset.
    MeasuredFromRealDataset,
    /// Measured across two or more independent domains.
    MeasuredCrossDomainReplicated,
    /// Retired because measured evidence shows the detector is
    /// redundant with others for this triple.
    RetiredRedundantByEvidence,
    /// Retired because measured evidence shows the detector
    /// produces too many false positives.
    RetiredHighFpByEvidence,
    /// Retired because measured runtime cost exceeds the policy
    /// budget.
    RetiredTooExpensiveByEvidence,
    /// Quarantined because the measurement is unstable.
    QuarantinedByUnstableMeasurement,
    /// Resurrected for a specific domain after a measured
    /// domain-relevance result.
    ResurrectedForDomainByEvidence,
}

impl UsefulnessReason {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnmeasuredAtT8 => "UnmeasuredAtT8",
            Self::LiteraturePriorOnly => "LiteraturePriorOnly",
            Self::RoleSeededOnly => "RoleSeededOnly",
            Self::GpuSurfaceSeededFromDsfbGpuDebugCore => "GpuSurfaceSeededFromDsfbGpuDebugCore",
            Self::MeasuredFromSyntheticFixture => "MeasuredFromSyntheticFixture",
            Self::MeasuredFromRealDataset => "MeasuredFromRealDataset",
            Self::MeasuredCrossDomainReplicated => "MeasuredCrossDomainReplicated",
            Self::RetiredRedundantByEvidence => "RetiredRedundantByEvidence",
            Self::RetiredHighFpByEvidence => "RetiredHighFpByEvidence",
            Self::RetiredTooExpensiveByEvidence => "RetiredTooExpensiveByEvidence",
            Self::QuarantinedByUnstableMeasurement => "QuarantinedByUnstableMeasurement",
            Self::ResurrectedForDomainByEvidence => "ResurrectedForDomainByEvidence",
        }
    }

    /// Returns true if the reason is consistent with the given
    /// evidence level. The verifier uses this for rule 7.
    #[must_use]
    pub fn is_consistent_with(self, level: UsefulnessEvidenceLevel) -> bool {
        use UsefulnessEvidenceLevel as Lvl;
        match self {
            Self::UnmeasuredAtT8 => level == Lvl::Unmeasured,
            Self::LiteraturePriorOnly => level == Lvl::LiteraturePrior,
            Self::RoleSeededOnly | Self::GpuSurfaceSeededFromDsfbGpuDebugCore => {
                level == Lvl::RoleSeeded
            }
            Self::MeasuredFromSyntheticFixture => level == Lvl::SyntheticFixtureMeasured,
            Self::MeasuredFromRealDataset => level == Lvl::RealDatasetMeasured,
            Self::MeasuredCrossDomainReplicated => level == Lvl::CrossDomainReplicated,
            Self::RetiredRedundantByEvidence
            | Self::RetiredHighFpByEvidence
            | Self::RetiredTooExpensiveByEvidence => level == Lvl::RetiredByEvidence,
            Self::QuarantinedByUnstableMeasurement | Self::ResurrectedForDomainByEvidence => true,
        }
    }
}

// ===================================================================
// Ledger-source provenance.
// ===================================================================

/// Where the ledger row was produced — the corpus seed, the
/// dsfb-gpu-debug-core kernel surface, or a named external
/// benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LedgerSource {
    /// Row was synthesised by the T.8 conservative corpus seed.
    AtlasCorpusSeedV1,
    /// Row reflects the existing dsfb-gpu-debug-core kernel
    /// surface (canonical IDs 14, 15, 41, 42, 43).
    DsfbGpuDebugCoreSurface,
    /// Row was produced by an external named benchmark (e.g. an
    /// R.12b saturation re-run).
    ExternalBenchmark(&'static str),
}

impl LedgerSource {
    /// Canonical wire name (for the report and any future TOML
    /// dump). External-benchmark entries embed their name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AtlasCorpusSeedV1 => "AtlasCorpusSeedV1",
            Self::DsfbGpuDebugCoreSurface => "DsfbGpuDebugCoreSurface",
            Self::ExternalBenchmark(name) => name,
        }
    }
}

// ===================================================================
// The richer ledger row.
// ===================================================================

/// One deterministic ledger row keyed by
/// `(canonical_id, task_id, domain, dataset_id)`.
///
/// Multiple rows per detector are allowed when measured evidence
/// arrives for distinct (task, dataset) triples.
///
/// Why these fields: the row is a court record, not a learned
/// weight. Every field has provenance. The empirical fields
/// (`unique_episode_gain`, `clean_window_false_positive_cost`,
/// etc.) are zero at T.8 because no real benchmark has populated
/// them yet; the [`UsefulnessEvidenceLevel`] gate prevents any
/// row from claiming a nonzero empirical fact below
/// `SyntheticFixtureMeasured`.
#[derive(Debug, Clone, Copy)]
pub struct UsefulnessLedgerRow {
    /// Which detector this row applies to.
    pub canonical_id: DetectorCanonicalId,
    /// Coarse task category this row was measured under.
    pub task_id: TaskId,
    /// Domain bitset (one or more bits set) the row applies to.
    /// At T.8 every seed row carries a single-bit domain derived
    /// from the detector's `origin_domains` field.
    pub domain: DomainTagSet,
    /// Pinned dataset identifier; `"none"` at T.8.
    pub dataset_id: DatasetId,
    /// Evidence ladder level — see [`UsefulnessEvidenceLevel`].
    pub evidence_level: UsefulnessEvidenceLevel,
    /// Lifecycle state for this row's triple — Active means the
    /// detector contributes to fusion for this task / domain /
    /// dataset; Dormant means it is not contributing yet but
    /// remains in the corpus; Retired*/Quarantined require
    /// measured negative evidence.
    pub lifecycle_state: LifecycleState,
    /// Whether the row carries a usable score.
    pub score_kind: UsefulnessScoreKind,

    /// Empirical: number of unique admissions this detector
    /// contributed across the recorded sample. MUST be 0 unless
    /// `evidence_level >= SyntheticFixtureMeasured`.
    pub unique_episode_gain: i64,
    /// Empirical: how many other detectors fire on the same
    /// admissions. MUST be 0 unless measured.
    pub redundant_with_count: u32,
    /// Empirical: false-positive cost on clean windows. MUST be
    /// 0 unless measured.
    pub clean_window_false_positive_cost: i64,
    /// Empirical: confuser-suppression contribution. MUST be 0
    /// unless measured.
    pub confuser_reduction_gain: i64,
    /// Empirical: p50 runtime cost in microseconds. MUST be 0
    /// unless measured.
    pub runtime_cost_us_p50: u32,
    /// Empirical: peak per-cell working set in bytes. MUST be 0
    /// unless measured.
    pub memory_cost_bytes: u64,
    /// Hand-rated: how legible the detector's contribution is in
    /// the case-file explanation. Allowed nonzero only when
    /// `score_kind != NotScored`.
    pub casefile_explanation_value: i32,
    /// Hand-rated: how readable the detector's name + reason code
    /// is for an operator. Allowed nonzero only when
    /// `score_kind != NotScored`.
    pub operator_readability_score: i32,
    /// How many measurement runs this row aggregates. MUST be 0
    /// unless measured.
    pub sample_count: u64,

    /// Provenance: which source produced this row.
    pub ledger_source: LedgerSource,
    /// Why this row is at its declared evidence level.
    pub reason_code: UsefulnessReason,
}

impl UsefulnessLedgerRow {
    /// True if every empirical field is zero (the canonical
    /// shape of a conservative seed row at T.8).
    #[must_use]
    pub const fn has_zero_empirical_fields(&self) -> bool {
        self.unique_episode_gain == 0
            && self.redundant_with_count == 0
            && self.clean_window_false_positive_cost == 0
            && self.confuser_reduction_gain == 0
            && self.runtime_cost_us_p50 == 0
            && self.memory_cost_bytes == 0
            && self.casefile_explanation_value == 0
            && self.operator_readability_score == 0
            && self.sample_count == 0
    }

    /// Canonical sort key for deterministic ledger ordering.
    /// The verifier uses this to assert two builds produce the
    /// same row order; the report uses it to render histograms.
    #[must_use]
    pub fn sort_key(&self) -> (u32, &'static str, u16, &'static str) {
        (
            self.canonical_id.0,
            self.task_id.0,
            self.domain.0,
            self.dataset_id.0,
        )
    }
}

// ===================================================================
// Deterministic score policy.
// ===================================================================

/// Compute a deterministic usefulness score for the row, or
/// `None` if `score_kind == NotScored`.
///
/// The formula is policy, not learning. Integer arithmetic only;
/// runtime cost is bucketed into 1024-µs chunks so the score is
/// host-independent.
///
/// `score = 4 * unique_episode_gain + 3 * confuser_reduction_gain`
/// `      + 2 * casefile_explanation_value + 1 * operator_readability_score`
/// `      - 3 * clean_window_false_positive_cost - 1 * runtime_bucket`
/// `      - 2 * redundant_with_count`
#[must_use]
pub fn usefulness_score(row: &UsefulnessLedgerRow) -> Option<i64> {
    match row.score_kind {
        UsefulnessScoreKind::NotScored => None,
        UsefulnessScoreKind::PriorScore | UsefulnessScoreKind::MeasuredScore => {
            let runtime_bucket = i64::from(row.runtime_cost_us_p50 / 1024);
            Some(
                4 * row.unique_episode_gain
                    + 3 * row.confuser_reduction_gain
                    + 2 * i64::from(row.casefile_explanation_value)
                    + i64::from(row.operator_readability_score)
                    - 3 * row.clean_window_false_positive_cost
                    - runtime_bucket
                    - 2 * i64::from(row.redundant_with_count),
            )
        }
    }
}

// ===================================================================
// Verifier.
// ===================================================================

/// One failure record from the T.8 verifier. Carries the
/// canonical_id, task_id, dataset_id triple plus a structured
/// kind so callers can dispatch on rule violations programmatically.
#[derive(Debug, Clone)]
pub struct UsefulnessLedgerError {
    /// Detector the failing row points at (or
    /// `DetectorCanonicalId(0)` for coverage errors that name no
    /// specific row).
    pub canonical_id: DetectorCanonicalId,
    /// Triple's task ID (or `TaskId("")` for coverage errors).
    pub task_id: TaskId,
    /// Triple's dataset ID (or `DatasetId("")` for coverage errors).
    pub dataset_id: DatasetId,
    /// Structured failure category.
    pub kind: UsefulnessLedgerErrorKind,
}

/// Structured failure category. The verifier rules below produce
/// these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsefulnessLedgerErrorKind {
    /// Rule 1: a canonical detector in `SEED` has no row in the
    /// ledger.
    DetectorMissingLedgerRow,
    /// Rule 2: a row at `Unmeasured`/`LiteraturePrior`/`RoleSeeded`
    /// claims a nonzero empirical field.
    UnmeasuredRowClaimsEmpiricalGain,
    /// Rule 3: the detector's L-band is L8 but no row reaches
    /// `RealDatasetMeasured` or higher.
    L8RecordWithoutMeasuredLedgerEvidence,
    /// Rule 4: a row carries a `Retired*` lifecycle without
    /// matching measured evidence.
    RetiredStateWithoutMeasuredEvidence,
    /// Rule 5: a row claims `GpuSurfaceSeededFromDsfbGpuDebugCore`
    /// but the canonical_id is not in the whitelist OR the
    /// detector's L-band is not L5/L6.
    GpuActiveClaimWithoutWhitelistOrLBand,
    /// Rule 6: `score_kind = NotScored` but `usefulness_score`
    /// returns `Some`.
    NotScoredButScoreReturnedSome,
    /// Rule 7: reason / evidence inconsistency.
    ReasonInconsistentWithEvidenceLevel,
    /// Rule 8: the same (canonical_id, task_id, dataset_id)
    /// triple appears with both `Active` and a `Retired*`
    /// lifecycle.
    SameTripleBothActiveAndRetired,
    /// Rule 9: a required field is missing (e.g. empty task_id /
    /// dataset_id, empty domain bitset).
    RequiredFieldMissing,
    /// Rule 10: two rows share the same
    /// (canonical_id, task_id, domain, dataset_id) tuple.
    DuplicateTriple,
    /// Rule 11: the row's canonical_id does not resolve to a
    /// record in `SEED`.
    UnknownDetectorId,
}

impl UsefulnessLedgerErrorKind {
    /// Human-readable description for the report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::DetectorMissingLedgerRow => String::from(
                "canonical detector has no row in the usefulness ledger",
            ),
            Self::UnmeasuredRowClaimsEmpiricalGain => String::from(
                "row at evidence_level Unmeasured/LiteraturePrior/RoleSeeded carries a nonzero empirical field; only SyntheticFixtureMeasured or higher may claim empirical numbers",
            ),
            Self::L8RecordWithoutMeasuredLedgerEvidence => String::from(
                "detector L-band is L8_LedgerCharacterised but no row reaches RealDatasetMeasured / CrossDomainReplicated",
            ),
            Self::RetiredStateWithoutMeasuredEvidence => String::from(
                "row carries a Retired*/Quarantined lifecycle but evidence_level < SyntheticFixtureMeasured or reason is not the matching Retired*ByEvidence",
            ),
            Self::GpuActiveClaimWithoutWhitelistOrLBand => String::from(
                "row claims GpuSurfaceSeededFromDsfbGpuDebugCore but canonical_id is not in GPU_IMPLEMENTED_CANONICAL_IDS or detector L-band is not L5/L6",
            ),
            Self::NotScoredButScoreReturnedSome => String::from(
                "row has score_kind = NotScored but usefulness_score returned Some(...)",
            ),
            Self::ReasonInconsistentWithEvidenceLevel => String::from(
                "row's reason_code is not consistent with its evidence_level (see UsefulnessReason::is_consistent_with)",
            ),
            Self::SameTripleBothActiveAndRetired => String::from(
                "same (canonical_id, task_id, dataset_id) triple appears with both Active and a Retired*/Quarantined lifecycle",
            ),
            Self::RequiredFieldMissing => String::from(
                "row has an empty task_id, dataset_id, or domain bitset",
            ),
            Self::DuplicateTriple => String::from(
                "two rows share the same (canonical_id, task_id, domain, dataset_id) tuple",
            ),
            Self::UnknownDetectorId => String::from(
                "row's canonical_id does not resolve to a record in SEED",
            ),
        }
    }
}

/// Aggregate verifier report.
#[derive(Debug, Clone, Default)]
pub struct UsefulnessLedgerVerifyReport {
    /// Records inspected (the SEED slice length).
    pub records_inspected: usize,
    /// Ledger rows inspected.
    pub rows_inspected: usize,
    /// Per-failure errors (empty when clean).
    pub errors: Vec<UsefulnessLedgerError>,
}

impl UsefulnessLedgerVerifyReport {
    /// True if no errors were recorded.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Walk `rows` against `records` and emit any rule violations.
///
/// The verifier is deterministic: two calls with the same inputs
/// produce the same error sequence in the same order. Errors
/// from a single row appear in rule-number ascending order so a
/// failure report is reviewable.
///
/// The function is intentionally one large rule-by-rule walker
/// so a reviewer can read every panel-locked rule in source
/// order without jumping helpers. The `too_many_lines` clippy
/// warning is locally allowed for that reason.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn verify_usefulness_ledger(
    records: &[LiteratureDetector],
    rows: &[UsefulnessLedgerRow],
) -> UsefulnessLedgerVerifyReport {
    let mut report = UsefulnessLedgerVerifyReport {
        records_inspected: records.len(),
        rows_inspected: rows.len(),
        errors: Vec::new(),
    };

    // Rule 11 + 1: every row's canonical_id resolves; every
    // canonical detector has at least one row.
    for r in rows {
        if !records.iter().any(|d| d.canonical_id == r.canonical_id) {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: r.canonical_id,
                task_id: r.task_id,
                dataset_id: r.dataset_id,
                kind: UsefulnessLedgerErrorKind::UnknownDetectorId,
            });
        }
    }
    for d in records {
        if !rows.iter().any(|r| r.canonical_id == d.canonical_id) {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: d.canonical_id,
                task_id: TaskId(""),
                dataset_id: DatasetId(""),
                kind: UsefulnessLedgerErrorKind::DetectorMissingLedgerRow,
            });
        }
    }

    // Rule 10: unique within triple.
    for (i, a) in rows.iter().enumerate() {
        for b in rows.iter().skip(i + 1) {
            if a.canonical_id == b.canonical_id
                && a.task_id == b.task_id
                && a.domain == b.domain
                && a.dataset_id == b.dataset_id
            {
                report.errors.push(UsefulnessLedgerError {
                    canonical_id: a.canonical_id,
                    task_id: a.task_id,
                    dataset_id: a.dataset_id,
                    kind: UsefulnessLedgerErrorKind::DuplicateTriple,
                });
            }
        }
    }

    // Rules 2 / 5 / 6 / 7 / 9 — per-row checks.
    for r in rows {
        // Rule 9: required fields. Empty task_id, dataset_id, or
        // domain bitset is forbidden.
        if r.task_id.0.is_empty() || r.dataset_id.0.is_empty() || r.domain.is_empty() {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: r.canonical_id,
                task_id: r.task_id,
                dataset_id: r.dataset_id,
                kind: UsefulnessLedgerErrorKind::RequiredFieldMissing,
            });
        }

        // Rule 2: unmeasured rows cannot claim empirical numbers.
        if r.evidence_level.forbids_empirical_claims() && !r.has_zero_empirical_fields() {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: r.canonical_id,
                task_id: r.task_id,
                dataset_id: r.dataset_id,
                kind: UsefulnessLedgerErrorKind::UnmeasuredRowClaimsEmpiricalGain,
            });
        }

        // Rule 5: GPU-surface reason requires whitelist + L5/L6.
        if r.reason_code == UsefulnessReason::GpuSurfaceSeededFromDsfbGpuDebugCore {
            let in_whitelist = GPU_IMPLEMENTED_CANONICAL_IDS.contains(&r.canonical_id);
            let lband_ok = records
                .iter()
                .find(|d| d.canonical_id == r.canonical_id)
                .is_some_and(|d| {
                    matches!(
                        d.implementation_status,
                        ImplementationLevel::L5_GpuImplemented
                            | ImplementationLevel::L6_CpuGpuByteEquivalent
                    )
                });
            if !in_whitelist || !lband_ok {
                report.errors.push(UsefulnessLedgerError {
                    canonical_id: r.canonical_id,
                    task_id: r.task_id,
                    dataset_id: r.dataset_id,
                    kind: UsefulnessLedgerErrorKind::GpuActiveClaimWithoutWhitelistOrLBand,
                });
            }
        }

        // Rule 6: NotScored blocks nonzero score.
        if r.score_kind == UsefulnessScoreKind::NotScored && usefulness_score(r).is_some() {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: r.canonical_id,
                task_id: r.task_id,
                dataset_id: r.dataset_id,
                kind: UsefulnessLedgerErrorKind::NotScoredButScoreReturnedSome,
            });
        }

        // Rule 7: reason / evidence consistency.
        if !r.reason_code.is_consistent_with(r.evidence_level) {
            report.errors.push(UsefulnessLedgerError {
                canonical_id: r.canonical_id,
                task_id: r.task_id,
                dataset_id: r.dataset_id,
                kind: UsefulnessLedgerErrorKind::ReasonInconsistentWithEvidenceLevel,
            });
        }

        // Rule 4: Retired*/Quarantined requires measured evidence.
        let is_retired_or_quarantined = matches!(
            r.lifecycle_state,
            LifecycleState::RetiredRedundant
                | LifecycleState::RetiredHighFalsePositive
                | LifecycleState::RetiredTooExpensive
                | LifecycleState::QuarantinedUnstable
        );
        if is_retired_or_quarantined {
            let measured = !r.evidence_level.forbids_empirical_claims()
                && r.evidence_level != UsefulnessEvidenceLevel::RetiredByEvidence
                || matches!(
                    r.reason_code,
                    UsefulnessReason::RetiredRedundantByEvidence
                        | UsefulnessReason::RetiredHighFpByEvidence
                        | UsefulnessReason::RetiredTooExpensiveByEvidence
                        | UsefulnessReason::QuarantinedByUnstableMeasurement
                );
            if !measured {
                report.errors.push(UsefulnessLedgerError {
                    canonical_id: r.canonical_id,
                    task_id: r.task_id,
                    dataset_id: r.dataset_id,
                    kind: UsefulnessLedgerErrorKind::RetiredStateWithoutMeasuredEvidence,
                });
            }
        }
    }

    // Rule 3: L8 gate.
    for d in records {
        if d.implementation_status == ImplementationLevel::L8_LedgerCharacterised {
            let has_measured = rows.iter().any(|r| {
                r.canonical_id == d.canonical_id
                    && matches!(
                        r.evidence_level,
                        UsefulnessEvidenceLevel::RealDatasetMeasured
                            | UsefulnessEvidenceLevel::CrossDomainReplicated
                    )
            });
            if !has_measured {
                report.errors.push(UsefulnessLedgerError {
                    canonical_id: d.canonical_id,
                    task_id: TaskId(""),
                    dataset_id: DatasetId(""),
                    kind: UsefulnessLedgerErrorKind::L8RecordWithoutMeasuredLedgerEvidence,
                });
            }
        }
    }

    // Rule 8: no triple is both Active and Retired*/Quarantined.
    for (i, a) in rows.iter().enumerate() {
        if a.lifecycle_state != LifecycleState::Active {
            continue;
        }
        for b in rows.iter().skip(i + 1) {
            if a.canonical_id == b.canonical_id
                && a.task_id == b.task_id
                && a.dataset_id == b.dataset_id
                && matches!(
                    b.lifecycle_state,
                    LifecycleState::RetiredRedundant
                        | LifecycleState::RetiredHighFalsePositive
                        | LifecycleState::RetiredTooExpensive
                        | LifecycleState::QuarantinedUnstable
                )
            {
                report.errors.push(UsefulnessLedgerError {
                    canonical_id: a.canonical_id,
                    task_id: a.task_id,
                    dataset_id: a.dataset_id,
                    kind: UsefulnessLedgerErrorKind::SameTripleBothActiveAndRetired,
                });
            }
        }
    }

    report
}

// ===================================================================
// Histogram helpers.
// ===================================================================

/// Histogram of ledger rows by evidence level. Canonical order
/// matches [`UsefulnessEvidenceLevel::all`].
#[derive(Debug, Clone, Copy, Default)]
pub struct EvidenceLevelHistogram {
    /// `Unmeasured` count.
    pub unmeasured: usize,
    /// `LiteraturePrior` count.
    pub literature_prior: usize,
    /// `RoleSeeded` count.
    pub role_seeded: usize,
    /// `SyntheticFixtureMeasured` count.
    pub synthetic_fixture: usize,
    /// `RealDatasetMeasured` count.
    pub real_dataset: usize,
    /// `CrossDomainReplicated` count.
    pub cross_domain: usize,
    /// `RetiredByEvidence` count.
    pub retired_by_evidence: usize,
}

impl EvidenceLevelHistogram {
    /// Sum of every bucket.
    #[must_use]
    pub fn total(&self) -> usize {
        self.unmeasured
            + self.literature_prior
            + self.role_seeded
            + self.synthetic_fixture
            + self.real_dataset
            + self.cross_domain
            + self.retired_by_evidence
    }
}

/// Compute the evidence-level histogram over a slice of rows.
#[must_use]
pub fn compute_evidence_histogram(rows: &[UsefulnessLedgerRow]) -> EvidenceLevelHistogram {
    let mut h = EvidenceLevelHistogram::default();
    for r in rows {
        match r.evidence_level {
            UsefulnessEvidenceLevel::Unmeasured => h.unmeasured += 1,
            UsefulnessEvidenceLevel::LiteraturePrior => h.literature_prior += 1,
            UsefulnessEvidenceLevel::RoleSeeded => h.role_seeded += 1,
            UsefulnessEvidenceLevel::SyntheticFixtureMeasured => h.synthetic_fixture += 1,
            UsefulnessEvidenceLevel::RealDatasetMeasured => h.real_dataset += 1,
            UsefulnessEvidenceLevel::CrossDomainReplicated => h.cross_domain += 1,
            UsefulnessEvidenceLevel::RetiredByEvidence => h.retired_by_evidence += 1,
        }
    }
    h
}

/// Histogram of ledger rows by lifecycle state.
#[derive(Debug, Clone, Copy, Default)]
pub struct LifecycleHistogram {
    /// `Active` count.
    pub active: usize,
    /// `Dormant` count.
    pub dormant: usize,
    /// `RetiredRedundant` count.
    pub retired_redundant: usize,
    /// `RetiredHighFalsePositive` count.
    pub retired_high_fp: usize,
    /// `RetiredTooExpensive` count.
    pub retired_too_expensive: usize,
    /// `QuarantinedUnstable` count.
    pub quarantined: usize,
    /// `ResurrectedForDomain` count.
    pub resurrected: usize,
}

impl LifecycleHistogram {
    /// Sum of every bucket.
    #[must_use]
    pub fn total(&self) -> usize {
        self.active
            + self.dormant
            + self.retired_redundant
            + self.retired_high_fp
            + self.retired_too_expensive
            + self.quarantined
            + self.resurrected
    }
}

/// Compute the lifecycle histogram over a slice of rows.
#[must_use]
pub fn compute_lifecycle_histogram(rows: &[UsefulnessLedgerRow]) -> LifecycleHistogram {
    let mut h = LifecycleHistogram::default();
    for r in rows {
        match r.lifecycle_state {
            LifecycleState::Active => h.active += 1,
            LifecycleState::Dormant => h.dormant += 1,
            LifecycleState::RetiredRedundant => h.retired_redundant += 1,
            LifecycleState::RetiredHighFalsePositive => h.retired_high_fp += 1,
            LifecycleState::RetiredTooExpensive => h.retired_too_expensive += 1,
            LifecycleState::QuarantinedUnstable => h.quarantined += 1,
            LifecycleState::ResurrectedForDomain => h.resurrected += 1,
        }
    }
    h
}

// ===================================================================
// Conservative T.8 seed.
// ===================================================================

/// The single task ID every conservative seed row uses at T.8.
/// Later commits add additional task IDs when real benchmarks
/// populate measured rows.
pub const SEED_TASK_ID: TaskId = TaskId("atlas_corpus_seed_v1");

/// The single dataset ID every conservative seed row uses at
/// T.8. T.9+ introduces real dataset IDs.
pub const SEED_DATASET_ID: DatasetId = DatasetId("none");

include!("usefulness_seed.rs");

/// Convenience accessor — calls
/// [`compute_evidence_histogram`] over [`USEFULNESS_LEDGER`].
#[must_use]
pub fn seed_evidence_histogram() -> EvidenceLevelHistogram {
    compute_evidence_histogram(USEFULNESS_LEDGER)
}

/// Convenience accessor — calls
/// [`compute_lifecycle_histogram`] over [`USEFULNESS_LEDGER`].
#[must_use]
pub fn seed_lifecycle_histogram() -> LifecycleHistogram {
    compute_lifecycle_histogram(USEFULNESS_LEDGER)
}

/// Verbose receipt-style format for one row. Used by the report
/// and any debug dump.
#[must_use]
pub fn format_row(row: &UsefulnessLedgerRow) -> String {
    format!(
        "[{id:>3}] task={task} domain=0x{domain:04x} dataset={dataset} \
         evidence={evidence} lifecycle={lifecycle:?} score={score} \
         reason={reason} source={source}",
        id = row.canonical_id.0,
        task = row.task_id.0,
        domain = row.domain.0,
        dataset = row.dataset_id.0,
        evidence = row.evidence_level.as_str(),
        lifecycle = row.lifecycle_state,
        score = row.score_kind.as_str(),
        reason = row.reason_code.as_str(),
        source = row.ledger_source.as_str(),
    )
}
