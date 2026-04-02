use crate::error::{DsfbSemiconductorError, Result};
use plotters::coord::Shift;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const UNIFIED_FIGURE_WIDTH: u32 = 1920;
const UNIFIED_FIGURE_HEIGHT: u32 = 1080;
const EXEC_SUMMARY_MARKDOWN_START: &str = "<!-- UNIFIED_VALUE_EXEC_SUMMARY_START -->";
const EXEC_SUMMARY_MARKDOWN_END: &str = "<!-- UNIFIED_VALUE_EXEC_SUMMARY_END -->";
const SECTION_MARKDOWN_START: &str = "<!-- UNIFIED_VALUE_SECTION_START -->";
const SECTION_MARKDOWN_END: &str = "<!-- UNIFIED_VALUE_SECTION_END -->";
const EXEC_SUMMARY_TEX_START: &str = "% UNIFIED_VALUE_EXEC_SUMMARY_START";
const EXEC_SUMMARY_TEX_END: &str = "% UNIFIED_VALUE_EXEC_SUMMARY_END";
const SECTION_TEX_START: &str = "% UNIFIED_VALUE_SECTION_START";
const SECTION_TEX_END: &str = "% UNIFIED_VALUE_SECTION_END";

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedValueFigureArtifacts {
    pub secom_run_dir: PathBuf,
    pub phm_run_dir: Option<PathBuf>,
    pub figure_path: PathBuf,
    pub csv_path: PathBuf,
    pub caption: String,
    pub phm_panel_available: bool,
    pub report_markdown_updated: bool,
    pub report_tex_updated: bool,
    pub paper_updated: bool,
}

#[derive(Debug, Clone)]
struct SecomFigureMetrics {
    baseline_investigation_points: usize,
    optimized_review_escalate_points: usize,
    delta_investigation_load: f64,
    baseline_episode_count: usize,
    optimized_episode_count: usize,
    delta_episode_count: f64,
    recall: usize,
    failure_runs: usize,
    dsfb_episode_count: usize,
    dsfb_pre_failure_episode_count: usize,
    episode_precision: f64,
    raw_boundary_precision: f64,
    raw_alarm_count: usize,
    precision_gain_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct PhmLeadTimeRow {
    run_id: String,
    dsfb_detection_time: Option<i64>,
    threshold_detection_time: Option<i64>,
    lead_time_delta: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct PhmEarlyWarningStats {
    threshold_baseline: String,
    total_runs: usize,
    comparable_runs: usize,
    mean_lead_delta: Option<f64>,
    median_lead_delta: Option<f64>,
    percent_runs_dsfb_earlier: f64,
    percent_runs_equal: f64,
    percent_runs_later: f64,
}

#[derive(Debug, Clone)]
struct PhmComparableLeadRow {
    run_id: String,
    dsfb_detection_time: i64,
    threshold_detection_time: i64,
    lead_time_delta: i64,
}

#[derive(Debug, Clone)]
struct PhmFigureMetrics {
    threshold_baseline: String,
    mean_dsfb_detection_time: f64,
    mean_threshold_detection_time: f64,
    mean_lead_delta: f64,
    median_lead_delta: f64,
    percent_runs_dsfb_earlier: f64,
    percent_runs_equal: f64,
    percent_runs_later: f64,
    comparable_earlier_runs: usize,
    comparable_equal_runs: usize,
    comparable_later_runs: usize,
    comparable_runs: usize,
    total_runs: usize,
    comparable_rows: Vec<PhmComparableLeadRow>,
}

#[derive(Debug, Clone, Serialize)]
struct UnifiedValueFigureCsvRow {
    panel: String,
    metric: String,
    item_label: String,
    baseline_label: String,
    dsfb_value: Option<f64>,
    baseline_value: Option<f64>,
    delta_value: Option<f64>,
    units: String,
    source_artifact: String,
    note: String,
}

pub fn render_unified_value_figure(
    secom_run_dir: &Path,
    phm_run_dir: Option<&Path>,
    paper_tex_path: Option<&Path>,
) -> Result<UnifiedValueFigureArtifacts> {
    let secom_metrics = load_secom_metrics(secom_run_dir)?;
    let phm_metrics = match phm_run_dir {
        Some(path) => load_phm_metrics(path)?,
        None => None,
    };

    let figure_dir = secom_run_dir.join("figures");
    fs::create_dir_all(&figure_dir)?;
    let figure_path = figure_dir.join("dsfb_unified_value_figure.png");
    let csv_path = secom_run_dir.join("dsfb_unified_value_figure.csv");
    let caption = unified_caption(&secom_metrics, phm_metrics.as_ref());

    draw_unified_value_figure(&figure_path, &secom_metrics, phm_metrics.as_ref())?;
    write_unified_value_csv(&csv_path, &secom_metrics, phm_metrics.as_ref())?;
    update_report_files(
        secom_run_dir,
        &figure_path,
        &caption,
        &secom_metrics,
        phm_metrics.as_ref(),
    )?;

    let paper_updated = match (phm_metrics.as_ref(), paper_tex_path) {
        (Some(_), Some(paper_tex_path)) => {
            sync_paper_figure_asset(&figure_path, paper_tex_path)?;
            update_paper_tex(paper_tex_path, &caption)?;
            true
        }
        _ => false,
    };

    Ok(UnifiedValueFigureArtifacts {
        secom_run_dir: secom_run_dir.to_path_buf(),
        phm_run_dir: phm_run_dir
            .filter(|path| phm_metrics.is_some() && path.exists())
            .map(Path::to_path_buf),
        figure_path,
        csv_path,
        caption,
        phm_panel_available: phm_metrics.is_some(),
        report_markdown_updated: secom_run_dir.join("engineering_report.md").exists(),
        report_tex_updated: secom_run_dir.join("engineering_report.tex").exists(),
        paper_updated,
    })
}

pub fn resolve_latest_completed_run(
    root: &Path,
    suffix: &str,
    required_file: &str,
) -> Option<PathBuf> {
    let mut candidates = fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("dsfb-semiconductor") && name.ends_with(suffix))
        })
        .filter(|path| path.join(required_file).exists())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn load_secom_metrics(run_dir: &Path) -> Result<SecomFigureMetrics> {
    let operator_targets = read_json(&run_dir.join("dsa_operator_delta_targets.json"))?;
    let episode_precision = read_json(&run_dir.join("episode_precision_metrics.json"))?;

    Ok(SecomFigureMetrics {
        baseline_investigation_points: required_usize(
            &operator_targets,
            &["baseline_investigation_points"],
        )?,
        optimized_review_escalate_points: required_usize(
            &operator_targets,
            &["optimized_review_escalate_points"],
        )?,
        delta_investigation_load: required_f64(&operator_targets, &["delta_investigation_load"])?,
        baseline_episode_count: required_usize(&operator_targets, &["baseline_episode_count"])?,
        optimized_episode_count: required_usize(&operator_targets, &["optimized_episode_count"])?,
        delta_episode_count: required_f64(&operator_targets, &["delta_episode_count"])?,
        recall: required_usize(
            &operator_targets,
            &["selected_configuration", "failure_recall"],
        )?,
        failure_runs: required_usize(
            &operator_targets,
            &["selected_configuration", "failure_runs"],
        )?,
        dsfb_episode_count: required_usize(&episode_precision, &["dsfb_episode_count"])?,
        dsfb_pre_failure_episode_count: required_usize(
            &episode_precision,
            &["dsfb_pre_failure_episode_count"],
        )?,
        episode_precision: required_f64(&episode_precision, &["dsfb_precision"])?,
        raw_boundary_precision: required_f64(&episode_precision, &["raw_alarm_precision"])?,
        raw_alarm_count: required_usize(&episode_precision, &["raw_alarm_count"])?,
        precision_gain_factor: required_f64(&episode_precision, &["precision_gain_factor"])?,
    })
}

fn load_phm_metrics(run_dir: &Path) -> Result<Option<PhmFigureMetrics>> {
    let stats_path = run_dir.join("phm2018_early_warning_stats.json");
    let lead_path = run_dir.join("phm2018_lead_time_metrics.csv");
    if !stats_path.exists() || !lead_path.exists() {
        return Ok(None);
    }

    let stats: PhmEarlyWarningStats = read_json(&stats_path)?;
    let rows = read_csv::<PhmLeadTimeRow>(&lead_path)?;
    let mut comparable_rows = rows
        .iter()
        .filter_map(|row| {
            Some(PhmComparableLeadRow {
                run_id: row.run_id.clone(),
                dsfb_detection_time: row.dsfb_detection_time?,
                threshold_detection_time: row.threshold_detection_time?,
                lead_time_delta: row.lead_time_delta?,
            })
        })
        .collect::<Vec<_>>();
    if comparable_rows.is_empty() {
        return Ok(None);
    }
    comparable_rows.sort_by(|left, right| {
        left.lead_time_delta
            .cmp(&right.lead_time_delta)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    let mean_dsfb_detection_time = comparable_rows
        .iter()
        .map(|row| row.dsfb_detection_time as f64)
        .sum::<f64>()
        / comparable_rows.len() as f64;
    let mean_threshold_detection_time = comparable_rows
        .iter()
        .map(|row| row.threshold_detection_time as f64)
        .sum::<f64>()
        / comparable_rows.len() as f64;
    let comparable_earlier_runs = comparable_rows
        .iter()
        .filter(|row| row.lead_time_delta > 0)
        .count();
    let comparable_equal_runs = comparable_rows
        .iter()
        .filter(|row| row.lead_time_delta == 0)
        .count();
    let comparable_later_runs =
        comparable_rows.len() - comparable_earlier_runs - comparable_equal_runs;

    Ok(Some(PhmFigureMetrics {
        threshold_baseline: stats.threshold_baseline,
        mean_dsfb_detection_time,
        mean_threshold_detection_time,
        mean_lead_delta: stats.mean_lead_delta.unwrap_or_default(),
        median_lead_delta: stats.median_lead_delta.unwrap_or_default(),
        percent_runs_dsfb_earlier: stats.percent_runs_dsfb_earlier,
        percent_runs_equal: stats.percent_runs_equal,
        percent_runs_later: stats.percent_runs_later,
        comparable_earlier_runs,
        comparable_equal_runs,
        comparable_later_runs,
        comparable_runs: stats.comparable_runs,
        total_runs: stats.total_runs,
        comparable_rows,
    }))
}

fn draw_unified_value_figure(
    output_path: &Path,
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (UNIFIED_FIGURE_WIDTH, UNIFIED_FIGURE_HEIGHT))
        .into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;

    let areas = root.split_evenly((1, 3));
    draw_secom_burden_panel(&areas[0], secom)?;
    draw_secom_precision_panel(&areas[1], secom)?;
    draw_phm_panel(&areas[2], phm)?;
    root.present().map_err(plot_error)?;
    Ok(())
}

fn draw_secom_burden_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    secom: &SecomFigureMetrics,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let (header, rest) = area.split_vertically(150);
    let (body, footer) = rest.split_vertically(610);
    let body_panels = body.split_evenly((2, 1));

    draw_panel_heading(
        &header,
        "A. SECOM burden compression",
        &[
            "SECOM is used only for structural compression and operator burden.",
            "Baselines are named explicitly: numeric-only DSA points and raw boundary episodes.",
        ],
    )?;
    draw_horizontal_pair_subchart(
        &body_panels[0],
        "Investigation-worthy points",
        "Numeric-only DSA baseline",
        secom.baseline_investigation_points as f64,
        "Policy-governed Review/Escalate",
        secom.optimized_review_escalate_points as f64,
    )?;
    draw_horizontal_pair_subchart(
        &body_panels[1],
        "Episode count",
        "Raw boundary episode basis",
        secom.baseline_episode_count as f64,
        "Optimized DSA episodes",
        secom.optimized_episode_count as f64,
    )?;
    draw_footer_lines(
        &footer,
        &[
            format!(
                "{:.1}% investigation-load reduction vs numeric-only DSA ({} -> {})",
                secom.delta_investigation_load * 100.0,
                format_count(secom.baseline_investigation_points),
                format_count(secom.optimized_review_escalate_points),
            ),
            format!(
                "{:.1}% episode reduction vs raw boundary episodes ({} -> {})",
                secom.delta_episode_count * 100.0,
                format_count(secom.baseline_episode_count),
                format_count(secom.optimized_episode_count),
            ),
            format!(
                "Bounded failure coverage preserved at {}/{} labeled failures",
                secom.recall, secom.failure_runs
            ),
        ],
    )?;
    Ok(())
}

fn draw_secom_precision_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    secom: &SecomFigureMetrics,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let (header, rest) = area.split_vertically(150);
    let (body, footer) = rest.split_vertically(610);
    draw_panel_heading(
        &header,
        "B. SECOM episode precision",
        &[
            "The SECOM precision panel is operator-facing: fewer episodes, higher relevance.",
            "Raw boundary precision is shown only as the low-precision structural basis.",
        ],
    )?;
    draw_precision_subchart(&body, secom)?;
    draw_footer_lines(
        &footer,
        &[
            format!(
                "{:.1}% of DSA episodes precede labeled failures ({} of {})",
                secom.episode_precision * 100.0,
                secom.dsfb_pre_failure_episode_count,
                secom.dsfb_episode_count,
            ),
            format!(
                "Raw boundary precision proxy: {:.2}% across {} raw structural episodes",
                secom.raw_boundary_precision * 100.0,
                format_count(secom.raw_alarm_count),
            ),
            format!(
                "Precision gain vs raw boundary precision proxy: {:.1}x",
                secom.precision_gain_factor,
            ),
        ],
    )?;
    Ok(())
}

fn draw_phm_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    phm: Option<&PhmFigureMetrics>,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    match phm {
        Some(phm) => {
            let (header, rest) = area.split_vertically(170);
            let (body, footer) = rest.split_vertically(610);
            draw_panel_heading(
                &header,
                "C. PHM 2018 lead-time comparison",
                &[
                    "PHM 2018 is the early-warning benchmark.",
                    "SECOM does not support this claim by itself.",
                ],
            )?;
            draw_phm_delta_subchart(&body, phm)?;
            draw_footer_lines(
                &footer,
                &[
                    format!(
                        "Baseline: {} after the healthy calibration prefix",
                        phm.threshold_baseline
                    ),
                    format!(
                        "Mean detection time: baseline {}, DSFB {}",
                        format_compact_value(phm.mean_threshold_detection_time),
                        format_compact_value(phm.mean_dsfb_detection_time),
                    ),
                    format!(
                        "Mean delta {}, median {}",
                        format_signed_compact_value(phm.mean_lead_delta),
                        format_signed_compact_value(phm.median_lead_delta),
                    ),
                    format!(
                        "Comparable runs: {} of {} total, with {} earlier, {} equal, {} later",
                        phm.comparable_runs,
                        phm.total_runs,
                        phm.comparable_earlier_runs,
                        phm.comparable_equal_runs,
                        phm.comparable_later_runs,
                    ),
                    format!(
                        "Saved {}-run split: {:.1}% earlier, {:.1}% equal, {:.1}% later or unavailable",
                        phm.total_runs,
                        phm.percent_runs_dsfb_earlier * 100.0,
                        phm.percent_runs_equal * 100.0,
                        phm.percent_runs_later * 100.0,
                    ),
                ],
            )?;
        }
        None => {
            let (header, rest) = area.split_vertically(170);
            let (body, footer) = rest.split_vertically(610);
            draw_panel_heading(
                &header,
                "C. PHM 2018 lead-time comparison",
                &[
                    "PHM 2018 is the early-warning benchmark.",
                    "Panel C is intentionally incomplete when no PHM artifacts are available.",
                ],
            )?;
            body.draw(&Rectangle::new(
                [(36, 36), (560, 540)],
                ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(2),
            ))
            .map_err(plot_error)?;
            draw_footer_lines(
                &footer,
                &[
                    "No completed PHM 2018 lead-time summary was found in the supplied run."
                        .into(),
                    "Expected artifacts: phm2018_lead_time_metrics.csv and phm2018_early_warning_stats.json."
                        .into(),
                    "The figure therefore does not claim early-warning value from SECOM alone."
                        .into(),
                ],
            )?;
        }
    }
    Ok(())
}

fn draw_panel_heading(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    title: &str,
    subtitle_lines: &[&str],
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    area.draw(&Text::new(
        title.to_string(),
        (24, 28),
        ("sans-serif", 30).into_font().style(FontStyle::Bold),
    ))
    .map_err(plot_error)?;
    for (line_index, line) in subtitle_lines.iter().enumerate() {
        area.draw(&Text::new(
            (*line).to_string(),
            (24, 72 + (line_index as i32 * 28)),
            ("sans-serif", 18).into_font().color(&BLACK),
        ))
        .map_err(plot_error)?;
    }
    Ok(())
}

fn draw_footer_lines(area: &DrawingArea<BitMapBackend<'_>, Shift>, lines: &[String]) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    for (line_index, line) in lines.iter().enumerate() {
        area.draw(&Text::new(
            line.clone(),
            (24, 28 + (line_index as i32 * 28)),
            ("sans-serif", 18).into_font().color(&BLACK),
        ))
        .map_err(plot_error)?;
    }
    Ok(())
}

fn draw_horizontal_pair_subchart(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    x_desc: &str,
    baseline_label: &str,
    baseline_value: f64,
    dsfb_label: &str,
    dsfb_value: f64,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let max_value = baseline_value.max(dsfb_value).max(1.0);
    let label_margin = max_value * 0.45;
    let mut chart = ChartBuilder::on(area)
        .margin(10)
        .x_label_area_size(34)
        .y_label_area_size(0)
        .build_cartesian_2d(-label_margin..(max_value * 1.12), 0f64..2f64)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_y_mesh()
        .disable_x_mesh()
        .y_labels(0)
        .x_labels(4)
        .x_desc(x_desc)
        .x_label_formatter(&|value| {
            if *value < 0.0 {
                String::new()
            } else {
                format_compact_value(*value)
            }
        })
        .label_style(("sans-serif", 16))
        .draw()
        .map_err(plot_error)?;

    let baseline_style = RGBColor(195, 195, 195).filled();
    let dsfb_style = RGBColor(45, 45, 45).filled();
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, 1.12), (baseline_value, 1.72)],
            baseline_style,
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, 0.28), (dsfb_value, 0.88)],
            dsfb_style,
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            baseline_label.to_string(),
            (-label_margin * 0.98, 1.42),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            dsfb_label.to_string(),
            (-label_margin * 0.98, 0.58),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format_count_f64(baseline_value),
            (baseline_value + max_value * 0.02, 1.42),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format_count_f64(dsfb_value),
            (dsfb_value + max_value * 0.02, 0.58),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    Ok(())
}

fn draw_precision_subchart(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    secom: &SecomFigureMetrics,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let label_margin = 40.0;
    let raw_percent = secom.raw_boundary_precision * 100.0;
    let dsfb_percent = secom.episode_precision * 100.0;
    let mut chart = ChartBuilder::on(area)
        .margin(10)
        .x_label_area_size(34)
        .y_label_area_size(0)
        .build_cartesian_2d(-label_margin..100f64, 0f64..2f64)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_y_mesh()
        .disable_x_mesh()
        .y_labels(0)
        .x_labels(5)
        .x_desc("episode precision (%)")
        .label_style(("sans-serif", 16))
        .draw()
        .map_err(plot_error)?;

    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, 1.12), (raw_percent, 1.72)],
            RGBColor(195, 195, 195).filled(),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, 0.28), (dsfb_percent, 0.88)],
            RGBColor(45, 45, 45).filled(),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            "Raw boundary precision proxy".to_string(),
            (-label_margin * 0.98, 1.42),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            "Optimized DSA episode precision".to_string(),
            (-label_margin * 0.98, 0.58),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format!("{raw_percent:.2}%"),
            (raw_percent.max(1.8) + 1.5, 1.42),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format!("{dsfb_percent:.1}%"),
            ((dsfb_percent + 2.0).min(95.0), 0.58),
            ("sans-serif", 17).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    Ok(())
}

fn draw_phm_delta_subchart(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    phm: &PhmFigureMetrics,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let span = phm
        .comparable_rows
        .iter()
        .map(|row| row.lead_time_delta.unsigned_abs() as f64)
        .fold(phm.mean_lead_delta.abs(), f64::max)
        .max(1.0);
    let axis_limit = span * 1.18;
    let row_count = phm.comparable_rows.len() as i32;
    let mut chart = ChartBuilder::on(area)
        .margin(10)
        .x_label_area_size(42)
        .y_label_area_size(90)
        .build_cartesian_2d(-axis_limit..axis_limit, 0i32..row_count)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .y_labels(phm.comparable_rows.len())
        .y_label_formatter(&|index| {
            phm.comparable_rows
                .get(*index as usize)
                .map(|row| row.run_id.clone())
                .unwrap_or_default()
        })
        .x_desc("threshold baseline minus DSFB detection time")
        .x_label_formatter(&|value| format_signed_compact_value(*value))
        .label_style(("sans-serif", 15))
        .draw()
        .map_err(plot_error)?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0), (0.0, row_count)],
            ShapeStyle::from(&BLACK.mix(0.6)).stroke_width(1),
        )))
        .map_err(plot_error)?;

    for (row_index, row) in phm.comparable_rows.iter().enumerate() {
        let y = row_index as i32;
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(0.0, y), (row.lead_time_delta as f64, y)],
                ShapeStyle::from(&RGBColor(150, 150, 150)).stroke_width(3),
            )))
            .map_err(plot_error)?;

        let point_style = match row.lead_time_delta.cmp(&0) {
            Ordering::Greater => ShapeStyle::from(&BLACK).filled(),
            Ordering::Equal => ShapeStyle::from(&BLACK.mix(0.7)).stroke_width(2),
            Ordering::Less => ShapeStyle::from(&RGBColor(110, 110, 110)).filled(),
        };
        chart
            .draw_series(std::iter::once(Circle::new(
                (row.lead_time_delta as f64, y),
                6,
                point_style,
            )))
            .map_err(plot_error)?;
    }
    Ok(())
}

fn write_unified_value_csv(
    csv_path: &Path,
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> Result<()> {
    let mut rows = vec![
        UnifiedValueFigureCsvRow {
            panel: "A".into(),
            metric: "investigation_points".into(),
            item_label: "summary".into(),
            baseline_label: "numeric_only_dsa".into(),
            dsfb_value: Some(secom.optimized_review_escalate_points as f64),
            baseline_value: Some(secom.baseline_investigation_points as f64),
            delta_value: Some(secom.delta_investigation_load),
            units: "count".into(),
            source_artifact: "dsa_operator_delta_targets.json".into(),
            note: "SECOM burden compression against numeric-only DSA investigation-worthy points"
                .into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "A".into(),
            metric: "episode_count".into(),
            item_label: "summary".into(),
            baseline_label: "raw_boundary".into(),
            dsfb_value: Some(secom.optimized_episode_count as f64),
            baseline_value: Some(secom.baseline_episode_count as f64),
            delta_value: Some(secom.delta_episode_count),
            units: "count".into(),
            source_artifact: "dsa_operator_delta_targets.json".into(),
            note: "SECOM episode compression against raw boundary episode count".into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "B".into(),
            metric: "episode_precision".into(),
            item_label: "summary".into(),
            baseline_label: "raw_boundary_precision_proxy".into(),
            dsfb_value: Some(secom.episode_precision),
            baseline_value: Some(secom.raw_boundary_precision),
            delta_value: Some(secom.precision_gain_factor),
            units: "fraction".into(),
            source_artifact: "episode_precision_metrics.json".into(),
            note: "delta_value is the precision gain factor versus the raw boundary basis".into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "B".into(),
            metric: "failure_linked_episode_count".into(),
            item_label: "summary".into(),
            baseline_label: "all_dsa_episodes".into(),
            dsfb_value: Some(secom.dsfb_pre_failure_episode_count as f64),
            baseline_value: Some(secom.dsfb_episode_count as f64),
            delta_value: None,
            units: "count".into(),
            source_artifact: "episode_precision_metrics.json".into(),
            note: "Failure-linked DSA episode count versus all DSA episodes".into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "B".into(),
            metric: "recall".into(),
            item_label: "summary".into(),
            baseline_label: "labeled_failure_runs".into(),
            dsfb_value: Some(secom.recall as f64),
            baseline_value: Some(secom.failure_runs as f64),
            delta_value: None,
            units: "count".into(),
            source_artifact: "dsa_operator_delta_targets.json".into(),
            note: "Bounded SECOM failure coverage shown alongside burden compression".into(),
        },
    ];

    match phm {
        Some(phm) => {
            rows.push(UnifiedValueFigureCsvRow {
                panel: "C".into(),
                metric: "mean_detection_time".into(),
                item_label: "summary".into(),
                baseline_label: phm.threshold_baseline.clone(),
                dsfb_value: Some(phm.mean_dsfb_detection_time),
                baseline_value: Some(phm.mean_threshold_detection_time),
                delta_value: Some(phm.mean_lead_delta),
                units: "time_index".into(),
                source_artifact: "phm2018_lead_time_metrics.csv; phm2018_early_warning_stats.json"
                    .into(),
                note: "delta_value is threshold-baseline detection time minus DSFB detection time"
                    .into(),
            });
            rows.push(UnifiedValueFigureCsvRow {
                panel: "C".into(),
                metric: "all_run_split".into(),
                item_label: "summary".into(),
                baseline_label: phm.threshold_baseline.clone(),
                dsfb_value: Some(phm.percent_runs_dsfb_earlier),
                baseline_value: Some(phm.percent_runs_later),
                delta_value: Some(phm.percent_runs_equal),
                units: "fraction".into(),
                source_artifact: "phm2018_early_warning_stats.json".into(),
                note: format!(
                    "dsfb_value stores the earlier fraction across all {} PHM runs, baseline_value stores the later-or-unavailable fraction, and delta_value stores the equal fraction",
                    phm.total_runs
                ),
            });
            rows.push(UnifiedValueFigureCsvRow {
                panel: "C".into(),
                metric: "comparable_run_split".into(),
                item_label: "summary".into(),
                baseline_label: phm.threshold_baseline.clone(),
                dsfb_value: Some(phm.comparable_earlier_runs as f64),
                baseline_value: Some(phm.comparable_later_runs as f64),
                delta_value: Some(phm.comparable_equal_runs as f64),
                units: "count".into(),
                source_artifact: "phm2018_lead_time_metrics.csv".into(),
                note: "Comparable-run counts: dsfb_value is earlier, baseline_value is later, delta_value is equal".into(),
            });
            for row in &phm.comparable_rows {
                rows.push(UnifiedValueFigureCsvRow {
                    panel: "C".into(),
                    metric: "lead_time_delta_by_run".into(),
                    item_label: row.run_id.clone(),
                    baseline_label: phm.threshold_baseline.clone(),
                    dsfb_value: Some(row.dsfb_detection_time as f64),
                    baseline_value: Some(row.threshold_detection_time as f64),
                    delta_value: Some(row.lead_time_delta as f64),
                    units: "time_index".into(),
                    source_artifact: "phm2018_lead_time_metrics.csv".into(),
                    note: "delta_value is threshold-baseline detection time minus DSFB detection time for this comparable PHM run".into(),
                });
            }
        }
        None => rows.push(UnifiedValueFigureCsvRow {
            panel: "C".into(),
            metric: "phm_panel_status".into(),
            item_label: "summary".into(),
            baseline_label: "no_completed_phm_artifact".into(),
            dsfb_value: None,
            baseline_value: None,
            delta_value: None,
            units: "n/a".into(),
            source_artifact: "not_available".into(),
            note:
                "No completed PHM 2018 lead-time artifact was available; Panel C is a placeholder."
                    .into(),
        }),
    }

    let mut writer = csv::Writer::from_path(csv_path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn update_report_files(
    secom_run_dir: &Path,
    figure_path: &Path,
    caption: &str,
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> Result<()> {
    let report_md_path = secom_run_dir.join("engineering_report.md");
    let report_tex_path = secom_run_dir.join("engineering_report.tex");

    if report_md_path.exists() {
        let original = fs::read_to_string(&report_md_path)?;
        let executive_block = build_markdown_exec_summary(secom, phm);
        let section_block = build_markdown_section(
            figure_path
                .strip_prefix(secom_run_dir)
                .unwrap_or(figure_path)
                .to_string_lossy()
                .replace('\\', "/")
                .as_str(),
            caption,
            secom,
            phm,
        );
        let updated = insert_markdown_exec_summary(&original, &executive_block);
        let updated = insert_markdown_section(&updated, &section_block);
        fs::write(&report_md_path, updated)?;
    }

    if report_tex_path.exists() {
        let original = fs::read_to_string(&report_tex_path)?;
        let executive_block = build_tex_exec_summary(secom, phm);
        let section_block = build_tex_section(caption, secom, phm);
        let updated = insert_tex_exec_summary(&original, &executive_block);
        let updated = insert_tex_section(&updated, &section_block);
        fs::write(&report_tex_path, updated)?;
    }

    Ok(())
}

fn sync_paper_figure_asset(figure_path: &Path, paper_tex_path: &Path) -> Result<()> {
    let paper_root = paper_tex_path.parent().ok_or_else(|| {
        DsfbSemiconductorError::DatasetFormat("paper tex path has no parent directory".into())
    })?;
    let figure_dir = paper_root.join("figures");
    fs::create_dir_all(&figure_dir)?;
    fs::copy(
        figure_path,
        figure_dir.join("dsfb_unified_value_figure.png"),
    )?;
    Ok(())
}

fn update_paper_tex(paper_tex_path: &Path, caption: &str) -> Result<()> {
    let original = fs::read_to_string(paper_tex_path)?;
    let figure_block = format!(
        "% UNIFIED_VALUE_FIGURE_START\n\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.98\\linewidth]{{figures/dsfb_unified_value_figure.png}}\n\\caption{{{}}}\n\\label{{fig:dsfb-unified-value}}\n\\end{{figure}}\n% UNIFIED_VALUE_FIGURE_END\n",
        latex_escape(caption),
    );
    let updated = if original.contains("% UNIFIED_VALUE_FIGURE_START") {
        replace_between(
            &original,
            "% UNIFIED_VALUE_FIGURE_START",
            "% UNIFIED_VALUE_FIGURE_END",
            &figure_block,
        )
    } else if let Some(index) = original.find("\\subsection{Closing Statement}") {
        let mut updated = String::new();
        updated.push_str(&original[..index]);
        updated.push_str(&figure_block);
        updated.push_str(&original[index..]);
        updated
    } else if let Some(index) = original.rfind("\\end{document}") {
        let mut updated = String::new();
        updated.push_str(&original[..index]);
        updated.push_str(&figure_block);
        updated.push_str(&original[index..]);
        updated
    } else {
        format!("{original}\n{figure_block}")
    };
    fs::write(paper_tex_path, updated)?;
    Ok(())
}

fn unified_caption(secom: &SecomFigureMetrics, phm: Option<&PhmFigureMetrics>) -> String {
    match phm {
        Some(phm) => format!(
            "Unified DSFB value figure. Left: on SECOM, the policy-governed DSFB layer reduces investigation-worthy alert burden from {} numeric-only DSA points to {} Review/Escalate points and compresses raw boundary episodes from {} to {} while preserving bounded failure coverage at {}/{}. Middle: the same SECOM run raises episode precision to {:.1}% versus a raw boundary precision proxy of {:.2}% ({:.1}x), improving operator selectivity without claiming SECOM early-warning superiority. Right: on PHM 2018, DSFB is compared only against the {} baseline after the healthy calibration prefix; the per-run lead-time picture remains mixed, but the mean baseline-minus-DSFB detection gap is {}. The figure therefore separates SECOM structural-compression value from bounded PHM early-warning evidence rather than forcing one dataset to support both claims.",
            format_count(secom.baseline_investigation_points),
            format_count(secom.optimized_review_escalate_points),
            format_count(secom.baseline_episode_count),
            format_count(secom.optimized_episode_count),
            secom.recall,
            secom.failure_runs,
            secom.episode_precision * 100.0,
            secom.raw_boundary_precision * 100.0,
            secom.precision_gain_factor,
            phm.threshold_baseline,
            format_signed_compact_value(phm.mean_lead_delta),
        ),
        None => format!(
            "Unified DSFB value figure. Left: on SECOM, the policy-governed DSFB layer reduces investigation-worthy alert burden from {} numeric-only DSA points to {} Review/Escalate points and compresses raw boundary episodes from {} to {} while preserving bounded failure coverage at {}/{}. Middle: the same SECOM run raises episode precision to {:.1}% versus a raw boundary precision proxy of {:.2}% ({:.1}x), improving operator selectivity without claiming SECOM early-warning superiority. Right: the PHM 2018 panel is intentionally marked unavailable because no completed PHM lead-time artifact exists in the supplied outputs, so the figure does not claim early-warning value from SECOM alone.",
            format_count(secom.baseline_investigation_points),
            format_count(secom.optimized_review_escalate_points),
            format_count(secom.baseline_episode_count),
            format_count(secom.optimized_episode_count),
            secom.recall,
            secom.failure_runs,
            secom.episode_precision * 100.0,
            secom.raw_boundary_precision * 100.0,
            secom.precision_gain_factor,
        ),
    }
}

fn build_markdown_exec_summary(
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> String {
    let phm_line = match phm {
        Some(phm) => format!(
            "- PHM 2018 lead-time result vs `{}`: mean baseline-minus-DSFB detection gap `{}`, comparable-run split `{}/{}/{}` (earlier/equal/later), saved all-run split `{:.1}%/{:.1}%/{:.1}%`\n",
            phm.threshold_baseline,
            format_signed_compact_value(phm.mean_lead_delta),
            phm.comparable_earlier_runs,
            phm.comparable_equal_runs,
            phm.comparable_later_runs,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0,
        ),
        None => "- PHM 2018 lead-time result: unavailable in current saved artifacts; the unified figure marks Panel C as intentionally incomplete\n".into(),
    };
    format!(
        "{EXEC_SUMMARY_MARKDOWN_START}\n- Investigation-load reduction vs numeric-only DSA baseline: {:.1}% ({} -> {})\n- Episode reduction vs raw boundary baseline: {:.1}% ({} -> {})\n- Episode precision vs raw boundary precision proxy: {:.1}% vs {:.2}%, {:.1}x gain\n{}{EXEC_SUMMARY_MARKDOWN_END}\n\n",
        secom.delta_investigation_load * 100.0,
        secom.baseline_investigation_points,
        secom.optimized_review_escalate_points,
        secom.delta_episode_count * 100.0,
        secom.baseline_episode_count,
        secom.optimized_episode_count,
        secom.episode_precision * 100.0,
        secom.raw_boundary_precision * 100.0,
        secom.precision_gain_factor,
        phm_line,
    )
}

fn build_tex_exec_summary(secom: &SecomFigureMetrics, phm: Option<&PhmFigureMetrics>) -> String {
    let phm_line = match phm {
        Some(phm) => format!(
            "\\item PHM 2018 lead-time result vs \\texttt{{{}}}: mean baseline-minus-DSFB detection gap {}, comparable-run split {}/{}/{} (earlier/equal/later), saved all-run split {:.1}\\%/{:.1}\\%/{:.1}\\%.",
            latex_escape(&phm.threshold_baseline),
            latex_escape(&format_signed_compact_value(phm.mean_lead_delta)),
            phm.comparable_earlier_runs,
            phm.comparable_equal_runs,
            phm.comparable_later_runs,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0
        ),
        None => "\\item PHM 2018 lead-time result: unavailable in the current saved artifacts, so Panel C is intentionally incomplete.".into(),
    };
    format!(
        "{EXEC_SUMMARY_TEX_START}\n\\begin{{itemize}}\n\\item Investigation-load reduction vs numeric-only DSA baseline: {:.1}\\% ({} to {}).\n\\item Episode reduction vs raw boundary baseline: {:.1}\\% ({} to {}).\n\\item Episode precision vs raw boundary precision proxy: {:.1}\\% vs {:.2}\\%, {:.1}x gain.\n{}\n\\end{{itemize}}\n{EXEC_SUMMARY_TEX_END}\n\n",
        secom.delta_investigation_load * 100.0,
        secom.baseline_investigation_points,
        secom.optimized_review_escalate_points,
        secom.delta_episode_count * 100.0,
        secom.baseline_episode_count,
        secom.optimized_episode_count,
        secom.episode_precision * 100.0,
        secom.raw_boundary_precision * 100.0,
        secom.precision_gain_factor,
        phm_line,
    )
}

fn build_markdown_section(
    figure_rel_path: &str,
    caption: &str,
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> String {
    let phm_text = match phm {
        Some(phm) => format!(
            "- PHM 2018 is used only for lead time. Panel C compares DSFB against the `{}` baseline after the healthy calibration prefix: mean baseline-minus-DSFB delta `{}`, median `{}`, comparable-run split `{}/{}/{}` (earlier/equal/later), saved all-run split `{:.1}%/{:.1}%/{:.1}%`.\n",
            phm.threshold_baseline,
            format_signed_compact_value(phm.mean_lead_delta),
            format_signed_compact_value(phm.median_lead_delta),
            phm.comparable_earlier_runs,
            phm.comparable_equal_runs,
            phm.comparable_later_runs,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0,
        ),
        None => "- PHM 2018 lead-time artifacts are not yet completed in the current crate-local outputs, so Panel C is a placeholder and the figure does not claim early-warning value from SECOM alone.\n".into(),
    };
    format!(
        "{SECTION_MARKDOWN_START}\n## Unified Structural Compression and Degradation Value\n\n![Unified DSFB value figure]({figure_rel_path})\n\n{caption}\n\n- SECOM burden compression uses the numeric-only DSA investigation baseline `{}` and the raw boundary episode baseline `{}`.\n- The SECOM operator result shown here is bounded: policy-governed Review/Escalate points fall from `{}` to `{}`, DSA episodes fall from `{}` to `{}`, and bounded failure coverage remains at `{}/{} `.\n- Episode precision is promoted as the primary SECOM operator metric: `{:.1}%` versus raw boundary precision proxy `{:.2}%`, a `{:.1}x` gain.\n{}- This section keeps the claims separated: SECOM supports burden compression and precision, while PHM 2018 provides bounded degradation-oriented timing evidence only.\n\n{SECTION_MARKDOWN_END}\n\n",
        "numeric_only_dsa",
        "raw_boundary",
        format_count(secom.baseline_investigation_points),
        format_count(secom.optimized_review_escalate_points),
        format_count(secom.baseline_episode_count),
        format_count(secom.optimized_episode_count),
        secom.recall,
        secom.failure_runs,
        secom.episode_precision * 100.0,
        secom.raw_boundary_precision * 100.0,
        secom.precision_gain_factor,
        phm_text,
    )
}

fn build_tex_section(
    caption: &str,
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> String {
    let phm_text = match phm {
        Some(phm) => format!(
            "PHM 2018 is used only for lead time. Panel C compares DSFB against the \\texttt{{{}}} baseline after the healthy calibration prefix: mean baseline-minus-DSFB delta {}, median {}, comparable-run split {}/{}/{} (earlier/equal/later), saved all-run split {:.1}\\%/{:.1}\\%/{:.1}\\%.",
            latex_escape(&phm.threshold_baseline),
            latex_escape(&format_signed_compact_value(phm.mean_lead_delta)),
            latex_escape(&format_signed_compact_value(phm.median_lead_delta)),
            phm.comparable_earlier_runs,
            phm.comparable_equal_runs,
            phm.comparable_later_runs,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0,
        ),
        None => "PHM 2018 lead-time artifacts are not yet completed in the current crate-local outputs, so Panel C is a placeholder and this figure does not claim early-warning value from SECOM alone.".into(),
    };
    format!(
        "{SECTION_TEX_START}\n\\section*{{Unified Structural Compression and Degradation Value}}\n\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.98\\linewidth]{{figures/dsfb_unified_value_figure.png}}\n\\caption{{{}}}\n\\end{{figure}}\n{}\n\nThe SECOM burden-compression panel uses the numeric-only DSA investigation baseline \\texttt{{numeric\\_only\\_dsa}} and the raw boundary episode baseline \\texttt{{raw\\_boundary}}. The bounded SECOM result shown here reduces policy-governed Review/Escalate points from {} to {}, reduces DSA episodes from {} to {}, and preserves bounded failure coverage at {}/{}. Episode precision is promoted as the primary SECOM operator metric: {:.1}\\% versus raw boundary precision proxy {:.2}\\%, a {:.1}x gain.\n\n{SECTION_TEX_END}\n\n",
        latex_escape(caption),
        phm_text,
        format_count(secom.baseline_investigation_points),
        format_count(secom.optimized_review_escalate_points),
        format_count(secom.baseline_episode_count),
        format_count(secom.optimized_episode_count),
        secom.recall,
        secom.failure_runs,
        secom.episode_precision * 100.0,
        secom.raw_boundary_precision * 100.0,
        secom.precision_gain_factor,
    )
}

fn insert_markdown_exec_summary(content: &str, block: &str) -> String {
    if content.contains(EXEC_SUMMARY_MARKDOWN_START) {
        return replace_between(
            content,
            EXEC_SUMMARY_MARKDOWN_START,
            EXEC_SUMMARY_MARKDOWN_END,
            block,
        );
    }
    let marker = "## Executive Summary\n\n";
    if let Some(index) = content.find(marker) {
        let insert_at = index + marker.len();
        let mut updated = String::new();
        updated.push_str(&content[..insert_at]);
        updated.push_str(block);
        updated.push_str(&content[insert_at..]);
        updated
    } else {
        format!("{block}{content}")
    }
}

fn insert_markdown_section(content: &str, block: &str) -> String {
    if content.contains(SECTION_MARKDOWN_START) {
        return replace_between(content, SECTION_MARKDOWN_START, SECTION_MARKDOWN_END, block);
    }
    let marker = "## Dataset";
    if let Some(index) = content.find(marker) {
        let mut updated = String::new();
        updated.push_str(&content[..index]);
        updated.push_str(block);
        updated.push_str(&content[index..]);
        updated
    } else {
        format!("{content}\n\n{block}")
    }
}

fn insert_tex_exec_summary(content: &str, block: &str) -> String {
    if content.contains(EXEC_SUMMARY_TEX_START) {
        return replace_between(content, EXEC_SUMMARY_TEX_START, EXEC_SUMMARY_TEX_END, block);
    }
    let marker = "\\section*{Executive summary}\n";
    if let Some(index) = content.find(marker) {
        let insert_at = index + marker.len();
        let mut updated = String::new();
        updated.push_str(&content[..insert_at]);
        updated.push_str(block);
        updated.push_str(&content[insert_at..]);
        updated
    } else {
        format!("{block}{content}")
    }
}

fn insert_tex_section(content: &str, block: &str) -> String {
    if content.contains(SECTION_TEX_START) {
        return replace_between(content, SECTION_TEX_START, SECTION_TEX_END, block);
    }
    let marker = "\\section*{Dataset}\n";
    if let Some(index) = content.find(marker) {
        let mut updated = String::new();
        updated.push_str(&content[..index]);
        updated.push_str(block);
        updated.push_str(&content[index..]);
        updated
    } else {
        format!("{content}\n\n{block}")
    }
}

fn replace_between(
    content: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> String {
    let start = content.find(start_marker).unwrap_or(0);
    let end = content
        .find(end_marker)
        .map(|index| index + end_marker.len())
        .unwrap_or(start);
    let mut updated = String::new();
    updated.push_str(&content[..start]);
    updated.push_str(replacement);
    updated.push_str(&content[end..]);
    updated
}

fn required_f64(json: &Value, path: &[&str]) -> Result<f64> {
    json_path(json, path)
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            DsfbSemiconductorError::DatasetFormat(format!("missing numeric JSON path {:?}", path))
        })
}

fn required_usize(json: &Value, path: &[&str]) -> Result<usize> {
    json_path(json, path)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| {
            DsfbSemiconductorError::DatasetFormat(format!("missing integer JSON path {:?}", path))
        })
}

fn json_path<'a>(json: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = json;
    for part in path {
        current = current.get(*part)?;
    }
    Some(current)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let file = fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

fn read_csv<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut rows = Vec::new();
    for row in reader.deserialize() {
        rows.push(row?);
    }
    Ok(rows)
}

fn plot_error<E: std::fmt::Display>(err: E) -> DsfbSemiconductorError {
    DsfbSemiconductorError::ExternalCommand(err.to_string())
}

fn format_count(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn format_count_f64(value: f64) -> String {
    format_count(value.round() as usize)
}

fn format_compact_value(value: f64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if abs >= 10_000.0 {
        format!("{:.1}k", value / 1_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.2}k", value / 1_000.0)
    } else {
        format!("{value:.0}")
    }
}

fn format_signed_compact_value(value: f64) -> String {
    if value > 0.0 {
        format!("+{}", format_compact_value(value))
    } else if value < 0.0 {
        format!("-{}", format_compact_value(value.abs()))
    } else {
        "0".into()
    }
}

fn latex_escape(input: &str) -> String {
    input
        .replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_secom_fixture(run_dir: &Path) {
        fs::create_dir_all(run_dir.join("figures")).unwrap();
        fs::write(
            run_dir.join("dsa_operator_delta_targets.json"),
            serde_json::json!({
                "selected_configuration": {
                    "failure_recall": 104,
                    "failure_runs": 104
                },
                "baseline_investigation_points": 10554,
                "optimized_review_escalate_points": 3854,
                "delta_investigation_load": 0.6348303960583664,
                "baseline_episode_count": 28607,
                "optimized_episode_count": 71,
                "delta_episode_count": 0.9975180899779774
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            run_dir.join("episode_precision_metrics.json"),
            serde_json::json!({
                "dsfb_episode_count": 71,
                "dsfb_pre_failure_episode_count": 57,
                "dsfb_precision": 0.8028169014084507,
                "raw_alarm_count": 28607,
                "raw_alarm_precision": 0.003635473835075331,
                "precision_gain_factor": 220.82868364030338
            })
            .to_string(),
        )
        .unwrap();
        fs::write(run_dir.join("engineering_report.md"), "# DSFB Semiconductor Engineering Report\n\n## Executive Summary\n\nExisting summary.\n\n## Dataset\n\nBody.\n").unwrap();
        fs::write(run_dir.join("engineering_report.tex"), "\\documentclass{article}\n\\begin{document}\n\\section*{Executive summary}\nExisting summary.\n\\section*{Dataset}\nBody.\n\\end{document}\n").unwrap();
    }

    fn write_phm_fixture(run_dir: &Path) {
        fs::create_dir_all(run_dir).unwrap();
        fs::write(
            run_dir.join("phm2018_early_warning_stats.json"),
            serde_json::json!({
                "threshold_baseline": "run_energy_scalar_threshold",
                "total_runs": 4,
                "comparable_runs": 4,
                "mean_lead_delta": 5.5,
                "median_lead_delta": 4.5,
                "percent_runs_dsfb_earlier": 0.75,
                "percent_runs_equal": 0.25,
                "percent_runs_later": 0.0
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            run_dir.join("phm2018_lead_time_metrics.csv"),
            "run_id,dsfb_detection_time,threshold_detection_time,lead_time_delta\nr1,10,16,6\nr2,11,16,5\nr3,12,16,4\nr4,16,16,0\n",
        )
        .unwrap();
    }

    #[test]
    fn secom_figure_metrics_load_from_saved_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let secom = temp.path().join("secom");
        write_secom_fixture(&secom);

        let metrics = load_secom_metrics(&secom).unwrap();
        assert_eq!(metrics.baseline_investigation_points, 10554);
        assert_eq!(metrics.optimized_review_escalate_points, 3854);
        assert_eq!(metrics.baseline_episode_count, 28607);
        assert_eq!(metrics.optimized_episode_count, 71);
        assert_eq!(metrics.dsfb_episode_count, 71);
        assert_eq!(metrics.dsfb_pre_failure_episode_count, 57);
        assert!((metrics.episode_precision - 0.8028169014084507).abs() < 1.0e-12);
        assert!((metrics.raw_boundary_precision - 0.003635473835075331).abs() < 1.0e-12);
    }

    #[test]
    fn phm_figure_metrics_load_from_saved_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let phm = temp.path().join("phm");
        write_phm_fixture(&phm);

        let metrics = load_phm_metrics(&phm).unwrap().unwrap();
        assert_eq!(metrics.threshold_baseline, "run_energy_scalar_threshold");
        assert_eq!(metrics.comparable_runs, 4);
        assert_eq!(metrics.total_runs, 4);
        assert_eq!(metrics.comparable_earlier_runs, 3);
        assert_eq!(metrics.comparable_equal_runs, 1);
        assert_eq!(metrics.comparable_later_runs, 0);
        assert_eq!(metrics.comparable_rows.len(), 4);
        assert_eq!(metrics.comparable_rows[0].run_id, "r4");
        assert_eq!(metrics.comparable_rows[0].lead_time_delta, 0);
        assert_eq!(metrics.comparable_rows[3].run_id, "r1");
        assert_eq!(metrics.comparable_rows[3].lead_time_delta, 6);
    }

    #[test]
    fn unified_figure_generation_is_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let secom_a = temp.path().join("secom_a");
        let secom_b = temp.path().join("secom_b");
        let phm = temp.path().join("phm");
        write_secom_fixture(&secom_a);
        write_secom_fixture(&secom_b);
        write_phm_fixture(&phm);

        let first = render_unified_value_figure(&secom_a, Some(&phm), None).unwrap();
        let second = render_unified_value_figure(&secom_b, Some(&phm), None).unwrap();

        assert_eq!(
            fs::read(&first.figure_path).unwrap(),
            fs::read(&second.figure_path).unwrap()
        );
        assert_eq!(
            fs::read_to_string(&first.csv_path).unwrap(),
            fs::read_to_string(&second.csv_path).unwrap()
        );
    }

    #[test]
    fn unified_figure_companion_csv_contains_secom_and_phm_rows() {
        let temp = tempfile::tempdir().unwrap();
        let secom = temp.path().join("secom");
        let phm = temp.path().join("phm");
        write_secom_fixture(&secom);
        write_phm_fixture(&phm);

        let artifacts = render_unified_value_figure(&secom, Some(&phm), None).unwrap();
        let csv = fs::read_to_string(&artifacts.csv_path).unwrap();
        assert!(csv.contains("A,investigation_points,summary,numeric_only_dsa"));
        assert!(csv.contains("B,episode_precision,summary,raw_boundary_precision_proxy"));
        assert!(csv.contains("C,lead_time_delta_by_run,r1,run_energy_scalar_threshold"));
        assert!(csv.contains("C,comparable_run_split,summary,run_energy_scalar_threshold"));
    }

    #[test]
    fn unified_figure_gracefully_degrades_without_phm_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let secom = temp.path().join("secom");
        write_secom_fixture(&secom);

        let artifacts = render_unified_value_figure(&secom, None, None).unwrap();
        assert!(artifacts.figure_path.exists());
        assert!(artifacts.csv_path.exists());
        assert!(!artifacts.phm_panel_available);
        let csv = fs::read_to_string(&artifacts.csv_path).unwrap();
        assert!(csv.contains("phm_panel_status"));
        let report = fs::read_to_string(secom.join("engineering_report.md")).unwrap();
        assert!(report.contains("## Unified Structural Compression and Degradation Value"));
        assert!(report.contains("Panel C is a placeholder"));
    }

    #[test]
    fn unified_figure_updates_paper_and_copies_asset() {
        let temp = tempfile::tempdir().unwrap();
        let secom = temp.path().join("secom");
        let phm = temp.path().join("phm");
        let paper_dir = temp.path().join("paper");
        write_secom_fixture(&secom);
        write_phm_fixture(&phm);
        fs::create_dir_all(paper_dir.join("figures")).unwrap();
        fs::write(
            paper_dir.join("semiconductor.tex"),
            "\\documentclass{article}\n\\begin{document}\n\\subsection{Closing Statement}\nBody.\n\\end{document}\n",
        )
        .unwrap();

        let artifacts = render_unified_value_figure(
            &secom,
            Some(&phm),
            Some(&paper_dir.join("semiconductor.tex")),
        )
        .unwrap();

        assert!(artifacts.paper_updated);
        assert!(paper_dir
            .join("figures/dsfb_unified_value_figure.png")
            .exists());
        let tex = fs::read_to_string(paper_dir.join("semiconductor.tex")).unwrap();
        assert!(tex.contains("\\label{fig:dsfb-unified-value}"));
        assert!(tex.contains("figures/dsfb_unified_value_figure.png"));
    }
}
