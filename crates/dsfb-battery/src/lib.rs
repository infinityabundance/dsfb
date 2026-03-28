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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(not(any(feature = "alloc", feature = "std")))]
compile_error!(
    "dsfb-battery requires the default `std` feature or the `alloc` feature for core-engine-only builds."
);

#[cfg(feature = "std")]
pub mod ablation;
#[cfg(feature = "std")]
pub mod addendum;
#[cfg(feature = "std")]
pub mod audit;
#[cfg(feature = "std")]
pub mod complexity;
#[cfg(feature = "std")]
pub mod compliance;
pub mod detection;
#[cfg(feature = "std")]
pub mod engineer_plots;
#[cfg(feature = "std")]
pub mod evaluation;
#[cfg(feature = "std")]
pub mod export;
pub mod ffi;
#[cfg(feature = "std")]
pub mod heuristics;
#[cfg(feature = "std")]
pub mod integration;
pub mod math;
#[cfg(feature = "std")]
pub mod multicell;
#[cfg(feature = "std")]
pub mod nasa;
#[cfg(feature = "std")]
pub mod noise_robustness;
#[cfg(feature = "std")]
pub mod output_paths;
#[cfg(feature = "std")]
pub mod plotting;
#[cfg(feature = "std")]
pub mod resource_trace;
#[cfg(feature = "std")]
pub mod sensitivity;
#[cfg(feature = "std")]
pub mod sota;
pub mod types;

#[cfg(kani)]
#[path = "../formal/kani/proofs.rs"]
mod kani_proofs;

#[cfg(feature = "std")]
pub use ablation::{
    build_cumulative_residual_detection, run_ablation_workflow, AblationArtifact,
    AblationCellSummary, AblationMethodSummary,
};
#[cfg(feature = "std")]
pub use addendum::{
    build_tamper_evident_trace, build_zero_burden_overlay, render_swapc_report,
    resolve_addendum_output_dir, run_addendum_workflow, tamper_evident_verification,
    AddendumImplementationSummary, AddendumOverlayArtifact, AddendumSeuResilienceArtifact,
    AddendumSwapcArtifact, BatteryPassportStubArtifact, MissionBusSignalDictionary,
    TamperEvidentTraceArtifact, TamperEvidentVerificationArtifact, TriStateLegendEntry,
};
#[cfg(feature = "std")]
pub use audit::{
    build_stage2_audit_trace, ArtifactManifest, AuditEvent, AuditTraceBuildContext,
    BenchmarkConfiguration, DatasetDescriptor, FailureModeObservation, InterfaceContract,
    OutputContract, RunMetadata, Stage2AuditTraceArtifact, SummaryOutcome,
};
#[cfg(feature = "std")]
pub use complexity::{
    estimate_dsfb_update_complexity, write_complexity_report, ComplexityArtifact,
    ComplexityMemoryFootprint, ComplexityOperationEstimate,
};
#[cfg(feature = "std")]
pub use compliance::{
    render_misra_equivalent_report, resolve_compliance_output_dir, run_compliance_workflow,
    run_compliance_workflow_from_input, ComplianceImplementationSummary, DeterminismCheckArtifact,
    OperatorOverlayRow, OperatorOverlaySummary, SafeRustAudit, SafeRustFinding, StandardStatus,
    StandardStatusRecord, StcTraceabilitySupport,
};
pub use detection::{
    build_dsfb_detection, build_threshold_detection, detect_dsfb_alarm, detect_eol,
    detect_threshold_alarm, run_dsfb_pipeline, verify_theorem1,
};
#[cfg(feature = "std")]
pub use export::{
    export_audit_trace_json, export_results_json, export_trajectory_csv, export_zip, Stage2Results,
};
#[cfg(feature = "std")]
pub use heuristics::{
    build_heuristic_observation_profile, load_heuristics_bank, load_nasa_heuristics_bank,
    retrieve_heuristic_matches, run_nasa_heuristics_bank_workflow, verify_heuristics_bank,
    verify_nasa_heuristics_bank, AmbiguityLevel, HeuristicEvidenceInstance,
    HeuristicEvidenceSet, HeuristicInventoryItem, HeuristicInterpretation,
    HeuristicMatchCriteria, HeuristicMatchResult, HeuristicObservationProfile,
    HeuristicPatternDescriptor, HeuristicPatternPersistence, HeuristicProvenance,
    HeuristicStatus, HeuristicTransitionObservation, HeuristicsBankArtifact,
    HeuristicsBankEntryRecord, HeuristicsBankEvidenceSummaryArtifact,
    HeuristicsBankInventoryArtifact, HeuristicsBankRetrievalArtifact,
    HeuristicsBankSummaryArtifact, HeuristicsBankVerification,
    HeuristicsBankWorkflowArtifact, HeuristicsIntegrityMetadata,
    NasaHeuristicsBankArtifact, NasaHeuristicsBankEntry, NasaHeuristicsBankVerification,
    NASA_HEURISTICS_BANK_JSON, NASA_HEURISTICS_BANK_SHA256,
};
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
pub use multicell::{run_multicell_workflow, MultiCellArtifact};
#[cfg(feature = "std")]
pub use nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells, NasaPcoeCellSpec};
#[cfg(feature = "std")]
pub use noise_robustness::{
    run_noise_robustness_workflow, NoiseRobustnessArtifact, NoiseRobustnessRecord,
};
#[cfg(feature = "std")]
pub use output_paths::resolve_helper_output_dir;
#[cfg(feature = "std")]
pub use plotting::{generate_all_figures, FigureContext};
#[cfg(feature = "std")]
pub use resource_trace::{
    run_resource_trace_workflow, verify_resource_trace_validation_hash, AuditHeader,
    ComputationalProfile, HeuristicsCost, HeuristicsDensityReport, MeasurementMode,
    MemoryBudgetReport, MemoryFootprint, ResourceMetric, ResourceRunSummary, ResourceTrace,
    ResourceTraceArtifacts, TimingSummary, WindowTrace,
};
#[cfg(feature = "std")]
pub use sensitivity::{run_sensitivity_workflow, SensitivityArtifact, SensitivityScenarioResult};
#[cfg(feature = "std")]
pub use sota::{
    run_sota_comparison_workflow, SotaComparisonArtifact, SotaMethodResult, SotaPerCellSummary,
};
pub use types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, HeuristicBankEntry,
    PipelineConfig, ReasonCode, SignTuple, Theorem1Result,
};

/// Load NASA PCoE battery capacity data from a CSV file.
///
/// This host-side convenience loader remains `std`-only. The feature-gated
/// core engine API in `types`, `math`, `detection`, and `ffi` is the
/// supported `no_std + alloc` surface.
///
/// Expects columns: cycle, capacity_ah, type
/// Returns a vector of (cycle, capacity_ah) tuples.
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
pub fn load_b0005_csv(
    path: &std::path::Path,
) -> Result<Vec<(usize, f64)>, Box<dyn std::error::Error>> {
    load_capacity_csv(path)
}
