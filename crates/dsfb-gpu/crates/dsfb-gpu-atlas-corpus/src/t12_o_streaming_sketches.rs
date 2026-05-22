//! T.12.o — Streaming Sketches: the fifteenth real literature
//! expansion proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.o files the Streaming Sketches amendment proposal.
//! > It admits only deterministic streaming-sketch witnesses:
//! > bounded-memory, mergeable or update-order-declared
//! > summaries for frequency, cardinality, quantile, heavy-
//! > hitter, membership, and moment / variance evidence whose
//! > hash family, width, depth, seed, bucket count, merge law,
//! > update order, error-bound semantics, residual definition,
//! > decision functional, confuser profile, and numeric mode
//! > are declared; resolves SEED collisions with KS / Missingness
//! > spike / Error burst / Cardinality drift; classifies
//! > variants as parameterizations or domain transfers; rejects
//! > learned streaming-anomaly scores and black-box approximate-
//! > streaming proprietary sketches without declared hash /
//! > width / depth / seed / merge contract; and preserves the
//! > frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"A streaming-sketch
//! witness is admissible only when the hash family, width,
//! depth, seed, bucket count, merge law, update order, error-
//! bound semantics, residual definition, decision functional,
//! confuser profile, and numeric mode are declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.o admits deterministic streaming-sketch witnesses:
//! > bounded-memory, mergeable or update-order-declared
//! > summaries for frequency, cardinality, quantile, heavy-
//! > hitter, membership, and moment / variance evidence. It
//! > does not admit probabilistic accuracy as certainty,
//! > randomized sketch behavior without seed / width / depth /
//! > hash-family declaration, privacy claims, database
//! > correctness authority, or approximate-query truth.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.o's design began with a grep of [`crate::seed::SEED`]
//! for every streaming-sketch candidate. The walk found
//! **four** T.12.o-relevant primitives already canonical:
//!
//! * **Kolmogorov-Smirnov two-sample test** at SEED id 8 —
//!   shared distribution-distance ancestor; streaming-
//!   approximate KS variants reduce to a sketch-
//!   parameterization of this primitive.
//! * **Missingness spike** at SEED id 13 — Bloom-filter-based
//!   missingness inversion reduces to a sketch-parameterization
//!   of this primitive.
//! * **Error burst** at SEED id 41 — sliding-window heavy-
//!   hitter sketches over error-event streams reduce to a
//!   sketch-parameterization of this primitive.
//! * **Cardinality drift** at SEED id 46 — pre-HLL cardinality
//!   estimators (Flajolet-Martin 1985 probabilistic counting,
//!   LogLog Durand-Flajolet 2003) reduce to a sketch-
//!   parameterization of this primitive.
//!
//! All four become `ExistingCanonicalAuthorityResolution`
//! records under the `StreamingSketches` source class.
//! **Panel-locked success-shape** (mirroring T.12.k / T.12.l /
//! T.12.m / T.12.n): the campaign's strength comes from
//! cross-class dedup discipline (4 authority resolutions over
//! the KS + missingness + error-burst + cardinality SEED
//! family that streaming-sketch summaries heavily reuse), not
//! detector count.
//!
//! Eight genuinely new canonicals at reserved ids 6501..=6508
//! survived the SEED-walk as structurally distinct streaming-
//! sketch decision functionals. Each declares an explicit
//! contract over hash family, width, depth, seed, bucket
//! count, merge law (where mergeable), update order, error-
//! bound semantics, and decision-functional residual; each is
//! a SKETCH-STATE decision (not a raw-stream decision):
//!
//! * **Count-Min sketch residual witness** (6501; Cormode-
//!   Muthukrishnan 2005 An Improved Data Stream Summary: the
//!   Count-Min Sketch and its Applications). Declared 2-wise-
//!   or-better-independent hash family + width w + depth d +
//!   per-row seed array + collision-resolution law (min over d
//!   rows). Sketch-state decision on per-key estimated count
//!   residual vs nominal baseline.
//! * **HyperLogLog cardinality shift witness** (6502; Flajolet
//!   / Fusy / Gandouet / Meunier 2007 HyperLogLog: the
//!   analysis of a near-optimal cardinality estimation
//!   algorithm). Declared hash family + bucket count m =
//!   2^precision + per-bucket leading-zero register + harmonic-
//!   mean estimator with bias correction. Sketch-state decision
//!   on per-window HLL cardinality estimate shift relative to
//!   baseline.
//! * **Bloom-filter membership anomaly witness** (6503; Bloom
//!   1970 Space/Time Trade-offs in Hash Coding with Allowable
//!   Errors). Declared hash family + bit-array size m + hash
//!   count k + seed array + declared false-positive-rate
//!   envelope (probabilistic — explicitly NOT a deterministic
//!   certainty claim). Sketch-state decision on per-window
//!   sketch-membership-test rate shift.
//! * **Misra-Gries heavy-hitter shift witness** (6504; Misra-
//!   Gries 1982 Finding Repeated Elements). Declared k counter
//!   slots + decrement-on-miss law (decrement all counters by 1
//!   on miss after k slots full; deterministic, no hash). Sketch-
//!   state decision on per-window heavy-hitter set shift
//!   relative to baseline.
//! * **Space-Saving heavy-hitter shift witness** (6505; Metwally
//!   / Agrawal / El Abbadi 2005 Efficient Computation of
//!   Frequent and Top-k Elements in Data Streams). Declared k
//!   counter slots + replace-smallest-on-miss law (replace the
//!   smallest-counter element with the new element + 1;
//!   deterministic, no hash). Sketch-state decision on per-
//!   window heavy-hitter set shift. Structurally distinct from
//!   Misra-Gries 6504 because the bookkeeping rule differs
//!   (replace-smallest vs decrement-all).
//! * **Greenwald-Khanna quantile summary drift witness** (6506;
//!   Greenwald-Khanna 2001 Space-Efficient Online Computation
//!   of Quantile Summaries). Declared epsilon error bound +
//!   tuple-insertion rule + tuple-merging law + bounded-tuple-
//!   count bound (O(log(epsilon * N) / epsilon)). Sketch-state
//!   decision on per-window epsilon-approximate quantile shift.
//! * **t-digest summary residual witness** (6507; Dunning 2019
//!   Computing Extremely Accurate Quantiles Using t-Digests).
//!   Declared centroid scale function (k_1 / k_2 / k_3) +
//!   compression delta + buffer-size + DETERMINISTIC centroid-
//!   merge law (commutative merge of centroids ordered by mean;
//!   NOT randomized). Sketch-state decision on per-window
//!   centroid distribution residual vs baseline.
//! * **Alon-Matias-Szegedy (AMS) moment sketch witness** (6508;
//!   Alon-Matias-Szegedy 1999 The Space Complexity of
//!   Approximating the Frequency Moments). Declared 4-wise-
//!   independent hash family, per-sketch seed, sketch width,
//!   moment order p, and signed-update rule (h(x) in {-1, +1}
//!   drawn from the 4-wise independent family). Sketch-state
//!   decision on per-window estimated p-th moment shift.
//!
//! Two domain transfers (panel-locked):
//!
//! * **Cardinality drift** (SEED 46) → `DomainTransferOf` as
//!   the shared cardinality ancestor for `StreamingSketches`
//!   (HLL cardinality shift 6502 plus Flajolet-Martin pre-HLL
//!   probabilistic counting 6509 are descendants).
//! * **Kolmogorov-Smirnov two-sample test** (SEED 8) →
//!   `DomainTransferOf` as the shared distribution-distance
//!   ancestor for `StreamingSketches` (sketch-approximate KS
//!   6510 is the streaming descendant).
//!
//! Four parameterizations (panel-candidate primitives that
//! collapsed on closer inspection):
//!
//! * **Flajolet-Martin / probabilistic counting / LogLog
//!   cardinality estimator** (6509; Flajolet-Martin 1985
//!   Probabilistic Counting Algorithms for Data Base
//!   Applications; Durand-Flajolet 2003 LogLog Counting of
//!   Large Cardinalities) → `ParameterizationOf(Cardinality
//!   drift, SEED 46)` with declared hash family + leading-
//!   zero / trailing-zero register + cardinality-estimator
//!   law + sketch-state decision rule. Pre-HLL cardinality
//!   sketches collapse here; HLL 6502 is the structurally-
//!   distinct successor with harmonic-mean estimator.
//! * **Streaming approximate KS via quantile sketch** (6510)
//!   → `ParameterizationOf(Kolmogorov-Smirnov two-sample
//!   test, SEED 8)` with declared quantile-sketch summary
//!   (Greenwald-Khanna or t-digest) + per-window KS distance
//!   estimate + sketch-error-budget contract.
//! * **Sliding-window error-burst sketch** (6511) →
//!   `ParameterizationOf(Error burst, SEED 41)` with declared
//!   sliding-window Count-Min or heavy-hitter sketch + window
//!   length + error-event family + per-window burst-threshold
//!   law.
//! * **Sketch-approximate missingness via Bloom-filter
//!   inversion** (6512) → `ParameterizationOf(Missingness
//!   spike, SEED 13)` with declared Bloom-filter membership
//!   contract + per-window expected-not-seen count + sketch-
//!   estimate-of-missingness-rate decision.
//!
//! Two rejections (ninth T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, h, i,
//! j, k, l, m, n):
//!
//! * **Learned streaming-anomaly score / online-learning
//!   detector** (6513) — `RejectedNotDeterministic`. Learned
//!   online-anomaly pipelines (Datadog Watchdog AI, DataRobot
//!   Streaming AutoML, Splunk Stream ML, AWS Lookout for
//!   Metrics, Azure Anomaly Detector streaming endpoint)
//!   expose anomaly scores from continuously-updated learned
//!   embeddings without a deterministic feature-extraction
//!   law, declared update rule, declared training-data anchor,
//!   declared tie-break law, or declared numeric mode.
//!   Admission requires a future T.12.x to admit a
//!   `Deterministic_Streaming_Anomaly_Proxy` canonical with
//!   deterministic feature-extraction law, declared formula,
//!   declared update rule with fixed step size and clipping
//!   law, declared training-data anchor, feature schema, tie-
//!   break, numeric mode, and no learned opaque embedding.
//! * **Black-box approximate-streaming proprietary sketch
//!   without declared hash / width / depth / seed / merge
//!   contract** (6514) — `RejectedNotDeterministic`. Vendor
//!   approximate-streaming sketches (commercial OLAP vendors
//!   like Snowflake APPROX_*, BigQuery APPROX_*, Druid
//!   approximate aggregators, ClickHouse approximate
//!   aggregators) often expose APPROX_COUNT_DISTINCT /
//!   APPROX_QUANTILES / APPROX_TOP_COUNT without declaring the
//!   underlying sketch's hash family, width / bucket count,
//!   depth, seed, or merge law. The court does NOT admit
//!   these as canonical witnesses; the decision functional
//!   cannot be replayed without the contract. Admission
//!   requires a future T.12.x to admit a
//!   `Deterministic_Vendor_Sketch_Proxy` canonical only if
//!   the vendor publishes the hash family + width + depth +
//!   seed + merge law + error-bound semantics or the user
//!   declares them at deployment time.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×8 (6501..=6508).
//! * `ExistingCanonicalAuthorityResolution` ×4 — SEED 8, 13,
//!   41, 46.
//! * `DomainTransferOf` ×2 — SEED 46 (cardinality ancestor) +
//!   SEED 8 (distribution-distance ancestor).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 8 + 4 + 2 + 4 + 2 = **20 dedup-court records**.
//!
//! ## Probabilistic-bound / approximate-query-truth / privacy /
//! anonymization discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND
//! ExistingCanonicalAuthorityResolution reason text MUST
//! describe its record as a "streaming-sketch witness" with
//! explicit declaration that sketch-state outputs are
//! PROBABILISTIC ESTIMATES under a declared error-bound
//! contract, NEVER as deterministic-certainty / approximate-
//! query-truth / privacy-preserving / anonymization-authority
//! claims. The dedicated load-bearing negatives scan every
//! such reason for forbidden terms across four parametric
//! scanners:
//!
//! - probabilistic-bound-as-deterministic-certainty terms
//!   (deterministic accuracy bound, deterministic count
//!   certainty, exact within probabilistic error, sketch
//!   estimate is exact);
//! - approximate-query-truth terms (approximate query truth,
//!   approximate count is exact, sketch query verdict,
//!   database correctness verdict);
//! - privacy / anonymization claim terms (anonymization
//!   authority, differential privacy guarantee, privacy-
//!   preserving certainty, k-anonymous output);
//! - mergeable-sketch-without-merge-law terms (every
//!   "mergeable" claim must accompany a declared merge law).
//!
//! Forbidden terms appear ONLY in `RejectedNotDeterministic`
//! reason text (where they describe what is NOT admitted).
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.o streaming-sketches
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
// Reserved id constants (panel-locked, 6501..=6514 used;
// 6515..=6599 reserved for future Streaming Sketches proposals)
// ---------------------------------------------------------------

/// Reserved canonical id for Count-Min sketch residual witness
/// (Cormode-Muthukrishnan 2005). Declared 2-wise-or-better-
/// independent hash family + width w + depth d + per-row seed
/// array + min-over-d collision-resolution law.
pub const CMS_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 6501;

/// Reserved canonical id for HyperLogLog cardinality shift
/// witness (Flajolet-Fusy-Gandouet-Meunier 2007). Declared
/// hash family + bucket count m = 2^precision + harmonic-mean
/// estimator with bias correction.
pub const HLL_RESERVED_CANONICAL_ID: u32 = 6502;

/// Reserved canonical id for Bloom-filter membership anomaly
/// witness (Bloom 1970). Declared hash family + bit-array
/// size m + hash count k + seed array + false-positive-rate
/// envelope (PROBABILISTIC, explicitly NOT deterministic
/// certainty).
pub const BLOOM_RESERVED_CANONICAL_ID: u32 = 6503;

/// Reserved canonical id for Misra-Gries heavy-hitter shift
/// witness (Misra-Gries 1982). Declared k counter slots +
/// decrement-on-miss law. No hash; deterministic counts.
pub const MISRA_GRIES_RESERVED_CANONICAL_ID: u32 = 6504;

/// Reserved canonical id for Space-Saving heavy-hitter shift
/// witness (Metwally-Agrawal-El Abbadi 2005). Declared k
/// counter slots + replace-smallest-on-miss law. Structurally
/// distinct from Misra-Gries because the bookkeeping rule
/// differs.
pub const SPACE_SAVING_RESERVED_CANONICAL_ID: u32 = 6505;

/// Reserved canonical id for Greenwald-Khanna quantile
/// summary drift witness (Greenwald-Khanna 2001). Declared
/// epsilon error bound + tuple-insertion rule + tuple-merging
/// law.
pub const GREENWALD_KHANNA_RESERVED_CANONICAL_ID: u32 = 6506;

/// Reserved canonical id for t-digest summary residual
/// witness (Dunning 2019). Declared centroid scale function +
/// compression delta + buffer-size + DETERMINISTIC centroid-
/// merge law.
pub const T_DIGEST_RESERVED_CANONICAL_ID: u32 = 6507;

/// Reserved canonical id for Alon-Matias-Szegedy (AMS) moment
/// sketch witness (Alon-Matias-Szegedy 1999). Declared 4-wise-
/// independent hash family + per-sketch seed + sketch width +
/// moment order p + signed-update rule.
pub const AMS_RESERVED_CANONICAL_ID: u32 = 6508;

/// Reserved id for Flajolet-Martin / probabilistic-counting /
/// LogLog cardinality estimator. `ParameterizationOf
/// (Cardinality drift, SEED 46)`.
pub const FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID: u32 = 6509;

/// Reserved id for Streaming approximate KS via quantile
/// sketch. `ParameterizationOf(Kolmogorov-Smirnov two-sample
/// test, SEED 8)`.
pub const STREAMING_KS_RESERVED_PRIMITIVE_ID: u32 = 6510;

/// Reserved id for Sliding-window error-burst sketch.
/// `ParameterizationOf(Error burst, SEED 41)`.
pub const SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID: u32 = 6511;

/// Reserved id for Sketch-approximate missingness via Bloom-
/// filter inversion. `ParameterizationOf(Missingness spike,
/// SEED 13)`.
pub const BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID: u32 = 6512;

/// Reserved id for Learned streaming-anomaly score / online-
/// learning detector. `RejectedNotDeterministic`.
pub const LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID: u32 = 6513;

/// Reserved id for Black-box approximate-streaming proprietary
/// sketch without declared hash / width / depth / seed / merge
/// contract. `RejectedNotDeterministic`.
pub const BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID: u32 = 6514;

// Existing SEED canonical ids referenced by T.12.o.

/// Kolmogorov-Smirnov two-sample test — SEED canonical id 8.
/// Shared distribution-distance ancestor; streaming-approximate
/// KS variants reduce to a sketch-parameterization here.
pub const KS_SEED_ID: u32 = 8;

/// Missingness spike — SEED canonical id 13. Bloom-filter-
/// based missingness inversion reduces to a sketch-
/// parameterization here.
pub const MISSINGNESS_SPIKE_SEED_ID: u32 = 13;

/// Error burst — SEED canonical id 41. Sliding-window heavy-
/// hitter sketches over error-event streams reduce to a
/// sketch-parameterization here.
pub const ERROR_BURST_SEED_ID: u32 = 41;

/// Cardinality drift — SEED canonical id 46. Pre-HLL
/// cardinality estimators (Flajolet-Martin 1985, LogLog
/// Durand-Flajolet 2003) reduce to a sketch-parameterization
/// here.
pub const CARDINALITY_DRIFT_SEED_ID: u32 = 46;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------

/// `CanonicalAddition`.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution`.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf`.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf`.
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

/// `RejectedNotDeterministic`.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

// ---------------------------------------------------------------
// Builders for the streaming-sketches expansion batch
// ---------------------------------------------------------------

/// Build the streaming-sketches `CorpusExpansionBatch` body.
fn build_streaming_sketches_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_o_streaming_sketches_first_proposal",
        SourceClass::StreamingSketches,
        streaming_sketches_proposed_primitives(),
        streaming_sketches_proposed_aliases(),
        streaming_sketches_proposed_dedup_records(),
        streaming_sketches_proposed_genealogy_edges(),
        streaming_sketches_proposed_source_refs(),
    )
}

/// Fourteen proposed primitives: 8 canonical (the streaming-
/// sketch primitives that survived SEED-walk as structurally
/// distinct sketch-state decision functionals) + 4
/// parameterization shells + 2 rejection shells. The "tight
/// canonical set, heavy contract discipline around hash family
/// / width / depth / seed / merge law / error-bound semantics,
/// clear rejection of learned streaming anomaly scores AND
/// black-box vendor sketches without contract declaration"
/// shape applies the panel-locked T.12.k / T.12.l / T.12.m /
/// T.12.n success posture to streaming sketches.
fn streaming_sketches_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CMS_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "Count-Min sketch residual witness",
            motivation: "Count-Min sketch residual streaming-sketch witness \
                 (Cormode-Muthukrishnan 2005). Required contract: hash family \
                 (declared 2-wise-or-better-independent family), width w, \
                 depth d, per-row seed array, collision-resolution law (min \
                 over d rows), merge law (cell-wise addition; mergeable when \
                 the two sketches share hash family + width + depth + seed), \
                 update order (deterministic per-element update; non-mergeable \
                 windowing requires declared sliding-window contract), error- \
                 bound semantics (probabilistic epsilon-delta bound; \
                 explicitly NOT a deterministic accuracy guarantee), residual \
                 definition (per-key estimated count residual vs nominal \
                 baseline), decision functional (per-window residual exceeds \
                 epsilon-derived threshold), confuser profile (hash-collision \
                 over-count, adversarial inputs targeting hash family, \
                 stationarity violation), numeric mode. Sketch-state decision \
                 only; the court does NOT admit approximate-query truth or \
                 database-correctness verdicts.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HLL_RESERVED_CANONICAL_ID),
            display_name: "HyperLogLog cardinality shift witness",
            motivation: "HyperLogLog cardinality shift streaming-sketch witness \
                 (Flajolet-Fusy-Gandouet-Meunier 2007). Required contract: \
                 hash family (declared uniform-output hash family with stated \
                 bit-width), bucket count m = 2^precision with declared \
                 precision parameter, per-bucket leading-zero-count register, \
                 estimator law (harmonic-mean of 2^register values with \
                 alpha_m bias correction and small-range / large-range \
                 corrections per Flajolet et al.), merge law (per-bucket max \
                 of register values; mergeable when sketches share hash \
                 family + precision), update order, error-bound semantics \
                 (probabilistic standard error 1.04 / sqrt(m); explicitly NOT \
                 a deterministic accuracy guarantee), residual definition \
                 (per-window HLL cardinality estimate residual vs baseline), \
                 decision functional (per-window estimate shift exceeds \
                 standard-error-derived threshold), confuser profile (hash- \
                 collision under-count for small cardinality, repeated-element \
                 stream confusion, precision-truncation bias), numeric mode. \
                 Sketch-state decision only; the court does NOT admit \
                 deterministic cardinality certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BLOOM_RESERVED_CANONICAL_ID),
            display_name: "Bloom-filter membership anomaly witness",
            motivation: "Bloom-filter membership anomaly streaming-sketch \
                 witness (Bloom 1970). Required contract: hash family \
                 (declared independent hash family with stated output range), \
                 bit-array size m, hash count k, seed array, declared FALSE- \
                 POSITIVE-RATE ENVELOPE (probabilistic; explicitly NOT a \
                 deterministic certainty claim; false negatives are zero by \
                 construction but false positives are probabilistic), merge \
                 law (bit-wise OR; mergeable when sketches share hash family \
                 + size + hash count + seed), update order, error-bound \
                 semantics (false-positive rate (1 - exp(-kn/m))^k), residual \
                 definition (per-window sketch-membership-test rate residual \
                 vs baseline), decision functional (per-window sketch- \
                 membership rate shift exceeds false-positive-envelope- \
                 derived threshold), confuser profile (filter saturation, \
                 adversarial input, hash-family collision), numeric mode. \
                 Sketch-state decision only; the court does NOT admit \
                 deterministic membership certainty or privacy-preserving \
                 authority.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MISRA_GRIES_RESERVED_CANONICAL_ID),
            display_name: "Misra-Gries heavy-hitter shift witness",
            motivation: "Misra-Gries heavy-hitter shift streaming-sketch \
                 witness (Misra-Gries 1982). Required contract: width \
                 (declared k counter slots; no hash family because counts \
                 are deterministic; no seed required), depth (1), decrement- \
                 on-miss law (on miss after k slots full, decrement all \
                 counters by 1; deterministic), merge law (combine counter \
                 lists then prune to k slots via repeated decrement; \
                 mergeable but the merge increases approximation error), \
                 update order (declared per-element update order; the \
                 sketch is order-sensitive within the decrement-merge step), \
                 error-bound semantics (every element with frequency > N/k \
                 is guaranteed to appear in the final k slots; explicitly a \
                 ONE-SIDED guarantee, NOT exact count certainty), residual \
                 definition (per-window heavy-hitter set residual vs \
                 baseline), decision functional (per-window heavy-hitter \
                 set shift), confuser profile (stationarity violation, \
                 adversarial element ordering, k undersized for actual \
                 heavy-hitter count), numeric mode. Sketch-state decision \
                 only; the court does NOT admit exact frequency certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SPACE_SAVING_RESERVED_CANONICAL_ID),
            display_name: "Space-Saving heavy-hitter shift witness",
            motivation: "Space-Saving heavy-hitter shift streaming-sketch \
                 witness (Metwally-Agrawal-El Abbadi 2005). Required \
                 contract: width (declared k counter slots; no hash family \
                 because counts are deterministic; no seed required), depth \
                 (1), replace-smallest-on-miss law (on miss after k slots \
                 full, replace the smallest-counter element with the new \
                 element and set its counter to smallest-counter + 1; \
                 deterministic), merge law (combine counter lists then \
                 prune to k slots via repeated replace-smallest), update \
                 order, error-bound semantics (per-element over-estimate is \
                 bounded by the smallest counter at miss time; one-sided \
                 guarantee, NOT exact count certainty), residual definition, \
                 decision functional (per-window heavy-hitter set shift), \
                 confuser profile, numeric mode. Structurally distinct from \
                 Misra-Gries 6504 because Space-Saving uses replace-\
                 smallest-on-miss whereas Misra-Gries uses decrement-all-on-\
                 miss; the bookkeeping rule is different. Sketch-state \
                 decision only.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(GREENWALD_KHANNA_RESERVED_CANONICAL_ID),
            display_name: "Greenwald-Khanna quantile summary drift witness",
            motivation: "Greenwald-Khanna quantile summary drift streaming-\
                 sketch witness (Greenwald-Khanna 2001). Required contract: \
                 width (declared epsilon error bound; tuple count bounded \
                 by O(log(epsilon * N) / epsilon)), depth (1; no hash \
                 family; no seed required), tuple-insertion rule (insert \
                 with declared g_i and delta_i bookkeeping), tuple-merging \
                 law (merge adjacent tuples when g_i + g_{i+1} + delta_{i+1} \
                 <= 2 * epsilon * N; deterministic), merge law (combine \
                 tuple lists from two sketches then re-prune via the \
                 declared merging law; mergeable with degraded epsilon), \
                 update order, error-bound semantics (DETERMINISTIC epsilon-\
                 approximate quantile guarantee; the answer for any \
                 quantile phi is within epsilon * N of the true rank; \
                 explicitly a deterministic ONE-SIDED rank bound, NOT \
                 exact-quantile certainty), residual definition (per-window \
                 epsilon-approximate quantile shift vs baseline), decision \
                 functional, confuser profile, numeric mode. Sketch-state \
                 decision only; the court does NOT admit exact-quantile \
                 certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(T_DIGEST_RESERVED_CANONICAL_ID),
            display_name: "t-digest summary residual witness",
            motivation: "t-digest summary residual streaming-sketch witness \
                 (Dunning 2019). Required contract: width (declared \
                 compression delta; buffer-size bound on centroid count is \
                 O(delta * log(N))), depth (1; no hash family; no seed \
                 required), centroid scale function (declared k_1 / k_2 / \
                 k_3 scale function), DETERMINISTIC centroid-merge law \
                 (commutative merge of centroids ordered by mean; centroids \
                 with combined weight below the scale-function threshold \
                 are merged; explicitly NOT randomized — the centroid-merge \
                 ordering is deterministic given the scale function and \
                 ingestion order), merge law (deterministic union of \
                 centroid lists followed by re-compression), update order, \
                 error-bound semantics (declared compression delta governs \
                 the centroid count and the per-quantile error envelope; \
                 explicitly approximate, NOT deterministic certainty), \
                 residual definition (per-window centroid distribution \
                 residual vs baseline), decision functional, confuser \
                 profile, numeric mode. Sketch-state decision only.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(AMS_RESERVED_CANONICAL_ID),
            display_name: "Alon-Matias-Szegedy (AMS) moment sketch witness",
            motivation: "Alon-Matias-Szegedy (AMS) moment sketch streaming-\
                 sketch witness (Alon-Matias-Szegedy 1999). Required \
                 contract: hash family (declared 4-wise-independent hash \
                 family with output in {-1, +1}; the 4-wise independence \
                 is necessary for unbiased second-moment estimation), \
                 per-sketch seed, sketch width (number of independent \
                 estimators), depth (averaging over width / median of \
                 medians per AMS construction), moment order p, signed-\
                 update rule (per-element update X += h(element)), merge \
                 law (cell-wise addition; mergeable when sketches share \
                 hash family + width + seed), update order, error-bound \
                 semantics (probabilistic epsilon-delta bound on the p-th \
                 frequency moment; explicitly NOT a deterministic accuracy \
                 guarantee), residual definition (per-window estimated p-th \
                 moment residual vs baseline), decision functional, \
                 confuser profile (4-wise-independence violation, sketch \
                 width too small for declared epsilon-delta target, \
                 adversarial element ordering), numeric mode. Sketch-state \
                 decision only.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID),
            display_name:
                "Flajolet-Martin / probabilistic counting / LogLog - parameterization shell",
            motivation: "Probabilistic-counting parameterization of \
                 Cardinality drift (SEED id 46) with declared hash family + \
                 leading-zero or trailing-zero register + cardinality-\
                 estimator law (Flajolet-Martin 1985 Probabilistic Counting \
                 Algorithms; Durand-Flajolet 2003 LogLog Counting of Large \
                 Cardinalities; pre-HLL geometric-mean / arithmetic-mean \
                 estimators). The court rules: pre-HLL cardinality \
                 estimators are ParameterizationOf(Cardinality drift, SEED \
                 46), NOT a new canonical primitive; HLL (6502) is the \
                 structurally-distinct successor with harmonic-mean \
                 estimator and bias correction. Appears in \
                 proposed_primitives but NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(STREAMING_KS_RESERVED_PRIMITIVE_ID),
            display_name: "Streaming approximate KS via quantile sketch - parameterization shell",
            motivation: "Quantile-sketch parameterization of Kolmogorov-\
                 Smirnov two-sample test (SEED id 8) with declared \
                 quantile-sketch summary (Greenwald-Khanna 6506 or t-digest \
                 6507) + per-window KS distance estimate computed against \
                 the sketch's epsilon-approximate quantile function + \
                 sketch-error-budget contract. The court rules: streaming \
                 approximate KS is ParameterizationOf(Kolmogorov-Smirnov \
                 two-sample test, SEED 8), NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Sliding-window error-burst sketch - parameterization shell",
            motivation: "Sketch-windowing parameterization of Error burst \
                 (SEED id 41) with declared sliding-window Count-Min sketch \
                 (6501) or heavy-hitter sketch (Misra-Gries 6504 / Space-\
                 Saving 6505) over the error-event stream + declared \
                 window length + error-event family + per-window burst-\
                 threshold law. The court rules: sliding-window error-burst \
                 sketch is ParameterizationOf(Error burst, SEED 41), NOT a \
                 new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID,
            ),
            display_name:
                "Sketch-approximate missingness via Bloom inversion - parameterization shell",
            motivation: "Bloom-filter-inversion parameterization of \
                 Missingness spike (SEED id 13) with declared Bloom-filter \
                 membership contract (6503) + per-window expected-not-seen \
                 count + sketch-estimate-of-missingness-rate decision. The \
                 court rules: sketch-approximate missingness via Bloom \
                 inversion is ParameterizationOf(Missingness spike, SEED \
                 13), NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID,
            ),
            display_name:
                "Learned streaming-anomaly score / online-learning detector - rejected shell",
            motivation: "Learned online-anomaly pipelines (Datadog Watchdog \
                 AI, DataRobot Streaming AutoML, Splunk Stream ML, AWS \
                 Lookout for Metrics, Azure Anomaly Detector streaming \
                 endpoint) expose anomaly scores from continuously-updated \
                 learned embeddings without a deterministic feature-\
                 extraction law, declared update rule with fixed step size \
                 and clipping law, declared training-data anchor, declared \
                 tie-break law, or declared numeric mode. The court does \
                 NOT admit these to the dedup-court delta's \
                 new_canonical_records. A future T.12.x may admit a \
                 Deterministic_Streaming_Anomaly_Proxy canonical only if a \
                 deterministic feature-extraction law, declared formula, \
                 declared update rule with fixed step size and clipping \
                 law, declared training-data anchor, feature schema, tie-\
                 break, numeric mode, and no learned opaque embedding are \
                 all brutally explicit.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Black-box approximate-streaming proprietary sketch - rejected shell",
            motivation: "Black-box approximate-streaming proprietary sketches \
                 (Snowflake APPROX_COUNT_DISTINCT / APPROX_PERCENTILE / \
                 APPROX_TOP_K, BigQuery APPROX_COUNT_DISTINCT / \
                 APPROX_QUANTILES / APPROX_TOP_COUNT, Druid approximate \
                 aggregators, ClickHouse uniqHLL12 / quantileTDigest / \
                 topK approximate aggregators, AWS Athena APPROX_*) often \
                 expose results without declaring the underlying sketch's \
                 hash family, width / bucket count, depth, seed, or merge \
                 law. The court does NOT admit these as canonical witnesses \
                 because the decision functional cannot be replayed without \
                 the contract. A future T.12.x may admit a \
                 Deterministic_Vendor_Sketch_Proxy canonical only if the \
                 vendor publishes the hash family + width + depth + seed + \
                 merge law + error-bound semantics, or if the user declares \
                 them at deployment time and pins the deployment \
                 configuration in a fixed receipt.",
        },
    ]
}

/// Zero alias claims (T.12.o routes everything through dedup
/// records and existing-canonical authority resolutions).
fn streaming_sketches_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty dedup-court decisions on the streaming-sketches batch.
fn streaming_sketches_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CMS_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "Count-Min sketch residual witness (Cormode-Muthukrishnan \
                 2005): declared hash family (2-wise-or-better-independent) + \
                 width w + depth d + per-row seed array + collision-resolution \
                 law (min over d rows) + merge law (cell-wise addition; \
                 mergeable when sketches share hash family + width + depth \
                 + seed) + update order + error-bound semantics \
                 (probabilistic epsilon-delta bound) + residual definition + \
                 decision functional + confuser profile + numeric mode. \
                 Streaming-sketch witness; the court does NOT admit \
                 approximate-query truth or database-correctness verdicts.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(HLL_RESERVED_CANONICAL_ID),
            reason: "HyperLogLog cardinality shift witness (Flajolet-Fusy-\
                 Gandouet-Meunier 2007): declared hash family + bucket count \
                 m = 2^precision with declared precision parameter + per-\
                 bucket leading-zero-count register + harmonic-mean \
                 estimator with bias correction + merge law (per-bucket max \
                 of register values) + update order + error-bound semantics \
                 (probabilistic standard error 1.04 / sqrt(m)) + residual \
                 definition + decision functional + confuser profile + \
                 numeric mode. Sketch-state cardinality-estimate decision \
                 only; the court does NOT admit deterministic cardinality \
                 certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BLOOM_RESERVED_CANONICAL_ID),
            reason: "Bloom-filter membership anomaly witness (Bloom 1970): \
                 declared hash family + bit-array size m + hash count k + \
                 seed array + declared probabilistic false-positive-rate \
                 envelope + merge law (bit-wise OR) + update order + error-\
                 bound semantics + residual definition + decision \
                 functional + confuser profile + numeric mode. Streaming-\
                 sketch membership witness; the court does NOT admit \
                 deterministic membership certainty, privacy-preserving \
                 authority, or anonymization authority.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MISRA_GRIES_RESERVED_CANONICAL_ID),
            reason: "Misra-Gries heavy-hitter shift witness (Misra-Gries \
                 1982): declared k counter slots (no hash family; counts \
                 are deterministic; no seed required) + depth 1 + decrement-\
                 on-miss law (decrement all counters by 1 on miss after k \
                 slots full) + merge law (combine counter lists then prune \
                 to k slots) + update order + error-bound semantics (every \
                 element with frequency > N/k guaranteed to appear in final \
                 k slots; explicitly one-sided guarantee) + residual \
                 definition + decision functional + confuser profile + \
                 numeric mode. Streaming-sketch heavy-hitter witness; the \
                 court does NOT admit exact-frequency certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SPACE_SAVING_RESERVED_CANONICAL_ID),
            reason: "Space-Saving heavy-hitter shift witness (Metwally-\
                 Agrawal-El Abbadi 2005): declared k counter slots (no hash \
                 family; counts are deterministic; no seed required) + \
                 depth 1 + replace-smallest-on-miss law (replace the \
                 smallest-counter element with new element on miss; \
                 deterministic) + merge law (combine counter lists then \
                 prune) + update order + error-bound semantics (per-element \
                 over-estimate bounded by the smallest counter at miss \
                 time; one-sided guarantee) + residual definition + \
                 decision functional + confuser profile + numeric mode. \
                 Structurally distinct from Misra-Gries 6504 because Space-\
                 Saving uses replace-smallest-on-miss whereas Misra-Gries \
                 uses decrement-all-on-miss; the bookkeeping rule is \
                 different. Streaming-sketch heavy-hitter witness.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(GREENWALD_KHANNA_RESERVED_CANONICAL_ID),
            reason: "Greenwald-Khanna quantile summary drift witness \
                 (Greenwald-Khanna 2001): declared epsilon error bound + \
                 tuple-count bound (O(log(epsilon * N) / epsilon)) + tuple-\
                 insertion rule + tuple-merging law (merge adjacent tuples \
                 when g_i + g_{i+1} + delta_{i+1} <= 2 * epsilon * N; \
                 deterministic) + merge law (combine and re-prune; \
                 mergeable with degraded epsilon) + update order + error-\
                 bound semantics (deterministic epsilon-approximate \
                 quantile guarantee; for any quantile phi the returned \
                 rank is within epsilon * N of the true rank; one-sided) + \
                 residual definition + decision functional + confuser \
                 profile + numeric mode. Streaming-sketch quantile witness; \
                 the court does NOT admit exact-quantile certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(T_DIGEST_RESERVED_CANONICAL_ID),
            reason: "t-digest summary residual witness (Dunning 2019): \
                 declared compression delta + buffer-size bound on centroid \
                 count (O(delta * log(N))) + centroid scale function \
                 (declared k_1 / k_2 / k_3) + DETERMINISTIC centroid-merge \
                 law (commutative merge of centroids ordered by mean; \
                 centroids below the scale-function threshold are merged; \
                 explicitly NOT randomized) + merge law (deterministic \
                 union of centroid lists followed by re-compression) + \
                 update order + error-bound semantics (declared compression \
                 delta governs centroid count and per-quantile error \
                 envelope; explicitly approximate, NOT deterministic \
                 certainty) + residual definition + decision functional + \
                 confuser profile + numeric mode. Streaming-sketch quantile \
                 witness with deterministic centroid-merge law.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(AMS_RESERVED_CANONICAL_ID),
            reason: "Alon-Matias-Szegedy (AMS) moment sketch witness (Alon-\
                 Matias-Szegedy 1999): declared 4-wise-independent hash \
                 family with output in {-1, +1} + per-sketch seed + sketch \
                 width (independent estimators) + depth (averaging / median-\
                 of-medians) + moment order p + signed-update rule + merge \
                 law (cell-wise addition; mergeable when sketches share \
                 hash family + width + seed) + update order + error-bound \
                 semantics (probabilistic epsilon-delta bound on p-th \
                 frequency moment) + residual definition + decision \
                 functional + confuser profile + numeric mode. Streaming-\
                 sketch moment witness; the court does NOT admit \
                 deterministic moment certainty.",
        },
        // -- 4 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(KS_SEED_ID),
            reason: "Kolmogorov-Smirnov two-sample test stays canonical at \
                 SEED id 8 under StreamingSketches. Streaming-approximate KS \
                 variants (6510) reduce to a sketch-parameterization of this \
                 primitive; the underlying decision functional remains the \
                 KS supremum-of-CDF-distance test. Declared sample contract \
                 + per-window CDF computation + supremum-distance decision \
                 law + numeric mode. No duplicate admitted; streaming-\
                 approximate KS (6510) collapses here as ParameterizationOf. \
                 The court does NOT admit approximate-query truth.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            reason: "Missingness spike stays canonical at SEED id 13 under \
                 StreamingSketches. Bloom-filter-based missingness inversion \
                 (6512) reduces to a sketch-parameterization of this \
                 primitive; the underlying decision functional remains the \
                 per-window missingness-rate-shift test. Declared per-\
                 window missingness rate + baseline + decision law + numeric \
                 mode. No duplicate admitted; sketch-approximate missingness \
                 via Bloom inversion (6512) collapses here as \
                 ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            reason: "Error burst stays canonical at SEED id 41 under \
                 StreamingSketches. Sliding-window heavy-hitter sketches \
                 (CMS, Misra-Gries, Space-Saving) over the error-event \
                 stream reduce to a sketch-parameterization of this \
                 primitive; the underlying decision functional remains the \
                 per-window error-burst threshold test. Declared error-\
                 event family + per-window count law + threshold law + \
                 numeric mode. No duplicate admitted; sliding-window error-\
                 burst sketch (6511) collapses here as ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            reason: "Cardinality drift stays canonical at SEED id 46 under \
                 StreamingSketches. Pre-HLL cardinality estimators \
                 (Flajolet-Martin 1985 probabilistic counting; Durand-\
                 Flajolet 2003 LogLog) reduce to a sketch-parameterization \
                 of this primitive. Declared per-window cardinality \
                 measurement + baseline + decision law + numeric mode. No \
                 duplicate admitted; Flajolet-Martin / probabilistic \
                 counting / LogLog (6509) collapses here as \
                 ParameterizationOf. HyperLogLog (6502) is structurally \
                 distinct and admitted as new canonical because its \
                 harmonic-mean estimator with bias correction is a \
                 different decision functional from the pre-HLL geometric-\
                 mean / arithmetic-mean estimators.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            reason: "Cardinality drift (SEED id 46) is the shared cardinality \
                 ancestor for the StreamingSketches source class. HyperLogLog \
                 (6502) and pre-HLL Flajolet-Martin probabilistic counting \
                 (6509) are descendants. The court records the domain \
                 transfer without re-canonicalising Cardinality drift.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(KS_SEED_ID),
            reason: "Kolmogorov-Smirnov two-sample test (SEED id 8) is the \
                 shared distribution-distance ancestor for the \
                 StreamingSketches source class. Streaming-approximate KS \
                 via quantile sketch (6510) is the streaming descendant. \
                 The court records the domain transfer without re-\
                 canonicalising the KS two-sample test.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID),
            reason: "Flajolet-Martin / probabilistic counting / LogLog \
                 cardinality estimator is ParameterizationOf(Cardinality \
                 drift, SEED id 46). Probabilistic-counting parameterization \
                 with declared hash family + leading-zero or trailing-zero \
                 register + cardinality-estimator law (Flajolet-Martin 1985; \
                 Durand-Flajolet 2003 LogLog). The court declines to admit \
                 pre-HLL cardinality estimators as a new canonical \
                 primitive; HLL (6502) is the structurally-distinct \
                 successor.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(STREAMING_KS_RESERVED_PRIMITIVE_ID),
            reason: "Streaming approximate KS via quantile sketch is \
                 ParameterizationOf(Kolmogorov-Smirnov two-sample test, \
                 SEED id 8). Quantile-sketch parameterization with declared \
                 quantile-sketch summary (Greenwald-Khanna 6506 or t-digest \
                 6507) + per-window KS distance estimate against the \
                 sketch's epsilon-approximate quantile function + sketch-\
                 error-budget contract. The court declines to admit \
                 streaming approximate KS as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID),
            reason: "Sliding-window error-burst sketch is \
                 ParameterizationOf(Error burst, SEED id 41). Sketch-\
                 windowing parameterization with declared sliding-window \
                 Count-Min sketch (6501) or heavy-hitter sketch (Misra-\
                 Gries 6504 / Space-Saving 6505) + declared window length \
                 + error-event family + per-window burst-threshold law. \
                 The court declines to admit sliding-window error-burst \
                 sketch as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID),
            reason: "Sketch-approximate missingness via Bloom inversion is \
                 ParameterizationOf(Missingness spike, SEED id 13). Bloom-\
                 filter-inversion parameterization with declared Bloom-\
                 filter membership contract (6503) + per-window expected-\
                 not-seen count + sketch-estimate-of-missingness-rate \
                 decision. The court declines to admit sketch-approximate \
                 missingness as a new canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID),
            reason: "Learned streaming-anomaly score / online-learning \
                 detector (Datadog Watchdog AI, DataRobot Streaming AutoML, \
                 Splunk Stream ML, AWS Lookout for Metrics, Azure Anomaly \
                 Detector streaming endpoint) exposes anomaly scores from \
                 continuously-updated learned embeddings without a \
                 deterministic feature-extraction law, declared formula, \
                 declared update rule with fixed step size and clipping \
                 law, declared training-data anchor, declared tie-break \
                 law, or declared numeric mode. Rejected unless reduced to \
                 a Deterministic_Streaming_Anomaly_Proxy with deterministic \
                 feature-extraction law + declared formula + declared \
                 update rule with fixed step size and clipping law + \
                 declared training-data anchor + feature schema + tie-\
                 break + numeric mode + no learned opaque embedding, all \
                 brutally explicit in a later T.12.x. The court does NOT \
                 issue anomaly verdicts from learned streaming pipelines; \
                 the rejection-shell describes what is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID),
            reason: "Black-box approximate-streaming proprietary sketch \
                 without declared hash / width / depth / seed / merge \
                 contract (Snowflake APPROX_COUNT_DISTINCT / \
                 APPROX_PERCENTILE / APPROX_TOP_K; BigQuery \
                 APPROX_COUNT_DISTINCT / APPROX_QUANTILES / \
                 APPROX_TOP_COUNT; Druid approximate aggregators; \
                 ClickHouse uniqHLL12 / quantileTDigest / topK approximate \
                 aggregators; AWS Athena APPROX_*) exposes results without \
                 declaring the underlying sketch's hash family, width / \
                 bucket count, depth, seed, or merge law. Rejected unless \
                 reduced to a Deterministic_Vendor_Sketch_Proxy with \
                 declared hash family + width + depth + seed + merge law + \
                 error-bound semantics (either via vendor publication or \
                 user deployment-time configuration pinned in a fixed \
                 receipt). The court does NOT admit approximate-query \
                 truth or database correctness verdicts from black-box \
                 vendor sketches; those terms appear here only to describe \
                 what is NOT admitted.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn streaming_sketches_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CMS_RESIDUAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HLL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BLOOM_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MISRA_GRIES_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SPACE_SAVING_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(MISRA_GRIES_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(GREENWALD_KHANNA_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(KS_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(T_DIGEST_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(GREENWALD_KHANNA_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(AMS_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CMS_RESIDUAL_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(CARDINALITY_DRIFT_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(STREAMING_KS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(KS_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID,
            ),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID,
            ),
            to_canonical_id: DetectorCanonicalId(MISSINGNESS_SPIKE_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Ten source refs supporting the streaming-sketches expansion.
fn streaming_sketches_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "cormode_muthukrishnan_2005",
            title: "An Improved Data Stream Summary: the Count-Min Sketch and \
                its Applications",
            year: 2005,
            venue: "Journal of Algorithms 55(1) (Count-Min sketch canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "flajolet_hll_2007",
            title: "HyperLogLog: the Analysis of a Near-Optimal Cardinality \
                Estimation Algorithm",
            year: 2007,
            venue: "Discrete Mathematics and Theoretical Computer Science \
                Proceedings (HyperLogLog canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "bloom_filter_1970",
            title: "Space/Time Trade-offs in Hash Coding with Allowable Errors",
            year: 1970,
            venue: "Communications of the ACM 13(7) (Bloom filter canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "misra_gries_1982",
            title: "Finding Repeated Elements",
            year: 1982,
            venue: "Science of Computer Programming 2(2) (Misra-Gries heavy-\
                hitter canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "metwally_space_saving_2005",
            title: "Efficient Computation of Frequent and Top-k Elements in Data \
                Streams",
            year: 2005,
            venue: "International Conference on Database Theory (Space-Saving \
                canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "greenwald_khanna_2001",
            title: "Space-Efficient Online Computation of Quantile Summaries",
            year: 2001,
            venue: "ACM SIGMOD International Conference on Management of Data \
                (Greenwald-Khanna quantile-summary canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "dunning_t_digest_2019",
            title: "Computing Extremely Accurate Quantiles Using t-Digests",
            year: 2019,
            venue: "arXiv:1902.04023 (t-digest canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "alon_matias_szegedy_1999",
            title: "The Space Complexity of Approximating the Frequency Moments",
            year: 1999,
            venue: "Journal of Computer and System Sciences 58(1) (AMS frequency-\
                moments sketch canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "flajolet_martin_1985",
            title: "Probabilistic Counting Algorithms for Data Base Applications",
            year: 1985,
            venue: "Journal of Computer and System Sciences 31(2) (pre-HLL \
                cardinality-estimation canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "vendor_streaming_refs",
            title: "Vendor streaming pipelines and approximate aggregators \
                (Datadog Watchdog AI, DataRobot Streaming AutoML, Splunk Stream \
                ML, AWS Lookout for Metrics, Azure Anomaly Detector streaming \
                endpoint; Snowflake APPROX_COUNT_DISTINCT / APPROX_PERCENTILE / \
                APPROX_TOP_K; BigQuery APPROX_COUNT_DISTINCT / APPROX_QUANTILES \
                / APPROX_TOP_COUNT; Druid approximate aggregators; ClickHouse \
                uniqHLL12 / quantileTDigest / topK; AWS Athena APPROX_*)",
            year: 2023,
            venue: "Vendor documentation (rejection-shell reference; vendor \
                scores lack public deterministic feature-extraction law or \
                public sketch hash / width / depth / seed / merge contract)",
        },
    ]
}

/// Build the T.12.o streaming-sketches `DedupCourtDelta`. The
/// delta names EIGHT new canonicals at 6501..=6508.
fn build_streaming_sketches_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_o_streaming_sketches_delta",
        vec![
            DetectorCanonicalId(CMS_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(HLL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BLOOM_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MISRA_GRIES_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SPACE_SAVING_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(GREENWALD_KHANNA_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(T_DIGEST_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(AMS_RESERVED_CANONICAL_ID),
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

/// Build the T.12.o Streaming Sketches
/// `CorpusAmendmentProposal`. Two builds against this static
/// seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_o_streaming_sketches_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_o_streaming_sketches_first_proposal",
        "T.12.o files the Streaming Sketches amendment proposal. Adds EIGHT \
         genuinely new canonical streaming-sketch primitives (Count-Min sketch \
         residual per Cormode-Muthukrishnan 2005, HyperLogLog cardinality \
         shift per Flajolet-Fusy-Gandouet-Meunier 2007, Bloom-filter \
         membership anomaly per Bloom 1970, Misra-Gries heavy-hitter shift \
         per Misra-Gries 1982, Space-Saving heavy-hitter shift per Metwally-\
         Agrawal-El Abbadi 2005 structurally distinct from Misra-Gries via \
         replace-smallest-on-miss rather than decrement-all-on-miss, Greenwald-\
         Khanna quantile summary drift per Greenwald-Khanna 2001 with \
         declared epsilon error bound, t-digest summary residual per Dunning \
         2019 with declared deterministic centroid-merge law, Alon-Matias-\
         Szegedy AMS moment sketch per Alon-Matias-Szegedy 1999 with declared \
         4-wise-independent hash family) at reserved canonical ids \
         6501..=6508. Each declares hash family (where applicable; absent for \
         Misra-Gries / Space-Saving / Greenwald-Khanna / t-digest which are \
         deterministic-counts / deterministic-tuple sketches) + width / \
         bucket count + depth + per-row or per-sketch seed array (where \
         applicable) + collision-resolution or update rule + merge law (where \
         mergeable) + update order + error-bound semantics (explicitly \
         probabilistic for hash-based sketches; deterministic-one-sided for \
         Misra-Gries / Space-Saving / Greenwald-Khanna) + residual definition \
         + decision functional + confuser profile + numeric mode contracts. \
         Records FOUR ExistingCanonicalAuthorityResolution decisions keeping \
         Kolmogorov-Smirnov two-sample test (SEED 8), Missingness spike (13), \
         Error burst (41), Cardinality drift (46) canonical under \
         StreamingSketches. Records TWO DomainTransferOf decisions: SEED 46 \
         Cardinality drift as shared cardinality ancestor; SEED 8 KS as \
         shared distribution-distance ancestor. Records FOUR ParameterizationOf \
         decisions (panel-candidate primitives that collapsed on closer \
         inspection): Flajolet-Martin / probabilistic-counting / LogLog \
         cardinality estimator (6509; Flajolet-Martin 1985 / Durand-Flajolet \
         2003) is ParameterizationOf(Cardinality drift, SEED 46) with HLL \
         (6502) as the structurally-distinct successor admitted separately as \
         new canonical; streaming approximate KS via quantile sketch (6510) is \
         ParameterizationOf(Kolmogorov-Smirnov two-sample test, SEED 8) with \
         declared quantile-sketch (Greenwald-Khanna 6506 or t-digest 6507); \
         sliding-window error-burst sketch (6511) is ParameterizationOf(Error \
         burst, SEED 41) with declared sliding-window CMS or heavy-hitter \
         sketch; sketch-approximate missingness via Bloom inversion (6512) is \
         ParameterizationOf(Missingness spike, SEED 13) with declared Bloom-\
         filter contract. Rejects TWO streaming records as \
         RejectedNotDeterministic (ninth T.12.x with two rejections, following \
         T.12.g / h / i / j / k / l / m / n): learned streaming-anomaly score \
         / online-learning detector (6513; Datadog Watchdog AI, DataRobot \
         Streaming AutoML, Splunk Stream ML, AWS Lookout for Metrics, Azure \
         Anomaly Detector streaming endpoint) and black-box approximate-\
         streaming proprietary sketch without declared hash / width / depth / \
         seed / merge contract (6514; Snowflake APPROX_*, BigQuery APPROX_*, \
         Druid approximate aggregators, ClickHouse uniqHLL12 / \
         quantileTDigest / topK, AWS Athena APPROX_*). Panel-locked non-claim: \
         T.12.o admits deterministic streaming-sketch witnesses; bounded-\
         memory, mergeable or update-order-declared summaries for frequency, \
         cardinality, quantile, heavy-hitter, membership, and moment / \
         variance evidence. It does not admit probabilistic accuracy as \
         certainty, randomized sketch behavior without seed / width / depth / \
         hash-family declaration, privacy claims, database correctness \
         authority, or approximate-query truth. Every CanonicalAddition / \
         ExistingCanonicalAuthorityResolution reason text declares the full \
         contract AND avoids the panel-locked forbidden terms (pinned by \
         t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract, \
         t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty, \
         t12_o_rejects_approximate_query_truth_claim_language, \
         t12_o_rejects_privacy_or_anonymization_claim_language, \
         t12_o_rejects_mergeable_sketch_without_merge_law, \
         t12_o_rejects_black_box_streaming_anomaly_score_without_formula \
         scanners). Does NOT mutate SEED (SEED.len() stays at 54); status = \
         Open pending review.",
        SourceClass::StreamingSketches,
        build_streaming_sketches_expansion_batch(),
        build_streaming_sketches_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_o_streaming_sketches",
    )
}
