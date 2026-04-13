use std::string::String;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DsfbError {
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse float error for field `{field}` with value `{value}`")]
    ParseFloat { field: &'static str, value: String },
}
