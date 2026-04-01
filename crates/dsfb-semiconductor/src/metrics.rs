use crate::baselines::BaselineSet;
use crate::config::PipelineConfig;
use crate::grammar::{GrammarReason, GrammarSet, GrammarState};
use crate::nominal::NominalModel;
use crate::precursor::DsaSignalSummary;
use crate::preprocessing::{DatasetSummary, PreparedDataset};
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMetrics {
    pub feature_index: usize,
    pub feature_name: String,
    pub analyzable: bool,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub rho: f64,
    pub ewma_healthy_mean: f64,
    pub ewma_healthy_std: f64,
    pub ewma_threshold: f64,
    pub cusum_healthy_mean: f64,
    pub cusum_healthy_std: f64,
    pub cusum_kappa: f64,
    pub cusum_alarm_threshold: f64,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
    pub missing_fraction: f64,
    pub ewma_alarm_points: usize,
    pub cusum_alarm_points: usize,
    pub dsfb_raw_boundary_points: usize,
    pub dsfb_persistent_boundary_points: usize,
    pub dsfb_raw_violation_points: usize,
    pub dsfb_persistent_violation_points: usize,
    pub threshold_alarm_points: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkSummary {
    pub dataset_summary: DatasetSummary,
    pub analyzable_feature_count: usize,
    pub threshold_alarm_points: usize,
    pub ewma_alarm_points: usize,
    pub cusum_alarm_points: usize,
    pub run_energy_alarm_points: usize,
    pub pca_fdc_alarm_points: usize,
    pub dsfb_raw_boundary_points: usize,
    pub dsfb_persistent_boundary_points: usize,
    pub dsfb_raw_violation_points: usize,
    pub dsfb_persistent_violation_points: usize,
    pub failure_runs: usize,
    pub failure_runs_with_preceding_dsfb_raw_signal: usize,
    pub failure_runs_with_preceding_dsfb_persistent_signal: usize,
    pub failure_runs_with_preceding_dsfb_raw_boundary_signal: usize,
    pub failure_runs_with_preceding_dsfb_persistent_boundary_signal: usize,
    pub failure_runs_with_preceding_dsfb_raw_violation_signal: usize,
    pub failure_runs_with_preceding_dsfb_persistent_violation_signal: usize,
    pub failure_runs_with_preceding_ewma_signal: usize,
    pub failure_runs_with_preceding_cusum_signal: usize,
    pub failure_runs_with_preceding_run_energy_signal: usize,
    pub failure_runs_with_preceding_pca_fdc_signal: usize,
    pub failure_runs_with_preceding_threshold_signal: usize,
    pub pass_runs: usize,
    pub pass_runs_with_dsfb_raw_boundary_signal: usize,
    pub pass_runs_with_dsfb_persistent_boundary_signal: usize,
    pub pass_runs_with_dsfb_raw_violation_signal: usize,
    pub pass_runs_with_dsfb_persistent_violation_signal: usize,
    pub pass_runs_with_ewma_signal: usize,
    pub pass_runs_with_cusum_signal: usize,
    pub pass_runs_with_run_energy_signal: usize,
    pub pass_runs_with_pca_fdc_signal: usize,
    pub pass_runs_with_threshold_signal: usize,
    pub pass_run_dsfb_raw_boundary_nuisance_rate: f64,
    pub pass_run_dsfb_persistent_boundary_nuisance_rate: f64,
    pub pass_run_dsfb_raw_violation_nuisance_rate: f64,
    pub pass_run_dsfb_persistent_violation_nuisance_rate: f64,
    pub pass_run_ewma_nuisance_rate: f64,
    pub pass_run_cusum_nuisance_rate: f64,
    pub pass_run_run_energy_nuisance_rate: f64,
    pub pass_run_pca_fdc_nuisance_rate: f64,
    pub pass_run_threshold_nuisance_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeadTimeSummary {
    pub failure_runs_with_raw_boundary_lead: usize,
    pub failure_runs_with_persistent_boundary_lead: usize,
    pub failure_runs_with_raw_violation_lead: usize,
    pub failure_runs_with_persistent_violation_lead: usize,
    pub failure_runs_with_threshold_lead: usize,
    pub failure_runs_with_ewma_lead: usize,
    pub failure_runs_with_cusum_lead: usize,
    pub failure_runs_with_run_energy_lead: usize,
    pub failure_runs_with_pca_fdc_lead: usize,
    pub mean_raw_boundary_lead_runs: Option<f64>,
    pub mean_persistent_boundary_lead_runs: Option<f64>,
    pub mean_raw_violation_lead_runs: Option<f64>,
    pub mean_persistent_violation_lead_runs: Option<f64>,
    pub mean_threshold_lead_runs: Option<f64>,
    pub mean_ewma_lead_runs: Option<f64>,
    pub mean_cusum_lead_runs: Option<f64>,
    pub mean_run_energy_lead_runs: Option<f64>,
    pub mean_pca_fdc_lead_runs: Option<f64>,
    pub mean_raw_boundary_minus_cusum_delta_runs: Option<f64>,
    pub mean_raw_boundary_minus_run_energy_delta_runs: Option<f64>,
    pub mean_raw_boundary_minus_pca_fdc_delta_runs: Option<f64>,
    pub mean_raw_boundary_minus_threshold_delta_runs: Option<f64>,
    pub mean_raw_boundary_minus_ewma_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_cusum_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_run_energy_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_pca_fdc_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_threshold_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_ewma_delta_runs: Option<f64>,
    pub mean_raw_violation_minus_cusum_delta_runs: Option<f64>,
    pub mean_raw_violation_minus_run_energy_delta_runs: Option<f64>,
    pub mean_raw_violation_minus_pca_fdc_delta_runs: Option<f64>,
    pub mean_raw_violation_minus_threshold_delta_runs: Option<f64>,
    pub mean_raw_violation_minus_ewma_delta_runs: Option<f64>,
    pub mean_persistent_violation_minus_cusum_delta_runs: Option<f64>,
    pub mean_persistent_violation_minus_run_energy_delta_runs: Option<f64>,
    pub mean_persistent_violation_minus_pca_fdc_delta_runs: Option<f64>,
    pub mean_persistent_violation_minus_threshold_delta_runs: Option<f64>,
    pub mean_persistent_violation_minus_ewma_delta_runs: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoundaryEpisodeSummary {
    pub raw_episode_count: usize,
    pub persistent_episode_count: usize,
    pub mean_raw_episode_length: Option<f64>,
    pub mean_persistent_episode_length: Option<f64>,
    pub max_raw_episode_length: usize,
    pub max_persistent_episode_length: usize,
    pub raw_non_escalating_episode_fraction: Option<f64>,
    pub persistent_non_escalating_episode_fraction: Option<f64>,
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
    pub earliest_dsfb_raw_boundary_run: Option<usize>,
    pub earliest_dsfb_persistent_boundary_run: Option<usize>,
    pub earliest_dsfb_raw_violation_run: Option<usize>,
    pub earliest_dsfb_persistent_violation_run: Option<usize>,
    pub earliest_threshold_run: Option<usize>,
    pub earliest_ewma_run: Option<usize>,
    pub earliest_cusum_run: Option<usize>,
    pub earliest_run_energy_run: Option<usize>,
    pub earliest_pca_fdc_run: Option<usize>,
    pub dsfb_raw_boundary_lead_runs: Option<usize>,
    pub dsfb_persistent_boundary_lead_runs: Option<usize>,
    pub dsfb_raw_violation_lead_runs: Option<usize>,
    pub dsfb_persistent_violation_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub cusum_lead_runs: Option<usize>,
    pub run_energy_lead_runs: Option<usize>,
    pub pca_fdc_lead_runs: Option<usize>,
    pub dsfb_raw_boundary_minus_cusum_delta_runs: Option<i64>,
    pub dsfb_raw_boundary_minus_run_energy_delta_runs: Option<i64>,
    pub dsfb_raw_boundary_minus_pca_fdc_delta_runs: Option<i64>,
    pub dsfb_raw_boundary_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_raw_boundary_minus_ewma_delta_runs: Option<i64>,
    pub dsfb_persistent_boundary_minus_cusum_delta_runs: Option<i64>,
    pub dsfb_persistent_boundary_minus_run_energy_delta_runs: Option<i64>,
    pub dsfb_persistent_boundary_minus_pca_fdc_delta_runs: Option<i64>,
    pub dsfb_persistent_boundary_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_persistent_boundary_minus_ewma_delta_runs: Option<i64>,
    pub dsfb_raw_violation_minus_cusum_delta_runs: Option<i64>,
    pub dsfb_raw_violation_minus_run_energy_delta_runs: Option<i64>,
    pub dsfb_raw_violation_minus_pca_fdc_delta_runs: Option<i64>,
    pub dsfb_raw_violation_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_raw_violation_minus_ewma_delta_runs: Option<i64>,
    pub dsfb_persistent_violation_minus_cusum_delta_runs: Option<i64>,
    pub dsfb_persistent_violation_minus_run_energy_delta_runs: Option<i64>,
    pub dsfb_persistent_violation_minus_pca_fdc_delta_runs: Option<i64>,
    pub dsfb_persistent_violation_minus_threshold_delta_runs: Option<i64>,
    pub dsfb_persistent_violation_minus_ewma_delta_runs: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DensityMetricRecord {
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub in_pre_failure_window: bool,
    pub raw_boundary_density: f64,
    pub persistent_boundary_density: f64,
    pub raw_violation_density: f64,
    pub persistent_violation_density: f64,
    pub threshold_density: f64,
    pub ewma_density: f64,
    pub cusum_density: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DensitySummary {
    pub density_window: usize,
    pub mean_raw_boundary_density_failure: f64,
    pub mean_raw_boundary_density_pass: f64,
    pub mean_persistent_boundary_density_failure: f64,
    pub mean_persistent_boundary_density_pass: f64,
    pub mean_raw_violation_density_failure: f64,
    pub mean_raw_violation_density_pass: f64,
    pub mean_persistent_violation_density_failure: f64,
    pub mean_persistent_violation_density_pass: f64,
    pub mean_threshold_density_failure: f64,
    pub mean_threshold_density_pass: f64,
    pub mean_ewma_density_failure: f64,
    pub mean_ewma_density_pass: f64,
    pub mean_cusum_density_failure: f64,
    pub mean_cusum_density_pass: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMetrics {
    pub summary: BenchmarkSummary,
    pub lead_time_summary: LeadTimeSummary,
    pub density_summary: DensitySummary,
    pub boundary_episode_summary: BoundaryEpisodeSummary,
    pub dsa_summary: Option<DsaSignalSummary>,
    pub motif_metrics: Vec<MotifMetric>,
    pub per_failure_run_signals: Vec<PerFailureRunSignal>,
    pub density_metrics: Vec<DensityMetricRecord>,
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
    config: &PipelineConfig,
) -> BenchmarkMetrics {
    let mut feature_metrics = Vec::new();
    let mut threshold_alarm_points = 0usize;
    let mut ewma_alarm_points = 0usize;
    let mut cusum_alarm_points = 0usize;
    let run_energy_alarm_points = baselines
        .run_energy
        .alarm
        .iter()
        .filter(|flag| **flag)
        .count();
    let pca_fdc_alarm_points = baselines.pca_fdc.alarm.iter().filter(|flag| **flag).count();
    let mut dsfb_raw_boundary_points = 0usize;
    let mut dsfb_persistent_boundary_points = 0usize;
    let mut dsfb_raw_violation_points = 0usize;
    let mut dsfb_persistent_violation_points = 0usize;

    for (((((feature, residual_trace), sign_trace), ewma_trace), cusum_trace), grammar_trace) in
        nominal
            .features
            .iter()
            .zip(&residuals.traces)
            .zip(&signs.traces)
            .zip(&baselines.ewma)
            .zip(&baselines.cusum)
            .zip(&grammar.traces)
    {
        let threshold_points = residual_trace
            .threshold_alarm
            .iter()
            .filter(|flag| **flag)
            .count();
        let ewma_points = ewma_trace.alarm.iter().filter(|flag| **flag).count();
        let cusum_points = cusum_trace.alarm.iter().filter(|flag| **flag).count();
        let raw_boundary_points = grammar_trace
            .raw_states
            .iter()
            .filter(|state| **state == GrammarState::Boundary)
            .count();
        let persistent_boundary_points = grammar_trace
            .persistent_boundary
            .iter()
            .filter(|flag| **flag)
            .count();
        let raw_violation_points = grammar_trace
            .raw_states
            .iter()
            .filter(|state| **state == GrammarState::Violation)
            .count();
        let persistent_violation_points = grammar_trace
            .persistent_violation
            .iter()
            .filter(|flag| **flag)
            .count();

        threshold_alarm_points += threshold_points;
        ewma_alarm_points += ewma_points;
        cusum_alarm_points += cusum_points;
        dsfb_raw_boundary_points += raw_boundary_points;
        dsfb_persistent_boundary_points += persistent_boundary_points;
        dsfb_raw_violation_points += raw_violation_points;
        dsfb_persistent_violation_points += persistent_violation_points;

        feature_metrics.push(FeatureMetrics {
            feature_index: feature.feature_index,
            feature_name: feature.feature_name.clone(),
            analyzable: feature.analyzable,
            healthy_mean: feature.healthy_mean,
            healthy_std: feature.healthy_std,
            rho: feature.rho,
            ewma_healthy_mean: ewma_trace.healthy_mean,
            ewma_healthy_std: ewma_trace.healthy_std,
            ewma_threshold: ewma_trace.threshold,
            cusum_healthy_mean: cusum_trace.healthy_mean,
            cusum_healthy_std: cusum_trace.healthy_std,
            cusum_kappa: cusum_trace.kappa,
            cusum_alarm_threshold: cusum_trace.alarm_threshold,
            drift_threshold: sign_trace.drift_threshold,
            slew_threshold: sign_trace.slew_threshold,
            missing_fraction: dataset.per_feature_missing_fraction[feature.feature_index],
            ewma_alarm_points: ewma_points,
            cusum_alarm_points: cusum_points,
            dsfb_raw_boundary_points: raw_boundary_points,
            dsfb_persistent_boundary_points: persistent_boundary_points,
            dsfb_raw_violation_points: raw_violation_points,
            dsfb_persistent_violation_points: persistent_violation_points,
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
        config.pre_failure_lookback_runs,
    );
    let motif_metrics = compute_motif_metrics(grammar, &failure_window_mask);
    let boundary_episode_summary = compute_boundary_episode_summary(grammar);
    let per_failure_run_signals = compute_per_failure_run_signals(
        dataset,
        residuals,
        baselines,
        grammar,
        config.pre_failure_lookback_runs,
        &failure_indices,
    );
    let lead_time_summary = summarize_lead_times(&per_failure_run_signals);
    let density_metrics = compute_density_metrics(
        dataset,
        nominal,
        residuals,
        baselines,
        grammar,
        config.density_window,
        &failure_window_mask,
    );
    let density_summary = summarize_densities(&density_metrics, config.density_window);

    let mut failure_runs_with_preceding_dsfb_raw_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_persistent_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_raw_boundary_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_persistent_boundary_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_raw_violation_signal = 0usize;
    let mut failure_runs_with_preceding_dsfb_persistent_violation_signal = 0usize;
    let mut failure_runs_with_preceding_ewma_signal = 0usize;
    let mut failure_runs_with_preceding_cusum_signal = 0usize;
    let mut failure_runs_with_preceding_run_energy_signal = 0usize;
    let mut failure_runs_with_preceding_pca_fdc_signal = 0usize;
    let mut failure_runs_with_preceding_threshold_signal = 0usize;
    for record in &per_failure_run_signals {
        if record.earliest_dsfb_raw_boundary_run.is_some()
            || record.earliest_dsfb_raw_violation_run.is_some()
        {
            failure_runs_with_preceding_dsfb_raw_signal += 1;
        }
        if record.earliest_dsfb_persistent_boundary_run.is_some()
            || record.earliest_dsfb_persistent_violation_run.is_some()
        {
            failure_runs_with_preceding_dsfb_persistent_signal += 1;
        }
        if record.earliest_dsfb_raw_boundary_run.is_some() {
            failure_runs_with_preceding_dsfb_raw_boundary_signal += 1;
        }
        if record.earliest_dsfb_persistent_boundary_run.is_some() {
            failure_runs_with_preceding_dsfb_persistent_boundary_signal += 1;
        }
        if record.earliest_dsfb_raw_violation_run.is_some() {
            failure_runs_with_preceding_dsfb_raw_violation_signal += 1;
        }
        if record.earliest_dsfb_persistent_violation_run.is_some() {
            failure_runs_with_preceding_dsfb_persistent_violation_signal += 1;
        }
        if record.earliest_ewma_run.is_some() {
            failure_runs_with_preceding_ewma_signal += 1;
        }
        if record.earliest_cusum_run.is_some() {
            failure_runs_with_preceding_cusum_signal += 1;
        }
        if record.earliest_run_energy_run.is_some() {
            failure_runs_with_preceding_run_energy_signal += 1;
        }
        if record.earliest_pca_fdc_run.is_some() {
            failure_runs_with_preceding_pca_fdc_signal += 1;
        }
        if record.earliest_threshold_run.is_some() {
            failure_runs_with_preceding_threshold_signal += 1;
        }
    }

    let pass_runs_with_dsfb_raw_boundary_signal =
        count_runs_with_signal(&pass_indices, |run_index| {
            any_trace_raw_state(grammar, run_index, GrammarState::Boundary)
        });
    let pass_runs_with_dsfb_persistent_boundary_signal =
        count_runs_with_signal(&pass_indices, |run_index| {
            any_trace_persistent(grammar, run_index, GrammarState::Boundary)
        });
    let pass_runs_with_dsfb_raw_violation_signal =
        count_runs_with_signal(&pass_indices, |run_index| {
            any_trace_raw_state(grammar, run_index, GrammarState::Violation)
        });
    let pass_runs_with_dsfb_persistent_violation_signal =
        count_runs_with_signal(&pass_indices, |run_index| {
            any_trace_persistent(grammar, run_index, GrammarState::Violation)
        });
    let pass_runs_with_ewma_signal = count_runs_with_signal(&pass_indices, |run_index| {
        baselines.ewma.iter().any(|trace| trace.alarm[run_index])
    });
    let pass_runs_with_cusum_signal = count_runs_with_signal(&pass_indices, |run_index| {
        baselines.cusum.iter().any(|trace| trace.alarm[run_index])
    });
    let pass_runs_with_run_energy_signal = count_runs_with_signal(&pass_indices, |run_index| {
        baselines.run_energy.alarm[run_index]
    });
    let pass_runs_with_pca_fdc_signal = count_runs_with_signal(&pass_indices, |run_index| {
        baselines.pca_fdc.alarm[run_index]
    });
    let pass_runs_with_threshold_signal = count_runs_with_signal(&pass_indices, |run_index| {
        residuals
            .traces
            .iter()
            .any(|trace| trace.threshold_alarm[run_index])
    });

    let mut top_feature_indices = feature_metrics
        .iter()
        .filter(|feature| nominal.features[feature.feature_index].analyzable)
        .collect::<Vec<_>>();
    top_feature_indices.sort_by(|left, right| {
        right
            .dsfb_persistent_boundary_points
            .cmp(&left.dsfb_persistent_boundary_points)
            .then_with(|| {
                right
                    .dsfb_raw_boundary_points
                    .cmp(&left.dsfb_raw_boundary_points)
            })
            .then_with(|| right.ewma_alarm_points.cmp(&left.ewma_alarm_points))
            .then_with(|| right.cusum_alarm_points.cmp(&left.cusum_alarm_points))
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
            cusum_alarm_points,
            run_energy_alarm_points,
            pca_fdc_alarm_points,
            dsfb_raw_boundary_points,
            dsfb_persistent_boundary_points,
            dsfb_raw_violation_points,
            dsfb_persistent_violation_points,
            failure_runs: failure_indices.len(),
            failure_runs_with_preceding_dsfb_raw_signal,
            failure_runs_with_preceding_dsfb_persistent_signal,
            failure_runs_with_preceding_dsfb_raw_boundary_signal,
            failure_runs_with_preceding_dsfb_persistent_boundary_signal,
            failure_runs_with_preceding_dsfb_raw_violation_signal,
            failure_runs_with_preceding_dsfb_persistent_violation_signal,
            failure_runs_with_preceding_ewma_signal,
            failure_runs_with_preceding_cusum_signal,
            failure_runs_with_preceding_run_energy_signal,
            failure_runs_with_preceding_pca_fdc_signal,
            failure_runs_with_preceding_threshold_signal,
            pass_runs,
            pass_runs_with_dsfb_raw_boundary_signal,
            pass_runs_with_dsfb_persistent_boundary_signal,
            pass_runs_with_dsfb_raw_violation_signal,
            pass_runs_with_dsfb_persistent_violation_signal,
            pass_runs_with_ewma_signal,
            pass_runs_with_cusum_signal,
            pass_runs_with_run_energy_signal,
            pass_runs_with_pca_fdc_signal,
            pass_runs_with_threshold_signal,
            pass_run_dsfb_raw_boundary_nuisance_rate: rate(
                pass_runs_with_dsfb_raw_boundary_signal,
                pass_runs,
            ),
            pass_run_dsfb_persistent_boundary_nuisance_rate: rate(
                pass_runs_with_dsfb_persistent_boundary_signal,
                pass_runs,
            ),
            pass_run_dsfb_raw_violation_nuisance_rate: rate(
                pass_runs_with_dsfb_raw_violation_signal,
                pass_runs,
            ),
            pass_run_dsfb_persistent_violation_nuisance_rate: rate(
                pass_runs_with_dsfb_persistent_violation_signal,
                pass_runs,
            ),
            pass_run_ewma_nuisance_rate: rate(pass_runs_with_ewma_signal, pass_runs),
            pass_run_cusum_nuisance_rate: rate(pass_runs_with_cusum_signal, pass_runs),
            pass_run_run_energy_nuisance_rate: rate(pass_runs_with_run_energy_signal, pass_runs),
            pass_run_pca_fdc_nuisance_rate: rate(pass_runs_with_pca_fdc_signal, pass_runs),
            pass_run_threshold_nuisance_rate: rate(pass_runs_with_threshold_signal, pass_runs),
        },
        lead_time_summary,
        density_summary,
        boundary_episode_summary,
        dsa_summary: None,
        motif_metrics,
        per_failure_run_signals,
        density_metrics,
        feature_metrics,
        top_feature_indices,
    }
}

fn count_runs_with_signal<F>(run_indices: &[usize], predicate: F) -> usize
where
    F: Fn(usize) -> bool,
{
    run_indices
        .iter()
        .filter(|&&run_index| predicate(run_index))
        .count()
}

fn any_trace_raw_state(grammar: &GrammarSet, run_index: usize, target: GrammarState) -> bool {
    grammar
        .traces
        .iter()
        .any(|trace| trace.raw_states[run_index] == target)
}

fn any_trace_persistent(grammar: &GrammarSet, run_index: usize, target: GrammarState) -> bool {
    grammar.traces.iter().any(|trace| match target {
        GrammarState::Boundary => trace.persistent_boundary[run_index],
        GrammarState::Violation => trace.persistent_violation[run_index],
        GrammarState::Admissible => false,
    })
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
            let earliest_dsfb_raw_boundary_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    any_trace_raw_state(grammar, run_index, GrammarState::Boundary)
                });
            let earliest_dsfb_persistent_boundary_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    any_trace_persistent(grammar, run_index, GrammarState::Boundary)
                });
            let earliest_dsfb_raw_violation_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    any_trace_raw_state(grammar, run_index, GrammarState::Violation)
                });
            let earliest_dsfb_persistent_violation_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    any_trace_persistent(grammar, run_index, GrammarState::Violation)
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
            let earliest_cusum_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    baselines.cusum.iter().any(|trace| trace.alarm[run_index])
                });
            let earliest_run_energy_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    baselines.run_energy.alarm[run_index]
                });
            let earliest_pca_fdc_run =
                earliest_signal_in_window(window_start, failure_index, |run_index| {
                    baselines.pca_fdc.alarm[run_index]
                });

            let dsfb_raw_boundary_lead_runs =
                earliest_dsfb_raw_boundary_run.map(|index| failure_index - index);
            let dsfb_persistent_boundary_lead_runs =
                earliest_dsfb_persistent_boundary_run.map(|index| failure_index - index);
            let dsfb_raw_violation_lead_runs =
                earliest_dsfb_raw_violation_run.map(|index| failure_index - index);
            let dsfb_persistent_violation_lead_runs =
                earliest_dsfb_persistent_violation_run.map(|index| failure_index - index);
            let threshold_lead_runs = earliest_threshold_run.map(|index| failure_index - index);
            let ewma_lead_runs = earliest_ewma_run.map(|index| failure_index - index);
            let cusum_lead_runs = earliest_cusum_run.map(|index| failure_index - index);
            let run_energy_lead_runs = earliest_run_energy_run.map(|index| failure_index - index);
            let pca_fdc_lead_runs = earliest_pca_fdc_run.map(|index| failure_index - index);

            PerFailureRunSignal {
                failure_run_index: failure_index,
                failure_timestamp: dataset.timestamps[failure_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                earliest_dsfb_raw_boundary_run,
                earliest_dsfb_persistent_boundary_run,
                earliest_dsfb_raw_violation_run,
                earliest_dsfb_persistent_violation_run,
                earliest_threshold_run,
                earliest_ewma_run,
                earliest_cusum_run,
                earliest_run_energy_run,
                earliest_pca_fdc_run,
                dsfb_raw_boundary_lead_runs,
                dsfb_persistent_boundary_lead_runs,
                dsfb_raw_violation_lead_runs,
                dsfb_persistent_violation_lead_runs,
                threshold_lead_runs,
                ewma_lead_runs,
                cusum_lead_runs,
                run_energy_lead_runs,
                pca_fdc_lead_runs,
                dsfb_raw_boundary_minus_cusum_delta_runs: paired_delta(
                    dsfb_raw_boundary_lead_runs,
                    cusum_lead_runs,
                ),
                dsfb_raw_boundary_minus_run_energy_delta_runs: paired_delta(
                    dsfb_raw_boundary_lead_runs,
                    run_energy_lead_runs,
                ),
                dsfb_raw_boundary_minus_pca_fdc_delta_runs: paired_delta(
                    dsfb_raw_boundary_lead_runs,
                    pca_fdc_lead_runs,
                ),
                dsfb_raw_boundary_minus_threshold_delta_runs: paired_delta(
                    dsfb_raw_boundary_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_raw_boundary_minus_ewma_delta_runs: paired_delta(
                    dsfb_raw_boundary_lead_runs,
                    ewma_lead_runs,
                ),
                dsfb_persistent_boundary_minus_threshold_delta_runs: paired_delta(
                    dsfb_persistent_boundary_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_persistent_boundary_minus_ewma_delta_runs: paired_delta(
                    dsfb_persistent_boundary_lead_runs,
                    ewma_lead_runs,
                ),
                dsfb_persistent_boundary_minus_cusum_delta_runs: paired_delta(
                    dsfb_persistent_boundary_lead_runs,
                    cusum_lead_runs,
                ),
                dsfb_persistent_boundary_minus_run_energy_delta_runs: paired_delta(
                    dsfb_persistent_boundary_lead_runs,
                    run_energy_lead_runs,
                ),
                dsfb_persistent_boundary_minus_pca_fdc_delta_runs: paired_delta(
                    dsfb_persistent_boundary_lead_runs,
                    pca_fdc_lead_runs,
                ),
                dsfb_raw_violation_minus_cusum_delta_runs: paired_delta(
                    dsfb_raw_violation_lead_runs,
                    cusum_lead_runs,
                ),
                dsfb_raw_violation_minus_run_energy_delta_runs: paired_delta(
                    dsfb_raw_violation_lead_runs,
                    run_energy_lead_runs,
                ),
                dsfb_raw_violation_minus_pca_fdc_delta_runs: paired_delta(
                    dsfb_raw_violation_lead_runs,
                    pca_fdc_lead_runs,
                ),
                dsfb_raw_violation_minus_threshold_delta_runs: paired_delta(
                    dsfb_raw_violation_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_raw_violation_minus_ewma_delta_runs: paired_delta(
                    dsfb_raw_violation_lead_runs,
                    ewma_lead_runs,
                ),
                dsfb_persistent_violation_minus_threshold_delta_runs: paired_delta(
                    dsfb_persistent_violation_lead_runs,
                    threshold_lead_runs,
                ),
                dsfb_persistent_violation_minus_ewma_delta_runs: paired_delta(
                    dsfb_persistent_violation_lead_runs,
                    ewma_lead_runs,
                ),
                dsfb_persistent_violation_minus_cusum_delta_runs: paired_delta(
                    dsfb_persistent_violation_lead_runs,
                    cusum_lead_runs,
                ),
                dsfb_persistent_violation_minus_run_energy_delta_runs: paired_delta(
                    dsfb_persistent_violation_lead_runs,
                    run_energy_lead_runs,
                ),
                dsfb_persistent_violation_minus_pca_fdc_delta_runs: paired_delta(
                    dsfb_persistent_violation_lead_runs,
                    pca_fdc_lead_runs,
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
        failure_runs_with_raw_boundary_lead: count_present(
            records
                .iter()
                .map(|record| record.dsfb_raw_boundary_lead_runs),
        ),
        failure_runs_with_persistent_boundary_lead: count_present(
            records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_lead_runs),
        ),
        failure_runs_with_raw_violation_lead: count_present(
            records
                .iter()
                .map(|record| record.dsfb_raw_violation_lead_runs),
        ),
        failure_runs_with_persistent_violation_lead: count_present(
            records
                .iter()
                .map(|record| record.dsfb_persistent_violation_lead_runs),
        ),
        failure_runs_with_threshold_lead: count_present(
            records.iter().map(|record| record.threshold_lead_runs),
        ),
        failure_runs_with_ewma_lead: count_present(
            records.iter().map(|record| record.ewma_lead_runs),
        ),
        failure_runs_with_cusum_lead: count_present(
            records.iter().map(|record| record.cusum_lead_runs),
        ),
        failure_runs_with_run_energy_lead: count_present(
            records.iter().map(|record| record.run_energy_lead_runs),
        ),
        failure_runs_with_pca_fdc_lead: count_present(
            records.iter().map(|record| record.pca_fdc_lead_runs),
        ),
        mean_raw_boundary_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_lead_runs)
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
        mean_cusum_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.cusum_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_run_energy_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.run_energy_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_pca_fdc_lead_runs: mean_option_usize(
            &records
                .iter()
                .map(|record| record.pca_fdc_lead_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_boundary_minus_cusum_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_minus_cusum_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_boundary_minus_run_energy_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_minus_run_energy_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_boundary_minus_pca_fdc_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_minus_pca_fdc_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_boundary_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_boundary_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_boundary_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_minus_cusum_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_minus_cusum_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_minus_run_energy_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_minus_run_energy_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_boundary_minus_pca_fdc_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_boundary_minus_pca_fdc_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_minus_cusum_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_minus_cusum_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_minus_run_energy_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_minus_run_energy_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_minus_pca_fdc_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_minus_pca_fdc_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_raw_violation_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_raw_violation_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_minus_threshold_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_minus_ewma_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_minus_cusum_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_minus_cusum_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_minus_run_energy_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_minus_run_energy_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_persistent_violation_minus_pca_fdc_delta_runs: mean_option_i64(
            &records
                .iter()
                .map(|record| record.dsfb_persistent_violation_minus_pca_fdc_delta_runs)
                .collect::<Vec<_>>(),
        ),
    }
}

fn compute_density_metrics(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    density_window: usize,
    failure_window_mask: &[bool],
) -> Vec<DensityMetricRecord> {
    let analyzable_feature_indices = nominal
        .features
        .iter()
        .filter(|feature| feature.analyzable)
        .map(|feature| feature.feature_index)
        .collect::<Vec<_>>();
    let feature_denominator = analyzable_feature_indices.len().max(1);

    (0..dataset.labels.len())
        .map(|run_index| {
            let start = run_index.saturating_sub(density_window.saturating_sub(1));
            let window_len = run_index - start + 1;
            let denominator = (window_len * feature_denominator) as f64;
            let mut raw_boundary_hits = 0usize;
            let mut persistent_boundary_hits = 0usize;
            let mut raw_violation_hits = 0usize;
            let mut persistent_violation_hits = 0usize;
            let mut threshold_hits = 0usize;
            let mut ewma_hits = 0usize;
            let mut cusum_hits = 0usize;

            for &feature_index in &analyzable_feature_indices {
                let grammar_trace = &grammar.traces[feature_index];
                let residual_trace = &residuals.traces[feature_index];
                let ewma_trace = &baselines.ewma[feature_index];
                let cusum_trace = &baselines.cusum[feature_index];
                for offset in start..=run_index {
                    if grammar_trace.raw_states[offset] == GrammarState::Boundary {
                        raw_boundary_hits += 1;
                    }
                    if grammar_trace.persistent_boundary[offset] {
                        persistent_boundary_hits += 1;
                    }
                    if grammar_trace.raw_states[offset] == GrammarState::Violation {
                        raw_violation_hits += 1;
                    }
                    if grammar_trace.persistent_violation[offset] {
                        persistent_violation_hits += 1;
                    }
                    if residual_trace.threshold_alarm[offset] {
                        threshold_hits += 1;
                    }
                    if ewma_trace.alarm[offset] {
                        ewma_hits += 1;
                    }
                    if cusum_trace.alarm[offset] {
                        cusum_hits += 1;
                    }
                }
            }

            DensityMetricRecord {
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                in_pre_failure_window: failure_window_mask[run_index],
                raw_boundary_density: raw_boundary_hits as f64 / denominator,
                persistent_boundary_density: persistent_boundary_hits as f64 / denominator,
                raw_violation_density: raw_violation_hits as f64 / denominator,
                persistent_violation_density: persistent_violation_hits as f64 / denominator,
                threshold_density: threshold_hits as f64 / denominator,
                ewma_density: ewma_hits as f64 / denominator,
                cusum_density: cusum_hits as f64 / denominator,
            }
        })
        .collect()
}

fn summarize_densities(records: &[DensityMetricRecord], density_window: usize) -> DensitySummary {
    let failure_records = records
        .iter()
        .filter(|record| record.label == 1)
        .collect::<Vec<_>>();
    let pass_records = records
        .iter()
        .filter(|record| record.label == -1)
        .collect::<Vec<_>>();

    DensitySummary {
        density_window,
        mean_raw_boundary_density_failure: mean_record_field(&failure_records, |record| {
            record.raw_boundary_density
        }),
        mean_raw_boundary_density_pass: mean_record_field(&pass_records, |record| {
            record.raw_boundary_density
        }),
        mean_persistent_boundary_density_failure: mean_record_field(&failure_records, |record| {
            record.persistent_boundary_density
        }),
        mean_persistent_boundary_density_pass: mean_record_field(&pass_records, |record| {
            record.persistent_boundary_density
        }),
        mean_raw_violation_density_failure: mean_record_field(&failure_records, |record| {
            record.raw_violation_density
        }),
        mean_raw_violation_density_pass: mean_record_field(&pass_records, |record| {
            record.raw_violation_density
        }),
        mean_persistent_violation_density_failure: mean_record_field(&failure_records, |record| {
            record.persistent_violation_density
        }),
        mean_persistent_violation_density_pass: mean_record_field(&pass_records, |record| {
            record.persistent_violation_density
        }),
        mean_threshold_density_failure: mean_record_field(&failure_records, |record| {
            record.threshold_density
        }),
        mean_threshold_density_pass: mean_record_field(&pass_records, |record| {
            record.threshold_density
        }),
        mean_ewma_density_failure: mean_record_field(&failure_records, |record| {
            record.ewma_density
        }),
        mean_ewma_density_pass: mean_record_field(&pass_records, |record| record.ewma_density),
        mean_cusum_density_failure: mean_record_field(&failure_records, |record| {
            record.cusum_density
        }),
        mean_cusum_density_pass: mean_record_field(&pass_records, |record| record.cusum_density),
    }
}

fn mean_record_field<F>(records: &[&DensityMetricRecord], selector: F) -> f64
where
    F: Fn(&DensityMetricRecord) -> f64,
{
    if records.is_empty() {
        0.0
    } else {
        records.iter().map(|record| selector(record)).sum::<f64>() / records.len() as f64
    }
}

fn compute_boundary_episode_summary(grammar: &GrammarSet) -> BoundaryEpisodeSummary {
    let raw = episode_stats(grammar, false);
    let persistent = episode_stats(grammar, true);

    BoundaryEpisodeSummary {
        raw_episode_count: raw.episode_count,
        persistent_episode_count: persistent.episode_count,
        mean_raw_episode_length: raw.mean_episode_length,
        mean_persistent_episode_length: persistent.mean_episode_length,
        max_raw_episode_length: raw.max_episode_length,
        max_persistent_episode_length: persistent.max_episode_length,
        raw_non_escalating_episode_fraction: raw.non_escalating_episode_fraction,
        persistent_non_escalating_episode_fraction: persistent.non_escalating_episode_fraction,
    }
}

struct EpisodeStats {
    episode_count: usize,
    mean_episode_length: Option<f64>,
    max_episode_length: usize,
    non_escalating_episode_fraction: Option<f64>,
}

fn episode_stats(grammar: &GrammarSet, persistent: bool) -> EpisodeStats {
    let mut episode_count = 0usize;
    let mut total_length = 0usize;
    let mut max_length = 0usize;
    let mut non_escalating_episode_count = 0usize;

    for trace in &grammar.traces {
        let mut index = 0usize;
        let len = trace.states.len();
        while index < len {
            let in_boundary = if persistent {
                trace.persistent_boundary[index]
            } else {
                trace.raw_states[index] == GrammarState::Boundary
            };
            if !in_boundary {
                index += 1;
                continue;
            }

            let start = index;
            while index < len
                && if persistent {
                    trace.persistent_boundary[index]
                } else {
                    trace.raw_states[index] == GrammarState::Boundary
                }
            {
                index += 1;
            }
            let length = index - start;
            episode_count += 1;
            total_length += length;
            max_length = max_length.max(length);

            let escalates = if persistent {
                index < len && trace.persistent_violation[index]
            } else {
                index < len && trace.raw_states[index] == GrammarState::Violation
            };
            if !escalates {
                non_escalating_episode_count += 1;
            }
        }
    }

    EpisodeStats {
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
                for (run_index, trace_reason) in trace.raw_reasons.iter().enumerate() {
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

fn count_present<I, T>(iter: I) -> usize
where
    I: Iterator<Item = Option<T>>,
{
    iter.filter(|value| value.is_some()).count()
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
