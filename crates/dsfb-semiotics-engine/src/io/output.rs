use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Local;

#[derive(Clone, Debug)]
pub struct OutputLayout {
    pub timestamp: String,
    pub run_dir: PathBuf,
    pub figures_dir: PathBuf,
    pub csv_dir: PathBuf,
    pub json_dir: PathBuf,
    pub report_dir: PathBuf,
}

pub fn create_output_layout(output_root: &Path) -> Result<OutputLayout> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;

    loop {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let run_dir = output_root.join(&timestamp);
        match fs::create_dir(&run_dir) {
            Ok(()) => {
                let figures_dir = run_dir.join("figures");
                let csv_dir = run_dir.join("csv");
                let json_dir = run_dir.join("json");
                let report_dir = run_dir.join("report");
                for dir in [&figures_dir, &csv_dir, &json_dir, &report_dir] {
                    fs::create_dir_all(dir)
                        .with_context(|| format!("failed to create {}", dir.display()))?;
                }
                return Ok(OutputLayout {
                    timestamp,
                    run_dir,
                    figures_dir,
                    csv_dir,
                    json_dir,
                    report_dir,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_secs(1));
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to create run directory {}", run_dir.display())
                });
            }
        }
    }
}

pub fn prepare_clean_export_layout(layout: &OutputLayout) -> Result<()> {
    fs::create_dir_all(&layout.run_dir)
        .with_context(|| format!("failed to create {}", layout.run_dir.display()))?;

    for entry in fs::read_dir(&layout.run_dir)
        .with_context(|| format!("failed to read {}", layout.run_dir.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to access {}", layout.run_dir.display()))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if matches!(name.as_ref(), "figures" | "csv" | "json" | "report") {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            } else {
                return Err(anyhow!(
                    "run directory {} contains unexpected subdirectory {}; refusing to mix artifacts with non-engine contents",
                    layout.run_dir.display(),
                    path.display(),
                ));
            }
        } else if path.is_file() {
            if name == "manifest.json" || name.ends_with(".zip") {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            } else {
                return Err(anyhow!(
                    "run directory {} contains unexpected file {}; refusing to export over potentially stale or foreign contents",
                    layout.run_dir.display(),
                    path.display(),
                ));
            }
        }
    }

    for dir in [
        &layout.figures_dir,
        &layout.csv_dir,
        &layout.json_dir,
        &layout.report_dir,
    ] {
        fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    }

    Ok(())
}
