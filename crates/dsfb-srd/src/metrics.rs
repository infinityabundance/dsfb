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

pub fn low_high_thresholds(thresholds: &[f64]) -> (f64, f64) {
    if thresholds.is_empty() {
        return (0.0, 1.0);
    }
    let last = thresholds.len() - 1;
    let low = thresholds[last / 10];
    let high = thresholds[(last * 9) / 10];
    (low, high)
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
