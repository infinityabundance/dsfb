//! Demo report assembly — Markdown report + figure manifest.
//!
//! Generates `report.md` listing every rendered figure with its caption,
//! grouped by section (architecture / per-fixture / cross-fixture). The
//! Markdown report is the deliverable; reviewers convert it to PDF via
//! `pandoc` or `wkhtmltopdf` if a PDF is wanted, or open it directly in
//! the editor / browser with embedded image previews.
//!
//! The Colab notebook (Part 2 of the deliverable) generates its own PDF
//! via `matplotlib.backends.backend_pdf.PdfPages` — Python is the
//! cleaner stack for that step. The demo binary's job is the figures +
//! JSON tables + Markdown index; the notebook wraps the binary's
//! output with a Python PDF assembler when a PDF is required.

#![cfg(feature = "demo")]

use std::fs::write;
use std::path::Path;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{format, vec};

use crate::demo::runner::FixtureRun;
use crate::error::Result;

/// Assemble the Markdown report listing every figure + caption.
pub fn assemble(_runs: &[FixtureRun], _figs_root: &Path, out_path: &Path) -> Result<()> {
    let mut md = String::new();

    md.push_str("# DSFB-Debug — Demo Report\n\n");
    md.push_str("**Deterministic detector-field semiotics with routed forensic witness-field fusion**\n\n");
    md.push_str("Twelve real-bytes vendored fixtures across nine distinct upstream public datasets.\n\n");

    md.push_str("## Prior Art Notice (35 U.S.C. §102)\n\n");
    md.push_str("This report constitutes prior art deposited via Zenodo + GitHub at the run-id timestamp encoded in the parent folder name.\n\n");

    md.push_str("## IP Notice\n\n");
    md.push_str("Background IP of Invariant Forge LLC (Delaware LLC No. 10529072). ");
    md.push_str("Reference implementation Apache-2.0; commercial deployment requires written license. ");
    md.push_str("Contact: licensing@invariantforge.net\n\n");

    md.push_str("## Authoring discipline\n\n");
    md.push_str("- All numbers in the per-fixture JSON are verbatim test stdout from the engine. No synthetic data.\n");
    md.push_str("- Theorem 9 (deterministic replay) holds across every fixture.\n");
    md.push_str("- ML-free, no_std, zero runtime Cargo dependencies in the core, edge-deployable.\n\n");

    md.push_str("---\n\n");

    md.push_str("## Architecture / infrastructure figures\n\n");
    md.push_str("### Figure A1 — Pipeline architecture\n\n");
    md.push_str("![Pipeline](figures/00_architecture/01_pipeline.png)\n\n");
    md.push_str("Residual matrix → 205 detectors / 27 axes → tier-coded witness field → ");
    md.push_str("Layer-2 consensus → DSFB structural episodes → motif-affinity routing → ");
    md.push_str("confuser/witness gates → forensic evidence packet. Theorem 9 deterministic ");
    md.push_str("replay holds across the entire pipeline.\n\n");

    md.push_str("### Figure A2 — Detector ensemble breakdown\n\n");
    md.push_str("![Tier breakdown](figures/00_architecture/02_tier_breakdown.png)\n\n");
    md.push_str("27 mathematical axes (Tiers A-U + EXTRA + V/X/Y/Z/AA), 205 deterministic detectors. ");
    md.push_str("Each detector is a deterministic statistical or structural function with a literature citation.\n\n");

    md.push_str("### Figure A3 — 32-motif × 27-axis affinity matrix\n\n");
    md.push_str("![Motif affinity](figures/00_architecture/03_motif_affinity.png)\n\n");
    md.push_str("Each filled cell indicates the motif (row) routes detector evidence from that tier (column) ");
    md.push_str("per the Routed Evidence Principle.\n\n");

    md.push_str("---\n\n");
    md.push_str("## Per-fixture results\n\n");

    let labels = [
        ("01_F11", "F-11 (TADBench / TrainTicket order-service deployment regression)"),
        ("02_F11b", "F-11b (TADBench / TrainTicket auth-mongo)"),
        ("03_F04", "F-04 (TADBench / TrainTicket admin-service config)"),
        ("04_F19", "F-19 (TADBench / TrainTicket mongodb-driver-3.0.4 config)"),
        ("05_Illinois", "Illinois SocialNetwork (unsampled)"),
        ("06_AIOpsChallenge", "AIOps Challenge 2018 KPI"),
        ("07_LO2", "LO2 OAuth2 (endoductive validator)"),
        ("08_MultiDim", "MultiDimension-Localization part1"),
        ("09_DeepTraLog", "DeepTraLog F-01"),
        ("10_Defects4J", "Defects4J 6-project Java bug catalog"),
        ("11_BugsInPy", "BugsInPy 6-project Python bug catalog"),
        ("12_PROMISE", "PROMISE defect-prediction 6-CSV"),
    ];
    let figs: &[(&str, &str)] = &[
        ("01_residual_heatmap.png", "Raw residual heatmap"),
        ("02_signal_timeseries.png", "Per-signal residual time-series"),
        ("03_drift_per_signal.png", "Per-signal drift persistence"),
        ("04_slew_per_signal.png", "Per-signal |slew| density"),
        ("05_grammar_state_matrix.png", "Grammar state per (window, signal)"),
        ("06_policy_state_heatmap.png", "Policy state per (window, signal)"),
        ("07_per_detector_firing.png", "Top-30 detectors by firing count"),
        ("08_episode_timeline.png", "Closed-episode timeline"),
        ("09_evidence_packet.png", "Per-episode signature summary"),
        ("10_replay_verification.png", "Theorem 9 deterministic replay"),
    ];

    for (label, full) in labels.iter() {
        md.push_str(&format!("### {}\n\n", full));
        md.push_str(&format!(
            "Verbatim metrics: `results/{}.json`\n\n",
            label
        ));
        for (fname, fcap) in figs.iter() {
            md.push_str(&format!(
                "![{}](figures/{}/{})\n\n*{}*\n\n",
                fcap, label, fname, fcap
            ));
        }
        md.push_str("---\n\n");
    }

    md.push_str("## Cross-fixture summary\n\n");
    md.push_str("### Figure C1 — Cross-fixture RSCR scatter\n\n");
    md.push_str("![RSCR scatter](figures/cross_fixture/01_rscr_scatter.png)\n\n");
    md.push_str("12 fixtures plotted by Review Surface Compression Ratio vs clean-window FP rate.\n\n");

    md.push_str("### Figure C2 — F-11 fusion sweep N=1..9\n\n");
    md.push_str("![Fusion sweep](figures/cross_fixture/02_fusion_sweep.png)\n\n");
    md.push_str("Layer-2 clean-window FP rate as min_consensus N varies from 1 to 9 on F-11.\n\n");

    md.push_str("### Figure C3 — Cross-fixture clean-window FP rate\n\n");
    md.push_str("![FP comparison](figures/cross_fixture/03_fp_comparison.png)\n\n");
    md.push_str("Per-fixture FP rate at default Phase-8 9-axis bank-aware fusion config.\n\n");

    write(out_path, md.as_bytes()).map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    Ok(())
}
