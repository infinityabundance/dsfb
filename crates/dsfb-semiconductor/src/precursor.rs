use crate::baselines::BaselineSet;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarReason, GrammarSet};
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

#[derive(Debug, Clone, Serialize)]
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
pub struct DsaFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
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
pub struct DsaSignalSummary {
    pub config: DsaConfig,
    pub weights: DsaWeights,
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
    pub dsfb_boundary_nuisance_proxy: f64,
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
    pub dsa: SignalComparisonRow,
    pub threshold: SignalComparisonRow,
    pub ewma: SignalComparisonRow,
    pub dsfb_persistent_boundary_nuisance_proxy: f64,
    pub improvement_vs_threshold: bool,
    pub improvement_vs_ewma: bool,
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
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub any_metric_improved: bool,
    pub validation_passed: bool,
}

pub fn evaluate_dsa(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    config: &DsaConfig,
    boundary_fraction_of_rho: f64,
    pre_failure_lookback_runs: usize,
) -> Result<DsaEvaluation> {
    config.validate()?;
    let weights = DsaWeights::default();
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
            .map(|norm| *norm >= boundary_fraction_of_rho * feature.rho && *norm < feature.rho)
            .collect::<Vec<_>>();
        let slew_flags = sign_trace
            .slew
            .iter()
            .map(|slew| slew.abs() >= sign_trace.slew_threshold)
            .collect::<Vec<_>>();
        let motif_flags = grammar_trace
            .raw_reasons
            .iter()
            .map(|reason| {
                matches!(
                    reason,
                    GrammarReason::SustainedOutwardDrift
                        | GrammarReason::AbruptSlewViolation
                        | GrammarReason::RecurrentBoundaryGrazing
                )
            })
            .collect::<Vec<_>>();
        let ewma_normalized = ewma_trace
            .ewma
            .iter()
            .map(|value| normalize_to_threshold(*value, ewma_trace.threshold))
            .collect::<Vec<_>>();

        let boundary_prefix = bool_prefix_sum(&boundary_flags);
        let slew_prefix = bool_prefix_sum(&slew_flags);
        let motif_prefix = bool_prefix_sum(&motif_flags);

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
            let drift_persistence = longest_outward_drift_streak(
                &sign_trace.drift,
                sign_trace.drift_threshold,
                start,
                run_index,
            ) as f64
                / window_len;
            let slew_density = window_fraction(&slew_prefix, start, run_index, window_len);
            let ewma_occupancy = window_mean(&ewma_normalized, start, run_index);
            let motif_recurrence = window_fraction(&motif_prefix, start, run_index, window_len);
            let consistent_window = window_is_consistent(&sign_trace.drift, start, run_index);
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
            let earliest_dsa = earliest_dsa_signal(&traces, window_start, failure_index);
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
                earliest_dsa_feature_index: earliest_dsa
                    .as_ref()
                    .map(|signal| signal.feature_index),
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
    let alert_run_count = (0..run_count)
        .filter(|&run_index| traces.iter().any(|trace| trace.dsa_alert[run_index]))
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
                .filter(|&&run_index| traces.iter().any(|trace| trace.dsa_alert[run_index]))
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
    let dsfb_boundary_nuisance_proxy = rate(
        pass_indices
            .iter()
            .filter(|&&run_index| {
                grammar
                    .traces
                    .iter()
                    .any(|trace| trace.persistent_boundary[run_index])
            })
            .count(),
        pass_indices.len(),
    );

    let improvement_vs_threshold = qualifies_as_improvement(&dsa_row, &threshold_row, true);
    let improvement_vs_ewma = qualifies_as_improvement(&dsa_row, &ewma_row, false);
    let threshold_recall_gate_passed =
        dsa_row.failure_run_recall >= threshold_row.failure_run_recall;
    let boundary_nuisance_gate_passed =
        dsa_row.pass_run_nuisance_proxy < dsfb_boundary_nuisance_proxy;
    let any_metric_improved = improves_any_metric(
        &dsa_row,
        &threshold_row,
        &ewma_row,
        dsfb_boundary_nuisance_proxy,
    );
    let validation_failures = validation_failures(
        &dsa_row,
        &threshold_row,
        dsfb_boundary_nuisance_proxy,
        threshold_recall_gate_passed,
        boundary_nuisance_gate_passed,
        any_metric_improved,
    );
    let validation_passed = validation_failures.is_empty();
    let conclusion = dsa_conclusion(
        &dsa_row,
        &threshold_row,
        &ewma_row,
        improvement_vs_threshold,
        improvement_vs_ewma,
        &validation_failures,
        any_metric_improved,
    );

    Ok(DsaEvaluation {
        traces,
        summary: DsaSignalSummary {
            config: config.clone(),
            weights: weights.clone(),
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
            dsfb_boundary_nuisance_proxy,
            threshold_recall_gate_passed,
            boundary_nuisance_gate_passed,
            any_metric_improved,
            validation_passed,
            validation_failures: validation_failures.clone(),
        },
        comparison_summary: DsaVsBaselinesSummary {
            dataset: "SECOM".into(),
            dsa: dsa_row,
            threshold: threshold_row,
            ewma: ewma_row,
            dsfb_persistent_boundary_nuisance_proxy: dsfb_boundary_nuisance_proxy,
            improvement_vs_threshold,
            improvement_vs_ewma,
            threshold_recall_gate_passed,
            boundary_nuisance_gate_passed,
            any_metric_improved,
            validation_passed,
            validation_failures,
            conclusion,
        },
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
    boundary_fraction_of_rho: f64,
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
            boundary_fraction_of_rho,
            pre_failure_lookback_runs,
        )?;
        rows.push(DsaCalibrationRow {
            config_id,
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
            threshold_recall_gate_passed: evaluation.summary.threshold_recall_gate_passed,
            boundary_nuisance_gate_passed: evaluation.summary.boundary_nuisance_gate_passed,
            any_metric_improved: evaluation.summary.any_metric_improved,
            validation_passed: evaluation.summary.validation_passed,
        });
    }

    Ok(rows)
}

fn empty_trace(feature_index: usize, feature_name: &str, run_count: usize) -> DsaFeatureTrace {
    DsaFeatureTrace {
        feature_index,
        feature_name: feature_name.into(),
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
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn longest_outward_drift_streak(
    drift: &[f64],
    drift_threshold: f64,
    start: usize,
    end: usize,
) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for run_index in start..=end {
        if drift[run_index] >= drift_threshold {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn window_is_consistent(drift: &[f64], start: usize, end: usize) -> bool {
    let mut observed_sign = 0i8;
    let mut previous_nonzero = 0i8;
    for run_index in start..=end {
        let sign = drift_sign(drift[run_index]);
        if sign == 0 {
            continue;
        }
        if observed_sign == 0 {
            observed_sign = sign;
        } else if sign != observed_sign {
            return false;
        }
        if previous_nonzero != 0 && sign != previous_nonzero {
            return false;
        }
        previous_nonzero = sign;
    }
    true
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
struct EarliestDsaSignal {
    run_index: usize,
    feature_index: usize,
    feature_name: String,
    score: f64,
}

fn earliest_dsa_signal(
    traces: &[DsaFeatureTrace],
    start: usize,
    end: usize,
) -> Option<EarliestDsaSignal> {
    let mut earliest: Option<EarliestDsaSignal> = None;

    for trace in traces {
        for run_index in start..end {
            if !trace.dsa_alert[run_index] {
                continue;
            }
            let candidate = EarliestDsaSignal {
                run_index,
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                score: trace.dsa_score[run_index],
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
    dsa: &SignalComparisonRow,
    baseline: &SignalComparisonRow,
    against_threshold: bool,
) -> bool {
    let lead_delta = if against_threshold {
        dsa.mean_lead_delta_vs_threshold_runs
    } else {
        dsa.mean_lead_delta_vs_ewma_runs
    };

    matches!(lead_delta, Some(delta) if delta > 0.0)
        && dsa.failure_run_recall >= baseline.failure_run_recall
        && dsa.pass_run_nuisance_proxy <= baseline.pass_run_nuisance_proxy
}

fn improves_any_metric(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    ewma: &SignalComparisonRow,
    dsfb_boundary_nuisance_proxy: f64,
) -> bool {
    matches!(dsa.mean_lead_delta_vs_threshold_runs, Some(delta) if delta > 0.0)
        || matches!(dsa.mean_lead_delta_vs_ewma_runs, Some(delta) if delta > 0.0)
        || option_greater(dsa.median_lead_time_runs, threshold.median_lead_time_runs)
        || option_greater(dsa.median_lead_time_runs, ewma.median_lead_time_runs)
        || dsa.failure_run_recall > threshold.failure_run_recall
        || dsa.failure_run_recall > ewma.failure_run_recall
        || dsa.pass_run_nuisance_proxy < threshold.pass_run_nuisance_proxy
        || dsa.pass_run_nuisance_proxy < ewma.pass_run_nuisance_proxy
        || dsa.pass_run_nuisance_proxy < dsfb_boundary_nuisance_proxy
}

fn validation_failures(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    dsfb_boundary_nuisance_proxy: f64,
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
            "pass-run nuisance {:.4} is not below persistent DSFB boundary nuisance {:.4}",
            dsa.pass_run_nuisance_proxy, dsfb_boundary_nuisance_proxy,
        ));
    }
    if !any_metric_improved {
        failures.push(
            "no saved DSA metric improves relative to threshold, EWMA, or persistent DSFB boundary nuisance"
                .into(),
        );
    }
    failures
}

fn dsa_conclusion(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    ewma: &SignalComparisonRow,
    improvement_vs_threshold: bool,
    improvement_vs_ewma: bool,
    validation_failures: &[String],
    any_metric_improved: bool,
) -> String {
    let nuisance_reduction_without_lead_gain =
        (dsa.pass_run_nuisance_proxy < threshold.pass_run_nuisance_proxy
            && !matches!(dsa.mean_lead_delta_vs_threshold_runs, Some(delta) if delta > 0.0))
            || (dsa.pass_run_nuisance_proxy < ewma.pass_run_nuisance_proxy
                && !matches!(dsa.mean_lead_delta_vs_ewma_runs, Some(delta) if delta > 0.0));

    if !validation_failures.is_empty() {
        if nuisance_reduction_without_lead_gain {
            return format!(
                "The saved DSA metrics reduce nuisance relative to at least one comparator, but do not improve lead time and fail validation gates: {}. No superiority claim is made. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            );
        }
        if !any_metric_improved {
            return format!(
                "The saved DSA metrics fail the required validation gates and show no improvement. Validation failures: {}. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            );
        }
        return format!(
            "The saved DSA metrics show mixed trade-offs, but the required validation gates fail: {}. No superiority claim is made. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
            validation_failures.join("; "),
            dsa.failure_run_recall,
            dsa.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
        );
    }

    if improvement_vs_threshold || improvement_vs_ewma {
        let mut wins = Vec::new();
        if improvement_vs_threshold {
            wins.push(format!(
                "threshold (lead delta {}, recall {}/{}, nuisance {:.4} vs {:.4})",
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                threshold.pass_run_nuisance_proxy,
            ));
        }
        if improvement_vs_ewma {
            wins.push(format!(
                "EWMA (lead delta {}, recall {}/{}, nuisance {:.4} vs {:.4})",
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                ewma.pass_run_nuisance_proxy,
            ));
        }
        return format!(
            "The saved DSA metrics show a qualified improvement relative to {} because mean lead is higher while failure-run recall is not lower and pass-run nuisance is not higher.",
            wins.join(" and ")
        );
    }

    if nuisance_reduction_without_lead_gain {
        return format!(
            "The saved DSA metrics reduce nuisance relative to at least one scalar baseline, but do not improve lead time, so no improvement claim is made. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
            dsa.failure_run_recall,
            dsa.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
        );
    }

    if !any_metric_improved {
        return format!(
            "The saved DSA metrics fail to improve lead time, recall, or nuisance relative to threshold or EWMA. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
            dsa.failure_run_recall,
            dsa.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
        );
    }

    format!(
        "The saved DSA metrics show mixed trade-offs but do not satisfy the crate's improvement rule. DSA recall is {}/{}, nuisance is {:.4}, and mean lead deltas are threshold={} and EWMA={}.",
        dsa.failure_run_recall,
        dsa.failure_runs,
        dsa.pass_run_nuisance_proxy,
        format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
    )
}

fn option_greater(left: Option<f64>, right: Option<f64>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left > right)
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
    fn dsa_persistence_gating_requires_consecutive_hits() {
        let alert = persistence_mask(&[false, true, true, false, true, true, true], 2);
        assert_eq!(alert, vec![false, false, true, false, false, true, true]);
    }

    #[test]
    fn dsa_consistency_rejects_sign_flips() {
        assert!(window_is_consistent(&[0.0, 0.2, 0.1, 0.0], 0, 3));
        assert!(!window_is_consistent(&[0.0, 0.2, -0.1, 0.3], 0, 3));
    }

    #[test]
    fn bounded_dsa_grid_matches_requested_size() {
        let grid = DsaCalibrationGrid::bounded_default();
        assert_eq!(grid.grid_point_count(), 27);
    }
}
