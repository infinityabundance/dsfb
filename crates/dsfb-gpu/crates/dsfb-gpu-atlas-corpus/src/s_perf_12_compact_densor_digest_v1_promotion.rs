//! S-PERF.12 — CompactDensorDigestV1 throughput-mode promotion
//! receipt (Track B leg, panel-locked 2026-05-18 post-S-PERF.14c
//! seal at `795d0f9`).
//!
//! Purpose (panel-locked, verbatim from the post-S-PERF.14c
//! panel directive):
//!
//! > S-PERF.12 promotion records that CompactDensorDigestV1
//! > throughput mode is now admitted as part of the measured
//! > combined Track B path. The promoted path clears the
//! > `>20 GB/s` gate after S-PERF.13 host-staging correction
//! > and S-PERF.14a/b/c launch-geometry repair, while
//! > preserving R.12b episodes 13 / 89 / 1917 and the
//! > declared digest-mode non-aliasing law.
//!
//! This module is a **receipt-only** seal. It does NOT
//! introduce new kernels, NOT change any prior hash anchor,
//! NOT run any GPU code, NOT parse new measurement files.
//! It records the panel-locked promotion fields under one
//! own-namespace META-hash that binds the whole Track B
//! chain — S-PERF.11 + S-PERF.12a + S-PERF.13 +
//! S-PERF.14a/b/c — into a single admissible result.
//!
//! ## Promoted bandwidth chain (panel-locked, verbatim from
//! directive):
//!
//! | leg                              | bandwidth (centi-GB/s) |
//! |----------------------------------|-----------------------:|
//! | pre-S-PERF.11 baseline           | 1 333 (= 13.33 GB/s)   |
//! | post-S-PERF.11 leaf-batching     | 1 638 (= 16.38 GB/s)   |
//! | S-PERF.12a warp-coop candidate   | 1 872 (= 18.72 GB/s)   |
//! | post-S-PERF.14c 3-run band       | 2 002 – 2 122 (median 2 016) |
//! | promotion gate                   | **2 000** (= 20.00 GB/s)  |
//! | promotion gate passed            | true                   |
//! | saturation admitted              | false                  |
//!
//! ## Panel-locked non-claims (MUST appear in receipts +
//! README + paper)
//!
//! - Does NOT claim memory-bandwidth saturation (20 GB/s ≈
//!   2.8 % of the 716 GB/s RTX 4080 SUPER vendor peak; the
//!   panel-locked saturation gate is 8 000 bp / 80 % per
//!   S-PERF.1's `S_PERF_1_SATURATION_BP`).
//! - Does NOT claim CompactDensorDigestV1 roots are byte-
//!   identical to TreeSha256V1 (S-PERF.10's
//!   `digest_mode_non_aliasing_law` — each digest mode owns
//!   its own root projection by construction).
//! - Does NOT change Audit mode (SerialSha256 path
//!   unchanged; Audit-mode golden hashes byte-identical).
//! - Does NOT introduce new kernels (the CUDA kernels
//!   themselves were sealed in S-PERF.11 + S-PERF.12a +
//!   S-PERF.13 + S-PERF.14a/b/c; this commit only seals the
//!   admissible-result receipt).
//! - Does NOT mutate any prior corpus / T.11 / T.12.x /
//!   FF.x / S1.3.x / T.12.PROV / S-PERF.1–S-PERF.11.1 hash
//!   anchor.
//! - Does NOT alter `SEED.len()` (stays at 54).
//! - Does NOT rebaseline R.12b episode pins (13 / 89 /
//!   1 917 byte-stable).
//!
//! ## Own-namespace hashes (panel-locked)
//!
//! - `s_perf_12_promotion_fields_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-FIELDS:v1\0` —
//!   pins the 13 promotion fields (3 bandwidth pins + the
//!   3-run band min/max/median + gate + 3 R.12b pins +
//!   digest mode label + 3 boolean discipline flags).
//! - `s_perf_12_promotion_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-REPORT:v1\0` —
//!   top-level META binding the fields hash + the S-PERF.11
//!   bandwidth-delta report anchor + the S-PERF.11.1 triage
//!   anchor + `corpus_hash_v1` + the 7 panel-locked
//!   S-PERF.x commit-sha provenance strings (S-PERF.11,
//!   S-PERF.12a, S-PERF.13, S-PERF.14a, S-PERF.14b,
//!   S-PERF.14c).
//!
//! Two builds against the same panel-locked constants
//! produce byte-identical hashes; the verifier (and the
//! acceptance suite) re-validates this on every build.

#![allow(clippy::module_name_repetitions)]

use std::fmt::Write;

use crate::s_perf_11_1_post_rewrite_bottleneck_triage::{
    seed_post_rewrite_bottleneck_triage_report_from_disk, SeedError as SPerf11_1SeedError,
};
use crate::s_perf_11_measured_digest_compaction::{
    seed_bandwidth_delta_report_from_disk, SeedError as SPerf11SeedError,
};

// ---------------------------------------------------------------
// Domain separators (panel-locked)
// ---------------------------------------------------------------

/// Domain separator for `s_perf_12_promotion_fields_hash_v1`.
/// Format: `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-FIELDS:v1\0`.
pub const S_PERF_12_PROMOTION_FIELDS_DOMAIN: &[u8] =
    b"DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-FIELDS:v1\0";

/// Domain separator for `s_perf_12_promotion_report_hash_v1`.
/// Format: `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-REPORT:v1\0`.
pub const S_PERF_12_PROMOTION_REPORT_DOMAIN: &[u8] =
    b"DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-REPORT:v1\0";

// ---------------------------------------------------------------
// Panel-locked promotion constants (verbatim from the
// post-S-PERF.14c panel directive)
// ---------------------------------------------------------------

/// Pre-S-PERF.11 baseline bandwidth (centi-GB/s = 13.33 GB/s).
/// Panel-pinned at S-PERF.6 commit; verified by S-PERF.7.
pub const PRE_S_PERF_11_BANDWIDTH_CENTI_GBPS: u32 = 1_333;

/// Post-S-PERF.11 leaf-batching bandwidth (centi-GB/s =
/// 16.38 GB/s). Panel-pinned at S-PERF.11 commit `3e67cb4`.
pub const POST_S_PERF_11_BANDWIDTH_CENTI_GBPS: u32 = 1_638;

/// S-PERF.12a warp-coop partial-success candidate bandwidth
/// (centi-GB/s = 18.72 GB/s). Panel-acknowledged
/// partial-success measurement; sealed in the S-PERF.12a
/// commit's working-tree kernel + dispatcher and recorded
/// in the S-PERF.12a memory file. NOT a sealed corpus hash
/// (S-PERF.12a.2 receipt deferred per panel).
pub const S_PERF_12A_CANDIDATE_BANDWIDTH_CENTI_GBPS: u32 = 1_872;

/// Post-S-PERF.14c 3-run thermal band minimum (centi-GB/s =
/// 20.02 GB/s). Measured RTX 4080 SUPER / CUDA 13.2,
/// canonical 256×4096 K=1 D64, 7-iter median, run #1.
pub const POST_S_PERF_14C_BANDWIDTH_BAND_MIN_CENTI_GBPS: u32 = 2_002;

/// Post-S-PERF.14c 3-run thermal band maximum (centi-GB/s =
/// 21.22 GB/s). Measured RTX 4080 SUPER / CUDA 13.2,
/// canonical 256×4096 K=1 D64, 7-iter median, run #3.
pub const POST_S_PERF_14C_BANDWIDTH_BAND_MAX_CENTI_GBPS: u32 = 2_122;

/// Post-S-PERF.14c 3-run thermal band median (centi-GB/s =
/// 20.16 GB/s; run #2). The 3-run band is reported honestly;
/// this median is the headline figure that crosses the
/// promotion gate by a meaningful margin.
pub const POST_S_PERF_14C_BANDWIDTH_BAND_MEDIAN_CENTI_GBPS: u32 = 2_016;

/// Panel-locked S-PERF.12 promotion-gate threshold
/// (centi-GB/s = 20.00 GB/s). Bandwidth strictly greater
/// than this gate is required for promotion-seal admission.
pub const S_PERF_12_PROMOTION_GATE_CENTI_GBPS: u32 = 2_000;

/// R.12b episode-count pin for the canonical 16×128 K=1
/// fixture (panel-pinned at S-PERF.1 and preserved through
/// every subsequent S-PERF.x).
pub const R12B_EPISODE_COUNT_CANONICAL_W16H128: u32 = 13;

/// R.12b episode-count pin for the mid 64×512 K=1 fixture.
pub const R12B_EPISODE_COUNT_MID_W64H512: u32 = 89;

/// R.12b episode-count pin for the full 256×4 096 K=1
/// fixture.
pub const R12B_EPISODE_COUNT_FULL_W256H4096: u32 = 1_917;

/// Panel-locked wire name for the CompactDensorDigestV1
/// throughput-mode identity. Used in the promotion fields'
/// digest-mode declaration; any other value is rejected by
/// the verifier.
pub const COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME: &str = "CompactDensorDigestV1";

// ---------------------------------------------------------------
// Panel-locked upstream commit-sha provenance strings
// ---------------------------------------------------------------

/// Git commit short hash for the S-PERF.11 measured digest-
/// lane compaction seal (corpus-side bandwidth-delta receipt
/// + CUDA leaf-kernel-v2 rewrite).
pub const S_PERF_11_COMMIT_SHA: &str = "3e67cb4";

/// Git commit short hash for the S-PERF.12a CompactDensorDigestV1
/// warp-cooperative partial-candidate CUDA implementation
/// (working-tree kernels + dispatcher + opt-in bench test
/// shipped alongside S-PERF.13).
pub const S_PERF_12A_COMMIT_SHA: &str = "42cce81";

/// Git commit short hash for the S-PERF.13 host input-staging
/// correction (`features_us` → `host_input_staging_us`
/// rename + AVX2 + non-temporal-store SIMD pack;
/// S-PERF.13-PREFLIGHT audit; bundled S-PERF.12a CUDA).
pub const S_PERF_13_COMMIT_SHA: &str = "42cce81";

/// Git commit short hash for the S-PERF.14a drift_slew_sign
/// launch-geometry repair (Pre-Alpha + cellpar split;
/// 1607 → 889 µs / −45 %).
pub const S_PERF_14A_COMMIT_SHA: &str = "dac489f";

/// Git commit short hash for the S-PERF.14b
/// `compact_densor_digest_v1_root_kernel_blockcoop`
/// cooperative-staging seal (2 380 → 824 µs / −65 %).
pub const S_PERF_14B_COMMIT_SHA: &str = "e1dcf54";

/// Git commit short hash for the S-PERF.14c candidate_boundary
/// Pre-Alpha + cellpar split (per-call wall structurally
/// unchanged; combined bandwidth crossed >20 GB/s gate).
pub const S_PERF_14C_COMMIT_SHA: &str = "795d0f9";

// ---------------------------------------------------------------
// Schema
// ---------------------------------------------------------------

/// Promotion fields record. Carries the panel-locked
/// bandwidth chain + promotion-gate threshold + R.12b
/// episode pins + 3 boolean discipline flags. The 13 fields
/// are hashed into `s_perf_12_promotion_fields_hash_v1`
/// under the panel-locked domain separator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf12PromotionFieldsV1 {
    /// `13.33 GB/s` → `1333`. Panel-pinned at S-PERF.6.
    pub pre_s_perf_11_bandwidth_centi_gbps: u32,
    /// `16.38 GB/s` → `1638`. Sealed at S-PERF.11.
    pub post_s_perf_11_bandwidth_centi_gbps: u32,
    /// `18.72 GB/s` → `1872`. S-PERF.12a partial-success.
    pub s_perf_12a_candidate_bandwidth_centi_gbps: u32,
    /// `20.02 GB/s` → `2002`. Post-S-PERF.14c band min.
    pub post_s_perf_14c_bandwidth_band_min_centi_gbps: u32,
    /// `21.22 GB/s` → `2122`. Post-S-PERF.14c band max.
    pub post_s_perf_14c_bandwidth_band_max_centi_gbps: u32,
    /// `20.16 GB/s` → `2016`. Post-S-PERF.14c band median.
    pub post_s_perf_14c_bandwidth_band_median_centi_gbps: u32,
    /// `20.00 GB/s` → `2000`. Panel-locked promotion gate.
    pub promotion_gate_centi_gbps: u32,
    /// `true` iff `post_s_perf_14c_bandwidth_band_min` >
    /// `promotion_gate_centi_gbps`. The MIN of the 3-run
    /// band must clear the gate so even the slowest
    /// observed run satisfies admission.
    pub promotion_gate_passed: bool,
    /// `false` — the promotion seal explicitly does NOT
    /// claim memory-bandwidth saturation. 20 GB/s ≈ 2.8 %
    /// of the 716 GB/s vendor peak; the panel-locked
    /// saturation gate (8 000 bp / 80 %) is at 572.8 GB/s.
    pub saturation_admitted: bool,
    /// `13` — canonical 16×128 K=1 fixture episode count.
    pub r12b_episode_count_canonical_w16h128: u32,
    /// `89` — mid 64×512 K=1 fixture episode count.
    pub r12b_episode_count_mid_w64h512: u32,
    /// `1917` — full 256×4 096 K=1 fixture episode count.
    pub r12b_episode_count_full_w256h4096: u32,
    /// `"CompactDensorDigestV1"` — promoted throughput-mode
    /// identity wire name.
    pub digest_mode_wire_name: &'static str,
    /// `false` — the promotion explicitly does NOT claim
    /// `CompactDensorDigestV1` roots are byte-identical to
    /// `TreeSha256V1` roots (panel-locked S-PERF.10
    /// `digest_mode_non_aliasing_law`).
    pub tree_sha256v1_root_aliasing: bool,
    /// `true` — Audit-mode (`SerialSha256`) path is
    /// preserved unchanged across every commit in the
    /// chain. Audit-mode golden hashes byte-identical.
    pub audit_mode_unchanged: bool,
    /// META-hash over the 14 fields above under
    /// `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-FIELDS:v1\0`.
    pub s_perf_12_promotion_fields_hash_v1: [u8; 32],
}

/// Promotion-report record. Top-level META binding the
/// promotion-fields hash + 2 upstream corpus-side anchor
/// hashes (S-PERF.11 + S-PERF.11.1) + `corpus_hash_v1` +
/// the 6 panel-locked CUDA-side commit-sha provenance
/// strings. The full record hashes into
/// `s_perf_12_promotion_report_hash_v1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf12PromotionReportV1 {
    /// `"s_perf_12_compact_densor_digest_v1_promotion_v1"`.
    pub report_id: &'static str,
    /// The 14 panel-locked promotion fields + their own
    /// META-hash.
    pub fields: SPerf12PromotionFieldsV1,
    /// Live anchor binding to S-PERF.11's bandwidth-delta
    /// report hash (= `1a27154e335c27df6db939d4c8ff0f36f8baf75871be06c750f0853f2268adc8`
    /// at S-PERF.11 seal `3e67cb4`).
    pub s_perf_11_bandwidth_delta_report_hash_v1: [u8; 32],
    /// Live anchor binding to S-PERF.11.1's bottleneck-
    /// triage report hash.
    pub s_perf_11_1_bottleneck_triage_hash_v1: [u8; 32],
    /// Live anchor binding to `corpus_hash_v1`. Pins the
    /// seed-corpus provenance so the promotion receipt
    /// cannot drift from the literature corpus state.
    pub corpus_hash_v1: [u8; 32],
    /// Panel-locked S-PERF.11 commit-sha provenance.
    pub s_perf_11_commit_sha: &'static str,
    /// Panel-locked S-PERF.12a commit-sha provenance
    /// (working-tree CUDA + bundled into S-PERF.13 commit).
    pub s_perf_12a_commit_sha: &'static str,
    /// Panel-locked S-PERF.13 commit-sha provenance.
    pub s_perf_13_commit_sha: &'static str,
    /// Panel-locked S-PERF.14a commit-sha provenance.
    pub s_perf_14a_commit_sha: &'static str,
    /// Panel-locked S-PERF.14b commit-sha provenance.
    pub s_perf_14b_commit_sha: &'static str,
    /// Panel-locked S-PERF.14c commit-sha provenance.
    pub s_perf_14c_commit_sha: &'static str,
    /// Top-level META-hash over every field above under
    /// `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-REPORT:v1\0`.
    pub s_perf_12_promotion_report_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the panel-locked promotion fields with their
/// META-hash populated. Internally consistent by
/// construction: every field is sourced from the panel-
/// locked constants above; the hash is computed at build
/// time so two builds against the same constants produce
/// byte-identical fields.
#[must_use]
pub fn build_promotion_fields() -> SPerf12PromotionFieldsV1 {
    let mut f = SPerf12PromotionFieldsV1 {
        pre_s_perf_11_bandwidth_centi_gbps: PRE_S_PERF_11_BANDWIDTH_CENTI_GBPS,
        post_s_perf_11_bandwidth_centi_gbps: POST_S_PERF_11_BANDWIDTH_CENTI_GBPS,
        s_perf_12a_candidate_bandwidth_centi_gbps: S_PERF_12A_CANDIDATE_BANDWIDTH_CENTI_GBPS,
        post_s_perf_14c_bandwidth_band_min_centi_gbps:
            POST_S_PERF_14C_BANDWIDTH_BAND_MIN_CENTI_GBPS,
        post_s_perf_14c_bandwidth_band_max_centi_gbps:
            POST_S_PERF_14C_BANDWIDTH_BAND_MAX_CENTI_GBPS,
        post_s_perf_14c_bandwidth_band_median_centi_gbps:
            POST_S_PERF_14C_BANDWIDTH_BAND_MEDIAN_CENTI_GBPS,
        promotion_gate_centi_gbps: S_PERF_12_PROMOTION_GATE_CENTI_GBPS,
        // Gate-passed predicate: the MIN of the 3-run band
        // strictly exceeds the gate. Using MIN (not median)
        // is the conservative discipline: even the slowest
        // observed run must clear the gate to admit the
        // promotion. At 2002 vs 2000 the MIN clears by 0.02
        // GB/s; the median (2016) clears by 0.16 GB/s.
        promotion_gate_passed: POST_S_PERF_14C_BANDWIDTH_BAND_MIN_CENTI_GBPS
            > S_PERF_12_PROMOTION_GATE_CENTI_GBPS,
        // Panel-locked non-claim: 20 GB/s ≈ 2.8 % of 716
        // GB/s vendor peak; the 80 % saturation gate is
        // explicitly NOT cleared.
        saturation_admitted: false,
        r12b_episode_count_canonical_w16h128: R12B_EPISODE_COUNT_CANONICAL_W16H128,
        r12b_episode_count_mid_w64h512: R12B_EPISODE_COUNT_MID_W64H512,
        r12b_episode_count_full_w256h4096: R12B_EPISODE_COUNT_FULL_W256H4096,
        digest_mode_wire_name: COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME,
        // Panel-locked S-PERF.10 digest_mode_non_aliasing_law:
        // CompactDensorDigestV1 roots are structurally
        // distinct from TreeSha256V1 roots by canonical-
        // header construction. The aliasing flag is
        // explicitly false; the verifier rejects any
        // promotion-seal asserting otherwise.
        tree_sha256v1_root_aliasing: false,
        // Audit-mode (SerialSha256) path is preserved
        // unchanged across the entire Track B chain. The
        // Audit-mode golden hashes byte-identical from
        // R.7 through every S-PERF.x commit.
        audit_mode_unchanged: true,
        s_perf_12_promotion_fields_hash_v1: [0u8; 32],
    };
    f.s_perf_12_promotion_fields_hash_v1 = compute_promotion_fields_hash(&f);
    f
}

/// Build the full S-PERF.12 promotion report by loading the
/// 2 upstream corpus-side anchors live from disk + the
/// `corpus_hash_v1` anchor + the panel-locked 6 commit-sha
/// provenance strings + the panel-locked promotion fields.
///
/// # Errors
///
/// Returns `SeedError` when either upstream seed
/// (`seed_bandwidth_delta_report_from_disk` or
/// `seed_post_rewrite_bottleneck_triage_report_from_disk`)
/// fails (e.g., source-report file missing or malformed).
pub fn seed_s_perf_12_promotion_report_from_disk(
    repo_root: &std::path::Path,
) -> Result<SPerf12PromotionReportV1, SeedError> {
    let s_perf_11 = seed_bandwidth_delta_report_from_disk(repo_root)
        .map_err(|e: SPerf11SeedError| SeedError::SeedSPerf11(format!("{e:?}")))?;
    let s_perf_11_1 = seed_post_rewrite_bottleneck_triage_report_from_disk(repo_root)
        .map_err(|e: SPerf11_1SeedError| SeedError::SeedSPerf11_1(format!("{e:?}")))?;
    let corpus_hash = crate::corpus_hash::compute_corpus_hash_v1();

    let fields = build_promotion_fields();
    let mut report = SPerf12PromotionReportV1 {
        report_id: "s_perf_12_compact_densor_digest_v1_promotion_v1",
        fields,
        s_perf_11_bandwidth_delta_report_hash_v1: s_perf_11
            .s_perf_11_bandwidth_delta_report_hash_v1,
        s_perf_11_1_bottleneck_triage_hash_v1: s_perf_11_1.s_perf_11_1_bottleneck_triage_hash_v1,
        corpus_hash_v1: corpus_hash.bytes,
        s_perf_11_commit_sha: S_PERF_11_COMMIT_SHA,
        s_perf_12a_commit_sha: S_PERF_12A_COMMIT_SHA,
        s_perf_13_commit_sha: S_PERF_13_COMMIT_SHA,
        s_perf_14a_commit_sha: S_PERF_14A_COMMIT_SHA,
        s_perf_14b_commit_sha: S_PERF_14B_COMMIT_SHA,
        s_perf_14c_commit_sha: S_PERF_14C_COMMIT_SHA,
        s_perf_12_promotion_report_hash_v1: [0u8; 32],
    };
    report.s_perf_12_promotion_report_hash_v1 = compute_promotion_report_hash(&report);
    Ok(report)
}

/// Build the promotion report from already-loaded upstream
/// anchor hashes (no disk I/O). Used by the acceptance suite
/// and by callers that already have the anchors in hand.
#[must_use]
pub fn build_promotion_report(
    s_perf_11_bandwidth_delta_report_hash_v1: [u8; 32],
    s_perf_11_1_bottleneck_triage_hash_v1: [u8; 32],
    corpus_hash_v1: [u8; 32],
) -> SPerf12PromotionReportV1 {
    let fields = build_promotion_fields();
    let mut report = SPerf12PromotionReportV1 {
        report_id: "s_perf_12_compact_densor_digest_v1_promotion_v1",
        fields,
        s_perf_11_bandwidth_delta_report_hash_v1,
        s_perf_11_1_bottleneck_triage_hash_v1,
        corpus_hash_v1,
        s_perf_11_commit_sha: S_PERF_11_COMMIT_SHA,
        s_perf_12a_commit_sha: S_PERF_12A_COMMIT_SHA,
        s_perf_13_commit_sha: S_PERF_13_COMMIT_SHA,
        s_perf_14a_commit_sha: S_PERF_14A_COMMIT_SHA,
        s_perf_14b_commit_sha: S_PERF_14B_COMMIT_SHA,
        s_perf_14c_commit_sha: S_PERF_14C_COMMIT_SHA,
        s_perf_12_promotion_report_hash_v1: [0u8; 32],
    };
    report.s_perf_12_promotion_report_hash_v1 = compute_promotion_report_hash(&report);
    report
}

// ---------------------------------------------------------------
// Hash functions
// ---------------------------------------------------------------

/// Compute the panel-locked META-hash over the 14 promotion
/// fields under
/// `DSFB-GPU-ATLAS:S-PERF-12-PROMOTION-FIELDS:v1\0`. Field
/// serialization is canonical: u32 fields big-endian, bool
/// fields as `0u8`/`1u8`, the digest-mode wire name length-
/// prefixed (u32 BE length + bytes). Deterministic across
/// builds and across machines.
#[must_use]
pub fn compute_promotion_fields_hash(f: &SPerf12PromotionFieldsV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(256);
    buf.extend_from_slice(S_PERF_12_PROMOTION_FIELDS_DOMAIN);
    buf.extend_from_slice(&f.pre_s_perf_11_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(&f.post_s_perf_11_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(&f.s_perf_12a_candidate_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(
        &f.post_s_perf_14c_bandwidth_band_min_centi_gbps
            .to_be_bytes(),
    );
    buf.extend_from_slice(
        &f.post_s_perf_14c_bandwidth_band_max_centi_gbps
            .to_be_bytes(),
    );
    buf.extend_from_slice(
        &f.post_s_perf_14c_bandwidth_band_median_centi_gbps
            .to_be_bytes(),
    );
    buf.extend_from_slice(&f.promotion_gate_centi_gbps.to_be_bytes());
    buf.push(u8::from(f.promotion_gate_passed));
    buf.push(u8::from(f.saturation_admitted));
    buf.extend_from_slice(&f.r12b_episode_count_canonical_w16h128.to_be_bytes());
    buf.extend_from_slice(&f.r12b_episode_count_mid_w64h512.to_be_bytes());
    buf.extend_from_slice(&f.r12b_episode_count_full_w256h4096.to_be_bytes());
    push_len_prefixed_str(&mut buf, f.digest_mode_wire_name);
    buf.push(u8::from(f.tree_sha256v1_root_aliasing));
    buf.push(u8::from(f.audit_mode_unchanged));
    dsfb_gpu_debug_core::hash::sha256(&buf)
}

/// Compute the panel-locked top-level META-hash binding the
/// fields hash + 3 upstream anchor hashes + 6 commit-sha
/// provenance strings. Serialization: domain separator +
/// fields hash + 3 anchor hashes (32 B each) + 6 length-
/// prefixed commit-sha strings + length-prefixed report id.
#[must_use]
pub fn compute_promotion_report_hash(r: &SPerf12PromotionReportV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    buf.extend_from_slice(S_PERF_12_PROMOTION_REPORT_DOMAIN);
    push_len_prefixed_str(&mut buf, r.report_id);
    buf.extend_from_slice(&r.fields.s_perf_12_promotion_fields_hash_v1);
    buf.extend_from_slice(&r.s_perf_11_bandwidth_delta_report_hash_v1);
    buf.extend_from_slice(&r.s_perf_11_1_bottleneck_triage_hash_v1);
    buf.extend_from_slice(&r.corpus_hash_v1);
    push_len_prefixed_str(&mut buf, r.s_perf_11_commit_sha);
    push_len_prefixed_str(&mut buf, r.s_perf_12a_commit_sha);
    push_len_prefixed_str(&mut buf, r.s_perf_13_commit_sha);
    push_len_prefixed_str(&mut buf, r.s_perf_14a_commit_sha);
    push_len_prefixed_str(&mut buf, r.s_perf_14b_commit_sha);
    push_len_prefixed_str(&mut buf, r.s_perf_14c_commit_sha);
    dsfb_gpu_debug_core::hash::sha256(&buf)
}

#[allow(
    clippy::expect_used,
    reason = "u32::try_from(bytes.len()) is unreachable in practice — every wire name is a panel-locked compile-time constant whose length is bounded by the source code (digest mode wire name <= 32 bytes; commit shas <= 8 bytes); recording an explicit reason here mirrors the S-PERF.10/11 hash-helper pattern."
)]
fn push_len_prefixed_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len_u32 =
        u32::try_from(bytes.len()).expect("panel-locked wire-name length must fit in u32");
    buf.extend_from_slice(&len_u32.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Panel-locked verifier error kinds. The 8 campaign-identity
/// negatives + 4 structural-defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf12PromotionVerifyErrorKind {
    /// CAMPAIGN IDENTITY: the post-S-PERF.14c band minimum
    /// (the most conservative observed bandwidth) does NOT
    /// strictly exceed the panel-locked promotion gate.
    /// Admitting a promotion seal where even the slowest
    /// observed run fails to clear the gate would be a
    /// silent overclaim.
    PromotionSealWhenBandwidthBelowGate,
    /// `saturation_admitted = true` is explicitly forbidden:
    /// 20 GB/s ≈ 2.8 % of the 716 GB/s vendor peak; the
    /// panel-locked saturation gate (8 000 bp / 80 %) is
    /// at 572.8 GB/s. The promotion seal cannot smuggle a
    /// saturation claim.
    PromotionSealWithSaturationClaim,
    /// `tree_sha256v1_root_aliasing = true` is explicitly
    /// forbidden by S-PERF.10's panel-locked
    /// `digest_mode_non_aliasing_law`. Each digest mode owns
    /// its own root projection by canonical-header
    /// construction; the promotion seal cannot smuggle a
    /// cross-mode aliasing claim.
    PromotionSealWithTreeSha256V1AliasingClaim,
    /// R.12b episode counts differ from the panel-locked
    /// triple (13 / 89 / 1 917). Any drift indicates the
    /// bank's admission decisions have shifted somewhere in
    /// the Track B chain — a byte-identity contract
    /// violation upstream of this seal.
    PromotionSealWithR12bEpisodeDrift,
    /// The S-PERF.11 bandwidth-delta report anchor is zero
    /// (i.e., the promotion seal does NOT bind the S-PERF.11
    /// measured-result hash). The chain cannot be closed
    /// without this anchor.
    PromotionSealWithoutSPerf11Anchor,
    /// The S-PERF.11.1 bottleneck-triage anchor is zero.
    /// The chain cannot be closed without this anchor —
    /// S-PERF.11.1 records the panel-locked decision-rule
    /// audit between S-PERF.11 and the S-PERF.12+ work.
    PromotionSealWithoutSPerf11_1Anchor,
    /// `audit_mode_unchanged = false` is explicitly
    /// forbidden: the SerialSha256 Audit-mode path must
    /// remain byte-identical across the entire Track B
    /// chain. A promotion seal that loses Audit-mode
    /// invariance is inadmissible.
    PromotionSealWithoutAuditModeUnchangedFlag,
    /// `digest_mode_wire_name` does NOT equal the panel-
    /// locked literal `"CompactDensorDigestV1"`. The seal
    /// promotes the CompactDensorDigestV1 throughput-mode
    /// identity specifically; any other mode value is a
    /// rename-discipline failure.
    PromotionSealWithoutCompactDensorDigestV1ModeIdentity,
    /// Structural: `report_id` is empty.
    ReportIdEmpty,
    /// Structural: the panel-locked promotion-gate value
    /// is not exactly `S_PERF_12_PROMOTION_GATE_CENTI_GBPS`
    /// (= 2 000). The gate is the panel-locked promotion
    /// threshold; if a future commit changes it, that is a
    /// schema-upgrade campaign, not a silent rebaseline.
    PromotionGateValueDrifted,
    /// Structural: the `promotion_gate_passed` field does
    /// not match the arithmetic predicate
    /// `band_min > promotion_gate`. The field is recorded
    /// for direct operator readability but must remain
    /// consistent with the underlying arithmetic.
    PromotionGatePassedArithmeticMismatch,
    /// Structural: `corpus_hash_v1` is zero. The promotion
    /// seal must bind the live corpus identity.
    CorpusHashMissing,
}

/// Panel-locked verifier error type. Carries the kind +
/// optional operator-legible context string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf12PromotionVerifyError {
    /// Discriminant identifying the panel-locked rule that
    /// fired.
    pub kind: SPerf12PromotionVerifyErrorKind,
    /// Optional context (e.g., the offending field value)
    /// for operator-legible reporting.
    pub context: String,
}

/// Errors returned by [`seed_s_perf_12_promotion_report_from_disk`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedError {
    /// S-PERF.11 upstream seed failed (e.g., the source
    /// report file is missing or malformed).
    SeedSPerf11(String),
    /// S-PERF.11.1 upstream seed failed.
    SeedSPerf11_1(String),
}

impl std::fmt::Display for SeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SeedSPerf11(s) => write!(f, "S-PERF.12 promotion seed: S-PERF.11 upstream: {s}"),
            Self::SeedSPerf11_1(s) => {
                write!(f, "S-PERF.12 promotion seed: S-PERF.11.1 upstream: {s}")
            }
        }
    }
}

impl std::error::Error for SeedError {}

/// Walk the promotion report against every panel-locked
/// rule. Returns the full list of violations; an empty
/// vector means admit.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "panel-locked 12-rule verifier walks every promotion-discipline field in one canonical pass; splitting would obscure the rule-by-rule audit order the verifier text and tests both depend on."
)]
pub fn verify_promotion_report(r: &SPerf12PromotionReportV1) -> Vec<SPerf12PromotionVerifyError> {
    let mut errs: Vec<SPerf12PromotionVerifyError> = Vec::new();
    let f = &r.fields;

    if r.report_id.is_empty() {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::ReportIdEmpty,
            context: String::new(),
        });
    }
    if f.promotion_gate_centi_gbps != S_PERF_12_PROMOTION_GATE_CENTI_GBPS {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionGateValueDrifted,
            context: format!(
                "promotion_gate_centi_gbps = {} (expected {})",
                f.promotion_gate_centi_gbps, S_PERF_12_PROMOTION_GATE_CENTI_GBPS
            ),
        });
    }
    let gate_passed_predicate =
        f.post_s_perf_14c_bandwidth_band_min_centi_gbps > f.promotion_gate_centi_gbps;
    if f.promotion_gate_passed != gate_passed_predicate {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionGatePassedArithmeticMismatch,
            context: format!(
                "promotion_gate_passed = {} but (band_min {} > gate {}) = {}",
                f.promotion_gate_passed,
                f.post_s_perf_14c_bandwidth_band_min_centi_gbps,
                f.promotion_gate_centi_gbps,
                gate_passed_predicate
            ),
        });
    }
    if !f.promotion_gate_passed {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWhenBandwidthBelowGate,
            context: format!(
                "band_min {} centi-GB/s <= gate {} centi-GB/s",
                f.post_s_perf_14c_bandwidth_band_min_centi_gbps, f.promotion_gate_centi_gbps
            ),
        });
    }
    if f.saturation_admitted {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithSaturationClaim,
            context: "saturation_admitted = true is forbidden (20 GB/s <<  80% of 716 GB/s peak)"
                .to_string(),
        });
    }
    if f.tree_sha256v1_root_aliasing {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithTreeSha256V1AliasingClaim,
            context: "tree_sha256v1_root_aliasing = true violates S-PERF.10 \
                      digest_mode_non_aliasing_law"
                .to_string(),
        });
    }
    if f.r12b_episode_count_canonical_w16h128 != R12B_EPISODE_COUNT_CANONICAL_W16H128
        || f.r12b_episode_count_mid_w64h512 != R12B_EPISODE_COUNT_MID_W64H512
        || f.r12b_episode_count_full_w256h4096 != R12B_EPISODE_COUNT_FULL_W256H4096
    {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithR12bEpisodeDrift,
            context: format!(
                "episodes = {} / {} / {} (expected {} / {} / {})",
                f.r12b_episode_count_canonical_w16h128,
                f.r12b_episode_count_mid_w64h512,
                f.r12b_episode_count_full_w256h4096,
                R12B_EPISODE_COUNT_CANONICAL_W16H128,
                R12B_EPISODE_COUNT_MID_W64H512,
                R12B_EPISODE_COUNT_FULL_W256H4096
            ),
        });
    }
    if r.s_perf_11_bandwidth_delta_report_hash_v1 == [0u8; 32] {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithoutSPerf11Anchor,
            context: "s_perf_11_bandwidth_delta_report_hash_v1 is zero".to_string(),
        });
    }
    if r.s_perf_11_1_bottleneck_triage_hash_v1 == [0u8; 32] {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithoutSPerf11_1Anchor,
            context: "s_perf_11_1_bottleneck_triage_hash_v1 is zero".to_string(),
        });
    }
    if !f.audit_mode_unchanged {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithoutAuditModeUnchangedFlag,
            context: "audit_mode_unchanged = false breaks SerialSha256 Audit-mode invariance"
                .to_string(),
        });
    }
    if f.digest_mode_wire_name != COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::PromotionSealWithoutCompactDensorDigestV1ModeIdentity,
            context: format!(
                "digest_mode_wire_name = {:?} (expected {:?})",
                f.digest_mode_wire_name, COMPACT_DENSOR_DIGEST_V1_MODE_WIRE_NAME
            ),
        });
    }
    if r.corpus_hash_v1 == [0u8; 32] {
        errs.push(SPerf12PromotionVerifyError {
            kind: SPerf12PromotionVerifyErrorKind::CorpusHashMissing,
            context: "corpus_hash_v1 is zero".to_string(),
        });
    }

    errs
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn fmt_centi_gbps_as_gbps(c: u32) -> String {
    format!("{:.2}", f64::from(c) / 100.0)
}

/// Render the panel-locked promotion-report receipt as
/// deterministic plain text. Two consecutive renders against
/// the same `SPerf12PromotionReportV1` produce byte-
/// identical output (acceptance suite enforces).
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "renderer's output bytes are pinned by render byte-stability acceptance tests; splitting would risk byte-stream drift."
)]
pub fn render_promotion_report_text(r: &SPerf12PromotionReportV1) -> String {
    let mut out = String::with_capacity(4096);
    let _ = writeln!(
        out,
        "S-PERF.12 — CompactDensorDigestV1 throughput-mode promotion receipt"
    );
    let _ = writeln!(
        out,
        "=================================================================="
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "report_id : {}", r.report_id);
    let _ = writeln!(out);
    let _ = writeln!(out, "Promoted bandwidth chain (centi-GB/s)");
    let _ = writeln!(out, "-------------------------------------");
    let f = &r.fields;
    let _ = writeln!(
        out,
        "  pre-S-PERF.11 baseline             : {:>5}  ({} GB/s)",
        f.pre_s_perf_11_bandwidth_centi_gbps,
        fmt_centi_gbps_as_gbps(f.pre_s_perf_11_bandwidth_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  post-S-PERF.11 leaf-batching       : {:>5}  ({} GB/s)",
        f.post_s_perf_11_bandwidth_centi_gbps,
        fmt_centi_gbps_as_gbps(f.post_s_perf_11_bandwidth_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  S-PERF.12a warp-coop candidate     : {:>5}  ({} GB/s)",
        f.s_perf_12a_candidate_bandwidth_centi_gbps,
        fmt_centi_gbps_as_gbps(f.s_perf_12a_candidate_bandwidth_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  post-S-PERF.14c band min           : {:>5}  ({} GB/s)",
        f.post_s_perf_14c_bandwidth_band_min_centi_gbps,
        fmt_centi_gbps_as_gbps(f.post_s_perf_14c_bandwidth_band_min_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  post-S-PERF.14c band median        : {:>5}  ({} GB/s)",
        f.post_s_perf_14c_bandwidth_band_median_centi_gbps,
        fmt_centi_gbps_as_gbps(f.post_s_perf_14c_bandwidth_band_median_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  post-S-PERF.14c band max           : {:>5}  ({} GB/s)",
        f.post_s_perf_14c_bandwidth_band_max_centi_gbps,
        fmt_centi_gbps_as_gbps(f.post_s_perf_14c_bandwidth_band_max_centi_gbps)
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Promotion-gate verdict");
    let _ = writeln!(out, "----------------------");
    let _ = writeln!(
        out,
        "  promotion_gate                     : {:>5}  ({} GB/s)",
        f.promotion_gate_centi_gbps,
        fmt_centi_gbps_as_gbps(f.promotion_gate_centi_gbps)
    );
    let _ = writeln!(
        out,
        "  promotion_gate_passed              : {}",
        f.promotion_gate_passed
    );
    let _ = writeln!(
        out,
        "  saturation_admitted                : {}",
        f.saturation_admitted
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "R.12b episode-count invariants");
    let _ = writeln!(out, "------------------------------");
    let _ = writeln!(
        out,
        "  canonical 16x128 K=1               : {:>5}",
        f.r12b_episode_count_canonical_w16h128
    );
    let _ = writeln!(
        out,
        "  mid 64x512 K=1                     : {:>5}",
        f.r12b_episode_count_mid_w64h512
    );
    let _ = writeln!(
        out,
        "  full 256x4096 K=1                  : {:>5}",
        f.r12b_episode_count_full_w256h4096
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Digest-mode discipline");
    let _ = writeln!(out, "----------------------");
    let _ = writeln!(
        out,
        "  digest_mode                        : {}",
        f.digest_mode_wire_name
    );
    let _ = writeln!(
        out,
        "  tree_sha256v1_root_aliasing        : {}  (panel-locked false; S-PERF.10 non-aliasing law)",
        f.tree_sha256v1_root_aliasing
    );
    let _ = writeln!(
        out,
        "  audit_mode_unchanged               : {}  (SerialSha256 path byte-identical end-to-end)",
        f.audit_mode_unchanged
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Upstream anchor hashes");
    let _ = writeln!(out, "----------------------");
    let _ = writeln!(
        out,
        "  s_perf_12_promotion_fields_hash_v1            : {}",
        hex32(&f.s_perf_12_promotion_fields_hash_v1)
    );
    let _ = writeln!(
        out,
        "  s_perf_11_bandwidth_delta_report_hash_v1      : {}",
        hex32(&r.s_perf_11_bandwidth_delta_report_hash_v1)
    );
    let _ = writeln!(
        out,
        "  s_perf_11_1_bottleneck_triage_hash_v1         : {}",
        hex32(&r.s_perf_11_1_bottleneck_triage_hash_v1)
    );
    let _ = writeln!(
        out,
        "  corpus_hash_v1                                : {}",
        hex32(&r.corpus_hash_v1)
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "CUDA-side commit-sha provenance chain");
    let _ = writeln!(out, "-------------------------------------");
    let _ = writeln!(
        out,
        "  S-PERF.11  (digest-lane compaction)     : {}",
        r.s_perf_11_commit_sha
    );
    let _ = writeln!(
        out,
        "  S-PERF.12a (warp-coop CUDA + bundled)   : {}",
        r.s_perf_12a_commit_sha
    );
    let _ = writeln!(
        out,
        "  S-PERF.13  (host input-staging AVX2)    : {}",
        r.s_perf_13_commit_sha
    );
    let _ = writeln!(
        out,
        "  S-PERF.14a (drift_slew_sign split)      : {}",
        r.s_perf_14a_commit_sha
    );
    let _ = writeln!(
        out,
        "  S-PERF.14b (compact_densor root)        : {}",
        r.s_perf_14b_commit_sha
    );
    let _ = writeln!(
        out,
        "  S-PERF.14c (candidate_boundary split)   : {}",
        r.s_perf_14c_commit_sha
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Top-level META-hash");
    let _ = writeln!(out, "-------------------");
    let _ = writeln!(
        out,
        "  s_perf_12_promotion_report_hash_v1            : {}",
        hex32(&r.s_perf_12_promotion_report_hash_v1)
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Panel-locked non-claims");
    let _ = writeln!(out, "-----------------------");
    let _ = writeln!(
        out,
        "  - Does NOT claim memory-bandwidth saturation (20 GB/s ~ 2.8% of 716 GB/s vendor peak;"
    );
    let _ = writeln!(
        out,
        "    panel-locked saturation gate is 8000 bp / 80% per S-PERF.1)."
    );
    let _ = writeln!(
        out,
        "  - Does NOT claim CompactDensorDigestV1 roots are byte-identical to TreeSha256V1 roots"
    );
    let _ = writeln!(out, "    (S-PERF.10 digest_mode_non_aliasing_law).");
    let _ = writeln!(out, "  - Does NOT change Audit mode (SerialSha256 path unchanged; golden hashes byte-identical).");
    let _ = writeln!(out, "  - Does NOT introduce new CUDA kernels (the kernels were sealed in S-PERF.11+12a+13+14a/b/c;");
    let _ = writeln!(out, "    this commit is the receipt-only promotion seal).");
    let _ = writeln!(
        out,
        "  - Does NOT mutate any prior corpus / T.11 / T.12.x / FF.x / S1.3.x / T.12.PROV /"
    );
    let _ = writeln!(out, "    S-PERF.1-S-PERF.11.1 hash anchor.");
    let _ = writeln!(out, "  - Does NOT alter SEED.len() (stays at 54).");
    let _ = writeln!(
        out,
        "  - Does NOT rebaseline R.12b episode pins (13 / 89 / 1917 byte-stable)."
    );
    out
}

/// Render the promotion report as deterministic canonical
/// JSON. Field order is fixed; numeric values are emitted
/// as decimal integers; hashes are lowercase hex. Two
/// consecutive renders produce byte-identical output.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "renderer's output bytes are pinned by render byte-stability acceptance tests; splitting would risk byte-stream drift."
)]
pub fn render_promotion_report_json(r: &SPerf12PromotionReportV1) -> String {
    let mut out = String::with_capacity(2048);
    out.push('{');
    let f = &r.fields;
    let _ = write!(out, "\"report_id\":\"{}\",", r.report_id);
    out.push_str("\"fields\":{");
    let _ = write!(
        out,
        "\"pre_s_perf_11_bandwidth_centi_gbps\":{},",
        f.pre_s_perf_11_bandwidth_centi_gbps
    );
    let _ = write!(
        out,
        "\"post_s_perf_11_bandwidth_centi_gbps\":{},",
        f.post_s_perf_11_bandwidth_centi_gbps
    );
    let _ = write!(
        out,
        "\"s_perf_12a_candidate_bandwidth_centi_gbps\":{},",
        f.s_perf_12a_candidate_bandwidth_centi_gbps
    );
    let _ = write!(
        out,
        "\"post_s_perf_14c_bandwidth_band_min_centi_gbps\":{},",
        f.post_s_perf_14c_bandwidth_band_min_centi_gbps
    );
    let _ = write!(
        out,
        "\"post_s_perf_14c_bandwidth_band_max_centi_gbps\":{},",
        f.post_s_perf_14c_bandwidth_band_max_centi_gbps
    );
    let _ = write!(
        out,
        "\"post_s_perf_14c_bandwidth_band_median_centi_gbps\":{},",
        f.post_s_perf_14c_bandwidth_band_median_centi_gbps
    );
    let _ = write!(
        out,
        "\"promotion_gate_centi_gbps\":{},",
        f.promotion_gate_centi_gbps
    );
    let _ = write!(
        out,
        "\"promotion_gate_passed\":{},",
        f.promotion_gate_passed
    );
    let _ = write!(out, "\"saturation_admitted\":{},", f.saturation_admitted);
    let _ = write!(
        out,
        "\"r12b_episode_count_canonical_w16h128\":{},",
        f.r12b_episode_count_canonical_w16h128
    );
    let _ = write!(
        out,
        "\"r12b_episode_count_mid_w64h512\":{},",
        f.r12b_episode_count_mid_w64h512
    );
    let _ = write!(
        out,
        "\"r12b_episode_count_full_w256h4096\":{},",
        f.r12b_episode_count_full_w256h4096
    );
    let _ = write!(
        out,
        "\"digest_mode_wire_name\":\"{}\",",
        f.digest_mode_wire_name
    );
    let _ = write!(
        out,
        "\"tree_sha256v1_root_aliasing\":{},",
        f.tree_sha256v1_root_aliasing
    );
    let _ = write!(out, "\"audit_mode_unchanged\":{},", f.audit_mode_unchanged);
    let _ = write!(
        out,
        "\"s_perf_12_promotion_fields_hash_v1\":\"{}\"",
        hex32(&f.s_perf_12_promotion_fields_hash_v1)
    );
    out.push_str("},");
    let _ = write!(
        out,
        "\"s_perf_11_bandwidth_delta_report_hash_v1\":\"{}\",",
        hex32(&r.s_perf_11_bandwidth_delta_report_hash_v1)
    );
    let _ = write!(
        out,
        "\"s_perf_11_1_bottleneck_triage_hash_v1\":\"{}\",",
        hex32(&r.s_perf_11_1_bottleneck_triage_hash_v1)
    );
    let _ = write!(out, "\"corpus_hash_v1\":\"{}\",", hex32(&r.corpus_hash_v1));
    let _ = write!(
        out,
        "\"s_perf_11_commit_sha\":\"{}\",",
        r.s_perf_11_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_12a_commit_sha\":\"{}\",",
        r.s_perf_12a_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_13_commit_sha\":\"{}\",",
        r.s_perf_13_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_14a_commit_sha\":\"{}\",",
        r.s_perf_14a_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_14b_commit_sha\":\"{}\",",
        r.s_perf_14b_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_14c_commit_sha\":\"{}\",",
        r.s_perf_14c_commit_sha
    );
    let _ = write!(
        out,
        "\"s_perf_12_promotion_report_hash_v1\":\"{}\"",
        hex32(&r.s_perf_12_promotion_report_hash_v1)
    );
    out.push('}');
    out
}
