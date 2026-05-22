//! `dsfb-gpu-debug bench` — measure pipeline wall-clock on CPU and GPU.
//!
//! Times only the pipeline functions (no fixture I/O, no canonical JSON
//! emission, no hash-chain construction outside the bare pipeline). The
//! goal is to surface the cost of the deterministic-inference chain
//! itself, separated from the surrounding orchestration.
//!
//! Flags:
//!
//! * `--iters N` (default 100): measured iterations.
//! * `--warmup N` (default 10): warmup iterations excluded from stats.
//!   Excluding the warmup from stats is what keeps first-call CUDA
//!   context creation out of the published numbers — the user's
//!   step 6 in the optimization roadmap.
//! * `--backend cpu|gpu|both` (default `both`).
//! * `--detail` (no value): also report per-stage CUDA-event timings
//!   for the GPU path (alloc / H2D / kernel 1..5 / D2H / free / total).
//!   Implies `--backend gpu` if not otherwise set.
//!
//! Numbers are reported as min / median / mean / max in microseconds.
//! No formal benchmarking framework — this is a transparency tool, not
//! a deployment performance claim. v0 launch geometry is 1 thread per
//! entity (16 threads), which is dramatically under-utilized hardware;
//! the bench reports what the architecture actually does today and
//! makes that posture observable.
//!
//! `.expect()` is used inside the GPU bench so a CUDA error aborts
//! loudly rather than silently producing meaningless timings.

#![allow(clippy::expect_used)]

use std::process::ExitCode;
use std::time::Instant;

use dsfb_gpu_debug_core::bank::bank_hash;
use dsfb_gpu_debug_core::casefile::{build_cpu, build_cpu_throughput};
use dsfb_gpu_debug_core::contract::Contract;
use dsfb_gpu_debug_core::event::TraceEvent;
use dsfb_gpu_debug_core::fixture::{synthesize, synthesize_scaled, DEFAULT_SEED};
use dsfb_gpu_debug_core::motif::registry_hash;

#[cfg(feature = "cuda")]
use dsfb_gpu_debug_cuda::{
    build_gpu_batched_throughput, build_gpu_batched_throughput_device_digests,
    build_gpu_layer_a_batched, build_gpu_layer_a_on_workspace, build_gpu_on_workspace,
    build_gpu_throughput_device_digests_on_workspace, build_gpu_throughput_on_workspace,
    build_gpu_timed_on_workspace, BatchedGpuWorkspace, GpuWorkspace,
};

use super::{parse_flags, usage_error};

#[allow(clippy::too_many_lines)]
pub fn parse_and_run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args) {
        Ok(f) => f,
        Err(message) => return usage_error(&message),
    };

    let iters: usize = flags
        .get("iters")
        .map_or(100, |s| s.parse::<usize>().unwrap_or(100));
    let warmup: usize = flags
        .get("warmup")
        .map_or(10, |s| s.parse::<usize>().unwrap_or(10));
    // `--detail` accepts no value; presence in the flags map (under the
    // hand-rolled parser's `--key value` convention) is enough. Treat any
    // value other than `false` as opt-in.
    let detail = flags.get("detail").is_some_and(|v| v != "false");
    // If the user asked for `--detail` without setting `--backend`, run
    // the GPU path; per-stage timings aren't meaningful for the CPU path.
    let backend = if detail && !flags.contains_key("backend") {
        "gpu"
    } else {
        flags.get("backend").map_or("both", String::as_str)
    };
    // `--mode audit|throughput|both`. Default is `audit` so the legacy
    // numbers continue to be the headline. `both` runs each mode in
    // sequence for direct comparison.
    let mode = flags.get("mode").map_or("audit", String::as_str);
    // `--layer A|B|C|all`. R.1: three-layer benchmark taxonomy.
    //
    //   * Layer A — device evidence fabric: GPU kernels + on-device
    //     digests; no host bank stage. (Skip-bank specialization
    //     lands in R.3; for R.1 we report the device-digests path
    //     as Layer A and note the caveat in the output file.)
    //   * Layer B — throughput verdict summary: Layer A + host bank
    //     stage finalises compact candidates into admitted episodes.
    //   * Layer C — full audit court: every intermediate cell
    //     materialised host-side; canonical-JSON case file emitted.
    //
    // When `--layer` is unset, the bench falls through to the legacy
    // `--mode`-based dispatch for backwards compatibility. When set,
    // the bench runs the selected layer's path and writes a layer
    // report to `reports/layer_<L>_<grid>x<windows>_K<N>.txt`.
    let layer = flags.get("layer").map(String::as_str);
    // `--scale n_entities x n_windows`. When set, replaces the canonical
    // 16x128 grid with a larger one and uses `synthesize_scaled` to fill
    // it. Recommended values for the paper's scaling table: 16x128,
    // 64x512, 128x1024, 256x1024.
    //
    // `--scale-large` (R.2): shorthand for `--scale 256x4096`, the
    // panel-locked headline profile for the courthouse-factory money
    // table. Mutually exclusive with `--scale`; explicit `--scale` wins.
    let scale = flags.get("scale").and_then(|s| parse_scale(s)).or_else(|| {
        flags
            .get("scale-large")
            .filter(|v| v.as_str() != "false")
            .map(|_| (256u32, 4096u32))
    });
    // `--materialize-catalog J` (R.3a): when set under `--layer A`,
    // after the Layer A bench reports the device-fabric measurements,
    // synthesize catalog J via the courthouse-factory generator and
    // run a single Layer C (full audit court) on it. The other K-1
    // catalogs stay in compact-summary form. This is the panel's
    // "docket digest with on-demand transcript expansion" — most
    // catalogs ride the fast path; the one a reviewer asks about
    // gets the full court transcript.
    let materialize_catalog: Option<u32> = flags
        .get("materialize-catalog")
        .and_then(|s| s.parse::<u32>().ok());
    // `--batch K`. When set (and the cuda feature is on), runs K
    // independent catalogs through `build_gpu_batched_throughput` per
    // iteration. Reports cases/sec and per-catalog amortized time so
    // the GPU-side parallelism win is visible. The cfg gate silences
    // the unused-variable lint on no-cuda builds; the rest of the
    // logic below dispatches the batched runner conditionally.
    #[cfg(feature = "cuda")]
    let batch: u32 = flags
        .get("batch")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    #[cfg(not(feature = "cuda"))]
    let _ = flags.get("batch");

    // Build the canonical inputs once.
    let (events, contract_dims, scaled_label) = match scale {
        None => (synthesize(DEFAULT_SEED), (16u32, 128u32), String::new()),
        Some((n_entities, n_windows)) => {
            let events = synthesize_scaled(DEFAULT_SEED, n_entities, n_windows, 4);
            (
                events,
                (n_entities, n_windows),
                format!(" [scaled {n_entities}x{n_windows}]"),
            )
        }
    };
    let mut contract = if scale.is_some() {
        Contract::scaled(contract_dims.0, contract_dims.1)
    } else {
        Contract::canonical()
    };
    contract.pin_bank_hash(bank_hash());
    contract.pin_detector_registry_hash(registry_hash());

    println!("dsfb-gpu-debug bench:{scaled_label}");
    println!("  events    : {}", events.len());
    println!("  n_entities: {}", contract.n_entities);
    println!("  n_windows : {}", contract.n_windows);
    println!("  warmup    : {warmup}");
    println!("  iters     : {iters}");
    println!();

    let run_audit = mode == "audit" || mode == "both";
    let run_throughput = mode == "throughput" || mode == "both";

    // R.1 — three-layer benchmark taxonomy. When `--layer` is set, the
    // layer-aware dispatcher runs and the legacy `--mode` path is
    // skipped. The legacy path remains the default for backwards
    // compatibility so existing bench invocations keep their numbers.
    if let Some(layer_spec) = layer {
        let layers: &[char] = match layer_spec {
            "A" | "a" => &['A'],
            "B" | "b" => &['B'],
            "C" | "c" => &['C'],
            "all" | "ABC" | "abc" => &['A', 'B', 'C'],
            other => {
                eprintln!("unknown --layer {other:?}; expected A | B | C | all");
                return ExitCode::from(1);
            }
        };
        let reports_dir = std::path::Path::new("reports");
        for &l in layers {
            run_layer_bench(
                l,
                &events,
                &contract,
                warmup,
                iters,
                #[cfg(feature = "cuda")]
                batch,
                #[cfg(not(feature = "cuda"))]
                0,
                Some(reports_dir),
            );
        }
        // R.3a — opt-in single-catalog transcript expansion. After the
        // Layer A/B/C bench reports the fabric numbers, if the caller
        // asked for `--materialize-catalog J`, run a Layer C audit on
        // catalog J specifically and report it. This honours the
        // "docket digest with on-demand transcript expansion" framing.
        if let Some(j) = materialize_catalog {
            run_materialize_catalog(j, &events, &contract, warmup.max(1), iters.max(1));
        }
        return ExitCode::SUCCESS;
    }

    if backend == "cpu" || backend == "both" {
        if run_audit {
            run_cpu_bench_audit(&events, &contract, warmup, iters);
        }
        if run_throughput {
            run_cpu_bench_throughput(&events, &contract, warmup, iters);
        }
    }

    // Tier 3B device-digest variants. `--device-digests` enables the
    // on-device per-stage SHA-256 path. When also batched (`--batch K`),
    // runs the parallel digest dispatcher with K SHA-256 streams. The
    // case files are byte-identical to the host-digest Throughput path
    // (pinned by `throughput_device_digests_equivalence`); only the
    // wall-time bookkeeping differs.
    #[cfg(feature = "cuda")]
    let device_digests = flags.get("device-digests").is_some_and(|v| v != "false");
    #[cfg(not(feature = "cuda"))]
    let _ = flags.get("device-digests");

    #[cfg(feature = "cuda")]
    if backend == "gpu" || backend == "both" {
        if detail {
            run_gpu_bench_with_detail(&events, &contract, warmup, iters);
        } else {
            if run_audit {
                run_gpu_bench_audit(&events, &contract, warmup, iters);
            }
            if run_throughput {
                run_gpu_bench_throughput(&events, &contract, warmup, iters);
            }
            if device_digests && run_throughput {
                run_gpu_bench_throughput_device_digests(&events, &contract, warmup, iters);
            }
            if batch > 0 {
                run_gpu_bench_batched(&events, &contract, batch, warmup, iters);
            }
            if batch > 0 && device_digests {
                run_gpu_bench_batched_device_digests(&events, &contract, batch, warmup, iters);
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    if backend == "gpu" || backend == "both" {
        let _ = detail;
        println!("GPU pipeline: built without --features cuda; skipping");
    }

    ExitCode::SUCCESS
}

/// Parse `<n_entities>x<n_windows>` (e.g. "256x1024") into a dimension
/// pair. Returns `None` for malformed input; the CLI then runs with
/// the canonical 16x128 contract.
fn parse_scale(s: &str) -> Option<(u32, u32)> {
    let (n_entities_s, n_windows_s) = s.split_once('x')?;
    let n_entities: u32 = n_entities_s.parse().ok()?;
    let n_windows: u32 = n_windows_s.parse().ok()?;
    if n_entities == 0 || n_windows == 0 {
        return None;
    }
    Some((n_entities, n_windows))
}

fn run_cpu_bench_audit(events: &[TraceEvent], contract: &Contract, warmup: usize, iters: usize) {
    for _ in 0..warmup {
        let _ = std::hint::black_box(build_cpu(events, contract));
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_cpu(events, contract);
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    report("CPU pipeline (Audit, build_cpu)", &samples_us);
}

fn run_cpu_bench_throughput(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) {
    for _ in 0..warmup {
        let _ = std::hint::black_box(build_cpu_throughput(events, contract));
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_cpu_throughput(events, contract);
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    report(
        "CPU pipeline (Throughput, build_cpu_throughput)",
        &samples_us,
    );
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_audit(events: &[TraceEvent], contract: &Contract, warmup: usize, iters: usize) {
    // Workspace allocated once, before warmup and the measured iters.
    // This is what makes the GPU numbers reflect the steady-state cost
    // of the deterministic-inference pipeline rather than the one-shot
    // cudaMalloc storm a fresh call paid for in v0.
    let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
    for _ in 0..warmup {
        let case = build_gpu_on_workspace(&mut workspace, events, contract).expect("CUDA pipeline");
        std::hint::black_box(case);
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_gpu_on_workspace(&mut workspace, events, contract).expect("CUDA pipeline");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    report(
        "GPU pipeline (Audit, workspace-resident, sm_75/80/89)",
        &samples_us,
    );
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_batched(
    events: &[TraceEvent],
    contract: &Contract,
    batch: u32,
    warmup: usize,
    iters: usize,
) {
    // Build K independent fixtures by perturbing the LCG seed. The
    // event payload differs per catalog, so the GPU sees real cross-
    // catalog independence (not K copies of the same bytes that the
    // memory subsystem could trivially de-duplicate).
    // R.2 courthouse-factory generator. Catalog 0 reuses the bench's
    // canonical `events` slice (so single-vs-batched comparisons share
    // catalog 0 exactly); the remaining K-1 catalogs come from the
    // factory at the same `(n_entities, n_windows)` grid. The Knuth
    // golden-ratio constant matches the previous `wrapping_mul(0x9E37)`
    // pattern that the bench used before R.2.
    let mut fixtures: Vec<Vec<TraceEvent>> = Vec::with_capacity(batch as usize);
    fixtures.push(events.to_vec());
    if batch > 1 {
        let extra = dsfb_gpu_debug_core::fixture::synthesize_courthouse_factory(
            dsfb_gpu_debug_core::fixture::DEFAULT_SEED.wrapping_add(0x9E37_79B9_7F4A_7C15),
            batch - 1,
            contract.n_entities,
            contract.n_windows,
            4,
        );
        fixtures.extend(extra);
    }
    let event_slices: Vec<&[TraceEvent]> = fixtures.iter().map(Vec::as_slice).collect();

    let mut workspace =
        BatchedGpuWorkspace::new(batch, contract).expect("batched workspace allocation");

    for _ in 0..warmup {
        let cases =
            build_gpu_batched_throughput(&mut workspace, &event_slices, contract).expect("CUDA");
        std::hint::black_box(cases);
    }

    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let cases =
            build_gpu_batched_throughput(&mut workspace, &event_slices, contract).expect("CUDA");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(cases);
        samples_us.push(dt);
    }

    let label = format!("GPU pipeline (Batched K={batch}, workspace-resident, sm_75/80/89)");
    report(&label, &samples_us);

    // Per-catalog amortized time + cases/sec. These are the metrics
    // the user roadmap calls the headline numbers for the batched
    // dispatch.
    let mut sorted = samples_us.clone();
    sorted.sort_unstable();
    let median_us = sorted[sorted.len() / 2];
    let per_catalog_us = median_us / u128::from(batch);
    let cases_per_sec = if median_us > 0 {
        1_000_000u128 * u128::from(batch) / median_us
    } else {
        0
    };
    println!(
        "  per-catalog amortized: {per_catalog_us} us    throughput: {cases_per_sec} cases/sec"
    );
    println!();
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_throughput(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) {
    let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
    for _ in 0..warmup {
        let case = build_gpu_throughput_on_workspace(&mut workspace, events, contract)
            .expect("CUDA pipeline");
        std::hint::black_box(case);
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_gpu_throughput_on_workspace(&mut workspace, events, contract)
            .expect("CUDA pipeline");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    report(
        "GPU pipeline (Throughput, workspace-resident, sm_75/80/89)",
        &samples_us,
    );
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_with_detail(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) {
    // Same pattern as `run_gpu_bench`: one workspace amortized across
    // warmup + measured iterations.
    let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");

    // Warmup uses the timing path too so its CUDA-event setup cost is
    // not folded into the measured samples.
    for _ in 0..warmup {
        let (case, _) =
            build_gpu_timed_on_workspace(&mut workspace, events, contract).expect("CUDA pipeline");
        std::hint::black_box(case);
    }

    // Per-iteration: collect host wall-clock plus the eight per-stage
    // microsecond fields from the CUDA-event timings struct. Each is
    // reported with the same min/median/mean/max statistics.
    let mut wall_us: Vec<u128> = Vec::with_capacity(iters);
    let mut alloc_us: Vec<u128> = Vec::with_capacity(iters);
    let mut h2d_us: Vec<u128> = Vec::with_capacity(iters);
    let mut k1_us: Vec<u128> = Vec::with_capacity(iters);
    let mut k2_us: Vec<u128> = Vec::with_capacity(iters);
    let mut k3_us: Vec<u128> = Vec::with_capacity(iters);
    let mut k4_us: Vec<u128> = Vec::with_capacity(iters);
    let mut k5_us: Vec<u128> = Vec::with_capacity(iters);
    let mut d2h_us: Vec<u128> = Vec::with_capacity(iters);
    let mut free_us: Vec<u128> = Vec::with_capacity(iters);
    let mut device_total_us: Vec<u128> = Vec::with_capacity(iters);

    for _ in 0..iters {
        let t0 = Instant::now();
        let (case, t) =
            build_gpu_timed_on_workspace(&mut workspace, events, contract).expect("CUDA pipeline");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        wall_us.push(dt);
        push_f32_as_u128(&mut alloc_us, t.alloc_us);
        push_f32_as_u128(&mut h2d_us, t.h2d_us);
        push_f32_as_u128(&mut k1_us, t.k1_residual_us);
        push_f32_as_u128(&mut k2_us, t.k2_sign_us);
        push_f32_as_u128(&mut k3_us, t.k3_detector_us);
        push_f32_as_u128(&mut k4_us, t.k4_consensus_us);
        push_f32_as_u128(&mut k5_us, t.k5_candidate_us);
        push_f32_as_u128(&mut d2h_us, t.d2h_us);
        push_f32_as_u128(&mut free_us, t.free_us);
        push_f32_as_u128(&mut device_total_us, t.total_us);
    }

    println!("GPU pipeline (build_gpu_timed --detail)");
    report_inline("host wall time     ", &wall_us);
    report_inline("device alloc       ", &alloc_us);
    report_inline("H2D (window feats) ", &h2d_us);
    report_inline("k1 residual_field  ", &k1_us);
    report_inline("k2 drift_slew_sign ", &k2_us);
    report_inline("k3 detector_motif  ", &k3_us);
    report_inline("k4 consensus_grid  ", &k4_us);
    report_inline("k5 candidate_coll. ", &k5_us);
    report_inline("D2H (all stages)   ", &d2h_us);
    report_inline("device free        ", &free_us);
    report_inline("device total       ", &device_total_us);
    println!();
}

/// Convert a `f32` microsecond reading to `u128` for stat aggregation.
/// CUDA events return millisecond floats; the kernel wrapper has
/// already converted to µs. We round to the nearest integer µs to
/// keep the sample type uniform with the wall-clock samples.
#[cfg(feature = "cuda")]
fn push_f32_as_u128(samples: &mut Vec<u128>, val: f32) {
    samples.push(val.round().max(0.0) as u128);
}

#[cfg(feature = "cuda")]
fn report_inline(label: &str, samples_us: &[u128]) {
    if samples_us.is_empty() {
        return;
    }
    let mut sorted = samples_us.to_vec();
    sorted.sort_unstable();
    let n = sorted.len() as u128;
    let min = *sorted.first().unwrap_or(&0);
    let max = *sorted.last().unwrap_or(&0);
    let median = sorted[sorted.len() / 2];
    let mean = sorted.iter().sum::<u128>() / n;
    println!(
        "  {label}  min={min:>6} us  median={median:>6} us  mean={mean:>6} us  max={max:>6} us"
    );
}

fn report(label: &str, samples_us: &[u128]) {
    let mut sorted = samples_us.to_vec();
    sorted.sort_unstable();
    let n = sorted.len() as u128;
    let min = *sorted.first().unwrap_or(&0);
    let max = *sorted.last().unwrap_or(&0);
    let median = sorted[sorted.len() / 2];
    let sum: u128 = sorted.iter().sum();
    let mean = if n == 0 { 0 } else { sum / n };
    println!("{label}");
    println!("  min    : {min:>8} us");
    println!("  median : {median:>8} us");
    println!("  mean   : {mean:>8} us");
    println!("  max    : {max:>8} us");
    println!("  samples: {n}");
    println!();
}

/// Layer-specific metrics block for R.1's three-layer benchmark
/// taxonomy. Reports catalogs/sec, cells/sec, and detector-evaluations
/// per second alongside the standard min/median/mean/max. The cells
/// and detector-evaluations counts use the actual grid dimensions and
/// the canonical 16-motif registry, so the per-second figures reflect
/// real evidence-field throughput rather than per-call wall time alone.
///
/// `samples_us` is the per-iteration wall time. `n_catalogs` is 1 for
/// the single-catalog path and K for the batched path. The median is
/// the reference because it is robust to occasional CUDA-event noise.
#[allow(clippy::too_many_arguments)]
fn report_layer(
    label: &str,
    samples_us: &[u128],
    layer: char,
    n_entities: u32,
    n_windows: u32,
    n_catalogs: u32,
    n_detectors: u32,
    out_dir: Option<&std::path::Path>,
    file_tag: &str,
) {
    let mut sorted = samples_us.to_vec();
    sorted.sort_unstable();
    let n_samples = sorted.len() as u128;
    let min = *sorted.first().unwrap_or(&0);
    let max = *sorted.last().unwrap_or(&0);
    let median = if sorted.is_empty() {
        0
    } else {
        sorted[sorted.len() / 2]
    };
    let sum: u128 = sorted.iter().sum();
    let mean = if n_samples == 0 { 0 } else { sum / n_samples };

    let catalogs = u128::from(n_catalogs);
    let cells = catalogs * u128::from(n_entities) * u128::from(n_windows);
    let det_evals = cells * u128::from(n_detectors);
    let one_sec = 1_000_000u128;
    let catalogs_per_sec = if median > 0 {
        catalogs * one_sec / median
    } else {
        0
    };
    let cells_per_sec = if median > 0 {
        cells * one_sec / median
    } else {
        0
    };
    let det_evals_per_sec = if median > 0 {
        det_evals * one_sec / median
    } else {
        0
    };
    let per_catalog_us = if catalogs > 0 {
        median / catalogs
    } else {
        median
    };

    println!("{label} [Layer {layer}]");
    println!("  min                  : {min:>10} us");
    println!("  median               : {median:>10} us");
    println!("  mean                 : {mean:>10} us");
    println!("  max                  : {max:>10} us");
    println!("  samples              : {n_samples}");
    println!("  n_catalogs (K)       : {n_catalogs}");
    println!("  per-catalog amortized: {per_catalog_us:>10} us");
    println!("  catalogs/sec         : {catalogs_per_sec}");
    println!("  cells/sec            : {cells_per_sec}");
    println!("  detector-evals/sec   : {det_evals_per_sec}");
    println!();

    if let Some(out_dir) = out_dir {
        let _ = std::fs::create_dir_all(out_dir);
        let filename =
            format!("layer_{layer}{file_tag}_{n_entities}x{n_windows}_K{n_catalogs}.txt");
        let path = out_dir.join(filename);
        let body = format!(
            "{label} [Layer {layer}]\n\
             n_entities       : {n_entities}\n\
             n_windows        : {n_windows}\n\
             n_catalogs (K)   : {n_catalogs}\n\
             n_detectors      : {n_detectors}\n\
             samples          : {n_samples}\n\
             min_us           : {min}\n\
             median_us        : {median}\n\
             mean_us          : {mean}\n\
             max_us           : {max}\n\
             per_catalog_us   : {per_catalog_us}\n\
             catalogs_per_sec : {catalogs_per_sec}\n\
             cells_per_sec    : {cells_per_sec}\n\
             det_evals_per_sec: {det_evals_per_sec}\n"
        );
        if let Err(e) = std::fs::write(&path, body) {
            eprintln!("warning: could not write {}: {e}", path.display());
        } else {
            println!("  wrote layer report -> {}", path.display());
            println!();
        }
    }
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_throughput_device_digests(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) {
    let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
    for _ in 0..warmup {
        let case =
            build_gpu_throughput_device_digests_on_workspace(events, contract, &mut workspace)
                .expect("CUDA pipeline (device digests)");
        std::hint::black_box(case);
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let case =
            build_gpu_throughput_device_digests_on_workspace(events, contract, &mut workspace)
                .expect("CUDA pipeline (device digests)");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    report(
        "GPU pipeline (Throughput, Tier 3B on-device SHA-256, sm_75/80/89)",
        &samples_us,
    );
}

#[cfg(feature = "cuda")]
fn run_gpu_bench_batched_device_digests(
    events: &[TraceEvent],
    contract: &Contract,
    batch: u32,
    warmup: usize,
    iters: usize,
) {
    // R.2 courthouse-factory generator. Catalog 0 reuses the bench's
    // canonical `events` slice (so single-vs-batched comparisons share
    // catalog 0 exactly); the remaining K-1 catalogs come from the
    // factory at the same `(n_entities, n_windows)` grid. The Knuth
    // golden-ratio constant matches the previous `wrapping_mul(0x9E37)`
    // pattern that the bench used before R.2.
    let mut fixtures: Vec<Vec<TraceEvent>> = Vec::with_capacity(batch as usize);
    fixtures.push(events.to_vec());
    if batch > 1 {
        let extra = dsfb_gpu_debug_core::fixture::synthesize_courthouse_factory(
            dsfb_gpu_debug_core::fixture::DEFAULT_SEED.wrapping_add(0x9E37_79B9_7F4A_7C15),
            batch - 1,
            contract.n_entities,
            contract.n_windows,
            4,
        );
        fixtures.extend(extra);
    }
    let event_slices: Vec<&[TraceEvent]> = fixtures.iter().map(Vec::as_slice).collect();

    let mut workspace =
        BatchedGpuWorkspace::new(batch, contract).expect("batched workspace allocation");

    for _ in 0..warmup {
        let cases =
            build_gpu_batched_throughput_device_digests(&mut workspace, &event_slices, contract)
                .expect("CUDA pipeline (batched device digests)");
        std::hint::black_box(cases);
    }
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let cases =
            build_gpu_batched_throughput_device_digests(&mut workspace, &event_slices, contract)
                .expect("CUDA pipeline (batched device digests)");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(cases);
        samples_us.push(dt);
    }
    let label = format!("GPU pipeline (Batched K={batch}, Tier 3B on-device SHA-256, sm_75/80/89)");
    report(&label, &samples_us);

    let mut sorted = samples_us.clone();
    sorted.sort_unstable();
    let median_us = sorted[sorted.len() / 2];
    let per_catalog_us = median_us / u128::from(batch);
    let cases_per_sec = if median_us > 0 {
        1_000_000u128 * u128::from(batch) / median_us
    } else {
        0
    };
    println!(
        "  per-catalog amortized: {per_catalog_us} us    throughput: {cases_per_sec} cases/sec"
    );
    println!();
}

/// R.1 layer-aware bench runner. Dispatches one of the three layer
/// flavours, collects samples, and emits a layer report (console +
/// `reports/layer_<L>_<grid>x<windows>_K<N>.txt`).
///
/// Layer mapping at R.1 (skip-bank specialization for Layer A arrives in R.3):
///   * Layer A — GPU device-digests path (closest current proxy for
///     "device evidence fabric only"; full skip-bank lands in R.3).
///   * Layer B — GPU throughput path with host bank stage.
///   * Layer C — full audit court (CPU and GPU audit, side by side).
///
/// `batch` is the K from `--batch K`; 0 means single-catalog. When the
/// cuda feature is off, GPU layers print a notice and return; CPU
/// Layer C still runs.
#[allow(clippy::too_many_lines)]
fn run_layer_bench(
    layer: char,
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
    batch: u32,
    out_dir: Option<&std::path::Path>,
) {
    let n_entities = contract.n_entities;
    let n_windows = contract.n_windows;
    let n_detectors = dsfb_gpu_debug_core::motif::MotifClass::COUNT as u32;
    // Reported K (1 for single-catalog, batch otherwise). Used inside the
    // cuda branches; the `let _` silences the unused-variable lint on
    // builds without the cuda feature where the variable is informational.
    #[cfg(feature = "cuda")]
    let n_catalogs = if batch == 0 { 1 } else { batch };
    #[cfg(not(feature = "cuda"))]
    let n_catalogs: u32 = if batch == 0 { 1 } else { batch };
    #[cfg(not(feature = "cuda"))]
    let _ = n_catalogs;

    match layer {
        'A' => {
            #[cfg(feature = "cuda")]
            {
                let samples = run_layer_a(events, contract, batch, warmup, iters);
                let label = if batch == 0 {
                    String::from(
                        "Layer A — device evidence fabric (Tier 3B device-digests, single-catalog)",
                    )
                } else {
                    format!(
                        "Layer A — device evidence fabric (Tier 3B device-digests, K={batch} batched)"
                    )
                };
                report_layer(
                    &label,
                    &samples,
                    'A',
                    n_entities,
                    n_windows,
                    n_catalogs,
                    n_detectors,
                    out_dir,
                    "",
                );
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = (events, contract, batch, warmup, iters);
                println!("Layer A — GPU pipeline: built without --features cuda; skipping");
                println!();
            }
        }
        'B' => {
            #[cfg(feature = "cuda")]
            {
                let samples = run_layer_b(events, contract, batch, warmup, iters);
                let label = if batch == 0 {
                    String::from(
                        "Layer B — throughput verdict summary (host bank stage, single-catalog)",
                    )
                } else {
                    format!(
                        "Layer B — throughput verdict summary (host bank stage, K={batch} batched)"
                    )
                };
                report_layer(
                    &label,
                    &samples,
                    'B',
                    n_entities,
                    n_windows,
                    n_catalogs,
                    n_detectors,
                    out_dir,
                    "",
                );
            }
            #[cfg(not(feature = "cuda"))]
            {
                let samples = run_layer_b_cpu(events, contract, warmup, iters);
                report_layer(
                    "Layer B — throughput verdict summary (CPU-only, no CUDA feature)",
                    &samples,
                    'B',
                    n_entities,
                    n_windows,
                    1,
                    n_detectors,
                    out_dir,
                    "_cpu",
                );
            }
        }
        'C' => {
            // Layer C: full audit court. Run both CPU and GPU sides so
            // the audit cost is visible per backend. The CPU and GPU
            // reports write to distinct files via the `_cpu` / `_gpu`
            // file_tag suffix so neither overwrites the other.
            let cpu_samples = run_layer_c_cpu(events, contract, warmup, iters);
            report_layer(
                "Layer C — full audit court (CPU)",
                &cpu_samples,
                'C',
                n_entities,
                n_windows,
                1,
                n_detectors,
                out_dir,
                "_cpu",
            );
            #[cfg(feature = "cuda")]
            {
                let gpu_samples = run_layer_c_gpu(events, contract, warmup, iters);
                report_layer(
                    "Layer C — full audit court (GPU)",
                    &gpu_samples,
                    'C',
                    n_entities,
                    n_windows,
                    1,
                    n_detectors,
                    out_dir,
                    "_gpu",
                );
            }
        }
        other => {
            eprintln!("run_layer_bench: unknown layer '{other}'");
        }
    }
}

#[cfg(feature = "cuda")]
pub(crate) fn run_layer_a(
    events: &[TraceEvent],
    contract: &Contract,
    batch: u32,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    // R.3a — Layer A bench uses the new skip-bank dispatch path
    // (`build_gpu_layer_a_*`) which returns `CompactCaseSummary` and
    // does NOT run the host bank stage. The bench therefore measures
    // the pure evidence-fabric cost, separated from per-catalog bank
    // admission. The single-vs-batched dispatch mirrors the Tier 3B
    // pattern from Section Q.
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    if batch == 0 {
        let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
        for _ in 0..warmup {
            let summary = build_gpu_layer_a_on_workspace(events, contract, &mut workspace)
                .expect("CUDA Layer A (skip-bank) pipeline");
            std::hint::black_box(summary);
        }
        for _ in 0..iters {
            let t0 = Instant::now();
            let summary = build_gpu_layer_a_on_workspace(events, contract, &mut workspace)
                .expect("CUDA Layer A (skip-bank) pipeline");
            let dt = t0.elapsed().as_micros();
            std::hint::black_box(summary);
            samples_us.push(dt);
        }
    } else {
        // R.2 courthouse-factory generator. Catalog 0 reuses the bench's
        // canonical `events` slice (so single-vs-batched comparisons share
        // catalog 0 exactly); the remaining K-1 catalogs come from the
        // factory at the same `(n_entities, n_windows)` grid.
        let mut fixtures: Vec<Vec<TraceEvent>> = Vec::with_capacity(batch as usize);
        fixtures.push(events.to_vec());
        if batch > 1 {
            let extra = dsfb_gpu_debug_core::fixture::synthesize_courthouse_factory(
                dsfb_gpu_debug_core::fixture::DEFAULT_SEED.wrapping_add(0x9E37_79B9_7F4A_7C15),
                batch - 1,
                contract.n_entities,
                contract.n_windows,
                4,
            );
            fixtures.extend(extra);
        }
        let event_slices: Vec<&[TraceEvent]> = fixtures.iter().map(Vec::as_slice).collect();
        let mut workspace =
            BatchedGpuWorkspace::new(batch, contract).expect("batched workspace allocation");
        for _ in 0..warmup {
            let summaries = build_gpu_layer_a_batched(&mut workspace, &event_slices, contract)
                .expect("CUDA Layer A (batched skip-bank) pipeline");
            std::hint::black_box(summaries);
        }
        for _ in 0..iters {
            let t0 = Instant::now();
            let summaries = build_gpu_layer_a_batched(&mut workspace, &event_slices, contract)
                .expect("CUDA Layer A (batched skip-bank) pipeline");
            let dt = t0.elapsed().as_micros();
            std::hint::black_box(summaries);
            samples_us.push(dt);
        }
    }
    samples_us
}

#[cfg(feature = "cuda")]
pub(crate) fn run_layer_b(
    events: &[TraceEvent],
    contract: &Contract,
    batch: u32,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    if batch == 0 {
        let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
        for _ in 0..warmup {
            let case = build_gpu_throughput_on_workspace(&mut workspace, events, contract)
                .expect("CUDA pipeline (throughput)");
            std::hint::black_box(case);
        }
        for _ in 0..iters {
            let t0 = Instant::now();
            let case = build_gpu_throughput_on_workspace(&mut workspace, events, contract)
                .expect("CUDA pipeline (throughput)");
            let dt = t0.elapsed().as_micros();
            std::hint::black_box(case);
            samples_us.push(dt);
        }
    } else {
        let fixtures: Vec<Vec<TraceEvent>> = (0..batch as u64)
            .map(|i| {
                if i == 0 {
                    events.to_vec()
                } else {
                    dsfb_gpu_debug_core::fixture::synthesize(
                        dsfb_gpu_debug_core::fixture::DEFAULT_SEED
                            .wrapping_add(i.wrapping_mul(0x9E37)),
                    )
                }
            })
            .collect();
        let event_slices: Vec<&[TraceEvent]> = fixtures.iter().map(Vec::as_slice).collect();
        let mut workspace =
            BatchedGpuWorkspace::new(batch, contract).expect("batched workspace allocation");
        for _ in 0..warmup {
            let cases = build_gpu_batched_throughput(&mut workspace, &event_slices, contract)
                .expect("CUDA pipeline (batched throughput)");
            std::hint::black_box(cases);
        }
        for _ in 0..iters {
            let t0 = Instant::now();
            let cases = build_gpu_batched_throughput(&mut workspace, &event_slices, contract)
                .expect("CUDA pipeline (batched throughput)");
            let dt = t0.elapsed().as_micros();
            std::hint::black_box(cases);
            samples_us.push(dt);
        }
    }
    samples_us
}

#[cfg(not(feature = "cuda"))]
fn run_layer_b_cpu(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..warmup {
        let _ = std::hint::black_box(build_cpu_throughput(events, contract));
    }
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_cpu_throughput(events, contract);
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us
}

/// CPU Layer B runner: single-catalog throughput case-file build on the
/// host. The headline comparator for the money table — every GPU row at
/// any scale measures speedup against this exact number. Always
/// available regardless of `--features cuda`.
pub(crate) fn run_layer_b_cpu_always(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..warmup {
        let _ = std::hint::black_box(build_cpu_throughput(events, contract));
    }
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_cpu_throughput(events, contract);
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us
}

pub(crate) fn run_layer_c_cpu(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..warmup {
        let _ = std::hint::black_box(build_cpu(events, contract));
    }
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_cpu(events, contract);
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us
}

#[cfg(feature = "cuda")]
pub(crate) fn run_layer_c_gpu(
    events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) -> Vec<u128> {
    let mut workspace = GpuWorkspace::new(contract).expect("workspace allocation");
    let mut samples_us: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..warmup {
        let case = build_gpu_on_workspace(&mut workspace, events, contract)
            .expect("CUDA pipeline (audit)");
        std::hint::black_box(case);
    }
    for _ in 0..iters {
        let t0 = Instant::now();
        let case = build_gpu_on_workspace(&mut workspace, events, contract)
            .expect("CUDA pipeline (audit)");
        let dt = t0.elapsed().as_micros();
        std::hint::black_box(case);
        samples_us.push(dt);
    }
    samples_us
}

/// R.3a — opt-in transcript expansion for a specific catalog index.
///
/// When the caller asks `--materialize-catalog J` alongside a `--layer`
/// run, this routine synthesises catalog `J` deterministically via the
/// courthouse-factory generator and runs a single Layer C audit court
/// over it. The Layer C report writes to
/// `reports/layer_C_materialize_J_<grid>x<windows>_K1.txt` so it does
/// not collide with the Layer C aggregate file from the main run.
///
/// `J = 0` re-runs catalog 0 (which is the bench's primary `events`
/// vector); other indices use the same Knuth-golden-ratio seed
/// derivation as the batched Layer A/B paths.
fn run_materialize_catalog(
    j: u32,
    primary_events: &[TraceEvent],
    contract: &Contract,
    warmup: usize,
    iters: usize,
) {
    let events: Vec<TraceEvent> = if j == 0 {
        primary_events.to_vec()
    } else {
        let derived_seed = dsfb_gpu_debug_core::fixture::DEFAULT_SEED
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            ^ u64::from(j - 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        dsfb_gpu_debug_core::fixture::synthesize_scaled(
            derived_seed,
            contract.n_entities,
            contract.n_windows,
            4,
        )
    };

    println!();
    println!("Materialising catalog J={j} as Layer C transcript on demand (R.3a opt-in)");
    println!("  derived events     : {}", events.len());

    let n_entities = contract.n_entities;
    let n_windows = contract.n_windows;
    let n_detectors = dsfb_gpu_debug_core::motif::MotifClass::COUNT as u32;
    let reports_dir = std::path::Path::new("reports");

    let cpu_samples = run_layer_c_cpu(&events, contract, warmup, iters);
    let cpu_label = format!("Layer C — materialised catalog J={j} (CPU)");
    report_layer(
        &cpu_label,
        &cpu_samples,
        'C',
        n_entities,
        n_windows,
        1,
        n_detectors,
        Some(reports_dir),
        &format!("_materialize_{j}_cpu"),
    );

    #[cfg(feature = "cuda")]
    {
        let gpu_samples = run_layer_c_gpu(&events, contract, warmup, iters);
        let gpu_label = format!("Layer C — materialised catalog J={j} (GPU)");
        report_layer(
            &gpu_label,
            &gpu_samples,
            'C',
            n_entities,
            n_windows,
            1,
            n_detectors,
            Some(reports_dir),
            &format!("_materialize_{j}_gpu"),
        );
    }
}
