//! Deterministic terminal dashboard replay for DSFB semiotics runs.
//!
//! The dashboard surface is split into:
//! - typed replay-event construction from completed engine outputs
//! - a dedicated CSV replay driver with deterministic clocking and control state
//! - ASCII rendering via `ratatui`
//!
//! None of these modules recompute residual, syntax, grammar, semantics, or comparator logic in
//! the UI layer itself.

mod build;
mod csv_replay;
mod render;
mod types;

pub use csv_replay::{
    CsvReplayDriver, CsvReplayRunState, CsvReplayTimingState, CSV_REPLAY_STATE_SCHEMA_VERSION,
};
pub use types::{
    DashboardReplay, DashboardReplayConfig, DashboardReplayEvent, DashboardReplayStream,
    DASHBOARD_EVENT_SCHEMA_VERSION,
};
