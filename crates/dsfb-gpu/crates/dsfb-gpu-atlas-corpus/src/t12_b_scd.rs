//! T.12.b — Sequential Change Detection: the second real
//! literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.b files the Sequential Change Detection amendment
//! > proposal. It adds only defensible new canonical primitives,
//! > resolves cross-class authority for existing Page-Hinkley /
//! > CUSUM / Mann-Kendall (plus the already-canonical Pettitt /
//! > SNHT / MOSUM / Buishand range records), rejects
//! > non-deterministic BOCPD as a canonical detector, and emits a
//! > dedup-court delta without mutating the frozen T.10 corpus.**
//!
//! **Main panel instruction (panel-locked)**: *"Do not chase
//! quantity yet. Prove cross-class dedup authority."*
//!
//! T.12.b's headline is therefore **cross-class dedup authority**,
//! not detector quantity. SPC (T.12.a) proved aliases +
//! compositions; T.12.b proves the court catches existing canonical
//! records that the SCD literature names under different framings
//! AND that it rejects probabilistic-in-origin methods (BOCPD)
//! as canonical without an explicit deterministic-reduction status.
//!
//! ## Why the canonical-addition list is 4, not 8
//!
//! An earlier draft proposed adding Shiryaev-Roberts, GLR, Pettitt,
//! Buishand range, SNHT, MOSUM, Binary segmentation, and
//! PELT-style deterministic as eight new canonicals. A walk of
//! `SEED` showed that **Pettitt (id 34), SNHT (id 35), MOSUM (id
//! 36), and Buishand range (id 37) are already canonical** —
//! they live in the change-point block of `seed.rs`. Promoting
//! them again would have produced the very duplication T.12.b
//! exists to forbid. The four are therefore recorded as
//! `ExistingCanonicalAuthorityResolution` records (panel-locked
//! "keep existing canonical; do not duplicate") rather than as
//! `CanonicalAddition` records, and four panel-required
//! load-bearing negatives are added to the acceptance suite to
//! pin the rule that no SCD-relevant `SEED` record can be silently
//! re-canonicalised by this or any future T.12.x proposal.
//!
//! Net result: **4 genuinely new canonicals** (Shiryaev-Roberts,
//! GLR, Binary segmentation, PELT-style deterministic), **7
//! existing-canonical SCD cross-class authority resolutions**
//! (CUSUM, Page-Hinkley, Mann-Kendall, Pettitt, SNHT, MOSUM,
//! Buishand range), and **1 RejectedNotDeterministic record**
//! for BOCPD. Thirteen dedup-court records total (4 + 7 + 1 +
//! 1; CUSUM is the subject of both an
//! `ExistingCanonicalAuthorityResolution` record and a
//! `DomainTransferOf` record so the court captures both
//! perspectives).
//!
//! ## Court-delta categories the proposal exercises
//!
//! Panel-locked wire names appearing in
//! `ProposedDedupRecord::decision_wire_name`:
//!
//! * `CanonicalAddition` — the four new SCD primitives at
//!   reserved canonical ids 5201, 5202, 5207, 5208.
//! * `ExistingCanonicalAuthorityResolution` — six panel-locked
//!   records keeping CUSUM (id 3), Page-Hinkley (id 4),
//!   Mann-Kendall (id 11), Pettitt (id 34), SNHT (id 35), MOSUM
//!   (id 36), Buishand range (id 37) canonical without
//!   duplication. (Note: CUSUM also receives a `DomainTransferOf`
//!   record — see next bullet — so it carries both perspectives.)
//! * `DomainTransferOf` — one record noting that CUSUM (SEED id
//!   3) is the shared ancestor for the SCD canonical-additions
//!   in this batch (Shiryaev-Roberts, GLR, Binary segmentation,
//!   PELT all extend the CUSUM-style sequential update / cost
//!   primitive); the court records the domain transfer without
//!   re-canonicalising CUSUM.
//! * `RejectedNotDeterministic` — one record for BOCPD at
//!   reserved id 5209, brutally explicit that BOCPD as a
//!   literature-original canonical primitive is probabilistic
//!   (Adams & MacKay 2007) and admitted neither to SEED nor to
//!   the dedup-court delta's `new_canonical_records` unless a
//!   later T.12.x proposal files a deterministic reduction with
//!   the hazard, prior, update law, truncation, and numeric mode
//!   declared.
//!
//! `DeferredToDriftDetection` is documented in the proposal
//! motivation + the per-section receipt prose: ADWIN, DDM, EDDM,
//! HDDM, KSWIN and the stream-drift family are intentionally NOT
//! folded into T.12.b — they belong in T.12.c (drift /
//! distribution distance). The receipt makes the deferral
//! explicit so a reviewer cannot mistake the omission for an
//! oversight.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * `registry_hash_v2`, every T.11/S1.3/T.12.0/T.12.a hash, and
//!   every `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.b SCD
//!   `corpus_amendment_proposal_hash_v1` distinct from T.12.0's
//!   empty proof-of-life hash AND from T.12.a's SPC hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` as T.12.0 / T.12.a; every `pub`
//! item AND every private helper carries a doc comment whose
//! first sentence states the WHY for a future engineer; 10-step
//! ritual; no `--no-verify`.

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

/// Reserved canonical id for Shiryaev-Roberts. The 5201..5209
/// range was set aside in the T.12.b memory so canonical ids
/// are visually grouped per T.12.x sub-campaign (T.12.a used
/// 5001/5002, T.12.b uses 5201+). The verifier rejects any id
/// in `SEED` so the constant exists as a single audit anchor
/// for the rest of the module.
pub const SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID: u32 = 5201;

/// Reserved canonical id for the GLR (generalized likelihood
/// ratio) change detector. Lorden 1971 / Siegmund-Venkatraman
/// 1995.
pub const GLR_RESERVED_CANONICAL_ID: u32 = 5202;

/// Reserved canonical id for Binary segmentation. Scott & Knott
/// 1974 (recursive single-changepoint search applied to a
/// homogeneous-subsequence cost). T.12.b admits BS as canonical
/// because the recursive-search functional is structurally
/// distinct from the single-changepoint statistics already in
/// SEED (Pettitt, SNHT). 5207 was chosen so the BS / PELT pair
/// is visually adjacent in the reserved range.
pub const BINARY_SEGMENTATION_RESERVED_CANONICAL_ID: u32 = 5207;

/// Reserved canonical id for PELT-style deterministic cost
/// changepoint detector. Killick, Fearnhead & Eckley 2012
/// (Pruned Exact Linear Time). T.12.b admits PELT as canonical
/// because the dynamic-programming pruning functional is
/// structurally distinct from binary segmentation's recursive
/// search.
pub const PELT_RESERVED_CANONICAL_ID: u32 = 5208;

/// Reserved canonical id for BOCPD (Adams & MacKay 2007). T.12.b
/// records BOCPD as `RejectedNotDeterministic` — it appears in
/// `proposed_primitives` (the proposal acknowledges the
/// literature record exists) but NOT in
/// `new_canonical_records` of the dedup-court delta. A future
/// T.12.x proposal may admit a `Deterministic_BOCPD_Proxy`
/// canonical with the hazard, prior, update law, truncation,
/// and numeric mode declared.
pub const BOCPD_RESERVED_PRIMITIVE_ID: u32 = 5209;

// Existing SEED canonical ids referenced by the T.12.b
// cross-class authority resolutions. Pinned here as named
// constants so reviewers can audit the references at a glance
// without grepping `seed.rs`.

/// CUSUM — already canonical in SEED at id 3.
pub const CUSUM_SEED_ID: u32 = 3;

/// Page-Hinkley test — already canonical in SEED at id 4.
pub const PAGE_HINKLEY_SEED_ID: u32 = 4;

/// Mann-Kendall trend test — already canonical in SEED at id
/// 11. Treated by T.12.b as a **trend witness**, not a generic
/// change-point detector; the cross-class authority resolution
/// preserves that distinction (panel-locked caution).
pub const MANN_KENDALL_SEED_ID: u32 = 11;

/// Pettitt change-point test — already canonical in SEED at id
/// 34.
pub const PETTITT_SEED_ID: u32 = 34;

/// Standard Normal Homogeneity Test (SNHT) — already canonical
/// in SEED at id 35.
pub const SNHT_SEED_ID: u32 = 35;

/// MOSUM (moving-sum-of-residuals) test — already canonical in
/// SEED at id 36.
pub const MOSUM_SEED_ID: u32 = 36;

/// Buishand range test — already canonical in SEED at id 37.
pub const BUISHAND_SEED_ID: u32 = 37;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------
//
// These string constants exist so the acceptance tests can match
// the wire names without copy-pasting string literals across the
// suite. The wire names ALSO appear inline in the
// `spc_proposed_dedup_records` builder so the in-source reading
// order matches the receipt rendering.

/// `CanonicalAddition` — a new literature primitive admitted to
/// the dedup-court delta as canonical. T.12.b emits this for
/// Shiryaev-Roberts, GLR, Binary segmentation, and PELT.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution` — an existing SEED
/// canonical is recognised as the authoritative SCD record;
/// no duplicate is admitted. T.12.b emits this for CUSUM (id 3),
/// Page-Hinkley (id 4), Mann-Kendall (id 11), Pettitt (id 34),
/// SNHT (id 35), MOSUM (id 36), Buishand range (id 37).
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf` — an existing SEED canonical is recorded
/// as the ancestor / shared sequential-change-detection
/// primitive for the new canonicals in this batch. T.12.b emits
/// one such record for CUSUM (the shared parent of
/// Shiryaev-Roberts / GLR / Binary segmentation / PELT) without
/// re-canonicalising CUSUM.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `RejectedNotDeterministic` — a literature record is
/// acknowledged but NOT admitted to the dedup-court delta's
/// `new_canonical_records` because its literature definition
/// is probabilistic in origin and no deterministic reduction
/// is declared in this proposal. T.12.b emits this for BOCPD.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

// ---------------------------------------------------------------
// Builders for the SCD expansion batch
// ---------------------------------------------------------------

/// Build the SCD `CorpusExpansionBatch` body: 5 proposed
/// primitives (4 admitted as canonical + 1 BOCPD rejection
/// shell), 0 alias claims (T.12.b adds no alias collapses; the
/// SCD literature's naming overlaps are already absorbed by the
/// existing SEED records the cross-class authority records
/// recognise), 12 dedup-court records, 6 genealogy edges, 6
/// source refs.
fn build_scd_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_b_scd_first_proposal",
        SourceClass::SequentialChangeDetection,
        scd_proposed_primitives(),
        scd_proposed_aliases(),
        scd_proposed_dedup_records(),
        scd_proposed_genealogy_edges(),
        scd_proposed_source_refs(),
    )
}

/// Five proposed primitives: the four genuinely new canonicals
/// (Shiryaev-Roberts, GLR, Binary segmentation, PELT) plus
/// BOCPD as a rejection-shell entry.
///
/// BOCPD appears in `proposed_primitives` so a reviewer reading
/// the batch sees that the literature record was considered and
/// rejected (not silently ignored), and so the cross-class
/// invariant `t12_b_rejects_bocpd_as_canonical_without_
/// deterministic_reduction_status` can assert against an actual
/// in-batch record.
fn scd_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID),
            display_name: "Shiryaev-Roberts change detector",
            motivation: "Sequential change detector built on Roberts' SPRT extension \
                 (Roberts 1959) and Shiryaev's quickest-detection formulation \
                 (Shiryaev 1963). Structurally distinct from CUSUM (different \
                 sequential-update law: posterior-likelihood ratio vs cumulative \
                 sum). SEED has no Shiryaev-Roberts; T.12.b admits it as canonical.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(GLR_RESERVED_CANONICAL_ID),
            display_name: "Generalized Likelihood Ratio (GLR) change detector",
            motivation: "Sequential change detector based on the generalized \
                 likelihood ratio test (Lorden 1971; Siegmund & Venkatraman 1995). \
                 Structurally distinct from CUSUM (different decision functional: \
                 supremum-of-window-GLR vs cumulative sum). SEED has no GLR \
                 change detector; T.12.b admits it as canonical.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            display_name: "Binary segmentation (recursive change-point search)",
            motivation: "Recursive single-changepoint search applied to a \
                 homogeneous-subsequence cost (Scott & Knott 1974). \
                 Structurally distinct from the single-changepoint statistics \
                 already in SEED (Pettitt, SNHT, Buishand) because the recursion \
                 is the load-bearing functional. SEED has no binary segmentation \
                 record; T.12.b admits it as canonical.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PELT_RESERVED_CANONICAL_ID),
            display_name: "PELT-style deterministic cost changepoint detector",
            motivation: "Pruned Exact Linear Time changepoint detection \
                 (Killick, Fearnhead & Eckley 2012). Structurally distinct from \
                 binary segmentation (different decision functional: \
                 dynamic-programming exact min-cost vs recursive single-cut \
                 search). SEED has no PELT record; T.12.b admits it as canonical.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BOCPD_RESERVED_PRIMITIVE_ID),
            display_name: "BOCPD (Bayesian Online Change Point Detection) - rejected shell",
            motivation: "Bayesian Online Change Point Detection (Adams & MacKay 2007) \
                 is probabilistic in origin: it maintains a run-length posterior \
                 over a declared hazard prior. T.12.b does NOT admit BOCPD to \
                 the dedup-court delta's new_canonical_records. A future T.12.x \
                 proposal may admit a Deterministic_BOCPD_Proxy canonical with \
                 the hazard, prior, update law, truncation, and numeric mode \
                 declared and brutally explicit.",
        },
    ]
}

/// Zero alias claims. T.12.b records its existing-canonical
/// recognitions via `proposed_dedup_records` with the
/// `ExistingCanonicalAuthorityResolution` wire name rather than
/// via `ProposedAliasClaim`, because alias claims collapse a NEW
/// alias name into an existing canonical (T.12.a's pattern) and
/// T.12.b is not introducing new alias names — it is recognising
/// existing canonicals AS the authoritative SCD records. Leaving
/// the alias list empty is the panel-locked correct posture.
fn scd_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Thirteen dedup-court decisions on the SCD batch:
///
/// * 4 `CanonicalAddition` — Shiryaev-Roberts, GLR, Binary
///   segmentation, PELT.
/// * 7 `ExistingCanonicalAuthorityResolution` — CUSUM (3),
///   Page-Hinkley (4), Mann-Kendall (11), Pettitt (34), SNHT
///   (35), MOSUM (36), Buishand range (37).
/// * 1 `DomainTransferOf` — CUSUM (3) as the shared ancestor
///   for the four new canonical additions.
/// * 1 `RejectedNotDeterministic` — BOCPD (reserved 5209).
///
/// The 13-record set is the panel-locked T.12.b headline:
/// cross-class dedup authority exercised over four court-delta
/// categories.
fn scd_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 4 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID),
            reason: "Shiryaev-Roberts: posterior-likelihood-ratio sequential update law; \
                 structurally distinct from CUSUM's cumulative-sum law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(GLR_RESERVED_CANONICAL_ID),
            reason: "GLR change detector: supremum-of-window generalized likelihood ratio; \
                 structurally distinct from CUSUM's cumulative-sum law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            reason: "Binary segmentation: recursive single-changepoint search functional; \
                 structurally distinct from Pettitt / SNHT single-changepoint statistics \
                 (the recursion is the load-bearing decision functional, not the inner \
                 test).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PELT_RESERVED_CANONICAL_ID),
            reason: "PELT-style deterministic: dynamic-programming exact-min-cost \
                 changepoint detection; structurally distinct from binary segmentation \
                 (different decision functional: exact DP vs recursive single-cut).",
        },
        // -- 7 ExistingCanonicalAuthorityResolution records --
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            reason: "CUSUM stays canonical at SEED id 3. The court records it as the \
                 shared sequential-SPC ancestor for SCD without re-canonicalising it.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PAGE_HINKLEY_SEED_ID),
            reason: "Page-Hinkley stays canonical at SEED id 4. SCD-adjacent; no \
                 duplicate is admitted under the SequentialChangeDetection source \
                 class. The court explicitly resolves the SPC / SCD overlap in \
                 Page-Hinkley's favour as a single canonical record.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MANN_KENDALL_SEED_ID),
            reason: "Mann-Kendall stays canonical at SEED id 11 as a TREND witness, \
                 not a generic change-point detector. The court records the SCD / \
                 trend-rank cross-class adjacency without relabelling Mann-Kendall \
                 broader than its trend-witness role.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PETTITT_SEED_ID),
            reason: "Pettitt change-point test stays canonical at SEED id 34. T.12.b \
                 explicitly catches the SCD-literature naming so no duplicate enters \
                 under reserved id 5xxx.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SNHT_SEED_ID),
            reason: "Standard Normal Homogeneity Test stays canonical at SEED id 35. \
                 T.12.b explicitly catches the SCD-literature naming so no duplicate \
                 enters under reserved id 5xxx.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MOSUM_SEED_ID),
            reason: "MOSUM (moving-sum-of-residuals) stays canonical at SEED id 36. \
                 T.12.b explicitly catches the SCD-literature naming so no duplicate \
                 enters under reserved id 5xxx.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(BUISHAND_SEED_ID),
            reason: "Buishand range test stays canonical at SEED id 37. T.12.b \
                 explicitly catches the SCD-literature naming so no duplicate \
                 enters under reserved id 5xxx.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            reason: "CUSUM (SEED id 3) is the shared sequential-update ancestor for \
                 the four new SCD canonical additions in this batch (Shiryaev-Roberts, \
                 GLR, Binary segmentation, PELT). The court records the domain \
                 transfer without re-canonicalising CUSUM.",
        },
        // -- 1 RejectedNotDeterministic record ---------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(BOCPD_RESERVED_PRIMITIVE_ID),
            reason: "BOCPD (Adams & MacKay 2007) is probabilistic in origin: it \
                 maintains a posterior over a declared hazard prior. Rejected as a \
                 literature-original canonical primitive for this deterministic corpus \
                 unless reduced to a declared deterministic proxy (hazard, prior, \
                 update law, truncation, numeric mode all brutally explicit) in a \
                 later T.12.x proposal.",
        },
    ]
}

/// Six genealogy edges proposed for the post-freeze graph.
///
/// All four new canonicals are recorded as `DerivedFrom CUSUM`,
/// matching the panel's panel-locked framing that CUSUM is the
/// shared sequential-SPC ancestor of the SCD canonical-addition
/// family. Binary segmentation is additionally recorded as
/// `Generalizes Pettitt` (BS uses Pettitt-style single-cut
/// inner tests recursively), and PELT is recorded as
/// `Generalizes BinarySegmentation` (PELT is the
/// dynamic-programming exact analogue of binary segmentation).
fn scd_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(GLR_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(PETTITT_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PELT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PELT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
    ]
}

/// Six source refs supporting the SCD expansion. We cite the
/// original methodological papers for each new canonical, plus
/// the BOCPD reference so the rejection record can point at the
/// probabilistic origin honestly.
fn scd_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "roberts_sr_1959",
            title: "Control Chart Tests Based on Geometric Moving Averages",
            year: 1959,
            venue: "Technometrics 1(3)",
        },
        ProposedSourceRef {
            citation_key: "shiryaev_qd_1963",
            title: "On Optimum Methods in Quickest Detection Problems",
            year: 1963,
            venue: "Theory of Probability and its Applications 8(1)",
        },
        ProposedSourceRef {
            citation_key: "lorden_glr_1971",
            title: "Procedures for Reacting to a Change in Distribution",
            year: 1971,
            venue: "Annals of Mathematical Statistics 42(6)",
        },
        ProposedSourceRef {
            citation_key: "scott_knott_binseg_1974",
            title: "A Cluster Analysis Method for Grouping Means in the Analysis of Variance",
            year: 1974,
            venue: "Biometrics 30(3)",
        },
        ProposedSourceRef {
            citation_key: "killick_fearnhead_eckley_pelt_2012",
            title: "Optimal Detection of Changepoints With a Linear Computational Cost",
            year: 2012,
            venue: "Journal of the American Statistical Association 107(500)",
        },
        ProposedSourceRef {
            citation_key: "adams_mackay_bocpd_2007",
            title: "Bayesian Online Changepoint Detection",
            year: 2007,
            venue: "arXiv:0710.3742 (rejection-shell reference; BOCPD is probabilistic)",
        },
    ]
}

/// Build the SCD `DedupCourtDelta`: the per-batch dedup outcome.
///
/// * `new_canonical_records` — the FOUR genuinely new canonicals
///   only (Shiryaev-Roberts, GLR, Binary segmentation, PELT).
///   **BOCPD is deliberately absent** from this list; that is
///   the load-bearing property the
///   `t12_b_rejects_bocpd_as_canonical_without_deterministic_reduction_status`
///   test pins.
/// * `new_alias_records` — empty (T.12.b adds no new alias
///   collapses; cross-class authority is recorded via
///   `proposed_dedup_records` instead).
/// * `new_composition_records` — empty (T.12.b proposes no
///   composition reclassifications; that was the T.12.a story
///   for Western Electric / Nelson rules).
/// * `rejection_records` — empty at the delta level. BOCPD's
///   rejection is encoded via the `RejectedNotDeterministic`
///   entry in `proposed_dedup_records` plus the proposed-
///   primitive shell; the delta's `rejection_records` field is
///   reserved for per-alias rejections, which T.12.b has none
///   of (no aliases proposed).
/// * `deferred_records` — empty at the delta level. The
///   panel-locked `DeferredToDriftDetection` posture (ADWIN,
///   DDM, EDDM, HDDM, KSWIN deferred to T.12.c) is documented
///   in the proposal motivation + the per-section receipt
///   prose rather than via an alias-id deferral, because the
///   deferral targets a category (drift / streaming) rather
///   than a specific alias id.
fn build_scd_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_b_scd_delta",
        vec![
            DetectorCanonicalId(SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(GLR_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BINARY_SEGMENTATION_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PELT_RESERVED_CANONICAL_ID),
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

/// Build the T.12.b SCD `CorpusAmendmentProposal`. Two builds
/// against this static seed produce byte-identical bytes.
///
/// **Panel-locked seed posture**:
///
/// * `status = Open` — proposal is filed but the court has not
///   ratified. A future formal freeze campaign produces
///   `corpus_hash_v2` once the court has approved this
///   proposal alongside enough other T.12.x sub-campaigns.
/// * `proposer_role = PanelMember` — filed by the same panel
///   that authored T.12.0 / T.12.a.
/// * `created_at_commit = "t12_b_scd"` — placeholder; a future
///   formal freeze campaign cites the actual commit hash.
#[must_use]
pub fn seed_t12_b_scd_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_b_scd_first_proposal",
        "T.12.b files the Sequential Change Detection amendment proposal. \
         Adds four genuinely new canonical SCD primitives (Shiryaev-Roberts, \
         GLR, Binary segmentation, PELT-style deterministic) at reserved \
         canonical ids 5201, 5202, 5207, 5208. Records seven \
         ExistingCanonicalAuthorityResolution decisions keeping CUSUM (SEED \
         id 3), Page-Hinkley (id 4), Mann-Kendall (id 11), Pettitt (id 34), \
         SNHT (id 35), MOSUM (id 36), Buishand range (id 37) canonical \
         without duplication. Records one DomainTransferOf decision noting \
         CUSUM as shared SCD ancestor. Rejects BOCPD (Adams & MacKay 2007) \
         as RejectedNotDeterministic at reserved id 5209 — BOCPD is \
         acknowledged in proposed_primitives but NOT in the delta's \
         new_canonical_records. Stream-drift detectors (ADWIN, DDM, EDDM, \
         HDDM, KSWIN) are explicitly deferred to T.12.c (drift / \
         distribution distance) — see the per-section receipt for the \
         DeferredToDriftDetection prose. Does NOT mutate SEED \
         (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::SequentialChangeDetection,
        build_scd_expansion_batch(),
        build_scd_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_b_scd",
    )
}
