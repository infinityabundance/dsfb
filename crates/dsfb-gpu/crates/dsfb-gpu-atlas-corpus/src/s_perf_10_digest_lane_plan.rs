//! S-PERF.10 --- DigestLanePlanV1 / digest-cost audit.
//!
//! ## Commit identity (panel-locked, verbatim)
//!
//! > **S-PERF.10 audits the measured digest-lane bottleneck
//! > and emits DigestLanePlanV1. It does not claim bandwidth
//! > improvement. It defines the preservation contract that
//! > any future digest compaction kernel rewrite must
//! > satisfy.**
//!
//! ## Why S-PERF.10 exists
//!
//! The S-PERF.8.1 hardening pass proved K batching is NOT
//! the full-scale lever (only +3.43% gain at the headline
//! scale because the workload is device-bound). The
//! S-PERF.6 receipt + S-PERF.7 source-report parser already
//! record, but do not codify, that the four `tree_digest`
//! stages (consensus + detector + sign + residual)
//! collectively dominate device time. S-PERF.10 codifies
//! that finding as a hashable receipt AND writes the
//! byte-identical digest-root preservation contract that
//! any future digest rewrite must satisfy. The next commit
//! (named **S-PERF.11 --- measured digest-lane compaction**
//! or **S-PERF.10b**) performs the actual rewrite and must
//! move the scoreboard while preserving the digest roots
//! this audit pins.
//!
//! ## What this DOES
//!
//! - Parses the four `tree_digest` rows from
//!   `reports/d64_stage_timing_256x4096_K1.txt` (us +
//!   percent-of-device-total) and computes
//!   `digest_total_us` + `digest_total_pct`.
//! - Builds the panel-locked digest-root preservation
//!   contract: four declared laws (digest root law,
//!   fragment merge order law, digest mode identity law,
//!   casefile chain law) that any future digest rewrite
//!   MUST satisfy.
//! - Binds the parsed audit + contract + upstream S-PERF.6
//!   baseline hash + S-PERF.7 verifier hash + S-PERF.8.1
//!   receipt hash + R.12b full-scale episode pin (1917)
//!   into a single hashable `DigestLanePlanV1` envelope.
//!
//! ## What this DOES NOT do
//!
//! - Does NOT change kernels.
//! - Does NOT claim bandwidth improvement.
//! - Does NOT benchmark anything.
//! - Does NOT run any CUDA code.
//! - Does NOT compact digests (that is the S-PERF.11 /
//!   S-PERF.10b commit).
//! - Does NOT mutate the S-PERF.6 / S-PERF.7 / S-PERF.8
//!   receipts or any prior hash anchor.
//! - Does NOT alter `SEED.len()` (stays 54).
//! - Does NOT rebaseline R.12b.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes (none folded upstream):
//!
//! - `digest_stage_cost_audit_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-STAGE-COST-AUDIT:v1\0`.
//!   Pins the four parsed stage timings + total.
//! - `digest_compaction_contract_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-COMPACTION-CONTRACT:v1\0`.
//!   Pins the four preservation laws.
//! - `digest_lane_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-10-DIGEST-LANE-PLAN:v1\0`.
//!   Top-level META-hash binding the audit + contract +
//!   S-PERF.6 + S-PERF.7 + S-PERF.8 anchors + episode pin.
//!
//! ## Track B linkage
//!
//! S-PERF.10 is Track B leg 3 (receipt-only). It does NOT
//! change measured bandwidth; it pins the digest lane as
//! the next attack target and writes the preservation
//! contract. The follow-on commit (S-PERF.11 / S-PERF.10b)
//! performs the measured rewrite and must cite this plan's
//! hash so the rewrite contract is mechanically anchored
//! to the audit.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    seed_rtx4080_super_measured_baseline_report, R12B_EPISODE_COUNT_FULL_W256H4096,
    S_PERF_6_SOURCE_REPORT_PATH,
};
use crate::s_perf_7_source_report_import_verifier::seed_source_report_import_verifier_report_from_disk;
use crate::s_perf_8_batched_k_saturation_receipt::seed_batched_k_saturation_receipt_from_disk;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `digest_stage_cost_audit_hash_v1`.
pub const S_PERF_10_DIGEST_STAGE_COST_AUDIT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-STAGE-COST-AUDIT:v1\0";

/// Schema id for `digest_stage_cost_audit_hash_v1`.
pub const S_PERF_10_DIGEST_STAGE_COST_AUDIT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-STAGE-COST-AUDIT:v1";

/// Domain separator for `digest_compaction_contract_hash_v1`.
pub const S_PERF_10_DIGEST_COMPACTION_CONTRACT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-COMPACTION-CONTRACT:v1\0";

/// Schema id for `digest_compaction_contract_hash_v1`.
pub const S_PERF_10_DIGEST_COMPACTION_CONTRACT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-COMPACTION-CONTRACT:v1";

/// Domain separator for the top-level `digest_lane_plan_hash_v1`.
pub const S_PERF_10_DIGEST_LANE_PLAN_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-LANE-PLAN:v1\0";

/// Schema id for `digest_lane_plan_hash_v1`.
pub const S_PERF_10_DIGEST_LANE_PLAN_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-10-DIGEST-LANE-PLAN:v1";

/// Panel-pinned source-report path the audit parses (same
/// file S-PERF.7's `parse_d64_stage_timing` reads).
pub const S_PERF_10_SOURCE_REPORT_PATH: &str = S_PERF_6_SOURCE_REPORT_PATH;

/// Panel-pinned minimum band edge for the digest-share
/// audit (basis points). Measured digest share at the
/// panel-locked profile is ~57.3%; the band 50%..65%
/// leaves room for thermal / system load variance without
/// re-baselining each commit. Out-of-band fires
/// `DigestPlanWithoutTotalDigestShare`.
pub const S_PERF_10_DIGEST_SHARE_MIN_BP: u32 = 5000;
/// Panel-pinned maximum band edge for the digest-share
/// audit (basis points). See [`S_PERF_10_DIGEST_SHARE_MIN_BP`].
pub const S_PERF_10_DIGEST_SHARE_MAX_BP: u32 = 6500;

/// Panel-locked stage label for the `tree_digest residual`
/// row. The parser matches this prefix.
pub const S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL: &str = "tree_digest residual";
/// Panel-locked stage label for the `tree_digest sign`
/// row. The parser matches this prefix.
pub const S_PERF_10_TREE_DIGEST_STAGE_SIGN: &str = "tree_digest sign";
/// Panel-locked stage label for the
/// `tree_digest detector (wide cells)` row. The full
/// `(wide cells)` suffix is panel-locked so a future kernel
/// rename surfaces as a stage-label-mismatch verifier
/// rejection.
pub const S_PERF_10_TREE_DIGEST_STAGE_DETECTOR: &str = "tree_digest detector (wide cells)";
/// Panel-locked stage label for the `tree_digest consensus`
/// row. This is the largest of the four digest stages
/// (~20.9% of device_total at the panel-pinned profile).
pub const S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS: &str = "tree_digest consensus";

/// Panel-locked same-mode digest-root preservation law.
/// Renamed and clarified per the 2026-05-18 panel verdict
/// from the earlier `digest_root_law` whose wording could
/// be misread as requiring SerialSha256 and TreeSha256V1
/// to produce identical roots across modes (which the
/// existing CUDA TreeSha256V1 design intentionally
/// violates). The corrected reading: byte-identical roots
/// within the declared digest mode only; cross-mode root
/// equality is NOT required. Folded into the contract
/// hash so any drift rebaselines the contract. This is
/// the law the CAMPAIGN IDENTITY verifier negative
/// (`DigestOptimisationClaimWithoutByteIdenticalDigestRoots`)
/// enforces.
pub const S_PERF_10_SAME_MODE_DIGEST_ROOT_LAW: &str =
    "Future digest compaction MUST preserve byte-identical digest roots within the \
     declared digest mode. A TreeSha256V1 rewrite MUST preserve TreeSha256V1 roots. \
     A SerialSha256 path MUST preserve SerialSha256 roots. \
     Cross-mode root equality is NOT required.";
/// Panel-locked canonical-fragment-merge-order law.
/// Renamed from `fragment_merge_order_law` per the panel
/// verdict for clarity. Per-block digest fragments MUST
/// be merged in canonical order; completion-order
/// merging is panel-forbidden (mirrors the S-PERF.8.1
/// merge-policy discipline).
pub const S_PERF_10_CANONICAL_FRAGMENT_MERGE_ORDER_LAW: &str =
    "Per-block digest fragments MUST be merged in canonical order; \
     completion-order merging is panel-forbidden.";
/// Panel-locked digest-mode-non-aliasing law. Renamed and
/// clarified per the panel verdict from the earlier
/// `digest_mode_identity_law` whose wording could be
/// misread as requiring cross-mode root equality. The
/// corrected reading: each declared digest mode owns its
/// own root-byte projection and MUST NOT alias to roots
/// from a different mode; mode identity is preserved
/// within the mode.
pub const S_PERF_10_DIGEST_MODE_NON_ALIASING_LAW: &str =
    "Digest mode identifiers (SerialSha256 / TreeSha256V1 / TreeSha256V2 / ...) \
     MUST NOT alias to roots from a different mode; each declared mode owns its \
     own root-byte projection. Mode identity is preserved within the mode; \
     cross-mode roots are NOT required to match.";
/// Panel-locked casefile-chain preservation law. Renamed
/// from `casefile_chain_law` per the panel verdict for
/// clarity. The 12-link CaseFile per-stage hash chain MUST
/// stay byte-identical; no digest rewrite may insert,
/// remove, or reorder chain links.
pub const S_PERF_10_CASEFILE_CHAIN_PRESERVATION_LAW: &str =
    "CaseFile per-stage hash chain MUST stay byte-identical; \
     no digest rewrite may insert, remove, or reorder the 12 chain links.";

// ---------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------

/// Structurally-typed parse error for the four `tree_digest`
/// rows in `reports/d64_stage_timing_256x4096_K1.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// One of the four panel-locked `tree_digest` rows is
    /// absent from the source report.
    MissingTreeDigestRow {
        /// Which stage label was missing.
        stage: &'static str,
    },
    /// Numeric field could not be parsed (us or pct).
    MalformedNumber {
        /// Which numeric field failed to parse.
        field: &'static str,
    },
}

// ---------------------------------------------------------------
// ParsedDigestStageRowV1
// ---------------------------------------------------------------

/// One `tree_digest` stage row from the source report.
/// Field order is the canonical hash order; do not reorder
/// without rebaselining the audit hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDigestStageRowV1 {
    /// Panel-locked stage label (e.g. `"tree_digest residual"`).
    pub stage_label: &'static str,
    /// Median per-stage device time in microseconds.
    pub us: u64,
    /// Share of `total_device_us` for this stage, basis
    /// points (1234 = 12.34%).
    pub pct_basis_points: u32,
}

// ---------------------------------------------------------------
// DigestStageCostAuditV1
// ---------------------------------------------------------------

/// Parsed view of the digest-lane share of device time.
/// Records all four `tree_digest` rows + the computed total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestStageCostAuditV1 {
    /// `tree_digest residual` row.
    pub residual: ParsedDigestStageRowV1,
    /// `tree_digest sign` row.
    pub sign: ParsedDigestStageRowV1,
    /// `tree_digest detector (wide cells)` row.
    pub detector: ParsedDigestStageRowV1,
    /// `tree_digest consensus` row.
    pub consensus: ParsedDigestStageRowV1,
    /// Sum of the four stage `us` values.
    pub digest_total_us: u64,
    /// Sum of the four stage `pct_basis_points` values.
    pub digest_total_pct_basis_points: u32,
    /// Source-report path the audit parsed.
    pub source_report_path: &'static str,
    /// SHA-256 over the canonical-byte projection.
    pub digest_stage_cost_audit_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// DigestCompactionContractV1
// ---------------------------------------------------------------

/// Panel-locked digest-root preservation contract. Any
/// future digest rewrite (S-PERF.11 / S-PERF.10b) MUST
/// satisfy every law verbatim. The contract is folded into
/// `digest_compaction_contract_hash_v1` so a future
/// commit's "preserve contract" claim is mechanically
/// verifiable against this snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestCompactionContractV1 {
    /// Digest-root law: byte-identical roots invariant.
    pub same_mode_digest_root_law: &'static str,
    /// Fragment merge order law: canonical, not
    /// completion-order.
    pub canonical_fragment_merge_order_law: &'static str,
    /// Digest mode identity law: SerialSha256 /
    /// TreeSha256V1 produce identical roots under fixed
    /// mode.
    pub digest_mode_non_aliasing_law: &'static str,
    /// CaseFile chain law: 12 chain links stay
    /// byte-identical.
    pub casefile_chain_preservation_law: &'static str,
    /// SHA-256 over the canonical-byte projection of the
    /// four laws above.
    pub digest_compaction_contract_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// DigestLanePlanV1
// ---------------------------------------------------------------

/// Top-level S-PERF.10 envelope binding the digest-cost
/// audit + preservation contract + upstream anchors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestLanePlanV1 {
    /// Plan identifier (canonical wire name).
    pub plan_id: &'static str,
    /// Parsed audit of the four `tree_digest` stages.
    pub audit: DigestStageCostAuditV1,
    /// Panel-locked digest-root preservation contract.
    pub contract: DigestCompactionContractV1,
    /// Upstream S-PERF.6 measured baseline-report hash.
    pub s_perf_6_baseline_report_hash_v1: [u8; 32],
    /// Upstream S-PERF.7 source-report-import verifier
    /// hash.
    pub s_perf_7_source_report_import_verifier_hash_v1: [u8; 32],
    /// Upstream S-PERF.8 batched-K saturation receipt
    /// hash (S-PERF.8.1 sealed).
    pub s_perf_8_batched_k_saturation_receipt_hash_v1: [u8; 32],
    /// Panel-pinned full-scale R.12b episode count
    /// (canonical 1917).
    pub r12b_episode_count_full_w256h4096: u32,
    /// Top-level META-hash binding everything above.
    pub digest_lane_plan_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// SPerf10VerifyError
// ---------------------------------------------------------------

/// Reject kinds the verifier emits. Each one corresponds to
/// a panel-required negative or a structural defect rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SPerf10VerifyErrorKind {
    /// Panel-required #1 (CAMPAIGN IDENTITY): any future
    /// digest compaction claim citing this plan MUST
    /// preserve byte-identical digest roots; this rule
    /// fires when the digest-root law text is missing or
    /// has been weakened.
    DigestOptimisationClaimWithoutByteIdenticalDigestRoots,
    /// Panel-required #2: parser must produce all four
    /// `tree_digest` rows.
    DigestPlanWithoutFourTreeDigestStageTimings,
    /// Panel-required #3: `digest_total_pct` must lie
    /// inside the panel-pinned band [50%, 65%].
    DigestPlanWithoutTotalDigestShare,
    /// Panel-required #4: top-level hash MUST bind the
    /// S-PERF.8.1 receipt hash (not `[0u8; 32]`).
    DigestPlanWithoutSPerf81Anchor,
    /// Panel-required #5: top-level hash MUST bind the
    /// S-PERF.6 measured baseline hash.
    DigestPlanWithoutSPerf6MeasuredBaselineAnchor,
    /// Panel-required #6: ANY field contains a forbidden
    /// bandwidth-improvement claim phrase.
    DigestPlanThatClaimsBandwidthImprovement,
    /// Panel-required #7: contract's four preservation
    /// laws must all be non-empty.
    DigestPlanWithoutFutureRewriteContract,
    /// Panel-required #8: full-scale episode count must
    /// equal panel-pinned 1917.
    DigestPlanWithEpisodeCountDrift,
    /// Structural defect: empty plan_id.
    PlanIdEmpty,
    /// Structural defect: empty source_report_path.
    SourceReportPathEmpty,
    /// Structural defect: parser failed to find a stage
    /// label OR the stage label was unexpected.
    StageLabelMismatch,
    /// Structural defect: digest_total_us != sum of the
    /// four stage us values.
    DigestTotalUsArithmeticMismatch,
    /// Structural defect: digest_total_pct_basis_points
    /// != sum of the four stage pct values (within ±10
    /// bp tolerance for rounding).
    DigestTotalPctArithmeticMismatch,
    /// Structural defect: S-PERF.7 verifier hash is the
    /// zero hash (binding absent).
    DigestPlanWithoutSPerf7VerifierAnchor,
}

/// Verifier error with a structured kind + field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf10VerifyError {
    /// Which rule was violated.
    pub kind: SPerf10VerifyErrorKind,
    /// Operator-legible field-name hint (e.g. "audit",
    /// "contract", "plan_id").
    pub field: &'static str,
}

/// Panel-locked forbidden-phrase scanner for the
/// `DigestPlanThatClaimsBandwidthImprovement` rule. S-PERF.10
/// is an audit; any field claiming a measurement-style
/// improvement is panel-forbidden and must wait for
/// S-PERF.11 / S-PERF.10b.
pub const S_PERF_10_FORBIDDEN_BANDWIDTH_CLAIM_SUBSTRINGS: &[&str] = &[
    "bandwidth improvement",
    "speedup",
    "gb/s gain",
    "saturation reached",
    "ratchets the scoreboard",
    "moves the scoreboard",
    "world record",
    "achieves saturation",
    "achieved saturation",
    "production-ready",
    "production performance",
    "outperforms",
];

// ---------------------------------------------------------------
// Parser
// ---------------------------------------------------------------

/// WHY: walks the on-disk d64 stage-timing source report
/// and extracts the four panel-locked `tree_digest` rows.
/// The parser does NOT extend S-PERF.7's parser (keeps that
/// hash chain byte-identical); reads the same source report
/// independently.
///
/// Returns the four stage rows + total + share on success,
/// or a structurally-typed [`ParseError`] on any missing or
/// malformed row.
///
/// # Errors
///
/// Returns [`ParseError`] when any of the four `tree_digest`
/// rows is absent or malformed.
pub fn parse_digest_stage_costs(text: &str) -> Result<DigestStageCostAuditV1, ParseError> {
    let mut residual: Option<(u64, u32)> = None;
    let mut sign: Option<(u64, u32)> = None;
    let mut detector: Option<(u64, u32)> = None;
    let mut consensus: Option<(u64, u32)> = None;

    for raw in text.lines() {
        let line = raw.trim();
        // Order the prefix matches so the more-specific
        // labels win first (`tree_digest detector (wide cells)`
        // before any future `tree_digest detector` short form).
        if line.starts_with("tree_digest detector (wide cells)") {
            detector = Some(parse_us_and_pct(line, "tree_digest_detector")?);
        } else if line.starts_with("tree_digest consensus") {
            consensus = Some(parse_us_and_pct(line, "tree_digest_consensus")?);
        } else if line.starts_with("tree_digest residual") {
            residual = Some(parse_us_and_pct(line, "tree_digest_residual")?);
        } else if line.starts_with("tree_digest sign") {
            sign = Some(parse_us_and_pct(line, "tree_digest_sign")?);
        }
    }

    let (residual_us, residual_pct) = residual.ok_or(ParseError::MissingTreeDigestRow {
        stage: S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
    })?;
    let (sign_us, sign_pct) = sign.ok_or(ParseError::MissingTreeDigestRow {
        stage: S_PERF_10_TREE_DIGEST_STAGE_SIGN,
    })?;
    let (detector_us, detector_pct) = detector.ok_or(ParseError::MissingTreeDigestRow {
        stage: S_PERF_10_TREE_DIGEST_STAGE_DETECTOR,
    })?;
    let (consensus_us, consensus_pct) = consensus.ok_or(ParseError::MissingTreeDigestRow {
        stage: S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS,
    })?;

    let digest_total_us = residual_us + sign_us + detector_us + consensus_us;
    let digest_total_pct_basis_points = residual_pct + sign_pct + detector_pct + consensus_pct;

    let mut audit = DigestStageCostAuditV1 {
        residual: ParsedDigestStageRowV1 {
            stage_label: S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL,
            us: residual_us,
            pct_basis_points: residual_pct,
        },
        sign: ParsedDigestStageRowV1 {
            stage_label: S_PERF_10_TREE_DIGEST_STAGE_SIGN,
            us: sign_us,
            pct_basis_points: sign_pct,
        },
        detector: ParsedDigestStageRowV1 {
            stage_label: S_PERF_10_TREE_DIGEST_STAGE_DETECTOR,
            us: detector_us,
            pct_basis_points: detector_pct,
        },
        consensus: ParsedDigestStageRowV1 {
            stage_label: S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS,
            us: consensus_us,
            pct_basis_points: consensus_pct,
        },
        digest_total_us,
        digest_total_pct_basis_points,
        source_report_path: S_PERF_10_SOURCE_REPORT_PATH,
        digest_stage_cost_audit_hash_v1: [0u8; 32],
    };
    audit.digest_stage_cost_audit_hash_v1 = compute_digest_stage_cost_audit_hash(&audit);
    Ok(audit)
}

/// WHY: stage rows look like
/// `tree_digest residual               |      2364 |  11.4`.
/// We split on `|` and read the integer us (col 1) plus
/// the decimal percent (col 2) which we convert to basis
/// points (11.4 -> 1140).
fn parse_us_and_pct(line: &str, field: &'static str) -> Result<(u64, u32), ParseError> {
    let mut parts = line.split('|');
    parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let us_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let pct_part = parts.next().ok_or(ParseError::MalformedNumber { field })?;
    let us = us_part
        .trim()
        .parse::<u64>()
        .map_err(|_| ParseError::MalformedNumber { field })?;
    let pct_basis_points = parse_decimal_pct_to_basis_points(pct_part.trim(), field)?;
    Ok((us, pct_basis_points))
}

/// WHY: the percent column is `11.4` style (one decimal
/// place). Convert to basis points: 11.4 -> 1140.
fn parse_decimal_pct_to_basis_points(text: &str, field: &'static str) -> Result<u32, ParseError> {
    let (int_part, frac_part) = text
        .split_once('.')
        .ok_or(ParseError::MalformedNumber { field })?;
    let int_v: u32 = int_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })?;
    // Frac part is a single decimal digit; if present as
    // longer (e.g. "10.42"), interpret with 2 decimals.
    let frac_v: u32 = frac_part
        .parse::<u32>()
        .map_err(|_| ParseError::MalformedNumber { field })?;
    let multiplier = match frac_part.len() {
        1 => 10,
        2 => 1,
        _ => return Err(ParseError::MalformedNumber { field }),
    };
    int_v
        .checked_mul(100)
        .and_then(|v| v.checked_add(frac_v * multiplier))
        .ok_or(ParseError::MalformedNumber { field })
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

/// WHY: canonical-byte projection of the audit. Field order
/// MUST stay stable. Two builds against the same source
/// report produce byte-identical hashes.
#[must_use]
pub fn compute_digest_stage_cost_audit_hash(a: &DigestStageCostAuditV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_10_DIGEST_STAGE_COST_AUDIT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(S_PERF_10_DIGEST_STAGE_COST_AUDIT_SCHEMA_V1.as_bytes());
    push_stage_row(&mut buf, &a.residual);
    push_stage_row(&mut buf, &a.sign);
    push_stage_row(&mut buf, &a.detector);
    push_stage_row(&mut buf, &a.consensus);
    buf.extend_from_slice(&a.digest_total_us.to_be_bytes());
    buf.extend_from_slice(&a.digest_total_pct_basis_points.to_be_bytes());
    push_len_prefixed_str(&mut buf, a.source_report_path);
    sha256(&buf)
}

/// WHY: canonical-byte projection of the panel-locked
/// preservation contract. Any drift in any law text
/// rebaselines this hash, surfacing tampering.
#[must_use]
pub fn compute_digest_compaction_contract_hash(c: &DigestCompactionContractV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_10_DIGEST_COMPACTION_CONTRACT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(S_PERF_10_DIGEST_COMPACTION_CONTRACT_SCHEMA_V1.as_bytes());
    push_len_prefixed_str(&mut buf, c.same_mode_digest_root_law);
    push_len_prefixed_str(&mut buf, c.canonical_fragment_merge_order_law);
    push_len_prefixed_str(&mut buf, c.digest_mode_non_aliasing_law);
    push_len_prefixed_str(&mut buf, c.casefile_chain_preservation_law);
    sha256(&buf)
}

/// WHY: top-level META-hash binding the audit + contract +
/// upstream S-PERF.6 / S-PERF.7 / S-PERF.8 anchors + the
/// R.12b full-scale episode pin. Any drift in any field
/// rebaselines this hash, surfacing tampering anywhere in
/// the chain.
#[must_use]
pub fn compute_digest_lane_plan_hash(p: &DigestLanePlanV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_10_DIGEST_LANE_PLAN_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(S_PERF_10_DIGEST_LANE_PLAN_SCHEMA_V1.as_bytes());
    push_len_prefixed_str(&mut buf, p.plan_id);
    buf.extend_from_slice(&p.audit.digest_stage_cost_audit_hash_v1);
    buf.extend_from_slice(&p.contract.digest_compaction_contract_hash_v1);
    buf.extend_from_slice(&p.s_perf_6_baseline_report_hash_v1);
    buf.extend_from_slice(&p.s_perf_7_source_report_import_verifier_hash_v1);
    buf.extend_from_slice(&p.s_perf_8_batched_k_saturation_receipt_hash_v1);
    buf.extend_from_slice(&p.r12b_episode_count_full_w256h4096.to_be_bytes());
    sha256(&buf)
}

/// Folds one stage row into the audit hash buffer.
fn push_stage_row(buf: &mut Vec<u8>, r: &ParsedDigestStageRowV1) {
    push_len_prefixed_str(buf, r.stage_label);
    buf.extend_from_slice(&r.us.to_be_bytes());
    buf.extend_from_slice(&r.pct_basis_points.to_be_bytes());
}

/// Length-prefixed string append (u64 BE length, then raw
/// bytes). Mirrors the canonical-byte layout every other
/// S-PERF receipt uses.
fn push_len_prefixed_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u64).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// WHY: builds the panel-locked preservation contract from
/// the four panel-locked law constants. Future digest
/// rewrites cite this snapshot (via the hash) so any drift
/// in the law text is mechanically detected.
#[must_use]
pub fn build_digest_compaction_contract() -> DigestCompactionContractV1 {
    let mut c = DigestCompactionContractV1 {
        same_mode_digest_root_law: S_PERF_10_SAME_MODE_DIGEST_ROOT_LAW,
        canonical_fragment_merge_order_law: S_PERF_10_CANONICAL_FRAGMENT_MERGE_ORDER_LAW,
        digest_mode_non_aliasing_law: S_PERF_10_DIGEST_MODE_NON_ALIASING_LAW,
        casefile_chain_preservation_law: S_PERF_10_CASEFILE_CHAIN_PRESERVATION_LAW,
        digest_compaction_contract_hash_v1: [0u8; 32],
    };
    c.digest_compaction_contract_hash_v1 = compute_digest_compaction_contract_hash(&c);
    c
}

/// WHY: assembles the top-level [`DigestLanePlanV1`] from
/// already-built parts. The caller supplies the audit (from
/// `parse_digest_stage_costs`), the contract (from
/// `build_digest_compaction_contract`), the three upstream
/// anchor hashes, and the full-scale episode pin. This
/// function pins the canonical hash order and computes the
/// final META-hash.
#[must_use]
pub fn build_digest_lane_plan(
    plan_id: &'static str,
    audit: DigestStageCostAuditV1,
    contract: DigestCompactionContractV1,
    s_perf_6_baseline_report_hash_v1: [u8; 32],
    s_perf_7_source_report_import_verifier_hash_v1: [u8; 32],
    s_perf_8_batched_k_saturation_receipt_hash_v1: [u8; 32],
    r12b_episode_count_full_w256h4096: u32,
) -> DigestLanePlanV1 {
    let mut p = DigestLanePlanV1 {
        plan_id,
        audit,
        contract,
        s_perf_6_baseline_report_hash_v1,
        s_perf_7_source_report_import_verifier_hash_v1,
        s_perf_8_batched_k_saturation_receipt_hash_v1,
        r12b_episode_count_full_w256h4096,
        digest_lane_plan_hash_v1: [0u8; 32],
    };
    p.digest_lane_plan_hash_v1 = compute_digest_lane_plan_hash(&p);
    p
}

// ---------------------------------------------------------------
// Live-disk seed
// ---------------------------------------------------------------

/// Errors returned by [`seed_digest_lane_plan_from_disk`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedError {
    /// Could not read the d64 stage timing source report.
    ReadSourceReport(String),
    /// Source report was malformed.
    ParseSourceReport(ParseError),
    /// Could not seed the S-PERF.7 verifier (needed for
    /// its hash binding).
    SeedSPerf7(String),
    /// Could not seed the S-PERF.8 receipt (needed for
    /// its hash binding).
    SeedSPerf8(String),
}

/// WHY: convenience seed that walks the on-disk source
/// report, builds the audit + preservation contract,
/// imports the S-PERF.7 verifier hash and the S-PERF.8.1
/// receipt hash, and assembles the top-level plan. Returns
/// a fully-pinned `DigestLanePlanV1`.
///
/// # Errors
///
/// Returns [`SeedError`] when any of the disk reads or
/// upstream seed builds fails.
pub fn seed_digest_lane_plan_from_disk(
    repo_root: &std::path::Path,
) -> Result<DigestLanePlanV1, SeedError> {
    let d64_path = repo_root.join(S_PERF_10_SOURCE_REPORT_PATH);
    let d64_text = std::fs::read_to_string(&d64_path)
        .map_err(|e| SeedError::ReadSourceReport(format!("{}: {e}", d64_path.display())))?;
    let audit = parse_digest_stage_costs(&d64_text).map_err(SeedError::ParseSourceReport)?;
    let contract = build_digest_compaction_contract();
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let s_perf_7 = seed_source_report_import_verifier_report_from_disk(repo_root)
        .map_err(|e| SeedError::SeedSPerf7(format!("{e:?}")))?;
    let s_perf_8 = seed_batched_k_saturation_receipt_from_disk(repo_root)
        .map_err(|e| SeedError::SeedSPerf8(format!("{e:?}")))?;
    Ok(build_digest_lane_plan(
        "s_perf_10_digest_lane_plan_v1",
        audit,
        contract,
        baseline.rtx4080_super_measured_baseline_report_hash_v1,
        s_perf_7.source_report_import_verifier_hash_v1,
        s_perf_8.batched_k_saturation_receipt_hash_v1,
        R12B_EPISODE_COUNT_FULL_W256H4096,
    ))
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// WHY: walks the eight panel-required negatives + the
/// structural defect rules. Returns the list of failed
/// rules; an empty list means the plan is admissible.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_digest_lane_plan(p: &DigestLanePlanV1) -> Vec<SPerf10VerifyError> {
    let mut errors: Vec<SPerf10VerifyError> = Vec::new();

    // Structural: plan_id non-empty.
    if p.plan_id.is_empty() {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::PlanIdEmpty,
            field: "plan_id",
        });
    }
    // Structural: source_report_path non-empty.
    if p.audit.source_report_path.is_empty() {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::SourceReportPathEmpty,
            field: "audit.source_report_path",
        });
    }
    // Panel-required #2: all four stage labels match the
    // panel-pinned labels.
    if p.audit.residual.stage_label != S_PERF_10_TREE_DIGEST_STAGE_RESIDUAL
        || p.audit.sign.stage_label != S_PERF_10_TREE_DIGEST_STAGE_SIGN
        || p.audit.detector.stage_label != S_PERF_10_TREE_DIGEST_STAGE_DETECTOR
        || p.audit.consensus.stage_label != S_PERF_10_TREE_DIGEST_STAGE_CONSENSUS
    {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutFourTreeDigestStageTimings,
            field: "audit.stage_label",
        });
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::StageLabelMismatch,
            field: "audit.stage_label",
        });
    }
    // Panel-required #2 (continued): all four stage us
    // values must be non-zero.
    if p.audit.residual.us == 0
        || p.audit.sign.us == 0
        || p.audit.detector.us == 0
        || p.audit.consensus.us == 0
    {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutFourTreeDigestStageTimings,
            field: "audit.*.us",
        });
    }
    // Structural: digest_total_us == sum of four stage us.
    let expected_total_us =
        p.audit.residual.us + p.audit.sign.us + p.audit.detector.us + p.audit.consensus.us;
    if p.audit.digest_total_us != expected_total_us {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestTotalUsArithmeticMismatch,
            field: "audit.digest_total_us",
        });
    }
    // Structural: digest_total_pct ≈ sum of four stage
    // pct (within ±10 bp for rounding).
    let expected_total_pct = p.audit.residual.pct_basis_points
        + p.audit.sign.pct_basis_points
        + p.audit.detector.pct_basis_points
        + p.audit.consensus.pct_basis_points;
    if (i64::from(p.audit.digest_total_pct_basis_points) - i64::from(expected_total_pct)).abs() > 10
    {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestTotalPctArithmeticMismatch,
            field: "audit.digest_total_pct_basis_points",
        });
    }
    // Panel-required #3: digest_total_pct must lie inside
    // the panel-pinned band [50%, 65%].
    if p.audit.digest_total_pct_basis_points < S_PERF_10_DIGEST_SHARE_MIN_BP
        || p.audit.digest_total_pct_basis_points > S_PERF_10_DIGEST_SHARE_MAX_BP
    {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutTotalDigestShare,
            field: "audit.digest_total_pct_basis_points",
        });
    }
    // Panel-required #1 (CAMPAIGN IDENTITY): the digest
    // root law must be the panel-locked text. Any drift
    // means the preservation rule has been weakened.
    if p.contract.same_mode_digest_root_law != S_PERF_10_SAME_MODE_DIGEST_ROOT_LAW {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestOptimisationClaimWithoutByteIdenticalDigestRoots,
            field: "contract.same_mode_digest_root_law",
        });
    }
    // Panel-required #7: the four preservation laws must
    // all be non-empty.
    if p.contract.same_mode_digest_root_law.is_empty()
        || p.contract.canonical_fragment_merge_order_law.is_empty()
        || p.contract.digest_mode_non_aliasing_law.is_empty()
        || p.contract.casefile_chain_preservation_law.is_empty()
    {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutFutureRewriteContract,
            field: "contract.*_law",
        });
    }
    // Panel-required #4: S-PERF.8.1 receipt hash bound.
    if p.s_perf_8_batched_k_saturation_receipt_hash_v1 == [0u8; 32] {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutSPerf81Anchor,
            field: "s_perf_8_batched_k_saturation_receipt_hash_v1",
        });
    }
    // Panel-required #5: S-PERF.6 baseline hash bound.
    if p.s_perf_6_baseline_report_hash_v1 == [0u8; 32] {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutSPerf6MeasuredBaselineAnchor,
            field: "s_perf_6_baseline_report_hash_v1",
        });
    }
    // Structural: S-PERF.7 verifier hash bound (paired
    // with the panel-required S-PERF.8.1 + S-PERF.6 anchors;
    // the receipt cites the verifier the bench rerun would
    // exercise, so absence is a structural defect).
    if p.s_perf_7_source_report_import_verifier_hash_v1 == [0u8; 32] {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithoutSPerf7VerifierAnchor,
            field: "s_perf_7_source_report_import_verifier_hash_v1",
        });
    }
    // Panel-required #8: episode count must equal panel-
    // pinned 1917 (full 256x4096).
    if p.r12b_episode_count_full_w256h4096 != R12B_EPISODE_COUNT_FULL_W256H4096 {
        errors.push(SPerf10VerifyError {
            kind: SPerf10VerifyErrorKind::DigestPlanWithEpisodeCountDrift,
            field: "r12b_episode_count_full_w256h4096",
        });
    }
    // Panel-required #6: forbidden-phrase scanner over
    // every label field. S-PERF.10 is an audit; any field
    // that smuggles in a bandwidth-improvement claim is
    // rejected.
    let label_fields = [
        p.plan_id,
        p.audit.source_report_path,
        p.contract.same_mode_digest_root_law,
        p.contract.canonical_fragment_merge_order_law,
        p.contract.digest_mode_non_aliasing_law,
        p.contract.casefile_chain_preservation_law,
    ];
    for field in label_fields {
        let lower = field.to_ascii_lowercase();
        for forbidden in S_PERF_10_FORBIDDEN_BANDWIDTH_CLAIM_SUBSTRINGS {
            if lower.contains(*forbidden) {
                errors.push(SPerf10VerifyError {
                    kind: SPerf10VerifyErrorKind::DigestPlanThatClaimsBandwidthImprovement,
                    field: "label",
                });
                break;
            }
        }
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// WHY: emits the plan as deterministic ASCII so the
/// on-disk artifact is byte-stable across two consecutive
/// builds and operator-legible.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_digest_lane_plan_text(p: &DigestLanePlanV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.10 DigestLanePlanV1");
    let _ = writeln!(s, "===========================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked conclusion (verbatim)");
    let _ = writeln!(
        s,
        "  S-PERF.10 audits the measured digest-lane bottleneck and emits"
    );
    let _ = writeln!(
        s,
        "  DigestLanePlanV1. It does not claim bandwidth improvement. It defines"
    );
    let _ = writeln!(
        s,
        "  the preservation contract that any future digest compaction kernel"
    );
    let _ = writeln!(s, "  rewrite must satisfy.");
    let _ = writeln!(s);
    let _ = writeln!(s, "Plan provenance");
    let _ = writeln!(s, "  plan_id                 : {}", p.plan_id);
    let _ = writeln!(
        s,
        "  source_report_path      : {}",
        p.audit.source_report_path
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Digest-lane share (live R.12b source report)");
    let _ = writeln!(s, "  stage                              |    us   |  pct  ");
    let _ = writeln!(s, "  ---------------------------------- | ------- | ------");
    let _ = writeln!(
        s,
        "  {:<34} | {:>7} | {:>5}%",
        p.audit.residual.stage_label,
        p.audit.residual.us,
        format_bp_to_percent(p.audit.residual.pct_basis_points)
    );
    let _ = writeln!(
        s,
        "  {:<34} | {:>7} | {:>5}%",
        p.audit.sign.stage_label,
        p.audit.sign.us,
        format_bp_to_percent(p.audit.sign.pct_basis_points)
    );
    let _ = writeln!(
        s,
        "  {:<34} | {:>7} | {:>5}%",
        p.audit.detector.stage_label,
        p.audit.detector.us,
        format_bp_to_percent(p.audit.detector.pct_basis_points)
    );
    let _ = writeln!(
        s,
        "  {:<34} | {:>7} | {:>5}%",
        p.audit.consensus.stage_label,
        p.audit.consensus.us,
        format_bp_to_percent(p.audit.consensus.pct_basis_points)
    );
    let _ = writeln!(s, "  ---------------------------------- | ------- | ------");
    let _ = writeln!(
        s,
        "  {:<34} | {:>7} | {:>5}%",
        "digest_total",
        p.audit.digest_total_us,
        format_bp_to_percent(p.audit.digest_total_pct_basis_points)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Preservation contract (panel-locked; future");
    let _ = writeln!(s, "S-PERF.11/10b rewrite MUST satisfy each law)");
    let _ = writeln!(s, "  same_mode_digest_root_law:");
    write_wrapped_law(&mut s, p.contract.same_mode_digest_root_law);
    let _ = writeln!(s, "  canonical_fragment_merge_order_law:");
    write_wrapped_law(&mut s, p.contract.canonical_fragment_merge_order_law);
    let _ = writeln!(s, "  digest_mode_non_aliasing_law:");
    write_wrapped_law(&mut s, p.contract.digest_mode_non_aliasing_law);
    let _ = writeln!(s, "  casefile_chain_preservation_law:");
    write_wrapped_law(&mut s, p.contract.casefile_chain_preservation_law);
    let _ = writeln!(s);
    let _ = writeln!(s, "Upstream anchor bindings");
    let _ = writeln!(
        s,
        "  s_perf_6_baseline_report_hash_v1            : {}",
        hex_str(&p.s_perf_6_baseline_report_hash_v1)
    );
    let _ = writeln!(
        s,
        "  s_perf_7_source_report_import_verifier_hash : {}",
        hex_str(&p.s_perf_7_source_report_import_verifier_hash_v1)
    );
    let _ = writeln!(
        s,
        "  s_perf_8_batched_k_saturation_receipt_hash  : {}",
        hex_str(&p.s_perf_8_batched_k_saturation_receipt_hash_v1)
    );
    let _ = writeln!(
        s,
        "  r12b_episode_count_full_w256h4096           : {}",
        p.r12b_episode_count_full_w256h4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Hash provenance (own namespace, none folded upstream)");
    let _ = writeln!(
        s,
        "  digest_stage_cost_audit_hash_v1     : {}",
        hex_str(&p.audit.digest_stage_cost_audit_hash_v1)
    );
    let _ = writeln!(
        s,
        "  digest_compaction_contract_hash_v1  : {}",
        hex_str(&p.contract.digest_compaction_contract_hash_v1)
    );
    let _ = writeln!(
        s,
        "  digest_lane_plan_hash_v1            : {}",
        hex_str(&p.digest_lane_plan_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked non-claims");
    let _ = writeln!(s, "  - Does NOT change kernels.");
    let _ = writeln!(s, "  - Does NOT claim bandwidth improvement.");
    let _ = writeln!(s, "  - Does NOT benchmark anything.");
    let _ = writeln!(s, "  - Does NOT run any CUDA code.");
    let _ = writeln!(
        s,
        "  - Does NOT compact digests (that is S-PERF.11 / S-PERF.10b)."
    );
    let _ = writeln!(
        s,
        "  - Does NOT mutate the S-PERF.6 / S-PERF.7 / S-PERF.8 receipts or any"
    );
    let _ = writeln!(s, "    prior hash anchor.");
    let _ = writeln!(s, "  - Does NOT alter SEED.len() (stays 54).");
    let _ = writeln!(s, "  - Does NOT rebaseline R.12b.");
    s
}

/// WHY: deterministic JSON form for machine consumers.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_digest_lane_plan_json(p: &DigestLanePlanV1) -> String {
    let mut s = String::new();
    s.push('{');
    write_json_str(&mut s, "plan_id", p.plan_id);
    s.push(',');
    write_json_str(&mut s, "source_report_path", p.audit.source_report_path);
    s.push(',');
    s.push_str("\"audit\":{");
    write_stage_json(&mut s, "residual", &p.audit.residual);
    s.push(',');
    write_stage_json(&mut s, "sign", &p.audit.sign);
    s.push(',');
    write_stage_json(&mut s, "detector", &p.audit.detector);
    s.push(',');
    write_stage_json(&mut s, "consensus", &p.audit.consensus);
    s.push(',');
    write_json_u64(&mut s, "digest_total_us", p.audit.digest_total_us);
    s.push(',');
    write_json_u32(
        &mut s,
        "digest_total_pct_basis_points",
        p.audit.digest_total_pct_basis_points,
    );
    s.push(',');
    write_json_hash(
        &mut s,
        "digest_stage_cost_audit_hash_v1",
        &p.audit.digest_stage_cost_audit_hash_v1,
    );
    s.push('}');
    s.push(',');
    s.push_str("\"contract\":{");
    write_json_str(
        &mut s,
        "same_mode_digest_root_law",
        p.contract.same_mode_digest_root_law,
    );
    s.push(',');
    write_json_str(
        &mut s,
        "canonical_fragment_merge_order_law",
        p.contract.canonical_fragment_merge_order_law,
    );
    s.push(',');
    write_json_str(
        &mut s,
        "digest_mode_non_aliasing_law",
        p.contract.digest_mode_non_aliasing_law,
    );
    s.push(',');
    write_json_str(
        &mut s,
        "casefile_chain_preservation_law",
        p.contract.casefile_chain_preservation_law,
    );
    s.push(',');
    write_json_hash(
        &mut s,
        "digest_compaction_contract_hash_v1",
        &p.contract.digest_compaction_contract_hash_v1,
    );
    s.push('}');
    s.push(',');
    write_json_hash(
        &mut s,
        "s_perf_6_baseline_report_hash_v1",
        &p.s_perf_6_baseline_report_hash_v1,
    );
    s.push(',');
    write_json_hash(
        &mut s,
        "s_perf_7_source_report_import_verifier_hash_v1",
        &p.s_perf_7_source_report_import_verifier_hash_v1,
    );
    s.push(',');
    write_json_hash(
        &mut s,
        "s_perf_8_batched_k_saturation_receipt_hash_v1",
        &p.s_perf_8_batched_k_saturation_receipt_hash_v1,
    );
    s.push(',');
    write_json_u32(
        &mut s,
        "r12b_episode_count_full_w256h4096",
        p.r12b_episode_count_full_w256h4096,
    );
    s.push(',');
    write_json_hash(
        &mut s,
        "digest_lane_plan_hash_v1",
        &p.digest_lane_plan_hash_v1,
    );
    s.push('}');
    s
}

fn write_stage_json(s: &mut String, name: &str, r: &ParsedDigestStageRowV1) {
    let _ = write!(s, "\"{name}\":{{");
    write_json_str(s, "stage_label", r.stage_label);
    s.push(',');
    write_json_u64(s, "us", r.us);
    s.push(',');
    write_json_u32(s, "pct_basis_points", r.pct_basis_points);
    s.push('}');
}

fn write_json_str(s: &mut String, name: &str, value: &str) {
    let _ = write!(s, "\"{name}\":\"");
    for c in value.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            _ => s.push(c),
        }
    }
    s.push('"');
}

fn write_json_u64(s: &mut String, name: &str, v: u64) {
    let _ = write!(s, "\"{name}\":{v}");
}

fn write_json_u32(s: &mut String, name: &str, v: u32) {
    let _ = write!(s, "\"{name}\":{v}");
}

fn write_json_hash(s: &mut String, name: &str, h: &[u8; 32]) {
    let _ = write!(s, "\"{name}\":\"{}\"", hex_str(h));
}

fn write_wrapped_law(s: &mut String, law: &str) {
    let max_col = 72;
    let mut current_len = 0usize;
    let _ = write!(s, "    ");
    for word in law.split_whitespace() {
        let word_len = word.len();
        if current_len > 0 && current_len + 1 + word_len > max_col {
            let _ = writeln!(s);
            let _ = write!(s, "    ");
            current_len = 0;
        }
        if current_len > 0 {
            s.push(' ');
            current_len += 1;
        }
        s.push_str(word);
        current_len += word_len;
    }
    if current_len > 0 {
        let _ = writeln!(s);
    }
}

fn format_bp_to_percent(bp: u32) -> String {
    format!("{:>3}.{:02}", bp / 100, bp % 100)
}

fn hex_str(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}
