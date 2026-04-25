//! ALOHA bimanual static teleoperation adapter (Zhao et al., 2023;
//! LeRobot `aloha_static_coffee` corpus).
//!
//! **Provenance.** Zhao, Kumar, Finn, *"Learning Fine-Grained Bimanual
//! Manipulation with Low-Cost Hardware"* (ALOHA hardware + dataset
//! release, Stanford 2023). The LeRobot `aloha_static_coffee` subset
//! contains 50 real bimanual teleoperation episodes at 50 Hz on the
//! physical ALOHA dual-arm setup (2 × 6-DoF ViperX + 2 × 1-DoF
//! grippers = 14 joint DoFs), 55 000 frames total.
//!
//! **Franka-Kitchen substitution rationale.** Franka Kitchen (Gupta
//! et al., CoRL 2019) was initially scoped as the §10.14 row but is
//! MuJoCo-simulation data (the environment is `FrankaKitchen-v1`, a
//! Gymnasium sim), disqualifying it under the crate's real-world-only
//! policy. ALOHA static coffee replaces it: same teleop-demonstration
//! regime, genuinely physical hardware, same residual category.
//!
//! Residual construction: Euclidean norm of 14-DoF joint-state
//! deviation from the first-sample nominal per timestep. See
//! `scripts/preprocess_datasets.py::preprocess_aloha_static`.

/// Placeholder fixture — real-data path is authoritative.
pub const FIXTURE_PLACEHOLDER: [f64; 5] = [0.01, 0.04, 0.10, 0.06, 0.02];

/// Fixture entry point.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    let n = out.len().min(FIXTURE_PLACEHOLDER.len());
    debug_assert!(n <= out.len() && n <= FIXTURE_PLACEHOLDER.len(), "n must respect both source and dest bounds");
    out[..n].copy_from_slice(&FIXTURE_PLACEHOLDER[..n]);
    debug_assert!(n > 0, "fixture must emit at least one sample");
    n
}
