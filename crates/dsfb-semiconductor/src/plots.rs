use crate::baselines::BaselineSet;
use crate::config::PipelineConfig;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::{GrammarSet, GrammarState};
use crate::metrics::BenchmarkMetrics;
use crate::nominal::NominalModel;
use crate::precursor::DsaEvaluation;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use crate::signs::SignSet;
use csv::Writer;
use plotters::coord::types::{RangedCoordf64, RangedCoordusize};
use plotters::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 1400;
const HEIGHT: u32 = 800;
const COMBINED_WIDTH: u32 = 2880;
const COMBINED_HEIGHT: u32 = 1620;

#[derive(Debug, Clone)]
struct DrscDsaCombinedRow {
    run_index: usize,
    timestamp: String,
    label: i8,
    feature: String,
    residual_over_rho: f64,
    drift_over_threshold: f64,
    slew_over_threshold: f64,
    display_state: GrammarState,
    persistent_boundary: bool,
    persistent_violation: bool,
    feature_dsa_alert: bool,
    run_level_dsa_alert: bool,
    feature_count_dsa_alert: usize,
    threshold_run_signal: bool,
    ewma_run_signal: bool,
}

#[derive(Debug, Clone)]
struct AnnotationCandidate {
    run_index: usize,
    label: String,
}

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
pub struct DsaFocusManifest {
    pub figure_file: String,
    pub trace_csv: String,
    pub feature_index: usize,
    pub feature_name: String,
    pub failure_run_index: usize,
    pub window_start_run_index: usize,
    pub window_end_run_index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DrscDsaCombinedManifest {
    pub figure_file: String,
    pub trace_csv: String,
    pub feature_index: usize,
    pub feature_name: String,
    pub failure_run_index: usize,
    pub window_start_run_index: usize,
    pub window_end_run_index: usize,
    pub feature_selection_basis: String,
    pub normalization_note: String,
    pub state_display_note: String,
    pub dsa_rendering_note: String,
    pub baseline_rendering_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FigureManifest {
    pub figure_dir: PathBuf,
    pub files: Vec<String>,
    pub drsc: Option<DrscManifest>,
    pub drsc_dsa_combined: Option<DrscDsaCombinedManifest>,
    pub dsa_focus: Option<DsaFocusManifest>,
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
    dsa: &DsaEvaluation,
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

    draw_baseline_comparison(&figure_dir, metrics, dsa)?;
    files.push("benchmark_comparison.png".into());

    let (drsc, drsc_dsa_combined, dsa_focus) = if let Some(feature_index) =
        metrics.top_feature_indices.first().copied()
    {
        let figure_file = "drsc_top_feature.png".to_string();
        let trace_csv = "drsc_top_feature.csv".to_string();
        let drsc_window = drsc_window(
            dataset,
            grammar,
            feature_index,
            config.pre_failure_lookback_runs,
        );
        draw_drsc_chart(
            &figure_dir.join(&figure_file),
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            dsa,
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
            dsa,
            feature_index,
            &drsc_window,
        )?;
        files.push(figure_file.clone());
        let combined_figure_file = "drsc_dsa_combined.png".to_string();
        let combined_trace_csv = "drsc_dsa_combined.csv".to_string();
        let combined_trace = build_drsc_dsa_combined_trace(
            dataset,
            nominal,
            residuals,
            signs,
            baselines,
            grammar,
            dsa,
            feature_index,
            &drsc_window,
        )?;
        draw_drsc_dsa_combined_chart(
            &figure_dir.join(&combined_figure_file),
            &combined_trace,
            &drsc_window,
        )?;
        write_drsc_dsa_combined_trace_csv(&run_dir.join(&combined_trace_csv), &combined_trace)?;
        files.push(combined_figure_file.clone());
        let dsa_figure_file = "dsa_top_feature.png".to_string();
        let dsa_trace_csv = "dsa_top_feature.csv".to_string();
        draw_dsa_focus_chart(
            &figure_dir.join(&dsa_figure_file),
            dataset,
            residuals,
            baselines,
            grammar,
            dsa,
            feature_index,
            config,
            &drsc_window,
        )?;
        write_dsa_focus_trace_csv(
            &run_dir.join(&dsa_trace_csv),
            dataset,
            baselines,
            grammar,
            dsa,
            feature_index,
            &drsc_window,
        )?;
        files.push(dsa_figure_file.clone());
        (
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
            }),
            Some(DrscDsaCombinedManifest {
                figure_file: combined_figure_file,
                trace_csv: combined_trace_csv,
                feature_index,
                feature_name: nominal.features[feature_index].feature_name.clone(),
                failure_run_index: drsc_window.failure_run_index,
                window_start_run_index: drsc_window.window_start,
                window_end_run_index: drsc_window.window_end,
                feature_selection_basis:
                    "Top boundary-activity feature selected by benchmark feature ranking."
                        .into(),
                normalization_note:
                    "Residual is residual/rho, drift is drift/drift-threshold, and slew is slew/slew-threshold; each scale falls back to 1.0 only if a saved threshold is non-positive."
                        .into(),
                state_display_note:
                    "Display band uses the actual persistent DSFB state alias mapping: Admissible, Boundary, Violation."
                        .into(),
                dsa_rendering_note:
                    "Panel 3 uses a two-strip binary rendering: upper strip is feature-level DSA alert; lower strip is the corroborated run-level DSA alert."
                        .into(),
                baseline_rendering_note:
                    "Panel 4 uses run-level threshold and EWMA any-feature alarm timing as binary trigger rows."
                        .into(),
            }),
            Some(DsaFocusManifest {
                figure_file: dsa_figure_file,
                trace_csv: dsa_trace_csv,
                feature_index,
                feature_name: nominal.features[feature_index].feature_name.clone(),
                failure_run_index: drsc_window.failure_run_index,
                window_start_run_index: drsc_window.window_start,
                window_end_run_index: drsc_window.window_end,
            }),
        )
    } else {
        (None, None, None)
    };

    Ok(FigureManifest {
        figure_dir,
        files,
        drsc,
        drsc_dsa_combined,
        dsa_focus,
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

fn draw_named_series<'a, DB: DrawingBackend>(
    chart: &mut ChartContext<'a, DB, Cartesian2d<RangedCoordusize, RangedCoordf64>>,
    start_index: usize,
    values: &[f64],
    color: RGBColor,
    label: &'static str,
) -> Result<()> {
    chart
        .draw_series(LineSeries::new(
            (start_index..(start_index + values.len())).zip(values.iter().copied()),
            ShapeStyle::from(color).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label(label)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], color.stroke_width(2)));
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
    dsa: &DsaEvaluation,
) -> Result<()> {
    let out_path = figure_dir.join("benchmark_comparison.png");
    let root = BitMapBackend::new(&out_path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let areas = root.split_evenly((1, 2));
    let lead_labels = [
        (
            "DSA",
            dsa.summary.mean_lead_time_runs.unwrap_or(0.0),
            MAGENTA.mix(0.7),
        ),
        (
            "DSFB raw boundary",
            metrics
                .lead_time_summary
                .mean_raw_boundary_lead_runs
                .unwrap_or(0.0),
            BLUE.mix(0.7),
        ),
        (
            "DSFB Violation",
            metrics
                .lead_time_summary
                .mean_raw_violation_lead_runs
                .unwrap_or(0.0),
            CYAN.mix(0.7),
        ),
        (
            "EWMA",
            metrics.lead_time_summary.mean_ewma_lead_runs.unwrap_or(0.0),
            GREEN.mix(0.7),
        ),
        (
            "CUSUM",
            metrics
                .lead_time_summary
                .mean_cusum_lead_runs
                .unwrap_or(0.0),
            RGBColor(120, 70, 20).mix(0.7),
        ),
        (
            "Run energy",
            dsa.comparison_summary
                .run_energy
                .mean_lead_time_runs
                .unwrap_or(0.0),
            RGBColor(90, 90, 90).mix(0.7),
        ),
        (
            "PCA T2/SPE",
            dsa.comparison_summary
                .pca_fdc
                .mean_lead_time_runs
                .unwrap_or(0.0),
            RGBColor(80, 40, 140).mix(0.7),
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
        ("DSA", dsa.summary.pass_run_nuisance_proxy, MAGENTA.mix(0.7)),
        (
            "DSFB raw boundary",
            metrics.summary.pass_run_dsfb_raw_boundary_nuisance_rate,
            BLUE.mix(0.7),
        ),
        (
            "DSFB Violation",
            metrics.summary.pass_run_dsfb_raw_violation_nuisance_rate,
            CYAN.mix(0.7),
        ),
        (
            "EWMA",
            metrics.summary.pass_run_ewma_nuisance_rate,
            GREEN.mix(0.7),
        ),
        (
            "CUSUM",
            metrics.summary.pass_run_cusum_nuisance_rate,
            RGBColor(120, 70, 20).mix(0.7),
        ),
        (
            "Run energy",
            metrics.summary.pass_run_run_energy_nuisance_rate,
            RGBColor(90, 90, 90).mix(0.7),
        ),
        (
            "PCA T2/SPE",
            metrics.summary.pass_run_pca_fdc_nuisance_rate,
            RGBColor(80, 40, 140).mix(0.7),
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
    dsa: &DsaEvaluation,
    feature_index: usize,
    config: &PipelineConfig,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let feature = &nominal.features[feature_index];
    let residual_trace = &residuals.traces[feature_index];
    let sign_trace = &signs.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let grammar_trace = &grammar.traces[feature_index];
    let dsa_trace = &dsa.traces[feature_index];

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

    let root = BitMapBackend::new(output_path, (WIDTH, HEIGHT + 420)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let areas = root.split_evenly((4, 1));

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

    let dsa_score = dsa_trace
        .dsa_score
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let dsa_score_max = dsa_score
        .iter()
        .copied()
        .fold(config.dsa.alert_tau.max(1.0), f64::max)
        .max(config.dsa.alert_tau);
    let mut dsa_chart = ChartBuilder::on(&areas[2])
        .caption(
            "DSA persistence-constrained overlay (feature + run level)",
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start..window_end, 0.0f64..(dsa_score_max * 1.15))
        .map_err(plot_error)?;
    dsa_chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc("DSA score")
        .draw()
        .map_err(plot_error)?;
    for run_index in window_start..window_end {
        if dsa.run_signals.primary_run_alert[run_index] {
            dsa_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, 0.0), (run_index + 1, dsa_score_max * 1.15)],
                    RGBAColor(160, 0, 160, 0.08).filled(),
                )))
                .map_err(plot_error)?;
        } else if !dsa_trace.consistent[run_index] {
            dsa_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, 0.0), (run_index + 1, dsa_score_max * 1.15)],
                    RGBAColor(180, 180, 180, 0.08).filled(),
                )))
                .map_err(plot_error)?;
        }
    }
    dsa_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(dsa_score.iter().copied()),
            ShapeStyle::from(RGBColor(160, 0, 160)).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("feature DSA score")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 18, y)],
                RGBColor(160, 0, 160).stroke_width(2),
            )
        });
    dsa_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (window_start, config.dsa.alert_tau),
                (window_end, config.dsa.alert_tau),
            ],
            RED.mix(0.8).stroke_width(2),
        )))
        .map_err(plot_error)?
        .label("DSA tau")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], RED.mix(0.8).stroke_width(2)));
    dsa_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    let run_energy_series = baselines.run_energy.energy[window_start..window_end]
        .iter()
        .map(|value| *value / positive_or_one(baselines.run_energy.threshold))
        .collect::<Vec<_>>();
    let pca_fdc_series = (window_start..window_end)
        .map(|run_index| {
            let t2 =
                baselines.pca_fdc.t2[run_index] / positive_or_one(baselines.pca_fdc.t2_threshold);
            let spe =
                baselines.pca_fdc.spe[run_index] / positive_or_one(baselines.pca_fdc.spe_threshold);
            t2.max(spe)
        })
        .collect::<Vec<_>>();
    let occupancy_max = occupancy_series
        .iter()
        .chain(ewma_series.iter())
        .chain(run_energy_series.iter())
        .chain(pca_fdc_series.iter())
        .copied()
        .fold(1.2_f64, f64::max)
        .max(1.2);
    let mut occupancy_chart = ChartBuilder::on(&areas[3])
        .caption(
            "Admissibility and run-level comparator overlay",
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
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(run_energy_series.iter().copied()),
            ShapeStyle::from(BLACK.mix(0.75)).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("run energy / threshold")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], BLACK.mix(0.75).stroke_width(2))
        });
    occupancy_chart
        .draw_series(LineSeries::new(
            (window_start..window_end).zip(pca_fdc_series.iter().copied()),
            ShapeStyle::from(RGBColor(80, 40, 140)).stroke_width(2),
        ))
        .map_err(plot_error)?
        .label("PCA T2/SPE / threshold")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 18, y)],
                RGBColor(80, 40, 140).stroke_width(2),
            )
        });
    occupancy_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (
                    window_start,
                    nominal.features[feature_index].rho * 0.0 + 1.0,
                ),
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
                    vec![(run_index as f64, 0.0f64), (run_index as f64, 3.0f64)],
                    color.mix(0.7).stroke_width(2),
                )))
                .map_err(plot_error)?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 18, y)], color.mix(0.7).stroke_width(2))
                });
            dsa_chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(run_index, 0.0f64), (run_index, dsa_score_max * 1.15)],
                    color.mix(0.55).stroke_width(2),
                )))
                .map_err(plot_error)?;
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

fn build_drsc_dsa_combined_trace(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    signs: &SignSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    dsa: &DsaEvaluation,
    feature_index: usize,
    drsc_window: &DrscWindow,
) -> Result<Vec<DrscDsaCombinedRow>> {
    if drsc_window.window_start >= drsc_window.window_end {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "combined DRSC+DSA figure requires a non-empty window".into(),
        ));
    }

    let feature = nominal.features.get(feature_index).ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(format!(
            "combined DRSC+DSA figure missing nominal feature index {feature_index}"
        ))
    })?;
    let residual_trace = residuals.traces.get(feature_index).ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(format!(
            "combined DRSC+DSA figure missing residual trace for feature index {feature_index}"
        ))
    })?;
    let sign_trace = signs.traces.get(feature_index).ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(format!(
            "combined DRSC+DSA figure missing sign trace for feature index {feature_index}"
        ))
    })?;
    let grammar_trace = grammar.traces.get(feature_index).ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(format!(
            "combined DRSC+DSA figure missing grammar trace for feature index {feature_index}"
        ))
    })?;
    let dsa_trace = dsa.traces.get(feature_index).ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat(format!(
            "combined DRSC+DSA figure missing DSA trace for feature index {feature_index}"
        ))
    })?;

    let residual_scale = positive_or_one(feature.rho);
    let drift_scale = positive_or_one(sign_trace.drift_threshold);
    let slew_scale = positive_or_one(sign_trace.slew_threshold);
    let threshold_run_signal = run_level_threshold_signal(residuals);
    let ewma_run_signal = run_level_ewma_signal(baselines);

    let mut rows = Vec::with_capacity(drsc_window.window_end - drsc_window.window_start);
    for run_index in drsc_window.window_start..drsc_window.window_end {
        rows.push(DrscDsaCombinedRow {
            run_index,
            timestamp: dataset.timestamps[run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            label: dataset.labels[run_index],
            feature: feature.feature_name.clone(),
            residual_over_rho: residual_trace.residuals[run_index] / residual_scale,
            drift_over_threshold: sign_trace.drift[run_index] / drift_scale,
            slew_over_threshold: sign_trace.slew[run_index] / slew_scale,
            display_state: display_state(grammar_trace, run_index),
            persistent_boundary: grammar_trace.persistent_boundary[run_index],
            persistent_violation: grammar_trace.persistent_violation[run_index],
            feature_dsa_alert: dsa_trace.dsa_alert[run_index],
            run_level_dsa_alert: dsa.run_signals.primary_run_alert[run_index],
            feature_count_dsa_alert: dsa.run_signals.feature_count_dsa_alert[run_index],
            threshold_run_signal: threshold_run_signal[run_index],
            ewma_run_signal: ewma_run_signal[run_index],
        });
    }

    if rows.is_empty() {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "combined DRSC+DSA figure produced no rows".into(),
        ));
    }

    Ok(rows)
}

fn write_drsc_dsa_combined_trace_csv(
    output_path: &Path,
    rows: &[DrscDsaCombinedRow],
) -> Result<()> {
    if rows.is_empty() {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "combined DRSC+DSA CSV requires at least one row".into(),
        ));
    }

    let mut writer = Writer::from_path(output_path)?;
    writer.write_record([
        "run_index",
        "timestamp",
        "label",
        "feature",
        "residual_over_rho",
        "drift_over_threshold",
        "slew_over_threshold",
        "display_state",
        "persistent_boundary",
        "persistent_violation",
        "feature_dsa_alert",
        "run_level_dsa_alert",
        "feature_count_dsa_alert",
        "threshold_run_signal",
        "ewma_run_signal",
    ])?;

    for row in rows {
        writer.write_record([
            row.run_index.to_string(),
            row.timestamp.clone(),
            row.label.to_string(),
            row.feature.clone(),
            row.residual_over_rho.to_string(),
            row.drift_over_threshold.to_string(),
            row.slew_over_threshold.to_string(),
            format!("{:?}", row.display_state),
            row.persistent_boundary.to_string(),
            row.persistent_violation.to_string(),
            row.feature_dsa_alert.to_string(),
            row.run_level_dsa_alert.to_string(),
            row.feature_count_dsa_alert.to_string(),
            row.threshold_run_signal.to_string(),
            row.ewma_run_signal.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn draw_drsc_dsa_combined_chart(
    output_path: &Path,
    rows: &[DrscDsaCombinedRow],
    drsc_window: &DrscWindow,
) -> Result<()> {
    if rows.is_empty() {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "combined DRSC+DSA figure requires at least one row".into(),
        ));
    }

    let window_start = rows.first().map(|row| row.run_index).unwrap_or(0);
    let window_end = rows.last().map(|row| row.run_index + 1).unwrap_or(0);
    let feature_name = rows
        .first()
        .map(|row| row.feature.clone())
        .unwrap_or_else(|| "unknown".into());
    let residual_points = rows
        .iter()
        .map(|row| (row.run_index, row.residual_over_rho))
        .collect::<Vec<_>>();
    let drift_points = rows
        .iter()
        .map(|row| (row.run_index, row.drift_over_threshold))
        .collect::<Vec<_>>();
    let slew_points = rows
        .iter()
        .map(|row| (row.run_index, row.slew_over_threshold))
        .collect::<Vec<_>>();

    let root =
        BitMapBackend::new(output_path, (COMBINED_WIDTH, COMBINED_HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;

    // Non-equal panel heights: signal panel 40%, state/DSA/scalar each 20%.
    let total_h = COMBINED_HEIGHT as f64;
    let panel_heights = [
        (total_h * 0.40) as u32,
        (total_h * 0.20) as u32,
        (total_h * 0.20) as u32,
        (total_h * 0.20) as u32,
    ];
    let mut panel_areas = Vec::with_capacity(4);
    let mut y_offset = 0;
    for h in &panel_heights {
        panel_areas.push(root.clone().shrink((0u32, y_offset), (COMBINED_WIDTH, *h)));
        y_offset += h;
    }

    let structure_max = residual_points
        .iter()
        .chain(drift_points.iter())
        .chain(slew_points.iter())
        .map(|(_, value)| value.abs())
        .fold(1.2_f64, f64::max)
        .max(1.2);
    let shared_x_labels = window_end.saturating_sub(window_start).max(2).min(8);

    // Panel (a): Continuous signal layer.
    let mut structure_chart = ChartBuilder::on(&panel_areas[0])
        .caption(
            format!(
                "(a) Normalized signals \u{2014} {} (runs {}\u{2013}{}, failure at {})",
                feature_name,
                window_start,
                window_end.saturating_sub(1),
                drsc_window.failure_run_index,
            ),
            ("sans-serif", 36),
        )
        .margin(22)
        .x_label_area_size(14)
        .y_label_area_size(100)
        .build_cartesian_2d(window_start..window_end, -structure_max..structure_max)
        .map_err(plot_error)?;
    structure_chart
        .configure_mesh()
        .disable_x_mesh()
        .light_line_style(WHITE)
        .x_labels(shared_x_labels)
        .y_desc("r/\u{03c1},  d/d\u{209c},  s/s\u{209c}")
        .label_style(("sans-serif", 26))
        .draw()
        .map_err(plot_error)?;
    // Zero baseline.
    structure_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(window_start, 0.0), (window_end, 0.0)],
            BLACK.mix(0.20).stroke_width(1),
        )))
        .map_err(plot_error)?;
    // Residual: solid black.
    structure_chart
        .draw_series(LineSeries::new(
            residual_points.iter().copied(),
            BLACK.stroke_width(3),
        ))
        .map_err(plot_error)?
        .label("residual / \u{03c1}")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 28, y)], BLACK.stroke_width(3)));
    // Drift: dashed mid-gray (drawn as skip-segment pairs for plotters compatibility).
    structure_chart
        .draw_series(
            drift_points
                .windows(2)
                .enumerate()
                .filter(|(index, _)| index % 2 == 0)
                .map(|(_, segment)| {
                    PathElement::new(
                        vec![(segment[0].0, segment[0].1), (segment[1].0, segment[1].1)],
                        RGBColor(80, 80, 80).stroke_width(3),
                    )
                }),
        )
        .map_err(plot_error)?
        .label("drift / d\u{209c}")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 10, y), (x + 18, y), (x + 28, y)],
                RGBColor(80, 80, 80).stroke_width(3),
            )
        });
    // Slew: light-gray dotted line + circle markers.
    structure_chart
        .draw_series(LineSeries::new(
            slew_points.iter().copied(),
            RGBColor(150, 150, 150).stroke_width(1),
        ))
        .map_err(plot_error)?;
    structure_chart
        .draw_series(rows.iter().map(|row| {
            Circle::new(
                (row.run_index, row.slew_over_threshold),
                6,
                RGBColor(150, 150, 150).filled(),
            )
        }))
        .map_err(plot_error)?
        .label("slew / s\u{209c}")
        .legend(|(x, y)| Circle::new((x + 14, y), 5, RGBColor(150, 150, 150).filled()));
    structure_chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(WHITE.mix(0.94))
        .border_style(BLACK)
        .label_font(("sans-serif", 24))
        .draw()
        .map_err(plot_error)?;

    // Panel (b): Deterministic DSFB state band.
    let mut state_chart = ChartBuilder::on(&panel_areas[1])
        .caption(
            "(b) DSFB state (Admissible / Boundary / Violation)",
            ("sans-serif", 32),
        )
        .margin(22)
        .x_label_area_size(14)
        .y_label_area_size(100)
        .build_cartesian_2d(window_start..window_end, 0.0f64..1.0f64)
        .map_err(plot_error)?;
    state_chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(shared_x_labels)
        .y_labels(0)
        .label_style(("sans-serif", 26))
        .draw()
        .map_err(plot_error)?;
    for row in rows {
        state_chart
            .draw_series(std::iter::once(Rectangle::new(
                [(row.run_index, 0.0), (row.run_index + 1, 1.0)],
                state_shade(row.display_state).filled(),
            )))
            .map_err(plot_error)?;
    }

    // Panel (c): DSA precursor activation.
    let mut dsa_chart = ChartBuilder::on(&panel_areas[2])
        .caption(
            "(c) DSA precursor (top: feature alert, bottom: corroborated run-level)",
            ("sans-serif", 32),
        )
        .margin(22)
        .x_label_area_size(14)
        .y_label_area_size(100)
        .build_cartesian_2d(window_start..window_end, 0.0f64..2.0f64)
        .map_err(plot_error)?;
    dsa_chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(shared_x_labels)
        .y_labels(0)
        .label_style(("sans-serif", 26))
        .draw()
        .map_err(plot_error)?;
    dsa_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(window_start, 1.0), (window_end, 1.0)],
            BLACK.mix(0.18).stroke_width(1),
        )))
        .map_err(plot_error)?;
    for row in rows {
        if row.run_level_dsa_alert {
            dsa_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(row.run_index, 0.12), (row.run_index + 1, 0.88)],
                    RGBColor(110, 110, 110).filled(),
                )))
                .map_err(plot_error)?;
        }
        if row.feature_dsa_alert {
            dsa_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(row.run_index, 1.12), (row.run_index + 1, 1.88)],
                    BLACK.filled(),
                )))
                .map_err(plot_error)?;
        }
    }

    // Panel (d): Scalar baseline trigger timing.
    let mut scalar_chart = ChartBuilder::on(&panel_areas[3])
        .caption(
            "(d) Scalar triggers (top: threshold, bottom: EWMA)",
            ("sans-serif", 32),
        )
        .margin(22)
        .x_label_area_size(50)
        .y_label_area_size(100)
        .build_cartesian_2d(window_start..window_end, 0.0f64..2.0f64)
        .map_err(plot_error)?;
    scalar_chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Run index")
        .y_labels(0)
        .label_style(("sans-serif", 26))
        .draw()
        .map_err(plot_error)?;
    scalar_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(window_start, 1.0), (window_end, 1.0)],
            BLACK.mix(0.18).stroke_width(1),
        )))
        .map_err(plot_error)?;
    for row in rows {
        if row.threshold_run_signal {
            scalar_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(row.run_index, 1.12), (row.run_index + 1, 1.88)],
                    RGBColor(64, 64, 64).filled(),
                )))
                .map_err(plot_error)?;
        }
        if row.ewma_run_signal {
            scalar_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(row.run_index, 0.12), (row.run_index + 1, 0.88)],
                    RGBColor(160, 160, 160).filled(),
                )))
                .map_err(plot_error)?;
        }
    }

    // Failure marker across all panels (dashed segments for grayscale visibility).
    for (chart, y_min, y_max) in [
        (&mut structure_chart, -structure_max, structure_max),
        (&mut state_chart, 0.0, 1.0),
        (&mut dsa_chart, 0.0, 2.0),
        (&mut scalar_chart, 0.0, 2.0),
    ] {
        draw_failure_marker_dashed(chart, drsc_window.failure_run_index, y_min, y_max)?;
    }

    // Annotations.
    if let Some(candidate) = boundary_filtered_annotation(rows) {
        annotate_chart(
            &mut state_chart,
            candidate.run_index,
            0.7,
            candidate.run_index.saturating_sub(6).max(window_start + 1),
            0.86,
            &candidate.label,
        )?;
    }
    if let Some(candidate) = precursor_annotation(rows) {
        annotate_chart(
            &mut dsa_chart,
            candidate.run_index,
            0.5,
            (candidate.run_index + 2).min(window_end.saturating_sub(1)),
            0.30,
            &candidate.label,
        )?;
    }
    if let Some(candidate) = scalar_annotation(rows) {
        annotate_chart(
            &mut scalar_chart,
            candidate.run_index,
            1.5,
            (candidate.run_index + 3).min(window_end.saturating_sub(1)),
            1.74,
            &candidate.label,
        )?;
    }

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
    dsa: &DsaEvaluation,
    feature_index: usize,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let feature = &nominal.features[feature_index];
    let residual_trace = &residuals.traces[feature_index];
    let sign_trace = &signs.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let grammar_trace = &grammar.traces[feature_index];
    let dsa_trace = &dsa.traces[feature_index];
    let residual_scale = positive_or_one(feature.rho);
    let drift_scale = positive_or_one(sign_trace.drift_threshold);
    let slew_scale = positive_or_one(sign_trace.slew_threshold);
    let ewma_scale = positive_or_one(ewma_trace.threshold);
    let run_energy_scale = positive_or_one(baselines.run_energy.threshold);
    let pca_t2_scale = positive_or_one(baselines.pca_fdc.t2_threshold);
    let pca_spe_scale = positive_or_one(baselines.pca_fdc.spe_threshold);

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
        "run_energy",
        "run_energy_over_threshold",
        "pca_t2",
        "pca_t2_over_threshold",
        "pca_spe",
        "pca_spe_over_threshold",
        "threshold_alarm",
        "ewma_alarm",
        "run_energy_alarm",
        "pca_fdc_alarm",
        "raw_state",
        "confirmed_state",
        "persistent_state",
        "raw_reason",
        "confirmed_reason",
        "persistent_boundary",
        "persistent_violation",
        "boundary_density_W",
        "drift_persistence_W",
        "slew_density_W",
        "ewma_occupancy_W",
        "motif_recurrence_W",
        "consistent",
        "dsa_score",
        "dsa_active",
        "dsa_alert",
        "primary_run_signal",
        "primary_run_alert",
        "any_feature_dsa_alert",
        "any_feature_raw_violation",
        "feature_count_dsa_alert",
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
            baselines.run_energy.energy[run_index].to_string(),
            (baselines.run_energy.energy[run_index] / run_energy_scale).to_string(),
            baselines.pca_fdc.t2[run_index].to_string(),
            (baselines.pca_fdc.t2[run_index] / pca_t2_scale).to_string(),
            baselines.pca_fdc.spe[run_index].to_string(),
            (baselines.pca_fdc.spe[run_index] / pca_spe_scale).to_string(),
            residual_trace.threshold_alarm[run_index].to_string(),
            ewma_trace.alarm[run_index].to_string(),
            baselines.run_energy.alarm[run_index].to_string(),
            baselines.pca_fdc.alarm[run_index].to_string(),
            format!("{:?}", grammar_trace.raw_states[run_index]),
            format!("{:?}", grammar_trace.states[run_index]),
            format!("{:?}", display_state(grammar_trace, run_index)),
            format!("{:?}", grammar_trace.raw_reasons[run_index]),
            format!("{:?}", grammar_trace.reasons[run_index]),
            grammar_trace.persistent_boundary[run_index].to_string(),
            grammar_trace.persistent_violation[run_index].to_string(),
            dsa_trace.boundary_density_w[run_index].to_string(),
            dsa_trace.drift_persistence_w[run_index].to_string(),
            dsa_trace.slew_density_w[run_index].to_string(),
            dsa_trace.ewma_occupancy_w[run_index].to_string(),
            dsa_trace.motif_recurrence_w[run_index].to_string(),
            dsa_trace.consistent[run_index].to_string(),
            dsa_trace.dsa_score[run_index].to_string(),
            dsa_trace.dsa_active[run_index].to_string(),
            dsa_trace.dsa_alert[run_index].to_string(),
            dsa.run_signals.primary_run_signal.clone(),
            dsa.run_signals.primary_run_alert[run_index].to_string(),
            dsa.run_signals.any_feature_dsa_alert[run_index].to_string(),
            dsa.run_signals.any_feature_raw_violation[run_index].to_string(),
            dsa.run_signals.feature_count_dsa_alert[run_index].to_string(),
            (run_index == drsc_window.failure_run_index).to_string(),
            (Some(run_index) == drsc_window.first_persistent_boundary_run).to_string(),
            (Some(run_index) == drsc_window.first_persistent_violation_run).to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn draw_dsa_focus_chart(
    output_path: &Path,
    _dataset: &PreparedDataset,
    residuals: &ResidualSet,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    dsa: &DsaEvaluation,
    feature_index: usize,
    config: &PipelineConfig,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let dsa_trace = &dsa.traces[feature_index];
    let grammar_trace = &grammar.traces[feature_index];
    let threshold_trace = &residuals.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let cusum_trace = &baselines.cusum[feature_index];
    let window_start = drsc_window.window_start;
    let window_end = drsc_window.window_end;
    let window_runs = window_end.saturating_sub(window_start);

    let root = BitMapBackend::new(output_path, (WIDTH, HEIGHT + 250)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;
    let areas = root.split_evenly((3, 1));

    let boundary_density = dsa_trace
        .boundary_density_w
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let drift_persistence = dsa_trace
        .drift_persistence_w
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let slew_density = dsa_trace
        .slew_density_w
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let ewma_occupancy = dsa_trace
        .ewma_occupancy_w
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let motif_recurrence = dsa_trace
        .motif_recurrence_w
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();
    let dsa_score = dsa_trace
        .dsa_score
        .iter()
        .skip(window_start)
        .take(window_runs)
        .copied()
        .collect::<Vec<_>>();

    let mut feature_chart = ChartBuilder::on(&areas[0])
        .caption(
            format!(
                "DSA structural features for feature {} around failure run {}",
                dsa_trace.feature_name, drsc_window.failure_run_index
            ),
            ("sans-serif", 26),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start..window_end, 0.0f64..1.05f64)
        .map_err(plot_error)?;
    feature_chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc("Rolling structural features")
        .draw()
        .map_err(plot_error)?;
    draw_named_series(
        &mut feature_chart,
        window_start,
        &boundary_density,
        BLUE,
        "boundary density",
    )?;
    draw_named_series(
        &mut feature_chart,
        window_start,
        &drift_persistence,
        GREEN,
        "drift persistence",
    )?;
    draw_named_series(
        &mut feature_chart,
        window_start,
        &slew_density,
        MAGENTA,
        "slew density",
    )?;
    draw_named_series(
        &mut feature_chart,
        window_start,
        &ewma_occupancy,
        CYAN,
        "EWMA occupancy",
    )?;
    draw_named_series(
        &mut feature_chart,
        window_start,
        &motif_recurrence,
        RGBColor(120, 70, 20),
        "motif recurrence",
    )?;
    feature_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    let score_max = dsa_score
        .iter()
        .copied()
        .fold(config.dsa.alert_tau.max(1.0), f64::max)
        .max(config.dsa.alert_tau);
    let mut score_chart = ChartBuilder::on(&areas[1])
        .caption(
            "DSA score, consistency, and persistence gate",
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(window_start..window_end, 0.0f64..(score_max * 1.15))
        .map_err(plot_error)?;
    score_chart
        .configure_mesh()
        .x_desc("Run index")
        .y_desc("DSA score")
        .draw()
        .map_err(plot_error)?;
    for run_index in window_start..window_end {
        if dsa_trace.dsa_alert[run_index] {
            score_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, 0.0), (run_index + 1, score_max * 1.15)],
                    RGBAColor(160, 0, 160, 0.10).filled(),
                )))
                .map_err(plot_error)?;
        } else if !dsa_trace.consistent[run_index] {
            score_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, 0.0), (run_index + 1, score_max * 1.15)],
                    RGBAColor(180, 180, 180, 0.10).filled(),
                )))
                .map_err(plot_error)?;
        }
    }
    draw_named_series(
        &mut score_chart,
        window_start,
        &dsa_score,
        RGBColor(160, 0, 160),
        "DSA score",
    )?;
    score_chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (window_start, config.dsa.alert_tau),
                (window_end, config.dsa.alert_tau),
            ],
            RED.mix(0.8).stroke_width(2),
        )))
        .map_err(plot_error)?
        .label("tau")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], RED.mix(0.8).stroke_width(2)));
    score_chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .map_err(plot_error)?;

    let raw_boundary_flags = grammar_trace
        .raw_states
        .iter()
        .map(|state| *state == GrammarState::Boundary)
        .collect::<Vec<_>>();
    let raw_violation_flags = grammar_trace
        .raw_states
        .iter()
        .map(|state| *state == GrammarState::Violation)
        .collect::<Vec<_>>();
    let signal_rows = [
        ("DSA alert", RGBColor(160, 0, 160), &dsa_trace.dsa_alert),
        ("raw boundary", RGBColor(255, 179, 0), &raw_boundary_flags),
        ("raw violation", RGBColor(200, 0, 0), &raw_violation_flags),
        ("threshold", RED, &threshold_trace.threshold_alarm),
        ("EWMA", GREEN, &ewma_trace.alarm),
        ("CUSUM", RGBColor(120, 70, 20), &cusum_trace.alarm),
        ("run energy", BLACK, &baselines.run_energy.alarm),
        (
            "PCA T2/SPE",
            RGBColor(80, 40, 140),
            &baselines.pca_fdc.alarm,
        ),
    ];
    let mut band_chart = ChartBuilder::on(&areas[2])
        .caption(
            "Feature-level alert band across DSA and comparators",
            ("sans-serif", 24),
        )
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(100)
        .build_cartesian_2d(window_start..window_end, 0..signal_rows.len())
        .map_err(plot_error)?;
    band_chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Run index")
        .y_labels(signal_rows.len())
        .y_label_formatter(&|idx| {
            signal_rows
                .get(*idx)
                .map(|(label, _, _)| label.to_string())
                .unwrap_or_default()
        })
        .draw()
        .map_err(plot_error)?;
    for (row_index, (_label, color, flags)) in signal_rows.iter().enumerate() {
        for run_index in window_start..window_end {
            let fill = if flags[run_index] {
                color.mix(0.75).filled()
            } else {
                WHITE.mix(0.0).filled()
            };
            band_chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(run_index, row_index), (run_index + 1, row_index + 1)],
                    fill,
                )))
                .map_err(plot_error)?;
        }
    }

    root.present().map_err(plot_error)?;
    Ok(())
}

fn write_dsa_focus_trace_csv(
    output_path: &Path,
    dataset: &PreparedDataset,
    baselines: &BaselineSet,
    grammar: &GrammarSet,
    dsa: &DsaEvaluation,
    feature_index: usize,
    drsc_window: &DrscWindow,
) -> Result<()> {
    let dsa_trace = &dsa.traces[feature_index];
    let grammar_trace = &grammar.traces[feature_index];
    let ewma_trace = &baselines.ewma[feature_index];
    let cusum_trace = &baselines.cusum[feature_index];
    let mut writer = Writer::from_path(output_path)?;
    writer.write_record([
        "run_index",
        "timestamp",
        "label",
        "feature",
        "boundary_density_W",
        "drift_persistence_W",
        "slew_density_W",
        "ewma_occupancy_W",
        "motif_recurrence_W",
        "consistent",
        "dsa_score",
        "dsa_active",
        "dsa_alert",
        "primary_run_signal",
        "primary_run_alert",
        "ewma_alarm",
        "cusum_alarm",
        "run_energy",
        "run_energy_over_threshold",
        "run_energy_alarm",
        "pca_t2",
        "pca_t2_over_threshold",
        "pca_spe",
        "pca_spe_over_threshold",
        "pca_fdc_alarm",
        "raw_state",
        "persistent_boundary",
        "persistent_violation",
    ])?;
    for run_index in drsc_window.window_start..drsc_window.window_end {
        writer.write_record([
            run_index.to_string(),
            dataset.timestamps[run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            dataset.labels[run_index].to_string(),
            dsa_trace.feature_name.clone(),
            dsa_trace.boundary_density_w[run_index].to_string(),
            dsa_trace.drift_persistence_w[run_index].to_string(),
            dsa_trace.slew_density_w[run_index].to_string(),
            dsa_trace.ewma_occupancy_w[run_index].to_string(),
            dsa_trace.motif_recurrence_w[run_index].to_string(),
            dsa_trace.consistent[run_index].to_string(),
            dsa_trace.dsa_score[run_index].to_string(),
            dsa_trace.dsa_active[run_index].to_string(),
            dsa_trace.dsa_alert[run_index].to_string(),
            dsa.run_signals.primary_run_signal.clone(),
            dsa.run_signals.primary_run_alert[run_index].to_string(),
            ewma_trace.alarm[run_index].to_string(),
            cusum_trace.alarm[run_index].to_string(),
            baselines.run_energy.energy[run_index].to_string(),
            (baselines.run_energy.energy[run_index]
                / positive_or_one(baselines.run_energy.threshold))
            .to_string(),
            baselines.run_energy.alarm[run_index].to_string(),
            baselines.pca_fdc.t2[run_index].to_string(),
            (baselines.pca_fdc.t2[run_index] / positive_or_one(baselines.pca_fdc.t2_threshold))
                .to_string(),
            baselines.pca_fdc.spe[run_index].to_string(),
            (baselines.pca_fdc.spe[run_index] / positive_or_one(baselines.pca_fdc.spe_threshold))
                .to_string(),
            baselines.pca_fdc.alarm[run_index].to_string(),
            format!("{:?}", grammar_trace.raw_states[run_index]),
            grammar_trace.persistent_boundary[run_index].to_string(),
            grammar_trace.persistent_violation[run_index].to_string(),
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
    let first_persistent_boundary_run =
        (window_start..failure_run_index).find(|&run_index| trace.persistent_boundary[run_index]);
    let first_persistent_violation_run =
        (window_start..failure_run_index).find(|&run_index| trace.persistent_violation[run_index]);

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

fn run_level_threshold_signal(residuals: &ResidualSet) -> Vec<bool> {
    let run_count = residuals
        .traces
        .first()
        .map(|trace| trace.threshold_alarm.len())
        .unwrap_or(0);
    (0..run_count)
        .map(|run_index| {
            residuals
                .traces
                .iter()
                .any(|trace| trace.threshold_alarm[run_index])
        })
        .collect()
}

fn run_level_ewma_signal(baselines: &BaselineSet) -> Vec<bool> {
    let run_count = baselines
        .ewma
        .first()
        .map(|trace| trace.alarm.len())
        .unwrap_or(0);
    (0..run_count)
        .map(|run_index| baselines.ewma.iter().any(|trace| trace.alarm[run_index]))
        .collect()
}

fn state_shade(state: GrammarState) -> RGBColor {
    match state {
        GrammarState::Admissible => RGBColor(234, 234, 234),
        GrammarState::Boundary => RGBColor(148, 148, 148),
        GrammarState::Violation => RGBColor(36, 36, 36),
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

fn boundary_filtered_annotation(rows: &[DrscDsaCombinedRow]) -> Option<AnnotationCandidate> {
    rows.iter()
        .find(|row| row.persistent_boundary && !row.feature_dsa_alert && !row.run_level_dsa_alert)
        .map(|row| AnnotationCandidate {
            run_index: row.run_index,
            label: "Boundary activity filtered".into(),
        })
}

fn precursor_annotation(rows: &[DrscDsaCombinedRow]) -> Option<AnnotationCandidate> {
    rows.iter()
        .find(|row| row.run_level_dsa_alert)
        .map(|row| AnnotationCandidate {
            run_index: row.run_index,
            label: "Persistent structural precursor".into(),
        })
}

fn scalar_annotation(rows: &[DrscDsaCombinedRow]) -> Option<AnnotationCandidate> {
    rows.iter()
        .find(|row| row.threshold_run_signal || row.ewma_run_signal)
        .map(|row| AnnotationCandidate {
            run_index: row.run_index,
            label: "Scalar trigger".into(),
        })
}

fn draw_failure_marker_dashed<DB: DrawingBackend>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordusize, RangedCoordf64>>,
    run_index: usize,
    y_min: f64,
    y_max: f64,
) -> Result<()> {
    // Draw a segmented vertical line to simulate dashing in grayscale.
    let segments = 8;
    let span = y_max - y_min;
    let seg_len = span / (2 * segments) as f64;
    for i in 0..segments {
        let lo = y_min + (2 * i) as f64 * seg_len;
        let hi = lo + seg_len;
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(run_index, lo), (run_index, hi)],
                BLACK.mix(0.55).stroke_width(3),
            )))
            .map_err(plot_error)?;
    }
    Ok(())
}

fn annotate_chart<DB: DrawingBackend>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordusize, RangedCoordf64>>,
    target_x: usize,
    target_y: f64,
    text_x: usize,
    text_y: f64,
    label: &str,
) -> Result<()> {
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(target_x, target_y), (text_x, text_y)],
            BLACK.stroke_width(2),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            label.to_string(),
            (text_x, text_y),
            ("sans-serif", 28).into_font(),
        )))
        .map_err(plot_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_annotation_candidates_are_detected_deterministically() {
        let rows = vec![
            DrscDsaCombinedRow {
                run_index: 5,
                timestamp: "2008-01-01 00:00:00".into(),
                label: -1,
                feature: "S000".into(),
                residual_over_rho: 0.0,
                drift_over_threshold: 0.0,
                slew_over_threshold: 0.0,
                display_state: GrammarState::Admissible,
                persistent_boundary: false,
                persistent_violation: false,
                feature_dsa_alert: false,
                run_level_dsa_alert: false,
                feature_count_dsa_alert: 0,
                threshold_run_signal: true,
                ewma_run_signal: false,
            },
            DrscDsaCombinedRow {
                run_index: 6,
                timestamp: "2008-01-01 00:10:00".into(),
                label: -1,
                feature: "S000".into(),
                residual_over_rho: 0.4,
                drift_over_threshold: 0.5,
                slew_over_threshold: 0.2,
                display_state: GrammarState::Boundary,
                persistent_boundary: true,
                persistent_violation: false,
                feature_dsa_alert: false,
                run_level_dsa_alert: false,
                feature_count_dsa_alert: 0,
                threshold_run_signal: false,
                ewma_run_signal: false,
            },
            DrscDsaCombinedRow {
                run_index: 7,
                timestamp: "2008-01-01 00:20:00".into(),
                label: -1,
                feature: "S000".into(),
                residual_over_rho: 0.6,
                drift_over_threshold: 0.7,
                slew_over_threshold: 0.3,
                display_state: GrammarState::Boundary,
                persistent_boundary: true,
                persistent_violation: false,
                feature_dsa_alert: true,
                run_level_dsa_alert: true,
                feature_count_dsa_alert: 3,
                threshold_run_signal: false,
                ewma_run_signal: false,
            },
        ];

        assert_eq!(boundary_filtered_annotation(&rows).unwrap().run_index, 6);
        assert_eq!(precursor_annotation(&rows).unwrap().run_index, 7);
        assert_eq!(scalar_annotation(&rows).unwrap().run_index, 5);
    }

    #[test]
    fn combined_render_rejects_empty_rows() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("drsc_dsa_combined.png");
        let err = draw_drsc_dsa_combined_chart(
            &path,
            &[],
            &DrscWindow {
                failure_run_index: 0,
                window_start: 0,
                window_end: 0,
                first_persistent_boundary_run: None,
                first_persistent_violation_run: None,
            },
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("combined DRSC+DSA figure requires at least one row"));
    }
}
