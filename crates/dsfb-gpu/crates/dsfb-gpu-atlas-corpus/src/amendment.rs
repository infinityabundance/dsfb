//! T.12.0 — `CorpusAmendmentProposal` scaffold: the legal
//! intake system for corpus expansion.
//!
//! **Panel-locked thesis**: *"T.12.0 introduces the amendment
//! court for corpus scale-out: new literature primitives enter
//! as reviewable amendment proposals, not silent mutations of
//! `corpus_hash_v1`."*
//!
//! T.12.0 is the **legal intake system** for the T.12 expansion
//! arc. No detectors land here; T.12.0 ships the schema, the
//! three new own-namespace hashes, the verifier, and a
//! conservative empty proposal as a proof-of-life. T.12.a
//! (statistical process control) uses this scaffold to file
//! its first real proposal.
//!
//! **Three new own-namespace hashes (panel-locked, NOT folded
//! upstream)**:
//!
//! * `literature_expansion_batch_hash_v1` under
//!   `DSFB-GPU-ATLAS:LITERATURE-EXPANSION-BATCH:v1\0`
//! * `corpus_amendment_proposal_hash_v1` under
//!   `DSFB-GPU-ATLAS:CORPUS-AMENDMENT-PROPOSAL:v1\0`
//! * `dedup_court_delta_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEDUP-COURT-DELTA:v1\0`
//!
//! **Hash posture**: `corpus_hash_v1`, `registry_hash_v2`,
//! every T.11a–T.11h hash, every S1.3a/b/c hash, and every
//! `DetectorPassport` hash are byte-identical after T.12.0.
//! `corpus_hash_v2` is NOT created here — only at a future
//! formal freeze campaign after enough T.12.x sub-campaigns
//! have landed.
//!
//! **Scope discipline (panel-locked)**: T.12.0 is **schema +
//! verifier + empty proof-of-life seed**. It does NOT add any
//! literature primitive, mutate `corpus_hash_v1`, create
//! `corpus_hash_v2`, change the registry, activate new
//! detectors, or execute on GPU. Same `no-silent-court-logic`
//! discipline as every prior court surface: every `pub` item
//! AND every private helper carries a doc comment whose first
//! sentence states the WHY for a future engineer.

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

use crate::seed::SEED;
use crate::types::{DetectorAliasId, DetectorCanonicalId};
use dsfb_gpu_debug_core::hash::sha256;

// ---------------------------------------------------------------
// Domain separators + schema constants
// ---------------------------------------------------------------

/// Domain separator for `literature_expansion_batch_hash_v1`.
/// Trailing `\0` is load-bearing.
pub const LITERATURE_EXPANSION_BATCH_DOMAIN: &str =
    "DSFB-GPU-ATLAS:LITERATURE-EXPANSION-BATCH:v1\0";

/// Domain separator for `corpus_amendment_proposal_hash_v1`.
pub const CORPUS_AMENDMENT_PROPOSAL_DOMAIN: &str = "DSFB-GPU-ATLAS:CORPUS-AMENDMENT-PROPOSAL:v1\0";

/// Domain separator for `dedup_court_delta_hash_v1`.
pub const DEDUP_COURT_DELTA_DOMAIN: &str = "DSFB-GPU-ATLAS:DEDUP-COURT-DELTA:v1\0";

/// Schema wire-name for the literature expansion batch.
pub const LITERATURE_EXPANSION_BATCH_SCHEMA_V1: &str = "LiteratureExpansionBatchV1";

/// Schema wire-name for the corpus amendment proposal.
pub const CORPUS_AMENDMENT_PROPOSAL_SCHEMA_V1: &str = "CorpusAmendmentProposalV1";

/// Schema wire-name for the dedup court delta.
pub const DEDUP_COURT_DELTA_SCHEMA_V1: &str = "DedupCourtDeltaV1";

// ---------------------------------------------------------------
// Source class enum (panel-locked, 23 variants)
// ---------------------------------------------------------------

/// The 23 panel-locked source classes a `CorpusAmendmentProposal`
/// may target. The verifier uses the enum's type-level
/// discipline to reject unknown classes; the wire-name list is
/// also used in canonical-byte hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SourceClass {
    /// Statistical process control (Shewhart, EWMA, CUSUM, ...).
    StatisticalProcessControl,
    /// Sequential change detection (Page-Hinkley, GLR, ...).
    SequentialChangeDetection,
    /// Concept-drift detection (ADWIN, DDM, EDDM, ...).
    DriftDetection,
    /// Robust statistics (robust z, MAD, Hampel, ...).
    RobustStatistics,
    /// Distribution distance (KS, KL, JS, Wasserstein, ...).
    DistributionDistance,
    /// Information theory (entropy, mutual info, ...).
    InformationTheory,
    /// Signal processing (matched filter, envelope, ...).
    SignalProcessing,
    /// Spectral + wavelet methods (FFT band energy, STFT, ...).
    SpectralAndWavelet,
    /// Time-series structure (ARIMA residuals, STL, ...).
    TimeSeriesStructure,
    /// Control residuals (innovation, parity-space, ...).
    ControlResiduals,
    /// Fault detection / diagnostics (observer, PCA SPE, ...).
    FaultDetectionDiagnostics,
    /// Condition monitoring (vibration, oscillation, ...).
    ConditionMonitoring,
    /// Industrial process monitoring (multivariate SPC, ...).
    IndustrialProcessMonitoring,
    /// Graph anomaly detection.
    GraphAnomalyDetection,
    /// Streaming sketches (count-min, HyperLogLog drift, ...).
    StreamingSketches,
    /// Data-quality rules (Great Expectations style).
    DataQualityRules,
    /// Database integrity constraints (FD, uniqueness, ...).
    DatabaseIntegrityConstraints,
    /// Observability / debugging (latency ramp, error burst, ...).
    ObservabilityDebugging,
    /// Medical biosignal (R-peak interval, ST-segment, ...).
    MedicalBiosignal,
    /// RF / communications (EVM, SNR, ...).
    RfCommunications,
    /// Chemometrics (PCA score outlier, T², Q, ...).
    Chemometrics,
    /// Econometrics (Chow test, Quandt-Andrews, ...).
    Econometrics,
    /// Reliability / survival analysis (Cox proportional hazards, ...).
    ReliabilitySurvival,
}

impl SourceClass {
    /// Stable wire name. Used in canonical-byte hash material
    /// and in CLI / receipt rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StatisticalProcessControl => "StatisticalProcessControl",
            Self::SequentialChangeDetection => "SequentialChangeDetection",
            Self::DriftDetection => "DriftDetection",
            Self::RobustStatistics => "RobustStatistics",
            Self::DistributionDistance => "DistributionDistance",
            Self::InformationTheory => "InformationTheory",
            Self::SignalProcessing => "SignalProcessing",
            Self::SpectralAndWavelet => "SpectralAndWavelet",
            Self::TimeSeriesStructure => "TimeSeriesStructure",
            Self::ControlResiduals => "ControlResiduals",
            Self::FaultDetectionDiagnostics => "FaultDetectionDiagnostics",
            Self::ConditionMonitoring => "ConditionMonitoring",
            Self::IndustrialProcessMonitoring => "IndustrialProcessMonitoring",
            Self::GraphAnomalyDetection => "GraphAnomalyDetection",
            Self::StreamingSketches => "StreamingSketches",
            Self::DataQualityRules => "DataQualityRules",
            Self::DatabaseIntegrityConstraints => "DatabaseIntegrityConstraints",
            Self::ObservabilityDebugging => "ObservabilityDebugging",
            Self::MedicalBiosignal => "MedicalBiosignal",
            Self::RfCommunications => "RfCommunications",
            Self::Chemometrics => "Chemometrics",
            Self::Econometrics => "Econometrics",
            Self::ReliabilitySurvival => "ReliabilitySurvival",
        }
    }
}

// ---------------------------------------------------------------
// Proposer + status enums
// ---------------------------------------------------------------

/// Who filed the amendment proposal. Used for audit posture
/// (the proposer's role affects how the court weighs the
/// proposal but does not change its hash-bound identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProposerRole {
    /// A panel member filed the proposal directly.
    PanelMember,
    /// An external reviewer / domain expert filed the
    /// proposal.
    ExternalReviewer,
    /// Automated robot ingestion (e.g. literature-scan
    /// pipeline). Future-arc.
    RobotIngestion,
}

impl ProposerRole {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PanelMember => "PanelMember",
            Self::ExternalReviewer => "ExternalReviewer",
            Self::RobotIngestion => "RobotIngestion",
        }
    }
}

/// Lifecycle status of an amendment proposal.
///
/// `Open` — in review; no expansion has been ratified.
/// `Accepted` — court has agreed; the proposal will land at
/// the next formal `corpus_hash_v2` freeze.
/// `Rejected` — court ruled against; the proposal is
/// preserved for audit history.
/// `Deferred` — court accepts the concern but waits on a
/// prerequisite (e.g. another T.12.x sub-campaign must land
/// first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProposalStatus {
    /// In review.
    Open,
    /// Court agreed; will land at next formal freeze.
    Accepted,
    /// Court ruled against.
    Rejected,
    /// Blocked on a prerequisite.
    Deferred,
}

impl ProposalStatus {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Accepted => "Accepted",
            Self::Rejected => "Rejected",
            Self::Deferred => "Deferred",
        }
    }
}

// ---------------------------------------------------------------
// Proposed-record shells
// ---------------------------------------------------------------

/// A proposed new literature primitive. T.12.0 ships the
/// shell only; T.12.a..m populate it with concrete records.
/// Each proposed primitive is identified by a panel-locked
/// canonical id reservation (a future commit promotes it to
/// `SEED` only when the proposal is `Accepted` at a
/// `corpus_hash_v2` freeze).
#[derive(Debug, Clone)]
pub struct ProposedPrimitive {
    /// Reserved canonical id (must NOT collide with any
    /// existing `SEED` record).
    pub reserved_canonical_id: DetectorCanonicalId,
    /// Operator-readable name.
    pub display_name: &'static str,
    /// One-line motivation for adding this primitive.
    pub motivation: &'static str,
}

/// A proposed alias claim. T.12.0 ships the shell; T.12.a..m
/// populate.
#[derive(Debug, Clone)]
pub struct ProposedAliasClaim {
    /// Reserved alias id.
    pub reserved_alias_id: DetectorAliasId,
    /// Canonical detector the alias collapses INTO (must
    /// exist in `SEED` or be a `ProposedPrimitive` in the
    /// same batch).
    pub collapses_into: DetectorCanonicalId,
    /// Operator-readable alias name.
    pub alias_name: &'static str,
}

/// A proposed dedup-court decision (Canonical / AliasOf /
/// ParameterizationOf / CompositionOf / ...). Shell only.
#[derive(Debug, Clone)]
pub struct ProposedDedupRecord {
    /// Wire name of the dedup decision (mirrors
    /// `CanonicalisationDecision::as_str`).
    pub decision_wire_name: &'static str,
    /// Canonical id the decision concerns.
    pub canonical_id: DetectorCanonicalId,
    /// Reason code (operator-readable).
    pub reason: &'static str,
}

/// A proposed genealogy edge addition.
#[derive(Debug, Clone)]
pub struct ProposedGenealogyEdge {
    /// Source canonical id.
    pub from_canonical_id: DetectorCanonicalId,
    /// Target canonical id.
    pub to_canonical_id: DetectorCanonicalId,
    /// Edge kind wire name (mirrors
    /// `GenealogyEdgeKind::as_str`).
    pub edge_kind_wire_name: &'static str,
}

/// A proposed source reference addition.
#[derive(Debug, Clone)]
pub struct ProposedSourceRef {
    /// Citation key.
    pub citation_key: &'static str,
    /// Title.
    pub title: &'static str,
    /// Publication year (0 for "engineering practice; no
    /// formal citation").
    pub year: u16,
    /// Venue or source.
    pub venue: &'static str,
}

/// A rejection record on a dedup-court delta.
#[derive(Debug, Clone)]
pub struct RejectionRecord {
    /// Which alias was rejected.
    pub rejected_alias_id: DetectorAliasId,
    /// Reason wire name.
    pub reason: &'static str,
}

// ---------------------------------------------------------------
// Public schema
// ---------------------------------------------------------------

/// One batch of proposed expansion records, bundled per
/// source-class. Two builds with identical fields produce
/// byte-identical hashes.
#[derive(Debug, Clone)]
pub struct CorpusExpansionBatch {
    /// Human-readable batch identifier (non-empty).
    pub batch_id: &'static str,
    /// Which source class this batch expands.
    pub source_class: SourceClass,
    /// New literature primitives proposed for canonicalisation.
    pub proposed_primitives: Vec<ProposedPrimitive>,
    /// Alias claims to be evaluated by the dedup court.
    pub proposed_aliases: Vec<ProposedAliasClaim>,
    /// Dedup-court decisions proposed for the batch.
    pub proposed_dedup_records: Vec<ProposedDedupRecord>,
    /// Genealogy edges proposed for the batch.
    pub proposed_genealogy_edges: Vec<ProposedGenealogyEdge>,
    /// Source-ref additions proposed for the batch.
    pub proposed_source_refs: Vec<ProposedSourceRef>,
    /// SHA-256 over the canonical-byte form. Two builds of
    /// the same batch produce byte-identical bytes.
    pub literature_expansion_batch_hash_v1: [u8; 32],
}

/// Dedup-court delta — the per-batch dedup diff. Records the
/// court's outcome on every record in the batch.
#[derive(Debug, Clone)]
pub struct DedupCourtDelta {
    /// Human-readable delta identifier (may be empty for
    /// proof-of-life proposals).
    pub delta_id: &'static str,
    /// New canonical primitives ratified by this delta.
    pub new_canonical_records: Vec<DetectorCanonicalId>,
    /// New alias records ratified by this delta.
    pub new_alias_records: Vec<DetectorAliasId>,
    /// New composition records ratified by this delta.
    pub new_composition_records: Vec<DetectorCanonicalId>,
    /// Records the court rejected.
    pub rejection_records: Vec<RejectionRecord>,
    /// Records the court deferred for review.
    pub deferred_records: Vec<DetectorAliasId>,
    /// SHA-256 over the canonical-byte form.
    pub dedup_court_delta_hash_v1: [u8; 32],
}

/// Top-level wrapper proposing a batch for formal review.
/// Carries provenance, motivation, target source-class,
/// proposer role, and resolution status.
#[derive(Debug, Clone)]
pub struct CorpusAmendmentProposal {
    /// Human-readable proposal identifier (non-empty).
    pub proposal_id: &'static str,
    /// Why this proposal exists (operator-readable).
    pub motivation: &'static str,
    /// Source class the proposal targets.
    pub target_source_class: SourceClass,
    /// The expansion batch.
    pub body: CorpusExpansionBatch,
    /// The dedup-court delta for the batch.
    pub dedup_court_delta: DedupCourtDelta,
    /// Lifecycle status.
    pub status: ProposalStatus,
    /// Who filed it.
    pub proposer_role: ProposerRole,
    /// Short commit hash where the proposal was filed (for
    /// `Accepted` status this MUST be non-empty so the future
    /// formal freeze can cite it).
    pub created_at_commit: &'static str,
    /// SHA-256 over every field above.
    pub corpus_amendment_proposal_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Canonical-byte serialisation
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u16(out: &mut Vec<u8>, v: u16) {
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

/// Append a `ProposedPrimitive` to the canonical byte buffer.
fn write_proposed_primitive(out: &mut Vec<u8>, p: &ProposedPrimitive) {
    write_u32(out, p.reserved_canonical_id.0);
    write_str(out, p.display_name);
    write_str(out, p.motivation);
}

/// Append a `ProposedAliasClaim` to the canonical byte buffer.
fn write_proposed_alias_claim(out: &mut Vec<u8>, a: &ProposedAliasClaim) {
    write_u32(out, a.reserved_alias_id.0);
    write_u32(out, a.collapses_into.0);
    write_str(out, a.alias_name);
}

/// Append a `ProposedDedupRecord` to the canonical byte buffer.
fn write_proposed_dedup_record(out: &mut Vec<u8>, r: &ProposedDedupRecord) {
    write_str(out, r.decision_wire_name);
    write_u32(out, r.canonical_id.0);
    write_str(out, r.reason);
}

/// Append a `ProposedGenealogyEdge` to the canonical byte buffer.
fn write_proposed_genealogy_edge(out: &mut Vec<u8>, e: &ProposedGenealogyEdge) {
    write_u32(out, e.from_canonical_id.0);
    write_u32(out, e.to_canonical_id.0);
    write_str(out, e.edge_kind_wire_name);
}

/// Append a `ProposedSourceRef` to the canonical byte buffer.
fn write_proposed_source_ref(out: &mut Vec<u8>, s: &ProposedSourceRef) {
    write_str(out, s.citation_key);
    write_str(out, s.title);
    write_u16(out, s.year);
    write_str(out, s.venue);
}

/// Append a `RejectionRecord` to the canonical byte buffer.
fn write_rejection_record(out: &mut Vec<u8>, r: &RejectionRecord) {
    write_u32(out, r.rejected_alias_id.0);
    write_str(out, r.reason);
}

/// Compute `literature_expansion_batch_hash_v1`. Two builds of
/// the same batch produce byte-identical output. Field order
/// matches the schema declaration.
#[must_use]
pub fn compute_literature_expansion_batch_hash_v1(b: &CorpusExpansionBatch) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(LITERATURE_EXPANSION_BATCH_DOMAIN.as_bytes());
    write_str(&mut buf, LITERATURE_EXPANSION_BATCH_SCHEMA_V1);
    write_str(&mut buf, b.batch_id);
    write_str(&mut buf, b.source_class.as_str());
    write_u32(
        &mut buf,
        u32::try_from(b.proposed_primitives.len()).unwrap_or(u32::MAX),
    );
    for p in &b.proposed_primitives {
        write_proposed_primitive(&mut buf, p);
    }
    write_u32(
        &mut buf,
        u32::try_from(b.proposed_aliases.len()).unwrap_or(u32::MAX),
    );
    for a in &b.proposed_aliases {
        write_proposed_alias_claim(&mut buf, a);
    }
    write_u32(
        &mut buf,
        u32::try_from(b.proposed_dedup_records.len()).unwrap_or(u32::MAX),
    );
    for r in &b.proposed_dedup_records {
        write_proposed_dedup_record(&mut buf, r);
    }
    write_u32(
        &mut buf,
        u32::try_from(b.proposed_genealogy_edges.len()).unwrap_or(u32::MAX),
    );
    for e in &b.proposed_genealogy_edges {
        write_proposed_genealogy_edge(&mut buf, e);
    }
    write_u32(
        &mut buf,
        u32::try_from(b.proposed_source_refs.len()).unwrap_or(u32::MAX),
    );
    for s in &b.proposed_source_refs {
        write_proposed_source_ref(&mut buf, s);
    }
    sha256(&buf)
}

/// Compute `dedup_court_delta_hash_v1`. Excludes its own field.
#[must_use]
pub fn compute_dedup_court_delta_hash_v1(d: &DedupCourtDelta) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(DEDUP_COURT_DELTA_DOMAIN.as_bytes());
    write_str(&mut buf, DEDUP_COURT_DELTA_SCHEMA_V1);
    write_str(&mut buf, d.delta_id);
    write_u32(
        &mut buf,
        u32::try_from(d.new_canonical_records.len()).unwrap_or(u32::MAX),
    );
    for c in &d.new_canonical_records {
        write_u32(&mut buf, c.0);
    }
    write_u32(
        &mut buf,
        u32::try_from(d.new_alias_records.len()).unwrap_or(u32::MAX),
    );
    for a in &d.new_alias_records {
        write_u32(&mut buf, a.0);
    }
    write_u32(
        &mut buf,
        u32::try_from(d.new_composition_records.len()).unwrap_or(u32::MAX),
    );
    for c in &d.new_composition_records {
        write_u32(&mut buf, c.0);
    }
    write_u32(
        &mut buf,
        u32::try_from(d.rejection_records.len()).unwrap_or(u32::MAX),
    );
    for r in &d.rejection_records {
        write_rejection_record(&mut buf, r);
    }
    write_u32(
        &mut buf,
        u32::try_from(d.deferred_records.len()).unwrap_or(u32::MAX),
    );
    for a in &d.deferred_records {
        write_u32(&mut buf, a.0);
    }
    sha256(&buf)
}

/// Compute `corpus_amendment_proposal_hash_v1`. Excludes its
/// own field. Field order matches the schema declaration so
/// adding a future field cannot silently collide.
#[must_use]
pub fn compute_corpus_amendment_proposal_hash_v1(p: &CorpusAmendmentProposal) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(CORPUS_AMENDMENT_PROPOSAL_DOMAIN.as_bytes());
    write_str(&mut buf, CORPUS_AMENDMENT_PROPOSAL_SCHEMA_V1);
    write_str(&mut buf, p.proposal_id);
    write_str(&mut buf, p.motivation);
    write_str(&mut buf, p.target_source_class.as_str());
    write_bytes(&mut buf, &p.body.literature_expansion_batch_hash_v1);
    write_bytes(&mut buf, &p.dedup_court_delta.dedup_court_delta_hash_v1);
    write_str(&mut buf, p.status.as_str());
    write_str(&mut buf, p.proposer_role.as_str());
    write_str(&mut buf, p.created_at_commit);
    sha256(&buf)
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build a `CorpusExpansionBatch` and populate its hash.
#[must_use]
pub fn build_expansion_batch(
    batch_id: &'static str,
    source_class: SourceClass,
    proposed_primitives: Vec<ProposedPrimitive>,
    proposed_aliases: Vec<ProposedAliasClaim>,
    proposed_dedup_records: Vec<ProposedDedupRecord>,
    proposed_genealogy_edges: Vec<ProposedGenealogyEdge>,
    proposed_source_refs: Vec<ProposedSourceRef>,
) -> CorpusExpansionBatch {
    let mut b = CorpusExpansionBatch {
        batch_id,
        source_class,
        proposed_primitives,
        proposed_aliases,
        proposed_dedup_records,
        proposed_genealogy_edges,
        proposed_source_refs,
        literature_expansion_batch_hash_v1: [0u8; 32],
    };
    b.literature_expansion_batch_hash_v1 = compute_literature_expansion_batch_hash_v1(&b);
    b
}

/// Build a `DedupCourtDelta` and populate its hash.
#[must_use]
pub fn build_dedup_court_delta(
    delta_id: &'static str,
    new_canonical_records: Vec<DetectorCanonicalId>,
    new_alias_records: Vec<DetectorAliasId>,
    new_composition_records: Vec<DetectorCanonicalId>,
    rejection_records: Vec<RejectionRecord>,
    deferred_records: Vec<DetectorAliasId>,
) -> DedupCourtDelta {
    let mut d = DedupCourtDelta {
        delta_id,
        new_canonical_records,
        new_alias_records,
        new_composition_records,
        rejection_records,
        deferred_records,
        dedup_court_delta_hash_v1: [0u8; 32],
    };
    d.dedup_court_delta_hash_v1 = compute_dedup_court_delta_hash_v1(&d);
    d
}

/// Build a `CorpusAmendmentProposal` and populate its hash.
#[must_use]
pub fn build_amendment_proposal(
    proposal_id: &'static str,
    motivation: &'static str,
    target_source_class: SourceClass,
    body: CorpusExpansionBatch,
    dedup_court_delta: DedupCourtDelta,
    status: ProposalStatus,
    proposer_role: ProposerRole,
    created_at_commit: &'static str,
) -> CorpusAmendmentProposal {
    let mut p = CorpusAmendmentProposal {
        proposal_id,
        motivation,
        target_source_class,
        body,
        dedup_court_delta,
        status,
        proposer_role,
        created_at_commit,
        corpus_amendment_proposal_hash_v1: [0u8; 32],
    };
    p.corpus_amendment_proposal_hash_v1 = compute_corpus_amendment_proposal_hash_v1(&p);
    p
}

// ---------------------------------------------------------------
// Conservative seed (T.12.0 proof-of-life)
// ---------------------------------------------------------------

/// The panel-locked T.12.0 empty proof-of-life proposal. Used
/// to exercise the schema + hash + verifier + CLI surface
/// without filing any actual expansion. T.12.a will replace
/// this with the first real proposal.
#[must_use]
pub fn seed_proof_of_life_proposal() -> CorpusAmendmentProposal {
    let empty_batch = build_expansion_batch(
        "t12_0_empty_batch",
        SourceClass::StatisticalProcessControl,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let empty_delta = build_dedup_court_delta(
        "t12_0_empty_delta",
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    build_amendment_proposal(
        "t12_0_proof_of_life",
        "Establish the amendment court intake schema before any T.12.x primitive campaign files its first proposal.",
        SourceClass::StatisticalProcessControl,
        empty_batch,
        empty_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_0_scaffold",
    )
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Categorical verifier reject kinds for T.12.0. Seven panel-
/// locked rules plus structural integrity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmendmentVerifyErrorKind {
    /// Rule 1: `batch_id` is empty.
    BatchIdEmpty,
    /// Rule 2: `proposal_id` is empty.
    ProposalIdEmpty,
    /// Rule 3: target_source_class declaration is malformed
    /// (placeholder; the enum prevents this at the type
    /// level, but the rule exists so hand-built proposals
    /// from a future TOML loader can still be rejected).
    UnknownSourceClass,
    /// Rule 4: structural — the stored
    /// `corpus_amendment_proposal_hash_v1` does not match the
    /// recomputed canonical-byte form.
    AmendmentProposalHashMismatch,
    /// Rule 5 (load-bearing): a dedup-court delta declares
    /// a `new_canonical_records` entry whose canonical_id
    /// collides with an existing `SEED` record (would
    /// silently mutate the corpus).
    DedupDeltaCollidesWithExistingSeedCanonicalId {
        /// The colliding canonical_id.
        canonical_id: DetectorCanonicalId,
    },
    /// Rule 6: status is `Accepted` AND body + dedup_delta
    /// are both empty (a no-op acceptance would silently
    /// commit nothing to a future freeze).
    AcceptedProposalWithoutBodyOrDelta,
    /// Rule 7: status is `Accepted` AND `created_at_commit`
    /// is empty (no future freeze can cite the proposal).
    AcceptedProposalWithoutFutureFreezeGate,
    /// Structural: `literature_expansion_batch_hash_v1`
    /// recorded on a batch does not match its recomputed
    /// canonical-byte form.
    BatchHashMismatch,
    /// Structural: `dedup_court_delta_hash_v1` recorded on a
    /// delta does not match its recomputed canonical-byte
    /// form.
    DedupDeltaHashMismatch,
}

impl AmendmentVerifyErrorKind {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BatchIdEmpty => "BatchIdEmpty",
            Self::ProposalIdEmpty => "ProposalIdEmpty",
            Self::UnknownSourceClass => "UnknownSourceClass",
            Self::AmendmentProposalHashMismatch => "AmendmentProposalHashMismatch",
            Self::DedupDeltaCollidesWithExistingSeedCanonicalId { .. } => {
                "DedupDeltaCollidesWithExistingSeedCanonicalId"
            }
            Self::AcceptedProposalWithoutBodyOrDelta => "AcceptedProposalWithoutBodyOrDelta",
            Self::AcceptedProposalWithoutFutureFreezeGate => {
                "AcceptedProposalWithoutFutureFreezeGate"
            }
            Self::BatchHashMismatch => "BatchHashMismatch",
            Self::DedupDeltaHashMismatch => "DedupDeltaHashMismatch",
        }
    }
}

/// One verifier failure record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AmendmentVerifyError {
    /// Categorical failure kind.
    pub kind: AmendmentVerifyErrorKind,
}

/// Walk an amendment proposal against the live SEED and the
/// canonical-byte hashes. Returns one error per defect; empty
/// Vec means the proposal is admissible.
#[must_use]
pub fn verify_amendment_proposal(p: &CorpusAmendmentProposal) -> Vec<AmendmentVerifyError> {
    let mut errors: Vec<AmendmentVerifyError> = Vec::new();

    if p.proposal_id.is_empty() {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::ProposalIdEmpty,
        });
    }
    if p.body.batch_id.is_empty() {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::BatchIdEmpty,
        });
    }
    if compute_literature_expansion_batch_hash_v1(&p.body)
        != p.body.literature_expansion_batch_hash_v1
    {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::BatchHashMismatch,
        });
    }
    if compute_dedup_court_delta_hash_v1(&p.dedup_court_delta)
        != p.dedup_court_delta.dedup_court_delta_hash_v1
    {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::DedupDeltaHashMismatch,
        });
    }
    if compute_corpus_amendment_proposal_hash_v1(p) != p.corpus_amendment_proposal_hash_v1 {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::AmendmentProposalHashMismatch,
        });
    }

    // Rule 5: collision with existing SEED canonical ids.
    let seed_ids: alloc::collections::BTreeSet<u32> =
        SEED.iter().map(|r| r.canonical_id.0).collect();
    for c in &p.dedup_court_delta.new_canonical_records {
        if seed_ids.contains(&c.0) {
            errors.push(AmendmentVerifyError {
                kind: AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id: *c,
                },
            });
        }
    }
    // Also check the batch's proposed_primitives — if a
    // batch reserves a canonical_id that collides with SEED,
    // surface the same rule (proposals are the intake; the
    // court should catch a collision at intake).
    for pr in &p.body.proposed_primitives {
        if seed_ids.contains(&pr.reserved_canonical_id.0) {
            errors.push(AmendmentVerifyError {
                kind: AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id: pr.reserved_canonical_id,
                },
            });
        }
    }

    // Rule 6: Accepted proposals must carry something.
    if matches!(p.status, ProposalStatus::Accepted) {
        let empty_body = p.body.proposed_primitives.is_empty()
            && p.body.proposed_aliases.is_empty()
            && p.body.proposed_dedup_records.is_empty()
            && p.body.proposed_genealogy_edges.is_empty()
            && p.body.proposed_source_refs.is_empty();
        let empty_delta = p.dedup_court_delta.new_canonical_records.is_empty()
            && p.dedup_court_delta.new_alias_records.is_empty()
            && p.dedup_court_delta.new_composition_records.is_empty()
            && p.dedup_court_delta.rejection_records.is_empty()
            && p.dedup_court_delta.deferred_records.is_empty();
        if empty_body && empty_delta {
            errors.push(AmendmentVerifyError {
                kind: AmendmentVerifyErrorKind::AcceptedProposalWithoutBodyOrDelta,
            });
        }
    }

    // Rule 7: Accepted proposals must declare their commit
    // anchor.
    if matches!(p.status, ProposalStatus::Accepted) && p.created_at_commit.is_empty() {
        errors.push(AmendmentVerifyError {
            kind: AmendmentVerifyErrorKind::AcceptedProposalWithoutFutureFreezeGate,
        });
    }

    errors
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

/// Escape a string for JSON (RFC 8259 §7). Display-only.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// Render an amendment proposal as deterministic text. Two
/// calls produce byte-identical strings.
#[must_use]
pub fn render_amendment_proposal_text(p: &CorpusAmendmentProposal) -> String {
    let mut out = String::with_capacity(4 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - CorpusAmendmentProposalV1 (T.12.0)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "proposal_id                          : {}\n",
        p.proposal_id
    ));
    out.push_str(&format!(
        "motivation                           : {}\n",
        p.motivation
    ));
    out.push_str(&format!(
        "target_source_class                  : {}\n",
        p.target_source_class.as_str()
    ));
    out.push_str(&format!(
        "status                               : {}\n",
        p.status.as_str()
    ));
    out.push_str(&format!(
        "proposer_role                        : {}\n",
        p.proposer_role.as_str()
    ));
    out.push_str(&format!(
        "created_at_commit                    : {}\n",
        p.created_at_commit
    ));
    out.push_str(&format!(
        "corpus_amendment_proposal_hash_v1    : {}\n",
        hex(&p.corpus_amendment_proposal_hash_v1)
    ));
    out.push_str(&format!(
        "literature_expansion_batch_hash_v1   : {}\n",
        hex(&p.body.literature_expansion_batch_hash_v1)
    ));
    out.push_str(&format!(
        "dedup_court_delta_hash_v1            : {}\n\n",
        hex(&p.dedup_court_delta.dedup_court_delta_hash_v1)
    ));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!("Expansion batch ({})\n", p.body.batch_id));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!(
        "  proposed_primitives    : {}\n",
        p.body.proposed_primitives.len()
    ));
    out.push_str(&format!(
        "  proposed_aliases       : {}\n",
        p.body.proposed_aliases.len()
    ));
    out.push_str(&format!(
        "  proposed_dedup_records : {}\n",
        p.body.proposed_dedup_records.len()
    ));
    out.push_str(&format!(
        "  proposed_genealogy_edges: {}\n",
        p.body.proposed_genealogy_edges.len()
    ));
    out.push_str(&format!(
        "  proposed_source_refs   : {}\n\n",
        p.body.proposed_source_refs.len()
    ));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!(
        "Dedup court delta ({})\n",
        p.dedup_court_delta.delta_id
    ));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!(
        "  new_canonical_records   : {}\n",
        p.dedup_court_delta.new_canonical_records.len()
    ));
    out.push_str(&format!(
        "  new_alias_records       : {}\n",
        p.dedup_court_delta.new_alias_records.len()
    ));
    out.push_str(&format!(
        "  new_composition_records : {}\n",
        p.dedup_court_delta.new_composition_records.len()
    ));
    out.push_str(&format!(
        "  rejection_records       : {}\n",
        p.dedup_court_delta.rejection_records.len()
    ));
    out.push_str(&format!(
        "  deferred_records        : {}\n",
        p.dedup_court_delta.deferred_records.len()
    ));
    out
}

/// Render an amendment proposal as deterministic JSON.
#[must_use]
pub fn render_amendment_proposal_json(p: &CorpusAmendmentProposal) -> String {
    let mut out = String::with_capacity(4 * 1024);
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"proposal_id\": \"{}\",\n",
        json_escape(p.proposal_id)
    ));
    out.push_str(&format!(
        "  \"motivation\": \"{}\",\n",
        json_escape(p.motivation)
    ));
    out.push_str(&format!(
        "  \"target_source_class\": \"{}\",\n",
        p.target_source_class.as_str()
    ));
    out.push_str(&format!("  \"status\": \"{}\",\n", p.status.as_str()));
    out.push_str(&format!(
        "  \"proposer_role\": \"{}\",\n",
        p.proposer_role.as_str()
    ));
    out.push_str(&format!(
        "  \"created_at_commit\": \"{}\",\n",
        p.created_at_commit
    ));
    out.push_str(&format!(
        "  \"corpus_amendment_proposal_hash_v1\": \"{}\",\n",
        hex(&p.corpus_amendment_proposal_hash_v1)
    ));
    out.push_str(&format!(
        "  \"literature_expansion_batch_hash_v1\": \"{}\",\n",
        hex(&p.body.literature_expansion_batch_hash_v1)
    ));
    out.push_str(&format!(
        "  \"dedup_court_delta_hash_v1\": \"{}\",\n",
        hex(&p.dedup_court_delta.dedup_court_delta_hash_v1)
    ));
    out.push_str(&format!("  \"batch_id\": \"{}\",\n", p.body.batch_id));
    out.push_str(&format!(
        "  \"delta_id\": \"{}\",\n",
        p.dedup_court_delta.delta_id
    ));
    out.push_str(&format!(
        "  \"proposed_primitives_count\": {},\n",
        p.body.proposed_primitives.len()
    ));
    out.push_str(&format!(
        "  \"proposed_aliases_count\": {},\n",
        p.body.proposed_aliases.len()
    ));
    out.push_str(&format!(
        "  \"proposed_dedup_records_count\": {},\n",
        p.body.proposed_dedup_records.len()
    ));
    out.push_str(&format!(
        "  \"proposed_genealogy_edges_count\": {},\n",
        p.body.proposed_genealogy_edges.len()
    ));
    out.push_str(&format!(
        "  \"proposed_source_refs_count\": {},\n",
        p.body.proposed_source_refs.len()
    ));
    out.push_str(&format!(
        "  \"new_canonical_records_count\": {},\n",
        p.dedup_court_delta.new_canonical_records.len()
    ));
    out.push_str(&format!(
        "  \"new_alias_records_count\": {},\n",
        p.dedup_court_delta.new_alias_records.len()
    ));
    out.push_str(&format!(
        "  \"new_composition_records_count\": {},\n",
        p.dedup_court_delta.new_composition_records.len()
    ));
    out.push_str(&format!(
        "  \"rejection_records_count\": {},\n",
        p.dedup_court_delta.rejection_records.len()
    ));
    out.push_str(&format!(
        "  \"deferred_records_count\": {}\n",
        p.dedup_court_delta.deferred_records.len()
    ));
    out.push_str("}\n");
    out
}
