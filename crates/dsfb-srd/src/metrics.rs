use crate::config::SimulationConfig;
use crate::event::{RegimeLabel, StructuralEvent};

pub fn discrete_derivative(points: &[(f64, f64)]) -> Vec<(f64, f64, f64)> {
    let mut rows = Vec::new();

    for pair in points.windows(2) {
        let (tau_left, rho_left) = pair[0];
        let (tau_right, rho_right) = pair[1];
        let delta_tau = tau_right - tau_left;
        if delta_tau.abs() <= f64::EPSILON {
            continue;
        }
        let derivative = (rho_right - rho_left) / delta_tau;
        rows.push(((tau_left + tau_right) * 0.5, derivative, derivative.abs()));
    }

    rows
}

pub fn component_entropy(component_sizes: &[usize], total_nodes: usize) -> f64 {
    if total_nodes == 0 {
        return 0.0;
    }

    component_sizes
        .iter()
        .copied()
        .filter(|&size| size > 0)
        .map(|size| {
            let probability = size as f64 / total_nodes as f64;
            -probability * probability.ln()
        })
        .sum()
}

pub fn low_high_thresholds(thresholds: &[f64]) -> (f64, f64) {
    if thresholds.is_empty() {
        return (0.0, 1.0);
    }
    let last = thresholds.len() - 1;
    let low = thresholds[last / 10];
    let high = thresholds[(last * 9) / 10];
    (low, high)
}

pub fn nearest_threshold(thresholds: &[f64], target: f64) -> f64 {
    thresholds
        .iter()
        .copied()
        .min_by(|left, right| {
            let left_distance = (left - target).abs();
            let right_distance = (right - target).abs();
            left_distance
                .partial_cmp(&right_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(target)
}

pub fn window_ranges(config: &SimulationConfig) -> Vec<(usize, usize)> {
    let window_size = suggested_window_size(config);
    if window_size >= config.n_events {
        return vec![(0, config.n_events)];
    }

    let step = suggested_window_step(config, window_size);
    let mut ranges = Vec::new();
    let mut start = 0usize;

    while start + window_size <= config.n_events {
        ranges.push((start, start + window_size));
        start += step;
    }

    if ranges.last().map(|(_, end)| *end).unwrap_or(0) < config.n_events {
        ranges.push((config.n_events - window_size, config.n_events));
    }

    ranges
}

pub fn window_regime(
    events: &[StructuralEvent],
    window_start: usize,
    window_end: usize,
) -> RegimeLabel {
    let midpoint = window_start + (window_end - window_start) / 2;
    events[midpoint].regime_label
}

fn suggested_window_size(config: &SimulationConfig) -> usize {
    let causal_scale = config.causal_window * 8;
    let history_scale = config.n_events / 12;
    causal_scale.max(history_scale).max(48).min(config.n_events)
}

fn suggested_window_step(config: &SimulationConfig, window_size: usize) -> usize {
    (window_size / 4).max(config.causal_window.max(1))
}

#[cfg(test)]
mod tests {
    use super::{component_entropy, nearest_threshold};

    #[test]
    fn component_entropy_is_zero_for_single_component() {
        assert!((component_entropy(&[10], 10) - 0.0).abs() <= f64::EPSILON);
    }

    #[test]
    fn component_entropy_matches_log_n_for_singletons() {
        let entropy = component_entropy(&[1, 1, 1, 1], 4);
        assert!((entropy - 4.0_f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn nearest_threshold_prefers_smallest_distance() {
        let thresholds = vec![0.0, 0.1, 0.2, 0.3];
        assert!((nearest_threshold(&thresholds, 0.26) - 0.3).abs() < 1e-12);
        assert!((nearest_threshold(&thresholds, 0.24) - 0.2).abs() < 1e-12);
    }
}
