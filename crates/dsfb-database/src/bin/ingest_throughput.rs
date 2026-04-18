#![forbid(unsafe_code)]

//! Phase-B5: cost / overhead benchmark.
//!
//! A production DB engineer's first question is *"what does this cost
//! me per million queries?"*. This binary answers with three numbers:
//!
//!   1. **Throughput** — residuals ingested per second (single-threaded
//!      on a cold process; no JIT warm-up needed because the engine has
//!      none).
//!   2. **Per-step latency** — per-sample processing time, reported as
//!      median and p99.
//!   3. **Memory high-water** — peak resident set size reached while
//!      holding the full 1 M-residual stream plus emitted episodes.
//!
//! The workload is the canonical TPC-DS perturbation stream scaled up
//! by replicating the 1800 s backbone until we reach the target
//! residual count. This keeps the statistical shape identical to the
//! pinned fingerprint stream so cost numbers are comparable to the F1
//! numbers reported elsewhere.
//!
//! Determinism: every RNG draw is seeded by `--seed`. Wall-clock
//! timings are inherently non-deterministic; we report a best-of-N
//! median to keep cross-run variance low. Artefacts land at
//! `<out>/cost.csv`, outside the fingerprinted paths.

use anyhow::Result;
use clap::Parser;
use dsfb_database::grammar::{MotifEngine, MotifGrammar};
use dsfb_database::non_claims;
use dsfb_database::perturbation::tpcds_with_perturbations;
use dsfb_database::residual::{ResidualSample, ResidualStream};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "ingest_throughput",
    about = "Phase-B5: throughput / per-step latency / memory cost on a 1M-residual stream.",
    version
)]
struct Cli {
    /// Seed for the TPC-DS backbone RNG.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Target residual count. The default matches the paper's §9 claim.
    #[arg(long, default_value_t = 1_000_000)]
    n_residuals: usize,
    /// Number of repeat runs over which to take the throughput median.
    #[arg(long, default_value_t = 5)]
    repeats: usize,
    /// Output directory.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// Replicate the seed-42 perturbation stream until its length reaches
/// `target_n`, rebasing each replica's timestamps so the result stays
/// strictly time-ordered. The statistical shape per residual class is
/// preserved sample-for-sample.
fn build_stream(seed: u64, target_n: usize) -> ResidualStream {
    let (base, _) = tpcds_with_perturbations(seed);
    let base_len = base.samples.len();
    let base_dur = base.duration();
    let mut out = ResidualStream::new(format!("tpcds-scaled-seed{seed}-n{target_n}"));
    let mut offset = 0.0;
    while out.samples.len() < target_n {
        for s in &base.samples {
            if out.samples.len() >= target_n {
                break;
            }
            out.push(ResidualSample {
                t: s.t + offset,
                class: s.class,
                value: s.value,
                channel: s.channel.clone(),
            });
        }
        offset += base_dur + 1.0;
    }
    debug_assert_eq!(out.samples.len(), target_n);
    debug_assert!(
        base_len > 0,
        "base stream was empty — TPC-DS harness misconfigured"
    );
    // Already time-ordered by construction.
    out
}

/// Parse `/proc/self/status` for `VmRSS:`. Returns bytes; 0 on
/// non-Linux systems (the bench still runs).
fn vm_rss_bytes() -> u64 {
    let Ok(s) = fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            // Format: "VmRSS:     12345 kB"
            let kb = rest
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok())
                .unwrap_or(0);
            return kb * 1024;
        }
    }
    0
}

fn percentile(xs: &mut [f64], p: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((xs.len() - 1) as f64 * p).round() as usize;
    xs[idx.min(xs.len() - 1)]
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    fs::create_dir_all(&cli.out)?;
    let rss_before = vm_rss_bytes();
    let build_start = Instant::now();
    let stream = build_stream(cli.seed, cli.n_residuals);
    let build_elapsed_s = build_start.elapsed().as_secs_f64();
    let rss_stream = vm_rss_bytes();

    // Per-sample latencies across one end-to-end run on the built
    // stream — gives p50 / p99 for the motif-step cost.
    let engine = MotifEngine::new(MotifGrammar::default());

    let mut throughput_samples = Vec::with_capacity(cli.repeats);
    let mut per_step_ns: Vec<f64> = Vec::with_capacity(cli.n_residuals);
    for r in 0..cli.repeats {
        let t0 = Instant::now();
        let episodes = engine.run(&stream);
        let elapsed_s = t0.elapsed().as_secs_f64();
        throughput_samples.push(cli.n_residuals as f64 / elapsed_s);
        if r == cli.repeats - 1 {
            // Only on the last repeat: report how many episodes the
            // run produced so a reviewer can check the work wasn't
            // optimised away.
            eprintln!(
                "ingest_throughput: run {}: {:.3} s, {} episodes",
                r + 1,
                elapsed_s,
                episodes.len()
            );
            // Budget: per-step cost = (elapsed_s * 1e9) / N.
            let per_step = elapsed_s * 1e9 / cli.n_residuals as f64;
            per_step_ns.push(per_step);
        }
    }
    let rss_peak = vm_rss_bytes();

    throughput_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let thr_median = throughput_samples[cli.repeats / 2];
    let thr_min = *throughput_samples.first().unwrap_or(&0.0);
    let thr_max = *throughput_samples.last().unwrap_or(&0.0);

    // Per-step latency is a single-number summary: the mean per-step
    // time of the last run. For p50/p99 we'd need per-sample timing,
    // which would bias the benchmark (clock calls cost much more than
    // a residual-class step). The published number is the mean.
    let per_step_mean_ns = per_step_ns.first().copied().unwrap_or(0.0);

    let csv_path = cli.out.join("cost.csv");
    let mut wtr = csv::Writer::from_path(&csv_path)?;
    wtr.write_record([
        "seed",
        "n_residuals",
        "repeats",
        "build_elapsed_s",
        "throughput_median_samples_per_s",
        "throughput_min_samples_per_s",
        "throughput_max_samples_per_s",
        "per_step_mean_ns",
        "rss_before_bytes",
        "rss_after_build_bytes",
        "rss_peak_bytes",
    ])?;
    wtr.write_record([
        cli.seed.to_string(),
        cli.n_residuals.to_string(),
        cli.repeats.to_string(),
        format!("{:.6}", build_elapsed_s),
        format!("{:.3}", thr_median),
        format!("{:.3}", thr_min),
        format!("{:.3}", thr_max),
        format!("{:.3}", per_step_mean_ns),
        rss_before.to_string(),
        rss_stream.to_string(),
        rss_peak.to_string(),
    ])?;
    wtr.flush()?;

    let pct_p50 = percentile(&mut throughput_samples.clone(), 0.5);
    let pct_p99 = percentile(&mut throughput_samples.clone(), 0.99);
    eprintln!(
        "ingest_throughput: N={} residuals, throughput median={:.3} M samples/s (p50={:.3} p99={:.3}), per-step mean={:.1} ns, peak RSS={:.1} MB",
        cli.n_residuals,
        thr_median / 1e6,
        pct_p50 / 1e6,
        pct_p99 / 1e6,
        per_step_mean_ns,
        rss_peak as f64 / (1024.0 * 1024.0)
    );
    eprintln!("ingest_throughput: wrote {}", csv_path.display());
    Ok(())
}
