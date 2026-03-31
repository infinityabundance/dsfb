use crate::baselines::BaselineSet;
use crate::grammar::{GrammarReason, GrammarSet, GrammarState};
use crate::nominal::NominalModel;
use crate::preprocessing::{DatasetSummary, PreparedDataset};
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::Serialize;
use std::collections::BTreeSet;

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
    pub failure_runs_with_preceding_dsfb_boundary_signal: usize,
    pub failure_runs_with_preceding_dsfb_violation_signal: usize,
    pub failure_runs_with_preceding_ewma_signal: usize,
    pub failure_runs_with_preceding_threshold_signal: usize,
    pub pass_runs: usize,
    pub pass_runs_with_dsfb_boundary_signal: usize,
    pub pass_runs_with_dsfb_violation_signal: usize,
    pub pass_runs_with_ewma_signal: usize,
    pub pass_runs_with_threshold_signal: usize,
    pub pass_run_dsfb_boundary_nuisance_rate: f64,
    pub pass_run_dsfb_violation_nuisance_rate: f64,
    pub pass_run_ewma_nuisance_rate: f64,
    pub pass_run_threshold_nuisance_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeadTimeSummary {
    pub failure_runs_with_boundary_lead: usize,
    pub failure_runs_with_violation_lead: usize,
    pub failure_runs_with_threshold_lead: usize,
    pub failure_runs_with_ewma_lead: usize,
    pub mean_boundary_lead_runs: Option<f64>,
    pub mean_violation_lead_runs: Option<f64>,
    pub mean_threshold_lead_runs: Option<f64>,
    pub mean_ewma_lead_runs: Option<f64>,
    pub mean_boundary_minus_threshold_delta_runs: Option<f64>,
    pub mean_boundary_minus_ewma_delta_runs: Option<f64>,
    pub mean_violation_minus_threshold_delta_runs: Option<f64>,
    pub mean_violation_minus_ewma_delta_runs: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoundaryEpisodeSummary {
    pub episode_count: usize,
    pub mean_episode_length: Option<f64>,
    pub max_episode_length: usize,
    pub non_escalating_episode_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MotifMetric {
    pub motif_name: String,
    pub point_hits: usize,
    pub run_hits: usize,
    pub pre_failure_window_run_hits: usize,
    pub pre_failure_window_precision_proxy: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerFailureRunSignal {
    pub failure_run_index: usize,
    pub failure_timestamp: String,
    pub earliest_dsfb_boundary_run: Option<usize>,
    pub earliest_dsfb_violation_run: Option<usize>,
    pub earliest_threshold_run: Option<usize>,
    pub earliest_ewma_run: Option<usize>,
    pub dsfb_boundary_lead_runs: Option<usize>,
    pub dsfb_violation_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub dsfb_boundary_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_boundary_minus_ewma_delta_runs: Option<i64>,
    pub dsfb_violation_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_violation_minus_ewma_delta_runs: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMetrics {
    pub summary: BenchmarkSummary,
    pub lead_time_summary: LeadTimeSummary,
    pub boundary_episode_summary: BoundaryEpisodeSummary,
    pub motif_metrics: Vec<MotifMetric>,
    pub per_failure_run_signals: Vec<PerFailureRunSignal>,
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
    let pass_indices = dataset
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| (*label == -1).then_some(index))
        .collect::<Vec<_>>();

    let failure_window_mask = failure_window_mask(
        dataset.labels.len(),
        &failure_indices,
        pre_failure_lookback_runs,
    );
    let motif_metrics = compute_motif_metrics(grammar, &failure_window_mask);
    let boundary_episode_summary = compute_boundary_episode_summary(grammar);
    let per_failure_run_signals = compute_per_failure_run_signals(
        dataset,
        residuals,
        baselines,
        grammar,
        pre_failure_lookback_runs,
        &failure_indices,
    );
    let lead_time_summary = summarize_lead_times(&per_failure_run_signals);

    let mut failure_runs_with_preceding_dsfb_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_boundary_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_violation_signal = 0usize;
    let mut failure_runs_with_preceding_ewma_signal = 0usize;
    let mut failure_runs_with_preceding_threshold_signal = 0usize;
    for record in &per_failure_run_signals {
        if record.earliest_dsfb_boundary_run.is_some()
            || record.earliest_dsfb_violation_run.is_some()
        {
            failure_runs_with_preceding_dsfb_signal += 1;
        }
        if record.earliest_dsfb_boundary_run.is_some() {
            failure_runs_with_preceding_dsfb_boundary_signal += 1;
        }
        if record.earliest_dsfb_violation_run.is_some() {
            failure_runs_with_preceding_dsfb_violation_signal += 1;
        }
        if record.earliest_ewma_run.is_some() {
            failure_runs_with_preceding_ewma_signal += 1;
        }
        if record.earliest_threshold_run.is_some() {
            failure_runs_with_preceding_threshold_signal += 1;
        }
    }

    let pass_runs_with_dsfb_boundary_signal = pass_indices
        .iter()
        .filter(|&&run_index| {
            grammar
                .traces
                .iter()
                .any(|trace| trace.states[run_index] == GrammarState::Boundary)
        })
        .count();
    let pass_runs_with_dsfb_violation_signal = pass_indices
        .iter()
        .filter(|&&run_index| {
            grammar
                .traces
                .iter()
                .any(|trace| trace.states[run_index] == GrammarState::Violation)
        })
        .count();
    let pass_runs_with_ewma_signal = pass_indices
        .iter()
        .filter(|&&run_index| baselines.ewma.iter().any(|trace| trace.alarm[run_index]))
        .count();
    let pass_runs_with_threshold_signal = pass_indices
        .iter()
        .filter(|&&run_index| {
            residuals
                .traces
                .iter()
                .any(|trace| trace.threshold_alarm[run_index])
        })
        .count();

    let mut top_feature_indices = feature_metrics
        .iter()
        .filter(|feature| nominal.features[feature.feature_index].analyzable)
        .collect::<Vec<_>>();
    top_feature_indices.sort_by(|left, right| {
        right
            .dsfb_boundary_points
            .cmp(&left.dsfb_boundary_points)
            .then_with(|| right.ewma_alarm_points.cmp(&left.ewma_alarm_points))
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

    let pass_runs = pass_indices.len();

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
            failure_runs_with_preceding_dsfb_boundary_signal,
            failure_runs_with_preceding_dsfb_violation_signal,
            failure_runs_with_preceding_ewma_signal,
            failure_runs_with_preceding_threshold_signal,
            pass_runs,
            pass_runs_with_dsfb_boundary_signal,
            pass_runs_with_dsfb_violation_signal,
            pass_runs_with_ewma_signal,
            pass_runs_with_threshold_signal,
            pass_run_dsfb_boundary_nuisance_rate: rate(
                pass_runs_with_dsfb_boundary_signal,
                pass_runs,
            ),
            pass_run_dsfb_violation_nuisance_rate: rate(
                pass_runs_with_dsfb_violation_signal,
                pass_runs,
            ),
            pass_run_ewma_nuisance_rate: rate(pass_runs_with_ewma_signal, pass_runs),
            pass_run_threshold_nuisance_rate: rate(pass_runs_with_threshold_signal, pass_runs),
        },
        lead_time_summary,
        boundary_episode_summary,
        motif_metrics,
        per_failure_run_signals,
        feature_metrics,
        top_feature_indices,
    }
}

fn failure_window_mask(
    run_count: usize,
    failure_indices: &[usize],
    pre_failure_lookback_runs: usize,
) -> Vec<bool> {
    let mut mask = vec![false; run_count];
    for &failure_index in failure_indices {
        let start = failure_index.saturating_sub(pre_failure_lookback_runs);
        for slot in &mut mask[start..failure_index] {
            *slot = true;
        }
    }
    mask
}

fn compute_per_failure_run_signals(
    dataset: &PreparedDataset,
    residuals: &ResidualSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    pre_failure_lookback_runs: usize,
    failure_indices: &[usize],
) -> Vec<PerFailureRunSignal> {
    failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            let earliest_dsfb_boundary_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    grammar
                        .traces
                        .iter()
                        .any(|trace| trace.states[run_index] == GrammarState::Boundary)
                });
            let earliest_dsfb_violation_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    grammar
                        .traces
                        .iter()
                        .any(|trace| trace.states[run_index] == GrammarState::Violation)
                });
            let earliest_threshold_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    residuals
                        .traces
                        .iter()
                        .any(|trace| trace.threshold_alarm[run_index])
                });
            let earliest_ewma_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    baselines.ewma.iter().any(|trace| trace.alarm[run_index])
                });

            let dsfb_boundary_lead_runs =
                earliest_dsfb_boundary_run.map(|index| failure_index - index);
            let dsfb_violation_lead_runs =
                earliest_dsfb_violation_run.map(|index| failure_index - index);
            let threshold_lead_runs = earliest_threshold_run.map(|index| failure_index - index);
            let ewma_lead_runs = earliest_ewma_run.map(|index| failure_index - index);

            PerFailureRunSignal {
                failure_run_index: failure_index,
                failure_timestamp: dataset.timestamps[failure_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                earliest_dsfb_boundary_run,
                earliest_dsfb_violation_run,
                earliest_threshold_run,
                earliest_ewma_run,
                dsfb_boundary_lead_runs,
                dsfb_violation_lead_runs,
                threshold_lead_runs,
                ewma_lead_runs,
                dsfb_boundary_minus_threshold_delta_runs: paired_delta(
                    dsfb_boundary_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_boundary_minus_ewma_delta_runs: paired_delta(
                    dsfb_boundary_lead_runs,
                    ewma_lead_runs,
                ),
                dsfb_violation_minus_threshold_delta_runs: paired_delta(
                    dsfb_violation_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_violation_minus_ewma_delta_runs: paired_delta(
                    dsfb_violation_lead_runs,
                    ewma_lead_runs,
                ),
            }
        })
        .collect()
}

fn earliest_signal_in_window<F>(start: usize, end: usize, predicate: F) -> Option<usize>
where
    F: Fn(usize) -> bool,
{
    (start..end).find(|&index| predicate(index))
}

fn paired_delta(left: Option<usize>, right: Option<usize>) -> Option<i64> {
    Some(left? as i64 - right? as i64)
}

fn summarize_lead_times(records: &[PerFailureRunSignal]) -> LeadTimeSummary {
    LeadTimeSummary {
        failure_runs_with_boundary_lead: records
            .iter()
            .filter(|record| record.dsfb_boundary_lead_runs.is_some())
            .count(),
        failure_runs_with_violation_lead: records
            .iter()
            .filter(|record| record.dsfb_violation_lead_runs.is_some())
            .count(),
        failure_runs_with_threshold_lead: records
            .iter()
            .filter(|record| record.threshold_lead_runs.is_some())
            .count(),
        failure_runs_with_ewma_lead: records
            .iter()
            .filter(|record| record.ewma_lead_runs.is_some())
            .count(),
        mean_boundary_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_boundary_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_violation_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_violation_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_threshold_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.threshold_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_ewma_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.ewma_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_boundary_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_boundary_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_boundary_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_boundary_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_violation_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_violation_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_violation_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_violation_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
    }
}

fn compute_boundary_episode_summary(grammar: &GrammarSet) -> BoundaryEpisodeSummary {
    let mut episode_count = 0usize;
    let mut total_length = 0usize;
    let mut max_length = 0usize;
    let mut non_escalating_episode_count = 0usize;

    for trace in &grammar.traces {
        let mut index = 0usize;
        while index < trace.states.len() {
            if trace.states[index] != GrammarState::Boundary {
                index += 1;
                continue;
            }

            let start = index;
            while index < trace.states.len() && trace.states[index] == GrammarState::Boundary {
                index += 1;
            }
            let length = index - start;
            episode_count += 1;
            total_length += length;
            max_length = max_length.max(length);
            let escalates =
                index < trace.states.len() && trace.states[index] == GrammarState::Violation;
            if !escalates {
                non_escalating_episode_count += 1;
            }
        }
    }

    BoundaryEpisodeSummary {
        episode_count,
        mean_episode_length: (episode_count > 0)
            .then_some(total_length as f64 / episode_count as f64),
        max_episode_length: max_length,
        non_escalating_episode_fraction: (episode_count > 0)
            .then_some(non_escalating_episode_count as f64 / episode_count as f64),
    }
}

fn compute_motif_metrics(grammar: &GrammarSet, failure_window_mask: &[bool]) -> Vec<MotifMetric> {
    let motif_specs = [
        (
            "pre_failure_slow_drift",
            GrammarReason::SustainedOutwardDrift,
        ),
        ("transient_excursion", GrammarReason::AbruptSlewViolation),
        (
            "recurrent_boundary_approach",
            GrammarReason::RecurrentBoundaryGrazing,
        ),
    ];

    motif_specs
        .iter()
        .map(|(motif_name, reason)| {
            let mut point_hits = 0usize;
            let mut run_hits = BTreeSet::new();
            let mut pre_failure_window_run_hits = BTreeSet::new();

            for trace in &grammar.traces {
                for (run_index, trace_reason) in trace.reasons.iter().enumerate() {
                    if trace_reason == reason {
                        point_hits += 1;
                        run_hits.insert(run_index);
                        if failure_window_mask[run_index] {
                            pre_failure_window_run_hits.insert(run_index);
                        }
                    }
                }
            }

            let run_hit_count = run_hits.len();
            let pre_failure_run_hit_count = pre_failure_window_run_hits.len();

            MotifMetric {
                motif_name: (*motif_name).into(),
                point_hits,
                run_hits: run_hit_count,
                pre_failure_window_run_hits: pre_failure_run_hit_count,
                pre_failure_window_precision_proxy: (run_hit_count > 0)
                    .then_some(pre_failure_run_hit_count as f64 / run_hit_count as f64),
            }
        })
        .collect()
}

fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn mean_option_usize(values: &[Option<usize>]) -> Option<f64> {
    let present = values.iter().flatten().copied().collect::<Vec<_>>();
    (!present.is_empty()).then_some(present.iter().sum::<usize>() as f64 / present.len() as f64)
}

fn mean_option_i64(values: &[Option<i64>]) -> Option<f64> {
    let present = values.iter().flatten().copied().collect::<Vec<_>>();
    (!present.is_empty()).then_some(present.iter().sum::<i64>() as f64 / present.len() as f64)
}
