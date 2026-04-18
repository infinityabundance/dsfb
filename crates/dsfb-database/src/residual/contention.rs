//! Contention residuals.
//!
//! Lock-wait time, queue depth, blocked-by chain length. Per facts #24–32 of
//! the paperstack, contention residuals are *engine-specific* and
//! *multigranular*: InnoDB's intention locks, SQL Server's blocked-process
//! report, Oracle's `V$LOCK`, PostgreSQL's `pg_stat_activity.wait_event`.
//! We store one residual per `(t, wait_event)`; the motif state machine
//! looks for ramps and chain-length growth.

use super::{ResidualClass, ResidualSample, ResidualStream};

/// Wait-time residual (seconds in queue or holding lock).
pub fn push_wait(stream: &mut ResidualStream, t: f64, wait_event: &str, wait_seconds: f64) {
    stream.push(
        ResidualSample::new(t, ResidualClass::Contention, wait_seconds).with_channel(wait_event),
    );
}

/// Blocked-by chain depth (1 = isolated wait; >1 = transitively blocked).
/// Encoded with a `#chain` suffix on the channel so the motif state machine
/// can disambiguate from raw wait residuals.
pub fn push_chain_depth(stream: &mut ResidualStream, t: f64, wait_event: &str, depth: usize) {
    stream.push(
        ResidualSample::new(t, ResidualClass::Contention, depth as f64)
            .with_channel(format!("{wait_event}#chain")),
    );
}
