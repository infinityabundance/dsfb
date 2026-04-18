#![forbid(unsafe_code)]

//! Phase-A4 bake-off: dsfb-database vs. three published change-point
//! baselines on the same residual streams.
//!
//! The bake-off answers a reviewer's mandatory question — *"how do
//! standard change-point detectors score on the same windows?"* — by
//! running ADWIN (Bifet & Gavaldà 2007), BOCPD (Adams & MacKay 2007),
//! and PELT (Killick 2012) over the exact residual class the dsfb motif
//! grammar consumes, scoring them with the exact same TP / FP / FN
//! rules, and emitting one comparison CSV per motif.
//!
//! Determinism: the three baselines and the dsfb motif engine are all
//! pure functions of the pinned single-seed TPC-DS stream. Running this
//! binary twice produces byte-identical CSVs.
//!
//! Fingerprint safety: outputs land at `<out>/bakeoff.<motif>.csv`, far
//! outside the paths the four pinned fingerprint-lock tests guard.

use anyhow::Result;
use clap::Parser;
use dsfb_database::baselines::{
    adwin::Adwin, bocpd::Bocpd, pelt::Pelt, run_detector, ChangePointDetector,
};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::non_claims;
use dsfb_database::perturbation::{tpcds_with_perturbations, PerturbationWindow};
use dsfb_database::residual::ResidualStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "baseline_bake_off",
    about = "Phase-A4: dsfb-database vs. ADWIN / BOCPD / PELT on the same perturbation stream.",
    version
)]
struct Cli {
    /// Seed for the controlled TPC-DS perturbation harness. Held fixed
    /// because the bake-off's point is apples-to-apples on a single
    /// published trace.
    #[arg(long, default_value_t = 42)]
    seed: u64,
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

/// Run one detector, wrap its change-points into Episodes, and score
/// them with the standard `evaluate` against the perturbation windows.
fn score_detector(
    detector: &dyn ChangePointDetector,
    stream: &ResidualStream,
    windows: &[PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Vec<PerMotifMetrics> {
    let mut all_eps = Vec::new();
    for motif in MotifClass::ALL {
        all_eps.extend(run_detector(detector, motif, stream));
    }
    evaluate(&all_eps, windows, samples, stream.duration())
}

/// Score the dsfb motif grammar itself for reference. This duplicates
/// the `reproduce` code path at default parameters, which is exactly
/// the comparison a reviewer needs in the bake-off.
fn score_dsfb(
    stream: &ResidualStream,
    windows: &[PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Vec<PerMotifMetrics> {
    let episodes = MotifEngine::new(MotifGrammar::default()).run(stream);
    evaluate(&episodes, windows, samples, stream.duration())
}

fn find_row(rows: &[PerMotifMetrics], motif: MotifClass) -> &PerMotifMetrics {
    rows.iter()
        .find(|r| r.motif == motif.name())
        .expect("evaluate() guarantees one row per motif")
}

/// Per-motif CSV, one row per (detector, motif). Row order is fixed by
/// `LABELS` so regenerating produces byte-identical output.
fn write_bakeoff_csv(
    path: &Path,
    seed: u64,
    motif: MotifClass,
    labelled: &[(&'static str, &[PerMotifMetrics])],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "detector",
        "motif",
        "seed",
        "tp",
        "fp",
        "fn",
        "precision",
        "recall",
        "f1",
        "ttd_median_s",
        "ttd_p95_s",
    ])?;
    for (label, rows) in labelled {
        let r = find_row(rows, motif);
        wtr.write_record([
            *label,
            motif.name(),
            &seed.to_string(),
            &r.tp.to_string(),
            &r.fp.to_string(),
            &r.fn_.to_string(),
            &format!("{:.6}", r.precision),
            &format!("{:.6}", r.recall),
            &format!("{:.6}", r.f1),
            &format!("{:.6}", r.time_to_detection_median_s),
            &format!("{:.6}", r.time_to_detection_p95_s),
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

    let adwin = Adwin::default();
    let bocpd = Bocpd::default();
    let pelt = Pelt::default();

    let dsfb_rows = score_dsfb(&stream, &windows, &samples);
    let adwin_rows = score_detector(&adwin, &stream, &windows, &samples);
    let bocpd_rows = score_detector(&bocpd, &stream, &windows, &samples);
    let pelt_rows = score_detector(&pelt, &stream, &windows, &samples);

    // dsfb first so the reference row appears at the top of every CSV
    // — makes a diff-based reviewer's job trivially easy.
    let labelled: Vec<(&'static str, &[PerMotifMetrics])> = vec![
        ("dsfb-database", dsfb_rows.as_slice()),
        ("adwin", adwin_rows.as_slice()),
        ("bocpd", bocpd_rows.as_slice()),
        ("pelt", pelt_rows.as_slice()),
    ];

    fs::create_dir_all(&cli.out)?;
    for motif in MotifClass::ALL {
        let csv_path = cli.out.join(format!("bakeoff.{}.csv", motif.name()));
        write_bakeoff_csv(&csv_path, cli.seed, motif, &labelled)?;
        eprintln!(
            "bake_off[{}]: dsfb F1 {:.3} | adwin F1 {:.3} | bocpd F1 {:.3} | pelt F1 {:.3} | wrote {}",
            motif.name(),
            find_row(&dsfb_rows, motif).f1,
            find_row(&adwin_rows, motif).f1,
            find_row(&bocpd_rows, motif).f1,
            find_row(&pelt_rows, motif).f1,
            csv_path.display()
        );
    }
    Ok(())
}
