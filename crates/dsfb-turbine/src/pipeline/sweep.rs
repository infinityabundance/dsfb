//! P0/P3: Automated sensitivity sweep with heatmap generation.
//!
//! Varies `envelope_sigma` and `persistence_threshold` independently
//! and jointly, recording first-Boundary cycle, first-Violation cycle,
//! and structural lead time for each configuration.
//!
//! Produces a sweep table and SVG heatmap of first-Boundary-cycle
//! vs (envelope_sigma, persistence_threshold).

use crate::core::config::DsfbConfig;
use crate::core::channels::INFORMATIVE_CHANNELS_FD001;
use crate::dataset::cmapss::CmapssDataset;
use crate::pipeline::fleet::evaluate_fleet;
use std::fmt::Write;

/// A single sweep result row.
#[derive(Debug, Clone)]
pub struct SweepPoint {
    /// Envelope sigma value.
    pub envelope_sigma: f64,
    /// Persistence threshold value.
    pub persistence_threshold: usize,
    /// Median first-Boundary cycle across fleet.
    pub median_first_boundary: f64,
    /// Median first-Violation cycle across fleet.
    pub median_first_violation: f64,
    /// Median structural lead time (cycles before end-of-life).
    pub median_lead_time: f64,
    /// Mean structural lead time.
    pub mean_lead_time: f64,
    /// Fraction of engines with at least one Boundary episode.
    pub boundary_detection_rate: f64,
    /// Fraction of engines with early warning (>30 RUL at Boundary).
    pub early_warning_rate: f64,
    /// False Boundary rate in healthy window (negative control).
    pub false_boundary_rate: f64,
}

/// Full sweep result.
#[derive(Debug)]
pub struct SweepResult {
    /// All sweep points.
    pub points: Vec<SweepPoint>,
    /// Recommended configuration (best trade-off).
    pub recommended_config: DsfbConfig,
    /// Recommended point index.
    pub recommended_idx: usize,
}

/// Envelope sigma sweep values (P0).
const SIGMA_SWEEP: &[f64] = &[2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];

/// Persistence threshold sweep values (P0).
const PERSIST_SWEEP: &[usize] = &[5, 10, 15, 20, 25, 30, 35, 40];

/// Runs the full 2D sensitivity sweep across envelope_sigma × persistence_threshold.
///
/// For each (sigma, persistence) pair, evaluates the full fleet and records metrics.
pub fn run_2d_sweep(
    dataset: &CmapssDataset,
    base_config: &DsfbConfig,
) -> SweepResult {
    let mut points = Vec::with_capacity(SIGMA_SWEEP.len() * PERSIST_SWEEP.len());

    for &sigma in SIGMA_SWEEP {
        for &persist in PERSIST_SWEEP {
            let config = DsfbConfig {
                envelope_sigma: sigma,
                persistence_threshold: persist,
                ..*base_config
            };

            let (results, metrics) = evaluate_fleet(dataset, &config, INFORMATIVE_CHANNELS_FD001);

            // Compute median first-boundary and first-violation
            let mut boundary_cycles: Vec<u32> = results.iter()
                .filter_map(|r| r.first_boundary_cycle)
                .collect();
            boundary_cycles.sort();
            let median_fb = if boundary_cycles.is_empty() {
                f64::NAN
            } else {
                boundary_cycles[boundary_cycles.len() / 2] as f64
            };

            let mut violation_cycles: Vec<u32> = results.iter()
                .filter_map(|r| r.first_violation_cycle)
                .collect();
            violation_cycles.sort();
            let median_fv = if violation_cycles.is_empty() {
                f64::NAN
            } else {
                violation_cycles[violation_cycles.len() / 2] as f64
            };

            // False boundary rate in healthy window
            let false_boundary = compute_false_boundary_rate(&results, &config);

            points.push(SweepPoint {
                envelope_sigma: sigma,
                persistence_threshold: persist,
                median_first_boundary: median_fb,
                median_first_violation: median_fv,
                median_lead_time: metrics.median_lead_time,
                mean_lead_time: metrics.mean_lead_time,
                boundary_detection_rate: metrics.engines_with_boundary as f64
                    / metrics.total_engines.max(1) as f64,
                early_warning_rate: metrics.early_warning_count as f64
                    / metrics.total_engines.max(1) as f64,
                false_boundary_rate: false_boundary,
            });
        }
    }

    // Find recommended configuration:
    // Target: median lead time 40–100 cycles, 100% detection, low false-boundary rate.
    let recommended_idx = find_recommended(&points);
    let rp = &points[recommended_idx];
    let recommended_config = DsfbConfig {
        envelope_sigma: rp.envelope_sigma,
        persistence_threshold: rp.persistence_threshold,
        ..*base_config
    };

    SweepResult {
        points,
        recommended_config,
        recommended_idx,
    }
}

/// Finds the recommended configuration from sweep points.
///
/// Scoring: maximize detection_rate × early_warning_rate × (1 - false_boundary_rate),
/// subject to median_lead_time being in operationally meaningful range (30–120 cycles).
fn find_recommended(points: &[SweepPoint]) -> usize {
    let mut best_idx = 0;
    let mut best_score = f64::NEG_INFINITY;

    for (i, p) in points.iter().enumerate() {
        // Penalize if median lead time is too early (>150) or too late (<20)
        let lead_score = if p.median_lead_time >= 30.0 && p.median_lead_time <= 120.0 {
            1.0
        } else if p.median_lead_time > 120.0 {
            120.0 / p.median_lead_time
        } else if p.median_lead_time < 30.0 && p.median_lead_time > 0.0 {
            p.median_lead_time / 30.0
        } else {
            0.01
        };

        let score = p.boundary_detection_rate
            * p.early_warning_rate
            * (1.0 - p.false_boundary_rate)
            * lead_score;

        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    best_idx
}

/// P4: Computes false-Boundary rate in the known-healthy window.
///
/// For each engine, checks whether DSFB emits a Boundary or Violation
/// state during the first `healthy_window` cycles (which are, by construction,
/// the healthy reference period). Any such emission is a false alarm.
fn compute_false_boundary_rate(
    results: &[crate::pipeline::engine_eval::EngineEvalResult],
    config: &DsfbConfig,
) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let mut false_alarms = 0u32;
    let hw = config.healthy_window;

    for result in results {
        // Check if any grammar state in the healthy window is non-Admissible
        for (k, state) in result.grammar_trajectory.iter().enumerate() {
            if k >= hw {
                break;
            }
            if *state != crate::core::grammar::GrammarState::Admissible {
                false_alarms += 1;
                break; // Count each engine at most once
            }
        }
    }

    false_alarms as f64 / results.len() as f64
}

/// Generates the sweep result as a text table.
pub fn sweep_table(result: &SweepResult) -> String {
    let mut out = String::with_capacity(8192);
    let _ = writeln!(out, "── DSFB Sensitivity Sweep (envelope_sigma × persistence_threshold) ──");
    let _ = writeln!(out);
    let _ = writeln!(out, "{:>7} {:>7} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "sigma", "persist", "med_fb", "med_fv", "med_lead", "det%", "ew%", "false%");
    let _ = writeln!(out, "{}", "─".repeat(80));

    for (i, p) in result.points.iter().enumerate() {
        let marker = if i == result.recommended_idx { " ◀ RECOMMENDED" } else { "" };
        let _ = writeln!(out, "{:7.1} {:7} {:10.1} {:10.1} {:10.1} {:7.1}% {:7.1}% {:7.1}%{}",
            p.envelope_sigma,
            p.persistence_threshold,
            p.median_first_boundary,
            p.median_first_violation,
            p.median_lead_time,
            p.boundary_detection_rate * 100.0,
            p.early_warning_rate * 100.0,
            p.false_boundary_rate * 100.0,
            marker,
        );
    }

    let _ = writeln!(out);
    let rp = &result.points[result.recommended_idx];
    let _ = writeln!(out, "Recommended: envelope_sigma={:.1}, persistence_threshold={}",
        rp.envelope_sigma, rp.persistence_threshold);
    let _ = writeln!(out, "  Median lead time: {:.1} cycles", rp.median_lead_time);
    let _ = writeln!(out, "  Detection rate: {:.1}%", rp.boundary_detection_rate * 100.0);
    let _ = writeln!(out, "  Early warning rate: {:.1}%", rp.early_warning_rate * 100.0);
    let _ = writeln!(out, "  False boundary rate: {:.1}%", rp.false_boundary_rate * 100.0);

    out
}

/// P3: Generates an SVG heatmap of median_lead_time vs (sigma, persistence).
pub fn sweep_heatmap_svg(result: &SweepResult) -> String {
    let mut svg = String::with_capacity(16384);
    let w = 700.0f64;
    let h = 500.0f64;
    let margin_l = 80.0;
    let margin_r = 120.0;
    let margin_t = 50.0;
    let margin_b = 60.0;
    let plot_w = w - margin_l - margin_r;
    let plot_h = h - margin_t - margin_b;

    let n_sigma = SIGMA_SWEEP.len();
    let n_persist = PERSIST_SWEEP.len();
    let cell_w = plot_w / n_persist as f64;
    let cell_h = plot_h / n_sigma as f64;

    let _ = writeln!(svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" font-family=\"monospace\" font-size=\"10\">", w, h);
    let _ = writeln!(svg, "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>", w, h);

    // Title
    let _ = writeln!(svg, "<text x=\"{}\" y=\"25\" font-size=\"13\" font-weight=\"bold\" text-anchor=\"middle\">DSFB Sensitivity: Median Structural Lead Time (cycles)</text>", w / 2.0);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"40\" font-size=\"9\" fill=\"{}\" text-anchor=\"middle\">C-MAPSS FD001 | 100 engines | Read-only observer</text>", w / 2.0, "#666");

    // Axis labels
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"11\" text-anchor=\"middle\">persistence_threshold</text>",
        margin_l + plot_w / 2.0, h - 10.0);
    let _ = writeln!(svg, "<text x=\"15\" y=\"{}\" font-size=\"11\" text-anchor=\"middle\" transform=\"rotate(-90,15,{})\">envelope_sigma</text>",
        margin_t + plot_h / 2.0, margin_t + plot_h / 2.0);

    // Find min/max lead time for color scaling
    let mut min_lead = f64::MAX;
    let mut max_lead = f64::MIN;
    for p in &result.points {
        if p.median_lead_time.is_finite() {
            if p.median_lead_time < min_lead { min_lead = p.median_lead_time; }
            if p.median_lead_time > max_lead { max_lead = p.median_lead_time; }
        }
    }
    if max_lead <= min_lead { max_lead = min_lead + 1.0; }

    // Draw heatmap cells
    for (si, &_sigma) in SIGMA_SWEEP.iter().enumerate() {
        for (pi, &_persist) in PERSIST_SWEEP.iter().enumerate() {
            // Find corresponding sweep point
            let idx = si * PERSIST_SWEEP.len() + pi;
            if idx >= result.points.len() { continue; }
            let p = &result.points[idx];

            let x = margin_l + pi as f64 * cell_w;
            let y = margin_t + si as f64 * cell_h;

            // Color: blue (short lead) → green (medium) → red (long)
            let norm = if p.median_lead_time.is_finite() {
                ((p.median_lead_time - min_lead) / (max_lead - min_lead)).clamp(0.0, 1.0)
            } else { 0.5 };

            let (r, g, b) = if norm < 0.5 {
                // Blue to green
                let t = norm * 2.0;
                ((50.0 * (1.0 - t)) as u8, (100.0 + 155.0 * t) as u8, (200.0 * (1.0 - t)) as u8)
            } else {
                // Green to red
                let t = (norm - 0.5) * 2.0;
                ((50.0 + 180.0 * t) as u8, (255.0 * (1.0 - t)) as u8, 30u8)
            };

            let is_recommended = idx == result.recommended_idx;
            let stroke = if is_recommended { "stroke=\"black\" stroke-width=\"3\"" } else { "stroke=\"white\" stroke-width=\"1\"" };

            let _ = writeln!(svg, "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"rgb({},{},{})\" {} opacity=\"0.85\"/>",
                x, y, cell_w, cell_h, r, g, b, stroke);

            // Cell text: lead time value
            let text_color = if norm > 0.3 && norm < 0.7 { "black" } else { "white" };
            let _ = writeln!(svg, "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"9\" fill=\"{}\" text-anchor=\"middle\">{:.0}</text>",
                x + cell_w / 2.0, y + cell_h / 2.0 + 3.0, text_color, p.median_lead_time);
        }
    }

    // X-axis tick labels (persistence)
    for (pi, &persist) in PERSIST_SWEEP.iter().enumerate() {
        let x = margin_l + pi as f64 * cell_w + cell_w / 2.0;
        let _ = writeln!(svg, "<text x=\"{:.1}\" y=\"{}\" font-size=\"9\" text-anchor=\"middle\">{}</text>",
            x, margin_t + plot_h + 15.0, persist);
    }

    // Y-axis tick labels (sigma)
    for (si, &sigma) in SIGMA_SWEEP.iter().enumerate() {
        let y = margin_t + si as f64 * cell_h + cell_h / 2.0 + 3.0;
        let _ = writeln!(svg, "<text x=\"{}\" y=\"{:.1}\" font-size=\"9\" text-anchor=\"end\">{:.1}</text>",
            margin_l - 8.0, y, sigma);
    }

    // Color bar legend
    let bar_x = w - margin_r + 20.0;
    let bar_w = 15.0;
    let bar_h = plot_h;
    for i in 0..50 {
        let frac = i as f64 / 49.0;
        let y = margin_t + frac * bar_h;
        let (r, g, b) = if frac < 0.5 {
            let t = frac * 2.0;
            ((50.0 * (1.0 - t)) as u8, (100.0 + 155.0 * t) as u8, (200.0 * (1.0 - t)) as u8)
        } else {
            let t = (frac - 0.5) * 2.0;
            ((50.0 + 180.0 * t) as u8, (255.0 * (1.0 - t)) as u8, 30u8)
        };
        let _ = writeln!(svg, "<rect x=\"{}\" y=\"{:.1}\" width=\"{}\" height=\"{:.1}\" fill=\"rgb({},{},{})\"/>",
            bar_x, y, bar_w, bar_h / 50.0 + 1.0, r, g, b);
    }
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\">{:.0}</text>", bar_x + bar_w + 4.0, margin_t + 8.0, min_lead);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\">{:.0}</text>", bar_x + bar_w + 4.0, margin_t + bar_h, max_lead);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\" transform=\"rotate(90,{},{})\" text-anchor=\"middle\">Lead (cycles)</text>",
        bar_x + bar_w + 25.0, margin_t + bar_h / 2.0, bar_x + bar_w + 25.0, margin_t + bar_h / 2.0);

    svg.push_str("</svg>");
    svg
}

/// Generates the sweep result as JSON for reproducibility.
pub fn sweep_json(result: &SweepResult) -> String {
    let mut json = String::with_capacity(8192);
    json.push_str("[\n");
    for (i, p) in result.points.iter().enumerate() {
        let _ = write!(json, "  {{\"envelope_sigma\":{:.1},\"persistence_threshold\":{},\"median_first_boundary\":{:.1},\"median_first_violation\":{:.1},\"median_lead_time\":{:.1},\"mean_lead_time\":{:.1},\"boundary_detection_rate\":{:.4},\"early_warning_rate\":{:.4},\"false_boundary_rate\":{:.4},\"recommended\":{}}}",
            p.envelope_sigma, p.persistence_threshold,
            p.median_first_boundary, p.median_first_violation,
            p.median_lead_time, p.mean_lead_time,
            p.boundary_detection_rate, p.early_warning_rate,
            p.false_boundary_rate, i == result.recommended_idx);
        if i + 1 < result.points.len() { json.push(','); }
        json.push('\n');
    }
    json.push_str("]\n");
    json
}
