//! Unitree G1 humanoid teleoperation adapter (Makolon0321 /
//! `unitree_g1_block_stack` open dataset, 2024–2025).
//!
//! **Provenance.** HuggingFace dataset `Makolon0321/unitree_g1_block_stack`,
//! Apache-2.0 licence, real Unitree G1 humanoid (23-DoF biped +
//! bimanual) performing a block-stacking task. 21 real teleoperation
//! episodes at 10 Hz, 13 726 frames total; this crate ingests the
//! first 10 episodes (3 671 frames) as the §10.13 row.
//!
//! **Cassie substitution rationale.** The originally-proposed row was
//! Cassie (OSU Dynamic Robotics, Siekmann et al., RSS 2021); the
//! canonical UMich-BipedLab `measurements_v1.mat` / `true_state_v1.mat`
//! recordings are MATLAB v5 files with Simulink Stateflow
//! opaque-wrapped time series that cannot be decoded outside MATLAB.
//! Unitree G1 replaces the row with the same "real bipedal humanoid
//! hardware teleoperation" residual category, open-licence and
//! directly consumable via HuggingFace parquet.
//!
//! Residual construction: Euclidean norm of the 74-dim whole-body
//! observation-state deviation from the 20 % early-window nominal per
//! timestep, concatenated across episodes. See
//! `scripts/preprocess_datasets.py::preprocess_unitree_g1`.

/// Placeholder fixture — real-data path is authoritative.
pub const FIXTURE_PLACEHOLDER: [f64; 6] = [0.03, 0.05, 0.15, 0.22, 0.12, 0.04];

/// Fixture entry point.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
