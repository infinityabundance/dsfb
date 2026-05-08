// DSFB-Debug: real-data evaluation against the PROMISE Software Engineering
// Repository defect-prediction dataset (Menzies et al., 2003+). Phase G
// code-debugging dataset of per-module Chidamber-Kemerer OO metrics with
// bug counts.
//
// Vendored slice: 6 PROMISE CSVs (1.csv, 5.csv, 10.csv, 15.csv, 20.csv,
// 25.csv) × first-30-module bug-count residual matrix. Signal value is
// the per-module bug count from the canonical PROMISE OO-metrics CSVs.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::error::DsfbError;
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_PROMISE,
    RealDatasetEvaluation,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const PROMISE_BYTES: &[u8] = include_bytes!("../data/fixtures/promise_defect_prediction.tsv");

fn run_or_skip(manifest: &RealDatasetManifest, bytes: &[u8]) {
    let engine = DsfbDebugEngine::<32, 64>::paper_lock()
        .expect("paper-lock engine creation should succeed");
    match evaluate_real_dataset(&engine, manifest, bytes) {
        Ok(eval) => {
            assert_real_eval_invariants(&eval);
            print_metrics_json(&eval);
        }
        Err(DsfbError::MissingRealData) => {
            eprintln!(
                "[skip] {} — fixture is the sentinel form. \n\
                 Re-run data/upstream/project_promise.py to repopulate.",
                manifest.name,
            );
        }
        Err(other) => {
            panic!("harness error for {}: {} ({other:?})", manifest.name, other);
        }
    }
}

fn assert_real_eval_invariants(eval: &RealDatasetEvaluation) {
    assert!(eval.deterministic_replay_holds,
            "Theorem 9 violated on real bytes: {}", eval.manifest_name);
    assert!(eval.metrics.fault_recall >= 0.0 && eval.metrics.fault_recall <= 1.0);
    assert!(eval.metrics.episode_precision >= 0.0 && eval.metrics.episode_precision <= 1.0);
    assert!(eval.metrics.rscr >= 0.0);
    assert!(eval.metrics.investigation_load_reduction_pct <= 100.0);
}

fn print_metrics_json(eval: &RealDatasetEvaluation) {
    println!("{{");
    println!("  \"manifest_name\": \"{}\",", eval.manifest_name);
    println!("  \"deterministic_replay_holds\": {},", eval.deterministic_replay_holds);
    println!("  \"episode_count\": {},", eval.episode_count);
    println!("  \"metrics\": {{");
    println!("    \"total_windows\": {},", eval.metrics.total_windows);
    println!("    \"total_signals\": {},", eval.metrics.total_signals);
    println!("    \"raw_anomaly_count\": {},", eval.metrics.raw_anomaly_count);
    println!("    \"dsfb_episode_count\": {},", eval.metrics.dsfb_episode_count);
    println!("    \"rscr\": {},", eval.metrics.rscr);
    println!("    \"episode_precision\": {},", eval.metrics.episode_precision);
    println!("    \"fault_recall\": {},", eval.metrics.fault_recall);
    println!("    \"investigation_load_reduction_pct\": {},",
             eval.metrics.investigation_load_reduction_pct);
    println!("    \"clean_window_false_episode_rate\": {}",
             eval.metrics.clean_window_false_episode_rate);
    println!("  }}");
    println!("}}");
}

#[test]
fn promise_defect_prediction_six_csv_module_count() {
    run_or_skip(&MANIFEST_PROMISE, PROMISE_BYTES);
}
