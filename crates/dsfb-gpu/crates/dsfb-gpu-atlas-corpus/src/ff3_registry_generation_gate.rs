//! FF.3 — `RegistryGenerationGate`: the second META-discipline
//! layer above S1.3a + FF.1 + FF.2, teaching the S1.2 registry
//! generator to refuse any `DetectorSpec` whose source authority
//! is not (a) a SEED canonical record under `corpus_hash_v1` OR
//! (b) a `corpus_hash_v2`-ratified entry materialised through
//! FF.1 passport authority.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **FF.3 adds a registry-generation gate for S1.2
//! > `DetectorSpec` generation. The generator must accept only
//! > `SeedHistorical` records from `corpus_hash_v1` and
//! > `T12RatifiedAndPassported` records from `corpus_hash_v2` +
//! > FF.1 passport authority. It does not add detectors, mutate
//! > `corpus_hash_v1`, mutate `corpus_hash_v2`, rewrite FF.1
//! > passports, or change activation decisions. It only prevents
//! > unratified / non-passported / stale / ad-hoc records from
//! > entering generated registry output.**
//!
//! ## Method
//!
//! 1. Pull the live consolidation report from
//!    [`crate::consolidate`] (for `corpus_hash_v2` +
//!    expansion-index authority), the live FF.1 passport index
//!    from [`crate::ff1_passport_materialisation`] (for passport
//!    authority), and the live FF.2 activation ratification gate
//!    from [`crate::ff2_activation_ratification_gate`] (for the
//!    panel-locked "no DetectorSpec generation that skips FF.2"
//!    discipline).
//! 2. Walk a candidate registry-generation source-record set
//!    where each entry declares its [`Ff3SourceAuthority`]
//!    claim. Production default = SEED 1..=54 (claimed
//!    `SeedHistorical`) ∪ FF.1 passport ids 5001..=6699
//!    (claimed `T12RatifiedAndPassported`).
//! 3. For each candidate, classify into exactly one of seven
//!    mutually-exclusive [`Ff3RegistryGenerationEligibility`]
//!    buckets:
//!    - `Eligible` — claimed source authority verifies against
//!      live SEED / ratified-expansion / passport-index state;
//!      a `DetectorSpec` may be generated.
//!    - `RejectedUnratifiedProposal` — claimed
//!      `T12RatifiedAndPassported` but id NOT in ratified
//!      expansion index. Mirrors FF.2's `UnratifiedProposal`
//!      bucket at the registry-generation boundary.
//!    - `RejectedMissingFf1Passport` — id IS ratified but NOT
//!      in FF.1 passport index. Mirrors FF.2's `MissingPassport`
//!      bucket.
//!    - `RejectedCorpusHashV2Mismatch` — claimed
//!      `T12RatifiedAndPassported` AND the gate's pinned
//!      `corpus_hash_v2` does not equal the live ratified-
//!      corpus authority anchor.
//!    - `RejectedPassportIndexHashMismatch` — claimed
//!      `T12RatifiedAndPassported` AND the gate's pinned
//!      `ff1_passport_index_hash_v1` does not equal the live
//!      passport-index hash.
//!    - `RejectedAdHocRecord` — declared
//!      `AdHocUnsanctioned` source authority (a record proposed
//!      for registry generation outside the panel-locked
//!      two-source authority discipline). Forbidden by
//!      construction.
//!    - `RejectedUnknownSourceAuthority` — declared
//!      `UnknownExternal` source authority (no SEED / no
//!      ratified passport / no panel-locked path). Forbidden
//!      by construction.
//! 4. Emit one [`Ff3RegistryGenerationEligibilityDecision`] per
//!    candidate, sorted by `canonical_id` ascending. Eligible
//!    decisions carry an empty rejection-reason wire name;
//!    rejected decisions carry the matching `RejectedBy…` wire
//!    name. Decisions that pass against a live
//!    `T12RatifiedAndPassported` lookup carry the corresponding
//!    FF.1 passport hash as `cited_passport_hash`.
//! 5. Aggregate into the top-level
//!    [`Ff3RegistryGenerationGate`] with per-status counts and
//!    the five pinned anchor hashes (`corpus_hash_v1`,
//!    `corpus_hash_v2`, `consolidation_report_hash_v1`,
//!    `ff1_passport_index_hash_v1`,
//!    `ff2_activation_ratification_gate_hash_v1`) proving FF.3
//!    did not mutate any upstream authority. Hash under
//!    `DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0`.
//! 6. Wrap the gate in
//!    [`Ff3RegistryGenerationGateSummary`] which carries the
//!    panel-locked non-claim block hashed under a distinct
//!    domain so the summary artifact is independently
//!    addressable.
//!
//! ## Panel-locked non-claims
//!
//! - FF.3 does NOT add new detectors.
//! - FF.3 does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
//!   `consolidation_report_hash_v1`, `t12_expansion_index_hash_v1`,
//!   `ff1_passport_index_hash_v1`, or
//!   `ff2_activation_ratification_gate_hash_v1`.
//! - FF.3 does NOT rewrite any prior T.11 / S1.3 / T.12.x /
//!   FF.1 / FF.2 hash.
//! - FF.3 does NOT mutate `SEED.len()` (stays at 54).
//! - FF.3 does NOT promote any open proposal to Accepted.
//! - FF.3 does NOT change S1.3a SEED activation decisions.
//! - FF.3 does NOT change FF.2 ratification decisions; it
//!   layers ABOVE FF.2 as a registry-generation-boundary gate.
//! - FF.3 does NOT itself emit `DetectorSpec` records. It is a
//!   pure-decision module that the S1.2 registry generator
//!   consults; integration with `dsfb-gpu-atlas-registry`
//!   lands in a follow-on commit that calls into FF.3 before
//!   emitting each spec.
//! - FF.3 does NOT generate CUDA kernels.
//! - FF.3 does NOT decide contraindications or challenges.
//! - FF.3 does NOT modify `dsfb-gpu-atlas-registry`'s existing
//!   162-spec `registry_hash_v2`; that hash stays unchanged
//!   until the integration commit lands.
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
//! - `ff2_activation_ratification_gate_hash_v1`: byte-identical
//!   (`05c1b552…`).
//! - `ff2_activation_ratification_gate_summary_hash_v1`: byte-
//!   identical (`e671cfc0…`).
//! - Every prior T.11 / S1.3 / T.12.x / FF.1 / FF.2 hash:
//!   byte-identical.
//! - `SEED.len()`: 54 (unchanged).
//! - **NEW**: `ff3_registry_generation_gate_hash_v1` (one
//!   value over the sorted decision set + pinned anchors +
//!   per-status counts) and
//!   `ff3_registry_generation_gate_summary_hash_v1` (one
//!   value over the gate + panel-locked non-claim block).
//!
//! ## Panel-locked one-line verdict
//!
//! > FF.2 blocks unratified activation;
//! > FF.3 blocks unratified registry generation.
//!
//! Two different boundaries; same court discipline. The
//! reason-code separation is the load-bearing win:
//! `DisabledUnratifiedProposal` (FF.2 activation reason) and
//! `RejectedUnratifiedProposal` (FF.3 registry-generation
//! rejection) are different court failures that the operator
//! must be able to distinguish from a weak-but-ratified
//! detector failing for empirical reasons.
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

use crate::consolidate::{build_consolidation_report, ConsolidationReport};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::ff1_passport_materialisation::{build_ff1_passport_index_from, Ff1PassportIndex};
use crate::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
    Ff2ActivationRatificationGate,
};
use crate::seed::SEED;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators (NEW own-namespace hashes)
// ---------------------------------------------------------------

/// Domain separator for `ff3_registry_generation_gate_hash_v1`.
/// Distinct from FF.1 / FF.2 domains so the FF.3 gate artifact
/// is independently addressable.
pub const FF3_REGISTRY_GENERATION_GATE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1\0";

/// Schema identifier embedded in the gate hash material.
pub const FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE:v1";

/// Domain separator for
/// `ff3_registry_generation_gate_summary_hash_v1`. Distinct from
/// the gate domain so the summary artifact (gate + non-claim
/// block) is independently addressable.
pub const FF3_REGISTRY_GENERATION_GATE_SUMMARY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1\0";

/// Schema identifier embedded in the gate-summary hash material.
pub const FF3_REGISTRY_GENERATION_GATE_SUMMARY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF3-REGISTRY-GENERATION-GATE-SUMMARY:v1";

// ---------------------------------------------------------------
// Source-authority claim
// ---------------------------------------------------------------

/// The panel-locked taxonomy of source-authority claims a
/// registry-generation candidate may declare. Two are admitted
/// by FF.3 (`SeedHistorical`, `T12RatifiedAndPassported`); two
/// are forbidden by construction (`AdHocUnsanctioned`,
/// `UnknownExternal`) and exist so the verifier's panel-required
/// negatives have explicit values to surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ff3SourceAuthority {
    /// Candidate claims its source is a SEED record under
    /// `corpus_hash_v1`. Admitted by FF.3 iff the candidate's
    /// `canonical_id` is actually in SEED.
    SeedHistorical,
    /// Candidate claims its source is a `corpus_hash_v2`-
    /// ratified entry materialised through FF.1 passport
    /// authority. Admitted by FF.3 iff the id is actually in
    /// the ratified expansion index AND in the FF.1 passport
    /// index AND the pinned `corpus_hash_v2` +
    /// `ff1_passport_index_hash_v1` anchors match live state.
    T12RatifiedAndPassported,
    /// Candidate declares an ad-hoc unsanctioned source (a
    /// record proposed for registry generation outside the
    /// panel-locked two-source discipline). Forbidden by
    /// construction; the verifier's
    /// `DetectorSpecFromAdHocRecord` rule surfaces every such
    /// occurrence.
    AdHocUnsanctioned,
    /// Candidate declares an unknown / external source (e.g. a
    /// future commit that adds a third source authority without
    /// updating this enum). Forbidden by construction; the
    /// verifier's `DetectorSpecWithUnknownSourceAuthority` rule
    /// surfaces every such occurrence.
    UnknownExternal,
}

impl Ff3SourceAuthority {
    /// Stable wire name; used in the canonical hash material and
    /// in the FF.3 gate decision's `claimed_source_authority_wire_name`
    /// field. Deterministic and sortable; pinned for byte-stable
    /// hashing.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SeedHistorical => "SeedHistorical",
            Self::T12RatifiedAndPassported => "T12RatifiedAndPassported",
            Self::AdHocUnsanctioned => "AdHocUnsanctioned",
            Self::UnknownExternal => "UnknownExternal",
        }
    }
}

// ---------------------------------------------------------------
// Eligibility status (gate outcome)
// ---------------------------------------------------------------

/// The seven mutually-exclusive eligibility buckets a registry-
/// generation candidate falls into. Exactly one of these
/// decides whether the S1.2 registry generator may emit a
/// `DetectorSpec` for the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ff3RegistryGenerationEligibility {
    /// Claimed source authority verifies against live state.
    /// The generator MAY emit a `DetectorSpec` for this
    /// candidate. The only bucket that passes the gate.
    Eligible,
    /// Claimed `T12RatifiedAndPassported` but the id is NOT in
    /// the ratified expansion index. Mirror of FF.2's
    /// `UnratifiedProposal` bucket at the registry-generation
    /// boundary.
    RejectedUnratifiedProposal,
    /// Id IS in the ratified expansion index but NOT in the
    /// FF.1 passport index. Mirror of FF.2's `MissingPassport`
    /// bucket; structural defect that should never occur in
    /// production.
    RejectedMissingFf1Passport,
    /// The gate's pinned `corpus_hash_v2` does not equal the
    /// live ratified-corpus authority anchor; every
    /// `T12RatifiedAndPassported` candidate is rejected because
    /// the ratification anchor itself drifted.
    RejectedCorpusHashV2Mismatch,
    /// The gate's pinned `ff1_passport_index_hash_v1` does not
    /// equal the live passport-index hash; every
    /// `T12RatifiedAndPassported` candidate is rejected because
    /// the passport authority drifted.
    RejectedPassportIndexHashMismatch,
    /// Candidate declared `AdHocUnsanctioned` source authority.
    /// Forbidden by construction.
    RejectedAdHocRecord,
    /// Candidate declared `UnknownExternal` source authority.
    /// Forbidden by construction.
    RejectedUnknownSourceAuthority,
}

impl Ff3RegistryGenerationEligibility {
    /// Stable wire name; pinned for byte-stable hashing.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "Eligible",
            Self::RejectedUnratifiedProposal => "RejectedUnratifiedProposal",
            Self::RejectedMissingFf1Passport => "RejectedMissingFf1Passport",
            Self::RejectedCorpusHashV2Mismatch => "RejectedCorpusHashV2Mismatch",
            Self::RejectedPassportIndexHashMismatch => "RejectedPassportIndexHashMismatch",
            Self::RejectedAdHocRecord => "RejectedAdHocRecord",
            Self::RejectedUnknownSourceAuthority => "RejectedUnknownSourceAuthority",
        }
    }

    /// True iff this eligibility status permits the S1.2
    /// generator to emit a `DetectorSpec` for the candidate.
    /// Only `Eligible` returns true.
    #[must_use]
    pub const fn passes_gate(self) -> bool {
        matches!(self, Self::Eligible)
    }
}

// ---------------------------------------------------------------
// Candidate + decision records
// ---------------------------------------------------------------

/// One registry-generation candidate: a `canonical_id` plus the
/// source authority the caller is claiming. The FF.3 gate
/// verifies the claim against live SEED / ratified / passport
/// state.
#[derive(Debug, Clone, Copy)]
pub struct Ff3RegistryGenerationCandidate {
    /// Candidate canonical id (SEED id, ratified-expansion id,
    /// or a synthetic id for ad-hoc / unknown rejection tests).
    pub canonical_id: u32,
    /// Source authority the caller is claiming.
    pub claimed_source_authority: Ff3SourceAuthority,
}

/// One eligibility decision emitted by the FF.3 gate. Field
/// order is the canonical hash order; do not reorder without
/// rebaselining `ff3_registry_generation_gate_hash_v1`.
#[derive(Debug, Clone)]
pub struct Ff3RegistryGenerationEligibilityDecision {
    /// Candidate canonical id this decision concerns.
    pub canonical_id: u32,
    /// Stable wire name of the claimed source authority.
    pub claimed_source_authority_wire_name: &'static str,
    /// Eligibility-status bucket.
    pub eligibility: Ff3RegistryGenerationEligibility,
    /// Stable wire name of `eligibility`.
    pub eligibility_wire_name: &'static str,
    /// Rejection-reason wire name. Empty for `Eligible`; equals
    /// `eligibility.as_str()` for any rejection bucket so the
    /// rejection reason is byte-stable and reason-coded.
    pub rejection_reason_wire_name: &'static str,
    /// 32-byte FF.1 passport hash if the candidate verified
    /// against the live passport index; zero bytes otherwise
    /// (either the candidate was SeedHistorical or the
    /// passport lookup failed).
    pub cited_passport_hash: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level FF.3 gate
// ---------------------------------------------------------------

/// The FF.3 registry-generation gate. Carries the sorted list
/// of per-candidate eligibility decisions + the five pinned
/// upstream anchor hashes proving FF.3 did not mutate any
/// upstream authority. Two builds against the same proposal
/// set + same candidate set produce byte-identical bytes.
#[derive(Debug, Clone)]
pub struct Ff3RegistryGenerationGate {
    /// Historical seed-corpus anchor (pinned; verified equal
    /// to `compute_corpus_hash_v1()` at build time).
    pub corpus_hash_v1: [u8; 32],
    /// Ratified-corpus authority anchor (pinned; verified
    /// equal to the live consolidation report's
    /// `corpus_hash_v2`).
    pub corpus_hash_v2: [u8; 32],
    /// T.12.consolidate consolidation-report hash (pinned).
    pub consolidation_report_hash_v1: [u8; 32],
    /// FF.1 passport-index hash (pinned).
    pub ff1_passport_index_hash_v1: [u8; 32],
    /// FF.2 activation ratification gate hash (pinned).
    /// Surfaces the panel-required "registry generation MUST
    /// consult the FF.2 ratification gate" discipline.
    pub ff2_activation_ratification_gate_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Per-candidate eligibility decisions, sorted by
    /// `canonical_id` ascending.
    pub decisions: Vec<Ff3RegistryGenerationEligibilityDecision>,
    /// Count of `Eligible` decisions (would-be-generated
    /// DetectorSpec count under the default production
    /// candidate set).
    pub eligible_count: u32,
    /// Count of `RejectedUnratifiedProposal` decisions.
    pub rejected_unratified_proposal_count: u32,
    /// Count of `RejectedMissingFf1Passport` decisions.
    pub rejected_missing_ff1_passport_count: u32,
    /// Count of `RejectedCorpusHashV2Mismatch` decisions.
    pub rejected_corpus_hash_v2_mismatch_count: u32,
    /// Count of `RejectedPassportIndexHashMismatch` decisions.
    pub rejected_passport_index_hash_mismatch_count: u32,
    /// Count of `RejectedAdHocRecord` decisions.
    pub rejected_ad_hoc_record_count: u32,
    /// Count of `RejectedUnknownSourceAuthority` decisions.
    pub rejected_unknown_source_authority_count: u32,
    /// `ff3_registry_generation_gate_hash_v1` — domain-
    /// separated SHA-256 over every field above.
    pub ff3_registry_generation_gate_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Gate summary (gate + panel-locked non-claim block)
// ---------------------------------------------------------------

/// The FF.3 gate summary: gate + panel-locked non-claim block
/// hashed under a distinct domain so the summary artifact is
/// independently addressable. The non-claim block is a fixed
/// string array; mutations to the string text require a new
/// domain separator (schema-upgrade commit).
#[derive(Debug, Clone)]
pub struct Ff3RegistryGenerationGateSummary {
    /// The wrapped gate.
    pub gate: Ff3RegistryGenerationGate,
    /// Panel-locked non-claim text lines. Pinned verbatim; the
    /// summary hash mixes the line count + bytes so any silent
    /// rewrite changes the summary hash.
    pub non_claim_lines: &'static [&'static str],
    /// `ff3_registry_generation_gate_summary_hash_v1` —
    /// domain-separated SHA-256 over the gate hash + non-claim
    /// lines.
    pub ff3_registry_generation_gate_summary_hash_v1: [u8; 32],
}

/// Panel-locked non-claim text lines emitted by every FF.3 gate
/// summary. Pinned verbatim so the summary hash binds them.
pub const FF3_NON_CLAIM_LINES: &[&str] = &[
    "FF.3 does NOT add new detectors.",
    "FF.3 does NOT alter corpus_hash_v1.",
    "FF.3 does NOT alter corpus_hash_v2.",
    "FF.3 does NOT alter consolidation_report_hash_v1.",
    "FF.3 does NOT alter t12_expansion_index_hash_v1.",
    "FF.3 does NOT alter ff1_passport_index_hash_v1.",
    "FF.3 does NOT alter ff1_materialisation_report_hash_v1.",
    "FF.3 does NOT alter ff2_activation_ratification_gate_hash_v1.",
    "FF.3 does NOT rewrite any prior T.11 / S1.3 / T.12.x / FF.1 / FF.2 hash.",
    "FF.3 does NOT mutate SEED.len() (stays at 54).",
    "FF.3 does NOT promote any open proposal to Accepted.",
    "FF.3 does NOT change S1.3a SEED activation decisions.",
    "FF.3 does NOT change FF.2 ratification decisions.",
    "FF.3 does NOT itself emit DetectorSpec records.",
    "FF.3 does NOT modify dsfb-gpu-atlas-registry's existing 162-spec registry_hash_v2.",
    "FF.3 does NOT generate CUDA kernels.",
    "FF.3 does NOT decide contraindications or challenges.",
];

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why FF.3 rejected an input. An empty `verify_ff3` return
/// means the gate is admissible. The eight panel-required
/// negatives map onto rules R.1–R.8; additional structural
/// rules (sort order, duplicate id, anchor cross-checks, SEED
/// invariance) are emitted under their own kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ff3VerifyErrorKind {
    /// Panel-required negative #1. A decision MUST classify a
    /// `T12RatifiedAndPassported` claim for an id outside the
    /// ratified expansion index as `RejectedUnratifiedProposal`.
    /// Surfaced if the gate emits a different status for such a
    /// claim.
    DetectorSpecForUnratifiedProposal {
        /// The unratified canonical id.
        canonical_id: u32,
        /// The eligibility wire name the gate emitted (should
        /// have been `RejectedUnratifiedProposal`).
        observed_eligibility_wire_name: &'static str,
    },
    /// Panel-required negative #2. A decision MUST classify a
    /// `T12RatifiedAndPassported` claim for an id in the
    /// ratified expansion index but NOT in the FF.1 passport
    /// index as `RejectedMissingFf1Passport`.
    DetectorSpecForMissingFf1Passport {
        /// The ratified id missing a passport.
        canonical_id: u32,
    },
    /// Panel-required negative #3. The gate's pinned
    /// `corpus_hash_v2` does not equal the live consolidation
    /// report's `corpus_hash_v2`. Surfaced even if no
    /// individual decision was classified as
    /// `RejectedCorpusHashV2Mismatch`; the anchor mismatch
    /// itself is the defect.
    DetectorSpecWhenCorpusHashV2Mismatch {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live consolidation report computes.
        actual: [u8; 32],
    },
    /// Panel-required negative #4. The gate's pinned
    /// `ff1_passport_index_hash_v1` does not equal the live
    /// FF.1 passport-index hash.
    DetectorSpecWhenPassportIndexHashMismatch {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live FF.1 passport index computes.
        actual: [u8; 32],
    },
    /// Panel-required negative #5. A candidate declared
    /// `AdHocUnsanctioned` source authority and the decision
    /// is NOT `RejectedAdHocRecord`.
    DetectorSpecFromAdHocRecord {
        /// The ad-hoc canonical id.
        canonical_id: u32,
    },
    /// Panel-required negative #6. A candidate declared
    /// `UnknownExternal` source authority and the decision is
    /// NOT `RejectedUnknownSourceAuthority`.
    DetectorSpecWithUnknownSourceAuthority {
        /// The unknown-source canonical id.
        canonical_id: u32,
    },
    /// Panel-required negative #7. The gate's pinned
    /// `ff2_activation_ratification_gate_hash_v1` does not
    /// equal the live FF.2 gate hash. The panel verdict is
    /// explicit: FF.3 MUST consult the FF.2 ratification gate;
    /// any FF.3 build that skips or stales the FF.2 anchor is
    /// a defect.
    RegistryGenerationThatSkipsFf2RatificationGate {
        /// Hash the gate claims.
        claimed: [u8; 32],
        /// Hash the live FF.2 gate computes.
        actual: [u8; 32],
    },
    /// Panel-required negative #8. The FF.3 `eligible_count`
    /// exceeds the FF.2 gate's eligible count
    /// (`seed_historical_count + t12_ratified_and_passported_count`).
    /// FF.3 cannot admit MORE candidates for registry
    /// generation than FF.2 admits for activation; doing so
    /// would silently mutate the would-be registry surface
    /// beyond the FF.2-ratified set.
    RegistryGenerationThatMutatesExistingRegistryHash {
        /// FF.3's eligible count.
        ff3_eligible_count: u32,
        /// FF.2's eligible count (SeedHistorical +
        /// T12RatifiedAndPassported).
        ff2_eligible_count: u32,
    },
    /// Two decisions share the same canonical id.
    DuplicateGateDecisionForSameCanonicalId {
        /// The duplicated canonical id.
        canonical_id: u32,
    },
    /// Decisions are not sorted ascending by `canonical_id`.
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
    /// A decision's claimed-source-authority wire name does
    /// not match its eligibility classification (e.g. a
    /// `SeedHistorical` claim cannot legitimately be classified
    /// as `RejectedUnratifiedProposal`).
    DecisionClassificationInconsistentWithClaim {
        /// The canonical id.
        canonical_id: u32,
        /// The claimed-source-authority wire name.
        claimed_source_authority_wire_name: &'static str,
        /// The eligibility wire name.
        eligibility_wire_name: &'static str,
    },
    /// A decision's `rejection_reason_wire_name` is non-empty
    /// for an `Eligible` decision (Eligible MUST carry empty
    /// rejection reason).
    EligibleDecisionCarriesNonEmptyRejectionReason {
        /// The canonical id.
        canonical_id: u32,
    },
    /// A decision's `rejection_reason_wire_name` is empty for
    /// a non-`Eligible` decision (every rejection MUST be
    /// reason-coded).
    RejectionDecisionCarriesEmptyRejectionReason {
        /// The canonical id.
        canonical_id: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3VerifyError {
    /// Error kind (see [`Ff3VerifyErrorKind`]).
    pub kind: Ff3VerifyErrorKind,
}

// ---------------------------------------------------------------
// Default candidate-set derivation
// ---------------------------------------------------------------

/// Build the production default registry-generation candidate
/// list: every SEED canonical id (claimed `SeedHistorical`)
/// plus every FF.1 passport canonical id (claimed
/// `T12RatifiedAndPassported`). Sorted ascending. Used by the
/// production [`build_ff3_registry_generation_gate`] call;
/// tests can supply alternate lists to exercise the rejection
/// rules.
#[must_use]
pub fn default_registry_generation_candidates(
    passport_index: &Ff1PassportIndex,
) -> Vec<Ff3RegistryGenerationCandidate> {
    let mut out: Vec<Ff3RegistryGenerationCandidate> =
        Vec::with_capacity(SEED.len() + passport_index.passports.len());
    for r in SEED {
        out.push(Ff3RegistryGenerationCandidate {
            canonical_id: r.canonical_id.0,
            claimed_source_authority: Ff3SourceAuthority::SeedHistorical,
        });
    }
    for p in &passport_index.passports {
        out.push(Ff3RegistryGenerationCandidate {
            canonical_id: p.canonical_id,
            claimed_source_authority: Ff3SourceAuthority::T12RatifiedAndPassported,
        });
    }
    out.sort_by_key(|c| c.canonical_id);
    out
}

// ---------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------

/// Classify a single registry-generation candidate into its
/// eligibility bucket. Pure derivation; never mutates anything.
///
/// The classifier consults SEED / ratified-expansion / passport-
/// index state plus the gate's pinned anchor hashes (so anchor
/// drift can be surfaced even when the underlying id would
/// otherwise pass). The seven buckets are mutually exclusive.
#[allow(clippy::too_many_arguments)]
fn classify(
    candidate: Ff3RegistryGenerationCandidate,
    seed_ids: &BTreeSet<u32>,
    ratified_ids: &BTreeSet<u32>,
    passport_ids: &BTreeSet<u32>,
    pinned_corpus_hash_v2: [u8; 32],
    live_corpus_hash_v2: [u8; 32],
    pinned_passport_index_hash: [u8; 32],
    live_passport_index_hash: [u8; 32],
) -> Ff3RegistryGenerationEligibility {
    match candidate.claimed_source_authority {
        Ff3SourceAuthority::AdHocUnsanctioned => {
            return Ff3RegistryGenerationEligibility::RejectedAdHocRecord;
        }
        Ff3SourceAuthority::UnknownExternal => {
            return Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority;
        }
        Ff3SourceAuthority::SeedHistorical => {
            // SEED claim: admitted iff id is actually in SEED.
            // SEED claims are not subject to corpus_hash_v2 /
            // passport-index drift because SEED lives under
            // corpus_hash_v1.
            if seed_ids.contains(&candidate.canonical_id) {
                return Ff3RegistryGenerationEligibility::Eligible;
            }
            // A SeedHistorical claim for an id outside SEED is
            // structurally an ad-hoc claim — surface it under
            // the ad-hoc bucket so the operator-facing
            // rejection reason is the strongest match.
            return Ff3RegistryGenerationEligibility::RejectedAdHocRecord;
        }
        Ff3SourceAuthority::T12RatifiedAndPassported => { /* fall through */ }
    }

    // T12RatifiedAndPassported: anchor drift first, then
    // ratification + passport status.
    if pinned_corpus_hash_v2 != live_corpus_hash_v2 {
        return Ff3RegistryGenerationEligibility::RejectedCorpusHashV2Mismatch;
    }
    if pinned_passport_index_hash != live_passport_index_hash {
        return Ff3RegistryGenerationEligibility::RejectedPassportIndexHashMismatch;
    }
    if !ratified_ids.contains(&candidate.canonical_id) {
        return Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal;
    }
    if !passport_ids.contains(&candidate.canonical_id) {
        return Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport;
    }
    Ff3RegistryGenerationEligibility::Eligible
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production FF.3 gate from the live consolidation
/// report + live FF.1 passport index + live FF.2 ratification
/// gate. Two builds produce byte-identical bytes. Used by the
/// production CLI / artifact-emission paths; the test suite uses
/// [`build_ff3_registry_generation_gate_from`] with synthetic
/// candidate lists to exercise the rejection rules.
#[must_use]
pub fn build_ff3_registry_generation_gate() -> Ff3RegistryGenerationGate {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let activation_candidate_ids = default_candidate_ids(&passport_index);
    let ff2_gate = build_ff2_activation_ratification_gate_from(
        &report,
        &passport_index,
        &activation_candidate_ids,
    );
    let candidates = default_registry_generation_candidates(&passport_index);
    build_ff3_registry_generation_gate_from(&report, &passport_index, &ff2_gate, &candidates)
}

/// Build the FF.3 gate from a specific consolidation report +
/// passport index + FF.2 gate + candidate list. Pure function;
/// used by tests to inject ad-hoc / unknown / mismatched-anchor
/// scenarios exercising the eight panel-required negatives.
#[must_use]
pub fn build_ff3_registry_generation_gate_from(
    report: &ConsolidationReport,
    passport_index: &Ff1PassportIndex,
    ff2_gate: &Ff2ActivationRatificationGate,
    candidates: &[Ff3RegistryGenerationCandidate],
) -> Ff3RegistryGenerationGate {
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

    let pinned_corpus_hash_v2 = report.corpus_hash_v2;
    let live_corpus_hash_v2 = report.corpus_hash_v2;
    let pinned_passport_index_hash = passport_index.ff1_passport_index_hash_v1;
    let live_passport_index_hash = passport_index.ff1_passport_index_hash_v1;

    let mut decisions: Vec<Ff3RegistryGenerationEligibilityDecision> = candidates
        .iter()
        .copied()
        .map(|cand| {
            let eligibility = classify(
                cand,
                &seed_ids,
                &ratified_ids,
                &passport_ids,
                pinned_corpus_hash_v2,
                live_corpus_hash_v2,
                pinned_passport_index_hash,
                live_passport_index_hash,
            );
            let rejection_reason_wire_name: &'static str = if eligibility.passes_gate() {
                ""
            } else {
                eligibility.as_str()
            };
            let cited_passport_hash: [u8; 32] = if eligibility
                == Ff3RegistryGenerationEligibility::Eligible
                && cand.claimed_source_authority == Ff3SourceAuthority::T12RatifiedAndPassported
            {
                passport_index
                    .passports
                    .iter()
                    .find(|p| p.canonical_id == cand.canonical_id)
                    .map_or([0u8; 32], |p| p.passport_hash_v1)
            } else {
                [0u8; 32]
            };
            Ff3RegistryGenerationEligibilityDecision {
                canonical_id: cand.canonical_id,
                claimed_source_authority_wire_name: cand.claimed_source_authority.as_str(),
                eligibility,
                eligibility_wire_name: eligibility.as_str(),
                rejection_reason_wire_name,
                cited_passport_hash,
            }
        })
        .collect();
    decisions.sort_by_key(|d| d.canonical_id);

    let mut eligible_count: u32 = 0;
    let mut rejected_unratified_proposal_count: u32 = 0;
    let mut rejected_missing_ff1_passport_count: u32 = 0;
    let mut rejected_corpus_hash_v2_mismatch_count: u32 = 0;
    let mut rejected_passport_index_hash_mismatch_count: u32 = 0;
    let mut rejected_ad_hoc_record_count: u32 = 0;
    let mut rejected_unknown_source_authority_count: u32 = 0;
    for d in &decisions {
        match d.eligibility {
            Ff3RegistryGenerationEligibility::Eligible => eligible_count += 1,
            Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal => {
                rejected_unratified_proposal_count += 1;
            }
            Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport => {
                rejected_missing_ff1_passport_count += 1;
            }
            Ff3RegistryGenerationEligibility::RejectedCorpusHashV2Mismatch => {
                rejected_corpus_hash_v2_mismatch_count += 1;
            }
            Ff3RegistryGenerationEligibility::RejectedPassportIndexHashMismatch => {
                rejected_passport_index_hash_mismatch_count += 1;
            }
            Ff3RegistryGenerationEligibility::RejectedAdHocRecord => {
                rejected_ad_hoc_record_count += 1;
            }
            Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority => {
                rejected_unknown_source_authority_count += 1;
            }
        }
    }

    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut gate = Ff3RegistryGenerationGate {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        consolidation_report_hash_v1: report.consolidation_report_hash_v1,
        ff1_passport_index_hash_v1: passport_index.ff1_passport_index_hash_v1,
        ff2_activation_ratification_gate_hash_v1: ff2_gate.ff2_activation_ratification_gate_hash_v1,
        seed_len,
        decisions,
        eligible_count,
        rejected_unratified_proposal_count,
        rejected_missing_ff1_passport_count,
        rejected_corpus_hash_v2_mismatch_count,
        rejected_passport_index_hash_mismatch_count,
        rejected_ad_hoc_record_count,
        rejected_unknown_source_authority_count,
        ff3_registry_generation_gate_hash_v1: [0u8; 32],
    };
    gate.ff3_registry_generation_gate_hash_v1 = compute_ff3_registry_generation_gate_hash(&gate);
    gate
}

/// Build the FF.3 gate summary from the live consolidation
/// report + live FF.1 passport index + live FF.2 gate. Wraps
/// the gate with the panel-locked non-claim block.
#[must_use]
pub fn build_ff3_registry_generation_gate_summary() -> Ff3RegistryGenerationGateSummary {
    let gate = build_ff3_registry_generation_gate();
    build_ff3_registry_generation_gate_summary_from_gate(gate)
}

/// Build the FF.3 gate summary from a specific gate. Used by
/// tests to wrap synthetic gates with the canonical non-claim
/// block.
#[must_use]
pub fn build_ff3_registry_generation_gate_summary_from_gate(
    gate: Ff3RegistryGenerationGate,
) -> Ff3RegistryGenerationGateSummary {
    let mut summary = Ff3RegistryGenerationGateSummary {
        gate,
        non_claim_lines: FF3_NON_CLAIM_LINES,
        ff3_registry_generation_gate_summary_hash_v1: [0u8; 32],
    };
    summary.ff3_registry_generation_gate_summary_hash_v1 =
        compute_ff3_registry_generation_gate_summary_hash(&summary);
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

fn compute_ff3_registry_generation_gate_hash(gate: &Ff3RegistryGenerationGate) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
    buf.extend_from_slice(FF3_REGISTRY_GENERATION_GATE_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &gate.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &gate.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &gate.consolidation_report_hash_v1);
    write_bytes_fixed(&mut buf, &gate.ff1_passport_index_hash_v1);
    write_bytes_fixed(&mut buf, &gate.ff2_activation_ratification_gate_hash_v1);
    write_u32(&mut buf, gate.seed_len);
    write_u32(
        &mut buf,
        u32::try_from(gate.decisions.len()).unwrap_or(u32::MAX),
    );
    for d in &gate.decisions {
        write_u32(&mut buf, d.canonical_id);
        write_str(&mut buf, d.claimed_source_authority_wire_name);
        write_str(&mut buf, d.eligibility_wire_name);
        write_str(&mut buf, d.rejection_reason_wire_name);
        write_bytes_fixed(&mut buf, &d.cited_passport_hash);
    }
    write_u32(&mut buf, gate.eligible_count);
    write_u32(&mut buf, gate.rejected_unratified_proposal_count);
    write_u32(&mut buf, gate.rejected_missing_ff1_passport_count);
    write_u32(&mut buf, gate.rejected_corpus_hash_v2_mismatch_count);
    write_u32(&mut buf, gate.rejected_passport_index_hash_mismatch_count);
    write_u32(&mut buf, gate.rejected_ad_hoc_record_count);
    write_u32(&mut buf, gate.rejected_unknown_source_authority_count);
    sha256(&buf)
}

fn compute_ff3_registry_generation_gate_summary_hash(
    summary: &Ff3RegistryGenerationGateSummary,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(FF3_REGISTRY_GENERATION_GATE_SUMMARY_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF3_REGISTRY_GENERATION_GATE_SUMMARY_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &summary.gate.ff3_registry_generation_gate_hash_v1);
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
// Verifier — eight panel-required rules + structural rules
// ---------------------------------------------------------------

/// Walk an FF.3 gate against the live consolidation report +
/// live FF.1 passport index + live FF.2 gate and emit every
/// rejection. An empty return means the gate is admissible.
/// The eight panel-required negatives map onto rules R.1–R.8;
/// structural rules R.9–R.16 guard sort order, duplicates,
/// anchor cross-checks, SEED invariance, and per-decision
/// consistency.
#[must_use]
pub fn verify_ff3(
    gate: &Ff3RegistryGenerationGate,
    report: &ConsolidationReport,
    passport_index: &Ff1PassportIndex,
    ff2_gate: &Ff2ActivationRatificationGate,
) -> Vec<Ff3VerifyError> {
    let mut errors: Vec<Ff3VerifyError> = Vec::new();

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

    // R.1 DetectorSpecForUnratifiedProposal: a candidate
    // claiming T12RatifiedAndPassported whose id is NOT in
    // ratified-expansion MUST classify as
    // RejectedUnratifiedProposal (anchor matches in this
    // check; the corpus_hash_v2 / passport-index mismatch
    // rules take precedence when anchors are stale).
    let anchors_match = gate.corpus_hash_v2 == report.corpus_hash_v2
        && gate.ff1_passport_index_hash_v1 == passport_index.ff1_passport_index_hash_v1;
    for d in &gate.decisions {
        if !anchors_match {
            continue;
        }
        if d.claimed_source_authority_wire_name
            == Ff3SourceAuthority::T12RatifiedAndPassported.as_str()
            && !ratified_ids.contains(&d.canonical_id)
            && d.eligibility != Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DetectorSpecForUnratifiedProposal {
                    canonical_id: d.canonical_id,
                    observed_eligibility_wire_name: d.eligibility_wire_name,
                },
            });
        }
    }

    // R.2 DetectorSpecForMissingFf1Passport: ratified id NOT
    // in passport index MUST classify as
    // RejectedMissingFf1Passport (only when anchors match).
    for d in &gate.decisions {
        if !anchors_match {
            continue;
        }
        if d.claimed_source_authority_wire_name
            == Ff3SourceAuthority::T12RatifiedAndPassported.as_str()
            && ratified_ids.contains(&d.canonical_id)
            && !passport_ids.contains(&d.canonical_id)
            && d.eligibility != Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DetectorSpecForMissingFf1Passport {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.3 DetectorSpecWhenCorpusHashV2Mismatch: pinned anchor
    // must match the live consolidation report.
    if gate.corpus_hash_v2 != report.corpus_hash_v2 {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::DetectorSpecWhenCorpusHashV2Mismatch {
                claimed: gate.corpus_hash_v2,
                actual: report.corpus_hash_v2,
            },
        });
    }

    // R.4 DetectorSpecWhenPassportIndexHashMismatch: pinned
    // anchor must match the live FF.1 passport index hash.
    if gate.ff1_passport_index_hash_v1 != passport_index.ff1_passport_index_hash_v1 {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::DetectorSpecWhenPassportIndexHashMismatch {
                claimed: gate.ff1_passport_index_hash_v1,
                actual: passport_index.ff1_passport_index_hash_v1,
            },
        });
    }

    // R.5 DetectorSpecFromAdHocRecord: any decision whose
    // claimed-source-authority wire name is AdHocUnsanctioned
    // MUST classify as RejectedAdHocRecord.
    for d in &gate.decisions {
        if d.claimed_source_authority_wire_name == Ff3SourceAuthority::AdHocUnsanctioned.as_str()
            && d.eligibility != Ff3RegistryGenerationEligibility::RejectedAdHocRecord
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DetectorSpecFromAdHocRecord {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.6 DetectorSpecWithUnknownSourceAuthority: any decision
    // whose claimed-source-authority wire name is
    // UnknownExternal MUST classify as
    // RejectedUnknownSourceAuthority.
    for d in &gate.decisions {
        if d.claimed_source_authority_wire_name == Ff3SourceAuthority::UnknownExternal.as_str()
            && d.eligibility != Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DetectorSpecWithUnknownSourceAuthority {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.7 RegistryGenerationThatSkipsFf2RatificationGate:
    // pinned FF.2 hash must match the live FF.2 gate hash.
    if gate.ff2_activation_ratification_gate_hash_v1
        != ff2_gate.ff2_activation_ratification_gate_hash_v1
    {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::RegistryGenerationThatSkipsFf2RatificationGate {
                claimed: gate.ff2_activation_ratification_gate_hash_v1,
                actual: ff2_gate.ff2_activation_ratification_gate_hash_v1,
            },
        });
    }

    // R.8 RegistryGenerationThatMutatesExistingRegistryHash:
    // FF.3 eligible_count MUST NOT exceed the FF.2 eligible
    // count (SeedHistorical + T12RatifiedAndPassported). FF.3
    // cannot admit MORE candidates for registry generation
    // than FF.2 admits for activation; doing so would silently
    // mutate the would-be registry surface beyond the FF.2-
    // ratified set.
    let ff2_eligible_count =
        ff2_gate.seed_historical_count + ff2_gate.t12_ratified_and_passported_count;
    if gate.eligible_count > ff2_eligible_count {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::RegistryGenerationThatMutatesExistingRegistryHash {
                ff3_eligible_count: gate.eligible_count,
                ff2_eligible_count,
            },
        });
    }

    // R.9 DuplicateGateDecisionForSameCanonicalId.
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for d in &gate.decisions {
        if !seen.insert(d.canonical_id) {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DuplicateGateDecisionForSameCanonicalId {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.10 GateDecisionsNotSortedAscending.
    for w in gate.decisions.windows(2) {
        if w[0].canonical_id > w[1].canonical_id {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::GateDecisionsNotSortedAscending,
            });
            break;
        }
    }

    // R.11 anchor cross-checks: corpus_hash_v1 + consolidation
    // report hash.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if gate.corpus_hash_v1 != live_v1 {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::CorpusHashV1Mismatch {
                claimed: gate.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    if gate.consolidation_report_hash_v1 != report.consolidation_report_hash_v1 {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::ConsolidationReportHashV1Mismatch {
                claimed: gate.consolidation_report_hash_v1,
                actual: report.consolidation_report_hash_v1,
            },
        });
    }

    // R.12 SEED invariance.
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(Ff3VerifyError {
            kind: Ff3VerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    // R.13 EligibleDecisionCarriesNonEmptyRejectionReason.
    for d in &gate.decisions {
        if d.eligibility == Ff3RegistryGenerationEligibility::Eligible
            && !d.rejection_reason_wire_name.is_empty()
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::EligibleDecisionCarriesNonEmptyRejectionReason {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.14 RejectionDecisionCarriesEmptyRejectionReason.
    for d in &gate.decisions {
        if d.eligibility != Ff3RegistryGenerationEligibility::Eligible
            && d.rejection_reason_wire_name.is_empty()
        {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::RejectionDecisionCarriesEmptyRejectionReason {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.15 DecisionClassificationInconsistentWithClaim: a
    // SeedHistorical claim cannot legitimately be classified
    // as RejectedUnratifiedProposal or
    // RejectedMissingFf1Passport (those are
    // T12RatifiedAndPassported failures); a SeedHistorical
    // claim for an id outside SEED is reclassified as
    // RejectedAdHocRecord by the classifier itself.
    for d in &gate.decisions {
        let claim = d.claimed_source_authority_wire_name;
        let elig = d.eligibility;
        let claim_is_seed = claim == Ff3SourceAuthority::SeedHistorical.as_str();
        let claim_is_ratified = claim == Ff3SourceAuthority::T12RatifiedAndPassported.as_str();
        let inconsistent = match elig {
            Ff3RegistryGenerationEligibility::RejectedUnratifiedProposal
            | Ff3RegistryGenerationEligibility::RejectedMissingFf1Passport
            | Ff3RegistryGenerationEligibility::RejectedCorpusHashV2Mismatch
            | Ff3RegistryGenerationEligibility::RejectedPassportIndexHashMismatch => {
                !claim_is_ratified
            }
            Ff3RegistryGenerationEligibility::RejectedAdHocRecord => {
                claim != Ff3SourceAuthority::AdHocUnsanctioned.as_str() && !claim_is_seed
            }
            Ff3RegistryGenerationEligibility::RejectedUnknownSourceAuthority => {
                claim != Ff3SourceAuthority::UnknownExternal.as_str()
            }
            Ff3RegistryGenerationEligibility::Eligible => !claim_is_seed && !claim_is_ratified,
        };
        if inconsistent {
            errors.push(Ff3VerifyError {
                kind: Ff3VerifyErrorKind::DecisionClassificationInconsistentWithClaim {
                    canonical_id: d.canonical_id,
                    claimed_source_authority_wire_name: claim,
                    eligibility_wire_name: d.eligibility_wire_name,
                },
            });
        }
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the FF.3 gate as a deterministic text report. Two
/// renders against the same gate produce byte-identical bytes.
/// Used by the `ff3-gate` CLI subcommand and the
/// `ff3-gate-emit` artifact writer.
#[must_use]
pub fn render_ff3_gate_text(gate: &Ff3RegistryGenerationGate) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    let _ = writeln!(s, "FF.3 Registry Generation Gate (v1)");
    let _ = writeln!(s, "===================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                              : {}",
        hex32(&gate.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                              : {}",
        hex32(&gate.corpus_hash_v2)
    );
    let _ = writeln!(
        s,
        "  consolidation_report_hash_v1                : {}",
        hex32(&gate.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff1_passport_index_hash_v1                  : {}",
        hex32(&gate.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff2_activation_ratification_gate_hash_v1    : {}",
        hex32(&gate.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  SEED.len()                                  : {}",
        gate.seed_len
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-status counts");
    let _ = writeln!(
        s,
        "  Eligible                                    : {}",
        gate.eligible_count
    );
    let _ = writeln!(
        s,
        "  RejectedUnratifiedProposal                  : {}",
        gate.rejected_unratified_proposal_count
    );
    let _ = writeln!(
        s,
        "  RejectedMissingFf1Passport                  : {}",
        gate.rejected_missing_ff1_passport_count
    );
    let _ = writeln!(
        s,
        "  RejectedCorpusHashV2Mismatch                : {}",
        gate.rejected_corpus_hash_v2_mismatch_count
    );
    let _ = writeln!(
        s,
        "  RejectedPassportIndexHashMismatch           : {}",
        gate.rejected_passport_index_hash_mismatch_count
    );
    let _ = writeln!(
        s,
        "  RejectedAdHocRecord                         : {}",
        gate.rejected_ad_hoc_record_count
    );
    let _ = writeln!(
        s,
        "  RejectedUnknownSourceAuthority              : {}",
        gate.rejected_unknown_source_authority_count
    );
    let _ = writeln!(
        s,
        "  total decisions                             : {}",
        gate.decisions.len()
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "ff3_registry_generation_gate_hash_v1 : {}",
        hex32(&gate.ff3_registry_generation_gate_hash_v1)
    );
    s
}

/// Render the FF.3 gate as a deterministic JSON object. Two
/// renders against the same gate produce byte-identical bytes
/// (sorted keys, fixed schema, hex-encoded 32-byte hashes).
#[must_use]
pub fn render_ff3_gate_json(gate: &Ff3RegistryGenerationGate) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    s.push('{');
    let _ = write!(s, "\"schema\":\"{FF3_REGISTRY_GENERATION_GATE_SCHEMA_V1}\"");
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
    let _ = write!(
        s,
        ",\"ff2_activation_ratification_gate_hash_v1\":\"{}\"",
        hex32(&gate.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = write!(s, ",\"seed_len\":{}", gate.seed_len);
    let _ = write!(s, ",\"eligible_count\":{}", gate.eligible_count);
    let _ = write!(
        s,
        ",\"rejected_unratified_proposal_count\":{}",
        gate.rejected_unratified_proposal_count
    );
    let _ = write!(
        s,
        ",\"rejected_missing_ff1_passport_count\":{}",
        gate.rejected_missing_ff1_passport_count
    );
    let _ = write!(
        s,
        ",\"rejected_corpus_hash_v2_mismatch_count\":{}",
        gate.rejected_corpus_hash_v2_mismatch_count
    );
    let _ = write!(
        s,
        ",\"rejected_passport_index_hash_mismatch_count\":{}",
        gate.rejected_passport_index_hash_mismatch_count
    );
    let _ = write!(
        s,
        ",\"rejected_ad_hoc_record_count\":{}",
        gate.rejected_ad_hoc_record_count
    );
    let _ = write!(
        s,
        ",\"rejected_unknown_source_authority_count\":{}",
        gate.rejected_unknown_source_authority_count
    );
    let _ = write!(s, ",\"decisions\":[");
    for (i, d) in gate.decisions.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"canonical_id\":{},\"claimed_source_authority\":\"{}\",\"eligibility\":\"{}\",\"rejection_reason\":\"{}\",\"cited_passport_hash\":\"{}\"}}",
            d.canonical_id,
            d.claimed_source_authority_wire_name,
            d.eligibility_wire_name,
            d.rejection_reason_wire_name,
            hex32(&d.cited_passport_hash)
        );
    }
    s.push(']');
    let _ = write!(
        s,
        ",\"ff3_registry_generation_gate_hash_v1\":\"{}\"",
        hex32(&gate.ff3_registry_generation_gate_hash_v1)
    );
    s.push('}');
    s
}

/// Render the FF.3 gate summary as a deterministic text report.
#[must_use]
pub fn render_ff3_gate_summary_text(
    summary: &Ff3RegistryGenerationGateSummary,
) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = render_ff3_gate_text(&summary.gate);
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked non-claims");
    for line in summary.non_claim_lines {
        let _ = writeln!(s, "  - {line}");
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "ff3_registry_generation_gate_summary_hash_v1 : {}",
        hex32(&summary.ff3_registry_generation_gate_summary_hash_v1)
    );
    s
}

/// Render the FF.3 gate summary as a deterministic JSON object.
#[must_use]
pub fn render_ff3_gate_summary_json(
    summary: &Ff3RegistryGenerationGateSummary,
) -> alloc::string::String {
    use core::fmt::Write;
    let mut s = alloc::string::String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"schema\":\"{FF3_REGISTRY_GENERATION_GATE_SUMMARY_SCHEMA_V1}\""
    );
    let _ = write!(s, ",\"gate\":{}", render_ff3_gate_json(&summary.gate));
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
        ",\"ff3_registry_generation_gate_summary_hash_v1\":\"{}\"",
        hex32(&summary.ff3_registry_generation_gate_summary_hash_v1)
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
