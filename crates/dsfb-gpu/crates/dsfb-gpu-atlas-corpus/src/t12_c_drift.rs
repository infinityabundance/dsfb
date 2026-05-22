//! T.12.c — Drift Detection and Distribution-Distance Authority:
//! the third real literature expansion proposal filed through
//! the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.c files the Drift Detection / Distribution Distance
//! > amendment proposal. It adds only deterministic
//! > drift-distance primitives whose reference-distribution,
//! > windowing, binning, and sampling contracts are declared;
//! > resolves collisions with existing SEED records; classifies
//! > streaming drift algorithms as canonical, parameterized,
//! > domain-transfer, or deferred without mutating the frozen
//! > T.10 corpus.**
//!
//! **Main panel instruction (panel-locked)**: *"Do not count
//! method names. Count distinct deterministic decision
//! functionals with declared reference / window / sampling
//! contracts."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! Per the panel's `t12_c_detects_existing_seed_collisions_
//! before_new_canonical_assignment` mandate, T.12.c's design
//! began with a grep of [`crate::seed::SEED`] for every
//! candidate name in the panel's draft list. The walk found
//! **eleven** distribution-distance primitives already
//! canonical: Kolmogorov-Smirnov (id 8), Kullback-Leibler (id
//! 9), Maximum Mean Discrepancy (id 10), Anderson-Darling (id
//! 26), Cramer-von Mises (id 27), Wasserstein / earth-mover
//! distance (id 28), Energy distance (id 29), Hellinger
//! distance (id 30), Population Stability Index (id 31),
//! Jensen-Shannon divergence (id 32), Total variation distance
//! (id 33). Promoting any of them again would have produced
//! the very duplication the amendment court exists to forbid;
//! all eleven are therefore recorded as
//! `ExistingCanonicalAuthorityResolution` records.
//!
//! Of the remaining candidates the panel named (ADWIN, DDM,
//! EDDM, HDDM, KSWIN, Kuiper), the court ruled:
//!
//! * **Kuiper** is structurally distinct from KS (decision
//!   functional is `D+ + D-` rather than `max(D+, D-)`; the
//!   test is circular-symmetric and assigns equal weight to
//!   both tails). `CanonicalAddition` at reserved id 5301.
//! * **ADWIN** (Bifet & Gavalda 2007) is structurally distinct:
//!   adaptive-window sequential drift detection using Hoeffding
//!   concentration bounds. `CanonicalAddition` at reserved id
//!   5302 with the declared deterministic contract recorded in
//!   the dedup-record reason (adaptive window pair, Hoeffding
//!   delta, deterministic cut + window-merge rule, numeric
//!   mode declared).
//! * **DDM** (Gama 2004) is a sequential test on a binary
//!   error-rate stream; structurally distinct from CUSUM
//!   (different statistic family: `p_i + 2 * s_i` warning /
//!   drift thresholds based on minimum-seen).
//!   `CanonicalAddition` at reserved id 5303.
//! * **HDDM** (Frias-Blanco 2014) replaces DDM's heuristic
//!   threshold with a Hoeffding-bound equality-of-means test;
//!   structurally distinct from DDM (different concentration
//!   argument). `CanonicalAddition` at reserved id 5304.
//! * **EDDM** (Baena-Garcia 2006) is the same DDM family but
//!   tracks distance between consecutive errors instead of
//!   error rate. Recorded as `ParameterizationOf(DDM)` at
//!   reserved id 5305 — the family relationship is the
//!   load-bearing decision; not promoted to
//!   `new_canonical_records`.
//! * **KSWIN** (Raab, Heusinger & Schleif 2020) is a
//!   streaming-windowed application of the Kolmogorov-Smirnov
//!   two-sample test. Recorded as `ParameterizationOf(KS, SEED
//!   id 8)` at reserved id 5306 with the declared streaming-
//!   window contract; not promoted to `new_canonical_records`.
//!
//! ## Court-delta categories the proposal exercises
//!
//! Panel-locked wire names appearing in
//! `ProposedDedupRecord::decision_wire_name`:
//!
//! * `CanonicalAddition` — the four new drift / distribution
//!   primitives at reserved canonical ids 5301..=5304.
//! * `ExistingCanonicalAuthorityResolution` — eleven panel-
//!   locked records keeping every existing SEED distribution-
//!   distance canonical (8, 9, 10, 26..=33) canonical without
//!   duplication.
//! * `DomainTransferOf` — one record naming KS (SEED id 8) as
//!   the most fundamental two-sample distribution-distance
//!   primitive and the shared ancestor recognised by the drift-
//!   detection source class.
//! * `ParameterizationOf` — two records (EDDM as
//!   `ParameterizationOf(DDM)`, KSWIN as
//!   `ParameterizationOf(KS)`) documenting that the literature
//!   record exists but the structurally-distinct decision
//!   functional belongs to the parent canonical.
//!
//! Total: 4 + 11 + 1 + 2 = **18 dedup-court records**.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * `registry_hash_v2`, every T.11/S1.3 hash, every
//!   T.12.0/T.12.a/T.12.b hash, and every `DetectorPassport`
//!   hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.c drift
//!   `corpus_amendment_proposal_hash_v1` distinct from every
//!   prior T.12.x proposal hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` as T.12.0 / T.12.a / T.12.b;
//! every `pub` item AND every private helper carries a doc
//! comment whose first sentence states the WHY for a future
//! engineer; 10-step ritual; no `--no-verify`.

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

/// Reserved canonical id for Kuiper test. 5301..5306 is the
/// T.12.c bucket (T.12.a used 5001/5002, T.12.b used 5201+).
/// Kuiper extends KS to circular distributions and uses the
/// `D+ + D-` decision functional (equal-tail-weight) instead
/// of KS's `max(D+, D-)`.
pub const KUIPER_RESERVED_CANONICAL_ID: u32 = 5301;

/// Reserved canonical id for ADWIN (Bifet & Gavalda 2007).
/// Adaptive-window sequential drift detection using Hoeffding
/// concentration bounds. CanonicalAddition with declared
/// adaptive-window contract (cut rule, delta, deterministic
/// window-merge rule, numeric mode all recorded in the
/// dedup-record reason).
pub const ADWIN_RESERVED_CANONICAL_ID: u32 = 5302;

/// Reserved canonical id for DDM (Drift Detection Method,
/// Gama 2004). Sequential test on a binary error-rate stream:
/// `p_i + 2 * s_i` warning / `p_i + 3 * s_i` drift thresholds
/// based on minimum-seen `p_min + s_min`. Structurally distinct
/// from CUSUM (different statistic family).
pub const DDM_RESERVED_CANONICAL_ID: u32 = 5303;

/// Reserved canonical id for HDDM (Hoeffding's-bound Drift
/// Detection Method, Frias-Blanco et al. 2014). Replaces DDM's
/// heuristic threshold with a Hoeffding-bound equality-of-means
/// concentration test. Structurally distinct from DDM (different
/// decision functional).
pub const HDDM_RESERVED_CANONICAL_ID: u32 = 5304;

/// Reserved id for EDDM (Early Drift Detection Method,
/// Baena-Garcia 2006). Tracks distance between consecutive
/// errors instead of error rate. Same DDM family —
/// `ParameterizationOf(DDM)`; appears in `proposed_primitives`
/// but NOT in `new_canonical_records`.
pub const EDDM_RESERVED_PRIMITIVE_ID: u32 = 5305;

/// Reserved id for KSWIN (Raab, Heusinger & Schleif 2020).
/// Streaming-windowed application of the Kolmogorov-Smirnov
/// two-sample test. `ParameterizationOf(KS, SEED id 8)`;
/// appears in `proposed_primitives` but NOT in
/// `new_canonical_records`.
pub const KSWIN_RESERVED_PRIMITIVE_ID: u32 = 5306;

// Existing SEED canonical ids referenced by the T.12.c
// cross-class authority resolutions. Pinned here as named
// constants so reviewers can audit the references at a glance
// without grepping `seed.rs`.

/// Kolmogorov-Smirnov two-sample test — SEED canonical id 8.
pub const KS_SEED_ID: u32 = 8;

/// Kullback-Leibler divergence — SEED canonical id 9.
pub const KL_SEED_ID: u32 = 9;

/// Maximum Mean Discrepancy (MMD) — SEED canonical id 10.
pub const MMD_SEED_ID: u32 = 10;

/// Anderson-Darling test — SEED canonical id 26.
pub const ANDERSON_DARLING_SEED_ID: u32 = 26;

/// Cramer-von Mises test — SEED canonical id 27.
pub const CRAMER_VON_MISES_SEED_ID: u32 = 27;

/// Wasserstein / earth-mover distance — SEED canonical id 28.
pub const WASSERSTEIN_SEED_ID: u32 = 28;

/// Energy distance — SEED canonical id 29.
pub const ENERGY_DISTANCE_SEED_ID: u32 = 29;

/// Hellinger distance — SEED canonical id 30.
pub const HELLINGER_SEED_ID: u32 = 30;

/// Population Stability Index (PSI) — SEED canonical id 31.
pub const PSI_SEED_ID: u32 = 31;

/// Jensen-Shannon divergence — SEED canonical id 32.
pub const JENSEN_SHANNON_SEED_ID: u32 = 32;

/// Total variation distance — SEED canonical id 33.
pub const TOTAL_VARIATION_SEED_ID: u32 = 33;

/// CUSUM — SEED canonical id 3, referenced by the DDM
/// genealogy edge as the sequential-test family ancestor.
pub const CUSUM_SEED_ID: u32 = 3;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------
//
// These string constants exist so the acceptance tests can match
// the wire names without copy-pasting string literals across the
// suite. `CanonicalAddition`, `ExistingCanonicalAuthorityResolution`,
// and `DomainTransferOf` were introduced in T.12.b; T.12.c adds
// the fourth category `ParameterizationOf`.

/// `CanonicalAddition` — a new literature primitive admitted to
/// the dedup-court delta as canonical.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution` — an existing SEED
/// canonical is recognised as the authoritative drift /
/// distribution-distance record; no duplicate is admitted.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf` — an existing SEED canonical is recorded
/// as the ancestor / shared distribution-distance primitive
/// for the drift-detection source class without
/// re-canonicalising it.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf` — a streaming or family-variant
/// literature record is acknowledged but NOT admitted to the
/// dedup-court delta's `new_canonical_records`; the family
/// relationship to the parent canonical is the load-bearing
/// decision. T.12.c emits this for EDDM (of DDM) and KSWIN
/// (of KS).
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

// ---------------------------------------------------------------
// Builders for the drift expansion batch
// ---------------------------------------------------------------

/// Build the drift `CorpusExpansionBatch` body: 6 proposed
/// primitives (4 admitted as canonical + 2 parameterizations of
/// existing canonicals), 0 alias claims, 18 dedup-court records,
/// 6 genealogy edges, 6 source refs.
fn build_drift_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_c_drift_first_proposal",
        SourceClass::DriftDetection,
        drift_proposed_primitives(),
        drift_proposed_aliases(),
        drift_proposed_dedup_records(),
        drift_proposed_genealogy_edges(),
        drift_proposed_source_refs(),
    )
}

/// Six proposed primitives: the four genuinely new canonicals
/// (Kuiper, ADWIN, DDM, HDDM) plus the two parameterization
/// shells (EDDM, KSWIN). The parameterizations appear in
/// `proposed_primitives` so a reviewer reading the batch sees
/// that the literature records were considered, and so the
/// cross-class invariant
/// `t12_c_rejects_kswin_as_canonical_without_ks_relationship_decision`
/// (and the EDDM analogue) can assert against actual in-batch
/// records.
fn drift_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(KUIPER_RESERVED_CANONICAL_ID),
            display_name: "Kuiper test (circular-symmetric two-sample)",
            motivation: "Kuiper (1960) extends the Kolmogorov-Smirnov two-sample test \
                 to the circle. Structurally distinct decision functional: D+ + D- \
                 (equal-tail-weight sum) vs KS's max(D+, D-) (single-tail maximum). \
                 SEED has KS at id 8 but no Kuiper record; T.12.c admits it as \
                 canonical. Requires reference-window pair; deterministic statistic \
                 with no probabilistic sampling step.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(ADWIN_RESERVED_CANONICAL_ID),
            display_name: "ADWIN (Adaptive Windowing)",
            motivation: "ADWIN (Bifet & Gavalda 2007) is an adaptive-window sequential \
                 drift detector. Maintains a variable-length window of recent values; \
                 partitions the window into all adjacent sub-window pairs; cuts the \
                 window at any pair whose mean-difference exceeds the Hoeffding-bound \
                 threshold for declared confidence delta. The deterministic contract \
                 declares: adaptive-window pair enumeration order, Hoeffding delta \
                 value, cut + window-merge tie-break rule, numeric mode. Deterministic \
                 throughout; no probabilistic sampling; no learned weights.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            display_name: "DDM (Drift Detection Method)",
            motivation: "DDM (Gama 2004) is a sequential test on a binary error-rate \
                 stream from an online classifier. Tracks running error rate p_i and \
                 its standard deviation s_i; signals warning when p_i + 2*s_i exceeds \
                 the running minimum p_min + s_min, drift when it exceeds p_min + \
                 2*s_min. Structurally distinct from CUSUM (different statistic \
                 family: cumulative-deviation-from-minimum-seen rather than \
                 cumulative-sum-of-deviations-from-mean). Requires ordered binary \
                 error sequence; deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HDDM_RESERVED_CANONICAL_ID),
            display_name: "HDDM (Hoeffding's-bound Drift Detection)",
            motivation: "HDDM (Frias-Blanco et al. 2014) replaces DDM's heuristic \
                 thresholds with a Hoeffding-bound equality-of-means concentration \
                 test. Two variants (HDDM-A on absolute means, HDDM-W on weighted \
                 means); both have a structurally distinct decision functional from \
                 DDM (concentration-bound test vs minimum-seen deviation). Requires \
                 ordered error sequence and a declared confidence delta; \
                 deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(EDDM_RESERVED_PRIMITIVE_ID),
            display_name: "EDDM (Early Drift Detection Method) - parameterization shell",
            motivation: "EDDM (Baena-Garcia et al. 2006) is the same DDM family but \
                 tracks distance between consecutive errors instead of error rate. \
                 The family relationship is load-bearing; T.12.c records EDDM as \
                 ParameterizationOf(DDM), NOT a new canonical primitive. A future \
                 T.12.x proposal may promote EDDM to canonical if the court rules \
                 the distance-between-errors functional is structurally distinct \
                 enough.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(KSWIN_RESERVED_PRIMITIVE_ID),
            display_name: "KSWIN (Kolmogorov-Smirnov Windowing) - parameterization shell",
            motivation: "KSWIN (Raab, Heusinger & Schleif 2020) is a streaming-windowed \
                 application of the Kolmogorov-Smirnov two-sample test (SEED id 8). \
                 The court rules: KSWIN is a windowing parameterization of KS, NOT a \
                 new canonical primitive. The reference-window pair, window length, \
                 and KS-statistic threshold contract are declared by the \
                 parameterization. A future T.12.x proposal may promote KSWIN to \
                 canonical only if the streaming-windowed decision functional is \
                 judged structurally distinct from sequential KS.",
        },
    ]
}

/// Zero alias claims. T.12.c records its existing-canonical
/// recognitions via `proposed_dedup_records` with the
/// `ExistingCanonicalAuthorityResolution` wire name (T.12.b's
/// pattern), not via `ProposedAliasClaim`.
fn drift_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Eighteen dedup-court decisions on the drift batch:
///
/// * 4 `CanonicalAddition` — Kuiper, ADWIN, DDM, HDDM.
/// * 11 `ExistingCanonicalAuthorityResolution` — KS (8), KL
///   (9), MMD (10), Anderson-Darling (26), Cramer-von Mises
///   (27), Wasserstein (28), Energy distance (29), Hellinger
///   (30), PSI (31), Jensen-Shannon (32), Total variation (33).
/// * 1 `DomainTransferOf` — KS (8) as shared two-sample
///   distribution-distance ancestor for the drift-detection
///   source class.
/// * 2 `ParameterizationOf` — EDDM of DDM (5303), KSWIN of KS
///   (8).
///
/// The 18-record set is the panel-locked T.12.c headline:
/// cross-class dedup authority exercised over four court-delta
/// categories with the new ParameterizationOf category landing
/// for the first time.
fn drift_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 4 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(KUIPER_RESERVED_CANONICAL_ID),
            reason: "Kuiper test: decision functional D+ + D- (equal-tail-weight); \
                 structurally distinct from KS's max(D+, D-). Requires reference \
                 window pair; deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(ADWIN_RESERVED_CANONICAL_ID),
            reason: "ADWIN: adaptive-window pair enumeration with Hoeffding-bound \
                 cut rule (declared adaptive window law). Requires ordered stream + \
                 reference window pair (the adaptive partition itself); deterministic \
                 with declared delta and tie-break rule.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            reason: "DDM: sequential test on binary error-rate stream; minimum-seen \
                 deviation functional p_i + 2*s_i vs p_min + s_min. Requires ordered \
                 binary error sequence + reference window (the running minimum \
                 estimate); deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(HDDM_RESERVED_CANONICAL_ID),
            reason: "HDDM: Hoeffding-bound equality-of-means concentration test on \
                 binary error stream. Structurally distinct from DDM (concentration \
                 bound vs minimum-seen heuristic). Requires ordered binary error \
                 sequence + reference window + declared confidence delta; \
                 deterministic.",
        },
        // -- 11 ExistingCanonicalAuthorityResolution records --
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(KS_SEED_ID),
            reason: "Kolmogorov-Smirnov stays canonical at SEED id 8. The drift-\
                 detection source class recognises KS as the authoritative two-sample \
                 distribution-distance primitive; no duplicate is admitted. Requires \
                 reference window pair.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(KL_SEED_ID),
            reason: "Kullback-Leibler divergence stays canonical at SEED id 9. \
                 Authoritative reference-distribution divergence; no duplicate. \
                 Requires reference distribution (the second argument of D_KL).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MMD_SEED_ID),
            reason: "Maximum Mean Discrepancy stays canonical at SEED id 10. \
                 Authoritative kernel-based two-sample test; no duplicate. Requires \
                 reference window pair + declared kernel.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ANDERSON_DARLING_SEED_ID),
            reason: "Anderson-Darling test stays canonical at SEED id 26. \
                 Authoritative tail-weighted goodness-of-fit / two-sample distance. \
                 Requires reference distribution OR reference window pair.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CRAMER_VON_MISES_SEED_ID),
            reason: "Cramer-von Mises test stays canonical at SEED id 27. \
                 Authoritative integrated-squared-deviation goodness-of-fit / \
                 two-sample distance. Requires reference distribution OR reference \
                 window pair.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(WASSERSTEIN_SEED_ID),
            reason: "Wasserstein / earth-mover distance stays canonical at SEED id \
                 28. Authoritative transport-cost distance; no duplicate. Requires \
                 reference distribution OR reference window pair.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ENERGY_DISTANCE_SEED_ID),
            reason: "Energy distance stays canonical at SEED id 29. Authoritative \
                 characteristic-function-based two-sample distance; no duplicate. \
                 Requires reference window pair.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(HELLINGER_SEED_ID),
            reason: "Hellinger distance stays canonical at SEED id 30. Authoritative \
                 bounded-square-root divergence; no duplicate. Requires reference \
                 distribution.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(PSI_SEED_ID),
            reason: "Population Stability Index stays canonical at SEED id 31. \
                 PSI is a binned distribution divergence; the contract REQUIRES a \
                 declared binning law (bin edges, bin count, treatment of empty bins) \
                 AND a reference distribution. The drift-detection source class \
                 recognises PSI under those constraints; no duplicate is admitted \
                 without the binning law declared.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(JENSEN_SHANNON_SEED_ID),
            reason: "Jensen-Shannon divergence stays canonical at SEED id 32. \
                 Symmetrised + bounded KL family; no duplicate. Requires reference \
                 distribution.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(TOTAL_VARIATION_SEED_ID),
            reason: "Total variation distance stays canonical at SEED id 33. \
                 Authoritative L1-distance over distributions; no duplicate. Requires \
                 reference distribution.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(KS_SEED_ID),
            reason: "Kolmogorov-Smirnov (SEED id 8) is the most fundamental two- \
                 sample distribution-distance primitive and the shared reference \
                 the drift-detection source class transfers in. The court records \
                 the domain transfer without re-canonicalising KS. Every other \
                 distance / divergence record above is recognised under \
                 ExistingCanonicalAuthorityResolution rather than DomainTransferOf \
                 to keep the wire-name set minimal (one DomainTransferOf per \
                 source-class pair).",
        },
        // -- 2 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(EDDM_RESERVED_PRIMITIVE_ID),
            reason: "EDDM is ParameterizationOf(DDM, T.12.c canonical 5303). The \
                 distance-between-consecutive-errors functional is the EDDM-specific \
                 parameterization; the family-level decision rules are DDM's. The \
                 court declines to admit EDDM as a new canonical primitive; a future \
                 T.12.x proposal may promote it only if the distance-between-errors \
                 functional is judged structurally distinct enough.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(KSWIN_RESERVED_PRIMITIVE_ID),
            reason: "KSWIN is ParameterizationOf(KS, SEED id 8). KSWIN's streaming- \
                 window decision functional is the windowing parameterization of the \
                 underlying Kolmogorov-Smirnov two-sample statistic. The court \
                 declines to admit KSWIN as a new canonical primitive; a future \
                 T.12.x proposal may promote KSWIN only if the streaming-windowed \
                 functional is judged structurally distinct from sequential KS.",
        },
    ]
}

/// Six genealogy edges proposed for the post-freeze graph.
fn drift_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(KUIPER_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(KS_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(ADWIN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(KS_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CUSUM_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HDDM_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(EDDM_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(KSWIN_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(KS_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Six source refs supporting the drift expansion: one per new
/// canonical (4) and one per parameterization shell (2).
fn drift_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "kuiper_circular_1960",
            title: "Tests Concerning Random Points on a Circle",
            year: 1960,
            venue:
                "Proceedings of the Koninklijke Nederlandse Akademie van Wetenschappen, Series A 63",
        },
        ProposedSourceRef {
            citation_key: "bifet_gavalda_adwin_2007",
            title: "Learning from Time-Changing Data with Adaptive Windowing",
            year: 2007,
            venue: "SIAM International Conference on Data Mining (SDM)",
        },
        ProposedSourceRef {
            citation_key: "gama_ddm_2004",
            title: "Learning with Drift Detection",
            year: 2004,
            venue: "SBIA 2004: Brazilian Symposium on Artificial Intelligence, LNAI 3171",
        },
        ProposedSourceRef {
            citation_key: "frias_blanco_hddm_2014",
            title: "Online and Non-Parametric Drift Detection Methods Based on Hoeffding's Bounds",
            year: 2014,
            venue: "IEEE Transactions on Knowledge and Data Engineering 27(3)",
        },
        ProposedSourceRef {
            citation_key: "baena_garcia_eddm_2006",
            title: "Early Drift Detection Method",
            year: 2006,
            venue:
                "Fourth International Workshop on Knowledge Discovery from Data Streams (IWKDDS)",
        },
        ProposedSourceRef {
            citation_key: "raab_kswin_2020",
            title: "Reactive Soft Prototype Computing for Concept Drift Streams",
            year: 2020,
            venue: "Neurocomputing 416",
        },
    ]
}

/// Build the drift `DedupCourtDelta`: the per-batch dedup
/// outcome.
///
/// * `new_canonical_records` — the FOUR genuinely new canonicals
///   only (Kuiper, ADWIN, DDM, HDDM). **EDDM and KSWIN are
///   deliberately absent** from this list — the load-bearing
///   property the `t12_c_rejects_kswin_as_canonical_without_
///   ks_relationship_decision` and EDDM-equivalent tests pin.
/// * `new_alias_records` — empty (no new alias collapses).
/// * `new_composition_records` — empty.
/// * `rejection_records` — empty at the delta level. T.12.c
///   has no probabilistic-in-origin candidates to reject; the
///   panel's `t12_c_rejects_probabilistic_or_randomized_distance_
///   without_deterministic_reduction` load-bearing negative is
///   encoded as an in-suite invariant that no record in the
///   actual proposal claims a probabilistic distance.
/// * `deferred_records` — empty at the delta level.
fn build_drift_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_c_drift_delta",
        vec![
            DetectorCanonicalId(KUIPER_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(ADWIN_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(DDM_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(HDDM_RESERVED_CANONICAL_ID),
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

/// Build the T.12.c drift `CorpusAmendmentProposal`. Two builds
/// against this static seed produce byte-identical bytes.
///
/// **Panel-locked seed posture**:
///
/// * `status = Open` — proposal is filed but the court has not
///   ratified. A future formal freeze campaign produces
///   `corpus_hash_v2` once enough T.12.x sub-campaigns have
///   landed.
/// * `proposer_role = PanelMember` — filed by the same panel
///   that authored T.12.0 / T.12.a / T.12.b.
/// * `created_at_commit = "t12_c_drift"` — placeholder; a
///   future formal freeze campaign cites the actual commit hash.
#[must_use]
pub fn seed_t12_c_drift_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_c_drift_first_proposal",
        "T.12.c files the Drift Detection / Distribution-Distance amendment \
         proposal. Adds four genuinely new canonical primitives (Kuiper, ADWIN, \
         DDM, HDDM) at reserved canonical ids 5301..=5304 with declared \
         deterministic contracts (reference-distribution / window-pair / binning \
         / sampling). Records eleven ExistingCanonicalAuthorityResolution \
         decisions keeping every existing SEED distribution-distance canonical \
         (KS id 8, KL 9, MMD 10, Anderson-Darling 26, Cramer-von Mises 27, \
         Wasserstein 28, Energy distance 29, Hellinger 30, PSI 31, Jensen-Shannon \
         32, Total variation 33) canonical under the DriftDetection source class \
         without duplication. Records one DomainTransferOf decision naming KS \
         (SEED id 8) as the shared two-sample distribution-distance ancestor. \
         Records two ParameterizationOf decisions: EDDM (Baena-Garcia 2006) is \
         ParameterizationOf(DDM); KSWIN (Raab et al. 2020) is ParameterizationOf \
         (KS) - both appear in proposed_primitives but NOT in the delta's \
         new_canonical_records. Each record's reason text declares the \
         reference-distribution / window-pair / binning / sampling contract the \
         literature method requires. Does NOT mutate SEED (SEED.len() stays at \
         54); status = Open pending review.",
        SourceClass::DriftDetection,
        build_drift_expansion_batch(),
        build_drift_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_c_drift",
    )
}
