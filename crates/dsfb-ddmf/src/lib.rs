//! Deterministic residual-envelope disturbance modeling framework (DDMF).
//!
//! This crate extends the core `dsfb` workspace with deterministic disturbance
//! generators, single-channel envelope tracking, and Monte Carlo sweep tooling.

pub mod disturbances;
pub mod envelope;
pub mod monte_carlo;
pub mod sim;

pub use disturbances::{build_disturbance, Disturbance, DisturbanceKind};
pub use envelope::{ResidualEnvelope, TrustWeight};
pub use monte_carlo::{
    example_impulse_result, example_persistent_result, run_monte_carlo, MonteCarloBatch,
    MonteCarloConfig, MonteCarloRunRecord, MonteCarloSummary, TrajectoryRow,
};
pub use sim::{
    run_multichannel_simulation, run_simulation, run_simulation_with_s0, SimulationConfig,
    SimulationResult,
};
