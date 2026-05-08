// DSFB-Debug: real-data evaluation against the Tsinghua AIOps Challenge
// 2020/2021 tri-modal (logs+metrics+traces) e-commerce microservice
// dataset (26 services).
//
// Why this dataset: it is the multi-modal fusion validator. Faults
// injected: packet loss, memory exhaustion, network delay, disk
// exhaustion, CPU exhaustion, JVM resource exhaustion. The residual
// projection in this fixture is per-service (latency_p50_ms, error_rate)
// at 1-second windows; full tri-modal fusion (with log-frequency
// residuals) is a follow-up extension once the harness's fixture format
// admits a third channel.
//
// Sentinel-tolerant pattern: when the fixture is the
// `# UPSTREAM_FIXTURE_NOT_VENDORED` sentinel form (fresh checkout), the
// test prints `[skip]` and returns Ok. Once the fixture is populated
// from upstream (per data/README.md §Extraction) and MANIFEST.toml's
// fixture_sha256 is updated, the test runs the full pipeline and prints
// a structured JSON metric block on stdout.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::error::DsfbError;
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_AIOPS_CHALLENGE,
    RealDatasetEvaluation,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const AIOPS_BYTES: &[u8] = include_bytes!("../data/fixtures/aiops_challenge.tsv");

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
fn aiops_challenge_tri_modal_microservice() {
    run_or_skip(&MANIFEST_AIOPS_CHALLENGE, AIOPS_BYTES);
}
