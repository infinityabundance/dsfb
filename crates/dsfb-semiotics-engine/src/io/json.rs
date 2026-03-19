use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

pub fn write_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("failed to write {}", path.display()))
}
