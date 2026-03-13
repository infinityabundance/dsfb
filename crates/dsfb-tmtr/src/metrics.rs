use serde::Serialize;

use crate::causal::CausalMetricsSummary;
use crate::observer::ObserverSeries;
use crate::scenario::ScenarioDefinition;
use crate::tmtr::RecursionStats;

#[derive(Debug, Clone, Serialize)]
pub struct PredictionTubePoint {
    pub scenario: String,
    pub mode: String,
    pub anchor_time: usize,
    pub future_time: usize,
    pub horizon_step: usize,
    pub center: f64,
    pub lower: f64,
    pub upper: f64,
    pub width: f64,
    pub ground_truth: f64,
    pub contained: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummaryRow {
    pub scenario: String,
    pub title: String,
    pub description: String,
    pub n_steps: usize,
    pub degraded_start: usize,
    pub degraded_end: usize,
    pub refinement_end: usize,
    pub baseline_rmse: f64,
    pub tmtr_rmse: f64,
    pub retro_baseline_mae: f64,
    pub retro_tmtr_mae: f64,
    pub retro_error_reduction_pct: f64,
    pub baseline_recovery_time: usize,
    pub tmtr_recovery_time: usize,
    pub baseline_avg_tube_width: f64,
    pub tmtr_avg_tube_width: f64,
    pub baseline_tube_containment: f64,
    pub tmtr_tube_containment: f64,
    pub monotonicity_violations: usize,
    pub total_correction_events: usize,
    pub avg_correction_trust_weight: f64,
    pub max_recursion_depth: usize,
    pub mean_recursion_depth: f64,
    pub convergence_iterations: usize,
    pub baseline_edge_count: usize,
    pub tmtr_edge_count: usize,
    pub baseline_cycle_count: usize,
    pub tmtr_cycle_count: usize,
    pub tmtr_backward_edge_count: usize,
    pub tmtr_reachable_nodes: usize,
    pub tmtr_max_path_length: usize,
    pub tmtr_local_window_edge_density: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotebookReadySummary {
    pub output_root: String,
    pub primary_scenario: String,
    pub primary_title: String,
    pub baseline_reconstruction_error: f64,
    pub tmtr_reconstruction_error: f64,
    pub percent_improvement: f64,
    pub baseline_average_prediction_tube_width: f64,
    pub tmtr_average_prediction_tube_width: f64,
    pub max_recursion_depth: usize,
    pub monotonicity_violations: usize,
    pub cycle_count: usize,
}

pub fn build_prediction_tubes(
    scenario: &ScenarioDefinition,
    mode: &str,
    primary: &ObserverSeries,
    truth: &[f64],
) -> Vec<PredictionTubePoint> {
    let mut tubes = Vec::new();
    for anchor_time in
        (scenario.tube_eval_start..scenario.tube_eval_end).step_by(scenario.tube_stride.max(1))
    {
        if anchor_time + 2 >= truth.len() {
            break;
        }
        let slope = primary.estimate[anchor_time] - primary.estimate[anchor_time - 1];
        let previous_slope = if anchor_time > 1 {
            primary.estimate[anchor_time - 1] - primary.estimate[anchor_time - 2]
        } else {
            slope
        };
        let curvature = slope - previous_slope;
        let residual_seed =
            primary.residual[anchor_time].abs() + 0.45 * primary.envelope[anchor_time];
        let trust_factor = 1.05 - primary.trust[anchor_time];

        for horizon_step in 1..=scenario.prediction_horizon {
            let future_time = anchor_time + horizon_step;
            if future_time >= truth.len() {
                break;
            }
            let h = horizon_step as f64;
            let center = primary.estimate[anchor_time]
                + slope * h
                + 0.5 * curvature * h * h / scenario.prediction_horizon as f64;
            let predicted_residual =
                residual_seed * (-0.12 * h).exp() + 0.012 * h * (1.0 + trust_factor);
            let width = predicted_residual.max(0.015)
                * (1.0 + 0.45 * h / scenario.prediction_horizon as f64);
            let lower = center - width;
            let upper = center + width;
            let ground_truth = truth[future_time];
            tubes.push(PredictionTubePoint {
                scenario: scenario.name.clone(),
                mode: mode.to_string(),
                anchor_time,
                future_time,
                horizon_step,
                center,
                lower,
                upper,
                width: upper - lower,
                ground_truth,
                contained: (lower..=upper).contains(&ground_truth),
            });
        }
    }
    tubes
}

pub fn summarize_scenario(
    scenario: &ScenarioDefinition,
    baseline_primary: &ObserverSeries,
    tmtr_primary: &ObserverSeries,
    baseline_tubes: &[PredictionTubePoint],
    tmtr_tubes: &[PredictionTubePoint],
    baseline_causal: &CausalMetricsSummary,
    tmtr_causal: &CausalMetricsSummary,
    recursion: &RecursionStats,
) -> ScenarioSummaryRow {
    let baseline_rmse = rmse(&baseline_primary.residual);
    let tmtr_rmse = rmse(&tmtr_primary.residual);
    let retro_baseline_mae = mean_abs_in_interval(
        &baseline_primary.residual,
        scenario.degraded_start,
        scenario.degraded_end,
    );
    let retro_tmtr_mae = mean_abs_in_interval(
        &tmtr_primary.residual,
        scenario.degraded_start,
        scenario.degraded_end,
    );
    let retro_error_reduction_pct = if retro_baseline_mae <= f64::EPSILON {
        0.0
    } else {
        (retro_baseline_mae - retro_tmtr_mae) / retro_baseline_mae * 100.0
    };

    ScenarioSummaryRow {
        scenario: scenario.name.clone(),
        title: scenario.title.clone(),
        description: scenario.description.clone(),
        n_steps: scenario.n_steps,
        degraded_start: scenario.degraded_start,
        degraded_end: scenario.degraded_end,
        refinement_end: scenario.refinement_end,
        baseline_rmse,
        tmtr_rmse,
        retro_baseline_mae,
        retro_tmtr_mae,
        retro_error_reduction_pct,
        baseline_recovery_time: recovery_time(baseline_primary, scenario.degraded_end),
        tmtr_recovery_time: recovery_time(tmtr_primary, scenario.degraded_end),
        baseline_avg_tube_width: average_tube_width(baseline_tubes),
        tmtr_avg_tube_width: average_tube_width(tmtr_tubes),
        baseline_tube_containment: tube_containment(baseline_tubes),
        tmtr_tube_containment: tube_containment(tmtr_tubes),
        monotonicity_violations: recursion.monotonicity_violations,
        total_correction_events: recursion.total_correction_events,
        avg_correction_trust_weight: recursion.average_correction_trust_weight,
        max_recursion_depth: recursion.max_recursion_depth,
        mean_recursion_depth: recursion.mean_recursion_depth,
        convergence_iterations: recursion.convergence_iterations,
        baseline_edge_count: baseline_causal.edge_count,
        tmtr_edge_count: tmtr_causal.edge_count,
        baseline_cycle_count: baseline_causal.cycle_count,
        tmtr_cycle_count: tmtr_causal.cycle_count,
        tmtr_backward_edge_count: tmtr_causal.backward_edge_count,
        tmtr_reachable_nodes: tmtr_causal.reachable_nodes_from_anchor,
        tmtr_max_path_length: tmtr_causal.max_path_length,
        tmtr_local_window_edge_density: tmtr_causal.local_window_edge_density,
    }
}

pub fn notebook_ready_summary(
    output_root: &str,
    summaries: &[ScenarioSummaryRow],
) -> NotebookReadySummary {
    let primary = summaries
        .iter()
        .find(|summary| summary.scenario == "disturbance_recovery")
        .or_else(|| summaries.first())
        .expect("at least one scenario summary");
    let percent_improvement = if primary.baseline_rmse <= f64::EPSILON {
        0.0
    } else {
        (primary.baseline_rmse - primary.tmtr_rmse) / primary.baseline_rmse * 100.0
    };
    NotebookReadySummary {
        output_root: output_root.to_string(),
        primary_scenario: primary.scenario.clone(),
        primary_title: primary.title.clone(),
        baseline_reconstruction_error: primary.baseline_rmse,
        tmtr_reconstruction_error: primary.tmtr_rmse,
        percent_improvement,
        baseline_average_prediction_tube_width: primary.baseline_avg_tube_width,
        tmtr_average_prediction_tube_width: primary.tmtr_avg_tube_width,
        max_recursion_depth: primary.max_recursion_depth,
        monotonicity_violations: primary.monotonicity_violations,
        cycle_count: primary.tmtr_cycle_count,
    }
}

fn rmse(residuals: &[f64]) -> f64 {
    if residuals.is_empty() {
        return 0.0;
    }
    (residuals.iter().map(|value| value * value).sum::<f64>() / residuals.len() as f64).sqrt()
}

fn mean_abs_in_interval(values: &[f64], start: usize, end: usize) -> f64 {
    let slice = &values[start.min(values.len())..=end.min(values.len().saturating_sub(1))];
    if slice.is_empty() {
        return 0.0;
    }
    slice.iter().map(|value| value.abs()).sum::<f64>() / slice.len() as f64
}

fn recovery_time(primary: &ObserverSeries, degraded_end: usize) -> usize {
    let threshold = 0.06;
    let window = 6usize;
    if primary.residual.is_empty() {
        return 0;
    }
    let max_start = primary.residual.len().saturating_sub(window + 1);
    for start in degraded_end.min(max_start)..=max_start {
        let stable = primary.residual[start..start + window]
            .iter()
            .all(|residual| residual.abs() <= threshold);
        if stable {
            return start.saturating_sub(degraded_end);
        }
    }
    primary.residual.len().saturating_sub(degraded_end)
}

fn average_tube_width(tubes: &[PredictionTubePoint]) -> f64 {
    if tubes.is_empty() {
        return 0.0;
    }
    tubes.iter().map(|tube| tube.width).sum::<f64>() / tubes.len() as f64
}

fn tube_containment(tubes: &[PredictionTubePoint]) -> f64 {
    if tubes.is_empty() {
        return 0.0;
    }
    let contained = tubes.iter().filter(|tube| tube.contained).count();
    contained as f64 / tubes.len() as f64
}
