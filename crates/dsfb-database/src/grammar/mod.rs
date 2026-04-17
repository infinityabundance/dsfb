//! Motif grammar layer.
//!
//! The grammar consumes a [`crate::residual::ResidualStream`] and emits a
//! sequence of *episodes*: typed, time-bounded structural events that
//! correspond to operator-recognisable database health states. The grammar
//! is **deterministic** (same input → same output, bytewise) and
//! **observer-only** (no engine state is touched).
//!
//! Each motif class is a small state machine over a single residual class,
//! parameterised by:
//!   * the DSFB observer (drift / slew thresholds via `dsfb::DsfbObserver`)
//!   * an envelope (deterministic threshold band)
//!   * a minimum dwell time (debounce against single-sample blips)
//!
//! Parameters are loaded from `spec/motifs.yaml` (see [`MotifGrammar::from_yaml`])
//! so that the paper, the crate, and the operator's deployment all share the
//! same numbers.

pub mod envelope;
pub mod motifs;
pub mod replay;

use crate::residual::{ResidualClass, ResidualStream};
use serde::{Deserialize, Serialize};

/// One of the five motif classes the paper claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotifClass {
    PlanRegressionOnset,
    CardinalityMismatchRegime,
    ContentionRamp,
    CacheCollapse,
    WorkloadPhaseTransition,
}

impl MotifClass {
    pub const ALL: [MotifClass; 5] = [
        Self::PlanRegressionOnset,
        Self::CardinalityMismatchRegime,
        Self::ContentionRamp,
        Self::CacheCollapse,
        Self::WorkloadPhaseTransition,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::PlanRegressionOnset => "plan_regression_onset",
            Self::CardinalityMismatchRegime => "cardinality_mismatch_regime",
            Self::ContentionRamp => "contention_ramp",
            Self::CacheCollapse => "cache_collapse",
            Self::WorkloadPhaseTransition => "workload_phase_transition",
        }
    }

    pub fn residual_class(&self) -> ResidualClass {
        match self {
            Self::PlanRegressionOnset => ResidualClass::PlanRegression,
            Self::CardinalityMismatchRegime => ResidualClass::Cardinality,
            Self::ContentionRamp => ResidualClass::Contention,
            Self::CacheCollapse => ResidualClass::CacheIo,
            Self::WorkloadPhaseTransition => ResidualClass::WorkloadPhase,
        }
    }
}

/// A single motif episode: a typed structural event with an explicit
/// boundary. The CLI emits these as JSON; the paper figures plot them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub motif: MotifClass,
    pub channel: Option<String>,
    pub t_start: f64,
    pub t_end: f64,
    /// Peak |residual| observed inside the episode (for ranking / plotting).
    pub peak: f64,
    /// EMA-smoothed residual at episode boundary (for traceability into the
    /// DSFB observer's trust state).
    pub ema_at_boundary: f64,
    /// Aggregate trust weight sum across channels at boundary (always 1.0
    /// up to floating-point tolerance — included for audit).
    pub trust_sum: f64,
}

/// Per-motif tunable parameters. Loaded from `spec/motifs.yaml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MotifParams {
    /// DSFB EMA smoothing.
    pub rho: f64,
    /// DSFB trust softness.
    pub sigma0: f64,
    /// Drift envelope: |EMA residual| above this enters drift state.
    pub drift_threshold: f64,
    /// Slew envelope: instantaneous |residual| above this enters boundary
    /// state.
    pub slew_threshold: f64,
    /// Minimum dwell time in seconds; episodes shorter than this are
    /// discarded as blips.
    pub min_dwell_seconds: f64,
}

impl MotifParams {
    /// Conservative defaults used in tests and as the paper's published
    /// baseline — every reported number can be reproduced by leaving these
    /// alone.
    pub fn default_for(class: MotifClass) -> Self {
        match class {
            MotifClass::PlanRegressionOnset => Self {
                rho: 0.9,
                sigma0: 0.05,
                drift_threshold: 0.20,
                slew_threshold: 0.50,
                min_dwell_seconds: 5.0,
            },
            MotifClass::CardinalityMismatchRegime => Self {
                rho: 0.9,
                sigma0: 0.05,
                drift_threshold: 0.5, // log10: 3.16x sustained mismatch
                slew_threshold: 1.0,  // 10x instantaneous
                min_dwell_seconds: 2.0,
            },
            MotifClass::ContentionRamp => Self {
                rho: 0.85,
                sigma0: 0.01,
                drift_threshold: 0.05,
                slew_threshold: 0.5,
                min_dwell_seconds: 1.0,
            },
            MotifClass::CacheCollapse => Self {
                rho: 0.9,
                sigma0: 0.02,
                drift_threshold: 0.10,
                slew_threshold: 0.30,
                min_dwell_seconds: 5.0,
            },
            MotifClass::WorkloadPhaseTransition => Self {
                rho: 0.9,
                sigma0: 0.02,
                drift_threshold: 0.15,
                slew_threshold: 0.35,
                min_dwell_seconds: 30.0,
            },
        }
    }
}

/// The whole grammar: one parameter set per motif class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotifGrammar {
    pub plan_regression_onset: MotifParams,
    pub cardinality_mismatch_regime: MotifParams,
    pub contention_ramp: MotifParams,
    pub cache_collapse: MotifParams,
    pub workload_phase_transition: MotifParams,
}

impl Default for MotifGrammar {
    fn default() -> Self {
        Self {
            plan_regression_onset: MotifParams::default_for(MotifClass::PlanRegressionOnset),
            cardinality_mismatch_regime: MotifParams::default_for(
                MotifClass::CardinalityMismatchRegime,
            ),
            contention_ramp: MotifParams::default_for(MotifClass::ContentionRamp),
            cache_collapse: MotifParams::default_for(MotifClass::CacheCollapse),
            workload_phase_transition: MotifParams::default_for(
                MotifClass::WorkloadPhaseTransition,
            ),
        }
    }
}

impl MotifGrammar {
    pub fn params(&self, class: MotifClass) -> &MotifParams {
        match class {
            MotifClass::PlanRegressionOnset => &self.plan_regression_onset,
            MotifClass::CardinalityMismatchRegime => &self.cardinality_mismatch_regime,
            MotifClass::ContentionRamp => &self.contention_ramp,
            MotifClass::CacheCollapse => &self.cache_collapse,
            MotifClass::WorkloadPhaseTransition => &self.workload_phase_transition,
        }
    }

    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

/// The thing that runs the grammar over a stream and emits episodes.
pub struct MotifEngine {
    grammar: MotifGrammar,
}

impl MotifEngine {
    pub fn new(grammar: MotifGrammar) -> Self {
        Self { grammar }
    }

    /// Run all five motif state machines over the stream. Output is
    /// time-ordered and deterministic for a given (stream, grammar) pair.
    pub fn run(&self, stream: &ResidualStream) -> Vec<Episode> {
        let mut all = Vec::new();
        for class in MotifClass::ALL {
            let params = self.grammar.params(class).clone();
            let eps = motifs::run_motif(class, &params, stream);
            all.extend(eps);
        }
        all.sort_by(|a, b| {
            a.t_start
                .partial_cmp(&b.t_start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all
    }
}
