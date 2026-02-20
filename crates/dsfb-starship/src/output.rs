use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use plotters::prelude::*;
use serde::Serialize;

use crate::config::SimConfig;

#[derive(Debug, Clone, Serialize)]
pub struct SimRecord {
    pub time_s: f64,
    pub altitude_m: f64,
    pub speed_mps: f64,
    pub mach: f64,
    pub dynamic_pressure_pa: f64,
    pub heat_flux_w_m2: f64,
    pub heat_shield_temp_k: f64,
    pub blackout: bool,

    pub truth_x_km: f64,
    pub truth_y_km: f64,
    pub truth_z_km: f64,

    pub inertial_x_km: f64,
    pub inertial_y_km: f64,
    pub inertial_z_km: f64,
    pub ekf_x_km: f64,
    pub ekf_y_km: f64,
    pub ekf_z_km: f64,
    pub dsfb_x_km: f64,
    pub dsfb_y_km: f64,
    pub dsfb_z_km: f64,

    pub inertial_pos_err_m: f64,
    pub inertial_vel_err_mps: f64,
    pub inertial_att_err_deg: f64,
    pub ekf_pos_err_m: f64,
    pub ekf_vel_err_mps: f64,
    pub ekf_att_err_deg: f64,
    pub dsfb_pos_err_m: f64,
    pub dsfb_vel_err_mps: f64,
    pub dsfb_att_err_deg: f64,

    pub dsfb_trust_imu0: f64,
    pub dsfb_trust_imu1: f64,
    pub dsfb_trust_imu2: f64,
    pub dsfb_resid_inc_imu0: f64,
    pub dsfb_resid_inc_imu1: f64,
    pub dsfb_resid_inc_imu2: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodMetrics {
    pub rmse_position_m: f64,
    pub rmse_velocity_mps: f64,
    pub rmse_attitude_deg: f64,
    pub final_position_error_m: f64,
    pub max_position_error_m: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub config: SimConfig,
    pub samples: usize,
    pub blackout_start_s: Option<f64>,
    pub blackout_end_s: Option<f64>,
    pub blackout_duration_s: f64,
    pub inertial: MethodMetrics,
    pub ekf: MethodMetrics,
    pub dsfb: MethodMetrics,
    pub outputs: OutputFiles,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputFiles {
    pub output_dir: PathBuf,
    pub csv_path: PathBuf,
    pub summary_path: PathBuf,
    pub plot_altitude_path: PathBuf,
    pub plot_error_path: PathBuf,
    pub plot_trust_path: PathBuf,
}

pub fn write_csv(path: &Path, records: &[SimRecord]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to open CSV path {}", path.display()))?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_summary(path: &Path, summary: &Summary) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(summary)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn make_plots(records: &[SimRecord], files: &OutputFiles) -> anyhow::Result<()> {
    plot_altitude(records, &files.plot_altitude_path)?;
    plot_position_error(records, &files.plot_error_path)?;
    plot_trust(records, &files.plot_trust_path)?;
    Ok(())
}

fn plot_altitude(records: &[SimRecord], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_time = records.last().map(|r| r.time_s).unwrap_or(1.0);
    let max_alt = records
        .iter()
        .map(|r| r.altitude_m)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("Starship Re-entry Altitude", ("sans-serif", 34).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(0.0..max_time, 0.0..max_alt)?;

    chart
        .configure_mesh()
        .x_desc("Time [s]")
        .y_desc("Altitude [m]")
        .draw()?;

    chart.draw_series(LineSeries::new(
        records.iter().map(|r| (r.time_s, r.altitude_m)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

fn plot_position_error(records: &[SimRecord], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_time = records.last().map(|r| r.time_s).unwrap_or(1.0);
    let max_err = records
        .iter()
        .map(|r| {
            r.inertial_pos_err_m
                .max(r.ekf_pos_err_m)
                .max(r.dsfb_pos_err_m)
                .max(1.0)
        })
        .fold(1.0_f64, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Position Error Comparison (Log Scale)",
            ("sans-serif", 34).into_font(),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(0.0..max_time, (1.0_f64..max_err).log_scale())?;

    chart
        .configure_mesh()
        .x_desc("Time [s]")
        .y_desc("Position Error [m]")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.inertial_pos_err_m.max(1.0))),
            &RED,
        ))?
        .label("Pure Inertial")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], RED.stroke_width(3)));

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.ekf_pos_err_m.max(1.0))),
            &GREEN,
        ))?
        .label("Simple EKF")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], GREEN.stroke_width(3)));

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.dsfb_pos_err_m.max(1.0))),
            &BLUE,
        ))?
        .label("DSFB")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], BLUE.stroke_width(3)));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .border_style(BLACK)
        .background_style(WHITE.mix(0.7))
        .draw()?;

    root.present()?;
    Ok(())
}

fn plot_trust(records: &[SimRecord], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_time = records.last().map(|r| r.time_s).unwrap_or(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("DSFB Trust Weights", ("sans-serif", 34).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_time, 0.0..1.0)?;

    chart
        .configure_mesh()
        .x_desc("Time [s]")
        .y_desc("Trust Weight")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.dsfb_trust_imu0)),
            &BLUE,
        ))?
        .label("IMU-0")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], BLUE.stroke_width(3)));

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.dsfb_trust_imu1)),
            &RED,
        ))?
        .label("IMU-1")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], RED.stroke_width(3)));

    chart
        .draw_series(LineSeries::new(
            records.iter().map(|r| (r.time_s, r.dsfb_trust_imu2)),
            &GREEN,
        ))?
        .label("IMU-2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], GREEN.stroke_width(3)));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerLeft)
        .border_style(BLACK)
        .background_style(WHITE.mix(0.7))
        .draw()?;

    root.present()?;
    Ok(())
}
