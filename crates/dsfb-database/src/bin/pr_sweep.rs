#![forbid(unsafe_code)]

//! Phase-A2 precision/recall/F1 sweep.
//!
//! For every motif, sweeps `(drift_threshold, slew_threshold)` as
//! multiplicative factors over the published-baseline defaults. The
//! ground-truth perturbation windows supply the labels so precision,
//! recall, and F1 are well-defined at every grid point. Emits one CSV +
//! one PNG per motif under `<out>/`.
//!
//! A reviewer's question — "are the published F1 numbers the best the
//! grammar can do, or is there a nearby operating point with higher
//! precision / recall?" — gets a direct answer: the baseline point is
//! rendered as a black × on every PR figure, and the swept points show
//! the surrounding region. No number in the paper is defensible without
//! this figure.
//!
//! Fingerprint safety: only one motif's thresholds are varied per
//! grid-point run, and the sweep writes to `<out>/pr.*` paths that are
//! outside the `paper_fingerprint_is_pinned` /
//! `paper_episode_fingerprint_is_pinned` coverage. The single-seed lock
//! is not disturbed.

use anyhow::Result;
use clap::Parser;
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar, MotifParams};
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::non_claims;
use dsfb_database::perturbation::tpcds_with_perturbations;
use dsfb_database::report::plots;
use dsfb_database::residual::ResidualStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "pr_sweep",
    about = "Phase-A2: precision/recall/F1 sweep across the motif envelope grid.",
    version
)]
struct Cli {
    /// Seed for the controlled TPC-DS perturbation harness. We hold
    /// this fixed across the grid so that every PR point is evaluated
    /// on the same residual stream — the only axis we vary is the
    /// envelope.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Output directory. Must be outside the fingerprinted paths.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// Published grid of multiplicative factors relative to each motif's
/// default drift / slew thresholds. 8 factors × 8 factors = 64 grid
/// points per motif, ≥25 as the plan requires, with enough density to
/// make the PR surface visible without overloading the figure.
const FACTORS: &[f64] = &[0.25, 0.50, 0.75, 1.00, 1.25, 1.50, 2.00, 3.00];

fn samples_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        h.insert(m, stream.iter_class(m.residual_class()).count());
    }
    h
}

/// Rebuild a grammar with one motif's drift/slew thresholds rescaled
/// by independent multiplicative factors, leaving all other motif
/// parameters and all other motifs' parameters at their default.
fn grammar_with_override(
    target: MotifClass,
    drift_factor: f64,
    slew_factor: f64,
    baseline: &MotifGrammar,
) -> MotifGrammar {
    let mut g = baseline.clone();
    let base = MotifParams::default_for(target);
    let new = MotifParams {
        drift_threshold: base.drift_threshold * drift_factor,
        slew_threshold: base.slew_threshold * slew_factor,
        ..base
    };
    match target {
        MotifClass::PlanRegressionOnset => g.plan_regression_onset = new,
        MotifClass::CardinalityMismatchRegime => g.cardinality_mismatch_regime = new,
        MotifClass::ContentionRamp => g.contention_ramp = new,
        MotifClass::CacheCollapse => g.cache_collapse = new,
        MotifClass::WorkloadPhaseTransition => g.workload_phase_transition = new,
    }
    g
}

/// Run one (motif, drift_factor, slew_factor) grid point and return the
/// metrics for the target motif only. Returns `None` if the metrics row
/// for the target motif is absent (should not happen; `evaluate`
/// guarantees one row per motif, but we keep the branch honest).
fn run_grid_point(
    target: MotifClass,
    drift_factor: f64,
    slew_factor: f64,
    stream: &ResidualStream,
    baseline: &MotifGrammar,
    windows: &[dsfb_database::perturbation::PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Option<PerMotifMetrics> {
    let g = grammar_with_override(target, drift_factor, slew_factor, baseline);
    let episodes = MotifEngine::new(g).run(stream);
    let rows = evaluate(&episodes.clone(), windows, samples, stream.duration());
    rows.into_iter().find(|r| r.motif == target.name())
}

/// Write a per-motif PR sweep CSV. Row order is lexicographic on
/// `(drift_factor, slew_factor)` so regenerating is bytewise identical.
fn write_pr_csv(
    path: &Path,
    rows: &[(f64, f64, PerMotifMetrics)],
    seed: u64,
    motif: MotifClass,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif",
        "seed",
        "drift_factor",
        "slew_factor",
        "drift_threshold",
        "slew_threshold",
        "tp",
        "fp",
        "fn",
        "precision",
        "recall",
        "f1",
    ])?;
    let base = MotifParams::default_for(motif);
    for (df, sf, m) in rows {
        wtr.write_record([
            &m.motif,
            &seed.to_string(),
            &format!("{:.3}", df),
            &format!("{:.3}", sf),
            &format!("{:.6}", base.drift_threshold * df),
            &format!("{:.6}", base.slew_threshold * sf),
            &m.tp.to_string(),
            &m.fp.to_string(),
            &m.fn_.to_string(),
            &format!("{:.6}", m.precision),
            &format!("{:.6}", m.recall),
            &format!("{:.6}", m.f1),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    // Build the residual stream once — the sweep varies only the
    // envelope, not the harness.
    let (stream, windows) = tpcds_with_perturbations(cli.seed);
    let samples = samples_per_motif(&stream);
    let baseline_grammar = MotifGrammar::default();

    fs::create_dir_all(&cli.out)?;
    for target in MotifClass::ALL {
        let mut grid_rows: Vec<(f64, f64, PerMotifMetrics)> =
            Vec::with_capacity(FACTORS.len() * FACTORS.len());
        for &df in FACTORS {
            for &sf in FACTORS {
                if let Some(m) = run_grid_point(
                    target,
                    df,
                    sf,
                    &stream,
                    &baseline_grammar,
                    &windows,
                    &samples,
                ) {
                    grid_rows.push((df, sf, m));
                }
            }
        }
        debug_assert_eq!(
            grid_rows.len(),
            FACTORS.len() * FACTORS.len(),
            "one grid point per (drift_factor, slew_factor)"
        );

        let csv_path = cli.out.join(format!("pr.{}.csv", target.name()));
        write_pr_csv(&csv_path, &grid_rows, cli.seed, target)?;

        // Baseline = (1.0, 1.0) grid point — guaranteed present by the
        // factor list.
        let baseline_point = grid_rows
            .iter()
            .find(|(df, sf, _)| (*df - 1.0).abs() < 1e-9 && (*sf - 1.0).abs() < 1e-9)
            .map(|(_, _, m)| (m.precision, m.recall));

        let plot_rows: Vec<(f64, f64, f64, String)> = grid_rows
            .iter()
            .map(|(df, sf, m)| {
                (
                    m.precision,
                    m.recall,
                    m.f1,
                    format!("drift*{df:.2}, slew*{sf:.2}"),
                )
            })
            .collect();
        let png_path = cli.out.join(format!("pr.{}.png", target.name()));
        // Caption is deliberately terse: the companion CSV (same
        // stem) carries every provenance detail (seed, FACTORS list,
        // thresholds) that an axis title cannot.
        plots::plot_pr_curve(
            &png_path,
            &format!("PR sweep: {}", target.name()),
            &plot_rows,
            baseline_point,
        )?;
        eprintln!(
            "pr_sweep[{}]: {} points, wrote {} + {}",
            target.name(),
            grid_rows.len(),
            csv_path.display(),
            png_path.display()
        );
    }
    Ok(())
}
