// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing integration helpers, validity token, tactical margin, and
// external residual handoff evaluation.

use crate::detection::{assign_reason_code, detect_threshold_alarm, evaluate_grammar_state};
use crate::types::{BatteryResidual, GrammarState, PipelineConfig, ReasonCode, SignTuple};
use chrono::{Duration, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ValidityToken {
    pub token_kind: String,
    pub generated_at_utc: String,
    pub valid_until_utc: String,
    pub sequence_id: usize,
    pub output_present: bool,
    pub stream_valid: bool,
    pub advisory_only: bool,
    pub consumer_action_if_absent_or_stale: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TacticalMarginSummary {
    pub threshold_fraction: f64,
    pub threshold_cycle: Option<usize>,
    pub first_non_admissible_cycle: Option<usize>,
    pub lead_time_vs_margin_cycles: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KneeOnsetNarrative {
    pub emitted: bool,
    pub messages: Vec<String>,
    pub first_slew_threshold_cycle: Option<usize>,
    pub persistence_confirmed_cycle: Option<usize>,
    pub first_knee_reason_cycle: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalResidualSample {
    pub cycle: usize,
    pub residual: f64,
    pub drift: f64,
    pub slew: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalResidualEvaluation {
    pub artifact_type: String,
    pub envelope_rho: f64,
    pub first_boundary_cycle: Option<usize>,
    pub first_violation_cycle: Option<usize>,
    pub final_state: GrammarState,
    pub primary_reason_code: Option<ReasonCode>,
    pub validity_token: ValidityToken,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub validity_token: ValidityToken,
    pub tactical_margin: TacticalMarginSummary,
    pub knee_onset_narrative: KneeOnsetNarrative,
    pub external_residual_mode_supported: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineerExtensionSummary {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub workflows_added: Vec<String>,
    pub approximations: Vec<String>,
    pub scaffolds: Vec<String>,
    pub output_locations: Vec<String>,
    pub production_outputs_untouched: bool,
}

pub fn build_validity_token(
    sequence_id: usize,
    stream_valid: bool,
    freshness_seconds: i64,
) -> ValidityToken {
    let generated = Utc::now();
    ValidityToken {
        token_kind: "dsfb_output_freshness".to_string(),
        generated_at_utc: generated.to_rfc3339(),
        valid_until_utc: (generated + Duration::seconds(freshness_seconds)).to_rfc3339(),
        sequence_id,
        output_present: true,
        stream_valid,
        advisory_only: true,
        consumer_action_if_absent_or_stale:
            "Ignore the advisory DSFB output until a fresh token is present.".to_string(),
    }
}

pub fn compute_tactical_margin_summary(
    capacities: &[f64],
    trajectory: &[BatteryResidual],
    threshold_fraction: f64,
) -> TacticalMarginSummary {
    let threshold_cycle = detect_threshold_alarm(capacities, threshold_fraction);
    let first_non_admissible_cycle = trajectory
        .iter()
        .find(|sample| sample.grammar_state != GrammarState::Admissible)
        .map(|sample| sample.cycle);

    TacticalMarginSummary {
        threshold_fraction,
        threshold_cycle,
        first_non_admissible_cycle,
        lead_time_vs_margin_cycles: first_non_admissible_cycle
            .zip(threshold_cycle)
            .map(|(signal, threshold)| threshold as i64 - signal as i64),
    }
}

pub fn build_knee_onset_narrative(
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
) -> KneeOnsetNarrative {
    let mut slew_counter = 0usize;
    let mut drift_counter = 0usize;
    let mut first_slew_threshold_cycle = None;
    let mut persistence_confirmed_cycle = None;
    let first_knee_reason_cycle = trajectory.iter().find_map(|sample| {
        if sample.reason_code == Some(ReasonCode::AcceleratingFadeKnee) {
            Some(sample.cycle)
        } else {
            None
        }
    });

    for sample in trajectory {
        if sample.sign.d < -config.drift_threshold {
            drift_counter += 1;
        } else {
            drift_counter = 0;
        }

        if sample.sign.s < -config.slew_threshold {
            if first_slew_threshold_cycle.is_none() {
                first_slew_threshold_cycle = Some(sample.cycle);
            }
            slew_counter += 1;
        } else {
            slew_counter = 0;
        }

        if persistence_confirmed_cycle.is_none()
            && slew_counter >= config.slew_persistence
            && drift_counter >= config.drift_persistence
        {
            persistence_confirmed_cycle = Some(sample.cycle);
        }
    }

    let mut messages = Vec::new();
    if let Some(cycle) = first_slew_threshold_cycle {
        messages.push(format!(
            "Acceleration-style slew threshold exceeded at cycle {}; this is a structural helper narrative, not a mechanism label.",
            cycle
        ));
    }
    if let Some(cycle) = persistence_confirmed_cycle {
        messages.push(format!(
            "Slew threshold persistence was confirmed at cycle {} under the current drift and slew persistence rules.",
            cycle
        ));
    }
    if let Some(cycle) = first_knee_reason_cycle {
        messages.push(format!(
            "The current rule set first emitted AcceleratingFadeKnee at cycle {}.",
            cycle
        ));
    }

    KneeOnsetNarrative {
        emitted: !messages.is_empty(),
        messages,
        first_slew_threshold_cycle,
        persistence_confirmed_cycle,
        first_knee_reason_cycle,
    }
}

pub fn build_engineer_integration_artifact(
    capacities: &[f64],
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
    tactical_margin_fraction: f64,
) -> IntegrationArtifact {
    let final_cycle = trajectory.last().map(|sample| sample.cycle).unwrap_or(0);
    IntegrationArtifact {
        artifact_type: "dsfb_battery_engineer_integration_helper".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_advisory_helper".to_string(),
        validity_token: build_validity_token(final_cycle, true, 60),
        tactical_margin: compute_tactical_margin_summary(
            capacities,
            trajectory,
            tactical_margin_fraction,
        ),
        knee_onset_narrative: build_knee_onset_narrative(trajectory, config),
        external_residual_mode_supported: true,
        notes: vec![
            "This helper describes read-only, advisory integration behavior only.".to_string(),
            "No fail-silent or deployment certification claim is made.".to_string(),
        ],
    }
}

pub fn build_shadow_mode_integration_spec() -> String {
    [
        "# Shadow-Mode Integration Spec",
        "",
        "```text",
        "input telemetry -> upstream estimator/residual producer -> DSFB helper",
        "if validity token is absent or stale: ignore DSFB advisory output",
        "else: publish advisory-only classification and lead-time helpers",
        "no DSFB output feeds back into estimator tuning, actuation, or protection logic",
        "```",
        "",
        "Pseudocode:",
        "",
        "1. Read telemetry and upstream residuals in a read-only path.",
        "2. Hand residuals, drift, and slew into DSFB helper evaluation.",
        "3. Emit advisory-only output and a freshness token.",
        "4. If the token is missing or stale, a consuming system ignores the DSFB output.",
        "5. DSFB does not write into estimator state, controls, or protection thresholds.",
    ]
    .join("\n")
}

pub fn build_adaptive_residual_handoff_note() -> String {
    [
        "# Adaptive Residual Handoff Note",
        "",
        "The current production path derives residuals from capacity relative to a healthy baseline.",
        "For engineer-facing integration, the helper mode can also ingest external residual, drift, and slew sequences from an upstream observer.",
        "This crate does not claim empirical validation on adaptive ECM residuals in the current dataset.",
        "The intended handoff shape is: cycle, residual, drift, slew, envelope_rho.",
    ]
    .join("\n")
}

pub fn build_partial_observability_scaffold_note() -> String {
    [
        "# Partial Observability Scaffold",
        "",
        "The current production workflow is single-channel and capacity-only.",
        "No partial-observability experiment is executed here because the crate does not currently ingest additional production channels in the paper-facing path.",
        "When additional channels exist, a future helper can evaluate per-channel validity, fuse channel-local grammar states, and mark InsufficientObservability when required channels are absent.",
    ]
    .join("\n")
}

pub fn load_external_residual_csv(
    path: &Path,
) -> Result<Vec<ExternalResidualSample>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut samples = Vec::new();
    for record in reader.deserialize() {
        samples.push(record?);
    }
    Ok(samples)
}

impl<'de> serde::Deserialize<'de> for ExternalResidualSample {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Row {
            cycle: usize,
            residual: f64,
            drift: f64,
            slew: f64,
        }

        let row = Row::deserialize(deserializer)?;
        Ok(ExternalResidualSample {
            cycle: row.cycle,
            residual: row.residual,
            drift: row.drift,
            slew: row.slew,
        })
    }
}

pub fn build_external_residual_evaluation(
    samples: &[ExternalResidualSample],
    envelope_rho: f64,
    config: &PipelineConfig,
) -> ExternalResidualEvaluation {
    let mut drift_counter = 0usize;
    let mut slew_counter = 0usize;
    let mut first_boundary_cycle = None;
    let mut first_violation_cycle = None;
    let mut final_state = GrammarState::Admissible;
    let mut primary_reason_code = None;

    for sample in samples {
        if sample.drift < -config.drift_threshold {
            drift_counter += 1;
        } else {
            drift_counter = 0;
        }
        if sample.slew < -config.slew_threshold {
            slew_counter += 1;
        } else {
            slew_counter = 0;
        }

        let state = evaluate_grammar_state(
            sample.residual,
            &crate::types::EnvelopeParams {
                mu: 0.0,
                sigma: envelope_rho / 3.0,
                rho: envelope_rho,
            },
            sample.drift,
            sample.slew,
            drift_counter,
            slew_counter,
            config,
        );
        final_state = state;
        let sign = SignTuple {
            r: sample.residual,
            d: sample.drift,
            s: sample.slew,
        };
        if primary_reason_code.is_none() {
            primary_reason_code =
                assign_reason_code(&sign, state, drift_counter, slew_counter, config);
        }
        if first_boundary_cycle.is_none() && state == GrammarState::Boundary {
            first_boundary_cycle = Some(sample.cycle);
        }
        if first_violation_cycle.is_none() && state == GrammarState::Violation {
            first_violation_cycle = Some(sample.cycle);
        }
    }

    let last_cycle = samples.last().map(|sample| sample.cycle).unwrap_or(0);
    ExternalResidualEvaluation {
        artifact_type: "dsfb_battery_external_residual_evaluation".to_string(),
        envelope_rho,
        first_boundary_cycle,
        first_violation_cycle,
        final_state,
        primary_reason_code,
        validity_token: build_validity_token(last_cycle, true, 60),
        notes: vec![
            "This mode consumes externally supplied residual, drift, and slew values.".to_string(),
            "No empirical adaptive-observer validation claim is made by this helper.".to_string(),
        ],
    }
}

pub fn write_engineer_extension_summary(
    summary: &EngineerExtensionSummary,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = Vec::new();
    lines.push("DSFB Battery Engineer Extensions Summary".to_string());
    lines.push(format!("Generated at: {}", summary.generated_at_utc));
    lines.push("Implemented workflows:".to_string());
    for workflow in &summary.workflows_added {
        lines.push(format!("- {}", workflow));
    }
    lines.push("Approximations:".to_string());
    for item in &summary.approximations {
        lines.push(format!("- {}", item));
    }
    lines.push("Scaffolds:".to_string());
    for item in &summary.scaffolds {
        lines.push(format!("- {}", item));
    }
    lines.push("Output locations:".to_string());
    for location in &summary.output_locations {
        lines.push(format!("- {}", location));
    }
    lines.push(format!(
        "Production outputs untouched: {}",
        summary.production_outputs_untouched
    ));
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

pub fn hash_token_label(label: &str) -> String {
    let digest = Sha256::digest(label.as_bytes());
    let mut hex = String::new();
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", stem, unique))
    }

    #[test]
    fn knee_onset_narrative_emits_when_persistence_confirms() {
        let config = PipelineConfig {
            healthy_window: 2,
            drift_window: 1,
            drift_persistence: 2,
            slew_persistence: 2,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.8,
            boundary_fraction: 0.8,
        };
        let trajectory = vec![
            BatteryResidual {
                cycle: 1,
                capacity_ah: 2.0,
                sign: SignTuple {
                    r: 0.0,
                    d: 0.0,
                    s: 0.0,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 2,
                capacity_ah: 1.95,
                sign: SignTuple {
                    r: -0.02,
                    d: -0.003,
                    s: -0.0015,
                },
                grammar_state: GrammarState::Boundary,
                reason_code: Some(ReasonCode::SustainedCapacityFade),
            },
            BatteryResidual {
                cycle: 3,
                capacity_ah: 1.90,
                sign: SignTuple {
                    r: -0.05,
                    d: -0.0035,
                    s: -0.0016,
                },
                grammar_state: GrammarState::Boundary,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
        ];

        let narrative = build_knee_onset_narrative(&trajectory, &config);
        assert!(narrative.emitted);
        assert_eq!(narrative.first_slew_threshold_cycle, Some(2));
        assert_eq!(narrative.persistence_confirmed_cycle, Some(3));
    }

    #[test]
    fn external_residual_mode_evaluates_sequence() {
        let config = PipelineConfig::default();
        let samples = vec![
            ExternalResidualSample {
                cycle: 1,
                residual: 0.0,
                drift: 0.0,
                slew: 0.0,
            },
            ExternalResidualSample {
                cycle: 2,
                residual: -0.01,
                drift: -0.003,
                slew: -0.001,
            },
            ExternalResidualSample {
                cycle: 3,
                residual: -0.05,
                drift: -0.004,
                slew: -0.0012,
            },
        ];
        let evaluation = build_external_residual_evaluation(&samples, 0.04, &config);
        assert_eq!(evaluation.first_violation_cycle, Some(3));
        assert!(evaluation.validity_token.output_present);
    }

    #[test]
    fn integration_summary_writes_outside_production_paths() {
        let output_dir = unique_temp_dir("dsfb-battery-engineer-summary");
        let summary = EngineerExtensionSummary {
            artifact_type: "dsfb_battery_engineer_extensions".to_string(),
            generated_at_utc: Utc::now().to_rfc3339(),
            workflows_added: vec!["integration".to_string(), "complexity".to_string()],
            approximations: vec!["none".to_string()],
            scaffolds: vec!["partial observability".to_string()],
            output_locations: vec![output_dir.display().to_string()],
            production_outputs_untouched: true,
        };
        let path = output_dir.join("implementation_summary.txt");
        write_engineer_extension_summary(&summary, &path).unwrap();
        assert!(path.exists());
        let entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));
        let _ = fs::remove_dir_all(output_dir);
    }
}
