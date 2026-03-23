use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dsfb_computer_graphics::config::DemoConfig;
use dsfb_computer_graphics::dsfb::run_gated_taa;
use dsfb_computer_graphics::metrics::analyze_demo_a;
use dsfb_computer_graphics::pipeline::{run_demo_a, run_demo_b};
use dsfb_computer_graphics::report::{COMPATIBILITY_SENTENCE, COST_SENTENCE, EXPERIMENT_SENTENCE};
use dsfb_computer_graphics::sampling::run_demo_b as run_demo_b_core;
use dsfb_computer_graphics::scene::generate_sequence;
use dsfb_computer_graphics::taa::{run_fixed_alpha, run_residual_threshold};

fn unique_output_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("generated")
        .join("test_runs")
        .join(format!("{name}_{stamp}"));
    fs::create_dir_all(&dir).expect("test output directory should be creatable");
    dir
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("text file should be readable")
}

fn analyze_default_demo() -> dsfb_computer_graphics::metrics::DemoAAnalysis {
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let residual_baseline = run_residual_threshold(
        &sequence,
        config.baseline_alpha,
        config.residual_baseline_alpha_high,
        config.residual_baseline_threshold_low,
        config.residual_baseline_threshold_high,
    );
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);
    analyze_demo_a(
        &sequence,
        &baseline,
        &residual_baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )
    .expect("analysis should succeed")
}

#[test]
fn scene_generation_is_deterministic() {
    let config = DemoConfig::default();
    let sequence_a = generate_sequence(&config.scene);
    let sequence_b = generate_sequence(&config.scene);

    assert_eq!(sequence_a.frames.len(), sequence_b.frames.len());
    for (frame_a, frame_b) in sequence_a.frames.iter().zip(sequence_b.frames.iter()) {
        assert_eq!(frame_a.layers, frame_b.layers);
        assert_eq!(frame_a.motion, frame_b.motion);
        assert_eq!(frame_a.disocclusion_mask, frame_b.disocclusion_mask);
        assert_eq!(
            frame_a
                .ground_truth
                .encode_png()
                .expect("frame_a should encode"),
            frame_b
                .ground_truth
                .encode_png()
                .expect("frame_b should encode")
        );
    }
}

#[test]
fn baseline_residual_baseline_and_dsfb_outputs_have_matching_shapes() {
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let residual_baseline = run_residual_threshold(
        &sequence,
        config.baseline_alpha,
        config.residual_baseline_alpha_high,
        config.residual_baseline_threshold_low,
        config.residual_baseline_threshold_high,
    );
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);

    assert_eq!(baseline.resolved_frames.len(), sequence.frames.len());
    assert_eq!(
        residual_baseline.taa.resolved_frames.len(),
        sequence.frames.len()
    );
    assert_eq!(dsfb.resolved_frames.len(), sequence.frames.len());
    assert_eq!(
        baseline.reprojected_history_frames.len(),
        sequence.frames.len()
    );
    assert_eq!(
        residual_baseline.taa.reprojected_history_frames.len(),
        sequence.frames.len()
    );
    assert_eq!(dsfb.reprojected_history_frames.len(), sequence.frames.len());

    for frame_index in 0..sequence.frames.len() {
        let gt = &sequence.frames[frame_index].ground_truth;
        assert_eq!(baseline.resolved_frames[frame_index].width(), gt.width());
        assert_eq!(baseline.resolved_frames[frame_index].height(), gt.height());
        assert_eq!(
            residual_baseline.taa.resolved_frames[frame_index].width(),
            gt.width()
        );
        assert_eq!(
            residual_baseline.taa.resolved_frames[frame_index].height(),
            gt.height()
        );
        assert_eq!(dsfb.resolved_frames[frame_index].width(), gt.width());
        assert_eq!(dsfb.resolved_frames[frame_index].height(), gt.height());
    }
}

#[test]
fn trust_values_remain_in_unit_interval() {
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);

    for supervision in &dsfb.supervision_frames {
        for trust in supervision.trust.values() {
            assert!(
                (0.0..=1.0).contains(trust),
                "trust value {trust} should stay in [0, 1]"
            );
        }
    }
}

#[test]
fn metrics_show_behavioral_separation_without_overclaiming() {
    let analysis = analyze_default_demo();
    let summary = &analysis.report.summary;

    assert!(summary.reveal_frame > 0);
    assert!(summary.persistence_mask_pixels >= 10);
    assert!(summary.baseline_ghost_persistence_frames > summary.dsfb_ghost_persistence_frames);
    assert!(summary.baseline_peak_roi_error > summary.dsfb_peak_roi_error);
    assert!(
        summary.cumulative_persistence_roi_mae_baseline
            > summary.cumulative_persistence_roi_mae_dsfb
    );
    assert!(
        summary.trust_error_correlation > 0.5,
        "correlation should remain meaningfully positive"
    );
    assert!(
        summary
            .primary_behavioral_result
            .contains("In this bounded synthetic setting"),
        "primary result should remain an empirical summary"
    );
}

#[test]
fn library_demo_writes_figures_metrics_report_summary_note_and_state_exports() {
    let config = DemoConfig::default();
    let output_dir = unique_output_dir("library_demo");
    let artifacts = run_demo_a(&config, &output_dir).expect("demo should succeed");

    for path in [
        artifacts.metrics_path.as_path(),
        artifacts.report_path.as_path(),
        artifacts.reviewer_summary_path.as_path(),
        artifacts.completion_note_path.as_path(),
        artifacts.scene_manifest_path.as_path(),
    ] {
        let metadata = fs::metadata(path).expect("artifact metadata should exist");
        assert!(
            metadata.len() > 0,
            "artifact {} should be non-empty",
            path.display()
        );
    }

    assert_eq!(artifacts.figure_paths.len(), 4);
    for figure in &artifacts.figure_paths {
        let metadata = fs::metadata(figure).expect("figure metadata should exist");
        assert!(
            metadata.len() > 0,
            "figure {} should be non-empty",
            figure.display()
        );
    }

    for relative in [
        "frames/residual/frame_06.png",
        "frames/trust/frame_06.png",
        "frames/alpha/frame_06.png",
        "frames/intervention/frame_06.png",
        "frames/proxy_residual/frame_06.png",
        "frames/proxy_visibility/frame_06.png",
        "frames/proxy_motion_edge/frame_06.png",
        "frames/proxy_thin/frame_06.png",
        "frames/state/frame_06.png",
        "frames/residual_baseline/frame_06.png",
        "frames/residual_baseline_trigger/frame_06.png",
        "frames/residual_baseline_alpha/frame_06.png",
    ] {
        let path = output_dir.join(relative);
        let metadata = fs::metadata(&path).expect("debug artifact metadata should exist");
        assert!(
            metadata.len() > 0,
            "debug artifact {} should be non-empty",
            path.display()
        );
    }
}

#[test]
fn demo_b_improves_roi_error_at_fixed_budget_and_writes_artifacts() {
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let residual_baseline = run_residual_threshold(
        &sequence,
        config.baseline_alpha,
        config.residual_baseline_alpha_high,
        config.residual_baseline_threshold_low,
        config.residual_baseline_threshold_high,
    );
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);
    let analysis = analyze_demo_a(
        &sequence,
        &baseline,
        &residual_baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )
    .expect("analysis should succeed");
    let run = run_demo_b_core(&config, &sequence, &dsfb, &analysis).expect("demo b should run");

    assert_eq!(
        run.metrics.uniform_total_samples, run.metrics.guided_total_samples,
        "guided sampling must preserve total budget"
    );
    assert!(
        run.metrics.guided_roi_mae < run.metrics.uniform_roi_mae,
        "guided sampling should reduce ROI MAE"
    );
    assert!(
        run.metrics.roi_mean_guided_spp > run.metrics.uniform_spp as f32,
        "guided sampling should spend more samples inside the ROI"
    );

    let output_dir = unique_output_dir("library_demo_b");
    let artifacts = run_demo_b(&config, &output_dir).expect("demo b artifacts should be written");
    for path in [
        artifacts.metrics_path.as_path(),
        artifacts.report_path.as_path(),
        artifacts.scene_manifest_path.as_path(),
    ] {
        let metadata = fs::metadata(path).expect("artifact metadata should exist");
        assert!(
            metadata.len() > 0,
            "artifact {} should be non-empty",
            path.display()
        );
    }
}

#[test]
fn required_exact_sentences_are_present_in_generated_report_and_docs() {
    let config = DemoConfig::default();
    let output_dir = unique_output_dir("sentence_check");
    let artifacts = run_demo_a(&config, &output_dir).expect("demo should succeed");
    let demo_b_artifacts = run_demo_b(&config, &output_dir).expect("demo b should succeed");

    let report = read(&artifacts.report_path);
    let demo_b_report = read(&demo_b_artifacts.report_path);
    let readme = read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"));
    let gpu_doc = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("gpu_implementation.md"),
    );

    assert!(report.contains(EXPERIMENT_SENTENCE));
    assert!(demo_b_report.contains(EXPERIMENT_SENTENCE));
    assert!(report.contains(COST_SENTENCE));
    assert!(report.contains(COMPATIBILITY_SENTENCE));
    assert!(readme.contains(EXPERIMENT_SENTENCE));
    assert!(gpu_doc.contains("GPU Implementation Considerations"));
    assert!(gpu_doc.contains(EXPERIMENT_SENTENCE));
    assert!(gpu_doc.contains(COST_SENTENCE));
    assert!(gpu_doc.contains(COMPATIBILITY_SENTENCE));
}

#[test]
fn readme_and_report_include_required_sections_and_default_numeric_summary() {
    let analysis = analyze_default_demo();
    let summary = &analysis.report.summary;
    let readme = read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"));

    for section in [
        "## DSFB Integration into Temporal Reuse",
        "## GPU Implementation Considerations",
        "## Mission and Transition Relevance",
        "## Product Framing and Integration Surfaces",
        "## What this crate does not claim",
        "## Limitations",
        "## Future Work",
    ] {
        assert!(readme.contains(section), "README missing section {section}");
    }

    assert!(
        readme.contains(&summary.primary_behavioral_result),
        "README should contain the default-run primary numeric summary"
    );
    if let Some(result) = &summary.secondary_behavioral_result {
        assert!(
            readme.contains(result),
            "README should contain the default-run secondary numeric summary"
        );
    }

    let output_dir = unique_output_dir("report_sections");
    let artifacts = run_demo_a(&DemoConfig::default(), &output_dir).expect("demo should succeed");
    let report = read(&artifacts.report_path);

    for section in [
        "## Numeric Demo Summary",
        "## DSFB State Exports",
        "## DSFB Integration into Temporal Reuse",
        "## GPU Implementation Considerations",
        "## Mission and Transition Relevance",
        "## Product Framing and Integration Surfaces",
        "## What this crate does not claim",
    ] {
        assert!(report.contains(section), "report missing section {section}");
    }
    assert!(report.contains(&summary.primary_behavioral_result));
}

#[test]
fn completion_note_contains_required_checklist_entries() {
    let config = DemoConfig::default();
    let output_dir = unique_output_dir("completion_note");
    let artifacts = run_demo_a(&config, &output_dir).expect("demo should succeed");
    let note = read(&artifacts.completion_note_path);

    for line in [
        "Only files inside crates/dsfb-computer-graphics were changed",
        "Demo A runs end-to-end",
        "Metrics are generated",
        "Figures are generated",
        "Report is generated",
        "Reviewer summary is generated",
        "Exact required sentences are present",
        "cargo fmt passed",
        "cargo clippy passed",
        "cargo test passed",
        "No fabricated performance claims were made",
    ] {
        assert!(note.contains(line), "completion note missing line {line}");
    }
}
