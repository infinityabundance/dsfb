use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::sweep::deterministic_drive;
use crate::AddError;

pub const RLT_EXAMPLE_STEPS: usize = 240;
pub const RLT_BOUNDED_THRESHOLD: f64 = 0.05;
pub const RLT_EXPANDING_THRESHOLD: f64 = 0.95;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RltSweep {
    pub escape_rate: Vec<f64>,
    pub expansion_ratio: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RltExampleKind {
    Bounded,
    Expanding,
}

impl RltExampleKind {
    pub fn filename_prefix(self) -> &'static str {
        match self {
            Self::Bounded => "bounded",
            Self::Expanding => "expanding",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RltTrajectoryPoint {
    pub step: usize,
    pub lambda: f64,
    pub vertex_id: i64,
    pub x: i32,
    pub y: i32,
    pub distance_from_start: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vertex {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
enum RltRegime {
    Bounded,
    Transitional,
    Expanding,
}

pub fn run_rlt_sweep(config: &SimulationConfig, lambda_grid: &[f64]) -> Result<RltSweep, AddError> {
    let mut escape_rate = Vec::with_capacity(lambda_grid.len());
    let mut expansion_ratio = Vec::with_capacity(lambda_grid.len());

    for &lambda in lambda_grid {
        let vertices = simulate_vertices(config, lambda, config.steps_per_run);
        let (escape, expansion) = summarize_trajectory(&vertices, config.steps_per_run);
        escape_rate.push(escape);
        expansion_ratio.push(expansion);
    }

    Ok(RltSweep {
        escape_rate,
        expansion_ratio,
    })
}

pub fn simulate_example_trajectory(
    config: &SimulationConfig,
    lambda: f64,
    steps: usize,
) -> Vec<RltTrajectoryPoint> {
    let vertices = simulate_vertices(config, lambda, steps);
    let mut adjacency: HashMap<Vertex, Vec<Vertex>> = HashMap::new();
    let origin = *vertices.first().unwrap_or(&Vertex { x: 0, y: 0 });
    let mut points = Vec::with_capacity(vertices.len());

    for (step, &vertex) in vertices.iter().enumerate() {
        if step > 0 {
            add_edge(&mut adjacency, vertices[step - 1], vertex);
        } else {
            adjacency.entry(vertex).or_default();
        }

        let distance_from_start = bfs_distance(&adjacency, origin, vertex).unwrap_or(step);
        points.push(RltTrajectoryPoint {
            step,
            lambda,
            vertex_id: encode_vertex(vertex),
            x: vertex.x,
            y: vertex.y,
            distance_from_start,
        });
    }

    points
}

pub fn find_representative_regime_indices(escape_rate: &[f64]) -> (usize, usize) {
    let bounded_idx = escape_rate
        .iter()
        .position(|&value| value <= RLT_BOUNDED_THRESHOLD)
        .unwrap_or_else(|| nearest_index(escape_rate, 0.0));

    let expanding_idx = escape_rate
        .iter()
        .position(|&value| value >= RLT_EXPANDING_THRESHOLD)
        .unwrap_or_else(|| nearest_index(escape_rate, 1.0));

    (bounded_idx, expanding_idx)
}

fn nearest_index(values: &[f64], target: f64) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_distance = (*left - target).abs();
            let right_distance = (*right - target).abs();
            left_distance
                .partial_cmp(&right_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn simulate_vertices(config: &SimulationConfig, lambda: f64, steps: usize) -> Vec<Vertex> {
    let lambda_norm = config.normalized_lambda(lambda);
    let drive = deterministic_drive(config.random_seed, lambda, 0xB170_u64);
    let mut current = Vertex { x: 0, y: 0 };
    let mut vertices = Vec::with_capacity(steps + 1);
    vertices.push(current);

    for step in 0..steps {
        current = resonance_step(current, step, lambda_norm, drive);
        vertices.push(current);
    }

    vertices
}

fn summarize_trajectory(vertices: &[Vertex], steps: usize) -> (f64, f64) {
    let origin = *vertices.first().unwrap_or(&Vertex { x: 0, y: 0 });
    let goal = *vertices.last().unwrap_or(&origin);
    let mut visited = HashSet::new();
    let mut adjacency: HashMap<Vertex, Vec<Vertex>> = HashMap::new();

    for (idx, &vertex) in vertices.iter().enumerate() {
        visited.insert(vertex);
        if idx > 0 {
            add_edge(&mut adjacency, vertices[idx - 1], vertex);
        } else {
            adjacency.entry(vertex).or_default();
        }
    }

    let distance = bfs_distance(&adjacency, origin, goal).unwrap_or(steps);
    (
        distance as f64 / steps.max(1) as f64,
        visited.len() as f64 / steps.max(1) as f64,
    )
}

fn resonance_step(
    current: Vertex,
    step: usize,
    lambda_norm: f64,
    drive: crate::sweep::DriveSignal,
) -> Vertex {
    let regime = classify_regime(lambda_norm);
    let phase_bucket =
        (lambda_norm * 11.0).round() as i32 + (drive.phase_bias * 5.0).round() as i32;
    let trust_sign = if drive.trust_bias >= 0.0 { 1 } else { -1 };

    match regime {
        RltRegime::Bounded => bounded_step(step, phase_bucket, trust_sign),
        RltRegime::Transitional => {
            transitional_step(current, step, lambda_norm, phase_bucket, trust_sign)
        }
        RltRegime::Expanding => expanding_step(current, step, phase_bucket, trust_sign),
    }
}

fn classify_regime(lambda_norm: f64) -> RltRegime {
    if lambda_norm < 0.22 {
        RltRegime::Bounded
    } else if lambda_norm < 0.58 {
        RltRegime::Transitional
    } else {
        RltRegime::Expanding
    }
}

fn bounded_step(step: usize, phase_bucket: i32, trust_sign: i32) -> Vertex {
    const CYCLE: [(i32, i32); 6] = [(0, 0), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];
    let idx = (step as i32 + phase_bucket).rem_euclid(CYCLE.len() as i32) as usize;
    let (x, y) = CYCLE[idx];
    Vertex {
        x: x * trust_sign,
        y,
    }
}

fn transitional_step(
    current: Vertex,
    step: usize,
    lambda_norm: f64,
    phase_bucket: i32,
    trust_sign: i32,
) -> Vertex {
    let leash = 2 + (lambda_norm * 10.0).round() as i32;
    let resonance_class = (step as i32 + phase_bucket).rem_euclid(6);
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
            y: current.y + 1,
        },
        _ => Vertex {
            x: current.x - trust_sign,
            y: current.y,
        },
    };

    let reset_period = ((16.0 - 10.0 * lambda_norm).round() as usize).clamp(6, 16);
    if step % reset_period == 0 {
        next = Vertex {
            x: phase_bucket.rem_euclid(3) - 1,
            y: (step / reset_period) as i32 % 3 - 1,
        };
    }

    next.x = next.x.clamp(-leash, leash);
    next.y = next.y.clamp(-leash, leash);
    next
}

fn expanding_step(current: Vertex, step: usize, phase_bucket: i32, trust_sign: i32) -> Vertex {
    let resonance_class = (step as i32 + phase_bucket).rem_euclid(5);
    let dy = match resonance_class {
        0 => 0,
        1 | 2 => 1,
        _ => 2,
    };

    Vertex {
        x: current.x + 1,
        y: current.y + dy + trust_sign.max(0),
    }
}

fn encode_vertex(vertex: Vertex) -> i64 {
    ((vertex.x as i64) << 32) ^ (vertex.y as u32 as i64)
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
