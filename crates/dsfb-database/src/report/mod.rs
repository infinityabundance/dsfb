//! Report generation: CSV + JSON sidecars + plotter PNGs that the LaTeX
//! paper includes verbatim. Every report header embeds the crate version
//! and the non-claim block so a reviewer can verify provenance.

// `plots` renders PNG figures via `plotters`; it is gated behind the
// `report` feature so library-mode consumers opt out of the full
// figure-rendering toolchain (font-kit, cm-super, etc.). Binaries that
// use `plots` (main, pr_sweep) set `required-features = ["report"]`.
#[cfg(feature = "report")]
pub mod plots;

use crate::grammar::Episode;
use crate::metrics::PerMotifMetrics;
use crate::non_claims;
#[cfg(feature = "report")]
use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ReportHeader {
    pub crate_version: &'static str,
    pub generated_at: String,
    pub non_claims: [&'static str; 5],
    pub source: String,
}

pub fn write_episodes_csv(path: &Path, episodes: &[Episode]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif",
        "channel",
        "t_start",
        "t_end",
        "peak",
        "ema_at_boundary",
        "trust_sum",
    ])?;
    for e in episodes {
        wtr.write_record([
            e.motif.name(),
            e.channel.as_deref().unwrap_or(""),
            &format!("{}", e.t_start),
            &format!("{}", e.t_end),
            &format!("{}", e.peak),
            &format!("{}", e.ema_at_boundary),
            &format!("{}", e.trust_sum),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_metrics_csv(path: &Path, metrics: &[PerMotifMetrics]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "motif",
        "tp",
        "fp",
        "fn",
        "precision",
        "recall",
        "f1",
        "ttd_median_s",
        "ttd_p95_s",
        "false_alarm_per_hour",
        "compression_ratio",
    ])?;
    for m in metrics {
        wtr.write_record([
            &m.motif,
            &m.tp.to_string(),
            &m.fp.to_string(),
            &m.fn_.to_string(),
            &format!("{:.4}", m.precision),
            &format!("{:.4}", m.recall),
            &format!("{:.4}", m.f1),
            &format!("{:.2}", m.time_to_detection_median_s),
            &format!("{:.2}", m.time_to_detection_p95_s),
            &format!("{:.4}", m.false_alarm_rate_per_hour),
            &format!("{:.2}", m.episode_compression_ratio),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

/// JSON sidecar emitter (pretty-printed). Gated behind `report` so the
/// library's default dependency tree does not carry `serde_json`. Main
/// and any binary that writes JSON artefacts must declare
/// `required-features = ["report"]`.
#[cfg(feature = "report")]
pub fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(value)?;
    File::create(path)
        .with_context(|| format!("creating {}", path.display()))?
        .write_all(s.as_bytes())?;
    Ok(())
}

/// Write a free-form text header that every report directory carries.
pub fn write_provenance(path: &Path, source: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    writeln!(f, "DSFB-Database report")?;
    writeln!(f, "crate_version = {}", crate::CRATE_VERSION)?;
    writeln!(f, "source        = {}", source)?;
    writeln!(f)?;
    writeln!(f, "Non-claims:")?;
    writeln!(f, "{}", non_claims::as_block())?;
    Ok(())
}
