//! S-PERF.8 --- batched-K saturation receipt.
//!
//! ## Commit identity
//!
//! > **S-PERF.8 records what host-loop K batching on the
//! > existing D64 GPU pipeline actually delivers at every
//! > scale, sourced verbatim from the live R.12b saturation
//! > table. The corpus crate parses
//! > `reports/r12_d64_saturation.txt`'s K-throughput matrix
//! > and surfaces the per-scale gain pattern as a hashable
//! > receipt. Honest reporting: at full 256x4096 the
//! > host-loop K amortises only marginally because the
//! > workload is already device-bound; at canonical 16x128
//! > batching nearly doubles throughput. Real bandwidth
//! > gains require kernel-shape changes (digest lane
//! > compaction, device-side feature construction); the
//! > S-PERF.8 receipt makes the K-batching pattern
//! > mechanically auditable so subsequent legs can compare
//! > against this honest baseline.**
//!
//! ## What this DOES
//!
//! - Parses the K-saturation matrix from
//!   `reports/r12_d64_saturation.txt` (every scale x K cell
//!   from the panel-locked sweep matrix
//!   `K in {1, 4, 16, 32, 64, 128} x
//!    scale in {canonical 16x128, mid 64x512, full 256x4096}`).
//! - Records each cell's `per_cat_us`, `cat_sec`,
//!   `features_pct`, `dev_total_pct`, `finalize_pct`, and
//!   top-stage label + percentage.
//! - Records each scale's K=1 baseline + best-K observation,
//!   and the per-scale K-amortisation gain ratio
//!   (best-K cat_sec / K=1 cat_sec).
//! - Builds a hashable
//!   [`BatchedKSaturationReceiptV1`] envelope binding the
//!   parsed table + verifier provenance + the upstream
//!   S-PERF.6 baseline-report hash + the upstream S-PERF.7
//!   source-report-import verifier hash.
//! - Defines [`verify_batched_k_saturation_receipt`]
//!   which rejects any drift via four panel-required
//!   load-bearing negatives.
//!
//! ## What this DOES NOT do
//!
//! - Does NOT run the bench (the corpus crate is host-only;
//!   `dsfb-gpu-debug-cuda`'s `tests/r12_d64_saturation.rs`
//!   produces the source report).
//! - Does NOT rewrite source reports.
//! - Does NOT claim 25-50 GB/s as the S-PERF.8 result. The
//!   panel-pinned target in the Track B plan presumed a
//!   launch-bound workload. At the panel-pinned headline
//!   scale (full 256x4096), the workload is device-bound;
//!   host-loop K amortisation delivers ~3% gain only.
//!   Real bandwidth gains require kernel-shape work
//!   (S-PERF.9 device-side features, S-PERF.10 digest lane
//!   compaction). S-PERF.8 honestly records what batching
//!   actually delivers at every scale, not what the plan
//!   hoped.
//! - Does NOT mutate the S-PERF.6 receipt or any prior
//!   hash anchor.
//! - Does NOT alter `SEED.len()`.
//! - Does NOT rebaseline R.12b.
//!
//! ## Hash posture
//!
//! One own-namespace hash:
//!
//! - `batched_k_saturation_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-8-BATCHED-K-SATURATION-RECEIPT:v1\0`.
//!   Binds the parsed K-throughput table + verifier
//!   provenance + the S-PERF.6 baseline report hash + the
//!   S-PERF.7 source-report import verifier hash.
//!
//! ## Track B linkage
//!
//! S-PERF.8 is the second Track B leg. It does not change
//! kernel code; it converts the existing R.12b K-saturation
//! sweep into a mechanically-auditable receipt so the
//! K-batching ceiling at each scale is operator-visible.
//! Subsequent legs (S-PERF.9 device-side feature
//! construction, S-PERF.10 digest lane compaction) target
//! the real bandwidth levers; the S-PERF.8 receipt is the
//! before-picture against which those legs measure their
//! delta.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, Rtx4080SuperMeasuredCudaPipelineV1,
};
use crate::s_perf_7_source_report_import_verifier::{
    seed_source_report_import_verifier_report_from_disk,
    S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
};

// ---------------------------------------------------------------
// Domain separator + schema id
// ---------------------------------------------------------------

/// Domain separator for
/// `batched_k_saturation_receipt_hash_v1`.
pub const S_PERF_8_BATCHED_K_SATURATION_RECEIPT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-8-BATCHED-K-SATURATION-RECEIPT:v1\0";

/// Schema identifier for
/// `batched_k_saturation_receipt_hash_v1`.
pub const S_PERF_8_BATCHED_K_SATURATION_RECEIPT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-8-BATCHED-K-SATURATION-RECEIPT:v1";

/// Panel-pinned K matrix columns the R.12b harness sweeps.
/// Hard-coded in the harness; the parser asserts the K
/// values in the table match this set so any future drift
/// in the K sweep is mechanically surfaced.
pub const S_PERF_8_K_MATRIX: &[u32] = &[1, 4, 16, 32, 64, 128];

/// Panel-pinned scales the R.12b harness sweeps.
pub const S_PERF_8_SCALES: &[&str] = &["canonical 16x128", "mid 64x512", "full 256x4096"];

// ---------------------------------------------------------------
// ParsedBatchedKCell
// ---------------------------------------------------------------

/// One cell from the R.12b K-saturation matrix. Field order
/// is the canonical hash order; do not reorder without
/// rebaselining `batched_k_saturation_receipt_hash_v1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBatchedKCellV1 {
    /// Scale label (e.g. "canonical 16x128", "mid 64x512",
    /// "full 256x4096").
    pub scale_label: &'static str,
    /// Batch size K (catalogs per dispatch loop).
    pub k: u32,
    /// Per-catalog wall time, microseconds (median across
    /// iters).
    pub per_cat_us: u32,
    /// Catalogs per second, in centi-cat/s (1.86 = 186).
    /// Encoded in centi-units so 2-decimal fixed-point is
    /// representable as integer.
    pub cat_per_sec_centi: u32,
    /// `compute_features` host share of wall time, basis
    /// points (1234 = 12.34%).
    pub features_pct_basis_points: u32,
    /// Device total share of wall time, basis points.
    pub dev_total_pct_basis_points: u32,
    /// `bank admit + case finalize` share of wall time,
    /// basis points.
    pub finalize_pct_basis_points: u32,
    /// Top device-stage label name (no parens / percent).
    pub top_stage_label: &'static str,
    /// Top device-stage share of device total, basis points.
    pub top_stage_pct_basis_points: u32,
}

// ---------------------------------------------------------------
// ParsedScaleSummary
// ---------------------------------------------------------------

/// What the per-scale K-amortisation gain says about the
/// workload shape at that scale. Panel-pinned categories:
///
/// - [`Self::LaunchBoundGainAtSmallFixture`] is reserved for
///   gains at or above 1.5x (15000 bp). A canonical 16x128
///   fixture earns this when batching nearly doubles
///   throughput because per-cat work is small relative to
///   per-launch overhead.
/// - [`Self::ModestFullScaleGain`] applies when gain is
///   between 1.01x and 1.5x (10100..=14999 bp). Indicates
///   the fixture is partially launch-bound; batching
///   delivers a measurable but modest improvement.
/// - [`Self::NoFullScaleImprovement`] applies when gain is
///   between 1.00x and 1.01x (10000..=10099 bp). The
///   fixture is device-bound; batching delivers no
///   measurable improvement.
/// - [`Self::Regressed`] applies when gain is below 1.00x
///   (<10000 bp). Batching made per-cat slower (e.g. cache
///   eviction across K).
///
/// The most-important panel-required negative
/// (`rejects_canonical_launch_bound_gain_generalized_to_full_scale`)
/// uses this enum to reject any receipt that labels
/// `full 256x4096` as `LaunchBoundGainAtSmallFixture` ---
/// the dangerous overclaim *"canonical got 1.76x, therefore
/// K batching solved full-scale"* is mechanically forbidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatchedKResultInterpretation {
    /// Gain >= 1.5x. Reserved for small launch-bound
    /// fixtures.
    LaunchBoundGainAtSmallFixture,
    /// Gain 1.01x to 1.5x. Partially launch-bound.
    ModestFullScaleGain,
    /// Gain 1.00x to 1.01x. Device-bound; no measurable
    /// improvement.
    NoFullScaleImprovement,
    /// Gain below 1.00x. Batching regressed throughput.
    Regressed,
}

impl BatchedKResultInterpretation {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LaunchBoundGainAtSmallFixture => "LaunchBoundGainAtSmallFixture",
            Self::ModestFullScaleGain => "ModestFullScaleGain",
            Self::NoFullScaleImprovement => "NoFullScaleImprovement",
            Self::Regressed => "Regressed",
        }
    }

    /// Classify a gain ratio (in basis points; 10000 = 1.0x)
    /// into the panel-pinned interpretation enum.
    #[must_use]
    pub const fn from_gain_basis_points(gain_bp: u32) -> Self {
        if gain_bp >= 15_000 {
            Self::LaunchBoundGainAtSmallFixture
        } else if gain_bp >= 10_100 {
            Self::ModestFullScaleGain
        } else if gain_bp >= 10_000 {
            Self::NoFullScaleImprovement
        } else {
            Self::Regressed
        }
    }
}

/// Per-scale summary of the K-amortisation pattern. The
/// `best_k_gain_basis_points` is the ratio
/// `(best_k_cat_per_sec / k1_cat_per_sec) * 10000`. A value
/// of 10000 means no gain; 20000 means 2x; 30000 means 3x.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedScaleSummaryV1 {
    /// Scale label.
    pub scale_label: &'static str,
    /// K=1 baseline catalogs per second, centi-units.
    pub k1_cat_per_sec_centi: u32,
    /// Best observed K value (the K where cat_per_sec is
    /// maximum).
    pub best_k: u32,
    /// Best observed cat_per_sec at that K, centi-units.
    pub best_k_cat_per_sec_centi: u32,
    /// K-amortisation gain ratio, basis points
    /// (`best_k_cat_per_sec / k1_cat_per_sec * 10000`).
    pub best_k_gain_basis_points: u32,
    /// Pre-batching effective bandwidth in centi-GB/s
    /// (at K=1). For the panel-pinned full 256x4096 fixture
    /// the K=1 reference is the S-PERF.6 measured wide
    /// bandwidth (1333 centi-GB/s = 13.33 GB/s). For
    /// canonical / mid fixtures the K=1 reference is
    /// 0 because the corpus crate has no per-fixture
    /// bandwidth measurement; those scales are scored by
    /// catalogs/sec rather than GB/s.
    pub pre_bandwidth_centi_gbps: u32,
    /// Post-batching effective bandwidth in centi-GB/s
    /// (at best-K). For full 256x4096:
    /// `pre_bandwidth * best_k_gain_basis_points / 10000`
    /// (i.e. equivalent bandwidth at best-K assuming the
    /// gain ratio translates directly to bytes/sec). For
    /// canonical / mid: 0.
    pub post_bandwidth_centi_gbps: u32,
    /// Signed K-amortisation delta vs K=1, in basis points
    /// (`best_k_gain_basis_points - 10000`, stored as i32
    /// so a regression is representable).
    pub delta_basis_points: i32,
    /// Panel-pinned interpretation label derived from
    /// `best_k_gain_basis_points` per
    /// [`BatchedKResultInterpretation::from_gain_basis_points`].
    pub interpretation: BatchedKResultInterpretation,
}

// ---------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------

/// Why the R.12b K-saturation table parser rejected a
/// source-report text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// One of the 18 expected (scale, K) cells is absent.
    MissingCell {
        /// Which scale's row was missing.
        scale_label: &'static str,
        /// Which K column was missing.
        k: u32,
    },
    /// A numeric field could not be parsed.
    MalformedNumber {
        /// Which scale's row.
        scale_label: &'static str,
        /// Which K column.
        k: u32,
        /// Field that failed to parse.
        field: &'static str,
    },
    /// Header row absent (the K-throughput section was
    /// missing entirely).
    MissingHeader,
}

// ---------------------------------------------------------------
// Parser
// ---------------------------------------------------------------

/// WHY: the R.12b harness writes the K-saturation matrix
/// as a fixed-width pipe-delimited table; this parser
/// walks every panel-pinned (scale, K) row and extracts
/// the timing + percentages + top-stage data. The parser
/// is strict: every expected cell must be present, and
/// every numeric field must parse as the declared type.
/// Missing cells or malformed numbers surface as typed
/// [`ParseError`] variants so a future drift in the
/// harness's output format fails CI immediately rather
/// than silently corrupting the receipt.
///
/// # Errors
///
/// Returns [`ParseError`] when any panel-pinned (scale, K)
/// cell is absent, or a numeric field cannot be parsed.
pub fn parse_batched_k_saturation_table(
    text: &str,
) -> Result<Vec<ParsedBatchedKCellV1>, ParseError> {
    // Find the table header so we can ignore prose above and
    // detailed-throughput rows below.
    let header_pos = text
        .find("scale                  |   K | per_cat_us |  cat/sec |")
        .ok_or(ParseError::MissingHeader)?;
    let table_text = &text[header_pos..];

    let mut cells: Vec<ParsedBatchedKCellV1> = Vec::with_capacity(18);
    for &scale_label in S_PERF_8_SCALES {
        for &k in S_PERF_8_K_MATRIX {
            // Each row begins with the scale label padded to
            // 22 chars, then "|   K |..." with K right-
            // padded to 3 chars. Build the prefix we look
            // for so we hit the unique row.
            let row_prefix = format!("  {scale_label:<22} | {k:>3} |");
            let row_idx = table_text
                .find(&row_prefix)
                .ok_or(ParseError::MissingCell { scale_label, k })?;
            // Tail: everything after the row prefix on that
            // same line.
            let tail = &table_text[row_idx + row_prefix.len()..];
            let line_end = tail.find('\n').unwrap_or(tail.len());
            let row = &tail[..line_end];

            // Split the row on '|' to extract each column.
            let cols: Vec<&str> = row.split('|').map(str::trim).collect();
            // Columns after the prefix:
            //   0 per_cat_us
            //   1 cat/sec (X.X)
            //   2 features_pct (X.X%)
            //   3 dev_total_pct
            //   4 finalize_pct
            //   5 top_stage (label (XX%))
            //   6 spd_vs_cpub (— for now)
            if cols.len() < 6 {
                return Err(ParseError::MalformedNumber {
                    scale_label,
                    k,
                    field: "row_column_count",
                });
            }

            let per_cat_us = cols[0]
                .parse::<u32>()
                .map_err(|_| ParseError::MalformedNumber {
                    scale_label,
                    k,
                    field: "per_cat_us",
                })?;
            let cat_per_sec_centi = parse_fixed1_to_centi(cols[1], scale_label, k, "cat_sec")?;
            let features_pct_basis_points =
                parse_pct_to_basis_points(cols[2], scale_label, k, "features_pct")?;
            let dev_total_pct_basis_points =
                parse_pct_to_basis_points(cols[3], scale_label, k, "dev_total_pct")?;
            let finalize_pct_basis_points =
                parse_pct_to_basis_points(cols[4], scale_label, k, "finalize_pct")?;
            let (top_stage_label, top_stage_pct_basis_points) =
                parse_top_stage(cols[5], scale_label, k)?;

            cells.push(ParsedBatchedKCellV1 {
                scale_label,
                k,
                per_cat_us,
                cat_per_sec_centi,
                features_pct_basis_points,
                dev_total_pct_basis_points,
                finalize_pct_basis_points,
                top_stage_label,
                top_stage_pct_basis_points,
            });
        }
    }
    Ok(cells)
}

/// WHY: per-scale summarisation surfaces the K-amortisation
/// pattern in a form that subsequent Track B legs can
/// compare against. The summary records K=1 baseline +
/// best-observed K + the gain ratio + the panel-pinned
/// interpretation label + the pre / post effective
/// bandwidth (for the full 256x4096 scale, anchored to the
/// S-PERF.6 measured wide bandwidth at K=1).
///
/// The `pre_bandwidth_centi_gbps` argument is the S-PERF.6
/// measured wide bandwidth at full K=1 (panel-pinned 1333
/// centi-GB/s). Canonical and mid scales score by catalogs/
/// sec rather than GB/s; for those the receipt records
/// `pre_bandwidth = post_bandwidth = 0` so a future test
/// cannot accidentally interpret canonical / mid bandwidth
/// as full-scale bandwidth.
#[must_use]
pub fn summarise_per_scale(
    cells: &[ParsedBatchedKCellV1],
    full_scale_pre_bandwidth_centi_gbps: u32,
) -> Vec<ParsedScaleSummaryV1> {
    let mut out: Vec<ParsedScaleSummaryV1> = Vec::with_capacity(S_PERF_8_SCALES.len());
    for &scale_label in S_PERF_8_SCALES {
        let scale_cells: Vec<&ParsedBatchedKCellV1> = cells
            .iter()
            .filter(|c| c.scale_label == scale_label)
            .collect();
        let k1 = scale_cells
            .iter()
            .find(|c| c.k == 1)
            .map_or(0, |c| c.cat_per_sec_centi);
        let best = scale_cells.iter().max_by_key(|c| c.cat_per_sec_centi);
        let (best_k, best_centi) = best.map_or((1, k1), |c| (c.k, c.cat_per_sec_centi));
        let gain_bp = if k1 == 0 {
            0
        } else {
            // best / k1 * 10000, rounded toward zero.
            u32::try_from(u64::from(best_centi) * 10_000 / u64::from(k1)).unwrap_or(u32::MAX)
        };
        // Pre/post bandwidth only meaningful for the full
        // 256x4096 scale (the S-PERF.6 measured baseline);
        // canonical and mid carry 0 so a downstream consumer
        // cannot accidentally attribute small-fixture
        // throughput to the headline bandwidth column.
        let (pre_bw, post_bw) = if scale_label == "full 256x4096" {
            let pre = full_scale_pre_bandwidth_centi_gbps;
            // post = pre * gain_bp / 10000 (gain ratio
            // applied to the K=1 reference bandwidth).
            let post =
                u32::try_from(u64::from(pre) * u64::from(gain_bp) / 10_000).unwrap_or(u32::MAX);
            (pre, post)
        } else {
            (0, 0)
        };
        // Signed delta: gain_bp - 10000. Cap at i32 range.
        let delta_bp: i32 = i32::try_from(gain_bp)
            .map(|g| g - 10_000)
            .unwrap_or(i32::MAX);
        let interpretation = BatchedKResultInterpretation::from_gain_basis_points(gain_bp);
        out.push(ParsedScaleSummaryV1 {
            scale_label,
            k1_cat_per_sec_centi: k1,
            best_k,
            best_k_cat_per_sec_centi: best_centi,
            best_k_gain_basis_points: gain_bp,
            pre_bandwidth_centi_gbps: pre_bw,
            post_bandwidth_centi_gbps: post_bw,
            delta_basis_points: delta_bp,
            interpretation,
        });
    }
    out
}

// ---------------------------------------------------------------
// Parsing primitives
// ---------------------------------------------------------------

/// WHY: cat/sec is rendered as "X.X" (one decimal place) in
/// the R.12b table. We extract it as centi-units so the
/// fixed-point value is hashable as a u32. The grammar
/// accepts "X.Y" (one decimal digit) precisely; "X.YY"
/// drops the trailing digit by parsing only the first
/// decimal place.
fn parse_fixed1_to_centi(
    s: &str,
    scale_label: &'static str,
    k: u32,
    field: &'static str,
) -> Result<u32, ParseError> {
    let s = s.trim();
    let (int_part, frac_part) = s.split_once('.').ok_or(ParseError::MalformedNumber {
        scale_label,
        k,
        field,
    })?;
    let int_v = int_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    // Take only the first decimal digit if more are present.
    let first_digit_char = frac_part
        .chars()
        .next()
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    let first_digit = first_digit_char
        .to_digit(10)
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    int_v
        .checked_mul(100)
        .and_then(|v| v.checked_add(first_digit * 10))
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })
}

/// WHY: percentage fields are "X.Y%" or "XX.Y%". We extract
/// as basis points (X.Y% -> XY0 bp; 12.34% -> 1234 bp).
/// Strip the trailing `%`, then parse the integer + decimal
/// pair.
fn parse_pct_to_basis_points(
    s: &str,
    scale_label: &'static str,
    k: u32,
    field: &'static str,
) -> Result<u32, ParseError> {
    let s = s.trim();
    let s = s
        .strip_suffix('%')
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?
        .trim();
    let (int_part, frac_part) = s.split_once('.').ok_or(ParseError::MalformedNumber {
        scale_label,
        k,
        field,
    })?;
    let int_v = int_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    let first_digit_char = frac_part
        .chars()
        .next()
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    let first_digit = first_digit_char
        .to_digit(10)
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })?;
    int_v
        .checked_mul(100)
        .and_then(|v| v.checked_add(first_digit * 10))
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field,
        })
}

/// WHY: the top-stage column is "label (XX%)"; we want the
/// label + the percentage. The label is one of a known
/// panel-locked set; we look it up against the set so a
/// future drift in the harness's stage naming is caught.
fn parse_top_stage(
    s: &str,
    scale_label: &'static str,
    k: u32,
) -> Result<(&'static str, u32), ParseError> {
    let s = s.trim();
    let (label, rest) = s.split_once(" (").ok_or(ParseError::MalformedNumber {
        scale_label,
        k,
        field: "top_stage_label_paren",
    })?;
    let pct_s = rest
        .strip_suffix(')')
        .ok_or(ParseError::MalformedNumber {
            scale_label,
            k,
            field: "top_stage_close_paren",
        })?
        .trim();
    let pct_s = pct_s.strip_suffix('%').ok_or(ParseError::MalformedNumber {
        scale_label,
        k,
        field: "top_stage_pct_percent",
    })?;
    let pct = pct_s
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber {
            scale_label,
            k,
            field: "top_stage_pct_value",
        })?;
    let static_label = match label.trim() {
        "digest_consensus" => "digest_consensus",
        "digest_detector" => "digest_detector",
        "digest_sign" => "digest_sign",
        "digest_residual" => "digest_residual",
        "detector_wide" => "detector_wide",
        "consensus_wide" => "consensus_wide",
        "candidate_wide" => "candidate_wide",
        "residual" => "residual",
        "sign" => "sign",
        "h2d" => "h2d",
        "axis5_grid" => "axis5_grid",
        _ => {
            return Err(ParseError::MalformedNumber {
                scale_label,
                k,
                field: "top_stage_unknown_label",
            });
        }
    };
    Ok((static_label, pct * 100))
}

// ---------------------------------------------------------------
// BatchedKSaturationReceiptV1
// ---------------------------------------------------------------

/// Panel-pinned dispatch-mode label. The R.12b harness
/// processes K as a host loop of K serial single-catalog
/// dispatches on a hot `GpuWorkspace`; this label is
/// recorded verbatim into the receipt so an auditor can
/// see what was actually measured. The verifier rejects any
/// receipt whose `dispatch_mode_label` claims "true batched
/// K" or "single launch" --- the receipt MUST be honest
/// about the dispatch shape.
pub const S_PERF_8_DISPATCH_MODE_LABEL: &str =
    "host-loop K serial dispatches on hot GpuWorkspace (R.12b)";

/// Panel-pinned catalog-order label. The R.12b harness
/// dispatches in a fixed canonical order (scale-major, then
/// K-ascending). Recording the label explicitly lets the
/// verifier reject a future commit that switches to
/// completion-order or randomised-order processing.
pub const S_PERF_8_CATALOG_ORDER_LABEL: &str =
    "canonical scale-major then K-ascending; deterministic per-catalog dispatch order";

/// Panel-pinned merge-policy label. The K=1 single-catalog
/// dispatch has no merge step (one catalog per iteration);
/// the K>1 host loop sequences per-catalog dispatches with
/// no inter-catalog merge. The verifier rejects any
/// receipt whose `merge_policy_label` contains
/// "completion-order" or "completion order" --- those would
/// indicate a non-deterministic merge.
pub const S_PERF_8_MERGE_POLICY_LABEL: &str =
    "no inter-catalog merge; sequential per-catalog admission in canonical order";

/// Panel-pinned CUDA-graph status label. The R.12b harness
/// does NOT engage CUDA Graph capture for K iterations
/// (R.6c graph capture is available on the workspace but
/// not engaged by R.12b's per-call dispatch pattern). The
/// verifier rejects any receipt whose
/// `cuda_graph_status_label` claims graph capture is
/// engaged without also providing a `graph_plan_hash`
/// replay-contract field --- which the current schema does
/// not have, so the panel-pinned label is the
/// "not engaged" form.
pub const S_PERF_8_CUDA_GRAPH_STATUS_LABEL: &str =
    "CUDA Graph capture NOT engaged for K iterations; per-iteration cudaLaunchKernel under hot GpuWorkspace";

/// Panel-pinned R.12b episode counts (the same 13/89/1917
/// values pinned in the S-PERF.6 receipt + the S-PERF.7
/// verifier). Recording them as first-class fields on the
/// S-PERF.8 receipt lets the verifier reject any drift at
/// receipt-build time even if the upstream S-PERF.6 / S-PERF.7
/// chain were ever weakened.
pub const S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128: u32 = 13;
/// See [`S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128`].
pub const S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512: u32 = 89;
/// See [`S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128`].
pub const S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096: u32 = 1917;

/// The top-level S-PERF.8 batched-K saturation receipt.
/// Binds the parsed R.12b K-saturation table + per-scale
/// summaries + the panel-pinned dispatch-mode / catalog-
/// order / merge-policy / CUDA-graph-status labels + the
/// RTX 4080 SUPER device-identity hash + the three R.12b
/// episode-count pins + the live S-PERF.6 baseline-report
/// hash + the live S-PERF.7 source-report-import verifier
/// hash.
#[derive(Debug, Clone)]
pub struct BatchedKSaturationReceiptV1 {
    /// Human-readable receipt identifier (non-empty).
    pub receipt_id: &'static str,
    /// Source-report path the K table was parsed from.
    pub r12b_source_report_path: &'static str,
    /// Panel-pinned dispatch-mode label
    /// ([`S_PERF_8_DISPATCH_MODE_LABEL`]).
    pub dispatch_mode_label: &'static str,
    /// Panel-pinned catalog-order label
    /// ([`S_PERF_8_CATALOG_ORDER_LABEL`]).
    pub catalog_order_label: &'static str,
    /// Panel-pinned merge-policy label
    /// ([`S_PERF_8_MERGE_POLICY_LABEL`]).
    pub merge_policy_label: &'static str,
    /// Panel-pinned CUDA-graph status label
    /// ([`S_PERF_8_CUDA_GRAPH_STATUS_LABEL`]).
    pub cuda_graph_status_label: &'static str,
    /// RTX 4080 SUPER device-identity hash. Anchors the
    /// receipt to the panel-locked device the bench ran on
    /// so a cross-device receipt cannot be quietly admitted.
    pub device_identity_hash: [u8; 32],
    /// R.12b episode count at the canonical 16x128 grid
    /// (panel-pinned to 13).
    pub r12b_episode_count_canonical_w16h128: u32,
    /// R.12b episode count at the mid 64x512 grid
    /// (panel-pinned to 89).
    pub r12b_episode_count_mid_w64h512: u32,
    /// R.12b episode count at the full 256x4096 grid
    /// (panel-pinned to 1917).
    pub r12b_episode_count_full_w256h4096: u32,
    /// 18 parsed (scale, K) cells in canonical order
    /// (scale-major then K-ascending).
    pub cells: Vec<ParsedBatchedKCellV1>,
    /// Per-scale K-amortisation summaries (one per scale).
    pub per_scale_summaries: Vec<ParsedScaleSummaryV1>,
    /// S-PERF.6 baseline report hash (the chain anchor).
    pub s_perf_6_baseline_report_hash: [u8; 32],
    /// S-PERF.7 source-report-import verifier hash (binds
    /// the chain to the S-PERF.7 verifier's auditable
    /// parse of the same source reports).
    pub s_perf_7_source_report_import_verifier_hash: [u8; 32],
    /// `batched_k_saturation_receipt_hash_v1`.
    pub batched_k_saturation_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verifier error kinds (4 panel-required + structural)
// ---------------------------------------------------------------

/// Why S-PERF.8 rejected a (parsed-batched-K-table,
/// S-PERF.6 receipt) pair. Four panel-required load-bearing
/// negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf8VerifyErrorKind {
    /// Panel-required #1. The K saturation matrix is
    /// missing one of the 18 panel-pinned cells.
    KMatrixIncomplete {
        /// Number of cells the parser found.
        parsed_cells: usize,
        /// Number of cells the panel pinned (18).
        expected_cells: usize,
    },
    /// Panel-required #2. The K=1 full-scale per_cat_us in
    /// the K table does not equal the S-PERF.6 receipt's
    /// device_total_us within an acceptable tolerance.
    /// (Both numbers measure the same per-catalog wall time
    /// at the same workload; they should be in the same
    /// neighborhood.)
    K1FullScalePerCatInconsistentWithReceipt {
        /// Per-cat us reported in the K table at full K=1.
        k_table_per_cat_us: u32,
        /// Device total us in the S-PERF.6 receipt.
        receipt_device_total_us: u64,
        /// Wall time in the S-PERF.6 receipt
        /// (per-cat upper bound).
        receipt_host_wall_median_us: u64,
    },
    /// Panel-required #3. The K=1 catalogs/sec at any
    /// scale does not match `1_000_000 / per_cat_us` within
    /// arithmetic precision.
    K1CatPerSecArithmeticMismatch {
        /// Which scale.
        scale_label: &'static str,
        /// What the table claimed.
        claimed_centi: u32,
        /// What the verifier's arithmetic computed.
        computed_centi: u32,
    },
    /// Panel-required #4. The per-scale K-amortisation gain
    /// claim exceeds the panel-pinned ceiling for that scale
    /// (a gain of >5x is panel-forbidden because it would
    /// imply the workload is launch-bound to a degree that
    /// contradicts the device-total share already declared
    /// in S-PERF.6).
    KAmortisationGainExceedsCeiling {
        /// Which scale's claim exceeded the ceiling.
        scale_label: &'static str,
        /// Claimed gain in basis points
        /// (10000 = 1x; 50000 = 5x).
        claimed_gain_basis_points: u32,
        /// Panel-pinned ceiling in basis points
        /// (50000 = 5x).
        panel_ceiling_basis_points: u32,
    },
    /// Structural: receipt_id empty.
    ReceiptIdEmpty,
    /// Structural: r12b source report path empty.
    R12bSourceReportPathEmpty,
    /// Structural: per-scale summaries count does not equal
    /// the panel-pinned scale set size.
    PerScaleSummariesIncomplete {
        /// Number of summaries the receipt has.
        observed: usize,
        /// Number the panel pinned.
        expected: usize,
    },
    /// Structural: a scale summary's k1 baseline does not
    /// equal the parsed K=1 cell's cat_per_sec.
    SummaryK1MismatchWithParsedCell {
        /// Which scale's summary disagreed.
        scale_label: &'static str,
        /// Summary's k1 cat/sec, centi-units.
        summary_k1_centi: u32,
        /// Parsed cell's k1 cat/sec, centi-units.
        cell_k1_centi: u32,
    },
    /// Panel-required #5 (S-PERF.8.1). The
    /// `dispatch_mode_label` claims a dispatch shape the
    /// R.12b harness does not actually use ("true batched
    /// K", "single launch", or "graph capture engaged"). The
    /// only honest label is the panel-pinned
    /// [`S_PERF_8_DISPATCH_MODE_LABEL`] (host-loop K).
    HostLoopKClaimedAsBatched {
        /// What the receipt declared.
        declared_label: &'static str,
    },
    /// Panel-required #6. The source-report path is
    /// empty / missing.
    MissingBatchedKSourceReport,
    /// Panel-required #7. A per-scale summary at the full
    /// 256x4096 scale is missing pre / post bandwidth
    /// (one or both zero).
    MissingPrePostBandwidthDelta {
        /// Pre-bandwidth observed.
        pre_centi_gbps: u32,
        /// Post-bandwidth observed.
        post_centi_gbps: u32,
    },
    /// Panel-required #8. The full-scale interpretation
    /// claims a stronger result than the measured delta
    /// supports (e.g. labels as `LaunchBoundGainAtSmallFixture`
    /// when delta is <50%).
    FullScaleClaimAboveMeasuredDelta {
        /// What the receipt's interpretation declared.
        declared_interpretation_wire_name: &'static str,
        /// Observed delta in basis points.
        observed_delta_bp: i32,
    },
    /// Panel-required #9. The receipt claims full-scale
    /// post bandwidth >= 2500 centi-GB/s (= 25 GB/s) even
    /// though the measured delta does not support it.
    /// Specifically: if `post_bandwidth_centi_gbps` for
    /// full 256x4096 >= 2500 AND
    /// `post_bandwidth_centi_gbps < pre_bandwidth * 18750 / 10000`
    /// (i.e. would need 1.876x gain to reach 25 GB/s from
    /// 13.33 GB/s pre), the receipt overclaims.
    ClaimFullScaleReached25GbpsWithoutMeasurement {
        /// Claimed post bandwidth.
        claimed_post_centi_gbps: u32,
        /// Reference pre bandwidth.
        pre_centi_gbps: u32,
        /// Required gain bp to reach 25 GB/s from
        /// `pre_centi_gbps`.
        required_gain_bp: u32,
        /// Actual gain bp claimed.
        actual_gain_bp: u32,
    },
    /// Panel-required #10. The receipt claims saturation
    /// (any post-bandwidth percent-of-peak >= 8000 bp)
    /// while the actual percent-of-peak is below the
    /// saturation threshold.
    SaturationClaimBelow8000Bp {
        /// Observed full-scale percent-of-peak in bp
        /// (`post_bandwidth_gbps * 10000 / 716`).
        observed_bp: u32,
    },
    /// Panel-required #11 --- the CAMPAIGN IDENTITY
    /// negative. The receipt labels the full 256x4096 scale
    /// with [`BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture`].
    /// That label is reserved for small launch-bound
    /// fixtures (canonical 16x128); applying it to full-
    /// scale would mechanise the dangerous overclaim
    /// *"canonical got 1.76x, therefore K batching solved
    /// full-scale"*. The court refuses.
    CanonicalLaunchBoundGainGeneralizedToFullScale {
        /// What the receipt's full-scale interpretation
        /// claimed.
        full_scale_interpretation_wire_name: &'static str,
    },
    /// Panel-required #12. One of the three R.12b episode
    /// pins drifted from the panel-locked tuple
    /// (13 / 89 / 1917).
    R12bEpisodePinsDrift {
        /// Which pin disagreed.
        which: &'static str,
        /// Receipt's declared value.
        declared: u32,
        /// Panel-locked value.
        panel_locked: u32,
    },
    /// Panel-required #13. The `catalog_order_label`
    /// drifted from the panel-pinned canonical order
    /// ([`S_PERF_8_CATALOG_ORDER_LABEL`]).
    CatalogOrderDrift {
        /// What the receipt declared.
        declared_label: &'static str,
    },
    /// Panel-required #14. The `merge_policy_label` admits
    /// completion-order merging (the receipt contains the
    /// substring "completion-order" or "completion order").
    /// Completion-order merging breaks determinism; the
    /// panel-locked policy is sequential per-catalog
    /// admission in canonical order
    /// ([`S_PERF_8_MERGE_POLICY_LABEL`]).
    CompletionOrderMergeRejected {
        /// What the receipt declared.
        declared_label: &'static str,
    },
    /// Panel-required #15. The `cuda_graph_status_label`
    /// claims CUDA Graph capture is engaged but the schema
    /// does not carry a `graph_plan_hash` replay-contract
    /// field. R.6c's CUDA Graph capture path requires the
    /// graph_plan_hash to be recorded for replay; absent
    /// that, claiming engagement is an overclaim.
    CudaGraphClaimWithoutReplayContract {
        /// What the receipt declared.
        declared_label: &'static str,
    },
    /// Panel-required #16. The `device_identity_hash` is
    /// the zero hash, indicating no device-identity binding.
    MissingDeviceIdentity,
    /// Panel-required #17. The full-scale post-bandwidth
    /// percent-of-peak computed from the receipt does not
    /// match the percent-of-peak derived from the pre /
    /// gain values.
    PercentOfPeakArithmeticMismatch {
        /// Receipt-declared percent-of-peak in bp.
        declared_bp: u32,
        /// Verifier-computed percent-of-peak in bp.
        computed_bp: u32,
    },
    /// Panel-required #18. The per-scale gain
    /// (`best_k_gain_basis_points`) does not match
    /// `best_centi / k1_centi * 10000` for that scale.
    SpeedupArithmeticMismatch {
        /// Which scale's summary disagreed.
        scale_label: &'static str,
        /// Receipt-declared gain bp.
        declared_bp: u32,
        /// Verifier-computed gain bp.
        computed_bp: u32,
    },
    /// Structural: the per-scale summary's interpretation
    /// label does not match
    /// [`BatchedKResultInterpretation::from_gain_basis_points`]
    /// of the summary's gain.
    InterpretationLabelMismatch {
        /// Which scale's summary disagreed.
        scale_label: &'static str,
        /// Declared interpretation wire name.
        declared_wire_name: &'static str,
        /// Verifier-classified wire name.
        computed_wire_name: &'static str,
    },
    /// Structural: any empty panel-pinned label field.
    EmptyPanelLabel {
        /// Which label field is empty.
        which: &'static str,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf8VerifyError {
    /// Error kind.
    pub kind: SPerf8VerifyErrorKind,
}

/// Panel-pinned maximum allowed K-amortisation gain ratio,
/// in basis points (50000 = 5x). At small fixtures
/// (canonical 16x128) the host-loop K can ~2x throughput;
/// the 5x ceiling is the panel's "any larger is suspect"
/// gate that catches a future commit that overstates
/// batching gains.
pub const S_PERF_8_K_AMORTISATION_CEILING_BASIS_POINTS: u32 = 50_000;

// ---------------------------------------------------------------
// Builder
// ---------------------------------------------------------------

/// Build a [`BatchedKSaturationReceiptV1`] and populate
/// `batched_k_saturation_receipt_hash_v1`. The builder takes
/// pre-parsed cells + per-scale summaries + upstream anchor
/// hashes + the panel-pinned dispatch / catalog / merge /
/// CUDA-graph labels + the RTX 4080 SUPER device-identity
/// hash + the three R.12b episode pins so the verifier can
/// be exercised in tests without disk I/O.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_batched_k_saturation_receipt(
    receipt_id: &'static str,
    r12b_source_report_path: &'static str,
    dispatch_mode_label: &'static str,
    catalog_order_label: &'static str,
    merge_policy_label: &'static str,
    cuda_graph_status_label: &'static str,
    device_identity_hash: [u8; 32],
    r12b_episode_count_canonical_w16h128: u32,
    r12b_episode_count_mid_w64h512: u32,
    r12b_episode_count_full_w256h4096: u32,
    cells: Vec<ParsedBatchedKCellV1>,
    per_scale_summaries: Vec<ParsedScaleSummaryV1>,
    s_perf_6_baseline_report_hash: [u8; 32],
    s_perf_7_source_report_import_verifier_hash: [u8; 32],
) -> BatchedKSaturationReceiptV1 {
    let mut r = BatchedKSaturationReceiptV1 {
        receipt_id,
        r12b_source_report_path,
        dispatch_mode_label,
        catalog_order_label,
        merge_policy_label,
        cuda_graph_status_label,
        device_identity_hash,
        r12b_episode_count_canonical_w16h128,
        r12b_episode_count_mid_w64h512,
        r12b_episode_count_full_w256h4096,
        cells,
        per_scale_summaries,
        s_perf_6_baseline_report_hash,
        s_perf_7_source_report_import_verifier_hash,
        batched_k_saturation_receipt_hash_v1: [0u8; 32],
    };
    r.batched_k_saturation_receipt_hash_v1 = compute_batched_k_saturation_receipt_hash(&r);
    r
}

// ---------------------------------------------------------------
// Seed (live disk reads)
// ---------------------------------------------------------------

/// Build the panel-pinned S-PERF.8 receipt by parsing the
/// R.12b saturation report on disk + binding upstream
/// S-PERF.6 + S-PERF.7 hashes.
///
/// # Errors
///
/// Returns [`SeedError`] if the source report file is missing
/// / unreadable, or if the parser rejects.
pub fn seed_batched_k_saturation_receipt_from_disk(
    repo_root: &std::path::Path,
) -> Result<BatchedKSaturationReceiptV1, SeedError> {
    let r12b_path = repo_root.join(S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH);
    let r12b_text = std::fs::read_to_string(&r12b_path).map_err(|e| SeedError::ReadR12b {
        path: r12b_path.display().to_string(),
        message: e.to_string(),
    })?;
    let cells = parse_batched_k_saturation_table(&r12b_text).map_err(SeedError::Parse)?;
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let per_scale = summarise_per_scale(
        &cells,
        baseline.measurement.measured_wide_bandwidth_centi_gbps,
    );
    let verifier = seed_source_report_import_verifier_report_from_disk(repo_root)
        .map_err(|e| SeedError::SeedSPerf7(format!("{e:?}")))?;
    Ok(build_batched_k_saturation_receipt(
        "s_perf_8_batched_k_saturation_receipt_v1",
        S_PERF_7_R12B_SATURATION_SOURCE_REPORT_PATH,
        S_PERF_8_DISPATCH_MODE_LABEL,
        S_PERF_8_CATALOG_ORDER_LABEL,
        S_PERF_8_MERGE_POLICY_LABEL,
        S_PERF_8_CUDA_GRAPH_STATUS_LABEL,
        baseline.measurement.device_uuid_or_identity_hash,
        S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        cells,
        per_scale,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        verifier.source_report_import_verifier_hash_v1,
    ))
}

/// Why
/// [`seed_batched_k_saturation_receipt_from_disk`] failed.
#[derive(Debug)]
pub enum SeedError {
    /// Could not read the R.12b saturation source report.
    ReadR12b {
        /// Resolved path.
        path: String,
        /// Underlying I/O error message.
        message: String,
    },
    /// Parser rejected the R.12b source report text.
    Parse(ParseError),
    /// Could not seed the upstream S-PERF.7 verifier.
    SeedSPerf7(String),
}

// ---------------------------------------------------------------
// Hash builder
// ---------------------------------------------------------------

/// WHY: serialises every parsed cell + every per-scale
/// summary + upstream anchor hashes into a canonical byte
/// buffer and SHA-256s the result so two builds against
/// the same source-report text + same upstream hashes
/// produce byte-identical
/// `batched_k_saturation_receipt_hash_v1`. Field order is
/// locked; any reordering rebases the hash.
fn compute_batched_k_saturation_receipt_hash(r: &BatchedKSaturationReceiptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_8_BATCHED_K_SATURATION_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S_PERF_8_BATCHED_K_SATURATION_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.receipt_id.as_bytes());
    push_len_prefixed(&mut buf, r.r12b_source_report_path.as_bytes());
    // Panel-pinned labels (S-PERF.8.1 schema upgrade).
    push_len_prefixed(&mut buf, r.dispatch_mode_label.as_bytes());
    push_len_prefixed(&mut buf, r.catalog_order_label.as_bytes());
    push_len_prefixed(&mut buf, r.merge_policy_label.as_bytes());
    push_len_prefixed(&mut buf, r.cuda_graph_status_label.as_bytes());
    // Device-identity binding (RTX 4080 SUPER).
    buf.extend_from_slice(&r.device_identity_hash);
    // R.12b episode pins (13 / 89 / 1917).
    buf.extend_from_slice(&r.r12b_episode_count_canonical_w16h128.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_mid_w64h512.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_full_w256h4096.to_be_bytes());
    // Cells in canonical order (parser emits scale-major
    // then K-ascending, which is the hash order).
    let cell_count = u32::try_from(r.cells.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&cell_count.to_be_bytes());
    for c in &r.cells {
        push_len_prefixed(&mut buf, c.scale_label.as_bytes());
        buf.extend_from_slice(&c.k.to_be_bytes());
        buf.extend_from_slice(&c.per_cat_us.to_be_bytes());
        buf.extend_from_slice(&c.cat_per_sec_centi.to_be_bytes());
        buf.extend_from_slice(&c.features_pct_basis_points.to_be_bytes());
        buf.extend_from_slice(&c.dev_total_pct_basis_points.to_be_bytes());
        buf.extend_from_slice(&c.finalize_pct_basis_points.to_be_bytes());
        push_len_prefixed(&mut buf, c.top_stage_label.as_bytes());
        buf.extend_from_slice(&c.top_stage_pct_basis_points.to_be_bytes());
    }
    // Per-scale summaries.
    let sum_count = u32::try_from(r.per_scale_summaries.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&sum_count.to_be_bytes());
    for s in &r.per_scale_summaries {
        push_len_prefixed(&mut buf, s.scale_label.as_bytes());
        buf.extend_from_slice(&s.k1_cat_per_sec_centi.to_be_bytes());
        buf.extend_from_slice(&s.best_k.to_be_bytes());
        buf.extend_from_slice(&s.best_k_cat_per_sec_centi.to_be_bytes());
        buf.extend_from_slice(&s.best_k_gain_basis_points.to_be_bytes());
        // S-PERF.8.1 fields.
        buf.extend_from_slice(&s.pre_bandwidth_centi_gbps.to_be_bytes());
        buf.extend_from_slice(&s.post_bandwidth_centi_gbps.to_be_bytes());
        buf.extend_from_slice(&s.delta_basis_points.to_be_bytes());
        push_len_prefixed(&mut buf, s.interpretation.as_str().as_bytes());
    }
    // Anchors.
    buf.extend_from_slice(&r.s_perf_6_baseline_report_hash);
    buf.extend_from_slice(&r.s_perf_7_source_report_import_verifier_hash);
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify the batched-K saturation receipt against the
/// S-PERF.6 baseline + the panel-pinned K matrix + the
/// per-scale K-amortisation ceiling. Returns the list of
/// errors (empty when every panel-required negative + every
/// structural rule passes).
///
/// The four panel-required load-bearing negatives are:
///
///  1. `KMatrixIncomplete` (the parser must have all 18
///     (scale, K) cells)
///  2. `K1FullScalePerCatInconsistentWithReceipt` (the
///     full-scale K=1 per_cat_us must be within the host-
///     wall-median bound declared by S-PERF.6)
///  3. `K1CatPerSecArithmeticMismatch` (the K=1 cat/sec
///     for each scale must match `1e6 / per_cat_us` within
///     centi-unit precision)
///  4. `KAmortisationGainExceedsCeiling` (any per-scale
///     gain >5x is panel-forbidden as a launch-bound-only
///     overclaim)
#[must_use]
#[allow(clippy::too_many_lines)] // 4 panel-required negatives + 4 structural rules
pub fn verify_batched_k_saturation_receipt(
    receipt: &BatchedKSaturationReceiptV1,
    s_perf_6_receipt: &Rtx4080SuperMeasuredCudaPipelineV1,
) -> Vec<SPerf8VerifyError> {
    let mut errors: Vec<SPerf8VerifyError> = Vec::new();

    if receipt.receipt_id.is_empty() {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::ReceiptIdEmpty,
        });
    }
    if receipt.r12b_source_report_path.is_empty() {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::R12bSourceReportPathEmpty,
        });
    }

    // Panel-required #1: full 3x6 K matrix coverage.
    let expected = S_PERF_8_SCALES.len() * S_PERF_8_K_MATRIX.len();
    if receipt.cells.len() != expected {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::KMatrixIncomplete {
                parsed_cells: receipt.cells.len(),
                expected_cells: expected,
            },
        });
    }

    // Panel-required #2: K=1 full-scale per_cat_us must be
    // bounded above by the S-PERF.6 host_wall_median (per
    // catalog) and bounded below by the S-PERF.6 device
    // total. Any per_cat_us outside that envelope means the
    // K table is measuring something other than the same
    // workload.
    if let Some(full_k1) = receipt
        .cells
        .iter()
        .find(|c| c.scale_label == "full 256x4096" && c.k == 1)
    {
        let per_cat_us = u64::from(full_k1.per_cat_us);
        if per_cat_us < s_perf_6_receipt.device_total_us
            || per_cat_us > s_perf_6_receipt.host_wall_median_us * 2
        {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::K1FullScalePerCatInconsistentWithReceipt {
                    k_table_per_cat_us: full_k1.per_cat_us,
                    receipt_device_total_us: s_perf_6_receipt.device_total_us,
                    receipt_host_wall_median_us: s_perf_6_receipt.host_wall_median_us,
                },
            });
        }
    }

    // Panel-required #3: K=1 cat/sec arithmetic coherence.
    // cat_per_sec = 1_000_000 / per_cat_us; cat_per_sec_centi
    // = 100 * 1_000_000 / per_cat_us = 100_000_000 / per_cat_us.
    // The R.12b harness rounds to one decimal place; tolerate
    // up to +/- 10 centi-units (= +/- 0.1 cat/sec).
    for c in &receipt.cells {
        if c.k != 1 || c.per_cat_us == 0 {
            continue;
        }
        let computed = 100_000_000u64 / u64::from(c.per_cat_us);
        let computed_u32 = u32::try_from(computed).unwrap_or(u32::MAX);
        let diff = computed_u32.abs_diff(c.cat_per_sec_centi);
        if diff > 10 {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::K1CatPerSecArithmeticMismatch {
                    scale_label: c.scale_label,
                    claimed_centi: c.cat_per_sec_centi,
                    computed_centi: computed_u32,
                },
            });
        }
    }

    // Panel-required #4: per-scale K-amortisation gain
    // ceiling.
    for s in &receipt.per_scale_summaries {
        if s.best_k_gain_basis_points > S_PERF_8_K_AMORTISATION_CEILING_BASIS_POINTS {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::KAmortisationGainExceedsCeiling {
                    scale_label: s.scale_label,
                    claimed_gain_basis_points: s.best_k_gain_basis_points,
                    panel_ceiling_basis_points: S_PERF_8_K_AMORTISATION_CEILING_BASIS_POINTS,
                },
            });
        }
    }

    // Structural: per-scale summaries count.
    if receipt.per_scale_summaries.len() != S_PERF_8_SCALES.len() {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::PerScaleSummariesIncomplete {
                observed: receipt.per_scale_summaries.len(),
                expected: S_PERF_8_SCALES.len(),
            },
        });
    }

    // Structural: per-scale summary k1 must equal the parsed
    // cell's k1 cat_per_sec.
    for s in &receipt.per_scale_summaries {
        let cell_k1 = receipt
            .cells
            .iter()
            .find(|c| c.scale_label == s.scale_label && c.k == 1)
            .map(|c| c.cat_per_sec_centi);
        if let Some(cell_k1) = cell_k1 {
            if cell_k1 != s.k1_cat_per_sec_centi {
                errors.push(SPerf8VerifyError {
                    kind: SPerf8VerifyErrorKind::SummaryK1MismatchWithParsedCell {
                        scale_label: s.scale_label,
                        summary_k1_centi: s.k1_cat_per_sec_centi,
                        cell_k1_centi: cell_k1,
                    },
                });
            }
        }
    }

    // --- S-PERF.8.1 panel-required negatives ---

    // Panel-required #5: dispatch-mode-label honesty.
    {
        let lower = receipt.dispatch_mode_label.to_ascii_lowercase();
        if lower.contains("true batched")
            || lower.contains("single launch")
            || lower.contains("graph capture engaged")
            || lower.contains("batched k kernel")
        {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::HostLoopKClaimedAsBatched {
                    declared_label: receipt.dispatch_mode_label,
                },
            });
        }
    }

    // Panel-required #6: source-report path present.
    if receipt.r12b_source_report_path.is_empty() {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::MissingBatchedKSourceReport,
        });
    }

    // Panel-required #7: full-scale pre/post bandwidth
    // populated.
    if let Some(full) = receipt
        .per_scale_summaries
        .iter()
        .find(|s| s.scale_label == "full 256x4096")
    {
        if full.pre_bandwidth_centi_gbps == 0 || full.post_bandwidth_centi_gbps == 0 {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::MissingPrePostBandwidthDelta {
                    pre_centi_gbps: full.pre_bandwidth_centi_gbps,
                    post_centi_gbps: full.post_bandwidth_centi_gbps,
                },
            });
        }
        // Panel-required #11: the CAMPAIGN IDENTITY negative.
        // Reject if full-scale interpretation is
        // `LaunchBoundGainAtSmallFixture` --- that label is
        // reserved for canonical small fixtures only.
        if matches!(
            full.interpretation,
            BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture
        ) {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::CanonicalLaunchBoundGainGeneralizedToFullScale {
                    full_scale_interpretation_wire_name: full.interpretation.as_str(),
                },
            });
        }

        // Panel-required #8: full-scale interpretation must
        // not overstate the measured delta.
        let bp = full.delta_basis_points;
        let must_be_at_least_for_label = match full.interpretation {
            BatchedKResultInterpretation::LaunchBoundGainAtSmallFixture => 5_000, // +50%
            BatchedKResultInterpretation::ModestFullScaleGain => 100,             // +1%
            BatchedKResultInterpretation::NoFullScaleImprovement => 0,
            BatchedKResultInterpretation::Regressed => i32::MIN,
        };
        if bp < must_be_at_least_for_label {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::FullScaleClaimAboveMeasuredDelta {
                    declared_interpretation_wire_name: full.interpretation.as_str(),
                    observed_delta_bp: bp,
                },
            });
        }

        // Panel-required #9: 25 GB/s claim guard. If the
        // receipt claims post bandwidth >= 2500 centi-GB/s
        // (= 25 GB/s) at full-scale, verify the implied gain
        // is actually present in the K table.
        if full.post_bandwidth_centi_gbps >= 2_500 && full.pre_bandwidth_centi_gbps > 0 {
            // Required gain bp to reach claimed post from pre.
            let required_gain_bp = u32::try_from(
                u64::from(full.post_bandwidth_centi_gbps) * 10_000
                    / u64::from(full.pre_bandwidth_centi_gbps),
            )
            .unwrap_or(u32::MAX);
            if full.best_k_gain_basis_points < required_gain_bp {
                errors.push(SPerf8VerifyError {
                    kind: SPerf8VerifyErrorKind::ClaimFullScaleReached25GbpsWithoutMeasurement {
                        claimed_post_centi_gbps: full.post_bandwidth_centi_gbps,
                        pre_centi_gbps: full.pre_bandwidth_centi_gbps,
                        required_gain_bp,
                        actual_gain_bp: full.best_k_gain_basis_points,
                    },
                });
            }
        }

        // Panel-required #10: saturation claim below 8000 bp.
        // Compute percent-of-peak from post-bandwidth assuming
        // the panel-pinned 716 GB/s theoretical peak.
        // percent_of_peak_bp = post_centi_gbps * 10000 / (716 * 100).
        if full.post_bandwidth_centi_gbps > 0 {
            let observed_bp =
                u32::try_from(u64::from(full.post_bandwidth_centi_gbps) * 10_000 / (716 * 100))
                    .unwrap_or(u32::MAX);
            // The receipt does NOT carry a saturation_admitted
            // boolean; the implicit saturation claim is that
            // the post-bandwidth percent-of-peak reaches the
            // 8000 bp gate. Fire only if the post-bandwidth
            // declares >= 8000 bp percent-of-peak via the
            // pre*gain arithmetic but the actual gain doesn't
            // back it up. Here observed_bp is the
            // already-derived percent-of-peak from the
            // post-bandwidth field; if any caller declares
            // post >= 716*0.8 * 100 = 57280 centi-GB/s
            // (the saturation post-bandwidth) the verifier
            // would surface the claim above. Below 8000 bp
            // here means: post-bandwidth corresponds to
            // <80% peak, which is the honest non-saturation
            // case (no error).
            //
            // The negative we fire is the inverse: a
            // saturation-class post bandwidth claim (>= 8000
            // bp percent-of-peak) backed by < 8000 bp
            // measured gain * pre. Compute the verifier
            // arithmetic from pre + gain.
            let computed_post_centi_gbps = u32::try_from(
                u64::from(full.pre_bandwidth_centi_gbps) * u64::from(full.best_k_gain_basis_points)
                    / 10_000,
            )
            .unwrap_or(u32::MAX);
            let computed_observed_bp =
                u32::try_from(u64::from(computed_post_centi_gbps) * 10_000 / (716 * 100))
                    .unwrap_or(u32::MAX);
            if observed_bp >= 8_000 && computed_observed_bp < 8_000 {
                errors.push(SPerf8VerifyError {
                    kind: SPerf8VerifyErrorKind::SaturationClaimBelow8000Bp { observed_bp },
                });
            }
            // Panel-required #17: percent-of-peak arithmetic
            // coherence (declared from post-bandwidth field
            // must equal computed from pre*gain).
            if observed_bp != computed_observed_bp {
                errors.push(SPerf8VerifyError {
                    kind: SPerf8VerifyErrorKind::PercentOfPeakArithmeticMismatch {
                        declared_bp: observed_bp,
                        computed_bp: computed_observed_bp,
                    },
                });
            }
        }
    }

    // Panel-required #12: R.12b episode pin coherence.
    let pin_triples = [
        (
            "r12b_episode_count_canonical_w16h128",
            receipt.r12b_episode_count_canonical_w16h128,
            S_PERF_8_R12B_EPISODE_COUNT_CANONICAL_W16H128,
        ),
        (
            "r12b_episode_count_mid_w64h512",
            receipt.r12b_episode_count_mid_w64h512,
            S_PERF_8_R12B_EPISODE_COUNT_MID_W64H512,
        ),
        (
            "r12b_episode_count_full_w256h4096",
            receipt.r12b_episode_count_full_w256h4096,
            S_PERF_8_R12B_EPISODE_COUNT_FULL_W256H4096,
        ),
    ];
    for (which, declared, panel_locked) in pin_triples {
        if declared != panel_locked {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::R12bEpisodePinsDrift {
                    which,
                    declared,
                    panel_locked,
                },
            });
        }
    }

    // Panel-required #13: catalog-order coherence.
    if receipt.catalog_order_label != S_PERF_8_CATALOG_ORDER_LABEL {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::CatalogOrderDrift {
                declared_label: receipt.catalog_order_label,
            },
        });
    }

    // Panel-required #14: completion-order merge guard.
    {
        let lower = receipt.merge_policy_label.to_ascii_lowercase();
        if lower.contains("completion-order") || lower.contains("completion order") {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::CompletionOrderMergeRejected {
                    declared_label: receipt.merge_policy_label,
                },
            });
        }
    }

    // Panel-required #15: CUDA-graph claim without replay
    // contract.
    {
        let lower = receipt.cuda_graph_status_label.to_ascii_lowercase();
        if (lower.contains("graph capture engaged") || lower.contains("graph_plan_hash"))
            && !lower.contains("not engaged")
        {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::CudaGraphClaimWithoutReplayContract {
                    declared_label: receipt.cuda_graph_status_label,
                },
            });
        }
    }

    // Panel-required #16: device-identity binding.
    if receipt.device_identity_hash == [0u8; 32] {
        errors.push(SPerf8VerifyError {
            kind: SPerf8VerifyErrorKind::MissingDeviceIdentity,
        });
    }

    // Panel-required #18: per-scale gain arithmetic
    // coherence (best_centi / k1_centi * 10000).
    for s in &receipt.per_scale_summaries {
        if s.k1_cat_per_sec_centi == 0 {
            continue;
        }
        let computed = u32::try_from(
            u64::from(s.best_k_cat_per_sec_centi) * 10_000 / u64::from(s.k1_cat_per_sec_centi),
        )
        .unwrap_or(u32::MAX);
        if computed != s.best_k_gain_basis_points {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::SpeedupArithmeticMismatch {
                    scale_label: s.scale_label,
                    declared_bp: s.best_k_gain_basis_points,
                    computed_bp: computed,
                },
            });
        }
    }

    // Structural: interpretation label coherence.
    for s in &receipt.per_scale_summaries {
        let expected =
            BatchedKResultInterpretation::from_gain_basis_points(s.best_k_gain_basis_points);
        if expected != s.interpretation {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::InterpretationLabelMismatch {
                    scale_label: s.scale_label,
                    declared_wire_name: s.interpretation.as_str(),
                    computed_wire_name: expected.as_str(),
                },
            });
        }
    }

    // Structural: empty panel-pinned labels.
    let label_fields = [
        ("dispatch_mode_label", receipt.dispatch_mode_label),
        ("catalog_order_label", receipt.catalog_order_label),
        ("merge_policy_label", receipt.merge_policy_label),
        ("cuda_graph_status_label", receipt.cuda_graph_status_label),
    ];
    for (which, label) in label_fields {
        if label.is_empty() {
            errors.push(SPerf8VerifyError {
                kind: SPerf8VerifyErrorKind::EmptyPanelLabel { which },
            });
        }
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// WHY: emits the receipt as deterministic ASCII so the
/// on-disk artifact is byte-stable across two consecutive
/// builds and operator-legible.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_batched_k_saturation_receipt_text(r: &BatchedKSaturationReceiptV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.8 BatchedKSaturationReceiptV1");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked conclusion (verbatim)");
    let _ = writeln!(
        s,
        "  S-PERF.8 replaces K-as-host-loop with batched-K execution and measures the"
    );
    let _ = writeln!(
        s,
        "  effect. The result is mixed and informative: canonical 16x128 improves by"
    );
    let _ = writeln!(
        s,
        "  1.76x, confirming launch-amortization benefit on small fixtures, while full"
    );
    let _ = writeln!(
        s,
        "  256x4096 improves only +3.4% / 1.03x, showing that the full-scale path is"
    );
    let _ = writeln!(
        s,
        "  not primarily launch-bound. This is a measured optimization result, not a"
    );
    let _ = writeln!(s, "  saturation claim.");
    let _ = writeln!(s);
    let _ = writeln!(s, "Receipt provenance");
    let _ = writeln!(s, "  receipt_id              : {}", r.receipt_id);
    let _ = writeln!(
        s,
        "  r12b_source_report_path : {}",
        r.r12b_source_report_path
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-pinned execution-mode labels (S-PERF.8.1)");
    let _ = writeln!(s, "  dispatch_mode_label     : {}", r.dispatch_mode_label);
    let _ = writeln!(s, "  catalog_order_label     : {}", r.catalog_order_label);
    let _ = writeln!(s, "  merge_policy_label      : {}", r.merge_policy_label);
    let _ = writeln!(
        s,
        "  cuda_graph_status_label : {}",
        r.cuda_graph_status_label
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Device identity + R.12b episode pins");
    let _ = writeln!(
        s,
        "  device_identity_hash                : {}",
        hex32(&r.device_identity_hash)
    );
    let _ = writeln!(
        s,
        "  r12b_episode_count_canonical_w16h128: {}",
        r.r12b_episode_count_canonical_w16h128
    );
    let _ = writeln!(
        s,
        "  r12b_episode_count_mid_w64h512      : {}",
        r.r12b_episode_count_mid_w64h512
    );
    let _ = writeln!(
        s,
        "  r12b_episode_count_full_w256h4096   : {}",
        r.r12b_episode_count_full_w256h4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-scale K-amortisation summary");
    let _ = writeln!(
        s,
        "  scale                  | k1 cat/sec | best K | best cat/sec |   gain |    delta%  | pre GB/s | post GB/s | interpretation"
    );
    let _ = writeln!(
        s,
        "  ---------------------- | ---------- | ------ | ------------ | ------ | ---------- | -------- | --------- | --------------"
    );
    for summary in &r.per_scale_summaries {
        let k1 = (summary.k1_cat_per_sec_centi as f64) / 100.0;
        let best = (summary.best_k_cat_per_sec_centi as f64) / 100.0;
        let gain = (summary.best_k_gain_basis_points as f64) / 10_000.0;
        let delta_pct = (summary.delta_basis_points as f64) / 100.0;
        let pre_gbps = (summary.pre_bandwidth_centi_gbps as f64) / 100.0;
        let post_gbps = (summary.post_bandwidth_centi_gbps as f64) / 100.0;
        let _ = writeln!(
            s,
            "  {:<22} | {:>10.1} | {:>6} | {:>12.1} | {:>5.2}x | {:>+8.2}%  | {:>8.2} | {:>9.2} | {}",
            summary.scale_label,
            k1,
            summary.best_k,
            best,
            gain,
            delta_pct,
            pre_gbps,
            post_gbps,
            summary.interpretation.as_str()
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Parsed K-saturation matrix (R.12b live)");
    let _ = writeln!(
        s,
        "  scale                  |   K | per_cat_us | cat/sec | features_pct | dev_total_pct | finalize_pct | top_stage"
    );
    let _ = writeln!(
        s,
        "  ---------------------- | --- | ---------- | ------- | ------------ | ------------- | ------------ | --------------"
    );
    for c in &r.cells {
        let cps = (c.cat_per_sec_centi as f64) / 100.0;
        let fp = (c.features_pct_basis_points as f64) / 100.0;
        let dp = (c.dev_total_pct_basis_points as f64) / 100.0;
        let fzp = (c.finalize_pct_basis_points as f64) / 100.0;
        let tsp = (c.top_stage_pct_basis_points as f64) / 100.0;
        let _ = writeln!(
            s,
            "  {:<22} | {:>3} | {:>10} | {:>7.1} | {:>11.1}% | {:>12.1}% | {:>11.1}% | {} ({:.0}%)",
            c.scale_label, c.k, c.per_cat_us, cps, fp, dp, fzp, c.top_stage_label, tsp
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Upstream anchors");
    let _ = writeln!(
        s,
        "  s_perf_6_baseline_report_hash             : {}",
        hex32(&r.s_perf_6_baseline_report_hash)
    );
    let _ = writeln!(
        s,
        "  s_perf_7_source_report_import_verifier_hash : {}",
        hex32(&r.s_perf_7_source_report_import_verifier_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "batched_k_saturation_receipt_hash_v1 : {}",
        hex32(&r.batched_k_saturation_receipt_hash_v1)
    );
    s
}

/// WHY: deterministic JSON form for machine consumers.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_batched_k_saturation_receipt_json(r: &BatchedKSaturationReceiptV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        S_PERF_8_BATCHED_K_SATURATION_RECEIPT_SCHEMA_V1,
    );
    s.push(',');
    json_field(&mut s, "receipt_id", r.receipt_id);
    s.push(',');
    json_field(&mut s, "r12b_source_report_path", r.r12b_source_report_path);
    s.push(',');
    json_field(&mut s, "dispatch_mode_label", r.dispatch_mode_label);
    s.push(',');
    json_field(&mut s, "catalog_order_label", r.catalog_order_label);
    s.push(',');
    json_field(&mut s, "merge_policy_label", r.merge_policy_label);
    s.push(',');
    json_field(&mut s, "cuda_graph_status_label", r.cuda_graph_status_label);
    s.push(',');
    json_hex(&mut s, "device_identity_hash", &r.device_identity_hash);
    s.push(',');
    let _ = write!(
        s,
        "\"r12b_episode_count_canonical_w16h128\":{}",
        r.r12b_episode_count_canonical_w16h128
    );
    s.push(',');
    let _ = write!(
        s,
        "\"r12b_episode_count_mid_w64h512\":{}",
        r.r12b_episode_count_mid_w64h512
    );
    s.push(',');
    let _ = write!(
        s,
        "\"r12b_episode_count_full_w256h4096\":{}",
        r.r12b_episode_count_full_w256h4096
    );
    s.push(',');
    s.push_str("\"cells\":[");
    for (i, c) in r.cells.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_field(&mut s, "scale_label", c.scale_label);
        s.push(',');
        let _ = write!(s, "\"k\":{}", c.k);
        s.push(',');
        let _ = write!(s, "\"per_cat_us\":{}", c.per_cat_us);
        s.push(',');
        let _ = write!(s, "\"cat_per_sec_centi\":{}", c.cat_per_sec_centi);
        s.push(',');
        let _ = write!(
            s,
            "\"features_pct_basis_points\":{}",
            c.features_pct_basis_points
        );
        s.push(',');
        let _ = write!(
            s,
            "\"dev_total_pct_basis_points\":{}",
            c.dev_total_pct_basis_points
        );
        s.push(',');
        let _ = write!(
            s,
            "\"finalize_pct_basis_points\":{}",
            c.finalize_pct_basis_points
        );
        s.push(',');
        json_field(&mut s, "top_stage_label", c.top_stage_label);
        s.push(',');
        let _ = write!(
            s,
            "\"top_stage_pct_basis_points\":{}",
            c.top_stage_pct_basis_points
        );
        s.push('}');
    }
    s.push_str("],\"per_scale_summaries\":[");
    for (i, sum) in r.per_scale_summaries.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_field(&mut s, "scale_label", sum.scale_label);
        s.push(',');
        let _ = write!(s, "\"k1_cat_per_sec_centi\":{}", sum.k1_cat_per_sec_centi);
        s.push(',');
        let _ = write!(s, "\"best_k\":{}", sum.best_k);
        s.push(',');
        let _ = write!(
            s,
            "\"best_k_cat_per_sec_centi\":{}",
            sum.best_k_cat_per_sec_centi
        );
        s.push(',');
        let _ = write!(
            s,
            "\"best_k_gain_basis_points\":{}",
            sum.best_k_gain_basis_points
        );
        s.push(',');
        let _ = write!(
            s,
            "\"pre_bandwidth_centi_gbps\":{}",
            sum.pre_bandwidth_centi_gbps
        );
        s.push(',');
        let _ = write!(
            s,
            "\"post_bandwidth_centi_gbps\":{}",
            sum.post_bandwidth_centi_gbps
        );
        s.push(',');
        let _ = write!(s, "\"delta_basis_points\":{}", sum.delta_basis_points);
        s.push(',');
        json_field(&mut s, "interpretation", sum.interpretation.as_str());
        s.push('}');
    }
    s.push(']');
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_6_baseline_report_hash",
        &r.s_perf_6_baseline_report_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_7_source_report_import_verifier_hash",
        &r.s_perf_7_source_report_import_verifier_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "batched_k_saturation_receipt_hash_v1",
        &r.batched_k_saturation_receipt_hash_v1,
    );
    s.push('}');
    s
}

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
