use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

use crate::engine::types::FigureArtifact;
use crate::figures::export::figure_paths;
use crate::figures::source::{FigureSourceRow, FigureSourceTable};
use crate::figures::styles::{BLUE, GOLD, GREEN, RED, SLATE, TEAL, WHITE_BG};

#[derive(Clone, Debug)]
struct ExtractedLineSeries {
    label: String,
    color: RGBColor,
    points: Vec<(f64, f64)>,
}

#[derive(Clone, Debug)]
struct ExtractedLinePanel {
    title: String,
    x_label: String,
    y_label: String,
    series: Vec<ExtractedLineSeries>,
}

#[derive(Clone, Debug)]
struct ExtractedBar {
    label: String,
    color: RGBColor,
    left: f64,
    right: f64,
    value: f64,
}

#[derive(Clone, Debug)]
struct ExtractedBarPanel {
    title: String,
    x_label: String,
    y_label: String,
    bars: Vec<ExtractedBar>,
}

#[derive(Clone, Copy, Debug)]
struct LinePanelStyle {
    caption_size: u32,
    margin: u32,
    x_label_area: u32,
    y_label_area: u32,
    zero_floor: bool,
    show_legend: bool,
}

#[derive(Clone, Copy, Debug)]
struct BarPanelStyle {
    caption_size: u32,
    margin: u32,
    x_label_area: u32,
    y_label_area: u32,
}

pub fn render_all_figures(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<Vec<FigureArtifact>> {
    let mut figures = vec![
        render_01(figure_tables, figures_dir)?,
        render_02(figure_tables, figures_dir)?,
        render_03(figure_tables, figures_dir)?,
        render_04(figure_tables, figures_dir)?,
        render_05(figure_tables, figures_dir)?,
        render_06(figure_tables, figures_dir)?,
        render_07(figure_tables, figures_dir)?,
        render_08(figure_tables, figures_dir)?,
        render_09(figure_tables, figures_dir)?,
        render_10(figure_tables, figures_dir)?,
        render_11(figure_tables, figures_dir)?,
        render_12(figure_tables, figures_dir)?,
        render_13(figure_tables, figures_dir)?,
    ];
    if has_figure_table(figure_tables, "figure_14_sweep_stability_summary") {
        figures.push(render_14(figure_tables, figures_dir)?);
    }
    Ok(figures)
}

fn figure_observation_overview<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((2, 1));
    draw_line_panel(
        &areas[0],
        &extract_line_panel(table, "observation_prediction", &["line"])?,
        LinePanelStyle {
            caption_size: 28,
            margin: 16,
            x_label_area: 36,
            y_label_area: 56,
            zero_floor: false,
            show_legend: true,
        },
    )?;
    draw_line_panel(
        &areas[1],
        &extract_line_panel(table, "residual_norm", &["line"])?,
        LinePanelStyle {
            caption_size: 28,
            margin: 16,
            x_label_area: 36,
            y_label_area: 56,
            zero_floor: true,
            show_legend: false,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_drift_slew<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((3, 1));
    for (index, panel_id, zero_floor) in [
        (0, "residual_norm", true),
        (1, "signed_radial_drift", false),
        (2, "slew_norm", true),
    ] {
        draw_line_panel(
            &areas[index],
            &extract_line_panel(table, panel_id, &["line"])?,
            LinePanelStyle {
                caption_size: 26,
                margin: 18,
                x_label_area: 34,
                y_label_area: 54,
                zero_floor,
                show_legend: false,
            },
        )?;
    }
    root.present()?;
    Ok(())
}

fn figure_sign_space<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let panel = extract_line_panel(table, "projection_plane", &["line"])?;
    let line_points = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().copied())
        .collect::<Vec<_>>();
    let marker_rows = marker_rows(table, "projection_plane");
    let annotation_rows = annotation_rows(table, "projection_plane");
    let x_values = line_points
        .iter()
        .map(|(x, _)| *x)
        .chain(marker_rows.iter().map(|row| row.x_value))
        .chain(annotation_rows.iter().map(|row| row.x_value))
        .collect::<Vec<_>>();
    let y_values = line_points
        .iter()
        .map(|(_, y)| *y)
        .chain(marker_rows.iter().map(|row| row.y_value))
        .chain(annotation_rows.iter().map(|row| row.y_value))
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&x_values);
    let (y_min, y_max) = bounds(&y_values);

    let mut chart = ChartBuilder::on(&root)
        .caption(&panel.title, ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc(&panel.x_label)
        .y_desc(&panel.y_label)
        .draw()?;

    for series in &panel.series {
        chart.draw_series(LineSeries::new(
            series.points.iter().copied(),
            &series.color,
        ))?;
    }
    for row in marker_rows {
        let color = color_for_key(&row.color_key);
        chart.draw_series(std::iter::once(Circle::new(
            (row.x_value, row.y_value),
            4,
            color.filled(),
        )))?;
        if !row.annotation_text.is_empty() {
            chart.draw_series(std::iter::once(Text::new(
                row.annotation_text.clone(),
                (row.x_value, row.y_value),
                ("sans-serif", 18).into_font().color(&color),
            )))?;
        }
    }
    for row in annotation_rows {
        chart.draw_series(std::iter::once(Text::new(
            row.annotation_text.clone(),
            (row.x_value, row.y_value),
            ("sans-serif", 14)
                .into_font()
                .color(&color_for_key(&row.color_key)),
        )))?;
    }
    root.present()?;
    Ok(())
}

fn figure_syntax_comparison<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_line_panel(
        &root,
        &extract_line_panel(table, "syntax_comparison", &["line"])?,
        LinePanelStyle {
            caption_size: 30,
            margin: 24,
            x_label_area: 40,
            y_label_area: 56,
            zero_floor: false,
            show_legend: true,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_envelope_exit<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_line_panel_with_segments(
        &root,
        &extract_line_panel(table, "norm_vs_envelope", &["line"])?,
        &segment_rows(table, "norm_vs_envelope"),
        LinePanelStyle {
            caption_size: 30,
            margin: 24,
            x_label_area: 40,
            y_label_area: 56,
            zero_floor: true,
            show_legend: true,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_envelope_invariance<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_line_panel_with_segments(
        &root,
        &extract_line_panel(table, "norm_vs_envelope", &["line"])?,
        &segment_rows(table, "norm_vs_envelope"),
        LinePanelStyle {
            caption_size: 30,
            margin: 24,
            x_label_area: 40,
            y_label_area: 56,
            zero_floor: true,
            show_legend: true,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_exit_invariance_pair<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_line_panel(
        &root,
        &extract_line_panel(table, "exit_invariance_pair", &["line"])?,
        LinePanelStyle {
            caption_size: 30,
            margin: 24,
            x_label_area: 40,
            y_label_area: 56,
            zero_floor: true,
            show_legend: true,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_residual_separation<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    draw_line_panel(
        &root,
        &extract_line_panel(table, "residual_separation", &["line"])?,
        LinePanelStyle {
            caption_size: 30,
            margin: 24,
            x_label_area: 40,
            y_label_area: 56,
            zero_floor: true,
            show_legend: false,
        },
    )?;
    root.present()?;
    Ok(())
}

fn figure_detectability_bounds<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "primary_magnitude_similarity")
    {
        let areas = root.split_evenly((3, 1));
        for (index, panel_id) in [
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence",
        ]
        .into_iter()
        .enumerate()
        {
            draw_line_panel(
                &areas[index],
                &extract_line_panel(table, panel_id, &["line"])?,
                LinePanelStyle {
                    caption_size: 22,
                    margin: 18,
                    x_label_area: 40,
                    y_label_area: 76,
                    zero_floor: true,
                    show_legend: true,
                },
            )?;
        }
    } else if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "detectability_context")
    {
        let areas = root.split_evenly((2, 1));
        draw_line_panel_with_segments(
            &areas[0],
            &extract_line_panel(table, "detectability_context", &["line"])?,
            &segment_rows(table, "detectability_context"),
            LinePanelStyle {
                caption_size: 24,
                margin: 18,
                x_label_area: 40,
                y_label_area: 60,
                zero_floor: true,
                show_legend: true,
            },
        )?;
        draw_bar_panel(
            &areas[1],
            &extract_bar_panel(table, "detectability_window_ratio")?,
            BarPanelStyle {
                caption_size: 24,
                margin: 18,
                x_label_area: 44,
                y_label_area: 64,
            },
        )?;
    } else if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "detectability_gap")
    {
        let areas = root.split_evenly((2, 1));
        draw_bar_panel(
            &areas[0],
            &extract_bar_panel(table, "detectability_bound")?,
            BarPanelStyle {
                caption_size: 24,
                margin: 18,
                x_label_area: 44,
                y_label_area: 64,
            },
        )?;
        draw_bar_panel(
            &areas[1],
            &extract_bar_panel(table, "detectability_gap")?,
            BarPanelStyle {
                caption_size: 24,
                margin: 18,
                x_label_area: 44,
                y_label_area: 64,
            },
        )?;
    } else {
        match extract_bar_panel(table, "detectability_bound") {
            Ok(panel) => draw_bar_panel(
                &root,
                &panel,
                BarPanelStyle {
                    caption_size: 30,
                    margin: 24,
                    x_label_area: 60,
                    y_label_area: 64,
                },
            )?,
            Err(_) => draw_annotation_only_panel(
                &root,
                table,
                "detectability_bound",
                "Predicted vs Observed Detectability Times",
            )?,
        }
    }
    root.present()?;
    Ok(())
}

fn figure_pipeline_flow<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let boxes = box_rows(table, "pipeline_flow");
    let segments = segment_rows(table, "pipeline_flow");
    let annotations = annotation_rows(table, "pipeline_flow");

    for row in boxes {
        let color = color_for_key(&row.color_key);
        let x0 = row.x_value.round() as i32;
        let y0 = row.y_value.round() as i32;
        let x1 = row.secondary_x_value.unwrap_or(row.x_value).round() as i32;
        let y1 = row.secondary_y_value.unwrap_or(row.y_value).round() as i32;
        root.draw(&Rectangle::new(
            [(x0, y0), (x1, y1)],
            color.mix(0.18).filled(),
        ))?;
        root.draw(&Rectangle::new([(x0, y0), (x1, y1)], color.stroke_width(3)))?;

        let (ordinal, subtitle) = row
            .annotation_text
            .split_once('|')
            .map(|(left, right)| (left.trim(), right.trim()))
            .unwrap_or(("", row.annotation_text.as_str()));
        let center_x = (x0 + x1) / 2;
        if !ordinal.is_empty() {
            root.draw(&Text::new(
                ordinal.to_string(),
                (center_x, y0 + 28),
                TextStyle::from(("sans-serif", 22).into_font())
                    .color(&color)
                    .pos(Pos::new(HPos::Center, VPos::Center)),
            ))?;
        }
        root.draw(&Text::new(
            row.series_label.clone(),
            (center_x, y0 + 78),
            TextStyle::from(("sans-serif", 28).into_font())
                .color(&BLACK)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
        if !subtitle.is_empty() {
            root.draw(&Text::new(
                subtitle.to_string(),
                (center_x, y0 + 128),
                TextStyle::from(("sans-serif", 20).into_font())
                    .color(&SLATE)
                    .pos(Pos::new(HPos::Center, VPos::Center)),
            ))?;
        }
    }

    for row in segments {
        let x0 = row.x_value;
        let y0 = row.y_value;
        let x1 = row.secondary_x_value.unwrap_or(row.x_value);
        let y1 = row.secondary_y_value.unwrap_or(row.y_value);
        root.draw(&PathElement::new(
            vec![
                (x0.round() as i32, y0.round() as i32),
                (x1.round() as i32, y1.round() as i32),
            ],
            BLACK.stroke_width(3),
        ))?;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let length = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / length;
        let uy = dy / length;
        let head = 16.0;
        let wing = 10.0;
        let arrow_left = (x1 - head * ux + wing * uy, y1 - head * uy - wing * ux);
        let arrow_right = (x1 - head * ux - wing * uy, y1 - head * uy + wing * ux);
        root.draw(&PathElement::new(
            vec![
                (arrow_left.0.round() as i32, arrow_left.1.round() as i32),
                (x1.round() as i32, y1.round() as i32),
                (arrow_right.0.round() as i32, arrow_right.1.round() as i32),
            ],
            BLACK.stroke_width(3),
        ))?;
    }

    for row in annotations {
        let color = if row.annotation_text == table.plot_title {
            BLACK
        } else {
            color_for_key(&row.color_key)
        };
        let font_size = match row.point_order {
            0 => 36,
            1 => 22,
            2 | 3 => 24,
            _ => 20,
        };
        root.draw(&Text::new(
            row.annotation_text.clone(),
            (row.x_value.round() as i32, row.y_value.round() as i32),
            TextStyle::from(("sans-serif", font_size).into_font())
                .color(&color)
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
    }

    root.present()?;
    Ok(())
}

fn figure_coordinated_group<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let notice_rows = annotation_rows(table, "coordinated_notice");
    if !notice_rows.is_empty() {
        for row in notice_rows {
            root.draw(&Text::new(
                row.annotation_text.clone(),
                (row.x_value.round() as i32, row.y_value.round() as i32),
                TextStyle::from(("sans-serif", 30).into_font())
                    .color(&color_for_key(&row.color_key))
                    .pos(Pos::new(HPos::Center, VPos::Center)),
            ))?;
        }
        root.present()?;
        return Ok(());
    }

    let areas = root.split_evenly((2, 1));
    draw_line_panel(
        &areas[0],
        &extract_line_panel(table, "local_channels", &["line"])?,
        LinePanelStyle {
            caption_size: 26,
            margin: 18,
            x_label_area: 34,
            y_label_area: 54,
            zero_floor: true,
            show_legend: true,
        },
    )?;
    draw_line_panel(
        &areas[1],
        &extract_line_panel(table, "aggregate_group", &["line"])?,
        LinePanelStyle {
            caption_size: 26,
            margin: 18,
            x_label_area: 34,
            y_label_area: 54,
            zero_floor: true,
            show_legend: true,
        },
    )?;
    root.present()?;
    Ok(())
}

fn draw_annotation_only_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
    panel_id: &str,
    fallback_title: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let annotation_rows = annotation_rows(table, panel_id);
    if annotation_rows.is_empty() {
        return Err(anyhow!(
            "missing annotation-only fallback rows for {}:{panel_id}",
            table.figure_id
        ));
    }
    let title = table
        .rows
        .iter()
        .find(|row| row.panel_id == panel_id)
        .map(|row| row.panel_title.clone())
        .unwrap_or_else(|| fallback_title.to_string());
    area.draw(&Text::new(
        title,
        (640, 180),
        TextStyle::from(("sans-serif", 30).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;
    for (index, row) in annotation_rows.into_iter().enumerate() {
        area.draw(&Text::new(
            row.annotation_text.clone(),
            (640, 300 + index as i32 * 42),
            TextStyle::from(("sans-serif", 20).into_font())
                .color(&color_for_key(&row.color_key))
                .pos(Pos::new(HPos::Center, VPos::Center)),
        ))?;
    }
    Ok(())
}

fn figure_semantic_retrieval<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((3, 1));
    if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "semantic_score_timeline")
    {
        for (index, panel_id) in [
            "semantic_score_timeline",
            "semantic_candidate_count_timeline",
            "semantic_disposition_timeline",
        ]
        .into_iter()
        .enumerate()
        {
            draw_line_panel(
                &areas[index],
                &extract_line_panel(table, panel_id, &["line"])?,
                LinePanelStyle {
                    caption_size: 22,
                    margin: 18,
                    x_label_area: 40,
                    y_label_area: 76,
                    zero_floor: true,
                    show_legend: true,
                },
            )?;
        }
    } else {
        for (index, panel_id) in [
            "post_regime_candidate_scores",
            "retrieval_filter_funnel",
            "retrieval_stage_rejections",
        ]
        .into_iter()
        .enumerate()
        {
            draw_bar_panel(
                &areas[index],
                &extract_bar_panel(table, panel_id)?,
                BarPanelStyle {
                    caption_size: 24,
                    margin: 18,
                    x_label_area: 44,
                    y_label_area: 56,
                },
            )?;
        }
    }
    root.present()?;
    Ok(())
}

fn figure_baseline_comparators<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let areas = root.split_evenly((3, 1));
    if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "baseline_alarm_timing")
    {
        draw_bar_panel(
            &areas[0],
            &extract_bar_panel(table, "baseline_alarm_timing")?,
            BarPanelStyle {
                caption_size: 22,
                margin: 18,
                x_label_area: 52,
                y_label_area: 70,
            },
        )?;
        draw_line_panel_with_segments(
            &areas[1],
            &extract_line_panel(table, "dsfb_grammar_timeline", &["line"])?,
            &segment_rows(table, "dsfb_grammar_timeline"),
            LinePanelStyle {
                caption_size: 22,
                margin: 18,
                x_label_area: 40,
                y_label_area: 84,
                zero_floor: true,
                show_legend: false,
            },
        )?;
        draw_line_panel(
            &areas[2],
            &extract_line_panel(table, "dsfb_semantic_timeline", &["line"])?,
            LinePanelStyle {
                caption_size: 22,
                margin: 18,
                x_label_area: 40,
                y_label_area: 96,
                zero_floor: true,
                show_legend: false,
            },
        )?;
    } else {
        for (index, panel_id) in [
            "comparator_first_trigger_time",
            "comparator_onset_rank",
            "comparator_trigger_counts",
        ]
        .into_iter()
        .enumerate()
        {
            draw_bar_panel(
                &areas[index],
                &extract_bar_panel(table, panel_id)?,
                BarPanelStyle {
                    caption_size: 24,
                    margin: 18,
                    x_label_area: 52,
                    y_label_area: 64,
                },
            )?;
        }
    }
    root.present()?;
    Ok(())
}

fn figure_sweep_summary<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    table: &FigureSourceTable,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE_BG)?;
    let panel = extract_line_panel(table, "sweep_semantic_stability", &["line-point"])?;
    let x_values = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|(x, _)| *x))
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&x_values);
    let mut chart = ChartBuilder::on(&root)
        .caption(&panel.title, ("sans-serif", 30))
        .margin(24)
        .x_label_area_size(56)
        .y_label_area_size(70)
        .build_cartesian_2d(x_min..x_max, -0.25..3.25)?;
    chart
        .configure_mesh()
        .x_desc(&panel.x_label)
        .y_desc("semantic disposition")
        .y_labels(4)
        .y_label_formatter(&|value: &f64| match value.round() as i32 {
            0 => "Unknown".to_string(),
            1 => "Ambiguous".to_string(),
            2 => "CompatibleSet".to_string(),
            _ => "Match".to_string(),
        })
        .draw()?;
    for series in &panel.series {
        chart.draw_series(LineSeries::new(
            series.points.iter().copied(),
            &series.color,
        ))?;
        chart.draw_series(
            series
                .points
                .iter()
                .copied()
                .map(|point| Circle::new(point, 4, GOLD.filled())),
        )?;
    }
    for row in annotation_rows(table, "sweep_semantic_stability") {
        let x = if x_min.is_finite() && x_max.is_finite() {
            x_min + (x_max - x_min) * 0.03
        } else {
            row.x_value
        };
        chart.draw_series(std::iter::once(Text::new(
            row.annotation_text.clone(),
            (x, 3.0),
            ("sans-serif", 16)
                .into_font()
                .color(&color_for_key(&row.color_key)),
        )))?;
    }
    root.present()?;
    Ok(())
}

fn extract_line_panel(
    table: &FigureSourceTable,
    panel_id: &str,
    kinds: &[&str],
) -> Result<ExtractedLinePanel> {
    let rows = table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && kinds.contains(&row.series_kind.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let first = rows
        .first()
        .with_context(|| format!("missing rows for {}:{panel_id}", table.figure_id))?;
    let title = first.panel_title.clone();
    let x_label = first.x_label.clone();
    let y_label = first.y_label.clone();
    let mut grouped = BTreeMap::<String, ExtractedLineSeries>::new();
    for row in rows {
        grouped
            .entry(row.series_id.clone())
            .or_insert_with(|| ExtractedLineSeries {
                label: row.series_label.clone(),
                color: color_for_key(&row.color_key),
                points: Vec::new(),
            })
            .points
            .push((row.x_value, row.y_value));
    }
    for series in grouped.values_mut() {
        series.points.sort_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    Ok(ExtractedLinePanel {
        title,
        x_label,
        y_label,
        series: grouped.into_values().collect(),
    })
}

fn extract_bar_panel(table: &FigureSourceTable, panel_id: &str) -> Result<ExtractedBarPanel> {
    let rows = table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && row.series_kind == "bar")
        .cloned()
        .collect::<Vec<_>>();
    let first = rows
        .first()
        .with_context(|| format!("missing bar rows for {}:{panel_id}", table.figure_id))?;
    let title = first.panel_title.clone();
    let x_label = first.x_label.clone();
    let y_label = first.y_label.clone();
    let mut bars = rows
        .into_iter()
        .map(|row| ExtractedBar {
            label: if row.x_tick_label.is_empty() {
                row.series_label
            } else {
                row.x_tick_label
            },
            color: color_for_key(&row.color_key),
            left: row.x_value,
            right: row.secondary_x_value.unwrap_or(row.x_value + 0.8),
            value: row.y_value,
        })
        .collect::<Vec<_>>();
    bars.sort_by(|left, right| {
        left.left
            .partial_cmp(&right.left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(ExtractedBarPanel {
        title,
        x_label,
        y_label,
        bars,
    })
}

fn segment_rows<'a>(table: &'a FigureSourceTable, panel_id: &str) -> Vec<&'a FigureSourceRow> {
    table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && row.series_kind == "segment")
        .collect()
}

fn marker_rows<'a>(table: &'a FigureSourceTable, panel_id: &str) -> Vec<&'a FigureSourceRow> {
    table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && row.series_kind == "marker")
        .collect()
}

fn annotation_rows<'a>(table: &'a FigureSourceTable, panel_id: &str) -> Vec<&'a FigureSourceRow> {
    table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && row.series_kind == "annotation")
        .collect()
}

fn box_rows<'a>(table: &'a FigureSourceTable, panel_id: &str) -> Vec<&'a FigureSourceRow> {
    table
        .rows
        .iter()
        .filter(|row| row.panel_id == panel_id && row.series_kind == "box")
        .collect()
}

fn draw_line_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    panel: &ExtractedLinePanel,
    style: LinePanelStyle,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let x_values = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|(x, _)| *x))
        .collect::<Vec<_>>();
    let y_values = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|(_, y)| *y))
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&x_values);
    let (_, raw_y_max) = bounds(&y_values);
    let (y_min, y_max) = if style.zero_floor {
        (0.0, (raw_y_max * 1.15).max(0.1))
    } else {
        bounds(&y_values)
    };

    let mut chart = ChartBuilder::on(area)
        .caption(&panel.title, ("sans-serif", style.caption_size))
        .margin(style.margin)
        .x_label_area_size(style.x_label_area)
        .y_label_area_size(style.y_label_area)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    chart
        .configure_mesh()
        .x_desc(&panel.x_label)
        .y_desc(&panel.y_label)
        .draw()?;

    for series in &panel.series {
        chart
            .draw_series(LineSeries::new(
                series.points.iter().copied(),
                &series.color,
            ))?
            .label(series.label.clone())
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 18, y)], series.color.stroke_width(3))
            });
    }
    if style.show_legend && panel.series.len() > 1 {
        chart.configure_series_labels().border_style(BLACK).draw()?;
    }
    Ok(())
}

fn draw_line_panel_with_segments<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    panel: &ExtractedLinePanel,
    segments: &[&FigureSourceRow],
    style: LinePanelStyle,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let x_values = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|(x, _)| *x))
        .chain(segments.iter().flat_map(|row| {
            [row.x_value, row.secondary_x_value.unwrap_or(row.x_value)].into_iter()
        }))
        .collect::<Vec<_>>();
    let y_values = panel
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|(_, y)| *y))
        .chain(segments.iter().flat_map(|row| {
            [row.y_value, row.secondary_y_value.unwrap_or(row.y_value)].into_iter()
        }))
        .collect::<Vec<_>>();
    let (x_min, x_max) = bounds(&x_values);
    let (_, raw_y_max) = bounds(&y_values);
    let y_max = if style.zero_floor {
        (raw_y_max * 1.15).max(0.1)
    } else {
        bounds(&y_values).1
    };

    let mut chart = ChartBuilder::on(area)
        .caption(&panel.title, ("sans-serif", style.caption_size))
        .margin(style.margin)
        .x_label_area_size(style.x_label_area)
        .y_label_area_size(style.y_label_area)
        .build_cartesian_2d(x_min..x_max, 0.0..y_max)?;
    chart
        .configure_mesh()
        .x_desc(&panel.x_label)
        .y_desc(&panel.y_label)
        .draw()?;
    for series in &panel.series {
        chart
            .draw_series(LineSeries::new(
                series.points.iter().copied(),
                &series.color,
            ))?
            .label(series.label.clone())
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 18, y)], series.color.stroke_width(3))
            });
    }
    for segment in segments {
        chart.draw_series(std::iter::once(PathElement::new(
            vec![
                (segment.x_value, segment.y_value),
                (
                    segment.secondary_x_value.unwrap_or(segment.x_value),
                    segment.secondary_y_value.unwrap_or(segment.y_value),
                ),
            ],
            color_for_key(&segment.color_key).mix(0.75).stroke_width(2),
        )))?;
        chart.draw_series(std::iter::once(Text::new(
            segment.series_label.clone(),
            (
                segment.x_value,
                segment.secondary_y_value.unwrap_or(segment.y_value) * 0.96,
            ),
            ("sans-serif", 14)
                .into_font()
                .color(&color_for_key(&segment.color_key)),
        )))?;
    }
    if style.show_legend && panel.series.len() > 1 {
        chart.configure_series_labels().border_style(BLACK).draw()?;
    }
    Ok(())
}

fn draw_bar_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    panel: &ExtractedBarPanel,
    style: BarPanelStyle,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let widest_bar = panel
        .bars
        .iter()
        .map(|bar| (bar.right - bar.left).abs())
        .fold(0.0_f64, f64::max)
        .max(0.18);
    let x_min = panel
        .bars
        .iter()
        .map(|bar| bar.left)
        .fold(f64::INFINITY, f64::min);
    let x_max = panel
        .bars
        .iter()
        .map(|bar| bar.right)
        .fold(f64::NEG_INFINITY, f64::max);
    let x_padding = ((x_max - x_min) * 0.12).max(widest_bar * 0.75);
    let y_max = panel
        .bars
        .iter()
        .map(|bar| bar.value)
        .fold(0.0, f64::max)
        .max(1.0);
    let y_upper = (y_max * 1.28).max(y_max + 0.24);
    let mut chart = ChartBuilder::on(area)
        .caption(&panel.title, ("sans-serif", style.caption_size))
        .margin(style.margin)
        .x_label_area_size(style.x_label_area)
        .y_label_area_size(style.y_label_area)
        .build_cartesian_2d((x_min - x_padding)..(x_max + x_padding), 0.0_f64..y_upper)?;
    chart
        .configure_mesh()
        .x_desc(&panel.x_label)
        .y_desc(&panel.y_label)
        .x_labels(0)
        .disable_x_mesh()
        .light_line_style(RGBAColor(0, 0, 0, 0.08))
        .bold_line_style(RGBAColor(0, 0, 0, 0.14))
        .draw()?;
    for bar in &panel.bars {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(bar.left, 0.0), (bar.right, bar.value)],
            bar.color.filled(),
        )))?;
    }

    let label_font_size = if panel.bars.len() > 5 { 12 } else { 14 };
    let mut label_groups = BTreeMap::<String, (f64, f64, f64)>::new();
    for bar in &panel.bars {
        label_groups
            .entry(bar.label.clone())
            .and_modify(|(left, right, top)| {
                *left = left.min(bar.left);
                *right = right.max(bar.right);
                *top = top.max(bar.value);
            })
            .or_insert((bar.left, bar.right, bar.value));
    }
    for (label, (left, right, top)) in label_groups {
        let mid = left + (right - left) * 0.5;
        let label_y = (top + y_upper * 0.04).min(y_upper * 0.95);
        chart.draw_series(std::iter::once(Text::new(
            label,
            (mid, label_y),
            TextStyle::from(("sans-serif", label_font_size).into_font())
                .color(&BLACK)
                .pos(Pos::new(HPos::Center, VPos::Bottom)),
        )))?;
    }
    Ok(())
}

fn render_01(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_01_residual_prediction_observation_overview";
    let caption = "Residual, observation, and prediction overview for the gradual degradation case. Synthetic deterministic demonstration only.";
    let size = (1280, 840);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_observation_overview(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_observation_overview(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_02(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_02_drift_and_slew_decomposition";
    let caption = "Residual norm, signed radial drift, and slew norm decomposition for a representative case. Synthetic deterministic demonstration only when the bundled scenario suite is used.";
    let size = (1280, 960);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_drift_slew(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_drift_slew(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_03(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_03_sign_space_projection";
    let caption = "Projected sign trajectory using the deterministic coordinates [||r||, dot(r,d)/||r||, ||s||]. Synthetic deterministic demonstration only when the bundled scenario suite is used.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_sign_space(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_sign_space(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_04(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_04_syntax_comparison";
    let caption = "Syntax comparison between monotone drift and curvature-dominated trajectories. Synthetic deterministic demonstration only.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_syntax_comparison(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_syntax_comparison(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_05(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_05_envelope_exit_under_sustained_outward_drift";
    let caption = "Residual norm and admissibility envelope for the sustained outward-drift exit case. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_envelope_exit(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_envelope_exit(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_06(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_06_envelope_invariance_under_inward_drift";
    let caption = "Residual norm and admissibility envelope for the inward-compatible invariance case. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_envelope_invariance(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_envelope_invariance(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_07(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_07_exit_invariance_pair_common_envelope";
    let caption = "Exit-invariance pair under a common visualization envelope, contrasting outward drift with inward-compatible containment. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_exit_invariance_pair(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_exit_invariance_pair(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_08(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_08_residual_trajectory_separation";
    let caption = "Residual trajectory separation between magnitude-matched admissible and detectable cases. Synthetic theorem-aligned demonstration only.";
    let size = (1280, 720);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_residual_separation(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_residual_separation(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_09(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_09_detectability_bound_comparison";
    let table = figure_table(figure_tables, figure_id)?;
    let caption = if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "primary_magnitude_similarity")
    {
        if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_bearings_public_demo")
        {
            "NASA Bearings paper figure. Two within-run windows are matched on similar primary residual magnitude, then contrasted by meta-residual slew and downstream grammar outcome. The figure argues conservatively that primary magnitude alone is insufficient for separation in this run."
        } else if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_milling_public_demo")
        {
            "NASA Milling paper figure. Two process windows are matched on similar primary residual behavior, then separated by higher-order residual structure and grammar outcome. The figure argues conservatively that first-order behavior alone is insufficient in this milling run."
        } else {
            "Synthetic paper figure. Two controlled cases retain similar primary residual magnitude while higher-order structure and grammar outcome diverge. The figure argues conservatively that first-order behavior alone is insufficient even in a controlled synthetic setting."
        }
    } else {
        "Run-specific detectability view. The exported figure preserves the paper-facing filename while using either multi-case bound-versus-observed timing summaries or single-run residual-versus-envelope context with windowed detectability ratios, depending on the executed run."
    };
    let size = (1280, 860);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_detectability_bounds(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_detectability_bounds(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_10(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_10_deterministic_pipeline_flow";
    let caption = "Deterministic layered engine flow showing residual extraction, sign construction, syntax, grammar, and semantic retrieval as auditable maps.";
    let size = (1600, 900);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_pipeline_flow(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_pipeline_flow(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_11(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_11_coordinated_group_semiotics";
    let caption = "Local versus aggregate envelopes for the grouped correlated case, supporting the grouped aggregate breach fraction used in the coordinated syntax and semantic summaries. Synthetic deterministic demonstration only.";
    let size = (1280, 840);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_coordinated_group(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_coordinated_group(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_12(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_12_semantic_retrieval_heuristics_bank";
    let table = figure_table(figure_tables, figure_id)?;
    let caption = if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "semantic_score_timeline")
    {
        if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_bearings_public_demo")
        {
            "NASA Bearings paper figure. The panels show semantic interpretation through time: evolving top-candidate score and score margin, narrowing candidate counts, and the disposition timeline. This is a semantic-process view, not a static bank-existence summary."
        } else if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_milling_public_demo")
        {
            "NASA Milling paper figure. The panels show semantic interpretation through the milling process: evolving top-candidate score and score margin, narrowing candidate counts, and the disposition timeline. This is a semantic-process view, not a static bank-existence summary."
        } else {
            "Synthetic paper figure. The panels show semantic interpretation through a controlled structural transition: evolving top-candidate score and score margin, narrowing candidate counts, and the disposition timeline. This is a semantic-process view, not a static bank-existence summary."
        }
    } else {
        "Run-specific constrained-retrieval process summary rendered from exported source rows. Panel 1 shows ranked post-regime candidate scores, panel 2 shows the deterministic filter funnel, and panel 3 shows stage-specific rejection counts. The figure remains within-run rather than cross-dataset."
    };
    let size = (1280, 860);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_semantic_retrieval(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_semantic_retrieval(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_13(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_13_internal_baseline_comparators";
    let table = figure_table(figure_tables, figure_id)?;
    let caption = if table
        .panel_ids
        .iter()
        .any(|panel_id| panel_id == "baseline_alarm_timing")
    {
        if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_bearings_public_demo")
        {
            "NASA Bearings paper figure. Panel A shows what the internal deterministic comparators see first, while Panels B and C show the additional DSFB grammar and semantic timelines. The figure is framed as an interpretability delta, not a performance benchmark."
        } else if table
            .rows
            .iter()
            .any(|row| row.scenario_id == "nasa_milling_public_demo")
        {
            "NASA Milling paper figure. Panel A shows what the internal deterministic comparators see first in the milling run, while Panels B and C show the additional DSFB grammar and semantic timelines. The figure is framed as an interpretability delta, not a performance benchmark."
        } else {
            "Synthetic paper figure. Panel A shows what the internal deterministic comparators see first in the controlled synthetic transition, while Panels B and C show the additional DSFB grammar and semantic timelines. The figure is framed as an interpretability delta, not a performance benchmark."
        }
    } else {
        "Run-specific internal deterministic comparator activity. The panels show first-trigger timing, onset ordering, and triggered-scenario counts within the executed run. These remain within-crate comparator views only, not field benchmarks."
    };
    let size = (1280, 920);
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_baseline_comparators(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_baseline_comparators(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn render_14(figure_tables: &[FigureSourceTable], figures_dir: &Path) -> Result<FigureArtifact> {
    let figure_id = "figure_14_sweep_stability_summary";
    let caption = "Deterministic sweep summary showing how semantic dispositions vary over the configured synthetic sweep parameter. This is an internal calibration-style plot only.";
    let size = (1280, 760);
    let table = figure_table(figure_tables, figure_id)?;
    let (png_path, svg_path) = figure_paths(figures_dir, figure_id);
    figure_sweep_summary(
        BitMapBackend::new(&png_path, size).into_drawing_area(),
        table,
    )?;
    figure_sweep_summary(SVGBackend::new(&svg_path, size).into_drawing_area(), table)?;
    Ok(artifact(figure_id, caption, png_path, svg_path))
}

fn figure_table<'a>(
    tables: &'a [FigureSourceTable],
    figure_id: &str,
) -> Result<&'a FigureSourceTable> {
    tables
        .iter()
        .find(|table| table.figure_id == figure_id)
        .with_context(|| format!("missing figure source table `{figure_id}`"))
}

fn has_figure_table(tables: &[FigureSourceTable], figure_id: &str) -> bool {
    tables.iter().any(|table| table.figure_id == figure_id)
}

fn color_for_key(color_key: &str) -> RGBColor {
    match color_key {
        "blue" => BLUE,
        "green" => GREEN,
        "red" => RED,
        "teal" => TEAL,
        "gold" => GOLD,
        "slate" => SLATE,
        _ => BLACK,
    }
}

fn bounds(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !min.is_finite() || !max.is_finite() {
        (0.0, 1.0)
    } else if (max - min).abs() < 1.0e-9 {
        (min.min(0.0) - 0.1, max.max(0.0) + 0.1)
    } else {
        let margin = (max - min) * 0.08;
        (min - margin, max + margin)
    }
}

fn artifact(
    figure_id: &str,
    caption: &str,
    png_path: std::path::PathBuf,
    svg_path: std::path::PathBuf,
) -> FigureArtifact {
    FigureArtifact {
        figure_id: figure_id.to_string(),
        caption: caption.to_string(),
        png_path: png_path.display().to_string(),
        svg_path: svg_path.display().to_string(),
    }
}
