use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;

use crate::disturbances::DisturbanceKind;
use crate::sim::{run_simulation_with_s0, SimulationConfig, SimulationResult};

pub const DEFAULT_MONTE_CARLO_RUNS: usize = 360;

#[derive(Clone, Debug)]
pub struct MonteCarloConfig {
    pub n_runs: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub rho: f64,
    pub beta: f64,
    pub epsilon_bound: f64,
    pub recovery_delta: f64,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            n_runs: DEFAULT_MONTE_CARLO_RUNS,
            n_steps: 180,
            seed: 2026,
            rho: 0.96,
            beta: 3.0,
            epsilon_bound: 0.0,
            recovery_delta: 0.03,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MonteCarloRunRecord {
    pub run_id: usize,
    pub regime_label: String,
    pub disturbance_type: String,
    #[serde(rename = "D")]
    pub d: f64,
    #[serde(rename = "B")]
    pub b: f64,
    #[serde(rename = "S")]
    pub s: f64,
    pub impulse_start: usize,
    pub impulse_len: usize,
    pub s0: f64,
    pub max_envelope: f64,
    pub min_trust: f64,
    pub time_to_recover: i64,
}

#[derive(Clone, Debug)]
pub struct MonteCarloBatch {
    pub records: Vec<MonteCarloRunRecord>,
    pub example_impulse: SimulationResult,
    pub example_persistent: SimulationResult,
}

#[derive(Clone, Debug, Serialize)]
pub struct MonteCarloSummary {
    pub n_runs: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub rho: f64,
    pub beta: f64,
    pub epsilon_bound: f64,
    pub recovery_delta: f64,
    pub mean_max_envelope: f64,
    pub min_observed_trust: f64,
    pub regime_counts: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrajectoryRow {
    pub n: usize,
    pub r: f64,
    pub d: f64,
    pub s: f64,
    pub w: f64,
}

pub fn run_monte_carlo(config: &MonteCarloConfig) -> MonteCarloBatch {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut records = Vec::with_capacity(config.n_runs);

    for run_id in 0..config.n_runs {
        let disturbance_kind = sample_disturbance(&mut rng, config.n_steps);
        let s0 = rng.gen_range(0.0..0.25);
        let sim_config = SimulationConfig {
            n_steps: config.n_steps,
            rho: config.rho,
            beta: config.beta,
            disturbance_kind: disturbance_kind.clone(),
            epsilon_bound: config.epsilon_bound,
        };
        let result = run_simulation_with_s0(&sim_config, s0);
        let (d, b, s, impulse_start, impulse_len) = disturbance_kind.monte_carlo_columns();

        records.push(MonteCarloRunRecord {
            run_id,
            regime_label: disturbance_kind.regime_label().to_string(),
            disturbance_type: disturbance_kind.disturbance_type().to_string(),
            d,
            b,
            s,
            impulse_start,
            impulse_len,
            s0,
            max_envelope: result.s.iter().copied().fold(0.0, f64::max),
            min_trust: result.w.iter().copied().fold(1.0, f64::min),
            time_to_recover: time_to_recover(
                &disturbance_kind,
                &result.s,
                config.epsilon_bound,
                config.recovery_delta,
            ),
        });
    }

    MonteCarloBatch {
        records,
        example_impulse: example_impulse_result(config.n_steps, config.rho, config.beta),
        example_persistent: example_persistent_result(config.n_steps, config.rho, config.beta),
    }
}

pub fn summarize_batch(config: &MonteCarloConfig, batch: &MonteCarloBatch) -> MonteCarloSummary {
    let mut regime_counts = BTreeMap::new();
    let mut sum_max_envelope = 0.0;
    let mut min_observed_trust = 1.0_f64;

    for record in &batch.records {
        sum_max_envelope += record.max_envelope;
        min_observed_trust = min_observed_trust.min(record.min_trust);
        *regime_counts
            .entry(record.regime_label.clone())
            .or_insert(0) += 1;
    }

    let mean_max_envelope = if batch.records.is_empty() {
        0.0
    } else {
        sum_max_envelope / batch.records.len() as f64
    };

    MonteCarloSummary {
        n_runs: config.n_runs,
        n_steps: config.n_steps,
        seed: config.seed,
        rho: config.rho,
        beta: config.beta,
        epsilon_bound: config.epsilon_bound,
        recovery_delta: config.recovery_delta,
        mean_max_envelope,
        min_observed_trust,
        regime_counts,
    }
}

pub fn example_impulse_result(n_steps: usize, rho: f64, beta: f64) -> SimulationResult {
    let config = SimulationConfig {
        n_steps,
        rho,
        beta,
        disturbance_kind: DisturbanceKind::Impulsive {
            amplitude: 1.4,
            start: 24,
            len: 7,
        },
        epsilon_bound: 0.0,
    };
    run_simulation_with_s0(&config, 0.0)
}

pub fn example_persistent_result(n_steps: usize, rho: f64, beta: f64) -> SimulationResult {
    let config = SimulationConfig {
        n_steps,
        rho,
        beta,
        disturbance_kind: DisturbanceKind::PersistentElevated {
            r_nom: 0.05,
            r_high: 0.65,
            step_time: 24,
        },
        epsilon_bound: 0.0,
    };
    run_simulation_with_s0(&config, 0.0)
}

pub fn trajectory_rows(result: &SimulationResult) -> Vec<TrajectoryRow> {
    (0..result.len())
        .map(|n| TrajectoryRow {
            n,
            r: result.r[n],
            d: result.d[n],
            s: result.s[n],
            w: result.w[n],
        })
        .collect()
}

fn sample_disturbance(rng: &mut StdRng, n_steps: usize) -> DisturbanceKind {
    match rng.gen_range(0..5) {
        0 => DisturbanceKind::PointwiseBounded {
            d: sample_signed(rng, 0.02, 0.35),
        },
        1 => DisturbanceKind::Drift {
            b: sample_signed(rng, 0.002, 0.03),
            s_max: rng.gen_range(0.15..0.85),
        },
        2 => DisturbanceKind::SlewRateBounded {
            s_max: rng.gen_range(0.01..0.09),
        },
        3 => {
            let max_start = (n_steps / 2).max(8);
            let max_len = (n_steps / 6).max(4);
            DisturbanceKind::Impulsive {
                amplitude: sample_signed(rng, 0.4, 2.0),
                start: rng.gen_range(6..max_start),
                len: rng.gen_range(2..max_len),
            }
        }
        _ => DisturbanceKind::PersistentElevated {
            r_nom: rng.gen_range(0.01..0.12),
            r_high: rng.gen_range(0.2..1.0),
            step_time: rng.gen_range(10..(n_steps / 2).max(11)),
        },
    }
}

fn sample_signed(rng: &mut StdRng, low: f64, high: f64) -> f64 {
    let amplitude = rng.gen_range(low..high);
    if rng.gen_bool(0.5) {
        amplitude
    } else {
        -amplitude
    }
}

fn time_to_recover(
    kind: &DisturbanceKind,
    envelope: &[f64],
    nominal_bound: f64,
    delta: f64,
) -> i64 {
    let Some(target) = kind.recovery_target(nominal_bound) else {
        return -1;
    };
    let Some(start) = kind.recovery_search_start() else {
        return -1;
    };

    envelope
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, s)| (*s - target).abs() <= delta)
        .map(|(n, _)| n as i64)
        .unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::{
        run_monte_carlo, summarize_batch, time_to_recover, MonteCarloConfig,
        DEFAULT_MONTE_CARLO_RUNS,
    };
    use crate::disturbances::DisturbanceKind;

    #[test]
    fn monte_carlo_is_reproducible() {
        let config = MonteCarloConfig {
            n_runs: 8,
            ..MonteCarloConfig::default()
        };
        let a = run_monte_carlo(&config);
        let b = run_monte_carlo(&config);
        assert_eq!(a.records[0].max_envelope, b.records[0].max_envelope);
        assert_eq!(a.records[0].regime_label, b.records[0].regime_label);
    }

    #[test]
    fn summary_counts_all_runs() {
        let config = MonteCarloConfig {
            n_runs: 10,
            ..MonteCarloConfig::default()
        };
        let batch = run_monte_carlo(&config);
        let summary = summarize_batch(&config, &batch);
        let counted: usize = summary.regime_counts.values().sum();
        assert_eq!(counted, 10);
    }

    #[test]
    fn persistent_elevated_does_not_report_recovery() {
        let config = MonteCarloConfig {
            n_runs: 1,
            n_steps: 64,
            ..MonteCarloConfig::default()
        };
        let result = super::example_persistent_result(config.n_steps, config.rho, config.beta);
        let t = time_to_recover(
            &DisturbanceKind::PersistentElevated {
                r_nom: 0.05,
                r_high: 0.65,
                step_time: 24,
            },
            &result.s,
            config.epsilon_bound,
            config.recovery_delta,
        );
        assert_eq!(t, -1);
    }

    #[test]
    fn slew_only_regime_is_marked_unbounded() {
        let kind = DisturbanceKind::SlewRateBounded { s_max: 0.05 };
        assert_eq!(kind.regime_label(), "unbounded");
    }

    #[test]
    fn default_monte_carlo_batch_is_x360() {
        assert_eq!(MonteCarloConfig::default().n_runs, DEFAULT_MONTE_CARLO_RUNS);
    }
}
