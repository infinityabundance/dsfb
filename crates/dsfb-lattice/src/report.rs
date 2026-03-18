use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use plotters::prelude::*;

use crate::detectability::{DetectabilitySummary, Envelope};
use crate::spectra::SpectrumAnalysis;
use crate::utils::{escape_pdf_text, max_abs, min_max, time_points, wrap_text};
use crate::{ExperimentResult, RunSummary, SofteningSweepResult};

pub struct ReportArtifacts {
    pub markdown_path: PathBuf,
    pub pdf_path: PathBuf,
    pub figure_paths: Vec<PathBuf>,
}

pub fn write_reports(
    run_dir: &Path,
    nominal: &SpectrumAnalysis,
    point_defect: Option<&ExperimentResult>,
    strain: Option<&ExperimentResult>,
    group_mode: Option<&ExperimentResult>,
    envelope: &Envelope,
    baseline_reference_norms: &[f64],
    detectability: Option<&DetectabilitySummary>,
    softening: Option<&SofteningSweepResult>,
    summary: &RunSummary,
    dt: f64,
) -> Result<ReportArtifacts> {
    let mut figure_paths = Vec::new();

    if let Some(point_defect) = point_defect {
        let spectrum_path = run_dir.join("figure_01_nominal_vs_point_spectrum.png");
        plot_nominal_vs_point_spectrum(&spectrum_path, nominal, point_defect)?;
        figure_paths.push(spectrum_path);

        let residual_path = run_dir.join("figure_03_residual_timeseries_point_defect.png");
        plot_point_defect_residuals(&residual_path, point_defect, dt)?;
        figure_paths.push(residual_path);

        let drift_slew_path = run_dir.join("figure_04_drift_slew_timeseries_point_defect.png");
        plot_drift_and_slew(&drift_slew_path, point_defect, dt)?;
        figure_paths.push(drift_slew_path);

        if let Some(detectability) = detectability {
            let detectability_path = run_dir.join("figure_05_detectability_envelope.png");
            plot_detectability(
                &detectability_path,
                envelope,
                baseline_reference_norms,
                &point_defect.simulation.residual_norms,
                detectability,
                dt,
            )?;
            figure_paths.push(detectability_path);
        }
    }

    if let (Some(point_defect), Some(strain), Some(group_mode)) = (point_defect, strain, group_mode) {
        let spectral_shift_path = run_dir.join("figure_02_spectral_shift_comparison.png");
        plot_spectral_shifts(&spectral_shift_path, point_defect, strain, group_mode)?;
        figure_paths.push(spectral_shift_path);

        let covariance_path = run_dir.join("figure_06_covariance_heatmap.png");
        plot_covariance_heatmaps(&covariance_path, point_defect, group_mode)?;
        figure_paths.push(covariance_path);
    }

    if let Some(softening) = softening {
        let softening_path = run_dir.join("figure_07_softening_precursor.png");
        plot_softening_precursor(&softening_path, softening)?;
        figure_paths.push(softening_path);
    }

    let markdown_path = run_dir.join("report.md");
    let markdown = render_markdown(summary);
    fs::write(&markdown_path, markdown)
        .with_context(|| format!("failed to write {}", markdown_path.display()))?;

    let pdf_path = run_dir.join("report.pdf");
    write_text_pdf(&pdf_path, summary)?;

    Ok(ReportArtifacts {
        markdown_path,
        pdf_path,
        figure_paths,
    })
}

fn plot_nominal_vs_point_spectrum(
    path: &Path,
    nominal: &SpectrumAnalysis,
    point_defect: &ExperimentResult,
) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_values: Vec<f64> = (1..=nominal.eigenvalues.len()).map(|index| index as f64).collect();
    let mut y_values = nominal.eigenvalues.clone();
    y_values.extend(point_defect.spectrum.eigenvalues.iter().copied());
    let (_, y_max) = min_max(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Nominal vs Point-Defect Eigenvalue Spectrum",
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(1.0_f64..x_values.len() as f64, 0.0_f64..(y_max * 1.08))?;

    chart
        .configure_mesh()
        .x_desc("Mode index")
        .y_desc("Eigenvalue")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            x_values
                .iter()
                .copied()
                .zip(nominal.eigenvalues.iter().copied()),
            &BLUE,
        ))?
        .label("nominal")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.filled()));

    chart
        .draw_series(LineSeries::new(
            x_values
                .iter()
                .copied()
                .zip(point_defect.spectrum.eigenvalues.iter().copied()),
            &RED,
        ))?
        .label("point defect")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.filled()));

    chart.draw_series(
        x_values
            .iter()
            .copied()
            .zip(nominal.eigenvalues.iter().copied())
            .map(|(x, y)| Circle::new((x, y), 4, BLUE.filled())),
    )?;
    chart.draw_series(
        x_values
            .iter()
            .copied()
            .zip(point_defect.spectrum.eigenvalues.iter().copied())
            .map(|(x, y)| TriangleMarker::new((x, y), 6, RED.filled())),
    )?;

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.85))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

fn plot_spectral_shifts(
    path: &Path,
    point_defect: &ExperimentResult,
    strain: &ExperimentResult,
    group_mode: &ExperimentResult,
) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mode_count = point_defect.comparison.per_mode_abs_shift.len();
    let x_values: Vec<f64> = (1..=mode_count).map(|index| index as f64).collect();
    let mut y_values = point_defect.comparison.per_mode_abs_shift.clone();
    y_values.extend(strain.comparison.per_mode_abs_shift.iter().copied());
    y_values.extend(group_mode.comparison.per_mode_abs_shift.iter().copied());
    y_values.push(point_defect.comparison.delta_norm_2);
    y_values.push(strain.comparison.delta_norm_2);
    y_values.push(group_mode.comparison.delta_norm_2);
    let (_, y_max) = min_max(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption("Spectral Shifts and Empirical Weyl Bound", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(1.0_f64..mode_count as f64, 0.0_f64..(y_max * 1.12))?;

    chart
        .configure_mesh()
        .x_desc("Mode index")
        .y_desc("Absolute eigenvalue shift")
        .draw()?;

    let series = [
        (&point_defect.comparison.per_mode_abs_shift, point_defect.comparison.delta_norm_2, &RED, "point defect"),
        (&strain.comparison.per_mode_abs_shift, strain.comparison.delta_norm_2, &BLUE, "distributed strain"),
        (&group_mode.comparison.per_mode_abs_shift, group_mode.comparison.delta_norm_2, &GREEN, "group mode"),
    ];

    for (shifts, bound, color, label) in series {
        chart
            .draw_series(LineSeries::new(
                x_values.iter().copied().zip(shifts.iter().copied()),
                color,
            ))?
            .label(format!("{label} shift"))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 16, y)], color.filled()));

        chart
            .draw_series(LineSeries::new(
                vec![(1.0, bound), (mode_count as f64, bound)],
                &color.mix(0.35),
            ))?
            .label(format!("{label} ||Delta||_2"))
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 16, y)], color.mix(0.35).filled())
            });
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.88))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn plot_point_defect_residuals(path: &Path, point_defect: &ExperimentResult, dt: f64) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let times = time_points(point_defect.simulation.residuals.len(), dt);
    let channels = point_defect
        .simulation
        .residuals
        .first()
        .map(|vector| vector.len().min(3))
        .unwrap_or(0);
    let mut channel_values = Vec::new();
    for residual in &point_defect.simulation.residuals {
        for channel in 0..channels {
            channel_values.push(residual[channel]);
        }
    }
    let bound = max_abs(&channel_values).max(0.02);

    let mut chart = ChartBuilder::on(&root)
        .caption("Point-Defect Modal Residual Channels", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0.0_f64..times.last().copied().unwrap_or(1.0),
            -bound * 1.1..bound * 1.1,
        )?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .y_desc("Residual amplitude")
        .draw()?;

    let colors = [&RED, &BLUE, &GREEN];
    for channel in 0..channels {
        chart
            .draw_series(LineSeries::new(
                times
                    .iter()
                    .copied()
                    .zip(point_defect.simulation.residuals.iter().map(|residual| residual[channel])),
                colors[channel],
            ))?
            .label(format!("mode {}", channel + 1))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 16, y)], colors[channel]));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.88))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn plot_drift_and_slew(path: &Path, point_defect: &ExperimentResult, dt: f64) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let times = time_points(point_defect.simulation.residual_norms.len(), dt);
    let mut y_values = point_defect.simulation.residual_norms.clone();
    y_values.extend(point_defect.simulation.drift_norms.iter().copied());
    y_values.extend(point_defect.simulation.slew_norms.iter().copied());
    let (_, y_max) = min_max(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption("Point-Defect Residual, Drift, and Slew Norms", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0_f64..times.last().copied().unwrap_or(1.0), 0.0_f64..(y_max * 1.1))?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .y_desc("Norm")
        .draw()?;

    let norm_series = [
        (&point_defect.simulation.residual_norms, &BLACK, "residual norm"),
        (&point_defect.simulation.drift_norms, &BLUE, "drift norm"),
        (&point_defect.simulation.slew_norms, &RED, "slew norm"),
    ];
    for (values, color, label) in norm_series {
        chart
            .draw_series(LineSeries::new(
                times.iter().copied().zip(values.iter().copied()),
                color,
            ))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 16, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.88))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn plot_detectability(
    path: &Path,
    envelope: &Envelope,
    baseline_reference_norms: &[f64],
    point_defect_norms: &[f64],
    detectability: &DetectabilitySummary,
    dt: f64,
) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let times = time_points(point_defect_norms.len(), dt);
    let mut y_values = envelope.upper.clone();
    y_values.extend(baseline_reference_norms.iter().copied());
    y_values.extend(point_defect_norms.iter().copied());
    let (_, y_max) = min_max(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption("Envelope-Based Detectability", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0_f64..times.last().copied().unwrap_or(1.0), 0.0_f64..(y_max * 1.1))?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .y_desc("Residual norm")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(envelope.upper.iter().copied()),
            &BLACK,
        ))?
        .label("envelope upper bound")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], BLACK));

    chart
        .draw_series(LineSeries::new(
            times
                .iter()
                .copied()
                .zip(baseline_reference_norms.iter().copied()),
            &BLUE,
        ))?
        .label("baseline variation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], BLUE));

    chart
        .draw_series(LineSeries::new(
            times.iter().copied().zip(point_defect_norms.iter().copied()),
            &RED,
        ))?
        .label("point defect residual")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], RED));

    if let Some(step) = detectability.first_crossing_step {
        let time = step as f64 * dt;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(time, 0.0), (time, y_max * 1.05)],
            RED.mix(0.35),
        )))?;
    }
    if let (Some(time), Some(signal_value)) = (
        detectability.first_crossing_time,
        detectability.signal_at_first_crossing,
    ) {
        chart.draw_series(std::iter::once(Circle::new(
            (time, signal_value),
            5,
            RED.filled(),
        )))?;
    }
    if let (Some(step), Some(first_crossing_step)) = (
        detectability.consecutive_crossing_step,
        detectability.first_crossing_step,
    ) {
        if step != first_crossing_step {
            let time = step as f64 * dt;
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(time, 0.0), (time, y_max * 1.05)],
                RGBColor(255, 140, 0).mix(0.35),
            )))?;
        }
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.88))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn plot_covariance_heatmaps(
    path: &Path,
    point_defect: &ExperimentResult,
    group_mode: &ExperimentResult,
) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((1, 2));

    let max_abs_covariance = point_defect
        .covariance
        .iter()
        .chain(group_mode.covariance.iter())
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max)
        .max(1.0e-6);

    draw_covariance_panel(
        &areas[0],
        &point_defect.covariance,
        "Point defect residual covariance",
        max_abs_covariance,
    )?;
    draw_covariance_panel(
        &areas[1],
        &group_mode.covariance,
        "Group-mode residual covariance",
        max_abs_covariance,
    )?;

    root.present()?;
    Ok(())
}

fn draw_covariance_panel(
    area: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>,
    covariance: &nalgebra::DMatrix<f64>,
    title: &str,
    max_abs_covariance: f64,
) -> Result<()> {
    let size = covariance.nrows().max(1) as f64;
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(45)
        .build_cartesian_2d(0.0_f64..size, 0.0_f64..size)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Channel")
        .y_desc("Channel")
        .draw()?;

    for row in 0..covariance.nrows() {
        for column in 0..covariance.ncols() {
            let value = covariance[(row, column)] / max_abs_covariance;
            let color = covariance_color(value);
            chart.draw_series(std::iter::once(Rectangle::new(
                [
                    (column as f64, row as f64),
                    (column as f64 + 1.0, row as f64 + 1.0),
                ],
                color.filled(),
            )))?;
        }
    }
    Ok(())
}

fn covariance_color(value: f64) -> RGBColor {
    let clipped = value.clamp(-1.0, 1.0);
    if clipped >= 0.0 {
        let intensity = (clipped * 200.0) as u8;
        RGBColor(40, 70 + intensity / 2, 120 + intensity / 2)
    } else {
        let intensity = (-clipped * 200.0) as u8;
        RGBColor(120 + intensity / 2, 40, 40 + intensity / 3)
    }
}

fn plot_softening_precursor(path: &Path, softening: &SofteningSweepResult) -> Result<()> {
    let root = BitMapBackend::new(path, (1280, 900)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 1));

    let x_min = softening.scales.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = softening
        .scales
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let (_, eigen_max) = min_max(&softening.smallest_eigenvalues);
    let mut upper_metrics = softening.max_residual_norms.clone();
    upper_metrics.extend(softening.max_drift_norms.iter().copied());
    upper_metrics.extend(softening.max_slew_norms.iter().copied());
    let (_, metrics_max) = min_max(&upper_metrics);

    let mut top = ChartBuilder::on(&areas[0])
        .caption("Softening Sweep: Smallest Eigenvalue", ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, 0.0_f64..(eigen_max * 1.1))?;
    top.configure_mesh()
        .x_desc("Global spring scale (lower is softer)")
        .y_desc("Smallest eigenvalue")
        .draw()?;
    top.draw_series(LineSeries::new(
        softening
            .scales
            .iter()
            .copied()
            .zip(softening.smallest_eigenvalues.iter().copied()),
        &BLUE,
    ))?;
    top.draw_series(
        softening
            .scales
            .iter()
            .copied()
            .zip(softening.smallest_eigenvalues.iter().copied())
            .map(|point| Circle::new(point, 4, BLUE.filled())),
    )?;

    let mut bottom = ChartBuilder::on(&areas[1])
        .caption("Softening Sweep: Residual / Drift / Slew Indicators", ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, 0.0_f64..(metrics_max * 1.1))?;
    bottom
        .configure_mesh()
        .x_desc("Global spring scale (lower is softer)")
        .y_desc("Maximum norm over time")
        .draw()?;

    let series = [
        (&softening.max_residual_norms, &BLACK, "max residual norm"),
        (&softening.max_drift_norms, &BLUE, "max drift norm"),
        (&softening.max_slew_norms, &RED, "max slew norm"),
    ];
    for (values, color, label) in series {
        bottom
            .draw_series(LineSeries::new(
                softening.scales.iter().copied().zip(values.iter().copied()),
                color,
            ))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 16, y)], color));
    }

    bottom
        .configure_series_labels()
        .background_style(WHITE.mix(0.88))
        .border_style(BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

fn render_markdown(summary: &RunSummary) -> String {
    let mut lines = Vec::new();
    lines.push("# DSFB Lattice Demo Report".to_string());
    lines.push(String::new());
    lines.push(format!("Run directory: `{}`", summary.run_dir));
    lines.push(format!("Selected example set: `{}`", summary.selected_example));
    lines.push(String::new());
    lines.push("## Scope".to_string());
    lines.push(
        "This report summarizes deterministic toy-model experiments for a fixed-end 1D lattice. The results illustrate bounded pieces of the paper's operator, perturbation, residual, and detectability logic. They do not establish universal structural identifiability or claim transfer to arbitrary materials."
            .to_string(),
    );
    lines.push(String::new());
    lines.push("## Nominal Reference".to_string());
    lines.push(format!(
        "- Sites: {}",
        summary.nominal_sites
    ));
    lines.push(format!(
        "- Smallest nominal eigenvalue: {:.6}",
        summary.nominal_smallest_eigenvalue
    ));
    lines.push(format!(
        "- Largest nominal eigenvalue: {:.6}",
        summary.nominal_largest_eigenvalue
    ));
    lines.push(String::new());
    lines.push("## Experiment Highlights".to_string());
    for experiment in &summary.experiments {
        lines.push(format!("### {}", experiment.name));
        lines.push(experiment.description.clone());
        lines.push(format!(
            "- delta norm 2: {:.6}",
            experiment.delta_norm_2
        ));
        lines.push(format!(
            "- max absolute eigenvalue shift: {:.6}",
            experiment.max_abs_shift
        ));
        lines.push(format!(
            "- spectral bound satisfied numerically: {}",
            experiment.bound_satisfied
        ));
        lines.push(format!(
            "- max residual norm: {:.6}",
            experiment.max_residual_norm
        ));
        lines.push(format!(
            "- max drift norm: {:.6}",
            experiment.max_drift_norm
        ));
        lines.push(format!(
            "- max slew norm: {:.6}",
            experiment.max_slew_norm
        ));
        lines.push(format!(
            "- covariance off-diagonal energy: {:.6}",
            experiment.covariance_offdiag_energy
        ));
    }
    if let Some(detectability) = &summary.detectability {
        lines.push(String::new());
        lines.push("## Detectability".to_string());
        lines.push(
            "Detectability is evaluated pointwise in time using the same-time comparison `||r(t)|| > E(t)`. Global peaks of the signal and envelope are reported separately for context; they need not occur at the same time and do not by themselves determine detection."
                .to_string(),
        );
        if let Some(step) = detectability.first_crossing_step {
            lines.push(format!("- first envelope crossing step: {step}"));
            lines.push(format!(
                "- first envelope crossing time: {:.6}",
                detectability.first_crossing_time.unwrap_or(0.0)
            ));
            lines.push(format!(
                "- signal at first crossing: {:.6}",
                detectability.signal_at_first_crossing.unwrap_or(0.0)
            ));
            lines.push(format!(
                "- envelope at first crossing: {:.6}",
                detectability.envelope_at_first_crossing.unwrap_or(0.0)
            ));
            lines.push(format!(
                "- crossing margin (signal - envelope) at first crossing: {:.6}",
                detectability.crossing_margin.unwrap_or(0.0)
            ));
        } else {
            lines.push("- first envelope crossing: not observed".to_string());
            lines.push("- signal / envelope values at first crossing: not applicable".to_string());
            lines.push("- crossing margin: not applicable".to_string());
        }
        if let Some(step) = detectability.consecutive_crossing_step {
            lines.push(format!("- first sustained crossing step: {step}"));
            lines.push(format!(
                "- first sustained crossing time: {:.6}",
                detectability.consecutive_crossing_time.unwrap_or(0.0)
            ));
        } else {
            lines.push("- first sustained crossing: not observed".to_string());
        }
        lines.push(format!(
            "- global signal peak: {:.6} at time {:.6}",
            detectability.global_signal_peak,
            detectability.global_signal_peak_time
        ));
        lines.push(format!(
            "- global envelope peak: {:.6} at time {:.6}",
            detectability.global_envelope_peak,
            detectability.global_envelope_peak_time
        ));
    }
    if let Some(softening) = &summary.softening {
        lines.push(String::new());
        lines.push("## Softening Sweep".to_string());
        lines.push(format!(
            "- softest scale studied: {:.3}",
            softening.softest_scale
        ));
        lines.push(format!(
            "- smallest eigenvalue at softest scale: {:.6}",
            softening.softest_smallest_eigenvalue
        ));
        lines.push(format!(
            "- max residual norm at softest scale: {:.6}",
            softening.softest_max_residual_norm
        ));
        lines.push(format!(
            "- max drift norm at softest scale: {:.6}",
            softening.softest_max_drift_norm
        ));
        lines.push(format!(
            "- max slew norm at softest scale: {:.6}",
            softening.softest_max_slew_norm
        ));
    }
    lines.push(String::new());
    lines.push("## Limitations".to_string());
    for limitation in &summary.limitations {
        lines.push(format!("- {limitation}"));
    }
    lines.join("\n")
}

fn write_text_pdf(path: &Path, summary: &RunSummary) -> Result<()> {
    let mut lines = Vec::new();
    lines.push("DSFB Lattice Demo Report".to_string());
    lines.push(String::new());
    lines.push(format!("Run directory: {}", summary.run_dir));
    lines.push(format!("Selected example set: {}", summary.selected_example));
    lines.push(format!(
        "Nominal smallest eigenvalue: {:.6}",
        summary.nominal_smallest_eigenvalue
    ));
    lines.push(format!(
        "Nominal largest eigenvalue: {:.6}",
        summary.nominal_largest_eigenvalue
    ));
    lines.push(String::new());
    lines.push("Experiment highlights".to_string());
    for experiment in &summary.experiments {
        lines.push(format!("{}:", experiment.name));
        lines.extend(wrap_text(&experiment.description, 86));
        lines.push(format!(
            "delta norm 2 = {:.6}, max shift = {:.6}, bound satisfied = {}",
            experiment.delta_norm_2, experiment.max_abs_shift, experiment.bound_satisfied
        ));
        lines.push(format!(
            "max residual = {:.6}, max drift = {:.6}, max slew = {:.6}",
            experiment.max_residual_norm, experiment.max_drift_norm, experiment.max_slew_norm
        ));
        lines.push(format!(
            "covariance off-diagonal energy = {:.6}",
            experiment.covariance_offdiag_energy
        ));
        lines.push(String::new());
    }
    if let Some(detectability) = &summary.detectability {
        lines.push("Detectability".to_string());
        lines.extend(wrap_text(
            "Detectability is evaluated pointwise in time using the same-time condition ||r(t)|| > E(t). Global peaks are reported separately for context and need not occur at the same time.",
            86,
        ));
        if let Some(step) = detectability.first_crossing_step {
            lines.push(format!(
                "first crossing step = {step}, first crossing time = {:.6}",
                detectability.first_crossing_time.unwrap_or(0.0)
            ));
            lines.push(format!(
                "signal_at_first_crossing = {:.6}, envelope_at_first_crossing = {:.6}, crossing_margin = {:.6}",
                detectability.signal_at_first_crossing.unwrap_or(0.0),
                detectability.envelope_at_first_crossing.unwrap_or(0.0),
                detectability.crossing_margin.unwrap_or(0.0)
            ));
        } else {
            lines.push("no first pointwise crossing was observed in this run".to_string());
        }
        if let Some(step) = detectability.consecutive_crossing_step {
            lines.push(format!(
                "sustained crossing step = {step}, sustained crossing time = {:.6}",
                detectability.consecutive_crossing_time.unwrap_or(0.0)
            ));
        } else {
            lines.push("no sustained crossing was observed under the configured consecutive-step rule".to_string());
        }
        lines.push(format!(
            "global signal peak = {:.6} at time {:.6}, global envelope peak = {:.6} at time {:.6}",
            detectability.global_signal_peak,
            detectability.global_signal_peak_time,
            detectability.global_envelope_peak,
            detectability.global_envelope_peak_time
        ));
        lines.push(String::new());
    }
    if let Some(softening) = &summary.softening {
        lines.push("Softening sweep".to_string());
        lines.push(format!(
            "softest scale = {:.3}, smallest eigenvalue there = {:.6}",
            softening.softest_scale, softening.softest_smallest_eigenvalue
        ));
        lines.push(format!(
            "softest-scale residual / drift / slew maxima = {:.6} / {:.6} / {:.6}",
            softening.softest_max_residual_norm,
            softening.softest_max_drift_norm,
            softening.softest_max_slew_norm
        ));
        lines.push(String::new());
    }
    lines.push("Limitations".to_string());
    for limitation in &summary.limitations {
        lines.extend(wrap_text(&format!("- {limitation}"), 86));
    }

    let pages = paginate_lines(&lines, 46);
    let mut objects = Vec::new();
    let mut page_ids = Vec::new();
    let font_id = 3usize;
    let mut next_id = 4usize;

    for page_lines in pages {
        let content_id = next_id;
        next_id += 1;
        let page_id = next_id;
        next_id += 1;
        let stream = build_pdf_stream(&page_lines);
        objects.push((
            content_id,
            format!(
                "<< /Length {} >>\nstream\n{}endstream\n",
                stream.len(),
                stream
            )
            .into_bytes(),
        ));
        objects.push((
            page_id,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>\n"
            )
            .into_bytes(),
        ));
        page_ids.push(page_id);
    }

    let kids = page_ids
        .iter()
        .map(|page_id| format!("{page_id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");

    objects.push((1, b"<< /Type /Catalog /Pages 2 0 R >>\n".to_vec()));
    objects.push((
        2,
        format!("<< /Type /Pages /Kids [{kids}] /Count {} >>\n", page_ids.len()).into_bytes(),
    ));
    objects.push((3, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n".to_vec()));
    objects.sort_by_key(|(id, _)| *id);

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = vec![0usize];
    for (id, data) in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        pdf.extend_from_slice(data);
        pdf.extend_from_slice(b"endobj\n");
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );

    fs::write(path, pdf).with_context(|| format!("failed to write {}", path.display()))
}

fn paginate_lines(lines: &[String], lines_per_page: usize) -> Vec<Vec<String>> {
    let mut pages = Vec::new();
    let mut current = Vec::new();
    for line in lines {
        if current.len() == lines_per_page {
            pages.push(current);
            current = Vec::new();
        }
        current.push(line.clone());
    }
    if !current.is_empty() {
        pages.push(current);
    }
    if pages.is_empty() {
        pages.push(vec![String::new()]);
    }
    pages
}

fn build_pdf_stream(lines: &[String]) -> String {
    let mut stream = String::from("BT\n/F1 16 Tf\n72 800 Td\n");
    let mut first = true;
    for line in lines {
        if first {
            stream.push_str(&format!("({}) Tj\n", escape_pdf_text(line)));
            first = false;
        } else {
            stream.push_str("0 -16 Td\n");
            stream.push_str(&format!("({}) Tj\n", escape_pdf_text(line)));
        }
    }
    stream.push_str("ET\n");
    stream
}
