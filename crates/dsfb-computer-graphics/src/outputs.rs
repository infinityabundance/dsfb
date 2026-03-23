use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};

pub const NOTEBOOK_OUTPUT_ROOT_NAME: &str = "output-dsfb-computer-graphics";
pub const ARTIFACT_MANIFEST_FILE_NAME: &str = "artifact_manifest.json";
pub const PDF_BUNDLE_FILE_NAME: &str = "artifacts_bundle.pdf";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RunLayout {
    pub output_root: PathBuf,
    pub run_dir: PathBuf,
    pub run_name: String,
    pub artifact_manifest_path: PathBuf,
    pub pdf_bundle_path: PathBuf,
    pub zip_bundle_path: PathBuf,
}

pub fn is_valid_timestamp_label(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut digit_count = 0usize;
    let mut hyphen_count = 0usize;

    for byte in bytes {
        if byte.is_ascii_digit() {
            digit_count += 1;
        } else if *byte == b'-' || *byte == b'_' {
            hyphen_count += 1;
        } else {
            return false;
        }
    }

    !value.is_empty() && digit_count >= 12 && hyphen_count >= 1
}

pub fn format_run_directory_name(timestamp: &str) -> String {
    format!("{NOTEBOOK_OUTPUT_ROOT_NAME}-{timestamp}")
}

pub fn format_zip_bundle_name(run_name: &str) -> String {
    format!("{run_name}.zip")
}

pub fn artifact_manifest_path(run_dir: &Path) -> PathBuf {
    run_dir.join(ARTIFACT_MANIFEST_FILE_NAME)
}

pub fn pdf_bundle_path(run_dir: &Path) -> PathBuf {
    run_dir.join(PDF_BUNDLE_FILE_NAME)
}

pub fn zip_bundle_path(output_root: &Path, run_name: &str) -> PathBuf {
    output_root.join(format_zip_bundle_name(run_name))
}

pub fn create_named_run_dir(output_root: &Path, run_name: &str) -> Result<RunLayout> {
    fs::create_dir_all(output_root)?;
    let run_dir = output_root.join(run_name);
    if run_dir.exists() {
        return Err(Error::Message(format!(
            "refusing to overwrite existing run directory {}",
            run_dir.display()
        )));
    }
    fs::create_dir_all(&run_dir)?;

    Ok(RunLayout {
        output_root: output_root.to_path_buf(),
        run_dir: run_dir.clone(),
        run_name: run_name.to_string(),
        artifact_manifest_path: artifact_manifest_path(&run_dir),
        pdf_bundle_path: pdf_bundle_path(&run_dir),
        zip_bundle_path: zip_bundle_path(output_root, run_name),
    })
}

pub fn create_timestamped_run_dir(output_root: &Path, timestamp: &str) -> Result<RunLayout> {
    if !is_valid_timestamp_label(timestamp) {
        return Err(Error::Message(format!(
            "timestamp label `{timestamp}` is invalid for notebook output naming"
        )));
    }

    create_named_run_dir(output_root, &format_run_directory_name(timestamp))
}
