//! FF.2 — `ActivationReason::DisabledUnratifiedProposal`: the
//! ratification gate that teaches the activation court to refuse
//! any detector proposal lacking `corpus_hash_v2` ratification +
//! FF.1 passport authority.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **FF.2 makes activation refuse any detector proposal that is
//! > not ratified by `corpus_hash_v2` and materialized through
//! > FF.1 passport authority. Core rule: no ratification + no
//! > passport = no activation. FF.2 adds
//! > `DisabledReason::DisabledUnratifiedProposal` so the
//! > activation court can explicitly disable detector proposals
//! > that are not present in the ratified `corpus_hash_v2`
//! > expansion index and not materialized in the FF.1 passport
//! > index. It does not add new detectors, alter
//! > `corpus_hash_v2`, rewrite FF.1 passports, or change prior
//! > activation decisions except by making the unratified-
//! > proposal failure mode explicit and reason-coded.**
//!
//! ## Method
//!
//! 1. Pull the live consolidation report from [`crate::consolidate`]
//!    to obtain the authoritative ratified expansion index (the
//!    98 ratified `CanonicalAddition` ids spanning the 5001..=6699
//!    reserved bands).
//! 2. Pull the live FF.1 passport index from
//!    [`crate::ff1_passport_materialisation`]. Every ratified
//!    canonical id MUST appear in the passport index; FF.1's
//!    invariants already pin this property, so a passport-index
//!    miss for a ratified id surfaces a structural defect.
//! 3. Walk a candidate canonical-id set and classify each id
//!    into exactly one of four mutually-exclusive
//!    [`Ff2RatificationStatus`] buckets:
//!    - `SeedHistorical` — id ∈ SEED (1..=54).
//!    - `T12RatifiedAndPassported` — id ∈ ratified expansion
//!      index AND id ∈ FF.1 passport index.
//!    - `MissingPassport` — id ∈ ratified expansion index but
//!      NOT in FF.1 passport index. Structural defect; should
//!      never occur in production because FF.1 materialises one
//!      passport per ratified entry. Reserved so the test surface
//!      can exercise the rejection rule explicitly.
//!    - `UnratifiedProposal` — id outside both SEED and the
//!      ratified expansion index. The new failure mode FF.2
//!      surfaces explicitly: an operator-facing reason code that
//!      replaces the silent `DisabledByWeakLBand` fallback the
//!      pre-FF.2 activation planner would have emitted.
//! 4. Emit one [`Ff2GateDecision`] per candidate id, sorted by
//!    `canonical_id` ascending. Decisions in the `SeedHistorical`
//!    / `T12RatifiedAndPassported` buckets carry an empty
//!    disabled-reason wire name (they pass the gate — downstream
//!    activation planning handles them). Decisions in the
//!    `MissingPassport` / `UnratifiedProposal` buckets carry
//!    `disabled_reason_wire_name = "DisabledUnratifiedProposal"`.
//! 5. Aggregate the decisions into the top-level
//!    [`Ff2ActivationRatificationGate`] with per-status counts and
//!    the four pinned anchor hashes (`corpus_hash_v1`,
//!    `corpus_hash_v2`, `ff1_passport_index_hash_v1`,
//!    `consolidation_report_hash_v1`) proving FF.2 did not mutate
//!    any upstream authority. Hash under
//!    `DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0`.
//! 6. Wrap the gate in [`Ff2ActivationRatificationGateSummary`]
//!    which carries the panel-locked non-claim block hashed under
//!    a distinct domain so the summary artifact is independently
//!    addressable.
//!
//! ## Panel-locked non-claims
//!
//! - FF.2 does NOT add new detectors.
//! - FF.2 does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
//!   `consolidation_report_hash_v1`, `t12_expansion_index_hash_v1`,
//!   or `ff1_passport_index_hash_v1`.
//! - FF.2 does NOT rewrite any historical T.11 / S1.3 / T.12.x /
//!   FF.1 hash.
//! - FF.2 does NOT mutate `SEED.len()` (stays at 54).
//! - FF.2 does NOT promote any open proposal to Accepted.
//! - FF.2 does NOT change prior S1.3a activation decisions for
//!   SEED ids: those continue to flow through
//!   [`crate::activation`]. FF.2 layers ABOVE S1.3a as a
//!   ratification-discipline gate that surfaces the unratified-
//!   proposal failure mode explicitly.
//! - FF.2 does NOT generate CUDA kernels.
//! - FF.2 does NOT decide contraindications or challenges.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! - `corpus_hash_v1`: byte-identical (`35c276c7…`).
//! - `corpus_hash_v2`: byte-identical (`f1d132eb…`).
//! - `consolidation_report_hash_v1`: byte-identical (`2842f6ae…`).
//! - `t12_expansion_index_hash_v1`: byte-identical (`11fe6543…`).
//! - `ff1_passport_index_hash_v1`: byte-identical (`1ad2dc2d…`).
//! - `ff1_materialisation_report_hash_v1`: byte-identical
//!   (`5edacbc4…`).
//! - Every prior T.11 / S1.3 / T.12.x hash: byte-identical.
//! - `SEED.len()`: 54 (unchanged).
//! - **NEW**: `ff2_activation_ratification_gate_hash_v1` (one
//!   value over the sorted decision set + pinned anchors) and
//!   `ff2_activation_ratification_gate_summary_hash_v1` (one
//!   value over the gate + panel-locked non-claim block).
//!
//! ## Panel-locked warning the verifier enforces
//!
//! > Do not let unratified proposals collapse into generic
//! > `DisabledByWeakLBand`. That would erase the court
//! > distinction. FF.2 exists so the operator can see: this
//! > detector is disabled because it is not ratified / not
//! > passported.
//!
//! The verifier's `SilentFallbackToDisabledByWeakLBand` rule
//! rejects any FF.2 gate decision whose non-ratified status
//! carries `DisabledByWeakLBand` (or any non-FF.2 wire name) as
//! its disabled reason. The only acceptable wire name for a
//! non-ratified decision is `DisabledUnratifiedProposal`.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior court-
//! surface module: every `pub` item AND every private helper
//! carries a doc comment whose first sentence states the WHY
//! for a future engineer. 10-step ritual; no `--no-verify`.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::activation::DisabledReason;
use crate::consolidate::{build_consolidation_report, ConsolidationReport};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::ff1_passport_materialisation::{
    build_ff1_passport_index, build_ff1_passport_index_from, Ff1PassportIndex,
};
use crate::seed::SEED;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators (NEW own-namespace hashes)
// ---------------------------------------------------------------

/// Domain separator for `ff2_activation_ratification_gate_hash_v1`.
/// Distinct from FF.1's passport / index / report domains so the
/// FF.2 gate artifact is independently addressable.
pub const FF2_ACTIVATION_GATE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1\0";

/// Schema identifier embedded in the gate hash material.
pub const FF2_ACTIVATION_GATE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE:v1";

/// Domain separator for
/// `ff2_activation_ratification_gate_summary_hash_v1`. Distinct
/// from the gate domain so the summary artifact (gate + non-claim
/// block) is independently addressable.
pub const FF2_ACTIVATION_GATE_SUMMARY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1\0";

/// Schema identifier embedded in the gate-summary hash material.
pub const FF2_ACTIVATION_GATE_SUMMARY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF2-ACTIVATION-RATIFICATION-GATE-SUMMARY:v1";

/// Wire name FF.2 emits in every non-ratified decision's
/// `disabled_reason_wire_name` field. Mirrored verbatim from the
/// new [`DisabledReason::DisabledUnratifiedProposal`] enum
/// variant's `as_str()`. Pinned here so the FF.2 module is
/// self-contained for the verifier's
/// `SilentFallbackToDisabledByWeakLBand` rule.
pub const DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME: &str = "DisabledUnratifiedProposal";

/// Wire name the pre-FF.2 activation planner would have emitted
/// for a non-ratified id (the silent fallback the panel warning
/// rejects). Used by the verifier's
/// `SilentFallbackToDisabledByWeakLBand` rule as the explicit
/// forbidden value.
pub const DISABLED_BY_WEAK_L_BAND_WIRE_NAME: &str = "DisabledByWeakLBand";

// ---------------------------------------------------------------
// Ratification status
// ---------------------------------------------------------------

/// The four mutually-exclusive ratification-status buckets every
/// candidate canonical id maps into. Stored as an enum so the
/// classifier is a single deterministic match; the wire name is
/// derived via [`Ff2RatificationStatus::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ff2RatificationStatus {
    /// Canonical id ∈ SEED (1..=54). The historical seed corpus;
    /// passes the FF.2 gate. Downstream S1.3a activation
    /// continues to issue the per-detector decision.
    SeedHistorical,
    /// Canonical id ∈ ratified expansion index AND ∈ FF.1
    /// passport index. The post-T.12.consolidate ratified
    /// surface; passes the FF.2 gate. Downstream activation
    /// continues with a passport binding.
    T12RatifiedAndPassported,
    /// Canonical id ∈ ratified expansion index but NOT ∈ FF.1
    /// passport index. Structural defect; should never occur in
    /// production because FF.1 materialises one passport per
    /// ratified entry. The bucket is reserved so the verifier's
    /// `ActivationForMissingFf1Passport` rule has an explicit
    /// status to surface during test-driven exercise of the
    /// rejection path.
    MissingPassport,
    /// Canonical id outside SEED AND outside the ratified
    /// expansion index. The new failure mode FF.2 surfaces
    /// explicitly: an operator-facing reason code replacing the
    /// silent `DisabledByWeakLBand` fallback the pre-FF.2 planner
    /// would have emitted.
    UnratifiedProposal,
}

impl Ff2RatificationStatus {
    /// Stable wire name; used in the canonical hash material and
    /// in the FF.2 gate decision's `status_wire_name` field.
    /// Deterministic and sortable; pinned for byte-stable
    /// hashing.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SeedHistorical => "SeedHistorical",
            Self::T12RatifiedAndPassported => "T12RatifiedAndPassported",
            Self::MissingPassport => "MissingPassport",
            Self::UnratifiedProposal => "UnratifiedProposal",
        }
    }

    /// True iff this status passes the FF.2 ratification gate
    /// (i.e. downstream S1.3a activation should consider the id).
    /// `SeedHistorical` and `T12RatifiedAndPassported` pass;
    /// `MissingPassport` and `UnratifiedProposal` fail.
    #[must_use]
    pub const fn passes_gate(self) -> bool {
        matches!(self, Self::SeedHistorical | Self::T12RatifiedAndPassported)
    }
}

// ---------------------------------------------------------------
// Gate decision
// ---------------------------------------------------------------

/// One per-candidate decision emitted by the FF.2 gate. Field
/// order is the canonical hash order; do not reorder without
/// rebaselining `ff2_activation_ratification_gate_hash_v1`.
#[derive(Debug, Clone)]
pub struct Ff2GateDecision {
    /// Candidate canonical id this decision concerns.
    pub canonical_id: u32,
    /// Ratification-status bucket.
    pub status: Ff2RatificationStatus,
    /// Stable wire name of `status` (mirror of
    /// [`Ff2RatificationStatus::as_str`]; pinned here so the hash
    /// material is field-deterministic without a status-enum
    /// match at hash time).
    pub status_wire_name: &'static str,
    /// Disabled-reason wire name. Empty string for decisions
    /// that pass the gate (`SeedHistorical`,
    /// `T12RatifiedAndPassported`). Non-empty
    /// `"DisabledUnratifiedProposal"` for decisions that fail
    /// the gate (`MissingPassport`, `UnratifiedProposal`); this
    /// is exactly the new [`DisabledReason::DisabledUnratifiedProposal`]
    /// variant's wire name.
    pub disabled_reason_wire_name: &'static str,
    /// 32-byte FF.1 passport hash for this id; zero bytes when
    /// no passport exists (status ≠ `T12RatifiedAndPassported`).
    pub cited_passport_hash: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level FF.2 gate
// ---------------------------------------------------------------

/// The FF.2 activation ratification gate. Carries the sorted
/// list of per-candidate decisions + the four pinned anchor
/// hashes proving FF.2 did not mutate any upstream authority.
/// Two builds against the same proposal set + same candidate id
/// list produce byte-identical bytes.
#[derive(Debug, Clone)]
pub struct Ff2ActivationRatificationGate {
    /// Historical seed-corpus anchor (pinned; verified equal to
    /// `compute_corpus_hash_v1()` at build time).
    pub corpus_hash_v1: [u8; 32],
    /// Ratified-corpus authority anchor (pinned; verified equal
    /// to the live consolidation report's `corpus_hash_v2`).
    pub corpus_hash_v2: [u8; 32],
    /// T.12.consolidate consolidation-report hash (pinned).
    pub consolidation_report_hash_v1: [u8; 32],
    /// FF.1 passport-index hash (pinned; verified equal to the
    /// live passport index hash at build time).
    pub ff1_passport_index_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Per-candidate decisions, sorted by `canonical_id`
    /// ascending.
    pub decisions: Vec<Ff2GateDecision>,
    /// Count of `SeedHistorical` decisions.
    pub seed_historical_count: u32,
    /// Count of `T12RatifiedAndPassported` decisions.
    pub t12_ratified_and_passported_count: u32,
    /// Count of `MissingPassport` decisions (structural defect;
    /// should be 0 in production).
    pub missing_passport_count: u32,
    /// Count of `UnratifiedProposal` decisions.
    pub unratified_proposal_count: u32,
    /// `ff2_activation_ratification_gate_hash_v1` — domain-
    /// separated SHA-256 over every field above.
    pub ff2_activation_ratification_gate_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Gate summary (gate + non-claim block)
// ---------------------------------------------------------------

/// The FF.2 gate summary: gate + panel-locked non-claim block
/// hashed under a distinct domain so the summary artifact is
/// independently addressable. The non-claim block is a fixed
/// string array; mutations to the string text require a new
/// domain separator (schema-upgrade commit).
#[derive(Debug, Clone)]
pub struct Ff2ActivationRatificationGateSummary {
    /// The wrapped gate.
    pub gate: Ff2ActivationRatificationGate,
    /// Panel-locked non-claim text lines. Pinned verbatim; the
    /// summary hash mixes the line count + bytes so any
    /// silent rewrite changes the summary hash.
    pub non_claim_lines: &'static [&'static str],
    /// `ff2_activation_ratification_gate_summary_hash_v1` —
    /// domain-separated SHA-256 over the gate hash + non-claim
    /// lines.
    pub ff2_activation_ratification_gate_summary_hash_v1: [u8; 32],
}

/// Panel-locked non-claim text lines emitted by every FF.2 gate
/// summary. Pinned verbatim so the summary hash binds them.
pub const FF2_NON_CLAIM_LINES: &[&str] = &[
    "FF.2 does NOT add new detectors.",
    "FF.2 does NOT alter corpus_hash_v1.",
    "FF.2 does NOT alter corpus_hash_v2.",
    "FF.2 does NOT alter consolidation_report_hash_v1.",
    "FF.2 does NOT alter t12_expansion_index_hash_v1.",
    "FF.2 does NOT alter ff1_passport_index_hash_v1.",
    "FF.2 does NOT alter ff1_materialisation_report_hash_v1.",
    "FF.2 does NOT rewrite any prior T.11 / S1.3 / T.12.x / FF.1 hash.",
    "FF.2 does NOT mutate SEED.len() (stays at 54).",
    "FF.2 does NOT promote any open proposal to Accepted.",
    "FF.2 does NOT change S1.3a SEED activation decisions.",
    "FF.2 does NOT generate CUDA kernels.",
    "FF.2 does NOT decide contraindications or challenges.",
];

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why FF.2 rejected an input. An empty `verify_ff2` return
/// means the gate is admissible. The six panel-required negatives
/// map onto these variants verbatim; additional structural rules
/// (sort order, duplicate id, anchor cross-check) are emitted
/// under their own kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ff2VerifyErrorKind {
    /// Panel-required negative #1: a candidate canonical id
    /// outside SEED and outside the ratified expansion index
    /// must be classified as `UnratifiedProposal`. Surfaced if
    /// the gate emits a different status for such an id.
    ActivationForUnratifiedProposal {
        /// The non-ratified canonical id.
        canonical_id: u32,
        /// The status the gate emitted (should have been
        /// `UnratifiedProposal`).
        observed_status_wire_name: &'static str,
    },
    /// Panel-required negative #2: a candidate canonical id in
    /// the ratified expansion index but missing from the FF.1
    /// passport index must be classified as `MissingPassport`.
    /// Surfaced if the gate emits a different status (e.g.
    /// claims `T12RatifiedAndPassported` for an id absent from
    /// the passport index).
    ActivationForMissingFf1Passport {
        /// The ratified canonical id missing a passport.
        canonical_id: u32,
    },
    /// Panel-required negative #3: the gate's pinned
    /// `ff1_passport_index_hash_v1` does not equal the live FF.1
    /// passport index hash.
    PassportIndexHashMismatch {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live FF.1 passport index computes.
        actual: [u8; 32],
    },
    /// Panel-required negative #4: a decision whose status is
    /// `UnratifiedProposal` or `MissingPassport` MUST carry
    /// `disabled_reason_wire_name = "DisabledUnratifiedProposal"`.
    /// Surfaced if the wire name is empty or missing.
    UnratifiedProposalWithoutReasonCode {
        /// The canonical id with the missing reason code.
        canonical_id: u32,
    },
    /// Panel-required negative #5: a decision whose status is
    /// `UnratifiedProposal` or `MissingPassport` MUST NOT carry
    /// `DisabledByWeakLBand` (or any non-FF.2 wire name) as its
    /// disabled reason. The panel warning is explicit:
    /// "Do not let unratified proposals collapse into generic
    /// DisabledByWeakLBand. That would erase the court
    /// distinction."
    SilentFallbackToDisabledByWeakLBand {
        /// The canonical id whose decision silently fell back.
        canonical_id: u32,
        /// The wire name actually observed (should have been
        /// `"DisabledUnratifiedProposal"`).
        observed_reason_wire_name: &'static str,
    },
    /// Panel-required negative #6: the gate's
    /// `corpus_hash_v2` field MUST be the live ratified-corpus
    /// authority anchor (non-zero, matching the live
    /// consolidation report). Surfaced if the field is the
    /// all-zero sentinel or does not match.
    ActivationReasonWithoutCorpusHashV2Binding {
        /// The `corpus_hash_v2` value observed on the gate.
        observed_corpus_hash_v2: [u8; 32],
    },
    /// Two decisions share the same canonical id.
    DuplicateGateDecisionForSameCanonicalId {
        /// The duplicated canonical id.
        canonical_id: u32,
    },
    /// Decisions are not sorted ascending by `canonical_id` (the
    /// gate's byte-stable hash requires canonical ordering).
    GateDecisionsNotSortedAscending,
    /// `corpus_hash_v1` pinned on the gate does not equal the
    /// live `compute_corpus_hash_v1()` result.
    CorpusHashV1Mismatch {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live `compute_corpus_hash_v1()` returns.
        actual: [u8; 32],
    },
    /// `consolidation_report_hash_v1` pinned on the gate does
    /// not equal the live consolidation report's hash.
    ConsolidationReportHashV1Mismatch {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live consolidation report computes.
        actual: [u8; 32],
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff2VerifyError {
    /// Error kind (see [`Ff2VerifyErrorKind`]).
    pub kind: Ff2VerifyErrorKind,
}

// ---------------------------------------------------------------
// Default candidate-id derivation
// ---------------------------------------------------------------

/// Build the production default candidate canonical-id list:
/// every SEED canonical id (1..=54) plus every FF.1 passport
/// canonical id (5001..=6699). Sorted ascending. Used by the
/// production [`build_ff2_activation_ratification_gate`] call;
/// tests can supply alternate lists to exercise the rejection
/// rules.
#[must_use]
pub fn default_candidate_ids(passport_index: &Ff1PassportIndex) -> Vec<u32> {
    let mut ids: Vec<u32> = Vec::with_capacity(SEED.len() + passport_index.passports.len());
    for r in SEED {
        ids.push(r.canonical_id.0);
    }
    for p in &passport_index.passports {
        ids.push(p.canonical_id);
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

// ---------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------

/// Classify a single candidate canonical id into its
/// ratification-status bucket. Pure derivation; never mutates
/// anything. The four buckets are mutually exclusive: SEED
/// membership takes precedence (an id in 1..=54 always reports
/// `SeedHistorical` even if a future commit accidentally
/// registers it in the expansion index — the SEED is frozen and
/// the verifier's `DuplicateReservedId` rule in
/// [`crate::consolidate`] guards the precondition).
fn classify(
    canonical_id: u32,
    seed_ids: &BTreeSet<u32>,
    ratified_ids: &BTreeSet<u32>,
    passport_ids: &BTreeSet<u32>,
) -> Ff2RatificationStatus {
    if seed_ids.contains(&canonical_id) {
        return Ff2RatificationStatus::SeedHistorical;
    }
    if ratified_ids.contains(&canonical_id) {
        if passport_ids.contains(&canonical_id) {
            return Ff2RatificationStatus::T12RatifiedAndPassported;
        }
        return Ff2RatificationStatus::MissingPassport;
    }
    Ff2RatificationStatus::UnratifiedProposal
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production FF.2 gate from the live consolidation
/// report + live FF.1 passport index. Two builds produce byte-
/// identical bytes. Used by the production CLI / artifact-
/// emission paths; the test suite uses
/// [`build_ff2_activation_ratification_gate_from`] with synthetic
/// id lists to exercise the rejection rules.
#[must_use]
pub fn build_ff2_activation_ratification_gate() -> Ff2ActivationRatificationGate {
    let report = build_consolidation_report();
    let index = build_ff1_passport_index_from(&report);
    let candidates = default_candidate_ids(&index);
    build_ff2_activation_ratification_gate_from(&report, &index, &candidates)
}

/// Build the FF.2 gate from a specific consolidation report +
/// passport index + candidate id list. Pure function; used by
/// tests to inject ids outside the ratified surface (exercising
/// `UnratifiedProposal`) and to inject ratified-without-passport
/// ids (exercising `MissingPassport`).
#[must_use]
pub fn build_ff2_activation_ratification_gate_from(
    report: &ConsolidationReport,
    passport_index: &Ff1PassportIndex,
    candidate_ids: &[u32],
) -> Ff2ActivationRatificationGate {
    let seed_ids: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let ratified_ids: BTreeSet<u32> = report
        .expansion_index
        .iter()
        .map(|e| e.canonical_id)
        .collect();
    let passport_ids: BTreeSet<u32> = passport_index
        .passports
        .iter()
        .map(|p| p.canonical_id)
        .collect();

    let mut decisions: Vec<Ff2GateDecision> = candidate_ids
        .iter()
        .copied()
        .map(|cid| {
            let status = classify(cid, &seed_ids, &ratified_ids, &passport_ids);
            let disabled_reason_wire_name: &'static str = if status.passes_gate() {
                ""
            } else {
                DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME
            };
            let cited_passport_hash: [u8; 32] =
                if status == Ff2RatificationStatus::T12RatifiedAndPassported {
                    passport_index
                        .passports
                        .iter()
                        .find(|p| p.canonical_id == cid)
                        .map_or([0u8; 32], |p| p.passport_hash_v1)
                } else {
                    [0u8; 32]
                };
            Ff2GateDecision {
                canonical_id: cid,
                status,
                status_wire_name: status.as_str(),
                disabled_reason_wire_name,
                cited_passport_hash,
            }
        })
        .collect();
    decisions.sort_by_key(|d| d.canonical_id);

    let mut seed_historical_count: u32 = 0;
    let mut t12_ratified_and_passported_count: u32 = 0;
    let mut missing_passport_count: u32 = 0;
    let mut unratified_proposal_count: u32 = 0;
    for d in &decisions {
        match d.status {
            Ff2RatificationStatus::SeedHistorical => seed_historical_count += 1,
            Ff2RatificationStatus::T12RatifiedAndPassported => {
                t12_ratified_and_passported_count += 1;
            }
            Ff2RatificationStatus::MissingPassport => missing_passport_count += 1,
            Ff2RatificationStatus::UnratifiedProposal => unratified_proposal_count += 1,
        }
    }

    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut gate = Ff2ActivationRatificationGate {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        consolidation_report_hash_v1: report.consolidation_report_hash_v1,
        ff1_passport_index_hash_v1: passport_index.ff1_passport_index_hash_v1,
        seed_len,
        decisions,
        seed_historical_count,
        t12_ratified_and_passported_count,
        missing_passport_count,
        unratified_proposal_count,
        ff2_activation_ratification_gate_hash_v1: [0u8; 32],
    };
    gate.ff2_activation_ratification_gate_hash_v1 = compute_ff2_activation_gate_hash(&gate);
    gate
}

/// Build the FF.2 gate summary from the live consolidation
/// report + live FF.1 passport index. Wraps the gate with the
/// panel-locked non-claim block; the summary hash binds the
/// non-claim lines so any silent rewrite surfaces.
#[must_use]
pub fn build_ff2_activation_ratification_gate_summary() -> Ff2ActivationRatificationGateSummary {
    let gate = build_ff2_activation_ratification_gate();
    build_ff2_activation_ratification_gate_summary_from_gate(gate)
}

/// Build the FF.2 gate summary from a specific gate. Used by
/// tests to wrap synthetic gates with the canonical non-claim
/// block.
#[must_use]
pub fn build_ff2_activation_ratification_gate_summary_from_gate(
    gate: Ff2ActivationRatificationGate,
) -> Ff2ActivationRatificationGateSummary {
    let mut summary = Ff2ActivationRatificationGateSummary {
        gate,
        non_claim_lines: FF2_NON_CLAIM_LINES,
        ff2_activation_ratification_gate_summary_hash_v1: [0u8; 32],
    };
    summary.ff2_activation_ratification_gate_summary_hash_v1 =
        compute_ff2_activation_gate_summary_hash(&summary);
    summary
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_bytes_fixed(out: &mut Vec<u8>, bytes: &[u8; 32]) {
    out.extend_from_slice(bytes);
}

fn compute_ff2_activation_gate_hash(gate: &Ff2ActivationRatificationGate) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(FF2_ACTIVATION_GATE_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF2_ACTIVATION_GATE_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &gate.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &gate.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &gate.consolidation_report_hash_v1);
    write_bytes_fixed(&mut buf, &gate.ff1_passport_index_hash_v1);
    write_u32(&mut buf, gate.seed_len);
    write_u32(
        &mut buf,
        u32::try_from(gate.decisions.len()).unwrap_or(u32::MAX),
    );
    for d in &gate.decisions {
        write_u32(&mut buf, d.canonical_id);
        write_str(&mut buf, d.status_wire_name);
        write_str(&mut buf, d.disabled_reason_wire_name);
        write_bytes_fixed(&mut buf, &d.cited_passport_hash);
    }
    write_u32(&mut buf, gate.seed_historical_count);
    write_u32(&mut buf, gate.t12_ratified_and_passported_count);
    write_u32(&mut buf, gate.missing_passport_count);
    write_u32(&mut buf, gate.unratified_proposal_count);
    sha256(&buf)
}

fn compute_ff2_activation_gate_summary_hash(
    summary: &Ff2ActivationRatificationGateSummary,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(FF2_ACTIVATION_GATE_SUMMARY_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF2_ACTIVATION_GATE_SUMMARY_SCHEMA_V1);
    write_bytes_fixed(
        &mut buf,
        &summary.gate.ff2_activation_ratification_gate_hash_v1,
    );
    write_u32(
        &mut buf,
        u32::try_from(summary.non_claim_lines.len()).unwrap_or(u32::MAX),
    );
    for line in summary.non_claim_lines {
        write_str(&mut buf, line);
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier — panel-required rules
// ---------------------------------------------------------------

/// Walk an FF.2 gate against the live consolidation report +
/// live FF.1 passport index and emit every rejection. An empty
/// return means the gate is admissible. The six panel-required
/// negatives map onto rules R.1–R.6; structural rules R.7–R.10
/// guard sort order, duplicates, anchor cross-check, and SEED
/// invariance.
#[must_use]
pub fn verify_ff2(
    gate: &Ff2ActivationRatificationGate,
    report: &ConsolidationReport,
    passport_index: &Ff1PassportIndex,
) -> Vec<Ff2VerifyError> {
    let mut errors: Vec<Ff2VerifyError> = Vec::new();

    let seed_ids: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let ratified_ids: BTreeSet<u32> = report
        .expansion_index
        .iter()
        .map(|e| e.canonical_id)
        .collect();
    let passport_ids: BTreeSet<u32> = passport_index
        .passports
        .iter()
        .map(|p| p.canonical_id)
        .collect();

    // R.1 ActivationForUnratifiedProposal: a candidate id
    // outside SEED and outside the ratified expansion index
    // MUST be classified as `UnratifiedProposal`.
    for d in &gate.decisions {
        let in_seed = seed_ids.contains(&d.canonical_id);
        let in_ratified = ratified_ids.contains(&d.canonical_id);
        if !in_seed && !in_ratified && d.status != Ff2RatificationStatus::UnratifiedProposal {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::ActivationForUnratifiedProposal {
                    canonical_id: d.canonical_id,
                    observed_status_wire_name: d.status_wire_name,
                },
            });
        }
    }

    // R.2 ActivationForMissingFf1Passport: a candidate id in
    // the ratified expansion index but NOT in the passport
    // index MUST be classified as `MissingPassport`. If the
    // decision instead claims `T12RatifiedAndPassported`, that
    // is a structural defect.
    for d in &gate.decisions {
        let in_ratified = ratified_ids.contains(&d.canonical_id);
        let in_passport = passport_ids.contains(&d.canonical_id);
        if in_ratified && !in_passport && d.status != Ff2RatificationStatus::MissingPassport {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::ActivationForMissingFf1Passport {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.3 PassportIndexHashMismatch: pinned anchor must match
    // the live FF.1 passport index hash.
    if gate.ff1_passport_index_hash_v1 != passport_index.ff1_passport_index_hash_v1 {
        errors.push(Ff2VerifyError {
            kind: Ff2VerifyErrorKind::PassportIndexHashMismatch {
                claimed: gate.ff1_passport_index_hash_v1,
                actual: passport_index.ff1_passport_index_hash_v1,
            },
        });
    }

    // R.4 UnratifiedProposalWithoutReasonCode: every decision
    // whose status is non-ratified MUST carry
    // `disabled_reason_wire_name = "DisabledUnratifiedProposal"`.
    for d in &gate.decisions {
        let non_ratified = matches!(
            d.status,
            Ff2RatificationStatus::UnratifiedProposal | Ff2RatificationStatus::MissingPassport
        );
        if non_ratified && d.disabled_reason_wire_name.is_empty() {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::UnratifiedProposalWithoutReasonCode {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.5 SilentFallbackToDisabledByWeakLBand: non-ratified
    // decisions MUST NOT carry `DisabledByWeakLBand` (or any
    // wire name other than `DisabledUnratifiedProposal`).
    for d in &gate.decisions {
        let non_ratified = matches!(
            d.status,
            Ff2RatificationStatus::UnratifiedProposal | Ff2RatificationStatus::MissingPassport
        );
        if non_ratified
            && !d.disabled_reason_wire_name.is_empty()
            && d.disabled_reason_wire_name != DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME
        {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::SilentFallbackToDisabledByWeakLBand {
                    canonical_id: d.canonical_id,
                    observed_reason_wire_name: d.disabled_reason_wire_name,
                },
            });
        }
    }

    // R.6 ActivationReasonWithoutCorpusHashV2Binding: the gate
    // MUST carry a non-zero `corpus_hash_v2` AND that hash MUST
    // equal the live consolidation report's `corpus_hash_v2`.
    if gate.corpus_hash_v2.iter().all(|b| *b == 0) || gate.corpus_hash_v2 != report.corpus_hash_v2 {
        errors.push(Ff2VerifyError {
            kind: Ff2VerifyErrorKind::ActivationReasonWithoutCorpusHashV2Binding {
                observed_corpus_hash_v2: gate.corpus_hash_v2,
            },
        });
    }

    // R.7 DuplicateGateDecisionForSameCanonicalId.
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for d in &gate.decisions {
        if !seen.insert(d.canonical_id) {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::DuplicateGateDecisionForSameCanonicalId {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.8 GateDecisionsNotSortedAscending.
    for w in gate.decisions.windows(2) {
        if w[0].canonical_id > w[1].canonical_id {
            errors.push(Ff2VerifyError {
                kind: Ff2VerifyErrorKind::GateDecisionsNotSortedAscending,
            });
            break;
        }
    }

    // R.9 anchor cross-checks.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if gate.corpus_hash_v1 != live_v1 {
        errors.push(Ff2VerifyError {
            kind: Ff2VerifyErrorKind::CorpusHashV1Mismatch {
                claimed: gate.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    if gate.consolidation_report_hash_v1 != report.consolidation_report_hash_v1 {
        errors.push(Ff2VerifyError {
            kind: Ff2VerifyErrorKind::ConsolidationReportHashV1Mismatch {
                claimed: gate.consolidation_report_hash_v1,
                actual: report.consolidation_report_hash_v1,
            },
        });
    }

    // R.10 SEED invariance.
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(Ff2VerifyError {
            kind: Ff2VerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Discipline assertion: enum wire-name parity
// ---------------------------------------------------------------

/// Cross-check at module-load time that the new
/// [`DisabledReason::DisabledUnratifiedProposal`] variant's wire
/// name matches the [`DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME`]
/// constant the FF.2 module relies on. Compile-time-evaluated
/// const assertion; surfaces a structural defect if a future
/// commit renames either side without updating the other.
const _: () = {
    let activation_wire = DisabledReason::DisabledUnratifiedProposal.as_str();
    let ff2_wire = DISABLED_UNRATIFIED_PROPOSAL_WIRE_NAME;
    let n = activation_wire.len();
    let m = ff2_wire.len();
    assert!(n == m, "FF.2 wire-name mismatch: lengths differ");
    let a = activation_wire.as_bytes();
    let b = ff2_wire.as_bytes();
    let mut i = 0;
    while i < n {
        assert!(a[i] == b[i], "FF.2 wire-name mismatch: byte differs");
        i += 1;
    }
};

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the FF.2 gate as a deterministic text report. Two
/// renders against the same gate produce byte-identical bytes.
/// Used by the `ff2-gate` CLI subcommand and the
/// `ff2-gate-emit` artifact writer.
#[must_use]
pub fn render_ff2_gate_text(gate: &Ff2ActivationRatificationGate) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    let _ = writeln!(s, "FF.2 Activation Ratification Gate (v1)");
    let _ = writeln!(s, "=======================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                  : {}",
        hex32(&gate.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                  : {}",
        hex32(&gate.corpus_hash_v2)
    );
    let _ = writeln!(
        s,
        "  consolidation_report_hash_v1    : {}",
        hex32(&gate.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff1_passport_index_hash_v1      : {}",
        hex32(&gate.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(s, "  SEED.len()                      : {}", gate.seed_len);
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-status counts");
    let _ = writeln!(
        s,
        "  SeedHistorical                  : {}",
        gate.seed_historical_count
    );
    let _ = writeln!(
        s,
        "  T12RatifiedAndPassported        : {}",
        gate.t12_ratified_and_passported_count
    );
    let _ = writeln!(
        s,
        "  MissingPassport                 : {}",
        gate.missing_passport_count
    );
    let _ = writeln!(
        s,
        "  UnratifiedProposal              : {}",
        gate.unratified_proposal_count
    );
    let _ = writeln!(
        s,
        "  total decisions                 : {}",
        gate.decisions.len()
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "ff2_activation_ratification_gate_hash_v1 : {}",
        hex32(&gate.ff2_activation_ratification_gate_hash_v1)
    );
    s
}

/// Render the FF.2 gate as a deterministic JSON object. Two
/// renders against the same gate produce byte-identical bytes
/// (sorted keys, fixed schema, hex-encoded 32-byte hashes).
#[must_use]
pub fn render_ff2_gate_json(gate: &Ff2ActivationRatificationGate) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    s.push('{');
    let _ = write!(s, "\"schema\":\"{FF2_ACTIVATION_GATE_SCHEMA_V1}\"");
    let _ = write!(s, ",\"corpus_hash_v1\":\"{}\"", hex32(&gate.corpus_hash_v1));
    let _ = write!(s, ",\"corpus_hash_v2\":\"{}\"", hex32(&gate.corpus_hash_v2));
    let _ = write!(
        s,
        ",\"consolidation_report_hash_v1\":\"{}\"",
        hex32(&gate.consolidation_report_hash_v1)
    );
    let _ = write!(
        s,
        ",\"ff1_passport_index_hash_v1\":\"{}\"",
        hex32(&gate.ff1_passport_index_hash_v1)
    );
    let _ = write!(s, ",\"seed_len\":{}", gate.seed_len);
    let _ = write!(
        s,
        ",\"seed_historical_count\":{}",
        gate.seed_historical_count
    );
    let _ = write!(
        s,
        ",\"t12_ratified_and_passported_count\":{}",
        gate.t12_ratified_and_passported_count
    );
    let _ = write!(
        s,
        ",\"missing_passport_count\":{}",
        gate.missing_passport_count
    );
    let _ = write!(
        s,
        ",\"unratified_proposal_count\":{}",
        gate.unratified_proposal_count
    );
    let _ = write!(s, ",\"decisions\":[");
    for (i, d) in gate.decisions.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"canonical_id\":{},\"status\":\"{}\",\"disabled_reason\":\"{}\",\"cited_passport_hash\":\"{}\"}}",
            d.canonical_id,
            d.status_wire_name,
            d.disabled_reason_wire_name,
            hex32(&d.cited_passport_hash)
        );
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"ff2_activation_ratification_gate_hash_v1\":\"{}\"",
        hex32(&gate.ff2_activation_ratification_gate_hash_v1)
    );
    s.push('}');
    s
}

/// Render the FF.2 gate summary as a deterministic text report.
#[must_use]
pub fn render_ff2_gate_summary_text(
    summary: &Ff2ActivationRatificationGateSummary,
) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = render_ff2_gate_text(&summary.gate);
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked non-claims");
    for line in summary.non_claim_lines {
        let _ = writeln!(s, "  - {line}");
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "ff2_activation_ratification_gate_summary_hash_v1 : {}",
        hex32(&summary.ff2_activation_ratification_gate_summary_hash_v1)
    );
    s
}

/// Render the FF.2 gate summary as a deterministic JSON object.
#[must_use]
pub fn render_ff2_gate_summary_json(
    summary: &Ff2ActivationRatificationGateSummary,
) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    s.push('{');
    let _ = write!(s, "\"schema\":\"{FF2_ACTIVATION_GATE_SUMMARY_SCHEMA_V1}\"");
    let _ = write!(s, ",\"gate\":{}", render_ff2_gate_json(&summary.gate));
    let _ = write!(s, ",\"non_claim_lines\":[");
    for (i, line) in summary.non_claim_lines.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "\"{line}\"");
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"ff2_activation_ratification_gate_summary_hash_v1\":\"{}\"",
        hex32(&summary.ff2_activation_ratification_gate_summary_hash_v1)
    );
    s.push('}');
    s
}

/// Hex-encode a 32-byte digest as a 64-character lowercase
/// string. Used by both renderers.
fn hex32(bytes: &[u8; 32]) -> alloc::string::String {
    let mut s = alloc::string::String::with_capacity(64);
    for b in bytes {
        s.push(nibble(*b >> 4));
        s.push(nibble(*b & 0x0f));
    }
    s
}

const fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + (n - 10)) as char,
        _ => '?',
    }
}

// Make `build_ff1_passport_index` reachable from a default-builder
// caller that doesn't already have a consolidation report in hand.
// This is a re-export-of-convenience; FF.2's other public API
// already exposes the canonical builders.
#[allow(dead_code)]
fn _ensure_default_passport_builder_is_reachable() -> Ff1PassportIndex {
    build_ff1_passport_index()
}
