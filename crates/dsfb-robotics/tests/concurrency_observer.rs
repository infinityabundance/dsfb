//! Loom-backed concurrency exploration of the DSFB observer.
//!
//! DSFB is deliberately a **single-threaded, purely-functional**
//! observer: `DsfbRoboticsEngine::observe` takes `&mut self` and a
//! `&[f64]` input, so the Rust borrow checker already rules out any
//! data race at compile time. There is no `Arc`, no `Mutex`, no
//! atomic, no interior mutability anywhere in the core.
//!
//! Loom is nevertheless useful as a **negative-space** test: it asserts
//! that we have not accidentally introduced a concurrency hazard
//! later. Under `#[cfg(loom)]`, this test runs the observer pipeline
//! and, if a future refactor adds `Arc<AtomicXxx>` or similar, loom's
//! exhaustive schedule exploration will flag ordering violations that
//! normal testing would miss.
//!
//! Run with:
//!
//! ```bash
//! RUSTFLAGS="--cfg loom" cargo +nightly test \
//!     --manifest-path crates/dsfb-robotics/Cargo.toml \
//!     --features std --test concurrency_observer
//! ```
//!
//! In a non-loom build this file compiles to a single sanity test
//! asserting the single-threaded baseline: two sequential observe
//! calls are deterministic.

#![cfg(feature = "std")]

use dsfb_robotics::datasets::kuka_lwr;
use dsfb_robotics::platform::RobotContext;
use dsfb_robotics::{DsfbRoboticsEngine, Episode};

#[cfg(not(loom))]
#[test]
fn two_sequential_observes_agree_on_deterministic_output() {
    // Baseline determinism test — actually runs under stock `cargo test`.
    // When loom is active, the `#[cfg(loom)]` test below supersedes this.
    let mut residuals = [0.0_f64; 6];
    let n = kuka_lwr::fixture_residuals(&mut residuals);

    let mut eng_a = DsfbRoboticsEngine::<8, 4>::new(0.1);
    let mut eng_b = DsfbRoboticsEngine::<8, 4>::new(0.1);
    let mut out_a = [Episode::empty(); 6];
    let mut out_b = [Episode::empty(); 6];
    let na = eng_a.observe(&residuals[..n], &mut out_a, RobotContext::ArmOperating);
    let nb = eng_b.observe(&residuals[..n], &mut out_b, RobotContext::ArmOperating);

    assert_eq!(na, nb);
    assert_eq!(&out_a[..na], &out_b[..nb]);
}

#[cfg(not(loom))]
#[test]
fn observer_contract_preserves_input_byte_identity() {
    // Belt-and-braces check: the compile-time `&[f64]` signature
    // already rules out mutation. If a future change replaces it with
    // `&mut [f64]` this test will drift away from the input-preservation
    // claim, acting as a tripwire.
    let mut residuals = [0.0_f64; 6];
    let n = kuka_lwr::fixture_residuals(&mut residuals);
    let before: [u64; 6] = core::array::from_fn(|i| residuals[i].to_bits());

    let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
    let mut out = [Episode::empty(); 6];
    let _ = eng.observe(&residuals[..n], &mut out, RobotContext::ArmOperating);

    let after: [u64; 6] = core::array::from_fn(|i| residuals[i].to_bits());
    assert_eq!(before, after, "observer contract violated: input slice mutated");
}

#[cfg(loom)]
#[test]
fn observe_under_loom_schedule_exploration() {
    // Under loom, single-threaded observe is a baseline for future
    // multi-threaded extensions. If an atomic is later added, loom's
    // scheduler will exhaustively explore interleavings.
    loom::model(|| {
        let mut residuals = [0.0_f64; 6];
        let n = kuka_lwr::fixture_residuals(&mut residuals);
        let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
        let mut out = [Episode::empty(); 6];
        let count = eng.observe(&residuals[..n], &mut out, RobotContext::ArmOperating);
        assert_eq!(count, n);
        for e in &out[..count] {
            assert!(matches!(e.grammar, "Admissible" | "Boundary" | "Violation"));
        }
    });
}

/// Loom matrix entry 2: two non-overlapping engines observe independent
/// streams concurrently. This is the canonical "library users may run
/// many engines from many threads" scenario. Under loom this exhaustively
/// explores every possible thread interleaving.
///
/// If a future refactor accidentally introduces shared mutable state
/// (a `static mut`, a `Mutex`, an `Arc`), loom will flag the resulting
/// non-determinism. Today the engine has no such state, so the test
/// passes deterministically across all explored schedules.
#[cfg(loom)]
#[test]
fn two_engines_two_threads_no_shared_state() {
    use loom::sync::Arc;
    use loom::thread;

    loom::model(|| {
        // Engines and outputs live on each thread's stack — Arc is only
        // used for the shared (immutable) input residuals array, which
        // models how callers might share read-only telemetry.
        let mut residuals = [0.0_f64; 6];
        let n = kuka_lwr::fixture_residuals(&mut residuals);
        let shared: Arc<[f64; 6]> = Arc::new(residuals);

        let s1 = Arc::clone(&shared);
        let t1 = thread::spawn(move || {
            let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
            let mut out = [Episode::empty(); 6];
            let c = eng.observe(&s1[..n], &mut out, RobotContext::ArmOperating);
            assert_eq!(c, n);
            // Return the committed grammar of the last episode for cross-thread sanity.
            out[c - 1].grammar
        });

        let s2 = Arc::clone(&shared);
        let t2 = thread::spawn(move || {
            let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
            let mut out = [Episode::empty(); 6];
            let c = eng.observe(&s2[..n], &mut out, RobotContext::ArmOperating);
            assert_eq!(c, n);
            out[c - 1].grammar
        });

        let g1 = t1.join().unwrap();
        let g2 = t2.join().unwrap();
        // Two independent engines on identical inputs must produce identical
        // outputs — irrespective of thread interleaving.
        assert_eq!(g1, g2, "engine output diverged across thread interleaving");
    });
}

/// Loom matrix entry 3: read-after-write boundary — one thread observes
/// the residual stream, then a second thread reads (but does not write)
/// the same input. Verifies that the input-preservation contract holds
/// under arbitrary thread schedules.
#[cfg(loom)]
#[test]
fn observe_then_read_no_input_mutation() {
    use loom::sync::Arc;
    use loom::thread;

    loom::model(|| {
        let mut residuals = [0.0_f64; 6];
        let n = kuka_lwr::fixture_residuals(&mut residuals);
        let initial: [u64; 6] = core::array::from_fn(|i| residuals[i].to_bits());
        let shared: Arc<[f64; 6]> = Arc::new(residuals);

        let s_obs = Arc::clone(&shared);
        let t_obs = thread::spawn(move || {
            let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
            let mut out = [Episode::empty(); 6];
            let c = eng.observe(&s_obs[..n], &mut out, RobotContext::ArmOperating);
            assert_eq!(c, n);
        });

        let s_read = Arc::clone(&shared);
        let t_read = thread::spawn(move || {
            let bits: [u64; 6] = core::array::from_fn(|i| s_read[i].to_bits());
            bits
        });

        t_obs.join().unwrap();
        let read_bits = t_read.join().unwrap();
        assert_eq!(
            initial, read_bits,
            "input residuals slice mutated across thread interleaving"
        );
    });
}
