use crate::baselines::BaselineSet;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarReason, GrammarSet, GrammarState};
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrecursorConfig {
    pub window: usize,
    pub persistence_runs: usize,
    pub boundary_density_tau: f64,
    pub drift_persistence_tau: f64,
    pub transition_cluster_tau: usize,
    pub ewma_occupancy_tau: f64,
    pub alert_tau: f64,
}

impl Default for PrecursorConfig {
    fn default() -> Self {
        Self {
            window: 10,
            persistence_runs: 2,
            boundary_density_tau: 0.3,
            drift_persistence_tau: 0.3,
            transition_cluster_tau: 2,
            ewma_occupancy_tau: 0.8,
            alert_tau: 2.5,
        }
    }
}

impl PrecursorConfig {
    pub fn validate(&self) -> Result<()> {
        if self.window == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor window must be positive".into(),
            ));
        }
        if self.persistence_runs == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor persistence_runs must be positive".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.boundary_density_tau) {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor boundary_density_tau must be in [0, 1]".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.drift_persistence_tau) {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor drift_persistence_tau must be in [0, 1]".into(),
            ));
        }
        if self.transition_cluster_tau == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor transition_cluster_tau must be positive".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.ewma_occupancy_tau) {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor ewma_occupancy_tau must be in [0, 1]".into(),
            ));
        }
        if self.alert_tau <= 0.0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor alert_tau must be positive".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecursorWeights {
    pub boundary_density: f64,
    pub drift_persistence: f64,
    pub transition_cluster: f64,
    pub ewma_occupancy: f64,
    pub motif_recurrence: f64,
}

impl Default for PrecursorWeights {
    fn default() -> Self {
        Self {
            boundary_density: 1.0,
            drift_persistence: 1.0,
            transition_cluster: 1.0,
            ewma_occupancy: 1.0,
            motif_recurrence: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecursorFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub boundary_density_w: Vec<f64>,
    pub violation_density_w: Vec<f64>,
    pub drift_persistence_w: Vec<f64>,
    pub transition_cluster_w: Vec<f64>,
    pub ewma_occupancy_w: Vec<f64>,
    pub motif_recurrence_w: Vec<f64>,
    pub precursor_score: Vec<f64>,
    pub precursor_active: Vec<bool>,
    pub precursor_alert: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecursorSignalSummary {
    pub config: PrecursorConfig,
    pub weights: PrecursorWeights,
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
pub struct PrecursorVsBaselinesSummary {
    pub dataset: String,
    pub precursor: SignalComparisonRow,
    pub threshold: SignalComparisonRow,
    pub ewma: SignalComparisonRow,
    pub improvement_vs_threshold: bool,
    pub improvement_vs_ewma: bool,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerFailureRunPrecursorSignal {
    pub failure_run_index: usize,
    pub failure_timestamp: String,
    pub earliest_precursor_run: Option<usize>,
    pub earliest_precursor_feature_index: Option<usize>,
    pub earliest_precursor_feature_name: Option<String>,
    pub precursor_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub precursor_minus_threshold_delta_runs: Option<i64>,
    pub precursor_minus_ewma_delta_runs: Option<i64>,
    pub precursor_alerting_feature_count: usize,
    pub max_precursor_score_in_lookback: Option<f64>,
    pub max_precursor_score_feature_index: Option<usize>,
    pub max_precursor_score_feature_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecursorEvaluation {
    pub traces: Vec<PrecursorFeatureTrace>,
    pub summary: PrecursorSignalSummary,
    pub comparison_summary: PrecursorVsBaselinesSummary,
    pub per_failure_run_signals: Vec<PerFailureRunPrecursorSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecursorCalibrationGrid {
    pub window: Vec<usize>,
    pub persistence_runs: Vec<usize>,
    pub boundary_density_tau: Vec<f64>,
    pub drift_persistence_tau: Vec<f64>,
    pub transition_cluster_tau: Vec<usize>,
    pub ewma_occupancy_tau: Vec<f64>,
    pub alert_tau: Vec<f64>,
}

impl PrecursorCalibrationGrid {
    pub fn bounded_default() -> Self {
        Self {
            window: vec![5, 10, 15, 20],
            persistence_runs: vec![2, 3, 4],
            boundary_density_tau: vec![0.2, 0.3, 0.4],
            drift_persistence_tau: vec![0.2, 0.3, 0.4],
            transition_cluster_tau: vec![1, 2, 3],
            ewma_occupancy_tau: vec![0.7, 0.8, 0.9],
            alert_tau: vec![2.0, 2.5, 3.0],
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.grid_point_count() == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "precursor calibration grid must contain at least one point".into(),
            ));
        }
        if self.grid_point_count() > 4096 {
            return Err(DsfbSemiconductorError::DatasetFormat(format!(
                "precursor calibration grid is too large ({})",
                self.grid_point_count()
            )));
        }
        Ok(())
    }

    pub fn grid_point_count(&self) -> usize {
        [
            self.window.len(),
            self.persistence_runs.len(),
            self.boundary_density_tau.len(),
            self.drift_persistence_tau.len(),
            self.transition_cluster_tau.len(),
            self.ewma_occupancy_tau.len(),
            self.alert_tau.len(),
        ]
        .into_iter()
        .product()
    }

    pub fn expand(&self) -> Vec<PrecursorConfig> {
        let mut out = Vec::with_capacity(self.grid_point_count());
        for &window in &self.window {
            for &persistence_runs in &self.persistence_runs {
                for &boundary_density_tau in &self.boundary_density_tau {
                    for &drift_persistence_tau in &self.drift_persistence_tau {
                        for &transition_cluster_tau in &self.transition_cluster_tau {
                            for &ewma_occupancy_tau in &self.ewma_occupancy_tau {
                                for &alert_tau in &self.alert_tau {
                                    out.push(PrecursorConfig {
                                        window,
                                        persistence_runs,
                                        boundary_density_tau,
                                        drift_persistence_tau,
                                        transition_cluster_tau,
                                        ewma_occupancy_tau,
                                        alert_tau,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecursorCalibrationRow {
    pub config_id: usize,
    pub window: usize,
    pub persistence_runs: usize,
    pub boundary_density_tau: f64,
    pub drift_persistence_tau: f64,
    pub transition_cluster_tau: usize,
    pub ewma_occupancy_tau: f64,
    pub alert_tau: f64,
    pub failure_run_recall: usize,
    pub failure_runs: usize,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
}

pub fn evaluate_precursor(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    config: &PrecursorConfig,
    pre_failure_lookback_runs: usize,
) -> Result<PrecursorEvaluation> {
    config.validate()?;
    let weights = PrecursorWeights::default();
    let run_count = dataset.labels.len();
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

        let boundary_flags = residual_trace
            .norms
            .iter()
            .map(|norm| {
                *norm >= config.boundary_density_tau * feature.rho && *norm < feature.rho
            })
            .collect::<Vec<_>>();
        let violation_flags = residual_trace
            .norms
            .iter()
            .map(|norm| *norm >= feature.rho)
            .collect::<Vec<_>>();
        let drift_flags = sign_trace
            .drift
            .iter()
            .map(|drift| *drift >= config.drift_persistence_tau * sign_trace.drift_threshold)
            .collect::<Vec<_>>();
        let transition_flags = grammar_trace
            .states
            .iter()
            .enumerate()
            .map(|(index, state)| {
                index > 0
                    && *state != grammar_trace.states[index - 1]
                    && (*state != GrammarState::Admissible
                        || grammar_trace.states[index - 1] != GrammarState::Admissible)
            })
            .collect::<Vec<_>>();
        let ewma_flags = ewma_trace
            .ewma
            .iter()
            .map(|value| ewma_trace.threshold > 0.0 && *value >= config.ewma_occupancy_tau * ewma_trace.threshold)
            .collect::<Vec<_>>();
        let slow_drift_flags = grammar_trace
            .raw_reasons
            .iter()
            .map(|reason| *reason == GrammarReason::SustainedOutwardDrift)
            .collect::<Vec<_>>();
        let slew_flags = grammar_trace
            .raw_reasons
            .iter()
            .map(|reason| *reason == GrammarReason::AbruptSlewViolation)
            .collect::<Vec<_>>();
        let grazing_flags = grammar_trace
            .raw_reasons
            .iter()
            .map(|reason| *reason == GrammarReason::RecurrentBoundaryGrazing)
            .collect::<Vec<_>>();

        let boundary_prefix = bool_prefix_sum(&boundary_flags);
        let violation_prefix = bool_prefix_sum(&violation_flags);
        let drift_prefix = bool_prefix_sum(&drift_flags);
        let transition_prefix = bool_prefix_sum(&transition_flags);
        let ewma_prefix = bool_prefix_sum(&ewma_flags);
        let slow_drift_prefix = bool_prefix_sum(&slow_drift_flags);
        let slew_prefix = bool_prefix_sum(&slew_flags);
        let grazing_prefix = bool_prefix_sum(&grazing_flags);

        let mut boundary_density_w = Vec::with_capacity(run_count);
        let mut violation_density_w = Vec::with_capacity(run_count);
        let mut drift_persistence_w = Vec::with_capacity(run_count);
        let mut transition_cluster_w = Vec::with_capacity(run_count);
        let mut ewma_occupancy_w = Vec::with_capacity(run_count);
        let mut motif_recurrence_w = Vec::with_capacity(run_count);
        let mut precursor_score = Vec::with_capacity(run_count);
        let mut precursor_active = Vec::with_capacity(run_count);

        for run_index in 0..run_count {
            let start = run_index.saturating_sub(config.window.saturating_sub(1));
            let window_len = (run_index - start + 1) as f64;
            let boundary_density =
                window_fraction(&boundary_prefix, start, run_index, window_len);
            let violation_density =
                window_fraction(&violation_prefix, start, run_index, window_len);
            let drift_persistence = window_fraction(&drift_prefix, start, run_index, window_len);
            let transition_cluster = (window_count(&transition_prefix, start, run_index) as f64
                / config.transition_cluster_tau as f64)
                .min(1.0);
            let ewma_occupancy = window_fraction(&ewma_prefix, start, run_index, window_len);
            let motif_recurrence = [
                window_count(&slow_drift_prefix, start, run_index),
                window_count(&slew_prefix, start, run_index),
                window_count(&grazing_prefix, start, run_index),
            ]
            .into_iter()
            .max()
            .unwrap_or(0) as f64
                / window_len;
            let score = weights.boundary_density * boundary_density
                + weights.drift_persistence * drift_persistence
                + weights.transition_cluster * transition_cluster
                + weights.ewma_occupancy * ewma_occupancy
                + weights.motif_recurrence * motif_recurrence;

            boundary_density_w.push(boundary_density);
            violation_density_w.push(violation_density);
            drift_persistence_w.push(drift_persistence);
            transition_cluster_w.push(transition_cluster);
            ewma_occupancy_w.push(ewma_occupancy);
            motif_recurrence_w.push(motif_recurrence);
            precursor_score.push(score);
            precursor_active.push(score >= config.alert_tau);
        }

        let precursor_alert = persistence_mask(&precursor_active, config.persistence_runs);
        traces.push(PrecursorFeatureTrace {
            feature_index: feature.feature_index,
            feature_name: feature.feature_name.clone(),
            boundary_density_w,
            violation_density_w,
            drift_persistence_w,
            transition_cluster_w,
            ewma_occupancy_w,
            motif_recurrence_w,
            precursor_score,
            precursor_active,
            precursor_alert,
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

    let per_failure_run_signals = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            let earliest_precursor = earliest_precursor_signal(&traces, window_start, failure_index);
            let earliest_threshold_run =
                earliest_baseline_signal(window_start, failure_index, |run_index| {
                    residuals
                        .traces
                        .iter()
                        .any(|trace| trace.threshold_alarm[run_index])
                });
            let earliest_ewma_run =
                earliest_baseline_signal(window_start, failure_index, |run_index| {
                    baselines.ewma.iter().any(|trace| trace.alarm[run_index])
                });
            let precursor_lead_runs =
                earliest_precursor.as_ref().map(|signal| failure_index - signal.run_index);
            let threshold_lead_runs = earliest_threshold_run.map(|index| failure_index - index);
            let ewma_lead_runs = earliest_ewma_run.map(|index| failure_index - index);
            let alerting_feature_count = traces
                .iter()
                .filter(|trace| trace.precursor_alert[window_start..failure_index].iter().any(|flag| *flag))
                .count();
            let max_score = max_precursor_score(&traces, window_start, failure_index);

            PerFailureRunPrecursorSignal {
                failure_run_index: failure_index,
                failure_timestamp: dataset.timestamps[failure_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                earliest_precursor_run: earliest_precursor.as_ref().map(|signal| signal.run_index),
                earliest_precursor_feature_index: earliest_precursor
                    .as_ref()
                    .map(|signal| signal.feature_index),
                earliest_precursor_feature_name: earliest_precursor
                    .as_ref()
                    .map(|signal| signal.feature_name.clone()),
                precursor_lead_runs,
                threshold_lead_runs,
                ewma_lead_runs,
                precursor_minus_threshold_delta_runs: paired_delta(
                    precursor_lead_runs,
                    threshold_lead_runs,
                ),
                precursor_minus_ewma_delta_runs: paired_delta(
                    precursor_lead_runs,
                    ewma_lead_runs,
                ),
                precursor_alerting_feature_count: alerting_feature_count,
                max_precursor_score_in_lookback: max_score.as_ref().map(|score| score.score),
                max_precursor_score_feature_index: max_score.as_ref().map(|score| score.feature_index),
                max_precursor_score_feature_name: max_score.as_ref().map(|score| score.feature_name.clone()),
            }
        })
        .collect::<Vec<_>>();

    let alert_point_count = traces
        .iter()
        .map(|trace| trace.precursor_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let alert_run_count = (0..run_count)
        .filter(|&run_index| traces.iter().any(|trace| trace.precursor_alert[run_index]))
        .count();
    let failure_run_recall = per_failure_run_signals
        .iter()
        .filter(|signal| signal.earliest_precursor_run.is_some())
        .count();
    let precursor_row = SignalComparisonRow {
        signal: "DSFB precursor".into(),
        failure_run_recall,
        failure_runs: failure_indices.len(),
        failure_run_recall_rate: rate(failure_run_recall, failure_indices.len()),
        mean_lead_time_runs: mean_option_usize(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.precursor_lead_runs)
                .collect::<Vec<_>>(),
        ),
        median_lead_time_runs: median_option_usize(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.precursor_lead_runs)
                .collect::<Vec<_>>(),
        ),
        pass_run_nuisance_proxy: rate(
            pass_indices
                .iter()
                .filter(|&&run_index| traces.iter().any(|trace| trace.precursor_alert[run_index]))
                .count(),
            pass_indices.len(),
        ),
        mean_lead_delta_vs_threshold_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.precursor_minus_threshold_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_lead_delta_vs_ewma_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.precursor_minus_ewma_delta_runs)
                .collect::<Vec<_>>(),
        ),
    };
    let threshold_row = baseline_row(
        "Threshold",
        failure_indices.len(),
        &per_failure_run_signals
            .iter()
            .map(|signal| signal.threshold_lead_runs)
            .collect::<Vec<_>>(),
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| {
                    residuals
                        .traces
                        .iter()
                        .any(|trace| trace.threshold_alarm[run_index])
                })
                .count(),
            pass_indices.len(),
        ),
    );
    let ewma_row = baseline_row(
        "EWMA",
        failure_indices.len(),
        &per_failure_run_signals
            .iter()
            .map(|signal| signal.ewma_lead_runs)
            .collect::<Vec<_>>(),
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| baselines.ewma.iter().any(|trace| trace.alarm[run_index]))
                .count(),
            pass_indices.len(),
        ),
    );
    let improvement_vs_threshold = qualifies_as_improvement(&precursor_row, &threshold_row, true);
    let improvement_vs_ewma = qualifies_as_improvement(&precursor_row, &ewma_row, false);
    let conclusion = precursor_conclusion(
        &precursor_row,
        &threshold_row,
        &ewma_row,
        improvement_vs_threshold,
        improvement_vs_ewma,
    );

    Ok(PrecursorEvaluation {
        traces,
        summary: PrecursorSignalSummary {
            config: config.clone(),
            weights: weights.clone(),
            analyzable_feature_count: nominal.features.iter().filter(|feature| feature.analyzable).count(),
            alert_point_count,
            alert_run_count,
            failure_runs: failure_indices.len(),
            failure_run_recall,
            failure_run_recall_rate: precursor_row.failure_run_recall_rate,
            mean_lead_time_runs: precursor_row.mean_lead_time_runs,
            median_lead_time_runs: precursor_row.median_lead_time_runs,
            pass_run_nuisance_proxy: precursor_row.pass_run_nuisance_proxy,
            mean_lead_delta_vs_threshold_runs: precursor_row.mean_lead_delta_vs_threshold_runs,
            mean_lead_delta_vs_ewma_runs: precursor_row.mean_lead_delta_vs_ewma_runs,
        },
        comparison_summary: PrecursorVsBaselinesSummary {
            dataset: "SECOM".into(),
            precursor: precursor_row,
            threshold: threshold_row,
            ewma: ewma_row,
            improvement_vs_threshold,
            improvement_vs_ewma,
            conclusion,
        },
        per_failure_run_signals,
    })
}

pub fn run_precursor_calibration_grid(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    grid: &PrecursorCalibrationGrid,
    pre_failure_lookback_runs: usize,
) -> Result<Vec<PrecursorCalibrationRow>> {
    grid.validate()?;

    let mut rows = Vec::with_capacity(grid.grid_point_count());
    for (config_id, config) in grid.expand().into_iter().enumerate() {
        let evaluation = evaluate_precursor(
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            &config,
            pre_failure_lookback_runs,
        )?;
        rows.push(PrecursorCalibrationRow {
            config_id,
            window: config.window,
            persistence_runs: config.persistence_runs,
            boundary_density_tau: config.boundary_density_tau,
            drift_persistence_tau: config.drift_persistence_tau,
            transition_cluster_tau: config.transition_cluster_tau,
            ewma_occupancy_tau: config.ewma_occupancy_tau,
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
        });
    }

    Ok(rows)
}

fn empty_trace(feature_index: usize, feature_name: &str, run_count: usize) -> PrecursorFeatureTrace {
    PrecursorFeatureTrace {
        feature_index,
        feature_name: feature_name.into(),
        boundary_density_w: vec![0.0; run_count],
        violation_density_w: vec![0.0; run_count],
        drift_persistence_w: vec![0.0; run_count],
        transition_cluster_w: vec![0.0; run_count],
        ewma_occupancy_w: vec![0.0; run_count],
        motif_recurrence_w: vec![0.0; run_count],
        precursor_score: vec![0.0; run_count],
        precursor_active: vec![false; run_count],
        precursor_alert: vec![false; run_count],
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

#[derive(Debug, Clone)]
struct EarliestPrecursorSignal {
    run_index: usize,
    feature_index: usize,
    feature_name: String,
    score: f64,
}

fn earliest_precursor_signal(
    traces: &[PrecursorFeatureTrace],
    start: usize,
    end: usize,
) -> Option<EarliestPrecursorSignal> {
    let mut earliest: Option<EarliestPrecursorSignal> = None;

    for trace in traces {
        for run_index in start..end {
            if !trace.precursor_alert[run_index] {
                continue;
            }
            let candidate = EarliestPrecursorSignal {
                run_index,
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                score: trace.precursor_score[run_index],
            };
            let should_replace = match &earliest {
                None => true,
                Some(current) => {
                    candidate.run_index < current.run_index
                        || (candidate.run_index == current.run_index
                            && candidate.score > current.score)
                        || (candidate.run_index == current.run_index
                            && candidate.score == current.score
                            && candidate.feature_index < current.feature_index)
                }
            };
            if should_replace {
                earliest = Some(candidate);
            }
            break;
        }
    }

    earliest
}

#[derive(Debug, Clone)]
struct MaxPrecursorScore {
    feature_index: usize,
    feature_name: String,
    score: f64,
}

fn max_precursor_score(
    traces: &[PrecursorFeatureTrace],
    start: usize,
    end: usize,
) -> Option<MaxPrecursorScore> {
    let mut max_score: Option<MaxPrecursorScore> = None;
    for trace in traces {
        for run_index in start..end {
            let candidate = MaxPrecursorScore {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                score: trace.precursor_score[run_index],
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

fn earliest_baseline_signal<F>(start: usize, end: usize, predicate: F) -> Option<usize>
where
    F: Fn(usize) -> bool,
{
    (start..end).find(|&run_index| predicate(run_index))
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

fn qualifies_as_improvement(
    precursor: &SignalComparisonRow,
    baseline: &SignalComparisonRow,
    against_threshold: bool,
) -> bool {
    let lead_delta = if against_threshold {
        precursor.mean_lead_delta_vs_threshold_runs
    } else {
        precursor.mean_lead_delta_vs_ewma_runs
    };

    matches!(lead_delta, Some(delta) if delta > 0.0)
        && precursor.failure_run_recall >= baseline.failure_run_recall
        && precursor.pass_run_nuisance_proxy <= baseline.pass_run_nuisance_proxy
}

fn precursor_conclusion(
    precursor: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    ewma: &SignalComparisonRow,
    improvement_vs_threshold: bool,
    improvement_vs_ewma: bool,
) -> String {
    if improvement_vs_threshold || improvement_vs_ewma {
        let mut wins = Vec::new();
        if improvement_vs_threshold {
            wins.push(format!(
                "threshold (lead delta {:.4}, recall {}/{}, nuisance {:.4} vs {:.4})",
                precursor.mean_lead_delta_vs_threshold_runs.unwrap_or(0.0),
                precursor.failure_run_recall,
                precursor.failure_runs,
                precursor.pass_run_nuisance_proxy,
                threshold.pass_run_nuisance_proxy,
            ));
        }
        if improvement_vs_ewma {
            wins.push(format!(
                "EWMA (lead delta {:.4}, recall {}/{}, nuisance {:.4} vs {:.4})",
                precursor.mean_lead_delta_vs_ewma_runs.unwrap_or(0.0),
                precursor.failure_run_recall,
                precursor.failure_runs,
                precursor.pass_run_nuisance_proxy,
                ewma.pass_run_nuisance_proxy,
            ));
        }
        format!(
            "The saved precursor metrics show a qualified improvement relative to {} because mean lead is higher while failure-run recall is not lower and pass-run nuisance is not higher.",
            wins.join(" and ")
        )
    } else {
        format!(
            "The saved precursor metrics do not show a clear improvement over threshold or EWMA. Precursor recall is {}/{}, nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}.",
            precursor.failure_run_recall,
            precursor.failure_runs,
            precursor.pass_run_nuisance_proxy,
            format_option_f64(precursor.mean_lead_delta_vs_threshold_runs),
            format_option_f64(precursor.mean_lead_delta_vs_ewma_runs),
        )
    }
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
    fn precursor_persistence_gating_requires_consecutive_hits() {
        let alert = persistence_mask(&[false, true, true, false, true, true, true], 2);
        assert_eq!(alert, vec![false, false, true, false, false, true, true]);
    }

    #[test]
    fn bounded_precursor_grid_matches_requested_size() {
        let grid = PrecursorCalibrationGrid::bounded_default();
        assert_eq!(grid.grid_point_count(), 2916);
    }
}
