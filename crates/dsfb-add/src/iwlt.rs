use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::sweep::deterministic_drive;
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IwltSweep {
    pub entropy_density: Vec<f64>,
    pub avg_increment: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    I,
    R,
    S,
}

pub fn run_iwlt_sweep(
    config: &SimulationConfig,
    lambda_grid: &[f64],
) -> Result<IwltSweep, AddError> {
    let mut entropy_density = Vec::with_capacity(lambda_grid.len());
    let mut avg_increment = Vec::with_capacity(lambda_grid.len());

    for (idx, &lambda) in lambda_grid.iter().enumerate() {
        let lambda_norm = config.normalized_lambda(lambda);
        let drive = deterministic_drive(config.random_seed, lambda, 0x1A17_u64 + idx as u64);
        let mut rng = StdRng::seed_from_u64(config.random_seed ^ 0x1A17_0000_u64 ^ idx as u64);

        let mut history: Vec<Event> = Vec::new();
        let mut entropies = Vec::with_capacity(config.steps_per_run + 1);
        entropies.push(0.0);

        for step in 0..config.steps_per_run {
            let irreversible_bias =
                (0.20 + 0.70 * lambda_norm + 0.08 * drive.phase_bias).clamp(0.0, 1.0);
            let structural_bias =
                (0.10 + 0.20 * (step as f64 * 0.05 + drive.trust_bias).cos()).abs();

            if rng.gen::<f64>() < irreversible_bias {
                history.push(Event::I);
                history.push(Event::S);
            } else if rng.gen::<f64>() < structural_bias {
                history.push(Event::S);
            } else {
                history.push(Event::R);
            }

            history = reduce_history(&history);
            entropies.push(history.len() as f64);
        }

        let final_entropy = *entropies.last().unwrap_or(&0.0);
        let increments: f64 = entropies.windows(2).map(|pair| pair[1] - pair[0]).sum();

        entropy_density.push(final_entropy / config.steps_per_run as f64);
        avg_increment.push(increments / config.steps_per_run as f64);
    }

    Ok(IwltSweep {
        entropy_density,
        avg_increment,
    })
}

fn reduce_history(history: &[Event]) -> Vec<Event> {
    let mut reduced = Vec::with_capacity(history.len());

    for &event in history {
        reduced.push(event);

        loop {
            if reduced.len() < 2 {
                break;
            }

            let len = reduced.len();
            let pair = (reduced[len - 2], reduced[len - 1]);

            match pair {
                (Event::R, Event::R) => {
                    reduced.pop();
                    reduced.pop();
                }
                (Event::R, Event::I) | (Event::R, Event::S) => {
                    let survivor = reduced.pop().unwrap_or(Event::S);
                    reduced.pop();
                    reduced.push(survivor);
                }
                _ => break,
            }
        }
    }

    reduced
}
