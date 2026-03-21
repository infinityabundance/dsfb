//! Replay-event construction helpers for dashboard surfaces.

use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use crate::engine::types::{EngineOutputBundle, GrammarState, ScenarioOutput};
use crate::evaluation::types::{BaselineComparatorResult, ScenarioEvaluationSummary};

use super::types::{
    DashboardReplayConfig, DashboardReplayEvent, DashboardReplayStream,
    DASHBOARD_EVENT_SCHEMA_VERSION,
};

pub(crate) fn build_streams_from_bundle(
    bundle: &EngineOutputBundle,
    config: &DashboardReplayConfig,
) -> Result<Vec<DashboardReplayStream>> {
    bundle
        .scenario_outputs
        .iter()
        .filter(|scenario| {
            config
                .scenario_filter
                .as_ref()
                .map(|filter| &scenario.record.id == filter)
                .unwrap_or(true)
        })
        .map(|scenario| build_stream(bundle, scenario, config))
        .collect()
}

pub(crate) fn build_stream(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
    config: &DashboardReplayConfig,
) -> Result<DashboardReplayStream> {
    let evaluation = bundle
        .evaluation
        .scenario_evaluations
        .iter()
        .find(|item| item.scenario_id == scenario.record.id)
        .ok_or_else(|| anyhow!("missing scenario evaluation for `{}`", scenario.record.id))?;
    let baselines = bundle
        .evaluation
        .baseline_results
        .iter()
        .filter(|item| item.scenario_id == scenario.record.id)
        .cloned()
        .collect::<Vec<_>>();

    Ok(DashboardReplayStream {
        schema_version: DASHBOARD_EVENT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        input_mode: bundle.run_metadata.input_mode.clone(),
        scenario_id: scenario.record.id.clone(),
        scenario_title: scenario.record.title.clone(),
        source_label: config
            .source_label
            .clone()
            .unwrap_or_else(|| scenario.record.id.clone()),
        events: build_events(
            bundle,
            scenario,
            evaluation,
            &baselines,
            config.trust_threshold,
        ),
    })
}

pub(crate) fn build_events(
    bundle: &EngineOutputBundle,
    scenario: &ScenarioOutput,
    evaluation: &ScenarioEvaluationSummary,
    baselines: &[BaselineComparatorResult],
    trust_threshold: f64,
) -> Vec<DashboardReplayEvent> {
    let mut log = Vec::new();
    let total_frames = scenario.sign.samples.len();
    let mut previous_state = None;
    let mut previous_syntax = None::<String>;
    let mut previous_reason = None::<String>;
    let mut previous_semantic = None::<String>;
    let mut previously_active_alarms = BTreeSet::new();
    let mut trust_was_below = false;

    scenario
        .sign
        .samples
        .iter()
        .zip(&scenario.grammar)
        .map(|(sample, grammar)| {
            let mut markers = Vec::new();
            if sample.step == 0 {
                let marker = format!("start scenario={} t={:.3}", scenario.record.id, sample.time);
                markers.push(marker.clone());
                log.push(marker);
            }

            let syntax_label = scenario.syntax.trajectory_label.clone();
            if previous_syntax.as_ref() != Some(&syntax_label) {
                let marker = format!("syntax -> {} at step {}", syntax_label, sample.step);
                markers.push(marker.clone());
                log.push(marker);
                previous_syntax = Some(syntax_label.clone());
            }

            if previous_state != Some(grammar.state) {
                let marker = format!(
                    "grammar -> {} at step {}",
                    grammar_state_label(grammar.state),
                    sample.step
                );
                markers.push(marker.clone());
                log.push(marker);
                previous_state = Some(grammar.state);
            }

            if previous_reason.as_deref() != Some(grammar.reason_text.as_str()) {
                let marker = format!(
                    "grammar_reason -> {} at step {}",
                    grammar.reason_text, sample.step
                );
                markers.push(marker.clone());
                log.push(marker);
                previous_reason = Some(grammar.reason_text.clone());
            }

            let semantic_disposition = format!("{:?}", scenario.semantics.disposition);
            if previous_semantic.as_ref() != Some(&semantic_disposition) {
                let marker = format!(
                    "semantic -> {} at step {}",
                    semantic_disposition, sample.step
                );
                markers.push(marker.clone());
                log.push(marker);
                previous_semantic = Some(semantic_disposition.clone());
            }

            for comparator in baselines {
                if comparator.triggered && comparator.first_trigger_step == Some(sample.step) {
                    let marker = format!(
                        "{} alarm at t={:.3}",
                        comparator.comparator_label,
                        comparator.first_trigger_time.unwrap_or(sample.time)
                    );
                    markers.push(marker.clone());
                    log.push(marker);
                }
            }

            let active_alarms = baselines
                .iter()
                .filter(|result| {
                    result.triggered
                        && result
                            .first_trigger_step
                            .map(|step| step <= sample.step)
                            .unwrap_or(false)
                })
                .map(|result| result.comparator_label.clone())
                .collect::<Vec<_>>();
            let active_alarm_set = active_alarms.iter().cloned().collect::<BTreeSet<_>>();
            for alarm in active_alarm_set.difference(&previously_active_alarms) {
                let marker = format!("comparator -> {} active at step {}", alarm, sample.step);
                markers.push(marker.clone());
                log.push(marker);
            }
            previously_active_alarms = active_alarm_set;

            let trust_is_below = grammar.trust_scalar.value() <= trust_threshold;
            if trust_is_below && !trust_was_below {
                let marker = format!(
                    "trust threshold {:.2} crossed at t={:.3}",
                    trust_threshold, sample.time
                );
                markers.push(marker.clone());
                log.push(marker);
            }
            trust_was_below = trust_is_below;

            let recent_log = log.iter().rev().take(4).cloned().collect::<Vec<_>>();
            let mut recent_log = recent_log.into_iter().rev().collect::<Vec<_>>();
            if recent_log.is_empty() {
                recent_log.push("no transitions yet".to_string());
            }

            DashboardReplayEvent {
                schema_version: DASHBOARD_EVENT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                scenario_title: scenario.record.title.clone(),
                frame_index: sample.step,
                total_frames,
                step: sample.step,
                time: sample.time,
                residual_norm: sample.residual_norm,
                drift_norm: sample.drift_norm,
                slew_norm: sample.slew_norm,
                projection_1: sample.projection[0],
                projection_2: sample.projection[1],
                projection_3: sample.projection[2],
                syntax_label,
                grammar_state: grammar_state_label(grammar.state).to_string(),
                grammar_margin: grammar.margin,
                grammar_reason_text: grammar.reason_text.clone(),
                trust_scalar: grammar.trust_scalar.value(),
                semantic_disposition,
                semantic_candidates: joined_candidates(scenario),
                selected_heuristics: joined_selected_heuristics(scenario),
                admissibility_audit: format!(
                    "post_admissibility={} post_scope={} rejected_scope={}",
                    evaluation.heuristic_candidates_post_admissibility,
                    evaluation.heuristic_candidates_post_scope,
                    evaluation.heuristics_rejected_by_scope
                ),
                comparator_alarms: if active_alarms.is_empty() {
                    "none".to_string()
                } else {
                    active_alarms.join(" | ")
                },
                event_markers: markers,
                event_log: recent_log.join(" | "),
            }
        })
        .collect()
}

pub(crate) fn joined_candidates(scenario: &ScenarioOutput) -> String {
    let joined = scenario
        .semantics
        .candidates
        .iter()
        .map(|candidate| candidate.entry.short_label.clone())
        .collect::<Vec<_>>()
        .join(" | ");
    if joined.is_empty() {
        "none".to_string()
    } else {
        joined
    }
}

pub(crate) fn joined_selected_heuristics(scenario: &ScenarioOutput) -> String {
    if scenario.semantics.selected_heuristic_ids.is_empty() {
        "none".to_string()
    } else {
        scenario.semantics.selected_heuristic_ids.join(" | ")
    }
}

pub(crate) fn grammar_state_label(state: GrammarState) -> &'static str {
    match state {
        GrammarState::Admissible => "admissible",
        GrammarState::Boundary => "boundary",
        GrammarState::Violation => "violation",
    }
}
