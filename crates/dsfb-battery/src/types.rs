// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Type definitions
//
// All types correspond to formal objects defined in:
//   "DSFB Structural Semiotics Engine for Battery Health Monitoring"
//   by Riaan de Beer, Version 1.0.

use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Definition 1 (Paper): Residual Sign Tuple
//   σ_k = (r_k, d_k, s_k)
//   r_k = y_k − ŷ_k        (residual)
//   d_k = r_k − r_{k−1}    (drift / first difference)
//   s_k = d_k − d_{k−1}    (slew / second difference)
// ---------------------------------------------------------------------------

/// A single sign tuple at cycle `k`, per Definition 1 of the paper.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignTuple {
    /// Residual: r_k = y_k − ŷ_k (capacity deviation from nominal, in Ah).
    pub r: f64,
    /// Drift: d_k (windowed first difference of residual, in Ah/cycle).
    pub d: f64,
    /// Slew: s_k (windowed second difference of residual, in Ah/cycle²).
    pub s: f64,
}

// ---------------------------------------------------------------------------
// Full per-cycle record combining raw data with semiotic analysis
// ---------------------------------------------------------------------------

/// Complete per-cycle battery residual record, combining raw capacity data
/// with the computed semiotic sign tuple and grammar classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryResidual {
    /// Cycle number (1-indexed).
    pub cycle: usize,
    /// Measured discharge capacity (Ah).
    pub capacity_ah: f64,
    /// Computed sign tuple (r_k, d_k, s_k) per Definition 1.
    pub sign: SignTuple,
    /// Grammar state classification per Definition 2.
    pub grammar_state: GrammarState,
    /// Optional typed reason code per Section 5.
    pub reason_code: Option<ReasonCode>,
}

// ---------------------------------------------------------------------------
// Definition 2 (Paper): Battery Grammar State
//   Three-level finite-state classification:
//     Admissible — residuals inside healthy envelope, no persistent outward drift
//     Boundary   — residuals near envelope or persistent outward drift without exit
//     Violation  — residuals exit admissible envelope
// ---------------------------------------------------------------------------

/// Grammar state at a given cycle, per Definition 2 of the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrammarState {
    /// Residuals inside healthy envelope; no persistent outward drift.
    Admissible,
    /// Residuals near or grazing envelope, or persistent outward drift
    /// without envelope exit.
    Boundary,
    /// Residuals exit admissible envelope or cross-channel configuration
    /// incompatible with regime.
    Violation,
}

impl fmt::Display for GrammarState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrammarState::Admissible => write!(f, "Admissible"),
            GrammarState::Boundary => write!(f, "Boundary"),
            GrammarState::Violation => write!(f, "Violation"),
        }
    }
}

// ---------------------------------------------------------------------------
// Section 5 (Paper): Battery-Domain Reason Codes
//   Typed structural interpretations under declared conditions.
// ---------------------------------------------------------------------------

/// Typed battery-domain reason code, per Section 5 of the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasonCode {
    /// Classification suppressed because the upstream sample stream or its
    /// fixed-window derivatives were invalid for this interval.
    InvalidStreamSuppression,
    /// Low-curvature monotone fade (e.g., SEI growth regime).
    SustainedCapacityFade,
    /// Sudden resistance increase (single-event or step change).
    AbruptResistanceSpike,
    /// Repeated envelope approach in voltage channel.
    RecurrentVoltageGrazing,
    /// Degradation coupled to thermal regime variation.
    ThermalDriftCoupling,
    /// Growing cell-to-cell spread in pack context.
    PackImbalanceExpansion,
    /// Transition from gradual to rapid degradation (knee onset).
    AcceleratingFadeKnee,
    /// Coupled resistance + voltage anomalies under aggressive conditions.
    PossibleLithiumPlatingSignature,
    /// Temperature-linked spike that reverses (not persistent degradation).
    TransientThermalExcursionNotPersistent,
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReasonCode::InvalidStreamSuppression => write!(f, "InvalidStreamSuppression"),
            ReasonCode::SustainedCapacityFade => write!(f, "SustainedCapacityFade"),
            ReasonCode::AbruptResistanceSpike => write!(f, "AbruptResistanceSpike"),
            ReasonCode::RecurrentVoltageGrazing => write!(f, "RecurrentVoltageGrazing"),
            ReasonCode::ThermalDriftCoupling => write!(f, "ThermalDriftCoupling"),
            ReasonCode::PackImbalanceExpansion => write!(f, "PackImbalanceExpansion"),
            ReasonCode::AcceleratingFadeKnee => write!(f, "AcceleratingFadeKnee"),
            ReasonCode::PossibleLithiumPlatingSignature => {
                write!(f, "PossibleLithiumPlatingSignature")
            }
            ReasonCode::TransientThermalExcursionNotPersistent => {
                write!(f, "TransientThermalExcursionNotPersistent")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Definition 3 (Paper): Admissibility Envelope
//   Constructed from healthy-window baseline variability (±3σ).
//   E_k ⊆ ℝ^m, regime-indexed.
//
//   For scalar health channel y_k with baseline window N_0:
//     μ_y^(0) = (1/N_0) Σ y_k
//     σ_y^(0) = sqrt((1/(N_0−1)) Σ (y_k − μ_y^(0))²)
//     ρ_y = 3 σ_y^(0)
//     Admissible iff |r_k^(y)| ≤ ρ_y
// ---------------------------------------------------------------------------

/// Admissibility envelope parameters computed from the healthy baseline window,
/// per Definition 3 of the paper.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnvelopeParams {
    /// Mean capacity over the healthy baseline window (Ah).
    pub mu: f64,
    /// Standard deviation of capacity over the healthy baseline window (Ah).
    pub sigma: f64,
    /// Envelope radius: ρ = 3σ (Ah).
    pub rho: f64,
}

// ---------------------------------------------------------------------------
// Detection result: comparison between DSFB and threshold baseline
// ---------------------------------------------------------------------------

/// Detection result for a single method, recording when alarm was raised
/// and the resulting lead time relative to end-of-life.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Name of the detection method.
    pub method: String,
    /// Cycle at which alarm was first raised (1-indexed), or None if never.
    pub alarm_cycle: Option<usize>,
    /// End-of-life cycle (first cycle where C_k < C_EOL).
    pub eol_cycle: Option<usize>,
    /// Lead time in cycles: eol_cycle − alarm_cycle.
    pub lead_time_cycles: Option<i64>,
}

// ---------------------------------------------------------------------------
// Definition 4 (Paper): Typed Heuristic Bank Entry
//   H_j = (P_j, R_j, A_j, I_j, U_j)
//   P = pattern descriptor, R = regime scope, A = admissibility assumptions,
//   I = candidate interpretation, U = ambiguity/uncertainty notes
// ---------------------------------------------------------------------------

/// A typed heuristic bank entry, per Definition 4 of the paper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicBankEntry {
    /// Pattern descriptor: characteristic semiotic signature.
    pub pattern: String,
    /// Regime scope: operating conditions under which the entry applies.
    pub regime_scope: String,
    /// Admissibility assumptions: what must hold for the entry to apply.
    pub admissibility_assumptions: String,
    /// Candidate interpretation: typed degradation motif name.
    pub interpretation: String,
    /// Ambiguity/uncertainty notes: caveats and alternative possibilities.
    pub uncertainty_notes: String,
}

// ---------------------------------------------------------------------------
// Pipeline configuration
// ---------------------------------------------------------------------------

/// Configuration for the DSFB battery pipeline, per the Stage II benchmark
/// specification in Section 8 of the paper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Number of initial cycles used as healthy reference window (N_h).
    /// Paper default: 20 cycles.
    pub healthy_window: usize,
    /// Window width for drift estimation (W).
    /// Paper default: 5 cycles.
    pub drift_window: usize,
    /// Persistence length for drift alarm (L_d): number of consecutive
    /// cycles with drift exceeding threshold before grammar transition.
    /// Paper default: 12 cycles.
    pub drift_persistence: usize,
    /// Persistence length for slew alarm (L_s): number of consecutive
    /// cycles with slew exceeding threshold before acceleration declared.
    /// Paper default: 8 cycles.
    pub slew_persistence: usize,
    /// Drift threshold (θ_d) in Ah/cycle. Outward drift exceeding this
    /// for L_d consecutive cycles triggers Boundary state.
    pub drift_threshold: f64,
    /// Slew threshold (θ_s) in Ah/cycle². Slew exceeding this for L_s
    /// consecutive cycles triggers acceleration detection.
    pub slew_threshold: f64,
    /// Fraction of initial capacity defining end-of-life (e.g. 0.80 for 80%).
    pub eol_fraction: f64,
    /// Boundary fraction: residual magnitude relative to envelope radius
    /// above which the state transitions to Boundary (e.g. 0.8 = 80% of ρ).
    pub boundary_fraction: f64,
}

impl Default for PipelineConfig {
    /// Default configuration matching the Stage II benchmark specification
    /// in Section 8 of the paper.
    fn default() -> Self {
        Self {
            healthy_window: 20,
            drift_window: 5,
            drift_persistence: 12,
            slew_persistence: 8,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        }
    }
}

// ---------------------------------------------------------------------------
// Theorem 1 verification result
// ---------------------------------------------------------------------------

/// Result of verifying Theorem 1 (Discrete-Time Finite Envelope Exit
/// Under Sustained Outward Drift) against observed data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theorem1Result {
    /// Envelope radius ρ (Ah).
    pub rho: f64,
    /// Observed sustained outward drift rate α (Ah/cycle), estimated as
    /// the minimum per-cycle outward drift η over the relevant window.
    pub alpha: f64,
    /// Maximum envelope expansion per cycle κ (for static envelope, κ = 0).
    pub kappa: f64,
    /// Computed exit bound: t* = ⌈ρ / (α − κ)⌉ (cycles).
    pub t_star: usize,
    /// Actual cycle at which DSFB first detected envelope exit or Boundary.
    pub actual_detection_cycle: Option<usize>,
    /// Whether the theorem bound was satisfied: t* ≥ actual detection lag.
    pub bound_satisfied: Option<bool>,
}
