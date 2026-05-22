//! T.12.d — Robust Statistics: the fourth real literature
//! expansion proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.d files the Robust Statistics amendment proposal.
//! > It resolves robust z / MAD aliases against the existing
//! > SEED canonical, admits only robust primitives with explicit
//! > estimator, window, trimming, quartile, or pair-selection
//! > laws, rejects stochastic RANSAC-style claims unless
//! > deterministically reduced, and preserves the frozen T.10
//! > corpus hash.**
//!
//! **Main panel instruction (panel-locked)**: *"Robust-statistics
//! names are alias-heavy. Make estimator law explicit, or
//! collapse / defer."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! Per the SEED-walk-first discipline T.12.b proved and T.12.c
//! exercised at scale, T.12.d began its design with a grep of
//! [`crate::seed::SEED`] for every candidate name. The walk
//! found **three** robust-statistics primitives already
//! canonical:
//!
//! * **Robust z-score (median / MAD)** at SEED id 6 — the
//!   shared robust-location-estimator ancestor for this source
//!   class. Catches modified z-score, median-MAD z-score, and
//!   MAD outlier detector aliases.
//! * **Hampel filter** at SEED id 7 — already canonical.
//!   Catches rolling-Hampel and Hampel-MAD-z-with-replacement
//!   aliases.
//! * **Tukey fences** at SEED id 18 — already canonical.
//!   Catches k×IQR and IQR-fence aliases.
//!
//! All three become `ExistingCanonicalAuthorityResolution`
//! records under the `RobustStatistics` source class. Of the
//! remaining candidates the panel named (Theil-Sen, biweight
//! midvariance, trimmed mean shift, winsorized mean shift,
//! modified z-score, rolling Hampel, k×IQR fence, RANSAC
//! residual proxy), the court ruled:
//!
//! * **Theil-Sen slope estimator** (Theil 1950 / Sen 1968) is
//!   structurally distinct: pair-median slope estimator with
//!   declared pair-selection + slope-median + tie-break + window
//!   laws. `CanonicalAddition` at reserved id 5401.
//! * **Biweight midvariance** (Mosteller & Tukey 1977) is a
//!   structurally distinct robust scale estimator: tuning-
//!   constant + convergence + iteration-bound law. Different
//!   functional from MAD (continuous descending weight vs hard
//!   median deviation). `CanonicalAddition` at reserved id 5402.
//! * **Trimmed mean shift** is a structurally distinct robust
//!   location-shift detector: declared trim-fraction law
//!   (symmetric / one-sided trim, percentile / count semantics).
//!   `CanonicalAddition` at reserved id 5403.
//! * **Winsorized mean shift** is a structurally distinct
//!   variant: declared winsor-limit law (replacement vs trimming
//!   semantics). Different functional from trimmed mean (replace
//!   vs drop). `CanonicalAddition` at reserved id 5404.
//! * **Modified z-score** (Iglewicz & Hoaglin 1993) is the
//!   classic 0.6745 × (x - median) / MAD parameterization of
//!   robust z. `ParameterizationOf(robust-z, SEED 6)` at
//!   reserved id 5405.
//! * **Rolling Hampel** is the windowed-application
//!   parameterization of the Hampel filter.
//!   `ParameterizationOf(Hampel, SEED 7)` at reserved id 5406.
//! * **k×IQR fence** is the parameterized variant of Tukey
//!   fences with declared IQR multiplier (Tukey's default 1.5
//!   for inner / 3.0 for outer fence is one parameterization).
//!   `ParameterizationOf(Tukey fences, SEED 18)` at reserved
//!   id 5407.
//! * **RANSAC residual proxy** (Fischler & Bolles 1981) is
//!   randomized in origin. Acknowledged in `proposed_primitives`
//!   at reserved id 5408 but `RejectedNotDeterministic` —
//!   admitted neither to SEED nor to `new_canonical_records`
//!   unless a future T.12.x proposal admits a
//!   `Deterministic_RANSAC_Proxy` canonical with the seed,
//!   sample schedule, iteration budget, and tie-break law
//!   declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! **T.12.d is the first proposal to exercise ALL FIVE
//! panel-locked court-delta categories** (CanonicalAddition
//! introduced in T.12.0, ExistingCanonicalAuthorityResolution +
//! DomainTransferOf introduced in T.12.b,
//! RejectedNotDeterministic introduced in T.12.b for BOCPD,
//! ParameterizationOf introduced in T.12.c). The wire-name set
//! is now closed at five.
//!
//! Panel-locked wire names appearing in
//! `ProposedDedupRecord::decision_wire_name`:
//!
//! * `CanonicalAddition` ×4 — Theil-Sen, biweight midvariance,
//!   trimmed mean shift, winsorized mean shift.
//! * `ExistingCanonicalAuthorityResolution` ×3 — robust-z (6),
//!   Hampel (7), Tukey fences (18).
//! * `DomainTransferOf` ×1 — robust-z (6) as shared robust-
//!   location-estimator ancestor for the RobustStatistics
//!   source class.
//! * `ParameterizationOf` ×3 — modified z-score
//!   (ParameterizationOf robust-z 6), rolling Hampel
//!   (ParameterizationOf Hampel 7), k×IQR fence
//!   (ParameterizationOf Tukey fences 18).
//! * `RejectedNotDeterministic` ×1 — RANSAC residual proxy
//!   (5408).
//!
//! Total: 4 + 3 + 1 + 3 + 1 = **12 dedup-court records**.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * `registry_hash_v2`, every T.11/S1.3 hash, every
//!   T.12.0/T.12.a/T.12.b/T.12.c hash, and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.d robust
//!   `corpus_amendment_proposal_hash_v1` distinct from every
//!   prior T.12.x proposal hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior T.12.x;
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

/// Reserved canonical id for Theil-Sen slope estimator
/// (Theil 1950 / Sen 1968). 5401..5408 is the T.12.d bucket
/// (T.12.a used 5001+, T.12.b 5201+, T.12.c 5301+). Theil-Sen
/// is a pair-median slope estimator structurally distinct from
/// any SEED robust statistic; declared pair-selection + slope-
/// median + tie-break + window laws.
pub const THEIL_SEN_RESERVED_CANONICAL_ID: u32 = 5401;

/// Reserved canonical id for biweight midvariance (Mosteller
/// & Tukey 1977; Beaton & Tukey 1974). Tukey biweight scale
/// estimator with declared tuning constant + convergence +
/// iteration-bound law.
pub const BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID: u32 = 5402;

/// Reserved canonical id for trimmed mean shift detector
/// (Tukey 1962). Robust location-shift detector with declared
/// trim-fraction law (symmetric / one-sided, percentile / count
/// semantics).
pub const TRIMMED_MEAN_RESERVED_CANONICAL_ID: u32 = 5403;

/// Reserved canonical id for winsorized mean shift detector
/// (Dixon 1960). Robust location-shift detector with declared
/// winsor-limit law (replacement vs trimming semantics). Distinct
/// from trimmed mean (replace vs drop).
pub const WINSORIZED_MEAN_RESERVED_CANONICAL_ID: u32 = 5404;

/// Reserved id for modified z-score (Iglewicz & Hoaglin 1993).
/// `ParameterizationOf(robust-z, SEED 6)` — the 0.6745 × (x -
/// median) / MAD constant-scaling parameterization. Appears in
/// `proposed_primitives` but NOT in `new_canonical_records`.
pub const MODIFIED_Z_RESERVED_PRIMITIVE_ID: u32 = 5405;

/// Reserved id for rolling Hampel filter.
/// `ParameterizationOf(Hampel, SEED 7)` — the windowed-
/// application parameterization. Appears in
/// `proposed_primitives` but NOT in `new_canonical_records`.
pub const ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID: u32 = 5406;

/// Reserved id for k×IQR fence detector.
/// `ParameterizationOf(Tukey fences, SEED 18)` — the
/// IQR-multiplier-parameterized variant (Tukey's defaults 1.5
/// inner / 3.0 outer are one parameterization point in the
/// k-family).
pub const K_IQR_FENCE_RESERVED_PRIMITIVE_ID: u32 = 5407;

/// Reserved id for RANSAC residual proxy (Fischler & Bolles
/// 1981). `RejectedNotDeterministic` — RANSAC is randomized in
/// its literature definition (uniformly random sample selection
/// over iteration budget). Acknowledged in
/// `proposed_primitives` but explicitly NOT in
/// `new_canonical_records`. A future T.12.x proposal may admit
/// a `Deterministic_RANSAC_Proxy` canonical with the seed,
/// sample schedule, iteration budget, and tie-break law
/// declared and brutally explicit.
pub const RANSAC_RESERVED_PRIMITIVE_ID: u32 = 5408;

// Existing SEED canonical ids referenced by the T.12.d
// cross-class authority resolutions.

/// Robust z-score (median / MAD) — SEED canonical id 6.
pub const ROBUST_Z_SEED_ID: u32 = 6;

/// Hampel filter — SEED canonical id 7.
pub const HAMPEL_SEED_ID: u32 = 7;

/// Tukey fences — SEED canonical id 18.
pub const TUKEY_FENCES_SEED_ID: u32 = 18;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------
//
// T.12.d is the first proposal to exercise ALL FIVE panel-locked
// court-delta categories. The wire-name set is now closed at
// five.

/// `CanonicalAddition` — a new literature primitive admitted to
/// the dedup-court delta as canonical.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution` — an existing SEED
/// canonical is recognised as the authoritative robust-statistics
/// record; no duplicate is admitted.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf` — an existing SEED canonical is recorded
/// as the ancestor / shared robust-location-estimator primitive
/// for the RobustStatistics source class.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf` — a family-variant literature record is
/// acknowledged but NOT admitted to the dedup-court delta's
/// `new_canonical_records`; the family relationship to the
/// parent canonical is the load-bearing decision.
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

/// `RejectedNotDeterministic` — a literature record is
/// acknowledged but NOT admitted to the dedup-court delta's
/// `new_canonical_records` because its literature definition
/// is probabilistic / randomized in origin and no deterministic
/// reduction is declared.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

// ---------------------------------------------------------------
// Builders for the robust expansion batch
// ---------------------------------------------------------------

/// Build the robust `CorpusExpansionBatch` body: 8 proposed
/// primitives (4 admitted as canonical + 3 parameterization
/// shells + 1 RANSAC rejection shell), 0 alias claims, 12
/// dedup-court records, 7 genealogy edges, 8 source refs.
fn build_robust_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_d_robust_first_proposal",
        SourceClass::RobustStatistics,
        robust_proposed_primitives(),
        robust_proposed_aliases(),
        robust_proposed_dedup_records(),
        robust_proposed_genealogy_edges(),
        robust_proposed_source_refs(),
    )
}

/// Eight proposed primitives: the four genuinely new canonicals
/// (Theil-Sen, biweight midvariance, trimmed mean shift,
/// winsorized mean shift) plus three parameterization shells
/// (modified z-score, rolling Hampel, k×IQR fence) plus the
/// RANSAC rejection shell.
fn robust_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(THEIL_SEN_RESERVED_CANONICAL_ID),
            display_name: "Theil-Sen slope estimator",
            motivation: "Theil (1950) / Sen (1968) pair-median slope estimator. \
                 Structurally distinct from any SEED robust statistic: computes \
                 the median of all pairwise (y_j - y_i) / (x_j - x_i) slopes over \
                 a declared pair-selection law. Required estimator-law \
                 declarations: pair-selection rule (all-pairs / window-pair / \
                 sampled-pair), slope-median rule (exact / odd-count midpoint), \
                 tie-break law for equal-x pairs, window law for the input \
                 sequence. Deterministic when pair-selection is fully enumerated.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID),
            display_name: "Biweight midvariance (Tukey biweight scale)",
            motivation: "Mosteller & Tukey (1977); Beaton & Tukey (1974). Robust \
                 scale estimator using Tukey's biweight (continuous descending \
                 weight) instead of MAD's hard median deviation. Different decision \
                 functional. Required estimator-law declarations: tuning constant \
                 c (typical Tukey c=9.0), iteration count / convergence threshold, \
                 numeric mode for accumulator. Deterministic with declared bounds.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TRIMMED_MEAN_RESERVED_CANONICAL_ID),
            display_name: "Trimmed mean shift detector",
            motivation: "Tukey (1962) robust location-shift detector. The trimmed \
                 mean drops a declared fraction of the highest and lowest values \
                 before averaging. Required estimator-law declarations: trim \
                 fraction alpha (symmetric / one-sided), percentile vs count \
                 semantics for the cut points, tie-break law at the trim boundary. \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(WINSORIZED_MEAN_RESERVED_CANONICAL_ID),
            display_name: "Winsorized mean shift detector",
            motivation: "Dixon (1960) robust location-shift detector. The winsorized \
                 mean REPLACES the highest and lowest declared fraction with the \
                 closest non-trimmed values rather than dropping them. Different \
                 functional from trimmed mean (replace vs drop). Required \
                 estimator-law declarations: winsor limit, replacement-rule (use \
                 percentile boundary value), tie-break law. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MODIFIED_Z_RESERVED_PRIMITIVE_ID),
            display_name: "Modified z-score (0.6745 / MAD scaling)",
            motivation: "Iglewicz & Hoaglin (1993) modified z-score: \
                 0.6745 * (x - median(x)) / MAD(x). The 0.6745 constant calibrates \
                 MAD to match the normal-distribution standard deviation. The \
                 court rules: modified z-score is a fixed-constant parameterization \
                 of the robust z-score family (SEED canonical 6), NOT a new \
                 canonical primitive. Appears in proposed_primitives but NOT in \
                 new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID),
            display_name: "Rolling Hampel filter",
            motivation: "Rolling-window application of the Hampel filter (SEED \
                 canonical 7). The court rules: rolling Hampel is a windowing \
                 parameterization of the Hampel filter, NOT a new canonical \
                 primitive. The window length + per-window local-median + MAD + \
                 replacement-rule contract is declared by the parameterization. \
                 Appears in proposed_primitives but NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(K_IQR_FENCE_RESERVED_PRIMITIVE_ID),
            display_name: "k x IQR fence detector",
            motivation: "k-multiplier-parameterized variant of Tukey fences (SEED \
                 canonical 18). Tukey's defaults (1.5 x IQR inner fence, 3.0 x IQR \
                 outer fence) are ONE parameterization point in the k-family; \
                 industrial practice uses other multipliers (2.0, 2.5) with the \
                 same decision functional. The court rules: k x IQR fence is a \
                 parameterization of Tukey fences, NOT a new canonical primitive. \
                 The quartile-estimator law, IQR-multiplier value, inclusive / \
                 exclusive fence semantics, and tie-handling at quartile \
                 boundaries are declared by the parameterization. Appears in \
                 proposed_primitives but NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RANSAC_RESERVED_PRIMITIVE_ID),
            display_name: "RANSAC residual proxy - rejected shell",
            motivation: "RANSAC (Fischler & Bolles 1981) is randomized in its \
                 literature definition: at each iteration it uniformly samples a \
                 minimal subset, fits the model, and counts inliers; final model \
                 is the iteration with highest inlier count. The court does NOT \
                 admit RANSAC to the dedup-court delta's new_canonical_records. \
                 A future T.12.x proposal may admit a Deterministic_RANSAC_Proxy \
                 canonical only if the sample seed, iteration budget, fixed \
                 sample schedule (deterministic enumeration of subsets in canonical \
                 order rather than random sampling), tie-break law, and numeric \
                 mode are all brutally explicit. Until then this is a literature \
                 acknowledgement; the deterministic reduction is required and \
                 deliberately not provided in T.12.d.",
        },
    ]
}

/// Zero alias claims. T.12.d records its existing-canonical
/// recognitions via `proposed_dedup_records` with the
/// `ExistingCanonicalAuthorityResolution` wire name.
fn robust_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twelve dedup-court decisions on the robust batch:
///
/// * 4 `CanonicalAddition` — Theil-Sen, biweight midvariance,
///   trimmed mean shift, winsorized mean shift.
/// * 3 `ExistingCanonicalAuthorityResolution` — robust-z (6),
///   Hampel (7), Tukey fences (18).
/// * 1 `DomainTransferOf` — robust-z (6) as shared robust-
///   location-estimator ancestor for the RobustStatistics
///   source class.
/// * 3 `ParameterizationOf` — modified z-score
///   (ParameterizationOf robust-z 6), rolling Hampel
///   (ParameterizationOf Hampel 7), k×IQR fence
///   (ParameterizationOf Tukey fences 18).
/// * 1 `RejectedNotDeterministic` — RANSAC residual proxy
///   (5408).
///
/// **T.12.d is the first proposal to exercise all five
/// panel-locked court-delta categories** (CanonicalAddition
/// from T.12.0, ExistingCanonicalAuthorityResolution +
/// DomainTransferOf + RejectedNotDeterministic from T.12.b,
/// ParameterizationOf from T.12.c). The wire-name set is now
/// closed at five.
fn robust_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 4 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(THEIL_SEN_RESERVED_CANONICAL_ID),
            reason: "Theil-Sen slope estimator: pair-median slope functional; \
                 structurally distinct from any SEED robust statistic. Declared \
                 estimator law: pair-selection rule (all-pairs / window-pair / \
                 sampled-pair), slope-median rule (exact / odd-count midpoint), \
                 tie-break law for equal-x pairs, window law for the input \
                 sequence. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID),
            reason: "Biweight midvariance: Tukey biweight scale estimator with \
                 continuous descending weight. Different decision functional from \
                 MAD (hard median deviation). Declared estimator law: tuning \
                 constant c, iteration count / convergence threshold, numeric \
                 mode for accumulator. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(TRIMMED_MEAN_RESERVED_CANONICAL_ID),
            reason: "Trimmed mean shift: drops a declared fraction of highest / \
                 lowest values before averaging. Declared estimator law: trim \
                 fraction alpha (symmetric / one-sided), percentile vs count \
                 semantics for cut points, tie-break law at trim boundary. \
                 Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(WINSORIZED_MEAN_RESERVED_CANONICAL_ID),
            reason: "Winsorized mean shift: REPLACES a declared fraction of \
                 highest / lowest values with closest non-trimmed values (vs \
                 trimmed mean's drop). Declared estimator law: winsor limit, \
                 replacement-rule (boundary-value), tie-break law. \
                 Deterministic.",
        },
        // -- 3 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            reason: "Robust z-score (median / MAD) stays canonical at SEED id 6. \
                 The RobustStatistics source class recognises robust-z as the \
                 authoritative robust-location estimator; no duplicate is \
                 admitted. The MAD outlier detector, median-MAD z-score, and \
                 robust-z alias names all collapse here. Declared estimator law: \
                 median + MAD + threshold multiplier, declared window.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(HAMPEL_SEED_ID),
            reason: "Hampel filter stays canonical at SEED id 7. Declared \
                 estimator law: windowed local median + MAD + threshold + \
                 replacement / rejection rule for outliers within the window. No \
                 duplicate is admitted; rolling-Hampel and Hampel-MAD-with- \
                 replacement aliases collapse here as ParameterizationOf records.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(TUKEY_FENCES_SEED_ID),
            reason: "Tukey fences stays canonical at SEED id 18. Declared \
                 estimator law: quartile estimator (linear interpolation / nearest \
                 / Hyndman-Fan #7 default), IQR multiplier (Tukey's 1.5 inner / \
                 3.0 outer), inclusive / exclusive fence semantics, tie-handling \
                 at quartile boundaries. No duplicate is admitted; k x IQR \
                 variants collapse here as ParameterizationOf records.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            reason: "Robust z-score (SEED id 6) is the most fundamental robust- \
                 location-estimator primitive and the shared reference the \
                 RobustStatistics source class transfers in. The court records \
                 the domain transfer without re-canonicalising robust-z. Every \
                 other SEED robust-statistics record above is recognised under \
                 ExistingCanonicalAuthorityResolution rather than \
                 DomainTransferOf to keep the wire-name set minimal (one \
                 DomainTransferOf per source-class pair).",
        },
        // -- 3 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(MODIFIED_Z_RESERVED_PRIMITIVE_ID),
            reason: "Modified z-score is ParameterizationOf(robust-z, SEED id 6). \
                 The 0.6745 / MAD scaling constant is the modified-z-specific \
                 parameterization that calibrates MAD to match the normal- \
                 distribution standard deviation; the family-level decision \
                 functional is robust z-score's. The court declines to admit \
                 modified z-score as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID),
            reason: "Rolling Hampel is ParameterizationOf(Hampel, SEED id 7). The \
                 rolling-window length is the parameterization; the family-level \
                 decision functional (local median + MAD + threshold + \
                 replacement) is the Hampel filter's. The court declines to \
                 admit rolling Hampel as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(K_IQR_FENCE_RESERVED_PRIMITIVE_ID),
            reason: "k x IQR fence is ParameterizationOf(Tukey fences, SEED id \
                 18). The IQR-multiplier value k is the parameterization; the \
                 family-level decision functional (quartile + multiplier x IQR \
                 + fence comparison) is Tukey fences'. The court declines to \
                 admit k x IQR fence as a new canonical primitive.",
        },
        // -- 1 RejectedNotDeterministic record ---------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(RANSAC_RESERVED_PRIMITIVE_ID),
            reason: "RANSAC residual proxy (Fischler & Bolles 1981) is randomized \
                 in its literature definition: uniformly random sample selection \
                 over iteration budget. Rejected as a literature-original \
                 canonical primitive for this deterministic corpus unless \
                 reduced to a declared deterministic proxy (sample seed, \
                 iteration budget, fixed sample schedule, tie-break law, numeric \
                 mode all brutally explicit) in a later T.12.x proposal. \
                 Deterministic stance: the rejection is on randomisation alone; \
                 the residual / inlier-counting functional itself is \
                 deterministic given a fixed sample.",
        },
    ]
}

/// Seven genealogy edges proposed for the post-freeze graph.
/// The four new canonicals are recorded as DerivedFrom robust-z
/// (SEED 6) reflecting the shared robust-statistics-family
/// ancestry. The three parameterizations carry
/// ParameterVariantOf edges to their parent canonicals.
fn robust_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(THEIL_SEN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(TRIMMED_MEAN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WINSORIZED_MEAN_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MODIFIED_Z_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(ROBUST_Z_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(HAMPEL_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(K_IQR_FENCE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(TUKEY_FENCES_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Eight source refs supporting the robust expansion: one per
/// new canonical (4) + one per parameterization shell (3) +
/// the RANSAC reference for the rejection record.
fn robust_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "theil_sen_1950",
            title: "A Rank-Invariant Method of Linear and Polynomial Regression Analysis",
            year: 1950,
            venue: "Proceedings of the Koninklijke Nederlandse Akademie van Wetenschappen 53",
        },
        ProposedSourceRef {
            citation_key: "sen_theil_extension_1968",
            title: "Estimates of the Regression Coefficient Based on Kendall's Tau",
            year: 1968,
            venue: "Journal of the American Statistical Association 63(324)",
        },
        ProposedSourceRef {
            citation_key: "mosteller_tukey_biweight_1977",
            title: "Data Analysis and Regression: A Second Course in Statistics",
            year: 1977,
            venue: "Addison-Wesley (Tukey biweight scale estimator chapter)",
        },
        ProposedSourceRef {
            citation_key: "tukey_trimmed_1962",
            title: "The Future of Data Analysis",
            year: 1962,
            venue: "Annals of Mathematical Statistics 33(1)",
        },
        ProposedSourceRef {
            citation_key: "dixon_winsorized_1960",
            title: "Simplified Estimation from Censored Normal Samples",
            year: 1960,
            venue: "Annals of Mathematical Statistics 31(2)",
        },
        ProposedSourceRef {
            citation_key: "iglewicz_hoaglin_modified_z_1993",
            title: "How to Detect and Handle Outliers (modified z-score)",
            year: 1993,
            venue: "ASQC Quality Press (Statistical Process Control series)",
        },
        ProposedSourceRef {
            citation_key: "davies_gather_hampel_1993",
            title: "The Identification of Multiple Outliers",
            year: 1993,
            venue: "Journal of the American Statistical Association 88(423)",
        },
        ProposedSourceRef {
            citation_key: "fischler_bolles_ransac_1981",
            title: "Random Sample Consensus: A Paradigm for Model Fitting (rejection-shell reference; RANSAC is randomized in origin)",
            year: 1981,
            venue: "Communications of the ACM 24(6)",
        },
    ]
}

/// Build the robust `DedupCourtDelta`: the per-batch dedup
/// outcome.
///
/// * `new_canonical_records` — the FOUR genuinely new canonicals
///   only (Theil-Sen, biweight midvariance, trimmed mean shift,
///   winsorized mean shift). EDDM-style and KSWIN-style
///   ParameterizationOf shells (modified z-score, rolling
///   Hampel, k×IQR fence) are deliberately absent from this
///   list; RANSAC is also deliberately absent (rejected).
/// * `new_alias_records` — empty.
/// * `new_composition_records` — empty.
/// * `rejection_records` — empty at the delta level. RANSAC's
///   rejection is encoded via the `RejectedNotDeterministic`
///   entry in `proposed_dedup_records` plus the proposed-
///   primitive shell, mirroring the BOCPD pattern from T.12.b.
/// * `deferred_records` — empty at the delta level.
fn build_robust_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_d_robust_delta",
        vec![
            DetectorCanonicalId(THEIL_SEN_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(TRIMMED_MEAN_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(WINSORIZED_MEAN_RESERVED_CANONICAL_ID),
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

/// Build the T.12.d robust `CorpusAmendmentProposal`. Two builds
/// against this static seed produce byte-identical bytes.
///
/// **Panel-locked seed posture**:
///
/// * `status = Open` — proposal is filed but the court has not
///   ratified.
/// * `proposer_role = PanelMember` — filed by the same panel
///   that authored T.12.0 / T.12.a / T.12.b / T.12.c.
/// * `created_at_commit = "t12_d_robust"` — placeholder; a
///   future formal freeze campaign cites the actual commit
///   hash.
#[must_use]
pub fn seed_t12_d_robust_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_d_robust_first_proposal",
        "T.12.d files the Robust Statistics amendment proposal. Adds four \
         genuinely new canonical primitives (Theil-Sen slope estimator, biweight \
         midvariance, trimmed mean shift, winsorized mean shift) at reserved \
         canonical ids 5401..=5404 with declared estimator-law contracts \
         (pair-selection / tuning constant / trim fraction / winsor limit). \
         Records three ExistingCanonicalAuthorityResolution decisions keeping \
         robust z-score (SEED id 6), Hampel filter (id 7), Tukey fences (id 18) \
         canonical under the RobustStatistics source class without duplication. \
         Records one DomainTransferOf decision naming robust z-score as the \
         shared robust-location-estimator ancestor. Records three \
         ParameterizationOf decisions: modified z-score is \
         ParameterizationOf(robust-z); rolling Hampel is ParameterizationOf \
         (Hampel); k x IQR fence is ParameterizationOf(Tukey fences) - all \
         three appear in proposed_primitives but NOT in new_canonical_records. \
         Rejects RANSAC residual proxy (Fischler & Bolles 1981) as \
         RejectedNotDeterministic at reserved id 5408 - RANSAC is randomized \
         in origin and admitted neither to SEED nor to new_canonical_records \
         unless a future T.12.x proposal admits a Deterministic_RANSAC_Proxy \
         with the sample seed, iteration budget, sample schedule, tie-break \
         law, and numeric mode declared. T.12.d is the first proposal to \
         exercise ALL FIVE panel-locked court-delta categories \
         (CanonicalAddition, ExistingCanonicalAuthorityResolution, \
         DomainTransferOf, ParameterizationOf, RejectedNotDeterministic). \
         Does NOT mutate SEED (SEED.len() stays at 54); status = Open pending \
         review.",
        SourceClass::RobustStatistics,
        build_robust_expansion_batch(),
        build_robust_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_d_robust",
    )
}
