pub mod aet;
pub mod analysis;
pub mod config;
pub mod iwlt;
pub mod output;
pub mod rlt;
pub mod sweep;
pub mod tcp;

use thiserror::Error;

pub use aet::AetSweep;
pub use config::SimulationConfig;
pub use iwlt::IwltSweep;
pub use output::create_timestamped_output_dir;
pub use rlt::RltSweep;
pub use sweep::{run_sweeps_into_dir, SweepResult};
pub use tcp::{TcpPoint, TcpSweep};

#[derive(Debug, Error)]
pub enum AddError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("{context} length mismatch: expected {expected}, got {got}")]
    LengthMismatch {
        context: &'static str,
        expected: usize,
        got: usize,
    },
}

pub fn run_all_sweeps(config: &SimulationConfig) -> Result<(), AddError> {
    let output_dir = create_timestamped_output_dir()?;
    sweep::run_sweeps_into_dir(config, &output_dir)?;
    Ok(())
}
