use anyhow::{anyhow, Context, Result};

use crate::cli::args::CsvInputConfig;
use crate::engine::types::{ObservedTrajectory, PredictedTrajectory, VectorSample};

pub fn load_csv_trajectories(
    config: &CsvInputConfig,
) -> Result<(ObservedTrajectory, PredictedTrajectory)> {
    if config.dt_fallback <= 0.0 {
        return Err(anyhow!(
            "CSV ingestion requires a positive dt fallback when explicit time values are not supplied; got {}",
            config.dt_fallback
        ));
    }
    let observed = read_vector_csv(
        &config.observed_csv,
        &config.scenario_id,
        config.channel_names.as_deref(),
        config.time_column.as_deref(),
        config.dt_fallback,
    )
    .with_context(|| {
        format!(
            "failed to parse observed CSV {}",
            config.observed_csv.display()
        )
    })?;
    let predicted = read_vector_csv(
        &config.predicted_csv,
        &config.scenario_id,
        config.channel_names.as_deref(),
        config.time_column.as_deref(),
        config.dt_fallback,
    )
    .with_context(|| {
        format!(
            "failed to parse predicted CSV {}",
            config.predicted_csv.display()
        )
    })?;

    if observed.channel_names != predicted.channel_names {
        return Err(anyhow!(
            "observed and predicted CSV channel names differ: {:?} vs {:?}",
            observed.channel_names,
            predicted.channel_names
        ));
    }
    if observed.samples.len() != predicted.samples.len() {
        return Err(anyhow!(
            "observed and predicted CSV row counts differ: {} vs {}",
            observed.samples.len(),
            predicted.samples.len()
        ));
    }
    for (index, (observed_sample, predicted_sample)) in
        observed.samples.iter().zip(&predicted.samples).enumerate()
    {
        if observed_sample.step != predicted_sample.step {
            return Err(anyhow!(
                "step mismatch at row {}: {} vs {}",
                index,
                observed_sample.step,
                predicted_sample.step
            ));
        }
        if (observed_sample.time - predicted_sample.time).abs() > 1.0e-12 {
            return Err(anyhow!(
                "time mismatch at row {}: {} vs {}",
                index,
                observed_sample.time,
                predicted_sample.time
            ));
        }
        if observed_sample.values.len() != predicted_sample.values.len() {
            return Err(anyhow!(
                "channel width mismatch at row {}: {} vs {}",
                index,
                observed_sample.values.len(),
                predicted_sample.values.len()
            ));
        }
    }

    Ok((observed, observed_to_predicted(predicted)))
}

fn read_vector_csv(
    path: &std::path::Path,
    scenario_id: &str,
    override_channel_names: Option<&[String]>,
    time_column_name: Option<&str>,
    dt_fallback: f64,
) -> Result<ObservedTrajectory> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("failed to read headers from {}", path.display()))?
        .iter()
        .map(|header| header.trim().to_string())
        .collect::<Vec<_>>();

    let time_index = match time_column_name {
        Some(name) => Some(
            headers
                .iter()
                .position(|header| header == name)
                .ok_or_else(|| {
                    anyhow!(
                        "CSV {} does not contain the requested time column `{}`",
                        path.display(),
                        name
                    )
                })?,
        ),
        None => headers.iter().position(|header| header == "time"),
    };
    let step_index = headers.iter().position(|header| header == "step");
    let data_indices = headers
        .iter()
        .enumerate()
        .filter(|(index, header)| Some(*index) != time_index && *header != "step")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if data_indices.is_empty() {
        return Err(anyhow!(
            "CSV {} must contain at least one channel column besides optional step/time columns",
            path.display()
        ));
    }

    let channel_names = if let Some(names) = override_channel_names {
        if names.len() != data_indices.len() {
            return Err(anyhow!(
                "channel override length {} does not match CSV channel count {} for {}",
                names.len(),
                data_indices.len(),
                path.display()
            ));
        }
        names.to_vec()
    } else {
        for index in &data_indices {
            if headers[*index].is_empty() {
                return Err(anyhow!(
                    "CSV {} contains an empty channel header; supply explicit headers or use --channel-names",
                    path.display()
                ));
            }
        }
        data_indices
            .iter()
            .map(|index| headers[*index].clone())
            .collect::<Vec<_>>()
    };

    let mut samples = Vec::new();
    for (row_index, record) in reader.records().enumerate() {
        let record = record
            .with_context(|| format!("failed to read row {} from {}", row_index, path.display()))?;
        let step = match step_index {
            Some(index) => parse_usize(record.get(index), "step", row_index, path)?,
            None => row_index,
        };
        let time = match time_index {
            Some(index) => parse_f64(record.get(index), &headers[index], row_index, path)?,
            None => row_index as f64 * dt_fallback,
        };
        let values = data_indices
            .iter()
            .map(|index| parse_f64(record.get(*index), &headers[*index], row_index, path))
            .collect::<Result<Vec<_>>>()?;
        samples.push(VectorSample { step, time, values });
    }

    if samples.is_empty() {
        return Err(anyhow!("CSV {} contained no data rows", path.display()));
    }
    validate_sample_order(path, &samples, step_index.is_some(), time_index.is_some())?;

    Ok(ObservedTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names,
        samples,
    })
}

fn parse_f64(
    value: Option<&str>,
    column: &str,
    row_index: usize,
    path: &std::path::Path,
) -> Result<f64> {
    let raw = value.ok_or_else(|| {
        anyhow!(
            "missing value for column `{}` at row {} in {}",
            column,
            row_index,
            path.display()
        )
    })?;
    let parsed = raw.trim().parse::<f64>().with_context(|| {
        format!(
            "failed to parse column `{}` at row {} in {} as f64",
            column,
            row_index,
            path.display()
        )
    })?;
    if !parsed.is_finite() {
        return Err(anyhow!(
            "non-finite value for column `{}` at row {} in {}",
            column,
            row_index,
            path.display()
        ));
    }
    Ok(parsed)
}

fn parse_usize(
    value: Option<&str>,
    column: &str,
    row_index: usize,
    path: &std::path::Path,
) -> Result<usize> {
    value
        .ok_or_else(|| {
            anyhow!(
                "missing value for column `{}` at row {} in {}",
                column,
                row_index,
                path.display()
            )
        })?
        .trim()
        .parse::<usize>()
        .with_context(|| {
            format!(
                "failed to parse column `{}` at row {} in {} as usize",
                column,
                row_index,
                path.display()
            )
        })
}

pub fn observed_to_predicted(trajectory: ObservedTrajectory) -> PredictedTrajectory {
    PredictedTrajectory {
        scenario_id: trajectory.scenario_id,
        channel_names: trajectory.channel_names,
        samples: trajectory.samples,
    }
}

fn validate_sample_order(
    path: &std::path::Path,
    samples: &[VectorSample],
    explicit_step_column: bool,
    explicit_time_column: bool,
) -> Result<()> {
    for window in samples.windows(2) {
        let left = &window[0];
        let right = &window[1];
        if explicit_step_column && right.step <= left.step {
            return Err(anyhow!(
                "CSV {} must have strictly increasing step values; found {} then {}",
                path.display(),
                left.step,
                right.step
            ));
        }
        if explicit_time_column && right.time <= left.time {
            return Err(anyhow!(
                "CSV {} must have strictly increasing time values; found {} then {}",
                path.display(),
                left.time,
                right.time
            ));
        }
    }
    Ok(())
}
