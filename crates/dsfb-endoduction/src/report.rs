//! PDF report assembly and ZIP archiving.

use crate::types::{RunManifest, WindowMetrics};
use anyhow::{Context, Result};
use printpdf::*;
use std::fs;
use std::io::Write;
use std::path::Path;

const PAGE_W: f32 = 210.0; // A4 mm
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 20.0;
const LINE_H: f32 = 5.0;

/// Generate the PDF report.
pub fn generate_pdf(
    manifest: &RunManifest,
    _metrics: &[WindowMetrics],
    figure_files: &[String],
    out_dir: &Path,
) -> Result<String> {
    let fname = "report.pdf";
    let path = out_dir.join(fname);

    let (doc, page1, layer1) = PdfDocument::new(
        "DSFB Endoduction — NASA IMS Bearing Analysis Report",
        Mm(PAGE_W),
        Mm(PAGE_H),
        "Title",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = PAGE_H - MARGIN;

    // Title
    write_line(&layer, &font_bold, 16.0, MARGIN, y, "DSFB Endoduction: NASA IMS Bearing Analysis Report");
    y -= LINE_H * 3.0;

    write_line(&layer, &font, 10.0, MARGIN, y, &format!("Timestamp: {}", manifest.timestamp));
    y -= LINE_H * 1.5;
    write_line(&layer, &font, 10.0, MARGIN, y, &format!("Crate version: {}", manifest.crate_version));
    y -= LINE_H * 1.5;
    if let Some(ref rev) = manifest.git_revision {
        write_line(&layer, &font, 10.0, MARGIN, y, &format!("Git revision: {rev}"));
        y -= LINE_H * 1.5;
    }
    write_line(&layer, &font, 10.0, MARGIN, y, &format!("Dataset: {}", manifest.dataset_source));
    y -= LINE_H * 2.0;

    // Configuration
    write_line(&layer, &font_bold, 12.0, MARGIN, y, "Configuration");
    y -= LINE_H * 1.5;
    let cfg = &manifest.config;
    for line in [
        format!("Bearing set: {}", cfg.bearing_set),
        format!("Primary channel: {}", cfg.primary_channel),
        format!("Window size: {}", cfg.window_size),
        format!("Nominal fraction: {:.2}", cfg.nominal_fraction),
        format!("Envelope quantile: {:.4}", cfg.envelope_quantile),
        format!("Sustained count: {}", cfg.sustained_count),
        format!("Trust threshold: {:.2}", cfg.trust_threshold),
        format!("Seed: {}", cfg.seed),
    ] {
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y, &line);
        y -= LINE_H;
    }
    y -= LINE_H;

    // Methodology
    write_line(&layer, &font_bold, 12.0, MARGIN, y, "Methodology");
    y -= LINE_H * 1.5;
    for line in [
        "1. Parse NASA IMS bearing run-to-failure data.",
        "2. Estimate nominal baseline from early-life windows.",
        "3. Compute residual r(t) = x_obs(t) - x_model(t) for each window.",
        "4. Estimate admissibility envelope from nominal regime.",
        "5. Compute structural motifs: drift, persistence, variance growth, autocorrelation.",
        "6. Aggregate into a bounded trust/precursor score.",
        "7. Compare against classical scalar diagnostics (RMS, kurtosis, crest factor, etc.).",
        "8. Evaluate lead time relative to failure reference.",
    ] {
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y, line);
        y -= LINE_H;
    }
    y -= LINE_H;

    // Key formulas
    write_line(&layer, &font_bold, 12.0, MARGIN, y, "Key Formulas");
    y -= LINE_H * 1.5;
    for line in [
        "Residual: r(t) = x_obs(t) - x_model(t)",
        "Admissibility: E_R = { r : |r_i| <= k * sigma_i  for all i }",
        "Breach fraction: B = |{i : r_i not in E_R}| / N",
        "Trust score: weighted sigmoid aggregate of structural indicators",
    ] {
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y, line);
        y -= LINE_H;
    }
    y -= LINE_H;

    // Summary metrics
    write_line(&layer, &font_bold, 12.0, MARGIN, y, "Summary Results");
    y -= LINE_H * 1.5;
    let summary = &manifest.summary;
    write_line(&layer, &font, 9.0, MARGIN + 5.0, y,
        &format!("Total windows: {}", summary.total_windows));
    y -= LINE_H;
    write_line(&layer, &font, 9.0, MARGIN + 5.0, y,
        &format!("Nominal end: window {}", summary.nominal_end_window));
    y -= LINE_H;
    write_line(&layer, &font, 9.0, MARGIN + 5.0, y,
        &format!("Failure reference: window {}", summary.failure_window));
    y -= LINE_H;
    if let Some(lead) = summary.dsfb_lead_time_windows {
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y,
            &format!("DSFB lead time: {} windows before failure", lead));
    } else {
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y,
            "DSFB lead time: not detected");
    }
    y -= LINE_H;

    // Baseline comparison
    y -= LINE_H;
    write_line(&layer, &font_bold, 12.0, MARGIN, y, "Baseline Comparison");
    y -= LINE_H * 1.5;
    let mut baseline_entries: Vec<_> = summary.baseline_lead_times.iter().collect();
    baseline_entries.sort_by_key(|(k, _)| (*k).clone());
    for (name, lead) in &baseline_entries {
        let det = summary.baseline_first_detections.get(*name).and_then(|d| *d);
        let line = match (det, lead) {
            (Some(d), Some(l)) => format!("{name}: first detection at window {d}, lead time {l} windows"),
            _ => format!("{name}: not detected"),
        };
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y, &line);
        y -= LINE_H;
    }

    // Figures list
    y -= LINE_H;
    if y > MARGIN + LINE_H * 3.0 {
        write_line(&layer, &font_bold, 12.0, MARGIN, y, "Figures Produced");
        y -= LINE_H * 1.5;
        for f in figure_files {
            if y < MARGIN + LINE_H {
                break;
            }
            write_line(&layer, &font, 9.0, MARGIN + 5.0, y, f);
            y -= LINE_H;
        }
    }

    // Limitations
    y -= LINE_H;
    if y > MARGIN + LINE_H * 8.0 {
        write_line(&layer, &font_bold, 12.0, MARGIN, y, "Limitations");
        y -= LINE_H * 1.5;
        for line in [
            "- This analysis evaluates a single bearing dataset and does not establish universality.",
            "- The trust score is an engineered proxy, not a thermodynamic measurement.",
            "- Spectral analysis uses a simplified DFT, not a full periodogram.",
            "- Results are consistent with the paper's hypothesis but do not prove it.",
            "- Lead-time estimates depend on threshold and sustained-count parameters.",
        ] {
            write_line(&layer, &font, 9.0, MARGIN + 5.0, y, line);
            y -= LINE_H;
        }
    }

    // Gates
    y -= LINE_H;
    if y > MARGIN + LINE_H * 3.0 {
        write_line(&layer, &font_bold, 12.0, MARGIN, y, "Acceptance Gates");
        y -= LINE_H * 1.5;
        let g = &manifest.gates;
        let status = if g.all_passed() { "ALL PASSED" } else { "SOME FAILED" };
        write_line(&layer, &font, 9.0, MARGIN + 5.0, y, &format!("Status: {status}"));
    }

    doc.save(&mut std::io::BufWriter::new(
        fs::File::create(&path).context("create PDF")?,
    ))
    .context("write PDF")?;

    Ok(fname.to_string())
}

fn write_line(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    size: f32,
    x: f32,
    y: f32,
    text: &str,
) {
    layer.use_text(text, Mm(size).0 as f32, Mm(x), Mm(y), font);
}

/// Create a ZIP archive of all files in the output directory.
pub fn create_zip(out_dir: &Path) -> Result<String> {
    let fname = "bundle.zip";
    let zip_path = out_dir.join(fname);
    let file = fs::File::create(&zip_path).context("create ZIP")?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            // Don't include the zip itself.
            if name == fname {
                continue;
            }
            zip.start_file(&name, options)?;
            let data = fs::read(&path)?;
            zip.write_all(&data)?;
        }
    }
    zip.finish()?;
    Ok(fname.to_string())
}
