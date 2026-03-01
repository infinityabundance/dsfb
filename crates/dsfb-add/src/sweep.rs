use std::fs;
use std::path::{Path, PathBuf};

use dsfb::{DsfbObserver, DsfbParams, DsfbState};
use serde::{Deserialize, Serialize};

use crate::aet::{self, AetSweep};
use crate::analysis::rlt_phase::{analyze_rlt_phase_boundary, RltPhaseBoundary};
use crate::analysis::structural_law::{diagnostics_from_fit, fit_with_ci, LinearFit};
use crate::config::SimulationConfig;
use crate::iwlt::{self, IwltSweep};
use crate::output::{
    write_aet_csv, write_cross_layer_thresholds_csv, write_diagnostics_summary_csv, write_iwlt_csv,
    write_rlt_csv, write_rlt_phase_boundary_csv, write_rlt_trajectory_csv,
    write_robustness_metrics_csv, write_structural_law_summary_csv, write_tcp_csv,
    write_tcp_phase_alignment_csv, write_tcp_points_csv, CrossLayerThresholdRow,
    DiagnosticsSummaryRow, PhaseBoundaryRow, RobustnessMetricRow, StructuralLawSummaryRow,
    TcpPhaseAlignmentRow,
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
    let use_step_suffix = sweep_steps.len() > 1;
    let canonical_steps = canonical_steps(config, &sweep_steps);

    let mut runs = Vec::with_capacity(sweep_steps.len());
    let mut phase_rows = Vec::new();
    let mut law_rows = Vec::new();
    let mut scaling_rows = Vec::new();
    let mut diagnostics_rows = Vec::new();
    let mut threshold_rows = Vec::new();
    let mut tcp_alignment_rows = Vec::new();
    let mut robustness_rows = Vec::new();

    let mut canonical_aet = None;
    let mut canonical_tcp = None;
    let mut canonical_rlt = None;
    let mut canonical_iwlt = None;

    for steps_per_run in sweep_steps {
        let mut run_config = config.clone();
        run_config.steps_per_run = steps_per_run;

        let is_canonical = steps_per_run == canonical_steps;
        let suffix = if use_step_suffix {
            format!("_N{steps_per_run}")
        } else {
            String::new()
        };

        let (aet, aet_perturbed) = if config.enable_aet {
            let baseline = aet::run_aet_sweep(&run_config, &lambda_grid)?;
            let perturbed = aet::run_aet_sweep_perturbed(&run_config, &lambda_grid)?;

            write_aet_csv(
                &output_dir.join(format!("aet_sweep{suffix}.csv")),
                &lambda_grid,
                &baseline.echo_slope,
                &baseline.avg_increment,
                steps_per_run,
                false,
            )?;
            write_aet_csv(
                &output_dir.join(format!("aet_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.echo_slope,
                &perturbed.avg_increment,
                steps_per_run,
                true,
            )?;

            if use_step_suffix && is_canonical {
                write_aet_csv(
                    &output_dir.join("aet_sweep.csv"),
                    &lambda_grid,
                    &baseline.echo_slope,
                    &baseline.avg_increment,
                    steps_per_run,
                    false,
                )?;
                write_aet_csv(
                    &output_dir.join("aet_sweep_perturbed.csv"),
                    &lambda_grid,
                    &perturbed.echo_slope,
                    &perturbed.avg_increment,
                    steps_per_run,
                    true,
                )?;
            }

            robustness_rows.push(comparison_metric(
                "aet_curve_l2_diff",
                steps_per_run,
                0.0,
                curve_l2_diff(&baseline.echo_slope, &perturbed.echo_slope),
            ));
            robustness_rows.push(comparison_metric(
                "aet_curve_max_abs_diff",
                steps_per_run,
                0.0,
                curve_max_abs_diff(&baseline.echo_slope, &perturbed.echo_slope),
            ));

            if is_canonical {
                canonical_aet = Some(baseline.clone());
            }

            (Some(baseline), Some(perturbed))
        } else {
            (None, None)
        };

        let tcp = if config.enable_tcp {
            let baseline = tcp::run_tcp_sweep(&run_config, &lambda_grid)?;
            write_tcp_csv(
                &output_dir.join(format!("tcp_sweep{suffix}.csv")),
                &lambda_grid,
                &baseline.betti0,
                &baseline.betti1,
                &baseline.l_tcp,
                &baseline.avg_radius,
                &baseline.max_radius,
                &baseline.variance_radius,
                steps_per_run,
                false,
            )?;

            if use_step_suffix && is_canonical {
                write_tcp_csv(
                    &output_dir.join("tcp_sweep.csv"),
                    &lambda_grid,
                    &baseline.betti0,
                    &baseline.betti1,
                    &baseline.l_tcp,
                    &baseline.avg_radius,
                    &baseline.max_radius,
                    &baseline.variance_radius,
                    steps_per_run,
                    false,
                )?;
            }

            for points_dir in points_dirs(output_dir, steps_per_run, use_step_suffix, is_canonical)
            {
                fs::create_dir_all(&points_dir)?;
                for (idx, runs_for_lambda) in baseline.point_cloud_runs.iter().enumerate() {
                    for (run_idx, points) in runs_for_lambda.iter().enumerate() {
                        let filename = format!("lambda_{idx:03}_run_{run_idx:02}.csv");
                        write_tcp_points_csv(&points_dir.join(filename), points)?;
                    }
                }
            }

            if is_canonical {
                canonical_tcp = Some(baseline.clone());
            }

            Some(baseline)
        } else {
            None
        };

        let (rlt, rlt_perturbed, baseline_phase, perturbed_phase) = if config.enable_rlt {
            let baseline = rlt::run_rlt_sweep(&run_config, &lambda_grid)?;
            let perturbed = rlt::run_rlt_sweep_perturbed(&run_config, &lambda_grid)?;
            let baseline_phase = analyze_rlt_phase_boundary(
                &lambda_grid,
                &baseline.expansion_ratio,
                &baseline.escape_rate,
            )?;
            let perturbed_phase = analyze_rlt_phase_boundary(
                &lambda_grid,
                &perturbed.expansion_ratio,
                &perturbed.escape_rate,
            )?;

            write_rlt_csv(
                &output_dir.join(format!("rlt_sweep{suffix}.csv")),
                &lambda_grid,
                &baseline.escape_rate,
                &baseline.expansion_ratio,
                steps_per_run,
                false,
            )?;
            write_rlt_csv(
                &output_dir.join(format!("rlt_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.escape_rate,
                &perturbed.expansion_ratio,
                steps_per_run,
                true,
            )?;

            if use_step_suffix && is_canonical {
                write_rlt_csv(
                    &output_dir.join("rlt_sweep.csv"),
                    &lambda_grid,
                    &baseline.escape_rate,
                    &baseline.expansion_ratio,
                    steps_per_run,
                    false,
                )?;
                write_rlt_csv(
                    &output_dir.join("rlt_sweep_perturbed.csv"),
                    &lambda_grid,
                    &perturbed.escape_rate,
                    &perturbed.expansion_ratio,
                    steps_per_run,
                    true,
                )?;
            }

            phase_rows.push(phase_row("baseline", false, steps_per_run, baseline_phase));
            phase_rows.push(phase_row("perturbed", true, steps_per_run, perturbed_phase));

            robustness_rows.push(comparison_metric(
                "rlt_curve_l2_diff",
                steps_per_run,
                0.0,
                curve_l2_diff(&baseline.expansion_ratio, &perturbed.expansion_ratio),
            ));
            robustness_rows.push(comparison_metric(
                "rlt_curve_max_abs_diff",
                steps_per_run,
                0.0,
                curve_max_abs_diff(&baseline.expansion_ratio, &perturbed.expansion_ratio),
            ));
            robustness_rows.push(comparison_metric_option(
                "lambda_star",
                steps_per_run,
                baseline_phase.lambda_star,
                perturbed_phase.lambda_star,
            ));
            robustness_rows.push(comparison_metric_option(
                "transition_width",
                steps_per_run,
                baseline_phase.transition_width,
                perturbed_phase.transition_width,
            ));
            robustness_rows.push(comparison_metric_option(
                "max_derivative",
                steps_per_run,
                baseline_phase.max_derivative,
                perturbed_phase.max_derivative,
            ));

            for examples_dir in
                example_dirs(output_dir, steps_per_run, use_step_suffix, is_canonical)
            {
                fs::create_dir_all(&examples_dir)?;
                let (bounded_idx, expanding_idx) =
                    rlt::find_representative_regime_indices(&baseline.escape_rate);
                for (kind, idx) in [
                    (RltExampleKind::Bounded, bounded_idx),
                    (RltExampleKind::Expanding, expanding_idx),
                ] {
                    let lambda = lambda_grid[idx];
                    let trajectory = rlt::simulate_example_trajectory(
                        &run_config,
                        lambda,
                        rlt::RLT_EXAMPLE_STEPS,
                    );
                    let filename =
                        format!("trajectory_{}_lambda_{idx:03}.csv", kind.filename_prefix());
                    write_rlt_trajectory_csv(&examples_dir.join(filename), &trajectory)?;
                }
            }

            if is_canonical {
                canonical_rlt = Some(baseline.clone());
            }

            (
                Some(baseline),
                Some(perturbed),
                Some(baseline_phase),
                Some(perturbed_phase),
            )
        } else {
            (None, None, None, None)
        };

        let (iwlt, iwlt_perturbed) = if config.enable_iwlt {
            let baseline = iwlt::run_iwlt_sweep(&run_config, &lambda_grid)?;
            let perturbed = iwlt::run_iwlt_sweep_perturbed(&run_config, &lambda_grid)?;

            write_iwlt_csv(
                &output_dir.join(format!("iwlt_sweep{suffix}.csv")),
                &lambda_grid,
                &baseline.entropy_density,
                &baseline.avg_increment,
                steps_per_run,
                false,
            )?;
            write_iwlt_csv(
                &output_dir.join(format!("iwlt_sweep_perturbed{suffix}.csv")),
                &lambda_grid,
                &perturbed.entropy_density,
                &perturbed.avg_increment,
                steps_per_run,
                true,
            )?;

            if use_step_suffix && is_canonical {
                write_iwlt_csv(
                    &output_dir.join("iwlt_sweep.csv"),
                    &lambda_grid,
                    &baseline.entropy_density,
                    &baseline.avg_increment,
                    steps_per_run,
                    false,
                )?;
                write_iwlt_csv(
                    &output_dir.join("iwlt_sweep_perturbed.csv"),
                    &lambda_grid,
                    &perturbed.entropy_density,
                    &perturbed.avg_increment,
                    steps_per_run,
                    true,
                )?;
            }

            robustness_rows.push(comparison_metric(
                "iwlt_curve_l2_diff",
                steps_per_run,
                0.0,
                curve_l2_diff(&baseline.entropy_density, &perturbed.entropy_density),
            ));
            robustness_rows.push(comparison_metric(
                "iwlt_curve_max_abs_diff",
                steps_per_run,
                0.0,
                curve_max_abs_diff(&baseline.entropy_density, &perturbed.entropy_density),
            ));

            if is_canonical {
                canonical_iwlt = Some(baseline.clone());
            }

            (Some(baseline), Some(perturbed))
        } else {
            (None, None)
        };

        if let (Some(aet_baseline), Some(iwlt_baseline)) = (&aet, &iwlt) {
            let baseline_fit =
                fit_with_ci(&aet_baseline.echo_slope, &iwlt_baseline.entropy_density)?;
            let baseline_diag = diagnostics_from_fit(
                &aet_baseline.echo_slope,
                &iwlt_baseline.entropy_density,
                &baseline_fit,
            )?;
            let baseline_row = law_summary_row(steps_per_run, false, baseline_fit, baseline_diag);
            law_rows.push(baseline_row.clone());
            scaling_rows.push(baseline_row);
            diagnostics_rows.push(DiagnosticsSummaryRow {
                steps_per_run,
                residual_mean: baseline_diag.residual_mean,
                residual_std: baseline_diag.residual_std,
                residual_skew_approx: baseline_diag.residual_skew_approx,
                residual_kurtosis_approx: baseline_diag.residual_kurtosis_approx,
                ratio_mean: baseline_diag.ratio_mean,
                ratio_std: baseline_diag.ratio_std,
                ratio_min: baseline_diag.ratio_min,
                ratio_max: baseline_diag.ratio_max,
            });

            if let Some(phase) = baseline_phase {
                if let Some(phase_index) = closest_lambda_index(&lambda_grid, phase.lambda_star) {
                    threshold_rows.push(CrossLayerThresholdRow {
                        steps_per_run,
                        lambda_star: phase.lambda_star,
                        echo_slope_star: Some(aet_baseline.echo_slope[phase_index]),
                        entropy_density_star: Some(iwlt_baseline.entropy_density[phase_index]),
                    });
                }
            }

            if let (Some(aet_perturbed_sweep), Some(iwlt_perturbed_sweep)) =
                (&aet_perturbed, &iwlt_perturbed)
            {
                let perturbed_fit = fit_with_ci(
                    &aet_perturbed_sweep.echo_slope,
                    &iwlt_perturbed_sweep.entropy_density,
                )?;
                let perturbed_diag = diagnostics_from_fit(
                    &aet_perturbed_sweep.echo_slope,
                    &iwlt_perturbed_sweep.entropy_density,
                    &perturbed_fit,
                )?;
                law_rows.push(law_summary_row(
                    steps_per_run,
                    true,
                    perturbed_fit,
                    perturbed_diag,
                ));

                robustness_rows.push(comparison_metric(
                    "structural_law_slope",
                    steps_per_run,
                    baseline_fit.slope,
                    perturbed_fit.slope,
                ));
                robustness_rows.push(comparison_metric(
                    "structural_law_intercept",
                    steps_per_run,
                    baseline_fit.intercept,
                    perturbed_fit.intercept,
                ));
                robustness_rows.push(comparison_metric(
                    "structural_law_r2",
                    steps_per_run,
                    baseline_fit.r2,
                    perturbed_fit.r2,
                ));
                robustness_rows.push(comparison_metric(
                    "structural_law_residual_variance",
                    steps_per_run,
                    baseline_fit.residual_variance,
                    perturbed_fit.residual_variance,
                ));
            }
        }

        if let (Some(tcp_baseline), Some(phase)) = (&tcp, baseline_phase) {
            tcp_alignment_rows.push(tcp_phase_alignment_row(
                steps_per_run,
                phase.lambda_star,
                peak_lambda(&lambda_grid, &tcp_baseline.l_tcp),
                peak_lambda_usize(&lambda_grid, &tcp_baseline.betti1),
            ));
        }

        let _ = rlt_perturbed;
        let _ = perturbed_phase;

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
    if !law_rows.is_empty() {
        write_structural_law_summary_csv(&output_dir.join("aet_iwlt_law_summary.csv"), &law_rows)?;
    }
    if !scaling_rows.is_empty() {
        write_structural_law_summary_csv(
            &output_dir.join("aet_iwlt_scaling_summary.csv"),
            &scaling_rows,
        )?;
    }
    if !diagnostics_rows.is_empty() {
        write_diagnostics_summary_csv(
            &output_dir.join("aet_iwlt_diagnostics_summary.csv"),
            &diagnostics_rows,
        )?;
    }
    if !threshold_rows.is_empty() {
        write_cross_layer_thresholds_csv(
            &output_dir.join("cross_layer_thresholds.csv"),
            &threshold_rows,
        )?;
    }
    if !tcp_alignment_rows.is_empty() {
        write_tcp_phase_alignment_csv(
            &output_dir.join("tcp_phase_alignment.csv"),
            &tcp_alignment_rows,
        )?;
    }
    if !robustness_rows.is_empty() {
        write_robustness_metrics_csv(&output_dir.join("robustness_metrics.csv"), &robustness_rows)?;
    }

    Ok(SweepResult {
        output_dir: output_dir.to_path_buf(),
        lambda_grid,
        runs,
        aet: canonical_aet,
        tcp: canonical_tcp,
        rlt: canonical_rlt,
        iwlt: canonical_iwlt,
    })
}

fn canonical_steps(config: &SimulationConfig, sweep_steps: &[usize]) -> usize {
    if sweep_steps.contains(&config.steps_per_run) {
        config.steps_per_run
    } else {
        sweep_steps[0]
    }
}

fn points_dirs(
    output_dir: &Path,
    steps_per_run: usize,
    use_step_suffix: bool,
    is_canonical: bool,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if use_step_suffix {
        dirs.push(output_dir.join(format!("tcp_points_N{steps_per_run}")));
        if is_canonical {
            dirs.push(output_dir.join("tcp_points"));
        }
    } else {
        dirs.push(output_dir.join("tcp_points"));
    }
    dirs
}

fn example_dirs(
    output_dir: &Path,
    steps_per_run: usize,
    use_step_suffix: bool,
    is_canonical: bool,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if use_step_suffix {
        dirs.push(output_dir.join(format!("rlt_examples_N{steps_per_run}")));
        if is_canonical {
            dirs.push(output_dir.join("rlt_examples"));
        }
    } else {
        dirs.push(output_dir.join("rlt_examples"));
    }
    dirs
}

fn phase_row(
    mode: &str,
    is_perturbed: bool,
    steps_per_run: usize,
    summary: RltPhaseBoundary,
) -> PhaseBoundaryRow {
    PhaseBoundaryRow {
        steps_per_run,
        mode: mode.to_string(),
        is_perturbed,
        lambda_star: summary.lambda_star,
        lambda_0_1: summary.lambda_0_1,
        lambda_0_9: summary.lambda_0_9,
        transition_width: summary.transition_width,
        max_derivative: summary.max_derivative,
    }
}

fn law_summary_row(
    steps_per_run: usize,
    is_perturbed: bool,
    fit: LinearFit,
    diagnostics: crate::analysis::structural_law::StructuralLawDiagnostics,
) -> StructuralLawSummaryRow {
    StructuralLawSummaryRow {
        steps_per_run,
        is_perturbed,
        pearson_r: fit.pearson_r,
        spearman_rho: fit.spearman_rho,
        slope: fit.slope,
        intercept: fit.intercept,
        r2: fit.r2,
        residual_variance: fit.residual_variance,
        mse_resid: fit.mse_resid,
        slope_ci_low: fit.slope_ci_low,
        slope_ci_high: fit.slope_ci_high,
        sample_count: fit.sample_count,
        ratio_mean: diagnostics.ratio_mean,
        ratio_std: diagnostics.ratio_std,
    }
}

fn closest_lambda_index(lambda_grid: &[f64], target: Option<f64>) -> Option<usize> {
    let target = target?;
    lambda_grid
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_delta = (*left - target).abs();
            let right_delta = (*right - target).abs();
            left_delta
                .partial_cmp(&right_delta)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
}

fn peak_lambda(lambda_grid: &[f64], values: &[f64]) -> Option<f64> {
    lambda_grid
        .iter()
        .zip(values.iter())
        .max_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(lambda, _)| *lambda)
}

fn peak_lambda_usize(lambda_grid: &[f64], values: &[usize]) -> Option<f64> {
    lambda_grid
        .iter()
        .zip(values.iter())
        .max_by_key(|(_, value)| **value)
        .map(|(lambda, _)| *lambda)
}

fn tcp_phase_alignment_row(
    steps_per_run: usize,
    lambda_star: Option<f64>,
    lambda_tp_peak: Option<f64>,
    lambda_b1_peak: Option<f64>,
) -> TcpPhaseAlignmentRow {
    TcpPhaseAlignmentRow {
        steps_per_run,
        lambda_star,
        lambda_tp_peak,
        lambda_b1_peak,
        delta_tp: option_diff(lambda_star, lambda_tp_peak),
        delta_b1: option_diff(lambda_star, lambda_b1_peak),
    }
}

fn comparison_metric(
    metric: &str,
    steps_per_run: usize,
    baseline: f64,
    perturbed: f64,
) -> RobustnessMetricRow {
    RobustnessMetricRow {
        metric: metric.to_string(),
        steps_per_run,
        baseline,
        perturbed,
        delta: perturbed - baseline,
    }
}

fn comparison_metric_option(
    metric: &str,
    steps_per_run: usize,
    baseline: Option<f64>,
    perturbed: Option<f64>,
) -> RobustnessMetricRow {
    comparison_metric(
        metric,
        steps_per_run,
        baseline.unwrap_or(f64::NAN),
        perturbed.unwrap_or(f64::NAN),
    )
}

fn option_diff(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left - right),
        _ => None,
    }
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
