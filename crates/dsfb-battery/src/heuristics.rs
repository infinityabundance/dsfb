// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Versioned, read-only heuristics-bank helpers.

use crate::evaluation::{evaluate_cell, CellEvaluationRun, CellEvaluationSummary};
use crate::load_capacity_csv;
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::types::{BatteryResidual, GrammarState, PipelineConfig, ReasonCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use thiserror::Error;

pub const HEURISTICS_BANK_JSON: &str = "config/heuristics_bank_v1.json";
pub const HEURISTICS_BANK_SHA256: &str = "config/heuristics_bank_v1.sha256";

pub const NASA_HEURISTICS_BANK_JSON: &str = "heuristics/heuristics_bank_v2.json";
pub const NASA_HEURISTICS_BANK_SHA256: &str = "heuristics/heuristics_bank_v2.sha256";

const NASA_HEURISTICS_SUMMARY_JSON: &str = "heuristics_bank_summary.json";
const NASA_HEURISTICS_INVENTORY_JSON: &str = "entry_inventory.json";
const NASA_HEURISTICS_EVIDENCE_JSON: &str = "evidence_summary.json";
const NASA_HEURISTICS_RETRIEVAL_JSON: &str = "retrieval_examples.json";
const NASA_HEURISTICS_VERIFICATION_JSON: &str = "heuristics_bank_verification.json";
const NASA_HEURISTICS_IMPLEMENTATION_SUMMARY: &str = "implementation_summary.txt";

#[derive(Debug, Error)]
pub enum HeuristicsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("heuristics bank hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("heuristics bank missing required fields")]
    InvalidShape,
    #[error("NASA heuristics bank hash mismatch: expected {expected}, got {actual}")]
    NasaHashMismatch { expected: String, actual: String },
    #[error("NASA heuristics bank missing required fields")]
    InvalidNasaShape,
    #[error("no NASA PCoE cell CSVs found in {0}")]
    NoCellData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsBankEntryRecord {
    pub entry_id: String,
    pub pattern: String,
    pub regime_scope: String,
    pub admissibility_assumptions: String,
    pub interpretation: String,
    pub uncertainty_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsBankArtifact {
    pub artifact_type: String,
    pub schema_version: String,
    pub bank_version: String,
    pub frozen: bool,
    pub entries: Vec<HeuristicsBankEntryRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankVerification {
    pub artifact_type: String,
    pub bank_version: String,
    pub verified: bool,
    pub expected_sha256: String,
    pub actual_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeuristicStatus {
    Illustrative,
    Candidate,
    Validated,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicsIntegrityMetadata {
    pub algorithm: String,
    pub sidecar_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicPatternPersistence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_cycles: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slew_cycles: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_non_admissible_cycles: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicPatternDescriptor {
    pub residual_behavior: String,
    pub drift_behavior: String,
    pub slew_behavior: String,
    pub persistence_requirements: HeuristicPatternPersistence,
    pub envelope_relation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_sequence: Vec<GrammarState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicInterpretation {
    pub structural_class: String,
    pub operational_meaning: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_mechanisms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeuristicEvidenceInstance {
    pub dataset: String,
    pub cell_id: String,
    pub cycle_interval_start: usize,
    pub cycle_interval_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_boundary_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_violation_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_85pct_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_threshold_baseline: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theorem_t_star: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_reference: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeuristicEvidenceSet {
    #[serde(default)]
    pub supporting_instances: Vec<HeuristicEvidenceInstance>,
    #[serde(default)]
    pub counter_examples: Vec<HeuristicEvidenceInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicProvenance {
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeuristicMatchCriteria {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_primary_reason_codes: Vec<ReasonCode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_final_states: Vec<GrammarState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_first_boundary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_first_violation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_positive_lead_vs_threshold_baseline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_persistent_elevation_confirmed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_boundary_return_before_violation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_violation_return: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_recurrent_reentry_loop: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_monotone_escalation_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_final_violation_persistent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_lead_time_vs_threshold_baseline: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_boundary_to_violation_gap_cycles: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_terminal_violation_run_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NasaHeuristicsBankEntry {
    pub id: String,
    pub version: String,
    pub title: String,
    pub status: HeuristicStatus,
    pub channels_required: Vec<String>,
    pub regime_scope: String,
    pub admissibility_assumptions: Vec<String>,
    pub pattern: HeuristicPatternDescriptor,
    pub interpretation: HeuristicInterpretation,
    pub ambiguity_notes: Vec<String>,
    pub exclusion_conditions: Vec<String>,
    pub evidence: HeuristicEvidenceSet,
    pub transfer_scope: String,
    pub known_failures: Vec<String>,
    pub provenance: HeuristicProvenance,
    pub match_criteria: HeuristicMatchCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NasaHeuristicsBankArtifact {
    pub artifact_type: String,
    pub schema_version: String,
    pub bank_version: String,
    pub read_only: bool,
    pub dataset_scope: String,
    pub signal_scope: Vec<String>,
    pub integrity: HeuristicsIntegrityMetadata,
    pub entries: Vec<NasaHeuristicsBankEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NasaHeuristicsBankVerification {
    pub artifact_type: String,
    pub bank_version: String,
    pub verified: bool,
    pub expected_sha256: String,
    pub actual_sha256: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeuristicTransitionObservation {
    pub cycle: usize,
    pub previous_state: GrammarState,
    pub current_state: GrammarState,
    pub reason_code: Option<ReasonCode>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeuristicObservationProfile {
    pub cell_id: String,
    pub cycle_count: usize,
    pub signal_scope: Vec<String>,
    pub final_state: GrammarState,
    pub first_boundary_cycle: Option<usize>,
    pub first_violation_cycle: Option<usize>,
    pub threshold_85pct_cycle: Option<usize>,
    pub eol_80pct_cycle: Option<usize>,
    pub lead_time_vs_threshold_baseline: Option<i64>,
    pub persistent_elevation_confirmed: Option<bool>,
    pub primary_reason_code: Option<ReasonCode>,
    pub theorem_t_star: usize,
    pub boundary_to_violation_gap_cycles: Option<usize>,
    pub boundary_return_count: usize,
    pub violation_return_count: usize,
    pub has_boundary_return_before_violation: bool,
    pub has_violation_return: bool,
    pub has_recurrent_reentry_loop: bool,
    pub monotone_escalation_only: bool,
    pub final_violation_persistent: bool,
    pub terminal_violation_run_length: Option<usize>,
    pub max_abs_residual: f64,
    pub max_abs_drift: f64,
    pub max_abs_slew: f64,
    pub transition_observations: Vec<HeuristicTransitionObservation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    None,
    Partial,
    Full,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeuristicMatchResult {
    pub heuristic_id: String,
    pub match_status: MatchStatus,
    pub match_strength: f64,
    pub satisfied_conditions: Vec<String>,
    pub unsatisfied_conditions: Vec<String>,
    pub competing_matches: Vec<String>,
    pub ambiguity_level: AmbiguityLevel,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankSummaryArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub bank_version: String,
    pub signal_scope: Vec<String>,
    pub cells_evaluated: Vec<String>,
    pub unavailable_cells: Vec<String>,
    pub entry_count: usize,
    pub status_counts: BTreeMap<String, usize>,
    pub generated_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicInventoryItem {
    pub heuristic_id: String,
    pub title: String,
    pub status: HeuristicStatus,
    pub structural_class: String,
    pub channels_required: Vec<String>,
    pub supporting_instance_count: usize,
    pub counter_example_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankInventoryArtifact {
    pub artifact_type: String,
    pub bank_version: String,
    pub inventory: Vec<HeuristicInventoryItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicEvidenceSummaryItem {
    pub heuristic_id: String,
    pub supporting_cells: Vec<String>,
    pub counter_example_cells: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankEvidenceSummaryArtifact {
    pub artifact_type: String,
    pub bank_version: String,
    pub entries: Vec<HeuristicEvidenceSummaryItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicRetrievalExample {
    pub cell_id: String,
    pub profile: HeuristicObservationProfile,
    pub matches: Vec<HeuristicMatchResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankRetrievalArtifact {
    pub artifact_type: String,
    pub bank_version: String,
    pub retrieval_examples: Vec<HeuristicRetrievalExample>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankWorkflowArtifact {
    pub summary: HeuristicsBankSummaryArtifact,
    pub inventory: HeuristicsBankInventoryArtifact,
    pub evidence_summary: HeuristicsBankEvidenceSummaryArtifact,
    pub retrieval_examples: HeuristicsBankRetrievalArtifact,
    pub verification: NasaHeuristicsBankVerification,
}

pub fn load_heuristics_bank(crate_dir: &Path) -> Result<HeuristicsBankArtifact, HeuristicsError> {
    let path = crate_dir.join(HEURISTICS_BANK_JSON);
    let json = std::fs::read_to_string(path)?;
    let bank: HeuristicsBankArtifact = serde_json::from_str(&json)?;
    if bank.artifact_type.is_empty()
        || bank.schema_version.is_empty()
        || bank.bank_version.is_empty()
        || bank.entries.is_empty()
    {
        return Err(HeuristicsError::InvalidShape);
    }
    Ok(bank)
}

pub fn verify_heuristics_bank(
    crate_dir: &Path,
) -> Result<HeuristicsBankVerification, HeuristicsError> {
    let bank = load_heuristics_bank(crate_dir)?;
    let json_path = crate_dir.join(HEURISTICS_BANK_JSON);
    let hash_path = crate_dir.join(HEURISTICS_BANK_SHA256);

    let bytes = std::fs::read(json_path)?;
    let actual_sha256 = sha256_hex(&bytes);
    let expected_sha256 = std::fs::read_to_string(hash_path)?.trim().to_string();

    if expected_sha256 != actual_sha256 {
        return Err(HeuristicsError::HashMismatch {
            expected: expected_sha256,
            actual: actual_sha256,
        });
    }

    Ok(HeuristicsBankVerification {
        artifact_type: "dsfb_battery_heuristics_bank_verification".to_string(),
        bank_version: bank.bank_version,
        verified: true,
        expected_sha256: expected_sha256.clone(),
        actual_sha256,
    })
}

pub fn load_nasa_heuristics_bank(
    crate_dir: &Path,
) -> Result<NasaHeuristicsBankArtifact, HeuristicsError> {
    let path = crate_dir.join(NASA_HEURISTICS_BANK_JSON);
    let json = std::fs::read_to_string(path)?;
    let bank: NasaHeuristicsBankArtifact = serde_json::from_str(&json)?;
    if bank.artifact_type.is_empty()
        || bank.schema_version.is_empty()
        || bank.bank_version.is_empty()
        || bank.entries.is_empty()
        || bank.dataset_scope.is_empty()
        || bank.signal_scope.is_empty()
        || bank.integrity.sidecar_path.is_empty()
    {
        return Err(HeuristicsError::InvalidNasaShape);
    }
    Ok(bank)
}

pub fn verify_nasa_heuristics_bank(
    crate_dir: &Path,
) -> Result<NasaHeuristicsBankVerification, HeuristicsError> {
    let bank = load_nasa_heuristics_bank(crate_dir)?;
    let json_path = crate_dir.join(NASA_HEURISTICS_BANK_JSON);
    let hash_path = crate_dir.join(NASA_HEURISTICS_BANK_SHA256);

    let bytes = std::fs::read(json_path)?;
    let actual_sha256 = sha256_hex(&bytes);
    let expected_sha256 = std::fs::read_to_string(hash_path)?.trim().to_string();

    if expected_sha256 != actual_sha256 {
        return Err(HeuristicsError::NasaHashMismatch {
            expected: expected_sha256,
            actual: actual_sha256,
        });
    }

    Ok(NasaHeuristicsBankVerification {
        artifact_type: "dsfb_battery_nasa_heuristics_bank_verification".to_string(),
        bank_version: bank.bank_version,
        verified: true,
        expected_sha256: expected_sha256.clone(),
        actual_sha256,
    })
}

pub fn build_heuristic_observation_profile(
    summary: &CellEvaluationSummary,
    trajectory: &[BatteryResidual],
) -> Result<HeuristicObservationProfile, HeuristicsError> {
    if trajectory.is_empty() {
        return Err(HeuristicsError::InvalidNasaShape);
    }

    let mut transitions = Vec::new();
    let mut previous_state = trajectory[0].grammar_state;
    let mut boundary_return_count = 0usize;
    let mut violation_return_count = 0usize;
    let mut has_regressive_transition = false;

    for sample in trajectory.iter().skip(1) {
        if sample.grammar_state != previous_state {
            if grammar_rank(sample.grammar_state) < grammar_rank(previous_state) {
                has_regressive_transition = true;
            }
            if previous_state == GrammarState::Boundary
                && sample.grammar_state == GrammarState::Admissible
            {
                boundary_return_count += 1;
            }
            if previous_state == GrammarState::Violation
                && sample.grammar_state == GrammarState::Admissible
            {
                violation_return_count += 1;
            }
            transitions.push(HeuristicTransitionObservation {
                cycle: sample.cycle,
                previous_state,
                current_state: sample.grammar_state,
                reason_code: sample.reason_code,
            });
            previous_state = sample.grammar_state;
        }
    }

    let first_violation_cycle = summary.first_violation_cycle;
    let has_boundary_return_before_violation = transitions.iter().any(|transition| {
        transition.previous_state == GrammarState::Boundary
            && transition.current_state == GrammarState::Admissible
            && transition.cycle < first_violation_cycle.unwrap_or(usize::MAX)
    });
    let has_violation_return = violation_return_count > 0;
    let has_recurrent_reentry_loop = if let Some(index) =
        transitions.iter().position(|transition| {
            transition.previous_state == GrammarState::Violation
                && transition.current_state == GrammarState::Admissible
        }) {
        transitions
            .iter()
            .skip(index + 1)
            .any(|transition| transition.previous_state == GrammarState::Admissible)
    } else {
        false
    };

    let final_state = trajectory.last().unwrap().grammar_state;
    let final_violation_persistent = final_state == GrammarState::Violation;
    let terminal_violation_run_length = if final_violation_persistent {
        let count = trajectory
            .iter()
            .rev()
            .take_while(|sample| sample.grammar_state == GrammarState::Violation)
            .count();
        Some(count)
    } else {
        None
    };

    let boundary_to_violation_gap_cycles = summary
        .first_boundary_cycle
        .zip(summary.first_violation_cycle)
        .map(|(boundary, violation)| violation.saturating_sub(boundary));

    let monotone_escalation_only = summary.first_boundary_cycle.is_some()
        && summary.first_violation_cycle.is_some()
        && final_state == GrammarState::Violation
        && !has_regressive_transition;

    let max_abs_residual = trajectory
        .iter()
        .map(|sample| sample.sign.r.abs())
        .fold(0.0f64, f64::max);
    let max_abs_drift = trajectory
        .iter()
        .map(|sample| sample.sign.d.abs())
        .fold(0.0f64, f64::max);
    let max_abs_slew = trajectory
        .iter()
        .map(|sample| sample.sign.s.abs())
        .fold(0.0f64, f64::max);

    Ok(HeuristicObservationProfile {
        cell_id: summary.cell_id.clone(),
        cycle_count: summary.cycle_count,
        signal_scope: vec![
            "capacity".to_string(),
            "residual".to_string(),
            "drift".to_string(),
            "slew".to_string(),
            "persistence".to_string(),
            "envelope_relation".to_string(),
            "grammar_state_transitions".to_string(),
        ],
        final_state,
        first_boundary_cycle: summary.first_boundary_cycle,
        first_violation_cycle: summary.first_violation_cycle,
        threshold_85pct_cycle: summary.threshold_85pct_cycle,
        eol_80pct_cycle: summary.eol_80pct_cycle,
        lead_time_vs_threshold_baseline: summary.lead_time_vs_threshold_baseline,
        persistent_elevation_confirmed: summary.persistent_elevation_confirmed,
        primary_reason_code: summary.primary_reason_code,
        theorem_t_star: summary.theorem_t_star,
        boundary_to_violation_gap_cycles,
        boundary_return_count,
        violation_return_count,
        has_boundary_return_before_violation,
        has_violation_return,
        has_recurrent_reentry_loop,
        monotone_escalation_only,
        final_violation_persistent,
        terminal_violation_run_length,
        max_abs_residual,
        max_abs_drift,
        max_abs_slew,
        transition_observations: transitions,
    })
}

pub fn retrieve_heuristic_matches(
    bank: &NasaHeuristicsBankArtifact,
    profile: &HeuristicObservationProfile,
) -> Vec<HeuristicMatchResult> {
    let mut results: Vec<HeuristicMatchResult> = bank
        .entries
        .iter()
        .map(|entry| evaluate_match(entry, profile))
        .collect();

    let full_match_count = results
        .iter()
        .filter(|result| result.match_status == MatchStatus::Full)
        .count();

    for index in 0..results.len() {
        let competing_matches: Vec<String> = results
            .iter()
            .enumerate()
            .filter(|(other_index, other)| {
                *other_index != index
                    && other.match_status != MatchStatus::None
                    && other.match_strength >= 0.5
            })
            .map(|(_, other)| other.heuristic_id.clone())
            .collect();

        let ambiguity_level = if full_match_count > 1 || competing_matches.len() > 2 {
            AmbiguityLevel::High
        } else if !competing_matches.is_empty() {
            AmbiguityLevel::Medium
        } else {
            AmbiguityLevel::Low
        };

        results[index].competing_matches = competing_matches;
        results[index].ambiguity_level = ambiguity_level;
    }

    results.sort_by(|left, right| {
        right
            .match_strength
            .partial_cmp(&left.match_strength)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.heuristic_id.cmp(&right.heuristic_id))
    });
    results
}

pub fn run_nasa_heuristics_bank_workflow(
    data_dir: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
) -> Result<HeuristicsBankWorkflowArtifact, Box<dyn std::error::Error>> {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bank = load_nasa_heuristics_bank(crate_dir)?;
    let verification = verify_nasa_heuristics_bank(crate_dir)?;

    let mut runs: Vec<CellEvaluationRun> = Vec::new();
    let mut unavailable_cells = Vec::new();

    for cell in supported_nasa_pcoe_cells() {
        let path = default_nasa_cell_csv_path(data_dir, cell);
        if !path.exists() {
            unavailable_cells.push(cell.cell_id.to_string());
            continue;
        }

        let raw_data = load_capacity_csv(&path)?;
        let run = evaluate_cell(
            cell.cell_id,
            path.to_string_lossy().as_ref(),
            &raw_data,
            config,
        )?;
        runs.push(run);
    }

    if runs.is_empty() {
        return Err(Box::new(HeuristicsError::NoCellData(
            data_dir.display().to_string(),
        )));
    }

    std::fs::create_dir_all(output_dir)?;

    let summary = build_summary_artifact(&bank, &runs, unavailable_cells.clone());
    let inventory = build_inventory_artifact(&bank);
    let evidence_summary = build_evidence_summary_artifact(&bank);
    let retrieval_examples = build_retrieval_artifact(&bank, &runs)?;

    write_pretty_json(&summary, &output_dir.join(NASA_HEURISTICS_SUMMARY_JSON))?;
    write_pretty_json(&inventory, &output_dir.join(NASA_HEURISTICS_INVENTORY_JSON))?;
    write_pretty_json(
        &evidence_summary,
        &output_dir.join(NASA_HEURISTICS_EVIDENCE_JSON),
    )?;
    write_pretty_json(
        &retrieval_examples,
        &output_dir.join(NASA_HEURISTICS_RETRIEVAL_JSON),
    )?;
    write_pretty_json(
        &verification,
        &output_dir.join(NASA_HEURISTICS_VERIFICATION_JSON),
    )?;
    write_implementation_summary(
        &bank,
        &runs,
        &output_dir.join(NASA_HEURISTICS_IMPLEMENTATION_SUMMARY),
        output_dir,
    )?;

    Ok(HeuristicsBankWorkflowArtifact {
        summary,
        inventory,
        evidence_summary,
        retrieval_examples,
        verification,
    })
}

fn build_summary_artifact(
    bank: &NasaHeuristicsBankArtifact,
    runs: &[CellEvaluationRun],
    unavailable_cells: Vec<String>,
) -> HeuristicsBankSummaryArtifact {
    let mut status_counts = BTreeMap::new();
    for entry in &bank.entries {
        let key = match entry.status {
            HeuristicStatus::Illustrative => "illustrative",
            HeuristicStatus::Candidate => "candidate",
            HeuristicStatus::Validated => "validated",
            HeuristicStatus::Deprecated => "deprecated",
        };
        *status_counts.entry(key.to_string()).or_insert(0usize) += 1;
    }

    HeuristicsBankSummaryArtifact {
        artifact_type: "dsfb_battery_heuristics_bank_summary".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        bank_version: bank.bank_version.clone(),
        signal_scope: bank.signal_scope.clone(),
        cells_evaluated: runs.iter().map(|run| run.summary.cell_id.clone()).collect(),
        unavailable_cells,
        entry_count: bank.entries.len(),
        status_counts,
        generated_artifacts: vec![
            NASA_HEURISTICS_SUMMARY_JSON.to_string(),
            NASA_HEURISTICS_INVENTORY_JSON.to_string(),
            NASA_HEURISTICS_EVIDENCE_JSON.to_string(),
            NASA_HEURISTICS_RETRIEVAL_JSON.to_string(),
            NASA_HEURISTICS_VERIFICATION_JSON.to_string(),
            NASA_HEURISTICS_IMPLEMENTATION_SUMMARY.to_string(),
        ],
    }
}

fn build_inventory_artifact(bank: &NasaHeuristicsBankArtifact) -> HeuristicsBankInventoryArtifact {
    let inventory = bank
        .entries
        .iter()
        .map(|entry| HeuristicInventoryItem {
            heuristic_id: entry.id.clone(),
            title: entry.title.clone(),
            status: entry.status,
            structural_class: entry.interpretation.structural_class.clone(),
            channels_required: entry.channels_required.clone(),
            supporting_instance_count: entry.evidence.supporting_instances.len(),
            counter_example_count: entry.evidence.counter_examples.len(),
        })
        .collect();

    HeuristicsBankInventoryArtifact {
        artifact_type: "dsfb_battery_heuristics_inventory".to_string(),
        bank_version: bank.bank_version.clone(),
        inventory,
    }
}

fn build_evidence_summary_artifact(
    bank: &NasaHeuristicsBankArtifact,
) -> HeuristicsBankEvidenceSummaryArtifact {
    let entries = bank
        .entries
        .iter()
        .map(|entry| {
            let supporting_cells = sorted_unique_cells(&entry.evidence.supporting_instances);
            let counter_example_cells = sorted_unique_cells(&entry.evidence.counter_examples);
            let mut notes = vec![format!(
                "{} supporting instances, {} counter-examples.",
                entry.evidence.supporting_instances.len(),
                entry.evidence.counter_examples.len()
            )];
            notes.extend(entry.ambiguity_notes.iter().cloned());

            HeuristicEvidenceSummaryItem {
                heuristic_id: entry.id.clone(),
                supporting_cells,
                counter_example_cells,
                notes,
            }
        })
        .collect();

    HeuristicsBankEvidenceSummaryArtifact {
        artifact_type: "dsfb_battery_heuristics_evidence_summary".to_string(),
        bank_version: bank.bank_version.clone(),
        entries,
    }
}

fn build_retrieval_artifact(
    bank: &NasaHeuristicsBankArtifact,
    runs: &[CellEvaluationRun],
) -> Result<HeuristicsBankRetrievalArtifact, HeuristicsError> {
    let mut retrieval_examples = Vec::new();
    for run in runs {
        let profile = build_heuristic_observation_profile(&run.summary, &run.trajectory)?;
        let matches = retrieve_heuristic_matches(bank, &profile);
        retrieval_examples.push(HeuristicRetrievalExample {
            cell_id: run.summary.cell_id.clone(),
            profile,
            matches,
        });
    }

    Ok(HeuristicsBankRetrievalArtifact {
        artifact_type: "dsfb_battery_heuristics_retrieval_examples".to_string(),
        bank_version: bank.bank_version.clone(),
        retrieval_examples,
    })
}

fn write_implementation_summary(
    bank: &NasaHeuristicsBankArtifact,
    runs: &[CellEvaluationRun],
    path: &Path,
    output_dir: &Path,
) -> Result<(), HeuristicsError> {
    let seeded_entries = bank
        .entries
        .iter()
        .map(|entry| format!("- {} ({:?})", entry.id, entry.status))
        .collect::<Vec<_>>()
        .join("\n");
    let evidence_cells = runs
        .iter()
        .map(|run| run.summary.cell_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let mut lines = Vec::new();
    lines.push("Heuristics-bank maturation implementation summary".to_string());
    lines.push(format!("Bank version: {}", bank.bank_version));
    lines.push(format!(
        "Signals grounded in: {}",
        bank.signal_scope.join(", ")
    ));
    lines.push("Seeded entries:".to_string());
    lines.push(seeded_entries);
    lines.push(format!("Evidence attached from cells: {}", evidence_cells));
    lines.push("Fully implemented:".to_string());
    lines.push(
        "- Typed NASA-grounded heuristics bank v2 with versioning and SHA-256 verification."
            .to_string(),
    );
    lines.push(
        "- Deterministic observation profiling and ambiguity-aware retrieval against the current DSFB evidence path."
            .to_string(),
    );
    lines.push(
        "- Isolated heuristics-bank helper artifacts: summary, inventory, evidence summary, retrieval examples, verification, and this summary."
            .to_string(),
    );
    lines.push("Scaffolded or intentionally conservative:".to_string());
    lines.push(
        "- No unique physical mechanism identification is claimed for any heuristic entry."
            .to_string(),
    );
    lines.push("- Cross-chemistry or non-NASA transfer is not claimed by this bank.".to_string());
    lines.push(
        "- The accelerating-fade/knee entry remains illustrative because the current default NASA PCoE runs do not populate that reason code."
            .to_string(),
    );
    lines.push(format!("Artifacts written to: {}", output_dir.display()));
    lines.push("Protection gates:".to_string());
    lines.push("- Existing dsfb-battery-demo behavior was left unchanged.".to_string());
    lines.push("- Existing mono-cell production figure filenames were not reused.".to_string());
    lines.push("- Existing production stage-II artifact filenames were not reused.".to_string());
    lines.push(
        "Confirmation: the existing mono-cell production figure path was not modified.".to_string(),
    );

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn write_pretty_json<T: Serialize>(value: &T, path: &Path) -> Result<(), HeuristicsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn sorted_unique_cells(instances: &[HeuristicEvidenceInstance]) -> Vec<String> {
    let mut cells = BTreeSet::new();
    for instance in instances {
        cells.insert(instance.cell_id.clone());
    }
    cells.into_iter().collect()
}

fn evaluate_match(
    entry: &NasaHeuristicsBankEntry,
    profile: &HeuristicObservationProfile,
) -> HeuristicMatchResult {
    let mut satisfied_conditions = Vec::new();
    let mut unsatisfied_conditions = Vec::new();
    let mut total_conditions = 0usize;

    if !entry
        .match_criteria
        .required_primary_reason_codes
        .is_empty()
    {
        total_conditions += 1;
        let satisfied = profile
            .primary_reason_code
            .map(|code| {
                entry
                    .match_criteria
                    .required_primary_reason_codes
                    .contains(&code)
            })
            .unwrap_or(false);
        push_condition(
            satisfied,
            format!(
                "primary_reason_code in {:?}",
                entry.match_criteria.required_primary_reason_codes
            ),
            &mut satisfied_conditions,
            &mut unsatisfied_conditions,
        );
    }

    if !entry.match_criteria.required_final_states.is_empty() {
        total_conditions += 1;
        let satisfied = entry
            .match_criteria
            .required_final_states
            .contains(&profile.final_state);
        push_condition(
            satisfied,
            format!(
                "final_state in {:?}",
                entry.match_criteria.required_final_states
            ),
            &mut satisfied_conditions,
            &mut unsatisfied_conditions,
        );
    }

    evaluate_optional_bool_condition(
        entry.match_criteria.requires_first_boundary,
        profile.first_boundary_cycle.is_some(),
        "first_boundary_cycle present",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_first_violation,
        profile.first_violation_cycle.is_some(),
        "first_violation_cycle present",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry
            .match_criteria
            .requires_positive_lead_vs_threshold_baseline,
        profile
            .lead_time_vs_threshold_baseline
            .map(|lead| lead > 0)
            .unwrap_or(false),
        "positive lead vs threshold baseline",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_persistent_elevation_confirmed,
        profile.persistent_elevation_confirmed.unwrap_or(false),
        "persistent elevation confirmed",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry
            .match_criteria
            .requires_boundary_return_before_violation,
        profile.has_boundary_return_before_violation,
        "boundary return before first violation",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_violation_return,
        profile.has_violation_return,
        "violation return to admissible",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_recurrent_reentry_loop,
        profile.has_recurrent_reentry_loop,
        "recurrent reentry loop after violation return",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_monotone_escalation_only,
        profile.monotone_escalation_only,
        "monotone escalation only",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );
    evaluate_optional_bool_condition(
        entry.match_criteria.requires_final_violation_persistent,
        profile.final_violation_persistent,
        "terminal persistent violation",
        &mut total_conditions,
        &mut satisfied_conditions,
        &mut unsatisfied_conditions,
    );

    if let Some(min_lead) = entry.match_criteria.min_lead_time_vs_threshold_baseline {
        total_conditions += 1;
        let satisfied = profile
            .lead_time_vs_threshold_baseline
            .map(|lead| lead >= min_lead)
            .unwrap_or(false);
        push_condition(
            satisfied,
            format!("lead_time_vs_threshold_baseline >= {}", min_lead),
            &mut satisfied_conditions,
            &mut unsatisfied_conditions,
        );
    }

    if let Some(max_gap) = entry.match_criteria.max_boundary_to_violation_gap_cycles {
        total_conditions += 1;
        let satisfied = profile
            .boundary_to_violation_gap_cycles
            .map(|gap| gap <= max_gap)
            .unwrap_or(false);
        push_condition(
            satisfied,
            format!("boundary_to_violation_gap_cycles <= {}", max_gap),
            &mut satisfied_conditions,
            &mut unsatisfied_conditions,
        );
    }

    if let Some(min_run_length) = entry.match_criteria.min_terminal_violation_run_length {
        total_conditions += 1;
        let satisfied = profile
            .terminal_violation_run_length
            .map(|run_length| run_length >= min_run_length)
            .unwrap_or(false);
        push_condition(
            satisfied,
            format!("terminal_violation_run_length >= {}", min_run_length),
            &mut satisfied_conditions,
            &mut unsatisfied_conditions,
        );
    }

    let match_strength = if total_conditions == 0 {
        0.0
    } else {
        satisfied_conditions.len() as f64 / total_conditions as f64
    };

    let match_status = if total_conditions > 0 && unsatisfied_conditions.is_empty() {
        MatchStatus::Full
    } else if !satisfied_conditions.is_empty() {
        MatchStatus::Partial
    } else {
        MatchStatus::None
    };

    HeuristicMatchResult {
        heuristic_id: entry.id.clone(),
        match_status,
        match_strength,
        satisfied_conditions,
        unsatisfied_conditions,
        competing_matches: Vec::new(),
        ambiguity_level: AmbiguityLevel::Low,
    }
}

fn evaluate_optional_bool_condition(
    expected: Option<bool>,
    observed: bool,
    label: &str,
    total_conditions: &mut usize,
    satisfied_conditions: &mut Vec<String>,
    unsatisfied_conditions: &mut Vec<String>,
) {
    if let Some(expected_value) = expected {
        *total_conditions += 1;
        let satisfied = observed == expected_value;
        push_condition(
            satisfied,
            label.to_string(),
            satisfied_conditions,
            unsatisfied_conditions,
        );
    }
}

fn push_condition(
    satisfied: bool,
    label: String,
    satisfied_conditions: &mut Vec<String>,
    unsatisfied_conditions: &mut Vec<String>,
) {
    if satisfied {
        satisfied_conditions.push(label);
    } else {
        unsatisfied_conditions.push(label);
    }
}

fn grammar_rank(state: GrammarState) -> usize {
    match state {
        GrammarState::Admissible => 0,
        GrammarState::Boundary => 1,
        GrammarState::Violation => 2,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", stem, unique))
    }

    #[test]
    fn heuristics_bank_verification_passes_for_tracked_bank() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let verification = verify_heuristics_bank(crate_dir).unwrap();
        assert!(verification.verified);
        assert_eq!(verification.expected_sha256, verification.actual_sha256);
    }

    #[test]
    fn nasa_heuristics_bank_verification_passes_for_tracked_bank() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let verification = verify_nasa_heuristics_bank(crate_dir).unwrap();
        assert!(verification.verified);
        assert_eq!(verification.expected_sha256, verification.actual_sha256);
    }

    #[test]
    fn retrieval_is_deterministic_for_b0007() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let bank = load_nasa_heuristics_bank(crate_dir).unwrap();
        let data_path = crate_dir.join("data/nasa_b0007_capacity.csv");
        let raw_data = load_capacity_csv(&data_path).unwrap();
        let run = evaluate_cell(
            "B0007",
            data_path.to_string_lossy().as_ref(),
            &raw_data,
            &PipelineConfig::default(),
        )
        .unwrap();

        let profile = build_heuristic_observation_profile(&run.summary, &run.trajectory).unwrap();
        let first = retrieve_heuristic_matches(&bank, &profile);
        let second = retrieve_heuristic_matches(&bank, &profile);

        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert!(first.iter().any(|result| {
            result.heuristic_id == "HB-NASA-004" && result.match_status == MatchStatus::Full
        }));
    }

    #[test]
    fn heuristics_bank_workflow_writes_only_to_its_output_directory() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let data_dir = crate_dir.join("data");
        let output_dir = unique_temp_dir("dsfb-battery-heuristics-bank");

        let artifact =
            run_nasa_heuristics_bank_workflow(&data_dir, &output_dir, &PipelineConfig::default())
                .unwrap();

        assert_eq!(
            artifact.summary.artifact_type,
            "dsfb_battery_heuristics_bank_summary"
        );
        assert!(output_dir.join(NASA_HEURISTICS_SUMMARY_JSON).exists());
        assert!(output_dir.join(NASA_HEURISTICS_INVENTORY_JSON).exists());
        assert!(output_dir.join(NASA_HEURISTICS_EVIDENCE_JSON).exists());
        assert!(output_dir.join(NASA_HEURISTICS_RETRIEVAL_JSON).exists());
        assert!(output_dir.join(NASA_HEURISTICS_VERIFICATION_JSON).exists());
        assert!(output_dir
            .join(NASA_HEURISTICS_IMPLEMENTATION_SUMMARY)
            .exists());
        assert!(!output_dir.join("stage2_detection_results.json").exists());

        let output_names: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(|entry| {
                entry
                    .ok()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
            })
            .collect();
        assert!(!output_names
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn production_figure_filenames_remain_unchanged_for_heuristics_helper() {
        let expected = vec![
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
        ];

        assert_eq!(production_figure_filenames(), expected.as_slice());
    }
}
