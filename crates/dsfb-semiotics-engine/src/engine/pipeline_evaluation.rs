//! Deterministic pipeline evaluation helpers for reproducibility aggregation and output comparison.

use anyhow::Result;

use crate::engine::types::{ReproducibilityCheck, ReproducibilitySummary, ScenarioOutput};
use crate::math::metrics::hash_serializable_hex;

// TRACE:CLAIM:CLM-COMPUTATIONAL-REPRODUCIBILITY:Layered output reproducibility:Hashes full scenario outputs twice under identical deterministic configuration.
pub(crate) fn compare_outputs(
    first: &ScenarioOutput,
    second: &ScenarioOutput,
) -> Result<ReproducibilityCheck> {
    let first_hash = hash_serializable_hex(format!("{}-first", first.record.id), first)?;
    let second_hash = hash_serializable_hex(format!("{}-second", second.record.id), second)?;
    Ok(ReproducibilityCheck {
        scenario_id: first.record.id.clone(),
        first_hash: first_hash.fnv1a_64_hex.clone(),
        second_hash: second_hash.fnv1a_64_hex.clone(),
        identical: first_hash.fnv1a_64_hex == second_hash.fnv1a_64_hex,
        materialized_components: vec![
            "observed".to_string(),
            "predicted".to_string(),
            "residual".to_string(),
            "drift".to_string(),
            "slew".to_string(),
            "sign".to_string(),
            "envelope".to_string(),
            "grammar".to_string(),
            "syntax".to_string(),
            "detectability".to_string(),
            "semantics".to_string(),
            "coordinated".to_string(),
        ],
        note: "Scenario output was materialized twice under identical deterministic configuration and hashed over full layered outputs, including grammar and semantics.".to_string(),
    })
}

// TRACE:CLAIM:CLM-REPRODUCIBILITY-SUMMARY:Aggregate reproducibility summary:Summarizes per-scenario identical reruns over the full layered output bundle.
pub(crate) fn summarize_reproducibility(checks: &[ReproducibilityCheck]) -> ReproducibilitySummary {
    let identical_count = checks.iter().filter(|check| check.identical).count();
    ReproducibilitySummary {
        scenario_count: checks.len(),
        identical_count,
        all_identical: identical_count == checks.len(),
        note: "Per-scenario reproducibility is evaluated over full materialized outputs rather than reduced norm summaries.".to_string(),
    }
}
