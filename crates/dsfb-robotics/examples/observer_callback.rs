//! Wire DSFB into a callback that an external observer pipeline could
//! invoke per timestep. Demonstrates the use case where DSFB augments
//! a streaming control / monitoring loop in real time without
//! interfering with it.
//!
//! Run with:
//!     cargo run --example observer_callback
//!
//! The pattern: the host observer pipeline owns a `DsfbRoboticsEngine`,
//! produces residuals in its main loop, and on each sample (a) hands
//! the residual to its existing downstream consumer, (b) hands a copy
//! to DSFB's `observe_one`. DSFB returns an `Episode` annotation; the
//! host pipeline appends the annotation to a side-channel review log
//! without altering the primary data flow. Removing DSFB removes the
//! side-channel log; the primary control / monitoring loop is
//! unchanged.

use dsfb_robotics::engine::DsfbRoboticsEngine;
use dsfb_robotics::platform::RobotContext;
use dsfb_robotics::Episode;

/// Stand-in for a host observer pipeline's per-sample callback. In a
/// real deployment this would be a Kalman-filter step, an
/// inverse-dynamics residual computation, etc.
fn host_pipeline_callback(k: usize, residual: f64) -> f64 {
    // Whatever the host already does — here we simulate by returning
    // the residual unchanged so the demo is reproducible.
    let _ = k;
    residual
}

/// DSFB augmentation callback: runs the engine on the residual the
/// host produced and returns the annotation. The host's primary path
/// is unaffected.
fn dsfb_augmentation_callback(
    engine: &mut DsfbRoboticsEngine<8, 4>,
    k: usize,
    residual: f64,
) -> Episode {
    // `below_floor` semantics: treat non-finite as below floor.
    let below_floor = !residual.is_finite();
    let norm = if residual.is_finite() { residual.abs() } else { 0.0 };
    engine.observe_one(norm, below_floor, RobotContext::ArmOperating, k)
}

fn main() {
    // Construct the engine once at startup; reuse across every
    // host-loop tick.
    let mut engine = DsfbRoboticsEngine::<8, 4>::new(0.15);
    let mut review_log: Vec<Episode> = Vec::new();

    // Simulated host loop — replace with the real residual stream in
    // a deployment.
    let host_loop_residuals = [
        0.02, 0.03, 0.05, 0.08, 0.12, 0.16, 0.21, 0.13, 0.08, 0.04,
    ];

    for (k, &raw_residual) in host_loop_residuals.iter().enumerate() {
        // 1. Host pipeline does its existing thing.
        let _consumed = host_pipeline_callback(k, raw_residual);
        // 2. DSFB augments alongside, no interference.
        let annotation = dsfb_augmentation_callback(&mut engine, k, raw_residual);
        // 3. Host appends the annotation to a side-channel review log.
        review_log.push(annotation);
    }

    println!("DSFB side-channel review log ({} episodes):", review_log.len());
    for ep in &review_log {
        println!(
            "  k={:>2}  norm²={:.4}  drift={:+.4}  grammar={}",
            ep.index, ep.residual_norm_sq, ep.drift, ep.grammar
        );
    }
}
