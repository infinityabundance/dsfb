//! Deterministic synthetic benchmarking crate for DSFB fusion diagnostics.
//!
//! This library exposes the simulation, method, metric, timing, and output
//! modules used by the `dsfb-fusion-bench` CLI binary.

pub mod io;
pub mod methods;
pub mod metrics;
pub mod sim {
    pub mod diagnostics;
    pub mod faults;
    pub mod state;
}
pub mod timing;
