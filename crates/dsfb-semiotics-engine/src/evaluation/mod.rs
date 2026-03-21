pub mod baselines;
pub mod sweeps;
pub mod types;

use std::collections::BTreeMap;

use crate::engine::bank::{HeuristicBankRegistry, HeuristicBankValidationReport};
use crate::engine::semantics::benchmark_retrieval_scaling;
use crate::engine::settings::{EngineSettings, EvaluationSettings};
use crate::engine::sign_layer::construct_signs;
use crate::engine::syntax_layer::characterize_syntax_with_coordination_configured;
use crate::engine::types::{EngineOutputBundle, GrammarState, SemanticDisposition};
use crate::evaluation::baselines::compute_baseline_results;
use crate::evaluation::types::{
    ArtifactCompletenessCheck, RetrievalLatencyRecord, RunEvaluationBundle, RunEvaluationSummary,
    ScenarioEvaluationSummary, SmoothingComparisonRecord, SweepPointResult, SweepRunSummary,
};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};

/// Evaluates a completed engine bundle with deterministic post-run summaries and internal
/// deterministic comparators.
pub fn evaluate_bundle(
    bundle: &EngineOutputBundle,
    settings: &EvaluationSettings,
    engine_settings: &EngineSettings,
    bank_registry: &HeuristicBankRegistry,
    bank_validation: &HeuristicBankValidationReport,
    sweep_summary: Option<SweepRunSummary>,
) -> RunEvaluationBundle {
    let engine_version = bundle.run_metadata.crate_version.clone();
    let bank_version = bundle.run_metadata.bank.bank_version.clone();
    let baseline_results = compute_baseline_results(bundle, settings);
    let mut baseline_trigger_counts = BTreeMap::new();
    for result in &baseline_results {
        *baseline_trigger_counts
            .entry(result.comparator_id.clone())
            .or_insert(0usize) += usize::from(result.triggered);
    }

    let reproducibility_lookup = bundle
        .reproducibility_checks
        .iter()
        .map(|check| (check.scenario_id.as_str(), check.identical))
        .collect::<BTreeMap<_, _>>();
    let mut semantic_disposition_counts = BTreeMap::new();
    let mut syntax_label_counts = BTreeMap::new();
    let mut boundary_interaction_count = 0usize;
    let mut violation_count = 0usize;
    let mut minimum_trust_scalar = 1.0f64;

    let scenario_evaluations = bundle
        .scenario_outputs
        .iter()
        .map(|scenario| {
            let worst_grammar = scenario
                .grammar
                .iter()
                .min_by(|left, right| {
                    left.trust_scalar
                        .value()
                        .total_cmp(&right.trust_scalar.value())
                })
                .or_else(|| scenario.grammar.last())
                .expect("scenario grammar should be populated");
            let boundary_samples = scenario
                .grammar
                .iter()
                .filter(|status| matches!(status.state, GrammarState::Boundary))
                .count();
            let violation_samples = scenario
                .grammar
                .iter()
                .filter(|status| matches!(status.state, GrammarState::Violation))
                .count();
            let first_boundary_time = scenario
                .grammar
                .iter()
                .find(|status| matches!(status.state, GrammarState::Boundary))
                .map(|status| status.time);
            let first_violation_time = scenario
                .grammar
                .iter()
                .find(|status| matches!(status.state, GrammarState::Violation))
                .map(|status| status.time);
            let triggered_baselines = baseline_results
                .iter()
                .filter(|result| result.scenario_id == scenario.record.id && result.triggered)
                .count();
            *semantic_disposition_counts
                .entry(format!("{:?}", scenario.semantics.disposition))
                .or_insert(0usize) += 1;
            *syntax_label_counts
                .entry(scenario.syntax.trajectory_label.clone())
                .or_insert(0usize) += 1;
            if boundary_samples > 0 {
                boundary_interaction_count += 1;
            }
            if violation_samples > 0 {
                violation_count += 1;
            }
            minimum_trust_scalar = minimum_trust_scalar.min(worst_grammar.trust_scalar.value());

            ScenarioEvaluationSummary {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: engine_version.clone(),
                bank_version: bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                input_mode: bundle.run_metadata.input_mode.clone(),
                syntax_label: scenario.syntax.trajectory_label.clone(),
                semantic_disposition: format!("{:?}", scenario.semantics.disposition),
                selected_heuristic_ids: scenario.semantics.selected_heuristic_ids.clone(),
                grammar_reason_code: format!("{:?}", worst_grammar.reason_code),
                grammar_reason_text: worst_grammar.reason_text.clone(),
                trust_scalar: worst_grammar.trust_scalar.value(),
                heuristic_bank_entry_count: scenario
                    .semantics
                    .retrieval_audit
                    .heuristic_bank_entry_count,
                heuristic_candidates_post_admissibility: scenario
                    .semantics
                    .retrieval_audit
                    .heuristic_candidates_post_admissibility,
                heuristic_candidates_post_regime: scenario
                    .semantics
                    .retrieval_audit
                    .heuristic_candidates_post_regime,
                heuristic_candidates_pre_scope: scenario
                    .semantics
                    .retrieval_audit
                    .heuristic_candidates_pre_scope,
                heuristic_candidates_post_scope: scenario
                    .semantics
                    .retrieval_audit
                    .heuristic_candidates_post_scope,
                heuristics_rejected_by_admissibility: scenario
                    .semantics
                    .retrieval_audit
                    .heuristics_rejected_by_admissibility,
                heuristics_rejected_by_regime: scenario
                    .semantics
                    .retrieval_audit
                    .heuristics_rejected_by_regime,
                heuristics_rejected_by_scope: scenario
                    .semantics
                    .retrieval_audit
                    .heuristics_rejected_by_scope,
                heuristics_selected_final: scenario
                    .semantics
                    .retrieval_audit
                    .heuristics_selected_final,
                boundary_sample_count: boundary_samples,
                violation_sample_count: violation_samples,
                first_boundary_time,
                first_violation_time,
                reproducible: *reproducibility_lookup
                    .get(scenario.record.id.as_str())
                    .unwrap_or(&false),
                triggered_baseline_count: triggered_baselines,
                unknown_reason_class: scenario.semantics.unknown_reason_class.clone(),
                note: match scenario.semantics.disposition {
                    SemanticDisposition::Unknown => "Unknown remains explicit when evidence is limited or bank coverage is intentionally absent.".to_string(),
                    SemanticDisposition::Ambiguous => "Ambiguity remains explicit when the typed bank does not authorize a unique compatible set.".to_string(),
                    SemanticDisposition::CompatibleSet => "CompatibleSet indicates explicitly bank-compatible motif coexistence under the sampled evidence.".to_string(),
                    SemanticDisposition::Match => "Match indicates one typed bank entry remained after admissibility, regime, and scope checks.".to_string(),
                },
            }
        })
        .collect::<Vec<_>>();
    let (sweep_results, computed_sweep_summary) = summarize_sweep(bundle);
    let sweep_summary = sweep_summary.or(computed_sweep_summary);

    let summary = RunEvaluationSummary {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: engine_version.clone(),
        bank_version: bank_version.clone(),
        evaluation_version: "evaluation/v1".to_string(),
        input_mode: bundle.run_metadata.input_mode.clone(),
        scenario_count: bundle.scenario_outputs.len(),
        semantic_disposition_counts,
        syntax_label_counts,
        boundary_interaction_count,
        violation_count,
        comparator_trigger_counts: baseline_trigger_counts,
        reproducible_scenario_count: bundle
            .reproducibility_checks
            .iter()
            .filter(|check| check.identical)
            .count(),
        all_reproducible: bundle.reproducibility_summary.all_identical,
        minimum_trust_scalar,
        note: "Evaluation summaries are deterministic post-run summaries over engine outputs and internal deterministic comparators. They are not field benchmarks.".to_string(),
    };
    let smoothing_comparison_report = build_smoothing_comparison_report(bundle, engine_settings);
    let retrieval_latency_report =
        build_retrieval_latency_report(bundle, bank_registry, engine_settings);

    RunEvaluationBundle {
        summary,
        scenario_evaluations,
        baseline_results,
        smoothing_comparison_report,
        retrieval_latency_report,
        bank_validation: bank_validation.clone(),
        artifact_completeness: None,
        sweep_results,
        sweep_summary,
    }
}

/// Adds a post-export artifact completeness record to an already computed evaluation bundle.
pub fn with_artifact_completeness(
    mut evaluation: RunEvaluationBundle,
    completeness: ArtifactCompletenessCheck,
) -> RunEvaluationBundle {
    evaluation.artifact_completeness = Some(completeness);
    evaluation
}

fn summarize_sweep(
    bundle: &EngineOutputBundle,
) -> (Vec<SweepPointResult>, Option<SweepRunSummary>) {
    let mut results = bundle
        .scenario_outputs
        .iter()
        .filter_map(|scenario| {
            Some(SweepPointResult {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                sweep_family: scenario.record.sweep_family.clone()?,
                scenario_id: scenario.record.id.clone(),
                parameter_name: scenario.record.sweep_parameter_name.clone()?,
                parameter_value: scenario.record.sweep_parameter_value?,
                secondary_parameter_name: scenario.record.sweep_secondary_parameter_name.clone(),
                secondary_parameter_value: scenario.record.sweep_secondary_parameter_value,
                syntax_label: scenario.syntax.trajectory_label.clone(),
                semantic_disposition: format!("{:?}", scenario.semantics.disposition),
                selected_heuristic_ids: scenario.semantics.selected_heuristic_ids.clone(),
                note: "Deterministic sweep member summary derived from the same layered engine outputs as ordinary scenario runs.".to_string(),
            })
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        left.parameter_value
            .total_cmp(&right.parameter_value)
            .then_with(|| left.scenario_id.cmp(&right.scenario_id))
    });
    if results.is_empty() {
        return (results, None);
    }

    let sweep_family = results[0].sweep_family.clone();
    let unique_syntax_labels = results
        .iter()
        .map(|result| result.syntax_label.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unique_semantic_dispositions = results
        .iter()
        .map(|result| result.semantic_disposition.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unknown_count = results
        .iter()
        .filter(|result| result.semantic_disposition == "Unknown")
        .count();
    let ambiguous_count = results
        .iter()
        .filter(|result| result.semantic_disposition == "Ambiguous")
        .count();
    let disposition_flip_count = results
        .windows(2)
        .filter(|window| window[0].semantic_disposition != window[1].semantic_disposition)
        .count();
    (
        results.clone(),
        Some(SweepRunSummary {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: bundle.run_metadata.crate_version.clone(),
            bank_version: bundle.run_metadata.bank.bank_version.clone(),
            sweep_family,
            member_count: results.len(),
            unique_syntax_labels,
            unique_semantic_dispositions,
            unknown_count,
            ambiguous_count,
            disposition_flip_count,
            note: "Sweep summaries report deterministic semantic and syntax transitions across a configured synthetic parameter family. They are internal calibration-style summaries, not field benchmarks.".to_string(),
        }),
    )
}

fn build_smoothing_comparison_report(
    bundle: &EngineOutputBundle,
    engine_settings: &EngineSettings,
) -> Vec<SmoothingComparisonRecord> {
    bundle
        .scenario_outputs
        .iter()
        .map(|scenario| {
            let raw_drift = compute_drift_trajectory(
                &scenario.residual,
                bundle.run_metadata.dt,
                &scenario.record.id,
            );
            let raw_slew = compute_slew_trajectory(
                &scenario.residual,
                bundle.run_metadata.dt,
                &scenario.record.id,
            );
            let raw_sign = construct_signs(&scenario.residual, &raw_drift, &raw_slew);
            let raw_syntax = characterize_syntax_with_coordination_configured(
                &raw_sign,
                &scenario.grammar,
                scenario.coordinated.as_ref(),
                &engine_settings.syntax,
            );
            SmoothingComparisonRecord {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                engine_version: bundle.run_metadata.crate_version.clone(),
                bank_version: bundle.run_metadata.bank.bank_version.clone(),
                scenario_id: scenario.record.id.clone(),
                smoothing_profile: engine_settings.smoothing.profile_label().to_string(),
                smoothing_mode: engine_settings.smoothing.mode.as_label().to_string(),
                smoothing_enabled: engine_settings.smoothing.enabled(),
                smoothing_alpha: engine_settings.smoothing.exponential_alpha,
                causal_window: engine_settings.smoothing.causal_window,
                estimated_lag_samples: engine_settings.smoothing.estimated_lag_samples(),
                maximum_settling_samples: engine_settings.smoothing.maximum_settling_samples(),
                raw_mean_squared_slew_norm: raw_syntax.mean_squared_slew_norm,
                active_mean_squared_slew_norm: scenario.syntax.mean_squared_slew_norm,
                raw_max_slew_norm: raw_syntax.max_slew_norm,
                active_max_slew_norm: scenario.syntax.max_slew_norm,
                raw_slew_spike_count: raw_syntax.slew_spike_count,
                active_slew_spike_count: scenario.syntax.slew_spike_count,
                raw_syntax_label: raw_syntax.trajectory_label,
                active_syntax_label: scenario.syntax.trajectory_label.clone(),
                note: "Raw and active syntax metrics are compared using the same grammar trajectory so smoothing effects stay isolated to derivative estimation rather than envelope evaluation.".to_string(),
            }
        })
        .collect()
}

fn build_retrieval_latency_report(
    bundle: &EngineOutputBundle,
    bank_registry: &HeuristicBankRegistry,
    engine_settings: &EngineSettings,
) -> Vec<RetrievalLatencyRecord> {
    if !engine_settings.retrieval_index.export_latency_report {
        return Vec::new();
    }
    let Some(reference) = bundle.scenario_outputs.first() else {
        return Vec::new();
    };
    benchmark_retrieval_scaling(
        &reference.syntax,
        &reference.grammar,
        reference.coordinated.as_ref(),
        bank_registry,
        &engine_settings.semantics,
        &engine_settings.retrieval_index,
    )
    .into_iter()
    .map(|observation| RetrievalLatencyRecord {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        bank_size: observation.bank_size,
        retrieval_path: observation.retrieval_path,
        linear_candidates_considered: observation.linear_candidates_considered,
        indexed_prefilter_candidate_count: observation.indexed_prefilter_candidate_count,
        indexed_post_scope_candidate_count: observation.indexed_post_scope_candidate_count,
        index_buckets_considered: observation.index_buckets_considered,
        note: observation.note,
    })
    .collect()
}
