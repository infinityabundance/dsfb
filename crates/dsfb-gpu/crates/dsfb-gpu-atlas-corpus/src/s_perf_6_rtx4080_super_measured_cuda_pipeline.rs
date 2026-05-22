//! S-PERF.6 --- RTX 4080 SUPER measured CUDA pipeline
//! baseline.
//!
//! ## Commit identity
//!
//! > **S-PERF.6 records the measured RTX 4080 SUPER CUDA
//! > pipeline result as a real bandwidth receipt: 13.33
//! > GB/s, 1.86% of the 716 GB/s vendor-datasheet peak,
//! > not a saturation claim. The measured values are
//! > sourced verbatim from
//! > `reports/d64_stage_timing_256x4096_K1.txt`.**
//!
//! Core rule (panel-locked):
//!
//! > Measure first. Claim second. Claim exactly what was
//! > measured, and no more.
//!
//! ## What this records
//!
//! `reports/d64_stage_timing_256x4096_K1.txt` records
//! wide bytes/sec = 13.33 GB/s at the 256x4096 K=1 D64
//! throughput profile (median of 3 iterations,
//! post-warmup). The receipt below encodes that measured
//! result into the corpus court honestly:
//!
//! - admissible measured CUDA pipeline bandwidth result;
//! - not a saturation claim
//!   (13.33 GB/s = 186 bp = 1.86 % of 716 GB/s peak,
//!   well below the 8000 bp saturation threshold);
//! - not a B300 / GB300 result;
//! - not production CUDA performance;
//! - not Layer-A purity --- the measured pipeline includes
//!   host-side `compute_features` and bank-admit + finalize
//!   segments outside what S-PERF.2 defines as Layer-A;
//!   both segments are honestly disclosed in the
//!   `host_compute_features_us` and
//!   `host_bank_admit_case_finalize_us` fields.
//!
//! ## Rounding law (panel-pinned: FLOOR)
//!
//! ```text
//! percent_of_peak_basis_points
//!   = measured_wide_bandwidth_centi_gbps * 10000
//!     / (theoretical_memory_bandwidth_gbps * 100)
//!   = 1333 * 10000 / (716 * 100)
//!   = 13_330_000 / 71_600
//!   = 186.17...
//!   -> floor 107   (integer division)
//! ```
//!
//! Bandwidth encoded as a `u32` in centi-GB/s so 13.33 is
//! representable as an integer (770). The theoretical
//! peak stays in integer GB/s (716) to match the
//! panel-locked S-PERF.1 anchor.
//!
//! ## Hash posture
//!
//! Three own-namespace hashes (none folded upstream):
//!
//! - `rtx4080_super_measured_cuda_pipeline_hash_v1` under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-CUDA-PIPELINE:v1\0`.
//!   Pins device identity, theoretical peak, every
//!   measured stage timing, host segments, measured
//!   bandwidth, and source_report_path.
//! - `rtx4080_super_measured_bandwidth_claim_hash_v1`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BANDWIDTH-CLAIM:v1\0`.
//!   Pins claim_kind, admissibility_reason, admitted,
//!   saturation_admitted,
//!   saturation_threshold_basis_points, and
//!   observed_percent_of_peak_basis_points.
//! - `rtx4080_super_measured_baseline_report_hash_v1`
//!   under
//!   `DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BASELINE-REPORT:v1\0`.
//!   Top-level META-hash binding the measurement, claim,
//!   four upstream anchor hashes (S-PERF.2, S-PERF.3,
//!   S-PERF.4, S-PERF.5), and three R.12b episode-count
//!   pins.
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.6 does NOT:
//!
//! - claim memory-bandwidth saturation (186 bp is far
//!   below the 8000 bp threshold);
//! - claim B300 or GB300 performance;
//! - claim production CUDA performance;
//! - claim Layer-A purity (host segments are honestly
//!   disclosed);
//! - generate new detector results;
//! - change any CUDA kernel;
//! - change any court decision;
//! - mutate any upstream hash anchor;
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs or episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate;
//! - download or fetch any dataset bytes;
//! - rebaseline the R.12b D64 saturation pinned
//!   baseline;
//! - run the benchmark itself. The corpus crate is
//!   panel-locked host-only with zero CUDA dependency;
//!   the measurement was captured by
//!   `dsfb-gpu-debug-cuda`'s existing bench harness and
//!   recorded into
//!   `reports/d64_stage_timing_256x4096_K1.txt`. S-PERF.6
//!   encodes that captured result into the corpus court.
//!
//! ## One-line verdict
//!
//! > S-PERF.6 measures 13.33 GB/s on the RTX 4080 SUPER.
//! > That is 1.86% of the 716 GB/s vendor-datasheet peak,
//! > not saturation.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::s_perf_1_device_traffic_receipt::{
    compute_device_identity_hash, S_PERF_1_SATURATION_BP,
};
use crate::s_perf_2_layer_a_resident_pipeline::seed_baseline_layer_a_traffic_receipt;
use crate::s_perf_3_public_data_saturation_bundle::seed_baseline_public_data_saturation_bundle;
use crate::s_perf_4_active_family_compaction::seed_baseline_family_compaction_benchmark_schema;
use crate::s_perf_5_effective_bandwidth_report::seed_baseline_effective_bandwidth_report;

// ---------------------------------------------------------------
// Panel-locked RTX 4080 SUPER device identity constants
// ---------------------------------------------------------------

/// Canonical device name pinned by the panel. The S-PERF.6
/// baseline rejects any measurement that declares a
/// different device name; cross-device claims must build
/// their own baseline module rather than re-purpose this
/// one.
pub const RTX_4080_SUPER_DEVICE_NAME: &str = "RTX 4080 SUPER";

/// Streaming-multiprocessor architecture for the RTX 4080
/// SUPER (sm_89, Ada Lovelace). Pinned by the panel because
/// it participates in `compute_device_identity_hash`; any
/// rename of `sm_arch` would mutate the device identity
/// hash and is forbidden.
pub const RTX_4080_SUPER_SM_ARCH: u32 = 89;

/// Vendor-datasheet theoretical memory bandwidth for the
/// RTX 4080 SUPER. Pinned by the panel; saturation claims
/// must use this value as the denominator for
/// percent-of-peak.
pub const RTX_4080_SUPER_THEORETICAL_PEAK_GBPS: u32 = 716;

// ---------------------------------------------------------------
// Panel-locked R.12b episode-count integrity pins
// ---------------------------------------------------------------

/// Panel-locked R.12b episode count at the canonical
/// `16 entities x 128 windows` grid. The pinned
/// `tests/r12_d64_saturation` regression check reports
/// `episodes/cat = 13` for this grid; the S-PERF.6
/// measured baseline MUST declare exactly this value. Any
/// future commit that alters R.12b's episode count to a
/// different value would constitute a rebaseline and is
/// forbidden by panel-required negative #10.
pub const R12B_EPISODE_COUNT_CANONICAL_W16H128: u32 = 13;

/// Panel-locked R.12b episode count at the mid
/// `64 entities x 512 windows` grid. See
/// [`R12B_EPISODE_COUNT_CANONICAL_W16H128`] for the
/// rebaseline-forbidden rationale.
pub const R12B_EPISODE_COUNT_MID_W64H512: u32 = 89;

/// Panel-locked R.12b episode count at the full
/// `256 entities x 4096 windows` grid. See
/// [`R12B_EPISODE_COUNT_CANONICAL_W16H128`] for the
/// rebaseline-forbidden rationale.
pub const R12B_EPISODE_COUNT_FULL_W256H4096: u32 = 1917;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for
/// `rtx4080_super_measured_cuda_pipeline_hash_v1`.
pub const S_PERF_6_MEASURED_CUDA_PIPELINE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-CUDA-PIPELINE:v1\0";

/// Schema identifier for
/// `rtx4080_super_measured_cuda_pipeline_hash_v1`.
pub const S_PERF_6_MEASURED_CUDA_PIPELINE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-CUDA-PIPELINE:v1";

/// Domain separator for
/// `rtx4080_super_measured_bandwidth_claim_hash_v1`.
pub const S_PERF_6_MEASURED_BANDWIDTH_CLAIM_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BANDWIDTH-CLAIM:v1\0";

/// Schema identifier for
/// `rtx4080_super_measured_bandwidth_claim_hash_v1`.
pub const S_PERF_6_MEASURED_BANDWIDTH_CLAIM_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BANDWIDTH-CLAIM:v1";

/// Domain separator for
/// `rtx4080_super_measured_baseline_report_hash_v1`.
pub const S_PERF_6_MEASURED_BASELINE_REPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BASELINE-REPORT:v1\0";

/// Schema identifier for
/// `rtx4080_super_measured_baseline_report_hash_v1`.
pub const S_PERF_6_MEASURED_BASELINE_REPORT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S-PERF-6-MEASURED-BASELINE-REPORT:v1";

// ---------------------------------------------------------------
// Panel-pinned measured values (verbatim from source receipt)
// ---------------------------------------------------------------

/// Source receipt path. The S-PERF.6 baseline cites this
/// file as the provenance of the measured values; the
/// verifier rejects any baseline that omits the path
/// (panel-required negative #3).
pub const S_PERF_6_SOURCE_REPORT_PATH: &str = "reports/d64_stage_timing_256x4096_K1.txt";

/// CUDA toolkit version under which the measurement was
/// captured (vendor identity for cross-machine
/// reproducibility).
pub const S_PERF_6_CUDA_VERSION: &str = "13.2";

/// Host wall-clock median, in microseconds (includes host
/// segments). Sourced verbatim from the live bench output
/// in `reports/d64_stage_timing_256x4096_K1.txt` (median
/// of 3 iters, post-warmup, RTX 4080 SUPER + CUDA 13.2).
pub const S_PERF_6_HOST_WALL_MEDIAN_US: u64 = 30_020;

/// Device-side total time (sum of every GPU stage), in
/// microseconds.
pub const S_PERF_6_DEVICE_TOTAL_US: u64 = 20_771;

/// `consensus_grid_kernel_wide` stage time, in microseconds.
pub const S_PERF_6_CONSENSUS_GRID_KERNEL_WIDE_US: u64 = 382;

/// `tree_digest consensus` stage time, in microseconds.
/// This stage is the dominant device-side cost
/// (20.88 % of device total).
pub const S_PERF_6_TREE_DIGEST_CONSENSUS_US: u64 = 4_338;

/// `tree_digest consensus` stage share of device total,
/// expressed in basis points (10000 = 100.00 %). 20.88 %
/// = 2088 bp.
pub const S_PERF_6_TREE_DIGEST_CONSENSUS_PERCENT_BP: u32 = 2_088;

/// Host-side `compute_features` segment time, in
/// microseconds. Honestly disclosed (the measured pipeline
/// is NOT pure Layer-A; window features run on the host
/// before H2D per the existing dsfb-gpu-debug v0 design).
pub const S_PERF_6_HOST_COMPUTE_FEATURES_US: u64 = 7_525;

/// Host-side `bank admit + case finalize` segment time, in
/// microseconds. Honestly disclosed (semantic admission is
/// CPU-only per the Semantic Non-Bypass Axiom).
pub const S_PERF_6_HOST_BANK_ADMIT_CASE_FINALIZE_US: u64 = 2_237;

/// Measured wide bytes/sec, in centi-GB/s (1333 = 13.33
/// GB/s). Encoded in centi-units so 13.33 is representable
/// as an integer.
pub const S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS: u32 = 1_333;

/// Pre-computed percent-of-peak basis-points value for the
/// panel-pinned measurement: 1333 * 10000 / (716 * 100) =
/// 186 (integer floor). The test
/// `percent_of_peak_computes_from_13_33_and_716` re-derives
/// this constant from the source values.
pub const S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS: u32 = 186;

// ---------------------------------------------------------------
// Forbidden claim-substring scanner (mirrors S-PERF.5 / 6
// pattern, extended with B300 / GB300 / production-performance
// forbidden phrases per S-PERF.6 directive)
// ---------------------------------------------------------------

/// Case-insensitive substring scanner over free-text fields
/// (`admissibility_reason` and `source_report_path`).
/// Extends the S-PERF.5 / S-PERF.6 set with the explicit
/// S-PERF.6 prohibitions: B300, GB300, production
/// performance. The scanner fires panel-required negatives
/// #7 / #8 / #9.
const S_PERF_6_FORBIDDEN_CLAIM_SUBSTRINGS: &[&str] = &[
    "achieves saturation",
    "saturates the bandwidth",
    "saturates peak",
    "% of peak",
    "percent of peak",
    "outperforms",
    "beats the baseline",
    "world record",
    "fastest gpu",
    "production-ready performance",
    "production cuda performance",
    "production performance",
    "petaflops",
    "memory-bandwidth saturation",
    "b300",
    "gb300",
];

// ---------------------------------------------------------------
// MeasuredCudaPipelineClaimKind (local enum; NOT touching
// S-PERF.5's BandwidthClaimKind)
// ---------------------------------------------------------------

/// What kind of measured-CUDA-pipeline claim the baseline
/// makes. Defined locally in the S-PERF.6 module so
/// adding a new variant does not touch the S-PERF.5
/// `BandwidthClaimKind` enum (preserving all prior S-PERF.5
/// hash values byte-identical).
///
/// Wire names are stable for the hash buffer; do not rename
/// without rebaselining
/// `rtx4080_super_measured_bandwidth_claim_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasuredCudaPipelineClaimKind {
    /// Admissible measured CUDA pipeline bandwidth result.
    /// The pipeline includes host segments (honestly
    /// disclosed); the claim is bandwidth, not saturation.
    MeasuredCudaPipelineBandwidth,
}

impl MeasuredCudaPipelineClaimKind {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MeasuredCudaPipelineBandwidth => "MeasuredCudaPipelineBandwidth",
        }
    }
}

// ---------------------------------------------------------------
// Panel-locked admissibility reason wire name
// ---------------------------------------------------------------

/// Admissibility reason for an admissible measured CUDA
/// pipeline bandwidth result on the RTX 4080 SUPER. The
/// wire name explicitly disclaims saturation, B300/GB300,
/// production performance, and Layer-A purity.
pub const ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE: &str =
    "AdmissibleMeasuredCudaPipelineBandwidthOnRtx4080SuperNotSaturationNotProductionNotPureLayerA";

// ---------------------------------------------------------------
// Rtx4080SuperMeasuredCudaPipelineV1
// ---------------------------------------------------------------

/// The raw measured CUDA pipeline record pinned to the RTX
/// 4080 SUPER device identity. Carries every measured stage
/// timing + host segments + measured wide bandwidth +
/// source-report provenance.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `rtx4080_super_measured_cuda_pipeline_hash_v1`.
#[derive(Debug, Clone)]
pub struct Rtx4080SuperMeasuredCudaPipelineV1 {
    /// Device name. MUST equal `RTX_4080_SUPER_DEVICE_NAME`
    /// (panel-required negative #4 part 1).
    pub device_name: &'static str,
    /// Streaming-multiprocessor architecture. MUST equal
    /// `RTX_4080_SUPER_SM_ARCH = 89` (panel-required
    /// negative #4 part 2).
    pub sm_arch: u32,
    /// Device identity hash. MUST equal
    /// `compute_device_identity_hash(...)` for the
    /// panel-locked RTX 4080 SUPER device (panel-required
    /// negative #4 part 3).
    pub device_uuid_or_identity_hash: [u8; 32],
    /// CUDA toolkit version under which the measurement
    /// was captured.
    pub cuda_version: &'static str,
    /// Theoretical peak memory bandwidth in GB/s. MUST
    /// equal `RTX_4080_SUPER_THEORETICAL_PEAK_GBPS = 716`.
    pub theoretical_memory_bandwidth_gbps: u32,

    /// Host wall-clock median, in microseconds (includes
    /// host segments). Provenance reference.
    pub host_wall_median_us: u64,
    /// Device-side total time (sum of all GPU stages), in
    /// microseconds. MUST be non-zero (panel-required
    /// negative #2).
    pub device_total_us: u64,
    /// `consensus_grid_kernel_wide` stage time, in
    /// microseconds.
    pub consensus_grid_kernel_wide_us: u64,
    /// `tree_digest consensus` stage time, in microseconds.
    /// MUST be non-zero (panel-required negative #11).
    pub tree_digest_consensus_us: u64,
    /// `tree_digest consensus` stage share of device total,
    /// expressed in basis points.
    pub tree_digest_consensus_percent_basis_points: u32,
    /// Host-side `compute_features` segment time, in
    /// microseconds. MUST be non-zero (panel-required
    /// negative #12 part 1) -- this is the honest
    /// disclosure that the measured pipeline includes
    /// host time outside Layer-A.
    pub host_compute_features_us: u64,
    /// Host-side `bank admit + case finalize` segment
    /// time, in microseconds. MUST be non-zero (panel-
    /// required negative #12 part 2).
    pub host_bank_admit_case_finalize_us: u64,

    /// Measured wide bytes/sec, in centi-GB/s. MUST be
    /// non-zero (panel-required negative #1). 770 = 13.33
    /// GB/s.
    pub measured_wide_bandwidth_centi_gbps: u32,
    /// Percent-of-peak in basis points (10000 = 100.00 %).
    /// MUST equal
    /// `measured_wide_bandwidth_centi_gbps * 10000 /
    /// (theoretical_memory_bandwidth_gbps * 100)` using
    /// integer floor (panel-required negative #5).
    pub percent_of_peak_basis_points: u32,

    /// Path to the measured-source report (the provenance
    /// of every measured value above). MUST be non-empty
    /// (panel-required negative #3).
    pub source_report_path: &'static str,

    /// `rtx4080_super_measured_cuda_pipeline_hash_v1`.
    pub rtx4080_super_measured_cuda_pipeline_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Rtx4080SuperMeasuredBandwidthClaimV1
// ---------------------------------------------------------------

/// The bandwidth-claim verdict for the RTX 4080 SUPER
/// measured CUDA pipeline result. Carries the claim kind +
/// panel-locked admissibility reason wire name + admitted
/// boolean + saturation discipline fields
/// (`saturation_threshold_basis_points`,
/// `saturation_admitted`,
/// `observed_percent_of_peak_basis_points`).
#[derive(Debug, Clone)]
pub struct Rtx4080SuperMeasuredBandwidthClaimV1 {
    /// What kind of measured-CUDA-pipeline claim is being
    /// made. MUST be `MeasuredCudaPipelineBandwidth`
    /// (panel-required negative #14: the baseline is not
    /// allowed to be NoClaim for a measured result).
    pub claim_kind: MeasuredCudaPipelineClaimKind,
    /// Panel-locked admissibility-reason wire name. MUST
    /// be non-empty (panel-required negative #13).
    pub admissibility_reason_wire_name: &'static str,
    /// `true` if the measured result is admissible.
    pub admitted: bool,
    /// Panel-locked S-PERF.1 saturation threshold in basis
    /// points. MUST equal 8000.
    pub saturation_threshold_basis_points: u32,
    /// The observed percent-of-peak in basis points
    /// (mirrors the measurement field).
    pub observed_percent_of_peak_basis_points: u32,
    /// `true` if the result reaches the saturation
    /// threshold. MUST be `false` for the panel-pinned
    /// 107-bp baseline (panel-required negatives #6 / #7).
    pub saturation_admitted: bool,
    /// `rtx4080_super_measured_bandwidth_claim_hash_v1`.
    pub rtx4080_super_measured_bandwidth_claim_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Rtx4080SuperMeasuredBaselineReportV1
// ---------------------------------------------------------------

/// The top-level S-PERF.6 baseline report. Binds the
/// measurement + claim + five upstream anchor hashes
/// (S-PERF.2 / S-PERF.3 / S-PERF.4 / S-PERF.5 / S-PERF.6) +
/// three R.12b episode-count integrity pins.
#[derive(Debug, Clone)]
pub struct Rtx4080SuperMeasuredBaselineReportV1 {
    /// Human-readable baseline identifier (non-empty).
    pub baseline_id: &'static str,
    /// Wrapped measured CUDA pipeline record.
    pub measurement: Rtx4080SuperMeasuredCudaPipelineV1,
    /// Wrapped bandwidth-claim verdict.
    pub claim: Rtx4080SuperMeasuredBandwidthClaimV1,
    /// S-PERF.2 Layer-A traffic receipt hash (cross-corpus
    /// integrity binding).
    pub s_perf_2_layer_a_traffic_receipt_hash: [u8; 32],
    /// S-PERF.3 public-data bundle hash.
    pub s_perf_3_public_data_bundle_hash: [u8; 32],
    /// S-PERF.4 family-compaction benchmark schema hash.
    pub s_perf_4_family_compaction_benchmark_schema_hash: [u8; 32],
    /// S-PERF.5 effective-bandwidth report hash.
    pub s_perf_5_effective_bandwidth_report_hash: [u8; 32],
    /// Panel-locked R.12b episode count at the canonical
    /// `16x128` grid. MUST equal 13 (panel-required
    /// negative #10 part 1).
    pub r12b_episode_count_canonical_w16h128: u32,
    /// Panel-locked R.12b episode count at the mid
    /// `64x512` grid. MUST equal 89 (panel-required
    /// negative #10 part 2).
    pub r12b_episode_count_mid_w64h512: u32,
    /// Panel-locked R.12b episode count at the full
    /// `256x4096` grid. MUST equal 1917 (panel-required
    /// negative #10 part 3).
    pub r12b_episode_count_full_w256h4096: u32,
    /// `rtx4080_super_measured_baseline_report_hash_v1`.
    pub rtx4080_super_measured_baseline_report_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.6 rejected a baseline. Fourteen panel-
/// required load-bearing negatives plus structural defect
/// rules (the directive lists exactly 14 negatives;
/// structural defects share variants).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf6VerifyErrorKind {
    /// Panel-required negative #1.
    /// `measured_wide_bandwidth_centi_gbps == 0`.
    ZeroMeasuredBandwidth,
    /// Panel-required negative #2. `device_total_us == 0`.
    ZeroDeviceTotalTime,
    /// Panel-required negative #3. `source_report_path`
    /// is empty.
    MissingSourceReportPath,
    /// Panel-required negative #4. The measurement's
    /// declared RTX 4080 SUPER identity (device_name,
    /// sm_arch, identity hash, or theoretical peak) does
    /// not match the panel-locked constants.
    MissingRtx4080SuperIdentity {
        /// Which device-identity field disagreed.
        which_field_wire_name: &'static str,
    },
    /// Panel-required negative #5. The measurement's
    /// `percent_of_peak_basis_points` does not equal the
    /// value computed from
    /// `measured_wide_bandwidth_centi_gbps` and
    /// `theoretical_memory_bandwidth_gbps` using the
    /// panel-locked floor arithmetic.
    PercentOfPeakArithmeticMismatch {
        /// What the measurement claimed.
        claimed: u32,
        /// What the verifier's floor arithmetic computed.
        computed: u32,
    },
    /// Panel-required negative #6. The claim declares
    /// `saturation_admitted == true` but the observed
    /// percent-of-peak is below
    /// [`S_PERF_1_SATURATION_BP`] (8000 bp).
    SaturationClaimBelow8000Bp {
        /// Observed basis-points value.
        observed_bp: u32,
    },
    /// Panel-required negative #7. The claim asserts that
    /// 13.33 GB/s reaches saturation (e.g.
    /// `observed_percent_of_peak_basis_points` reports a
    /// value at or above 8000 bp but the measured
    /// bandwidth corresponds to centi-GB/s far below
    /// what 8000 bp of 716 GB/s would require). Also
    /// fires if the admissibility-reason string contains
    /// "saturation" / "saturates" forbidden phrases.
    ClaimThat7_70GbpsIsSaturation,
    /// Panel-required negative #8. The admissibility-
    /// reason or source-report-path contains "b300" /
    /// "gb300" forbidden substrings (case-insensitive).
    ClaimThatResultIsB300OrGb300,
    /// Panel-required negative #9. The admissibility-
    /// reason or source-report-path contains
    /// "production performance" / "production-ready" /
    /// similar forbidden substrings.
    ClaimThatResultIsProductionPerformance,
    /// Panel-required negative #10. One of the three R.12b
    /// episode-count pins
    /// (`r12b_episode_count_canonical_w16h128`,
    /// `r12b_episode_count_mid_w64h512`,
    /// `r12b_episode_count_full_w256h4096`) does not match
    /// the panel-locked values (13 / 89 / 1917). Any
    /// rebaselining of the R.12b pinned saturation
    /// baseline surfaces here.
    RebaselineOfR12bEpisodeCounts {
        /// Which pin disagreed.
        which_pin_wire_name: &'static str,
        /// What the baseline declared.
        declared: u32,
        /// What the panel pinned.
        panel_locked: u32,
    },
    /// Panel-required negative #11. The measurement's
    /// `tree_digest_consensus_us` is zero. The
    /// dominant-stage timing MUST be present for any
    /// measured CUDA pipeline result.
    MissingTreeDigestStageTiming,
    /// Panel-required negative #12. The measurement's
    /// host segment timings (`host_compute_features_us`
    /// AND `host_bank_admit_case_finalize_us`) are both
    /// zero. At least one MUST be disclosed (these
    /// segments are the honest record that the measured
    /// pipeline is NOT pure Layer-A).
    MissingHostSegmentDisclosure,
    /// Panel-required negative #13. The claim's
    /// `admissibility_reason_wire_name` is empty.
    EmptyClaimKind,
    /// Panel-required negative #14. The claim is somehow
    /// not the measured-CUDA-pipeline-bandwidth variant
    /// (reserved for future-claim-kind extensions; today
    /// the enum has only one variant so this is a
    /// schema-defence guard).
    NoClaimBaselineForMeasuredResult,
    /// Structural defect: free-text field contains a
    /// forbidden substring not covered by negatives #7 /
    /// #8 / #9 specifically (catch-all for the broader
    /// 16-substring scanner).
    ForbiddenSubstringInsideReport {
        /// Schema field where the violation appeared.
        location: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Structural defect: `baseline_id` is empty.
    BaselineIdEmpty,
    /// Structural defect: saturation threshold field does
    /// not equal panel-locked 8000.
    SaturationThresholdMismatch {
        /// What the claim declared.
        declared: u32,
    },
    /// Structural defect: `observed_percent_of_peak_basis_points`
    /// in the claim does not equal the measurement's
    /// `percent_of_peak_basis_points`.
    ObservedPercentOfPeakMismatch {
        /// What the claim declared.
        claim_observed: u32,
        /// What the measurement recorded.
        measurement_observed: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf6VerifyError {
    /// Error kind (see [`SPerf6VerifyErrorKind`]).
    pub kind: SPerf6VerifyErrorKind,
}

// ---------------------------------------------------------------
// Arithmetic helper (panel-locked FLOOR rounding)
// ---------------------------------------------------------------

/// Compute percent-of-peak in basis points using the
/// panel-locked FLOOR rounding law:
///
/// ```text
/// percent_of_peak_basis_points
///   = measured_centi_gbps * 10000 / (theoretical_gbps * 100)
/// ```
///
/// For the panel-pinned (770, 716) inputs this returns 107.
/// Returns 0 when the theoretical peak is zero (guards
/// division-by-zero).
#[must_use]
pub fn compute_s_perf_6_percent_of_peak_basis_points(
    measured_centi_gbps: u32,
    theoretical_gbps: u32,
) -> u32 {
    if theoretical_gbps == 0 {
        return 0;
    }
    let denom: u64 = u64::from(theoretical_gbps) * 100;
    let numerator: u64 = u64::from(measured_centi_gbps) * 10_000;
    u32::try_from(numerator / denom).unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build an [`Rtx4080SuperMeasuredCudaPipelineV1`] and
/// populate
/// `rtx4080_super_measured_cuda_pipeline_hash_v1`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_rtx4080_super_measured_cuda_pipeline(
    device_name: &'static str,
    sm_arch: u32,
    device_uuid_or_identity_hash: [u8; 32],
    cuda_version: &'static str,
    theoretical_memory_bandwidth_gbps: u32,
    host_wall_median_us: u64,
    device_total_us: u64,
    consensus_grid_kernel_wide_us: u64,
    tree_digest_consensus_us: u64,
    tree_digest_consensus_percent_basis_points: u32,
    host_compute_features_us: u64,
    host_bank_admit_case_finalize_us: u64,
    measured_wide_bandwidth_centi_gbps: u32,
    percent_of_peak_basis_points: u32,
    source_report_path: &'static str,
) -> Rtx4080SuperMeasuredCudaPipelineV1 {
    let mut p = Rtx4080SuperMeasuredCudaPipelineV1 {
        device_name,
        sm_arch,
        device_uuid_or_identity_hash,
        cuda_version,
        theoretical_memory_bandwidth_gbps,
        host_wall_median_us,
        device_total_us,
        consensus_grid_kernel_wide_us,
        tree_digest_consensus_us,
        tree_digest_consensus_percent_basis_points,
        host_compute_features_us,
        host_bank_admit_case_finalize_us,
        measured_wide_bandwidth_centi_gbps,
        percent_of_peak_basis_points,
        source_report_path,
        rtx4080_super_measured_cuda_pipeline_hash_v1: [0u8; 32],
    };
    p.rtx4080_super_measured_cuda_pipeline_hash_v1 =
        compute_rtx4080_super_measured_cuda_pipeline_hash(&p);
    p
}

/// Build an [`Rtx4080SuperMeasuredBandwidthClaimV1`] and
/// populate
/// `rtx4080_super_measured_bandwidth_claim_hash_v1`.
#[must_use]
pub fn build_rtx4080_super_measured_bandwidth_claim(
    claim_kind: MeasuredCudaPipelineClaimKind,
    admissibility_reason_wire_name: &'static str,
    admitted: bool,
    saturation_threshold_basis_points: u32,
    observed_percent_of_peak_basis_points: u32,
    saturation_admitted: bool,
) -> Rtx4080SuperMeasuredBandwidthClaimV1 {
    let mut c = Rtx4080SuperMeasuredBandwidthClaimV1 {
        claim_kind,
        admissibility_reason_wire_name,
        admitted,
        saturation_threshold_basis_points,
        observed_percent_of_peak_basis_points,
        saturation_admitted,
        rtx4080_super_measured_bandwidth_claim_hash_v1: [0u8; 32],
    };
    c.rtx4080_super_measured_bandwidth_claim_hash_v1 =
        compute_rtx4080_super_measured_bandwidth_claim_hash(&c);
    c
}

/// Build an [`Rtx4080SuperMeasuredBaselineReportV1`] and
/// populate
/// `rtx4080_super_measured_baseline_report_hash_v1`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_rtx4080_super_measured_baseline_report(
    baseline_id: &'static str,
    measurement: Rtx4080SuperMeasuredCudaPipelineV1,
    claim: Rtx4080SuperMeasuredBandwidthClaimV1,
    s_perf_2_layer_a_traffic_receipt_hash: [u8; 32],
    s_perf_3_public_data_bundle_hash: [u8; 32],
    s_perf_4_family_compaction_benchmark_schema_hash: [u8; 32],
    s_perf_5_effective_bandwidth_report_hash: [u8; 32],
    r12b_episode_count_canonical_w16h128: u32,
    r12b_episode_count_mid_w64h512: u32,
    r12b_episode_count_full_w256h4096: u32,
) -> Rtx4080SuperMeasuredBaselineReportV1 {
    let mut r = Rtx4080SuperMeasuredBaselineReportV1 {
        baseline_id,
        measurement,
        claim,
        s_perf_2_layer_a_traffic_receipt_hash,
        s_perf_3_public_data_bundle_hash,
        s_perf_4_family_compaction_benchmark_schema_hash,
        s_perf_5_effective_bandwidth_report_hash,
        r12b_episode_count_canonical_w16h128,
        r12b_episode_count_mid_w64h512,
        r12b_episode_count_full_w256h4096,
        rtx4080_super_measured_baseline_report_hash_v1: [0u8; 32],
    };
    r.rtx4080_super_measured_baseline_report_hash_v1 =
        compute_rtx4080_super_measured_baseline_report_hash(&r);
    r
}

// ---------------------------------------------------------------
// Seed (panel-pinned measured baseline)
// ---------------------------------------------------------------

/// Build the panel-pinned measured CUDA pipeline record for
/// the RTX 4080 SUPER, sourced verbatim from
/// `reports/d64_stage_timing_256x4096_K1.txt`.
#[must_use]
pub fn seed_rtx4080_super_measured_cuda_pipeline() -> Rtx4080SuperMeasuredCudaPipelineV1 {
    let device_identity =
        compute_device_identity_hash(RTX_4080_SUPER_DEVICE_NAME, RTX_4080_SUPER_SM_ARCH);
    build_rtx4080_super_measured_cuda_pipeline(
        RTX_4080_SUPER_DEVICE_NAME,
        RTX_4080_SUPER_SM_ARCH,
        device_identity,
        S_PERF_6_CUDA_VERSION,
        RTX_4080_SUPER_THEORETICAL_PEAK_GBPS,
        S_PERF_6_HOST_WALL_MEDIAN_US,
        S_PERF_6_DEVICE_TOTAL_US,
        S_PERF_6_CONSENSUS_GRID_KERNEL_WIDE_US,
        S_PERF_6_TREE_DIGEST_CONSENSUS_US,
        S_PERF_6_TREE_DIGEST_CONSENSUS_PERCENT_BP,
        S_PERF_6_HOST_COMPUTE_FEATURES_US,
        S_PERF_6_HOST_BANK_ADMIT_CASE_FINALIZE_US,
        S_PERF_6_MEASURED_WIDE_BANDWIDTH_CENTI_GBPS,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        S_PERF_6_SOURCE_REPORT_PATH,
    )
}

/// Build the panel-pinned bandwidth-claim verdict for the
/// measured RTX 4080 SUPER CUDA pipeline result.
/// Admissible (admitted=true) but NOT saturating
/// (saturation_admitted=false, observed 186 bp << 8000 bp
/// threshold).
#[must_use]
pub fn seed_rtx4080_super_measured_bandwidth_claim() -> Rtx4080SuperMeasuredBandwidthClaimV1 {
    build_rtx4080_super_measured_bandwidth_claim(
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth,
        ADMISSIBILITY_REASON_RTX4080_SUPER_MEASURED_CUDA_PIPELINE,
        true,
        S_PERF_1_SATURATION_BP,
        S_PERF_6_PERCENT_OF_PEAK_BASIS_POINTS,
        false,
    )
}

/// Build the panel-pinned baseline report. Composes the
/// measured pipeline record + the admissible bandwidth-
/// claim verdict + the live S-PERF.2 / S-PERF.3 / S-PERF.4
/// / S-PERF.5 upstream anchor hashes + the three R.12b
/// episode-count integrity pins. The chain cannot drift
/// from production court state.
#[must_use]
pub fn seed_rtx4080_super_measured_baseline_report() -> Rtx4080SuperMeasuredBaselineReportV1 {
    let measurement = seed_rtx4080_super_measured_cuda_pipeline();
    let claim = seed_rtx4080_super_measured_bandwidth_claim();
    let layer_a = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    let report_5 = seed_baseline_effective_bandwidth_report();
    build_rtx4080_super_measured_baseline_report(
        "s_perf_6_baseline_rtx4080_super_measured_cuda_pipeline_v1",
        measurement,
        claim,
        layer_a.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
        report_5.effective_bandwidth_report_hash_v1,
        R12B_EPISODE_COUNT_CANONICAL_W16H128,
        R12B_EPISODE_COUNT_MID_W64H512,
        R12B_EPISODE_COUNT_FULL_W256H4096,
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

/// WHY: serialises every measured field of the pipeline
/// record into a canonical byte buffer and SHA-256s the
/// result so two builds against the same measured-source
/// receipt produce byte-identical hashes. The domain
/// separator + schema id are the first bytes so the hash
/// cannot collide with a different schema accidentally
/// using the same field layout. Field order is locked: any
/// reordering rebases the hash and breaks
/// `PINNED_MEASURED_CUDA_PIPELINE_HASH_V1` in the test
/// suite, which is the explicit panel-decision gate
/// against silent rebaselining.
fn compute_rtx4080_super_measured_cuda_pipeline_hash(
    p: &Rtx4080SuperMeasuredCudaPipelineV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_6_MEASURED_CUDA_PIPELINE_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S_PERF_6_MEASURED_CUDA_PIPELINE_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, p.device_name.as_bytes());
    buf.extend_from_slice(&p.sm_arch.to_be_bytes());
    buf.extend_from_slice(&p.device_uuid_or_identity_hash);
    push_len_prefixed(&mut buf, p.cuda_version.as_bytes());
    buf.extend_from_slice(&p.theoretical_memory_bandwidth_gbps.to_be_bytes());
    buf.extend_from_slice(&p.host_wall_median_us.to_be_bytes());
    buf.extend_from_slice(&p.device_total_us.to_be_bytes());
    buf.extend_from_slice(&p.consensus_grid_kernel_wide_us.to_be_bytes());
    buf.extend_from_slice(&p.tree_digest_consensus_us.to_be_bytes());
    buf.extend_from_slice(&p.tree_digest_consensus_percent_basis_points.to_be_bytes());
    buf.extend_from_slice(&p.host_compute_features_us.to_be_bytes());
    buf.extend_from_slice(&p.host_bank_admit_case_finalize_us.to_be_bytes());
    buf.extend_from_slice(&p.measured_wide_bandwidth_centi_gbps.to_be_bytes());
    buf.extend_from_slice(&p.percent_of_peak_basis_points.to_be_bytes());
    push_len_prefixed(&mut buf, p.source_report_path.as_bytes());
    sha256(&buf)
}

/// WHY: pins the verdict bytes (claim_kind wire name,
/// admissibility reason wire name, admitted boolean,
/// saturation threshold, observed percent-of-peak,
/// saturation_admitted boolean) into a single hash. The
/// admissibility reason wire name is length-prefixed so a
/// future longer / shorter reason cannot collide with the
/// current 89-character panel-locked string. The
/// `saturation_threshold_basis_points` and
/// `observed_percent_of_peak_basis_points` fields are
/// included even though they appear redundant with the
/// measurement: they bind the verdict to the specific
/// threshold + observed value the verdict was issued
/// against, so a future schema change that mutated either
/// would surface as a hash change.
fn compute_rtx4080_super_measured_bandwidth_claim_hash(
    c: &Rtx4080SuperMeasuredBandwidthClaimV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_6_MEASURED_BANDWIDTH_CLAIM_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S_PERF_6_MEASURED_BANDWIDTH_CLAIM_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, c.claim_kind.as_str().as_bytes());
    push_len_prefixed(&mut buf, c.admissibility_reason_wire_name.as_bytes());
    buf.push(u8::from(c.admitted));
    buf.extend_from_slice(&c.saturation_threshold_basis_points.to_be_bytes());
    buf.extend_from_slice(&c.observed_percent_of_peak_basis_points.to_be_bytes());
    buf.push(u8::from(c.saturation_admitted));
    sha256(&buf)
}

/// WHY: top-level META-hash. Binds the measurement hash +
/// claim hash + five upstream anchor hashes (S-PERF.2 +
/// S-PERF.3 + S-PERF.4 + S-PERF.5) + three R.12b
/// episode-count integrity pins. The upstream anchors are
/// stored as raw 32-byte hashes (NOT length-prefixed; they
/// are always exactly 32 bytes) so the byte form is the
/// same regardless of which upstream module computed them.
/// The three R.12b pins are big-endian u32s so any drift
/// in the panel-locked (13, 89, 1917) tuple changes the
/// baseline hash and breaks
/// `PINNED_MEASURED_BASELINE_REPORT_HASH_V1` in the test
/// suite -- this is the cross-corpus integrity guard from
/// panel-required negative #10.
fn compute_rtx4080_super_measured_baseline_report_hash(
    r: &Rtx4080SuperMeasuredBaselineReportV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S_PERF_6_MEASURED_BASELINE_REPORT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S_PERF_6_MEASURED_BASELINE_REPORT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.baseline_id.as_bytes());
    buf.extend_from_slice(&r.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1);
    buf.extend_from_slice(&r.claim.rtx4080_super_measured_bandwidth_claim_hash_v1);
    buf.extend_from_slice(&r.s_perf_2_layer_a_traffic_receipt_hash);
    buf.extend_from_slice(&r.s_perf_3_public_data_bundle_hash);
    buf.extend_from_slice(&r.s_perf_4_family_compaction_benchmark_schema_hash);
    buf.extend_from_slice(&r.s_perf_5_effective_bandwidth_report_hash);
    buf.extend_from_slice(&r.r12b_episode_count_canonical_w16h128.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_mid_w64h512.to_be_bytes());
    buf.extend_from_slice(&r.r12b_episode_count_full_w256h4096.to_be_bytes());
    sha256(&buf)
}

/// WHY: prepends a big-endian `u32` length to the byte
/// payload so two different strings cannot hash to the
/// same buffer just because their concatenation aliases
/// (e.g. `"ab" + "cd"` vs `"abc" + "d"`). The saturating
/// `try_from` is defensive against a hypothetical >4 GB
/// string -- in practice every string this function sees
/// is bounded by panel-locked wire names (under 200
/// bytes), so the saturating branch is unreachable; it
/// exists so the function is a total function with no
/// panic path.
fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify an S-PERF.6 measured baseline report. Returns the
/// list of errors (empty when the report satisfies every
/// panel-required + structural rule).
#[must_use]
#[allow(clippy::too_many_lines)] // 14 panel-required negatives + structural
pub fn verify_rtx4080_super_measured_baseline_report(
    report: &Rtx4080SuperMeasuredBaselineReportV1,
) -> Vec<SPerf6VerifyError> {
    let mut errors: Vec<SPerf6VerifyError> = Vec::new();
    let m = &report.measurement;
    let c = &report.claim;

    // Panel-required negative #1.
    if m.measured_wide_bandwidth_centi_gbps == 0 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ZeroMeasuredBandwidth,
        });
    }

    // Panel-required negative #2.
    if m.device_total_us == 0 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ZeroDeviceTotalTime,
        });
    }

    // Panel-required negative #3.
    if m.source_report_path.is_empty() {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingSourceReportPath,
        });
    }

    // Panel-required negative #4: device identity.
    if m.device_name != RTX_4080_SUPER_DEVICE_NAME {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity {
                which_field_wire_name: "device_name",
            },
        });
    }
    if m.sm_arch != RTX_4080_SUPER_SM_ARCH {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity {
                which_field_wire_name: "sm_arch",
            },
        });
    }
    let expected_identity =
        compute_device_identity_hash(RTX_4080_SUPER_DEVICE_NAME, RTX_4080_SUPER_SM_ARCH);
    if m.device_uuid_or_identity_hash != expected_identity {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity {
                which_field_wire_name: "device_uuid_or_identity_hash",
            },
        });
    }
    if m.theoretical_memory_bandwidth_gbps != RTX_4080_SUPER_THEORETICAL_PEAK_GBPS {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingRtx4080SuperIdentity {
                which_field_wire_name: "theoretical_memory_bandwidth_gbps",
            },
        });
    }

    // Panel-required negative #5: arithmetic coherence.
    let expected_pct_bp = compute_s_perf_6_percent_of_peak_basis_points(
        m.measured_wide_bandwidth_centi_gbps,
        m.theoretical_memory_bandwidth_gbps,
    );
    if m.percent_of_peak_basis_points != expected_pct_bp {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::PercentOfPeakArithmeticMismatch {
                claimed: m.percent_of_peak_basis_points,
                computed: expected_pct_bp,
            },
        });
    }

    // Panel-required negative #6: saturation_admitted with
    // observed bp below threshold.
    if c.saturation_admitted && c.observed_percent_of_peak_basis_points < S_PERF_1_SATURATION_BP {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::SaturationClaimBelow8000Bp {
                observed_bp: c.observed_percent_of_peak_basis_points,
            },
        });
    }

    // Panel-required negative #7: "13.33 GB/s is saturation"
    // is forbidden. Fires when saturation_admitted == true
    // AND the measured bandwidth is below what 8000 bp of
    // 716 GB/s would require (i.e. < ~572 GB/s = 57200
    // centi-GB/s). The 1333 centi-GB/s baseline trivially
    // satisfies this guard. Also fires on forbidden
    // "saturation" substrings in admissibility_reason.
    let saturation_floor_centi_gbps: u32 =
        (u64::from(S_PERF_1_SATURATION_BP) * u64::from(m.theoretical_memory_bandwidth_gbps) * 100
            / 10_000) as u32;
    if c.saturation_admitted && m.measured_wide_bandwidth_centi_gbps < saturation_floor_centi_gbps {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ClaimThat7_70GbpsIsSaturation,
        });
    }
    // Forbidden saturation-claim substring check on
    // admissibility_reason (case-insensitive).
    if contains_ascii_case_insensitive(c.admissibility_reason_wire_name, "achieves saturation")
        || contains_ascii_case_insensitive(
            c.admissibility_reason_wire_name,
            "saturates the bandwidth",
        )
        || contains_ascii_case_insensitive(c.admissibility_reason_wire_name, "saturates peak")
        || contains_ascii_case_insensitive(
            c.admissibility_reason_wire_name,
            "memory-bandwidth saturation",
        )
    {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ClaimThat7_70GbpsIsSaturation,
        });
    }

    // Panel-required negative #8: B300 / GB300 forbidden
    // substrings.
    if contains_ascii_case_insensitive(c.admissibility_reason_wire_name, "b300")
        || contains_ascii_case_insensitive(c.admissibility_reason_wire_name, "gb300")
        || contains_ascii_case_insensitive(m.source_report_path, "b300")
        || contains_ascii_case_insensitive(m.source_report_path, "gb300")
    {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ClaimThatResultIsB300OrGb300,
        });
    }

    // Panel-required negative #9: production-performance
    // forbidden substrings.
    if contains_ascii_case_insensitive(c.admissibility_reason_wire_name, "production performance")
        || contains_ascii_case_insensitive(
            c.admissibility_reason_wire_name,
            "production-ready performance",
        )
        || contains_ascii_case_insensitive(
            c.admissibility_reason_wire_name,
            "production cuda performance",
        )
    {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ClaimThatResultIsProductionPerformance,
        });
    }

    // Panel-required negative #10: R.12b episode-count
    // rebaseline rejection.
    if report.r12b_episode_count_canonical_w16h128 != R12B_EPISODE_COUNT_CANONICAL_W16H128 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts {
                which_pin_wire_name: "r12b_episode_count_canonical_w16h128",
                declared: report.r12b_episode_count_canonical_w16h128,
                panel_locked: R12B_EPISODE_COUNT_CANONICAL_W16H128,
            },
        });
    }
    if report.r12b_episode_count_mid_w64h512 != R12B_EPISODE_COUNT_MID_W64H512 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts {
                which_pin_wire_name: "r12b_episode_count_mid_w64h512",
                declared: report.r12b_episode_count_mid_w64h512,
                panel_locked: R12B_EPISODE_COUNT_MID_W64H512,
            },
        });
    }
    if report.r12b_episode_count_full_w256h4096 != R12B_EPISODE_COUNT_FULL_W256H4096 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::RebaselineOfR12bEpisodeCounts {
                which_pin_wire_name: "r12b_episode_count_full_w256h4096",
                declared: report.r12b_episode_count_full_w256h4096,
                panel_locked: R12B_EPISODE_COUNT_FULL_W256H4096,
            },
        });
    }

    // Panel-required negative #11: tree_digest stage timing.
    if m.tree_digest_consensus_us == 0 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingTreeDigestStageTiming,
        });
    }

    // Panel-required negative #12: host-segment disclosure.
    if m.host_compute_features_us == 0 && m.host_bank_admit_case_finalize_us == 0 {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::MissingHostSegmentDisclosure,
        });
    }

    // Panel-required negative #13: empty claim kind /
    // empty admissibility reason.
    if c.admissibility_reason_wire_name.is_empty() {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::EmptyClaimKind,
        });
    }

    // Panel-required negative #14: NoClaim baseline for
    // measured result. (The enum has only one variant
    // today, but the variant check is a forward-compatibility
    // guard: any future schema-upgrade that adds a "NoClaim"
    // variant on this enum would surface here.)
    match c.claim_kind {
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth => {}
    }
    // Additional defence: admitted must be true for the
    // measured-CUDA-pipeline-bandwidth variant (the only
    // legitimate measured-result outcome). Reject any
    // measured baseline that declares admitted=false; that
    // is the "NoClaim baseline for measured result" failure
    // mode the panel forbids.
    if matches!(
        c.claim_kind,
        MeasuredCudaPipelineClaimKind::MeasuredCudaPipelineBandwidth
    ) && !c.admitted
    {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::NoClaimBaselineForMeasuredResult,
        });
    }

    // Structural defect: empty baseline_id.
    if report.baseline_id.is_empty() {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::BaselineIdEmpty,
        });
    }

    // Structural defect: saturation threshold mismatch.
    if c.saturation_threshold_basis_points != S_PERF_1_SATURATION_BP {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::SaturationThresholdMismatch {
                declared: c.saturation_threshold_basis_points,
            },
        });
    }

    // Structural defect: observed percent-of-peak coherence.
    if c.observed_percent_of_peak_basis_points != m.percent_of_peak_basis_points {
        errors.push(SPerf6VerifyError {
            kind: SPerf6VerifyErrorKind::ObservedPercentOfPeakMismatch {
                claim_observed: c.observed_percent_of_peak_basis_points,
                measurement_observed: m.percent_of_peak_basis_points,
            },
        });
    }

    // Catch-all forbidden-substring scan over both fields,
    // excluding the more-specific scans above.
    scan_for_forbidden_substring(
        c.admissibility_reason_wire_name,
        "admissibility_reason_wire_name",
        &mut errors,
    );
    scan_for_forbidden_substring(m.source_report_path, "source_report_path", &mut errors);

    errors
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// WHY: the catch-all 16-substring scanner that fires
/// `ForbiddenSubstringInsideReport` for any phrase that
/// would let a future caller smuggle a saturation /
/// production-performance / B300 / GB300 / world-record
/// claim into a free-text field. The specific panel-
/// required negatives (#7 / #8 / #9) duplicate some of
/// these checks with their own kinds for operator
/// legibility, but this catch-all guarantees no forbidden
/// phrase escapes through a wire field we forgot to
/// special-case. The scanner runs over both
/// `admissibility_reason_wire_name` and
/// `source_report_path` per the directive.
fn scan_for_forbidden_substring(
    text: &'static str,
    location: &'static str,
    errors: &mut Vec<SPerf6VerifyError>,
) {
    for &forbidden in S_PERF_6_FORBIDDEN_CLAIM_SUBSTRINGS {
        if contains_ascii_case_insensitive(text, forbidden) {
            errors.push(SPerf6VerifyError {
                kind: SPerf6VerifyErrorKind::ForbiddenSubstringInsideReport {
                    location,
                    forbidden_substring: forbidden,
                },
            });
        }
    }
}

/// WHY: case-insensitive substring scan over ASCII bytes.
/// The corpus crate is panel-locked zero-dep so we cannot
/// pull `regex` or `aho-corasick`; this hand-rolled scan
/// is intentionally simple and slow (O(haystack × needle))
/// because the haystacks are short panel-locked wire
/// names. The empty-needle guard returns false so an
/// accidentally empty entry in the forbidden list cannot
/// match every string. `eq_ignore_ascii_case` handles the
/// case folding byte-for-byte without allocating; the
/// scanner is correct for ASCII-only inputs, which every
/// panel-locked wire field is by construction (the
/// forbidden-claim substrings are also ASCII).
fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    for window_start in 0..=h.len() - n.len() {
        let mut ok = true;
        for i in 0..n.len() {
            if !h[window_start + i].eq_ignore_ascii_case(&n[i]) {
                ok = false;
                break;
            }
        }
        if ok {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------
// Panel-locked report + bottleneck sentences (verbatim)
// ---------------------------------------------------------------

/// Panel-locked report sentence (MUST appear verbatim in
/// receipts, README, and paper).
pub const S_PERF_6_REPORT_SENTENCE: &str =
    "The RTX 4080 SUPER measured CUDA pipeline baseline reports 13.33 GB/s, approximately 1.86% of the declared 716 GB/s theoretical memory-bandwidth anchor. This is an admissible measured CUDA pipeline bandwidth result, not a saturation claim.";

/// Panel-locked bottleneck sentence (MUST appear verbatim in
/// receipts and paper).
pub const S_PERF_6_BOTTLENECK_SENTENCE: &str =
    "The profile does not indicate memory-bandwidth saturation. The measured path is dominated by pipeline structure including tree_digest consensus and host-side feature/admission/finalization segments.";

// ---------------------------------------------------------------
// Renderers --- text
// ---------------------------------------------------------------

/// Render the measured CUDA pipeline record as
/// deterministic text.
#[must_use]
pub fn render_rtx4080_super_measured_cuda_pipeline_text(
    p: &Rtx4080SuperMeasuredCudaPipelineV1,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.6 Rtx4080SuperMeasuredCudaPipelineV1");
    let _ = writeln!(s, "=============================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Device identity (panel-locked RTX 4080 SUPER)");
    let _ = writeln!(s, "  device_name           : {}", p.device_name);
    let _ = writeln!(s, "  sm_arch               : sm_{}", p.sm_arch);
    let _ = writeln!(s, "  cuda_version          : {}", p.cuda_version);
    let _ = writeln!(
        s,
        "  theoretical_peak_gbps : {}",
        p.theoretical_memory_bandwidth_gbps
    );
    let _ = writeln!(
        s,
        "  device_identity_hash  : {}",
        hex32(&p.device_uuid_or_identity_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Measured stage timings (microseconds)");
    let _ = writeln!(
        s,
        "  host_wall_median_us              : {}",
        p.host_wall_median_us
    );
    let _ = writeln!(
        s,
        "  device_total_us                  : {}",
        p.device_total_us
    );
    let _ = writeln!(
        s,
        "  consensus_grid_kernel_wide_us    : {}",
        p.consensus_grid_kernel_wide_us
    );
    let _ = writeln!(
        s,
        "  tree_digest_consensus_us         : {} ({}.{:02}%)",
        p.tree_digest_consensus_us,
        p.tree_digest_consensus_percent_basis_points / 100,
        p.tree_digest_consensus_percent_basis_points % 100,
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Host segments (honestly disclosed)");
    let _ = writeln!(
        s,
        "  host_compute_features_us         : {}",
        p.host_compute_features_us
    );
    let _ = writeln!(
        s,
        "  host_bank_admit_case_finalize_us : {}",
        p.host_bank_admit_case_finalize_us
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Measured bandwidth");
    let _ = writeln!(
        s,
        "  measured_wide_bandwidth          : {}.{:02} GB/s ({} centi-GB/s)",
        p.measured_wide_bandwidth_centi_gbps / 100,
        p.measured_wide_bandwidth_centi_gbps % 100,
        p.measured_wide_bandwidth_centi_gbps,
    );
    let _ = writeln!(
        s,
        "  percent_of_peak_basis_points     : {} ({}.{:02}%)",
        p.percent_of_peak_basis_points,
        p.percent_of_peak_basis_points / 100,
        p.percent_of_peak_basis_points % 100,
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Provenance");
    let _ = writeln!(s, "  source_report_path : {}", p.source_report_path);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "rtx4080_super_measured_cuda_pipeline_hash_v1 : {}",
        hex32(&p.rtx4080_super_measured_cuda_pipeline_hash_v1)
    );
    s
}

/// Render the bandwidth-claim verdict as deterministic text.
#[must_use]
pub fn render_rtx4080_super_measured_bandwidth_claim_text(
    c: &Rtx4080SuperMeasuredBandwidthClaimV1,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.6 Rtx4080SuperMeasuredBandwidthClaimV1");
    let _ = writeln!(s, "===============================================");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "  claim_kind                              : {}",
        c.claim_kind.as_str()
    );
    let _ = writeln!(
        s,
        "  admissibility_reason_wire_name          : {}",
        c.admissibility_reason_wire_name
    );
    let _ = writeln!(
        s,
        "  admitted                                : {}",
        c.admitted
    );
    let _ = writeln!(
        s,
        "  saturation_threshold_basis_points       : {}",
        c.saturation_threshold_basis_points
    );
    let _ = writeln!(
        s,
        "  observed_percent_of_peak_basis_points   : {}",
        c.observed_percent_of_peak_basis_points
    );
    let _ = writeln!(
        s,
        "  saturation_admitted                     : {}",
        c.saturation_admitted
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "rtx4080_super_measured_bandwidth_claim_hash_v1 : {}",
        hex32(&c.rtx4080_super_measured_bandwidth_claim_hash_v1)
    );
    s
}

/// Render the baseline report as deterministic text.
#[must_use]
pub fn render_rtx4080_super_measured_baseline_report_text(
    r: &Rtx4080SuperMeasuredBaselineReportV1,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.6 Rtx4080SuperMeasuredBaselineReportV1");
    let _ = writeln!(s, "===============================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Identity");
    let _ = writeln!(s, "  baseline_id : {}", r.baseline_id);
    let _ = writeln!(s);
    let _ = writeln!(s, "Bound hashes");
    let _ = writeln!(
        s,
        "  rtx4080_super_measured_cuda_pipeline_hash_v1     : {}",
        hex32(&r.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1)
    );
    let _ = writeln!(
        s,
        "  rtx4080_super_measured_bandwidth_claim_hash_v1   : {}",
        hex32(&r.claim.rtx4080_super_measured_bandwidth_claim_hash_v1)
    );
    let _ = writeln!(
        s,
        "  s_perf_2_layer_a_traffic_receipt_hash            : {}",
        hex32(&r.s_perf_2_layer_a_traffic_receipt_hash)
    );
    let _ = writeln!(
        s,
        "  s_perf_3_public_data_bundle_hash                 : {}",
        hex32(&r.s_perf_3_public_data_bundle_hash)
    );
    let _ = writeln!(
        s,
        "  s_perf_4_family_compaction_benchmark_schema_hash : {}",
        hex32(&r.s_perf_4_family_compaction_benchmark_schema_hash)
    );
    let _ = writeln!(
        s,
        "  s_perf_5_effective_bandwidth_report_hash         : {}",
        hex32(&r.s_perf_5_effective_bandwidth_report_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "R.12b episode-count integrity pins (panel-locked)");
    let _ = writeln!(
        s,
        "  canonical 16x128 : {}",
        r.r12b_episode_count_canonical_w16h128
    );
    let _ = writeln!(
        s,
        "  mid       64x512 : {}",
        r.r12b_episode_count_mid_w64h512
    );
    let _ = writeln!(
        s,
        "  full     256x4096: {}",
        r.r12b_episode_count_full_w256h4096
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Verdict");
    let _ = writeln!(s, "  claim_kind          : {}", r.claim.claim_kind.as_str());
    let _ = writeln!(s, "  admitted            : {}", r.claim.admitted);
    let _ = writeln!(s, "  saturation_admitted : {}", r.claim.saturation_admitted);
    let _ = writeln!(
        s,
        "  observed_bp         : {} (threshold {})",
        r.claim.observed_percent_of_peak_basis_points, r.claim.saturation_threshold_basis_points,
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked report sentence");
    let _ = writeln!(s, "  {S_PERF_6_REPORT_SENTENCE}");
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked bottleneck sentence");
    let _ = writeln!(s, "  {S_PERF_6_BOTTLENECK_SENTENCE}");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "rtx4080_super_measured_baseline_report_hash_v1 : {}",
        hex32(&r.rtx4080_super_measured_baseline_report_hash_v1)
    );
    s
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render the measured CUDA pipeline record as canonical
/// JSON.
#[must_use]
pub fn render_rtx4080_super_measured_cuda_pipeline_json(
    p: &Rtx4080SuperMeasuredCudaPipelineV1,
) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        S_PERF_6_MEASURED_CUDA_PIPELINE_SCHEMA_V1,
    );
    s.push(',');
    json_string(&mut s, "device_name", p.device_name);
    s.push(',');
    let _ = write!(s, "\"sm_arch\":{}", p.sm_arch);
    s.push(',');
    json_hex(
        &mut s,
        "device_uuid_or_identity_hash",
        &p.device_uuid_or_identity_hash,
    );
    s.push(',');
    json_string(&mut s, "cuda_version", p.cuda_version);
    s.push(',');
    let _ = write!(
        s,
        "\"theoretical_memory_bandwidth_gbps\":{}",
        p.theoretical_memory_bandwidth_gbps
    );
    s.push(',');
    let _ = write!(s, "\"host_wall_median_us\":{}", p.host_wall_median_us);
    s.push(',');
    let _ = write!(s, "\"device_total_us\":{}", p.device_total_us);
    s.push(',');
    let _ = write!(
        s,
        "\"consensus_grid_kernel_wide_us\":{}",
        p.consensus_grid_kernel_wide_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"tree_digest_consensus_us\":{}",
        p.tree_digest_consensus_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"tree_digest_consensus_percent_basis_points\":{}",
        p.tree_digest_consensus_percent_basis_points
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_compute_features_us\":{}",
        p.host_compute_features_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_bank_admit_case_finalize_us\":{}",
        p.host_bank_admit_case_finalize_us
    );
    s.push(',');
    let _ = write!(
        s,
        "\"measured_wide_bandwidth_centi_gbps\":{}",
        p.measured_wide_bandwidth_centi_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"percent_of_peak_basis_points\":{}",
        p.percent_of_peak_basis_points
    );
    s.push(',');
    json_string(&mut s, "source_report_path", p.source_report_path);
    s.push(',');
    json_hex(
        &mut s,
        "rtx4080_super_measured_cuda_pipeline_hash_v1",
        &p.rtx4080_super_measured_cuda_pipeline_hash_v1,
    );
    s.push('}');
    s
}

/// Render the bandwidth-claim verdict as canonical JSON.
#[must_use]
pub fn render_rtx4080_super_measured_bandwidth_claim_json(
    c: &Rtx4080SuperMeasuredBandwidthClaimV1,
) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        S_PERF_6_MEASURED_BANDWIDTH_CLAIM_SCHEMA_V1,
    );
    s.push(',');
    json_string(&mut s, "claim_kind", c.claim_kind.as_str());
    s.push(',');
    json_string(
        &mut s,
        "admissibility_reason_wire_name",
        c.admissibility_reason_wire_name,
    );
    s.push(',');
    let _ = write!(s, "\"admitted\":{}", c.admitted);
    s.push(',');
    let _ = write!(
        s,
        "\"saturation_threshold_basis_points\":{}",
        c.saturation_threshold_basis_points
    );
    s.push(',');
    let _ = write!(
        s,
        "\"observed_percent_of_peak_basis_points\":{}",
        c.observed_percent_of_peak_basis_points
    );
    s.push(',');
    let _ = write!(s, "\"saturation_admitted\":{}", c.saturation_admitted);
    s.push(',');
    json_hex(
        &mut s,
        "rtx4080_super_measured_bandwidth_claim_hash_v1",
        &c.rtx4080_super_measured_bandwidth_claim_hash_v1,
    );
    s.push('}');
    s
}

/// Render the baseline report as canonical JSON.
#[must_use]
pub fn render_rtx4080_super_measured_baseline_report_json(
    r: &Rtx4080SuperMeasuredBaselineReportV1,
) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        S_PERF_6_MEASURED_BASELINE_REPORT_SCHEMA_V1,
    );
    s.push(',');
    json_string(&mut s, "baseline_id", r.baseline_id);
    s.push(',');
    json_hex(
        &mut s,
        "rtx4080_super_measured_cuda_pipeline_hash_v1",
        &r.measurement.rtx4080_super_measured_cuda_pipeline_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "rtx4080_super_measured_bandwidth_claim_hash_v1",
        &r.claim.rtx4080_super_measured_bandwidth_claim_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_2_layer_a_traffic_receipt_hash",
        &r.s_perf_2_layer_a_traffic_receipt_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_3_public_data_bundle_hash",
        &r.s_perf_3_public_data_bundle_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_4_family_compaction_benchmark_schema_hash",
        &r.s_perf_4_family_compaction_benchmark_schema_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "s_perf_5_effective_bandwidth_report_hash",
        &r.s_perf_5_effective_bandwidth_report_hash,
    );
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
    json_string(&mut s, "claim_kind", r.claim.claim_kind.as_str());
    s.push(',');
    let _ = write!(s, "\"admitted\":{}", r.claim.admitted);
    s.push(',');
    let _ = write!(s, "\"saturation_admitted\":{}", r.claim.saturation_admitted);
    s.push(',');
    json_hex(
        &mut s,
        "rtx4080_super_measured_baseline_report_hash_v1",
        &r.rtx4080_super_measured_baseline_report_hash_v1,
    );
    s.push('}');
    s
}

/// WHY: emits a single `"key":"value"` pair into the JSON
/// buffer for a panel-locked schema-id field. No JSON
/// escaping is performed because every key + value passed
/// to this function is a `&str` controlled by the panel
/// (schema ids and wire-name constants in this module),
/// none of which contain `"` or `\` characters. A future
/// caller passing untrusted text here would break the
/// invariant; the helper is `fn`-private precisely to
/// prevent that.
fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

/// WHY: same shape as `json_field` but with a public-name
/// distinction at the call site so a reader knows which
/// fields hold caller-supplied strings vs panel-locked
/// schema identifiers. The actual byte output is the
/// same; the function exists to make the rendered JSON
/// audit-trail self-documenting. Every value this helper
/// emits today is still panel-locked (device_name,
/// cuda_version, claim_kind wire name, admissibility
/// reason wire name, source_report_path) so the no-escape
/// invariant holds.
fn json_string(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

/// WHY: emits a 32-byte hash as a 64-character lowercase
/// hex string field. The hex form is used (vs base64 or
/// raw bytes) because the JSON renderer is meant to be
/// human-readable by auditors comparing against the
/// `cargo run -- s-perf-6r-baseline` text output; both
/// renderers emit the same hex form so a reader can
/// `grep` for a hash prefix across either artifact.
fn json_hex(s: &mut String, key: &str, value: &[u8; 32]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    let _ = s.write_str(&hex32(value));
    s.push('"');
}

/// WHY: format a 32-byte hash as a 64-character lowercase
/// hex string. The two-character `{:02x}` width is
/// load-bearing -- without it, a leading-zero byte would
/// drop to a single hex digit and the resulting string
/// would be the wrong length for hash comparison. Used by
/// both the text and JSON renderers; the deterministic
/// formatting matters because renderer byte-stability is
/// asserted in the test suite.
fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Test-only read-only access to the forbidden-claim
/// substring set.
#[doc(hidden)]
#[must_use]
pub fn forbidden_claim_substrings() -> &'static [&'static str] {
    S_PERF_6_FORBIDDEN_CLAIM_SUBSTRINGS
}
