// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Export utilities
//
// CSV and JSON artifact export for semiotic trajectory data and
// detection results.

use crate::audit::Stage2AuditTraceArtifact;
use crate::types::{BatteryResidual, DetectionResult, Theorem1Result};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// Errors arising from export operations.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV write error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Export the per-cycle semiotic trajectory to CSV.
///
/// Columns: cycle, capacity_ah, residual, drift, slew, grammar_state, reason_code
pub fn export_trajectory_csv(
    trajectory: &[BatteryResidual],
    path: &Path,
) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "cycle",
        "capacity_ah",
        "residual",
        "drift",
        "slew",
        "grammar_state",
        "reason_code",
    ])?;
    for br in trajectory {
        wtr.write_record(&[
            br.cycle.to_string(),
            format!("{:.6}", br.capacity_ah),
            format!("{:.6}", br.sign.r),
            format!("{:.6}", br.sign.d),
            format!("{:.6}", br.sign.s),
            br.grammar_state.to_string(),
            br.reason_code.map(|rc| rc.to_string()).unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Stage II detection results JSON schema.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Stage2Results {
    /// Data provenance description.
    pub data_provenance: String,
    /// Pipeline configuration used.
    pub config: crate::types::PipelineConfig,
    /// Envelope parameters computed from healthy window.
    pub envelope: crate::types::EnvelopeParams,
    /// DSFB detection result.
    pub dsfb_detection: DetectionResult,
    /// Threshold baseline detection result.
    pub threshold_detection: DetectionResult,
    /// Theorem 1 verification result.
    pub theorem1: Theorem1Result,
}

/// Export detection results to JSON.
pub fn export_results_json(results: &Stage2Results, path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(results)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export the audit-trace contract for the Stage II artifact.
pub fn export_audit_trace_json(
    artifact: &Stage2AuditTraceArtifact,
    path: &Path,
) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(artifact)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Package an entire output directory into a ZIP archive.
///
/// All files directly inside `dir` are included in the ZIP.
pub fn export_zip(dir: &Path, zip_path: &Path) -> Result<(), ExportError> {
    use std::io::Read;
    let file = std::fs::File::create(zip_path)?;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = entry.file_name();
            archive
                .start_file(name.to_string_lossy().as_ref(), options)
                .map_err(|e| ExportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let mut f = std::fs::File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            archive.write_all(&buf)?;
        }
    }
    archive
        .finish()
        .map_err(|e| ExportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{build_stage2_audit_trace, AuditTraceBuildContext};
    use crate::types::{
        BatteryResidual, EnvelopeParams, GrammarState, PipelineConfig, ReasonCode, SignTuple,
    };
    use serde_json::Value;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_results() -> (Stage2Results, Vec<(usize, f64)>, Vec<BatteryResidual>) {
        let config = PipelineConfig {
            healthy_window: 2,
            drift_window: 1,
            drift_persistence: 1,
            slew_persistence: 1,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        };

        let raw_input = vec![
            (1, 2.0000),
            (2, 1.9950),
            (3, 1.9200),
            (4, 1.6800),
            (5, 1.5800),
        ];

        let trajectory = vec![
            BatteryResidual {
                cycle: 1,
                capacity_ah: 2.0000,
                sign: SignTuple {
                    r: 0.0025,
                    d: 0.0,
                    s: 0.0,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 2,
                capacity_ah: 1.9950,
                sign: SignTuple {
                    r: -0.0025,
                    d: -0.0015,
                    s: -0.0005,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 3,
                capacity_ah: 1.9200,
                sign: SignTuple {
                    r: -0.0400,
                    d: -0.0030,
                    s: -0.0012,
                },
                grammar_state: GrammarState::Boundary,
                reason_code: Some(ReasonCode::SustainedCapacityFade),
            },
            BatteryResidual {
                cycle: 4,
                capacity_ah: 1.6800,
                sign: SignTuple {
                    r: -0.0850,
                    d: -0.0040,
                    s: -0.0018,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
            BatteryResidual {
                cycle: 5,
                capacity_ah: 1.5800,
                sign: SignTuple {
                    r: -0.1200,
                    d: -0.0045,
                    s: -0.0015,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
        ];

        let results = Stage2Results {
            data_provenance:
                "NASA PCoE Battery Dataset, Cell B0005 (capacity-only synthetic contract sample)"
                    .to_string(),
            config,
            envelope: EnvelopeParams {
                mu: 1.9975,
                sigma: 0.0167,
                rho: 0.0500,
            },
            dsfb_detection: DetectionResult {
                method: "DSFB Structural Alarm".to_string(),
                alarm_cycle: Some(3),
                eol_cycle: Some(5),
                lead_time_cycles: Some(2),
            },
            threshold_detection: DetectionResult {
                method: "Threshold Baseline (85% of initial)".to_string(),
                alarm_cycle: Some(4),
                eol_cycle: Some(5),
                lead_time_cycles: Some(1),
            },
            theorem1: Theorem1Result {
                rho: 0.0500,
                alpha: 0.0035,
                kappa: 0.0,
                t_star: 15,
                actual_detection_cycle: Some(3),
                bound_satisfied: Some(true),
            },
        };

        (results, raw_input, trajectory)
    }

    #[test]
    fn audit_trace_export_sets_expected_artifact_type() {
        let (results, raw_input, trajectory) = sample_results();
        let figures = vec!["fig06_grammar_state_timeline.svg".to_string()];
        let tables = vec!["semiotic_trajectory.csv".to_string()];
        let artifact = build_stage2_audit_trace(AuditTraceBuildContext {
            results: &results,
            raw_input: &raw_input,
            trajectory: &trajectory,
            source_artifact: Some(std::path::Path::new("data/nasa_b0005_capacity.csv")),
            supporting_figures: &figures,
            supporting_tables: &tables,
            dataset_name: None,
            cell_id: None,
            benchmark_id: None,
            regime_tag: None,
        })
        .unwrap();

        let value = serde_json::to_value(&artifact).unwrap();
        assert_eq!(value["artifact_type"], "dsfb_battery_audit_trace");
        assert_eq!(value["output_contract"]["kind"], "audit_trace");
        assert_eq!(value["data_provenance"], results.data_provenance);
    }

    #[test]
    fn stage2_detection_results_emission_still_succeeds() {
        let (results, raw_input, trajectory) = sample_results();
        let figures = vec![
            "fig01_capacity_fade.svg".to_string(),
            "fig06_grammar_state_timeline.svg".to_string(),
        ];
        let tables = vec!["semiotic_trajectory.csv".to_string()];
        let artifact = build_stage2_audit_trace(AuditTraceBuildContext {
            results: &results,
            raw_input: &raw_input,
            trajectory: &trajectory,
            source_artifact: Some(std::path::Path::new("data/nasa_b0005_capacity.csv")),
            supporting_figures: &figures,
            supporting_tables: &tables,
            dataset_name: None,
            cell_id: None,
            benchmark_id: None,
            regime_tag: None,
        })
        .unwrap();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dsfb-battery-audit-export-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("stage2_detection_results.json");

        export_audit_trace_json(&artifact, &path).unwrap();

        assert!(path.exists());

        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            value["artifact_manifest"]["primary_json"],
            "stage2_detection_results.json"
        );
        assert_eq!(value["summary_outcome"]["first_boundary_cycle"], 3);
        assert_eq!(value["summary_outcome"]["capacity_85pct_cycle"], 4);
        assert_eq!(value["summary_outcome"]["capacity_80pct_cycle"], 5);
        assert_eq!(value["summary_outcome"]["t_star"], 15);

        let _ = fs::remove_dir_all(&dir);
    }
}
