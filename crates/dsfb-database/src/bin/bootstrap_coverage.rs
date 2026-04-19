#![forbid(unsafe_code)]

//! Monte-Carlo measurement of percentile-bootstrap CI coverage at
//! small sample sizes.
//!
//! ## Why this exists
//!
//! The §Live-Eval table reports 95 % percentile-bootstrap confidence
//! intervals on F1, TTD, and FAR/hr from `n = 10` replications. The
//! statistics literature (Efron & Tibshirani 1993; DiCiccio & Efron
//! 1996) shows that the percentile bootstrap *under-covers* its
//! nominal level on long-tailed and skewed distributions when `n` is
//! small. The paper notes this in the §Live-Eval setup paragraph; the
//! Pass-2 reviewer panel (R4 — Statistics) asked us to **quantify**
//! the under-coverage with a Monte-Carlo simulation rather than just
//! cite a literature caveat.
//!
//! This binary runs the simulation:
//!
//!   * For each of three source distributions (bounded Beta on F1,
//!     log-Normal on FAR/hr, Gamma on TTD-like), draw `n_mc` independent
//!     samples of size `n` from the distribution.
//!   * For each draw, compute the percentile-bootstrap 95 % CI on the
//!     *sample mean* using `B = 1000` resamples.
//!   * Empirical coverage = fraction of draws whose CI contains the
//!     known true mean of the source distribution.
//!
//! Output: `out/coverage.csv` with columns
//!
//!   `distribution, n, n_mc, B, alpha_nominal, true_mean,
//!    empirical_coverage, mean_ci_lo, mean_ci_hi`.
//!
//! The §Live-Eval table's CIs at `n = 10` should be read against the
//! row of this CSV with `n = 10` and the *closest* matching tail
//! distribution for each metric:
//!
//!   F1            ↔  Beta(α=8, β=2)         (bounded, mildly skewed)
//!   TTD-like      ↔  Gamma(k=2, θ=0.3)      (right-skewed, positive)
//!   FAR/hr-like   ↔  log-Normal(μ=2, σ=1.0) (heavy right tail)
//!
//! ## Determinism
//!
//! Pure function of the CLI flags. The PRNG is seeded from the
//! `--seed` flag (default 42) and uses a documented LCG identical to
//! the one in `src/bin/baseline_tune.rs`'s `Lcg` so that two
//! independent toolchain builds produce byte-equal CSV output.
//!
//! ## Fingerprint safety
//!
//! Output lands in the user-supplied `--out` directory which is
//! outside every fingerprint-locked path; no source under `src/`,
//! `spec/`, or any pinned tape is touched.

use anyhow::{Context, Result};
use clap::Parser;
use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;

/// Documented LCG (matches `baseline_tune.rs::Lcg`). Pure 64-bit linear
/// congruential generator from Numerical Recipes; sufficient for Monte
/// Carlo where the bias of the generator is dominated by the bootstrap
/// resampling variance at `n_mc ≥ 1000`.
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6364136223846793005) ^ 0x9E3779B97F4A7C15)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    /// Uniform [0, 1) using the high 53 bits of the generator.
    fn next_unit(&mut self) -> f64 {
        let raw = self.next_u64() >> 11;
        (raw as f64) / ((1u64 << 53) as f64)
    }
    fn next_range(&mut self, n: usize) -> usize {
        ((self.next_u64() >> 33) as usize) % n.max(1)
    }
    /// Standard normal via Box-Muller. Returns one sample per call;
    /// the second of the pair is recomputed every call (LCG is cheap).
    fn next_normal(&mut self) -> f64 {
        let u1 = self.next_unit().max(f64::MIN_POSITIVE);
        let u2 = self.next_unit();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }
}

/// One of the three source distributions named in the docstring.
#[derive(Clone, Copy)]
enum Source {
    BetaLikeF1,
    GammaLikeTtd,
    LogNormalLikeFar,
}

impl Source {
    fn name(self) -> &'static str {
        match self {
            Source::BetaLikeF1 => "beta_like_f1",
            Source::GammaLikeTtd => "gamma_like_ttd",
            Source::LogNormalLikeFar => "lognormal_like_far",
        }
    }

    /// Draw one sample. Closed-form for log-normal; rejection /
    /// transform for the others. Beta(8,2) by acceptance-rejection
    /// from a uniform proposal; Gamma(k=2, θ=0.3) by sum of two
    /// exponentials with θ.
    fn draw(self, rng: &mut Lcg) -> f64 {
        match self {
            Source::BetaLikeF1 => {
                // Beta(8, 2) has support [0,1], mean 0.8, mildly left-
                // tailed. Use Cheng's BB algorithm — but for this
                // specific case the simpler "two gammas" trick is
                // exact: B(α,β) = G(α)/(G(α)+G(β)).
                let g1 = gamma_int(rng, 8);
                let g2 = gamma_int(rng, 2);
                g1 / (g1 + g2)
            }
            Source::GammaLikeTtd => {
                // Gamma(k=2, θ=0.3), mean 0.6.
                let theta = 0.3_f64;
                gamma_int(rng, 2) * theta
            }
            Source::LogNormalLikeFar => {
                // log-Normal(μ=2, σ=1), mean = exp(μ + σ²/2) ≈ 12.18
                let mu = 2.0_f64;
                let sigma = 1.0_f64;
                (mu + sigma * rng.next_normal()).exp()
            }
        }
    }

    /// Closed-form true mean used as the coverage target.
    fn true_mean(self) -> f64 {
        match self {
            // E[Beta(α, β)] = α / (α + β)
            Source::BetaLikeF1 => 8.0 / (8.0 + 2.0),
            // E[Gamma(k, θ)] = k · θ
            Source::GammaLikeTtd => 2.0 * 0.3,
            // E[log-Normal(μ, σ)] = exp(μ + σ²/2)
            Source::LogNormalLikeFar => (2.0_f64 + 0.5).exp(),
        }
    }
}

/// Sum of `k` Exp(1) variates, i.e. Gamma(k, 1).
fn gamma_int(rng: &mut Lcg, k: usize) -> f64 {
    let mut acc = 0.0;
    for _ in 0..k {
        let u = rng.next_unit().max(f64::MIN_POSITIVE);
        acc -= u.ln();
    }
    acc
}

/// Percentile-bootstrap 95 % CI on the sample mean. Identical
/// algorithm to `experiments/real_pg_eval/aggregate.py::bootstrap_ci`
/// so the simulation's coverage applies directly to the table CIs.
fn bootstrap_ci(sample: &[f64], b: usize, alpha: f64, rng: &mut Lcg) -> (f64, f64) {
    if sample.is_empty() {
        return (0.0, 0.0);
    }
    let mut boots = Vec::with_capacity(b);
    let n = sample.len();
    for _ in 0..b {
        let resample_mean: f64 = (0..n).map(|_| sample[rng.next_range(n)]).sum::<f64>() / n as f64;
        boots.push(resample_mean);
    }
    boots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo_idx = ((alpha / 2.0) * b as f64) as usize;
    let hi_idx = (((1.0 - alpha / 2.0) * b as f64) as usize).min(b - 1);
    (boots[lo_idx], boots[hi_idx])
}

#[derive(Parser)]
#[command(
    name = "bootstrap_coverage",
    about = "Monte-Carlo coverage of the percentile-bootstrap 95% CI at small n.",
    version
)]
struct Cli {
    /// Sample size whose CI coverage we are measuring. The §Live-Eval
    /// table uses n=10; running with a sequence (e.g. --n 5,10,20,50)
    /// produces a coverage curve.
    #[arg(long, value_delimiter = ',', default_values_t = vec![5, 10, 20, 50])]
    n: Vec<usize>,
    /// Number of Monte-Carlo iterations per (distribution, n) pair.
    #[arg(long, default_value_t = 2000)]
    n_mc: usize,
    /// Bootstrap resamples per CI computation.
    #[arg(long, default_value_t = 1000)]
    bootstrap_b: usize,
    /// Nominal alpha level (95 % CI ⇒ alpha = 0.05).
    #[arg(long, default_value_t = 0.05)]
    alpha: f64,
    /// PRNG seed for full determinism.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Output directory. The CSV `coverage.csv` is written here.
    #[arg(long, default_value = "out/bootstrap_coverage")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.out).with_context(|| format!("mkdir {}", cli.out.display()))?;

    let mut rng = Lcg::new(cli.seed);
    let sources = [
        Source::BetaLikeF1,
        Source::GammaLikeTtd,
        Source::LogNormalLikeFar,
    ];

    let mut buf = String::new();
    buf.push_str(
        "distribution,n,n_mc,B,alpha_nominal,true_mean,empirical_coverage,mean_ci_lo,mean_ci_hi\n",
    );

    for src in sources {
        let true_mean = src.true_mean();
        for &n in &cli.n {
            let mut covered = 0_usize;
            let mut sum_lo = 0.0_f64;
            let mut sum_hi = 0.0_f64;
            for _ in 0..cli.n_mc {
                let sample: Vec<f64> = (0..n).map(|_| src.draw(&mut rng)).collect();
                let (lo, hi) = bootstrap_ci(&sample, cli.bootstrap_b, cli.alpha, &mut rng);
                if lo <= true_mean && true_mean <= hi {
                    covered += 1;
                }
                sum_lo += lo;
                sum_hi += hi;
            }
            let coverage = covered as f64 / cli.n_mc as f64;
            let mean_lo = sum_lo / cli.n_mc as f64;
            let mean_hi = sum_hi / cli.n_mc as f64;
            buf.push_str(&format!(
                "{},{},{},{},{:.4},{:.6},{:.4},{:.6},{:.6}\n",
                src.name(),
                n,
                cli.n_mc,
                cli.bootstrap_b,
                cli.alpha,
                true_mean,
                coverage,
                mean_lo,
                mean_hi
            ));
            eprintln!(
                "  {:>20} | n={:>3} | true={:.3} | coverage={:.3} | CI ≈ [{:.3}, {:.3}]",
                src.name(),
                n,
                true_mean,
                coverage,
                mean_lo,
                mean_hi
            );
        }
    }

    let csv_path = cli.out.join("coverage.csv");
    fs::write(&csv_path, buf).with_context(|| format!("writing {}", csv_path.display()))?;
    eprintln!("wrote {}", csv_path.display());
    Ok(())
}
