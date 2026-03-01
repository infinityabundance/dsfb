use std::fs;
use std::path::{Path, PathBuf};

use dsfb::{DsfbObserver, DsfbParams, DsfbState};
use serde::{Deserialize, Serialize};

use crate::aet::{self, AetSweep};
use crate::config::SimulationConfig;
use crate::iwlt::{self, IwltSweep};
use crate::output::{
    write_aet_csv, write_iwlt_csv, write_rlt_csv, write_rlt_trajectory_csv, write_tcp_csv,
    write_tcp_points_csv,
};
use crate::rlt::{self, RltExampleKind, RltSweep};
use crate::tcp::{self, TcpSweep};
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepResult {
    pub output_dir: PathBuf,
    pub lambda_grid: Vec<f64>,
    pub aet: Option<AetSweep>,
    pub tcp: Option<TcpSweep>,
    pub rlt: Option<RltSweep>,
    pub iwlt: Option<IwltSweep>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DriveSignal {
    pub phase_bias: f64,
    pub trust_bias: f64,
    pub drift_bias: f64,
}

pub(crate) fn deterministic_drive(seed: u64, lambda: f64, salt: u64) -> DriveSignal {
    let mut observer = DsfbObserver::new(DsfbParams::new(0.35, 0.08, 0.01, 0.92, 0.15), 2);
    observer.init(DsfbState::new(lambda * 0.25, 0.0, 0.0));

    let phase = lambda * std::f64::consts::TAU + (seed ^ salt) as f64 * 1.0e-6;
    let dt = 0.125;

    for step in 0..24 {
        let t = step as f64 * dt;
        let quantized0 =
            (((seed.wrapping_add(salt).wrapping_add(step as u64)) % 11) as f64 - 5.0) * 0.01;
        let quantized1 =
            (((seed ^ salt).wrapping_add((step * 3) as u64) % 13) as f64 - 6.0) * 0.008;

        let channel0 = lambda + 0.32 * (phase + 1.7 * t).sin() + quantized0;
        let channel1 = lambda + 0.27 * (phase * 0.8 + 2.3 * t).cos() + quantized1;

        observer.step(&[channel0, channel1], dt);
    }

    let state = observer.state();
    DriveSignal {
        phase_bias: state.phi.tanh(),
        trust_bias: observer.trust_weight(0) - observer.trust_weight(1),
        drift_bias: state.omega.tanh(),
    }
}

pub fn run_sweeps_into_dir(
    config: &SimulationConfig,
    output_dir: &Path,
) -> Result<SweepResult, AddError> {
    config.validate()?;
    fs::create_dir_all(output_dir)?;

    let lambda_grid = config.lambda_grid();

    let aet = if config.enable_aet {
        let sweep = aet::run_aet_sweep(config, &lambda_grid)?;
        write_aet_csv(
            &output_dir.join("aet_sweep.csv"),
            &lambda_grid,
            &sweep.echo_slope,
            &sweep.avg_increment,
        )?;
        Some(sweep)
    } else {
        None
    };

    let tcp = if config.enable_tcp {
        let sweep = tcp::run_tcp_sweep(config, &lambda_grid)?;
        write_tcp_csv(
            &output_dir.join("tcp_sweep.csv"),
            &lambda_grid,
            &sweep.betti0,
            &sweep.betti1,
            &sweep.l_tcp,
            &sweep.avg_radius,
            &sweep.max_radius,
            &sweep.variance_radius,
        )?;

        let points_dir = output_dir.join("tcp_points");
        fs::create_dir_all(&points_dir)?;
        for (idx, runs) in sweep.point_cloud_runs.iter().enumerate() {
            for (run_idx, points) in runs.iter().enumerate() {
                let filename = format!("lambda_{idx:03}_run_{run_idx:02}.csv");
                write_tcp_points_csv(&points_dir.join(filename), points)?;
            }
        }

        Some(sweep)
    } else {
        None
    };

    let rlt = if config.enable_rlt {
        let sweep = rlt::run_rlt_sweep(config, &lambda_grid)?;
        write_rlt_csv(
            &output_dir.join("rlt_sweep.csv"),
            &lambda_grid,
            &sweep.escape_rate,
            &sweep.expansion_ratio,
        )?;

        let examples_dir = output_dir.join("rlt_examples");
        fs::create_dir_all(&examples_dir)?;
        let (bounded_idx, expanding_idx) =
            rlt::find_representative_regime_indices(&sweep.escape_rate);
        for (kind, idx) in [
            (RltExampleKind::Bounded, bounded_idx),
            (RltExampleKind::Expanding, expanding_idx),
        ] {
            let lambda = lambda_grid[idx];
            let trajectory =
                rlt::simulate_example_trajectory(config, lambda, rlt::RLT_EXAMPLE_STEPS);
            let filename = format!("trajectory_{}_lambda_{idx:03}.csv", kind.filename_prefix());
            write_rlt_trajectory_csv(&examples_dir.join(filename), &trajectory)?;
        }

        Some(sweep)
    } else {
        None
    };

    let iwlt = if config.enable_iwlt {
        let sweep = iwlt::run_iwlt_sweep(config, &lambda_grid)?;
        write_iwlt_csv(
            &output_dir.join("iwlt_sweep.csv"),
            &lambda_grid,
            &sweep.entropy_density,
            &sweep.avg_increment,
        )?;
        Some(sweep)
    } else {
        None
    };

    Ok(SweepResult {
        output_dir: output_dir.to_path_buf(),
        lambda_grid,
        aet,
        tcp,
        rlt,
        iwlt,
    })
}
