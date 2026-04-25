//! DROID manipulation adapter (Khazatsky et al., Stanford/TRI 2024).
//!
//! **Provenance.** Khazatsky et al., *"DROID: A Large-Scale In-the-Wild
//! Robot Manipulation Dataset"*, 2024. 76 000+ teleoperation
//! demonstrations collected across 18 institutions on real Franka
//! Emika Panda robots performing 350+ tasks.
//!
//! **Residual DSFB structures.** Per-timestep Euclidean norm of the
//! deviation of measured joint proprioception from the early-window
//! nominal across the 7-DoF Panda state.
//!
//! This adapter does not ship a Rust loader for DROID's RLDS/TFRecord
//! format — the preprocessor (`scripts/preprocess_datasets.py`)
//! consumes the raw format and emits `data/processed/droid.csv` with
//! one residual norm per row, which
//! [`crate::paper_lock::run_real_data`] then ingests.

/// DROID-compatible residual stream is sized at runtime by the
/// preprocessor; the Rust surface only exposes a placeholder fixture
/// so dsfb-gray can audit the module's documentation coverage.
pub const FIXTURE_PLACEHOLDER: [f64; 4] = [0.01, 0.02, 0.05, 0.03];

/// No in-crate smoke fixture — DROID is large-scale and only exercised
/// via the real-data path.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
