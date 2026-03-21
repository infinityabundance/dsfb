use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::bank::HeuristicBankValidationReport;

/// Result of one simple deterministic baseline comparator.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineComparatorResult {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub scenario_id: String,
    pub comparator_id: String,
    pub comparator_label: String,
    pub triggered: bool,
    pub first_trigger_step: Option<usize>,
    pub first_trigger_time: Option<f64>,
    pub comparator_summary: String,
    pub distinction_note: String,
}

/// Scenario-level evaluation summary derived from a completed engine run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioEvaluationSummary {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub scenario_id: String,
    pub input_mode: String,
    pub syntax_label: String,
    pub semantic_disposition: String,
    pub selected_heuristic_ids: Vec<String>,
    pub grammar_reason_code: String,
    pub grammar_reason_text: String,
    pub trust_scalar: f64,
    pub heuristic_bank_entry_count: usize,
    pub heuristic_candidates_post_admissibility: usize,
    pub heuristic_candidates_post_regime: usize,
    pub heuristic_candidates_pre_scope: usize,
    pub heuristic_candidates_post_scope: usize,
    pub heuristics_rejected_by_admissibility: usize,
    pub heuristics_rejected_by_regime: usize,
    pub heuristics_rejected_by_scope: usize,
    pub heuristics_selected_final: usize,
    pub boundary_sample_count: usize,
    pub violation_sample_count: usize,
    pub first_boundary_time: Option<f64>,
    pub first_violation_time: Option<f64>,
    pub reproducible: bool,
    pub triggered_baseline_count: usize,
    pub unknown_reason_class: Option<String>,
    pub note: String,
}

/// Run-level deterministic evaluation summary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunEvaluationSummary {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub evaluation_version: String,
    pub input_mode: String,
    pub scenario_count: usize,
    pub semantic_disposition_counts: BTreeMap<String, usize>,
    pub syntax_label_counts: BTreeMap<String, usize>,
    pub boundary_interaction_count: usize,
    pub violation_count: usize,
    pub comparator_trigger_counts: BTreeMap<String, usize>,
    pub reproducible_scenario_count: usize,
    pub all_reproducible: bool,
    pub minimum_trust_scalar: f64,
    pub note: String,
}

/// Deterministic smoothing-comparison record for one scenario.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmoothingComparisonRecord {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub scenario_id: String,
    pub smoothing_mode: String,
    pub smoothing_enabled: bool,
    pub smoothing_alpha: f64,
    pub raw_mean_squared_slew_norm: f64,
    pub active_mean_squared_slew_norm: f64,
    pub raw_max_slew_norm: f64,
    pub active_max_slew_norm: f64,
    pub raw_slew_spike_count: usize,
    pub active_slew_spike_count: usize,
    pub raw_syntax_label: String,
    pub active_syntax_label: String,
    pub note: String,
}

/// Deterministic retrieval-scaling record using candidate-count proxies rather than wall-clock timing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalLatencyRecord {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub bank_size: usize,
    pub retrieval_path: String,
    pub linear_candidates_considered: usize,
    pub indexed_prefilter_candidate_count: usize,
    pub indexed_post_scope_candidate_count: usize,
    pub index_buckets_considered: usize,
    pub note: String,
}

/// Exported artifact completeness check recorded after artifact generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactCompletenessCheck {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_count: usize,
    pub csv_count: usize,
    pub json_count: usize,
    pub report_markdown_present: bool,
    pub report_pdf_present: bool,
    pub zip_present: bool,
    pub manifest_present: bool,
    pub complete: bool,
    pub note: String,
}

/// Integrity check for a derived summary figure and its machine-readable source table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FigureIntegrityCheck {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub figure_id: String,
    pub expected_panel_count: usize,
    pub observed_panel_count: usize,
    pub expected_panels: Vec<String>,
    pub observed_panels: Vec<String>,
    pub panel_labels: Vec<String>,
    pub series_lengths: Vec<usize>,
    pub source_row_count: usize,
    pub source_table_present: bool,
    pub nonempty_series: bool,
    pub nonzero_values_present: bool,
    pub count_like_panels_integerlike: bool,
    pub consistent_with_source: bool,
    pub integrity_passed: bool,
    pub failures: Vec<String>,
    pub source_csv: String,
    pub source_json: String,
    pub png_path: String,
    pub svg_path: String,
    pub png_present: bool,
    pub svg_present: bool,
    pub note: String,
}

/// One deterministic sweep result for a generated sweep member.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepPointResult {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub sweep_family: String,
    pub scenario_id: String,
    pub parameter_name: String,
    pub parameter_value: f64,
    pub secondary_parameter_name: Option<String>,
    pub secondary_parameter_value: Option<f64>,
    pub syntax_label: String,
    pub semantic_disposition: String,
    pub selected_heuristic_ids: Vec<String>,
    pub note: String,
}

/// Run-level summary for a deterministic sweep family.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepRunSummary {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub sweep_family: String,
    pub member_count: usize,
    pub unique_syntax_labels: Vec<String>,
    pub unique_semantic_dispositions: Vec<String>,
    pub unknown_count: usize,
    pub ambiguous_count: usize,
    pub disposition_flip_count: usize,
    pub note: String,
}

/// Evaluation bundle kept separate from the core engine outputs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunEvaluationBundle {
    pub summary: RunEvaluationSummary,
    pub scenario_evaluations: Vec<ScenarioEvaluationSummary>,
    pub baseline_results: Vec<BaselineComparatorResult>,
    pub smoothing_comparison_report: Vec<SmoothingComparisonRecord>,
    pub retrieval_latency_report: Vec<RetrievalLatencyRecord>,
    pub bank_validation: HeuristicBankValidationReport,
    pub artifact_completeness: Option<ArtifactCompletenessCheck>,
    pub sweep_results: Vec<SweepPointResult>,
    pub sweep_summary: Option<SweepRunSummary>,
}
