use std::ops::Range;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::engine::event_timeline::{
    build_prefix_semantic_timeline, build_scenario_event_timeline,
};
use crate::engine::types::{
    EngineOutputBundle, GrammarState, ScenarioOutput, SemanticDisposition, VectorSample,
};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

mod upgraded;

use self::upgraded::{prepare_figure_09, prepare_figure_12, prepare_figure_13};

/// Generic machine-readable row for publication-style figure source tables.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FigureSourceRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub plot_title: String,
    pub panel_id: String,
    pub panel_title: String,
    pub x_label: String,
    pub y_label: String,
    pub series_id: String,
    pub series_label: String,
    pub series_kind: String,
    pub color_key: String,
    pub point_order: usize,
    pub x_value: f64,
    pub y_value: f64,
    pub secondary_x_value: Option<f64>,
    pub secondary_y_value: Option<f64>,
    pub x_tick_label: String,
    pub annotation_text: String,
    pub scenario_id: String,
    pub note: String,
}

/// Prepared machine-readable source table for one publication-style figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FigureSourceTable {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub plot_title: String,
    pub generation_timestamp: String,
    pub expected_panel_count: usize,
    pub expected_panel_ids: Vec<String>,
    pub count_like_panel_ids: Vec<String>,
    pub panel_ids: Vec<String>,
    pub series_ids: Vec<String>,
    pub rows: Vec<FigureSourceRow>,
}

/// Machine-readable source row for the detectability comparison summary figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectabilityFigureSourceRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub scenario_id: String,
    pub predicted_upper_bound: Option<f64>,
    pub observed_crossing_time: Option<f64>,
    pub first_boundary_time: Option<f64>,
    pub first_violation_time: Option<f64>,
    pub min_margin_time: Option<f64>,
    pub min_margin_value: Option<f64>,
    pub bound_satisfied: Option<bool>,
    pub note: String,
}

/// Machine-readable source row for the semantic retrieval summary figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticRetrievalFigureSourceRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub representative_rank: usize,
    pub selection_reason: String,
    pub scenario_id: String,
    pub leading_candidate_score: f64,
    pub runner_up_candidate_score: f64,
    pub top_score_margin: f64,
    pub heuristic_bank_entry_count: usize,
    pub prefilter_candidate_count: usize,
    pub heuristic_candidates_post_admissibility: usize,
    pub heuristic_candidates_post_regime: usize,
    pub heuristic_candidates_pre_scope: usize,
    pub heuristic_candidates_post_scope: usize,
    pub heuristics_rejected_by_admissibility: usize,
    pub heuristics_rejected_by_regime: usize,
    pub heuristics_rejected_by_scope: usize,
    pub heuristics_selected_final: usize,
    pub semantic_disposition: String,
    pub disposition_code: i32,
    pub ranked_post_regime_candidate_labels: String,
    pub note: String,
}

/// Machine-readable source row for the internal deterministic comparator summary figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineComparatorFigureSourceRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub comparator_id: String,
    pub comparator_label: String,
    pub triggered_scenario_count: usize,
    pub earliest_first_trigger_time: Option<f64>,
    pub latest_first_trigger_time: Option<f64>,
    pub median_first_trigger_time: Option<f64>,
    pub onset_rank: Option<usize>,
    pub note: String,
}

/// Machine-readable source row for the sweep stability summary figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepSummaryFigureSourceRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub sweep_family: String,
    pub scenario_id: String,
    pub parameter_name: String,
    pub parameter_value: f64,
    pub semantic_disposition: String,
    pub disposition_code: i32,
    pub selected_heuristic_ids: Vec<String>,
    pub note: String,
}

/// Prepares one generic machine-readable source table for every publication-style figure emitted
/// by the crate.
pub fn prepare_publication_figure_source_tables(
    bundle: &EngineOutputBundle,
) -> Result<Vec<FigureSourceTable>> {
    let mut tables = vec![
        prepare_figure_01(bundle)?,
        prepare_figure_02(bundle)?,
        prepare_figure_03(bundle)?,
        prepare_figure_04(bundle)?,
        prepare_figure_05(bundle)?,
        prepare_figure_06(bundle)?,
        prepare_figure_07(bundle)?,
        prepare_figure_08(bundle)?,
        prepare_figure_09(bundle),
        prepare_figure_10(bundle),
        prepare_figure_11(bundle)?,
        prepare_figure_12(bundle),
        prepare_figure_13(bundle),
    ];
    if !bundle.evaluation.sweep_results.is_empty() {
        tables.push(prepare_figure_14(bundle));
    }
    for table in &mut tables {
        finalize_source_table(table, &bundle.run_metadata.timestamp);
    }
    Ok(tables)
}

/// Returns the detectability figure source rows in the same scenario order used for plotting.
pub fn detectability_source_rows(bundle: &EngineOutputBundle) -> Vec<DetectabilityFigureSourceRow> {
    detectability_cases(bundle)
        .into_iter()
        .map(|scenario| DetectabilityFigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            figure_id: "figure_09_detectability_bound".to_string(),
            scenario_id: scenario.record.id.clone(),
            predicted_upper_bound: scenario.detectability.predicted_upper_bound,
            observed_crossing_time: scenario.detectability.observed_crossing_time,
            first_boundary_time: first_non_admissible_time(scenario),
            first_violation_time: first_violation_time(scenario),
            min_margin_time: min_margin_sample(scenario).map(|sample| sample.time),
            min_margin_value: min_margin_sample(scenario).map(|sample| sample.margin),
            bound_satisfied: scenario.detectability.bound_satisfied,
            note: "Source row for the run-specific detectability comparison figure, including theorem-aligned bounds when available and observed grammar or margin timing context when they are not.".to_string(),
        })
        .collect()
}

/// Returns the semantic retrieval summary source rows in the same representative order used for
/// plotting.
pub fn semantic_retrieval_source_rows(
    bundle: &EngineOutputBundle,
) -> Vec<SemanticRetrievalFigureSourceRow> {
    representative_semantic_scenarios(bundle)
        .into_iter()
        .enumerate()
        .map(|(index, (selection_reason, scenario))| SemanticRetrievalFigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            figure_id: "figure_12_semantic_retrieval_heuristics_bank".to_string(),
            representative_rank: index + 1,
            selection_reason,
            scenario_id: scenario.record.id.clone(),
            leading_candidate_score: scenario
                .semantics
                .retrieval_audit
                .ranked_candidates_post_regime
                .first()
                .map(|candidate| candidate.score)
                .unwrap_or(0.0),
            runner_up_candidate_score: scenario
                .semantics
                .retrieval_audit
                .ranked_candidates_post_regime
                .get(1)
                .map(|candidate| candidate.score)
                .unwrap_or(0.0),
            top_score_margin: scenario
                .semantics
                .retrieval_audit
                .ranked_candidates_post_regime
                .first()
                .map(|candidate| candidate.score)
                .unwrap_or(0.0)
                - scenario
                    .semantics
                    .retrieval_audit
                    .ranked_candidates_post_regime
                    .get(1)
                    .map(|candidate| candidate.score)
                    .unwrap_or(0.0),
            heuristic_bank_entry_count: scenario
                .semantics
                .retrieval_audit
                .heuristic_bank_entry_count,
            prefilter_candidate_count: scenario
                .semantics
                .retrieval_audit
                .prefilter_candidate_count,
            heuristic_candidates_post_admissibility: scenario
                .semantics
                .retrieval_audit
                .heuristic_candidates_post_admissibility,
            heuristic_candidates_post_regime: scenario
                .semantics
                .retrieval_audit
                .heuristic_candidates_post_regime,
            heuristic_candidates_pre_scope: scenario
                .semantics
                .retrieval_audit
                .heuristic_candidates_pre_scope,
            heuristic_candidates_post_scope: scenario
                .semantics
                .retrieval_audit
                .heuristic_candidates_post_scope,
            heuristics_rejected_by_admissibility: scenario
                .semantics
                .retrieval_audit
                .heuristics_rejected_by_admissibility,
            heuristics_rejected_by_regime: scenario
                .semantics
                .retrieval_audit
                .heuristics_rejected_by_regime,
            heuristics_rejected_by_scope: scenario
                .semantics
                .retrieval_audit
                .heuristics_rejected_by_scope,
            heuristics_selected_final: scenario
                .semantics
                .retrieval_audit
                .heuristics_selected_final,
            semantic_disposition: format!("{:?}", scenario.semantics.disposition),
            disposition_code: semantic_disposition_code(&scenario.semantics.disposition),
            ranked_post_regime_candidate_labels: scenario
                .semantics
                .retrieval_audit
                .ranked_candidates_post_regime
                .iter()
                .map(|candidate| candidate.short_label.clone())
                .collect::<Vec<_>>()
                .join(" | "),
            note: "Source row for the run-specific semantic retrieval process figure. Leading score and margin are derived from the ranked post-regime candidate list, while the stage counts come from the exported retrieval audit.".to_string(),
        })
        .collect()
}

/// Returns the comparator-trigger source rows used in the deterministic comparator summary.
pub fn baseline_comparator_source_rows(
    bundle: &EngineOutputBundle,
) -> Vec<BaselineComparatorFigureSourceRow> {
    let comparators = [
        ("baseline_residual_threshold", "Residual threshold"),
        ("baseline_moving_average_trend", "Moving-average trend"),
        ("baseline_cusum", "CUSUM"),
        ("baseline_slew_spike", "Slew spike"),
        ("baseline_envelope_interaction", "Envelope interaction"),
        (
            "baseline_innovation_chi_squared_style",
            "Innovation-style squared residual",
        ),
    ];
    let mut onset_lookup = comparators
        .iter()
        .filter_map(|(id, _)| {
            let mut sorted = bundle
                .evaluation
                .baseline_results
                .iter()
                .filter(|result| result.comparator_id == *id)
                .filter_map(|result| result.first_trigger_time)
                .collect::<Vec<_>>();
            if sorted.is_empty() {
                return None;
            }
            sorted.sort_by(|left, right| {
                left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(((*id).to_string(), sorted[sorted.len() / 2]))
        })
        .collect::<Vec<_>>();
    onset_lookup.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    let onset_ranks = onset_lookup
        .iter()
        .enumerate()
        .map(|(index, (comparator_id, _))| (comparator_id.clone(), index + 1))
        .collect::<std::collections::BTreeMap<_, _>>();

    comparators
    .into_iter()
    .map(|(id, label)| {
        let times = bundle
            .evaluation
            .baseline_results
            .iter()
            .filter(|result| result.comparator_id == id)
            .filter_map(|result| result.first_trigger_time)
            .collect::<Vec<_>>();
        let median_first_trigger_time = if times.is_empty() {
            None
        } else {
            let mut sorted = times.clone();
            sorted.sort_by(|left, right| {
                left.partial_cmp(right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(sorted[sorted.len() / 2])
        };
        BaselineComparatorFigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            figure_id: "figure_13_internal_baseline_comparators".to_string(),
            comparator_id: id.to_string(),
            comparator_label: label.to_string(),
            triggered_scenario_count: bundle
                .evaluation
                .summary
                .comparator_trigger_counts
                .get(id)
                .copied()
                .unwrap_or(0),
            earliest_first_trigger_time: times.iter().copied().reduce(f64::min),
            latest_first_trigger_time: times.iter().copied().reduce(f64::max),
            median_first_trigger_time,
            onset_rank: onset_ranks.get(id).copied(),
            note: "Source row for the run-specific internal deterministic comparator figure, including triggered-scenario counts and first-trigger timing summaries.".to_string(),
        }
    })
    .collect()
}

/// Returns the sweep stability source rows used for the deterministic sweep summary figure.
pub fn sweep_summary_source_rows(bundle: &EngineOutputBundle) -> Vec<SweepSummaryFigureSourceRow> {
    bundle
        .evaluation
        .sweep_results
        .iter()
        .map(|result| SweepSummaryFigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            figure_id: "figure_14_sweep_stability_summary".to_string(),
            sweep_family: result.sweep_family.clone(),
            scenario_id: result.scenario_id.clone(),
            parameter_name: result.parameter_name.clone(),
            parameter_value: result.parameter_value,
            semantic_disposition: result.semantic_disposition.clone(),
            disposition_code: disposition_label_code(&result.semantic_disposition),
            selected_heuristic_ids: result.selected_heuristic_ids.clone(),
            note: "Source row for the deterministic sweep disposition-stability figure."
                .to_string(),
        })
        .collect()
}

fn prepare_figure_01(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = source_scenario_or_first(bundle, "gradual_degradation")?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_01_residual_prediction_observation_overview",
        "Residual, Observation, and Prediction Overview",
        &["observation_prediction", "residual_norm"],
        &[],
    );
    push_vector_series(
        &mut table,
        "observation_prediction",
        "Observation and Prediction",
        "time",
        "channel 1 trajectory",
        "observed",
        "observed",
        "line",
        "blue",
        &scenario.record.id,
        &scenario.observed.samples,
        0,
        "Channel-1 observed trajectory rendered in the upper panel.",
    );
    push_vector_series(
        &mut table,
        "observation_prediction",
        "Observation and Prediction",
        "time",
        "channel 1 trajectory",
        "predicted",
        "predicted",
        "line",
        "green",
        &scenario.record.id,
        &scenario.predicted.samples,
        0,
        "Channel-1 predicted trajectory rendered in the upper panel.",
    );
    push_scalar_series(
        &mut table,
        "residual_norm",
        "Residual Norm",
        "time",
        "||r(t)||",
        "residual_norm",
        "residual norm",
        "line",
        "red",
        &scenario.record.id,
        scenario
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm rendered in the lower panel.",
    );
    Ok(table)
}

fn prepare_figure_02(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = source_scenario_or_first(bundle, "abrupt_event")?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_02_drift_and_slew_decomposition",
        "Drift and Slew Decomposition",
        &["residual_norm", "signed_radial_drift", "slew_norm"],
        &[],
    );
    push_scalar_series(
        &mut table,
        "residual_norm",
        "Residual Norm",
        "time",
        "||r(t)||",
        "residual_norm",
        "residual norm",
        "line",
        "blue",
        &scenario.record.id,
        scenario
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm rendered in the first panel.",
    );
    push_scalar_series(
        &mut table,
        "signed_radial_drift",
        "Signed Radial Drift",
        "time",
        "dot(r,d)/||r||",
        "signed_radial_drift",
        "signed radial drift",
        "line",
        "green",
        &scenario.record.id,
        scenario
            .sign
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.projection[1])),
        "Signed radial drift rendered in the second panel.",
    );
    push_scalar_series(
        &mut table,
        "slew_norm",
        "Slew Norm",
        "time",
        "||s(t)||",
        "slew_norm",
        "slew norm",
        "line",
        "red",
        &scenario.record.id,
        scenario
            .slew
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Slew norm rendered in the third panel.",
    );
    Ok(table)
}

fn prepare_figure_03(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = source_scenario_or_first(bundle, "curvature_onset")?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_03_sign_space_projection",
        "Projected Sign Trajectory",
        &["projection_plane"],
        &[],
    );
    let x_label = format!(
        "projection coordinate 1: {}",
        scenario.sign.projection_metadata.axis_labels[0]
    );
    let y_label = format!(
        "projection coordinate 2: {}",
        scenario.sign.projection_metadata.axis_labels[1]
    );
    push_scalar_pair_series(
        &mut table,
        "projection_plane",
        "Projected Sign Trajectory",
        &x_label,
        &y_label,
        "trajectory",
        "projected trajectory",
        "line",
        "teal",
        &scenario.record.id,
        scenario
            .sign
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.projection[0], sample.projection[1])),
        "Projected sign trajectory in the residual-norm versus signed-radial-drift plane.",
    );
    if let Some(first) = scenario.sign.samples.first() {
        push_annotation_point(
            &mut table,
            "projection_plane",
            "Projected Sign Trajectory",
            &x_label,
            &y_label,
            "start_marker",
            "start marker",
            "marker",
            "green",
            &scenario.record.id,
            0,
            first.projection[0],
            first.projection[1],
            "",
            "start",
            "Start-point marker and label.",
        );
    }
    if let Some(last) = scenario.sign.samples.last() {
        push_annotation_point(
            &mut table,
            "projection_plane",
            "Projected Sign Trajectory",
            &x_label,
            &y_label,
            "end_marker",
            "end marker",
            "marker",
            "red",
            &scenario.record.id,
            1,
            last.projection[0],
            last.projection[1],
            "",
            "end",
            "End-point marker and label.",
        );
    }
    let x_values = scenario
        .sign
        .samples
        .iter()
        .map(|sample| sample.projection[0])
        .collect::<Vec<_>>();
    let y_values = scenario
        .sign
        .samples
        .iter()
        .map(|sample| sample.projection[1])
        .collect::<Vec<_>>();
    let (x_min, x_max) = scalar_bounds(&x_values);
    let (y_min, y_max) = scalar_bounds(&y_values);
    push_annotation_point(
        &mut table,
        "projection_plane",
        "Projected Sign Trajectory",
        &x_label,
        &y_label,
        "projection_note",
        "projection note",
        "annotation",
        "slate",
        &scenario.record.id,
        2,
        x_min + (x_max - x_min) * 0.03,
        y_max - (y_max - y_min) * 0.08,
        "",
        &scenario.sign.projection_metadata.note,
        "Projection-construction note rendered inside the chart area.",
    );
    Ok(table)
}

fn prepare_figure_04(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let (monotone, curvature) =
        source_scenario_pair_or_first(bundle, "gradual_degradation", "curvature_onset")?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_04_syntax_comparison",
        "Syntax Comparison",
        &["syntax_comparison"],
        &[],
    );
    push_scalar_series(
        &mut table,
        "syntax_comparison",
        "Syntax Comparison",
        "time",
        "residual norm",
        "monotone_drift",
        "monotone drift",
        "line",
        "blue",
        &monotone.record.id,
        monotone
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the monotone-drift representative case.",
    );
    push_scalar_series(
        &mut table,
        "syntax_comparison",
        "Syntax Comparison",
        "time",
        "residual norm",
        "curvature_dominated",
        "curvature dominated",
        "line",
        "red",
        &curvature.record.id,
        curvature
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the curvature-dominated representative case.",
    );
    Ok(table)
}

fn prepare_figure_05(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    prepare_norm_vs_envelope_table(
        bundle,
        "outward_exit_case_a",
        "figure_05_envelope_exit_under_sustained_outward_drift",
        "Envelope Exit Under Sustained Outward Drift",
    )
}

fn prepare_figure_06(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    prepare_norm_vs_envelope_table(
        bundle,
        "inward_invariance",
        "figure_06_envelope_invariance_under_inward_drift",
        "Envelope Invariance Under Inward-Compatible Drift",
    )
}

fn prepare_figure_07(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let (exit_case, invariance_case) =
        source_scenario_pair_or_first(bundle, "outward_exit_case_a", "inward_invariance")?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_07_exit_invariance_pair_common_envelope",
        "Exit-Invariance Pair on Shared Envelope",
        &["exit_invariance_pair"],
        &[],
    );
    push_scalar_series(
        &mut table,
        "exit_invariance_pair",
        "Exit-Invariance Pair on Shared Envelope",
        "time",
        "norm / envelope radius",
        "shared_envelope",
        "shared envelope",
        "line",
        "slate",
        &exit_case.record.id,
        exit_case
            .envelope
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.radius)),
        "Shared envelope used for both exit and invariance trajectories.",
    );
    push_scalar_series(
        &mut table,
        "exit_invariance_pair",
        "Exit-Invariance Pair on Shared Envelope",
        "time",
        "norm / envelope radius",
        "outward_drift",
        "outward drift",
        "line",
        "red",
        &exit_case.record.id,
        exit_case
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the outward-drift representative case.",
    );
    push_scalar_series(
        &mut table,
        "exit_invariance_pair",
        "Exit-Invariance Pair on Shared Envelope",
        "time",
        "norm / envelope radius",
        "inward_compatible",
        "inward compatible",
        "line",
        "green",
        &invariance_case.record.id,
        invariance_case
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the inward-compatible representative case.",
    );
    Ok(table)
}

fn prepare_figure_08(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let (admissible, detectable) = source_scenario_pair_or_first(
        bundle,
        "magnitude_matched_admissible",
        "magnitude_matched_detectable",
    )?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_08_residual_trajectory_separation",
        "Residual Trajectory Separation",
        &["residual_separation"],
        &[],
    );
    push_scalar_series(
        &mut table,
        "residual_separation",
        "Residual Trajectory Separation",
        "time",
        "||r(t)||",
        "shared_envelope",
        "shared envelope",
        "line",
        "slate",
        &detectable.record.id,
        detectable
            .envelope
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.radius)),
        "Shared envelope used for the magnitude-matched admissible and detectable cases.",
    );
    push_scalar_series(
        &mut table,
        "residual_separation",
        "Residual Trajectory Separation",
        "time",
        "||r(t)||",
        "admissible_case",
        "admissible case",
        "line",
        "blue",
        &admissible.record.id,
        admissible
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the admissible magnitude-matched case.",
    );
    push_scalar_series(
        &mut table,
        "residual_separation",
        "Residual Trajectory Separation",
        "time",
        "||r(t)||",
        "detectable_case",
        "detectable case",
        "line",
        "red",
        &detectable.record.id,
        detectable
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm of the detectable magnitude-matched case.",
    );
    Ok(table)
}

fn prepare_figure_10(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_10_deterministic_pipeline_flow",
        "Deterministic Structural Semiotics Engine",
        &["pipeline_flow"],
        &[],
    );
    for (index, (x0, y0, x1, y1, color_key, ordinal, title, subtitle)) in [
        (
            70.0,
            250.0,
            310.0,
            420.0,
            "blue",
            "1",
            "Residual Layer",
            "r(t) = y(t) - y_hat(t)",
        ),
        (
            370.0,
            250.0,
            610.0,
            420.0,
            "teal",
            "2",
            "Sign Layer",
            "sigma(t) = (r(t), d(t), s(t))",
        ),
        (
            670.0,
            250.0,
            910.0,
            420.0,
            "green",
            "3",
            "Syntax Layer",
            "drift / slew structure",
        ),
        (
            970.0,
            250.0,
            1210.0,
            420.0,
            "gold",
            "4",
            "Grammar Layer",
            "||r(t)|| <= rho(t)",
        ),
        (
            1270.0,
            250.0,
            1510.0,
            420.0,
            "red",
            "5",
            "Semantics Layer",
            "heuristics bank retrieval",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        push_box_row(
            &mut table,
            "pipeline_flow",
            "Deterministic Structural Semiotics Engine",
            "",
            "",
            &format!("node_{}", index + 1),
            title,
            color_key,
            index,
            x0,
            y0,
            x1,
            y1,
            "",
            &format!("{ordinal} | {subtitle}"),
            "Pipeline node rectangle with ordinal and subtitle encoded in the annotation text.",
        );
    }
    for (index, (x0, y0, x1, y1)) in [
        (328.0, 335.0, 352.0, 335.0),
        (628.0, 335.0, 652.0, 335.0),
        (928.0, 335.0, 952.0, 335.0),
        (1228.0, 335.0, 1252.0, 335.0),
    ]
    .into_iter()
    .enumerate()
    {
        push_segment_row(
            &mut table,
            "pipeline_flow",
            "Deterministic Structural Semiotics Engine",
            "",
            "",
            &format!("edge_{}", index + 1),
            "flow edge",
            "black",
            index,
            x0,
            y0,
            x1,
            y1,
            "",
            "Pipeline edge segment.",
        );
    }
    for (order, x, y, label, note) in [
        (
            0,
            800.0,
            110.0,
            "Deterministic Structural Semiotics Engine",
            "Figure title annotation.",
        ),
        (
            1,
            800.0,
            150.0,
            "Fixed layered maps from residual extraction to constrained semantic retrieval",
            "Figure subtitle annotation.",
        ),
        (
            2,
            800.0,
            560.0,
            "Each layer is deterministic and auditable.",
            "Figure footer annotation line 1.",
        ),
        (
            3,
            800.0,
            605.0,
            "Identical inputs yield identical intermediate objects, grammar decisions, and semantic outputs.",
            "Figure footer annotation line 2.",
        ),
    ] {
        push_annotation_point(
            &mut table,
            "pipeline_flow",
            "Deterministic Structural Semiotics Engine",
            "",
            "",
            &format!("annotation_{order}"),
            "annotation",
            "annotation",
            "slate",
            "",
            order,
            x,
            y,
            "",
            label,
            note,
        );
    }
    table
}

fn prepare_figure_11(bundle: &EngineOutputBundle) -> Result<FigureSourceTable> {
    let scenario = source_scenario_or_first(bundle, "grouped_correlated")?;
    let has_multi_channel_fallback = scenario_channel_count(scenario) > 1;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_11_coordinated_group_semiotics",
        "Coordinated Group Semiotics",
        if scenario.coordinated.is_some() || has_multi_channel_fallback {
            &["local_channels", "aggregate_group"]
        } else {
            &["coordination_notice"]
        },
        &[],
    );
    if let Some(coordinated) = scenario.coordinated.as_ref() {
        for (channel_index, label, color_key) in
            [(0, "ch1", "blue"), (1, "ch2", "green"), (2, "ch3", "teal")]
        {
            push_scalar_series(
                &mut table,
                "local_channels",
                "Local Channel Absolute Residuals",
                "time",
                "local |r_i(t)|",
                &format!("local_channel_{}", channel_index + 1),
                label,
                "line",
                color_key,
                &scenario.record.id,
                scenario
                    .residual
                    .samples
                    .iter()
                    .enumerate()
                    .map(|(index, sample)| {
                        (
                            index,
                            sample.time,
                            sample
                                .values
                                .get(channel_index)
                                .copied()
                                .unwrap_or_default()
                                .abs(),
                        )
                    }),
                "Absolute residual of one local channel.",
            );
        }
        push_scalar_series(
            &mut table,
            "local_channels",
            "Local Channel Absolute Residuals",
            "time",
            "local |r_i(t)|",
            "local_envelope",
            "local envelope",
            "line",
            "slate",
            &scenario.record.id,
            scenario
                .envelope
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| (index, sample.time, sample.radius)),
            "Local envelope radius.",
        );
        push_scalar_series(
            &mut table,
            "aggregate_group",
            "Aggregate Group Residual and Envelope",
            "time",
            "aggregate metric",
            "aggregate_abs_mean",
            "aggregate abs mean",
            "line",
            "red",
            &scenario.record.id,
            coordinated
                .points
                .iter()
                .enumerate()
                .map(|(index, point)| (index, point.time, point.aggregate_abs_mean)),
            "Aggregate grouped residual magnitude.",
        );
        push_scalar_series(
            &mut table,
            "aggregate_group",
            "Aggregate Group Residual and Envelope",
            "time",
            "aggregate metric",
            "aggregate_envelope",
            "aggregate envelope",
            "line",
            "slate",
            &scenario.record.id,
            coordinated
                .points
                .iter()
                .enumerate()
                .map(|(index, point)| (index, point.time, point.aggregate_radius)),
            "Aggregate grouped envelope radius.",
        );
    } else if has_multi_channel_fallback {
        for channel_index in 0..scenario_channel_count(scenario).min(3) {
            let channel_label = scenario_channel_label(scenario, channel_index);
            push_scalar_series(
                &mut table,
                "local_channels",
                "Local Channel Absolute Residuals",
                "time",
                "local |r_i(t)|",
                &format!("local_channel_{}", channel_index + 1),
                &channel_label,
                "line",
                ["blue", "green", "teal"][channel_index % 3],
                &scenario.record.id,
                scenario
                    .residual
                    .samples
                    .iter()
                    .enumerate()
                    .map(|(index, sample)| {
                        (
                            index,
                            sample.time,
                            sample
                                .values
                                .get(channel_index)
                                .copied()
                                .unwrap_or_default()
                                .abs(),
                        )
                    }),
                "Absolute residual of one available channel from the selected multi-channel run.",
            );
        }
        push_scalar_series(
            &mut table,
            "local_channels",
            "Local Channel Absolute Residuals",
            "time",
            "local |r_i(t)|",
            "local_envelope",
            "local envelope",
            "line",
            "slate",
            &scenario.record.id,
            scenario
                .envelope
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| (index, sample.time, sample.radius)),
            "Envelope radius reused as a common local reference for the multi-channel fallback view.",
        );
        push_scalar_series(
            &mut table,
            "aggregate_group",
            "Aggregate Multi-Channel Residual and Envelope",
            "time",
            "aggregate metric",
            "aggregate_abs_mean",
            "aggregate abs mean",
            "line",
            "red",
            &scenario.record.id,
            scenario
                .residual
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| {
                    (
                        index,
                        sample.time,
                        mean_abs_values(&sample.values).unwrap_or(sample.norm),
                    )
                }),
            "Fallback aggregate absolute-mean residual across all available channels when no explicit grouped aggregate was configured.",
        );
        push_scalar_series(
            &mut table,
            "aggregate_group",
            "Aggregate Multi-Channel Residual and Envelope",
            "time",
            "aggregate metric",
            "aggregate_envelope",
            "aggregate envelope",
            "line",
            "slate",
            &scenario.record.id,
            scenario
                .envelope
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| (index, sample.time, sample.radius)),
            "Envelope radius paired with the fallback aggregate multi-channel residual view.",
        );
        push_annotation_point(
            &mut table,
            "aggregate_group",
            "Aggregate Multi-Channel Residual and Envelope",
            "time",
            "aggregate metric",
            "aggregate_group_note",
            "aggregate note",
            "annotation",
            "slate",
            &scenario.record.id,
            scenario.residual.samples.len(),
            0.0,
            0.0,
            "",
            "No explicit grouped structure was configured for this run, so the figure falls back to aggregate multi-channel residual magnitude.",
            "Fallback note for multi-channel runs without a configured coordinated aggregate structure.",
        );
    } else {
        push_annotation_point(
            &mut table,
            "coordinated_notice",
            "Coordinated / grouped structure not configured for this run",
            "",
            "",
            "coordinated_notice",
            "annotation",
            "annotation",
            "slate",
            &scenario.record.id,
            0,
            640.0,
            220.0,
            "",
            "Coordinated / grouped structure not configured for this run",
            "Fallback message rendered when no grouped structure exists.",
        );
    }
    Ok(table)
}

#[derive(Clone, Debug)]
struct MatchedWindowSelection {
    channel_index: usize,
    stable_window: Range<usize>,
    departure_window: Range<usize>,
    stable_primary_mean: f64,
    departure_primary_mean: f64,
    stable_meta_mean: f64,
    departure_meta_mean: f64,
    stable_outcome_mean: f64,
    departure_outcome_mean: f64,
}

#[derive(Clone, Debug)]
struct SyntheticScenarioPairSelection<'a> {
    admissible_case: &'a ScenarioOutput,
    detectable_case: &'a ScenarioOutput,
    admissible_primary_mean: f64,
    detectable_primary_mean: f64,
    admissible_meta_mean: f64,
    detectable_meta_mean: f64,
    admissible_outcome_mean: f64,
    detectable_outcome_mean: f64,
}

fn prepare_figure_14(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        "figure_14_sweep_stability_summary",
        "Sweep Semantic Stability",
        &["sweep_semantic_stability"],
        &[],
    );
    for row in sweep_summary_source_rows(bundle) {
        let point_order = table.rows.len();
        push_annotation_point(
            &mut table,
            "sweep_semantic_stability",
            "Sweep Semantic Stability",
            &row.parameter_name,
            "semantic disposition",
            "sweep_disposition",
            "sweep disposition",
            "line-point",
            "teal",
            &row.scenario_id,
            point_order,
            row.parameter_value,
            row.disposition_code as f64,
            "",
            "",
            "Sweep stability line/scatter point.",
        );
    }
    if let Some(summary) = &bundle.evaluation.sweep_summary {
        let point_order = table.rows.len();
        push_annotation_point(
            &mut table,
            "sweep_semantic_stability",
            "Sweep Semantic Stability",
            &summary.sweep_family,
            "semantic disposition",
            "sweep_summary_note",
            "sweep summary note",
            "annotation",
            "slate",
            "",
            point_order,
            0.0,
            3.0,
            "",
            &format!(
                "family={} | members={} | flips={}",
                summary.sweep_family, summary.member_count, summary.disposition_flip_count
            ),
            "Sweep summary annotation rendered inside the chart area.",
        );
    }
    table
}

fn prepare_norm_vs_envelope_table(
    bundle: &EngineOutputBundle,
    scenario_id: &str,
    figure_id: &str,
    plot_title: &str,
) -> Result<FigureSourceTable> {
    let scenario = source_scenario_or_first(bundle, scenario_id)?;
    let mut table = new_source_table(
        &bundle.run_metadata.timestamp,
        &bundle.run_metadata.bank.bank_version,
        figure_id,
        plot_title,
        &["norm_vs_envelope"],
        &[],
    );
    push_scalar_series(
        &mut table,
        "norm_vs_envelope",
        plot_title,
        "time",
        "norm / envelope radius",
        "envelope",
        "envelope",
        "line",
        "slate",
        &scenario.record.id,
        scenario
            .envelope
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.radius)),
        "Admissibility envelope radius.",
    );
    push_scalar_series(
        &mut table,
        "norm_vs_envelope",
        plot_title,
        "time",
        "norm / envelope radius",
        "residual_norm",
        "residual norm",
        "line",
        "red",
        &scenario.record.id,
        scenario
            .residual
            .samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (index, sample.time, sample.norm)),
        "Residual norm trajectory.",
    );
    if let Some(exit_time) = scenario.detectability.observed_crossing_time {
        let y_max = scenario
            .envelope
            .samples
            .iter()
            .map(|sample| sample.radius)
            .chain(scenario.residual.samples.iter().map(|sample| sample.norm))
            .fold(0.0, f64::max)
            * 1.10;
        let point_order = table.rows.len();
        push_segment_row(
            &mut table,
            "norm_vs_envelope",
            plot_title,
            "time",
            "norm / envelope radius",
            "exit_marker",
            "exit marker",
            "black",
            point_order,
            exit_time,
            0.0,
            exit_time,
            y_max,
            "",
            "Observed exit-time marker line.",
        );
    }
    Ok(table)
}

fn new_source_table(
    generation_timestamp: &str,
    bank_version: &str,
    figure_id: &str,
    plot_title: &str,
    expected_panel_ids: &[&str],
    count_like_panel_ids: &[&str],
) -> FigureSourceTable {
    FigureSourceTable {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        bank_version: bank_version.to_string(),
        figure_id: figure_id.to_string(),
        plot_title: plot_title.to_string(),
        generation_timestamp: generation_timestamp.to_string(),
        expected_panel_count: expected_panel_ids.len(),
        expected_panel_ids: expected_panel_ids
            .iter()
            .map(|panel_id| (*panel_id).to_string())
            .collect(),
        count_like_panel_ids: count_like_panel_ids
            .iter()
            .map(|panel_id| (*panel_id).to_string())
            .collect(),
        panel_ids: Vec::new(),
        series_ids: Vec::new(),
        rows: Vec::new(),
    }
}

fn finalize_source_table(table: &mut FigureSourceTable, generation_timestamp: &str) {
    table.generation_timestamp = generation_timestamp.to_string();
    table.panel_ids = ordered_unique(table.rows.iter().map(|row| row.panel_id.clone()));
    table.series_ids = ordered_unique(table.rows.iter().map(|row| row.series_id.clone()));
}

fn ordered_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut ordered = Vec::new();
    for value in values {
        if !ordered.contains(&value) {
            ordered.push(value);
        }
    }
    ordered
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_vector_series(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    series_kind: &str,
    color_key: &str,
    scenario_id: &str,
    samples: &[VectorSample],
    channel_index: usize,
    note: &str,
) {
    for (order, sample) in samples.iter().enumerate() {
        table.rows.push(FigureSourceRow {
            schema_version: table.schema_version.clone(),
            engine_version: table.engine_version.clone(),
            bank_version: table.bank_version.clone(),
            figure_id: table.figure_id.clone(),
            plot_title: table.plot_title.clone(),
            panel_id: panel_id.to_string(),
            panel_title: panel_title.to_string(),
            x_label: x_label.to_string(),
            y_label: y_label.to_string(),
            series_id: series_id.to_string(),
            series_label: series_label.to_string(),
            series_kind: series_kind.to_string(),
            color_key: color_key.to_string(),
            point_order: order,
            x_value: sample.time,
            y_value: sample
                .values
                .get(channel_index)
                .copied()
                .unwrap_or_default(),
            secondary_x_value: None,
            secondary_y_value: None,
            x_tick_label: String::new(),
            annotation_text: String::new(),
            scenario_id: scenario_id.to_string(),
            note: note.to_string(),
        });
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_scalar_series(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    series_kind: &str,
    color_key: &str,
    scenario_id: &str,
    points: impl IntoIterator<Item = (usize, f64, f64)>,
    note: &str,
) {
    for (order, x_value, y_value) in points {
        table.rows.push(FigureSourceRow {
            schema_version: table.schema_version.clone(),
            engine_version: table.engine_version.clone(),
            bank_version: table.bank_version.clone(),
            figure_id: table.figure_id.clone(),
            plot_title: table.plot_title.clone(),
            panel_id: panel_id.to_string(),
            panel_title: panel_title.to_string(),
            x_label: x_label.to_string(),
            y_label: y_label.to_string(),
            series_id: series_id.to_string(),
            series_label: series_label.to_string(),
            series_kind: series_kind.to_string(),
            color_key: color_key.to_string(),
            point_order: order,
            x_value,
            y_value,
            secondary_x_value: None,
            secondary_y_value: None,
            x_tick_label: String::new(),
            annotation_text: String::new(),
            scenario_id: scenario_id.to_string(),
            note: note.to_string(),
        });
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_scalar_pair_series(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    series_kind: &str,
    color_key: &str,
    scenario_id: &str,
    points: impl IntoIterator<Item = (usize, f64, f64)>,
    note: &str,
) {
    push_scalar_series(
        table,
        panel_id,
        panel_title,
        x_label,
        y_label,
        series_id,
        series_label,
        series_kind,
        color_key,
        scenario_id,
        points,
        note,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_bar_row(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    color_key: &str,
    scenario_id: &str,
    point_order: usize,
    x_value: f64,
    secondary_x_value: f64,
    y_value: f64,
    x_tick_label: &str,
    note: &str,
) {
    table.rows.push(FigureSourceRow {
        schema_version: table.schema_version.clone(),
        engine_version: table.engine_version.clone(),
        bank_version: table.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        plot_title: table.plot_title.clone(),
        panel_id: panel_id.to_string(),
        panel_title: panel_title.to_string(),
        x_label: x_label.to_string(),
        y_label: y_label.to_string(),
        series_id: series_id.to_string(),
        series_label: series_label.to_string(),
        series_kind: "bar".to_string(),
        color_key: color_key.to_string(),
        point_order,
        x_value,
        y_value,
        secondary_x_value: Some(secondary_x_value),
        secondary_y_value: None,
        x_tick_label: x_tick_label.to_string(),
        annotation_text: String::new(),
        scenario_id: scenario_id.to_string(),
        note: note.to_string(),
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_annotation_point(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    series_kind: &str,
    color_key: &str,
    scenario_id: &str,
    point_order: usize,
    x_value: f64,
    y_value: f64,
    x_tick_label: &str,
    annotation_text: &str,
    note: &str,
) {
    table.rows.push(FigureSourceRow {
        schema_version: table.schema_version.clone(),
        engine_version: table.engine_version.clone(),
        bank_version: table.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        plot_title: table.plot_title.clone(),
        panel_id: panel_id.to_string(),
        panel_title: panel_title.to_string(),
        x_label: x_label.to_string(),
        y_label: y_label.to_string(),
        series_id: series_id.to_string(),
        series_label: series_label.to_string(),
        series_kind: series_kind.to_string(),
        color_key: color_key.to_string(),
        point_order,
        x_value,
        y_value,
        secondary_x_value: None,
        secondary_y_value: None,
        x_tick_label: x_tick_label.to_string(),
        annotation_text: annotation_text.to_string(),
        scenario_id: scenario_id.to_string(),
        note: note.to_string(),
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_segment_row(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    color_key: &str,
    point_order: usize,
    x_value: f64,
    y_value: f64,
    secondary_x_value: f64,
    secondary_y_value: f64,
    scenario_id: &str,
    note: &str,
) {
    table.rows.push(FigureSourceRow {
        schema_version: table.schema_version.clone(),
        engine_version: table.engine_version.clone(),
        bank_version: table.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        plot_title: table.plot_title.clone(),
        panel_id: panel_id.to_string(),
        panel_title: panel_title.to_string(),
        x_label: x_label.to_string(),
        y_label: y_label.to_string(),
        series_id: series_id.to_string(),
        series_label: series_label.to_string(),
        series_kind: "segment".to_string(),
        color_key: color_key.to_string(),
        point_order,
        x_value,
        y_value,
        secondary_x_value: Some(secondary_x_value),
        secondary_y_value: Some(secondary_y_value),
        x_tick_label: String::new(),
        annotation_text: String::new(),
        scenario_id: scenario_id.to_string(),
        note: note.to_string(),
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-source rows are emitted field-by-field to keep the exported schema explicit and auditable."
)]
fn push_box_row(
    table: &mut FigureSourceTable,
    panel_id: &str,
    panel_title: &str,
    x_label: &str,
    y_label: &str,
    series_id: &str,
    series_label: &str,
    color_key: &str,
    point_order: usize,
    x_value: f64,
    y_value: f64,
    secondary_x_value: f64,
    secondary_y_value: f64,
    x_tick_label: &str,
    annotation_text: &str,
    note: &str,
) {
    table.rows.push(FigureSourceRow {
        schema_version: table.schema_version.clone(),
        engine_version: table.engine_version.clone(),
        bank_version: table.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        plot_title: table.plot_title.clone(),
        panel_id: panel_id.to_string(),
        panel_title: panel_title.to_string(),
        x_label: x_label.to_string(),
        y_label: y_label.to_string(),
        series_id: series_id.to_string(),
        series_label: series_label.to_string(),
        series_kind: "box".to_string(),
        color_key: color_key.to_string(),
        point_order,
        x_value,
        y_value,
        secondary_x_value: Some(secondary_x_value),
        secondary_y_value: Some(secondary_y_value),
        x_tick_label: x_tick_label.to_string(),
        annotation_text: annotation_text.to_string(),
        scenario_id: String::new(),
        note: note.to_string(),
    });
}

fn source_scenario_or_first<'a>(
    bundle: &'a EngineOutputBundle,
    id: &str,
) -> Result<&'a ScenarioOutput> {
    bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == id)
        .or_else(|| bundle.scenario_outputs.first())
        .with_context(|| format!("missing scenario for figure source `{id}`"))
}

fn source_scenario_pair_or_first<'a>(
    bundle: &'a EngineOutputBundle,
    first_id: &str,
    second_id: &str,
) -> Result<(&'a ScenarioOutput, &'a ScenarioOutput)> {
    let first = source_scenario_or_first(bundle, first_id)?;
    let second = bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == second_id)
        .or_else(|| {
            bundle
                .scenario_outputs
                .iter()
                .find(|scenario| scenario.record.id != first.record.id)
        })
        .unwrap_or(first);
    Ok((first, second))
}

fn scalar_bounds(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !min.is_finite() || !max.is_finite() || (max - min).abs() < 1.0e-12 {
        (min.min(0.0) - 0.1, max.max(0.0) + 0.1)
    } else {
        (min, max)
    }
}

fn representative_semantic_scenarios(
    bundle: &EngineOutputBundle,
) -> Vec<(String, &ScenarioOutput)> {
    let mut selected = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for preferred in ["outward_exit_case_a", "regime_switch", "nominal_stable"] {
        if let Some(scenario) = bundle
            .scenario_outputs
            .iter()
            .find(|scenario| scenario.record.id == preferred)
        {
            if seen.insert(scenario.record.id.clone()) {
                selected.push(("preferred-id".to_string(), scenario));
            }
        }
    }

    for scenario in &bundle.scenario_outputs {
        if selected.len() >= 3 {
            break;
        }
        if seen.insert(scenario.record.id.clone()) {
            selected.push(("fallback-order".to_string(), scenario));
        }
    }

    selected
}

fn compact_scenario_tick_label(scenario_id: &str, representative_rank: usize) -> String {
    let cleaned = scenario_id
        .strip_suffix("_public_demo")
        .unwrap_or(scenario_id)
        .replace('_', " ");
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        format!("run {representative_rank}")
    } else if trimmed.chars().count() > 18 {
        let shortened = trimmed.chars().take(15).collect::<String>();
        format!("{shortened}...")
    } else {
        trimmed.to_string()
    }
}

fn compact_candidate_tick_label(short_label: &str, candidate_rank: usize) -> String {
    let cleaned = short_label.trim();
    if cleaned.is_empty() {
        format!("cand {candidate_rank}")
    } else if cleaned.chars().count() > 12 {
        let shortened = cleaned.chars().take(10).collect::<String>();
        format!("{shortened}..")
    } else {
        cleaned.to_string()
    }
}

fn representative_candidate_score_bars(
    bundle: &EngineOutputBundle,
    scenario_id: &str,
) -> Vec<(String, f64)> {
    bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == scenario_id)
        .map(|scenario| {
            scenario
                .semantics
                .retrieval_audit
                .ranked_candidates_post_regime
                .iter()
                .take(4)
                .enumerate()
                .map(|(index, candidate)| {
                    (
                        compact_candidate_tick_label(&candidate.short_label, index + 1),
                        candidate.score,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn short_comparator_tick_label(comparator_id: &str) -> &'static str {
    match comparator_id {
        "baseline_residual_threshold" => "threshold",
        "baseline_moving_average_trend" => "MA trend",
        "baseline_cusum" => "CUSUM",
        "baseline_slew_spike" => "slew",
        "baseline_envelope_interaction" => "envelope",
        "baseline_innovation_chi_squared_style" => "innovation",
        _ => "comparator",
    }
}

fn detectability_cases(bundle: &EngineOutputBundle) -> Vec<&ScenarioOutput> {
    let preferred = [
        "outward_exit_case_a",
        "outward_exit_case_b",
        "outward_exit_case_c",
        "magnitude_matched_detectable",
    ];
    let selected = preferred
        .into_iter()
        .filter_map(|id| {
            bundle
                .scenario_outputs
                .iter()
                .find(|scenario| scenario.record.id == id)
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        bundle
            .scenario_outputs
            .iter()
            .filter(|scenario| scenario.detectability.predicted_upper_bound.is_some())
            .collect()
    } else {
        selected
    }
}

fn detectability_summary_scenario(bundle: &EngineOutputBundle) -> Option<&ScenarioOutput> {
    bundle
        .scenario_outputs
        .iter()
        .find(|scenario| {
            first_non_admissible_time(scenario).is_some()
                || first_violation_time(scenario).is_some()
                || scenario.detectability.observed_crossing_time.is_some()
        })
        .or_else(|| bundle.scenario_outputs.first())
}

fn is_primary_milling_bundle(bundle: &EngineOutputBundle) -> bool {
    bundle
        .scenario_outputs
        .iter()
        .any(|scenario| scenario.record.id == "nasa_milling_public_demo")
}

fn is_primary_bearings_bundle(bundle: &EngineOutputBundle) -> bool {
    bundle
        .scenario_outputs
        .iter()
        .any(|scenario| scenario.record.id == "nasa_bearings_public_demo")
}

fn is_synthetic_bundle(bundle: &EngineOutputBundle) -> bool {
    bundle.run_metadata.input_mode == "synthetic"
        && !is_primary_milling_bundle(bundle)
        && !is_primary_bearings_bundle(bundle)
}

fn milling_primary_scenario(bundle: &EngineOutputBundle) -> Result<&ScenarioOutput> {
    source_scenario_or_first(bundle, "nasa_milling_public_demo")
}

fn bearings_primary_scenario(bundle: &EngineOutputBundle) -> Result<&ScenarioOutput> {
    source_scenario_or_first(bundle, "nasa_bearings_public_demo")
}

fn synthetic_transition_scenario(bundle: &EngineOutputBundle) -> Result<&ScenarioOutput> {
    let mut best: Option<(usize, &ScenarioOutput)> = None;
    for scenario in &bundle.scenario_outputs {
        let timeline = match build_prefix_semantic_timeline(bundle, scenario) {
            Ok(timeline) => timeline,
            Err(_) => continue,
        };
        let semantic_transitions =
            count_transitions(timeline.iter().map(|point| point.semantic_disposition_code));
        let candidate_transitions = count_transitions(timeline.iter().map(|point| {
            (
                point.post_regime_candidate_count,
                point.post_scope_candidate_count,
            )
        }));
        let grammar_transitions = count_transitions(
            scenario
                .grammar
                .iter()
                .map(|status| grammar_state_code(status.state)),
        );
        let triggered_comparators = bundle
            .evaluation
            .baseline_results
            .iter()
            .filter(|result| result.scenario_id == scenario.record.id && result.triggered)
            .count();
        let preference_bonus = match scenario.record.id.as_str() {
            "regime_switch" => 20,
            "abrupt_event" => 12,
            "curvature_onset" => 8,
            _ => 0,
        };
        let score = semantic_transitions * 20
            + candidate_transitions * 12
            + grammar_transitions * 8
            + triggered_comparators * 4
            + preference_bonus;
        match best {
            Some((best_score, _)) if score <= best_score => {}
            _ => best = Some((score, scenario)),
        }
    }
    best.map(|(_, scenario)| scenario)
        .or_else(|| {
            bundle
                .scenario_outputs
                .iter()
                .find(|scenario| scenario.record.id == "regime_switch")
        })
        .or_else(|| bundle.scenario_outputs.first())
        .with_context(|| "missing synthetic scenario for upgraded figure generation")
}

fn select_synthetic_structural_pair(
    bundle: &EngineOutputBundle,
) -> Option<SyntheticScenarioPairSelection<'_>> {
    let mut best: Option<(f64, SyntheticScenarioPairSelection<'_>)> = None;
    for (left_index, left) in bundle.scenario_outputs.iter().enumerate() {
        let left_primary_mean = mean_norm(left.residual.samples.iter().map(|sample| sample.norm));
        let left_meta_mean = mean_norm(left.slew.samples.iter().map(|sample| sample.norm));
        let left_outcome_mean = mean_grammar_state_code(&left.grammar)?;
        for right in bundle.scenario_outputs.iter().skip(left_index + 1) {
            let right_primary_mean =
                mean_norm(right.residual.samples.iter().map(|sample| sample.norm));
            let right_meta_mean = mean_norm(right.slew.samples.iter().map(|sample| sample.norm));
            let right_outcome_mean = mean_grammar_state_code(&right.grammar)?;
            let primary_gap = (left_primary_mean - right_primary_mean).abs();
            let meta_gap = (left_meta_mean - right_meta_mean).abs();
            let outcome_gap = (left_outcome_mean - right_outcome_mean).abs();
            if meta_gap <= 1.0e-3 || outcome_gap <= 0.0 {
                continue;
            }
            let (
                admissible_case,
                detectable_case,
                admissible_primary_mean,
                detectable_primary_mean,
                admissible_meta_mean,
                detectable_meta_mean,
                admissible_outcome_mean,
                detectable_outcome_mean,
            ) = if left_outcome_mean <= right_outcome_mean {
                (
                    left,
                    right,
                    left_primary_mean,
                    right_primary_mean,
                    left_meta_mean,
                    right_meta_mean,
                    left_outcome_mean,
                    right_outcome_mean,
                )
            } else {
                (
                    right,
                    left,
                    right_primary_mean,
                    left_primary_mean,
                    right_meta_mean,
                    left_meta_mean,
                    right_outcome_mean,
                    left_outcome_mean,
                )
            };
            let preference_bonus = match (
                admissible_case.record.id.as_str(),
                detectable_case.record.id.as_str(),
            ) {
                ("abrupt_event", "imu_thermal_drift_gps_denied") => 0.02,
                ("nominal_stable", "imu_thermal_drift_gps_denied") => 0.01,
                _ => 0.0,
            };
            let score = primary_gap / (1.0e-9 + meta_gap + outcome_gap) - preference_bonus;
            let selection = SyntheticScenarioPairSelection {
                admissible_case,
                detectable_case,
                admissible_primary_mean,
                detectable_primary_mean,
                admissible_meta_mean,
                detectable_meta_mean,
                admissible_outcome_mean,
                detectable_outcome_mean,
            };
            match &best {
                Some((best_score, _)) if score >= *best_score => {}
                _ => best = Some((score, selection)),
            }
        }
    }
    best.map(|(_, selection)| selection).or_else(|| {
        source_scenario_pair_or_first(bundle, "abrupt_event", "imu_thermal_drift_gps_denied")
            .ok()
            .map(
                |(admissible_case, detectable_case)| SyntheticScenarioPairSelection {
                    admissible_primary_mean: mean_norm(
                        admissible_case
                            .residual
                            .samples
                            .iter()
                            .map(|sample| sample.norm),
                    ),
                    detectable_primary_mean: mean_norm(
                        detectable_case
                            .residual
                            .samples
                            .iter()
                            .map(|sample| sample.norm),
                    ),
                    admissible_meta_mean: mean_norm(
                        admissible_case
                            .slew
                            .samples
                            .iter()
                            .map(|sample| sample.norm),
                    ),
                    detectable_meta_mean: mean_norm(
                        detectable_case
                            .slew
                            .samples
                            .iter()
                            .map(|sample| sample.norm),
                    ),
                    admissible_outcome_mean: mean_grammar_state_code(&admissible_case.grammar)
                        .unwrap_or_default(),
                    detectable_outcome_mean: mean_grammar_state_code(&detectable_case.grammar)
                        .unwrap_or_default(),
                    admissible_case,
                    detectable_case,
                },
            )
    })
}

fn first_non_admissible_time(scenario: &ScenarioOutput) -> Option<f64> {
    scenario
        .grammar
        .iter()
        .find(|status| !matches!(status.state, GrammarState::Admissible))
        .map(|status| status.time)
        .or(scenario.detectability.observed_crossing_time)
}

fn first_violation_time(scenario: &ScenarioOutput) -> Option<f64> {
    scenario
        .grammar
        .iter()
        .find(|status| matches!(status.state, GrammarState::Violation))
        .map(|status| status.time)
}

fn grammar_state_code(state: GrammarState) -> i32 {
    match state {
        GrammarState::Admissible => 0,
        GrammarState::Boundary => 1,
        GrammarState::Violation => 2,
    }
}

fn count_transitions<T>(values: impl IntoIterator<Item = T>) -> usize
where
    T: PartialEq,
{
    let mut iter = values.into_iter();
    let Some(mut previous) = iter.next() else {
        return 0;
    };
    let mut transitions = 0;
    for value in iter {
        if value != previous {
            transitions += 1;
            previous = value;
        }
    }
    transitions
}

fn select_matched_windows(scenario: &ScenarioOutput) -> Option<MatchedWindowSelection> {
    let sample_count = scenario
        .residual
        .samples
        .len()
        .min(scenario.slew.samples.len())
        .min(scenario.grammar.len());
    if sample_count < 8 {
        return None;
    }
    let first_non_admissible = scenario
        .grammar
        .iter()
        .position(|status| status.state != GrammarState::Admissible)?;
    let first_violation = scenario
        .grammar
        .iter()
        .position(|status| status.state == GrammarState::Violation)
        .unwrap_or(first_non_admissible);
    let window_len = 4usize.min(sample_count / 2).max(3);
    let departure_start = first_violation
        .saturating_sub(window_len / 2)
        .min(sample_count.saturating_sub(window_len));
    let departure_window = departure_start..departure_start + window_len;
    if first_non_admissible < window_len {
        return None;
    }
    let stable_last_start = first_non_admissible.saturating_sub(window_len);
    let channel_count = scenario_channel_count(scenario).min(3);
    let mut best: Option<(f64, MatchedWindowSelection)> = None;
    for channel_index in 0..channel_count {
        let departure_primary_mean = mean_abs_channel_residual(
            &scenario.residual.samples[departure_window.clone()],
            channel_index,
        )?;
        let departure_meta_mean = mean_abs_channel_slew(
            &scenario.slew.samples[departure_window.clone()],
            channel_index,
        )?;
        let departure_outcome_mean =
            mean_grammar_state_code(&scenario.grammar[departure_window.clone()])?;
        for stable_start in 0..=stable_last_start {
            let stable_window = stable_start..stable_start + window_len;
            let stable_primary_mean = mean_abs_channel_residual(
                &scenario.residual.samples[stable_window.clone()],
                channel_index,
            )?;
            let stable_meta_mean = mean_abs_channel_slew(
                &scenario.slew.samples[stable_window.clone()],
                channel_index,
            )?;
            let stable_outcome_mean =
                mean_grammar_state_code(&scenario.grammar[stable_window.clone()])?;
            let primary_gap = (stable_primary_mean - departure_primary_mean).abs();
            let meta_gap = (stable_meta_mean - departure_meta_mean).abs();
            let outcome_gap = (stable_outcome_mean - departure_outcome_mean).abs();
            if meta_gap <= 0.0 || outcome_gap <= 0.0 {
                continue;
            }
            let score = primary_gap / (1.0e-9 + meta_gap + outcome_gap);
            let selection = MatchedWindowSelection {
                channel_index,
                stable_window: stable_window.clone(),
                departure_window: departure_window.clone(),
                stable_primary_mean,
                departure_primary_mean,
                stable_meta_mean,
                departure_meta_mean,
                stable_outcome_mean,
                departure_outcome_mean,
            };
            match &best {
                Some((best_score, _)) if score >= *best_score => {}
                _ => best = Some((score, selection)),
            }
        }
    }
    best.map(|(_, selection)| selection)
}

fn select_process_windows(scenario: &ScenarioOutput) -> Option<MatchedWindowSelection> {
    let sample_count = scenario
        .residual
        .samples
        .len()
        .min(scenario.slew.samples.len())
        .min(scenario.grammar.len());
    if sample_count < 8 {
        return None;
    }
    let max_window_len = 6usize.min(sample_count / 2).max(3);
    let channel_count = scenario_channel_count(scenario).min(3);
    let mut best: Option<(f64, MatchedWindowSelection)> = None;
    for window_len in 3..=max_window_len {
        for channel_index in 0..channel_count {
            for left_start in 0..=sample_count.saturating_sub(window_len) {
                let left_window = left_start..left_start + window_len;
                let left_primary_mean = mean_abs_channel_residual(
                    &scenario.residual.samples[left_window.clone()],
                    channel_index,
                )?;
                let left_meta_mean = mean_abs_channel_slew(
                    &scenario.slew.samples[left_window.clone()],
                    channel_index,
                )?;
                let left_outcome_mean =
                    mean_grammar_state_code(&scenario.grammar[left_window.clone()])?;
                for right_start in
                    (left_start + window_len)..=sample_count.saturating_sub(window_len)
                {
                    let right_window = right_start..right_start + window_len;
                    let right_primary_mean = mean_abs_channel_residual(
                        &scenario.residual.samples[right_window.clone()],
                        channel_index,
                    )?;
                    let right_meta_mean = mean_abs_channel_slew(
                        &scenario.slew.samples[right_window.clone()],
                        channel_index,
                    )?;
                    let right_outcome_mean =
                        mean_grammar_state_code(&scenario.grammar[right_window.clone()])?;
                    let primary_gap = (left_primary_mean - right_primary_mean).abs();
                    let meta_gap = (left_meta_mean - right_meta_mean).abs();
                    let outcome_gap = (left_outcome_mean - right_outcome_mean).abs();
                    if meta_gap <= 1.0e-3 || outcome_gap <= 0.0 {
                        continue;
                    }
                    let (
                        stable_window,
                        departure_window,
                        stable_primary_mean,
                        departure_primary_mean,
                        stable_meta_mean,
                        departure_meta_mean,
                        stable_outcome_mean,
                        departure_outcome_mean,
                    ) = if left_outcome_mean <= right_outcome_mean {
                        (
                            left_window.clone(),
                            right_window.clone(),
                            left_primary_mean,
                            right_primary_mean,
                            left_meta_mean,
                            right_meta_mean,
                            left_outcome_mean,
                            right_outcome_mean,
                        )
                    } else {
                        (
                            right_window.clone(),
                            left_window.clone(),
                            right_primary_mean,
                            left_primary_mean,
                            right_meta_mean,
                            left_meta_mean,
                            right_outcome_mean,
                            left_outcome_mean,
                        )
                    };
                    let temporal_separation = (departure_window.start as isize
                        - stable_window.end as isize)
                        .unsigned_abs() as f64;
                    let score = primary_gap / (1.0e-9 + meta_gap + outcome_gap)
                        - temporal_separation * 1.0e-6;
                    let selection = MatchedWindowSelection {
                        channel_index,
                        stable_window,
                        departure_window,
                        stable_primary_mean,
                        departure_primary_mean,
                        stable_meta_mean,
                        departure_meta_mean,
                        stable_outcome_mean,
                        departure_outcome_mean,
                    };
                    match &best {
                        Some((best_score, _)) if score >= *best_score => {}
                        _ => best = Some((score, selection)),
                    }
                }
            }
        }
    }
    best.map(|(_, selection)| selection)
}

fn mean_abs_channel_residual(
    samples: &[crate::engine::types::ResidualSample],
    channel_index: usize,
) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    Some(
        samples
            .iter()
            .map(|sample| {
                sample
                    .values
                    .get(channel_index)
                    .copied()
                    .unwrap_or_default()
                    .abs()
            })
            .sum::<f64>()
            / samples.len() as f64,
    )
}

fn mean_norm(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut count = 0usize;
    let mut total = 0.0;
    for value in values {
        count += 1;
        total += value;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

fn mean_abs_channel_slew(
    samples: &[crate::engine::types::SlewSample],
    channel_index: usize,
) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    Some(
        samples
            .iter()
            .map(|sample| {
                sample
                    .values
                    .get(channel_index)
                    .copied()
                    .unwrap_or_default()
                    .abs()
            })
            .sum::<f64>()
            / samples.len() as f64,
    )
}

fn mean_grammar_state_code(samples: &[crate::engine::types::GrammarStatus]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    Some(
        samples
            .iter()
            .map(|status| grammar_state_code(status.state) as f64)
            .sum::<f64>()
            / samples.len() as f64,
    )
}

fn min_margin_sample(scenario: &ScenarioOutput) -> Option<&crate::engine::types::GrammarStatus> {
    scenario.grammar.iter().min_by(|left, right| {
        left.margin
            .partial_cmp(&right.margin)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn detectability_window_max_ratio(
    scenario: &ScenarioOutput,
    window_count: usize,
) -> Vec<(f64, String)> {
    let sample_count = scenario
        .residual
        .samples
        .len()
        .min(scenario.envelope.samples.len());
    if sample_count == 0 {
        return vec![(0.0, "w1".to_string())];
    }
    let windows = window_count.max(1).min(sample_count);
    let window_len = sample_count.div_ceil(windows);
    (0..windows)
        .filter_map(|window_index| {
            let start = window_index * window_len;
            if start >= sample_count {
                return None;
            }
            let end = ((window_index + 1) * window_len).min(sample_count);
            let max_ratio = scenario.residual.samples[start..end]
                .iter()
                .zip(&scenario.envelope.samples[start..end])
                .map(|(residual, envelope)| {
                    let radius = envelope.radius.abs().max(1.0e-9);
                    residual.norm / radius
                })
                .fold(0.0_f64, f64::max);
            Some((max_ratio, format!("w{}", window_index + 1)))
        })
        .collect()
}

fn scenario_channel_count(scenario: &ScenarioOutput) -> usize {
    scenario.residual.channel_names.len().max(
        scenario
            .residual
            .samples
            .iter()
            .map(|sample| sample.values.len())
            .max()
            .unwrap_or(0),
    )
}

fn scenario_channel_label(scenario: &ScenarioOutput, channel_index: usize) -> String {
    scenario
        .residual
        .channel_names
        .get(channel_index)
        .cloned()
        .unwrap_or_else(|| format!("ch{}", channel_index + 1))
}

fn mean_abs_values(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().map(|value| value.abs()).sum::<f64>() / values.len() as f64)
    }
}

/// Discrete code exported for figure-friendly semantic disposition plots.
pub fn semantic_disposition_code(disposition: &SemanticDisposition) -> i32 {
    match disposition {
        SemanticDisposition::Unknown => 0,
        SemanticDisposition::Ambiguous => 1,
        SemanticDisposition::CompatibleSet => 2,
        SemanticDisposition::Match => 3,
    }
}

/// Discrete code exported for figure-friendly semantic disposition plots from labels.
pub fn disposition_label_code(label: &str) -> i32 {
    match label {
        "Match" => 3,
        "CompatibleSet" => 2,
        "Ambiguous" => 1,
        _ => 0,
    }
}
