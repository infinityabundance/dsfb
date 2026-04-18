#![forbid(unsafe_code)]

//! Phase-B1: synthetic injection over a real(-shaped) base trace.
//!
//! A reviewer's next question after the TPC-DS perturbation bake-off is
//! *"does the motif grammar also catch injected perturbations when the
//! carrier stream comes from a different corpus?"* This binary answers
//! that by overlaying a single parametric perturbation window on top of
//! each dataset's real-adapter exemplar (Snowset, SQLShare, CEB, JOB)
//! and reporting per-(dataset, motif, magnitude) detection latency and
//! onset-localization error against the injected ground truth.
//!
//! Outputs land at `<out>/inject_over_real.csv`, far outside the
//! fingerprinted paths the four paper locks guard.
//!
//! Determinism: every sample in the base and every RNG draw for the
//! injection is a pure function of the pinned seed; re-running produces
//! a byte-identical CSV.
//!
//! Non-claim alignment: this binary does **not** validate detection on
//! real workloads — non-claim #4 is unchanged. It measures detection on
//! a *synthetically-injected window over a real-shaped carrier*. The
//! CSV `carrier` column documents exactly that.

use anyhow::Result;
use clap::Parser;
use dsfb_database::adapters::{
    ceb::Ceb, job::Job, snowset::Snowset, sqlshare::SqlShare, DatasetAdapter,
};
use dsfb_database::grammar::{Episode, MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::non_claims;
use dsfb_database::residual::{
    cache_io, cardinality, contention, plan_regression, workload_phase, ResidualStream,
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "inject_over_real",
    about = "Phase-B1: overlay one parametric perturbation on each adapter's real-shaped exemplar and measure detection latency / localization error.",
    version
)]
struct Cli {
    /// RNG seed for the exemplar generators. Held fixed across the
    /// grid so the only varying factor across rows is
    /// (dataset, motif, scale).
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Injection onset in seconds after the start of the base stream.
    /// Chosen to sit inside every adapter's exemplar range while
    /// leaving enough pre-window EMA history.
    #[arg(long, default_value_t = 400.0)]
    onset_s: f64,
    /// Injection duration in seconds.
    #[arg(long, default_value_t = 120.0)]
    duration_s: f64,
    /// Output directory.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// Append a single parametric perturbation window to `stream` for the
/// chosen motif class. The magnitudes mirror the TPC-DS harness so a
/// reviewer can diff the two binaries side-by-side.
fn inject(
    stream: &mut ResidualStream,
    motif: MotifClass,
    onset_s: f64,
    duration_s: f64,
    scale: f64,
) {
    let channel = "inj";
    let end = onset_s + duration_s;
    match motif {
        MotifClass::PlanRegressionOnset => {
            // 50 ms baseline → 50 + 250·scale ms sustained (no jitter; the
            // exemplar carrier already supplies realistic noise).
            let mut t = onset_s;
            while t < end {
                plan_regression::push_latency(stream, t, channel, 50.0 + 250.0 * scale, 50.0);
                t += 1.0;
            }
        }
        MotifClass::CardinalityMismatchRegime => {
            // est/actual = 1 + 29·scale, matching the TPC-DS q17 window.
            let mut t = onset_s;
            while t < end {
                let true_rows = 30_000.0;
                let est_rows = true_rows / (1.0 + 29.0 * scale);
                cardinality::push(stream, t, channel, est_rows, true_rows);
                t += 1.0;
            }
        }
        MotifClass::ContentionRamp => {
            // Linear wait ramp 0.05 → 0.05 + 1.5·scale seconds.
            let mut t = onset_s;
            while t < end {
                let progress = (t - onset_s) / duration_s;
                let wait_s = 0.05 + 1.5 * scale * progress;
                contention::push_wait(stream, t, channel, wait_s);
                let depth = (1.0 + 4.0 * scale * progress) as usize;
                contention::push_chain_depth(stream, t, channel, depth);
                t += 1.0;
            }
        }
        MotifClass::CacheCollapse => {
            // Hit-ratio drop 0.95 → 0.95 − 0.45·scale.
            let mut t = onset_s;
            while t < end {
                cache_io::push_hit_ratio(stream, t, channel, 0.95, 0.95 - 0.45 * scale);
                t += 1.0;
            }
        }
        MotifClass::WorkloadPhaseTransition => {
            // JSD elevated to 0.05 + 0.4·scale, 30-s bucket cadence.
            let bucket = 30.0;
            let mut t = onset_s;
            while t < end {
                workload_phase::push_jsd(stream, t, channel, 0.05 + 0.4 * scale);
                t += bucket;
            }
        }
    }
}

/// Find the earliest episode of `motif` that overlaps `[onset, onset+duration]`.
/// `None` means the motif emitted nothing overlapping the window.
fn first_matching(
    episodes: &[Episode],
    motif: MotifClass,
    onset: f64,
    dur: f64,
) -> Option<&Episode> {
    let end = onset + dur;
    episodes
        .iter()
        .filter(|e| e.motif == motif && e.t_end >= onset && e.t_start <= end)
        .min_by(|a, b| {
            a.t_start
                .partial_cmp(&b.t_start)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Build the base stream for `carrier_name` by calling the adapter's
/// exemplar generator. `exemplar()` is deterministic in `seed`.
fn base_stream(carrier_name: &str, seed: u64) -> ResidualStream {
    match carrier_name {
        "snowset" => Snowset.exemplar(seed),
        "sqlshare" => SqlShare.exemplar(seed),
        "ceb" => Ceb.exemplar(seed),
        "job" => Job.exemplar(seed),
        _ => unreachable!("CARRIERS list mismatch"),
    }
}

const CARRIERS: [&str; 4] = ["snowset", "sqlshare", "ceb", "job"];
const SCALES: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    fs::create_dir_all(&cli.out)?;
    let csv_path = cli.out.join("inject_over_real.csv");
    let mut wtr = csv::Writer::from_path(&csv_path)?;
    wtr.write_record([
        "carrier",
        "motif",
        "seed",
        "onset_s",
        "duration_s",
        "scale",
        "detected",
        "ttd_s",
        "localization_onset_err_s",
        "localization_closure_err_s",
        "episode_peak",
        "episode_ema_at_boundary",
    ])?;

    for carrier in CARRIERS {
        for motif in MotifClass::ALL {
            for scale in SCALES {
                let mut stream = base_stream(carrier, cli.seed);
                inject(&mut stream, motif, cli.onset_s, cli.duration_s, scale);
                stream.sort();
                let episodes = MotifEngine::new(MotifGrammar::default()).run(&stream);

                let m = first_matching(&episodes, motif, cli.onset_s, cli.duration_s);
                let (detected, ttd, loc_on, loc_close, peak, ema) = match m {
                    Some(e) => (
                        true,
                        (e.t_start - cli.onset_s).max(0.0),
                        (e.t_start - cli.onset_s).abs(),
                        (e.t_end - (cli.onset_s + cli.duration_s)).abs(),
                        e.peak,
                        e.ema_at_boundary,
                    ),
                    None => (false, f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN),
                };
                let fmt = |x: f64| {
                    if x.is_nan() {
                        "nan".to_string()
                    } else {
                        format!("{:.6}", x)
                    }
                };
                wtr.write_record([
                    carrier,
                    motif.name(),
                    &cli.seed.to_string(),
                    &format!("{:.3}", cli.onset_s),
                    &format!("{:.3}", cli.duration_s),
                    &format!("{:.3}", scale),
                    &detected.to_string(),
                    &fmt(ttd),
                    &fmt(loc_on),
                    &fmt(loc_close),
                    &fmt(peak),
                    &fmt(ema),
                ])?;
            }
        }
    }
    wtr.flush()?;
    eprintln!("inject_over_real: wrote {}", csv_path.display());
    Ok(())
}
