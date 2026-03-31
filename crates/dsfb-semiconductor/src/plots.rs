use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarSet, GrammarState};
use crate::metrics::BenchmarkMetrics;
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use plotters::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 1400;
const HEIGHT: u32 = 800;

#[derive(Debug, Clone, Serialize)]
pub struct FigureManifest {
    pub figure_dir: PathBuf,
    pub files: Vec<String>,
}

pub fn generate_figures(
    run_dir: &Path,
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    grammar: &GrammarSet,
    metrics: &BenchmarkMetrics,
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
        drift_for_feature,
    )?;
    files.push("top_feature_drift.png".into());

    draw_multi_feature_chart(
        &figure_dir.join("top_feature_slew.png"),
        "Top feature slew traces",
        "Slew",
        &metrics.top_feature_indices,
        nominal,
        residuals,
        signs,
        slew_for_feature,
    )?;
    files.push("top_feature_slew.png".into());

    draw_grammar_timeline(&figure_dir, metrics, grammar)?;
    files.push("grammar_timeline.png".into());

    draw_baseline_comparison(&figure_dir, metrics)?;
    files.push("benchmark_comparison.png".into());

    Ok(FigureManifest { figure_dir, files })
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
    selector: F,
) -> Result<()>
where
    F: Fn(usize, &ResidualSet, &SignSet) -> Vec<f64>,
{
    let root = BitMapBackend::new(output_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;

    let mut all_values = Vec::new();
    for feature_index in top_feature_indices {
        all_values.extend(selector(*feature_index, residuals, signs));
    }
    let (min_value, max_value) = value_range(&all_values);
    let x_upper = residuals.traces.first().map(|trace| trace.norms.len()).unwrap_or(0);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0..x_upper, min_value..max_value)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc(y_desc)
        .draw()
        .map_err(plot_error)?;

    let palette = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];
    for (series_index, feature_index) in top_feature_indices.iter().enumerate() {
        let values = selector(*feature_index, residuals, signs);
        let color = palette[series_index % palette.len()];
        chart
            .draw_series(LineSeries::new(
                values.into_iter().enumerate(),
                ShapeStyle::from(color).stroke_width(2),
            ))
            .map_err(plot_error)?
            .label(nominal.features[*feature_index].feature_name.clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.85))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
}

fn residual_norms_for_feature(
    feature_index: usize,
    residuals: &ResidualSet,
    _signs: &SignSet,
) -> Vec<f64> {
    residuals.traces[feature_index].norms.clone()
}

fn drift_for_feature(feature_index: usize, _residuals: &ResidualSet, signs: &SignSet) -> Vec<f64> {
    signs.traces[feature_index].drift.clone()
}

fn slew_for_feature(feature_index: usize, _residuals: &ResidualSet, signs: &SignSet) -> Vec<f64> {
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
        .caption("DSFB grammar-state timeline (top features)", ("sans-serif", 28))
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
        for (run_index, state) in trace.states.iter().enumerate() {
            let color = match state {
                GrammarState::Admissible => RGBColor(220, 220, 220),
                GrammarState::Boundary => RGBColor(255, 179, 0),
                GrammarState::Violation => RGBColor(200, 0, 0),
            };
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

fn draw_baseline_comparison(figure_dir: &Path, metrics: &BenchmarkMetrics) -> Result<()> {
    let out_path = figure_dir.join("benchmark_comparison.png");
    let root = BitMapBackend::new(&out_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let max_value = metrics
        .summary
        .failure_runs
        .max(metrics.summary.failure_runs_with_preceding_dsfb_signal)
        .max(metrics.summary.failure_runs_with_preceding_threshold_signal)
        .max(1);

    let labels = [
        (
            "DSFB in pre-failure window",
            metrics.summary.failure_runs_with_preceding_dsfb_signal,
            BLUE.mix(0.7),
        ),
        (
            "Threshold in pre-failure window",
            metrics.summary.failure_runs_with_preceding_threshold_signal,
            RED.mix(0.7),
        ),
    ];

    let mut chart = ChartBuilder::on(&root)
        .caption("Failure-window benchmark comparison", ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(0..labels.len(), 0usize..(max_value + 5))
        .map_err(plot_error)?;
    chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(labels.len())
        .x_label_formatter(&|idx| {
            labels
                .get(*idx)
                .map(|row| row.0.to_string())
                .unwrap_or_default()
        })
        .y_desc("Failure runs with preceding signal")
        .draw()
        .map_err(plot_error)?;

    chart
        .draw_series(labels.iter().enumerate().map(|(index, (_, value, color))| {
            Rectangle::new([(index, 0usize), (index + 1, *value)], color.filled())
        }))
        .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
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
