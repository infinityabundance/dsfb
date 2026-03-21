use std::fs;
use std::path::PathBuf;

fn notebook_text() -> String {
    let text = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dsfb_semiotics_engine_colab.ipynb"),
    )
    .unwrap();
    let notebook: serde_json::Value = serde_json::from_str(&text).unwrap();
    notebook["cells"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|cell| {
            cell["source"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|line| line.as_str())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "notebook is missing expected snippet: {needle}"
        );
    }
}

#[test]
fn test_colab_notebook_contains_download_section_logic() {
    let notebook = notebook_text();
    assert!(notebook.contains("render_artifact_download_section"));
    assert!(notebook.contains("## Artifact Downloads"));
    assert!(notebook.contains("from IPython.display import HTML, Markdown, display"));
    assert!(notebook.contains("import ipywidgets as widgets"));
    assert!(notebook.contains("files.download(str(artifact_path))"));
}

#[test]
fn test_colab_notebook_references_pdf_and_zip_outputs() {
    let notebook = notebook_text();
    assert!(notebook.contains("report_pdf"));
    assert!(notebook.contains("zip_archive"));
    assert!(notebook.contains("PDF report"));
    assert!(notebook.contains("ZIP bundle"));
    assert!(notebook.contains("widgets.Button("));
}

#[test]
fn test_colab_notebook_handles_missing_artifacts_cleanly() {
    let notebook = notebook_text();
    assert!(notebook.contains("artifact_path.exists()"));
    assert!(notebook.contains("The artifact was not found, so no download button was rendered."));
}

#[test]
fn test_colab_notebook_uses_resolved_output_paths() {
    let notebook = notebook_text();
    assert!(notebook.contains("resolve_artifact_path"));
    assert!(notebook.contains("Resolved report PDF:"));
    assert!(notebook.contains("Resolved ZIP bundle:"));
}

#[test]
fn test_colab_notebook_surfaces_run_metadata_summary() {
    let notebook = notebook_text();
    assert!(notebook.contains("## Run Metadata Summary"));
    assert!(notebook.contains("Validation mode"));
    assert!(notebook.contains("Bank source"));
    assert!(notebook.contains("Numeric mode"));
    assert!(notebook.contains("Online buffer capacity"));
    assert!(notebook.contains("Trust scalar exported"));
}

#[test]
fn test_colab_notebook_contains_nasa_milling_section() {
    let notebook = notebook_text();
    assert!(notebook.contains("## Public Dataset Demo: NASA Milling"));
}

#[test]
fn test_colab_notebook_contains_nasa_bearings_section() {
    let notebook = notebook_text();
    assert!(notebook.contains("## Public Dataset Demo: NASA Bearings"));
}

#[test]
fn test_colab_notebook_runs_fetch_for_milling() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_milling\"",
            "prepare_public_dataset_raw_inputs",
            "fetch_public_dataset.py",
            "fetch raw dataset or reuse deterministic raw-summary cache",
            "rebuild raw summary from verified local archive",
            "--force-regenerate",
        ],
    );
}

#[test]
fn test_colab_notebook_runs_fetch_for_bearings() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_bearings\"",
            "prepare_public_dataset_raw_inputs",
            "fetch_public_dataset.py",
            "fetch raw dataset or reuse deterministic raw-summary cache",
            "rebuild raw summary from verified local archive",
            "--force-regenerate",
        ],
    );
}

#[test]
fn test_colab_notebook_runs_preprocess_for_milling() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_milling\"",
            "preprocess_public_dataset.py",
            "preprocess into DSFB-compatible CSV inputs",
        ],
    );
}

#[test]
fn test_colab_notebook_runs_preprocess_for_bearings() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_bearings\"",
            "preprocess_public_dataset.py",
            "preprocess into DSFB-compatible CSV inputs",
        ],
    );
}

#[test]
fn test_colab_notebook_runs_engine_pipeline_for_milling() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_milling\"",
            "dsfb-public-dataset-demo",
            "\"--phase\"",
            "\"run\"",
        ],
    );
}

#[test]
fn test_colab_notebook_runs_engine_pipeline_for_bearings() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "dataset_slug=\"nasa_bearings\"",
            "dsfb-public-dataset-demo",
            "\"--phase\"",
            "\"run\"",
        ],
    );
}

#[test]
fn test_colab_notebook_exposes_pdf_and_zip_for_milling() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "render_public_dataset_download_section(\"NASA Milling\", NASA_MILLING_RESULT)",
            "f\"{dataset_label} PDF report\"",
            "f\"{dataset_label} ZIP bundle\"",
        ],
    );
}

#[test]
fn test_colab_notebook_exposes_pdf_and_zip_for_bearings() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "render_public_dataset_download_section(\"NASA Bearings\", NASA_BEARINGS_RESULT)",
            "f\"{dataset_label} PDF report\"",
            "f\"{dataset_label} ZIP bundle\"",
        ],
    );
}

#[test]
fn test_colab_notebook_displays_resolved_paths_for_milling() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "render_public_dataset_results(NASA_MILLING_RESULT)",
            "Resolved {dataset_label} output directory:",
            "Resolved {dataset_label} manifest path:",
            "Resolved {dataset_label} figure output directory:",
            "Resolved {dataset_label} report PDF:",
            "Resolved {dataset_label} ZIP bundle:",
        ],
    );
}

#[test]
fn test_colab_notebook_displays_resolved_paths_for_bearings() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "render_public_dataset_results(NASA_BEARINGS_RESULT)",
            "Resolved {dataset_label} output directory:",
            "Resolved {dataset_label} manifest path:",
            "Resolved {dataset_label} figure output directory:",
            "Resolved {dataset_label} report PDF:",
            "Resolved {dataset_label} ZIP bundle:",
        ],
    );
}

#[test]
fn test_colab_notebook_preserves_figure_basename_conventions() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "paper_facing_figure_basename",
            "figure_path.name",
            "Paper-facing figure basenames are preserved exactly as emitted by the Rust pipeline.",
            "Dataset separation happens only through dataset-specific directories;",
        ],
    );
}

#[test]
fn test_colab_notebook_does_not_require_manual_dataset_download() {
    let notebook = notebook_text();
    assert!(notebook.contains("fetch_public_dataset.py"));
    assert!(notebook.contains("downloaded automatically"));
    assert!(notebook.contains("reused after integrity verification"));
    assert!(notebook.contains("raw summary cache is reused"));
    assert!(!notebook.contains("download this yourself"));
    assert!(!notebook.contains("manual download"));
}

#[test]
fn test_colab_notebook_rebuilds_from_scratch_each_run_or_explicitly_cleans_dataset_workdirs() {
    let notebook = notebook_text();
    assert_contains_all(
        &notebook,
        &[
            "clean_public_dataset_workdir",
            "clear_notebook_dataset_path",
            "derived outputs",
            "--force-regenerate",
            "reused so notebook reruns are deterministic without depending on a second network fetch",
        ],
    );
}
