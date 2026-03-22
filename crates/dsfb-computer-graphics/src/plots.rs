use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::frame::{BoundingBox, Color, ImageFrame, ScalarField};
use crate::metrics::MetricsReport;
use crate::sampling::DemoBMetrics;

pub struct DemoBFigureInputs<'a> {
    pub reference: &'a ImageFrame,
    pub uniform: &'a ImageFrame,
    pub guided: &'a ImageFrame,
    pub uniform_error: &'a ScalarField,
    pub guided_error: &'a ScalarField,
    pub guided_spp: &'a ScalarField,
    pub focus_bbox: BoundingBox,
    pub metrics: &'a DemoBMetrics,
}

pub fn write_system_diagram(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let labels = [
        "Inputs",
        "Residuals",
        "Proxies",
        "Grammar",
        "Trust",
        "Intervention",
    ];
    let fills = [
        "#11324d", "#204a68", "#2f5d7c", "#457b9d", "#4d9078", "#a44a3f",
    ];
    let mut boxes = String::new();

    for (index, (label, fill)) in labels.iter().zip(fills.iter()).enumerate() {
        let x = 40 + index as i32 * 170;
        let arrow_x = x + 134;
        let _ = write!(
            boxes,
            r##"<rect x="{x}" y="86" rx="18" ry="18" width="134" height="72" fill="{fill}" stroke="#f4f7fb" stroke-width="2"/>"##
        );
        let _ = write!(
            boxes,
            r##"<text x="{}" y="130" text-anchor="middle" font-size="26" font-family="Arial, Helvetica, sans-serif" fill="#f8fbff">{label}</text>"##,
            x + 67
        );
        if index + 1 < labels.len() {
            let _ = write!(
                boxes,
                r##"<line x1="{arrow_x}" y1="122" x2="{}" y2="122" stroke="#f4f7fb" stroke-width="4" marker-end="url(#arrow)"/>"##,
                x + 170
            );
        }
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1080" height="240" viewBox="0 0 1080 240">
<defs>
  <marker id="arrow" markerWidth="12" markerHeight="12" refX="10" refY="6" orient="auto">
    <path d="M0,0 L12,6 L0,12 z" fill="#f4f7fb"/>
  </marker>
</defs>
<rect width="1080" height="240" fill="#0b1320"/>
<text x="40" y="42" font-size="32" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB Supervisory Flow for Temporal Accumulation</text>
<text x="40" y="68" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#b9c6d3">Inputs → Residuals → Proxies → Grammar → Trust → Intervention</text>
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
    let legend = color_ramp_svg(760.0, 120.0, 36.0, 240.0);

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="980" height="520" viewBox="0 0 980 520">
<rect width="980" height="520" fill="#0b1320"/>
<text x="36" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Figure 2. Trust Map on the Canonical Reveal Frame</text>
<text x="36" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Red encodes low trust; stable regions remain largely unmarked.</text>
<image href="{base_png}" x="36" y="72" width="640" height="384" preserveAspectRatio="none"/>
<image href="{overlay_png}" x="36" y="72" width="640" height="384" opacity="0.74" preserveAspectRatio="none"/>
<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="none" stroke="#f4f7fb" stroke-width="2.5" stroke-dasharray="10 7"/>
<text x="{x}" y="{label_y}" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">thin-geometry disocclusion</text>
<line x1="{line_x1}" y1="{line_y1}" x2="{line_x2}" y2="{line_y2}" stroke="#f4f7fb" stroke-width="2"/>
<text x="722" y="102" font-size="22" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Trust Legend</text>
{legend}
<text x="806" y="386" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">low trust</text>
<text x="806" y="148" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">high trust</text>
<text x="722" y="430" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate</text>
<text x="722" y="452" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">behavioral differences rather than establish</text>
<text x="722" y="474" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">optimal performance.</text>
</svg>"##,
        label_y = y - 12.0,
        line_x1 = x + width,
        line_y1 = y + 8.0,
        line_x2 = x + width + 42.0,
        line_y2 = y - 8.0,
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_before_after_figure(
    baseline_frame: &ImageFrame,
    dsfb_frame: &ImageFrame,
    focus_bbox: BoundingBox,
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let baseline_png = png_data_uri(&baseline_frame.encode_png()?);
    let dsfb_png = png_data_uri(&dsfb_frame.encode_png()?);
    let crop = focus_bbox.expand(baseline_frame.width(), baseline_frame.height(), 2);
    let baseline_crop = png_data_uri(&baseline_frame.crop(crop).encode_png()?);
    let dsfb_crop = png_data_uri(&dsfb_frame.crop(crop).encode_png()?);

    let baseline_box_x = crop.min_x as f32 * 3.2 + 36.0;
    let baseline_box_y = crop.min_y as f32 * 3.2 + 78.0;
    let baseline_box_w = crop.width() as f32 * 3.2;
    let baseline_box_h = crop.height() as f32 * 3.2;
    let dsfb_box_x = baseline_box_x + 420.0;

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1140" height="700" viewBox="0 0 1140 700">
<rect width="1140" height="700" fill="#0b1320"/>
<text x="36" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Figure 3. Baseline Ghosting vs DSFB-Gated Response</text>
<text x="36" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Both panels use the same deterministic frame. The dashed box isolates the persistence ROI.</text>
<text x="200" y="98" font-size="22" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Baseline fixed-alpha TAA</text>
<text x="646" y="98" font-size="22" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB-gated TAA</text>
<image href="{baseline_png}" x="36" y="112" width="420" height="268" preserveAspectRatio="none"/>
<image href="{dsfb_png}" x="492" y="112" width="420" height="268" preserveAspectRatio="none"/>
<rect x="{baseline_box_x}" y="{baseline_box_y}" width="{baseline_box_w}" height="{baseline_box_h}" fill="none" stroke="#f4f7fb" stroke-width="2.5" stroke-dasharray="10 7"/>
<rect x="{dsfb_box_x}" y="{baseline_box_y}" width="{baseline_box_w}" height="{baseline_box_h}" fill="none" stroke="#f4f7fb" stroke-width="2.5" stroke-dasharray="10 7"/>
<text x="36" y="430" font-size="20" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">ROI zoom</text>
<image href="{baseline_crop}" x="36" y="452" width="420" height="210" preserveAspectRatio="none"/>
<image href="{dsfb_crop}" x="492" y="452" width="420" height="210" preserveAspectRatio="none"/>
<text x="940" y="168" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Interpretation</text>
<text x="940" y="198" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Left: stale history remains on the revealed thin line.</text>
<text x="940" y="222" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Right: low trust raises the current-frame weight earlier.</text>
<text x="940" y="278" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate</text>
<text x="940" y="300" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">behavioral differences rather than establish</text>
<text x="940" y="322" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">optimal performance.</text>
</svg>"##
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_trust_vs_error_figure(metrics: &MetricsReport, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let width = 960.0f32;
    let height = 540.0f32;
    let left = 88.0f32;
    let right = 820.0f32;
    let top = 78.0f32;
    let bottom = 460.0f32;
    let inner_width = right - left;
    let inner_height = bottom - top;

    let max_error = metrics
        .frame_metrics
        .iter()
        .map(|frame| {
            frame
                .persistence_roi_mae_baseline
                .max(frame.persistence_roi_mae_dsfb)
        })
        .fold(0.0f32, f32::max)
        .max(metrics.summary.persistence_threshold)
        .max(0.05);
    let frame_count = metrics.frame_metrics.len().max(2);
    let x_scale = inner_width / (frame_count.saturating_sub(1)) as f32;

    let baseline_path = polyline(
        &metrics
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.persistence_roi_mae_baseline / max_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let dsfb_path = polyline(
        &metrics
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - (frame.persistence_roi_mae_dsfb / max_error) * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let trust_path = polyline(
        &metrics
            .frame_metrics
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                (
                    left + index as f32 * x_scale,
                    bottom - frame.persistence_roi_trust * inner_height,
                )
            })
            .collect::<Vec<_>>(),
    );
    let reveal_x = left + metrics.summary.reveal_frame as f32 * x_scale;

    let mut y_grid = String::new();
    for tick in 0..=5 {
        let value = max_error * tick as f32 / 5.0;
        let y = bottom - inner_height * tick as f32 / 5.0;
        let _ = write!(
            y_grid,
            r##"<line x1="{left}" y1="{y}" x2="{right}" y2="{y}" stroke="#324253" stroke-width="1"/>"##
        );
        let _ = write!(
            y_grid,
            r##"<text x="28" y="{}" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{value:.3}</text>"##,
            y + 5.0
        );
        let trust_label = 1.0 - tick as f32 / 5.0;
        let _ = write!(
            y_grid,
            r##"<text x="842" y="{}" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{trust_label:.2}</text>"##,
            y + 5.0
        );
    }

    let mut x_grid = String::new();
    for tick in 0..frame_count {
        let x = left + tick as f32 * x_scale;
        let _ = write!(
            x_grid,
            r##"<line x1="{x}" y1="{top}" x2="{x}" y2="{bottom}" stroke="#1d2833" stroke-width="1"/>"##
        );
        let _ = write!(
            x_grid,
            r##"<text x="{}" y="486" font-size="15" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">{tick}</text>"##,
            x - 6.0
        );
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="{width}" height="{height}" fill="#0b1320"/>
<text x="36" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Figure 4. Trust and Error Through Failure Onset</text>
<text x="36" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">ROI error uses the revealed thin-geometry mask; trust uses the DSFB trust field on that same mask.</text>
{y_grid}
{x_grid}
<line x1="{left}" y1="{top}" x2="{left}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{left}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<line x1="{right}" y1="{top}" x2="{right}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2"/>
<path d="{baseline_path}" fill="none" stroke="#ef476f" stroke-width="3.5"/>
<path d="{dsfb_path}" fill="none" stroke="#4cc9f0" stroke-width="3.5"/>
<path d="{trust_path}" fill="none" stroke="#8bd450" stroke-width="3.5" stroke-dasharray="12 8"/>
<line x1="{reveal_x}" y1="{top}" x2="{reveal_x}" y2="{bottom}" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="9 7"/>
<text x="{reveal_label_x}" y="104" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">reveal onset</text>
<text x="98" y="520" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">frame index</text>
<text x="18" y="86" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">error</text>
<text x="834" y="86" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">trust</text>
<line x1="560" y1="88" x2="606" y2="88" stroke="#ef476f" stroke-width="3.5"/>
<text x="614" y="93" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">baseline ROI error</text>
<line x1="560" y1="116" x2="606" y2="116" stroke="#4cc9f0" stroke-width="3.5"/>
<text x="614" y="121" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB ROI error</text>
<line x1="560" y1="144" x2="606" y2="144" stroke="#8bd450" stroke-width="3.5" stroke-dasharray="12 8"/>
<text x="614" y="149" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB ROI trust</text>
<text x="560" y="198" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate</text>
<text x="560" y="220" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">behavioral differences rather than establish</text>
<text x="560" y="242" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">optimal performance.</text>
</svg>"##,
        reveal_label_x = (reveal_x + 10.0).min(right - 90.0),
    );

    fs::write(path, svg)?;
    Ok(())
}

pub fn write_demo_b_sampling_figure(inputs: &DemoBFigureInputs<'_>, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let reference_png = png_data_uri(&inputs.reference.encode_png()?);
    let uniform_png = png_data_uri(&inputs.uniform.encode_png()?);
    let guided_png = png_data_uri(&inputs.guided.encode_png()?);
    let uniform_error_png =
        png_data_uri(&field_to_image(inputs.uniform_error, error_palette).encode_png()?);
    let guided_error_png =
        png_data_uri(&field_to_image(inputs.guided_error, error_palette).encode_png()?);
    let guided_spp_png = png_data_uri(
        &field_to_image(inputs.guided_spp, |value| {
            allocation_palette(value, inputs.metrics.guided_max_spp as f32)
        })
        .encode_png()?,
    );

    let box_x = inputs.focus_bbox.min_x as f32 * 2.0 + 32.0;
    let box_y = inputs.focus_bbox.min_y as f32 * 2.0 + 90.0;
    let box_w = inputs.focus_bbox.width() as f32 * 2.0;
    let box_h = inputs.focus_bbox.height() as f32 * 2.0;

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1040" height="860" viewBox="0 0 1040 860">
<rect width="1040" height="860" fill="#0b1320"/>
<text x="28" y="40" font-size="30" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Demo B. Fixed-Budget Adaptive Sampling on the Reveal Frame</text>
<text x="28" y="64" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Top row: reference, uniform allocation, DSFB-guided allocation. Bottom row: uniform error, DSFB error, guided sample density.</text>
<text x="72" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Reference ({reference_spp} spp)</text>
<text x="392" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Uniform budget ({uniform_spp} spp/pixel)</text>
<text x="724" y="86" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB-guided fixed budget</text>
<image href="{reference_png}" x="32" y="90" width="320" height="192" preserveAspectRatio="none"/>
<image href="{uniform_png}" x="360" y="90" width="320" height="192" preserveAspectRatio="none"/>
<image href="{guided_png}" x="688" y="90" width="320" height="192" preserveAspectRatio="none"/>
<rect x="{box_x}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{uniform_box_x}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<rect x="{guided_box_x}" y="{box_y}" width="{box_w}" height="{box_h}" fill="none" stroke="#f4f7fb" stroke-width="2" stroke-dasharray="8 6"/>
<text x="78" y="326" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Uniform absolute error</text>
<text x="403" y="326" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">DSFB-guided absolute error</text>
<text x="735" y="326" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Guided samples per pixel</text>
<image href="{uniform_error_png}" x="32" y="330" width="320" height="192" preserveAspectRatio="none"/>
<image href="{guided_error_png}" x="360" y="330" width="320" height="192" preserveAspectRatio="none"/>
<image href="{guided_spp_png}" x="688" y="330" width="320" height="192" preserveAspectRatio="none"/>
<text x="32" y="572" font-size="18" font-family="Arial, Helvetica, sans-serif" fill="#f4f7fb">Measured outcome</text>
<text x="32" y="604" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Uniform MAE: {uniform_mae:.5}</text>
<text x="32" y="628" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Guided MAE: {guided_mae:.5}</text>
<text x="32" y="652" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Uniform ROI MAE: {uniform_roi_mae:.5}</text>
<text x="32" y="676" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Guided ROI MAE: {guided_roi_mae:.5}</text>
<text x="32" y="700" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">ROI mean spp: uniform {uniform_spp:.2}, guided {guided_roi_spp:.2}</text>
<text x="32" y="724" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">Guided max spp: {max_guided_spp}</text>
<text x="32" y="764" font-size="16" font-family="Arial, Helvetica, sans-serif" fill="#c6d2dd">The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.</text>
</svg>"##,
        reference_spp = inputs.metrics.reference_spp,
        uniform_spp = inputs.metrics.uniform_spp,
        uniform_mae = inputs.metrics.uniform_mae,
        guided_mae = inputs.metrics.guided_mae,
        uniform_roi_mae = inputs.metrics.uniform_roi_mae,
        guided_roi_mae = inputs.metrics.guided_roi_mae,
        guided_roi_spp = inputs.metrics.roi_mean_guided_spp,
        max_guided_spp = inputs.metrics.max_guided_spp,
        uniform_box_x = box_x + 328.0,
        guided_box_x = box_x + 656.0,
    );

    fs::write(path, svg)?;
    Ok(())
}

fn trust_overlay_image(trust: &ScalarField) -> ImageFrame {
    let mut frame = ImageFrame::new(trust.width(), trust.height());
    for y in 0..trust.height() {
        for x in 0..trust.width() {
            let hazard = (1.0 - trust.get(x, y)).clamp(0.0, 1.0);
            let color = if hazard <= 0.02 {
                Color::rgb(0.0, 0.0, 0.0)
            } else {
                Color::rgb(0.95 * hazard, 0.15 + 0.65 * hazard, 0.10 * hazard)
            };
            frame.set(x, y, color);
        }
    }
    frame
}

fn color_ramp_svg(x: f32, y: f32, width: f32, height: f32) -> String {
    format!(
        r##"<defs>
  <linearGradient id="trustRamp" x1="0%" y1="0%" x2="0%" y2="100%">
    <stop offset="0%" stop-color="#15202b"/>
    <stop offset="40%" stop-color="#ffb703"/>
    <stop offset="100%" stop-color="#ef476f"/>
  </linearGradient>
</defs>
<rect x="{x}" y="{y}" width="{width}" height="{height}" rx="12" fill="url(#trustRamp)" stroke="#f4f7fb" stroke-width="1.5"/>"##
    )
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

fn error_palette(value: f32) -> Color {
    let normalized = (value / 0.20).clamp(0.0, 1.0);
    Color::rgb(
        0.12 + 0.88 * normalized,
        0.08 + 0.75 * normalized.powf(0.6),
        0.16 * (1.0 - normalized),
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
