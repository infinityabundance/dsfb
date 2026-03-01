use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::sweep::deterministic_drive;
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetSweep {
    pub echo_slope: Vec<f64>,
    pub avg_increment: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    A,
    B,
}

pub fn run_aet_sweep(config: &SimulationConfig, lambda_grid: &[f64]) -> Result<AetSweep, AddError> {
    let mut echo_slope = Vec::with_capacity(lambda_grid.len());
    let mut avg_increment = Vec::with_capacity(lambda_grid.len());

    for (idx, &lambda) in lambda_grid.iter().enumerate() {
        let lambda_norm = config.normalized_lambda(lambda);
        let drive = deterministic_drive(config.random_seed, lambda, 0xAE70_u64 + idx as u64);
        let mut rng = StdRng::seed_from_u64(config.random_seed ^ 0xA370_0000_u64 ^ idx as u64);

        let mut word = reduce_word(&[Symbol::A]);
        let mut lengths = Vec::with_capacity(config.steps_per_run + 1);
        lengths.push(word.len() as f64);

        for step in 0..config.steps_per_run {
            let phase_term = ((step as f64) * 0.03125 + drive.phase_bias).sin() * 0.05;
            let growth_bias =
                (0.12 + 0.76 * lambda_norm + 0.10 * drive.phase_bias + phase_term).clamp(0.0, 1.0);

            let generator = if rng.gen::<f64>() < growth_bias {
                Symbol::A
            } else {
                Symbol::B
            };

            let mut candidate = Vec::with_capacity(word.len() + 1);
            candidate.push(generator);
            candidate.extend_from_slice(&word);
            word = reduce_word(&candidate);
            lengths.push(word.len() as f64);
        }

        let initial = lengths[0];
        let final_length = *lengths.last().unwrap_or(&initial);
        let increments: f64 = lengths.windows(2).map(|pair| pair[1] - pair[0]).sum();

        echo_slope.push((final_length - initial) / config.steps_per_run as f64);
        avg_increment.push(increments / config.steps_per_run as f64);
    }

    Ok(AetSweep {
        echo_slope,
        avg_increment,
    })
}

fn reduce_word(word: &[Symbol]) -> Vec<Symbol> {
    let mut reduced = Vec::with_capacity(word.len());

    for &symbol in word {
        reduced.push(symbol);

        loop {
            if reduced.len() < 2 {
                break;
            }

            let len = reduced.len();
            let pair = (reduced[len - 2], reduced[len - 1]);

            match pair {
                (Symbol::B, Symbol::A) => {
                    let protected = reduced.pop().unwrap_or(Symbol::A);
                    reduced.pop();
                    reduced.push(protected);
                }
                (Symbol::B, Symbol::B) => {
                    reduced.pop();
                    reduced.pop();
                }
                _ => break,
            }
        }
    }

    reduced
}
