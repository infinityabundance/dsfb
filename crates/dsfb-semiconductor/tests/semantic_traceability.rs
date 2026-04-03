use dsfb_semiconductor::semiotics::{
    FeatureGrammarStateRecord, FeatureMotifTimelineRecord, FeaturePolicyDecisionRecord,
    FeatureSignRecord, ScaffoldSemioticsArtifacts,
};
use dsfb_semiconductor::traceability::build_traceability_entries;

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
fn traceability_chain_is_complete_and_human_readable() {
    let mut scaffold = empty_scaffold();
    scaffold.feature_signs.push(FeatureSignRecord {
        feature_index: 59,
        feature_name: "S059".into(),
        feature_role: "primary recurrent-boundary precursor".into(),
        group_name: "group_a".into(),
        run_index: 42,
        timestamp: "2008-01-01T00:42:00Z".into(),
        label: 1,
        normalized_residual: 1.84,
        drift: 0.22,
        slew: 0.03,
        normalized_residual_norm: 1.84,
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
            run_index: 42,
            timestamp: "2008-01-01T00:42:00Z".into(),
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
            run_index: 42,
            timestamp: "2008-01-01T00:42:00Z".into(),
            label: 1,
            grammar_state: "SustainedDrift".into(),
            raw_state: "Boundary".into(),
            confirmed_state: "Boundary".into(),
            raw_reason: "SustainedOutwardDrift".into(),
            confirmed_reason: "SustainedOutwardDrift".into(),
            normalized_envelope_ratio: 0.81,
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
            run_index: 42,
            timestamp: "2008-01-01T00:42:00Z".into(),
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
    assert_eq!(entries[0].event_id, "S059:42:escalate");
    assert_eq!(entries[0].features, vec!["S059".to_string()]);
    assert_eq!(entries[0].motif, "slow_drift_precursor");
    assert_eq!(entries[0].grammar, "SustainedDrift");
    assert_eq!(entries[0].semantic, "pre-failure cluster");
    assert_eq!(entries[0].policy, "Escalate");
    assert_eq!(
        entries[0].chain,
        "Residual -> Sign -> Motif -> Grammar -> Semantic -> Policy"
    );
    assert_eq!(entries[0].integration_mode, "read_only_side_channel");
}
