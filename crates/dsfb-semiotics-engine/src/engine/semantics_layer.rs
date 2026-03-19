use crate::engine::types::{
    CoordinatedResidualStructure, GrammarState, GrammarStatus, HeuristicCandidate,
    SemanticDisposition, SemanticMatchResult, SyntaxCharacterization,
};

pub fn retrieve_semantics(
    scenario_id: &str,
    syntax: &SyntaxCharacterization,
    grammar: &[GrammarStatus],
    coordinated: Option<&CoordinatedResidualStructure>,
) -> SemanticMatchResult {
    let violation_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Violation))
        .count();
    let boundary_count = grammar
        .iter()
        .filter(|status| matches!(status.state, GrammarState::Boundary))
        .count();

    let mut candidates = Vec::new();

    if syntax.monotone_drift_fraction > 0.78 && syntax.max_slew_norm < 0.12 {
        candidates.push(HeuristicCandidate {
            heuristic_id: "H-MONOTONE-DRIFT".to_string(),
            label: "gradual degradation candidate".to_string(),
            score: syntax.monotone_drift_fraction,
            rationale: "Sustained monotone drift with low curvature is compatible with a slow structural migration motif.".to_string(),
        });
    }

    if syntax.slew_spike_count > 0 && syntax.max_slew_norm > 0.2 {
        candidates.push(HeuristicCandidate {
            heuristic_id: "H-SLEW-SPIKE".to_string(),
            label: "discrete event candidate".to_string(),
            score: syntax.max_slew_norm,
            rationale: "Localized high-slew content is compatible with an abrupt event, switching action, or impact-like disturbance.".to_string(),
        });
    }

    if syntax.repeated_grazing_count >= 3 && violation_count == 0 && boundary_count >= 3 {
        candidates.push(HeuristicCandidate {
            heuristic_id: "H-GRAZING".to_string(),
            label: "near-boundary operation candidate".to_string(),
            score: syntax.repeated_grazing_count as f64,
            rationale: "Repeated admissibility grazing with recovery is compatible with marginal or stressed operation rather than a decisive regime departure.".to_string(),
        });
    }

    if let Some(coordinated) = coordinated {
        let negative_margin_count = coordinated
            .points
            .iter()
            .filter(|point| point.aggregate_margin < 0.0)
            .count();
        if negative_margin_count > 0 {
            candidates.push(HeuristicCandidate {
                heuristic_id: "H-COORDINATED-RISE".to_string(),
                label: "correlated degradation or common-mode disturbance candidate".to_string(),
                score: negative_margin_count as f64,
                rationale: "Aggregate group envelope breach indicates correlated structure across a monitored subset rather than an isolated local excursion.".to_string(),
            });
        }
    }

    let disposition = match candidates.len() {
        0 => SemanticDisposition::Unknown,
        1 => SemanticDisposition::Match,
        _ => SemanticDisposition::Ambiguous,
    };
    let selected_labels = candidates
        .iter()
        .map(|candidate| candidate.label.clone())
        .collect::<Vec<_>>();

    SemanticMatchResult {
        scenario_id: scenario_id.to_string(),
        disposition,
        motif_summary: format!(
            "syntax={}, outward_fraction={:.3}, max_slew={:.3}, grammar_violations={}",
            syntax.trajectory_label,
            syntax.outward_drift_fraction,
            syntax.max_slew_norm,
            violation_count
        ),
        candidates,
        selected_labels,
        note: "Heuristic outputs are constrained, synthetic motif matches only. They are not universal causal labels and are allowed to remain ambiguous or unknown.".to_string(),
    }
}
