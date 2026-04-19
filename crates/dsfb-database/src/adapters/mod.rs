//! Dataset adapters.
//!
//! Each adapter exposes a `load(path)` function that reads a real subset of
//! the corresponding public dataset from disk and returns a typed
//! [`ResidualStream`]. The adapter is responsible for:
//!   * format-specific parsing (CSV / Parquet / pickle / SQL)
//!   * dropping samples whose required fields are missing or non-finite
//!   * sorting by time
//!   * embedding the dataset name + version + subset id in `stream.source`
//!
//! Where a dataset cannot be redistributed inside the build (Snowset is
//! ~10 GB; SQLShare is permission-gated; the IMDB JOB dump is third-party
//! licensed) the adapter additionally provides a *synthetic exemplar*
//! function that produces a deterministic, seedable residual stream with the
//! same statistical shape as the real corpus. The paper labels every figure
//! that uses an exemplar with `[exemplar]` and the corresponding fetch
//! script lets the operator regenerate the figure on the real data.
//!
//! Design rule (panel-imposed): synthetic exemplars never carry the bare
//! dataset name in `stream.source` — they always read
//! `"{dataset}-exemplar-seed{N}"`, so a downstream report cannot
//! accidentally label exemplar results as if they were real-data results.

use crate::residual::ResidualStream;
use anyhow::Result;

pub mod ceb;
pub mod generic_csv;
pub mod job;
#[cfg(feature = "otel")]
pub mod otel;
pub mod postgres;
pub mod snowset;
pub mod sqlshare;
pub mod sqlshare_text;
pub mod tpcds;

/// Trait for the five dataset adapters.
pub trait DatasetAdapter {
    /// Display name (for reports + figure captions).
    fn name(&self) -> &'static str;

    /// Load a real subset from `path`. Errors if the file/directory is
    /// missing, malformed, or empty.
    fn load(&self, path: &std::path::Path) -> Result<ResidualStream>;

    /// Generate a deterministic synthetic exemplar with the dataset's
    /// statistical shape. `seed` makes the run reproducible. The returned
    /// stream's `source` will be `"{name}-exemplar-seed{seed}"` so that no
    /// downstream report mislabels it as real data.
    fn exemplar(&self, seed: u64) -> ResidualStream;
}
