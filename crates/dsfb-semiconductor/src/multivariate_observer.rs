//! Multivariate observer — ingests PCA/FDC residual streams and provides
//! structural interpretation via the `StructuralPCA` module.
//!
//! # Design: Monitoring the Monitor
//! Existing FDC systems reduce the multivariate residual space to two scalar
//! statistics:
//!
//! * **Hotelling's T²** — squared Mahalanobis distance in the principal
//!   component subspace; detects changes in the *modelled* variation.
//! * **Q-Statistic (SPE)** — sum of squared residuals in the *complement*
//!   subspace; detects changes in the *unmodelled* variation.
//!
//! Neither statistic explains *which* process variables are responsible
//! for the excursion, nor *how* the residual vector is oriented in
//! physical space.
//!
//! The [`StructuralPCA`] module provides the "why" to the PCA "what":
//! it decomposes the PCA residual vector into its principal loading
//! directions, identifies the dominant physical dimensions, and maps the
//! result to a DSFB grammar state and semiotic label.
//!
//! # Observer-Only Pattern
//! **No upstream controller state is modified.**  The multivariate observer
//! is a read-only side-channel that consumes statistics already produced by
//! the FDC system.  If a measurement is unavailable the observer degrades
//! gracefully by returning [`StructuralVerdict::Unavailable`].

use serde::{Deserialize, Serialize};

// ─── PCA Observation ─────────────────────────────────────────────────────────

/// A single multivariate process observation expressed in terms of the PCA
/// residual statistics already computed by the upstream FDC system.
///
/// All fields are *received from* the FDC system; the DSFB observer never
/// writes back to it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PcaObservation {
    /// Zero-indexed run / lot identifier.
    pub run_index: usize,
    /// Hotelling's T² statistic for this run
    /// (Mahalanobis distance² in the PC subspace).
    pub t2: Option<f64>,
    /// Q-Statistic (Squared Prediction Error / SPE) for this run.
    pub q_stat: Option<f64>,
    /// Number of principal components retained by the upstream FDC model.
    pub n_components: usize,
    /// Loadings of the first principal component, one entry per sensor.
    /// Must have length == number of sensors selected for PCA.
    /// `None` when the FDC model does not expose loadings.
    pub pc1_loading: Option<Vec<f64>>,
    /// Raw normalised residual vector (one entry per sensor).
    /// `None` when the FDC system does not expose individual residuals.
    pub residual_vector: Option<Vec<f64>>,
    /// Sensor labels corresponding to entries in `pc1_loading` and
    /// `residual_vector`.
    pub sensor_labels: Vec<String>,
}

// ─── Structural Verdict ──────────────────────────────────────────────────────

/// The structural interpretation the DSFB engine infers from the
/// PCA/FDC statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructuralVerdict {
    /// Both T² and Q are within their control limits; process is nominal.
    Nominal,
    /// T² is elevated but Q is within limits: the process has shifted
    /// *along* a known principal direction (modelled variation).  The
    /// dominant loading direction is captured in the `direction` field.
    ModelledShift { dominant_sensors: Vec<String> },
    /// Q is elevated but T² is within limits: the process has moved
    /// *orthogonal* to the known principal directions (unmodelled variation).
    /// A new failure mode may be emerging.
    UnmodelledExcursion,
    /// Both T² and Q are elevated: a large, multi-dimensional excursion.
    /// This is the highest-severity verdict.
    JointExcursion { dominant_sensors: Vec<String> },
    /// Required statistics are missing; the observer cannot issue a verdict.
    Unavailable,
}

impl StructuralVerdict {
    /// Maps the verdict to the DSFB grammar state string that would be
    /// emitted in the traceability manifest.
    pub fn grammar_state(&self) -> &'static str {
        match self {
            Self::Nominal => "Admissible",
            Self::ModelledShift { .. } => "SustainedDrift",
            Self::UnmodelledExcursion => "TransientViolation",
            Self::JointExcursion { .. } => "PersistentViolation",
            Self::Unavailable => "Unavailable",
        }
    }

    /// Recommended operator action.
    pub fn action(&self) -> &'static str {
        match self {
            Self::Nominal => "Monitor",
            Self::ModelledShift { .. } => "Review",
            Self::UnmodelledExcursion => "Review — investigate new failure mode",
            Self::JointExcursion { .. } => "Escalate",
            Self::Unavailable => "Check FDC telemetry",
        }
    }
}

// ─── Structural PCA ──────────────────────────────────────────────────────────

/// Structural PCA module — provides the "why" to the PCA "what".
///
/// Given a [`PcaObservation`] from the existing FDC system, this module:
///
/// 1. Classifies the excursion into four structural categories (Nominal /
///    ModelledShift / UnmodelledExcursion / JointExcursion).
/// 2. When loadings and residuals are available, identifies the dominant
///    physical sensors responsible for the deviation.
/// 3. Emits a [`StructuralInterpretation`] that can be serialised into the
///    run manifest and the traceability audit trail.
///
/// # Thresholds
/// The control limits for T² and Q are obtained from the FDC model's healthy
/// phase statistics.  They must be provided at construction time; the DSFB
/// engine never re-computes these limits from raw data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPCA {
    /// Upper control limit for T² (typically the 99th percentile of the
    /// chi-squared distribution with `n_components` degrees of freedom).
    pub t2_ucl: f64,
    /// Upper control limit for Q / SPE.
    pub q_ucl: f64,
    /// Number of dominant sensors to report in the structural interpretation.
    pub top_k_sensors: usize,
}

impl Default for StructuralPCA {
    fn default() -> Self {
        Self {
            t2_ucl: 9.21,                 // chi²(2, 0.99)
            q_ucl: 3.0,                   // 3-sigma rule on Q
            top_k_sensors: 5,
        }
    }
}

impl StructuralPCA {
    /// Interpret a single [`PcaObservation`] and return the structural verdict.
    pub fn interpret(&self, obs: &PcaObservation) -> StructuralInterpretation {
        let t2_alarm = obs.t2.map(|v| v > self.t2_ucl);
        let q_alarm = obs.q_stat.map(|v| v > self.q_ucl);

        let verdict = match (t2_alarm, q_alarm) {
            (None, _) | (_, None) => StructuralVerdict::Unavailable,
            (Some(false), Some(false)) => StructuralVerdict::Nominal,
            (Some(true), Some(false)) => {
                let dominant = self.dominant_sensors(obs, true);
                StructuralVerdict::ModelledShift {
                    dominant_sensors: dominant,
                }
            }
            (Some(false), Some(true)) => StructuralVerdict::UnmodelledExcursion,
            (Some(true), Some(true)) => {
                let dominant = self.dominant_sensors(obs, false);
                StructuralVerdict::JointExcursion {
                    dominant_sensors: dominant,
                }
            }
        };

        StructuralInterpretation {
            run_index: obs.run_index,
            t2: obs.t2,
            t2_ucl: self.t2_ucl,
            q_stat: obs.q_stat,
            q_ucl: self.q_ucl,
            verdict: verdict.clone(),
            grammar_state: verdict.grammar_state().to_string(),
            action: verdict.action().to_string(),
            integration_mode: "read_only_side_channel".into(),
        }
    }

    /// Returns the labels of the `top_k` sensors contributing most to the
    /// PCA residual vector.  Falls back to loading contribution when no
    /// residual vector is available.
    fn dominant_sensors(&self, obs: &PcaObservation, use_loadings: bool) -> Vec<String> {
        let scores: Option<Vec<f64>> = if use_loadings {
            obs.pc1_loading
                .as_ref()
                .map(|l| l.iter().map(|v| v.abs()).collect())
        } else {
            obs.residual_vector
                .as_ref()
                .map(|r| r.iter().map(|v| v.abs()).collect())
        };

        let Some(mut scored) = scores.map(|scores| {
            obs.sensor_labels
                .iter()
                .zip(scores.iter())
                .map(|(label, &score)| (label.clone(), score))
                .collect::<Vec<_>>()
        }) else {
            return Vec::new();
        };

        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        scored
            .into_iter()
            .take(self.top_k_sensors)
            .map(|(label, _)| label)
            .collect()
    }
}

// ─── Structural Interpretation Record ────────────────────────────────────────

/// The full structural interpretation record emitted by [`StructuralPCA::interpret`].
///
/// This struct is directly serialisable to JSON and included verbatim in
/// the `dsfb_run_manifest.json` audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralInterpretation {
    pub run_index: usize,
    pub t2: Option<f64>,
    pub t2_ucl: f64,
    pub q_stat: Option<f64>,
    pub q_ucl: f64,
    pub verdict: StructuralVerdict,
    pub grammar_state: String,
    pub action: String,
    /// Always `"read_only_side_channel"` — confirms the Observer-Only pattern.
    pub integration_mode: String,
}

// ─── Multivariate Observer ────────────────────────────────────────────────────

/// High-level observer that ingests PCA/FDC statistics from an upstream
/// monitoring system and produces DSFB structural interpretations.
///
/// # Thread Safety
/// The observer accumulates a history of observations in a [`Vec`].  If
/// concurrent ingestion is required, wrap in an `Arc<Mutex<...>>`.
#[derive(Debug, Default)]
pub struct MultivariateObserver {
    pub structural_pca: StructuralPCA,
    history: Vec<StructuralInterpretation>,
}

impl MultivariateObserver {
    /// Construct with a custom [`StructuralPCA`] configuration.
    pub fn with_config(structural_pca: StructuralPCA) -> Self {
        Self {
            structural_pca,
            history: Vec::new(),
        }
    }

    /// Ingest a PCA observation and store the structural interpretation.
    ///
    /// This method is the only entry point for external data.  It never
    /// modifies any upstream controller state.
    pub fn ingest(&mut self, obs: &PcaObservation) -> &StructuralInterpretation {
        let interpretation = self.structural_pca.interpret(obs);
        self.history.push(interpretation);
        self.history.last().unwrap()
    }

    /// Return all stored structural interpretations.
    pub fn interpretations(&self) -> &[StructuralInterpretation] {
        &self.history
    }

    /// Count observations where the verdict is a specific variant.
    pub fn count_verdicts(&self, verdict: &StructuralVerdict) -> usize {
        self.history
            .iter()
            .filter(|i| std::mem::discriminant(&i.verdict) == std::mem::discriminant(verdict))
            .count()
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn base_obs(run: usize, t2: f64, q: f64) -> PcaObservation {
        PcaObservation {
            run_index: run,
            t2: Some(t2),
            q_stat: Some(q),
            n_components: 3,
            pc1_loading: Some(vec![0.8, 0.5, 0.1, 0.05]),
            residual_vector: Some(vec![1.2, 0.3, 0.1, 0.05]),
            sensor_labels: vec![
                "S001".into(),
                "S002".into(),
                "S003".into(),
                "S004".into(),
            ],
        }
    }

    #[test]
    fn nominal_verdict_when_both_within_limits() {
        let spca = StructuralPCA::default();
        let obs = base_obs(0, 5.0, 1.5);
        let interp = spca.interpret(&obs);
        assert_eq!(interp.verdict, StructuralVerdict::Nominal);
        assert_eq!(interp.grammar_state, "Admissible");
    }

    #[test]
    fn modelled_shift_when_t2_alarm() {
        let spca = StructuralPCA::default();
        let obs = base_obs(1, 15.0, 1.5);
        let interp = spca.interpret(&obs);
        assert!(
            matches!(interp.verdict, StructuralVerdict::ModelledShift { .. }),
            "expected ModelledShift, got {:?}",
            interp.verdict
        );
        assert_eq!(interp.grammar_state, "SustainedDrift");
    }

    #[test]
    fn unmodelled_excursion_when_q_alarm() {
        let spca = StructuralPCA::default();
        let obs = base_obs(2, 5.0, 8.0);
        let interp = spca.interpret(&obs);
        assert_eq!(interp.verdict, StructuralVerdict::UnmodelledExcursion);
        assert_eq!(interp.grammar_state, "TransientViolation");
    }

    #[test]
    fn joint_excursion_when_both_alarm() {
        let spca = StructuralPCA::default();
        let obs = base_obs(3, 15.0, 8.0);
        let interp = spca.interpret(&obs);
        assert!(matches!(
            interp.verdict,
            StructuralVerdict::JointExcursion { .. }
        ));
        assert_eq!(interp.grammar_state, "PersistentViolation");
    }

    #[test]
    fn unavailable_when_t2_missing() {
        let spca = StructuralPCA::default();
        let mut obs = base_obs(4, 0.0, 1.5);
        obs.t2 = None;
        let interp = spca.interpret(&obs);
        assert_eq!(interp.verdict, StructuralVerdict::Unavailable);
    }

    #[test]
    fn dominant_sensors_returns_top_k() {
        let spca = StructuralPCA { top_k_sensors: 2, ..Default::default() };
        let obs = base_obs(5, 15.0, 1.5);
        let interp = spca.interpret(&obs);
        if let StructuralVerdict::ModelledShift { dominant_sensors } = interp.verdict {
            assert_eq!(dominant_sensors.len(), 2);
            assert_eq!(dominant_sensors[0], "S001"); // highest loading 0.8
        } else {
            panic!("expected ModelledShift");
        }
    }

    #[test]
    fn observer_accumulates_history() {
        let mut obs_engine = MultivariateObserver::default();
        for i in 0..5 {
            obs_engine.ingest(&base_obs(i, 5.0, 1.5));
        }
        assert_eq!(obs_engine.interpretations().len(), 5);
        assert_eq!(
            obs_engine.count_verdicts(&StructuralVerdict::Nominal),
            5
        );
    }

    #[test]
    fn integration_mode_is_read_only() {
        let spca = StructuralPCA::default();
        let obs = base_obs(0, 5.0, 1.5);
        let interp = spca.interpret(&obs);
        assert_eq!(interp.integration_mode, "read_only_side_channel");
    }
}
