// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Export utilities
//
// CSV and JSON artifact export for semiotic trajectory data and
// detection results.

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
            br.reason_code
                .map(|rc| rc.to_string())
                .unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Stage II detection results JSON schema.
#[derive(Debug, serde::Serialize)]
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
                .start_file(
                    name.to_string_lossy().as_ref(),
                    options,
                )
                .map_err(|e| ExportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let mut f = std::fs::File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            archive
                .write_all(&buf)?;
        }
    }
    archive
        .finish()
        .map_err(|e| ExportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}
