use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::engine::types::{EngineOutputBundle, ScenarioOutput, SemanticDisposition, VectorSample};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

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
    pub expected_panel_count: usize,
    pub count_like_panel_ids: Vec<String>,
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
    pub heuristic_bank_entry_count: usize,
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
            bound_satisfied: scenario.detectability.bound_satisfied,
            note: "Source row for the predicted-versus-observed detectability comparison figure."
                .to_string(),
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
                .candidates
                .first()
                .map(|candidate| candidate.score)
                .unwrap_or(0.0),
            heuristic_bank_entry_count: scenario
                .semantics
                .retrieval_audit
                .heuristic_bank_entry_count,
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
            note: "Panel 1 uses `leading_candidate_score`, panel 2 uses `heuristic_candidates_post_admissibility`, and panel 3 uses `disposition_code`."
                .to_string(),
        })
        .collect()
}

/// Returns the comparator-trigger source rows used in the deterministic comparator summary.
pub fn baseline_comparator_source_rows(
    bundle: &EngineOutputBundle,
) -> Vec<BaselineComparatorFigureSourceRow> {
    [
        ("baseline_residual_threshold", "Residual threshold"),
        ("baseline_moving_average_trend", "Moving-average trend"),
        ("baseline_cusum", "CUSUM"),
        ("baseline_slew_spike", "Slew spike"),
        ("baseline_envelope_interaction", "Envelope interaction"),
        (
            "baseline_innovation_chi_squared_style",
            "Innovation-style squared residual",
        ),
    ]
    .into_iter()
    .map(|(id, label)| BaselineComparatorFigureSourceRow {
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
        note: "Source row for the internal deterministic comparator trigger-count figure."
            .to_string(),
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
        &bundle.run_metadata.bank.bank_version,
        "figure_01_residual_prediction_observation_overview",
        "Residual, Observation, and Prediction Overview",
        2,
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
        &bundle.run_metadata.bank.bank_version,
        "figure_02_drift_and_slew_decomposition",
        "Drift and Slew Decomposition",
        3,
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
        &bundle.run_metadata.bank.bank_version,
        "figure_03_sign_space_projection",
        "Projected Sign Trajectory",
        1,
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
        &bundle.run_metadata.bank.bank_version,
        "figure_04_syntax_comparison",
        "Syntax Comparison",
        1,
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
        &bundle.run_metadata.bank.bank_version,
        "figure_07_exit_invariance_pair_common_envelope",
        "Exit-Invariance Pair on Shared Envelope",
        1,
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
        &bundle.run_metadata.bank.bank_version,
        "figure_08_residual_trajectory_separation",
        "Residual Trajectory Separation",
        1,
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

fn prepare_figure_09(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_09_detectability_bound_comparison",
        "Predicted vs Observed Detectability Times",
        1,
        &[],
    );
    let rows = detectability_source_rows(bundle);
    if rows.is_empty() {
        push_annotation_point(
            &mut table,
            "detectability_bound",
            "Predicted vs Observed Detectability Times",
            "scenario index",
            "time to first exit",
            "detectability_notice",
            "detectability notice",
            "annotation",
            "slate",
            "",
            0,
            0.5,
            0.5,
            "",
            "No detectability-bound comparison rows were available for this run selection.",
            "Fallback annotation rendered when no scenarios in the selected bundle expose a detectability-bound comparison.",
        );
    }
    for (index, row) in rows.into_iter().enumerate() {
        if let Some(predicted) = row.predicted_upper_bound {
            push_bar_row(
                &mut table,
                "detectability_bound",
                "Predicted vs Observed Detectability Times",
                "scenario index",
                "time to first exit",
                "predicted_upper_bound",
                "predicted upper bound",
                "blue",
                &row.scenario_id,
                index * 2,
                index as f64 + 0.25,
                index as f64 + 0.43,
                predicted,
                &row.scenario_id,
                "Predicted bound bar rendered for this scenario.",
            );
        }
        if let Some(observed) = row.observed_crossing_time {
            push_bar_row(
                &mut table,
                "detectability_bound",
                "Predicted vs Observed Detectability Times",
                "scenario index",
                "time to first exit",
                "observed_crossing_time",
                "observed crossing time",
                "red",
                &row.scenario_id,
                index * 2 + 1,
                index as f64 + 0.47,
                index as f64 + 0.65,
                observed,
                &row.scenario_id,
                "Observed crossing-time bar rendered for this scenario.",
            );
        }
    }
    table
}

fn prepare_figure_10(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_10_deterministic_pipeline_flow",
        "Deterministic Structural Semiotics Engine",
        1,
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
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_11_coordinated_group_semiotics",
        "Coordinated Group Semiotics",
        if scenario.coordinated.is_some() { 2 } else { 1 },
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

fn prepare_figure_12(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_12_semantic_retrieval_heuristics_bank",
        "Representative Constrained Retrieval Summary",
        3,
        &["admissibility_filter_count"],
    );
    for (index, row) in semantic_retrieval_source_rows(bundle)
        .into_iter()
        .enumerate()
    {
        let label = row.scenario_id.clone();
        push_bar_row(
            &mut table,
            "leading_candidate_score",
            "Leading Candidate Score",
            "representative scenario",
            "candidate score",
            "leading_candidate_score",
            "leading candidate score",
            ["blue", "red", "slate"][index % 3],
            &row.scenario_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.64,
            row.leading_candidate_score,
            &label,
            "Leading candidate score bar.",
        );
        push_bar_row(
            &mut table,
            "admissibility_filter_count",
            "Candidates After Admissibility Filter",
            "representative scenario",
            "heuristic count",
            "post_admissibility_count",
            "post-admissibility count",
            "gold",
            &row.scenario_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.64,
            row.heuristic_candidates_post_admissibility as f64,
            &label,
            "Admissibility-qualified heuristic count bar.",
        );
        push_bar_row(
            &mut table,
            "retrieval_disposition_code",
            "Final Retrieval Disposition Code",
            "representative scenario",
            "disposition code (0..3)",
            "retrieval_disposition_code",
            "retrieval disposition code",
            ["blue", "red", "slate"][index % 3],
            &row.scenario_id,
            index,
            index as f64 + 0.18,
            index as f64 + 0.64,
            row.disposition_code as f64,
            &label,
            "Disposition-code bar (Unknown=0, Ambiguous=1, CompatibleSet=2, Match=3).",
        );
    }
    table
}

fn prepare_figure_13(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_13_internal_baseline_comparators",
        "Internal Deterministic Comparator Trigger Counts",
        1,
        &["comparator_trigger_counts"],
    );
    for (index, row) in baseline_comparator_source_rows(bundle)
        .into_iter()
        .enumerate()
    {
        push_bar_row(
            &mut table,
            "comparator_trigger_counts",
            "Internal Deterministic Comparator Trigger Counts",
            "comparator",
            "triggered scenarios",
            &row.comparator_id,
            &row.comparator_label,
            "blue",
            &row.comparator_id,
            index,
            index as f64,
            index as f64 + 1.0,
            row.triggered_scenario_count as f64,
            &row.comparator_label,
            "Comparator trigger-count bar.",
        );
    }
    table
}

fn prepare_figure_14(bundle: &EngineOutputBundle) -> FigureSourceTable {
    let mut table = new_source_table(
        &bundle.run_metadata.bank.bank_version,
        "figure_14_sweep_stability_summary",
        "Sweep Semantic Stability",
        1,
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
        &bundle.run_metadata.bank.bank_version,
        figure_id,
        plot_title,
        1,
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
    bank_version: &str,
    figure_id: &str,
    plot_title: &str,
    expected_panel_count: usize,
    count_like_panel_ids: &[&str],
) -> FigureSourceTable {
    FigureSourceTable {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        bank_version: bank_version.to_string(),
        figure_id: figure_id.to_string(),
        plot_title: plot_title.to_string(),
        expected_panel_count,
        count_like_panel_ids: count_like_panel_ids
            .iter()
            .map(|panel_id| (*panel_id).to_string())
            .collect(),
        rows: Vec::new(),
    }
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
