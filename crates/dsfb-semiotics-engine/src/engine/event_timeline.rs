//! Event-timeline helpers for operator-facing artifacts and figure generation.
//!
//! The helpers here derive deterministic, time-ordered structural events from existing run
//! outputs. They do not invent new labels; they summarize syntax, grammar, semantic, and
//! comparator transitions already implied by the executed run.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::engine::bank::{BankSourceKind, HeuristicBankRegistry, LoadedBankDescriptor};
use crate::engine::semantics::{
    build_retrieval_index, retrieve_semantics_with_context, SemanticRetrievalContext,
};
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use crate::engine::types::{
    EngineOutputBundle, GrammarState, ResidualTrajectory, ScenarioOutput, SemanticDisposition,
};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use crate::math::smoothing::smooth_residual_trajectory;

/// Machine-readable timeline row describing one structural event in sequence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioEventTimelineRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub scenario_id: String,
    pub step: usize,
    pub time: f64,
    pub layer: String,
    pub event_code: String,
    pub event_label: String,
    pub detail: String,
    pub note: String,
}

/// Prefix-derived semantic and syntax state used by Bearings paper figures and event exports.
#[derive(Clone, Debug)]
pub struct PrefixSemanticTimelinePoint {
    pub step: usize,
    pub time: f64,
    pub syntax_label: String,
    pub grammar_state: GrammarState,
    pub grammar_reason_text: String,
    pub semantic_disposition: SemanticDisposition,
    pub semantic_disposition_code: i32,
    pub top_score: f64,
    pub post_regime_candidate_count: usize,
    pub post_scope_candidate_count: usize,
    pub top_score_margin: f64,
    pub top_candidate_labels: Vec<String>,
}

pub(crate) fn build_prefix_semantic_timeline(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
) -> Result<Vec<PrefixSemanticTimelinePoint>> {
    let registry = load_registry_for_descriptor(&bundle.run_metadata.bank)?;
    let index = build_retrieval_index(
        &registry,
        &bundle.run_metadata.engine_settings.retrieval_index,
    );
    let mut timeline = Vec::with_capacity(scenario.residual.samples.len());

    for end in 0..scenario.residual.samples.len() {
        let residual_prefix = ResidualTrajectory {
            scenario_id: scenario.record.id.clone(),
            channel_names: scenario.residual.channel_names.clone(),
            samples: scenario.residual.samples[..=end].to_vec(),
        };
        let smoothed_prefix = smooth_residual_trajectory(
            &residual_prefix,
            &bundle.run_metadata.engine_settings.smoothing,
        );
        let drift_prefix = compute_drift_trajectory(
            &smoothed_prefix,
            bundle.run_metadata.dt,
            &scenario.record.id,
        );
        let slew_prefix = compute_slew_trajectory(
            &smoothed_prefix,
            bundle.run_metadata.dt,
            &scenario.record.id,
        );
        let sign_prefix = construct_signs(&residual_prefix, &drift_prefix, &slew_prefix);
        let grammar_prefix = scenario.grammar[..=end].to_vec();
        let syntax_prefix = characterize_syntax_with_coordination_configured(
            &sign_prefix,
            &grammar_prefix,
            None,
            &bundle.run_metadata.engine_settings.syntax,
        );
        let semantics_prefix = retrieve_semantics_with_context(SemanticRetrievalContext {
            scenario_id: &scenario.record.id,
            syntax: &syntax_prefix,
            grammar: &grammar_prefix,
            coordinated: None,
            registry: &registry,
            settings: &bundle.run_metadata.engine_settings.semantics,
            index_settings: &bundle.run_metadata.engine_settings.retrieval_index,
            index: Some(&index),
        });
        let top_score_margin = semantics_prefix
            .retrieval_audit
            .ranked_candidates_post_regime
            .first()
            .map(|candidate| candidate.score)
            .unwrap_or(0.0)
            - semantics_prefix
                .retrieval_audit
                .ranked_candidates_post_regime
                .get(1)
                .map(|candidate| candidate.score)
                .unwrap_or(0.0);
        let top_score = semantics_prefix
            .retrieval_audit
            .ranked_candidates_post_regime
            .first()
            .map(|candidate| candidate.score)
            .unwrap_or(0.0);
        let grammar_status = grammar_prefix
            .last()
            .ok_or_else(|| anyhow!("prefix grammar was unexpectedly empty"))?;
        timeline.push(PrefixSemanticTimelinePoint {
            step: grammar_status.step,
            time: grammar_status.time,
            syntax_label: syntax_prefix.trajectory_label,
            grammar_state: grammar_status.state,
            grammar_reason_text: grammar_status.reason_text.clone(),
            semantic_disposition_code: crate::figures::source::semantic_disposition_code(
                &semantics_prefix.disposition,
            ),
            semantic_disposition: semantics_prefix.disposition,
            top_score,
            post_regime_candidate_count: semantics_prefix
                .retrieval_audit
                .heuristic_candidates_post_regime,
            post_scope_candidate_count: semantics_prefix
                .retrieval_audit
                .heuristic_candidates_post_scope,
            top_score_margin,
            top_candidate_labels: semantics_prefix
                .retrieval_audit
                .ranked_candidates_post_regime
                .iter()
                .take(3)
                .map(|candidate| candidate.short_label.clone())
                .collect(),
        });
    }

    Ok(timeline)
}

pub(crate) fn build_scenario_event_timeline(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
) -> Result<Vec<ScenarioEventTimelineRow>> {
    let mut rows = Vec::new();
    let prefix_timeline = build_prefix_semantic_timeline(bundle, scenario)?;

    let mut previous_grammar_key = None::<(GrammarState, String)>;
    for status in &scenario.grammar {
        let key = (status.state, status.reason_text.clone());
        if previous_grammar_key.as_ref() != Some(&key) {
            rows.push(ScenarioEventTimelineRow {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                step: status.step,
                time: status.time,
                layer: "grammar".to_string(),
                event_code: format!("{:?}", status.reason_code).to_lowercase(),
                event_label: format!("{:?}", status.state),
                detail: status.reason_text.clone(),
                note: "Grammar event emitted when the state or reason text changes along the executed trajectory.".to_string(),
            });
            previous_grammar_key = Some(key);
        }
    }

    let mut previous_syntax_label = None::<String>;
    let mut previous_semantic_key = None::<(i32, String)>;
    for point in prefix_timeline {
        if previous_syntax_label.as_ref() != Some(&point.syntax_label) {
            rows.push(ScenarioEventTimelineRow {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                step: point.step,
                time: point.time,
                layer: "syntax".to_string(),
                event_code: point.syntax_label.replace('-', "_"),
                event_label: point.syntax_label.clone(),
                detail: format!(
                    "grammar={:?}, reason={}",
                    point.grammar_state, point.grammar_reason_text
                ),
                note: "Syntax event emitted when the prefix-derived syntax headline changes."
                    .to_string(),
            });
            previous_syntax_label = Some(point.syntax_label.clone());
        }

        let semantic_detail = if point.top_candidate_labels.is_empty() {
            "no ranked post-regime candidates".to_string()
        } else {
            format!(
                "top_candidates={}, post_regime_count={}, post_scope_count={}, top_score_margin={:.6}",
                point.top_candidate_labels.join("|"),
                point.post_regime_candidate_count,
                point.post_scope_candidate_count,
                point.top_score_margin
            )
        };
        let semantic_key = (point.semantic_disposition_code, semantic_detail.clone());
        if previous_semantic_key.as_ref() != Some(&semantic_key) {
            rows.push(ScenarioEventTimelineRow {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                step: point.step,
                time: point.time,
                layer: "semantics".to_string(),
                event_code: format!("{:?}", point.semantic_disposition).to_lowercase(),
                event_label: format!("{:?}", point.semantic_disposition),
                detail: semantic_detail,
                note: "Semantic event emitted when the prefix-derived disposition or ranked-candidate summary changes.".to_string(),
            });
            previous_semantic_key = Some(semantic_key);
        }
    }

    for result in bundle
        .evaluation
        .baseline_results
        .iter()
        .filter(|result| result.scenario_id == scenario.record.id)
        .filter(|result| result.triggered)
    {
        rows.push(ScenarioEventTimelineRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            scenario_id: scenario.record.id.clone(),
            step: result.first_trigger_step.unwrap_or_default(),
            time: result.first_trigger_time.unwrap_or_default(),
            layer: "comparator".to_string(),
            event_code: result.comparator_id.clone(),
            event_label: result.comparator_id.clone(),
            detail: result.comparator_summary.clone(),
            note: "Comparator event emitted at the first alarm time for the internal deterministic baseline comparator.".to_string(),
        });
    }

    rows.sort_by(|left, right| {
        left.time
            .partial_cmp(&right.time)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.layer.cmp(&right.layer))
            .then_with(|| left.event_code.cmp(&right.event_code))
    });
    Ok(rows)
}

fn load_registry_for_descriptor(
    descriptor: &LoadedBankDescriptor,
) -> Result<HeuristicBankRegistry> {
    match descriptor.source_kind {
        BankSourceKind::Builtin => Ok(HeuristicBankRegistry::builtin()),
        BankSourceKind::External => {
            let path = descriptor
                .source_path
                .as_deref()
                .ok_or_else(|| anyhow!("external bank descriptor did not record a source path"))?;
            let path = Path::new(path);
            #[cfg(feature = "external-bank")]
            {
                let (registry, _, _) =
                    HeuristicBankRegistry::load_external_json(path, descriptor.strict_validation)?;
                Ok(registry)
            }
            #[cfg(not(feature = "external-bank"))]
            {
                let _ = path;
                Err(anyhow!(
                    "external-bank support is required to rebuild semantic timelines from an external bank descriptor"
                ))
            }
        }
    }
}
