// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Figure generation (SVG)

use crate::types::{
    BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, PipelineConfig,
    Theorem1Result,
};
use plotters::prelude::*;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlotError {
    #[error("plotting error: {0}")]
    Drawing(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for figure generation.
pub struct FigureContext<'a> {
    pub capacities: &'a [f64],
    pub trajectory: &'a [BatteryResidual],
    pub envelope: &'a EnvelopeParams,
    pub config: &'a PipelineConfig,
    pub dsfb_detection: &'a DetectionResult,
    pub threshold_detection: &'a DetectionResult,
    pub theorem1: &'a Theorem1Result,
    pub data_provenance: &'a str,
}

/// Generate all 12 figures as SVG in the output directory.
pub fn generate_all_figures(ctx: &FigureContext, output_dir: &Path) -> Result<(), PlotError> {
    std::fs::create_dir_all(output_dir).map_err(PlotError::Io)?;

    let n = ctx.capacities.len();
    let cycles: Vec<f64> = (1..=n).map(|c| c as f64).collect();
    let residuals: Vec<f64> = ctx.trajectory.iter().map(|br| br.sign.r).collect();
    let drifts: Vec<f64> = ctx.trajectory.iter().map(|br| br.sign.d).collect();
    let slews: Vec<f64> = ctx.trajectory.iter().map(|br| br.sign.s).collect();
    let eol_capacity = ctx.config.eol_fraction * ctx.capacities[0];

    draw_line(output_dir, "fig01_capacity_fade",
        "Figure 1: Raw Capacity Fade Curve", "Cycle Number", "Discharge Capacity (Ah)",
        &cycles, &[(&ctx.capacities.to_vec(), "Measured capacity", &BLUE)],
        Some((eol_capacity, "EOL threshold", &RED)))?;

    draw_line(output_dir, "fig02_residual_trajectory",
        "Figure 2: Residual Trajectory r_k", "Cycle Number", "Residual r_k (Ah)",
        &cycles, &[(&residuals, "r_k = y_k - mu", &BLUE)],
        Some((0.0, "Zero line", &BLACK)))?;

    draw_line(output_dir, "fig03_drift_trajectory",
        "Figure 3: Drift Trajectory d_k", "Cycle Number", "Drift d_k (Ah/cycle)",
        &cycles, &[(&drifts, "d_k (windowed drift)", &BLUE)],
        Some((-ctx.config.drift_threshold, "Drift threshold", &RED)))?;

    draw_line(output_dir, "fig04_slew_trajectory",
        "Figure 4: Slew Trajectory s_k", "Cycle Number", "Slew s_k",
        &cycles, &[(&slews, "s_k (windowed slew)", &BLUE)],
        Some((-ctx.config.slew_threshold, "Slew threshold", &RED)))?;

    {
        let upper: Vec<f64> = std::iter::repeat(ctx.envelope.rho).take(n).collect();
        let lower: Vec<f64> = std::iter::repeat(-ctx.envelope.rho).take(n).collect();
        draw_envelope(output_dir, "fig05_admissibility_envelope",
            "Figure 5: Admissibility Envelope", "Cycle Number", "Residual r_k (Ah)",
            &cycles, &residuals, &upper, &lower)?;
    }

    draw_grammar_timeline(output_dir, "fig06_grammar_state_timeline", ctx.trajectory)?;

    draw_detection_comparison(output_dir, "fig07_detection_comparison",
        &cycles, ctx.capacities, eol_capacity, ctx.dsfb_detection, ctx.threshold_detection)?;

    draw_theorem1(output_dir, "fig08_theorem1_verification",
        &cycles, &residuals, ctx.envelope, ctx.theorem1, ctx.config)?;

    draw_scatter(output_dir, "fig09_semiotic_projection",
        "Figure 9: Semiotic Projection (r_k vs d_k)", "Residual r_k (Ah)", "Drift d_k (Ah/cycle)",
        &residuals, &drifts, ctx.trajectory)?;

    {
        let cum_drift: Vec<f64> = drifts.iter().scan(0.0, |acc, &d| { *acc += d; Some(*acc) }).collect();
        draw_line(output_dir, "fig10_cumulative_drift",
            "Figure 10: Cumulative Drift", "Cycle Number", "Cumulative Drift (Ah)",
            &cycles, &[(&cum_drift, "Cumulative drift", &BLUE)],
            Some((0.0, "Zero", &BLACK)))?;
    }

    draw_lead_time_bars(output_dir, "fig11_lead_time_comparison",
        ctx.dsfb_detection, ctx.threshold_detection)?;

    draw_heuristic_bank(output_dir, "fig12_heuristics_bank_entry")?;

    Ok(())
}

const FW: u32 = 1000;
const FH: u32 = 500;

fn svgp(dir: &Path, stem: &str) -> std::path::PathBuf {
    dir.join(format!("{}.svg", stem))
}

fn pe<T: std::fmt::Display>(e: T) -> PlotError {
    PlotError::Drawing(e.to_string())
}

fn draw_line(
    dir: &Path, stem: &str, title: &str, xlabel: &str, ylabel: &str,
    x: &[f64], series: &[(&Vec<f64>, &str, &RGBColor)],
    hline: Option<(f64, &str, &RGBColor)>,
) -> Result<(), PlotError> {
    let path = svgp(dir, stem);
    let (x_min, x_max) = (x[0], x[x.len() - 1]);
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    for (data, _, _) in series {
        for &v in data.iter() {
            if v < y_min { y_min = v; }
            if v > y_max { y_max = v; }
        }
    }
    if let Some((hv, _, _)) = hline {
        if hv < y_min { y_min = hv; }
        if hv > y_max { y_max = hv; }
    }
    let m = (y_max - y_min) * 0.08;
    y_min -= m;
    y_max += m;

    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(x_min..x_max, y_min..y_max).map_err(pe)?;
    chart.configure_mesh().x_desc(xlabel).y_desc(ylabel).draw().map_err(pe)?;

    for (data, label, color) in series {
        chart.draw_series(LineSeries::new(
            x.iter().zip(data.iter()).map(|(&xi, &yi)| (xi, yi)),
            ShapeStyle::from(*color).stroke_width(2),
        )).map_err(pe)?
        .label(*label)
        .legend(move |(lx, ly)| PathElement::new(vec![(lx, ly), (lx + 20, ly)], *color));
    }
    if let Some((hv, hlabel, hcolor)) = hline {
        chart.draw_series(LineSeries::new(
            vec![(x_min, hv), (x_max, hv)],
            ShapeStyle::from(*hcolor).stroke_width(1),
        )).map_err(pe)?
        .label(hlabel)
        .legend(move |(lx, ly)| PathElement::new(vec![(lx, ly), (lx + 20, ly)], *hcolor));
    }
    chart.configure_series_labels().border_style(BLACK).background_style(WHITE.mix(0.8))
        .draw().map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_envelope(
    dir: &Path, stem: &str, title: &str, xlabel: &str, ylabel: &str,
    x: &[f64], data: &[f64], upper: &[f64], lower: &[f64],
) -> Result<(), PlotError> {
    let path = svgp(dir, stem);
    let (x_min, x_max) = (x[0], x[x.len() - 1]);
    let y_min = lower.iter().chain(data.iter()).cloned().fold(f64::INFINITY, f64::min) * 1.2;
    let y_max = upper.iter().chain(data.iter()).cloned().fold(f64::NEG_INFINITY, f64::max) * 1.2;

    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(x_min..x_max, y_min..y_max).map_err(pe)?;
    chart.configure_mesh().x_desc(xlabel).y_desc(ylabel).draw().map_err(pe)?;

    chart.draw_series(LineSeries::new(
        x.iter().zip(upper.iter()).map(|(&xi, &yi)| (xi, yi)),
        ShapeStyle::from(&RED).stroke_width(1),
    )).map_err(pe)?.label("+rho").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], RED));

    chart.draw_series(LineSeries::new(
        x.iter().zip(lower.iter()).map(|(&xi, &yi)| (xi, yi)),
        ShapeStyle::from(&RED).stroke_width(1),
    )).map_err(pe)?.label("-rho").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], RED));

    chart.draw_series(LineSeries::new(
        x.iter().zip(data.iter()).map(|(&xi, &yi)| (xi, yi)),
        ShapeStyle::from(&BLUE).stroke_width(2),
    )).map_err(pe)?.label("Residual r_k").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], BLUE));

    chart.configure_series_labels().border_style(BLACK).background_style(WHITE.mix(0.8))
        .draw().map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_grammar_timeline(
    dir: &Path, stem: &str, trajectory: &[BatteryResidual],
) -> Result<(), PlotError> {
    let n = trajectory.len();
    let x: Vec<f64> = (1..=n).map(|c| c as f64).collect();
    let y: Vec<f64> = trajectory.iter().map(|br| match br.grammar_state {
        GrammarState::Admissible => 0.0,
        GrammarState::Boundary => 1.0,
        GrammarState::Violation => 2.0,
    }).collect();

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 6: Grammar State Timeline", ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(1.0..n as f64, -0.5..2.5).map_err(pe)?;
    chart.configure_mesh().x_desc("Cycle Number").y_desc("Grammar State").draw().map_err(pe)?;

    for i in 0..n {
        let color = match trajectory[i].grammar_state {
            GrammarState::Admissible => GREEN,
            GrammarState::Boundary => RGBColor(255, 165, 0),
            GrammarState::Violation => RED,
        };
        chart.draw_series(std::iter::once(Rectangle::new(
            [(x[i] - 0.5, -0.3), (x[i] + 0.5, y[i] + 0.3)], color.filled(),
        ))).map_err(pe)?;
    }
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_detection_comparison(
    dir: &Path, stem: &str, x: &[f64], capacities: &[f64],
    eol_capacity: f64, dsfb: &DetectionResult, threshold: &DetectionResult,
) -> Result<(), PlotError> {
    let (x_min, x_max) = (x[0], x[x.len() - 1]);
    let y_min = capacities.iter().cloned().fold(f64::INFINITY, f64::min) - 0.05;
    let y_max = capacities.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + 0.05;

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 7: Detection Comparison", ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(x_min..x_max, y_min..y_max).map_err(pe)?;
    chart.configure_mesh().x_desc("Cycle Number").y_desc("Capacity (Ah)").draw().map_err(pe)?;

    chart.draw_series(LineSeries::new(
        x.iter().zip(capacities.iter()).map(|(&xi, &yi)| (xi, yi)),
        ShapeStyle::from(&BLUE).stroke_width(2),
    )).map_err(pe)?.label("Capacity").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], BLUE));

    chart.draw_series(LineSeries::new(
        vec![(x_min, eol_capacity), (x_max, eol_capacity)],
        ShapeStyle::from(&RED).stroke_width(1),
    )).map_err(pe)?.label("EOL").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], RED));

    if let Some(cycle) = dsfb.alarm_cycle {
        chart.draw_series(LineSeries::new(
            vec![(cycle as f64, y_min), (cycle as f64, y_max)],
            ShapeStyle::from(&GREEN).stroke_width(2),
        )).map_err(pe)?.label("DSFB alarm").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], GREEN));
    }
    if let Some(cycle) = threshold.alarm_cycle {
        chart.draw_series(LineSeries::new(
            vec![(cycle as f64, y_min), (cycle as f64, y_max)],
            ShapeStyle::from(&RGBColor(128, 0, 128)).stroke_width(2),
        )).map_err(pe)?.label("Threshold alarm").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], RGBColor(128, 0, 128)));
    }
    chart.configure_series_labels().border_style(BLACK).background_style(WHITE.mix(0.8))
        .draw().map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_theorem1(
    dir: &Path, stem: &str, x: &[f64], residuals: &[f64],
    envelope: &EnvelopeParams, thm1: &Theorem1Result, config: &PipelineConfig,
) -> Result<(), PlotError> {
    let (x_min, x_max) = (x[0], x[x.len() - 1]);
    let abs_r: Vec<f64> = residuals.iter().map(|r| r.abs()).collect();
    let y_max = abs_r.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 1.1;
    let k0 = config.healthy_window as f64;

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 8: Theorem 1 Verification", ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(x_min..x_max, 0.0..y_max).map_err(pe)?;
    chart.configure_mesh().x_desc("Cycle Number").y_desc("|r_k| (Ah)").draw().map_err(pe)?;

    chart.draw_series(LineSeries::new(
        x.iter().zip(abs_r.iter()).map(|(&xi, &yi)| (xi, yi)),
        ShapeStyle::from(&BLUE).stroke_width(2),
    )).map_err(pe)?.label("|r_k|").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], BLUE));

    chart.draw_series(LineSeries::new(
        vec![(x_min, envelope.rho), (x_max, envelope.rho)],
        ShapeStyle::from(&RED).stroke_width(1),
    )).map_err(pe)?.label("rho (envelope)").legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], RED));

    chart.draw_series(LineSeries::new(
        vec![(k0, 0.0), (k0 + thm1.t_star as f64, envelope.rho)],
        ShapeStyle::from(&GREEN).stroke_width(2),
    )).map_err(pe)?.label(format!("Thm 1 (t*={})", thm1.t_star))
    .legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], GREEN));

    chart.configure_series_labels().border_style(BLACK).background_style(WHITE.mix(0.8))
        .draw().map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_scatter(
    dir: &Path, stem: &str, title: &str, xlabel: &str, ylabel: &str,
    xs: &[f64], ys: &[f64], trajectory: &[BatteryResidual],
) -> Result<(), PlotError> {
    let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mx = (x_max - x_min) * 0.08;
    let my = (y_max - y_min) * 0.08;

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 14)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d((x_min - mx)..(x_max + mx), (y_min - my)..(y_max + my)).map_err(pe)?;
    chart.configure_mesh().x_desc(xlabel).y_desc(ylabel).draw().map_err(pe)?;

    for (i, br) in trajectory.iter().enumerate() {
        let color = match br.grammar_state {
            GrammarState::Admissible => GREEN,
            GrammarState::Boundary => RGBColor(255, 165, 0),
            GrammarState::Violation => RED,
        };
        chart.draw_series(std::iter::once(Circle::new((xs[i], ys[i]), 3, color.filled()))).map_err(pe)?;
    }
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_lead_time_bars(
    dir: &Path, stem: &str, dsfb: &DetectionResult, threshold: &DetectionResult,
) -> Result<(), PlotError> {
    let dsfb_lead = dsfb.lead_time_cycles.unwrap_or(0) as f64;
    let thresh_lead = threshold.lead_time_cycles.unwrap_or(0) as f64;
    let y_max = dsfb_lead.max(thresh_lead) * 1.2;

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, FH)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 11: Lead Time Comparison", ("sans-serif", 16)).margin(10)
        .x_label_area_size(35).y_label_area_size(55)
        .build_cartesian_2d(0.0..3.0, 0.0..y_max).map_err(pe)?;
    chart.configure_mesh().y_desc("Lead Time (cycles)").draw().map_err(pe)?;

    chart.draw_series(std::iter::once(Rectangle::new(
        [(0.5, 0.0), (1.3, dsfb_lead)], GREEN.filled(),
    ))).map_err(pe)?.label(format!("DSFB ({:.0})", dsfb_lead))
    .legend(|(lx, ly)| Rectangle::new([(lx, ly-5), (lx+15, ly+5)], GREEN.filled()));

    chart.draw_series(std::iter::once(Rectangle::new(
        [(1.7, 0.0), (2.5, thresh_lead)], RGBColor(128, 0, 128).filled(),
    ))).map_err(pe)?.label(format!("Threshold ({:.0})", thresh_lead))
    .legend(|(lx, ly)| Rectangle::new([(lx, ly-5), (lx+15, ly+5)], RGBColor(128, 0, 128).filled()));

    chart.configure_series_labels().border_style(BLACK).background_style(WHITE.mix(0.8))
        .draw().map_err(pe)?;
    root.present().map_err(pe)?;
    Ok(())
}

fn draw_heuristic_bank(dir: &Path, stem: &str) -> Result<(), PlotError> {
    let n = 100;
    let x: Vec<f64> = (1..=n).map(|c| c as f64).collect();
    let cap: Vec<f64> = x.iter().map(|&c| 2.0 - 0.003 * c).collect();

    let path = svgp(dir, stem);
    let root = SVGBackend::new(&path, (FW, 600)).into_drawing_area();
    root.fill(&WHITE).map_err(pe)?;
    let (upper, lower) = root.split_vertically(400);

    {
        let mut chart = ChartBuilder::on(&upper)
            .caption("Figure 12: Heuristics Bank Entry — SEI Growth Motif", ("sans-serif", 14))
            .margin(10).x_label_area_size(35).y_label_area_size(55)
            .build_cartesian_2d(0.0..100.0, 1.5..2.1).map_err(pe)?;
        chart.configure_mesh().x_desc("Cycle").y_desc("Capacity (Ah)").draw().map_err(pe)?;
        chart.draw_series(LineSeries::new(
            x.iter().zip(cap.iter()).map(|(&xi, &yi)| (xi, yi)),
            ShapeStyle::from(&BLUE).stroke_width(2),
        )).map_err(pe)?.label("SEI growth motif")
        .legend(|(lx, ly)| PathElement::new(vec![(lx, ly), (lx+20, ly)], BLUE));
        chart.configure_series_labels().border_style(BLACK).draw().map_err(pe)?;
    }

    let ts = TextStyle::from(("sans-serif", 11).into_font());
    let entries = [
        ("P (Pattern):", "Monotone neg. drift, near-zero slew"),
        ("R (Regime):", "Constant-current, moderate temp"),
        ("A (Assumptions):", "Envelope static, no regime change"),
        ("I (Interpretation):", "SEI layer growth (capacity fade)"),
        ("U (Uncertainty):", "Cannot distinguish SEI vs mild plating"),
    ];
    for (i, (key, val)) in entries.iter().enumerate() {
        let y = 20 + i as i32 * 22;
        lower.draw_text(key, &ts, (20, y)).map_err(pe)?;
        lower.draw_text(val, &ts, (200, y)).map_err(pe)?;
    }
    root.present().map_err(pe)?;
    Ok(())
}
