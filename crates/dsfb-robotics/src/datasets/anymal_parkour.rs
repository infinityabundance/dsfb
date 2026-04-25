//! ANYmal-C parkour quadruped-balance adapter (Miki et al., *Science
//! Robotics* 2022).
//!
//! **Provenance.** Miki, Lee, Hwangbo, Wellhausen, Koltun, Hutter,
//! *"Learning robust perceptive locomotion for quadrupedal robots in
//! the wild"*, *Science Robotics*, 2022. ANYmal-C traversing real
//! outdoor environments including mountains, forests, and staircases.
//!
//! Residual construction: Euclidean norm of IMU + joint-state
//! deviation from the nominal stance per timestep. See
//! `scripts/preprocess_datasets.py::preprocess_anymal_parkour` for
//! the raw-log → residual pipeline.

/// Placeholder fixture — real-data path is authoritative.
pub const FIXTURE_PLACEHOLDER: [f64; 6] = [0.05, 0.08, 0.12, 0.20, 0.10, 0.06];

/// Fixture entry point.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
