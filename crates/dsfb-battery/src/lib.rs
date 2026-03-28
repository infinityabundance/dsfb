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

pub mod ablation;
pub mod audit;
pub mod complexity;
pub mod detection;
pub mod engineer_plots;
pub mod evaluation;
pub mod export;
pub mod ffi;
pub mod heuristics;
pub mod integration;
pub mod math;
pub mod multicell;
pub mod nasa;
pub mod noise_robustness;
pub mod output_paths;
pub mod plotting;
pub mod sensitivity;
pub mod sota;
pub mod types;

pub use ablation::{
    build_cumulative_residual_detection, run_ablation_workflow, AblationArtifact,
    AblationCellSummary, AblationMethodSummary,
};
pub use audit::{
    build_stage2_audit_trace, ArtifactManifest, AuditEvent, AuditTraceBuildContext,
    BenchmarkConfiguration, DatasetDescriptor, FailureModeObservation, InterfaceContract,
    OutputContract, RunMetadata, Stage2AuditTraceArtifact, SummaryOutcome,
};
pub use complexity::{
    estimate_dsfb_update_complexity, write_complexity_report, ComplexityArtifact,
    ComplexityMemoryFootprint, ComplexityOperationEstimate,
};
pub use detection::{
    build_dsfb_detection, build_threshold_detection, detect_dsfb_alarm, detect_eol,
    detect_threshold_alarm, run_dsfb_pipeline, verify_theorem1,
};
pub use export::{
    export_audit_trace_json, export_results_json, export_trajectory_csv, export_zip, Stage2Results,
};
pub use heuristics::{
    load_heuristics_bank, verify_heuristics_bank, HeuristicsBankArtifact,
    HeuristicsBankEntryRecord, HeuristicsBankVerification,
};
pub use integration::{
    build_adaptive_residual_handoff_note, build_engineer_integration_artifact,
    build_external_residual_evaluation, build_knee_onset_narrative,
    build_partial_observability_scaffold_note, build_shadow_mode_integration_spec,
    build_validity_token, compute_tactical_margin_summary, load_external_residual_csv,
    write_engineer_extension_summary, EngineerExtensionSummary, ExternalResidualEvaluation,
    ExternalResidualSample, IntegrationArtifact, KneeOnsetNarrative, TacticalMarginSummary,
    ValidityToken,
};
pub use math::{
    compute_all_drifts, compute_all_residuals, compute_all_slews, compute_drift, compute_envelope,
    compute_residual, compute_slew, theorem1_exit_bound,
};
pub use multicell::{run_multicell_workflow, MultiCellArtifact};
pub use nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells, NasaPcoeCellSpec};
pub use noise_robustness::{
    run_noise_robustness_workflow, NoiseRobustnessArtifact, NoiseRobustnessRecord,
};
pub use output_paths::resolve_helper_output_dir;
pub use plotting::{generate_all_figures, FigureContext};
pub use sensitivity::{run_sensitivity_workflow, SensitivityArtifact, SensitivityScenarioResult};
pub use sota::{
    run_sota_comparison_workflow, SotaComparisonArtifact, SotaMethodResult, SotaPerCellSummary,
};
pub use types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, HeuristicBankEntry,
    PipelineConfig, ReasonCode, SignTuple, Theorem1Result,
};

/// Load NASA PCoE battery capacity data from a CSV file.
///
/// Expects columns: cycle, capacity_ah, type
/// Returns a vector of (cycle, capacity_ah) tuples.
pub fn load_capacity_csv(
    path: &std::path::Path,
) -> Result<Vec<(usize, f64)>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in reader.records() {
        let record = result?;
        let cycle: usize = record.get(0).ok_or("missing cycle column")?.parse()?;
        let capacity: f64 = record.get(1).ok_or("missing capacity_ah column")?.parse()?;
        data.push((cycle, capacity));
    }
    Ok(data)
}

/// Load NASA PCoE B0005 capacity data from a CSV file.
///
/// This is retained for backward compatibility with the existing
/// single-cell production path.
pub fn load_b0005_csv(
    path: &std::path::Path,
) -> Result<Vec<(usize, f64)>, Box<dyn std::error::Error>> {
    load_capacity_csv(path)
}
