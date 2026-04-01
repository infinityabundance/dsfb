use crate::baselines::BaselineSet;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarReason, GrammarSet, GrammarState};
use crate::heuristics::{
    dsa_contributing_motif_names, heuristic_policy_definition, FeaturePolicyOverride,
    HeuristicAlertClass, HeuristicPolicyDefinition,
};
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsaConfig {
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub corroborating_feature_count_min: usize,
}

impl Default for DsaConfig {
    fn default() -> Self {
        Self {
            window: 5,
            persistence_runs: 2,
            alert_tau: 2.0,
            corroborating_feature_count_min: 2,
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
        if self.corroborating_feature_count_min == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "dsa corroborating_feature_count_min must be positive".into(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecallRescueConfig {
    pub enabled: bool,
    pub priority_one_score_margin: f64,
    pub priority_two_score_margin: f64,
    pub minimum_ewma_occupancy: f64,
    pub minimum_boundary_density: f64,
    pub minimum_motif_recurrence: f64,
}

impl Default for RecallRescueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            priority_one_score_margin: 0.10,
            priority_two_score_margin: 0.40,
            minimum_ewma_occupancy: 0.65,
            minimum_boundary_density: 0.40,
            minimum_motif_recurrence: 0.40,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DsaPolicyRuntime {
    pub feature_policy_overrides: Vec<FeaturePolicyOverride>,
    pub recall_rescue: RecallRescueConfig,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DsaPolicyState {
    Silent,
    Watch,
    Review,
    Escalate,
}

impl DsaPolicyState {
    fn is_review_or_escalate(self) -> bool {
        matches!(self, Self::Review | Self::Escalate)
    }

    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Watch => "watch",
            Self::Review => "review",
            Self::Escalate => "escalate",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMotifPolicyContribution {
    pub motif_name: String,
    pub alert_class_default: HeuristicAlertClass,
    pub watch_points: usize,
    pub review_points: usize,
    pub escalate_points: usize,
    pub silent_suppression_points: usize,
    pub pass_review_or_escalate_points: usize,
    pub pre_failure_review_or_escalate_points: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaMotifPolicyContribution {
    pub motif_name: String,
    pub alert_class_default: HeuristicAlertClass,
    pub watch_points: usize,
    pub review_points: usize,
    pub escalate_points: usize,
    pub silent_suppression_points: usize,
    pub pass_review_or_escalate_points: usize,
    pub pre_failure_review_or_escalate_points: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaParameterManifest {
    pub config: DsaConfig,
    pub weights: DsaWeights,
    pub feature_policy_override_count: usize,
    pub feature_policy_override_summary: Vec<String>,
    pub policy_engine_definition: String,
    pub feature_level_state_definition: String,
    pub primary_run_signal: String,
    pub primary_run_signal_definition: String,
    pub secondary_run_signal: String,
    pub tertiary_run_signal: String,
    pub strict_escalate_signal: String,
    pub rolling_window_definition: String,
    pub boundary_density_basis: String,
    pub drift_persistence_definition: String,
    pub slew_density_definition: String,
    pub ewma_occupancy_formula: String,
    pub motif_names_used_for_recurrence: Vec<String>,
    pub directional_consistency_rule: String,
    pub silence_rule: String,
    pub corroboration_rule: String,
    pub recall_rescue_definition: String,
    pub recall_tolerance_runs_for_primary_success: usize,
    pub primary_success_condition_definition: String,
    pub optimization_priority_order: Vec<String>,
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
    pub fragmentation_proxy_w: Vec<f64>,
    pub consistent: Vec<bool>,
    pub dsa_score: Vec<f64>,
    pub dsa_active: Vec<bool>,
    pub numeric_dsa_alert: Vec<bool>,
    pub dsa_alert: Vec<bool>,
    pub resolved_alert_class: Vec<HeuristicAlertClass>,
    pub policy_state: Vec<DsaPolicyState>,
    pub policy_suppressed_to_silent: Vec<bool>,
    pub rescue_transition: Vec<String>,
    pub rescued_to_review: Vec<bool>,
    pub motif_policy_contributions: Vec<FeatureMotifPolicyContribution>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaRunSignals {
    pub primary_run_signal: String,
    pub corroborating_feature_count_min: usize,
    pub primary_run_alert: Vec<bool>,
    pub any_feature_dsa_alert: Vec<bool>,
    pub any_feature_raw_violation: Vec<bool>,
    pub feature_count_dsa_alert: Vec<usize>,
    pub watch_feature_count: Vec<usize>,
    pub review_feature_count: Vec<usize>,
    pub escalate_feature_count: Vec<usize>,
    pub strict_escalate_run_alert: Vec<bool>,
    pub numeric_primary_run_alert: Vec<bool>,
    pub numeric_feature_count_dsa_alert: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaEpisodeSummary {
    pub primary_signal: String,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub dsa_episodes_preceding_failure: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub precursor_quality: Option<f64>,
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
    pub numeric_alert_point_count: usize,
    pub numeric_alert_run_count: usize,
    pub watch_point_count: usize,
    pub review_point_count: usize,
    pub escalate_point_count: usize,
    pub silenced_point_count: usize,
    pub rescued_point_count: usize,
    pub rescued_watch_to_review_points: usize,
    pub rescued_review_to_escalate_points: usize,
    pub failure_runs: usize,
    pub failure_run_recall: usize,
    pub failure_run_recall_rate: f64,
    pub numeric_primary_failure_run_recall: usize,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub numeric_primary_pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_cusum_runs: Option<f64>,
    pub mean_lead_delta_vs_run_energy_runs: Option<f64>,
    pub mean_lead_delta_vs_pca_fdc_runs: Option<f64>,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
    pub raw_boundary_nuisance_proxy: f64,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub dsa_episodes_preceding_failure: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub precursor_quality: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub primary_success_condition_met: bool,
    pub any_metric_improved: bool,
    pub validation_passed: bool,
    pub success_condition_failures: Vec<String>,
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
    pub mean_lead_delta_vs_cusum_runs: Option<f64>,
    pub mean_lead_delta_vs_run_energy_runs: Option<f64>,
    pub mean_lead_delta_vs_pca_fdc_runs: Option<f64>,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaComponentContribution {
    pub component: String,
    pub mean_value_on_alert_points: f64,
    pub mean_value_on_all_points: f64,
    pub total_value_on_alert_points: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaVsBaselinesSummary {
    pub dataset: String,
    pub primary_run_signal: String,
    pub dsa: SignalComparisonRow,
    pub numeric_dsa: SignalComparisonRow,
    pub threshold: SignalComparisonRow,
    pub ewma: SignalComparisonRow,
    pub cusum: SignalComparisonRow,
    pub run_energy: SignalComparisonRow,
    pub pca_fdc: SignalComparisonRow,
    pub dsfb_violation: SignalComparisonRow,
    pub dsfb_raw_boundary: SignalComparisonRow,
    pub episode_summary: DsaEpisodeSummary,
    pub failure_recall_delta_vs_threshold: i64,
    pub failure_recall_delta_vs_ewma: i64,
    pub failure_recall_delta_vs_cusum: i64,
    pub failure_recall_delta_vs_run_energy: i64,
    pub failure_recall_delta_vs_pca_fdc: i64,
    pub failure_recall_delta_vs_violation: i64,
    pub pass_run_nuisance_delta_vs_threshold: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_violation: f64,
    pub pass_run_nuisance_delta_vs_cusum: f64,
    pub pass_run_nuisance_delta_vs_run_energy: f64,
    pub pass_run_nuisance_delta_vs_pca_fdc: f64,
    pub pass_run_nuisance_delta_vs_raw_boundary: f64,
    pub pass_run_nuisance_delta_vs_numeric_dsa: f64,
    pub precursor_quality: Option<f64>,
    pub dsa_episodes_preceding_failure: usize,
    pub component_contributions: Vec<DsaComponentContribution>,
    pub motif_policy_contributions: Vec<DsaMotifPolicyContribution>,
    pub policy_vs_numeric_recall_delta: i64,
    pub watch_point_count: usize,
    pub review_point_count: usize,
    pub escalate_point_count: usize,
    pub silenced_point_count: usize,
    pub nuisance_improved: bool,
    pub lead_time_improved: bool,
    pub recall_preserved: bool,
    pub compression_improved: bool,
    pub nothing_improved: bool,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub primary_success_condition_met: bool,
    pub any_metric_improved: bool,
    pub validation_passed: bool,
    pub success_condition_failures: Vec<String>,
    pub validation_failures: Vec<String>,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerFailureRunDsaSignal {
    pub failure_run_index: usize,
    pub failure_timestamp: String,
    pub earliest_dsa_run: Option<usize>,
    pub earliest_primary_source: Option<String>,
    pub earliest_dsa_feature_index: Option<usize>,
    pub earliest_dsa_feature_name: Option<String>,
    pub dsa_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub cusum_lead_runs: Option<usize>,
    pub run_energy_lead_runs: Option<usize>,
    pub pca_fdc_lead_runs: Option<usize>,
    pub dsa_minus_cusum_delta_runs: Option<i64>,
    pub dsa_minus_run_energy_delta_runs: Option<i64>,
    pub dsa_minus_pca_fdc_delta_runs: Option<i64>,
    pub dsa_minus_threshold_delta_runs: Option<i64>,
    pub dsa_minus_ewma_delta_runs: Option<i64>,
    pub dsa_alerting_feature_count: usize,
    pub max_dsa_score_in_lookback: Option<f64>,
    pub max_dsa_score_feature_index: Option<usize>,
    pub max_dsa_score_feature_name: Option<String>,
    pub max_dsa_score_run_index: Option<usize>,
    pub max_dsa_score_boundary_density_w: Option<f64>,
    pub max_dsa_score_drift_persistence_w: Option<f64>,
    pub max_dsa_score_slew_density_w: Option<f64>,
    pub max_dsa_score_ewma_occupancy_w: Option<f64>,
    pub max_dsa_score_motif_recurrence_w: Option<f64>,
    pub max_dsa_score_fragmentation_proxy_w: Option<f64>,
    pub max_dsa_score_consistent: Option<bool>,
    pub max_dsa_score_policy_state: Option<String>,
    pub max_dsa_score_resolved_alert_class: Option<String>,
    pub max_dsa_score_numeric_dsa_alert: Option<bool>,
    pub max_dsa_score_dsa_alert: Option<bool>,
    pub max_dsa_score_policy_suppressed: Option<bool>,
    pub max_dsa_score_rescue_transition: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaEvaluation {
    pub traces: Vec<DsaFeatureTrace>,
    pub run_signals: DsaRunSignals,
    pub episode_summary: DsaEpisodeSummary,
    pub parameter_manifest: DsaParameterManifest,
    pub policy_runtime: DsaPolicyRuntime,
    pub summary: DsaSignalSummary,
    pub comparison_summary: DsaVsBaselinesSummary,
    pub motif_policy_contributions: Vec<DsaMotifPolicyContribution>,
    pub per_failure_run_signals: Vec<PerFailureRunDsaSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsaCalibrationGrid {
    pub window: Vec<usize>,
    pub persistence_runs: Vec<usize>,
    pub alert_tau: Vec<f64>,
    pub corroborating_feature_count_min: Vec<usize>,
}

impl DsaCalibrationGrid {
    pub fn bounded_default() -> Self {
        Self {
            window: vec![5, 10, 15],
            persistence_runs: vec![2, 3, 4],
            alert_tau: vec![2.0, 2.5, 3.0],
            corroborating_feature_count_min: vec![2, 3, 5],
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
            self.corroborating_feature_count_min.len(),
        ]
        .into_iter()
        .product()
    }

    pub fn expand(&self) -> Vec<DsaConfig> {
        let mut out = Vec::with_capacity(self.grid_point_count());
        for &window in &self.window {
            for &persistence_runs in &self.persistence_runs {
                for &alert_tau in &self.alert_tau {
                    for &corroborating_feature_count_min in &self.corroborating_feature_count_min {
                        out.push(DsaConfig {
                            window,
                            persistence_runs,
                            alert_tau,
                            corroborating_feature_count_min,
                        });
                    }
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
    pub corroborating_feature_count_min: usize,
    pub failure_run_recall: usize,
    pub failure_runs: usize,
    pub threshold_failure_run_recall: usize,
    pub ewma_failure_run_recall: usize,
    pub failure_recall_delta_vs_threshold: i64,
    pub failure_recall_delta_vs_ewma: i64,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub mean_lead_delta_vs_cusum_runs: Option<f64>,
    pub mean_lead_delta_vs_run_energy_runs: Option<f64>,
    pub mean_lead_delta_vs_pca_fdc_runs: Option<f64>,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
    pub pass_run_nuisance_delta_vs_cusum: f64,
    pub pass_run_nuisance_delta_vs_run_energy: f64,
    pub pass_run_nuisance_delta_vs_pca_fdc: f64,
    pub pass_run_nuisance_delta_vs_threshold: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_raw_boundary: f64,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub dsa_episodes_preceding_failure: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub precursor_quality: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
    pub nuisance_improved: bool,
    pub lead_time_improved: bool,
    pub recall_preserved: bool,
    pub compression_improved: bool,
    pub any_metric_improved: bool,
    pub nothing_improved: bool,
    pub threshold_recall_gate_passed: bool,
    pub boundary_nuisance_gate_passed: bool,
    pub primary_success_condition_met: bool,
    pub validation_passed: bool,
    pub success_condition_failures: String,
    pub validation_failures: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaCorroborationSummary {
    pub corroborating_feature_count_min: usize,
    pub representative_row: Option<DsaCalibrationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsaGridSummary {
    pub grid_point_count: usize,
    pub optimization_priority_order: Vec<String>,
    pub primary_success_condition_definition: String,
    pub success_row_count: usize,
    pub any_success_row: bool,
    pub closest_to_success: Option<DsaCalibrationRow>,
    pub best_success_row: Option<DsaCalibrationRow>,
    pub best_precursor_quality_row: Option<DsaCalibrationRow>,
    pub corroboration_summaries: Vec<DsaCorroborationSummary>,
    pub cross_feature_corroboration_effect: String,
    pub limiting_factor: String,
}

const DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS: usize = 1;
const POLICY_LOCAL_EWMA_CORROBORATION_MIN: f64 = 0.75;

#[derive(Debug, Clone)]
struct MotifContributionState {
    motif_name: &'static str,
    default_alert_class: HeuristicAlertClass,
    contribution_state: DsaPolicyState,
    fragmentation_proxy: f64,
    suppressed_to_silent: bool,
}

fn dsa_optimization_priority_order() -> Vec<String> {
    vec![
        "1. Reduce nuisance vs raw DSFB boundary".into(),
        "2. Reduce nuisance vs EWMA".into(),
        "3. Preserve threshold recall".into(),
        "4. Improve lead time without sacrificing nuisance".into(),
    ]
}

fn dsa_primary_success_condition_definition() -> String {
    format!(
        "DSA is considered successful only if pass-run nuisance is lower than EWMA nuisance and failure recall is within {} run(s) of threshold recall.",
        DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS
    )
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
    evaluate_dsa_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        config,
        pre_failure_lookback_runs,
        &DsaPolicyRuntime::default(),
    )
}

pub fn evaluate_dsa_with_policy(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    config: &DsaConfig,
    pre_failure_lookback_runs: usize,
    policy_runtime: &DsaPolicyRuntime,
) -> Result<DsaEvaluation> {
    config.validate()?;
    let weights = DsaWeights::default();
    let run_count = dataset.labels.len();
    let failure_indices = dataset
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| (*label == 1).then_some(index))
        .collect::<Vec<_>>();
    let failure_window_mask =
        build_failure_window_mask(run_count, &failure_indices, pre_failure_lookback_runs);
    let feature_policy_overrides = policy_runtime
        .feature_policy_overrides
        .iter()
        .map(|override_entry| (override_entry.feature_index, override_entry))
        .collect::<BTreeMap<_, _>>();
    let mut traces = Vec::with_capacity(residuals.traces.len());

    for (((residual_trace, sign_trace), ewma_trace), grammar_trace) in residuals
        .traces
        .iter()
        .zip(&signs.traces)
        .zip(&baselines.ewma)
        .zip(&grammar.traces)
    {
        let feature = &nominal.features[residual_trace.feature_index];
        let feature_override = feature_policy_overrides
            .get(&feature.feature_index)
            .copied();
        if !feature.analyzable {
            traces.push(empty_trace(
                feature.feature_index,
                &feature.feature_name,
                run_count,
            ));
            continue;
        }

        let boundary_basis_hit = grammar_trace
            .raw_states
            .iter()
            .map(|state| *state == GrammarState::Boundary)
            .collect::<Vec<_>>();
        let raw_violation_hit = grammar_trace
            .raw_states
            .iter()
            .map(|state| *state == GrammarState::Violation)
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
        let motif_flags = dsa_contributing_motif_names()
            .iter()
            .map(|&motif_name| {
                let flags = grammar_trace
                    .raw_reasons
                    .iter()
                    .map(|reason| dsa_motif_name(reason) == Some(motif_name))
                    .collect::<Vec<_>>();
                (motif_name, flags)
            })
            .collect::<Vec<_>>();
        let motif_hit = (0..run_count)
            .map(|run_index| motif_flags.iter().any(|(_, flags)| flags[run_index]))
            .collect::<Vec<_>>();
        let ewma_normalized = ewma_trace
            .ewma
            .iter()
            .map(|value| normalize_to_threshold(*value, ewma_trace.threshold))
            .collect::<Vec<_>>();

        let boundary_prefix = bool_prefix_sum(&boundary_basis_hit);
        let raw_violation_prefix = bool_prefix_sum(&raw_violation_hit);
        let drift_prefix = bool_prefix_sum(&drift_outward_hit);
        let slew_prefix = bool_prefix_sum(&slew_hit);
        let motif_prefix = bool_prefix_sum(&motif_hit);
        let non_imputed_prefix = bool_prefix_sum(
            &residual_trace
                .is_imputed
                .iter()
                .map(|is_imputed| !*is_imputed)
                .collect::<Vec<_>>(),
        );

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
            let boundary_density =
                window_fraction_nonimputed(&boundary_prefix, &non_imputed_prefix, start, run_index);
            let drift_persistence =
                window_fraction_nonimputed(&drift_prefix, &non_imputed_prefix, start, run_index);
            let slew_density = window_fraction(&slew_prefix, start, run_index, window_len);
            let ewma_occupancy = window_mean(&ewma_normalized, start, run_index);
            let motif_recurrence = window_fraction(&motif_prefix, start, run_index, window_len);
            let consistent_window = window_is_consistent(
                &sign_trace.drift,
                sign_trace.drift_threshold,
                start,
                run_index,
            );
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

        let numeric_dsa_alert = persistence_mask(&dsa_active, config.persistence_runs);
        let mut dsa_alert = Vec::with_capacity(run_count);
        let mut fragmentation_proxy_w = Vec::with_capacity(run_count);
        let mut resolved_alert_class = Vec::with_capacity(run_count);
        let mut policy_state = Vec::with_capacity(run_count);
        let mut policy_suppressed_to_silent = Vec::with_capacity(run_count);
        let mut rescue_transition = Vec::with_capacity(run_count);
        let mut rescued_to_review = Vec::with_capacity(run_count);
        let mut motif_policy_contributions = dsa_contributing_motif_names()
            .iter()
            .map(|motif_name| {
                let policy = heuristic_policy_definition(motif_name)
                    .unwrap_or_else(|| panic!("missing heuristic policy for {motif_name}"));
                (
                    (*motif_name).to_string(),
                    FeatureMotifPolicyContribution {
                        motif_name: (*motif_name).into(),
                        alert_class_default: policy.alert_class_default,
                        watch_points: 0,
                        review_points: 0,
                        escalate_points: 0,
                        silent_suppression_points: 0,
                        pass_review_or_escalate_points: 0,
                        pre_failure_review_or_escalate_points: 0,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        for run_index in 0..run_count {
            let start = run_index.saturating_sub(config.window.saturating_sub(1));
            let raw_violation_recent = window_count(&raw_violation_prefix, start, run_index) > 0;
            let local_corroboration = raw_violation_recent
                || ewma_occupancy_w[run_index] >= POLICY_LOCAL_EWMA_CORROBORATION_MIN;
            let contributions = motif_flags
                .iter()
                .filter_map(|(motif_name, flags)| {
                    let policy = heuristic_policy_definition(motif_name)?;
                    let contribution = motif_contribution_state(
                        policy,
                        feature_override,
                        flags,
                        run_index,
                        dsa_active[run_index],
                        numeric_dsa_alert[run_index],
                        local_corroboration,
                        raw_violation_recent,
                    )?;
                    Some(contribution)
                })
                .collect::<Vec<_>>();

            let dominant_class = dominant_alert_class(&contributions);
            let dominant_state = dominant_policy_state(&contributions);
            let fragmentation = contributions
                .iter()
                .map(|contribution| contribution.fragmentation_proxy)
                .fold(0.0, f64::max);
            let silenced =
                numeric_dsa_alert[run_index] && matches!(dominant_state, DsaPolicyState::Silent);

            for contribution in &contributions {
                if let Some(row) = motif_policy_contributions.get_mut(contribution.motif_name) {
                    match contribution.contribution_state {
                        DsaPolicyState::Silent => {
                            if contribution.suppressed_to_silent {
                                row.silent_suppression_points += 1;
                            }
                        }
                        DsaPolicyState::Watch => row.watch_points += 1,
                        DsaPolicyState::Review => row.review_points += 1,
                        DsaPolicyState::Escalate => row.escalate_points += 1,
                    }
                    if contribution.contribution_state.is_review_or_escalate() {
                        if dataset.labels[run_index] == -1 {
                            row.pass_review_or_escalate_points += 1;
                        }
                        if failure_window_mask[run_index] {
                            row.pre_failure_review_or_escalate_points += 1;
                        }
                    }
                }
            }

            dsa_alert.push(dominant_state.is_review_or_escalate());
            fragmentation_proxy_w.push(fragmentation);
            resolved_alert_class.push(dominant_class);
            policy_state.push(dominant_state);
            policy_suppressed_to_silent.push(silenced);
            rescue_transition.push("none".into());
            rescued_to_review.push(false);
        }

        if policy_runtime.recall_rescue.enabled {
            for run_index in 0..run_count {
                let transition = apply_recall_rescue(
                    feature_override,
                    &policy_runtime.recall_rescue,
                    config,
                    &resolved_alert_class,
                    &mut policy_state,
                    &mut dsa_alert,
                    &fragmentation_proxy_w,
                    &dsa_score,
                    &boundary_density_w,
                    &ewma_occupancy_w,
                    &motif_recurrence_w,
                    &consistent,
                    run_index,
                );
                if let Some(transition_name) = transition {
                    rescue_transition[run_index] = transition_name.to_string();
                    rescued_to_review[run_index] = transition_name == "watch_to_review"
                        || transition_name == "silent_to_review";
                }
            }
        }

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
            fragmentation_proxy_w,
            consistent,
            dsa_score,
            dsa_active,
            numeric_dsa_alert,
            dsa_alert,
            resolved_alert_class,
            policy_state,
            policy_suppressed_to_silent,
            rescue_transition,
            rescued_to_review,
            motif_policy_contributions: motif_policy_contributions.into_values().collect(),
        });
    }

    Ok(assemble_dsa_evaluation(
        dataset,
        nominal,
        residuals,
        baselines,
        grammar,
        traces,
        config,
        &weights,
        pre_failure_lookback_runs,
        policy_runtime,
        None,
        None,
    ))
}

pub fn project_dsa_to_cohort(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    base_evaluation: &DsaEvaluation,
    selected_feature_indices: &[usize],
    corroborating_feature_count_min: usize,
    pre_failure_lookback_runs: usize,
    cohort_name: &str,
) -> Result<DsaEvaluation> {
    if corroborating_feature_count_min == 0 {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "cohort corroborating_feature_count_min must be positive".into(),
        ));
    }

    let mut selected_mask = vec![false; base_evaluation.traces.len()];
    for &feature_index in selected_feature_indices {
        if feature_index < selected_mask.len() {
            selected_mask[feature_index] = true;
        }
    }

    let mut traces = base_evaluation.traces.clone();
    for (feature_index, trace) in traces.iter_mut().enumerate() {
        if !selected_mask[feature_index] {
            trace.dsa_score.fill(0.0);
            trace.dsa_active.fill(false);
            trace.numeric_dsa_alert.fill(false);
            trace.dsa_alert.fill(false);
            trace.resolved_alert_class.fill(HeuristicAlertClass::Silent);
            trace.policy_state.fill(DsaPolicyState::Silent);
            trace.policy_suppressed_to_silent.fill(false);
            for contribution in &mut trace.motif_policy_contributions {
                contribution.watch_points = 0;
                contribution.review_points = 0;
                contribution.escalate_points = 0;
                contribution.silent_suppression_points = 0;
                contribution.pass_review_or_escalate_points = 0;
                contribution.pre_failure_review_or_escalate_points = 0;
            }
        }
    }

    let mut config = base_evaluation.parameter_manifest.config.clone();
    config.corroborating_feature_count_min = corroborating_feature_count_min;
    config.validate()?;

    let mut evaluation = assemble_dsa_evaluation(
        dataset,
        nominal,
        residuals,
        baselines,
        grammar,
        traces,
        &config,
        &base_evaluation.parameter_manifest.weights,
        pre_failure_lookback_runs,
        &base_evaluation.policy_runtime,
        Some(&selected_mask),
        Some(selected_feature_indices.len()),
    );

    let primary_signal = format!(
        "cohort {}: feature_count_review_or_escalate(k) >= {}",
        cohort_name, corroborating_feature_count_min
    );
    evaluation.run_signals.primary_run_signal = primary_signal.clone();
    evaluation.episode_summary.primary_signal = primary_signal.clone();
    evaluation.summary.primary_run_signal = primary_signal.clone();
    evaluation.summary.config.corroborating_feature_count_min = corroborating_feature_count_min;
    evaluation.comparison_summary.primary_run_signal = primary_signal.clone();
    evaluation
        .parameter_manifest
        .config
        .corroborating_feature_count_min = corroborating_feature_count_min;
    evaluation.parameter_manifest.primary_run_signal = primary_signal.clone();
    evaluation.parameter_manifest.primary_run_signal_definition = format!(
        "Primary run-level cohort DSA decision is {} over the deterministic selected feature cohort.",
        primary_signal
    );
    evaluation.parameter_manifest.corroboration_rule = format!(
        "RUN_LEVEL_DSA(k) is true only when the count of selected cohort features in Review or Escalate is at least {}.",
        corroborating_feature_count_min
    );

    Ok(evaluation)
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
            corroborating_feature_count_min: config.corroborating_feature_count_min,
            failure_run_recall: evaluation.summary.failure_run_recall,
            failure_runs: evaluation.summary.failure_runs,
            threshold_failure_run_recall: evaluation
                .comparison_summary
                .threshold
                .failure_run_recall,
            ewma_failure_run_recall: evaluation.comparison_summary.ewma.failure_run_recall,
            failure_recall_delta_vs_threshold: evaluation
                .comparison_summary
                .failure_recall_delta_vs_threshold,
            failure_recall_delta_vs_ewma: evaluation
                .comparison_summary
                .failure_recall_delta_vs_ewma,
            mean_lead_time_runs: evaluation.summary.mean_lead_time_runs,
            median_lead_time_runs: evaluation.summary.median_lead_time_runs,
            pass_run_nuisance_proxy: evaluation.summary.pass_run_nuisance_proxy,
            mean_lead_delta_vs_cusum_runs: evaluation.summary.mean_lead_delta_vs_cusum_runs,
            mean_lead_delta_vs_run_energy_runs: evaluation
                .summary
                .mean_lead_delta_vs_run_energy_runs,
            mean_lead_delta_vs_pca_fdc_runs: evaluation.summary.mean_lead_delta_vs_pca_fdc_runs,
            mean_lead_delta_vs_threshold_runs: evaluation.summary.mean_lead_delta_vs_threshold_runs,
            mean_lead_delta_vs_ewma_runs: evaluation.summary.mean_lead_delta_vs_ewma_runs,
            pass_run_nuisance_delta_vs_cusum: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_cusum,
            pass_run_nuisance_delta_vs_run_energy: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_run_energy,
            pass_run_nuisance_delta_vs_pca_fdc: evaluation
                .comparison_summary
                .pass_run_nuisance_delta_vs_pca_fdc,
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
            dsa_episodes_preceding_failure: evaluation
                .episode_summary
                .dsa_episodes_preceding_failure,
            mean_dsa_episode_length_runs: evaluation.episode_summary.mean_dsa_episode_length_runs,
            max_dsa_episode_length_runs: evaluation.episode_summary.max_dsa_episode_length_runs,
            compression_ratio: evaluation.episode_summary.compression_ratio,
            precursor_quality: evaluation.episode_summary.precursor_quality,
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
            primary_success_condition_met: evaluation.summary.primary_success_condition_met,
            validation_passed: evaluation.summary.validation_passed,
            success_condition_failures: evaluation.summary.success_condition_failures.join("; "),
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
        fragmentation_proxy_w: vec![0.0; run_count],
        consistent: vec![true; run_count],
        dsa_score: vec![0.0; run_count],
        dsa_active: vec![false; run_count],
        numeric_dsa_alert: vec![false; run_count],
        dsa_alert: vec![false; run_count],
        resolved_alert_class: vec![HeuristicAlertClass::Silent; run_count],
        policy_state: vec![DsaPolicyState::Silent; run_count],
        policy_suppressed_to_silent: vec![false; run_count],
        rescue_transition: vec!["none".into(); run_count],
        rescued_to_review: vec![false; run_count],
        motif_policy_contributions: Vec::new(),
    }
}

fn assemble_dsa_evaluation(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    traces: Vec<DsaFeatureTrace>,
    config: &DsaConfig,
    weights: &DsaWeights,
    pre_failure_lookback_runs: usize,
    policy_runtime: &DsaPolicyRuntime,
    raw_boundary_episode_feature_mask: Option<&[bool]>,
    analyzable_feature_count_override: Option<usize>,
) -> DsaEvaluation {
    let run_count = dataset.labels.len();
    let motif_names = dsa_contributing_motif_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
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
    let cusum_run_signal = (0..run_count)
        .map(|run_index| baselines.cusum.iter().any(|trace| trace.alarm[run_index]))
        .collect::<Vec<_>>();
    let run_energy_run_signal = baselines.run_energy.alarm.clone();
    let pca_fdc_run_signal = baselines.pca_fdc.alarm.clone();
    let run_signals = build_run_signals(&traces, &raw_violation_run_signal, config, run_count);

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
    let failure_window_mask =
        build_failure_window_mask(run_count, &failure_indices, pre_failure_lookback_runs);

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
    let cusum_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&cusum_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let run_energy_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&run_energy_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let pca_fdc_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(&pca_fdc_run_signal, window_start, failure_index)
                .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();
    let numeric_dsa_leads = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            earliest_run_signal(
                &run_signals.numeric_primary_run_alert,
                window_start,
                failure_index,
            )
            .map(|run_index| failure_index - run_index)
        })
        .collect::<Vec<_>>();

    let per_failure_run_signals = failure_indices
        .iter()
        .map(|&failure_index| {
            let window_start = failure_index.saturating_sub(pre_failure_lookback_runs);
            let earliest_dsa = earliest_primary_signal(
                &traces,
                &run_signals.primary_run_alert,
                window_start,
                failure_index,
            );
            let earliest_threshold_run =
                earliest_run_signal(&threshold_run_signal, window_start, failure_index);
            let earliest_ewma_run =
                earliest_run_signal(&ewma_run_signal, window_start, failure_index);
            let earliest_cusum_run =
                earliest_run_signal(&cusum_run_signal, window_start, failure_index);
            let earliest_run_energy_run =
                earliest_run_signal(&run_energy_run_signal, window_start, failure_index);
            let earliest_pca_fdc_run =
                earliest_run_signal(&pca_fdc_run_signal, window_start, failure_index);
            let dsa_lead_runs = earliest_dsa
                .as_ref()
                .map(|signal| failure_index - signal.run_index);
            let threshold_lead_runs = earliest_threshold_run.map(|index| failure_index - index);
            let ewma_lead_runs = earliest_ewma_run.map(|index| failure_index - index);
            let cusum_lead_runs = earliest_cusum_run.map(|index| failure_index - index);
            let run_energy_lead_runs = earliest_run_energy_run.map(|index| failure_index - index);
            let pca_fdc_lead_runs = earliest_pca_fdc_run.map(|index| failure_index - index);
            let alerting_feature_count = traces
                .iter()
                .filter(|trace| {
                    trace.dsa_alert[window_start..failure_index]
                        .iter()
                        .any(|flag| *flag)
                })
                .count();
            let max_score = max_dsa_score(&traces, window_start, failure_index);

            PerFailureRunDsaSignal {
                failure_run_index: failure_index,
                failure_timestamp: dataset.timestamps[failure_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                earliest_dsa_run: earliest_dsa.as_ref().map(|signal| signal.run_index),
                earliest_primary_source: earliest_dsa.as_ref().map(|signal| signal.source.clone()),
                earliest_dsa_feature_index: earliest_dsa
                    .as_ref()
                    .map(|signal| signal.feature_index),
                earliest_dsa_feature_name: earliest_dsa
                    .as_ref()
                    .map(|signal| signal.feature_name.clone()),
                dsa_lead_runs,
                threshold_lead_runs,
                ewma_lead_runs,
                cusum_lead_runs,
                run_energy_lead_runs,
                pca_fdc_lead_runs,
                dsa_minus_cusum_delta_runs: paired_delta(dsa_lead_runs, cusum_lead_runs),
                dsa_minus_run_energy_delta_runs: paired_delta(dsa_lead_runs, run_energy_lead_runs),
                dsa_minus_pca_fdc_delta_runs: paired_delta(dsa_lead_runs, pca_fdc_lead_runs),
                dsa_minus_threshold_delta_runs: paired_delta(dsa_lead_runs, threshold_lead_runs),
                dsa_minus_ewma_delta_runs: paired_delta(dsa_lead_runs, ewma_lead_runs),
                dsa_alerting_feature_count: alerting_feature_count,
                max_dsa_score_in_lookback: max_score.as_ref().map(|score| score.score),
                max_dsa_score_feature_index: max_score.as_ref().map(|score| score.feature_index),
                max_dsa_score_feature_name: max_score
                    .as_ref()
                    .map(|score| score.feature_name.clone()),
                max_dsa_score_run_index: max_score.as_ref().map(|score| score.run_index),
                max_dsa_score_boundary_density_w: max_score
                    .as_ref()
                    .map(|score| score.boundary_density_w),
                max_dsa_score_drift_persistence_w: max_score
                    .as_ref()
                    .map(|score| score.drift_persistence_w),
                max_dsa_score_slew_density_w: max_score.as_ref().map(|score| score.slew_density_w),
                max_dsa_score_ewma_occupancy_w: max_score
                    .as_ref()
                    .map(|score| score.ewma_occupancy_w),
                max_dsa_score_motif_recurrence_w: max_score
                    .as_ref()
                    .map(|score| score.motif_recurrence_w),
                max_dsa_score_fragmentation_proxy_w: max_score
                    .as_ref()
                    .map(|score| score.fragmentation_proxy_w),
                max_dsa_score_consistent: max_score.as_ref().map(|score| score.consistent),
                max_dsa_score_policy_state: max_score
                    .as_ref()
                    .map(|score| score.policy_state.as_lowercase().to_string()),
                max_dsa_score_resolved_alert_class: max_score
                    .as_ref()
                    .map(|score| format!("{:?}", score.resolved_alert_class)),
                max_dsa_score_numeric_dsa_alert: max_score
                    .as_ref()
                    .map(|score| score.numeric_dsa_alert),
                max_dsa_score_dsa_alert: max_score.as_ref().map(|score| score.dsa_alert),
                max_dsa_score_policy_suppressed: max_score
                    .as_ref()
                    .map(|score| score.policy_suppressed_to_silent),
                max_dsa_score_rescue_transition: max_score.as_ref().and_then(|score| {
                    (score.rescue_transition != "none").then_some(score.rescue_transition.clone())
                }),
            }
        })
        .collect::<Vec<_>>();

    let alert_point_count = traces
        .iter()
        .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let alert_run_count = run_signals
        .primary_run_alert
        .iter()
        .filter(|flag| **flag)
        .count();
    let numeric_alert_point_count = traces
        .iter()
        .map(|trace| trace.numeric_dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let numeric_alert_run_count = run_signals
        .numeric_primary_run_alert
        .iter()
        .filter(|flag| **flag)
        .count();
    let watch_point_count = traces
        .iter()
        .map(|trace| {
            trace
                .policy_state
                .iter()
                .filter(|state| **state == DsaPolicyState::Watch)
                .count()
        })
        .sum::<usize>();
    let review_point_count = traces
        .iter()
        .map(|trace| {
            trace
                .policy_state
                .iter()
                .filter(|state| **state == DsaPolicyState::Review)
                .count()
        })
        .sum::<usize>();
    let escalate_point_count = traces
        .iter()
        .map(|trace| {
            trace
                .policy_state
                .iter()
                .filter(|state| **state == DsaPolicyState::Escalate)
                .count()
        })
        .sum::<usize>();
    let silenced_point_count = traces
        .iter()
        .map(|trace| {
            trace
                .policy_suppressed_to_silent
                .iter()
                .filter(|flag| **flag)
                .count()
        })
        .sum::<usize>();
    let rescued_point_count = traces
        .iter()
        .map(|trace| {
            trace
                .rescue_transition
                .iter()
                .filter(|transition| transition.as_str() != "none")
                .count()
        })
        .sum::<usize>();
    let rescued_watch_to_review_points = traces
        .iter()
        .map(|trace| {
            trace
                .rescue_transition
                .iter()
                .filter(|transition| transition.as_str() == "watch_to_review")
                .count()
        })
        .sum::<usize>();
    let rescued_review_to_escalate_points = traces
        .iter()
        .map(|trace| {
            trace
                .rescue_transition
                .iter()
                .filter(|transition| transition.as_str() == "review_to_escalate")
                .count()
        })
        .sum::<usize>();
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
                .filter(|&&run_index| run_signals.primary_run_alert[run_index])
                .count(),
            pass_indices.len(),
        ),
        mean_lead_delta_vs_cusum_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_minus_cusum_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_lead_delta_vs_run_energy_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_minus_run_energy_delta_runs)
                .collect::<Vec<_>>(),
        ),
        mean_lead_delta_vs_pca_fdc_runs: mean_option_i64(
            &per_failure_run_signals
                .iter()
                .map(|signal| signal.dsa_minus_pca_fdc_delta_runs)
                .collect::<Vec<_>>(),
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
    let numeric_dsa_row = SignalComparisonRow {
        signal: "Numeric-only DSA".into(),
        failure_run_recall: count_present(numeric_dsa_leads.iter().copied()),
        failure_runs: failure_indices.len(),
        failure_run_recall_rate: rate(
            count_present(numeric_dsa_leads.iter().copied()),
            failure_indices.len(),
        ),
        mean_lead_time_runs: mean_option_usize(&numeric_dsa_leads),
        median_lead_time_runs: median_option_usize(&numeric_dsa_leads),
        pass_run_nuisance_proxy: rate(
            pass_indices
                .iter()
                .filter(|&&run_index| run_signals.numeric_primary_run_alert[run_index])
                .count(),
            pass_indices.len(),
        ),
        mean_lead_delta_vs_cusum_runs: None,
        mean_lead_delta_vs_run_energy_runs: None,
        mean_lead_delta_vs_pca_fdc_runs: None,
        mean_lead_delta_vs_threshold_runs: None,
        mean_lead_delta_vs_ewma_runs: None,
    };
    let motif_policy_contributions = aggregate_motif_policy_contributions(&traces);
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
    let cusum_row = baseline_row(
        "CUSUM",
        failure_indices.len(),
        &cusum_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| cusum_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );
    let run_energy_row = baseline_row(
        "Run energy",
        failure_indices.len(),
        &run_energy_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| run_energy_run_signal[run_index])
                .count(),
            pass_indices.len(),
        ),
    );
    let pca_fdc_row = baseline_row(
        "PCA T2/SPE",
        failure_indices.len(),
        &pca_fdc_leads,
        rate(
            pass_indices
                .iter()
                .filter(|&&run_index| pca_fdc_run_signal[run_index])
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
        .enumerate()
        .filter(|(feature_index, _)| {
            raw_boundary_episode_feature_mask
                .map(|mask| mask.get(*feature_index).copied().unwrap_or(false))
                .unwrap_or(true)
        })
        .map(|(_, trace)| {
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
        &run_signals.primary_run_alert,
        raw_boundary_episode_count,
        &raw_violation_run_signal,
        &failure_window_mask,
    );
    let component_contributions = component_contributions(&traces);
    let nuisance_improved = dsa_row.pass_run_nuisance_proxy < threshold_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < ewma_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < cusum_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < run_energy_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < pca_fdc_row.pass_run_nuisance_proxy
        || dsa_row.pass_run_nuisance_proxy < dsfb_raw_boundary_row.pass_run_nuisance_proxy;
    let lead_time_improved = matches!(dsa_row.mean_lead_delta_vs_threshold_runs, Some(delta) if delta > 0.0)
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
    let success_condition_failures =
        success_condition_failures(&dsa_row, &threshold_row, &ewma_row);
    let primary_success_condition_met = success_condition_failures.is_empty();
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
        &dsfb_violation_row,
        &dsfb_raw_boundary_row,
        &episode_summary,
        primary_success_condition_met,
        nuisance_improved,
        lead_time_improved,
        recall_preserved,
        compression_improved,
        nothing_improved,
        &component_contributions,
        &success_condition_failures,
        &validation_failures,
    );

    let comparison_summary = DsaVsBaselinesSummary {
        dataset: "SECOM".into(),
        primary_run_signal: run_signals.primary_run_signal.clone(),
        dsa: dsa_row.clone(),
        numeric_dsa: numeric_dsa_row.clone(),
        threshold: threshold_row.clone(),
        ewma: ewma_row.clone(),
        cusum: cusum_row.clone(),
        run_energy: run_energy_row.clone(),
        pca_fdc: pca_fdc_row.clone(),
        dsfb_violation: dsfb_violation_row.clone(),
        dsfb_raw_boundary: dsfb_raw_boundary_row.clone(),
        episode_summary: episode_summary.clone(),
        failure_recall_delta_vs_threshold: dsa_row.failure_run_recall as i64
            - threshold_row.failure_run_recall as i64,
        failure_recall_delta_vs_ewma: dsa_row.failure_run_recall as i64
            - ewma_row.failure_run_recall as i64,
        failure_recall_delta_vs_cusum: dsa_row.failure_run_recall as i64
            - cusum_row.failure_run_recall as i64,
        failure_recall_delta_vs_run_energy: dsa_row.failure_run_recall as i64
            - run_energy_row.failure_run_recall as i64,
        failure_recall_delta_vs_pca_fdc: dsa_row.failure_run_recall as i64
            - pca_fdc_row.failure_run_recall as i64,
        failure_recall_delta_vs_violation: dsa_row.failure_run_recall as i64
            - dsfb_violation_row.failure_run_recall as i64,
        pass_run_nuisance_delta_vs_threshold: dsa_row.pass_run_nuisance_proxy
            - threshold_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_ewma: dsa_row.pass_run_nuisance_proxy
            - ewma_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_violation: dsa_row.pass_run_nuisance_proxy
            - dsfb_violation_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_cusum: dsa_row.pass_run_nuisance_proxy
            - cusum_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_run_energy: dsa_row.pass_run_nuisance_proxy
            - run_energy_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_pca_fdc: dsa_row.pass_run_nuisance_proxy
            - pca_fdc_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_raw_boundary: dsa_row.pass_run_nuisance_proxy
            - dsfb_raw_boundary_row.pass_run_nuisance_proxy,
        pass_run_nuisance_delta_vs_numeric_dsa: dsa_row.pass_run_nuisance_proxy
            - numeric_dsa_row.pass_run_nuisance_proxy,
        precursor_quality: episode_summary.precursor_quality,
        dsa_episodes_preceding_failure: episode_summary.dsa_episodes_preceding_failure,
        component_contributions: component_contributions.clone(),
        motif_policy_contributions: motif_policy_contributions.clone(),
        policy_vs_numeric_recall_delta: dsa_row.failure_run_recall as i64
            - numeric_dsa_row.failure_run_recall as i64,
        watch_point_count,
        review_point_count,
        escalate_point_count,
        silenced_point_count,
        nuisance_improved,
        lead_time_improved,
        recall_preserved,
        compression_improved,
        nothing_improved,
        threshold_recall_gate_passed,
        boundary_nuisance_gate_passed,
        primary_success_condition_met,
        any_metric_improved,
        validation_passed,
        success_condition_failures: success_condition_failures.clone(),
        validation_failures: validation_failures.clone(),
        conclusion,
    };
    let parameter_manifest = DsaParameterManifest {
        config: config.clone(),
        weights: weights.clone(),
        feature_policy_override_count: policy_runtime.feature_policy_overrides.len(),
        feature_policy_override_summary: policy_runtime
            .feature_policy_overrides
            .iter()
            .map(|override_entry| {
                format!(
                    "{}: rescue_eligible={}, rescue_priority={}, override_reason={}",
                    override_entry.feature_name,
                    override_entry.rescue_eligible,
                    override_entry.rescue_priority,
                    override_entry.override_reason
                )
            })
            .collect(),
        policy_engine_definition:
            "Deterministic heuristics-governed policy engine over structural DSA candidates with explicit Silent/Watch/Review/Escalate feature states."
                .into(),
        feature_level_state_definition:
            "Feature state is resolved from active motif policies, structural score >= tau, persistence gating, directional consistency, local corroboration, and explicit silence suppression."
                .into(),
        primary_run_signal: run_signals.primary_run_signal.clone(),
        primary_run_signal_definition: format!(
            "Primary run-level DSA decision is feature_count_review_or_escalate(k) >= {}. This is a cross-feature corroboration gate above the frozen DSFB states and scalar baselines.",
            config.corroborating_feature_count_min
        ),
        secondary_run_signal: "any_feature_review_or_escalate(k)".into(),
        tertiary_run_signal: "feature_count_escalate(k)".into(),
        strict_escalate_signal: format!(
            "feature_count_escalate(k) >= {}",
            config.corroborating_feature_count_min
        ),
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
        silence_rule:
            "A structural candidate remains Silent when no motif policy activates, when motif hits stay below deterministic minimum_hits/minimum_window rules, when fragmentation exceeds the motif ceiling, or when persistence/corroboration gates fail."
                .into(),
        corroboration_rule: format!(
            "RUN_LEVEL_DSA(k) is true only when the count of features in Review or Escalate is at least {}.",
            config.corroborating_feature_count_min
        ),
        recall_rescue_definition: if policy_runtime.recall_rescue.enabled {
            format!(
                "Bounded recall rescue is enabled for explicit feature overrides only. Rescue promotes Watch-class near-miss structure when dsa_score >= tau - margin, boundary_density >= {:.2}, motif_recurrence >= {:.2}, ewma_occupancy >= {:.2}, and the override-specific watch-hit / fragmentation guards pass. Priority-2 overrides may rescue repeated Watch-class structure even when the directional-consistency rule fails.",
                policy_runtime.recall_rescue.minimum_boundary_density,
                policy_runtime.recall_rescue.minimum_motif_recurrence,
                policy_runtime.recall_rescue.minimum_ewma_occupancy,
            )
        } else {
            "Bounded recall rescue is disabled.".into()
        },
        recall_tolerance_runs_for_primary_success: DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS,
        primary_success_condition_definition: dsa_primary_success_condition_definition(),
        optimization_priority_order: dsa_optimization_priority_order(),
    };

    DsaEvaluation {
        traces,
        run_signals: run_signals.clone(),
        episode_summary: episode_summary.clone(),
        parameter_manifest,
        policy_runtime: policy_runtime.clone(),
        summary: DsaSignalSummary {
            config: config.clone(),
            weights: weights.clone(),
            primary_run_signal: run_signals.primary_run_signal.clone(),
            analyzable_feature_count: analyzable_feature_count_override.unwrap_or_else(|| {
                nominal
                    .features
                    .iter()
                    .filter(|feature| feature.analyzable)
                    .count()
            }),
            alert_point_count,
            alert_run_count,
            numeric_alert_point_count,
            numeric_alert_run_count,
            watch_point_count,
            review_point_count,
            escalate_point_count,
            silenced_point_count,
            rescued_point_count,
            rescued_watch_to_review_points,
            rescued_review_to_escalate_points,
            failure_runs: failure_indices.len(),
            failure_run_recall,
            failure_run_recall_rate: dsa_row.failure_run_recall_rate,
            numeric_primary_failure_run_recall: numeric_dsa_row.failure_run_recall,
            mean_lead_time_runs: dsa_row.mean_lead_time_runs,
            median_lead_time_runs: dsa_row.median_lead_time_runs,
            pass_run_nuisance_proxy: dsa_row.pass_run_nuisance_proxy,
            numeric_primary_pass_run_nuisance_proxy: numeric_dsa_row.pass_run_nuisance_proxy,
            mean_lead_delta_vs_cusum_runs: dsa_row.mean_lead_delta_vs_cusum_runs,
            mean_lead_delta_vs_run_energy_runs: dsa_row.mean_lead_delta_vs_run_energy_runs,
            mean_lead_delta_vs_pca_fdc_runs: dsa_row.mean_lead_delta_vs_pca_fdc_runs,
            mean_lead_delta_vs_threshold_runs: dsa_row.mean_lead_delta_vs_threshold_runs,
            mean_lead_delta_vs_ewma_runs: dsa_row.mean_lead_delta_vs_ewma_runs,
            raw_boundary_nuisance_proxy: dsfb_raw_boundary_row.pass_run_nuisance_proxy,
            raw_boundary_episode_count: episode_summary.raw_boundary_episode_count,
            dsa_episode_count: episode_summary.dsa_episode_count,
            dsa_episodes_preceding_failure: episode_summary.dsa_episodes_preceding_failure,
            mean_dsa_episode_length_runs: episode_summary.mean_dsa_episode_length_runs,
            max_dsa_episode_length_runs: episode_summary.max_dsa_episode_length_runs,
            compression_ratio: episode_summary.compression_ratio,
            precursor_quality: episode_summary.precursor_quality,
            non_escalating_dsa_episode_fraction: episode_summary
                .non_escalating_dsa_episode_fraction,
            threshold_recall_gate_passed,
            boundary_nuisance_gate_passed,
            primary_success_condition_met,
            any_metric_improved,
            validation_passed,
            success_condition_failures,
            validation_failures,
        },
        comparison_summary,
        motif_policy_contributions,
        per_failure_run_signals,
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

fn motif_contribution_state(
    policy: HeuristicPolicyDefinition,
    feature_override: Option<&FeaturePolicyOverride>,
    flags: &[bool],
    run_index: usize,
    structural_active: bool,
    numeric_alert: bool,
    local_corroboration: bool,
    raw_violation_recent: bool,
) -> Option<MotifContributionState> {
    let minimum_window = feature_override
        .and_then(|override_entry| override_entry.minimum_window_override)
        .unwrap_or(policy.minimum_window);
    let minimum_hits = feature_override
        .and_then(|override_entry| override_entry.minimum_hits_override)
        .unwrap_or(policy.minimum_hits);
    let maximum_allowed_fragmentation = feature_override
        .and_then(|override_entry| override_entry.maximum_allowed_fragmentation_override)
        .unwrap_or_else(|| policy.maximum_allowed_fragmentation());
    let alert_class_default = feature_override
        .and_then(|override_entry| override_entry.alert_class_override)
        .unwrap_or(policy.alert_class_default);
    let requires_persistence = feature_override
        .and_then(|override_entry| override_entry.requires_persistence_override)
        .unwrap_or(policy.requires_persistence);
    let requires_corroboration = feature_override
        .and_then(|override_entry| override_entry.requires_corroboration_override)
        .unwrap_or(policy.requires_corroboration);
    let start = run_index.saturating_sub(minimum_window.saturating_sub(1));
    let hits = flags[start..=run_index]
        .iter()
        .filter(|flag| **flag)
        .count();
    if hits == 0 {
        return None;
    }

    let fragmentation_proxy = episode_ranges(&flags[start..=run_index]).len() as f64 / hits as f64;
    let policy_active =
        hits >= minimum_hits && fragmentation_proxy <= maximum_allowed_fragmentation;
    let suppressed_to_silent = structural_active && !policy_active;
    let mut contribution_state = if !structural_active || !policy_active {
        DsaPolicyState::Silent
    } else {
        match alert_class_default {
            HeuristicAlertClass::Silent => {
                if policy.promotes_alert && numeric_alert && local_corroboration {
                    if raw_violation_recent {
                        DsaPolicyState::Review
                    } else {
                        DsaPolicyState::Watch
                    }
                } else {
                    DsaPolicyState::Silent
                }
            }
            HeuristicAlertClass::Watch => {
                if requires_persistence && !numeric_alert {
                    DsaPolicyState::Silent
                } else if requires_corroboration && !local_corroboration {
                    if policy.suppresses_alert {
                        DsaPolicyState::Silent
                    } else {
                        DsaPolicyState::Watch
                    }
                } else if policy.promotes_alert && numeric_alert && hits > minimum_hits {
                    DsaPolicyState::Review
                } else {
                    DsaPolicyState::Watch
                }
            }
            HeuristicAlertClass::Review => {
                if requires_persistence && !numeric_alert {
                    if policy.suppresses_alert {
                        DsaPolicyState::Silent
                    } else {
                        DsaPolicyState::Watch
                    }
                } else {
                    DsaPolicyState::Review
                }
            }
            HeuristicAlertClass::Escalate => {
                if numeric_alert && local_corroboration {
                    DsaPolicyState::Escalate
                } else if numeric_alert {
                    DsaPolicyState::Review
                } else {
                    DsaPolicyState::Watch
                }
            }
        }
    };

    if contribution_state == DsaPolicyState::Review
        && policy.promotes_alert
        && numeric_alert
        && local_corroboration
        && raw_violation_recent
    {
        contribution_state = DsaPolicyState::Escalate;
    }

    Some(MotifContributionState {
        motif_name: policy.motif_name,
        default_alert_class: alert_class_default,
        contribution_state,
        fragmentation_proxy,
        suppressed_to_silent,
    })
}

fn apply_recall_rescue(
    feature_override: Option<&FeaturePolicyOverride>,
    rescue_config: &RecallRescueConfig,
    config: &DsaConfig,
    resolved_alert_class: &[HeuristicAlertClass],
    policy_state: &mut [DsaPolicyState],
    dsa_alert: &mut [bool],
    fragmentation_proxy_w: &[f64],
    dsa_score: &[f64],
    boundary_density_w: &[f64],
    ewma_occupancy_w: &[f64],
    motif_recurrence_w: &[f64],
    consistent: &[bool],
    run_index: usize,
) -> Option<&'static str> {
    let override_entry = feature_override?;
    if !override_entry.rescue_eligible {
        return None;
    }
    if policy_state[run_index].is_review_or_escalate() {
        return None;
    }
    if !matches!(
        resolved_alert_class[run_index],
        HeuristicAlertClass::Watch | HeuristicAlertClass::Review
    ) {
        return None;
    }
    let minimum_window = override_entry
        .minimum_window_override
        .unwrap_or(config.window);
    let minimum_hits = override_entry.minimum_hits_override.unwrap_or(3);
    let fragmentation_ceiling = override_entry
        .maximum_allowed_fragmentation_override
        .unwrap_or(0.5);
    let score_margin = if override_entry.rescue_priority >= 2 {
        rescue_config.priority_two_score_margin
    } else {
        rescue_config.priority_one_score_margin
    };
    let recent_start = run_index.saturating_sub(minimum_window.saturating_sub(1));
    let recent_watch_hits = (recent_start..=run_index)
        .filter(|&index| matches!(resolved_alert_class[index], HeuristicAlertClass::Watch))
        .count();
    let score_floor = (config.alert_tau - score_margin).max(0.0);
    let consistency_satisfied = consistent[run_index]
        || (override_entry.rescue_priority >= 2
            && matches!(resolved_alert_class[run_index], HeuristicAlertClass::Watch));

    if !consistency_satisfied
        || recent_watch_hits < minimum_hits
        || fragmentation_proxy_w[run_index] > fragmentation_ceiling
        || boundary_density_w[run_index] < rescue_config.minimum_boundary_density
        || motif_recurrence_w[run_index] < rescue_config.minimum_motif_recurrence
        || ewma_occupancy_w[run_index] < rescue_config.minimum_ewma_occupancy
        || dsa_score[run_index] < score_floor
    {
        return None;
    }

    if policy_state[run_index] == DsaPolicyState::Silent {
        policy_state[run_index] = DsaPolicyState::Review;
        dsa_alert[run_index] = true;
        return Some("watch_to_review");
    }

    if policy_state[run_index] == DsaPolicyState::Review {
        policy_state[run_index] = DsaPolicyState::Escalate;
        dsa_alert[run_index] = true;
        return Some("review_to_escalate");
    }

    None
}

fn dominant_alert_class(contributions: &[MotifContributionState]) -> HeuristicAlertClass {
    contributions
        .iter()
        .map(|contribution| contribution.default_alert_class)
        .max()
        .unwrap_or(HeuristicAlertClass::Silent)
}

fn dominant_policy_state(contributions: &[MotifContributionState]) -> DsaPolicyState {
    contributions
        .iter()
        .map(|contribution| contribution.contribution_state)
        .max()
        .unwrap_or(DsaPolicyState::Silent)
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

fn window_fraction_nonimputed(
    hit_prefix: &[usize],
    non_imputed_prefix: &[usize],
    start: usize,
    end: usize,
) -> f64 {
    let total_non_imputed = window_count(non_imputed_prefix, start, end);
    if total_non_imputed == 0 {
        0.0
    } else {
        window_count(hit_prefix, start, end) as f64 / total_non_imputed as f64
    }
}

fn window_mean(values: &[f64], start: usize, end: usize) -> f64 {
    let slice = &values[start..=end];
    slice.iter().sum::<f64>() / slice.len() as f64
}

fn window_is_consistent(drift: &[f64], drift_threshold: f64, start: usize, end: usize) -> bool {
    let mut previous_thresholded = 0i8;

    for run_index in start..=end {
        let thresholded_sign = if drift[run_index] >= drift_threshold {
            1
        } else if drift[run_index] <= -drift_threshold {
            -1
        } else {
            0
        };
        if thresholded_sign < 0 {
            return false;
        }
        if thresholded_sign != 0 {
            if previous_thresholded != 0 && thresholded_sign != previous_thresholded {
                return false;
            }
            previous_thresholded = thresholded_sign;
        }
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

fn build_run_signals(
    traces: &[DsaFeatureTrace],
    raw_violation_run_signal: &[bool],
    config: &DsaConfig,
    run_count: usize,
) -> DsaRunSignals {
    let mut any_feature_dsa_alert = Vec::with_capacity(run_count);
    let mut primary_run_alert = Vec::with_capacity(run_count);
    let mut feature_count_dsa_alert = Vec::with_capacity(run_count);
    let mut watch_feature_count = Vec::with_capacity(run_count);
    let mut review_feature_count = Vec::with_capacity(run_count);
    let mut escalate_feature_count = Vec::with_capacity(run_count);
    let mut strict_escalate_run_alert = Vec::with_capacity(run_count);
    let mut numeric_primary_run_alert = Vec::with_capacity(run_count);
    let mut numeric_feature_count_dsa_alert = Vec::with_capacity(run_count);

    for run_index in 0..run_count {
        let review_count = traces
            .iter()
            .filter(|trace| trace.dsa_alert[run_index])
            .count();
        let watch_count = traces
            .iter()
            .filter(|trace| trace.policy_state[run_index] == DsaPolicyState::Watch)
            .count();
        let escalate_count = traces
            .iter()
            .filter(|trace| trace.policy_state[run_index] == DsaPolicyState::Escalate)
            .count();
        let numeric_count = traces
            .iter()
            .filter(|trace| trace.numeric_dsa_alert[run_index])
            .count();
        feature_count_dsa_alert.push(review_count);
        watch_feature_count.push(watch_count);
        review_feature_count.push(review_count.saturating_sub(escalate_count));
        escalate_feature_count.push(escalate_count);
        numeric_feature_count_dsa_alert.push(numeric_count);
        strict_escalate_run_alert.push(escalate_count >= config.corroborating_feature_count_min);
        numeric_primary_run_alert.push(numeric_count >= config.corroborating_feature_count_min);
        let any_dsa = review_count > 0;
        any_feature_dsa_alert.push(any_dsa);
        primary_run_alert.push(review_count >= config.corroborating_feature_count_min);
    }

    DsaRunSignals {
        primary_run_signal: format!(
            "feature_count_review_or_escalate(k) >= {}",
            config.corroborating_feature_count_min
        ),
        corroborating_feature_count_min: config.corroborating_feature_count_min,
        primary_run_alert,
        any_feature_dsa_alert,
        any_feature_raw_violation: raw_violation_run_signal.to_vec(),
        feature_count_dsa_alert,
        watch_feature_count,
        review_feature_count,
        escalate_feature_count,
        strict_escalate_run_alert,
        numeric_primary_run_alert,
        numeric_feature_count_dsa_alert,
    }
}

fn compute_episode_summary(
    primary_signal_name: &str,
    dsa_signal: &[bool],
    raw_boundary_episode_count: usize,
    raw_violation_signal: &[bool],
    failure_window_mask: &[bool],
) -> DsaEpisodeSummary {
    let dsa_episodes = episode_ranges(dsa_signal);
    let dsa_lengths = dsa_episodes
        .iter()
        .map(|(start, end)| end - start + 1)
        .collect::<Vec<_>>();
    let dsa_episodes_preceding_failure = dsa_episodes
        .iter()
        .filter(|(start, end)| (*start..=*end).any(|run| failure_window_mask[run]))
        .count();
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
        dsa_episodes_preceding_failure,
        mean_dsa_episode_length_runs: mean_usize(&dsa_lengths),
        max_dsa_episode_length_runs: dsa_lengths.iter().copied().max().unwrap_or(0),
        compression_ratio: if dsa_episodes.is_empty() {
            None
        } else {
            Some(raw_boundary_episode_count as f64 / dsa_episodes.len() as f64)
        },
        precursor_quality: if dsa_episodes.is_empty() {
            None
        } else {
            Some(dsa_episodes_preceding_failure as f64 / dsa_episodes.len() as f64)
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
struct EarliestPrimarySignal {
    run_index: usize,
    feature_index: usize,
    feature_name: String,
    score: f64,
    source: String,
}

fn earliest_primary_signal(
    traces: &[DsaFeatureTrace],
    primary_signal: &[bool],
    start: usize,
    end: usize,
) -> Option<EarliestPrimarySignal> {
    let run_index = earliest_run_signal(primary_signal, start, end)?;
    traces
        .iter()
        .filter(|trace| trace.dsa_alert[run_index])
        .map(|trace| EarliestPrimarySignal {
            run_index,
            feature_index: trace.feature_index,
            feature_name: trace.feature_name.clone(),
            score: trace.dsa_score[run_index],
            source: "DSA".into(),
        })
        .max_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.feature_index.cmp(&left.feature_index))
        })
}

fn build_failure_window_mask(
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

#[derive(Debug, Clone)]
struct MaxDsaScore {
    feature_index: usize,
    feature_name: String,
    run_index: usize,
    score: f64,
    boundary_density_w: f64,
    drift_persistence_w: f64,
    slew_density_w: f64,
    ewma_occupancy_w: f64,
    motif_recurrence_w: f64,
    fragmentation_proxy_w: f64,
    consistent: bool,
    policy_state: DsaPolicyState,
    resolved_alert_class: HeuristicAlertClass,
    numeric_dsa_alert: bool,
    dsa_alert: bool,
    policy_suppressed_to_silent: bool,
    rescue_transition: String,
}

fn max_dsa_score(traces: &[DsaFeatureTrace], start: usize, end: usize) -> Option<MaxDsaScore> {
    let mut max_score: Option<MaxDsaScore> = None;
    for trace in traces {
        for run_index in start..end {
            let candidate = MaxDsaScore {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                run_index,
                score: trace.dsa_score[run_index],
                boundary_density_w: trace.boundary_density_w[run_index],
                drift_persistence_w: trace.drift_persistence_w[run_index],
                slew_density_w: trace.slew_density_w[run_index],
                ewma_occupancy_w: trace.ewma_occupancy_w[run_index],
                motif_recurrence_w: trace.motif_recurrence_w[run_index],
                fragmentation_proxy_w: trace.fragmentation_proxy_w[run_index],
                consistent: trace.consistent[run_index],
                policy_state: trace.policy_state[run_index],
                resolved_alert_class: trace.resolved_alert_class[run_index],
                numeric_dsa_alert: trace.numeric_dsa_alert[run_index],
                dsa_alert: trace.dsa_alert[run_index],
                policy_suppressed_to_silent: trace.policy_suppressed_to_silent[run_index],
                rescue_transition: trace.rescue_transition[run_index].clone(),
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
        mean_lead_delta_vs_cusum_runs: None,
        mean_lead_delta_vs_run_energy_runs: None,
        mean_lead_delta_vs_pca_fdc_runs: None,
        mean_lead_delta_vs_threshold_runs: None,
        mean_lead_delta_vs_ewma_runs: None,
    }
}

fn component_contributions(traces: &[DsaFeatureTrace]) -> Vec<DsaComponentContribution> {
    let total_points = traces
        .iter()
        .map(|trace| trace.dsa_score.len())
        .sum::<usize>()
        .max(1);
    let alert_points = traces
        .iter()
        .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>()
        .max(1);

    let component_rows = [
        (
            "boundary_density_W",
            traces
                .iter()
                .flat_map(|trace| trace.boundary_density_w.iter().copied())
                .sum::<f64>(),
            traces
                .iter()
                .map(|trace| {
                    trace
                        .boundary_density_w
                        .iter()
                        .zip(&trace.dsa_alert)
                        .filter_map(|(value, flag)| flag.then_some(*value))
                        .sum::<f64>()
                })
                .sum::<f64>(),
        ),
        (
            "drift_persistence_W",
            traces
                .iter()
                .flat_map(|trace| trace.drift_persistence_w.iter().copied())
                .sum::<f64>(),
            traces
                .iter()
                .map(|trace| {
                    trace
                        .drift_persistence_w
                        .iter()
                        .zip(&trace.dsa_alert)
                        .filter_map(|(value, flag)| flag.then_some(*value))
                        .sum::<f64>()
                })
                .sum::<f64>(),
        ),
        (
            "slew_density_W",
            traces
                .iter()
                .flat_map(|trace| trace.slew_density_w.iter().copied())
                .sum::<f64>(),
            traces
                .iter()
                .map(|trace| {
                    trace
                        .slew_density_w
                        .iter()
                        .zip(&trace.dsa_alert)
                        .filter_map(|(value, flag)| flag.then_some(*value))
                        .sum::<f64>()
                })
                .sum::<f64>(),
        ),
        (
            "ewma_occupancy_W",
            traces
                .iter()
                .flat_map(|trace| trace.ewma_occupancy_w.iter().copied())
                .sum::<f64>(),
            traces
                .iter()
                .map(|trace| {
                    trace
                        .ewma_occupancy_w
                        .iter()
                        .zip(&trace.dsa_alert)
                        .filter_map(|(value, flag)| flag.then_some(*value))
                        .sum::<f64>()
                })
                .sum::<f64>(),
        ),
        (
            "motif_recurrence_W",
            traces
                .iter()
                .flat_map(|trace| trace.motif_recurrence_w.iter().copied())
                .sum::<f64>(),
            traces
                .iter()
                .map(|trace| {
                    trace
                        .motif_recurrence_w
                        .iter()
                        .zip(&trace.dsa_alert)
                        .filter_map(|(value, flag)| flag.then_some(*value))
                        .sum::<f64>()
                })
                .sum::<f64>(),
        ),
    ];

    let mut out = component_rows
        .into_iter()
        .map(|(component, all_sum, alert_sum)| DsaComponentContribution {
            component: component.into(),
            mean_value_on_alert_points: alert_sum / alert_points as f64,
            mean_value_on_all_points: all_sum / total_points as f64,
            total_value_on_alert_points: alert_sum,
        })
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        right
            .mean_value_on_alert_points
            .partial_cmp(&left.mean_value_on_alert_points)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.component.cmp(&right.component))
    });
    out
}

fn aggregate_motif_policy_contributions(
    traces: &[DsaFeatureTrace],
) -> Vec<DsaMotifPolicyContribution> {
    let mut aggregated = BTreeMap::<String, DsaMotifPolicyContribution>::new();
    for trace in traces {
        for contribution in &trace.motif_policy_contributions {
            let entry = aggregated
                .entry(contribution.motif_name.clone())
                .or_insert_with(|| DsaMotifPolicyContribution {
                    motif_name: contribution.motif_name.clone(),
                    alert_class_default: contribution.alert_class_default,
                    watch_points: 0,
                    review_points: 0,
                    escalate_points: 0,
                    silent_suppression_points: 0,
                    pass_review_or_escalate_points: 0,
                    pre_failure_review_or_escalate_points: 0,
                });
            entry.watch_points += contribution.watch_points;
            entry.review_points += contribution.review_points;
            entry.escalate_points += contribution.escalate_points;
            entry.silent_suppression_points += contribution.silent_suppression_points;
            entry.pass_review_or_escalate_points += contribution.pass_review_or_escalate_points;
            entry.pre_failure_review_or_escalate_points +=
                contribution.pre_failure_review_or_escalate_points;
        }
    }

    aggregated.into_values().collect::<Vec<_>>()
}

fn success_condition_failures(
    dsa: &SignalComparisonRow,
    threshold: &SignalComparisonRow,
    ewma: &SignalComparisonRow,
) -> Vec<String> {
    let mut failures = Vec::new();
    if dsa.pass_run_nuisance_proxy >= ewma.pass_run_nuisance_proxy {
        failures.push(format!(
            "pass-run nuisance {:.4} is not below EWMA nuisance {:.4}",
            dsa.pass_run_nuisance_proxy, ewma.pass_run_nuisance_proxy
        ));
    }
    let recall_floor = threshold
        .failure_run_recall
        .saturating_sub(DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS);
    if dsa.failure_run_recall < recall_floor {
        failures.push(format!(
            "failure recall {}/{} is more than {} run(s) below threshold recall {}/{}",
            dsa.failure_run_recall,
            dsa.failure_runs,
            DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS,
            threshold.failure_run_recall,
            threshold.failure_runs,
        ));
    }
    failures
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
    dsfb_violation: &SignalComparisonRow,
    raw_boundary: &SignalComparisonRow,
    episode_summary: &DsaEpisodeSummary,
    primary_success_condition_met: bool,
    nuisance_improved: bool,
    lead_time_improved: bool,
    recall_preserved: bool,
    compression_improved: bool,
    nothing_improved: bool,
    component_contributions: &[DsaComponentContribution],
    success_condition_failures: &[String],
    validation_failures: &[String],
) -> String {
    let top_components = component_contributions
        .iter()
        .take(3)
        .map(|row| format!("{}={:.4}", row.component, row.mean_value_on_alert_points))
        .collect::<Vec<_>>()
        .join(", ");

    if primary_success_condition_met {
        if !validation_failures.is_empty() {
            return format!(
                "DSA meets the primary success condition: pass-run nuisance {:.4} is below EWMA nuisance {:.4}, failure recall is {}/{}, mean lead deltas are threshold={} and EWMA={}, precursor quality is {}, and compression ratio is {}. It still fails the stricter validation gates ({}), so no superiority claim is made and DSFB Violation remains the frozen instantaneous envelope-exit comparator.",
                dsa.pass_run_nuisance_proxy,
                ewma.pass_run_nuisance_proxy,
                dsa.failure_run_recall,
                dsa.failure_runs,
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
                format_option_f64(episode_summary.precursor_quality),
                format_option_f64(episode_summary.compression_ratio),
                validation_failures.join("; "),
            );
        }
        return format!(
            "DSA meets the primary success condition: pass-run nuisance {:.4} is below EWMA nuisance {:.4}, failure recall is {}/{}, mean lead deltas are threshold={} and EWMA={}, precursor quality is {}, and compression ratio is {}. DSFB Violation remains the frozen instantaneous envelope-exit comparator.",
            dsa.pass_run_nuisance_proxy,
            ewma.pass_run_nuisance_proxy,
            dsa.failure_run_recall,
            dsa.failure_runs,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            format_option_f64(episode_summary.precursor_quality),
            format_option_f64(episode_summary.compression_ratio),
        );
    }

    if !validation_failures.is_empty() {
        if nuisance_improved && !lead_time_improved {
            return format!(
                "DSA improves nuisance relative to at least one comparator but does not satisfy the primary success condition ({}) and also fails validation gates ({}). Recall is {}/{}, threshold recall is {}/{}, EWMA recall is {}/{}, DSFB Violation recall is {}/{}, raw-boundary nuisance delta is {:.4}, precursor quality is {}, compression ratio is {}, and the strongest DSA components were {}. No superiority claim is made.",
                success_condition_failures.join("; "),
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                threshold.failure_run_recall,
                threshold.failure_runs,
                ewma.failure_run_recall,
                ewma.failure_runs,
                dsfb_violation.failure_run_recall,
                dsfb_violation.failure_runs,
                dsa.pass_run_nuisance_proxy - raw_boundary.pass_run_nuisance_proxy,
                format_option_f64(episode_summary.precursor_quality),
                format_option_f64(episode_summary.compression_ratio),
                top_components,
            );
        }

        if nothing_improved {
            return format!(
                "DSA fails to improve nuisance, lead time, recall, or compression and fails both the primary success condition ({}) and validation gates ({}). Recall is {}/{}, pass-run nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}, precursor quality is {}, compression ratio is {}, and the strongest DSA components were {}.",
                success_condition_failures.join("; "),
                validation_failures.join("; "),
                dsa.failure_run_recall,
                dsa.failure_runs,
                dsa.pass_run_nuisance_proxy,
                format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
                format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
                format_option_f64(episode_summary.precursor_quality),
                format_option_f64(episode_summary.compression_ratio),
                top_components,
            );
        }

        return format!(
            "DSA shows mixed trade-offs but does not satisfy the primary success condition ({}) and fails validation gates ({}). Nuisance improved: {}, lead time improved: {}, recall preserved: {}, compression improved: {}, precursor quality is {}, and the strongest DSA components were {}. No superiority claim is made.",
            success_condition_failures.join("; "),
            validation_failures.join("; "),
            nuisance_improved,
            lead_time_improved,
            recall_preserved,
            compression_improved,
            format_option_f64(episode_summary.precursor_quality),
            top_components,
        );
    }

    if nuisance_improved && !lead_time_improved {
        return format!(
            "DSA reduces nuisance relative to at least one comparator, but it does not satisfy the primary success condition ({}). Recall is {}/{}, threshold recall is {}/{}, pass-run nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}, precursor quality is {}, compression ratio is {}, and the strongest DSA components were {}. No superiority claim is made.",
            success_condition_failures.join("; "),
            dsa.failure_run_recall,
            dsa.failure_runs,
            threshold.failure_run_recall,
            threshold.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            format_option_f64(episode_summary.precursor_quality),
            format_option_f64(episode_summary.compression_ratio),
            top_components,
        );
    }

    if nothing_improved {
        return format!(
            "DSA fails to improve nuisance, lead time, recall, or compression relative to the logged baselines, and it does not satisfy the primary success condition ({}). Recall is {}/{}, pass-run nuisance is {:.4}, mean lead deltas are threshold={} and EWMA={}, precursor quality is {}, compression ratio is {}, and the strongest DSA components were {}.",
            success_condition_failures.join("; "),
            dsa.failure_run_recall,
            dsa.failure_runs,
            dsa.pass_run_nuisance_proxy,
            format_option_f64(dsa.mean_lead_delta_vs_threshold_runs),
            format_option_f64(dsa.mean_lead_delta_vs_ewma_runs),
            format_option_f64(episode_summary.precursor_quality),
            format_option_f64(episode_summary.compression_ratio),
            top_components,
        );
    }

    format!(
        "DSA shows mixed trade-offs without satisfying the primary success condition ({}). Recall preserved: {}, nuisance improved: {}, lead time improved: {}, precursor quality is {}, compression ratio is {}, and the strongest DSA components were {}.",
        success_condition_failures.join("; "),
        recall_preserved,
        nuisance_improved,
        lead_time_improved,
        format_option_f64(episode_summary.precursor_quality),
        format_option_f64(episode_summary.compression_ratio),
        top_components,
    )
}

pub fn summarize_dsa_grid(rows: &[DsaCalibrationRow]) -> DsaGridSummary {
    let mut by_corroboration: BTreeMap<usize, Vec<DsaCalibrationRow>> = BTreeMap::new();
    for row in rows {
        by_corroboration
            .entry(row.corroborating_feature_count_min)
            .or_default()
            .push(row.clone());
    }
    let corroboration_summaries = by_corroboration
        .iter()
        .map(
            |(&corroborating_feature_count_min, grouped_rows)| DsaCorroborationSummary {
                corroborating_feature_count_min,
                representative_row: choose_closest_to_success(grouped_rows),
            },
        )
        .collect::<Vec<_>>();
    let success_row_count = rows
        .iter()
        .filter(|row| row.primary_success_condition_met)
        .count();
    DsaGridSummary {
        grid_point_count: rows.len(),
        optimization_priority_order: dsa_optimization_priority_order(),
        primary_success_condition_definition: dsa_primary_success_condition_definition(),
        success_row_count,
        any_success_row: success_row_count > 0,
        closest_to_success: choose_closest_to_success(rows),
        best_success_row: choose_best_success_row(rows),
        best_precursor_quality_row: choose_best_precursor_quality_row(rows),
        cross_feature_corroboration_effect: cross_feature_corroboration_effect(
            &corroboration_summaries,
        ),
        limiting_factor: limiting_factor(rows),
        corroboration_summaries,
    }
}

fn choose_closest_to_success(rows: &[DsaCalibrationRow]) -> Option<DsaCalibrationRow> {
    if let Some(best_success_row) = choose_best_success_row(rows) {
        return Some(best_success_row);
    }
    rows.iter()
        .cloned()
        .max_by(compare_dsa_rows_by_success_closeness)
}

fn choose_best_success_row(rows: &[DsaCalibrationRow]) -> Option<DsaCalibrationRow> {
    rows.iter()
        .filter(|row| row.primary_success_condition_met)
        .cloned()
        .max_by(compare_dsa_rows_by_priority)
}

fn choose_best_precursor_quality_row(rows: &[DsaCalibrationRow]) -> Option<DsaCalibrationRow> {
    rows.iter().cloned().max_by(|left, right| {
        compare_option_f64(left.precursor_quality, right.precursor_quality)
            .then_with(|| compare_dsa_rows_by_priority(left, right))
    })
}

fn compare_dsa_rows_by_priority(
    left: &DsaCalibrationRow,
    right: &DsaCalibrationRow,
) -> std::cmp::Ordering {
    compare_f64_lower_is_better(
        left.pass_run_nuisance_delta_vs_raw_boundary,
        right.pass_run_nuisance_delta_vs_raw_boundary,
    )
    .then_with(|| {
        compare_f64_lower_is_better(
            left.pass_run_nuisance_delta_vs_ewma,
            right.pass_run_nuisance_delta_vs_ewma,
        )
    })
    .then_with(|| left.failure_run_recall.cmp(&right.failure_run_recall))
    .then_with(|| {
        compare_option_f64(
            averaged_lead_delta(
                left.mean_lead_delta_vs_threshold_runs,
                left.mean_lead_delta_vs_ewma_runs,
            ),
            averaged_lead_delta(
                right.mean_lead_delta_vs_threshold_runs,
                right.mean_lead_delta_vs_ewma_runs,
            ),
        )
    })
    .then_with(|| compare_option_f64(left.precursor_quality, right.precursor_quality))
    .then_with(|| compare_option_f64(left.compression_ratio, right.compression_ratio))
    .then_with(|| {
        right
            .corroborating_feature_count_min
            .cmp(&left.corroborating_feature_count_min)
    })
}

fn compare_dsa_rows_by_success_closeness(
    left: &DsaCalibrationRow,
    right: &DsaCalibrationRow,
) -> std::cmp::Ordering {
    compare_usize_lower_is_better(
        unmet_primary_success_conditions(left),
        unmet_primary_success_conditions(right),
    )
    .then_with(|| {
        compare_usize_lower_is_better(
            recall_shortfall_vs_primary_success_floor(left),
            recall_shortfall_vs_primary_success_floor(right),
        )
    })
    .then_with(|| {
        compare_f64_lower_is_better(
            ewma_nuisance_gap_to_primary_success(left),
            ewma_nuisance_gap_to_primary_success(right),
        )
    })
    .then_with(|| compare_dsa_rows_by_priority(left, right))
}

fn unmet_primary_success_conditions(row: &DsaCalibrationRow) -> usize {
    usize::from(ewma_nuisance_gap_to_primary_success(row) > 0.0)
        + usize::from(recall_shortfall_vs_primary_success_floor(row) > 0)
}

fn recall_shortfall_vs_primary_success_floor(row: &DsaCalibrationRow) -> usize {
    let recall_floor = row
        .threshold_failure_run_recall
        .saturating_sub(DSA_PRIMARY_SUCCESS_RECALL_TOLERANCE_RUNS);
    recall_floor.saturating_sub(row.failure_run_recall)
}

fn ewma_nuisance_gap_to_primary_success(row: &DsaCalibrationRow) -> f64 {
    row.pass_run_nuisance_delta_vs_ewma.max(0.0)
}

fn compare_usize_lower_is_better(left: usize, right: usize) -> std::cmp::Ordering {
    right.cmp(&left)
}

fn compare_f64_lower_is_better(left: f64, right: f64) -> std::cmp::Ordering {
    right
        .partial_cmp(&left)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn compare_option_f64(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left
            .partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn averaged_lead_delta(threshold_delta: Option<f64>, ewma_delta: Option<f64>) -> Option<f64> {
    let mut values = Vec::new();
    if let Some(value) = threshold_delta {
        values.push(value);
    }
    if let Some(value) = ewma_delta {
        values.push(value);
    }
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn cross_feature_corroboration_effect(
    corroboration_summaries: &[DsaCorroborationSummary],
) -> String {
    let Some(base) = corroboration_summaries
        .iter()
        .find(|summary| summary.corroborating_feature_count_min == 2)
        .and_then(|summary| summary.representative_row.as_ref())
    else {
        return "Cross-feature corroboration effect could not be determined from the saved grid."
            .into();
    };

    let higher_rows = corroboration_summaries
        .iter()
        .filter(|summary| summary.corroborating_feature_count_min > 2)
        .filter_map(|summary| summary.representative_row.as_ref())
        .collect::<Vec<_>>();
    if higher_rows.is_empty() {
        return "Cross-feature corroboration effect could not be determined from the saved grid."
            .into();
    }

    let all_lower_nuisance = higher_rows.iter().all(|row| {
        row.pass_run_nuisance_delta_vs_raw_boundary < base.pass_run_nuisance_delta_vs_raw_boundary
            && row.pass_run_nuisance_delta_vs_ewma < base.pass_run_nuisance_delta_vs_ewma
    });
    let any_lower_recall = higher_rows
        .iter()
        .any(|row| row.failure_run_recall < base.failure_run_recall);
    let any_higher_recall = higher_rows
        .iter()
        .any(|row| row.failure_run_recall > base.failure_run_recall);

    if all_lower_nuisance && any_lower_recall {
        "Higher cross-feature corroboration reduced nuisance but degraded recall relative to m=2."
            .into()
    } else if all_lower_nuisance && !any_lower_recall {
        "Higher cross-feature corroboration reduced nuisance without reducing recall relative to m=2."
            .into()
    } else if !all_lower_nuisance && any_lower_recall {
        "Higher cross-feature corroboration degraded recall without a consistent nuisance benefit relative to m=2."
            .into()
    } else if any_higher_recall {
        "Higher cross-feature corroboration improved recall at some settings, but nuisance trade-offs remained mixed relative to m=2."
            .into()
    } else {
        "Cross-feature corroboration produced mixed nuisance and recall trade-offs across m=2,3,5."
            .into()
    }
}

fn limiting_factor(rows: &[DsaCalibrationRow]) -> String {
    let any_recall_gate_passed = rows.iter().any(|row| row.threshold_recall_gate_passed);
    let any_ewma_nuisance_success = rows.iter().any(|row| {
        row.primary_success_condition_met
            || !row.success_condition_failures.contains("EWMA nuisance")
    });
    if !any_recall_gate_passed {
        "Recall was the limiting factor across the saved grid.".into()
    } else if !any_ewma_nuisance_success {
        "Nuisance relative to EWMA was the limiting factor across the saved grid.".into()
    } else if rows.iter().any(|row| row.primary_success_condition_met) {
        "The saved grid contains at least one row that satisfies the primary success condition."
            .into()
    } else {
        "Both nuisance and recall remained limiting factors across different parts of the saved grid.".into()
    }
}

fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn count_present<I, T>(iter: I) -> usize
where
    I: Iterator<Item = Option<T>>,
{
    iter.filter(|value| value.is_some()).count()
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
    fn dsa_consistency_uses_thresholded_direction_regimes() {
        assert!(window_is_consistent(&[0.0, 0.2, 0.1, 0.0], 0.15, 0, 3));
        assert!(window_is_consistent(&[0.0, 0.2, -0.1, 0.3], 0.15, 0, 3));
        assert!(!window_is_consistent(&[0.0, 0.2, -0.2, 0.3], 0.15, 0, 3));
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
        assert_eq!(grid.grid_point_count(), 81);
    }

    #[test]
    fn ewma_normalization_is_clipped() {
        assert_eq!(normalize_to_threshold(5.0, 2.0), 1.0);
        assert_eq!(normalize_to_threshold(1.0, 2.0), 0.5);
        assert_eq!(normalize_to_threshold(1.0, 0.0), 0.0);
    }

    #[test]
    fn closest_to_success_prefers_small_recall_shortfall_over_zero_recall() {
        let summary = summarize_dsa_grid(&[
            DsaCalibrationRow {
                config_id: 0,
                primary_run_signal: "feature_count_dsa_alert(k) >= 2".into(),
                window: 5,
                persistence_runs: 2,
                alert_tau: 2.0,
                corroborating_feature_count_min: 2,
                failure_run_recall: 102,
                failure_runs: 104,
                threshold_failure_run_recall: 104,
                ewma_failure_run_recall: 104,
                failure_recall_delta_vs_threshold: -2,
                failure_recall_delta_vs_ewma: -2,
                mean_lead_time_runs: Some(19.2),
                median_lead_time_runs: Some(20.0),
                pass_run_nuisance_proxy: 0.94,
                mean_lead_delta_vs_cusum_runs: Some(-0.7),
                mean_lead_delta_vs_run_energy_runs: Some(2.8),
                mean_lead_delta_vs_pca_fdc_runs: Some(0.1),
                mean_lead_delta_vs_threshold_runs: Some(-0.6),
                mean_lead_delta_vs_ewma_runs: Some(-0.6),
                pass_run_nuisance_delta_vs_cusum: -0.06,
                pass_run_nuisance_delta_vs_run_energy: 0.41,
                pass_run_nuisance_delta_vs_pca_fdc: 0.01,
                pass_run_nuisance_delta_vs_threshold: -0.03,
                pass_run_nuisance_delta_vs_ewma: -0.04,
                pass_run_nuisance_delta_vs_raw_boundary: -0.05,
                raw_boundary_episode_count: 100,
                dsa_episode_count: 10,
                dsa_episodes_preceding_failure: 10,
                mean_dsa_episode_length_runs: Some(5.0),
                max_dsa_episode_length_runs: 8,
                compression_ratio: Some(10.0),
                precursor_quality: Some(1.0),
                non_escalating_dsa_episode_fraction: Some(0.0),
                nuisance_improved: true,
                lead_time_improved: false,
                recall_preserved: false,
                compression_improved: true,
                any_metric_improved: true,
                nothing_improved: false,
                threshold_recall_gate_passed: false,
                boundary_nuisance_gate_passed: true,
                primary_success_condition_met: true,
                validation_passed: false,
                success_condition_failures: String::new(),
                validation_failures: "recall shortfall".into(),
            },
            DsaCalibrationRow {
                config_id: 1,
                primary_run_signal: "feature_count_dsa_alert(k) >= 5".into(),
                window: 15,
                persistence_runs: 4,
                alert_tau: 3.0,
                corroborating_feature_count_min: 5,
                failure_run_recall: 0,
                failure_runs: 104,
                threshold_failure_run_recall: 104,
                ewma_failure_run_recall: 104,
                failure_recall_delta_vs_threshold: -104,
                failure_recall_delta_vs_ewma: -104,
                mean_lead_time_runs: None,
                median_lead_time_runs: None,
                pass_run_nuisance_proxy: 0.0,
                mean_lead_delta_vs_cusum_runs: None,
                mean_lead_delta_vs_run_energy_runs: None,
                mean_lead_delta_vs_pca_fdc_runs: None,
                mean_lead_delta_vs_threshold_runs: None,
                mean_lead_delta_vs_ewma_runs: None,
                pass_run_nuisance_delta_vs_cusum: -1.0,
                pass_run_nuisance_delta_vs_run_energy: -0.5,
                pass_run_nuisance_delta_vs_pca_fdc: -0.9,
                pass_run_nuisance_delta_vs_threshold: -0.97,
                pass_run_nuisance_delta_vs_ewma: -0.98,
                pass_run_nuisance_delta_vs_raw_boundary: -0.99,
                raw_boundary_episode_count: 100,
                dsa_episode_count: 0,
                dsa_episodes_preceding_failure: 0,
                mean_dsa_episode_length_runs: None,
                max_dsa_episode_length_runs: 0,
                compression_ratio: None,
                precursor_quality: None,
                non_escalating_dsa_episode_fraction: None,
                nuisance_improved: true,
                lead_time_improved: false,
                recall_preserved: false,
                compression_improved: false,
                any_metric_improved: true,
                nothing_improved: false,
                threshold_recall_gate_passed: false,
                boundary_nuisance_gate_passed: true,
                primary_success_condition_met: false,
                validation_passed: false,
                success_condition_failures: "recall shortfall".into(),
                validation_failures: "recall shortfall".into(),
            },
        ]);

        assert_eq!(summary.closest_to_success.unwrap().config_id, 0);
    }

    #[test]
    fn policy_suppression_is_triggered_for_fragmented_recurrent_hits() {
        let policy = heuristic_policy_definition("recurrent_boundary_approach").unwrap();
        let flags = vec![
            true, false, false, true, false, false, false, false, false, true,
        ];
        let contribution =
            motif_contribution_state(policy, None, &flags, 9, true, true, true, false).unwrap();

        assert_eq!(contribution.contribution_state, DsaPolicyState::Silent);
        assert!(contribution.suppressed_to_silent);
        assert!(contribution.fragmentation_proxy > policy.maximum_allowed_fragmentation());
    }

    #[test]
    fn silent_motif_promotes_to_escalate_under_numeric_and_violation_corroboration() {
        let policy = heuristic_policy_definition("transient_excursion").unwrap();
        let flags = vec![false, false, true, true, true];
        let contribution =
            motif_contribution_state(policy, None, &flags, 4, true, true, true, true).unwrap();

        assert_eq!(contribution.contribution_state, DsaPolicyState::Escalate);
        assert!(!contribution.suppressed_to_silent);
    }

    #[test]
    fn corroboration_counts_only_review_or_escalate_states() {
        let run_count = 2;
        let config = DsaConfig {
            corroborating_feature_count_min: 2,
            ..DsaConfig::default()
        };

        let mut review_trace = empty_trace(0, "S000", run_count);
        review_trace.dsa_alert = vec![true, false];
        review_trace.numeric_dsa_alert = vec![true, false];
        review_trace.policy_state = vec![DsaPolicyState::Review, DsaPolicyState::Silent];

        let mut escalate_trace = empty_trace(1, "S001", run_count);
        escalate_trace.dsa_alert = vec![true, false];
        escalate_trace.numeric_dsa_alert = vec![true, false];
        escalate_trace.policy_state = vec![DsaPolicyState::Escalate, DsaPolicyState::Silent];

        let mut watch_trace = empty_trace(2, "S002", run_count);
        watch_trace.dsa_alert = vec![false, false];
        watch_trace.numeric_dsa_alert = vec![true, true];
        watch_trace.policy_state = vec![DsaPolicyState::Watch, DsaPolicyState::Watch];

        let signals = build_run_signals(
            &[review_trace, escalate_trace, watch_trace],
            &[false, false],
            &config,
            run_count,
        );

        assert_eq!(signals.primary_run_alert, vec![true, false]);
        assert_eq!(signals.feature_count_dsa_alert, vec![2, 0]);
        assert_eq!(signals.watch_feature_count, vec![1, 1]);
        assert_eq!(signals.review_feature_count, vec![1, 0]);
        assert_eq!(signals.escalate_feature_count, vec![1, 0]);
        assert_eq!(signals.strict_escalate_run_alert, vec![false, false]);
        assert_eq!(signals.numeric_primary_run_alert, vec![true, false]);
        assert_eq!(signals.numeric_feature_count_dsa_alert, vec![3, 1]);
    }

    #[test]
    fn priority_two_rescue_can_recover_inconsistent_watch_near_miss() {
        let override_entry = FeaturePolicyOverride {
            feature_index: 133,
            feature_name: "S134".into(),
            alert_class_override: None,
            requires_persistence_override: Some(false),
            requires_corroboration_override: Some(false),
            minimum_window_override: Some(5),
            minimum_hits_override: Some(4),
            maximum_allowed_fragmentation_override: Some(0.5),
            rescue_eligible: true,
            rescue_priority: 2,
            override_reason: "test".into(),
        };
        let rescue = RecallRescueConfig::default();
        let config = DsaConfig {
            alert_tau: 2.0,
            ..DsaConfig::default()
        };
        let resolved = vec![
            HeuristicAlertClass::Silent,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
        ];
        let mut policy_state = vec![DsaPolicyState::Silent; 5];
        let mut dsa_alert = vec![false; 5];
        let fragmentation = vec![0.0, 0.25, 0.25, 0.25, 0.25];
        let dsa_score = vec![0.0, 1.3, 1.45, 1.55, 1.70];
        let boundary = vec![0.0, 0.2, 0.3, 0.4, 0.4];
        let ewma = vec![0.2, 0.5, 0.6, 0.66, 0.70];
        let motif = vec![0.0, 0.2, 0.3, 0.4, 0.4];
        let consistent = vec![true, true, true, false, false];

        let transition = apply_recall_rescue(
            Some(&override_entry),
            &rescue,
            &config,
            &resolved,
            &mut policy_state,
            &mut dsa_alert,
            &fragmentation,
            &dsa_score,
            &boundary,
            &ewma,
            &motif,
            &consistent,
            4,
        );

        assert_eq!(transition, Some("watch_to_review"));
        assert_eq!(policy_state[4], DsaPolicyState::Review);
        assert!(dsa_alert[4]);
    }

    #[test]
    fn rescue_respects_feature_specific_fragmentation_override() {
        let override_entry = FeaturePolicyOverride {
            feature_index: 274,
            feature_name: "S275".into(),
            alert_class_override: None,
            requires_persistence_override: Some(false),
            requires_corroboration_override: Some(false),
            minimum_window_override: Some(5),
            minimum_hits_override: Some(4),
            maximum_allowed_fragmentation_override: Some(1.0),
            rescue_eligible: true,
            rescue_priority: 2,
            override_reason: "test".into(),
        };
        let rescue = RecallRescueConfig::default();
        let config = DsaConfig {
            alert_tau: 2.0,
            ..DsaConfig::default()
        };
        let resolved = vec![
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
            HeuristicAlertClass::Watch,
        ];
        let mut policy_state = vec![DsaPolicyState::Silent; 5];
        let mut dsa_alert = vec![false; 5];
        let fragmentation = vec![1.0; 5];
        let dsa_score = vec![1.65, 1.70, 1.74, 1.79, 1.83];
        let boundary = vec![0.4, 0.4, 0.4, 0.5, 0.5];
        let ewma = vec![0.68, 0.69, 0.70, 0.71, 0.72];
        let motif = vec![0.4, 0.4, 0.4, 0.5, 0.5];
        let consistent = vec![true; 5];

        let transition = apply_recall_rescue(
            Some(&override_entry),
            &rescue,
            &config,
            &resolved,
            &mut policy_state,
            &mut dsa_alert,
            &fragmentation,
            &dsa_score,
            &boundary,
            &ewma,
            &motif,
            &consistent,
            4,
        );

        assert_eq!(transition, Some("watch_to_review"));
        assert_eq!(policy_state[4], DsaPolicyState::Review);
        assert!(dsa_alert[4]);
    }
}
