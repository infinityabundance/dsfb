/// Mixed-regime confirmation.
///
/// A true mixed-regime case requires that BOTH aliasing pressure AND variance/noise
/// pressure are materially active in the same evaluated frame — not hypothetically,
/// but measurably with computed signal values.
///
/// This module confirms one such case from internal synthetic scenario data and
/// labels it `mixed_regime_confirmed_internal`. It does NOT claim engine-native
/// confirmation — that remains pending until a real engine capture is provided with
/// an appropriate scene.
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::DemoConfig;
use crate::error::Result;
use crate::external::build_owned_inputs_from_sequence;
use crate::scene::{generate_sequence_for_definition, scenario_by_id, ScenarioId};
use crate::taa::run_fixed_alpha_baseline;

pub const MIXED_REGIME_CONFIRMED_INTERNAL: &str = "mixed_regime_confirmed_internal";

/// Generate the mixed-regime confirmation report.
/// Returns the path to `{output_dir}/mixed_regime_confirmation_report.md`.
pub fn confirm_mixed_regime(config: &DemoConfig, output_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)?;

    // Use NoisyReprojection: this scenario deliberately combines
    // (a) thin-structure disocclusion events (aliasing pressure) with
    // (b) noisy motion reprojection (variance/noise pressure).
    // We compute both signals from actual pixel data.
    let scenario_id = ScenarioId::NoisyReprojection;
    let definition = scenario_by_id(&config.scene, scenario_id).ok_or_else(|| {
        crate::error::Error::Message(
            "mixed-regime scenario (noisy_reprojection) not found in suite".to_string(),
        )
    })?;

    let sequence = generate_sequence_for_definition(&definition);
    let frame_index = definition
        .onset_frame
        .min(sequence.frames.len().saturating_sub(1))
        .max(1);

    let fixed_alpha = run_fixed_alpha_baseline(&sequence, config.baseline.fixed_alpha);
    let previous_history = fixed_alpha.taa.resolved_frames.get(frame_index - 1);
    let inputs = build_owned_inputs_from_sequence(&sequence, frame_index, previous_history)?;

    let w = inputs.width();
    let h = inputs.height();
    let n = w * h;

    // ── Signal 1: Aliasing pressure ──────────────────────────────────────────
    // Measure spatial gradient magnitude in the current color frame.
    // High gradient magnitude in ROI pixels = high aliasing pressure.
    let gradient_magnitudes: Vec<f32> = compute_gradient_magnitudes(&inputs.current_color, w, h);
    let roi_pixels: Vec<usize> = sequence
        .target_mask
        .iter()
        .enumerate()
        .filter(|(_, &m)| m)
        .map(|(i, _)| i)
        .collect();
    let roi_count = roi_pixels.len();

    let roi_mean_gradient = if roi_count > 0 {
        roi_pixels
            .iter()
            .map(|&i| gradient_magnitudes[i])
            .sum::<f32>()
            / roi_count as f32
    } else {
        0.0
    };
    let all_mean_gradient: f32 = gradient_magnitudes.iter().sum::<f32>() / n as f32;
    // ROI enrichment ratio: how much higher is gradient in ROI vs background
    let aliasing_enrichment = if all_mean_gradient > 1e-6 {
        roi_mean_gradient / all_mean_gradient
    } else {
        1.0
    };
    // Aliasing is confirmed if ROI gradient is meaningfully elevated above background
    let aliasing_threshold = 1.5_f32;
    let aliasing_confirmed = aliasing_enrichment >= aliasing_threshold;

    // ── Signal 2: Variance/noise pressure ────────────────────────────────────
    // Measure temporal variance: difference between current and reprojected history.
    // High temporal variance = high noise/instability pressure.
    let temporal_variance: Vec<f32> =
        compute_temporal_variance(&inputs.current_color, &inputs.reprojected_history, n);

    let roi_mean_variance = if roi_count > 0 {
        roi_pixels
            .iter()
            .map(|&i| temporal_variance[i])
            .sum::<f32>()
            / roi_count as f32
    } else {
        0.0
    };
    let all_mean_variance: f32 = temporal_variance.iter().sum::<f32>() / n as f32;
    let variance_enrichment = if all_mean_variance > 1e-6 {
        roi_mean_variance / all_mean_variance
    } else {
        1.0
    };
    // Variance is confirmed if ROI variance is meaningfully elevated
    let variance_threshold = 1.3_f32;
    let variance_confirmed = variance_enrichment >= variance_threshold;

    // Additionally check motion vector noise: compute mv magnitude variance
    let mv_magnitudes: Vec<f32> = inputs
        .motion_vectors
        .iter()
        .map(|mv| (mv.to_prev_x * mv.to_prev_x + mv.to_prev_y * mv.to_prev_y).sqrt())
        .collect();
    let roi_mean_mv = if roi_count > 0 {
        roi_pixels.iter().map(|&i| mv_magnitudes[i]).sum::<f32>() / roi_count as f32
    } else {
        0.0
    };
    let all_mean_mv: f32 = mv_magnitudes.iter().sum::<f32>() / n as f32;
    let mv_enrichment = if all_mean_mv > 1e-6 {
        roi_mean_mv / all_mean_mv
    } else {
        1.0
    };

    // Both signals must be confirmed for mixed_regime_confirmed_internal
    let mixed_regime = aliasing_confirmed && variance_confirmed;

    // Write the report
    let report_path = output_dir.join("mixed_regime_confirmation_report.md");
    write_confirmation_report(
        &report_path,
        mixed_regime,
        scenario_id.as_str(),
        frame_index,
        w,
        h,
        roi_count,
        n,
        roi_mean_gradient,
        all_mean_gradient,
        aliasing_enrichment,
        aliasing_threshold,
        aliasing_confirmed,
        roi_mean_variance,
        all_mean_variance,
        variance_enrichment,
        variance_threshold,
        variance_confirmed,
        roi_mean_mv,
        all_mean_mv,
        mv_enrichment,
    )?;

    Ok(report_path)
}

// ─── Signal computation ───────────────────────────────────────────────────────

fn compute_gradient_magnitudes(frame: &crate::frame::ImageFrame, w: usize, h: usize) -> Vec<f32> {
    let px = |x: usize, y: usize| -> f32 {
        let c = frame.get(x, y);
        (c.r + c.g + c.b) / 3.0
    };
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let cx = x.min(w.saturating_sub(1));
            let cy = y.min(h.saturating_sub(1));
            let dx = if cx + 1 < w { px(cx + 1, cy) - px(cx, cy) } else { 0.0 };
            let dy = if cy + 1 < h { px(cx, cy + 1) - px(cx, cy) } else { 0.0 };
            out[y * w + x] = (dx * dx + dy * dy).sqrt();
        }
    }
    out
}

fn compute_temporal_variance(
    current: &crate::frame::ImageFrame,
    history: &crate::frame::ImageFrame,
    n: usize,
) -> Vec<f32> {
    let w = current.width();
    let h = current.height();
    (0..n)
        .map(|i| {
            let x = i % w;
            let y = i / w;
            if x < w && y < h {
                let c = current.get(x, y);
                let hc = history.get(x, y);
                let dr = c.r - hc.r;
                let dg = c.g - hc.g;
                let db = c.b - hc.b;
                (dr * dr + dg * dg + db * db) / 3.0
            } else {
                0.0
            }
        })
        .collect()
}

// ─── Report writer ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn write_confirmation_report(
    path: &Path,
    mixed_regime: bool,
    scenario_id: &str,
    frame_index: usize,
    w: usize,
    h: usize,
    roi_count: usize,
    total_pixels: usize,
    roi_mean_gradient: f32,
    all_mean_gradient: f32,
    aliasing_enrichment: f32,
    aliasing_threshold: f32,
    aliasing_confirmed: bool,
    roi_mean_variance: f32,
    all_mean_variance: f32,
    variance_enrichment: f32,
    variance_threshold: f32,
    variance_confirmed: bool,
    roi_mean_mv: f32,
    all_mean_mv: f32,
    mv_enrichment: f32,
) -> Result<()> {
    let mut buf = String::new();
    let _ = writeln!(buf, "# Mixed-Regime Confirmation Report\n");

    let regime_label = if mixed_regime {
        MIXED_REGIME_CONFIRMED_INTERNAL
    } else {
        "mixed_regime_not_confirmed_internal"
    };
    let _ = writeln!(buf, "**mixed_regime_status:** {regime_label}\n");
    let _ = writeln!(buf, "**source:** internal synthetic scenario (`{scenario_id}`)");
    let _ = writeln!(buf, "**frame_index:** {frame_index}");
    let _ = writeln!(buf, "**resolution:** {w}×{h}");
    let _ = writeln!(buf, "**roi_pixels:** {roi_count} / {total_pixels} ({:.1}%)\n", roi_count as f64 / total_pixels as f64 * 100.0);

    let _ = writeln!(buf, "## 1. Source of Case\n");
    let _ = writeln!(buf,
        "Scenario: **{scenario_id}** (`NoisyReprojection`). This scenario deliberately combines:\n\
        - Thin-structure disocclusion events at frame onset (aliasing pressure)\n\
        - Noisy motion reprojection that creates temporal frame-to-frame instability (variance/noise pressure)\n\
        \n\
        Both signals are computed from actual pixel data at frame index {frame_index} — not inferred \
        or claimed without evidence.\n"
    );

    let _ = writeln!(buf, "## 2. Why Aliasing Pressure Is Present\n");
    let _ = writeln!(buf, "**Signal:** spatial gradient magnitude in current color frame within ROI\n");
    let _ = writeln!(buf, "| Metric | Value |");
    let _ = writeln!(buf, "|--------|-------|");
    let _ = writeln!(buf, "| ROI mean gradient magnitude | {roi_mean_gradient:.5} |");
    let _ = writeln!(buf, "| Background mean gradient magnitude | {all_mean_gradient:.5} |");
    let _ = writeln!(buf, "| ROI enrichment ratio | {aliasing_enrichment:.3}× |");
    let _ = writeln!(buf, "| Threshold for confirmation | {aliasing_threshold:.1}× |");
    let _ = writeln!(buf, "| **Aliasing confirmed** | **{aliasing_confirmed}** |\n");
    let _ = writeln!(buf,
        "Interpretation: ROI pixels exhibit {aliasing_enrichment:.2}× higher spatial frequency \
        (gradient magnitude) than non-ROI pixels. This reflects the thin-structure disocclusion \
        event where high-frequency edge detail is revealed at the onset frame. A ratio \u{2265}{aliasing_threshold:.1}\u{00D7} \
        is classified as material aliasing pressure.\n"
    );

    let _ = writeln!(buf, "## 3. Why Variance/Noise Pressure Is Present\n");
    let _ = writeln!(buf,
        "**Signal:** temporal variance (mean squared difference between current frame and reprojected history)\n"
    );
    let _ = writeln!(buf, "| Metric | Value |");
    let _ = writeln!(buf, "|--------|-------|");
    let _ = writeln!(buf, "| ROI mean temporal variance | {roi_mean_variance:.5} |");
    let _ = writeln!(buf, "| Background mean temporal variance | {all_mean_variance:.5} |");
    let _ = writeln!(buf, "| ROI enrichment ratio | {variance_enrichment:.3}× |");
    let _ = writeln!(buf, "| Threshold for confirmation | {variance_threshold:.1}× |");
    let _ = writeln!(buf, "| **Variance confirmed** | **{variance_confirmed}** |\n");
    let _ = writeln!(buf, "**Motion vector enrichment (supporting):**");
    let _ = writeln!(buf, "| Metric | Value |");
    let _ = writeln!(buf, "|--------|-------|");
    let _ = writeln!(buf, "| ROI mean MV magnitude | {roi_mean_mv:.4} px |");
    let _ = writeln!(buf, "| Background mean MV magnitude | {all_mean_mv:.4} px |");
    let _ = writeln!(buf, "| ROI MV enrichment ratio | {mv_enrichment:.3}× |\n");
    let _ = writeln!(
        buf,
        "Interpretation: ROI pixels exhibit {variance_enrichment:.2}× higher temporal frame-to-frame \
        instability than non-ROI pixels. This reflects the noisy reprojection model where motion \
        estimates have added stochastic error at the thin structure boundary, creating material \
        variance/noise pressure co-active with the aliasing pressure above.\n"
    );

    let _ = writeln!(buf, "## 4. Confirmation Classification\n");
    if mixed_regime {
        let _ = writeln!(buf,
            "**Classification: `mixed_regime_confirmed_internal`**\n\
            \n\
            Both aliasing pressure (enrichment {aliasing_enrichment:.2}x >= threshold {aliasing_threshold:.1}x) \
            and variance/noise pressure (enrichment {variance_enrichment:.2}x >= threshold {variance_threshold:.1}x) \
            are materially active in the **same ROI** at the **same frame**. This is not a claim -- \
            it is the direct output of computing both signals from the same pixel set.\n"
        );
    } else {
        let _ = writeln!(buf,
            "**Classification: `mixed_regime_not_confirmed_internal`**\n\
            \n\
            At least one signal did not reach its threshold:\n\
            - Aliasing: enrichment {aliasing_enrichment:.2}x (threshold {aliasing_threshold:.1}x) -> {aliasing_confirmed}\n\
            - Variance: enrichment {variance_enrichment:.2}x (threshold {variance_threshold:.1}x) -> {variance_confirmed}\n\
            \n\
            This should not occur for the `noisy_reprojection` scenario. If it does, the scenario \
            parameters may need review.\n"
        );
    }

    let _ = writeln!(buf, "## 5. Engine-Native Confirmation Status\n");
    let _ = writeln!(buf,
        "**Engine-native mixed-regime: NOT CONFIRMED**\n\
        \n\
        No real engine capture has been provided. The classification above is `internal-only`. \
        A true engine-native mixed-regime case requires a renderer capture with a scene that \
        naturally produces both aliasing and variance pressure in the same ROI (e.g., a thin wire \
        or foliage element under noisy TAA reprojection). Engine-native confirmation remains \
        pending.\n"
    );

    let _ = writeln!(buf, "## 6. What Still Remains Unproven\n");
    let _ = writeln!(buf, "- Mixed-regime on real engine-native data (pending capture + appropriate scene)");
    let _ = writeln!(buf, "- Renderer-specific noise sources (e.g., blue-noise dither patterns) not evaluated");
    let _ = writeln!(buf, "- Sub-pixel jitter interaction with aliasing is not separately quantified\n");

    let _ = writeln!(buf, "## What Is Not Proven\n");
    let _ = writeln!(buf, "- Engine-native mixed-regime confirmation (internal synthetic only)");
    let _ = writeln!(buf, "- Renderer-specific variance sources not evaluated\n");
    let _ = writeln!(buf, "## Remaining Blockers\n");
    let _ = writeln!(buf, "- **EXTERNAL**: Engine-native mixed-regime requires real capture with appropriate scene.");
    let _ = writeln!(buf, "- **INTERNAL** (resolved): Internal confirmation computed from actual signal values.");
    fs::write(path, buf)?;
    Ok(())
}
