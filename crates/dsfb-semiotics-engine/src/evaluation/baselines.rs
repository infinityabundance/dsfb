use crate::engine::settings::EvaluationSettings;
use crate::engine::types::{EngineOutputBundle, GrammarState};
use crate::evaluation::types::BaselineComparatorResult;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::metrics::{mean, standard_deviation};

pub fn compute_baseline_results(
    bundle: &EngineOutputBundle,
    settings: &EvaluationSettings,
) -> Vec<BaselineComparatorResult> {
    let mut results = Vec::new();
    for scenario in &bundle.scenario_outputs {
        let residual_norms = scenario
            .residual
            .samples
            .iter()
            .map(|sample| sample.norm)
            .collect::<Vec<_>>();
        let slew_norms = scenario
            .slew
            .samples
            .iter()
            .map(|sample| sample.norm)
            .collect::<Vec<_>>();
        let envelope_radii = scenario
            .envelope
            .samples
            .iter()
            .map(|sample| sample.radius)
            .collect::<Vec<_>>();

        let residual_threshold =
            envelope_radii.first().copied().unwrap_or(0.0) * settings.residual_threshold_scale;
        let residual_crossing = residual_norms
            .iter()
            .enumerate()
            .find(|(_, value)| **value > residual_threshold);
        results.push(BaselineComparatorResult {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            scenario_id: scenario.record.id.clone(),
            comparator_id: "baseline_residual_threshold".to_string(),
            comparator_label: "Residual threshold only".to_string(),
            triggered: residual_crossing.is_some(),
            first_trigger_step: residual_crossing.map(|(index, _)| index),
            first_trigger_time: residual_crossing.map(|(index, _)| scenario.residual.samples[index].time),
            comparator_summary: format!(
                "Triggered when residual norm exceeded the initial envelope radius scaled by {}.",
                settings.residual_threshold_scale
            ),
            distinction_note: "This internal comparator ignores drift, grammar evolution, and typed semantic retrieval.".to_string(),
        });

        let moving_average = moving_average(&residual_norms, settings.moving_average_window);
        let moving_average_trigger = moving_average
            .windows(2)
            .enumerate()
            .find(|(_, window)| window[1] - window[0] > settings.moving_average_trend_deadband);
        results.push(BaselineComparatorResult {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            scenario_id: scenario.record.id.clone(),
            comparator_id: "baseline_moving_average_trend".to_string(),
            comparator_label: "Moving-average residual trend only".to_string(),
            triggered: moving_average_trigger.is_some(),
            first_trigger_step: moving_average_trigger.map(|(index, _)| index + 1),
            first_trigger_time: moving_average_trigger
                .map(|(index, _)| scenario.residual.samples[index + 1].time),
            comparator_summary: format!(
                "Triggered when the residual-norm moving average increased faster than {} over a window of {} samples.",
                settings.moving_average_trend_deadband,
                settings.moving_average_window
            ),
            distinction_note: "This internal comparator ignores admissibility state and multi-layer syntax structure.".to_string(),
        });

        let slew_threshold = (mean(&slew_norms)
            + settings.slew_spike_sigma_factor * standard_deviation(&slew_norms))
        .max(settings.slew_spike_floor);
        let slew_spike = slew_norms
            .iter()
            .enumerate()
            .find(|(_, value)| **value > slew_threshold);
        results.push(BaselineComparatorResult {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            scenario_id: scenario.record.id.clone(),
            comparator_id: "baseline_slew_spike".to_string(),
            comparator_label: "Slew spike only".to_string(),
            triggered: slew_spike.is_some(),
            first_trigger_step: slew_spike.map(|(index, _)| index),
            first_trigger_time: slew_spike.map(|(index, _)| scenario.slew.samples[index].time),
            comparator_summary: format!(
                "Triggered when slew norm exceeded mean + {} sigma with floor {}.",
                settings.slew_spike_sigma_factor,
                settings.slew_spike_floor
            ),
            distinction_note: "This internal comparator ignores residual-norm persistence and envelope interaction.".to_string(),
        });

        let envelope_interaction = scenario
            .grammar
            .iter()
            .find(|status| !matches!(status.state, GrammarState::Admissible));
        results.push(BaselineComparatorResult {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            scenario_id: scenario.record.id.clone(),
            comparator_id: "baseline_envelope_interaction".to_string(),
            comparator_label: "Envelope interaction only".to_string(),
            triggered: envelope_interaction.is_some(),
            first_trigger_step: envelope_interaction.map(|status| status.step),
            first_trigger_time: envelope_interaction.map(|status| status.time),
            comparator_summary: "Triggered when grammar entered Boundary or Violation without using syntax or semantic structure.".to_string(),
            distinction_note: "This internal comparator collapses all boundary interaction into one flag.".to_string(),
        });
    }
    results
}

fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let width = window.max(1);
    (0..values.len())
        .map(|index| {
            let start = index.saturating_sub(width.saturating_sub(1));
            let slice = &values[start..=index];
            slice.iter().sum::<f64>() / slice.len() as f64
        })
        .collect()
}
