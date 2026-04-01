//! Deterministic feature-cohort DSA selection and rating-delta forecast.

use crate::baselines::BaselineSet;
use crate::error::Result;
use crate::metrics::BenchmarkMetrics;
use crate::nominal::NominalModel;
use crate::precursor::{evaluate_dsa, project_dsa_to_cohort, DsaConfig, DsaEvaluation};
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
const MISSINGNESS_PENALTY_THRESHOLD: f64 = 0.50;
const MISSINGNESS_PENALTY_VALUE: f64 = 2.0;
const RECALL_TOLERANCE: usize = 1;
const CORROBORATION_SWEEP: &[usize] = &[1, 2, 3, 5];
const DSA_WINDOW_SWEEP: &[usize] = &[5, 10, 15];
const DSA_PERSISTENCE_SWEEP: &[usize] = &[2, 3, 4];
const DSA_TAU_SWEEP: &[f64] = &[2.0, 2.5, 3.0];
const CURRENT_BASELINE_SCORE: f64 = 8.1;
const FORECAST_PRIMARY_ONLY: f64 = 8.8;
const FORECAST_PRIMARY_PLUS_SECONDARY: f64 = 9.1;
const FORECAST_RECALL_SHORTFALL_VALUE: f64 = 8.3;
const SEED_FEATURES: &[&str] = &["S059", "S044", "S061", "S222", "S354", "S173"];

#[derive(Debug, Clone, Serialize)]
pub struct FeatureRankingRow {
    pub feature_index: usize,
    pub feature_name: String,
    pub dsfb_raw_boundary_points: usize,
    pub dsfb_persistent_boundary_points: usize,
    pub dsfb_raw_violation_points: usize,
    pub dsfb_persistent_violation_points: usize,
    pub ewma_alarm_points: usize,
    pub threshold_alarm_points: usize,
    pub missing_fraction: f64,
    pub z_boundary: f64,
    pub z_violation: f64,
    pub z_ewma: f64,
    pub missingness_penalty: f64,
    pub candidate_score: f64,
    pub score_breakdown: String,
    pub rank: usize,
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
    pub ewma_nuisance: f64,
    pub threshold_nuisance: f64,
    pub pass_run_nuisance_delta_vs_ewma: f64,
    pub pass_run_nuisance_delta_vs_threshold: f64,
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
    pub primary_success: bool,
    pub primary_success_reason: String,
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
    pub closest_nuisance: f64,
    pub closest_recall: usize,
    pub ewma_nuisance: f64,
    pub threshold_recall: usize,
    pub limiting_factor: String,
    pub corroboration_effect: String,
    pub ranking_quality_note: String,
    pub all_feature_dsa_vs_cohort_note: String,
    pub best_near_success_source: String,
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
    pub selected_evaluation: DsaEvaluation,
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
                feature_index: feature.feature_index,
                feature_name: feature.feature_name.clone(),
                dsfb_raw_boundary_points: feature.dsfb_raw_boundary_points,
                dsfb_persistent_boundary_points: feature.dsfb_persistent_boundary_points,
                dsfb_raw_violation_points: feature.dsfb_raw_violation_points,
                dsfb_persistent_violation_points: feature.dsfb_persistent_violation_points,
                ewma_alarm_points: feature.ewma_alarm_points,
                threshold_alarm_points: feature.threshold_alarm_points,
                missing_fraction: feature.missing_fraction,
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

pub fn write_feature_ranking_csv(path: &Path, ranking: &[FeatureRankingRow]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
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
        "missing_fraction",
        "z_boundary",
        "z_violation",
        "z_ewma",
        "missingness_penalty",
        "candidate_score",
        "score_breakdown",
    ])?;
    for row in ranking {
        writer.write_record([
            row.rank.to_string(),
            row.feature_index.to_string(),
            row.feature_name.clone(),
            RANKING_FORMULA.to_string(),
            row.dsfb_raw_boundary_points.to_string(),
            row.dsfb_persistent_boundary_points.to_string(),
            row.dsfb_raw_violation_points.to_string(),
            row.dsfb_persistent_violation_points.to_string(),
            row.ewma_alarm_points.to_string(),
            row.threshold_alarm_points.to_string(),
            format!("{:.6}", row.missing_fraction),
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

pub fn build_feature_cohorts(ranking: &[FeatureRankingRow]) -> FeatureCohorts {
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
        ranking_formula: RANKING_FORMULA.into(),
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
    let cohort_specs = [
        ("top_4", cohorts.top_4.as_slice()),
        ("top_8", cohorts.top_8.as_slice()),
        ("top_16", cohorts.top_16.as_slice()),
        ("all_features", cohorts.all_features.as_slice()),
    ];

    let threshold_recall = metrics.summary.failure_runs_with_preceding_threshold_signal;
    let ewma_nuisance = metrics.summary.pass_run_ewma_nuisance_rate;
    let mut grid_rows = Vec::new();
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

                        grid_rows.push(build_grid_row(
                            grid_row_id,
                            feature_trace_config_id,
                            cohort_name,
                            members.len(),
                            &base_config,
                            corroborating_m,
                            &evaluation,
                            metrics,
                        ));
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
        selected_evaluation,
    })
}

pub fn write_cohort_results_csv(path: &Path, results: &[CohortGridResult]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
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
        "ewma_nuisance",
        "threshold_nuisance",
        "pass_run_nuisance_delta_vs_ewma",
        "pass_run_nuisance_delta_vs_threshold",
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
        "primary_success",
        "primary_success_reason",
    ])?;
    for row in results {
        writer.write_record([
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
            format!("{:.6}", row.ewma_nuisance),
            format!("{:.6}", row.threshold_nuisance),
            format!("{:.6}", row.pass_run_nuisance_delta_vs_ewma),
            format!("{:.6}", row.pass_run_nuisance_delta_vs_threshold),
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
            row.primary_success.to_string(),
            row.primary_success_reason.clone(),
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
         - Nuisance: {:.6}\n\
         - Recall: {}\n\
         - EWMA nuisance target: {:.6}\n\
         - Threshold recall target: {}\n\n\
         ## Limiting factor\n\n\
         {}\n\n\
         ## Cross-feature corroboration effect\n\n\
         {}\n\n\
         ## Ranking quality\n\n\
         {}\n\n\
         ## All-feature DSA vs cohort DSA\n\n\
         {}\n\n\
         ## Best near-success source\n\n\
         {}\n",
        analysis.closest_cohort,
        analysis.closest_grid_point,
        analysis.closest_nuisance,
        analysis.closest_recall,
        analysis.ewma_nuisance,
        analysis.threshold_recall,
        analysis.limiting_factor,
        analysis.corroboration_effect,
        analysis.ranking_quality_note,
        analysis.all_feature_dsa_vs_cohort_note,
        analysis.best_near_success_source,
    );
    std::fs::write(path, content)?;
    Ok(())
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
        "- Ranking formula: `{}`\n- Missingness penalty: {:.1} when `missing_fraction > {:.2}`\n- Selected cohorts: top_4={}, top_8={}, top_16={}, all_features={}\n- Primary success condition: {}\n- Full bounded cohort grid: `W in {{5,10,15}}`, `K in {{2,3,4}}`, `tau in {{2.0,2.5,3.0}}`, `m in {{1,2,3,5}}` where valid\n\n",
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
    out.push_str("| Cohort | W | K | tau | m | Recall | Mean lead | Nuisance | Episodes | Compression | Precursor quality | Success |\n");
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
            "- Selected configuration: {}\n- Recall: {}/{}\n- Mean lead: {}\n- Median lead: {}\n- Nuisance: {:.4} versus EWMA {:.4}\n- Compression ratio: {}\n- Precursor quality: {}\n- Primary success met: {}\n\n",
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
        ewma_nuisance,
        threshold_nuisance,
        pass_run_nuisance_delta_vs_ewma: evaluation.summary.pass_run_nuisance_proxy - ewma_nuisance,
        pass_run_nuisance_delta_vs_threshold: evaluation.summary.pass_run_nuisance_proxy
            - threshold_nuisance,
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
        primary_success,
        primary_success_reason: primary_success_reason(
            evaluation.summary.failure_run_recall,
            threshold_recall,
            evaluation.summary.pass_run_nuisance_proxy,
            ewma_nuisance,
        ),
    }
}

fn build_best_by_cohort(rows: &[CohortGridResult]) -> Vec<CohortBestRow> {
    let mut grouped = BTreeMap::<String, Vec<CohortGridResult>>::new();
    for row in rows {
        grouped
            .entry(row.cohort_name.clone())
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
        (true, true) => "The primary success condition was met.".into(),
    }
}

fn build_failure_analysis(
    rows: &[CohortGridResult],
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

    Some(CohortFailureAnalysis {
        closest_cohort: closest.cohort_name.clone(),
        closest_grid_point: row_grid_point(&closest),
        closest_nuisance: closest.pass_run_nuisance_proxy,
        closest_recall: closest.failure_recall,
        ewma_nuisance,
        threshold_recall,
        limiting_factor: limiting_factor.to_string(),
        corroboration_effect: corroboration_effect.to_string(),
        ranking_quality_note,
        all_feature_dsa_vs_cohort_note,
        best_near_success_source,
    })
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
        ewma_nuisance: metrics.summary.pass_run_ewma_nuisance_rate,
        threshold_nuisance: metrics.summary.pass_run_threshold_nuisance_rate,
        pass_run_nuisance_delta_vs_ewma: dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        pass_run_nuisance_delta_vs_threshold: dsa
            .comparison_summary
            .pass_run_nuisance_delta_vs_threshold,
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
        .find(|best| best.cohort_name == "all_features")
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
    format!("{} ({})", row.cohort_name, row_grid_point(row))
}

fn optimization_priority_order() -> Vec<String> {
    vec![
        "1. Reduce nuisance vs raw DSFB boundary".into(),
        "2. Reduce nuisance vs EWMA".into(),
        "3. Preserve recall relative to threshold within tolerance".into(),
        "4. Improve lead time if possible".into(),
    ]
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

    fn sample_ranking() -> Vec<FeatureRankingRow> {
        vec![
            FeatureRankingRow {
                feature_index: 58,
                feature_name: "S059".into(),
                dsfb_raw_boundary_points: 682,
                dsfb_persistent_boundary_points: 650,
                dsfb_raw_violation_points: 31,
                dsfb_persistent_violation_points: 4,
                ewma_alarm_points: 624,
                threshold_alarm_points: 31,
                missing_fraction: 0.0025,
                z_boundary: 5.0,
                z_violation: -0.1,
                z_ewma: 3.0,
                missingness_penalty: 0.0,
                candidate_score: 8.1,
                score_breakdown: "".into(),
                rank: 1,
            },
            FeatureRankingRow {
                feature_index: 43,
                feature_name: "S044".into(),
                dsfb_raw_boundary_points: 400,
                dsfb_persistent_boundary_points: 380,
                dsfb_raw_violation_points: 18,
                dsfb_persistent_violation_points: 2,
                ewma_alarm_points: 210,
                threshold_alarm_points: 18,
                missing_fraction: 0.01,
                z_boundary: 1.2,
                z_violation: -0.5,
                z_ewma: 0.9,
                missingness_penalty: 0.0,
                candidate_score: 2.6,
                score_breakdown: "".into(),
                rank: 2,
            },
            FeatureRankingRow {
                feature_index: 60,
                feature_name: "S061".into(),
                dsfb_raw_boundary_points: 340,
                dsfb_persistent_boundary_points: 320,
                dsfb_raw_violation_points: 18,
                dsfb_persistent_violation_points: 1,
                ewma_alarm_points: 190,
                threshold_alarm_points: 18,
                missing_fraction: 0.01,
                z_boundary: 1.0,
                z_violation: -0.5,
                z_ewma: 0.8,
                missingness_penalty: 0.0,
                candidate_score: 2.3,
                score_breakdown: "".into(),
                rank: 3,
            },
            FeatureRankingRow {
                feature_index: 221,
                feature_name: "S222".into(),
                dsfb_raw_boundary_points: 341,
                dsfb_persistent_boundary_points: 300,
                dsfb_raw_violation_points: 7,
                dsfb_persistent_violation_points: 0,
                ewma_alarm_points: 160,
                threshold_alarm_points: 7,
                missing_fraction: 0.02,
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
            ewma_nuisance: 0.2,
            threshold_nuisance: 0.3,
            pass_run_nuisance_delta_vs_ewma: -0.1,
            pass_run_nuisance_delta_vs_threshold: -0.2,
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
}
