use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dsfb_computer_graphics::config::DemoConfig;
use dsfb_computer_graphics::outputs::{
    create_named_run_dir, create_timestamped_run_dir, format_run_directory_name,
    format_zip_bundle_name, ARTIFACT_MANIFEST_FILE_NAME, NOTEBOOK_OUTPUT_ROOT_NAME,
    PDF_BUNDLE_FILE_NAME,
};
use dsfb_computer_graphics::pipeline::run_all;
use dsfb_computer_graphics::report::EXPERIMENT_SENTENCE;
use serde_json::Value;

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
fn timestamped_output_layout_helpers_create_expected_paths() {
    let output_root = unique_output_dir("notebook_layout");
    let timestamp = "20260323-101530";
    let layout = create_timestamped_run_dir(&output_root, timestamp).expect("timestamped run dir");
    let expected_run_name = format_run_directory_name(timestamp);

    assert_eq!(layout.run_name, expected_run_name);
    assert_eq!(layout.run_dir, output_root.join(&expected_run_name));
    assert_eq!(
        layout.artifact_manifest_path,
        layout.run_dir.join(ARTIFACT_MANIFEST_FILE_NAME)
    );
    assert_eq!(
        layout.pdf_bundle_path,
        layout.run_dir.join(PDF_BUNDLE_FILE_NAME)
    );
    assert_eq!(
        layout.zip_bundle_path,
        output_root.join(format_zip_bundle_name(&expected_run_name))
    );

    assert!(
        create_named_run_dir(&output_root, &expected_run_name).is_err(),
        "existing run directories should not be overwritten"
    );
    assert!(
        create_timestamped_run_dir(&output_root, "bad timestamp label").is_err(),
        "invalid timestamp labels should be rejected"
    );
}

#[test]
fn run_all_writes_notebook_manifest_with_relative_paths() {
    let output_dir = unique_output_dir("run_all_manifest");
    let mut config = DemoConfig::default();
    config.scene.frame_count = 12;
    config.scene.move_frames = 4;
    config.demo_b_reference_spp = 24;
    let artifacts = run_all(&config, &output_dir).expect("run-all should succeed");
    let manifest_text = read(&artifacts.manifest_path);
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("manifest should be valid json");
    let run_name = output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .expect("output dir should have a utf-8 file name")
        .to_string();
    let expected_zip_name = format_zip_bundle_name(&run_name);

    assert_eq!(
        manifest["output_root_name"].as_str(),
        Some(NOTEBOOK_OUTPUT_ROOT_NAME)
    );
    assert_eq!(manifest["run_name"].as_str(), Some(run_name.as_str()));
    assert_eq!(
        manifest["artifact_manifest_file_name"].as_str(),
        Some(ARTIFACT_MANIFEST_FILE_NAME)
    );
    assert_eq!(
        manifest["pdf_bundle_file_name"].as_str(),
        Some(PDF_BUNDLE_FILE_NAME)
    );
    assert_eq!(
        manifest["zip_bundle_file_name"].as_str(),
        Some(expected_zip_name.as_str())
    );
    assert_eq!(
        manifest["demo_a"]["report_path"].as_str(),
        Some("report.md")
    );
    assert_eq!(
        manifest["demo_b"]["report_path"].as_str(),
        Some("demo_b/report.md")
    );

    let demo_a_figures = manifest["demo_a"]["figure_paths"]
        .as_array()
        .expect("demo a figures should be an array");
    let demo_b_figures = manifest["demo_b"]["figure_paths"]
        .as_array()
        .expect("demo b figures should be an array");
    let reviewer_reports = manifest["reviewer_report_paths"]
        .as_array()
        .expect("reviewer reports should be an array");

    assert!(
        demo_a_figures.len() >= 9,
        "demo a manifest should surface the expanded figure set"
    );
    assert!(
        demo_b_figures.len() >= 2,
        "demo b manifest should surface both main figures"
    );
    assert!(
        reviewer_reports.len() >= 3,
        "reviewer manifest should include the extra decision reports"
    );
}

#[test]
fn notebook_json_is_valid_and_contains_required_controls() {
    let notebook_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("colab")
        .join("dsfb_computer_graphics_demo.ipynb");
    let notebook_text = read(&notebook_path);
    let notebook: Value =
        serde_json::from_str(&notebook_text).expect("notebook should be valid json");
    let cells = notebook["cells"]
        .as_array()
        .expect("notebook should contain a cells array");

    assert!(
        cells.len() >= 10,
        "notebook should contain multiple sections"
    );
    assert!(notebook_text.contains(EXPERIMENT_SENTENCE));
    assert!(notebook_text.contains("Download PDF"));
    assert!(notebook_text.contains("Download ZIP"));
    assert!(notebook_text.contains("cargo run -- run-all --output"));
    assert!(notebook_text.contains("fig_intervention_alpha.svg"));
    assert!(notebook_text.contains("Strong heuristic"));
}

#[test]
fn colab_documentation_describes_bundle_outputs() {
    let doc = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("colab_notebook.md"),
    );
    let bundle_script = read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("colab")
            .join("build_artifact_bundle.py"),
    );

    assert!(doc.contains("artifacts_bundle.pdf"));
    assert!(doc.contains("output-dsfb-computer-graphics-YYYYMMDD-HHMMSS"));
    assert!(doc.contains("ZIP"));
    assert!(doc.contains(EXPERIMENT_SENTENCE));
    assert!(bundle_script.contains("find_run"));
    assert!(bundle_script.contains("find_policy"));
}
