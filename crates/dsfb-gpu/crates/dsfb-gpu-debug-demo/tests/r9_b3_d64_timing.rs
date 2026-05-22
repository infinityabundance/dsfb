//! R.9.b.3 measurement — full-pipeline wall-time comparison
//! between the D16 compact Throughput path (R.11) and the new
//! D64 wide compact Throughput path (R.9.b.3). Runs as a test so
//! the pre-commit gate keeps the measurement honest; emits a
//! report to `reports/r9_b3_d64_timing.txt`.

#![cfg(feature = "cuda")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, synthesize_scaled, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::{registry_hash, DetectorProfile};
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact,
    build_gpu_throughput_pinned_async_on_workspace_tree_compact, GpuWorkspace,
};

fn d16_contract(n_entities: u32, n_windows: u32) -> Contract {
    let mut c = if n_entities == 16 && n_windows == 128 {
        Contract::canonical()
    } else {
        Contract::scaled(n_entities, n_windows)
    };
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(registry_hash());
    c
}

fn d64_contract(n_entities: u32, n_windows: u32) -> Contract {
    let mut c = if n_entities == 16 && n_windows == 128 {
        Contract::canonical()
    } else {
        Contract::scaled(n_entities, n_windows)
    };
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

fn fixture_events(n_entities: u32, n_windows: u32) -> Vec<dsfb_gpu_debug_core::event::TraceEvent> {
    if n_entities == 16 && n_windows == 128 {
        synthesize(DEFAULT_SEED)
    } else {
        synthesize_scaled(DEFAULT_SEED, n_entities, n_windows, 4)
    }
}

fn d16_median_us(
    events: &[dsfb_gpu_debug_core::event::TraceEvent],
    contract: &Contract,
    fixture: &FixtureHashes,
    warmup: usize,
    iters: usize,
) -> u128 {
    let mut ws = GpuWorkspace::new_with_pinned_async(contract).unwrap();
    for _ in 0..warmup {
        let _ = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
            events, contract, &mut ws, fixture,
        );
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_gpu_throughput_pinned_async_on_workspace_tree_compact(
            events, contract, &mut ws, fixture,
        )
        .unwrap();
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us.sort_unstable();
    samples_us[samples_us.len() / 2]
}

fn d64_median_us(
    events: &[dsfb_gpu_debug_core::event::TraceEvent],
    contract: &Contract,
    fixture: &FixtureHashes,
    warmup: usize,
    iters: usize,
) -> u128 {
    let mut ws = GpuWorkspace::new_with_pinned_async(contract).unwrap();
    for _ in 0..warmup {
        let _ = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
            events, contract, &mut ws, fixture,
        );
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact(
            events, contract, &mut ws, fixture,
        )
        .unwrap();
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us.sort_unstable();
    samples_us[samples_us.len() / 2]
}

#[test]
fn r9_b3_d64_full_pipeline_timing() {
    use core::fmt::Write;
    // Three K=1 scale points matching the rest of the R.8+ bench
    // taxonomy. Tight iter counts so the test fits in the pre-commit
    // gate's budget (~10 seconds at scale-large).
    let points: [(&str, u32, u32, usize, usize); 3] = [
        ("canonical 16x128", 16, 128, 3, 10),
        ("mid-scale 64x512", 64, 512, 2, 5),
        ("full-scale 256x4096", 256, 4096, 1, 3),
    ];

    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== R.9.b.3 D64 full-pipeline timing (one-shot, K=1) ==="
    );
    let _ = writeln!(
        out,
        "Hardware: RTX 4080 SUPER, CUDA 13.2. Bench is per-iter wall median."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  scale                 | D16 compact (us) | D64 compact (us) | ratio D16/D64"
    );
    let _ = writeln!(
        out,
        "  --------------------- | ---------------- | ---------------- | -------------"
    );
    for &(label, n_entities, n_windows, warmup, iters) in &points {
        let events = fixture_events(n_entities, n_windows);

        let c16 = d16_contract(n_entities, n_windows);
        let f16 = {
            let features = compute_features(
                &events,
                c16.n_windows,
                c16.n_entities,
                u64::from(c16.window_size_ms) * 1_000_000,
            );
            FixtureHashes::compute(&events, &features)
        };
        let c64 = d64_contract(n_entities, n_windows);
        let f64 = {
            let features = compute_features(
                &events,
                c64.n_windows,
                c64.n_entities,
                u64::from(c64.window_size_ms) * 1_000_000,
            );
            FixtureHashes::compute(&events, &features)
        };

        let d16_us = d16_median_us(&events, &c16, &f16, warmup, iters);
        let d64_us = d64_median_us(&events, &c64, &f64, warmup, iters);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let ratio = if d64_us > 0 {
            (d16_us as u64) as f64 / (d64_us as u64) as f64
        } else {
            0.0
        };
        let _ = writeln!(
            out,
            "  {label:<21} | {d16_us:>16} | {d64_us:>16} | {ratio:>13.2}x"
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Notes:");
    let _ = writeln!(
        out,
        "  * D16 = R.11 compact path (canonical 16-motif court)."
    );
    let _ = writeln!(
        out,
        "  * D64 = R.9.b.3 wide path: 16 motifs x 4 variants, with on-device"
    );
    let _ = writeln!(
        out,
        "    OR projection to canonical 16-bit mask before consensus +"
    );
    let _ = writeln!(out, "    candidate. Bank ABI unchanged.");
    let _ = writeln!(
        out,
        "  * Wide cells (264 B each) stay on-device; only candidates +"
    );
    let _ = writeln!(out, "    counts + 4 x 32-byte stage digests cross PCIe.");
    let _ = writeln!(
        out,
        "  * The D64 case file commits to DetectorProfile::D64.registry_hash"
    );
    let _ = writeln!(
        out,
        "    in its `detector_registry` chain link; D16 commits to the"
    );
    let _ = writeln!(
        out,
        "    canonical motif registry hash. Receipts cannot be confused."
    );

    print!("{out}");

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let path = reports_dir.join("r9_b3_d64_timing.txt");
    let _ = std::fs::write(&path, &out);
}
