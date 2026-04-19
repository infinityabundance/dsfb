//! Pass-2 M5: Criterion microbenchmark for the live PostgreSQL
//! distiller (`live::distiller::DistillerState::ingest`). Measures
//! per-snapshot ingest latency on a synthetic snapshot sized at the
//! live-adapter's typical pg_stat_statements / pg_stat_io row count.
//!
//! Read-only: never modifies `src/live/distiller.rs`. Snapshots are
//! constructed using the publicly-exported row types only.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dsfb_database::live::distiller::{
    ActivityRow, DistillerState, PgssRow, Snapshot, StatDatabaseRow, StatIoRow,
};

fn build_snapshot(t: f64, n_pgss: usize, n_io: usize) -> Snapshot {
    let pgss = (0..n_pgss)
        .map(|i| PgssRow {
            query_id: format!("q{:04x}", i),
            // Counters grow linearly in t to simulate steady traffic;
            // the distiller computes a per-channel delta, so a true
            // residual signal needs a non-degenerate counter shape.
            calls: (i as u64 + 1).wrapping_mul(t as u64).max(1),
            total_exec_time_ms: (i as f64 + 1.0) * t,
        })
        .collect();
    let activity = (0..4)
        .map(|i| ActivityRow {
            wait_event_type: "Lock".to_string(),
            wait_event: format!("Lwait{}", i),
            state: "active".to_string(),
        })
        .collect();
    let stat_io = (0..n_io)
        .map(|i| StatIoRow {
            backend_type: "client backend".to_string(),
            object: format!("relation{}", i % 4),
            context: "normal".to_string(),
            reads: (i as u64 + 1).wrapping_mul(t as u64).max(1),
            hits: (i as u64 + 4).wrapping_mul(t as u64).max(1),
            read_time_ms: (i as f64 + 1.0) * t * 0.1,
        })
        .collect();
    let stat_database = vec![StatDatabaseRow {
        datname: "bench".to_string(),
        blks_hit: 1_000 + (t as u64) * 100,
        blks_read: 50 + (t as u64) * 5,
    }];
    Snapshot {
        t,
        pgss,
        activity,
        stat_io,
        stat_database,
    }
}

fn bench_distiller(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_distiller_ingest");
    for &(n_pgss, n_io) in &[(8usize, 4usize), (32, 8), (128, 16)] {
        // Pre-build 60 snapshots (≈ 30 s of pulsed scrape at 500 ms);
        // benchmark feeds them sequentially through one DistillerState
        // so the per-channel delta machinery is genuinely exercised.
        let snaps: Vec<Snapshot> = (0..60)
            .map(|i| build_snapshot((i + 1) as f64 * 0.5, n_pgss, n_io))
            .collect();
        group.throughput(Throughput::Elements(snaps.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("pgss={n_pgss}_io={n_io}"), snaps.len()),
            &snaps,
            |b, snaps| {
                b.iter(|| {
                    let mut state = DistillerState::new();
                    for s in snaps {
                        black_box(state.ingest(black_box(s)));
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_distiller);
criterion_main!(benches);
