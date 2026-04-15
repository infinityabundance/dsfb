//! # DSFB Structural Semiotics Engine
//!
//! Deterministic, non-intrusive interpretation of telemetry trajectories for
//! distributed Rust systems.
//!
//! The core observer converts runtime telemetry streams into typed structural
//! objects defined by residuals, drift, slew, admissibility envelopes, grammar
//! states, and reason codes. With the default `std` feature enabled, this crate
//! also includes the deterministic fault-injection harness and plain-text/CSV
//! report generation used by the demo binary.
//!
//! ## What This Crate Does
//!
//! - Observes telemetry through immutable references only
//! - Classifies trajectories as `Admissible`, `Boundary`, or `Violation`
//! - Emits deterministic reason codes from a finite heuristics bank
//! - Records an audit trace for replay and verification
//!
//! ## Feature Layout
//!
//! - Core observer modules are available in `no_std` mode
//! - Scenario injection and report generation require the default `std` feature
//! - Static crate scanning and attestation export require the default `std` feature
//!
//! ## Example
//!
//! ```rust
//! use dsfb_gray::{DsfbObserver, ObserverConfig, ResidualSample, ResidualSource};
//!
//! let config = ObserverConfig::fast_response();
//! let mut observer = DsfbObserver::new(ResidualSource::Latency, &config);
//! let sample = ResidualSample {
//!     value: 12.0,
//!     baseline: 10.0,
//!     timestamp_ns: 1_000,
//!     source: ResidualSource::Latency,
//! };
//! let result = observer.observe(&sample);
//! assert_eq!(result.sign.residual, 2.0);
//! ```
//!
//! ## Non-Interference Contract
//!
//! **Contract Version 1.0**: The observer accepts telemetry through immutable
//! references only (`&ResidualSample`). No mutable reference to any upstream
//! system component is created, stored, or transmitted.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod adapter;
mod audit;
mod envelope;
mod episode;
mod grammar;
mod heuristics;
mod observer;
mod regime;
mod residual;

#[cfg(feature = "std")]
mod evaluation;
#[cfg(feature = "std")]
mod inject;
#[cfg(feature = "std")]
mod report;
#[cfg(feature = "std")]
mod scan;

pub use adapter::{IdentityTelemetryAdapter, TelemetryAdapter};
pub use audit::{AuditEvent, AuditTrace};
pub use envelope::{AdmissibilityEnvelope, EnvelopePosition};
pub use episode::{Episode, EpisodeBuilder};
pub use grammar::{GrammarState, GrammarTransition};
pub use heuristics::{AppliedStaticPrior, StaticPrior, StaticPriorSet};
pub use heuristics::{HeuristicEntry, HeuristicId, HeuristicsBank, MatchResult};
pub use observer::{
    DsfbObserver, MultiChannelObserver, ObservationResult, ObserverConfig, ReasonEvidence,
};
pub use regime::{RegimeClassifier, WorkloadPhase};
pub use residual::{ResidualSample, ResidualSign, ResidualSource};

#[cfg(feature = "std")]
pub use evaluation::{
    build_public_evaluation, render_public_evaluation_report, reproducibility_verified,
    write_public_artifacts, NegativeControlRow, PrimaryEvaluationRow, PublicArtifactPaths,
    PublicEvaluationBundle, SensitivitySweepRow,
};
#[cfg(feature = "std")]
pub use inject::{
    run_scenario, AsyncStarvationScenario, ChannelBackpressureScenario, ClockDriftScenario,
    FaultScenario, PartialPartitionScenario, SampleRecord, ScenarioResult,
};
#[cfg(feature = "std")]
pub use report::{generate_csv, generate_report};
#[cfg(feature = "std")]
pub use scan::{
    derive_static_priors_from_scan, export_scan_artifacts, migrate_legacy_scan_artifacts,
    prepare_scan_output_run, render_scan_attestation_statement, render_scan_dsse_envelope,
    render_scan_report, render_scan_sarif, scan_crate_source, scan_crate_source_with_profile,
    CrateSourceScanReport, HeuristicSourceMatch, ScanArtifactPaths, ScanEvidence, ScanProfile,
    ScanRunPaths, ScanSigningKey, DEFAULT_SCAN_OUTPUT_ROOT,
};

/// Reason codes for distributed system structural interpretations.
///
/// Each variant encodes a specific structural pattern recognized by the
/// heuristics bank. `UnclassifiedStructuralAnomaly` is emitted when the
/// grammar state transitions to `Boundary` or `Violation` but no heuristic
/// entry matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    /// Sustained monotonic increase in latency residuals across multiple
    /// observation windows, consistent with resource exhaustion or degradation.
    SustainedLatencyDrift,
    /// Consensus heartbeat round-trip time shows persistent directional
    /// drift toward election timeout boundary.
    ConsensusHeartbeatDegradation,
    /// Tokio runtime task poll duration increasing monotonically, consistent
    /// with blocking operations in async context or runtime starvation.
    AsyncRuntimeStarvation,
    /// Bounded channel queue depth approaching capacity with characteristic
    /// drift-then-slew signature at backpressure onset.
    ChannelBackpressureOnset,
    /// Asymmetric connectivity pattern: some node pairs communicate normally
    /// while others show persistent latency drift or packet loss.
    PartialPartitionSignature,
    /// Clock source divergence between nodes producing monotonic drift in
    /// timestamp-derived residuals.
    ClockDriftDivergence,
    /// RSS or allocation rate showing persistent growth trajectory beyond
    /// admissibility envelope for the current workload phase.
    MemoryPressureEscalation,
    /// Throughput (ops/sec or bytes/sec) showing persistent decline not
    /// attributable to workload reduction.
    ThroughputDegradation,
    /// Serialization or deserialization latency increasing with characteristic
    /// step-change at payload size or schema version boundaries.
    SerializationDrift,
    /// gRPC or HTTP/2 flow control window approaching exhaustion with
    /// characteristic drift-then-violation pattern.
    FlowControlExhaustion,
    /// Grammar state transitioned but no heuristic bank entry matched.
    UnclassifiedStructuralAnomaly,
    /// No anomaly detected. Residual trajectory remains within admissibility
    /// envelope. Grammar state is Admissible.
    NoAnomaly,
}

impl ReasonCode {
    /// Returns a human-readable description of this reason code.
    pub fn description(&self) -> &'static str {
        match self {
            Self::SustainedLatencyDrift => {
                "Sustained monotonic latency increase across observation windows"
            }
            Self::ConsensusHeartbeatDegradation => {
                "Consensus heartbeat RTT drifting toward election timeout"
            }
            Self::AsyncRuntimeStarvation => {
                "Async runtime task poll duration increasing monotonically"
            }
            Self::ChannelBackpressureOnset => {
                "Bounded channel approaching capacity with drift-slew onset"
            }
            Self::PartialPartitionSignature => {
                "Asymmetric connectivity: selective latency drift between nodes"
            }
            Self::ClockDriftDivergence => {
                "Clock source divergence producing timestamp residual drift"
            }
            Self::MemoryPressureEscalation => {
                "Memory growth trajectory exceeding workload-phase envelope"
            }
            Self::ThroughputDegradation => {
                "Persistent throughput decline not attributable to workload"
            }
            Self::SerializationDrift => {
                "Serialization latency drift with payload/schema boundary slew"
            }
            Self::FlowControlExhaustion => "Flow control window approaching exhaustion",
            Self::UnclassifiedStructuralAnomaly => {
                "Structural anomaly detected; no heuristic match"
            }
            Self::NoAnomaly => "No structural anomaly detected",
        }
    }
}

/// Crate version for audit trace embedding.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Non-interference contract version.
pub const CONTRACT_VERSION: &str = "1.0";
