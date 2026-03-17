use nalgebra::DMatrix;
use serde::Serialize;

use crate::config::TrustGateMode;

#[derive(Debug, Clone)]
pub struct TrustModel {
    mode: TrustGateMode,
    recovery: f64,
    floor: f64,
    node_trust: Vec<f64>,
    edge_trust: DMatrix<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrustSnapshot {
    pub mean_node_trust: f64,
    pub min_node_trust: f64,
    pub affected_mean_trust: f64,
    pub edge_trust_mean: f64,
    pub node_trust: Vec<f64>,
}

impl TrustModel {
    pub fn new(mode: TrustGateMode, node_count: usize) -> Self {
        Self {
            mode,
            recovery: 0.90,
            floor: 0.08,
            node_trust: vec![1.0; node_count],
            edge_trust: DMatrix::from_element(node_count, node_count, 1.0),
        }
    }

    pub fn current_edge_trust(&self) -> &DMatrix<f64> {
        &self.edge_trust
    }

    pub fn update(
        &mut self,
        adjacency: &DMatrix<f64>,
        pair_disagreement: &DMatrix<f64>,
        global_score: f64,
        affected_nodes: &[usize],
    ) -> TrustSnapshot {
        let global_score = global_score.max(0.0);
        let mut node_scores = vec![0.0; self.node_trust.len()];
        for node in 0..self.node_trust.len() {
            let mut weighted = 0.0;
            let mut total_weight = 0.0;
            for other in 0..self.node_trust.len() {
                let weight = adjacency[(node, other)];
                weighted += weight * pair_disagreement[(node, other)];
                total_weight += weight;
            }
            node_scores[node] = if total_weight > 0.0 {
                weighted / total_weight
            } else {
                0.0
            };
        }

        let score_mean = node_scores.iter().sum::<f64>() / node_scores.len() as f64;
        let score_std = (node_scores
            .iter()
            .map(|value| {
                let delta = value - score_mean;
                delta * delta
            })
            .sum::<f64>()
            / node_scores.len() as f64)
            .sqrt()
            .max(1.0e-6);

        let pair_mean = pair_disagreement.iter().sum::<f64>()
            / (pair_disagreement.nrows() * pair_disagreement.ncols()).max(1) as f64;
        let pair_scale = (pair_disagreement
            .iter()
            .map(|value| {
                let delta = value - pair_mean;
                delta * delta
            })
            .sum::<f64>()
            / (pair_disagreement.nrows() * pair_disagreement.ncols()).max(1) as f64)
            .sqrt()
            .max(1.0e-6);

        for node in 0..self.node_trust.len() {
            let local_score = ((node_scores[node] - score_mean) / score_std).max(0.0);
            let target = match self.mode {
                TrustGateMode::BinaryEnvelope => {
                    if local_score < 0.75 && global_score < 0.9 {
                        1.0
                    } else {
                        self.floor
                    }
                }
                TrustGateMode::SmoothDecay => {
                    (-1.35 * local_score - 1.10 * global_score.min(4.0)).exp()
                }
            }
            .clamp(self.floor, 1.0);
            self.node_trust[node] = (self.recovery * self.node_trust[node]
                + (1.0 - self.recovery) * target)
                .clamp(self.floor, 1.0);
        }

        let n = self.node_trust.len();
        for row in 0..n {
            self.edge_trust[(row, row)] = 0.0;
            for col in (row + 1)..n {
                let pair_relative =
                    ((pair_disagreement[(row, col)] - pair_mean) / pair_scale).max(0.0);
                let pair_factor = (-0.45 * pair_relative).exp();
                let value = self.node_trust[row].min(self.node_trust[col]) * pair_factor;
                self.edge_trust[(row, col)] = value;
                self.edge_trust[(col, row)] = value;
            }
        }

        let edge_values = self
            .edge_trust
            .iter()
            .copied()
            .filter(|value| *value > 0.0)
            .collect::<Vec<_>>();
        let edge_trust_mean = if edge_values.is_empty() {
            0.0
        } else {
            edge_values.iter().sum::<f64>() / edge_values.len() as f64
        };
        let mean_node_trust = self.node_trust.iter().sum::<f64>() / self.node_trust.len() as f64;
        let min_node_trust = self.node_trust.iter().copied().fold(1.0_f64, f64::min);
        let affected_mean_trust = if affected_nodes.is_empty() {
            mean_node_trust
        } else {
            affected_nodes
                .iter()
                .map(|index| self.node_trust[*index])
                .sum::<f64>()
                / affected_nodes.len() as f64
        };

        TrustSnapshot {
            mean_node_trust,
            min_node_trust,
            affected_mean_trust,
            edge_trust_mean,
            node_trust: self.node_trust.clone(),
        }
    }
}
