use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::DemoConfig;
use crate::dsfb::{run_gated_taa, DsfbRun};
use crate::error::Result;
use crate::frame::save_scalar_field_png;
use crate::metrics::{analyze_demo_a, DemoAAnalysis, MetricsReport};
use crate::plots::{
    write_before_after_figure, write_system_diagram, write_trust_map_figure,
    write_trust_vs_error_figure,
};
use crate::report::write_report;
use crate::scene::{build_manifest, generate_sequence, SceneManifest, SceneSequence};
use crate::taa::{run_fixed_alpha, TaaRun};

#[derive(Clone, Debug, Serialize)]
pub struct DemoAArtifacts {
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
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);
    let analysis = analyze_demo_a(
        &sequence,
        &baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )?;

    write_frames(&sequence, &baseline, &dsfb, output_dir)?;
    write_debug_fields(&dsfb, output_dir)?;
    let metrics_path = write_metrics_json(output_dir, &analysis.report)?;
    let scene_manifest_path = write_scene_manifest(output_dir, &sequence)?;
    let figure_paths = write_figures(output_dir, &sequence, &baseline, &dsfb, &analysis)?;
    let report_path = output_dir.join("report.md");
    write_report(&report_path, config, &analysis.report)?;

    Ok(DemoAArtifacts {
        output_dir: output_dir.to_path_buf(),
        metrics_path,
        report_path,
        figure_paths,
        scene_manifest_path,
    })
}

fn write_frames(
    sequence: &SceneSequence,
    baseline: &TaaRun,
    dsfb: &DsfbRun,
    output_dir: &Path,
) -> Result<()> {
    let gt_dir = output_dir.join("frames").join("gt");
    let baseline_dir = output_dir.join("frames").join("baseline");
    let dsfb_dir = output_dir.join("frames").join("dsfb");
    fs::create_dir_all(&gt_dir)?;
    fs::create_dir_all(&baseline_dir)?;
    fs::create_dir_all(&dsfb_dir)?;

    for frame_index in 0..sequence.frames.len() {
        sequence.frames[frame_index]
            .ground_truth
            .save_png(&gt_dir.join(format!("frame_{frame_index:02}.png")))?;
        baseline.resolved_frames[frame_index]
            .save_png(&baseline_dir.join(format!("frame_{frame_index:02}.png")))?;
        dsfb.resolved_frames[frame_index]
            .save_png(&dsfb_dir.join(format!("frame_{frame_index:02}.png")))?;
    }

    Ok(())
}

fn write_debug_fields(dsfb: &DsfbRun, output_dir: &Path) -> Result<()> {
    let trust_dir = output_dir.join("frames").join("trust");
    fs::create_dir_all(&trust_dir)?;
    for (frame_index, supervision) in dsfb.supervision_frames.iter().enumerate() {
        save_scalar_field_png(
            &supervision.trust,
            &trust_dir.join(format!("frame_{frame_index:02}.png")),
            |trust| {
                let hazard = (1.0 - trust).clamp(0.0, 1.0);
                [
                    (hazard * 255.0).round() as u8,
                    (hazard * 160.0).round() as u8,
                    0,
                    255,
                ]
            },
        )?;
    }
    Ok(())
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
