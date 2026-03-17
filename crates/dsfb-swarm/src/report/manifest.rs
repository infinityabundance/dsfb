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
    pub best_baseline_true_positive_rate: Option<f64>,
    pub best_baseline_false_positive_rate: Option<f64>,
    pub lead_time_gain_vs_best_baseline: Option<f64>,
    pub tpr_gain_vs_best_baseline: Option<f64>,
    pub fpr_delta_vs_best_baseline: Option<f64>,
    pub fpr_reduction_vs_best_baseline: Option<f64>,
    pub dsfb_advantage_margin: Option<f64>,
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
            best_baseline_true_positive_rate: value.best_baseline_true_positive_rate,
            best_baseline_false_positive_rate: value.best_baseline_false_positive_rate,
            lead_time_gain_vs_best_baseline: value.lead_time_gain_vs_best_baseline,
            tpr_gain_vs_best_baseline: value.tpr_gain_vs_best_baseline,
            fpr_delta_vs_best_baseline: value.fpr_delta_vs_best_baseline,
            fpr_reduction_vs_best_baseline: value.fpr_reduction_vs_best_baseline,
            dsfb_advantage_margin: value.dsfb_advantage_margin,
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
    pub dsfb_advantage_margin: Option<f64>,
    pub trust_suppression_delay: Option<f64>,
    pub scalar_true_positive_rate: f64,
    pub scalar_false_positive_rate: f64,
    pub multimode_true_positive_rate: f64,
    pub multimode_false_positive_rate: f64,
    pub winner: String,
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
                .max_by(|left, right| {
                    hero_rank(scenario, left).total_cmp(&hero_rank(scenario, right))
                })
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
            dsfb_advantage_margin: value.dsfb_advantage_margin,
            trust_suppression_delay: value.trust_suppression_delay,
            scalar_true_positive_rate: value.scalar_true_positive_rate,
            scalar_false_positive_rate: value.scalar_false_positive_rate,
            multimode_true_positive_rate: value.multimode_true_positive_rate,
            multimode_false_positive_rate: value.multimode_false_positive_rate,
            winner: winner_label(value),
        }
    }
}

fn hero_rank(scenario: &&str, row: &BenchmarkRow) -> f64 {
    let scalar_lead = row.scalar_detection_lead_time.unwrap_or(f64::NEG_INFINITY);
    let multimode_lead = row
        .multimode_detection_lead_time
        .unwrap_or(f64::NEG_INFINITY);
    let dsfb_best_lead = scalar_lead.max(multimode_lead);
    let baseline_lead = row.best_baseline_lead_time.unwrap_or(f64::NEG_INFINITY);
    let lead_gain = row.lead_time_gain_vs_best_baseline.unwrap_or(-2.0);
    let advantage_margin = row.dsfb_advantage_margin.unwrap_or(lead_gain);
    let multimode_advantage = row.multimode_minus_scalar_seconds.unwrap_or(0.0);
    let trust_delay = row.trust_suppression_delay.unwrap_or(0.0);
    let multimode_edge = row.multimode_true_positive_rate
        - row.scalar_true_positive_rate
        - 0.5 * (row.multimode_false_positive_rate - row.scalar_false_positive_rate).max(0.0);
    let best_fpr = row
        .scalar_false_positive_rate
        .min(row.multimode_false_positive_rate);
    let baseline_available = if row.best_baseline_lead_time.is_some() {
        1.0
    } else {
        0.0
    };
    match *scenario {
        "adversarial_agent" => {
            40.0 * multimode_advantage.max(0.0)
                + 25.0 * multimode_edge
                + 16.0 * trust_delay
                + 4.0 * baseline_available
                - 12.0 * best_fpr
                + row.agents as f64 * 1.0e-3
                - row.noise_level
        }
        _ => {
            52.0 * dsfb_best_lead.max(-1.0)
                + 130.0 * advantage_margin
                + 20.0 * multimode_advantage.max(0.0)
                + 4.0 * trust_delay
                + 6.0 * baseline_available
                + 4.0 * baseline_lead.max(-1.0)
                - 12.0 * best_fpr
                + row.agents as f64 * 1.0e-3
                - row.noise_level
        }
    }
}

fn winner_label(row: &BenchmarkRow) -> String {
    let scalar_lead = row.scalar_detection_lead_time.unwrap_or(f64::NEG_INFINITY);
    let multimode_lead = row
        .multimode_detection_lead_time
        .unwrap_or(f64::NEG_INFINITY);
    let baseline_lead = row.best_baseline_lead_time.unwrap_or(f64::NEG_INFINITY);
    let trust_delay = row.trust_suppression_delay.unwrap_or(0.0);
    let advantage_margin = row.dsfb_advantage_margin.unwrap_or_else(|| {
        row.lead_time_gain_vs_best_baseline
            .unwrap_or(f64::NEG_INFINITY)
    });
    let multimode_tpr_edge = row.multimode_true_positive_rate - row.scalar_true_positive_rate;
    let scalar_fpr = row.scalar_false_positive_rate;
    let multimode_fpr = row.multimode_false_positive_rate;

    if trust_delay > 0.0 && multimode_tpr_edge > 0.05 && multimode_fpr <= scalar_fpr + 0.01 {
        return "multimode + trust".to_string();
    }
    if baseline_lead > scalar_lead.max(multimode_lead) + 0.08 && advantage_margin < 0.0 {
        return format!("baseline ({})", row.best_baseline_name);
    }
    if multimode_lead > scalar_lead + 0.06
        && multimode_lead >= baseline_lead
        && multimode_fpr <= scalar_fpr + 0.01
    {
        return "multimode".to_string();
    }
    if scalar_lead >= multimode_lead - 0.08 && scalar_lead >= baseline_lead + 0.08 {
        return "scalar".to_string();
    }
    if scalar_lead.is_finite() || multimode_lead.is_finite() {
        return "dsfb tie".to_string();
    }
    if trust_delay > 0.0 {
        return "trust signal".to_string();
    }
    "no clear winner".to_string()
}
