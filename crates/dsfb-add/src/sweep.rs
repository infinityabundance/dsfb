use std::fs;
use std::path::{Path, PathBuf};

use dsfb::{DsfbObserver, DsfbParams, DsfbState};
use serde::{Deserialize, Serialize};

use crate::aet::{self, AetSweep};
use crate::analysis::rlt_phase::analyze_rlt_phase_boundary;
use crate::config::SimulationConfig;
use crate::iwlt::{self, IwltSweep};
use crate::output::{
    write_aet_csv, write_iwlt_csv, write_rlt_csv, write_rlt_phase_boundary_csv,
    write_rlt_trajectory_csv, write_robustness_metrics_csv, write_tcp_csv, write_tcp_points_csv,
    PhaseBoundaryRow, RobustnessMetricRow,
};
use crate::rlt::{self, RltExampleKind, RltSweep};
use crate::tcp::{self, TcpSweep};
use crate::AddError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepRunResult {
    pub steps_per_run: usize,
    pub aet: Option<AetSweep>,
    pub tcp: Option<TcpSweep>,
    pub rlt: Option<RltSweep>,
    pub iwlt: Option<IwltSweep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepResult {
    pub output_dir: PathBuf,
    pub lambda_grid: Vec<f64>,
    pub runs: Vec<SweepRunResult>,
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
    let sweep_steps = config.sweep_steps();
    let use_step_suffix = !config.multi_steps_per_run.is_empty();

    let mut runs = Vec::with_capacity(sweep_steps.len());
    let mut phase_rows = Vec::new();
    let mut robustness_rows = Vec::new();

    let mut last_aet = None;
    let mut last_tcp = None;
    let mut last_rlt = None;
    let mut last_iwlt = None;

    for steps_per_run in sweep_steps {
        let mut run_config = config.clone();
        run_config.steps_per_run = steps_per_run;

        let suffix = if use_step_suffix {
            format!("_N{steps_per_run}")
        } else {
            String::new()
        };

        let aet = if config.enable_aet {
            let sweep = aet::run_aet_sweep(&run_config, &lambda_grid)?;
            write_aet_csv(
                &output_dir.join(format!("aet_sweep{suffix}.csv")),
                &lambda_grid,
                &sweep.echo_slope,
                &sweep.avg_increment,
                steps_per_run,
                false,
            )?;

            let perturbed = aet::run_aet_sweep_perturbed(&run_config, &lambda_grid)?;
            write_aet_csv(
                &output_dir.join(format!("aet_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.echo_slope,
                &perturbed.avg_increment,
                steps_per_run,
                true,
            )?;

            robustness_rows.extend(curve_robustness_metrics(
                "AET",
                steps_per_run,
                &sweep.echo_slope,
                &perturbed.echo_slope,
            ));

            Some(sweep)
        } else {
            None
        };

        let tcp = if config.enable_tcp {
            let sweep = tcp::run_tcp_sweep(&run_config, &lambda_grid)?;
            write_tcp_csv(
                &output_dir.join(format!("tcp_sweep{suffix}.csv")),
                &lambda_grid,
                &sweep.betti0,
                &sweep.betti1,
                &sweep.l_tcp,
                &sweep.avg_radius,
                &sweep.max_radius,
                &sweep.variance_radius,
                steps_per_run,
                false,
            )?;

            let points_dir = if use_step_suffix {
                output_dir.join(format!("tcp_points_N{steps_per_run}"))
            } else {
                output_dir.join("tcp_points")
            };
            fs::create_dir_all(&points_dir)?;
            for (idx, runs_for_lambda) in sweep.point_cloud_runs.iter().enumerate() {
                for (run_idx, points) in runs_for_lambda.iter().enumerate() {
                    let filename = format!("lambda_{idx:03}_run_{run_idx:02}.csv");
                    write_tcp_points_csv(&points_dir.join(filename), points)?;
                }
            }

            Some(sweep)
        } else {
            None
        };

        let rlt = if config.enable_rlt {
            let sweep = rlt::run_rlt_sweep(&run_config, &lambda_grid)?;
            write_rlt_csv(
                &output_dir.join(format!("rlt_sweep{suffix}.csv")),
                &lambda_grid,
                &sweep.escape_rate,
                &sweep.expansion_ratio,
                steps_per_run,
                false,
            )?;

            let baseline_phase = analyze_rlt_phase_boundary(&lambda_grid, &sweep.expansion_ratio)?;
            phase_rows.push(PhaseBoundaryRow {
                steps_per_run,
                is_perturbed: false,
                lambda_star: baseline_phase.lambda_star,
                lambda_0_1: baseline_phase.lambda_0_1,
                lambda_0_9: baseline_phase.lambda_0_9,
                transition_width: baseline_phase.transition_width,
            });

            let perturbed = rlt::run_rlt_sweep_perturbed(&run_config, &lambda_grid)?;
            write_rlt_csv(
                &output_dir.join(format!("rlt_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.escape_rate,
                &perturbed.expansion_ratio,
                steps_per_run,
                true,
            )?;

            let perturbed_phase =
                analyze_rlt_phase_boundary(&lambda_grid, &perturbed.expansion_ratio)?;
            phase_rows.push(PhaseBoundaryRow {
                steps_per_run,
                is_perturbed: true,
                lambda_star: perturbed_phase.lambda_star,
                lambda_0_1: perturbed_phase.lambda_0_1,
                lambda_0_9: perturbed_phase.lambda_0_9,
                transition_width: perturbed_phase.transition_width,
            });

            robustness_rows.extend(curve_robustness_metrics(
                "RLT",
                steps_per_run,
                &sweep.expansion_ratio,
                &perturbed.expansion_ratio,
            ));
            robustness_rows.push(RobustnessMetricRow {
                subsystem: "RLT".to_string(),
                steps_per_run,
                metric_name: "lambda_star_shift".to_string(),
                value: match (baseline_phase.lambda_star, perturbed_phase.lambda_star) {
                    (Some(base), Some(perturbed_value)) => perturbed_value - base,
                    _ => f64::NAN,
                },
            });

            let examples_dir = if use_step_suffix {
                output_dir.join(format!("rlt_examples_N{steps_per_run}"))
            } else {
                output_dir.join("rlt_examples")
            };
            fs::create_dir_all(&examples_dir)?;
            let (bounded_idx, expanding_idx) =
                rlt::find_representative_regime_indices(&sweep.escape_rate);
            for (kind, idx) in [
                (RltExampleKind::Bounded, bounded_idx),
                (RltExampleKind::Expanding, expanding_idx),
            ] {
                let lambda = lambda_grid[idx];
                let trajectory =
                    rlt::simulate_example_trajectory(&run_config, lambda, rlt::RLT_EXAMPLE_STEPS);
                let filename = format!("trajectory_{}_lambda_{idx:03}.csv", kind.filename_prefix());
                write_rlt_trajectory_csv(&examples_dir.join(filename), &trajectory)?;
            }

            Some(sweep)
        } else {
            None
        };

        let iwlt = if config.enable_iwlt {
            let sweep = iwlt::run_iwlt_sweep(&run_config, &lambda_grid)?;
            write_iwlt_csv(
                &output_dir.join(format!("iwlt_sweep{suffix}.csv")),
                &lambda_grid,
                &sweep.entropy_density,
                &sweep.avg_increment,
                steps_per_run,
                false,
            )?;

            let perturbed = iwlt::run_iwlt_sweep_perturbed(&run_config, &lambda_grid)?;
            write_iwlt_csv(
                &output_dir.join(format!("iwlt_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.entropy_density,
                &perturbed.avg_increment,
                steps_per_run,
                true,
            )?;

            robustness_rows.extend(curve_robustness_metrics(
                "IWLT",
                steps_per_run,
                &sweep.entropy_density,
                &perturbed.entropy_density,
            ));

            Some(sweep)
        } else {
            None
        };

        last_aet = aet.clone();
        last_tcp = tcp.clone();
        last_rlt = rlt.clone();
        last_iwlt = iwlt.clone();

        runs.push(SweepRunResult {
            steps_per_run,
            aet,
            tcp,
            rlt,
            iwlt,
        });
    }

    if !phase_rows.is_empty() {
        write_rlt_phase_boundary_csv(&output_dir.join("rlt_phase_boundary.csv"), &phase_rows)?;
    }

    if !robustness_rows.is_empty() {
        write_robustness_metrics_csv(&output_dir.join("robustness_metrics.csv"), &robustness_rows)?;
    }

    Ok(SweepResult {
        output_dir: output_dir.to_path_buf(),
        lambda_grid,
        runs,
        aet: last_aet,
        tcp: last_tcp,
        rlt: last_rlt,
        iwlt: last_iwlt,
    })
}

fn curve_robustness_metrics(
    subsystem: &str,
    steps_per_run: usize,
    baseline: &[f64],
    perturbed: &[f64],
) -> Vec<RobustnessMetricRow> {
    vec![
        RobustnessMetricRow {
            subsystem: subsystem.to_string(),
            steps_per_run,
            metric_name: "l2_diff".to_string(),
            value: curve_l2_diff(baseline, perturbed),
        },
        RobustnessMetricRow {
            subsystem: subsystem.to_string(),
            steps_per_run,
            metric_name: "max_abs_diff".to_string(),
            value: curve_max_abs_diff(baseline, perturbed),
        },
    ]
}

fn curve_l2_diff(baseline: &[f64], perturbed: &[f64]) -> f64 {
    baseline
        .iter()
        .zip(perturbed.iter())
        .map(|(base, perturbed_value)| {
            let delta = perturbed_value - base;
            delta * delta
        })
        .sum::<f64>()
        .sqrt()
}

fn curve_max_abs_diff(baseline: &[f64], perturbed: &[f64]) -> f64 {
    baseline
        .iter()
        .zip(perturbed.iter())
        .map(|(base, perturbed_value)| (perturbed_value - base).abs())
        .fold(0.0_f64, f64::max)
}
