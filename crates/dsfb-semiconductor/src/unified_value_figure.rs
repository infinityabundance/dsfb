use crate::error::{DsfbSemiconductorError, Result};
use plotters::coord::Shift;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const UNIFIED_FIGURE_WIDTH: u32 = 1800;
const UNIFIED_FIGURE_HEIGHT: u32 = 1012;
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
    episode_precision: f64,
    raw_boundary_precision: f64,
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
    total_runs: usize,
    comparable_runs: usize,
    mean_lead_delta: Option<f64>,
    median_lead_delta: Option<f64>,
    percent_runs_dsfb_earlier: f64,
    percent_runs_equal: f64,
    percent_runs_later: f64,
}

#[derive(Debug, Clone)]
struct PhmFigureMetrics {
    mean_dsfb_detection_time: f64,
    mean_threshold_detection_time: f64,
    mean_lead_delta: f64,
    median_lead_delta: f64,
    percent_runs_dsfb_earlier: f64,
    percent_runs_equal: f64,
    percent_runs_later: f64,
    comparable_runs: usize,
    total_runs: usize,
}

#[derive(Debug, Clone, Serialize)]
struct UnifiedValueFigureCsvRow {
    panel: String,
    metric: String,
    baseline_label: String,
    dsfb_value: Option<f64>,
    baseline_value: Option<f64>,
    delta_value: Option<f64>,
    units: String,
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
    let caption = unified_caption(phm_metrics.is_some());

    draw_unified_value_figure(&figure_path, &secom_metrics, phm_metrics.as_ref())?;
    write_unified_value_csv(&csv_path, &secom_metrics, phm_metrics.as_ref())?;
    update_report_files(
        secom_run_dir,
        &figure_path,
        &caption,
        &secom_metrics,
        phm_metrics.as_ref(),
    )?;

    let paper_updated = if phm_metrics.is_some() {
        if let Some(paper_tex_path) = paper_tex_path {
            update_paper_tex(paper_tex_path, &caption)?;
            true
        } else {
            false
        }
    } else {
        false
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
        episode_precision: required_f64(&episode_precision, &["dsfb_precision"])?,
        raw_boundary_precision: required_f64(&episode_precision, &["raw_alarm_precision"])?,
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
    let comparable_rows = rows
        .iter()
        .filter(|row| row.dsfb_detection_time.is_some() && row.threshold_detection_time.is_some())
        .collect::<Vec<_>>();
    if comparable_rows.is_empty() {
        return Ok(None);
    }

    let mean_dsfb_detection_time = comparable_rows
        .iter()
        .filter_map(|row| row.dsfb_detection_time.map(|value| value as f64))
        .sum::<f64>()
        / comparable_rows.len() as f64;
    let mean_threshold_detection_time = comparable_rows
        .iter()
        .filter_map(|row| row.threshold_detection_time.map(|value| value as f64))
        .sum::<f64>()
        / comparable_rows.len() as f64;

    Ok(Some(PhmFigureMetrics {
        mean_dsfb_detection_time,
        mean_threshold_detection_time,
        mean_lead_delta: stats.mean_lead_delta.unwrap_or_default(),
        median_lead_delta: stats.median_lead_delta.unwrap_or_default(),
        percent_runs_dsfb_earlier: stats.percent_runs_dsfb_earlier,
        percent_runs_equal: stats.percent_runs_equal,
        percent_runs_later: stats.percent_runs_later,
        comparable_runs: stats.comparable_runs,
        total_runs: stats.total_runs,
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
    let y_max = (secom.baseline_episode_count as f64 * 1.10)
        .max(secom.baseline_investigation_points as f64 * 2.0);
    let mut chart = ChartBuilder::on(area)
        .margin(24)
        .caption("A. SECOM burden compression", ("sans-serif", 28))
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(0f64..5f64, 0f64..y_max)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .y_desc("count")
        .x_labels(0)
        .draw()
        .map_err(plot_error)?;

    let baseline_style = RGBColor(200, 200, 200).filled();
    let dsfb_style = RGBColor(60, 60, 60).filled();

    let bars = [
        (
            0.7,
            1.5,
            secom.baseline_investigation_points as f64,
            baseline_style,
            "Numeric-only DSA\ninvestigation points",
        ),
        (
            1.7,
            2.5,
            secom.optimized_review_escalate_points as f64,
            dsfb_style,
            "Policy DSA\nReview/Escalate",
        ),
        (
            3.0,
            3.8,
            secom.baseline_episode_count as f64,
            baseline_style,
            "Raw boundary\nepisodes",
        ),
        (
            4.0,
            4.8,
            secom.optimized_episode_count as f64,
            dsfb_style,
            "Optimized DSA\nepisodes",
        ),
    ];
    for (left, right, height, style, label) in bars {
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(left, 0.0), (right, height)],
                style,
            )))
            .map_err(plot_error)?;
        chart
            .draw_series(std::iter::once(Text::new(
                format!("{height:.0}"),
                ((left + right) / 2.0, height + y_max * 0.02),
                ("sans-serif", 18).into_font().color(&BLACK),
            )))
            .map_err(plot_error)?;
        chart
            .draw_series(std::iter::once(Text::new(
                label.to_string(),
                ((left + right) / 2.0, -y_max * 0.05),
                ("sans-serif", 18).into_font().color(&BLACK),
            )))
            .map_err(plot_error)?;
    }

    area.draw(&Text::new(
        format!(
            "{:.1}% investigation-load reduction vs numeric-only DSA baseline",
            secom.delta_investigation_load * 100.0
        ),
        (32, 64),
        ("sans-serif", 20).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    area.draw(&Text::new(
        format!(
            "{:.1}% episode reduction vs raw boundary baseline",
            secom.delta_episode_count * 100.0
        ),
        (32, 92),
        ("sans-serif", 20).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    area.draw(&Text::new(
        format!(
            "Bounded failure coverage preserved at {}/{}",
            secom.recall, secom.failure_runs
        ),
        (32, 120),
        ("sans-serif", 18).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    Ok(())
}

fn draw_secom_precision_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    secom: &SecomFigureMetrics,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    let max_percent = (secom.episode_precision * 100.0 * 1.15).max(85.0);
    let mut chart = ChartBuilder::on(area)
        .margin(24)
        .caption("B. SECOM episode precision", ("sans-serif", 28))
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(0f64..3f64, 0f64..max_percent)
        .map_err(plot_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .y_desc("precision (%)")
        .x_labels(0)
        .draw()
        .map_err(plot_error)?;

    let raw_percent = secom.raw_boundary_precision * 100.0;
    let dsfb_percent = secom.episode_precision * 100.0;
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.6, 0.0), (1.3, raw_percent)],
            RGBColor(200, 200, 200).filled(),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(1.7, 0.0), (2.4, dsfb_percent)],
            RGBColor(60, 60, 60).filled(),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format!("{raw_percent:.2}%"),
            (0.95, raw_percent + max_percent * 0.03),
            ("sans-serif", 18).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            format!("{dsfb_percent:.1}%"),
            (2.05, dsfb_percent + max_percent * 0.03),
            ("sans-serif", 18).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            "Raw boundary\nprecision proxy".to_string(),
            (0.95, -max_percent * 0.08),
            ("sans-serif", 18).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;
    chart
        .draw_series(std::iter::once(Text::new(
            "Optimized DSA\nepisode precision".to_string(),
            (2.05, -max_percent * 0.08),
            ("sans-serif", 18).into_font().color(&BLACK),
        )))
        .map_err(plot_error)?;

    area.draw(&Text::new(
        format!(
            "{:.1}% of DSA episodes precede labeled failures",
            dsfb_percent
        ),
        (32, 64),
        ("sans-serif", 20).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    area.draw(&Text::new(
        format!(
            "Precision gain vs raw boundary basis: {:.1}x",
            secom.precision_gain_factor
        ),
        (32, 92),
        ("sans-serif", 20).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    area.draw(&Text::new(
        "Few episodes, higher relevance".to_string(),
        (32, 120),
        ("sans-serif", 18).into_font().color(&BLACK),
    ))
    .map_err(plot_error)?;
    Ok(())
}

fn draw_phm_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    phm: Option<&PhmFigureMetrics>,
) -> Result<()> {
    area.fill(&WHITE).map_err(plot_error)?;
    match phm {
        Some(phm) => {
            let max_y = (phm
                .mean_threshold_detection_time
                .max(phm.mean_dsfb_detection_time)
                * 1.15)
                .max(1.0);
            let mut chart = ChartBuilder::on(area)
                .margin(24)
                .caption("C. PHM 2018 lead-time advantage", ("sans-serif", 28))
                .x_label_area_size(50)
                .y_label_area_size(70)
                .build_cartesian_2d(0f64..3f64, 0f64..max_y)
                .map_err(plot_error)?;

            chart
                .configure_mesh()
                .disable_mesh()
                .y_desc("mean detection time")
                .x_labels(0)
                .draw()
                .map_err(plot_error)?;

            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(0.6, 0.0), (1.3, phm.mean_threshold_detection_time)],
                    RGBColor(200, 200, 200).filled(),
                )))
                .map_err(plot_error)?;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(1.7, 0.0), (2.4, phm.mean_dsfb_detection_time)],
                    RGBColor(60, 60, 60).filled(),
                )))
                .map_err(plot_error)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("{:.1}", phm.mean_threshold_detection_time),
                    (0.95, phm.mean_threshold_detection_time + max_y * 0.03),
                    ("sans-serif", 18).into_font().color(&BLACK),
                )))
                .map_err(plot_error)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    format!("{:.1}", phm.mean_dsfb_detection_time),
                    (2.05, phm.mean_dsfb_detection_time + max_y * 0.03),
                    ("sans-serif", 18).into_font().color(&BLACK),
                )))
                .map_err(plot_error)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    "Threshold".to_string(),
                    (0.95, -max_y * 0.08),
                    ("sans-serif", 18).into_font().color(&BLACK),
                )))
                .map_err(plot_error)?;
            chart
                .draw_series(std::iter::once(Text::new(
                    "DSFB".to_string(),
                    (2.05, -max_y * 0.08),
                    ("sans-serif", 18).into_font().color(&BLACK),
                )))
                .map_err(plot_error)?;

            area.draw(&Text::new(
                "PHM 2018 is the early-warning benchmark".to_string(),
                (32, 64),
                ("sans-serif", 20).into_font().color(&BLACK),
            ))
            .map_err(plot_error)?;
            area.draw(&Text::new(
                "SECOM does not support this claim by itself".to_string(),
                (32, 92),
                ("sans-serif", 18).into_font().color(&BLACK),
            ))
            .map_err(plot_error)?;
            area.draw(&Text::new(
                format!(
                    "Mean threshold-minus-DSFB delta {:.2}; DSFB earlier on {:.1}% of {} comparable runs",
                    phm.mean_lead_delta,
                    phm.percent_runs_dsfb_earlier * 100.0,
                    phm.comparable_runs
                ),
                (32, 120),
                ("sans-serif", 18).into_font().color(&BLACK),
            ))
            .map_err(plot_error)?;
        }
        None => {
            area.draw(&Text::new(
                "C. PHM 2018 lead-time advantage".to_string(),
                (32, 42),
                ("sans-serif", 28).into_font().color(&BLACK),
            ))
            .map_err(plot_error)?;
            area.draw(&Rectangle::new(
                [(28, 72), (540, 820)],
                ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(2),
            ))
            .map_err(plot_error)?;
            for (line_index, line) in [
                "PHM panel unavailable in current saved artifacts.",
                "No completed PHM 2018 lead-time summary was found,",
                "so this figure does not claim early-warning value",
                "from SECOM alone.",
                "",
                "Expected artifacts:",
                "- phm2018_lead_time_metrics.csv",
                "- phm2018_early_warning_stats.json",
            ]
            .iter()
            .enumerate()
            {
                area.draw(&Text::new(
                    (*line).to_string(),
                    (48, 130 + (line_index as i32 * 36)),
                    ("sans-serif", 22).into_font().color(&BLACK),
                ))
                .map_err(plot_error)?;
            }
        }
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
            baseline_label: "numeric_only_dsa".into(),
            dsfb_value: Some(secom.optimized_review_escalate_points as f64),
            baseline_value: Some(secom.baseline_investigation_points as f64),
            delta_value: Some(secom.delta_investigation_load),
            units: "count".into(),
            note: "SECOM burden compression against numeric-only DSA investigation-worthy points"
                .into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "A".into(),
            metric: "episode_count".into(),
            baseline_label: "raw_boundary".into(),
            dsfb_value: Some(secom.optimized_episode_count as f64),
            baseline_value: Some(secom.baseline_episode_count as f64),
            delta_value: Some(secom.delta_episode_count),
            units: "count".into(),
            note: "SECOM episode compression against raw boundary episode count".into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "B".into(),
            metric: "episode_precision".into(),
            baseline_label: "raw_boundary_precision_proxy".into(),
            dsfb_value: Some(secom.episode_precision),
            baseline_value: Some(secom.raw_boundary_precision),
            delta_value: Some(secom.precision_gain_factor),
            units: "fraction".into(),
            note: "delta_value is the precision gain factor versus the raw boundary basis".into(),
        },
        UnifiedValueFigureCsvRow {
            panel: "B".into(),
            metric: "recall".into(),
            baseline_label: "labeled_failure_runs".into(),
            dsfb_value: Some(secom.recall as f64),
            baseline_value: Some(secom.failure_runs as f64),
            delta_value: None,
            units: "count".into(),
            note: "Bounded SECOM failure coverage shown alongside burden compression".into(),
        },
    ];

    match phm {
        Some(phm) => {
            rows.push(UnifiedValueFigureCsvRow {
                panel: "C".into(),
                metric: "mean_detection_time".into(),
                baseline_label: "threshold".into(),
                dsfb_value: Some(phm.mean_dsfb_detection_time),
                baseline_value: Some(phm.mean_threshold_detection_time),
                delta_value: Some(phm.mean_lead_delta),
                units: "time_index".into(),
                note: "delta_value is threshold detection time minus DSFB detection time".into(),
            });
            rows.push(UnifiedValueFigureCsvRow {
                panel: "C".into(),
                metric: "percent_runs_dsfb_earlier".into(),
                baseline_label: "comparable_runs".into(),
                dsfb_value: Some(phm.percent_runs_dsfb_earlier),
                baseline_value: Some(phm.comparable_runs as f64),
                delta_value: Some(phm.percent_runs_equal),
                units: "fraction".into(),
                note: format!(
                    "delta_value stores the equal-detection fraction; later fraction is {:.4}",
                    phm.percent_runs_later
                ),
            });
        }
        None => rows.push(UnifiedValueFigureCsvRow {
            panel: "C".into(),
            metric: "phm_panel_status".into(),
            baseline_label: "no_completed_phm_artifact".into(),
            dsfb_value: None,
            baseline_value: None,
            delta_value: None,
            units: "n/a".into(),
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

fn unified_caption(phm_available: bool) -> String {
    if phm_available {
        "Unified DSFB value figure. Left: on SECOM, the policy-governed DSFB layer reduces investigation-worthy alert burden and collapses raw structural episode count while preserving bounded failure coverage. Middle: the same SECOM run shows that DSA episodes are substantially more failure-relevant than the raw boundary basis, making the operator workflow more selective. Right: on PHM 2018, DSFB is evaluated on a degradation-oriented benchmark where early-warning lead time can be measured directly against scalar threshold detection. The figure therefore separates structural compression value (SECOM) from early-warning value (PHM 2018) rather than forcing one dataset to support both claims.".into()
    } else {
        "Unified DSFB value figure. Left: on SECOM, the policy-governed DSFB layer reduces investigation-worthy alert burden and collapses raw structural episode count while preserving bounded failure coverage. Middle: the same SECOM run shows that DSA episodes are substantially more failure-relevant than the raw boundary basis, making the operator workflow more selective. Right: the PHM 2018 early-warning panel is intentionally marked unavailable because no completed PHM lead-time artifact exists in the current saved outputs, so the figure does not claim early-warning value from SECOM alone.".into()
    }
}

fn build_markdown_exec_summary(
    secom: &SecomFigureMetrics,
    phm: Option<&PhmFigureMetrics>,
) -> String {
    let phm_line = match phm {
        Some(phm) => format!(
            "- PHM 2018 lead-time result: mean threshold-minus-DSFB delta {:.2}, DSFB earlier on {:.1}% of comparable runs\n",
            phm.mean_lead_delta,
            phm.percent_runs_dsfb_earlier * 100.0
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
            "\\item PHM 2018 lead-time result: mean threshold-minus-DSFB delta {:.2}, with DSFB earlier on {:.1}\\% of comparable runs.",
            phm.mean_lead_delta,
            phm.percent_runs_dsfb_earlier * 100.0
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
            "- PHM 2018 is used only for lead time. Panel C compares DSFB with threshold on the completed PHM benchmark: mean threshold-minus-DSFB delta `{:.2}`, median `{:.2}`, DSFB earlier on `{:.1}%`, equal on `{:.1}%`, later on `{:.1}%`.\n",
            phm.mean_lead_delta,
            phm.median_lead_delta,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0,
        ),
        None => "- PHM 2018 lead-time artifacts are not yet completed in the current crate-local outputs, so Panel C is a placeholder and the figure does not claim early-warning value from SECOM alone.\n".into(),
    };
    format!(
        "{SECTION_MARKDOWN_START}\n## Unified Structural Compression and Early-Warning Value\n\n![Unified DSFB value figure]({figure_rel_path})\n\n{caption}\n\n- SECOM burden compression uses the numeric-only DSA investigation baseline `{}` and the raw boundary episode baseline `{}`.\n- The SECOM operator result shown here is bounded: policy-governed Review/Escalate points fall to `{}`, DSA episodes fall to `{}`, and bounded failure coverage remains at `{}/{} `.\n- Episode precision is promoted as the primary SECOM operator metric: `{:.1}%` versus raw boundary precision proxy `{:.2}%`, a `{:.1}x` gain.\n{}- This section keeps the claims separated: SECOM supports burden compression and precision, while PHM 2018 is the lead-time benchmark when completed.\n\n{SECTION_MARKDOWN_END}\n\n",
        secom.baseline_investigation_points,
        secom.baseline_episode_count,
        secom.optimized_review_escalate_points,
        secom.optimized_episode_count,
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
            "PHM 2018 is used only for lead time. Panel C compares DSFB with threshold on the completed PHM benchmark: mean threshold-minus-DSFB delta {:.2}, median {:.2}, DSFB earlier on {:.1}\\%, equal on {:.1}\\%, and later on {:.1}\\%.",
            phm.mean_lead_delta,
            phm.median_lead_delta,
            phm.percent_runs_dsfb_earlier * 100.0,
            phm.percent_runs_equal * 100.0,
            phm.percent_runs_later * 100.0,
        ),
        None => "PHM 2018 lead-time artifacts are not yet completed in the current crate-local outputs, so Panel C is a placeholder and this figure does not claim early-warning value from SECOM alone.".into(),
    };
    format!(
        "{SECTION_TEX_START}\n\\section*{{Unified Structural Compression and Early-Warning Value}}\n\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.98\\linewidth]{{figures/dsfb_unified_value_figure.png}}\n\\caption{{{}}}\n\\end{{figure}}\n{}\n\nThe SECOM burden-compression panel uses the numeric-only DSA investigation baseline {} and the raw boundary episode baseline {}. The bounded SECOM result shown here reduces policy-governed Review/Escalate points to {}, reduces DSA episodes to {}, and preserves bounded failure coverage at {}/{}. Episode precision is promoted as the primary SECOM operator metric: {:.1}\\% versus raw boundary precision proxy {:.2}\\%, a {:.1}x gain.\n\n{SECTION_TEX_END}\n\n",
        latex_escape(caption),
        latex_escape(&phm_text),
        secom.baseline_investigation_points,
        secom.baseline_episode_count,
        secom.optimized_review_escalate_points,
        secom.optimized_episode_count,
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
        assert!(report.contains("## Unified Structural Compression and Early-Warning Value"));
        assert!(report.contains("Panel C is a placeholder"));
    }
}
