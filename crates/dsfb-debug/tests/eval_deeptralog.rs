// DSFB-Debug: real-data evaluation against the DeepTraLog dataset
// (Zhang et al., ICSE 2022) — combined log + trace data for normal and
// abnormal microservice executions.
//
// Why this dataset: the log + trace fusion validator. The residual
// projection in this fixture interleaves per-service log-frequency
// residuals alongside the canonical (latency_p50_ms, error_rate)
// projections. This exercises the heuristics bank's
// SustainedOutwardDrift / RecurrentBoundaryGrazing branches on
// log-derived features — i.e. the "the log volume is silently rising
// before the trace latency does" early-warning case that motivated
// graph-based anomaly detection in the original DeepTraLog paper.
//
// Sentinel-tolerant pattern matching eval_tadbench.rs.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::error::DsfbError;
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_DEEPTRALOG,
    RealDatasetEvaluation,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const DEEPTRALOG_BYTES: &[u8] = include_bytes!("../data/fixtures/deeptralog.tsv");

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
                 Populate per crates/dsfb-debug/data/README.md §Extraction. \n\
                 paper-lock evaluation refused, exactly as the policy requires.",
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
fn deeptralog_log_trace_fusion() {
    run_or_skip(&MANIFEST_DEEPTRALOG, DEEPTRALOG_BYTES);
}
