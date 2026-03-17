use nalgebra::{DMatrix, Vector2};
use serde::Serialize;

use crate::config::ScenarioKind;
use crate::sim::agents::cluster_of;

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioDefinition {
    pub kind: ScenarioKind,
    pub name: &'static str,
    pub description: &'static str,
    pub onset_step: usize,
}

impl ScenarioDefinition {
    pub fn from_kind(kind: ScenarioKind, total_steps: usize) -> Self {
        match kind {
            ScenarioKind::Nominal => Self {
                kind,
                name: "nominal",
                description: "Stable coordination with bounded deterministic excitation and no persistent topology loss.",
                onset_step: total_steps,
            },
            ScenarioKind::GradualEdgeDegradation => Self {
                kind,
                name: "gradual_edge_degradation",
                description: "Bridge edges weaken progressively, creating persistent negative residual drift before visible connectivity collapse.",
                onset_step: total_steps / 3,
            },
            ScenarioKind::AdversarialAgent => Self {
                kind,
                name: "adversarial_agent",
                description: "One agent injects inconsistent state and motion, disturbing local spectral structure until trust gating attenuates its influence.",
                onset_step: total_steps / 4,
            },
            ScenarioKind::CommunicationLoss => Self {
                kind,
                name: "communication_loss",
                description: "A bridge set experiences abrupt communication loss, forcing fragmentation and algebraic connectivity collapse.",
                onset_step: total_steps / 2,
            },
            ScenarioKind::All => Self {
                kind: ScenarioKind::Nominal,
                name: "nominal",
                description: "Alias scenario definition",
                onset_step: total_steps,
            },
        }
    }

    pub fn affected_nodes(&self, agent_count: usize) -> Vec<usize> {
        match self.kind {
            ScenarioKind::Nominal => Vec::new(),
            ScenarioKind::GradualEdgeDegradation => vec![agent_count / 2 - 1, agent_count / 2],
            ScenarioKind::AdversarialAgent => vec![0],
            ScenarioKind::CommunicationLoss => vec![agent_count / 2 - 1, agent_count / 2],
            ScenarioKind::All => Vec::new(),
        }
    }

    pub fn apply_edge_modifiers(
        &self,
        step: usize,
        nominal_adjacency: &DMatrix<f64>,
    ) -> DMatrix<f64> {
        let n = nominal_adjacency.nrows();
        let mut effective = nominal_adjacency.clone();
        let progress = self.progress(step);
        for row in 0..n {
            for col in (row + 1)..n {
                let base = nominal_adjacency[(row, col)];
                if base <= 0.0 {
                    continue;
                }
                let modifier = match self.kind {
                    ScenarioKind::Nominal => 1.0,
                    ScenarioKind::GradualEdgeDegradation => gradual_modifier(progress, row, col, n),
                    ScenarioKind::AdversarialAgent => adversarial_modifier(step, row, col),
                    ScenarioKind::CommunicationLoss => {
                        communication_loss_modifier(progress, row, col, n)
                    }
                    ScenarioKind::All => 1.0,
                };
                effective[(row, col)] = base * modifier;
                effective[(col, row)] = base * modifier;
            }
        }
        effective
    }

    pub fn scalar_bias(&self, step: usize, index: usize, agent_count: usize) -> f64 {
        let progress = self.progress(step);
        match self.kind {
            ScenarioKind::Nominal => 0.02 * (0.04 * step as f64 + 0.13 * index as f64).sin(),
            ScenarioKind::GradualEdgeDegradation => {
                if self.is_bridge_node(index, agent_count) {
                    -0.10 * progress
                } else {
                    0.02 * (0.05 * step as f64 + index as f64 * 0.08).sin()
                }
            }
            ScenarioKind::AdversarialAgent => {
                if index == 0 && step >= self.onset_step {
                    0.75 * (0.31 * step as f64).sin() + 0.30 * (0.14 * step as f64).cos()
                } else {
                    0.02 * (0.04 * step as f64 + 0.13 * index as f64).sin()
                }
            }
            ScenarioKind::CommunicationLoss => {
                if self.is_bridge_node(index, agent_count) && step >= self.onset_step {
                    -0.25
                } else {
                    0.01 * (0.03 * step as f64 + index as f64 * 0.17).sin()
                }
            }
            ScenarioKind::All => 0.0,
        }
    }

    pub fn position_force(&self, step: usize, index: usize, agent_count: usize) -> Vector2<f64> {
        let progress = self.progress(step);
        match self.kind {
            ScenarioKind::Nominal => Vector2::new(0.0, 0.0),
            ScenarioKind::GradualEdgeDegradation => {
                if self.is_bridge_node(index, agent_count) {
                    let direction = if cluster_of(index, agent_count) == 0 {
                        -1.0
                    } else {
                        1.0
                    };
                    Vector2::new(0.04 * progress * direction, 0.0)
                } else {
                    Vector2::zeros()
                }
            }
            ScenarioKind::AdversarialAgent => {
                if index == 0 && step >= self.onset_step {
                    Vector2::new(
                        0.18 * (0.2 * step as f64).cos(),
                        0.18 * (0.2 * step as f64).sin(),
                    )
                } else {
                    Vector2::zeros()
                }
            }
            ScenarioKind::CommunicationLoss => {
                let direction = if cluster_of(index, agent_count) == 0 {
                    -1.0
                } else {
                    1.0
                };
                Vector2::new(0.10 * progress * direction, 0.0)
            }
            ScenarioKind::All => Vector2::zeros(),
        }
    }

    fn progress(&self, step: usize) -> f64 {
        if step <= self.onset_step {
            0.0
        } else {
            ((step - self.onset_step) as f64 / (self.onset_step.max(1) as f64)).clamp(0.0, 1.0)
        }
    }

    fn is_bridge_node(&self, index: usize, agent_count: usize) -> bool {
        index == agent_count / 2 - 1 || index == agent_count / 2
    }
}

fn gradual_modifier(progress: f64, row: usize, col: usize, agent_count: usize) -> f64 {
    let left = cluster_of(row, agent_count);
    let right = cluster_of(col, agent_count);
    if left != right {
        1.0 - 0.88 * progress
    } else if row.abs_diff(col) <= 1 {
        1.0 - 0.22 * progress
    } else {
        1.0
    }
}

fn adversarial_modifier(step: usize, row: usize, col: usize) -> f64 {
    if row == 0 || col == 0 {
        (0.88 - 0.35 * (0.23 * step as f64).sin().abs()).clamp(0.18, 1.0)
    } else {
        1.0
    }
}

fn communication_loss_modifier(progress: f64, row: usize, col: usize, agent_count: usize) -> f64 {
    let left = cluster_of(row, agent_count);
    let right = cluster_of(col, agent_count);
    if left != right {
        (1.0 - 1.2 * progress).clamp(0.0, 1.0)
    } else {
        1.0
    }
}
