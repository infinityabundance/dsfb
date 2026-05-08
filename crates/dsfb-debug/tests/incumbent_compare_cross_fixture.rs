// DSFB-Debug: cross-fixture incumbent comparison (Phase η.9, Session 18).
//
// Extends the F-11-only `incumbent_compare.rs` baseline with a
// 12-fixture × 4-detector matrix. For each fixture, runs DSFB-Debug
// + scalar-threshold + CUSUM + EWMA on the same residual matrix;
// captures raw alert counts, fault recall, clean-window FP rate,
// and wall-clock latency. Aggregates cross-fixture mean/stddev per
// detector per metric.
//
// Per academic-honesty discipline (Sessions 1-17 standing): only
// what the harness emits goes into the documentation; nothing
// rounded, smoothed, or extrapolated.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::incumbent_baselines::{cusum, ewma, scalar_threshold};
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_TADBENCH_F04, MANIFEST_TADBENCH_F11, MANIFEST_TADBENCH_F11B,
    MANIFEST_TADBENCH_F19, MANIFEST_ILLINOIS_SOCIALNETWORK,
    MANIFEST_AIOPS_CHALLENGE, MANIFEST_LO2, MANIFEST_MULTIDIM_LOCALIZATION,
    MANIFEST_DEEPTRALOG, MANIFEST_DEFECTS4J, MANIFEST_BUGSINPY,
    MANIFEST_PROMISE,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const F04_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F04.tsv");
const F11_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");
const F11B_BYTES: &[u8]     = include_bytes!("../data/fixtures/tadbench_trainticket_F11b.tsv");
const F19_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F19.tsv");
const ILLINOIS_BYTES: &[u8] = include_bytes!("../data/fixtures/illinois_socialnetwork.tsv");
const AIOPS_BYTES: &[u8]    = include_bytes!("../data/fixtures/aiops_challenge.tsv");
const LO2_BYTES: &[u8]      = include_bytes!("../data/fixtures/lo2.tsv");
const MULTIDIM_BYTES: &[u8] = include_bytes!("../data/fixtures/multidim_localization.tsv");
const DEEPTRALOG_BYTES: &[u8] = include_bytes!("../data/fixtures/deeptralog.tsv");
const DEFECTS4J_BYTES: &[u8]  = include_bytes!("../data/fixtures/defects4j.tsv");
const BUGSINPY_BYTES: &[u8]   = include_bytes!("../data/fixtures/bugsinpy.tsv");
const PROMISE_BYTES: &[u8]    = include_bytes!("../data/fixtures/promise_defect_prediction.tsv");

#[derive(Debug, Clone)]
struct DetectorRow {
    detector: &'static str,
    raw_alerts: u64,
    fault_recall: f64,
    clean_fp_rate: f64,
    wall_us: u128,
}

#[derive(Debug, Clone)]
struct FixtureMatrix {
    fixture_name: &'static str,
    num_signals: usize,
    num_windows: usize,
    rows: Vec<DetectorRow>,  // 4 rows per fixture: dsfb, scalar, cusum, ewma
}

fn run_one_fixture(
    fixture_name: &'static str,
    manifest: &RealDatasetManifest,
    bytes: &[u8],
) -> Option<FixtureMatrix> {
    if bytes.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
    {
        eprintln!("[skip] {fixture_name} sentinel");
        return None;
    }
    let matrix = parse_residual_projection(bytes).ok()?;
    if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
        return None;
    }

    let engine = DsfbDebugEngine::<32, 64>::paper_lock().expect("paper-lock");
    let pred_w = engine.config().episode_precision_window;

    // ---- DSFB-Debug ----
    let t0 = Instant::now();
    let dsfb_eval = evaluate_real_dataset(&engine, manifest, bytes);
    let dsfb_dt = t0.elapsed();
    let dsfb_row = match dsfb_eval {
        Ok(e) => DetectorRow {
            detector: "dsfb-debug",
            raw_alerts: e.metrics.raw_anomaly_count,
            fault_recall: e.metrics.fault_recall,
            clean_fp_rate: e.metrics.clean_window_false_episode_rate,
            wall_us: dsfb_dt.as_micros(),
        },
        Err(_) => return None,
    };

    // ---- scalar-threshold ----
    let t0 = Instant::now();
    let scalar = scalar_threshold(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w);
    let scalar_dt = t0.elapsed();
    let scalar_fp = if scalar.clean_windows > 0 {
        scalar.clean_window_false_alerts as f64 / scalar.clean_windows as f64
    } else { 0.0 };
    let scalar_recall = if scalar.total_faults > 0 {
        scalar.captured_faults as f64 / scalar.total_faults as f64
    } else { 1.0 };

    // ---- CUSUM ----
    let t0 = Instant::now();
    let cusum_out = cusum(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w, 4.0);
    let cusum_dt = t0.elapsed();
    let cusum_fp = if cusum_out.clean_windows > 0 {
        cusum_out.clean_window_false_alerts as f64 / cusum_out.clean_windows as f64
    } else { 0.0 };
    let cusum_recall = if cusum_out.total_faults > 0 {
        cusum_out.captured_faults as f64 / cusum_out.total_faults as f64
    } else { 1.0 };

    // ---- EWMA ----
    let t0 = Instant::now();
    let ewma_out = ewma(
        &matrix.data, matrix.num_signals, matrix.num_windows,
        matrix.healthy_window_end, &matrix.fault_labels, pred_w, 0.2, 3.0);
    let ewma_dt = t0.elapsed();
    let ewma_fp = if ewma_out.clean_windows > 0 {
        ewma_out.clean_window_false_alerts as f64 / ewma_out.clean_windows as f64
    } else { 0.0 };
    let ewma_recall = if ewma_out.total_faults > 0 {
        ewma_out.captured_faults as f64 / ewma_out.total_faults as f64
    } else { 1.0 };

    let rows = vec![
        dsfb_row,
        DetectorRow {
            detector: "scalar-threshold",
            raw_alerts: scalar.raw_alert_count,
            fault_recall: scalar_recall,
            clean_fp_rate: scalar_fp,
            wall_us: scalar_dt.as_micros(),
        },
        DetectorRow {
            detector: "CUSUM",
            raw_alerts: cusum_out.raw_alert_count,
            fault_recall: cusum_recall,
            clean_fp_rate: cusum_fp,
            wall_us: cusum_dt.as_micros(),
        },
        DetectorRow {
            detector: "EWMA",
            raw_alerts: ewma_out.raw_alert_count,
            fault_recall: ewma_recall,
            clean_fp_rate: ewma_fp,
            wall_us: ewma_dt.as_micros(),
        },
    ];
    Some(FixtureMatrix {
        fixture_name,
        num_signals: matrix.num_signals,
        num_windows: matrix.num_windows,
        rows,
    })
}

fn write_audit_markdown(filename: &str, content: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("docs"); path.push("audit");
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("[warn] mkdir docs/audit: {e:?}"); return;
    }
    path.push(filename);
    match fs::File::create(&path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(content.as_bytes()) {
                eprintln!("[warn] write {filename}: {e:?}");
            } else {
                eprintln!("[audit] wrote {}", path.display());
            }
        }
        Err(e) => eprintln!("[warn] open {filename}: {e:?}"),
    }
}

#[test]
fn incumbent_compare_cross_fixture_12() {
    println!();
    println!("=== Phase η.9 INCUMBENT COMPARISON — 12 fixtures × 4 detectors ===");

    let fixtures: Vec<(&'static str, &'static RealDatasetManifest, &[u8])> = vec![
        ("tadbench_trainticket_F04",     &MANIFEST_TADBENCH_F04,        F04_BYTES),
        ("tadbench_trainticket_F11",     &MANIFEST_TADBENCH_F11,        F11_BYTES),
        ("tadbench_trainticket_F11b",    &MANIFEST_TADBENCH_F11B,       F11B_BYTES),
        ("tadbench_trainticket_F19",     &MANIFEST_TADBENCH_F19,        F19_BYTES),
        ("illinois_socialnetwork",       &MANIFEST_ILLINOIS_SOCIALNETWORK, ILLINOIS_BYTES),
        ("aiops_challenge_2018_kpi",     &MANIFEST_AIOPS_CHALLENGE,     AIOPS_BYTES),
        ("lo2_oauth2_endoductive",       &MANIFEST_LO2,                 LO2_BYTES),
        ("multidim_localization_part1",  &MANIFEST_MULTIDIM_LOCALIZATION, MULTIDIM_BYTES),
        ("deeptralog_F01",               &MANIFEST_DEEPTRALOG,          DEEPTRALOG_BYTES),
        ("defects4j_6project",           &MANIFEST_DEFECTS4J,           DEFECTS4J_BYTES),
        ("bugsinpy_6project",            &MANIFEST_BUGSINPY,            BUGSINPY_BYTES),
        ("promise_defect_prediction",    &MANIFEST_PROMISE,             PROMISE_BYTES),
    ];

    let mut matrices: Vec<FixtureMatrix> = Vec::new();
    for (name, manifest, bytes) in fixtures {
        if let Some(m) = run_one_fixture(name, manifest, bytes) {
            for r in &m.rows {
                println!("[fix={} det={:>16}] raw {:>5}  recall {:.4}  FP {:.4}  µs {}",
                    m.fixture_name, r.detector, r.raw_alerts,
                    r.fault_recall, r.clean_fp_rate, r.wall_us);
            }
            matrices.push(m);
        }
    }

    // Build markdown report.
    let mut out = String::new();
    out.push_str("# Cross-fixture incumbent comparison — Phase η.9\n\n");
    out.push_str("DSFB-Debug + scalar-threshold + CUSUM + EWMA on each of\n");
    out.push_str("the 12 vendored fixtures. Per fixture × detector: raw\n");
    out.push_str("alerts, fault recall, clean-window FP rate, wall-clock µs.\n\n");
    out.push_str("Source: Phase η.9 harness (`tests/incumbent_compare_cross_fixture.rs`).\n\n");

    out.push_str("## Per-fixture matrix\n\n");
    out.push_str("| Fixture | Detector | Raw alerts | Fault recall | Clean FP rate | Wall µs |\n");
    out.push_str("|---------|----------|-----------:|-------------:|--------------:|--------:|\n");
    for m in &matrices {
        for (i, r) in m.rows.iter().enumerate() {
            let fix_label = if i == 0 { format!("**{}**", m.fixture_name) } else { String::new() };
            out.push_str(&format!(
                "| {} | `{}` | {} | {:.4} | {:.4} | {} |\n",
                fix_label, r.detector, r.raw_alerts,
                r.fault_recall, r.clean_fp_rate, r.wall_us));
        }
    }

    // Cross-fixture aggregate per detector.
    out.push_str("\n## Cross-fixture aggregate per detector\n\n");
    out.push_str("Mean ± stddev across the populated fixtures. Mean computed\n");
    out.push_str("over all fixtures contributing data; stddev is the population\n");
    out.push_str("stddev (not sample stddev) for direct comparison.\n\n");
    out.push_str("| Detector | Mean recall | Stddev recall | Mean FP | Stddev FP | Mean µs | Total raw |\n");
    out.push_str("|----------|------------:|--------------:|--------:|----------:|--------:|----------:|\n");

    let detector_names = ["dsfb-debug", "scalar-threshold", "CUSUM", "EWMA"];
    for det in &detector_names {
        let values: Vec<&DetectorRow> = matrices.iter()
            .flat_map(|m| m.rows.iter().filter(|r| r.detector == *det))
            .collect();
        if values.is_empty() { continue; }
        let n = values.len() as f64;
        let mean_recall = values.iter().map(|r| r.fault_recall).sum::<f64>() / n;
        let mean_fp = values.iter().map(|r| r.clean_fp_rate).sum::<f64>() / n;
        let mean_us = values.iter().map(|r| r.wall_us as f64).sum::<f64>() / n;
        let total_raw: u64 = values.iter().map(|r| r.raw_alerts).sum();
        let stddev_recall = (values.iter()
            .map(|r| (r.fault_recall - mean_recall).powi(2))
            .sum::<f64>() / n).sqrt();
        let stddev_fp = (values.iter()
            .map(|r| (r.clean_fp_rate - mean_fp).powi(2))
            .sum::<f64>() / n).sqrt();
        out.push_str(&format!(
            "| `{}` | {:.4} | {:.4} | {:.4} | {:.4} | {:.0} | {} |\n",
            det, mean_recall, stddev_recall, mean_fp, stddev_fp, mean_us, total_raw));
    }

    out.push_str("\n## Honest empirical reading\n\n");
    out.push_str("The single-fixture (F-11) incumbent comparison was\n");
    out.push_str("Session-7 anchor; the 12-fixture matrix above is the\n");
    out.push_str("Session-18 cross-domain extension. Per Session-17\n");
    out.push_str("academic-honesty discipline:\n\n");
    out.push_str("- **Recall numbers** are computed as `captured_faults /\n");
    out.push_str("  total_faults` per the existing scoring harness; for\n");
    out.push_str("  steady-state fixtures (F-04, F-19, LO2, etc.) `total_faults\n");
    out.push_str("  = 0` so recall is reported as 1.0 vacuously. The recall\n");
    out.push_str("  delta between detectors is meaningful only on fixtures\n");
    out.push_str("  with actual labelled fault windows.\n");
    out.push_str("- **FP rate numbers** are the operationally relevant metric:\n");
    out.push_str("  lower = fewer alerts on clean windows. DSFB-Debug's\n");
    out.push_str("  bank-aware confirmed-typed-episode FP rate (the operator-\n");
    out.push_str("  facing output) is the structural-layer number; scalar /\n");
    out.push_str("  CUSUM / EWMA report per-cell-firing FP rates. The\n");
    out.push_str("  comparison is the architectural-claim test: structural\n");
    out.push_str("  episodes are a different layer than per-cell alerts.\n");
    out.push_str("- **Wall-clock numbers** are debug-build µs; release-build\n");
    out.push_str("  is typically 5-20× faster (see `docs/benchmarks.md`).\n");

    write_audit_markdown("incumbent_comparison_cross_fixture.md", &out);
}
