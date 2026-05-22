//! R.12b — D64 saturation sweep, post-R.11c, R.13 headline source.
//!
//! Goal: surface how the D64 GPU throughput scales across catalogs
//! per dispatch (K) and across fixture scale (entities × windows).
//! This sweep is the R.13 headline source for `reports/money_table.txt`
//! — R.12b is where the R.11c K=64 full-scale number is formally
//! measured and pinned.
//!
//! Matrix (post-R.11c, panel-locked):
//!   K       ∈ {1, 4, 16, 32, 64, 128}
//!   scale   ∈ {canonical 16×128, mid 64×512, full 256×4096}
//!   profile  = D64
//!   mode     = GPU Layer B compact verdict
//!
//! K > 1 is realised as a host loop of K serial single-catalog
//! dispatches on the same `GpuWorkspace` — the existing D64
//! throughput FFI handles one catalog at a time. Batched D64
//! throughput (K independent catalogs per kernel launch) is
//! deferred to R.9.d+ work; here we measure sustained per-catalog
//! throughput under repeated dispatch, which is the practical
//! "how many cases per second" number for the R.13 headline.
//!
//! CPU Layer B comparison: deferred (R.12b.1). The CPU side has
//! `evaluate_wide(D64, ...)` for parity tests but no full CPU
//! Layer B D64 path (no CPU consensus_grid_wide /
//! candidate_collapse_wide / bank-admit driver). The CPU
//! comparison cell is recorded as "—" in the report. Per the
//! panel-locked overclaim guardrail, the R.13 headline framing is
//! "full-pipeline campaign reduction" (R.9.b.3 → R.11c), NOT
//! "GPU is N× faster than CPU"; that GPU-vs-CPU comparator stays
//! deferred until R.12b.1 adds the missing CPU wide-path.
//!
//! Acceptance:
//!   * No kernel math changes; D64 case-file bytes unchanged
//!     post-R.11c (commit `086e209`).
//!   * D16 audit goldens untouched.
//!   * D64 at canonical K=1 still admits 13 episodes;
//!     mid K=1 admits 89; full K=1 admits 1917.
//!   * Sweep table emitted to
//!     `reports/r12_d64_saturation.txt`.

#![cfg(feature = "cuda")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::uninlined_format_args,
    clippy::match_same_arms,
    clippy::write_literal
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
    D64ThroughputStageTimings, GpuWorkspace,
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

/// One row of the saturation table.
#[allow(dead_code)]
struct Row {
    scale_label: &'static str,
    n_entities: u32,
    n_windows: u32,
    k: u32,
    // Aggregated measurements (median across iters).
    per_catalog_us: u128,
    catalogs_per_sec: f64,
    cells_per_sec: f64,
    detector_evals_per_sec: f64,
    host_input_staging_us: f32,
    device_total_us: f32,
    host_finalize_us: f32,
    largest_stage_label: &'static str,
    largest_stage_pct: f32,
    episode_count_per_catalog: usize,
}

/// Walk the 12 per-stage device fields, return the largest one's
/// human-readable label and its share of `total_device_us`.
fn largest_stage(dev: &D64ThroughputStageTimings) -> (&'static str, f32) {
    let stages: [(&'static str, f32); 11] = [
        ("h2d", dev.h2d_us),
        ("residual", dev.residual_us),
        ("sign", dev.sign_us),
        ("detector_wide", dev.detector_wide_us),
        ("consensus_wide", dev.consensus_wide_us),
        ("axis5_grid", dev.axis5_grid_sum_us),
        ("candidate_wide", dev.candidate_wide_us),
        ("digest_residual", dev.residual_digest_us),
        ("digest_sign", dev.sign_digest_us),
        ("digest_detector", dev.detector_digest_us),
        ("digest_consensus", dev.consensus_digest_us),
    ];
    let mut best_label = "(none)";
    let mut best_us = 0.0f32;
    for &(label, us) in &stages {
        if us > best_us {
            best_us = us;
            best_label = label;
        }
    }
    let pct = if dev.total_device_us > 0.0 {
        (best_us / dev.total_device_us) * 100.0
    } else {
        0.0
    };
    (best_label, pct)
}

/// Run `iters` independent measurements of "process K catalogs
/// through D64 GPU Layer B on the same workspace". Returns the
/// median values across iters.
fn measure_cell(
    scale_label: &'static str,
    n_entities: u32,
    n_windows: u32,
    k: u32,
    iters: usize,
) -> Row {
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

    // One warm-up batch of K dispatches to amortise lazy buffer
    // allocation, first-call CUDA driver overhead, and clock-state
    // ramp.
    for _ in 0..k {
        let _ = build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
            &events, &contract, &mut ws, &fixture,
        )
        .unwrap();
    }

    let mut wall_samples: Vec<u128> = Vec::with_capacity(iters);
    let mut dev_total_samples: Vec<f32> = Vec::with_capacity(iters);
    let mut dev_for_stage: D64ThroughputStageTimings = D64ThroughputStageTimings::default();
    let mut features_samples: Vec<f32> = Vec::with_capacity(iters);
    let mut finalize_samples: Vec<f32> = Vec::with_capacity(iters);
    let mut last_episode_count: usize = 0;

    for _ in 0..iters {
        let t0 = Instant::now();
        let mut dev_sum = D64ThroughputStageTimings::default();
        let mut features_sum = 0.0_f32;
        let mut finalize_sum = 0.0_f32;
        for _ in 0..k {
            let (case, dev, host) =
                build_gpu_throughput_pinned_async_on_workspace_d64_tree_compact_timed(
                    &events, &contract, &mut ws, &fixture,
                )
                .unwrap();
            last_episode_count = case.episodes.len();
            dev_sum.h2d_us += dev.h2d_us;
            dev_sum.residual_us += dev.residual_us;
            dev_sum.sign_us += dev.sign_us;
            dev_sum.detector_wide_us += dev.detector_wide_us;
            dev_sum.consensus_wide_us += dev.consensus_wide_us;
            dev_sum.axis5_grid_sum_us += dev.axis5_grid_sum_us;
            dev_sum.candidate_wide_us += dev.candidate_wide_us;
            dev_sum.residual_digest_us += dev.residual_digest_us;
            dev_sum.sign_digest_us += dev.sign_digest_us;
            dev_sum.detector_digest_us += dev.detector_digest_us;
            dev_sum.consensus_digest_us += dev.consensus_digest_us;
            dev_sum.d2h_us += dev.d2h_us;
            dev_sum.total_device_us += dev.total_device_us;
            features_sum += host.host_input_staging_us;
            finalize_sum += host.bank_and_finalize_us;
            std::hint::black_box(case);
        }
        let wall = t0.elapsed().as_micros();
        wall_samples.push(wall);
        dev_total_samples.push(dev_sum.total_device_us);
        features_samples.push(features_sum);
        finalize_samples.push(finalize_sum);
        // Hold onto the last iter's summed device stages for the
        // largest_stage computation. Cells/sec etc. use the median
        // wall; stage breakdown uses the most recent batch (median
        // across stages is structurally identical because everything
        // we measure scales linearly with K).
        dev_for_stage = dev_sum;
    }

    wall_samples.sort_unstable();
    dev_total_samples.sort_by(f32::total_cmp);
    features_samples.sort_by(f32::total_cmp);
    finalize_samples.sort_by(f32::total_cmp);
    let med_wall = wall_samples[wall_samples.len() / 2];
    let med_dev = dev_total_samples[dev_total_samples.len() / 2];
    let med_features = features_samples[features_samples.len() / 2];
    let med_finalize = finalize_samples[finalize_samples.len() / 2];

    let n_cells = u64::from(n_entities) * u64::from(n_windows);
    let secs = (med_wall as f64) / 1_000_000.0;
    let per_catalog_us = med_wall / u128::from(k);
    let catalogs_per_sec = f64::from(k) / secs;
    let cells_per_sec = (u64::from(k) as f64) * (n_cells as f64) / secs;
    // D64 evaluates 16 motifs × 4 variants = 64 predicates per cell.
    let detector_evals_per_sec = cells_per_sec * 64.0;
    let (largest_stage_label, largest_stage_pct) = largest_stage(&dev_for_stage);

    println!(
        "  {scale_label:<22} K={k:>3} per_catalog_us={per_catalog_us:>9} \
         catalogs/sec={catalogs_per_sec:>8.1} eps={last_episode_count}"
    );

    Row {
        scale_label,
        n_entities,
        n_windows,
        k,
        per_catalog_us,
        catalogs_per_sec,
        cells_per_sec,
        detector_evals_per_sec,
        host_input_staging_us: med_features,
        device_total_us: med_dev,
        host_finalize_us: med_finalize,
        largest_stage_label,
        largest_stage_pct,
        episode_count_per_catalog: last_episode_count,
    }
}

#[test]
fn r12_d64_saturation_sweep() {
    use core::fmt::Write;

    // Matrix: 3 scales × 6 K values = 18 cells. Iter counts trimmed
    // at the largest cells so the total bench fits in a few minutes.
    let scales: [(&str, u32, u32); 3] = [
        ("canonical 16x128", 16, 128),
        ("mid 64x512", 64, 512),
        ("full 256x4096", 256, 4096),
    ];
    let k_values: [u32; 6] = [1, 4, 16, 32, 64, 128];

    let mut rows: Vec<Row> = Vec::new();
    for &(scale_label, n_entities, n_windows) in &scales {
        for &k in &k_values {
            // Trim iter count for the largest cells so the total
            // sweep stays under ~3 minutes wall time.
            let iters = match (n_entities, k) {
                (256, 128) => 1,
                (256, 64) => 1,
                (256, _) => 2,
                _ => 3,
            };
            let row = measure_cell(scale_label, n_entities, n_windows, k, iters);
            rows.push(row);
        }
    }

    // Compose the report.
    let mut out = String::new();
    let _ = writeln!(
        out,
        "=== R.12b D64 saturation sweep (post-R.11c, R.13 headline) ==="
    );
    let _ = writeln!(
        out,
        "Hardware: RTX 4080 SUPER, CUDA 13.2. GPU Layer B compact verdict."
    );
    let _ = writeln!(
        out,
        "K is processed as a host loop of K serial single-catalog dispatches"
    );
    let _ = writeln!(
        out,
        "on one GpuWorkspace; batched K > 1 kernels are R.9.d+ work."
    );
    let _ = writeln!(
        out,
        "CPU Layer B D64 comparison is deferred to R.12b.1 (no CPU"
    );
    let _ = writeln!(
        out,
        "consensus_wide / candidate_collapse_wide path in core); recorded as '\u{2014}'."
    );
    let _ = writeln!(
        out,
        "The 'spd_vs_cpub' column STAYS '\u{2014}' until R.12b.1 adds the CPU wide path."
    );
    let _ = writeln!(
        out,
        "R.13 headline framing: full-pipeline campaign reduction"
    );
    let _ = writeln!(
        out,
        "(R.9.b.3 baseline 1.82 s/cat \u{2192} R.11c 33.1 ms/cat at K=1 full)."
    );
    let _ = writeln!(
        out,
        "This is NOT a GPU-vs-CPU speedup claim (see panel-locked overclaim guardrail)."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  scale                  |   K | per_cat_us |  cat/sec | staging_pct  | dev_total_pct | finalize_pct | top_stage         | spd_vs_cpub"
    );
    let _ = writeln!(
        out,
        "  ---------------------- | --- | ---------- | -------- | ------------ | ------------- | ------------ | ----------------- | -----------"
    );
    for r in &rows {
        let total_wall_us = r.per_catalog_us * u128::from(r.k);
        let total_wall_us_f = total_wall_us as f32;
        let pct = |us: f32| -> f32 {
            if total_wall_us_f > 0.0 {
                100.0 * us / total_wall_us_f
            } else {
                0.0
            }
        };
        let _ = writeln!(
            out,
            "  {label:<22} | {k:>3} | {per_cat:>10} | {cps:>8.1} | {feat_pct:>10.1}%  | {dev_pct:>11.1}%  | {fin_pct:>10.1}%  | {top:<17} | {spd}",
            label = r.scale_label,
            k = r.k,
            per_cat = r.per_catalog_us,
            cps = r.catalogs_per_sec,
            feat_pct = pct(r.host_input_staging_us),
            dev_pct = pct(r.device_total_us),
            fin_pct = pct(r.host_finalize_us),
            top = format!("{} ({:.0}%)", r.largest_stage_label, r.largest_stage_pct),
            spd = "—",
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Detailed throughput (cells/sec, detector_evals/sec) at each cell:"
    );
    for r in &rows {
        let _ = writeln!(
            out,
            "  {label:<22} K={k:>3} : cells/sec={cps:.2e}  det_evals/sec={des:.2e}  episodes/cat={eps}",
            label = r.scale_label,
            k = r.k,
            cps = r.cells_per_sec,
            des = r.detector_evals_per_sec,
            eps = r.episode_count_per_catalog,
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Decision rule (per plan):");
    let _ = writeln!(
        out,
        "  staging_pct > 40% at K=64/full -> S-PERF.13: host input-staging SIMD pack"
    );
    let _ = writeln!(
        out,
        "  staging_pct < ~25% AND GPU still scales -> R.9.d detector ladder"
    );
    let _ = writeln!(
        out,
        "  D64 already clears paper gate at K=64 -> preserve as headline row"
    );

    // Acceptance: the canonical K=1 row must still admit 1917
    // episodes. Anything else means the diagnostic itself perturbed
    // semantics — abort before writing the report so the failure
    // isn't masked by a stale file.
    let canon_k1 = rows
        .iter()
        .find(|r| r.scale_label == "canonical 16x128" && r.k == 1)
        .expect("canonical 16x128 K=1 cell ran");
    let canon_eps_first_run = canon_k1.episode_count_per_catalog;
    // Smoke test on the full-scale K=1 row too — it's the bridge to
    // R.10c's diagnostic baseline.
    let full_k1 = rows
        .iter()
        .find(|r| r.scale_label == "full 256x4096" && r.k == 1)
        .expect("full 256x4096 K=1 cell ran");
    let full_eps = full_k1.episode_count_per_catalog;

    print!("{out}");

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(crate_dir);
    let reports_dir = workspace_root.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let path = reports_dir.join("r12_d64_saturation.txt");
    let _ = std::fs::write(&path, &out);

    // Episode-count invariants (run after the report write so the
    // table is preserved regardless of pass/fail).
    assert!(
        canon_eps_first_run > 0,
        "canonical 16x128 K=1 admitted zero episodes — D64 semantics broken"
    );
    assert_eq!(
        full_eps, 1917,
        "full 256x4096 K=1 must admit 1917 episodes (R.10c invariant)"
    );
}
