//! W_pred Sensitivity Sweep — Measured on Real RadioML 2018.01a GOLD
//!
//! Companion to `wpred_sweep.rs` (synthetic AR(1)). This example:
//!
//! 1. Loads the RadioML 2018.01a GOLD HDF5 at the hard-coded default path
//!    `data/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5` as a **flat stream**
//!    (via `load_radioml`) — the same protocol that produces paper Table IV.
//! 2. Runs the DSFB Stage III pipeline once.
//! 3. Recomputes episode **precision** for `W_pred ∈ {3, 5, 7}` post-hoc
//!    on the fixed episode stream — this is the precision metric defined in
//!    `finalise_episodes`: an episode is a precursor iff a GT event lies in
//!    `[close_k, close_k + W_pred]`.
//! 4. Additionally runs the per-class protocol (`load_radioml_per_class`)
//!    and reports its aggregate W-sweep for completeness (fig_149 protocol).
//! 5. Prints two tables suitable for Table V fill-in and §VII.G reference.
//!
//! ## Why post-hoc recomputation of precursor labels
//!
//! `WPRED` in `src/pipeline.rs` is a post-hoc precursor-window parameter;
//! it does not change engine behaviour, only the precursor label. This is
//! exactly the sensitivity axis Table V reports for the real-data columns.
//! The engine's internal `DSA_WINDOW_W = 10` is unchanged.
//!
//! ## Observer-only framing (paper guardrail)
//!
//! DSFB reads the amplitude-template Wasserstein-2 residual that a
//! carrier-synchronised demodulator already computes and usually discards.
//! This sweep does not replace, compete with, or detect earlier than any
//! modulation classifier — it structures the producer's residual into
//! typed episodes whose precursor window is the sensitivity axis under test.
//!
//! ## Usage
//!
//! ```text
//! cargo run --release --features std,serde,hdf5_loader \
//!     --example wpred_sweep_real
//! ```
//!
//! Est. wall-clock on an 8-core x86-64: ≈45 min (dominated by the 21 GB
//! X-dataset read + per-capture sorted-amplitude residual pass).

#[cfg(all(feature = "std", feature = "serde", feature = "hdf5_loader"))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    use dsfb_rf::hdf5_loader::{load_radioml, load_radioml_per_class};
    use dsfb_rf::pipeline::{run_stage_iii, EvaluationResult};

    let path = "data/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5";
    println!();
    println!("══════════════════════════════════════════════════════════════════════");
    println!(" DSFB-RF  W_pred Sensitivity Sweep on Real RadioML 2018.01a GOLD");
    println!(" Dataset: {}", path);
    println!("══════════════════════════════════════════════════════════════════════");

    // ─── Flat-stream protocol (matches paper Table IV and Table V W=5) ──
    println!();
    println!(" ─── Protocol A: Flat stream (matches paper Table IV) ───────────────");
    let (obs_flat, events_flat) = load_radioml(path)?;
    let result_flat: EvaluationResult = run_stage_iii("radioml_flat", &obs_flat, &events_flat);
    result_flat.print_summary();

    let (flat_w3, flat_w5, flat_w7, flat_total) =
        recompute_precision_sweep(&result_flat, &events_flat);

    // ─── Per-class protocol (matches fig_149 framing) ───────────────────
    println!();
    println!(" ─── Protocol B: Per-class (matches fig_149 framing) ────────────────");
    let per_class = load_radioml_per_class(path)?;
    let n_classes = per_class.len();
    let mut pc_total = 0usize;
    let mut pc_w3 = 0usize;
    let mut pc_w5 = 0usize;
    let mut pc_w7 = 0usize;
    let mut pc_recall_num = 0usize;
    let mut pc_recall_den = 0usize;
    for (cls, (obs, events)) in per_class.iter().enumerate() {
        let result = run_stage_iii("radioml_per_class", obs, events);
        let (w3, w5, w7, total) = recompute_precision_sweep(&result, events);
        pc_total += total;
        pc_w3 += w3;
        pc_w5 += w5;
        pc_w7 += w7;
        pc_recall_num += result.recall_numerator;
        pc_recall_den += result.recall_denominator;
        if cls < 3 || cls >= n_classes - 3 || cls % 6 == 0 {
            println!(
                "   class {:2}: {:>4} eps  precision(W=5)={:>5.1}%  recall={}/{}",
                cls,
                result.dsfb_episode_count,
                result.episode_precision * 100.0,
                result.recall_numerator,
                result.recall_denominator
            );
        }
    }

    let prec = |p: usize, total: usize| -> f32 {
        if total == 0 { 0.0 } else { p as f32 / total as f32 }
    };

    println!();
    println!(" ─── Table V Fill (flat-stream protocol — paper Table IV methodology) ");
    println!();
    println!(" ┌────────────┬──────────────────────────────┬───────────────────────┐");
    println!(" │  W_pred    │  RadioML precision (measured) │  Precursor / total   │");
    println!(" ├────────────┼──────────────────────────────┼───────────────────────┤");
    println!(" │  W = 3     │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(flat_w3, flat_total) * 100.0, flat_w3, flat_total);
    println!(" │  W = 5 ✦   │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(flat_w5, flat_total) * 100.0, flat_w5, flat_total);
    println!(" │  W = 7     │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(flat_w7, flat_total) * 100.0, flat_w7, flat_total);
    println!(" └────────────┴──────────────────────────────┴───────────────────────┘");
    println!();

    println!(" ─── Per-class protocol companion (24 classes × B=128) ──────────────");
    println!(" │  W = 3     │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(pc_w3, pc_total) * 100.0, pc_w3, pc_total);
    println!(" │  W = 5     │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(pc_w5, pc_total) * 100.0, pc_w5, pc_total);
    println!(" │  W = 7     │  {:>5.1}%                      │  {:>5} / {:<5}       │",
        prec(pc_w7, pc_total) * 100.0, pc_w7, pc_total);
    println!();
    let rec_pct = if pc_recall_den > 0 {
        pc_recall_num as f32 / pc_recall_den as f32 * 100.0
    } else { 0.0 };
    println!(" Per-class aggregate recall: {}/{} = {:.1}%", pc_recall_num, pc_recall_den, rec_pct);
    println!();
    println!(" Notes:");
    println!("   1. Episode set fixed; W_pred changes only the precursor label window.");
    println!("   2. Flat-stream protocol matches paper Table IV methodology (Table V W=5 = 73.6%).");
    println!("   3. Per-class protocol matches §VII.G / fig_149 framing.");
    println!("   4. Observer-only: DSFB reads the amplitude-template residual the");
    println!("      upstream demodulator already produces; no competition claim.");
    println!("══════════════════════════════════════════════════════════════════════");

    let summary = serde_json::json!({
        "dataset": "RadioML 2018.01a GOLD (real, local)",
        "flat_stream": {
            "total_episodes": flat_total,
            "precision_w3": prec(flat_w3, flat_total),
            "precision_w5": prec(flat_w5, flat_total),
            "precision_w7": prec(flat_w7, flat_total),
            "episode_count": result_flat.dsfb_episode_count,
            "recall_numerator": result_flat.recall_numerator,
            "recall_denominator": result_flat.recall_denominator,
        },
        "per_class": {
            "n_classes": n_classes,
            "total_episodes": pc_total,
            "precision_w3": prec(pc_w3, pc_total),
            "precision_w5": prec(pc_w5, pc_total),
            "precision_w7": prec(pc_w7, pc_total),
            "recall_numerator": pc_recall_num,
            "recall_denominator": pc_recall_den,
        },
        "note": "Observer-only precision: fraction of closed episodes with a GT event in [close_k, close_k+W].",
    });
    let out_path = "wpred_sweep_real.json";
    std::fs::write(out_path, serde_json::to_string_pretty(&summary)?)?;
    println!(" Summary JSON: {}", out_path);

    Ok(())
}

/// Post-hoc precision recomputation for W_pred ∈ {3,5,7} on a fixed
/// episode stream. Returns (hits_w3, hits_w5, hits_w7, total_closed_eps).
#[cfg(all(feature = "std", feature = "serde", feature = "hdf5_loader"))]
fn recompute_precision_sweep(
    result: &dsfb_rf::pipeline::EvaluationResult,
    events: &[dsfb_rf::pipeline::RegimeTransitionEvent],
) -> (usize, usize, usize, usize) {
    let mut total = 0usize;
    let mut w3 = 0usize;
    let mut w5 = 0usize;
    let mut w7 = 0usize;
    for ep in &result.episodes {
        if let Some(close) = ep.close_k {
            total += 1;
            if events.iter().any(|ev| close <= ev.k && ev.k <= close + 3) { w3 += 1; }
            if events.iter().any(|ev| close <= ev.k && ev.k <= close + 5) { w5 += 1; }
            if events.iter().any(|ev| close <= ev.k && ev.k <= close + 7) { w7 += 1; }
        }
    }
    (w3, w5, w7, total)
}

#[cfg(not(all(feature = "std", feature = "serde", feature = "hdf5_loader")))]
fn main() {
    eprintln!("wpred_sweep_real requires --features std,serde,hdf5_loader");
}
