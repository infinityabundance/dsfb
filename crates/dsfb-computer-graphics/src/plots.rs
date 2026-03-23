use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::frame::{BoundingBox, Color, ImageFrame, ScalarField};
use crate::metrics::{AblationEntry, AggregateRunScore, DemoASuiteMetrics, ScenarioReport};
use crate::report::TrustDiagnostics;
use crate::sampling::{BudgetCurve, DemoBScenarioReport, DemoBScenarioRun};
use crate::scaling::ResolutionScalingMetrics;
use crate::sensitivity::ParameterSensitivityMetrics;

pub struct ScenarioMosaicEntry<'a> {
    pub scenario_title: &'a str,
    pub baseline: &'a ImageFrame,
    pub heuristic: &'a ImageFrame,
    pub host_realistic: &'a ImageFrame,
    pub focus_bbox: BoundingBox,
}

pub fn write_system_diagram(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let labels = [
        "Host Buffers",
        "Residuals",
        "Proxies",
        "State / Grammar",
        "Trust",
        "Alpha / Budget",
    ];
    let fills = [
        "#11324d", "#204a68", "#2f5d7c", "#457b9d", "#4d9078", "#a44a3f",
    ];
    let mut boxes = String::new();
    for (index, (label, fill)) in labels.iter().zip(fills.iter()).enumerate() {
        let x = 40 + index as i32 * 172;
        let arrow_x = x + 134;
        let _ = write!(
            boxes,
            r##"<rect x="{x}" y="86" rx="18" ry="18" width="134" height="72" fill="{fill}" stroke="#f4f7fb" stroke-width="2"/>"##
        );
        let _ = write!(
            boxes,
            r##"<text x="{}" y="130" text-anchor="middle" font-size="24" font-family="Arial, Helvetica, sans-serif" fill="#f8fbff">{label}</text>"##,
            x + 67
        );
        if index + 1 < labels.len() {
            let _ = write!(
                boxes,
                r##"<line x1="{arrow_x}" y1="122" x2="{}" y2="122" stroke="#f4f7fb" stroke-width="4" marker-end="url(#arrow)"/>"##,
                x + 172
            );
        }
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1110" height="240" viewBox="0 0 1110 240">
<defs>
  <marker id="arrow" markerWidth="12" markerHeight="12" refX="10" refY="6" orient="auto">
    <path d="M0,0 L12,6 L0,12 z" fill="#f4f7fb"/>
  </marker>
</defs>
<rect width="1110" height="240" fill="#0b1320"/>
<text x="40" y="42" font-size="32" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB Supervisory Flow and Integration Surface</text>
<text x="40" y="68" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#b9c6d3">Inputs → Residuals → Proxies → Grammar → Trust → Intervention / Modulation</text>
{boxes}
</svg>"##
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_trust_map_figure(
    current_frame: &ImageFrame,
    trust: &ScalarField,
    focus_bbox: BoundingBox,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let base_png = png_data_uri(&current_frame.encode_png()?);
    let overlay_png = png_data_uri(&trust_overlay_image(trust).encode_png()?);
    let x = focus_bbox.min_x as f32 * 4.0 + 36.0;
    let y = focus_bbox.min_y as f32 * 4.0 + 72.0;
    let width = focus_bbox.width() as f32 * 4.0;
    let height = focus_bbox.height() as f32 * 4.0;

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="960" height="500" viewBox="0 0 960 500">
<rect width="960" height="500" fill="#0b1320"/>
<text x="36" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Canonical Trust Map</text>
<text x="36" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Low trust is overlaid on the actual host-realistic reveal frame.</text>
<image href="{base_png}" x="36" y="72" width="640" height="384" preserveAspectRatio="none"/>
<image href="{overlay_png}" x="36" y="72" width="640" height="384" opacity="0.72" preserveAspectRatio="none"/>
<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="none" stroke="#f4f7fb" stroke-width="2.5" stroke-dasharray="10 7"/>
<text x="720" y="112" font-size="22" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Legend</text>
<rect x="720" y="138" width="28" height="220" rx="10" fill="url(#trustRamp)" stroke="#f4f7fb" stroke-width="1.5"/>
<defs>
  <linearGradient id="trustRamp" x1="0%" y1="0%" x2="0%" y2="100%">
    <stop offset="0%" stop-color="#0d1b2a"/>
    <stop offset="40%" stop-color="#ffb703"/>
    <stop offset="100%" stop-color="#ef476f"/>
  </linearGradient>
</defs>
<text x="760" y="156" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">high trust</text>
<text x="760" y="354" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">low trust</text>
<text x="720" y="406" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate</text>
<text x="720" y="428" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">behavioral differences rather than establish</text>
<text x="720" y="450" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">optimal performance.</text>
</svg>"##
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_before_after_figure(
    baseline_frame: &ImageFrame,
    strong_heuristic_frame: &ImageFrame,
    host_realistic_frame: &ImageFrame,
    focus_bbox: BoundingBox,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let baseline_png = png_data_uri(&baseline_frame.encode_png()?);
    let strong_png = png_data_uri(&strong_heuristic_frame.encode_png()?);
    let host_png = png_data_uri(&host_realistic_frame.encode_png()?);
    let crop = focus_bbox.expand(baseline_frame.width(), baseline_frame.height(), 2);
    let baseline_crop = png_data_uri(&baseline_frame.crop(crop).encode_png()?);
    let strong_crop = png_data_uri(&strong_heuristic_frame.crop(crop).encode_png()?);
    let host_crop = png_data_uri(&host_realistic_frame.crop(crop).encode_png()?);
    let box_x = crop.min_x as f32 * 2.2 + 30.0;
    let box_y = crop.min_y as f32 * 2.2 + 90.0;
    let box_w = crop.width() as f32 * 2.2;
    let box_h = crop.height() as f32 * 2.2;

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1340" height="760" viewBox="0 0 1340 760">
<rect width="1340" height="760" fill="#0b1320"/>
<text x="30" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Canonical Baseline Comparison</text>
<text x="30" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Fixed alpha, strong heuristic, and host-realistic DSFB on the same deterministic frame.</text>
<text x="110" y="88" font-size="20" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Fixed alpha</text>
<text x="520" y="88" font-size="20" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Strong heuristic</text>
<text x="936" y="88" font-size="20" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Host-realistic DSFB</text>
<image href="{baseline_png}" x="30" y="96" width="360" height="220" preserveAspectRatio="none"/>
<image href="{strong_png}" x="450" y="96" width="360" height="220" preserveAspectRatio="none"/>
<image href="{host_png}" x="870" y="96" width="360" height="220" preserveAspectRatio="none"/>
<rect x="{box_x}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{box_x2}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{box_x3}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<text x="32" y="360" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">ROI zooms</text>
<image href="{baseline_crop}" x="30" y="382" width="360" height="180" preserveAspectRatio="none"/>
<image href="{strong_crop}" x="450" y="382" width="360" height="180" preserveAspectRatio="none"/>
<image href="{host_crop}" x="870" y="382" width="360" height="180" preserveAspectRatio="none"/>
<text x="30" y="626" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.</text>
</svg>"##,
        box_x2 = box_x + 420.0,
        box_x3 = box_x + 840.0,
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_trust_vs_error_figure(scenario: &ScenarioReport, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let fixed = scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == "fixed_alpha")
        .expect("fixed_alpha run required");
    let host = scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == "dsfb_host_realistic")
        .expect("dsfb_host_realistic run required");
    let trust_values = host
        .frame_metrics
        .iter()
        .map(|frame| frame.trust_roi_mean.unwrap_or(1.0))
        .collect::<Vec<_>>();
    let width = 960.0f32;
    let height = 540.0f32;
    let left = 88.0f32;
    let right = 820.0f32;
    let top = 78.0f32;
    let bottom = 460.0f32;
    let inner_width = right - left;
    let inner_height = bottom - top;
    let max_error = fixed
        .frame_metrics
        .iter()
        .chain(host.frame_metrics.iter())
        .map(|frame| frame.roi_mae)
        .fold(0.05f32, f32::max);
    let frame_count = fixed.frame_metrics.len().max(2);
    let x_scale = inner_width / (frame_count.saturating_sub(1)) as f32;

    let fixed_path = polyline(
        &fixed
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.roi_mae / max_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let host_path = polyline(
        &host
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.roi_mae / max_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let trust_path = polyline(
        &trust_values
            .iter()
            .enumerate()
            .map(|(index, trust)| {
                (
                    left + index as f32 * x_scale,
                    bottom - trust.clamp(0.0, 1.0) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="36" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Canonical ROI Error vs Trust</text>
<text x="36" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">x-axis = frame index, left y-axis = ROI MAE, right y-axis = host-realistic trust.</text>
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{right}" y1="{top}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<path d="{fixed_path}" fill="none" stroke="#ef476f" stroke-width="3.5"/>
<path d="{host_path}" fill="none" stroke="#4cc9f0" stroke-width="3.5"/>
<path d="{trust_path}" fill="none" stroke="#8bd450" stroke-width="3.5" stroke-dasharray="10 8"/>
<line x1="{onset_x}" y1="{top}" x2="{onset_x}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<text x="560" y="110" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Fixed alpha ROI error</text>
<line x1="500" y1="104" x2="548" y2="104" stroke="#ef476f" stroke-width="3.5"/>
<text x="560" y="138" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Host-realistic DSFB ROI error</text>
<line x1="500" y1="132" x2="548" y2="132" stroke="#4cc9f0" stroke-width="3.5"/>
<text x="560" y="166" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Host-realistic trust</text>
<line x1="500" y1="160" x2="548" y2="160" stroke="#8bd450" stroke-width="3.5" stroke-dasharray="10 8"/>
<text x="36" y="515" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">frame index</text>
<text x="18" y="88" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">error</text>
<text x="840" y="88" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">trust</text>
<text x="500" y="236" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">onset frame: {onset_frame}</text>
<text x="500" y="260" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">host vs fixed cumulative ROI gain: {roi_gain:.5}</text>
</svg>"##,
        onset_x = left + scenario.onset_frame as f32 * x_scale,
        onset_frame = scenario.onset_frame,
        roi_gain = scenario.host_realistic_vs_fixed_alpha_cumulative_roi_gain,
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_intervention_alpha_figure(
    current_frame: &ImageFrame,
    intervention: &ScalarField,
    alpha: &ScalarField,
    focus_bbox: BoundingBox,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let base_png = png_data_uri(&current_frame.encode_png()?);
    let intervention_png =
        png_data_uri(&field_to_image(intervention, intervention_palette).encode_png()?);
    let alpha_png = png_data_uri(&field_to_image(alpha, alpha_palette).encode_png()?);
    let x = focus_bbox.min_x as f32 * 2.4 + 30.0;
    let y = focus_bbox.min_y as f32 * 2.4 + 86.0;
    let width = focus_bbox.width() as f32 * 2.4;
    let height = focus_bbox.height() as f32 * 2.4;

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1040" height="560" viewBox="0 0 1040 560">
<rect width="1040" height="560" fill="#0b1320"/>
<text x="30" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Intervention and Alpha Maps</text>
<text x="30" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Current frame, intervention field, and alpha field for the canonical host-realistic onset frame.</text>
<text x="112" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Current frame</text>
<text x="454" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Intervention</text>
<text x="790" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Alpha</text>
<image href="{base_png}" x="30" y="94" width="300" height="180" preserveAspectRatio="none"/>
<image href="{intervention_png}" x="370" y="94" width="300" height="180" preserveAspectRatio="none"/>
<image href="{alpha_png}" x="710" y="94" width="300" height="180" preserveAspectRatio="none"/>
<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{x2}" y="{y}" width="{width}" height="{height}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{x3}" y="{y}" width="{width}" height="{height}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
</svg>"##,
        x2 = x + 340.0,
        x3 = x + 680.0,
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_ablation_bar_figure(entries: &[AblationEntry], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let width = 980.0;
    let height = 520.0;
    let left = 210.0;
    let top = 84.0;
    let bar_height = 28.0;
    let row_gap = 18.0;
    let max_value = entries
        .iter()
        .map(|entry| entry.canonical_cumulative_roi_mae)
        .fold(0.01f32, f32::max);
    let mut rows = String::new();
    for (index, entry) in entries.iter().enumerate() {
        let y = top + index as f32 * (bar_height + row_gap);
        let width_px = 620.0 * (entry.canonical_cumulative_roi_mae / max_value);
        let _ = write!(
            rows,
            r##"<text x="18" y="{}" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{}</text>
<rect x="{left}" y="{y}" width="{width_px}" height="{bar_height}" rx="8" fill="#4cc9f0"/>
<text x="{}" y="{}" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{:.5}</text>"##,
            y + 20.0,
            entry.label,
            left + width_px + 10.0,
            y + 19.0,
            entry.canonical_cumulative_roi_mae
        );
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="18" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Canonical Ablation Comparison</text>
<text x="18" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Bar length = cumulative ROI MAE on the canonical scenario. Lower is better.</text>
{rows}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_roi_nonroi_error_figure(scenario: &ScenarioReport, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let fixed = scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == "fixed_alpha")
        .expect("fixed_alpha run required");
    let host = scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == "dsfb_host_realistic")
        .expect("dsfb_host_realistic run required");
    let width = 960.0f32;
    let height = 520.0f32;
    let left = 80.0f32;
    let right = 860.0f32;
    let top = 80.0f32;
    let bottom = 440.0f32;
    let inner_width = right - left;
    let inner_height = bottom - top;
    let max_value = fixed
        .frame_metrics
        .iter()
        .chain(host.frame_metrics.iter())
        .map(|frame| frame.roi_mae.max(frame.non_roi_mae))
        .fold(0.05f32, f32::max);
    let frame_count = fixed.frame_metrics.len().max(2);
    let x_scale = inner_width / (frame_count.saturating_sub(1)) as f32;
    let fixed_roi = polyline(
        &fixed
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.roi_mae / max_value) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let host_roi = polyline(
        &host
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.roi_mae / max_value) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let host_non_roi = polyline(
        &host
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.non_roi_mae / max_value) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="28" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">ROI vs Non-ROI Error</text>
<text x="28" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Canonical scenario. This makes the non-target stability tradeoff visible.</text>
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<path d="{fixed_roi}" fill="none" stroke="#ef476f" stroke-width="3.5"/>
<path d="{host_roi}" fill="none" stroke="#4cc9f0" stroke-width="3.5"/>
<path d="{host_non_roi}" fill="none" stroke="#ffd166" stroke-width="3.5" stroke-dasharray="10 8"/>
<text x="540" y="110" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Fixed alpha ROI</text>
<line x1="484" y1="104" x2="530" y2="104" stroke="#ef476f" stroke-width="3.5"/>
<text x="540" y="138" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Host-realistic ROI</text>
<line x1="484" y1="132" x2="530" y2="132" stroke="#4cc9f0" stroke-width="3.5"/>
<text x="540" y="166" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Host-realistic non-ROI</text>
<line x1="484" y1="160" x2="530" y2="160" stroke="#ffd166" stroke-width="3.5" stroke-dasharray="10 8"/>
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_leaderboard_figure(entries: &[AggregateRunScore], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut rows = String::new();
    let mut y = 110.0f32;
    for (rank, entry) in entries.iter().take(10).enumerate() {
        let _ = write!(
            rows,
            r##"<text x="24" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{rank_label}</text>
<text x="80" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{label}</text>
<text x="420" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{mean_rank:.2}</text>
<text x="560" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{roi_mae:.5}</text>
<text x="720" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{non_roi_mae:.5}</text>
<text x="870" y="{y}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{wins}</text>"##,
            rank_label = rank + 1,
            label = entry.label,
            mean_rank = entry.mean_rank,
            roi_mae = entry.mean_cumulative_roi_mae,
            non_roi_mae = entry.mean_non_roi_mae,
            wins = entry.benefit_scenarios_won,
        );
        y += 34.0;
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="980" height="520" viewBox="0 0 980 520">
<rect width="980" height="520" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Aggregate Scenario Leaderboard</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Mean rank uses scenario-appropriate primary metrics. Lower is better.</text>
<text x="24" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Rank</text>
<text x="80" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Run</text>
<text x="420" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Mean rank</text>
<text x="560" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Mean ROI MAE</text>
<text x="720" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Mean non-ROI MAE</text>
<text x="870" y="94" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Wins</text>
{rows}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_scenario_mosaic_figure(
    entries: &[ScenarioMosaicEntry<'_>],
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut body = String::new();
    let tile_width = 300.0;
    let tile_height = 180.0;
    let row_height = 245.0;
    for (row, entry) in entries.iter().enumerate() {
        let y = 92.0 + row as f32 * row_height;
        let baseline_png = png_data_uri(&entry.baseline.encode_png()?);
        let heuristic_png = png_data_uri(&entry.heuristic.encode_png()?);
        let host_png = png_data_uri(&entry.host_realistic.encode_png()?);
        let _ = write!(
            body,
            r##"<text x="24" y="{}" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{}</text>
<image href="{baseline_png}" x="24" y="{}" width="{tile_width}" height="{tile_height}" preserveAspectRatio="none"/>
<image href="{heuristic_png}" x="348" y="{}" width="{tile_width}" height="{tile_height}" preserveAspectRatio="none"/>
<image href="{host_png}" x="672" y="{}" width="{tile_width}" height="{tile_height}" preserveAspectRatio="none"/>"##,
            y - 10.0,
            entry.scenario_title,
            y,
            y,
            y,
        );
    }

    let height = 100.0 + entries.len() as f32 * row_height;
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="{height}" viewBox="0 0 1000 {height}">
<rect width="1000" height="{height}" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Per-Scenario Comparison Mosaic</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Each row shows fixed alpha, strong heuristic, and host-realistic DSFB on a different scenario.</text>
<text x="120" y="88" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Fixed alpha</text>
<text x="430" y="88" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Strong heuristic</text>
<text x="746" y="88" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Host-realistic DSFB</text>
{body}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_demo_b_sampling_figure(
    scenario_report: &DemoBScenarioReport,
    scenario_run: &DemoBScenarioRun,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let uniform = scenario_run
        .policy_runs
        .iter()
        .find(|run| run.policy_id == crate::sampling::AllocationPolicyId::Uniform)
        .expect("uniform run required");
    let imported = scenario_run
        .policy_runs
        .iter()
        .find(|run| run.policy_id == crate::sampling::AllocationPolicyId::ImportedTrust)
        .expect("imported trust run required");
    let combined = scenario_run
        .policy_runs
        .iter()
        .find(|run| run.policy_id == crate::sampling::AllocationPolicyId::CombinedHeuristic)
        .expect("combined heuristic run required");
    let reference_png = png_data_uri(&scenario_run.reference_frame.encode_png()?);
    let uniform_png = png_data_uri(&uniform.frame.encode_png()?);
    let combined_png = png_data_uri(&combined.frame.encode_png()?);
    let imported_png = png_data_uri(&imported.frame.encode_png()?);
    let combined_spp = png_data_uri(
        &field_to_image(&combined.spp, |value| {
            allocation_palette(value, combined.metrics.max_spp as f32)
        })
        .encode_png()?,
    );
    let imported_spp = png_data_uri(
        &field_to_image(&imported.spp, |value| {
            allocation_palette(value, imported.metrics.max_spp as f32)
        })
        .encode_png()?,
    );

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1180" height="760" viewBox="0 0 1180 760">
<rect width="1180" height="760" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Demo B Policy Comparison</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{}</text>
<text x="60" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Reference</text>
<text x="346" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Uniform</text>
<text x="628" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Combined heuristic</text>
<text x="896" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Imported trust</text>
<image href="{reference_png}" x="24" y="102" width="260" height="156" preserveAspectRatio="none"/>
<image href="{uniform_png}" x="308" y="102" width="260" height="156" preserveAspectRatio="none"/>
<image href="{combined_png}" x="592" y="102" width="260" height="156" preserveAspectRatio="none"/>
<image href="{imported_png}" x="876" y="102" width="260" height="156" preserveAspectRatio="none"/>
<text x="320" y="306" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Combined heuristic spp</text>
<text x="888" y="306" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Imported trust spp</text>
<image href="{combined_spp}" x="308" y="314" width="260" height="156" preserveAspectRatio="none"/>
<image href="{imported_spp}" x="876" y="314" width="260" height="156" preserveAspectRatio="none"/>
<text x="24" y="536" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Uniform ROI MAE: {uniform_roi:.5}</text>
<text x="24" y="560" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Combined ROI MAE: {combined_roi:.5}</text>
<text x="24" y="584" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Imported trust ROI MAE: {imported_roi:.5}</text>
</svg>"##,
        scenario_report.headline,
        uniform_roi = uniform.metrics.roi_mae,
        combined_roi = combined.metrics.roi_mae,
        imported_roi = imported.metrics.roi_mae,
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_demo_b_budget_efficiency_figure(curves: &[BudgetCurve], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let relevant = curves
        .iter()
        .filter(|curve| curve.scenario_id == "thin_reveal")
        .collect::<Vec<_>>();
    let width = 960.0;
    let height = 520.0;
    let left = 90.0;
    let right = 860.0;
    let top = 80.0;
    let bottom = 430.0;
    let inner_width = right - left;
    let inner_height = bottom - top;
    let max_x = relevant
        .iter()
        .flat_map(|curve| curve.points.iter().map(|point| point.average_spp))
        .fold(1.0f32, f32::max);
    let max_y = relevant
        .iter()
        .flat_map(|curve| curve.points.iter().map(|point| point.roi_mae))
        .fold(0.05f32, f32::max);
    let colors = [
        ("uniform", "#ef476f"),
        ("combined_heuristic", "#ffd166"),
        ("imported_trust", "#4cc9f0"),
        ("hybrid_trust_variance", "#8bd450"),
    ];
    let mut paths = String::new();
    let mut legend_y = 118.0;
    for (policy_id, color) in colors {
        if let Some(curve) = relevant.iter().find(|curve| curve.policy_id == policy_id) {
            let polyline_path = polyline(
                &curve
                    .points
                    .iter()
                    .map(|point| {
                        (
                            left + (point.average_spp / max_x) * inner_width,
                            bottom - (point.roi_mae / max_y) * inner_height,
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let _ = write!(
                paths,
                r##"<path d="{polyline_path}" fill="none" stroke="{color}" stroke-width="3.5"/>
<line x1="620" y1="{legend_y}" x2="664" y2="{legend_y}" stroke="{color}" stroke-width="3.5"/>
<text x="674" y="{}" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{policy_id}</text>"##,
                legend_y + 5.0
            );
            legend_y += 28.0;
        }
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="28" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Demo B Budget Efficiency</text>
<text x="28" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Canonical scenario. x-axis = mean spp, y-axis = ROI MAE.</text>
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
{paths}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_trust_histogram_figure(diagnostics: &TrustDiagnostics, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entry = diagnostics
        .scenarios
        .iter()
        .find(|entry| {
            entry.scenario_id == "motion_bias_band" && entry.run_id == "dsfb_host_realistic"
        })
        .or_else(|| diagnostics.scenarios.first())
        .expect("trust diagnostics required");
    let max_count = entry
        .histogram
        .iter()
        .map(|bin| bin.sample_count)
        .max()
        .unwrap_or(1) as f32;
    let mut bars = String::new();
    for (index, bin) in entry.histogram.iter().enumerate() {
        let x = 70.0 + index as f32 * 52.0;
        let height = 220.0 * (bin.sample_count as f32 / max_count.max(1.0));
        let y = 340.0 - height;
        let _ = write!(
            bars,
            r##"<rect x="{x}" y="{y}" width="34" height="{height}" rx="4" fill="#4cc9f0"/>
<text x="{label_x}" y="362" font-size="12" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{lower:.1}</text>"##,
            label_x = x - 2.0,
            lower = bin.lower,
        );
    }
    let mut calibration = String::new();
    for (index, bin) in entry.calibration_bins.iter().enumerate() {
        let x = 650.0 + index as f32 * 60.0;
        let error_height = 120.0 * (bin.mean_error / 0.25).clamp(0.0, 1.0);
        let y = 320.0 - error_height;
        let _ = write!(
            calibration,
            r##"<rect x="{x}" y="{y}" width="28" height="{error_height}" rx="4" fill="#ffd166"/>
<text x="{label_x}" y="342" font-size="12" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{trust:.2}</text>"##,
            label_x = x - 6.0,
            trust = bin.mean_trust,
        );
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="980" height="420" viewBox="0 0 980 420">
<rect width="980" height="420" fill="#0b1320"/>
<text x="28" y="36" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Trust Histogram and Calibration Bins</text>
<text x="28" y="60" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{scenario} / {run}</text>
<text x="70" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Histogram</text>
<line x1="60" y1="340" x2="590" y2="340" stroke="#f4f7fb" stroke-width="2"/>
{bars}
<text x="650" y="94" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Calibration bins</text>
<line x1="640" y1="320" x2="940" y2="320" stroke="#f4f7fb" stroke-width="2"/>
{calibration}
<text x="650" y="374" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Degenerate correlation hidden from headlines when bin occupancy is weak.</text>
</svg>"##,
        scenario = entry.scenario_id,
        run = entry.run_id,
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_roi_taxonomy_figure(demo_a: &DemoASuiteMetrics, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let max_pixels = demo_a
        .scenarios
        .iter()
        .map(|scenario| scenario.target_pixels)
        .max()
        .unwrap_or(1) as f32;
    let mut rows = String::new();
    for (index, scenario) in demo_a.scenarios.iter().enumerate() {
        let y = 96.0 + index as f32 * 44.0;
        let width = 420.0 * (scenario.target_pixels as f32 / max_pixels.max(1.0));
        let color = match scenario.support_category {
            crate::scene::ScenarioSupportCategory::PointLikeRoi => "#ef476f",
            crate::scene::ScenarioSupportCategory::RegionRoi => "#4cc9f0",
            crate::scene::ScenarioSupportCategory::NegativeControl => "#8bd450",
        };
        let _ = write!(
            rows,
            r##"<text x="24" y="{y}" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{label}</text>
<rect x="320" y="{bar_y}" width="{width}" height="20" rx="6" fill="{color}"/>
<text x="{value_x}" y="{y}" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{pixels}</text>"##,
            label = scenario.scenario_id,
            bar_y = y - 14.0,
            value_x = 750.0,
            pixels = scenario.target_pixels,
        );
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="900" height="460" viewBox="0 0 900 460">
<rect width="900" height="460" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">ROI Size and Scenario Taxonomy</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Point-like ROI, region ROI, and negative-control scenarios are separated explicitly.</text>
{rows}
<text x="24" y="420" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Red = point-like ROI, blue = region ROI, green = negative control.</text>
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_parameter_sensitivity_figure(
    sensitivity: &ParameterSensitivityMetrics,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let points = sensitivity.sweep_points.iter().take(18).collect::<Vec<_>>();
    let max_value = points
        .iter()
        .map(|point| point.region_mean_cumulative_roi_mae)
        .fold(0.01f32, f32::max);
    let mut bars = String::new();
    for (index, point) in points.iter().enumerate() {
        let x = 56.0 + index as f32 * 46.0;
        let height = 220.0 * (point.region_mean_cumulative_roi_mae / max_value);
        let y = 340.0 - height;
        let fill = if point.robust_corridor_member {
            "#4cc9f0"
        } else {
            "#ef476f"
        };
        let _ = write!(
            bars,
            r##"<rect x="{x}" y="{y}" width="28" height="{height}" rx="4" fill="{fill}"/>
<text x="{label_x}" y="362" font-size="10" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd" transform="rotate(45 {label_x} 362)">{label}</text>"##,
            label_x = x - 2.0,
            label = point.parameter_id,
        );
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="960" height="420" viewBox="0 0 960 420">
<rect width="960" height="420" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Parameter Sensitivity Corridor</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Blue sweep points stay within the report's robustness corridor. Red points are fragile.</text>
<line x1="44" y1="340" x2="920" y2="340" stroke="#f4f7fb" stroke-width="2"/>
{bars}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_resolution_scaling_figure(
    scaling: &ResolutionScalingMetrics,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let relevant = scaling
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.scenario_id.as_str(),
                "thin_reveal" | "reveal_band" | "motion_bias_band"
            )
        })
        .collect::<Vec<_>>();
    let max_gain = relevant
        .iter()
        .map(|entry| entry.host_realistic_vs_fixed_alpha_gain.abs())
        .fold(0.01f32, f32::max);
    let mut bars = String::new();
    for (index, entry) in relevant.iter().enumerate() {
        let x = 52.0 + index as f32 * 58.0;
        let height = 190.0 * (entry.host_realistic_vs_fixed_alpha_gain.abs() / max_gain);
        let y = if entry.host_realistic_vs_fixed_alpha_gain >= 0.0 {
            250.0 - height
        } else {
            250.0
        };
        let fill = if entry.host_realistic_vs_fixed_alpha_gain >= 0.0 {
            "#4cc9f0"
        } else {
            "#ef476f"
        };
        let _ = write!(
            bars,
            r##"<rect x="{x}" y="{y}" width="32" height="{height}" rx="4" fill="{fill}"/>
<text x="{label_x}" y="388" font-size="10" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd" transform="rotate(45 {label_x} 388)">{label}</text>"##,
            label_x = x - 8.0,
            label = format!("{}@{}x{}", entry.scenario_id, entry.width, entry.height),
        );
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="980" height="440" viewBox="0 0 980 440">
<rect width="980" height="440" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Resolution Scaling Gain</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Positive bars mean host-realistic DSFB beats fixed alpha on cumulative ROI MAE.</text>
<line x1="40" y1="250" x2="944" y2="250" stroke="#f4f7fb" stroke-width="2"/>
{bars}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

pub fn write_motion_relevance_figure(demo_a: &DemoASuiteMetrics, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let scenarios = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.scenario_id.as_str(),
                "fast_pan" | "motion_bias_band" | "reveal_band"
            )
        })
        .collect::<Vec<_>>();
    let mut bars = String::new();
    for (index, scenario) in scenarios.iter().enumerate() {
        let host = scenario
            .runs
            .iter()
            .find(|run| run.summary.run_id == "dsfb_host_realistic")
            .expect("host run required");
        let motion = scenario
            .runs
            .iter()
            .find(|run| run.summary.run_id == "dsfb_motion_augmented")
            .expect("motion run required");
        let gain = host.summary.cumulative_roi_mae - motion.summary.cumulative_roi_mae;
        let height = 160.0 * (gain.abs() / 0.5).clamp(0.0, 1.0);
        let x = 120.0 + index as f32 * 220.0;
        let y = if gain >= 0.0 { 270.0 - height } else { 270.0 };
        let color = if gain >= 0.0 { "#4cc9f0" } else { "#ef476f" };
        let _ = write!(
            bars,
            r##"<rect x="{x}" y="{y}" width="68" height="{height}" rx="6" fill="{color}"/>
<text x="{text_x}" y="320" font-size="14" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">{label}</text>"##,
            text_x = x - 10.0,
            label = scenario.scenario_id,
        );
    }
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="860" height="380" viewBox="0 0 860 380">
<rect width="860" height="380" fill="#0b1320"/>
<text x="24" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Motion Relevance</text>
<text x="24" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Positive bars mean the optional motion-augmented path improved over the minimum host path.</text>
<line x1="70" y1="270" x2="810" y2="270" stroke="#f4f7fb" stroke-width="2"/>
{bars}
</svg>"##
    );
    fs::write(path, svg)?;
    Ok(())
}

fn trust_overlay_image(trust: &ScalarField) -> ImageFrame {
    let mut frame = ImageFrame::new(trust.width(), trust.height());
    for y in 0..trust.height() {
        for x in 0..trust.width() {
            let hazard = (1.0 - trust.get(x, y)).clamp(0.0, 1.0);
            frame.set(
                x,
                y,
                if hazard <= 0.02 {
                    Color::rgb(0.0, 0.0, 0.0)
                } else {
                    Color::rgb(0.95 * hazard, 0.15 + 0.65 * hazard, 0.10 * hazard)
                },
            );
        }
    }
    frame
}

fn field_to_image(field: &ScalarField, mapper: impl Fn(f32) -> Color) -> ImageFrame {
    let mut frame = ImageFrame::new(field.width(), field.height());
    for y in 0..field.height() {
        for x in 0..field.width() {
            frame.set(x, y, mapper(field.get(x, y)));
        }
    }
    frame
}

fn intervention_palette(value: f32) -> Color {
    let normalized = value.clamp(0.0, 1.0);
    Color::rgb(
        0.12 + 0.86 * normalized,
        0.18 + 0.55 * (1.0 - normalized),
        0.12,
    )
}

fn alpha_palette(value: f32) -> Color {
    let normalized = value.clamp(0.0, 1.0);
    Color::rgb(
        0.20 + 0.72 * normalized,
        0.16 + 0.22 * normalized,
        0.34 + 0.40 * (1.0 - normalized),
    )
}

fn allocation_palette(value: f32, max_value: f32) -> Color {
    let normalized = if max_value <= f32::EPSILON {
        0.0
    } else {
        (value / max_value).clamp(0.0, 1.0)
    };
    Color::rgb(
        0.10 + 0.88 * normalized,
        0.20 + 0.55 * (1.0 - normalized),
        0.30 + 0.50 * normalized,
    )
}

fn polyline(points: &[(f32, f32)]) -> String {
    let mut path = String::new();
    if let Some((x, y)) = points.first().copied() {
        let _ = write!(path, "M{x:.2},{y:.2}");
    }
    for (x, y) in points.iter().skip(1) {
        let _ = write!(path, " L{x:.2},{y:.2}");
    }
    path
}

fn png_data_uri(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64_encode(bytes))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut chunks = bytes.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let combined = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
        output.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(combined & 0x3f) as usize] as char);
    }

    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let first = remainder[0] as u32;
        let second = remainder.get(1).copied().unwrap_or(0) as u32;
        let combined = (first << 16) | (second << 8);
        output.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
        if remainder.len() == 2 {
            output.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
            output.push('=');
        } else {
            output.push('=');
            output.push('=');
        }
    }

    output
}
