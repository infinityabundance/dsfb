#[cfg(feature = "std")]
use crate::error::Result;
use crate::grammar::layer::GrammarState;
use crate::semantics::SemanticMatch;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::{String, ToString}, vec::Vec};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyDecision {
    pub timestamp: f64,
    pub decision: String,
}

pub fn derive_policy(
    semantic_matches: &[SemanticMatch],
    grammar_states: &[GrammarState],
) -> Vec<PolicyDecision> {
    let mut by_timestamp = BTreeMap::<u64, String>::new();

    for state in grammar_states {
        let fallback = if state.state == "Admissible" {
            "Silent"
        } else {
            "Watch"
        };
        by_timestamp
            .entry(state.timestamp.to_bits())
            .and_modify(|current| {
                if decision_rank(fallback) > decision_rank(current) {
                    *current = fallback.to_string();
                }
            })
            .or_insert_with(|| fallback.to_string());
    }

    for semantic in semantic_matches {
        by_timestamp
            .entry(semantic.timestamp.to_bits())
            .and_modify(|current| {
                if decision_rank(&semantic.action) > decision_rank(current) {
                    *current = semantic.action.clone();
                }
            })
            .or_insert_with(|| semantic.action.clone());
    }

    by_timestamp
        .into_iter()
        .map(|(timestamp_bits, decision)| PolicyDecision {
            timestamp: f64::from_bits(timestamp_bits),
            decision,
        })
        .collect()
}

#[cfg(feature = "std")]
pub fn write_policy_decisions_csv(
    path: &std::path::Path,
    rows: &[PolicyDecision],
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn decision_rank(decision: &str) -> usize {
    match decision {
        "Escalate" => 3,
        "Review" => 2,
        "Watch" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_uses_semantics_and_grammar_only() {
        let grammar = vec![
            GrammarState {
                feature_id: "S059".into(),
                state: "BoundaryGrazing".into(),
                timestamp: 1.0,
            },
            GrammarState {
                feature_id: "S059".into(),
                state: "Admissible".into(),
                timestamp: 2.0,
            },
        ];
        let semantics = vec![SemanticMatch {
            timestamp: 1.0,
            feature_id: "S059".into(),
            heuristic_id: "failure_slow_drift_review".into(),
            motif_type: "slow_drift_precursor".into(),
            grammar_state: "SustainedDrift".into(),
            action: "Review".into(),
        }];
        let decisions = derive_policy(&semantics, &grammar);
        assert_eq!(decisions[0].decision, "Review");
        assert_eq!(decisions[1].decision, "Silent");
    }
}
