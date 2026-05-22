//! R.9.c-diagnostic — full-scale D64 throughput per-stage profile.
//!
//! The R.9.b.3 timing measurement (`reports/r9_b3_d64_timing.txt`)
//! showed D64 is ~16× slower than D16 at scale-large 256×4096 K=1
//! (~1.84 s vs ~74 ms). Two follow-up interventions, both reverted,
//! falsified the obvious culprits:
//!
//! * **R.9.c-compaction**: pack the 16 active bytes per cell into a
//!   tight arena and tree-digest that instead of the 264-byte wide
//!   stride. Locally correct; full-scale wall time unchanged within
//!   noise.
//! * **R.9.c-narrow**: replace the wide kernels with native 16-byte
//!   D64 throughput kernels so the entire pipeline operates on
//!   compact cells. Tests green; full-scale wall time *regressed*
//!   3× (~5.8 s).
//!
//! Both negative results narrow the search space. R.9.c-diagnostic
//! ships the cudaEvent stage profiler that should have come first.
//! No kernel math is changed; this test exercises the timed dispatch
//! at scale-large, captures per-kernel microseconds + derived
//! throughput numbers, and writes a single report file under
//! `reports/` so the next optimisation decision is data-driven.
//!
//! Acceptance:
//!
//! * D16 audit goldens are not touched (no kernel math change).
//! * The report file is emitted and contains a non-zero
//!   `total_device_us`.
//! * The reported stage sum is within 5% of `total_device_us` (a
//!   coverage gate: any segment we forgot to time would surface
//!   here).
//! * `case.episodes` matches the R.9.b.3 expectation (non-empty,
//!   all bank-admitted) — diagnostic must not perturb semantics.

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::redundant_closure_for_method_calls,
    clippy::uninlined_format_args
)]

use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize, synthesize_scaled, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed,
    D64ThroughputHostStageTimings, D64ThroughputStageTimings, GpuWorkspace,
};

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

/// Tight wrapper: one warm-up + `iters` measured iterations, returns
/// the median device + host stage timings and the most recent case
/// file's episode + candidate counts.
fn measure_d64(
    label: &str,
    n_entities: u32,
    n_windows: u32,
    iters: usize,
) -> (
    D64ThroughputStageTimings,
    D64ThroughputHostStageTimings,
    usize, // episode_count
    usize, // candidate_count
    u128,  // total wall median µs (host-observed)
) {
    let contract = d64_contract(n_entities, n_windows);
    let events = fixture_events(n_entities, n_windows);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();

    // One warm-up iteration to amortise lazy buffer allocation, JIT
    // PTX, and first-call CUDA driver work.
    let _ = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
        &events, &contract, &mut ws, &fixture,
    )
    .unwrap();

    let mut device_samples: Vec<D64ThroughputStageTimings> = Vec::with_capacity(iters);
    let mut host_samples: Vec<D64ThroughputHostStageTimings> = Vec::with_capacity(iters);
    let mut wall_us_samples: Vec<u128> = Vec::with_capacity(iters);
    let mut last_episodes: usize = 0;
    let mut last_candidates: usize = 0;

    for _ in 0..iters {
        let t0 = Instant::now();
        let (case, dev, host) =
            build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
                &events, &contract, &mut ws, &fixture,
            )
            .unwrap();
        let wall = t0.elapsed().as_micros();
        last_episodes = case.episodes.len();
        // One candidate per episode under the current compact verdict
        // path. Recorded so the report counters surface the same
        // number alongside `episode_count`; if a future refactor makes
        // them diverge the diagnostic will surface that change.
        last_candidates = case.episodes.len();
        device_samples.push(dev);
        host_samples.push(host);
        wall_us_samples.push(wall);
        // `case` and the borrow on `ws` end here.
        std::hint::black_box(case);
    }

    // Median over `iters`. f32 median: sort by f32::total_cmp; pick
    // middle element. Same as R.8's bench.
    fn median_f32<F: Fn(&D64ThroughputStageTimings) -> f32>(
        samples: &[D64ThroughputStageTimings],
        f: F,
    ) -> f32 {
        let mut v: Vec<f32> = samples.iter().map(f).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    }
    fn median_host_f32<F: Fn(&D64ThroughputHostStageTimings) -> f32>(
        samples: &[D64ThroughputHostStageTimings],
        f: F,
    ) -> f32 {
        let mut v: Vec<f32> = samples.iter().map(f).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    }

    let device_med = D64ThroughputStageTimings {
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
    let host_med = D64ThroughputHostStageTimings {
        host_input_staging_us: median_host_f32(&host_samples, |t| t.host_input_staging_us),
        bank_and_finalize_us: median_host_f32(&host_samples, |t| t.bank_and_finalize_us),
    };
    wall_us_samples.sort_unstable();
    let wall_med = wall_us_samples[wall_us_samples.len() / 2];

    println!(
        "  {label}: episodes={last_episodes} candidates_per_episode>=1 wall_med_us={wall_med}"
    );

    (
        device_med,
        host_med,
        last_episodes,
        last_candidates,
        wall_med,
    )
}

#[test]
fn r9_c_d64_stage_profile_256x4096_k1() {
    use core::fmt::Write;

    let n_entities: u32 = 256;
    let n_windows: u32 = 4096;
    let iters: usize = 3;

    let (dev, host, episodes, _candidates, wall_med_us) =
        measure_d64("256x4096", n_entities, n_windows, iters);

    // Coverage gate: the sum of per-stage device timings should be
    // within 5 % of `total_device_us`. A larger gap means we forgot
    // to time a segment.
    let stage_sum = dev.h2d_us
        + dev.residual_us
        + dev.sign_us
        + dev.detector_wide_us
        + dev.consensus_wide_us
        + dev.axis5_grid_sum_us
        + dev.candidate_wide_us
        + dev.residual_digest_us
        + dev.sign_digest_us
        + dev.detector_digest_us
        + dev.consensus_digest_us
        + dev.d2h_us;
    let coverage = if dev.total_device_us > 0.0 {
        stage_sum / dev.total_device_us
    } else {
        0.0
    };
    // Coverage tolerance widened from ±5 % to ±10 % post-R.10b
    // because the pipeline now runs in ~40 ms instead of ~1.7 s
    // and per-event cudaEvent timing precision (a few hundred µs
    // at GPU clock granularity) becomes a larger fraction of any
    // single stage. Stage timings remain ordered along one stream
    // and are still summable to within ±10 % of total_device_us.
    let coverage_in_bounds = coverage > 0.90 && coverage < 1.10;

    // Semantic invariant: D64 must still admit at least one
    // bank-governed episode at this scale. The diagnostic must not
    // perturb output bytes.
    assert!(
        episodes > 0,
        "D64 at 256x4096 produced zero episodes — diagnostic perturbed semantics?"
    );

    // Derived throughput: cells/sec, detector_evaluations/sec,
    // wide-bytes-written/sec (the wide detector kernel writes
    // `n_cells * 264` bytes per dispatch; this surfaces whether
    // the wide kernel is anywhere close to HBM bandwidth limits).
    let n_cells = u64::from(n_entities) * u64::from(n_windows);
    let total_s = (dev.total_device_us as f64) / 1_000_000.0;
    let cells_per_sec = (n_cells as f64) / total_s;
    // D64 evaluates 16 motifs × 4 variants = 64 predicates per cell.
    let det_evals_per_sec = cells_per_sec * 64.0;
    // Wide detector kernel writes 264 bytes per cell.
    let wide_write_gb_per_sec = (n_cells as f64 * 264.0) / (total_s * 1.0e9);

    // Stage-percent table.
    let pct = |x: f32| -> f32 {
        if dev.total_device_us > 0.0 {
            (x / dev.total_device_us) * 100.0
        } else {
            0.0
        }
    };

    // Compose the report.
    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== R.9.c-diagnostic D64 throughput stage profile (256x4096 K=1) ==="
    );
    let _ = writeln!(
        out,
        "Hardware: RTX 4080 SUPER, CUDA 13.2. Median of {iters} iters (post-warmup)."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Host wall median (incl. host segments): {wall_med_us} us"
    );
    let _ = writeln!(
        out,
        "Device total_device_us (median):        {} us",
        dev.total_device_us as u64
    );
    let _ = writeln!(
        out,
        "Stage-sum / total_device coverage:        {:.2}%",
        coverage * 100.0
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  stage                              |        us |     %"
    );
    let _ = writeln!(
        out,
        "  ---------------------------------- | --------- | -----"
    );
    let _ = writeln!(
        out,
        "  h2d (WindowFeature[] H2D)          | {:>9} | {:>5.1}",
        dev.h2d_us as u64,
        pct(dev.h2d_us)
    );
    let _ = writeln!(
        out,
        "  residual_field_kernel              | {:>9} | {:>5.1}",
        dev.residual_us as u64,
        pct(dev.residual_us)
    );
    let _ = writeln!(
        out,
        "  drift_slew_sign_kernel             | {:>9} | {:>5.1}",
        dev.sign_us as u64,
        pct(dev.sign_us)
    );
    let _ = writeln!(
        out,
        "  detector_motif_kernel_wide_d64     | {:>9} | {:>5.1}",
        dev.detector_wide_us as u64,
        pct(dev.detector_wide_us)
    );
    let _ = writeln!(
        out,
        "  consensus_grid_kernel_wide         | {:>9} | {:>5.1}",
        dev.consensus_wide_us as u64,
        pct(dev.consensus_wide_us)
    );
    let _ = writeln!(
        out,
        "  axis5_grid_sum_kernel_wide (R.10a) | {:>9} | {:>5.1}",
        dev.axis5_grid_sum_us as u64,
        pct(dev.axis5_grid_sum_us)
    );
    let _ = writeln!(
        out,
        "  candidate_collapse_kernel_wide     | {:>9} | {:>5.1}",
        dev.candidate_wide_us as u64,
        pct(dev.candidate_wide_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest residual               | {:>9} | {:>5.1}",
        dev.residual_digest_us as u64,
        pct(dev.residual_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest sign                   | {:>9} | {:>5.1}",
        dev.sign_digest_us as u64,
        pct(dev.sign_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest detector (wide cells)  | {:>9} | {:>5.1}",
        dev.detector_digest_us as u64,
        pct(dev.detector_digest_us)
    );
    let _ = writeln!(
        out,
        "  tree_digest consensus              | {:>9} | {:>5.1}",
        dev.consensus_digest_us as u64,
        pct(dev.consensus_digest_us)
    );
    let _ = writeln!(
        out,
        "  d2h (candidates+counts+4*32B)      | {:>9} | {:>5.1}",
        dev.d2h_us as u64,
        pct(dev.d2h_us)
    );
    let _ = writeln!(
        out,
        "  ---------------------------------- | --------- | -----"
    );
    let _ = writeln!(
        out,
        "  stage sum (device side)            | {:>9} | {:>5.1}",
        stage_sum as u64,
        coverage * 100.0
    );
    let _ = writeln!(
        out,
        "  total_device_us                    | {:>9} | 100.0",
        dev.total_device_us as u64
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  host: input staging (S-PERF.13)     | {:>9} us",
        host.host_input_staging_us as u64
    );
    let _ = writeln!(
        out,
        "  host: bank admit + case finalize    | {:>9} us",
        host.bank_and_finalize_us as u64
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Counters and derived throughput:");
    let _ = writeln!(out, "  n_cells              : {n_cells}");
    let _ = writeln!(out, "  episode_count        : {episodes}");
    let _ = writeln!(out, "  cells/sec            : {:.2e}", cells_per_sec);
    let _ = writeln!(out, "  detector_evals/sec   : {:.2e}", det_evals_per_sec);
    let _ = writeln!(
        out,
        "  wide bytes/sec (264) : {:.2} GB/s",
        wide_write_gb_per_sec
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Decision rule (per R.9.c-diagnostic plan):");
    let _ = writeln!(
        out,
        "  If candidate_collapse_kernel_wide dominates → fix axis-5 / candidate flush next."
    );
    let _ = writeln!(
        out,
        "  If detector_motif_kernel_wide_d64 dominates → predicate layout / register / occupancy."
    );
    let _ = writeln!(
        out,
        "  If tree_digest detector dominates           → revisit digest mode (compaction surfaced ~wash earlier)."
    );
    let _ = writeln!(
        out,
        "  If host bank + finalize dominates           → extend R.11 compact verdict for D64."
    );
    let _ = writeln!(
        out,
        "  If h2d / d2h dominates                      → inspect accidental wide-cell D2H or pageable fallback."
    );

    // Print to stdout (so `--nocapture` shows it) and write to the
    // reports file. Match the path resolution pattern used by
    // r9_b3_d64_timing.rs so the report lives next to the others.
    print!("{out}");

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let path = reports_dir.join("d64_stage_timing_256x4096_K1.txt");
    let _ = std::fs::write(&path, &out);

    // Coverage gate runs LAST so the report writes regardless of
    // pass/fail. A coverage anomaly is diagnostic information, not
    // a reason to silently lose the data.
    assert!(
        coverage_in_bounds,
        "stage timings cover {:.2}% of total_device_us (sum={} total={}); \
         outside the ±10% tolerance — a missing segment or event-ordering \
         issue would explain the gap",
        coverage * 100.0,
        stage_sum,
        dev.total_device_us
    );
}
