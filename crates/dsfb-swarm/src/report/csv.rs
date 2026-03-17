use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use csv::Writer;
use serde::Serialize;

pub fn write_csv_rows<T>(path: &Path, rows: impl IntoIterator<Item = T>) -> Result<()>
where
    T: Serialize,
{
    let file = File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let mut writer = Writer::from_writer(file);
    for row in rows {
        writer
            .serialize(row)
            .with_context(|| format!("failed to serialize row into {}", path.display()))?;
    }
    writer.flush().with_context(|| format!("failed to flush {}", path.display()))
}
