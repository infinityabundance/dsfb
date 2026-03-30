use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use dsfb_computer_graphics::config::DemoConfig;
use dsfb_computer_graphics::external_validation::{
    CANONICAL_HEADLINE_STATEMENT, PURE_DSFB_LIMITATION_STATEMENT, ROI_CONTRACT_ALPHA,
    ROI_CONTRACT_STATEMENT, ROI_HONESTY_STATEMENT,
};
use dsfb_computer_graphics::frame::{Color, ImageFrame};
use dsfb_computer_graphics::unreal_native::{
    run_unreal_native, UNREAL_NATIVE_EVIDENCE_MANIFEST_FILE_NAME, UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME,
    UNREAL_NATIVE_PDF_FILE_NAME, UNREAL_NATIVE_ZIP_FILE_NAME,
};
use serde_json::{json, Value};

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

fn write(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be creatable");
    }
    fs::write(path, content).expect("file should be writable");
}

fn simple_png(path: impl AsRef<Path>, rgb: [f32; 3]) {
    let mut frame = ImageFrame::new(32, 18);
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            frame.set(x, y, Color::rgb(rgb[0], rgb[1], rgb[2]));
        }
    }
    frame.save_png(path.as_ref()).expect("png should save");
}

#[test]
fn roi_contract_alpha_is_locked() {
    assert!(
        (ROI_CONTRACT_ALPHA - 0.15).abs() <= f32::EPSILON,
        "ROI alpha must remain fixed at 0.15"
    );
}

#[test]
fn unreal_native_rejects_non_real_provenance() {
    let root = unique_output_dir("unreal_native_rejects_non_real");
    let manifest_path = root.join("manifest.json");
    write(
        &manifest_path,
        &serde_json::to_string_pretty(&json!({
            "schema_version": "dsfb_unreal_native_v1",
            "dataset_kind": "unreal_native",
            "provenance_label": "unreal_native",
            "dataset_id": "bad_fixture",
            "description": "bad fixture",
            "engine": {
                "engine_name": "unreal_engine",
                "engine_version": "5.7.2",
                "capture_tool": "test",
                "real_engine_capture": false
            },
            "contract": {
                "color_space": "linear_rgb_pre_tonemap",
                "tonemap": "disabled",
                "depth_convention": "monotonic_linear_depth",
                "normal_space": "view_space_unit",
                "motion_vector_convention": "pixel_offset_to_prev",
                "coordinate_space": "screen_space_current_to_previous",
                "history_source": "previous_frame_export_plus_motion_reprojection"
            },
            "frames": [
                {
                    "label": "frame_0001",
                    "frame_index": 1,
                    "history_frame_index": 0,
                    "buffers": {
                        "current_color": { "path": "missing.exr", "format": "exr_rgb32f", "semantic": "current_color" },
                        "previous_color": { "path": "missing.exr", "format": "exr_rgb32f", "semantic": "previous_color" },
                        "motion_vectors": { "path": "missing.exr", "format": "exr_rg32f", "semantic": "motion_vectors" },
                        "current_depth": { "path": "missing.exr", "format": "exr_r32f", "semantic": "current_depth" },
                        "previous_depth": { "path": "missing.exr", "format": "exr_r32f", "semantic": "previous_depth" },
                        "current_normals": { "path": "missing.exr", "format": "exr_rgb32f", "semantic": "current_normals" },
                        "previous_normals": { "path": "missing.exr", "format": "exr_rgb32f", "semantic": "previous_normals" },
                        "metadata": { "path": "missing.json", "format": "json_metadata", "semantic": "metadata" }
                    }
                }
            ]
        }))
        .expect("manifest json"),
    );

    let error = run_unreal_native(
        &DemoConfig::default(),
        &manifest_path,
        &root.join("out"),
        Some("test_run"),
        &[],
    )
    .expect_err("non-real provenance should fail");
    assert!(
        error.to_string().contains("not marked as a real Unreal capture"),
        "actual error: {error}"
    );
}

#[test]
fn unreal_native_rejects_missing_required_buffer() {
    let root = unique_output_dir("unreal_native_missing_buffer");
    let data_dir = root.join("data");
    let metadata_path = data_dir.join("metadata.json");
    write(
        &metadata_path,
        &serde_json::to_string_pretty(&json!({
            "frame_index": 1,
            "history_frame_index": 0,
            "width": 32,
            "height": 18,
            "source_kind": "unreal_native",
            "real_external_data": true,
            "provenance_label": "unreal_native",
            "scene_name": "fixture_scene",
            "shot_name": "fixture_shot",
            "notes": []
        }))
        .expect("metadata json"),
    );
    let manifest_path = root.join("manifest.json");
    write(
        &manifest_path,
        &serde_json::to_string_pretty(&json!({
            "schema_version": "dsfb_unreal_native_v1",
            "dataset_kind": "unreal_native",
            "provenance_label": "unreal_native",
            "dataset_id": "missing_buffer_fixture",
            "description": "missing buffer fixture",
            "engine": {
                "engine_name": "unreal_engine",
                "engine_version": "5.7.2",
                "capture_tool": "test",
                "real_engine_capture": true
            },
            "contract": {
                "color_space": "linear_rgb_pre_tonemap",
                "tonemap": "disabled",
                "depth_convention": "monotonic_linear_depth",
                "normal_space": "view_space_unit",
                "motion_vector_convention": "pixel_offset_to_prev",
                "coordinate_space": "screen_space_current_to_previous",
                "history_source": "previous_frame_export_plus_motion_reprojection"
            },
            "frames": [
                {
                    "label": "frame_0001",
                    "frame_index": 1,
                    "history_frame_index": 0,
                    "buffers": {
                        "current_color": { "path": "data/current_color.exr", "format": "exr_rgb32f", "semantic": "current_color", "width": 32, "height": 18, "channels": 3 },
                        "previous_color": { "path": "data/previous_color.exr", "format": "exr_rgb32f", "semantic": "previous_color", "width": 32, "height": 18, "channels": 3 },
                        "motion_vectors": { "path": "data/motion_vectors.exr", "format": "exr_rg32f", "semantic": "motion_vectors", "width": 32, "height": 18, "channels": 2 },
                        "current_depth": { "path": "data/current_depth.exr", "format": "exr_r32f", "semantic": "current_depth", "width": 32, "height": 18, "channels": 1 },
                        "previous_depth": { "path": "data/previous_depth.exr", "format": "exr_r32f", "semantic": "previous_depth", "width": 32, "height": 18, "channels": 1 },
                        "current_normals": { "path": "data/current_normals.exr", "format": "exr_rgb32f", "semantic": "current_normals", "width": 32, "height": 18, "channels": 3 },
                        "previous_normals": { "path": "data/previous_normals.exr", "format": "exr_rgb32f", "semantic": "previous_normals", "width": 32, "height": 18, "channels": 3 },
                        "metadata": { "path": "data/metadata.json", "format": "json_metadata", "semantic": "metadata" }
                    }
                }
            ]
        }))
        .expect("manifest json"),
    );

    let error = run_unreal_native(
        &DemoConfig::default(),
        &manifest_path,
        &root.join("out"),
        Some("test_run"),
        &[],
    )
    .expect_err("missing required buffers should fail");
    assert!(
        error.to_string().contains("No such file")
            || error.to_string().contains("I/O error")
            || error.to_string().contains("image error"),
        "actual error: {error}"
    );
}

#[test]
fn unreal_native_bundle_builder_creates_pdf_and_zip() {
    let python = Command::new("python3").arg("--version").output();
    if python.is_err() {
        return;
    }

    let root = unique_output_dir("unreal_native_bundle_builder");
    let panels_dir = root.join("per_frame/frame_0001");
    fs::create_dir_all(&panels_dir).expect("panel dir should exist");

    for (name, rgb) in [
        ("current_frame.png", [0.9, 0.3, 0.2]),
        ("baseline_or_host_output.png", [0.2, 0.3, 0.9]),
        ("trust_map.png", [0.2, 0.4, 1.0]),
        ("alpha_map.png", [1.0, 0.6, 0.2]),
        ("intervention_map.png", [1.0, 0.2, 0.2]),
        ("residual_map.png", [1.0, 0.4, 0.1]),
        ("instability_overlay.png", [0.8, 0.1, 0.1]),
        ("roi_overlay.png", [0.1, 0.8, 0.2])
    ] {
        simple_png(panels_dir.join(name), rgb);
    }

    write(
        root.join("summary.json"),
        &serde_json::to_string_pretty(&json!({
            "dataset_id": "bundle_fixture",
            "provenance_label": "unreal_native",
            "capture_count": 1,
            "executive_capture_label": "frame_0001"
        }))
        .expect("summary json"),
    );
    write(
        root.join(UNREAL_NATIVE_EVIDENCE_MANIFEST_FILE_NAME),
        &serde_json::to_string_pretty(&json!({
            "dataset_id": "bundle_fixture",
            "provenance_label": "unreal_native",
            "run_name": "bundle_fixture_run",
            "pdf_file_name": UNREAL_NATIVE_PDF_FILE_NAME,
            "zip_file_name": UNREAL_NATIVE_ZIP_FILE_NAME,
            "executive_sheet_file_name": UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME,
            "summary_file_name": "summary.json",
            "metrics_summary_file_name": "metrics_summary.json",
            "comparison_summary_file_name": "comparison_summary.md",
            "failure_modes_file_name": "failure_modes.md",
            "frames": [
                {
                    "label": "frame_0001",
                    "scene_name": "fixture_scene",
                    "shot_name": "fixture_shot",
                    "frame_index": 1,
                    "classification": "dsfb_helpful",
                    "explanation": {
                        "what_went_wrong": "A temporal instability region is visible.",
                        "what_dsfb_detected": "DSFB concentrated low trust and intervention there.",
                        "what_dsfb_changed": "DSFB would gate the temporal reuse path.",
                        "overhead_and_caveat": "This is a bounded monitor result."
                    },
                    "key_metrics": [
                        { "label": "DSFB ROI MAE", "value": "0.01000" },
                        { "label": "Strong heuristic ROI MAE", "value": "0.02000" },
                        { "label": "Mean trust", "value": "0.4500" },
                        { "label": "Intervention rate", "value": "0.3000" },
                        { "label": "GPU total ms", "value": "1.5000" }
                    ],
                    "current_frame_path": "per_frame/frame_0001/current_frame.png",
                    "baseline_frame_path": "per_frame/frame_0001/baseline_or_host_output.png",
                    "trust_map_path": "per_frame/frame_0001/trust_map.png",
                    "alpha_map_path": "per_frame/frame_0001/alpha_map.png",
                    "intervention_map_path": "per_frame/frame_0001/intervention_map.png",
                    "residual_map_path": "per_frame/frame_0001/residual_map.png",
                    "instability_overlay_path": "per_frame/frame_0001/instability_overlay.png",
                    "roi_overlay_path": "per_frame/frame_0001/roi_overlay.png",
                    "output_panel_path": "per_frame/frame_0001/boardroom_panel_frame_0001.png"
                }
            ]
        }))
        .expect("manifest json"),
    );
    write(root.join("metrics_summary.json"), "{}");
    write(root.join("comparison_summary.md"), "# Comparison");
    write(root.join("failure_modes.md"), "# Failure Modes");

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("colab")
        .join("build_unreal_native_bundle.py");
    let status = Command::new("python3")
        .arg(&script)
        .arg("--run-dir")
        .arg(&root)
        .status()
        .expect("bundle builder should run");
    assert!(status.success(), "bundle builder should succeed");
    assert!(root.join(UNREAL_NATIVE_EXECUTIVE_SHEET_FILE_NAME).exists());
    assert!(root.join(UNREAL_NATIVE_PDF_FILE_NAME).exists());
    assert!(root.join(UNREAL_NATIVE_ZIP_FILE_NAME).exists());
    assert!(root
        .join("per_frame/frame_0001/boardroom_panel_frame_0001.png")
        .exists());
}

#[test]
fn unreal_native_sample_manifest_smoke_runs() {
    let python = Command::new("python3").arg("--version").output();
    if python.is_err() {
        return;
    }

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("unreal_native_capture_manifest.json");
    let output_root = unique_output_dir("unreal_native_sample_manifest_smoke").join("out");
    let artifacts = run_unreal_native(
        &DemoConfig::default(),
        &manifest_path,
        &output_root,
        Some("sample_manifest_smoke"),
        &[],
    )
    .expect("checked-in Unreal-native sample should replay successfully");

    assert!(artifacts.summary_path.exists());
    assert!(artifacts.metrics_summary_path.exists());
    assert!(artifacts.comparison_summary_path.exists());
    assert!(artifacts.failure_modes_path.exists());
    assert!(artifacts.executive_sheet_path.exists());
    assert!(artifacts.pdf_path.exists());
    assert!(artifacts.zip_path.exists());
    assert!(artifacts.run_dir.join("canonical_metric_sheet.md").exists());
    assert!(artifacts.run_dir.join("aggregation_summary.md").exists());
    assert!(artifacts.run_dir.join("figures/trust_histogram.svg").exists());
    assert!(artifacts.run_dir.join("figures/trust_vs_error.svg").exists());
    assert!(artifacts
        .run_dir
        .join("figures/trust_temporal_trajectory.svg")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures/trust_temporal_trajectory.json")
        .exists());
    assert!(artifacts
        .run_dir
        .join("figures/trust_conditioned_error_map.png")
        .exists());
    assert!(artifacts
        .run_dir
        .join("per_frame/frame_0001/roi_mask.json")
        .exists());

    let summary_text = fs::read_to_string(&artifacts.summary_path).expect("summary should exist");
    let summary: Value = serde_json::from_str(&summary_text).expect("summary should be valid json");
    assert_eq!(summary["dataset_kind"], "unreal_native");
    assert_eq!(summary["provenance_label"], "unreal_native");
    assert_eq!(summary["capture_count"], 5);
    assert!(summary_text.contains(ROI_CONTRACT_STATEMENT));

    let comparison_summary = fs::read_to_string(&artifacts.comparison_summary_path)
        .expect("comparison summary should exist");
    assert!(comparison_summary.contains(ROI_CONTRACT_STATEMENT));
    assert!(comparison_summary.contains(CANONICAL_HEADLINE_STATEMENT));
    assert!(comparison_summary.contains(PURE_DSFB_LIMITATION_STATEMENT));
    assert!(comparison_summary.contains(ROI_HONESTY_STATEMENT));
    assert!(comparison_summary.contains("DSFB + heuristic ROI MAE"));
    assert!(comparison_summary.contains("onset `frame_0001`"));
    assert!(comparison_summary.contains("peak ROI `frame_0002`"));
    assert!(comparison_summary.contains("recovery-side `frame_0005`"));
    assert!(comparison_summary.contains("0.78657 -> 0.35245 -> 0.49284"));
    assert!(comparison_summary.contains("0.21345 -> 0.64758 -> 0.50715"));
    assert!(comparison_summary.contains("0.23822"));
    assert!(comparison_summary.contains("0.26347"));
    assert!(comparison_summary.contains("0.30026"));

    let canonical_metric_sheet = fs::read_to_string(artifacts.run_dir.join("canonical_metric_sheet.md"))
        .expect("canonical metric sheet should exist");
    assert!(canonical_metric_sheet.contains("Strong heuristic"));
    assert!(canonical_metric_sheet.contains("DSFB + heuristic"));
    assert!(canonical_metric_sheet.contains(ROI_CONTRACT_STATEMENT));

    let aggregation_summary = fs::read_to_string(artifacts.run_dir.join("aggregation_summary.md"))
        .expect("aggregation summary should exist");
    assert!(aggregation_summary.contains("Real capture count in this run: `5`"));
    assert!(aggregation_summary.contains("DSFB + heuristic mean ± std"));

    let metrics_summary_text =
        fs::read_to_string(&artifacts.metrics_summary_path).expect("metrics summary should exist");
    let metrics_summary: Value =
        serde_json::from_str(&metrics_summary_text).expect("metrics summary should be valid json");
    assert_eq!(metrics_summary["capture_count"], 5);
    assert_eq!(
        metrics_summary["captures"][0]["roi_source"],
        "fixed_alpha_local_contrast_0p15"
    );
    assert_eq!(metrics_summary["captures"][0]["reference_source"], "reference_color");

    let current_status_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("CURRENT_STATUS.md");
    assert!(current_status_path.exists(), "CURRENT_STATUS.md must exist");
    let current_status =
        fs::read_to_string(&current_status_path).expect("CURRENT_STATUS.md should be readable");
    assert!(current_status.contains(CANONICAL_HEADLINE_STATEMENT));
    assert!(current_status.contains(PURE_DSFB_LIMITATION_STATEMENT));
    assert!(current_status.contains(ROI_HONESTY_STATEMENT));
    assert!(current_status.contains("0.00501 +- 0.00178"));
    assert!(current_status.contains("0.00657 +- 0.00247"));
    assert!(current_status.contains("50.60% +- 18.61%"));
    assert!(current_status.contains("0.78657 -> 0.35245 -> 0.49284"));
    assert!(current_status.contains("0.21345 -> 0.64758 -> 0.50715"));
}

#[test]
fn unreal_native_notebook_is_valid_and_strict() {
    let notebook_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("colab")
        .join("dsfb_unreal_native_evidence.ipynb");
    let notebook_text = fs::read_to_string(&notebook_path).expect("notebook should exist");
    let notebook: Value =
        serde_json::from_str(&notebook_text).expect("notebook should be valid json");
    let cells = notebook["cells"]
        .as_array()
        .expect("notebook should contain a cell array");

    assert!(cells.len() >= 6, "notebook should contain multiple sections");
    assert!(notebook_text.contains("run-unreal-native"));
    assert!(notebook_text.contains("unreal_native"));
    assert!(notebook_text.contains("refuse to mislabel synthetic data"));
    assert!(notebook_text.contains("Download PDF"));
    assert!(notebook_text.contains("Download ZIP"));
}
