//! DSFB engine configuration.
//!
//! All parameters that govern grammar transitions, envelope construction,
//! and persistence rules are declared here. Configuration is immutable
//! after construction — there is no runtime mutation.

/// Complete DSFB engine configuration. Immutable after construction.
///
/// Every parameter that affects DSFB output is declared in this struct.
/// Reproducibility requires that this configuration be version-locked
/// to the paper and crate version.
#[derive(Debug, Clone, Copy)]
pub struct DsfbConfig {
    /// Number of initial cycles used to construct the healthy-window baseline.
    /// Residuals are computed relative to the mean of these cycles.
    pub healthy_window: usize,

    /// Sliding window length for drift estimation (first discrete difference).
    pub drift_window: usize,

    /// Sliding window length for slew estimation (second discrete difference).
    pub slew_window: usize,

    /// Number of consecutive cycles of sustained drift required to trigger
    /// a grammar transition from Admissible to Boundary.
    pub persistence_threshold: usize,

    /// Number of consecutive cycles of sustained slew required to trigger
    /// a grammar transition from Boundary to Violation.
    pub slew_persistence_threshold: usize,

    /// Envelope width multiplier (in units of healthy-window standard deviation).
    /// Envelope = healthy_mean ± envelope_sigma * healthy_std.
    pub envelope_sigma: f64,

    /// Minimum absolute drift rate (per cycle) to be considered structurally
    /// significant. Below this, drift is treated as noise.
    pub drift_floor: f64,

    /// Minimum absolute slew rate (per cycle) to be considered structurally
    /// significant. Below this, slew is treated as noise.
    pub slew_floor: f64,

    /// Fraction of informative channels that must independently signal
    /// Boundary or Violation for a multi-channel grammar transition.
    /// Range: 0.0 to 1.0. Value of 0.5 means majority vote.
    pub channel_vote_fraction: f64,

    /// Maximum number of channels tracked simultaneously.
    /// Fixed at compile time for no_alloc compatibility.
    pub max_channels: usize,
}

impl DsfbConfig {
    /// Default configuration for C-MAPSS FD001 evaluation.
    ///
    /// These values are the declared starting configuration.
    /// Sensitivity analysis varies each parameter independently.
    #[must_use]
    pub const fn cmapss_fd001_default() -> Self {
        Self {
            healthy_window: 20,
            drift_window: 10,
            slew_window: 10,
            persistence_threshold: 15,
            slew_persistence_threshold: 10,
            envelope_sigma: 2.5,
            drift_floor: 0.001,
            slew_floor: 0.0005,
            channel_vote_fraction: 0.3,
            max_channels: 14,
        }
    }

    /// Configuration for C-MAPSS FD003 (two fault modes).
    #[must_use]
    pub const fn cmapss_fd003_default() -> Self {
        Self {
            healthy_window: 20,
            drift_window: 10,
            slew_window: 10,
            persistence_threshold: 15,
            slew_persistence_threshold: 10,
            envelope_sigma: 2.5,
            drift_floor: 0.001,
            slew_floor: 0.0005,
            channel_vote_fraction: 0.3,
            max_channels: 14,
        }
    }

    /// Configuration for C-MAPSS FD002 (six operating conditions).
    /// Slightly wider envelope to account for regime variability.
    #[must_use]
    pub const fn cmapss_fd002_default() -> Self {
        Self {
            healthy_window: 25,
            drift_window: 12,
            slew_window: 12,
            persistence_threshold: 18,
            slew_persistence_threshold: 12,
            envelope_sigma: 3.0,
            drift_floor: 0.001,
            slew_floor: 0.0005,
            channel_vote_fraction: 0.3,
            max_channels: 14,
        }
    }
}

impl Default for DsfbConfig {
    fn default() -> Self {
        Self::cmapss_fd001_default()
    }
}
