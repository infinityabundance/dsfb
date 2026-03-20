//! Compatibility assessment for multi-candidate semantic retrieval results.

use crate::engine::types::HeuristicCandidate;

#[derive(Clone, Debug, Default)]
pub(crate) struct CompatibilityAssessment {
    pub compatible_pairs: Vec<String>,
    pub conflicts: Vec<String>,
    pub unresolved: Vec<String>,
}

pub(crate) fn compatibility_assessment(
    candidates: &[HeuristicCandidate],
) -> CompatibilityAssessment {
    let mut compatible_pairs = Vec::new();
    let mut conflicts = Vec::new();
    let mut unresolved = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let left = &candidates[i].entry;
            let right = &candidates[j].entry;
            if left.incompatible_with.contains(&right.heuristic_id)
                || right.incompatible_with.contains(&left.heuristic_id)
            {
                conflicts.push(format!(
                    "{} conflicts with {} under the bank compatibility rules.",
                    left.motif_label, right.motif_label
                ));
            } else if left.compatible_with.contains(&right.heuristic_id)
                && right.compatible_with.contains(&left.heuristic_id)
            {
                compatible_pairs.push(format!(
                    "{} and {} are reported together because the typed bank explicitly marks the pair as jointly compatible under the current admissibility-qualified evidence.",
                    left.motif_label, right.motif_label
                ));
            } else if !left.compatible_with.contains(&right.heuristic_id)
                || !right.compatible_with.contains(&left.heuristic_id)
            {
                unresolved.push(format!(
                    "{} and {} both matched, but the bank does not mark the pair as explicitly compatible.",
                    left.motif_label, right.motif_label
                ));
            }
        }
    }
    CompatibilityAssessment {
        compatible_pairs,
        conflicts,
        unresolved,
    }
}
