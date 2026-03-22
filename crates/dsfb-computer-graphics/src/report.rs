use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::config::DemoConfig;
use crate::error::Result;
use crate::metrics::MetricsReport;
use crate::sampling::DemoBMetrics;

pub fn write_report(path: &Path, config: &DemoConfig, metrics: &MetricsReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let summary = &metrics.summary;

    let _ = writeln!(markdown, "# DSFB Computer Graphics Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Overview");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This crate implements a minimal synthetic experiment for temporal accumulation supervision. The scene is deterministic, bounded, and designed to make thin-geometry disocclusion visually interpretable."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Canonical Scene");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The canonical sequence uses a moving opaque foreground rectangle, a static structured background, a one-pixel vertical element, and a one-pixel diagonal element. The object motion creates disocclusion when it stops after exposing the thin structure."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Resolution: {} x {}",
        config.scene.width, config.scene.height
    );
    let _ = writeln!(markdown, "- Frame count: {}", config.scene.frame_count);
    let _ = writeln!(markdown, "- Reveal frame: {}", summary.reveal_frame);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Baseline");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The baseline is fixed-alpha temporal accumulation with alpha = {:.2}. It uses the same reprojection field as the DSFB-gated path and intentionally omits production heuristics so the control-path difference remains explicit.",
        config.baseline_alpha
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## DSFB-Gated Version");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The DSFB path reuses the same temporal pipeline and only replaces the blending control path. Trust is computed from local residual evidence, visibility change, motion-edge structure, and thin-geometry support. Blend weights follow alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - trust_t(u))."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- alpha_min = {:.2}, alpha_max = {:.2}",
        config.dsfb_alpha_min, config.dsfb_alpha_max
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Metrics");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Average baseline MAE: {:.5}",
        summary.average_overall_mae_baseline
    );
    let _ = writeln!(
        markdown,
        "- Average DSFB MAE: {:.5}",
        summary.average_overall_mae_dsfb
    );
    let _ = writeln!(
        markdown,
        "- Baseline ghost persistence: {} frames",
        summary.baseline_ghost_persistence_frames
    );
    let _ = writeln!(
        markdown,
        "- DSFB ghost persistence: {} frames",
        summary.dsfb_ghost_persistence_frames
    );
    let _ = writeln!(
        markdown,
        "- Cumulative baseline ROI error: {:.5}",
        summary.cumulative_persistence_roi_mae_baseline
    );
    let _ = writeln!(
        markdown,
        "- Cumulative DSFB ROI error: {:.5}",
        summary.cumulative_persistence_roi_mae_dsfb
    );
    let _ = writeln!(
        markdown,
        "- Trust/error correlation at reveal: {:.4}",
        summary.trust_error_correlation
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "In this bounded synthetic setting, the DSFB-gated path demonstrates reduced ghost persistence on the revealed thin structure relative to the baseline."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Figures");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "![Figure 1](figures/fig_system_diagram.svg)");
    let _ = writeln!(
        markdown,
        "Figure 1. System diagram of the supervisory flow. “The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "![Figure 2](figures/fig_trust_map.svg)");
    let _ = writeln!(
        markdown,
        "Figure 2. Trust map overlay on the reveal frame, showing low trust near the disoccluded thin structure and motion edges. “The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "![Figure 3](figures/fig_before_after.svg)");
    let _ = writeln!(
        markdown,
        "Figure 3. Baseline fixed-alpha TAA versus DSFB-gated TAA on the same frame and ROI. “The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "![Figure 4](figures/fig_trust_vs_error.svg)");
    let _ = writeln!(
        markdown,
        "Figure 4. Persistence ROI error and trust over time, illustrating the trust response at failure onset. “The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## GPU Implementation Considerations");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "A future GPU realization can evaluate the supervisory layer per pixel or per tile because the current formulation uses only local residuals, local proxies, and bounded temporal history. The same structure is compatible with async-compute placement inside a broader graphics frame graph."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Concrete buffers include a residual buffer, proxy buffer, trust buffer, and history buffer. Optional debug or reduced-resolution support can use ROI masks and tile summary buffers.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Optimization strategies considered by the crate documentation include half resolution trust, tile aggregation, and temporal reuse of proxy.");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "“The framework is compatible with tiled and asynchronous GPU execution.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Limitations");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This is a minimal synthetic demonstration, not a production renderer."
    );
    let _ = writeln!(markdown, "- The baseline is intentionally simple and does not include broader anti-ghosting heuristics.");
    let _ = writeln!(
        markdown,
        "- The artifact does not measure GPU timings or make optimality claims."
    );
    let _ = writeln!(
        markdown,
        "- Demo B adaptive sampling is bounded to a static reveal-frame fixed-budget study rather than a full temporal SAR system."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Future Work");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Extend Demo B from a static reveal-frame study to a temporal adaptive-sampling controller."
    );
    let _ = writeln!(
        markdown,
        "- Compare against stronger baselines such as variance gating or neighborhood clamping."
    );
    let _ = writeln!(
        markdown,
        "- Extend the synthetic scene to richer depth complexity while retaining determinism."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_demo_b_report(path: &Path, config: &DemoConfig, metrics: &DemoBMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# DSFB Computer Graphics Demo B Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Overview");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo B is a bounded fixed-budget adaptive-sampling study on the canonical reveal frame. It uses the DSFB trust field from Demo A as a supervisory signal for sample redistribution rather than as a temporal blend controller."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Sampling Surface");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The estimator operates on a continuous version of the reveal frame with subpixel thin geometry, sharp foreground-object edges, and the same disocclusion event used by Demo A."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Resolution: {} x {}",
        config.scene.width, config.scene.height
    );
    let _ = writeln!(markdown, "- Reveal frame: {}", metrics.reveal_frame);
    let _ = writeln!(
        markdown,
        "- Reference estimate: {} spp per pixel",
        metrics.reference_spp
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Budget Fairness");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The uniform baseline and the DSFB-guided allocation use the same total sample budget: {} samples.",
        metrics.uniform_total_samples
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The guided policy assigns a minimum of {} spp per pixel, caps at {} spp per pixel, and redistributes the remaining budget according to low-trust hazard weights.",
        metrics.guided_min_spp, metrics.guided_max_spp
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Metrics");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Uniform MAE: {:.5}", metrics.uniform_mae);
    let _ = writeln!(markdown, "- Guided MAE: {:.5}", metrics.guided_mae);
    let _ = writeln!(markdown, "- Uniform RMSE: {:.5}", metrics.uniform_rmse);
    let _ = writeln!(markdown, "- Guided RMSE: {:.5}", metrics.guided_rmse);
    let _ = writeln!(
        markdown,
        "- Uniform ROI MAE: {:.5}",
        metrics.uniform_roi_mae
    );
    let _ = writeln!(markdown, "- Guided ROI MAE: {:.5}", metrics.guided_roi_mae);
    let _ = writeln!(
        markdown,
        "- Uniform ROI RMSE: {:.5}",
        metrics.uniform_roi_rmse
    );
    let _ = writeln!(
        markdown,
        "- Guided ROI RMSE: {:.5}",
        metrics.guided_roi_rmse
    );
    let _ = writeln!(
        markdown,
        "- ROI mean spp: uniform {:.2}, guided {:.2}",
        metrics.uniform_spp as f32, metrics.roi_mean_guided_spp
    );
    let _ = writeln!(markdown, "- Guided max spp: {}", metrics.max_guided_spp);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "In this bounded synthetic setting, DSFB-guided allocation reduces reveal-region sampling error at fixed budget by steering more samples toward the low-trust thin-geometry disocclusion."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Figures");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "![Demo B Figure](figures/fig_demo_b_sampling.svg)"
    );
    let _ = writeln!(
        markdown,
        "Composite view of the reference, uniform estimator, guided estimator, error maps, and guided sample density. “The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Limitations");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This is a static reveal-frame study rather than a temporal adaptive-sampling controller."
    );
    let _ = writeln!(
        markdown,
        "- The sampling surface is analytic and deterministic, not a production path tracer."
    );
    let _ = writeln!(
        markdown,
        "- The result demonstrates budget reallocation behavior rather than optimal sampling policy design."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Future Work");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Extend the allocation policy over time so trust persistence influences future-frame budgets."
    );
    let _ = writeln!(
        markdown,
        "- Compare DSFB guidance against gradient-only or variance-only sampling heuristics."
    );
    let _ = writeln!(
        markdown,
        "- Move from this analytic surface to a stochastic renderer while preserving determinism for artifact generation."
    );

    fs::write(path, markdown)?;
    Ok(())
}
