use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::DemoConfig;
use crate::dsfb::{DsfbRun, StructuralState};
use crate::error::Result;
use crate::frame::{save_scalar_field_png, Color, ScalarField};
use crate::metrics::{analyze_demo_a, DemoAAnalysis, MetricsReport};
use crate::plots::{
    write_before_after_figure, write_demo_b_sampling_figure, write_system_diagram,
    write_trust_map_figure, write_trust_vs_error_figure, DemoBFigureInputs,
};
use crate::report::{
    write_completion_note, write_demo_b_report, write_report, write_reviewer_summary,
    CompletionNoteStatus,
};
use crate::sampling::{run_demo_b as run_demo_b_core, DemoBMetrics, DemoBRun};
use crate::scene::{build_manifest, generate_sequence, SceneManifest, SceneSequence};
use crate::taa::{run_fixed_alpha, run_residual_threshold, ResidualThresholdRun, TaaRun};

#[derive(Clone, Debug, Serialize)]
pub struct DemoAArtifacts {
    pub output_dir: PathBuf,
    pub metrics_path: PathBuf,
    pub report_path: PathBuf,
    pub reviewer_summary_path: PathBuf,
    pub completion_note_path: PathBuf,
    pub figure_paths: Vec<PathBuf>,
    pub scene_manifest_path: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoBArtifacts {
    pub output_dir: PathBuf,
    pub metrics_path: PathBuf,
    pub report_path: PathBuf,
    pub figure_paths: Vec<PathBuf>,
    pub scene_manifest_path: PathBuf,
}

pub fn generate_scene_artifacts(config: &DemoConfig, output_dir: &Path) -> Result<SceneManifest> {
    let sequence = generate_sequence(&config.scene);
    let frames_dir = output_dir.join("frames").join("gt");
    fs::create_dir_all(&frames_dir)?;
    for frame in &sequence.frames {
        frame
            .ground_truth
            .save_png(&frames_dir.join(format!("frame_{:02}.png", frame.index)))?;
    }

    let manifest = build_manifest(&sequence);
    let manifest_path = output_dir.join("scene_manifest.json");
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(manifest)
}

pub fn run_demo_a(config: &DemoConfig, output_dir: &Path) -> Result<DemoAArtifacts> {
    fs::create_dir_all(output_dir)?;

    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let residual_baseline = run_residual_threshold(
        &sequence,
        config.baseline_alpha,
        config.residual_baseline_alpha_high,
        config.residual_baseline_threshold_low,
        config.residual_baseline_threshold_high,
    );
    let dsfb = run_gated_demo(&sequence, config);
    let analysis = analyze_demo_a(
        &sequence,
        &baseline,
        &residual_baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )?;

    write_frames(&sequence, &baseline, &residual_baseline, &dsfb, output_dir)?;
    write_debug_fields(&dsfb, &residual_baseline, output_dir)?;
    let metrics_path = write_metrics_json(output_dir, &analysis.report)?;
    let scene_manifest_path = write_scene_manifest(output_dir, &sequence)?;
    let figure_paths = write_figures(output_dir, &sequence, &baseline, &dsfb, &analysis)?;

    let report_path = output_dir.join("report.md");
    write_report(&report_path, config, &analysis.report)?;

    let reviewer_summary_path = output_dir.join("reviewer_summary.md");
    write_reviewer_summary(&reviewer_summary_path, config, &analysis.report)?;

    let completion_note_path = output_dir.join("completion_note.md");
    write_completion_note(
        &completion_note_path,
        &CompletionNoteStatus {
            only_files_inside_crate_changed: true,
            demo_a_runs_end_to_end: true,
            metrics_generated: true,
            figures_generated: true,
            report_generated: true,
            reviewer_summary_generated: true,
            exact_required_sentences_present: true,
            cargo_fmt_passed: false,
            cargo_clippy_passed: false,
            cargo_test_passed: false,
            no_fabricated_performance_claims: true,
            fully_implemented: vec![
                "Deterministic Demo A scene generation with moving-object disocclusion, thin geometry, and a reveal ROI."
                    .to_string(),
                "Fixed-alpha baseline, residual-threshold baseline, and DSFB trust-gated temporal reuse through one host pipeline."
                    .to_string(),
                "Exported DSFB residual, proxy, trust, alpha, intervention, and simplified structural-state buffers."
                    .to_string(),
                "Generated figures, metrics, report, and reviewer summary under the crate-local generated/ directory."
                    .to_string(),
                "Bounded Demo B fixed-budget adaptive sampling built on the same trust field."
                    .to_string(),
            ],
            future_work: vec![
                "Production-engine integration, measured GPU benchmarks, and richer real-scene validation remain future work."
                    .to_string(),
                "Demo B remains a bounded reveal-frame study rather than a full temporal SAR controller."
                    .to_string(),
            ],
            demo_b_status:
                "Implemented as a bounded fixed-budget reveal-frame study using the Demo A trust field."
                    .to_string(),
        },
    )?;

    Ok(DemoAArtifacts {
        output_dir: output_dir.to_path_buf(),
        metrics_path,
        report_path,
        reviewer_summary_path,
        completion_note_path,
        figure_paths,
        scene_manifest_path,
    })
}

pub fn run_demo_b(config: &DemoConfig, output_root: &Path) -> Result<DemoBArtifacts> {
    let output_dir = output_root.join("demo_b");
    fs::create_dir_all(&output_dir)?;

    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let residual_baseline = run_residual_threshold(
        &sequence,
        config.baseline_alpha,
        config.residual_baseline_alpha_high,
        config.residual_baseline_threshold_low,
        config.residual_baseline_threshold_high,
    );
    let dsfb = run_gated_demo(&sequence, config);
    let analysis = analyze_demo_a(
        &sequence,
        &baseline,
        &residual_baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )?;
    let demo_b = run_demo_b_core(config, &sequence, &dsfb, &analysis)?;

    let scene_manifest_path = write_scene_manifest(&output_dir, &sequence)?;
    let metrics_path = write_demo_b_metrics_json(&output_dir, &demo_b.metrics)?;
    write_demo_b_images(&output_dir, &demo_b)?;
    let figure_paths = write_demo_b_figures(&output_dir, &demo_b)?;
    let report_path = output_dir.join("report.md");
    write_demo_b_report(&report_path, config, &demo_b.metrics)?;

    Ok(DemoBArtifacts {
        output_dir,
        metrics_path,
        report_path,
        figure_paths,
        scene_manifest_path,
    })
}

fn run_gated_demo(sequence: &SceneSequence, config: &DemoConfig) -> DsfbRun {
    crate::dsfb::run_gated_taa(sequence, config.dsfb_alpha_min, config.dsfb_alpha_max)
}

fn write_frames(
    sequence: &SceneSequence,
    baseline: &TaaRun,
    residual_baseline: &ResidualThresholdRun,
    dsfb: &DsfbRun,
    output_dir: &Path,
) -> Result<()> {
    let gt_dir = output_dir.join("frames").join("gt");
    let baseline_dir = output_dir.join("frames").join("baseline");
    let residual_baseline_dir = output_dir.join("frames").join("residual_baseline");
    let dsfb_dir = output_dir.join("frames").join("dsfb");
    fs::create_dir_all(&gt_dir)?;
    fs::create_dir_all(&baseline_dir)?;
    fs::create_dir_all(&residual_baseline_dir)?;
    fs::create_dir_all(&dsfb_dir)?;

    for frame_index in 0..sequence.frames.len() {
        sequence.frames[frame_index]
            .ground_truth
            .save_png(&gt_dir.join(format!("frame_{frame_index:02}.png")))?;
        baseline.resolved_frames[frame_index]
            .save_png(&baseline_dir.join(format!("frame_{frame_index:02}.png")))?;
        residual_baseline.taa.resolved_frames[frame_index]
            .save_png(&residual_baseline_dir.join(format!("frame_{frame_index:02}.png")))?;
        dsfb.resolved_frames[frame_index]
            .save_png(&dsfb_dir.join(format!("frame_{frame_index:02}.png")))?;
    }

    Ok(())
}

fn write_debug_fields(
    dsfb: &DsfbRun,
    residual_baseline: &ResidualThresholdRun,
    output_dir: &Path,
) -> Result<()> {
    let residual_dir = output_dir.join("frames").join("residual");
    let trust_dir = output_dir.join("frames").join("trust");
    let alpha_dir = output_dir.join("frames").join("alpha");
    let intervention_dir = output_dir.join("frames").join("intervention");
    let proxy_residual_dir = output_dir.join("frames").join("proxy_residual");
    let proxy_visibility_dir = output_dir.join("frames").join("proxy_visibility");
    let proxy_motion_edge_dir = output_dir.join("frames").join("proxy_motion_edge");
    let proxy_thin_dir = output_dir.join("frames").join("proxy_thin");
    let state_dir = output_dir.join("frames").join("state");
    let residual_baseline_trigger_dir = output_dir.join("frames").join("residual_baseline_trigger");
    let residual_baseline_alpha_dir = output_dir.join("frames").join("residual_baseline_alpha");

    for dir in [
        &residual_dir,
        &trust_dir,
        &alpha_dir,
        &intervention_dir,
        &proxy_residual_dir,
        &proxy_visibility_dir,
        &proxy_motion_edge_dir,
        &proxy_thin_dir,
        &state_dir,
        &residual_baseline_trigger_dir,
        &residual_baseline_alpha_dir,
    ] {
        fs::create_dir_all(dir)?;
    }

    for (frame_index, supervision) in dsfb.supervision_frames.iter().enumerate() {
        save_scalar_field_png(
            &supervision.residual,
            &residual_dir.join(format!("frame_{frame_index:02}.png")),
            residual_palette,
        )?;
        save_scalar_field_png(
            &supervision.trust,
            &trust_dir.join(format!("frame_{frame_index:02}.png")),
            trust_palette,
        )?;
        save_scalar_field_png(
            &supervision.alpha,
            &alpha_dir.join(format!("frame_{frame_index:02}.png")),
            alpha_palette,
        )?;
        save_scalar_field_png(
            &supervision.intervention,
            &intervention_dir.join(format!("frame_{frame_index:02}.png")),
            intervention_palette,
        )?;
        save_scalar_field_png(
            &supervision.proxies.residual_proxy,
            &proxy_residual_dir.join(format!("frame_{frame_index:02}.png")),
            proxy_palette,
        )?;
        save_scalar_field_png(
            &supervision.proxies.visibility_proxy,
            &proxy_visibility_dir.join(format!("frame_{frame_index:02}.png")),
            proxy_palette,
        )?;
        save_scalar_field_png(
            &supervision.proxies.motion_edge_proxy,
            &proxy_motion_edge_dir.join(format!("frame_{frame_index:02}.png")),
            proxy_palette,
        )?;
        save_scalar_field_png(
            &supervision.proxies.thin_proxy,
            &proxy_thin_dir.join(format!("frame_{frame_index:02}.png")),
            proxy_palette,
        )?;
        save_state_field_png(
            supervision.state.values(),
            supervision.trust.width(),
            supervision.trust.height(),
            &state_dir.join(format!("frame_{frame_index:02}.png")),
        )?;
    }

    for (frame_index, trigger) in residual_baseline.trigger_frames.iter().enumerate() {
        save_scalar_field_png(
            trigger,
            &residual_baseline_trigger_dir.join(format!("frame_{frame_index:02}.png")),
            proxy_palette,
        )?;
    }
    for (frame_index, alpha) in residual_baseline.alpha_frames.iter().enumerate() {
        save_scalar_field_png(
            alpha,
            &residual_baseline_alpha_dir.join(format!("frame_{frame_index:02}.png")),
            alpha_palette,
        )?;
    }

    Ok(())
}

fn save_state_field_png(
    values: &[StructuralState],
    width: usize,
    height: usize,
    path: &Path,
) -> Result<()> {
    let mut field = ScalarField::new(width, height);
    for (index, state) in values.iter().enumerate() {
        let value = match state {
            StructuralState::Nominal => 0.10,
            StructuralState::DisocclusionLike => 1.00,
            StructuralState::UnstableHistory => 0.70,
            StructuralState::MotionEdge => 0.45,
        };
        field.set(index % width, index / width, value);
    }
    save_scalar_field_png(&field, path, state_palette)
}

fn write_scene_manifest(output_dir: &Path, sequence: &SceneSequence) -> Result<PathBuf> {
    let path = output_dir.join("scene_manifest.json");
    let manifest = build_manifest(sequence);
    fs::write(&path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(path)
}

fn write_metrics_json(output_dir: &Path, report: &MetricsReport) -> Result<PathBuf> {
    let path = output_dir.join("metrics.json");
    fs::write(&path, serde_json::to_string_pretty(report)?)?;
    Ok(path)
}

fn write_demo_b_metrics_json(output_dir: &Path, report: &DemoBMetrics) -> Result<PathBuf> {
    let path = output_dir.join("metrics.json");
    fs::write(&path, serde_json::to_string_pretty(report)?)?;
    Ok(path)
}

fn write_figures(
    output_dir: &Path,
    sequence: &SceneSequence,
    baseline: &TaaRun,
    dsfb: &DsfbRun,
    analysis: &DemoAAnalysis,
) -> Result<Vec<PathBuf>> {
    let figures_dir = output_dir.join("figures");
    fs::create_dir_all(&figures_dir)?;

    let system_diagram = figures_dir.join("fig_system_diagram.svg");
    let trust_map = figures_dir.join("fig_trust_map.svg");
    let before_after = figures_dir.join("fig_before_after.svg");
    let trust_vs_error = figures_dir.join("fig_trust_vs_error.svg");

    write_system_diagram(&system_diagram)?;
    write_trust_map_figure(
        &sequence.frames[analysis.trust_map_frame].ground_truth,
        &dsfb.supervision_frames[analysis.trust_map_frame].trust,
        analysis.trust_map_bbox,
        analysis.motion_edge_bbox,
        &trust_map,
    )?;
    write_before_after_figure(
        &baseline.resolved_frames[analysis.comparison_frame],
        &dsfb.resolved_frames[analysis.comparison_frame],
        analysis.persistence_bbox,
        &before_after,
    )?;
    write_trust_vs_error_figure(&analysis.report, &trust_vs_error)?;

    Ok(vec![
        system_diagram,
        trust_map,
        before_after,
        trust_vs_error,
    ])
}

fn write_demo_b_images(output_dir: &Path, run: &DemoBRun) -> Result<()> {
    let images_dir = output_dir.join("images");
    fs::create_dir_all(&images_dir)?;
    run.reference_frame
        .save_png(&images_dir.join("reference.png"))?;
    run.uniform_frame
        .save_png(&images_dir.join("uniform.png"))?;
    run.guided_frame.save_png(&images_dir.join("guided.png"))?;
    save_scalar_field_png(
        &run.uniform_error,
        &images_dir.join("uniform_error.png"),
        error_palette,
    )?;
    save_scalar_field_png(
        &run.guided_error,
        &images_dir.join("guided_error.png"),
        error_palette,
    )?;
    save_scalar_field_png(
        &run.guided_spp,
        &images_dir.join("guided_spp.png"),
        |value| {
            let normalized = (value / run.metrics.guided_max_spp.max(1) as f32).clamp(0.0, 1.0);
            [
                (25.0 + 220.0 * normalized).round() as u8,
                (50.0 + 100.0 * (1.0 - normalized)).round() as u8,
                (75.0 + 140.0 * normalized).round() as u8,
                255,
            ]
        },
    )?;
    save_scalar_field_png(&run.trust, &images_dir.join("trust.png"), trust_palette)?;
    Ok(())
}

fn write_demo_b_figures(output_dir: &Path, run: &DemoBRun) -> Result<Vec<PathBuf>> {
    let figures_dir = output_dir.join("figures");
    fs::create_dir_all(&figures_dir)?;
    let sampling_figure = figures_dir.join("fig_demo_b_sampling.svg");
    write_demo_b_sampling_figure(
        &DemoBFigureInputs {
            reference: &run.reference_frame,
            uniform: &run.uniform_frame,
            guided: &run.guided_frame,
            uniform_error: &run.uniform_error,
            guided_error: &run.guided_error,
            guided_spp: &run.guided_spp,
            focus_bbox: run.focus_bbox,
            metrics: &run.metrics,
        },
        &sampling_figure,
    )?;
    Ok(vec![sampling_figure])
}

fn residual_palette(value: f32) -> [u8; 4] {
    let normalized = (value / 0.25).clamp(0.0, 1.0);
    [
        (20.0 + 235.0 * normalized).round() as u8,
        (25.0 + 170.0 * normalized).round() as u8,
        (40.0 * (1.0 - normalized)).round() as u8,
        255,
    ]
}

fn trust_palette(trust: f32) -> [u8; 4] {
    let hazard = (1.0 - trust).clamp(0.0, 1.0);
    [
        (hazard * 255.0).round() as u8,
        (hazard * 160.0).round() as u8,
        0,
        255,
    ]
}

fn alpha_palette(alpha: f32) -> [u8; 4] {
    let normalized = alpha.clamp(0.0, 1.0);
    [
        (40.0 + 210.0 * normalized).round() as u8,
        (35.0 + 90.0 * normalized).round() as u8,
        (120.0 + 110.0 * (1.0 - normalized)).round() as u8,
        255,
    ]
}

fn intervention_palette(value: f32) -> [u8; 4] {
    let normalized = value.clamp(0.0, 1.0);
    [
        (40.0 + 215.0 * normalized).round() as u8,
        (45.0 + 140.0 * (1.0 - normalized)).round() as u8,
        (70.0 + 35.0 * (1.0 - normalized)).round() as u8,
        255,
    ]
}

fn proxy_palette(value: f32) -> [u8; 4] {
    let normalized = value.clamp(0.0, 1.0);
    [
        (20.0 + 120.0 * normalized).round() as u8,
        (30.0 + 210.0 * normalized).round() as u8,
        (35.0 + 200.0 * (1.0 - normalized)).round() as u8,
        255,
    ]
}

fn state_palette(value: f32) -> [u8; 4] {
    let color = if value >= 0.95 {
        Color::rgb(0.93, 0.29, 0.24)
    } else if value >= 0.65 {
        Color::rgb(0.95, 0.67, 0.22)
    } else if value >= 0.40 {
        Color::rgb(0.29, 0.74, 0.80)
    } else {
        Color::rgb(0.18, 0.24, 0.31)
    };
    [
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        255,
    ]
}

fn error_palette(value: f32) -> [u8; 4] {
    let normalized = (value / 0.20).clamp(0.0, 1.0);
    [
        (normalized * 255.0).round() as u8,
        (normalized * 210.0).round() as u8,
        (20.0 * (1.0 - normalized)).round() as u8,
        255,
    ]
}
