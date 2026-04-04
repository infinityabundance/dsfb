use crate::error::Result;
use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn repo_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("crate directory to live under <repo>/crates/dsfb-semiconductor")
}

pub fn default_data_root() -> PathBuf {
    crate_root().join("data").join("raw")
}

pub fn default_output_root() -> PathBuf {
    repo_root().join("output-dsfb-semiconductor")
}

pub fn create_timestamped_run_dir(output_root: &Path, dataset: &str) -> std::io::Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let base = format!("{timestamp}_dsfb-semiconductor_{dataset}");
    for attempt in 0..1000 {
        let candidate = if attempt == 0 {
            output_root.join(&base)
        } else {
            output_root.join(format!("{base}_{attempt:03}"))
        };
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "failed to allocate unique run directory under {}",
            output_root.display()
        ),
    ))
}

/// Compile a `.tex` file in-place, running pdflatex up to three times to
/// resolve cross-references.  Returns the pdf path (if produced) and any
/// captured error text.
pub(crate) fn compile_pdf(tex_path: &Path, output_dir: &Path) -> (Option<PathBuf>, Option<String>) {
    let filename = tex_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "engineering_report.tex".into());
    let pdf_path = output_dir.join(filename.replace(".tex", ".pdf"));
    let mut combined_output = String::new();
    let mut any_success = false;

    for _ in 0..3 {
        match Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-output-directory")
            .arg(".")
            .arg(&filename)
            .current_dir(output_dir)
            .output()
        {
            Ok(output) => {
                let pass_output = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                );
                let needs_rerun = pass_output.contains("Rerun to get outlines right")
                    || pass_output.contains("Label(s) may have changed")
                    || pass_output.contains("Rerun to get cross-references right");
                combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                if output.status.success() {
                    any_success = true;
                    if !needs_rerun {
                        break;
                    }
                }
            }
            Err(err) => {
                if pdf_path.exists() {
                    return (Some(pdf_path), Some(err.to_string()));
                }
                return (None, Some(err.to_string()));
            }
        }
    }

    if any_success && pdf_path.exists() {
        return (Some(pdf_path), None);
    }
    if pdf_path.exists() {
        return (
            Some(pdf_path),
            (!combined_output.trim().is_empty()).then_some(combined_output),
        );
    }
    (
        None,
        (!combined_output.trim().is_empty()).then_some(combined_output),
    )
}

/// Recursively ZIP all files in `run_dir` into `zip_path`.
pub(crate) fn zip_directory(run_dir: &Path, zip_path: &Path) -> Result<()> {
    use zip::write::SimpleFileOptions;
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut stack = vec![run_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(run_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                stack.push(path);
            } else {
                zip.start_file(relative, options)?;
                zip.write_all(&fs::read(path)?)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_root_targets_repo_level_directory() {
        let output_root = default_output_root();
        assert_eq!(
            output_root.file_name().unwrap(),
            "output-dsfb-semiconductor"
        );
    }

    #[test]
    fn timestamped_dir_contains_dataset_suffix() {
        let temp = tempfile::tempdir().unwrap();
        let run_dir = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        let name = run_dir.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with("_secom"));
        assert!(name.contains("dsfb-semiconductor"));
    }

    #[test]
    fn timestamped_dir_never_reuses_existing_directory_name() {
        let temp = tempfile::tempdir().unwrap();
        let first = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        let second = create_timestamped_run_dir(temp.path(), "secom").unwrap();
        assert_ne!(first, second);
    }
}
