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
    let mut xs: Vec<(f64, f64)> = intervals.into_iter().filter(|(a, b)| b >= a).collect();
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

/// Display-layer humanisation of channel identifier strings. Adapter
/// channel IDs (stored in `ResidualSample.channel` / `Episode.channel`)
/// feed the pinned episode fingerprints, so we never rewrite them at
/// source; this helper is called *only* by plot label code.
///
/// Pass-through: short strings (≤20 chars) that contain any non-digit
/// character (e.g. `"1a"`, `"q3#sp5"`, `"ord[0-199]"`, `"wh_a/q1"`) are
/// returned unchanged.
///
/// Shortened: pure-digit strings of any length (e.g. Snowset's
/// `warehouseId` cast as `"7891774171123969…"`) or anything longer than
/// 20 characters collapse to a stable 6-char base36 tag derived from
/// SHA-256 over the raw bytes, rendered as `id@xxxxxx`. The hash is
/// deterministic across runs so regenerated figures carry identical
/// labels.
fn humanize_channel_label(raw: &str) -> String {
    let needs_hash = raw.len() > 20 || raw.chars().all(|c| c.is_ascii_digit());
    if !needs_hash {
        return raw.to_string();
    }
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(raw.as_bytes());
    // Fold the first 6 bytes into a u64 and render as base36 (0-9a-z).
    let mut acc: u64 = 0;
    for b in &digest[..6] {
        acc = (acc << 8) | (*b as u64);
    }
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut tag = [b'0'; 6];
    for slot in tag.iter_mut().rev() {
        *slot = ALPHABET[(acc % 36) as usize];
        acc /= 36;
    }
    let tag_str = std::str::from_utf8(&tag).unwrap_or("??????");
    format!("id@{}", tag_str)
}

/// True when a residual channel is pinned at a structural cap — in
/// practice, JSD saturates at 1.0 when the compared skeleton histograms
/// are fully disjoint. When this is the case, the residual overlay's
/// merged-interval shading collapses to a solid block (because every
/// sample is at the cap and every episode spans the full axis), which
/// the eye reads as "red rectangle, no signal". Detecting saturation
/// lets the overlay switch to per-episode tick marks so the episode
/// *onsets* remain legible and the reader understands the shape is a
/// property of the data, not the plot.
fn residual_is_saturated(values: &[f64], cap: f64) -> bool {
    if values.len() < 3 {
        return false;
    }
    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    for v in values {
        if *v < mn {
            mn = *v;
        }
        if *v > mx {
            mx = *v;
        }
    }
    (mx - mn).abs() < 1e-6 && (mx - cap).abs() < 1e-6
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
#[allow(clippy::too_many_arguments)]
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
    debug_assert!(slew_threshold.is_finite() && slew_threshold > 0.0);
    debug_assert!(drift_threshold.is_finite() && drift_threshold > 0.0);

    let root = BitMapBackend::new(path, (1100, 540)).into_drawing_area();
    root.fill(&WHITE)?;

    let samples: Vec<(f64, f64)> = stream.iter_class(class).map(|s| (s.t, s.value)).collect();
    let motif_eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();

    if samples.is_empty() {
        draw_empty_overlay_placeholder(&root, title)?;
        root.present()?;
        return Ok(());
    }

    let (v_min, v_max, _mean, std) = stats_of(samples.iter().map(|s| s.1));
    let t_min = samples.first().map(|s| s.0).unwrap_or(0.0);
    let t_max = samples.last().map(|s| s.0).unwrap_or(t_min + 1.0);
    debug_assert!(
        t_max >= t_min,
        "sorted-sample invariant required by overlay"
    );

    if std < 1e-9 && motif_eps.is_empty() {
        draw_flat_residual_placeholder(&root, title, samples.len(), v_min, std)?;
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

    let slew_color = RGBColor(214, 39, 40);
    let drift_color = RGBColor(255, 127, 14);
    draw_threshold_reference_lines(
        &mut chart,
        t_min,
        t_max,
        y_lo,
        y_hi,
        slew_threshold,
        slew_color,
        drift_threshold,
        drift_color,
    )?;

    let sample_values: Vec<f64> = samples.iter().map(|s| s.1).collect();
    let saturated =
        class == ResidualClass::WorkloadPhase && residual_is_saturated(&sample_values, 1.0);
    if saturated {
        draw_episode_ticks_saturated(&root, &mut chart, &motif_eps, y_lo, y_hi)?;
    } else {
        draw_episode_bands_merged(&mut chart, &motif_eps, y_lo, y_hi)?;
    }

    draw_residual_trace_and_peaks(&mut chart, &samples, &motif_eps, y_lo, y_hi)?;
    draw_threshold_legend_entries(
        &mut chart,
        t_min,
        slew_threshold,
        slew_color,
        drift_threshold,
        drift_color,
    )?;
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.85).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;
    draw_overlay_summary(&root, &motif_eps, std, t_max - t_min)?;
    root.present()?;
    Ok(())
}

fn draw_empty_overlay_placeholder<D: DrawingBackend>(
    root: &DrawingArea<D, plotters::coord::Shift>,
    title: &str,
) -> Result<()>
where
    D::ErrorType: 'static,
{
    let area = root.titled(title, ("sans-serif", 22))?;
    area.draw_text(
        "(no samples in this residual class)",
        &TextStyle::from(("sans-serif", 18)),
        (420, 250),
    )?;
    Ok(())
}

fn draw_flat_residual_placeholder<D: DrawingBackend>(
    root: &DrawingArea<D, plotters::coord::Shift>,
    title: &str,
    n: usize,
    value: f64,
    std: f64,
) -> Result<()>
where
    D::ErrorType: 'static,
{
    let area = root.titled(title, ("sans-serif", 22))?;
    let style = TextStyle::from(("sans-serif", 18)).color(&RGBColor(60, 60, 60));
    area.draw_text(
        &format!(
            "flat residual: N = {}, value = {:.3}, std = {:.2e}",
            n, value, std
        ),
        &style,
        (60, 210),
    )?;
    area.draw_text(
        "(no motif fired -- this channel is structurally stable)",
        &style,
        (60, 240),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_threshold_reference_lines<DB>(
    chart: &mut ChartContext<
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    t_min: f64,
    t_max: f64,
    y_lo: f64,
    y_hi: f64,
    slew_threshold: f64,
    slew_color: RGBColor,
    drift_threshold: f64,
    drift_color: RGBColor,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    for level in [slew_threshold, -slew_threshold] {
        if level > y_lo && level < y_hi {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(t_min, level), (t_max, level)],
                slew_color.mix(0.8).stroke_width(1),
            )))?;
        }
    }
    for level in [drift_threshold, -drift_threshold] {
        if level > y_lo && level < y_hi {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(t_min, level), (t_max, level)],
                drift_color.mix(0.7).stroke_width(1),
            )))?;
        }
    }
    Ok(())
}

fn draw_episode_ticks_saturated<DB>(
    root: &DrawingArea<DB, plotters::coord::Shift>,
    chart: &mut ChartContext<
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    motif_eps: &[&Episode],
    y_lo: f64,
    y_hi: f64,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let onset_color = RGBAColor(220, 60, 60, 0.80);
    let close_color = RGBAColor(220, 60, 60, 0.35);
    for ep in motif_eps {
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(ep.t_start, y_lo), (ep.t_start, y_hi)],
            onset_color.stroke_width(2),
        )))?;
        if (ep.t_end - ep.t_start).abs() > f64::EPSILON {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(ep.t_end, y_lo), (ep.t_end, y_hi)],
                close_color.stroke_width(1),
            )))?;
        }
    }
    root.draw_text(
        "JSD saturated at 1.0 (fully disjoint skeleton histograms) -- ticks = episode onset/close",
        &TextStyle::from(("sans-serif", 12)).color(&RGBColor(160, 30, 30)),
        (450, 44),
    )?;
    Ok(())
}

fn draw_episode_bands_merged<DB>(
    chart: &mut ChartContext<
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    motif_eps: &[&Episode],
    y_lo: f64,
    y_hi: f64,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let merged = merge_intervals(motif_eps.iter().map(|e| (e.t_start, e.t_end)));
    let ep_color = RGBAColor(220, 60, 60, 0.18);
    for (t0, t1) in &merged {
        chart.draw_series(std::iter::once(Rectangle::new(
            [(*t0, y_lo), (*t1, y_hi)],
            ep_color.filled(),
        )))?;
    }
    Ok(())
}

fn draw_residual_trace_and_peaks<DB>(
    chart: &mut ChartContext<
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    samples: &[(f64, f64)],
    motif_eps: &[&Episode],
    y_lo: f64,
    y_hi: f64,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    chart
        .draw_series(LineSeries::new(
            samples.iter().cloned(),
            RGBColor(30, 30, 30).stroke_width(1),
        ))?
        .label(format!("residual ({} samples)", samples.len()))
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 18, y)],
                RGBColor(30, 30, 30).stroke_width(2),
            )
        });
    if samples.len() < 600 {
        chart.draw_series(
            samples
                .iter()
                .map(|p| Circle::new(*p, 2, RGBColor(30, 30, 30).filled())),
        )?;
    }
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
            .draw_series(
                peak_pts
                    .iter()
                    .map(|p| Circle::new(*p, 4, peak_color.filled())),
            )?
            .label(format!("episode peak ({})", motif_eps.len()))
            .legend(move |(x, y)| Circle::new((x + 9, y), 4, peak_color.filled()));
    }
    Ok(())
}

fn draw_threshold_legend_entries<DB>(
    chart: &mut ChartContext<
        DB,
        plotters::coord::cartesian::Cartesian2d<
            plotters::coord::types::RangedCoordf64,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    t_min: f64,
    slew_threshold: f64,
    slew_color: RGBColor,
    drift_threshold: f64,
    drift_color: RGBColor,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
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
    Ok(())
}

fn draw_overlay_summary<D: DrawingBackend>(
    root: &DrawingArea<D, plotters::coord::Shift>,
    motif_eps: &[&Episode],
    std: f64,
    duration: f64,
) -> Result<()>
where
    D::ErrorType: 'static,
{
    let max_peak = motif_eps
        .iter()
        .map(|e| e.peak.abs())
        .fold(0.0_f64, f64::max);
    let summary = format!(
        "episodes = {}   max |peak| = {:.3}   std = {:.3}   duration = {:.1}s",
        motif_eps.len(),
        max_peak,
        std,
        duration,
    );
    root.draw_text(
        &summary,
        &TextStyle::from(("sans-serif", 13)).color(&RGBColor(60, 60, 60)),
        (20, 508),
    )?;
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
) -> Result<bool> {
    let ranked = rank_channels_by_std(stream, class, max_rows);
    debug_assert!(ranked.len() <= max_rows, "ranker must respect the cap");
    if ranked.is_empty() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let height_per_row: u32 = 120;
    let width: u32 = 1100;
    let header: u32 = 64;
    let footer: u32 = 32;
    let n = ranked.len() as u32;
    let total_h = header + n * height_per_row + footer;
    let root = BitMapBackend::new(path, (width, total_h)).into_drawing_area();
    root.fill(&WHITE)?;

    let (t_min, t_max) = global_time_range(&ranked);
    debug_assert!(
        t_max >= t_min || !t_min.is_finite(),
        "non-degenerate global range expected once >=1 channel was ranked"
    );
    draw_small_multiples_header(
        &root,
        title,
        ranked.len(),
        total_channel_count(stream, class),
    )?;

    let (_hdr_area, below) = root.split_vertically(header);
    let (body, _ftr_area) = below.split_vertically(n * height_per_row);
    let rows = body.split_evenly((ranked.len(), 1));
    let last_idx = ranked.len() - 1;

    for (i, ((chan, pts, std), row_area)) in ranked.iter().zip(rows.iter()).enumerate() {
        draw_small_multiples_row(
            row_area,
            chan,
            pts,
            *std,
            t_min,
            t_max,
            episodes,
            motif,
            i == last_idx,
        )?;
    }

    root.present()?;
    Ok(true)
}

/// (channel name, residual samples as `(t, value)`, per-channel std).
/// Internal alias used by the small-multiples ranker + renderer.
type RankedChannel = (String, Vec<(f64, f64)>, f64);

/// Group samples by channel, drop channels with fewer than 3 samples
/// or std=0, and return the top `max_rows` ranked by descending std.
fn rank_channels_by_std(
    stream: &ResidualStream,
    class: ResidualClass,
    max_rows: usize,
) -> Vec<RankedChannel> {
    let mut by_chan: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for s in stream.iter_class(class) {
        let ch = s.channel.clone().unwrap_or_else(|| "(none)".to_string());
        by_chan.entry(ch).or_default().push((s.t, s.value));
    }
    let mut ranked: Vec<RankedChannel> = by_chan
        .into_iter()
        .filter(|(_, v)| v.len() >= 3)
        .map(|(k, v)| {
            let (_, _, _, std) = stats_of(v.iter().map(|p| p.1));
            (k, v, std)
        })
        .filter(|(_, _, std)| *std > 0.0)
        .collect();
    ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    ranked.into_iter().take(max_rows).collect()
}

/// Global (t_min, t_max) across all ranked channels so every row
/// shares the same x-axis.
fn global_time_range(ranked: &[RankedChannel]) -> (f64, f64) {
    let t_min = ranked
        .iter()
        .filter_map(|(_, v, _)| v.first().map(|p| p.0))
        .fold(f64::INFINITY, f64::min);
    let t_max = ranked
        .iter()
        .filter_map(|(_, v, _)| v.last().map(|p| p.0))
        .fold(f64::NEG_INFINITY, f64::max);
    (t_min, t_max)
}

fn total_channel_count(stream: &ResidualStream, class: ResidualClass) -> usize {
    stream
        .iter_class(class)
        .map(|s| s.channel.as_deref().unwrap_or(""))
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn draw_small_multiples_header<D: DrawingBackend>(
    root: &DrawingArea<D, plotters::coord::Shift>,
    title: &str,
    shown: usize,
    total_channels: usize,
) -> Result<()>
where
    D::ErrorType: 'static,
{
    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 22)).color(&BLACK),
        (20, 16),
    )?;
    root.draw_text(
        &format!(
            "top {} channels by residual std (of {} total) — each row has its own y-axis",
            shown, total_channels,
        ),
        &TextStyle::from(("sans-serif", 12)).color(&RGBColor(80, 80, 80)),
        (20, 44),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_small_multiples_row<D: DrawingBackend>(
    row_area: &DrawingArea<D, plotters::coord::Shift>,
    chan: &str,
    pts: &[(f64, f64)],
    std: f64,
    t_min: f64,
    t_max: f64,
    episodes: &[Episode],
    motif: MotifClass,
    is_last: bool,
) -> Result<()>
where
    D::ErrorType: 'static,
{
    let (v_min, v_max, _, _) = stats_of(pts.iter().map(|p| p.1));
    let pad = ((v_max - v_min).abs() * 0.15).max(0.01);
    let y_lo = v_min - pad;
    let y_hi = v_max + pad;

    let mut chart = ChartBuilder::on(row_area)
        .margin_left(110)
        .margin_right(20)
        .margin_top(10)
        .margin_bottom(6)
        .x_label_area_size(if is_last { 24 } else { 0 })
        .y_label_area_size(46)
        .build_cartesian_2d(t_min..t_max, y_lo..y_hi)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .y_labels(3)
        .y_label_formatter(&|v| format!("{:.2}", v))
        .x_desc(if is_last {
            "t (stream-local seconds)"
        } else {
            ""
        })
        .label_style(("sans-serif", 11))
        .light_line_style(RGBAColor(220, 220, 220, 0.3))
        .draw()?;

    let per_chan: Vec<(f64, f64)> = episodes
        .iter()
        .filter(|e| e.motif == motif && e.channel.as_deref() == Some(chan))
        .map(|e| (e.t_start, e.t_end))
        .collect();
    let ep_ct = per_chan.len();
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

    let shown = humanize_channel_label(chan);
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
    row_area.draw_text(
        &format!("episodes: {}", ep_ct),
        &TextStyle::from(("sans-serif", 10)).color(&RGBColor(100, 100, 100)),
        (6, 56),
    )?;
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
) -> Result<bool> {
    let eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();

    // Gate: below five episodes, or when both peak magnitudes and
    // durations are constant, a 12-bin histogram collapses to a single
    // bar per panel. Refuse to emit and let the caller fall back to
    // `plot_episode_table`, which actually conveys per-episode data.
    if eps.len() < 5 {
        return Ok(false);
    }
    let peaks_all: Vec<f64> = eps.iter().map(|e| e.peak.abs()).collect();
    let durs_all: Vec<f64> = eps.iter().map(|e| (e.t_end - e.t_start).max(0.0)).collect();
    let (_, _, _, peaks_std) = stats_of(peaks_all.iter().cloned());
    let (_, _, _, durs_std) = stats_of(durs_all.iter().cloned());
    if peaks_std < 1e-9 && durs_std < 1e-9 {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 22)).color(&BLACK),
        (20, 14),
    )?;

    let (left, right) = root.margin(50, 30, 20, 20).split_horizontally(530);

    // --- left: peak magnitude histogram (log-spaced bins) ---
    let peaks: Vec<f64> = eps.iter().map(|e| e.peak.abs()).collect();
    let pmin = peaks
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min)
        .max(1e-3);
    let pmax = peaks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bins = 12usize;
    let log_lo = pmin.ln();
    let log_hi = pmax.ln().max(log_lo + 0.1);
    let step = (log_hi - log_lo) / bins as f64;
    let mut counts = vec![0u32; bins];
    for p in &peaks {
        let idx =
            (((p.ln() - log_lo) / step).floor() as isize).clamp(0, bins as isize - 1) as usize;
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
    Ok(true)
}

// ---------------------------------------------------------------------------
// NEW: compact tabular episode listing (small-N fallback for distribution)

/// Renders every episode for a motif as a single-line row: channel /
/// t_start / duration / peak / EMA-at-boundary. Intended as the
/// replacement figure when `plot_episode_distribution` refuses to emit
/// (N < 5, or all peaks+durations identical). Keeps academic honesty:
/// shows the data that exists rather than a misleading 1-bar histogram.
///
/// Returns `false` (no figure emitted) when there are no matching
/// episodes, so the caller can drop the file path entirely.
pub fn plot_episode_table(
    path: &Path,
    title: &str,
    episodes: &[Episode],
    motif: MotifClass,
) -> Result<bool> {
    let eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();
    if eps.is_empty() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let row_h: u32 = 26;
    let header_h: u32 = 80;
    let footer_h: u32 = 28;
    let width: u32 = 1100;
    let total_h = header_h + row_h * (eps.len() as u32 + 1) + footer_h;

    let root = BitMapBackend::new(path, (width, total_h)).into_drawing_area();
    root.fill(&WHITE)?;

    root.draw_text(
        title,
        &TextStyle::from(("sans-serif", 20)).color(&BLACK),
        (16, 14),
    )?;
    root.draw_text(
        &format!(
            "N = {} episodes -- compact listing (distribution histogram withheld: insufficient variance)",
            eps.len()
        ),
        &TextStyle::from(("sans-serif", 12)).color(&RGBColor(80, 80, 80)),
        (16, 44),
    )?;

    // Column layout.
    let cols: [(i32, &str); 5] = [
        (20, "channel"),
        (380, "t_start (s)"),
        (540, "duration (s)"),
        (720, "peak"),
        (880, "ema@boundary"),
    ];
    let header_style = TextStyle::from(("sans-serif", 13)).color(&BLACK);
    let header_y = (header_h - row_h + 8) as i32;
    for (x, name) in &cols {
        root.draw_text(name, &header_style, (*x, header_y))?;
    }
    // Underline the header row.
    root.draw(&PathElement::new(
        vec![
            (10, header_h as i32 + 2),
            (width as i32 - 10, header_h as i32 + 2),
        ],
        RGBColor(140, 140, 140).stroke_width(1),
    ))?;

    let row_style = TextStyle::from(("sans-serif", 12)).color(&RGBColor(30, 30, 30));
    for (i, ep) in eps.iter().enumerate() {
        let y = header_h as i32 + row_h as i32 * (i as i32 + 1) - row_h as i32 + 8;
        let chan_display = ep
            .channel
            .as_deref()
            .map(humanize_channel_label)
            .unwrap_or_else(|| "(none)".to_string());
        let dur = (ep.t_end - ep.t_start).max(0.0);
        let fields: [(i32, String); 5] = [
            (cols[0].0, chan_display),
            (cols[1].0, format!("{:.3}", ep.t_start)),
            (cols[2].0, format!("{:.3}", dur)),
            (cols[3].0, format!("{:+.4}", ep.peak)),
            (cols[4].0, format!("{:.4}", ep.ema_at_boundary)),
        ];
        for (x, text) in &fields {
            root.draw_text(text, &row_style, (*x, y))?;
        }
    }

    root.present()?;
    Ok(true)
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
    let chans: Vec<String> = chans
        .into_iter()
        .take(top_k_channels)
        .map(|(c, _)| c)
        .collect();

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
    root.draw_text("motif (row) / channel (col)", &header_text, (14, 58))?;
    let col_labels: Vec<String> = std::iter::once("total".to_string())
        .chain(chans.iter().cloned())
        .collect();
    for (j, lbl) in col_labels.iter().enumerate() {
        let x = left_label_w + (j as u32) * cell_w + 8;
        let y = 52;
        let shown: String = if j == 0 {
            lbl.clone()
        } else {
            humanize_channel_label(lbl)
        };
        root.draw_text(&shown, &header_small, (x as i32, y))?;
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
                [
                    (x as i32, y as i32),
                    ((x + cell_w) as i32, (y + cell_h) as i32),
                ],
                fill.filled(),
            ))?;
            root.draw(&Rectangle::new(
                [
                    (x as i32, y as i32),
                    ((x + cell_w) as i32, (y + cell_h) as i32),
                ],
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

pub fn plot_metric_bars(path: &Path, title: &str, bars: &[(String, f64)]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (900, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max = bars
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0_f64, f64::max)
        .max(0.01);
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
        .x_label_formatter(&|i| bars.get(*i).map(|b| b.0.clone()).unwrap_or_default())
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
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 18, y)], color.stroke_width(2))
            });
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

/// Precision–recall scatter across a parameter sweep.
///
/// `rows` is one `(precision, recall, f1, label)` per sweep point. The
/// chart places recall on the x-axis, precision on the y-axis, and
/// colours each point by F1 (red = 0.0, green = 1.0). The published-
/// baseline operating point is rendered as a black star if `baseline` is
/// `Some((p, r))`, so a reviewer can locate the canonical result within
/// the swept region without reading the CSV.
///
/// Diagonal F1-isoclines at 0.25, 0.5, 0.75 are drawn as dashed grey
/// reference lines. The axes are clamped to `[0, 1.05]` regardless of
/// the sweep range so figures from different motifs are directly
/// comparable — one of the properties a reviewer or licensing counsel
/// will check for honesty.
pub fn plot_pr_curve(
    path: &Path,
    title: &str,
    rows: &[(f64, f64, f64, String)],
    baseline: Option<(f64, f64)>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    debug_assert!(
        rows.iter().all(|(p, r, f, _)| (0.0..=1.0).contains(p)
            && (0.0..=1.0).contains(r)
            && (0.0..=1.0).contains(f)),
        "PR rows must be in [0,1]"
    );
    let root = BitMapBackend::new(path, (820, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    // Caption font size is 16: plotters' no-TTF bitmap font panics when
    // a caption rendered at size 22 overflows the canvas width, and the
    // longer motif names (e.g. `cardinality_mismatch_regime`) do exactly
    // that at the figure width chosen here. 16 keeps the caption
    // legible while safely fitting all five motif names.
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 16))
        .margin(15)
        .x_label_area_size(55)
        .y_label_area_size(65)
        .build_cartesian_2d(0.0_f64..1.05_f64, 0.0_f64..1.05_f64)?;
    chart
        .configure_mesh()
        .x_desc("recall")
        .y_desc("precision")
        .x_label_formatter(&|v| format!("{:.2}", v))
        .y_label_formatter(&|v| format!("{:.2}", v))
        .draw()?;

    // F1-isocline reference lines. For a target F1 value f*, the curve
    // p(r) = f* r / (2 r − f*) holds on the interval (f*/2, 1]. Drawing
    // 0.25 / 0.5 / 0.75 gives the reader a visual scale for "how close
    // to the ideal corner is this operating point".
    for f_iso in [0.25_f64, 0.5, 0.75] {
        let grey = RGBColor(170, 170, 170);
        let pts: Vec<(f64, f64)> = (1..=100)
            .map(|i| f_iso * 0.5 + (1.0 - f_iso * 0.5) * (i as f64 / 100.0))
            .map(|r| {
                let denom = 2.0 * r - f_iso;
                let p = f_iso * r / denom;
                (r, p)
            })
            .filter(|(_, p)| (0.0..=1.05).contains(p))
            .collect();
        chart.draw_series(LineSeries::new(pts, grey.stroke_width(1)))?;
    }

    // Scatter points, coloured by F1: a 5-stop gradient from muted red
    // (F1=0) through amber to green (F1=1). Keeps the figure legible in
    // greyscale print too — the vertical position encodes precision, so
    // colour is a secondary channel.
    fn f1_color(f1: f64) -> RGBColor {
        let stops: [(f64, (u8, u8, u8)); 5] = [
            (0.0, (178, 24, 43)),
            (0.25, (239, 138, 98)),
            (0.5, (253, 219, 199)),
            (0.75, (103, 169, 207)),
            (1.0, (33, 102, 172)),
        ];
        let f = f1.clamp(0.0, 1.0);
        for win in stops.windows(2) {
            let (t0, c0) = win[0];
            let (t1, c1) = win[1];
            if (t0..=t1).contains(&f) {
                let u = if t1 > t0 { (f - t0) / (t1 - t0) } else { 0.0 };
                let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * u) as u8;
                return RGBColor(mix(c0.0, c1.0), mix(c0.1, c1.1), mix(c0.2, c1.2));
            }
        }
        RGBColor(stops[4].1 .0, stops[4].1 .1, stops[4].1 .2)
    }

    chart.draw_series(rows.iter().map(|(p, r, f1, _)| {
        let color = f1_color(*f1);
        Circle::new((*r, *p), 4, color.filled())
    }))?;

    if let Some((p_b, r_b)) = baseline {
        debug_assert!(
            (0.0..=1.0).contains(&p_b) && (0.0..=1.0).contains(&r_b),
            "baseline PR point in [0,1]"
        );
        // Draw the baseline as a larger black-outlined circle over a
        // white fill so it reads clearly against any F1-colour dot
        // already in the same (recall, precision) location. Text
        // annotation is intentionally omitted — plotters without the
        // TTF feature is flaky on some glyph sequences, and the CSV
        // already carries the factor=1.00 row for unambiguous lookup.
        chart.draw_series(std::iter::once(Circle::new((r_b, p_b), 7, WHITE.filled())))?;
        chart.draw_series(std::iter::once(Circle::new(
            (r_b, p_b),
            7,
            BLACK.stroke_width(2),
        )))?;
        chart.draw_series(std::iter::once(Cross::new(
            (r_b, p_b),
            6,
            BLACK.stroke_width(2),
        )))?;
    }

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
        area.draw_text(
            "(no samples)",
            &TextStyle::from(("sans-serif", 18)),
            (440, 360),
        )?;
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
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(-slew_threshold, 0.0), (-slew_threshold, s_max)],
        slew_color.stroke_width(2),
    )))?;
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![
                (r_min - r_pad, drift_threshold),
                (r_max + r_pad, drift_threshold),
            ],
            drift_color.stroke_width(2),
        )))?
        .label(format!("drift threshold = {:.2}", drift_threshold))
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], drift_color.stroke_width(2))
        });

    // Lyapunov-style sublevel-set overlay: V(r,s) := max(|r|/θ_slew,
    // s/θ_drift) ≤ 1 is exactly the stable region of the state machine's
    // own rule. See Appendix E. The contour `V = 1` is a dashed
    // rectangle |r| = θ_slew and s = θ_drift. This does NOT claim
    // stability of the underlying workload — only of the observer's
    // decision rule.
    let lyap_color = RGBColor(50, 50, 50);
    let lyap_style = ShapeStyle {
        color: lyap_color.to_rgba(),
        filled: false,
        stroke_width: 2,
    };
    let rect_pts = vec![
        (-slew_threshold, drift_threshold),
        (slew_threshold, drift_threshold),
        (slew_threshold, 0.0),
        (-slew_threshold, 0.0),
        (-slew_threshold, drift_threshold),
    ];
    chart
        .draw_series(rect_pts.windows(2).map(|seg| {
            PathElement::new(
                vec![seg[0], seg[1]],
                ShapeStyle {
                    color: lyap_style.color,
                    filled: false,
                    stroke_width: 1,
                }
                .stroke_width(1),
            )
        }))?
        .label("V(r,s) ≤ 1 (stable sublevel set)")
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 18, y)], lyap_color.stroke_width(1))
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
        .draw_series(
            stable_pts
                .iter()
                .map(|p| Circle::new(*p, 3, stable_color.filled())),
        )?
        .label("Stable")
        .legend(move |(x, y)| Circle::new((x + 9, y), 4, stable_color.filled()));
    chart
        .draw_series(
            drift_pts
                .iter()
                .map(|p| Circle::new(*p, 3, drift_pt_color.filled())),
        )?
        .label("Drift")
        .legend(move |(x, y)| Circle::new((x + 9, y), 4, drift_pt_color.filled()));
    chart
        .draw_series(
            boundary_pts
                .iter()
                .map(|p| Circle::new(*p, 3, boundary_color.filled())),
        )?
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
        .draw_series(
            raw_pts
                .iter()
                .map(|p| Circle::new(*p, 2, raw_color.filled())),
        )?
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

/// DSFB vs baselines visual contrast on a single trace window.
///
/// Four stacked subplots sharing the same x-axis (time):
///   (1) raw residual trace on the chosen channel,
///   (2) PELT change-points as vertical ticks,
///   (3) BOCPD change-points as vertical ticks,
///   (4) DSFB episode as a single filled rectangle `[t_start, t_end]`.
///
/// This is a **structural** contrast, not a detection-quality
/// comparison: baselines emit points (which we wrap in a nominal
/// detection window for scoring), DSFB emits a typed, bounded,
/// grammar-constrained episode. The caller is expected to quote that
/// distinction verbatim in the figure caption.
#[allow(clippy::too_many_arguments)]
pub fn plot_detector_contrast(
    path: &Path,
    title: &str,
    stream: &ResidualStream,
    class: ResidualClass,
    channel: Option<&str>,
    dsfb_episodes: &[Episode],
    pelt_events: &[f64],
    bocpd_events: &[f64],
    t_lo: f64,
    t_hi: f64,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 780)).into_drawing_area();
    root.fill(&WHITE)?;

    let root = root.titled(title, ("sans-serif", 22))?;
    let panels = root.split_evenly((4, 1));

    let samples: Vec<(f64, f64)> = stream
        .iter_class(class)
        .filter(|s| match (channel, s.channel.as_deref()) {
            (Some(want), Some(have)) => want == have,
            (None, _) => true,
            (Some(_), None) => false,
        })
        .filter(|s| s.t >= t_lo && s.t <= t_hi)
        .map(|s| (s.t, s.value))
        .collect();

    let (v_min, v_max, _, _) = stats_of(samples.iter().map(|s| s.1));
    let y_pad = (v_max - v_min).abs().max(0.01) * 0.10;

    {
        let mut chart = ChartBuilder::on(&panels[0])
            .caption("residual", ("sans-serif", 16))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(55)
            .build_cartesian_2d(t_lo..t_hi, (v_min - y_pad)..(v_max + y_pad))?;
        chart.configure_mesh().x_desc("t (s)").y_desc("r").draw()?;
        chart.draw_series(LineSeries::new(
            samples.iter().cloned(),
            RGBColor(31, 119, 180).stroke_width(1),
        ))?;
    }

    let tick_style = |color: RGBColor| color.stroke_width(2);

    {
        let mut chart = ChartBuilder::on(&panels[1])
            .caption("PELT change-points", ("sans-serif", 16))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(55)
            .build_cartesian_2d(t_lo..t_hi, 0.0_f64..1.0_f64)?;
        chart
            .configure_mesh()
            .x_desc("t (s)")
            .disable_y_axis()
            .draw()?;
        let pelt_color = RGBColor(148, 103, 189);
        for &t in pelt_events.iter().filter(|t| **t >= t_lo && **t <= t_hi) {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(t, 0.0), (t, 1.0)],
                tick_style(pelt_color),
            )))?;
        }
    }

    {
        let mut chart = ChartBuilder::on(&panels[2])
            .caption("BOCPD change-points", ("sans-serif", 16))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(55)
            .build_cartesian_2d(t_lo..t_hi, 0.0_f64..1.0_f64)?;
        chart
            .configure_mesh()
            .x_desc("t (s)")
            .disable_y_axis()
            .draw()?;
        let bocpd_color = RGBColor(44, 160, 44);
        for &t in bocpd_events.iter().filter(|t| **t >= t_lo && **t <= t_hi) {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(t, 0.0), (t, 1.0)],
                tick_style(bocpd_color),
            )))?;
        }
    }

    {
        let mut chart = ChartBuilder::on(&panels[3])
            .caption("DSFB episode (typed, bounded)", ("sans-serif", 16))
            .margin(8)
            .x_label_area_size(25)
            .y_label_area_size(55)
            .build_cartesian_2d(t_lo..t_hi, 0.0_f64..1.0_f64)?;
        chart
            .configure_mesh()
            .x_desc("t (s)")
            .disable_y_axis()
            .draw()?;
        let ep_color = RGBColor(214, 39, 40).mix(0.4);
        for ep in dsfb_episodes
            .iter()
            .filter(|e| e.t_end >= t_lo && e.t_start <= t_hi)
        {
            let a = ep.t_start.max(t_lo);
            let b = ep.t_end.min(t_hi);
            chart.draw_series(std::iter::once(Rectangle::new(
                [(a, 0.1), (b, 0.9)],
                ep_color.filled(),
            )))?;
        }
    }

    root.present()?;
    Ok(())
}

/// Refusal contrast on a pure-noise null trace.
///
/// Two stacked subplots sharing the same x-axis:
///   (1) null trace with baseline change-points overlaid as red ticks,
///   (2) null trace with DSFB episodes overlaid as filled rectangles
///       (empty by construction at published thresholds).
///
/// The figure anchors the "Cases Where Interpretation Is Not Justified"
/// section of the paper.
pub fn plot_refusal_contrast(
    path: &Path,
    title: &str,
    null_trace: &[(f64, f64)],
    dsfb_episodes: &[Episode],
    baseline_events: &[(f64, &'static str)],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1100, 620)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.titled(title, ("sans-serif", 22))?;
    let panels = root.split_evenly((2, 1));

    if null_trace.is_empty() {
        panels[0].draw_text(
            "(empty null trace)",
            &TextStyle::from(("sans-serif", 18)),
            (440, 280),
        )?;
        root.present()?;
        return Ok(());
    }

    let t_lo = null_trace.first().map(|p| p.0).unwrap_or(0.0);
    let t_hi = null_trace.last().map(|p| p.0).unwrap_or(t_lo + 1.0);
    let (v_min, v_max, _, _) = stats_of(null_trace.iter().map(|s| s.1));
    let y_pad = (v_max - v_min).abs().max(0.01) * 0.10;

    {
        let mut chart = ChartBuilder::on(&panels[0])
            .caption(
                "null trace — baselines fire false alarms",
                ("sans-serif", 16),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(t_lo..t_hi, (v_min - y_pad)..(v_max + y_pad))?;
        chart.configure_mesh().x_desc("t (s)").y_desc("r").draw()?;
        chart.draw_series(LineSeries::new(
            null_trace.iter().cloned(),
            RGBAColor(120, 120, 120, 0.7).stroke_width(1),
        ))?;
        let fa_color = RGBColor(214, 39, 40);
        for (t, _who) in baseline_events
            .iter()
            .filter(|(t, _)| *t >= t_lo && *t <= t_hi)
        {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(*t, v_min - y_pad), (*t, v_max + y_pad)],
                fa_color.stroke_width(2),
            )))?;
        }
    }

    {
        let mut chart = ChartBuilder::on(&panels[1])
            .caption(
                "null trace — DSFB refuses interpretation",
                ("sans-serif", 16),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(t_lo..t_hi, (v_min - y_pad)..(v_max + y_pad))?;
        chart.configure_mesh().x_desc("t (s)").y_desc("r").draw()?;
        chart.draw_series(LineSeries::new(
            null_trace.iter().cloned(),
            RGBAColor(120, 120, 120, 0.7).stroke_width(1),
        ))?;
        let ep_color = RGBColor(31, 119, 180).mix(0.4);
        for ep in dsfb_episodes
            .iter()
            .filter(|e| e.t_end >= t_lo && e.t_start <= t_hi)
        {
            let a = ep.t_start.max(t_lo);
            let b = ep.t_end.min(t_hi);
            chart.draw_series(std::iter::once(Rectangle::new(
                [(a, v_min - y_pad), (b, v_max + y_pad)],
                ep_color.filled(),
            )))?;
        }
        if dsfb_episodes
            .iter()
            .filter(|e| e.t_end >= t_lo && e.t_start <= t_hi)
            .count()
            == 0
        {
            panels[1].draw_text(
                "(no DSFB episodes — refusal by construction)",
                &TextStyle::from(("sans-serif", 14)).color(&RGBColor(80, 80, 80)),
                (320, 270),
            )?;
        }
    }

    root.present()?;
    Ok(())
}
