use crate::error::Result;
use crate::semiotics::{
    FeatureGrammarStateRecord, FeatureMotifTimelineRecord, FeatureSignRecord,
    ScaffoldSemioticsArtifacts,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

const TRACE_CHAIN: &str = "Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy";
const INTEGRATION_MODE: &str = "read_only_side_channel";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TraceabilitySign {
    pub normalized_residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub normalized_residual_norm: f64,
    pub sigma_norm: f64,
    pub is_imputed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TraceabilityEntry {
    pub event_id: String,
    pub features: Vec<String>,
    pub feature_role: String,
    pub group_name: String,
    pub run_index: usize,
    pub timestamp: String,
    pub label: i8,
    pub residual: f64,
    pub sign: TraceabilitySign,
    pub motif: String,
    pub grammar: String,
    pub semantic: String,
    pub policy: String,
    pub rationale: String,
    pub chain: String,
    pub integration_mode: String,
}

pub fn build_traceability_entries(
    scaffold: &ScaffoldSemioticsArtifacts,
) -> Vec<TraceabilityEntry> {
    let sign_rows = index_signs(&scaffold.feature_signs);
    let motif_rows = index_motifs(&scaffold.feature_motif_timeline);
    let grammar_rows = index_grammar(&scaffold.feature_grammar_states);

    let mut entries = scaffold
        .feature_policy_decisions
        .iter()
        .filter(|row| {
            row.investigation_worthy
                || row.semantic_label.is_some()
                || row.policy_state != "Silent"
                || row.grammar_state != "admissible"
        })
        .filter_map(|policy_row| {
            let key = (policy_row.feature_name.as_str(), policy_row.run_index);
            let sign_row = sign_rows.get(&key)?;
            let motif_row = motif_rows.get(&key)?;
            let grammar_row = grammar_rows.get(&key)?;
            Some(TraceabilityEntry {
                event_id: format!(
                    "{}:{}:{}",
                    policy_row.feature_name,
                    policy_row.run_index,
                    policy_row.policy_state.to_lowercase()
                ),
                features: vec![policy_row.feature_name.clone()],
                feature_role: policy_row.feature_role.clone(),
                group_name: policy_row.group_name.clone(),
                run_index: policy_row.run_index,
                timestamp: policy_row.timestamp.clone(),
                label: policy_row.label,
                residual: sign_row.normalized_residual,
                sign: TraceabilitySign {
                    normalized_residual: sign_row.normalized_residual,
                    drift: sign_row.drift,
                    slew: sign_row.slew,
                    normalized_residual_norm: sign_row.normalized_residual_norm,
                    sigma_norm: sign_row.sigma_norm,
                    is_imputed: sign_row.is_imputed,
                },
                motif: motif_row.motif_label.clone(),
                grammar: grammar_row.grammar_state.clone(),
                semantic: policy_row
                    .semantic_label
                    .clone()
                    .unwrap_or_else(|| "no_semantic_match".into()),
                policy: policy_row.policy_state.clone(),
                rationale: policy_row.rationale.clone(),
                chain: TRACE_CHAIN.into(),
                integration_mode: INTEGRATION_MODE.into(),
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        left.run_index
            .cmp(&right.run_index)
            .then_with(|| left.features[0].cmp(&right.features[0]))
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    entries
}

pub fn write_traceability_json(path: &Path, entries: &[TraceabilityEntry]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, entries)?;
    Ok(())
}

fn index_signs(rows: &[FeatureSignRecord]) -> BTreeMap<(&str, usize), &FeatureSignRecord> {
    rows.iter()
        .map(|row| ((row.feature_name.as_str(), row.run_index), row))
        .collect()
}

fn index_motifs(
    rows: &[FeatureMotifTimelineRecord],
) -> BTreeMap<(&str, usize), &FeatureMotifTimelineRecord> {
    rows.iter()
        .map(|row| ((row.feature_name.as_str(), row.run_index), row))
        .collect()
}

fn index_grammar(
    rows: &[FeatureGrammarStateRecord],
) -> BTreeMap<(&str, usize), &FeatureGrammarStateRecord> {
    rows.iter()
        .map(|row| ((row.feature_name.as_str(), row.run_index), row))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiotics::{
        FeatureGrammarStateRecord, FeatureMotifTimelineRecord, FeaturePolicyDecisionRecord,
        FeatureSignRecord, ScaffoldSemioticsArtifacts,
    };

    fn empty_scaffold() -> ScaffoldSemioticsArtifacts {
        ScaffoldSemioticsArtifacts {
            feature_signs: Vec::new(),
            feature_motif_timeline: Vec::new(),
            feature_grammar_states: Vec::new(),
            envelope_interaction_summary: Vec::new(),
            heuristics_bank_expanded: Vec::new(),
            feature_policy_decisions: Vec::new(),
            group_definitions: Vec::new(),
            group_signs: Vec::new(),
            group_grammar_states: Vec::new(),
            group_semantic_matches: Vec::new(),
        }
    }

    #[test]
    fn traceability_entries_preserve_full_chain() {
        let mut scaffold = empty_scaffold();
        scaffold.feature_signs.push(FeatureSignRecord {
            feature_index: 59,
            feature_name: "S059".into(),
            feature_role: "primary recurrent-boundary precursor".into(),
            group_name: "group_a".into(),
            run_index: 11,
            timestamp: "2008-01-01T00:11:00Z".into(),
            label: 1,
            normalized_residual: 1.8,
            drift: 0.3,
            slew: 0.1,
            normalized_residual_norm: 1.8,
            sigma_norm: 1.0,
            is_imputed: false,
        });
        scaffold
            .feature_motif_timeline
            .push(FeatureMotifTimelineRecord {
                feature_index: 59,
                feature_name: "S059".into(),
                feature_role: "primary recurrent-boundary precursor".into(),
                group_name: "group_a".into(),
                run_index: 11,
                timestamp: "2008-01-01T00:11:00Z".into(),
                label: 1,
                motif_label: "slow_drift_precursor".into(),
            });
        scaffold
            .feature_grammar_states
            .push(FeatureGrammarStateRecord {
                feature_index: 59,
                feature_name: "S059".into(),
                feature_role: "primary recurrent-boundary precursor".into(),
                group_name: "group_a".into(),
                run_index: 11,
                timestamp: "2008-01-01T00:11:00Z".into(),
                label: 1,
                grammar_state: "SustainedDrift".into(),
                raw_state: "Boundary".into(),
                confirmed_state: "Boundary".into(),
                raw_reason: "SustainedOutwardDrift".into(),
                confirmed_reason: "SustainedOutwardDrift".into(),
                normalized_envelope_ratio: 0.82,
                persistent_boundary: true,
                persistent_violation: false,
                suppressed_by_imputation: false,
            });
        scaffold
            .feature_policy_decisions
            .push(FeaturePolicyDecisionRecord {
                feature_index: 59,
                feature_name: "S059".into(),
                feature_role: "primary recurrent-boundary precursor".into(),
                group_name: "group_a".into(),
                run_index: 11,
                timestamp: "2008-01-01T00:11:00Z".into(),
                label: 1,
                grammar_state: "SustainedDrift".into(),
                motif_label: "slow_drift_precursor".into(),
                semantic_label: Some("pre-failure cluster".into()),
                policy_ceiling: "Escalate".into(),
                policy_state: "Escalate".into(),
                investigation_worthy: true,
                corroborated: true,
                corroborated_by: "S133".into(),
                rationale: "persistent outward drift with corroboration".into(),
            });

        let entries = build_traceability_entries(&scaffold);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].features, vec!["S059".to_string()]);
        assert_eq!(entries[0].motif, "slow_drift_precursor");
        assert_eq!(entries[0].grammar, "SustainedDrift");
        assert_eq!(entries[0].semantic, "pre-failure cluster");
        assert_eq!(entries[0].policy, "Escalate");
        assert_eq!(entries[0].chain, TRACE_CHAIN);
        assert_eq!(entries[0].integration_mode, INTEGRATION_MODE);
    }
}
