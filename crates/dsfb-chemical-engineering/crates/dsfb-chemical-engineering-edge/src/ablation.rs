//! `AblationCourtV1` — deterministic component-ablation receipts (P63).
//!
//! "Which part of DSFB is doing the work?" is answered by ablation: disable one component (drift, slew,
//! envelope grazing, a detector family, quorum) and measure how the headline metric moves. This runs an
//! ablation arm per component against the full-pipeline baseline, records each arm's metric and its
//! **delta vs full**, and seals the whole court under a `court_hash`. An arm with a large delta is a
//! component the result depends on; a near-zero delta discloses a component that is (for that metric)
//! inert — both are honest, citable findings.
//!
//! Deterministic (the caller supplies pure metric evaluators); additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One ablation arm: the component disabled, the metric with it disabled, and the change vs full.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AblationArm {
    pub component: String,
    pub metric: f64,
    /// `metric − full_metric`: how much disabling this component moved the headline metric.
    pub delta_vs_full: f64,
}

/// A hash-sealed ablation court (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AblationCourtV1 {
    /// What the metric measures (e.g. `"baseline_fp_rate"`, `"n_episodes"`, `"detection_delay"`).
    pub metric_name: String,
    /// The metric with the full pipeline (all components enabled).
    pub full_metric: f64,
    pub arms: Vec<AblationArm>,
    /// The component whose ablation moved the metric the most (largest `|delta_vs_full|`), or `""`.
    pub most_load_bearing: String,
    pub court_hash: String,
}

impl AblationCourtV1 {
    fn seal(metric_name: &str, full: f64, arms: &[AblationArm], most: &str) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"ablation_court_v1");
        h.field("metric_name", metric_name.as_bytes());
        h.f64q("full_metric", full);
        for a in arms {
            h.field("component", a.component.as_bytes());
            h.f64q("metric", a.metric);
            h.f64q("delta_vs_full", a.delta_vs_full);
        }
        h.field("most_load_bearing", most.as_bytes());
        h.finalize_hex()
    }

    /// Run the ablation court: `full_eval()` gives the all-components baseline; each `(component,
    /// ablated_eval)` gives the metric with that component disabled. Each arm's `delta_vs_full` is
    /// recorded and the most load-bearing component (largest `|delta|`) identified. All evaluators must
    /// be pure + deterministic for the court to be reproducible.
    pub fn run<F, A>(metric_name: impl Into<String>, full_eval: F, arms: Vec<(String, A)>) -> Self
    where
        F: Fn() -> f64,
        A: Fn() -> f64,
    {
        let metric_name = metric_name.into();
        let full_metric = full_eval();
        let arms: Vec<AblationArm> = arms
            .into_iter()
            .map(|(component, eval)| {
                let metric = eval();
                AblationArm {
                    component,
                    metric,
                    delta_vs_full: metric - full_metric,
                }
            })
            .collect();
        let most_load_bearing = arms
            .iter()
            .max_by(|a, b| {
                a.delta_vs_full
                    .abs()
                    .partial_cmp(&b.delta_vs_full.abs())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|a| a.component.clone())
            .unwrap_or_default();
        let court_hash = Self::seal(&metric_name, full_metric, &arms, &most_load_bearing);
        AblationCourtV1 {
            metric_name,
            full_metric,
            arms,
            most_load_bearing,
            court_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.metric_name,
            self.full_metric,
            &self.arms,
            &self.most_load_bearing,
        ) == self.court_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ablation_records_deltas_and_finds_load_bearing_component() {
        // Full metric 10; ablating "drift" -> 4 (delta -6, big), "slew" -> 9 (delta -1, small).
        let court = AblationCourtV1::run(
            "n_episodes",
            || 10.0,
            vec![
                ("drift".to_string(), (|| 4.0) as fn() -> f64),
                ("slew".to_string(), (|| 9.0) as fn() -> f64),
            ],
        );
        assert!((court.full_metric - 10.0).abs() < 1e-12);
        assert_eq!(court.arms.len(), 2);
        assert!((court.arms[0].delta_vs_full - (-6.0)).abs() < 1e-12);
        assert_eq!(court.most_load_bearing, "drift");
        assert!(court.verify());
    }

    #[test]
    fn deterministic_and_tamper_evident() {
        let mk = || {
            AblationCourtV1::run(
                "m",
                || 1.0,
                vec![("c".to_string(), (|| 0.5) as fn() -> f64)],
            )
        };
        assert_eq!(mk().court_hash, mk().court_hash);
        let mut t = mk();
        t.arms[0].metric = 0.0;
        assert!(!t.verify());
    }
}
