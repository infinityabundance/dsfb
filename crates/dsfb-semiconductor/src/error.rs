// std-only. Excluded from the kernel build by lib.rs.
// Kernel modules use Option or local error enums where error handling is needed.
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DsfbSemiconductorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("dataset format error: {0}")]
    DatasetFormat(String),
    #[error("dataset missing: {dataset} not available at {path}")]
    DatasetMissing {
        dataset: &'static str,
        path: PathBuf,
    },
    #[error("external command failed: {0}")]
    ExternalCommand(String),
    #[error("network fetch failed: {0}")]
    Network(String),
    /// Configuration or validation error — e.g., invalid signature schema.
    #[error("configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, DsfbSemiconductorError>;
