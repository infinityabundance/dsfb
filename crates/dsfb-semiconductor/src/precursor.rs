use crate::baselines::BaselineSet;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarReason, GrammarSet, GrammarState};
use crate::heuristics::dsa_contributing_motif_names;
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsaConfig {
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
}

impl Default for DsaConfig {
    fn default() -> Self {
        Self {
            window: 10,
            persistence_runs: 2,
            alert_tau: 2.5,
        }
    }
}

impl DsaConfig {
    pub fn validate(&self) -> Result<()> {
        if self.window == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "dsa window must be positive".into(),
            ));
        }
        if self.persistence_runs == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "dsa persistence_runs must be positive".into(),
            ));
        }
        if self.alert_tau <= 0.0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "dsa alert_tau must be positive".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsaWeights {
    pub boundary_density: f64,
    pub drift_persistence: f64,
    pub slew_density: f64,
    pub ewma_occupancy: f64,
    pub motif_recurrence: f64,
}

impl Default for DsaWeights {
    fn default() -> Self {
        Self {
            boundary_density: 1.0,
            drift_persistence: 1.0,
            slew_density: 1.0,
            ewma_occupancy: 1.0,
            motif_recurrence: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaParameterManifest {
    pub config: DsaConfig,
    pub weights: DsaWeights,
    pub primary_run_signal: String,
    pub secondary_run_signal: String,
    pub rolling_window_definition: String,
    pub boundary_density_basis: String,
    pub drift_persistence_definition: String,
    pub slew_density_definition: String,
    pub ewma_occupancy_formula: String,
    pub motif_names_used_for_recurrence: Vec<String>,
    pub directional_consistency_rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub boundary_basis_hit: Vec<bool>,
    pub drift_outward_hit: Vec<bool>,
    pub slew_hit: Vec<bool>,
    pub motif_hit: Vec<bool>,
    pub boundary_density_w: Vec<f64>,
    pub drift_persistence_w: Vec<f64>,
    pub slew_density_w: Vec<f64>,
    pub ewma_occupancy_w: Vec<f64>,
    pub motif_recurrence_w: Vec<f64>,
    pub consistent: Vec<bool>,
    pub dsa_score: Vec<f64>,
    pub dsa_active: Vec<bool>,
    pub dsa_alert: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaRunSignals {
    pub primary_run_signal: String,
    pub any_feature_dsa_alert: Vec<bool>,
    pub feature_count_dsa_alert: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaEpisodeSummary {
    pub primary_signal: String,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaSignalSummary {
    pub config: DsaConfig,
    pub weights: DsaWeights,
    pub primary_run_signal: String,
    pub analyzable_feature_count: usize,
    pub alert_point_count: usize,
    pub alert_run_count: usize,
    pub failure_runs: usize,
    pub failure_run_recall: usize,
    pub failure_run_recall_rate: f64,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
    pub raw_boundary_nuisance_proxy: f64,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub any_metric_improved: bool,
    pub validation_passed: bool,
    pub validation_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalComparisonRow {
    pub signal: String,
    pub failure_run_recall: usize,
    pub failure_runs: usize,
    pub failure_run_recall_rate: f64,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaVsBaselinesSummary {
    pub dataset: String,
    pub primary_run_signal: String,
    pub dsa: SignalComparisonRow,
    pub threshold: SignalComparisonRow,
    pub ewma: SignalComparisonRow,
    pub dsfb_violation: SignalComparisonRow,
    pub dsfb_raw_boundary: SignalComparisonRow,
    pub episode_summary: DsaEpisodeSummary,
    pub failure_recall_delta_vs_threshold: i64,
    pub failure_recall_delta_vs_ewma: i64,
    pub failure_recall_delta_vs_violation: i64,
    pub pass_run_nuisance_delta_vs_threshold: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_raw_boundary: f64,
    pub nuisance_improved: bool,
    pub lead_time_improved: bool,
    pub recall_preserved: bool,
    pub compression_improved: bool,
    pub nothing_improved: bool,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub any_metric_improved: bool,
    pub validation_passed: bool,
    pub validation_failures: Vec<String>,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerFailureRunDsaSignal {
    pub failure_run_index: usize,
    pub failure_timestamp: String,
    pub earliest_dsa_run: Option<usize>,
    pub earliest_dsa_feature_index: Option<usize>,
    pub earliest_dsa_feature_name: Option<String>,
    pub dsa_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub dsa_minus_threshold_delta_runs: Option<i64>,
    pub dsa_minus_ewma_delta_runs: Option<i64>,
    pub dsa_alerting_feature_count: usize,
    pub max_dsa_score_in_lookback: Option<f64>,
    pub max_dsa_score_feature_index: Option<usize>,
    pub max_dsa_score_feature_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaEvaluation {
    pub traces: Vec<DsaFeatureTrace>,
    pub run_signals: DsaRunSignals,
    pub episode_summary: DsaEpisodeSummary,
    pub parameter_manifest: DsaParameterManifest,
    pub summary: DsaSignalSummary,
    pub comparison_summary: DsaVsBaselinesSummary,
    pub per_failure_run_signals: Vec<PerFailureRunDsaSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsaCalibrationGrid {
    pub window: Vec<usize>,
    pub persistence_runs: Vec<usize>,
    pub alert_tau: Vec<f64>,
}

impl DsaCalibrationGrid {
    pub fn bounded_default() -> Self {
        Self {
            window: vec![5, 10, 15],
            persistence_runs: vec![2, 3, 4],
            alert_tau: vec![2.0, 2.5, 3.0],
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.grid_point_count() == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "dsa calibration grid must contain at least one point".into(),
            ));
        }
        if self.grid_point_count() > 256 {
            return Err(DsfbSemiconductorError::DatasetFormat(format!(
                "dsa calibration grid is too large ({})",
                self.grid_point_count()
            )));
        }
        Ok(())
    }

    pub fn grid_point_count(&self) -> usize {
        [
            self.window.len(),
            self.persistence_runs.len(),
            self.alert_tau.len(),
        ]
        .into_iter()
        .product()
    }

    pub fn expand(&self) -> Vec<DsaConfig> {
        let mut out = Vec::with_capacity(self.grid_point_count());
        for &window in &self.window {
            for &persistence_runs in &self.persistence_runs {
                for &alert_tau in &self.alert_tau {
                    out.push(DsaConfig {
                        window,
                        persistence_runs,
                        alert_tau,
                    });
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaCalibrationRow {
    pub config_id: usize,
    pub primary_run_signal: String,
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub failure_run_recall: usize,
    pub failure_runs: usize,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
    pub pass_run_nuisance_delta_vs_threshold: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_raw_boundary: f64,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
    pub nuisance_improved: bool,
    pub lead_time_improved: bool,
    pub recall_preserved: bool,
    pub compression_improved: bool,
    pub any_metric_improved: bool,
    pub nothing_improved: bool,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub validation_passed: bool,
    pub validation_failures: String,
}

pub fn evaluate_dsa(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    config: &DsaConfig,
    pre_failure_lookback_runs: usize,
) -> Result<DsaEvaluation> {
    config.validate()?;
    let weights = DsaWeights::default();
    let run_count = dataset.labels.len();
    let motif_names = dsa_contributing_motif_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    let mut traces = Vec::with_capacity(residuals.traces.len());

    for (((residual_trace, sign_trace), ewma_trace), grammar_trace) in residuals
        .traces
        .iter()
        .zip(&signs.traces)
        .zip(&baselines.ewma)
        .zip(&grammar.traces)
    {
        let feature = &nominal.features[residual_trace.feature_index];
        if !feature.analyzable {
            traces.push(empty_trace(feature.feature_index, &feature.feature_name, run_count));
            continue;
        }

        let boundary_basis_hit = grammar_trace
            .raw_states
            .iter()
            .map(|state| *state == GrammarState::Boundary)
            .collect::<Vec<_>>();
        let drift_outward_hit = sign_trace
            .drift
            .iter()
            .map(|drift| *drift >= sign_trace.drift_threshold)
            .collect::<Vec<_>>();
        let slew_hit = sign_trace
            .slew
            .iter()
            .map(|slew| slew.abs() >= sign_trace.slew_threshold)
            .collect::<Vec<_>>();
        let motif_hit = grammar_trace
            .raw_reasons
            .iter()
            .map(|reason| {
                dsa_motif_name(reason)
                    .map(|name| motif_names.iter().any(|candidate| candidate == name))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        let ewma_normalized = ewma_trace
            .ewma
            .iter()
            .map(|value| normalize_to_threshold(*value, ewma_trace.threshold))
            .collect::<Vec<_>>();

        let boundary_prefix = bool_prefix_sum(&boundary_basis_hit);
        let drift_prefix = bool_prefix_sum(&drift_outward_hit);
        let slew_prefix = bool_prefix_sum(&slew_hit);
        let motif_prefix = bool_prefix_sum(&motif_hit);

        let mut boundary_density_w = Vec::with_capacity(run_count);
        let mut drift_persistence_w = Vec::with_capacity(run_count);
        let mut slew_density_w = Vec::with_capacity(run_count);
        let mut ewma_occupancy_w = Vec::with_capacity(run_count);
        let mut motif_recurrence_w = Vec::with_capacity(run_count);
        let mut consistent = Vec::with_capacity(run_count);
        let mut dsa_score = Vec::with_capacity(run_count);
        let mut dsa_active = Vec::with_capacity(run_count);

        for run_index in 0..run_count {
            let start = run_index.saturating_sub(config.window.saturating_sub(1));
            let window_len = (run_index - start + 1) as f64;
            let boundary_density = window_fraction(&boundary_prefix, start, run_index, window_len);
            let drift_persistence = window_fraction(&drift_prefix, start, run_index, window_len);
            let slew_density = window_fraction(&slew_prefix, start, run_index, window_len);
            let ewma_occupancy = window_mean(&ewma_normalized, start, run_index);
            let motif_recurrence = window_fraction(&motif_prefix, start, run_index, window_len);
            let consistent_window =
                window_is_consistent(&sign_trace.drift, sign_trace.drift_threshold, start, run_index);
            let score = weights.boundary_density * boundary_density
                + weights.drift_persistence * drift_persistence
                + weights.slew_density * slew_density
                + weights.ewma_occupancy * ewma_occupancy
                + weights.motif_recurrence * motif_recurrence;

            boundary_density_w.push(boundary_density);
            drift_persistence_w.push(drift_persistence);
            slew_density_w.push(slew_density);
            ewma_occupancy_w.push(ewma_occupancy);
            motif_recurrence_w.push(motif_recurrence);
            consistent.push(consistent_window);
            dsa_score.push(score);
            dsa_active.push(score >= config.alert_tau && consistent_window);
        }

        let dsa_alert = persistence_mask(&dsa_active, config.persistence_runs);
        traces.push(DsaFeatureTrace {
            feature_index: feature.feature_index,
            feature_name: feature.feature_name.clone(),
            boundary_basis_hit,
            drift_outward_hit,
            slew_hit,
            motif_hit,
            boundary_density_w,
            drift_persistence_w,
            slew_density_w,
            ewma_occupancy_w,
            motif_recurrence_w,
            consistent,
            dsa_score,
            dsa_active,
            dsa_alert,
        });
    }

    let run_signals = build_run_signals(&traces, run_count);
    let raw_boundary_run_signal = (0..run_count)
        .map(|run_index| {
            grammar
                .traces
                .iter()
                .any(|trace| trace.raw_states[run_index] == GrammarState::Boundary)
        })
        .collect::<Vec<_>>();
    let raw_violation_run_signal = (0..run_count)
        .map(|run_index| {
            grammar
                .traces
                .iter()
                .any(|trace| trace.raw_states[run_index] == GrammarState::Violation)
        })
        .collect::<Vec<_>>();
    let threshold_run_signal = (0..run_count)
        .map(|run_index| {
            residuals
                .traces
                .iter()
                .any(|trace| trace.threshold_alarm[run_index])
        })
        .collect::<Vec<_>>();
    let ewma_run_signal = (0..run_count)
        .map(|run_index| baselines.ewma.iter().any(|trace| trace.alarm[run_index]))
        .collect::<Vec<_>>();

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

    let raw_boundary_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&raw_boundary_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let raw_violation_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&raw_violation_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let threshold_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&threshold_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let ewma_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&ewma_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();

    let per_failure_run_signals = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            let earliest_dsa =
                earliest_dsa_signal(&traces, &run_signals.any_feature_dsa_alert, window_start, failure_index);
            let earliest_threshold_run =
                earliest_run_signal(&threshold_run_signal, window_start, failure_index);
            let earliest_ewma_run = earliest_run_signal(&ewma_run_signal, window_start, failure_index);
            let dsa_lead_runs = earliest_dsa
                .as_ref()
                .map(|signal| failure_index - signal.run_index);
            let threshold_lead_runs = earliest_threshold_run.map(|index| failure_index - index);
            let ewma_lead_runs = earliest_ewma_run.map(|index| failure_index - index);
            let alerting_feature_count = traces
                .iter()
                .filter(|trace| trace.dsa_alert[window_start..failure_index].iter().any(|flag| *flag))
                .count();
            let max_score = max_dsa_score(&traces, window_start, failure_index);

            PerFailureRunDsaSignal {
                failure_run_index: failure_index,
                failure_timestamp: dataset.timestamps[failure_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                earliest_dsa_run: earliest_dsa.as_ref().map(|signal| signal.run_index),
                earliest_dsa_feature_index: earliest_dsa.as_ref().map(|signal| signal.feature_index),
                earliest_dsa_feature_name: earliest_dsa
                    .as_ref()
                    .map(|signal| signal.feature_name.clone()),
                dsa_lead_runs,
                threshold_lead_runs,
                ewma_lead_runs,
                dsa_minus_threshold_delta_runs: paired_delta(dsa_lead_runs, threshold_lead_runs),
                dsa_minus_ewma_delta_runs: paired_delta(dsa_lead_runs, ewma_lead_runs),
                dsa_alerting_feature_count: alerting_feature_count,
                max_dsa_score_in_lookback: max_score.as_ref().map(|score| score.score),
                max_dsa_score_feature_index: max_score.as_ref().map(|score| score.feature_index),
                max_dsa_score_feature_name: max_score
                    .as_ref()
                    .map(|score| score.feature_name.clone()),
            }
        })
        .collect::<Vec<_>>();

    let alert_point_count = traces
        .iter()
        .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let alert_run_count = run_signals
        .any_feature_dsa_alert
        .iter()
        .filter(|flag| **flag)
        .count();
    let failure_run_recall = per_failure_run_signals
        .iter()
        .filter(|signal| signal.earliest_dsa_run.is_some())
        .count();
    let dsa_row = SignalComparisonRow {
        signal: "DSA".into(),
        failure_run_recall,
        failure_runs: failure_indices.len(),
        failure_run_recall_rate: rate(failure_run_recall, failure_indices.len()),
        mean_lead_time_runs: mean_option_usize(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_lead_runs)
                .collect::<Vec<_>>(),
        ),
        median_lead_time_runs: median_option_usize(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_lead_runs)
                .collect::<Vec<_>>(),
        ),
        pass_run_nuisance_proxy: rate(
            pass_indices
                .iter()
                .filter(|&&run_index| run_signals.any_feature_dsa_alert[run_index])
                .count(),
            pass_indices.len(),
        ),
        mean_lead_delta_vs_threshold_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_lead_delta_vs_ewma_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
    };
    let threshold_row = baseline_row(
        "Threshold",
        failure_indices.len(),
        &threshold_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| threshold_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );
    let ewma_row = baseline_row(
        "EWMA",
        failure_indices.len(),
        &ewma_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| ewma_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );
    let dsfb_violation_row = baseline_row(
        "DSFB Violation",
        failure_indices.len(),
        &raw_violation_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| raw_violation_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );
    let dsfb_raw_boundary_row = baseline_row(
        "DSFB Raw Boundary",
        failure_indices.len(),
        &raw_boundary_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| raw_boundary_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );

    let raw_boundary_episode_count = grammar
        .traces
        .iter()
        .map(|trace| {
            episode_ranges(
                &trace
                    .raw_states
                    .iter()
                    .map(|state| *state == GrammarState::Boundary)
                    .collect::<Vec<_>>(),
            )
            .len()
        })
        .sum::<usize>();
    let episode_summary = compute_episode_summary(
        &run_signals.primary_run_signal,
        &run_signals.any_feature_dsa_alert,
        raw_boundary_episode_count,
        &raw_violation_run_signal,
    );
    let nuisance_improved = dsa_row.pass_run_nuisance_proxy < threshold_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < ewma_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < dsfb_raw_boundary_row.pass_run_nuisance_proxy;
    let lead_time_improved =
        matches!(dsa_row.mean_lead_delta_vs_threshold_runs, Some(delta) if delta > 0.0)
            || matches!(dsa_row.mean_lead_delta_vs_ewma_runs, Some(delta) if delta > 0.0);
    let recall_preserved = dsa_row.failure_run_recall >= threshold_row.failure_run_recall
        && dsa_row.failure_run_recall >= ewma_row.failure_run_recall
        && dsa_row.failure_run_recall >= dsfb_violation_row.failure_run_recall;
    let recall_improved = dsa_row.failure_run_recall > threshold_row.failure_run_recall
        || dsa_row.failure_run_recall > ewma_row.failure_run_recall
        || dsa_row.failure_run_recall > dsfb_violation_row.failure_run_recall;
    let compression_improved =
        matches!(episode_summary.compression_ratio, Some(ratio) if ratio > 1.0);
    let any_metric_improved =
        nuisance_improved || lead_time_improved || recall_improved || compression_improved;
    let nothing_improved = !any_metric_improved;
    let threshold_recall_gate_passed =
        dsa_row.failure_run_recall >= threshold_row.failure_run_recall;
    let boundary_nuisance_gate_passed =
        dsa_row.pass_run_nuisance_proxy < dsfb_raw_boundary_row.pass_run_nuisance_proxy;
    let validation_failures = validation_failures(
        &dsa_row,
        &threshold_row,
        dsfb_raw_boundary_row.pass_run_nuisance_proxy,
        threshold_recall_gate_passed,
        boundary_nuisance_gate_passed,
        any_metric_improved,
    );
    let validation_passed = validation_failures.is_empty();
    let conclusion = dsa_conclusion(
        &dsa_row,
        &threshold_row,
        &ewma_row,
        &dsfb_raw_boundary_row,
        &episode_summary,
        nuisance_improved,
        lead_time_improved,
        recall_preserved,
        compression_improved,
        nothing_improved,
        &validation_failures,
    );

    let comparison_summary = DsaVsBaselinesSummary {
        dataset: "SECOM".into(),
        primary_run_signal: run_signals.primary_run_signal.clone(),
        dsa: dsa_row.clone(),
        threshold: threshold_row.clone(),
        ewma: ewma_row.clone(),
        dsfb_violation: dsfb_violation_row.clone(),
        dsfb_raw_boundary: dsfb_raw_boundary_row.clone(),
        episode_summary: episode_summary.clone(),
        failure_recall_delta_vs_threshold: dsa_row.failure_run_recall as i64
            - threshold_row.failure_run_recall as i64,
        failure_recall_delta_vs_ewma: dsa_row.failure_run_recall as i64
            - ewma_row.failure_run_recall as i64,
        failure_recall_delta_vs_violation: dsa_row.failure_run_recall as i64
            - dsfb_violation_row.failure_run_recall as i64,
        pass_run_nuisance_delta_vs_threshold: dsa_row.pass_run_nuisance_proxy
            - threshold_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_ewma: dsa_row.pass_run_nuisance_proxy
            - ewma_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_raw_boundary: dsa_row.pass_run_nuisance_proxy
            - dsfb_raw_boundary_row.pass_run_nuisance_proxy,
        nuisance_improved,
        lead_time_improved,
        recall_preserved,
        compression_improved,
        nothing_improved,
        threshold_recall_gate_passed,
        boundary_nuisance_gate_passed,
        any_metric_improved,
        validation_passed,
        validation_failures: validation_failures.clone(),
        conclusion,
    };
    let parameter_manifest = DsaParameterManifest {
        config: config.clone(),
        weights: weights.clone(),
        primary_run_signal: run_signals.primary_run_signal.clone(),
        secondary_run_signal: "feature_count_dsa_alert(k)".into(),
        rolling_window_definition: format!(
            "Trailing inclusive rolling window of up to {} runs per analyzable feature.",
            config.window
        ),
        boundary_density_basis:
            "Fraction of the trailing window where the feature is in the raw DSFB Boundary state."
                .into(),
        drift_persistence_definition:
            "Fraction of the trailing window where drift >= drift threshold and the drift sign is outward (positive residual-norm drift)."
                .into(),
        slew_density_definition:
            "Fraction of the trailing window where absolute slew >= slew threshold.".into(),
        ewma_occupancy_formula:
            "Mean over the trailing window of clamp(EWMA / EWMA_threshold, 0, 1).".into(),
        motif_names_used_for_recurrence: motif_names,
        directional_consistency_rule:
            "CONSISTENT is true only when thresholded drift signs in the window are never inward and the nonzero drift sign never flips within the window."
                .into(),
    };

    Ok(DsaEvaluation {
        traces,
        run_signals,
        episode_summary: episode_summary.clone(),
        parameter_manifest,
        summary: DsaSignalSummary {
            config: config.clone(),
            weights,
            primary_run_signal: "any_feature_dsa_alert(k)".into(),
            analyzable_feature_count: nominal
                .features
                .iter()
                .filter(|feature| feature.analyzable)
                .count(),
            alert_point_count,
            alert_run_count,
            failure_runs: failure_indices.len(),
            failure_run_recall,
            failure_run_recall_rate: dsa_row.failure_run_recall_rate,
            mean_lead_time_runs: dsa_row.mean_lead_time_runs,
            median_lead_time_runs: dsa_row.median_lead_time_runs,
            pass_run_nuisance_proxy: dsa_row.pass_run_nuisance_proxy,
            mean_lead_delta_vs_threshold_runs: dsa_row.mean_lead_delta_vs_threshold_runs,
            mean_lead_delta_vs_ewma_runs: dsa_row.mean_lead_delta_vs_ewma_runs,
            raw_boundary_nuisance_proxy: dsfb_raw_boundary_row.pass_run_nuisance_proxy,
            raw_boundary_episode_count: episode_summary.raw_boundary_episode_count,
            dsa_episode_count: episode_summary.dsa_episode_count,
            mean_dsa_episode_length_runs: episode_summary.mean_dsa_episode_length_runs,
            max_dsa_episode_length_runs: episode_summary.max_dsa_episode_length_runs,
            compression_ratio: episode_summary.compression_ratio,
            non_escalating_dsa_episode_fraction: episode_summary
                .non_escalating_dsa_episode_fraction,
            threshold_recall_gate_passed,
            boundary_nuisance_gate_passed,
            any_metric_improved,
            validation_passed,
            validation_failures,
        },
        comparison_summary,
        per_failure_run_signals,
    })
}

pub fn run_dsa_calibration_grid(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    grid: &DsaCalibrationGrid,
    pre_failure_lookback_runs: usize,
) -> Result<Vec<DsaCalibrationRow>> {
    grid.validate()?;

    let mut rows = Vec::with_capacity(grid.grid_point_count());
    for (config_id, config) in grid.expand().into_iter().enumerate() {
        let evaluation = evaluate_dsa(
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            &config,
            pre_failure_lookback_runs,
        )?;
        rows.push(DsaCalibrationRow {
            config_id,
            primary_run_signal: evaluation.run_signals.primary_run_signal.clone(),
            window: config.window,
            persistence_runs: config.persistence_runs,
            alert_tau: config.alert_tau,
            failure_run_recall: evaluation.summary.failure_run_recall,
            failure_runs: evaluation.summary.failure_runs,
            mean_lead_time_runs: evaluation.summary.mean_lead_time_runs,
            median_lead_time_runs: evaluation.summary.median_lead_time_runs,
            pass_run_nuisance_proxy: evaluation.summary.pass_run_nuisance_proxy,
            mean_lead_delta_vs_threshold_runs: evaluation
                .summary
                .mean_lead_delta_vs_threshold_runs,
            mean_lead_delta_vs_ewma_runs: evaluation.summary.mean_lead_delta_vs_ewma_runs,
            pass_run_nuisance_delta_vs_threshold: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_threshold,
            pass_run_nuisance_delta_vs_ewma: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_ewma,
            pass_run_nuisance_delta_vs_raw_boundary: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_raw_boundary,
            raw_boundary_episode_count: evaluation.episode_summary.raw_boundary_episode_count,
            dsa_episode_count: evaluation.episode_summary.dsa_episode_count,
            mean_dsa_episode_length_runs: evaluation
                .episode_summary
                .mean_dsa_episode_length_runs,
            max_dsa_episode_length_runs: evaluation
                .episode_summary
                .max_dsa_episode_length_runs,
            compression_ratio: evaluation.episode_summary.compression_ratio,
            non_escalating_dsa_episode_fraction: evaluation
                .episode_summary
                .non_escalating_dsa_episode_fraction,
            nuisance_improved: evaluation.comparison_summary.nuisance_improved,
            lead_time_improved: evaluation.comparison_summary.lead_time_improved,
            recall_preserved: evaluation.comparison_summary.recall_preserved,
            compression_improved: evaluation.comparison_summary.compression_improved,
            any_metric_improved: evaluation.summary.any_metric_improved,
            nothing_improved: evaluation.comparison_summary.nothing_improved,
            threshold_recall_gate_passed: evaluation.summary.threshold_recall_gate_passed,
            boundary_nuisance_gate_passed: evaluation.summary.boundary_nuisance_gate_passed,
            validation_passed: evaluation.summary.validation_passed,
            validation_failures: evaluation.summary.validation_failures.join("; "),
        });
    }

    Ok(rows)
}

fn empty_trace(feature_index: usize, feature_name: &str, run_count: usize) -> DsaFeatureTrace {
    DsaFeatureTrace {
        feature_index,
        feature_name: feature_name.into(),
        boundary_basis_hit: vec![false; run_count],
        drift_outward_hit: vec![false; run_count],
        slew_hit: vec![false; run_count],
        motif_hit: vec![false; run_count],
        boundary_density_w: vec![0.0; run_count],
        drift_persistence_w: vec![0.0; run_count],
        slew_density_w: vec![0.0; run_count],
        ewma_occupancy_w: vec![0.0; run_count],
        motif_recurrence_w: vec![0.0; run_count],
        consistent: vec![true; run_count],
        dsa_score: vec![0.0; run_count],
        dsa_active: vec![false; run_count],
        dsa_alert: vec![false; run_count],
    }
}

fn dsa_motif_name(reason: &GrammarReason) -> Option<&'static str> {
    match reason {
        GrammarReason::SustainedOutwardDrift => Some("pre_failure_slow_drift"),
        GrammarReason::AbruptSlewViolation => Some("transient_excursion"),
        GrammarReason::RecurrentBoundaryGrazing => Some("recurrent_boundary_approach"),
        GrammarReason::Admissible | GrammarReason::EnvelopeViolation => None,
    }
}

fn normalize_to_threshold(value: f64, threshold: f64) -> f64 {
    if threshold <= 0.0 {
        0.0
    } else {
        (value / threshold).clamp(0.0, 1.0)
    }
}

fn bool_prefix_sum(flags: &[bool]) -> Vec<usize> {
    let mut prefix = Vec::with_capacity(flags.len() + 1);
    prefix.push(0);
    let mut total = 0usize;
    for flag in flags {
        total += usize::from(*flag);
        prefix.push(total);
    }
    prefix
}

fn window_count(prefix: &[usize], start: usize, end: usize) -> usize {
    prefix[end + 1] - prefix[start]
}

fn window_fraction(prefix: &[usize], start: usize, end: usize, window_len: f64) -> f64 {
    window_count(prefix, start, end) as f64 / window_len
}

fn window_mean(values: &[f64], start: usize, end: usize) -> f64 {
    let slice = &values[start..=end];
    slice.iter().sum::<f64>() / slice.len() as f64
}

fn drift_sign(value: f64) -> i8 {
    if value > 1.0e-12 {
        1
    } else if value < -1.0e-12 {
        -1
    } else {
        0
    }
}

fn window_is_consistent(drift: &[f64], drift_threshold: f64, start: usize, end: usize) -> bool {
    let mut previous_nonzero = 0i8;
    let mut thresholded_signs = Vec::new();

    for run_index in start..=end {
        let sign = drift_sign(drift[run_index]);
        if sign != 0 {
            if previous_nonzero != 0 && sign != previous_nonzero {
                return false;
            }
            previous_nonzero = sign;
        }

        if drift[run_index] >= drift_threshold {
            thresholded_signs.push(1);
        } else if drift[run_index] <= -drift_threshold {
            thresholded_signs.push(-1);
        }
    }

    thresholded_signs.iter().all(|sign| *sign > 0)
}

fn persistence_mask(values: &[bool], persistence_runs: usize) -> Vec<bool> {
    let mut out = Vec::with_capacity(values.len());
    let mut consecutive = 0usize;
    for value in values {
        if *value {
            consecutive += 1;
            out.push(consecutive >= persistence_runs);
        } else {
            consecutive = 0;
            out.push(false);
        }
    }
    out
}

fn build_run_signals(traces: &[DsaFeatureTrace], run_count: usize) -> DsaRunSignals {
    let mut any_feature_dsa_alert = Vec::with_capacity(run_count);
    let mut feature_count_dsa_alert = Vec::with_capacity(run_count);

    for run_index in 0..run_count {
        let count = traces
            .iter()
            .filter(|trace| trace.dsa_alert[run_index])
            .count();
        feature_count_dsa_alert.push(count);
        any_feature_dsa_alert.push(count > 0);
    }

    DsaRunSignals {
        primary_run_signal: "any_feature_dsa_alert(k)".into(),
        any_feature_dsa_alert,
        feature_count_dsa_alert,
    }
}

fn compute_episode_summary(
    primary_signal_name: &str,
    dsa_signal: &[bool],
    raw_boundary_episode_count: usize,
    raw_violation_signal: &[bool],
) -> DsaEpisodeSummary {
    let dsa_episodes = episode_ranges(dsa_signal);
    let dsa_lengths = dsa_episodes
        .iter()
        .map(|(start, end)| end - start + 1)
        .collect::<Vec<_>>();
    let non_escalating_dsa_episode_fraction = if dsa_episodes.is_empty() {
        None
    } else {
        Some(
            dsa_episodes
                .iter()
                .filter(|(start, end)| !(*start..=*end).any(|run| raw_violation_signal[run]))
                .count() as f64
                / dsa_episodes.len() as f64,
        )
    };

    DsaEpisodeSummary {
        primary_signal: primary_signal_name.into(),
        raw_boundary_episode_count,
        dsa_episode_count: dsa_episodes.len(),
        mean_dsa_episode_length_runs: mean_usize(&dsa_lengths),
        max_dsa_episode_length_runs: dsa_lengths.iter().copied().max().unwrap_or(0),
        compression_ratio: if dsa_episodes.is_empty() {
            None
        } else {
            Some(raw_boundary_episode_count as f64 / dsa_episodes.len() as f64)
        },
        non_escalating_dsa_episode_fraction,
    }
}

fn episode_ranges(signal: &[bool]) -> Vec<(usize, usize)> {
    let mut episodes = Vec::new();
    let mut current_start: Option<usize> = None;

    for (run_index, flag) in signal.iter().copied().enumerate() {
        match (current_start, flag) {
            (None, true) => current_start = Some(run_index),
            (Some(start), false) => {
                episodes.push((start, run_index - 1));
                current_start = None;
            }
            _ => {}
        }
    }

    if let Some(start) = current_start {
        episodes.push((start, signal.len().saturating_sub(1)));
    }

    episodes
}

#[derive(Debug, Clone)]
struct EarliestDsaSignal {
    run_index: usize,
    feature_index: usize,
    feature_name: String,
    score: f64,
}

fn earliest_dsa_signal(
    traces: &[DsaFeatureTrace],
    primary_signal: &[bool],
    start: usize,
    end: usize,
) -> Option<EarliestDsaSignal> {
    let run_index = earliest_run_signal(primary_signal, start, end)?;
    traces
        .iter()
        .filter(|trace| trace.dsa_alert[run_index])
        .map(|trace| EarliestDsaSignal {
            run_index,
            feature_index: trace.feature_index,
            feature_name: trace.feature_name.clone(),
            score: trace.dsa_score[run_index],
        })
        .max_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.feature_index.cmp(&left.feature_index))
        })
}

#[derive(Debug, Clone)]
struct MaxDsaScore {
    feature_index: usize,
    feature_name: String,
    score: f64,
}

fn max_dsa_score(traces: &[DsaFeatureTrace], start: usize, end: usize) -> Option<MaxDsaScore> {
    let mut max_score: Option<MaxDsaScore> = None;
    for trace in traces {
        for run_index in start..end {
            let candidate = MaxDsaScore {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                score: trace.dsa_score[run_index],
            };
            let should_replace = match &max_score {
                None => true,
                Some(current) => {
                    candidate.score > current.score
                        || (candidate.score == current.score
                            && candidate.feature_index < current.feature_index)
                }
            };
            if should_replace {
                max_score = Some(candidate);
            }
        }
    }
    max_score
}

fn earliest_run_signal(signal: &[bool], start: usize, end: usize) -> Option<usize> {
    (start..end).find(|&run_index| signal[run_index])
}

fn paired_delta(left: Option<usize>, right: Option<usize>) -> Option<i64> {
    Some(left? as i64 - right? as i64)
}

fn baseline_row(
    signal: &str,
    failure_runs: usize,
    lead_values: &[Option<usize>],
    nuisance: f64,
) -> SignalComparisonRow {
    let recall = lead_values.iter().filter(|value| value.is_some()).count();
    SignalComparisonRow {
        signal: signal.into(),
        failure_run_recall: recall,
        failure_runs,
        failure_run_recall_rate: rate(recall, failure_runs),
        mean_lead_time_runs: mean_option_usize(lead_values),
        median_lead_time_runs: median_option_usize(lead_values),
        pass_run_nuisance_proxy: nuisance,
        mean_lead_delta_vs_threshold_runs: None,
        mean_lead_delta_vs_ewma_runs: None,
    }
}

fn validation_failures(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    raw_boundary_nuisance_proxy: f64,
    threshold_recall_gate_passed: bool,
    boundary_nuisance_gate_passed: bool,
    any_metric_improved: bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    if !threshold_recall_gate_passed {
        failures.push(format!(
            "failure recall {}/{} is below threshold recall {}/{}",
            dsa.failure_run_recall,
            dsa.failure_runs,
            threshold.failure_run_recall,
            threshold.failure_runs,
        ));
    }
    if !boundary_nuisance_gate_passed {
        failures.push(format!(
            "pass-run nuisance {:.4} is not below raw DSFB boundary nuisance {:.4}",
            dsa.pass_run_nuisance_proxy, raw_boundary_nuisance_proxy,
        ));
    }
    if !any_metric_improved {
        failures.push(
            "no saved DSA metric improves nuisance, lead time, recall, or compression relative to the logged baselines"
                .into(),
        );
    }
    failures
}

fn dsa_conclusion(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    ewma: &SignalComparisonRow,
    raw_boundary: &SignalComparisonRow,
    episode_summary: &DsaEpisodeSummary,
    nuisance_improved: bool,
    lead_time_improved: bool,
    recall_preserved: bool,
    compression_improved: bool,
    nothing_improved: bool,
    validation_failures: &[String],
) -> String {
    if !validation_failures.is_empty() {
        if nuisance_improved && !lead_time_improved {
            return format!(
                "DSA reduces nuisance relative to at least one baseline, but does not improve lead time and fails validation gates: {}. Recall is {}/{}, threshold recall is {}/{}, EWMA recall is {}/{}, raw-boundary nuisance delta is {:.4}, and compression ratio is {}. No superiority claim is made.",
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                threshold.failure_run_recall,
                threshold.failure_runs,
                ewma.failure_run_recall,
                ewma.failure_runs,
                dsa.pass_run_nuisance_proxy - raw_boundary.pass_run_nuisance_proxy,
                format_option_f64(episode_summary.compression_ratio),
            );
        }

        if nothing_improved {
            return format!(
                "DSA fails to improve nuisance, lead time, recall, or compression and fails validation gates: {}. Recall is {}/{}, pass-run nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}, and compression ratio is {}.",
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
                format_option_f64(episode_summary.compression_ratio),
            );
        }

        return format!(
            "DSA shows mixed trade-offs but fails validation gates: {}. Nuisance improved: {}, lead time improved: {}, recall preserved: {}, compression improved: {}. No superiority claim is made.",
            validation_failures.join("; "),
            nuisance_improved,
            lead_time_improved,
            recall_preserved,
            compression_improved,
        );
    }

    if lead_time_improved && nuisance_improved && recall_preserved {
        return format!(
            "The saved DSA metrics show a qualified improvement: recall is preserved, pass-run nuisance is lower, and mean lead time is higher than at least one scalar baseline. Compression ratio is {}.",
            format_option_f64(episode_summary.compression_ratio),
        );
    }

    if nuisance_improved && !lead_time_improved {
        return format!(
            "DSA reduces nuisance but does not improve lead time. Recall is {}/{}, mean lead deltas are threshold={} and EWMA={}, and compression ratio is {}. No superiority claim is made.",
            dsa.failure_run_recall,
            dsa.failure_runs,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            format_option_f64(episode_summary.compression_ratio),
        );
    }

    if nothing_improved {
        return format!(
            "DSA fails to improve nuisance, lead time, recall, or compression relative to the logged baselines. Recall is {}/{}, pass-run nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}, and compression ratio is {}.",
            dsa.failure_run_recall,
            dsa.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            format_option_f64(episode_summary.compression_ratio),
        );
    }

    format!(
        "DSA shows mixed trade-offs without a clean superiority result. Recall preserved: {}, nuisance improved: {}, lead time improved: {}, and compression ratio is {}.",
        recall_preserved,
        nuisance_improved,
        lead_time_improved,
        format_option_f64(episode_summary.compression_ratio),
    )
}

fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn mean_usize(values: &[usize]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<usize>() as f64 / values.len() as f64)
}

fn mean_option_usize(values: &[Option<usize>]) -> Option<f64> {
    let present = values.iter().flatten().copied().collect::<Vec<_>>();
    mean_usize(&present)
}

fn mean_option_i64(values: &[Option<i64>]) -> Option<f64> {
    let present = values.iter().flatten().copied().collect::<Vec<_>>();
    (!present.is_empty()).then_some(present.iter().sum::<i64>() as f64 / present.len() as f64)
}

fn median_option_usize(values: &[Option<usize>]) -> Option<f64> {
    let mut present = values.iter().flatten().copied().collect::<Vec<_>>();
    if present.is_empty() {
        return None;
    }
    present.sort_unstable();
    let middle = present.len() / 2;
    if present.len() % 2 == 1 {
        Some(present[middle] as f64)
    } else {
        Some((present[middle - 1] + present[middle]) as f64 / 2.0)
    }
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsa_persistence_gating_requires_consecutive_hits() {
        let alert = persistence_mask(&[false, true, true, false, true, true, true], 2);
        assert_eq!(alert, vec![false, false, true, false, false, true, true]);
    }

    #[test]
    fn dsa_consistency_rejects_any_inward_flip() {
        assert!(window_is_consistent(&[0.0, 0.2, 0.1, 0.0], 0.15, 0, 3));
        assert!(!window_is_consistent(&[0.0, 0.2, -0.1, 0.3], 0.15, 0, 3));
        assert!(!window_is_consistent(&[-0.2, -0.3, -0.1], 0.15, 0, 2));
    }

    #[test]
    fn episode_ranges_are_computed_deterministically() {
        assert_eq!(
            episode_ranges(&[false, true, true, false, true, false]),
            vec![(1, 2), (4, 4)]
        );
    }

    #[test]
    fn bounded_dsa_grid_matches_requested_size() {
        let grid = DsaCalibrationGrid::bounded_default();
        assert_eq!(grid.grid_point_count(), 27);
    }

    #[test]
    fn ewma_normalization_is_clipped() {
        assert_eq!(normalize_to_threshold(5.0, 2.0), 1.0);
        assert_eq!(normalize_to_threshold(1.0, 2.0), 0.5);
        assert_eq!(normalize_to_threshold(1.0, 0.0), 0.0);
    }
}
