use crate::grammar::{FeatureGrammarTrace, GrammarReason, GrammarSet, GrammarState};
use crate::heuristics::{
    expanded_semantic_policy_definitions, heuristic_policy_definition, HeuristicAlertClass,
    PERSISTENT_INSTABILITY_CLUSTER, PRE_FAILURE_SLOW_DRIFT, RECURRENT_BOUNDARY_APPROACH,
    TRANSIENT_EXCURSION, TRANSITION_CLUSTER_SUPPORT, TRANSITION_EXCURSION,
    WATCH_ONLY_BOUNDARY_GRAZING,
};
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::{ResidualFeatureTrace, ResidualSet};
use crate::signs::{FeatureSigns, SignSet};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DsfbMotifClass {
    StableAdmissible,
    RecurrentBoundaryApproach,
    PreFailureSlowDrift,
    TransitionExcursion,
    PersistentInstabilityCluster,
    TransitionClusterSupport,
    WatchOnlyBoundaryGrazing,
}

impl DsfbMotifClass {
    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::StableAdmissible => "stable_admissible",
            Self::RecurrentBoundaryApproach => "recurrent_boundary_approach",
            Self::PreFailureSlowDrift => "pre_failure_slow_drift",
            Self::TransitionExcursion => "transition_excursion",
            Self::PersistentInstabilityCluster => "persistent_instability_cluster",
            Self::TransitionClusterSupport => "transition_cluster_support",
            Self::WatchOnlyBoundaryGrazing => "watch_only_boundary_grazing",
        }
    }

    pub fn definition(self) -> &'static str {
        match self {
            Self::StableAdmissible => "Low residual magnitude, low drift, and low slew inside the admissibility envelope.",
            Self::RecurrentBoundaryApproach => "Repeated boundary proximity with outward structural tendency and bounded fragmentation.",
            Self::PreFailureSlowDrift => "Persistent outward drift with moderate residual growth and limited abrupt slew.",
            Self::TransitionExcursion => "Elevated slew burst aligned with a grammar transition or violation onset.",
            Self::PersistentInstabilityCluster => "Repeated or sustained outward grammar pressure that is not reducible to isolated spikes.",
            Self::TransitionClusterSupport => "Corroborating burst or pressure feature aligned with a primary structural transition.",
            Self::WatchOnlyBoundaryGrazing => "Boundary proximity without sufficient persistence or corroboration for Review promotion.",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScaffoldGrammarState {
    Admissible,
    BoundaryGrazing,
    SustainedOutwardDrift,
    TransientViolation,
    PersistentViolation,
    Recovery,
}

impl ScaffoldGrammarState {
    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::Admissible => "admissible",
            Self::BoundaryGrazing => "boundary_grazing",
            Self::SustainedOutwardDrift => "sustained_outward_drift",
            Self::TransientViolation => "transient_violation",
            Self::PersistentViolation => "persistent_violation",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMotifTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub labels: Vec<DsfbMotifClass>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MotifSummaryRow {
    pub motif_label: String,
    pub definition: String,
    pub point_hits: usize,
    pub pre_failure_point_hits: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticMatchRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub grammar_state: String,
    pub grammar_reason: String,
    pub motif_label: String,
    pub heuristic_name: String,
    pub alert_class_default: String,
    pub grammar_constraints: String,
    pub regime_conditions: String,
    pub applicability_rules: String,
    pub feature_scope: String,
    pub ambiguity_note: String,
    pub rescue_eligibility_guidance: String,
    pub burden_contribution_class: String,
    pub structural_score_proxy: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuralDeltaMetrics {
    pub grammar_violation_precision: Option<f64>,
    pub motif_precision_pre_failure: Option<f64>,
    pub structural_separation_score: Option<f64>,
    pub precursor_stability_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MotifSet {
    pub traces: Vec<FeatureMotifTrace>,
    pub summary_rows: Vec<MotifSummaryRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticLayer {
    pub semantic_matches: Vec<SemanticMatchRecord>,
    pub ranked_candidates: Vec<SemanticMatchRecord>,
    pub structural_delta_metrics: StructuralDeltaMetrics,
}

#[derive(Debug, Clone)]
pub struct FeatureSemanticFlags {
    pub semantic_flags: BTreeMap<&'static str, Vec<bool>>,
    pub any_semantic_match: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureSignRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub normalized_residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub normalized_residual_norm: f64,
    pub sigma_norm: f64,
    pub is_imputed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureMotifTimelineRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub motif_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureGrammarStateRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub grammar_state: String,
    pub raw_state: String,
    pub confirmed_state: String,
    pub raw_reason: String,
    pub confirmed_reason: String,
    pub normalized_envelope_ratio: f64,
    pub persistent_boundary: bool,
    pub persistent_violation: bool,
    pub suppressed_by_imputation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvelopeInteractionSummaryRow {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub group_name: String,
    pub boundary_grazing_points: usize,
    pub sustained_outward_drift_points: usize,
    pub transient_violation_points: usize,
    pub persistent_violation_points: usize,
    pub recovery_points: usize,
    pub max_normalized_envelope_ratio: f64,
    pub mean_normalized_envelope_ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpandedHeuristicEntry {
    pub heuristic_name: String,
    pub motif_signature: String,
    pub allowed_grammar_states: String,
    pub role_class: String,
    pub feature_scope: String,
    pub interpretation_text: String,
    pub ambiguity_note: String,
    pub rescue_eligibility_guidance: String,
    pub burden_contribution_class: String,
    pub alert_class_default: String,
    pub requires_persistence: bool,
    pub requires_corroboration: bool,
    pub minimum_window: usize,
    pub minimum_hits: usize,
    pub maximum_allowed_fragmentation: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeaturePolicyDecisionRecord {
    pub feature_index: usize,
    pub feature_name: String,
    pub feature_role: String,
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub grammar_state: String,
    pub motif_label: String,
    pub semantic_label: Option<String>,
    pub policy_ceiling: String,
    pub policy_state: String,
    pub investigation_worthy: bool,
    pub corroborated: bool,
    pub corroborated_by: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupSignRecord {
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub active_feature_count: usize,
    pub normalized_residual_mean: f64,
    pub drift_mean: f64,
    pub slew_mean: f64,
    pub envelope_separation_mean: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupGrammarStateRecord {
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub active_feature_count: usize,
    pub grammar_state: String,
    pub boundary_member_count: usize,
    pub pressure_member_count: usize,
    pub violation_member_count: usize,
    pub envelope_separation_mean: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupSemanticMatchRecord {
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub grammar_state: String,
    pub heuristic_name: String,
    pub participating_features: String,
    pub structural_score_proxy: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupDefinitionRecord {
    pub group_name: String,
    pub member_features: String,
    pub member_roles: String,
    pub preferred_motifs: String,
    pub empirical_basis: String,
    pub group_size: usize,
    pub rescue_eligible_member_count: usize,
    pub highest_rescue_priority: String,
    pub semantic_match_count: usize,
    pub dominant_group_heuristic: Option<String>,
    pub pressure_run_count: usize,
    pub violation_run_count: usize,
    pub mean_active_feature_count: f64,
    pub mean_envelope_separation: f64,
    pub coactivation_member_threshold: usize,
    pub minimum_failure_coactivation_runs: usize,
    pub failure_coactivation_run_count: usize,
    pub pass_coactivation_run_count: usize,
    pub validated: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaffoldSemioticsArtifacts {
    pub feature_signs: Vec<FeatureSignRecord>,
    pub feature_motif_timeline: Vec<FeatureMotifTimelineRecord>,
    pub feature_grammar_states: Vec<FeatureGrammarStateRecord>,
    pub envelope_interaction_summary: Vec<EnvelopeInteractionSummaryRow>,
    pub heuristics_bank_expanded: Vec<ExpandedHeuristicEntry>,
    pub feature_policy_decisions: Vec<FeaturePolicyDecisionRecord>,
    pub group_definitions: Vec<GroupDefinitionRecord>,
    pub group_signs: Vec<GroupSignRecord>,
    pub group_grammar_states: Vec<GroupGrammarStateRecord>,
    pub group_semantic_matches: Vec<GroupSemanticMatchRecord>,
}

#[derive(Debug, Clone, Copy)]
struct FeatureScaffoldSpec {
    feature_name: &'static str,
    role: &'static str,
    preferred_motifs: &'static [&'static str],
    default_policy_ceiling: HeuristicAlertClass,
    rescue_eligible: bool,
    rescue_priority: &'static str,
    group_name: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct GroupScaffoldSpec {
    group_name: &'static str,
    members: &'static [&'static str],
}

const FEATURE_SCAFFOLD: &[FeatureScaffoldSpec] = &[
    FeatureScaffoldSpec {
        feature_name: "S059",
        role: "persistent_boundary_approach_precursor",
        preferred_motifs: &[RECURRENT_BOUNDARY_APPROACH, PRE_FAILURE_SLOW_DRIFT],
        default_policy_ceiling: HeuristicAlertClass::Review,
        rescue_eligible: true,
        rescue_priority: "high",
        group_name: "group_a",
    },
    FeatureScaffoldSpec {
        feature_name: "S133",
        role: "slow_structural_drift_precursor",
        preferred_motifs: &[PRE_FAILURE_SLOW_DRIFT],
        default_policy_ceiling: HeuristicAlertClass::Review,
        rescue_eligible: true,
        rescue_priority: "high",
        group_name: "group_a",
    },
    FeatureScaffoldSpec {
        feature_name: "S123",
        role: "transition_instability_feature",
        preferred_motifs: &[TRANSITION_EXCURSION, PERSISTENT_INSTABILITY_CLUSTER],
        default_policy_ceiling: HeuristicAlertClass::Escalate,
        rescue_eligible: true,
        rescue_priority: "medium",
        group_name: "group_b",
    },
    FeatureScaffoldSpec {
        feature_name: "S540",
        role: "burst_support_corroborator",
        preferred_motifs: &[RECURRENT_BOUNDARY_APPROACH, TRANSITION_CLUSTER_SUPPORT],
        default_policy_ceiling: HeuristicAlertClass::Review,
        rescue_eligible: false,
        rescue_priority: "low",
        group_name: "group_b",
    },
    FeatureScaffoldSpec {
        feature_name: "S128",
        role: "co_burst_corroborator",
        preferred_motifs: &[TRANSITION_CLUSTER_SUPPORT],
        default_policy_ceiling: HeuristicAlertClass::Review,
        rescue_eligible: false,
        rescue_priority: "low",
        group_name: "group_b",
    },
    FeatureScaffoldSpec {
        feature_name: "S104",
        role: "low_amplitude_sentinel",
        preferred_motifs: &[WATCH_ONLY_BOUNDARY_GRAZING],
        default_policy_ceiling: HeuristicAlertClass::Watch,
        rescue_eligible: false,
        rescue_priority: "none",
        group_name: "group_c",
    },
];

const GROUP_SCAFFOLD: &[GroupScaffoldSpec] = &[
    GroupScaffoldSpec {
        group_name: "group_a",
        members: &["S059", "S133"],
    },
    GroupScaffoldSpec {
        group_name: "group_b",
        members: &["S123", "S540", "S128"],
    },
    GroupScaffoldSpec {
        group_name: "group_c",
        members: &["S104"],
    },
];

const GROUP_FAILURE_COACTIVATION_MIN: usize = 2;
const GROUP_MEMBER_COACTIVATION_MIN: usize = 2;

pub fn classify_motifs(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    grammar: &GrammarSet,
    pre_failure_lookback_runs: usize,
) -> MotifSet {
    let failure_window_mask =
        failure_window_mask(dataset.labels.len(), &dataset.labels, pre_failure_lookback_runs);
    let mut traces = Vec::with_capacity(residuals.traces.len());
    let mut counts = BTreeMap::<DsfbMotifClass, (usize, usize)>::new();

    for (((residual_trace, sign_trace), grammar_trace), feature) in residuals
        .traces
        .iter()
        .zip(&signs.traces)
        .zip(&grammar.traces)
        .zip(&nominal.features)
    {
        let labels =
            classify_feature_motif_labels(residual_trace, sign_trace, grammar_trace, feature.rho);
        for (run_index, label) in labels.iter().copied().enumerate() {
            let entry = counts.entry(label).or_insert((0, 0));
            entry.0 += 1;
            if failure_window_mask[run_index] {
                entry.1 += 1;
            }
        }
        traces.push(FeatureMotifTrace {
            feature_index: residual_trace.feature_index,
            feature_name: residual_trace.feature_name.clone(),
            labels,
        });
    }

    let summary_rows = [
        DsfbMotifClass::StableAdmissible,
        DsfbMotifClass::RecurrentBoundaryApproach,
        DsfbMotifClass::PreFailureSlowDrift,
        DsfbMotifClass::TransitionExcursion,
        DsfbMotifClass::PersistentInstabilityCluster,
        DsfbMotifClass::TransitionClusterSupport,
        DsfbMotifClass::WatchOnlyBoundaryGrazing,
    ]
    .into_iter()
    .map(|label| {
        let (point_hits, pre_failure_point_hits) = counts.get(&label).copied().unwrap_or((0, 0));
        MotifSummaryRow {
            motif_label: label.as_lowercase().into(),
            definition: label.definition().into(),
            point_hits,
            pre_failure_point_hits,
        }
    })
    .collect();

    MotifSet {
        traces,
        summary_rows,
    }
}

pub fn build_semantic_layer(
    dataset: &PreparedDataset,
    residuals: &ResidualSet,
    signs: &SignSet,
    grammar: &GrammarSet,
    motifs: &MotifSet,
    nominal: &NominalModel,
    pre_failure_lookback_runs: usize,
) -> SemanticLayer {
    let failure_window_mask =
        failure_window_mask(dataset.labels.len(), &dataset.labels, pre_failure_lookback_runs);
    let mut semantic_matches = Vec::new();
    let mut ranked_candidates = Vec::new();

    for ((((residual_trace, sign_trace), grammar_trace), motif_trace), feature) in residuals
        .traces
        .iter()
        .zip(&signs.traces)
        .zip(&grammar.traces)
        .zip(&motifs.traces)
        .zip(&nominal.features)
    {
        let matches = build_feature_semantic_matches(
            dataset,
            residual_trace,
            sign_trace,
            grammar_trace,
            motif_trace,
            feature.rho,
        );
        ranked_candidates.extend(matches.iter().cloned());
        semantic_matches.extend(matches);
    }

    let structural_delta_metrics = compute_structural_delta_metrics(
        residuals,
        grammar,
        &semantic_matches,
        nominal,
        &failure_window_mask,
    );

    SemanticLayer {
        semantic_matches,
        ranked_candidates,
        structural_delta_metrics,
    }
}

pub fn feature_semantic_flags(
    residual_trace: &ResidualFeatureTrace,
    sign_trace: &FeatureSigns,
    grammar_trace: &FeatureGrammarTrace,
    feature_rho: f64,
) -> FeatureSemanticFlags {
    let mut semantic_flags = BTreeMap::<&'static str, Vec<bool>>::new();
    for heuristic_name in [
        PRE_FAILURE_SLOW_DRIFT,
        TRANSIENT_EXCURSION,
        RECURRENT_BOUNDARY_APPROACH,
    ] {
        semantic_flags.insert(heuristic_name, vec![false; residual_trace.norms.len()]);
    }

    let mut any_semantic_match = vec![false; residual_trace.norms.len()];
    for run_index in 0..residual_trace.norms.len() {
        let state = grammar_trace.raw_states[run_index];
        let reason = grammar_trace.raw_reasons[run_index];

        if state == GrammarState::Boundary && reason == GrammarReason::SustainedOutwardDrift {
            semantic_flags
                .get_mut(PRE_FAILURE_SLOW_DRIFT)
                .expect("pre_failure_slow_drift bucket")
                [run_index] = true;
            any_semantic_match[run_index] = true;
        }
        if matches!(state, GrammarState::Boundary | GrammarState::Violation)
            && reason == GrammarReason::AbruptSlewViolation
        {
            semantic_flags
                .get_mut(TRANSIENT_EXCURSION)
                .expect("transient_excursion bucket")
                [run_index] = true;
            any_semantic_match[run_index] = true;
        }
        if state == GrammarState::Boundary && reason == GrammarReason::RecurrentBoundaryGrazing {
            semantic_flags
                .get_mut(RECURRENT_BOUNDARY_APPROACH)
                .expect("recurrent_boundary_approach bucket")
                [run_index] = true;
            any_semantic_match[run_index] = true;
        }
    }

    FeatureSemanticFlags {
        semantic_flags,
        any_semantic_match,
    }
}

pub fn build_scaffold_semiotics(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    grammar: &GrammarSet,
    motifs: &MotifSet,
    semantic_layer: &SemanticLayer,
) -> ScaffoldSemioticsArtifacts {
    let selected = FEATURE_SCAFFOLD
        .iter()
        .filter_map(|spec| {
            let feature = nominal
                .features
                .iter()
                .find(|feature| feature.feature_name == spec.feature_name)?;
            let residual_trace = residuals
                .traces
                .iter()
                .find(|trace| trace.feature_index == feature.feature_index)?;
            let grammar_trace = grammar
                .traces
                .iter()
                .find(|trace| trace.feature_index == feature.feature_index)?;
            let motif_trace = motifs
                .traces
                .iter()
                .find(|trace| trace.feature_index == feature.feature_index)?;
            Some((spec, feature.rho, residual_trace, grammar_trace, motif_trace))
        })
        .collect::<Vec<_>>();

    let feature_signs = build_feature_sign_records(dataset, &selected);
    let feature_motif_timeline = build_feature_motif_timeline(dataset, &selected);
    let feature_grammar_states = build_feature_grammar_states(dataset, &selected);
    let envelope_interaction_summary = build_envelope_interaction_summary(&selected);
    let heuristics_bank_expanded = build_expanded_heuristics_bank();
    let candidate_group_signs = build_group_signs(dataset, &feature_signs);
    let candidate_group_grammar_states =
        build_group_grammar_states(dataset, &candidate_group_signs, &feature_grammar_states);
    let candidate_group_semantic_matches =
        build_group_semantic_matches(dataset, &candidate_group_grammar_states, semantic_layer);
    let group_definitions = build_group_definitions(
        &candidate_group_signs,
        &candidate_group_grammar_states,
        &candidate_group_semantic_matches,
    );
    let valid_group_names = group_definitions
        .iter()
        .filter(|row| row.validated)
        .map(|row| row.group_name.as_str())
        .collect::<BTreeSet<_>>();
    let group_signs = candidate_group_signs
        .into_iter()
        .filter(|row| valid_group_names.contains(row.group_name.as_str()))
        .collect::<Vec<_>>();
    let group_grammar_states = candidate_group_grammar_states
        .into_iter()
        .filter(|row| valid_group_names.contains(row.group_name.as_str()))
        .collect::<Vec<_>>();
    let group_semantic_matches = candidate_group_semantic_matches
        .into_iter()
        .filter(|row| valid_group_names.contains(row.group_name.as_str()))
        .collect::<Vec<_>>();
    let feature_policy_decisions = build_feature_policy_decisions(
        dataset,
        &selected,
        semantic_layer,
        &group_semantic_matches,
    );

    ScaffoldSemioticsArtifacts {
        feature_signs,
        feature_motif_timeline,
        feature_grammar_states,
        envelope_interaction_summary,
        heuristics_bank_expanded,
        feature_policy_decisions,
        group_definitions,
        group_signs,
        group_grammar_states,
        group_semantic_matches,
    }
}

fn classify_feature_motif_labels(
    residual_trace: &ResidualFeatureTrace,
    sign_trace: &FeatureSigns,
    grammar_trace: &FeatureGrammarTrace,
    feature_rho: f64,
) -> Vec<DsfbMotifClass> {
    let feature_spec = feature_scaffold_spec(&residual_trace.feature_name);
    let mut labels = Vec::with_capacity(residual_trace.norms.len());
    for run_index in 0..residual_trace.norms.len() {
        let grammar_label = classify_grammar_label(grammar_trace, run_index);
        let norm_ratio = residual_trace.norms[run_index] / feature_rho.max(1.0e-12);
        let drift_threshold = sign_trace.drift_threshold.max(1.0e-12);
        let slew_threshold = sign_trace.slew_threshold.max(1.0e-12);
        let drift_ratio = sign_trace.drift[run_index].abs() / drift_threshold;
        let slew_ratio = sign_trace.slew[run_index].abs() / slew_threshold;
        let recent_start = run_index.saturating_sub(4);
        let recent_non_admissible = (recent_start..=run_index)
            .filter(|&index| classify_grammar_label(grammar_trace, index) != ScaffoldGrammarState::Admissible)
            .count();
        let recent_pressure = (recent_start..=run_index)
            .filter(|&index| {
                matches!(
                    classify_grammar_label(grammar_trace, index),
                    ScaffoldGrammarState::BoundaryGrazing
                        | ScaffoldGrammarState::SustainedOutwardDrift
                        | ScaffoldGrammarState::TransientViolation
                        | ScaffoldGrammarState::PersistentViolation
                )
            })
            .count();
        let is_corroborator = feature_spec
            .map(|spec| {
                matches!(
                    spec.role,
                    "burst_support_corroborator" | "co_burst_corroborator"
                )
            })
            .unwrap_or(false);
        let is_sentinel = feature_spec
            .map(|spec| spec.role == "low_amplitude_sentinel")
            .unwrap_or(false);

        let label = if grammar_label == ScaffoldGrammarState::PersistentViolation
            || (recent_pressure >= 3
                && matches!(grammar_label, ScaffoldGrammarState::SustainedOutwardDrift))
        {
            if is_corroborator {
                DsfbMotifClass::TransitionClusterSupport
            } else {
                DsfbMotifClass::PersistentInstabilityCluster
            }
        } else if matches!(grammar_label, ScaffoldGrammarState::TransientViolation)
            && slew_ratio >= 1.0
        {
            if is_corroborator {
                DsfbMotifClass::TransitionClusterSupport
            } else {
                DsfbMotifClass::TransitionExcursion
            }
        } else if grammar_label == ScaffoldGrammarState::SustainedOutwardDrift
            && sign_trace.drift[run_index] >= sign_trace.drift_threshold
            && slew_ratio < 1.0
            && norm_ratio >= 0.40
        {
            DsfbMotifClass::PreFailureSlowDrift
        } else if grammar_label == ScaffoldGrammarState::BoundaryGrazing {
            if is_sentinel || recent_non_admissible < 2 {
                DsfbMotifClass::WatchOnlyBoundaryGrazing
            } else {
                DsfbMotifClass::RecurrentBoundaryApproach
            }
        } else if is_corroborator && recent_pressure >= 2 {
            DsfbMotifClass::TransitionClusterSupport
        } else if norm_ratio <= 0.25
            && drift_ratio < 1.0
            && slew_ratio < 1.0
            && matches!(
                grammar_label,
                ScaffoldGrammarState::Admissible | ScaffoldGrammarState::Recovery
            )
        {
            DsfbMotifClass::StableAdmissible
        } else if recent_non_admissible >= 2 {
            DsfbMotifClass::RecurrentBoundaryApproach
        } else {
            DsfbMotifClass::StableAdmissible
        };
        labels.push(label);
    }
    labels
}

fn build_feature_semantic_matches(
    dataset: &PreparedDataset,
    residual_trace: &ResidualFeatureTrace,
    sign_trace: &FeatureSigns,
    grammar_trace: &FeatureGrammarTrace,
    motif_trace: &FeatureMotifTrace,
    feature_rho: f64,
) -> Vec<SemanticMatchRecord> {
    let mut rows = Vec::new();
    let feature_role = feature_scaffold_spec(&residual_trace.feature_name)
        .map(|spec| spec.role)
        .unwrap_or("unscaffolded_feature");
    for run_index in 0..motif_trace.labels.len() {
        let grammar_label = classify_grammar_label(grammar_trace, run_index);
        let candidates = semantic_candidates_for_run(
            &residual_trace.feature_name,
            grammar_label,
            grammar_trace.raw_reasons[run_index],
            motif_trace.labels[run_index],
        );
        for (rank, heuristic_name) in candidates.into_iter().enumerate() {
            let Some(policy) = heuristic_policy_definition(heuristic_name) else {
                continue;
            };
            let metadata = expanded_heuristic_entry_metadata(heuristic_name);
            rows.push(SemanticMatchRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: feature_role.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                grammar_state: grammar_label.as_lowercase().into(),
                grammar_reason: format!("{:?}", grammar_trace.raw_reasons[run_index]),
                motif_label: motif_trace.labels[run_index].as_lowercase().into(),
                heuristic_name: heuristic_name.into(),
                alert_class_default: policy.alert_class_default.as_lowercase().into(),
                grammar_constraints: metadata.allowed_grammar_states.into(),
                regime_conditions: semantic_regime_conditions(heuristic_name).into(),
                applicability_rules: semantic_applicability_rules(heuristic_name).into(),
                feature_scope: metadata.feature_scope.into(),
                ambiguity_note: metadata.ambiguity_note.into(),
                rescue_eligibility_guidance: metadata.rescue_eligibility_guidance.into(),
                burden_contribution_class: metadata.burden_contribution_class.into(),
                structural_score_proxy: sign_trace.drift[run_index].abs()
                    + sign_trace.slew[run_index].abs()
                    + (residual_trace.norms[run_index] / feature_rho.max(1.0e-12)),
                rank: rank + 1,
            });
        }
    }
    rows
}

fn semantic_candidates_for_run(
    feature_name: &str,
    grammar_state: ScaffoldGrammarState,
    grammar_reason: GrammarReason,
    motif_label: DsfbMotifClass,
) -> Vec<&'static str> {
    let mut candidates = Vec::new();
    let feature_spec = feature_scaffold_spec(feature_name);
    let is_corroborator = feature_spec
        .map(|spec| {
            matches!(
                spec.role,
                "burst_support_corroborator" | "co_burst_corroborator"
            )
        })
        .unwrap_or(false);

    if grammar_state == ScaffoldGrammarState::SustainedOutwardDrift
        && motif_label == DsfbMotifClass::PreFailureSlowDrift
    {
        candidates.push(PRE_FAILURE_SLOW_DRIFT);
    }
    if matches!(
        grammar_state,
        ScaffoldGrammarState::TransientViolation | ScaffoldGrammarState::PersistentViolation
    ) && motif_label == DsfbMotifClass::TransitionExcursion
    {
        candidates.push(TRANSITION_EXCURSION);
    }
    if grammar_state == ScaffoldGrammarState::BoundaryGrazing
        && motif_label == DsfbMotifClass::RecurrentBoundaryApproach
    {
        candidates.push(RECURRENT_BOUNDARY_APPROACH);
    }
    if matches!(
        grammar_state,
        ScaffoldGrammarState::SustainedOutwardDrift | ScaffoldGrammarState::PersistentViolation
    ) && motif_label == DsfbMotifClass::PersistentInstabilityCluster
    {
        candidates.push(PERSISTENT_INSTABILITY_CLUSTER);
    }
    if is_corroborator
        && matches!(
            grammar_state,
            ScaffoldGrammarState::BoundaryGrazing
                | ScaffoldGrammarState::SustainedOutwardDrift
                | ScaffoldGrammarState::TransientViolation
        )
        && motif_label == DsfbMotifClass::TransitionClusterSupport
    {
        candidates.push(TRANSITION_CLUSTER_SUPPORT);
    }
    if grammar_state == ScaffoldGrammarState::BoundaryGrazing
        && motif_label == DsfbMotifClass::WatchOnlyBoundaryGrazing
    {
        candidates.push(WATCH_ONLY_BOUNDARY_GRAZING);
    }

    if grammar_reason == GrammarReason::AbruptSlewViolation
        && candidates.is_empty()
        && !is_corroborator
    {
        candidates.push(TRANSITION_EXCURSION);
    }

    candidates
}

fn semantic_flag_aliases(heuristic_name: &'static str) -> &'static [&'static str] {
    match heuristic_name {
        TRANSITION_EXCURSION => &[TRANSIENT_EXCURSION],
        PRE_FAILURE_SLOW_DRIFT => &[PRE_FAILURE_SLOW_DRIFT],
        RECURRENT_BOUNDARY_APPROACH => &[RECURRENT_BOUNDARY_APPROACH],
        _ => &[],
    }
}

fn build_feature_sign_records(
    dataset: &PreparedDataset,
    selected: &[(
        &FeatureScaffoldSpec,
        f64,
        &ResidualFeatureTrace,
        &FeatureGrammarTrace,
        &FeatureMotifTrace,
    )],
) -> Vec<FeatureSignRecord> {
    let mut rows = Vec::new();
    for (spec, rho, residual_trace, _, _) in selected {
        let mut normalized_residual = Vec::with_capacity(residual_trace.residuals.len());
        let mut drift = vec![0.0; residual_trace.residuals.len()];
        let mut slew = vec![0.0; residual_trace.residuals.len()];
        for run_index in 0..residual_trace.residuals.len() {
            normalized_residual.push(residual_trace.residuals[run_index] / rho.max(1.0e-12));
        }
        for run_index in 1..normalized_residual.len() {
            if residual_trace.is_imputed[run_index] || residual_trace.is_imputed[run_index - 1] {
                drift[run_index] = 0.0;
            } else {
                drift[run_index] = normalized_residual[run_index] - normalized_residual[run_index - 1];
            }
        }
        for run_index in 1..drift.len() {
            if residual_trace.is_imputed[run_index] || residual_trace.is_imputed[run_index - 1] {
                slew[run_index] = 0.0;
            } else {
                slew[run_index] = drift[run_index] - drift[run_index - 1];
            }
        }
        for run_index in 0..normalized_residual.len() {
            rows.push(FeatureSignRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: spec.role.into(),
                group_name: spec.group_name.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                normalized_residual: normalized_residual[run_index],
                drift: drift[run_index],
                slew: slew[run_index],
                normalized_residual_norm: residual_trace.norms[run_index] / rho.max(1.0e-12),
                sigma_norm: (
                    normalized_residual[run_index] * normalized_residual[run_index]
                        + drift[run_index] * drift[run_index]
                        + slew[run_index] * slew[run_index]
                )
                .sqrt(),
                is_imputed: residual_trace.is_imputed[run_index],
            });
        }
    }
    rows
}

fn build_feature_motif_timeline(
    dataset: &PreparedDataset,
    selected: &[(
        &FeatureScaffoldSpec,
        f64,
        &ResidualFeatureTrace,
        &FeatureGrammarTrace,
        &FeatureMotifTrace,
    )],
) -> Vec<FeatureMotifTimelineRecord> {
    let mut rows = Vec::new();
    for (spec, _, residual_trace, _, motif_trace) in selected {
        for (run_index, motif_label) in motif_trace.labels.iter().enumerate() {
            rows.push(FeatureMotifTimelineRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: spec.role.into(),
                group_name: spec.group_name.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                motif_label: motif_label.as_lowercase().into(),
            });
        }
    }
    rows
}

fn build_feature_grammar_states(
    dataset: &PreparedDataset,
    selected: &[(
        &FeatureScaffoldSpec,
        f64,
        &ResidualFeatureTrace,
        &FeatureGrammarTrace,
        &FeatureMotifTrace,
    )],
) -> Vec<FeatureGrammarStateRecord> {
    let mut rows = Vec::new();
    for (spec, rho, residual_trace, grammar_trace, _) in selected {
        for run_index in 0..grammar_trace.raw_states.len() {
            rows.push(FeatureGrammarStateRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: spec.role.into(),
                group_name: spec.group_name.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                grammar_state: classify_grammar_label(grammar_trace, run_index)
                    .as_lowercase()
                    .into(),
                raw_state: format!("{:?}", grammar_trace.raw_states[run_index]),
                confirmed_state: format!("{:?}", grammar_trace.states[run_index]),
                raw_reason: format!("{:?}", grammar_trace.raw_reasons[run_index]),
                confirmed_reason: format!("{:?}", grammar_trace.reasons[run_index]),
                normalized_envelope_ratio: residual_trace.norms[run_index] / rho.max(1.0e-12),
                persistent_boundary: grammar_trace.persistent_boundary[run_index],
                persistent_violation: grammar_trace.persistent_violation[run_index],
                suppressed_by_imputation: grammar_trace.suppressed_by_imputation[run_index],
            });
        }
    }
    rows
}

fn build_envelope_interaction_summary(
    selected: &[(
        &FeatureScaffoldSpec,
        f64,
        &ResidualFeatureTrace,
        &FeatureGrammarTrace,
        &FeatureMotifTrace,
    )],
) -> Vec<EnvelopeInteractionSummaryRow> {
    selected
        .iter()
        .map(|(spec, rho, residual_trace, grammar_trace, _)| {
            let mut boundary_grazing_points = 0usize;
            let mut sustained_outward_drift_points = 0usize;
            let mut transient_violation_points = 0usize;
            let mut persistent_violation_points = 0usize;
            let mut recovery_points = 0usize;
            let mut max_ratio = 0.0_f64;
            let mut ratios = Vec::with_capacity(residual_trace.norms.len());
            for run_index in 0..residual_trace.norms.len() {
                let grammar_label = classify_grammar_label(grammar_trace, run_index);
                match grammar_label {
                    ScaffoldGrammarState::BoundaryGrazing => boundary_grazing_points += 1,
                    ScaffoldGrammarState::SustainedOutwardDrift => {
                        sustained_outward_drift_points += 1
                    }
                    ScaffoldGrammarState::TransientViolation => transient_violation_points += 1,
                    ScaffoldGrammarState::PersistentViolation => persistent_violation_points += 1,
                    ScaffoldGrammarState::Recovery => recovery_points += 1,
                    ScaffoldGrammarState::Admissible => {}
                }
                let ratio = residual_trace.norms[run_index] / rho.max(1.0e-12);
                max_ratio = max_ratio.max(ratio);
                ratios.push(ratio);
            }
            EnvelopeInteractionSummaryRow {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: spec.role.into(),
                group_name: spec.group_name.into(),
                boundary_grazing_points,
                sustained_outward_drift_points,
                transient_violation_points,
                persistent_violation_points,
                recovery_points,
                max_normalized_envelope_ratio: max_ratio,
                mean_normalized_envelope_ratio: mean(&ratios).unwrap_or(0.0),
            }
        })
        .collect()
}

fn build_expanded_heuristics_bank() -> Vec<ExpandedHeuristicEntry> {
    expanded_semantic_policy_definitions()
        .into_iter()
        .map(|definition| {
            let metadata = expanded_heuristic_entry_metadata(definition.motif_name);
            ExpandedHeuristicEntry {
                heuristic_name: definition.motif_name.into(),
                motif_signature: definition.signature_definition.into(),
                allowed_grammar_states: metadata.allowed_grammar_states.into(),
                role_class: metadata.role_class.into(),
                feature_scope: metadata.feature_scope.into(),
                interpretation_text: definition.interpretation.into(),
                ambiguity_note: metadata.ambiguity_note.into(),
                rescue_eligibility_guidance: metadata.rescue_eligibility_guidance.into(),
                burden_contribution_class: metadata.burden_contribution_class.into(),
                alert_class_default: definition.alert_class_default.as_lowercase().into(),
                requires_persistence: definition.requires_persistence,
                requires_corroboration: definition.requires_corroboration,
                minimum_window: definition.minimum_window,
                minimum_hits: definition.minimum_hits,
                maximum_allowed_fragmentation: definition.maximum_allowed_fragmentation(),
            }
        })
        .collect()
}

fn build_group_signs(
    dataset: &PreparedDataset,
    feature_signs: &[FeatureSignRecord],
) -> Vec<GroupSignRecord> {
    let by_key = feature_signs.iter().fold(
        BTreeMap::<(&str, usize), Vec<&FeatureSignRecord>>::new(),
        |mut acc, row| {
            acc.entry((row.group_name.as_str(), row.run_index))
                .or_default()
                .push(row);
            acc
        },
    );
    let mut rows: Vec<GroupSignRecord> = Vec::new();
    for group in GROUP_SCAFFOLD {
        for run_index in 0..dataset.labels.len() {
            let members = by_key
                .get(&(group.group_name, run_index))
                .cloned()
                .unwrap_or_default();
            let active_feature_count = members.len();
            let normalized_residual_mean = mean_of_records(&members, |row| row.normalized_residual);
            let drift_mean = mean_of_records(&members, |row| row.drift);
            let slew_mean = mean_of_records(&members, |row| row.slew);
            let envelope_separation_mean =
                mean_of_records(&members, |row| row.normalized_residual_norm);
            rows.push(GroupSignRecord {
                group_name: group.group_name.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                active_feature_count,
                normalized_residual_mean,
                drift_mean,
                slew_mean,
                envelope_separation_mean,
            });
        }
    }
    rows
}

fn build_group_grammar_states(
    dataset: &PreparedDataset,
    group_signs: &[GroupSignRecord],
    feature_grammar_states: &[FeatureGrammarStateRecord],
) -> Vec<GroupGrammarStateRecord> {
    let feature_by_key = feature_grammar_states.iter().fold(
        BTreeMap::<(&str, usize), Vec<&FeatureGrammarStateRecord>>::new(),
        |mut acc, row| {
            acc.entry((row.group_name.as_str(), row.run_index))
                .or_default()
                .push(row);
            acc
        },
    );
    let mut rows: Vec<GroupGrammarStateRecord> = Vec::new();
    for sign_row in group_signs {
        let members = feature_by_key
            .get(&(sign_row.group_name.as_str(), sign_row.run_index))
            .cloned()
            .unwrap_or_default();
        let boundary_member_count = members
            .iter()
            .filter(|row| row.grammar_state == ScaffoldGrammarState::BoundaryGrazing.as_lowercase())
            .count();
        let pressure_member_count = members
            .iter()
            .filter(|row| {
                matches!(
                    row.grammar_state.as_str(),
                    "boundary_grazing"
                        | "sustained_outward_drift"
                        | "transient_violation"
                        | "persistent_violation"
                )
            })
            .count();
        let violation_member_count = members
            .iter()
            .filter(|row| {
                matches!(
                    row.grammar_state.as_str(),
                    "transient_violation" | "persistent_violation"
                )
            })
            .count();
        let previous_state = sign_row
            .run_index
            .checked_sub(1)
            .and_then(|previous| {
                rows.iter()
                    .rev()
                    .find(|row| row.group_name == sign_row.group_name && row.run_index == previous)
                    .map(|row| row.grammar_state.as_str())
            });
        let grammar_state = if sign_row.active_feature_count == 0 {
            ScaffoldGrammarState::Admissible
        } else if sign_row.envelope_separation_mean >= 1.0 && violation_member_count >= 2 {
            ScaffoldGrammarState::PersistentViolation
        } else if sign_row.envelope_separation_mean >= 1.0 && violation_member_count >= 1 {
            ScaffoldGrammarState::TransientViolation
        } else if sign_row.envelope_separation_mean >= 0.6 && sign_row.drift_mean > 0.0 {
            ScaffoldGrammarState::SustainedOutwardDrift
        } else if sign_row.envelope_separation_mean >= 0.6 && boundary_member_count >= 1 {
            ScaffoldGrammarState::BoundaryGrazing
        } else if previous_state.is_some_and(|state| {
            matches!(
                state,
                "boundary_grazing"
                    | "sustained_outward_drift"
                    | "transient_violation"
                    | "persistent_violation"
            )
        }) && sign_row.envelope_separation_mean < 0.4
        {
            ScaffoldGrammarState::Recovery
        } else {
            ScaffoldGrammarState::Admissible
        };
        rows.push(GroupGrammarStateRecord {
            group_name: sign_row.group_name.clone(),
            run_index: sign_row.run_index,
            timestamp: dataset.timestamps[sign_row.run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            label: dataset.labels[sign_row.run_index],
            active_feature_count: sign_row.active_feature_count,
            grammar_state: grammar_state.as_lowercase().into(),
            boundary_member_count,
            pressure_member_count,
            violation_member_count,
            envelope_separation_mean: sign_row.envelope_separation_mean,
        });
    }
    rows
}

fn build_group_semantic_matches(
    dataset: &PreparedDataset,
    group_grammar_states: &[GroupGrammarStateRecord],
    semantic_layer: &SemanticLayer,
) -> Vec<GroupSemanticMatchRecord> {
    let feature_matches = semantic_layer.semantic_matches.iter().fold(
        BTreeMap::<(&str, usize), Vec<&SemanticMatchRecord>>::new(),
        |mut acc, row| {
            if let Some(spec) = feature_scaffold_spec(&row.feature_name) {
                acc.entry((spec.group_name, row.run_index)).or_default().push(row);
            }
            acc
        },
    );

    let mut rows = Vec::new();
    for group_row in group_grammar_states {
        let Some(group_spec) = group_scaffold_spec(&group_row.group_name) else {
            continue;
        };
        let matches = feature_matches
            .get(&(group_row.group_name.as_str(), group_row.run_index))
            .cloned()
            .unwrap_or_default();
        let mut candidates = Vec::<(&'static str, Vec<String>, f64)>::new();
        let participating_features = matches
            .iter()
            .map(|row| row.feature_name.clone())
            .collect::<Vec<_>>();
        let score = matches
            .iter()
            .map(|row| row.structural_score_proxy)
            .fold(0.0, f64::max);
        if group_row.group_name == "group_a" {
            if matches.iter().any(|row| row.heuristic_name == PRE_FAILURE_SLOW_DRIFT)
                && matches!(
                    group_row.grammar_state.as_str(),
                    "sustained_outward_drift" | "persistent_violation"
                )
            {
                candidates.push((PRE_FAILURE_SLOW_DRIFT, participating_features.clone(), score));
            }
            if matches
                .iter()
                .any(|row| row.heuristic_name == RECURRENT_BOUNDARY_APPROACH)
                && matches!(
                    group_row.grammar_state.as_str(),
                    "boundary_grazing" | "sustained_outward_drift"
                )
            {
                candidates.push((
                    RECURRENT_BOUNDARY_APPROACH,
                    participating_features.clone(),
                    score,
                ));
            }
        } else if group_row.group_name == "group_b" {
            if matches.iter().any(|row| row.heuristic_name == TRANSITION_EXCURSION)
                && matches!(
                    group_row.grammar_state.as_str(),
                    "transient_violation" | "persistent_violation"
                )
            {
                candidates.push((
                    PERSISTENT_INSTABILITY_CLUSTER,
                    participating_features.clone(),
                    score,
                ));
            }
            if matches
                .iter()
                .any(|row| row.heuristic_name == TRANSITION_CLUSTER_SUPPORT)
            {
                candidates.push((
                    TRANSITION_CLUSTER_SUPPORT,
                    participating_features.clone(),
                    score,
                ));
            }
        } else if group_row.group_name == "group_c"
            && group_row.grammar_state == ScaffoldGrammarState::BoundaryGrazing.as_lowercase()
            && matches
                .iter()
                .any(|row| row.heuristic_name == WATCH_ONLY_BOUNDARY_GRAZING)
        {
            candidates.push((
                WATCH_ONLY_BOUNDARY_GRAZING,
                participating_features.clone(),
                score,
            ));
        }
        for (rank, (heuristic_name, participating, structural_score_proxy)) in
            candidates.into_iter().enumerate()
        {
            rows.push(GroupSemanticMatchRecord {
                group_name: group_spec.group_name.into(),
                run_index: group_row.run_index,
                timestamp: dataset.timestamps[group_row.run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[group_row.run_index],
                grammar_state: group_row.grammar_state.clone(),
                heuristic_name: heuristic_name.into(),
                participating_features: participating.join(","),
                structural_score_proxy,
                rank: rank + 1,
            });
        }
    }
    rows
}

fn build_group_definitions(
    group_signs: &[GroupSignRecord],
    group_grammar_states: &[GroupGrammarStateRecord],
    group_semantic_matches: &[GroupSemanticMatchRecord],
) -> Vec<GroupDefinitionRecord> {
    let group_signs_by_name = group_signs.iter().fold(
        BTreeMap::<&str, Vec<&GroupSignRecord>>::new(),
        |mut acc, row| {
            acc.entry(row.group_name.as_str()).or_default().push(row);
            acc
        },
    );
    let group_grammar_by_name = group_grammar_states.iter().fold(
        BTreeMap::<&str, Vec<&GroupGrammarStateRecord>>::new(),
        |mut acc, row| {
            acc.entry(row.group_name.as_str()).or_default().push(row);
            acc
        },
    );
    let group_semantics_by_name = group_semantic_matches.iter().fold(
        BTreeMap::<&str, Vec<&GroupSemanticMatchRecord>>::new(),
        |mut acc, row| {
            acc.entry(row.group_name.as_str()).or_default().push(row);
            acc
        },
    );

    GROUP_SCAFFOLD
        .iter()
        .map(|group| {
            let signs = group_signs_by_name
                .get(group.group_name)
                .cloned()
                .unwrap_or_default();
            let grammar_rows = group_grammar_by_name
                .get(group.group_name)
                .cloned()
                .unwrap_or_default();
            let semantic_rows = group_semantics_by_name
                .get(group.group_name)
                .cloned()
                .unwrap_or_default();
            let members = group
                .members
                .iter()
                .filter_map(|feature_name| feature_scaffold_spec(feature_name))
                .collect::<Vec<_>>();
            let preferred_motifs = members
                .iter()
                .flat_map(|spec| spec.preferred_motifs.iter().copied())
                .collect::<Vec<_>>();
            let mut preferred_motifs = preferred_motifs;
            preferred_motifs.sort_unstable();
            preferred_motifs.dedup();
            let mut heuristic_counts = BTreeMap::<String, usize>::new();
            for row in &semantic_rows {
                *heuristic_counts
                    .entry(row.heuristic_name.clone())
                    .or_default() += 1;
            }
            let dominant_group_heuristic = heuristic_counts
                .into_iter()
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                .map(|(name, _)| name);
            let highest_rescue_priority = members
                .iter()
                .map(|spec| spec.rescue_priority)
                .max_by_key(|priority| rescue_priority_rank(priority))
                .unwrap_or("none");
            let failure_coactivation_run_count = grammar_rows
                .iter()
                .filter(|row| {
                    row.label == 1
                        && row.pressure_member_count >= GROUP_MEMBER_COACTIVATION_MIN
                })
                .count();
            let pass_coactivation_run_count = grammar_rows
                .iter()
                .filter(|row| {
                    row.label == -1
                        && row.pressure_member_count >= GROUP_MEMBER_COACTIVATION_MIN
                })
                .count();
            let validated = group.group_name != "group_c"
                && group.members.len() >= GROUP_MEMBER_COACTIVATION_MIN
                && failure_coactivation_run_count >= GROUP_FAILURE_COACTIVATION_MIN
                && pass_coactivation_run_count == 0;
            let rejection_reason = if validated {
                None
            } else if group.members.len() < GROUP_MEMBER_COACTIVATION_MIN {
                Some("group size is below the co-activation threshold".into())
            } else if failure_coactivation_run_count < GROUP_FAILURE_COACTIVATION_MIN {
                Some("failure co-activation is below the required minimum".into())
            } else if pass_coactivation_run_count > 0 {
                Some("group co-activates in pass runs and is rejected".into())
            } else {
                Some("group failed strict grouped-semiotics validation".into())
            };

            GroupDefinitionRecord {
                group_name: group.group_name.into(),
                member_features: group.members.join(","),
                member_roles: members
                    .iter()
                    .map(|spec| spec.role)
                    .collect::<Vec<_>>()
                    .join(","),
                preferred_motifs: preferred_motifs.join(","),
                empirical_basis: format!(
                    "Scaffolded from saved top-feature co-activity; {} grouped semantic matches, {} pressure runs, {} violation runs.",
                    semantic_rows.len(),
                    grammar_rows
                        .iter()
                        .filter(|row| {
                            matches!(
                                row.grammar_state.as_str(),
                                "boundary_grazing"
                                    | "sustained_outward_drift"
                                    | "transient_violation"
                                    | "persistent_violation"
                            )
                        })
                        .count(),
                    grammar_rows
                        .iter()
                        .filter(|row| {
                            matches!(
                                row.grammar_state.as_str(),
                                "transient_violation" | "persistent_violation"
                            )
                        })
                        .count()
                ),
                group_size: group.members.len(),
                rescue_eligible_member_count: members
                    .iter()
                    .filter(|spec| spec.rescue_eligible)
                    .count(),
                highest_rescue_priority: highest_rescue_priority.into(),
                semantic_match_count: semantic_rows.len(),
                dominant_group_heuristic,
                pressure_run_count: grammar_rows
                    .iter()
                    .filter(|row| {
                        matches!(
                            row.grammar_state.as_str(),
                            "boundary_grazing"
                                | "sustained_outward_drift"
                                | "transient_violation"
                                | "persistent_violation"
                        )
                    })
                    .count(),
                violation_run_count: grammar_rows
                    .iter()
                    .filter(|row| {
                        matches!(
                            row.grammar_state.as_str(),
                            "transient_violation" | "persistent_violation"
                        )
                    })
                    .count(),
                mean_active_feature_count: mean_of_records(&signs, |row| row.active_feature_count as f64),
                mean_envelope_separation: mean_of_records(&signs, |row| row.envelope_separation_mean),
                coactivation_member_threshold: GROUP_MEMBER_COACTIVATION_MIN,
                minimum_failure_coactivation_runs: GROUP_FAILURE_COACTIVATION_MIN,
                failure_coactivation_run_count,
                pass_coactivation_run_count,
                validated,
                rejection_reason,
            }
        })
        .collect()
}

fn build_feature_policy_decisions(
    dataset: &PreparedDataset,
    selected: &[(
        &FeatureScaffoldSpec,
        f64,
        &ResidualFeatureTrace,
        &FeatureGrammarTrace,
        &FeatureMotifTrace,
    )],
    semantic_layer: &SemanticLayer,
    group_semantic_matches: &[GroupSemanticMatchRecord],
) -> Vec<FeaturePolicyDecisionRecord> {
    let semantic_by_feature_run = semantic_layer
        .ranked_candidates
        .iter()
        .filter_map(|row| {
            feature_scaffold_spec(&row.feature_name)?;
            Some(((row.feature_name.clone(), row.run_index), row.clone()))
        })
        .fold(
            BTreeMap::<(String, usize), SemanticMatchRecord>::new(),
            |mut acc, (key, row)| {
                acc.entry(key)
                    .and_modify(|existing| {
                        if row.rank < existing.rank {
                            *existing = row.clone();
                        }
                    })
                    .or_insert(row);
                acc
            },
        );
    let group_support = group_semantic_matches.iter().fold(
        BTreeMap::<(&str, usize), Vec<&GroupSemanticMatchRecord>>::new(),
        |mut acc, row| {
            acc.entry((row.group_name.as_str(), row.run_index))
                .or_default()
                .push(row);
            acc
        },
    );

    let mut rows = Vec::new();
    let mut base_states = BTreeMap::<(String, usize), String>::new();

    for (spec, _, residual_trace, grammar_trace, motif_trace) in selected {
        for run_index in 0..residual_trace.norms.len() {
            let semantic = semantic_by_feature_run
                .get(&(residual_trace.feature_name.clone(), run_index))
                .cloned();
            let grammar_state = classify_grammar_label(grammar_trace, run_index);
            let motif_label = motif_trace.labels[run_index];
            let group_matches = group_support
                .get(&(spec.group_name, run_index))
                .cloned()
                .unwrap_or_default();
            let mut corroborators = group_matches
                .iter()
                .flat_map(|row| row.participating_features.split(','))
                .map(str::trim)
                .filter(|name| !name.is_empty() && *name != residual_trace.feature_name)
                .map(str::to_string)
                .collect::<Vec<_>>();
            corroborators.sort();
            corroborators.dedup();

            let (policy_state, rationale) = base_policy_state(
                spec,
                grammar_state,
                motif_label,
                semantic.as_ref(),
                !corroborators.is_empty(),
            );
            base_states.insert(
                (residual_trace.feature_name.clone(), run_index),
                policy_state.to_string(),
            );
            rows.push(FeaturePolicyDecisionRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                feature_role: spec.role.into(),
                group_name: spec.group_name.into(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                grammar_state: grammar_state.as_lowercase().into(),
                motif_label: motif_label.as_lowercase().into(),
                semantic_label: semantic.as_ref().map(|row| row.heuristic_name.clone()),
                policy_ceiling: spec.default_policy_ceiling.as_lowercase().into(),
                policy_state: policy_state.into(),
                investigation_worthy: matches!(policy_state, "review" | "escalate"),
                corroborated: !corroborators.is_empty(),
                corroborated_by: corroborators.join(","),
                rationale,
            });
        }
    }

    for row in &mut rows {
        let related_support = related_feature_support(
            &base_states,
            &row.feature_name,
            row.run_index,
            row.corroborated,
        );
        if row.feature_name == "S059"
            && row.policy_state == "review"
            && (related_support || row.grammar_state == ScaffoldGrammarState::PersistentViolation.as_lowercase())
        {
            row.policy_state = "escalate".into();
            row.investigation_worthy = true;
            row.rationale = "primary precursor escalated only after corroboration or persistent violation".into();
        } else if row.feature_name == "S133"
            && row.policy_state == "review"
            && related_support
        {
            row.policy_state = "escalate".into();
            row.investigation_worthy = true;
            row.rationale = "slow structural drift escalated only with corroboration from S059 or S123".into();
        } else if row.feature_name == "S123"
            && row.policy_state == "review"
            && (related_support
                || row.grammar_state == ScaffoldGrammarState::PersistentViolation.as_lowercase())
        {
            row.policy_state = "escalate".into();
            row.investigation_worthy = true;
            row.rationale = "transition instability escalated under grammar transition support or persistent violation".into();
        } else if matches!(row.feature_name.as_str(), "S540" | "S128")
            && row.policy_state == "escalate"
        {
            row.policy_state = "review".into();
            row.investigation_worthy = true;
            row.rationale = "corroborator never escalates alone in the scaffold policy".into();
        } else if row.feature_name == "S104" && row.policy_state != "watch" {
            row.policy_state = "watch".into();
            row.investigation_worthy = false;
            row.rationale = "sentinel remains watch-only by scaffold design".into();
        }
    }

    rows
}

fn base_policy_state(
    spec: &FeatureScaffoldSpec,
    grammar_state: ScaffoldGrammarState,
    motif_label: DsfbMotifClass,
    semantic: Option<&SemanticMatchRecord>,
    corroborated: bool,
) -> (&'static str, String) {
    match spec.feature_name {
        "S059" => {
            if semantic
                .as_ref()
                .is_some_and(|row| row.heuristic_name == RECURRENT_BOUNDARY_APPROACH)
                && grammar_state == ScaffoldGrammarState::BoundaryGrazing
            {
                ("review", "S059 recurrent boundary approach promoted to Review under boundary grazing".into())
            } else if semantic
                .as_ref()
                .is_some_and(|row| row.heuristic_name == PRE_FAILURE_SLOW_DRIFT)
                && matches!(
                    grammar_state,
                    ScaffoldGrammarState::SustainedOutwardDrift
                        | ScaffoldGrammarState::PersistentViolation
                )
            {
                (
                    if corroborated { "review" } else { "watch" },
                    "S059 slow drift retained unless grammar and corroboration support promotion".into(),
                )
            } else {
                ("silent", "S059 remained below scaffold promotion conditions".into())
            }
        }
        "S133" => {
            if semantic
                .as_ref()
                .is_some_and(|row| row.heuristic_name == PRE_FAILURE_SLOW_DRIFT)
                && grammar_state == ScaffoldGrammarState::SustainedOutwardDrift
            {
                ("review", "S133 slow structural drift promoted to Review under sustained outward drift".into())
            } else {
                ("silent", "S133 remained below scaffold promotion conditions".into())
            }
        }
        "S123" => {
            if semantic
                .as_ref()
                .is_some_and(|row| row.heuristic_name == TRANSITION_EXCURSION)
                && matches!(
                    grammar_state,
                    ScaffoldGrammarState::TransientViolation
                        | ScaffoldGrammarState::PersistentViolation
                )
            {
                (
                    if grammar_state == ScaffoldGrammarState::PersistentViolation {
                        "escalate"
                    } else {
                        "review"
                    },
                    "S123 transition instability promoted rapidly after grammar-qualified transition excursion".into(),
                )
            } else if motif_label == DsfbMotifClass::PersistentInstabilityCluster {
                ("review", "S123 persistent instability cluster retained at Review until corroborated".into())
            } else {
                ("silent", "S123 remained below scaffold promotion conditions".into())
            }
        }
        "S540" | "S128" => {
            if semantic
                .as_ref()
                .is_some_and(|row| row.heuristic_name == TRANSITION_CLUSTER_SUPPORT)
            {
                (
                    if corroborated { "review" } else { "watch" },
                    "secondary corroborator contributes support but cannot escalate alone".into(),
                )
            } else {
                ("silent", "secondary corroborator remained structurally isolated".into())
            }
        }
        "S104" => {
            if matches!(motif_label, DsfbMotifClass::WatchOnlyBoundaryGrazing)
                && grammar_state == ScaffoldGrammarState::BoundaryGrazing
            {
                ("watch", "sentinel boundary grazing is watch-only by scaffold design".into())
            } else {
                ("silent", "sentinel remained admissible or unsupported".into())
            }
        }
        _ => ("silent", format!("{} is outside the scaffolded policy set", spec.feature_name)),
    }
}

fn related_feature_support(
    base_states: &BTreeMap<(String, usize), String>,
    feature_name: &str,
    run_index: usize,
    corroborated: bool,
) -> bool {
    if corroborated {
        return true;
    }
    match feature_name {
        "S059" => {
            has_active_state(base_states, "S133", run_index)
                || has_active_state(base_states, "S123", run_index)
        }
        "S133" => {
            has_active_state(base_states, "S059", run_index)
                || has_active_state(base_states, "S123", run_index)
        }
        "S123" => {
            has_active_state(base_states, "S059", run_index)
                || has_active_state(base_states, "S540", run_index)
                || has_active_state(base_states, "S128", run_index)
        }
        _ => false,
    }
}

fn has_active_state(
    base_states: &BTreeMap<(String, usize), String>,
    feature_name: &str,
    run_index: usize,
) -> bool {
    base_states
        .get(&(feature_name.to_string(), run_index))
        .is_some_and(|state| matches!(state.as_str(), "review" | "escalate"))
}

fn feature_scaffold_spec(feature_name: &str) -> Option<&'static FeatureScaffoldSpec> {
    FEATURE_SCAFFOLD
        .iter()
        .find(|spec| spec.feature_name == feature_name)
}

fn group_scaffold_spec(group_name: &str) -> Option<&'static GroupScaffoldSpec> {
    GROUP_SCAFFOLD
        .iter()
        .find(|spec| spec.group_name == group_name)
}

fn rescue_priority_rank(priority: &str) -> usize {
    match priority {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn classify_grammar_label(grammar_trace: &FeatureGrammarTrace, run_index: usize) -> ScaffoldGrammarState {
    let confirmed_state = grammar_trace.states[run_index];
    let confirmed_reason = grammar_trace.reasons[run_index];
    if confirmed_state == GrammarState::Admissible {
        if run_index > 0
            && grammar_trace.states[run_index - 1] != GrammarState::Admissible
        {
            ScaffoldGrammarState::Recovery
        } else {
            ScaffoldGrammarState::Admissible
        }
    } else if grammar_trace.persistent_violation[run_index] {
        ScaffoldGrammarState::PersistentViolation
    } else if confirmed_state == GrammarState::Violation
        || confirmed_reason == GrammarReason::EnvelopeViolation
    {
        ScaffoldGrammarState::TransientViolation
    } else if confirmed_reason == GrammarReason::SustainedOutwardDrift {
        ScaffoldGrammarState::SustainedOutwardDrift
    } else if confirmed_reason == GrammarReason::RecurrentBoundaryGrazing {
        ScaffoldGrammarState::BoundaryGrazing
    } else if confirmed_reason == GrammarReason::AbruptSlewViolation {
        ScaffoldGrammarState::TransientViolation
    } else {
        ScaffoldGrammarState::BoundaryGrazing
    }
}

#[derive(Debug, Clone, Copy)]
struct ExpandedHeuristicMetadata {
    allowed_grammar_states: &'static str,
    role_class: &'static str,
    feature_scope: &'static str,
    ambiguity_note: &'static str,
    rescue_eligibility_guidance: &'static str,
    burden_contribution_class: &'static str,
}

fn expanded_heuristic_entry_metadata(heuristic_name: &str) -> ExpandedHeuristicMetadata {
    match heuristic_name {
        PRE_FAILURE_SLOW_DRIFT => ExpandedHeuristicMetadata {
            allowed_grammar_states: "sustained_outward_drift,persistent_violation",
            role_class: "primary_precursor",
            feature_scope: "S059,S133 and compatible slow structural precursor features",
            ambiguity_note: "Slow drift remains interpretive and does not identify a unique mechanism.",
            rescue_eligibility_guidance: "Rescue-eligible on primary precursor features only.",
            burden_contribution_class: "review_burden_candidate",
        },
        RECURRENT_BOUNDARY_APPROACH => ExpandedHeuristicMetadata {
            allowed_grammar_states: "boundary_grazing,sustained_outward_drift",
            role_class: "precursor_or_support",
            feature_scope: "S059,S540 and compatible boundary-pressure features",
            ambiguity_note: "Repeated boundary approach is structurally meaningful but not uniquely causal.",
            rescue_eligibility_guidance: "Use for bounded Review promotion only when persistence is visible.",
            burden_contribution_class: "watch_to_review_pressure",
        },
        TRANSITION_EXCURSION => ExpandedHeuristicMetadata {
            allowed_grammar_states: "transient_violation,persistent_violation",
            role_class: "transition_instability",
            feature_scope: "S123 and compatible transition-instability features",
            ambiguity_note: "Transition excursions indicate abrupt structural change, not root-cause identity.",
            rescue_eligibility_guidance: "Rescue-eligible for transition features with persistent grammar support.",
            burden_contribution_class: "escalation_candidate",
        },
        PERSISTENT_INSTABILITY_CLUSTER => ExpandedHeuristicMetadata {
            allowed_grammar_states: "sustained_outward_drift,persistent_violation",
            role_class: "instability_cluster",
            feature_scope: "S123 and grouped instability clusters with sustained pressure",
            ambiguity_note: "Persistent clusters support detectability more strongly than identifiability.",
            rescue_eligibility_guidance: "Use to justify escalation only with grouped corroboration.",
            burden_contribution_class: "review_or_escalate_cluster",
        },
        TRANSITION_CLUSTER_SUPPORT => ExpandedHeuristicMetadata {
            allowed_grammar_states: "boundary_grazing,sustained_outward_drift,transient_violation",
            role_class: "secondary_corroborator",
            feature_scope: "S540,S128 and other corroborating support features",
            ambiguity_note: "Support motifs indicate alignment with a primary precursor, not a standalone alarm.",
            rescue_eligibility_guidance: "Never rescue to escalation from support features alone.",
            burden_contribution_class: "corroboration_support",
        },
        WATCH_ONLY_BOUNDARY_GRAZING => ExpandedHeuristicMetadata {
            allowed_grammar_states: "boundary_grazing",
            role_class: "sentinel",
            feature_scope: "S104 and low-amplitude sentinel features",
            ambiguity_note: "Boundary grazing is deliberately treated as semantically ambiguous and low-confidence.",
            rescue_eligibility_guidance: "Not rescue-eligible; retain as watch-only.",
            burden_contribution_class: "watch_only_burden",
        },
        _ => ExpandedHeuristicMetadata {
            allowed_grammar_states: "grammar_filtered",
            role_class: "generic",
            feature_scope: "all_features",
            ambiguity_note: "Structural semantics remain non-unique.",
            rescue_eligibility_guidance: "Apply only after grammar filtering.",
            burden_contribution_class: "generic",
        },
    }
}

fn semantic_regime_conditions(heuristic_name: &str) -> &'static str {
    match heuristic_name {
        PRE_FAILURE_SLOW_DRIFT => {
            "persistent signed drift, moderate residual growth, and limited abrupt slew"
        }
        RECURRENT_BOUNDARY_APPROACH => {
            "repeated boundary proximity with outward tendency and bounded fragmentation"
        }
        TRANSITION_EXCURSION => {
            "elevated slew burst aligned with a grammar transition or violation onset"
        }
        PERSISTENT_INSTABILITY_CLUSTER => {
            "repeated outward grammar pressure that is not reducible to isolated spikes"
        }
        TRANSITION_CLUSTER_SUPPORT => {
            "corroborating burst or pressure feature aligned with a grouped primary feature"
        }
        WATCH_ONLY_BOUNDARY_GRAZING => {
            "boundary proximity without sufficient persistence or corroboration for Review"
        }
        _ => "deterministic structural regime",
    }
}

fn semantic_applicability_rules(heuristic_name: &str) -> &'static str {
    match heuristic_name {
        PRE_FAILURE_SLOW_DRIFT => {
            "filter by grammar first, then apply only on slow structural precursor features"
        }
        RECURRENT_BOUNDARY_APPROACH => {
            "filter by grammar first, then apply on recurrent boundary-pressure features"
        }
        TRANSITION_EXCURSION => {
            "filter by grammar first, then apply on abrupt transition features"
        }
        PERSISTENT_INSTABILITY_CLUSTER => {
            "filter by grammar first, then require sustained grouped pressure"
        }
        TRANSITION_CLUSTER_SUPPORT => {
            "filter by grammar first, then require corroborator scope compatibility"
        }
        WATCH_ONLY_BOUNDARY_GRAZING => {
            "filter by grammar first, then confine to watch-only sentinel handling"
        }
        _ => "apply only after grammar filtering",
    }
}

fn compute_structural_delta_metrics(
    residuals: &ResidualSet,
    grammar: &GrammarSet,
    semantic_matches: &[SemanticMatchRecord],
    nominal: &NominalModel,
    failure_window_mask: &[bool],
) -> StructuralDeltaMetrics {
    let total_violation_points = grammar
        .traces
        .iter()
        .flat_map(|trace| trace.raw_states.iter().copied().enumerate())
        .filter(|(_, state)| *state == GrammarState::Violation)
        .count();
    let pre_failure_violation_points = grammar
        .traces
        .iter()
        .flat_map(|trace| trace.raw_states.iter().copied().enumerate())
        .filter(|(run_index, state)| {
            *state == GrammarState::Violation && failure_window_mask[*run_index]
        })
        .count();
    let grammar_violation_precision = (total_violation_points > 0)
        .then_some(pre_failure_violation_points as f64 / total_violation_points as f64);

    let motif_precision_pre_failure = if semantic_matches.is_empty() {
        None
    } else {
        Some(
            semantic_matches
                .iter()
                .filter(|row| failure_window_mask[row.run_index])
                .count() as f64
                / semantic_matches.len() as f64,
        )
    };

    let mut failure_separation = Vec::new();
    let mut pass_separation = Vec::new();
    for (trace, feature) in residuals.traces.iter().zip(&nominal.features) {
        for (run_index, norm) in trace.norms.iter().copied().enumerate() {
            let separation = norm / feature.rho.max(1.0e-12);
            if failure_window_mask[run_index] {
                failure_separation.push(separation);
            } else {
                pass_separation.push(separation);
            }
        }
    }
    let structural_separation_score = mean(&failure_separation)
        .zip(mean(&pass_separation))
        .map(|(failure, pass)| failure - pass);

    let precursor_stability_score = if semantic_matches.is_empty() {
        None
    } else {
        let mut grouped = BTreeMap::<(&str, usize), Vec<usize>>::new();
        for row in semantic_matches {
            grouped
                .entry((row.heuristic_name.as_str(), row.feature_index))
                .or_default()
                .push(row.run_index);
        }
        let mut matched_pre_failure_points = 0usize;
        let mut stable_pre_failure_points = 0usize;
        for runs in grouped.values_mut() {
            runs.sort_unstable();
            let mut episode_len = 0usize;
            let mut previous: Option<usize> = None;
            for &run_index in runs.iter() {
                if previous.is_some_and(|previous| run_index == previous + 1) {
                    episode_len += 1;
                } else {
                    episode_len = 1;
                }
                if failure_window_mask[run_index] {
                    matched_pre_failure_points += 1;
                    if episode_len >= 2 {
                        stable_pre_failure_points += 1;
                    }
                }
                previous = Some(run_index);
            }
        }
        (matched_pre_failure_points > 0)
            .then_some(stable_pre_failure_points as f64 / matched_pre_failure_points as f64)
    };

    StructuralDeltaMetrics {
        grammar_violation_precision,
        motif_precision_pre_failure,
        structural_separation_score,
        precursor_stability_score,
    }
}

fn failure_window_mask(
    run_count: usize,
    labels: &[i8],
    pre_failure_lookback_runs: usize,
) -> Vec<bool> {
    let failure_indices = labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| (*label == 1).then_some(index))
        .collect::<Vec<_>>();
    let mut mask = vec![false; run_count];
    for failure_index in failure_indices {
        let start = failure_index.saturating_sub(pre_failure_lookback_runs);
        for slot in &mut mask[start..failure_index] {
            *slot = true;
        }
    }
    mask
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
}

fn mean_of_records<T>(records: &[T], project: impl Fn(&T) -> f64) -> f64 {
    if records.is_empty() {
        0.0
    } else {
        records.iter().map(project).sum::<f64>() / records.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_candidates_are_grammar_conditioned() {
        let candidates = semantic_candidates_for_run(
            "S059",
            ScaffoldGrammarState::SustainedOutwardDrift,
            GrammarReason::SustainedOutwardDrift,
            DsfbMotifClass::PreFailureSlowDrift,
        );
        assert_eq!(candidates, vec![PRE_FAILURE_SLOW_DRIFT]);

        let candidates = semantic_candidates_for_run(
            "S104",
            ScaffoldGrammarState::Admissible,
            GrammarReason::Admissible,
            DsfbMotifClass::StableAdmissible,
        );
        assert!(candidates.is_empty());
    }

    #[test]
    fn scaffold_feature_specs_are_deterministic() {
        let s059 = feature_scaffold_spec("S059").unwrap();
        assert_eq!(s059.role, "persistent_boundary_approach_precursor");
        assert_eq!(s059.default_policy_ceiling, HeuristicAlertClass::Review);
        assert_eq!(s059.preferred_motifs[0], RECURRENT_BOUNDARY_APPROACH);

        let s104 = feature_scaffold_spec("S104").unwrap();
        assert_eq!(s104.default_policy_ceiling, HeuristicAlertClass::Watch);
        assert_eq!(s104.rescue_priority, "none");
    }
}
