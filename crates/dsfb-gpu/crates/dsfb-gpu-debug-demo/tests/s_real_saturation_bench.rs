//! S-REAL saturation bench — real-data sibling of
//! `r9_c_d64_stage_profile_256x4096_k1`.
//!
//! WHY THIS TEST EXISTS (for the future engineer reading cold):
//!
//! The S-REAL audit binary (`dsfb-gpu-debug s-real-1-audit`) runs
//! a full provenance + casefile + replay-verification pipeline per
//! dataset. That pipeline has constant per-call overhead — TSV
//! parse, host compute_features, casefile JSON emit, audit_report
//! HTML render, replay-verification dispatch — that prevents the
//! dispatcher from reaching saturation regime even on 1M-event
//! fixtures. The throughput script `s_real_throughput_bench.sh`
//! measures that pipeline honestly at ~10 MB/s TSV throughput.
//!
//! This bench is the OTHER side of the comparison: the same
//! BatchedGpuWorkspace + CUDA Graphs + pinned-async + tight
//! measurement loop the S-PERF.16.a bench (`r9_c_d64_stage_profile_256x4096_k1`)
//! uses on a synthetic 256×4096 fixture, but pointed at a REAL
//! residual-projection-v2 TSV from `data/fixtures/`. Result: real
//! events flowing through the saturation harness, surfacing real
//! GB/s (wide bytes/sec) on the same dispatcher path that hit
//! 22.74 GB/s median in S-PERF.16.a.
//!
//! Env-var control (because #[test] can't take CLI args):
//!   DSFB_REAL_BENCH_TSV  : path to the residual-projection-v2 TSV
//!                         to bench. Default:
//!                         data/fixtures/radioml_2018_snr30_1024x1024.tsv
//!                         (the 1M-cell RadioML fixture; matches
//!                         the S-PERF.16.a magnitude).
//!   DSFB_REAL_BENCH_ITERS: measurement iterations (default 3,
//!                         matching r9_c_d64). 1 warmup runs in
//!                         addition.
//!
//! Output:
//!   reports/s_real_saturation_<basename>.txt with the full
//!   per-stage timing breakdown and wide bytes/sec.
//!
//! Honest framing (panel-locked, MUST hold in receipt):
//!   - This bench is a SATURATION HARNESS, not the audit binary.
//!     It skips replay-verification + casefile emit + audit_report
//!     render. It reports wall ONLY for the dispatcher path. The
//!     numbers are directly comparable to S-PERF.16.a; they are
//!     NOT directly comparable to the s_real_throughput_bench.sh
//!     numbers (which measure the audit-binary path).
//!   - Wide bytes/sec uses the panel-locked 264-byte
//!     `DetectorCellWide` accounting (same as S-PERF.16.a). It is
//!     a LOGICAL throughput on the dispatcher's internal arena,
//!     NOT the physical DRAM bandwidth (for DRAM%, see
//!     `scripts/s_real_perf_per_dataset.sh --ncu <id>`).
//!   - Cross-driver / cross-CUDA / cross-hardware throughput
//!     identity is NOT claimed.
//!
//! Run:
//!   cargo test --release --features cuda --test s_real_saturation_bench -- --nocapture
//!   DSFB_REAL_BENCH_TSV=data/fixtures/cmapss_fd001_unit1.tsv \
//!       cargo test --release --features cuda --test s_real_saturation_bench -- --nocapture

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::redundant_closure_for_method_calls,
    clippy::uninlined_format_args
)]

use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::hash::sha256;
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed,
    D64ThroughputStageTimings, GpuWorkspace,
};
use dsfb_gpu_debug_demo::cli::ingest::{
    load_residual_projection_tsv, lower_to_trace_events, sha256_to_hex_lower, LoweringConfig,
};

#[test]
fn s_real_saturation_bench() {
    use core::fmt::Write;

    // ───────────────────────────────────────────────────────────
    // Phase (a) — environment-variable resolution.
    // WHY: `#[test]` cannot take CLI args, so the bench harness is
    // parameterised entirely through env vars. The defaults match
    // the panel-locked S-PERF.16.a magnitude (1 M-cell RadioML
    // RF I/Q + 3 measured iters) so a no-args run reproduces the
    // canonical saturation-class number without any operator input.
    // ───────────────────────────────────────────────────────────
    // Resolve TSV path. Env var wins; default fixture is the 1M-
    // cell RadioML projection (matches S-PERF.16.a magnitude).
    let tsv_rel = std::env::var("DSFB_REAL_BENCH_TSV")
        .unwrap_or_else(|_| "data/fixtures/radioml_2018_snr30_1024x1024.tsv".to_string());
    let iters: usize = std::env::var("DSFB_REAL_BENCH_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let tsv_path = workspace_root.join(&tsv_rel);

    assert!(
        tsv_path.is_file(),
        "TSV not found at {} (set DSFB_REAL_BENCH_TSV to override)",
        tsv_path.display()
    );

    // ───────────────────────────────────────────────────────────
    // Phase (b) — TSV load + SHA-pin verification.
    // WHY: the audit binary's `s-real-audit` driver enforces
    // byte-identity against `data/fixtures/MANIFEST.toml` by
    // construction (refuses to admit a TSV whose SHA-256 mismatches
    // the pinned constant). The saturation bench is operator-
    // pointable at arbitrary TSV paths, so it computes the SHA on
    // the fly and emits it into the receipt so the operator can
    // cross-check against MANIFEST.toml if a pin exists.
    // ───────────────────────────────────────────────────────────
    // Load TSV → ResidualProjectionFixture → Vec<TraceEvent>.
    let tsv_bytes = std::fs::read(&tsv_path).expect("read TSV");
    let actual_sha = sha256_to_hex_lower(&sha256(&tsv_bytes));
    let fixture = load_residual_projection_tsv(&tsv_bytes, &actual_sha)
        .expect("residual-projection-v2 parse");
    // ───────────────────────────────────────────────────────────
    // Phase (c) — lower fixture to deterministic `TraceEvent[]`.
    // WHY: the dispatcher consumes events, not raw fixture rows.
    // The lowering is deterministic per `LoweringConfig::default()`
    // (no fixture-specific overrides) so the same TSV always
    // produces the same event stream. The saturation bench cares
    // about throughput on this exact event stream, not about
    // discovering events.
    // ───────────────────────────────────────────────────────────
    let cfg = LoweringConfig::default();
    let events = lower_to_trace_events(&fixture, &cfg);
    let n_entities = fixture.declared_num_signals;
    let n_windows = fixture.declared_num_windows;
    let event_count = events.len();
    let fixture_bytes = tsv_bytes.len();

    // ───────────────────────────────────────────────────────────
    // Phase (d) — build S-PERF.16.a-shaped contract.
    // WHY: pinning the same D64 registry hash + bank hash that
    // S-PERF.16.a pins forces the dispatcher to take the exact
    // same code path (no detector-profile divergence, no bank-
    // surface divergence). Only the events differ between the
    // synthetic 256×4096 anchor and the real-data fixture, so
    // any wall-time delta is attributable to data shape, not to
    // a different dispatcher specialisation.
    // ───────────────────────────────────────────────────────────
    let mut contract = Contract::scaled(n_entities, n_windows);
    contract.pin_bank_hash(bank_hash());
    contract.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());

    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture_hashes = FixtureHashes::compute(&events, &features);

    // ───────────────────────────────────────────────────────────
    // Phase (e) — GPU workspace allocation + graph capture.
    // WHY: `new_with_pinned_async` allocates the pinned host
    // staging buffers + device-side BatchedGpuWorkspace + captures
    // the CUDA Graph on first dispatch. Doing this ONCE (before
    // the measurement loop) means every measured dispatch hits the
    // amortised steady-state path — exactly the regime S-PERF.16.a
    // measures. Per-call allocation + capture overhead would
    // dominate the wall on small fixtures otherwise.
    // ───────────────────────────────────────────────────────────
    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();

    // ───────────────────────────────────────────────────────────
    // Phase (f) — measurement loop + per-iter timing.
    // WHY: the warmup iter is excluded from the median because the
    // first dispatch pays cold-cache cost (PTX JIT, first-call
    // driver work, lazy buffer first-touch) that the rest don't.
    // Capturing `iters` more dispatches and taking the median
    // washes out single-run thermal jitter; cudaEvent timings are
    // tighter than Instant::now() walls but both are recorded.
    // ───────────────────────────────────────────────────────────
    // Warmup. Same role as r9_c_d64_stage_profile_256x4096_k1's
    // warmup: amortise lazy buffer allocation, JIT PTX, first-
    // call CUDA driver work. Excluded from the measurement
    // median by construction.
    let _ = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
        &events,
        &contract,
        &mut ws,
        &fixture_hashes,
    )
    .unwrap();

    // Measurement loop. One dispatch per iter, tight wall +
    // cudaEvent stage timings captured. Same shape as r9_c_d64.
    let mut device_samples: Vec<D64ThroughputStageTimings> = Vec::with_capacity(iters);
    let mut wall_us_samples: Vec<u128> = Vec::with_capacity(iters);
    let mut last_episodes: usize = 0;

    for _ in 0..iters {
        let t0 = Instant::now();
        let (case, dev, _host) =
            build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
                &events,
                &contract,
                &mut ws,
                &fixture_hashes,
            )
            .unwrap();
        let wall = t0.elapsed().as_micros();
        last_episodes = case.episodes.len();
        device_samples.push(dev);
        wall_us_samples.push(wall);
        std::hint::black_box(case);
    }

    // ───────────────────────────────────────────────────────────
    // Phase (g) — median + min + max computation.
    // WHY: `f32::total_cmp` produces a total order over NaN/inf
    // edge cases that the natural `<` would leave undefined. The
    // median is robust to a single outlier iteration; with 3 iters
    // the middle element IS the median by construction. Matches
    // `r9_c_d64_stage_profile_256x4096_k1`'s aggregation discipline
    // exactly so the two receipts are directly comparable.
    // ───────────────────────────────────────────────────────────
    // Median across iters. f32::total_cmp matches r9_c_d64's
    // approach; pick the middle element of the sorted vector.
    fn median_f32<F: Fn(&D64ThroughputStageTimings) -> f32>(
        samples: &[D64ThroughputStageTimings],
        f: F,
    ) -> f32 {
        let mut v: Vec<f32> = samples.iter().map(f).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    }
    let dev = D64ThroughputStageTimings {
        h2d_us: median_f32(&device_samples, |t| t.h2d_us),
        residual_us: median_f32(&device_samples, |t| t.residual_us),
        sign_us: median_f32(&device_samples, |t| t.sign_us),
        detector_wide_us: median_f32(&device_samples, |t| t.detector_wide_us),
        consensus_wide_us: median_f32(&device_samples, |t| t.consensus_wide_us),
        axis5_grid_sum_us: median_f32(&device_samples, |t| t.axis5_grid_sum_us),
        candidate_wide_us: median_f32(&device_samples, |t| t.candidate_wide_us),
        residual_digest_us: median_f32(&device_samples, |t| t.residual_digest_us),
        sign_digest_us: median_f32(&device_samples, |t| t.sign_digest_us),
        detector_digest_us: median_f32(&device_samples, |t| t.detector_digest_us),
        consensus_digest_us: median_f32(&device_samples, |t| t.consensus_digest_us),
        d2h_us: median_f32(&device_samples, |t| t.d2h_us),
        total_device_us: median_f32(&device_samples, |t| t.total_device_us),
    };
    wall_us_samples.sort_unstable();
    let wall_med_us = wall_us_samples[wall_us_samples.len() / 2];

    // ───────────────────────────────────────────────────────────
    // Phase (h) — wide-bytes/sec arithmetic + classification.
    // WHY: "wide GB/s" is panel-locked LOGICAL throughput on the
    // 264-byte `DetectorCellWide` arena — same accounting as
    // S-PERF.16.a. It is NOT physical DRAM bandwidth (for DRAM%
    // see `scripts/s_real_perf_per_dataset.sh --ncu <id>`). The
    // saturation-class / transition / launch-bound classification
    // (≥ 50 %, 5–50 %, < 5 % of S-PERF.16.a's 22.74 GB/s synthetic
    // anchor) is property of cell-count + dispatcher-shape, NOT a
    // detector-quality or domain-truth claim.
    // ───────────────────────────────────────────────────────────
    // Derived throughput. Same formulas as r9_c_d64:
    //   n_cells = n_entities × n_windows
    //   wide_bytes/sec = (n_cells × 264 bytes) / total_device_us
    //   cells/sec, detector_evals/sec (D64 = 64 motifs/cell)
    let n_cells: u64 = u64::from(n_entities) * u64::from(n_windows);
    let total_device_us_f = f64::from(dev.total_device_us).max(1.0);
    let cells_per_sec = n_cells as f64 * 1_000_000.0 / total_device_us_f;
    let det_evals_per_sec = cells_per_sec * 64.0;
    let wide_bytes_per_sec = (n_cells as f64) * 264.0 * 1_000_000.0 / total_device_us_f;
    let wide_gb_per_sec = wide_bytes_per_sec / 1e9;
    let tsv_gb_per_sec = (fixture_bytes as f64) * 1_000_000.0 / total_device_us_f / 1e9;

    // ───────────────────────────────────────────────────────────
    // Phase (i) — receipt emission.
    // WHY: the receipt at `reports/s_real_saturation_<basename>.txt`
    // is the per-fixture audit artifact the sweep script
    // (`scripts/s_real_saturation_sweep.sh`) aggregates into
    // `reports/s_real_saturation_sweep.txt`. The receipt MUST be
    // self-describing — operator should be able to read one
    // receipt cold and understand the saturation-class verdict
    // without re-running the bench. Layout mirrors r9_c_d64's
    // S-PERF receipt so a reader who knows that format can read
    // either one cold.
    // ───────────────────────────────────────────────────────────
    // Build the human-readable report. Mirror the r9_c_d64
    // layout so a reader who knows the S-PERF format can read
    // this one cold.
    let mut out = String::with_capacity(4096);
    let _ = writeln!(
        out,
        "=== S-REAL saturation bench ({}, n_entities={}, n_windows={}, iters={}) ===",
        tsv_rel, n_entities, n_windows, iters
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Source TSV:");
    let _ = writeln!(out, "  path                 : {}", tsv_path.display());
    let _ = writeln!(out, "  byte_size            : {} bytes", fixture_bytes);
    let _ = writeln!(out, "  sha256               : {}", actual_sha);
    let _ = writeln!(out, "  n_entities           : {}", n_entities);
    let _ = writeln!(out, "  n_windows            : {}", n_windows);
    let _ = writeln!(
        out,
        "  healthy_window_end   : {}",
        fixture.declared_healthy_window_end
    );
    let _ = writeln!(out, "  events_lowered       : {}", event_count);
    let _ = writeln!(out, "  n_cells (E × W)      : {}", n_cells);
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Per-stage device timings (median over {iters} iters, microseconds):"
    );
    let _ = writeln!(
        out,
        "  stage                              | us       | % of device"
    );
    let _ = writeln!(
        out,
        "  -----------------------------------+----------+------------"
    );
    let pct = |us: f32| {
        if dev.total_device_us > 0.0 {
            (us / dev.total_device_us) * 100.0
        } else {
            0.0
        }
    };
    let _ = writeln!(
        out,
        "  h2d                                | {:>8} | {:>5.1}",
        dev.h2d_us as u64,
        pct(dev.h2d_us)
    );
    let _ = writeln!(
        out,
        "  residual_field                     | {:>8} | {:>5.1}",
        dev.residual_us as u64,
        pct(dev.residual_us)
    );
    let _ = writeln!(
        out,
        "  drift_slew_sign                    | {:>8} | {:>5.1}",
        dev.sign_us as u64,
        pct(dev.sign_us)
    );
    let _ = writeln!(
        out,
        "  detector_motif (D64 wide)          | {:>8} | {:>5.1}",
        dev.detector_wide_us as u64,
        pct(dev.detector_wide_us)
    );
    let _ = writeln!(
        out,
        "  consensus_grid (wide)              | {:>8} | {:>5.1}",
        dev.consensus_wide_us as u64,
        pct(dev.consensus_wide_us)
    );
    let _ = writeln!(
        out,
        "  axis5_grid_sum                     | {:>8} | {:>5.1}",
        dev.axis5_grid_sum_us as u64,
        pct(dev.axis5_grid_sum_us)
    );
    let _ = writeln!(
        out,
        "  candidate_collapse                 | {:>8} | {:>5.1}",
        dev.candidate_wide_us as u64,
        pct(dev.candidate_wide_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest residual               | {:>8} | {:>5.1}",
        dev.residual_digest_us as u64,
        pct(dev.residual_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest sign                   | {:>8} | {:>5.1}",
        dev.sign_digest_us as u64,
        pct(dev.sign_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest detector (wide cells)  | {:>8} | {:>5.1}",
        dev.detector_digest_us as u64,
        pct(dev.detector_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest consensus              | {:>8} | {:>5.1}",
        dev.consensus_digest_us as u64,
        pct(dev.consensus_digest_us)
    );
    let _ = writeln!(
        out,
        "  d2h                                | {:>8} | {:>5.1}",
        dev.d2h_us as u64,
        pct(dev.d2h_us)
    );
    let _ = writeln!(
        out,
        "  total_device_us                    | {:>8} | 100.0",
        dev.total_device_us as u64
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Host wall (host Instant, includes H2D copy + kernels + D2H):"
    );
    let _ = writeln!(out, "  wall_median_us       : {}", wall_med_us);
    let _ = writeln!(out);
    let _ = writeln!(out, "Counters and derived throughput:");
    let _ = writeln!(out, "  episode_count        : {}", last_episodes);
    let _ = writeln!(out, "  cells/sec            : {:.2e}", cells_per_sec);
    let _ = writeln!(out, "  detector_evals/sec   : {:.2e}", det_evals_per_sec);
    let _ = writeln!(out, "  wide bytes/sec (264) : {:.2} GB/s", wide_gb_per_sec);
    let _ = writeln!(
        out,
        "  TSV bytes/sec        : {:.2} GB/s  (TSV input bytes / total_device_us)",
        tsv_gb_per_sec
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Comparison anchor:");
    let _ = writeln!(
        out,
        "  S-PERF.16.a saturation median (256x4096 K=1, synthetic): 22.74 GB/s"
    );
    let _ = writeln!(
        out,
        "  this bench (real data, same dispatcher path)            : {:.2} GB/s",
        wide_gb_per_sec
    );
    let _ = writeln!(
        out,
        "  ratio (this / S-PERF.16.a)                              : {:.3}",
        wide_gb_per_sec / 22.74
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Honest framing (panel-locked):");
    let _ = writeln!(
        out,
        "  - This bench mirrors r9_c_d64_stage_profile_256x4096_k1's"
    );
    let _ = writeln!(
        out,
        "    measurement protocol (warmup + tight loop + median)."
    );
    let _ = writeln!(
        out,
        "    The numbers above are directly comparable to S-PERF.16.a's."
    );
    let _ = writeln!(
        out,
        "  - The TSV-bytes/sec column is for SCALE reference (much"
    );
    let _ = writeln!(
        out,
        "    smaller than wide-bytes/sec because the TSV is a text"
    );
    let _ = writeln!(
        out,
        "    representation of cell residuals at 6-decimal places)."
    );
    let _ = writeln!(out, "  - wide bytes/sec uses the panel-locked 264-byte");
    let _ = writeln!(
        out,
        "    DetectorCellWide accounting; it is logical throughput on"
    );
    let _ = writeln!(
        out,
        "    the dispatcher's internal arena, NOT physical DRAM bandwidth."
    );
    let _ = writeln!(
        out,
        "  - cross-driver / cross-CUDA / cross-hardware identity is"
    );
    let _ = writeln!(out, "    NOT claimed.");

    // Print to stdout (so `--nocapture` shows it) and write to
    // reports/. Filename derived from the TSV basename so multiple
    // datasets produce side-by-side receipts.
    print!("{out}");
    let basename = tsv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let report_path = reports_dir.join(format!("s_real_saturation_{basename}.txt"));
    let _ = std::fs::write(&report_path, &out);

    // Sanity: dispatcher must have produced a non-zero device wall
    // and the dispatcher must have processed events. A zero wall
    // means the timer never fired (broken bench), not a 0 GB/s
    // performance claim.
    assert!(
        dev.total_device_us > 0.0,
        "total_device_us was 0 — bench did not measure the dispatcher correctly"
    );
    assert!(
        event_count > 0,
        "lowering produced 0 events; the TSV at {} has no finite cells",
        tsv_path.display()
    );
}
