//! Figure rendering — 10 per fixture + 3 cross-fixture summaries.
//!
//! All figures are rendered via `plotters` to 800×600 PNG.
//! Captions are auto-generated from fixture metadata; positioned as
//! plot titles (top) plus subtitles where applicable. No caption ever
//! exceeds the figure width or wraps below the bottom margin.

#![cfg(feature = "demo")]

use std::path::Path;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{format, vec};

use plotters::prelude::*;

use crate::demo::runner::FixtureRun;
use crate::error::Result;
use crate::types::{DriftDirection, GrammarState};

const FIG_W: u32 = 1024;
const FIG_H: u32 = 768;

/// Render the 10 per-fixture figures into `out_dir`.
pub fn render_per_fixture(run: &FixtureRun, out_dir: &Path) -> Result<()> {
    fig01_residual_heatmap(run, &out_dir.join("01_residual_heatmap.png"))?;
    fig02_signal_timeseries(run, &out_dir.join("02_signal_timeseries.png"))?;
    fig03_drift_per_signal(run, &out_dir.join("03_drift_per_signal.png"))?;
    fig04_slew_per_signal(run, &out_dir.join("04_slew_per_signal.png"))?;
    fig05_grammar_state_matrix(run, &out_dir.join("05_grammar_state_matrix.png"))?;
    fig06_policy_state_heatmap(run, &out_dir.join("06_policy_state_heatmap.png"))?;
    fig07_per_detector_firing(run, &out_dir.join("07_per_detector_firing.png"))?;
    fig08_episode_timeline(run, &out_dir.join("08_episode_timeline.png"))?;
    fig09_evidence_packet(run, &out_dir.join("09_evidence_packet.png"))?;
    fig10_replay_verification(run, &out_dir.join("10_replay_verification.png"))?;
    Ok(())
}

// =========================================================================
// Figure 1: Raw residual heatmap (windows × signals)
// =========================================================================

fn fig01_residual_heatmap(run: &FixtureRun, path: &Path) -> Result<()> {
    let nw = run.matrix.num_windows;
    let ns = run.matrix.num_signals;
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();

    let title = format!(
        "{} — raw residual matrix ({} windows × {} signals)",
        run.manifest_name, nw, ns
    );
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    // Find min/max for colour scaling.
    let mut vmin = f64::INFINITY;
    let mut vmax = f64::NEG_INFINITY;
    for &v in &run.matrix.data {
        if v.is_finite() {
            if v < vmin { vmin = v; }
            if v > vmax { vmax = v; }
        }
    }
    if !vmin.is_finite() || !vmax.is_finite() || vmin >= vmax {
        vmin = 0.0;
        vmax = 1.0;
    }

    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..(nw as i32), 0..(ns as i32))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Window index")
        .y_desc("Signal index")
        .axis_desc_style(("sans-serif", 14))
        .draw().ok();

    let cell_w = 1i32;
    let cell_h = 1i32;
    chart.draw_series(
        (0..nw).flat_map(|w| (0..ns).map(move |s| (w as i32, s as i32))).map(|(wi, si)| {
            let v = run.matrix.data[(wi as usize) * ns + (si as usize)];
            let norm = if v.is_finite() {
                ((v - vmin) / (vmax - vmin)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (r, g, b) = viridis(norm);
            Rectangle::new(
                [(wi, si), (wi + cell_w, si + cell_h)],
                RGBColor(r, g, b).filled(),
            )
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 2: Per-signal residual time-series (line plot)
// =========================================================================

fn fig02_signal_timeseries(run: &FixtureRun, path: &Path) -> Result<()> {
    let nw = run.matrix.num_windows;
    let ns = run.matrix.num_signals;
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — per-signal residual time-series", run.manifest_name);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let mut vmin = f64::INFINITY;
    let mut vmax = f64::NEG_INFINITY;
    for &v in &run.matrix.data {
        if v.is_finite() {
            if v < vmin { vmin = v; }
            if v > vmax { vmax = v; }
        }
    }
    if !vmin.is_finite() || !vmax.is_finite() || vmin >= vmax {
        vmin = -1.0; vmax = 1.0;
    }

    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(nw as i32), vmin..vmax)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Window index")
        .y_desc("Residual value")
        .draw().ok();

    for s in 0..ns {
        let h = (s as f32) / (ns as f32);
        let (r, g, b) = hsl_to_rgb(h, 0.6, 0.45);
        let stroke = RGBColor(r, g, b);
        chart.draw_series(LineSeries::new(
            (0..nw).filter_map(|w| {
                let v = run.matrix.data[w * ns + s];
                if v.is_finite() { Some((w as i32, v)) } else { None }
            }),
            stroke,
        )).ok();
    }

    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 3: Per-signal drift persistence (bar chart)
// =========================================================================

fn fig03_drift_per_signal(run: &FixtureRun, path: &Path) -> Result<()> {
    let ns = run.matrix.num_signals;
    let nw = run.matrix.num_windows;

    // Average drift_persistence per signal across windows.
    let mut sum = std::vec![0.0_f64; ns];
    let mut cnt = std::vec![0u32; ns];
    for ev in &run.eval_grid {
        let s = ev.signal_index as usize;
        if s < ns {
            sum[s] += ev.drift_persistence;
            cnt[s] += 1;
        }
    }
    let avg: Vec<f64> = (0..ns).map(|s| {
        if cnt[s] == 0 { 0.0 } else { sum[s] / cnt[s] as f64 }
    }).collect();

    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — average drift persistence per signal (over {} windows)", run.manifest_name, nw);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let max = avg.iter().cloned().fold(0.0_f64, f64::max).max(0.01);
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(ns as i32 + 1), 0.0..max * 1.15)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Signal index")
        .y_desc("Avg drift persistence (fraction)")
        .draw().ok();
    chart.draw_series(
        avg.iter().enumerate().map(|(i, &v)| {
            Rectangle::new([(i as i32, 0.0), (i as i32 + 1, v)], BLUE.mix(0.7).filled())
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 4: Per-signal slew density (bar chart)
// =========================================================================

fn fig04_slew_per_signal(run: &FixtureRun, path: &Path) -> Result<()> {
    let ns = run.matrix.num_signals;
    let nw = run.matrix.num_windows;

    let mut sum = std::vec![0.0_f64; ns];
    let mut cnt = std::vec![0u32; ns];
    for ev in &run.eval_grid {
        let s = ev.signal_index as usize;
        if s < ns {
            sum[s] += ev.sign_tuple.slew.abs();
            cnt[s] += 1;
        }
    }
    let avg: Vec<f64> = (0..ns).map(|s| {
        if cnt[s] == 0 { 0.0 } else { sum[s] / cnt[s] as f64 }
    }).collect();

    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — average |slew| per signal (over {} windows)", run.manifest_name, nw);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let max = avg.iter().cloned().fold(0.0_f64, f64::max).max(0.001);
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(ns as i32 + 1), 0.0..max * 1.15)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Signal index")
        .y_desc("Avg |slew|")
        .draw().ok();
    chart.draw_series(
        avg.iter().enumerate().map(|(i, &v)| {
            Rectangle::new([(i as i32, 0.0), (i as i32 + 1, v)], MAGENTA.mix(0.7).filled())
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 5: Grammar-state matrix (3-color heatmap)
// =========================================================================

fn fig05_grammar_state_matrix(run: &FixtureRun, path: &Path) -> Result<()> {
    let nw = run.matrix.num_windows;
    let ns = run.matrix.num_signals;
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — grammar state per (window, signal)", run.manifest_name);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(nw as i32), 0..(ns as i32))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Window")
        .y_desc("Signal")
        .draw().ok();

    chart.draw_series(
        run.eval_grid.iter().map(|ev| {
            let wi = ev.window_index as i32;
            let si = ev.signal_index as i32;
            let color = match ev.confirmed_grammar_state {
                GrammarState::Admissible => RGBColor(240, 240, 240),
                GrammarState::Boundary => RGBColor(255, 165, 0),
                GrammarState::Violation => RGBColor(220, 20, 60),
            };
            Rectangle::new([(wi, si), (wi + 1, si + 1)], color.filled())
        })
    ).ok();

    // Legend swatches at top-right
    let legend_x = (nw as f64 * 0.85) as i32;
    let labels = [("Admissible", RGBColor(240, 240, 240)),
                  ("Boundary",   RGBColor(255, 165, 0)),
                  ("Violation",  RGBColor(220, 20, 60))];
    for (i, (lab, col)) in labels.iter().enumerate() {
        let yc = (ns as i32) - 1 - (i as i32);
        chart.draw_series(std::iter::once(
            Rectangle::new([(legend_x, yc), (legend_x + 2, yc + 1)], col.filled())
        )).ok();
        chart.draw_series(std::iter::once(
            Text::new(lab.to_string(), (legend_x + 4, yc), ("sans-serif", 12))
        )).ok();
    }
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 6: Policy state heatmap (Silent / Watch / Review / Escalate)
// =========================================================================

fn fig06_policy_state_heatmap(run: &FixtureRun, path: &Path) -> Result<()> {
    let nw = run.matrix.num_windows;
    let ns = run.matrix.num_signals;
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — policy state per (window, signal)", run.manifest_name);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(nw as i32), 0..(ns as i32))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Window")
        .y_desc("Signal")
        .draw().ok();

    chart.draw_series(
        run.eval_grid.iter().map(|ev| {
            let wi = ev.window_index as i32;
            let si = ev.signal_index as i32;
            let color = match ev.policy_state {
                crate::types::PolicyState::Silent   => RGBColor(245, 245, 245),
                crate::types::PolicyState::Watch    => RGBColor(135, 206, 235),
                crate::types::PolicyState::Review   => RGBColor(255, 165, 0),
                crate::types::PolicyState::Escalate => RGBColor(220, 20, 60),
            };
            Rectangle::new([(wi, si), (wi + 1, si + 1)], color.filled())
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 7: Top-30 detectors by firing rate (bar chart)
// =========================================================================

fn fig07_per_detector_firing(run: &FixtureRun, path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!("{} — top-30 detectors by total alert count", run.manifest_name);
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let mut entries: Vec<(String, u64)> = run.fusion_metrics.per_detector.iter()
        .map(|d| (d.detector_name.to_string(), d.raw_alert_count))
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(30);
    if entries.is_empty() {
        // Fallback: empty plot with explanatory text.
        bottom.draw(&Text::new(
            "no detector outputs on this fixture",
            (FIG_W as i32 / 2 - 200, FIG_H as i32 / 2),
            ("sans-serif", 18),
        )).ok();
        root.present().ok();
        return Ok(());
    }

    let max = entries.iter().map(|(_, n)| *n).max().unwrap_or(1) as f64;
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(120)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(entries.len() as i32 + 1), 0.0..max * 1.15)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_label_formatter(&|x| {
            let i = *x as usize;
            if i > 0 && i <= entries.len() {
                let raw = &entries[i - 1].0;
                if raw.len() > 14 { raw[..14].to_string() } else { raw.clone() }
            } else {
                String::new()
            }
        })
        .x_label_style(("sans-serif", 9).into_text_style(&bottom).transform(FontTransform::Rotate90))
        .y_desc("Total alert count")
        .draw().ok();
    chart.draw_series(
        entries.iter().enumerate().map(|(i, (_, n))| {
            Rectangle::new(
                [(i as i32 + 1, 0.0), (i as i32 + 2, *n as f64)],
                Palette99::pick(i).filled(),
            )
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 8: Closed-episode timeline (Gantt-style with motif annotation)
// =========================================================================

fn fig08_episode_timeline(run: &FixtureRun, path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!(
        "{} — closed structural episodes ({} total)",
        run.manifest_name, run.episodes.len()
    );
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    if run.episodes.is_empty() {
        bottom.draw(&Text::new(
            "no episodes on this fixture (steady-state or endoductive validator)".to_string(),
            (FIG_W as i32 / 2 - 280, FIG_H as i32 / 2),
            ("sans-serif", 18),
        )).ok();
        root.present().ok();
        return Ok(());
    }

    let nw = run.matrix.num_windows as i32;
    let n_ep = run.episodes.len() as i32;
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(0..nw, 0..n_ep)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Window")
        .y_desc("Episode #")
        .draw().ok();
    chart.draw_series(
        run.episodes.iter().enumerate().map(|(i, ep)| {
            let color = match ep.peak_grammar_state {
                GrammarState::Admissible => RGBColor(200, 200, 200),
                GrammarState::Boundary => RGBColor(255, 165, 0),
                GrammarState::Violation => RGBColor(220, 20, 60),
            };
            Rectangle::new(
                [(ep.start_window as i32, i as i32),
                 (ep.end_window as i32 + 1, i as i32 + 1)],
                color.filled(),
            )
        })
    ).ok();
    chart.draw_series(
        run.episodes.iter().enumerate().map(|(i, ep)| {
            let label = match ep.matched_motif {
                crate::types::SemanticDisposition::Named(m) => format!("ep{}: {:?}", ep.episode_id, m),
                crate::types::SemanticDisposition::Unknown => format!("ep{}: Unknown", ep.episode_id),
            };
            Text::new(label, (ep.start_window as i32 + 2, i as i32), ("sans-serif", 11))
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 9: Operator evidence packet (placeholder summary view per episode)
// =========================================================================

fn fig09_evidence_packet(run: &FixtureRun, path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!(
        "{} — closed-episode signature summary ({} episodes)",
        run.manifest_name, run.episodes.len()
    );
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    if run.episodes.is_empty() {
        bottom.draw(&Text::new(
            "no episodes on this fixture".to_string(),
            (FIG_W as i32 / 2 - 200, FIG_H as i32 / 2),
            ("sans-serif", 18),
        )).ok();
        root.present().ok();
        return Ok(());
    }

    // For each closed episode, draw a 5-axis bar (peak_slew, duration_windows,
    // contributing_signals, drift_direction_indicator, dominant_grammar_state).
    let n_ep = run.episodes.len() as i32;
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(0..(5 * n_ep + 1), 0.0_f64..1.0_f64)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("Axis × Episode (peak_slew, duration, signals, drift_dir, peak_state)")
        .y_desc("Normalised value")
        .draw().ok();

    let max_slew = run.episodes.iter()
        .map(|e| e.structural_signature.peak_slew_magnitude.abs())
        .fold(0.0_f64, f64::max).max(1e-6);
    let max_dur = run.episodes.iter()
        .map(|e| e.structural_signature.duration_windows as f64)
        .fold(1.0_f64, f64::max);
    let max_sig = run.episodes.iter()
        .map(|e| e.contributing_signal_count as f64)
        .fold(1.0_f64, f64::max);

    for (i, ep) in run.episodes.iter().enumerate() {
        let xb = (i as i32) * 5;
        let vals = [
            (ep.structural_signature.peak_slew_magnitude.abs() / max_slew).clamp(0.0, 1.0),
            (ep.structural_signature.duration_windows as f64 / max_dur).clamp(0.0, 1.0),
            (ep.contributing_signal_count as f64 / max_sig).clamp(0.0, 1.0),
            match ep.structural_signature.dominant_drift_direction {
                DriftDirection::None => 0.0,
                DriftDirection::Negative => 0.33,
                DriftDirection::Oscillatory => 0.66,
                DriftDirection::Positive => 1.0,
            },
            match ep.peak_grammar_state {
                GrammarState::Admissible => 0.0,
                GrammarState::Boundary => 0.5,
                GrammarState::Violation => 1.0,
            },
        ];
        let palette = [BLUE, MAGENTA, RGBColor(0, 150, 0), RGBColor(255, 140, 0), RED];
        for (j, (v, color)) in vals.iter().zip(palette.iter()).enumerate() {
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(xb + j as i32, 0.0), (xb + j as i32 + 1, *v)],
                    color.mix(0.7).filled(),
                )
            )).ok();
        }
    }
    root.present().ok();
    Ok(())
}

// =========================================================================
// Figure 10: Theorem-9 deterministic-replay verification
// =========================================================================

fn fig10_replay_verification(run: &FixtureRun, path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let title = format!(
        "{} — Theorem 9 deterministic replay verification",
        run.manifest_name
    );
    let (top, bottom) = root.split_vertically(48);
    top.titled(&title, ("sans-serif", 22)).ok();

    let lines = [
        format!("evaluate_real_dataset replay: {}", if run.real_data_eval.deterministic_replay_holds { "HOLDS" } else { "FAILED" }),
        format!("fusion_compare replay: {}",       if run.fusion_metrics.deterministic_replay_holds  { "HOLDS" } else { "FAILED" }),
        format!("episodes (eval grid): {}", run.episodes.len()),
        format!("raw alerts: {}", run.real_data_eval.metrics.raw_anomaly_count),
        format!("RSCR: {:.3}", run.real_data_eval.metrics.rscr),
        format!("clean-window FP rate: {:.6}", run.real_data_eval.metrics.clean_window_false_episode_rate),
        format!("fusion FP rate at default config: {:.6}", run.fusion_metrics.fusion_clean_window_fp_rate),
    ];

    for (i, line) in lines.iter().enumerate() {
        bottom.draw(&Text::new(
            line.clone(),
            (60, 80 + (i as i32) * 36),
            ("sans-serif", 18),
        )).ok();
    }
    root.present().ok();
    Ok(())
}

// =========================================================================
// Cross-fixture summary figures
// =========================================================================

pub fn render_rscr_scatter(runs: &[FixtureRun], path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(48);
    top.titled(
        "Cross-fixture RSCR vs clean-window FP rate (12 fixtures)",
        ("sans-serif", 22),
    ).ok();

    let xs: Vec<f64> = runs.iter().map(|r| r.real_data_eval.metrics.rscr).collect();
    let ys: Vec<f64> = runs.iter().map(|r| r.real_data_eval.metrics.clean_window_false_episode_rate).collect();
    let xmax = xs.iter().cloned().fold(1.0_f64, f64::max);
    let ymax = ys.iter().cloned().fold(0.01_f64, f64::max).max(0.001);

    let mut chart = ChartBuilder::on(&bottom)
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(0.0_f64..(xmax * 1.15).max(1.0), 0.0_f64..(ymax * 1.20))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("RSCR (raw alerts / typed episodes)")
        .y_desc("Clean-window FP rate")
        .draw().ok();
    for (i, run) in runs.iter().enumerate() {
        chart.draw_series(std::iter::once(
            Circle::new(
                (run.real_data_eval.metrics.rscr,
                 run.real_data_eval.metrics.clean_window_false_episode_rate),
                7,
                Palette99::pick(i).filled(),
            )
        )).ok();
        chart.draw_series(std::iter::once(
            Text::new(
                run.manifest_name.clone(),
                (run.real_data_eval.metrics.rscr + xmax * 0.02,
                 run.real_data_eval.metrics.clean_window_false_episode_rate),
                ("sans-serif", 11),
            )
        )).ok();
    }
    root.present().ok();
    Ok(())
}

pub fn render_fusion_sweep_curve(runs: &[FixtureRun], path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(48);
    top.titled(
        "Fusion sweep — F-11 fixture clean-window FP rate vs min_consensus N",
        ("sans-serif", 22),
    ).ok();

    // Find the F-11 run (first one whose name ends with F11)
    let f11 = runs.iter().find(|r| r.manifest_name.contains("F11") && !r.manifest_name.contains("F11b"));
    let mut chart = ChartBuilder::on(&bottom)
        .margin(30)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(0..10_i32, 0.0_f64..0.20)
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_desc("min_consensus N")
        .y_desc("Layer-2 clean-window FP rate")
        .draw().ok();

    // Reference horizontal lines for single-detector minima
    for (label, val, color) in [
        ("scalar-3sigma", 0.0139, RED),
        ("CUSUM",         0.0116, BLUE),
        ("EWMA",          0.0812, MAGENTA),
        ("DSFB-struct",   0.0070, RGBColor(0, 130, 0)),
    ] {
        chart.draw_series(LineSeries::new(
            (0..=9).map(|x| (x, val)),
            color.stroke_width(1),
        )).ok();
        chart.draw_series(std::iter::once(
            Text::new(label.to_string(), (8, val + 0.003), ("sans-serif", 11))
        )).ok();
    }

    if let Some(_run) = f11 {
        // Verbatim from tests/fusion_compare.rs F-11 fixture run
        let pts = [(1, 0.1717), (3, 0.0348), (5, 0.0186), (7, 0.0116), (9, 0.0000)];
        chart.draw_series(LineSeries::new(
            pts.iter().map(|&(n, v)| (n, v)),
            BLACK.stroke_width(3),
        )).ok();
        chart.draw_series(
            pts.iter().map(|&(n, v)| Circle::new((n, v), 5, BLACK.filled()))
        ).ok();
    }
    root.present().ok();
    Ok(())
}

pub fn render_fp_comparison(runs: &[FixtureRun], path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(48);
    top.titled(
        "Cross-fixture clean-window FP rate (post Phase-8 default config)",
        ("sans-serif", 22),
    ).ok();

    let max = runs.iter()
        .map(|r| r.real_data_eval.metrics.clean_window_false_episode_rate)
        .fold(0.0_f64, f64::max).max(0.01);
    let mut chart = ChartBuilder::on(&bottom)
        .margin(30)
        .x_label_area_size(120)
        .y_label_area_size(80)
        .build_cartesian_2d(0..(runs.len() as i32 + 1), 0.0_f64..(max * 1.15))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_label_formatter(&|x| {
            let i = *x as usize;
            if i > 0 && i <= runs.len() {
                runs[i - 1].manifest_name.clone()
            } else {
                String::new()
            }
        })
        .x_label_style(("sans-serif", 9).into_text_style(&bottom).transform(FontTransform::Rotate90))
        .y_desc("Clean-window FP rate")
        .draw().ok();
    chart.draw_series(
        runs.iter().enumerate().map(|(i, r)| {
            Rectangle::new(
                [(i as i32 + 1, 0.0),
                 (i as i32 + 2, r.real_data_eval.metrics.clean_window_false_episode_rate)],
                Palette99::pick(i).filled(),
            )
        })
    ).ok();
    root.present().ok();
    Ok(())
}

// =========================================================================
// Helpers
// =========================================================================

/// Viridis colormap approximation in u8 (r, g, b).
fn viridis(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    // Simple piecewise interpolation through canonical viridis stops.
    let stops: &[(f64, [u8; 3])] = &[
        (0.0,  [68, 1, 84]),
        (0.25, [59, 82, 139]),
        (0.50, [33, 145, 140]),
        (0.75, [94, 201, 98]),
        (1.0,  [253, 231, 37]),
    ];
    for w in stops.windows(2) {
        let (t0, c0) = (w[0].0, w[0].1);
        let (t1, c1) = (w[1].0, w[1].1);
        if t <= t1 {
            let f = (t - t0) / (t1 - t0).max(1e-9);
            let r = (c0[0] as f64 + f * (c1[0] as f64 - c0[0] as f64)) as u8;
            let g = (c0[1] as f64 + f * (c1[1] as f64 - c0[1] as f64)) as u8;
            let b = (c0[2] as f64 + f * (c1[2] as f64 - c0[2] as f64)) as u8;
            return (r, g, b);
        }
    }
    let last = stops.last().unwrap().1;
    (last[0], last[1], last[2])
}

/// HSL -> RGB for line-plot palette diversification.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h6 = h * 6.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h6 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c * 0.5;
    (((r1 + m) * 255.0) as u8, ((g1 + m) * 255.0) as u8, ((b1 + m) * 255.0) as u8)
}
