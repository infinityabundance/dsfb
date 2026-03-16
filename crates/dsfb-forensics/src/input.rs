//! Trace loading and validation.
//!
//! References: `DSFB-03`, `DSFB-07`, and `DSFB-08`. The loader enforces a
//! deterministic forward-image schema so residual semantics stay explicit and
//! replayable across runs.

use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Optional truth state carried by an input step.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct TruthState {
    /// Structural phase or position.
    pub phi: f64,
    /// Structural drift or velocity.
    pub omega: f64,
    /// Structural slew or acceleration.
    pub alpha: f64,
}

/// One input trace step.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TraceStep {
    /// Sequence index.
    pub step: usize,
    /// Positive time delta in seconds.
    pub dt: f64,
    /// Scalar observations, one per channel.
    pub measurements: Vec<f64>,
    /// Optional truth state for error analysis.
    #[serde(default)]
    pub truth: Option<TruthState>,
}

/// Full trace document loaded from CSV or JSON.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TraceDocument {
    /// Stable channel names in input order.
    pub channel_names: Vec<String>,
    /// Ordered trace steps.
    pub steps: Vec<TraceStep>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonTrace {
    Document(TraceDocument),
    Steps(Vec<TraceStep>),
}

/// Load a trace from CSV or JSON.
///
/// References: `DSFB-06`, `DSFB-07`, and `CORE-10`.
pub fn load_trace(path: &Path) -> Result<TraceDocument> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let document = match extension {
        "json" => load_json_trace(path)?,
        "csv" => load_csv_trace(path)?,
        _ => bail!(
            "unsupported trace format for {}: expected .csv or .json",
            path.display()
        ),
    };
    validate_trace(&document)?;
    Ok(document)
}

fn load_json_trace(path: &Path) -> Result<TraceDocument> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    match serde_json::from_reader::<_, JsonTrace>(file)
        .with_context(|| format!("failed to parse {}", path.display()))?
    {
        JsonTrace::Document(document) => Ok(document),
        JsonTrace::Steps(steps) => Ok(TraceDocument {
            channel_names: default_channel_names(steps.first().map_or(0, |step| step.measurements.len())),
            steps,
        }),
    }
}

fn load_csv_trace(path: &Path) -> Result<TraceDocument> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("failed to read headers from {}", path.display()))?
        .clone();

    let step_index = headers.iter().position(|header| header == "step");
    let dt_index = header_index(&headers, "dt")?;
    let truth_phi_index = headers.iter().position(|header| header == "truth_phi");
    let truth_omega_index = headers.iter().position(|header| header == "truth_omega");
    let truth_alpha_index = headers.iter().position(|header| header == "truth_alpha");

    let mut measurement_columns = Vec::new();
    let mut channel_names = Vec::new();
    for (index, header) in headers.iter().enumerate() {
        if header.starts_with("measurement_") {
            measurement_columns.push(index);
            channel_names.push(header.to_string());
        }
    }
    if measurement_columns.is_empty() {
        bail!(
            "trace {} must contain at least one measurement_* column",
            path.display()
        );
    }

    let mut steps = Vec::new();
    for (row_index, row) in reader.records().enumerate() {
        let row = row.with_context(|| format!("failed to read row {} from {}", row_index, path.display()))?;
        let step = match step_index {
            Some(index) => parse_usize(cell(&row, index, "step", path)?, "step", path)?,
            None => row_index,
        };
        let dt = parse_f64(cell(&row, dt_index, "dt", path)?, "dt", path)?;

        let mut measurements = Vec::with_capacity(measurement_columns.len());
        for &index in &measurement_columns {
            measurements.push(parse_f64(
                cell(&row, index, headers.get(index).unwrap_or("measurement"), path)?,
                headers.get(index).unwrap_or("measurement"),
                path,
            )?);
        }

        let truth = match (truth_phi_index, truth_omega_index, truth_alpha_index) {
            (Some(phi), Some(omega), Some(alpha)) => Some(TruthState {
                phi: parse_f64(cell(&row, phi, "truth_phi", path)?, "truth_phi", path)?,
                omega: parse_f64(cell(&row, omega, "truth_omega", path)?, "truth_omega", path)?,
                alpha: parse_f64(cell(&row, alpha, "truth_alpha", path)?, "truth_alpha", path)?,
            }),
            _ => None,
        };

        steps.push(TraceStep {
            step,
            dt,
            measurements,
            truth,
        });
    }

    Ok(TraceDocument {
        channel_names,
        steps,
    })
}

fn validate_trace(document: &TraceDocument) -> Result<()> {
    if document.steps.is_empty() {
        bail!("trace must contain at least one step");
    }
    if document.channel_names.is_empty() {
        bail!("trace must contain at least one channel");
    }

    let channel_count = document.channel_names.len();
    for (index, step) in document.steps.iter().enumerate() {
        if step.measurements.len() != channel_count {
            bail!(
                "step {} has {} measurements but {} channels were declared",
                index,
                step.measurements.len(),
                channel_count
            );
        }
        if !(step.dt.is_finite() && step.dt > 0.0) {
            bail!("step {} has non-positive dt {}", index, step.dt);
        }
        if !step.measurements.iter().all(|value| value.is_finite()) {
            bail!("step {} contains a non-finite measurement", index);
        }
        if let Some(truth) = step.truth {
            if !(truth.phi.is_finite() && truth.omega.is_finite() && truth.alpha.is_finite()) {
                bail!("step {} contains a non-finite truth state", index);
            }
        }
    }

    Ok(())
}

fn default_channel_names(channel_count: usize) -> Vec<String> {
    (0..channel_count)
        .map(|index| format!("channel_{index}"))
        .collect()
}

fn header_index(headers: &csv::StringRecord, target: &str) -> Result<usize> {
    headers
        .iter()
        .position(|header| header == target)
        .ok_or_else(|| anyhow!("missing required column `{target}`"))
}

fn cell<'a>(row: &'a csv::StringRecord, index: usize, label: &str, path: &Path) -> Result<&'a str> {
    row.get(index)
        .ok_or_else(|| anyhow!("missing {label} field in {}", path.display()))
}

fn parse_f64(text: &str, label: &str, path: &Path) -> Result<f64> {
    text.parse::<f64>()
        .with_context(|| format!("failed to parse {label} in {}", path.display()))
}

fn parse_usize(text: &str, label: &str, path: &Path) -> Result<usize> {
    text.parse::<usize>()
        .with_context(|| format!("failed to parse {label} in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_trace() {
        let document = TraceDocument {
            channel_names: vec!["measurement_0".to_string()],
            steps: Vec::new(),
        };
        assert!(validate_trace(&document).is_err());
    }
}
