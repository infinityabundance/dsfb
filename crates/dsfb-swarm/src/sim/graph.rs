use nalgebra::DMatrix;
use serde::Serialize;

use crate::sim::agents::AgentState;

#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub adjacency: DMatrix<f64>,
    pub distances: DMatrix<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeRecord {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

pub fn build_nominal_graph(
    agents: &[AgentState],
    radius: f64,
    k_neighbors: usize,
    gain: f64,
) -> GraphSnapshot {
    let n = agents.len();
    let mut distances = DMatrix::zeros(n, n);
    let mut adjacency = DMatrix::zeros(n, n);
    let nearest = nearest_neighbors(agents, k_neighbors);

    for row in 0..n {
        for col in (row + 1)..n {
            let distance = (agents[row].position - agents[col].position).norm();
            distances[(row, col)] = distance;
            distances[(col, row)] = distance;
            let connected =
                distance <= radius || nearest[row].contains(&col) || nearest[col].contains(&row);
            if connected {
                let weight = gain * (-(distance / radius).powi(2)).exp();
                adjacency[(row, col)] = weight;
                adjacency[(col, row)] = weight;
            }
        }
    }

    GraphSnapshot {
        adjacency,
        distances,
    }
}

pub fn edge_records(adjacency: &DMatrix<f64>) -> Vec<EdgeRecord> {
    let mut edges = Vec::new();
    for row in 0..adjacency.nrows() {
        for col in (row + 1)..adjacency.ncols() {
            let weight = adjacency[(row, col)];
            if weight > 1.0e-8 {
                edges.push(EdgeRecord {
                    source: row,
                    target: col,
                    weight,
                });
            }
        }
    }
    edges
}

pub fn pair_disagreement(agents: &[AgentState], adjacency: &DMatrix<f64>) -> DMatrix<f64> {
    let n = agents.len();
    let mut disagreement = DMatrix::zeros(n, n);
    for row in 0..n {
        for col in (row + 1)..n {
            let scalar_gap = (agents[row].scalar - agents[col].scalar).abs();
            let velocity_gap = (agents[row].velocity - agents[col].velocity).norm();
            let position_gap = (agents[row].position - agents[col].position).norm();
            let score = adjacency[(row, col)]
                * (0.65 * scalar_gap + 0.20 * velocity_gap + 0.15 * position_gap);
            disagreement[(row, col)] = score;
            disagreement[(col, row)] = score;
        }
    }
    disagreement
}

fn nearest_neighbors(agents: &[AgentState], k_neighbors: usize) -> Vec<Vec<usize>> {
    let n = agents.len();
    (0..n)
        .map(|index| {
            let mut others = (0..n)
                .filter(|other| *other != index)
                .map(|other| {
                    (
                        other,
                        (agents[index].position - agents[other].position).norm(),
                    )
                })
                .collect::<Vec<_>>();
            others.sort_by(|left, right| left.1.total_cmp(&right.1));
            others
                .into_iter()
                .take(k_neighbors)
                .map(|pair| pair.0)
                .collect::<Vec<_>>()
        })
        .collect()
}
