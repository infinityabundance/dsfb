//! Pass-2 M5: Criterion microbenchmark for the motif engine.
//!
//! Measures `MotifEngine::run` throughput on a synthetic residual stream
//! sized at the live-tape order of magnitude (≈ 8000 samples ≈ 4000 s
//! of pulsed scrape at 500 ms cadence). The bench is deterministic
//! (fixed seed, fixed shape, no I/O) so cross-revision regressions can
//! be tracked over time.
//!
//! Read-only: never modifies `src/grammar/` or `src/residual/`. The
//! grammar is constructed via `MotifParams::default_for(...)` per
//! class, identical to the production engine path.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar, MotifParams};
use dsfb_database::residual::{ResidualClass, ResidualSample, ResidualStream};

fn build_stream(n: usize, seed: u64) -> ResidualStream {
    let mut s = ResidualStream::new("bench");
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let classes = [
        ResidualClass::PlanRegression,
        ResidualClass::Cardinality,
        ResidualClass::Contention,
        ResidualClass::CacheIo,
        ResidualClass::WorkloadPhase,
    ];
    let mut t = 0.0_f64;
    for i in 0..n {
        // Linear-congruential PRNG; same constants as
        // src/bin/baseline_tune.rs::Lcg. Bench-local, deterministic.
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let class = classes[(state as usize) % classes.len()];
        let raw = ((state >> 32) as i32 as f64) / (i32::MAX as f64);
        // Inject a small mean-shift halfway through so the engine does
        // some real work (otherwise an all-zero stream short-circuits).
        let bias = if i > n / 2 { 1.5 } else { 0.0 };
        let value = raw * 4.0 + bias;
        t += 0.5; // 500 ms cadence — matches live POLL_INTERVAL_MS.
        s.samples.push(ResidualSample {
            t,
            class,
            value,
            channel: Some(format!("ch{}", i % 8)),
        });
    }
    s.sort();
    s
}

fn grammar() -> MotifGrammar {
    MotifGrammar {
        plan_regression_onset: MotifParams::default_for(MotifClass::PlanRegressionOnset),
        cardinality_mismatch_regime: MotifParams::default_for(
            MotifClass::CardinalityMismatchRegime,
        ),
        contention_ramp: MotifParams::default_for(MotifClass::ContentionRamp),
        cache_collapse: MotifParams::default_for(MotifClass::CacheCollapse),
        workload_phase_transition: MotifParams::default_for(MotifClass::WorkloadPhaseTransition),
    }
}

fn bench_motif_engine(c: &mut Criterion) {
    for &n in &[256_usize, 1024, 8192] {
        let stream = build_stream(n, 42);
        let g = grammar();
        let mut group = c.benchmark_group("motif_engine_run");
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| {
                let engine = MotifEngine::new(g.clone());
                black_box(engine.run(black_box(&stream)));
            });
        });
        group.finish();
    }
}

criterion_group!(benches, bench_motif_engine);
criterion_main!(benches);
