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
const MAX_MINIMAL_HEURISTICS: usize = 20;

#[derive(Debug, Clone, Copy)]
struct FeatureRoleLockSpec {
    feature_name: &'static str,
    initial_role: &'static str,
    preferred_semantic_labels: &'static [&'static str],
    preferred_grounded_motif_types: &'static [&'static str],
    preferred_grammar_states: &'static [&'static str],
    revised_role: Option<&'static str>,
}

const FEATURE_ROLE_LOCK: &[FeatureRoleLockSpec] = &[
    FeatureRoleLockSpec {
        feature_name: "S059",
        initial_role: "primary recurrent-boundary precursor",
        preferred_semantic_labels: &["recurrent_boundary_approach", "pre_failure_slow_drift"],
        preferred_grounded_motif_types: &["boundary_grazing_precursor", "slow_drift_precursor"],
        preferred_grammar_states: &["BoundaryGrazing", "SustainedDrift"],
        revised_role: Some("persistence-gated recurrent-boundary review feature"),
    },
    FeatureRoleLockSpec {
        feature_name: "S123",
        initial_role: "transition / instability feature",
        preferred_semantic_labels: &["transition_excursion", "persistent_instability_cluster"],
        preferred_grounded_motif_types: &["persistent_instability", "burst_instability"],
        preferred_grammar_states: &["TransientViolation", "PersistentViolation"],
        revised_role: Some("transition / instability support feature"),
    },
    FeatureRoleLockSpec {
        feature_name: "S133",
        initial_role: "candidate slow-drift precursor",
        preferred_semantic_labels: &["pre_failure_slow_drift"],
        preferred_grounded_motif_types: &["slow_drift_precursor"],
        preferred_grammar_states: &["SustainedDrift"],
        revised_role: Some("semantically ambiguous review-only candidate"),
    },
    FeatureRoleLockSpec {
        feature_name: "S540",
        initial_role: "burst-support corroborator",
        preferred_semantic_labels: &["transition_cluster_support"],
        preferred_grounded_motif_types: &["burst_instability"],
        preferred_grammar_states: &["TransientViolation", "PersistentViolation"],
        revised_role: Some("review-only burst-support corroborator"),
    },
    FeatureRoleLockSpec {
        feature_name: "S128",
        initial_role: "co-burst corroborator",
        preferred_semantic_labels: &["transition_cluster_support"],
        preferred_grounded_motif_types: &["burst_instability"],
        preferred_grammar_states: &["TransientViolation", "PersistentViolation"],
        revised_role: Some("review-only co-burst corroborator"),
    },
    FeatureRoleLockSpec {
        feature_name: "S104",
        initial_role: "watch-only sentinel",
        preferred_semantic_labels: &["watch_only_boundary_grazing"],
        preferred_grounded_motif_types: &["boundary_grazing_precursor", "noise_like"],
        preferred_grammar_states: &["BoundaryGrazing"],
        revised_role: Some("watch-only nuisance sentinel"),
    },
    FeatureRoleLockSpec {
        feature_name: "S134",
        initial_role: "recall-rescue feature",
        preferred_semantic_labels: &[],
        preferred_grounded_motif_types: &["recovery_pattern", "persistent_instability"],
        preferred_grammar_states: &["BoundaryGrazing", "Recovery"],
        revised_role: Some("bounded recall-rescue feature"),
    },
    FeatureRoleLockSpec {
        feature_name: "S275",
        initial_role: "recall-rescue feature",
        preferred_semantic_labels: &[],
        preferred_grounded_motif_types: &["recovery_pattern", "persistent_instability"],
        preferred_grammar_states: &["BoundaryGrazing", "Recovery"],
        revised_role: Some("bounded recall-rescue feature"),
    },
];

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
    pub behavior_classification: String,
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
pub struct MissedFailurePriorityRow {
    pub failure_id: usize,
    pub timestamp: String,
    pub exact_miss_rule: String,
    pub top_feature_name: Option<String>,
    pub signal_strength: f64,
    pub feature_concentration: f64,
    pub separation_from_noise: f64,
    pub recoverability_estimate: f64,
    pub priority_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureToMotifRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub motif_type: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NegativeControlReport {
    pub pass_run_count: usize,
    pub pass_run_false_activation_count: usize,
    pub false_activation_rate: f64,
    pub pass_run_false_episode_count: usize,
    pub false_episode_rate: f64,
    pub clean_window_count: usize,
    pub clean_window_false_activation_count: usize,
    pub clean_window_false_activation_rate: f64,
    pub clean_window_false_episode_count: usize,
    pub clean_window_false_episode_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureRoleValidationRow {
    pub feature_id: String,
    pub initial_role: String,
    #[serde(rename = "supported / revised / rejected")]
    pub validation_result: String,
    #[serde(rename = "motif evidence summary")]
    pub motif_evidence_summary: String,
    #[serde(rename = "grammar evidence summary")]
    pub grammar_evidence_summary: String,
    #[serde(rename = "pass-run burden summary")]
    pub pass_run_burden_summary: String,
    #[serde(rename = "failure contribution summary")]
    pub failure_contribution_summary: String,
    pub final_role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupValidationRow {
    pub group_id: String,
    pub group_members: String,
    #[serde(rename = "failure co-activation count")]
    pub failure_coactivation_count: usize,
    #[serde(rename = "pass co-activation count")]
    pub pass_coactivation_count: usize,
    #[serde(rename = "retained_or_rejected")]
    pub retained_or_rejected: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicProvenanceRow {
    pub heuristic_id: String,
    pub derived_from_failures: String,
    pub uses_features: String,
    pub targets_nuisance_class: String,
    pub intended_effect: String,
    pub action: String,
    pub constraints: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMotifGroundingRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub motif_type: String,
    pub dominant_dsfb_motif: String,
    pub dominant_grammar_state: String,
    pub failure_local_recovery_case_count: usize,
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
    pub missed_failure_priority: Vec<MissedFailurePriorityRow>,
    pub failure_cases: Vec<FailureCaseReport>,
    pub feature_motif_grounding: Vec<FeatureMotifGroundingRecord>,
    pub feature_to_motif: Vec<FeatureToMotifRecord>,
    pub negative_control_report: NegativeControlReport,
    pub minimal_heuristics_bank: Vec<MinimalHeuristicEntry>,
    pub heuristic_provenance: Vec<HeuristicProvenanceRow>,
    pub policy_burden_summary: Vec<PolicyBurdenSummaryRow>,
    pub feature_role_validation: Vec<FeatureRoleValidationRow>,
    pub group_validation: Vec<GroupValidationRow>,
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
        &failure_cases,
        pre_failure_lookback_runs,
    );
    let missed_failure_priority =
        build_missed_failure_priority(&failure_cases, &feature_motif_grounding);
    let feature_to_motif = build_feature_to_motif(&failure_cases, &feature_motif_grounding);
    let negative_control_report =
        build_negative_control_report(dataset, optimized_dsa, pre_failure_lookback_runs);
    let minimal_heuristics_bank = build_minimal_heuristics_bank(
        missed_failure_diagnostics,
        policy_operator_burden_contributions,
        &feature_motif_grounding,
    );
    let heuristic_provenance = build_heuristic_provenance(&minimal_heuristics_bank);
    let policy_burden_summary = build_policy_burden_summary(dataset, optimized_dsa);
    let feature_role_validation = build_feature_role_validation(
        dataset,
        &feature_bundles,
        semantic_layer,
        &feature_motif_grounding,
        &policy_burden_summary,
        optimized_dsa,
        missed_failure_diagnostics,
        pre_failure_lookback_runs,
    );
    let group_validation = build_group_validation(&scaffold_semiotics.group_definitions);
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
        missed_failure_priority,
        failure_cases,
        feature_motif_grounding,
        feature_to_motif,
        negative_control_report,
        minimal_heuristics_bank,
        heuristic_provenance,
        policy_burden_summary,
        feature_role_validation,
        group_validation,
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
                        .unwrap_or_else(|| {
                            panic!("missing feature bundle {}", candidate.feature_index)
                        });
                    let semantic_labels = semantic_labels_in_window(
                        semantic_layer,
                        &candidate.feature_name,
                        start,
                        failure_index,
                    );
                    let dominant_dsfb_motif =
                        dominant_motif_in_window(bundle.motif, start, failure_index);
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
                        behavior_classification: classify_feature_behavior(
                            &bundle.sign.drift[start..failure_index],
                            &bundle.sign.slew[start..failure_index],
                        )
                        .into(),
                        ranking_score_proxy: candidate.ranking_score_proxy,
                        max_dsa_score: candidate.max_dsa_score,
                        max_policy_state: candidate.max_policy_state,
                        initial_motif_hypothesis,
                        dominant_dsfb_motif,
                        dominant_grammar_state,
                        semantic_labels,
                        failure_stage,
                        failure_explanation,
                        residual_trajectory: bundle.residual.residuals[start..failure_index]
                            .to_vec(),
                        drift_trajectory: bundle.sign.drift[start..failure_index].to_vec(),
                        slew_trajectory: bundle.sign.slew[start..failure_index].to_vec(),
                        motif_timeline: bundle.motif.labels[start..failure_index]
                            .iter()
                            .map(|label| label.as_lowercase().to_string())
                            .collect(),
                        grammar_state_timeline: (start..failure_index)
                            .map(|run_index| {
                                failure_grammar_state_label(bundle.grammar, run_index).to_string()
                            })
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
    failure_cases: &[FailureCaseReport],
    pre_failure_lookback_runs: usize,
) -> Vec<FeatureMotifGroundingRecord> {
    let failure_indices = baseline_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| row.failure_run_index)
        .collect::<Vec<_>>();
    let failure_window_mask = build_failure_window_mask(
        dataset.labels.len(),
        &failure_indices,
        pre_failure_lookback_runs,
    );
    let candidate_features = grounding_candidate_features(
        baseline_dsa,
        optimized_dsa,
        missed_failure_diagnostics,
        policy_operator_burden_contributions,
        failure_cases,
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
            let dominant_grammar_state =
                dominant_grammar_state_for_mask(bundle.grammar, &failure_window_mask, true);
            let failure_local_recovery_case_count = failure_cases
                .iter()
                .flat_map(|case| case.top_contributing_features.iter())
                .filter(|feature| {
                    feature.feature_index == feature_index
                        && feature.initial_motif_hypothesis == "recovery_pattern"
                        && feature.dominant_grammar_state == "Recovery"
                })
                .count();
            let motif_type = grounded_motif_type(
                dominant_dsfb_motif.as_str(),
                dominant_grammar_state.as_str(),
                failure_local_recovery_case_count,
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
                dominant_grammar_state,
                failure_local_recovery_case_count,
                failure_window_semantic_hits: failure_semantic_hits,
                pass_run_semantic_hits: pass_semantic_hits,
                failure_window_pressure_hits: failure_pressure_hits,
                pass_run_pressure_hits: pass_pressure_hits,
                mean_abs_drift_failure,
                mean_abs_drift_pass,
                mean_abs_slew_failure,
                mean_abs_slew_pass,
                justification: format!(
                    "Failure-window semantic hits={}, pass-run semantic hits={}, failure pressure hits={}, pass pressure hits={}, failure-local recovery cases={}, |drift|_failure={:.4}, |slew|_failure={:.4}.",
                    failure_semantic_hits,
                    pass_semantic_hits,
                    failure_pressure_hits,
                    pass_pressure_hits,
                    failure_local_recovery_case_count,
                    mean_abs_drift_failure,
                    mean_abs_slew_failure,
                ),
            })
        })
        .collect()
}

fn build_missed_failure_priority(
    failure_cases: &[FailureCaseReport],
    feature_motif_grounding: &[FeatureMotifGroundingRecord],
) -> Vec<MissedFailurePriorityRow> {
    let grounding_by_name = feature_motif_grounding
        .iter()
        .map(|row| (row.feature_name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut rows = failure_cases
        .iter()
        .map(|case| {
            let signal_strength = case
                .top_contributing_features
                .iter()
                .map(|feature| feature.max_dsa_score)
                .fold(0.0, f64::max);
            let total_proxy = case
                .top_contributing_features
                .iter()
                .map(|feature| feature.ranking_score_proxy)
                .sum::<f64>();
            let feature_concentration = case
                .top_contributing_features
                .first()
                .map(|feature| feature.ranking_score_proxy / total_proxy.max(1.0e-9))
                .unwrap_or(0.0);
            let top_feature_name = case
                .top_contributing_features
                .first()
                .map(|feature| feature.feature_name.clone());
            let separation_from_noise = top_feature_name
                .as_deref()
                .and_then(|feature_name| grounding_by_name.get(feature_name))
                .map(|row| {
                    (row.failure_window_semantic_hits as f64
                        + row.failure_window_pressure_hits as f64)
                        / (1.0
                            + row.pass_run_semantic_hits as f64
                            + row.pass_run_pressure_hits as f64)
                })
                .unwrap_or(0.0);
            let recoverability_estimate = if case.optimized_detected_by_dsa {
                1.0
            } else {
                0.0
            };
            let priority_score = signal_strength
                + feature_concentration
                + separation_from_noise
                + recoverability_estimate;
            MissedFailurePriorityRow {
                failure_id: case.failure_id,
                timestamp: case.failure_timestamp.clone(),
                exact_miss_rule: case.exact_miss_rule.clone(),
                top_feature_name,
                signal_strength,
                feature_concentration,
                separation_from_noise,
                recoverability_estimate,
                priority_score,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .priority_score
            .partial_cmp(&left.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.failure_id.cmp(&right.failure_id))
    });
    rows
}

fn build_feature_to_motif(
    failure_cases: &[FailureCaseReport],
    feature_motif_grounding: &[FeatureMotifGroundingRecord],
) -> Vec<FeatureToMotifRecord> {
    let mut counts = BTreeMap::<usize, usize>::new();
    for case in failure_cases {
        for feature in &case.top_contributing_features {
            *counts.entry(feature.feature_index).or_default() += 1;
        }
    }
    let grounding_by_index = feature_motif_grounding
        .iter()
        .map(|row| (row.feature_index, row))
        .collect::<BTreeMap<_, _>>();
    let mut selected = counts.into_iter().collect::<Vec<_>>();
    selected.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    selected
        .into_iter()
        .take(15)
        .filter_map(|(feature_index, _)| {
            let grounding = grounding_by_index.get(&feature_index)?;
            Some(FeatureToMotifRecord {
                feature_index,
                feature_name: grounding.feature_name.clone(),
                motif_type: grounding.motif_type.clone(),
                justification: grounding.justification.clone(),
            })
        })
        .collect()
}

fn build_negative_control_report(
    dataset: &PreparedDataset,
    optimized_dsa: &DsaEvaluation,
    pre_failure_lookback_runs: usize,
) -> NegativeControlReport {
    let failure_indices = dataset
        .labels
        .iter()
        .enumerate()
        .filter_map(|(run_index, label)| (*label == 1).then_some(run_index))
        .collect::<Vec<_>>();
    let failure_window_mask = build_failure_window_mask(
        dataset.labels.len(),
        &failure_indices,
        pre_failure_lookback_runs,
    );
    let pass_run_count = dataset.labels.iter().filter(|label| **label == -1).count();
    let pass_run_false_activation_count = optimized_dsa
        .run_signals
        .primary_run_alert
        .iter()
        .enumerate()
        .filter(|(run_index, flag)| dataset.labels[*run_index] == -1 && **flag)
        .count();
    let pass_only_signal = optimized_dsa
        .run_signals
        .primary_run_alert
        .iter()
        .enumerate()
        .map(|(run_index, flag)| dataset.labels[run_index] == -1 && *flag)
        .collect::<Vec<_>>();
    let clean_window_mask = dataset
        .labels
        .iter()
        .enumerate()
        .map(|(run_index, label)| *label == -1 && !failure_window_mask[run_index])
        .collect::<Vec<_>>();
    let clean_window_count = clean_window_mask.iter().filter(|flag| **flag).count();
    let clean_window_false_activation_count = optimized_dsa
        .run_signals
        .primary_run_alert
        .iter()
        .enumerate()
        .filter(|(run_index, flag)| clean_window_mask[*run_index] && **flag)
        .count();
    let clean_window_signal = optimized_dsa
        .run_signals
        .primary_run_alert
        .iter()
        .enumerate()
        .map(|(run_index, flag)| clean_window_mask[run_index] && *flag)
        .collect::<Vec<_>>();

    NegativeControlReport {
        pass_run_count,
        pass_run_false_activation_count,
        false_activation_rate: pass_run_false_activation_count as f64
            / pass_run_count.max(1) as f64,
        pass_run_false_episode_count: episode_ranges(&pass_only_signal).len(),
        false_episode_rate: episode_ranges(&pass_only_signal).len() as f64
            / pass_run_count.max(1) as f64,
        clean_window_count,
        clean_window_false_activation_count,
        clean_window_false_activation_rate: clean_window_false_activation_count as f64
            / clean_window_count.max(1) as f64,
        clean_window_false_episode_count: episode_ranges(&clean_window_signal).len(),
        clean_window_false_episode_rate: episode_ranges(&clean_window_signal).len() as f64
            / clean_window_count.max(1) as f64,
    }
}

fn build_heuristic_provenance(
    minimal_heuristics_bank: &[MinimalHeuristicEntry],
) -> Vec<HeuristicProvenanceRow> {
    minimal_heuristics_bank
        .iter()
        .map(|entry| {
            let derived_from_failures = if entry.target_problem_type == "missed_failure" {
                bracketed_csv_list(&[entry.target_identifier.clone()])
            } else {
                "[]".into()
            };
            let uses_features = entry
                .target_feature_name
                .as_ref()
                .map(|feature_name| bracketed_csv_list(std::slice::from_ref(feature_name)))
                .unwrap_or_else(|| "[]".into());
            let targets_nuisance_class = if entry.target_problem_type == "nuisance_class" {
                entry.target_identifier.clone()
            } else {
                String::new()
            };
            let intended_effect = if entry.target_problem_type == "missed_failure" {
                "recover_failure"
            } else {
                "suppress_burden"
            };
            HeuristicProvenanceRow {
                heuristic_id: entry.heuristic_id.clone(),
                derived_from_failures,
                uses_features,
                targets_nuisance_class,
                intended_effect: intended_effect.into(),
                action: entry.policy_action.clone(),
                constraints: format!(
                    "motif={}, grammar_states={}, semantic_requirement={}",
                    entry.target_motif_type,
                    entry.target_grammar_states.join("|"),
                    entry.semantic_requirement
                ),
            }
        })
        .collect()
}

fn build_feature_role_validation(
    dataset: &PreparedDataset,
    feature_bundles: &BTreeMap<usize, FeatureBundle<'_>>,
    semantic_layer: &SemanticLayer,
    feature_motif_grounding: &[FeatureMotifGroundingRecord],
    policy_burden_summary: &[PolicyBurdenSummaryRow],
    optimized_dsa: &DsaEvaluation,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
    pre_failure_lookback_runs: usize,
) -> Vec<FeatureRoleValidationRow> {
    let failure_indices = optimized_dsa
        .per_failure_run_signals
        .iter()
        .map(|row| row.failure_run_index)
        .collect::<Vec<_>>();
    let failure_window_mask = build_failure_window_mask(
        dataset.labels.len(),
        &failure_indices,
        pre_failure_lookback_runs,
    );
    let grounding_by_name = feature_motif_grounding
        .iter()
        .map(|row| (row.feature_name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let burden_by_name = policy_burden_summary
        .iter()
        .filter(|row| row.scope == "feature")
        .map(|row| (row.name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let earliest_counts = optimized_dsa
        .per_failure_run_signals
        .iter()
        .filter_map(|row| row.earliest_dsa_feature_name.as_deref())
        .fold(BTreeMap::<&str, usize>::new(), |mut acc, feature_name| {
            *acc.entry(feature_name).or_default() += 1;
            acc
        });
    let max_score_counts = optimized_dsa
        .per_failure_run_signals
        .iter()
        .filter_map(|row| row.max_dsa_score_feature_name.as_deref())
        .fold(BTreeMap::<&str, usize>::new(), |mut acc, feature_name| {
            *acc.entry(feature_name).or_default() += 1;
            acc
        });
    let direct_miss_counts = missed_failure_diagnostics
        .iter()
        .filter_map(|row| row.nearest_feature_name.as_deref())
        .fold(BTreeMap::<&str, usize>::new(), |mut acc, feature_name| {
            *acc.entry(feature_name).or_default() += 1;
            acc
        });
    let optimized_target_counts = missed_failure_diagnostics
        .iter()
        .filter_map(|row| row.optimized_feature_name.as_deref())
        .fold(BTreeMap::<&str, usize>::new(), |mut acc, feature_name| {
            *acc.entry(feature_name).or_default() += 1;
            acc
        });
    let recovered_by_name = missed_failure_diagnostics
        .iter()
        .filter(|row| row.recovered_after_optimization)
        .filter_map(|row| row.optimized_feature_name.as_deref())
        .fold(BTreeMap::<&str, usize>::new(), |mut acc, feature_name| {
            *acc.entry(feature_name).or_default() += 1;
            acc
        });

    FEATURE_ROLE_LOCK
        .iter()
        .map(|spec| {
            let feature_name = spec.feature_name;
            let grounding = grounding_by_name.get(feature_name).copied();
            let burden = burden_by_name.get(feature_name).copied();
            let bundle = feature_index_from_name(feature_name)
                .and_then(|feature_index| feature_bundles.get(&feature_index));
            let earliest_count = earliest_counts.get(feature_name).copied().unwrap_or(0);
            let max_score_count = max_score_counts.get(feature_name).copied().unwrap_or(0);
            let direct_miss_count = direct_miss_counts.get(feature_name).copied().unwrap_or(0);
            let optimized_target_count = optimized_target_counts.get(feature_name).copied().unwrap_or(0);
            let recovered_count = recovered_by_name.get(feature_name).copied().unwrap_or(0);
            let pass_burden = burden
                .map(|row| row.pass_review_escalate_points)
                .unwrap_or(0);
            let linked_failure_ids = linked_failure_ids_for_feature(
                feature_name,
                optimized_dsa,
                missed_failure_diagnostics,
            );
            let preferred_failure_semantic_hits = preferred_semantic_hit_count(
                semantic_layer,
                feature_name,
                &failure_window_mask,
                true,
                spec.preferred_semantic_labels,
            );
            let preferred_pass_semantic_hits = preferred_semantic_hit_count(
                semantic_layer,
                feature_name,
                &failure_window_mask,
                false,
                spec.preferred_semantic_labels,
            );
            let preferred_failure_grammar_hits = bundle
                .map(|bundle| {
                    preferred_grammar_hit_count(
                        bundle.grammar,
                        &failure_window_mask,
                        true,
                        spec.preferred_grammar_states,
                    )
                })
                .unwrap_or(0);
            let preferred_pass_grammar_hits = bundle
                .map(|bundle| {
                    preferred_grammar_hit_count(
                        bundle.grammar,
                        &failure_window_mask,
                        false,
                        spec.preferred_grammar_states,
                    )
                })
                .unwrap_or(0);
            let (linked_groundings, preferred_grounded_windows) = bundle
                .map(|bundle| {
                    linked_window_grounding_counts(
                        bundle,
                        &linked_failure_ids,
                        pre_failure_lookback_runs,
                        spec.preferred_grounded_motif_types,
                    )
                })
                .unwrap_or_default();
            let (linked_grammar, preferred_grammar_windows) = bundle
                .map(|bundle| {
                    linked_window_grammar_counts(
                        bundle.grammar,
                        &linked_failure_ids,
                        pre_failure_lookback_runs,
                        spec.preferred_grammar_states,
                    )
                })
                .unwrap_or_default();

            let structural_support = preferred_grounded_windows > 0
                || preferred_grammar_windows > 0
                || (preferred_failure_semantic_hits > 0
                    && preferred_failure_semantic_hits >= preferred_pass_semantic_hits);
            let operational_failure_link = earliest_count + max_score_count + direct_miss_count + optimized_target_count;
            let (validation_result, final_role) = match feature_name {
                "S059" => {
                    if structural_support && operational_failure_link > 0 {
                        ("supported", spec.initial_role)
                    } else if operational_failure_link > 0 {
                        (
                            "revised",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    } else if pass_burden > 0 {
                        (
                            "rejected",
                            "high-burden recurrent-boundary candidate without failure-local support",
                        )
                    } else {
                        ("revised", spec.revised_role.unwrap_or(spec.initial_role))
                    }
                }
                "S123" => {
                    if structural_support && (earliest_count > 0 || max_score_count > 0) {
                        ("supported", spec.initial_role)
                    } else if operational_failure_link > 0 {
                        (
                            "revised",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    } else {
                        ("rejected", "transition / instability feature without failure-local support")
                    }
                }
                "S133" => {
                    if structural_support
                        && preferred_failure_semantic_hits > preferred_pass_semantic_hits
                        && preferred_grammar_windows > 0
                    {
                        ("supported", spec.initial_role)
                    } else {
                        (
                            "revised",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    }
                }
                "S540" | "S128" => {
                    if structural_support && max_score_count > 0 {
                        ("supported", spec.initial_role)
                    } else if operational_failure_link > 0 {
                        (
                            "revised",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    } else {
                        ("rejected", "corroborator remained failure-unlinked")
                    }
                }
                "S104" => {
                    let sentinel_supported = grounding.is_some_and(|row| {
                        matches!(
                            row.motif_type.as_str(),
                            "boundary_grazing_precursor" | "noise_like"
                        )
                    }) && pass_burden == 0
                        && earliest_count == 0
                        && recovered_count == 0;
                    if sentinel_supported {
                        ("supported", spec.initial_role)
                    } else {
                        (
                            "revised",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    }
                }
                "S134" | "S275" => {
                    if direct_miss_count > 0 || optimized_target_count > 0 || recovered_count > 0 {
                        (
                            "supported",
                            spec.revised_role.unwrap_or(spec.initial_role),
                        )
                    } else {
                        ("rejected", "unlinked recall-rescue feature")
                    }
                }
                _ => ("supported", spec.initial_role),
            };

            FeatureRoleValidationRow {
                feature_id: feature_name.into(),
                initial_role: spec.initial_role.into(),
                validation_result: validation_result.into(),
                motif_evidence_summary: format!(
                    "preferred semantic hits failure/pass={}/{}, linked failure windows={}, linked grounded motifs={}, global grounding={}",
                    preferred_failure_semantic_hits,
                    preferred_pass_semantic_hits,
                    linked_failure_ids.len(),
                    summarize_count_map(&linked_groundings),
                    grounding
                        .map(|row| format!("{} via {}", row.motif_type, row.dominant_dsfb_motif))
                        .unwrap_or_else(|| "none".into()),
                ),
                grammar_evidence_summary: format!(
                    "preferred grammar hits failure/pass={}/{}, preferred linked windows={}, linked dominant grammar={}, global grammar={}",
                    preferred_failure_grammar_hits,
                    preferred_pass_grammar_hits,
                    preferred_grammar_windows,
                    summarize_count_map(&linked_grammar),
                    grounding
                        .map(|row| row.dominant_grammar_state.clone())
                        .unwrap_or_else(|| "none".into()),
                ),
                pass_run_burden_summary: burden
                    .map(|row| {
                        format!(
                            "pass review/escalate points={}, silent suppressions={}, preferred semantic pass hits={}, preferred grammar pass hits={}",
                            row.pass_review_escalate_points,
                            row.silent_suppression_points,
                            preferred_pass_semantic_hits,
                            preferred_pass_grammar_hits,
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "no pass-run burden, preferred semantic pass hits={}, preferred grammar pass hits={}",
                            preferred_pass_semantic_hits, preferred_pass_grammar_hits
                        )
                    }),
                failure_contribution_summary: format!(
                    "earliest detection count={}, max-score failure count={}, direct miss linkages={}, optimized rescue targets={}, recovered missed failures={}, linked failures={}",
                    earliest_count,
                    max_score_count,
                    direct_miss_count,
                    optimized_target_count,
                    recovered_count,
                    bracketed_usize_list(&linked_failure_ids),
                ),
                final_role: final_role.into(),
            }
        })
        .collect()
}

fn build_group_validation(definitions: &[GroupDefinitionRecord]) -> Vec<GroupValidationRow> {
    definitions
        .iter()
        .map(|row| GroupValidationRow {
            group_id: group_display_name(&row.group_name).into(),
            group_members: row.member_features.clone(),
            failure_coactivation_count: row.failure_coactivation_run_count,
            pass_coactivation_count: row.pass_coactivation_run_count,
            retained_or_rejected: if row.validated {
                "retained".into()
            } else {
                "rejected".into()
            },
            reason: row
                .rejection_reason
                .clone()
                .unwrap_or_else(|| "validated by empirical failure-vs-pass gate".into()),
        })
        .collect()
}

fn linked_failure_ids_for_feature(
    feature_name: &str,
    optimized_dsa: &DsaEvaluation,
    missed_failure_diagnostics: &[MissedFailureDiagnosticRow],
) -> Vec<usize> {
    let mut linked = BTreeSet::new();
    for row in &optimized_dsa.per_failure_run_signals {
        if row.earliest_dsa_feature_name.as_deref() == Some(feature_name)
            || row.max_dsa_score_feature_name.as_deref() == Some(feature_name)
        {
            linked.insert(row.failure_run_index);
        }
    }
    for row in missed_failure_diagnostics {
        if row.nearest_feature_name.as_deref() == Some(feature_name)
            || row.optimized_feature_name.as_deref() == Some(feature_name)
        {
            linked.insert(row.failure_run_index);
        }
    }
    linked.into_iter().collect()
}

fn preferred_semantic_hit_count(
    semantic_layer: &SemanticLayer,
    feature_name: &str,
    failure_window_mask: &[bool],
    in_failure_window: bool,
    preferred_labels: &[&str],
) -> usize {
    if preferred_labels.is_empty() {
        return 0;
    }
    semantic_layer
        .semantic_matches
        .iter()
        .filter(|row| row.feature_name == feature_name)
        .filter(|row| failure_window_mask[row.run_index] == in_failure_window)
        .filter(|row| preferred_labels.contains(&row.heuristic_name.as_str()))
        .count()
}

fn preferred_grammar_hit_count(
    grammar_trace: &FeatureGrammarTrace,
    failure_window_mask: &[bool],
    in_failure_window: bool,
    preferred_states: &[&str],
) -> usize {
    if preferred_states.is_empty() {
        return 0;
    }
    grammar_trace
        .raw_states
        .iter()
        .enumerate()
        .filter(|(run_index, _)| failure_window_mask[*run_index] == in_failure_window)
        .filter(|(run_index, _)| {
            preferred_states.contains(&failure_grammar_state_label(grammar_trace, *run_index))
        })
        .count()
}

fn linked_window_grounding_counts(
    bundle: &FeatureBundle<'_>,
    linked_failure_ids: &[usize],
    pre_failure_lookback_runs: usize,
    preferred_grounded_motif_types: &[&str],
) -> (BTreeMap<String, usize>, usize) {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut preferred_window_count = 0usize;
    for &failure_index in linked_failure_ids {
        let start = failure_index.saturating_sub(pre_failure_lookback_runs);
        let grounded = grounded_motif_type_for_window(bundle, start, failure_index).to_string();
        if preferred_grounded_motif_types.contains(&grounded.as_str()) {
            preferred_window_count += 1;
        }
        *counts.entry(grounded).or_default() += 1;
    }
    (counts, preferred_window_count)
}

fn linked_window_grammar_counts(
    grammar_trace: &FeatureGrammarTrace,
    linked_failure_ids: &[usize],
    pre_failure_lookback_runs: usize,
    preferred_grammar_states: &[&str],
) -> (BTreeMap<String, usize>, usize) {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut preferred_window_count = 0usize;
    for &failure_index in linked_failure_ids {
        let start = failure_index.saturating_sub(pre_failure_lookback_runs);
        let dominant = dominant_grammar_state_in_window(grammar_trace, start, failure_index);
        if preferred_grammar_states.contains(&dominant.as_str()) {
            preferred_window_count += 1;
        }
        *counts.entry(dominant).or_default() += 1;
    }
    (counts, preferred_window_count)
}

fn summarize_count_map(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        "none".into()
    } else {
        counts
            .iter()
            .map(|(label, count)| format!("{label}:{count}"))
            .collect::<Vec<_>>()
            .join(";")
    }
}

fn bracketed_csv_list(values: &[String]) -> String {
    if values.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", values.join(","))
    }
}

fn bracketed_usize_list(values: &[usize]) -> String {
    let items = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    bracketed_csv_list(&items)
}

fn group_display_name(group_name: &str) -> &'static str {
    match group_name {
        "group_a" => "Group A",
        "group_b" => "Group B",
        "group_c" => "Group C",
        _ => "Unknown Group",
    }
}

fn classify_feature_behavior(drift: &[f64], slew: &[f64]) -> &'static str {
    let mean_abs_drift = mean_abs(drift);
    let mean_abs_slew = mean_abs(slew);
    if mean_abs_drift < 0.05 && mean_abs_slew < 0.05 {
        "stable"
    } else if mean_abs_slew > mean_abs_drift * 1.5 {
        "spiking"
    } else if drift.iter().any(|value| *value > 0.0) && drift.iter().any(|value| *value < 0.0) {
        "oscillatory"
    } else {
        "drifting"
    }
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
        let feature_name = match diagnostic.nearest_feature_name.as_deref() {
            Some("S134" | "S275") => diagnostic.nearest_feature_name.clone(),
            _ => diagnostic
                .optimized_feature_name
                .clone()
                .or_else(|| diagnostic.nearest_feature_name.clone()),
        };
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
                "Built from missed failure {} with nearest feature {:?}, operationalized feature {:?}, exact miss rule {}, bounded rescue recoverable={}.",
                diagnostic.failure_run_index,
                diagnostic.nearest_feature_name,
                diagnostic.optimized_feature_name,
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
                ranking_score_proxy: max_dsa_score
                    + 0.25 * motif_hits as f64
                    + 0.25 * pressure_hits as f64,
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
            row.feature_name == feature_name
                && row.run_index >= start
                && row.run_index < failure_index
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
                ScalarSupportMode::Threshold => bundle.residual.threshold_alarm
                    [start..failure_index]
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
    failure_cases: &[FailureCaseReport],
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
    for case in failure_cases {
        for feature in &case.top_contributing_features {
            features.insert(feature.feature_index);
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
        .filter(|row| {
            row.feature_name == feature_name
                && failure_window_mask[row.run_index] == in_failure_window
        })
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

fn dominant_grammar_state_for_mask(
    grammar_trace: &FeatureGrammarTrace,
    mask: &[bool],
    select_value: bool,
) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for run_index in 0..grammar_trace.raw_states.len() {
        if mask[run_index] != select_value {
            continue;
        }
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

fn grounded_motif_type(
    dominant_dsfb_motif: &str,
    dominant_grammar_state: &str,
    failure_local_recovery_case_count: usize,
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
    if (dominant_grammar_state == "Recovery" && failure_semantic_hits > 0)
        || (failure_local_recovery_case_count > 0 && failure_semantic_hits >= pass_semantic_hits)
    {
        return "recovery_pattern";
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
        _ if failure_pressure_hits > pass_pressure_hits
            && mean_abs_drift_failure > mean_abs_slew_failure =>
        {
            "slow_drift_precursor"
        }
        _ if dominant_grammar_state == "BoundaryGrazing" => "boundary_grazing_precursor",
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
        dominant_grammar_state_in_window(bundle.grammar, start, failure_index).as_str(),
        0,
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
    if states
        .iter()
        .any(|state| matches!(state, DsaPolicyState::Escalate))
    {
        "escalate"
    } else if states
        .iter()
        .any(|state| matches!(state, DsaPolicyState::Review))
    {
        "review"
    } else if states
        .iter()
        .any(|state| matches!(state, DsaPolicyState::Watch))
    {
        "watch"
    } else {
        "silent"
    }
}

fn failure_grammar_state_label(
    grammar_trace: &FeatureGrammarTrace,
    run_index: usize,
) -> &'static str {
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

fn episode_ranges(signal: &[bool]) -> Vec<(usize, usize)> {
    let mut episodes = Vec::new();
    let mut start = None::<usize>;
    for (run_index, flag) in signal.iter().copied().enumerate() {
        match (start, flag) {
            (None, true) => start = Some(run_index),
            (Some(episode_start), false) => {
                episodes.push((episode_start, run_index - 1));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(episode_start) = start {
        episodes.push((episode_start, signal.len().saturating_sub(1)));
    }
    episodes
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
        assert_eq!(
            grounded_motif_type("stable_admissible", "Admissible", 0, 0, 0, 0, 0, 0.0, 0.0),
            "null"
        );
        assert_eq!(
            grounded_motif_type(
                "recurrent_boundary_approach",
                "BoundaryGrazing",
                0,
                2,
                10,
                1,
                5,
                0.1,
                0.1
            ),
            "noise_like"
        );
        assert_eq!(
            grounded_motif_type(
                "persistent_instability_cluster",
                "Recovery",
                0,
                3,
                1,
                2,
                0,
                0.1,
                0.2
            ),
            "recovery_pattern"
        );
        assert_eq!(
            grounded_motif_type("stable_admissible", "Admissible", 1, 4, 1, 6, 0, 0.1, 0.2),
            "recovery_pattern"
        );
    }
}
