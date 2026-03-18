use std::fs::{self, File};
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;
use serde::Serialize;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

pub fn create_timestamped_run_directory(output_root: &Path) -> Result<(String, PathBuf)> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create output root {}", output_root.display()))?;

    loop {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let run_dir = output_root.join(&timestamp);
        match fs::create_dir(&run_dir) {
            Ok(()) => return Ok((timestamp, run_dir)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_secs(1));
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to create run directory {}", run_dir.display()));
            }
        }
    }
}

pub fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let file = File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("failed to serialize {}", path.display()))
}

pub fn write_csv_rows<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut writer =
        csv::Writer::from_path(path).with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer
            .serialize(row)
            .with_context(|| format!("failed to serialize row into {}", path.display()))?;
    }
    writer
        .flush()
        .with_context(|| format!("failed to flush {}", path.display()))
}

pub fn zip_directory(source_dir: &Path, destination_zip: &Path) -> Result<()> {
    let file = File::create(destination_zip)
        .with_context(|| format!("failed to create {}", destination_zip.display()))?;
    let mut archive = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    add_directory_recursive(&mut archive, source_dir, source_dir, options)?;
    archive
        .finish()
        .with_context(|| format!("failed to finalize {}", destination_zip.display()))?;
    Ok(())
}

fn add_directory_recursive<W: Write + Seek>(
    archive: &mut zip::ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)
        .with_context(|| format!("failed to read {}", current_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to access {}", current_dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            add_directory_recursive(archive, base_dir, &path, options)?;
        } else if path.is_file() {
            let relative = relative_name(base_dir, &path)?;
            archive
                .start_file(relative, options)
                .with_context(|| format!("failed to start zip entry for {}", path.display()))?;
            let bytes = fs::read(&path)
                .with_context(|| format!("failed to read file bytes from {}", path.display()))?;
            archive
                .write_all(&bytes)
                .with_context(|| format!("failed to write zip entry for {}", path.display()))?;
        }
    }
    Ok(())
}

fn relative_name(base_dir: &Path, path: &Path) -> Result<String> {
    let parent = base_dir
        .parent()
        .with_context(|| format!("{} has no parent directory", base_dir.display()))?;
    let relative = path
        .strip_prefix(parent)
        .with_context(|| format!("failed to strip prefix for {}", path.display()))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
