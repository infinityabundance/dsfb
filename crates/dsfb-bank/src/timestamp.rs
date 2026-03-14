use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;

#[derive(Debug, Clone)]
pub struct RunDirectory {
    pub output_root: PathBuf,
    pub timestamp: String,
    pub run_dir: PathBuf,
}

pub fn create_timestamped_run_dir(output_root: &Path) -> Result<RunDirectory> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;
    loop {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let run_dir = output_root.join(&timestamp);
        if !run_dir.exists() {
            fs::create_dir_all(&run_dir)
                .with_context(|| format!("failed to create {}", run_dir.display()))?;
            return Ok(RunDirectory {
                output_root: output_root.to_path_buf(),
                timestamp,
                run_dir,
            });
        }
        thread::sleep(Duration::from_millis(1100));
    }
}
