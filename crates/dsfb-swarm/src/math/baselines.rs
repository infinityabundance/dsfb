use nalgebra::DMatrix;
use serde::Serialize;

use crate::sim::agents::AgentState;

#[derive(Debug, Clone, Serialize)]
pub struct BaselineRow {
    pub step: usize,
    pub time: f64,
    pub state_norm_score: f64,
    pub disagreement_energy_score: f64,
    pub raw_lambda2_score: f64,
    pub state_norm_flag: bool,
    pub disagreement_energy_flag: bool,
    pub raw_lambda2_flag: bool,
}

#[derive(Debug, Clone)]
pub struct BaselineMonitor {
    warmup_steps: usize,
    step: usize,
    state_norm_max: f64,
    disagreement_max: f64,
    lambda2_min: f64,
}

impl BaselineMonitor {
    pub fn new(warmup_steps: usize) -> Self {
        Self {
            warmup_steps,
            step: 0,
            state_norm_max: 0.0,
            disagreement_max: 0.0,
            lambda2_min: f64::MAX,
        }
    }

    pub fn update(
        &mut self,
        agents: &[AgentState],
        adjacency: &DMatrix<f64>,
        lambda2: f64,
        time: f64,
    ) -> BaselineRow {
        let mean_scalar = agents.iter().map(|agent| agent.scalar).sum::<f64>() / agents.len() as f64;
        let state_norm_score = (agents
            .iter()
            .map(|agent| {
                let delta = agent.scalar - mean_scalar;
                delta * delta
            })
            .sum::<f64>()
            / agents.len() as f64)
            .sqrt();

        let mut disagreement_energy_score = 0.0;
        for row in 0..adjacency.nrows() {
            for col in (row + 1)..adjacency.ncols() {
                let weight = adjacency[(row, col)];
                let delta = agents[row].scalar - agents[col].scalar;
                disagreement_energy_score += weight * delta * delta;
            }
        }

        self.state_norm_max = self.state_norm_max.max(state_norm_score);
        self.disagreement_max = self.disagreement_max.max(disagreement_energy_score);
        self.lambda2_min = self.lambda2_min.min(lambda2);

        let (state_norm_flag, disagreement_energy_flag, raw_lambda2_flag) = if self.step < self.warmup_steps {
            (false, false, false)
        } else {
            (
                state_norm_score > 2.4 * self.state_norm_max.max(0.02),
                disagreement_energy_score > 2.2 * self.disagreement_max.max(0.02),
                lambda2 < 0.82 * self.lambda2_min.max(0.02),
            )
        };
        let row = BaselineRow {
            step: self.step,
            time,
            state_norm_score,
            disagreement_energy_score,
            raw_lambda2_score: lambda2,
            state_norm_flag,
            disagreement_energy_flag,
            raw_lambda2_flag,
        };
        self.step += 1;
        row
    }
}
