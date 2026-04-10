//! SVG figure generation for DSFB gas turbine evaluation.

use crate::pipeline::engine_eval::EngineEvalResult;
use crate::pipeline::metrics::FleetMetrics;
use crate::core::grammar::GrammarState;
use std::fmt::Write;

const GREEN: &str = "#2d7d46";
const AMBER: &str = "#d4a017";
const RED: &str = "#c0392b";
const GRAY: &str = "#ccc";
const DARK: &str = "#333";
const LIGHT: &str = "#666";

/// Generates an SVG grammar-state trajectory for one engine.
pub fn grammar_trajectory_svg(result: &EngineEvalResult) -> String {
    let n = result.grammar_trajectory.len();
    if n == 0 { return String::new(); }
    let w = 800.0f64;
    let h = 200.0f64;
    let margin = 40.0;
    let plot_w = w - 2.0 * margin;
    let plot_h = h - 2.0 * margin;
    let x_scale = plot_w / n as f64;

    let mut svg = String::with_capacity(8192);
    let _ = writeln!(svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" font-family=\"monospace\" font-size=\"10\">", w, h);
    let _ = writeln!(svg, "<rect width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"{}\"/>", w, h, GRAY);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"15\" font-size=\"12\" font-weight=\"bold\">Engine Unit {} - Grammar-State Trajectory ({} cycles)</text>", margin, result.unit, n);

    for (i, state) in result.grammar_trajectory.iter().enumerate() {
        let x = margin + i as f64 * x_scale;
        let color = match state {
            GrammarState::Admissible => GREEN,
            GrammarState::Boundary => AMBER,
            GrammarState::Violation => RED,
        };
        let bw = x_scale.max(1.0);
        let _ = writeln!(svg, "<rect x=\"{:.1}\" y=\"{}\" width=\"{:.1}\" height=\"{}\" fill=\"{}\" opacity=\"0.7\"/>", x, margin, bw, plot_h, color);
    }

    if let Some(fb) = result.first_boundary_cycle {
        let x = margin + (fb as f64 - 1.0) * x_scale;
        let y2 = margin + plot_h;
        let _ = writeln!(svg, "<line x1=\"{:.1}\" y1=\"{}\" x2=\"{:.1}\" y2=\"{}\" stroke=\"{}\" stroke-dasharray=\"4,2\"/>", x, margin, x, y2, DARK);
        let _ = writeln!(svg, "<text x=\"{:.1}\" y=\"{}\" font-size=\"8\" fill=\"{}\">B@{}</text>", x + 2.0, margin - 3.0, DARK, fb);
    }
    if let Some(fv) = result.first_violation_cycle {
        let x = margin + (fv as f64 - 1.0) * x_scale;
        let y2 = margin + plot_h;
        let _ = writeln!(svg, "<line x1=\"{:.1}\" y1=\"{}\" x2=\"{:.1}\" y2=\"{}\" stroke=\"{}\" stroke-dasharray=\"4,2\"/>", x, margin, x, y2, RED);
        let _ = writeln!(svg, "<text x=\"{:.1}\" y=\"{}\" font-size=\"8\" fill=\"{}\">V@{}</text>", x + 2.0, margin - 3.0, RED, fv);
    }
    if let Some(lt) = result.structural_lead_time {
        let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"9\" fill=\"{}\">Lead time: {} cycles</text>", w - margin - 120.0, h - 5.0, DARK, lt);
    }
    let ly = h - 8.0;
    let _ = writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"8\" height=\"8\" fill=\"{}\"/>", margin, ly, GREEN);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\">Admissible</text>", margin + 12.0, ly + 7.0);
    let _ = writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"8\" height=\"8\" fill=\"{}\"/>", margin + 75.0, ly, AMBER);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\">Boundary</text>", margin + 87.0, ly + 7.0);
    let _ = writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"8\" height=\"8\" fill=\"{}\"/>", margin + 145.0, ly, RED);
    let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" font-size=\"8\">Violation</text>", margin + 157.0, ly + 7.0);
    svg.push_str("</svg>");
    svg
}

/// Generates a fleet summary metrics SVG.
pub fn fleet_summary_svg(metrics: &FleetMetrics, dataset_name: &str) -> String {
    let mut svg = String::with_capacity(4096);
    let _ = writeln!(svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 600 280\" font-family=\"monospace\" font-size=\"11\">");
    let _ = writeln!(svg, "<rect width=\"600\" height=\"280\" fill=\"white\" stroke=\"{}\"/>", GRAY);
    let _ = writeln!(svg, "<text x=\"20\" y=\"25\" font-size=\"14\" font-weight=\"bold\">DSFB Fleet Evaluation - {}</text>", dataset_name);
    let _ = writeln!(svg, "<text x=\"20\" y=\"45\" fill=\"{}\" font-size=\"9\">Non-interference contract: v1.0-read-only-observer-only</text>", LIGHT);

    let lines = [
        format!("Engines evaluated:           {}", metrics.total_engines),
        format!("Engines with Boundary:       {} ({:.1}%)", metrics.engines_with_boundary, 100.0 * metrics.engines_with_boundary as f64 / metrics.total_engines.max(1) as f64),
        format!("Engines with Violation:       {} ({:.1}%)", metrics.engines_with_violation, 100.0 * metrics.engines_with_violation as f64 / metrics.total_engines.max(1) as f64),
        format!("Mean structural lead time:   {:.1} cycles", metrics.mean_lead_time),
        format!("Median structural lead time: {:.1} cycles", metrics.median_lead_time),
        format!("Min / Max lead time:         {} / {} cycles", metrics.min_lead_time, metrics.max_lead_time),
        format!("Total episodes:              {}", metrics.total_episodes),
        format!("Mean episodes per engine:    {:.2}", metrics.mean_episodes_per_engine),
        format!("Theorem 1 satisfaction rate: {:.1}%", 100.0 * metrics.theorem_satisfaction_rate),
        format!("Early warning (>30 RUL):     {} ({:.1}%)", metrics.early_warning_count, 100.0 * metrics.early_warning_count as f64 / metrics.total_engines.max(1) as f64),
    ];
    for (i, line) in lines.iter().enumerate() {
        let y = 70 + i * 20;
        let _ = writeln!(svg, "<text x=\"20\" y=\"{}\">{}</text>", y, line);
    }
    svg.push_str("</svg>");
    svg
}
