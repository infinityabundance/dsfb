use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::sweep::deterministic_drive;
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RltSweep {
    pub escape_rate: Vec<f64>,
    pub expansion_ratio: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vertex {
    x: i32,
    y: i32,
}

pub fn run_rlt_sweep(config: &SimulationConfig, lambda_grid: &[f64]) -> Result<RltSweep, AddError> {
    let mut escape_rate = Vec::with_capacity(lambda_grid.len());
    let mut expansion_ratio = Vec::with_capacity(lambda_grid.len());

    for (idx, &lambda) in lambda_grid.iter().enumerate() {
        let lambda_norm = config.normalized_lambda(lambda);
        let drive = deterministic_drive(config.random_seed, lambda, 0xB170_u64 + idx as u64);
        let mut current = Vertex { x: 0, y: 0 };
        let origin = current;

        let mut visited = HashSet::from([origin]);
        let mut adjacency: HashMap<Vertex, Vec<Vertex>> = HashMap::new();

        for step in 0..config.steps_per_run {
            let next = resonance_step(current, step, lambda_norm, drive);
            add_edge(&mut adjacency, current, next);
            visited.insert(next);
            current = next;
        }

        let distance = bfs_distance(&adjacency, origin, current).unwrap_or(config.steps_per_run);
        escape_rate.push(distance as f64 / config.steps_per_run as f64);
        expansion_ratio.push(visited.len() as f64 / config.steps_per_run as f64);
    }

    Ok(RltSweep {
        escape_rate,
        expansion_ratio,
    })
}

fn resonance_step(
    current: Vertex,
    step: usize,
    lambda_norm: f64,
    drive: crate::sweep::DriveSignal,
) -> Vertex {
    let phase_bucket =
        (lambda_norm * 13.0).round() as i32 + (drive.phase_bias * 5.0).round() as i32;
    let trust_sign = if drive.trust_bias >= 0.0 { 1 } else { -1 };
    let resonance_class =
        (current.x * 3 - current.y * 2 + phase_bucket + step as i32).rem_euclid(6);

    let mut next = match resonance_class {
        0 => Vertex {
            x: current.x + 1,
            y: current.y,
        },
        1 => Vertex {
            x: current.x,
            y: current.y + 1,
        },
        2 => Vertex {
            x: current.x - 1,
            y: current.y + trust_sign,
        },
        3 => Vertex {
            x: current.x + trust_sign,
            y: current.y - 1,
        },
        4 => Vertex {
            x: current.x + 1,
            y: current.y + trust_sign,
        },
        _ => Vertex {
            x: current.x - trust_sign,
            y: current.y + 1,
        },
    };

    if (next.x + 2 * next.y + phase_bucket).rem_euclid(5) == 0 {
        next.x += trust_sign;
    }

    if (2 * next.x - next.y + (drive.drift_bias * 7.0).round() as i32).rem_euclid(7) == 0 {
        next.y += 1;
    }

    next
}

fn add_edge(adjacency: &mut HashMap<Vertex, Vec<Vertex>>, a: Vertex, b: Vertex) {
    adjacency.entry(a).or_default();
    adjacency.entry(b).or_default();

    if let Some(neighbors) = adjacency.get_mut(&a) {
        if !neighbors.contains(&b) {
            neighbors.push(b);
        }
    }

    if let Some(neighbors) = adjacency.get_mut(&b) {
        if !neighbors.contains(&a) {
            neighbors.push(a);
        }
    }
}

fn bfs_distance(
    adjacency: &HashMap<Vertex, Vec<Vertex>>,
    start: Vertex,
    goal: Vertex,
) -> Option<usize> {
    if start == goal {
        return Some(0);
    }

    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([(start, 0_usize)]);

    while let Some((vertex, distance)) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&vertex) {
            for &neighbor in neighbors {
                if !seen.insert(neighbor) {
                    continue;
                }

                if neighbor == goal {
                    return Some(distance + 1);
                }

                queue.push_back((neighbor, distance + 1));
            }
        }
    }

    None
}
