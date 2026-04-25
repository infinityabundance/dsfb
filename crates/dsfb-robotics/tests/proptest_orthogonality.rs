//! Property tests for the orthogonality / non-compete posture.
//!
//! The DSFB framework augments existing methods; it does not compete
//! with them. The paper makes this posture explicit; this test makes
//! it mechanically enforceable in the codebase.
//!
//! The property tests in this file assert that for arbitrary residual
//! streams (within the FSM's documented input domain):
//!
//! 1. The emitted [`PaperLockReport`] never carries a key that
//!    references an incumbent comparator (no "incumbent_*",
//!    "outperforms_*", "earlier_than_*", "vs_threshold", etc.).
//! 2. The grammar census never asserts pre-emption of an incumbent
//!    detector (no Boolean comparisons against threshold values, no
//!    "would have been triggered" semantics).
//! 3. Re-running the same residual stream through the engine produces
//!    a byte-identical JSON output (already covered by the
//!    serialized_report_is_byte_identical_across_runs unit test, but
//!    re-asserted here at proptest scale to catch any drift introduced
//!    by future refactors).
//!
//! These properties are advisory orthogonality checks at the surface
//! level: they cannot prove the paper makes no over-claims (that is a
//! human-review property), but they do prevent silent introduction of
//! over-claim-shaped fields into the JSON output.
#![cfg(feature = "paper_lock")]

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::{
    run_real_data_with_csv_path, serialize_report,
};
use proptest::prelude::*;

/// Words that, if present in an emitted JSON field name, would
/// violate the orthogonality posture.
const FORBIDDEN_FIELD_NAME_FRAGMENTS: &[&str] = &[
    "incumbent",
    "outperform",
    "earlier_than",
    "faster_than",
    "vs_threshold",
    "vs_cusum",
    "vs_ewma",
    "wins_over",
    "beats_",
    "lead_time",
    "would_have_triggered",
    "preempts",
    "pre_empts",
];

fn write_csv(stream: &[f64]) -> std::path::PathBuf {
    use std::io::Write;
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "dsfb_proptest_ortho_{}.csv",
        std::process::id() ^ stream.len() as u32
    ));
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "residual_norm").unwrap();
    for v in stream {
        writeln!(f, "{:.17}", v).unwrap();
    }
    path
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Property: For arbitrary finite residual streams, no JSON
    /// field name in the emitted PaperLockReport contains any
    /// orthogonality-violating word.
    #[test]
    fn report_json_contains_no_forbidden_fragments(
        // Residuals: lengths 32..512 to keep proptest fast.
        residuals in prop::collection::vec(-100.0_f64..100.0_f64, 32..512),
    ) {
        let csv = write_csv(&residuals);
        let report = run_real_data_with_csv_path(
            DatasetId::Cwru,  // arbitrary slug — we only care about the JSON
            false,
            &csv,
        ).unwrap();
        let json = serialize_report(&report).unwrap();
        let lower = json.to_ascii_lowercase();
        for &frag in FORBIDDEN_FIELD_NAME_FRAGMENTS {
            prop_assert!(
                !lower.contains(frag),
                "JSON output contained forbidden orthogonality-violating fragment '{frag}'"
            );
        }
        let _ = std::fs::remove_file(&csv);
    }

    /// Property: bit-exact reproducibility — identical inputs produce
    /// byte-identical JSON across repeat invocations.
    #[test]
    fn json_is_byte_identical_across_repeat_runs(
        residuals in prop::collection::vec(-50.0_f64..50.0_f64, 32..256),
    ) {
        let csv = write_csv(&residuals);
        let a = serialize_report(
            &run_real_data_with_csv_path(DatasetId::Cwru, false, &csv).unwrap()
        ).unwrap();
        let b = serialize_report(
            &run_real_data_with_csv_path(DatasetId::Cwru, false, &csv).unwrap()
        ).unwrap();
        prop_assert_eq!(a, b);
        let _ = std::fs::remove_file(&csv);
    }

    /// Property: census quantities sum to the input stream length
    /// (no episode is double-counted or dropped).
    #[test]
    fn grammar_census_partitions_the_stream(
        residuals in prop::collection::vec(-200.0_f64..200.0_f64, 32..1024),
    ) {
        let csv = write_csv(&residuals);
        let report = run_real_data_with_csv_path(DatasetId::Cwru, false, &csv).unwrap();
        let agg = &report.aggregate;
        prop_assert_eq!(
            agg.admissible + agg.boundary + agg.violation,
            agg.total_samples,
            "census must partition the stream",
        );
        prop_assert_eq!(agg.total_samples, residuals.len());
        prop_assert!(agg.compression_ratio >= 0.0 && agg.compression_ratio <= 1.0);
        prop_assert!(agg.max_residual_norm_sq >= 0.0);
        let _ = std::fs::remove_file(&csv);
    }
}
