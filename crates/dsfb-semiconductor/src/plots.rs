use crate::baselines::BaselineSet;
use crate::config::PipelineConfig;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarSet, GrammarState};
use crate::metrics::BenchmarkMetrics;
use crate::nominal::NominalModel;
use crate::precursor::PrecursorEvaluation;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use csv::Writer;
use plotters::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 1400;
const HEIGHT: u32 = 800;

#[derive(Debug, Clone, Serialize)]
pub struct DrscManifest {
    pub figure_file: String,
    pub trace_csv: String,
    pub feature_index: usize,
    pub feature_name: String,
    pub failure_run_index: usize,
    pub window_start_run_index: usize,
    pub window_end_run_index: usize,
    pub first_persistent_boundary_run: Option<usize>,
    pub first_persistent_violation_run: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FigureManifest {
    pub figure_dir: PathBuf,
    pub files: Vec<String>,
    pub drsc: Option<DrscManifest>,
}

pub fn generate_figures(
    run_dir: &Path,
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    metrics: &BenchmarkMetrics,
    precursor: &PrecursorEvaluation,
    config: &PipelineConfig,
) -> Result<FigureManifest> {
    let figure_dir = run_dir.join("figures");
    std::fs::create_dir_all(&figure_dir)?;

    let mut files = Vec::new();
    draw_missingness_chart(&figure_dir, dataset)?;
    files.push("missingness_top20.png".into());

    draw_multi_feature_chart(
        &figure_dir.join("top_feature_residual_norms.png"),
        "Top feature residual norms",
        "Residual norm",
        &metrics.top_feature_indices,
        nominal,
        residuals,
        signs,
        baselines,
        residual_norms_for_feature,
    )?;
    files.push("top_feature_residual_norms.png".into());

    draw_multi_feature_chart(
        &figure_dir.join("top_feature_drift.png"),
        "Top feature drift traces",
        "Drift",
        &metrics.top_feature_indices,
        nominal,
        residuals,
        signs,
        baselines,
        drift_for_feature,
    )?;
    files.push("top_feature_drift.png".into());

    draw_multi_feature_chart(
        &figure_dir.join("top_feature_ewma.png"),
        "Top feature EWMA traces",
        "EWMA residual norm",
        &metrics.top_feature_indices,
        nominal,
        residuals,
        signs,
        baselines,
        ewma_for_feature,
    )?;
    files.push("top_feature_ewma.png".into());

    draw_multi_feature_chart(
        &figure_dir.join("top_feature_slew.png"),
        "Top feature slew traces",
        "Slew",
        &metrics.top_feature_indices,
        nominal,
        residuals,
        signs,
        baselines,
        slew_for_feature,
    )?;
    files.push("top_feature_slew.png".into());

    draw_grammar_timeline(&figure_dir, metrics, grammar)?;
    files.push("grammar_timeline.png".into());

    draw_baseline_comparison(&figure_dir, metrics, precursor)?;
    files.push("benchmark_comparison.png".into());

    let drsc = if let Some(feature_index) = metrics.top_feature_indices.first().copied() {
        let figure_file = "drsc_top_feature.png".to_string();
        let trace_csv = "drsc_top_feature.csv".to_string();
        let drsc_window = drsc_window(dataset, grammar, feature_index, config.pre_failure_lookback_runs);
        draw_drsc_chart(
            &figure_dir.join(&figure_file),
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            feature_index,
            config,
            &drsc_window,
        )?;
        write_drsc_trace_csv(
            &run_dir.join(&trace_csv),
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            feature_index,
            &drsc_window,
        )?;
        files.push(figure_file.clone());
        Some(DrscManifest {
            figure_file,
            trace_csv,
            feature_index,
            feature_name: nominal.features[feature_index].feature_name.clone(),
            failure_run_index: drsc_window.failure_run_index,
            window_start_run_index: drsc_window.window_start,
            window_end_run_index: drsc_window.window_end,
            first_persistent_boundary_run: drsc_window.first_persistent_boundary_run,
            first_persistent_violation_run: drsc_window.first_persistent_violation_run,
        })
    } else {
        None
    };

    Ok(FigureManifest {
        figure_dir,
        files,
        drsc,
    })
}

fn draw_missingness_chart(figure_dir: &Path, dataset: &PreparedDataset) -> Result<()> {
    let mut rows = dataset
        .feature_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), dataset.per_feature_missing_fraction[index]))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(20);

    let out_path = figure_dir.join("missingness_top20.png");
    let root = BitMapBackend::new(&out_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let max_missing = rows
        .iter()
        .map(|(_, value)| *value)
        .fold(0.0_f64, f64::max)
        .max(0.1);

    let mut chart = ChartBuilder::on(&root)
        .caption("SECOM top-20 feature missingness", ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(0..rows.len(), 0.0f64..max_missing * 1.15)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(rows.len())
        .x_label_formatter(&|idx| rows.get(*idx).map(|row| row.0.clone()).unwrap_or_default())
        .y_desc("Missing fraction")
        .draw()
        .map_err(plot_error)?;

    chart
        .draw_series(rows.iter().enumerate().map(|(index, (_, value))| {
            Rectangle::new([(index, 0.0), (index + 1, *value)], BLUE.mix(0.7).filled())
        }))
        .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
}

fn draw_multi_feature_chart<F>(
    output_path: &Path,
    title: &str,
    y_desc: &str,
    top_feature_indices: &[usize],
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    selector: F,
) -> Result<()>
where
    F: Fn(usize, &ResidualSet, &SignSet, &BaselineSet) -> Vec<f64>,
{
    let root = BitMapBackend::new(output_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let titled = root.titled(title, ("sans-serif", 28)).map_err(plot_error)?;
    let columns = 3usize;
    let rows = top_feature_indices.len().max(1).div_ceil(columns);
    let areas = titled.split_evenly((rows, columns));
    let x_upper = residuals
        .traces
        .first()
        .map(|trace| trace.norms.len())
        .unwrap_or(0);

    for (area, feature_index) in areas.into_iter().zip(top_feature_indices.iter().copied()) {
        let values = selector(feature_index, residuals, signs, baselines);
        let (min_value, max_value) = value_range(&values);
        let mut chart = ChartBuilder::on(&area)
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(45)
            .caption(
                nominal.features[feature_index].feature_name.as_str(),
                ("sans-serif", 20),
            )
            .build_cartesian_2d(0..x_upper, min_value..max_value)
            .map_err(plot_error)?;

        chart
            .configure_mesh()
            .x_desc("Run")
            .y_desc(y_desc)
            .max_light_lines(4)
            .draw()
            .map_err(plot_error)?;

        chart
            .draw_series(LineSeries::new(
                values.into_iter().enumerate(),
                ShapeStyle::from(BLUE).stroke_width(2),
            ))
            .map_err(plot_error)?;
    }

    root.present().map_err(plot_error)?;
    Ok(())
}

fn residual_norms_for_feature(
    feature_index: usize,
    residuals: &ResidualSet,
    _signs: &SignSet,
    _baselines: &BaselineSet,
) -> Vec<f64> {
    residuals.traces[feature_index].norms.clone()
}

fn drift_for_feature(
    feature_index: usize,
    _residuals: &ResidualSet,
    signs: &SignSet,
    _baselines: &BaselineSet,
) -> Vec<f64> {
    signs.traces[feature_index].drift.clone()
}

fn ewma_for_feature(
    feature_index: usize,
    _residuals: &ResidualSet,
    _signs: &SignSet,
    baselines: &BaselineSet,
) -> Vec<f64> {
    baselines.ewma[feature_index].ewma.clone()
}

fn slew_for_feature(
    feature_index: usize,
    _residuals: &ResidualSet,
    signs: &SignSet,
    _baselines: &BaselineSet,
) -> Vec<f64> {
    signs.traces[feature_index].slew.clone()
}

fn draw_grammar_timeline(
    figure_dir: &Path,
    metrics: &BenchmarkMetrics,
    grammar: &GrammarSet,
) -> Result<()> {
    let out_path = figure_dir.join("grammar_timeline.png");
    let root = BitMapBackend::new(&out_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;

    let feature_indices = metrics.top_feature_indices.clone();
    let run_count = grammar
        .traces
        .first()
        .map(|trace| trace.states.len())
        .unwrap_or_default();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "DSFB grammar-state timeline (top features)",
            ("sans-serif", 28),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(120)
        .build_cartesian_2d(0..run_count, 0..feature_indices.len())
        .map_err(plot_error)?;
    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Run index")
        .y_labels(feature_indices.len())
        .y_label_formatter(&|idx| {
            feature_indices
                .get(*idx)
                .map(|feature_index| format!("S{:03}", feature_index + 1))
                .unwrap_or_default()
        })
        .draw()
        .map_err(plot_error)?;

    for (row_index, feature_index) in feature_indices.iter().enumerate() {
        let trace = &grammar.traces[*feature_index];
        for run_index in 0..trace.states.len() {
            let color = state_color(display_state(trace, run_index));
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, row_index), (run_index + 1, row_index + 1)],
                    color.filled(),
                )))
                .map_err(plot_error)?;
        }
    }

    root.present().map_err(plot_error)?;
    Ok(())
}

fn draw_baseline_comparison(
    figure_dir: &Path,
    metrics: &BenchmarkMetrics,
    precursor: &PrecursorEvaluation,
) -> Result<()> {
    let out_path = figure_dir.join("benchmark_comparison.png");
    let root = BitMapBackend::new(&out_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let areas = root.split_evenly((1, 2));
    let lead_labels = [
        (
            "DSFB precursor",
            precursor.summary.mean_lead_time_runs.unwrap_or(0.0),
            MAGENTA.mix(0.7),
        ),
        (
            "DSFB persistent boundary",
            metrics
                .lead_time_summary
                .mean_persistent_boundary_lead_runs
                .unwrap_or(0.0),
            BLUE.mix(0.7),
        ),
        (
            "DSFB persistent violation",
            metrics
                .lead_time_summary
                .mean_persistent_violation_lead_runs
                .unwrap_or(0.0),
            CYAN.mix(0.7),
        ),
        (
            "EWMA",
            metrics
                .lead_time_summary
                .mean_ewma_lead_runs
                .unwrap_or(0.0),
            GREEN.mix(0.7),
        ),
        (
            "Threshold",
            metrics
                .lead_time_summary
                .mean_threshold_lead_runs
                .unwrap_or(0.0),
            RED.mix(0.7),
        ),
    ];
    let nuisance_labels = [
        (
            "DSFB precursor",
            precursor.summary.pass_run_nuisance_proxy,
            MAGENTA.mix(0.7),
        ),
        (
            "DSFB persistent boundary",
            metrics
                .summary
                .pass_run_dsfb_persistent_boundary_nuisance_rate,
            BLUE.mix(0.7),
        ),
        (
            "DSFB persistent violation",
            metrics
                .summary
                .pass_run_dsfb_persistent_violation_nuisance_rate,
            CYAN.mix(0.7),
        ),
        (
            "EWMA",
            metrics.summary.pass_run_ewma_nuisance_rate,
            GREEN.mix(0.7),
        ),
        (
            "Threshold",
            metrics.summary.pass_run_threshold_nuisance_rate,
            RED.mix(0.7),
        ),
    ];

    let max_lead = lead_labels
        .iter()
        .map(|(_, value, _)| *value)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let max_nuisance = nuisance_labels
        .iter()
        .map(|(_, value, _)| *value)
        .fold(0.0_f64, f64::max)
        .max(0.05);

    let mut lead_chart = ChartBuilder::on(&areas[0])
        .caption("Mean pre-failure lead", ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..lead_labels.len(), 0.0f64..(max_lead * 1.1))
        .map_err(plot_error)?;
    lead_chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(lead_labels.len())
        .x_label_formatter(&|idx| {
            lead_labels
                .get(*idx)
                .map(|row| row.0.to_string())
                .unwrap_or_default()
        })
        .y_desc("Mean lead runs")
        .draw()
        .map_err(plot_error)?;
    lead_chart
        .draw_series(
            lead_labels
                .iter()
                .enumerate()
                .map(|(index, (_label, value, color))| {
                    Rectangle::new([(index, 0.0f64), (index + 1, *value)], color.filled())
                }),
        )
        .map_err(plot_error)?;

    let mut nuisance_chart = ChartBuilder::on(&areas[1])
        .caption("Pass-run nuisance proxy", ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..nuisance_labels.len(), 0.0f64..(max_nuisance * 1.2))
        .map_err(plot_error)?;
    nuisance_chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(nuisance_labels.len())
        .x_label_formatter(&|idx| {
            nuisance_labels
                .get(*idx)
                .map(|row| row.0.to_string())
                .unwrap_or_default()
        })
        .y_desc("Fraction of pass-labeled runs with signal")
        .draw()
        .map_err(plot_error)?;
    nuisance_chart
        .draw_series(
            nuisance_labels
                .iter()
                .enumerate()
                .map(|(index, (_label, value, color))| {
                    Rectangle::new([(index, 0.0f64), (index + 1, *value)], color.filled())
                }),
        )
        .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
}

fn draw_drsc_chart(
    output_path: &Path,
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    feature_index: usize,
    config: &PipelineConfig,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let feature = &nominal.features[feature_index];
    let residual_trace = &residuals.traces[feature_index];
    let sign_trace = &signs.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let grammar_trace = &grammar.traces[feature_index];

    let window_start = drsc_window.window_start;
    let window_end = drsc_window.window_end;
    let window_runs = window_end.saturating_sub(window_start);
    let residual_scale = positive_or_one(feature.rho);
    let drift_scale = positive_or_one(sign_trace.drift_threshold);
    let slew_scale = positive_or_one(sign_trace.slew_threshold);
    let ewma_scale = positive_or_one(ewma_trace.threshold);

    let residual_series = residual_trace
        .residuals
        .iter()
        .skip(window_start)
        .take(window_runs)
        .map(|value| *value / residual_scale)
        .collect::<Vec<_>>();
    let drift_series = sign_trace
        .drift
        .iter()
        .skip(window_start)
        .take(window_runs)
        .map(|value| *value / drift_scale)
        .collect::<Vec<_>>();
    let slew_series = sign_trace
        .slew
        .iter()
        .skip(window_start)
        .take(window_runs)
        .map(|value| *value / slew_scale)
        .collect::<Vec<_>>();
    let occupancy_series = residual_trace
        .norms
        .iter()
        .skip(window_start)
        .take(window_runs)
        .map(|value| *value / residual_scale)
        .collect::<Vec<_>>();
    let ewma_series = ewma_trace
        .ewma
        .iter()
        .skip(window_start)
        .take(window_runs)
        .map(|value| *value / ewma_scale)
        .collect::<Vec<_>>();

    let root = BitMapBackend::new(output_path, (WIDTH, HEIGHT + 250)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let areas = root.split_evenly((3, 1));

    let structure_max = residual_series
        .iter()
        .chain(drift_series.iter())
        .chain(slew_series.iter())
        .map(|value| value.abs())
        .fold(1.2_f64, f64::max)
        .max(1.2);
    let mut structure_chart = ChartBuilder::on(&areas[0])
        .caption(
            format!(
                "DRSC: persistent-state view for feature {} around failure run {}",
                feature.feature_name, drsc_window.failure_run_index
            ),
            ("sans-serif", 26),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start..window_end, -structure_max..structure_max)
        .map_err(plot_error)?;
    structure_chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc("Normalized residual / drift / slew")
        .draw()
        .map_err(plot_error)?;
    structure_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(residual_series.iter().copied()),
            ShapeStyle::from(BLUE).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("residual / rho")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], BLUE.stroke_width(2)));
    structure_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(drift_series.iter().copied()),
            ShapeStyle::from(GREEN).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("drift / drift threshold")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], GREEN.stroke_width(2)));
    structure_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(slew_series.iter().copied()),
            ShapeStyle::from(MAGENTA).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("slew / slew threshold")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], MAGENTA.stroke_width(2)));
    structure_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    let mut state_chart = ChartBuilder::on(&areas[1])
        .caption(
            "Persistent deterministic state band (hysteresis confirmed)",
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start as f64..window_end as f64, 0.0f64..3.0f64)
        .map_err(plot_error)?;
    state_chart
        .configure_mesh()
        .disable_y_mesh()
        .disable_x_mesh()
        .x_desc("Run index")
        .y_labels(0)
        .draw()
        .map_err(plot_error)?;
    for run_index in window_start..window_end {
        let color = state_color(display_state(grammar_trace, run_index));
        state_chart
            .draw_series(std::iter::once(Rectangle::new(
                [(run_index as f64, 0.0), ((run_index + 1) as f64, 3.0)],
                color.filled(),
            )))
            .map_err(plot_error)?;
    }

    let occupancy_max = occupancy_series
        .iter()
        .chain(ewma_series.iter())
        .copied()
        .fold(1.2_f64, f64::max)
        .max(1.2);
    let mut occupancy_chart = ChartBuilder::on(&areas[2])
        .caption(
            "Admissibility overlay (persistent-state context)",
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start..window_end, 0.0f64..occupancy_max * 1.1)
        .map_err(plot_error)?;
    occupancy_chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc("Normalized occupancy")
        .draw()
        .map_err(plot_error)?;
    for run_index in window_start..window_end {
        if dataset.labels[run_index] == 1 {
            occupancy_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, 0.0), (run_index + 1, occupancy_max * 1.1)],
                    RGBAColor(160, 0, 0, 0.08).filled(),
                )))
                .map_err(plot_error)?;
        }
    }
    occupancy_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(occupancy_series.iter().copied()),
            ShapeStyle::from(BLUE).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("|r| / rho")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], BLUE.stroke_width(2)));
    occupancy_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(ewma_series.iter().copied()),
            ShapeStyle::from(GREEN).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("EWMA / EWMA threshold")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], GREEN.stroke_width(2)));
    occupancy_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (window_start, nominal.features[feature_index].rho * 0.0 + 1.0),
                (window_end, 1.0),
            ],
            RED.mix(0.6).stroke_width(2),
        )))
        .map_err(plot_error)?
        .label("violation threshold")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], RED.mix(0.6).stroke_width(2)));
    occupancy_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (window_start, config.boundary_fraction_of_rho),
                (window_end, config.boundary_fraction_of_rho),
            ],
            RGBColor(255, 179, 0).mix(0.8).stroke_width(2),
        )))
        .map_err(plot_error)?
        .label("boundary fraction of rho")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 18, y)],
                RGBColor(255, 179, 0).mix(0.8).stroke_width(2),
            )
        });

    for (run_index, label, color) in [
        (
            drsc_window.first_persistent_boundary_run,
            "first persistent boundary",
            RGBColor(255, 179, 0),
        ),
        (
            drsc_window.first_persistent_violation_run,
            "first persistent violation",
            RGBColor(200, 0, 0),
        ),
        (
            Some(drsc_window.failure_run_index),
            "failure label",
            RGBColor(90, 90, 90),
        ),
    ] {
        if let Some(run_index) = run_index {
            structure_chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(run_index, -structure_max), (run_index, structure_max)],
                    color.mix(0.55).stroke_width(2),
                )))
                .map_err(plot_error)?;
            occupancy_chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(run_index, 0.0f64), (run_index, occupancy_max * 1.1)],
                    color.mix(0.55).stroke_width(2),
                )))
                .map_err(plot_error)?;
            state_chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![
                        (run_index as f64, 0.0f64),
                        (run_index as f64, 3.0f64),
                    ],
                    color.mix(0.7).stroke_width(2),
                )))
                .map_err(plot_error)?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 18, y)], color.mix(0.7).stroke_width(2))
                });
        }
    }
    state_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;
    occupancy_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
}

fn write_drsc_trace_csv(
    output_path: &Path,
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    feature_index: usize,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let feature = &nominal.features[feature_index];
    let residual_trace = &residuals.traces[feature_index];
    let sign_trace = &signs.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let grammar_trace = &grammar.traces[feature_index];
    let residual_scale = positive_or_one(feature.rho);
    let drift_scale = positive_or_one(sign_trace.drift_threshold);
    let slew_scale = positive_or_one(sign_trace.slew_threshold);
    let ewma_scale = positive_or_one(ewma_trace.threshold);

    let mut writer = Writer::from_path(output_path)?;
    writer.write_record([
        "run_index",
        "timestamp",
        "label",
        "feature",
        "residual",
        "residual_norm",
        "residual_over_rho",
        "drift",
        "drift_over_threshold",
        "slew",
        "slew_over_threshold",
        "ewma",
        "ewma_over_threshold",
        "threshold_alarm",
        "ewma_alarm",
        "raw_state",
        "confirmed_state",
        "persistent_state",
        "raw_reason",
        "confirmed_reason",
        "persistent_boundary",
        "persistent_violation",
        "is_failure_run",
        "is_first_persistent_boundary_before_failure",
        "is_first_persistent_violation_before_failure",
    ])?;

    for run_index in drsc_window.window_start..drsc_window.window_end {
        writer.write_record([
            run_index.to_string(),
            dataset.timestamps[run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            dataset.labels[run_index].to_string(),
            feature.feature_name.clone(),
            residual_trace.residuals[run_index].to_string(),
            residual_trace.norms[run_index].to_string(),
            (residual_trace.residuals[run_index] / residual_scale).to_string(),
            sign_trace.drift[run_index].to_string(),
            (sign_trace.drift[run_index] / drift_scale).to_string(),
            sign_trace.slew[run_index].to_string(),
            (sign_trace.slew[run_index] / slew_scale).to_string(),
            ewma_trace.ewma[run_index].to_string(),
            (ewma_trace.ewma[run_index] / ewma_scale).to_string(),
            residual_trace.threshold_alarm[run_index].to_string(),
            ewma_trace.alarm[run_index].to_string(),
            format!("{:?}", grammar_trace.raw_states[run_index]),
            format!("{:?}", grammar_trace.states[run_index]),
            format!("{:?}", display_state(grammar_trace, run_index)),
            format!("{:?}", grammar_trace.raw_reasons[run_index]),
            format!("{:?}", grammar_trace.reasons[run_index]),
            grammar_trace.persistent_boundary[run_index].to_string(),
            grammar_trace.persistent_violation[run_index].to_string(),
            (run_index == drsc_window.failure_run_index).to_string(),
            (Some(run_index) == drsc_window.first_persistent_boundary_run).to_string(),
            (Some(run_index) == drsc_window.first_persistent_violation_run).to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct DrscWindow {
    failure_run_index: usize,
    window_start: usize,
    window_end: usize,
    first_persistent_boundary_run: Option<usize>,
    first_persistent_violation_run: Option<usize>,
}

fn drsc_window(
    dataset: &PreparedDataset,
    grammar: &GrammarSet,
    feature_index: usize,
    lookback_runs: usize,
) -> DrscWindow {
    let trace = &grammar.traces[feature_index];
    let failure_run_index = dataset
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| (*label == 1).then_some(index))
        .find(|&failure_index| {
            let start = failure_index.saturating_sub(lookback_runs);
            trace.persistent_boundary[start..failure_index]
                .iter()
                .any(|flag| *flag)
                || trace.persistent_violation[start..failure_index]
                    .iter()
                    .any(|flag| *flag)
        })
        .or_else(|| {
            dataset
                .labels
                .iter()
                .enumerate()
                .find_map(|(index, label)| (*label == 1).then_some(index))
        })
        .unwrap_or_else(|| dataset.labels.len().saturating_sub(1));
    let window_start = failure_run_index.saturating_sub(lookback_runs);
    let window_end = (failure_run_index + 1).min(dataset.labels.len());
    let first_persistent_boundary_run = (window_start..failure_run_index)
        .find(|&run_index| trace.persistent_boundary[run_index]);
    let first_persistent_violation_run = (window_start..failure_run_index)
        .find(|&run_index| trace.persistent_violation[run_index]);

    DrscWindow {
        failure_run_index,
        window_start,
        window_end,
        first_persistent_boundary_run,
        first_persistent_violation_run,
    }
}

fn display_state(trace: &crate::grammar::FeatureGrammarTrace, run_index: usize) -> GrammarState {
    if trace.persistent_violation[run_index] {
        GrammarState::Violation
    } else if trace.persistent_boundary[run_index] {
        GrammarState::Boundary
    } else {
        GrammarState::Admissible
    }
}

fn state_color(state: GrammarState) -> RGBColor {
    match state {
        GrammarState::Admissible => RGBColor(220, 220, 220),
        GrammarState::Boundary => RGBColor(255, 179, 0),
        GrammarState::Violation => RGBColor(200, 0, 0),
    }
}

fn value_range(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (-1.0, 1.0);
    }
    let min_value = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_value = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max_value - min_value).abs() < f64::EPSILON {
        (min_value - 1.0, max_value + 1.0)
    } else {
        let padding = (max_value - min_value) * 0.1;
        (min_value - padding, max_value + padding)
    }
}

fn plot_error<E: std::fmt::Display>(err: E) -> DsfbSemiconductorError {
    DsfbSemiconductorError::DatasetFormat(format!("plotting error: {err}"))
}

fn positive_or_one(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}
