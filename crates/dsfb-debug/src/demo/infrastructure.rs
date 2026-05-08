//! Architecture / infrastructure figures (3 figures rendered once at the
//! top of the demo run, before any per-fixture cells).
//!
//! 1. `01_pipeline.png` — pipeline flowchart (residual matrix → 205
//!    detectors → consensus → DSFB structural episodes → motif-affinity
//!    routing → witness gates → forensic evidence packet).
//! 2. `02_tier_breakdown.png` — per-tier detector count bar chart
//!    (27 tiers).
//! 3. `03_motif_affinity.png` — 32-motif × 27-tier affinity bitmap
//!    heatmap (the IP claim, visualised).

#![cfg(feature = "demo")]

use std::path::Path;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{format, vec};

use plotters::prelude::*;

use crate::error::Result;
use crate::heuristics_bank::{
    HeuristicsBank,
    TIER_BIT_A, TIER_BIT_AA, TIER_BIT_B, TIER_BIT_C, TIER_BIT_D, TIER_BIT_E,
    TIER_BIT_EXTRA, TIER_BIT_F, TIER_BIT_G, TIER_BIT_H, TIER_BIT_I, TIER_BIT_J,
    TIER_BIT_K, TIER_BIT_L, TIER_BIT_M, TIER_BIT_N, TIER_BIT_O, TIER_BIT_P,
    TIER_BIT_Q, TIER_BIT_R, TIER_BIT_S, TIER_BIT_T, TIER_BIT_U, TIER_BIT_V,
    TIER_BIT_W, TIER_BIT_X, TIER_BIT_Y, TIER_BIT_Z,
};

const FIG_W: u32 = 1280;
const FIG_H: u32 = 800;

/// Per-tier detector counts (matches `incumbent_baselines.rs` tier budgets
/// per the post-Phase-8 panel-recorded inventory).
const TIER_LABELS: &[(&str, u32)] = &[
    ("A", 3), ("B", 3), ("C", 5), ("D", 5), ("E", 3), ("F", 4),
    ("EXTRA", 5),
    ("G", 9), ("H", 10), ("I", 10), ("J", 10), ("K", 10), ("L", 9),
    ("M", 18), ("N", 8), ("O", 10), ("P", 9), ("Q", 10), ("R", 8),
    ("S", 3), ("T", 6), ("U", 8),
    ("V", 8), ("X", 8), ("Y", 8), ("Z", 8), ("AA", 11),
];

/// Render the architecture pipeline figure.
pub fn render_pipeline_figure(path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(60);
    top.titled(
        "DSFB-Debug — deterministic detector-field semiotics with routed forensic witness-field fusion",
        ("sans-serif", 22),
    ).ok();

    // Hand-laid box positions (8 stages in a flow).
    let stages = [
        ("Residual\nmatrix",                40,  280),
        ("205 detectors\n× 27 axes",        220, 280),
        ("Tier-coded\nwitness field",       400, 280),
        ("Layer-2\nconsensus",              580, 280),
        ("DSFB structural\nepisodes",       580, 480),
        ("Motif affinity\nrouting (32)",    400, 480),
        ("Confuser /\nwitness gates",       220, 480),
        ("Operator\nevidence packet",        40, 480),
    ];
    let box_w = 140i32;
    let box_h = 70i32;

    for (label, x, y) in stages.iter() {
        bottom.draw(&Rectangle::new(
            [(*x as i32, *y as i32), (*x as i32 + box_w, *y as i32 + box_h)],
            ShapeStyle {
                color: RGBColor(220, 235, 255).into(),
                filled: true,
                stroke_width: 2,
            },
        )).ok();
        bottom.draw(&Rectangle::new(
            [(*x as i32, *y as i32), (*x as i32 + box_w, *y as i32 + box_h)],
            BLACK.stroke_width(2),
        )).ok();
        for (i, line) in label.split('\n').enumerate() {
            bottom.draw(&Text::new(
                line.to_string(),
                (*x as i32 + 10, *y as i32 + 20 + (i as i32) * 22),
                ("sans-serif", 14),
            )).ok();
        }
    }

    // Arrows between consecutive stages (top row left→right; vertical
    // step; bottom row right→left to form a U).
    let arrows = [
        // top row →
        ((40 + box_w, 280 + box_h / 2),  (220,                280 + box_h / 2)),
        ((220 + box_w, 280 + box_h / 2), (400,                280 + box_h / 2)),
        ((400 + box_w, 280 + box_h / 2), (580,                280 + box_h / 2)),
        // top→bottom (Layer-2 → episodes)
        ((580 + box_w / 2, 280 + box_h), (580 + box_w / 2,    480)),
        // bottom row ←
        ((580,            480 + box_h / 2), (400 + box_w,    480 + box_h / 2)),
        ((400,            480 + box_h / 2), (220 + box_w,    480 + box_h / 2)),
        ((220,            480 + box_h / 2), ( 40 + box_w,    480 + box_h / 2)),
    ];
    for (start, end) in arrows.iter() {
        bottom.draw(&PathElement::new(
            std::vec![*start, *end],
            BLACK.stroke_width(2),
        )).ok();
        // Arrow head — small triangle at end.
        let (ex, ey) = *end;
        let (sx, sy) = *start;
        let dx = ex as f64 - sx as f64;
        let dy = ey as f64 - sy as f64;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let nx = -uy;
        let ny = ux;
        let head: Vec<(i32, i32)> = std::vec![
            (ex, ey),
            ((ex as f64 - 12.0 * ux + 6.0 * nx) as i32, (ey as f64 - 12.0 * uy + 6.0 * ny) as i32),
            ((ex as f64 - 12.0 * ux - 6.0 * nx) as i32, (ey as f64 - 12.0 * uy - 6.0 * ny) as i32),
            (ex, ey),
        ];
        bottom.draw(&Polygon::new(head, BLACK.filled())).ok();
    }

    // Side annotations.
    let annotations = [
        ("Theorem 9 holds across the entire pipeline", 380, 100),
        ("9 fusion axes (Phase 0-8 anti-hallucination ladder)", 380, 130),
        ("32 motifs anchored to IEEE 24765 / Avizienis-Laprie-Randell", 380, 160),
        ("ML-free, no_std, zero runtime deps, edge-deployable", 380, 190),
    ];
    for (msg, x, y) in annotations.iter() {
        bottom.draw(&Text::new(
            msg.to_string(),
            (*x, *y),
            ("sans-serif", 14),
        )).ok();
    }
    root.present().ok();
    Ok(())
}

/// Render per-tier detector count bar chart (27 tiers).
pub fn render_tier_breakdown(path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(48);
    top.titled(
        "Detector ensemble — 205 detectors organised across 27 mathematical axes",
        ("sans-serif", 22),
    ).ok();

    let max = TIER_LABELS.iter().map(|(_, n)| *n).max().unwrap_or(20) as f64;
    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0..(TIER_LABELS.len() as i32 + 1), 0.0_f64..(max * 1.15))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_label_formatter(&|x| {
            let i = *x as usize;
            if i > 0 && i <= TIER_LABELS.len() {
                TIER_LABELS[i - 1].0.to_string()
            } else {
                String::new()
            }
        })
        .x_desc("Tier")
        .y_desc("Detector count")
        .draw().ok();

    chart.draw_series(
        TIER_LABELS.iter().enumerate().map(|(i, (_, n))| {
            Rectangle::new(
                [(i as i32 + 1, 0.0), (i as i32 + 2, *n as f64)],
                Palette99::pick(i).filled(),
            )
        })
    ).ok();
    chart.draw_series(
        TIER_LABELS.iter().enumerate().map(|(i, (_, n))| {
            Text::new(format!("{}", n), (i as i32 + 1, *n as f64 + max * 0.02), ("sans-serif", 10))
        })
    ).ok();
    root.present().ok();
    Ok(())
}

/// Render 32-motif × 27-tier affinity bitmap heatmap.
pub fn render_motif_affinity_heatmap(path: &Path) -> Result<()> {
    let backend = BitMapBackend::new(path, (FIG_W, FIG_H));
    let root = backend.into_drawing_area();
    root.fill(&WHITE).ok();
    let (top, bottom) = root.split_vertically(48);
    top.titled(
        "32-motif × 27-axis affinity matrix (Routed Evidence Principle visualised)",
        ("sans-serif", 22),
    ).ok();

    let bank: HeuristicsBank<64> = HeuristicsBank::with_canonical_motifs();
    let entries = bank.entries_iter().collect::<Vec<_>>();
    let n_motifs = entries.len();
    let tier_bits: &[(&str, u32)] = &[
        ("A", TIER_BIT_A), ("B", TIER_BIT_B), ("C", TIER_BIT_C), ("D", TIER_BIT_D),
        ("E", TIER_BIT_E), ("F", TIER_BIT_F), ("EXTRA", TIER_BIT_EXTRA),
        ("G", TIER_BIT_G), ("H", TIER_BIT_H), ("I", TIER_BIT_I), ("J", TIER_BIT_J),
        ("K", TIER_BIT_K), ("L", TIER_BIT_L), ("M", TIER_BIT_M), ("N", TIER_BIT_N),
        ("O", TIER_BIT_O), ("P", TIER_BIT_P), ("Q", TIER_BIT_Q), ("R", TIER_BIT_R),
        ("S", TIER_BIT_S), ("T", TIER_BIT_T), ("U", TIER_BIT_U),
        ("V", TIER_BIT_V), ("W", TIER_BIT_W), ("X", TIER_BIT_X), ("Y", TIER_BIT_Y),
        ("Z", TIER_BIT_Z), ("AA", TIER_BIT_AA),
    ];

    let mut chart = ChartBuilder::on(&bottom)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(180)
        .build_cartesian_2d(0..(tier_bits.len() as i32), 0..(n_motifs as i32))
        .map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    chart.configure_mesh()
        .x_label_formatter(&|x| {
            let i = *x as usize;
            if i < tier_bits.len() { tier_bits[i].0.to_string() } else { String::new() }
        })
        .y_label_formatter(&|y| {
            let i = *y as usize;
            if i < n_motifs {
                let s = format!("{:?}", entries[i].motif_class);
                if s.len() > 24 { s[..24].to_string() } else { s }
            } else { String::new() }
        })
        .x_label_style(("sans-serif", 9))
        .y_label_style(("sans-serif", 9))
        .x_desc("Detector tier")
        .y_desc("Motif")
        .draw().ok();

    for (mi, entry) in entries.iter().enumerate() {
        for (ti, (_, bit)) in tier_bits.iter().enumerate() {
            let on = (entry.affinity_tiers & bit) != 0;
            let color = if on { RGBColor(20, 20, 60) } else { RGBColor(245, 245, 245) };
            chart.draw_series(std::iter::once(
                Rectangle::new([(ti as i32, mi as i32), (ti as i32 + 1, mi as i32 + 1)], color.filled())
            )).ok();
        }
    }
    root.present().ok();
    Ok(())
}
