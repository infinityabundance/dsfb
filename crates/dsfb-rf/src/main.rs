//! DSFB-RF verification CLI.
//!
//! # Subcommand
//!
//! ## `paper-lock [PATH]`
//!
//! Runs the full DSFB Stage III evaluation on a RadioML 2018.01a HDF5 file
//! using the **amplitude-template residual** — the paper's §IX residual
//! construction projected to the amplitude domain.
//!
//! For each of the 24 modulation classes:
//!
//! 1. An amplitude template is built from calibration captures at SNR >= +10 dB:
//!    `template[n] = mean(|x_k[n]|)` over the healthy window.
//!
//! 2. The per-capture residual norm is:
//!    `||r(k)|| = RMS(|x_k[n]| - template[n])`
//!
//! 3. DSFB calibrates on the healthy window (`rho = 3*sigma`), then observes
//!    captures in degradation order (descending SNR).  The structural
//!    transition at the demodulation threshold triggers DSFB episodes.
//!
//! The amplitude domain (`|x| = sqrt(I^2 + Q^2)`) is phase-invariant —
//! immune to the random per-capture carrier phase offsets in RadioML — while
//! preserving each modulation's deterministic amplitude shape.
//!
//! Results are compared advisory-only against paper Table IV.  The paper used
//! carrier-synchronised decoder residuals; the amplitude-template residual
//! captures the same structural phenomenon via a different projection.
//!
//! # Usage
//!
//! ```text
//! cargo run --features std,paper_lock,hdf5_loader -- paper-lock [PATH]
//! ```

use dsfb_rf::hdf5_loader::load_radioml_per_class;
use dsfb_rf::paper_lock;
use dsfb_rf::pipeline::run_stage_iii;

// Canonical modulation-class names for RadioML 2018.01a (24 classes in order).
const MOD_NAMES: [&str; 24] = [
    "OOK", "4ASK", "8ASK", "BPSK", "QPSK", "8PSK", "16PSK", "32PSK",
    "16APSK", "32APSK", "16QAM", "32QAM", "64QAM", "128QAM", "256QAM",
    "AM-DSB-WC", "AM-DSB-SC", "FM", "GMSK",
    "OFDM-64", "OFDM-128", "OFDM-256", "OFDM-512", "OFDM-1024",
];

fn main() {
    let args: Vec<String> = std::env::args().take(8).collect();

    match args.get(1).map(String::as_str) {
        Some("paper-lock") => {
            let path = args.get(2).map(String::as_str).unwrap_or("data/RML2018.01a.hdf5");
            if let Err(e) = run_paper_lock(path) {
                eprintln!("Error: {e}");
                std::process::exit(2);
            }
        }
        Some(unknown) => {
            eprintln!("Unknown subcommand: {unknown:?}");
            print_usage(&args[0]);
            std::process::exit(1);
        }
        None => {
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(argv0: &str) {
    eprintln!("DSFB-RF Verification CLI");
    eprintln!("Usage: {argv0} <SUBCOMMAND> [args]");
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!("  paper-lock [PATH]    Verify RadioML 2018.01a against paper Table IV");
    eprintln!("                       PATH defaults to data/RML2018.01a.hdf5");
}

struct PaperLockAgg {
    n_classes: usize,
    total_raw_boundary: usize,
    total_episodes: usize,
    sum_true_positive: f32,
    total_recall_num: usize,
    total_recall_den: usize,
    sum_false_ep_rate: f32,
}

fn run_paper_lock(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    print_paper_lock_header(path);
    let class_data = load_radioml_per_class(path)
        .map_err(|e| format!("Failed to load RadioML dataset: {e}"))?;
    let n_classes = class_data.len();
    if n_classes > MOD_NAMES.len() {
        return Err(format!("Dataset has {n_classes} classes; MOD_NAMES has {}", MOD_NAMES.len()).into());
    }
    let agg = evaluate_all_classes(&class_data);
    let agg_precision = print_paper_lock_summary(&agg);
    paper_lock::advisory_check_radioml_aggregate(
        agg.total_episodes, agg_precision, agg.total_recall_num, agg.total_recall_den,
    );
    Ok(())
}

fn print_paper_lock_header(path: &str) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     DSFB-RF  RadioML 2018.01a  Stage III Evaluation      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("  Dataset : {path}");
    println!();
    println!("Running Stage III evaluation (per-class)…");
    println!("  {:>3}  {:12}  {:>5}  {:>8}  {:>8}  {:>8}  {:>8}",
        "cls", "modulation", "raw", "eps", "compr", "prec%", "recall");
}

fn evaluate_all_classes(
    class_data: &[(std::vec::Vec<dsfb_rf::pipeline::RfObservation>,
                    std::vec::Vec<dsfb_rf::pipeline::RegimeTransitionEvent>)],
) -> PaperLockAgg {
    let mut agg = PaperLockAgg {
        n_classes: class_data.len(),
        total_raw_boundary: 0, total_episodes: 0,
        sum_true_positive: 0.0, total_recall_num: 0, total_recall_den: 0,
        sum_false_ep_rate: 0.0,
    };
    for (cls, (obs, events)) in class_data.iter().enumerate() {
        let r = run_stage_iii("RadioML 2018.01a", obs, events);
        let tp = r.episode_precision * r.dsfb_episode_count as f32;
        let name = if cls < MOD_NAMES.len() { MOD_NAMES[cls] } else { "?" };
        println!("  {:>3}  {:12}  {:>5}  {:>5}  {:>6.1}×  {:>7.1}%  {:>3}/{:<3}",
            cls, name, r.raw_boundary_count, r.dsfb_episode_count,
            r.compression_factor, r.episode_precision * 100.0,
            r.recall_numerator, r.recall_denominator);
        agg.total_raw_boundary += r.raw_boundary_count;
        agg.total_episodes += r.dsfb_episode_count;
        agg.sum_true_positive += tp;
        agg.total_recall_num += r.recall_numerator;
        agg.total_recall_den += r.recall_denominator;
        agg.sum_false_ep_rate += r.false_episode_rate_clean;
    }
    agg
}

fn print_paper_lock_summary(agg: &PaperLockAgg) -> f32 {
    let agg_precision = if agg.total_episodes > 0 {
        agg.sum_true_positive / agg.total_episodes as f32
    } else { 0.0 };
    let agg_recall = if agg.total_recall_den > 0 {
        agg.total_recall_num as f32 / agg.total_recall_den as f32
    } else { 0.0 };
    let agg_false_ep = agg.sum_false_ep_rate / agg.n_classes as f32;
    let compression = if agg.total_episodes > 0 {
        agg.total_raw_boundary as f32 / agg.total_episodes as f32
    } else { 1.0 };
    println!("══════════════════════════════════════════════════════");
    println!(" DSFB-RF Stage III  —  RadioML 2018.01a (per-class aggregate)");
    println!("══════════════════════════════════════════════════════");
    println!(" Raw boundary events:    {:>8}", agg.total_raw_boundary);
    println!(" DSFB episodes:          {:>8}", agg.total_episodes);
    println!(" Compression:            {:>7.1}×", compression);
    println!(" Episode precision:      {:>7.1}%", agg_precision * 100.0);
    println!(" Recall:              {}/{} ({:.1}%)",
        agg.total_recall_num, agg.total_recall_den, agg_recall * 100.0);
    println!(" False ep. rate (clean): {:>7.1}%", agg_false_ep * 100.0);
    println!("══════════════════════════════════════════════════════");
    agg_precision
}

