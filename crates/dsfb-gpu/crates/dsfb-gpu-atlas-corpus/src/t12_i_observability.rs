//! T.12.i — Observability / Debugging: the ninth real
//! literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.i files the Observability / Debugging amendment
//! > proposal. It admits only deterministic software-
//! > observability witnesses whose trace / span / log / metric
//! > field, aggregation window, topology scope, baseline,
//! > decision law, and confuser semantics are declared;
//! > resolves collisions with the existing DSFB-GPU-Debug bank
//! > surface; classifies deployment / runtime variants as
//! > parameterizations or domain transfers; rejects learned
//! > APM anomaly scores or underspecified vendor heuristics;
//! > and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"An observability
//! symptom is not a detector until the telemetry field,
//! aggregation law, baseline, topology scope, and confuser
//! semantics are declared."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.i's design began with a grep of [`crate::seed::SEED`]
//! for every observability / debugging candidate. The walk
//! found **five** T.12.i-relevant primitives already canonical
//! — exactly the dsfb-gpu-debug-core bank surface that
//! motivated DSFB-GPU-Debug in the first place:
//!
//! * **Latency ramp** at SEED id 14 — sustained latency rise
//!   over a window pair.
//! * **Single-window spike confuser** at SEED id 15 — the
//!   panel-locked debug-bank negative witness suppressing
//!   one-window outliers.
//! * **Error burst** at SEED id 41 — per-window error-event
//!   count exceeding baseline.
//! * **Slew shock** at SEED id 42 — sudden derivative change
//!   in a tracked metric.
//! * **Fanout cascade** at SEED id 43 — propagating call-graph
//!   amplification.
//!
//! All five become `ExistingCanonicalAuthorityResolution`
//! records under the `ObservabilityDebugging` source class,
//! each with the specific telemetry-field + aggregation-law +
//! window-law + baseline-law + topology-scope + decision-law +
//! confuser-profile contract declared. **Re-adding any of
//! these as a new canonical would inflate the corpus and
//! double-count the existing bank surface; the court refuses.**
//!
//! Eight genuinely new canonicals at reserved ids 5901..=5908
//! are admitted with declared telemetry-and-decision-law
//! contracts:
//!
//! * **Retry storm** (5901) — declared retry-event field (HTTP
//!   retry / RPC retry / library-level retry) + counting law +
//!   window + per-service / per-route scope + threshold +
//!   confuser profile (legitimate idempotent retry vs storm).
//! * **Queue-depth pressure** (5902) — declared queue-depth
//!   metric source (broker / channel / thread-pool queue) +
//!   capacity contract + aggregation law (max / quantile / mean
//!   over window) + threshold + confuser profile (warm-up,
//!   batch-arrival).
//! * **Saturation precursor** (5903) — declared resource
//!   capacity contract (CPU / memory / file-descriptors / net
//!   bandwidth) + utilisation aggregation + slope or threshold
//!   law + confuser profile (Brendan-Gregg USE-style: must
//!   distinguish utilisation from saturation from error).
//! * **Cold-start transient** (5904) — declared deployment /
//!   warm-up marker (process-start, container-start, function-
//!   cold-start) + warmup window + suppression law + decision
//!   law (transient vs sustained).
//! * **Timeout burst** (5905) — declared timeout-event field
//!   (timeout error code / span status TimedOut), counting law,
//!   window, scope, threshold. Structurally distinct from SEED
//!   41 Error burst because timeout is a SPECIFIC failure class,
//!   not the general error-event class.
//! * **GC pause spike** (5906) — declared language runtime
//!   (JVM / .NET CLR / Go runtime / V8) + GC pause metric +
//!   window + quantile or max law + threshold + confuser
//!   profile (full-GC vs minor-GC).
//! * **Thread-pool exhaustion** (5907) — declared pool source
//!   (executor / connection-pool / async-runtime) + pool-
//!   capacity contract + active-count metric + saturation law +
//!   threshold. Genealogy: `DerivedFrom(Saturation precursor)`.
//! * **Backpressure propagation** (5908) — declared
//!   producer-consumer scope, queue / buffer occupancy field,
//!   flow-control signal, propagation-law (upstream throttle /
//!   drop / cascade), and decision law, with genealogy
//!   `DerivedFrom(Fanout cascade)`.
//!
//! Two domain transfers (panel-suggested):
//!
//! * **Fanout cascade** (SEED 43) → `DomainTransferOf` for the
//!   `ObservabilityDebugging` source class. The same cascade
//!   primitive recognised under graph topology (T.12.g) is
//!   recognised again under service-call observability without
//!   re-canonicalising.
//! * **Error burst** (SEED 41) → `DomainTransferOf` for the
//!   `ObservabilityDebugging` source class as the shared
//!   rate-burst ancestor for service telemetry (HTTP 5xx
//!   bursts, RPC error bursts, log error-rate bursts all
//!   collapse to this ancestor).
//!
//! Four parameterizations:
//!
//! * **HTTP 5xx burst** (5909) → `ParameterizationOf(Error
//!   burst, SEED 41)` — error-event field parameterized to
//!   HTTP status codes 5xx.
//! * **p95 / p99 latency ramp** (5910) → `ParameterizationOf
//!   (Latency ramp, SEED 14)` — aggregation law parameterized
//!   to upper quantiles (p95 / p99 / p99.9).
//! * **k-hop dependency fanout** (5911) →
//!   `ParameterizationOf(Fanout cascade, SEED 43)` — topology
//!   scope parameterized to a declared hop limit on the
//!   dependency graph.
//! * **Retry-rate burst** (5912) → `ParameterizationOf(Retry
//!   storm, 5901)` — aggregation law parameterized to a rate
//!   normalisation (retries / request) rather than absolute
//!   counts.
//!
//! Two rejections (third T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g and
//! T.12.h):
//!
//! * **Vendor APM black-box anomaly score** (5913) —
//!   `RejectedNotDeterministic`. Vendor APM products (Datadog
//!   anomaly detection, New Relic AI-applied intelligence,
//!   Dynatrace Davis, Splunk MLTK, AWS DevOps Guru) expose
//!   "anomaly scores" without a stable public decision
//!   functional. Admission requires a future T.12.x proposal
//!   to admit a `Deterministic_APM_Score_Proxy` canonical with
//!   the model-identification anchor, training-data anchor,
//!   feature schema, tie-break, and numeric mode all brutally
//!   explicit.
//! * **Learned incident classifier** (5914) —
//!   `RejectedNotDeterministic`. Learned classifiers (PagerDuty
//!   intelligent triage, Splunk On-Call ML classifiers,
//!   ServiceNow AIOps) classify incidents by learned
//!   embeddings + supervised training on historic incidents.
//!   Admission requires model-identification seed + training-
//!   data anchor + label schema + tie-break + numeric mode
//!   declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×8.
//! * `ExistingCanonicalAuthorityResolution` ×5.
//! * `DomainTransferOf` ×2 — Fanout cascade and Error burst as
//!   shared ancestors for `ObservabilityDebugging`.
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 8 + 5 + 2 + 4 + 2 = **21 dedup-court records**.
//!
//! ## Bank-surface non-duplication (panel-locked)
//!
//! The dsfb-gpu-debug bank surface IDs (14, 15, 41, 42, 43)
//! are L6 GPU-implemented per [`crate::lband::
//! GPU_IMPLEMENTED_CANONICAL_IDS`]. T.12.i's main job is to
//! ratify them under the observability source class WITHOUT
//! re-canonicalising any. Inflating the corpus with renamed
//! duplicates of these would erase the L6 honesty marker.
//!
//! ## APM-rejection non-claim (panel-locked)
//!
//! Vendor APM anomaly scores (5913) and learned incident
//! classifiers (5914) are explicitly NOT admitted to
//! `new_canonical_records`. The court is not in the business
//! of laundering proprietary black-box scores as deterministic
//! witnesses. The reason text carries the panel-locked phrasing
//! "deterministic formula" / "model-identification anchor"
//! requirement so a future activation planner cannot promote
//! a vendor score without satisfying the contract.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11/S1.3/T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.i observability
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
// Reserved id constants (panel-locked, 5901..=5914 bucket)
// ---------------------------------------------------------------

/// Reserved canonical id for Retry storm.
/// 5901..=5914 is the T.12.i bucket.
pub const RETRY_STORM_RESERVED_CANONICAL_ID: u32 = 5901;

/// Reserved canonical id for Queue-depth pressure.
pub const QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID: u32 = 5902;

/// Reserved canonical id for Saturation precursor (USE-method
/// resource-saturation precursor).
pub const SATURATION_PRECURSOR_RESERVED_CANONICAL_ID: u32 = 5903;

/// Reserved canonical id for Cold-start transient.
pub const COLD_START_TRANSIENT_RESERVED_CANONICAL_ID: u32 = 5904;

/// Reserved canonical id for Timeout burst (distinct from
/// Error burst's general error-event class).
pub const TIMEOUT_BURST_RESERVED_CANONICAL_ID: u32 = 5905;

/// Reserved canonical id for GC pause spike.
pub const GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID: u32 = 5906;

/// Reserved canonical id for Thread-pool exhaustion.
/// Genealogy: `DerivedFrom(Saturation precursor)`.
pub const THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID: u32 = 5907;

/// Reserved canonical id for Backpressure propagation.
/// Genealogy: `DerivedFrom(Fanout cascade)`.
pub const BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID: u32 = 5908;

/// Reserved id for HTTP 5xx burst.
/// `ParameterizationOf(Error burst, SEED 41)`.
pub const HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID: u32 = 5909;

/// Reserved id for p95 / p99 latency ramp.
/// `ParameterizationOf(Latency ramp, SEED 14)`.
pub const QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID: u32 = 5910;

/// Reserved id for k-hop dependency fanout.
/// `ParameterizationOf(Fanout cascade, SEED 43)`.
pub const K_HOP_FANOUT_RESERVED_PRIMITIVE_ID: u32 = 5911;

/// Reserved id for Retry-rate burst.
/// `ParameterizationOf(Retry storm, 5901)`.
pub const RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID: u32 = 5912;

/// Reserved id for Vendor APM black-box anomaly score.
/// `RejectedNotDeterministic`.
pub const VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID: u32 = 5913;

/// Reserved id for Learned incident classifier.
/// `RejectedNotDeterministic`.
pub const LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 5914;

// Existing SEED canonical ids referenced by T.12.i.

/// Latency ramp — SEED canonical id 14. L6 dsfb-gpu-debug bank
/// surface.
pub const LATENCY_RAMP_SEED_ID: u32 = 14;

/// Single-window spike confuser — SEED canonical id 15. L6
/// dsfb-gpu-debug bank surface; the panel-locked debug-bank
/// negative witness suppressing one-window outliers.
pub const SINGLE_WINDOW_SPIKE_CONFUSER_SEED_ID: u32 = 15;

/// Error burst — SEED canonical id 41. L6 dsfb-gpu-debug bank
/// surface.
pub const ERROR_BURST_SEED_ID: u32 = 41;

/// Slew shock — SEED canonical id 42. L6 dsfb-gpu-debug bank
/// surface.
pub const SLEW_SHOCK_SEED_ID: u32 = 42;

/// Fanout cascade — SEED canonical id 43. L6 dsfb-gpu-debug
/// bank surface; the shared cascade ancestor recognised by
/// both T.12.g (graph topology) and T.12.i (service-call
/// observability).
pub const FANOUT_CASCADE_SEED_ID: u32 = 43;

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
// Builders for the observability expansion batch
// ---------------------------------------------------------------

/// Build the observability `CorpusExpansionBatch` body.
fn build_observability_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_i_observability_first_proposal",
        SourceClass::ObservabilityDebugging,
        observability_proposed_primitives(),
        observability_proposed_aliases(),
        observability_proposed_dedup_records(),
        observability_proposed_genealogy_edges(),
        observability_proposed_source_refs(),
    )
}

/// Fourteen proposed primitives: 8 canonical + 4 parameterization
/// shells + 2 rejection shells.
fn observability_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RETRY_STORM_RESERVED_CANONICAL_ID),
            display_name: "Retry storm",
            motivation: "Retry-storm detector for distributed systems. Required \
                 telemetry contract: retry-event field (HTTP retry count / RPC \
                 retry count / library-level retry counter), counting law (count \
                 per window vs rate per request), per-service or per-route scope, \
                 baseline window, threshold, confuser profile (legitimate \
                 idempotent retry vs storm; deployment-triggered retry surge). \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID),
            display_name: "Queue-depth pressure",
            motivation: "Queue-depth pressure detector. Required telemetry contract: \
                 queue-depth metric source (message broker / async channel / \
                 thread-pool work queue / database connection-pool wait queue), \
                 capacity contract (max-depth declaration), aggregation law (max / \
                 quantile / EWMA over window), threshold, confuser profile (warm- \
                 up batch arrival; deploy-induced backlog). Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SATURATION_PRECURSOR_RESERVED_CANONICAL_ID),
            display_name: "Saturation precursor",
            motivation: "Resource-saturation precursor detector (Brendan Gregg USE \
                 method). Required telemetry contract: resource capacity contract \
                 (CPU cores / memory bytes / file-descriptors / network bandwidth \
                 / disk IOPS), utilisation aggregation, slope or threshold law \
                 (rising toward capacity), confuser profile (must distinguish \
                 utilisation from saturation from error per USE method). \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COLD_START_TRANSIENT_RESERVED_CANONICAL_ID),
            display_name: "Cold-start transient",
            motivation: "Cold-start transient detector. Required telemetry contract: \
                 deployment / warm-up marker (process-start timestamp / container- \
                 start event / serverless function cold-start indicator), warmup \
                 window declaration, suppression law (transient anomaly during \
                 warm-up is NOT an episode), decision law (transient vs sustained \
                 cutoff), confuser profile (deploy event vs steady-state). \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TIMEOUT_BURST_RESERVED_CANONICAL_ID),
            display_name: "Timeout burst",
            motivation: "Timeout-burst detector. Required telemetry contract: \
                 timeout-event field (timeout error code, span status TimedOut, \
                 connection-timeout exception class), counting law, window, scope \
                 (per-service / per-route / per-dependency), threshold. \
                 Structurally distinct from Error burst (SEED 41) - timeout is a \
                 SPECIFIC failure class, not the general error-event class. \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID),
            display_name: "GC pause spike",
            motivation: "Garbage-collection pause spike detector. Required \
                 telemetry contract: language runtime (JVM / .NET CLR / Go runtime \
                 / V8 / Erlang BEAM), GC pause-duration metric, window, quantile \
                 or max aggregation law, threshold, confuser profile (full-GC vs \
                 minor-GC / nursery vs tenured collection). Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID,
            ),
            display_name: "Thread-pool exhaustion",
            motivation: "Thread-pool exhaustion detector. Required telemetry \
                 contract: pool source (executor / connection-pool / async-runtime \
                 worker pool), pool-capacity contract, active-count metric, \
                 saturation law (active = capacity for sustained window), \
                 threshold. Genealogy: DerivedFrom(Saturation precursor) - this is \
                 a thread-pool-specific saturation witness with declared pool \
                 capacity. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID,
            ),
            display_name: "Backpressure propagation",
            motivation: "Backpressure propagation detector. Required telemetry \
                 contract: producer-consumer scope (upstream / downstream service \
                 pair), queue / buffer occupancy field, flow-control signal \
                 (Reactive Streams request-n / TCP zero-window / explicit \
                 throttle), propagation law (upstream throttle vs drop vs \
                 cascade), decision law. Genealogy: DerivedFrom(Fanout cascade) - \
                 this is a flow-control-specific cascade witness. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID),
            display_name: "HTTP 5xx burst - parameterization shell",
            motivation: "Error-event-field parameterization of Error burst (SEED \
                 id 41) where the error-event field is restricted to HTTP status \
                 codes in the 5xx range. The court rules: HTTP 5xx burst is \
                 ParameterizationOf(Error burst, SEED 41), NOT a new canonical \
                 primitive. Appears in proposed_primitives but NOT in \
                 new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID),
            display_name: "p95 / p99 latency ramp - parameterization shell",
            motivation: "Aggregation-law parameterization of Latency ramp (SEED id \
                 14) where the aggregation law is restricted to upper quantiles \
                 (p95 / p99 / p99.9). The court rules: p95 / p99 latency ramp is \
                 ParameterizationOf(Latency ramp, SEED 14) with declared quantile \
                 parameter, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            display_name: "k-hop dependency fanout - parameterization shell",
            motivation: "Topology-scope parameterization of Fanout cascade (SEED \
                 id 43) where the cascade scope is restricted to a declared hop \
                 limit k on the dependency graph. The court rules: k-hop \
                 dependency fanout is ParameterizationOf(Fanout cascade, SEED 43) \
                 with declared hop-limit parameter, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID),
            display_name: "Retry-rate burst - parameterization shell",
            motivation: "Aggregation-law parameterization of Retry storm (5901) \
                 where counts are normalised to retry-rate (retries per request) \
                 instead of absolute retry counts. The court rules: retry-rate \
                 burst is ParameterizationOf(Retry storm, 5901) with declared \
                 rate-normalisation, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID),
            display_name: "Vendor APM black-box anomaly score - rejected shell",
            motivation: "Vendor APM anomaly scores (Datadog anomaly detection, \
                 New Relic AI-applied intelligence, Dynatrace Davis, Splunk MLTK, \
                 AWS DevOps Guru) expose a numeric 'anomaly score' without a \
                 stable public decision functional, declared training-data anchor, \
                 or model-identification anchor. The court does NOT admit vendor \
                 APM scores to the dedup-court delta's new_canonical_records. A \
                 future T.12.x proposal may admit a \
                 Deterministic_APM_Score_Proxy canonical only if a deterministic \
                 formula, model-identification anchor, training-data anchor, \
                 feature schema, tie-break, and numeric mode are all brutally \
                 explicit.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Learned incident classifier - rejected shell",
            motivation: "Learned incident classifiers (PagerDuty intelligent \
                 triage, Splunk On-Call ML classifiers, ServiceNow AIOps) \
                 classify production incidents by learned embeddings + supervised \
                 training on historic incident labels. The court does NOT admit \
                 learned incident classifiers to the dedup-court delta's \
                 new_canonical_records. A future T.12.x proposal may admit a \
                 Deterministic_Incident_Classifier_Proxy canonical only if the \
                 model-identification seed, training-data anchor (pinned-fixture- \
                 hash), label schema (pinned), tie-break law, and numeric mode \
                 are all brutally explicit.",
        },
    ]
}

/// Zero alias claims (T.12.i routes everything through dedup
/// records and existing-canonical authority resolutions).
fn observability_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty-one dedup-court decisions on the observability batch.
fn observability_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(RETRY_STORM_RESERVED_CANONICAL_ID),
            reason: "Retry storm: declared retry-event field (HTTP retry / RPC \
                 retry / library-level retry counter) + counting law + window + \
                 per-service / per-route scope + threshold + confuser profile \
                 (legitimate idempotent retry vs storm; deploy-triggered surge). \
                 Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID),
            reason: "Queue-depth pressure: declared queue-depth metric source \
                 (broker / async channel / thread-pool work queue / DB connection- \
                 pool wait queue) + capacity contract (max-depth declaration) + \
                 aggregation law (max / quantile / EWMA over window) + threshold \
                 + confuser profile (warm-up batch arrival; deploy backlog). \
                 Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SATURATION_PRECURSOR_RESERVED_CANONICAL_ID),
            reason: "Saturation precursor (USE method): declared resource capacity \
                 contract (CPU cores / memory bytes / file-descriptors / net \
                 bandwidth / disk IOPS) + utilisation aggregation + slope or \
                 threshold law (rising toward capacity) + confuser profile (must \
                 distinguish utilisation from saturation from error per USE \
                 method).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(COLD_START_TRANSIENT_RESERVED_CANONICAL_ID),
            reason: "Cold-start transient: declared deployment / warm-up marker \
                 (process-start timestamp / container-start event / serverless \
                 cold-start indicator) + warmup window + suppression law \
                 (transient anomaly during warm-up is NOT an episode) + decision \
                 law (transient vs sustained cutoff) + confuser profile (deploy \
                 event vs steady-state).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(TIMEOUT_BURST_RESERVED_CANONICAL_ID),
            reason: "Timeout burst: declared timeout-event field (timeout error \
                 code / span status TimedOut / connection-timeout exception class) \
                 + counting law + window + scope (per-service / per-route / per- \
                 dependency) + threshold. Structurally distinct from Error burst \
                 (SEED 41) - timeout is a SPECIFIC failure class.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID),
            reason: "GC pause spike: declared language runtime (JVM / .NET CLR / \
                 Go runtime / V8 / Erlang BEAM) + GC pause-duration metric + \
                 window + quantile or max aggregation law + threshold + confuser \
                 profile (full-GC vs minor-GC / nursery vs tenured collection).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID),
            reason: "Thread-pool exhaustion: declared pool source (executor / \
                 connection-pool / async-runtime worker pool) + pool-capacity \
                 contract + active-count metric + saturation law (active = \
                 capacity for sustained window) + threshold. Genealogy: \
                 DerivedFrom(Saturation precursor, 5903) - thread-pool-specific \
                 saturation witness with declared pool capacity.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID),
            reason: "Backpressure propagation: declared producer-consumer scope \
                 (upstream / downstream service pair) + queue or buffer occupancy \
                 field + flow-control signal (Reactive Streams request-n / TCP \
                 zero-window / explicit throttle) + propagation law (upstream \
                 throttle vs drop vs cascade) + decision law. Genealogy: \
                 DerivedFrom(Fanout cascade, SEED 43) - flow-control-specific \
                 cascade witness.",
        },
        // -- 5 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(LATENCY_RAMP_SEED_ID),
            reason: "Latency ramp stays canonical at SEED id 14 (L6 dsfb-gpu-debug \
                 bank surface). Declared telemetry field (duration_ms / span \
                 duration / latency-histogram bucket) + aggregation law (mean / \
                 quantile / EWMA over window) + window pair (baseline + active) \
                 + per-service / per-route scope + threshold + confuser profile \
                 (single-window spike). No duplicate admitted; p95 / p99 latency \
                 ramp (5910 below) collapses here as ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SINGLE_WINDOW_SPIKE_CONFUSER_SEED_ID),
            reason: "Single-window spike confuser stays canonical at SEED id 15 \
                 (L6 dsfb-gpu-debug bank surface; panel-locked debug-bank \
                 negative witness). Declared window-count law (count == 1 \
                 suppresses) + suppression scope + per-detector confuser binding. \
                 No duplicate admitted - the negative-witness machinery is the \
                 bank surface, not an observability primitive in its own right.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            reason: "Error burst stays canonical at SEED id 41 (L6 dsfb-gpu-debug \
                 bank surface). Declared error-event field (general error counter \
                 / span status Error) + counting law (count per window) + window \
                 + per-service / per-route scope + baseline + threshold + \
                 confuser profile (deploy-induced error surge; warmup). No \
                 duplicate admitted; HTTP 5xx burst (5909 below) collapses here \
                 as ParameterizationOf.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SLEW_SHOCK_SEED_ID),
            reason: "Slew shock stays canonical at SEED id 42 (L6 dsfb-gpu-debug \
                 bank surface). Declared tracked-metric field + derivative / \
                 slew-rate law + window + threshold + confuser profile (clock \
                 skew / missing data points). No duplicate admitted - queue- \
                 depth pressure (5902) and saturation precursor (5903) are \
                 structurally distinct (utilisation aggregation, not \
                 derivative-of-metric).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            reason: "Fanout cascade stays canonical at SEED id 43 (L6 dsfb-gpu- \
                 debug bank surface). Declared call-graph topology + per-edge \
                 propagation indicator + cascade-amplification law + window + \
                 threshold + confuser profile (legitimate batch fanout vs \
                 cascade). No duplicate admitted; k-hop dependency fanout (5911 \
                 below) collapses here as ParameterizationOf; backpressure \
                 propagation (5908 above) is DerivedFrom this canonical with \
                 flow-control-specific telemetry contracts.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            reason: "Fanout cascade (SEED id 43) is the shared cascade ancestor \
                 for the ObservabilityDebugging source class (same primitive \
                 recognised under T.12.g GraphAnomalyDetection). The court \
                 records the domain transfer without re-canonicalising Fanout \
                 cascade.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            reason: "Error burst (SEED id 41) is the shared rate-burst ancestor \
                 for the ObservabilityDebugging source class (HTTP 5xx bursts, \
                 RPC error bursts, log error-rate bursts all collapse to this \
                 ancestor). The court records the domain transfer without re- \
                 canonicalising Error burst.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID),
            reason: "HTTP 5xx burst is ParameterizationOf(Error burst, SEED id \
                 41). Error-event field parameterized to HTTP status codes 5xx. \
                 The court declines to admit HTTP 5xx burst as a new canonical \
                 primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID),
            reason: "p95 / p99 latency ramp is ParameterizationOf(Latency ramp, \
                 SEED id 14). Aggregation law parameterized to upper quantiles \
                 (p95 / p99 / p99.9). The court declines to admit p95 / p99 \
                 latency ramp as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            reason: "k-hop dependency fanout is ParameterizationOf(Fanout cascade, \
                 SEED id 43). Topology scope parameterized to a declared hop \
                 limit k on the dependency graph. The court declines to admit \
                 k-hop dependency fanout as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID),
            reason: "Retry-rate burst is ParameterizationOf(Retry storm, 5901). \
                 Aggregation law parameterized to a rate normalisation (retries \
                 per request) instead of absolute counts. The court declines to \
                 admit retry-rate burst as a new canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID),
            reason: "Vendor APM black-box anomaly score (Datadog anomaly \
                 detection, New Relic AI-applied intelligence, Dynatrace Davis, \
                 Splunk MLTK, AWS DevOps Guru) exposes a numeric anomaly score \
                 without a deterministic formula, model-identification anchor, \
                 training-data anchor, feature schema, tie-break, or numeric \
                 mode. Rejected unless reduced to a declared Deterministic_APM_ \
                 Score_Proxy canonical with all six contract fields brutally \
                 explicit in a later T.12.x.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID),
            reason: "Learned incident classifier (PagerDuty intelligent triage, \
                 Splunk On-Call ML classifiers, ServiceNow AIOps) classifies \
                 production incidents by learned embeddings + supervised \
                 training on historic incident labels. Rejected unless reduced \
                 to a declared Deterministic_Incident_Classifier_Proxy canonical \
                 with model-identification seed + training-data anchor pinned- \
                 fixture-hash + label schema pinned + tie-break law + numeric \
                 mode all brutally explicit in a later T.12.x.",
        },
    ]
}

/// Twelve genealogy edges proposed for the post-freeze graph.
fn observability_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RETRY_STORM_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SLEW_SHOCK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SATURATION_PRECURSOR_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SLEW_SHOCK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COLD_START_TRANSIENT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(LATENCY_RAMP_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(TIMEOUT_BURST_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(LATENCY_RAMP_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SATURATION_PRECURSOR_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(LATENCY_RAMP_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(RETRY_STORM_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the observability expansion.
fn observability_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "brooker_retries_aws_builders_library",
            title: "Timeouts, retries, and backoff with jitter",
            year: 2019,
            venue: "Amazon Builders' Library (retry-storm reference)",
        },
        ProposedSourceRef {
            citation_key: "kleinrock_queueing_systems_1975",
            title: "Queueing Systems Volume I: Theory",
            year: 1975,
            venue: "Wiley (queue-depth pressure foundational reference)",
        },
        ProposedSourceRef {
            citation_key: "gregg_use_method_2013",
            title: "The USE Method: addressing performance issues",
            year: 2013,
            venue: "USENIX LISA13 / brendangregg.com (saturation precursor)",
        },
        ProposedSourceRef {
            citation_key: "hellerstein_serverless_cold_start_2019",
            title: "Serverless Computing: One Step Forward, Two Steps Back",
            year: 2019,
            venue: "CIDR 2019 (cold-start transient reference)",
        },
        ProposedSourceRef {
            citation_key: "rfc7231_http_semantics",
            title: "Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content",
            year: 2014,
            venue: "IETF RFC 7231 (HTTP 5xx status code semantics)",
        },
        ProposedSourceRef {
            citation_key: "jvm_hotspot_gc_tuning",
            title: "HotSpot Virtual Machine Garbage Collection Tuning Guide",
            year: 2023,
            venue: "Oracle / OpenJDK documentation (GC pause spike reference)",
        },
        ProposedSourceRef {
            citation_key: "netflix_hystrix_2012",
            title: "Hystrix: Latency and Fault Tolerance for Distributed Systems",
            year: 2012,
            venue: "Netflix Tech Blog (thread-pool exhaustion / bulkhead pattern)",
        },
        ProposedSourceRef {
            citation_key: "reactive_streams_specification_1_0",
            title: "Reactive Streams Specification 1.0",
            year: 2015,
            venue: "reactive-streams.org (backpressure propagation reference)",
        },
        ProposedSourceRef {
            citation_key: "majors_observability_2022",
            title: "Observability Engineering",
            year: 2022,
            venue: "O'Reilly Media (rejection-shell reference for vendor APM \
                anomaly scores and learned incident classifiers; honest framing of \
                vendor black-box limits)",
        },
    ]
}

/// Build the T.12.i observability `DedupCourtDelta`.
fn build_observability_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_i_observability_delta",
        vec![
            DetectorCanonicalId(RETRY_STORM_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SATURATION_PRECURSOR_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(COLD_START_TRANSIENT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(TIMEOUT_BURST_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID),
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

/// Build the T.12.i observability `CorpusAmendmentProposal`.
/// Two builds against this static seed produce byte-identical
/// bytes.
#[must_use]
pub fn seed_t12_i_observability_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_i_observability_first_proposal",
        "T.12.i files the Observability / Debugging amendment proposal. Adds eight \
         genuinely new canonical observability primitives (retry storm, queue-depth \
         pressure, saturation precursor, cold-start transient, timeout burst, GC \
         pause spike, thread-pool exhaustion, backpressure propagation) at reserved \
         canonical ids 5901..=5908 with declared telemetry-field + aggregation-law \
         + window-law + baseline + topology-scope + threshold + confuser-profile \
         contracts. Records five ExistingCanonicalAuthorityResolution decisions \
         keeping the dsfb-gpu-debug bank surface canonical without duplication: \
         Latency ramp (SEED id 14), Single-window spike confuser (id 15), Error \
         burst (id 41), Slew shock (id 42), Fanout cascade (id 43). Records two \
         DomainTransferOf decisions: Fanout cascade as the shared cascade ancestor \
         for ObservabilityDebugging (re-recognised from T.12.g GraphAnomalyDetection \
         under the observability source class) and Error burst as the shared rate- \
         burst ancestor for service telemetry. Records four ParameterizationOf \
         decisions: HTTP 5xx burst is ParameterizationOf(Error burst); p95 / p99 \
         latency ramp is ParameterizationOf(Latency ramp); k-hop dependency fanout \
         is ParameterizationOf(Fanout cascade); retry-rate burst is \
         ParameterizationOf(Retry storm). Rejects TWO observability literature \
         records as RejectedNotDeterministic (third T.12.x proposal with two \
         rejection records in one commit, following T.12.g and T.12.h): vendor APM \
         black-box anomaly score (5913 - Datadog / New Relic / Dynatrace / Splunk \
         MLTK / AWS DevOps Guru) and learned incident classifier (5914 - PagerDuty / \
         Splunk On-Call / ServiceNow AIOps). Every record's reason text declares \
         its specific telemetry-field + aggregation + decision + confuser contract \
         - the panel-locked warning was 'an observability symptom is not a detector \
         until the telemetry field, aggregation law, baseline, topology scope, and \
         confuser semantics are declared'. The dsfb-gpu-debug bank surface IDs (14, \
         15, 41, 42, 43) are NOT re-canonicalised; inflating the corpus with \
         renamed duplicates would erase the L6 honesty marker. Does NOT mutate SEED \
         (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::ObservabilityDebugging,
        build_observability_expansion_batch(),
        build_observability_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_i_observability",
    )
}
