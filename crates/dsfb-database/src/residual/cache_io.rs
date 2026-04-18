//! Cache / buffer collapse residuals.
//!
//! Two channels per source:
//!   * **hit-ratio drop** — `expected_hit_ratio − observed_hit_ratio`,
//!     positive when the cache is failing.
//!   * **I/O wait amplification** — `observed_io_wait_seconds /
//!     baseline_io_wait_seconds − 1.0`, positive when I/O is slower than
//!     the rolling baseline.
//!
//! In PostgreSQL 16+ both are first-class via `pg_stat_io.hits` and
//! `pg_stat_io.read_time` (fact #44). In SQL Server they are reachable via
//! `sys.dm_os_buffer_descriptors` plus Query Store I/O wait stats. In Oracle
//! they are in `V$SEGMENT_STATISTICS` plus `V$SYSTEM_EVENT`.

use super::{ResidualClass, ResidualSample, ResidualStream};

pub fn push_hit_ratio(
    stream: &mut ResidualStream,
    t: f64,
    cache_id: &str,
    expected: f64,
    observed: f64,
) {
    let drop = expected - observed;
    stream.push(ResidualSample::new(t, ResidualClass::CacheIo, drop).with_channel(cache_id));
}

pub fn push_io_amplification(
    stream: &mut ResidualStream,
    t: f64,
    file_id: &str,
    observed_seconds: f64,
    baseline_seconds: f64,
) {
    let amp = if baseline_seconds > 0.0 {
        observed_seconds / baseline_seconds - 1.0
    } else {
        0.0
    };
    stream.push(
        ResidualSample::new(t, ResidualClass::CacheIo, amp).with_channel(format!("{file_id}#io")),
    );
}
