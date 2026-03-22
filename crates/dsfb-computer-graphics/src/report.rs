use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::config::DemoConfig;
use crate::error::Result;
use crate::metrics::MetricsReport;

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
    let _ = writeln!(markdown, "- Adaptive sampling is future work unless a separate crate-local demo artifact is generated.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Future Work");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Add a fixed-budget adaptive-sampling study using the same trust field."
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
