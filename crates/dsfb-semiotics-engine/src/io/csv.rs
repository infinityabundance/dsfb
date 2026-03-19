use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

pub fn write_rows<T: Serialize>(path: &Path, rows: impl IntoIterator<Item = T>) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer
            .serialize(row)
            .with_context(|| format!("failed to serialize row into {}", path.display()))?;
    }
    writer
        .flush()
        .with_context(|| format!("failed to flush {}", path.display()))
}
