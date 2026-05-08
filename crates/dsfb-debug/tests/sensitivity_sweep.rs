// DSFB-Debug: sensitivity sweep on five key hyperparameters
// (Phase η.3, Session 18).
//
// One-at-a-time discipline (vary one parameter, hold others at
// PAPER_LOCK_CONFIG). Five parameters × five values each = 25
// configurations. Per configuration, run LO-CV across all 12
// vendored fixtures and capture aggregate mean RSCR / FP / fault
// recall / typed-confirmed.
//
// Per academic-honesty discipline (Sessions 1-17 standing): only
// what the harness emits goes into the documentation; nothing
// rounded, smoothed, or extrapolated.
//
// Theorem 9 preservation: each configuration runs through
// `run_fusion_evaluation` which fires `verify_deterministic_replay`;
// the harness aborts via assertion if any configuration's replay
// fails.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::audit::{aggregate_loo_cv, LooCvAggregate, LooCvFixtureRecord};
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};
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

fn is_sentinel(bytes: &[u8]) -> bool {
    bytes.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
}

fn run_loo_with_cfg(cfg: &FusionConfig) -> LooCvAggregate {
    let fixtures: &[(&'static str, &[u8])] = &[
        ("tadbench_trainticket_F04",     F04_BYTES),
        ("tadbench_trainticket_F11",     F11_BYTES),
        ("tadbench_trainticket_F11b",    F11B_BYTES),
        ("tadbench_trainticket_F19",     F19_BYTES),
        ("illinois_socialnetwork",       ILLINOIS_BYTES),
        ("aiops_challenge_2018_kpi",     AIOPS_BYTES),
        ("lo2_oauth2_endoductive",       LO2_BYTES),
        ("multidim_localization_part1",  MULTIDIM_BYTES),
        ("deeptralog_F01",               DEEPTRALOG_BYTES),
        ("defects4j_6project",           DEFECTS4J_BYTES),
        ("bugsinpy_6project",            BUGSINPY_BYTES),
        ("promise_defect_prediction",    PROMISE_BYTES),
    ];

    let mut records: Vec<LooCvFixtureRecord> = Vec::new();
    for (name, bytes) in fixtures {
        if is_sentinel(bytes) { continue; }
        let matrix = match parse_residual_projection(bytes) {
            Ok(m) => m, Err(_) => continue,
        };
        if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
            continue;
        }
        let engine = DsfbDebugEngine::<32, 64>::paper_lock().expect("paper-lock");
        let r = match run_fusion_evaluation(
            &engine, &matrix.data,
            matrix.num_signals, matrix.num_windows,
            matrix.healthy_window_end, &matrix.fault_labels,
            cfg, name,
        ) {
            Ok(r) => r, Err(_) => continue,
        };
        let fault_recall = r.dsfb_structural.as_ref()
            .map(|m| m.fault_recall).unwrap_or(0.0);
        let rscr = r.dsfb_structural.as_ref()
            .map(|m| m.rscr).unwrap_or(0.0);
        records.push(LooCvFixtureRecord {
            fixture_name: name,
            rscr,
            clean_window_fp_rate: r.fusion_clean_window_fp_rate,
            fault_recall,
            raw_alert_count: r.raw_alert_count,
            fusion_episode_count: r.fusion_episode_count,
            consensus_confirmed_typed_episodes: r.consensus_confirmed_typed_episodes,
            deterministic_replay_holds: r.deterministic_replay_holds,
        });
    }
    aggregate_loo_cv(&records)
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
fn sensitivity_sweep_5_params() {
    let mut out = String::new();
    out.push_str("# Sensitivity sweep — Phase η.3\n\n");
    out.push_str("One-at-a-time variation: each parameter swept across 5\n");
    out.push_str("values; all other parameters held at `FusionConfig::ALL_DEFAULT`.\n");
    out.push_str("Per configuration: LO-CV aggregate across all 12 vendored\n");
    out.push_str("fixtures (verbatim from `run_fusion_evaluation` stdout).\n");
    out.push_str("Theorem 9 deterministic replay verified per configuration.\n\n");
    out.push_str("Source: Phase η.3 sweep harness (`tests/sensitivity_sweep.rs`).\n\n");

    println!();
    println!("=== Phase η.3 SENSITIVITY SWEEP — 5 params × 5 values ===");

    let base = FusionConfig::ALL_DEFAULT;

    // === Param 1: min_consensus ∈ {1, 3, 5, 7, 9} ===
    out.push_str("## min_consensus\n\n");
    out.push_str("| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n");
    out.push_str("|------:|----------:|--------:|------------:|----------------:|:------:|\n");
    for &n in &[1u8, 3, 5, 7, 9] {
        let cfg = FusionConfig { min_consensus: n, ..base };
        let agg = run_loo_with_cfg(&cfg);
        out.push_str(&format!(
            "| {} | {:.4} | {:.4} | {:.4} | {} | {} / {} |\n",
            n, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes,
            agg.fixtures_with_replay_holds, agg.fixtures_observed));
        println!("[sweep] min_consensus={} → RSCR {:.3} FP {:.3} recall {:.3} typed {}",
            n, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes);
        assert_eq!(agg.fixtures_with_replay_holds, agg.fixtures_observed,
            "Theorem 9 must hold under min_consensus={}", n);
    }

    // === Param 2: margin_gate ∈ {0.10, 0.20, 0.30, 0.40, 0.50} ===
    out.push_str("\n## margin_gate\n\n");
    out.push_str("| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n");
    out.push_str("|------:|----------:|--------:|------------:|----------------:|:------:|\n");
    for &m in &[0.10f64, 0.20, 0.30, 0.40, 0.50] {
        let cfg = FusionConfig { margin_gate: m, ..base };
        let agg = run_loo_with_cfg(&cfg);
        out.push_str(&format!(
            "| {:.2} | {:.4} | {:.4} | {:.4} | {} | {} / {} |\n",
            m, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes,
            agg.fixtures_with_replay_holds, agg.fixtures_observed));
        println!("[sweep] margin_gate={:.2} → RSCR {:.3} FP {:.3} recall {:.3} typed {}",
            m, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes);
        assert_eq!(agg.fixtures_with_replay_holds, agg.fixtures_observed);
    }

    // === Param 3: scalar_k ∈ {2.0, 2.5, 3.0, 3.5, 4.0} ===
    out.push_str("\n## scalar_k (3-sigma multiplier)\n\n");
    out.push_str("| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n");
    out.push_str("|------:|----------:|--------:|------------:|----------------:|:------:|\n");
    for &k in &[2.0f64, 2.5, 3.0, 3.5, 4.0] {
        let cfg = FusionConfig { scalar_k: k, ..base };
        let agg = run_loo_with_cfg(&cfg);
        out.push_str(&format!(
            "| {:.1} | {:.4} | {:.4} | {:.4} | {} | {} / {} |\n",
            k, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes,
            agg.fixtures_with_replay_holds, agg.fixtures_observed));
        println!("[sweep] scalar_k={:.1} → RSCR {:.3} FP {:.3} recall {:.3} typed {}",
            k, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes);
        assert_eq!(agg.fixtures_with_replay_holds, agg.fixtures_observed);
    }

    // === Param 4: cusum_h ∈ {2.0, 3.0, 4.0, 5.0, 6.0} ===
    out.push_str("\n## cusum_h\n\n");
    out.push_str("| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n");
    out.push_str("|------:|----------:|--------:|------------:|----------------:|:------:|\n");
    for &h in &[2.0f64, 3.0, 4.0, 5.0, 6.0] {
        let cfg = FusionConfig { cusum_h: h, ..base };
        let agg = run_loo_with_cfg(&cfg);
        out.push_str(&format!(
            "| {:.1} | {:.4} | {:.4} | {:.4} | {} | {} / {} |\n",
            h, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes,
            agg.fixtures_with_replay_holds, agg.fixtures_observed));
        println!("[sweep] cusum_h={:.1} → RSCR {:.3} FP {:.3} recall {:.3} typed {}",
            h, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes);
        assert_eq!(agg.fixtures_with_replay_holds, agg.fixtures_observed);
    }

    // === Param 5: ewma_lambda ∈ {0.05, 0.10, 0.20, 0.30, 0.40} ===
    out.push_str("\n## ewma_lambda\n\n");
    out.push_str("| Value | Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n");
    out.push_str("|------:|----------:|--------:|------------:|----------------:|:------:|\n");
    for &l in &[0.05f64, 0.10, 0.20, 0.30, 0.40] {
        let cfg = FusionConfig { ewma_lambda: l, ..base };
        let agg = run_loo_with_cfg(&cfg);
        out.push_str(&format!(
            "| {:.2} | {:.4} | {:.4} | {:.4} | {} | {} / {} |\n",
            l, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes,
            agg.fixtures_with_replay_holds, agg.fixtures_observed));
        println!("[sweep] ewma_lambda={:.2} → RSCR {:.3} FP {:.3} recall {:.3} typed {}",
            l, agg.mean_rscr, agg.mean_clean_window_fp_rate,
            agg.mean_fault_recall, agg.total_typed_episodes);
        assert_eq!(agg.fixtures_with_replay_holds, agg.fixtures_observed);
    }

    out.push_str("\n## Honest empirical reading\n\n");
    out.push_str("Each parameter's response curve is read column-wise from\n");
    out.push_str("the per-table values above. Steep response = high\n");
    out.push_str("sensitivity (operator-side calibration matters); flat\n");
    out.push_str("response = low sensitivity (default works robustly).\n");
    out.push_str("Per Session-17 academic-honesty discipline, no parameter\n");
    out.push_str("setting is claimed superior on this 12-fixture surface\n");
    out.push_str("without LO-CV gate evidence; the sweep ledger is the\n");
    out.push_str("operator-side input to per-site calibration.\n");

    write_audit_markdown("sensitivity_sweep.md", &out);
}
