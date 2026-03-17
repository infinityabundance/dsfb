use serde::Serialize;

use crate::config::ScenarioKind;
use crate::math::metrics::ScenarioSummary;

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRow {
    pub scenario: String,
    pub scenario_kind: String,
    pub agents: usize,
    pub noise_level: f64,
    pub visible_failure_step: Option<usize>,
    pub scalar_detection_step: Option<usize>,
    pub multimode_detection_step: Option<usize>,
    pub scalar_detection_lead_time: Option<f64>,
    pub multimode_detection_lead_time: Option<f64>,
    pub baseline_state_lead_time: Option<f64>,
    pub baseline_disagreement_lead_time: Option<f64>,
    pub baseline_lambda2_lead_time: Option<f64>,
    pub best_baseline_name: String,
    pub best_baseline_lead_time: Option<f64>,
    pub lead_time_gain_vs_best_baseline: Option<f64>,
    pub tpr_gain_vs_best_baseline: Option<f64>,
    pub fpr_reduction_vs_best_baseline: Option<f64>,
    pub multimode_minus_scalar_seconds: Option<f64>,
    pub scalar_true_positive_rate: f64,
    pub scalar_false_positive_rate: f64,
    pub multimode_true_positive_rate: f64,
    pub multimode_false_positive_rate: f64,
    pub trust_drop_step: Option<usize>,
    pub trust_suppression_delay: Option<f64>,
    pub peak_mode_shape_norm: f64,
    pub peak_stack_score: f64,
    pub residual_topology_correlation: f64,
    pub runtime_ms: f64,
}

impl From<&ScenarioSummary> for BenchmarkRow {
    fn from(value: &ScenarioSummary) -> Self {
        Self {
            scenario: value.scenario.clone(),
            scenario_kind: value.scenario_kind.clone(),
            agents: value.agents,
            noise_level: value.noise_level,
            visible_failure_step: value.visible_failure_step,
            scalar_detection_step: value.scalar_detection_step,
            multimode_detection_step: value.multimode_detection_step,
            scalar_detection_lead_time: value.scalar_detection_lead_time,
            multimode_detection_lead_time: value.multimode_detection_lead_time,
            baseline_state_lead_time: value.baseline_state_lead_time,
            baseline_disagreement_lead_time: value.baseline_disagreement_lead_time,
            baseline_lambda2_lead_time: value.baseline_lambda2_lead_time,
            best_baseline_name: value.best_baseline_name.clone(),
            best_baseline_lead_time: value.best_baseline_lead_time,
            lead_time_gain_vs_best_baseline: value.lead_time_gain_vs_best_baseline,
            tpr_gain_vs_best_baseline: value.tpr_gain_vs_best_baseline,
            fpr_reduction_vs_best_baseline: value.fpr_reduction_vs_best_baseline,
            multimode_minus_scalar_seconds: value.multimode_minus_scalar_seconds,
            scalar_true_positive_rate: value.scalar_true_positive_rate,
            scalar_false_positive_rate: value.scalar_false_positive_rate,
            multimode_true_positive_rate: value.multimode_true_positive_rate,
            multimode_false_positive_rate: value.multimode_false_positive_rate,
            trust_drop_step: value.trust_drop_step,
            trust_suppression_delay: value.trust_suppression_delay,
            peak_mode_shape_norm: value.peak_mode_shape_norm,
            peak_stack_score: value.peak_stack_score,
            residual_topology_correlation: value.residual_topology_correlation,
            runtime_ms: value.runtime_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunManifest {
    pub crate_name: &'static str,
    pub crate_version: &'static str,
    pub command: String,
    pub timestamp: String,
    pub scenario_kinds: Vec<String>,
    pub artifact_inventory: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeroBenchmarkRow {
    pub scenario: String,
    pub agents: usize,
    pub noise_level: f64,
    pub scalar_lead_time: Option<f64>,
    pub multimode_lead_time: Option<f64>,
    pub best_baseline_name: String,
    pub best_baseline_lead_time: Option<f64>,
    pub lead_time_gain_vs_best_baseline: Option<f64>,
    pub trust_suppression_delay: Option<f64>,
    pub scalar_true_positive_rate: f64,
    pub scalar_false_positive_rate: f64,
    pub multimode_true_positive_rate: f64,
    pub multimode_false_positive_rate: f64,
}

pub fn scenario_names(kinds: &[ScenarioKind]) -> Vec<String> {
    kinds.iter().map(|kind| kind.as_str().to_string()).collect()
}

pub fn select_hero_rows(rows: &[BenchmarkRow]) -> Vec<HeroBenchmarkRow> {
    let hero_scenarios = [
        "gradual_edge_degradation",
        "adversarial_agent",
        "communication_loss",
    ];
    hero_scenarios
        .iter()
        .filter_map(|scenario| {
            rows.iter()
                .filter(|row| row.scenario == *scenario)
                .max_by(|left, right| hero_rank(left).total_cmp(&hero_rank(right)))
                .map(HeroBenchmarkRow::from)
        })
        .collect()
}

impl From<&BenchmarkRow> for HeroBenchmarkRow {
    fn from(value: &BenchmarkRow) -> Self {
        Self {
            scenario: value.scenario.clone(),
            agents: value.agents,
            noise_level: value.noise_level,
            scalar_lead_time: value.scalar_detection_lead_time,
            multimode_lead_time: value.multimode_detection_lead_time,
            best_baseline_name: value.best_baseline_name.clone(),
            best_baseline_lead_time: value.best_baseline_lead_time,
            lead_time_gain_vs_best_baseline: value.lead_time_gain_vs_best_baseline,
            trust_suppression_delay: value.trust_suppression_delay,
            scalar_true_positive_rate: value.scalar_true_positive_rate,
            scalar_false_positive_rate: value.scalar_false_positive_rate,
            multimode_true_positive_rate: value.multimode_true_positive_rate,
            multimode_false_positive_rate: value.multimode_false_positive_rate,
        }
    }
}

fn hero_rank(row: &BenchmarkRow) -> f64 {
    let lead_gain = row.lead_time_gain_vs_best_baseline.unwrap_or(-1.0);
    let multimode_advantage = row.multimode_minus_scalar_seconds.unwrap_or(-0.5);
    let trust_delay = row.trust_suppression_delay.unwrap_or(0.0);
    let tpr = row.multimode_true_positive_rate.max(row.scalar_true_positive_rate);
    100.0 * lead_gain + 10.0 * multimode_advantage + 2.0 * trust_delay + tpr + row.agents as f64 * 1.0e-3
        - row.noise_level
}
