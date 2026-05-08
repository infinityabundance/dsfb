//! DSFB-Debug: engine configuration — paper §F.4 + Appendix F.
//!
//! `EngineConfig` carries every operator-tunable parameter for the
//! DSFB pipeline. The `paper_lock()` factory pins the entire config
//! to the values reported in paper §13 — the canonical settings under
//! which all empirical claims are reproducible.
//!
//! # Paper-lock parameters
//!
//! Each pinned value is documented in-source with its paper §F.4
//! anchor and operational rationale (e.g. `drift_window = 7` because
//! it captures sustained drift without over-smoothing transients;
//! `n_confirm = 2` because Boundary→Violation transitions need a
//! two-window confirmation to suppress single-window touches).
//!
//! Operators running on real production residuals SHOULD start with
//! `paper_lock()` and adjust per `calibration::recommend_config_from_healthy`
//! recommendations. The calibration tool is advisory — operators
//! decide whether to apply.

use crate::error::{DsfbError, Result};

/// Engine configuration — all parameters from paper's fixed protocol
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EngineConfig {
    /// W: consecutive windows for drift estimation (paper: 5)
    pub drift_window: usize,
    /// DSA drift window (paper: 10)
    pub dsa_window: usize,
    /// K: minimum grammar hits for state confirmation (paper: 4)
    pub persistence_threshold: usize,
    /// τ: directional consistency threshold for DSA (paper: 2.0)
    pub consistency_gate: f64,
    /// m: minimum co-activating signals for run-level alert (paper: 1)
    pub corroboration_count: usize,
    /// n_confirm: consecutive steps for hysteresis (paper: 2)
    pub hysteresis_confirm: usize,
    /// Fraction of ρ for boundary entry (paper: 0.5)
    pub boundary_fraction: f64,
    /// Windows for temporal episode grouping
    pub episode_correlation_window: u64,
    /// W_pred: look-ahead for episode precision (paper: 5)
    pub episode_precision_window: u64,
    /// Minimum healthy windows for baseline (paper: 100)
    pub min_healthy_windows: usize,
    /// Slew threshold δ_s for slew density computation
    pub slew_delta: f64,
}

/// Paper-lock configuration — exact parameters from the paper
/// Used by paper-lock CI test to verify headline metrics
pub const PAPER_LOCK_CONFIG: EngineConfig = EngineConfig {
    drift_window: 5,
    dsa_window: 10,
    persistence_threshold: 4,
    consistency_gate: 2.0,
    corroboration_count: 1,
    hysteresis_confirm: 2,
    boundary_fraction: 0.5,
    episode_correlation_window: 5,
    episode_precision_window: 5,
    min_healthy_windows: 100,
    slew_delta: 0.1,
};

impl EngineConfig {
    pub fn validate(&self) -> Result<()> {
        if self.drift_window == 0 {
            return Err(DsfbError::InvalidConfig("drift_window must be > 0"));
        }
        if self.dsa_window == 0 {
            return Err(DsfbError::InvalidConfig("dsa_window must be > 0"));
        }
        if self.persistence_threshold == 0 {
            return Err(DsfbError::InvalidConfig("persistence_threshold must be > 0"));
        }
        if self.consistency_gate <= 0.0 {
            return Err(DsfbError::InvalidConfig("consistency_gate must be > 0"));
        }
        if self.corroboration_count == 0 {
            return Err(DsfbError::InvalidConfig("corroboration_count must be > 0"));
        }
        if self.hysteresis_confirm == 0 {
            return Err(DsfbError::InvalidConfig("hysteresis_confirm must be > 0"));
        }
        if self.boundary_fraction <= 0.0 || self.boundary_fraction >= 1.0 {
            return Err(DsfbError::InvalidConfig("boundary_fraction must be in (0, 1)"));
        }
        Ok(())
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        PAPER_LOCK_CONFIG
    }
}
