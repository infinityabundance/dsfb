#![forbid(unsafe_code)]

//! Phase-A1 multi-seed variance sweep.
//!
//! Runs the controlled TPC-DS perturbation pipeline across a contiguous
//! seed range and reports, per `(motif, metric)`, the mean / stddev /
//! min / max across seeds. The single-seed pinned fingerprint at
//! `seed=42` is not touched — this binary never writes into the
//! fingerprinted paths. It writes to `<out>/variance.csv` only, which is
//! outside the `paper_fingerprint_is_pinned` and
//! `paper_episode_fingerprint_is_pinned` coverage.
//!
//! The reviewer-facing point is simple: any number quoted from the
//! single-seed `reproduce --seed 42` run can be cross-referenced against
//! this sweep's `(mean, stddev)` to see whether it is load-bearing or a
//! seed-dependent accident.

use anyhow::Result;
use clap::Parser;
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::non_claims;
use dsfb_database::perturbation::tpcds_with_perturbations;
use dsfb_database::residual::ResidualStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "variance_sweep",
    about = "Phase-A1: multi-seed variance over the TPC-DS perturbation pipeline.",
    version
)]
struct Cli {
    /// Inclusive lower bound of the seed range (default 1).
    #[arg(long, default_value_t = 1)]
    seed_lo: u64,
    /// Inclusive upper bound of the seed range (default 64).
    #[arg(long, default_value_t = 64)]
    seed_hi: u64,
    /// Output directory. Must be outside the fingerprinted paths.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

fn samples_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        h.insert(m, stream.iter_class(m.residual_class()).count());
    }
    h
}

/// Per-(motif, metric) accumulator. Welford's online variance keeps the
/// sweep O(1) memory in the number of seeds and avoids catastrophic
/// cancellation on tightly-clustered metrics (e.g. F1 ≈ 1.0 across
/// seeds).
#[derive(Clone, Default)]
struct Welford {
    n: u64,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl Welford {
    fn new() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    fn push(&mut self, x: f64) {
        debug_assert!(x.is_finite(), "variance accumulator input must be finite");
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }

    /// Sample stddev (n-1 denominator). Returns 0 for n<=1 — a
    /// single-seed point cannot have variance and we prefer a clean
    /// zero over NaN so CSV consumers do not special-case it.
    fn stddev(&self) -> f64 {
        if self.n <= 1 {
            0.0
        } else {
            (self.m2 / (self.n - 1) as f64).sqrt()
        }
    }
}

/// The metric columns we aggregate. Mirrors `PerMotifMetrics` fields
/// except `motif` (which is the row key) and the integer counts (which
/// we still aggregate as f64 means — stating "mean TP = 3.2" is the
/// right thing for a 64-seed sweep).
const METRICS: &[&str] = &[
    "tp",
    "fp",
    "fn",
    "precision",
    "recall",
    "f1",
    "ttd_median_s",
    "ttd_p95_s",
    "false_alarm_per_hour",
    "compression_ratio",
];

fn metric_value(m: &PerMotifMetrics, name: &str) -> f64 {
    match name {
        "tp" => m.tp as f64,
        "fp" => m.fp as f64,
        "fn" => m.fn_ as f64,
        "precision" => m.precision,
        "recall" => m.recall,
        "f1" => m.f1,
        "ttd_median_s" => m.time_to_detection_median_s,
        "ttd_p95_s" => m.time_to_detection_p95_s,
        "false_alarm_per_hour" => m.false_alarm_rate_per_hour,
        "compression_ratio" => m.episode_compression_ratio,
        other => panic!("unknown metric key: {other}"),
    }
}

fn run_seed(seed: u64) -> Vec<PerMotifMetrics> {
    let (stream, windows) = tpcds_with_perturbations(seed);
    let grammar = MotifGrammar::default();
    let episodes = MotifEngine::new(grammar).run(&stream);
    let samples = samples_per_motif(&stream);
    evaluate(&episodes, &windows, &samples, stream.duration())
}

fn write_variance_csv(
    path: &Path,
    seed_lo: u64,
    seed_hi: u64,
    accum: &HashMap<(String, String), Welford>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif", "metric", "n", "mean", "stddev", "min", "max", "seed_lo", "seed_hi",
    ])?;
    // Deterministic row order: motif in MotifClass::ALL order, metric
    // in METRICS order. A text-diffing reviewer can compare two runs
    // byte-for-byte.
    for m in MotifClass::ALL {
        for metric in METRICS {
            let key = (m.name().to_string(), (*metric).to_string());
            let w = accum
                .get(&key)
                .expect("accumulator populated for every (motif, metric)");
            wtr.write_record([
                m.name(),
                metric,
                &w.n.to_string(),
                &format!("{:.6}", w.mean),
                &format!("{:.6}", w.stddev()),
                &format!("{:.6}", w.min),
                &format!("{:.6}", w.max),
                &seed_lo.to_string(),
                &seed_hi.to_string(),
            ])?;
        }
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();
    anyhow::ensure!(cli.seed_lo <= cli.seed_hi, "--seed-lo must be <= --seed-hi");

    let mut accum: HashMap<(String, String), Welford> = HashMap::new();
    for m in MotifClass::ALL {
        for metric in METRICS {
            accum.insert(
                (m.name().to_string(), (*metric).to_string()),
                Welford::new(),
            );
        }
    }

    for seed in cli.seed_lo..=cli.seed_hi {
        let metrics = run_seed(seed);
        debug_assert_eq!(
            metrics.len(),
            MotifClass::ALL.len(),
            "one metrics row per motif at every seed"
        );
        for row in &metrics {
            for metric in METRICS {
                let key = (row.motif.clone(), (*metric).to_string());
                let w = accum
                    .get_mut(&key)
                    .expect("accumulator present for every (motif, metric)");
                w.push(metric_value(row, metric));
            }
        }
    }

    fs::create_dir_all(&cli.out)?;
    let csv_path = cli.out.join("variance.csv");
    write_variance_csv(&csv_path, cli.seed_lo, cli.seed_hi, &accum)?;
    eprintln!(
        "variance sweep: seeds {}..={}, wrote {}",
        cli.seed_lo,
        cli.seed_hi,
        csv_path.display()
    );
    Ok(())
}
