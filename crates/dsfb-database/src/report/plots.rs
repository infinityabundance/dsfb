//! Plot generation. Produces PNGs that the LaTeX paper includes; figure
//! filenames are stable so `paper/dsfb-database.tex` references them by
//! name and re-runs are bit-comparable in caption text (the bitmap may
//! differ on font availability — the build script uses the bundled
//! plotters TTF).

use crate::grammar::{Episode, MotifClass};
use crate::residual::{ResidualClass, ResidualStream};
use anyhow::Result;
use plotters::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// small helpers

fn stats_of(values: impl Iterator<Item = f64>) -> (f64, f64, f64, f64) {
    // returns (min, max, mean, std)
    let xs: Vec<f64> = values.collect();
    if xs.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mn = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / xs.len() as f64;
    (mn, mx, mean, var.sqrt())
}

/// Union of a list of [t_start, t_end] intervals. Output is disjoint
/// and sorted by t_start; used to draw episode shading without
/// alpha-stacking when many episodes overlap (e.g. JOB with ~90
/// simultaneous cardinality episodes across query ids — each at alpha
/// 0.10 stacks to effectively opaque red).
fn merge_intervals(intervals: impl IntoIterator<Item = (f64, f64)>) -> Vec<(f64, f64)> {
    let mut xs: Vec<(f64, f64)> = intervals
        .into_iter()
        .filter(|(a, b)| b >= a)
        .collect();
    if xs.is_empty() {
        return Vec::new();
    }
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = Vec::with_capacity(xs.len());
    let (mut cur_a, mut cur_b) = xs[0];
    for (a, b) in xs.into_iter().skip(1) {
        if a <= cur_b {
            if b > cur_b {
                cur_b = b;
            }
        } else {
            out.push((cur_a, cur_b));
            cur_a = a;
            cur_b = b;
        }
    }
    out.push((cur_a, cur_b));
    out
}

/// Pad a [min, max] y-range so that the slew threshold is visible and the
/// line does not sit flush against the frame. Returns a pair with an
/// enforced minimum span — degenerate (all-equal) residuals still produce
/// a plot whose frame is meaningful rather than collapsed to a line.
fn pad_y_range(v_min: f64, v_max: f64, slew: f64) -> (f64, f64) {
    let span = (v_max - v_min).abs();
    let want_span = (slew.abs() * 2.4).max(0.02);
    let (lo, hi) = if span < want_span {
        let mid = (v_min + v_max) * 0.5;
        (mid - want_span * 0.5, mid + want_span * 0.5)
    } else {
        let pad = span * 0.10;
        (v_min - pad, v_max + pad)
    };
    (lo, hi)
}

// ---------------------------------------------------------------------------
// residual overlay (upgraded)

/// Residual + episode overlay for one motif class.
///
/// The figure is designed to be *interpretable at a glance*:
///
///   * the residual trace is plotted in black (line) with dots for sparse
///     data (<600 samples) so a single-warehouse residual does not render
///     as an invisible hairline;
///   * the ±slew and ±drift envelope thresholds are drawn as horizontal
///     reference lines so the reader can see *why* episodes fire;
///   * episode shading uses a fixed-alpha band clamped to the actual data
///     y-range (previous version stacked to v_min−v_pad / v_max+v_pad,
///     which saturated to solid red when many episodes overlapped —
///     see e.g. JOB cardinality);
///   * peak markers live at the true signed `ep.peak`, not
///     `ep.peak.copysign(1.0)` (that collapsed every marker to the top
///     of the axis and silently hid sign information);
///   * a summary stripe at the top right reports episode count, max
///     peak, and stream duration;
///   * degenerate residuals (std ≈ 0) produce a labelled "flat residual
///     (std=X)" overlay rather than a blank chart — this is the honest
///     read on e.g. the Snowset plan_regression channel, where a single
///     warehouse does not change plan.
pub fn plot_residual_overlay(
    path: &Path,
    title: &str,
    stream: &ResidualStream,
    class: ResidualClass,
    episodes: &[Episode],
    motif: MotifClass,
    slew_threshold: f64,
    drift_threshold: f64,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 540)).into_drawing_area();
    root.fill(&WHITE)?;

    let samples: Vec<(f64, f64)> = stream
        .iter_class(class)
        .map(|s| (s.t, s.value))
        .collect();
    let motif_eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();

    if samples.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        area.draw_text(
            "(no samples in this residual class)",
            &TextStyle::from(("sans-serif", 18)),
            (420, 250),
        )?;
        root.present()?;
        return Ok(());
    }

    let (v_min, v_max, _mean, std) = stats_of(samples.iter().map(|s| s.1));
    let t_min = samples.first().map(|s| s.0).unwrap_or(0.0);
    let t_max = samples.last().map(|s| s.0).unwrap_or(t_min + 1.0);

    // Degenerate-residual case: std essentially zero. Render honest
    // labelling rather than a misleading blank plot.
    if std < 1e-9 && motif_eps.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        let line1 = format!(
            "flat residual: N = {}, value = {:.3}, std = {:.2e}",
            samples.len(),
            v_min,
            std,
        );
        let line2 = "(no motif fired -- this channel is structurally stable)".to_string();
        let style = TextStyle::from(("sans-serif", 18)).color(&RGBColor(60, 60, 60));
        area.draw_text(&line1, &style, (60, 210))?;
        area.draw_text(&line2, &style, (60, 240))?;
        root.present()?;
        return Ok(());
    }

    let (y_lo, y_hi) = pad_y_range(v_min, v_max, slew_threshold);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, y_lo..y_hi)?;

    chart
        .configure_mesh()
        .x_desc("t (stream-local seconds)")
        .y_desc(format!("residual ({})", class.name()))
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;

    // Reference envelopes.
    let slew_color = RGBColor(214, 39, 40);
    let drift_color = RGBColor(255, 127, 14);
    for level in [slew_threshold, -slew_threshold] {
        if level > y_lo && level < y_hi {
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(t_min, level), (t_max, level)],
                    slew_color.mix(0.8).stroke_width(1),
                )))?;
        }
    }
    for level in [drift_threshold, -drift_threshold] {
        if level > y_lo && level < y_hi {
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(t_min, level), (t_max, level)],
                    drift_color.mix(0.7).stroke_width(1),
                )))?;
        }
    }

    // Episode shading: union overlapping intervals so stacking does
    // not saturate the background. With 95 overlapping episodes on the
    // JOB cardinality channel the old per-episode draw produced a
    // solid-red frame; the union is drawn once at a single alpha so the
    // residual trace stays readable.
    let merged = merge_intervals(motif_eps.iter().map(|e| (e.t_start, e.t_end)));
    let ep_color = RGBAColor(220, 60, 60, 0.18);
    for (t0, t1) in &merged {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(*t0, y_lo), (*t1, y_hi)],
            ep_color.filled(),
        )))?;
    }

    // Residual trace: line + dots when sparse (sparse residuals appear as
    // near-invisible hairlines without markers — real bundled samples
    // often have <1000 points).
    chart
        .draw_series(LineSeries::new(
            samples.iter().cloned(),
            RGBColor(30, 30, 30).stroke_width(1),
        ))?
        .label(format!("residual ({} samples)", samples.len()))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], RGBColor(30, 30, 30).stroke_width(2))
        });
    if samples.len() < 600 {
        chart.draw_series(
            samples
                .iter()
                .map(|p| Circle::new(*p, 2, RGBColor(30, 30, 30).filled())),
        )?;
    }

    // Peak markers at the true signed peak (not copysign(1.0)).
    // Single draw_series call so the legend entry has exactly one
    // visible marker in-chart (previous per-episode loop + offscreen
    // sentinel produced a ghost marker clipped to the left frame).
    let peak_color = RGBColor(200, 20, 40);
    let peak_pts: Vec<(f64, f64)> = motif_eps
        .iter()
        .map(|ep| {
            let mid = (ep.t_start + ep.t_end) * 0.5;
            let y = ep.peak.clamp(y_lo, y_hi);
            (mid, y)
        })
        .collect();
    if !peak_pts.is_empty() {
        chart
            .draw_series(peak_pts.iter().map(|p| Circle::new(*p, 4, peak_color.filled())))?
            .label(format!("episode peak ({})", motif_eps.len()))
            .legend(move |(x, y)| Circle::new((x + 9, y), 4, peak_color.filled()));
    }
    // Threshold-line legend entries. Re-draw as a (t_min, y)..(t_min, y)
    // zero-length element so the path is not visible in-chart; the
    // legend swatch still gets drawn by plotters' series_labels logic.
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(t_min, slew_threshold), (t_min, slew_threshold)],
            slew_color.stroke_width(2),
        )))?
        .label(format!("+/-slew = {:.2}", slew_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], slew_color.stroke_width(2))
        });
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(t_min, drift_threshold), (t_min, drift_threshold)],
            drift_color.stroke_width(2),
        )))?
        .label(format!("+/-drift = {:.2}", drift_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], drift_color.stroke_width(2))
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.85).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    // Corner summary strip (plot-area-absolute text).
    let max_peak = motif_eps.iter().map(|e| e.peak.abs()).fold(0.0_f64, f64::max);
    let summary = format!(
        "episodes = {}   max |peak| = {:.3}   std = {:.3}   duration = {:.1}s",
        motif_eps.len(),
        max_peak,
        std,
        t_max - t_min,
    );
    root.draw_text(
        &summary,
        &TextStyle::from(("sans-serif", 13)).color(&RGBColor(60, 60, 60)),
        (20, 508),
    )?;

    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// NEW: per-channel small multiples

/// Per-channel small-multiples residual strip plot.
///
/// Picks the top `max_rows` channels by `|residual|` standard deviation
/// (i.e. the channels that *moved* the most) and lays them out one per
/// row. Each row shares the global x-axis (time) and uses a local y-axis
/// to keep small signals visible. Episode windows for that channel are
/// shaded per row.
///
/// This addresses the problem that a single-line overlay plot collapses
/// per-channel structure: for e.g. JOB with 113 query ids, or Snowset
/// with many warehouses, the reader cannot tell *which* channel is
/// driving the episode stream. The small-multiples layout makes that
/// explicit without pretending there is a single global y-axis scale.
pub fn plot_channel_small_multiples(
    path: &Path,
    title: &str,
    stream: &ResidualStream,
    class: ResidualClass,
    episodes: &[Episode],
    motif: MotifClass,
    max_rows: usize,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Group samples by channel; channels with fewer than 3 samples or
    // std = 0 are dropped (no information to display).
    let mut by_chan: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for s in stream.iter_class(class) {
        let ch = s.channel.clone().unwrap_or_else(|| "(none)".to_string());
        by_chan.entry(ch).or_default().push((s.t, s.value));
    }

    let mut ranked: Vec<(String, Vec<(f64, f64)>, f64)> = by_chan
        .into_iter()
        .filter(|(_, v)| v.len() >= 3)
        .map(|(k, v)| {
            let (_, _, _, std) = stats_of(v.iter().map(|p| p.1));
            (k, v, std)
        })
        .filter(|(_, _, std)| *std > 0.0)
        .collect();
    ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let ranked: Vec<_> = ranked.into_iter().take(max_rows).collect();

    let height_per_row: u32 = 120;
    let width: u32 = 1100;
    let header: u32 = 64;
    let footer: u32 = 32;
    let n = ranked.len().max(1) as u32;
    let total_h = header + n * height_per_row + footer;

    let root = BitMapBackend::new(path, (width, total_h)).into_drawing_area();
    root.fill(&WHITE)?;

    if ranked.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        area.draw_text(
            "(no channels with non-degenerate residuals)",
            &TextStyle::from(("sans-serif", 18)),
            (340, 180),
        )?;
        root.present()?;
        return Ok(());
    }

    // Global time extent, so every row shares the same x-axis.
    let t_min = ranked
        .iter()
        .filter_map(|(_, v, _)| v.first().map(|p| p.0))
        .fold(f64::INFINITY, f64::min);
    let t_max = ranked
        .iter()
        .filter_map(|(_, v, _)| v.last().map(|p| p.0))
        .fold(f64::NEG_INFINITY, f64::max);

    let total_channels = stream
        .iter_class(class)
        .map(|s| s.channel.as_deref().unwrap_or(""))
        .collect::<std::collections::HashSet<_>>()
        .len();

    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 22)).color(&BLACK),
        (20, 16),
    )?;
    root.draw_text(
        &format!(
            "top {} channels by residual std (of {} total) — each row has its own y-axis",
            ranked.len(),
            total_channels,
        ),
        &TextStyle::from(("sans-serif", 12)).color(&RGBColor(80, 80, 80)),
        (20, 44),
    )?;

    // Split the drawing area: header strip, body (rows), footer.
    let (_hdr_area, below) = root.split_vertically(header);
    let (body, _ftr_area) = below.split_vertically(n * height_per_row);
    let rows = body.split_evenly((ranked.len(), 1));
    let last_idx = ranked.len() - 1;

    for (i, ((chan, pts, std), row_area)) in ranked.iter().zip(rows.iter()).enumerate() {
        let (v_min, v_max, _, _) = stats_of(pts.iter().map(|p| p.1));
        let pad = ((v_max - v_min).abs() * 0.15).max(0.01);
        let y_lo = v_min - pad;
        let y_hi = v_max + pad;

        let mut chart = ChartBuilder::on(row_area)
            .margin_left(110)
            .margin_right(20)
            .margin_top(10)
            .margin_bottom(6)
            .x_label_area_size(if i == last_idx { 24 } else { 0 })
            .y_label_area_size(46)
            .build_cartesian_2d(t_min..t_max, y_lo..y_hi)?;
        chart
            .configure_mesh()
            .disable_x_mesh()
            .y_labels(3)
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc(if i == last_idx { "t (stream-local seconds)" } else { "" })
            .label_style(("sans-serif", 11))
            .light_line_style(RGBAColor(220, 220, 220, 0.3))
            .draw()?;

        // Per-channel episode shading; union to avoid alpha-stacking.
        let per_chan: Vec<(f64, f64)> = episodes
            .iter()
            .filter(|e| e.motif == motif && e.channel.as_deref() == Some(chan.as_str()))
            .map(|e| (e.t_start, e.t_end))
            .collect();
        let merged = merge_intervals(per_chan);
        let ep_color = RGBAColor(220, 60, 60, 0.20);
        for (t0, t1) in &merged {
            chart.draw_series(std::iter::once(Rectangle::new(
                [(*t0, y_lo), (*t1, y_hi)],
                ep_color.filled(),
            )))?;
        }

        chart.draw_series(LineSeries::new(
            pts.iter().cloned(),
            RGBColor(30, 30, 30).stroke_width(1),
        ))?;
        if pts.len() < 300 {
            chart.draw_series(
                pts.iter()
                    .map(|p| Circle::new(*p, 1, RGBColor(30, 30, 30).filled())),
            )?;
        }

        // Left-gutter label drawn on the row drawing area.
        let shown = if chan.len() > 18 {
            format!("{}…", &chan[..17])
        } else {
            chan.clone()
        };
        row_area.draw_text(
            &shown,
            &TextStyle::from(("sans-serif", 12)).color(&BLACK),
            (6, 18),
        )?;
        row_area.draw_text(
            &format!("σ={:.3}", std),
            &TextStyle::from(("sans-serif", 10)).color(&RGBColor(100, 100, 100)),
            (6, 38),
        )?;
        let ep_ct = episodes
            .iter()
            .filter(|e| e.motif == motif && e.channel.as_deref() == Some(chan.as_str()))
            .count();
        row_area.draw_text(
            &format!("episodes: {}", ep_ct),
            &TextStyle::from(("sans-serif", 10)).color(&RGBColor(100, 100, 100)),
            (6, 56),
        )?;
    }

    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// NEW: episode distribution (peak + duration)

/// Two-panel distribution figure for a single motif's episodes:
/// log-spaced histogram of peak magnitudes and duration histogram.
///
/// Purpose: summarises the episode stream in a way a reader can absorb
/// without reading the CSV. Reveals whether episodes are dominated by a
/// handful of high-peak events or by many small near-threshold ones
/// — operationally very different regimes.
pub fn plot_episode_distribution(
    path: &Path,
    title: &str,
    episodes: &[Episode],
    motif: MotifClass,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();
    if eps.is_empty() {
        let area = root.titled(title, ("sans-serif", 22))?;
        area.draw_text(
            "(no episodes emitted for this motif on this stream)",
            &TextStyle::from(("sans-serif", 18)).color(&RGBColor(80, 80, 80)),
            (320, 220),
        )?;
        root.present()?;
        return Ok(());
    }

    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 22)).color(&BLACK),
        (20, 14),
    )?;

    let (left, right) = root.margin(50, 30, 20, 20).split_horizontally(530);

    // --- left: peak magnitude histogram (log-spaced bins) ---
    let peaks: Vec<f64> = eps.iter().map(|e| e.peak.abs()).collect();
    let pmin = peaks.iter().cloned().fold(f64::INFINITY, f64::min).max(1e-3);
    let pmax = peaks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bins = 12usize;
    let log_lo = pmin.ln();
    let log_hi = pmax.ln().max(log_lo + 0.1);
    let step = (log_hi - log_lo) / bins as f64;
    let mut counts = vec![0u32; bins];
    for p in &peaks {
        let idx = (((p.ln() - log_lo) / step).floor() as isize)
            .clamp(0, bins as isize - 1) as usize;
        counts[idx] += 1;
    }
    let max_count = *counts.iter().max().unwrap_or(&1) as u32;
    let mut lc = ChartBuilder::on(&left)
        .caption("peak |residual| distribution", ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(46)
        .build_cartesian_2d(0..bins, 0u32..(max_count + 1).max(2))?;
    lc.configure_mesh()
        .x_desc("bin (log-scaled |peak|)")
        .y_desc("episode count")
        .x_label_formatter(&|i| {
            let low = (log_lo + step * (*i as f64)).exp();
            format!("{:.2}", low)
        })
        .label_style(("sans-serif", 11))
        .draw()?;
    let blue = RGBColor(31, 119, 180);
    lc.draw_series(counts.iter().enumerate().map(|(i, c)| {
        let mut bar = Rectangle::new([(i, 0), (i + 1, *c)], blue.filled());
        bar.set_margin(0, 0, 2, 2);
        bar
    }))?;

    // --- right: duration histogram (linear bins, seconds) ---
    let durs: Vec<f64> = eps.iter().map(|e| (e.t_end - e.t_start).max(0.0)).collect();
    let dmin = 0.0;
    let dmax = durs.iter().cloned().fold(0.0_f64, f64::max).max(0.1);
    let step_d = (dmax - dmin) / bins as f64;
    let mut counts_d = vec![0u32; bins];
    for d in &durs {
        let idx = if step_d > 0.0 {
            (((d - dmin) / step_d).floor() as isize).clamp(0, bins as isize - 1) as usize
        } else {
            0
        };
        counts_d[idx] += 1;
    }
    let max_count_d = *counts_d.iter().max().unwrap_or(&1) as u32;
    let mut rc = ChartBuilder::on(&right)
        .caption("episode duration distribution", ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(46)
        .build_cartesian_2d(0..bins, 0u32..(max_count_d + 1).max(2))?;
    rc.configure_mesh()
        .x_desc("bin (seconds)")
        .y_desc("episode count")
        .x_label_formatter(&|i| format!("{:.1}", dmin + step_d * (*i as f64)))
        .label_style(("sans-serif", 11))
        .draw()?;
    let green = RGBColor(44, 160, 44);
    rc.draw_series(counts_d.iter().enumerate().map(|(i, c)| {
        let mut bar = Rectangle::new([(i, 0), (i + 1, *c)], green.filled());
        bar.set_margin(0, 0, 2, 2);
        bar
    }))?;

    let peaks_sum = format!(
        "N = {}   min|peak| = {:.3}   max|peak| = {:.3}   min dur = {:.2}s   max dur = {:.2}s",
        eps.len(),
        pmin,
        pmax,
        durs.iter().cloned().fold(f64::INFINITY, f64::min),
        dmax,
    );
    root.draw_text(
        &peaks_sum,
        &TextStyle::from(("sans-serif", 12)).color(&RGBColor(80, 80, 80)),
        (20, 454),
    )?;

    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// NEW: episode summary table as PNG

/// Renders an N×M episode-count summary matrix as a PNG (rows = motif,
/// columns = "total" plus the top-K channels by episode count across
/// motifs). Cell values are episode counts; colour intensity is
/// log-scaled so a single outlier channel does not wash out the rest.
///
/// Intent: give the reader a one-figure read of the shape of the
/// episode stream across an entire dataset's output, without scrolling
/// a CSV.
pub fn plot_episode_summary_table(
    path: &Path,
    title: &str,
    episodes: &[Episode],
    top_k_channels: usize,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Rank channels by total episode count.
    let mut chan_counts: BTreeMap<String, u32> = BTreeMap::new();
    for e in episodes {
        let ch = e.channel.clone().unwrap_or_else(|| "(none)".to_string());
        *chan_counts.entry(ch).or_default() += 1;
    }
    let mut chans: Vec<(String, u32)> = chan_counts.into_iter().collect();
    chans.sort_by(|a, b| b.1.cmp(&a.1));
    let chans: Vec<String> = chans.into_iter().take(top_k_channels).map(|(c, _)| c).collect();

    let motifs = MotifClass::ALL;
    let n_rows = motifs.len();
    let n_cols = 1 + chans.len(); // "total" + top channels

    // Cell size + margins
    let cell_w: u32 = 110;
    let cell_h: u32 = 42;
    let left_label_w: u32 = 250;
    let header_h: u32 = 100;
    let footer_h: u32 = 40;
    let w = left_label_w + cell_w * n_cols as u32 + 20;
    let h = header_h + cell_h * n_rows as u32 + footer_h;

    let root = BitMapBackend::new(path, (w, h)).into_drawing_area();
    root.fill(&WHITE)?;

    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 20)).color(&BLACK),
        (16, 14),
    )?;

    // Column headers
    let header_text = TextStyle::from(("sans-serif", 12)).color(&BLACK);
    let header_small = TextStyle::from(("sans-serif", 11)).color(&RGBColor(70, 70, 70));
    root.draw_text(
        "motif (row) / channel (col)",
        &header_text,
        (14, 58),
    )?;
    let col_labels: Vec<String> = std::iter::once("total".to_string())
        .chain(chans.iter().cloned())
        .collect();
    for (j, lbl) in col_labels.iter().enumerate() {
        let x = left_label_w + (j as u32) * cell_w + 8;
        let y = 52;
        let shown: String = if lbl.len() > 14 {
            format!("{}...", &lbl[..13])
        } else {
            lbl.clone()
        };
        root.draw_text(
            &shown,
            &header_small,
            (x as i32, y),
        )?;
    }

    // Total max for colour scaling
    let per_cell = |motif: MotifClass, chan: Option<&str>| -> u32 {
        episodes
            .iter()
            .filter(|e| e.motif == motif)
            .filter(|e| match chan {
                None => true,
                Some(c) => e.channel.as_deref() == Some(c),
            })
            .count() as u32
    };

    let mut max_cell = 1u32;
    for m in motifs {
        max_cell = max_cell.max(per_cell(m, None));
        for c in &chans {
            max_cell = max_cell.max(per_cell(m, Some(c)));
        }
    }

    // Draw rows
    for (i, m) in motifs.iter().enumerate() {
        let y = header_h + (i as u32) * cell_h;
        // row label
        root.draw_text(
            m.name(),
            &TextStyle::from(("sans-serif", 12)).color(&BLACK),
            (14, y as i32 + 16),
        )?;

        // cells
        for (j, lbl) in col_labels.iter().enumerate() {
            let x = left_label_w + (j as u32) * cell_w;
            let count = if j == 0 {
                per_cell(*m, None)
            } else {
                per_cell(*m, Some(lbl.as_str()))
            };
            // Log-scaled color intensity.
            let frac = if count == 0 {
                0.0
            } else {
                ((count as f64).ln_1p() / (max_cell as f64).ln_1p()).clamp(0.0, 1.0)
            };
            let fill = RGBColor(
                (255.0 - frac * 210.0) as u8,
                (255.0 - frac * 120.0) as u8,
                (255.0 - frac * 40.0) as u8,
            );
            root.draw(&Rectangle::new(
                [(x as i32, y as i32), ((x + cell_w) as i32, (y + cell_h) as i32)],
                fill.filled(),
            ))?;
            root.draw(&Rectangle::new(
                [(x as i32, y as i32), ((x + cell_w) as i32, (y + cell_h) as i32)],
                RGBColor(180, 180, 180).stroke_width(1),
            ))?;
            let text_color = if frac > 0.55 { WHITE } else { BLACK };
            root.draw_text(
                &format!("{}", count),
                &TextStyle::from(("sans-serif", 13)).color(&text_color),
                (x as i32 + 18, y as i32 + 14),
            )?;
        }
    }

    root.draw_text(
        &format!(
            "cells = episode count per (motif, channel); colour scales as log(1+count); total cap = {}",
            max_cell
        ),
        &TextStyle::from(("sans-serif", 11)).color(&RGBColor(80, 80, 80)),
        (14, (h - 20) as i32),
    )?;

    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// unchanged below

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
