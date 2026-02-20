pub mod config;
pub mod estimators;
pub mod output;
pub mod physics;
pub mod sensors;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::Utc;
use nalgebra::Vector3;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::StandardNormal;

use crate::config::SimConfig;
use crate::estimators::{mean_measurement, DsfbFusionLayer, NavState, SimpleEkf};
use crate::output::{make_plots, write_csv, write_summary, MethodMetrics, OutputFiles, SimRecord, Summary};
use crate::physics::{initial_truth_state, truth_step, ReentryEventState, VehicleParams};
use crate::sensors::ImuArray;

pub fn run_simulation(cfg: &SimConfig, output_dir: &Path) -> anyhow::Result<Summary> {
    cfg.validate()?;
    let output_base_dir = resolve_output_base_dir(output_dir);
    let output_dir = create_timestamped_run_dir(&output_base_dir)?;

    let vehicle = VehicleParams::default();
    let mut truth = initial_truth_state(cfg, &vehicle);
    let mut events = ReentryEventState::default();
    let mut imu_array = ImuArray::new(cfg.seed, cfg.imu_count);

    let mut inertial = NavState::from_truth_with_seed_error(&truth, 1.00);
    let mut ekf = SimpleEkf::new(NavState::from_truth_with_seed_error(&truth, 1.12));
    let mut dsfb_nav = NavState::from_truth_with_seed_error(&truth, 0.86);
    let mut dsfb_fusion = DsfbFusionLayer::new(cfg);

    let mut gnss_rng = ChaCha8Rng::seed_from_u64(cfg.seed ^ 0xCAB00D1E_u64);

    let mut records = Vec::with_capacity(cfg.steps());

    let mut blackout_start: Option<f64> = None;
    let mut blackout_end: Option<f64> = None;

    for step_idx in 0..cfg.steps() {
        let t_s = step_idx as f64 * cfg.dt;

        let truth_sample = truth_step(&mut truth, &vehicle, cfg, t_s, cfg.dt, &mut events);
        let imu_measurements = imu_array.measure(
            truth_sample.aero.specific_force_b_mps2,
            truth.omega_b_rps,
            truth.heat_shield_temp_k,
            t_s,
            &events,
        );

        // Pure inertial baseline: first IMU only.
        if let Some(primary) = imu_measurements.first() {
            inertial.propagate(primary.accel_b_mps2, primary.gyro_b_rps, cfg.dt);
        }

        // Simple EKF baseline: average IMU propagation + GNSS update when not in blackout.
        let mean_imu = mean_measurement(&imu_measurements);
        ekf.propagate(mean_imu.accel_b_mps2, mean_imu.gyro_b_rps, cfg.dt);

        // DSFB fusion over redundant IMUs.
        let dsfb_out = dsfb_fusion.fuse(&imu_measurements, cfg.dt);
        dsfb_nav.propagate(dsfb_out.fused_accel_b_mps2, dsfb_out.fused_gyro_b_rps, cfg.dt);

        if !finite_nav(&truth.pos_n_m, &truth.vel_n_mps)
            || !finite_nav(&inertial.pos_n_m, &inertial.vel_n_mps)
            || !finite_nav(&ekf.nav.pos_n_m, &ekf.nav.vel_n_mps)
            || !finite_nav(&dsfb_nav.pos_n_m, &dsfb_nav.vel_n_mps)
        {
            break;
        }

        let is_blackout = truth_sample.blackout;
        if is_blackout {
            if blackout_start.is_none() {
                blackout_start = Some(t_s);
            }
        } else if blackout_start.is_some() && blackout_end.is_none() {
            blackout_end = Some(t_s);
        }

        // GNSS aiding outside blackout at 1 Hz.
        if !is_blackout && step_idx % (1.0 / cfg.dt).round().max(1.0) as usize == 0 {
            let gnss_pos = truth.pos_n_m
                + Vector3::new(
                    gaussian(&mut gnss_rng, 5.5),
                    gaussian(&mut gnss_rng, 5.5),
                    gaussian(&mut gnss_rng, 7.0),
                );
            let gnss_vel = truth.vel_n_mps
                + Vector3::new(
                    gaussian(&mut gnss_rng, 0.75),
                    gaussian(&mut gnss_rng, 0.75),
                    gaussian(&mut gnss_rng, 0.90),
                );

            ekf.update_gnss(gnss_pos, gnss_vel);

            dsfb_nav.pos_n_m = dsfb_nav.pos_n_m * 0.75 + gnss_pos * 0.25;
            dsfb_nav.vel_n_mps = dsfb_nav.vel_n_mps * 0.70 + gnss_vel * 0.30;
        }

        let trust_imu0 = *dsfb_out.trust_weights.first().unwrap_or(&0.0);
        let trust_imu1 = *dsfb_out.trust_weights.get(1).unwrap_or(&0.0);
        let trust_imu2 = *dsfb_out.trust_weights.get(2).unwrap_or(&0.0);

        let resid_imu0 = *dsfb_out.residual_increments.first().unwrap_or(&0.0);
        let resid_imu1 = *dsfb_out.residual_increments.get(1).unwrap_or(&0.0);
        let resid_imu2 = *dsfb_out.residual_increments.get(2).unwrap_or(&0.0);

        records.push(SimRecord {
            time_s: t_s,
            altitude_m: truth.altitude_m(),
            speed_mps: truth.vel_n_mps.norm(),
            mach: truth_sample.aero.mach,
            dynamic_pressure_pa: truth_sample.aero.dynamic_pressure_pa,
            heat_flux_w_m2: truth_sample.heat_flux_w_m2,
            heat_shield_temp_k: truth.heat_shield_temp_k,
            blackout: is_blackout,

            truth_x_km: truth.pos_n_m.x / 1_000.0,
            truth_y_km: truth.pos_n_m.y / 1_000.0,
            truth_z_km: truth.pos_n_m.z / 1_000.0,

            inertial_x_km: inertial.pos_n_m.x / 1_000.0,
            inertial_y_km: inertial.pos_n_m.y / 1_000.0,
            inertial_z_km: inertial.pos_n_m.z / 1_000.0,
            ekf_x_km: ekf.nav.pos_n_m.x / 1_000.0,
            ekf_y_km: ekf.nav.pos_n_m.y / 1_000.0,
            ekf_z_km: ekf.nav.pos_n_m.z / 1_000.0,
            dsfb_x_km: dsfb_nav.pos_n_m.x / 1_000.0,
            dsfb_y_km: dsfb_nav.pos_n_m.y / 1_000.0,
            dsfb_z_km: dsfb_nav.pos_n_m.z / 1_000.0,

            inertial_pos_err_m: inertial.position_error_m(&truth),
            inertial_vel_err_mps: inertial.velocity_error_mps(&truth),
            inertial_att_err_deg: inertial.attitude_error_deg(&truth),
            ekf_pos_err_m: ekf.nav.position_error_m(&truth),
            ekf_vel_err_mps: ekf.nav.velocity_error_mps(&truth),
            ekf_att_err_deg: ekf.nav.attitude_error_deg(&truth),
            dsfb_pos_err_m: dsfb_nav.position_error_m(&truth),
            dsfb_vel_err_mps: dsfb_nav.velocity_error_mps(&truth),
            dsfb_att_err_deg: dsfb_nav.attitude_error_deg(&truth),

            dsfb_trust_imu0: trust_imu0,
            dsfb_trust_imu1: trust_imu1,
            dsfb_trust_imu2: trust_imu2,
            dsfb_resid_inc_imu0: resid_imu0,
            dsfb_resid_inc_imu1: resid_imu1,
            dsfb_resid_inc_imu2: resid_imu2,
        });

        if truth.altitude_m() <= 18_000.0 {
            break;
        }
    }

    let blackout_duration_s = if let (Some(start), Some(end)) = (blackout_start, blackout_end) {
        (end - start).max(0.0)
    } else {
        0.0
    };

    let files = OutputFiles {
        output_dir: output_dir.clone(),
        csv_path: output_dir.join("starship_timeseries.csv"),
        summary_path: output_dir.join("starship_summary.json"),
        plot_altitude_path: output_dir.join("plot_altitude.png"),
        plot_error_path: output_dir.join("plot_position_error_log.png"),
        plot_trust_path: output_dir.join("plot_dsfb_trust.png"),
    };

    let inertial_metrics = compute_metrics(
        &records,
        |r| r.inertial_pos_err_m,
        |r| r.inertial_vel_err_mps,
        |r| r.inertial_att_err_deg,
    );
    let ekf_metrics = compute_metrics(
        &records,
        |r| r.ekf_pos_err_m,
        |r| r.ekf_vel_err_mps,
        |r| r.ekf_att_err_deg,
    );
    let dsfb_metrics = compute_metrics(
        &records,
        |r| r.dsfb_pos_err_m,
        |r| r.dsfb_vel_err_mps,
        |r| r.dsfb_att_err_deg,
    );

    let summary = Summary {
        config: cfg.clone(),
        samples: records.len(),
        blackout_start_s: blackout_start,
        blackout_end_s: blackout_end,
        blackout_duration_s,
        inertial: inertial_metrics,
        ekf: ekf_metrics,
        dsfb: dsfb_metrics,
        outputs: files.clone(),
    };

    write_csv(&files.csv_path, &records)?;
    write_summary(&files.summary_path, &summary)?;
    make_plots(&records, &files)?;

    Ok(summary)
}

fn compute_metrics(
    records: &[SimRecord],
    pos_fn: impl Fn(&SimRecord) -> f64,
    vel_fn: impl Fn(&SimRecord) -> f64,
    att_fn: impl Fn(&SimRecord) -> f64,
) -> MethodMetrics {
    let mut pos_sq = 0.0;
    let mut vel_sq = 0.0;
    let mut att_sq = 0.0;
    let mut max_pos = 0.0_f64;
    let mut count = 0.0_f64;

    for r in records {
        let p = pos_fn(r);
        let v = vel_fn(r);
        let a = att_fn(r);
        if !(p.is_finite() && v.is_finite() && a.is_finite()) {
            continue;
        }
        pos_sq += p * p;
        vel_sq += v * v;
        att_sq += a * a;
        max_pos = max_pos.max(p);
        count += 1.0;
    }

    let final_pos = records
        .iter()
        .rev()
        .find_map(|r| {
            let p = pos_fn(r);
            if p.is_finite() {
                Some(p)
            } else {
                None
            }
        })
        .unwrap_or(0.0);
    let n = count.max(1.0);

    MethodMetrics {
        rmse_position_m: (pos_sq / n).sqrt(),
        rmse_velocity_mps: (vel_sq / n).sqrt(),
        rmse_attitude_deg: (att_sq / n).sqrt(),
        final_position_error_m: final_pos,
        max_position_error_m: max_pos,
    }
}

fn gaussian(rng: &mut ChaCha8Rng, sigma: f64) -> f64 {
    let z: f64 = rng.sample(StandardNormal);
    sigma * z
}

fn finite_nav(pos: &Vector3<f64>, vel: &Vector3<f64>) -> bool {
    pos.iter().all(|v| v.is_finite()) && vel.iter().all(|v| v.is_finite())
}

pub fn workspace_root_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../.."))
}

pub fn default_output_base_dir() -> PathBuf {
    workspace_root_dir().join("output-dsfb-starship")
}

fn resolve_output_base_dir(requested: &Path) -> PathBuf {
    if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        workspace_root_dir().join(requested)
    }
}

fn create_timestamped_run_dir(base_dir: &Path) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(base_dir)
        .with_context(|| format!("failed to create output base directory {}", base_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let run_dir = base_dir.join(&timestamp);
    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
        return Ok(run_dir);
    }

    let mut counter: usize = 1;
    loop {
        let candidate = base_dir.join(format!("{timestamp}-{counter:02}"));
        if !candidate.exists() {
            fs::create_dir_all(&candidate)?;
            return Ok(candidate);
        }
        counter += 1;
    }
}

#[pyfunction]
#[pyo3(signature = (output_dir=None, dt=None, t_final=None, rho=None, slew_threshold=None, seed=None))]
fn run_starship_simulation(
    output_dir: Option<String>,
    dt: Option<f64>,
    t_final: Option<f64>,
    rho: Option<f64>,
    slew_threshold: Option<f64>,
    seed: Option<u64>,
) -> PyResult<String> {
    let mut cfg = SimConfig::default();

    if let Some(v) = dt {
        cfg.dt = v;
    }
    if let Some(v) = t_final {
        cfg.t_final = v;
    }
    if let Some(v) = rho {
        cfg.rho = v;
    }
    if let Some(v) = slew_threshold {
        cfg.slew_threshold_accel = v;
        cfg.slew_threshold_gyro = (v * 0.055).max(0.15);
    }
    if let Some(v) = seed {
        cfg.seed = v;
    }

    let out = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output-dsfb-starship"));

    let summary = run_simulation(&cfg, &out)
        .map_err(|e| PyRuntimeError::new_err(format!("simulation failed: {e:#}")))?;

    serde_json::to_string_pretty(&summary)
        .map_err(|e| PyRuntimeError::new_err(format!("summary serialization failed: {e}")))
}

#[pyfunction]
fn default_config_json() -> PyResult<String> {
    serde_json::to_string_pretty(&SimConfig::default())
        .map_err(|e| PyRuntimeError::new_err(format!("config serialization failed: {e}")))
}

#[pymodule]
fn dsfb_starship(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_starship_simulation, m)?)?;
    m.add_function(wrap_pyfunction!(default_config_json, m)?)?;
    Ok(())
}
