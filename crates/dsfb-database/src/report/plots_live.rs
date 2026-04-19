//! Live-adapter figures for the paper's §Live and §Live Evaluation
//! sections. Lives in its own file so the large plots.rs stays
//! navigable.
//!
//! Three figures in one file:
//!   * [`plot_live_pulsed_scrape`] — synthetic three-panel cadence
//!     figure (architecture illustration; see §Live).
//!   * [`plot_live_real_pg`] — real-engine three-panel residual
//!     trajectory with threshold reference lines, ground-truth fault
//!     window, and per-detector detection timestamps (empirical
//!     evaluation; see §Live Evaluation).
//!   * [`plot_live_determinism_overlay`] — two-panel overlay of two
//!     independent real-engine tapes, illustrating the asymmetry
//!     pinned by the 7th non-claim (engine→tape non-deterministic,
//!     tape→episodes byte-stable).

use anyhow::Result;
use plotters::prelude::*;
use std::path::Path;

/// Three-panel pulsed-scrape cadence figure for the live PostgreSQL
/// adapter's paper section.
///
/// - **Top**: raw cumulative counters (total_exec_time_ms on the left
///   axis, calls on the right) as the live adapter sees them — both
///   are monotonically non-decreasing by construction.
/// - **Middle**: the distilled per-call latency residual for a single
///   pinned qid — fixed-magnitude baseline until the planted
///   perturbation window, then a shifted plateau.
/// - **Bottom**: the emitted `plan_regression_onset` episode rectangle
///   on a time axis, with a throttle-factor trace overlaid to show a
///   synthetic backpressure event.
///
/// The figure is a byte-deterministic function of the input data;
/// regeneration is the responsibility of
/// `src/bin/live_pulsed_scrape_figure.rs`.
#[allow(clippy::too_many_arguments)]
pub fn plot_live_pulsed_scrape(
    path: &Path,
    snapshots_t: &[f64],
    total_exec_ms: &[f64],
    calls_cum: &[f64],
    residual_t: &[f64],
    residual_v: &[f64],
    episode_window: Option<(f64, f64)>,
    throttle_t: &[f64],
    throttle_factor: &[f64],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    debug_assert_eq!(snapshots_t.len(), total_exec_ms.len());
    debug_assert_eq!(snapshots_t.len(), calls_cum.len());
    debug_assert_eq!(residual_t.len(), residual_v.len());
    debug_assert_eq!(throttle_t.len(), throttle_factor.len());

    let root = BitMapBackend::new(path, (1100, 820)).into_drawing_area();
    root.fill(&WHITE)?;
    let panels = root.split_evenly((3, 1));

    let t_min = *snapshots_t.first().unwrap_or(&0.0);
    let t_max = *snapshots_t.last().unwrap_or(&(t_min + 1.0));
    let exec_max = total_exec_ms
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1.0);
    let calls_max = calls_cum
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1.0);
    let mut top = ChartBuilder::on(&panels[0])
        .caption(
            "Raw cumulative counters (pg_stat_statements, one qid)",
            ("sans-serif", 20),
        )
        .margin(12)
        .x_label_area_size(35)
        .y_label_area_size(70)
        .right_y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, 0.0..exec_max * 1.05)?
        .set_secondary_coord(t_min..t_max, 0.0..calls_max * 1.05);
    top.configure_mesh()
        .x_desc("snapshot t (s)")
        .y_desc("total_exec_time_ms (cumulative)")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    top.configure_secondary_axes()
        .y_desc("calls (cumulative)")
        .draw()?;
    let exec_color = RGBColor(31, 119, 180);
    let calls_color = RGBColor(255, 127, 14);
    top.draw_series(LineSeries::new(
        snapshots_t.iter().zip(total_exec_ms).map(|(t, v)| (*t, *v)),
        exec_color.stroke_width(2),
    ))?
    .label("total_exec_time_ms")
    .legend(move |(x, y)| {
        PathElement::new([(x, y), (x + 18, y)], exec_color.stroke_width(2))
    });
    top.draw_secondary_series(LineSeries::new(
        snapshots_t.iter().zip(calls_cum).map(|(t, v)| (*t, *v)),
        calls_color.stroke_width(2),
    ))?
    .label("calls")
    .legend(move |(x, y)| {
        PathElement::new([(x, y), (x + 18, y)], calls_color.stroke_width(2))
    });
    top.configure_series_labels()
        .background_style(WHITE.mix(0.85).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    let r_min = residual_v
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let r_max = residual_v
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max)
        .max(1.0);
    let spread = (r_max - r_min).abs().max(1.0);
    let y_lo = r_min.min(0.0) - 0.05 * spread;
    let y_hi = r_max + 0.10 * spread;
    let mut mid = ChartBuilder::on(&panels[1])
        .caption(
            "Distilled per-call latency residual",
            ("sans-serif", 20),
        )
        .margin(12)
        .x_label_area_size(35)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, y_lo..y_hi)?;
    mid.configure_mesh()
        .x_desc("t (s)")
        .y_desc("residual (ms/call)")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    if let Some((es, ee)) = episode_window {
        mid.draw_series(std::iter::once(Rectangle::new(
            [(es, y_lo), (ee, y_hi)],
            RGBAColor(214, 39, 40, 0.10).filled(),
        )))?;
    }
    let res_color = RGBColor(44, 160, 44);
    mid.draw_series(LineSeries::new(
        residual_t.iter().zip(residual_v).map(|(t, v)| (*t, *v)),
        res_color.stroke_width(2),
    ))?;

    let throttle_max = throttle_factor
        .iter()
        .cloned()
        .fold(1.0_f64, f64::max)
        .max(1.0);
    let y_hi_bot = (throttle_max * 1.15).max(2.0);
    let mut bot = ChartBuilder::on(&panels[2])
        .caption(
            "Emitted plan_regression_onset episode + synthetic throttle factor",
            ("sans-serif", 20),
        )
        .margin(12)
        .x_label_area_size(35)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, 0.0..y_hi_bot)?;
    bot.configure_mesh()
        .x_desc("t (s)")
        .y_desc("throttle factor (× nominal sleep)")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    if let Some((es, ee)) = episode_window {
        bot.draw_series(std::iter::once(Rectangle::new(
            [(es, 0.0), (ee, y_hi_bot)],
            RGBAColor(214, 39, 40, 0.18).filled(),
        )))?
        .label("plan_regression_onset episode")
        .legend(move |(x, y)| {
            Rectangle::new(
                [(x, y - 6), (x + 18, y + 6)],
                RGBAColor(214, 39, 40, 0.35).filled(),
            )
        });
    }
    let throttle_color = RGBColor(148, 103, 189);
    bot.draw_series(LineSeries::new(
        throttle_t.iter().zip(throttle_factor).map(|(t, v)| (*t, *v)),
        throttle_color.stroke_width(2),
    ))?
    .label("throttle factor")
    .legend(move |(x, y)| {
        PathElement::new([(x, y), (x + 18, y)], throttle_color.stroke_width(2))
    });
    bot.configure_series_labels()
        .background_style(WHITE.mix(0.85).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    root.present()?;
    Ok(())
}

/// One qid's residual trace.
pub struct QidTrace<'a> {
    pub label: &'a str,
    pub t: &'a [f64],
    pub v: &'a [f64],
}

/// One detector's detection timestamp on the pinned-fault qid.
pub struct DetectorMark<'a> {
    pub label: &'a str,
    pub t: Option<f64>,
}

/// One cache-IO bucket trace (1 − hit_ratio in [0, 1]).
pub struct CacheBucketTrace<'a> {
    pub label: &'a str,
    pub t: &'a [f64],
    pub v: &'a [f64],
}

/// One emitted episode rectangle (motif class + [t_start, t_end]).
pub struct EpisodeRect<'a> {
    pub motif: &'a str,
    pub t_start: f64,
    pub t_end: f64,
}

const PLAN_REGRESSION_SLEW_DEFAULT: f64 = 0.50;
const CACHE_COLLAPSE_DRIFT_DEFAULT: f64 = 0.10;

fn qid_color(i: usize) -> RGBColor {
    // Matplotlib tab10-ish palette, cycled.
    const P: [(u8, u8, u8); 6] = [
        (31, 119, 180),
        (255, 127, 14),
        (44, 160, 44),
        (214, 39, 40),
        (148, 103, 189),
        (140, 86, 75),
    ];
    let (r, g, b) = P[i % P.len()];
    RGBColor(r, g, b)
}

fn motif_color(m: &str) -> RGBColor {
    match m {
        "plan_regression_onset" => RGBColor(44, 160, 44),
        "cache_collapse" => RGBColor(148, 103, 189),
        "contention_ramp" => RGBColor(214, 39, 40),
        "workload_phase_transition" => RGBColor(255, 127, 14),
        "cardinality_mismatch_regime" => RGBColor(140, 86, 75),
        _ => RGBColor(127, 127, 127),
    }
}

/// Three-panel real-engine residual trajectory figure for §Live
/// Evaluation. Unlike [`plot_live_pulsed_scrape`] (which plots a
/// deterministically-synthesised fixture for architectural
/// illustration), this figure is a function of a live-captured tape
/// plus its ground-truth annotation plus the bakeoff CSV that scores
/// DSFB and three published baselines on that tape.
///
/// The caller is responsible for having read the input CSVs / tape
/// and extracted the per-qid residual traces, the cache-IO bucket
/// traces, the emitted episodes, and the per-detector detection
/// timestamps on the pinned-fault qid. This keeps the plotter
/// layer pure and lets the calling binary (`render_live_eval_figs`)
/// centralise file I/O.
#[allow(clippy::too_many_arguments)]
pub fn plot_live_real_pg(
    path: &Path,
    plan_traces: &[QidTrace<'_>],
    cache_traces: &[CacheBucketTrace<'_>],
    poll_t: &[f64],
    throttle: &[f64],
    ground_truth_window: Option<(f64, f64)>,
    detector_marks: &[DetectorMark<'_>],
    episodes: &[EpisodeRect<'_>],
    caption: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let root = BitMapBackend::new(path, (1200, 980)).into_drawing_area();
    root.fill(&WHITE)?;
    let (title_area, panels_area) = root.split_vertically(38);
    title_area.draw(&Text::new(
        caption.to_string(),
        (14, 12),
        ("sans-serif", 16).into_font(),
    ))?;
    let panels = panels_area.split_evenly((3, 1));

    let mut t_min = f64::INFINITY;
    let mut t_max = f64::NEG_INFINITY;
    for tr in plan_traces {
        for t in tr.t {
            t_min = t_min.min(*t);
            t_max = t_max.max(*t);
        }
    }
    for tr in cache_traces {
        for t in tr.t {
            t_min = t_min.min(*t);
            t_max = t_max.max(*t);
        }
    }
    for t in poll_t {
        t_min = t_min.min(*t);
        t_max = t_max.max(*t);
    }
    if !t_min.is_finite() || !t_max.is_finite() || t_min >= t_max {
        t_min = 0.0;
        t_max = 1.0;
    }

    // --- Panel A: plan-regression residuals with slew reference ---
    let mut plan_min = 0.0_f64;
    let mut plan_max = PLAN_REGRESSION_SLEW_DEFAULT;
    for tr in plan_traces {
        for v in tr.v {
            if v.is_finite() {
                plan_min = plan_min.min(*v);
                plan_max = plan_max.max(*v);
            }
        }
    }
    let plan_span = (plan_max - plan_min).abs().max(0.2);
    let plan_lo = plan_min - 0.08 * plan_span;
    let plan_hi = plan_max + 0.12 * plan_span;

    let mut top = ChartBuilder::on(&panels[0])
        .caption(
            "Panel A — plan_regression residual, top-k qids",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, plan_lo..plan_hi)?;
    top.configure_mesh()
        .x_desc("t since first poll (s)")
        .y_desc("residual (frac of baseline)")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    if let Some((gs, ge)) = ground_truth_window {
        top.draw_series(std::iter::once(Rectangle::new(
            [(gs, plan_lo), (ge, plan_hi)],
            RGBAColor(150, 150, 150, 0.18).filled(),
        )))?
        .label("ground-truth fault window")
        .legend(|(x, y)| {
            Rectangle::new(
                [(x, y - 5), (x + 18, y + 5)],
                RGBAColor(150, 150, 150, 0.4).filled(),
            )
        });
    }
    // horizontal threshold reference line (slew)
    top.draw_series(LineSeries::new(
        [(t_min, PLAN_REGRESSION_SLEW_DEFAULT), (t_max, PLAN_REGRESSION_SLEW_DEFAULT)],
        BLACK.mix(0.5).stroke_width(1),
    ))?
    .label("slew_threshold = 0.50")
    .legend(|(x, y)| PathElement::new([(x, y), (x + 18, y)], BLACK.mix(0.6).stroke_width(1)));
    for (i, tr) in plan_traces.iter().enumerate() {
        let col = qid_color(i);
        let pts: Vec<(f64, f64)> = tr
            .t
            .iter()
            .zip(tr.v)
            .map(|(t, v)| (*t, *v))
            .collect();
        top.draw_series(LineSeries::new(pts, col.stroke_width(2)))?
            .label(tr.label.to_string())
            .legend(move |(x, y)| {
                PathElement::new([(x, y), (x + 18, y)], col.stroke_width(2))
            });
    }
    for m in detector_marks {
        if let Some(t) = m.t {
            let (col, _) = match m.label {
                "dsfb-database" => (RGBColor(44, 160, 44), 2),
                "adwin" => (RGBColor(214, 39, 40), 2),
                "bocpd" => (RGBColor(255, 127, 14), 2),
                "pelt" => (RGBColor(148, 103, 189), 2),
                _ => (BLACK, 1),
            };
            top.draw_series(LineSeries::new(
                [(t, plan_lo), (t, plan_hi)],
                col.mix(0.9).stroke_width(2),
            ))?
            .label(format!("{} detect @ {:.1}s", m.label, t))
            .legend(move |(x, y)| {
                PathElement::new([(x, y), (x + 18, y)], col.stroke_width(2))
            });
        }
    }
    top.configure_series_labels()
        .background_style(WHITE.mix(0.8).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    // --- Panel B: cache-IO 1 - hit_ratio with drift reference ---
    let mut cache_max = CACHE_COLLAPSE_DRIFT_DEFAULT;
    for tr in cache_traces {
        for v in tr.v {
            if v.is_finite() {
                cache_max = cache_max.max(*v);
            }
        }
    }
    let cache_hi = (cache_max * 1.15).max(0.15).min(1.05);

    let mut mid = ChartBuilder::on(&panels[1])
        .caption(
            "Panel B — cache_io 1 − hit_ratio by bucket (pg_stat_io)",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, 0.0_f64..cache_hi)?;
    mid.configure_mesh()
        .x_desc("t (s)")
        .y_desc("1 − hit_ratio")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    mid.draw_series(LineSeries::new(
        [(t_min, CACHE_COLLAPSE_DRIFT_DEFAULT), (t_max, CACHE_COLLAPSE_DRIFT_DEFAULT)],
        BLACK.mix(0.5).stroke_width(1),
    ))?
    .label("drift_threshold = 0.10")
    .legend(|(x, y)| PathElement::new([(x, y), (x + 18, y)], BLACK.mix(0.6).stroke_width(1)));
    for (i, tr) in cache_traces.iter().enumerate() {
        let col = qid_color(i + 1);
        let pts: Vec<(f64, f64)> = tr.t.iter().zip(tr.v).map(|(t, v)| (*t, *v)).collect();
        mid.draw_series(LineSeries::new(pts, col.stroke_width(2)))?
            .label(tr.label.to_string())
            .legend(move |(x, y)| {
                PathElement::new([(x, y), (x + 18, y)], col.stroke_width(2))
            });
    }
    mid.configure_series_labels()
        .background_style(WHITE.mix(0.8).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    // --- Panel C: throttle + episode rectangles ---
    let throttle_hi = throttle
        .iter()
        .cloned()
        .fold(2.0_f64, f64::max)
        .max(2.0)
        * 1.1;
    let mut bot = ChartBuilder::on(&panels[2])
        .caption(
            "Panel C — scraper throttle factor + emitted episode windows",
            ("sans-serif", 18),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, 0.0_f64..throttle_hi)?;
    bot.configure_mesh()
        .x_desc("t (s)")
        .y_desc("throttle (× nominal)")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    for ep in episodes {
        let col = motif_color(ep.motif);
        bot.draw_series(std::iter::once(Rectangle::new(
            [(ep.t_start, 0.0), (ep.t_end, throttle_hi)],
            col.mix(0.18).filled(),
        )))?;
    }
    // one legend entry per unique motif in episodes
    let mut seen_motifs: Vec<&str> = Vec::new();
    for ep in episodes {
        if !seen_motifs.contains(&ep.motif) {
            seen_motifs.push(ep.motif);
            let col = motif_color(ep.motif);
            let label = ep.motif.to_string();
            bot.draw_series(std::iter::once(Rectangle::new(
                [(t_max - 0.001, -1.0), (t_max, -0.999)],
                col.mix(0.18).filled(),
            )))?
            .label(label)
            .legend(move |(x, y)| {
                Rectangle::new([(x, y - 5), (x + 18, y + 5)], col.mix(0.35).filled())
            });
        }
    }
    let throttle_color = RGBColor(85, 85, 85);
    bot.draw_series(LineSeries::new(
        poll_t.iter().zip(throttle).map(|(t, v)| (*t, *v)),
        throttle_color.stroke_width(2),
    ))?
    .label("throttle factor")
    .legend(move |(x, y)| {
        PathElement::new([(x, y), (x + 18, y)], throttle_color.stroke_width(2))
    });
    bot.configure_series_labels()
        .background_style(WHITE.mix(0.8).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Two-panel overlay of two independent live-captured tapes + their
/// replayed episode streams. Illustrates the asymmetry pinned by the
/// 7th non-claim: two live runs against the same workload + same
/// planted fault produce different tapes (engine → tape
/// non-deterministic), but each tape's replay emits a byte-stable
/// episode stream (tape → episodes deterministic).
pub fn plot_live_determinism_overlay(
    path: &Path,
    tape_a_label: &str,
    tape_a_trace: &QidTrace<'_>,
    tape_a_episodes: &[EpisodeRect<'_>],
    tape_a_sha_prefix: &str,
    tape_a_episode_fp_prefix: &str,
    tape_b_label: &str,
    tape_b_trace: &QidTrace<'_>,
    tape_b_episodes: &[EpisodeRect<'_>],
    tape_b_sha_prefix: &str,
    tape_b_episode_fp_prefix: &str,
    caption: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let root = BitMapBackend::new(path, (1200, 820)).into_drawing_area();
    root.fill(&WHITE)?;
    let (title_area, panels_area) = root.split_vertically(38);
    title_area.draw(&Text::new(
        caption.to_string(),
        (14, 12),
        ("sans-serif", 16).into_font(),
    ))?;
    let panels = panels_area.split_evenly((2, 1));

    // --- Panel A: residual overlay ---
    let mut t_min = f64::INFINITY;
    let mut t_max = f64::NEG_INFINITY;
    for t in tape_a_trace.t.iter().chain(tape_b_trace.t.iter()) {
        t_min = t_min.min(*t);
        t_max = t_max.max(*t);
    }
    if !t_min.is_finite() || !t_max.is_finite() || t_min >= t_max {
        t_min = 0.0;
        t_max = 1.0;
    }
    let mut v_max = 1.0_f64;
    for v in tape_a_trace.v.iter().chain(tape_b_trace.v.iter()) {
        if v.is_finite() {
            v_max = v_max.max(*v);
        }
    }
    let v_hi = v_max * 1.15;
    let v_lo = -0.10_f64.min(-0.10 * v_max.abs());

    let mut top = ChartBuilder::on(&panels[0])
        .caption(
            "Panel A — residual traces from two independent tapes (engine → tape non-deterministic)",
            ("sans-serif", 16),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, v_lo..v_hi)?;
    top.configure_mesh()
        .x_desc("t since first poll (s)")
        .y_desc("plan_regression residual")
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    let a_col = RGBColor(31, 119, 180);
    let b_col = RGBColor(214, 39, 40);
    top.draw_series(LineSeries::new(
        tape_a_trace.t.iter().zip(tape_a_trace.v).map(|(t, v)| (*t, *v)),
        a_col.stroke_width(2),
    ))?
    .label(format!(
        "{} tape SHA {}…",
        tape_a_label, tape_a_sha_prefix
    ))
    .legend(move |(x, y)| PathElement::new([(x, y), (x + 18, y)], a_col.stroke_width(2)));
    top.draw_series(LineSeries::new(
        tape_b_trace.t.iter().zip(tape_b_trace.v).map(|(t, v)| (*t, *v)),
        b_col.stroke_width(2),
    ))?
    .label(format!(
        "{} tape SHA {}…",
        tape_b_label, tape_b_sha_prefix
    ))
    .legend(move |(x, y)| PathElement::new([(x, y), (x + 18, y)], b_col.stroke_width(2)));
    top.configure_series_labels()
        .background_style(WHITE.mix(0.8).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    // --- Panel B: episodes from each tape's replay ---
    let mut bot = ChartBuilder::on(&panels[1])
        .caption(
            "Panel B — episodes emitted by replay of each tape (tape → episodes byte-stable)",
            ("sans-serif", 16),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(70)
        .build_cartesian_2d(t_min..t_max, 0.0_f64..2.0_f64)?;
    bot.configure_mesh()
        .x_desc("t (s)")
        .y_desc("tape lane")
        .disable_y_mesh()
        .light_line_style(RGBAColor(200, 200, 200, 0.3))
        .draw()?;
    // lane 1 (bottom): tape A
    for ep in tape_a_episodes {
        let col = motif_color(ep.motif);
        bot.draw_series(std::iter::once(Rectangle::new(
            [(ep.t_start, 0.1), (ep.t_end, 0.9)],
            col.mix(0.4).filled(),
        )))?;
    }
    // lane 2 (top): tape B
    for ep in tape_b_episodes {
        let col = motif_color(ep.motif);
        bot.draw_series(std::iter::once(Rectangle::new(
            [(ep.t_start, 1.1), (ep.t_end, 1.9)],
            col.mix(0.4).filled(),
        )))?;
    }
    bot.draw_series(LineSeries::new(
        [(t_min, 1.0), (t_max, 1.0)],
        BLACK.mix(0.3).stroke_width(1),
    ))?;
    // text annotations for episode fingerprints
    bot.draw_series(std::iter::once(Text::new(
        format!("{} — episode fingerprint {}…", tape_a_label, tape_a_episode_fp_prefix),
        (t_min + (t_max - t_min) * 0.02, 0.08),
        ("sans-serif", 13).into_font().color(&BLACK.mix(0.75)),
    )))?;
    bot.draw_series(std::iter::once(Text::new(
        format!("{} — episode fingerprint {}…", tape_b_label, tape_b_episode_fp_prefix),
        (t_min + (t_max - t_min) * 0.02, 1.08),
        ("sans-serif", 13).into_font().color(&BLACK.mix(0.75)),
    )))?;

    // motif legend
    let mut seen: Vec<&str> = Vec::new();
    for ep in tape_a_episodes.iter().chain(tape_b_episodes.iter()) {
        if !seen.contains(&ep.motif) {
            seen.push(ep.motif);
            let col = motif_color(ep.motif);
            let label = ep.motif.to_string();
            bot.draw_series(std::iter::once(Rectangle::new(
                [(t_max - 0.001, -1.0), (t_max, -0.999)],
                col.mix(0.4).filled(),
            )))?
            .label(label)
            .legend(move |(x, y)| {
                Rectangle::new([(x, y - 5), (x + 18, y + 5)], col.mix(0.55).filled())
            });
        }
    }
    bot.configure_series_labels()
        .background_style(WHITE.mix(0.8).filled())
        .border_style(BLACK.mix(0.4))
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;
    Ok(())
}
