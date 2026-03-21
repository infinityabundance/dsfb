//! Public NASA dataset demo helpers.
//!
//! This module keeps the public-data demo paths, replay exports, and sample-artifact mirroring
//! in one place so the dataset-specific fetch/preprocess scripts can stay small and auditable.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::dashboard::{DashboardReplayEvent, DashboardReplayStream};
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;

/// Trace interface: public NASA dataset demo surfaces are routed through explicit crate paths.
/// TRACE:INTERFACE:PUBDATA-001:PUBLIC_DATASET_PATHS:crate-local demo paths for NASA datasets
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[value(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PublicDatasetKind {
    NasaMilling,
    NasaBearings,
}

impl PublicDatasetKind {
    #[must_use]
    pub fn all() -> [Self; 2] {
        [Self::NasaMilling, Self::NasaBearings]
    }

    #[must_use]
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::NasaMilling => "nasa_milling",
            Self::NasaBearings => "nasa_bearings",
        }
    }

    #[must_use]
    pub fn scenario_id(self) -> &'static str {
        match self {
            Self::NasaMilling => "nasa_milling_public_demo",
            Self::NasaBearings => "nasa_bearings_public_demo",
        }
    }

    #[must_use]
    pub fn as_label(self) -> &'static str {
        match self {
            Self::NasaMilling => "NASA Milling",
            Self::NasaBearings => "NASA Bearings",
        }
    }

    #[must_use]
    pub fn source_url(self) -> &'static str {
        match self {
            Self::NasaMilling => "https://phm-datasets.s3.amazonaws.com/NASA/3.+Milling.zip",
            Self::NasaBearings => "https://phm-datasets.s3.amazonaws.com/NASA/4.+Bearings.zip",
        }
    }

    #[must_use]
    pub fn source_archive_name(self) -> &'static str {
        match self {
            Self::NasaMilling => "nasa_milling.zip",
            Self::NasaBearings => "nasa_bearings.zip",
        }
    }

    #[must_use]
    pub fn source_sha256(self) -> &'static str {
        match self {
            Self::NasaMilling => "bdba8d52ec1a1baab24c2be58480e6ac62508c8cc1f8219f47ebde8fc9ebc474",
            Self::NasaBearings => {
                "21001ac266c465f5d345ec42d7b508c6a6328487fd9d4d7774422dd5ea10ad83"
            }
        }
    }

    #[must_use]
    pub fn crate_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[must_use]
    pub fn tools_fetch_script(self) -> PathBuf {
        Self::crate_root().join("tools/fetch_public_dataset.py")
    }

    #[must_use]
    pub fn tools_preprocess_script(self) -> PathBuf {
        Self::crate_root().join("tools/preprocess_public_dataset.py")
    }

    #[must_use]
    pub fn source_archive_path(self) -> PathBuf {
        Self::crate_root()
            .join("data/public_dataset/source")
            .join(self.source_archive_name())
    }

    #[must_use]
    pub fn raw_summary_path(self) -> PathBuf {
        Self::crate_root()
            .join("data/public_dataset/raw")
            .join(format!("{}_raw_summary.csv", self.as_slug()))
    }

    #[must_use]
    pub fn processed_dir(self) -> PathBuf {
        Self::crate_root()
            .join("data/processed")
            .join(self.as_slug())
    }

    #[must_use]
    pub fn processed_observed_path(self) -> PathBuf {
        self.processed_dir().join("observed.csv")
    }

    #[must_use]
    pub fn processed_predicted_path(self) -> PathBuf {
        self.processed_dir().join("predicted.csv")
    }

    #[must_use]
    pub fn processed_metadata_path(self) -> PathBuf {
        self.processed_dir().join("metadata.json")
    }

    #[must_use]
    pub fn generated_root(self) -> PathBuf {
        Self::crate_root()
            .join("artifacts/public_dataset_demo")
            .join(self.as_slug())
            .join("generated")
    }

    #[must_use]
    pub fn latest_root(self) -> PathBuf {
        Self::crate_root()
            .join("artifacts/public_dataset_demo")
            .join(self.as_slug())
            .join("latest")
    }

    #[must_use]
    pub fn sample_root(self) -> PathBuf {
        Self::crate_root()
            .join("examples/public_dataset_demo")
            .join(self.as_slug())
    }

    #[must_use]
    pub fn docs_path(self) -> PathBuf {
        Self::crate_root().join("docs/public_dataset_demo.md")
    }
}

/// Small metadata record for a committed public dataset artifact snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicDatasetArtifactSummary {
    pub schema_version: String,
    pub dataset: String,
    pub dataset_label: String,
    pub source_url: String,
    pub source_archive: String,
    pub raw_summary_csv: String,
    pub processed_observed_csv: String,
    pub processed_predicted_csv: String,
    pub replay_events_csv: String,
    pub replay_events_json: String,
    pub replay_ascii: String,
    pub manifest_json: String,
    pub report_pdf: String,
    pub zip_archive: String,
    pub first_png: Option<String>,
}

/// Flattened replay row used for CSV export.
#[derive(Clone, Debug, Serialize)]
pub struct ReplayEventCsvRow {
    pub schema_version: String,
    pub engine_version: String,
    pub bank_version: String,
    pub scenario_id: String,
    pub scenario_title: String,
    pub frame_index: usize,
    pub total_frames: usize,
    pub step: usize,
    pub time: f64,
    pub residual_norm: f64,
    pub drift_norm: f64,
    pub slew_norm: f64,
    pub projection_1: f64,
    pub projection_2: f64,
    pub projection_3: f64,
    pub syntax_label: String,
    pub grammar_state: String,
    pub grammar_margin: f64,
    pub grammar_reason_text: String,
    pub trust_scalar: f64,
    pub semantic_disposition: String,
    pub semantic_candidates: String,
    pub selected_heuristics: String,
    pub admissibility_audit: String,
    pub comparator_alarms: String,
    pub event_markers: String,
    pub event_log: String,
}

impl From<&DashboardReplayEvent> for ReplayEventCsvRow {
    fn from(value: &DashboardReplayEvent) -> Self {
        Self {
            schema_version: value.schema_version.clone(),
            engine_version: value.engine_version.clone(),
            bank_version: value.bank_version.clone(),
            scenario_id: value.scenario_id.clone(),
            scenario_title: value.scenario_title.clone(),
            frame_index: value.frame_index,
            total_frames: value.total_frames,
            step: value.step,
            time: value.time,
            residual_norm: value.residual_norm,
            drift_norm: value.drift_norm,
            slew_norm: value.slew_norm,
            projection_1: value.projection_1,
            projection_2: value.projection_2,
            projection_3: value.projection_3,
            syntax_label: value.syntax_label.clone(),
            grammar_state: value.grammar_state.clone(),
            grammar_margin: value.grammar_margin,
            grammar_reason_text: value.grammar_reason_text.clone(),
            trust_scalar: value.trust_scalar,
            semantic_disposition: value.semantic_disposition.clone(),
            semantic_candidates: value.semantic_candidates.clone(),
            selected_heuristics: value.selected_heuristics.clone(),
            admissibility_audit: value.admissibility_audit.clone(),
            comparator_alarms: value.comparator_alarms.clone(),
            event_markers: value.event_markers.join(" | "),
            event_log: value.event_log.clone(),
        }
    }
}

/// Paths written for replay-specific public dataset exports.
#[derive(Clone, Debug)]
pub struct ReplayArtifactPaths {
    pub replay_events_csv: PathBuf,
    pub replay_events_json: PathBuf,
    pub replay_ascii: PathBuf,
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow!("path {} has no parent directory", path.display()));
    };
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))
}

pub fn clear_dir(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))
}

pub fn mirror_directory(src: &Path, dst: &Path) -> Result<()> {
    clear_dir(dst)?;
    copy_directory_contents(src, dst)
}

fn copy_directory_contents(src: &Path, dst: &Path) -> Result<()> {
    ensure_dir(dst)?;
    for entry in
        fs::read_dir(src).with_context(|| format!("failed to read directory {}", src.display()))?
    {
        let entry = entry.with_context(|| format!("failed to access {}", src.display()))?;
        let source_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_contents(&source_path, &dest_path)?;
        } else {
            fs::copy(&source_path, &dest_path).with_context(|| {
                format!(
                    "failed to copy public dataset artifact {} -> {}",
                    source_path.display(),
                    dest_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn write_replay_artifacts(
    stream: &DashboardReplayStream,
    ascii: &str,
    replay_dir: &Path,
) -> Result<ReplayArtifactPaths> {
    ensure_dir(replay_dir)?;
    let replay_events_csv = replay_dir.join("replay_events.csv");
    let replay_events_json = replay_dir.join("replay_events.json");
    let replay_ascii = replay_dir.join("replay_ascii.txt");

    write_rows(
        &replay_events_csv,
        stream.events.iter().map(ReplayEventCsvRow::from),
    )?;
    write_pretty(&replay_events_json, stream)?;
    fs::write(&replay_ascii, ascii)
        .with_context(|| format!("failed to write {}", replay_ascii.display()))?;

    Ok(ReplayArtifactPaths {
        replay_events_csv,
        replay_events_json,
        replay_ascii,
    })
}

pub fn find_first_png(root: &Path) -> Result<Option<PathBuf>> {
    if !root.exists() {
        return Ok(None);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in
            fs::read_dir(&next).with_context(|| format!("failed to read {}", next.display()))?
        {
            let entry = entry.with_context(|| format!("failed to access {}", next.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension() == Some(OsStr::new("png")) {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}
