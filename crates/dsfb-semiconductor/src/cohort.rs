//! Deterministic feature-cohort DSA selection and rating-delta forecast.

use crate::baselines::BaselineSet;
use crate::error::Result;
use crate::heuristics::{FeaturePolicyOverride, HeuristicAlertClass};
use crate::metrics::BenchmarkMetrics;
use crate::nominal::NominalModel;
use crate::precursor::{
    evaluate_dsa, evaluate_dsa_with_policy, project_dsa_to_cohort, DsaConfig, DsaEvaluation,
    DsaPolicyRuntime, RecallRescueConfig,
};
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use crate::{error::DsfbSemiconductorError, grammar::GrammarSet};
use csv::Writer;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::Path;

const RANKING_FORMULA: &str =
    "candidate_score = z(dsfb_raw_boundary_points) - z(dsfb_raw_violation_points) + z(ewma_alarm_points) - I(missing_fraction > 0.50) * 2.0";
const RECALL_AWARE_RANKING_FORMULA: &str =
    "candidate_score_recall = z(pre_failure_run_hits) + z(motif_precision_proxy) + z(ewma_alarm_points) + 0.5 * z(dsfb_raw_boundary_points) + 0.5 * z(recall_rescue_contribution) - 0.5 * z(dsfb_raw_violation_points) - I(missing_fraction > 0.50) * 2.0";
const MISSINGNESS_PENALTY_THRESHOLD: f64 = 0.50;
const MISSINGNESS_PENALTY_VALUE: f64 = 2.0;
const RECALL_TOLERANCE: usize = 1;
const PRIMARY_DELTA_TARGET: f64 = 0.40;
const SECONDARY_DELTA_TARGET: f64 = 0.40;
const CORROBORATION_SWEEP: &[usize] = &[1, 2, 3, 5];
const DSA_WINDOW_SWEEP: &[usize] = &[5, 10, 15];
const DSA_PERSISTENCE_SWEEP: &[usize] = &[2, 3, 4];
const DSA_TAU_SWEEP: &[f64] = &[2.0, 2.5, 3.0];
const CURRENT_BASELINE_SCORE: f64 = 8.1;
const FORECAST_PRIMARY_ONLY: f64 = 8.8;
const FORECAST_PRIMARY_PLUS_SECONDARY: f64 = 9.1;
const FORECAST_RECALL_SHORTFALL_VALUE: f64 = 8.3;
const SEED_FEATURES: &[&str] = &["S059", "S044", "S061", "S222", "S354", "S173"];
const OPTIMIZATION_RESCUE_WINDOW: usize = 5;
const OPTIMIZATION_RESCUE_MIN_HITS: usize = 4;
const OPTIMIZATION_RESCUE_FRAGMENTATION: f64 = 0.5;
const OPTIMIZATION_OVERRIDE_MAX_MISSINGNESS: f64 = 0.05;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureRankingRow {
    pub ranking_strategy: String,
    pub ranking_formula: String,
    pub feature_index: usize,
    pub feature_name: String,
    pub dsfb_raw_boundary_points: usize,
    pub dsfb_persistent_boundary_points: usize,
    pub dsfb_raw_violation_points: usize,
    pub dsfb_persistent_violation_points: usize,
    pub ewma_alarm_points: usize,
    pub threshold_alarm_points: usize,
    pub pre_failure_run_hits: usize,
    pub motif_precision_proxy: Option<f64>,
    pub recall_rescue_contribution: Option<f64>,
    pub missing_fraction: f64,
    pub z_pre_failure_run_hits: Option<f64>,
    pub z_motif_precision_proxy: Option<f64>,
    pub z_recall_rescue_contribution: Option<f64>,
    pub z_boundary: f64,
    pub z_violation: f64,
    pub z_ewma: f64,
    pub missingness_penalty: f64,
    pub candidate_score: f64,
    pub score_breakdown: String,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureRankingComparisonRow {
    pub feature_index: usize,
    pub feature_name: String,
    pub compression_rank: Option<usize>,
    pub recall_aware_rank: Option<usize>,
    pub compression_score: Option<f64>,
    pub recall_aware_score: Option<f64>,
    pub rank_delta_recall_minus_compression: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortMember {
    pub feature_index: usize,
    pub feature_name: String,
    pub ranking_score: f64,
    pub dsfb_boundary_points: usize,
    pub dsfb_violation_points: usize,
    pub ewma_alarm_points: usize,
    pub threshold_alarm_points: usize,
    pub missing_fraction: f64,
    pub reason_for_inclusion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedFeatureReport {
    pub feature_name: String,
    pub found_in_ranking: bool,
    pub rank: Option<usize>,
    pub candidate_score: Option<f64>,
    pub in_top_4: bool,
    pub in_top_8: bool,
    pub in_top_16: bool,
    pub top_4_note: String,
    pub top_8_note: String,
    pub top_16_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedFeatureCheckArtifact {
    pub ranking_formula: String,
    pub requested_seed_features: Vec<String>,
    pub seed_feature_report: Vec<SeedFeatureReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureCohorts {
    pub ranking_formula: String,
    pub missingness_penalty_threshold: f64,
    pub missingness_penalty_value: f64,
    pub top_4: Vec<CohortMember>,
    pub top_8: Vec<CohortMember>,
    pub top_16: Vec<CohortMember>,
    pub all_features: Vec<CohortMember>,
    pub seed_feature_report: Vec<SeedFeatureReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortGridResult {
    pub ranking_strategy: String,
    pub ranking_formula: String,
    pub grid_row_id: usize,
    pub feature_trace_config_id: usize,
    pub cohort_name: String,
    pub cohort_size: usize,
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub corroborating_m: usize,
    pub primary_run_signal: String,
    pub failure_recall: usize,
    pub failure_runs: usize,
    pub failure_recall_rate: f64,
    pub threshold_recall: usize,
    pub ewma_recall: usize,
    pub failure_recall_delta_vs_threshold: i64,
    pub failure_recall_delta_vs_ewma: i64,
    pub mean_lead_time_runs: Option<f64>,
    pub median_lead_time_runs: Option<f64>,
    pub threshold_mean_lead_time_runs: Option<f64>,
    pub ewma_mean_lead_time_runs: Option<f64>,
    pub mean_lead_delta_vs_threshold_runs: Option<f64>,
    pub mean_lead_delta_vs_ewma_runs: Option<f64>,
    pub pass_run_nuisance_proxy: f64,
    pub numeric_pass_run_nuisance_proxy: f64,
    pub ewma_nuisance: f64,
    pub threshold_nuisance: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_threshold: f64,
    pub pass_run_nuisance_delta_vs_numeric_dsa: f64,
    pub raw_boundary_episode_count: usize,
    pub dsa_episode_count: usize,
    pub dsa_episodes_preceding_failure: usize,
    pub mean_dsa_episode_length_runs: Option<f64>,
    pub max_dsa_episode_length_runs: usize,
    pub compression_ratio: Option<f64>,
    pub precursor_quality: Option<f64>,
    pub non_escalating_dsa_episode_fraction: Option<f64>,
    pub feature_level_active_points: usize,
    pub feature_level_alert_points: usize,
    pub persistence_suppression_fraction: Option<f64>,
    pub numeric_failure_recall: usize,
    pub policy_vs_numeric_recall_delta: i64,
    pub watch_point_count: usize,
    pub review_point_count: usize,
    pub escalate_point_count: usize,
    pub silenced_point_count: usize,
    pub rescued_point_count: usize,
    pub rescued_watch_to_review_points: usize,
    pub rescued_review_to_escalate_points: usize,
    pub primary_success: bool,
    pub primary_success_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeaturePolicySummaryRow {
    pub feature_index: usize,
    pub feature_name: String,
    pub compression_rank: Option<usize>,
    pub recall_aware_rank: Option<usize>,
    pub pre_failure_run_hits: usize,
    pub motif_precision_proxy: Option<f64>,
    pub missing_fraction: f64,
    pub rescue_eligible: bool,
    pub rescue_priority: usize,
    pub alert_class_override: Option<HeuristicAlertClass>,
    pub requires_persistence_override: Option<bool>,
    pub requires_corroboration_override: Option<bool>,
    pub minimum_window_override: Option<usize>,
    pub minimum_hits_override: Option<usize>,
    pub maximum_allowed_fragmentation_override: Option<f64>,
    pub override_reason: String,
    pub allow_watch_only: Option<bool>,
    pub allow_review_without_escalate: Option<bool>,
    pub suppress_if_isolated: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallRescueResultRow {
    pub ranking_strategy: String,
    pub cohort_name: String,
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub corroborating_m: usize,
    pub failure_recall: usize,
    pub pass_run_nuisance_proxy: f64,
    pub rescued_point_count: usize,
    pub rescued_watch_to_review_points: usize,
    pub rescued_review_to_escalate_points: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissedFailureDiagnosticRow {
    pub failure_run_index: usize,
    pub nearest_feature_name: Option<String>,
    pub nearest_feature_score: Option<f64>,
    pub nearest_feature_policy_state: Option<String>,
    pub nearest_feature_resolved_alert_class: Option<String>,
    pub nearest_feature_boundary_density_w: Option<f64>,
    pub nearest_feature_ewma_occupancy_w: Option<f64>,
    pub nearest_feature_motif_recurrence_w: Option<f64>,
    pub nearest_feature_fragmentation_proxy_w: Option<f64>,
    pub nearest_feature_consistent: Option<bool>,
    pub ranking_exclusion: bool,
    pub cohort_selection: bool,
    pub policy_suppression: bool,
    pub fragmentation_ceiling: bool,
    pub directional_consistency_gate: bool,
    pub persistence_gate: bool,
    pub corroboration_threshold: bool,
    pub rescue_gate_not_activating: bool,
    pub exact_miss_rule: String,
    pub bounded_rescue_would_recover: bool,
    pub recovered_after_optimization: bool,
    pub optimized_feature_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallCriticalFeatureRow {
    pub failure_run_index: usize,
    pub feature_index: Option<usize>,
    pub feature_name: Option<String>,
    pub compression_rank: Option<usize>,
    pub recall_aware_rank: Option<usize>,
    pub max_structural_score: Option<f64>,
    pub resolved_alert_class: Option<String>,
    pub policy_state: Option<String>,
    pub boundary_density_w: Option<f64>,
    pub ewma_occupancy_w: Option<f64>,
    pub motif_recurrence_w: Option<f64>,
    pub fragmentation_proxy_w: Option<f64>,
    pub consistent: Option<bool>,
    pub exact_miss_rule: String,
    pub feature_override_exists: bool,
    pub rescue_priority: Option<usize>,
    pub allow_review_without_escalate: Option<bool>,
    pub bounded_feature_override_would_recover: bool,
    pub recovered_after_optimization: bool,
    pub optimized_feature_name: Option<String>,
    pub recall_rescue_contribution: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyContributionAnalysisRow {
    pub configuration_role: String,
    pub contribution_type: String,
    pub name: String,
    pub value: f64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortMotifPolicyContributionRow {
    pub grid_row_id: usize,
    pub cohort_name: String,
    pub cohort_size: usize,
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub corroborating_m: usize,
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
pub struct CohortBestRow {
    pub cohort_name: String,
    pub best_row: CohortGridResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortFailureAnalysis {
    pub closest_cohort: String,
    pub closest_grid_point: String,
    pub closest_policy_setting: String,
    pub closest_nuisance: f64,
    pub closest_recall: usize,
    pub ewma_nuisance: f64,
    pub threshold_recall: usize,
    pub limiting_factor: String,
    pub corroboration_effect: String,
    pub policy_vs_numeric_note: String,
    pub ranking_quality_note: String,
    pub all_feature_dsa_vs_cohort_note: String,
    pub best_near_success_source: String,
    pub nuisance_motif_classes: String,
    pub useful_precursor_motif_classes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortGridSummary {
    pub ranking_formula: String,
    pub primary_success_condition_definition: String,
    pub recall_tolerance_runs: usize,
    pub grid_point_count: usize,
    pub optimization_priority_order: Vec<String>,
    pub success_row_count: usize,
    pub any_success_row: bool,
    pub closest_to_success: Option<CohortGridResult>,
    pub best_success_row: Option<CohortGridResult>,
    pub best_precursor_quality_row: Option<CohortGridResult>,
    pub cross_feature_corroboration_effect: String,
    pub limiting_factor: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CohortDsaSummary {
    pub ranking_formula: String,
    pub primary_success_condition: String,
    pub recall_tolerance_runs: usize,
    pub cohort_results: Vec<CohortGridResult>,
    pub best_by_cohort: Vec<CohortBestRow>,
    pub closest_to_success: Option<CohortGridResult>,
    pub best_primary_success: Option<CohortGridResult>,
    pub best_precursor_quality_row: Option<CohortGridResult>,
    pub selected_configuration: Option<CohortGridResult>,
    pub best_cohort: Option<String>,
    pub any_primary_success: bool,
    pub failure_analysis: Option<CohortFailureAnalysis>,
    pub grid_point_count: usize,
    pub optimization_priority_order: Vec<String>,
    pub cross_feature_corroboration_effect: String,
    pub limiting_factor: String,
}

#[derive(Debug, Clone)]
pub struct CohortExecution {
    pub grid_summary: CohortGridSummary,
    pub summary: CohortDsaSummary,
    pub motif_policy_contributions: Vec<CohortMotifPolicyContributionRow>,
    pub selected_evaluation: DsaEvaluation,
}

#[derive(Debug, Clone)]
pub struct OptimizationExecution {
    pub baseline_feature_ranking: Vec<FeatureRankingRow>,
    pub baseline_feature_cohorts: FeatureCohorts,
    pub baseline_execution: CohortExecution,
    pub recall_aware_feature_ranking: Vec<FeatureRankingRow>,
    pub ranking_comparison: Vec<FeatureRankingComparisonRow>,
    pub recall_aware_feature_cohorts: FeatureCohorts,
    pub feature_policy_overrides: Vec<FeaturePolicyOverride>,
    pub feature_policy_summary: Vec<FeaturePolicySummaryRow>,
    pub optimized_execution: CohortExecution,
    pub recall_aware_execution: CohortExecution,
    pub pareto_frontier: Vec<CohortGridResult>,
    pub stage_a_candidates: Vec<CohortGridResult>,
    pub stage_b_candidates: Vec<CohortGridResult>,
    pub recall_rescue_results: Vec<RecallRescueResultRow>,
    pub missed_failure_diagnostics: Vec<MissedFailureDiagnosticRow>,
    pub recall_critical_features: Vec<RecallCriticalFeatureRow>,
    pub policy_contribution_analysis: Vec<PolicyContributionAnalysisRow>,
    pub delta_target_assessment: DeltaTargetAssessment,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaCandidateSummary {
    pub configuration: String,
    pub ranking_strategy: String,
    pub cohort_name: String,
    pub window: usize,
    pub persistence_runs: usize,
    pub alert_tau: f64,
    pub corroborating_m: usize,
    pub failure_recall: usize,
    pub failure_runs: usize,
    pub pass_run_nuisance_proxy: f64,
    pub delta_nuisance_vs_ewma: f64,
    pub delta_nuisance_vs_current_dsa: f64,
    pub mean_lead_time_runs: Option<f64>,
    pub precursor_quality: Option<f64>,
    pub compression_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaTargetAssessment {
    pub primary_target_definition: String,
    pub secondary_target_definition: String,
    pub ewma_nuisance_baseline: f64,
    pub current_policy_dsa_nuisance_baseline: f64,
    pub primary_delta_target: f64,
    pub secondary_delta_target: f64,
    pub primary_target_nuisance_ceiling: f64,
    pub secondary_target_nuisance_ceiling: f64,
    pub selected_configuration: DeltaCandidateSummary,
    pub primary_target_met: bool,
    pub ideal_target_met: bool,
    pub secondary_target_met: bool,
    pub mean_lead_time_ge_ewma: bool,
    pub mean_lead_time_ge_threshold: bool,
    pub best_recall_103_candidate: Option<DeltaCandidateSummary>,
    pub best_recall_104_candidate: Option<DeltaCandidateSummary>,
    pub best_secondary_target_candidate: Option<DeltaCandidateSummary>,
    pub best_stage_a_delta_candidate: Option<DeltaCandidateSummary>,
    pub best_reachable_pareto_point: DeltaCandidateSummary,
    pub assessment_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryForecast {
    pub category: String,
    pub current: String,
    pub forecast: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForecastSupportingMetrics {
    pub chosen_configuration: String,
    pub dsa_nuisance: f64,
    pub ewma_nuisance: f64,
    pub dsa_recall: usize,
    pub threshold_recall: usize,
    pub recall_tolerance_runs: usize,
    pub dsa_mean_lead_time_runs: Option<f64>,
    pub ewma_mean_lead_time_runs: Option<f64>,
    pub threshold_mean_lead_time_runs: Option<f64>,
    pub dsa_precursor_quality: Option<f64>,
    pub all_feature_dsa_precursor_quality: Option<f64>,
    pub dsa_compression_ratio: Option<f64>,
    pub all_feature_dsa_compression_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatingDeltaForecast {
    pub current_baseline_score: f64,
    pub primary_success_condition: String,
    pub recall_tolerance_runs: usize,
    pub chosen_configuration: String,
    pub primary_success_met: bool,
    pub secondary_targets_met: bool,
    pub secondary_lead_time_vs_ewma: bool,
    pub secondary_lead_time_vs_threshold: bool,
    pub secondary_precursor_quality_vs_all_feature_dsa: Option<bool>,
    pub secondary_compression_material: Option<bool>,
    pub forecast_score_if_primary_success_only: f64,
    pub forecast_score_if_primary_plus_secondary_success: f64,
    pub achieved_forecast_score: f64,
    pub forecast_justification: String,
    pub category_forecasts: Vec<CategoryForecast>,
    pub supporting_metrics: ForecastSupportingMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatingDeltaFailureAnalysis {
    pub closest_configuration: String,
    pub dsa_nuisance: f64,
    pub ewma_nuisance: f64,
    pub dsa_recall: usize,
    pub threshold_recall: usize,
    pub recall_tolerance_runs: usize,
    pub nuisance_gap: f64,
    pub recall_gap_runs: i64,
    pub nuisance_missed_by: String,
    pub recall_preserved: bool,
    pub limiting_factor: String,
}

pub fn compute_feature_ranking(metrics: &BenchmarkMetrics) -> Vec<FeatureRankingRow> {
    let analyzable = metrics
        .feature_metrics
        .iter()
        .filter(|feature| feature.analyzable)
        .collect::<Vec<_>>();
    if analyzable.is_empty() {
        return Vec::new();
    }

    let boundary_values = analyzable
        .iter()
        .map(|feature| feature.dsfb_raw_boundary_points as f64)
        .collect::<Vec<_>>();
    let violation_values = analyzable
        .iter()
        .map(|feature| feature.dsfb_raw_violation_points as f64)
        .collect::<Vec<_>>();
    let ewma_values = analyzable
        .iter()
        .map(|feature| feature.ewma_alarm_points as f64)
        .collect::<Vec<_>>();

    let (boundary_mean, boundary_std) = mean_std(&boundary_values);
    let (violation_mean, violation_std) = mean_std(&violation_values);
    let (ewma_mean, ewma_std) = mean_std(&ewma_values);

    let mut ranking = analyzable
        .iter()
        .map(|feature| {
            let z_boundary = z_score(
                feature.dsfb_raw_boundary_points as f64,
                boundary_mean,
                boundary_std,
            );
            let z_violation = z_score(
                feature.dsfb_raw_violation_points as f64,
                violation_mean,
                violation_std,
            );
            let z_ewma = z_score(feature.ewma_alarm_points as f64, ewma_mean, ewma_std);
            let missingness_penalty = if feature.missing_fraction > MISSINGNESS_PENALTY_THRESHOLD {
                MISSINGNESS_PENALTY_VALUE
            } else {
                0.0
            };
            let candidate_score = z_boundary - z_violation + z_ewma - missingness_penalty;

            FeatureRankingRow {
                ranking_strategy: "compression_biased".into(),
                ranking_formula: RANKING_FORMULA.into(),
                feature_index: feature.feature_index,
                feature_name: feature.feature_name.clone(),
                dsfb_raw_boundary_points: feature.dsfb_raw_boundary_points,
                dsfb_persistent_boundary_points: feature.dsfb_persistent_boundary_points,
                dsfb_raw_violation_points: feature.dsfb_raw_violation_points,
                dsfb_persistent_violation_points: feature.dsfb_persistent_violation_points,
                ewma_alarm_points: feature.ewma_alarm_points,
                threshold_alarm_points: feature.threshold_alarm_points,
                pre_failure_run_hits: feature.pre_failure_run_hits,
                motif_precision_proxy: feature.motif_precision_proxy,
                recall_rescue_contribution: None,
                missing_fraction: feature.missing_fraction,
                z_pre_failure_run_hits: None,
                z_motif_precision_proxy: None,
                z_recall_rescue_contribution: None,
                z_boundary,
                z_violation,
                z_ewma,
                missingness_penalty,
                candidate_score,
                score_breakdown: format!(
                    "{:+.4} boundary - {:+.4} violation + {:+.4} ewma - {:.1} missingness",
                    z_boundary, z_violation, z_ewma, missingness_penalty
                ),
                rank: 0,
            }
        })
        .collect::<Vec<_>>();

    ranking.sort_by(|left, right| {
        right
            .candidate_score
            .partial_cmp(&left.candidate_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.feature_name.cmp(&right.feature_name))
    });

    for (index, row) in ranking.iter_mut().enumerate() {
        row.rank = index + 1;
    }

    ranking
}

pub fn compute_feature_ranking_recall_aware(
    metrics: &BenchmarkMetrics,
    recall_rescue_contributions: &BTreeMap<usize, f64>,
) -> Vec<FeatureRankingRow> {
    let analyzable = metrics
        .feature_metrics
        .iter()
        .filter(|feature| feature.analyzable)
        .collect::<Vec<_>>();
    if analyzable.is_empty() {
        return Vec::new();
    }

    let pre_failure_values = analyzable
        .iter()
        .map(|feature| feature.pre_failure_run_hits as f64)
        .collect::<Vec<_>>();
    let motif_precision_values = analyzable
        .iter()
        .map(|feature| feature.motif_precision_proxy.unwrap_or(0.0))
        .collect::<Vec<_>>();
    let ewma_values = analyzable
        .iter()
        .map(|feature| feature.ewma_alarm_points as f64)
        .collect::<Vec<_>>();
    let boundary_values = analyzable
        .iter()
        .map(|feature| feature.dsfb_raw_boundary_points as f64)
        .collect::<Vec<_>>();
    let violation_values = analyzable
        .iter()
        .map(|feature| feature.dsfb_raw_violation_points as f64)
        .collect::<Vec<_>>();
    let recall_rescue_values = analyzable
        .iter()
        .map(|feature| {
            recall_rescue_contributions
                .get(&feature.feature_index)
                .copied()
                .unwrap_or(0.0)
        })
        .collect::<Vec<_>>();

    let (pre_failure_mean, pre_failure_std) = mean_std(&pre_failure_values);
    let (motif_precision_mean, motif_precision_std) = mean_std(&motif_precision_values);
    let (ewma_mean, ewma_std) = mean_std(&ewma_values);
    let (boundary_mean, boundary_std) = mean_std(&boundary_values);
    let (violation_mean, violation_std) = mean_std(&violation_values);
    let (recall_rescue_mean, recall_rescue_std) = mean_std(&recall_rescue_values);

    let mut ranking = analyzable
        .iter()
        .map(|feature| {
            let z_pre_failure_run_hits = z_score(
                feature.pre_failure_run_hits as f64,
                pre_failure_mean,
                pre_failure_std,
            );
            let z_motif_precision_proxy = z_score(
                feature.motif_precision_proxy.unwrap_or(0.0),
                motif_precision_mean,
                motif_precision_std,
            );
            let z_ewma = z_score(feature.ewma_alarm_points as f64, ewma_mean, ewma_std);
            let z_boundary = z_score(
                feature.dsfb_raw_boundary_points as f64,
                boundary_mean,
                boundary_std,
            );
            let z_violation = z_score(
                feature.dsfb_raw_violation_points as f64,
                violation_mean,
                violation_std,
            );
            let recall_rescue_contribution = recall_rescue_contributions
                .get(&feature.feature_index)
                .copied()
                .unwrap_or(0.0);
            let z_recall_rescue_contribution = z_score(
                recall_rescue_contribution,
                recall_rescue_mean,
                recall_rescue_std,
            );
            let missingness_penalty = if feature.missing_fraction > MISSINGNESS_PENALTY_THRESHOLD {
                MISSINGNESS_PENALTY_VALUE
            } else {
                0.0
            };
            let candidate_score = z_pre_failure_run_hits
                + z_motif_precision_proxy
                + z_ewma
                + 0.5 * z_boundary
                + 0.5 * z_recall_rescue_contribution
                - 0.5 * z_violation
                - missingness_penalty;

            FeatureRankingRow {
                ranking_strategy: "recall_aware".into(),
                ranking_formula: RECALL_AWARE_RANKING_FORMULA.into(),
                feature_index: feature.feature_index,
                feature_name: feature.feature_name.clone(),
                dsfb_raw_boundary_points: feature.dsfb_raw_boundary_points,
                dsfb_persistent_boundary_points: feature.dsfb_persistent_boundary_points,
                dsfb_raw_violation_points: feature.dsfb_raw_violation_points,
                dsfb_persistent_violation_points: feature.dsfb_persistent_violation_points,
                ewma_alarm_points: feature.ewma_alarm_points,
                threshold_alarm_points: feature.threshold_alarm_points,
                pre_failure_run_hits: feature.pre_failure_run_hits,
                motif_precision_proxy: feature.motif_precision_proxy,
                recall_rescue_contribution: Some(recall_rescue_contribution),
                missing_fraction: feature.missing_fraction,
                z_pre_failure_run_hits: Some(z_pre_failure_run_hits),
                z_motif_precision_proxy: Some(z_motif_precision_proxy),
                z_recall_rescue_contribution: Some(z_recall_rescue_contribution),
                z_boundary,
                z_violation,
                z_ewma,
                missingness_penalty,
                candidate_score,
                score_breakdown: format!(
                    "{:+.4} pre_failure + {:+.4} motif_precision + {:+.4} ewma + 0.5*{:+.4} boundary + 0.5*{:+.4} recall_rescue - 0.5*{:+.4} violation - {:.1} missingness",
                    z_pre_failure_run_hits,
                    z_motif_precision_proxy,
                    z_ewma,
                    z_boundary,
                    z_recall_rescue_contribution,
                    z_violation,
                    missingness_penalty
                ),
                rank: 0,
            }
        })
        .collect::<Vec<_>>();

    ranking.sort_by(|left, right| {
        right
            .candidate_score
            .partial_cmp(&left.candidate_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.feature_name.cmp(&right.feature_name))
    });

    for (index, row) in ranking.iter_mut().enumerate() {
        row.rank = index + 1;
    }

    ranking
}

pub fn write_feature_ranking_csv(path: &Path, ranking: &[FeatureRankingRow]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "ranking_strategy",
        "rank",
        "feature_index",
        "feature_name",
        "ranking_formula",
        "dsfb_raw_boundary_points",
        "dsfb_persistent_boundary_points",
        "dsfb_raw_violation_points",
        "dsfb_persistent_violation_points",
        "ewma_alarm_points",
        "threshold_alarm_points",
        "pre_failure_run_hits",
        "motif_precision_proxy",
        "recall_rescue_contribution",
        "missing_fraction",
        "z_pre_failure_run_hits",
        "z_motif_precision_proxy",
        "z_recall_rescue_contribution",
        "z_boundary",
        "z_violation",
        "z_ewma",
        "missingness_penalty",
        "candidate_score",
        "score_breakdown",
    ])?;
    for row in ranking {
        writer.write_record([
            row.ranking_strategy.clone(),
            row.rank.to_string(),
            row.feature_index.to_string(),
            row.feature_name.clone(),
            row.ranking_formula.clone(),
            row.dsfb_raw_boundary_points.to_string(),
            row.dsfb_persistent_boundary_points.to_string(),
            row.dsfb_raw_violation_points.to_string(),
            row.dsfb_persistent_violation_points.to_string(),
            row.ewma_alarm_points.to_string(),
            row.threshold_alarm_points.to_string(),
            row.pre_failure_run_hits.to_string(),
            format_option_csv(row.motif_precision_proxy),
            format_option_csv(row.recall_rescue_contribution),
            format!("{:.6}", row.missing_fraction),
            format_option_csv(row.z_pre_failure_run_hits),
            format_option_csv(row.z_motif_precision_proxy),
            format_option_csv(row.z_recall_rescue_contribution),
            format!("{:.6}", row.z_boundary),
            format!("{:.6}", row.z_violation),
            format!("{:.6}", row.z_ewma),
            format!("{:.6}", row.missingness_penalty),
            format!("{:.6}", row.candidate_score),
            row.score_breakdown.clone(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn compare_feature_rankings(
    compression_ranking: &[FeatureRankingRow],
    recall_aware_ranking: &[FeatureRankingRow],
) -> Vec<FeatureRankingComparisonRow> {
    let compression_by_feature = compression_ranking
        .iter()
        .map(|row| (&row.feature_name, row))
        .collect::<BTreeMap<_, _>>();
    let recall_by_feature = recall_aware_ranking
        .iter()
        .map(|row| (&row.feature_name, row))
        .collect::<BTreeMap<_, _>>();

    let mut feature_names = compression_by_feature
        .keys()
        .copied()
        .chain(recall_by_feature.keys().copied())
        .collect::<Vec<_>>();
    feature_names.sort_unstable();
    feature_names.dedup();

    feature_names
        .into_iter()
        .map(|feature_name| {
            let compression = compression_by_feature.get(feature_name).copied();
            let recall = recall_by_feature.get(feature_name).copied();
            FeatureRankingComparisonRow {
                feature_index: compression
                    .or(recall)
                    .map(|row| row.feature_index)
                    .unwrap_or_default(),
                feature_name: feature_name.to_string(),
                compression_rank: compression.map(|row| row.rank),
                recall_aware_rank: recall.map(|row| row.rank),
                compression_score: compression.map(|row| row.candidate_score),
                recall_aware_score: recall.map(|row| row.candidate_score),
                rank_delta_recall_minus_compression: match (compression, recall) {
                    (Some(compression), Some(recall)) => {
                        Some(recall.rank as i64 - compression.rank as i64)
                    }
                    _ => None,
                },
            }
        })
        .collect()
}

pub fn write_feature_ranking_comparison_csv(
    path: &Path,
    rows: &[FeatureRankingComparisonRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_feature_policy_summary_csv(
    path: &Path,
    rows: &[FeaturePolicySummaryRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_recall_rescue_results_csv(path: &Path, rows: &[RecallRescueResultRow]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_missed_failure_diagnostics_csv(
    path: &Path,
    rows: &[MissedFailureDiagnosticRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_recall_critical_features_csv(
    path: &Path,
    rows: &[RecallCriticalFeatureRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_policy_contribution_analysis_csv(
    path: &Path,
    rows: &[PolicyContributionAnalysisRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn build_feature_cohorts(ranking: &[FeatureRankingRow]) -> FeatureCohorts {
    let ranking_formula = ranking
        .first()
        .map(|row| row.ranking_formula.clone())
        .unwrap_or_else(|| RANKING_FORMULA.into());
    let top_4 = ranking
        .iter()
        .take(4)
        .map(|row| cohort_member(row, "top_4"))
        .collect::<Vec<_>>();
    let top_8 = ranking
        .iter()
        .take(8)
        .map(|row| cohort_member(row, "top_8"))
        .collect::<Vec<_>>();
    let top_16 = ranking
        .iter()
        .take(16)
        .map(|row| cohort_member(row, "top_16"))
        .collect::<Vec<_>>();
    let all_features = ranking
        .iter()
        .map(|row| cohort_member(row, "all_features"))
        .collect::<Vec<_>>();

    let seed_feature_report = SEED_FEATURES
        .iter()
        .map(|seed| {
            if let Some(row) = ranking.iter().find(|row| row.feature_name == *seed) {
                SeedFeatureReport {
                    feature_name: (*seed).to_string(),
                    found_in_ranking: true,
                    rank: Some(row.rank),
                    candidate_score: Some(row.candidate_score),
                    in_top_4: row.rank <= 4,
                    in_top_8: row.rank <= 8,
                    in_top_16: row.rank <= 16,
                    top_4_note: seed_membership_note(row, 4, "top_4"),
                    top_8_note: seed_membership_note(row, 8, "top_8"),
                    top_16_note: seed_membership_note(row, 16, "top_16"),
                }
            } else {
                let note =
                    "Excluded because the feature is not analyzable in the saved run metrics."
                        .to_string();
                SeedFeatureReport {
                    feature_name: (*seed).to_string(),
                    found_in_ranking: false,
                    rank: None,
                    candidate_score: None,
                    in_top_4: false,
                    in_top_8: false,
                    in_top_16: false,
                    top_4_note: note.clone(),
                    top_8_note: note.clone(),
                    top_16_note: note,
                }
            }
        })
        .collect::<Vec<_>>();

    FeatureCohorts {
        ranking_formula,
        missingness_penalty_threshold: MISSINGNESS_PENALTY_THRESHOLD,
        missingness_penalty_value: MISSINGNESS_PENALTY_VALUE,
        top_4,
        top_8,
        top_16,
        all_features,
        seed_feature_report,
    }
}

pub fn build_seed_feature_check(cohorts: &FeatureCohorts) -> SeedFeatureCheckArtifact {
    SeedFeatureCheckArtifact {
        ranking_formula: cohorts.ranking_formula.clone(),
        requested_seed_features: SEED_FEATURES
            .iter()
            .map(|seed| (*seed).to_string())
            .collect(),
        seed_feature_report: cohorts.seed_feature_report.clone(),
    }
}

pub fn run_cohort_dsa_grid(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    cohorts: &FeatureCohorts,
    pre_failure_lookback_runs: usize,
    metrics: &BenchmarkMetrics,
) -> Result<CohortExecution> {
    run_cohort_dsa_grid_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        cohorts,
        pre_failure_lookback_runs,
        metrics,
        &DsaPolicyRuntime::default(),
        "compression_biased",
    )
}

pub fn run_cohort_dsa_grid_with_policy(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    cohorts: &FeatureCohorts,
    pre_failure_lookback_runs: usize,
    metrics: &BenchmarkMetrics,
    policy_runtime: &DsaPolicyRuntime,
    ranking_strategy: &str,
) -> Result<CohortExecution> {
    let cohort_specs = [
        ("top_4", cohorts.top_4.as_slice()),
        ("top_8", cohorts.top_8.as_slice()),
        ("top_16", cohorts.top_16.as_slice()),
        ("all_features", cohorts.all_features.as_slice()),
    ];

    let threshold_recall = metrics.summary.failure_runs_with_preceding_threshold_signal;
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let mut grid_rows = Vec::new();
    let mut motif_policy_rows = Vec::new();
    let mut feature_trace_config_id = 0usize;
    let mut grid_row_id = 0usize;

    for &window in DSA_WINDOW_SWEEP {
        for &persistence_runs in DSA_PERSISTENCE_SWEEP {
            for &alert_tau in DSA_TAU_SWEEP {
                let base_config = DsaConfig {
                    window,
                    persistence_runs,
                    alert_tau,
                    corroborating_feature_count_min: 1,
                };
                let base_evaluation = evaluate_dsa_with_policy(
                    dataset,
                    nominal,
                    residuals,
                    signs,
                    baselines,
                    grammar,
                    &base_config,
                    pre_failure_lookback_runs,
                    policy_runtime,
                )?;

                for (cohort_name, members) in cohort_specs {
                    if members.is_empty() {
                        continue;
                    }
                    let feature_indices = members
                        .iter()
                        .map(|member| member.feature_index)
                        .collect::<Vec<_>>();

                    for &corroborating_m in CORROBORATION_SWEEP {
                        if corroborating_m > feature_indices.len() {
                            continue;
                        }
                        let evaluation = project_dsa_to_cohort(
                            dataset,
                            nominal,
                            residuals,
                            baselines,
                            grammar,
                            &base_evaluation,
                            &feature_indices,
                            corroborating_m,
                            pre_failure_lookback_runs,
                            cohort_name,
                        )?;

                        let row = build_grid_row(
                            grid_row_id,
                            feature_trace_config_id,
                            ranking_strategy,
                            &cohorts.ranking_formula,
                            cohort_name,
                            members.len(),
                            &base_config,
                            corroborating_m,
                            &evaluation,
                            metrics,
                        );
                        motif_policy_rows.extend(build_motif_policy_rows(&row, &evaluation));
                        grid_rows.push(row);
                        grid_row_id += 1;
                    }
                }

                feature_trace_config_id += 1;
            }
        }
    }

    let best_by_cohort = build_best_by_cohort(&grid_rows);
    let closest_to_success = choose_closest_to_success(&grid_rows);
    let best_primary_success = grid_rows
        .iter()
        .filter(|row| row.primary_success)
        .cloned()
        .min_by(compare_successful_rows);
    let best_precursor_quality_row = grid_rows.iter().cloned().max_by(|left, right| {
        compare_option_f64(left.precursor_quality, right.precursor_quality)
            .then_with(|| compare_successful_rows(left, right))
    });
    let any_primary_success = best_primary_success.is_some();
    let selected_configuration = best_primary_success
        .clone()
        .or_else(|| closest_to_success.clone());
    let best_cohort = selected_configuration.as_ref().map(row_label);
    let corroboration_effect = corroboration_effect(&grid_rows);
    let limiting_factor = limiting_factor_from_row(
        selected_configuration.as_ref(),
        ewma_nuisance,
        threshold_recall,
    );
    let failure_analysis = if any_primary_success {
        None
    } else {
        build_failure_analysis(
            &grid_rows,
            &motif_policy_rows,
            cohorts,
            ewma_nuisance,
            threshold_recall,
            selected_configuration.as_ref(),
            &corroboration_effect,
            &limiting_factor,
        )
    };

    let summary = CohortDsaSummary {
        ranking_formula: RANKING_FORMULA.into(),
        primary_success_condition: primary_success_condition(),
        recall_tolerance_runs: RECALL_TOLERANCE,
        cohort_results: grid_rows.clone(),
        best_by_cohort,
        closest_to_success: closest_to_success.clone(),
        best_primary_success: best_primary_success.clone(),
        best_precursor_quality_row: best_precursor_quality_row.clone(),
        selected_configuration: selected_configuration.clone(),
        best_cohort,
        any_primary_success,
        failure_analysis,
        grid_point_count: grid_rows.len(),
        optimization_priority_order: optimization_priority_order(),
        cross_feature_corroboration_effect: corroboration_effect.clone(),
        limiting_factor: limiting_factor.clone(),
    };

    let grid_summary = CohortGridSummary {
        ranking_formula: RANKING_FORMULA.into(),
        primary_success_condition_definition: primary_success_condition(),
        recall_tolerance_runs: RECALL_TOLERANCE,
        grid_point_count: grid_rows.len(),
        optimization_priority_order: optimization_priority_order(),
        success_row_count: grid_rows.iter().filter(|row| row.primary_success).count(),
        any_success_row: any_primary_success,
        closest_to_success: closest_to_success.clone(),
        best_success_row: best_primary_success.clone(),
        best_precursor_quality_row: best_precursor_quality_row,
        cross_feature_corroboration_effect: corroboration_effect,
        limiting_factor,
    };

    let selected_row = selected_configuration.ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat("cohort grid produced no selectable row".into())
    })?;
    let selected_evaluation = rebuild_selected_evaluation(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        cohorts,
        pre_failure_lookback_runs,
        &selected_row,
    )?;

    Ok(CohortExecution {
        grid_summary,
        summary,
        motif_policy_contributions: motif_policy_rows,
        selected_evaluation,
    })
}

pub fn run_recall_optimization(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    metrics: &BenchmarkMetrics,
    pre_failure_lookback_runs: usize,
) -> Result<OptimizationExecution> {
    let baseline_feature_ranking = compute_feature_ranking(metrics);
    let baseline_feature_cohorts = build_feature_cohorts(&baseline_feature_ranking);
    let baseline_execution = run_cohort_dsa_grid(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &baseline_feature_cohorts,
        pre_failure_lookback_runs,
        metrics,
    )?;

    let recall_rescue_contributions =
        recall_rescue_contribution_by_feature(&baseline_execution.selected_evaluation);
    let recall_aware_feature_ranking =
        compute_feature_ranking_recall_aware(metrics, &recall_rescue_contributions);
    let ranking_comparison =
        compare_feature_rankings(&baseline_feature_ranking, &recall_aware_feature_ranking);
    let recall_aware_feature_cohorts = build_feature_cohorts(&recall_aware_feature_ranking);
    let feature_policy_overrides = build_feature_policy_overrides(
        metrics,
        baseline_execution
            .summary
            .selected_configuration
            .as_ref()
            .unwrap_or_else(|| {
                panic!("baseline cohort execution must provide a selected configuration")
            }),
        &baseline_execution.selected_evaluation,
        &recall_aware_feature_ranking,
    );
    let feature_policy_summary = build_feature_policy_summary(
        metrics,
        &baseline_feature_ranking,
        &recall_aware_feature_ranking,
        &feature_policy_overrides,
    );
    let policy_runtime = DsaPolicyRuntime {
        feature_policy_overrides: feature_policy_overrides.clone(),
        recall_rescue: RecallRescueConfig {
            enabled: true,
            ..RecallRescueConfig::default()
        },
    };

    let optimized_compression_execution = run_cohort_dsa_grid_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &baseline_feature_cohorts,
        pre_failure_lookback_runs,
        metrics,
        &policy_runtime,
        "compression_biased",
    )?;
    let recall_aware_execution = run_cohort_dsa_grid_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &recall_aware_feature_cohorts,
        pre_failure_lookback_runs,
        metrics,
        &policy_runtime,
        "recall_aware",
    )?;

    let mut union_rows = optimized_compression_execution
        .summary
        .cohort_results
        .clone();
    union_rows.extend(recall_aware_execution.summary.cohort_results.clone());

    let current_policy_dsa_nuisance = baseline_execution
        .summary
        .selected_configuration
        .as_ref()
        .map(|row| row.pass_run_nuisance_proxy)
        .unwrap_or(
            metrics
                .summary
                .pass_run_dsfb_persistent_boundary_nuisance_rate,
        );
    let pareto_frontier = pareto_frontier(&union_rows);
    let stage_a_candidates = stage_a_candidates(
        &union_rows,
        metrics.summary.pass_run_dsfb_raw_boundary_nuisance_rate,
        current_policy_dsa_nuisance,
    );
    let stage_b_candidates = stage_b_candidates(
        &stage_a_candidates,
        metrics.summary.pass_run_ewma_nuisance_rate,
        current_policy_dsa_nuisance,
    );
    let selected_row = choose_optimized_row(
        &stage_b_candidates,
        &union_rows,
        metrics.summary.pass_run_ewma_nuisance_rate,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
        current_policy_dsa_nuisance,
    )
    .ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(
            "optimized search produced no selectable configuration".into(),
        )
    })?;

    let selected_evaluation = rebuild_selected_evaluation_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &baseline_feature_cohorts,
        &recall_aware_feature_cohorts,
        pre_failure_lookback_runs,
        &selected_row,
        &policy_runtime,
    )?;

    let mut optimized_execution = if selected_row.ranking_strategy == "recall_aware" {
        recall_aware_execution.clone()
    } else {
        optimized_compression_execution.clone()
    };
    optimized_execution.selected_evaluation = selected_evaluation.clone();
    optimized_execution.summary.selected_configuration = Some(selected_row.clone());

    let recall_rescue_results = union_rows
        .iter()
        .map(|row| RecallRescueResultRow {
            ranking_strategy: row.ranking_strategy.clone(),
            cohort_name: row.cohort_name.clone(),
            window: row.window,
            persistence_runs: row.persistence_runs,
            alert_tau: row.alert_tau,
            corroborating_m: row.corroborating_m,
            failure_recall: row.failure_recall,
            pass_run_nuisance_proxy: row.pass_run_nuisance_proxy,
            rescued_point_count: row.rescued_point_count,
            rescued_watch_to_review_points: row.rescued_watch_to_review_points,
            rescued_review_to_escalate_points: row.rescued_review_to_escalate_points,
        })
        .collect::<Vec<_>>();
    let missed_failure_diagnostics = build_missed_failure_diagnostics(
        &baseline_execution.selected_evaluation,
        &selected_evaluation,
        &feature_policy_overrides,
    );
    let recall_critical_features = build_recall_critical_features(
        &baseline_execution.selected_evaluation,
        &selected_evaluation,
        &baseline_feature_ranking,
        &recall_aware_feature_ranking,
        &feature_policy_overrides,
        &recall_rescue_contributions,
    );
    let policy_contribution_analysis = build_policy_contribution_analysis(
        &baseline_execution.selected_evaluation,
        &selected_evaluation,
        &selected_row,
    );
    let delta_target_assessment = compute_delta_target_assessment(
        &selected_row,
        &stage_a_candidates,
        &union_rows,
        baseline_execution
            .summary
            .selected_configuration
            .as_ref()
            .unwrap_or_else(|| {
                panic!("baseline cohort execution must provide a selected configuration")
            }),
        metrics,
    );

    Ok(OptimizationExecution {
        baseline_feature_ranking,
        baseline_feature_cohorts,
        baseline_execution,
        recall_aware_feature_ranking,
        ranking_comparison,
        recall_aware_feature_cohorts,
        feature_policy_overrides,
        feature_policy_summary,
        optimized_execution,
        recall_aware_execution,
        pareto_frontier,
        stage_a_candidates,
        stage_b_candidates,
        recall_rescue_results,
        missed_failure_diagnostics,
        recall_critical_features,
        policy_contribution_analysis,
        delta_target_assessment,
    })
}

fn build_feature_policy_overrides(
    metrics: &BenchmarkMetrics,
    baseline_selected_row: &CohortGridResult,
    baseline_evaluation: &DsaEvaluation,
    recall_aware_ranking: &[FeatureRankingRow],
) -> Vec<FeaturePolicyOverride> {
    let feature_metrics = metrics
        .feature_metrics
        .iter()
        .map(|feature| (feature.feature_index, feature))
        .collect::<BTreeMap<_, _>>();
    let recall_rank_by_feature = recall_aware_ranking
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let mut missed_feature_stats = BTreeMap::<usize, (String, usize, f64)>::new();

    for signal in baseline_evaluation
        .per_failure_run_signals
        .iter()
        .filter(|signal| signal.earliest_dsa_run.is_none())
    {
        let Some(feature_index) = signal.max_dsa_score_feature_index else {
            continue;
        };
        let Some(feature_name) = signal.max_dsa_score_feature_name.as_ref() else {
            continue;
        };
        let score = signal.max_dsa_score_in_lookback.unwrap_or(0.0);
        let entry = missed_feature_stats
            .entry(feature_index)
            .or_insert_with(|| (feature_name.clone(), 0, 0.0));
        entry.1 += 1;
        entry.2 = entry.2.max(score);
    }

    let mut overrides = missed_feature_stats
        .into_iter()
        .filter_map(|(feature_index, (feature_name, miss_count, max_score))| {
            let feature_metric = feature_metrics.get(&feature_index)?;
            let recall_rank = recall_rank_by_feature.get(&feature_index).map(|row| row.rank);
            let max_score_floor = baseline_selected_row.alert_tau - 0.40;
            if max_score < max_score_floor
                || feature_metric.missing_fraction > OPTIMIZATION_OVERRIDE_MAX_MISSINGNESS
                || feature_metric.pre_failure_run_hits == 0
                || feature_metric.motif_precision_proxy.unwrap_or(0.0) <= 0.0
            {
                return None;
            }

            let rescue_priority =
                if miss_count >= 2 || max_score >= baseline_selected_row.alert_tau - 0.10 {
                    2
                } else {
                    1
                };
            let fragmentation_override =
                if feature_metric.motif_precision_proxy.unwrap_or(0.0) >= 0.70
                    && max_score >= baseline_selected_row.alert_tau - 0.10
                {
                    1.0
                } else {
                    OPTIMIZATION_RESCUE_FRAGMENTATION
                };

            Some(FeaturePolicyOverride {
                feature_index,
                feature_name: feature_name.clone(),
                alert_class_override: None,
                requires_persistence_override: Some(false),
                requires_corroboration_override: Some(false),
                minimum_window_override: Some(OPTIMIZATION_RESCUE_WINDOW),
                minimum_hits_override: Some(OPTIMIZATION_RESCUE_MIN_HITS),
                maximum_allowed_fragmentation_override: Some(fragmentation_override),
                rescue_eligible: true,
                rescue_priority,
                allow_watch_only: Some(false),
                allow_review_without_escalate: Some(true),
                suppress_if_isolated: Some(false),
                override_reason: format!(
                    "Feature was the nearest current-DSA miss on {} failure run(s), max near-miss score {:.4}, recall-aware rank {}, pre_failure_run_hits={}, motif_precision_proxy={}, rescue_fragmentation_ceiling={:.2}.",
                    miss_count,
                    max_score,
                    recall_rank
                        .map(|rank| rank.to_string())
                        .unwrap_or_else(|| "n/a".into()),
                    feature_metric.pre_failure_run_hits,
                    format_option_f64(feature_metric.motif_precision_proxy),
                    fragmentation_override,
                ),
            })
        })
        .collect::<Vec<_>>();

    overrides.sort_by(|left, right| {
        right
            .rescue_priority
            .cmp(&left.rescue_priority)
            .then_with(|| left.feature_name.cmp(&right.feature_name))
    });
    overrides
}

fn build_feature_policy_summary(
    metrics: &BenchmarkMetrics,
    baseline_ranking: &[FeatureRankingRow],
    recall_aware_ranking: &[FeatureRankingRow],
    overrides: &[FeaturePolicyOverride],
) -> Vec<FeaturePolicySummaryRow> {
    let feature_metrics = metrics
        .feature_metrics
        .iter()
        .map(|feature| (feature.feature_index, feature))
        .collect::<BTreeMap<_, _>>();
    let baseline_by_feature = baseline_ranking
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let recall_by_feature = recall_aware_ranking
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();

    overrides
        .iter()
        .filter_map(|override_entry| {
            let feature_metric = feature_metrics.get(&override_entry.feature_index)?;
            Some(FeaturePolicySummaryRow {
                feature_index: override_entry.feature_index,
                feature_name: override_entry.feature_name.clone(),
                compression_rank: baseline_by_feature
                    .get(&override_entry.feature_index)
                    .map(|row| row.rank),
                recall_aware_rank: recall_by_feature
                    .get(&override_entry.feature_index)
                    .map(|row| row.rank),
                pre_failure_run_hits: feature_metric.pre_failure_run_hits,
                motif_precision_proxy: feature_metric.motif_precision_proxy,
                missing_fraction: feature_metric.missing_fraction,
                rescue_eligible: override_entry.rescue_eligible,
                rescue_priority: override_entry.rescue_priority,
                alert_class_override: override_entry.alert_class_override,
                requires_persistence_override: override_entry.requires_persistence_override,
                requires_corroboration_override: override_entry.requires_corroboration_override,
                minimum_window_override: override_entry.minimum_window_override,
                minimum_hits_override: override_entry.minimum_hits_override,
                maximum_allowed_fragmentation_override: override_entry
                    .maximum_allowed_fragmentation_override,
                override_reason: override_entry.override_reason.clone(),
                allow_watch_only: override_entry.allow_watch_only,
                allow_review_without_escalate: override_entry.allow_review_without_escalate,
                suppress_if_isolated: override_entry.suppress_if_isolated,
            })
        })
        .collect()
}

fn recall_rescue_contribution_by_feature(
    baseline_evaluation: &DsaEvaluation,
) -> BTreeMap<usize, f64> {
    let mut contributions = BTreeMap::<usize, f64>::new();
    for signal in baseline_evaluation
        .per_failure_run_signals
        .iter()
        .filter(|signal| signal.earliest_dsa_run.is_none())
    {
        let Some(feature_index) = signal.max_dsa_score_feature_index else {
            continue;
        };
        *contributions.entry(feature_index).or_default() += 1.0;
    }
    contributions
}

fn build_recall_critical_features(
    baseline: &DsaEvaluation,
    optimized: &DsaEvaluation,
    baseline_ranking: &[FeatureRankingRow],
    recall_aware_ranking: &[FeatureRankingRow],
    feature_policy_overrides: &[FeaturePolicyOverride],
    recall_rescue_contributions: &BTreeMap<usize, f64>,
) -> Vec<RecallCriticalFeatureRow> {
    let optimized_by_failure = optimized
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let baseline_rank_by_feature = baseline_ranking
        .iter()
        .map(|row| (row.feature_index, row.rank))
        .collect::<BTreeMap<_, _>>();
    let recall_rank_by_feature = recall_aware_ranking
        .iter()
        .map(|row| (row.feature_index, row.rank))
        .collect::<BTreeMap<_, _>>();
    let overrides_by_feature = feature_policy_overrides
        .iter()
        .map(|override_entry| (override_entry.feature_index, override_entry))
        .collect::<BTreeMap<_, _>>();

    baseline
        .per_failure_run_signals
        .iter()
        .filter(|row| row.earliest_dsa_run.is_none())
        .map(|row| {
            let feature_index = row.max_dsa_score_feature_index;
            let override_entry = feature_index
                .and_then(|feature_index| overrides_by_feature.get(&feature_index).copied());
            let optimized_row = optimized_by_failure.get(&row.failure_run_index).copied();

            RecallCriticalFeatureRow {
                failure_run_index: row.failure_run_index,
                feature_index,
                feature_name: row.max_dsa_score_feature_name.clone(),
                compression_rank: feature_index.and_then(|feature_index| {
                    baseline_rank_by_feature.get(&feature_index).copied()
                }),
                recall_aware_rank: feature_index
                    .and_then(|feature_index| recall_rank_by_feature.get(&feature_index).copied()),
                max_structural_score: row.max_dsa_score_in_lookback,
                resolved_alert_class: row.max_dsa_score_resolved_alert_class.clone(),
                policy_state: row.max_dsa_score_policy_state.clone(),
                boundary_density_w: row.max_dsa_score_boundary_density_w,
                ewma_occupancy_w: row.max_dsa_score_ewma_occupancy_w,
                motif_recurrence_w: row.max_dsa_score_motif_recurrence_w,
                fragmentation_proxy_w: row.max_dsa_score_fragmentation_proxy_w,
                consistent: row.max_dsa_score_consistent,
                exact_miss_rule: if row
                    .max_dsa_score_consistent
                    .is_some_and(|consistent| !consistent)
                    && row
                        .max_dsa_score_resolved_alert_class
                        .as_deref()
                        .is_some_and(|class| class == "Watch" || class == "Review")
                {
                    "directional_consistency_gate".into()
                } else if row.max_dsa_score_numeric_dsa_alert == Some(false)
                    && row.max_dsa_score_in_lookback.is_some()
                {
                    "watch_class_near_miss_below_numeric_gate".into()
                } else if row.max_dsa_score_in_lookback.unwrap_or(0.0) < 2.0 {
                    "numeric_score_below_tau".into()
                } else {
                    "policy_state_never_reached_review".into()
                },
                feature_override_exists: override_entry.is_some(),
                rescue_priority: override_entry
                    .map(|override_entry| override_entry.rescue_priority),
                allow_review_without_escalate: override_entry
                    .and_then(|override_entry| override_entry.allow_review_without_escalate),
                bounded_feature_override_would_recover: optimized_row
                    .is_some_and(|optimized_row| optimized_row.earliest_dsa_run.is_some()),
                recovered_after_optimization: optimized_row
                    .is_some_and(|optimized_row| optimized_row.earliest_dsa_run.is_some()),
                optimized_feature_name: optimized_row
                    .and_then(|optimized_row| optimized_row.earliest_dsa_feature_name.clone()),
                recall_rescue_contribution: feature_index
                    .and_then(|feature_index| {
                        recall_rescue_contributions.get(&feature_index).copied()
                    })
                    .unwrap_or(0.0),
            }
        })
        .collect()
}

fn pareto_frontier(rows: &[CohortGridResult]) -> Vec<CohortGridResult> {
    let recall_floor = 100usize;
    let candidate_pool = rows
        .iter()
        .filter(|row| row.failure_recall >= recall_floor)
        .collect::<Vec<_>>();
    let candidate_pool = if candidate_pool.is_empty() {
        rows.iter().collect::<Vec<_>>()
    } else {
        candidate_pool
    };

    let mut frontier = candidate_pool
        .iter()
        .filter(|row| {
            !candidate_pool.iter().any(|other| {
                other.grid_row_id != row.grid_row_id
                    && delta_nuisance_relative(row.ewma_nuisance, other.pass_run_nuisance_proxy)
                        >= delta_nuisance_relative(row.ewma_nuisance, row.pass_run_nuisance_proxy)
                    && other.failure_recall >= row.failure_recall
                    && (delta_nuisance_relative(row.ewma_nuisance, other.pass_run_nuisance_proxy)
                        > delta_nuisance_relative(row.ewma_nuisance, row.pass_run_nuisance_proxy)
                        || other.failure_recall > row.failure_recall)
            })
        })
        .map(|row| (*row).clone())
        .collect::<Vec<_>>();
    frontier.sort_by(|left, right| compare_stage_b_rows(left, right, left.ewma_nuisance));
    frontier
}

fn stage_a_candidates(
    rows: &[CohortGridResult],
    raw_boundary_nuisance: f64,
    current_policy_dsa_nuisance: f64,
) -> Vec<CohortGridResult> {
    let mut candidates = rows
        .iter()
        .filter(|row| {
            row.pass_run_nuisance_proxy < raw_boundary_nuisance && row.failure_recall >= 100
        })
        .cloned()
        .collect::<Vec<_>>();
    candidates
        .sort_by(|left, right| compare_stage_a_rows(left, right, current_policy_dsa_nuisance));
    candidates
}

fn stage_b_candidates(
    rows: &[CohortGridResult],
    ewma_nuisance: f64,
    current_policy_dsa_nuisance: f64,
) -> Vec<CohortGridResult> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        (left.pass_run_nuisance_proxy < ewma_nuisance)
            .cmp(&(right.pass_run_nuisance_proxy < ewma_nuisance))
            .reverse()
            .then_with(|| compare_stage_b_rows(left, right, current_policy_dsa_nuisance))
    });
    candidates
}

fn choose_optimized_row(
    stage_b_candidates: &[CohortGridResult],
    all_rows: &[CohortGridResult],
    ewma_nuisance: f64,
    threshold_recall: usize,
    current_policy_dsa_nuisance: f64,
) -> Option<CohortGridResult> {
    stage_b_candidates.first().cloned().or_else(|| {
        all_rows.iter().cloned().min_by(|left, right| {
            let left_primary_gap = primary_success_gap(left);
            let right_primary_gap = primary_success_gap(right);
            left_primary_gap
                .partial_cmp(&right_primary_gap)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    (left.pass_run_nuisance_proxy < ewma_nuisance)
                        .cmp(&(right.pass_run_nuisance_proxy < ewma_nuisance))
                        .reverse()
                })
                .then_with(|| {
                    let left_recall_gap = threshold_recall.saturating_sub(left.failure_recall);
                    let right_recall_gap = threshold_recall.saturating_sub(right.failure_recall);
                    left_recall_gap.cmp(&right_recall_gap)
                })
                .then_with(|| compare_stage_b_rows(left, right, current_policy_dsa_nuisance))
        })
    })
}

fn compare_stage_a_rows(
    left: &CohortGridResult,
    right: &CohortGridResult,
    current_policy_dsa_nuisance: f64,
) -> Ordering {
    delta_nuisance_relative(right.ewma_nuisance, right.pass_run_nuisance_proxy)
        .partial_cmp(&delta_nuisance_relative(
            left.ewma_nuisance,
            left.pass_run_nuisance_proxy,
        ))
        .unwrap_or(Ordering::Equal)
        .then_with(|| {
            delta_nuisance_relative(current_policy_dsa_nuisance, right.pass_run_nuisance_proxy)
                .partial_cmp(&delta_nuisance_relative(
                    current_policy_dsa_nuisance,
                    left.pass_run_nuisance_proxy,
                ))
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| right.failure_recall.cmp(&left.failure_recall))
        .then_with(|| compare_option_f64(right.precursor_quality, left.precursor_quality))
        .then_with(|| compare_option_f64(right.mean_lead_time_runs, left.mean_lead_time_runs))
        .then_with(|| compare_option_f64(right.compression_ratio, left.compression_ratio))
}

fn compare_stage_b_rows(
    left: &CohortGridResult,
    right: &CohortGridResult,
    current_policy_dsa_nuisance: f64,
) -> Ordering {
    right
        .failure_recall
        .cmp(&left.failure_recall)
        .then_with(|| {
            delta_nuisance_relative(right.ewma_nuisance, right.pass_run_nuisance_proxy)
                .partial_cmp(&delta_nuisance_relative(
                    left.ewma_nuisance,
                    left.pass_run_nuisance_proxy,
                ))
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| compare_option_f64(right.precursor_quality, left.precursor_quality))
        .then_with(|| compare_option_f64(right.mean_lead_time_runs, left.mean_lead_time_runs))
        .then_with(|| compare_option_f64(right.compression_ratio, left.compression_ratio))
        .then_with(|| {
            delta_nuisance_relative(current_policy_dsa_nuisance, right.pass_run_nuisance_proxy)
                .partial_cmp(&delta_nuisance_relative(
                    current_policy_dsa_nuisance,
                    left.pass_run_nuisance_proxy,
                ))
                .unwrap_or(Ordering::Equal)
        })
}

fn rebuild_selected_evaluation_with_policy(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    baseline_cohorts: &FeatureCohorts,
    recall_aware_cohorts: &FeatureCohorts,
    pre_failure_lookback_runs: usize,
    row: &CohortGridResult,
    policy_runtime: &DsaPolicyRuntime,
) -> Result<DsaEvaluation> {
    let cohorts = if row.ranking_strategy == "recall_aware" {
        recall_aware_cohorts
    } else {
        baseline_cohorts
    };
    let base_config = DsaConfig {
        window: row.window,
        persistence_runs: row.persistence_runs,
        alert_tau: row.alert_tau,
        corroborating_feature_count_min: 1,
    };
    let base_evaluation = evaluate_dsa_with_policy(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &base_config,
        pre_failure_lookback_runs,
        policy_runtime,
    )?;
    let feature_indices = cohort_members(cohorts, &row.cohort_name)
        .iter()
        .map(|member| member.feature_index)
        .collect::<Vec<_>>();
    project_dsa_to_cohort(
        dataset,
        nominal,
        residuals,
        baselines,
        grammar,
        &base_evaluation,
        &feature_indices,
        row.corroborating_m,
        pre_failure_lookback_runs,
        &row.cohort_name,
    )
}

fn build_missed_failure_diagnostics(
    baseline: &DsaEvaluation,
    optimized: &DsaEvaluation,
    feature_policy_overrides: &[FeaturePolicyOverride],
) -> Vec<MissedFailureDiagnosticRow> {
    let optimized_by_failure = optimized
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let overrides_by_feature = feature_policy_overrides
        .iter()
        .map(|override_entry| (override_entry.feature_name.as_str(), override_entry))
        .collect::<BTreeMap<_, _>>();

    baseline
        .per_failure_run_signals
        .iter()
        .filter(|row| row.earliest_dsa_run.is_none())
        .map(|row| {
            let optimized_row = optimized_by_failure.get(&row.failure_run_index).copied();
            let resolved_watch = row
                .max_dsa_score_resolved_alert_class
                .as_deref()
                .is_some_and(|value| value == "Watch" || value == "Review");
            let override_entry = row
                .max_dsa_score_feature_name
                .as_deref()
                .and_then(|feature_name| overrides_by_feature.get(feature_name))
                .copied();
            let fragmentation_ceiling = override_entry.is_some_and(|override_entry| {
                row.max_dsa_score_fragmentation_proxy_w.unwrap_or(0.0)
                    > override_entry
                        .maximum_allowed_fragmentation_override
                        .unwrap_or(OPTIMIZATION_RESCUE_FRAGMENTATION)
            });
            let directional_consistency_gate =
                row.max_dsa_score_consistent == Some(false) && resolved_watch;
            let policy_suppression = row.max_dsa_score_policy_suppressed.unwrap_or(false)
                || (row
                    .max_dsa_score_policy_state
                    .as_deref()
                    .is_some_and(|state| state == "silent")
                    && resolved_watch);
            let persistence_gate = row
                .max_dsa_score_policy_state
                .as_deref()
                .is_some_and(|state| state == "silent")
                && row.max_dsa_score_numeric_dsa_alert == Some(false)
                && row.max_dsa_score_in_lookback.is_some();
            let rescue_eligible = override_entry.is_some();
            let recovered_after_optimization =
                optimized_row.is_some_and(|optimized_row| optimized_row.earliest_dsa_run.is_some());

            MissedFailureDiagnosticRow {
                failure_run_index: row.failure_run_index,
                nearest_feature_name: row.max_dsa_score_feature_name.clone(),
                nearest_feature_score: row.max_dsa_score_in_lookback,
                nearest_feature_policy_state: row.max_dsa_score_policy_state.clone(),
                nearest_feature_resolved_alert_class: row
                    .max_dsa_score_resolved_alert_class
                    .clone(),
                nearest_feature_boundary_density_w: row.max_dsa_score_boundary_density_w,
                nearest_feature_ewma_occupancy_w: row.max_dsa_score_ewma_occupancy_w,
                nearest_feature_motif_recurrence_w: row.max_dsa_score_motif_recurrence_w,
                nearest_feature_fragmentation_proxy_w: row.max_dsa_score_fragmentation_proxy_w,
                nearest_feature_consistent: row.max_dsa_score_consistent,
                ranking_exclusion: false,
                cohort_selection: false,
                policy_suppression,
                fragmentation_ceiling,
                directional_consistency_gate,
                persistence_gate,
                corroboration_threshold: false,
                rescue_gate_not_activating: rescue_eligible && !recovered_after_optimization,
                exact_miss_rule: if fragmentation_ceiling {
                    "feature_override_fragmentation_ceiling".into()
                } else if directional_consistency_gate {
                    "directional_consistency_gate".into()
                } else if persistence_gate {
                    "watch_class_near_miss_below_numeric_gate".into()
                } else if row.max_dsa_score_in_lookback.unwrap_or(0.0) < 2.0 {
                    "numeric_score_below_tau".into()
                } else {
                    "policy_state_never_reached_review".into()
                },
                bounded_rescue_would_recover: recovered_after_optimization,
                recovered_after_optimization,
                optimized_feature_name: optimized_row
                    .and_then(|row| row.earliest_dsa_feature_name.clone()),
            }
        })
        .collect()
}

fn build_policy_contribution_analysis(
    baseline: &DsaEvaluation,
    optimized: &DsaEvaluation,
    selected_row: &CohortGridResult,
) -> Vec<PolicyContributionAnalysisRow> {
    let baseline_missed = baseline
        .per_failure_run_signals
        .iter()
        .filter(|row| row.earliest_dsa_run.is_none())
        .map(|row| row.failure_run_index)
        .collect::<Vec<_>>();
    let optimized_by_failure = optimized
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();

    for contribution in &optimized.motif_policy_contributions {
        rows.push(PolicyContributionAnalysisRow {
            configuration_role: if selected_row.primary_success {
                "best_success".into()
            } else {
                "best_near_success".into()
            },
            contribution_type: "motif_nuisance_suppression".into(),
            name: contribution.motif_name.clone(),
            value: contribution.silent_suppression_points as f64,
            note: "silent_suppression_points".into(),
        });
        rows.push(PolicyContributionAnalysisRow {
            configuration_role: if selected_row.primary_success {
                "best_success".into()
            } else {
                "best_near_success".into()
            },
            contribution_type: "motif_pre_failure_review_or_escalate".into(),
            name: contribution.motif_name.clone(),
            value: contribution.pre_failure_review_or_escalate_points as f64,
            note: "pre_failure_review_or_escalate_points".into(),
        });
    }

    let mut rescued_feature_counts = BTreeMap::<String, usize>::new();
    for failure_run_index in baseline_missed {
        if let Some(optimized_row) = optimized_by_failure.get(&failure_run_index) {
            if let Some(feature_name) = &optimized_row.earliest_dsa_feature_name {
                *rescued_feature_counts
                    .entry(feature_name.clone())
                    .or_default() += 1;
            }
        }
    }
    for (feature_name, count) in rescued_feature_counts {
        rows.push(PolicyContributionAnalysisRow {
            configuration_role: if selected_row.primary_success {
                "best_success".into()
            } else {
                "best_near_success".into()
            },
            contribution_type: "rescued_failure_feature".into(),
            name: feature_name,
            value: count as f64,
            note: "recovered baseline-missed failures".into(),
        });
    }

    let mut rescue_transition_counts = BTreeMap::<String, usize>::new();
    for trace in &optimized.traces {
        for transition in &trace.rescue_transition {
            if transition != "none" {
                *rescue_transition_counts
                    .entry(transition.clone())
                    .or_default() += 1;
            }
        }
    }
    for (transition, count) in rescue_transition_counts {
        rows.push(PolicyContributionAnalysisRow {
            configuration_role: if selected_row.primary_success {
                "best_success".into()
            } else {
                "best_near_success".into()
            },
            contribution_type: "rescue_transition".into(),
            name: transition,
            value: count as f64,
            note: "rescued feature points".into(),
        });
    }

    rows
}

fn compute_delta_target_assessment(
    selected_row: &CohortGridResult,
    stage_a_candidates: &[CohortGridResult],
    all_rows: &[CohortGridResult],
    current_policy_baseline_row: &CohortGridResult,
    metrics: &BenchmarkMetrics,
) -> DeltaTargetAssessment {
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let current_policy_dsa_nuisance = current_policy_baseline_row.pass_run_nuisance_proxy;
    let primary_target_nuisance_ceiling = ewma_nuisance * (1.0 - PRIMARY_DELTA_TARGET);
    let secondary_target_nuisance_ceiling =
        current_policy_dsa_nuisance * (1.0 - SECONDARY_DELTA_TARGET);

    let selected_configuration =
        delta_candidate_summary(selected_row, ewma_nuisance, current_policy_dsa_nuisance);
    let best_recall_103_candidate = all_rows
        .iter()
        .filter(|row| row.failure_recall >= 103)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .min_by(|left, right| compare_stage_a_rows(left, right, current_policy_dsa_nuisance))
        .map(|row| delta_candidate_summary(&row, ewma_nuisance, current_policy_dsa_nuisance));
    let best_recall_104_candidate = all_rows
        .iter()
        .filter(|row| row.failure_recall >= 104)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .min_by(|left, right| compare_stage_a_rows(left, right, current_policy_dsa_nuisance))
        .map(|row| delta_candidate_summary(&row, ewma_nuisance, current_policy_dsa_nuisance));
    let best_secondary_target_candidate = all_rows
        .iter()
        .filter(|row| row.failure_recall >= 100)
        .cloned()
        .max_by(|left, right| {
            delta_nuisance_relative(current_policy_dsa_nuisance, left.pass_run_nuisance_proxy)
                .partial_cmp(&delta_nuisance_relative(
                    current_policy_dsa_nuisance,
                    right.pass_run_nuisance_proxy,
                ))
                .unwrap_or(Ordering::Equal)
        })
        .map(|row| delta_candidate_summary(&row, ewma_nuisance, current_policy_dsa_nuisance));
    let best_stage_a_delta_candidate = stage_a_candidates
        .first()
        .map(|row| delta_candidate_summary(row, ewma_nuisance, current_policy_dsa_nuisance));
    let best_reachable_pareto_point = best_recall_103_candidate
        .clone()
        .or_else(|| best_stage_a_delta_candidate.clone())
        .unwrap_or_else(|| selected_configuration.clone());

    let primary_target_met = selected_configuration.delta_nuisance_vs_ewma >= PRIMARY_DELTA_TARGET
        && selected_configuration.failure_recall >= 103;
    let ideal_target_met = selected_configuration.delta_nuisance_vs_ewma >= PRIMARY_DELTA_TARGET
        && selected_configuration.failure_recall >= 104;
    let secondary_target_met = selected_configuration.delta_nuisance_vs_current_dsa
        >= SECONDARY_DELTA_TARGET
        && selected_configuration.failure_recall >= 100;
    let mean_lead_time_ge_ewma = paired_ge(
        selected_row.mean_lead_time_runs,
        metrics.lead_time_summary.mean_ewma_lead_runs,
    );
    let mean_lead_time_ge_threshold = paired_ge(
        selected_row.mean_lead_time_runs,
        metrics.lead_time_summary.mean_threshold_lead_runs,
    );

    let assessment_note = if primary_target_met {
        format!(
            "Primary 40% nuisance-reduction target reached on {} with delta_nuisance_vs_ewma {:.4} and recall {}/{}.",
            selected_configuration.configuration,
            selected_configuration.delta_nuisance_vs_ewma,
            selected_configuration.failure_recall,
            selected_configuration.failure_runs,
        )
    } else if let Some(best_recall_103_candidate) = &best_recall_103_candidate {
        format!(
            "Primary 40% nuisance-reduction target was not reachable in the saved deterministic sweep. The best row retaining recall >= 103/104 was {} with nuisance {:.4}, delta_nuisance_vs_ewma {:.4}, and delta_nuisance_vs_current_dsa {:.4}. Reaching the primary target would require nuisance <= {:.4}; no recall >= 103 row achieved that ceiling.",
            best_recall_103_candidate.configuration,
            best_recall_103_candidate.pass_run_nuisance_proxy,
            best_recall_103_candidate.delta_nuisance_vs_ewma,
            best_recall_103_candidate.delta_nuisance_vs_current_dsa,
            primary_target_nuisance_ceiling,
        )
    } else if let Some(best_secondary_target_candidate) = &best_secondary_target_candidate {
        format!(
            "No recall-preserving row reached the primary 40% delta target. The best row with recall >= 100/104 was {} with delta_nuisance_vs_ewma {:.4} and delta_nuisance_vs_current_dsa {:.4}; the secondary 40% target would require nuisance <= {:.4}.",
            best_secondary_target_candidate.configuration,
            best_secondary_target_candidate.delta_nuisance_vs_ewma,
            best_secondary_target_candidate.delta_nuisance_vs_current_dsa,
            secondary_target_nuisance_ceiling,
        )
    } else {
        format!(
            "No saved row satisfied even the Stage A recall floor, so the 40% target is unachievable under the current deterministic search."
        )
    };

    DeltaTargetAssessment {
        primary_target_definition: predeclared_primary_target(),
        secondary_target_definition: predeclared_secondary_target(),
        ewma_nuisance_baseline: ewma_nuisance,
        current_policy_dsa_nuisance_baseline: current_policy_dsa_nuisance,
        primary_delta_target: PRIMARY_DELTA_TARGET,
        secondary_delta_target: SECONDARY_DELTA_TARGET,
        primary_target_nuisance_ceiling,
        secondary_target_nuisance_ceiling,
        selected_configuration,
        primary_target_met,
        ideal_target_met,
        secondary_target_met,
        mean_lead_time_ge_ewma,
        mean_lead_time_ge_threshold,
        best_recall_103_candidate,
        best_recall_104_candidate,
        best_secondary_target_candidate,
        best_stage_a_delta_candidate,
        best_reachable_pareto_point,
        assessment_note,
    }
}

fn delta_candidate_summary(
    row: &CohortGridResult,
    ewma_nuisance: f64,
    current_policy_dsa_nuisance: f64,
) -> DeltaCandidateSummary {
    DeltaCandidateSummary {
        configuration: row_label(row),
        ranking_strategy: row.ranking_strategy.clone(),
        cohort_name: row.cohort_name.clone(),
        window: row.window,
        persistence_runs: row.persistence_runs,
        alert_tau: row.alert_tau,
        corroborating_m: row.corroborating_m,
        failure_recall: row.failure_recall,
        failure_runs: row.failure_runs,
        pass_run_nuisance_proxy: row.pass_run_nuisance_proxy,
        delta_nuisance_vs_ewma: delta_nuisance_relative(ewma_nuisance, row.pass_run_nuisance_proxy),
        delta_nuisance_vs_current_dsa: delta_nuisance_relative(
            current_policy_dsa_nuisance,
            row.pass_run_nuisance_proxy,
        ),
        mean_lead_time_runs: row.mean_lead_time_runs,
        precursor_quality: row.precursor_quality,
        compression_ratio: row.compression_ratio,
    }
}

fn delta_nuisance_relative(baseline_nuisance: f64, dsa_nuisance: f64) -> f64 {
    if baseline_nuisance.abs() <= f64::EPSILON {
        0.0
    } else {
        (baseline_nuisance - dsa_nuisance) / baseline_nuisance
    }
}

pub fn write_cohort_results_csv(path: &Path, results: &[CohortGridResult]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "ranking_strategy",
        "ranking_formula",
        "grid_row_id",
        "feature_trace_config_id",
        "cohort_name",
        "cohort_size",
        "window",
        "persistence_runs",
        "alert_tau",
        "corroborating_m",
        "primary_run_signal",
        "failure_recall",
        "failure_runs",
        "failure_recall_rate",
        "threshold_recall",
        "ewma_recall",
        "failure_recall_delta_vs_threshold",
        "failure_recall_delta_vs_ewma",
        "mean_lead_time_runs",
        "median_lead_time_runs",
        "threshold_mean_lead_time_runs",
        "ewma_mean_lead_time_runs",
        "mean_lead_delta_vs_threshold_runs",
        "mean_lead_delta_vs_ewma_runs",
        "pass_run_nuisance_proxy",
        "numeric_pass_run_nuisance_proxy",
        "ewma_nuisance",
        "threshold_nuisance",
        "pass_run_nuisance_delta_vs_ewma",
        "pass_run_nuisance_delta_vs_threshold",
        "pass_run_nuisance_delta_vs_numeric_dsa",
        "raw_boundary_episode_count",
        "dsa_episode_count",
        "dsa_episodes_preceding_failure",
        "mean_dsa_episode_length_runs",
        "max_dsa_episode_length_runs",
        "compression_ratio",
        "precursor_quality",
        "non_escalating_dsa_episode_fraction",
        "feature_level_active_points",
        "feature_level_alert_points",
        "persistence_suppression_fraction",
        "numeric_failure_recall",
        "policy_vs_numeric_recall_delta",
        "watch_point_count",
        "review_point_count",
        "escalate_point_count",
        "silenced_point_count",
        "rescued_point_count",
        "rescued_watch_to_review_points",
        "rescued_review_to_escalate_points",
        "primary_success",
        "primary_success_reason",
    ])?;
    for row in results {
        writer.write_record([
            row.ranking_strategy.clone(),
            row.ranking_formula.clone(),
            row.grid_row_id.to_string(),
            row.feature_trace_config_id.to_string(),
            row.cohort_name.clone(),
            row.cohort_size.to_string(),
            row.window.to_string(),
            row.persistence_runs.to_string(),
            format!("{:.6}", row.alert_tau),
            row.corroborating_m.to_string(),
            row.primary_run_signal.clone(),
            row.failure_recall.to_string(),
            row.failure_runs.to_string(),
            format!("{:.6}", row.failure_recall_rate),
            row.threshold_recall.to_string(),
            row.ewma_recall.to_string(),
            row.failure_recall_delta_vs_threshold.to_string(),
            row.failure_recall_delta_vs_ewma.to_string(),
            format_option_csv(row.mean_lead_time_runs),
            format_option_csv(row.median_lead_time_runs),
            format_option_csv(row.threshold_mean_lead_time_runs),
            format_option_csv(row.ewma_mean_lead_time_runs),
            format_option_csv(row.mean_lead_delta_vs_threshold_runs),
            format_option_csv(row.mean_lead_delta_vs_ewma_runs),
            format!("{:.6}", row.pass_run_nuisance_proxy),
            format!("{:.6}", row.numeric_pass_run_nuisance_proxy),
            format!("{:.6}", row.ewma_nuisance),
            format!("{:.6}", row.threshold_nuisance),
            format!("{:.6}", row.pass_run_nuisance_delta_vs_ewma),
            format!("{:.6}", row.pass_run_nuisance_delta_vs_threshold),
            format!("{:.6}", row.pass_run_nuisance_delta_vs_numeric_dsa),
            row.raw_boundary_episode_count.to_string(),
            row.dsa_episode_count.to_string(),
            row.dsa_episodes_preceding_failure.to_string(),
            format_option_csv(row.mean_dsa_episode_length_runs),
            row.max_dsa_episode_length_runs.to_string(),
            format_option_csv(row.compression_ratio),
            format_option_csv(row.precursor_quality),
            format_option_csv(row.non_escalating_dsa_episode_fraction),
            row.feature_level_active_points.to_string(),
            row.feature_level_alert_points.to_string(),
            format_option_csv(row.persistence_suppression_fraction),
            row.numeric_failure_recall.to_string(),
            row.policy_vs_numeric_recall_delta.to_string(),
            row.watch_point_count.to_string(),
            row.review_point_count.to_string(),
            row.escalate_point_count.to_string(),
            row.silenced_point_count.to_string(),
            row.rescued_point_count.to_string(),
            row.rescued_watch_to_review_points.to_string(),
            row.rescued_review_to_escalate_points.to_string(),
            row.primary_success.to_string(),
            row.primary_success_reason.clone(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_motif_policy_contributions_csv(
    path: &Path,
    rows: &[CohortMotifPolicyContributionRow],
) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "grid_row_id",
        "cohort_name",
        "cohort_size",
        "window",
        "persistence_runs",
        "alert_tau",
        "corroborating_m",
        "motif_name",
        "alert_class_default",
        "watch_points",
        "review_points",
        "escalate_points",
        "silent_suppression_points",
        "pass_review_or_escalate_points",
        "pre_failure_review_or_escalate_points",
    ])?;
    for row in rows {
        writer.write_record([
            row.grid_row_id.to_string(),
            row.cohort_name.clone(),
            row.cohort_size.to_string(),
            row.window.to_string(),
            row.persistence_runs.to_string(),
            format!("{:.6}", row.alert_tau),
            row.corroborating_m.to_string(),
            row.motif_name.clone(),
            format!("{:?}", row.alert_class_default),
            row.watch_points.to_string(),
            row.review_points.to_string(),
            row.escalate_points.to_string(),
            row.silent_suppression_points.to_string(),
            row.pass_review_or_escalate_points.to_string(),
            row.pre_failure_review_or_escalate_points.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_precursor_quality_csv(path: &Path, results: &[CohortGridResult]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "cohort_name",
        "window",
        "persistence_runs",
        "alert_tau",
        "corroborating_m",
        "raw_boundary_episode_count",
        "dsa_episode_count",
        "dsa_episodes_preceding_failure",
        "precursor_quality",
        "compression_ratio",
    ])?;
    for row in results {
        writer.write_record([
            row.cohort_name.clone(),
            row.window.to_string(),
            row.persistence_runs.to_string(),
            format!("{:.6}", row.alert_tau),
            row.corroborating_m.to_string(),
            row.raw_boundary_episode_count.to_string(),
            row.dsa_episode_count.to_string(),
            row.dsa_episodes_preceding_failure.to_string(),
            format_option_csv(row.precursor_quality),
            format_option_csv(row.compression_ratio),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_failure_analysis_md(path: &Path, analysis: &CohortFailureAnalysis) -> Result<()> {
    let content = format!(
        "# DSA Cohort Failure Analysis\n\n\
         ## Closest near-success configuration\n\n\
         - Cohort: {}\n\
         - Grid point: {}\n\
         - Policy setting: {}\n\
         - Nuisance: {:.6}\n\
         - Recall: {}\n\
         - EWMA nuisance target: {:.6}\n\
         - Threshold recall target: {}\n\n\
         ## Limiting factor\n\n\
         {}\n\n\
         ## Cross-feature corroboration effect\n\n\
         {}\n\n\
         ## Policy vs numeric-only DSA\n\n\
         {}\n\n\
         ## Ranking quality\n\n\
         {}\n\n\
         ## All-feature DSA vs cohort DSA\n\n\
         {}\n\n\
         ## Motif classes most responsible for nuisance\n\n\
         {}\n\n\
         ## Motif classes most responsible for useful precursor episodes\n\n\
         {}\n\n\
         ## Best near-success source\n\n\
         {}\n",
        analysis.closest_cohort,
        analysis.closest_grid_point,
        analysis.closest_policy_setting,
        analysis.closest_nuisance,
        analysis.closest_recall,
        analysis.ewma_nuisance,
        analysis.threshold_recall,
        analysis.limiting_factor,
        analysis.corroboration_effect,
        analysis.policy_vs_numeric_note,
        analysis.ranking_quality_note,
        analysis.all_feature_dsa_vs_cohort_note,
        analysis.nuisance_motif_classes,
        analysis.useful_precursor_motif_classes,
        analysis.best_near_success_source,
    );
    std::fs::write(path, content)?;
    Ok(())
}

pub fn write_heuristic_policy_failure_analysis_md(
    path: &Path,
    analysis: &CohortFailureAnalysis,
) -> Result<()> {
    write_failure_analysis_md(path, analysis)
}

pub fn compute_rating_delta_forecast(
    dsa: &DsaEvaluation,
    metrics: &BenchmarkMetrics,
    cohort_summary: Option<&CohortDsaSummary>,
) -> RatingDeltaForecast {
    let chosen = cohort_summary
        .and_then(|summary| summary.selected_configuration.as_ref())
        .cloned()
        .unwrap_or_else(|| fallback_row_from_dsa(dsa, metrics));
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let threshold_recall = metrics.summary.failure_runs_with_preceding_threshold_signal;
    let best_all_features = cohort_summary
        .and_then(best_all_features_row)
        .cloned()
        .unwrap_or_else(|| fallback_row_from_dsa(dsa, metrics));

    let primary_success_met = chosen.pass_run_nuisance_proxy < ewma_nuisance
        && chosen.failure_recall + RECALL_TOLERANCE >= threshold_recall;
    let secondary_lead_time_vs_ewma = paired_ge(
        chosen.mean_lead_time_runs,
        metrics.lead_time_summary.mean_ewma_lead_runs,
    );
    let secondary_lead_time_vs_threshold = paired_ge(
        chosen.mean_lead_time_runs,
        metrics.lead_time_summary.mean_threshold_lead_runs,
    );
    let secondary_precursor_quality_vs_all_feature_dsa = compare_option_gt(
        chosen.precursor_quality,
        best_all_features.precursor_quality,
    );
    let secondary_compression_material = chosen.compression_ratio.map(|ratio| ratio > 1.0);
    let secondary_targets_met = secondary_lead_time_vs_ewma && secondary_lead_time_vs_threshold;

    let (achieved_forecast_score, forecast_justification) = if primary_success_met
        && secondary_targets_met
    {
        (
            FORECAST_PRIMARY_PLUS_SECONDARY,
            format!(
                "Primary success met for {}: nuisance {:.4} < EWMA {:.4}, recall {} >= threshold {} - {}. Mean lead {} is at least EWMA {} and threshold {}.",
                row_label(&chosen),
                chosen.pass_run_nuisance_proxy,
                ewma_nuisance,
                chosen.failure_recall,
                threshold_recall,
                RECALL_TOLERANCE,
                format_option_f64(chosen.mean_lead_time_runs),
                format_option_f64(metrics.lead_time_summary.mean_ewma_lead_runs),
                format_option_f64(metrics.lead_time_summary.mean_threshold_lead_runs),
            ),
        )
    } else if primary_success_met {
        (
            FORECAST_PRIMARY_ONLY,
            format!(
                "Primary success met for {}: nuisance {:.4} < EWMA {:.4}, recall {} >= threshold {} - {}. Mean lead {} does not meet both secondary lead-time targets.",
                row_label(&chosen),
                chosen.pass_run_nuisance_proxy,
                ewma_nuisance,
                chosen.failure_recall,
                threshold_recall,
                RECALL_TOLERANCE,
                format_option_f64(chosen.mean_lead_time_runs),
            ),
        )
    } else if chosen.pass_run_nuisance_proxy < ewma_nuisance {
        (
            FORECAST_RECALL_SHORTFALL_VALUE,
            format!(
                "Nuisance improved for {} ({:.4} < EWMA {:.4}) but recall {} is below threshold {} - {}.",
                row_label(&chosen),
                chosen.pass_run_nuisance_proxy,
                ewma_nuisance,
                chosen.failure_recall,
                threshold_recall,
                RECALL_TOLERANCE,
            ),
        )
    } else {
        (
            CURRENT_BASELINE_SCORE,
            format!(
                "Primary success condition not met for {}. Nuisance {:.4} vs EWMA {:.4}; recall {} vs threshold {} - {}.",
                row_label(&chosen),
                chosen.pass_run_nuisance_proxy,
                ewma_nuisance,
                chosen.failure_recall,
                threshold_recall,
                RECALL_TOLERANCE,
            ),
        )
    };

    RatingDeltaForecast {
        current_baseline_score: CURRENT_BASELINE_SCORE,
        primary_success_condition: rating_primary_success_condition(),
        recall_tolerance_runs: RECALL_TOLERANCE,
        chosen_configuration: row_label(&chosen),
        primary_success_met,
        secondary_targets_met,
        secondary_lead_time_vs_ewma,
        secondary_lead_time_vs_threshold,
        secondary_precursor_quality_vs_all_feature_dsa,
        secondary_compression_material,
        forecast_score_if_primary_success_only: FORECAST_PRIMARY_ONLY,
        forecast_score_if_primary_plus_secondary_success: FORECAST_PRIMARY_PLUS_SECONDARY,
        achieved_forecast_score,
        forecast_justification,
        category_forecasts: build_category_forecasts(primary_success_met, secondary_targets_met),
        supporting_metrics: ForecastSupportingMetrics {
            chosen_configuration: row_label(&chosen),
            dsa_nuisance: chosen.pass_run_nuisance_proxy,
            ewma_nuisance,
            dsa_recall: chosen.failure_recall,
            threshold_recall,
            recall_tolerance_runs: RECALL_TOLERANCE,
            dsa_mean_lead_time_runs: chosen.mean_lead_time_runs,
            ewma_mean_lead_time_runs: metrics.lead_time_summary.mean_ewma_lead_runs,
            threshold_mean_lead_time_runs: metrics.lead_time_summary.mean_threshold_lead_runs,
            dsa_precursor_quality: chosen.precursor_quality,
            all_feature_dsa_precursor_quality: best_all_features.precursor_quality,
            dsa_compression_ratio: chosen.compression_ratio,
            all_feature_dsa_compression_ratio: best_all_features.compression_ratio,
        },
    }
}

pub fn compute_rating_failure_analysis(
    dsa: &DsaEvaluation,
    metrics: &BenchmarkMetrics,
    cohort_summary: Option<&CohortDsaSummary>,
) -> Option<RatingDeltaFailureAnalysis> {
    let chosen = cohort_summary
        .and_then(|summary| summary.selected_configuration.as_ref())
        .cloned()
        .unwrap_or_else(|| fallback_row_from_dsa(dsa, metrics));
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let threshold_recall = metrics.summary.failure_runs_with_preceding_threshold_signal;
    let primary_success_met = chosen.pass_run_nuisance_proxy < ewma_nuisance
        && chosen.failure_recall + RECALL_TOLERANCE >= threshold_recall;
    if primary_success_met {
        return None;
    }

    let nuisance_gap = (chosen.pass_run_nuisance_proxy - ewma_nuisance).max(0.0);
    let recall_gap_runs = if chosen.failure_recall + RECALL_TOLERANCE >= threshold_recall {
        0
    } else {
        (threshold_recall - RECALL_TOLERANCE - chosen.failure_recall) as i64
    };

    Some(RatingDeltaFailureAnalysis {
        closest_configuration: row_label(&chosen),
        dsa_nuisance: chosen.pass_run_nuisance_proxy,
        ewma_nuisance,
        dsa_recall: chosen.failure_recall,
        threshold_recall,
        recall_tolerance_runs: RECALL_TOLERANCE,
        nuisance_gap,
        recall_gap_runs,
        nuisance_missed_by: if nuisance_gap == 0.0 {
            "no miss; nuisance target was met".into()
        } else if nuisance_gap <= 0.01 {
            "small margin".into()
        } else {
            "large margin".into()
        },
        recall_preserved: recall_gap_runs == 0,
        limiting_factor: determine_rating_limiting_factor(
            cohort_summary,
            &chosen,
            ewma_nuisance,
            threshold_recall,
        ),
    })
}

pub fn write_rating_failure_analysis_md(
    path: &Path,
    analysis: &RatingDeltaFailureAnalysis,
) -> Result<()> {
    let content = format!(
        "# DSA Rating Delta Failure Analysis\n\n\
         ## Closest near-success configuration\n\n\
         - Configuration: {}\n\
         - DSA nuisance: {:.6}\n\
         - EWMA nuisance: {:.6}\n\
         - DSA recall: {}\n\
         - Threshold recall: {}\n\
         - Recall tolerance: {} run(s)\n\n\
         ## Nuisance\n\n\
         - Gap from EWMA: {:.6}\n\
         - Missed by: {}\n\n\
         ## Recall\n\n\
         - Recall gap from threshold - tolerance: {}\n\
         - Recall preserved: {}\n\n\
         ## Limiting factor\n\n\
         {}\n",
        analysis.closest_configuration,
        analysis.dsa_nuisance,
        analysis.ewma_nuisance,
        analysis.dsa_recall,
        analysis.threshold_recall,
        analysis.recall_tolerance_runs,
        analysis.nuisance_gap,
        analysis.nuisance_missed_by,
        analysis.recall_gap_runs,
        analysis.recall_preserved,
        analysis.limiting_factor,
    );
    std::fs::write(path, content)?;
    Ok(())
}

pub fn cohort_report_section(cohorts: &FeatureCohorts, summary: &CohortDsaSummary) -> String {
    let mut out = String::new();
    out.push_str("## Feature-Cohort DSA Selection\n\n");
    out.push_str(&format!(
        "- Ranking formula: `{}`\n- Missingness penalty: {:.1} when `missing_fraction > {:.2}`\n- Selected cohorts: top_4={}, top_8={}, top_16={}, all_features={}\n- Legacy one-run-tolerance cohort gate used inside the bounded sweep: {}\n- Full bounded cohort grid: `W in {{5,10,15}}`, `K in {{2,3,4}}`, `tau in {{2.0,2.5,3.0}}`, `m in {{1,2,3,5}}` where valid\n\n",
        summary.ranking_formula,
        cohorts.missingness_penalty_value,
        cohorts.missingness_penalty_threshold,
        cohorts.top_4.len(),
        cohorts.top_8.len(),
        cohorts.top_16.len(),
        cohorts.all_features.len(),
        summary.primary_success_condition,
    ));

    out.push_str("### Seed-feature check\n\n");
    for seed in &cohorts.seed_feature_report {
        if seed.found_in_ranking {
            out.push_str(&format!(
                "- {}: rank {}, score {:.4}, top_4={}, top_8={}, top_16={}\n",
                seed.feature_name,
                seed.rank.unwrap_or(0),
                seed.candidate_score.unwrap_or(0.0),
                seed.in_top_4,
                seed.in_top_8,
                seed.in_top_16,
            ));
        } else {
            out.push_str(&format!(
                "- {}: not present in the analyzable-feature ranking\n",
                seed.feature_name,
            ));
        }
    }
    out.push('\n');

    out.push_str("### Best row per cohort\n\n");
    out.push_str("| Cohort | W | K | tau | m | Recall | Mean lead | Nuisance | Episodes | Compression | Precursor quality | Legacy gate |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for best in &summary.best_by_cohort {
        let row = &best.best_row;
        out.push_str(&format!(
            "| {} | {} | {} | {:.1} | {} | {}/{} | {} | {:.4} | {} | {} | {} | {} |\n",
            row.cohort_name,
            row.window,
            row.persistence_runs,
            row.alert_tau,
            row.corroborating_m,
            row.failure_recall,
            row.failure_runs,
            format_option_f64(row.mean_lead_time_runs),
            row.pass_run_nuisance_proxy,
            row.dsa_episode_count,
            format_option_f64(row.compression_ratio),
            format_option_f64(row.precursor_quality),
            if row.primary_success { "yes" } else { "no" },
        ));
    }
    out.push('\n');

    if let Some(selected) = &summary.selected_configuration {
        out.push_str("### Best cohort/grid result\n\n");
        out.push_str(&format!(
            "- Selected configuration: {}\n- Recall: {}/{}\n- Mean lead: {}\n- Median lead: {}\n- Nuisance: {:.4} versus EWMA {:.4}\n- Compression ratio: {}\n- Precursor quality: {}\n- Legacy one-run-tolerance cohort gate met: {}\n\n",
            row_label(selected),
            selected.failure_recall,
            selected.failure_runs,
            format_option_f64(selected.mean_lead_time_runs),
            format_option_f64(selected.median_lead_time_runs),
            selected.pass_run_nuisance_proxy,
            selected.ewma_nuisance,
            format_option_f64(selected.compression_ratio),
            format_option_f64(selected.precursor_quality),
            selected.primary_success,
        ));
    }

    if let Some(failure_analysis) = &summary.failure_analysis {
        out.push_str("### Failure analysis\n\n");
        out.push_str(&format!(
            "- Closest cohort: {}\n- Closest grid point: {}\n- Limiting factor: {}\n- Corroboration effect: {}\n- Ranking quality: {}\n- All-feature vs cohort: {}\n- Best near-success source: {}\n\n",
            failure_analysis.closest_cohort,
            failure_analysis.closest_grid_point,
            failure_analysis.limiting_factor,
            failure_analysis.corroboration_effect,
            failure_analysis.ranking_quality_note,
            failure_analysis.all_feature_dsa_vs_cohort_note,
            failure_analysis.best_near_success_source,
        ));
    }

    out.push_str("- Saved artifacts: `dsa_feature_ranking.csv`, `dsa_seed_feature_check.json`, `dsa_feature_cohorts.json`, `dsa_grid_results.csv`, `dsa_cohort_results.csv`, `dsa_cohort_summary.json`, `dsa_cohort_precursor_quality.csv`\n");
    if summary.failure_analysis.is_some() {
        out.push_str("- Failure-analysis artifact: `dsa_cohort_failure_analysis.md`\n");
    }
    out.push('\n');
    out
}

pub fn rating_forecast_report_section(forecast: &RatingDeltaForecast) -> String {
    let mut out = String::new();
    out.push_str("## Rating Delta Forecast\n\n");
    out.push_str(&format!(
        "- Primary success condition: {}\n- Primary success met: {}\n- Chosen configuration: {}\n- Forecast score if primary success only: {:.1}\n- Forecast score if primary + secondary success: {:.1}\n- Forecast score under current measured result: {:.1}\n\n",
        forecast.primary_success_condition,
        forecast.primary_success_met,
        forecast.chosen_configuration,
        forecast.forecast_score_if_primary_success_only,
        forecast.forecast_score_if_primary_plus_secondary_success,
        forecast.achieved_forecast_score,
    ));
    out.push_str("*Forecast only. This is not an achieved score.*\n\n");
    out.push_str(&format!("{}\n\n", forecast.forecast_justification));
    out.push_str(&format!(
        "- DSA nuisance: {:.6}\n- EWMA nuisance: {:.6}\n- DSA recall: {}\n- Threshold recall: {}\n- Recall tolerance: {} run(s)\n- DSA mean lead: {}\n- EWMA mean lead: {}\n- Threshold mean lead: {}\n- DSA precursor quality: {}\n- All-feature DSA precursor quality: {}\n- DSA compression ratio: {}\n- All-feature DSA compression ratio: {}\n\n",
        forecast.supporting_metrics.dsa_nuisance,
        forecast.supporting_metrics.ewma_nuisance,
        forecast.supporting_metrics.dsa_recall,
        forecast.supporting_metrics.threshold_recall,
        forecast.supporting_metrics.recall_tolerance_runs,
        format_option_f64(forecast.supporting_metrics.dsa_mean_lead_time_runs),
        format_option_f64(forecast.supporting_metrics.ewma_mean_lead_time_runs),
        format_option_f64(forecast.supporting_metrics.threshold_mean_lead_time_runs),
        format_option_f64(forecast.supporting_metrics.dsa_precursor_quality),
        format_option_f64(forecast.supporting_metrics.all_feature_dsa_precursor_quality),
        format_option_f64(forecast.supporting_metrics.dsa_compression_ratio),
        format_option_f64(forecast.supporting_metrics.all_feature_dsa_compression_ratio),
    ));
    out
}

fn build_grid_row(
    grid_row_id: usize,
    feature_trace_config_id: usize,
    ranking_strategy: &str,
    ranking_formula: &str,
    cohort_name: &str,
    cohort_size: usize,
    config: &DsaConfig,
    corroborating_m: usize,
    evaluation: &DsaEvaluation,
    metrics: &BenchmarkMetrics,
) -> CohortGridResult {
    let feature_level_active_points = evaluation
        .traces
        .iter()
        .map(|trace| trace.dsa_active.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let feature_level_alert_points = evaluation
        .traces
        .iter()
        .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let threshold_recall = metrics.summary.failure_runs_with_preceding_threshold_signal;
    let ewma_recall = metrics.summary.failure_runs_with_preceding_ewma_signal;
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let threshold_nuisance = metrics.summary.pass_run_threshold_nuisance_rate;
    let primary_success = evaluation.summary.pass_run_nuisance_proxy < ewma_nuisance
        && evaluation.summary.failure_run_recall + RECALL_TOLERANCE >= threshold_recall;

    CohortGridResult {
        ranking_strategy: ranking_strategy.to_string(),
        ranking_formula: ranking_formula.to_string(),
        grid_row_id,
        feature_trace_config_id,
        cohort_name: cohort_name.to_string(),
        cohort_size,
        window: config.window,
        persistence_runs: config.persistence_runs,
        alert_tau: config.alert_tau,
        corroborating_m,
        primary_run_signal: evaluation.run_signals.primary_run_signal.clone(),
        failure_recall: evaluation.summary.failure_run_recall,
        failure_runs: evaluation.summary.failure_runs,
        failure_recall_rate: evaluation.summary.failure_run_recall_rate,
        threshold_recall,
        ewma_recall,
        failure_recall_delta_vs_threshold: evaluation
            .comparison_summary
            .failure_recall_delta_vs_threshold,
        failure_recall_delta_vs_ewma: evaluation.comparison_summary.failure_recall_delta_vs_ewma,
        mean_lead_time_runs: evaluation.summary.mean_lead_time_runs,
        median_lead_time_runs: evaluation.summary.median_lead_time_runs,
        threshold_mean_lead_time_runs: metrics.lead_time_summary.mean_threshold_lead_runs,
        ewma_mean_lead_time_runs: metrics.lead_time_summary.mean_ewma_lead_runs,
        mean_lead_delta_vs_threshold_runs: evaluation.summary.mean_lead_delta_vs_threshold_runs,
        mean_lead_delta_vs_ewma_runs: evaluation.summary.mean_lead_delta_vs_ewma_runs,
        pass_run_nuisance_proxy: evaluation.summary.pass_run_nuisance_proxy,
        numeric_pass_run_nuisance_proxy: evaluation.summary.numeric_primary_pass_run_nuisance_proxy,
        ewma_nuisance,
        threshold_nuisance,
        pass_run_nuisance_delta_vs_ewma: evaluation.summary.pass_run_nuisance_proxy - ewma_nuisance,
        pass_run_nuisance_delta_vs_threshold: evaluation.summary.pass_run_nuisance_proxy
            - threshold_nuisance,
        pass_run_nuisance_delta_vs_numeric_dsa: evaluation
            .comparison_summary
            .pass_run_nuisance_delta_vs_numeric_dsa,
        raw_boundary_episode_count: evaluation.episode_summary.raw_boundary_episode_count,
        dsa_episode_count: evaluation.episode_summary.dsa_episode_count,
        dsa_episodes_preceding_failure: evaluation.episode_summary.dsa_episodes_preceding_failure,
        mean_dsa_episode_length_runs: evaluation.episode_summary.mean_dsa_episode_length_runs,
        max_dsa_episode_length_runs: evaluation.episode_summary.max_dsa_episode_length_runs,
        compression_ratio: evaluation.episode_summary.compression_ratio,
        precursor_quality: evaluation.episode_summary.precursor_quality,
        non_escalating_dsa_episode_fraction: evaluation
            .episode_summary
            .non_escalating_dsa_episode_fraction,
        feature_level_active_points,
        feature_level_alert_points,
        persistence_suppression_fraction: if feature_level_active_points == 0 {
            None
        } else {
            Some(1.0 - feature_level_alert_points as f64 / feature_level_active_points as f64)
        },
        numeric_failure_recall: evaluation.summary.numeric_primary_failure_run_recall,
        policy_vs_numeric_recall_delta: evaluation
            .comparison_summary
            .policy_vs_numeric_recall_delta,
        watch_point_count: evaluation.summary.watch_point_count,
        review_point_count: evaluation.summary.review_point_count,
        escalate_point_count: evaluation.summary.escalate_point_count,
        silenced_point_count: evaluation.summary.silenced_point_count,
        rescued_point_count: evaluation.summary.rescued_point_count,
        rescued_watch_to_review_points: evaluation.summary.rescued_watch_to_review_points,
        rescued_review_to_escalate_points: evaluation.summary.rescued_review_to_escalate_points,
        primary_success,
        primary_success_reason: primary_success_reason(
            evaluation.summary.failure_run_recall,
            threshold_recall,
            evaluation.summary.pass_run_nuisance_proxy,
            ewma_nuisance,
        ),
    }
}

fn build_motif_policy_rows(
    row: &CohortGridResult,
    evaluation: &DsaEvaluation,
) -> Vec<CohortMotifPolicyContributionRow> {
    evaluation
        .motif_policy_contributions
        .iter()
        .map(|contribution| CohortMotifPolicyContributionRow {
            grid_row_id: row.grid_row_id,
            cohort_name: row.cohort_name.clone(),
            cohort_size: row.cohort_size,
            window: row.window,
            persistence_runs: row.persistence_runs,
            alert_tau: row.alert_tau,
            corroborating_m: row.corroborating_m,
            motif_name: contribution.motif_name.clone(),
            alert_class_default: contribution.alert_class_default,
            watch_points: contribution.watch_points,
            review_points: contribution.review_points,
            escalate_points: contribution.escalate_points,
            silent_suppression_points: contribution.silent_suppression_points,
            pass_review_or_escalate_points: contribution.pass_review_or_escalate_points,
            pre_failure_review_or_escalate_points: contribution
                .pre_failure_review_or_escalate_points,
        })
        .collect()
}

fn build_best_by_cohort(rows: &[CohortGridResult]) -> Vec<CohortBestRow> {
    let mut grouped = BTreeMap::<String, Vec<CohortGridResult>>::new();
    for row in rows {
        grouped
            .entry(format!("{} [{}]", row.cohort_name, row.ranking_strategy))
            .or_default()
            .push(row.clone());
    }
    grouped
        .into_iter()
        .filter_map(|(cohort_name, cohort_rows)| {
            best_row(&cohort_rows).map(|best_row| CohortBestRow {
                cohort_name,
                best_row,
            })
        })
        .collect()
}

fn best_row(rows: &[CohortGridResult]) -> Option<CohortGridResult> {
    let success_rows = rows
        .iter()
        .filter(|row| row.primary_success)
        .cloned()
        .collect::<Vec<_>>();
    if !success_rows.is_empty() {
        return success_rows.into_iter().min_by(compare_successful_rows);
    }
    choose_closest_to_success(rows)
}

fn choose_closest_to_success(rows: &[CohortGridResult]) -> Option<CohortGridResult> {
    rows.iter().cloned().min_by(|left, right| {
        primary_success_gap(left)
            .partial_cmp(&primary_success_gap(right))
            .unwrap_or(Ordering::Equal)
            .then_with(|| compare_successful_rows(left, right))
    })
}

fn compare_successful_rows(left: &CohortGridResult, right: &CohortGridResult) -> Ordering {
    left.pass_run_nuisance_proxy
        .partial_cmp(&right.pass_run_nuisance_proxy)
        .unwrap_or(Ordering::Equal)
        .then_with(|| right.failure_recall.cmp(&left.failure_recall))
        .then_with(|| compare_option_f64(right.mean_lead_time_runs, left.mean_lead_time_runs))
        .then_with(|| compare_option_f64(right.precursor_quality, left.precursor_quality))
        .then_with(|| compare_option_f64(right.compression_ratio, left.compression_ratio))
        .then_with(|| left.cohort_name.cmp(&right.cohort_name))
        .then_with(|| left.window.cmp(&right.window))
        .then_with(|| left.persistence_runs.cmp(&right.persistence_runs))
        .then_with(|| left.corroborating_m.cmp(&right.corroborating_m))
}

fn primary_success_gap(row: &CohortGridResult) -> f64 {
    let nuisance_gap = (row.pass_run_nuisance_proxy - row.ewma_nuisance).max(0.0);
    let recall_floor = row.threshold_recall.saturating_sub(RECALL_TOLERANCE);
    let recall_gap =
        recall_floor.saturating_sub(row.failure_recall) as f64 / row.threshold_recall.max(1) as f64;
    nuisance_gap + recall_gap
}

fn corroboration_effect(rows: &[CohortGridResult]) -> String {
    let best_m1 = rows
        .iter()
        .filter(|row| row.corroborating_m == 1)
        .min_by(|left, right| {
            primary_success_gap(left)
                .partial_cmp(&primary_success_gap(right))
                .unwrap_or(Ordering::Equal)
        });
    let best_m_gt_1 = rows
        .iter()
        .filter(|row| row.corroborating_m > 1)
        .min_by(|left, right| {
            primary_success_gap(left)
                .partial_cmp(&primary_success_gap(right))
                .unwrap_or(Ordering::Equal)
        });
    match (best_m1, best_m_gt_1) {
        (Some(best_m1), Some(best_m_gt_1)) => {
            let m1_gap = primary_success_gap(best_m1);
            let higher_gap = primary_success_gap(best_m_gt_1);
            if higher_gap + 1.0e-9 < m1_gap {
                format!(
                    "Cross-feature corroboration improved the closest result: {} beat {} with gap {:.4} vs {:.4}.",
                    row_label(best_m_gt_1),
                    row_label(best_m1),
                    higher_gap,
                    m1_gap,
                )
            } else if m1_gap + 1.0e-9 < higher_gap {
                format!(
                    "Cross-feature corroboration degraded the closest result: {} beat {} with gap {:.4} vs {:.4}.",
                    row_label(best_m1),
                    row_label(best_m_gt_1),
                    m1_gap,
                    higher_gap,
                )
            } else {
                "Cross-feature corroboration produced effectively tied nuisance/recall trade-offs."
                    .to_string()
            }
        }
        _ => "Cross-feature corroboration effect could not be separated from the saved sweep."
            .to_string(),
    }
}

fn limiting_factor_from_row(
    row: Option<&CohortGridResult>,
    ewma_nuisance: f64,
    threshold_recall: usize,
) -> String {
    let Some(row) = row else {
        return "No cohort row was available for limiting-factor analysis.".into();
    };
    let nuisance_ok = row.pass_run_nuisance_proxy < ewma_nuisance;
    let recall_ok = row.failure_recall + RECALL_TOLERANCE >= threshold_recall;
    match (nuisance_ok, recall_ok) {
        (false, true) => "Nuisance was the limiting factor.".into(),
        (true, false) => "Recall was the limiting factor.".into(),
        (false, false) => "Both nuisance and recall remained limiting factors.".into(),
        (true, true) => "The legacy one-run-tolerance cohort gate was met on this row.".into(),
    }
}

fn build_failure_analysis(
    rows: &[CohortGridResult],
    motif_policy_rows: &[CohortMotifPolicyContributionRow],
    cohorts: &FeatureCohorts,
    ewma_nuisance: f64,
    threshold_recall: usize,
    selected_row: Option<&CohortGridResult>,
    corroboration_effect: &str,
    limiting_factor: &str,
) -> Option<CohortFailureAnalysis> {
    let closest = choose_closest_to_success(rows)?;
    let best_all_features = rows
        .iter()
        .filter(|row| row.cohort_name == "all_features")
        .cloned()
        .collect::<Vec<_>>();
    let best_ranked = rows
        .iter()
        .filter(|row| row.cohort_name != "all_features")
        .cloned()
        .collect::<Vec<_>>();
    let best_all_features = best_row(&best_all_features);
    let best_ranked = best_row(&best_ranked);
    let all_feature_dsa_vs_cohort_note = match (&best_all_features, &best_ranked) {
        (Some(best_all_features), Some(best_ranked)) => {
            let all_gap = primary_success_gap(best_all_features);
            let ranked_gap = primary_success_gap(best_ranked);
            if ranked_gap + 1.0e-9 < all_gap {
                format!(
                    "Ranked cohort DSA was better than all-feature DSA: {} beat {}.",
                    row_label(best_ranked),
                    row_label(best_all_features),
                )
            } else if all_gap + 1.0e-9 < ranked_gap {
                format!(
                    "All-feature DSA remained better than the ranked cohorts: {} beat {}.",
                    row_label(best_all_features),
                    row_label(best_ranked),
                )
            } else {
                "All-feature DSA and the best ranked cohort were effectively tied.".into()
            }
        }
        _ => {
            "Not enough saved cohort rows to compare all-feature DSA against ranked cohorts.".into()
        }
    };

    let ranking_reference = best_ranked
        .as_ref()
        .map(|row| row.cohort_name.clone())
        .unwrap_or_else(|| closest.cohort_name.clone());
    let ranking_quality_note = ranking_quality_note(cohorts, &ranking_reference);
    let best_near_success_source = selected_row
        .map(row_label)
        .unwrap_or_else(|| row_label(&closest));
    let policy_vs_numeric_note = policy_vs_numeric_note(&closest);
    let nuisance_motif_classes = dominant_motif_note(motif_policy_rows, closest.grid_row_id, true);
    let useful_precursor_motif_classes =
        dominant_motif_note(motif_policy_rows, closest.grid_row_id, false);

    Some(CohortFailureAnalysis {
        closest_cohort: closest.cohort_name.clone(),
        closest_grid_point: row_grid_point(&closest),
        closest_policy_setting: row_label(&closest),
        closest_nuisance: closest.pass_run_nuisance_proxy,
        closest_recall: closest.failure_recall,
        ewma_nuisance,
        threshold_recall,
        limiting_factor: limiting_factor.to_string(),
        corroboration_effect: corroboration_effect.to_string(),
        policy_vs_numeric_note,
        ranking_quality_note,
        all_feature_dsa_vs_cohort_note,
        best_near_success_source,
        nuisance_motif_classes,
        useful_precursor_motif_classes,
    })
}

fn policy_vs_numeric_note(row: &CohortGridResult) -> String {
    if row.pass_run_nuisance_delta_vs_numeric_dsa < 0.0 && row.policy_vs_numeric_recall_delta >= 0 {
        format!(
            "Policy suppression helped relative to numeric-only DSA: nuisance improved from {:.4} to {:.4} without recall loss ({} to {}).",
            row.numeric_pass_run_nuisance_proxy,
            row.pass_run_nuisance_proxy,
            row.numeric_failure_recall,
            row.failure_recall,
        )
    } else if row.pass_run_nuisance_delta_vs_numeric_dsa < 0.0 {
        format!(
            "Policy suppression reduced nuisance relative to numeric-only DSA ({:.4} to {:.4}) but lost recall ({} to {}).",
            row.numeric_pass_run_nuisance_proxy,
            row.pass_run_nuisance_proxy,
            row.numeric_failure_recall,
            row.failure_recall,
        )
    } else if row.pass_run_nuisance_delta_vs_numeric_dsa > 0.0 {
        format!(
            "Policy suppression hurt nuisance relative to numeric-only DSA: {:.4} vs {:.4}.",
            row.pass_run_nuisance_proxy, row.numeric_pass_run_nuisance_proxy,
        )
    } else {
        "Policy suppression and numeric-only DSA were effectively tied on pass-run nuisance.".into()
    }
}

fn dominant_motif_note(
    motif_policy_rows: &[CohortMotifPolicyContributionRow],
    grid_row_id: usize,
    nuisance: bool,
) -> String {
    let mut rows = motif_policy_rows
        .iter()
        .filter(|row| row.grid_row_id == grid_row_id)
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return "No motif-policy contribution rows were available.".into();
    }
    rows.sort_by(|left, right| {
        let left_score = if nuisance {
            left.pass_review_or_escalate_points
        } else {
            left.pre_failure_review_or_escalate_points
        };
        let right_score = if nuisance {
            right.pass_review_or_escalate_points
        } else {
            right.pre_failure_review_or_escalate_points
        };
        right_score
            .cmp(&left_score)
            .then_with(|| left.motif_name.cmp(&right.motif_name))
    });
    let top = rows[0];
    let score = if nuisance {
        top.pass_review_or_escalate_points
    } else {
        top.pre_failure_review_or_escalate_points
    };
    if nuisance {
        format!(
            "{} ({:?}) contributed the most pass-run Review/Escalate points: {}.",
            top.motif_name, top.alert_class_default, score
        )
    } else {
        format!(
            "{} ({:?}) contributed the most pre-failure Review/Escalate points: {}.",
            top.motif_name, top.alert_class_default, score
        )
    }
}

fn ranking_quality_note(cohorts: &FeatureCohorts, cohort_name: &str) -> String {
    let selected = cohort_members(cohorts, cohort_name);
    if selected.is_empty() {
        return "Ranking quality could not be assessed because the selected cohort was empty."
            .to_string();
    }

    let selected_violation_ratio = average_ratio(
        selected,
        |member| member.dsfb_violation_points,
        |member| member.dsfb_boundary_points,
    );
    let selected_threshold_ratio = average_ratio(
        selected,
        |member| member.threshold_alarm_points,
        |member| member.dsfb_boundary_points,
    );
    let all_violation_ratio = average_ratio(
        &cohorts.all_features,
        |member| member.dsfb_violation_points,
        |member| member.dsfb_boundary_points,
    );
    let all_threshold_ratio = average_ratio(
        &cohorts.all_features,
        |member| member.threshold_alarm_points,
        |member| member.dsfb_boundary_points,
    );

    if selected_violation_ratio > all_violation_ratio * 1.25
        || selected_threshold_ratio > all_threshold_ratio * 1.25
    {
        format!(
            "Ranking appears to have over-selected noisy features: cohort violation/boundary ratio {:.4} vs all-feature {:.4}, threshold/boundary ratio {:.4} vs all-feature {:.4}.",
            selected_violation_ratio,
            all_violation_ratio,
            selected_threshold_ratio,
            all_threshold_ratio,
        )
    } else {
        format!(
            "Ranking did not obviously over-select noisy features: cohort violation/boundary ratio {:.4} vs all-feature {:.4}, threshold/boundary ratio {:.4} vs all-feature {:.4}.",
            selected_violation_ratio,
            all_violation_ratio,
            selected_threshold_ratio,
            all_threshold_ratio,
        )
    }
}

fn rebuild_selected_evaluation(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    cohorts: &FeatureCohorts,
    pre_failure_lookback_runs: usize,
    row: &CohortGridResult,
) -> Result<DsaEvaluation> {
    let base_config = DsaConfig {
        window: row.window,
        persistence_runs: row.persistence_runs,
        alert_tau: row.alert_tau,
        corroborating_feature_count_min: 1,
    };
    let base_evaluation = evaluate_dsa(
        dataset,
        nominal,
        residuals,
        signs,
        baselines,
        grammar,
        &base_config,
        pre_failure_lookback_runs,
    )?;
    let feature_indices = cohort_members(cohorts, &row.cohort_name)
        .iter()
        .map(|member| member.feature_index)
        .collect::<Vec<_>>();
    project_dsa_to_cohort(
        dataset,
        nominal,
        residuals,
        baselines,
        grammar,
        &base_evaluation,
        &feature_indices,
        row.corroborating_m,
        pre_failure_lookback_runs,
        &row.cohort_name,
    )
}

fn fallback_row_from_dsa(dsa: &DsaEvaluation, metrics: &BenchmarkMetrics) -> CohortGridResult {
    CohortGridResult {
        ranking_strategy: "selected".into(),
        ranking_formula: "selected evaluation".into(),
        grid_row_id: 0,
        feature_trace_config_id: 0,
        cohort_name: "default_all_features".into(),
        cohort_size: dsa.summary.analyzable_feature_count,
        window: dsa.summary.config.window,
        persistence_runs: dsa.summary.config.persistence_runs,
        alert_tau: dsa.summary.config.alert_tau,
        corroborating_m: dsa.summary.config.corroborating_feature_count_min,
        primary_run_signal: dsa.run_signals.primary_run_signal.clone(),
        failure_recall: dsa.summary.failure_run_recall,
        failure_runs: dsa.summary.failure_runs,
        failure_recall_rate: dsa.summary.failure_run_recall_rate,
        threshold_recall: metrics.summary.failure_runs_with_preceding_threshold_signal,
        ewma_recall: metrics.summary.failure_runs_with_preceding_ewma_signal,
        failure_recall_delta_vs_threshold: dsa.comparison_summary.failure_recall_delta_vs_threshold,
        failure_recall_delta_vs_ewma: dsa.comparison_summary.failure_recall_delta_vs_ewma,
        mean_lead_time_runs: dsa.summary.mean_lead_time_runs,
        median_lead_time_runs: dsa.summary.median_lead_time_runs,
        threshold_mean_lead_time_runs: metrics.lead_time_summary.mean_threshold_lead_runs,
        ewma_mean_lead_time_runs: metrics.lead_time_summary.mean_ewma_lead_runs,
        mean_lead_delta_vs_threshold_runs: dsa.summary.mean_lead_delta_vs_threshold_runs,
        mean_lead_delta_vs_ewma_runs: dsa.summary.mean_lead_delta_vs_ewma_runs,
        pass_run_nuisance_proxy: dsa.summary.pass_run_nuisance_proxy,
        numeric_pass_run_nuisance_proxy: dsa.summary.numeric_primary_pass_run_nuisance_proxy,
        ewma_nuisance: metrics.summary.pass_run_ewma_nuisance_rate,
        threshold_nuisance: metrics.summary.pass_run_threshold_nuisance_rate,
        pass_run_nuisance_delta_vs_ewma: dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        pass_run_nuisance_delta_vs_threshold: dsa
            .comparison_summary
            .pass_run_nuisance_delta_vs_threshold,
        pass_run_nuisance_delta_vs_numeric_dsa: dsa
            .comparison_summary
            .pass_run_nuisance_delta_vs_numeric_dsa,
        raw_boundary_episode_count: dsa.episode_summary.raw_boundary_episode_count,
        dsa_episode_count: dsa.episode_summary.dsa_episode_count,
        dsa_episodes_preceding_failure: dsa.episode_summary.dsa_episodes_preceding_failure,
        mean_dsa_episode_length_runs: dsa.episode_summary.mean_dsa_episode_length_runs,
        max_dsa_episode_length_runs: dsa.episode_summary.max_dsa_episode_length_runs,
        compression_ratio: dsa.episode_summary.compression_ratio,
        precursor_quality: dsa.episode_summary.precursor_quality,
        non_escalating_dsa_episode_fraction: dsa
            .episode_summary
            .non_escalating_dsa_episode_fraction,
        feature_level_active_points: dsa
            .traces
            .iter()
            .map(|trace| trace.dsa_active.iter().filter(|flag| **flag).count())
            .sum(),
        feature_level_alert_points: dsa
            .traces
            .iter()
            .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
            .sum(),
        persistence_suppression_fraction: overall_persistence_suppression_fraction(dsa),
        numeric_failure_recall: dsa.summary.numeric_primary_failure_run_recall,
        policy_vs_numeric_recall_delta: dsa.comparison_summary.policy_vs_numeric_recall_delta,
        watch_point_count: dsa.summary.watch_point_count,
        review_point_count: dsa.summary.review_point_count,
        escalate_point_count: dsa.summary.escalate_point_count,
        silenced_point_count: dsa.summary.silenced_point_count,
        rescued_point_count: dsa.summary.rescued_point_count,
        rescued_watch_to_review_points: dsa.summary.rescued_watch_to_review_points,
        rescued_review_to_escalate_points: dsa.summary.rescued_review_to_escalate_points,
        primary_success: dsa.summary.pass_run_nuisance_proxy
            < metrics.summary.pass_run_ewma_nuisance_rate
            && dsa.summary.failure_run_recall + RECALL_TOLERANCE
                >= metrics.summary.failure_runs_with_preceding_threshold_signal,
        primary_success_reason: primary_success_reason(
            dsa.summary.failure_run_recall,
            metrics.summary.failure_runs_with_preceding_threshold_signal,
            dsa.summary.pass_run_nuisance_proxy,
            metrics.summary.pass_run_ewma_nuisance_rate,
        ),
    }
}

fn best_all_features_row(summary: &CohortDsaSummary) -> Option<&CohortGridResult> {
    summary
        .best_by_cohort
        .iter()
        .find(|best| best.cohort_name.starts_with("all_features"))
        .map(|best| &best.best_row)
}

fn determine_rating_limiting_factor(
    cohort_summary: Option<&CohortDsaSummary>,
    chosen: &CohortGridResult,
    ewma_nuisance: f64,
    threshold_recall: usize,
) -> String {
    if let Some(summary) = cohort_summary {
        let best_all_features = best_all_features_row(summary);
        let best_ranked = summary
            .best_by_cohort
            .iter()
            .filter(|best| best.cohort_name != "all_features")
            .map(|best| &best.best_row)
            .min_by(|left, right| {
                primary_success_gap(left)
                    .partial_cmp(&primary_success_gap(right))
                    .unwrap_or(Ordering::Equal)
            });
        if let (Some(best_all_features), Some(best_ranked)) = (best_all_features, best_ranked) {
            if primary_success_gap(best_all_features) + 1.0e-9 < primary_success_gap(best_ranked) {
                return format!(
                    "cohort selection: {} stayed closer to the nuisance/recall target than {}",
                    row_label(best_all_features),
                    row_label(best_ranked),
                );
            }
        }

        let same_cohort_rows = summary
            .cohort_results
            .iter()
            .filter(|row| row.cohort_name == chosen.cohort_name)
            .collect::<Vec<_>>();
        let any_recall_ok = same_cohort_rows
            .iter()
            .any(|row| row.failure_recall + RECALL_TOLERANCE >= threshold_recall);
        let any_nuisance_ok = same_cohort_rows
            .iter()
            .any(|row| row.pass_run_nuisance_proxy < ewma_nuisance);
        let any_joint_success = same_cohort_rows.iter().any(|row| row.primary_success);
        if any_recall_ok && any_nuisance_ok && !any_joint_success {
            return format!(
                "corroboration threshold: cohort {} required different m values to satisfy recall and nuisance separately, but no single corroboration count satisfied both",
                chosen.cohort_name,
            );
        }
    }

    if let Some(persistence_suppression_fraction) = chosen.persistence_suppression_fraction {
        if persistence_suppression_fraction > 0.25
            && chosen.failure_recall + RECALL_TOLERANCE < threshold_recall
        {
            return format!(
                "persistence gate: {:.1}% of feature-level active points were suppressed before alert emission in {}",
                persistence_suppression_fraction * 100.0,
                row_label(chosen),
            );
        }
    }

    format!(
        "DSA score composition: even the closest configuration ({}) left nuisance {:.4} vs EWMA {:.4} and recall {} vs threshold {} - {}",
        row_label(chosen),
        chosen.pass_run_nuisance_proxy,
        ewma_nuisance,
        chosen.failure_recall,
        threshold_recall,
        RECALL_TOLERANCE,
    )
}

fn build_category_forecasts(
    primary_success_met: bool,
    secondary_targets_met: bool,
) -> Vec<CategoryForecast> {
    if primary_success_met && secondary_targets_met {
        vec![
            CategoryForecast {
                category: "empirical_rigor".into(),
                current: "strong".into(),
                forecast: "strong".into(),
                justification:
                    "Measured DSA nuisance reduction with recall preservation and lead-time parity strengthens the empirical package."
                        .into(),
            },
            CategoryForecast {
                category: "operator_usefulness".into(),
                current: "moderate".into(),
                forecast: "strong".into(),
                justification:
                    "Operator-facing nuisance fell below EWMA while recall stayed near threshold level."
                        .into(),
            },
            CategoryForecast {
                category: "sbir_readiness".into(),
                current: "moderate".into(),
                forecast: "strong".into(),
                justification:
                    "A concrete DSA win over scalar monitoring baselines improves commercialization credibility."
                        .into(),
            },
            CategoryForecast {
                category: "licensing_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate-strong".into(),
                justification:
                    "Measured operator value supports licensing discussions, while evidence remains bounded to the current benchmark."
                        .into(),
            },
            CategoryForecast {
                category: "paper_readiness".into(),
                current: "moderate".into(),
                forecast: "strong".into(),
                justification:
                    "Feature-cohort DSA would add a concrete positive empirical result to the paper narrative."
                        .into(),
            },
        ]
    } else if primary_success_met {
        vec![
            CategoryForecast {
                category: "empirical_rigor".into(),
                current: "strong".into(),
                forecast: "strong".into(),
                justification:
                    "Primary success is still a hard empirical result even without full lead-time improvement."
                        .into(),
            },
            CategoryForecast {
                category: "operator_usefulness".into(),
                current: "moderate".into(),
                forecast: "moderate-strong".into(),
                justification:
                    "Lower nuisance with preserved recall is a partial operator-facing improvement."
                        .into(),
            },
            CategoryForecast {
                category: "sbir_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate-strong".into(),
                justification:
                    "Primary success advances readiness even if secondary improvements are incomplete."
                        .into(),
            },
            CategoryForecast {
                category: "licensing_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate".into(),
                justification:
                    "Without stronger secondary metrics the licensing case improves only modestly."
                        .into(),
            },
            CategoryForecast {
                category: "paper_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate-strong".into(),
                justification:
                    "A bounded success claim remains paper-relevant even without stronger lead-time gains."
                        .into(),
            },
        ]
    } else {
        vec![
            CategoryForecast {
                category: "empirical_rigor".into(),
                current: "strong".into(),
                forecast: "strong".into(),
                justification:
                    "The package remains rigorous even when cohort DSA does not clear the forecast target."
                        .into(),
            },
            CategoryForecast {
                category: "operator_usefulness".into(),
                current: "moderate".into(),
                forecast: "moderate".into(),
                justification:
                    "No measured cohort configuration achieved the target nuisance/recall trade-off."
                        .into(),
            },
            CategoryForecast {
                category: "sbir_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate".into(),
                justification: "Without a concrete DSA win, readiness does not materially change."
                    .into(),
            },
            CategoryForecast {
                category: "licensing_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate".into(),
                justification: "No measured licensing-relevant delta was demonstrated.".into(),
            },
            CategoryForecast {
                category: "paper_readiness".into(),
                current: "moderate".into(),
                forecast: "moderate".into(),
                justification:
                    "The negative result remains publishable, but it does not support a stronger forecast."
                        .into(),
            },
        ]
    }
}

fn cohort_member(row: &FeatureRankingRow, cohort_name: &str) -> CohortMember {
    CohortMember {
        feature_index: row.feature_index,
        feature_name: row.feature_name.clone(),
        ranking_score: row.candidate_score,
        dsfb_boundary_points: row.dsfb_raw_boundary_points,
        dsfb_violation_points: row.dsfb_raw_violation_points,
        ewma_alarm_points: row.ewma_alarm_points,
        threshold_alarm_points: row.threshold_alarm_points,
        missing_fraction: row.missing_fraction,
        reason_for_inclusion: format!(
            "Included in {} at rank {} because score {:.4} = z_boundary({:+.4}) - z_violation({:+.4}) + z_ewma({:+.4}) - penalty({:.1}).",
            cohort_name,
            row.rank,
            row.candidate_score,
            row.z_boundary,
            row.z_violation,
            row.z_ewma,
            row.missingness_penalty,
        ),
    }
}

fn seed_membership_note(row: &FeatureRankingRow, cutoff: usize, cohort_name: &str) -> String {
    if row.rank <= cutoff {
        format!(
            "Included in {} at rank {} with score {:.4}.",
            cohort_name, row.rank, row.candidate_score
        )
    } else {
        format!(
            "Excluded from {} because rank {} is outside the cutoff. Score {:.4} = z_boundary({:+.4}) - z_violation({:+.4}) + z_ewma({:+.4}) - penalty({:.1}).",
            cohort_name,
            row.rank,
            row.candidate_score,
            row.z_boundary,
            row.z_violation,
            row.z_ewma,
            row.missingness_penalty,
        )
    }
}

fn cohort_members<'a>(cohorts: &'a FeatureCohorts, cohort_name: &str) -> &'a [CohortMember] {
    match cohort_name {
        "top_4" => cohorts.top_4.as_slice(),
        "top_8" => cohorts.top_8.as_slice(),
        "top_16" => cohorts.top_16.as_slice(),
        "all_features" => cohorts.all_features.as_slice(),
        _ => &[],
    }
}

fn average_ratio<T, FNum, FDen>(items: &[T], numerator: FNum, denominator: FDen) -> f64
where
    FNum: Fn(&T) -> usize,
    FDen: Fn(&T) -> usize,
{
    if items.is_empty() {
        return 0.0;
    }
    items
        .iter()
        .map(|item| numerator(item) as f64 / denominator(item).max(1) as f64)
        .sum::<f64>()
        / items.len() as f64
}

fn primary_success_reason(
    failure_recall: usize,
    threshold_recall: usize,
    nuisance: f64,
    ewma_nuisance: f64,
) -> String {
    let nuisance_ok = nuisance < ewma_nuisance;
    let recall_ok = failure_recall + RECALL_TOLERANCE >= threshold_recall;
    if nuisance_ok && recall_ok {
        format!(
            "Success: nuisance {:.4} < EWMA {:.4} and recall {} >= threshold {} - {}.",
            nuisance, ewma_nuisance, failure_recall, threshold_recall, RECALL_TOLERANCE
        )
    } else {
        let mut parts = Vec::new();
        if !nuisance_ok {
            parts.push(format!(
                "nuisance {:.4} >= EWMA {:.4}",
                nuisance, ewma_nuisance
            ));
        }
        if !recall_ok {
            parts.push(format!(
                "recall {} < threshold {} - {}",
                failure_recall, threshold_recall, RECALL_TOLERANCE
            ));
        }
        parts.join("; ")
    }
}

fn row_grid_point(row: &CohortGridResult) -> String {
    format!(
        "W={}, K={}, tau={:.1}, m={}",
        row.window, row.persistence_runs, row.alert_tau, row.corroborating_m
    )
}

fn row_label(row: &CohortGridResult) -> String {
    format!(
        "{} [{}] ({})",
        row.cohort_name,
        row.ranking_strategy,
        row_grid_point(row)
    )
}

fn optimization_priority_order() -> Vec<String> {
    vec![
        "1. Maximize delta_nuisance_vs_ewma".into(),
        "2. Preserve or improve recall toward 103/104 and ideally 104/104".into(),
        "3. Maximize precursor quality".into(),
        "4. Preserve or improve mean lead time vs EWMA/threshold".into(),
        "5. Maintain or improve compression ratio without sacrificing recall badly".into(),
    ]
}

fn predeclared_primary_target() -> String {
    format!(
        "delta_nuisance_vs_ewma >= {:.2} AND DSA recall >= 103/104, where delta_nuisance_vs_ewma = (EWMA_nuisance - DSA_nuisance) / EWMA_nuisance",
        PRIMARY_DELTA_TARGET
    )
}

fn predeclared_secondary_target() -> String {
    format!(
        "delta_nuisance_vs_current_dsa >= {:.2} AND DSA recall >= 100/104, where delta_nuisance_vs_current_dsa = (current_policy_dsa_nuisance - optimized_dsa_nuisance) / current_policy_dsa_nuisance",
        SECONDARY_DELTA_TARGET
    )
}

fn primary_success_condition() -> String {
    format!(
        "pass-run nuisance < EWMA nuisance AND failure recall >= threshold recall - {} run(s)",
        RECALL_TOLERANCE
    )
}

fn rating_primary_success_condition() -> String {
    format!(
        "DSA pass-run nuisance < EWMA pass-run nuisance AND DSA failure recall >= threshold failure recall - {} run(s)",
        RECALL_TOLERANCE
    )
}

fn overall_persistence_suppression_fraction(dsa: &DsaEvaluation) -> Option<f64> {
    let active_points = dsa
        .traces
        .iter()
        .map(|trace| trace.dsa_active.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    let alert_points = dsa
        .traces
        .iter()
        .map(|trace| trace.dsa_alert.iter().filter(|flag| **flag).count())
        .sum::<usize>();
    if active_points == 0 {
        None
    } else {
        Some(1.0 - alert_points as f64 / active_points as f64)
    }
}

fn compare_option_gt(left: Option<f64>, right: Option<f64>) -> Option<bool> {
    Some(left? > right?)
}

fn format_option_csv(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.6}")).unwrap_or_default()
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

fn paired_ge(left: Option<f64>, right: Option<f64>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left >= right)
}

fn compare_option_f64(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn mean_std(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 1.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    let std = variance.sqrt();
    (mean, if std > f64::EPSILON { std } else { 1.0 })
}

fn z_score(value: f64, mean: f64, std: f64) -> f64 {
    (value - mean) / std
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{
        BenchmarkMetrics, BenchmarkSummary, BoundaryEpisodeSummary, DensitySummary, LeadTimeSummary,
    };
    use crate::preprocessing::DatasetSummary;

    fn sample_ranking() -> Vec<FeatureRankingRow> {
        vec![
            FeatureRankingRow {
                ranking_strategy: "compression_biased".into(),
                ranking_formula: RANKING_FORMULA.into(),
                feature_index: 58,
                feature_name: "S059".into(),
                dsfb_raw_boundary_points: 682,
                dsfb_persistent_boundary_points: 650,
                dsfb_raw_violation_points: 31,
                dsfb_persistent_violation_points: 4,
                ewma_alarm_points: 624,
                threshold_alarm_points: 31,
                pre_failure_run_hits: 20,
                motif_precision_proxy: Some(0.6),
                recall_rescue_contribution: None,
                missing_fraction: 0.0025,
                z_pre_failure_run_hits: None,
                z_motif_precision_proxy: None,
                z_recall_rescue_contribution: None,
                z_boundary: 5.0,
                z_violation: -0.1,
                z_ewma: 3.0,
                missingness_penalty: 0.0,
                candidate_score: 8.1,
                score_breakdown: "".into(),
                rank: 1,
            },
            FeatureRankingRow {
                ranking_strategy: "compression_biased".into(),
                ranking_formula: RANKING_FORMULA.into(),
                feature_index: 43,
                feature_name: "S044".into(),
                dsfb_raw_boundary_points: 400,
                dsfb_persistent_boundary_points: 380,
                dsfb_raw_violation_points: 18,
                dsfb_persistent_violation_points: 2,
                ewma_alarm_points: 210,
                threshold_alarm_points: 18,
                pre_failure_run_hits: 14,
                motif_precision_proxy: Some(0.5),
                recall_rescue_contribution: None,
                missing_fraction: 0.01,
                z_pre_failure_run_hits: None,
                z_motif_precision_proxy: None,
                z_recall_rescue_contribution: None,
                z_boundary: 1.2,
                z_violation: -0.5,
                z_ewma: 0.9,
                missingness_penalty: 0.0,
                candidate_score: 2.6,
                score_breakdown: "".into(),
                rank: 2,
            },
            FeatureRankingRow {
                ranking_strategy: "compression_biased".into(),
                ranking_formula: RANKING_FORMULA.into(),
                feature_index: 60,
                feature_name: "S061".into(),
                dsfb_raw_boundary_points: 340,
                dsfb_persistent_boundary_points: 320,
                dsfb_raw_violation_points: 18,
                dsfb_persistent_violation_points: 1,
                ewma_alarm_points: 190,
                threshold_alarm_points: 18,
                pre_failure_run_hits: 12,
                motif_precision_proxy: Some(0.45),
                recall_rescue_contribution: None,
                missing_fraction: 0.01,
                z_pre_failure_run_hits: None,
                z_motif_precision_proxy: None,
                z_recall_rescue_contribution: None,
                z_boundary: 1.0,
                z_violation: -0.5,
                z_ewma: 0.8,
                missingness_penalty: 0.0,
                candidate_score: 2.3,
                score_breakdown: "".into(),
                rank: 3,
            },
            FeatureRankingRow {
                ranking_strategy: "compression_biased".into(),
                ranking_formula: RANKING_FORMULA.into(),
                feature_index: 221,
                feature_name: "S222".into(),
                dsfb_raw_boundary_points: 341,
                dsfb_persistent_boundary_points: 300,
                dsfb_raw_violation_points: 7,
                dsfb_persistent_violation_points: 0,
                ewma_alarm_points: 160,
                threshold_alarm_points: 7,
                pre_failure_run_hits: 11,
                motif_precision_proxy: Some(0.55),
                recall_rescue_contribution: None,
                missing_fraction: 0.02,
                z_pre_failure_run_hits: None,
                z_motif_precision_proxy: None,
                z_recall_rescue_contribution: None,
                z_boundary: 1.1,
                z_violation: -0.8,
                z_ewma: 0.6,
                missingness_penalty: 0.0,
                candidate_score: 2.5,
                score_breakdown: "".into(),
                rank: 4,
            },
        ]
    }

    fn sample_metrics_for_delta_target() -> BenchmarkMetrics {
        BenchmarkMetrics {
            summary: BenchmarkSummary {
                dataset_summary: DatasetSummary {
                    run_count: 10,
                    feature_count: 3,
                    pass_count: 8,
                    fail_count: 2,
                    dataset_missing_fraction: 0.0,
                    healthy_pass_runs_requested: 3,
                    healthy_pass_runs_found: 3,
                },
                analyzable_feature_count: 3,
                grammar_imputation_suppression_points: 0,
                threshold_alarm_points: 0,
                ewma_alarm_points: 0,
                cusum_alarm_points: 0,
                run_energy_alarm_points: 0,
                pca_fdc_alarm_points: 0,
                dsfb_raw_boundary_points: 0,
                dsfb_persistent_boundary_points: 0,
                dsfb_raw_violation_points: 0,
                dsfb_persistent_violation_points: 0,
                failure_runs: 104,
                failure_runs_with_preceding_dsfb_raw_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_signal: 0,
                failure_runs_with_preceding_dsfb_raw_boundary_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_boundary_signal: 0,
                failure_runs_with_preceding_dsfb_raw_violation_signal: 0,
                failure_runs_with_preceding_dsfb_persistent_violation_signal: 0,
                failure_runs_with_preceding_ewma_signal: 104,
                failure_runs_with_preceding_cusum_signal: 104,
                failure_runs_with_preceding_run_energy_signal: 0,
                failure_runs_with_preceding_pca_fdc_signal: 103,
                failure_runs_with_preceding_threshold_signal: 104,
                pass_runs: 731,
                pass_runs_with_dsfb_raw_boundary_signal: 0,
                pass_runs_with_dsfb_persistent_boundary_signal: 0,
                pass_runs_with_dsfb_raw_violation_signal: 0,
                pass_runs_with_dsfb_persistent_violation_signal: 0,
                pass_runs_with_ewma_signal: 0,
                pass_runs_with_cusum_signal: 0,
                pass_runs_with_run_energy_signal: 0,
                pass_runs_with_pca_fdc_signal: 0,
                pass_runs_with_threshold_signal: 0,
                pass_run_dsfb_raw_boundary_nuisance_rate: 0.9986329460,
                pass_run_dsfb_persistent_boundary_nuisance_rate: 0.9904,
                pass_run_dsfb_raw_violation_nuisance_rate: 0.9740259740,
                pass_run_dsfb_persistent_violation_nuisance_rate: 0.7724,
                pass_run_ewma_nuisance_rate: 0.9863294600136705,
                pass_run_cusum_nuisance_rate: 1.0,
                pass_run_run_energy_nuisance_rate: 0.5263,
                pass_run_pca_fdc_nuisance_rate: 0.9316,
                pass_run_threshold_nuisance_rate: 0.974025974025974,
            },
            lead_time_summary: LeadTimeSummary {
                failure_runs_with_raw_boundary_lead: 103,
                failure_runs_with_persistent_boundary_lead: 103,
                failure_runs_with_raw_violation_lead: 104,
                failure_runs_with_persistent_violation_lead: 104,
                failure_runs_with_threshold_lead: 104,
                failure_runs_with_ewma_lead: 104,
                failure_runs_with_cusum_lead: 104,
                failure_runs_with_run_energy_lead: 0,
                failure_runs_with_pca_fdc_lead: 103,
                mean_raw_boundary_lead_runs: Some(19.67),
                mean_persistent_boundary_lead_runs: Some(19.54),
                mean_raw_violation_lead_runs: Some(19.56),
                mean_persistent_violation_lead_runs: Some(18.0),
                mean_threshold_lead_runs: Some(19.557692307692307),
                mean_ewma_lead_runs: Some(19.576923076923077),
                mean_cusum_lead_runs: Some(19.58653846153846),
                mean_run_energy_lead_runs: Some(16.31),
                mean_pca_fdc_lead_runs: Some(19.009708737864077),
                mean_raw_boundary_minus_cusum_delta_runs: None,
                mean_raw_boundary_minus_run_energy_delta_runs: None,
                mean_raw_boundary_minus_pca_fdc_delta_runs: None,
                mean_raw_boundary_minus_threshold_delta_runs: None,
                mean_raw_boundary_minus_ewma_delta_runs: None,
                mean_persistent_boundary_minus_cusum_delta_runs: None,
                mean_persistent_boundary_minus_run_energy_delta_runs: None,
                mean_persistent_boundary_minus_pca_fdc_delta_runs: None,
                mean_persistent_boundary_minus_threshold_delta_runs: None,
                mean_persistent_boundary_minus_ewma_delta_runs: None,
                mean_raw_violation_minus_cusum_delta_runs: None,
                mean_raw_violation_minus_run_energy_delta_runs: None,
                mean_raw_violation_minus_pca_fdc_delta_runs: None,
                mean_raw_violation_minus_threshold_delta_runs: None,
                mean_raw_violation_minus_ewma_delta_runs: None,
                mean_persistent_violation_minus_cusum_delta_runs: None,
                mean_persistent_violation_minus_run_energy_delta_runs: None,
                mean_persistent_violation_minus_pca_fdc_delta_runs: None,
                mean_persistent_violation_minus_threshold_delta_runs: None,
                mean_persistent_violation_minus_ewma_delta_runs: None,
            },
            density_summary: DensitySummary {
                density_window: 5,
                mean_raw_boundary_density_failure: 0.0,
                mean_raw_boundary_density_pass: 0.0,
                mean_persistent_boundary_density_failure: 0.0,
                mean_persistent_boundary_density_pass: 0.0,
                mean_raw_violation_density_failure: 0.0,
                mean_raw_violation_density_pass: 0.0,
                mean_persistent_violation_density_failure: 0.0,
                mean_persistent_violation_density_pass: 0.0,
                mean_threshold_density_failure: 0.0,
                mean_threshold_density_pass: 0.0,
                mean_ewma_density_failure: 0.0,
                mean_ewma_density_pass: 0.0,
                mean_cusum_density_failure: 0.0,
                mean_cusum_density_pass: 0.0,
            },
            boundary_episode_summary: BoundaryEpisodeSummary {
                raw_episode_count: 28607,
                persistent_episode_count: 0,
                mean_raw_episode_length: None,
                mean_persistent_episode_length: None,
                max_raw_episode_length: 0,
                max_persistent_episode_length: 0,
                raw_non_escalating_episode_fraction: None,
                persistent_non_escalating_episode_fraction: None,
            },
            dsa_summary: None,
            motif_metrics: Vec::new(),
            per_failure_run_signals: Vec::new(),
            density_metrics: Vec::new(),
            feature_metrics: Vec::new(),
            top_feature_indices: Vec::new(),
        }
    }

    #[test]
    fn cohort_selection_is_deterministic() {
        let first = build_feature_cohorts(&sample_ranking());
        let second = build_feature_cohorts(&sample_ranking());
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
        assert_eq!(first.top_4.len(), 4);
        assert!(first
            .seed_feature_report
            .iter()
            .any(|seed| seed.feature_name == "S059"));
    }

    #[test]
    fn seed_feature_check_artifact_is_emitted_deterministically() {
        let cohorts = build_feature_cohorts(&sample_ranking());
        let artifact = build_seed_feature_check(&cohorts);
        assert_eq!(artifact.requested_seed_features.len(), 6);
        assert_eq!(artifact.seed_feature_report[0].feature_name, "S059");
        assert!(artifact.seed_feature_report[0].in_top_4);
    }

    #[test]
    fn precursor_quality_csv_format_is_stable() {
        let row = CohortGridResult {
            ranking_strategy: "compression_biased".into(),
            ranking_formula: RANKING_FORMULA.into(),
            grid_row_id: 1,
            feature_trace_config_id: 0,
            cohort_name: "top_4".into(),
            cohort_size: 4,
            window: 5,
            persistence_runs: 2,
            alert_tau: 2.0,
            corroborating_m: 2,
            primary_run_signal: "signal".into(),
            failure_recall: 10,
            failure_runs: 12,
            failure_recall_rate: 0.8333,
            threshold_recall: 11,
            ewma_recall: 11,
            failure_recall_delta_vs_threshold: -1,
            failure_recall_delta_vs_ewma: -1,
            mean_lead_time_runs: Some(3.0),
            median_lead_time_runs: Some(3.0),
            threshold_mean_lead_time_runs: Some(2.0),
            ewma_mean_lead_time_runs: Some(2.0),
            mean_lead_delta_vs_threshold_runs: Some(1.0),
            mean_lead_delta_vs_ewma_runs: Some(1.0),
            pass_run_nuisance_proxy: 0.1,
            numeric_pass_run_nuisance_proxy: 0.15,
            ewma_nuisance: 0.2,
            threshold_nuisance: 0.3,
            pass_run_nuisance_delta_vs_ewma: -0.1,
            pass_run_nuisance_delta_vs_threshold: -0.2,
            pass_run_nuisance_delta_vs_numeric_dsa: -0.05,
            raw_boundary_episode_count: 20,
            dsa_episode_count: 4,
            dsa_episodes_preceding_failure: 3,
            mean_dsa_episode_length_runs: Some(2.0),
            max_dsa_episode_length_runs: 5,
            compression_ratio: Some(5.0),
            precursor_quality: Some(0.75),
            non_escalating_dsa_episode_fraction: Some(0.25),
            feature_level_active_points: 8,
            feature_level_alert_points: 4,
            persistence_suppression_fraction: Some(0.5),
            numeric_failure_recall: 11,
            policy_vs_numeric_recall_delta: -1,
            watch_point_count: 3,
            review_point_count: 3,
            escalate_point_count: 1,
            silenced_point_count: 2,
            rescued_point_count: 1,
            rescued_watch_to_review_points: 1,
            rescued_review_to_escalate_points: 0,
            primary_success: true,
            primary_success_reason: "ok".into(),
        };
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("precursor_quality.csv");
        write_precursor_quality_csv(&path, &[row]).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("cohort_name,window,persistence_runs,alert_tau"));
        assert!(content.contains("top_4,5,2,2.000000,2,20,4,3,0.750000,5.000000"));
    }

    #[test]
    fn delta_target_assessment_reports_unreached_forty_percent_goal() {
        let baseline_row = CohortGridResult {
            ranking_strategy: "compression_biased".into(),
            ranking_formula: RANKING_FORMULA.into(),
            grid_row_id: 0,
            feature_trace_config_id: 0,
            cohort_name: "all_features".into(),
            cohort_size: 100,
            window: 10,
            persistence_runs: 2,
            alert_tau: 2.0,
            corroborating_m: 1,
            primary_run_signal: "signal".into(),
            failure_recall: 100,
            failure_runs: 104,
            failure_recall_rate: 100.0 / 104.0,
            threshold_recall: 104,
            ewma_recall: 104,
            failure_recall_delta_vs_threshold: -4,
            failure_recall_delta_vs_ewma: -4,
            mean_lead_time_runs: Some(18.7),
            median_lead_time_runs: Some(20.0),
            threshold_mean_lead_time_runs: Some(19.557692307692307),
            ewma_mean_lead_time_runs: Some(19.576923076923077),
            mean_lead_delta_vs_threshold_runs: Some(-0.8577),
            mean_lead_delta_vs_ewma_runs: Some(-0.8769),
            pass_run_nuisance_proxy: 0.8311688311688312,
            numeric_pass_run_nuisance_proxy: 0.9330,
            ewma_nuisance: 0.9863294600136705,
            threshold_nuisance: 0.974025974025974,
            pass_run_nuisance_delta_vs_ewma: -0.15516062884483928,
            pass_run_nuisance_delta_vs_threshold: -0.1428571428571428,
            pass_run_nuisance_delta_vs_numeric_dsa: -0.10183116883116884,
            raw_boundary_episode_count: 28607,
            dsa_episode_count: 65,
            dsa_episodes_preceding_failure: 52,
            mean_dsa_episode_length_runs: Some(17.0),
            max_dsa_episode_length_runs: 110,
            compression_ratio: Some(440.10769230769233),
            precursor_quality: Some(0.8),
            non_escalating_dsa_episode_fraction: Some(0.0),
            feature_level_active_points: 0,
            feature_level_alert_points: 0,
            persistence_suppression_fraction: None,
            numeric_failure_recall: 99,
            policy_vs_numeric_recall_delta: 1,
            watch_point_count: 0,
            review_point_count: 0,
            escalate_point_count: 0,
            silenced_point_count: 0,
            rescued_point_count: 0,
            rescued_watch_to_review_points: 0,
            rescued_review_to_escalate_points: 0,
            primary_success: false,
            primary_success_reason: "baseline".into(),
        };
        let optimized_row = CohortGridResult {
            ranking_strategy: "compression_biased".into(),
            ranking_formula: RANKING_FORMULA.into(),
            grid_row_id: 1,
            feature_trace_config_id: 0,
            cohort_name: "all_features".into(),
            cohort_size: 100,
            window: 10,
            persistence_runs: 4,
            alert_tau: 2.0,
            corroborating_m: 1,
            primary_run_signal: "signal".into(),
            failure_recall: 103,
            failure_runs: 104,
            failure_recall_rate: 103.0 / 104.0,
            threshold_recall: 104,
            ewma_recall: 104,
            failure_recall_delta_vs_threshold: -1,
            failure_recall_delta_vs_ewma: -1,
            mean_lead_time_runs: Some(17.980582524271846),
            median_lead_time_runs: Some(20.0),
            threshold_mean_lead_time_runs: Some(19.557692307692307),
            ewma_mean_lead_time_runs: Some(19.576923076923077),
            mean_lead_delta_vs_threshold_runs: Some(-1.7475728155339805),
            mean_lead_delta_vs_ewma_runs: Some(-1.766990291262136),
            pass_run_nuisance_proxy: 0.7997265892002734,
            numeric_pass_run_nuisance_proxy: 0.9180,
            ewma_nuisance: 0.9863294600136705,
            threshold_nuisance: 0.974025974025974,
            pass_run_nuisance_delta_vs_ewma: -0.1866028708133971,
            pass_run_nuisance_delta_vs_threshold: -0.17429938482570062,
            pass_run_nuisance_delta_vs_numeric_dsa: -0.11827341079972659,
            raw_boundary_episode_count: 28607,
            dsa_episode_count: 73,
            dsa_episodes_preceding_failure: 57,
            mean_dsa_episode_length_runs: Some(17.041095890410958),
            max_dsa_episode_length_runs: 110,
            compression_ratio: Some(391.8767123287671),
            precursor_quality: Some(0.7808219178082192),
            non_escalating_dsa_episode_fraction: Some(0.0),
            feature_level_active_points: 0,
            feature_level_alert_points: 0,
            persistence_suppression_fraction: None,
            numeric_failure_recall: 99,
            policy_vs_numeric_recall_delta: 4,
            watch_point_count: 0,
            review_point_count: 0,
            escalate_point_count: 0,
            silenced_point_count: 0,
            rescued_point_count: 57,
            rescued_watch_to_review_points: 57,
            rescued_review_to_escalate_points: 0,
            primary_success: true,
            primary_success_reason: "selected".into(),
        };
        let metrics = sample_metrics_for_delta_target();
        let assessment = compute_delta_target_assessment(
            &optimized_row,
            std::slice::from_ref(&optimized_row),
            std::slice::from_ref(&optimized_row),
            &baseline_row,
            &metrics,
        );

        assert!(!assessment.primary_target_met);
        assert!(!assessment.ideal_target_met);
        assert!(!assessment.secondary_target_met);
        assert!(
            (assessment.selected_configuration.delta_nuisance_vs_ewma - 0.18918918918918917).abs()
                < 1.0e-9
        );
        assert!(
            (assessment
                .selected_configuration
                .delta_nuisance_vs_current_dsa
                - 0.037828947368421136)
                .abs()
                < 1.0e-9
        );
    }

    #[test]
    fn delta_target_assessment_prefers_best_recall_preserving_delta_row() {
        let template_row = CohortGridResult {
            ranking_strategy: "compression_biased".into(),
            ranking_formula: RANKING_FORMULA.into(),
            grid_row_id: 1,
            feature_trace_config_id: 0,
            cohort_name: "all_features".into(),
            cohort_size: 100,
            window: 10,
            persistence_runs: 4,
            alert_tau: 2.0,
            corroborating_m: 1,
            primary_run_signal: "signal".into(),
            failure_recall: 103,
            failure_runs: 104,
            failure_recall_rate: 103.0 / 104.0,
            threshold_recall: 104,
            ewma_recall: 104,
            failure_recall_delta_vs_threshold: -1,
            failure_recall_delta_vs_ewma: -1,
            mean_lead_time_runs: Some(17.980582524271846),
            median_lead_time_runs: Some(20.0),
            threshold_mean_lead_time_runs: Some(19.557692307692307),
            ewma_mean_lead_time_runs: Some(19.576923076923077),
            mean_lead_delta_vs_threshold_runs: Some(-1.7475728155339805),
            mean_lead_delta_vs_ewma_runs: Some(-1.766990291262136),
            pass_run_nuisance_proxy: 0.7997265892002734,
            numeric_pass_run_nuisance_proxy: 0.9180,
            ewma_nuisance: 0.9863294600136705,
            threshold_nuisance: 0.974025974025974,
            pass_run_nuisance_delta_vs_ewma: -0.1866028708133971,
            pass_run_nuisance_delta_vs_threshold: -0.17429938482570062,
            pass_run_nuisance_delta_vs_numeric_dsa: -0.11827341079972659,
            raw_boundary_episode_count: 28607,
            dsa_episode_count: 73,
            dsa_episodes_preceding_failure: 57,
            mean_dsa_episode_length_runs: Some(17.041095890410958),
            max_dsa_episode_length_runs: 110,
            compression_ratio: Some(391.8767123287671),
            precursor_quality: Some(0.7808219178082192),
            non_escalating_dsa_episode_fraction: Some(0.0),
            feature_level_active_points: 0,
            feature_level_alert_points: 0,
            persistence_suppression_fraction: None,
            numeric_failure_recall: 99,
            policy_vs_numeric_recall_delta: 4,
            watch_point_count: 0,
            review_point_count: 0,
            escalate_point_count: 0,
            silenced_point_count: 0,
            rescued_point_count: 57,
            rescued_watch_to_review_points: 57,
            rescued_review_to_escalate_points: 0,
            primary_success: true,
            primary_success_reason: "selected".into(),
        };
        let baseline_row = CohortGridResult {
            failure_recall: 100,
            failure_recall_rate: 100.0 / 104.0,
            failure_recall_delta_vs_threshold: -4,
            failure_recall_delta_vs_ewma: -4,
            mean_lead_time_runs: Some(18.7),
            mean_lead_delta_vs_threshold_runs: Some(-0.8577),
            mean_lead_delta_vs_ewma_runs: Some(-0.8769),
            pass_run_nuisance_proxy: 0.8311688311688312,
            numeric_pass_run_nuisance_proxy: 0.9330,
            dsa_episode_count: 65,
            compression_ratio: Some(440.10769230769233),
            precursor_quality: Some(0.8),
            numeric_failure_recall: 99,
            policy_vs_numeric_recall_delta: 1,
            rescued_point_count: 0,
            rescued_watch_to_review_points: 0,
            primary_success: false,
            primary_success_reason: "baseline".into(),
            ..template_row.clone()
        };
        let selected_row = template_row.clone();
        let weaker_recall_preserving_row = CohortGridResult {
            ranking_strategy: "recall_aware".into(),
            persistence_runs: 2,
            pass_run_nuisance_proxy: 0.8386876281613124,
            pass_run_nuisance_delta_vs_ewma: -0.14764183185235812,
            pass_run_nuisance_delta_vs_threshold: -0.13533834586466164,
            pass_run_nuisance_delta_vs_numeric_dsa: -0.09432679900680764,
            dsa_episode_count: 67,
            compression_ratio: Some(426.97014925373134),
            precursor_quality: Some(0.8059701492537313),
            ..template_row.clone()
        };
        let metrics = sample_metrics_for_delta_target();
        let assessment = compute_delta_target_assessment(
            &selected_row,
            std::slice::from_ref(&selected_row),
            &[selected_row.clone(), weaker_recall_preserving_row],
            &baseline_row,
            &metrics,
        );

        let best = assessment
            .best_recall_103_candidate
            .expect("best recall row");
        assert_eq!(best.configuration, row_label(&selected_row));
        assert!((best.delta_nuisance_vs_ewma - 0.18918918918918917).abs() < 1.0e-9);
    }
}
