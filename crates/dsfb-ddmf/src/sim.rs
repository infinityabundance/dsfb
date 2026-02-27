use dsfb::TrustStats;
use serde::{Deserialize, Serialize};

use crate::disturbances::{build_disturbance, DisturbanceKind};
use crate::envelope::{ResidualEnvelope, TrustWeight};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub n_steps: usize,
    pub rho: f64,
    pub beta: f64,
    pub disturbance_kind: DisturbanceKind,
    pub epsilon_bound: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SimulationResult {
    pub s: Vec<f64>,
    pub w: Vec<f64>,
    pub r: Vec<f64>,
    pub d: Vec<f64>,
}

impl SimulationResult {
    pub fn len(&self) -> usize {
        self.s.len()
    }

    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    pub fn final_trust_stats(&self) -> TrustStats {
        TrustStats {
            residual_ema: *self.s.last().unwrap_or(&0.0),
            weight: *self.w.last().unwrap_or(&1.0),
        }
    }
}

pub fn run_simulation(config: &SimulationConfig) -> SimulationResult {
    run_simulation_with_s0(config, 0.0)
}

pub fn run_simulation_with_s0(config: &SimulationConfig, s0: f64) -> SimulationResult {
    simulate_channel(config, s0, 0, &config.disturbance_kind)
}

pub fn run_multichannel_simulation(
    config: &SimulationConfig,
    n_channels: usize,
    group_assignments: Option<&[usize]>,
    correlated_groups: bool,
) -> Vec<SimulationResult> {
    assert!(n_channels > 0, "n_channels must be > 0");

    if let Some(groups) = group_assignments {
        assert_eq!(
            groups.len(),
            n_channels,
            "group_assignments length must match n_channels",
        );
    }

    let default_groups: Vec<usize> = (0..n_channels).collect();
    let groups = group_assignments.unwrap_or(&default_groups);

    (0..n_channels)
        .map(|channel_idx| {
            let key = if correlated_groups {
                groups[channel_idx]
            } else {
                channel_idx
            };
            let kind = config.disturbance_kind.channelized(key);
            let s0 = 0.02 * key as f64;
            simulate_channel(config, s0, key, &kind)
        })
        .collect()
}

fn simulate_channel(
    config: &SimulationConfig,
    s0: f64,
    channel_key: usize,
    disturbance_kind: &DisturbanceKind,
) -> SimulationResult {
    assert!(config.n_steps > 0, "n_steps must be > 0");
    assert!(
        config.rho > 0.0 && config.rho < 1.0,
        "rho must be in (0, 1)"
    );
    assert!(config.beta > 0.0, "beta must be > 0");
    assert!(
        config.epsilon_bound.is_finite() && config.epsilon_bound >= 0.0,
        "epsilon_bound must be finite and >= 0",
    );

    let mut envelope = ResidualEnvelope::new(config.rho, s0);
    let mut disturbance = build_disturbance(disturbance_kind);
    disturbance.reset();

    let mut result = SimulationResult {
        s: Vec::with_capacity(config.n_steps),
        w: Vec::with_capacity(config.n_steps),
        r: Vec::with_capacity(config.n_steps),
        d: Vec::with_capacity(config.n_steps),
    };

    for n in 0..config.n_steps {
        let d = disturbance.next(n);
        let epsilon = epsilon_at(n, config.epsilon_bound, channel_key);
        let r = epsilon + d;
        let s = envelope.update(r);
        let w = TrustWeight::weight(config.beta, s);

        result.d.push(d);
        result.r.push(r);
        result.s.push(s);
        result.w.push(w);
    }

    result
}

fn epsilon_at(n: usize, epsilon_bound: f64, channel_key: usize) -> f64 {
    if epsilon_bound == 0.0 {
        return 0.0;
    }

    let phase = channel_key as f64 * 0.37;
    let a = (0.17 * n as f64 + phase).sin();
    let b = (0.043 * n as f64 + 0.5 * phase).cos();
    epsilon_bound * (0.6 * a + 0.4 * b)
}

#[cfg(test)]
mod tests {
    use super::{run_multichannel_simulation, run_simulation, SimulationConfig};
    use crate::disturbances::DisturbanceKind;

    #[test]
    fn pointwise_simulation_reaches_plateau() {
        let config = SimulationConfig {
            n_steps: 64,
            rho: 0.95,
            beta: 2.0,
            disturbance_kind: DisturbanceKind::PointwiseBounded { d: 0.4 },
            epsilon_bound: 0.0,
        };

        let result = run_simulation(&config);
        let final_s = *result.s.last().expect("result should be non-empty");
        assert!(final_s > 0.35 && final_s < 0.41);
    }

    #[test]
    fn multichannel_group_correlation_reuses_disturbance() {
        let config = SimulationConfig {
            n_steps: 12,
            rho: 0.9,
            beta: 3.0,
            disturbance_kind: DisturbanceKind::PersistentElevated {
                r_nom: 0.1,
                r_high: 0.5,
                step_time: 4,
            },
            epsilon_bound: 0.0,
        };

        let results = run_multichannel_simulation(&config, 3, Some(&[0, 0, 1]), true);
        assert_eq!(results[0].d, results[1].d);
        assert_ne!(results[0].d, results[2].d);
    }
}
