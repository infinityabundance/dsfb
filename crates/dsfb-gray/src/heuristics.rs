//! Heuristics bank: typed degradation motifs with Rust-specific provenance.
//!
//! The heuristics bank is the semantic layer of the DSFB engine. It maps
//! residual sign patterns (drift direction, slew shape, source channel
//! correlation) to named reason codes with human-readable provenance.
//!
//! ## Design Decision: Finite and Versioned
//!
//! The heuristics bank is explicitly finite. Novel patterns not represented
//! in the bank produce `UnclassifiedStructuralAnomaly` — the system admits
//! what it does not know rather than fabricating an explanation.
//!
//! ## Failure Mode FM-07: Heuristics Bank Incompleteness
//!
//! Novel failure modes not in the bank produce UnclassifiedStructuralAnomaly.
//! This is by design. The bank is versioned and extensible.

use crate::grammar::GrammarState;
use crate::residual::{ResidualSign, ResidualSource};
use crate::ReasonCode;

const MAX_STATIC_PRIORS: usize = 16;

/// Unique identifier for a heuristic entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeuristicId(pub &'static str);

/// A single entry in the heuristics bank.
///
/// Each entry encodes a specific structural pattern recognized by experienced
/// Rust/distributed-systems engineers, formalized into a deterministic matching
/// rule.
#[derive(Debug, Clone, Copy)]
pub struct HeuristicEntry {
    /// Unique identifier (e.g., "H-RAFT-01").
    pub id: HeuristicId,
    /// Which residual source(s) this heuristic applies to.
    pub primary_source: ResidualSource,
    /// Minimum drift magnitude to trigger (absolute value).
    pub drift_threshold: f64,
    /// Minimum slew magnitude to trigger (absolute value). Use 0.0 for drift-only.
    pub slew_threshold: f64,
    /// Whether drift must be positive (true) or magnitude-only (false).
    pub drift_positive_required: bool,
    /// Reason code emitted when this heuristic matches.
    pub reason_code: ReasonCode,
    /// Human-readable description of the pattern.
    pub description: &'static str,
    /// Rust-specific provenance: what real-world Rust pattern produces this.
    pub provenance: &'static str,
}

/// Result of matching a residual sign against the heuristics bank.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The reason code from the best-matching heuristic, or
    /// `UnclassifiedStructuralAnomaly` if no match.
    pub reason_code: ReasonCode,
    /// The heuristic ID that matched, if any.
    pub matched_heuristic: Option<HeuristicId>,
    /// Match confidence: how strongly the residual sign matches the pattern.
    /// 0.0 = no match, 1.0 = exact match. Based on how far past thresholds.
    pub confidence: f64,
    /// Human-readable description of the matched structural pattern.
    pub description: &'static str,
    /// Rust-specific provenance of the matched structural pattern.
    pub provenance: &'static str,
    /// Static prior applied to the winning heuristic, if any.
    pub applied_prior: Option<AppliedStaticPrior>,
}

/// Static prior for one heuristic, typically sourced from the crate scanner.
///
/// The scales are bounded threshold multipliers. Values lower than `1.0`
/// make the heuristic easier to trigger, while values above `1.0` make it
/// harder to trigger. The observer clamps them to a safe range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaticPrior {
    /// Heuristic to which the prior applies.
    pub heuristic_id: HeuristicId,
    /// Confidence assigned by the static scanner or caller, in `[0.0, 1.0]`.
    pub confidence: f64,
    /// Drift-threshold scale factor.
    pub drift_scale: f64,
    /// Slew-threshold scale factor.
    pub slew_scale: f64,
}

impl StaticPrior {
    /// Create a new static prior with bounded confidence and scales.
    pub fn new(
        heuristic_id: HeuristicId,
        confidence: f64,
        drift_scale: f64,
        slew_scale: f64,
    ) -> Self {
        Self {
            heuristic_id,
            confidence: confidence.clamp(0.0, 1.0),
            drift_scale: drift_scale.clamp(0.5, 2.0),
            slew_scale: slew_scale.clamp(0.5, 2.0),
        }
    }
}

/// Static prior actually applied during one heuristic match.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppliedStaticPrior {
    /// Heuristic to which the prior was applied.
    pub heuristic_id: HeuristicId,
    /// Prior confidence used for this match.
    pub confidence: f64,
    /// Effective drift-threshold scale used for this match.
    pub drift_scale: f64,
    /// Effective slew-threshold scale used for this match.
    pub slew_scale: f64,
}

/// Fixed-capacity collection of static heuristic priors.
///
/// This type is `no_std` and `no_alloc` friendly so it can be carried into the
/// core observer without introducing heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct StaticPriorSet {
    priors: [Option<StaticPrior>; MAX_STATIC_PRIORS],
    len: usize,
}

impl StaticPriorSet {
    /// Create an empty prior set.
    pub const fn new() -> Self {
        Self {
            priors: [None; MAX_STATIC_PRIORS],
            len: 0,
        }
    }

    /// Add or replace one prior. If the set is full, the last slot is reused.
    pub fn with_prior(mut self, prior: StaticPrior) -> Self {
        if let Some(existing) = self
            .priors
            .iter_mut()
            .flatten()
            .find(|existing| existing.heuristic_id == prior.heuristic_id)
        {
            *existing = prior;
            return self;
        }

        if self.len < MAX_STATIC_PRIORS {
            self.priors[self.len] = Some(prior);
            self.len += 1;
        } else if let Some(slot) = self.priors.last_mut() {
            *slot = Some(prior);
        }
        self
    }

    /// Return the prior for one heuristic, if present.
    pub fn get(&self, heuristic_id: HeuristicId) -> Option<StaticPrior> {
        self.priors
            .iter()
            .flatten()
            .find(|prior| prior.heuristic_id == heuristic_id)
            .copied()
    }

    /// Number of configured priors.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for StaticPriorSet {
    fn default() -> Self {
        Self::new()
    }
}

/// The heuristics bank: a fixed-size collection of typed degradation motifs.
///
/// Version 1.0: 12 entries covering the primary distributed Rust system
/// failure patterns. The bank is finite and versioned — novel patterns
/// produce `UnclassifiedStructuralAnomaly`.
pub struct HeuristicsBank {
    entries: &'static [HeuristicEntry],
}

/// Default heuristics bank with Rust-specific entries.
pub const DEFAULT_ENTRIES: &[HeuristicEntry] = &[
    HeuristicEntry {
        id: HeuristicId("H-ALLOC-01"),
        primary_source: ResidualSource::MemoryUsage,
        drift_threshold: 0.05,
        slew_threshold: 0.02,
        drift_positive_required: true,
        reason_code: ReasonCode::MemoryPressureEscalation,
        description:
            "Monotonic increase in allocation latency with step-change at capacity doubling",
        provenance: "Vec<T> capacity doubling in hot loop; jemalloc arena exhaustion",
    },
    HeuristicEntry {
        id: HeuristicId("H-LOCK-01"),
        primary_source: ResidualSource::Latency,
        drift_threshold: 0.03,
        slew_threshold: 0.01,
        drift_positive_required: true,
        reason_code: ReasonCode::SustainedLatencyDrift,
        description: "Gradual increase in write-hold duration with burst at reader-count threshold",
        provenance: "tokio::sync::RwLock under read-heavy → write-heavy transition",
    },
    HeuristicEntry {
        id: HeuristicId("H-RAFT-01"),
        primary_source: ResidualSource::HeartbeatRtt,
        drift_threshold: 0.04,
        slew_threshold: 0.0,
        drift_positive_required: true,
        reason_code: ReasonCode::ConsensusHeartbeatDegradation,
        description: "Increasing RTT to one follower drifting toward election timeout",
        provenance: "openraft follower with injected clock drift approaching election_timeout_ms",
    },
    HeuristicEntry {
        id: HeuristicId("H-ASYNC-01"),
        primary_source: ResidualSource::PollDuration,
        drift_threshold: 0.02,
        slew_threshold: 0.0,
        drift_positive_required: true,
        reason_code: ReasonCode::AsyncRuntimeStarvation,
        description: "Gradual increase in poll time indicating blocking in async context",
        provenance: "Blocking operation in async context; tokio runtime starvation",
    },
    HeuristicEntry {
        id: HeuristicId("H-TCP-01"),
        primary_source: ResidualSource::Latency,
        drift_threshold: 0.06,
        slew_threshold: 0.03,
        drift_positive_required: true,
        reason_code: ReasonCode::PartialPartitionSignature,
        description: "Burst of retransmits followed by drift in RTT variance",
        provenance: "Partial network partition; selective packet loss on specific routes",
    },
    HeuristicEntry {
        id: HeuristicId("H-CHAN-01"),
        primary_source: ResidualSource::QueueDepth,
        drift_threshold: 0.05,
        slew_threshold: 0.02,
        drift_positive_required: true,
        reason_code: ReasonCode::ChannelBackpressureOnset,
        description: "Growing queue depth with drift-then-slew at backpressure onset",
        provenance: "tokio::sync::mpsc bounded channel approaching capacity",
    },
    HeuristicEntry {
        id: HeuristicId("H-CLOCK-01"),
        primary_source: ResidualSource::HeartbeatRtt,
        drift_threshold: 0.02,
        slew_threshold: 0.0,
        drift_positive_required: false,
        reason_code: ReasonCode::ClockDriftDivergence,
        description: "Monotonic drift in timestamp-derived residuals between nodes",
        provenance: "TSC vs HPET clock source discrepancy between cluster nodes",
    },
    HeuristicEntry {
        id: HeuristicId("H-THRU-01"),
        primary_source: ResidualSource::Throughput,
        drift_threshold: 0.03,
        slew_threshold: 0.0,
        drift_positive_required: false,
        reason_code: ReasonCode::ThroughputDegradation,
        description: "Persistent throughput decline not attributable to workload reduction",
        provenance: "Resource contention from co-located process; IO scheduler starvation",
    },
    HeuristicEntry {
        id: HeuristicId("H-SERDE-01"),
        primary_source: ResidualSource::SerdeLatency,
        drift_threshold: 0.04,
        slew_threshold: 0.02,
        drift_positive_required: true,
        reason_code: ReasonCode::SerializationDrift,
        description: "Serialization latency increasing with step-change at schema boundary",
        provenance: "serde deserialization with growing payload; schema migration overhead",
    },
    HeuristicEntry {
        id: HeuristicId("H-GRPC-01"),
        primary_source: ResidualSource::FlowControlWindow,
        drift_threshold: 0.05,
        slew_threshold: 0.03,
        drift_positive_required: true,
        reason_code: ReasonCode::FlowControlExhaustion,
        description: "Flow control window approaching exhaustion with drift-then-violation",
        provenance: "tonic stream backpressure; h2 flow control window starvation",
    },
    HeuristicEntry {
        id: HeuristicId("H-DNS-01"),
        primary_source: ResidualSource::DnsLatency,
        drift_threshold: 0.03,
        slew_threshold: 0.01,
        drift_positive_required: true,
        reason_code: ReasonCode::SustainedLatencyDrift,
        description: "DNS resolution time increasing with step-change at cache expiry",
        provenance: "trust-dns resolver cache poisoning or upstream resolver degradation",
    },
    HeuristicEntry {
        id: HeuristicId("H-ERR-01"),
        primary_source: ResidualSource::ErrorRate,
        drift_threshold: 0.02,
        slew_threshold: 0.01,
        drift_positive_required: true,
        reason_code: ReasonCode::SustainedLatencyDrift,
        description: "Error rate growing monotonically with acceleration at saturation",
        provenance: "Connection pool exhaustion; timeout cascade in microservice chain",
    },
];

impl HeuristicsBank {
    /// Create a bank with the default Rust distributed-systems entries.
    pub fn default_bank() -> Self {
        Self {
            entries: DEFAULT_ENTRIES,
        }
    }

    /// Create a bank with custom entries.
    pub fn custom(entries: &'static [HeuristicEntry]) -> Self {
        Self { entries }
    }

    /// Match a residual sign against the bank.
    ///
    /// Returns the best-matching heuristic (highest confidence) or
    /// `UnclassifiedStructuralAnomaly` if no entry matches.
    ///
    /// Only matches when the grammar state is `Boundary` or `Violation`.
    /// In `Admissible` state, returns `NoAnomaly`.
    pub fn match_sign(&self, sign: &ResidualSign, grammar_state: GrammarState) -> MatchResult {
        self.match_sign_with_priors(sign, grammar_state, &StaticPriorSet::default())
    }

    /// Match a residual sign against the bank using optional static priors.
    ///
    /// Static priors do not force a detection. They only apply bounded
    /// threshold scaling to the candidate heuristic they target.
    pub fn match_sign_with_priors(
        &self,
        sign: &ResidualSign,
        grammar_state: GrammarState,
        priors: &StaticPriorSet,
    ) -> MatchResult {
        if grammar_state == GrammarState::Admissible {
            return no_anomaly_match();
        }

        let mut best_match: Option<(&HeuristicEntry, f64, Option<AppliedStaticPrior>)> = None;

        for entry in self.entries.iter() {
            if let Some(candidate) = evaluate_heuristic_entry(entry, sign, priors) {
                match best_match {
                    None => best_match = Some(candidate),
                    Some((_, best_conf, _)) if candidate.1 > best_conf => {
                        best_match = Some(candidate)
                    }
                    Some((_, best_conf, _)) if candidate.1 <= best_conf => {}
                    Some((_, _, _)) => {}
                }
            }
        }

        match best_match {
            Some((entry, confidence, applied_prior)) => {
                matched_heuristic_result(entry, confidence, applied_prior)
            }
            None => unmatched_anomaly_result(),
        }
    }

    /// Number of entries in the bank.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the bank is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Bank version identifier.
    pub fn version(&self) -> &'static str {
        "1.0.0"
    }
}

fn no_anomaly_match() -> MatchResult {
    MatchResult {
        reason_code: ReasonCode::NoAnomaly,
        matched_heuristic: None,
        confidence: 0.0,
        description: "No structural anomaly detected",
        provenance: "Grammar state remained Admissible",
        applied_prior: None,
    }
}

fn evaluate_heuristic_entry(
    entry: &'static HeuristicEntry,
    sign: &ResidualSign,
    priors: &StaticPriorSet,
) -> Option<(&'static HeuristicEntry, f64, Option<AppliedStaticPrior>)> {
    if entry.primary_source != sign.source {
        return None;
    }

    let applied_prior = applied_prior_for_entry(entry, priors);
    let (effective_drift_threshold, effective_slew_threshold) =
        effective_thresholds(entry, applied_prior.as_ref());
    if !drift_threshold_matches(entry, sign, effective_drift_threshold) {
        return None;
    }

    Some((
        entry,
        confidence_for_match(sign, effective_drift_threshold, effective_slew_threshold),
        applied_prior,
    ))
}

fn applied_prior_for_entry(
    entry: &HeuristicEntry,
    priors: &StaticPriorSet,
) -> Option<AppliedStaticPrior> {
    priors.get(entry.id).map(|prior| AppliedStaticPrior {
        heuristic_id: prior.heuristic_id,
        confidence: prior.confidence,
        drift_scale: prior.drift_scale,
        slew_scale: prior.slew_scale,
    })
}

fn effective_thresholds(
    entry: &HeuristicEntry,
    applied_prior: Option<&AppliedStaticPrior>,
) -> (f64, f64) {
    let drift_scale = applied_prior.map(|prior| prior.drift_scale).unwrap_or(1.0);
    let slew_scale = applied_prior.map(|prior| prior.slew_scale).unwrap_or(1.0);
    (
        entry.drift_threshold * drift_scale,
        entry.slew_threshold * slew_scale,
    )
}

fn drift_threshold_matches(entry: &HeuristicEntry, sign: &ResidualSign, threshold: f64) -> bool {
    let drift_abs = sign.drift.abs();
    drift_abs >= threshold && !(entry.drift_positive_required && sign.drift < 0.0)
}

fn confidence_for_match(
    sign: &ResidualSign,
    effective_drift_threshold: f64,
    effective_slew_threshold: f64,
) -> f64 {
    let drift_confidence = (sign.drift.abs() / effective_drift_threshold.max(1e-12)).min(3.0) / 3.0;
    let slew_confidence = if effective_slew_threshold > 0.0 {
        let slew_abs = sign.slew.abs();
        if slew_abs < effective_slew_threshold {
            0.3
        } else {
            (slew_abs / effective_slew_threshold.max(1e-12)).min(3.0) / 3.0
        }
    } else {
        0.5
    };

    (drift_confidence + slew_confidence) / 2.0
}

fn matched_heuristic_result(
    entry: &HeuristicEntry,
    confidence: f64,
    applied_prior: Option<AppliedStaticPrior>,
) -> MatchResult {
    MatchResult {
        reason_code: entry.reason_code,
        matched_heuristic: Some(entry.id),
        confidence,
        description: entry.description,
        provenance: entry.provenance,
        applied_prior,
    }
}

fn unmatched_anomaly_result() -> MatchResult {
    MatchResult {
        reason_code: ReasonCode::UnclassifiedStructuralAnomaly,
        matched_heuristic: None,
        confidence: 0.0,
        description: "Structural anomaly detected; no heuristic match",
        provenance: "Grammar state transitioned but no bank entry satisfied its thresholds",
        applied_prior: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sign(source: ResidualSource, drift: f64, slew: f64) -> ResidualSign {
        ResidualSign {
            residual: 5.0,
            drift,
            slew,
            timestamp_ns: 0,
            source,
        }
    }

    #[test]
    fn test_admissible_returns_no_anomaly() {
        let bank = HeuristicsBank::default_bank();
        let sign = make_sign(ResidualSource::Latency, 0.5, 0.1);
        let result = bank.match_sign(&sign, GrammarState::Admissible);
        assert_eq!(result.reason_code, ReasonCode::NoAnomaly);
    }

    #[test]
    fn test_heartbeat_drift_matches_raft_heuristic() {
        let bank = HeuristicsBank::default_bank();
        let sign = make_sign(ResidualSource::HeartbeatRtt, 0.1, 0.0);
        let result = bank.match_sign(&sign, GrammarState::Boundary);
        assert!(
            result.reason_code == ReasonCode::ConsensusHeartbeatDegradation
                || result.reason_code == ReasonCode::ClockDriftDivergence
        );
        assert!(result.matched_heuristic.is_some());
    }

    #[test]
    fn test_unmatched_source_returns_unclassified() {
        let bank = HeuristicsBank::default_bank();
        // Custom source has no heuristic entries
        let sign = make_sign(ResidualSource::Custom("unknown"), 0.5, 0.3);
        let result = bank.match_sign(&sign, GrammarState::Violation);
        assert_eq!(
            result.reason_code,
            ReasonCode::UnclassifiedStructuralAnomaly
        );
    }

    #[test]
    fn test_bank_has_12_entries() {
        let bank = HeuristicsBank::default_bank();
        assert_eq!(bank.len(), 12);
    }
}
