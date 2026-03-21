use crate::engine::types::{
    DetectabilityBoundInputs, DetectabilityResult, GrammarState, GrammarStatus, ResidualTrajectory,
};

// TRACE:ALGORITHM:ALG-FIRST-EXIT:First envelope exit detection:Finds the earliest grammar violation sample used by detectability reporting.
pub fn first_exit_index(grammar: &[GrammarStatus]) -> Option<usize> {
    grammar
        .iter()
        .position(|status| matches!(status.state, GrammarState::Violation))
}

// TRACE:THEOREM:THM-DETECTABILITY-BOUND:Configured detectability upper bound:Computes theorem-aligned exit-time summaries when explicit bound assumptions are attached.
pub fn compute_detectability_result(
    scenario_id: &str,
    residual: &ResidualTrajectory,
    grammar: &[GrammarStatus],
    bound_inputs: Option<DetectabilityBoundInputs>,
    reference: Option<&ResidualTrajectory>,
) -> DetectabilityResult {
    let first_exit = first_exit_index(grammar);
    let observed_crossing_time = first_exit.map(|index| residual.samples[index].time);
    let observed_crossing_step = first_exit.map(|index| residual.samples[index].step);
    let sampling_dt = residual
        .samples
        .windows(2)
        .next()
        .map(|window| window[1].time - window[0].time)
        .unwrap_or(0.0);
    let separation_at_exit = match (first_exit, reference) {
        (Some(index), Some(reference)) if index < reference.samples.len() => {
            Some(residual.samples[index].norm - reference.samples[index].norm)
        }
        _ => None,
    };

    let (predicted_upper_bound, bound_satisfied) = match (bound_inputs, observed_crossing_time) {
        (Some(inputs), Some(observed)) if inputs.alpha > inputs.kappa => {
            let upper_bound = inputs.delta0 / (inputs.alpha - inputs.kappa);
            (
                Some(upper_bound),
                Some(observed - inputs.t0 <= upper_bound + sampling_dt + 1.0e-9),
            )
        }
        (Some(inputs), None) if inputs.alpha > inputs.kappa => (
            Some(inputs.delta0 / (inputs.alpha - inputs.kappa)),
            Some(false),
        ),
        _ => (None, None),
    };

    DetectabilityResult {
        scenario_id: scenario_id.to_string(),
        observed_crossing_step,
        observed_crossing_time,
        predicted_upper_bound,
        bound_satisfied,
        separation_at_exit,
        note: match (predicted_upper_bound, observed_crossing_time) {
            (Some(_), Some(_)) => {
                "Theorem-aligned bound applied under configured outward-drift assumptions, with one-sample tolerance for discrete observation of the first crossing."
                    .to_string()
            }
            (Some(_), None) => {
                "Configured bound assumptions were present, but no envelope exit occurred in the sampled horizon."
                    .to_string()
            }
            _ => "No explicit bound assumptions were attached to this scenario.".to_string(),
        },
    }
}
