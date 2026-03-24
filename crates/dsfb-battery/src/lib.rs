// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Library root
//
// Standalone crate implementing the DSFB structural semiotics engine
// for battery health monitoring, as described in:
//   "DSFB Structural Semiotics Engine for Battery Health Monitoring"
//   by Riaan de Beer, Version 1.0.
//
// This crate implements:
//   - Residual sign tuple construction (Definition 1)
//   - Admissibility envelope parameterization (Definition 3)
//   - Grammar state evaluation (Definition 2, Proposition 3)
//   - Typed reason code assignment (Section 5)
//   - Theorem 1 exit bound verification
//   - Detection comparison: DSFB structural alarm vs threshold baseline
//   - CSV/JSON artifact export

pub mod detection;
pub mod export;
pub mod math;
pub mod types;

pub use detection::{
    build_dsfb_detection, build_threshold_detection, detect_dsfb_alarm, detect_eol,
    detect_threshold_alarm, run_dsfb_pipeline, verify_theorem1,
};
pub use export::{export_results_json, export_trajectory_csv, Stage2Results};
pub use math::{
    compute_all_drifts, compute_all_residuals, compute_all_slews, compute_drift, compute_envelope,
    compute_residual, compute_slew, theorem1_exit_bound,
};
pub use types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, HeuristicBankEntry,
    PipelineConfig, ReasonCode, SignTuple, Theorem1Result,
};

/// Load NASA PCoE B0005 capacity data from a CSV file.
///
/// Expects columns: cycle, capacity_ah, type
/// Returns a vector of (cycle, capacity_ah) tuples.
pub fn load_b0005_csv(
    path: &std::path::Path,
) -> Result<Vec<(usize, f64)>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in reader.records() {
        let record = result?;
        let cycle: usize = record
            .get(0)
            .ok_or("missing cycle column")?
            .parse()?;
        let capacity: f64 = record
            .get(1)
            .ok_or("missing capacity_ah column")?
            .parse()?;
        data.push((cycle, capacity));
    }
    Ok(data)
}
