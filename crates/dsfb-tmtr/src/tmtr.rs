use serde::Serialize;

use crate::config::SimulationConfig;
use crate::kernel::evaluate_kernel;
use crate::observer::{ObserverSeries, ObserverSpec};
use crate::scenario::ScenarioDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct CorrectionEvent {
    pub scenario: String,
    pub source_level: usize,
    pub target_level: usize,
    pub anchor_time: usize,
    pub corrected_time: usize,
    pub delta_window: usize,
    pub trust_weight: f64,
    pub kernel_weight: f64,
    pub compatibility: f64,
    pub correction_magnitude: f64,
    pub recursion_depth: usize,
    pub iteration: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecursionStats {
    pub total_correction_events: usize,
    pub max_recursion_depth: usize,
    pub mean_recursion_depth: f64,
    pub convergence_iterations: usize,
    pub average_correction_magnitude: f64,
    pub average_correction_trust_weight: f64,
    pub monotonicity_violations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TmtrResult {
    pub observers: Vec<ObserverSeries>,
    pub correction_events: Vec<CorrectionEvent>,
    pub recursion_stats: RecursionStats,
}

pub fn apply_tmtr(
    definition: &ScenarioDefinition,
    config: &SimulationConfig,
    specs: &[ObserverSpec],
    baseline: &[ObserverSeries],
    truth: &[f64],
) -> TmtrResult {
    let mut observers = baseline.to_vec();
    let mut events = Vec::new();
    let mut convergence_iterations = 0;

    for iteration in 0..config.max_iterations {
        convergence_iterations = iteration + 1;
        let mut iteration_change = 0.0;
        let mut depth_reached = vec![0usize; observers.len()];

        for source_index in (1..observers.len()).rev() {
            let target_index = source_index - 1;
            let recursion_depth = 1 + depth_reached[source_index];
            if recursion_depth > config.max_recursion_depth {
                continue;
            }

            let mut pair_changed = false;
            let source = observers[source_index].clone();
            let target = &mut observers[target_index];
            let eta = definition.eta[target_index];
            let delta = definition.delta.min(config.delta);

            for anchor_time in 0..truth.len() {
                let source_trust = source.trust[anchor_time];
                let target_trust = target.trust[anchor_time];
                if !source.available[anchor_time] {
                    continue;
                }
                if source_trust < config.trust_threshold
                    || source_trust <= target_trust + config.min_trust_gap
                {
                    continue;
                }

                let window_start = anchor_time.saturating_sub(delta);
                for corrected_time in window_start..=anchor_time {
                    let evaluation = evaluate_kernel(
                        config.kernel,
                        &source,
                        target,
                        corrected_time,
                        anchor_time,
                        delta,
                        definition.resonance_threshold,
                    );
                    let retro_weight =
                        1.0 - (anchor_time - corrected_time) as f64 / (delta.max(1) as f64 + 1.0);
                    let correction =
                        eta * source_trust * retro_weight.max(0.05) * evaluation.signal;
                    if correction.abs() < config.convergence_tolerance * 0.1 {
                        continue;
                    }
                    target.estimate[corrected_time] += correction;
                    iteration_change += correction.abs();
                    pair_changed = true;
                    events.push(CorrectionEvent {
                        scenario: definition.name.clone(),
                        source_level: source.level,
                        target_level: target.level,
                        anchor_time,
                        corrected_time,
                        delta_window: delta,
                        trust_weight: source_trust,
                        kernel_weight: evaluation.weight,
                        compatibility: evaluation.compatibility,
                        correction_magnitude: correction,
                        recursion_depth,
                        iteration: iteration + 1,
                    });
                }
            }

            if pair_changed {
                depth_reached[target_index] = recursion_depth;
                target.recompute_after_estimate_update(truth, &specs[target_index]);
            }
        }

        if should_stop(iteration_change, config.convergence_tolerance) {
            break;
        }
    }

    let total_events = events.len();
    let depth_sum = events
        .iter()
        .map(|event| event.recursion_depth as f64)
        .sum::<f64>();
    let magnitude_sum = events
        .iter()
        .map(|event| event.correction_magnitude.abs())
        .sum::<f64>();
    let trust_sum = events.iter().map(|event| event.trust_weight).sum::<f64>();
    let max_depth = events
        .iter()
        .map(|event| event.recursion_depth)
        .max()
        .unwrap_or(0);

    TmtrResult {
        observers,
        correction_events: events,
        recursion_stats: RecursionStats {
            total_correction_events: total_events,
            max_recursion_depth: max_depth,
            mean_recursion_depth: if total_events == 0 {
                0.0
            } else {
                depth_sum / total_events as f64
            },
            convergence_iterations,
            average_correction_magnitude: if total_events == 0 {
                0.0
            } else {
                magnitude_sum / total_events as f64
            },
            average_correction_trust_weight: if total_events == 0 {
                0.0
            } else {
                trust_sum / total_events as f64
            },
            monotonicity_violations: 0,
        },
    }
}

pub fn should_stop(total_change: f64, tolerance: f64) -> bool {
    total_change <= tolerance
}

#[cfg(test)]
mod tests {
    use super::should_stop;

    #[test]
    fn bounded_recursion_stop_condition_triggers() {
        assert!(should_stop(0.0005, 0.001));
        assert!(!should_stop(0.01, 0.001));
    }
}
