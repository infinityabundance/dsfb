// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Reusable cell-level evaluation helpers for additive workflows.

use crate::detection::{
    build_dsfb_detection, build_threshold_detection, run_dsfb_pipeline, verify_theorem1,
};
use crate::export::Stage2Results;
use crate::types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, PipelineConfig, ReasonCode,
    Theorem1Result,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CellEvaluationSummary {
    pub cell_id: String,
    pub source_csv: String,
    pub cycle_count: usize,
    pub initial_capacity_ah: f64,
    pub final_capacity_ah: f64,
    pub dsfb_alarm_cycle: Option<usize>,
    pub first_boundary_cycle: Option<usize>,
    pub first_violation_cycle: Option<usize>,
    pub threshold_85pct_cycle: Option<usize>,
    pub eol_80pct_cycle: Option<usize>,
    pub lead_time_vs_threshold_baseline: Option<i64>,
    pub persistent_elevation_confirmed: Option<bool>,
    pub primary_reason_code: Option<ReasonCode>,
    pub theorem_t_star: usize,
}

#[derive(Debug, Clone)]
pub struct CellEvaluationRun {
    pub summary: CellEvaluationSummary,
    pub raw_data: Vec<(usize, f64)>,
    pub capacities: Vec<f64>,
    pub envelope: EnvelopeParams,
    pub trajectory: Vec<BatteryResidual>,
    pub dsfb_detection: DetectionResult,
    pub threshold_detection: DetectionResult,
    pub theorem1: Theorem1Result,
    pub stage2_results: Stage2Results,
}

pub fn evaluate_cell(
    cell_id: &str,
    source_csv: &str,
    raw_data: &[(usize, f64)],
    config: &PipelineConfig,
) -> Result<CellEvaluationRun, Box<dyn std::error::Error>> {
    let capacities: Vec<f64> = raw_data.iter().map(|(_, capacity)| *capacity).collect();
    let eol_capacity = config.eol_fraction * capacities[0];
    let (envelope, trajectory) = run_dsfb_pipeline(&capacities, config)?;
    let dsfb_detection = build_dsfb_detection(&trajectory, &capacities, eol_capacity);
    let threshold_detection = build_threshold_detection(&capacities, 0.85, eol_capacity);
    let theorem1 = verify_theorem1(&envelope, &trajectory, config);
    let stage2_results = Stage2Results {
        data_provenance: format!(
            "NASA PCoE Battery Dataset, Cell {} (cell-level repeated evaluation helper)",
            cell_id
        ),
        config: config.clone(),
        envelope,
        dsfb_detection: dsfb_detection.clone(),
        threshold_detection: threshold_detection.clone(),
        theorem1: theorem1.clone(),
    };

    let first_boundary_cycle = trajectory
        .iter()
        .find(|sample| sample.grammar_state == GrammarState::Boundary)
        .map(|sample| sample.cycle);
    let first_violation_cycle = trajectory
        .iter()
        .find(|sample| sample.grammar_state == GrammarState::Violation)
        .map(|sample| sample.cycle);
    let first_non_admissible = trajectory
        .iter()
        .find(|sample| sample.grammar_state != GrammarState::Admissible)
        .map(|sample| sample.cycle);
    let lead_time_vs_threshold_baseline = first_non_admissible
        .zip(threshold_detection.alarm_cycle)
        .map(|(dsfb, threshold)| threshold as i64 - dsfb as i64);
    let persistent_elevation_confirmed = first_non_admissible.map(|cycle| {
        trajectory
            .iter()
            .skip(cycle.saturating_sub(1))
            .take(2)
            .all(|sample| sample.grammar_state != GrammarState::Admissible)
    });
    let primary_reason_code = first_non_admissible
        .and_then(|cycle| trajectory.iter().find(|sample| sample.cycle == cycle))
        .and_then(|sample| sample.reason_code)
        .or_else(|| trajectory.iter().find_map(|sample| sample.reason_code));

    let summary = CellEvaluationSummary {
        cell_id: cell_id.to_string(),
        source_csv: source_csv.to_string(),
        cycle_count: capacities.len(),
        initial_capacity_ah: capacities[0],
        final_capacity_ah: capacities[capacities.len() - 1],
        dsfb_alarm_cycle: dsfb_detection.alarm_cycle,
        first_boundary_cycle,
        first_violation_cycle,
        threshold_85pct_cycle: threshold_detection.alarm_cycle,
        eol_80pct_cycle: dsfb_detection.eol_cycle,
        lead_time_vs_threshold_baseline,
        persistent_elevation_confirmed,
        primary_reason_code,
        theorem_t_star: theorem1.t_star,
    };

    Ok(CellEvaluationRun {
        summary,
        raw_data: raw_data.to_vec(),
        capacities,
        envelope,
        trajectory,
        dsfb_detection,
        threshold_detection,
        theorem1,
        stage2_results,
    })
}

pub fn production_figure_filenames() -> &'static [&'static str] {
    &[
        "fig01_capacity_fade.svg",
        "fig02_residual_trajectory.svg",
        "fig03_drift_trajectory.svg",
        "fig04_slew_trajectory.svg",
        "fig05_admissibility_envelope.svg",
        "fig06_grammar_state_timeline.svg",
        "fig07_detection_comparison.svg",
        "fig08_theorem1_verification.svg",
        "fig09_semiotic_projection.svg",
        "fig10_cumulative_drift.svg",
        "fig11_lead_time_comparison.svg",
        "fig12_heuristics_bank_entry.svg",
    ]
}
