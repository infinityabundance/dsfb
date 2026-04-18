#![forbid(unsafe_code)]

//! Phase-A3 false-alarm calibration over a quiet null trace.
//!
//! A reviewer's natural question — "when nothing is happening, how often
//! does the motif grammar cry wolf?" — gets a direct number here.
//! We build a deterministic residual stream containing only Gaussian
//! measurement noise at a per-class sigma matched to the TPC-DS backbone's
//! pre-perturbation variance (see `perturbation::tpcds_with_perturbations`
//! for the reference values). Any episode the grammar opens on this stream
//! is, by construction, a false alarm.
//!
//! We sweep a contiguous seed range so the reported rate has both a mean
//! and a confidence interval — a single-seed null-trace number would be
//! as brittle as the single-seed F1 numbers the variance sweep already
//! dispatches.
//!
//! Fingerprint safety: the null-trace stream has a distinct source string
//! (`null-trace-...`) and is written to `<out>/null.csv`, outside the
//! `paper_fingerprint_is_pinned` / `paper_episode_fingerprint_is_pinned`
//! coverage. No pinned artefact is touched.

use anyhow::Result;
use clap::Parser;
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::non_claims;
use dsfb_database::residual::{ResidualClass, ResidualSample, ResidualStream};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "null_trace",
    about = "Phase-A3: false-alarm calibration on a quiet Gaussian null trace.",
    version
)]
struct Cli {
    /// Inclusive lower bound of the seed range. A range rather than a
    /// single seed so the output carries a confidence interval, not just
    /// a point estimate.
    #[arg(long, default_value_t = 1)]
    seed_lo: u64,
    /// Inclusive upper bound of the seed range.
    #[arg(long, default_value_t = 32)]
    seed_hi: u64,
    /// Duration of each per-seed null trace in seconds. Default 3600 = 1
    /// hour, which makes `false_alarms_per_hour` trivially readable
    /// (alarms == per-hour rate).
    #[arg(long, default_value_t = 3600.0)]
    duration_s: f64,
    /// Sample rate (Hz) per residual class. 1 Hz matches the TPC-DS
    /// harness cadence.
    #[arg(long, default_value_t = 1.0)]
    rate_hz: f64,
    /// Output directory. Must be outside the fingerprinted paths.
    #[arg(long, default_value = "out")]
    out: PathBuf,
    /// Multiplicative scale applied to every per-class quiet sigma. 1.0
    /// is the calibrated null trace; values >1 sanity-check that the
    /// grammar *does* fire when noise is loud enough, values <1 confirm
    /// the margin under the envelope. Reported in the CSV alongside
    /// `sigma_quiet`. Does not change the RNG trajectory for a given
    /// seed — draws are Gaussian, scale is applied post-draw.
    #[arg(long, default_value_t = 1.0)]
    sigma_scale: f64,
}

/// Per-class quiet-regime sigma, chosen to match the noise floor of the
/// canonical TPC-DS backbone (see `perturbation::tpcds_with_perturbations`
/// for the generator that sets these floors):
/// * PlanRegression: latency is `(actual − baseline) / baseline`; backbone
///   injects ±2 ms on a 50 ms baseline, giving an empirical σ ≈ 0.023. We
///   round up to 0.03 to stay honest about the jitter envelope.
/// * Cardinality: `log10(actual/estimated)` with backbone ±8 % noise gives
///   σ ≈ 0.033. Rounded to 0.03 for the same reason.
/// * Contention: the backbone has no lock traffic, so σ = 0 in practice.
///   We seed with a deliberately tiny 0.005 s floor to keep the motif's
///   EMA/trust machinery exercised.
/// * CacheIo: hit-ratio drop with backbone ±0.005 → σ ≈ 0.003.
/// * WorkloadPhase: JSD floor when distributions agree is 0; we inject
///   0.01 as a below-threshold stand-in (drift_threshold is 0.15).
///
/// The constants live here rather than in `MotifParams` because they
/// describe the *stream*, not the envelope. Changing them changes the
/// reported calibration, so they are documented in-prose in paper §6.3
/// (null-trace sanity).
fn sigma_for(class: ResidualClass) -> f64 {
    match class {
        ResidualClass::PlanRegression => 0.03,
        ResidualClass::Cardinality => 0.03,
        ResidualClass::Contention => 0.005,
        ResidualClass::CacheIo => 0.003,
        ResidualClass::WorkloadPhase => 0.01,
    }
}

/// Box–Muller from two U(0,1) samples. Using the Marsaglia polar form
/// would save a `sin`/`cos`; the Box–Muller path is deterministic,
/// branch-free, and fast enough for 3600 × 5 × 32 ≈ 6 × 10⁵ draws.
fn gauss_pair<R: Rng>(rng: &mut R) -> (f64, f64) {
    let u1: f64 = rng.gen_range(f64::EPSILON..1.0);
    let u2: f64 = rng.gen_range(0.0..1.0);
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    (r * theta.cos(), r * theta.sin())
}

fn null_stream(seed: u64, duration_s: f64, rate_hz: f64, sigma_scale: f64) -> ResidualStream {
    debug_assert!(duration_s > 0.0 && rate_hz > 0.0 && sigma_scale > 0.0);
    let mut stream = ResidualStream::new(format!(
        "null-trace-seed{seed}-dur{:.0}s-rate{:.2}hz-sigma{:.3}",
        duration_s, rate_hz, sigma_scale
    ));
    let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
    let n = (duration_s * rate_hz).round() as u64;
    let dt = 1.0 / rate_hz;
    // Draw all Gaussian samples in class-blocks so the RNG trajectory is
    // stable across `n` / `duration_s` tweaks for a given (seed, class).
    for class in ResidualClass::ALL {
        let sigma = sigma_for(class) * sigma_scale;
        let channel = format!("null_{}", class.name());
        let mut i = 0u64;
        while i < n {
            let (g1, g2) = gauss_pair(&mut rng);
            let t0 = i as f64 * dt;
            stream.push(ResidualSample::new(t0, class, sigma * g1).with_channel(channel.clone()));
            if i + 1 < n {
                let t1 = (i + 1) as f64 * dt;
                stream
                    .push(ResidualSample::new(t1, class, sigma * g2).with_channel(channel.clone()));
            }
            i += 2;
        }
    }
    stream.sort();
    stream
}

/// Count episodes per motif. The null stream has no ground-truth windows,
/// so every opened episode is by definition a false alarm.
fn false_alarms_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let grammar = MotifGrammar::default();
    let eps = MotifEngine::new(grammar).run(stream);
    let mut counts: HashMap<MotifClass, usize> = MotifClass::ALL.iter().map(|m| (*m, 0)).collect();
    for e in &eps {
        *counts.entry(e.motif).or_insert(0) += 1;
    }
    counts
}

/// Welford mirror of the variance_sweep binary — kept local (trivial
/// struct) rather than promoted into the library, per the project's
/// "no premature abstraction" norm.
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
        debug_assert!(x.is_finite());
        self.n += 1;
        let d = x - self.mean;
        self.mean += d / self.n as f64;
        let d2 = x - self.mean;
        self.m2 += d * d2;
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }
    fn stddev(&self) -> f64 {
        if self.n <= 1 {
            0.0
        } else {
            (self.m2 / (self.n - 1) as f64).sqrt()
        }
    }
    /// 95 % Student-t is the right interval for small seed counts, but the
    /// added dependency is not worth it — for n ≥ 20 the normal-z (1.96)
    /// is within 3 % of the t-interval, and we call out the
    /// approximation in paper §6.3. We clamp the lower bound at 0 because
    /// a rate cannot go negative.
    fn ci95(&self) -> (f64, f64) {
        if self.n == 0 {
            return (0.0, 0.0);
        }
        let se = self.stddev() / (self.n as f64).sqrt();
        let half = 1.96 * se;
        ((self.mean - half).max(0.0), self.mean + half)
    }
}

fn write_null_csv(path: &Path, cli: &Cli, accum: &HashMap<MotifClass, Welford>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif",
        "n_seeds",
        "duration_s",
        "rate_hz",
        "sigma_scale",
        "mean_false_alarms_per_hour",
        "stddev_false_alarms_per_hour",
        "min_per_hour",
        "max_per_hour",
        "ci95_lo_per_hour",
        "ci95_hi_per_hour",
        "sigma_quiet",
        "seed_lo",
        "seed_hi",
    ])?;
    for m in MotifClass::ALL {
        let w = accum
            .get(&m)
            .expect("accumulator populated for every motif");
        let (lo, hi) = w.ci95();
        let sigma = sigma_for(m.residual_class()) * cli.sigma_scale;
        wtr.write_record([
            m.name(),
            &w.n.to_string(),
            &format!("{:.3}", cli.duration_s),
            &format!("{:.3}", cli.rate_hz),
            &format!("{:.3}", cli.sigma_scale),
            &format!("{:.6}", w.mean),
            &format!("{:.6}", w.stddev()),
            &format!("{:.6}", w.min),
            &format!("{:.6}", w.max),
            &format!("{:.6}", lo),
            &format!("{:.6}", hi),
            &format!("{:.6}", sigma),
            &cli.seed_lo.to_string(),
            &cli.seed_hi.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();
    anyhow::ensure!(cli.seed_lo <= cli.seed_hi, "--seed-lo must be <= --seed-hi");
    anyhow::ensure!(cli.duration_s > 0.0, "--duration-s must be > 0");
    anyhow::ensure!(cli.rate_hz > 0.0, "--rate-hz must be > 0");
    anyhow::ensure!(cli.sigma_scale > 0.0, "--sigma-scale must be > 0");

    let mut accum: HashMap<MotifClass, Welford> = MotifClass::ALL
        .iter()
        .map(|m| (*m, Welford::new()))
        .collect();

    let hours = cli.duration_s / 3600.0;
    for seed in cli.seed_lo..=cli.seed_hi {
        let stream = null_stream(seed, cli.duration_s, cli.rate_hz, cli.sigma_scale);
        let counts = false_alarms_per_motif(&stream);
        for m in MotifClass::ALL {
            let count = *counts.get(&m).unwrap_or(&0) as f64;
            accum.get_mut(&m).unwrap().push(count / hours);
        }
    }

    fs::create_dir_all(&cli.out)?;
    let csv_path = cli.out.join("null.csv");
    write_null_csv(&csv_path, &cli, &accum)?;
    eprintln!(
        "null_trace: seeds {}..={}, duration {:.1} s @ {:.2} Hz, sigma_scale {:.3}, wrote {}",
        cli.seed_lo,
        cli.seed_hi,
        cli.duration_s,
        cli.rate_hz,
        cli.sigma_scale,
        csv_path.display()
    );
    Ok(())
}
