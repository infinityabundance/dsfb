use crate::baselines::BaselineSet;
use crate::grammar::{GrammarSet, GrammarState};
use crate::nominal::NominalModel;
use crate::preprocessing::{DatasetSummary, PreparedDataset};
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMetrics {
    pub feature_index: usize,
    pub feature_name: String,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub rho: f64,
    pub ewma_healthy_mean: f64,
    pub ewma_healthy_std: f64,
    pub ewma_threshold: f64,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
    pub missing_fraction: f64,
    pub ewma_alarm_points: usize,
    pub dsfb_boundary_points: usize,
    pub dsfb_violation_points: usize,
    pub threshold_alarm_points: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkSummary {
    pub dataset_summary: DatasetSummary,
    pub analyzable_feature_count: usize,
    pub threshold_alarm_points: usize,
    pub ewma_alarm_points: usize,
    pub dsfb_boundary_points: usize,
    pub dsfb_violation_points: usize,
    pub failure_runs: usize,
    pub failure_runs_with_preceding_dsfb_signal: usize,
    pub failure_runs_with_preceding_ewma_signal: usize,
    pub failure_runs_with_preceding_threshold_signal: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMetrics {
    pub summary: BenchmarkSummary,
    pub feature_metrics: Vec<FeatureMetrics>,
    pub top_feature_indices: Vec<usize>,
}

pub fn compute_metrics(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    pre_failure_lookback_runs: usize,
) -> BenchmarkMetrics {
    let mut feature_metrics = Vec::new();
    let mut threshold_alarm_points = 0usize;
    let mut ewma_alarm_points = 0usize;
    let mut dsfb_boundary_points = 0usize;
    let mut dsfb_violation_points = 0usize;

    for ((((feature, residual_trace), sign_trace), ewma_trace), grammar_trace) in nominal
        .features
        .iter()
        .zip(&residuals.traces)
        .zip(&signs.traces)
        .zip(&baselines.ewma)
        .zip(&grammar.traces)
    {
        let threshold_points = residual_trace
            .threshold_alarm
            .iter()
            .filter(|flag| **flag)
            .count();
        let ewma_points = ewma_trace.alarm.iter().filter(|flag| **flag).count();
        let boundary_points = grammar_trace
            .states
            .iter()
            .filter(|state| **state == GrammarState::Boundary)
            .count();
        let violation_points = grammar_trace
            .states
            .iter()
            .filter(|state| **state == GrammarState::Violation)
            .count();

        threshold_alarm_points += threshold_points;
        ewma_alarm_points += ewma_points;
        dsfb_boundary_points += boundary_points;
        dsfb_violation_points += violation_points;

        feature_metrics.push(FeatureMetrics {
            feature_index: feature.feature_index,
            feature_name: feature.feature_name.clone(),
            healthy_mean: feature.healthy_mean,
            healthy_std: feature.healthy_std,
            rho: feature.rho,
            ewma_healthy_mean: ewma_trace.healthy_mean,
            ewma_healthy_std: ewma_trace.healthy_std,
            ewma_threshold: ewma_trace.threshold,
            drift_threshold: sign_trace.drift_threshold,
            slew_threshold: sign_trace.slew_threshold,
            missing_fraction: dataset.per_feature_missing_fraction[feature.feature_index],
            ewma_alarm_points: ewma_points,
            dsfb_boundary_points: boundary_points,
            dsfb_violation_points: violation_points,
            threshold_alarm_points: threshold_points,
        });
    }

    let failure_indices = dataset
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| (*label == 1).then_some(index))
        .collect::<Vec<_>>();

    let mut failure_runs_with_preceding_dsfb_signal = 0usize;
    let mut failure_runs_with_preceding_ewma_signal = 0usize;
    let mut failure_runs_with_preceding_threshold_signal = 0usize;
    for &failure_index in &failure_indices {
        let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
        let dsfb_hit = grammar.traces.iter().any(|trace| {
            trace.states[window_start..failure_index]
                .iter()
                .any(|state| matches!(state, GrammarState::Boundary | GrammarState::Violation))
        });
        let ewma_hit = baselines.ewma.iter().any(|trace| {
            trace.alarm[window_start..failure_index]
                .iter()
                .any(|flag| *flag)
        });
        let threshold_hit = residuals.traces.iter().any(|trace| {
            trace.threshold_alarm[window_start..failure_index]
                .iter()
                .any(|flag| *flag)
        });
        if dsfb_hit {
            failure_runs_with_preceding_dsfb_signal += 1;
        }
        if ewma_hit {
            failure_runs_with_preceding_ewma_signal += 1;
        }
        if threshold_hit {
            failure_runs_with_preceding_threshold_signal += 1;
        }
    }

    let mut top_feature_indices = feature_metrics
        .iter()
        .filter(|feature| nominal.features[feature.feature_index].analyzable)
        .collect::<Vec<_>>();
    top_feature_indices.sort_by(|left, right| {
        right
            .dsfb_boundary_points
            .cmp(&left.dsfb_boundary_points)
            .then_with(|| {
                right
                    .threshold_alarm_points
                    .cmp(&left.threshold_alarm_points)
            })
            .then_with(|| left.feature_index.cmp(&right.feature_index))
    });
    let top_feature_indices = top_feature_indices
        .into_iter()
        .take(6)
        .map(|feature| feature.feature_index)
        .collect::<Vec<_>>();

    BenchmarkMetrics {
        summary: BenchmarkSummary {
            dataset_summary: dataset.summary.clone(),
            analyzable_feature_count: nominal
                .features
                .iter()
                .filter(|feature| feature.analyzable)
                .count(),
            threshold_alarm_points,
            ewma_alarm_points,
            dsfb_boundary_points,
            dsfb_violation_points,
            failure_runs: failure_indices.len(),
            failure_runs_with_preceding_dsfb_signal,
            failure_runs_with_preceding_ewma_signal,
            failure_runs_with_preceding_threshold_signal,
        },
        feature_metrics,
        top_feature_indices,
    }
}
