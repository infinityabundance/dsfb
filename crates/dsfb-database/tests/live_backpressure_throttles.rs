//! Backpressure throttle / recovery integration test.
//!
//! Feeds measured `(wall, self_time, interval)` triples through the
//! public [`BackpressureState`] state machine and asserts the two
//! invariants documented in the scraper module:
//!
//! 1. A sustained over-budget wall-clock poll duration causes the
//!    next-sleep to double within a bounded number of polls.
//! 2. Once the rolling median returns under budget, `RECOVERY_GOOD_POLLS`
//!    consecutive good polls halve the sleep back toward `interval`.
//!
//! The state machine is pure, so we do not need `tokio::time::pause`
//! — the test drives the machine by passing durations directly.
//! Determinism therefore follows from not reading the wall clock
//! inside the state transitions. (The `PollReport.t_wall_start`
//! field reads `SystemTime::now()` but we do not assert on it.)

#![cfg(feature = "live-postgres")]

use dsfb_database::live::{BackpressureState, Budget};
use std::time::Duration;

#[test]
fn sustained_slow_polls_double_the_sleep() {
    let interval = Duration::from_millis(100);
    let budget = Budget {
        max_poll_ms: 50,
        cpu_pct: 1.0, // disable CPU branch
    };
    let mut bp = BackpressureState::new(interval, budget);
    assert_eq!(bp.current_sleep(), interval);

    // Feed 16 polls that each take 200ms (over a 50ms budget). The
    // rolling median crosses the budget after 9 polls (> half the
    // window); from then on each subsequent over-budget poll doubles
    // the sleep.
    for _ in 0..16 {
        bp.record_and_plan(
            Duration::from_millis(200),
            Duration::from_millis(1),
            Duration::from_millis(100),
        );
    }
    assert!(
        bp.current_sleep() > interval,
        "next-sleep should have grown past the nominal interval under sustained over-budget polls; got {:?}",
        bp.current_sleep()
    );
}

#[test]
fn recovery_halves_sleep_after_good_polls() {
    let interval = Duration::from_millis(100);
    let budget = Budget {
        max_poll_ms: 50,
        cpu_pct: 1.0,
    };
    let mut bp = BackpressureState::new(interval, budget);

    // Saturate: 16 over-budget polls.
    for _ in 0..16 {
        bp.record_and_plan(
            Duration::from_millis(200),
            Duration::from_millis(1),
            Duration::from_millis(100),
        );
    }
    let saturated = bp.current_sleep();
    assert!(saturated > interval);

    // Recovery: enough good polls to flip the rolling median and then
    // trigger a recovery halving. 16 polls refresh the window; 3 more
    // satisfy RECOVERY_GOOD_POLLS.
    for _ in 0..(16 + 3) {
        bp.record_and_plan(
            Duration::from_millis(10),
            Duration::from_millis(1),
            Duration::from_millis(100),
        );
    }
    assert!(
        bp.current_sleep() < saturated,
        "sustained good polls should halve the sleep back; before={:?} after={:?}",
        saturated,
        bp.current_sleep()
    );
}

#[test]
fn cpu_budget_breach_triggers_throttle_independently_of_wall_clock() {
    // A poll can be fast (small wall clock) but CPU-hungry relative
    // to the interval since the last poll. The scraper must still
    // throttle on that signal — so an operator can cap the
    // observer's CPU cost even on a very fast engine.
    let interval = Duration::from_millis(1000);
    let budget = Budget {
        max_poll_ms: 5_000, // very lax wall-clock budget
        cpu_pct: 0.05,      // 5 % CPU ceiling
    };
    let mut bp = BackpressureState::new(interval, budget);

    // Self-time 200 ms per 1000 ms interval = 20 % CPU, well over
    // the 5 % ceiling. After 16 polls this is reflected in the
    // rolling ratio.
    for _ in 0..16 {
        bp.record_and_plan(
            Duration::from_millis(200),  // fast wall clock
            Duration::from_millis(200),  // but 200 ms of self-time
            Duration::from_millis(1000), // over a 1 s interval
        );
    }
    assert!(
        bp.current_sleep() > interval,
        "CPU-budget breach alone should have throttled the sleep; got {:?}",
        bp.current_sleep()
    );
}
