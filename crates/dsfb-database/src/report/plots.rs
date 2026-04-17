//! Plot generation. Produces PNGs that the LaTeX paper includes; figure
//! filenames are stable so `paper/dsfb-database.tex` references them by
//! name and re-runs are bit-comparable in caption text (the bitmap may
//! differ on font availability — the build script uses the bundled
//! plotters TTF).

use crate::grammar::{Episode, MotifClass};
use crate::residual::{ResidualClass, ResidualStream};
use anyhow::Result;
use plotters::prelude::*;
use std::path::Path;

pub fn plot_residual_overlay(
    path: &Path,
    title: &str,
    stream: &ResidualStream,
    class: ResidualClass,
    episodes: &[Episode],
    motif: MotifClass,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1000, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let samples: Vec<(f64, f64)> = stream
        .iter_class(class)
        .map(|s| (s.t, s.value))
        .collect();
    if samples.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        area.draw_text("(no samples)", &TextStyle::from(("sans-serif", 18)), (480, 230))?;
        root.present()?;
        return Ok(());
    }
    let t_min = samples.first().map(|s| s.0).unwrap_or(0.0);
    let t_max = samples.last().map(|s| s.0).unwrap_or(1.0);
    let v_min = samples.iter().map(|s| s.1).fold(f64::INFINITY, f64::min);
    let v_max = samples.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
    let v_pad = ((v_max - v_min).abs() * 0.1).max(0.01);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, (v_min - v_pad)..(v_max + v_pad))?;

    chart
        .configure_mesh()
        .x_desc("t (s)")
        .y_desc(format!("residual ({:?})", class))
        .draw()?;

    chart.draw_series(LineSeries::new(samples, &BLACK))?;

    for ep in episodes.iter().filter(|e| e.motif == motif) {
        let mid = (ep.t_start + ep.t_end) / 2.0;
        chart.draw_series(std::iter::once(Rectangle::new(
            [(ep.t_start, v_min - v_pad), (ep.t_end, v_max + v_pad)],
            RGBAColor(255, 80, 80, 0.18).filled(),
        )))?;
        chart.draw_series(std::iter::once(Circle::new(
            (mid, ep.peak.copysign(1.0)),
            4,
            RED.filled(),
        )))?;
    }

    root.present()?;
    Ok(())
}

pub fn plot_metric_bars(
    path: &Path,
    title: &str,
    bars: &[(String, f64)],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (900, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max = bars.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max).max(0.01);
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(0..bars.len(), 0.0..(max * 1.15))?;
    chart
        .configure_mesh()
        .x_desc("motif")
        .y_desc("value")
        .x_label_formatter(&|i| {
            bars.get(*i).map(|b| b.0.clone()).unwrap_or_default()
        })
        .draw()?;
    chart.draw_series(bars.iter().enumerate().map(|(i, (_, v))| {
        let mut bar = Rectangle::new([(i, 0.0), (i + 1, *v)], BLUE.filled());
        bar.set_margin(0, 0, 12, 12);
        bar
    }))?;
    root.present()?;
    Ok(())
}

/// Per-motif degradation curves under perturbation-magnitude scaling.
///
/// `scales` is the x-axis (perturbation magnitude as a fraction of the
/// canonical seed=42 setting; 1.0 reproduces the published baseline).
/// `series` is one (motif name, F1-per-scale) pair per motif. The plot
/// renders one line per motif so a reviewer can read off the operating
/// envelope: where each motif holds, where it degrades, and where it
/// breaks down. This replaces the uninformative uniform-F1 bar chart.
pub fn plot_stress_curves(
    path: &Path,
    title: &str,
    scales: &[f64],
    series: &[(String, Vec<f64>)],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1000, 560)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = scales.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = scales.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .right_y_label_area_size(20)
        .build_cartesian_2d(x_min..x_max, 0.0_f64..1.05_f64)?;

    chart
        .configure_mesh()
        .x_desc("perturbation magnitude scale (1.0 = published baseline)")
        .y_desc("F1")
        .x_label_formatter(&|v| format!("{:.2}", v))
        .y_label_formatter(&|v| format!("{:.2}", v))
        .draw()?;

    let palette: [RGBColor; 5] = [
        RGBColor(31, 119, 180),
        RGBColor(255, 127, 14),
        RGBColor(44, 160, 44),
        RGBColor(214, 39, 40),
        RGBColor(148, 103, 189),
    ];

    for (i, (name, ys)) in series.iter().enumerate() {
        let color = palette[i % palette.len()];
        let pts: Vec<(f64, f64)> = scales.iter().cloned().zip(ys.iter().cloned()).collect();
        chart
            .draw_series(LineSeries::new(pts.clone(), color.stroke_width(2)))?
            .label(name.clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 18, y)], color.stroke_width(2)));
        chart.draw_series(pts.into_iter().map(|p| Circle::new(p, 4, color.filled())))?;
    }
    chart
        .configure_series_labels()
        .background_style(WHITE.filled())
        .border_style(BLACK)
        .position(SeriesLabelPosition::LowerRight)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Per-motif noise-reduction funnel: raw residual samples → naive
/// slew-threshold crossings → DSFB episodes.
///
/// `rows` is one tuple per motif: (motif name, raw samples in class,
/// naive samples-above-slew-threshold, DSFB episodes emitted). The chart
/// uses a log-scale y-axis with the count printed above each bar so the
/// reader can read the absolute values without decoding the log mapping.
///
/// What the chart shows that a single compression-ratio bar cannot: the
/// *gap between naive thresholding and DSFB* per motif. A flat-threshold
/// alerter would page the operator once per orange bar; DSFB collapses
/// each cluster to the red episode bar. The orange→red ratio is the
/// motif layer's contribution; the blue→orange ratio is just the noise
/// floor of the residual channel.
pub fn plot_pipeline_funnel(
    path: &Path,
    title: &str,
    rows: &[(String, u64, u64, u64)],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 560)).into_drawing_area();
    root.fill(&WHITE)?;

    // Add 1 so log scale handles zero counts gracefully.
    let max = rows
        .iter()
        .flat_map(|(_, r, n, e)| [*r, *n, *e])
        .max()
        .unwrap_or(1)
        .max(1);

    let groups = rows.len();
    let bars_per_group = 3usize;
    let cells = groups * (bars_per_group + 1); // +1 spacer per group

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0..cells, (1f64..(max as f64) * 2.5).log_scale())?;

    chart
        .configure_mesh()
        .x_desc("motif")
        .y_desc("count (log scale)")
        .x_label_formatter(&|i| {
            let group = *i / (bars_per_group + 1);
            let in_group = *i % (bars_per_group + 1);
            // Label centered on middle bar of each triplet.
            if in_group == 1 {
                rows.get(group).map(|r| r.0.clone()).unwrap_or_default()
            } else {
                String::new()
            }
        })
        .draw()?;

    let blue = RGBColor(31, 119, 180);
    let orange = RGBColor(255, 127, 14);
    let red = RGBColor(214, 39, 40);

    for (gi, (_, raw, naive, eps)) in rows.iter().enumerate() {
        let base = gi * (bars_per_group + 1);
        let series: [(u64, RGBColor, &str); 3] = [
            (*raw, blue, "raw samples"),
            (*naive, orange, "naive >slew threshold"),
            (*eps, red, "DSFB episodes"),
        ];
        for (bi, entry) in series.iter().enumerate() {
            let val = entry.0;
            let color = entry.1;
            let label = entry.2;
            let x0 = base + bi;
            let x1 = base + bi + 1;
            let y = (val as f64).max(1.0);
            let mut bar = Rectangle::new([(x0, 1.0), (x1, y)], color.filled());
            bar.set_margin(0, 0, 2, 2);
            // Only attach legend on the first group so the legend is unique.
            if gi == 0 {
                chart
                    .draw_series(std::iter::once(bar))?
                    .label(label)
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 12, y + 5)], color.filled())
                    });
            } else {
                chart.draw_series(std::iter::once(bar))?;
            }
            // Numeric annotation above each bar.
            chart.draw_series(std::iter::once(Text::new(
                format!("{}", val),
                (x0, y * 1.15),
                ("sans-serif", 13).into_font().color(&BLACK),
            )))?;
        }
    }
    chart
        .configure_series_labels()
        .background_style(WHITE.filled())
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Drift--slew phase portrait on a single channel.
///
/// Plots `r_k` (x-axis) against `s_k` (y-axis) for one motif's worth
/// of samples. The slew threshold is drawn as a vertical line, the
/// drift threshold as a horizontal line — together they partition
/// the plane into the three envelope classes the motif state machine
/// reads (Stable, Drift, Boundary). Each `(r_k, s_k)` point is
/// coloured by which class it falls in. Points are connected by a
/// thin gray line in temporal order so the trajectory is visible.
///
/// The figure is named "phase portrait" because that is what it is:
/// a plot of system state in `(r, s)` coordinates. It is **not**
/// a strange attractor and not a claim of dynamical chaos — DSFB is
/// a deterministic fixed-rule observer, not a chaotic system. The
/// figure exists only because the threshold partition is structurally
/// what the motif state machine reads, and showing the partition
/// directly is more legible than the time-series view alone.
pub fn plot_phase_portrait(
    path: &Path,
    title: &str,
    raw: &[(f64, f64)],
    ema: &[(f64, f64)],
    slew_threshold: f64,
    drift_threshold: f64,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (900, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    if raw.is_empty() || ema.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        area.draw_text("(no samples)", &TextStyle::from(("sans-serif", 18)), (440, 360))?;
        root.present()?;
        return Ok(());
    }
    let n = raw.len().min(ema.len());
    let pts: Vec<(f64, f64)> = (0..n).map(|i| (raw[i].1, ema[i].1.abs())).collect();

    let r_min = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let r_max = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let s_max = pts
        .iter()
        .map(|p| p.1)
        .fold(0.0_f64, f64::max)
        .max(drift_threshold)
        * 1.15;
    let r_pad = (r_max - r_min).abs().max(0.01) * 0.10;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d((r_min - r_pad)..(r_max + r_pad), 0.0_f64..s_max)?;

    chart
        .configure_mesh()
        .x_desc("r_k (residual)")
        .y_desc("s_k (EMA |residual|)")
        .draw()?;

    // Trajectory line (gray) in temporal order.
    chart.draw_series(LineSeries::new(
        pts.clone(),
        RGBAColor(120, 120, 120, 0.45).stroke_width(1),
    ))?;

    // Threshold lines.
    let slew_color = RGBColor(214, 39, 40);
    let drift_color = RGBColor(255, 127, 14);
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(slew_threshold, 0.0), (slew_threshold, s_max)],
            slew_color.stroke_width(2),
        )))?
        .label(format!("slew threshold = {:.2}", slew_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], slew_color.stroke_width(2))
        });
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(-slew_threshold, 0.0), (-slew_threshold, s_max)],
            slew_color.stroke_width(2),
        )))?;
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(r_min - r_pad, drift_threshold), (r_max + r_pad, drift_threshold)],
            drift_color.stroke_width(2),
        )))?
        .label(format!("drift threshold = {:.2}", drift_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], drift_color.stroke_width(2))
        });

    // Per-envelope-class point colours: Stable (blue), Drift
    // (orange), Boundary (red). Slew dominates drift when both fire
    // (matches the motif state machine's read order).
    let stable_color = RGBColor(31, 119, 180);
    let drift_pt_color = RGBColor(255, 127, 14);
    let boundary_color = RGBColor(214, 39, 40);

    let mut stable_pts = Vec::new();
    let mut drift_pts = Vec::new();
    let mut boundary_pts = Vec::new();
    for &(r, s) in &pts {
        if r.abs() >= slew_threshold {
            boundary_pts.push((r, s));
        } else if s >= drift_threshold {
            drift_pts.push((r, s));
        } else {
            stable_pts.push((r, s));
        }
    }
    chart
        .draw_series(stable_pts.iter().map(|p| Circle::new(*p, 3, stable_color.filled())))?
        .label("Stable")
        .legend(move |(x, y)| Circle::new((x + 9, y), 4, stable_color.filled()));
    chart
        .draw_series(drift_pts.iter().map(|p| Circle::new(*p, 3, drift_pt_color.filled())))?
        .label("Drift")
        .legend(move |(x, y)| Circle::new((x + 9, y), 4, drift_pt_color.filled()));
    chart
        .draw_series(boundary_pts.iter().map(|p| Circle::new(*p, 3, boundary_color.filled())))?
        .label("Boundary (slew)")
        .legend(move |(x, y)| Circle::new((x + 9, y), 4, boundary_color.filled()));

    chart
        .configure_series_labels()
        .background_style(WHITE.filled())
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Drift-vs-slew anatomy on a single channel.
///
/// Shows what the state machine sees: raw |residual| (gray dots),
/// EMA-smoothed |residual| (blue line), the slew threshold (red
/// dashed) and the drift threshold (orange dashed). The episode the
/// motif emits — if any — is shaded. The point of the figure is to
/// make the difference between "instantaneous boundary breach"
/// (`|r_k| ≥ θ_slew`) and "persistent drift" (`s_k ≥ θ_drift`) visible
/// at a glance, so a reader can see why DSFB does not collapse into
/// a flat-threshold alerter.
pub fn plot_drift_slew_anatomy(
    path: &Path,
    title: &str,
    raw: &[(f64, f64)],
    ema: &[(f64, f64)],
    slew_threshold: f64,
    drift_threshold: f64,
    episode: Option<(f64, f64)>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 520)).into_drawing_area();
    root.fill(&WHITE)?;

    let t_min = raw.first().map(|p| p.0).unwrap_or(0.0);
    let t_max = raw.last().map(|p| p.0).unwrap_or(1.0);
    let v_max = raw
        .iter()
        .chain(ema.iter())
        .map(|p| p.1.abs())
        .fold(0.0_f64, f64::max)
        .max(slew_threshold)
        .max(drift_threshold)
        * 1.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, 0.0_f64..v_max)?;

    chart
        .configure_mesh()
        .x_desc("t (s)")
        .y_desc("|residual|")
        .draw()?;

    if let Some((t0, t1)) = episode {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(t0, 0.0), (t1, v_max)],
            RGBAColor(40, 160, 80, 0.16).filled(),
        )))?;
    }

    let slew_color = RGBColor(214, 39, 40);
    let drift_color = RGBColor(255, 127, 14);
    let raw_color = RGBColor(120, 120, 120);
    let ema_color = RGBColor(31, 119, 180);

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(t_min, slew_threshold), (t_max, slew_threshold)],
            slew_color.stroke_width(2),
        )))?
        .label(format!("slew threshold = {:.2}", slew_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], slew_color.stroke_width(2))
        });

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(t_min, drift_threshold), (t_max, drift_threshold)],
            drift_color.stroke_width(2),
        )))?
        .label(format!("drift threshold = {:.2}", drift_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], drift_color.stroke_width(2))
        });

    let raw_pts: Vec<(f64, f64)> = raw.iter().map(|p| (p.0, p.1.abs())).collect();
    chart
        .draw_series(raw_pts.iter().map(|p| Circle::new(*p, 2, raw_color.filled())))?
        .label("|raw residual|")
        .legend(move |(x, y)| Circle::new((x + 9, y), 3, raw_color.filled()));

    let ema_pts: Vec<(f64, f64)> = ema.iter().map(|p| (p.0, p.1.abs())).collect();
    chart
        .draw_series(LineSeries::new(ema_pts, ema_color.stroke_width(2)))?
        .label("EMA |residual|")
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], ema_color.stroke_width(2))
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.filled())
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    root.present()?;
    Ok(())
}
