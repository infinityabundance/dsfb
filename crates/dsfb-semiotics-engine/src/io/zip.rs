use std::fs::{self, File};
use std::io::{Seek, Write};
use std::path::Path;

use anyhow::{Context, Result};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

pub fn zip_directory(source_dir: &Path, destination_zip: &Path) -> Result<()> {
    let file = File::create(destination_zip)
        .with_context(|| format!("failed to create {}", destination_zip.display()))?;
    let mut archive = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    add_directory_recursive(
        &mut archive,
        source_dir,
        source_dir,
        destination_zip,
        options,
    )?;
    archive
        .finish()
        .with_context(|| format!("failed to finalize {}", destination_zip.display()))?;
    Ok(())
}

fn add_directory_recursive<W: Write + Seek>(
    archive: &mut zip::ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    destination_zip: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)
        .with_context(|| format!("failed to read {}", current_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to access {}", current_dir.display()))?;
        let path = entry.path();
        if path == destination_zip {
            continue;
        }
        if path.is_dir() {
            add_directory_recursive(archive, base_dir, &path, destination_zip, options)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(base_dir.parent().unwrap_or(base_dir))
                .with_context(|| format!("failed to relativize {}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            archive
                .start_file(relative, options)
                .with_context(|| format!("failed to start zip entry for {}", path.display()))?;
            archive
                .write_all(
                    &fs::read(&path)
                        .with_context(|| format!("failed to read {}", path.display()))?,
                )
                .with_context(|| format!("failed to write zip entry for {}", path.display()))?;
        }
    }
    Ok(())
}
