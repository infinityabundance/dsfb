use std::fs;
use std::path::PathBuf;

fn notebook_text() -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dsfb_semiotics_engine_colab.ipynb"),
    )
    .unwrap()
}

#[test]
fn test_colab_notebook_contains_download_section_logic() {
    let notebook = notebook_text();
    assert!(notebook.contains("render_artifact_download_section"));
    assert!(notebook.contains("## Artifact Downloads"));
    assert!(notebook.contains("from IPython.display import HTML, Markdown, display"));
    assert!(notebook.contains("google.colab.files.download"));
}

#[test]
fn test_colab_notebook_references_pdf_and_zip_outputs() {
    let notebook = notebook_text();
    assert!(notebook.contains("report_pdf"));
    assert!(notebook.contains("zip_archive"));
    assert!(notebook.contains("PDF report"));
    assert!(notebook.contains("ZIP bundle"));
    assert!(notebook.contains("<button type='button'"));
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
