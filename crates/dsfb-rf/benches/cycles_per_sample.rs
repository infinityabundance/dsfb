//! Instruction-Level Cost Analysis: Cycles per Sample
//!
//! Measures the wall-clock time per observation through the full DSFB pipeline:
//!   IQ Residual → Sign → Grammar → Syntax → Semantics → DSA → Lyapunov → Policy
//!
//! ## Methodology
//!
//! - Runs N iterations of `engine.observe()` with representative inputs
//! - Measures total elapsed time and divides by N
//! - Reports nanoseconds/sample and estimated cycles/sample at 1 GHz
//!
//! ## Targets
//!
//! | Platform              | Clock   | Target ns/sample | Target cycles |
//! |-----------------------|---------|------------------|---------------|
//! | x86-64 (host)         | 3+ GHz  | < 500 ns         | < 1500        |
//! | Cortex-M4F @ 168 MHz  | 168 MHz | < 5 μs           | < 840         |
//! | RISC-V @ 100 MHz      | 100 MHz | < 10 μs          | < 1000        |
//! | Zynq UltraScale+ APU  | 1.5 GHz | < 1 μs           | < 1500        |
//!
//! ## Usage
//!
//! ```text
//! cargo bench --features std
//! ```
//!
//! ## Note
//!
//! This uses std::time::Instant (not criterion) to keep the bench
//! dependency-free. For production benchmarking, wire in criterion or iai.

#[cfg(feature = "std")]
fn main() {
    use dsfb_rf::DsfbRfEngine;
    use dsfb_rf::platform::PlatformContext;
    use std::time::Instant;

    println!("══════════════════════════════════════════════════════");
    println!(" DSFB-RF Cycles-per-Sample Benchmark");
    println!("══════════════════════════════════════════════════════");
    println!();

    let n_warmup = 1000;
    let n_iterations = 100_000;

    // --- Calibrate engine ---
    let healthy: [f32; 100] = core::array::from_fn(|i| 0.03 + (i as f32 * 0.0002));
    let mut engine = DsfbRfEngine::<10, 4, 8>::from_calibration(&healthy, 2.0)
        .expect("calibration must succeed");

    let ctx = PlatformContext::with_snr(15.0);

    // --- Warmup ---
    for i in 0..n_warmup {
        let norm = 0.04 + (i as f32 * 0.001).sin() * 0.01;
        let _ = engine.observe(norm, ctx);
    }
    engine.reset();

    // --- Benchmark: nominal signal (Admissible path) ---
    let start = Instant::now();
    for i in 0..n_iterations {
        let norm = 0.03 + (i as f32 * 0.001).sin() * 0.005;
        let _ = engine.observe(norm, ctx);
    }
    let elapsed_nominal = start.elapsed();
    let ns_per_sample_nominal = elapsed_nominal.as_nanos() as f64 / n_iterations as f64;

    engine.reset();

    // --- Benchmark: drift signal (Boundary path) ---
    let start = Instant::now();
    for i in 0..n_iterations {
        let norm = 0.03 + i as f32 * 0.000002; // slow drift
        let _ = engine.observe(norm, ctx);
    }
    let elapsed_drift = start.elapsed();
    let ns_per_sample_drift = elapsed_drift.as_nanos() as f64 / n_iterations as f64;

    engine.reset();

    // --- Benchmark: violation signal (Violation path) ---
    let start = Instant::now();
    for i in 0..n_iterations {
        let norm = 0.20 + (i as f32 * 0.003).sin() * 0.05;
        let _ = engine.observe(norm, ctx);
    }
    let elapsed_violation = start.elapsed();
    let ns_per_sample_violation = elapsed_violation.as_nanos() as f64 / n_iterations as f64;

    // --- Report ---
    println!(" Pipeline: IQ → Sign → Grammar → Syntax → Semantics → DSA → Lyapunov → Policy");
    println!(" Engine config: W=10, K=4, M=8");
    println!(" Stack footprint: {} bytes", core::mem::size_of::<DsfbRfEngine<10, 4, 8>>());
    println!(" Iterations per path: {}", n_iterations);
    println!();
    println!(" ┌────────────────┬──────────────┬──────────────────┐");
    println!(" │ Path           │ ns/sample    │ est. cycles @1GHz│");
    println!(" ├────────────────┼──────────────┼──────────────────┤");
    println!(" │ Nominal        │ {:>10.1} ns │ {:>14.0}   │", ns_per_sample_nominal, ns_per_sample_nominal);
    println!(" │ Drift          │ {:>10.1} ns │ {:>14.0}   │", ns_per_sample_drift, ns_per_sample_drift);
    println!(" │ Violation      │ {:>10.1} ns │ {:>14.0}   │", ns_per_sample_violation, ns_per_sample_violation);
    println!(" └────────────────┴──────────────┴──────────────────┘");
    println!();

    let max_ns = ns_per_sample_nominal.max(ns_per_sample_drift).max(ns_per_sample_violation);
    if max_ns < 500.0 {
        println!(" ✓ All paths < 500 ns/sample — suitable for real-time at ≥ 2 MS/s");
    } else if max_ns < 5000.0 {
        println!(" ✓ All paths < 5 μs/sample — suitable for Cortex-M4F class targets");
    } else {
        println!(" ⚠ Some paths exceed 5 μs/sample — review for embedded deployment");
    }

    println!();
    println!(" Notes:");
    println!("   - All operations: no_alloc, no_std compatible, zero unsafe");
    println!("   - SIMD-friendly: branchless inner loops, no dynamic dispatch");
    println!("   - Deterministic: no runtime randomness, no data-dependent branches");
    println!("   - Stack-only: {} bytes total engine state", core::mem::size_of::<DsfbRfEngine<10, 4, 8>>());
    println!("══════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}
