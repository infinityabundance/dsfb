#![forbid(unsafe_code)]

//! Baseline hyperparameter tuning with held-out replication discipline.
//!
//! Sweeps a small, documented grid of hyperparameters for each of the
//! three published change-point baselines (ADWIN, BOCPD, PELT) on a
//! training split (one replication per fault class), picks the best
//! macro-F1 config per baseline, and evaluates the frozen tuned config
//! on the held-out test split (the remaining replications). DSFB is
//! evaluated at defaults — we deliberately do not re-tune DSFB on the
//! real tapes so that the comparison is:
//!
//!   * baselines as good as the training split allows
//!   * DSFB exactly as published
//!
//! Output: one CSV row per (baseline, split) with macro-F1 mean and
//! 95 % percentile-bootstrap CI across the split's tapes.
//!
//! The per-(detector, fault) breakdown on the test split is written
//! to `per_fault.csv` alongside.
//!
//! Fingerprint safety: outputs land in the user-supplied `--out`
//! directory, outside every fingerprint-locked path. The binary does
//! not mutate any adapter, grammar, or spec file.

use anyhow::{Context, Result};
use clap::Parser;
use dsfb_database::baselines::{
    adwin::Adwin, bocpd::Bocpd, pelt::Pelt, run_detector, ChangePointDetector,
};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};
use dsfb_database::live::tape::load_and_verify;
use dsfb_database::metrics::{evaluate, PerMotifMetrics};
use dsfb_database::perturbation::{PerturbationClass, PerturbationWindow};
use dsfb_database::residual::ResidualStream;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "baseline_tune",
    about = "Sweep baseline hyperparams on a training split; evaluate frozen best config on a held-out test split.",
    version
)]
struct Cli {
    /// Root directory with per-fault subdirs of per-replication tapes.
    /// Layout: `<root>/<fault>/r{01..N}/{live.tape.jsonl, ground_truth.json}`.
    #[arg(long)]
    root: PathBuf,
    /// Replication index (1-based) to use as the training split. Every
    /// other replication in each fault directory is test.
    #[arg(long, default_value_t = 1)]
    train_rep: usize,
    /// Output directory for tuned_summary.csv + per_fault.csv.
    #[arg(long, default_value = "out/baseline_tune")]
    out: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct GroundTruthWindow {
    motif: String,
    channel: String,
    t_start: f64,
    t_end: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct GroundTruth {
    tape_sha256: String,
    #[allow(dead_code)]
    fault_description: String,
    windows: Vec<GroundTruthWindow>,
}

struct Tape {
    stream: ResidualStream,
    windows: Vec<PerturbationWindow>,
    fault: String,
    rep: String,
    exercised_motif: MotifClass,
}

fn motif_name_to_class(s: &str) -> Option<MotifClass> {
    MotifClass::ALL.iter().copied().find(|m| m.name() == s)
}

fn motif_class_to_perturbation(m: MotifClass) -> PerturbationClass {
    match m {
        MotifClass::PlanRegressionOnset => PerturbationClass::LatencyInjection,
        MotifClass::CardinalityMismatchRegime => PerturbationClass::StatisticsStaleness,
        MotifClass::ContentionRamp => PerturbationClass::LockHold,
        MotifClass::CacheCollapse => PerturbationClass::CacheEviction,
        MotifClass::WorkloadPhaseTransition => PerturbationClass::WorkloadShift,
    }
}

fn load_gt_windows(path: &Path) -> Result<(GroundTruth, Vec<PerturbationWindow>, MotifClass)> {
    let bytes = fs::read(path)
        .with_context(|| format!("reading ground-truth {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let _: String = h.finalize().iter().map(|b| format!("{:02x}", b)).collect();
    let gt: GroundTruth = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing ground-truth {}", path.display()))?;
    let mut windows = Vec::with_capacity(gt.windows.len());
    let mut exercised: Option<MotifClass> = None;
    for w in &gt.windows {
        let class = motif_name_to_class(&w.motif)
            .with_context(|| format!("unknown motif {}", w.motif))?;
        if exercised.is_none() {
            exercised = Some(class);
        }
        windows.push(PerturbationWindow {
            class: motif_class_to_perturbation(class),
            t_start: w.t_start,
            t_end: w.t_end,
            channel: w.channel.clone(),
            magnitude: 1.0,
            seed: 0,
        });
    }
    let exercised = exercised.context("ground_truth has no windows")?;
    Ok((gt, windows, exercised))
}

fn discover_tapes(root: &Path) -> Result<Vec<Tape>> {
    let mut tapes = Vec::new();
    let mut fault_dirs: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        // Skip aggregator-only dirs that have no r*/ children.
        if entry.path().join("provenance.txt").exists()
            || fs::read_dir(entry.path())?
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with('r'))
        {
            fault_dirs.push(entry.path());
            let _ = name;
        }
    }
    fault_dirs.sort();
    for fault_dir in fault_dirs {
        let fault = fault_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut reps: Vec<PathBuf> = fs::read_dir(&fault_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .map(|s| s.to_string_lossy().starts_with('r'))
                        .unwrap_or(false)
            })
            .collect();
        reps.sort();
        for rep_dir in reps {
            let rep = rep_dir
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let tape_path = rep_dir.join("live.tape.jsonl");
            let gt_path = rep_dir.join("ground_truth.json");
            if !tape_path.exists() || !gt_path.exists() {
                continue;
            }
            let (gt, windows, exercised) = load_gt_windows(&gt_path)?;
            let (stream, manifest) = load_and_verify(&tape_path)
                .with_context(|| format!("loading tape {}", tape_path.display()))?;
            if gt.tape_sha256 != manifest.sha256 {
                anyhow::bail!(
                    "tape/gt mismatch in {} (gt={}, tape={})",
                    rep_dir.display(),
                    gt.tape_sha256,
                    manifest.sha256
                );
            }
            tapes.push(Tape {
                stream,
                windows,
                fault: fault.clone(),
                rep,
                exercised_motif: exercised,
            });
        }
    }
    Ok(tapes)
}

fn samples_per_motif(stream: &ResidualStream) -> HashMap<MotifClass, usize> {
    let mut h = HashMap::new();
    for m in MotifClass::ALL {
        h.insert(m, stream.iter_class(m.residual_class()).count());
    }
    h
}

fn score_detector_f1(
    det: &dyn ChangePointDetector,
    tape: &Tape,
) -> f64 {
    let samples = samples_per_motif(&tape.stream);
    let mut eps = Vec::new();
    for m in MotifClass::ALL {
        eps.extend(run_detector(det, m, &tape.stream));
    }
    let rows = evaluate(&eps, &tape.windows, &samples, tape.stream.duration());
    find_motif_f1(&rows, tape.exercised_motif)
}

fn score_dsfb_f1(tape: &Tape) -> f64 {
    let engine = MotifEngine::new(MotifGrammar::default());
    let episodes = engine.run(&tape.stream);
    let samples = samples_per_motif(&tape.stream);
    let rows = evaluate(&episodes, &tape.windows, &samples, tape.stream.duration());
    find_motif_f1(&rows, tape.exercised_motif)
}

fn find_motif_f1(rows: &[PerMotifMetrics], motif: MotifClass) -> f64 {
    rows.iter()
        .find(|r| r.motif == motif.name())
        .map(|r| r.f1)
        .unwrap_or(0.0)
}

fn mean(vs: &[f64]) -> f64 {
    if vs.is_empty() {
        0.0
    } else {
        vs.iter().sum::<f64>() / vs.len() as f64
    }
}

/// Small deterministic LCG so the binary has no `rand` dep beyond
/// what the workspace already pulls in. Seeded at 42 for reproducibility.
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
    boots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = boots[(alpha / 2.0 * b as f64) as usize];
    let hi = boots[((1.0 - alpha / 2.0) * b as f64) as usize];
    (mean(vs), lo, hi)
}

/// Grid for ADWIN: δ ∈ {1e-3, 2e-3, 5e-3, 1e-2}, min_side ∈ {3, 5, 10}.
fn adwin_grid() -> Vec<(Adwin, String)> {
    let deltas = [0.001, 0.002, 0.005, 0.01];
    let sides = [3usize, 5, 10];
    let mut out = Vec::new();
    for &d in &deltas {
        for &s in &sides {
            out.push((
                Adwin { delta: d, min_side: s },
                format!("delta={d:.3};min_side={s}"),
            ));
        }
    }
    out
}

/// Grid for BOCPD: expected_run_length ∈ {50, 100, 200, 500},
/// map_drop_min ∈ {1, 2, 3}.
fn bocpd_grid() -> Vec<(Bocpd, String)> {
    let lambdas = [50.0, 100.0, 200.0, 500.0];
    let drops = [1usize, 2, 3];
    let mut out = Vec::new();
    for &l in &lambdas {
        for &d in &drops {
            let mut b = Bocpd::default();
            b.expected_run_length = l;
            b.map_drop_min = d;
            out.push((b, format!("lambda={l:.0};map_drop_min={d}")));
        }
    }
    out
}

/// Grid for PELT: penalty_k ∈ {1, 2, 4, 8}, min_seg_len ∈ {3, 5, 10}.
fn pelt_grid() -> Vec<(Pelt, String)> {
    let ks = [1.0, 2.0, 4.0, 8.0];
    let seglens = [3usize, 5, 10];
    let mut out = Vec::new();
    for &k in &ks {
        for &s in &seglens {
            out.push((
                Pelt { penalty_k: k, min_seg_len: s },
                format!("penalty_k={k:.1};min_seg_len={s}"),
            ));
        }
    }
    out
}

fn tune_and_eval<D, F>(
    name: &str,
    train: &[Tape],
    test: &[Tape],
    grid: Vec<(D, String)>,
    score: F,
) -> TuneResult
where
    D: ChangePointDetector,
    F: Fn(&D, &Tape) -> f64,
{
    let mut best_cfg: Option<String> = None;
    let mut best_cfg_idx: Option<usize> = None;
    let mut best_train = -1.0f64;

    for (i, (det, cfg)) in grid.iter().enumerate() {
        let tr: Vec<f64> = train.iter().map(|t| score(det, t)).collect();
        let m = mean(&tr);
        if m > best_train {
            best_train = m;
            best_cfg = Some(cfg.clone());
            best_cfg_idx = Some(i);
        }
    }

    let best_det = best_cfg_idx.map(|i| &grid[i].0);
    let test_f1s: Vec<f64> = if let Some(d) = best_det {
        test.iter().map(|t| score(d, t)).collect()
    } else {
        Vec::new()
    };
    let per_fault: Vec<(String, String, f64)> = if let Some(d) = best_det {
        test.iter()
            .map(|t| (t.fault.clone(), t.rep.clone(), score(d, t)))
            .collect()
    } else {
        Vec::new()
    };
    let (m, lo, hi) = bootstrap_ci(&test_f1s);
    TuneResult {
        baseline: name.to_string(),
        best_config: best_cfg.unwrap_or_default(),
        f1_train: best_train,
        f1_test_mean: m,
        f1_test_ci95_lo: lo,
        f1_test_ci95_hi: hi,
        f1_test_n: test_f1s.len(),
        per_fault,
    }
}

struct TuneResult {
    baseline: String,
    best_config: String,
    f1_train: f64,
    f1_test_mean: f64,
    f1_test_ci95_lo: f64,
    f1_test_ci95_hi: f64,
    f1_test_n: usize,
    per_fault: Vec<(String, String, f64)>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.out)?;

    eprintln!("scanning tapes under {}", cli.root.display());
    let all = discover_tapes(&cli.root)?;
    eprintln!("discovered {} tapes", all.len());

    let train_rep_label = format!("r{:02}", cli.train_rep);
    let (train, test): (Vec<&Tape>, Vec<&Tape>) = all.iter().partition(|t| t.rep == train_rep_label);
    let train: Vec<Tape> = train
        .into_iter()
        .map(|t| Tape {
            stream: t.stream.clone(),
            windows: t.windows.clone(),
            fault: t.fault.clone(),
            rep: t.rep.clone(),
            exercised_motif: t.exercised_motif,
        })
        .collect();
    let test: Vec<Tape> = test
        .into_iter()
        .map(|t| Tape {
            stream: t.stream.clone(),
            windows: t.windows.clone(),
            fault: t.fault.clone(),
            rep: t.rep.clone(),
            exercised_motif: t.exercised_motif,
        })
        .collect();

    eprintln!(
        "train: {} tapes (rep={}); test: {} tapes",
        train.len(),
        train_rep_label,
        test.len()
    );

    let adwin_res =
        tune_and_eval("adwin", &train, &test, adwin_grid(), |d, t| score_detector_f1(d, t));
    let bocpd_res =
        tune_and_eval("bocpd", &train, &test, bocpd_grid(), |d, t| score_detector_f1(d, t));
    let pelt_res =
        tune_and_eval("pelt", &train, &test, pelt_grid(), |d, t| score_detector_f1(d, t));

    // DSFB at defaults.
    let dsfb_train_f1s: Vec<f64> = train.iter().map(score_dsfb_f1).collect();
    let dsfb_test_f1s: Vec<f64> = test.iter().map(score_dsfb_f1).collect();
    let (dsfb_m, dsfb_lo, dsfb_hi) = bootstrap_ci(&dsfb_test_f1s);
    let dsfb_res = TuneResult {
        baseline: "dsfb-database".to_string(),
        best_config: "defaults".to_string(),
        f1_train: mean(&dsfb_train_f1s),
        f1_test_mean: dsfb_m,
        f1_test_ci95_lo: dsfb_lo,
        f1_test_ci95_hi: dsfb_hi,
        f1_test_n: dsfb_test_f1s.len(),
        per_fault: test
            .iter()
            .zip(dsfb_test_f1s.iter())
            .map(|(t, f)| (t.fault.clone(), t.rep.clone(), *f))
            .collect(),
    };

    let results = [dsfb_res, adwin_res, bocpd_res, pelt_res];

    // tuned_summary.csv
    let mut buf = String::new();
    buf.push_str("baseline,best_config,f1_train,f1_test_mean,f1_test_ci95_lo,f1_test_ci95_hi,f1_test_n\n");
    for r in &results {
        buf.push_str(&format!(
            "{},{},{:.6},{:.6},{:.6},{:.6},{}\n",
            r.baseline,
            r.best_config,
            r.f1_train,
            r.f1_test_mean,
            r.f1_test_ci95_lo,
            r.f1_test_ci95_hi,
            r.f1_test_n
        ));
    }
    let summary_path = cli.out.join("tuned_summary.csv");
    fs::write(&summary_path, buf)
        .with_context(|| format!("writing {}", summary_path.display()))?;
    eprintln!("wrote {}", summary_path.display());

    // per_fault.csv
    let mut buf = String::new();
    buf.push_str("baseline,fault,rep,f1\n");
    for r in &results {
        for (fault, rep, f1) in &r.per_fault {
            buf.push_str(&format!(
                "{},{},{},{:.6}\n",
                r.baseline, fault, rep, f1
            ));
        }
    }
    let pf_path = cli.out.join("per_fault.csv");
    fs::write(&pf_path, buf)
        .with_context(|| format!("writing {}", pf_path.display()))?;
    eprintln!("wrote {}", pf_path.display());

    for r in &results {
        eprintln!(
            "  {:>14} | best={:<32} | train F1={:.3} | test F1={:.3} [{:.3}, {:.3}] (n={})",
            r.baseline,
            r.best_config,
            r.f1_train,
            r.f1_test_mean,
            r.f1_test_ci95_lo,
            r.f1_test_ci95_hi,
            r.f1_test_n
        );
    }
    Ok(())
}
