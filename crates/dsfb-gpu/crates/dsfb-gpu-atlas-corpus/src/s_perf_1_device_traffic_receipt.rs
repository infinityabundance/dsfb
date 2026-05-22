//! S-PERF.1 --- `DeviceTrafficReceiptV1`.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S-PERF.1 defines the byte-accounting receipt required
//! > before DSFB-GPU can make any serious memory-bandwidth or
//! > saturation claim. It does not claim bandwidth saturation,
//! > does not benchmark B300 / GB300, does not change CUDA
//! > kernels, and does not alter court authority. It creates
//! > the measurement law.**
//!
//! Core rule (panel-locked, 8 lines; see
//! [`S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES`]):
//!
//! 1. No bandwidth claim without byte accounting.
//! 2. No peak-percentage claim without declared device bandwidth.
//! 3. No CUDA timing claim without CUDA event timing.
//! 4. No Layer-A claim if host JSON / report time is included.
//! 5. No cross-device comparison without device identity.
//! 6. No effective bandwidth when total accounted bytes equals
//!    zero.
//! 7. No percent-of-peak above 100 without explicit error flag.
//! 8. Every receipt MUST declare contract hashes.
//!
//! ## Why
//!
//! After T.12.PROV made the science creditable, S-PERF.1 makes
//! future CUDA performance claims accountable. The Atlas paper
//! and README will eventually carry effective-bandwidth + peak-
//! percentage numbers; without a measurement law the first
//! such number can over-claim by accident (or by selection of
//! the timing method). S-PERF.1 pins the receipt shape AND the
//! verifier so every later performance commit MUST either
//! satisfy the rules or surface an explicit error flag.
//!
//! ## Hash posture
//!
//! Two new own-namespace hashes (none folded upstream):
//!
//! - `device_traffic_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEVICE-TRAFFIC-RECEIPT:v1\0`. Pins the
//!   bytes of one measurement receipt.
//! - `device_bandwidth_claim_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:DEVICE-BANDWIDTH-CLAIM-POLICY:v1\0`. Pins
//!   the 8-line panel-locked policy doctrine.
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.1 does NOT:
//!
//! - claim that DSFB-GPU has measured peak memory-bandwidth
//!   saturation on any GPU;
//! - claim production CUDA performance numbers;
//! - benchmark B300 / GB300 cloud hardware (that is the
//!   S-PERF.7 / S-MG.6 victory-lap commit, gated on the rest
//!   of S-PERF + S-MG landing);
//! - change any CUDA kernel;
//! - change any court decision (S1.3a / FF.2 / FF.3 / S1.3d /
//!   S1.3e / S1.3f / S1.3g);
//! - mutate any upstream hash anchor (`corpus_hash_v1`,
//!   `corpus_hash_v2`, T.11.* / T.12.* / FF.* / S1.3.* hashes
//!   all byte-stable);
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs / witness records / fusion tensors /
//!   candidate intervals / episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate.
//!
//! S-PERF.1 ships ONLY the receipt schema + policy +
//! verifier + builder + renderers. The first actual
//! measurement against this receipt is a separate S-PERF.2+
//! commit gated on the Layer-A device-resident densor pipeline
//! work.
//!
//! ## Panel-locked one-line verdict
//!
//! > T.12.PROV made the science creditable; S-PERF.1 makes
//! > future CUDA performance claims accountable.

use core::fmt::Write;
use std::collections::BTreeSet;

use dsfb_gpu_debug_core::sha256;

use crate::corpus_hash::compute_corpus_hash_v1;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `device_traffic_receipt_hash_v1`. The
/// trailing `\0` byte is panel-locked: it ensures the receipt
/// hash cannot be silently absorbed into a sibling
/// domain-separator string by careless concatenation.
pub const DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:DEVICE-TRAFFIC-RECEIPT:v1\0";

/// Schema identifier for `device_traffic_receipt_hash_v1`
/// (recorded inside the hash buffer; mirrors the domain
/// separator without the trailing NUL byte for human-readable
/// pretty-printing).
pub const DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:DEVICE-TRAFFIC-RECEIPT:v1";

/// Domain separator for `device_bandwidth_claim_policy_hash_v1`.
/// Same NUL-terminated discipline as the receipt domain.
pub const DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:DEVICE-BANDWIDTH-CLAIM-POLICY:v1\0";

/// Schema identifier for `device_bandwidth_claim_policy_hash_v1`.
pub const DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:DEVICE-BANDWIDTH-CLAIM-POLICY:v1";

/// Domain separator for the device identity hash (used when a
/// caller wants a deterministic stand-in for `cudaDeviceGetUuid`
/// in environments where the real UUID is not available, e.g.
/// when the receipt is constructed before any CUDA runtime call).
pub const DEVICE_IDENTITY_HASH_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:DEVICE-IDENTITY:v1\0";

// ---------------------------------------------------------------
// Panel-locked bandwidth claim policy lines
// ---------------------------------------------------------------

/// The eight-line panel-locked bandwidth-claim policy. The
/// verifier in [`verify_device_traffic_receipt`] enforces one
/// rule per line (in order); the policy renderer prints these
/// verbatim so the contract is human-readable AND machine-
/// hashable.
///
/// **Do not reorder, edit, or extend without rebaselining**
/// `device_bandwidth_claim_policy_hash_v1`. The hash is
/// canonical-byte over these lines exactly.
pub const S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES: &[&str] = &[
    "No bandwidth claim without byte accounting.",
    "No peak-percentage claim without declared device bandwidth.",
    "No CUDA timing claim without CUDA event timing.",
    "No Layer-A claim if host JSON / report time is included.",
    "No cross-device comparison without device identity.",
    "No effective bandwidth when total accounted bytes equals zero.",
    "No percent-of-peak above 100 without explicit error flag.",
    "Every receipt MUST declare contract hashes.",
];

/// Saturation-claim threshold. The verifier treats any receipt
/// with `percent_of_peak_basis_points >= S_PERF_1_SATURATION_BP`
/// (i.e. >= 80 % of theoretical bandwidth) as a saturation claim.
/// Saturation claims MUST be backed by `TimingMethod::CudaEvent`
/// (or `CudaStreamSync`); anything looser fires the panel-locked
/// negative
/// [`SPerf1VerifyErrorKind::SaturationClaimWithoutCudaEventTiming`].
///
/// 8 000 basis points = 80.00 %. Panel-locked: changing this
/// constant changes the saturation policy contract and rebaselines
/// the policy hash; do not adjust without an explicit commit
/// noting the rationale.
pub const S_PERF_1_SATURATION_BP: u32 = 8_000;

// ---------------------------------------------------------------
// TimingMethod
// ---------------------------------------------------------------

/// How the receipt's `measured_kernel_time_us` was obtained.
/// The choice of timing method governs which bandwidth claims
/// are admissible: Layer-A claims require device-resident
/// timing; saturation claims require CUDA event timing.
///
/// Wire names are stable for the hash buffer; do not rename
/// without rebaselining `device_traffic_receipt_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingMethod {
    /// `cudaEventRecord` + `cudaEventElapsedTime` around the
    /// kernel sequence. The strictest option; required for any
    /// saturation claim.
    CudaEvent,
    /// `cudaStreamSynchronize` + host clock around the stream.
    /// Includes some launch overhead but stays device-side.
    CudaStreamSync,
    /// `std::time::Instant` around the FFI call only (no
    /// host post-processing). Acceptable for Layer-B; not
    /// acceptable for Layer-A saturation claims.
    HostInstantOnly,
    /// Host clock that included JSON serialisation, case-file
    /// finalisation, or other host orchestration. The
    /// panel-locked Layer-A trip wire: any Layer-A receipt
    /// using this timing method is rejected.
    HostJsonInclusiveTime,
    /// Timing method unknown or unspecified. Always rejected
    /// for Layer-A claims; rejected for any saturation claim;
    /// rejected when the receipt records a non-zero
    /// `measured_kernel_time_us`.
    Unknown,
}

impl TimingMethod {
    /// Canonical wire name for the hash buffer + renderers.
    /// Mirrors the variant name. Stable across releases.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CudaEvent => "CudaEvent",
            Self::CudaStreamSync => "CudaStreamSync",
            Self::HostInstantOnly => "HostInstantOnly",
            Self::HostJsonInclusiveTime => "HostJsonInclusiveTime",
            Self::Unknown => "Unknown",
        }
    }

    /// True iff this timing method is admissible as backing
    /// for a Layer-A claim (`CudaEvent` or `CudaStreamSync`).
    /// Used by the verifier's negative #4 (saturation) and as
    /// part of negative #3 (Layer-A trip wire).
    #[must_use]
    pub const fn is_device_resident(self) -> bool {
        matches!(self, Self::CudaEvent | Self::CudaStreamSync)
    }
}

// ---------------------------------------------------------------
// DeviceBandwidthLayer
// ---------------------------------------------------------------

/// Which bandwidth-accounting layer the receipt describes.
/// Layer A = device evidence fabric (kernels + on-device
/// digests only); Layer B = throughput verdict summaries
/// (Layer A + CPU bank admission); Layer C = full audit court
/// (every intermediate cell materialised host-side; canonical
/// JSON case file emitted).
///
/// Stable wire names; do not rename without rebaselining
/// `device_traffic_receipt_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceBandwidthLayer {
    /// Layer A: device-resident evidence fabric. The strictest
    /// timing posture; the layer that should eventually scale
    /// massively under the S-PERF.* campaign.
    LayerA,
    /// Layer B: throughput verdict summaries (Layer A + CPU
    /// bank admission). Acceptable to time with
    /// `HostInstantOnly` because the CPU bank stage runs on
    /// the host by design.
    LayerB,
    /// Layer C: full audit court. Slowest by design; timing
    /// posture is informational (the layer's purpose is
    /// reproducibility, not throughput).
    LayerC,
}

impl DeviceBandwidthLayer {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LayerA => "LayerA",
            Self::LayerB => "LayerB",
            Self::LayerC => "LayerC",
        }
    }
}

// ---------------------------------------------------------------
// DeviceTrafficReceiptV1
// ---------------------------------------------------------------

/// One device-traffic receipt --- the byte-accounting envelope
/// every future memory-bandwidth claim MUST cite.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `device_traffic_receipt_hash_v1`. Two
/// receipts with byte-identical fields produce byte-identical
/// hashes regardless of when or where they were built.
#[derive(Debug, Clone)]
pub struct DeviceTrafficReceiptV1 {
    /// Human-readable device name (e.g. `"RTX 4080 SUPER"`).
    /// Non-empty (structural rule).
    pub device_name: &'static str,
    /// Deterministic device-identity hash. In production, this
    /// is the SHA-256 of `cudaDeviceGetUuid` output; for
    /// receipt construction before any CUDA call, use
    /// [`compute_device_identity_hash`] to derive a stand-in
    /// from `(device_name, sm_arch)`. Non-zero (panel-locked
    /// negative #5).
    pub device_uuid_or_identity_hash: [u8; 32],
    /// Compute capability (e.g. 89 for sm_89). Non-zero
    /// (structural rule).
    pub sm_arch: u32,
    /// NVIDIA driver version string (e.g. `"13.2.0"`).
    /// Non-empty (structural rule).
    pub driver_version: &'static str,
    /// CUDA runtime / toolkit version string (e.g. `"13.2"`).
    /// Non-empty (structural rule).
    pub cuda_version: &'static str,
    /// Theoretical peak memory bandwidth, in GB/s. Pulled from
    /// the vendor datasheet (e.g. 716 GB/s for RTX 4080 SUPER).
    /// Required to be non-zero whenever
    /// `percent_of_peak_basis_points > 0` (panel-locked
    /// negative #2).
    pub theoretical_memory_bandwidth_gbps: u32,
    /// Measured kernel time, in microseconds. Whose definition
    /// of "kernel time" depends on [`TimingMethod`].
    pub measured_kernel_time_us: u64,
    /// How `measured_kernel_time_us` was obtained.
    pub timing_method: TimingMethod,
    /// Which bandwidth-accounting layer this receipt describes.
    pub layer: DeviceBandwidthLayer,
    /// Number of detectors active during the timed run.
    pub detector_count: u32,
    /// Number of catalogs processed during the timed run.
    pub catalog_count: u32,
    /// Bytes of input artifact data fed to the kernel sequence
    /// (compact-event H2D bytes for D64+ throughput dispatch).
    pub input_bytes: u64,
    /// Bytes of evidence-densor (residual) data read by the
    /// kernel sequence.
    pub evidence_bytes_read: u64,
    /// Bytes of evidence-densor data written by the kernel
    /// sequence (e.g. sign field, drift field).
    pub evidence_bytes_written: u64,
    /// Bytes of witness-densor data written by the kernel
    /// sequence (detector cell outputs).
    pub witness_bytes_written: u64,
    /// Bytes of fusion-densor data read AND written by the
    /// fusion stage (consensus + axis fusion).
    pub fusion_bytes_read_written: u64,
    /// Bytes of stage-digest data read during the on-device
    /// digest reduction (R.8.5 tree-digest leaves).
    pub digest_bytes_read: u64,
    /// Bytes of compact candidate-summary D2H output.
    pub candidate_summary_bytes: u64,
    /// Total accounted device bytes. MUST equal the sum of the
    /// seven byte fields above (structural rule
    /// `AccountedBytesMismatchSumOfFields`).
    pub total_accounted_device_bytes: u64,
    /// Effective bandwidth, in GB/s. Computed by the caller
    /// as `total_accounted_device_bytes * 1_000 /
    /// measured_kernel_time_us` (integer GB/s). The verifier
    /// only enforces consistency (zero bytes ⇒ zero
    /// bandwidth; non-zero bandwidth ⇒ non-zero bytes).
    pub effective_bandwidth_gbps: u32,
    /// Percent of theoretical peak, in basis points
    /// (10000 = 100.00 %). Allowing fractional precision lets
    /// reviewers spot any percent-of-peak claim near 100 %
    /// without trusting host-side floating-point. The verifier
    /// rejects any value > 10000 unless
    /// `accounting_overflow_acknowledged` is set.
    pub percent_of_peak_basis_points: u32,
    /// When `true`, the receipt explicitly acknowledges that
    /// `percent_of_peak_basis_points` exceeds 10000 due to
    /// accounting overcounting (e.g. a buffer was traversed
    /// twice in a way the byte fields cannot represent without
    /// double-counting). Required for panel-locked negative #7
    /// to NOT fire.
    pub accounting_overflow_acknowledged: bool,
    /// Hashes of artifacts the timed run was taken against
    /// (e.g. `corpus_hash_v1`, fixture hashes). Optional but
    /// recommended for replay.
    pub artifact_hashes: Vec<[u8; 32]>,
    /// Hashes of the execution-contract identities that
    /// defined the timed run (numeric mode, kernel-plan hash,
    /// registry hash, contract-toml hash, ...). Non-empty
    /// (panel-locked negative #8).
    pub contract_hashes: Vec<[u8; 32]>,
    /// `device_traffic_receipt_hash_v1`. Populated by
    /// [`build_device_traffic_receipt`] from the canonical-byte
    /// projection of every field above.
    pub device_traffic_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// DeviceBandwidthClaimPolicyV1
// ---------------------------------------------------------------

/// The panel-locked bandwidth-claim policy. Pins the 8 rules
/// the verifier enforces and gives the policy its own
/// `device_bandwidth_claim_policy_hash_v1` so the contract is
/// citable as a hash, not just as prose.
///
/// At S-PERF.1 baseline the policy lines equal
/// [`S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES`] verbatim.
#[derive(Debug, Clone)]
pub struct DeviceBandwidthClaimPolicyV1 {
    /// The 8 panel-locked policy lines.
    pub policy_lines: Vec<&'static str>,
    /// `device_bandwidth_claim_policy_hash_v1`.
    pub device_bandwidth_claim_policy_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// DeviceTrafficReceiptComparison
// ---------------------------------------------------------------

/// A cross-device comparison verdict. Carries the receipts
/// being compared plus the verifier verdict. Distinct from
/// [`DeviceTrafficReceiptV1`] so the comparison rule
/// (panel-locked negative #5) can be enforced on the
/// comparison act itself, not on the underlying receipts.
#[derive(Debug, Clone)]
pub struct DeviceTrafficReceiptComparison<'a> {
    /// The receipts being compared (≥ 2 for any meaningful
    /// comparison; the verifier rejects a 0- or 1-receipt
    /// comparison as ill-formed).
    pub receipts: &'a [DeviceTrafficReceiptV1],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.1 rejected a receipt or comparison. Eight
/// panel-required load-bearing negatives plus structural
/// defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf1VerifyErrorKind {
    /// Panel-required negative #1. Any non-zero bandwidth
    /// claim (`percent_of_peak_basis_points > 0` OR
    /// `effective_bandwidth_gbps > 0`) without supporting
    /// byte accounting (`total_accounted_device_bytes == 0`).
    BandwidthClaimWithoutByteAccounting {
        /// The percent-of-peak basis points the receipt
        /// claimed.
        percent_of_peak_basis_points: u32,
        /// The effective-bandwidth GB/s the receipt claimed.
        effective_bandwidth_gbps: u32,
    },
    /// Panel-required negative #2. Percent-of-peak claim with
    /// `theoretical_memory_bandwidth_gbps == 0` (no anchor to
    /// take a percentage against).
    PeakPercentageWithoutDeviceBandwidthDeclared {
        /// The percent-of-peak basis points the receipt
        /// claimed.
        percent_of_peak_basis_points: u32,
    },
    /// Panel-required negative #3. Layer-A receipt timed with
    /// `TimingMethod::HostJsonInclusiveTime`. Layer-A means
    /// "device-resident evidence fabric"; host JSON time
    /// destroys the claim.
    LayerAClaimWithHostJsonInclusiveTime,
    /// Panel-required negative #4. Saturation claim
    /// (percent-of-peak >= [`S_PERF_1_SATURATION_BP`]) backed
    /// by anything weaker than `TimingMethod::CudaEvent` or
    /// `TimingMethod::CudaStreamSync`.
    SaturationClaimWithoutCudaEventTiming {
        /// Observed percent-of-peak basis points.
        percent_of_peak_basis_points: u32,
        /// The (insufficient) timing method declared.
        timing_method_wire_name: &'static str,
    },
    /// Panel-required negative #5. Cross-device comparison
    /// includes at least one receipt with a zero
    /// `device_uuid_or_identity_hash` (no device identity to
    /// compare against).
    CrossDeviceComparisonWithoutDeviceIdentity,
    /// Panel-required negative #6. Effective-bandwidth claim
    /// (`effective_bandwidth_gbps > 0`) with
    /// `total_accounted_device_bytes == 0`. Distinct from
    /// negative #1 because this fires on the effective-
    /// bandwidth axis specifically (a percent-of-peak claim
    /// without bytes also fires negative #1; both can fire
    /// together).
    EffectiveBandwidthWhenTotalBytesZero {
        /// The effective bandwidth the receipt claimed.
        effective_bandwidth_gbps: u32,
    },
    /// Panel-required negative #7. Percent-of-peak above 100 %
    /// (`percent_of_peak_basis_points > 10000`) without the
    /// `accounting_overflow_acknowledged` flag set.
    PercentOfPeakAbove100WithoutErrorFlag {
        /// Observed percent-of-peak basis points.
        percent_of_peak_basis_points: u32,
    },
    /// Panel-required negative #8. Receipt declares no
    /// contract hashes (`contract_hashes.is_empty()`). Every
    /// performance claim must be tied to a specific execution
    /// contract.
    ReceiptMissingContractHashes,
    /// Structural defect: `device_name` is empty.
    DeviceNameEmpty,
    /// Structural defect: `driver_version` is empty.
    DriverVersionEmpty,
    /// Structural defect: `cuda_version` is empty.
    CudaVersionEmpty,
    /// Structural defect: `sm_arch == 0`. Compute capability
    /// 0 is meaningless; rejection prevents anonymous arch
    /// claims.
    SmArchZero,
    /// Structural defect: `timing_method == TimingMethod::Unknown`
    /// while `measured_kernel_time_us > 0`. An unknown timing
    /// method for a non-zero time measurement is incoherent.
    TimingMethodUnknownWithNonZeroTime,
    /// Structural defect: `total_accounted_device_bytes` does
    /// not equal the sum of the seven byte fields.
    AccountedBytesMismatchSumOfFields {
        /// The total the receipt claimed.
        claimed: u64,
        /// The sum the verifier computed from the seven byte
        /// fields.
        computed_sum: u64,
    },
    /// Structural defect: comparison includes fewer than 2
    /// receipts (a 0- or 1-receipt comparison is meaningless).
    ComparisonRequiresAtLeastTwoReceipts {
        /// Observed receipt count.
        actual: usize,
    },
    /// Structural defect: comparison includes two receipts
    /// with identical `device_uuid_or_identity_hash` (cross-
    /// device comparison requires distinct devices). Distinct
    /// from negative #5 (which fires on zero identity); this
    /// fires on identical non-zero identity.
    ComparisonReceiptsShareDeviceIdentity,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf1VerifyError {
    /// Error kind (see [`SPerf1VerifyErrorKind`]).
    pub kind: SPerf1VerifyErrorKind,
}

// ---------------------------------------------------------------
// Device identity helper
// ---------------------------------------------------------------

/// Compute a deterministic device-identity hash from
/// `(device_name, sm_arch)`. Used as a stand-in when a real
/// `cudaDeviceGetUuid` value is not available (e.g. when the
/// receipt is constructed before any CUDA call).
///
/// The resulting hash is non-zero by construction (the domain
/// separator alone guarantees a non-trivial input), so it
/// always satisfies panel-locked negative #5 when a caller uses
/// it deliberately.
#[must_use]
pub fn compute_device_identity_hash(device_name: &str, sm_arch: u32) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(DEVICE_IDENTITY_HASH_DOMAIN_V1.as_bytes());
    push_len_prefixed(&mut buf, device_name.as_bytes());
    buf.extend_from_slice(&sm_arch.to_be_bytes());
    sha256(&buf)
}

// ---------------------------------------------------------------
// Receipt builder
// ---------------------------------------------------------------

/// Build a [`DeviceTrafficReceiptV1`] and populate
/// `device_traffic_receipt_hash_v1` from the canonical-byte
/// projection of every field.
///
/// The caller is responsible for supplying the seven byte
/// fields AND `total_accounted_device_bytes`; the builder does
/// NOT auto-sum the byte fields because some receipt callers
/// may legitimately want to declare a different total (e.g. a
/// future variant where a shared buffer is counted once but the
/// per-field sum would double-count). The verifier rule
/// `AccountedBytesMismatchSumOfFields` catches mismatches at
/// admission time so the bug surfaces if the caller mis-sums.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_device_traffic_receipt(
    device_name: &'static str,
    device_uuid_or_identity_hash: [u8; 32],
    sm_arch: u32,
    driver_version: &'static str,
    cuda_version: &'static str,
    theoretical_memory_bandwidth_gbps: u32,
    measured_kernel_time_us: u64,
    timing_method: TimingMethod,
    layer: DeviceBandwidthLayer,
    detector_count: u32,
    catalog_count: u32,
    input_bytes: u64,
    evidence_bytes_read: u64,
    evidence_bytes_written: u64,
    witness_bytes_written: u64,
    fusion_bytes_read_written: u64,
    digest_bytes_read: u64,
    candidate_summary_bytes: u64,
    total_accounted_device_bytes: u64,
    effective_bandwidth_gbps: u32,
    percent_of_peak_basis_points: u32,
    accounting_overflow_acknowledged: bool,
    artifact_hashes: Vec<[u8; 32]>,
    contract_hashes: Vec<[u8; 32]>,
) -> DeviceTrafficReceiptV1 {
    let mut r = DeviceTrafficReceiptV1 {
        device_name,
        device_uuid_or_identity_hash,
        sm_arch,
        driver_version,
        cuda_version,
        theoretical_memory_bandwidth_gbps,
        measured_kernel_time_us,
        timing_method,
        layer,
        detector_count,
        catalog_count,
        input_bytes,
        evidence_bytes_read,
        evidence_bytes_written,
        witness_bytes_written,
        fusion_bytes_read_written,
        digest_bytes_read,
        candidate_summary_bytes,
        total_accounted_device_bytes,
        effective_bandwidth_gbps,
        percent_of_peak_basis_points,
        accounting_overflow_acknowledged,
        artifact_hashes,
        contract_hashes,
        device_traffic_receipt_hash_v1: [0u8; 32],
    };
    r.device_traffic_receipt_hash_v1 = compute_device_traffic_receipt_hash(&r);
    r
}

/// Build the panel-locked [`DeviceBandwidthClaimPolicyV1`].
/// Always returns the 8-line constant policy with its hash
/// populated; future versions of the policy require an
/// explicit schema bump.
#[must_use]
pub fn build_panel_locked_bandwidth_claim_policy() -> DeviceBandwidthClaimPolicyV1 {
    let mut p = DeviceBandwidthClaimPolicyV1 {
        policy_lines: S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES.to_vec(),
        device_bandwidth_claim_policy_hash_v1: [0u8; 32],
    };
    p.device_bandwidth_claim_policy_hash_v1 = compute_bandwidth_claim_policy_hash(&p);
    p
}

// ---------------------------------------------------------------
// Seed (uninstrumented baseline receipt)
// ---------------------------------------------------------------

/// Build the S-PERF.1 baseline receipt: device identity
/// declared for the RTX 4080 SUPER reference host, every
/// measurement field zero, contract hashes set to a single
/// `corpus_hash_v1` anchor.
///
/// The baseline represents "S-PERF.1 receipt schema exists;
/// no kernel has yet been timed". The verifier accepts the
/// baseline because zero bytes + zero time + zero bandwidth +
/// zero percent-of-peak is consistent; every later S-PERF.*
/// commit replaces these zeros with measured values.
///
/// Use this seed in tests and CLI output as a known-good
/// reference; do NOT use it as evidence of any measured
/// bandwidth.
#[must_use]
pub fn seed_baseline_uninstrumented_receipt() -> DeviceTrafficReceiptV1 {
    let corpus_anchor = compute_corpus_hash_v1().bytes;
    build_device_traffic_receipt(
        "RTX 4080 SUPER",
        compute_device_identity_hash("RTX 4080 SUPER", 89),
        89,
        "13.2.0",
        "13.2",
        716, // panel-locked: vendor datasheet peak for RTX 4080 SUPER
        0,   // measured_kernel_time_us — uninstrumented baseline
        TimingMethod::CudaEvent,
        DeviceBandwidthLayer::LayerA,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, // total_accounted_device_bytes
        0, // effective_bandwidth_gbps
        0, // percent_of_peak_basis_points
        false,
        vec![corpus_anchor], // artifact_hashes
        vec![corpus_anchor], // contract_hashes (must be non-empty per rule #8)
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_device_traffic_receipt_hash(r: &DeviceTrafficReceiptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(DEVICE_TRAFFIC_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.device_name.as_bytes());
    buf.extend_from_slice(&r.device_uuid_or_identity_hash);
    buf.extend_from_slice(&r.sm_arch.to_be_bytes());
    push_len_prefixed(&mut buf, r.driver_version.as_bytes());
    push_len_prefixed(&mut buf, r.cuda_version.as_bytes());
    buf.extend_from_slice(&r.theoretical_memory_bandwidth_gbps.to_be_bytes());
    buf.extend_from_slice(&r.measured_kernel_time_us.to_be_bytes());
    push_len_prefixed(&mut buf, r.timing_method.as_str().as_bytes());
    push_len_prefixed(&mut buf, r.layer.as_str().as_bytes());
    buf.extend_from_slice(&r.detector_count.to_be_bytes());
    buf.extend_from_slice(&r.catalog_count.to_be_bytes());
    buf.extend_from_slice(&r.input_bytes.to_be_bytes());
    buf.extend_from_slice(&r.evidence_bytes_read.to_be_bytes());
    buf.extend_from_slice(&r.evidence_bytes_written.to_be_bytes());
    buf.extend_from_slice(&r.witness_bytes_written.to_be_bytes());
    buf.extend_from_slice(&r.fusion_bytes_read_written.to_be_bytes());
    buf.extend_from_slice(&r.digest_bytes_read.to_be_bytes());
    buf.extend_from_slice(&r.candidate_summary_bytes.to_be_bytes());
    buf.extend_from_slice(&r.total_accounted_device_bytes.to_be_bytes());
    buf.extend_from_slice(&r.effective_bandwidth_gbps.to_be_bytes());
    buf.extend_from_slice(&r.percent_of_peak_basis_points.to_be_bytes());
    buf.push(u8::from(r.accounting_overflow_acknowledged));
    push_hash_list(&mut buf, &r.artifact_hashes);
    push_hash_list(&mut buf, &r.contract_hashes);
    sha256(&buf)
}

fn compute_bandwidth_claim_policy_hash(p: &DeviceBandwidthClaimPolicyV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(DEVICE_BANDWIDTH_CLAIM_POLICY_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    let n = u32::try_from(p.policy_lines.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for line in &p.policy_lines {
        push_len_prefixed(&mut buf, line.as_bytes());
    }
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn push_hash_list(buf: &mut Vec<u8>, hashes: &[[u8; 32]]) {
    let n = u32::try_from(hashes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for h in hashes {
        buf.extend_from_slice(h);
    }
}

// ---------------------------------------------------------------
// Verifier --- single receipt
// ---------------------------------------------------------------

/// Verify one device-traffic receipt against the 8 panel-locked
/// policy rules plus structural defects. Returns the list of
/// errors (empty on admission). The verifier is total: it never
/// short-circuits; every receipt is walked through every rule
/// so a single defective receipt surfaces every applicable
/// failure at once.
#[must_use]
#[allow(clippy::too_many_lines)] // 8 panel-required negatives + structural rules; splitting obscures panel numbering
pub fn verify_device_traffic_receipt(receipt: &DeviceTrafficReceiptV1) -> Vec<SPerf1VerifyError> {
    let mut errors: Vec<SPerf1VerifyError> = Vec::new();

    // Panel-required negative #1: bandwidth claim without byte accounting.
    // A claim is "non-trivial" when EITHER effective_bandwidth_gbps OR
    // percent_of_peak_basis_points is non-zero. Either one without supporting
    // total_accounted_device_bytes is rejected.
    if (receipt.effective_bandwidth_gbps > 0 || receipt.percent_of_peak_basis_points > 0)
        && receipt.total_accounted_device_bytes == 0
    {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::BandwidthClaimWithoutByteAccounting {
                percent_of_peak_basis_points: receipt.percent_of_peak_basis_points,
                effective_bandwidth_gbps: receipt.effective_bandwidth_gbps,
            },
        });
    }

    // Panel-required negative #2: peak-percentage without declared device bandwidth.
    if receipt.percent_of_peak_basis_points > 0 && receipt.theoretical_memory_bandwidth_gbps == 0 {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::PeakPercentageWithoutDeviceBandwidthDeclared {
                percent_of_peak_basis_points: receipt.percent_of_peak_basis_points,
            },
        });
    }

    // Panel-required negative #3: Layer-A claim timed with host JSON inclusive.
    if matches!(receipt.layer, DeviceBandwidthLayer::LayerA)
        && matches!(receipt.timing_method, TimingMethod::HostJsonInclusiveTime)
    {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::LayerAClaimWithHostJsonInclusiveTime,
        });
    }

    // Panel-required negative #4: saturation claim without CUDA event timing.
    // Saturation = percent_of_peak_basis_points >= S_PERF_1_SATURATION_BP
    // (80.00 %). CudaStreamSync is also admissible because it still times
    // the device-side kernel sequence; HostInstantOnly /
    // HostJsonInclusiveTime / Unknown are insufficient.
    if receipt.percent_of_peak_basis_points >= S_PERF_1_SATURATION_BP
        && !receipt.timing_method.is_device_resident()
    {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::SaturationClaimWithoutCudaEventTiming {
                percent_of_peak_basis_points: receipt.percent_of_peak_basis_points,
                timing_method_wire_name: receipt.timing_method.as_str(),
            },
        });
    }

    // Panel-required negative #5: cross-device comparison without device identity.
    // The per-receipt rule: device_uuid_or_identity_hash MUST be non-zero
    // so the receipt is admissible for any future cross-device comparison.
    // The companion comparison verifier (verify_cross_device_comparison)
    // surfaces the same error variant on the comparison act.
    if receipt.device_uuid_or_identity_hash == [0u8; 32] {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::CrossDeviceComparisonWithoutDeviceIdentity,
        });
    }

    // Panel-required negative #6: effective bandwidth declared with zero total bytes.
    if receipt.effective_bandwidth_gbps > 0 && receipt.total_accounted_device_bytes == 0 {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::EffectiveBandwidthWhenTotalBytesZero {
                effective_bandwidth_gbps: receipt.effective_bandwidth_gbps,
            },
        });
    }

    // Panel-required negative #7: percent-of-peak above 100 without explicit flag.
    if receipt.percent_of_peak_basis_points > 10_000 && !receipt.accounting_overflow_acknowledged {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::PercentOfPeakAbove100WithoutErrorFlag {
                percent_of_peak_basis_points: receipt.percent_of_peak_basis_points,
            },
        });
    }

    // Panel-required negative #8: receipt missing contract hashes.
    if receipt.contract_hashes.is_empty() {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::ReceiptMissingContractHashes,
        });
    }

    // Structural defects.
    if receipt.device_name.is_empty() {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::DeviceNameEmpty,
        });
    }
    if receipt.driver_version.is_empty() {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::DriverVersionEmpty,
        });
    }
    if receipt.cuda_version.is_empty() {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::CudaVersionEmpty,
        });
    }
    if receipt.sm_arch == 0 {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::SmArchZero,
        });
    }
    if matches!(receipt.timing_method, TimingMethod::Unknown) && receipt.measured_kernel_time_us > 0
    {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::TimingMethodUnknownWithNonZeroTime,
        });
    }
    let computed_sum: u64 = receipt
        .input_bytes
        .saturating_add(receipt.evidence_bytes_read)
        .saturating_add(receipt.evidence_bytes_written)
        .saturating_add(receipt.witness_bytes_written)
        .saturating_add(receipt.fusion_bytes_read_written)
        .saturating_add(receipt.digest_bytes_read)
        .saturating_add(receipt.candidate_summary_bytes);
    if computed_sum != receipt.total_accounted_device_bytes {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::AccountedBytesMismatchSumOfFields {
                claimed: receipt.total_accounted_device_bytes,
                computed_sum,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- cross-device comparison
// ---------------------------------------------------------------

/// Verify a cross-device comparison. Panel-required negative #5
/// fires on a per-receipt basis (zero device identity), but a
/// comparison ALSO requires (a) at least 2 receipts and
/// (b) distinct device identities. This helper surfaces those
/// comparison-level defects explicitly so a future S-PERF.7 /
/// S-MG.6 cross-arch claim cannot land without satisfying the
/// cross-device discipline.
#[must_use]
pub fn verify_cross_device_comparison(
    comparison: &DeviceTrafficReceiptComparison<'_>,
) -> Vec<SPerf1VerifyError> {
    let mut errors: Vec<SPerf1VerifyError> = Vec::new();

    if comparison.receipts.len() < 2 {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::ComparisonRequiresAtLeastTwoReceipts {
                actual: comparison.receipts.len(),
            },
        });
        return errors;
    }

    // If any receipt has zero device identity, surface the
    // panel-locked negative #5.
    for r in comparison.receipts {
        if r.device_uuid_or_identity_hash == [0u8; 32] {
            errors.push(SPerf1VerifyError {
                kind: SPerf1VerifyErrorKind::CrossDeviceComparisonWithoutDeviceIdentity,
            });
            break;
        }
    }

    // Distinct device identities: a comparison whose receipts
    // all share the same non-zero UUID is not a CROSS-device
    // comparison.
    let unique_uuids: BTreeSet<[u8; 32]> = comparison
        .receipts
        .iter()
        .map(|r| r.device_uuid_or_identity_hash)
        .collect();
    if unique_uuids.len() < comparison.receipts.len() {
        errors.push(SPerf1VerifyError {
            kind: SPerf1VerifyErrorKind::ComparisonReceiptsShareDeviceIdentity,
        });
    }

    errors
}

// ---------------------------------------------------------------
// Renderers --- text
// ---------------------------------------------------------------

/// Render the receipt as deterministic text. Two builds of the
/// same receipt produce byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)] // wide schema; one writeln per field
pub fn render_device_traffic_receipt_text(r: &DeviceTrafficReceiptV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.1 DeviceTrafficReceiptV1");
    let _ = writeln!(s, "===============================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Device identity");
    let _ = writeln!(s, "  device_name                       : {}", r.device_name);
    let _ = writeln!(
        s,
        "  device_uuid_or_identity_hash      : {}",
        hex32(&r.device_uuid_or_identity_hash)
    );
    let _ = writeln!(s, "  sm_arch                           : {}", r.sm_arch);
    let _ = writeln!(
        s,
        "  driver_version                    : {}",
        r.driver_version
    );
    let _ = writeln!(
        s,
        "  cuda_version                      : {}",
        r.cuda_version
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Bandwidth posture");
    let _ = writeln!(
        s,
        "  theoretical_memory_bandwidth_gbps : {}",
        r.theoretical_memory_bandwidth_gbps
    );
    let _ = writeln!(
        s,
        "  measured_kernel_time_us           : {}",
        r.measured_kernel_time_us
    );
    let _ = writeln!(
        s,
        "  timing_method                     : {}",
        r.timing_method.as_str()
    );
    let _ = writeln!(
        s,
        "  layer                             : {}",
        r.layer.as_str()
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Workload");
    let _ = writeln!(
        s,
        "  detector_count                    : {}",
        r.detector_count
    );
    let _ = writeln!(
        s,
        "  catalog_count                     : {}",
        r.catalog_count
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Byte accounting");
    let _ = writeln!(s, "  input_bytes                       : {}", r.input_bytes);
    let _ = writeln!(
        s,
        "  evidence_bytes_read               : {}",
        r.evidence_bytes_read
    );
    let _ = writeln!(
        s,
        "  evidence_bytes_written            : {}",
        r.evidence_bytes_written
    );
    let _ = writeln!(
        s,
        "  witness_bytes_written             : {}",
        r.witness_bytes_written
    );
    let _ = writeln!(
        s,
        "  fusion_bytes_read_written         : {}",
        r.fusion_bytes_read_written
    );
    let _ = writeln!(
        s,
        "  digest_bytes_read                 : {}",
        r.digest_bytes_read
    );
    let _ = writeln!(
        s,
        "  candidate_summary_bytes           : {}",
        r.candidate_summary_bytes
    );
    let _ = writeln!(
        s,
        "  total_accounted_device_bytes      : {}",
        r.total_accounted_device_bytes
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Effective claim");
    let _ = writeln!(
        s,
        "  effective_bandwidth_gbps          : {}",
        r.effective_bandwidth_gbps
    );
    let _ = writeln!(
        s,
        "  percent_of_peak_basis_points      : {}",
        r.percent_of_peak_basis_points
    );
    let _ = writeln!(
        s,
        "  accounting_overflow_acknowledged  : {}",
        r.accounting_overflow_acknowledged
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Anchors");
    let _ = writeln!(s, "  artifact_hashes ({})", r.artifact_hashes.len());
    for h in &r.artifact_hashes {
        let _ = writeln!(s, "    {}", hex32(h));
    }
    let _ = writeln!(s, "  contract_hashes ({})", r.contract_hashes.len());
    for h in &r.contract_hashes {
        let _ = writeln!(s, "    {}", hex32(h));
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "device_traffic_receipt_hash_v1 : {}",
        hex32(&r.device_traffic_receipt_hash_v1)
    );
    s
}

/// Render the panel-locked bandwidth-claim policy as
/// deterministic text. Two builds produce byte-identical
/// output.
#[must_use]
pub fn render_bandwidth_claim_policy_text(p: &DeviceBandwidthClaimPolicyV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.1 DeviceBandwidthClaimPolicyV1");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked policy lines ({})", p.policy_lines.len());
    for (i, line) in p.policy_lines.iter().enumerate() {
        let _ = writeln!(s, "  {}. {}", i + 1, line);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "device_bandwidth_claim_policy_hash_v1 : {}",
        hex32(&p.device_bandwidth_claim_policy_hash_v1)
    );
    s
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render the receipt as canonical JSON. Two builds produce
/// byte-identical output.
#[must_use]
pub fn render_device_traffic_receipt_json(r: &DeviceTrafficReceiptV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", DEVICE_TRAFFIC_RECEIPT_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "device_name", r.device_name);
    s.push(',');
    json_hex(
        &mut s,
        "device_uuid_or_identity_hash",
        &r.device_uuid_or_identity_hash,
    );
    s.push(',');
    let _ = write!(s, "\"sm_arch\":{}", r.sm_arch);
    s.push(',');
    json_string(&mut s, "driver_version", r.driver_version);
    s.push(',');
    json_string(&mut s, "cuda_version", r.cuda_version);
    s.push(',');
    let _ = write!(
        s,
        "\"theoretical_memory_bandwidth_gbps\":{}",
        r.theoretical_memory_bandwidth_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"measured_kernel_time_us\":{}",
        r.measured_kernel_time_us
    );
    s.push(',');
    json_string(&mut s, "timing_method", r.timing_method.as_str());
    s.push(',');
    json_string(&mut s, "layer", r.layer.as_str());
    s.push(',');
    let _ = write!(s, "\"detector_count\":{}", r.detector_count);
    s.push(',');
    let _ = write!(s, "\"catalog_count\":{}", r.catalog_count);
    s.push(',');
    let _ = write!(s, "\"input_bytes\":{}", r.input_bytes);
    s.push(',');
    let _ = write!(s, "\"evidence_bytes_read\":{}", r.evidence_bytes_read);
    s.push(',');
    let _ = write!(s, "\"evidence_bytes_written\":{}", r.evidence_bytes_written);
    s.push(',');
    let _ = write!(s, "\"witness_bytes_written\":{}", r.witness_bytes_written);
    s.push(',');
    let _ = write!(
        s,
        "\"fusion_bytes_read_written\":{}",
        r.fusion_bytes_read_written
    );
    s.push(',');
    let _ = write!(s, "\"digest_bytes_read\":{}", r.digest_bytes_read);
    s.push(',');
    let _ = write!(
        s,
        "\"candidate_summary_bytes\":{}",
        r.candidate_summary_bytes
    );
    s.push(',');
    let _ = write!(
        s,
        "\"total_accounted_device_bytes\":{}",
        r.total_accounted_device_bytes
    );
    s.push(',');
    let _ = write!(
        s,
        "\"effective_bandwidth_gbps\":{}",
        r.effective_bandwidth_gbps
    );
    s.push(',');
    let _ = write!(
        s,
        "\"percent_of_peak_basis_points\":{}",
        r.percent_of_peak_basis_points
    );
    s.push(',');
    let _ = write!(
        s,
        "\"accounting_overflow_acknowledged\":{}",
        r.accounting_overflow_acknowledged
    );
    s.push(',');
    json_hex_list(&mut s, "artifact_hashes", &r.artifact_hashes);
    s.push(',');
    json_hex_list(&mut s, "contract_hashes", &r.contract_hashes);
    s.push(',');
    json_hex(
        &mut s,
        "device_traffic_receipt_hash_v1",
        &r.device_traffic_receipt_hash_v1,
    );
    s.push('}');
    s
}

/// Render the bandwidth-claim policy as canonical JSON.
#[must_use]
pub fn render_bandwidth_claim_policy_json(p: &DeviceBandwidthClaimPolicyV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", DEVICE_BANDWIDTH_CLAIM_POLICY_SCHEMA_V1);
    s.push(',');
    let _ = write!(s, "\"policy_line_count\":{}", p.policy_lines.len());
    s.push(',');
    s.push_str("\"policy_lines\":[");
    for (i, line) in p.policy_lines.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        json_quoted(&mut s, line);
    }
    s.push(']');
    s.push(',');
    json_hex(
        &mut s,
        "device_bandwidth_claim_policy_hash_v1",
        &p.device_bandwidth_claim_policy_hash_v1,
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
    s.push_str("\":");
    json_quoted(s, value);
}

fn json_quoted(s: &mut String, value: &str) {
    s.push('"');
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

fn json_hex_list(s: &mut String, key: &str, values: &[[u8; 32]]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":[");
    for (i, h) in values.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        let _ = s.write_str(&hex32(h));
        s.push('"');
    }
    s.push(']');
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Test-only read-only access to the panel-locked policy lines
/// for cross-checking that the constant has not drifted from
/// the in-memory builder output.
#[doc(hidden)]
#[must_use]
pub fn panel_locked_bandwidth_claim_policy_lines() -> &'static [&'static str] {
    S_PERF_1_BANDWIDTH_CLAIM_POLICY_LINES
}
