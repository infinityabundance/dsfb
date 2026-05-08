// DSFB-Debug: hand-rolled microbenchmark for fusion pipeline
// (Phase η.6, Session 18).
//
// No `criterion` Cargo dependency (preserves zero-runtime-deps
// discipline). Uses `std::time::Instant` for wall-clock measurement.
//
// Benchmarks:
//   - Per-fixture end-to-end run_fusion_evaluation latency (smallest
//     and largest fixtures: F-11b 4 windows × 6 signals = 24 cells;
//     F-11 431 windows × 16 signals = 6,896 cells).
//   - Derived: ns/cell (per-(window, signal) latency) for L-2
//     fusion + L-3 typed-episode emission.
//
// Per academic-honesty discipline: runs N iterations, reports
// median + min + max + ns/cell. Numbers are wall-clock at the
// timestamp of execution; reproducible only relative to that
// machine and that build flag set. Documented honestly.
//
// Theorem 9 preservation: the bench iterations all pass through
// `run_fusion_evaluation` which fires `verify_deterministic_replay`;
// every iteration is asserted to maintain replay.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};
use dsfb_debug::DsfbDebugEngine;

const F11B_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11b.tsv");
const F11_BYTES:  &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");

const ITERATIONS: u32 = 10;

fn is_sentinel(bytes: &[u8]) -> bool {
    bytes.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
}

#[derive(Debug, Clone)]
struct BenchResult {
    fixture: &'static str,
    num_signals: usize,
    num_windows: usize,
    cells: usize,
    median_us: u128,
    min_us: u128,
    max_us: u128,
    ns_per_cell: f64,
}

fn bench_fixture(fixture_name: &'static str, bytes: &[u8]) -> Option<BenchResult> {
    if is_sentinel(bytes) {
        eprintln!("[skip] {fixture_name} — sentinel");
        return None;
    }
    let matrix = parse_residual_projection(bytes).ok()?;
    if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
        return None;
    }
    let cells = matrix.num_signals * matrix.num_windows;
    let cfg = FusionConfig::ALL_DEFAULT;
    let engine = DsfbDebugEngine::<32, 64>::paper_lock().expect("paper-lock");

    // Warm-up iteration (not measured) — caches, branch predictor.
    let _ = run_fusion_evaluation(
        &engine, &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, &cfg, fixture_name,
    ).expect("warm-up should succeed");

    let mut samples_us: Vec<u128> = Vec::with_capacity(ITERATIONS as usize);
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let r = run_fusion_evaluation(
            &engine, &matrix.data, matrix.num_signals, matrix.num_windows,
            matrix.healthy_window_end, &matrix.fault_labels, &cfg, fixture_name,
        ).expect("benchmark iteration should succeed");
        let elapsed = t0.elapsed().as_micros();
        samples_us.push(elapsed);
        // Theorem 9: replay must hold for every benchmark iteration.
        assert!(r.deterministic_replay_holds,
            "Theorem 9 must hold under benchmark iteration");
    }
    samples_us.sort();
    let median = samples_us[ITERATIONS as usize / 2];
    let min = samples_us[0];
    let max = samples_us[ITERATIONS as usize - 1];
    let ns_per_cell = (median as f64 * 1000.0) / cells as f64;
    Some(BenchResult {
        fixture: fixture_name,
        num_signals: matrix.num_signals,
        num_windows: matrix.num_windows,
        cells,
        median_us: median,
        min_us: min,
        max_us: max,
        ns_per_cell,
    })
}

fn write_audit_markdown(filename: &str, content: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("docs");
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("[warn] mkdir docs: {e:?}"); return;
    }
    path.push(filename);
    match fs::File::create(&path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(content.as_bytes()) {
                eprintln!("[warn] write {filename}: {e:?}");
            } else {
                eprintln!("[bench] wrote {}", path.display());
            }
        }
        Err(e) => eprintln!("[warn] open {filename}: {e:?}"),
    }
}

#[test]
fn fusion_timing_bench() {
    println!();
    println!("=== Phase η.6 TIMING BENCHMARK (hand-rolled, {ITERATIONS} iter, std::time::Instant) ===");

    let r1 = bench_fixture("tadbench_trainticket_F11b", F11B_BYTES);
    let r2 = bench_fixture("tadbench_trainticket_F11",  F11_BYTES);

    let mut out = String::new();
    out.push_str("# Fusion timing benchmarks — Phase η.6\n\n");
    out.push_str("Hand-rolled microbenchmark using `std::time::Instant`. No\n");
    out.push_str("`criterion` Cargo dependency (zero-runtime-deps discipline\n");
    out.push_str("preserved). Per fixture: 1 warm-up iteration (not measured)\n");
    out.push_str(&format!("+ {ITERATIONS} measured iterations; reports median + min + max\n"));
    out.push_str("wall-clock µs and derived ns-per-cell latency.\n\n");
    out.push_str("Source: Phase η.6 bench harness (`tests/timing_benchmarks.rs`).\n");
    out.push_str("Theorem 9 deterministic replay verified per iteration.\n\n");
    out.push_str("Build flags: `cargo test --features \"std paper-lock\"` (debug build;\n");
    out.push_str("release build is typically 5-20× faster but not the documented\n");
    out.push_str("baseline because cargo-test defaults to debug).\n\n");

    out.push_str("## Per-fixture timing\n\n");
    out.push_str("| Fixture | Signals | Windows | Cells | Median µs | Min µs | Max µs | ns/cell |\n");
    out.push_str("|---------|--------:|--------:|------:|----------:|-------:|-------:|--------:|\n");
    for r_opt in [&r1, &r2] {
        if let Some(r) = r_opt {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} | {} | {:.1} |\n",
                r.fixture, r.num_signals, r.num_windows, r.cells,
                r.median_us, r.min_us, r.max_us, r.ns_per_cell));
            println!("[bench] {} ({} cells): median {} µs, ns/cell {:.1}",
                r.fixture, r.cells, r.median_us, r.ns_per_cell);
        }
    }

    out.push_str("\n## Honest empirical reading\n\n");
    out.push_str("The end-to-end `run_fusion_evaluation` per-cell latency on\n");
    out.push_str("the F-11 fixture (6,896 cells, 205 detectors, 9-axis fusion)\n");
    out.push_str("is the published debug-build benchmark. Production-deployment\n");
    out.push_str("sizing should:\n\n");
    out.push_str("1. Run `cargo test --release --features \"std paper-lock\"` for\n");
    out.push_str("   the realistic latency (debug builds run 5-20× slower).\n");
    out.push_str("2. Multiply ns/cell × num_signals × evaluation_rate to derive\n");
    out.push_str("   required CPU budget per pipeline.\n");
    out.push_str("3. The 9-axis bank-aware fusion and Theorem 9 deterministic-\n");
    out.push_str("   replay verification both contribute to the per-cell cost\n");
    out.push_str("   reported here; they are non-optional in `paper-lock` mode.\n\n");
    out.push_str("Per Session-17 academic-honesty discipline, no production-\n");
    out.push_str("latency claim is made — the numbers in this table are\n");
    out.push_str("verbatim test stdout on the run-time machine (cargo-test\n");
    out.push_str("debug build); a partner deployment should re-run the bench\n");
    out.push_str("on their hardware and capture site-specific numbers.\n");

    write_audit_markdown("benchmarks.md", &out);
}
