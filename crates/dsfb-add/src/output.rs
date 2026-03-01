use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use csv::Writer;

use crate::{rlt::RltTrajectoryPoint, AddError, TcpPoint};

#[derive(Debug, Clone)]
pub struct PhaseBoundaryRow {
    pub steps_per_run: usize,
    pub is_perturbed: bool,
    pub lambda_star: Option<f64>,
    pub lambda_0_1: Option<f64>,
    pub lambda_0_9: Option<f64>,
    pub transition_width: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct RobustnessMetricRow {
    pub subsystem: String,
    pub steps_per_run: usize,
    pub metric_name: String,
    pub value: f64,
}

pub fn repo_root_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

pub fn create_timestamped_output_dir() -> Result<PathBuf, AddError> {
    let output_root = repo_root_dir().join("output-dsfb-add");
    fs::create_dir_all(&output_root)?;

    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let mut output_dir = output_root.join(&timestamp);
    let mut counter = 1_u32;

    while output_dir.exists() {
        output_dir = output_root.join(format!("{timestamp}-{counter:02}"));
        counter += 1;
    }

    fs::create_dir_all(&output_dir)?;
    Ok(output_dir)
}

fn ensure_len(context: &'static str, expected: usize, actual: usize) -> Result<(), AddError> {
    if expected == actual {
        return Ok(());
    }

    Err(AddError::LengthMismatch {
        context,
        expected,
        got: actual,
    })
}

fn fmt_f64(value: f64) -> String {
    format!("{value:.10}")
}

fn fmt_option_f64(value: Option<f64>) -> String {
    value.map(fmt_f64).unwrap_or_default()
}

pub fn write_aet_csv(
    path: &Path,
    lambda_grid: &[f64],
    echo_slope: &[f64],
    avg_increment: &[f64],
    steps_per_run: usize,
    is_perturbed: bool,
) -> Result<(), AddError> {
    ensure_len("aet echo_slope", lambda_grid.len(), echo_slope.len())?;
    ensure_len("aet avg_increment", lambda_grid.len(), avg_increment.len())?;

    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "lambda",
        "echo_slope",
        "avg_increment",
        "steps_per_run",
        "is_perturbed",
    ])?;

    for idx in 0..lambda_grid.len() {
        writer.write_record([
            fmt_f64(lambda_grid[idx]),
            fmt_f64(echo_slope[idx]),
            fmt_f64(avg_increment[idx]),
            steps_per_run.to_string(),
            is_perturbed.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_tcp_csv(
    path: &Path,
    lambda_grid: &[f64],
    betti0: &[usize],
    betti1: &[usize],
    l_tcp: &[f64],
    avg_radius: &[f64],
    max_radius: &[f64],
    variance_radius: &[f64],
    steps_per_run: usize,
    is_perturbed: bool,
) -> Result<(), AddError> {
    ensure_len("tcp betti0", lambda_grid.len(), betti0.len())?;
    ensure_len("tcp betti1", lambda_grid.len(), betti1.len())?;
    ensure_len("tcp l_tcp", lambda_grid.len(), l_tcp.len())?;
    ensure_len("tcp avg_radius", lambda_grid.len(), avg_radius.len())?;
    ensure_len("tcp max_radius", lambda_grid.len(), max_radius.len())?;
    ensure_len(
        "tcp variance_radius",
        lambda_grid.len(),
        variance_radius.len(),
    )?;

    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "lambda",
        "betti0",
        "betti1",
        "l_tcp",
        "avg_radius",
        "max_radius",
        "variance_radius",
        "steps_per_run",
        "is_perturbed",
    ])?;

    for idx in 0..lambda_grid.len() {
        writer.write_record([
            fmt_f64(lambda_grid[idx]),
            betti0[idx].to_string(),
            betti1[idx].to_string(),
            fmt_f64(l_tcp[idx]),
            fmt_f64(avg_radius[idx]),
            fmt_f64(max_radius[idx]),
            fmt_f64(variance_radius[idx]),
            steps_per_run.to_string(),
            is_perturbed.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_rlt_csv(
    path: &Path,
    lambda_grid: &[f64],
    escape_rate: &[f64],
    expansion_ratio: &[f64],
    steps_per_run: usize,
    is_perturbed: bool,
) -> Result<(), AddError> {
    ensure_len("rlt escape_rate", lambda_grid.len(), escape_rate.len())?;
    ensure_len(
        "rlt expansion_ratio",
        lambda_grid.len(),
        expansion_ratio.len(),
    )?;

    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "lambda",
        "escape_rate",
        "expansion_ratio",
        "steps_per_run",
        "is_perturbed",
    ])?;

    for idx in 0..lambda_grid.len() {
        writer.write_record([
            fmt_f64(lambda_grid[idx]),
            fmt_f64(escape_rate[idx]),
            fmt_f64(expansion_ratio[idx]),
            steps_per_run.to_string(),
            is_perturbed.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_iwlt_csv(
    path: &Path,
    lambda_grid: &[f64],
    entropy_density: &[f64],
    avg_increment: &[f64],
    steps_per_run: usize,
    is_perturbed: bool,
) -> Result<(), AddError> {
    ensure_len(
        "iwlt entropy_density",
        lambda_grid.len(),
        entropy_density.len(),
    )?;
    ensure_len("iwlt avg_increment", lambda_grid.len(), avg_increment.len())?;

    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "lambda",
        "entropy_density",
        "avg_increment",
        "steps_per_run",
        "is_perturbed",
    ])?;

    for idx in 0..lambda_grid.len() {
        writer.write_record([
            fmt_f64(lambda_grid[idx]),
            fmt_f64(entropy_density[idx]),
            fmt_f64(avg_increment[idx]),
            steps_per_run.to_string(),
            is_perturbed.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_tcp_points_csv(path: &Path, points: &[TcpPoint]) -> Result<(), AddError> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["t", "x", "y"])?;

    for point in points {
        writer.write_record([point.t.to_string(), fmt_f64(point.x), fmt_f64(point.y)])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_rlt_trajectory_csv(
    path: &Path,
    points: &[RltTrajectoryPoint],
) -> Result<(), AddError> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "step",
        "lambda",
        "vertex_id",
        "x",
        "y",
        "distance_from_start",
    ])?;

    for point in points {
        writer.write_record([
            point.step.to_string(),
            fmt_f64(point.lambda),
            point.vertex_id.to_string(),
            point.x.to_string(),
            point.y.to_string(),
            point.distance_from_start.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_rlt_phase_boundary_csv(
    path: &Path,
    rows: &[PhaseBoundaryRow],
) -> Result<(), AddError> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record([
        "steps_per_run",
        "is_perturbed",
        "lambda_star",
        "lambda_0_1",
        "lambda_0_9",
        "transition_width",
    ])?;

    for row in rows {
        writer.write_record([
            row.steps_per_run.to_string(),
            row.is_perturbed.to_string(),
            fmt_option_f64(row.lambda_star),
            fmt_option_f64(row.lambda_0_1),
            fmt_option_f64(row.lambda_0_9),
            fmt_option_f64(row.transition_width),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_robustness_metrics_csv(
    path: &Path,
    rows: &[RobustnessMetricRow],
) -> Result<(), AddError> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["subsystem", "steps_per_run", "metric_name", "value"])?;

    for row in rows {
        writer.write_record([
            row.subsystem.clone(),
            row.steps_per_run.to_string(),
            row.metric_name.clone(),
            fmt_f64(row.value),
        ])?;
    }

    writer.flush()?;
    Ok(())
}
