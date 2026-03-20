use serde::{Deserialize, Serialize};

use crate::engine::types::{EngineOutputBundle, ScenarioOutput, SemanticDisposition};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

/// Machine-readable source row for the detectability comparison summary figure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectabilityFigureSourceRow {
    pub schema_version: String,
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

/// Returns the detectability figure source rows in the same scenario order used for plotting.
pub fn detectability_source_rows(bundle: &EngineOutputBundle) -> Vec<DetectabilityFigureSourceRow> {
    detectability_cases(bundle)
        .into_iter()
        .map(|scenario| DetectabilityFigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
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
        ("baseline_slew_spike", "Slew spike"),
        ("baseline_envelope_interaction", "Envelope interaction"),
    ]
    .into_iter()
    .map(|(id, label)| BaselineComparatorFigureSourceRow {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
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
