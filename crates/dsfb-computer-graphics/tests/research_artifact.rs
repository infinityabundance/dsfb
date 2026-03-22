use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dsfb_computer_graphics::config::DemoConfig;
use dsfb_computer_graphics::dsfb::run_gated_taa;
use dsfb_computer_graphics::metrics::analyze_demo_a;
use dsfb_computer_graphics::pipeline::run_demo_a;
use dsfb_computer_graphics::scene::generate_sequence;
use dsfb_computer_graphics::taa::run_fixed_alpha;

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
fn baseline_and_dsfb_outputs_have_matching_shapes() {
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);

    assert_eq!(baseline.resolved_frames.len(), sequence.frames.len());
    assert_eq!(dsfb.resolved_frames.len(), sequence.frames.len());
    assert_eq!(
        baseline.reprojected_history_frames.len(),
        sequence.frames.len()
    );
    assert_eq!(dsfb.reprojected_history_frames.len(), sequence.frames.len());

    for frame_index in 0..sequence.frames.len() {
        let gt = &sequence.frames[frame_index].ground_truth;
        assert_eq!(baseline.resolved_frames[frame_index].width(), gt.width());
        assert_eq!(baseline.resolved_frames[frame_index].height(), gt.height());
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
    let config = DemoConfig::default();
    let sequence = generate_sequence(&config.scene);
    let baseline = run_fixed_alpha(&sequence, config.baseline_alpha);
    let dsfb = run_gated_taa(&sequence, config.dsfb_alpha_min, config.dsfb_alpha_max);
    let analysis = analyze_demo_a(
        &sequence,
        &baseline,
        &dsfb,
        config.trust_map_frame_offset,
        config.comparison_frame_offset,
    )
    .expect("analysis should succeed");

    assert!(analysis.report.summary.reveal_frame > 0);
    assert!(analysis.report.summary.persistence_mask_pixels >= 10);
    assert!(
        analysis.report.summary.baseline_ghost_persistence_frames
            > analysis.report.summary.dsfb_ghost_persistence_frames
    );
    assert!(
        analysis
            .report
            .summary
            .cumulative_persistence_roi_mae_baseline
            > analysis.report.summary.cumulative_persistence_roi_mae_dsfb
    );
    assert!(
        analysis.report.summary.trust_error_correlation > 0.5,
        "correlation should remain meaningfully positive"
    );
}

#[test]
fn library_demo_writes_figures_metrics_and_report() {
    let config = DemoConfig::default();
    let output_dir = unique_output_dir("library_demo");
    let artifacts = run_demo_a(&config, &output_dir).expect("demo should succeed");

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

    assert_eq!(artifacts.figure_paths.len(), 4);
    for figure in &artifacts.figure_paths {
        let metadata = fs::metadata(figure).expect("figure metadata should exist");
        assert!(
            metadata.len() > 0,
            "figure {} should be non-empty",
            figure.display()
        );
    }
}

#[test]
fn required_exact_sentences_are_present_in_generated_report_and_docs() {
    let config = DemoConfig::default();
    let output_dir = unique_output_dir("sentence_check");
    let artifacts = run_demo_a(&config, &output_dir).expect("demo should succeed");

    let report = read(&artifacts.report_path);
    let readme = read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"));
    let gpu_doc = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("gpu_implementation.md"),
    );

    let experiment_sentence =
        "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”";
    let cost_sentence = "“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”";
    let compatibility_sentence =
        "“The framework is compatible with tiled and asynchronous GPU execution.”";

    assert!(report.contains(experiment_sentence));
    assert!(report.contains(cost_sentence));
    assert!(report.contains(compatibility_sentence));
    assert!(readme.contains(experiment_sentence));
    assert!(gpu_doc.contains("GPU Implementation Considerations"));
    assert!(gpu_doc.contains(experiment_sentence));
    assert!(gpu_doc.contains(cost_sentence));
    assert!(gpu_doc.contains(compatibility_sentence));
}
