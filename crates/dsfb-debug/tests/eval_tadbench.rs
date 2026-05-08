// DSFB-Debug: real-data evaluation tests against the TADBench / TrainTicket
// fault-injection slices vendored under data/fixtures/.
//
// Gating:
//   - Requires `std` (Vec for variable-size buffers, JSON-on-stdout).
//   - Requires `paper-lock` (the entry point that hard-errors on missing
//     real data).
//
// Behaviour on a fresh checkout (sentinel fixtures):
//   - Each test calls `evaluate_real_dataset` and expects either:
//     * Ok(eval) — fixture is populated; print BenchmarkMetrics and assert
//       Theorem 9 (deterministic_replay_holds == true).
//     * Err(MissingRealData) — fixture is the sentinel form; print an
//       actionable message pointing the reviewer at data/README.md and
//       return Ok so `cargo test` does not fail merely because the
//       fixture is not yet vendored. This satisfies the policy
//       "paper-lock hard-errors on missing real data" without forcing
//       a fresh checkout to populate fixtures before running tests.
//   - Any other error variant fails the test (the harness must be wired
//     correctly even when fixtures are not vendored).

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::error::DsfbError;
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_TADBENCH_F04,
    MANIFEST_TADBENCH_F11,
    MANIFEST_TADBENCH_F11B,
    MANIFEST_TADBENCH_F19,
    RealDatasetEvaluation,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const F04_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F04.tsv");
const F11_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");
const F11B_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11b.tsv");
const F19_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F19.tsv");

fn run_or_skip(manifest: &RealDatasetManifest, bytes: &[u8]) {
    let engine = DsfbDebugEngine::<32, 64>::paper_lock()
        .expect("paper-lock engine creation should succeed");
    match evaluate_real_dataset(&engine, manifest, bytes) {
        Ok(eval) => {
            assert_real_eval_invariants(&eval);
            print_metrics_json(&eval);
        }
        Err(DsfbError::MissingRealData) => {
            // Sentinel-only fixture on a fresh checkout. Surface an
            // actionable message and let the test pass — the harness is
            // correctly wired; the fixture itself is the missing piece.
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
    // Range-bound invariants only — academic-honesty mode. We never
    // assert exact numerical RSCR / fault_recall values: those are
    // empirical findings to be reported in the paper, not test fixtures
    // to be hard-coded.
    assert!(eval.deterministic_replay_holds,
            "Theorem 9 violated on real bytes: {}", eval.manifest_name);
    assert!(eval.metrics.fault_recall >= 0.0 && eval.metrics.fault_recall <= 1.0);
    assert!(eval.metrics.episode_precision >= 0.0 && eval.metrics.episode_precision <= 1.0);
    assert!(eval.metrics.rscr >= 0.0);
    assert!(eval.metrics.investigation_load_reduction_pct <= 100.0);
}

fn print_metrics_json(eval: &RealDatasetEvaluation) {
    // Minimal hand-rolled JSON emitter — keeps zero-dep policy intact.
    // Reviewers parse this off stdout into the paper's results table.
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
fn tadbench_trainticket_f04_cascading_timeout() {
    run_or_skip(&MANIFEST_TADBENCH_F04, F04_BYTES);
}

#[test]
fn tadbench_trainticket_f11_deployment_regression() {
    run_or_skip(&MANIFEST_TADBENCH_F11, F11_BYTES);
}

#[test]
fn tadbench_trainticket_f11b_auth_mongo_fault() {
    run_or_skip(&MANIFEST_TADBENCH_F11B, F11B_BYTES);
}

#[test]
fn tadbench_trainticket_f19_mongodb_driver_regression() {
    run_or_skip(&MANIFEST_TADBENCH_F19, F19_BYTES);
}
