//! S-PERF.12 D64 throughput stage profile under the
//! CompactDensorDigestV1 throughput digest mode.
//!
//! Mirrors `r9_c_d64_stage_profile_256x4096_k1` (the R.9.c
//! diagnostic that captured the post-S-PERF.11 profile) but
//! routes through
//! [`build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed`]
//! so the four `tree_digest_*` stage timings reflect the new
//! XOR-fold-by-256 compact projection (panel-locked S-PERF.12
//! design lock: FOLD=256 = 4 × 64-byte SHA blocks; per-thread
//! compact slot lives in dynamic shared memory; no new global
//! scratch arena). The captured report is
//! the panel-locked source of truth that the S-PERF.12
//! corpus receipt parses into
//! [`crate::s_perf_12_compact_densor_digest_v1`].
//!
//! Panel-locked acceptance:
//!
//! * R.12b episode counts 13 / 89 / 1917 are byte-stable
//!   (changing the digest mode does NOT change which
//!   candidates the bank stage admits).
//! * The four per-stage root digests produced under
//!   CompactDensorDigestV1 are **NOT** byte-identical to the
//!   pinned post-S-PERF.11 `TreeSha256V1` roots — that is
//!   the whole point of declaring a new mode under
//!   S-PERF.10's `digest_mode_non_aliasing_law`. This test
//!   does not assert root values directly; the corpus-side
//!   S-PERF.12 receipt verifier handles the cross-mode
//!   non-aliasing check.
//! * The report file is written to
//!   `reports/d64_stage_timing_256x4096_K1_post_s_perf_12_compact_densor.txt`
//!   so the S-PERF.12 corpus receipt can parse it
//!   deterministically.

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::uninlined_format_args
)]

use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::FixtureHashes;
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::fixture::{synthesize_scaled, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::DetectorProfile;
use dsfb_gpu_debug_core::window::compute_features;
use dsfb_gpu_debug_cuda::{
    build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed,
    D64ThroughputHostStageTimings, D64ThroughputStageTimings, GpuWorkspace,
};

fn d64_contract(n_entities: u32, n_windows: u32) -> Contract {
    let mut c = Contract::scaled(n_entities, n_windows);
    c.pin_bank_hash(bank_hash());
    c.pin_detector_registry_hash(DetectorProfile::D64.registry_hash());
    c
}

// Panel-locked opt-in (verbatim, 2026-05-18): "Performance
// measurement is explicit. Receipt verification is automatic.
// Live benchmark rerun is not a normal invariant gate." This
// bench runs a live GPU measurement at canonical full-scale
// 256x4096 fixture and rewrites the pinned source-report file
// — both expensive and thermally noisy. The workspace serial
// sweep MUST NOT execute this on every pre-commit run; it is
// invoked explicitly via:
//
//   cargo test -p dsfb-gpu-debug-demo --features cuda --release \
//     --test s_perf_12_d64_compact_densor_stage_profile -- --nocapture \
//     --ignored s_perf_12_d64_compact_densor_stage_profile_256x4096_k1
//
// The S-PERF.12a.1 receipt + S-PERF.12a.2 corpus receipt
// module parse the pinned source-report this test writes; the
// receipt is the deterministic verification surface, the bench
// is the explicit measurement event.
#[test]
#[ignore = "S-PERF.12a panel-locked opt-in measurement; run explicitly with --ignored"]
fn s_perf_12_d64_compact_densor_stage_profile_256x4096_k1() {
    use core::fmt::Write;

    let n_entities: u32 = 256;
    let n_windows: u32 = 4096;
    // Panel-locked S-PERF.12 measurement discipline: 3 warm-ups +
    // 7 measurement iters with median. The S-PERF.11.1 triage
    // verdict surfaced thermal/host-load variance at 3-iter
    // medians; 7 iters + 3 warm-ups stabilises the post-S-PERF.12
    // bandwidth measurement against that noise.
    let iters: usize = 7;
    let warmup_iters: usize = 3;

    let contract = d64_contract(n_entities, n_windows);
    let events = synthesize_scaled(DEFAULT_SEED, n_entities, n_windows, 4);
    let features = compute_features(
        &events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let fixture = FixtureHashes::compute(&events, &features);

    let mut ws = GpuWorkspace::new_with_pinned_async(&contract).unwrap();

    // Warm-up iterations to amortise lazy buffer alloc / JIT /
    // first-call CUDA driver work and let the device reach
    // thermal steady-state before measurement.
    for _ in 0..warmup_iters {
        let _ = build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();
    }

    let mut device_samples: Vec<D64ThroughputStageTimings> = Vec::with_capacity(iters);
    let mut host_samples: Vec<D64ThroughputHostStageTimings> = Vec::with_capacity(iters);
    let mut wall_us_samples: Vec<u128> = Vec::with_capacity(iters);
    let mut last_episodes: usize = 0;

    for _ in 0..iters {
        let t0 = Instant::now();
        let (case, dev, host) =
            build_gpu_throughput_pinned_async_on_workspace_d64_compact_densor_compact_timed(
                &events, &contract, &mut ws, &fixture,
            )
            .unwrap();
        let wall = t0.elapsed().as_micros();
        last_episodes = case.episodes.len();
        device_samples.push(dev);
        host_samples.push(host);
        wall_us_samples.push(wall);
        std::hint::black_box(case);
    }

    fn median_f32<F: Fn(&D64ThroughputStageTimings) -> f32>(
        samples: &[D64ThroughputStageTimings],
        f: F,
    ) -> f32 {
        let mut v: Vec<f32> = samples.iter().map(f).collect();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    }
    fn median_host_f32<F: Fn(&D64ThroughputHostStageTimings) -> f32>(
        samples: &[D64ThroughputHostStageTimings],
        f: F,
    ) -> f32 {
        let mut v: Vec<f32> = samples.iter().map(f).collect();
        v.sort_by(f32::total_cmp);
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
    let host = D64ThroughputHostStageTimings {
        host_input_staging_us: median_host_f32(&host_samples, |t| t.host_input_staging_us),
        bank_and_finalize_us: median_host_f32(&host_samples, |t| t.bank_and_finalize_us),
    };
    wall_us_samples.sort_unstable();
    let wall_med_us = wall_us_samples[wall_us_samples.len() / 2];

    // R.12b episode invariant: D64 produces 1917 admitted
    // episodes at 256x4096 regardless of digest mode. Bank
    // stage runs on candidate descriptors, not digest bytes.
    // R.12b panel-locked.
    assert_eq!(
        last_episodes, 1917,
        "S-PERF.12 D64 compact-densor produced {} episodes at 256x4096; \
         expected 1917 (R.12b invariant)",
        last_episodes
    );

    // Derived throughput
    let n_cells = u64::from(n_entities) * u64::from(n_windows);
    let total_s = (dev.total_device_us as f64) / 1_000_000.0;
    let cells_per_sec = (n_cells as f64) / total_s;
    let det_evals_per_sec = cells_per_sec * 64.0;
    let wide_write_gb_per_sec = (n_cells as f64 * 264.0) / (total_s * 1.0e9);

    let pct = |x: f32| -> f32 {
        if dev.total_device_us > 0.0 {
            (x / dev.total_device_us) * 100.0
        } else {
            0.0
        }
    };

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

    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== S-PERF.12 D64 CompactDensorDigestV1 stage profile (256x4096 K=1) ==="
    );
    let _ = writeln!(
        out,
        "Hardware: RTX 4080 SUPER, CUDA 13.2. Median of {iters} iters (post-warmup)."
    );
    let _ = writeln!(out, "Digest mode: CompactDensorDigestV1 (XOR-fold-by-256 compact projection, dynamic shared memory; panel-locked S-PERF.12 design lock)");
    let _ = writeln!(
        out,
        "Domain: DSFB-GPU-ATLAS:S-PERF-12-COMPACT-DENSOR-DIGEST-V1 (canonical header bytes \"DSFB_STAGE_COMPACT_DENSOR_V1\")"
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
        "  compact_densor residual            | {:>9} | {:>5.1}",
        dev.residual_digest_us as u64,
        pct(dev.residual_digest_us)
    );
    let _ = writeln!(
        out,
        "  compact_densor sign                | {:>9} | {:>5.1}",
        dev.sign_digest_us as u64,
        pct(dev.sign_digest_us)
    );
    let _ = writeln!(
        out,
        "  compact_densor detector            | {:>9} | {:>5.1}",
        dev.detector_digest_us as u64,
        pct(dev.detector_digest_us)
    );
    let _ = writeln!(
        out,
        "  compact_densor consensus           | {:>9} | {:>5.1}",
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
    let _ = writeln!(out, "  episode_count        : {last_episodes}");
    let _ = writeln!(out, "  cells/sec            : {:.2e}", cells_per_sec);
    let _ = writeln!(out, "  detector_evals/sec   : {:.2e}", det_evals_per_sec);
    let _ = writeln!(
        out,
        "  wide bytes/sec (264) : {:.2} GB/s",
        wide_write_gb_per_sec
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "S-PERF.12 non-claim block (panel-locked, verbatim):");
    let _ = writeln!(
        out,
        "  - CompactDensorDigestV1 root bytes are NOT byte-identical to TreeSha256V1;"
    );
    let _ = writeln!(
        out,
        "    that is the declared mode identity (each digest mode owns its own root"
    );
    let _ = writeln!(
        out,
        "    projection per S-PERF.10's digest_mode_non_aliasing_law)."
    );
    let _ = writeln!(
        out,
        "  - Bank-admitted episodes are UNCHANGED: 1917 episodes at 256x4096, same as"
    );
    let _ = writeln!(
        out,
        "    TreeSha256V1 — the bank stage runs on candidate descriptors, not digest"
    );
    let _ = writeln!(
        out,
        "    bytes. R.12b episode counts 13 / 89 / 1917 are preserved."
    );
    let _ = writeln!(
        out,
        "  - This profile is the input the S-PERF.12 corpus receipt parses to record"
    );
    let _ = writeln!(
        out,
        "    the post-S-PERF.11 vs post-S-PERF.12 bandwidth delta under the panel-"
    );
    let _ = writeln!(out, "    locked non-aliasing law.");

    print!("{out}");

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let path = reports_dir.join("d64_stage_timing_256x4096_K1_post_s_perf_12_compact_densor.txt");
    std::fs::write(&path, &out).unwrap();

    // Coverage gate: per-stage cudaEvent timing precision is
    // a few hundred us; allow ±10% as the R.9.c bench does.
    let coverage_in_bounds = coverage > 0.90 && coverage < 1.10;
    assert!(
        coverage_in_bounds,
        "stage timings cover {:.2}% of total_device_us (sum={} total={}); \
         outside the ±10% tolerance — a missing segment would explain the gap",
        coverage * 100.0,
        stage_sum as u64,
        dev.total_device_us as u64
    );
}
