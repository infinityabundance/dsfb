//! S1.3b — `ActivationDecisionTranscript` + `ActivationDiffV1`:
//! the explanation + diff court for S1.3a activation decisions.
//!
//! **Thesis (panel-locked)**: *"DSFB-GPU-Atlas does not merely
//! choose detectors; it issues replayable activation decisions
//! under a hash-bound court record."* A decision without an
//! explanation is opaque. S1.3a knows what each detector's
//! status is; S1.3b knows **why**, with full citation back to
//! the court artifact(s) that drove the decision, and can diff
//! two plans deterministically.
//!
//! The module answers five panel-locked operator questions:
//!  1. Why was detector N WarnOnly?
//!  2. Why was detector N Disabled?
//!  3. What would need to change for detector N to become
//!     Enabled?
//!  4. What changed between two activation plans?
//!  5. Which court artifact caused the block?
//!
//! **Hash posture**: every upstream anchor (`corpus_hash_v1`,
//! `registry_hash_v2`, every T.11a–T.11h hash, AND
//! `activation_plan_hash_v1`) is unchanged. S1.3b introduces two
//! own-namespace hashes:
//!
//! * `activation_decision_transcript_hash_v1` under domain
//!   `DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1\0`
//! * `activation_diff_hash_v1` under domain
//!   `DSFB-GPU-ATLAS:ACTIVATION-DIFF:v1\0`
//!
//! Two builds against the same sealed court stack produce
//! byte-identical bytes.
//!
//! **Scope discipline (panel-locked)**: S1.3b ships explanation
//! and diff machinery only. It does NOT ship TaskManifest or
//! DatasetManifest consumption (S1.3c), budget pruning (S1.3d),
//! KernelPlan emission (S1.3e), or CaseFileV2 activation
//! section integration (S1.3f). Same `no-silent-court-logic`
//! discipline as S1.3a: every `pub` item AND every private
//! helper carries a doc comment whose first sentence states
//! the WHY for a future engineer.

#![allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::format_push_string,
    clippy::doc_overindented_list_items
)]

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::activation::{
    collect_activation_plan, ActivationPlanV1, ActivationStatus, DetectorActivationDecision,
    DisabledReason, KNOWN_S12_REGISTRY_HASH_V2,
};
use crate::challenge_docket::{collect_challenge_docket, ChallengeDocketEntry, ChallengeTarget};
use crate::contraindication::{collect_contraindications, DetectorContraindicationReceiptV1};
use crate::coverage_holes::{
    collect_coverage_holes, CoverageHoleEntry, CoverageHoleSeverity, CoverageHoleSubject,
};
use crate::lband::GPU_IMPLEMENTED_CANONICAL_IDS;
use crate::passport::passport_for;
use crate::seed::SEED;
use crate::types::{DetectorCanonicalId, ImplementationLevel};
use dsfb_gpu_debug_core::hash::sha256;

// ---------------------------------------------------------------
// Domain separators + schema constants
// ---------------------------------------------------------------

/// Domain separator for `activation_decision_transcript_hash_v1`.
/// Trailing `\0` is load-bearing and pins the schema against
/// any other DSFB-GPU-Atlas hash.
pub const ACTIVATION_TRANSCRIPT_DOMAIN: &str = "DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1\0";

/// Domain separator for `activation_diff_hash_v1`. Trailing
/// `\0` is load-bearing.
pub const ACTIVATION_DIFF_DOMAIN: &str = "DSFB-GPU-ATLAS:ACTIVATION-DIFF:v1\0";

/// Wire-name for the transcript schema. Written into the hash
/// material so a future v2 cannot collide with v1.
pub const ACTIVATION_TRANSCRIPT_SCHEMA_V1: &str = "ActivationDecisionTranscriptV1";

/// Wire-name for the diff schema.
pub const ACTIVATION_DIFF_SCHEMA_V1: &str = "ActivationDiffV1";

// ---------------------------------------------------------------
// Public schema
// ---------------------------------------------------------------

/// Court artifact category for a `ContributingFact`. Each kind
/// names a distinct upstream surface; the planner's decision
/// traces back to one or more of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactKind {
    /// `DetectorPassport` (T.11a) — per-detector legal identity.
    Passport,
    /// `CoverageHoleReport` entry (T.11h).
    CoverageHole,
    /// `DetectorContraindicationReceiptV1` entry (T.11g).
    Contraindication,
    /// `ChallengeDocket` entry (T.11f).
    Challenge,
    /// L-band ladder fact (T.7). The "artifact id" is the
    /// numeric L-band tier.
    LBand,
    /// Registry hash (S1.2). Anchor only.
    RegistryHash,
    /// Corpus hash (T.10). Anchor only.
    CorpusHash,
}

impl ArtifactKind {
    /// Stable wire name used in canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passport => "Passport",
            Self::CoverageHole => "CoverageHole",
            Self::Contraindication => "Contraindication",
            Self::Challenge => "Challenge",
            Self::LBand => "LBand",
            Self::RegistryHash => "RegistryHash",
            Self::CorpusHash => "CorpusHash",
        }
    }
}

/// Role a `ContributingFact` plays in the final decision.
/// `Blocking` facts force a Disable/Deferred status; `Warning`
/// facts attach non-blocking concerns to Enabled/WarnOnly;
/// `Supporting` facts justify an Enabled status; `Informational`
/// facts surface context without affecting the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FactRole {
    /// Force a Disable / Deferred outcome.
    Blocking,
    /// Attach a non-blocking warning to Enabled / WarnOnly.
    Warning,
    /// Justify Enabled (e.g. passport-complete, role-seeded
    /// GPU surface).
    Supporting,
    /// Context only; does not change the decision.
    Informational,
}

impl FactRole {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocking => "Blocking",
            Self::Warning => "Warning",
            Self::Supporting => "Supporting",
            Self::Informational => "Informational",
        }
    }
}

/// A single court fact that contributed to the planner's
/// decision. Each fact identifies an upstream artifact by
/// `(artifact_kind, artifact_id, artifact_hash)` plus the
/// categorical reason code and an operator-readable message.
///
/// The 32-byte `artifact_hash` is the same citation the
/// planner stored in `blocking_receipt_hashes` /
/// `warning_receipt_hashes`; the transcript surfaces it
/// alongside the kind + id + reason so an auditor can trace
/// the decision link-by-link without re-running the planner.
#[derive(Debug, Clone)]
pub struct ContributingFact {
    /// Which court surface emitted the fact.
    pub artifact_kind: ArtifactKind,
    /// Stable numeric id within the surface (canonical_id for
    /// Passport / Contraindication; hole_id for CoverageHole;
    /// challenge_id for Challenge; L-band tier for LBand;
    /// 0 for anchor-only kinds).
    pub artifact_id: u32,
    /// 32-byte hash citing the upstream artifact.
    pub artifact_hash: [u8; 32],
    /// What the fact does to the decision.
    pub role: FactRole,
    /// Wire-named reason code. Mirrors the planner's reason
    /// enum string when the fact maps to a single
    /// EnabledReason / DisabledReason; otherwise carries the
    /// surface-level category (e.g. coverage-hole reason
    /// wire name).
    pub reason_code: &'static str,
    /// Short operator-readable sentence.
    pub operator_message: &'static str,
}

/// Single link in the chain of artifacts that blocked a
/// detector. For Disabled decisions, every blocking fact
/// appears as one `BlockingLink`. For Enabled / WarnOnly,
/// the chain is empty.
#[derive(Debug, Clone)]
pub struct BlockingLink {
    /// Order in the blocking chain (1 = root cause; higher =
    /// secondary blockers that would still apply if the root
    /// were resolved).
    pub link_order: u32,
    /// The blocking fact.
    pub fact: ContributingFact,
}

/// What the operator would need to change in the upstream
/// court for this detector to become Enabled. Categorical and
/// declarative — S1.3b does not execute the change; it tells
/// the operator what change is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CounterfactualStep {
    /// Promote L-band from L0/L1/L2 to L3+ by landing a host
    /// implementation backed by canonicalised math.
    PromoteLBandToL3OrHigher,
    /// Land a measured benchmark receipt to lift L4 -> L5/L6.
    LandBenchmarkReceipt,
    /// Resolve the open / sustained challenge.
    ResolveChallenge,
    /// Add the missing `closest_aliases` / `closest_non_aliases`
    /// / `adversarial_twins` declaration to the contraindication
    /// receipt.
    DeclareContraindicationFields,
    /// Add a Confuser witness to the family.
    AddFamilyConfuserWitness,
    /// Declare `required_sampling_law`.
    DeclareRequiredSamplingLaw,
    /// Declare `required_units`.
    DeclareRequiredUnits,
    /// Refresh provenance — add a post-2000 source ref with DOI.
    RefreshSourceProvenance,
    /// Provide a TaskManifest / DatasetManifest input that
    /// supplies the missing evidence contract (S1.3c+).
    SupplyEvidenceContract,
}

impl CounterfactualStep {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromoteLBandToL3OrHigher => "PromoteLBandToL3OrHigher",
            Self::LandBenchmarkReceipt => "LandBenchmarkReceipt",
            Self::ResolveChallenge => "ResolveChallenge",
            Self::DeclareContraindicationFields => "DeclareContraindicationFields",
            Self::AddFamilyConfuserWitness => "AddFamilyConfuserWitness",
            Self::DeclareRequiredSamplingLaw => "DeclareRequiredSamplingLaw",
            Self::DeclareRequiredUnits => "DeclareRequiredUnits",
            Self::RefreshSourceProvenance => "RefreshSourceProvenance",
            Self::SupplyEvidenceContract => "SupplyEvidenceContract",
        }
    }
}

/// Wrapper for the enabled/disabled reason emitted by the
/// planner, carrying just the wire name. We do not re-export
/// the planner enums because the transcript may eventually
/// describe a decision the planner itself did not emit
/// (e.g. an audit-time replay of a stored plan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FinalReason {
    /// Wire name; matches `EnabledReason::as_str()` or
    /// `DisabledReason::as_str()`.
    pub wire_name: &'static str,
}

/// Per-detector replayable explanation transcript. Every
/// transcript carries the cited contributing facts (sorted
/// deterministically), the blocking chain (for Disabled
/// decisions), the counterfactual path to Enabled (if any),
/// and a 32-byte canonical-byte hash.
#[derive(Debug, Clone)]
pub struct ActivationDecisionTranscript {
    /// Canonical detector this transcript explains.
    pub canonical_id: DetectorCanonicalId,
    /// Operator-readable name.
    pub display_name: &'static str,
    /// Outcome from the source plan.
    pub activation_status: ActivationStatus,
    /// Final reason (Enabled or Disabled wire name).
    pub final_reason: FinalReason,
    /// All facts that contributed to the outcome. Sorted by
    /// `(artifact_kind, artifact_id, role)` ascending so two
    /// builds emit byte-identical transcripts.
    pub contributing_facts: Vec<ContributingFact>,
    /// For Disabled decisions, the ordered blocking chain.
    /// Empty for Enabled / WarnOnly.
    pub blocking_chain: Vec<BlockingLink>,
    /// Steps that would change the outcome to Enabled.
    /// Empty for already-Enabled detectors.
    pub counterfactual_path_to_enabled: Vec<CounterfactualStep>,
    /// 32-byte hash over the canonical-byte projection of
    /// every field above. Excluded from its own input
    /// (computed after the body is finalised).
    pub transcript_hash_v1: [u8; 32],
}

/// Categorical change kind in an `ActivationDiffV1` row. Used
/// to surface the kind of drift between two plans without
/// requiring a byte-diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiffChangeKind {
    /// Decision present in new plan, absent in old.
    DecisionAdded,
    /// Decision present in old plan, absent in new.
    DecisionRemoved,
    /// Activation status changed (e.g. Disabled -> Enabled).
    StatusChanged,
    /// Status unchanged, but the categorical reason changed.
    ReasonChanged,
    /// Status + reason unchanged, but the citation list (court
    /// artifacts driving the decision) changed.
    CitationChanged,
}

impl DiffChangeKind {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DecisionAdded => "DecisionAdded",
            Self::DecisionRemoved => "DecisionRemoved",
            Self::StatusChanged => "StatusChanged",
            Self::ReasonChanged => "ReasonChanged",
            Self::CitationChanged => "CitationChanged",
        }
    }
}

/// One row in an `ActivationDiffV1`: one categorical change
/// against one canonical detector.
#[derive(Debug, Clone)]
pub struct ActivationDiffRow {
    /// Which detector changed.
    pub canonical_id: DetectorCanonicalId,
    /// What category of change occurred.
    pub kind: DiffChangeKind,
    /// Old status wire name (`""` if `DecisionAdded`).
    pub old_status: &'static str,
    /// New status wire name (`""` if `DecisionRemoved`).
    pub new_status: &'static str,
    /// Old reason wire name (`""` if absent).
    pub old_reason: &'static str,
    /// New reason wire name (`""` if absent).
    pub new_reason: &'static str,
}

/// Two-plan structural diff. Rows are sorted by
/// `(canonical_id, change_kind)` ascending.
#[derive(Debug, Clone)]
pub struct ActivationDiffV1 {
    /// `activation_plan_hash_v1` of the old plan.
    pub old_activation_plan_hash_v1: [u8; 32],
    /// `activation_plan_hash_v1` of the new plan.
    pub new_activation_plan_hash_v1: [u8; 32],
    /// `corpus_hash_v1` of both plans (panel-locked: diffing
    /// across corpus generations is forbidden — see the
    /// `DiffRejectsMismatchedCorpusHash` verifier rule).
    pub corpus_hash_v1: [u8; 32],
    /// Per-detector diff rows.
    pub rows: Vec<ActivationDiffRow>,
    /// Summary counts.
    pub decisions_added: u32,
    /// Decisions removed.
    pub decisions_removed: u32,
    /// Decisions whose status changed.
    pub decisions_status_changed: u32,
    /// Decisions whose categorical reason changed.
    pub decisions_reason_changed: u32,
    /// Decisions whose citation list changed.
    pub decisions_citation_changed: u32,
    /// 32-byte canonical-byte hash over every field above
    /// except itself.
    pub activation_diff_hash_v1: [u8; 32],
}

/// The full S1.3b audit wrapper: one transcript per decision
/// in a source plan. Useful for bulk export.
#[derive(Debug, Clone)]
pub struct ActivationPlanAuditV1 {
    /// `activation_plan_hash_v1` of the source plan.
    pub source_activation_plan_hash_v1: [u8; 32],
    /// One transcript per decision; sorted by `canonical_id`.
    pub transcripts: Vec<ActivationDecisionTranscript>,
}

// ---------------------------------------------------------------
// Builder helpers
// ---------------------------------------------------------------

/// Synthesize a 32-byte L-band citation hash. The L-band fact
/// lives on the detector record; we hash a stable form so a
/// transcript citing L-band has a distinct citation per
/// canonical_id under an audit-specific sub-domain.
fn lband_citation_hash(canonical_id: DetectorCanonicalId, level: ImplementationLevel) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1:LBAND\0");
    buf.extend_from_slice(&canonical_id.0.to_be_bytes());
    buf.extend_from_slice(level.as_str().as_bytes());
    sha256(&buf)
}

/// Map an `ImplementationLevel` to its tier number for the
/// `artifact_id` field on an L-band ContributingFact.
fn lband_tier(level: ImplementationLevel) -> u32 {
    match level {
        ImplementationLevel::L0_CitedOnly => 0,
        ImplementationLevel::L1_Canonicalised => 1,
        ImplementationLevel::L2_DeterministicFormula => 2,
        ImplementationLevel::L3_CpuImplemented => 3,
        ImplementationLevel::L4_CpuVerified => 4,
        ImplementationLevel::L5_GpuImplemented => 5,
        ImplementationLevel::L6_CpuGpuByteEquivalent => 6,
        ImplementationLevel::L7_BenchmarkCharacterised => 7,
        ImplementationLevel::L8_LedgerCharacterised => 8,
    }
}

/// Compose the operator-readable message for an L-band fact.
fn lband_message(level: ImplementationLevel) -> &'static str {
    match level {
        ImplementationLevel::L0_CitedOnly => "L0 (cited only) — no canonicalised form yet",
        ImplementationLevel::L1_Canonicalised => "L1 (canonicalised) — no host implementation",
        ImplementationLevel::L2_DeterministicFormula => {
            "L2 (deterministic formula) — no host implementation"
        }
        ImplementationLevel::L3_CpuImplemented => "L3 (CPU implementation, unverified)",
        ImplementationLevel::L4_CpuVerified => "L4 (CPU implementation, verified)",
        ImplementationLevel::L5_GpuImplemented => "L5 (GPU implementation, no CPU equivalence)",
        ImplementationLevel::L6_CpuGpuByteEquivalent => {
            "L6 (CPU/GPU byte-equivalent; dsfb-gpu-debug-core bank surface)"
        }
        ImplementationLevel::L7_BenchmarkCharacterised => {
            "L7 (benchmark-characterised; gated until measured artifact lands)"
        }
        ImplementationLevel::L8_LedgerCharacterised => {
            "L8 (ledger-characterised; gated until measured (task × dataset) evidence lands)"
        }
    }
}

/// Synthetic citation hash for a coverage hole. Mirrors the
/// helper in `activation.rs`; we duplicate the body (rather
/// than re-exporting it) to keep `activation_audit.rs`
/// self-contained for review.
fn coverage_hole_citation_hash(h: &CoverageHoleEntry) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1:HOLE\0");
    buf.extend_from_slice(&h.hole_id.0.to_be_bytes());
    buf.extend_from_slice(h.kind.as_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(h.severity.as_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(h.reason.as_str().as_bytes());
    sha256(&buf)
}

/// Synthetic citation hash for a contraindication receipt.
fn contraindication_citation_hash(canonical_id: DetectorCanonicalId) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(48);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1:CONTRAINDICATION\0");
    buf.extend_from_slice(&canonical_id.0.to_be_bytes());
    sha256(&buf)
}

/// Synthetic citation hash for a challenge entry.
fn challenge_citation_hash(c: &ChallengeDocketEntry) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-TRANSCRIPT:v1:CHALLENGE\0");
    buf.extend_from_slice(&c.challenge_id.0.to_be_bytes());
    buf.extend_from_slice(c.status.as_str().as_bytes());
    sha256(&buf)
}

/// Build the transcript for a single canonical detector by
/// walking the sealed court stack. Two builds against the same
/// stack produce byte-identical bytes. Returns `None` for an
/// unknown canonical_id.
#[must_use]
pub fn build_transcript_for(
    canonical_id: DetectorCanonicalId,
) -> Option<ActivationDecisionTranscript> {
    let record = SEED.iter().find(|r| r.canonical_id == canonical_id)?;
    let passport = passport_for(canonical_id)?;
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let decision = plan
        .decisions
        .iter()
        .find(|d| d.canonical_id == canonical_id)?;

    let contraindications = collect_contraindications();
    let coverage = collect_coverage_holes();
    let docket = collect_challenge_docket();

    let mut facts: Vec<ContributingFact> = Vec::new();

    // (1) Passport is always cited — informational at minimum.
    let passport_role = match decision.activation_status {
        ActivationStatus::Enabled | ActivationStatus::WarnOnly => FactRole::Supporting,
        ActivationStatus::Disabled | ActivationStatus::Deferred => FactRole::Informational,
    };
    facts.push(ContributingFact {
        artifact_kind: ArtifactKind::Passport,
        artifact_id: canonical_id.0,
        artifact_hash: passport.passport_hash,
        role: passport_role,
        reason_code: "PassportPresent",
        operator_message: "Passport is the per-detector legal-identity anchor (T.11a).",
    });

    // (2) L-band.
    let lband = record.implementation_status;
    let lband_role = if matches!(
        decision.disabled_reason,
        Some(DisabledReason::DisabledByWeakLBand)
    ) {
        FactRole::Blocking
    } else if matches!(
        lband,
        ImplementationLevel::L5_GpuImplemented | ImplementationLevel::L6_CpuGpuByteEquivalent
    ) {
        FactRole::Supporting
    } else {
        FactRole::Informational
    };
    facts.push(ContributingFact {
        artifact_kind: ArtifactKind::LBand,
        artifact_id: lband_tier(lband),
        artifact_hash: lband_citation_hash(canonical_id, lband),
        role: lband_role,
        reason_code: lband.as_str(),
        operator_message: lband_message(lband),
    });

    // (3) Coverage holes (per-detector and family-level).
    for h in &coverage.holes {
        let touches_me = match h.subject {
            CoverageHoleSubject::Detector(id) => id == canonical_id,
            CoverageHoleSubject::Family(f) => f == record.primitive_family,
            _ => false,
        };
        if !touches_me {
            continue;
        }
        let role = match h.severity {
            CoverageHoleSeverity::Critical | CoverageHoleSeverity::High => FactRole::Blocking,
            CoverageHoleSeverity::Medium => FactRole::Warning,
            CoverageHoleSeverity::Low => FactRole::Informational,
        };
        facts.push(ContributingFact {
            artifact_kind: ArtifactKind::CoverageHole,
            artifact_id: h.hole_id.0,
            artifact_hash: coverage_hole_citation_hash(h),
            role,
            reason_code: h.reason.as_str(),
            operator_message: coverage_hole_message(h),
        });
    }

    // (4) Contraindication.
    let contra = contraindications
        .receipts
        .iter()
        .find(|c| c.canonical_id == canonical_id);
    if let Some(c) = contra {
        let role = if c.do_not_use_for.is_empty() {
            FactRole::Informational
        } else {
            // At S1.3a contraindications attach as warnings,
            // not blocks. The transcript records that role.
            FactRole::Warning
        };
        facts.push(ContributingFact {
            artifact_kind: ArtifactKind::Contraindication,
            artifact_id: canonical_id.0,
            artifact_hash: contraindication_citation_hash(canonical_id),
            role,
            reason_code: "ContraindicationReceiptPresent",
            operator_message: contraindication_message(c),
        });
    }

    // (5) Challenges targeting this detector.
    for c in &docket.challenges {
        let targets_me = matches!(
            c.target,
            ChallengeTarget::Detector(id) | ChallengeTarget::Passport(id)
                if id == canonical_id
        );
        if !targets_me {
            continue;
        }
        let role = match c.status {
            crate::challenge_docket::ChallengeStatus::Sustained => FactRole::Blocking,
            crate::challenge_docket::ChallengeStatus::Open => {
                if matches!(
                    c.severity,
                    crate::challenge_docket::ChallengeSeverity::Critical
                ) {
                    FactRole::Blocking
                } else {
                    FactRole::Warning
                }
            }
            crate::challenge_docket::ChallengeStatus::Deferred => FactRole::Warning,
            _ => FactRole::Informational,
        };
        facts.push(ContributingFact {
            artifact_kind: ArtifactKind::Challenge,
            artifact_id: c.challenge_id.0,
            artifact_hash: challenge_citation_hash(c),
            role,
            reason_code: c.status.as_str(),
            operator_message: c.claim,
        });
    }

    // (6) GPU surface bonus fact (Supporting) for the five
    //     role-seeded GPU IDs.
    if GPU_IMPLEMENTED_CANONICAL_IDS.contains(&canonical_id) {
        facts.push(ContributingFact {
            artifact_kind: ArtifactKind::Passport,
            artifact_id: canonical_id.0,
            artifact_hash: passport.passport_hash,
            role: FactRole::Supporting,
            reason_code: "GpuSurfaceRoleSeeded",
            operator_message:
                "Detector is one of the five GPU-implemented dsfb-gpu-debug-core bank-surface IDs.",
        });
    }

    // (7) Corpus + Registry anchors (Informational).
    facts.push(ContributingFact {
        artifact_kind: ArtifactKind::CorpusHash,
        artifact_id: 0,
        artifact_hash: plan.corpus_hash_v1,
        role: FactRole::Informational,
        reason_code: "AnchorBound",
        operator_message: "All facts hash-bound to T.10 corpus_hash_v1.",
    });
    facts.push(ContributingFact {
        artifact_kind: ArtifactKind::RegistryHash,
        artifact_id: 0,
        artifact_hash: plan.registry_hash_v2,
        role: FactRole::Informational,
        reason_code: "AnchorBound",
        operator_message: "All facts hash-bound to S1.2 registry_hash_v2.",
    });

    // Sort facts deterministically.
    facts.sort_by(|a, b| {
        (a.artifact_kind, a.artifact_id, a.role).cmp(&(b.artifact_kind, b.artifact_id, b.role))
    });

    // Blocking chain: every Blocking fact in encounter order.
    let mut blocking_chain: Vec<BlockingLink> = Vec::new();
    let mut link_order = 1u32;
    for f in &facts {
        if matches!(f.role, FactRole::Blocking) {
            blocking_chain.push(BlockingLink {
                link_order,
                fact: f.clone(),
            });
            link_order += 1;
        }
    }

    // Counterfactual path to Enabled.
    let counterfactual_path_to_enabled = derive_counterfactual_path(decision, lband, &facts);

    let final_reason = FinalReason {
        wire_name: match (decision.enabled_reason, decision.disabled_reason) {
            (Some(r), _) => r.as_str(),
            (_, Some(r)) => r.as_str(),
            _ => "Unknown",
        },
    };

    let mut transcript = ActivationDecisionTranscript {
        canonical_id,
        display_name: record.display_name,
        activation_status: decision.activation_status,
        final_reason,
        contributing_facts: facts,
        blocking_chain,
        counterfactual_path_to_enabled,
        transcript_hash_v1: [0u8; 32],
    };
    transcript.transcript_hash_v1 = compute_transcript_hash_v1(&transcript);
    Some(transcript)
}

/// Build an `ActivationPlanAuditV1` over every decision in the
/// canonical plan. Useful for `bulk-emit` export.
#[must_use]
pub fn build_plan_audit() -> ActivationPlanAuditV1 {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let mut transcripts: Vec<ActivationDecisionTranscript> = plan
        .decisions
        .iter()
        .filter_map(|d| build_transcript_for(d.canonical_id))
        .collect();
    transcripts.sort_by_key(|t| t.canonical_id.0);
    ActivationPlanAuditV1 {
        source_activation_plan_hash_v1: plan.activation_plan_hash_v1,
        transcripts,
    }
}

/// Operator-readable message for a coverage hole. Composes a
/// short categorical sentence from the hole's severity + reason.
fn coverage_hole_message(h: &CoverageHoleEntry) -> &'static str {
    use crate::coverage_holes::CoverageHoleReason as R;
    match h.reason {
        R::DetectorMissingClosestAliases => "Receipt missing closest_aliases declaration.",
        R::DetectorMissingClosestNonAliases => "Receipt missing closest_non_aliases declaration.",
        R::DetectorMissingAdversarialTwins => "Receipt missing adversarial_twins declaration.",
        R::DetectorMissingGenealogyEdge => "Detector lacks a genealogy edge.",
        R::FamilyMissingConfuserCoverage => "Family has Primary witness but no Confuser.",
        R::FamilyMissingCleanWindowWitness => "Family has no CleanWindow witness.",
        R::FamilyMissingBoundaryOrRecoveryWitness => "Family has no Boundary / Recovery witness.",
        R::FamilyImplementationBandTooLow => "Family roster sits at L0/L1/L2.",
        R::LBandL7OrL8GatedByMissingArtifact => "L7/L8 gated until measured artifact lands.",
        R::FamilyMissingGpuFamilyMapping => "Family lacks GPU-family kernel mapping.",
        R::SemanticsMissingSamplingLaw => "Receipt missing required_sampling_law.",
        R::SemanticsMissingUnitSemantics => "Receipt missing required_units.",
        R::SemanticsMissingInputContractDeclaration => "input_requirements bitset is empty.",
        R::SemanticsTimeSeriesWithoutRegularityAssumption => {
            "Time-series detector lacks regularity assumption."
        }
        R::SemanticsSpectralWithoutSampleRateAssumption => {
            "Spectral detector lacks sample-rate assumption."
        }
        R::JurisprudenceThinPrecedentSupport => "Thin precedent support for this detector.",
        R::JurisprudenceGrammarRuleWithFewPrecedentLinks => {
            "Grammar rule with few precedent links."
        }
        R::JurisprudenceChallengeKindWithoutContraindicationCrossLink => {
            "ChallengeKind has no contraindication cross-link."
        }
        R::JurisprudenceOverruledOrDeferredChallengeLacksFutureGate => {
            "Overruled / Deferred challenge lacks a future gate."
        }
        R::SourceRefOlderThanThresholdWithoutModernValidation => {
            "Source ref older than threshold without modern validation."
        }
        R::SourceRefMissingDoiOrUrlWhereExpected => "Source ref missing DOI/URL where expected.",
        R::SourceRefEngineeringPracticeNeedingLaterCitation => {
            "Engineering-practice provenance needs later citation."
        }
        R::SourceRefAliasWithWeakSourceSupport => "Alias has weak source support.",
        R::ReasonCodeCoverageIncompleteOnSurface => {
            "Surface reason-code coverage tally is below 100%."
        }
    }
}

/// Operator-readable message for a contraindication receipt
/// based on whether it has a non-empty `do_not_use_for` set.
fn contraindication_message(c: &DetectorContraindicationReceiptV1) -> &'static str {
    if c.do_not_use_for.is_empty() {
        "Contraindication receipt has no do_not_use_for disqualifiers."
    } else {
        "Contraindication receipt declares do_not_use_for disqualifiers; warning attached."
    }
}

/// Derive the counterfactual path: what categorical changes
/// would lift the decision to Enabled. Empty for already-Enabled
/// detectors. The list is deterministic and sorted.
fn derive_counterfactual_path(
    decision: &DetectorActivationDecision,
    lband: ImplementationLevel,
    facts: &[ContributingFact],
) -> Vec<CounterfactualStep> {
    let mut steps: BTreeSet<CounterfactualStep> = BTreeSet::new();
    if matches!(
        decision.activation_status,
        ActivationStatus::Enabled | ActivationStatus::WarnOnly
    ) {
        return Vec::new();
    }
    // Weak L-band: promote.
    if matches!(
        decision.disabled_reason,
        Some(DisabledReason::DisabledByWeakLBand)
    ) {
        steps.insert(CounterfactualStep::PromoteLBandToL3OrHigher);
    }
    // L4 -> L5/L6 requires a benchmark receipt (informational
    // helper for downstream campaigns).
    if matches!(lband, ImplementationLevel::L4_CpuVerified) {
        steps.insert(CounterfactualStep::LandBenchmarkReceipt);
    }
    // Disable driven by coverage hole / contraindication /
    // challenge: declare the missing facts.
    for f in facts {
        if !matches!(f.role, FactRole::Blocking | FactRole::Warning) {
            continue;
        }
        match f.artifact_kind {
            ArtifactKind::CoverageHole => {
                // Pick the most informative remediation per
                // coverage-hole reason code.
                if f.reason_code == "SemanticsMissingSamplingLaw" {
                    steps.insert(CounterfactualStep::DeclareRequiredSamplingLaw);
                } else if f.reason_code == "SemanticsMissingUnitSemantics" {
                    steps.insert(CounterfactualStep::DeclareRequiredUnits);
                } else if f.reason_code == "FamilyMissingConfuserCoverage" {
                    steps.insert(CounterfactualStep::AddFamilyConfuserWitness);
                } else if f.reason_code.starts_with("SourceRef") {
                    steps.insert(CounterfactualStep::RefreshSourceProvenance);
                } else if f.reason_code.starts_with("DetectorMissing") {
                    steps.insert(CounterfactualStep::DeclareContraindicationFields);
                }
            }
            ArtifactKind::Challenge => {
                steps.insert(CounterfactualStep::ResolveChallenge);
            }
            _ => {}
        }
    }
    if matches!(
        decision.disabled_reason,
        Some(DisabledReason::DisabledByDomainMismatch)
    ) {
        steps.insert(CounterfactualStep::SupplyEvidenceContract);
    }
    steps.into_iter().collect()
}

// ---------------------------------------------------------------
// Canonical-byte serialisation + hash (transcript)
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

/// Serialise a `ContributingFact` to the canonical byte buffer.
/// Field order: kind / id / hash / role / reason / message.
/// Every enum is written by its wire name.
fn write_fact(out: &mut Vec<u8>, f: &ContributingFact) {
    write_str(out, f.artifact_kind.as_str());
    write_u32(out, f.artifact_id);
    write_bytes(out, &f.artifact_hash);
    write_str(out, f.role.as_str());
    write_str(out, f.reason_code);
    write_str(out, f.operator_message);
}

/// Compute the transcript hash. Two calls on the same transcript
/// body produce byte-identical bytes; the
/// `transcript_hash_v1` field itself is excluded.
#[must_use]
pub fn compute_transcript_hash_v1(t: &ActivationDecisionTranscript) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
    buf.extend_from_slice(ACTIVATION_TRANSCRIPT_DOMAIN.as_bytes());
    write_str(&mut buf, ACTIVATION_TRANSCRIPT_SCHEMA_V1);
    write_u32(&mut buf, t.canonical_id.0);
    write_str(&mut buf, t.display_name);
    write_str(&mut buf, t.activation_status.as_str());
    write_str(&mut buf, t.final_reason.wire_name);
    write_u32(
        &mut buf,
        u32::try_from(t.contributing_facts.len()).unwrap_or(u32::MAX),
    );
    for f in &t.contributing_facts {
        write_fact(&mut buf, f);
    }
    write_u32(
        &mut buf,
        u32::try_from(t.blocking_chain.len()).unwrap_or(u32::MAX),
    );
    for link in &t.blocking_chain {
        write_u32(&mut buf, link.link_order);
        write_fact(&mut buf, &link.fact);
    }
    write_u32(
        &mut buf,
        u32::try_from(t.counterfactual_path_to_enabled.len()).unwrap_or(u32::MAX),
    );
    for step in &t.counterfactual_path_to_enabled {
        write_str(&mut buf, step.as_str());
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Diff
// ---------------------------------------------------------------

/// Build an `ActivationDiffV1` between two plans. The diff is
/// **structural** (categorical change kinds) — it does not byte-
/// compare. Two plans built against the same court stack
/// produce an empty diff with zero counts.
///
/// **Panel-locked rule**: two plans whose `corpus_hash_v1`
/// differ cannot be diffed; the verifier rejects such a diff
/// via `DiffRejectsMismatchedCorpusHash`. Diffing across corpus
/// generations is meaningless because the evidence base
/// changed under both feet.
#[must_use]
pub fn build_diff(old: &ActivationPlanV1, new: &ActivationPlanV1) -> ActivationDiffV1 {
    let mut rows: Vec<ActivationDiffRow> = Vec::new();
    let old_by_id: alloc::collections::BTreeMap<u32, &DetectorActivationDecision> = old
        .decisions
        .iter()
        .map(|d| (d.canonical_id.0, d))
        .collect();
    let new_by_id: alloc::collections::BTreeMap<u32, &DetectorActivationDecision> = new
        .decisions
        .iter()
        .map(|d| (d.canonical_id.0, d))
        .collect();
    let mut all_ids: BTreeSet<u32> = BTreeSet::new();
    all_ids.extend(old_by_id.keys());
    all_ids.extend(new_by_id.keys());

    let mut added = 0u32;
    let mut removed = 0u32;
    let mut status_changed = 0u32;
    let mut reason_changed = 0u32;
    let mut citation_changed = 0u32;

    for id in all_ids {
        let old_d = old_by_id.get(&id).copied();
        let new_d = new_by_id.get(&id).copied();
        match (old_d, new_d) {
            (None, Some(n)) => {
                added += 1;
                rows.push(ActivationDiffRow {
                    canonical_id: DetectorCanonicalId(id),
                    kind: DiffChangeKind::DecisionAdded,
                    old_status: "",
                    new_status: n.activation_status.as_str(),
                    old_reason: "",
                    new_reason: reason_wire(n),
                });
            }
            (Some(o), None) => {
                removed += 1;
                rows.push(ActivationDiffRow {
                    canonical_id: DetectorCanonicalId(id),
                    kind: DiffChangeKind::DecisionRemoved,
                    old_status: o.activation_status.as_str(),
                    new_status: "",
                    old_reason: reason_wire(o),
                    new_reason: "",
                });
            }
            (Some(o), Some(n)) => {
                let status_diff = o.activation_status != n.activation_status;
                let reason_diff = reason_wire(o) != reason_wire(n);
                let citation_diff = o.cited_challenge_ids != n.cited_challenge_ids
                    || o.cited_contraindication_ids != n.cited_contraindication_ids
                    || o.cited_coverage_hole_ids != n.cited_coverage_hole_ids
                    || o.blocking_receipt_hashes != n.blocking_receipt_hashes
                    || o.warning_receipt_hashes != n.warning_receipt_hashes;
                if status_diff {
                    status_changed += 1;
                    rows.push(ActivationDiffRow {
                        canonical_id: DetectorCanonicalId(id),
                        kind: DiffChangeKind::StatusChanged,
                        old_status: o.activation_status.as_str(),
                        new_status: n.activation_status.as_str(),
                        old_reason: reason_wire(o),
                        new_reason: reason_wire(n),
                    });
                } else if reason_diff {
                    reason_changed += 1;
                    rows.push(ActivationDiffRow {
                        canonical_id: DetectorCanonicalId(id),
                        kind: DiffChangeKind::ReasonChanged,
                        old_status: o.activation_status.as_str(),
                        new_status: n.activation_status.as_str(),
                        old_reason: reason_wire(o),
                        new_reason: reason_wire(n),
                    });
                } else if citation_diff {
                    citation_changed += 1;
                    rows.push(ActivationDiffRow {
                        canonical_id: DetectorCanonicalId(id),
                        kind: DiffChangeKind::CitationChanged,
                        old_status: o.activation_status.as_str(),
                        new_status: n.activation_status.as_str(),
                        old_reason: reason_wire(o),
                        new_reason: reason_wire(n),
                    });
                }
            }
            (None, None) => {}
        }
    }
    rows.sort_by(|a, b| (a.canonical_id.0, a.kind).cmp(&(b.canonical_id.0, b.kind)));

    let mut diff = ActivationDiffV1 {
        old_activation_plan_hash_v1: old.activation_plan_hash_v1,
        new_activation_plan_hash_v1: new.activation_plan_hash_v1,
        corpus_hash_v1: new.corpus_hash_v1,
        rows,
        decisions_added: added,
        decisions_removed: removed,
        decisions_status_changed: status_changed,
        decisions_reason_changed: reason_changed,
        decisions_citation_changed: citation_changed,
        activation_diff_hash_v1: [0u8; 32],
    };
    diff.activation_diff_hash_v1 = compute_diff_hash_v1(&diff);
    diff
}

/// Extract the wire-name reason from a decision (whichever of
/// enabled_reason / disabled_reason is Some). Returns `""` if
/// neither is set (a structural defect surfaced by the
/// activation-plan verifier).
fn reason_wire(d: &DetectorActivationDecision) -> &'static str {
    if let Some(r) = d.enabled_reason {
        r.as_str()
    } else if let Some(r) = d.disabled_reason {
        r.as_str()
    } else {
        ""
    }
}

/// Serialise an `ActivationDiffRow`. Field order is canonical
/// hash order; every string is length-prefixed.
fn write_diff_row(out: &mut Vec<u8>, r: &ActivationDiffRow) {
    write_u32(out, r.canonical_id.0);
    write_str(out, r.kind.as_str());
    write_str(out, r.old_status);
    write_str(out, r.new_status);
    write_str(out, r.old_reason);
    write_str(out, r.new_reason);
}

/// Compute `activation_diff_hash_v1`. Excludes its own field.
#[must_use]
pub fn compute_diff_hash_v1(d: &ActivationDiffV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
    buf.extend_from_slice(ACTIVATION_DIFF_DOMAIN.as_bytes());
    write_str(&mut buf, ACTIVATION_DIFF_SCHEMA_V1);
    write_bytes(&mut buf, &d.old_activation_plan_hash_v1);
    write_bytes(&mut buf, &d.new_activation_plan_hash_v1);
    write_bytes(&mut buf, &d.corpus_hash_v1);
    write_u32(&mut buf, u32::try_from(d.rows.len()).unwrap_or(u32::MAX));
    for r in &d.rows {
        write_diff_row(&mut buf, r);
    }
    write_u32(&mut buf, d.decisions_added);
    write_u32(&mut buf, d.decisions_removed);
    write_u32(&mut buf, d.decisions_status_changed);
    write_u32(&mut buf, d.decisions_reason_changed);
    write_u32(&mut buf, d.decisions_citation_changed);
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verifier error for transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TranscriptVerifyError {
    /// Subject canonical_id (0 for plan-level).
    pub canonical_id: DetectorCanonicalId,
    /// Categorical failure kind.
    pub kind: TranscriptVerifyErrorKind,
}

/// Categorical reject kinds for transcripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranscriptVerifyErrorKind {
    /// Disabled / Deferred transcript without any Blocking
    /// ContributingFact (load-bearing).
    DisabledTranscriptWithoutBlockingFact,
    /// Transcript references a canonical_id not in SEED.
    UnknownDetector,
    /// final_reason wire name is empty (corresponds to a plan
    /// decision missing both reasons).
    FinalReasonMissing,
    /// transcript_hash_v1 is all zeros.
    TranscriptHashMissing,
}

impl TranscriptVerifyErrorKind {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisabledTranscriptWithoutBlockingFact => "DisabledTranscriptWithoutBlockingFact",
            Self::UnknownDetector => "UnknownDetector",
            Self::FinalReasonMissing => "FinalReasonMissing",
            Self::TranscriptHashMissing => "TranscriptHashMissing",
        }
    }
}

/// Walk a transcript and return one error per defect.
#[must_use]
pub fn verify_transcript(t: &ActivationDecisionTranscript) -> Vec<TranscriptVerifyError> {
    let mut errors: Vec<TranscriptVerifyError> = Vec::new();
    let known: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    if !known.contains(&t.canonical_id.0) {
        errors.push(TranscriptVerifyError {
            canonical_id: t.canonical_id,
            kind: TranscriptVerifyErrorKind::UnknownDetector,
        });
    }
    if t.final_reason.wire_name.is_empty() {
        errors.push(TranscriptVerifyError {
            canonical_id: t.canonical_id,
            kind: TranscriptVerifyErrorKind::FinalReasonMissing,
        });
    }
    if t.transcript_hash_v1 == [0u8; 32] {
        errors.push(TranscriptVerifyError {
            canonical_id: t.canonical_id,
            kind: TranscriptVerifyErrorKind::TranscriptHashMissing,
        });
    }
    if matches!(
        t.activation_status,
        ActivationStatus::Disabled | ActivationStatus::Deferred
    ) {
        let has_blocking = t
            .contributing_facts
            .iter()
            .any(|f| matches!(f.role, FactRole::Blocking));
        if !has_blocking {
            errors.push(TranscriptVerifyError {
                canonical_id: t.canonical_id,
                kind: TranscriptVerifyErrorKind::DisabledTranscriptWithoutBlockingFact,
            });
        }
    }
    errors
}

/// Verifier error for diffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiffVerifyError {
    /// Categorical failure kind.
    pub kind: DiffVerifyErrorKind,
}

/// Categorical reject kinds for diffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffVerifyErrorKind {
    /// The two plans have different `corpus_hash_v1` — meaningless
    /// to diff across corpus generations.
    DiffRejectsMismatchedCorpusHash,
    /// `activation_diff_hash_v1` is all zeros (un-finalised).
    DiffHashMissing,
}

impl DiffVerifyErrorKind {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DiffRejectsMismatchedCorpusHash => "DiffRejectsMismatchedCorpusHash",
            Self::DiffHashMissing => "DiffHashMissing",
        }
    }
}

/// Verify a diff between two plans against their corpus
/// anchors. Caller supplies the source plans so the verifier
/// can crosscheck the diff's `corpus_hash_v1` against both.
#[must_use]
pub fn verify_diff(
    diff: &ActivationDiffV1,
    old: &ActivationPlanV1,
    new: &ActivationPlanV1,
) -> Vec<DiffVerifyError> {
    let mut errors: Vec<DiffVerifyError> = Vec::new();
    if old.corpus_hash_v1 != new.corpus_hash_v1 || diff.corpus_hash_v1 != new.corpus_hash_v1 {
        errors.push(DiffVerifyError {
            kind: DiffVerifyErrorKind::DiffRejectsMismatchedCorpusHash,
        });
    }
    if diff.activation_diff_hash_v1 == [0u8; 32] {
        errors.push(DiffVerifyError {
            kind: DiffVerifyErrorKind::DiffHashMissing,
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

/// Render a transcript as deterministic text. Two calls produce
/// byte-identical strings.
#[must_use]
pub fn render_transcript_text(t: &ActivationDecisionTranscript) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - ActivationDecisionTranscript (S1.3b)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "canonical_id              : {}\n",
        t.canonical_id.0
    ));
    out.push_str(&format!("display_name              : {}\n", t.display_name));
    out.push_str(&format!(
        "activation_status         : {}\n",
        t.activation_status.as_str()
    ));
    out.push_str(&format!(
        "final_reason              : {}\n",
        t.final_reason.wire_name
    ));
    out.push_str(&format!(
        "transcript_hash_v1        : {}\n\n",
        hex(&t.transcript_hash_v1)
    ));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str("Contributing facts (sorted by kind / id / role)\n");
    out.push_str("----------------------------------------------------------------\n");
    for f in &t.contributing_facts {
        out.push_str(&format!(
            "  [{:<14}] id={:<6} role={:<13} reason={:<48} {}\n",
            f.artifact_kind.as_str(),
            f.artifact_id,
            f.role.as_str(),
            f.reason_code,
            f.operator_message,
        ));
    }
    if !t.blocking_chain.is_empty() {
        out.push('\n');
        out.push_str("----------------------------------------------------------------\n");
        out.push_str("Blocking chain (root cause first)\n");
        out.push_str("----------------------------------------------------------------\n");
        for link in &t.blocking_chain {
            out.push_str(&format!(
                "  #{:<3} [{:<14}] id={:<6} reason={}\n",
                link.link_order,
                link.fact.artifact_kind.as_str(),
                link.fact.artifact_id,
                link.fact.reason_code,
            ));
        }
    }
    if !t.counterfactual_path_to_enabled.is_empty() {
        out.push('\n');
        out.push_str("----------------------------------------------------------------\n");
        out.push_str("Counterfactual path to Enabled\n");
        out.push_str("----------------------------------------------------------------\n");
        for step in &t.counterfactual_path_to_enabled {
            out.push_str(&format!("  - {}\n", step.as_str()));
        }
    }
    out
}

/// Render an `ActivationPlanAuditV1` (one transcript per
/// detector) as deterministic text.
#[must_use]
pub fn render_audit_text(audit: &ActivationPlanAuditV1) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - ActivationPlanAuditV1 (S1.3b)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "source_activation_plan_hash_v1 : {}\n",
        hex(&audit.source_activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "transcript_count               : {}\n\n",
        audit.transcripts.len()
    ));
    for t in &audit.transcripts {
        out.push_str(&render_transcript_text(t));
        out.push('\n');
    }
    out
}

/// Render a transcript as deterministic JSON.
#[must_use]
pub fn render_transcript_json(t: &ActivationDecisionTranscript) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("{\n");
    out.push_str(&format!("  \"canonical_id\": {},\n", t.canonical_id.0));
    out.push_str(&format!(
        "  \"display_name\": \"{}\",\n",
        json_escape(t.display_name)
    ));
    out.push_str(&format!(
        "  \"activation_status\": \"{}\",\n",
        t.activation_status.as_str()
    ));
    out.push_str(&format!(
        "  \"final_reason\": \"{}\",\n",
        t.final_reason.wire_name
    ));
    out.push_str(&format!(
        "  \"transcript_hash_v1\": \"{}\",\n",
        hex(&t.transcript_hash_v1)
    ));
    out.push_str("  \"contributing_facts\": [\n");
    for (i, f) in t.contributing_facts.iter().enumerate() {
        let comma = if i + 1 == t.contributing_facts.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            "    {{ \"artifact_kind\": \"{}\", \"artifact_id\": {}, \"artifact_hash\": \"{}\", \"role\": \"{}\", \"reason_code\": \"{}\", \"operator_message\": \"{}\" }}{comma}\n",
            f.artifact_kind.as_str(),
            f.artifact_id,
            hex(&f.artifact_hash),
            f.role.as_str(),
            f.reason_code,
            json_escape(f.operator_message),
        ));
    }
    out.push_str("  ],\n  \"blocking_chain\": [");
    for (i, link) in t.blocking_chain.iter().enumerate() {
        let comma = if i + 1 == t.blocking_chain.len() {
            ""
        } else {
            ", "
        };
        out.push_str(&format!(
            "{{ \"link_order\": {}, \"artifact_kind\": \"{}\", \"artifact_id\": {}, \"reason_code\": \"{}\" }}{comma}",
            link.link_order,
            link.fact.artifact_kind.as_str(),
            link.fact.artifact_id,
            link.fact.reason_code,
        ));
    }
    out.push_str("],\n  \"counterfactual_path_to_enabled\": [");
    for (i, step) in t.counterfactual_path_to_enabled.iter().enumerate() {
        let comma = if i + 1 == t.counterfactual_path_to_enabled.len() {
            ""
        } else {
            ", "
        };
        out.push_str(&format!("\"{}\"{comma}", step.as_str()));
    }
    out.push_str("]\n}\n");
    out
}

/// Render an audit as deterministic JSON.
#[must_use]
pub fn render_audit_json(audit: &ActivationPlanAuditV1) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"source_activation_plan_hash_v1\": \"{}\",\n",
        hex(&audit.source_activation_plan_hash_v1)
    ));
    out.push_str("  \"transcripts\": [\n");
    for (i, t) in audit.transcripts.iter().enumerate() {
        let comma = if i + 1 == audit.transcripts.len() {
            ""
        } else {
            ","
        };
        let body = render_transcript_json(t);
        // Indent the nested object by 4 spaces and trim trailing newline.
        let mut indented = String::with_capacity(body.len() + 64);
        for line in body.trim_end().lines() {
            indented.push_str("    ");
            indented.push_str(line);
            indented.push('\n');
        }
        out.push_str(&indented);
        out.push_str(comma);
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

/// Render an `ActivationDiffV1` as deterministic text.
#[must_use]
pub fn render_diff_text(d: &ActivationDiffV1) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - ActivationDiffV1 (S1.3b)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "old_activation_plan_hash_v1 : {}\n",
        hex(&d.old_activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "new_activation_plan_hash_v1 : {}\n",
        hex(&d.new_activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "corpus_hash_v1              : {}\n",
        hex(&d.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "activation_diff_hash_v1     : {}\n\n",
        hex(&d.activation_diff_hash_v1)
    ));
    out.push_str("----------------------------------------------------------------\n");
    out.push_str("Change summary\n");
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!("  DecisionAdded     : {}\n", d.decisions_added));
    out.push_str(&format!("  DecisionRemoved   : {}\n", d.decisions_removed));
    out.push_str(&format!(
        "  StatusChanged     : {}\n",
        d.decisions_status_changed
    ));
    out.push_str(&format!(
        "  ReasonChanged     : {}\n",
        d.decisions_reason_changed
    ));
    out.push_str(&format!(
        "  CitationChanged   : {}\n\n",
        d.decisions_citation_changed
    ));
    if d.rows.is_empty() {
        out.push_str("(no per-detector changes)\n");
    } else {
        out.push_str("----------------------------------------------------------------\n");
        out.push_str("Per-detector changes\n");
        out.push_str("----------------------------------------------------------------\n");
        for r in &d.rows {
            out.push_str(&format!(
                "  #{:<3} {:<18} {:<10} -> {:<10} reason: {} -> {}\n",
                r.canonical_id.0,
                r.kind.as_str(),
                r.old_status,
                r.new_status,
                r.old_reason,
                r.new_reason,
            ));
        }
    }
    out
}

/// Render an `ActivationDiffV1` as deterministic JSON.
#[must_use]
pub fn render_diff_json(d: &ActivationDiffV1) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"old_activation_plan_hash_v1\": \"{}\",\n",
        hex(&d.old_activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "  \"new_activation_plan_hash_v1\": \"{}\",\n",
        hex(&d.new_activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "  \"corpus_hash_v1\": \"{}\",\n",
        hex(&d.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "  \"activation_diff_hash_v1\": \"{}\",\n",
        hex(&d.activation_diff_hash_v1)
    ));
    out.push_str(&format!("  \"decisions_added\": {},\n", d.decisions_added));
    out.push_str(&format!(
        "  \"decisions_removed\": {},\n",
        d.decisions_removed
    ));
    out.push_str(&format!(
        "  \"decisions_status_changed\": {},\n",
        d.decisions_status_changed
    ));
    out.push_str(&format!(
        "  \"decisions_reason_changed\": {},\n",
        d.decisions_reason_changed
    ));
    out.push_str(&format!(
        "  \"decisions_citation_changed\": {},\n",
        d.decisions_citation_changed
    ));
    out.push_str("  \"rows\": [\n");
    for (i, r) in d.rows.iter().enumerate() {
        let comma = if i + 1 == d.rows.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"canonical_id\": {}, \"kind\": \"{}\", \"old_status\": \"{}\", \"new_status\": \"{}\", \"old_reason\": \"{}\", \"new_reason\": \"{}\" }}{comma}\n",
            r.canonical_id.0,
            r.kind.as_str(),
            r.old_status,
            r.new_status,
            r.old_reason,
            r.new_reason,
        ));
    }
    out.push_str("  ]\n}\n");
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
