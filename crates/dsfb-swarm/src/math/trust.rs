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
            recovery: 0.86,
            floor: 0.05,
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
        let max_pair = pair_disagreement
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            .max(1.0e-9);

        for node in 0..self.node_trust.len() {
            let mut weighted = 0.0;
            let mut total_weight = 0.0;
            for other in 0..self.node_trust.len() {
                let weight = adjacency[(node, other)];
                weighted += weight * pair_disagreement[(node, other)] / max_pair;
                total_weight += weight;
            }
            let local_score = if total_weight > 0.0 {
                weighted / total_weight
            } else {
                0.0
            };
            let target = match self.mode {
                TrustGateMode::BinaryEnvelope => {
                    if local_score < 0.65 && global_score < 1.0 {
                        1.0
                    } else {
                        self.floor
                    }
                }
                TrustGateMode::SmoothDecay => {
                    (-2.8 * (0.7 * local_score + 0.3 * global_score.min(3.0))).exp()
                }
            }
            .clamp(self.floor, 1.0);
            self.node_trust[node] = (self.recovery * self.node_trust[node] + (1.0 - self.recovery) * target)
                .clamp(self.floor, 1.0);
        }

        let n = self.node_trust.len();
        for row in 0..n {
            self.edge_trust[(row, row)] = 0.0;
            for col in (row + 1)..n {
                let pair_factor = (-1.2 * (pair_disagreement[(row, col)] / max_pair)).exp();
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
