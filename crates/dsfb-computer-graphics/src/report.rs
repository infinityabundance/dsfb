use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::config::DemoConfig;
use crate::error::Result;
use crate::metrics::MetricsReport;
use crate::sampling::DemoBMetrics;

pub const EXPERIMENT_SENTENCE: &str =
    "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”";
pub const COST_SENTENCE: &str = "“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”";
pub const COMPATIBILITY_SENTENCE: &str =
    "“The framework is compatible with tiled and asynchronous GPU execution.”";

#[derive(Clone, Debug)]
pub struct CompletionNoteStatus {
    pub only_files_inside_crate_changed: bool,
    pub demo_a_runs_end_to_end: bool,
    pub metrics_generated: bool,
    pub figures_generated: bool,
    pub report_generated: bool,
    pub reviewer_summary_generated: bool,
    pub exact_required_sentences_present: bool,
    pub cargo_fmt_passed: bool,
    pub cargo_clippy_passed: bool,
    pub cargo_test_passed: bool,
    pub no_fabricated_performance_claims: bool,
    pub fully_implemented: Vec<String>,
    pub future_work: Vec<String>,
    pub demo_b_status: String,
}

pub fn write_report(path: &Path, config: &DemoConfig, metrics: &MetricsReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let summary = &metrics.summary;
    let mut markdown = String::new();

    let _ = writeln!(markdown, "# DSFB Computer Graphics Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Overview");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This crate is a bounded transition artifact for temporal reuse supervision. It packages a deterministic scene, a fixed-alpha baseline, a stronger residual-threshold baseline, a DSFB supervisory path, real generated figures, and replayable metrics so a reviewer can evaluate the behavior quickly."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is demonstrated: a deterministic reveal event in which stale temporal history persists on thin geometry for the fixed-alpha baseline, while the DSFB supervisory signal lowers trust, raises the current-frame blend weight, and reduces persistence error."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is not demonstrated: production-optimal tuning, field readiness, GPU benchmarks, or superiority against a full commercial temporal reconstruction stack."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What remains future work: engine integration, broader scenes, measured hardware studies, and larger comparative baselines."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Numeric Demo Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Metric | Fixed-alpha baseline | Residual-threshold baseline | DSFB |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: |");
    let _ = writeln!(
        markdown,
        "| Ghost persistence frames | {} | {} | {} |",
        summary.baseline_ghost_persistence_frames,
        summary.residual_baseline_ghost_persistence_frames,
        summary.dsfb_ghost_persistence_frames
    );
    let _ = writeln!(
        markdown,
        "| Peak ROI error | {:.5} | {:.5} | {:.5} |",
        summary.baseline_peak_roi_error,
        summary.residual_baseline_peak_roi_error,
        summary.dsfb_peak_roi_error
    );
    let _ = writeln!(
        markdown,
        "| Cumulative ROI error | {:.5} | {:.5} | {:.5} |",
        summary.cumulative_persistence_roi_mae_baseline,
        summary.cumulative_persistence_roi_mae_residual_baseline,
        summary.cumulative_persistence_roi_mae_dsfb
    );
    let _ = writeln!(
        markdown,
        "| Average overall MAE | {:.5} | {:.5} | {:.5} |",
        summary.average_overall_mae_baseline,
        summary.average_overall_mae_residual_baseline,
        summary.average_overall_mae_dsfb
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Reveal frame: {}", summary.reveal_frame);
    let _ = writeln!(markdown, "- Trust-drop frame: {}", summary.trust_drop_frame);
    let _ = writeln!(
        markdown,
        "- Trust-minimum frame: {}",
        summary.trust_min_frame
    );
    let _ = writeln!(
        markdown,
        "- Residual-baseline response frame: {}",
        summary.residual_baseline_response_frame
    );
    let _ = writeln!(
        markdown,
        "- Trust/error correlation at reveal: {:.4}",
        summary.trust_error_correlation
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", summary.primary_behavioral_result);
    if let Some(result) = &summary.secondary_behavioral_result {
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "{result}");
    }
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Canonical Scene");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The canonical sequence contains a moving foreground object, a deterministic disocclusion event, a one-pixel vertical structure, a one-pixel diagonal structure, and a persistence ROI derived from the revealed thin pixels."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Resolution: {} x {}",
        config.scene.width, config.scene.height
    );
    let _ = writeln!(markdown, "- Frame count: {}", config.scene.frame_count);
    let _ = writeln!(
        markdown,
        "- Persistence mask pixels: {}",
        summary.persistence_mask_pixels
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## DSFB State Exports");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The crate exports first-class DSFB state rather than only the final gated image. Under `generated/frames/`, the run writes residual, trust, alpha, intervention, residual-proxy, visibility-proxy, motion-edge-proxy, thin-proxy, and simplified structural-state images for every frame."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The simplified structural-state field is crate-scoped and intentionally honest rather than universal. It uses the labels `nominal`, `disocclusion-like`, `unstable-history`, and `motion-edge` as a bounded grammar for this artifact."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## DSFB Integration into Temporal Reuse");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Baseline temporal blend equation with a fixed blend weight:"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "```text");
    let _ = writeln!(
        markdown,
        "C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{{t-1}}_reproj(u)"
    );
    let _ = writeln!(markdown, "```");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "DSFB trust-modulated blend equation with the same underlying estimator:"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "```text");
    let _ = writeln!(
        markdown,
        "C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{{t-1}}_reproj(u)"
    );
    let _ = writeln!(
        markdown,
        "alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))"
    );
    let _ = writeln!(markdown, "```");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "High trust means the supervisory layer keeps the blend close to the history-preserving setting. Low trust means the supervisory layer increases the current-frame weight so revealed or unstable regions flush stale history sooner."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The underlying estimator is unchanged. The crate demonstrates a supervisory blend modulation layer that can sit on top of an existing temporal reuse path without replacing the underlying renderer or estimator."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Figures");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- `fig_system_diagram.svg`: Inputs → Residuals → Proxies → Grammar → Trust → Intervention."
    );
    let _ = writeln!(
        markdown,
        "- `fig_trust_map.svg`: trust overlay on the actual reveal frame with disocclusion and motion-edge highlights."
    );
    let _ = writeln!(
        markdown,
        "- `fig_before_after.svg`: baseline fixed-alpha output versus DSFB on the same comparison frame and ROI."
    );
    let _ = writeln!(
        markdown,
        "- `fig_trust_vs_error.svg`: frame index on the x-axis, ROI error on the left y-axis, DSFB ROI trust on the right y-axis."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## GPU Implementation Considerations");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "### Execution Model");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The supervisory path is organized around local per-pixel operations and can also be lifted to a per-tile realization. Residuals, proxies, trust, and blend modulation all depend on local evidence plus bounded temporal history, which makes the design a plausible async-compute candidate in a larger frame graph."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "### Memory Layout");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Residual buffer: scalar discrepancy between current and reprojected history."
    );
    let _ = writeln!(
        markdown,
        "- Proxy buffer: residual, visibility, motion-edge, and thin-structure cues."
    );
    let _ = writeln!(
        markdown,
        "- Trust buffer: scalar supervisory field used to derive alpha modulation."
    );
    let _ = writeln!(
        markdown,
        "- Optional history and tile-summary buffers: bounded temporal memory plus coarse reduction outputs."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "### Optimization Strategies");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- half resolution trust");
    let _ = writeln!(markdown, "- tile aggregation");
    let _ = writeln!(markdown, "- temporal reuse of proxy");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "### Cost Table");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Operation group | Per-pixel / per-tile character | Memory footprint class | Reduction strategy |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | --- |");
    let _ = writeln!(
        markdown,
        "| Residual evaluation | per-pixel local arithmetic | one scalar buffer | half resolution trust when full precision is unnecessary |"
    );
    let _ = writeln!(
        markdown,
        "| Proxy synthesis | per-pixel with optional neighborhood lookups | packed proxy channels | reuse and compact proxy packing |"
    );
    let _ = writeln!(
        markdown,
        "| Grammar and trust update | per-pixel or per-tile aggregation | one trust buffer plus optional tile summaries | tile aggregation |"
    );
    let _ = writeln!(
        markdown,
        "| Blend modulation | per-pixel scalar modulation | no extra color history beyond temporal reuse itself | fuse with existing resolve pass |"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COST_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COMPATIBILITY_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "All cost discussion in this crate is architectural or approximate. It is not presented as measured production benchmarking."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Mission and Transition Relevance");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This artifact is relevant to reliability and assurance in visual pipelines because it surfaces replayable residual, proxy, trust, and intervention evidence rather than only a final image. That supports early detection of estimator failure modes, bounded auditability, and after-action review for safety-adjacent or mission-adjacent visual systems."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The crate is a synthetic feasibility artifact, not a fielded mission system. It illustrates a bounded feasibility demonstration for supervisory evidence in temporal reuse rather than deployment readiness."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Product Framing and Integration Surfaces");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "In product terms, this implementation demonstrates the shape of an attachable supervisory trust layer: a middleware-style surface that can modulate temporal reuse, emit traces, and expose a routing signal for adaptive compute without replacing the base estimator."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Surface | Current crate coverage | Future extension |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- |");
    let _ = writeln!(
        markdown,
        "| TAA / temporal reuse | implemented in Demo A | extend to engine integration and richer reprojection paths |"
    );
    let _ = writeln!(
        markdown,
        "| adaptive sampling / SAR | implemented as bounded Demo B fixed-budget reveal-frame study | extend to temporal policy and broader sampling controllers |"
    );
    let _ = writeln!(
        markdown,
        "| logging / QA | implemented through generated metrics, figures, and reports | extend to engine traces, fleet replay, and automated regression checks |"
    );
    let _ = writeln!(
        markdown,
        "| adaptive compute routing | partially illustrated through trust and intervention fields | extend to budget schedulers and cross-pass policy |"
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## What this crate does not claim");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- It does not claim funding, licensing, or instant transition outcomes."
    );
    let _ = writeln!(
        markdown,
        "- It does not claim production-optimal TAA or temporal reconstruction."
    );
    let _ = writeln!(
        markdown,
        "- It does not claim measured GPU timings or hardware-specific performance wins."
    );
    let _ = writeln!(
        markdown,
        "- It does not claim readiness for mission deployment or safety certification."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Limitations");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The scene is deterministic and synthetic rather than photoreal or field captured."
    );
    let _ = writeln!(
        markdown,
        "- The residual-threshold baseline is stronger than fixed alpha but still not a full commercial anti-ghosting stack."
    );
    let _ = writeln!(
        markdown,
        "- The structural grammar is intentionally simplified and scoped to this crate."
    );
    let _ = writeln!(
        markdown,
        "- Demo B is bounded to a reveal-frame fixed-budget study."
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Future Work");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Extend the crate to richer scenes and engine-connected reprojection data while preserving replayability."
    );
    let _ = writeln!(
        markdown,
        "- Add additional comparative baselines such as variance gating, neighborhood clipping, or learned confidence predictors."
    );
    let _ = writeln!(
        markdown,
        "- Measure an actual GPU implementation and label it explicitly as measured hardware data."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_reviewer_summary(
    path: &Path,
    config: &DemoConfig,
    metrics: &MetricsReport,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let summary = &metrics.summary;
    let mut markdown = String::new();

    let _ = writeln!(markdown, "# Reviewer Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This crate packages a deterministic temporal reuse study for DSFB supervision. Demo A uses a moving occluder, a disocclusion event, thin worst-case geometry, fixed-alpha TAA, a residual-threshold baseline, and a DSFB path that only changes the supervisory blend-control layer."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", summary.primary_behavioral_result);
    if let Some(result) = &summary.secondary_behavioral_result {
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "{result}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "DSFB plugs into temporal reuse through blend modulation rather than estimator replacement:"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "```text");
    let _ = writeln!(
        markdown,
        "C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{{t-1}}_reproj(u)"
    );
    let _ = writeln!(
        markdown,
        "alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))"
    );
    let _ = writeln!(markdown, "```");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Estimated systems footprint: local per-pixel or per-tile residual/proxy/trust operations, bounded temporal memory, and a linear-with-pixel-count supervisory pass that can be evaluated at reduced resolution."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Transition relevance: replayable supervisory evidence, visible failure-response timing, and an attachable middleware shape for temporal reuse, QA logging, and adaptive compute routing."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Commercial relevance: the current crate is not an SDK, but it demonstrates the product shape of a supervisory trust layer that could attach to engine temporal reuse, reconstruction, or traceability workflows."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Default run details: {} frames at {} x {}, reveal frame {}, trust-drop frame {}, residual-baseline response frame {}.",
        config.scene.frame_count,
        config.scene.width,
        config.scene.height,
        summary.reveal_frame,
        summary.trust_drop_frame,
        summary.residual_baseline_response_frame
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_completion_note(path: &Path, status: &CompletionNoteStatus) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Completion Note");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Boundary compliance note: this artifact is constrained to `crates/dsfb-computer-graphics`, and the completion note is intended to confirm that crate-local boundary."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Checklist");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- {} Only files inside crates/dsfb-computer-graphics were changed",
        checkbox(status.only_files_inside_crate_changed)
    );
    let _ = writeln!(
        markdown,
        "- {} Demo A runs end-to-end",
        checkbox(status.demo_a_runs_end_to_end)
    );
    let _ = writeln!(
        markdown,
        "- {} Metrics are generated",
        checkbox(status.metrics_generated)
    );
    let _ = writeln!(
        markdown,
        "- {} Figures are generated",
        checkbox(status.figures_generated)
    );
    let _ = writeln!(
        markdown,
        "- {} Report is generated",
        checkbox(status.report_generated)
    );
    let _ = writeln!(
        markdown,
        "- {} Reviewer summary is generated",
        checkbox(status.reviewer_summary_generated)
    );
    let _ = writeln!(
        markdown,
        "- {} Exact required sentences are present",
        checkbox(status.exact_required_sentences_present)
    );
    let _ = writeln!(
        markdown,
        "- {} cargo fmt passed",
        checkbox(status.cargo_fmt_passed)
    );
    let _ = writeln!(
        markdown,
        "- {} cargo clippy passed",
        checkbox(status.cargo_clippy_passed)
    );
    let _ = writeln!(
        markdown,
        "- {} cargo test passed",
        checkbox(status.cargo_test_passed)
    );
    let _ = writeln!(
        markdown,
        "- {} No fabricated performance claims were made",
        checkbox(status.no_fabricated_performance_claims)
    );
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Fully Implemented");
    let _ = writeln!(markdown);
    for item in &status.fully_implemented {
        let _ = writeln!(markdown, "- {item}");
    }
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Intentionally Left Future Work");
    let _ = writeln!(markdown);
    for item in &status.future_work {
        let _ = writeln!(markdown, "- {item}");
    }
    let _ = writeln!(markdown);

    let _ = writeln!(markdown, "## Demo B Status");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", status.demo_b_status);

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
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
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
        "- ROI mean guided spp: {:.2}",
        metrics.roi_mean_guided_spp
    );
    let _ = writeln!(
        markdown,
        "- Trust ROI mean carried from Demo A: {:.4}",
        metrics.trust_roi_mean
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "In this bounded study, the DSFB-guided allocation is intended to show how a trust field could steer fixed-budget sampling rather than prove an optimal adaptive-sampling policy."
    );

    fs::write(path, markdown)?;
    Ok(())
}

fn checkbox(value: bool) -> &'static str {
    if value {
        "[x]"
    } else {
        "[ ]"
    }
}
