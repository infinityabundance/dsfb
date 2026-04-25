//! Minimal smoke example: construct an engine, push a handful of
//! residual samples, print the committed grammar state per sample.
//!
//! Run with:
//!     cargo run --example engine_smoke
//!
//! Demonstrates the core no_std API surface: [`DsfbRoboticsEngine::new`]
//! constructs an engine pinned to compile-time `W` (drift window) and
//! `K` (grazing window), [`DsfbRoboticsEngine::observe`] streams a
//! caller-owned `&[f64]` of residual norms into a caller-owned
//! `&mut [Episode]` output buffer, and the engine reads only — no
//! mutation of the input slice, no heap allocation in the hot path.

use dsfb_robotics::engine::DsfbRoboticsEngine;
use dsfb_robotics::platform::RobotContext;
use dsfb_robotics::Episode;

fn main() {
    // Twelve hand-crafted residual norms representative of a rising
    // trajectory followed by a small recovery — exactly the shape a
    // bearing knee point or a wrench-residual spike produces.
    let residuals: &[f64] = &[
        0.01, 0.02, 0.04, 0.07, 0.11, 0.16, 0.22, 0.29, 0.18, 0.10, 0.05, 0.03,
    ];

    // Calibrate the engine with rho = 0.20. This would normally come
    // from Stage III calibration over a healthy window; here we hard-
    // code it for didactic clarity.
    let mut engine = DsfbRoboticsEngine::<8, 4>::new(0.20);

    // Caller owns the output buffer — no heap allocation in the engine.
    let mut episodes = [Episode::empty(); 16];
    let n = engine.observe(residuals, &mut episodes, RobotContext::ArmOperating);

    println!(" k    norm    grammar");
    for (i, ep) in episodes[..n].iter().enumerate() {
        println!("{:>3}  {:>6.3}    {}", i, residuals[i], ep.grammar);
    }
}
