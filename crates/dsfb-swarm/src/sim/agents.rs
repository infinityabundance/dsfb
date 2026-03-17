use nalgebra::Vector2;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AgentState {
    pub id: usize,
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub scalar: f64,
}

pub fn initialize_agents(count: usize) -> Vec<AgentState> {
    let half = count / 2;
    (0..count)
        .map(|index| {
            let local_index = if index < half { index } else { index - half };
            let local_count = half.max(2);
            let t = if local_count == 1 {
                0.0
            } else {
                local_index as f64 / (local_count - 1) as f64
            };
            let base_y = -1.1 + 2.2 * t + 0.08 * (0.37 * index as f64).sin();
            let mut base_x = if index < half { -0.92 } else { 0.92 };
            if index == half.saturating_sub(1) {
                base_x = -0.28;
            }
            if index == half {
                base_x = 0.28;
            }
            let scalar = if index < half { -0.35 } else { 0.35 } + 0.05 * (0.19 * index as f64).cos();
            AgentState {
                id: index,
                position: Vector2::new(base_x, base_y),
                velocity: Vector2::zeros(),
                scalar,
            }
        })
        .collect()
}

pub fn cluster_of(index: usize, count: usize) -> usize {
    if index < count / 2 {
        0
    } else {
        1
    }
}
