#[cfg(feature = "std")]
use crate::error::Result;
use crate::grammar::layer::GrammarState;
use crate::syntax::MotifTimelinePoint;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Heuristic {
    pub id: String,
    pub features: Vec<String>,
    pub motif_signature: Vec<String>,
    pub grammar_states: Vec<String>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticMatch {
    pub timestamp: f64,
    pub feature_id: String,
    pub heuristic_id: String,
    pub motif_type: String,
    pub grammar_state: String,
    pub action: String,
}

pub fn minimal_heuristics_bank() -> Vec<Heuristic> {
    vec![
        Heuristic {
            id: "failure_slow_drift_review".into(),
            features: vec!["S059".into(), "S133".into(), "S134".into(), "S275".into()],
            motif_signature: vec!["slow_drift_precursor".into()],
            grammar_states: vec!["SustainedDrift".into(), "PersistentViolation".into()],
            action: "Review".into(),
        },
        Heuristic {
            id: "nuisance_boundary_grazing_watch".into(),
            features: vec!["S059".into(), "S104".into()],
            motif_signature: vec!["boundary_grazing".into()],
            grammar_states: vec!["BoundaryGrazing".into()],
            action: "Watch".into(),
        },
        Heuristic {
            id: "transition_instability_escalate".into(),
            features: vec!["S123".into(), "S540".into(), "S128".into()],
            motif_signature: vec![
                "persistent_instability".into(),
                "burst_instability".into(),
                "transient_excursion".into(),
            ],
            grammar_states: vec!["TransientViolation".into(), "PersistentViolation".into()],
            action: "Escalate".into(),
        },
        Heuristic {
            id: "recovery_deescalate_silent".into(),
            features: Vec::new(),
            motif_signature: vec!["recovery_pattern".into()],
            grammar_states: vec!["Recovery".into()],
            action: "Silent".into(),
        },
        Heuristic {
            id: "noise_suppress_silent".into(),
            features: Vec::new(),
            motif_signature: vec!["noise_like".into(), "null".into()],
            grammar_states: vec!["Admissible".into()],
            action: "Silent".into(),
        },
    ]
}

pub fn match_semantics(
    motifs: &[MotifTimelinePoint],
    grammar_states: &[GrammarState],
    heuristics: &[Heuristic],
) -> Vec<SemanticMatch> {
    let motif_map = motifs.iter().fold(BTreeMap::new(), |mut acc, row| {
        acc.insert(
            (row.feature_id.clone(), row.timestamp.to_bits()),
            row.motif_type.clone(),
        );
        acc
    });

    let mut matches = Vec::new();
    for state in grammar_states {
        let Some(motif_type) =
            motif_map.get(&(state.feature_id.clone(), state.timestamp.to_bits()))
        else {
            continue;
        };
        for heuristic in heuristics {
            let feature_match = heuristic.features.is_empty()
                || heuristic
                    .features
                    .iter()
                    .any(|feature| feature == &state.feature_id);
            let motif_match = heuristic
                .motif_signature
                .iter()
                .any(|motif| motif == motif_type);
            let grammar_match = heuristic
                .grammar_states
                .iter()
                .any(|grammar| grammar == &state.state);
            if feature_match && motif_match && grammar_match {
                matches.push(SemanticMatch {
                    timestamp: state.timestamp,
                    feature_id: state.feature_id.clone(),
                    heuristic_id: heuristic.id.clone(),
                    motif_type: motif_type.clone(),
                    grammar_state: state.state.clone(),
                    action: heuristic.action.clone(),
                });
            }
        }
    }

    matches.sort_by(|left, right| {
        left.timestamp
            .total_cmp(&right.timestamp)
            .then_with(|| left.feature_id.cmp(&right.feature_id))
            .then_with(|| left.heuristic_id.cmp(&right.heuristic_id))
    });
    matches
}

#[cfg(feature = "std")]
pub fn write_heuristics_bank_json(path: &std::path::Path, rows: &[Heuristic]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, rows)?;
    Ok(())
}

#[cfg(feature = "std")]
pub fn write_semantic_matches_csv(path: &std::path::Path, rows: &[SemanticMatch]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantics_require_grammar_qualification() {
        let motifs = vec![MotifTimelinePoint {
            feature_id: "S059".into(),
            motif_type: "slow_drift_precursor".into(),
            timestamp: 1.0,
        }];
        let heuristics = minimal_heuristics_bank();
        let no_grammar = match_semantics(&motifs, &[], &heuristics);
        assert!(no_grammar.is_empty());

        let grammar = vec![GrammarState {
            feature_id: "S059".into(),
            state: "SustainedDrift".into(),
            timestamp: 1.0,
        }];
        let qualified = match_semantics(&motifs, &grammar, &heuristics);
        assert_eq!(qualified.len(), 1);
        assert_eq!(qualified[0].action, "Review");
    }
}
