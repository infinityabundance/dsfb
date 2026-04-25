//! aloha_static_screw_driver adapter — see crate's docs for provenance.
//!
//! Real data flows through `scripts/preprocess_datasets.py` to
//! `data/processed/aloha_static_screw_driver.csv`; the canonical entry point is
//! `paper-lock aloha_static_screw_driver` which loads that CSV via
//! `crate::paper_lock::run_real_data`.

/// Placeholder fixture — real-data path is authoritative for this dataset.
pub const FIXTURE_PLACEHOLDER: [f64; 5] = [0.01, 0.05, 0.10, 0.06, 0.02];

/// Fixture entry point retained for CLI / smoke-test symmetry across all datasets.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
