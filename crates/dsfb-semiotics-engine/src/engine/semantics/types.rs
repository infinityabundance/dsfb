//! Internal semantics support types shared across retrieval, explanations, and governance helpers.

use std::collections::BTreeSet;

use crate::engine::types::CoordinatedResidualStructure;

#[derive(Clone, Debug)]
pub(crate) struct GrammarEvidence {
    pub boundary_count: usize,
    pub violation_count: usize,
    pub regimes: Vec<String>,
}

pub(crate) fn available_regimes(
    evidence: &GrammarEvidence,
    coordinated: Option<&CoordinatedResidualStructure>,
) -> Vec<String> {
    let mut regimes = evidence.regimes.iter().cloned().collect::<BTreeSet<_>>();
    if coordinated.is_some() {
        regimes.insert("aggregate".to_string());
    }
    regimes.into_iter().collect()
}

pub(crate) fn coordinated_group_breach_ratio(
    coordinated: Option<&CoordinatedResidualStructure>,
) -> f64 {
    match coordinated {
        Some(structure) if !structure.points.is_empty() => {
            structure
                .points
                .iter()
                .filter(|point| point.aggregate_margin < 0.0)
                .count() as f64
                / structure.points.len() as f64
        }
        _ => 0.0,
    }
}
