//! T.12.a — Statistical Process Control: the first real
//! literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.a files the first real corpus expansion proposal:
//! > Statistical Process Control. It proposes canonical SPC
//! > primitives, collapses known aliases and rule-set
//! > compositions, emits a dedup-court delta, and proves the
//! > literature corpus can grow without mutating the frozen
//! > T.10 corpus hash.**
//!
//! T.12.a's job is **the first real proposal** — not the actual
//! expansion. The proposal carries `status = Open`; a future
//! formal freeze campaign produces `corpus_hash_v2` once the
//! court has ratified this proposal alongside enough other
//! T.12.x sub-campaigns.
//!
//! **Scope (panel-locked)**:
//!
//! * **Proposes 2 new canonical SPC primitives** that are NOT
//!   yet in `SEED`: `MEWMA` (Multivariate Exponentially
//!   Weighted Moving Average chart) and `MCUSUM` (Multivariate
//!   CUSUM chart). Reserved canonical IDs **5001** and **5002**
//!   (well above SEED's 54-record range).
//! * **Records dedup-court decisions on 3 already-canonical
//!   SEED records**: PCA SPE / Q residual (id 20) gains two
//!   recorded aliases (Q statistic, Squared Prediction Error);
//!   Hotelling T-squared (id 5) gains one alias ("Hotelling
//!   T-square chart").
//! * **Records dedup-court compositions on 2 already-canonical
//!   SEED records**: Western Electric SPC rules (id 16) marked
//!   `CompositionOf(Shewhart)`; Nelson SPC rules (id 17) marked
//!   `CompositionOf(Shewhart, Western Electric)`. These are
//!   *proposed* decisions — they do not retroactively mutate
//!   SEED.
//! * **Proposes 4 genealogy edges**: `MEWMA derived_from EWMA`,
//!   `MCUSUM derived_from CUSUM`, `Western Electric
//!   derived_from Shewhart`, `Nelson derived_from Western
//!   Electric`.
//! * **Proposes 4 source-ref records** for the new primitives
//!   and the composition decisions.
//!
//! **Page-Hinkley note (panel-locked guidance)**: Page-Hinkley
//! is already canonical in `SEED` at id 4. The user's panel
//! verdict cautioned that Page-Hinkley is SPC-adjacent and
//! sequential-change-detection-adjacent. T.12.a takes the
//! "leave final authority to T.12.b" path — Page-Hinkley is
//! NOT touched by this proposal. The summary receipt
//! documents the deferral explicitly so T.12.b's authors see
//! it.
//!
//! **Hash posture (panel-locked, MUST hold)**:
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * `registry_hash_v2`, every T.11/S1.3 hash, every
//!   DetectorPassport hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial
//!   `corpus_amendment_proposal_hash_v1` for the SPC proposal
//!   (distinct from T.12.0's empty proof-of-life hash).
//!
//! **Discipline**: same `no-silent-court-logic` as T.12.0;
//! every `pub` item AND every private helper carries WHY
//! commentary; 10-step ritual; no `--no-verify`.

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

/// Reserved canonical id for MEWMA. Chosen well above
/// `SEED.len() == 54` so the T.12.0 collision verifier does not
/// fire. 5001 is also clearly distinct from the 1xxx alias id
/// range in `claims.rs` so a human reading the proposal can tell
/// canonical-vs-alias intent at a glance.
pub const MEWMA_RESERVED_CANONICAL_ID: u32 = 5001;

/// Reserved canonical id for MCUSUM.
pub const MCUSUM_RESERVED_CANONICAL_ID: u32 = 5002;

/// Reserved alias id for "Q statistic" → PCA_SPE_Q_RESIDUAL.
/// Starts at 5101 (cleanly above the existing 1010..1012
/// claims-table ids and above the canonical 5xxx range we
/// reserved for new T.12.a canonicals).
pub const Q_STATISTIC_ALIAS_ID: u32 = 5101;

/// Reserved alias id for "Squared Prediction Error" →
/// PCA_SPE_Q_RESIDUAL.
pub const SPE_ALIAS_ID: u32 = 5102;

/// Reserved alias id for "Hotelling T-square chart" →
/// HOTELLING_T2.
pub const HOTELLING_TSQUARE_ALIAS_ID: u32 = 5103;

// Canonical ids referenced by the SPC dedup-court delta — these
// are EXISTING SEED records the proposal records decisions
// against (without mutating them). Pinned here as named
// constants so reviewers can audit the references at a glance.

/// Shewhart control chart — already canonical in SEED.
pub const SHEWHART_SEED_ID: u32 = 1;

/// EWMA control chart — already canonical in SEED.
pub const EWMA_SEED_ID: u32 = 2;

/// CUSUM — already canonical in SEED.
pub const CUSUM_SEED_ID: u32 = 3;

/// Hotelling T-squared — already canonical in SEED.
pub const HOTELLING_T2_SEED_ID: u32 = 5;

/// Western Electric SPC rules — already canonical in SEED.
pub const WESTERN_ELECTRIC_SEED_ID: u32 = 16;

/// Nelson SPC rules — already canonical in SEED.
pub const NELSON_SEED_ID: u32 = 17;

/// PCA SPE / Q residual — already canonical in SEED.
pub const PCA_SPE_Q_SEED_ID: u32 = 20;

// ---------------------------------------------------------------
// Builders for the SPC expansion batch
// ---------------------------------------------------------------

/// Build the SPC `CorpusExpansionBatch` body: 2 new canonical
/// primitives, 3 alias claims, 5 dedup-court records, 4
/// genealogy edges, 4 source refs.
fn build_spc_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_a_spc_first_proposal",
        SourceClass::StatisticalProcessControl,
        spc_proposed_primitives(),
        spc_proposed_aliases(),
        spc_proposed_dedup_records(),
        spc_proposed_genealogy_edges(),
        spc_proposed_source_refs(),
    )
}

/// Two new canonical SPC primitives: MEWMA + MCUSUM. Reserved
/// canonical ids 5001/5002. SEED stays at 54 records — these
/// are PROPOSED, not yet admitted.
fn spc_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MEWMA_RESERVED_CANONICAL_ID),
            display_name: "Multivariate EWMA (MEWMA) chart",
            motivation: "Multivariate extension of EWMA for joint-variable control. \
                 SEED currently has univariate EWMA (canonical 2) but no \
                 multivariate companion; MEWMA closes that gap.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MCUSUM_RESERVED_CANONICAL_ID),
            display_name: "Multivariate CUSUM (MCUSUM) chart",
            motivation: "Multivariate extension of CUSUM (Crosier 1988). SEED currently \
                 has univariate CUSUM (canonical 3) but no multivariate \
                 companion; MCUSUM closes that gap.",
        },
    ]
}

/// Three SPC alias claims:
///   - "Q statistic" → PCA SPE / Q residual (id 20)
///   - "Squared Prediction Error" → PCA SPE / Q residual (id 20)
///   - "Hotelling T-square chart" → Hotelling T-squared (id 5)
fn spc_proposed_aliases() -> Vec<ProposedAliasClaim> {
    vec![
        ProposedAliasClaim {
            reserved_alias_id: DetectorAliasId(Q_STATISTIC_ALIAS_ID),
            collapses_into: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            alias_name: "Q statistic (PCA / PLS literature)",
        },
        ProposedAliasClaim {
            reserved_alias_id: DetectorAliasId(SPE_ALIAS_ID),
            collapses_into: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            alias_name: "Squared Prediction Error (SPE)",
        },
        ProposedAliasClaim {
            reserved_alias_id: DetectorAliasId(HOTELLING_TSQUARE_ALIAS_ID),
            collapses_into: DetectorCanonicalId(HOTELLING_T2_SEED_ID),
            alias_name: "Hotelling T-square chart",
        },
    ]
}

/// Five dedup-court decisions:
///   - 2 Canonical decisions (the new MEWMA / MCUSUM
///     primitives that will be admitted at the next freeze)
///   - 2 CompositionOf decisions (Western Electric → Shewhart;
///     Nelson → Shewhart + Western Electric)
///   - 1 AliasOf summary decision documenting the three alias
///     collapses above (the per-alias records carry the
///     individual `collapses_into` ids; this row is a
///     court-level acknowledgment that the PCA_SPE_Q_RESIDUAL
///     family now has documented aliases under the
///     amendment-court regime)
fn spc_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        ProposedDedupRecord {
            decision_wire_name: "Canonical",
            canonical_id: DetectorCanonicalId(MEWMA_RESERVED_CANONICAL_ID),
            reason: "Multivariate EWMA is structurally distinct from univariate \
                 EWMA: different statistic family (vector statistic + \
                 covariance-aware update law).",
        },
        ProposedDedupRecord {
            decision_wire_name: "Canonical",
            canonical_id: DetectorCanonicalId(MCUSUM_RESERVED_CANONICAL_ID),
            reason: "Multivariate CUSUM (Crosier 1988) is structurally distinct \
                 from univariate CUSUM: different statistic family (joint \
                 likelihood ratio over correlated channels).",
        },
        ProposedDedupRecord {
            decision_wire_name: "CompositionOf",
            canonical_id: DetectorCanonicalId(WESTERN_ELECTRIC_SEED_ID),
            reason: "Western Electric rules are a deterministic rule-set \
                 composition over Shewhart x-bar chart violations. \
                 Court-level reclassification: SEED canonical 16 is admitted \
                 as CompositionOf(Shewhart).",
        },
        ProposedDedupRecord {
            decision_wire_name: "CompositionOf",
            canonical_id: DetectorCanonicalId(NELSON_SEED_ID),
            reason: "Nelson rules extend Western Electric with additional \
                 run-length / trend patterns. Court-level reclassification: \
                 SEED canonical 17 is admitted as CompositionOf(Shewhart, \
                 Western Electric).",
        },
        ProposedDedupRecord {
            decision_wire_name: "AliasOf",
            canonical_id: DetectorCanonicalId(PCA_SPE_Q_SEED_ID),
            reason: "Q statistic / Squared Prediction Error / SPE all collapse \
                 into PCA_SPE_Q_RESIDUAL. The chemometrics and FDD \
                 literatures use these names interchangeably for the same \
                 residual-magnitude statistic over PCA reconstruction error.",
        },
    ]
}

/// Four genealogy edges proposed for the post-freeze graph.
fn spc_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MEWMA_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(EWMA_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MCUSUM_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WESTERN_ELECTRIC_SEED_ID),
            to_canonical_id: DetectorCanonicalId(SHEWHART_SEED_ID),
            edge_kind_wire_name: "Composes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(NELSON_SEED_ID),
            to_canonical_id: DetectorCanonicalId(WESTERN_ELECTRIC_SEED_ID),
            edge_kind_wire_name: "Composes",
        },
    ]
}

/// Four source refs supporting the SPC expansion. We cite the
/// original methodological papers, not implementations.
fn spc_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "lowry_mewma_1992",
            title: "A Multivariate Exponentially Weighted Moving Average Control Chart",
            year: 1992,
            venue: "Technometrics 34(1)",
        },
        ProposedSourceRef {
            citation_key: "crosier_mcusum_1988",
            title: "Multivariate Generalizations of CUSUM Quality Control Schemes",
            year: 1988,
            venue: "Technometrics 30(3)",
        },
        ProposedSourceRef {
            citation_key: "western_electric_1956",
            title: "Statistical Quality Control Handbook",
            year: 1956,
            venue: "Western Electric Company",
        },
        ProposedSourceRef {
            citation_key: "nelson_runs_1984",
            title: "The Shewhart Control Chart — Tests for Special Causes",
            year: 1984,
            venue: "Journal of Quality Technology 16(4)",
        },
    ]
}

/// Build the SPC `DedupCourtDelta`: the per-batch dedup
/// outcome. T.12.a admits the 2 new canonicals plus the 3
/// aliases; no rejections; no deferrals; the 2 compositions
/// are recorded against existing SEED ids via
/// `new_composition_records`.
fn build_spc_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_a_spc_delta",
        // new canonical records: the 2 new primitives.
        vec![
            DetectorCanonicalId(MEWMA_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MCUSUM_RESERVED_CANONICAL_ID),
        ],
        // new alias records: the 3 SPC aliases.
        vec![
            DetectorAliasId(Q_STATISTIC_ALIAS_ID),
            DetectorAliasId(SPE_ALIAS_ID),
            DetectorAliasId(HOTELLING_TSQUARE_ALIAS_ID),
        ],
        // new composition records: re-classify the 2 existing
        // SEED records (Western Electric, Nelson). These do NOT
        // mutate SEED — they document the court's would-be
        // re-classification if the proposal is accepted.
        vec![
            DetectorCanonicalId(WESTERN_ELECTRIC_SEED_ID),
            DetectorCanonicalId(NELSON_SEED_ID),
        ],
        // no rejections in T.12.a.
        Vec::<RejectionRecord>::new(),
        // no deferrals in T.12.a (Page-Hinkley is left to
        // T.12.b's authority; not touched by this proposal).
        Vec::<DetectorAliasId>::new(),
    )
}

// ---------------------------------------------------------------
// Public seed entry point
// ---------------------------------------------------------------

/// Build the T.12.a SPC `CorpusAmendmentProposal`. Two builds
/// against this static seed produce byte-identical bytes.
///
/// **Panel-locked seed posture**:
///
/// * `status = Open` — proposal is filed but the court has not
///   ratified. A future formal freeze campaign produces
///   `corpus_hash_v2` once the court has approved this
///   proposal alongside enough other T.12.x sub-campaigns.
/// * `proposer_role = PanelMember` — filed by the same panel
///   that authored the T.12.0 scaffold.
/// * `created_at_commit = "t12_a_spc"` — placeholder; a future
///   formal freeze campaign cites the actual commit hash.
#[must_use]
pub fn seed_t12_a_spc_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_a_spc_first_proposal",
        "T.12.a files the first real corpus expansion proposal: \
         Statistical Process Control. Proposes 2 new canonical \
         primitives (MEWMA, MCUSUM), 3 alias collapses (Q \
         statistic / SPE → PCA_SPE_Q_RESIDUAL; Hotelling T-square \
         → HOTELLING_T2), and 2 composition reclassifications \
         (Western Electric / Nelson rules → CompositionOf). \
         Does NOT mutate SEED; status = Open pending review.",
        SourceClass::StatisticalProcessControl,
        build_spc_expansion_batch(),
        build_spc_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_a_spc",
    )
}
