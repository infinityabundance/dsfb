use std::fs::{self, File};
use std::path::Path;

use anyhow::{Context, Result};
use csv::Writer;
use serde::Serialize;

pub fn write_csv_rows<T>(path: &Path, rows: &[T]) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let mut writer = Writer::from_writer(file);
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}
