use nalgebra::DMatrix;
use serde::Serialize;

use crate::sim::agents::AgentState;

#[derive(Debug, Clone, Serialize)]
pub struct BaselineRow {
    pub scenario: String,
    pub step: usize,
    pub time: f64,
    pub state_norm_score: f64,
    pub disagreement_energy_score: f64,
    pub raw_lambda2_score: f64,
    pub state_norm_threshold: f64,
    pub disagreement_energy_threshold: f64,
    pub raw_lambda2_threshold: f64,
    pub state_norm_flag: bool,
    pub disagreement_energy_flag: bool,
    pub raw_lambda2_flag: bool,
}

#[derive(Debug, Clone)]
pub struct BaselineMonitor {
    warmup_steps: usize,
    step: usize,
    state_norm_samples: Vec<f64>,
    disagreement_samples: Vec<f64>,
    lambda2_samples: Vec<f64>,
    state_norm_threshold: f64,
    disagreement_threshold: f64,
    lambda2_threshold: f64,
    state_norm_persistence: usize,
    disagreement_persistence: usize,
    lambda2_persistence: usize,
}

impl BaselineMonitor {
    pub fn new(warmup_steps: usize) -> Self {
        Self {
            warmup_steps,
            step: 0,
            state_norm_samples: Vec::new(),
            disagreement_samples: Vec::new(),
            lambda2_samples: Vec::new(),
            state_norm_threshold: 0.04,
            disagreement_threshold: 0.04,
            lambda2_threshold: 0.02,
            state_norm_persistence: 0,
            disagreement_persistence: 0,
            lambda2_persistence: 0,
        }
    }

    pub fn update(
        &mut self,
        scenario: &str,
        agents: &[AgentState],
        adjacency: &DMatrix<f64>,
        lambda2: f64,
        time: f64,
    ) -> BaselineRow {
        let mean_scalar = agents.iter().map(|agent| agent.scalar).sum::<f64>() / agents.len() as f64;
        let scalar_energy = agents
            .iter()
            .map(|agent| agent.scalar * agent.scalar)
            .sum::<f64>()
            / agents.len() as f64;
        let scalar_scale = 1.0 + scalar_energy.sqrt();
        let state_norm_score = (agents
            .iter()
            .map(|agent| {
                let delta = agent.scalar - mean_scalar;
                delta * delta
            })
            .sum::<f64>()
            / agents.len() as f64)
            .sqrt()
            / scalar_scale;

        let mut disagreement_energy_score = 0.0;
        let mut total_weight = 0.0;
        for row in 0..adjacency.nrows() {
            for col in (row + 1)..adjacency.ncols() {
                let weight = adjacency[(row, col)];
                let delta = agents[row].scalar - agents[col].scalar;
                disagreement_energy_score += weight * delta * delta;
                total_weight += weight;
            }
        }
        if total_weight > 0.0 {
            disagreement_energy_score =
                (disagreement_energy_score / total_weight).sqrt() / scalar_scale;
        }

        let (state_norm_flag, disagreement_energy_flag, raw_lambda2_flag) = if self.step < self.warmup_steps {
            self.state_norm_samples.push(state_norm_score);
            self.disagreement_samples.push(disagreement_energy_score);
            self.lambda2_samples.push(lambda2);
            (false, false, false)
        } else {
            if self.step == self.warmup_steps {
                self.state_norm_threshold =
                    upper_limit(tail_window(&self.state_norm_samples, 18), 2.8, 0.03);
                self.disagreement_threshold =
                    upper_limit(tail_window(&self.disagreement_samples, 18), 2.6, 0.03);
                self.lambda2_threshold =
                    lower_limit(tail_window(&self.lambda2_samples, 18), 2.4, 0.02);
            }

            update_counter(
                &mut self.state_norm_persistence,
                state_norm_score > self.state_norm_threshold,
            );
            update_counter(
                &mut self.disagreement_persistence,
                disagreement_energy_score > self.disagreement_threshold,
            );
            update_counter(&mut self.lambda2_persistence, lambda2 < self.lambda2_threshold);

            (
                self.state_norm_persistence >= 3,
                self.disagreement_persistence >= 3,
                self.lambda2_persistence >= 3,
            )
        };
        let row = BaselineRow {
            scenario: scenario.to_string(),
            step: self.step,
            time,
            state_norm_score,
            disagreement_energy_score,
            raw_lambda2_score: lambda2,
            state_norm_threshold: self.state_norm_threshold,
            disagreement_energy_threshold: self.disagreement_threshold,
            raw_lambda2_threshold: self.lambda2_threshold,
            state_norm_flag,
            disagreement_energy_flag,
            raw_lambda2_flag,
        };
        self.step += 1;
        row
    }
}

fn upper_limit(samples: &[f64], factor: f64, floor: f64) -> f64 {
    if samples.is_empty() {
        return floor;
    }
    let center = median(samples);
    let deviations = samples
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    let mad = median(&deviations);
    let scale = (1.4826 * mad).max(0.12 * center.abs());
    (center + factor * scale).max(floor)
}

fn lower_limit(samples: &[f64], factor: f64, floor: f64) -> f64 {
    if samples.is_empty() {
        return floor;
    }
    let center = median(samples);
    let deviations = samples
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    let mad = median(&deviations);
    let scale = (1.4826 * mad).max(0.08 * center.abs());
    (center - factor * scale).max(floor)
}

fn tail_window(samples: &[f64], max_len: usize) -> &[f64] {
    let start = samples.len().saturating_sub(max_len);
    &samples[start..]
}

fn update_counter(counter: &mut usize, active: bool) {
    if active {
        *counter += 1;
    } else {
        *counter = 0;
    }
}

fn median(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by(|left, right| left.total_cmp(right));
    let middle = ordered.len() / 2;
    if ordered.len() % 2 == 0 {
        0.5 * (ordered[middle - 1] + ordered[middle])
    } else {
        ordered[middle]
    }
}
