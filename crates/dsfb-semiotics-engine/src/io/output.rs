use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
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
