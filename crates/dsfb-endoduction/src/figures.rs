//! Deterministic artifact generation: 12 publication-grade PNG figures.
//!
//! All figures use a clean white background with black/grey palette.
//! No chartjunk. Publication-grade labels.

use crate::types::WindowMetrics;
use anyhow::{Context, Result};
use plotters::prelude::*;
use std::path::Path;

const W: u32 = 1200;
const H: u32 = 600;

/// Generate all 12 figures into the given output directory.
///
/// Returns a list of file names produced.
pub fn generate_all(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    early_signal: &[f64],
    late_signal: &[f64],
    sample_residual: &[f64],
    sample_observation: &[f64],
    sample_model: &[f64],
    _envelope_upper: &[f64],
    _envelope_lower: &[f64],
    out_dir: &Path,
) -> Result<Vec<String>> {
    let mut files = Vec::new();

    files.push(fig01_dataset_overview(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig02_raw_signal_snapshots(early_signal, late_signal, out_dir)?);
    files.push(fig03_conventional_diagnostics(metrics, nominal_end, out_dir)?);
    files.push(fig04_nominal_vs_observation(sample_observation, sample_model, sample_residual, out_dir)?);
    files.push(fig05_residual_evolution(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig06_admissibility_envelope(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig07_structural_grammar_panel(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig08_trust_score(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig09_baseline_comparison(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig10_lead_time_comparison(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig11_robustness(metrics, nominal_end, failure_window, out_dir)?);
    files.push(fig12_summary_synthesis(metrics, nominal_end, failure_window, out_dir)?);

    Ok(files)
}

/// Helper: determine y-axis range from data with margin.
fn y_range(data: &[f64]) -> std::ops::Range<f64> {
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let margin = (max - min).abs() * 0.1 + 1e-10;
    (min - margin)..(max + margin)
}

/// Helper: abbreviated file index axis label.
fn x_label() -> &'static str {
    "Window Index (chronological)"
}

// ---- Figure 1: Dataset overview timeline ----

fn fig01_dataset_overview(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig01_dataset_overview.png";
    let path = out_dir.join(fname);
    let rms: Vec<f64> = metrics.iter().map(|m| m.rms).collect();
    let n = rms.len();

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 1: Dataset Overview — Run Duration & Regimes", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, y_range(&rms))?;

    chart.configure_mesh().x_desc(x_label()).y_desc("RMS").draw()?;

    // Nominal shading
    chart.draw_series(std::iter::once(Rectangle::new(
        [(0, chart.y_range().start), (nominal_end, chart.y_range().end)],
        RGBColor(220, 240, 220).filled(),
    )))?;

    // Failure line
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(failure_window, chart.y_range().start), (failure_window, chart.y_range().end)],
        RED.stroke_width(2),
    )))?
    .label("Failure reference")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    chart.draw_series(LineSeries::new(
        rms.iter().enumerate().map(|(i, &v)| (i, v)),
        BLACK.stroke_width(1),
    ))?
    .label("RMS")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(1)));

    chart.configure_series_labels().border_style(BLACK).draw()?;

    root.present().context("fig01 render")?;
    Ok(fname.to_string())
}

// ---- Figure 2: Raw vibration signal snapshots ----

fn fig02_raw_signal_snapshots(
    early: &[f64],
    late: &[f64],
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig02_raw_signal_snapshots.png";
    let path = out_dir.join(fname);

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 1));

    // Early window
    {
        let range = y_range(early);
        let mut chart = ChartBuilder::on(&areas[0])
            .caption("Early Nominal Window", ("sans-serif", 16))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..early.len(), range)?;
        chart.configure_mesh().y_desc("Amplitude").draw()?;
        chart.draw_series(LineSeries::new(
            early.iter().enumerate().map(|(i, &v)| (i, v)),
            BLACK.stroke_width(1),
        ))?;
    }

    // Late window
    {
        let range = y_range(late);
        let mut chart = ChartBuilder::on(&areas[1])
            .caption("Late Degradation Window", ("sans-serif", 16))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..late.len(), range)?;
        chart.configure_mesh().x_desc("Sample").y_desc("Amplitude").draw()?;
        chart.draw_series(LineSeries::new(
            late.iter().enumerate().map(|(i, &v)| (i, v)),
            RGBColor(180, 0, 0).stroke_width(1),
        ))?;
    }

    root.present().context("fig02 render")?;
    Ok(fname.to_string())
}

// ---- Figure 3: Conventional diagnostics ----

fn fig03_conventional_diagnostics(
    metrics: &[WindowMetrics],
    _nominal_end: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig03_conventional_diagnostics.png";
    let path = out_dir.join(fname);
    let n = metrics.len();

    let rms: Vec<f64> = metrics.iter().map(|m| m.rms).collect();
    let kurt: Vec<f64> = metrics.iter().map(|m| m.kurtosis).collect();
    let var: Vec<f64> = metrics.iter().map(|m| m.baseline_rolling_var).collect();

    let root = BitMapBackend::new(&path, (W, H + 200)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((3, 1));

    for (area, (data, label)) in areas
        .iter()
        .zip([(&rms, "RMS"), (&kurt, "Kurtosis"), (&var, "Variance")])
    {
        let range = y_range(data);
        let mut chart = ChartBuilder::on(area)
            .caption(format!("Conventional: {label}"), ("sans-serif", 16))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(0..n, range)?;
        chart.configure_mesh().y_desc(label).draw()?;
        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, &v)| (i, v)),
            BLACK.stroke_width(1),
        ))?;
    }

    root.present().context("fig03 render")?;
    Ok(fname.to_string())
}

// ---- Figure 4: Nominal model vs observation ----

fn fig04_nominal_vs_observation(
    obs: &[f64],
    model: &[f64],
    residual: &[f64],
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig04_nominal_vs_observation.png";
    let path = out_dir.join(fname);
    let show_n = obs.len().min(1024);

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((2, 1));

    // Top: observation vs model
    {
        let mut all = obs[..show_n].to_vec();
        all.extend_from_slice(&model[..show_n.min(model.len())]);
        let range = y_range(&all);
        let mut chart = ChartBuilder::on(&areas[0])
            .caption("Observation vs Nominal Model", ("sans-serif", 16))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..show_n, range)?;
        chart.configure_mesh().y_desc("Amplitude").draw()?;
        chart.draw_series(LineSeries::new(
            obs[..show_n].iter().enumerate().map(|(i, &v)| (i, v)),
            BLACK.stroke_width(1),
        ))?
        .label("x_obs(t)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(1)));
        chart.draw_series(LineSeries::new(
            model[..show_n.min(model.len())]
                .iter()
                .enumerate()
                .map(|(i, &v)| (i, v)),
            BLUE.stroke_width(1),
        ))?
        .label("x_model(t)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(1)));
        chart.configure_series_labels().border_style(BLACK).draw()?;
    }

    // Bottom: residual
    {
        let range = y_range(&residual[..show_n.min(residual.len())]);
        let mut chart = ChartBuilder::on(&areas[1])
            .caption("Residual r(t) = x_obs(t) - x_model(t)", ("sans-serif", 16))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0..show_n.min(residual.len()), range)?;
        chart.configure_mesh().x_desc("Sample").y_desc("Residual").draw()?;
        chart.draw_series(LineSeries::new(
            residual[..show_n.min(residual.len())]
                .iter()
                .enumerate()
                .map(|(i, &v)| (i, v)),
            RGBColor(0, 100, 0).stroke_width(1),
        ))?;
    }

    root.present().context("fig04 render")?;
    Ok(fname.to_string())
}

// ---- Figure 5: Residual evolution over time ----

fn fig05_residual_evolution(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig05_residual_evolution.png";
    let path = out_dir.join(fname);
    let n = metrics.len();
    let res_var: Vec<f64> = metrics.iter().map(|m| m.residual_variance).collect();

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 5: Residual Variance Evolution", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, y_range(&res_var))?;
    chart.configure_mesh().x_desc(x_label()).y_desc("Residual Variance").draw()?;

    chart.draw_series(std::iter::once(Rectangle::new(
        [(0, chart.y_range().start), (nominal_end, chart.y_range().end)],
        RGBColor(220, 240, 220).filled(),
    )))?;
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(failure_window, chart.y_range().start), (failure_window, chart.y_range().end)],
        RED.stroke_width(2),
    )))?;

    chart.draw_series(LineSeries::new(
        res_var.iter().enumerate().map(|(i, &v)| (i, v)),
        BLACK.stroke_width(1),
    ))?;

    root.present().context("fig05 render")?;
    Ok(fname.to_string())
}

// ---- Figure 6: Admissibility envelope figure ----

fn fig06_admissibility_envelope(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig06_admissibility_envelope.png";
    let path = out_dir.join(fname);
    let n = metrics.len();
    let breach: Vec<f64> = metrics.iter().map(|m| m.envelope_breach_fraction).collect();

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 6: Envelope Breach Density Over Time", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, 0.0..1.0)?;
    chart.configure_mesh().x_desc(x_label()).y_desc("Breach Fraction").draw()?;

    chart.draw_series(std::iter::once(Rectangle::new(
        [(0, 0.0), (nominal_end, 1.0)],
        RGBColor(220, 240, 220).filled(),
    )))?;
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(failure_window, 0.0), (failure_window, 1.0)],
        RED.stroke_width(2),
    )))?;

    chart.draw_series(LineSeries::new(
        breach.iter().enumerate().map(|(i, &v)| (i, v)),
        RGBColor(0, 0, 180).stroke_width(1),
    ))?;

    root.present().context("fig06 render")?;
    Ok(fname.to_string())
}

// ---- Figure 7: Structural grammar motif panel ----

fn fig07_structural_grammar_panel(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig07_structural_grammar_panel.png";
    let path = out_dir.join(fname);
    let n = metrics.len();

    let drift: Vec<f64> = metrics.iter().map(|m| m.drift.abs()).collect();
    let persist: Vec<f64> = metrics.iter().map(|m| m.persistence).collect();
    let var_gr: Vec<f64> = metrics.iter().map(|m| m.variance_growth).collect();
    let ac: Vec<f64> = metrics.iter().map(|m| m.residual_autocorr).collect();

    let root = BitMapBackend::new(&path, (W, H + 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((4, 1));

    let datasets: Vec<(&[f64], &str, RGBColor)> = vec![
        (&drift, "Drift Magnitude", RGBColor(0, 0, 0)),
        (&persist, "Persistence", RGBColor(0, 100, 0)),
        (&var_gr, "Variance Growth", RGBColor(0, 0, 180)),
        (&ac, "Autocorrelation", RGBColor(180, 0, 0)),
    ];

    for (area, (data, label, color)) in areas.iter().zip(datasets) {
        let range = y_range(data);
        let mut chart = ChartBuilder::on(area)
            .caption(format!("Motif: {label}"), ("sans-serif", 14))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(60)
            .build_cartesian_2d(0..n, range.clone())?;
        chart.configure_mesh().y_desc(label).draw()?;
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0, range.start), (nominal_end, range.end)],
            RGBColor(220, 240, 220).filled(),
        )))?;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(failure_window, range.start), (failure_window, range.end)],
            RED.stroke_width(1),
        )))?;
        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, &v)| (i, v)),
            color.stroke_width(1),
        ))?;
    }

    root.present().context("fig07 render")?;
    Ok(fname.to_string())
}

// ---- Figure 8: Trust / precursor score ----

fn fig08_trust_score(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig08_trust_score.png";
    let path = out_dir.join(fname);
    let n = metrics.len();
    let trust: Vec<f64> = metrics.iter().map(|m| m.trust_score).collect();

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 8: DSFB Trust / Precursor Score", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, 0.0..1.0)?;
    chart.configure_mesh().x_desc(x_label()).y_desc("Trust Score").draw()?;

    chart.draw_series(std::iter::once(Rectangle::new(
        [(0, 0.0), (nominal_end, 1.0)],
        RGBColor(220, 240, 220).filled(),
    )))?;
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(failure_window, 0.0), (failure_window, 1.0)],
        RED.stroke_width(2),
    )))?
    .label("Failure reference")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    // Threshold line
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(0, 0.5), (n, 0.5)],
        BLACK.stroke_width(1),
    )))?
    .label("Threshold (0.5)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.stroke_width(1)));

    chart.draw_series(LineSeries::new(
        trust.iter().enumerate().map(|(i, &v)| (i, v)),
        RGBColor(0, 0, 200).stroke_width(2),
    ))?
    .label("DSFB Trust Score")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(0, 0, 200).stroke_width(2)));

    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present().context("fig08 render")?;
    Ok(fname.to_string())
}

// ---- Figure 9: Baseline comparison ----

fn fig09_baseline_comparison(
    metrics: &[WindowMetrics],
    _nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig09_baseline_comparison.png";
    let path = out_dir.join(fname);
    let n = metrics.len();

    // Normalise each series to [0, 1] for comparison.
    let trust: Vec<f64> = metrics.iter().map(|m| m.trust_score).collect();
    let rms = normalise_series(&metrics.iter().map(|m| m.rms).collect::<Vec<_>>());
    let kurt = normalise_series(&metrics.iter().map(|m| m.kurtosis).collect::<Vec<_>>());
    let cf = normalise_series(&metrics.iter().map(|m| m.crest_factor).collect::<Vec<_>>());
    let var = normalise_series(&metrics.iter().map(|m| m.baseline_rolling_var).collect::<Vec<_>>());

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 9: DSFB vs Baselines (normalised)", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n, 0.0..1.1)?;
    chart.configure_mesh().x_desc(x_label()).y_desc("Normalised Score").draw()?;

    chart.draw_series(std::iter::once(PathElement::new(
        vec![(failure_window, 0.0), (failure_window, 1.1)],
        RED.stroke_width(2),
    )))?;

    let series_items: Vec<(Vec<f64>, &str, RGBColor)> = vec![
        (trust, "DSFB Trust", RGBColor(0, 0, 200)),
        (rms, "RMS (norm)", RGBColor(100, 100, 100)),
        (kurt, "Kurtosis (norm)", RGBColor(0, 150, 0)),
        (cf, "Crest Factor (norm)", RGBColor(200, 100, 0)),
        (var, "Variance (norm)", RGBColor(150, 0, 150)),
    ];

    for (data, label, color) in &series_items {
        let color = *color;
        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, &v)| (i, v)),
            color.stroke_width(1),
        ))?
        .label(*label)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(1)));
    }

    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present().context("fig09 render")?;
    Ok(fname.to_string())
}

/// Normalise a series to [0, 1].
fn normalise_series(v: &[f64]) -> Vec<f64> {
    let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range.abs() < 1e-30 {
        return vec![0.0; v.len()];
    }
    v.iter().map(|&x| (x - min) / range).collect()
}

// ---- Figure 10: Lead-time comparison chart ----

fn fig10_lead_time_comparison(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig10_lead_time_comparison.png";
    let path = out_dir.join(fname);

    // Compute first sustained detections (reuse evaluation logic).
    let sustained = 5;
    let trust_flags: Vec<bool> = metrics.iter().map(|m| m.trust_score >= 0.5).collect();
    let dsfb_det = crate::baselines::first_sustained_detection(&trust_flags, sustained);

    // Baseline detections using simple threshold (mean + 3*std from nominal).
    let methods: Vec<(&str, Vec<f64>)> = vec![
        ("RMS", metrics.iter().map(|m| m.rms).collect()),
        ("Kurtosis", metrics.iter().map(|m| m.kurtosis).collect()),
        ("Crest Factor", metrics.iter().map(|m| m.crest_factor).collect()),
        ("Variance", metrics.iter().map(|m| m.baseline_rolling_var).collect()),
        ("Autocorr", metrics.iter().map(|m| m.residual_autocorr).collect()),
        ("Spectral", metrics.iter().map(|m| m.spectral_band_energy).collect()),
    ];

    let mut bars: Vec<(&str, i64)> = Vec::new();
    if let Some(d) = dsfb_det {
        bars.push(("DSFB", failure_window as i64 - d as i64));
    }
    for (name, vals) in &methods {
        let nom_vals: Vec<f64> = vals[..nominal_end].to_vec();
        let mean = crate::baseline::mean(&nom_vals);
        let std = crate::baseline::std_dev(&nom_vals);
        let flags: Vec<bool> = vals.iter().map(|&v| v > mean + 3.0 * std).collect();
        if let Some(d) = crate::baselines::first_sustained_detection(&flags, sustained) {
            bars.push((name, failure_window as i64 - d as i64));
        }
    }

    bars.sort_by(|a, b| b.1.cmp(&a.1));

    let n_bars = bars.len().max(1);
    let max_lead = bars.iter().map(|b| b.1).max().unwrap_or(1).max(1);

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 10: Lead-Time Comparison (windows before failure)", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(120)
        .build_cartesian_2d(0..max_lead + 10, 0..n_bars)?;
    chart.configure_mesh().y_desc("Method").x_desc("Lead Time (windows)").draw()?;

    for (i, &(name, lead)) in bars.iter().enumerate() {
        let color = if name == "DSFB" {
            RGBColor(0, 0, 200)
        } else {
            RGBColor(120, 120, 120)
        };
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0_i64, i), (lead, i + 1)],
            color.filled(),
        )))?
        .label(name)
        .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.filled()));
    }

    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present().context("fig10 render")?;
    Ok(fname.to_string())
}

// ---- Figure 11: Robustness / sensitivity ----

fn fig11_robustness(
    metrics: &[WindowMetrics],
    _nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig11_robustness.png";
    let path = out_dir.join(fname);

    // Show trust score detection sensitivity across different thresholds.
    let thresholds: Vec<f64> = (1..10).map(|i| i as f64 * 0.1).collect();
    let sustained = 5;
    let mut detections: Vec<(f64, Option<usize>)> = Vec::new();
    for &thr in &thresholds {
        let flags: Vec<bool> = metrics.iter().map(|m| m.trust_score >= thr).collect();
        let det = crate::baselines::first_sustained_detection(&flags, sustained);
        detections.push((thr, det));
    }

    let leads: Vec<f64> = detections
        .iter()
        .map(|(_, d)| d.map_or(0.0, |d| (failure_window as i64 - d as i64) as f64))
        .collect();

    let root = BitMapBackend::new(&path, (W, H)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_lead = leads.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
    let mut chart = ChartBuilder::on(&root)
        .caption("Figure 11: Detection Sensitivity vs Trust Threshold", ("sans-serif", 20))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..1.0, 0.0..max_lead * 1.1)?;
    chart
        .configure_mesh()
        .x_desc("Trust Threshold")
        .y_desc("Lead Time (windows)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        thresholds.iter().zip(leads.iter()).map(|(&t, &l)| (t, l)),
        BLACK.stroke_width(2),
    ))?;
    chart.draw_series(PointSeries::of_element(
        thresholds.iter().zip(leads.iter()).map(|(&t, &l)| (t, l)),
        4,
        BLACK.filled(),
        &|c, s, st| Circle::new(c, s, st),
    ))?;

    root.present().context("fig11 render")?;
    Ok(fname.to_string())
}

// ---- Figure 12: Summary synthesis ----

fn fig12_summary_synthesis(
    metrics: &[WindowMetrics],
    nominal_end: usize,
    failure_window: usize,
    out_dir: &Path,
) -> Result<String> {
    let fname = "fig12_summary_synthesis.png";
    let path = out_dir.join(fname);
    let n = metrics.len();

    let trust: Vec<f64> = metrics.iter().map(|m| m.trust_score).collect();
    let breach: Vec<f64> = metrics.iter().map(|m| m.envelope_breach_fraction).collect();
    let rms = normalise_series(&metrics.iter().map(|m| m.rms).collect::<Vec<_>>());

    let root = BitMapBackend::new(&path, (W, H + 200)).into_drawing_area();
    root.fill(&WHITE)?;
    let areas = root.split_evenly((3, 1));

    // Panel 1: RMS (normalised)
    {
        let mut chart = ChartBuilder::on(&areas[0])
            .caption("Raw RMS (normalised)", ("sans-serif", 14))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(50)
            .build_cartesian_2d(0..n, 0.0..1.1)?;
        chart.configure_mesh().draw()?;
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0, 0.0), (nominal_end, 1.1)],
            RGBColor(220, 240, 220).filled(),
        )))?;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(failure_window, 0.0), (failure_window, 1.1)],
            RED.stroke_width(1),
        )))?;
        chart.draw_series(LineSeries::new(
            rms.iter().enumerate().map(|(i, &v)| (i, v)),
            BLACK.stroke_width(1),
        ))?;
    }

    // Panel 2: Breach fraction
    {
        let mut chart = ChartBuilder::on(&areas[1])
            .caption("Envelope Breach Density", ("sans-serif", 14))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(50)
            .build_cartesian_2d(0..n, 0.0..1.0)?;
        chart.configure_mesh().draw()?;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(failure_window, 0.0), (failure_window, 1.0)],
            RED.stroke_width(1),
        )))?;
        chart.draw_series(LineSeries::new(
            breach.iter().enumerate().map(|(i, &v)| (i, v)),
            RGBColor(0, 0, 180).stroke_width(1),
        ))?;
    }

    // Panel 3: Trust score
    {
        let mut chart = ChartBuilder::on(&areas[2])
            .caption("DSFB Trust Score → Precursor Detection", ("sans-serif", 14))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(50)
            .build_cartesian_2d(0..n, 0.0..1.0)?;
        chart.configure_mesh().x_desc(x_label()).draw()?;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(failure_window, 0.0), (failure_window, 1.0)],
            RED.stroke_width(1),
        )))?;
        chart.draw_series(LineSeries::new(
            trust.iter().enumerate().map(|(i, &v)| (i, v)),
            RGBColor(0, 0, 200).stroke_width(2),
        ))?;
    }

    root.present().context("fig12 render")?;
    Ok(fname.to_string())
}
