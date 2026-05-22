//! S-PERF.7 --- source-report import verifier.
//!
//! ## Commit identity
//!
//! > **S-PERF.7 makes the S-PERF.6 measurement chain
//! > mechanically empirical: the corpus crate now PARSES the
//! > on-disk bench source reports and rejects any drift
//! > between the parsed values and the panel-pinned S-PERF.6
//! > receipt constants.**
//!
//! Before S-PERF.7 the receipt could silently drift away
//! from disk: a hand-edit of the `S_PERF_6_*` const prefix in
//! `s_perf_6_rtx4080_super_measured_cuda_pipeline.rs` would
//! pass every existing test even if the underlying bench
//! report on disk reported a different number. S-PERF.7
//! closes that loop. The parser walks the actual bytes of
//! `reports/d64_stage_timing_256x4096_K1.txt` and
//! `reports/r12_d64_saturation.txt`, extracts the measured
//! values + R.12b episode pins, and the verifier asserts
//! they match the S-PERF.6 receipt field-for-field. Any
//! divergence fires a panel-required negative.
//!
//! ## What this DOES
//!
//! - Parses `reports/d64_stage_timing_256x4096_K1.txt` into
//!   a typed [`ParsedD64StageTimingV1`] struct.
//! - Parses `reports/r12_d64_saturation.txt` (the R.12b
//!   saturation sweep) into a typed [`ParsedR12bSaturationV1`]
//!   struct holding the three episode-count pins
//!   (canonical / mid / full at K=1).
//! - Builds a hashable
//!   [`SourceReportImportVerifierReportV1`] envelope
//!   binding the two parsed reports + the S-PERF.6
//!   baseline-report hash + verifier provenance.
//! - Defines [`verify_source_reports_match_s_perf_6_baseline`]
//!   which rejects any drift via four panel-required
//!   load-bearing negatives.
//!
//! ## What this DOES NOT do
//!
//! - Does NOT run the bench (the corpus crate is host-only;
//!   the bench lives in `dsfb-gpu-debug-cuda`).
//! - Does NOT rewrite source reports.
//! - Does NOT mutate the S-PERF.6 receipt.
//! - Does NOT mutate any prior hash anchor.
//! - Does NOT alter `SEED.len()`.
//! - Does NOT change court state.
//! - Does NOT rebaseline R.12b.
//!
//! ## Hash posture
//!
//! One own-namespace hash:
//!
//! - `source_report_import_verifier_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-7-SOURCE-REPORT-IMPORT-VERIFIER:v1\0`.
//!   Binds the parsed values + verifier provenance + the
//!   S-PERF.6 baseline report hash.
//!
//! ## Track B linkage
//!
//! S-PERF.7 is the first Track B leg. It does not change the
//! measured bandwidth number; it strengthens the measurement
//! chain so subsequent legs (S-PERF.8 batched K, S-PERF.9
//! device-side feature construction, etc.) can ratchet the
//! live measurement upward with the receipt automatically
//! tracking the bench output.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, Rtx4080SuperMeasuredCudaPipelineV1,
    R12B_EPISODE_COUNT_CANONICAL_W16H128, R12B_EPISODE_COUNT_FULL_W256H4096,
    R12B_EPISODE_COUNT_MID_W64H512, S_PERF_6_SOURCE_REPORT_PATH,
};

// ---------------------------------------------------------------
// Domain separator + schema id
// ---------------------------------------------------------------

/// Domain separator for
/// `source_report_import_verifier_hash_v1`.
pub const S_PERF_7_SOURCE_REPORT_IMPORT_VERIFIER_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-7-SOURCE-REPORT-IMPORT-VERIFIER:v1\0";

/// Schema identifier for
/// `source_report_import_verifier_hash_v1`.
pub const S_PERF_7_SOURCE_REPORT_IMPORT_VERIFIER_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-7-SOURCE-REPORT-IMPORT-VERIFIER:v1";

/// Panel-pinned R.12b saturation source-report path. The
/// verifier cites this file alongside the S-PERF.6 source
/// report so any drift in the R.12b episode counts surfaces
/// at receipt-build time.
pub const S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH: &str = "reports/r12_d64_saturation.txt";

// ---------------------------------------------------------------
// ParsedD64StageTimingV1
// ---------------------------------------------------------------

/// Parsed view of `reports/d64_stage_timing_256x4096_K1.txt`.
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `source_report_import_verifier_hash_v1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedD64StageTimingV1 {
    /// Host wall-clock median, microseconds (line:
    /// "Host wall median (incl. host segments): N us").
    pub host_wall_median_us: u64,
    /// Device-side total, microseconds (line:
    /// "Device total_device_us (median): N us").
    pub device_total_us: u64,
    /// Consensus-grid-kernel-wide stage, microseconds
    /// (stage table column).
    pub consensus_grid_kernel_wide_us: u64,
    /// Tree-digest consensus stage, microseconds.
    pub tree_digest_consensus_us: u64,
    /// Host compute_features segment, microseconds
    /// (line: "host: compute_features ... | N us").
    pub host_compute_features_us: u64,
    /// Host bank-admit + case-finalize segment, microseconds.
    pub host_bank_admit_case_finalize_us: u64,
    /// Measured wide bytes/sec converted to centi-GB/s
    /// (parsed from the "wide bytes/sec (264) : X.XX GB/s"
    /// line; 13.33 GB/s -> 1333). Encoded in centi-units so
    /// 2-decimal-place GB/s is representable as a u32.
    pub measured_wide_bandwidth_centi_gbps: u32,
    /// Episode count at the 256x4096 K=1 D64 throughput
    /// scale (the report's `episode_count` line).
    pub episode_count_full_256x4096: u32,
}

// ---------------------------------------------------------------
// ParsedR12bSaturationV1
// ---------------------------------------------------------------

/// Parsed view of the R.12b saturation source report,
/// specifically the three K=1 episode-count pins. Field
/// order is the canonical hash order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedR12bSaturationV1 {
    /// Episode count at the canonical `16 entities x 128
    /// windows` grid, K=1.
    pub episode_count_canonical_w16h128: u32,
    /// Episode count at the mid `64 entities x 512 windows`
    /// grid, K=1.
    pub episode_count_mid_w64h512: u32,
    /// Episode count at the full `256 entities x 4096
    /// windows` grid, K=1.
    pub episode_count_full_w256h4096: u32,
}

// ---------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------

/// Why the parser rejected a source-report text. Each
/// variant identifies the specific line / field that could
/// not be extracted; the verifier uses these as structural
/// guards independent of the four panel-required negatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// `Host wall median (...): N us` line absent.
    MissingHostWallMedian,
    /// `Device total_device_us (median): N us` line absent.
    MissingDeviceTotal,
    /// Stage-table row for `consensus_grid_kernel_wide`
    /// absent.
    MissingConsensusGridKernelWide,
    /// Stage-table row for `tree_digest consensus` absent.
    MissingTreeDigestConsensus,
    /// `host: compute_features ... | N us` line absent.
    MissingHostComputeFeatures,
    /// `host: bank admit + case finalize ... | N us` line
    /// absent.
    MissingHostBankAdmitCaseFinalize,
    /// `wide bytes/sec (264) : X.XX GB/s` line absent.
    MissingWideBandwidth,
    /// `episode_count : N` line absent.
    MissingEpisodeCount,
    /// A numeric field could not be parsed as the declared
    /// integer / fixed-point form.
    MalformedNumber {
        /// Which field failed to parse.
        field: &'static str,
    },
    /// `canonical 16x128 K= 1` episode pin row absent.
    MissingEpisodesCanonicalW16H128,
    /// `mid 64x512 K= 1` episode pin row absent.
    MissingEpisodesMidW64H512,
    /// `full 256x4096 K= 1` episode pin row absent.
    MissingEpisodesFullW256H4096,
}

// ---------------------------------------------------------------
// Parser: d64_stage_timing_256x4096_K1.txt
// ---------------------------------------------------------------

/// WHY: every Track B leg writes new measurement values to
/// `reports/d64_stage_timing_256x4096_K1.txt` via the live
/// bench, and the S-PERF.6 receipt MUST reflect those
/// values. The parser is the only mechanically-enforced
/// bridge; without it the receipt can silently drift away
/// from disk.
///
/// Returns the parsed timings + bandwidth + episode count
/// on success, or a structurally-typed [`ParseError`] on
/// any malformed line. The parser is line-oriented and
/// tolerant of trailing whitespace; the panel-locked output
/// format is fixed and any divergence is intentionally an
/// error rather than a silent fallback.
///
/// # Errors
///
/// Returns [`ParseError`] when any required line is missing
/// or a numeric / fixed-point field is malformed.
pub fn parse_d64_stage_timing(text: &str) -> Result<ParsedD64StageTimingV1, ParseError> {
    let mut host_wall_median_us: Option<u64> = None;
    let mut device_total_us: Option<u64> = None;
    let mut consensus_grid_kernel_wide_us: Option<u64> = None;
    let mut tree_digest_consensus_us: Option<u64> = None;
    let mut host_compute_features_us: Option<u64> = None;
    let mut host_bank_admit_case_finalize_us: Option<u64> = None;
    let mut measured_wide_bandwidth_centi_gbps: Option<u32> = None;
    let mut episode_count_full_256x4096: Option<u32> = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("Host wall median ") {
            // "Host wall median (incl. host segments): 30020 us"
            host_wall_median_us = Some(parse_us_after_colon(line, "host_wall_median_us")?);
        } else if line.starts_with("Device total_device_us ") {
            // "Device total_device_us (median):        20771 us"
            device_total_us = Some(parse_us_after_colon(line, "device_total_us")?);
        } else if line.starts_with("consensus_grid_kernel_wide ") {
            // "consensus_grid_kernel_wide         |       382 |   1.8"
            consensus_grid_kernel_wide_us =
                Some(parse_first_pipe_us(line, "consensus_grid_kernel_wide_us")?);
        } else if line.starts_with("tree_digest consensus ") {
            // "tree_digest consensus              |      4338 |  20.9"
            tree_digest_consensus_us = Some(parse_first_pipe_us(line, "tree_digest_consensus_us")?);
        } else if line.starts_with("host: compute_features") {
            // "host: compute_features              |      7525 us"
            host_compute_features_us =
                Some(parse_pipe_us_with_unit(line, "host_compute_features_us")?);
        } else if line.starts_with("host: bank admit + case finalize") {
            // "host: bank admit + case finalize    |      2237 us"
            host_bank_admit_case_finalize_us = Some(parse_pipe_us_with_unit(
                line,
                "host_bank_admit_case_finalize_us",
            )?);
        } else if line.starts_with("wide bytes/sec") {
            // "wide bytes/sec (264) : 13.33 GB/s"
            measured_wide_bandwidth_centi_gbps = Some(parse_gbps_to_centi(
                line,
                "measured_wide_bandwidth_centi_gbps",
            )?);
        } else if line.starts_with("episode_count ") {
            // "episode_count        : 1917"
            episode_count_full_256x4096 =
                Some(parse_u32_after_colon(line, "episode_count_full_256x4096")?);
        }
    }

    Ok(ParsedD64StageTimingV1 {
        host_wall_median_us: host_wall_median_us.ok_or(ParseError::MissingHostWallMedian)?,
        device_total_us: device_total_us.ok_or(ParseError::MissingDeviceTotal)?,
        consensus_grid_kernel_wide_us: consensus_grid_kernel_wide_us
            .ok_or(ParseError::MissingConsensusGridKernelWide)?,
        tree_digest_consensus_us: tree_digest_consensus_us
            .ok_or(ParseError::MissingTreeDigestConsensus)?,
        host_compute_features_us: host_compute_features_us
            .ok_or(ParseError::MissingHostComputeFeatures)?,
        host_bank_admit_case_finalize_us: host_bank_admit_case_finalize_us
            .ok_or(ParseError::MissingHostBankAdmitCaseFinalize)?,
        measured_wide_bandwidth_centi_gbps: measured_wide_bandwidth_centi_gbps
            .ok_or(ParseError::MissingWideBandwidth)?,
        episode_count_full_256x4096: episode_count_full_256x4096
            .ok_or(ParseError::MissingEpisodeCount)?,
    })
}

// ---------------------------------------------------------------
// Parser: r12_d64_saturation.txt (episode pins only)
// ---------------------------------------------------------------

/// WHY: the R.12b saturation harness records per-scale
/// episode counts at every K. The panel-locked integrity
/// pins (13 / 89 / 1917) are the K=1 values for the three
/// canonical scales. The parser extracts only those three
/// values; the rest of the report is wall-time data that
/// drifts run-to-run (the verifier intentionally ignores
/// wall-time, because pinned-wall-time would force a
/// rebaseline on every thermal-load change).
///
/// Returns the three episode counts on success; rejects on
/// any missing pin via the structural [`ParseError`] variants.
///
/// # Errors
///
/// Returns [`ParseError`] when any of the three K=1
/// episode-count rows (canonical / mid / full) is absent.
pub fn parse_r12b_d64_saturation(text: &str) -> Result<ParsedR12bSaturationV1, ParseError> {
    let mut canonical: Option<u32> = None;
    let mut mid: Option<u32> = None;
    let mut full: Option<u32> = None;

    for raw in text.lines() {
        let line = raw.trim();
        // Match the "Detailed throughput ... episodes/cat=N"
        // K=1 lines for each scale.
        if line.starts_with("canonical 16x128") && line.contains("K=  1") {
            canonical = Some(parse_episodes_per_cat(
                line,
                "episode_count_canonical_w16h128",
            )?);
        } else if line.starts_with("mid 64x512") && line.contains("K=  1") {
            mid = Some(parse_episodes_per_cat(line, "episode_count_mid_w64h512")?);
        } else if line.starts_with("full 256x4096") && line.contains("K=  1") {
            full = Some(parse_episodes_per_cat(
                line,
                "episode_count_full_w256h4096",
            )?);
        }
    }

    Ok(ParsedR12bSaturationV1 {
        episode_count_canonical_w16h128: canonical
            .ok_or(ParseError::MissingEpisodesCanonicalW16H128)?,
        episode_count_mid_w64h512: mid.ok_or(ParseError::MissingEpisodesMidW64H512)?,
        episode_count_full_w256h4096: full.ok_or(ParseError::MissingEpisodesFullW256H4096)?,
    })
}

// ---------------------------------------------------------------
// Parsing primitives
// ---------------------------------------------------------------

/// WHY: the line `Host wall median (...): 30020 us` ends in
/// `N us` after a `:`. We extract the integer after the last
/// `:` and drop the trailing ` us`. Tolerant of any
/// whitespace pattern; deliberately strict on the trailing
/// unit so a future format-change to `ms` or `ns` surfaces
/// as a malformed-number error rather than a silent
/// off-by-1000x bug.
fn parse_us_after_colon(line: &str, field: &'static str) -> Result<u64, ParseError> {
    let after = line
        .rsplit_once(':')
        .map(|(_, rhs)| rhs.trim())
        .ok_or(ParseError::MalformedNumber { field })?;
    let stripped = after
        .strip_suffix(" us")
        .or_else(|| after.strip_suffix("us"))
        .ok_or(ParseError::MalformedNumber { field })?;
    stripped
        .trim()
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// WHY: stage-table rows look like
/// `consensus_grid_kernel_wide | 382 | 1.8`. We split on
/// `|` and take the integer in the first numeric column.
fn parse_first_pipe_us(line: &str, field: &'static str) -> Result<u64, ParseError> {
    let mut parts = line.split('|');
    parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let us_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    us_part
        .trim()
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// WHY: host-segment rows look like
/// `host: compute_features | 7525 us`. Same pipe split,
/// then strip the trailing ` us`.
fn parse_pipe_us_with_unit(line: &str, field: &'static str) -> Result<u64, ParseError> {
    let mut parts = line.split('|');
    parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let us_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let trimmed = us_part.trim();
    let stripped = trimmed
        .strip_suffix(" us")
        .or_else(|| trimmed.strip_suffix("us"))
        .ok_or(ParseError::MalformedNumber { field })?;
    stripped
        .trim()
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// WHY: the bandwidth line is
/// `wide bytes/sec (264) : 13.33 GB/s`. We extract the
/// fixed-point GB/s value after the last `:`, strip the
/// trailing ` GB/s`, and convert to centi-GB/s (multiply by
/// 100). The fixed-point grammar is "X.YY" with exactly
/// two decimal digits; anything else surfaces as a
/// malformed-number error so silent rounding cannot
/// disguise a 7.7 vs 7.70 vs 7.700 ambiguity.
fn parse_gbps_to_centi(line: &str, field: &'static str) -> Result<u32, ParseError> {
    let after = line
        .rsplit_once(':')
        .map(|(_, rhs)| rhs.trim())
        .ok_or(ParseError::MalformedNumber { field })?;
    let stripped = after
        .strip_suffix(" GB/s")
        .ok_or(ParseError::MalformedNumber { field })?;
    let (int_part, frac_part) = stripped
        .trim()
        .split_once('.')
        .ok_or(ParseError::MalformedNumber { field })?;
    if frac_part.len() != 2 {
        return Err(ParseError::MalformedNumber { field });
    }
    let int_v: u32 = int_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })?;
    let frac_v: u32 = frac_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })?;
    int_v
        .checked_mul(100)
        .and_then(|s| s.checked_add(frac_v))
        .ok_or(ParseError::MalformedNumber { field })
}

/// WHY: `episode_count : 1917` has the integer after a `:`.
/// No trailing unit; just parse the integer.
fn parse_u32_after_colon(line: &str, field: &'static str) -> Result<u32, ParseError> {
    let after = line
        .rsplit_once(':')
        .map(|(_, rhs)| rhs.trim())
        .ok_or(ParseError::MalformedNumber { field })?;
    after
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// WHY: R.12b detailed-throughput lines look like
/// `canonical 16x128       K=  1 : cells/sec=...  det_evals/sec=...  episodes/cat=13`.
/// The `episodes/cat=N` is at the end; we split on
/// `episodes/cat=` and parse the integer that follows.
fn parse_episodes_per_cat(line: &str, field: &'static str) -> Result<u32, ParseError> {
    let after = line
        .split_once("episodes/cat=")
        .map(|(_, rhs)| rhs.trim())
        .ok_or(ParseError::MalformedNumber { field })?;
    after
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

// ---------------------------------------------------------------
// SourceReportImportVerifierReportV1
// ---------------------------------------------------------------

/// The top-level S-PERF.7 verifier report. Binds the parsed
/// d64 + r12b reports + verifier provenance + the live
/// S-PERF.6 baseline-report hash so an auditor can prove
/// the verifier was run against the same baseline the
/// corpus crate's S-PERF.6 receipt encodes.
#[derive(Debug, Clone)]
pub struct SourceReportImportVerifierReportV1 {
    /// Human-readable verifier identifier (non-empty).
    pub verifier_id: &'static str,
    /// Path the d64 stage-timing report was read from.
    pub d64_source_report_path: &'static str,
    /// Path the R.12b saturation report was read from.
    pub r12b_source_report_path: &'static str,
    /// Parsed view of the d64 stage timing report.
    pub parsed_d64: ParsedD64StageTimingV1,
    /// Parsed view of the R.12b saturation report's K=1
    /// episode pins.
    pub parsed_r12b: ParsedR12bSaturationV1,
    /// S-PERF.6 baseline report hash (the chain anchor; if
    /// the corpus crate's S-PERF.6 receipt rebaselines the
    /// verifier report rebaselines too, by construction).
    pub s_perf_6_baseline_report_hash: [u8; 32],
    /// `source_report_import_verifier_hash_v1`.
    pub source_report_import_verifier_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verifier error kinds (4 panel-required + structural)
// ---------------------------------------------------------------

/// Why S-PERF.7 rejected a (parsed-source-reports,
/// S-PERF.6 receipt) pair. Four panel-required load-bearing
/// negatives plus structural defect rules covering the
/// non-required stage-timing fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf7VerifyErrorKind {
    /// Panel-required #1. The d64 source report's measured
    /// wide bandwidth (centi-GB/s) does not equal the
    /// S-PERF.6 receipt's pinned value.
    SourceReportBandwidthDiffers {
        /// What the source report carried.
        source_centi_gbps: u32,
        /// What the S-PERF.6 receipt encoded.
        receipt_centi_gbps: u32,
    },
    /// Panel-required #2. The d64 source report's device
    /// total time does not equal the receipt's pinned
    /// value.
    SourceReportDeviceTotalDiffers {
        /// Source report value.
        source_us: u64,
        /// Receipt value.
        receipt_us: u64,
    },
    /// Panel-required #3. A host segment (compute_features
    /// or bank-admit + case-finalize) on disk does not equal
    /// the receipt's pinned value.
    SourceReportHostSegmentDiffers {
        /// Which host segment disagreed.
        which: &'static str,
        /// Source report value.
        source_us: u64,
        /// Receipt value.
        receipt_us: u64,
    },
    /// Panel-required #4. An R.12b episode pin on disk does
    /// not equal the panel-locked (13 / 89 / 1917) tuple.
    R12bEpisodePinsDiffer {
        /// Which pin disagreed.
        which: &'static str,
        /// What the on-disk report says.
        source_count: u32,
        /// What the panel-locked constant says.
        panel_locked: u32,
    },
    /// Structural: tree_digest consensus stage timing drift.
    SourceReportTreeDigestConsensusDiffers {
        /// Source report value.
        source_us: u64,
        /// Receipt value.
        receipt_us: u64,
    },
    /// Structural: consensus_grid_kernel_wide drift.
    SourceReportConsensusGridDiffers {
        /// Source report value.
        source_us: u64,
        /// Receipt value.
        receipt_us: u64,
    },
    /// Structural: host_wall_median drift.
    SourceReportHostWallMedianDiffers {
        /// Source report value.
        source_us: u64,
        /// Receipt value.
        receipt_us: u64,
    },
    /// Structural: episode_count line at the 256x4096 K=1
    /// scale in the d64 report does not equal the
    /// panel-locked full-scale pin.
    SourceReportEpisodeCountDiffersFromFullPin {
        /// Source report value.
        source_count: u32,
        /// Panel-locked value (R12B_EPISODE_COUNT_FULL_W256H4096).
        panel_locked: u32,
    },
    /// Structural: cross-report episode-count drift (the d64
    /// report's `episode_count` line at 256x4096 K=1 must
    /// equal the R.12b `full 256x4096 K=1 episodes/cat=` row).
    CrossReportEpisodeCountInconsistent {
        /// d64 source report value.
        d64_source_count: u32,
        /// R.12b source report value.
        r12b_source_count: u32,
    },
    /// Structural: verifier_id empty.
    VerifierIdEmpty,
    /// Structural: d64 source report path empty.
    D64SourceReportPathEmpty,
    /// Structural: R.12b source report path empty.
    R12bSourceReportPathEmpty,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf7VerifyError {
    /// Error kind (see [`SPerf7VerifyErrorKind`]).
    pub kind: SPerf7VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builder
// ---------------------------------------------------------------

/// Build a [`SourceReportImportVerifierReportV1`] and
/// populate `source_report_import_verifier_hash_v1`. The
/// builder takes pre-parsed values + the upstream
/// [`crate::s_perf_6_rtx4080_super_measured_cuda_pipeline::Rtx4080SuperMeasuredBaselineReportV1`] so the verifier
/// can be exercised in tests without disk I/O.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_source_report_import_verifier_report(
    verifier_id: &'static str,
    d64_source_report_path: &'static str,
    r12b_source_report_path: &'static str,
    parsed_d64: ParsedD64StageTimingV1,
    parsed_r12b: ParsedR12bSaturationV1,
    s_perf_6_baseline_report_hash: [u8; 32],
) -> SourceReportImportVerifierReportV1 {
    let mut r = SourceReportImportVerifierReportV1 {
        verifier_id,
        d64_source_report_path,
        r12b_source_report_path,
        parsed_d64,
        parsed_r12b,
        s_perf_6_baseline_report_hash,
        source_report_import_verifier_hash_v1: [0u8; 32],
    };
    r.source_report_import_verifier_hash_v1 = compute_source_report_import_verifier_hash(&r);
    r
}

// ---------------------------------------------------------------
// Seed (live disk reads)
// ---------------------------------------------------------------

/// Build the panel-pinned S-PERF.7 verifier report by
/// reading the two on-disk source reports + composing
/// against the live S-PERF.6 baseline. Returns an error if
/// either file is missing or if either parser rejects.
///
/// Path discipline: the source-report paths are resolved
/// relative to the repository root, which is taken to be
/// the parent of the current crate's `CARGO_MANIFEST_DIR`
/// minus the `crates/dsfb-gpu-atlas-corpus` suffix.
///
/// # Errors
///
/// Returns [`SeedError`] if either source-report file is
/// missing or unreadable, or if either parser fails.
pub fn seed_source_report_import_verifier_report_from_disk(
    repo_root: &std::path::Path,
) -> Result<SourceReportImportVerifierReportV1, SeedError> {
    let d64_path = repo_root.join(S_PERF_6_SOURCE_REPORT_PATH);
    let r12b_path = repo_root.join(S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH);
    let d64_text = std::fs::read_to_string(&d64_path).map_err(|e| SeedError::ReadD64 {
        path: d64_path.display().to_string(),
        message: e.to_string(),
    })?;
    let r12b_text = std::fs::read_to_string(&r12b_path).map_err(|e| SeedError::ReadR12b {
        path: r12b_path.display().to_string(),
        message: e.to_string(),
    })?;
    let parsed_d64 = parse_d64_stage_timing(&d64_text).map_err(SeedError::ParseD64)?;
    let parsed_r12b = parse_r12b_d64_saturation(&r12b_text).map_err(SeedError::ParseR12b)?;
    let baseline = seed_rtx4080_super_measured_baseline_report();
    Ok(build_source_report_import_verifier_report(
        "s_perf_7_source_report_import_verifier_v1",
        S_PERF_6_SOURCE_REPORT_PATH,
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        parsed_d64,
        parsed_r12b,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
    ))
}

/// Why [`seed_source_report_import_verifier_report_from_disk`]
/// failed. Surfaces filesystem errors and parser errors
/// distinctly so a caller (CLI, test harness) can show the
/// operator exactly which source report is broken.
#[derive(Debug)]
pub enum SeedError {
    /// Could not read the d64 stage-timing source report.
    ReadD64 {
        /// Resolved path.
        path: String,
        /// Underlying I/O error message.
        message: String,
    },
    /// Could not read the R.12b saturation source report.
    ReadR12b {
        /// Resolved path.
        path: String,
        /// Underlying I/O error message.
        message: String,
    },
    /// Parser rejected the d64 source report text.
    ParseD64(ParseError),
    /// Parser rejected the R.12b source report text.
    ParseR12b(ParseError),
}

// ---------------------------------------------------------------
// Hash builder
// ---------------------------------------------------------------

/// WHY: serialises every parsed field plus verifier
/// provenance plus the upstream S-PERF.6 baseline-report
/// hash into a canonical byte buffer and SHA-256s the
/// result so two builds against the same source-report
/// text plus same S-PERF.6 baseline produce byte-identical
/// `source_report_import_verifier_hash_v1`. Field order is
/// locked; any reordering rebases the hash.
fn compute_source_report_import_verifier_hash(r: &SourceReportImportVerifierReportV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_7_SOURCE_REPORT_IMPORT_VERIFIER_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S_PERF_7_SOURCE_REPORT_IMPORT_VERIFIER_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.verifier_id.as_bytes());
    push_len_prefixed(&mut buf, r.d64_source_report_path.as_bytes());
    push_len_prefixed(&mut buf, r.r12b_source_report_path.as_bytes());
    // Parsed d64 fields in canonical order.
    buf.extend_from_slice(&r.parsed_d64.host_wall_median_us.to_be_bytes());
    buf.extend_from_slice(&r.parsed_d64.device_total_us.to_be_bytes());
    buf.extend_from_slice(&r.parsed_d64.consensus_grid_kernel_wide_us.to_be_bytes());
    buf.extend_from_slice(&r.parsed_d64.tree_digest_consensus_us.to_be_bytes());
    buf.extend_from_slice(&r.parsed_d64.host_compute_features_us.to_be_bytes());
    buf.extend_from_slice(&r.parsed_d64.host_bank_admit_case_finalize_us.to_be_bytes());
    buf.extend_from_slice(
        &r.parsed_d64
            .measured_wide_bandwidth_centi_gbps
            .to_be_bytes(),
    );
    buf.extend_from_slice(&r.parsed_d64.episode_count_full_256x4096.to_be_bytes());
    // Parsed R.12b episode pins in canonical order.
    buf.extend_from_slice(&r.parsed_r12b.episode_count_canonical_w16h128.to_be_bytes());
    buf.extend_from_slice(&r.parsed_r12b.episode_count_mid_w64h512.to_be_bytes());
    buf.extend_from_slice(&r.parsed_r12b.episode_count_full_w256h4096.to_be_bytes());
    // Anchor: bind to the live S-PERF.6 baseline.
    buf.extend_from_slice(&r.s_perf_6_baseline_report_hash);
    sha256(&buf)
}

/// WHY: prepends a big-endian u32 length so two different
/// strings cannot hash to the same buffer just because
/// their concatenation aliases.
fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify that the parsed source reports match the
/// panel-pinned S-PERF.6 receipt + the panel-locked R.12b
/// episode-count integrity rule. Returns the list of
/// errors (empty when every panel-required negative + every
/// structural rule passes).
///
/// The four panel-required load-bearing negatives are:
///
///  1. `SourceReportBandwidthDiffers`
///  2. `SourceReportDeviceTotalDiffers`
///  3. `SourceReportHostSegmentDiffers` (fires on either
///     host_compute_features or host_bank_admit_case_finalize)
///  4. `R12bEpisodePinsDiffer` (fires on any of the three
///     R.12b pins disagreeing with 13 / 89 / 1917)
///
/// Plus structural rules for non-required stage timings and
/// the cross-report episode-count consistency check.
#[must_use]
#[allow(clippy::too_many_lines)] // 4 panel-required negatives + 8 structural rules
pub fn verify_source_reports_match_s_perf_6_baseline(
    report: &SourceReportImportVerifierReportV1,
    receipt: &Rtx4080SuperMeasuredCudaPipelineV1,
) -> Vec<SPerf7VerifyError> {
    let mut errors: Vec<SPerf7VerifyError> = Vec::new();

    if report.verifier_id.is_empty() {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::VerifierIdEmpty,
        });
    }
    if report.d64_source_report_path.is_empty() {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::D64SourceReportPathEmpty,
        });
    }
    if report.r12b_source_report_path.is_empty() {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::R12bSourceReportPathEmpty,
        });
    }

    let p = &report.parsed_d64;
    let r = &report.parsed_r12b;

    // Panel-required #1: bandwidth coherence.
    if p.measured_wide_bandwidth_centi_gbps != receipt.measured_wide_bandwidth_centi_gbps {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportBandwidthDiffers {
                source_centi_gbps: p.measured_wide_bandwidth_centi_gbps,
                receipt_centi_gbps: receipt.measured_wide_bandwidth_centi_gbps,
            },
        });
    }

    // Panel-required #2: device total coherence.
    if p.device_total_us != receipt.device_total_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportDeviceTotalDiffers {
                source_us: p.device_total_us,
                receipt_us: receipt.device_total_us,
            },
        });
    }

    // Panel-required #3: host segment coherence (both
    // segments checked; a single mismatch fires the
    // negative).
    if p.host_compute_features_us != receipt.host_compute_features_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportHostSegmentDiffers {
                which: "host_compute_features_us",
                source_us: p.host_compute_features_us,
                receipt_us: receipt.host_compute_features_us,
            },
        });
    }
    if p.host_bank_admit_case_finalize_us != receipt.host_bank_admit_case_finalize_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportHostSegmentDiffers {
                which: "host_bank_admit_case_finalize_us",
                source_us: p.host_bank_admit_case_finalize_us,
                receipt_us: receipt.host_bank_admit_case_finalize_us,
            },
        });
    }

    // Panel-required #4: R.12b episode-pin coherence
    // against panel-locked (13 / 89 / 1917).
    if r.episode_count_canonical_w16h128 != R12B_EPISODE_COUNT_CANONICAL_W16H128 {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_canonical_w16h128",
                source_count: r.episode_count_canonical_w16h128,
                panel_locked: R12B_EPISODE_COUNT_CANONICAL_W16H128,
            },
        });
    }
    if r.episode_count_mid_w64h512 != R12B_EPISODE_COUNT_MID_W64H512 {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_mid_w64h512",
                source_count: r.episode_count_mid_w64h512,
                panel_locked: R12B_EPISODE_COUNT_MID_W64H512,
            },
        });
    }
    if r.episode_count_full_w256h4096 != R12B_EPISODE_COUNT_FULL_W256H4096 {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::R12bEpisodePinsDiffer {
                which: "episode_count_full_w256h4096",
                source_count: r.episode_count_full_w256h4096,
                panel_locked: R12B_EPISODE_COUNT_FULL_W256H4096,
            },
        });
    }

    // Structural: tree_digest consensus coherence.
    if p.tree_digest_consensus_us != receipt.tree_digest_consensus_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportTreeDigestConsensusDiffers {
                source_us: p.tree_digest_consensus_us,
                receipt_us: receipt.tree_digest_consensus_us,
            },
        });
    }
    if p.consensus_grid_kernel_wide_us != receipt.consensus_grid_kernel_wide_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportConsensusGridDiffers {
                source_us: p.consensus_grid_kernel_wide_us,
                receipt_us: receipt.consensus_grid_kernel_wide_us,
            },
        });
    }
    if p.host_wall_median_us != receipt.host_wall_median_us {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportHostWallMedianDiffers {
                source_us: p.host_wall_median_us,
                receipt_us: receipt.host_wall_median_us,
            },
        });
    }

    // Structural: cross-report episode-count consistency.
    if p.episode_count_full_256x4096 != R12B_EPISODE_COUNT_FULL_W256H4096 {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::SourceReportEpisodeCountDiffersFromFullPin {
                source_count: p.episode_count_full_256x4096,
                panel_locked: R12B_EPISODE_COUNT_FULL_W256H4096,
            },
        });
    }
    if p.episode_count_full_256x4096 != r.episode_count_full_w256h4096 {
        errors.push(SPerf7VerifyError {
            kind: SPerf7VerifyErrorKind::CrossReportEpisodeCountInconsistent {
                d64_source_count: p.episode_count_full_256x4096,
                r12b_source_count: r.episode_count_full_w256h4096,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// WHY: emits the verifier report as deterministic ASCII so
/// the on-disk artifact is byte-stable across two consecutive
/// builds and operator-legible.
#[must_use]
pub fn render_source_report_import_verifier_report_text(
    r: &SourceReportImportVerifierReportV1,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.7 SourceReportImportVerifierReportV1");
    let _ = writeln!(s, "============================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Verifier provenance");
    let _ = writeln!(s, "  verifier_id              : {}", r.verifier_id);
    let _ = writeln!(
        s,
        "  d64_source_report_path   : {}",
        r.d64_source_report_path
    );
    let _ = writeln!(
        s,
        "  r12b_source_report_path  : {}",
        r.r12b_source_report_path
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Parsed d64 stage timing (live disk read)");
    let _ = writeln!(
        s,
        "  host_wall_median_us              : {}",
        r.parsed_d64.host_wall_median_us
    );
    let _ = writeln!(
        s,
        "  device_total_us                  : {}",
        r.parsed_d64.device_total_us
    );
    let _ = writeln!(
        s,
        "  consensus_grid_kernel_wide_us    : {}",
        r.parsed_d64.consensus_grid_kernel_wide_us
    );
    let _ = writeln!(
        s,
        "  tree_digest_consensus_us         : {}",
        r.parsed_d64.tree_digest_consensus_us
    );
    let _ = writeln!(
        s,
        "  host_compute_features_us         : {}",
        r.parsed_d64.host_compute_features_us
    );
    let _ = writeln!(
        s,
        "  host_bank_admit_case_finalize_us : {}",
        r.parsed_d64.host_bank_admit_case_finalize_us
    );
    let _ = writeln!(
        s,
        "  measured_wide_bandwidth_centi_gbps : {}",
        r.parsed_d64.measured_wide_bandwidth_centi_gbps
    );
    let _ = writeln!(
        s,
        "  episode_count_full_256x4096        : {}",
        r.parsed_d64.episode_count_full_256x4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Parsed R.12b saturation episode pins (live disk read)");
    let _ = writeln!(
        s,
        "  episode_count_canonical_w16h128  : {}",
        r.parsed_r12b.episode_count_canonical_w16h128
    );
    let _ = writeln!(
        s,
        "  episode_count_mid_w64h512        : {}",
        r.parsed_r12b.episode_count_mid_w64h512
    );
    let _ = writeln!(
        s,
        "  episode_count_full_w256h4096     : {}",
        r.parsed_r12b.episode_count_full_w256h4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Upstream anchor");
    let _ = writeln!(
        s,
        "  s_perf_6_baseline_report_hash : {}",
        hex32(&r.s_perf_6_baseline_report_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "source_report_import_verifier_hash_v1 : {}",
        hex32(&r.source_report_import_verifier_hash_v1)
    );
    s
}

/// WHY: deterministic JSON form for machine consumers.
#[must_use]
pub fn render_source_report_import_verifier_report_json(
    r: &SourceReportImportVerifierReportV1,
) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        S_PERF_7_SOURCE_REPORT_IMPORT_VERIFIER_SCHEMA_V1,
    );
    s.push(',');
    json_field(&mut s, "verifier_id", r.verifier_id);
    s.push(',');
    json_field(&mut s, "d64_source_report_path", r.d64_source_report_path);
    s.push(',');
    json_field(&mut s, "r12b_source_report_path", r.r12b_source_report_path);
    s.push(',');
    let _ = write!(
        s,
        "\"host_wall_median_us\":{}",
        r.parsed_d64.host_wall_median_us
    );
    s.push(',');
    let _ = write!(s, "\"device_total_us\":{}", r.parsed_d64.device_total_us);
    s.push(',');
    let _ = write!(
        s,
        "\"consensus_grid_kernel_wide_us\":{}",
        r.parsed_d64.consensus_grid_kernel_wide_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"tree_digest_consensus_us\":{}",
        r.parsed_d64.tree_digest_consensus_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_compute_features_us\":{}",
        r.parsed_d64.host_compute_features_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_bank_admit_case_finalize_us\":{}",
        r.parsed_d64.host_bank_admit_case_finalize_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"measured_wide_bandwidth_centi_gbps\":{}",
        r.parsed_d64.measured_wide_bandwidth_centi_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"episode_count_full_256x4096\":{}",
        r.parsed_d64.episode_count_full_256x4096
    );
    s.push(',');
    let _ = write!(
        s,
        "\"episode_count_canonical_w16h128\":{}",
        r.parsed_r12b.episode_count_canonical_w16h128
    );
    s.push(',');
    let _ = write!(
        s,
        "\"episode_count_mid_w64h512\":{}",
        r.parsed_r12b.episode_count_mid_w64h512
    );
    s.push(',');
    let _ = write!(
        s,
        "\"episode_count_full_w256h4096\":{}",
        r.parsed_r12b.episode_count_full_w256h4096
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_6_baseline_report_hash",
        &r.s_perf_6_baseline_report_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "source_report_import_verifier_hash_v1",
        &r.source_report_import_verifier_hash_v1,
    );
    s.push('}');
    s
}

/// WHY: small JSON helpers; no serde dependency per the
/// host-only zero-dep corpus discipline.
fn json_field(s: &mut String, k: &str, v: &str) {
    let _ = write!(s, "\"{k}\":");
    json_string(s, v);
}

fn json_string(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

fn json_hex(s: &mut String, k: &str, v: &[u8; 32]) {
    let _ = write!(s, "\"{k}\":\"{}\"", hex32(v));
}

fn hex32(v: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for &b in v {
        let _ = write!(s, "{b:02x}");
    }
    s
}
