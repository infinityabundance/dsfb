//! Filesystem layout and timestamped output management.
//!
//! References: `DSFB-06` for deterministic replayability and `CORE-10` for
//! compositional reproducibility of the full audit run.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;

/// Workspace-root output directory name required by the specification.
pub const OUTPUT_ROOT_NAME: &str = "output-dsfb-forensics";

/// Timestamped run directory information.
#[derive(Clone, Debug)]
pub struct RunDirectory {
    /// Workspace root containing the `output-dsfb-forensics` folder.
    pub workspace_root: PathBuf,
    /// Root output folder.
    pub output_root: PathBuf,
    /// Unique run directory for this execution.
    pub run_dir: PathBuf,
    /// Timestamp label in `YYYYMMDD_HHMMSS` format.
    pub timestamp: String,
}

/// Resolve the enclosing workspace root.
///
/// References: `DSFB-06` and `CORE-10`.
pub fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir
        .parent()
        .context("crate manifest must live under a crates directory")?;
    let root = crates_dir
        .parent()
        .context("crate manifest must live two directories below the workspace root")?;
    if !root.join("Cargo.toml").exists() {
        bail!("workspace root {} does not contain Cargo.toml", root.display());
    }
    Ok(root.to_path_buf())
}

/// Create a timestamped run directory under the workspace root.
///
/// References: `DSFB-06` and `CORE-10`.
pub fn create_run_directory() -> Result<RunDirectory> {
    let root = workspace_root()?;
    create_run_directory_at(&root)
}

/// Create a timestamped run directory under an explicit workspace root.
///
/// References: `DSFB-06` and `CORE-10`.
pub fn create_run_directory_at(workspace_root: &Path) -> Result<RunDirectory> {
    let output_root = workspace_root.join(OUTPUT_ROOT_NAME);
    fs::create_dir_all(&output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;
    let (timestamp, run_dir) = create_unique_timestamp_directory(&output_root)?;
    Ok(RunDirectory {
        workspace_root: workspace_root.to_path_buf(),
        output_root,
        run_dir,
        timestamp,
    })
}

/// Write pretty JSON to disk.
///
/// References: `DSFB-06` and `CORE-10`.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let file = File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("failed to write {}", path.display()))
}

/// Write UTF-8 text to disk.
///
/// References: `DSFB-06` and `CORE-10`.
pub fn write_text(path: &Path, body: &str) -> Result<()> {
    let mut file = File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    file.write_all(body.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))
}

fn create_unique_timestamp_directory(output_root: &Path) -> Result<(String, PathBuf)> {
    for _ in 0..5 {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let run_dir = output_root.join(&timestamp);
        match fs::create_dir(&run_dir) {
            Ok(()) => return Ok((timestamp, run_dir)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_secs(1));
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to create {}", run_dir.display()));
            }
        }
    }
    bail!(
        "failed to allocate a unique timestamped directory under {}",
        output_root.display()
    )
}
