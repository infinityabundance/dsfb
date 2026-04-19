#![forbid(unsafe_code)]

//! Public-trace bake-off: false-alarm-per-hour bound on the four
//! publicly-cited real-workload shapes (Snowset, SQLShare, CEB, JOB)
//! and the TPC-DS reference set.
//!
//! These traces carry no fault annotations, so every emitted episode
//! is counted as a false alarm by construction. The resulting FAR/hr
//! is a *workload-stress upper bound* on the detector's false-alarm
//! rate — useful for operator capacity planning. It is explicitly
//! **not** a detection-quality claim; detection quality lives in
//! `paper/tables/live_eval_mean_ci.tex` and
//! `paper/tables/baseline_tuned.tex` on the planted-fault protocol.
//!
//! Per seed s ∈ 1..=`--seeds`:
//!   * each adapter's [`DatasetAdapter::exemplar(s)`] produces a
//!     deterministic residual stream with the shape of the real
//!     corpus (the corpora themselves are third-party-licensed /
//!     permission-gated and live outside the crate; the paper's
//!     §3 makes the exemplar-vs-real distinction explicit).
//!   * each detector runs on that stream.
//!   * FAR/hr is episode_count / (duration_seconds / 3600).
//!
//! Output: `<out>/public_trace_far.csv` (one row per
//! detector × dataset, aggregated across seeds) and
//! `<out>/public_trace_far_per_seed.csv` (fully raw).
//!
//! Fingerprint safety: outputs land in the user-supplied `--out`
//! directory, outside every fingerprint-locked path.

use anyhow::Result;
use clap::Parser;
use dsfb_database::adapters::{ceb::Ceb, job::Job, snowset::Snowset, sqlshare::SqlShare, tpcds::TpcDs, DatasetAdapter};
use dsfb_database::baselines::{
    adwin::Adwin, bocpd::Bocpd, pelt::Pelt, run_detector, ChangePointDetector,
};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::residual::ResidualStream;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "public_trace_bakeoff",
    about = "False-alarm-per-hour upper bound on public traces (Snowset, SQLShare, CEB, JOB, TPC-DS).",
    version
)]
struct Cli {
    /// Number of exemplar seeds to run per adapter. Seeds are 1..=seeds.
    #[arg(long, default_value_t = 10)]
    seeds: u64,
    /// Output directory for per-dataset FAR/hr CSVs.
    #[arg(long, default_value = "out/public_trace")]
    out: PathBuf,
}

fn adapters() -> Vec<Box<dyn DatasetAdapter>> {
    vec![
        Box::new(Snowset),
        Box::new(SqlShare),
        Box::new(Ceb),
        Box::new(Job),
        Box::new(TpcDs),
    ]
}

fn count_dsfb_episodes(stream: &ResidualStream) -> usize {
    let engine = MotifEngine::new(MotifGrammar::default());
    engine.run(stream).len()
}

fn count_detector_episodes(det: &dyn ChangePointDetector, stream: &ResidualStream) -> usize {
    let mut total = 0usize;
    for m in MotifClass::ALL {
        total += run_detector(det, m, stream).len();
    }
    total
}

fn far_per_hour(n_episodes: usize, duration_s: f64) -> f64 {
    if duration_s <= 0.0 {
        return 0.0;
    }
    (n_episodes as f64) * 3600.0 / duration_s
}

struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6364136223846793005) ^ 0x9E3779B97F4A7C15)
    }
    fn next_range(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n.max(1)
    }
}

fn bootstrap_ci(vs: &[f64]) -> (f64, f64, f64) {
    if vs.len() < 2 {
        let v = vs.first().copied().unwrap_or(0.0);
        return (v, v, v);
    }
    let b = 1000usize;
    let alpha = 0.05f64;
    let mut lcg = Lcg::new(42);
    let mut boots = Vec::with_capacity(b);
    for _ in 0..b {
        let s: f64 = (0..vs.len())
            .map(|_| vs[lcg.next_range(vs.len())])
            .sum::<f64>()
            / vs.len() as f64;
        boots.push(s);
    }
    boots.sort_by(|a, bv| a.partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal));
    let mean = vs.iter().sum::<f64>() / vs.len() as f64;
    let lo = boots[(alpha / 2.0 * b as f64) as usize];
    let hi = boots[((1.0 - alpha / 2.0) * b as f64) as usize];
    (mean, lo, hi)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.out)?;

    let detectors: Vec<(&'static str, Box<dyn ChangePointDetector>)> = vec![
        ("adwin", Box::new(Adwin::default())),
        ("bocpd", Box::new(Bocpd::default())),
        ("pelt", Box::new(Pelt::default())),
    ];
    let ads = adapters();

    let mut per_seed = String::new();
    per_seed.push_str("detector,dataset,seed,n_episodes,duration_s,far_per_hour\n");
    let mut by_key: std::collections::BTreeMap<(String, String), Vec<f64>> =
        std::collections::BTreeMap::new();

    for seed in 1..=cli.seeds {
        for a in &ads {
            let stream = a.exemplar(seed);
            let dur = stream.duration();

            let dsfb_eps = count_dsfb_episodes(&stream);
            let dsfb_far = far_per_hour(dsfb_eps, dur);
            per_seed.push_str(&format!(
                "dsfb-database,{},{},{},{:.3},{:.3}\n",
                a.name(), seed, dsfb_eps, dur, dsfb_far
            ));
            by_key
                .entry(("dsfb-database".to_string(), a.name().to_string()))
                .or_default()
                .push(dsfb_far);

            for (label, det) in &detectors {
                let eps = count_detector_episodes(det.as_ref(), &stream);
                let far = far_per_hour(eps, dur);
                per_seed.push_str(&format!(
                    "{},{},{},{},{:.3},{:.3}\n",
                    label, a.name(), seed, eps, dur, far
                ));
                by_key
                    .entry((label.to_string(), a.name().to_string()))
                    .or_default()
                    .push(far);
            }
        }
    }

    let per_seed_path = cli.out.join("public_trace_far_per_seed.csv");
    fs::write(&per_seed_path, &per_seed)?;
    eprintln!("wrote {}", per_seed_path.display());

    let mut agg = String::new();
    agg.push_str("detector,dataset,n_seeds,far_per_hour_mean,far_per_hour_ci95_lo,far_per_hour_ci95_hi\n");
    for ((det, ds), vs) in &by_key {
        let (m, lo, hi) = bootstrap_ci(vs);
        agg.push_str(&format!(
            "{},{},{},{:.3},{:.3},{:.3}\n",
            det, ds, vs.len(), m, lo, hi
        ));
    }
    let agg_path = cli.out.join("public_trace_far.csv");
    fs::write(&agg_path, &agg)?;
    eprintln!("wrote {}", agg_path.display());

    for ((det, ds), vs) in &by_key {
        let (m, lo, hi) = bootstrap_ci(vs);
        eprintln!(
            "  {:>14} | {:>16} | FAR/hr {:>8.1} [{:>7.1}, {:>7.1}] (n={})",
            det, ds, m, lo, hi, vs.len()
        );
    }
    Ok(())
}
