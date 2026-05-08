//! DSFB-Debug: residual-projection (TSV) fixture parser — v1 + v2.
//!
//! # The fixture format
//!
//! The residual-projection format is a deterministic projection of
//! upstream trace data (Jaeger spans, OTLP exports, KPI time-series,
//! per-module bug counts, etc.) onto a per-window, per-signal
//! residual matrix. Each fixture is hand-extracted from a canonical
//! upstream archive (DOI-pinned, SHA-256-gated) and stored in-tree
//! under `data/fixtures/`. Provenance lives in the fixture header
//! and is cross-referenced in `data/MANIFEST.toml`.
//!
//! Twelve real-bytes vendored fixtures use this format (post Phase
//! G); each is produced by a deterministic Python script under
//! `data/upstream/project_*.py`.
//!
//! # Format (line-oriented, ASCII)
//!
//! - Lines starting with `#` are header comments. The parser
//!   extracts a small set of declared keys from these comments.
//! - Blank lines are ignored.
//! - All other lines are TAB-separated `f64` values, one row per
//!   window. Each row must contain exactly `num_signals` values.
//!
//! # Required header keys (any order, before the data rows)
//!
//! - `# residual-projection v1` or `# residual-projection v2`
//! - `# num_windows=<u32>`
//! - `# num_signals=<u16>`
//! - `# healthy_window_end=<u32>`
//! - `# fault_labels=<comma-separated u32 window indices, may be empty>`
//! - `# upstream_doi=<string>`
//! - `# upstream_archive_sha256=<64-hex>` (or `upstream_sha256` legacy form)
//! - `# extracted_at=<ISO-8601>`
//! - `# license=<SPDX>`
//!
//! # v2 additions (optional)
//!
//! - `# channels=<csv of per-signal labels>` — operator-readable
//!   channel names (e.g.\
//!   `ts-order-service_latency_p50_ms,ts-order-service_error_rate,...`).
//!   Surfaced as `OwnedResidualMatrix.channels: Vec<String>` for
//!   the renderer (`render::render_episode_summary`) to substitute
//!   into dashboard hints.
//!
//! # Sentinel detection
//!
//! A fixture containing the marker string
//! `# UPSTREAM_FIXTURE_NOT_VENDORED` (case-sensitive, anywhere in
//! the bytes) is recognised as a sentinel placeholder. The parser
//! returns `is_sentinel = true`; the harness's
//! `verify_fixture_integrity` returns `DsfbError::MissingRealData`.
//! The crate never falls back to anything synthetic.
//
// Sentinel: a fixture with the comment line `# UPSTREAM_FIXTURE_NOT_VENDORED`
// (no data rows) is treated as an explicit "fixture not yet populated"
// marker. `parse_residual_projection` returns an empty matrix that the
// real-data evaluator detects as `MissingRealData`.

#![cfg(feature = "std")]

extern crate std;

use std::string::{String, ToString};
use std::vec::Vec;

use crate::error::{DsfbError, Result};

#[derive(Debug, Clone)]
pub struct OwnedResidualMatrix {
    /// Row-major [window][signal] f64 values. `data.len() == num_windows * num_signals`.
    pub data: Vec<f64>,
    /// Number of signals per window.
    pub num_signals: usize,
    /// Number of windows.
    pub num_windows: usize,
    /// Boundary index between healthy baseline and evaluation regions.
    pub healthy_window_end: usize,
    /// Per-window fault labels. `fault_labels.len() == num_windows`.
    pub fault_labels: Vec<bool>,
    /// True if the fixture was the sentinel "not yet vendored" form.
    pub is_sentinel: bool,
    /// Verbatim header for downstream inspection / logging.
    pub header_provenance: String,
    /// Multi-modal channel labels — populated from the optional
    /// `# channels=<csv>` header line in residual-projection v2
    /// fixtures. v1 fixtures parse with `channels` empty.
    /// When non-empty, `channels.len() == num_signals` and
    /// `channels[s]` names what `data[w * num_signals + s]` represents
    /// (e.g. `"ts-order-service_latency_p50_ms"`,
    /// `"light-oauth2-oauth2-service_log_volume"`).
    pub channels: Vec<String>,
}

pub fn parse_residual_projection(bytes: &[u8]) -> Result<OwnedResidualMatrix> {
    let text = match core::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return Err(DsfbError::ParseError { record: 0, field: 0 }),
    };

    let mut num_windows: Option<usize> = None;
    let mut num_signals: Option<usize> = None;
    let mut healthy_window_end: Option<usize> = None;
    let mut fault_label_indices: Vec<usize> = Vec::new();
    let mut header_provenance = String::new();
    let mut is_sentinel = false;
    let mut data: Vec<f64> = Vec::new();
    let mut row_count: usize = 0;
    let mut channels: Vec<String> = Vec::new();

    for (line_no, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(comment) = line.strip_prefix('#') {
            let comment = comment.trim();
            header_provenance.push_str(comment);
            header_provenance.push('\n');
            if comment == "UPSTREAM_FIXTURE_NOT_VENDORED" {
                is_sentinel = true;
                continue;
            }
            if let Some((key, value)) = comment.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "num_windows" => {
                        num_windows = value.parse::<usize>().ok();
                    }
                    "num_signals" => {
                        num_signals = value.parse::<usize>().ok();
                    }
                    "healthy_window_end" => {
                        healthy_window_end = value.parse::<usize>().ok();
                    }
                    "fault_labels" => {
                        if !value.is_empty() {
                            for tok in value.split(',') {
                                if let Ok(idx) = tok.trim().parse::<usize>() {
                                    fault_label_indices.push(idx);
                                }
                            }
                        }
                    }
                    "channels" => {
                        if !value.is_empty() {
                            for tok in value.split(',') {
                                channels.push(tok.trim().to_string());
                            }
                        }
                    }
                    _ => {} // ignored — provenance/license/etc.
                }
            }
            continue;
        }

        // Data row.
        if is_sentinel {
            // Sentinel files must contain no data rows.
            return Err(DsfbError::ParseError {
                record: line_no as u64,
                field: 0,
            });
        }

        let n_signals = match num_signals {
            Some(n) if n > 0 => n,
            _ => return Err(DsfbError::ParseError { record: line_no as u64, field: 0 }),
        };

        let mut field_count: u16 = 0;
        for tok in line.split('\t') {
            let tok = tok.trim();
            if tok.is_empty() {
                continue;
            }
            let v: f64 = match tok.parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(DsfbError::ParseError {
                        record: line_no as u64,
                        field: field_count,
                    });
                }
            };
            data.push(v);
            field_count += 1;
        }
        if field_count as usize != n_signals {
            return Err(DsfbError::ParseError {
                record: line_no as u64,
                field: field_count,
            });
        }
        row_count += 1;
    }

    if is_sentinel {
        return Ok(OwnedResidualMatrix {
            data: Vec::new(),
            num_signals: 0,
            num_windows: 0,
            healthy_window_end: 0,
            fault_labels: Vec::new(),
            is_sentinel: true,
            header_provenance,
            channels,
        });
    }

    let num_signals = num_signals.ok_or(DsfbError::InvalidConfig("missing num_signals header"))?;
    let num_windows = num_windows.ok_or(DsfbError::InvalidConfig("missing num_windows header"))?;
    let healthy_window_end = healthy_window_end
        .ok_or(DsfbError::InvalidConfig("missing healthy_window_end header"))?;

    if row_count != num_windows {
        return Err(DsfbError::DimensionMismatch {
            expected: num_windows,
            got: row_count,
        });
    }
    if data.len() != num_windows * num_signals {
        return Err(DsfbError::DimensionMismatch {
            expected: num_windows * num_signals,
            got: data.len(),
        });
    }
    if healthy_window_end > num_windows {
        return Err(DsfbError::InvalidConfig("healthy_window_end > num_windows"));
    }

    let mut fault_labels = std::vec![false; num_windows];
    for idx in fault_label_indices {
        if idx < num_windows {
            fault_labels[idx] = true;
        }
    }

    // If channels metadata was provided, validate shape — channels.len()
    // must match num_signals. Mismatch = malformed fixture; refuse rather
    // than silently dropping the channel labels.
    if !channels.is_empty() && channels.len() != num_signals {
        return Err(DsfbError::DimensionMismatch {
            expected: num_signals,
            got: channels.len(),
        });
    }

    Ok(OwnedResidualMatrix {
        data,
        num_signals,
        num_windows,
        healthy_window_end,
        fault_labels,
        is_sentinel: false,
        header_provenance,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_real_shape_extract() {
        // A 4-window × 2-signal fixture. Each row holds two real-valued
        // residual projections (e.g. p50_latency_ms, error_rate). This is
        // a literal fixed-shape test input — not a generator — exercising
        // the parser invariants. The "real" property comes from upstream
        // fixtures vendored in data/fixtures/, not from this unit test.
        let bytes = b"# residual-projection v1\n\
                      # num_windows=4\n\
                      # num_signals=2\n\
                      # healthy_window_end=2\n\
                      # fault_labels=3\n\
                      # upstream_doi=test-only\n\
                      # license=Apache-2.0\n\
                      100.0\t0.001\n\
                      101.5\t0.002\n\
                      150.0\t0.040\n\
                      155.2\t0.055\n";
        let m = parse_residual_projection(bytes).expect("parse should succeed");
        assert!(!m.is_sentinel);
        assert_eq!(m.num_signals, 2);
        assert_eq!(m.num_windows, 4);
        assert_eq!(m.healthy_window_end, 2);
        assert_eq!(m.data.len(), 8);
        assert!((m.data[0] - 100.0).abs() < 1e-12);
        assert_eq!(m.fault_labels, std::vec![false, false, false, true]);
        // v1 fixtures have no `channels` metadata.
        assert!(m.channels.is_empty(), "v1 fixture must yield empty channels");
    }

    #[test]
    fn parses_v2_channels_header() {
        // residual-projection v2: optional `# channels=<csv>` header line
        // declares per-signal channel semantics. Parser must surface
        // these as `OwnedResidualMatrix.channels`.
        let bytes = b"# residual-projection v2\n\
                      # num_windows=2\n\
                      # num_signals=3\n\
                      # healthy_window_end=1\n\
                      # fault_labels=\n\
                      # channels=svc_a_latency_p50_ms,svc_a_error_rate,svc_a_log_volume\n\
                      # license=Apache-2.0\n\
                      100.0\t0.001\t42.0\n\
                      150.0\t0.040\t73.0\n";
        let m = parse_residual_projection(bytes).expect("v2 parse should succeed");
        assert_eq!(m.channels.len(), 3);
        assert_eq!(m.channels[0], "svc_a_latency_p50_ms");
        assert_eq!(m.channels[1], "svc_a_error_rate");
        assert_eq!(m.channels[2], "svc_a_log_volume");
    }

    #[test]
    fn channels_count_must_match_num_signals() {
        // num_signals=2 but channels declares 3 → DimensionMismatch.
        let bytes = b"# num_windows=1\n\
                      # num_signals=2\n\
                      # healthy_window_end=0\n\
                      # fault_labels=\n\
                      # channels=a,b,c\n\
                      100.0\t0.001\n";
        let r = parse_residual_projection(bytes);
        assert!(matches!(r, Err(DsfbError::DimensionMismatch { .. })),
                "channel count mismatch must surface as DimensionMismatch");
    }

    #[test]
    fn detects_sentinel_fixture() {
        let bytes = b"# residual-projection v1\n\
                      # UPSTREAM_FIXTURE_NOT_VENDORED\n\
                      # extraction_recipe=see data/README.md\n";
        let m = parse_residual_projection(bytes).expect("sentinel parse should succeed");
        assert!(m.is_sentinel);
        assert_eq!(m.num_signals, 0);
        assert_eq!(m.num_windows, 0);
    }

    #[test]
    fn rejects_short_row() {
        let bytes = b"# num_windows=1\n# num_signals=2\n# healthy_window_end=0\n# fault_labels=\n100.0\n";
        assert!(parse_residual_projection(bytes).is_err());
    }
}
