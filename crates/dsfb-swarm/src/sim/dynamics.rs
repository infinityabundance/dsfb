use nalgebra::{DMatrix, Vector2};

use crate::config::RunConfig;
use crate::sim::agents::AgentState;
use crate::sim::scenarios::ScenarioDefinition;

pub fn evolve_agents(
    agents: &mut [AgentState],
    effective_adjacency: &DMatrix<f64>,
    config: &RunConfig,
    scenario: &ScenarioDefinition,
    step: usize,
) {
    let n = agents.len();
    let dt = config.dt;
    let centroid = agents
        .iter()
        .fold(Vector2::zeros(), |acc, agent| acc + agent.position)
        / n as f64;

    let mut positions = vec![Vector2::zeros(); n];
    let mut velocities = vec![Vector2::zeros(); n];
    let mut scalars = vec![0.0; n];

    for index in 0..n {
        let mut consensus_position = Vector2::zeros();
        let mut consensus_scalar = 0.0;
        for other in 0..n {
            let weight = effective_adjacency[(index, other)];
            if weight <= 0.0 {
                continue;
            }
            consensus_position += weight * (agents[other].position - agents[index].position);
            consensus_scalar += weight * (agents[other].scalar - agents[index].scalar);
        }

        let orbit = Vector2::new(
            0.22 * (0.04 * step as f64 + 0.17 * index as f64).cos(),
            0.22 * (0.05 * step as f64 + 0.21 * index as f64).sin(),
        );
        let recenter = -0.08 * (agents[index].position - centroid);
        let forcing = scenario.position_force(step, index, n);
        let noise = deterministic_noise_vector(step, index, config.noise_level * 0.15);
        let acceleration = 0.52 * consensus_position + 0.18 * orbit + recenter + forcing + noise;

        velocities[index] = 0.78 * agents[index].velocity + dt * acceleration;
        positions[index] = agents[index].position + dt * velocities[index];

        let scalar_drive = 0.06 * (0.03 * step as f64 + index as f64 * 0.11).sin();
        let scalar_noise = deterministic_noise_scalar(step, index, config.noise_level * 0.25);
        let scalar_bias = scenario.scalar_bias(step, index, n);
        scalars[index] = agents[index].scalar + dt * (0.95 * consensus_scalar + scalar_drive + scalar_noise + scalar_bias);
    }

    for index in 0..n {
        agents[index].position = positions[index];
        agents[index].velocity = velocities[index];
        agents[index].scalar = scalars[index];
    }
}

fn deterministic_noise_scalar(step: usize, index: usize, amplitude: f64) -> f64 {
    amplitude
        * ((0.19 * step as f64 + 0.71 * index as f64).sin()
            + 0.5 * (0.07 * step as f64 + 0.37 * index as f64).cos())
}

fn deterministic_noise_vector(step: usize, index: usize, amplitude: f64) -> Vector2<f64> {
    Vector2::new(
        amplitude * (0.17 * step as f64 + 0.41 * index as f64).sin(),
        amplitude * (0.11 * step as f64 + 0.29 * index as f64).cos(),
    )
}
