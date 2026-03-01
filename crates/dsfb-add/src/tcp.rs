use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::sweep::deterministic_drive;
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpPoint {
    pub t: usize,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSweep {
    pub betti0: Vec<usize>,
    pub betti1: Vec<usize>,
    pub l_tcp: Vec<f64>,
    pub avg_radius: Vec<f64>,
    pub max_radius: Vec<f64>,
    pub variance_radius: Vec<f64>,
    pub point_clouds: Vec<Vec<TcpPoint>>,
}

pub fn run_tcp_sweep(config: &SimulationConfig, lambda_grid: &[f64]) -> Result<TcpSweep, AddError> {
    let mut betti0 = Vec::with_capacity(lambda_grid.len());
    let mut betti1 = Vec::with_capacity(lambda_grid.len());
    let mut l_tcp = Vec::with_capacity(lambda_grid.len());
    let mut avg_radius = Vec::with_capacity(lambda_grid.len());
    let mut max_radius = Vec::with_capacity(lambda_grid.len());
    let mut variance_radius = Vec::with_capacity(lambda_grid.len());
    let mut point_clouds = Vec::with_capacity(lambda_grid.len());

    for (idx, &lambda) in lambda_grid.iter().enumerate() {
        let lambda_norm = config.normalized_lambda(lambda);
        let drive = deterministic_drive(config.random_seed, lambda, 0x7CD0_u64 + idx as u64);

        let mut x = 0.25 + 0.35 * drive.phase_bias;
        let mut y = -0.15 + 0.30 * drive.trust_bias;
        let mut points = Vec::with_capacity(config.steps_per_run);

        for step in 0..config.steps_per_run {
            points.push(TcpPoint { t: step, x, y });

            let a = 1.08 + 0.72 * lambda_norm + 0.08 * drive.phase_bias;
            let b = 0.22 + 0.11 * lambda_norm + 0.04 * drive.trust_bias.abs();
            let forcing = 0.22 * ((step as f64) * 0.043 + lambda * std::f64::consts::TAU).sin();

            let next_x = (1.0 - a * x * x + y + forcing + 0.06 * drive.drift_bias).tanh();
            let next_y = (b * x + 0.30 * ((step as f64) * 0.0375 + drive.phase_bias).cos()).tanh();

            x = next_x;
            y = next_y;
        }

        let radii: Vec<f64> = points
            .iter()
            .map(|point| (point.x * point.x + point.y * point.y).sqrt())
            .collect();

        let radius_mean = radii.iter().sum::<f64>() / radii.len() as f64;
        let radius_max = radii.iter().copied().fold(0.0_f64, f64::max);
        let radius_variance = radii
            .iter()
            .map(|radius| {
                let delta = radius - radius_mean;
                delta * delta
            })
            .sum::<f64>()
            / radii.len() as f64;

        let (components, holes) = occupancy_topology(&points, 18);
        let tcp_scale = components as f64 + holes as f64 + radius_variance;

        betti0.push(components);
        betti1.push(holes);
        l_tcp.push(tcp_scale);
        avg_radius.push(radius_mean);
        max_radius.push(radius_max);
        variance_radius.push(radius_variance);
        point_clouds.push(points);
    }

    Ok(TcpSweep {
        betti0,
        betti1,
        l_tcp,
        avg_radius,
        max_radius,
        variance_radius,
        point_clouds,
    })
}

fn occupancy_topology(points: &[TcpPoint], grid_size: usize) -> (usize, usize) {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);

    let span_x = (max_x - min_x).max(1e-6);
    let span_y = (max_y - min_y).max(1e-6);

    let mut grid = vec![vec![false; grid_size]; grid_size];
    for point in points {
        let x_norm = ((point.x - min_x) / span_x).clamp(0.0, 1.0);
        let y_norm = ((point.y - min_y) / span_y).clamp(0.0, 1.0);

        let i = ((x_norm * (grid_size as f64 - 1.0)).round() as usize).min(grid_size - 1);
        let j = ((y_norm * (grid_size as f64 - 1.0)).round() as usize).min(grid_size - 1);
        grid[j][i] = true;
    }

    let components = count_true_components(&grid);
    let holes = count_false_holes(&grid);
    (components, holes)
}

fn count_true_components(grid: &[Vec<bool>]) -> usize {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut seen = HashSet::new();
    let mut components = 0_usize;

    for row in 0..rows {
        for col in 0..cols {
            if !grid[row][col] || seen.contains(&(row, col)) {
                continue;
            }

            components += 1;
            let mut queue = VecDeque::from([(row, col)]);
            seen.insert((row, col));

            while let Some((r, c)) = queue.pop_front() {
                for (nr, nc) in neighbors(r, c, rows, cols) {
                    if grid[nr][nc] && seen.insert((nr, nc)) {
                        queue.push_back((nr, nc));
                    }
                }
            }
        }
    }

    components
}

fn count_false_holes(grid: &[Vec<bool>]) -> usize {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut seen = HashSet::new();
    let mut holes = 0_usize;

    for row in 0..rows {
        for col in 0..cols {
            if grid[row][col] || seen.contains(&(row, col)) {
                continue;
            }

            let mut queue = VecDeque::from([(row, col)]);
            let mut touches_boundary = false;
            seen.insert((row, col));

            while let Some((r, c)) = queue.pop_front() {
                if r == 0 || c == 0 || r + 1 == rows || c + 1 == cols {
                    touches_boundary = true;
                }

                for (nr, nc) in neighbors(r, c, rows, cols) {
                    if !grid[nr][nc] && seen.insert((nr, nc)) {
                        queue.push_back((nr, nc));
                    }
                }
            }

            if !touches_boundary {
                holes += 1;
            }
        }
    }

    holes
}

fn neighbors(row: usize, col: usize, rows: usize, cols: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);

    if row > 0 {
        out.push((row - 1, col));
    }
    if row + 1 < rows {
        out.push((row + 1, col));
    }
    if col > 0 {
        out.push((row, col - 1));
    }
    if col + 1 < cols {
        out.push((row, col + 1));
    }

    out
}
