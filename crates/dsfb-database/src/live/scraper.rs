//! Pulsed-scrape loop with measured backpressure.
//!
//! The scraper issues every variant of [`super::queries::AllowedQuery`]
//! once per tick and returns a bundled [`super::distiller::Snapshot`].
//! It maintains a rolling window of the last 16 poll wall-clock
//! durations plus a coarse self-time / wall-clock CPU ratio, and
//! adjusts the next inter-poll sleep:
//!
//! * if median poll wall-clock exceeds `budget.max_poll_ms`, the next
//!   sleep doubles (bounded at 60 s);
//! * if the rolling CPU ratio exceeds `budget.cpu_pct`, the next sleep
//!   doubles (bounded at 60 s);
//! * if three consecutive polls are within budget, the next sleep
//!   halves back toward the nominal `interval`.
//!
//! **This is a measurement-based governor, not a contract.** It does
//! not guarantee a CPU bound; it reacts to observed pressure with a
//! bounded-exponential back-off. The paper's 7th non-claim and the
//! §Live section both state this explicitly.
//!
//! The scraper also writes a telemetry-of-the-telemetry row to
//! `out/live/poll_log.csv` each tick — the operator can see exactly
//! how much the observer is costing them.

use super::distiller::{ActivityRow, PgssRow, Snapshot, StatDatabaseRow, StatIoRow};
use super::queries::AllowedQuery;
use super::readonly_conn::ReadOnlyPgConn;
use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Resource budget for the scraper.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Hard upper bound on a single poll's wall-clock duration. If
    /// exceeded, the next inter-poll sleep is doubled.
    pub max_poll_ms: u64,
    /// Rolling CPU ratio ceiling (self-time / wall-clock). If
    /// exceeded, the next inter-poll sleep is doubled.
    pub cpu_pct: f64,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_poll_ms: 500,
            cpu_pct: 0.1,
        }
    }
}

pub const ROLLING_WINDOW: usize = 16;
pub const RECOVERY_GOOD_POLLS: usize = 3;
pub const MAX_SLEEP: Duration = Duration::from_secs(60);
pub const MIN_SLEEP: Duration = Duration::from_millis(50);

/// Pure backpressure state machine extracted from [`Scraper`].
///
/// This is the same logic the scraper runs inline, extracted so that
/// integration tests (`tests/live_backpressure_throttles.rs`) can
/// exercise it without needing a live PostgreSQL connection. Pure by
/// construction: every input is a measured duration, every output is
/// a deterministic plan.
#[derive(Debug, Clone)]
pub struct BackpressureState {
    interval: Duration,
    budget: Budget,
    rolling_wall_ms: VecDeque<u64>,
    rolling_self_ms: VecDeque<u64>,
    rolling_interval_ms: VecDeque<u64>,
    consecutive_good: usize,
    current_sleep: Duration,
}

impl BackpressureState {
    pub fn new(interval: Duration, budget: Budget) -> Self {
        Self {
            interval,
            budget,
            rolling_wall_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            rolling_self_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            rolling_interval_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            consecutive_good: 0,
            current_sleep: interval,
        }
    }

    pub fn record_and_plan(
        &mut self,
        wall: Duration,
        self_time: Duration,
        interval_since_last: Duration,
    ) -> PollReport {
        push_bounded(&mut self.rolling_wall_ms, wall.as_millis() as u64);
        push_bounded(&mut self.rolling_self_ms, self_time.as_millis() as u64);
        push_bounded(
            &mut self.rolling_interval_ms,
            interval_since_last.as_millis() as u64,
        );
        let median_wall = median(&self.rolling_wall_ms);
        let cpu_pct = rolling_cpu_ratio(&self.rolling_self_ms, &self.rolling_interval_ms);
        let over_budget =
            median_wall > self.budget.max_poll_ms || cpu_pct > self.budget.cpu_pct;
        if over_budget {
            self.consecutive_good = 0;
            self.current_sleep = (self.current_sleep * 2).min(MAX_SLEEP);
        } else {
            self.consecutive_good += 1;
            if self.consecutive_good >= RECOVERY_GOOD_POLLS {
                let halved = self.current_sleep / 2;
                self.current_sleep = halved.max(self.interval).max(MIN_SLEEP);
                self.consecutive_good = 0;
            }
        }
        let throttle_factor = self.current_sleep.as_secs_f64() / self.interval.as_secs_f64();
        PollReport {
            t_wall_start: unix_epoch_seconds(),
            snapshot_duration_ms: wall.as_millis() as u64,
            cpu_pct_rolling: cpu_pct,
            throttle_factor,
        }
    }

    pub fn current_sleep(&self) -> Duration {
        self.current_sleep
    }

    pub fn nominal_interval(&self) -> Duration {
        self.interval
    }
}

/// Per-poll self-report written to `poll_log.csv`.
#[derive(Debug, Clone)]
pub struct PollReport {
    pub t_wall_start: f64,
    pub snapshot_duration_ms: u64,
    pub cpu_pct_rolling: f64,
    pub throttle_factor: f64,
}

/// Pulsed scraper over a [`ReadOnlyPgConn`]. Owns the budget policy
/// and the rolling measurement window.
pub struct Scraper {
    conn: ReadOnlyPgConn,
    interval: Duration,
    budget: Budget,
    rolling_wall_ms: VecDeque<u64>,
    rolling_self_ms: VecDeque<u64>,
    rolling_interval_ms: VecDeque<u64>,
    consecutive_good: usize,
    current_sleep: Duration,
}

impl Scraper {
    pub fn new(conn: ReadOnlyPgConn, interval: Duration, budget: Budget) -> Self {
        Self {
            conn,
            interval,
            budget,
            rolling_wall_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            rolling_self_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            rolling_interval_ms: VecDeque::with_capacity(ROLLING_WINDOW),
            consecutive_good: 0,
            current_sleep: interval,
        }
    }

    /// Execute one poll cycle (every AllowedQuery variant, in the
    /// [`AllowedQuery::ALL`] order) and return a bundled Snapshot.
    /// Returns the wall-clock duration alongside the Snapshot so the
    /// caller can record it.
    pub async fn next_snapshot(&mut self) -> Result<(Snapshot, Duration)> {
        let start = Instant::now();
        let t_abs = unix_epoch_seconds();
        let mut snap = Snapshot::default();
        snap.t = t_abs;
        for q in AllowedQuery::ALL.iter() {
            let rows = match self.conn.query_allowed(*q).await {
                Ok(r) => r,
                Err(e) => {
                    // pg_stat_io does not exist on PG < 16. Treat a
                    // row-not-found-style error softly — we fall back
                    // to pg_stat_database at distillation time.
                    if matches!(q, AllowedQuery::PgStatIoSnapshot) {
                        eprintln!(
                            "warning: {:?} query failed (likely PG <16); falling back to pg_stat_database: {}",
                            q, e
                        );
                        Vec::new()
                    } else {
                        return Err(e).with_context(|| format!("poll failed on {:?}", q));
                    }
                }
            };
            decode_into_snapshot(*q, rows, &mut snap)?;
        }
        let wall = start.elapsed();
        Ok((snap, wall))
    }

    /// Record a completed poll's measurements and compute the next
    /// inter-poll sleep. Returns a [`PollReport`] that the caller
    /// appends to `poll_log.csv`.
    pub fn record_and_plan(
        &mut self,
        wall: Duration,
        self_time: Duration,
        interval_since_last: Duration,
    ) -> PollReport {
        push_bounded(&mut self.rolling_wall_ms, wall.as_millis() as u64);
        push_bounded(&mut self.rolling_self_ms, self_time.as_millis() as u64);
        push_bounded(
            &mut self.rolling_interval_ms,
            interval_since_last.as_millis() as u64,
        );
        let median_wall = median(&self.rolling_wall_ms);
        let cpu_pct = rolling_cpu_ratio(&self.rolling_self_ms, &self.rolling_interval_ms);
        let over_budget = median_wall > self.budget.max_poll_ms
            || cpu_pct > self.budget.cpu_pct;
        if over_budget {
            self.consecutive_good = 0;
            self.current_sleep = (self.current_sleep * 2).min(MAX_SLEEP);
        } else {
            self.consecutive_good += 1;
            if self.consecutive_good >= RECOVERY_GOOD_POLLS {
                let halved = self.current_sleep / 2;
                self.current_sleep = halved.max(self.interval).max(MIN_SLEEP);
                self.consecutive_good = 0;
            }
        }
        let throttle_factor = self.current_sleep.as_secs_f64() / self.interval.as_secs_f64();
        PollReport {
            t_wall_start: unix_epoch_seconds(),
            snapshot_duration_ms: wall.as_millis() as u64,
            cpu_pct_rolling: cpu_pct,
            throttle_factor,
        }
    }

    /// Next inter-poll sleep after any throttling adjustments.
    pub fn next_sleep(&self) -> Duration {
        self.current_sleep
    }

    pub fn nominal_interval(&self) -> Duration {
        self.interval
    }
}

fn push_bounded<T>(dq: &mut VecDeque<T>, v: T) {
    if dq.len() == ROLLING_WINDOW {
        dq.pop_front();
    }
    dq.push_back(v);
}

fn median(dq: &VecDeque<u64>) -> u64 {
    if dq.is_empty() {
        return 0;
    }
    let mut v: Vec<u64> = dq.iter().copied().collect();
    v.sort_unstable();
    v[v.len() / 2]
}

fn rolling_cpu_ratio(self_ms: &VecDeque<u64>, interval_ms: &VecDeque<u64>) -> f64 {
    let s: u64 = self_ms.iter().sum();
    let i: u64 = interval_ms.iter().sum();
    if i == 0 {
        return 0.0;
    }
    s as f64 / i as f64
}

fn unix_epoch_seconds() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn decode_into_snapshot(
    q: AllowedQuery,
    rows: Vec<tokio_postgres::Row>,
    snap: &mut Snapshot,
) -> Result<()> {
    match q {
        AllowedQuery::PgStatStatementsSnapshot => {
            for r in rows {
                let _t: f64 = r.try_get::<_, f64>(0).unwrap_or(0.0);
                let qid: String = r.try_get::<_, String>(1).unwrap_or_default();
                let calls: i64 = r.try_get::<_, i64>(2).unwrap_or(0);
                let total: f64 = r.try_get::<_, f64>(3).unwrap_or(0.0);
                snap.pgss.push(PgssRow {
                    query_id: qid,
                    calls: calls.max(0) as u64,
                    total_exec_time_ms: total,
                });
            }
        }
        AllowedQuery::PgStatActivitySnapshot => {
            for r in rows {
                let _t: f64 = r.try_get::<_, f64>(0).unwrap_or(0.0);
                let wet: String = r.try_get::<_, String>(1).unwrap_or_default();
                let we: String = r.try_get::<_, String>(2).unwrap_or_default();
                let state: String = r
                    .try_get::<_, Option<String>>(3)
                    .unwrap_or_default()
                    .unwrap_or_default();
                snap.activity.push(ActivityRow {
                    wait_event_type: wet,
                    wait_event: we,
                    state,
                });
            }
        }
        AllowedQuery::PgStatIoSnapshot => {
            for r in rows {
                let _t: f64 = r.try_get::<_, f64>(0).unwrap_or(0.0);
                let backend: String = r.try_get::<_, String>(1).unwrap_or_default();
                let object: String = r.try_get::<_, String>(2).unwrap_or_default();
                let context: String = r.try_get::<_, String>(3).unwrap_or_default();
                let reads: i64 = r.try_get::<_, i64>(4).unwrap_or(0);
                let hits: i64 = r.try_get::<_, i64>(5).unwrap_or(0);
                let rt: f64 = r.try_get::<_, f64>(6).unwrap_or(0.0);
                snap.stat_io.push(StatIoRow {
                    backend_type: backend,
                    object,
                    context,
                    reads: reads.max(0) as u64,
                    hits: hits.max(0) as u64,
                    read_time_ms: rt,
                });
            }
        }
        AllowedQuery::PgStatDatabaseSnapshot => {
            for r in rows {
                let _t: f64 = r.try_get::<_, f64>(0).unwrap_or(0.0);
                let datname: String = r.try_get::<_, String>(1).unwrap_or_default();
                let hits: i64 = r.try_get::<_, i64>(2).unwrap_or(0);
                let reads: i64 = r.try_get::<_, i64>(3).unwrap_or(0);
                snap.stat_database.push(StatDatabaseRow {
                    datname,
                    blks_hit: hits.max(0) as u64,
                    blks_read: reads.max(0) as u64,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The scraper's backpressure algorithm is pure state — we can
    // exercise it without a live connection by feeding it measured
    // (wall, self, interval) triples directly. The same logic
    // governs the live binary.
    //
    // We build a `Scraper` through a private test-only constructor
    // that skips the real `tokio_postgres` connection step.
    //
    // NOTE: in production, `Scraper::new` requires a live
    // `ReadOnlyPgConn`. We use the backpressure state machine
    // separately in `tests/live_backpressure_throttles.rs`.

    fn bp_state(interval: Duration, budget: Budget) -> BackpressureState {
        BackpressureState {
            interval,
            budget,
            rolling_wall_ms: VecDeque::new(),
            rolling_self_ms: VecDeque::new(),
            rolling_interval_ms: VecDeque::new(),
            consecutive_good: 0,
            current_sleep: interval,
        }
    }

    #[test]
    fn doubles_under_sustained_slow_response() {
        let mut st = bp_state(
            Duration::from_millis(100),
            Budget {
                max_poll_ms: 50,
                cpu_pct: 1.0, // disable CPU branch
            },
        );
        for _ in 0..ROLLING_WINDOW {
            st.record_and_plan(
                Duration::from_millis(200),
                Duration::from_millis(1),
                Duration::from_millis(100),
            );
        }
        assert!(
            st.current_sleep > Duration::from_millis(100),
            "next-sleep should have doubled at least once under sustained slow response, got {:?}",
            st.current_sleep
        );
    }

    #[test]
    fn recovers_once_rolling_median_drops() {
        // Recovery kicks in the cycle after the rolling median falls
        // back below budget. With a 16-poll window, ROLLING_WINDOW/2
        // + 1 good polls flip the median; RECOVERY_GOOD_POLLS
        // consecutive good polls after that halve the sleep.
        let mut st = bp_state(
            Duration::from_millis(100),
            Budget {
                max_poll_ms: 50,
                cpu_pct: 1.0,
            },
        );
        for _ in 0..ROLLING_WINDOW {
            st.record_and_plan(
                Duration::from_millis(200),
                Duration::from_millis(1),
                Duration::from_millis(100),
            );
        }
        let saturated = st.current_sleep;
        assert!(saturated > Duration::from_millis(100));
        // Fill the rolling window with good polls so the median
        // drops, then observe recovery.
        for _ in 0..(ROLLING_WINDOW + RECOVERY_GOOD_POLLS) {
            st.record_and_plan(
                Duration::from_millis(10),
                Duration::from_millis(1),
                Duration::from_millis(100),
            );
        }
        assert!(
            st.current_sleep < saturated,
            "sustained good polls should halve the sleep; before={:?} after={:?}",
            saturated,
            st.current_sleep
        );
    }

    /// Test-only shim mirroring the backpressure state inside
    /// `Scraper`. Construction of a real `Scraper` requires a live
    /// PostgreSQL; the state-machine logic is exercised here and by
    /// `tests/live_backpressure_throttles.rs`.
    struct BackpressureState {
        interval: Duration,
        budget: Budget,
        rolling_wall_ms: VecDeque<u64>,
        rolling_self_ms: VecDeque<u64>,
        rolling_interval_ms: VecDeque<u64>,
        consecutive_good: usize,
        current_sleep: Duration,
    }

    impl BackpressureState {
        fn record_and_plan(
            &mut self,
            wall: Duration,
            self_time: Duration,
            interval_since_last: Duration,
        ) {
            push_bounded(&mut self.rolling_wall_ms, wall.as_millis() as u64);
            push_bounded(&mut self.rolling_self_ms, self_time.as_millis() as u64);
            push_bounded(
                &mut self.rolling_interval_ms,
                interval_since_last.as_millis() as u64,
            );
            let median_wall = median(&self.rolling_wall_ms);
            let cpu_pct = rolling_cpu_ratio(&self.rolling_self_ms, &self.rolling_interval_ms);
            let over_budget = median_wall > self.budget.max_poll_ms
                || cpu_pct > self.budget.cpu_pct;
            if over_budget {
                self.consecutive_good = 0;
                self.current_sleep = (self.current_sleep * 2).min(MAX_SLEEP);
            } else {
                self.consecutive_good += 1;
                if self.consecutive_good >= RECOVERY_GOOD_POLLS {
                    let halved = self.current_sleep / 2;
                    self.current_sleep = halved.max(self.interval).max(MIN_SLEEP);
                    self.consecutive_good = 0;
                }
            }
        }
    }
}
