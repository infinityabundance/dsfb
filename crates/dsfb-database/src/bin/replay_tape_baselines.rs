#![forbid(unsafe_code)]

//! Replay a SHA-256-pinned residual tape through DSFB + three
//! published change-point baselines (ADWIN, BOCPD, PELT) and score
//! each against a ground-truth windows file.
//!
//! This binary is the live-adapter analogue of
//! [`baseline_bake_off.rs`](./baseline_bake_off.rs): the two detectors
//! read the *same* residual stream, are scored by the *same*
//! `evaluate()` path, and emit bytewise-comparable CSVs. The
//! difference is the input: `baseline_bake_off` runs on the
//! controlled-perturbation TPC-DS stream; `replay_tape_baselines` runs
//! on a tape captured from a live PostgreSQL engine. The tape + the
//! ground-truth JSON together constitute the *real-engine* evaluation
//! fixture the paper's `\S{Live Evaluation}` cites.
//!
//! Determinism. The tape-load path re-verifies the tape SHA-256
//! against the sidecar manifest, so a byte of tape drift aborts the
//! replay before any detector runs. The three baselines are pure
//! functions of the residual stream. Running this binary twice on the
//! same (tape, ground-truth) pair produces byte-identical CSV output;
//! this is the claim pinned by
//! [`tests/live_replay_baselines_reproducibility.rs`](../../tests/live_replay_baselines_reproducibility.rs).
//!
//! Fingerprint safety: outputs land in the user-supplied `--out`
//! directory, which is outside every fingerprint-locked path.

use anyhow::{Context, Result};
use clap::Parser;
use dsfb_database::baselines::{
    adwin::Adwin, bocpd::Bocpd, pelt::Pelt, run_detector, ChangePointDetector,
};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::live::tape::load_and_verify;
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::non_claims;
use dsfb_database::perturbation::{PerturbationClass, PerturbationWindow};
use dsfb_database::residual::ResidualStream;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "replay_tape_baselines",
    about = "Score DSFB + ADWIN/BOCPD/PELT against ground-truth windows on a pinned live tape.",
    version
)]
struct Cli {
    /// Path to the residual tape (JSONL; a `.hash` sidecar manifest
    /// must sit next to it).
    #[arg(long)]
    tape: PathBuf,
    /// Path to the ground-truth windows JSON (schema below).
    #[arg(long)]
    ground_truth: PathBuf,
    /// Output directory for the bakeoff CSV.
    #[arg(long, default_value = "out")]
    out: PathBuf,
}

/// Per-fault ground-truth window. `motif` is a string that must map
/// onto a [`MotifClass`] name (`plan_regression_onset`,
/// `cardinality_mismatch_regime`, `contention_ramp`, `cache_collapse`,
/// `workload_phase_transition`). `channel` is the residual-channel
/// string the fault was planted on; for pg_stat_statements the
/// channel is `md5(queryid::text)`, i.e. the adapter's anonymised
/// qid.
#[derive(Debug, Clone, Deserialize)]
struct GroundTruthWindow {
    motif: String,
    channel: String,
    t_start: f64,
    t_end: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct GroundTruth {
    /// Expected tape SHA-256 (32-byte hex). The loader refuses any
    /// tape whose hash does not match; this is a belt-and-braces
    /// check that the ground truth was authored against *this* tape.
    tape_sha256: String,
    fault_description: String,
    windows: Vec<GroundTruthWindow>,
    #[serde(default)]
    #[allow(dead_code)]
    notes: Option<String>,
}

fn motif_name_to_class(s: &str) -> Result<MotifClass> {
    for m in MotifClass::ALL {
        if m.name() == s {
            return Ok(m);
        }
    }
    anyhow::bail!(
        "ground_truth.json: unknown motif name '{}'; valid: {:?}",
        s,
        MotifClass::ALL.iter().map(|m| m.name()).collect::<Vec<_>>()
    );
}

fn motif_class_to_perturbation(m: MotifClass) -> PerturbationClass {
    // Inverse of `metrics::perturbation_to_motif`. Kept local so the
    // one-to-one mapping is pinned by both the ground-truth loader
    // and the scoring path.
    match m {
        MotifClass::PlanRegressionOnset => PerturbationClass::LatencyInjection,
        MotifClass::CardinalityMismatchRegime => PerturbationClass::StatisticsStaleness,
        MotifClass::ContentionRamp => PerturbationClass::LockHold,
        MotifClass::CacheCollapse => PerturbationClass::CacheEviction,
        MotifClass::WorkloadPhaseTransition => PerturbationClass::WorkloadShift,
    }
}

fn load_ground_truth(path: &Path) -> Result<(GroundTruth, Vec<PerturbationWindow>, String)> {
    let bytes = fs::read(path)
        .with_context(|| format!("reading ground-truth file {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let gt_sha: String = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    let gt: GroundTruth = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing ground-truth JSON {}", path.display()))?;
    let mut wins = Vec::with_capacity(gt.windows.len());
    for w in &gt.windows {
        let class = motif_name_to_class(&w.motif)?;
        wins.push(PerturbationWindow {
            class: motif_class_to_perturbation(class),
            t_start: w.t_start,
            t_end: w.t_end,
            channel: w.channel.clone(),
            magnitude: 1.0,
            seed: 0,
        });
    }
    Ok((gt, wins, gt_sha))
}

fn samples_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        h.insert(m, stream.iter_class(m.residual_class()).count());
    }
    h
}

fn score_dsfb(
    stream: &ResidualStream,
    windows: &[PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Vec<PerMotifMetrics> {
    let engine = MotifEngine::new(MotifGrammar::default());
    let episodes = engine.run(stream);
    evaluate(&episodes, windows, samples, stream.duration())
}

fn score_detector(
    detector: &dyn ChangePointDetector,
    stream: &ResidualStream,
    windows: &[PerturbationWindow],
    samples: &HashMap<MotifClass, usize>,
) -> Vec<PerMotifMetrics> {
    let mut eps = Vec::new();
    for m in MotifClass::ALL {
        eps.extend(run_detector(detector, m, stream));
    }
    evaluate(&eps, windows, samples, stream.duration())
}

fn find_row(rows: &[PerMotifMetrics], motif: MotifClass) -> &PerMotifMetrics {
    rows.iter()
        .find(|r| r.motif == motif.name())
        .expect("evaluate() always emits one row per MotifClass::ALL")
}

fn write_bakeoff_csv(
    path: &Path,
    tape_sha: &str,
    gt_sha: &str,
    labelled: &[(&'static str, &[PerMotifMetrics])],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Header comment rows encode provenance so the CSV is
    // self-describing: a future reader does not need the run.sh script
    // or a hash manifest on disk to prove the scoring input.
    let mut buf = String::new();
    buf.push_str(&format!("# tape_sha256={}\n", tape_sha));
    buf.push_str(&format!("# ground_truth_sha256={}\n", gt_sha));
    buf.push_str("detector,motif,tp,fp,fn,precision,recall,f1,ttd_median_s,ttd_p95_s,false_alarm_per_hour\n");
    for (label, rows) in labelled {
        for motif in MotifClass::ALL {
            let r = find_row(rows, motif);
            buf.push_str(&format!(
                "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}\n",
                label,
                motif.name(),
                r.tp,
                r.fp,
                r.fn_,
                r.precision,
                r.recall,
                r.f1,
                r.time_to_detection_median_s,
                r.time_to_detection_p95_s,
                r.false_alarm_rate_per_hour,
            ));
        }
    }
    fs::write(path, buf).with_context(|| format!("writing bakeoff CSV {}", path.display()))?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    non_claims::print();

    let (stream, manifest) = load_and_verify(&cli.tape)
        .with_context(|| format!("loading tape {}", cli.tape.display()))?;
    let (gt, windows, gt_sha) = load_ground_truth(&cli.ground_truth)?;

    if gt.tape_sha256 != manifest.sha256 {
        anyhow::bail!(
            "ground_truth.tape_sha256={} but tape manifest.sha256={}: the ground-truth annotation was authored against a different tape",
            gt.tape_sha256,
            manifest.sha256
        );
    }

    eprintln!("tape.sha256        = {}", manifest.sha256);
    eprintln!("ground_truth.sha256 = {}", gt_sha);
    eprintln!("fault_description   = {}", gt.fault_description);
    eprintln!(
        "residuals in stream = {}  |  duration = {:.3}s  |  ground_truth_windows = {}",
        stream.len(),
        stream.duration(),
        gt.windows.len()
    );

    let samples = samples_per_motif(&stream);
    let dsfb = score_dsfb(&stream, &windows, &samples);
    let adwin = score_detector(&Adwin::default(), &stream, &windows, &samples);
    let bocpd = score_detector(&Bocpd::default(), &stream, &windows, &samples);
    let pelt = score_detector(&Pelt::default(), &stream, &windows, &samples);

    let labelled: Vec<(&'static str, &[PerMotifMetrics])> = vec![
        ("dsfb-database", dsfb.as_slice()),
        ("adwin", adwin.as_slice()),
        ("bocpd", bocpd.as_slice()),
        ("pelt", pelt.as_slice()),
    ];
    fs::create_dir_all(&cli.out)?;
    let csv_path = cli.out.join("bakeoff.csv");
    write_bakeoff_csv(&csv_path, &manifest.sha256, &gt_sha, &labelled)?;
    eprintln!("wrote {}", csv_path.display());
    for (label, rows) in &labelled {
        for motif in MotifClass::ALL {
            let r = find_row(rows, motif);
            eprintln!(
                "  {:>14} | {:<30} | P={:.3} R={:.3} F1={:.3} TTDm={:.2}s",
                label,
                motif.name(),
                r.precision,
                r.recall,
                r.f1,
                r.time_to_detection_median_s
            );
        }
    }
    Ok(())
}
