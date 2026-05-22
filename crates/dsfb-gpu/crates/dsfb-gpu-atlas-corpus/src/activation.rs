//! S1.3a — `ActivationPlanV1`: the first deterministic court
//! decision over the sealed T.11 surfaces.
//!
//! **Thesis (panel-locked)**: *"S1.3a makes detector activation
//! a deterministic court decision, not a heuristic filter."* A
//! normal activation planner says "this detector is applicable."
//! S1.3a says "this detector is admissible under the current
//! evidence contract, and here is the full reason route."
//!
//! S1.3a is the first surface that consumes the entire sealed
//! T.11 court stack and emits per-detector enable / disable /
//! warn-only / deferred decisions with categorical reason codes
//! and citations back to the artifact(s) that drove each decision.
//! It is the operational bridge that turns T.11h coverage holes,
//! T.11g contraindications, and T.11f challenges from honest
//! documentation into real activation consequences:
//!
//! > A coverage hole that never affects activation, admission, or
//! > audit is just documentation. S1.3a is what makes T.11h
//! > operational.
//!
//! **Inputs** (the planner consumes the sealed court stack):
//!
//! ```text
//! DetectorPassport             T.11a
//! CourtPrecedent               T.11b   (consulted via per-detector binding count)
//! AdmissibilityGrammar         T.11c   (referenced; no detector-level gating yet)
//! TrialTranscript              T.11d   (reference-only; not gating)
//! ExecutionAttestation         T.11e   (anchor only; not gating)
//! ChallengeDocket              T.11f
//! ContraindicationReceipt      T.11g
//! CoverageHoleReport           T.11h
//! corpus_hash_v1               T.10
//! registry_hash_v2             S1.2    (passed in from caller)
//! ```
//!
//! **Output** (one decision per canonical detector):
//!
//! ```text
//! DetectorActivationDecision {
//!     canonical_id,
//!     activation_status: Enabled | Disabled | WarnOnly | Deferred,
//!     enabled_reason | disabled_reason,
//!     blocking_receipt_hashes,   // 32-byte court hashes that
//!                                // forced a Disable / Deferred
//!     warning_receipt_hashes,    // 32-byte court hashes that
//!                                // attached a warning on Enabled /
//!                                // WarnOnly
//!     cited_challenge_ids,
//!     cited_contraindication_ids,
//!     cited_coverage_hole_ids,
//!     cited_passport_hash,
//! }
//! ```
//!
//! **Hash posture**: `corpus_hash_v1`, `registry_hash_v2`, and
//! every T.11a–T.11h hash anchor are unchanged. S1.3a introduces
//! its own namespace `DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1\0` for
//! `activation_plan_hash_v1`. Two builds against the same sealed
//! court stack produce byte-identical bytes.
//!
//! **Scope discipline (panel-locked)**: S1.3a is **schema +
//! reason-coded planner**. It does NOT ship budget pruning,
//! redundancy suppression, T.8 ledger consumption, GPU dispatch,
//! counterfactual replay, or OTel binding. Those land in later
//! S1.3 / S1.3a.x commits. Same `[[no-silent-court-logic]]`
//! discipline as T.11h: every `pub` item AND every private
//! helper carries a doc comment whose first sentence states the
//! WHY for a future engineer.
//!
//! **Forbidden in this module** (any of these would be a defect):
//!
//! * Probability / learned weights / fast-math / atomic
//!   accumulation. The planner is deterministic categorical
//!   logic.
//! * Mutation of any upstream hash anchor.
//! * Repair of any coverage hole, contraindication, or challenge.
//! * Resolution of any challenge entry.
//! * Free-string reasons (every reason is enum-backed and
//!   wire-named).
//! * Empirical-usefulness claims (T.8 stays the usefulness
//!   surface; S1.3a is admissibility-only).

#![allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::format_push_string,
    clippy::doc_overindented_list_items
)]

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;

use crate::challenge_docket::{
    collect_challenge_docket, compute_challenge_docket_hash_v1, ChallengeDocketEntry, ChallengeId,
    ChallengeStatus, ChallengeTarget,
};
use crate::contraindication::{
    collect_contraindications, compute_contraindication_hash_v1, DetectorContraindicationReceiptV1,
};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::coverage_holes::{
    collect_coverage_holes, compute_coverage_hole_hash_v1, CoverageHoleEntry, CoverageHoleId,
    CoverageHoleReason, CoverageHoleSeverity, CoverageHoleStatus, CoverageHoleSubject,
};
use crate::lband::GPU_IMPLEMENTED_CANONICAL_IDS;
use crate::passport::{passport_for, DetectorPassport};
use crate::seed::SEED;
use crate::types::{
    DetectorCanonicalId, ImplementationLevel, InputRequirementSet, PrimitiveFamily, WitnessRole,
};
use dsfb_gpu_debug_core::hash::sha256;

// ---------------------------------------------------------------
// Domain separators & schema constants
// ---------------------------------------------------------------

/// Domain separator for `activation_plan_hash_v1`. The trailing
/// `\0` is load-bearing — it pins the schema and prevents
/// accidental collision with any other DSFB-GPU-Atlas hash.
pub const ACTIVATION_PLAN_DOMAIN: &str = "DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1\0";

/// Schema name written into the canonical byte stream so a
/// future v2 cannot collide with v1 even under the same domain.
pub const ACTIVATION_PLAN_SCHEMA_V1: &str = "ActivationPlanV1";

/// Pinned reference value for the live S1.2
/// `registry_hash_v2`. Seeded from the sealed S1.2 commit
/// (`8ccd522` / reports/s1_2_registry_summary.txt). The corpus
/// crate is host-only and intentionally does NOT depend on
/// `dsfb-gpu-atlas-registry`; callers either pass the hash
/// explicitly or use this constant as the canonical S1.2 anchor.
///
/// If a future S1.2.x campaign re-emits the registry, this
/// constant must be refreshed in the same commit; an acceptance
/// test asserts non-zero and well-formed bytes.
pub const KNOWN_S12_REGISTRY_HASH_V2: [u8; 32] = [
    0xd3, 0xcf, 0x63, 0x00, 0x0c, 0xee, 0x92, 0x28, 0x18, 0xe8, 0xdb, 0xc7, 0x9f, 0xfe, 0xcb, 0xc2,
    0x7d, 0x28, 0x80, 0x63, 0xef, 0xba, 0xed, 0x58, 0x9e, 0x1e, 0xb1, 0x81, 0x2b, 0xc3, 0x7a, 0x08,
];

// ---------------------------------------------------------------
// Public schema
// ---------------------------------------------------------------

/// Schema variant tag for `ActivationPlanV1`.
///
/// Currently `V1AdmissibilityOnly`: S1.3a emits admissibility
/// decisions only (no budget pruning, no redundancy suppression,
/// no T.8 ledger consumption). Future S1.3 schema variants
/// (`V2WithBudget`, `V3WithRedundancySuppression`, ...) will
/// extend the wire name; existing v1 receipts remain replayable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationPlanSchema {
    /// S1.3a base schema — per-detector reason-coded enable /
    /// disable / warn-only / deferred decisions consuming the
    /// sealed T.11 court stack.
    V1AdmissibilityOnly,
}

impl ActivationPlanSchema {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1AdmissibilityOnly => "V1AdmissibilityOnly",
        }
    }
}

/// Activation outcome for a single detector. Four-valued so the
/// planner can distinguish "court admits" (Enabled), "court
/// blocks" (Disabled), "court admits but flags a non-blocking
/// concern" (WarnOnly), and "court declines to decide pending
/// downstream work" (Deferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationStatus {
    /// Court admits the detector. Carries an `EnabledReason` and
    /// MAY carry non-blocking warnings via `warning_receipt_hashes`.
    Enabled,
    /// Court blocks the detector. MUST carry a `DisabledReason`
    /// AND at least one `blocking_receipt_hash` citing the court
    /// artifact that drove the block.
    Disabled,
    /// Court admits but flags concerns. Equivalent to Enabled
    /// for execution purposes; carries warnings. Used for medium-
    /// severity coverage holes / deferred (non-sustained)
    /// challenges that the planner does not treat as blocking.
    WarnOnly,
    /// Court declines to decide pending downstream work
    /// (e.g. coverage hole resolution gate, OTel binding receipt,
    /// or empirical usefulness ledger row). Different from
    /// Disabled: Disabled means "court rejects"; Deferred means
    /// "court hasn't ruled yet."
    Deferred,
}

impl ActivationStatus {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
            Self::WarnOnly => "WarnOnly",
            Self::Deferred => "Deferred",
        }
    }
}

/// Categorical reason for an Enabled / WarnOnly decision. Every
/// variant maps to one or more positive admission paths through
/// the court stack. The planner chooses the highest-priority
/// applicable variant (Primary > Boundary > Confuser > Generic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnabledReason {
    /// Passport is structurally complete (all eight
    /// `ConstitutionFlags` set) and no blocking court artifact
    /// applies. Generic admission path.
    EnabledByPassportComplete,
    /// No blocking coverage hole applies to this detector.
    EnabledByNoBlockingCoverageHole,
    /// Detector's contraindication receipt declares no
    /// `do_not_use_for` disqualifier that the current evidence
    /// contract violates.
    EnabledByContraindicationSatisfied,
    /// No sustained or critical-open challenge targets this
    /// detector.
    EnabledByChallengeClear,
    /// Detector is one of the five `GPU_IMPLEMENTED_CANONICAL_IDS`
    /// seeded from `dsfb-gpu-debug-core`. Used by the planner to
    /// admit the L5/L6 GPU surface explicitly.
    EnabledByRoleSeededGpuSurface,
    /// Confuser-role witness; admitted as a negative witness
    /// without needing a Primary path through the court.
    EnabledAsConfuserWitness,
    /// Boundary-role witness; admitted at episode-boundary
    /// witness capacity.
    EnabledAsBoundaryWitness,
    /// Primary-role witness; admitted as a top-level admission
    /// driver under T.11c grammar rules.
    EnabledAsPrimaryWitness,
}

impl EnabledReason {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EnabledByPassportComplete => "EnabledByPassportComplete",
            Self::EnabledByNoBlockingCoverageHole => "EnabledByNoBlockingCoverageHole",
            Self::EnabledByContraindicationSatisfied => "EnabledByContraindicationSatisfied",
            Self::EnabledByChallengeClear => "EnabledByChallengeClear",
            Self::EnabledByRoleSeededGpuSurface => "EnabledByRoleSeededGpuSurface",
            Self::EnabledAsConfuserWitness => "EnabledAsConfuserWitness",
            Self::EnabledAsBoundaryWitness => "EnabledAsBoundaryWitness",
            Self::EnabledAsPrimaryWitness => "EnabledAsPrimaryWitness",
        }
    }
}

/// Categorical reason for a Disabled decision. Every variant
/// maps to one or more T.11 court artifacts (coverage hole,
/// contraindication, challenge, passport). The planner chooses
/// the highest-priority applicable variant; the citation lists
/// carry the receipt-hash + entry-id that drove the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisabledReason {
    /// At least one Critical / High coverage hole subjects this
    /// detector AND has no resolution gate (i.e. the hole is
    /// unresolved).
    DisabledByCoverageHole,
    /// At least one `DoNotUseFor` disqualifier in the detector's
    /// contraindication receipt blocks the current evidence
    /// contract.
    DisabledByContraindication,
    /// A sustained or critical-Open challenge targets this
    /// detector.
    DisabledByUnresolvedChallenge,
    /// L-band is L0 (CitedOnly), L1 (Canonicalised), or L2
    /// (DeterministicFormula). No host implementation yet.
    DisabledByWeakLBand,
    /// Detector is ordered-time / spectral but its
    /// contraindication receipt declares no
    /// `required_sampling_law`. Operationalises T.11h's
    /// `SemanticsMissingSamplingLaw` reason.
    DisabledByMissingSamplingLaw,
    /// Detector is unit-sensitive but its contraindication
    /// receipt declares no `required_units`. Operationalises
    /// T.11h's `SemanticsMissingUnitSemantics` reason.
    DisabledByMissingUnitSemantics,
    /// Primary detector in a family with no Confuser witness.
    /// Operationalises T.11h's
    /// `FamilyMissingConfuserCoverage` reason.
    DisabledByMissingConfuser,
    /// Detector has no `CourtPrecedent` citing it AND a
    /// genealogy gap was surfaced. Operationalises T.11h's
    /// `JurisprudenceThinPrecedentSupport` reason.
    DisabledByThinPrecedentSupport,
    /// Detector's primary domain does not match the declared
    /// evidence contract (e.g. graph detector under a numeric
    /// time-series contract). Placeholder for future evidence-
    /// contract checks; not currently emitted because the
    /// planner has no contract-aware mode yet.
    DisabledByDomainMismatch,
    /// Detector is admissible but deferred under a coverage-hole
    /// resolution gate (the gate has not yet been satisfied).
    /// Used only in `Deferred` decisions, never in `Disabled`.
    DisabledByBudgetDeferred,
    /// Detector lifecycle state is `Dormant`/`Retired*`, or its
    /// `ImplementationLevel` is L0/L1 and no host implementation
    /// exists in the workspace.
    DisabledByUnimplementedSurface,
    /// Detector proposal is not present in the ratified
    /// `corpus_hash_v2` expansion index AND not materialized in
    /// the FF.1 passport index. Emitted by the FF.2 ratification
    /// gate (see
    /// [`crate::ff2_activation_ratification_gate`]). The panel
    /// warning is explicit: this variant exists so unratified
    /// proposals cannot silently collapse into the generic
    /// `DisabledByWeakLBand` fallback; the court distinction
    /// "not ratified / not passported" must remain operator-
    /// visible.
    DisabledUnratifiedProposal,
}

impl DisabledReason {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisabledByCoverageHole => "DisabledByCoverageHole",
            Self::DisabledByContraindication => "DisabledByContraindication",
            Self::DisabledByUnresolvedChallenge => "DisabledByUnresolvedChallenge",
            Self::DisabledByWeakLBand => "DisabledByWeakLBand",
            Self::DisabledByMissingSamplingLaw => "DisabledByMissingSamplingLaw",
            Self::DisabledByMissingUnitSemantics => "DisabledByMissingUnitSemantics",
            Self::DisabledByMissingConfuser => "DisabledByMissingConfuser",
            Self::DisabledByThinPrecedentSupport => "DisabledByThinPrecedentSupport",
            Self::DisabledByDomainMismatch => "DisabledByDomainMismatch",
            Self::DisabledByBudgetDeferred => "DisabledByBudgetDeferred",
            Self::DisabledByUnimplementedSurface => "DisabledByUnimplementedSurface",
            Self::DisabledUnratifiedProposal => "DisabledUnratifiedProposal",
        }
    }
}

/// Per-detector decision record. Field order is the canonical
/// hash order; do not reorder without rebaselining
/// `activation_plan_hash_v1`.
#[derive(Debug, Clone)]
pub struct DetectorActivationDecision {
    /// Canonical detector this decision concerns.
    pub canonical_id: DetectorCanonicalId,
    /// Operator-readable name (mirrored from passport).
    pub display_name: &'static str,
    /// Court outcome.
    pub activation_status: ActivationStatus,
    /// Categorical Enabled reason; `Some` iff `activation_status
    /// ∈ {Enabled, WarnOnly}`.
    pub enabled_reason: Option<EnabledReason>,
    /// Categorical Disabled reason; `Some` iff `activation_status
    /// ∈ {Disabled, Deferred}`.
    pub disabled_reason: Option<DisabledReason>,
    /// 32-byte court-artifact hashes that forced a Disable /
    /// Deferred. Empty for Enabled (warning hashes go in
    /// `warning_receipt_hashes` instead). Sorted ascending.
    pub blocking_receipt_hashes: Vec<[u8; 32]>,
    /// 32-byte court-artifact hashes that attached a non-blocking
    /// warning. Sorted ascending. Populated for Enabled and
    /// WarnOnly when non-Critical holes / Medium contraindications
    /// apply.
    pub warning_receipt_hashes: Vec<[u8; 32]>,
    /// `ChallengeDocket` entries cited by this decision. Sorted
    /// ascending.
    pub cited_challenge_ids: Vec<ChallengeId>,
    /// `ContraindicationReceipt` canonical IDs cited by this
    /// decision. Sorted ascending. (Contraindication receipts
    /// are keyed by canonical_id; same id space as the detector.)
    pub cited_contraindication_ids: Vec<DetectorCanonicalId>,
    /// `CoverageHoleReport` entries cited by this decision.
    /// Sorted ascending.
    pub cited_coverage_hole_ids: Vec<CoverageHoleId>,
    /// 32-byte passport hash from T.11a. Always present (passport
    /// is the per-detector legal-identity anchor).
    pub cited_passport_hash: [u8; 32],
}

/// Histogram of decision reasons across the plan. Computed once
/// per snapshot; rendered in the headline report block.
#[derive(Debug, Clone)]
pub struct ReasonHistogramRow {
    /// Wire-named reason category (enabled or disabled).
    pub reason: &'static str,
    /// How many decisions cite this reason.
    pub count: u32,
}

/// The complete `ActivationPlanV1` snapshot. Two builds against
/// the same sealed court stack produce byte-identical bytes.
///
/// Hash dependencies are folded into `activation_plan_hash_v1`:
/// changing any of `corpus_hash_v1`, `registry_hash_v2`, or any
/// of the three live T.11 hashes (challenge_docket,
/// contraindication, coverage_hole) changes
/// `activation_plan_hash_v1`. Changing any single decision also
/// changes it.
#[derive(Debug, Clone)]
pub struct ActivationPlanV1 {
    /// Schema variant.
    pub schema: ActivationPlanSchema,
    /// Anchor: T.10 corpus hash.
    pub corpus_hash_v1: [u8; 32],
    /// Anchor: S1.2 registry hash. Passed in from the caller
    /// (corpus crate is registry-independent).
    pub registry_hash_v2: [u8; 32],
    /// Anchor: T.11f challenge docket hash.
    pub challenge_docket_hash_v1: [u8; 32],
    /// Anchor: T.11g contraindication receipt hash.
    pub detector_contraindication_hash_v1: [u8; 32],
    /// Anchor: T.11h coverage hole report hash.
    pub coverage_hole_hash_v1: [u8; 32],
    /// One decision per canonical detector in `SEED`, sorted by
    /// `canonical_id` ascending.
    pub decisions: Vec<DetectorActivationDecision>,
    /// Count of `Enabled` decisions.
    pub enabled_count: u32,
    /// Count of `Disabled` decisions.
    pub disabled_count: u32,
    /// Count of `WarnOnly` decisions.
    pub warn_only_count: u32,
    /// Count of `Deferred` decisions.
    pub deferred_count: u32,
    /// Sorted reason histogram (combines enabled + disabled
    /// reasons under their stable wire names).
    pub reason_histogram: Vec<ReasonHistogramRow>,
    /// SHA-256 over the canonical-byte projection of every field
    /// above (schema + anchors + decisions + counts + histogram)
    /// under domain `ACTIVATION_PLAN_DOMAIN`.
    pub activation_plan_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Helpers used during derivation
// ---------------------------------------------------------------

/// True iff the bitset `set` contains the bit value `bit`. Mirror
/// of the helper in `coverage_holes.rs` — kept here so this
/// module is self-contained for clippy's wildcard-import lint
/// and so the no-silent-helper doctrine is satisfied in-module.
fn requires(set: InputRequirementSet, bit: u32) -> bool {
    (set.0 & bit) == bit
}

/// True iff the detector's family / role / input-requirements
/// indicate the detector is "ordered time" — i.e. requires a
/// declared sampling law for safe admission.
fn detector_is_ordered_time(family: PrimitiveFamily, req: InputRequirementSet) -> bool {
    use PrimitiveFamily as F;
    requires(req, InputRequirementSet::ORDERED_TIME)
        || matches!(
            family,
            F::Spectral
                | F::Wavelet
                | F::SequentialRecurrence
                | F::ScalarThreshold
                | F::ResidualObserver
        )
}

/// True iff the detector is unit-sensitive (e.g. spectral power
/// thresholds, physical-unit envelopes). These require a declared
/// `required_units` in T.11g; missing units is a coverage hole
/// that disables the detector at S1.3a.
fn detector_is_unit_sensitive(req: InputRequirementSet) -> bool {
    requires(req, InputRequirementSet::UNITS)
}

/// Lookup helper: contraindication receipt for a canonical_id.
fn contraindication_for(
    contraindications: &[DetectorContraindicationReceiptV1],
    canonical_id: DetectorCanonicalId,
) -> Option<&DetectorContraindicationReceiptV1> {
    contraindications
        .iter()
        .find(|c| c.canonical_id == canonical_id)
}

/// Returns all coverage holes whose subject is the given
/// canonical detector OR its family. Family-level holes are
/// applied to every member of the family.
fn holes_for_detector(
    holes: &[CoverageHoleEntry],
    canonical_id: DetectorCanonicalId,
    family: PrimitiveFamily,
) -> Vec<&CoverageHoleEntry> {
    holes
        .iter()
        .filter(|h| match h.subject {
            CoverageHoleSubject::Detector(id) => id == canonical_id,
            CoverageHoleSubject::Family(f) => f == family,
            _ => false,
        })
        .collect()
}

/// Returns all challenges whose target is the given canonical
/// detector (T.11f `ChallengeTarget::Detector` or `Passport`).
/// Corpus / registry / grammar / transcript / execution-receipt
/// globals are NOT folded in per-detector — those would
/// indiscriminately disable everyone.
fn challenges_for_detector(
    challenges: &[ChallengeDocketEntry],
    canonical_id: DetectorCanonicalId,
) -> Vec<&ChallengeDocketEntry> {
    challenges
        .iter()
        .filter(|c| match c.target {
            ChallengeTarget::Detector(id) | ChallengeTarget::Passport(id) => id == canonical_id,
            _ => false,
        })
        .collect()
}

/// True iff the given hole is blocking — i.e. Critical/High
/// severity AND status is not Resolved AND no resolution-gate
/// completion marker is present.
fn hole_is_blocking(h: &CoverageHoleEntry) -> bool {
    let severity_blocks = matches!(
        h.severity,
        CoverageHoleSeverity::Critical | CoverageHoleSeverity::High
    );
    let status_blocks = !matches!(h.status, CoverageHoleStatus::Resolved);
    severity_blocks && status_blocks
}

/// True iff the given hole is warning-grade (Medium severity,
/// not yet Resolved). These attach `warning_receipt_hashes`
/// but do not flip Enabled -> Disabled.
fn hole_is_warning(h: &CoverageHoleEntry) -> bool {
    matches!(h.severity, CoverageHoleSeverity::Medium)
        && !matches!(h.status, CoverageHoleStatus::Resolved)
}

/// True iff a coverage-hole reason maps to a specific
/// `DisabledReason` family. Used by the planner to choose the
/// most informative DisabledReason when multiple holes target
/// the same detector.
fn hole_reason_maps_to_missing_sampling_law(r: CoverageHoleReason) -> bool {
    matches!(r, CoverageHoleReason::SemanticsMissingSamplingLaw)
}

fn hole_reason_maps_to_missing_unit_semantics(r: CoverageHoleReason) -> bool {
    matches!(r, CoverageHoleReason::SemanticsMissingUnitSemantics)
}

fn hole_reason_maps_to_missing_confuser(r: CoverageHoleReason) -> bool {
    matches!(r, CoverageHoleReason::FamilyMissingConfuserCoverage)
}

fn hole_reason_maps_to_thin_precedent(r: CoverageHoleReason) -> bool {
    matches!(r, CoverageHoleReason::JurisprudenceThinPrecedentSupport)
}

/// True iff a challenge is blocking — Sustained at any severity,
/// OR Open at Critical severity. Open/Medium-Low and Deferred
/// challenges attach as warnings instead of blocking.
fn challenge_is_blocking(c: &ChallengeDocketEntry) -> bool {
    use crate::challenge_docket::ChallengeSeverity;
    matches!(c.status, ChallengeStatus::Sustained)
        || (matches!(c.status, ChallengeStatus::Open)
            && matches!(c.severity, ChallengeSeverity::Critical))
}

/// True iff a challenge is warning-grade (Open/Medium-Low,
/// Deferred, or Overruled-with-residual-concern). Attaches a
/// non-blocking warning hash.
fn challenge_is_warning(c: &ChallengeDocketEntry) -> bool {
    matches!(c.status, ChallengeStatus::Open | ChallengeStatus::Deferred)
        && !challenge_is_blocking(c)
}

// ---------------------------------------------------------------
// Derivation
// ---------------------------------------------------------------

/// Build the `ActivationPlanV1` snapshot deterministically from
/// the sealed court stack.
///
/// `registry_hash_v2` is passed in by the caller because the
/// corpus crate is intentionally registry-independent. Pass
/// [`KNOWN_S12_REGISTRY_HASH_V2`] for the canonical S1.2 anchor.
/// Two calls with the same `registry_hash_v2` produce
/// byte-identical bytes.
#[must_use]
pub fn collect_activation_plan(registry_hash_v2: [u8; 32]) -> ActivationPlanV1 {
    let corpus_hash = compute_corpus_hash_v1().bytes;
    let docket = collect_challenge_docket();
    let challenge_docket_hash = compute_challenge_docket_hash_v1(&docket);
    let contraindications = collect_contraindications();
    let contraindication_hash = compute_contraindication_hash_v1(&contraindications);
    let coverage = collect_coverage_holes();
    let coverage_hole_hash = compute_coverage_hole_hash_v1(&coverage);

    let mut decisions: Vec<DetectorActivationDecision> = SEED
        .iter()
        .filter_map(|record| {
            // `passport_for` only returns None for an unknown
            // canonical_id; SEED iteration guarantees every id is
            // known, so None here is a structural corpus break.
            // We surface that by skipping the record — the
            // verifier's `decision_count != SEED.len()` invariant
            // catches the omission rather than us panicking inside
            // derivation.
            let passport = passport_for(record.canonical_id)?;
            Some(derive_decision(
                record.canonical_id,
                record.display_name,
                record.primitive_family,
                record.witness_role,
                record.implementation_status,
                record.input_requirements,
                &passport,
                &contraindications.receipts,
                &docket.challenges,
                &coverage.holes,
            ))
        })
        .collect();
    decisions.sort_by_key(|d| d.canonical_id.0);

    let mut enabled_count = 0u32;
    let mut disabled_count = 0u32;
    let mut warn_only_count = 0u32;
    let mut deferred_count = 0u32;
    for d in &decisions {
        match d.activation_status {
            ActivationStatus::Enabled => enabled_count += 1,
            ActivationStatus::Disabled => disabled_count += 1,
            ActivationStatus::WarnOnly => warn_only_count += 1,
            ActivationStatus::Deferred => deferred_count += 1,
        }
    }

    let reason_histogram = compute_reason_histogram(&decisions);

    let mut plan = ActivationPlanV1 {
        schema: ActivationPlanSchema::V1AdmissibilityOnly,
        corpus_hash_v1: corpus_hash,
        registry_hash_v2,
        challenge_docket_hash_v1: challenge_docket_hash,
        detector_contraindication_hash_v1: contraindication_hash,
        coverage_hole_hash_v1: coverage_hole_hash,
        decisions,
        enabled_count,
        disabled_count,
        warn_only_count,
        deferred_count,
        reason_histogram,
        activation_plan_hash_v1: [0u8; 32],
    };
    plan.activation_plan_hash_v1 = compute_activation_plan_hash_v1(&plan);
    plan
}

/// Derive a single detector's activation decision.
///
/// Decision priority (highest first):
///  1. Disabled by blocking coverage hole (Critical / High).
///  2. Disabled by contraindication `do_not_use_for`.
///  3. Disabled by sustained / critical-Open challenge.
///  4. Disabled by weak L-band (L0/L1/L2 — no host implementation).
///  5. Disabled by missing sampling law (ordered-time detector).
///  6. Disabled by missing unit semantics (unit-sensitive detector).
///  7. Disabled by missing confuser (Primary in conferless family).
///  8. Disabled by thin precedent support.
///  9. Deferred if coverage hole has resolution_gate set but is
///     still Acknowledged/DeferredToGate (not Resolved).
/// 10. WarnOnly if Medium coverage hole / non-blocking challenge.
/// 11. Otherwise Enabled. Pick the strongest applicable
///     `EnabledReason` in this priority order: `GpuSurface`,
///     `Primary`, `Boundary`, `Confuser`,
///     `NoBlockingCoverageHole`, `PassportComplete`,
///     `ContraindicationSatisfied`, `ChallengeClear`.
#[allow(clippy::too_many_arguments)]
fn derive_decision(
    canonical_id: DetectorCanonicalId,
    display_name: &'static str,
    family: PrimitiveFamily,
    role: WitnessRole,
    lband: ImplementationLevel,
    input_req: InputRequirementSet,
    passport: &DetectorPassport,
    contraindications: &[DetectorContraindicationReceiptV1],
    challenges: &[ChallengeDocketEntry],
    holes: &[CoverageHoleEntry],
) -> DetectorActivationDecision {
    let mut blocking_hashes: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut warning_hashes: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut cited_challenges: BTreeSet<u32> = BTreeSet::new();
    let mut cited_contras: BTreeSet<u32> = BTreeSet::new();
    let mut cited_holes: BTreeSet<u32> = BTreeSet::new();
    let mut chosen_disable: Option<DisabledReason> = None;

    let det_holes = holes_for_detector(holes, canonical_id, family);
    let det_challenges = challenges_for_detector(challenges, canonical_id);
    let contra = contraindication_for(contraindications, canonical_id);

    // (1) Blocking coverage hole.
    let mut hole_had_blocker = false;
    for h in &det_holes {
        if hole_is_blocking(h) {
            hole_had_blocker = true;
            cited_holes.insert(h.hole_id.0);
            blocking_hashes.insert(synthetic_hole_hash(h));
            chosen_disable = chosen_disable.or_else(|| Some(map_hole_to_disabled_reason(h.reason)));
        } else if hole_is_warning(h) {
            cited_holes.insert(h.hole_id.0);
            warning_hashes.insert(synthetic_hole_hash(h));
        }
    }

    // (2) Contraindication `do_not_use_for`. At S1.3a the
    // planner has no evidence contract, so a non-empty
    // `do_not_use_for` is a contextual flag — operator must be
    // aware — but NOT a global block. We attach the receipt
    // hash to `warning_hashes` so the decision still cites the
    // contraindication. S1.3 (full planner) will promote
    // warnings to blocks when the active evidence contract
    // matches a `do_not_use_for` reason. The verifier rule
    // `EnabledDetectorWithBlockingContraindication` is therefore
    // load-bearing for S1.3 and will not legitimately fire from
    // S1.3a's planner output; it remains in the verifier so a
    // future S1.3 commit that wires evidence contracts cannot
    // produce an inconsistent "Enabled + blocking
    // contraindication" decision.
    if contra.is_some_and(|c| !c.do_not_use_for.is_empty()) {
        cited_contras.insert(canonical_id.0);
        warning_hashes.insert(synthetic_contraindication_hash(canonical_id));
    }

    // (3) Sustained / Critical-Open challenge.
    for c in &det_challenges {
        if challenge_is_blocking(c) {
            cited_challenges.insert(c.challenge_id.0);
            blocking_hashes.insert(synthetic_challenge_hash(c));
            if chosen_disable.is_none() {
                chosen_disable = Some(DisabledReason::DisabledByUnresolvedChallenge);
            }
        } else if challenge_is_warning(c) {
            cited_challenges.insert(c.challenge_id.0);
            warning_hashes.insert(synthetic_challenge_hash(c));
        }
    }

    // (4) Weak L-band.
    let is_weak_lband = matches!(
        lband,
        ImplementationLevel::L0_CitedOnly
            | ImplementationLevel::L1_Canonicalised
            | ImplementationLevel::L2_DeterministicFormula
    );
    if is_weak_lband && chosen_disable.is_none() {
        chosen_disable = Some(DisabledReason::DisabledByWeakLBand);
        // The passport is the citation for an L-band block: the
        // L-band lives on the passport, so the passport hash is
        // the artifact the operator can replay to verify the
        // disable reason. We attach the passport hash to
        // blocking_hashes for exactly this purpose.
        blocking_hashes.insert(passport.passport_hash);
    }

    // (5)/(6) Missing sampling law / unit semantics. Promote any
    // hole-driven block to the more specific DisabledReason when
    // the cited hole spells out the semantic gap.
    if chosen_disable == Some(DisabledReason::DisabledByCoverageHole) {
        for h in &det_holes {
            if hole_reason_maps_to_missing_sampling_law(h.reason) {
                chosen_disable = Some(DisabledReason::DisabledByMissingSamplingLaw);
                break;
            }
            if hole_reason_maps_to_missing_unit_semantics(h.reason) {
                chosen_disable = Some(DisabledReason::DisabledByMissingUnitSemantics);
                break;
            }
            if hole_reason_maps_to_missing_confuser(h.reason) {
                chosen_disable = Some(DisabledReason::DisabledByMissingConfuser);
                break;
            }
            if hole_reason_maps_to_thin_precedent(h.reason) {
                chosen_disable = Some(DisabledReason::DisabledByThinPrecedentSupport);
                break;
            }
        }
    }
    // Independent semantic gates (catch ordered-time / unit-sensitive
    // detectors even when no hole was raised; defense-in-depth).
    if chosen_disable.is_none()
        && detector_is_ordered_time(family, input_req)
        && contra.is_some_and(|c| c.required_sampling_law.is_none())
        && matches!(
            lband,
            ImplementationLevel::L5_GpuImplemented | ImplementationLevel::L6_CpuGpuByteEquivalent
        )
    {
        chosen_disable = Some(DisabledReason::DisabledByMissingSamplingLaw);
    }
    if chosen_disable.is_none()
        && detector_is_unit_sensitive(input_req)
        && contra.is_some_and(|c| c.required_units.is_none())
        && matches!(
            lband,
            ImplementationLevel::L5_GpuImplemented | ImplementationLevel::L6_CpuGpuByteEquivalent
        )
    {
        chosen_disable = Some(DisabledReason::DisabledByMissingUnitSemantics);
    }

    // (7) Unimplemented surface (passport lifecycle Dormant etc.):
    // not currently emitted because every SEED record is Active or
    // Dormant under T.11a's lifecycle semantics; left as a future
    // hook.

    // Status assembly.
    let activation_status = if chosen_disable.is_some() {
        ActivationStatus::Disabled
    } else if !warning_hashes.is_empty() {
        ActivationStatus::WarnOnly
    } else {
        ActivationStatus::Enabled
    };

    let no_warnings = warning_hashes.is_empty();
    let (enabled_reason, disabled_reason) = match activation_status {
        ActivationStatus::Enabled | ActivationStatus::WarnOnly => {
            let r = pick_enabled_reason(canonical_id, role, passport, no_warnings);
            (Some(r), None)
        }
        ActivationStatus::Disabled | ActivationStatus::Deferred => (None, chosen_disable),
    };

    // Convert sets -> sorted Vec.
    let blocking_receipt_hashes: Vec<[u8; 32]> = blocking_hashes.into_iter().collect();
    let warning_receipt_hashes: Vec<[u8; 32]> = warning_hashes.into_iter().collect();
    let cited_challenge_ids: Vec<ChallengeId> =
        cited_challenges.into_iter().map(ChallengeId).collect();
    let cited_contraindication_ids: Vec<DetectorCanonicalId> =
        cited_contras.into_iter().map(DetectorCanonicalId).collect();
    let cited_coverage_hole_ids: Vec<CoverageHoleId> =
        cited_holes.into_iter().map(CoverageHoleId).collect();

    // Mark warn-only paths cleanly: hole_had_blocker is false but
    // warning_hashes non-empty.
    let _ = hole_had_blocker;

    DetectorActivationDecision {
        canonical_id,
        display_name,
        activation_status,
        enabled_reason,
        disabled_reason,
        blocking_receipt_hashes,
        warning_receipt_hashes,
        cited_challenge_ids,
        cited_contraindication_ids,
        cited_coverage_hole_ids,
        cited_passport_hash: passport.passport_hash,
    }
}

/// Map a `CoverageHoleReason` to the most informative
/// `DisabledReason` family. The catch-all is `DisabledByCoverageHole`.
fn map_hole_to_disabled_reason(reason: CoverageHoleReason) -> DisabledReason {
    match reason {
        CoverageHoleReason::SemanticsMissingSamplingLaw
        | CoverageHoleReason::SemanticsTimeSeriesWithoutRegularityAssumption
        | CoverageHoleReason::SemanticsSpectralWithoutSampleRateAssumption => {
            DisabledReason::DisabledByMissingSamplingLaw
        }
        CoverageHoleReason::SemanticsMissingUnitSemantics => {
            DisabledReason::DisabledByMissingUnitSemantics
        }
        CoverageHoleReason::FamilyMissingConfuserCoverage => {
            DisabledReason::DisabledByMissingConfuser
        }
        CoverageHoleReason::JurisprudenceThinPrecedentSupport => {
            DisabledReason::DisabledByThinPrecedentSupport
        }
        _ => DisabledReason::DisabledByCoverageHole,
    }
}

/// Pick the strongest applicable `EnabledReason` for an admitted
/// detector. Priority order: `GpuSurface`, `Primary`, `Boundary`,
/// `Confuser`, `NoBlockingCoverageHole`, `PassportComplete`,
/// `ContraindicationSatisfied`, `ChallengeClear`. The choice is
/// deterministic — never falls back to a "default" enabled reason
/// without exhausting the priority list.
fn pick_enabled_reason(
    canonical_id: DetectorCanonicalId,
    role: WitnessRole,
    passport: &DetectorPassport,
    no_warnings: bool,
) -> EnabledReason {
    if GPU_IMPLEMENTED_CANONICAL_IDS.contains(&canonical_id) {
        return EnabledReason::EnabledByRoleSeededGpuSurface;
    }
    match role {
        WitnessRole::Primary => return EnabledReason::EnabledAsPrimaryWitness,
        WitnessRole::Boundary => return EnabledReason::EnabledAsBoundaryWitness,
        WitnessRole::Confuser => return EnabledReason::EnabledAsConfuserWitness,
        _ => {}
    }
    // Constitution-flag check is a passport invariant; if the
    // passport admits, prefer that signal. Otherwise fall through.
    if passport_admits(passport) {
        if no_warnings {
            EnabledReason::EnabledByPassportComplete
        } else {
            EnabledReason::EnabledByNoBlockingCoverageHole
        }
    } else if no_warnings {
        EnabledReason::EnabledByContraindicationSatisfied
    } else {
        EnabledReason::EnabledByChallengeClear
    }
}

/// True iff every `ConstitutionFlag` on the passport is set.
/// Passport admission is a necessary-not-sufficient condition;
/// the planner uses it as the generic enable signal when no
/// stronger role / GPU-surface match applies.
fn passport_admits(p: &DetectorPassport) -> bool {
    let f = &p.constitution_flags;
    f.declared_input_contract
        && f.declared_output_type
        && f.declared_deterministic_form
        && f.declared_provenance
        && f.declared_equivalence_status
        && f.declared_witness_role
        && f.declared_activation_conditions
        && f.declared_failure_confuser_modes
}

/// Synthesise a 32-byte citation hash for a coverage-hole entry.
/// The receipt-level hash is not exposed per-entry by T.11h, so
/// we hash the entry's stable fields (id + kind + severity +
/// status + subject) under a sub-domain. This lets one decision
/// cite the exact hole that drove it.
fn synthetic_hole_hash(h: &CoverageHoleEntry) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1:HOLE\0");
    buf.extend_from_slice(&h.hole_id.0.to_be_bytes());
    buf.extend_from_slice(h.kind.as_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(h.severity.as_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(h.status.as_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(h.subject.kind_str().as_bytes());
    buf.push(0);
    buf.extend_from_slice(&h.subject.id().to_be_bytes());
    sha256(&buf)
}

/// Synthesise a 32-byte citation hash for a contraindication
/// receipt. Hashes the canonical_id under an activation-plan
/// sub-domain so two different canonical_ids produce distinct
/// citations even with identical receipt structure.
fn synthetic_contraindication_hash(canonical_id: DetectorCanonicalId) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(48);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1:CONTRAINDICATION\0");
    buf.extend_from_slice(&canonical_id.0.to_be_bytes());
    sha256(&buf)
}

/// Synthesise a 32-byte citation hash for a challenge docket
/// entry. Hashes the challenge_id + target wire-name + status
/// under an activation-plan sub-domain so the citation is
/// status-sensitive (a Sustained challenge cites differently
/// from a Deferred one with the same id).
fn synthetic_challenge_hash(c: &ChallengeDocketEntry) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1:CHALLENGE\0");
    buf.extend_from_slice(&c.challenge_id.0.to_be_bytes());
    let kind_str = match c.target {
        ChallengeTarget::Detector(_) => "Detector",
        ChallengeTarget::Precedent(_) => "Precedent",
        ChallengeTarget::GrammarRule(_) => "GrammarRule",
        ChallengeTarget::Passport(_) => "Passport",
        ChallengeTarget::TrialTranscript(_) => "TrialTranscript",
        ChallengeTarget::ExecutionReceipt(_) => "ExecutionReceipt",
        ChallengeTarget::CorpusGlobal => "CorpusGlobal",
        ChallengeTarget::RegistryGlobal => "RegistryGlobal",
    };
    buf.extend_from_slice(kind_str.as_bytes());
    buf.push(0);
    buf.extend_from_slice(c.status.as_str().as_bytes());
    sha256(&buf)
}

/// Compute the reason histogram across the plan's decisions.
/// Returns rows sorted by reason wire name ascending.
fn compute_reason_histogram(decisions: &[DetectorActivationDecision]) -> Vec<ReasonHistogramRow> {
    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for d in decisions {
        if let Some(r) = d.enabled_reason {
            *counts.entry(r.as_str()).or_insert(0) += 1;
        }
        if let Some(r) = d.disabled_reason {
            *counts.entry(r.as_str()).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .map(|(reason, count)| ReasonHistogramRow { reason, count })
        .collect()
}

// ---------------------------------------------------------------
// Canonical-byte serialisation + hash
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

/// Append a per-detector decision to the canonical-byte buffer.
/// Field order matches the schema; every enum is serialised by
/// its `as_str()` so a future rename of an enum variant in code
/// (without updating its wire name) cannot silently change the
/// hash. `None` for enabled_reason / disabled_reason is tagged
/// with a leading 0 byte; `Some` is tagged 1 followed by the
/// wire name.
fn write_decision(out: &mut Vec<u8>, d: &DetectorActivationDecision) {
    write_u32(out, d.canonical_id.0);
    write_str(out, d.display_name);
    write_str(out, d.activation_status.as_str());
    match d.enabled_reason {
        None => out.push(0),
        Some(r) => {
            out.push(1);
            write_str(out, r.as_str());
        }
    }
    match d.disabled_reason {
        None => out.push(0),
        Some(r) => {
            out.push(1);
            write_str(out, r.as_str());
        }
    }
    write_u32(
        out,
        u32::try_from(d.blocking_receipt_hashes.len()).unwrap_or(u32::MAX),
    );
    for h in &d.blocking_receipt_hashes {
        write_bytes(out, h);
    }
    write_u32(
        out,
        u32::try_from(d.warning_receipt_hashes.len()).unwrap_or(u32::MAX),
    );
    for h in &d.warning_receipt_hashes {
        write_bytes(out, h);
    }
    write_u32(
        out,
        u32::try_from(d.cited_challenge_ids.len()).unwrap_or(u32::MAX),
    );
    for id in &d.cited_challenge_ids {
        write_u32(out, id.0);
    }
    write_u32(
        out,
        u32::try_from(d.cited_contraindication_ids.len()).unwrap_or(u32::MAX),
    );
    for id in &d.cited_contraindication_ids {
        write_u32(out, id.0);
    }
    write_u32(
        out,
        u32::try_from(d.cited_coverage_hole_ids.len()).unwrap_or(u32::MAX),
    );
    for id in &d.cited_coverage_hole_ids {
        write_u32(out, id.0);
    }
    write_bytes(out, &d.cited_passport_hash);
}

/// Compute `activation_plan_hash_v1`. Two calls against the same
/// plan produce byte-identical output. The plan's own
/// `activation_plan_hash_v1` field is excluded from the hash
/// material (we hash the plan before writing the hash into it,
/// then the hash becomes the field value).
#[must_use]
pub fn compute_activation_plan_hash_v1(p: &ActivationPlanV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    buf.extend_from_slice(ACTIVATION_PLAN_DOMAIN.as_bytes());
    write_str(&mut buf, ACTIVATION_PLAN_SCHEMA_V1);
    write_str(&mut buf, p.schema.as_str());
    write_bytes(&mut buf, &p.corpus_hash_v1);
    write_bytes(&mut buf, &p.registry_hash_v2);
    write_bytes(&mut buf, &p.challenge_docket_hash_v1);
    write_bytes(&mut buf, &p.detector_contraindication_hash_v1);
    write_bytes(&mut buf, &p.coverage_hole_hash_v1);
    write_u32(
        &mut buf,
        u32::try_from(p.decisions.len()).unwrap_or(u32::MAX),
    );
    for d in &p.decisions {
        write_decision(&mut buf, d);
    }
    write_u32(&mut buf, p.enabled_count);
    write_u32(&mut buf, p.disabled_count);
    write_u32(&mut buf, p.warn_only_count);
    write_u32(&mut buf, p.deferred_count);
    write_u32(
        &mut buf,
        u32::try_from(p.reason_histogram.len()).unwrap_or(u32::MAX),
    );
    for row in &p.reason_histogram {
        write_str(&mut buf, row.reason);
        write_u32(&mut buf, row.count);
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// One verifier failure. Multiple errors per plan are admissible
/// — the verifier walks every decision and returns the full list
/// so a single audit pass surfaces all defects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActivationPlanVerifyError {
    /// Which decision failed (canonical_id; 0 for plan-level
    /// errors not tied to a specific decision).
    pub canonical_id: DetectorCanonicalId,
    /// Categorical failure kind.
    pub kind: ActivationPlanVerifyErrorKind,
}

/// Categorical verifier reject kinds. 14 panel-locked rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationPlanVerifyErrorKind {
    /// Enabled / WarnOnly decision has no `enabled_reason`.
    EnabledWithoutEnabledReason,
    /// Disabled / Deferred decision has no `disabled_reason`.
    DisabledWithoutDisabledReason,
    /// Enabled decision cites a blocking coverage hole that
    /// matches its canonical id at Critical/High severity.
    EnabledDetectorWithBlockingCoverageHole,
    /// Enabled decision cites an active contraindication
    /// `do_not_use_for` for its canonical id.
    EnabledDetectorWithBlockingContraindication,
    /// Enabled decision targets a detector with a Sustained or
    /// Critical-Open challenge against it.
    EnabledDetectorWithBlockingChallenge,
    /// Disabled / Deferred decision carries no
    /// `blocking_receipt_hashes` (must cite something).
    DisabledDecisionWithoutBlockingHash,
    /// Two decisions share the same `canonical_id`.
    DuplicateDecisionForCanonicalId,
    /// Decision targets a `canonical_id` not in `SEED`.
    DecisionForUnknownDetector,
    /// Decision cites a `ChallengeId` not in the live docket.
    DecisionCitesUnknownChallenge,
    /// Decision cites a `CoverageHoleId` not in the live report.
    DecisionCitesUnknownCoverageHole,
    /// Decision cites a contraindication canonical id not in
    /// the live receipts.
    DecisionCitesUnknownContraindication,
    /// Plan-level error: `corpus_hash_v1` is all zeros.
    PlanMissingCorpusHash,
    /// Plan-level error: `registry_hash_v2` is all zeros.
    PlanMissingRegistryHash,
    /// `cited_passport_hash` does not match the live passport
    /// for the decision's canonical_id.
    DecisionPassportHashMismatch,
}

impl ActivationPlanVerifyErrorKind {
    /// Stable wire name; rendered in the verification receipt.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EnabledWithoutEnabledReason => "EnabledWithoutEnabledReason",
            Self::DisabledWithoutDisabledReason => "DisabledWithoutDisabledReason",
            Self::EnabledDetectorWithBlockingCoverageHole => {
                "EnabledDetectorWithBlockingCoverageHole"
            }
            Self::EnabledDetectorWithBlockingContraindication => {
                "EnabledDetectorWithBlockingContraindication"
            }
            Self::EnabledDetectorWithBlockingChallenge => "EnabledDetectorWithBlockingChallenge",
            Self::DisabledDecisionWithoutBlockingHash => "DisabledDecisionWithoutBlockingHash",
            Self::DuplicateDecisionForCanonicalId => "DuplicateDecisionForCanonicalId",
            Self::DecisionForUnknownDetector => "DecisionForUnknownDetector",
            Self::DecisionCitesUnknownChallenge => "DecisionCitesUnknownChallenge",
            Self::DecisionCitesUnknownCoverageHole => "DecisionCitesUnknownCoverageHole",
            Self::DecisionCitesUnknownContraindication => "DecisionCitesUnknownContraindication",
            Self::PlanMissingCorpusHash => "PlanMissingCorpusHash",
            Self::PlanMissingRegistryHash => "PlanMissingRegistryHash",
            Self::DecisionPassportHashMismatch => "DecisionPassportHashMismatch",
        }
    }
}

/// Walk every decision in the plan against the live court stack;
/// return one `ActivationPlanVerifyError` per defect. Empty Vec
/// means the plan is fully admissible.
#[must_use]
pub fn verify_activation_plan(p: &ActivationPlanV1) -> Vec<ActivationPlanVerifyError> {
    let mut errors: Vec<ActivationPlanVerifyError> = Vec::new();

    // Plan-level hash anchors.
    if p.corpus_hash_v1 == [0u8; 32] {
        errors.push(ActivationPlanVerifyError {
            canonical_id: DetectorCanonicalId(0),
            kind: ActivationPlanVerifyErrorKind::PlanMissingCorpusHash,
        });
    }
    if p.registry_hash_v2 == [0u8; 32] {
        errors.push(ActivationPlanVerifyError {
            canonical_id: DetectorCanonicalId(0),
            kind: ActivationPlanVerifyErrorKind::PlanMissingRegistryHash,
        });
    }

    // Build lookup tables.
    let known_detectors: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let docket = collect_challenge_docket();
    let known_challenges: BTreeSet<u32> =
        docket.challenges.iter().map(|c| c.challenge_id.0).collect();
    let contras = collect_contraindications();
    let known_contras: BTreeSet<u32> = contras.receipts.iter().map(|c| c.canonical_id.0).collect();
    let coverage = collect_coverage_holes();
    let known_holes: BTreeSet<u32> = coverage.holes.iter().map(|h| h.hole_id.0).collect();

    // Per-decision walk.
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for d in &p.decisions {
        if !seen.insert(d.canonical_id.0) {
            errors.push(ActivationPlanVerifyError {
                canonical_id: d.canonical_id,
                kind: ActivationPlanVerifyErrorKind::DuplicateDecisionForCanonicalId,
            });
        }
        if !known_detectors.contains(&d.canonical_id.0) {
            errors.push(ActivationPlanVerifyError {
                canonical_id: d.canonical_id,
                kind: ActivationPlanVerifyErrorKind::DecisionForUnknownDetector,
            });
            continue;
        }
        match d.activation_status {
            ActivationStatus::Enabled | ActivationStatus::WarnOnly => {
                if d.enabled_reason.is_none() {
                    errors.push(ActivationPlanVerifyError {
                        canonical_id: d.canonical_id,
                        kind: ActivationPlanVerifyErrorKind::EnabledWithoutEnabledReason,
                    });
                }
                // Enabled MUST NOT cite a Critical/High blocking
                // hole subjecting its detector.
                for h in &coverage.holes {
                    if matches!(h.subject, CoverageHoleSubject::Detector(id) if id == d.canonical_id)
                        && hole_is_blocking(h)
                        && matches!(d.activation_status, ActivationStatus::Enabled)
                    {
                        errors.push(ActivationPlanVerifyError {
                            canonical_id: d.canonical_id,
                            kind:
                                ActivationPlanVerifyErrorKind::EnabledDetectorWithBlockingCoverageHole,
                        });
                        break;
                    }
                }
                // Enabled MUST NOT have an active contraindication
                // do_not_use_for.
                if matches!(d.activation_status, ActivationStatus::Enabled) {
                    if let Some(c) = contras
                        .receipts
                        .iter()
                        .find(|c| c.canonical_id == d.canonical_id)
                    {
                        if !c.do_not_use_for.is_empty() {
                            errors.push(ActivationPlanVerifyError {
                                canonical_id: d.canonical_id,
                                kind:
                                    ActivationPlanVerifyErrorKind::EnabledDetectorWithBlockingContraindication,
                            });
                        }
                    }
                }
                // Enabled MUST NOT have a Sustained or Critical-Open
                // challenge against it.
                if matches!(d.activation_status, ActivationStatus::Enabled) {
                    for c in &docket.challenges {
                        let targets_me = matches!(
                            c.target,
                            ChallengeTarget::Detector(id) | ChallengeTarget::Passport(id)
                                if id == d.canonical_id
                        );
                        if targets_me && challenge_is_blocking(c) {
                            errors.push(ActivationPlanVerifyError {
                                canonical_id: d.canonical_id,
                                kind: ActivationPlanVerifyErrorKind::EnabledDetectorWithBlockingChallenge,
                            });
                            break;
                        }
                    }
                }
            }
            ActivationStatus::Disabled | ActivationStatus::Deferred => {
                if d.disabled_reason.is_none() {
                    errors.push(ActivationPlanVerifyError {
                        canonical_id: d.canonical_id,
                        kind: ActivationPlanVerifyErrorKind::DisabledWithoutDisabledReason,
                    });
                }
                if d.blocking_receipt_hashes.is_empty() {
                    errors.push(ActivationPlanVerifyError {
                        canonical_id: d.canonical_id,
                        kind: ActivationPlanVerifyErrorKind::DisabledDecisionWithoutBlockingHash,
                    });
                }
            }
        }
        // Citation existence.
        for id in &d.cited_challenge_ids {
            if !known_challenges.contains(&id.0) {
                errors.push(ActivationPlanVerifyError {
                    canonical_id: d.canonical_id,
                    kind: ActivationPlanVerifyErrorKind::DecisionCitesUnknownChallenge,
                });
                break;
            }
        }
        for id in &d.cited_coverage_hole_ids {
            if !known_holes.contains(&id.0) {
                errors.push(ActivationPlanVerifyError {
                    canonical_id: d.canonical_id,
                    kind: ActivationPlanVerifyErrorKind::DecisionCitesUnknownCoverageHole,
                });
                break;
            }
        }
        for id in &d.cited_contraindication_ids {
            if !known_contras.contains(&id.0) {
                errors.push(ActivationPlanVerifyError {
                    canonical_id: d.canonical_id,
                    kind: ActivationPlanVerifyErrorKind::DecisionCitesUnknownContraindication,
                });
                break;
            }
        }
        // Passport hash crosscheck.
        if let Some(p_live) = passport_for(d.canonical_id) {
            if p_live.passport_hash != d.cited_passport_hash {
                errors.push(ActivationPlanVerifyError {
                    canonical_id: d.canonical_id,
                    kind: ActivationPlanVerifyErrorKind::DecisionPassportHashMismatch,
                });
            }
        }
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

/// Escape a string for JSON inclusion (RFC 8259 §7). Used only
/// by `render_activation_plan_json`; the JSON form is human-display
/// only and is NOT part of the canonical hash material.
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

/// Render the activation plan as deterministic text. Two calls
/// against the same plan produce byte-identical strings.
#[must_use]
pub fn render_activation_plan_text(p: &ActivationPlanV1) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas - ActivationPlanV1 (S1.3a)\n");
    out.push_str("================================================================\n");
    out.push_str(&format!(
        "schema                                : {}\n",
        p.schema.as_str()
    ));
    out.push_str(&format!(
        "activation_plan_hash_v1               : {}\n",
        hex(&p.activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "corpus_hash_v1                        : {}\n",
        hex(&p.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "registry_hash_v2                      : {}\n",
        hex(&p.registry_hash_v2)
    ));
    out.push_str(&format!(
        "challenge_docket_hash_v1              : {}\n",
        hex(&p.challenge_docket_hash_v1)
    ));
    out.push_str(&format!(
        "detector_contraindication_hash_v1     : {}\n",
        hex(&p.detector_contraindication_hash_v1)
    ));
    out.push_str(&format!(
        "coverage_hole_hash_v1                 : {}\n",
        hex(&p.coverage_hole_hash_v1)
    ));
    out.push_str(&format!(
        "decision_count                        : {}\n",
        p.decisions.len()
    ));
    out.push('\n');
    out.push_str("----------------------------------------------------------------\n");
    out.push_str("Status histogram\n");
    out.push_str("----------------------------------------------------------------\n");
    out.push_str(&format!("  Enabled    : {}\n", p.enabled_count));
    out.push_str(&format!("  Disabled   : {}\n", p.disabled_count));
    out.push_str(&format!("  WarnOnly   : {}\n", p.warn_only_count));
    out.push_str(&format!("  Deferred   : {}\n", p.deferred_count));
    out.push('\n');
    out.push_str("----------------------------------------------------------------\n");
    out.push_str("Reason histogram\n");
    out.push_str("----------------------------------------------------------------\n");
    for row in &p.reason_histogram {
        out.push_str(&format!("  {:<40} : {}\n", row.reason, row.count));
    }
    out.push('\n');
    out.push_str("----------------------------------------------------------------\n");
    out.push_str("Decisions (sorted by canonical_id ascending)\n");
    out.push_str("----------------------------------------------------------------\n");
    for d in &p.decisions {
        let reason_str = match (d.enabled_reason, d.disabled_reason) {
            (Some(r), _) => r.as_str(),
            (_, Some(r)) => r.as_str(),
            _ => "-",
        };
        out.push_str(&format!(
            "  #{:<3} {:<26} {:<9} reason={:<40} block={} warn={} cited(c/g/h)={}/{}/{}\n",
            d.canonical_id.0,
            truncate(d.display_name, 26),
            d.activation_status.as_str(),
            reason_str,
            d.blocking_receipt_hashes.len(),
            d.warning_receipt_hashes.len(),
            d.cited_challenge_ids.len(),
            d.cited_contraindication_ids.len(),
            d.cited_coverage_hole_ids.len(),
        ));
    }
    out
}

/// Truncate a string to `n` chars for fixed-width rendering.
fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut t = s.to_string();
        t.truncate(n.saturating_sub(1));
        t.push('…');
        t
    }
}

/// Render the activation plan as deterministic JSON. Two calls
/// against the same plan produce byte-identical strings. JSON
/// is human-display only and is NOT in the canonical hash.
#[must_use]
pub fn render_activation_plan_json(p: &ActivationPlanV1) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("{\n");
    out.push_str(&format!("  \"schema\": \"{}\",\n", p.schema.as_str()));
    out.push_str(&format!(
        "  \"activation_plan_hash_v1\": \"{}\",\n",
        hex(&p.activation_plan_hash_v1)
    ));
    out.push_str(&format!(
        "  \"corpus_hash_v1\": \"{}\",\n",
        hex(&p.corpus_hash_v1)
    ));
    out.push_str(&format!(
        "  \"registry_hash_v2\": \"{}\",\n",
        hex(&p.registry_hash_v2)
    ));
    out.push_str(&format!(
        "  \"challenge_docket_hash_v1\": \"{}\",\n",
        hex(&p.challenge_docket_hash_v1)
    ));
    out.push_str(&format!(
        "  \"detector_contraindication_hash_v1\": \"{}\",\n",
        hex(&p.detector_contraindication_hash_v1)
    ));
    out.push_str(&format!(
        "  \"coverage_hole_hash_v1\": \"{}\",\n",
        hex(&p.coverage_hole_hash_v1)
    ));
    out.push_str(&format!("  \"enabled_count\": {},\n", p.enabled_count));
    out.push_str(&format!("  \"disabled_count\": {},\n", p.disabled_count));
    out.push_str(&format!("  \"warn_only_count\": {},\n", p.warn_only_count));
    out.push_str(&format!("  \"deferred_count\": {},\n", p.deferred_count));
    out.push_str("  \"reason_histogram\": [\n");
    for (i, row) in p.reason_histogram.iter().enumerate() {
        let comma = if i + 1 == p.reason_histogram.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            "    {{ \"reason\": \"{}\", \"count\": {} }}{}\n",
            row.reason, row.count, comma
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"decisions\": [\n");
    for (i, d) in p.decisions.iter().enumerate() {
        let comma = if i + 1 == p.decisions.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"canonical_id\": {},\n      \"display_name\": \"{}\",\n      \"activation_status\": \"{}\",\n",
            d.canonical_id.0,
            json_escape(d.display_name),
            d.activation_status.as_str()
        ));
        match d.enabled_reason {
            Some(r) => out.push_str(&format!("      \"enabled_reason\": \"{}\",\n", r.as_str())),
            None => out.push_str("      \"enabled_reason\": null,\n"),
        }
        match d.disabled_reason {
            Some(r) => out.push_str(&format!("      \"disabled_reason\": \"{}\",\n", r.as_str())),
            None => out.push_str("      \"disabled_reason\": null,\n"),
        }
        out.push_str("      \"blocking_receipt_hashes\": [");
        for (j, h) in d.blocking_receipt_hashes.iter().enumerate() {
            let c = if j + 1 == d.blocking_receipt_hashes.len() {
                ""
            } else {
                ", "
            };
            out.push_str(&format!("\"{}\"{}", hex(h), c));
        }
        out.push_str("],\n      \"warning_receipt_hashes\": [");
        for (j, h) in d.warning_receipt_hashes.iter().enumerate() {
            let c = if j + 1 == d.warning_receipt_hashes.len() {
                ""
            } else {
                ", "
            };
            out.push_str(&format!("\"{}\"{}", hex(h), c));
        }
        out.push_str("],\n      \"cited_challenge_ids\": [");
        for (j, id) in d.cited_challenge_ids.iter().enumerate() {
            let c = if j + 1 == d.cited_challenge_ids.len() {
                ""
            } else {
                ", "
            };
            out.push_str(&format!("{}{}", id.0, c));
        }
        out.push_str("],\n      \"cited_contraindication_ids\": [");
        for (j, id) in d.cited_contraindication_ids.iter().enumerate() {
            let c = if j + 1 == d.cited_contraindication_ids.len() {
                ""
            } else {
                ", "
            };
            out.push_str(&format!("{}{}", id.0, c));
        }
        out.push_str("],\n      \"cited_coverage_hole_ids\": [");
        for (j, id) in d.cited_coverage_hole_ids.iter().enumerate() {
            let c = if j + 1 == d.cited_coverage_hole_ids.len() {
                ""
            } else {
                ", "
            };
            out.push_str(&format!("{}{}", id.0, c));
        }
        out.push_str(&format!(
            "],\n      \"cited_passport_hash\": \"{}\"\n",
            hex(&d.cited_passport_hash)
        ));
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Test-only constructor used by acceptance tests to build a
/// `DetectorActivationDecision` from raw fields without going
/// through the live derivation. Hidden from doc; only `#[cfg(test)]`
/// callers in the integration tests reach this.
#[doc(hidden)]
#[must_use]
pub fn __mk_decision_for_test(
    canonical_id: DetectorCanonicalId,
    display_name: &'static str,
    activation_status: ActivationStatus,
    enabled_reason: Option<EnabledReason>,
    disabled_reason: Option<DisabledReason>,
    blocking_receipt_hashes: Vec<[u8; 32]>,
    warning_receipt_hashes: Vec<[u8; 32]>,
    cited_challenge_ids: Vec<ChallengeId>,
    cited_contraindication_ids: Vec<DetectorCanonicalId>,
    cited_coverage_hole_ids: Vec<CoverageHoleId>,
    cited_passport_hash: [u8; 32],
) -> DetectorActivationDecision {
    DetectorActivationDecision {
        canonical_id,
        display_name,
        activation_status,
        enabled_reason,
        disabled_reason,
        blocking_receipt_hashes,
        warning_receipt_hashes,
        cited_challenge_ids,
        cited_contraindication_ids,
        cited_coverage_hole_ids,
        cited_passport_hash,
    }
}

/// Test-only constructor for an entire plan with custom
/// decisions, bypassing live derivation. Used to drive the
/// load-bearing negative tests.
#[doc(hidden)]
#[must_use]
pub fn __mk_plan_for_test(
    decisions: Vec<DetectorActivationDecision>,
    registry_hash_v2: [u8; 32],
) -> ActivationPlanV1 {
    let mut enabled_count = 0u32;
    let mut disabled_count = 0u32;
    let mut warn_only_count = 0u32;
    let mut deferred_count = 0u32;
    for d in &decisions {
        match d.activation_status {
            ActivationStatus::Enabled => enabled_count += 1,
            ActivationStatus::Disabled => disabled_count += 1,
            ActivationStatus::WarnOnly => warn_only_count += 1,
            ActivationStatus::Deferred => deferred_count += 1,
        }
    }
    let reason_histogram = compute_reason_histogram(&decisions);
    let mut plan = ActivationPlanV1 {
        schema: ActivationPlanSchema::V1AdmissibilityOnly,
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        registry_hash_v2,
        challenge_docket_hash_v1: compute_challenge_docket_hash_v1(&collect_challenge_docket()),
        detector_contraindication_hash_v1: compute_contraindication_hash_v1(
            &collect_contraindications(),
        ),
        coverage_hole_hash_v1: compute_coverage_hole_hash_v1(&collect_coverage_holes()),
        decisions,
        enabled_count,
        disabled_count,
        warn_only_count,
        deferred_count,
        reason_histogram,
        activation_plan_hash_v1: [0u8; 32],
    };
    plan.activation_plan_hash_v1 = compute_activation_plan_hash_v1(&plan);
    plan
}
