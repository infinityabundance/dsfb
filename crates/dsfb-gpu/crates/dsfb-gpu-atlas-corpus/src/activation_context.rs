//! S1.3c — `TaskManifestV1` + `DatasetManifestV1` +
//! `ActivationContextV1`: bind activation decisions to a
//! declared task, domain, schema, units, sampling law, and
//! artifact fixedness contract.
//!
//! **Panel-locked thesis**: *"S1.3c makes activation
//! context-bound: detector decisions are issued against a
//! declared task, domain, schema, units, sampling law, and
//! artifact fixedness contract."* Before S1.3c the court says
//! "given the present corpus/legal surfaces, here is the
//! activation state." After S1.3c the court says "given THIS
//! specific task and dataset, here is the activation state."
//!
//! S1.3c does NOT replace S1.3a. It provides the context-bound
//! input shape so a future S1.3d planner can refine activation
//! decisions against an actual evidence contract (sampling law,
//! unit semantics, artifact fixedness, schema). The verifier
//! enforces:
//!
//! * fixed artifacts must declare `source_artifact_hash`;
//! * time-series tasks must declare a `TimestampLaw`;
//! * spectral-style detectors require a `SamplingLaw`;
//! * unit-sensitive detectors require declared
//!   `UnitSemantics`;
//! * non-empty `task_id` / `dataset_id`;
//! * non-empty `DomainTagSet` on the task manifest;
//! * non-zero `schema_hash` on the dataset manifest;
//! * context fact citations resolve.
//!
//! **Hash posture**: three NEW own-namespace hashes
//! (`task_manifest_hash_v1`, `dataset_manifest_hash_v1`,
//! `activation_context_hash_v1`) under their own domain
//! separators. Every upstream anchor (corpus / registry /
//! every T.11 hash / `activation_plan_hash_v1` /
//! `activation_decision_transcript_hash_v1` /
//! `activation_diff_hash_v1`) is byte-identical after S1.3c.
//!
//! **Scope discipline (panel-locked)**: S1.3c is **schema +
//! verifier + conservative seed manifest**. It does NOT ship
//! budget pruning (S1.3d), kernel-plan emission (S1.3e),
//! case-file activation section (S1.3f), GPU dispatch, T.8
//! ledger consumption, OTel ingestion, or EBOM/PROV exports.
//! Same `no-silent-court-logic` discipline as S1.3a/S1.3b:
//! every `pub` item AND every private helper carries a doc
//! comment whose first sentence states the WHY for a future
//! engineer.

#![allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::format_push_string,
    clippy::doc_overindented_list_items
)]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::corpus_hash::compute_corpus_hash_v1;
use crate::types::{DetectorCanonicalId, DomainTagSet, PrimitiveFamily, WitnessRole};
use dsfb_gpu_debug_core::hash::sha256;

// ---------------------------------------------------------------
// Domain separators + schema constants
// ---------------------------------------------------------------

/// Domain separator for `task_manifest_hash_v1`. Trailing `\0`
/// is load-bearing.
pub const TASK_MANIFEST_DOMAIN: &str = "DSFB-GPU-ATLAS:TASK-MANIFEST:v1\0";

/// Domain separator for `dataset_manifest_hash_v1`. Trailing
/// `\0` is load-bearing.
pub const DATASET_MANIFEST_DOMAIN: &str = "DSFB-GPU-ATLAS:DATASET-MANIFEST:v1\0";

/// Domain separator for `activation_context_hash_v1`. Trailing
/// `\0` is load-bearing.
pub const ACTIVATION_CONTEXT_DOMAIN: &str = "DSFB-GPU-ATLAS:ACTIVATION-CONTEXT:v1\0";

/// Schema wire-name for the task manifest.
pub const TASK_MANIFEST_SCHEMA_V1: &str = "TaskManifestV1";

/// Schema wire-name for the dataset manifest.
pub const DATASET_MANIFEST_SCHEMA_V1: &str = "DatasetManifestV1";

/// Schema wire-name for the activation context.
pub const ACTIVATION_CONTEXT_SCHEMA_V1: &str = "ActivationContextV1";

// ---------------------------------------------------------------
// TaskManifest enums
// ---------------------------------------------------------------

/// Categorical task kind. Each variant declares the dominant
/// evidence-court posture the planner should expect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskKind {
    /// The dsfb-gpu-debug fixture task: trace events admitted
    /// as residual evidence with bank-governed episodes.
    DebugTraceResidualCourt,
    /// Distributed-system telemetry forensic analysis.
    TelemetryForensicAnalysis,
    /// Tabular dataset structure-anomaly inference.
    TabularDatasetStructure,
    /// Univariate/multivariate time-series anomaly inference.
    TimeSeriesAnomalyCourt,
    /// Graph/topology structural anomaly.
    GraphTopologyCourt,
    /// Industrial-process FDD with parity-space residuals.
    IndustrialFDD,
    /// Generic data-quality court (missingness, cardinality,
    /// constraint violations).
    DataQualityCourt,
}

impl TaskKind {
    /// Stable wire name used in canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DebugTraceResidualCourt => "DebugTraceResidualCourt",
            Self::TelemetryForensicAnalysis => "TelemetryForensicAnalysis",
            Self::TabularDatasetStructure => "TabularDatasetStructure",
            Self::TimeSeriesAnomalyCourt => "TimeSeriesAnomalyCourt",
            Self::GraphTopologyCourt => "GraphTopologyCourt",
            Self::IndustrialFDD => "IndustrialFDD",
            Self::DataQualityCourt => "DataQualityCourt",
        }
    }

    /// True iff this task kind is intrinsically time-series
    /// (requires a `TimestampLaw` declaration on the dataset
    /// manifest). Drives verifier rule #6.
    #[must_use]
    pub const fn is_time_series(self) -> bool {
        matches!(
            self,
            Self::DebugTraceResidualCourt
                | Self::TelemetryForensicAnalysis
                | Self::TimeSeriesAnomalyCourt
                | Self::IndustrialFDD
        )
    }
}

/// Bitset of episode-kind targets the task is interested in.
/// Used by S1.3d (budget) to short-circuit detectors whose
/// `target_episode_kinds` don't intersect this set; S1.3c only
/// records the declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetEpisodeKindSet(pub u32);

impl TargetEpisodeKindSet {
    /// Primary positive episode (Primary witness driven).
    pub const PRIMARY: u32 = 1 << 0;
    /// Boundary / envelope-exit episode.
    pub const BOUNDARY: u32 = 1 << 1;
    /// Recovery edge episode.
    pub const RECOVERY: u32 = 1 << 2;
    /// Drift / distribution shift episode.
    pub const DRIFT: u32 = 1 << 3;
    /// Spike / single-window transient.
    pub const SPIKE: u32 = 1 << 4;
    /// Fanout / cascade episode (graph-topology driven).
    pub const FANOUT_CASCADE: u32 = 1 << 5;
    /// Data-quality episode (missingness, schema, etc.).
    pub const DATA_QUALITY: u32 = 1 << 6;

    /// True if at least one episode kind is declared.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Stable ordered list of (bit, wire-name) pairs for
    /// canonical-byte serialisation.
    #[must_use]
    pub const fn bit_names() -> &'static [(u32, &'static str)] {
        &[
            (Self::PRIMARY, "PRIMARY"),
            (Self::BOUNDARY, "BOUNDARY"),
            (Self::RECOVERY, "RECOVERY"),
            (Self::DRIFT, "DRIFT"),
            (Self::SPIKE, "SPIKE"),
            (Self::FANOUT_CASCADE, "FANOUT_CASCADE"),
            (Self::DATA_QUALITY, "DATA_QUALITY"),
        ]
    }
}

/// Bitset of witness roles. Mirrors `WitnessRole` but compacts
/// to a u32 for stable canonical-byte hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WitnessRoleSet(pub u32);

impl WitnessRoleSet {
    /// Primary witness.
    pub const PRIMARY: u32 = 1 << 0;
    /// Corroborating witness.
    pub const CORROBORATING: u32 = 1 << 1;
    /// Confuser (negative) witness.
    pub const CONFUSER: u32 = 1 << 2;
    /// Boundary witness.
    pub const BOUNDARY: u32 = 1 << 3;
    /// Clean-window witness.
    pub const CLEAN_WINDOW: u32 = 1 << 4;
    /// Recovery witness.
    pub const RECOVERY: u32 = 1 << 5;
    /// Timing witness.
    pub const TIMING: u32 = 1 << 6;
    /// Distribution-shape witness.
    pub const DISTRIBUTION: u32 = 1 << 7;
    /// Topology witness.
    pub const TOPOLOGY: u32 = 1 << 8;
    /// Causality-proxy witness.
    pub const CAUSALITY_PROXY: u32 = 1 << 9;

    /// True if at least one role is declared.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Stable ordered list of (bit, wire-name) pairs.
    #[must_use]
    pub const fn bit_names() -> &'static [(u32, &'static str)] {
        &[
            (Self::PRIMARY, "PRIMARY"),
            (Self::CORROBORATING, "CORROBORATING"),
            (Self::CONFUSER, "CONFUSER"),
            (Self::BOUNDARY, "BOUNDARY"),
            (Self::CLEAN_WINDOW, "CLEAN_WINDOW"),
            (Self::RECOVERY, "RECOVERY"),
            (Self::TIMING, "TIMING"),
            (Self::DISTRIBUTION, "DISTRIBUTION"),
            (Self::TOPOLOGY, "TOPOLOGY"),
            (Self::CAUSALITY_PROXY, "CAUSALITY_PROXY"),
        ]
    }

    /// Convert a `WitnessRole` variant to its set bit.
    #[must_use]
    pub const fn from_role(r: WitnessRole) -> u32 {
        match r {
            WitnessRole::Primary => Self::PRIMARY,
            WitnessRole::Corroborating => Self::CORROBORATING,
            WitnessRole::Confuser => Self::CONFUSER,
            WitnessRole::Boundary => Self::BOUNDARY,
            WitnessRole::CleanWindow => Self::CLEAN_WINDOW,
            WitnessRole::Recovery => Self::RECOVERY,
            WitnessRole::Timing => Self::TIMING,
            WitnessRole::Distribution => Self::DISTRIBUTION,
            WitnessRole::Topology => Self::TOPOLOGY,
            WitnessRole::CausalityProxy => Self::CAUSALITY_PROXY,
        }
    }
}

/// Strictness level for the planner under this context. Higher
/// levels demand more declared evidence-contract facts; the
/// court refuses to admit a detector whose context-contract
/// preconditions aren't satisfied. The seed default is
/// `Phase5_6` (matches the v0 strictness ladder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StrictnessLevel {
    /// Phase 0 — relaxed; useful for exploratory diff against
    /// a fully-conservative plan.
    Phase0,
    /// Phase 5.6 — standard discipline (default).
    Phase5_6,
    /// Phase 7 — adds confuser-suppression requirements.
    Phase7,
    /// Phase 8 — the Anti-Hallucination Ladder's strictest
    /// rung.
    Phase8,
}

impl StrictnessLevel {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Phase0 => "Phase0",
            Self::Phase5_6 => "Phase5_6",
            Self::Phase7 => "Phase7",
            Self::Phase8 => "Phase8",
        }
    }
}

// ---------------------------------------------------------------
// DatasetManifest enums
// ---------------------------------------------------------------

/// Artifact fixedness — operationalises the foundational
/// doctrine: a fixed artifact admits a fully replayable court;
/// a streaming artifact admits only prefix-replay; an unfixed
/// external reference is INADMISSIBLE unless materialised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactFixedness {
    /// Source bytes are immutable (e.g. a committed file).
    /// Full deterministic replay required; the verifier
    /// rejects a manifest declaring `FixedBytes` without
    /// `source_artifact_hash`.
    FixedBytes,
    /// Rows are immutable; row-by-row hashing admissible.
    FixedRows,
    /// Catalog of events is immutable (a sealed trace).
    FixedEventCatalog,
    /// Schema is fixed but rows may change between snapshots;
    /// schema-hash + row-snapshot hash required.
    FixedSchemaMutableRows,
    /// Append-only stream; prefix-replay admissible.
    StreamingAppendOnly,
    /// Mutable stream; snapshot-hash required per replay.
    StreamingMutable,
    /// External reference not yet materialised; INADMISSIBLE
    /// in the court until materialised. The verifier surfaces
    /// this on its own dedicated rule.
    UnfixedExternalReference,
}

impl ArtifactFixedness {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedBytes => "FixedBytes",
            Self::FixedRows => "FixedRows",
            Self::FixedEventCatalog => "FixedEventCatalog",
            Self::FixedSchemaMutableRows => "FixedSchemaMutableRows",
            Self::StreamingAppendOnly => "StreamingAppendOnly",
            Self::StreamingMutable => "StreamingMutable",
            Self::UnfixedExternalReference => "UnfixedExternalReference",
        }
    }

    /// True iff this fixedness class requires a non-zero
    /// `source_artifact_hash` (load-bearing negative #1).
    #[must_use]
    pub const fn requires_source_artifact_hash(self) -> bool {
        matches!(
            self,
            Self::FixedBytes | Self::FixedRows | Self::FixedEventCatalog
        )
    }
}

/// Bitset of column kinds present in the dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnKindSet(pub u32);

impl ColumnKindSet {
    /// Numeric continuous column.
    pub const NUMERIC_CONTINUOUS: u32 = 1 << 0;
    /// Numeric integer / count column.
    pub const NUMERIC_INTEGER: u32 = 1 << 1;
    /// Boolean / two-state indicator column.
    pub const BOOLEAN: u32 = 1 << 2;
    /// Categorical / enum column.
    pub const CATEGORICAL: u32 = 1 << 3;
    /// Ordered-time column (timestamp).
    pub const TIMESTAMP: u32 = 1 << 4;
    /// Graph node / edge column.
    pub const GRAPH_REF: u32 = 1 << 5;
    /// Latency / duration column.
    pub const LATENCY: u32 = 1 << 6;
    /// Error / failure indicator column.
    pub const ERROR_INDICATOR: u32 = 1 << 7;
    /// Missingness mask column.
    pub const MISSINGNESS_MASK: u32 = 1 << 8;

    /// True if at least one column kind is declared.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Stable ordered list of (bit, wire-name) pairs.
    #[must_use]
    pub const fn bit_names() -> &'static [(u32, &'static str)] {
        &[
            (Self::NUMERIC_CONTINUOUS, "NUMERIC_CONTINUOUS"),
            (Self::NUMERIC_INTEGER, "NUMERIC_INTEGER"),
            (Self::BOOLEAN, "BOOLEAN"),
            (Self::CATEGORICAL, "CATEGORICAL"),
            (Self::TIMESTAMP, "TIMESTAMP"),
            (Self::GRAPH_REF, "GRAPH_REF"),
            (Self::LATENCY, "LATENCY"),
            (Self::ERROR_INDICATOR, "ERROR_INDICATOR"),
            (Self::MISSINGNESS_MASK, "MISSINGNESS_MASK"),
        ]
    }
}

/// Unit semantics declared by the dataset. Used by the
/// verifier to decide whether a unit-sensitive detector can
/// activate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UnitSemantics {
    /// No unit declaration; only dimensionless/categorical
    /// detectors admissible.
    NoUnitsDeclared,
    /// Latency in milliseconds + error-indicator (the
    /// dsfb-gpu-debug fixture default).
    LatencyMillisecondsAndErrorIndicator,
    /// Latency in microseconds + error-indicator.
    LatencyMicrosecondsAndErrorIndicator,
    /// Generic physical-units schema (units carried per column).
    PerColumnPhysicalUnits,
    /// Dimensionless ratios only (no SI units).
    DimensionlessRatios,
    /// Counts / cardinalities only.
    CountsOnly,
    /// Categorical labels only.
    CategoricalLabelsOnly,
}

impl UnitSemantics {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoUnitsDeclared => "NoUnitsDeclared",
            Self::LatencyMillisecondsAndErrorIndicator => "LatencyMillisecondsAndErrorIndicator",
            Self::LatencyMicrosecondsAndErrorIndicator => "LatencyMicrosecondsAndErrorIndicator",
            Self::PerColumnPhysicalUnits => "PerColumnPhysicalUnits",
            Self::DimensionlessRatios => "DimensionlessRatios",
            Self::CountsOnly => "CountsOnly",
            Self::CategoricalLabelsOnly => "CategoricalLabelsOnly",
        }
    }

    /// True iff this variant satisfies a unit-sensitive
    /// detector's preconditions (any declaration other than
    /// `NoUnitsDeclared`).
    #[must_use]
    pub const fn declares_units(self) -> bool {
        !matches!(self, Self::NoUnitsDeclared)
    }
}

/// Sampling law declared by the dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SamplingLaw {
    /// No sampling-law declaration; spectral / sequential
    /// detectors INADMISSIBLE.
    NoSamplingLawDeclared,
    /// Ordered regular windows (the dsfb-gpu-debug fixture
    /// default).
    OrderedRegularWindows,
    /// Ordered non-regular (jittered) samples.
    OrderedNonRegular,
    /// Unordered row set (e.g. tabular dataset).
    UnorderedRowSet,
    /// Graph-adjacency sampling.
    GraphAdjacency,
}

impl SamplingLaw {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoSamplingLawDeclared => "NoSamplingLawDeclared",
            Self::OrderedRegularWindows => "OrderedRegularWindows",
            Self::OrderedNonRegular => "OrderedNonRegular",
            Self::UnorderedRowSet => "UnorderedRowSet",
            Self::GraphAdjacency => "GraphAdjacency",
        }
    }

    /// True iff a declaration that satisfies a spectral or
    /// sequential detector's sampling preconditions.
    #[must_use]
    pub const fn declares_sampling(self) -> bool {
        !matches!(self, Self::NoSamplingLawDeclared)
    }
}

/// Missingness profile declared by the dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MissingnessProfile {
    /// No missingness declared (assume dense).
    NoneDeclared,
    /// Sparse random missingness.
    SparseRandom,
    /// Burst missingness (clustered nulls).
    Burst,
    /// Structured missingness (column-correlated).
    Structured,
}

impl MissingnessProfile {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoneDeclared => "NoneDeclared",
            Self::SparseRandom => "SparseRandom",
            Self::Burst => "Burst",
            Self::Structured => "Structured",
        }
    }
}

/// Timestamp law declared by the dataset (when timestamps are
/// present). Required for any task whose `is_time_series` is
/// true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimestampLaw {
    /// No timestamp law declared.
    NoneDeclared,
    /// Monotonic strictly increasing.
    MonotonicStrict,
    /// Monotonic non-decreasing.
    MonotonicNonDecreasing,
    /// Unordered timestamps.
    Unordered,
}

impl TimestampLaw {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoneDeclared => "NoneDeclared",
            Self::MonotonicStrict => "MonotonicStrict",
            Self::MonotonicNonDecreasing => "MonotonicNonDecreasing",
            Self::Unordered => "Unordered",
        }
    }

    /// True iff a non-`NoneDeclared` declaration.
    #[must_use]
    pub const fn declares_timestamps(self) -> bool {
        !matches!(self, Self::NoneDeclared)
    }
}

// ---------------------------------------------------------------
// Schema structs
// ---------------------------------------------------------------

/// `TaskManifestV1` — declared task identity and witness-role
/// shape. All fields are required; the verifier rejects empty
/// `task_id` / `domain_tags`.
#[derive(Debug, Clone)]
pub struct TaskManifestV1 {
    /// Human-readable task identifier (non-empty).
    pub task_id: &'static str,
    /// Categorical task family.
    pub task_kind: TaskKind,
    /// Bitset of domains the task spans.
    pub domain_tags: DomainTagSet,
    /// Episode kinds the task is interested in admitting.
    pub target_episode_kinds: TargetEpisodeKindSet,
    /// Witness roles the task requires (planner must admit at
    /// least one detector per required role).
    pub required_witness_roles: WitnessRoleSet,
    /// Witness roles the task forbids (planner must NOT admit
    /// any detector in any forbidden role).
    pub forbidden_witness_roles: WitnessRoleSet,
    /// Strictness rung from the Anti-Hallucination Ladder.
    pub strictness_level: StrictnessLevel,
    /// SHA-256 hash over the canonical-byte form. Two builds
    /// of the same manifest produce byte-identical bytes.
    pub task_manifest_hash_v1: [u8; 32],
}

/// `DatasetManifestV1` — declared dataset identity, schema,
/// units, sampling law, missingness, timestamp law, and
/// artifact fixedness.
#[derive(Debug, Clone)]
pub struct DatasetManifestV1 {
    /// Human-readable dataset identifier (non-empty).
    pub dataset_id: &'static str,
    /// Artifact-fixedness class.
    pub artifact_fixedness: ArtifactFixedness,
    /// 32-byte schema fingerprint. Verifier rejects all-zeros
    /// when fixedness requires a schema declaration.
    pub schema_hash: [u8; 32],
    /// Bitset of column kinds present.
    pub column_kinds: ColumnKindSet,
    /// Declared unit semantics.
    pub unit_semantics: UnitSemantics,
    /// Declared sampling law.
    pub sampling_law: SamplingLaw,
    /// Declared missingness profile.
    pub missingness_profile: MissingnessProfile,
    /// Declared timestamp law.
    pub timestamp_law: TimestampLaw,
    /// 32-byte source-artifact identifier. Required for
    /// `requires_source_artifact_hash()` fixedness variants;
    /// all-zeros admissible for streaming / unfixed.
    pub source_artifact_hash: [u8; 32],
    /// SHA-256 hash over the canonical-byte form.
    pub dataset_manifest_hash_v1: [u8; 32],
}

/// `ActivationContextV1` — binds an activation plan to a
/// specific task + dataset by hash. The court refuses to
/// activate detectors against a context whose upstream anchors
/// don't match the live corpus + registry.
#[derive(Debug, Clone)]
pub struct ActivationContextV1 {
    /// T.10 corpus hash anchor.
    pub corpus_hash_v1: [u8; 32],
    /// S1.2 registry hash anchor.
    pub registry_hash_v2: [u8; 32],
    /// Task manifest hash.
    pub task_manifest_hash_v1: [u8; 32],
    /// Dataset manifest hash.
    pub dataset_manifest_hash_v1: [u8; 32],
    /// T.11h coverage hole report hash.
    pub coverage_hole_hash_v1: [u8; 32],
    /// T.11g contraindication receipt hash.
    pub detector_contraindication_hash_v1: [u8; 32],
    /// SHA-256 over every field above.
    pub activation_context_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Canonical-byte serialisation
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

/// Serialise a bitset by enumerating its (bit, name) pairs in
/// the canonical order declared by the type. Each set bit
/// writes its wire name; unset bits are skipped. The terminator
/// is the empty-string write so a future bit addition cannot
/// silently collide with the unset-bit case.
fn write_bitset_pairs(out: &mut Vec<u8>, bits: u32, pairs: &[(u32, &'static str)]) {
    let mut count: u32 = 0;
    for (bit, _) in pairs {
        if (bits & bit) != 0 {
            count += 1;
        }
    }
    write_u32(out, count);
    for (bit, name) in pairs {
        if (bits & bit) != 0 {
            write_str(out, name);
        }
    }
}

/// Canonical-byte hash for a `TaskManifestV1`. Two manifests
/// with identical fields produce byte-identical bytes.
#[must_use]
pub fn compute_task_manifest_hash_v1(t: &TaskManifestV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    buf.extend_from_slice(TASK_MANIFEST_DOMAIN.as_bytes());
    write_str(&mut buf, TASK_MANIFEST_SCHEMA_V1);
    write_str(&mut buf, t.task_id);
    write_str(&mut buf, t.task_kind.as_str());
    write_u32(&mut buf, u32::from(t.domain_tags.0));
    write_u32(&mut buf, t.target_episode_kinds.0);
    write_bitset_pairs(
        &mut buf,
        t.target_episode_kinds.0,
        TargetEpisodeKindSet::bit_names(),
    );
    write_u32(&mut buf, t.required_witness_roles.0);
    write_bitset_pairs(
        &mut buf,
        t.required_witness_roles.0,
        WitnessRoleSet::bit_names(),
    );
    write_u32(&mut buf, t.forbidden_witness_roles.0);
    write_bitset_pairs(
        &mut buf,
        t.forbidden_witness_roles.0,
        WitnessRoleSet::bit_names(),
    );
    write_str(&mut buf, t.strictness_level.as_str());
    sha256(&buf)
}

/// Canonical-byte hash for a `DatasetManifestV1`.
#[must_use]
pub fn compute_dataset_manifest_hash_v1(d: &DatasetManifestV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    buf.extend_from_slice(DATASET_MANIFEST_DOMAIN.as_bytes());
    write_str(&mut buf, DATASET_MANIFEST_SCHEMA_V1);
    write_str(&mut buf, d.dataset_id);
    write_str(&mut buf, d.artifact_fixedness.as_str());
    write_bytes(&mut buf, &d.schema_hash);
    write_u32(&mut buf, d.column_kinds.0);
    write_bitset_pairs(&mut buf, d.column_kinds.0, ColumnKindSet::bit_names());
    write_str(&mut buf, d.unit_semantics.as_str());
    write_str(&mut buf, d.sampling_law.as_str());
    write_str(&mut buf, d.missingness_profile.as_str());
    write_str(&mut buf, d.timestamp_law.as_str());
    write_bytes(&mut buf, &d.source_artifact_hash);
    sha256(&buf)
}

/// Canonical-byte hash for an `ActivationContextV1`.
#[must_use]
pub fn compute_activation_context_hash_v1(c: &ActivationContextV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    buf.extend_from_slice(ACTIVATION_CONTEXT_DOMAIN.as_bytes());
    write_str(&mut buf, ACTIVATION_CONTEXT_SCHEMA_V1);
    write_bytes(&mut buf, &c.corpus_hash_v1);
    write_bytes(&mut buf, &c.registry_hash_v2);
    write_bytes(&mut buf, &c.task_manifest_hash_v1);
    write_bytes(&mut buf, &c.dataset_manifest_hash_v1);
    write_bytes(&mut buf, &c.coverage_hole_hash_v1);
    write_bytes(&mut buf, &c.detector_contraindication_hash_v1);
    sha256(&buf)
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build a `TaskManifestV1` and populate its hash. Helper for
/// constructing seed manifests with canonical-byte finalisation.
#[must_use]
pub fn build_task_manifest(
    task_id: &'static str,
    task_kind: TaskKind,
    domain_tags: DomainTagSet,
    target_episode_kinds: TargetEpisodeKindSet,
    required_witness_roles: WitnessRoleSet,
    forbidden_witness_roles: WitnessRoleSet,
    strictness_level: StrictnessLevel,
) -> TaskManifestV1 {
    let mut m = TaskManifestV1 {
        task_id,
        task_kind,
        domain_tags,
        target_episode_kinds,
        required_witness_roles,
        forbidden_witness_roles,
        strictness_level,
        task_manifest_hash_v1: [0u8; 32],
    };
    m.task_manifest_hash_v1 = compute_task_manifest_hash_v1(&m);
    m
}

/// Build a `DatasetManifestV1` and populate its hash.
#[must_use]
pub fn build_dataset_manifest(
    dataset_id: &'static str,
    artifact_fixedness: ArtifactFixedness,
    schema_hash: [u8; 32],
    column_kinds: ColumnKindSet,
    unit_semantics: UnitSemantics,
    sampling_law: SamplingLaw,
    missingness_profile: MissingnessProfile,
    timestamp_law: TimestampLaw,
    source_artifact_hash: [u8; 32],
) -> DatasetManifestV1 {
    let mut m = DatasetManifestV1 {
        dataset_id,
        artifact_fixedness,
        schema_hash,
        column_kinds,
        unit_semantics,
        sampling_law,
        missingness_profile,
        timestamp_law,
        source_artifact_hash,
        dataset_manifest_hash_v1: [0u8; 32],
    };
    m.dataset_manifest_hash_v1 = compute_dataset_manifest_hash_v1(&m);
    m
}

/// Build an `ActivationContextV1` binding a task + dataset to
/// the live corpus + T.11 hashes. The caller supplies the
/// T.11g / T.11h hashes; we compute the context hash here.
#[must_use]
pub fn build_activation_context(
    task: &TaskManifestV1,
    dataset: &DatasetManifestV1,
    registry_hash_v2: [u8; 32],
    coverage_hole_hash_v1: [u8; 32],
    detector_contraindication_hash_v1: [u8; 32],
) -> ActivationContextV1 {
    let mut c = ActivationContextV1 {
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        registry_hash_v2,
        task_manifest_hash_v1: task.task_manifest_hash_v1,
        dataset_manifest_hash_v1: dataset.dataset_manifest_hash_v1,
        coverage_hole_hash_v1,
        detector_contraindication_hash_v1,
        activation_context_hash_v1: [0u8; 32],
    };
    c.activation_context_hash_v1 = compute_activation_context_hash_v1(&c);
    c
}

// ---------------------------------------------------------------
// Conservative seed (DSFB-GPU-Debug task family)
// ---------------------------------------------------------------

/// The panel-suggested conservative seed task: the
/// dsfb-gpu-debug-core fixture as a TaskManifestV1.
#[must_use]
pub fn seed_task_manifest() -> TaskManifestV1 {
    build_task_manifest(
        "dsfb_gpu_debug_default_task",
        TaskKind::DebugTraceResidualCourt,
        DomainTagSet(DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::TIME_SERIES),
        TargetEpisodeKindSet(
            TargetEpisodeKindSet::PRIMARY
                | TargetEpisodeKindSet::BOUNDARY
                | TargetEpisodeKindSet::RECOVERY
                | TargetEpisodeKindSet::SPIKE,
        ),
        WitnessRoleSet(
            WitnessRoleSet::PRIMARY
                | WitnessRoleSet::CORROBORATING
                | WitnessRoleSet::CONFUSER
                | WitnessRoleSet::BOUNDARY
                | WitnessRoleSet::RECOVERY,
        ),
        WitnessRoleSet(0),
        StrictnessLevel::Phase5_6,
    )
}

/// Conservative seed dataset manifest matching the
/// dsfb-gpu-debug-core synthetic fixture.
#[must_use]
pub fn seed_dataset_manifest() -> DatasetManifestV1 {
    // The schema hash is a stable synthetic value over the
    // fixture's column layout (entity_id, window_idx,
    // latency_ms, error_count). Mirroring v0 fixture bytes
    // exactly is deferred to S1.3c.x; the seed pins the
    // hash so the verifier can crosscheck downstream.
    let schema_hash = sha256(b"DSFB-GPU-ATLAS:SCHEMA:dsfb_gpu_debug_default_fixture\0");
    let source_artifact_hash = sha256(b"DSFB-GPU-ATLAS:SOURCE:dsfb_gpu_debug_default_fixture\0");
    build_dataset_manifest(
        "dsfb_gpu_debug_default_fixture",
        ArtifactFixedness::FixedEventCatalog,
        schema_hash,
        ColumnKindSet(
            ColumnKindSet::NUMERIC_INTEGER
                | ColumnKindSet::TIMESTAMP
                | ColumnKindSet::LATENCY
                | ColumnKindSet::ERROR_INDICATOR,
        ),
        UnitSemantics::LatencyMillisecondsAndErrorIndicator,
        SamplingLaw::OrderedRegularWindows,
        MissingnessProfile::NoneDeclared,
        TimestampLaw::MonotonicStrict,
        source_artifact_hash,
    )
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Categorical reject kinds for the S1.3c verifier. 11
/// panel-locked rules plus structural-integrity checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextVerifyErrorKind {
    /// Rule 1: activation context with all-zero `corpus_hash_v1`.
    ContextMissingCorpusHash,
    /// Rule 2: activation context with all-zero `registry_hash_v2`.
    ContextMissingRegistryHash,
    /// Rule 3: task manifest with empty `task_id`.
    TaskManifestMissingTaskId,
    /// Rule 4: dataset manifest with empty `dataset_id`.
    DatasetManifestMissingDatasetId,
    /// Rule 5 (load-bearing): fixed-artifact fixedness with
    /// all-zero `source_artifact_hash`.
    FixedArtifactMissingSourceHash,
    /// Rule 6 (load-bearing): time-series task with
    /// `TimestampLaw::NoneDeclared`.
    TimeSeriesTaskWithoutTimestampLaw,
    /// Rule 7 (load-bearing on activation path): spectral
    /// detector activation when the dataset declares
    /// `NoSamplingLawDeclared`. (Surfaces during the per-
    /// detector activation crosscheck.)
    SpectralDetectorWithoutSamplingLaw {
        /// Detector whose activation triggered the rule.
        canonical_id: DetectorCanonicalId,
    },
    /// Rule 8 (load-bearing on activation path): unit-sensitive
    /// detector activation when the dataset declares
    /// `NoUnitsDeclared`.
    UnitSensitiveDetectorWithoutUnits {
        /// Detector whose activation triggered the rule.
        canonical_id: DetectorCanonicalId,
    },
    /// Rule 9: task manifest with empty `domain_tags`.
    TaskManifestDomainTagsEmpty,
    /// Rule 10: dataset manifest with all-zero `schema_hash`.
    DatasetManifestSchemaHashZero,
    /// Rule 11: activation decision citing a context fact not
    /// present in the bound context (placeholder for the
    /// per-decision crosscheck wired in S1.3d).
    DecisionCitesContextFactNotPresent {
        /// Detector whose decision cited the missing fact.
        canonical_id: DetectorCanonicalId,
    },
    /// Structural: hash recorded on a manifest does not match
    /// the recomputed canonical-byte form.
    ManifestHashMismatch,
}

impl ContextVerifyErrorKind {
    /// Stable wire name for use in receipt rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContextMissingCorpusHash => "ContextMissingCorpusHash",
            Self::ContextMissingRegistryHash => "ContextMissingRegistryHash",
            Self::TaskManifestMissingTaskId => "TaskManifestMissingTaskId",
            Self::DatasetManifestMissingDatasetId => "DatasetManifestMissingDatasetId",
            Self::FixedArtifactMissingSourceHash => "FixedArtifactMissingSourceHash",
            Self::TimeSeriesTaskWithoutTimestampLaw => "TimeSeriesTaskWithoutTimestampLaw",
            Self::SpectralDetectorWithoutSamplingLaw { .. } => "SpectralDetectorWithoutSamplingLaw",
            Self::UnitSensitiveDetectorWithoutUnits { .. } => "UnitSensitiveDetectorWithoutUnits",
            Self::TaskManifestDomainTagsEmpty => "TaskManifestDomainTagsEmpty",
            Self::DatasetManifestSchemaHashZero => "DatasetManifestSchemaHashZero",
            Self::DecisionCitesContextFactNotPresent { .. } => "DecisionCitesContextFactNotPresent",
            Self::ManifestHashMismatch => "ManifestHashMismatch",
        }
    }
}

/// One verifier failure. Multiple errors per inspection are
/// admissible — the verifier walks every rule and returns the
/// full list so a single audit pass surfaces all defects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextVerifyError {
    /// Categorical failure kind.
    pub kind: ContextVerifyErrorKind,
}

/// Verify a `TaskManifestV1`. Rules 3, 6, 9, plus structural
/// hash crosscheck.
#[must_use]
pub fn verify_task_manifest(t: &TaskManifestV1) -> Vec<ContextVerifyError> {
    let mut errors: Vec<ContextVerifyError> = Vec::new();
    if t.task_id.is_empty() {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::TaskManifestMissingTaskId,
        });
    }
    if t.domain_tags.0 == 0 {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::TaskManifestDomainTagsEmpty,
        });
    }
    if compute_task_manifest_hash_v1(t) != t.task_manifest_hash_v1 {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::ManifestHashMismatch,
        });
    }
    errors
}

/// Verify a `DatasetManifestV1`. Rules 4, 5, 10 plus structural
/// hash crosscheck.
#[must_use]
pub fn verify_dataset_manifest(d: &DatasetManifestV1) -> Vec<ContextVerifyError> {
    let mut errors: Vec<ContextVerifyError> = Vec::new();
    if d.dataset_id.is_empty() {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::DatasetManifestMissingDatasetId,
        });
    }
    if d.schema_hash == [0u8; 32] {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::DatasetManifestSchemaHashZero,
        });
    }
    if d.artifact_fixedness.requires_source_artifact_hash() && d.source_artifact_hash == [0u8; 32] {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::FixedArtifactMissingSourceHash,
        });
    }
    if compute_dataset_manifest_hash_v1(d) != d.dataset_manifest_hash_v1 {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::ManifestHashMismatch,
        });
    }
    errors
}

/// Verify an `ActivationContextV1` plus the bound task +
/// dataset. Rules 1, 2, 6 (cross-pair: task is time-series and
/// dataset has no timestamp law), 9, 10, plus hash crosscheck.
#[must_use]
pub fn verify_activation_context(
    c: &ActivationContextV1,
    task: &TaskManifestV1,
    dataset: &DatasetManifestV1,
) -> Vec<ContextVerifyError> {
    let mut errors: Vec<ContextVerifyError> = Vec::new();
    errors.extend(verify_task_manifest(task));
    errors.extend(verify_dataset_manifest(dataset));
    if c.corpus_hash_v1 == [0u8; 32] {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::ContextMissingCorpusHash,
        });
    }
    if c.registry_hash_v2 == [0u8; 32] {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::ContextMissingRegistryHash,
        });
    }
    if task.task_kind.is_time_series() && !dataset.timestamp_law.declares_timestamps() {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::TimeSeriesTaskWithoutTimestampLaw,
        });
    }
    if compute_activation_context_hash_v1(c) != c.activation_context_hash_v1 {
        errors.push(ContextVerifyError {
            kind: ContextVerifyErrorKind::ManifestHashMismatch,
        });
    }
    errors
}

/// Per-detector activation crosscheck under a bound context.
/// Surfaces rules 7 + 8 + 11. The caller provides the
/// (canonical_id, primitive_family, input_requirements) tuples
/// for the candidate-Enabled detectors; the verifier returns
/// one error per detector that violates the context contract.
///
/// `unit_sensitive_ids` is the set of detector canonical IDs
/// the corpus declares unit-sensitive (typically derived from
/// `InputRequirementSet::UNITS`).
///
/// `spectral_ids` is the set of canonical IDs whose family
/// requires a declared `SamplingLaw` (Spectral / Wavelet /
/// SequentialRecurrence / ScalarThreshold under the
/// `detector_is_ordered_time` rule from S1.3a).
#[must_use]
pub fn verify_activation_against_context(
    enabled_ids: &[DetectorCanonicalId],
    spectral_ids: &[DetectorCanonicalId],
    unit_sensitive_ids: &[DetectorCanonicalId],
    dataset: &DatasetManifestV1,
) -> Vec<ContextVerifyError> {
    let mut errors: Vec<ContextVerifyError> = Vec::new();
    for id in enabled_ids {
        if spectral_ids.contains(id) && !dataset.sampling_law.declares_sampling() {
            errors.push(ContextVerifyError {
                kind: ContextVerifyErrorKind::SpectralDetectorWithoutSamplingLaw {
                    canonical_id: *id,
                },
            });
        }
        if unit_sensitive_ids.contains(id) && !dataset.unit_semantics.declares_units() {
            errors.push(ContextVerifyError {
                kind: ContextVerifyErrorKind::UnitSensitiveDetectorWithoutUnits {
                    canonical_id: *id,
                },
            });
        }
    }
    errors
}

/// True iff a primitive family requires a declared
/// `SamplingLaw` under S1.3c. Mirrors the
/// `detector_is_ordered_time` rule in S1.3a (activation.rs);
/// kept here so this module is self-contained for review.
#[must_use]
pub const fn family_requires_sampling_law(family: PrimitiveFamily) -> bool {
    matches!(
        family,
        PrimitiveFamily::Spectral
            | PrimitiveFamily::Wavelet
            | PrimitiveFamily::SequentialRecurrence
            | PrimitiveFamily::ResidualObserver
    )
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

/// Lowercase hex display for a 32-byte hash. Display-only; the
/// canonical hash material is the raw bytes.
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Render a u32 bitset as a comma-joined list of set wire
/// names.
fn bitset_text(bits: u32, pairs: &[(u32, &'static str)]) -> String {
    let mut out = String::new();
    let mut first = true;
    for (bit, name) in pairs {
        if (bits & bit) != 0 {
            if !first {
                out.push_str(", ");
            }
            out.push_str(name);
            first = false;
        }
    }
    if first {
        out.push_str("(none)");
    }
    out
}

/// Render a u16 bitset (e.g. `DomainTagSet`) as a comma-joined
/// list of set wire names. `DomainTagSet::bit_names()` returns
/// `(u16, &str)`; we adapt rather than widen so we don't churn
/// the upstream type's API.
fn bitset_text_u16(bits: u16, pairs: &[(u16, &'static str)]) -> String {
    let mut out = String::new();
    let mut first = true;
    for (bit, name) in pairs {
        if (bits & bit) != 0 {
            if !first {
                out.push_str(", ");
            }
            out.push_str(name);
            first = false;
        }
    }
    if first {
        out.push_str("(none)");
    }
    out
}

/// Render a task manifest as deterministic text.
#[must_use]
pub fn render_task_manifest_text(t: &TaskManifestV1) -> String {
    let mut out = String::with_capacity(2 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - TaskManifestV1 (S1.3c)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!("task_id                  : {}\n", t.task_id));
    out.push_str(&format!(
        "task_kind                : {}\n",
        t.task_kind.as_str()
    ));
    out.push_str(&format!(
        "domain_tags              : {}\n",
        bitset_text_u16(t.domain_tags.0, DomainTagSet::bit_names())
    ));
    out.push_str(&format!(
        "target_episode_kinds     : {}\n",
        bitset_text(t.target_episode_kinds.0, TargetEpisodeKindSet::bit_names())
    ));
    out.push_str(&format!(
        "required_witness_roles   : {}\n",
        bitset_text(t.required_witness_roles.0, WitnessRoleSet::bit_names())
    ));
    out.push_str(&format!(
        "forbidden_witness_roles  : {}\n",
        bitset_text(t.forbidden_witness_roles.0, WitnessRoleSet::bit_names())
    ));
    out.push_str(&format!(
        "strictness_level         : {}\n",
        t.strictness_level.as_str()
    ));
    out.push_str(&format!(
        "task_manifest_hash_v1    : {}\n",
        hex(&t.task_manifest_hash_v1)
    ));
    out
}

/// Render a dataset manifest as deterministic text.
#[must_use]
pub fn render_dataset_manifest_text(d: &DatasetManifestV1) -> String {
    let mut out = String::with_capacity(2 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - DatasetManifestV1 (S1.3c)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!("dataset_id               : {}\n", d.dataset_id));
    out.push_str(&format!(
        "artifact_fixedness       : {}\n",
        d.artifact_fixedness.as_str()
    ));
    out.push_str(&format!(
        "schema_hash              : {}\n",
        hex(&d.schema_hash)
    ));
    out.push_str(&format!(
        "column_kinds             : {}\n",
        bitset_text(d.column_kinds.0, ColumnKindSet::bit_names())
    ));
    out.push_str(&format!(
        "unit_semantics           : {}\n",
        d.unit_semantics.as_str()
    ));
    out.push_str(&format!(
        "sampling_law             : {}\n",
        d.sampling_law.as_str()
    ));
    out.push_str(&format!(
        "missingness_profile      : {}\n",
        d.missingness_profile.as_str()
    ));
    out.push_str(&format!(
        "timestamp_law            : {}\n",
        d.timestamp_law.as_str()
    ));
    out.push_str(&format!(
        "source_artifact_hash     : {}\n",
        hex(&d.source_artifact_hash)
    ));
    out.push_str(&format!(
        "dataset_manifest_hash_v1 : {}\n",
        hex(&d.dataset_manifest_hash_v1)
    ));
    out
}

/// Render an activation context as deterministic text.
#[must_use]
pub fn render_activation_context_text(c: &ActivationContextV1) -> String {
    let mut out = String::with_capacity(2 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - ActivationContextV1 (S1.3c)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "corpus_hash_v1                        : {}\n",
        hex(&c.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "registry_hash_v2                      : {}\n",
        hex(&c.registry_hash_v2)
    ));
    out.push_str(&format!(
        "task_manifest_hash_v1                 : {}\n",
        hex(&c.task_manifest_hash_v1)
    ));
    out.push_str(&format!(
        "dataset_manifest_hash_v1              : {}\n",
        hex(&c.dataset_manifest_hash_v1)
    ));
    out.push_str(&format!(
        "coverage_hole_hash_v1                 : {}\n",
        hex(&c.coverage_hole_hash_v1)
    ));
    out.push_str(&format!(
        "detector_contraindication_hash_v1     : {}\n",
        hex(&c.detector_contraindication_hash_v1)
    ));
    out.push_str(&format!(
        "activation_context_hash_v1            : {}\n",
        hex(&c.activation_context_hash_v1)
    ));
    out
}

/// Render a task manifest as deterministic JSON.
#[must_use]
pub fn render_task_manifest_json(t: &TaskManifestV1) -> String {
    let mut out = String::with_capacity(2 * 1024);
    out.push_str("{\n");
    out.push_str(&format!("  \"task_id\": \"{}\",\n", t.task_id));
    out.push_str(&format!("  \"task_kind\": \"{}\",\n", t.task_kind.as_str()));
    out.push_str(&format!(
        "  \"domain_tags\": \"{}\",\n",
        bitset_text_u16(t.domain_tags.0, DomainTagSet::bit_names())
    ));
    out.push_str(&format!(
        "  \"target_episode_kinds\": \"{}\",\n",
        bitset_text(t.target_episode_kinds.0, TargetEpisodeKindSet::bit_names())
    ));
    out.push_str(&format!(
        "  \"required_witness_roles\": \"{}\",\n",
        bitset_text(t.required_witness_roles.0, WitnessRoleSet::bit_names())
    ));
    out.push_str(&format!(
        "  \"forbidden_witness_roles\": \"{}\",\n",
        bitset_text(t.forbidden_witness_roles.0, WitnessRoleSet::bit_names())
    ));
    out.push_str(&format!(
        "  \"strictness_level\": \"{}\",\n",
        t.strictness_level.as_str()
    ));
    out.push_str(&format!(
        "  \"task_manifest_hash_v1\": \"{}\"\n",
        hex(&t.task_manifest_hash_v1)
    ));
    out.push_str("}\n");
    out
}

/// Render a dataset manifest as deterministic JSON.
#[must_use]
pub fn render_dataset_manifest_json(d: &DatasetManifestV1) -> String {
    let mut out = String::with_capacity(2 * 1024);
    out.push_str("{\n");
    out.push_str(&format!("  \"dataset_id\": \"{}\",\n", d.dataset_id));
    out.push_str(&format!(
        "  \"artifact_fixedness\": \"{}\",\n",
        d.artifact_fixedness.as_str()
    ));
    out.push_str(&format!(
        "  \"schema_hash\": \"{}\",\n",
        hex(&d.schema_hash)
    ));
    out.push_str(&format!(
        "  \"column_kinds\": \"{}\",\n",
        bitset_text(d.column_kinds.0, ColumnKindSet::bit_names())
    ));
    out.push_str(&format!(
        "  \"unit_semantics\": \"{}\",\n",
        d.unit_semantics.as_str()
    ));
    out.push_str(&format!(
        "  \"sampling_law\": \"{}\",\n",
        d.sampling_law.as_str()
    ));
    out.push_str(&format!(
        "  \"missingness_profile\": \"{}\",\n",
        d.missingness_profile.as_str()
    ));
    out.push_str(&format!(
        "  \"timestamp_law\": \"{}\",\n",
        d.timestamp_law.as_str()
    ));
    out.push_str(&format!(
        "  \"source_artifact_hash\": \"{}\",\n",
        hex(&d.source_artifact_hash)
    ));
    out.push_str(&format!(
        "  \"dataset_manifest_hash_v1\": \"{}\"\n",
        hex(&d.dataset_manifest_hash_v1)
    ));
    out.push_str("}\n");
    out
}

/// Render an activation context as deterministic JSON.
#[must_use]
pub fn render_activation_context_json(c: &ActivationContextV1) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"corpus_hash_v1\": \"{}\",\n",
        hex(&c.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "  \"registry_hash_v2\": \"{}\",\n",
        hex(&c.registry_hash_v2)
    ));
    out.push_str(&format!(
        "  \"task_manifest_hash_v1\": \"{}\",\n",
        hex(&c.task_manifest_hash_v1)
    ));
    out.push_str(&format!(
        "  \"dataset_manifest_hash_v1\": \"{}\",\n",
        hex(&c.dataset_manifest_hash_v1)
    ));
    out.push_str(&format!(
        "  \"coverage_hole_hash_v1\": \"{}\",\n",
        hex(&c.coverage_hole_hash_v1)
    ));
    out.push_str(&format!(
        "  \"detector_contraindication_hash_v1\": \"{}\",\n",
        hex(&c.detector_contraindication_hash_v1)
    ));
    out.push_str(&format!(
        "  \"activation_context_hash_v1\": \"{}\"\n",
        hex(&c.activation_context_hash_v1)
    ));
    out.push_str("}\n");
    out
}
