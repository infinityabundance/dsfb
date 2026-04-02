use crate::baselines::{BaselineSet, EwmaFeatureTrace};
use crate::cohort::{MissedFailureDiagnosticRow, OperatorBurdenContributionRow};
use crate::grammar::{FeatureGrammarTrace, GrammarReason, GrammarSet, GrammarState};
use crate::metrics::{BenchmarkMetrics, PerFailureRunSignal};
use crate::precursor::{DsaEvaluation, DsaFeatureTrace, DsaPolicyState, PerFailureRunDsaSignal};
use crate::preprocessing::PreparedDataset;
use crate::residual::{ResidualFeatureTrace, ResidualSet};
use crate::semiotics::{
    DsfbMotifClass, FeatureMotifTrace, GroupDefinitionRecord, MotifSet, ScaffoldSemioticsArtifacts,
    SemanticLayer,
};
use crate::signs::{FeatureSigns, SignSet};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const MAX_FAILURE_CASE_FEATURES: usize = 5;
const MAX_FAILURE_INDEX_ACTIVITY: usize = 3;
const GROUP_FAILURE_COACTIVATION_MIN: usize = 2;
const GROUP_MEMBER_COACTIVATION_MIN: usize = 2;
const MAX_MINIMAL_HEURISTICS: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct FailuresIndex {
    pub total_failure_count: usize,
    pub missed_failure_ids: Vec<usize>,
    pub entries: Vec<FailureIndexEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureIndexEntry {
    pub failure_id: usize,
    pub timestamp: String,
    pub detected_by_dsa: bool,
    pub detected_by_optimized_dsa: bool,
    pub detected_by_ewma: bool,
    pub detected_by_threshold: bool,
    pub lead_time: Option<usize>,
    pub optimized_lead_time: Option<usize>,
    pub feature_activity_summary: Vec<FeatureActivitySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureActivitySummary {
    pub feature_index: usize,
    pub feature_name: String,
    pub max_dsa_score: f64,
    pub max_policy_state: String,
    pub motif_hits: usize,
    pub pressure_hits: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureCaseReport {
    pub failure_id: usize,
    pub failure_timestamp: String,
    pub baseline_detected_by_dsa: bool,
    pub optimized_detected_by_dsa: bool,
    pub detected_by_ewma: bool,
    pub detected_by_threshold: bool,
    pub baseline_dsa_lead_runs: Option<usize>,
    pub optimized_dsa_lead_runs: Option<usize>,
    pub ewma_lead_runs: Option<usize>,
    pub threshold_lead_runs: Option<usize>,
    pub exact_miss_rule: String,
    pub failure_stage: String,
    pub failure_explanation: String,
    pub ewma_detection_explanation: String,
    pub threshold_detection_explanation: String,
    pub top_contributing_features: Vec<FailureCaseFeatureDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureCaseFeatureDetail {
    pub feature_index: usize,
    pub feature_name: String,
    pub ranking_score_proxy: f64,
    pub max_dsa_score: f64,
    pub max_policy_state: String,
    pub initial_motif_hypothesis: String,
    pub dominant_dsfb_motif: String,
    pub dominant_grammar_state: String,
    pub semantic_labels: Vec<String>,
    pub failure_stage: String,
    pub failure_explanation: String,
    pub residual_trajectory: Vec<f64>,
    pub drift_trajectory: Vec<f64>,
    pub slew_trajectory: Vec<f64>,
    pub motif_timeline: Vec<String>,
    pub grammar_state_timeline: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMotifGroundingRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub motif_type: String,
    pub dominant_dsfb_motif: String,
    pub failure_window_semantic_hits: usize,
    pub pass_run_semantic_hits: usize,
    pub failure_window_pressure_hits: usize,
    pub pass_run_pressure_hits: usize,
    pub mean_abs_drift_failure: f64,
    pub mean_abs_drift_pass: f64,
    pub mean_abs_slew_failure: f64,
    pub mean_abs_slew_pass: f64,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MinimalHeuristicEntry {
    pub heuristic_id: String,
    pub target_problem_type: String,
    pub target_identifier: String,
    pub target_feature_name: Option<String>,
    pub target_motif_type: String,
    pub target_grammar_states: Vec<String>,
    pub semantic_requirement: String,
    pub policy_action: String,
    pub burden_effect_class: String,
    pub justification: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyBurdenSummaryRow {
    pub scope: String,
    pub name: String,
    pub watch_points: usize,
    pub review_points: usize,
    pub escalate_points: usize,
    pub pass_review_escalate_points: usize,
    pub pre_failure_review_escalate_points: usize,
    pub silent_suppression_points: usize,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsfbVsEwmaCase {
    pub failure_id: usize,
    pub failure_timestamp: String,
    pub recovered: bool,
    pub recovered_feature_name: String,
    pub baseline_miss_rule: String,
    pub ewma_detected: bool,
    pub ewma_lead_runs: Option<usize>,
    pub optimized_dsa_lead_runs: Option<usize>,
    pub explanation: String,
    pub window: Vec<DsfbVsEwmaWindowPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsfbVsEwmaWindowPoint {
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub ewma_value: f64,
    pub ewma_threshold: f64,
    pub ewma_alarm: bool,
    pub residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub motif_label: String,
    pub grammar_state: String,
    pub policy_state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureDrivenArtifacts {
    pub failures_index: FailuresIndex,
    pub failure_cases: Vec<FailureCaseReport>,
    pub feature_motif_grounding: Vec<FeatureMotifGroundingRecord>,
    pub minimal_heuristics_bank: Vec<MinimalHeuristicEntry>,
    pub policy_burden_summary: Vec<PolicyBurdenSummaryRow>,
    pub dsfb_vs_ewma_cases: Vec<DsfbVsEwmaCase>,
}

struct FeatureBundle<'a> {
    residual: &'a ResidualFeatureTrace,
    sign: &'a FeatureSigns,
    grammar: &'a FeatureGrammarTrace,
    motif: &'a FeatureMotifTrace,
    ewma: &'a EwmaFeatureTrace,
    baseline_dsa: &'a DsaFeatureTrace,
    optimized_dsa: &'a DsaFeatureTrace,
}

#[derive(Debug, Clone)]
struct FeatureActivityCandidate {
    feature_index: usize,
    feature_name: String,
    ranking_score_proxy: f64,
    max_dsa_score: f64,
    max_policy_state: String,
    motif_hits: usize,
    pressure_hits: usize,
}

pub fn build_failure_driven_artifacts(
    dataset: &PreparedDataset,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    motifs: &MotifSet,
    semantic_layer: &SemanticLayer,
    scaffold_semiotics: &ScaffoldSemioticsArtifacts,
    metrics: &BenchmarkMetrics,
    baseline_dsa: &DsaEvaluation,
    optimized_dsa: &DsaEvaluation,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    policy_operator_burden_contributions: &[OperatorBurdenContributionRow],
    pre_failure_lookback_runs: usize,
) -> FailureDrivenArtifacts {
    let feature_bundles = index_feature_bundles(
        residuals,
        signs,
        baselines,
        grammar,
        motifs,
        baseline_dsa,
        optimized_dsa,
    );
    let baseline_by_failure = baseline_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let optimized_by_failure = optimized_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let metrics_by_failure = metrics
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let diagnostics_by_failure = missed_failure_diagnostics
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();

    let failures_index = build_failures_index(
        dataset,
        &feature_bundles,
        baseline_dsa,
        optimized_dsa,
        metrics,
        pre_failure_lookback_runs,
    );
    let failure_cases = build_failure_cases(
        dataset,
        &feature_bundles,
        semantic_layer,
        &baseline_by_failure,
        &optimized_by_failure,
        &metrics_by_failure,
        &diagnostics_by_failure,
        pre_failure_lookback_runs,
    );
    let feature_motif_grounding = build_feature_motif_grounding(
        dataset,
        &feature_bundles,
        semantic_layer,
        baseline_dsa,
        optimized_dsa,
        missed_failure_diagnostics,
        policy_operator_burden_contributions,
        pre_failure_lookback_runs,
    );
    let minimal_heuristics_bank = build_minimal_heuristics_bank(
        missed_failure_diagnostics,
        policy_operator_burden_contributions,
        &feature_motif_grounding,
    );
    let policy_burden_summary = build_policy_burden_summary(dataset, optimized_dsa);
    let dsfb_vs_ewma_cases = build_dsfb_vs_ewma_cases(
        dataset,
        &feature_bundles,
        &optimized_by_failure,
        &metrics_by_failure,
        &diagnostics_by_failure,
        missed_failure_diagnostics,
        pre_failure_lookback_runs,
    );

    let _ = scaffold_semiotics;

    FailureDrivenArtifacts {
        failures_index,
        failure_cases,
        feature_motif_grounding,
        minimal_heuristics_bank,
        policy_burden_summary,
        dsfb_vs_ewma_cases,
    }
}

pub fn validated_group_definitions(
    definitions: &[GroupDefinitionRecord],
) -> Vec<GroupDefinitionRecord> {
    definitions
        .iter()
        .filter(|row| row.validated)
        .cloned()
        .collect()
}

pub fn grouped_semiotics_rejected(definitions: &[GroupDefinitionRecord]) -> bool {
    definitions.iter().all(|row| !row.validated)
}

fn index_feature_bundles<'a>(
    residuals: &'a ResidualSet,
    signs: &'a SignSet,
    baselines: &'a BaselineSet,
    grammar: &'a GrammarSet,
    motifs: &'a MotifSet,
    baseline_dsa: &'a DsaEvaluation,
    optimized_dsa: &'a DsaEvaluation,
) -> BTreeMap<usize, FeatureBundle<'a>> {
    let residual_by_feature = residuals
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let sign_by_feature = signs
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let grammar_by_feature = grammar
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let motif_by_feature = motifs
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let ewma_by_feature = baselines
        .ewma
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let baseline_dsa_by_feature = baseline_dsa
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let optimized_dsa_by_feature = optimized_dsa
        .traces
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();

    residual_by_feature
        .iter()
        .filter_map(|(&feature_index, residual)| {
            Some((
                feature_index,
                FeatureBundle {
                    residual,
                    sign: sign_by_feature.get(&feature_index)?,
                    grammar: grammar_by_feature.get(&feature_index)?,
                    motif: motif_by_feature.get(&feature_index)?,
                    ewma: ewma_by_feature.get(&feature_index)?,
                    baseline_dsa: baseline_dsa_by_feature.get(&feature_index)?,
                    optimized_dsa: optimized_dsa_by_feature.get(&feature_index)?,
                },
            ))
        })
        .collect()
}

fn build_failures_index(
    dataset: &PreparedDataset,
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    baseline_dsa: &DsaEvaluation,
    optimized_dsa: &DsaEvaluation,
    metrics: &BenchmarkMetrics,
    pre_failure_lookback_runs: usize,
) -> FailuresIndex {
    let optimized_by_failure = optimized_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();
    let metrics_by_failure = metrics
        .per_failure_run_signals
        .iter()
        .map(|row| (row.failure_run_index, row))
        .collect::<BTreeMap<_, _>>();

    let missed_failure_ids = baseline_dsa
        .per_failure_run_signals
        .iter()
        .filter(|row| row.earliest_dsa_run.is_none())
        .map(|row| row.failure_run_index)
        .collect::<Vec<_>>();

    let entries = baseline_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| {
            let optimized = optimized_by_failure
                .get(&row.failure_run_index)
                .copied()
                .unwrap_or(row);
            let metric_row = metrics_by_failure
                .get(&row.failure_run_index)
                .copied()
                .unwrap_or_else(|| {
                    panic!(
                        "missing metrics failure row for failure {}",
                        row.failure_run_index
                    )
                });
            let activity = top_feature_activity_for_failure(
                feature_bundles,
                row.failure_run_index,
                pre_failure_lookback_runs,
                MAX_FAILURE_INDEX_ACTIVITY,
                true,
            )
            .into_iter()
            .map(|candidate| FeatureActivitySummary {
                feature_index: candidate.feature_index,
                feature_name: candidate.feature_name,
                max_dsa_score: candidate.max_dsa_score,
                max_policy_state: candidate.max_policy_state,
                motif_hits: candidate.motif_hits,
                pressure_hits: candidate.pressure_hits,
            })
            .collect::<Vec<_>>();

            FailureIndexEntry {
                failure_id: row.failure_run_index,
                timestamp: row.failure_timestamp.clone(),
                detected_by_dsa: row.earliest_dsa_run.is_some(),
                detected_by_optimized_dsa: optimized.earliest_dsa_run.is_some(),
                detected_by_ewma: metric_row.earliest_ewma_run.is_some(),
                detected_by_threshold: metric_row.earliest_threshold_run.is_some(),
                lead_time: row.dsa_lead_runs,
                optimized_lead_time: optimized.dsa_lead_runs,
                feature_activity_summary: activity,
            }
        })
        .collect::<Vec<_>>();

    let _ = dataset;

    FailuresIndex {
        total_failure_count: baseline_dsa.per_failure_run_signals.len(),
        missed_failure_ids,
        entries,
    }
}

fn build_failure_cases(
    dataset: &PreparedDataset,
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    semantic_layer: &SemanticLayer,
    baseline_by_failure: &BTreeMap<usize, &PerFailureRunDsaSignal>,
    optimized_by_failure: &BTreeMap<usize, &PerFailureRunDsaSignal>,
    metrics_by_failure: &BTreeMap<usize, &PerFailureRunSignal>,
    diagnostics_by_failure: &BTreeMap<usize, &MissedFailureDiagnosticRow>,
    pre_failure_lookback_runs: usize,
) -> Vec<FailureCaseReport> {
    baseline_by_failure
        .values()
        .filter(|row| row.earliest_dsa_run.is_none())
        .map(|baseline_row| {
            let failure_index = baseline_row.failure_run_index;
            let optimized_row = optimized_by_failure
                .get(&failure_index)
                .copied()
                .unwrap_or(baseline_row);
            let metrics_row = metrics_by_failure
                .get(&failure_index)
                .copied()
                .unwrap_or_else(|| panic!("missing metrics case for failure {failure_index}"));
            let diagnostic = diagnostics_by_failure.get(&failure_index).copied();
            let start = failure_index.saturating_sub(pre_failure_lookback_runs);
            let top_candidates = top_feature_activity_for_failure(
                feature_bundles,
                failure_index,
                pre_failure_lookback_runs,
                MAX_FAILURE_CASE_FEATURES,
                false,
            );

            let top_contributing_features = top_candidates
                .into_iter()
                .map(|candidate| {
                    let bundle = feature_bundles
                        .get(&candidate.feature_index)
                        .unwrap_or_else(|| panic!("missing feature bundle {}", candidate.feature_index));
                    let semantic_labels =
                        semantic_labels_in_window(semantic_layer, &candidate.feature_name, start, failure_index);
                    let dominant_dsfb_motif = dominant_motif_in_window(bundle.motif, start, failure_index);
                    let dominant_grammar_state =
                        dominant_grammar_state_in_window(bundle.grammar, start, failure_index);
                    let initial_motif_hypothesis =
                        grounded_motif_type_for_window(bundle, start, failure_index).to_string();
                    let (failure_stage, failure_explanation) = feature_failure_explanation(
                        bundle,
                        start,
                        failure_index,
                        diagnostic,
                        &semantic_labels,
                    );

                    FailureCaseFeatureDetail {
                        feature_index: candidate.feature_index,
                        feature_name: candidate.feature_name.clone(),
                        ranking_score_proxy: candidate.ranking_score_proxy,
                        max_dsa_score: candidate.max_dsa_score,
                        max_policy_state: candidate.max_policy_state,
                        initial_motif_hypothesis,
                        dominant_dsfb_motif,
                        dominant_grammar_state,
                        semantic_labels,
                        failure_stage,
                        failure_explanation,
                        residual_trajectory: bundle.residual.residuals[start..failure_index].to_vec(),
                        drift_trajectory: bundle.sign.drift[start..failure_index].to_vec(),
                        slew_trajectory: bundle.sign.slew[start..failure_index].to_vec(),
                        motif_timeline: bundle.motif.labels[start..failure_index]
                            .iter()
                            .map(|label| label.as_lowercase().to_string())
                            .collect(),
                        grammar_state_timeline: (start..failure_index)
                            .map(|run_index| failure_grammar_state_label(bundle.grammar, run_index).to_string())
                            .collect(),
                    }
                })
                .collect::<Vec<_>>();

            let threshold_support = scalar_supporting_features(
                feature_bundles,
                start,
                failure_index,
                ScalarSupportMode::Threshold,
            );
            let ewma_support = scalar_supporting_features(
                feature_bundles,
                start,
                failure_index,
                ScalarSupportMode::Ewma,
            );

            FailureCaseReport {
                failure_id: failure_index,
                failure_timestamp: baseline_row.failure_timestamp.clone(),
                baseline_detected_by_dsa: baseline_row.earliest_dsa_run.is_some(),
                optimized_detected_by_dsa: optimized_row.earliest_dsa_run.is_some(),
                detected_by_ewma: metrics_row.earliest_ewma_run.is_some(),
                detected_by_threshold: metrics_row.earliest_threshold_run.is_some(),
                baseline_dsa_lead_runs: baseline_row.dsa_lead_runs,
                optimized_dsa_lead_runs: optimized_row.dsa_lead_runs,
                ewma_lead_runs: metrics_row.ewma_lead_runs,
                threshold_lead_runs: metrics_row.threshold_lead_runs,
                exact_miss_rule: diagnostic
                    .map(|row| row.exact_miss_rule.clone())
                    .unwrap_or_else(|| "unclassified".into()),
                failure_stage: failure_stage_label(diagnostic),
                failure_explanation: failure_explanation_text(diagnostic, optimized_row),
                ewma_detection_explanation: scalar_detection_explanation(
                    "EWMA",
                    metrics_row.earliest_ewma_run.is_some(),
                    metrics_row.ewma_lead_runs,
                    &ewma_support,
                ),
                threshold_detection_explanation: scalar_detection_explanation(
                    "Threshold",
                    metrics_row.earliest_threshold_run.is_some(),
                    metrics_row.threshold_lead_runs,
                    &threshold_support,
                ),
                top_contributing_features,
            }
        })
        .collect()
}

fn build_feature_motif_grounding(
    dataset: &PreparedDataset,
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    semantic_layer: &SemanticLayer,
    baseline_dsa: &DsaEvaluation,
    optimized_dsa: &DsaEvaluation,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    policy_operator_burden_contributions: &[OperatorBurdenContributionRow],
    pre_failure_lookback_runs: usize,
) -> Vec<FeatureMotifGroundingRecord> {
    let failure_indices = baseline_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| row.failure_run_index)
        .collect::<Vec<_>>();
    let failure_window_mask = build_failure_window_mask(dataset.labels.len(), &failure_indices, pre_failure_lookback_runs);
    let candidate_features = grounding_candidate_features(
        baseline_dsa,
        optimized_dsa,
        missed_failure_diagnostics,
        policy_operator_burden_contributions,
    );

    candidate_features
        .into_iter()
        .filter_map(|feature_index| {
            let bundle = feature_bundles.get(&feature_index)?;
            let failure_semantic_hits = semantic_hit_count(
                semantic_layer,
                &bundle.residual.feature_name,
                &failure_window_mask,
                true,
            );
            let pass_semantic_hits = semantic_hit_count(
                semantic_layer,
                &bundle.residual.feature_name,
                &failure_window_mask,
                false,
            );
            let failure_pressure_hits = pressure_hit_count(bundle.grammar, &failure_window_mask, true);
            let pass_pressure_hits = pressure_hit_count(bundle.grammar, &failure_window_mask, false);
            let mean_abs_drift_failure =
                masked_mean_abs(&bundle.sign.drift, &failure_window_mask, true);
            let mean_abs_drift_pass = masked_mean_abs(&bundle.sign.drift, &failure_window_mask, false);
            let mean_abs_slew_failure =
                masked_mean_abs(&bundle.sign.slew, &failure_window_mask, true);
            let mean_abs_slew_pass = masked_mean_abs(&bundle.sign.slew, &failure_window_mask, false);
            let dominant_dsfb_motif = dominant_motif_for_mask(bundle.motif, &failure_window_mask, true);
            let motif_type = grounded_motif_type(
                dominant_dsfb_motif.as_str(),
                failure_semantic_hits,
                pass_semantic_hits,
                failure_pressure_hits,
                pass_pressure_hits,
                mean_abs_drift_failure,
                mean_abs_slew_failure,
            );

            Some(FeatureMotifGroundingRecord {
                feature_index,
                feature_name: bundle.residual.feature_name.clone(),
                motif_type: motif_type.to_string(),
                dominant_dsfb_motif,
                failure_window_semantic_hits: failure_semantic_hits,
                pass_run_semantic_hits: pass_semantic_hits,
                failure_window_pressure_hits: failure_pressure_hits,
                pass_run_pressure_hits: pass_pressure_hits,
                mean_abs_drift_failure,
                mean_abs_drift_pass,
                mean_abs_slew_failure,
                mean_abs_slew_pass,
                justification: format!(
                    "Failure-window semantic hits={}, pass-run semantic hits={}, failure pressure hits={}, pass pressure hits={}, |drift|_failure={:.4}, |slew|_failure={:.4}.",
                    failure_semantic_hits,
                    pass_semantic_hits,
                    failure_pressure_hits,
                    pass_pressure_hits,
                    mean_abs_drift_failure,
                    mean_abs_slew_failure,
                ),
            })
        })
        .collect()
}

fn build_minimal_heuristics_bank(
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    policy_operator_burden_contributions: &[OperatorBurdenContributionRow],
    feature_motif_grounding: &[FeatureMotifGroundingRecord],
) -> Vec<MinimalHeuristicEntry> {
    let grounding_by_name = feature_motif_grounding
        .iter()
        .map(|row| (row.feature_name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut entries = Vec::new();

    for diagnostic in missed_failure_diagnostics {
        let feature_name = diagnostic
            .optimized_feature_name
            .clone()
            .or_else(|| diagnostic.nearest_feature_name.clone());
        let motif_type = feature_name
            .as_deref()
            .and_then(|name| grounding_by_name.get(name).copied())
            .map(|row| row.motif_type.clone())
            .unwrap_or_else(|| "null".into());
        let (target_grammar_states, policy_action, status, burden_effect_class) =
            match diagnostic.exact_miss_rule.as_str() {
                "directional_consistency_gate" => (
                    vec!["BoundaryGrazing".into(), "SustainedDrift".into()],
                    "bounded Watch->Review promotion with persistence retained and corroboration relaxed only for this feature".into(),
                    if diagnostic.recovered_after_optimization {
                        "active_recovery".into()
                    } else {
                        "candidate_only".into()
                    },
                    "recall_recovery".into(),
                ),
                "watch_class_near_miss_below_numeric_gate" => (
                    vec!["BoundaryGrazing".into()],
                    if diagnostic.recovered_after_optimization {
                        "priority rescue from Watch->Review under high EWMA occupancy and bounded fragmentation".into()
                    } else {
                        "reject rescue because the feature never develops sufficient grammar-qualified motif support".into()
                    },
                    if diagnostic.recovered_after_optimization {
                        "active_recovery".into()
                    } else {
                        "rejected_for_insufficient_structure".into()
                    },
                    if diagnostic.recovered_after_optimization {
                        "recall_recovery".into()
                    } else {
                        "bounded_rejection".into()
                    },
                ),
                _ => (
                    vec!["Admissible".into()],
                    "no policy action accepted because the missed case lacks a recoverable DSFB structure".into(),
                    "rejected_for_insufficient_structure".into(),
                    "bounded_rejection".into(),
                ),
            };

        entries.push(MinimalHeuristicEntry {
            heuristic_id: format!("failure_{}_{}", diagnostic.failure_run_index, diagnostic.exact_miss_rule),
            target_problem_type: "missed_failure".into(),
            target_identifier: diagnostic.failure_run_index.to_string(),
            target_feature_name: feature_name.clone(),
            target_motif_type: motif_type,
            target_grammar_states,
            semantic_requirement: "grammar-qualified semantic match required before any policy promotion".into(),
            policy_action,
            burden_effect_class,
            justification: format!(
                "Built from missed failure {} with nearest feature {:?}, exact miss rule {}, bounded rescue recoverable={}.",
                diagnostic.failure_run_index,
                diagnostic.nearest_feature_name,
                diagnostic.exact_miss_rule,
                diagnostic.bounded_rescue_would_recover,
            ),
            status,
        });
    }

    let nuisance_entries = policy_operator_burden_contributions
        .iter()
        .filter(|row| {
            row.configuration_role == "baseline"
                && row.contribution_scope == "motif"
                && row.contribution_type == "review_escalate_burden"
        })
        .collect::<Vec<_>>();
    let mut nuisance_entries = nuisance_entries;
    nuisance_entries.sort_by(|left, right| {
        right
            .value
            .partial_cmp(&left.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for nuisance in nuisance_entries.into_iter().take(4) {
        entries.push(MinimalHeuristicEntry {
            heuristic_id: format!("nuisance_{}", nuisance.name),
            target_problem_type: "nuisance_class".into(),
            target_identifier: nuisance.name.clone(),
            target_feature_name: None,
            target_motif_type: nuisance.name.clone(),
            target_grammar_states: match nuisance.name.as_str() {
                "recurrent_boundary_approach" => {
                    vec!["BoundaryGrazing".into(), "SustainedDrift".into()]
                }
                "watch_only_boundary_grazing" => vec!["BoundaryGrazing".into()],
                _ => vec!["grammar_filtered".into()],
            },
            semantic_requirement: "apply only after grammar filtering and semantic retrieval".into(),
            policy_action: match nuisance.name.as_str() {
                "recurrent_boundary_approach" => {
                    "keep default Watch ceiling unless persistence and bounded corroboration justify Review".into()
                }
                "watch_only_boundary_grazing" => {
                    "retain Watch-only handling to suppress isolated burden".into()
                }
                _ => "suppress isolated nuisance contribution before Review/Escalate promotion".into(),
            },
            burden_effect_class: "nuisance_suppression".into(),
            justification: format!(
                "Derived from baseline nuisance pattern {} with {:.0} pass-run Review/Escalate points.",
                nuisance.name, nuisance.value
            ),
            status: "active_suppression".into(),
        });
    }

    entries.sort_by(|left, right| left.heuristic_id.cmp(&right.heuristic_id));
    entries.truncate(MAX_MINIMAL_HEURISTICS);
    entries
}

fn build_policy_burden_summary(
    dataset: &PreparedDataset,
    optimized_dsa: &DsaEvaluation,
) -> Vec<PolicyBurdenSummaryRow> {
    let mut rows = optimized_dsa
        .motif_policy_contributions
        .iter()
        .map(|row| PolicyBurdenSummaryRow {
            scope: "motif".into(),
            name: row.motif_name.clone(),
            watch_points: row.watch_points,
            review_points: row.review_points,
            escalate_points: row.escalate_points,
            pass_review_escalate_points: row.pass_review_or_escalate_points,
            pre_failure_review_escalate_points: row.pre_failure_review_or_escalate_points,
            silent_suppression_points: row.silent_suppression_points,
            justification: "optimized selected-row motif burden summary".into(),
        })
        .collect::<Vec<_>>();

    let mut feature_rows = optimized_dsa
        .traces
        .iter()
        .filter_map(|trace| {
            let mut watch_points = 0usize;
            let mut review_points = 0usize;
            let mut escalate_points = 0usize;
            let mut pass_review_escalate_points = 0usize;
            let mut pre_failure_review_escalate_points = 0usize;
            let mut silent_suppression_points = 0usize;
            for run_index in 0..trace.policy_state.len() {
                match trace.policy_state[run_index] {
                    DsaPolicyState::Silent => {
                        if trace.policy_suppressed_to_silent[run_index] {
                            silent_suppression_points += 1;
                        }
                    }
                    DsaPolicyState::Watch => watch_points += 1,
                    DsaPolicyState::Review => {
                        review_points += 1;
                        if dataset.labels[run_index] == -1 {
                            pass_review_escalate_points += 1;
                        }
                    }
                    DsaPolicyState::Escalate => {
                        escalate_points += 1;
                        if dataset.labels[run_index] == -1 {
                            pass_review_escalate_points += 1;
                        }
                    }
                }
                if dataset.labels[run_index] == 1 && trace.dsa_alert[run_index] {
                    pre_failure_review_escalate_points += 1;
                }
            }
            let investigation_points = review_points + escalate_points;
            (investigation_points > 0).then(|| PolicyBurdenSummaryRow {
                scope: "feature".into(),
                name: trace.feature_name.clone(),
                watch_points,
                review_points,
                escalate_points,
                pass_review_escalate_points,
                pre_failure_review_escalate_points,
                silent_suppression_points,
                justification: "optimized selected-row feature burden summary".into(),
            })
        })
        .collect::<Vec<_>>();

    feature_rows.sort_by(|left, right| {
        (right.review_points + right.escalate_points)
            .cmp(&(left.review_points + left.escalate_points))
            .then_with(|| left.name.cmp(&right.name))
    });
    feature_rows.truncate(12);
    rows.extend(feature_rows);
    rows
}

fn build_dsfb_vs_ewma_cases(
    dataset: &PreparedDataset,
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    optimized_by_failure: &BTreeMap<usize, &PerFailureRunDsaSignal>,
    metrics_by_failure: &BTreeMap<usize, &PerFailureRunSignal>,
    diagnostics_by_failure: &BTreeMap<usize, &MissedFailureDiagnosticRow>,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    pre_failure_lookback_runs: usize,
) -> Vec<DsfbVsEwmaCase> {
    missed_failure_diagnostics
        .iter()
        .filter(|row| row.recovered_after_optimization)
        .filter_map(|diagnostic| {
            let feature_name = diagnostic
                .optimized_feature_name
                .clone()
                .or_else(|| diagnostic.nearest_feature_name.clone())?;
            let bundle = feature_bundles
                .values()
                .find(|row| row.residual.feature_name == feature_name)?;
            let optimized_row = optimized_by_failure.get(&diagnostic.failure_run_index).copied()?;
            let metrics_row = metrics_by_failure.get(&diagnostic.failure_run_index).copied()?;
            let start = diagnostic
                .failure_run_index
                .saturating_sub(pre_failure_lookback_runs);
            let window = (start..diagnostic.failure_run_index)
                .map(|run_index| DsfbVsEwmaWindowPoint {
                    run_index,
                    timestamp: dataset.timestamps[run_index]
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                    label: dataset.labels[run_index],
                    ewma_value: bundle.ewma.ewma[run_index],
                    ewma_threshold: bundle.ewma.threshold,
                    ewma_alarm: bundle.ewma.alarm[run_index],
                    residual: bundle.residual.residuals[run_index],
                    drift: bundle.sign.drift[run_index],
                    slew: bundle.sign.slew[run_index],
                    motif_label: bundle.motif.labels[run_index].as_lowercase().into(),
                    grammar_state: failure_grammar_state_label(bundle.grammar, run_index).into(),
                    policy_state: bundle.optimized_dsa.policy_state[run_index]
                        .as_lowercase()
                        .into(),
                })
                .collect::<Vec<_>>();
            let explanation = if metrics_row.earliest_ewma_run.is_some() {
                format!(
                    "EWMA also detected failure {} with lead {:?}, so DSFB does not add scalar coverage here. The DSFB recovery adds structure instead: feature {} shows motif {} under grammar {} and is promoted by a bounded rescue tied to {}.",
                    diagnostic.failure_run_index,
                    metrics_row.ewma_lead_runs,
                    feature_name,
                    dominant_motif_in_window(bundle.motif, start, diagnostic.failure_run_index),
                    dominant_grammar_state_in_window(bundle.grammar, start, diagnostic.failure_run_index),
                    diagnostic.exact_miss_rule,
                )
            } else {
                format!(
                    "EWMA missed failure {}, while DSFB recovered it through feature {} with motif {} under grammar {}.",
                    diagnostic.failure_run_index,
                    feature_name,
                    dominant_motif_in_window(bundle.motif, start, diagnostic.failure_run_index),
                    dominant_grammar_state_in_window(bundle.grammar, start, diagnostic.failure_run_index),
                )
            };
            let _ = diagnostics_by_failure;

            Some(DsfbVsEwmaCase {
                failure_id: diagnostic.failure_run_index,
                failure_timestamp: optimized_row.failure_timestamp.clone(),
                recovered: true,
                recovered_feature_name: feature_name,
                baseline_miss_rule: diagnostic.exact_miss_rule.clone(),
                ewma_detected: metrics_row.earliest_ewma_run.is_some(),
                ewma_lead_runs: metrics_row.ewma_lead_runs,
                optimized_dsa_lead_runs: optimized_row.dsa_lead_runs,
                explanation,
                window,
            })
        })
        .collect()
}

fn top_feature_activity_for_failure(
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    failure_index: usize,
    pre_failure_lookback_runs: usize,
    limit: usize,
    require_nonzero_score: bool,
) -> Vec<FeatureActivityCandidate> {
    let start = failure_index.saturating_sub(pre_failure_lookback_runs);
    let mut candidates = feature_bundles
        .iter()
        .map(|(&feature_index, bundle)| {
            let max_dsa_score = bundle.baseline_dsa.dsa_score[start..failure_index]
                .iter()
                .copied()
                .fold(0.0, f64::max);
            let motif_hits = bundle.baseline_dsa.motif_hit[start..failure_index]
                .iter()
                .filter(|flag| **flag)
                .count();
            let pressure_hits = (start..failure_index)
                .filter(|&run_index| {
                    matches!(
                        failure_grammar_state_label(bundle.grammar, run_index),
                        "BoundaryGrazing"
                            | "SustainedDrift"
                            | "TransientViolation"
                            | "PersistentViolation"
                    )
                })
                .count();
            let max_policy_state = dominant_policy_state_in_window(
                &bundle.baseline_dsa.policy_state[start..failure_index],
            )
            .into();
            FeatureActivityCandidate {
                feature_index,
                feature_name: bundle.residual.feature_name.clone(),
                ranking_score_proxy: max_dsa_score + 0.25 * motif_hits as f64 + 0.25 * pressure_hits as f64,
                max_dsa_score,
                max_policy_state,
                motif_hits,
                pressure_hits,
            }
        })
        .filter(|candidate| !require_nonzero_score || candidate.ranking_score_proxy > 0.0)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .ranking_score_proxy
            .partial_cmp(&left.ranking_score_proxy)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.feature_name.cmp(&right.feature_name))
    });
    candidates.truncate(limit);
    candidates
}

fn semantic_labels_in_window(
    semantic_layer: &SemanticLayer,
    feature_name: &str,
    start: usize,
    failure_index: usize,
) -> Vec<String> {
    let mut labels = semantic_layer
        .semantic_matches
        .iter()
        .filter(|row| {
            row.feature_name == feature_name && row.run_index >= start && row.run_index < failure_index
        })
        .map(|row| row.heuristic_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if labels.is_empty() {
        labels.push("none".into());
    }
    labels
}

fn feature_failure_explanation(
    bundle: &FeatureBundle<'_>,
    start: usize,
    failure_index: usize,
    diagnostic: Option<&MissedFailureDiagnosticRow>,
    semantic_labels: &[String],
) -> (String, String) {
    let has_non_stable_motif = bundle.motif.labels[start..failure_index]
        .iter()
        .any(|label| *label != DsfbMotifClass::StableAdmissible);
    let has_pressure = (start..failure_index).any(|run_index| {
        matches!(
            failure_grammar_state_label(bundle.grammar, run_index),
            "BoundaryGrazing" | "SustainedDrift" | "TransientViolation" | "PersistentViolation"
        )
    });
    let has_semantic = semantic_labels.iter().any(|label| label != "none");
    let has_review = bundle.baseline_dsa.policy_state[start..failure_index]
        .iter()
        .any(|state| matches!(state, DsaPolicyState::Review | DsaPolicyState::Escalate));

    if !has_non_stable_motif {
        (
            "syntax".into(),
            "No non-trivial DSFB motif was detected in the failure lookback window.".into(),
        )
    } else if !has_pressure {
        (
            "grammar".into(),
            "A motif candidate was present but never qualified by a non-admissible envelope interaction.".into(),
        )
    } else if !has_semantic {
        (
            "semantics".into(),
            "Syntax and grammar were present, but no grammar-qualified semantic retrieval survived the heuristics bank.".into(),
        )
    } else if !has_review {
        (
            "policy".into(),
            diagnostic
                .map(|row| {
                    format!(
                        "Semantic evidence existed but policy never promoted beyond {} because of {}.",
                        row.nearest_feature_policy_state
                            .clone()
                            .unwrap_or_else(|| "silent".into()),
                        row.exact_miss_rule
                    )
                })
                .unwrap_or_else(|| {
                    "Semantic evidence existed but policy never promoted to Review/Escalate.".into()
                }),
        )
    } else {
        (
            "support".into(),
            "The feature carried supportive structure but was not the decisive failure-localized signal.".into(),
        )
    }
}

fn failure_stage_label(diagnostic: Option<&MissedFailureDiagnosticRow>) -> String {
    match diagnostic {
        Some(row) if row.ranking_exclusion => "ranking".into(),
        Some(row) if row.cohort_selection => "cohort_selection".into(),
        Some(row) if row.policy_suppression => "policy".into(),
        Some(row) if row.fragmentation_ceiling => "policy_fragmentation".into(),
        Some(row) if row.directional_consistency_gate => "policy_directionality".into(),
        Some(row) if row.persistence_gate => "policy_persistence".into(),
        Some(row) if row.corroboration_threshold => "policy_corroboration".into(),
        Some(row) if row.rescue_gate_not_activating => "rescue".into(),
        Some(_) => "policy".into(),
        None => "unclassified".into(),
    }
}

fn failure_explanation_text(
    diagnostic: Option<&MissedFailureDiagnosticRow>,
    optimized_row: &PerFailureRunDsaSignal,
) -> String {
    match diagnostic {
        Some(row) if row.recovered_after_optimization => format!(
            "Baseline DSA missed the failure because of {} on feature {:?}; the bounded recovery layer later recovered it via {:?}.",
            row.exact_miss_rule, row.nearest_feature_name, row.optimized_feature_name
        ),
        Some(row) => format!(
            "Baseline DSA missed the failure because of {} on feature {:?}; no bounded recovery rule recovered it.",
            row.exact_miss_rule, row.nearest_feature_name
        ),
        None if optimized_row.earliest_dsa_run.is_some() => {
            "The failure was recovered during optimization, but no baseline miss diagnostic row was available.".into()
        }
        None => "The failure remains missed and no detailed diagnostic row was available.".into(),
    }
}

fn scalar_detection_explanation(
    layer_name: &str,
    detected: bool,
    lead_runs: Option<usize>,
    supporting_features: &[String],
) -> String {
    if detected {
        format!(
            "{} detected the failure with lead {:?}; supporting features in the lookback were {}.",
            layer_name,
            lead_runs,
            if supporting_features.is_empty() {
                "none".into()
            } else {
                supporting_features.join(",")
            }
        )
    } else {
        format!("{layer_name} did not detect the failure in the configured lookback window.")
    }
}

fn scalar_supporting_features(
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    start: usize,
    failure_index: usize,
    mode: ScalarSupportMode,
) -> Vec<String> {
    let mut scored = feature_bundles
        .values()
        .map(|bundle| {
            let score = match mode {
                ScalarSupportMode::Threshold => bundle.residual.threshold_alarm[start..failure_index]
                    .iter()
                    .filter(|flag| **flag)
                    .count(),
                ScalarSupportMode::Ewma => bundle.ewma.alarm[start..failure_index]
                    .iter()
                    .filter(|flag| **flag)
                    .count(),
            };
            (bundle.residual.feature_name.clone(), score)
        })
        .filter(|(_, score)| *score > 0)
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    scored.into_iter().take(3).map(|(name, _)| name).collect()
}

fn grounding_candidate_features(
    baseline_dsa: &DsaEvaluation,
    optimized_dsa: &DsaEvaluation,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    policy_operator_burden_contributions: &[OperatorBurdenContributionRow],
) -> Vec<usize> {
    let mut features = BTreeSet::new();
    for row in &baseline_dsa.per_failure_run_signals {
        if let Some(feature_index) = row.max_dsa_score_feature_index {
            features.insert(feature_index);
        }
        if let Some(feature_index) = row.earliest_dsa_feature_index {
            features.insert(feature_index);
        }
    }
    for row in &optimized_dsa.per_failure_run_signals {
        if let Some(feature_index) = row.earliest_dsa_feature_index {
            features.insert(feature_index);
        }
    }
    for row in missed_failure_diagnostics {
        if let Some(feature_name) = row.nearest_feature_name.as_deref() {
            if let Some(index) = feature_index_from_name(feature_name) {
                features.insert(index);
            }
        }
        if let Some(feature_name) = row.optimized_feature_name.as_deref() {
            if let Some(index) = feature_index_from_name(feature_name) {
                features.insert(index);
            }
        }
    }
    let mut burden_features = policy_operator_burden_contributions
        .iter()
        .filter(|row| row.configuration_role == "optimized" && row.contribution_scope == "feature")
        .filter_map(|row| feature_index_from_name(&row.name).map(|index| (index, row.value)))
        .collect::<Vec<_>>();
    burden_features.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (feature_index, _) in burden_features.into_iter().take(12) {
        features.insert(feature_index);
    }
    features.into_iter().collect()
}

fn semantic_hit_count(
    semantic_layer: &SemanticLayer,
    feature_name: &str,
    failure_window_mask: &[bool],
    in_failure_window: bool,
) -> usize {
    semantic_layer
        .semantic_matches
        .iter()
        .filter(|row| row.feature_name == feature_name && failure_window_mask[row.run_index] == in_failure_window)
        .count()
}

fn pressure_hit_count(
    grammar_trace: &FeatureGrammarTrace,
    failure_window_mask: &[bool],
    in_failure_window: bool,
) -> usize {
    grammar_trace
        .raw_states
        .iter()
        .enumerate()
        .filter(|(run_index, _)| failure_window_mask[*run_index] == in_failure_window)
        .filter(|(run_index, _)| {
            matches!(
                failure_grammar_state_label(grammar_trace, *run_index),
                "BoundaryGrazing" | "SustainedDrift" | "TransientViolation" | "PersistentViolation"
            )
        })
        .count()
}

fn masked_mean_abs(values: &[f64], mask: &[bool], select_value: bool) -> f64 {
    let selected = values
        .iter()
        .zip(mask)
        .filter(|(_, flag)| **flag == select_value)
        .map(|(value, _)| value.abs())
        .collect::<Vec<_>>();
    mean(&selected).unwrap_or(0.0)
}

fn dominant_motif_for_mask(
    motif_trace: &FeatureMotifTrace,
    mask: &[bool],
    select_value: bool,
) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for (run_index, label) in motif_trace.labels.iter().enumerate() {
        if mask[run_index] != select_value {
            continue;
        }
        *counts.entry(label.as_lowercase()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(label, _)| label.to_string())
        .unwrap_or_else(|| "stable_admissible".into())
}

fn grounded_motif_type(
    dominant_dsfb_motif: &str,
    failure_semantic_hits: usize,
    pass_semantic_hits: usize,
    failure_pressure_hits: usize,
    pass_pressure_hits: usize,
    mean_abs_drift_failure: f64,
    mean_abs_slew_failure: f64,
) -> &'static str {
    if failure_semantic_hits == 0 && failure_pressure_hits == 0 {
        return "null";
    }
    if pass_semantic_hits > failure_semantic_hits.saturating_mul(3) && failure_pressure_hits <= 1 {
        return "noise_like";
    }
    match dominant_dsfb_motif {
        "pre_failure_slow_drift" => "slow_drift_precursor",
        "recurrent_boundary_approach" | "watch_only_boundary_grazing" => {
            "boundary_grazing_precursor"
        }
        "transition_excursion" => {
            if mean_abs_slew_failure > mean_abs_drift_failure {
                "transient_excursion"
            } else {
                "burst_instability"
            }
        }
        "persistent_instability_cluster" => "persistent_instability",
        "transition_cluster_support" => "burst_instability",
        _ if failure_pressure_hits > pass_pressure_hits && mean_abs_drift_failure > mean_abs_slew_failure => {
            "slow_drift_precursor"
        }
        _ if failure_pressure_hits > pass_pressure_hits => "persistent_instability",
        _ => "noise_like",
    }
}

fn grounded_motif_type_for_window(
    bundle: &FeatureBundle<'_>,
    start: usize,
    failure_index: usize,
) -> &'static str {
    let window_mask = (0..bundle.motif.labels.len())
        .map(|run_index| run_index >= start && run_index < failure_index)
        .collect::<Vec<_>>();
    grounded_motif_type(
        dominant_motif_for_mask(bundle.motif, &window_mask, true).as_str(),
        bundle.motif.labels[start..failure_index]
            .iter()
            .filter(|label| **label != DsfbMotifClass::StableAdmissible)
            .count(),
        0,
        pressure_hit_count(bundle.grammar, &window_mask, true),
        0,
        mean_abs(&bundle.sign.drift[start..failure_index]),
        mean_abs(&bundle.sign.slew[start..failure_index]),
    )
}

fn build_failure_window_mask(
    len: usize,
    failure_indices: &[usize],
    pre_failure_lookback_runs: usize,
) -> Vec<bool> {
    let mut mask = vec![false; len];
    for &failure_index in failure_indices {
        let start = failure_index.saturating_sub(pre_failure_lookback_runs);
        for flag in &mut mask[start..failure_index] {
            *flag = true;
        }
    }
    mask
}

fn dominant_motif_in_window(
    motif_trace: &FeatureMotifTrace,
    start: usize,
    failure_index: usize,
) -> String {
    let mask = (0..motif_trace.labels.len())
        .map(|run_index| run_index >= start && run_index < failure_index)
        .collect::<Vec<_>>();
    dominant_motif_for_mask(motif_trace, &mask, true)
}

fn dominant_grammar_state_in_window(
    grammar_trace: &FeatureGrammarTrace,
    start: usize,
    failure_index: usize,
) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for run_index in start..failure_index {
        *counts
            .entry(failure_grammar_state_label(grammar_trace, run_index))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(label, _)| label.to_string())
        .unwrap_or_else(|| "Admissible".into())
}

fn dominant_policy_state_in_window(states: &[DsaPolicyState]) -> &'static str {
    if states.iter().any(|state| matches!(state, DsaPolicyState::Escalate)) {
        "escalate"
    } else if states.iter().any(|state| matches!(state, DsaPolicyState::Review)) {
        "review"
    } else if states.iter().any(|state| matches!(state, DsaPolicyState::Watch)) {
        "watch"
    } else {
        "silent"
    }
}

fn failure_grammar_state_label(grammar_trace: &FeatureGrammarTrace, run_index: usize) -> &'static str {
    if grammar_trace.raw_states[run_index] == GrammarState::Violation {
        if grammar_trace.persistent_violation[run_index] {
            "PersistentViolation"
        } else {
            "TransientViolation"
        }
    } else if grammar_trace.raw_states[run_index] == GrammarState::Boundary {
        if grammar_trace.raw_reasons[run_index] == GrammarReason::SustainedOutwardDrift {
            "SustainedDrift"
        } else {
            "BoundaryGrazing"
        }
    } else if run_index > 0
        && matches!(
            failure_grammar_state_label(grammar_trace, run_index - 1),
            "BoundaryGrazing" | "SustainedDrift" | "TransientViolation" | "PersistentViolation"
        )
    {
        "Recovery"
    } else {
        "Admissible"
    }
}

fn feature_index_from_name(feature_name: &str) -> Option<usize> {
    feature_name
        .strip_prefix('S')
        .and_then(|value| value.parse::<usize>().ok())
        .and_then(|value| value.checked_sub(1))
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
}

fn mean_abs(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|value| value.abs()).sum::<f64>() / values.len() as f64
    }
}

enum ScalarSupportMode {
    Threshold,
    Ewma,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_semiotics_rejection_helper_is_deterministic() {
        let definitions = vec![
            GroupDefinitionRecord {
                group_name: "group_a".into(),
                member_features: "S001,S002".into(),
                member_roles: "a,b".into(),
                preferred_motifs: "slow_drift_precursor".into(),
                empirical_basis: "fixture".into(),
                group_size: 2,
                rescue_eligible_member_count: 1,
                highest_rescue_priority: "high".into(),
                semantic_match_count: 0,
                dominant_group_heuristic: None,
                pressure_run_count: 0,
                violation_run_count: 0,
                mean_active_feature_count: 0.0,
                mean_envelope_separation: 0.0,
                coactivation_member_threshold: 2,
                minimum_failure_coactivation_runs: 2,
                failure_coactivation_run_count: 1,
                pass_coactivation_run_count: 0,
                validated: false,
                rejection_reason: Some("failure co-activation below threshold".into()),
            },
            GroupDefinitionRecord {
                group_name: "group_b".into(),
                member_features: "S003,S004".into(),
                member_roles: "c,d".into(),
                preferred_motifs: "persistent_instability".into(),
                empirical_basis: "fixture".into(),
                group_size: 2,
                rescue_eligible_member_count: 1,
                highest_rescue_priority: "medium".into(),
                semantic_match_count: 3,
                dominant_group_heuristic: Some("persistent_instability_cluster".into()),
                pressure_run_count: 3,
                violation_run_count: 1,
                mean_active_feature_count: 2.0,
                mean_envelope_separation: 0.8,
                coactivation_member_threshold: 2,
                minimum_failure_coactivation_runs: 2,
                failure_coactivation_run_count: 2,
                pass_coactivation_run_count: 0,
                validated: true,
                rejection_reason: None,
            },
        ];

        assert!(!grouped_semiotics_rejected(&definitions));
        let validated = validated_group_definitions(&definitions);
        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0].group_name, "group_b");
    }

    #[test]
    fn grounded_motif_type_prefers_null_without_failure_structure() {
        assert_eq!(grounded_motif_type("stable_admissible", 0, 0, 0, 0, 0.0, 0.0), "null");
        assert_eq!(
            grounded_motif_type("recurrent_boundary_approach", 2, 10, 1, 5, 0.1, 0.1),
            "noise_like"
        );
    }
}
