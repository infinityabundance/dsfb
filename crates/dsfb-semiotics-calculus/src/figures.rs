//! # Figure Generation (`dsfb-semiotics-calculus`)
//!
//! Produces the ten canonical figures that characterize the DSSC framework.
//! All figures are pure SVG — no external image libraries required.
//! Each figure is self-contained and reproducible from the crate's core logic.
//!
//! ## Figure catalogue
//!
//! | # | Name | What it shows |
//! |---|------|--------------|
//! | 1 | `residual_sign_triple` | The (‖r‖, ṙ, r̈) triple across a synthetic trajectory |
//! | 2 | `admissibility_envelope` | ρ_min / ρ_max / δ-band geometry with region annotation |
//! | 3 | `grammar_fsm_diagram` | Adm → Bdy → Vio state machine with transition labels |
//! | 4 | `grammar_state_trajectory` | Grammar state sequence overlaid on residual trace |
//! | 5 | `persistence_counter` | Consecutive-state dwell counts per grammar episode |
//! | 6 | `endoductive_operator_flow` | Data-flow diagram of ℰ: trajectory → episode |
//! | 7 | `provenance_tag_anatomy` | Breakdown of ProvenanceTag fields in a single episode |
//! | 8 | `bank_monotonicity` | Bank size vs. augmentation step (Theorem 7.2) |
//! | 9 | `observer_noninterference` | Read-only observer vs. mutable host (structural proof) |
//! | 10 | `cross_stream_fusion` | GrammarFusion (G₁ ⋈ G₂) on two concurrent streams |

#![allow(non_snake_case, missing_docs)]  // W/H are idiomatic for width/height in SVG; figure fns documented in module-level catalogue table

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{AdmissibilityEnvelope, GrammarFsm, GrammarState, ResidualSign};

// ─── colour palette ────────────────────────────────────────────────────────
const C_ADM: &str = "#1a7a4a";   // green — Admissible
const C_BDY: &str = "#c47c00";   // amber — Boundary
const C_VIO: &str = "#c0392b";   // red   — Violation
const C_MID: &str = "#2c3e50";   // near-black
const C_GRID: &str = "#e8e8e8";  // light grey
const C_BG: &str = "#fafafa";
const C_ACCENT: &str = "#2980b9";
const C_GREY: &str = "#555555";
const C_MUTED: &str = "#888888";
const C_LTRED: &str = "#fff5f5";
const C_LTGRN: &str = "#f0fff4";
const C_LINEN: &str = "white";
const C_BORD: &str = "#dddddd";
const C_PALE: &str = "#cccccc";

// ─── helpers ───────────────────────────────────────────────────────────────

fn svg_open(w: f64, h: f64) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" \
width="{w}" height="{h}" font-family="'DejaVu Sans',Arial,sans-serif" \
font-size="11" fill="{C_MID}" background="{C_BG}">"#,
        w = w, h = h
    )
}

fn svg_close() -> &'static str { "</svg>" }

fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str, sw: f64) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        x, y, w, h, fill, stroke, sw
    )
}

fn line(x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, sw: f64) -> String {
    format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
        x1, y1, x2, y2, stroke, sw
    )
}

fn polyline(pts: &[(f64, f64)], stroke: &str, sw: f64, fill: &str) -> String {
    let pts_str: String = pts
        .iter()
        .map(|(x, y)| format!("{:.2},{:.2}", x, y))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"<polyline points="{}" stroke="{}" stroke-width="{}" fill="{}"/>"#,
        pts_str, stroke, sw, fill
    )
}

fn text(x: f64, y: f64, anchor: &str, size: f64, bold: bool, color: &str, content: &str) -> String {
    let weight = if bold { "bold" } else { "normal" };
    format!(
        r#"<text x="{}" y="{}" text-anchor="{}" font-size="{}" font-weight="{}" fill="{}">{}</text>"#,
        x, y, anchor, size, weight, color, content
    )
}

fn circle(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, sw: f64) -> String {
    format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        cx, cy, r, fill, stroke, sw
    )
}

fn chart_bg(x: f64, y: f64, w: f64, h: f64, n_hgrid: usize) -> String {
    let mut s = String::new();
    s.push_str(&rect(x, y, w, h, C_BG, C_GRID, 1.0));
    for i in 0..=n_hgrid {
        let gy = y + h * i as f64 / n_hgrid as f64;
        s.push_str(&line(x, gy, x + w, gy, C_GRID, 1.0));
    }
    s
}

fn caption(cx: f64, y: f64, fig_num: usize, title: &str, subtitle: &str) -> String {
    format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" font-weight=\"bold\" fill=\"{C_MID}\">Figure {}. {}</text>\n<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"9\" fill=\"{C_GREY}\">{}</text>",
        cx, y, fig_num, title,
        cx, y + 14.0, subtitle
    )
}

fn state_color(g: GrammarState) -> &'static str {
    match g {
        GrammarState::Admissible => C_ADM,
        GrammarState::Boundary   => C_BDY,
        GrammarState::Violation  => C_VIO,
    }
}

// ─── synthetic trajectory ──────────────────────────────────────────────────

/// Generate a realistic synthetic residual trajectory for demonstration.
///
/// Returns residuals in [0.0, 1.4] with:
/// - nominal phase (steps 0–11): small bounded oscillation
/// - boundary phase (steps 12–16): gradual approach to ρ_max = 1.0
/// - violation bursts (steps 17–20, 34–38): excursions beyond ρ_max
/// - recovery phase following each violation
/// - second nominal run (steps 21–33): stable regime
pub fn synthetic_trajectory() -> Vec<f64> {
    vec![
        // Nominal phase
        0.15, 0.22, 0.18, 0.28, 0.20, 0.25, 0.19, 0.30, 0.22, 0.26, 0.21, 0.24,
        // Boundary approach
        0.55, 0.68, 0.78, 0.85, 0.92,
        // Violation burst 1
        1.05, 1.18, 1.10, 1.22,
        // Recovery to nominal
        0.80, 0.60, 0.40, 0.28, 0.22, 0.19, 0.25, 0.20, 0.27, 0.21, 0.23, 0.20, 0.22,
        // Second violation burst
        0.70, 0.88, 1.04, 1.15, 1.08,
        // Final recovery
        0.72, 0.50, 0.30, 0.22, 0.18,
    ]
}

/// Compute grammar state sequence for a trajectory.
fn compute_grammar_trace(
    residuals: &[f64],
    envelope: &AdmissibilityEnvelope,
) -> (Vec<ResidualSign>, Vec<GrammarState>) {
    let mut fsm = GrammarFsm::new();
    let mut signs = Vec::new();
    let mut states = Vec::new();
    let mut prev = 0.0_f64;
    let mut prev2 = 0.0_f64;
    for &r in residuals {
        let sign = ResidualSign::from_scalar(r, prev, prev2);
        let state = fsm.step(&sign, envelope);
        signs.push(sign);
        states.push(state);
        prev2 = prev;
        prev = r;
    }
    (signs, states)
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 1 — Residual Sign Triple
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_01_residual_sign_triple() -> String {
    let traj = synthetic_trajectory();
    let n = traj.len();
    let env = AdmissibilityEnvelope::new(0.1, 1.0, 0.02);
    let (signs, _) = compute_grammar_trace(&traj, &env);

    let (W, H) = (760.0_f64, 420.0_f64);
    let (lm, tm, rm, bm) = (60.0, 50.0, 20.0, 70.0);
    let cw = W - lm - rm;
    let ch = (H - tm - bm - 30.0) / 3.0;

    let mag: Vec<f64> = signs.iter().map(|s| s.magnitude).collect();
    let drift: Vec<f64> = signs.iter().map(|s| s.drift).collect();
    let slew: Vec<f64> = signs.iter().map(|s| s.slew).collect();

    let mag_max = mag.iter().cloned().fold(0.0_f64, f64::max) * 1.1;
    let drift_abs = drift.iter().cloned().map(f64::abs).fold(0.0_f64, f64::max) * 1.2;
    let slew_abs = slew.iter().cloned().map(f64::abs).fold(0.0_f64, f64::max) * 1.2;

    let px = |i: usize| lm + i as f64 * cw / (n - 1) as f64;
    let py = |val: f64, lo: f64, hi: f64, top: f64| top + ch - (val - lo) / (hi - lo) * ch;

    let panel = |top: f64, vals: &[f64], lo: f64, hi: f64, color: &str, label: &str| -> String {
        let mut s = chart_bg(lm, top, cw, ch, 4);
        let pts: Vec<(f64, f64)> = vals
            .iter()
            .enumerate()
            .map(|(i, &v)| (px(i), py(v, lo, hi, top)))
            .collect();
        s.push_str(&polyline(&pts, color, 1.8, "none"));
        for (i, &v) in vals.iter().enumerate() {
            s.push_str(&circle(px(i), py(v, lo, hi, top), 2.2, color, color, 0.0));
        }
        // y-axis label
        s.push_str(&text(lm - 8.0, top + ch / 2.0, "end", 10.0, true, C_MID, label));
        // y-axis zero line (if range spans 0)
        if lo < 0.0 && hi > 0.0 {
            let z = py(0.0, lo, hi, top);
            s.push_str(&line(lm, z, lm + cw, z, "#aaaaaa", 0.8));
        }
        s
    };

    let t1 = tm;
    let t2 = tm + ch + 10.0;
    let t3 = tm + 2.0 * (ch + 10.0);

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Residual Sign Triple: σ(k) = (‖r(k)‖, ṙ(k), r̈(k))"));

    svg.push_str(&panel(t1, &mag, 0.0, mag_max.max(0.01), C_ACCENT, "‖r(k)‖"));
    svg.push_str(&panel(t2, &drift, -drift_abs, drift_abs, C_VIO, "ṙ(k)"));
    svg.push_str(&panel(t3, &slew, -slew_abs, slew_abs, C_BDY, "r̈(k)"));

    // x-axis labels
    for i in (0..n).step_by(5) {
        svg.push_str(&text(px(i), H - bm + 14.0, "middle", 9.0, false, C_MID, &format!("{}", i)));
    }
    svg.push_str(&text(W / 2.0, H - bm + 30.0, "middle", 10.0, false, "#555", "step k"));

    svg.push_str(&caption(W / 2.0, H - 5.0, 1,
        "Residual Sign Triple",
        "Magnitude (top), drift ṙ (middle), slew r̈ (bottom) across synthetic trajectory"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 2 — Admissibility Envelope Geometry
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_02_admissibility_envelope() -> String {
    let (W, H) = (640.0_f64, 420.0_f64);
    let cx = W / 2.0;
    let cy = H / 2.0 - 20.0;
    let scale = 160.0; // pixels per unit
    let rho_min = 0.25;
    let rho_max = 1.0;
    let delta = 0.04;

    let r_inner = rho_min * scale;
    let r_outer = rho_max * scale;
    let r_bdy_lo = (rho_max - delta) * scale;
    let r_bdy_hi = (rho_max + delta) * scale;

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Admissibility Envelope: E ⊆ V with B(0,ρ_min) ⊆ E ⊆ B(0,ρ_max)"));

    // Exterior label
    svg.push_str(&text(cx + r_bdy_hi + 14.0, cy - 14.0, "start", 11.0, true, C_VIO, "Exterior (Vio)"));

    // δ-band annulus (drawn as two filled circles with background showing through)
    svg.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}33" stroke="{}" stroke-width="1.5" stroke-dasharray="5,3"/>"#,
        cx, cy, r_bdy_hi, C_VIO, C_VIO));
    svg.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}55" stroke="{}" stroke-width="1.5" stroke-dasharray="5,3"/>"#,
        cx, cy, r_bdy_lo, C_BDY, C_BDY));

    // Interior (Adm)
    svg.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}44" stroke="{}" stroke-width="2"/>"#,
        cx, cy, r_inner, C_ADM, C_ADM));

    // The main envelope boundary
    svg.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="2.5"/>"#,
        cx, cy, r_outer, C_MID));

    // Centre
    svg.push_str(&circle(cx, cy, 4.0, C_MID, C_MID, 0.0));
    svg.push_str(&text(cx + 5.0, cy - 6.0, "start", 10.0, false, C_MID, "0 ∈ int(E)"));

    // Radii annotations
    let ann = |r: f64, label: &str, color: &str| -> String {
        let px = cx + r / 1.414;
        let py = cy - r / 1.414;
        format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" stroke-dasharray="3,2"/>
<text x="{}" y="{}" font-size="10" fill="{}">{}</text>"#,
            cx, cy, px, py, color,
            px + 4.0, py - 4.0, color, label
        )
    };
    svg.push_str(&ann(r_inner, "ρ_min = 0.25", C_ADM));
    svg.push_str(&ann(r_outer, "ρ_max = 1.0", C_MID));
    svg.push_str(&ann(r_bdy_hi, "ρ_max + δ", C_VIO));

    // Region labels
    svg.push_str(&text(cx - 14.0, cy + r_inner * 0.5, "middle", 10.0, true, C_ADM, "Interior"));
    svg.push_str(&text(cx - 14.0, cy + r_inner * 0.5 + 13.0, "middle", 9.0, false, C_ADM, "(Adm)"));

    svg.push_str(&text(cx + r_bdy_lo * 0.72, cy + r_bdy_lo * 0.55, "middle", 9.0, true, C_BDY, "δ-band"));
    svg.push_str(&text(cx + r_bdy_lo * 0.72, cy + r_bdy_lo * 0.55 + 12.0, "middle", 8.0, false, C_BDY, "(Bdy)"));

    svg.push_str(&caption(W / 2.0, H - 18.0, 2,
        "Admissibility Envelope Geometry",
        "B(0,ρ_min) ⊆ E = B(0,ρ_max); δ-band boundary layer; region → grammar state"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 3 — Grammar FSM State Machine Diagram
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_03_grammar_fsm_diagram() -> String {
    let (W, H) = (640.0_f64, 340.0_f64);
    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 24.0, "middle", 13.0, true, C_MID,
        "Grammar FSM: G = {Adm, Bdy, Vio}, δ: G × Σ → G (total)"));

    // Node centres
    let adm = (160.0_f64, 170.0_f64);
    let bdy = (360.0_f64, 170.0_f64);
    let vio = (560.0_f64, 170.0_f64);
    let r = 50.0_f64;

    // Nodes
    for (c, color, label) in [
        (adm, C_ADM, "Adm"),
        (bdy, C_BDY, "Bdy"),
        (vio, C_VIO, "Vio"),
    ] {
        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}22" stroke="{}" stroke-width="2.5"/>"#,
            c.0, c.1, r, color, color));
        svg.push_str(&text(c.0, c.1 - 5.0, "middle", 13.0, true, color, label));
    }
    // State labels
    svg.push_str(&text(adm.0, adm.1 + 12.0, "middle", 9.0, false, C_ADM, "Nominal"));
    svg.push_str(&text(bdy.0, bdy.1 + 12.0, "middle", 9.0, false, C_BDY, "Early-warning"));
    svg.push_str(&text(vio.0, vio.1 + 12.0, "middle", 9.0, false, C_VIO, "ℰ fires"));

    // Transitions (via curved paths)
    // Adm → Bdy
    svg.push_str(&format!(
        r#"<path d="M {},{} Q {},{} {},{}" fill="none" stroke="{}" stroke-width="1.8" marker-end="url(#arr)"/>
<text x="{}" y="{}" text-anchor="middle" font-size="9" fill="{C_BDY}">Boundary</text>"#,
        adm.0 + r, adm.1 - 6.0,
        (adm.0 + bdy.0) / 2.0, adm.1 - 38.0,
        bdy.0 - r, bdy.1 - 6.0, C_BDY,
        (adm.0 + bdy.0) / 2.0, adm.1 - 48.0));
    // Bdy → Vio
    svg.push_str(&format!(
        r#"<path d="M {},{} Q {},{} {},{}" fill="none" stroke="{}" stroke-width="1.8" marker-end="url(#arr)"/>
<text x="{}" y="{}" text-anchor="middle" font-size="9" fill="{C_VIO}">Exterior</text>"#,
        bdy.0 + r, bdy.1 - 6.0,
        (bdy.0 + vio.0) / 2.0, bdy.1 - 38.0,
        vio.0 - r, vio.1 - 6.0, C_VIO,
        (bdy.0 + vio.0) / 2.0, bdy.1 - 48.0));
    // Bdy → Adm
    svg.push_str(&format!(
        r#"<path d="M {},{} Q {},{} {},{}" fill="none" stroke="{}" stroke-width="1.4" stroke-dasharray="5,3" marker-end="url(#arr)"/>
<text x="{}" y="{}" text-anchor="middle" font-size="9" fill="{C_ADM}">Interior</text>"#,
        bdy.0 - r, bdy.1 + 6.0,
        (adm.0 + bdy.0) / 2.0, bdy.1 + 40.0,
        adm.0 + r, adm.1 + 6.0, C_ADM,
        (adm.0 + bdy.0) / 2.0, bdy.1 + 50.0));
    // Vio → Adm (reset after episode)
    svg.push_str(&format!(
        "<path d=\"M {},{} C {},{} {},{} {},{}\" fill=\"none\" stroke=\"{C_MUTED}\" stroke-width=\"1.2\" stroke-dasharray=\"4,4\" marker-end=\"url(#arr)\"/>\n<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"8\" fill=\"{C_MUTED}\">reset (\u{2130} emitted)</text>",
        vio.0, vio.1 + r,
        vio.0 - 20.0, vio.1 + 90.0,
        adm.0 + 20.0, adm.1 + 90.0,
        adm.0, adm.1 + r,
        (adm.0 + vio.0) / 2.0, vio.1 + 105.0));
    // Self-loops (Adm → Adm, Vio → Adm handled above)
    svg.push_str(&format!(
        r#"<path d="M {},{} a 22 22 0 1 1 1 0" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="4,3"/>
<text x="{}" y="{}" text-anchor="middle" font-size="8" fill="{C_ADM}">Interior</text>"#,
        adm.0 - r + 5.0, adm.1 - 3.0, C_ADM,
        adm.0 - r - 20.0, adm.1 - 42.0));

    // Arrow marker def
    svg.push_str(&format!("<defs><marker id=\"arr\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" orient=\"auto\"><path d=\"M0,0 L6,3 L0,6 Z\" fill=\"{C_GREY}\"/></marker></defs>"));

    svg.push_str(&caption(W / 2.0, H - 18.0, 3,
        "Grammar FSM",
        "Deterministic total transition δ: G × Σ → G — Theorem 3.1 (Totality)"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 4 — Grammar State Trajectory
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_04_grammar_state_trajectory() -> String {
    let traj = synthetic_trajectory();
    let n = traj.len();
    let env = AdmissibilityEnvelope::new(0.1, 1.0, 0.02);
    let (_, states) = compute_grammar_trace(&traj, &env);

    let (W, H) = (760.0_f64, 380.0_f64);
    let (lm, tm, rm, bm) = (60.0, 50.0, 20.0, 70.0);
    let cw = W - lm - rm;
    let ch = H - tm - bm;

    let rho_max = 1.0_f64;
    let delta = 0.02_f64;
    let r_scale = ch / 1.5;

    let px = |i: usize| lm + i as f64 * cw / (n - 1) as f64;
    let py_r = |v: f64| tm + ch - v * r_scale * 0.9;

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Grammar State Sequence Overlaid on Residual Trajectory"));

    svg.push_str(&chart_bg(lm, tm, cw, ch, 5));

    // ρ_max and δ-band guide lines
    let y_bdy_lo = py_r(rho_max - delta);
    let y_bdy_hi = py_r(rho_max + delta);
    let y_rmax = py_r(rho_max);
    svg.push_str(&rect(lm, y_bdy_hi, cw, y_bdy_lo - y_bdy_hi, "#f39c1222", "none", 0.0));
    svg.push_str(&line(lm, y_rmax, lm + cw, y_rmax, C_MID, 1.0));
    svg.push_str(&text(lm + cw + 3.0, y_rmax + 4.0, "start", 9.0, false, C_MID, "ρ_max"));

    // Residual curve with state colouring per segment
    for i in 0..n {
        let x = px(i);
        let yp = py_r(traj[i]);
        let col = state_color(states[i]);
        if i > 0 {
            let xp = px(i - 1);
            let ypp = py_r(traj[i - 1]);
            svg.push_str(&line(xp, ypp, x, yp, col, 2.2));
        }
        svg.push_str(&circle(x, yp, 3.0, col, col, 0.0));
    }

    // Grammar state bands at bottom
    let band_h = 14.0;
    let band_y = tm + ch + 4.0;
    for i in 0..n {
        let x = px(i);
        let col = state_color(states[i]);
        let bw = cw / (n - 1) as f64;
        svg.push_str(&rect(x - bw / 2.0, band_y, bw, band_h, &format!("{}88", col), "none", 0.0));
    }
    // state band labels: find representative positions
    for &(label, state) in &[("Adm", GrammarState::Admissible), ("Bdy", GrammarState::Boundary), ("Vio", GrammarState::Violation)] {
        if let Some(i) = states.iter().position(|&s| s == state) {
            svg.push_str(&text(px(i), band_y + band_h - 2.0, "middle", 8.0, true, state_color(state), label));
        }
    }

    // x-axis labels
    for i in (0..n).step_by(5) {
        svg.push_str(&text(px(i), band_y + band_h + 15.0, "middle", 9.0, false, C_MID, &format!("{}", i)));
    }
    // y-axis ticks
    for &v in &[0.0, 0.5, 1.0] {
        let y = py_r(v);
        svg.push_str(&line(lm - 4.0, y, lm, y, C_MID, 1.0));
        svg.push_str(&text(lm - 6.0, y + 4.0, "end", 9.0, false, C_MID, &format!("{:.1}", v)));
    }
    svg.push_str(&text(lm - 42.0, tm + ch / 2.0, "middle", 10.0, false, C_MID, "‖r(k)‖"));

    svg.push_str(&caption(W / 2.0, H - 5.0, 4,
        "Grammar State Trajectory",
        "Residual trace coloured by grammar state; state band at bottom; ρ_max guideline"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 5 — Persistence Counter
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_05_persistence_counter() -> String {
    let traj = synthetic_trajectory();
    let n = traj.len();
    let env = AdmissibilityEnvelope::new(0.1, 1.0, 0.02);

    // Recompute persistence locally
    let mut fsm = GrammarFsm::new();
    let mut prev = 0.0_f64;
    let mut prev2 = 0.0_f64;
    let mut persistence = Vec::new();
    let mut states = Vec::new();
    for &r in &traj {
        let sign = ResidualSign::from_scalar(r, prev, prev2);
        fsm.step(&sign, &env);
        persistence.push(fsm.persistence());
        states.push(fsm.state());
        prev2 = prev;
        prev = r;
    }

    let (W, H) = (760.0_f64, 340.0_f64);
    let (lm, tm, rm, bm) = (60.0, 50.0, 20.0, 70.0);
    let cw = W - lm - rm;
    let ch = H - tm - bm;

    let max_p = *persistence.iter().max().unwrap_or(&1) as f64;
    let px = |i: usize| lm + i as f64 * cw / (n - 1) as f64;
    let py = |v: f64| tm + ch - v / max_p.max(1.0) * ch * 0.9;

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Persistence Counter K(k): Consecutive-Step Dwell in Grammar State"));
    svg.push_str(&chart_bg(lm, tm, cw, ch, 5));

    // Bars coloured by state
    let bar_w = cw / n as f64 * 0.8;
    for i in 0..n {
        let x = px(i) - bar_w / 2.0;
        let p = persistence[i] as f64;
        let col = state_color(states[i]);
        let yp = py(p);
        svg.push_str(&rect(x, yp, bar_w, tm + ch - yp, &format!("{}99", col), col, 0.5));
    }

    // x/y labels
    for i in (0..n).step_by(5) {
        svg.push_str(&text(px(i), tm + ch + 14.0, "middle", 9.0, false, C_MID, &format!("{}", i)));
    }
    for &v in &[0.0, 5.0, 10.0] {
        if v <= max_p {
            let y = py(v);
            svg.push_str(&line(lm - 4.0, y, lm, y, C_MID, 1.0));
            svg.push_str(&text(lm - 6.0, y + 4.0, "end", 9.0, false, C_MID, &format!("{:.0}", v)));
        }
    }
    svg.push_str(&text(lm - 42.0, tm + ch / 2.0, "middle", 10.0, false, C_MID, "K(k)"));

    // Legend
    for (i, (lbl, col)) in [("Adm", C_ADM), ("Bdy", C_BDY), ("Vio", C_VIO)].iter().enumerate() {
        let lx = lm + 10.0 + i as f64 * 80.0;
        let ly = tm + 12.0;
        svg.push_str(&rect(lx, ly - 9.0, 12.0, 10.0, &format!("{}99", col), col, 0.5));
        svg.push_str(&text(lx + 15.0, ly, "start", 10.0, false, C_MID, lbl));
    }

    svg.push_str(&caption(W / 2.0, H - 5.0, 5,
        "Persistence Counter",
        "K(k) = consecutive steps in current grammar state; resets on transition"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 6 — Endoductive Operator Data-Flow
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_06_endoductive_operator() -> String {
    let (W, H) = (660.0_f64, 370.0_f64);
    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 24.0, "middle", 13.0, true, C_MID,
        "Endoductive Operator ℰ: Full Data-Flow Diagram"));

    // box helper
    let bx = |x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str, lbl: &str, sublbl: &str| -> String {
        let mut s = rect(x, y, w, h, fill, stroke, 1.8);
        s.push_str(&text(x + w / 2.0, y + h / 2.0 - 4.0, "middle", 10.5, true, C_MID, lbl));
        if !sublbl.is_empty() {
            s.push_str(&text(x + w / 2.0, y + h / 2.0 + 10.0, "middle", 9.0, false, "#555", sublbl));
        }
        s
    };

    let inputs = [
        (50.0_f64, 60.0_f64, "Trajectory r", "‖r(k₀:k*)‖"),
        (50.0_f64, 130.0_f64, "Sign Sequence σ", "(‖r‖, ṙ, r̈) per step"),
        (50.0_f64, 200.0_f64, "Grammar Path g", "Adm/Bdy/Vio per step"),
        (50.0_f64, 270.0_f64, "Heuristics Bank h", "Monotone pattern store"),
    ];
    for (x, y, lbl, sub) in &inputs {
        svg.push_str(&bx(*x, *y, 160.0, 40.0, &format!("{}18", C_ACCENT), C_ACCENT, lbl, sub));
    }

    // Central ℰ box
    svg.push_str(&bx(310.0, 155.0, 100.0, 70.0, &format!("{}22", C_MID), C_MID, "ℰ", "endoductive op."));

    // Output boxes
    svg.push_str(&bx(490.0, 100.0, 150.0, 50.0, &format!("{}22", C_ADM), C_ADM, "Motif m", "Named or Unknown"));
    svg.push_str(&bx(490.0, 200.0, 150.0, 50.0, &format!("{}22", C_ACCENT), C_ACCENT, "ProvenanceTag φ", "(σ, g, α, range)"));

    // Episode wrapper
    svg.push_str(&format!(
        r#"<rect x="480" y="80" width="175" height="190" fill="none" stroke="{}" stroke-width="1.5" stroke-dasharray="6,3" rx="6"/>"#, C_MID));
    svg.push_str(&text(567.5, 76.0, "middle", 10.0, true, C_MID, "Episode (m, φ)"));

    // Arrows from inputs to ℰ
    for (y_offset, _) in [(80.0, 0), (150.0, 0), (220.0, 0), (290.0, 0)] {
        svg.push_str(&format!(
            r#"<line x1="210" y1="{}" x2="310" y2="190" stroke="{}" stroke-width="1.2" stroke-dasharray="4,3" marker-end="url(#ar2)"/>"#,
            y_offset, C_ACCENT));
    }
    // Arrows to outputs
    svg.push_str(&format!(
        r#"<line x1="410" y1="175" x2="490" y2="130" stroke="{}" stroke-width="1.5" marker-end="url(#ar2)"/>
<line x1="410" y1="195" x2="490" y2="225" stroke="{}" stroke-width="1.5" marker-end="url(#ar2)"/>"#,
        C_MID, C_MID));

    // Guarantee box
    svg.push_str(&bx(480.0, 300.0, 175.0, 40.0, "#fffff0", "#c47c00",
        "Guaranteed total", "∀ input → Episode (Corollary 5.4)"));

    svg.push_str(&format!("<defs><marker id=\"ar2\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" orient=\"auto\"><path d=\"M0,0 L6,3 L0,6 Z\" fill=\"{C_GREY}\"/></marker></defs>"));

    svg.push_str(&caption(W / 2.0, H - 18.0, 6,
        "Endoductive Operator ℰ",
        "Data-flow from trajectory to Episode; totality guaranteed by Rust return type"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 7 — ProvenanceTag Anatomy
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_07_provenance_tag_anatomy() -> String {
    let (W, H) = (660.0_f64, 380.0_f64);
    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 24.0, "middle", 13.0, true, C_MID,
        "ProvenanceTag φ: Deterministic Audit Certificate (Theorem 8.3)"));

    // Outer episode frame
    svg.push_str(&format!(
        r#"<rect x="40" y="48" width="580" height="260" rx="10" fill="none" stroke="{}" stroke-width="2"/>"#, C_MID));
    svg.push_str(&text(330.0, 44.0, "middle", 11.0, true, C_MID, "Episode (m, φ)"));

    // Left: Motif
    svg.push_str(&format!(
        r#"<rect x="55" y="65" width="140" height="225" rx="6" fill="{}22" stroke="{}" stroke-width="1.5"/>"#,
        C_ADM, C_ADM));
    svg.push_str(&text(125.0, 82.0, "middle", 10.5, true, C_ADM, "Motif m"));
    svg.push_str(&text(125.0, 102.0, "middle", 10.0, false, C_MID, "Named(\"drift_up\")"));
    svg.push_str(&text(125.0, 118.0, "middle", 9.0, false, "#555", "— or —"));
    svg.push_str(&text(125.0, 134.0, "middle", 10.0, false, C_MID, "Unknown"));
    svg.push_str(&text(125.0, 154.0, "middle", 8.5, false, "#555", "(epistemically honest;"));
    svg.push_str(&text(125.0, 167.0, "middle", 8.5, false, "#555", "provenance still complete)"));
    svg.push_str(&text(125.0, 200.0, "middle", 8.5, false, "#555", "Corollary 5.4:"));
    svg.push_str(&text(125.0, 213.0, "middle", 8.5, false, "#555", "No Silent Failure"));

    // Right: ProvenanceTag fields
    svg.push_str(&format!(
        r#"<rect x="210" y="65" width="395" height="225" rx="6" fill="{}18" stroke="{}" stroke-width="1.5"/>"#,
        C_ACCENT, C_ACCENT));
    svg.push_str(&text(408.0, 82.0, "middle", 10.5, true, C_ACCENT, "ProvenanceTag φ"));

    let fields = [
        ("sign_sequence", "σ(k₀), …, σ(k*)", "Vec<ResidualSign> — observed (‖r‖,ṙ,r̈) per step"),
        ("grammar_path",  "g(k₀), …, g(k*)", "Vec<GrammarState> — Adm/Bdy/Vio sequence"),
        ("add_descriptor","α",               "String — ADD algebraic invariant (growth, reachability)"),
        ("step_range",    "(k₀, k*)",        "Episode window indices in trajectory"),
    ];
    for (i, (field, notation, desc)) in fields.iter().enumerate() {
        let fy = 102.0 + i as f64 * 44.0;
        svg.push_str(&format!(
            "<rect x=\"218\" y=\"{}\" width=\"380\" height=\"38\" rx=\"4\" fill=\"{C_LINEN}\" stroke=\"{C_BORD}\" stroke-width=\"1\"/>", fy));
        svg.push_str(&text(228.0, fy + 14.0, "start", 10.5, true, C_MID, field));
        svg.push_str(&text(228.0, fy + 28.0, "start", 9.0, false, C_GREY,
            &format!("  {} \u{2014} {}", notation, desc)));
    }

    // Replay guarantee
    svg.push_str(&format!(
        "<rect x=\"40\" y=\"318\" width=\"580\" height=\"34\" rx=\"6\" fill=\"{C_LTGRN}\" stroke=\"{}\" stroke-width=\"1.2\"/>", C_ADM));
    svg.push_str(&text(330.0, 330.0, "middle", 9.5, true, C_ADM,
        "Theorem 8.3 — Deterministic Auditability:"));
    svg.push_str(&text(330.0, 344.0, "middle", 9.0, false, C_MID,
        "Any observer re-running ℰ with (φ, h) reproduces the identical Episode exactly."));

    svg.push_str(&caption(W / 2.0, H - 5.0, 7,
        "ProvenanceTag Anatomy",
        "Complete replayable derivation certificate — zero additional logging infrastructure required"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 8 — Bank Monotonicity (Theorem 7.2)
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_08_bank_monotonicity() -> String {
    let (W, H) = (640.0_f64, 360.0_f64);
    let (lm, tm, rm, bm) = (70.0, 50.0, 30.0, 70.0);
    let cw = W - lm - rm;
    let ch = H - tm - bm;

    // Simulate bank augmentation steps
    let aug_steps = 14_usize;
    let motif_counts: Vec<usize> = (0..aug_steps)
        .map(|i| {
            // 0 motifs up to step 2 (Day-One), then linearly grows
            if i < 2 { 0 } else { i - 1 }
        })
        .collect();
    let pattern_counts: Vec<usize> = (0..aug_steps)
        .map(|i| if i < 2 { 0 } else { (i - 1) * 2 })
        .collect();

    let max_p = *pattern_counts.iter().max().unwrap_or(&1) as f64;
    let px = |i: usize| lm + i as f64 * cw / (aug_steps - 1) as f64;
    let py = |v: f64| tm + ch - v / max_p.max(1.0) * ch * 0.88;

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Bank Monotonicity (Theorem 7.2): augment(h, m, P) — never removes"));
    svg.push_str(&chart_bg(lm, tm, cw, ch, 6));

    // Pattern count bars
    for i in 0..aug_steps {
        let bw = cw / aug_steps as f64 * 0.5;
        let x = px(i) - bw / 2.0;
        let y = py(pattern_counts[i] as f64);
        let h_bar = tm + ch - y;
        svg.push_str(&rect(x, y, bw, h_bar, &format!("{}88", C_ACCENT), C_ACCENT, 0.8));
    }

    // Motif count line
    let motif_pts: Vec<(f64, f64)> = motif_counts
        .iter()
        .enumerate()
        .map(|(i, &v)| (px(i), py(v as f64)))
        .collect();
    svg.push_str(&polyline(&motif_pts, C_ADM, 2.0, "none"));
    for (x, y) in &motif_pts {
        svg.push_str(&circle(*x, *y, 3.5, C_ADM, C_ADM, 0.0));
    }

    // Day-One annotation
    let day_x = px(1);
    svg.push_str(&line(day_x, tm, day_x, tm + ch, "#888", 1.0));
    svg.push_str(&text(day_x + 4.0, tm + 14.0, "start", 9.0, true, "#888", "Day-One"));
    svg.push_str(&text(day_x + 4.0, tm + 26.0, "start", 8.0, false, "#888", "(h = ∅, total,"));
    svg.push_str(&text(day_x + 4.0, tm + 38.0, "start", 8.0, false, "#888", "Prop. 9.1)"));

    // x/y labels
    for i in (0..aug_steps).step_by(2) {
        svg.push_str(&text(px(i), tm + ch + 14.0, "middle", 9.0, false, C_MID, &format!("{}", i)));
    }
    svg.push_str(&text(W / 2.0, tm + ch + 30.0, "middle", 9.5, false, C_MID, "augmentation step"));
    for &v in &[0.0, 5.0, 10.0, 15.0, 20.0, 24.0] {
        if v <= max_p * 1.01 {
            let y = py(v);
            svg.push_str(&line(lm - 4.0, y, lm, y, C_MID, 1.0));
            svg.push_str(&text(lm - 6.0, y + 4.0, "end", 9.0, false, C_MID, &format!("{:.0}", v)));
        }
    }
    svg.push_str(&text(lm - 52.0, tm + ch / 2.0, "middle", 10.0, false, C_MID, "count"));

    // Legend
    svg.push_str(&rect(lm + 10.0, tm + 5.0, 12.0, 10.0, &format!("{}88", C_ACCENT), C_ACCENT, 0.8));
    svg.push_str(&text(lm + 26.0, tm + 14.0, "start", 9.5, false, C_MID, "Patterns in bank"));
    svg.push_str(&circle(lm + 110.0 + 10.0, tm + 10.0, 4.0, C_ADM, C_ADM, 0.0));
    svg.push_str(&text(lm + 128.0, tm + 14.0, "start", 9.5, false, C_MID, "Named motifs"));

    svg.push_str(&caption(W / 2.0, H - 5.0, 8,
        "Bank Monotonicity",
        "augment(h, m, P) only appends — no removal; Day-One valid with h = ∅"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 9 — Observer Non-Interference
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_09_observer_noninterference() -> String {
    let (W, H) = (660.0_f64, 370.0_f64);
    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 24.0, "middle", 13.0, true, C_MID,
        "Non-Interference (SC-2): Observer is a Pure Read-Only Functor"));

    // Host system box
    svg.push_str(&format!(
        r#"<rect x="40" y="55" width="210" height="235" rx="10" fill="{}18" stroke="{}" stroke-width="2.5"/>"#,
        C_VIO, C_VIO));
    svg.push_str(&text(145.0, 75.0, "middle", 11.5, true, C_VIO, "Host System"));
    svg.push_str(&text(145.0, 95.0, "middle", 9.0, false, "#555", "(state S_t — mutable)"));

    let host_items = ["&mut estimator", "&mut buffer", "&mut history", "write path active"];
    for (i, item) in host_items.iter().enumerate() {
        svg.push_str(&format!(
            "<rect x=\"58\" y=\"{}\" width=\"178\" height=\"28\" rx=\"4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n<text x=\"68\" y=\"{}\" font-size=\"9.5\" fill=\"{}\">{}</text>",
            108 + i * 36, C_LTRED, C_VIO,
            108 + i * 36 + 17, C_VIO, item));
    }

    // Arrow: host → residual (read only)
    svg.push_str(&format!(
        r#"<line x1="250" y1="172" x2="350" y2="172" stroke="{}" stroke-width="2" marker-end="url(#ar3)"/>
<text x="300" y="162" text-anchor="middle" font-size="9" fill="{C_ADM}">‖r(k)‖</text>
<text x="300" y="188" text-anchor="middle" font-size="8" fill="{C_ADM}">shared ref. only</text>"#, C_ADM));

    // Observer box
    svg.push_str(&format!(
        r#"<rect x="350" y="55" width="270" height="235" rx="10" fill="{}18" stroke="{}" stroke-width="2.5"/>"#,
        C_ADM, C_ADM));
    svg.push_str(&text(485.0, 75.0, "middle", 11.5, true, C_ADM, "Observer 𝒪"));
    svg.push_str(&text(485.0, 95.0, "middle", 9.0, false, "#555", "(pure, no &mut to host)"));

    let obs_items = ["envelope: AdmissibilityEnvelope", "bank: HeuristicsBank", "fsm: GrammarFsm (own)", "enduce: impl Enduce"];
    for (i, item) in obs_items.iter().enumerate() {
        svg.push_str(&format!(
            "<rect x=\"362\" y=\"{}\" width=\"248\" height=\"28\" rx=\"4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n<text x=\"372\" y=\"{}\" font-size=\"9.5\" fill=\"{}\">{}</text>",
            108 + i * 36, C_LTGRN, C_ADM,
            108 + i * 36 + 17, C_ADM, item));
    }

    // No write back arrow (crossed)
    svg.push_str(&format!(
        "<line x1=\"350\" y1=\"230\" x2=\"254\" y2=\"230\" stroke=\"{C_PALE}\" stroke-width=\"1.5\" stroke-dasharray=\"5,4\"/>\n<line x1=\"310\" y1=\"218\" x2=\"294\" y2=\"242\" stroke=\"{}\" stroke-width=\"2.5\"/>\n<line x1=\"294\" y1=\"218\" x2=\"310\" y2=\"242\" stroke=\"{}\" stroke-width=\"2.5\"/>\n<text x=\"302\" y=\"260\" text-anchor=\"middle\" font-size=\"9\" fill=\"{C_VIO}\">no write-back</text>\n<text x=\"302\" y=\"273\" text-anchor=\"middle\" font-size=\"8.5\" fill=\"{C_VIO}\">structurally impossible</text>",
        C_VIO, C_VIO));

    // Output arrow → Episode
    svg.push_str(&format!(
        r#"<line x1="618" y1="172" x2="648" y2="172" stroke="{}" stroke-width="2" marker-end="url(#ar3)"/>
<text x="656" y="168" font-size="9" fill="{C_MID}">Episode</text>
<text x="656" y="180" font-size="9" fill="{C_MID}">(m, φ)</text>"#, C_MID));

    // Rust enforcement note
    svg.push_str(&format!(
        "<rect x=\"40\" y=\"300\" width=\"580\" height=\"40\" rx=\"6\" fill=\"{C_LTGRN}\" stroke=\"{}\" stroke-width=\"1.2\"/>", C_ADM));
    svg.push_str(&text(330.0, 316.0, "middle", 9.5, true, C_ADM,
        "Rust ownership enforcement (not a policy constraint — a structural impossibility):"));
    svg.push_str(&text(330.0, 330.0, "middle", 9.0, false, C_MID,
        "Observer takes residuals by value (f64 Copy). No &mut path to host exists. Borrow checker verifies."));

    svg.push_str(&format!("<defs><marker id=\"ar3\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" orient=\"auto\"><path d=\"M0,0 L6,3 L0,6 Z\" fill=\"{C_GREY}\"/></marker></defs>"));

    svg.push_str(&caption(W / 2.0, H - 5.0, 9,
        "Observer Non-Interference",
        "SC-2: Observer holds no &mut to the host system; Rust borrow checker enforces structurally"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// FIGURE 10 — Cross-Stream Grammar Fusion G₁ ⋈ G₂
// ═══════════════════════════════════════════════════════════════════════════

pub fn figure_10_cross_stream_fusion() -> String {
    // Separate perturbed trajectory for stream 2
    let traj1 = synthetic_trajectory();
    let traj2: Vec<f64> = traj1
        .iter()
        .enumerate()
        .map(|(i, &v)| (v * 0.8 + 0.05 + (i % 5) as f64 * 0.04).min(1.35))
        .collect();

    let env1 = AdmissibilityEnvelope::new(0.1, 1.0, 0.02);
    let env2 = AdmissibilityEnvelope::new(0.1, 0.88, 0.02);

    let (_, states1) = compute_grammar_trace(&traj1, &env1);
    let (_, states2) = compute_grammar_trace(&traj2, &env2);

    let n = traj1.len();
    let (W, H) = (760.0_f64, 400.0_f64);
    let (lm, tm, rm, bm) = (60.0, 50.0, 20.0, 80.0);
    let cw = W - lm - rm;
    let ch = (H - tm - bm - 20.0) / 2.0;

    let px = |i: usize| lm + i as f64 * cw / (n - 1) as f64;
    let py = |v: f64, top: f64| top + ch - v / 1.5 * ch * 0.88;

    let stream_panel = |traj: &[f64], states: &[GrammarState], top: f64, label: &str| -> String {
        let mut s = chart_bg(lm, top, cw, ch, 4);
        s.push_str(&text(lm + 4.0, top + 13.0, "start", 10.0, true, C_MID, label));
        for i in 0..traj.len() {
            if i > 0 {
                let x0 = px(i - 1); let x1 = px(i);
                let y0 = py(traj[i - 1], top); let y1 = py(traj[i], top);
                s.push_str(&line(x0, y0, x1, y1, state_color(states[i]), 2.0));
            }
            s.push_str(&circle(px(i), py(traj[i], top), 2.5, state_color(states[i]), state_color(states[i]), 0.0));
        }
        s
    };

    let mut svg = svg_open(W, H);
    svg.push_str(&rect(0.0, 0.0, W, H, C_BG, "none", 0.0));
    svg.push_str(&text(W / 2.0, 22.0, "middle", 13.0, true, C_MID,
        "Cross-Stream Grammar Fusion G₁ ⋈ G₂ (Definition 7.1)"));

    svg.push_str(&stream_panel(&traj1, &states1, tm, "Stream 1 — env: ρ_max = 1.0"));
    svg.push_str(&stream_panel(&traj2, &states2, tm + ch + 20.0, "Stream 2 — env: ρ_max = 0.88"));

    // Joint violation markers
    for i in 0..n {
        if states1[i] == GrammarState::Violation && states2[i] == GrammarState::Violation {
            let x = px(i);
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1.5" stroke-dasharray="3,2"/>
<text x="{}" y="{}" text-anchor="middle" font-size="8" fill="{C_VIO}">⋈ Vio</text>"#,
                x, tm, x, tm + 2.0 * ch + 20.0, C_VIO,
                x, tm + 2.0 * ch + 36.0));
        }
    }

    // Legend
    for (i, (lbl, col)) in [("Adm", C_ADM), ("Bdy", C_BDY), ("Vio", C_VIO), ("⋈ joint Vio", C_VIO)].iter().enumerate() {
        let lx = lm + i as f64 * 130.0;
        let ly = H - bm + 14.0;
        if i < 3 {
            svg.push_str(&circle(lx + 6.0, ly - 4.0, 5.0, col, col, 0.0));
        } else {
            svg.push_str(&line(lx, ly - 4.0, lx + 12.0, ly - 4.0, col, 1.5));
        }
        svg.push_str(&text(lx + 14.0, ly, "start", 9.5, false, C_MID, lbl));
    }

    // x-axis
    for i in (0..n).step_by(5) {
        svg.push_str(&text(px(i), H - bm + 32.0, "middle", 9.0, false, C_MID, &format!("{}", i)));
    }
    svg.push_str(&text(W / 2.0, H - 5.0 - 14.0, "middle", 9.0, false, "#555",
        "Determinism of product guaranteed (Proposition 7.1): independent per-component transitions"));
    svg.push_str(&caption(W / 2.0, H - 5.0, 10,
        "Cross-Stream Grammar Fusion",
        "G₁ ⋈ G₂: independent per-component transitions; joint Vio detection across streams"));
    svg.push_str(svg_close());
    svg
}

// ═══════════════════════════════════════════════════════════════════════════
// Catalogue struct
// ═══════════════════════════════════════════════════════════════════════════

/// Ordered collection of all ten canonical DSSC figures, each as `(filename, svg_content)`.
pub struct FigureCatalogue {
    /// The figure entries: `(filename, SVG string)` pairs, in figure order 1–10.
    pub items: Vec<(&'static str, String)>,
}

impl FigureCatalogue {
    /// Generate all ten canonical figures.
    pub fn generate() -> Self {
        Self {
            items: vec![
                ("fig01_residual_sign_triple.svg",     figure_01_residual_sign_triple()),
                ("fig02_admissibility_envelope.svg",   figure_02_admissibility_envelope()),
                ("fig03_grammar_fsm_diagram.svg",      figure_03_grammar_fsm_diagram()),
                ("fig04_grammar_state_trajectory.svg", figure_04_grammar_state_trajectory()),
                ("fig05_persistence_counter.svg",      figure_05_persistence_counter()),
                ("fig06_endoductive_operator.svg",     figure_06_endoductive_operator()),
                ("fig07_provenance_tag_anatomy.svg",   figure_07_provenance_tag_anatomy()),
                ("fig08_bank_monotonicity.svg",        figure_08_bank_monotonicity()),
                ("fig09_observer_noninterference.svg", figure_09_observer_noninterference()),
                ("fig10_cross_stream_fusion.svg",      figure_10_cross_stream_fusion()),
            ],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// JSON summary
// ═══════════════════════════════════════════════════════════════════════════

/// Generate a JSON summary of the run for auditability.
pub fn run_summary_json(out_dir: &Path, figure_paths: &[PathBuf]) -> serde_json::Value {
    let traj = synthetic_trajectory();
    let env = AdmissibilityEnvelope::new(0.1, 1.0, 0.02);
    let (signs, states) = compute_grammar_trace(&traj, &env);

    let n_adm = states.iter().filter(|&&s| s == GrammarState::Admissible).count();
    let n_bdy = states.iter().filter(|&&s| s == GrammarState::Boundary).count();
    let n_vio = states.iter().filter(|&&s| s == GrammarState::Violation).count();
    let max_mag = signs.iter().map(|s| s.magnitude).fold(0.0_f64, f64::max);
    let max_drift = signs.iter().map(|s| s.drift.abs()).fold(0.0_f64, f64::max);

    serde_json::json!({
        "crate": "dsfb-semiotics-calculus",
        "version": env!("CARGO_PKG_VERSION"),
        "generated_utc": chrono_stub(),
        "trajectory_length": traj.len(),
        "envelope": { "rho_min": 0.1, "rho_max": 1.0, "delta": 0.02 },
        "grammar_state_distribution": {
            "Admissible": n_adm,
            "Boundary": n_bdy,
            "Violation": n_vio
        },
        "sign_statistics": {
            "max_magnitude": max_mag,
            "max_abs_drift": max_drift,
        },
        "figures_generated": figure_paths.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy()).collect::<Vec<_>>(),
        "output_dir": out_dir.display().to_string(),
        "safety_case_properties": [
            "SC-1 Determinism: enforced by Enduce::enduce return type",
            "SC-2 Non-Interference: Observer holds no &mut to host; Rust borrow checker",
            "SC-3 Auditability: every Episode carries ProvenanceTag",
            "SC-4 Coverage: GrammarFsm::step is total",
            "SC-5 No Silent Failure: Motif::Unknown with full provenance, never None",
            "SC-6 Graceful Degradation: impulsive inputs produce Unknown, not panics"
        ]
    })
}

fn chrono_stub() -> String {
    // Deterministic stub without chrono dependency
    "2026-04-06T00:00:00Z (build time; use chrono for runtime timestamp)".to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// Write all artifacts to disk + ZIP
// ═══════════════════════════════════════════════════════════════════════════

/// Write all ten SVG figures, the JSON summary, and a zip bundle.
/// Returns the list of file paths written.
pub fn write_all_artifacts(out_dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)?;

    let catalogue = FigureCatalogue::generate();
    let mut written: Vec<PathBuf> = Vec::new();

    // Write SVGs
    for (name, svg) in &catalogue.items {
        let path = out_dir.join(name);
        fs::write(&path, svg.as_bytes())?;
        written.push(path);
    }

    // Write JSON summary
    let summary = run_summary_json(out_dir, &written);
    let summary_path = out_dir.join("summary.json");
    fs::write(&summary_path, serde_json::to_string_pretty(&summary).unwrap())?;
    written.push(summary_path.clone());

    // Write human-readable report
    let report = build_report(&summary);
    let report_path = out_dir.join("report.md");
    fs::write(&report_path, report.as_bytes())?;
    written.push(report_path.clone());

    // Build ZIP
    let zip_path = out_dir.join("dsfb-semiotics-calculus-artifacts.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let opts = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for path in &written {
        if path == &zip_path { continue; }
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        zip.start_file(name, opts)?;
        let bytes = fs::read(path)?;
        use std::io::Write as IoWrite;
        zip.write_all(&bytes)?;
    }
    zip.finish()?;
    written.push(zip_path);

    Ok(written)
}

fn build_report(summary: &serde_json::Value) -> String {
    let traj_len = summary["trajectory_length"].as_u64().unwrap_or(0);
    let adm = summary["grammar_state_distribution"]["Admissible"].as_u64().unwrap_or(0);
    let bdy = summary["grammar_state_distribution"]["Boundary"].as_u64().unwrap_or(0);
    let vio = summary["grammar_state_distribution"]["Violation"].as_u64().unwrap_or(0);

    format!(r#"# DSFB Structural Semiotics Calculus — Artifact Report

**Crate:** `dsfb-semiotics-calculus` v{}  
**Framework:** DSFB Structural Semiotics Calculus (DSSC)  
**Invariant Forge LLC** — April 2026

---

## Synthetic Trajectory Statistics

| Metric | Value |
|--------|-------|
| Trajectory length | {} steps |
| Admissible steps | {} ({:.1}%) |
| Boundary steps | {} ({:.1}%) |
| Violation steps | {} ({:.1}%) |

Envelope configuration: ρ_min = 0.1, ρ_max = 1.0, δ = 0.02 (satisfies δ ≤ ρ_min/4).

---

## Safety-Case Properties (Rust-enforced)

| Property | Enforcement mechanism |
|----------|----------------------|
| SC-1 Determinism | `Enduce::enduce` returns `Episode`, never `Option` |
| SC-2 Non-Interference | `Observer` holds no `&mut` to observed system (borrow checker) |
| SC-3 Auditability | Every `Episode` carries full `ProvenanceTag` |
| SC-4 Coverage | `GrammarFsm::step` is total over all inputs |
| SC-5 No Silent Failure | `Motif::Unknown` with provenance — never `None`, never silent |
| SC-6 Graceful Degradation | Impulsive inputs yield `Unknown` episodes, not panics |

---

## Figures Generated

{}

---

## IP Notice

Apache 2.0 applies to this software artifact. The underlying theoretical framework
constitutes proprietary Background IP of Invariant Forge LLC (Delaware LLC No. 10529072).
Commercial deployment requires a written license. Inquiries: licensing@invariantforge.net
"#,
        env!("CARGO_PKG_VERSION"),
        traj_len,
        adm, adm as f64 / traj_len as f64 * 100.0,
        bdy, bdy as f64 / traj_len as f64 * 100.0,
        vio, vio as f64 / traj_len as f64 * 100.0,
        (1..=10).map(|i| format!("- Figure {}", i)).collect::<Vec<_>>().join("\n"),
    )
}
