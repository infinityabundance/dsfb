//! S-PERF.5 --- effective-bandwidth report.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S-PERF.5 computes and reports effective bandwidth for
//! > the Layer-A evidence factory using the S-PERF.1 traffic
//! > law, S-PERF.2 Layer-A boundary, S-PERF.3 public workload
//! > bundle, and S-PERF.4 compacted active-family plan. It
//! > does not claim saturation unless the measured percent-
//! > of-peak meets the S-PERF.1 threshold and the timing
//! > method is admissible.**
//!
//! Core rule (panel-locked):
//!
//! > Effective bandwidth report ≠ saturation claim.
//! > Saturation requires accounted device bytes, admissible
//! > CUDA timing, declared device peak bandwidth,
//! > percent_of_peak >= 8000 bp, and a Layer-A-only timing
//! > boundary.
//!
//! ## Why
//!
//! S-PERF.1 supplied the measurement law (byte accounting +
//! 8 panel-locked rules). S-PERF.2 isolated the Layer-A
//! pipeline (no host JSON / casefile / transcript /
//! semantic-admission time mixed in). S-PERF.3 pinned the
//! public-data workload. S-PERF.4 compacted the 152 active
//! detectors into 14 GPU-family lanes. S-PERF.5 is the
//! *verdict layer*: given a measured `DeviceTrafficReceiptV1`,
//! compute the effective bandwidth and decide whether the
//! claim it implies is admissible under the S-PERF.1 +
//! S-PERF.2 + S-PERF.3 + S-PERF.4 anchors.
//!
//! The verifier rejects ten panel-required negatives spanning
//! all four upstream-citation rules, saturation-threshold
//! rules, byte / time arithmetic coherence, Layer-A
//! host-timing exclusion, cross-device-identity discipline,
//! and benchmark-claim-requires-public-bundle discipline.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes (none folded upstream):
//!
//! - `layer_a_bandwidth_measurement_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-BANDWIDTH-MEASUREMENT:v1\0`.
//!   Pins the raw measurement bytes + time + computed
//!   effective bandwidth + percent-of-peak + the inner
//!   `DeviceTrafficReceiptV1` reference + the LayerA
//!   forbidden-flag mirror.
//! - `bandwidth_claim_admission_hash_v1` under
//!   `DSFB-GPU-ATLAS:BANDWIDTH-CLAIM-ADMISSION:v1\0`. Pins
//!   the verdict: which claim kind, which admissibility
//!   reason wire name, and whether the claim was admitted.
//! - `effective_bandwidth_report_hash_v1` under
//!   `DSFB-GPU-ATLAS:EFFECTIVE-BANDWIDTH-REPORT:v1\0`.
//!   Top-level META-hash binding the measurement + the
//!   admission + the four upstream anchor hashes
//!   (`device_traffic_receipt_hash_v1`,
//!   `layer_a_traffic_receipt_hash`,
//!   `public_data_bundle_hash`,
//!   `family_compaction_benchmark_schema_hash`).
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.5 does NOT:
//!
//! - claim memory-bandwidth saturation at baseline (the
//!   baseline report is `BandwidthClaimKind::NoClaim`;
//!   future S-PERF.* commits emit measured reports);
//! - run any benchmark itself;
//! - change any CUDA kernel;
//! - change any court decision (S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e / S1.3f / S1.3g);
//! - mutate any upstream hash anchor (`corpus_hash_v1`,
//!   `corpus_hash_v2`, every T.11.* / T.12.* / FF.* /
//!   S1.3.* / T.12.PROV / S-PERF.1 / S-PERF.2 / S-PERF.3
//!   / S-PERF.4 hash byte-identical);
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs, witness records, fusion
//!   tensors, candidate intervals, or episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate;
//! - download or fetch any dataset bytes.
//!
//! S-PERF.5 ships ONLY the report schema + verifier +
//! builder + uninstrumented baseline + renderers.
//!
//! ## Panel-locked one-line verdict
//!
//! > S-PERF.4 packs the active witnesses into benchmarkable
//! > lanes; S-PERF.5 turns measured Layer-A bytes and time
//! > into an admissible bandwidth report.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::s_perf_1_device_traffic_receipt::{
    seed_baseline_uninstrumented_receipt, S_PERF_1_SATURATION_BP,
};
use crate::s_perf_2_layer_a_resident_pipeline::seed_baseline_layer_a_traffic_receipt;
use crate::s_perf_3_public_data_saturation_bundle::seed_baseline_public_data_saturation_bundle;
use crate::s_perf_4_active_family_compaction::seed_baseline_family_compaction_benchmark_schema;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for
/// `layer_a_bandwidth_measurement_hash_v1`.
pub const LAYER_A_BANDWIDTH_MEASUREMENT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:LAYER-A-BANDWIDTH-MEASUREMENT:v1\0";

/// Schema identifier for
/// `layer_a_bandwidth_measurement_hash_v1`.
pub const LAYER_A_BANDWIDTH_MEASUREMENT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:LAYER-A-BANDWIDTH-MEASUREMENT:v1";

/// Domain separator for
/// `bandwidth_claim_admission_hash_v1`.
pub const BANDWIDTH_CLAIM_ADMISSION_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:BANDWIDTH-CLAIM-ADMISSION:v1\0";

/// Schema identifier for
/// `bandwidth_claim_admission_hash_v1`.
pub const BANDWIDTH_CLAIM_ADMISSION_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:BANDWIDTH-CLAIM-ADMISSION:v1";

/// Domain separator for
/// `effective_bandwidth_report_hash_v1`.
pub const EFFECTIVE_BANDWIDTH_REPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:EFFECTIVE-BANDWIDTH-REPORT:v1\0";

/// Schema identifier for
/// `effective_bandwidth_report_hash_v1`.
pub const EFFECTIVE_BANDWIDTH_REPORT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:EFFECTIVE-BANDWIDTH-REPORT:v1";

// ---------------------------------------------------------------
// Forbidden benchmark-claim substrings (mirrors S-PERF.3 /
// S-PERF.4 set verbatim)
// ---------------------------------------------------------------

/// Substrings that MUST NOT appear in the S-PERF.5 report's
/// free-text fields (`report_id` and
/// `admissibility_reason_wire_name`). Even though S-PERF.5
/// is the verdict layer and admissibility reasons reference
/// claims directly, the prose CLAIMS belong in the structured
/// `claim_kind` + `admitted` fields, not in free-text
/// flourishes. The scanner is case-insensitive.
const S_PERF_5_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS: &[&str] = &[
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
    "petaflops",
    "memory-bandwidth saturation",
];

// ---------------------------------------------------------------
// BandwidthClaimKind
// ---------------------------------------------------------------

/// What kind of bandwidth claim the report makes.
///
/// Wire names are stable for the hash buffer; do not rename
/// without rebaselining
/// `bandwidth_claim_admission_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BandwidthClaimKind {
    /// Baseline / uninstrumented report. No measurement was
    /// taken; the report exists to pin the receipt chain.
    NoClaim,
    /// Effective bandwidth claim only (non-zero
    /// `effective_bandwidth_gbps` backed by non-zero
    /// `total_accounted_device_bytes` and
    /// `measured_kernel_time_us`).
    EffectiveBandwidth,
    /// Percent-of-peak claim below the panel-locked
    /// saturation threshold (`percent_of_peak_basis_points <
    /// S_PERF_1_SATURATION_BP`).
    PercentOfPeak,
    /// Saturation claim (`percent_of_peak_basis_points >=
    /// S_PERF_1_SATURATION_BP`). Requires CUDA-event or
    /// CUDA-stream-sync timing per the panel-locked saturation
    /// rule.
    Saturation,
}

impl BandwidthClaimKind {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoClaim => "NoClaim",
            Self::EffectiveBandwidth => "EffectiveBandwidth",
            Self::PercentOfPeak => "PercentOfPeak",
            Self::Saturation => "Saturation",
        }
    }
}

// ---------------------------------------------------------------
// Panel-locked admissibility reason wire names
// ---------------------------------------------------------------

/// Admissibility reason for a `NoClaim` baseline (the
/// reasoning trail is "no claim, no rules to check beyond
/// schema integrity").
pub const ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM: &str = "AdmissibleNoClaim";

/// Admissibility reason for an `EffectiveBandwidth` claim
/// backed by non-zero byte accounting + non-zero time.
pub const ADMISSIBILITY_REASON_ADMISSIBLE_EFFECTIVE_BANDWIDTH: &str =
    "AdmissibleEffectiveBandwidthBackedByByteAccounting";

/// Admissibility reason for a `PercentOfPeak` claim with a
/// declared theoretical peak bandwidth.
pub const ADMISSIBILITY_REASON_ADMISSIBLE_PERCENT_OF_PEAK: &str =
    "AdmissiblePercentOfPeakWithDeclaredDeviceBandwidth";

/// Admissibility reason for a `Saturation` claim backed by
/// CUDA-event timing (or CUDA-stream-sync), `percent_of_peak
/// >= 8000` bp, and the full Layer-A receipt chain.
pub const ADMISSIBILITY_REASON_ADMISSIBLE_SATURATION: &str =
    "AdmissibleSaturationWithCudaEventTimingAndFullReceiptChain";

// ---------------------------------------------------------------
// LayerABandwidthMeasurementV1
// ---------------------------------------------------------------

/// The raw bandwidth measurement record. Mirrors the relevant
/// fields from the cited `DeviceTrafficReceiptV1` plus the
/// LayerA forbidden-flag mirror so the verifier can enforce
/// the Layer-A timing-boundary rule without re-resolving the
/// hash reference.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `layer_a_bandwidth_measurement_hash_v1`.
#[derive(Debug, Clone)]
pub struct LayerABandwidthMeasurementV1 {
    /// Hash of the S-PERF.1 `DeviceTrafficReceiptV1` this
    /// measurement was taken from. Non-zero (panel-required
    /// negative #1).
    pub device_traffic_receipt_hash_v1: [u8; 32],
    /// Device identity hash (must equal the inner receipt's
    /// `device_uuid_or_identity_hash`).
    pub device_uuid_or_identity_hash: [u8; 32],
    /// Theoretical peak memory bandwidth, in GB/s (e.g. 716
    /// for RTX 4080 SUPER). Required to be non-zero for any
    /// percent-of-peak claim (S-PERF.1 negative #2 mirror).
    pub theoretical_memory_bandwidth_gbps: u32,
    /// Measured kernel time, in microseconds.
    pub measured_kernel_time_us: u64,
    /// Timing method wire name (mirrors S-PERF.1's
    /// `TimingMethod::as_str` value). Layer-A measurements
    /// MUST be timed with `CudaEvent` or `CudaStreamSync` for
    /// saturation claims (panel-required negative #6).
    pub timing_method_wire_name: &'static str,
    /// Total accounted device bytes (mirrors S-PERF.1's
    /// `total_accounted_device_bytes`).
    pub total_accounted_device_bytes: u64,
    /// Effective bandwidth in integer GB/s, computed as
    /// `total_accounted_device_bytes` divided by
    /// `(measured_kernel_time_us * 1000)`. The verifier rejects
    /// mismatches via panel-required negative number 7.
    pub effective_bandwidth_gbps: u32,
    /// Percent-of-peak in basis points (10000 = 100.00 %).
    /// Computed as `effective_bandwidth_gbps * 10000 /
    /// theoretical_memory_bandwidth_gbps`. The verifier
    /// rejects mismatches via the same arithmetic-coherence
    /// rule.
    pub percent_of_peak_basis_points: u32,
    /// Mirror of the inner LayerATrafficReceipt's pipeline
    /// `host_json_emission_present` flag. MUST be `false`
    /// (panel-required negative #8 part 1).
    pub inner_host_json_emission_present: bool,
    /// Mirror of the inner LayerATrafficReceipt's pipeline
    /// `casefile_materialization_present` flag. MUST be
    /// `false` (panel-required negative #8 part 2).
    pub inner_casefile_materialization_present: bool,
    /// Mirror of the inner LayerATrafficReceipt's pipeline
    /// `host_transcript_present` flag. MUST be `false`
    /// (panel-required negative #8 part 3).
    pub inner_host_transcript_present: bool,
    /// `layer_a_bandwidth_measurement_hash_v1`.
    pub layer_a_bandwidth_measurement_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// BandwidthClaimAdmissionV1
// ---------------------------------------------------------------

/// The verdict on whether the measurement admits the claim
/// it implies. Carries the claim kind + a panel-locked
/// admissibility reason wire name + a boolean admittance
/// flag.
#[derive(Debug, Clone)]
pub struct BandwidthClaimAdmissionV1 {
    /// Which kind of claim the measurement implies.
    pub claim_kind: BandwidthClaimKind,
    /// Panel-locked wire name explaining why the claim was
    /// (or was not) admitted (see the
    /// `ADMISSIBILITY_REASON_*` constants).
    pub admissibility_reason_wire_name: &'static str,
    /// `true` if the claim is admissible under the S-PERF.1
    /// + S-PERF.2 + S-PERF.3 + S-PERF.4 rules.
    pub admitted: bool,
    /// `bandwidth_claim_admission_hash_v1`.
    pub bandwidth_claim_admission_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// EffectiveBandwidthReportV1
// ---------------------------------------------------------------

/// The top-level S-PERF.5 report. Binds the measurement + the
/// admission + the four upstream anchor hashes (S-PERF.1
/// DeviceTrafficReceipt + S-PERF.2 LayerATrafficReceipt +
/// S-PERF.3 PublicDataSaturationBundle + S-PERF.4
/// FamilyCompactionBenchmarkSchema).
#[derive(Debug, Clone)]
pub struct EffectiveBandwidthReportV1 {
    /// Human-readable report identifier (non-empty).
    pub report_id: &'static str,
    /// Wrapped measurement.
    pub measurement: LayerABandwidthMeasurementV1,
    /// Wrapped admission verdict.
    pub admission: BandwidthClaimAdmissionV1,
    /// S-PERF.2 LayerA traffic receipt hash (panel-required
    /// negative #2).
    pub layer_a_traffic_receipt_hash: [u8; 32],
    /// S-PERF.3 public-data saturation bundle hash (panel-
    /// required negatives #3 and #10).
    pub public_data_bundle_hash: [u8; 32],
    /// S-PERF.4 family compaction benchmark schema hash
    /// (panel-required negative #4).
    pub family_compaction_benchmark_schema_hash: [u8; 32],
    /// `effective_bandwidth_report_hash_v1`.
    pub effective_bandwidth_report_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.5 rejected a report or measurement. Ten panel-
/// required load-bearing negatives plus structural defect
/// rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf5VerifyErrorKind {
    /// Panel-required negative #1. The measurement has
    /// `device_traffic_receipt_hash_v1 == [0; 32]` (no
    /// S-PERF.1 receipt cited).
    ReportWithoutSPerf1Receipt,
    /// Panel-required negative #2. The report has
    /// `layer_a_traffic_receipt_hash == [0; 32]`.
    ReportWithoutSPerf2LayerAReceipt,
    /// Panel-required negative #3. The report has
    /// `public_data_bundle_hash == [0; 32]`.
    ReportWithoutSPerf3BundleHash,
    /// Panel-required negative #4. The report has
    /// `family_compaction_benchmark_schema_hash == [0; 32]`.
    ReportWithoutSPerf4CompactionHash,
    /// Panel-required negative #5. The admission declares
    /// `claim_kind == Saturation` but the measurement's
    /// `percent_of_peak_basis_points` is below the panel-
    /// locked saturation threshold
    /// ([`crate::s_perf_1_device_traffic_receipt::S_PERF_1_SATURATION_BP`]
    /// = 8000 bp).
    SaturationClaimBelow8000Bp {
        /// Observed basis-points value.
        observed_bp: u32,
    },
    /// Panel-required negative #6. The admission declares
    /// `claim_kind == Saturation` but the measurement's
    /// `timing_method_wire_name` is neither `CudaEvent` nor
    /// `CudaStreamSync`.
    SaturationClaimWithHostTiming {
        /// The (insufficient) timing method observed.
        observed_timing_method_wire_name: &'static str,
    },
    /// Panel-required negative #7. The measurement's
    /// `effective_bandwidth_gbps` does not equal the value
    /// computed from `total_accounted_device_bytes` and
    /// `measured_kernel_time_us` (or
    /// `percent_of_peak_basis_points` does not equal the
    /// value computed from
    /// `effective_bandwidth_gbps` and
    /// `theoretical_memory_bandwidth_gbps`).
    EffectiveBandwidthMismatchFromBytesAndTime {
        /// Which derived field disagreed.
        which_field_wire_name: &'static str,
        /// What the measurement claimed.
        claimed: u64,
        /// What the verifier's integer arithmetic computed.
        computed: u64,
    },
    /// Panel-required negative #8. The measurement's inner-
    /// forbidden-flag mirror declares one or more of
    /// `inner_host_json_emission_present`,
    /// `inner_casefile_materialization_present`,
    /// `inner_host_transcript_present` as `true`.
    ReportThatIncludesHostJsonOrCasefileTime {
        /// Which forbidden flag was set.
        flag_wire_name: &'static str,
    },
    /// Panel-required negative #9. The admission carries a
    /// non-`NoClaim` claim_kind but the measurement's
    /// `device_uuid_or_identity_hash` is `[0; 32]` (no device
    /// identity to back the cross-device claim).
    CrossDeviceClaimWithoutDeviceIdentity,
    /// Panel-required negative #10. The admission carries a
    /// non-`NoClaim` claim_kind but the report's
    /// `public_data_bundle_hash` is `[0; 32]` (the
    /// benchmark-shaped claim is not backed by a public
    /// artifact manifest bundle).
    BenchmarkClaimWithoutPublicArtifactManifest,
    /// Structural defect: `report_id` is empty.
    ReportIdEmpty,
    /// Structural defect: `admissibility_reason_wire_name` is
    /// empty.
    AdmissibilityReasonEmpty,
    /// Structural defect: free-text field contains a
    /// forbidden benchmark-claim substring (case-insensitive
    /// scan).
    BenchmarkClaimInsideReport {
        /// Schema field where the violation appeared.
        location: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Structural defect: claim_kind and
    /// percent_of_peak_basis_points are incoherent
    /// (e.g. `NoClaim` with non-zero basis-points;
    /// `PercentOfPeak` with basis-points outside `(0,
    /// 8000)`; `EffectiveBandwidth` with zero effective
    /// bandwidth).
    ClaimKindIncoherentWithMeasurement {
        /// The claim kind that failed coherence.
        claim_kind_wire_name: &'static str,
        /// Observed percent-of-peak basis points.
        observed_bp: u32,
        /// Observed effective bandwidth GB/s.
        observed_effective_bandwidth_gbps: u32,
    },
    /// Structural defect: admission's `admitted` flag is
    /// `false` but the verifier finds no panel-required
    /// negative rule violated. The admission is itself
    /// internally inconsistent (a rejected verdict without a
    /// reason).
    InadmissibleClaimWithoutVerifierReason,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf5VerifyError {
    /// Error kind (see [`SPerf5VerifyErrorKind`]).
    pub kind: SPerf5VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build a [`LayerABandwidthMeasurementV1`] and populate
/// `layer_a_bandwidth_measurement_hash_v1`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_layer_a_bandwidth_measurement(
    device_traffic_receipt_hash_v1: [u8; 32],
    device_uuid_or_identity_hash: [u8; 32],
    theoretical_memory_bandwidth_gbps: u32,
    measured_kernel_time_us: u64,
    timing_method_wire_name: &'static str,
    total_accounted_device_bytes: u64,
    effective_bandwidth_gbps: u32,
    percent_of_peak_basis_points: u32,
    inner_host_json_emission_present: bool,
    inner_casefile_materialization_present: bool,
    inner_host_transcript_present: bool,
) -> LayerABandwidthMeasurementV1 {
    let mut m = LayerABandwidthMeasurementV1 {
        device_traffic_receipt_hash_v1,
        device_uuid_or_identity_hash,
        theoretical_memory_bandwidth_gbps,
        measured_kernel_time_us,
        timing_method_wire_name,
        total_accounted_device_bytes,
        effective_bandwidth_gbps,
        percent_of_peak_basis_points,
        inner_host_json_emission_present,
        inner_casefile_materialization_present,
        inner_host_transcript_present,
        layer_a_bandwidth_measurement_hash_v1: [0u8; 32],
    };
    m.layer_a_bandwidth_measurement_hash_v1 = compute_layer_a_bandwidth_measurement_hash(&m);
    m
}

/// Build a [`BandwidthClaimAdmissionV1`] and populate
/// `bandwidth_claim_admission_hash_v1`.
#[must_use]
pub fn build_bandwidth_claim_admission(
    claim_kind: BandwidthClaimKind,
    admissibility_reason_wire_name: &'static str,
    admitted: bool,
) -> BandwidthClaimAdmissionV1 {
    let mut a = BandwidthClaimAdmissionV1 {
        claim_kind,
        admissibility_reason_wire_name,
        admitted,
        bandwidth_claim_admission_hash_v1: [0u8; 32],
    };
    a.bandwidth_claim_admission_hash_v1 = compute_bandwidth_claim_admission_hash(&a);
    a
}

/// Build an [`EffectiveBandwidthReportV1`] and populate
/// `effective_bandwidth_report_hash_v1`.
#[must_use]
pub fn build_effective_bandwidth_report(
    report_id: &'static str,
    measurement: LayerABandwidthMeasurementV1,
    admission: BandwidthClaimAdmissionV1,
    layer_a_traffic_receipt_hash: [u8; 32],
    public_data_bundle_hash: [u8; 32],
    family_compaction_benchmark_schema_hash: [u8; 32],
) -> EffectiveBandwidthReportV1 {
    let mut r = EffectiveBandwidthReportV1 {
        report_id,
        measurement,
        admission,
        layer_a_traffic_receipt_hash,
        public_data_bundle_hash,
        family_compaction_benchmark_schema_hash,
        effective_bandwidth_report_hash_v1: [0u8; 32],
    };
    r.effective_bandwidth_report_hash_v1 = compute_effective_bandwidth_report_hash(&r);
    r
}

// ---------------------------------------------------------------
// Seed (uninstrumented baseline; references the live S-PERF.1
// / S-PERF.2 / S-PERF.3 / S-PERF.4 baselines so the chain
// cannot drift from production state)
// ---------------------------------------------------------------

/// Build the panel-locked uninstrumented baseline measurement.
/// Mirrors the S-PERF.1 baseline DeviceTrafficReceipt: every
/// measurement field zero, but the device identity + timing
/// method + theoretical peak are pulled verbatim from the
/// inner receipt. The LayerA forbidden-flag mirror is taken
/// from the S-PERF.2 baseline pipeline (all false).
#[must_use]
pub fn seed_baseline_layer_a_bandwidth_measurement() -> LayerABandwidthMeasurementV1 {
    let inner = seed_baseline_uninstrumented_receipt();
    let layer_a_traffic = seed_baseline_layer_a_traffic_receipt();
    let pipeline = &layer_a_traffic.pipeline;
    build_layer_a_bandwidth_measurement(
        inner.device_traffic_receipt_hash_v1,
        inner.device_uuid_or_identity_hash,
        inner.theoretical_memory_bandwidth_gbps,
        inner.measured_kernel_time_us,
        inner.timing_method.as_str(),
        inner.total_accounted_device_bytes,
        inner.effective_bandwidth_gbps,
        inner.percent_of_peak_basis_points,
        pipeline.host_json_emission_present,
        pipeline.casefile_materialization_present,
        pipeline.host_transcript_present,
    )
}

/// Build the panel-locked baseline admission verdict
/// (`NoClaim` since the baseline is uninstrumented).
#[must_use]
pub fn seed_baseline_bandwidth_claim_admission() -> BandwidthClaimAdmissionV1 {
    build_bandwidth_claim_admission(
        BandwidthClaimKind::NoClaim,
        ADMISSIBILITY_REASON_ADMISSIBLE_NO_CLAIM,
        true,
    )
}

/// Build the panel-locked uninstrumented baseline report.
/// Composes the baseline measurement + the baseline admission
/// + the live S-PERF.2 / S-PERF.3 / S-PERF.4 baseline hashes.
#[must_use]
pub fn seed_baseline_effective_bandwidth_report() -> EffectiveBandwidthReportV1 {
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let layer_a_traffic = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    let compaction = seed_baseline_family_compaction_benchmark_schema();
    build_effective_bandwidth_report(
        "s_perf_5_baseline_effective_bandwidth_report_v1",
        measurement,
        admission,
        layer_a_traffic.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
        compaction.family_compaction_benchmark_schema_hash_v1,
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_layer_a_bandwidth_measurement_hash(m: &LayerABandwidthMeasurementV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(LAYER_A_BANDWIDTH_MEASUREMENT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(LAYER_A_BANDWIDTH_MEASUREMENT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&m.device_traffic_receipt_hash_v1);
    buf.extend_from_slice(&m.device_uuid_or_identity_hash);
    buf.extend_from_slice(&m.theoretical_memory_bandwidth_gbps.to_be_bytes());
    buf.extend_from_slice(&m.measured_kernel_time_us.to_be_bytes());
    push_len_prefixed(&mut buf, m.timing_method_wire_name.as_bytes());
    buf.extend_from_slice(&m.total_accounted_device_bytes.to_be_bytes());
    buf.extend_from_slice(&m.effective_bandwidth_gbps.to_be_bytes());
    buf.extend_from_slice(&m.percent_of_peak_basis_points.to_be_bytes());
    buf.push(u8::from(m.inner_host_json_emission_present));
    buf.push(u8::from(m.inner_casefile_materialization_present));
    buf.push(u8::from(m.inner_host_transcript_present));
    sha256(&buf)
}

fn compute_bandwidth_claim_admission_hash(a: &BandwidthClaimAdmissionV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(BANDWIDTH_CLAIM_ADMISSION_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(BANDWIDTH_CLAIM_ADMISSION_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, a.claim_kind.as_str().as_bytes());
    push_len_prefixed(&mut buf, a.admissibility_reason_wire_name.as_bytes());
    buf.push(u8::from(a.admitted));
    sha256(&buf)
}

fn compute_effective_bandwidth_report_hash(r: &EffectiveBandwidthReportV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(EFFECTIVE_BANDWIDTH_REPORT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(EFFECTIVE_BANDWIDTH_REPORT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.report_id.as_bytes());
    buf.extend_from_slice(&r.measurement.layer_a_bandwidth_measurement_hash_v1);
    buf.extend_from_slice(&r.admission.bandwidth_claim_admission_hash_v1);
    buf.extend_from_slice(&r.layer_a_traffic_receipt_hash);
    buf.extend_from_slice(&r.public_data_bundle_hash);
    buf.extend_from_slice(&r.family_compaction_benchmark_schema_hash);
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

/// Verify an effective-bandwidth report. Returns the list of
/// errors (empty when the report satisfies every panel-
/// required + structural rule).
#[must_use]
#[allow(clippy::too_many_lines)] // 10 panel-required negatives + structural
pub fn verify_effective_bandwidth_report(
    report: &EffectiveBandwidthReportV1,
) -> Vec<SPerf5VerifyError> {
    let mut errors: Vec<SPerf5VerifyError> = Vec::new();
    let m = &report.measurement;
    let a = &report.admission;

    // Panel-required negative #1: missing S-PERF.1 receipt.
    if m.device_traffic_receipt_hash_v1 == [0u8; 32] {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportWithoutSPerf1Receipt,
        });
    }
    // Panel-required negative #2: missing S-PERF.2 receipt.
    if report.layer_a_traffic_receipt_hash == [0u8; 32] {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportWithoutSPerf2LayerAReceipt,
        });
    }
    // Panel-required negative #3: missing S-PERF.3 bundle hash.
    if report.public_data_bundle_hash == [0u8; 32] {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportWithoutSPerf3BundleHash,
        });
    }
    // Panel-required negative #4: missing S-PERF.4 compaction hash.
    if report.family_compaction_benchmark_schema_hash == [0u8; 32] {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportWithoutSPerf4CompactionHash,
        });
    }

    // Panel-required negative #5: Saturation claim below
    // threshold.
    if matches!(a.claim_kind, BandwidthClaimKind::Saturation)
        && m.percent_of_peak_basis_points < S_PERF_1_SATURATION_BP
    {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::SaturationClaimBelow8000Bp {
                observed_bp: m.percent_of_peak_basis_points,
            },
        });
    }

    // Panel-required negative #6: Saturation claim with host
    // timing.
    if matches!(a.claim_kind, BandwidthClaimKind::Saturation)
        && m.timing_method_wire_name != "CudaEvent"
        && m.timing_method_wire_name != "CudaStreamSync"
    {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::SaturationClaimWithHostTiming {
                observed_timing_method_wire_name: m.timing_method_wire_name,
            },
        });
    }

    // Panel-required negative #7: effective bandwidth / percent
    // -of-peak arithmetic coherence.
    let expected_eff_bw = if m.measured_kernel_time_us == 0 {
        0u32
    } else {
        // bandwidth_gbps = bytes / (time_us * 1000)
        let denom = m.measured_kernel_time_us.saturating_mul(1000);
        if denom == 0 {
            0u32
        } else {
            u32::try_from(m.total_accounted_device_bytes / denom).unwrap_or(u32::MAX)
        }
    };
    if m.effective_bandwidth_gbps != expected_eff_bw {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::EffectiveBandwidthMismatchFromBytesAndTime {
                which_field_wire_name: "effective_bandwidth_gbps",
                claimed: u64::from(m.effective_bandwidth_gbps),
                computed: u64::from(expected_eff_bw),
            },
        });
    }
    let expected_pct_bp = if m.theoretical_memory_bandwidth_gbps == 0 {
        0u32
    } else {
        m.effective_bandwidth_gbps.saturating_mul(10_000) / m.theoretical_memory_bandwidth_gbps
    };
    if m.percent_of_peak_basis_points != expected_pct_bp {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::EffectiveBandwidthMismatchFromBytesAndTime {
                which_field_wire_name: "percent_of_peak_basis_points",
                claimed: u64::from(m.percent_of_peak_basis_points),
                computed: u64::from(expected_pct_bp),
            },
        });
    }

    // Panel-required negative #8: host JSON / casefile /
    // transcript flag set on the inner LayerA receipt.
    if m.inner_host_json_emission_present {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name: "inner_host_json_emission_present",
            },
        });
    }
    if m.inner_casefile_materialization_present {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name: "inner_casefile_materialization_present",
            },
        });
    }
    if m.inner_host_transcript_present {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportThatIncludesHostJsonOrCasefileTime {
                flag_wire_name: "inner_host_transcript_present",
            },
        });
    }

    // Panel-required negative #9: non-NoClaim with zero device
    // identity.
    if !matches!(a.claim_kind, BandwidthClaimKind::NoClaim)
        && m.device_uuid_or_identity_hash == [0u8; 32]
    {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::CrossDeviceClaimWithoutDeviceIdentity,
        });
    }

    // Panel-required negative #10: non-NoClaim with zero
    // public data bundle hash. Note: this overlaps with
    // negative #3 (zero bundle hash) but the panel-required
    // negative explicitly ties the rule to the presence of a
    // non-trivial claim. We emit both errors if both fire so
    // the operator sees the dual failure.
    if !matches!(a.claim_kind, BandwidthClaimKind::NoClaim)
        && report.public_data_bundle_hash == [0u8; 32]
    {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::BenchmarkClaimWithoutPublicArtifactManifest,
        });
    }

    // Structural: empty report id.
    if report.report_id.is_empty() {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ReportIdEmpty,
        });
    }
    // Structural: empty admissibility reason.
    if a.admissibility_reason_wire_name.is_empty() {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::AdmissibilityReasonEmpty,
        });
    }
    // Structural: case-insensitive scan over free-text fields.
    scan_for_forbidden_substring(report.report_id, "report_id", &mut errors);
    scan_for_forbidden_substring(
        a.admissibility_reason_wire_name,
        "admissibility_reason_wire_name",
        &mut errors,
    );

    // Structural: claim_kind / measurement coherence.
    let coherence_violation = match a.claim_kind {
        BandwidthClaimKind::NoClaim => {
            m.percent_of_peak_basis_points != 0 || m.effective_bandwidth_gbps != 0
        }
        BandwidthClaimKind::EffectiveBandwidth => m.effective_bandwidth_gbps == 0,
        BandwidthClaimKind::PercentOfPeak => {
            m.percent_of_peak_basis_points == 0
                || m.percent_of_peak_basis_points >= S_PERF_1_SATURATION_BP
        }
        BandwidthClaimKind::Saturation => m.percent_of_peak_basis_points < S_PERF_1_SATURATION_BP,
    };
    if coherence_violation {
        errors.push(SPerf5VerifyError {
            kind: SPerf5VerifyErrorKind::ClaimKindIncoherentWithMeasurement {
                claim_kind_wire_name: a.claim_kind.as_str(),
                observed_bp: m.percent_of_peak_basis_points,
                observed_effective_bandwidth_gbps: m.effective_bandwidth_gbps,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn scan_for_forbidden_substring(
    text: &'static str,
    location: &'static str,
    errors: &mut Vec<SPerf5VerifyError>,
) {
    for &forbidden in S_PERF_5_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS {
        if contains_ascii_case_insensitive(text, forbidden) {
            errors.push(SPerf5VerifyError {
                kind: SPerf5VerifyErrorKind::BenchmarkClaimInsideReport {
                    location,
                    forbidden_substring: forbidden,
                },
            });
        }
    }
}

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
// Renderers --- text
// ---------------------------------------------------------------

/// Render the measurement as deterministic text.
#[must_use]
pub fn render_layer_a_bandwidth_measurement_text(m: &LayerABandwidthMeasurementV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.5 LayerABandwidthMeasurementV1");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Inner receipt references");
    let _ = writeln!(
        s,
        "  device_traffic_receipt_hash_v1    : {}",
        hex32(&m.device_traffic_receipt_hash_v1)
    );
    let _ = writeln!(
        s,
        "  device_uuid_or_identity_hash      : {}",
        hex32(&m.device_uuid_or_identity_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Measurement");
    let _ = writeln!(
        s,
        "  theoretical_memory_bandwidth_gbps : {}",
        m.theoretical_memory_bandwidth_gbps
    );
    let _ = writeln!(
        s,
        "  measured_kernel_time_us           : {}",
        m.measured_kernel_time_us
    );
    let _ = writeln!(
        s,
        "  timing_method_wire_name           : {}",
        m.timing_method_wire_name
    );
    let _ = writeln!(
        s,
        "  total_accounted_device_bytes      : {}",
        m.total_accounted_device_bytes
    );
    let _ = writeln!(
        s,
        "  effective_bandwidth_gbps          : {}",
        m.effective_bandwidth_gbps
    );
    let _ = writeln!(
        s,
        "  percent_of_peak_basis_points      : {}",
        m.percent_of_peak_basis_points
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Inner LayerA forbidden-flag mirror");
    let _ = writeln!(
        s,
        "  inner_host_json_emission_present       : {}",
        m.inner_host_json_emission_present
    );
    let _ = writeln!(
        s,
        "  inner_casefile_materialization_present : {}",
        m.inner_casefile_materialization_present
    );
    let _ = writeln!(
        s,
        "  inner_host_transcript_present          : {}",
        m.inner_host_transcript_present
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "layer_a_bandwidth_measurement_hash_v1 : {}",
        hex32(&m.layer_a_bandwidth_measurement_hash_v1)
    );
    s
}

/// Render the admission verdict as deterministic text.
#[must_use]
pub fn render_bandwidth_claim_admission_text(a: &BandwidthClaimAdmissionV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.5 BandwidthClaimAdmissionV1");
    let _ = writeln!(s, "==================================");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "  claim_kind                       : {}",
        a.claim_kind.as_str()
    );
    let _ = writeln!(
        s,
        "  admissibility_reason_wire_name   : {}",
        a.admissibility_reason_wire_name
    );
    let _ = writeln!(s, "  admitted                         : {}", a.admitted);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "bandwidth_claim_admission_hash_v1 : {}",
        hex32(&a.bandwidth_claim_admission_hash_v1)
    );
    s
}

/// Render the report as deterministic text.
#[must_use]
pub fn render_effective_bandwidth_report_text(r: &EffectiveBandwidthReportV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.5 EffectiveBandwidthReportV1");
    let _ = writeln!(s, "===================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Identity");
    let _ = writeln!(s, "  report_id : {}", r.report_id);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Bound hashes (4 upstream anchors + measurement + admission)"
    );
    let _ = writeln!(
        s,
        "  device_traffic_receipt_hash_v1            : {}",
        hex32(&r.measurement.device_traffic_receipt_hash_v1)
    );
    let _ = writeln!(
        s,
        "  layer_a_traffic_receipt_hash              : {}",
        hex32(&r.layer_a_traffic_receipt_hash)
    );
    let _ = writeln!(
        s,
        "  public_data_bundle_hash                   : {}",
        hex32(&r.public_data_bundle_hash)
    );
    let _ = writeln!(
        s,
        "  family_compaction_benchmark_schema_hash   : {}",
        hex32(&r.family_compaction_benchmark_schema_hash)
    );
    let _ = writeln!(
        s,
        "  layer_a_bandwidth_measurement_hash_v1     : {}",
        hex32(&r.measurement.layer_a_bandwidth_measurement_hash_v1)
    );
    let _ = writeln!(
        s,
        "  bandwidth_claim_admission_hash_v1         : {}",
        hex32(&r.admission.bandwidth_claim_admission_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Claim");
    let _ = writeln!(s, "  claim_kind  : {}", r.admission.claim_kind.as_str());
    let _ = writeln!(s, "  admitted    : {}", r.admission.admitted);
    let _ = writeln!(
        s,
        "  reason      : {}",
        r.admission.admissibility_reason_wire_name
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "effective_bandwidth_report_hash_v1 : {}",
        hex32(&r.effective_bandwidth_report_hash_v1)
    );
    s
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render the measurement as canonical JSON.
#[must_use]
pub fn render_layer_a_bandwidth_measurement_json(m: &LayerABandwidthMeasurementV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", LAYER_A_BANDWIDTH_MEASUREMENT_SCHEMA_V1);
    s.push(',');
    json_hex(
        &mut s,
        "device_traffic_receipt_hash_v1",
        &m.device_traffic_receipt_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "device_uuid_or_identity_hash",
        &m.device_uuid_or_identity_hash,
    );
    s.push(',');
    let _ = write!(
        s,
        "\"theoretical_memory_bandwidth_gbps\":{}",
        m.theoretical_memory_bandwidth_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"measured_kernel_time_us\":{}",
        m.measured_kernel_time_us
    );
    s.push(',');
    json_string(&mut s, "timing_method_wire_name", m.timing_method_wire_name);
    s.push(',');
    let _ = write!(
        s,
        "\"total_accounted_device_bytes\":{}",
        m.total_accounted_device_bytes
    );
    s.push(',');
    let _ = write!(
        s,
        "\"effective_bandwidth_gbps\":{}",
        m.effective_bandwidth_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"percent_of_peak_basis_points\":{}",
        m.percent_of_peak_basis_points
    );
    s.push(',');
    let _ = write!(
        s,
        "\"inner_host_json_emission_present\":{}",
        m.inner_host_json_emission_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"inner_casefile_materialization_present\":{}",
        m.inner_casefile_materialization_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"inner_host_transcript_present\":{}",
        m.inner_host_transcript_present
    );
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_bandwidth_measurement_hash_v1",
        &m.layer_a_bandwidth_measurement_hash_v1,
    );
    s.push('}');
    s
}

/// Render the admission verdict as canonical JSON.
#[must_use]
pub fn render_bandwidth_claim_admission_json(a: &BandwidthClaimAdmissionV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", BANDWIDTH_CLAIM_ADMISSION_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "claim_kind", a.claim_kind.as_str());
    s.push(',');
    json_string(
        &mut s,
        "admissibility_reason_wire_name",
        a.admissibility_reason_wire_name,
    );
    s.push(',');
    let _ = write!(s, "\"admitted\":{}", a.admitted);
    s.push(',');
    json_hex(
        &mut s,
        "bandwidth_claim_admission_hash_v1",
        &a.bandwidth_claim_admission_hash_v1,
    );
    s.push('}');
    s
}

/// Render the report as canonical JSON.
#[must_use]
pub fn render_effective_bandwidth_report_json(r: &EffectiveBandwidthReportV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", EFFECTIVE_BANDWIDTH_REPORT_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "report_id", r.report_id);
    s.push(',');
    json_hex(
        &mut s,
        "device_traffic_receipt_hash_v1",
        &r.measurement.device_traffic_receipt_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_traffic_receipt_hash",
        &r.layer_a_traffic_receipt_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "public_data_bundle_hash",
        &r.public_data_bundle_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "family_compaction_benchmark_schema_hash",
        &r.family_compaction_benchmark_schema_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_bandwidth_measurement_hash_v1",
        &r.measurement.layer_a_bandwidth_measurement_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "bandwidth_claim_admission_hash_v1",
        &r.admission.bandwidth_claim_admission_hash_v1,
    );
    s.push(',');
    json_string(&mut s, "claim_kind", r.admission.claim_kind.as_str());
    s.push(',');
    let _ = write!(s, "\"admitted\":{}", r.admission.admitted);
    s.push(',');
    json_hex(
        &mut s,
        "effective_bandwidth_report_hash_v1",
        &r.effective_bandwidth_report_hash_v1,
    );
    s.push('}');
    s
}

fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_string(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_hex(s: &mut String, key: &str, value: &[u8; 32]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    let _ = s.write_str(&hex32(value));
    s.push('"');
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Test-only read-only access to the forbidden benchmark-
/// claim substring set.
#[doc(hidden)]
#[must_use]
pub fn forbidden_benchmark_claim_substrings() -> &'static [&'static str] {
    S_PERF_5_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS
}
