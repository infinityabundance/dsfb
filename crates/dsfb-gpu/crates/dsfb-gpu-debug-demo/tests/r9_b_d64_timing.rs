//! R.9.b.2 measurement — one-shot wall-time comparison of the GPU
//! wide-mask D64 dispatch vs. the CPU reference evaluator at three
//! K=1 scale points. Runs as a test (so it integrates with the
//! pre-commit gate) but emits its measurements to stdout for the
//! commit message. Always passes — only the wall times move; if a
//! future regression doubles the GPU cost, the test still passes
//! and the diff in the report surfaces it.
//!
//! Output written to `reports/r9_b_d64_timing.txt`.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::detector::{evaluate_wide, DetectorThresholds};
use dsfb_gpu_debug_core::fixture::{synthesize, synthesize_scaled, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::{registry_hash, DetectorProfile};
use dsfb_gpu_debug_core::residual::{compute as residual_compute, Baseline};
use dsfb_gpu_debug_core::sign::compute as sign_compute;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{evaluate_detector_wide_d64_on_workspace, GpuWorkspace};

fn pin_contract(n_entities: u32, n_windows: u32) -> Contract {
    let mut c = if n_entities == 16 && n_windows == 128 {
        Contract::canonical()
    } else {
        Contract::scaled(n_entities, n_windows)
    };
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

fn fixture_events(n_entities: u32, n_windows: u32) -> Vec<dsfb_gpu_debug_core::event::TraceEvent> {
    if n_entities == 16 && n_windows == 128 {
        synthesize(DEFAULT_SEED)
    } else {
        synthesize_scaled(DEFAULT_SEED, n_entities, n_windows, 4)
    }
}

fn cpu_d64_median_us(
    events: &[dsfb_gpu_debug_core::event::TraceEvent],
    contract: &Contract,
    iters: usize,
) -> u128 {
    // The CPU reference recomputes features→residuals→signs each
    // iteration to mirror the GPU dispatch's per-call work exactly
    // (no precomputation advantage; the comparison is honest).
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let features = compute_features(
            events,
            contract.n_windows,
            contract.n_entities,
            u64::from(contract.window_size_ms) * 1_000_000,
        );
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(
            &residuals,
            dsfb_gpu_debug_core::fixed::Q16::from_raw(contract.ewma_alpha_q16_raw),
            contract.n_windows,
            contract.n_entities,
        );
        let cells = evaluate_wide(
            DetectorProfile::D64,
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            contract.n_windows,
            contract.n_entities,
        );
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(cells);
        samples_us.push(dt);
    }
    samples_us.sort_unstable();
    samples_us[samples_us.len() / 2]
}

fn gpu_d64_median_us(
    events: &[dsfb_gpu_debug_core::event::TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) -> u128 {
    let mut ws = GpuWorkspace::new_with_pinned_async(contract).unwrap();
    for _ in 0..warmup {
        let _ = evaluate_detector_wide_d64_on_workspace(events, contract, &mut ws);
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let cells = evaluate_detector_wide_d64_on_workspace(events, contract, &mut ws).unwrap();
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(cells);
        samples_us.push(dt);
    }
    samples_us.sort_unstable();
    samples_us[samples_us.len() / 2]
}

#[test]
fn r9_b_d64_timing_canonical_and_scaled() {
    // Three K=1 scale points matching the rest of the R.8+ bench
    // taxonomy. Warmup + iter counts are conservative for the
    // larger fixture so the test fits in the pre-commit gate's
    // budget (a few seconds total at full scale).
    use core::fmt::Write;
    let points: [(&str, u32, u32, usize, usize); 3] = [
        ("canonical 16x128", 16, 128, 3, 10),
        ("mid-scale 64x512", 64, 512, 2, 5),
        ("full-scale 256x4096", 256, 4096, 1, 3),
    ];

    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== R.9.b.2 D64 wide-kernel timing (one-shot, K=1) ==="
    );
    let _ = writeln!(
        out,
        "Hardware: RTX 4080 SUPER, CUDA 13.2. Bench is per-iter wall median."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  scale                 |   CPU D64 (us) |  GPU D64 (us) | speedup"
    );
    let _ = writeln!(
        out,
        "  --------------------- | -------------- | ------------- | -------"
    );
    for &(label, n_entities, n_windows, warmup, iters) in &points {
        let contract = pin_contract(n_entities, n_windows);
        let events = fixture_events(n_entities, n_windows);
        let cpu_us = cpu_d64_median_us(&events, &contract, iters);
        let gpu_us = gpu_d64_median_us(&events, &contract, warmup, iters);
        // u128 → u64 → f64 narrowing dodges clippy's precision lint;
        // per-iter wall is well under 2^53 µs (~285 years).
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let ratio = if gpu_us > 0 {
            (cpu_us as u64) as f64 / (gpu_us as u64) as f64
        } else {
            0.0
        };
        let _ = writeln!(
            out,
            "  {label:<21} | {cpu_us:>14} | {gpu_us:>13} | {ratio:>5.2}x"
        );
    }

    print!("{out}");
    // cargo test runs with cwd = the crate dir (not the workspace
    // root), so a bare `reports/` would land at the crate's own
    // reports/, not the canonical workspace-root reports/. Walk
    // up one level from `CARGO_MANIFEST_DIR` (= crate dir)
    // → workspace `crates/` → workspace root. Robust against
    // future workspace layout changes only if the workspace
    // root stays two levels above the crate, which is the
    // current and historical layout.
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let path = reports_dir.join("r9_b_d64_timing.txt");
    let _ = std::fs::write(&path, &out);
}
