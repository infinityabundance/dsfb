#![forbid(unsafe_code)]

//! Phase-B4: one-at-a-time motif-parameter ablation.
//!
//! `pr_sweep` covers the 2-D `(drift, slew)` envelope. This binary
//! complements it by sweeping *each* of the five `MotifParams` knobs
//! independently — ρ (EMA smoothing), σ₀ (trust softness),
//! drift_threshold, slew_threshold, and min_dwell_seconds — while
//! holding the other four at published defaults.
//!
//! The anti-cherry-picked-parameters claim a reviewer looks for is
//! "how wide is the parameter band over which this motif's $F_1$
//! remains at its reported value?". The ablation CSV answers exactly
//! that: one row per (motif, parameter, factor), with $F_1$ and the
//! other per-motif metrics at each point.
//!
//! Fingerprint safety: only the per-motif grammar is varied; the
//! residual stream itself is the pinned seed-42 TPC-DS perturbation
//! stream. Artefacts land at `<out>/ablation.<motif>.csv`, outside
//! the fingerprint-lock coverage.

use anyhow::Result;
use clap::Parser;
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar, MotifParams};
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::non_claims;
use dsfb_database::perturbation::{tpcds_with_perturbations, PerturbationWindow};
use dsfb_database::residual::ResidualStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "ablation_sweep",
    about = "Phase-B4: one-at-a-time ablation of each MotifParams knob per motif.",
    version
)]
struct Cli {
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// Multiplicative factors applied to the published default. ρ is
/// handled additively (see `sweep_param`) because it is bounded at 1.
const FACTORS: &[f64] = &[0.25, 0.50, 0.75, 1.00, 1.25, 1.50, 2.00, 3.00];

/// Additive offsets for ρ. ρ is a smoothing factor in [0,1); a
/// multiplicative sweep would either saturate or fall off a cliff at
/// the boundary. Additive deltas centred on the default keep every
/// probe inside (0, 1).
const RHO_DELTAS: &[f64] = &[-0.20, -0.10, -0.05, 0.0, 0.025, 0.05, 0.075, 0.099];

fn samples_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        h.insert(m, stream.iter_class(m.residual_class()).count());
    }
    h
}

fn install(g: &mut MotifGrammar, target: MotifClass, p: MotifParams) {
    match target {
        MotifClass::PlanRegressionOnset => g.plan_regression_onset = p,
        MotifClass::CardinalityMismatchRegime => g.cardinality_mismatch_regime = p,
        MotifClass::ContentionRamp => g.contention_ramp = p,
        MotifClass::CacheCollapse => g.cache_collapse = p,
        MotifClass::WorkloadPhaseTransition => g.workload_phase_transition = p,
    }
}

/// Apply a perturbation to a single parameter and return the mutated
/// grammar. For `rho`, the perturbation is additive (bounded domain);
/// for every other parameter it is multiplicative.
fn grammar_with_perturbed_param(target: MotifClass, param_name: &str, factor: f64) -> MotifGrammar {
    let mut g = MotifGrammar::default();
    let base = MotifParams::default_for(target);
    let new = match param_name {
        "rho" => MotifParams {
            // factor is an additive offset here
            rho: (base.rho + factor).clamp(0.0, 0.999),
            ..base
        },
        "sigma0" => MotifParams {
            sigma0: base.sigma0 * factor,
            ..base
        },
        "drift_threshold" => MotifParams {
            drift_threshold: base.drift_threshold * factor,
            ..base
        },
        "slew_threshold" => MotifParams {
            slew_threshold: base.slew_threshold * factor,
            ..base
        },
        "min_dwell_seconds" => MotifParams {
            min_dwell_seconds: base.min_dwell_seconds * factor,
            ..base
        },
        _ => unreachable!("unknown param_name {param_name}"),
    };
    install(&mut g, target, new);
    g
}

fn run_point(
    g: MotifGrammar,
    target: MotifClass,
    stream: &ResidualStream,
    windows: &[PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Option<PerMotifMetrics> {
    let episodes = MotifEngine::new(g).run(stream);
    let rows = evaluate(&episodes, windows, samples, stream.duration());
    rows.into_iter().find(|r| r.motif == target.name())
}

/// Return the set of probe values and the on-disk value to log for a
/// given parameter. For `rho` we log `base.rho + delta`; for the
/// multiplicative knobs we log `base * factor`.
fn probe_values(param_name: &str) -> Vec<f64> {
    match param_name {
        "rho" => RHO_DELTAS.to_vec(),
        _ => FACTORS.to_vec(),
    }
}

fn effective_value(base: &MotifParams, param_name: &str, probe: f64) -> f64 {
    match param_name {
        "rho" => (base.rho + probe).clamp(0.0, 0.999),
        "sigma0" => base.sigma0 * probe,
        "drift_threshold" => base.drift_threshold * probe,
        "slew_threshold" => base.slew_threshold * probe,
        "min_dwell_seconds" => base.min_dwell_seconds * probe,
        _ => unreachable!(),
    }
}

const PARAMS: [&str; 5] = [
    "rho",
    "sigma0",
    "drift_threshold",
    "slew_threshold",
    "min_dwell_seconds",
];

fn write_ablation_csv(
    path: &Path,
    motif: MotifClass,
    seed: u64,
    rows: &[(&'static str, f64, f64, PerMotifMetrics)],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif",
        "seed",
        "param",
        "probe",
        "effective_value",
        "tp",
        "fp",
        "fn",
        "precision",
        "recall",
        "f1",
        "ttd_median_s",
        "ttd_p95_s",
    ])?;
    for (param, probe, eff, m) in rows {
        wtr.write_record([
            motif.name(),
            &seed.to_string(),
            param,
            &format!("{:.6}", probe),
            &format!("{:.6}", eff),
            &m.tp.to_string(),
            &m.fp.to_string(),
            &m.fn_.to_string(),
            &format!("{:.6}", m.precision),
            &format!("{:.6}", m.recall),
            &format!("{:.6}", m.f1),
            &format!("{:.6}", m.time_to_detection_median_s),
            &format!("{:.6}", m.time_to_detection_p95_s),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    let (stream, windows) = tpcds_with_perturbations(cli.seed);
    let samples = samples_per_motif(&stream);
    fs::create_dir_all(&cli.out)?;

    for target in MotifClass::ALL {
        let base = MotifParams::default_for(target);
        let mut rows: Vec<(&'static str, f64, f64, PerMotifMetrics)> = Vec::new();
        for param in PARAMS {
            for probe in probe_values(param) {
                let g = grammar_with_perturbed_param(target, param, probe);
                if let Some(m) = run_point(g, target, &stream, &windows, &samples) {
                    rows.push((param, probe, effective_value(&base, param, probe), m));
                }
            }
        }
        let path = cli.out.join(format!("ablation.{}.csv", target.name()));
        write_ablation_csv(&path, target, cli.seed, &rows)?;
        let min_f1 = rows
            .iter()
            .map(|(_, _, _, m)| m.f1)
            .fold(f64::INFINITY, f64::min);
        let max_f1 = rows
            .iter()
            .map(|(_, _, _, m)| m.f1)
            .fold(f64::NEG_INFINITY, f64::max);
        eprintln!(
            "ablation[{}]: {} points, F1 range [{:.3}, {:.3}], wrote {}",
            target.name(),
            rows.len(),
            min_f1,
            max_f1,
            path.display()
        );
    }
    Ok(())
}
