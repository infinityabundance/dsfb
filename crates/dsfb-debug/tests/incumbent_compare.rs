// DSFB-Debug: incumbent-comparison run on the vendored F-11 fixture.
//
// Runs DSFB-Debug + scalar-threshold + CUSUM + EWMA on the same
// residual-projection-v2 matrix from the TrainTicket-Anomaly F-11
// fixture. Captures raw alert counts, episode counts, fault recall,
// clean-window FP rate, and wall-clock processing time. Emits the
// comparison matrix to stdout in a structure ready for verbatim
// inclusion in `docs/incumbent_comparison.md` and paper §13.8.
//
// Per academic-honesty discipline: the test reports the numbers
// actually emitted by the harness; nothing is rounded, smoothed, or
// extrapolated. If the F-11 fixture is the sentinel form on a fresh
// checkout, the test prints `[skip]` and returns Ok.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::time::Instant;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::error::DsfbError;
use dsfb_debug::incumbent_baselines::{cusum, ewma, scalar_threshold, DetectorOutput};
use dsfb_debug::real_data::{evaluate_real_dataset, MANIFEST_TADBENCH_F11};
use dsfb_debug::DsfbDebugEngine;

const F11_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");

#[test]
fn comparison_matrix_on_f11_fixture() {
    // Detect sentinel-form: exit gracefully.
    if F11_BYTES.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
    {
        eprintln!("[skip] F-11 fixture is the sentinel form. \
                   Run extraction recipe in data/README.md to populate.");
        return;
    }

    let matrix = parse_residual_projection(F11_BYTES)
        .expect("F-11 fixture should parse cleanly");
    if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
        eprintln!("[skip] F-11 fixture parsed empty (sentinel-form variant).");
        return;
    }

    // ---------- DSFB-Debug ----------
    let engine = DsfbDebugEngine::<32, 64>::paper_lock()
        .expect("paper-lock engine creation should succeed");
    let dsfb_t0 = Instant::now();
    let dsfb_eval = evaluate_real_dataset(&engine, &MANIFEST_TADBENCH_F11, F11_BYTES);
    let dsfb_dt = dsfb_t0.elapsed();
    let dsfb_eval = match dsfb_eval {
        Ok(e) => e,
        Err(DsfbError::MissingRealData) => {
            eprintln!("[skip] real-data eval refused at the engine level.");
            return;
        }
        Err(other) => panic!("DSFB-Debug evaluation failed: {other:?}"),
    };

    // ---------- Incumbent baselines ----------
    let pred_w = engine.config().episode_precision_window;

    let scalar_t0 = Instant::now();
    let scalar = scalar_threshold(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w);
    let scalar_dt = scalar_t0.elapsed();

    let cusum_t0 = Instant::now();
    let cusum_out = cusum(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w, 4.0);
    let cusum_dt = cusum_t0.elapsed();

    let ewma_t0 = Instant::now();
    let ewma_out = ewma(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w, 0.2, 3.0);
    let ewma_dt = ewma_t0.elapsed();

    // ---------- Emit comparison matrix ----------
    println!();
    println!("=== INCUMBENT COMPARISON MATRIX — TADBench/TrainTicket F-11 ===");
    println!("fixture: tadbench_trainticket_F11");
    println!("upstream_archive_sha256: 18456279cdcbc66b020bddd117a79ff453137fbc60f88cba81f2609fc1a74403");
    println!("fixture_sha256: 07c8f08558c00d62c48fa5833aef9ecaf7324968f2028ce244e4c8e5248512c3");
    println!("num_signals: {}, num_windows: {}, healthy_window_end: {}",
             matrix.num_signals, matrix.num_windows, matrix.healthy_window_end);
    println!();
    println!("| Detector               | Raw alerts | Episodes | RSCR  | Fault recall | Clean-window FP rate | Wall-clock (µs) | Deterministic |");
    println!("|------------------------|-----------:|---------:|------:|-------------:|---------------------:|----------------:|--------------:|");

    println!("| **dsfb-debug v0.2**    | {:>10} | {:>8} | {:>5.2} | {:>12.4} | {:>20.6} | {:>15} | yes (Theorem 9) |",
             dsfb_eval.metrics.raw_anomaly_count,
             dsfb_eval.episode_count,
             dsfb_eval.metrics.rscr,
             dsfb_eval.metrics.fault_recall,
             dsfb_eval.metrics.clean_window_false_episode_rate,
             dsfb_dt.as_micros());

    print_baseline_row(&scalar, scalar_dt.as_micros());
    print_baseline_row(&cusum_out, cusum_dt.as_micros());
    print_baseline_row(&ewma_out, ewma_dt.as_micros());

    println!();
    println!("Notes:");
    println!("  * Flat-detector RSCR is reported as 1.0 (no episode aggregation)");
    println!("    for symmetry; their per-window alert stream is what an");
    println!("    operator would actually consume.");
    println!("  * On this fixture (steady-state regressed-config run, no");
    println!("    explicit fault-window labels), fault_recall = vacuous 1.0");
    println!("    for any detector that fires at least one alert near a");
    println!("    labeled fault. The numerical comparison that signals real");
    println!("    differentiation here is **clean-window FP rate** and");
    println!("    **raw alert count vs episode count**.");
    println!();

    // Range-bound invariants only — no exact-number assertion.
    assert!(dsfb_eval.metrics.fault_recall >= 0.0
            && dsfb_eval.metrics.fault_recall <= 1.0);
    assert!(scalar.raw_alert_count >= scalar.alert_windows);
    assert!(cusum_out.raw_alert_count >= cusum_out.alert_windows);
    assert!(ewma_out.raw_alert_count >= ewma_out.alert_windows);
    // DSFB-Debug must produce at most as many episodes as raw anomalies.
    assert!(dsfb_eval.metrics.dsfb_episode_count
            <= dsfb_eval.metrics.raw_anomaly_count.max(1));
}

fn print_baseline_row(d: &DetectorOutput, micros: u128) {
    println!("| {:<22} | {:>10} | {:>8} | {:>5.2} | {:>12.4} | {:>20.6} | {:>15} | yes |",
             d.detector_name,
             d.raw_alert_count,
             d.episode_count,
             d.rscr(),
             d.fault_recall(),
             d.clean_window_fp_rate(),
             micros);
}
