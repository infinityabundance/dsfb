use serde::{Deserialize, Serialize};

/// Deterministic syntax thresholds used by the syntax layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxThresholds {
    pub sign_deadband: f64,
    pub margin_deadband: f64,
    pub slew_spike_sigma_factor: f64,
    pub slew_spike_floor: f64,
    pub coordinated_rise_min_group_breach_fraction: f64,
    pub coordinated_rise_min_outward_fraction: f64,
    pub coordinated_rise_min_channel_alignment: f64,
    pub coordinated_rise_min_radial_persistence: f64,
    pub persistent_outward_min_fraction: f64,
    pub persistent_outward_min_path_monotonicity: f64,
    pub persistent_outward_min_radial_persistence: f64,
    pub persistent_outward_max_mean_squared_slew: f64,
    pub persistent_outward_max_late_slew_growth: f64,
    pub inward_containment_min_fraction: f64,
    pub discrete_event_min_spike_strength: f64,
    pub discrete_event_min_max_slew_norm: f64,
    pub discrete_event_min_late_slew_growth: f64,
    pub curvature_transition_min_late_slew_growth: f64,
    pub curvature_transition_min_mean_squared_slew: f64,
    pub curvature_transition_min_max_slew_norm: f64,
    pub near_boundary_min_episode_count: usize,
    pub baseline_like_max_outward_inward_imbalance: f64,
    pub baseline_like_max_path_monotonicity: f64,
    pub baseline_like_max_mean_squared_slew: f64,
    pub baseline_like_max_slew_norm: f64,
    pub baseline_like_max_late_slew_growth: f64,
    pub baseline_like_max_spike_strength: f64,
    pub oscillatory_max_path_monotonicity: f64,
    pub oscillatory_min_sign_persistence: f64,
    pub oscillatory_max_violation_fraction: f64,
    pub oscillatory_min_outward_inward_balance: f64,
    pub oscillatory_min_max_slew_norm: f64,
    pub oscillatory_max_slew_spike_strength: f64,
    pub noisy_min_slew_spike_count: usize,
    pub noisy_min_mean_squared_slew: f64,
    pub noisy_min_outward_inward_balance: f64,
    pub curvature_transition_spike_strength_floor: f64,
    pub curvature_transition_spike_norm_floor: f64,
}

/// Deterministic semantic retrieval thresholds that are not encoded inside individual bank
/// entries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticRetrievalSettings {
    pub comparison_epsilon: f64,
    pub observation_limited_max_directional_fraction: f64,
    pub observation_limited_max_radial_persistence: f64,
    pub observation_limited_max_radial_dominance: f64,
    pub observation_limited_max_late_slew_growth: f64,
}

/// Deterministic sign-generator preconditioning mode.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmoothingMode {
    Disabled,
    ExponentialMovingAverage,
    SafetyFirst,
}

/// Deterministic optional smoothing settings used before numerical differentiation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmoothingSettings {
    pub mode: SmoothingMode,
    pub exponential_alpha: f64,
    pub causal_window: usize,
}

impl SmoothingSettings {
    /// Returns whether smoothing is active for sign generation.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        !matches!(self.mode, SmoothingMode::Disabled)
    }

    /// Returns the machine-readable smoothing-profile label exported in metadata.
    #[must_use]
    pub const fn profile_label(&self) -> &'static str {
        match self.mode {
            SmoothingMode::Disabled => "disabled",
            SmoothingMode::ExponentialMovingAverage => "default_low_latency",
            SmoothingMode::SafetyFirst => "safety_first",
        }
    }

    /// Returns the estimated centroid delay in samples for the active profile.
    #[must_use]
    pub fn estimated_lag_samples(&self) -> f64 {
        match self.mode {
            SmoothingMode::Disabled => 0.0,
            SmoothingMode::ExponentialMovingAverage => {
                let alpha = self.exponential_alpha.clamp(1.0e-6, 1.0);
                ((1.0 - alpha) / alpha).clamp(0.0, 8.0)
            }
            SmoothingMode::SafetyFirst => self.maximum_settling_samples() as f64 / 2.0,
        }
    }

    /// Returns the conservative maximum settling horizon in samples for the active profile.
    #[must_use]
    pub fn maximum_settling_samples(&self) -> usize {
        match self.mode {
            SmoothingMode::Disabled => 0,
            SmoothingMode::ExponentialMovingAverage => 1,
            SmoothingMode::SafetyFirst => self.causal_window.max(2).saturating_sub(1),
        }
    }

    /// Returns conservative integration guidance for guidance-loop users.
    #[must_use]
    pub const fn guidance_loop_caution_note(&self) -> &'static str {
        match self.mode {
            SmoothingMode::Disabled => {
                "No derivative preconditioning is active. Jitter enters the finite-difference path directly."
            }
            SmoothingMode::ExponentialMovingAverage => {
                "Low-latency smoothing is active. Treat the derivative path as slightly delayed and confirm the added lag against the downstream control stack."
            }
            SmoothingMode::SafetyFirst => {
                "Safety-first smoothing prioritizes jitter attenuation over immediacy. Use the exported lag bound as an integration aid rather than as a closed-loop stability guarantee."
            }
        }
    }

    /// Returns a preconfigured safety-first profile with conservative bounded lag.
    #[must_use]
    pub fn safety_first() -> Self {
        Self {
            mode: SmoothingMode::SafetyFirst,
            exponential_alpha: 0.18,
            causal_window: 5,
        }
    }
}

impl SmoothingMode {
    /// Returns the machine-readable smoothing label.
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::ExponentialMovingAverage => "exponential_moving_average",
            Self::SafetyFirst => "safety_first",
        }
    }
}

/// Deterministic indexed-retrieval settings for larger semantic banks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalIndexSettings {
    pub enabled: bool,
    pub minimum_bank_size: usize,
    pub export_latency_report: bool,
    pub benchmark_scaling_points: Vec<usize>,
}

/// Deterministic empirical evaluation settings for baseline comparators and sweeps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluationSettings {
    pub residual_threshold_scale: f64,
    pub moving_average_window: usize,
    pub moving_average_trend_deadband: f64,
    pub cusum_drift_allowance: f64,
    pub cusum_alarm_threshold: f64,
    pub slew_spike_sigma_factor: f64,
    pub slew_spike_floor: f64,
    pub innovation_detector_scale: f64,
    pub innovation_alarm_threshold: f64,
    pub default_sweep_points: usize,
}

/// Deterministic plotting settings used by artifact integrity checks and exported figure metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlottingSettings {
    pub count_like_integer_tolerance: f64,
}

/// Deterministic bounded-history settings for the deployment-oriented online engine path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnlineEngineSettings {
    pub history_buffer_capacity: usize,
    pub offline_history_enabled: bool,
    pub numeric_mode: String,
}

/// Deterministic formatting settings for report- and evaluation-facing summaries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportingSettings {
    pub small_value_threshold: f64,
    pub compact_precision: usize,
    pub detailed_precision: usize,
}

/// Top-level deterministic engine settings captured in run metadata.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EngineSettings {
    pub syntax: SyntaxThresholds,
    pub semantics: SemanticRetrievalSettings,
    pub smoothing: SmoothingSettings,
    pub retrieval_index: RetrievalIndexSettings,
    pub evaluation: EvaluationSettings,
    pub plotting: PlottingSettings,
    pub reporting: ReportingSettings,
    pub online: OnlineEngineSettings,
}

impl Default for SyntaxThresholds {
    fn default() -> Self {
        Self {
            sign_deadband: 1.0e-6,
            margin_deadband: 1.0e-6,
            slew_spike_sigma_factor: 1.5,
            slew_spike_floor: 1.0e-4,
            coordinated_rise_min_group_breach_fraction: 0.08,
            coordinated_rise_min_outward_fraction: 0.45,
            coordinated_rise_min_channel_alignment: 0.55,
            coordinated_rise_min_radial_persistence: 0.45,
            persistent_outward_min_fraction: 0.68,
            persistent_outward_min_path_monotonicity: 0.74,
            persistent_outward_min_radial_persistence: 0.70,
            persistent_outward_max_mean_squared_slew: 0.02,
            persistent_outward_max_late_slew_growth: 0.35,
            inward_containment_min_fraction: 0.60,
            discrete_event_min_spike_strength: 0.02,
            discrete_event_min_max_slew_norm: 0.01,
            discrete_event_min_late_slew_growth: 0.40,
            curvature_transition_min_late_slew_growth: 0.45,
            curvature_transition_min_mean_squared_slew: 0.02,
            curvature_transition_min_max_slew_norm: 0.01,
            near_boundary_min_episode_count: 3,
            baseline_like_max_outward_inward_imbalance: 0.08,
            baseline_like_max_path_monotonicity: 0.08,
            baseline_like_max_mean_squared_slew: 1.0e-5,
            baseline_like_max_slew_norm: 0.002,
            baseline_like_max_late_slew_growth: 0.20,
            baseline_like_max_spike_strength: 1.0e-3,
            oscillatory_max_path_monotonicity: 0.40,
            oscillatory_min_sign_persistence: 0.40,
            oscillatory_max_violation_fraction: 0.0,
            oscillatory_min_outward_inward_balance: 0.65,
            oscillatory_min_max_slew_norm: 0.005,
            oscillatory_max_slew_spike_strength: 0.003,
            noisy_min_slew_spike_count: 2,
            noisy_min_mean_squared_slew: 0.002,
            noisy_min_outward_inward_balance: 0.45,
            curvature_transition_spike_strength_floor: 0.015,
            curvature_transition_spike_norm_floor: 0.005,
        }
    }
}

impl Default for SemanticRetrievalSettings {
    fn default() -> Self {
        Self {
            comparison_epsilon: 1.0e-9,
            observation_limited_max_directional_fraction: 0.35,
            observation_limited_max_radial_persistence: 0.35,
            observation_limited_max_radial_dominance: 0.35,
            observation_limited_max_late_slew_growth: 0.15,
        }
    }
}

impl Default for SmoothingSettings {
    fn default() -> Self {
        Self {
            mode: SmoothingMode::Disabled,
            exponential_alpha: 0.35,
            causal_window: 5,
        }
    }
}

impl Default for RetrievalIndexSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            minimum_bank_size: 24,
            export_latency_report: true,
            benchmark_scaling_points: vec![16, 64, 256],
        }
    }
}

impl Default for EvaluationSettings {
    fn default() -> Self {
        Self {
            residual_threshold_scale: 1.0,
            moving_average_window: 7,
            moving_average_trend_deadband: 1.0e-4,
            cusum_drift_allowance: 5.0e-4,
            cusum_alarm_threshold: 0.05,
            slew_spike_sigma_factor: 1.5,
            slew_spike_floor: 1.0e-4,
            innovation_detector_scale: 1.0,
            innovation_alarm_threshold: 1.0,
            default_sweep_points: 5,
        }
    }
}

impl Default for PlottingSettings {
    fn default() -> Self {
        Self {
            count_like_integer_tolerance: 1.0e-9,
        }
    }
}

impl Default for OnlineEngineSettings {
    fn default() -> Self {
        Self {
            history_buffer_capacity: 64,
            offline_history_enabled: false,
            numeric_mode: if cfg!(feature = "numeric-fixed") {
                "fixed_q16_16".to_string()
            } else if cfg!(feature = "numeric-f32") {
                "f32".to_string()
            } else {
                "f64".to_string()
            },
        }
    }
}

impl Default for ReportingSettings {
    fn default() -> Self {
        Self {
            small_value_threshold: 1.0e-3,
            compact_precision: 3,
            detailed_precision: 6,
        }
    }
}
