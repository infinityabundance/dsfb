//! S-PERF.11.1 --- post-S-PERF.11 bottleneck triage.
//!
//! ## Panel-locked thesis (verbatim)
//!
//! > S-PERF.11.1 re-profiles the device wall on the committed
//! > post-S-PERF.11 kernel and records the dominant stage
//! > classification + chosen `next_strike` recommendation
//! > under one own-namespace hash. It does not change kernels,
//! > does not claim bandwidth improvement, and does not execute
//! > the next strike.
//!
//! ## Panel-locked decision rule (verbatim)
//!
//! > Question: Did digest remain the dominant wall, or did the
//! > bottleneck move?
//! >
//! > - If digest still dominates: proceed with **S-PERF.12 ---
//! >   CompactDensorDigestV1 throughput mode**.
//! > - If host `compute_features` surfaced: re-rank toward
//! >   **S-PERF.13 --- device-side deterministic feature
//! >   construction**.
//! > - If detector_motif / consensus / candidate collapse
//! >   surfaced: attack the new measured wall instead, and
//! >   defer S-PERF.12 until that stage no longer dominates.
//!
//! ## Panel-locked one-line verdict (verbatim)
//!
//! > S-PERF.11 proves the saturation campaign can move the
//! > scoreboard while preserving deterministic evidence roots;
//! > S-PERF.11.1 re-profiles the device wall and records the
//! > panel-locked next strike under one hashable triage
//! > receipt.
//!
//! ## What this DOES
//!
//! - Parses the live post-S-PERF.11 stage profile from
//!   `reports/d64_stage_timing_256x4096_K1_post_s_perf_11_triage.txt`
//!   (12 device-stage timings + 2 host segments + wide
//!   bytes/sec bandwidth + host_wall + device_total).
//! - Buckets per-stage timings into the 7 panel-locked
//!   categories named in the decision tree (digest /
//!   host_compute_features / detector_motif / consensus /
//!   candidate_collapse / host_bank_admit_finalize / other).
//! - Classifies the dominant bucket and applies the panel-
//!   locked decision tree to emit a `NextStrikeRecommendation`.
//! - Binds parsed timings, classification, recommendation,
//!   triage-run bandwidth, the S-PERF.11 measurement anchor,
//!   and the R.12b episode pin into a single hashable
//!   [`PostRewriteBottleneckTriageReportV1`] envelope.
//!
//! ## What this DOES NOT do
//!
//! - Does NOT change kernels.
//! - Does NOT claim bandwidth improvement.
//! - Does NOT mutate the pinned post-S-PERF.11 source-report
//!   file `reports/d64_stage_timing_256x4096_K1_post_s_perf_11.txt`.
//! - Does NOT mutate any prior S-PERF / T.11 / T.12.x / FF.x /
//!   S1.3.x / T.12.PROV hash anchor.
//! - Does NOT alter `SEED.len()` (stays 54).
//! - Does NOT rebaseline R.12b episodes.
//! - Does NOT execute the next strike (that is the commit
//!   after S-PERF.11.1, named in the triage receipt's
//!   `next_strike_recommendation` field).

use crate::s_perf_11_measured_digest_compaction::{
    seed_bandwidth_delta_report_from_disk, SeedError as SPerf11SeedError,
};
use dsfb_gpu_debug_core::sha256;
use std::fmt::Write as _;

/// Panel-locked pinned bandwidth (13.33 GB/s in centi-GB/s)
/// recorded at S-PERF.11 seal for the pre-rewrite reference.
pub const S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS: u32 = 1333;

/// Panel-locked pinned bandwidth (16.38 GB/s in centi-GB/s)
/// recorded at S-PERF.11 seal for the post-rewrite measured
/// reference (see S-PERF.11 receipt verbatim wording).
pub const S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS: u32 = 1638;

// ---------------------------------------------------------------
// Constants (panel-locked)
// ---------------------------------------------------------------

/// Canonical triage source-report path (panel-locked).
pub const S_PERF_11_1_SOURCE_REPORT_PATH: &str =
    "reports/d64_stage_timing_256x4096_K1_post_s_perf_11_triage.txt";

/// Panel-locked domain separator for the triage META-hash.
const S_PERF_11_1_BOTTLENECK_TRIAGE_DOMAIN: &[u8] =
    b"DSFB-GPU-ATLAS:S-PERF-11-1-BOTTLENECK-TRIAGE:v1\0";

/// Canonical 12 device-stage labels (panel-locked order).
/// MUST match the labels emitted by
/// `r9_c_d64_stage_profile_256x4096_k1` so the parser binds
/// the right rows.
pub const S_PERF_11_1_DEVICE_STAGE_LABELS: [&str; 12] = [
    "h2d (WindowFeature[] H2D)",
    "residual_field_kernel",
    "drift_slew_sign_kernel",
    "detector_motif_kernel_wide_d64",
    "consensus_grid_kernel_wide",
    "axis5_grid_sum_kernel_wide (R.10a)",
    "candidate_collapse_kernel_wide",
    "tree_digest residual",
    "tree_digest sign",
    "tree_digest detector (wide cells)",
    "tree_digest consensus",
    "d2h (candidates+counts+4*32B)",
];

/// Panel-locked R.12b episode pin for the canonical 16x128 K=1
/// fixture. MUST equal the pin asserted by every prior S-PERF
/// receipt.
pub const S_PERF_11_1_R12B_EPISODES_CANONICAL: u32 = 13;
/// Panel-locked R.12b episode pin for the mid 64x512 K=1
/// fixture.
pub const S_PERF_11_1_R12B_EPISODES_MID: u32 = 89;
/// Panel-locked R.12b episode pin for the full 256x4096 K=1
/// fixture.
pub const S_PERF_11_1_R12B_EPISODES_FULL: u32 = 1917;

/// Wire names for `BottleneckCategory` (canonical-bytes
/// projection — MUST stay stable for hash determinism).
const WIRE_DIGEST_STILL_DOMINANT: &str = "DigestStillDominant";
const WIRE_HOST_COMPUTE_FEATURES_SURFACED: &str = "HostComputeFeaturesSurfaced";
const WIRE_DETECTOR_MOTIF_SURFACED: &str = "DetectorMotifSurfaced";
const WIRE_CONSENSUS_SURFACED: &str = "ConsensusSurfaced";
const WIRE_CANDIDATE_COLLAPSE_SURFACED: &str = "CandidateCollapseSurfaced";
const WIRE_HOST_BANK_ADMIT_SURFACED: &str = "HostBankAdmitSurfaced";
const WIRE_OTHER: &str = "Other";

/// Wire names for `NextStrikeRecommendation`.
const WIRE_SPERF12_COMPACT_DENSOR_DIGEST_V1: &str = "SPerf12CompactDensorDigestV1";
const WIRE_SPERF13_DEVICE_SIDE_FEATURE_CONSTRUCTION: &str = "SPerf13DeviceSideFeatureConstruction";
const WIRE_RE_RANK_BEFORE_NEXT_STRIKE: &str = "ReRankBeforeNextStrike";

/// Forbidden bandwidth-improvement-claim substrings
/// (case-insensitive scanner). Triage is a re-profile, not a
/// measurement campaign — it MUST NOT carry any phrasing that
/// claims the triage itself moves the scoreboard.
const FORBIDDEN_BANDWIDTH_CLAIM_PHRASES: &[&str] = &[
    "triage improves bandwidth",
    "triage moves the scoreboard",
    "triage speeds up",
    "triage delivers a speedup",
    "triage demonstrates a bandwidth gain",
    "triage records a bandwidth improvement",
];

// ---------------------------------------------------------------
// Enums
// ---------------------------------------------------------------

/// Panel-locked classification of the dominant post-S-PERF.11
/// device-side stage bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckCategory {
    /// Aggregate of the four `tree_digest` stages still
    /// dominates the device wall (≥ every other bucket).
    DigestStillDominant,
    /// Host `compute_features` segment is the largest single
    /// bucket.
    HostComputeFeaturesSurfaced,
    /// Device `detector_motif_kernel_wide_d64` is the largest
    /// single bucket.
    DetectorMotifSurfaced,
    /// Device `consensus_grid_kernel_wide` is the largest
    /// single bucket (distinct from the digest-consensus
    /// stage; this is the structural consensus reduction
    /// kernel, not its tree-digest of the bytes).
    ConsensusSurfaced,
    /// Device `candidate_collapse_kernel_wide` is the largest
    /// single bucket.
    CandidateCollapseSurfaced,
    /// Host bank admit + case finalize segment is the largest
    /// single bucket.
    HostBankAdmitSurfaced,
    /// Largest bucket is none of the panel-named families
    /// (e.g. h2d / d2h dominates). Forces a re-rank.
    Other,
}

impl BottleneckCategory {
    /// Canonical wire name used for hash bytes + receipts.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::DigestStillDominant => WIRE_DIGEST_STILL_DOMINANT,
            Self::HostComputeFeaturesSurfaced => WIRE_HOST_COMPUTE_FEATURES_SURFACED,
            Self::DetectorMotifSurfaced => WIRE_DETECTOR_MOTIF_SURFACED,
            Self::ConsensusSurfaced => WIRE_CONSENSUS_SURFACED,
            Self::CandidateCollapseSurfaced => WIRE_CANDIDATE_COLLAPSE_SURFACED,
            Self::HostBankAdmitSurfaced => WIRE_HOST_BANK_ADMIT_SURFACED,
            Self::Other => WIRE_OTHER,
        }
    }
}

/// Panel-locked next-strike recommendation. The verifier
/// enforces that the recommendation matches the decision tree
/// applied to `BottleneckCategory`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextStrikeRecommendation {
    /// Digest still dominates → proceed with S-PERF.12 compact
    /// densor digest mode.
    SPerf12CompactDensorDigestV1,
    /// Host `compute_features` surfaced → re-rank toward
    /// S-PERF.13 device-side feature construction.
    SPerf13DeviceSideFeatureConstruction,
    /// Detector / consensus / candidate / host-bank / other
    /// surfaced → re-rank and attack the new wall instead.
    ReRankBeforeNextStrike,
}

impl NextStrikeRecommendation {
    /// Canonical wire name used for hash bytes + receipts.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::SPerf12CompactDensorDigestV1 => WIRE_SPERF12_COMPACT_DENSOR_DIGEST_V1,
            Self::SPerf13DeviceSideFeatureConstruction => {
                WIRE_SPERF13_DEVICE_SIDE_FEATURE_CONSTRUCTION
            }
            Self::ReRankBeforeNextStrike => WIRE_RE_RANK_BEFORE_NEXT_STRIKE,
        }
    }
}

// ---------------------------------------------------------------
// Schema
// ---------------------------------------------------------------

/// One parsed device-stage row. Pairs the canonical label with
/// the live `us` measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriageStageTimingV1 {
    /// Canonical stage label (mirrors one entry of
    /// `S_PERF_11_1_DEVICE_STAGE_LABELS`).
    pub stage_label: &'static str,
    /// Live `us` reading for this stage from the triage
    /// source report.
    pub us: u64,
}

/// Top-level S-PERF.11.1 triage report. Carries the parsed
/// per-stage device + host timings, the triage-run bandwidth,
/// the panel-locked classification + recommendation, the R.12b
/// episode pin, the S-PERF.11 anchor, and the META-hash.
#[derive(Debug, Clone)]
pub struct PostRewriteBottleneckTriageReportV1 {
    /// Panel-locked stable identifier for this triage report.
    pub report_id: &'static str,
    /// Canonical path to the triage source-report file the
    /// per-stage timings were parsed from.
    pub source_report_path: &'static str,

    /// 12 device-stage timings in canonical order (matches
    /// `S_PERF_11_1_DEVICE_STAGE_LABELS`).
    pub device_stages: [TriageStageTimingV1; 12],

    /// Live `host: compute_features` us reading from the
    /// triage source report.
    pub host_compute_features_us: u64,
    /// Live `host: bank admit + case finalize` us reading.
    pub host_bank_admit_case_finalize_us: u64,
    /// Live `Host wall median` us reading.
    pub host_wall_median_us: u64,
    /// Live `Device total_device_us (median)` us reading.
    pub device_total_us: u64,

    /// Live `wide bytes/sec` triage-run bandwidth in
    /// centi-GB/s (e.g. 914 = 9.14 GB/s).
    pub triage_run_bandwidth_centi_gbps: u32,
    /// Panel-locked pinned pre-S-PERF.11 bandwidth
    /// (1333 = 13.33 GB/s) — the reference the rewrite
    /// improved on.
    pub pre_s_perf_11_pinned_bandwidth_centi_gbps: u32,
    /// Panel-locked pinned post-S-PERF.11 bandwidth
    /// (1638 = 16.38 GB/s) — the measured S-PERF.11 result;
    /// triage cannot drift this value without rebaselining
    /// S-PERF.11 itself.
    pub post_s_perf_11_pinned_bandwidth_centi_gbps: u32,

    /// Label of the bucket that won the triage classification
    /// (e.g. `"tree_digest (4-stage aggregate)"`).
    pub dominant_stage_label: &'static str,
    /// `us` value the dominant bucket totalled.
    pub dominant_stage_us: u64,
    /// `dominant_stage_us * 10_000 / device_total_us` (basis
    /// points). Verifier rejects arithmetic mismatch.
    pub dominant_stage_pct_basis_points_of_device_total: u32,
    /// Panel-locked categorical bucket the classifier assigned.
    pub bottleneck_category: BottleneckCategory,
    /// Panel-locked next-strike recommendation produced by
    /// applying the verbatim decision tree to
    /// `bottleneck_category`.
    pub next_strike_recommendation: NextStrikeRecommendation,

    /// R.12b episode count pin for the canonical 16x128
    /// fixture (MUST equal 13).
    pub r12b_episode_count_canonical_w16h128: u32,
    /// R.12b episode count pin for the mid 64x512 fixture
    /// (MUST equal 89).
    pub r12b_episode_count_mid_w64h512: u32,
    /// R.12b episode count pin for the full 256x4096 fixture
    /// (MUST equal 1917).
    pub r12b_episode_count_full_w256h4096: u32,

    /// Anchor binding to the S-PERF.11 measured-result hash
    /// (rebaselined at S-PERF.11 seal to
    /// `1a27154e335c27df6db939d4c8ff0f36f8baf75871be06c750f0853f2268adc8`).
    pub s_perf_11_bandwidth_delta_report_hash_v1: [u8; 32],

    /// Top-level META-hash over every byte-stable field above
    /// under the panel-locked domain
    /// `DSFB-GPU-ATLAS:S-PERF-11-1-BOTTLENECK-TRIAGE:v1\0`.
    pub s_perf_11_1_bottleneck_triage_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Parse errors
// ---------------------------------------------------------------

/// Errors returned by [`parse_post_rewrite_d64_stage_profile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// One of the 12 canonical stage labels is missing from
    /// the triage source report.
    MissingDeviceStage {
        /// Canonical stage label that was not found.
        stage: &'static str,
    },
    /// `Host wall median` line missing.
    MissingHostWallMedian,
    /// `Device total_device_us (median)` line missing.
    MissingDeviceTotalUs,
    /// `host: compute_features` line missing.
    MissingHostComputeFeatures,
    /// `host: bank admit + case finalize` line missing.
    MissingHostBankAdmit,
    /// `wide bytes/sec (264) : X.YY GB/s` line missing or
    /// malformed.
    MissingOrMalformedBandwidth,
    /// Numeric column could not be parsed.
    MalformedNumber {
        /// Symbolic name of the field the malformed value
        /// came from (for operator-legible error reporting).
        field: &'static str,
    },
}

// ---------------------------------------------------------------
// Parser
// ---------------------------------------------------------------

/// WHY: parses the live triage source report (output of
/// `r9_c_d64_stage_profile_256x4096_k1`) into the 12 canonical
/// device-stage timings + 2 host segments + bandwidth +
/// device/host totals. Independent from S-PERF.7 / S-PERF.10
/// parsers (keeps their hash chains byte-identical).
///
/// # Errors
///
/// Returns [`ParseError`] if any canonical stage label is
/// missing, if a numeric column is malformed, or if the wide
/// bytes/sec line is missing.
pub fn parse_post_rewrite_d64_stage_profile(text: &str) -> Result<ParsedTriageProfile, ParseError> {
    let mut device_stages: [Option<u64>; 12] = [None; 12];
    let mut host_compute_features_us: Option<u64> = None;
    let mut host_bank_admit_us: Option<u64> = None;
    let mut host_wall_us: Option<u64> = None;
    let mut device_total_us: Option<u64> = None;
    let mut bandwidth_centi_gbps: Option<u32> = None;

    for raw in text.lines() {
        let line = raw.trim_start();

        if let Some(rest) = line.strip_prefix("Host wall median") {
            host_wall_us = Some(parse_trailing_us(rest, "host_wall_median_us")?);
        } else if let Some(rest) = line.strip_prefix("Device total_device_us") {
            device_total_us = Some(parse_trailing_us(rest, "device_total_us")?);
        } else if line.starts_with("host: compute_features") {
            host_compute_features_us = Some(parse_pipe_host_us(line, "host_compute_features_us")?);
        } else if line.starts_with("host: bank admit + case finalize") {
            host_bank_admit_us = Some(parse_pipe_host_us(
                line,
                "host_bank_admit_case_finalize_us",
            )?);
        } else if let Some(rest) = line.strip_prefix("wide bytes/sec") {
            bandwidth_centi_gbps = Some(parse_wide_bytes_gbps(rest)?);
        } else if line.starts_with("h2d (")
            || line.starts_with("residual_field_kernel")
            || line.starts_with("drift_slew_sign_kernel")
            || line.starts_with("detector_motif_kernel_wide_d64")
            || line.starts_with("consensus_grid_kernel_wide")
            || line.starts_with("axis5_grid_sum_kernel_wide")
            || line.starts_with("candidate_collapse_kernel_wide")
            || line.starts_with("tree_digest residual")
            || line.starts_with("tree_digest sign")
            || line.starts_with("tree_digest detector")
            || line.starts_with("tree_digest consensus")
            || line.starts_with("d2h (")
        {
            for (idx, label) in S_PERF_11_1_DEVICE_STAGE_LABELS.iter().enumerate() {
                if line.starts_with(label) {
                    let us = parse_pipe_us(line, label)?;
                    device_stages[idx] = Some(us);
                    break;
                }
            }
        }
    }

    let mut parsed_stages = [TriageStageTimingV1 {
        stage_label: "",
        us: 0,
    }; 12];
    for (idx, slot) in device_stages.iter().enumerate() {
        let us = slot.ok_or(ParseError::MissingDeviceStage {
            stage: S_PERF_11_1_DEVICE_STAGE_LABELS[idx],
        })?;
        parsed_stages[idx] = TriageStageTimingV1 {
            stage_label: S_PERF_11_1_DEVICE_STAGE_LABELS[idx],
            us,
        };
    }

    Ok(ParsedTriageProfile {
        device_stages: parsed_stages,
        host_compute_features_us: host_compute_features_us
            .ok_or(ParseError::MissingHostComputeFeatures)?,
        host_bank_admit_case_finalize_us: host_bank_admit_us
            .ok_or(ParseError::MissingHostBankAdmit)?,
        host_wall_median_us: host_wall_us.ok_or(ParseError::MissingHostWallMedian)?,
        device_total_us: device_total_us.ok_or(ParseError::MissingDeviceTotalUs)?,
        triage_run_bandwidth_centi_gbps: bandwidth_centi_gbps
            .ok_or(ParseError::MissingOrMalformedBandwidth)?,
    })
}

/// Per-parse output (everything we need to build the receipt
/// envelope).
#[derive(Debug, Clone, Copy)]
pub struct ParsedTriageProfile {
    /// 12 device-stage timings in canonical order.
    pub device_stages: [TriageStageTimingV1; 12],
    /// `host: compute_features` reading (us).
    pub host_compute_features_us: u64,
    /// `host: bank admit + case finalize` reading (us).
    pub host_bank_admit_case_finalize_us: u64,
    /// `Host wall median` reading (us).
    pub host_wall_median_us: u64,
    /// `Device total_device_us (median)` reading (us).
    pub device_total_us: u64,
    /// `wide bytes/sec` reading in centi-GB/s.
    pub triage_run_bandwidth_centi_gbps: u32,
}

/// Parses `... |      6077 |  20.1` style rows and returns
/// the integer `us`.
fn parse_pipe_us(line: &str, field: &'static str) -> Result<u64, ParseError> {
    let mut parts = line.split('|');
    parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let us_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    us_part
        .trim()
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// Parses host pipe-separated lines like
/// `  host: compute_features              |      7821 us`
/// returning the integer microsecond value.
fn parse_pipe_host_us(line: &str, field: &'static str) -> Result<u64, ParseError> {
    let mut parts = line.split('|');
    parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let value_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let trimmed = value_part.trim();
    let n_text = trimmed.split_whitespace().next().unwrap_or("");
    n_text
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// Parses trailing `: <integer> us` (host/device wall lines).
fn parse_trailing_us(rest: &str, field: &'static str) -> Result<u64, ParseError> {
    // Strip any leading "(...)" prose and the colon.
    let after_colon = rest
        .split_once(':')
        .map(|(_, after)| after)
        .ok_or(ParseError::MalformedNumber { field })?;
    let trimmed = after_colon.trim();
    let n_text = trimmed.split_whitespace().next().unwrap_or("");
    n_text
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })
}

/// Parses `(264) : 9.14 GB/s` from the `wide bytes/sec`
/// trailer. Strict 2-decimal grammar mirroring S-PERF.7
/// (rejects 1- or 3-decimal forms so silent rounding cannot
/// disguise a precision drop).
fn parse_wide_bytes_gbps(rest: &str) -> Result<u32, ParseError> {
    let after_colon = rest
        .split_once(':')
        .map(|(_, after)| after)
        .ok_or(ParseError::MissingOrMalformedBandwidth)?;
    let trimmed = after_colon.trim();
    let num_text = trimmed.split_whitespace().next().unwrap_or("");
    let (int_part, frac_part) = num_text
        .split_once('.')
        .ok_or(ParseError::MissingOrMalformedBandwidth)?;
    if frac_part.len() != 2 {
        return Err(ParseError::MissingOrMalformedBandwidth);
    }
    let int_v: u32 = int_part
        .parse::<u32>()
        .map_err(|_| ParseError::MissingOrMalformedBandwidth)?;
    let frac_v: u32 = frac_part
        .parse::<u32>()
        .map_err(|_| ParseError::MissingOrMalformedBandwidth)?;
    Ok(int_v * 100 + frac_v)
}

// ---------------------------------------------------------------
// Classifier + Recommender (panel-locked decision tree)
// ---------------------------------------------------------------

/// WHY: aggregates the four `tree_digest` stages and 6 other
/// panel-named buckets, picks the dominant bucket by `us`
/// descending (canonical tie-break: digest > detector > host
/// compute > consensus > candidate > host bank > other), and
/// emits the matching `BottleneckCategory`.
///
/// Returns `(category, dominant_stage_label, dominant_us)`.
#[must_use]
pub fn classify_dominant_stage(
    profile: &ParsedTriageProfile,
) -> (BottleneckCategory, &'static str, u64) {
    // Device-side bucket sums.
    let digest_total_us = profile.device_stages[7].us  // tree_digest residual
        + profile.device_stages[8].us  // tree_digest sign
        + profile.device_stages[9].us  // tree_digest detector (wide cells)
        + profile.device_stages[10].us; // tree_digest consensus
    let detector_motif_us = profile.device_stages[3].us;
    let consensus_us = profile.device_stages[4].us; // consensus_grid_kernel_wide
    let candidate_collapse_us = profile.device_stages[6].us;
    let host_compute_us = profile.host_compute_features_us;
    let host_bank_us = profile.host_bank_admit_case_finalize_us;

    // h2d + d2h + residual_field + drift_slew_sign + axis5 are
    // the "other" residual bucket.
    let other_us = profile.device_stages[0].us  // h2d
        + profile.device_stages[1].us  // residual_field_kernel
        + profile.device_stages[2].us  // drift_slew_sign_kernel
        + profile.device_stages[5].us  // axis5_grid_sum_kernel_wide
        + profile.device_stages[11].us; // d2h

    // Tie-break order (panel-locked, mirrors the decision-tree
    // ordering): digest first, then device-side, then host-side,
    // then other.
    let buckets: [(BottleneckCategory, &'static str, u64); 7] = [
        (
            BottleneckCategory::DigestStillDominant,
            "tree_digest (4-stage aggregate)",
            digest_total_us,
        ),
        (
            BottleneckCategory::DetectorMotifSurfaced,
            "detector_motif_kernel_wide_d64",
            detector_motif_us,
        ),
        (
            BottleneckCategory::HostComputeFeaturesSurfaced,
            "host: compute_features",
            host_compute_us,
        ),
        (
            BottleneckCategory::ConsensusSurfaced,
            "consensus_grid_kernel_wide",
            consensus_us,
        ),
        (
            BottleneckCategory::CandidateCollapseSurfaced,
            "candidate_collapse_kernel_wide",
            candidate_collapse_us,
        ),
        (
            BottleneckCategory::HostBankAdmitSurfaced,
            "host: bank admit + case finalize",
            host_bank_us,
        ),
        (
            BottleneckCategory::Other,
            "h2d + residual_field + drift_slew_sign + axis5 + d2h",
            other_us,
        ),
    ];

    // Pick the bucket with the largest us; canonical tie-break
    // = first-listed wins.
    let mut winner = buckets[0];
    for &candidate in &buckets[1..] {
        if candidate.2 > winner.2 {
            winner = candidate;
        }
    }
    winner
}

/// WHY: applies the panel-locked decision tree (digest -> S-PERF.12;
/// host_compute_features -> S-PERF.13; everything else -> re-rank
/// and attack the new wall instead).
#[must_use]
pub const fn recommend_next_strike(category: BottleneckCategory) -> NextStrikeRecommendation {
    match category {
        BottleneckCategory::DigestStillDominant => {
            NextStrikeRecommendation::SPerf12CompactDensorDigestV1
        }
        BottleneckCategory::HostComputeFeaturesSurfaced => {
            NextStrikeRecommendation::SPerf13DeviceSideFeatureConstruction
        }
        BottleneckCategory::DetectorMotifSurfaced
        | BottleneckCategory::ConsensusSurfaced
        | BottleneckCategory::CandidateCollapseSurfaced
        | BottleneckCategory::HostBankAdmitSurfaced
        | BottleneckCategory::Other => NextStrikeRecommendation::ReRankBeforeNextStrike,
    }
}

// ---------------------------------------------------------------
// Hash builder
// ---------------------------------------------------------------

/// WHY: canonical-byte projection of the triage report. Field
/// order MUST stay stable. Two builds against the same triage
/// source-report produce byte-identical hashes.
///
/// # Panics
///
/// Panics if `S_PERF_11_1_BOTTLENECK_TRIAGE_DOMAIN`'s len does
/// not fit in `u64` (unreachable for the panel-locked
/// constant).
#[must_use]
pub fn compute_post_rewrite_bottleneck_triage_hash(
    r: &PostRewriteBottleneckTriageReportV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2048);
    buf.extend_from_slice(S_PERF_11_1_BOTTLENECK_TRIAGE_DOMAIN);

    push_len_prefixed_str(&mut buf, r.report_id);
    push_len_prefixed_str(&mut buf, r.source_report_path);

    for stage in &r.device_stages {
        push_len_prefixed_str(&mut buf, stage.stage_label);
        buf.extend_from_slice(&stage.us.to_be_bytes());
    }
    buf.extend_from_slice(&r.host_compute_features_us.to_be_bytes());
    buf.extend_from_slice(&r.host_bank_admit_case_finalize_us.to_be_bytes());
    buf.extend_from_slice(&r.host_wall_median_us.to_be_bytes());
    buf.extend_from_slice(&r.device_total_us.to_be_bytes());

    buf.extend_from_slice(&r.triage_run_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(&r.pre_s_perf_11_pinned_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(&r.post_s_perf_11_pinned_bandwidth_centi_gbps.to_be_bytes());

    push_len_prefixed_str(&mut buf, r.dominant_stage_label);
    buf.extend_from_slice(&r.dominant_stage_us.to_be_bytes());
    buf.extend_from_slice(
        &r.dominant_stage_pct_basis_points_of_device_total
            .to_be_bytes(),
    );
    push_len_prefixed_str(&mut buf, r.bottleneck_category.wire());
    push_len_prefixed_str(&mut buf, r.next_strike_recommendation.wire());

    buf.extend_from_slice(&r.r12b_episode_count_canonical_w16h128.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_mid_w64h512.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_full_w256h4096.to_be_bytes());

    buf.extend_from_slice(&r.s_perf_11_bandwidth_delta_report_hash_v1);

    sha256(&buf)
}

#[expect(
    clippy::expect_used,
    reason = "S_PERF_11_1_BOTTLENECK_TRIAGE_DOMAIN's len is bounded by the panel-locked constant; this expect is unreachable in practice."
)]
fn push_len_prefixed_str(buf: &mut Vec<u8>, s: &str) {
    let len = u64::try_from(s.len()).expect("string length fits in u64");
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

// ---------------------------------------------------------------
// Builder
// ---------------------------------------------------------------

/// WHY: assembles the top-level
/// [`PostRewriteBottleneckTriageReportV1`] from a parsed
/// profile + the S-PERF.11 anchor hash. Computes the dominant
/// classification, the recommendation, and the META-hash.
#[must_use]
pub fn build_post_rewrite_bottleneck_triage_report(
    report_id: &'static str,
    profile: ParsedTriageProfile,
    s_perf_11_bandwidth_delta_report_hash_v1: [u8; 32],
) -> PostRewriteBottleneckTriageReportV1 {
    let (category, dominant_label, dominant_us) = classify_dominant_stage(&profile);
    let recommendation = recommend_next_strike(category);

    let device_total = profile.device_total_us;
    let pct_basis_points = if device_total == 0 {
        0
    } else {
        // basis points = us * 10_000 / device_total, saturating cast to u32.
        u32::try_from((dominant_us as u128 * 10_000 / device_total as u128).min(u32::MAX as u128))
            .unwrap_or(u32::MAX)
    };

    let mut r = PostRewriteBottleneckTriageReportV1 {
        report_id,
        source_report_path: S_PERF_11_1_SOURCE_REPORT_PATH,
        device_stages: profile.device_stages,
        host_compute_features_us: profile.host_compute_features_us,
        host_bank_admit_case_finalize_us: profile.host_bank_admit_case_finalize_us,
        host_wall_median_us: profile.host_wall_median_us,
        device_total_us: profile.device_total_us,
        triage_run_bandwidth_centi_gbps: profile.triage_run_bandwidth_centi_gbps,
        pre_s_perf_11_pinned_bandwidth_centi_gbps: S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS,
        post_s_perf_11_pinned_bandwidth_centi_gbps: S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS,
        dominant_stage_label: dominant_label,
        dominant_stage_us: dominant_us,
        dominant_stage_pct_basis_points_of_device_total: pct_basis_points,
        bottleneck_category: category,
        next_strike_recommendation: recommendation,
        r12b_episode_count_canonical_w16h128: S_PERF_11_1_R12B_EPISODES_CANONICAL,
        r12b_episode_count_mid_w64h512: S_PERF_11_1_R12B_EPISODES_MID,
        r12b_episode_count_full_w256h4096: S_PERF_11_1_R12B_EPISODES_FULL,
        s_perf_11_bandwidth_delta_report_hash_v1,
        s_perf_11_1_bottleneck_triage_hash_v1: [0u8; 32],
    };
    r.s_perf_11_1_bottleneck_triage_hash_v1 = compute_post_rewrite_bottleneck_triage_hash(&r);
    r
}

// ---------------------------------------------------------------
// Live-disk seed
// ---------------------------------------------------------------

/// Errors returned by [`seed_post_rewrite_bottleneck_triage_report_from_disk`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedError {
    /// Could not read the triage source report.
    ReadSourceReport(String),
    /// Triage source report was malformed.
    ParseSourceReport(ParseError),
    /// Could not seed the upstream S-PERF.11 receipt (needed
    /// for its hash binding).
    SeedSPerf11(String),
}

/// WHY: convenience seed that walks the on-disk triage source
/// report, parses it, imports the S-PERF.11 anchor hash, and
/// assembles the top-level report. Returns a fully-pinned
/// `PostRewriteBottleneckTriageReportV1`.
///
/// # Errors
///
/// Returns [`SeedError`] when the triage source report cannot
/// be read or parsed, or when the upstream S-PERF.11 seed
/// fails.
pub fn seed_post_rewrite_bottleneck_triage_report_from_disk(
    repo_root: &std::path::Path,
) -> Result<PostRewriteBottleneckTriageReportV1, SeedError> {
    let triage_path = repo_root.join(S_PERF_11_1_SOURCE_REPORT_PATH);
    let triage_text = std::fs::read_to_string(&triage_path)
        .map_err(|e| SeedError::ReadSourceReport(format!("{}: {e}", triage_path.display())))?;
    let profile =
        parse_post_rewrite_d64_stage_profile(&triage_text).map_err(SeedError::ParseSourceReport)?;

    let s_perf_11 = seed_bandwidth_delta_report_from_disk(repo_root)
        .map_err(|e: SPerf11SeedError| SeedError::SeedSPerf11(format!("{e:?}")))?;

    Ok(build_post_rewrite_bottleneck_triage_report(
        "s_perf_11_1_post_rewrite_bottleneck_triage_v1",
        profile,
        s_perf_11.s_perf_11_bandwidth_delta_report_hash_v1,
    ))
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Panel-locked verifier errors. Six panel-required campaign-
/// identity negatives + 4 structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf11_1VerifyError {
    /// CAMPAIGN IDENTITY: triage cites a zero or stale
    /// S-PERF.11 anchor hash.
    TriageWithoutPostSPerf11Anchor,
    /// Recommendation does not match the dominant stage
    /// classification's bucket sums.
    DecisionWithoutDominantStageEvidence,
    /// Recommendation does not match the panel-locked decision
    /// tree applied to the bottleneck category.
    DecisionInconsistentWithPanelLockedRule,
    /// R.12b episode counts differ from 13 / 89 / 1917.
    TriageWithR12bEpisodeDrift,
    /// `post_s_perf_11_pinned_bandwidth_centi_gbps` differs
    /// from the panel-locked pinned value (1638).
    TriageWithPinnedPostSPerf11BandwidthDrift,
    /// Receipt body contains a forbidden bandwidth-improvement
    /// claim phrase (case-insensitive scan).
    TriageThatClaimsBandwidthImprovement,
    /// Structural: `report_id` is empty.
    ReportIdEmpty,
    /// Structural: `source_report_path` is empty.
    SourceReportPathEmpty,
    /// Structural: dominant-stage basis-points field does not
    /// match `(dominant_us * 10_000) / device_total_us`.
    DominantStagePctArithmeticMismatch,
    /// Structural: device-stage array has a label not matching
    /// `S_PERF_11_1_DEVICE_STAGE_LABELS`.
    PerStageLabelMismatch {
        /// Index in the canonical 12-stage array where the
        /// mismatch was detected.
        at: usize,
        /// Canonical label expected at that index.
        expected: &'static str,
    },
}

/// WHY: runs every panel-required negative + structural rule
/// against the candidate triage report. Returns an empty
/// `Vec` when the report is admissible.
#[must_use]
pub fn verify_post_rewrite_bottleneck_triage_report(
    r: &PostRewriteBottleneckTriageReportV1,
) -> Vec<SPerf11_1VerifyError> {
    let mut errs: Vec<SPerf11_1VerifyError> = Vec::new();

    // Structural.
    if r.report_id.is_empty() {
        errs.push(SPerf11_1VerifyError::ReportIdEmpty);
    }
    if r.source_report_path.is_empty() {
        errs.push(SPerf11_1VerifyError::SourceReportPathEmpty);
    }
    for (idx, stage) in r.device_stages.iter().enumerate() {
        let expected = S_PERF_11_1_DEVICE_STAGE_LABELS[idx];
        if stage.stage_label != expected {
            errs.push(SPerf11_1VerifyError::PerStageLabelMismatch { at: idx, expected });
        }
    }
    if r.device_total_us > 0 {
        let expected_bp = u32::try_from(
            (r.dominant_stage_us as u128 * 10_000 / r.device_total_us as u128)
                .min(u32::MAX as u128),
        )
        .unwrap_or(u32::MAX);
        if r.dominant_stage_pct_basis_points_of_device_total != expected_bp {
            errs.push(SPerf11_1VerifyError::DominantStagePctArithmeticMismatch);
        }
    }

    // CAMPAIGN IDENTITY: non-zero S-PERF.11 anchor required.
    if r.s_perf_11_bandwidth_delta_report_hash_v1 == [0u8; 32] {
        errs.push(SPerf11_1VerifyError::TriageWithoutPostSPerf11Anchor);
    }

    // Decision must match classifier output for the parsed
    // profile.
    let reconstructed_profile = ParsedTriageProfile {
        device_stages: r.device_stages,
        host_compute_features_us: r.host_compute_features_us,
        host_bank_admit_case_finalize_us: r.host_bank_admit_case_finalize_us,
        host_wall_median_us: r.host_wall_median_us,
        device_total_us: r.device_total_us,
        triage_run_bandwidth_centi_gbps: r.triage_run_bandwidth_centi_gbps,
    };
    let (expected_cat, expected_label, expected_us) =
        classify_dominant_stage(&reconstructed_profile);
    if r.bottleneck_category != expected_cat
        || r.dominant_stage_label != expected_label
        || r.dominant_stage_us != expected_us
    {
        errs.push(SPerf11_1VerifyError::DecisionWithoutDominantStageEvidence);
    }

    // Recommendation must match the panel-locked decision tree
    // applied to the (claimed) bottleneck category.
    let expected_rec = recommend_next_strike(r.bottleneck_category);
    if r.next_strike_recommendation != expected_rec {
        errs.push(SPerf11_1VerifyError::DecisionInconsistentWithPanelLockedRule);
    }

    // R.12b episode pins MUST equal the panel-locked counts.
    if r.r12b_episode_count_canonical_w16h128 != S_PERF_11_1_R12B_EPISODES_CANONICAL
        || r.r12b_episode_count_mid_w64h512 != S_PERF_11_1_R12B_EPISODES_MID
        || r.r12b_episode_count_full_w256h4096 != S_PERF_11_1_R12B_EPISODES_FULL
    {
        errs.push(SPerf11_1VerifyError::TriageWithR12bEpisodeDrift);
    }

    // The S-PERF.11 pinned-post bandwidth MUST equal the
    // panel-locked 1638 centi-GB/s. Pinned-pre MUST equal 1333.
    if r.post_s_perf_11_pinned_bandwidth_centi_gbps != S_PERF_11_PINNED_POST_BANDWIDTH_CENTI_GBPS
        || r.pre_s_perf_11_pinned_bandwidth_centi_gbps != S_PERF_11_PINNED_PRE_BANDWIDTH_CENTI_GBPS
    {
        errs.push(SPerf11_1VerifyError::TriageWithPinnedPostSPerf11BandwidthDrift);
    }

    // Forbidden bandwidth-claim scanner (case-insensitive).
    let scan_lower = format!(
        "{} {} {} {}",
        r.report_id,
        r.source_report_path,
        r.dominant_stage_label,
        r.bottleneck_category.wire()
    )
    .to_lowercase();
    for phrase in FORBIDDEN_BANDWIDTH_CLAIM_PHRASES {
        if scan_lower.contains(phrase) {
            errs.push(SPerf11_1VerifyError::TriageThatClaimsBandwidthImprovement);
            break;
        }
    }

    errs
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Renders the triage report in human-readable text form.
/// Byte-stable across two consecutive emits against the same
/// receipt.
#[must_use]
pub fn render_post_rewrite_bottleneck_triage_report_text(
    r: &PostRewriteBottleneckTriageReportV1,
) -> String {
    let mut s = String::with_capacity(2048);
    render_text_header(&mut s, r);
    render_text_stages(&mut s, r);
    render_text_classification(&mut s, r);
    render_text_anchors(&mut s, r);
    s
}

fn render_text_header(s: &mut String, r: &PostRewriteBottleneckTriageReportV1) {
    let _ = writeln!(s, "=== S-PERF.11.1 post-S-PERF.11 bottleneck triage ===");
    let _ = writeln!(s, "report_id            : {}", r.report_id);
    let _ = writeln!(s, "source_report_path   : {}", r.source_report_path);
    let _ = writeln!(s, "host_wall_median_us  : {}", r.host_wall_median_us);
    let _ = writeln!(s, "device_total_us      : {}", r.device_total_us);
    let _ = writeln!(
        s,
        "triage_run_bandwidth : {}.{:02} GB/s",
        r.triage_run_bandwidth_centi_gbps / 100,
        r.triage_run_bandwidth_centi_gbps % 100
    );
    let _ = writeln!(
        s,
        "pre  S-PERF.11 pinned: {}.{:02} GB/s",
        r.pre_s_perf_11_pinned_bandwidth_centi_gbps / 100,
        r.pre_s_perf_11_pinned_bandwidth_centi_gbps % 100
    );
    let _ = writeln!(
        s,
        "post S-PERF.11 pinned: {}.{:02} GB/s",
        r.post_s_perf_11_pinned_bandwidth_centi_gbps / 100,
        r.post_s_perf_11_pinned_bandwidth_centi_gbps % 100
    );
    let _ = writeln!(s);
}

fn render_text_stages(s: &mut String, r: &PostRewriteBottleneckTriageReportV1) {
    let _ = writeln!(s, "Per-stage device timings (us):");
    for stage in &r.device_stages {
        let _ = writeln!(s, "  {:<36} {:>8}", stage.stage_label, stage.us);
    }
    let _ = writeln!(
        s,
        "  {:<36} {:>8}",
        "host: compute_features", r.host_compute_features_us
    );
    let _ = writeln!(
        s,
        "  {:<36} {:>8}",
        "host: bank admit + case finalize", r.host_bank_admit_case_finalize_us
    );
    let _ = writeln!(s);
}

fn render_text_classification(s: &mut String, r: &PostRewriteBottleneckTriageReportV1) {
    let _ = writeln!(s, "Classification (panel-locked):");
    let _ = writeln!(
        s,
        "  dominant_stage_label              : {}",
        r.dominant_stage_label
    );
    let _ = writeln!(
        s,
        "  dominant_stage_us                 : {}",
        r.dominant_stage_us
    );
    let _ = writeln!(
        s,
        "  dominant_stage_pct_basis_points   : {}",
        r.dominant_stage_pct_basis_points_of_device_total
    );
    let _ = writeln!(
        s,
        "  bottleneck_category               : {}",
        r.bottleneck_category.wire()
    );
    let _ = writeln!(
        s,
        "  next_strike_recommendation        : {}",
        r.next_strike_recommendation.wire()
    );
    let _ = writeln!(s);
}

fn render_text_anchors(s: &mut String, r: &PostRewriteBottleneckTriageReportV1) {
    let _ = writeln!(s, "R.12b episode pins (panel-locked, byte-stable):");
    let _ = writeln!(
        s,
        "  canonical_w16h128                 : {}",
        r.r12b_episode_count_canonical_w16h128
    );
    let _ = writeln!(
        s,
        "  mid_w64h512                       : {}",
        r.r12b_episode_count_mid_w64h512
    );
    let _ = writeln!(
        s,
        "  full_w256h4096                    : {}",
        r.r12b_episode_count_full_w256h4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Upstream anchor hashes:");
    let _ = writeln!(
        s,
        "  s_perf_11_bandwidth_delta_report_hash_v1 : {}",
        hex32(&r.s_perf_11_bandwidth_delta_report_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "META-hash:");
    let _ = writeln!(
        s,
        "  s_perf_11_1_bottleneck_triage_hash_v1    : {}",
        hex32(&r.s_perf_11_1_bottleneck_triage_hash_v1)
    );
}

/// Renders the triage report in JSON form. Byte-stable across
/// two consecutive emits against the same receipt; field
/// ordering matches the text renderer for auditor parity.
#[must_use]
pub fn render_post_rewrite_bottleneck_triage_report_json(
    r: &PostRewriteBottleneckTriageReportV1,
) -> String {
    let mut s = String::with_capacity(2048);
    s.push('{');
    let _ = write!(s, "\"report_id\":\"{}\",", r.report_id);
    let _ = write!(s, "\"source_report_path\":\"{}\",", r.source_report_path);
    let _ = write!(s, "\"host_wall_median_us\":{},", r.host_wall_median_us);
    let _ = write!(s, "\"device_total_us\":{},", r.device_total_us);
    let _ = write!(
        s,
        "\"triage_run_bandwidth_centi_gbps\":{},",
        r.triage_run_bandwidth_centi_gbps
    );
    let _ = write!(
        s,
        "\"pre_s_perf_11_pinned_bandwidth_centi_gbps\":{},",
        r.pre_s_perf_11_pinned_bandwidth_centi_gbps
    );
    let _ = write!(
        s,
        "\"post_s_perf_11_pinned_bandwidth_centi_gbps\":{},",
        r.post_s_perf_11_pinned_bandwidth_centi_gbps
    );
    s.push_str("\"device_stages\":[");
    for (idx, stage) in r.device_stages.iter().enumerate() {
        if idx > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            "{{\"label\":\"{}\",\"us\":{}}}",
            stage.stage_label, stage.us
        );
    }
    s.push_str("],");
    let _ = write!(
        s,
        "\"host_compute_features_us\":{},",
        r.host_compute_features_us
    );
    let _ = write!(
        s,
        "\"host_bank_admit_case_finalize_us\":{},",
        r.host_bank_admit_case_finalize_us
    );
    let _ = write!(
        s,
        "\"dominant_stage_label\":\"{}\",",
        r.dominant_stage_label
    );
    let _ = write!(s, "\"dominant_stage_us\":{},", r.dominant_stage_us);
    let _ = write!(
        s,
        "\"dominant_stage_pct_basis_points_of_device_total\":{},",
        r.dominant_stage_pct_basis_points_of_device_total
    );
    let _ = write!(
        s,
        "\"bottleneck_category\":\"{}\",",
        r.bottleneck_category.wire()
    );
    let _ = write!(
        s,
        "\"next_strike_recommendation\":\"{}\",",
        r.next_strike_recommendation.wire()
    );
    let _ = write!(
        s,
        "\"r12b_episode_count_canonical_w16h128\":{},",
        r.r12b_episode_count_canonical_w16h128
    );
    let _ = write!(
        s,
        "\"r12b_episode_count_mid_w64h512\":{},",
        r.r12b_episode_count_mid_w64h512
    );
    let _ = write!(
        s,
        "\"r12b_episode_count_full_w256h4096\":{},",
        r.r12b_episode_count_full_w256h4096
    );
    let _ = write!(
        s,
        "\"s_perf_11_bandwidth_delta_report_hash_v1\":\"{}\",",
        hex32(&r.s_perf_11_bandwidth_delta_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"s_perf_11_1_bottleneck_triage_hash_v1\":\"{}\"",
        hex32(&r.s_perf_11_1_bottleneck_triage_hash_v1)
    );
    s.push('}');
    s
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}
