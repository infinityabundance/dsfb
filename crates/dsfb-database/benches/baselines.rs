//! Pass-2 M5: Criterion microbenchmark for the published baseline
//! change-point detectors (ADWIN, BOCPD, PELT) used by the §7
//! bake-off. Compares throughput on a synthetic series with one
//! mid-stream mean shift — the canonical change-detection input.
//!
//! Read-only: never modifies `src/baselines/`. The detectors are
//! constructed via `Default::default()`, identical to the
//! `replay_tape_baselines` binary's instantiation.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dsfb_database::baselines::{adwin::Adwin, bocpd::Bocpd, pelt::Pelt, ChangePointDetector};

fn build_series(n: usize, seed: u64) -> Vec<(f64, f64)> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut series = Vec::with_capacity(n);
    for i in 0..n {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let raw = ((state >> 32) as i32 as f64) / (i32::MAX as f64);
        // Mean-shift halfway through; +1.5 is well above unit-Gaussian
        // noise so every detector should fire at least one CP.
        let bias = if i > n / 2 { 1.5 } else { 0.0 };
        series.push((i as f64 * 0.5, raw + bias));
    }
    series
}

fn detectors() -> Vec<(&'static str, Box<dyn ChangePointDetector>)> {
    vec![
        ("adwin", Box::new(Adwin::default())),
        ("bocpd", Box::new(Bocpd::default())),
        ("pelt", Box::new(Pelt::default())),
    ]
}

fn bench_baselines(c: &mut Criterion) {
    let mut group = c.benchmark_group("baselines_detect");
    for &n in &[256_usize, 1024, 4096] {
        let series = build_series(n, 42);
        group.throughput(Throughput::Elements(n as u64));
        for (name, det) in detectors() {
            group.bench_with_input(BenchmarkId::new(name, n), &n, |b, _| {
                b.iter(|| {
                    black_box(det.detect(black_box(&series)));
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_baselines);
criterion_main!(benches);
