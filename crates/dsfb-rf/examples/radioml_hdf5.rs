// ══════════════════════════════════════════════════════════════════════════════
//  SYNTHETIC / DIAGNOSTIC STUB — NOT A REAL-DATASET RESULT
//  ──────────────────────────────────────────────────────────
//  This example runs a RAW-IQ AMPLITUDE PROXY in place of the receiver-residual
//  input DSFB is designed for. The precision / recall numbers it prints are
//  NOT valid claims about DSFB detection performance, and no headline number
//  in the companion paper's Table 1 depends on this example.
//
//  For the real-dataset reproduction path see REPRODUCE.md §2.2 and the
//  `paper-lock` binary driven by `data/RML2018.01a.hdf5` (20 GB, external).
//  The in-repo stratified 240-capture slice `data/slices/radioml_2018_slice.hdf5`
//  is a SMOKE-TEST asset only.
// ══════════════════════════════════════════════════════════════════════════════
//! RadioML 2018.01a raw-IQ proxy evaluation (NOT the paper's receiver-residual protocol).
//!
//! # Scope
//!
//! This example demonstrates the DSFB-RF pipeline with a raw-IQ amplitude
//! (RMS) proxy in place of the intended receiver-residual input.  It is
//! provided for transparency and diagnostics only.  The resulting precision
//! and recall numbers are NOT valid claims about DSFB detection performance.
//!
//! DSFB is an **augmentation of a downstream receiver chain**.  The correct
//! `residual_norm` for a RadioML evaluation is the norm of the demodulator or
//! classifier error after symbol detection — not the raw IQ amplitude.
//! Without a pre-trained receiver the DSFB feature space collapses:
//! per-cell normalised RMS is ≈ 1.0 in every (mod, SNR) cell, giving the
//! engine no signal to work with.
//!
//! # Usage
//!
//! ```text
//! cargo run --example radioml_hdf5 --features std,paper_lock,hdf5_loader \
//!     -- [path/to/RML2018.01a.hdf5] [--report-only] [--per-class]
//! ```
//!
//! Default path: `data/RML2018.01a.hdf5`

use dsfb_rf::hdf5_loader::{load_radioml, load_radioml_per_class};
use dsfb_rf::paper_lock::PaperLockConfig;
use dsfb_rf::pipeline::run_stage_iii;

const DEFAULT_PATH: &str = "data/RML2018.01a.hdf5";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[SYNTHETIC STUB] raw-IQ amplitude proxy — no paper claim depends on this example's output (see REPRODUCE.md §3)");
    let args: Vec<String> = std::env::args().collect();
    let path = args.iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(String::as_str)
        .unwrap_or(DEFAULT_PATH);
    let report_only = args.iter().any(|a| a == "--report-only");
    let per_class   = args.iter().any(|a| a == "--per-class");

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   DSFB-RF RadioML 2018.01a — RAW-IQ PROXY EVALUATION         ║");
    println!("║   NOT the paper's receiver-residual protocol. See hdf5_loader ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  File    : {path}");
    println!("  Mode    : {}", match (per_class, report_only) {
        (true,  true)  => "per-class  (paper-lock skipped)",
        (true,  false) => "per-class",
        (false, true)  => "flat-stream  (paper-lock skipped)",
        (false, false) => "flat-stream  paper-lock verification",
    });
    println!();

    if per_class {
        return run_per_class(path, report_only);
    }
    run_flat_stream(path, report_only)
}

// ── Per-modulation-class evaluation ──────────────────────────────────────────

fn run_per_class(path: &str, report_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let classes = load_radioml_per_class(path)
        .map_err(|e| format!("Failed to load RadioML dataset: {e}"))?;

    println!();
    println!("Running per-class Stage III evaluation (24 classes)…");
    println!("{:>6}  {:>8}  {:>8}  {:>10}  {:>8}  {:>7}",
        "Class", "Episodes", "GT-evs", "Precision%", "Recall%", "Compress");

    let mut total_eps:  usize = 0;
    let mut total_gt:   usize = 0;
    let mut total_rcl:  usize = 0;
    let mut total_raw:  usize = 0;

    for (cls, (obs, evs)) in classes.iter().enumerate() {
        let r = run_stage_iii("RadioML per-class", obs, evs);
        let prec_pct = r.episode_precision * 100.0;
        let rcl_pct  = r.recall() * 100.0;
        println!("{:>6}  {:>8}  {:>8}  {:>10.1}  {:>8.1}  {:>7.1}×",
            cls, r.dsfb_episode_count, evs.len(), prec_pct, rcl_pct, r.compression_factor);
        total_eps += r.dsfb_episode_count;
        total_gt  += evs.len();
        total_rcl += r.recall_numerator;
        total_raw += r.raw_boundary_count;
    }

    println!("──────────────────────────────────────────────────────────");
    let agg_prec = if total_eps > 0 {
        // count true-positive episodes: for each class, precision × episode_count
        // re-derive: total_precursor = sum over classes of (precision_i × eps_i)
        // but we already summed episodes; recompute from gt covered / total episodes
        total_rcl as f32 / total_eps as f32 * 100.0
    } else { 0.0 };
    let agg_compress = if total_eps > 0 { total_raw as f32 / total_eps as f32 } else { 0.0 };
    println!("{:>6}  {:>8}  {:>8}  {:>10.1}  {:>8.1}  {:>7.1}×",
        "TOTAL", total_eps, total_gt,
        agg_prec,
        total_rcl as f32 / total_gt as f32 * 100.0,
        agg_compress);

    if !report_only {
        println!();
        println!("  (paper-lock not defined for per-class protocol — use --report-only)");
    }
    Ok(())
}

// ── Flat-stream evaluation (original protocol) ────────────────────────────────

fn run_flat_stream(path: &str, report_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (observations, events) = load_radioml(path)
        .map_err(|e| format!("Failed to load RadioML dataset: {e}"))?;

    // Dataset variant diagnostic
    let is_gold_variant = events.len() != 102;
    if is_gold_variant {
        println!();
        println!("┌──────────────────────────────────────────────────────┐");
        println!("│                  Dataset Variant Note                │");
        println!("└──────────────────────────────────────────────────────┘");
        println!(" Ground-truth events detected in file order: {}", events.len());
        println!(" Paper Table IV expects: 102");
        println!();
        println!(" This is likely the GOLD_XYZ_OSC 1024-sample variant.");
        println!(" In the GOLD file (mod-class-major, SNR-minor ordering):");
        println!("   24 classes × 2 zero-crossings − 1 trailing = 47 GT events.");
        println!();
        println!(" For exact Table IV metric reproduction, use:");
        println!("   RML2018.01a.hdf5 (128 samples/cap, standard variant)");
        println!("   https://www.deepsig.ai/datasets");
        println!();
        println!(" Stage III will run with the {} GT events in this file.", events.len());
    }

    println!();
    println!("Running Stage III evaluation…");
    let result = run_stage_iii("RadioML 2018.01a", &observations, &events);
    result.print_summary();

    if report_only {
        println!();
        println!("  (--report-only: advisory check skipped)");
        return Ok(());
    }

    let config = PaperLockConfig::from_paper();
    println!();
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│   Paper Reference (Table IV — informational only)        │");
    println!("└──────────────────────────────────────────────────────────┘");
    println!(" Reference (receiver-residual): {} eps / {:.1}% prec / {}/{} recall",
        config.radioml_reference.episode_count,
        config.radioml_reference.precision * 100.0,
        config.radioml_reference.recall_min,
        102);
    println!(" Measured  (raw-IQ proxy)     : {} eps / {:.1}% prec / {}/{} recall",
        result.dsfb_episode_count,
        result.episode_precision * 100.0,
        result.recall_numerator,
        result.recall_denominator);
    println!();
    println!("  NOTE: Raw-IQ proxy != receiver-residual. Mismatch is expected.");
    println!("  See paper §F and hdf5_loader module docs for the correct protocol.");
    println!();

    // Advisory only — never exit(1) for RadioML raw-IQ results
    dsfb_rf::paper_lock::advisory_check_radioml(&result);
    Ok(())
}
