use crate::grammar::{FeatureGrammarTrace, GrammarReason, GrammarSet, GrammarState};
use crate::heuristics::{
    heuristic_policy_definition, HeuristicAlertClass, PRE_FAILURE_SLOW_DRIFT,
    RECURRENT_BOUNDARY_APPROACH, TRANSIENT_EXCURSION,
};
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::{ResidualFeatureTrace, ResidualSet};
use crate::signs::{FeatureSigns, SignSet};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DsfbMotifClass {
    MonotoneDrift,
    CurvatureDominated,
    Oscillatory,
    BoundaryApproaching,
    Burst,
    StableAdmissible,
}

impl DsfbMotifClass {
    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::MonotoneDrift => "monotone_drift",
            Self::CurvatureDominated => "curvature_dominated",
            Self::Oscillatory => "oscillatory",
            Self::BoundaryApproaching => "boundary_approaching",
            Self::Burst => "burst",
            Self::StableAdmissible => "stable_admissible",
        }
    }

    pub fn definition(self) -> &'static str {
        match self {
            Self::MonotoneDrift => {
                "Residual sign stays outward with thresholded positive drift and sub-slew curvature."
            }
            Self::CurvatureDominated => {
                "Second-order structure dominates: absolute slew exceeds the healthy-window slew threshold without direct envelope exit."
            }
            Self::Oscillatory => {
                "Thresholded drift direction flips across neighboring runs, indicating flicker or alternating excursions."
            }
            Self::BoundaryApproaching => {
                "Residual norm revisits or approaches the local envelope boundary without a direct confirmed violation."
            }
            Self::Burst => {
                "Abrupt excursion with strong curvature and direct boundary or violation interaction."
            }
            Self::StableAdmissible => {
                "Admissible low-structure regime with no thresholded drift or slew dominance."
            }
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
        let labels = classify_feature_motif_labels(
            residual_trace,
            sign_trace,
            grammar_trace,
            feature.rho,
        );
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
        DsfbMotifClass::MonotoneDrift,
        DsfbMotifClass::CurvatureDominated,
        DsfbMotifClass::Oscillatory,
        DsfbMotifClass::BoundaryApproaching,
        DsfbMotifClass::Burst,
        DsfbMotifClass::StableAdmissible,
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
    let motif_labels = classify_feature_motif_labels(
        residual_trace,
        sign_trace,
        grammar_trace,
        feature_rho,
    );
    let mut semantic_flags = BTreeMap::<&'static str, Vec<bool>>::new();
    for heuristic_name in [
        PRE_FAILURE_SLOW_DRIFT,
        TRANSIENT_EXCURSION,
        RECURRENT_BOUNDARY_APPROACH,
    ] {
        semantic_flags.insert(heuristic_name, vec![false; motif_labels.len()]);
    }

    let mut any_semantic_match = vec![false; motif_labels.len()];
    for run_index in 0..motif_labels.len() {
        let matched = semantic_candidates_for_run(
            grammar_trace.raw_states[run_index],
            grammar_trace.raw_reasons[run_index],
            motif_labels[run_index],
        );
        for (rank, heuristic_name) in matched.into_iter().enumerate() {
            let flags = semantic_flags
                .get_mut(heuristic_name)
                .unwrap_or_else(|| panic!("missing semantic flag bucket for {heuristic_name}"));
            flags[run_index] = true;
            if rank == 0 {
                any_semantic_match[run_index] = true;
            }
        }
    }

    FeatureSemanticFlags {
        semantic_flags,
        any_semantic_match,
    }
}

fn classify_feature_motif_labels(
    residual_trace: &ResidualFeatureTrace,
    sign_trace: &FeatureSigns,
    grammar_trace: &FeatureGrammarTrace,
    feature_rho: f64,
) -> Vec<DsfbMotifClass> {
    let mut labels = Vec::with_capacity(residual_trace.norms.len());
    for run_index in 0..residual_trace.norms.len() {
        let norm = residual_trace.norms[run_index];
        let drift = sign_trace.drift[run_index];
        let slew_abs = sign_trace.slew[run_index].abs();
        let drift_threshold = sign_trace.drift_threshold;
        let slew_threshold = sign_trace.slew_threshold;
        let raw_state = grammar_trace.raw_states[run_index];
        let raw_reason = grammar_trace.raw_reasons[run_index];
        let drift_sign = thresholded_sign(drift, drift_threshold);
        let previous_sign = run_index
            .checked_sub(1)
            .map(|index| thresholded_sign(sign_trace.drift[index], drift_threshold))
            .unwrap_or(0);

        let label = if raw_state == GrammarState::Violation && slew_abs >= slew_threshold {
            DsfbMotifClass::Burst
        } else if raw_reason == GrammarReason::AbruptSlewViolation || slew_abs >= slew_threshold {
            if raw_state == GrammarState::Boundary || raw_state == GrammarState::Violation {
                DsfbMotifClass::CurvatureDominated
            } else {
                DsfbMotifClass::Oscillatory
            }
        } else if raw_reason == GrammarReason::SustainedOutwardDrift
            && drift_sign > 0
            && slew_abs < slew_threshold.max(1.0e-12)
        {
            DsfbMotifClass::MonotoneDrift
        } else if raw_reason == GrammarReason::RecurrentBoundaryGrazing
            || raw_state == GrammarState::Boundary
        {
            DsfbMotifClass::BoundaryApproaching
        } else if drift_sign != 0 && previous_sign != 0 && drift_sign != previous_sign {
            DsfbMotifClass::Oscillatory
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
    for run_index in 0..motif_trace.labels.len() {
        let candidates = semantic_candidates_for_run(
            grammar_trace.raw_states[run_index],
            grammar_trace.raw_reasons[run_index],
            motif_trace.labels[run_index],
        );
        for (rank, heuristic_name) in candidates.into_iter().enumerate() {
            let policy = heuristic_policy_definition(heuristic_name)
                .unwrap_or_else(|| panic!("missing heuristic definition for {heuristic_name}"));
            let (grammar_constraints, regime_conditions, applicability_rules) =
                semantic_rule_strings(heuristic_name);
            rows.push(SemanticMatchRecord {
                feature_index: residual_trace.feature_index,
                feature_name: residual_trace.feature_name.clone(),
                run_index,
                timestamp: dataset.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                label: dataset.labels[run_index],
                grammar_state: format!("{:?}", grammar_trace.raw_states[run_index]),
                grammar_reason: format!("{:?}", grammar_trace.raw_reasons[run_index]),
                motif_label: motif_trace.labels[run_index].as_lowercase().into(),
                heuristic_name: heuristic_name.into(),
                alert_class_default: policy.alert_class_default.as_lowercase().into(),
                grammar_constraints: grammar_constraints.into(),
                regime_conditions: regime_conditions.into(),
                applicability_rules: applicability_rules.into(),
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
    grammar_state: GrammarState,
    grammar_reason: GrammarReason,
    motif_label: DsfbMotifClass,
) -> Vec<&'static str> {
    let mut candidates = Vec::new();

    if grammar_state == GrammarState::Boundary
        && grammar_reason == GrammarReason::SustainedOutwardDrift
        && matches!(
            motif_label,
            DsfbMotifClass::MonotoneDrift | DsfbMotifClass::BoundaryApproaching
        )
    {
        candidates.push(PRE_FAILURE_SLOW_DRIFT);
    }
    if matches!(grammar_state, GrammarState::Boundary | GrammarState::Violation)
        && grammar_reason == GrammarReason::AbruptSlewViolation
        && matches!(
            motif_label,
            DsfbMotifClass::CurvatureDominated | DsfbMotifClass::Burst
        )
    {
        candidates.push(TRANSIENT_EXCURSION);
    }
    if grammar_state == GrammarState::Boundary
        && grammar_reason == GrammarReason::RecurrentBoundaryGrazing
        && matches!(
            motif_label,
            DsfbMotifClass::BoundaryApproaching | DsfbMotifClass::Oscillatory
        )
    {
        candidates.push(RECURRENT_BOUNDARY_APPROACH);
    }

    candidates
}

fn semantic_rule_strings(
    heuristic_name: &str,
) -> (&'static str, &'static str, &'static str) {
    match heuristic_name {
        PRE_FAILURE_SLOW_DRIFT => (
            "grammar_state=Boundary and grammar_reason=SustainedOutwardDrift",
            "outward drift is thresholded and curvature stays sub-dominant",
            "apply only after grammar filtering confirms boundary proximity without direct violation",
        ),
        TRANSIENT_EXCURSION => (
            "grammar_state in {Boundary, Violation} and grammar_reason=AbruptSlewViolation",
            "curvature dominates the local sign tuple and the trajectory remains non-admissible",
            "apply only after grammar filtering confirms abrupt boundary interaction",
        ),
        RECURRENT_BOUNDARY_APPROACH => (
            "grammar_state=Boundary and grammar_reason=RecurrentBoundaryGrazing",
            "boundary revisitation persists without direct envelope exit",
            "apply only after grammar filtering confirms repeated boundary approach",
        ),
        _ => (
            "grammar filtered",
            "deterministic structural regime",
            "apply only after grammar filtering",
        ),
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
    let structural_separation_score =
        mean(&failure_separation).zip(mean(&pass_separation)).map(|(failure, pass)| failure - pass);

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

fn failure_window_mask(run_count: usize, labels: &[i8], pre_failure_lookback_runs: usize) -> Vec<bool> {
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

fn thresholded_sign(value: f64, threshold: f64) -> i8 {
    if value >= threshold {
        1
    } else if value <= -threshold {
        -1
    } else {
        0
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_candidates_are_grammar_conditioned() {
        let candidates = semantic_candidates_for_run(
            GrammarState::Boundary,
            GrammarReason::SustainedOutwardDrift,
            DsfbMotifClass::MonotoneDrift,
        );
        assert_eq!(candidates, vec![PRE_FAILURE_SLOW_DRIFT]);

        let candidates = semantic_candidates_for_run(
            GrammarState::Admissible,
            GrammarReason::Admissible,
            DsfbMotifClass::StableAdmissible,
        );
        assert!(candidates.is_empty());
    }
}
