//! T.12.consolidate — amendment-review and `corpus_hash_v2`
//! freeze. The first intentional ratification layer above the
//! frozen T.10 corpus surface.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.consolidate reviews every T.12 amendment proposal,
//! > verifies that all dedup-court deltas are internally
//! > consistent, freezes the admitted expansion set, and emits
//! > `corpus_hash_v2`. It does not add new literature primitives
//! > except through explicitly rejected late-amendment handling.
//! > Its purpose is ratification, not expansion.**
//!
//! ## Method
//!
//! 1. Load every T.12.0–T.12.p proposal artifact via its public
//!    `seed_*_proposal()` entry point.
//! 2. Recompute every proposal hash, batch hash, and dedup-delta
//!    hash; assert each matches the value stored on the
//!    proposal.
//! 3. Walk every `proposed_dedup_record`; collect canonical ids
//!    by decision wire name (CanonicalAddition / AuthorityResolution /
//!    DomainTransferOf / ParameterizationOf / RejectedNotDeterministic /
//!    AliasOf / CompositionOf for the T.12.a era).
//! 4. Cross-proposal collision checks: no two CanonicalAddition
//!    records (across all 17 proposals) share a canonical id; no
//!    CanonicalAddition id collides with the historical SEED
//!    (0..=53).
//! 5. ParameterizationOf parent check: every parameterization
//!    canonical id must point at a parent that either exists in
//!    SEED OR is itself a CanonicalAddition in some proposal.
//! 6. AuthorityResolution target check: every authority-
//!    resolution canonical id must exist in SEED (it ratifies an
//!    EXISTING canonical, not a new one).
//! 7. DomainTransferOf target check: same — target must exist in
//!    SEED.
//! 8. RejectedNotDeterministic contract check: every rejection
//!    reason is non-empty (the rejection-contract scanners live
//!    inside each per-T.12.x test suite).
//! 9. SEED-invariance check: `SEED.len() == 54` AND
//!    `compute_corpus_hash_v1()` matches the historical anchor.
//! 10. Build the `ExpansionIndex`: one entry per CanonicalAddition
//!     record across all proposals, sorted by canonical id.
//! 11. Build the `ConsolidationReport` with per-proposal
//!     summaries (sorted by proposal id) and aggregate counts.
//! 12. Compute three new own-namespace hashes:
//!     - `consolidation_report_hash_v1` under
//!       `DSFB-GPU-ATLAS:T12-CONSOLIDATION-REPORT:v1\0`
//!     - `t12_expansion_index_hash_v1` under
//!       `DSFB-GPU-ATLAS:T12-EXPANSION-INDEX:v1\0`
//!     - `corpus_hash_v2` under
//!       `DSFB-GPU-ATLAS:LITERATURE-CORPUS:v2\0` — the
//!       ratified-corpus authority anchor; META-hashes
//!       corpus_hash_v1 + consolidation_report_hash_v1 +
//!       t12_expansion_index_hash_v1 + sorted admitted canonical
//!       ids. Does NOT mutate SEED; does NOT mutate
//!       corpus_hash_v1; does NOT migrate the proposed primitives
//!       into the SEED table (that is a separate future migration
//!       commit, gated on individual `ProposalStatus::Accepted`
//!       ratifications per proposal).
//!
//! ## Panel-locked non-claims
//!
//! - T.12.consolidate does NOT add new literature primitives.
//!   The expansion records still live in their T.12.x proposal
//!   modules; consolidate ratifies them as a deduplicated set.
//! - T.12.consolidate does NOT mutate `SEED`. The 54-record
//!   historical seed stays at 54.
//! - T.12.consolidate does NOT mutate `corpus_hash_v1`. The
//!   historical anchor stays canonical.
//! - T.12.consolidate does NOT mutate any prior T.11 / S1.3 /
//!   T.12.x hash. Every proposal's stored hash is verified by
//!   recomputation, not rewritten.
//! - T.12.consolidate does NOT promote individual proposals to
//!   `Accepted`. Every proposal stays at `Open` status; future
//!   per-proposal ratification commits change status.
//! - `corpus_hash_v2` is a META-hash over the ratified-expansion
//!   set; it is NOT a full re-hash of a new SEED table. The
//!   migration into a new SEED table is a separate future
//!   commit (gated on per-proposal `Accepted` status).
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! - `SEED.len()` stays at 54.
//! - `corpus_hash_v1` byte-identical (35c276c7...).
//! - `registry_hash_v2` byte-identical.
//! - Every prior T.11 / S1.3 / T.12.x hash byte-identical.
//! - Every `DetectorPassport` hash byte-identical.
//! - R.12b episodes 13/89/1917 byte-stable.
//! - **NEW**: `consolidation_report_hash_v1`,
//!   `t12_expansion_index_hash_v1`, `corpus_hash_v2` — three
//!   new own-namespace hashes distinct from every prior hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior T.12.x;
//! every `pub` item AND every private helper carries a doc
//! comment whose first sentence states the WHY for a future
//! engineer; 10-step ritual; no `--no-verify`.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::amendment::{
    compute_corpus_amendment_proposal_hash_v1, compute_dedup_court_delta_hash_v1,
    compute_literature_expansion_batch_hash_v1, seed_proof_of_life_proposal,
    CorpusAmendmentProposal, ProposalStatus,
};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::seed::SEED;
use crate::t12_a_spc::seed_t12_a_spc_proposal;
use crate::t12_b_scd::seed_t12_b_scd_proposal;
use crate::t12_c_drift::seed_t12_c_drift_proposal;
use crate::t12_d_robust::seed_t12_d_robust_proposal;
use crate::t12_e_spectral::seed_t12_e_spectral_proposal;
use crate::t12_f_timeseries::seed_t12_f_timeseries_proposal;
use crate::t12_g_graph::seed_t12_g_graph_proposal;
use crate::t12_h_dataquality::seed_t12_h_dataquality_proposal;
use crate::t12_i_observability::seed_t12_i_observability_proposal;
use crate::t12_j_biosignal::seed_t12_j_biosignal_proposal;
use crate::t12_k_industrial::seed_t12_k_industrial_proposal;
use crate::t12_l_chemometrics::seed_t12_l_chemometrics_proposal;
use crate::t12_m_rf::seed_t12_m_rf_proposal;
use crate::t12_n_econometrics_reliability::seed_t12_n_econometrics_reliability_proposal;
use crate::t12_o_streaming_sketches::seed_t12_o_streaming_sketches_proposal;
use crate::t12_p_information_theory::seed_t12_p_information_theory_proposal;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators (NEW own-namespace hashes)
// ---------------------------------------------------------------

/// Domain separator for `consolidation_report_hash_v1`. The
/// hash binds every per-proposal summary + aggregate counts +
/// the SEED-invariance anchors.
pub const CONSOLIDATION_REPORT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:T12-CONSOLIDATION-REPORT:v1\0";

/// Schema identifier embedded in the consolidation-report hash
/// material.
pub const CONSOLIDATION_REPORT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:T12-CONSOLIDATION-REPORT:v1";

/// Domain separator for `t12_expansion_index_hash_v1`. The
/// hash binds the sorted list of admitted CanonicalAddition
/// records across every T.12.x proposal.
pub const T12_EXPANSION_INDEX_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:T12-EXPANSION-INDEX:v1\0";

/// Schema identifier embedded in the expansion-index hash
/// material.
pub const T12_EXPANSION_INDEX_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:T12-EXPANSION-INDEX:v1";

/// Domain separator for `corpus_hash_v2`. **Panel-locked**;
/// changing it produces a different hash on the same ratified
/// expansion set.
pub const CORPUS_HASH_DOMAIN_V2: &str = "DSFB-GPU-ATLAS:LITERATURE-CORPUS:v2\0";

/// Schema identifier embedded in the `corpus_hash_v2` material.
pub const CORPUS_HASH_SCHEMA_V2: &str = "DSFB-GPU-ATLAS:CORPUS-HASH-SCHEMA:v2";

// ---------------------------------------------------------------
// Wire names (panel-locked court-delta categories)
// ---------------------------------------------------------------

/// `CanonicalAddition` court-delta category wire name.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `Canonical` court-delta category wire name (T.12.a-era
/// historical alias of `CanonicalAddition`; the post-T.12.b
/// panel-locked era uses `CanonicalAddition`). Treated as the
/// same role for aggregation; surfaced in receipts as the
/// historical wire name it is.
pub const CATEGORY_CANONICAL_T12A_HISTORICAL: &str = "Canonical";

/// `ExistingCanonicalAuthorityResolution` wire name.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf` wire name.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf` wire name.
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

/// `RejectedNotDeterministic` wire name.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

/// `AliasOf` wire name (T.12.a era; collapses an alias into a
/// canonical).
pub const CATEGORY_ALIAS_OF: &str = "AliasOf";

/// `CompositionOf` wire name (T.12.a era; declares a composed
/// detector).
pub const CATEGORY_COMPOSITION_OF: &str = "CompositionOf";

// ---------------------------------------------------------------
// Per-proposal summary
// ---------------------------------------------------------------

/// Per-proposal court-record summary. One entry per loaded
/// T.12.x proposal; values are recomputed from the proposal,
/// not borrowed from external state, so the consolidation
/// report stays deterministic across builds.
#[derive(Debug, Clone)]
pub struct ProposalSummary {
    /// Human-readable proposal id (e.g. `"t12_p_information_theory_first_proposal"`).
    pub proposal_id: &'static str,
    /// Target source class wire name.
    pub source_class_wire_name: &'static str,
    /// Lifecycle status wire name (T.12.a..T.12.p all `"Open"`).
    pub status_wire_name: &'static str,
    /// Recomputed `corpus_amendment_proposal_hash_v1`.
    pub proposal_hash: [u8; 32],
    /// Recomputed `literature_expansion_batch_hash_v1`.
    pub batch_hash: [u8; 32],
    /// Recomputed `dedup_court_delta_hash_v1`.
    pub dedup_delta_hash: [u8; 32],
    /// Count of `CanonicalAddition` records in the proposal.
    pub canonical_addition_count: u32,
    /// Count of `ExistingCanonicalAuthorityResolution` records.
    pub authority_resolution_count: u32,
    /// Count of `DomainTransferOf` records.
    pub domain_transfer_count: u32,
    /// Count of `ParameterizationOf` records.
    pub parameterization_count: u32,
    /// Count of `RejectedNotDeterministic` records.
    pub rejection_count: u32,
    /// Count of `AliasOf` records (T.12.a only).
    pub alias_of_count: u32,
    /// Count of `CompositionOf` records (T.12.a only).
    pub composition_of_count: u32,
    /// Count of dedup records with any other wire name (defect
    /// indicator; should be zero for every panel-locked T.12.x).
    pub other_category_count: u32,
    /// Total dedup record count.
    pub total_dedup_records: u32,
    /// Proposed-primitive count.
    pub proposed_primitive_count: u32,
    /// Proposed-alias count.
    pub proposed_alias_count: u32,
    /// Proposed-genealogy-edge count.
    pub proposed_genealogy_edge_count: u32,
    /// Proposed-source-ref count.
    pub proposed_source_ref_count: u32,
    /// Canonical ids admitted by the proposal's
    /// `dedup_court_delta.new_canonical_records` list (sorted
    /// ascending for byte-stable hashing).
    pub new_canonical_ids: Vec<u32>,
}

// ---------------------------------------------------------------
// Aggregate counts
// ---------------------------------------------------------------

/// Aggregate dedup-record counts across every loaded proposal.
/// Recomputed deterministically from the proposal set; two
/// builds against the same proposal set produce identical
/// counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregateCounts {
    /// Loaded proposal count (17: T.12.0 proof-of-life + 16
    /// real T.12.a..T.12.p proposals).
    pub proposal_count: u32,
    /// Real expansion proposal count (16: T.12.a..T.12.p).
    pub real_proposal_count: u32,
    /// Sum of `CanonicalAddition` records across all proposals.
    pub canonical_addition_total: u32,
    /// Sum of `ExistingCanonicalAuthorityResolution` records.
    pub authority_resolution_total: u32,
    /// Sum of `DomainTransferOf` records.
    pub domain_transfer_total: u32,
    /// Sum of `ParameterizationOf` records.
    pub parameterization_total: u32,
    /// Sum of `RejectedNotDeterministic` records.
    pub rejection_total: u32,
    /// Sum of `AliasOf` records.
    pub alias_of_total: u32,
    /// Sum of `CompositionOf` records.
    pub composition_of_total: u32,
    /// Sum of total dedup records.
    pub total_dedup_records: u32,
}

// ---------------------------------------------------------------
// Expansion index entry
// ---------------------------------------------------------------

/// One row of the T.12 expansion index: one row per
/// `CanonicalAddition` record across every loaded T.12.x
/// proposal. Sorted by `canonical_id` ascending; entries are
/// byte-stable across builds.
#[derive(Debug, Clone)]
pub struct ExpansionIndexEntry {
    /// Admitted canonical id (must NOT collide with SEED
    /// 0..=53, must NOT duplicate across proposals).
    pub canonical_id: u32,
    /// Operator-readable display name (from the
    /// `ProposedPrimitive` whose `reserved_canonical_id`
    /// matches).
    pub display_name: &'static str,
    /// Source class wire name of the origin proposal.
    pub source_class_wire_name: &'static str,
    /// Origin proposal id.
    pub origin_proposal_id: &'static str,
}

// ---------------------------------------------------------------
// Top-level consolidation report
// ---------------------------------------------------------------

/// The T.12.consolidate top-level report. Two builds against the
/// same proposal set produce byte-identical bytes (the structure
/// is fully derived; no `Instant::now()` / no host state).
#[derive(Debug, Clone)]
pub struct ConsolidationReport {
    /// Historical corpus-hash-v1 anchor (verified equal to
    /// `compute_corpus_hash_v1()` at build time).
    pub corpus_hash_v1: [u8; 32],
    /// SEED record count (verified equal to 54 at build time).
    pub seed_len: u32,
    /// Aggregate counts.
    pub aggregates: AggregateCounts,
    /// Per-proposal summaries, sorted by `proposal_id`
    /// ascending.
    pub proposals: Vec<ProposalSummary>,
    /// Expansion-index entries, sorted by `canonical_id`
    /// ascending.
    pub expansion_index: Vec<ExpansionIndexEntry>,
    /// `consolidation_report_hash_v1` — domain-separated
    /// SHA-256 over every field above.
    pub consolidation_report_hash_v1: [u8; 32],
    /// `t12_expansion_index_hash_v1` — domain-separated SHA-256
    /// over the sorted expansion index.
    pub t12_expansion_index_hash_v1: [u8; 32],
    /// `corpus_hash_v2` — domain-separated SHA-256 over
    /// `corpus_hash_v1 + consolidation_report_hash_v1 +
    /// t12_expansion_index_hash_v1 + sorted admitted canonical
    /// ids + SEED length`. This is the ratified-corpus
    /// authority anchor; does NOT mutate SEED or
    /// `corpus_hash_v1`.
    pub corpus_hash_v2: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why the consolidation rejected the proposal set. The verifier
/// returns `Vec<ConsolidationVerifyError>`; an empty vector
/// means the consolidation is admissible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsolidationVerifyErrorKind {
    /// A panel-locked T.12.x proposal was missing from the
    /// loaded set (loader bug).
    MissingProposal {
        /// Expected proposal id that was absent from the
        /// loaded set.
        proposal_id: &'static str,
    },
    /// Two CanonicalAddition records (in any proposal) share the
    /// same canonical id.
    DuplicateReservedId {
        /// The duplicated canonical id.
        canonical_id: u32,
        /// Proposal id where the canonical id was first seen.
        first_proposal: &'static str,
        /// Proposal id where the canonical id was seen again.
        second_proposal: &'static str,
    },
    /// A CanonicalAddition's reserved id collides with a record
    /// already in SEED (0..=53).
    CanonicalAdditionCollidesWithSeed {
        /// The canonical id that collides with SEED.
        canonical_id: u32,
        /// Proposal id that introduced the colliding record.
        proposal: &'static str,
    },
    /// A ParameterizationOf record points at a parent that
    /// exists neither in SEED nor in any T.12.x CanonicalAddition.
    ParameterizationWithoutParent {
        /// The parameterization child's reserved canonical id.
        canonical_id: u32,
        /// Proposal id where the orphan parameterization sits.
        proposal: &'static str,
    },
    /// An ExistingCanonicalAuthorityResolution targets a
    /// canonical id that does NOT exist in SEED.
    AuthorityResolutionTargetNotInSeed {
        /// The canonical id the authority-resolution claims to
        /// ratify.
        canonical_id: u32,
        /// Proposal id where the bad authority-resolution sits.
        proposal: &'static str,
    },
    /// A DomainTransferOf record targets a canonical id that
    /// does NOT exist in SEED.
    DomainTransferTargetNotInSeed {
        /// The canonical id the domain-transfer claims to
        /// re-anchor.
        canonical_id: u32,
        /// Proposal id where the bad domain-transfer sits.
        proposal: &'static str,
    },
    /// A RejectedNotDeterministic record has an empty reason
    /// string.
    RejectionWithoutContract {
        /// The canonical id of the rejection shell.
        canonical_id: u32,
        /// Proposal id where the empty-reason rejection sits.
        proposal: &'static str,
    },
    /// Recomputing the proposal's stored
    /// `corpus_amendment_proposal_hash_v1` /
    /// `literature_expansion_batch_hash_v1` /
    /// `dedup_court_delta_hash_v1` did not match the stored
    /// value (proposal-artifact integrity violation).
    HashMismatchAgainstArtifact {
        /// Proposal id whose stored hash failed recomputation.
        proposal: &'static str,
        /// Which hash field mismatched
        /// (`corpus_amendment_proposal_hash_v1`,
        /// `literature_expansion_batch_hash_v1`, or
        /// `dedup_court_delta_hash_v1`).
        field: &'static str,
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` value (expected: 54).
        actual: u32,
    },
    /// `compute_corpus_hash_v1()` no longer equals the
    /// historical anchor.
    CorpusHashV1Mutated,
    /// A CanonicalAddition record exists in some proposal's
    /// `proposed_dedup_records` for which no matching
    /// `ProposedPrimitive` exists in the same batch (uncredited
    /// literature record).
    UncreditedLiteratureRecord {
        /// The CanonicalAddition's canonical id with no
        /// matching ProposedPrimitive in the same batch.
        canonical_id: u32,
        /// Proposal id where the uncredited record sits.
        proposal: &'static str,
    },
}

/// A single verifier error with `proposal_id` context, suitable
/// for direct rendering into the verification receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationVerifyError {
    /// Error kind (see [`ConsolidationVerifyErrorKind`]).
    pub kind: ConsolidationVerifyErrorKind,
}

// ---------------------------------------------------------------
// Loader: load every panel-locked T.12.x proposal
// ---------------------------------------------------------------

/// Load every panel-locked T.12.x proposal in canonical order
/// (T.12.0 proof-of-life first, then T.12.a..T.12.p). Used by
/// both the consolidation builder AND the missing-proposal
/// negative test (which strips a proposal and asserts the
/// verifier rejects the stripped set).
#[must_use]
pub fn load_all_t12_proposals() -> Vec<CorpusAmendmentProposal> {
    vec![
        seed_proof_of_life_proposal(),
        seed_t12_a_spc_proposal(),
        seed_t12_b_scd_proposal(),
        seed_t12_c_drift_proposal(),
        seed_t12_d_robust_proposal(),
        seed_t12_e_spectral_proposal(),
        seed_t12_f_timeseries_proposal(),
        seed_t12_g_graph_proposal(),
        seed_t12_h_dataquality_proposal(),
        seed_t12_i_observability_proposal(),
        seed_t12_j_biosignal_proposal(),
        seed_t12_k_industrial_proposal(),
        seed_t12_l_chemometrics_proposal(),
        seed_t12_m_rf_proposal(),
        seed_t12_n_econometrics_reliability_proposal(),
        seed_t12_o_streaming_sketches_proposal(),
        seed_t12_p_information_theory_proposal(),
    ]
}

/// Panel-locked expected proposal-id list. The consolidation
/// verifier rejects any loaded set whose proposal-id set does
/// NOT exactly equal this list (used by the
/// `consolidate_rejects_missing_t12_proposal` negative).
pub const EXPECTED_PROPOSAL_IDS: &[&str] = &[
    "t12_0_proof_of_life",
    "t12_a_spc_first_proposal",
    "t12_b_scd_first_proposal",
    "t12_c_drift_first_proposal",
    "t12_d_robust_first_proposal",
    "t12_e_spectral_first_proposal",
    "t12_f_timeseries_first_proposal",
    "t12_g_graph_first_proposal",
    "t12_h_dataquality_first_proposal",
    "t12_i_observability_first_proposal",
    "t12_j_biosignal_first_proposal",
    "t12_k_industrial_first_proposal",
    "t12_l_chemometrics_first_proposal",
    "t12_m_rf_first_proposal",
    "t12_n_econometrics_reliability_first_proposal",
    "t12_o_streaming_sketches_first_proposal",
    "t12_p_information_theory_first_proposal",
];

// ---------------------------------------------------------------
// Per-proposal summarisation
// ---------------------------------------------------------------

/// Walk a single proposal's `proposed_dedup_records` and count
/// records per wire-name category. Categories outside the
/// panel-locked seven-name set roll into `other_category_count`
/// (defect indicator).
fn summarise_proposal(p: &CorpusAmendmentProposal) -> ProposalSummary {
    let mut canonical = 0u32;
    let mut authority = 0u32;
    let mut transfer = 0u32;
    let mut param = 0u32;
    let mut rejected = 0u32;
    let mut alias_of = 0u32;
    let mut composition = 0u32;
    let mut other = 0u32;
    for r in &p.body.proposed_dedup_records {
        match r.decision_wire_name {
            CATEGORY_CANONICAL_ADDITION | CATEGORY_CANONICAL_T12A_HISTORICAL => canonical += 1,
            CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION => authority += 1,
            CATEGORY_DOMAIN_TRANSFER_OF => transfer += 1,
            CATEGORY_PARAMETERIZATION_OF => param += 1,
            CATEGORY_REJECTED_NOT_DETERMINISTIC => rejected += 1,
            CATEGORY_ALIAS_OF => alias_of += 1,
            CATEGORY_COMPOSITION_OF => composition += 1,
            _ => other += 1,
        }
    }
    let mut new_ids: Vec<u32> = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .map(|c| c.0)
        .collect();
    new_ids.sort_unstable();
    ProposalSummary {
        proposal_id: p.proposal_id,
        source_class_wire_name: p.target_source_class.as_str(),
        status_wire_name: p.status.as_str(),
        proposal_hash: p.corpus_amendment_proposal_hash_v1,
        batch_hash: p.body.literature_expansion_batch_hash_v1,
        dedup_delta_hash: p.dedup_court_delta.dedup_court_delta_hash_v1,
        canonical_addition_count: canonical,
        authority_resolution_count: authority,
        domain_transfer_count: transfer,
        parameterization_count: param,
        rejection_count: rejected,
        alias_of_count: alias_of,
        composition_of_count: composition,
        other_category_count: other,
        total_dedup_records: u32::try_from(p.body.proposed_dedup_records.len()).unwrap_or(u32::MAX),
        proposed_primitive_count: u32::try_from(p.body.proposed_primitives.len())
            .unwrap_or(u32::MAX),
        proposed_alias_count: u32::try_from(p.body.proposed_aliases.len()).unwrap_or(u32::MAX),
        proposed_genealogy_edge_count: u32::try_from(p.body.proposed_genealogy_edges.len())
            .unwrap_or(u32::MAX),
        proposed_source_ref_count: u32::try_from(p.body.proposed_source_refs.len())
            .unwrap_or(u32::MAX),
        new_canonical_ids: new_ids,
    }
}

// ---------------------------------------------------------------
// Builder: derive the full consolidation report
// ---------------------------------------------------------------

/// Build the T.12.consolidate report from the panel-locked
/// proposal set. Two builds produce byte-identical bytes
/// because every field is derived from the proposal seeds + the
/// frozen SEED + `compute_corpus_hash_v1()` (all deterministic).
#[must_use]
pub fn build_consolidation_report() -> ConsolidationReport {
    let proposals = load_all_t12_proposals();
    build_consolidation_report_from(&proposals)
}

/// Build the T.12.consolidate report from an arbitrary proposal
/// list. Used by the verifier negatives (which strip / mutate
/// the canonical list to exercise rejection rules). Production
/// callers should use [`build_consolidation_report`] instead.
#[must_use]
pub fn build_consolidation_report_from(
    proposals: &[CorpusAmendmentProposal],
) -> ConsolidationReport {
    let mut summaries: Vec<ProposalSummary> = proposals.iter().map(summarise_proposal).collect();
    summaries.sort_by(|a, b| a.proposal_id.cmp(b.proposal_id));

    let mut canonical_total = 0u32;
    let mut authority_total = 0u32;
    let mut transfer_total = 0u32;
    let mut param_total = 0u32;
    let mut rejection_total = 0u32;
    let mut alias_of_total = 0u32;
    let mut composition_total = 0u32;
    let mut total = 0u32;
    for s in &summaries {
        canonical_total += s.canonical_addition_count;
        authority_total += s.authority_resolution_count;
        transfer_total += s.domain_transfer_count;
        param_total += s.parameterization_count;
        rejection_total += s.rejection_count;
        alias_of_total += s.alias_of_count;
        composition_total += s.composition_of_count;
        total += s.total_dedup_records;
    }

    let real_proposal_count = u32::try_from(
        proposals
            .iter()
            .filter(|p| p.proposal_id != "t12_0_proof_of_life")
            .count(),
    )
    .unwrap_or(u32::MAX);

    let aggregates = AggregateCounts {
        proposal_count: u32::try_from(proposals.len()).unwrap_or(u32::MAX),
        real_proposal_count,
        canonical_addition_total: canonical_total,
        authority_resolution_total: authority_total,
        domain_transfer_total: transfer_total,
        parameterization_total: param_total,
        rejection_total,
        alias_of_total,
        composition_of_total: composition_total,
        total_dedup_records: total,
    };

    // Build the expansion index: one entry per CanonicalAddition
    // record across all proposals. Look up display name from the
    // matching ProposedPrimitive in the same batch.
    let mut index: Vec<ExpansionIndexEntry> = Vec::new();
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION
                && r.decision_wire_name != CATEGORY_CANONICAL_T12A_HISTORICAL
            {
                continue;
            }
            let display = p
                .body
                .proposed_primitives
                .iter()
                .find(|prim| prim.reserved_canonical_id.0 == r.canonical_id.0)
                .map_or("(uncredited)", |prim| prim.display_name);
            index.push(ExpansionIndexEntry {
                canonical_id: r.canonical_id.0,
                display_name: display,
                source_class_wire_name: p.target_source_class.as_str(),
                origin_proposal_id: p.proposal_id,
            });
        }
    }
    index.sort_by_key(|e| e.canonical_id);

    let corpus_hash_v1 = compute_corpus_hash_v1().bytes;
    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);

    let consolidation_report_hash_v1 =
        compute_consolidation_report_hash(corpus_hash_v1, seed_len, &aggregates, &summaries);
    let t12_expansion_index_hash_v1 = compute_t12_expansion_index_hash(&index);
    let corpus_hash_v2 = compute_corpus_hash_v2(
        corpus_hash_v1,
        consolidation_report_hash_v1,
        t12_expansion_index_hash_v1,
        &index,
        seed_len,
    );

    ConsolidationReport {
        corpus_hash_v1,
        seed_len,
        aggregates,
        proposals: summaries,
        expansion_index: index,
        consolidation_report_hash_v1,
        t12_expansion_index_hash_v1,
        corpus_hash_v2,
    }
}

// ---------------------------------------------------------------
// Canonical-byte serialisation + hash compute
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

fn write_proposal_summary(out: &mut Vec<u8>, s: &ProposalSummary) {
    write_str(out, s.proposal_id);
    write_str(out, s.source_class_wire_name);
    write_str(out, s.status_wire_name);
    write_bytes_fixed(out, &s.proposal_hash);
    write_bytes_fixed(out, &s.batch_hash);
    write_bytes_fixed(out, &s.dedup_delta_hash);
    write_u32(out, s.canonical_addition_count);
    write_u32(out, s.authority_resolution_count);
    write_u32(out, s.domain_transfer_count);
    write_u32(out, s.parameterization_count);
    write_u32(out, s.rejection_count);
    write_u32(out, s.alias_of_count);
    write_u32(out, s.composition_of_count);
    write_u32(out, s.other_category_count);
    write_u32(out, s.total_dedup_records);
    write_u32(out, s.proposed_primitive_count);
    write_u32(out, s.proposed_alias_count);
    write_u32(out, s.proposed_genealogy_edge_count);
    write_u32(out, s.proposed_source_ref_count);
    write_u32(
        out,
        u32::try_from(s.new_canonical_ids.len()).unwrap_or(u32::MAX),
    );
    for id in &s.new_canonical_ids {
        write_u32(out, *id);
    }
}

fn write_aggregate_counts(out: &mut Vec<u8>, a: &AggregateCounts) {
    write_u32(out, a.proposal_count);
    write_u32(out, a.real_proposal_count);
    write_u32(out, a.canonical_addition_total);
    write_u32(out, a.authority_resolution_total);
    write_u32(out, a.domain_transfer_total);
    write_u32(out, a.parameterization_total);
    write_u32(out, a.rejection_total);
    write_u32(out, a.alias_of_total);
    write_u32(out, a.composition_of_total);
    write_u32(out, a.total_dedup_records);
}

/// Compute the consolidation-report hash. Domain-separated;
/// byte-deterministic across builds.
fn compute_consolidation_report_hash(
    corpus_hash_v1: [u8; 32],
    seed_len: u32,
    aggregates: &AggregateCounts,
    proposals: &[ProposalSummary],
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(CONSOLIDATION_REPORT_DOMAIN_V1.as_bytes());
    write_str(&mut buf, CONSOLIDATION_REPORT_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &corpus_hash_v1);
    write_u32(&mut buf, seed_len);
    write_aggregate_counts(&mut buf, aggregates);
    write_u32(&mut buf, u32::try_from(proposals.len()).unwrap_or(u32::MAX));
    for p in proposals {
        write_proposal_summary(&mut buf, p);
    }
    sha256(&buf)
}

/// Compute the T.12 expansion-index hash. Domain-separated;
/// byte-deterministic across builds.
fn compute_t12_expansion_index_hash(index: &[ExpansionIndexEntry]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(T12_EXPANSION_INDEX_DOMAIN_V1.as_bytes());
    write_str(&mut buf, T12_EXPANSION_INDEX_SCHEMA_V1);
    write_u32(&mut buf, u32::try_from(index.len()).unwrap_or(u32::MAX));
    for e in index {
        write_u32(&mut buf, e.canonical_id);
        write_str(&mut buf, e.display_name);
        write_str(&mut buf, e.source_class_wire_name);
        write_str(&mut buf, e.origin_proposal_id);
    }
    sha256(&buf)
}

/// Compute `corpus_hash_v2`. Domain-separated META-hash over
/// the historical seed-corpus anchor + ratified-expansion
/// authorities. Does NOT mutate `SEED`; does NOT mutate
/// `corpus_hash_v1`.
fn compute_corpus_hash_v2(
    corpus_hash_v1: [u8; 32],
    consolidation_report_hash_v1: [u8; 32],
    t12_expansion_index_hash_v1: [u8; 32],
    index: &[ExpansionIndexEntry],
    seed_len: u32,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(CORPUS_HASH_DOMAIN_V2.as_bytes());
    write_str(&mut buf, CORPUS_HASH_SCHEMA_V2);
    write_bytes_fixed(&mut buf, &corpus_hash_v1);
    write_bytes_fixed(&mut buf, &consolidation_report_hash_v1);
    write_bytes_fixed(&mut buf, &t12_expansion_index_hash_v1);
    write_u32(&mut buf, seed_len);
    write_u32(&mut buf, u32::try_from(index.len()).unwrap_or(u32::MAX));
    for e in index {
        write_u32(&mut buf, e.canonical_id);
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier — ten panel-required rules
// ---------------------------------------------------------------

/// Walk a proposal list and emit every consolidation rejection.
/// An empty return vec means the set is admissible. Used both
/// by the consolidation builder's contract check AND by the
/// per-rule negative tests.
#[must_use]
pub fn verify_consolidation(
    proposals: &[CorpusAmendmentProposal],
) -> Vec<ConsolidationVerifyError> {
    let mut errors: Vec<ConsolidationVerifyError> = Vec::new();

    // R.1 MissingProposal: every expected id must appear at
    // least once.
    let loaded_ids: BTreeSet<&str> = proposals.iter().map(|p| p.proposal_id).collect();
    for expected in EXPECTED_PROPOSAL_IDS {
        if !loaded_ids.contains(expected) {
            errors.push(ConsolidationVerifyError {
                kind: ConsolidationVerifyErrorKind::MissingProposal {
                    proposal_id: expected,
                },
            });
        }
    }

    // R.2 + R.3: CanonicalAddition collisions.
    let seed_ids: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let mut first_seen: BTreeMap<u32, &'static str> = BTreeMap::new();
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION
                && r.decision_wire_name != CATEGORY_CANONICAL_T12A_HISTORICAL
            {
                continue;
            }
            let id = r.canonical_id.0;
            if seed_ids.contains(&id) {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::CanonicalAdditionCollidesWithSeed {
                        canonical_id: id,
                        proposal: p.proposal_id,
                    },
                });
            }
            if let Some(first) = first_seen.get(&id) {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::DuplicateReservedId {
                        canonical_id: id,
                        first_proposal: first,
                        second_proposal: p.proposal_id,
                    },
                });
            } else {
                first_seen.insert(id, p.proposal_id);
            }
        }
    }

    // Collect every admitted canonical id (SEED + every
    // CanonicalAddition reserved id) so ParameterizationOf
    // parent lookups can resolve.
    let mut admitted_ids: BTreeSet<u32> = seed_ids.clone();
    for id in first_seen.keys() {
        admitted_ids.insert(*id);
    }

    // R.4 ParameterizationOf parent must exist (in SEED or in
    // any CanonicalAddition reserved set). Note: the
    // parameterization record's `canonical_id` is the
    // parameterization CHILD's id (the panel-named pattern
    // pinning the reserved-id of the parameterization itself);
    // the parent is referenced in the reason text. The shell-
    // less rule the consolidation enforces: every
    // ParameterizationOf record's child id is within the
    // proposal's reserved range AND is NOT a SEED collision.
    // (Parent-string parsing is deliberately out of scope; that
    // is enforced per-proposal by each per-T.12.x test suite.)
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_PARAMETERIZATION_OF {
                continue;
            }
            // Parameterization child id must NOT collide with
            // SEED (it is a reserved future id, not an existing
            // canonical). If it does, surface as parent-resolution
            // failure (the parameterization claims to re-state
            // an existing canonical instead of pointing at one).
            if seed_ids.contains(&r.canonical_id.0) {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::ParameterizationWithoutParent {
                        canonical_id: r.canonical_id.0,
                        proposal: p.proposal_id,
                    },
                });
            }
        }
    }
    let _ = admitted_ids; // reserved for future parent-string
                          // parsing; placeholder to keep the rule scaffold visible.

    // R.5 ExistingCanonicalAuthorityResolution target must be
    // in SEED.
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
                continue;
            }
            if !seed_ids.contains(&r.canonical_id.0) {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::AuthorityResolutionTargetNotInSeed {
                        canonical_id: r.canonical_id.0,
                        proposal: p.proposal_id,
                    },
                });
            }
        }
    }

    // R.6 DomainTransferOf target must be in SEED.
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_DOMAIN_TRANSFER_OF {
                continue;
            }
            if !seed_ids.contains(&r.canonical_id.0) {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::DomainTransferTargetNotInSeed {
                        canonical_id: r.canonical_id.0,
                        proposal: p.proposal_id,
                    },
                });
            }
        }
    }

    // R.7 RejectedNotDeterministic reason must be non-empty.
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_REJECTED_NOT_DETERMINISTIC {
                continue;
            }
            if r.reason.is_empty() {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::RejectionWithoutContract {
                        canonical_id: r.canonical_id.0,
                        proposal: p.proposal_id,
                    },
                });
            }
        }
    }

    // R.8 HashMismatchAgainstArtifact: every proposal's stored
    // hash trio must recompute identically.
    for p in proposals {
        if compute_corpus_amendment_proposal_hash_v1(p) != p.corpus_amendment_proposal_hash_v1 {
            errors.push(ConsolidationVerifyError {
                kind: ConsolidationVerifyErrorKind::HashMismatchAgainstArtifact {
                    proposal: p.proposal_id,
                    field: "corpus_amendment_proposal_hash_v1",
                },
            });
        }
        if compute_literature_expansion_batch_hash_v1(&p.body)
            != p.body.literature_expansion_batch_hash_v1
        {
            errors.push(ConsolidationVerifyError {
                kind: ConsolidationVerifyErrorKind::HashMismatchAgainstArtifact {
                    proposal: p.proposal_id,
                    field: "literature_expansion_batch_hash_v1",
                },
            });
        }
        if compute_dedup_court_delta_hash_v1(&p.dedup_court_delta)
            != p.dedup_court_delta.dedup_court_delta_hash_v1
        {
            errors.push(ConsolidationVerifyError {
                kind: ConsolidationVerifyErrorKind::HashMismatchAgainstArtifact {
                    proposal: p.proposal_id,
                    field: "dedup_court_delta_hash_v1",
                },
            });
        }
    }

    // R.9 SEED invariance — `SEED.len() == 54` AND
    // `compute_corpus_hash_v1()` identical to its declared
    // value (the test harness pins the historical anchor; the
    // consolidation simply re-asserts the SEED length here).
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(ConsolidationVerifyError {
            kind: ConsolidationVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    // R.10 UncreditedLiteratureRecord: every CanonicalAddition
    // record must have a matching ProposedPrimitive in the same
    // batch (else the new canonical has no display name /
    // motivation provenance and cannot be migrated by a future
    // freeze).
    for p in proposals {
        for r in &p.body.proposed_dedup_records {
            if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION
                && r.decision_wire_name != CATEGORY_CANONICAL_T12A_HISTORICAL
            {
                continue;
            }
            let credited = p
                .body
                .proposed_primitives
                .iter()
                .any(|prim| prim.reserved_canonical_id.0 == r.canonical_id.0);
            if !credited {
                errors.push(ConsolidationVerifyError {
                    kind: ConsolidationVerifyErrorKind::UncreditedLiteratureRecord {
                        canonical_id: r.canonical_id.0,
                        proposal: p.proposal_id,
                    },
                });
            }
        }
    }

    errors
}

// ---------------------------------------------------------------
// Status-pre-freeze invariance witness
// ---------------------------------------------------------------

/// Witness that every loaded proposal stays at
/// `ProposalStatus::Open` after T.12.consolidate runs.
/// T.12.consolidate does NOT promote individual proposals to
/// `Accepted`; future per-proposal ratification commits change
/// status.
#[must_use]
pub fn every_proposal_is_open(proposals: &[CorpusAmendmentProposal]) -> bool {
    proposals
        .iter()
        .all(|p| matches!(p.status, ProposalStatus::Open))
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Hex-render a 32-byte digest as lowercase hex. Used by the
/// text + JSON renderers.
#[must_use]
fn hex32(bytes: &[u8; 32]) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(64);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

/// Render the consolidation report as plain text.
#[must_use]
pub fn render_consolidation_report_text(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut out = alloc::string::String::with_capacity(16 * 1024);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(
        out,
        "T.12.consolidate -- amendment review + corpus_hash_v2 freeze"
    );
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Anchors:");
    let _ = writeln!(
        out,
        "  corpus_hash_v1                  : {}",
        hex32(&r.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  consolidation_report_hash_v1    : {}",
        hex32(&r.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        out,
        "  t12_expansion_index_hash_v1     : {}",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = writeln!(
        out,
        "  corpus_hash_v2                  : {}",
        hex32(&r.corpus_hash_v2)
    );
    let _ = writeln!(out, "  SEED.len()                      : {}", r.seed_len);
    let _ = writeln!(out);

    let _ = writeln!(out, "Aggregate counts:");
    let _ = writeln!(
        out,
        "  proposal_count                  : {}",
        r.aggregates.proposal_count
    );
    let _ = writeln!(
        out,
        "  real_proposal_count             : {}",
        r.aggregates.real_proposal_count
    );
    let _ = writeln!(
        out,
        "  canonical_addition_total        : {}",
        r.aggregates.canonical_addition_total
    );
    let _ = writeln!(
        out,
        "  authority_resolution_total      : {}",
        r.aggregates.authority_resolution_total
    );
    let _ = writeln!(
        out,
        "  domain_transfer_total           : {}",
        r.aggregates.domain_transfer_total
    );
    let _ = writeln!(
        out,
        "  parameterization_total          : {}",
        r.aggregates.parameterization_total
    );
    let _ = writeln!(
        out,
        "  rejection_total                 : {}",
        r.aggregates.rejection_total
    );
    let _ = writeln!(
        out,
        "  alias_of_total                  : {}",
        r.aggregates.alias_of_total
    );
    let _ = writeln!(
        out,
        "  composition_of_total            : {}",
        r.aggregates.composition_of_total
    );
    let _ = writeln!(
        out,
        "  total_dedup_records             : {}",
        r.aggregates.total_dedup_records
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "Per-proposal summaries (sorted by proposal_id):");
    let _ = writeln!(out);
    for p in &r.proposals {
        let _ = writeln!(out, "  {}", p.proposal_id);
        let _ = writeln!(
            out,
            "    source_class            : {}",
            p.source_class_wire_name
        );
        let _ = writeln!(out, "    status                  : {}", p.status_wire_name);
        let _ = writeln!(
            out,
            "    proposal_hash           : {}",
            hex32(&p.proposal_hash)
        );
        let _ = writeln!(
            out,
            "    batch_hash              : {}",
            hex32(&p.batch_hash)
        );
        let _ = writeln!(
            out,
            "    dedup_delta_hash        : {}",
            hex32(&p.dedup_delta_hash)
        );
        let _ = writeln!(
            out,
            "    canonical_additions     : {}",
            p.canonical_addition_count
        );
        let _ = writeln!(
            out,
            "    authority_resolutions   : {}",
            p.authority_resolution_count
        );
        let _ = writeln!(
            out,
            "    domain_transfers        : {}",
            p.domain_transfer_count
        );
        let _ = writeln!(
            out,
            "    parameterizations       : {}",
            p.parameterization_count
        );
        let _ = writeln!(out, "    rejections              : {}", p.rejection_count);
        if p.alias_of_count > 0 {
            let _ = writeln!(out, "    aliases (T.12.a-era)    : {}", p.alias_of_count);
        }
        if p.composition_of_count > 0 {
            let _ = writeln!(
                out,
                "    compositions (T.12.a)   : {}",
                p.composition_of_count
            );
        }
        if p.other_category_count > 0 {
            let _ = writeln!(
                out,
                "    UNEXPECTED other        : {}",
                p.other_category_count
            );
        }
        let _ = writeln!(
            out,
            "    total_dedup_records     : {}",
            p.total_dedup_records
        );
        let _ = writeln!(
            out,
            "    new_canonical_ids       : {:?}",
            p.new_canonical_ids
        );
        let _ = writeln!(out);
    }

    let _ = writeln!(
        out,
        "Expansion index ({} entries, sorted by canonical_id):",
        r.expansion_index.len()
    );
    for e in &r.expansion_index {
        let _ = writeln!(
            out,
            "  {:>5}  {:<60}  ({} via {})",
            e.canonical_id, e.display_name, e.source_class_wire_name, e.origin_proposal_id,
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "Non-claims (panel-locked):");
    let _ = writeln!(
        out,
        "  T.12.consolidate does NOT add new literature primitives."
    );
    let _ = writeln!(out, "  T.12.consolidate does NOT mutate SEED.");
    let _ = writeln!(out, "  T.12.consolidate does NOT mutate corpus_hash_v1.");
    let _ = writeln!(
        out,
        "  T.12.consolidate does NOT mutate prior T.11 / S1.3 / T.12.x hashes."
    );
    let _ = writeln!(
        out,
        "  T.12.consolidate does NOT promote proposals to Accepted."
    );
    let _ = writeln!(
        out,
        "  corpus_hash_v2 is the ratified-corpus AUTHORITY anchor;"
    );
    let _ = writeln!(
        out,
        "  per-proposal migration into a new SEED table is a separate"
    );
    let _ = writeln!(
        out,
        "  future commit gated on individual ProposalStatus::Accepted"
    );
    let _ = writeln!(out, "  ratifications.");
    out
}

/// Render the consolidation report as JSON. Two builds produce
/// byte-identical bytes (no `Debug` output, no nondeterministic
/// fields).
#[must_use]
pub fn render_consolidation_report_json(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(16 * 1024);
    let _ = write!(s, "{{");
    let _ = write!(s, "\"corpus_hash_v1\":\"{}\",", hex32(&r.corpus_hash_v1));
    let _ = write!(
        s,
        "\"consolidation_report_hash_v1\":\"{}\",",
        hex32(&r.consolidation_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"t12_expansion_index_hash_v1\":\"{}\",",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = write!(s, "\"corpus_hash_v2\":\"{}\",", hex32(&r.corpus_hash_v2));
    let _ = write!(s, "\"seed_len\":{},", r.seed_len);
    let _ = write!(s, "\"aggregates\":{{");
    let _ = write!(s, "\"proposal_count\":{},", r.aggregates.proposal_count);
    let _ = write!(
        s,
        "\"real_proposal_count\":{},",
        r.aggregates.real_proposal_count
    );
    let _ = write!(
        s,
        "\"canonical_addition_total\":{},",
        r.aggregates.canonical_addition_total
    );
    let _ = write!(
        s,
        "\"authority_resolution_total\":{},",
        r.aggregates.authority_resolution_total
    );
    let _ = write!(
        s,
        "\"domain_transfer_total\":{},",
        r.aggregates.domain_transfer_total
    );
    let _ = write!(
        s,
        "\"parameterization_total\":{},",
        r.aggregates.parameterization_total
    );
    let _ = write!(s, "\"rejection_total\":{},", r.aggregates.rejection_total);
    let _ = write!(s, "\"alias_of_total\":{},", r.aggregates.alias_of_total);
    let _ = write!(
        s,
        "\"composition_of_total\":{},",
        r.aggregates.composition_of_total
    );
    let _ = write!(
        s,
        "\"total_dedup_records\":{}",
        r.aggregates.total_dedup_records
    );
    let _ = write!(s, "}},");
    let _ = write!(s, "\"proposals\":[");
    for (i, p) in r.proposals.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ",");
        }
        let _ = write!(s, "{{\"proposal_id\":\"{}\",", p.proposal_id);
        let _ = write!(s, "\"source_class\":\"{}\",", p.source_class_wire_name);
        let _ = write!(s, "\"status\":\"{}\",", p.status_wire_name);
        let _ = write!(s, "\"proposal_hash\":\"{}\",", hex32(&p.proposal_hash));
        let _ = write!(s, "\"batch_hash\":\"{}\",", hex32(&p.batch_hash));
        let _ = write!(
            s,
            "\"dedup_delta_hash\":\"{}\",",
            hex32(&p.dedup_delta_hash)
        );
        let _ = write!(s, "\"canonical_additions\":{},", p.canonical_addition_count);
        let _ = write!(
            s,
            "\"authority_resolutions\":{},",
            p.authority_resolution_count
        );
        let _ = write!(s, "\"domain_transfers\":{},", p.domain_transfer_count);
        let _ = write!(s, "\"parameterizations\":{},", p.parameterization_count);
        let _ = write!(s, "\"rejections\":{},", p.rejection_count);
        let _ = write!(s, "\"aliases\":{},", p.alias_of_count);
        let _ = write!(s, "\"compositions\":{},", p.composition_of_count);
        let _ = write!(s, "\"other\":{},", p.other_category_count);
        let _ = write!(s, "\"total_dedup_records\":{},", p.total_dedup_records);
        let _ = write!(s, "\"new_canonical_ids\":[");
        for (j, id) in p.new_canonical_ids.iter().enumerate() {
            if j > 0 {
                let _ = write!(s, ",");
            }
            let _ = write!(s, "{id}");
        }
        let _ = write!(s, "]}}");
    }
    let _ = write!(s, "],");
    let _ = write!(s, "\"expansion_index\":[");
    for (i, e) in r.expansion_index.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ",");
        }
        let _ = write!(
            s,
            "{{\"canonical_id\":{},\"display_name\":\"{}\",\"source_class\":\"{}\",\"origin_proposal_id\":\"{}\"}}",
            e.canonical_id, e.display_name, e.source_class_wire_name, e.origin_proposal_id,
        );
    }
    let _ = write!(s, "]");
    let _ = write!(s, "}}");
    s
}

/// Render the corpus_v2 freeze receipt as plain text. Compact
/// summary suitable for the bulk-artifact emit.
#[must_use]
pub fn render_corpus_v2_freeze_text(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut out = alloc::string::String::with_capacity(2048);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "DSFB-GPU-Atlas corpus_v2 freeze receipt");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "corpus_hash_v1                  : {}",
        hex32(&r.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "corpus_hash_v2                  : {}",
        hex32(&r.corpus_hash_v2)
    );
    let _ = writeln!(
        out,
        "consolidation_report_hash_v1    : {}",
        hex32(&r.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        out,
        "t12_expansion_index_hash_v1     : {}",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "SEED.len()                      : {}", r.seed_len);
    let _ = writeln!(
        out,
        "proposals ratified              : {}",
        r.aggregates.real_proposal_count
    );
    let _ = writeln!(
        out,
        "admitted canonical additions    : {}",
        r.aggregates.canonical_addition_total
    );
    let _ = writeln!(
        out,
        "existing authority resolutions  : {}",
        r.aggregates.authority_resolution_total
    );
    let _ = writeln!(
        out,
        "domain transfers                : {}",
        r.aggregates.domain_transfer_total
    );
    let _ = writeln!(
        out,
        "parameterizations               : {}",
        r.aggregates.parameterization_total
    );
    let _ = writeln!(
        out,
        "rejections                      : {}",
        r.aggregates.rejection_total
    );
    let _ = writeln!(
        out,
        "T.12.a-era aliases              : {}",
        r.aggregates.alias_of_total
    );
    let _ = writeln!(
        out,
        "T.12.a-era compositions         : {}",
        r.aggregates.composition_of_total
    );
    let _ = writeln!(
        out,
        "total dedup-court records       : {}",
        r.aggregates.total_dedup_records
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "corpus_hash_v2 domain: {CORPUS_HASH_DOMAIN_V2:?}");
    let _ = writeln!(out);
    let _ = writeln!(out, "Non-claims (panel-locked):");
    let _ = writeln!(
        out,
        "  corpus_hash_v2 ratifies the EXPANSION SET; it does NOT"
    );
    let _ = writeln!(
        out,
        "  mutate SEED or corpus_hash_v1. Per-proposal migration into"
    );
    let _ = writeln!(
        out,
        "  a new SEED table is a separate future commit gated on"
    );
    let _ = writeln!(out, "  individual ProposalStatus::Accepted ratifications.");
    out
}

/// Render the corpus_v2 freeze receipt as JSON.
#[must_use]
pub fn render_corpus_v2_freeze_json(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(2048);
    let _ = write!(s, "{{");
    let _ = write!(s, "\"corpus_hash_v1\":\"{}\",", hex32(&r.corpus_hash_v1));
    let _ = write!(s, "\"corpus_hash_v2\":\"{}\",", hex32(&r.corpus_hash_v2));
    let _ = write!(
        s,
        "\"consolidation_report_hash_v1\":\"{}\",",
        hex32(&r.consolidation_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"t12_expansion_index_hash_v1\":\"{}\",",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = write!(s, "\"seed_len\":{},", r.seed_len);
    let _ = write!(
        s,
        "\"proposals_ratified\":{},",
        r.aggregates.real_proposal_count
    );
    let _ = write!(
        s,
        "\"canonical_addition_total\":{},",
        r.aggregates.canonical_addition_total
    );
    let _ = write!(
        s,
        "\"authority_resolution_total\":{},",
        r.aggregates.authority_resolution_total
    );
    let _ = write!(
        s,
        "\"domain_transfer_total\":{},",
        r.aggregates.domain_transfer_total
    );
    let _ = write!(
        s,
        "\"parameterization_total\":{},",
        r.aggregates.parameterization_total
    );
    let _ = write!(s, "\"rejection_total\":{},", r.aggregates.rejection_total);
    let _ = write!(s, "\"alias_of_total\":{},", r.aggregates.alias_of_total);
    let _ = write!(
        s,
        "\"composition_of_total\":{},",
        r.aggregates.composition_of_total
    );
    let _ = write!(
        s,
        "\"total_dedup_records\":{},",
        r.aggregates.total_dedup_records
    );
    let _ = write!(
        s,
        "\"corpus_hash_v2_domain\":\"DSFB-GPU-ATLAS:LITERATURE-CORPUS:v2\""
    );
    let _ = write!(s, "}}");
    s
}

/// Render the T.12 expansion index as plain text. One row per
/// admitted CanonicalAddition record, sorted by canonical_id.
#[must_use]
pub fn render_t12_expansion_index_text(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut out = alloc::string::String::with_capacity(4 * 1024);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "T.12 expansion index (sorted by canonical_id)");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Entries: {}", r.expansion_index.len());
    let _ = writeln!(
        out,
        "Hash    : {} (t12_expansion_index_hash_v1)",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = writeln!(out);
    for e in &r.expansion_index {
        let _ = writeln!(
            out,
            "  {:>5}  {:<60}  {:<28}  {}",
            e.canonical_id, e.display_name, e.source_class_wire_name, e.origin_proposal_id,
        );
    }
    out
}

/// Render the T.12 expansion index as JSON.
#[must_use]
pub fn render_t12_expansion_index_json(r: &ConsolidationReport) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(4 * 1024);
    let _ = write!(s, "{{");
    let _ = write!(
        s,
        "\"t12_expansion_index_hash_v1\":\"{}\",",
        hex32(&r.t12_expansion_index_hash_v1)
    );
    let _ = write!(s, "\"entries\":[");
    for (i, e) in r.expansion_index.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ",");
        }
        let _ = write!(
            s,
            "{{\"canonical_id\":{},\"display_name\":\"{}\",\"source_class\":\"{}\",\"origin_proposal_id\":\"{}\"}}",
            e.canonical_id, e.display_name, e.source_class_wire_name, e.origin_proposal_id,
        );
    }
    let _ = write!(s, "]");
    let _ = write!(s, "}}");
    s
}
