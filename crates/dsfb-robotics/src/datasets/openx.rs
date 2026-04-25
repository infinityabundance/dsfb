//! Open X-Embodiment adapter (RT-X 2024).
//!
//! **Provenance.** The Open X-Embodiment collaboration, *"Open
//! X-Embodiment: Robotic Learning Datasets and RT-X Models"*, RSS
//! 2024. A cross-robot manipulation corpus aggregated from 22 robot
//! embodiments (Franka, UR, xArm, WidowX, Sawyer, Kuka, Stretch,
//! Aloha, and more).
//!
//! Residual construction: Euclidean norm of joint-state deviation from
//! the per-sample early-window nominal, aggregated across the
//! embodiment's degrees of freedom. See
//! `scripts/preprocess_datasets.py::preprocess_openx` for the raw
//! RLDS/TFRecord → residual pipeline.

/// Placeholder fixture — real-data path is the source of truth.
pub const FIXTURE_PLACEHOLDER: [f64; 4] = [0.02, 0.01, 0.03, 0.02];

/// Fixture entry point retained for CLI / smoke-test symmetry.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
