// DSFB-Debug: real-data evaluation against the MultiDimension-Localization
// dataset (NetManAIOps) — 21 microservices × 19 metrics per service =
// 399 time series, 58 anomaly cases with ground-truth root-cause labels.
//
// Why this dataset: the high-dimensional service-graph validator. The
// canonical TADBench / Illinois fixtures exercise the engine at modest
// signal counts (≤ 10). MultiDim-Localization stresses the engine at
// 399 signals, exposing how `aggregate_episodes` correlates structural
// transitions across a wide service graph. Note: at 399 signals, the
// internal flat-aggregation cap (8192) admits ≤ 20 windows per slice;
// multi-slice evaluation is therefore the recommended pattern when the
// fixture is populated, and BufferTooSmall must not appear (otherwise
// the slice is too wide for a single eval — split into per-anomaly-case
// slices).
//
// Sentinel-tolerant pattern matching eval_tadbench.rs.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::error::DsfbError;
use dsfb_debug::real_data::{
    evaluate_real_dataset,
    MANIFEST_MULTIDIM_LOCALIZATION,
    RealDatasetEvaluation,
    RealDatasetManifest,
};
use dsfb_debug::DsfbDebugEngine;

const MULTIDIM_BYTES: &[u8] = include_bytes!("../data/fixtures/multidim_localization.tsv");

fn run_or_skip(manifest: &RealDatasetManifest, bytes: &[u8]) {
    // The const-generic MAX_SIGNALS is sized for this dataset
    // specifically — 399 signals fit inside a 512-signal const-generic
    // engine. The 8192 flat cap then admits up to 20 windows per slice.
    let engine = DsfbDebugEngine::<512, 64>::paper_lock()
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
        Err(DsfbError::BufferTooSmall { needed, available }) => {
            panic!(
                "{}: BufferTooSmall (needed {}, available {}). \
                 The fixture has too many (signals * windows). \
                 Split into per-anomaly-case slices and re-vendor.",
                manifest.name, needed, available,
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
fn multidim_localization_high_dimensional_service_graph() {
    run_or_skip(&MANIFEST_MULTIDIM_LOCALIZATION, MULTIDIM_BYTES);
}
