//! Run-specific plot renderers for the upgraded paper/demo figures.

use super::*;

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

pub(super) fn render_09(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<FigureArtifact> {
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

pub(super) fn render_10(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<FigureArtifact> {
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

pub(super) fn render_11(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<FigureArtifact> {
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

pub(super) fn render_12(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<FigureArtifact> {
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

pub(super) fn render_13(
    figure_tables: &[FigureSourceTable],
    figures_dir: &Path,
) -> Result<FigureArtifact> {
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
