// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing additive figures for multi-cell and ablation workflows.

use crate::ablation::AblationCellSummary;
use crate::evaluation::{CellEvaluationRun, CellEvaluationSummary};
use crate::noise_robustness::NoiseRobustnessRecord;
use crate::sensitivity::SensitivityScenarioResult;
use crate::sota::SotaPerCellSummary;
use crate::types::GrammarState;
use plotters::prelude::*;
use std::path::Path;
use thiserror::Error;

const WIDTH: u32 = 1100;
const HEIGHT: u32 = 520;

#[derive(Debug, Error)]
pub enum EngineerPlotError {
    #[error("plotting error: {0}")]
    Drawing(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn generate_multicell_lead_time_figure(
    summaries: &[CellEvaluationSummary],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if summaries.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;

    let max_lead = summaries
        .iter()
        .filter_map(|summary| summary.lead_time_vs_threshold_baseline)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Engineer Figure: DSFB lead vs 85% threshold across NASA PCoE cells",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..summaries.len() as f64, 0.0..(max_lead as f64 + 10.0))
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("Cell")
        .y_desc("Cycles earlier than 85% threshold")
        .disable_mesh()
        .draw()
        .map_err(pe)?;

    for (index, summary) in summaries.iter().enumerate() {
        let x0 = index as f64 + 0.1;
        let x1 = index as f64 + 0.9;
        let lead = summary.lead_time_vs_threshold_baseline.unwrap_or(0) as f64;
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(x0, 0.0), (x1, lead.max(0.0))],
                GREEN.filled(),
            )))
            .map_err(pe)?;
        chart
            .draw_series(std::iter::once(Text::new(
                format!(
                    "{}\n{}",
                    summary.cell_id,
                    summary.lead_time_vs_threshold_baseline.unwrap_or(0)
                ),
                (index as f64 + 0.5, lead.max(0.0) + 1.5),
                ("sans-serif", 14).into_font().color(&BLACK),
            )))
            .map_err(pe)?;
    }

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_multicell_trigger_cycle_figure(
    summaries: &[CellEvaluationSummary],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if summaries.is_empty() {
        return Ok(());
    }

    let max_cycle = summaries
        .iter()
        .flat_map(|summary| {
            [
                summary.dsfb_alarm_cycle,
                summary.first_violation_cycle,
                summary.threshold_85pct_cycle,
                summary.eol_80pct_cycle,
            ]
        })
        .flatten()
        .max()
        .unwrap_or(1);

    let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Engineer Figure: Trigger-cycle overview across NASA PCoE cells",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..summaries.len() as f64, 0.0..(max_cycle as f64 + 10.0))
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("Cell")
        .y_desc("Cycle")
        .disable_mesh()
        .draw()
        .map_err(pe)?;

    for (index, summary) in summaries.iter().enumerate() {
        let x = index as f64 + 0.5;
        chart
            .draw_series(std::iter::once(Text::new(
                summary.cell_id.clone(),
                (x, 2.0),
                ("sans-serif", 14).into_font().color(&BLACK),
            )))
            .map_err(pe)?;

        for (cycle, color, label, offset) in [
            (summary.dsfb_alarm_cycle, GREEN, "DSFB", -0.18),
            (summary.first_violation_cycle, RED, "Violation", -0.06),
            (
                summary.threshold_85pct_cycle,
                RGBColor(128, 0, 128),
                "85%",
                0.06,
            ),
            (summary.eol_80pct_cycle, BLUE, "EOL", 0.18),
        ] {
            if let Some(cycle) = cycle {
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (x + offset, cycle as f64),
                        5,
                        color.filled(),
                    )))
                    .map_err(pe)?;
                chart
                    .draw_series(std::iter::once(Text::new(
                        label.to_string(),
                        (x + offset, cycle as f64 + 3.0),
                        ("sans-serif", 11).into_font().color(&color),
                    )))
                    .map_err(pe)?;
            }
        }
    }

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_ablation_comparison_figure(
    cells: &[AblationCellSummary],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if cells.is_empty() {
        return Ok(());
    }

    let max_cycle = cells
        .iter()
        .flat_map(|cell| {
            [
                cell.threshold_baseline.trigger_cycle,
                cell.cumulative_residual.trigger_cycle,
                cell.dsfb.trigger_cycle,
            ]
        })
        .flatten()
        .max()
        .unwrap_or(1);

    let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Engineer Figure: Ablation trigger-cycle comparison",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..cells.len() as f64, 0.0..(max_cycle as f64 + 10.0))
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("Cell")
        .y_desc("Trigger cycle")
        .disable_mesh()
        .draw()
        .map_err(pe)?;

    for (index, cell) in cells.iter().enumerate() {
        let base = index as f64;
        for (offset, cycle, color) in [
            (
                0.10,
                cell.threshold_baseline.trigger_cycle,
                RGBColor(128, 0, 128),
            ),
            (0.38, cell.cumulative_residual.trigger_cycle, BLUE),
            (0.66, cell.dsfb.trigger_cycle, GREEN),
        ] {
            if let Some(cycle) = cycle {
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(base + offset, 0.0), (base + offset + 0.18, cycle as f64)],
                        color.filled(),
                    )))
                    .map_err(pe)?;
            }
        }

        chart
            .draw_series(std::iter::once(Text::new(
                cell.cell_id.clone(),
                (base + 0.45, 2.0),
                ("sans-serif", 14).into_font().color(&BLACK),
            )))
            .map_err(pe)?;
    }

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_multicell_residual_state_overview(
    runs: &[CellEvaluationRun],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if runs.is_empty() {
        return Ok(());
    }

    let cols = 2usize;
    let rows = ((runs.len() + cols - 1) / cols).max(1);
    let root = SVGBackend::new(path, (WIDTH, (HEIGHT as usize * rows) as u32)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let areas = root.split_evenly((rows, cols));

    for (area, run) in areas.into_iter().zip(runs.iter()) {
        let x_min = 1.0;
        let x_max = run.trajectory.len() as f64;
        let y_min = run
            .trajectory
            .iter()
            .map(|sample| sample.sign.r)
            .fold(f64::INFINITY, f64::min)
            .min(-run.envelope.rho)
            * 1.2;
        let y_max = run
            .trajectory
            .iter()
            .map(|sample| sample.sign.r)
            .fold(f64::NEG_INFINITY, f64::max)
            .max(run.envelope.rho)
            * 1.2;

        let mut chart = ChartBuilder::on(&area)
            .caption(
                format!(
                    "{} residual and grammar-state overview",
                    run.summary.cell_id
                ),
                ("sans-serif", 15),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(45)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(pe)?;
        chart
            .configure_mesh()
            .x_desc("Cycle")
            .y_desc("Residual (Ah)")
            .draw()
            .map_err(pe)?;

        chart
            .draw_series(LineSeries::new(
                run.trajectory
                    .iter()
                    .map(|sample| (sample.cycle as f64, sample.sign.r)),
                ShapeStyle::from(&BLUE).stroke_width(2),
            ))
            .map_err(pe)?;
        chart
            .draw_series(LineSeries::new(
                vec![(x_min, run.envelope.rho), (x_max, run.envelope.rho)],
                ShapeStyle::from(&RED).stroke_width(1),
            ))
            .map_err(pe)?;
        chart
            .draw_series(LineSeries::new(
                vec![(x_min, -run.envelope.rho), (x_max, -run.envelope.rho)],
                ShapeStyle::from(&RED).stroke_width(1),
            ))
            .map_err(pe)?;

        for sample in &run.trajectory {
            let color = match sample.grammar_state {
                GrammarState::Admissible => GREEN,
                GrammarState::Boundary => RGBColor(255, 165, 0),
                GrammarState::Violation => RED,
            };
            chart
                .draw_series(std::iter::once(Circle::new(
                    (sample.cycle as f64, sample.sign.r),
                    3,
                    color.filled(),
                )))
                .map_err(pe)?;
        }
    }

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_sensitivity_overview_figure(
    scenarios: &[SensitivityScenarioResult],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if scenarios.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path, (WIDTH, HEIGHT * 2)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let areas = root.split_evenly((2, 1));

    let max_cycle = scenarios
        .iter()
        .flat_map(|scenario| {
            [
                scenario.first_boundary_cycle,
                scenario.first_violation_cycle,
                scenario.dsfb_alarm_cycle,
                scenario.threshold_85pct_cycle,
                scenario.tactical_margin.threshold_cycle,
            ]
        })
        .flatten()
        .max()
        .unwrap_or(1) as f64
        + 10.0;

    let mut chart_top = ChartBuilder::on(&areas[0])
        .caption(
            "Engineer Figure: Sensitivity trigger overview",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..scenarios.len() as f64, 0.0..max_cycle)
        .map_err(pe)?;
    chart_top
        .configure_mesh()
        .x_desc("Scenario index")
        .y_desc("Cycle")
        .draw()
        .map_err(pe)?;

    for (index, scenario) in scenarios.iter().enumerate() {
        let x = index as f64 + 0.5;
        for (value, color) in [
            (scenario.first_boundary_cycle, GREEN),
            (scenario.first_violation_cycle, RED),
            (scenario.dsfb_alarm_cycle, BLUE),
            (scenario.threshold_85pct_cycle, RGBColor(128, 0, 128)),
            (
                scenario.tactical_margin.threshold_cycle,
                RGBColor(255, 140, 0),
            ),
        ] {
            if let Some(value) = value {
                chart_top
                    .draw_series(std::iter::once(Circle::new(
                        (x, value as f64),
                        4,
                        color.filled(),
                    )))
                    .map_err(pe)?;
            }
        }
    }

    let max_t_star = scenarios
        .iter()
        .map(|scenario| scenario.theorem_t_star)
        .max()
        .unwrap_or(1) as f64
        + 10.0;

    let mut chart_bottom = ChartBuilder::on(&areas[1])
        .caption(
            "Engineer Figure: Sensitivity lead time and t*",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..scenarios.len() as f64, 0.0..max_t_star)
        .map_err(pe)?;
    chart_bottom
        .configure_mesh()
        .x_desc("Scenario index")
        .y_desc("Cycles")
        .draw()
        .map_err(pe)?;

    chart_bottom
        .draw_series(LineSeries::new(
            scenarios
                .iter()
                .enumerate()
                .map(|(index, scenario)| (index as f64 + 0.5, scenario.theorem_t_star as f64)),
            ShapeStyle::from(&RED).stroke_width(2),
        ))
        .map_err(pe)?;
    chart_bottom
        .draw_series(LineSeries::new(
            scenarios.iter().enumerate().map(|(index, scenario)| {
                (
                    index as f64 + 0.5,
                    scenario.lead_time_vs_threshold_baseline.unwrap_or(0) as f64,
                )
            }),
            ShapeStyle::from(&BLUE).stroke_width(2),
        ))
        .map_err(pe)?;

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_noise_robustness_figure(
    records: &[NoiseRobustnessRecord],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if records.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let max_cycle = records
        .iter()
        .flat_map(|record| [record.clean_dsfb_alarm_cycle, record.noisy_dsfb_alarm_cycle])
        .flatten()
        .max()
        .unwrap_or(1) as f64
        + 10.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Engineer Figure: Noise robustness DSFB trigger shift",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..records.len() as f64, 0.0..max_cycle)
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("Cell/noise scenario index")
        .y_desc("Cycle")
        .draw()
        .map_err(pe)?;

    for (index, record) in records.iter().enumerate() {
        let x = index as f64 + 0.5;
        if let Some(cycle) = record.clean_dsfb_alarm_cycle {
            chart
                .draw_series(std::iter::once(Circle::new(
                    (x - 0.08, cycle as f64),
                    4,
                    BLUE.filled(),
                )))
                .map_err(pe)?;
        }
        if let Some(cycle) = record.noisy_dsfb_alarm_cycle {
            chart
                .draw_series(std::iter::once(Circle::new(
                    (x + 0.08, cycle as f64),
                    4,
                    RED.filled(),
                )))
                .map_err(pe)?;
        }
    }

    root.present().map_err(pe)?;
    Ok(())
}

pub fn generate_sota_comparison_figure(
    summaries: &[SotaPerCellSummary],
    path: &Path,
) -> Result<(), EngineerPlotError> {
    if summaries.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let max_cycle = summaries
        .iter()
        .flat_map(|summary| {
            [
                summary.threshold_baseline.trigger_cycle,
                summary.cusum_style.trigger_cycle,
                summary.ml_style_rul_proxy.trigger_cycle,
                summary.eis_style_proxy.trigger_cycle,
                summary.dsfb.trigger_cycle,
            ]
        })
        .flatten()
        .max()
        .unwrap_or(1) as f64
        + 10.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Engineer Figure: Comparison trigger cycles",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..summaries.len() as f64, 0.0..max_cycle)
        .map_err(pe)?;
    chart
        .configure_mesh()
        .x_desc("Cell")
        .y_desc("Trigger cycle")
        .draw()
        .map_err(pe)?;

    for (index, summary) in summaries.iter().enumerate() {
        let base = index as f64;
        for (offset, cycle, color) in [
            (
                0.08,
                summary.threshold_baseline.trigger_cycle,
                RGBColor(128, 0, 128),
            ),
            (0.24, summary.cusum_style.trigger_cycle, BLUE),
            (
                0.40,
                summary.ml_style_rul_proxy.trigger_cycle,
                RGBColor(90, 90, 90),
            ),
            (
                0.56,
                summary.eis_style_proxy.trigger_cycle,
                RGBColor(255, 140, 0),
            ),
            (0.72, summary.dsfb.trigger_cycle, GREEN),
        ] {
            if let Some(cycle) = cycle {
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(base + offset, 0.0), (base + offset + 0.12, cycle as f64)],
                        color.filled(),
                    )))
                    .map_err(pe)?;
            }
        }
    }

    root.present().map_err(pe)?;
    Ok(())
}

fn pe<T: std::fmt::Display>(error: T) -> EngineerPlotError {
    EngineerPlotError::Drawing(error.to_string())
}
