//! S-PERF.2 --- Layer-A resident densor pipeline.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S-PERF.2 builds the first Layer-A device-resident
//! > evidence pipeline so traffic receipts can measure GPU
//! > evidence production without host JSON, report rendering,
//! > transcript construction, or court explanation time mixed
//! > in.**
//!
//! Core rule (panel-locked):
//!
//! > Layer-A measures evidence production only:
//! > EvidenceDensor → WitnessDensor → FusionDensor →
//! > CandidateDensor + digests. No host-side transcript. No
//! > JSON/report timing. No CaseFileV2 materialization. No
//! > semantic admission timing.
//!
//! ## Why
//!
//! S-PERF.1 defined the byte-accounting receipt every future
//! bandwidth claim must cite. S-PERF.2 supplies the
//! *pipeline shape* that receipt is taken against: a
//! deterministic device-resident evidence pipeline whose
//! H2D / D2H byte traffic is exclusively (a) the input
//! artifact going up, and (b) compact stage digests +
//! candidate summaries coming back. Witness, fusion, and
//! evidence densors stay on the device for the duration of
//! a batch; the host does NOT see them. Without this
//! isolation, any "Layer-A" performance claim would silently
//! include host-side overhead the GPU did not actually do.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes (none folded upstream):
//!
//! - `layer_a_resident_pipeline_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-RESIDENT-PIPELINE:v1\0`. Pins
//!   the bytes of one Layer-A pipeline schema (stage list,
//!   per-densor residency declarations, forbidden-host-
//!   activity flags).
//! - `layer_a_device_residency_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-DEVICE-RESIDENCY-RECEIPT:v1\0`.
//!   Pins the per-densor H2D / D2H byte accounting for one
//!   pipeline run.
//! - `layer_a_traffic_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:LAYER-A-TRAFFIC-RECEIPT:v1\0`. Top-
//!   level META-hash binding the pipeline + residency
//!   receipt + a referenced S-PERF.1
//!   `device_traffic_receipt_hash_v1` + the court-authority
//!   anchor list the Layer-A pipeline promises NOT to
//!   mutate.
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.2 does NOT:
//!
//! - claim that DSFB-GPU has measured peak memory-bandwidth
//!   saturation (that is a separate S-PERF.* commit gated on
//!   the Layer-A pipeline running on real hardware);
//! - claim production CUDA performance numbers;
//! - benchmark B300 / GB300 cloud hardware;
//! - change any CUDA kernel;
//! - change any court decision (S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e / S1.3f / S1.3g);
//! - mutate any upstream hash anchor (`corpus_hash_v1`,
//!   `corpus_hash_v2`, every T.11.* / T.12.* / FF.* /
//!   S1.3.* / T.12.PROV / S-PERF.1 hash byte-identical);
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs / witness records / fusion
//!   tensors / candidate intervals / episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate.
//!
//! S-PERF.2 ships ONLY the pipeline schema, the residency
//! receipt, the traffic-receipt envelope, the verifier, the
//! builder, and the renderers. The first actual measurement
//! against this pipeline is a separate S-PERF.3+ commit
//! gated on the public-data saturation bundle work.
//!
//! ## Panel-locked one-line verdict
//!
//! > S-PERF.1 gave the ruler; S-PERF.2 isolates the GPU
//! > evidence-factory path the ruler will measure.

use core::fmt::Write;
use std::collections::BTreeSet;

use dsfb_gpu_debug_core::sha256;

use crate::corpus_hash::compute_corpus_hash_v1;
use crate::s_perf_1_device_traffic_receipt::{
    compute_device_identity_hash, seed_baseline_uninstrumented_receipt, TimingMethod,
};

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `layer_a_resident_pipeline_hash_v1`.
/// The trailing `\0` byte mirrors the S-PERF.1 discipline:
/// it ensures the pipeline hash cannot be silently absorbed
/// into a sibling domain by careless concatenation.
pub const LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:LAYER-A-RESIDENT-PIPELINE:v1\0";

/// Schema identifier for `layer_a_resident_pipeline_hash_v1`.
pub const LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:LAYER-A-RESIDENT-PIPELINE:v1";

/// Domain separator for
/// `layer_a_device_residency_receipt_hash_v1`.
pub const LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:LAYER-A-DEVICE-RESIDENCY-RECEIPT:v1\0";

/// Schema identifier for
/// `layer_a_device_residency_receipt_hash_v1`.
pub const LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:LAYER-A-DEVICE-RESIDENCY-RECEIPT:v1";

/// Domain separator for `layer_a_traffic_receipt_hash_v1`.
pub const LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:LAYER-A-TRAFFIC-RECEIPT:v1\0";

/// Schema identifier for `layer_a_traffic_receipt_hash_v1`.
pub const LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:LAYER-A-TRAFFIC-RECEIPT:v1";

// ---------------------------------------------------------------
// Panel-locked Layer-A canonical stage names
// ---------------------------------------------------------------

/// The five panel-locked Layer-A stage names. The Layer-A
/// pipeline is a fixed-form sequence: residual / evidence
/// densor projection → witness densor evaluation → fusion
/// densor reduction → candidate densor collapse → stage
/// digest emission. Each stage stays device-resident; the
/// only H2D traffic is the input artifact, and the only D2H
/// traffic is the stage digests + compact candidate summary.
///
/// Pipelines built via [`seed_baseline_layer_a_pipeline`]
/// declare exactly these five stages. Future S-PERF.* commits
/// MAY declare a different stage breakdown (the schema admits
/// any non-empty `stage_names` list as long as `stage_count`
/// matches), but the panel-locked baseline carries these five.
pub const LAYER_A_CANONICAL_STAGE_NAMES: &[&str] = &[
    "EvidenceDensorProjection",
    "WitnessDensorEvaluation",
    "FusionDensorReduction",
    "CandidateDensorCollapse",
    "StageDigestEmission",
];

// ---------------------------------------------------------------
// LayerADensorKind
// ---------------------------------------------------------------

/// The densor categories Layer-A's residency receipt accounts
/// for. The five categories cover the entire H2D / D2H byte
/// surface of a Layer-A run: input artifact bytes (Evidence),
/// per-cell witness outputs (Witness), fusion-plane outputs
/// (Fusion), compact candidate descriptors (Candidate), and
/// the five stage digests (StageDigest).
///
/// Wire names are stable for the hash buffer; do not rename
/// without rebaselining `layer_a_resident_pipeline_hash_v1` +
/// `layer_a_device_residency_receipt_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayerADensorKind {
    /// Residual / evidence densor cells projected from the
    /// input artifact. Must stay device-resident in Layer-A
    /// (only the input artifact's H2D copy traverses PCIe).
    Evidence,
    /// Per-cell witness densor outputs (detector firings).
    /// Device-resident-only in Layer-A.
    Witness,
    /// Fusion-plane densor outputs (axis-fused witness
    /// evidence). Device-resident-only in Layer-A.
    Fusion,
    /// Compact candidate descriptors (post-bank-style
    /// candidate collapse). Layer-A may D2H these as a
    /// compact summary (panel-locked: candidate summary is
    /// the only non-digest D2H allowed).
    Candidate,
    /// The five stage digests (tree-digest leaves). Always
    /// D2H-allowed in Layer-A; capped to 32 bytes × 5 = 160
    /// bytes per catalog.
    StageDigest,
}

impl LayerADensorKind {
    /// Canonical wire name for the hash buffer + renderers.
    /// Mirrors the variant name. Stable across releases.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "Evidence",
            Self::Witness => "Witness",
            Self::Fusion => "Fusion",
            Self::Candidate => "Candidate",
            Self::StageDigest => "StageDigest",
        }
    }
}

// ---------------------------------------------------------------
// DeviceResidencyClass
// ---------------------------------------------------------------

/// How a densor's device-residency is declared. Layer-A
/// requires every Evidence / Witness / Fusion densor to be
/// `DeviceResidentOnly`; Candidate + StageDigest densors may
/// be `DeviceResidentWithCompactD2H`. `HostMaterialized`
/// is forbidden for Layer-A (it would mean the host saw the
/// full densor bytes, which moves the workload out of
/// Layer-A's measurement window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceResidencyClass {
    /// Allocated, read, written, and consumed entirely on the
    /// device; never copied back. Layer-A admits this for
    /// Evidence / Witness / Fusion densors.
    DeviceResidentOnly,
    /// Allocated on the device; copied back as a compact
    /// summary or digest. Layer-A admits this for Candidate
    /// (compact descriptors) and StageDigest (tree-digest
    /// leaves) densors.
    DeviceResidentWithCompactD2H,
    /// Materialised on the host (Layer-C territory). Layer-A
    /// rejects this class on any densor.
    HostMaterialized,
}

impl DeviceResidencyClass {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeviceResidentOnly => "DeviceResidentOnly",
            Self::DeviceResidentWithCompactD2H => "DeviceResidentWithCompactD2H",
            Self::HostMaterialized => "HostMaterialized",
        }
    }
}

// ---------------------------------------------------------------
// LayerADensorResidencyDeclaration
// ---------------------------------------------------------------

/// One per-densor residency declaration. The Layer-A pipeline
/// MUST carry one declaration per densor kind it uses; the
/// verifier uses this set to (a) confirm Witness / Fusion /
/// Evidence densors are `DeviceResidentOnly` (negative #4,
/// "no full-witness D2H dump") and (b) compute the per-densor
/// expected D2H ceiling for residency-receipt admission.
///
/// `expected_max_d2h_bytes_per_catalog = 0` for
/// `DeviceResidentOnly` densors; non-zero for
/// `DeviceResidentWithCompactD2H` densors (the panel-locked
/// caps are 32 × 5 = 160 bytes for stage digests, and a
/// reasonable per-catalog cap for candidate summaries set by
/// the caller).
#[derive(Debug, Clone)]
pub struct LayerADensorResidencyDeclaration {
    /// Which densor category this declaration covers.
    pub densor_kind: LayerADensorKind,
    /// How the densor is allowed to be transferred.
    pub residency_class: DeviceResidencyClass,
    /// Caller-declared upper bound on D2H bytes per catalog
    /// for this densor. Zero when `residency_class ==
    /// DeviceResidentOnly`; non-zero when
    /// `residency_class == DeviceResidentWithCompactD2H`.
    /// The verifier enforces the receipt's per-densor D2H
    /// bytes do not exceed this cap.
    pub expected_max_d2h_bytes_per_catalog: u64,
}

// ---------------------------------------------------------------
// LayerAResidentPipelineV1
// ---------------------------------------------------------------

/// The Layer-A pipeline schema. Declares the stage sequence,
/// the per-densor residency policy, and the panel-locked set
/// of *forbidden* host activities. Every "forbidden" flag
/// MUST be `false` for Layer-A admission.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `layer_a_resident_pipeline_hash_v1`.
#[derive(Debug, Clone)]
pub struct LayerAResidentPipelineV1 {
    /// Human-readable pipeline identifier (non-empty).
    pub pipeline_id: &'static str,
    /// Number of stages (must equal `stage_names.len()`).
    pub stage_count: u32,
    /// Ordered list of stage wire names. The panel-locked
    /// baseline declares [`LAYER_A_CANONICAL_STAGE_NAMES`].
    pub stage_names: Vec<&'static str>,
    /// One residency declaration per densor kind the pipeline
    /// uses. The panel-locked baseline declares all five
    /// kinds.
    pub residency_declarations: Vec<LayerADensorResidencyDeclaration>,
    /// True iff the pipeline materialises a `CaseFileV2`
    /// inside the timed window. Layer-A MUST be `false`
    /// (panel-locked negative #2).
    pub casefile_materialization_present: bool,
    /// True iff the pipeline constructs a host-side trial
    /// transcript inside the timed window. Layer-A MUST be
    /// `false`.
    pub host_transcript_present: bool,
    /// True iff the pipeline emits JSON / report bytes inside
    /// the timed window. Layer-A MUST be `false` (panel-
    /// locked negative #1).
    pub host_json_emission_present: bool,
    /// True iff the pipeline runs semantic-admission (bank)
    /// decisions inside the timed window. Layer-A MUST be
    /// `false`; bank admission is the Layer-B boundary.
    pub semantic_admission_present: bool,
    /// True iff the pipeline would mutate any court-authority
    /// hash anchor. Layer-A MUST be `false` (panel-locked
    /// negative #8); the Layer-A measurement window must be
    /// court-state-invariant.
    pub mutates_court_authority_hashes: bool,
    /// `layer_a_resident_pipeline_hash_v1`. Populated by
    /// [`build_layer_a_resident_pipeline`] from the canonical-
    /// byte projection of every field above.
    pub layer_a_resident_pipeline_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// LayerADeviceResidencyReceiptV1
// ---------------------------------------------------------------

/// Per-densor H2D / D2H byte accounting for one Layer-A
/// pipeline run. The receipt references the pipeline schema
/// by hash so the verifier can cross-check residency-class
/// declarations against the measured D2H bytes (panel-locked
/// negative #4: any `DeviceResidentOnly` densor with
/// non-zero D2H bytes is rejected).
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `layer_a_device_residency_receipt_hash_v1`.
#[derive(Debug, Clone)]
pub struct LayerADeviceResidencyReceiptV1 {
    /// Hash of the pipeline schema this receipt was measured
    /// against (must equal
    /// `pipeline.layer_a_resident_pipeline_hash_v1`).
    pub pipeline_hash: [u8; 32],
    /// Per-densor H2D bytes (sorted ascending by densor wire
    /// name). The verifier rejects empty lists (panel-locked
    /// negative #5: missing H2D / D2H byte accounting).
    pub per_densor_h2d_bytes: Vec<(LayerADensorKind, u64)>,
    /// Per-densor D2H bytes (sorted ascending by densor wire
    /// name). Same empty-list rule as `per_densor_h2d_bytes`.
    pub per_densor_d2h_bytes: Vec<(LayerADensorKind, u64)>,
    /// Sum of `per_densor_h2d_bytes` values (structural
    /// invariant: must equal the sum).
    pub total_h2d_bytes: u64,
    /// Sum of `per_densor_d2h_bytes` values.
    pub total_d2h_bytes: u64,
    /// `layer_a_device_residency_receipt_hash_v1`.
    pub layer_a_device_residency_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// LayerATrafficReceiptV1
// ---------------------------------------------------------------

/// Top-level Layer-A traffic receipt. Binds the pipeline +
/// the residency receipt + a referenced S-PERF.1
/// `DeviceTrafficReceiptV1` by hash + the court-authority
/// hash anchors the pipeline promises NOT to mutate. One
/// hash pins the whole Layer-A measurement chain.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `layer_a_traffic_receipt_hash_v1`.
#[derive(Debug, Clone)]
pub struct LayerATrafficReceiptV1 {
    /// The Layer-A pipeline schema this receipt was measured
    /// against.
    pub pipeline: LayerAResidentPipelineV1,
    /// The residency receipt that pins per-densor H2D / D2H
    /// byte accounting.
    pub residency_receipt: LayerADeviceResidencyReceiptV1,
    /// Hash of the S-PERF.1 `DeviceTrafficReceiptV1` this
    /// Layer-A claim is taken against. Must be non-zero
    /// (panel-locked negative #7).
    pub device_traffic_receipt_hash_v1: [u8; 32],
    /// Wire-name copy of the inner `DeviceTrafficReceiptV1`'s
    /// `timing_method`. Carried separately so the Layer-A
    /// verifier can enforce the S-PERF.1 Layer-A timing rule
    /// without re-resolving the hash reference (panel-locked
    /// negative #6: must be `CudaEvent` or `CudaStreamSync`).
    pub inner_timing_method_wire_name: &'static str,
    /// The court-authority hash anchors the Layer-A pipeline
    /// promises to keep byte-stable across the timed window.
    /// Must include `corpus_hash_v1` at minimum.
    pub court_authority_hash_anchors: Vec<[u8; 32]>,
    /// `layer_a_traffic_receipt_hash_v1`. Populated by
    /// [`build_layer_a_traffic_receipt`].
    pub layer_a_traffic_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.2 rejected a pipeline, residency receipt, or
/// traffic receipt. Eight panel-required load-bearing
/// negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf2VerifyErrorKind {
    /// Panel-required negative #1. Layer-A pipeline declares
    /// `host_json_emission_present == true`; the host-side
    /// JSON emission is happening inside the timed window,
    /// which is forbidden for Layer-A.
    LayerAReceiptWithHostJsonTime,
    /// Panel-required negative #2. Layer-A pipeline declares
    /// `casefile_materialization_present == true`; the
    /// host-side `CaseFileV2` build is happening inside the
    /// timed window.
    LayerAReceiptWithCasefileMaterializationTime,
    /// Panel-required negative #3. Pipeline has zero
    /// per-densor residency declarations.
    PipelineWithoutDeviceResidencyDeclaration,
    /// Panel-required negative #4. A densor declared
    /// `DeviceResidentOnly` (Evidence / Witness / Fusion)
    /// has non-zero D2H bytes in the residency receipt --- a
    /// full witness/fusion/evidence D2H dump that would
    /// violate "summary only" Layer-A discipline.
    D2hFullWitnessDumpWhenSummaryOnlyDeclared {
        /// Which densor was illegally dumped.
        densor_kind_wire_name: &'static str,
        /// How many D2H bytes were observed.
        observed_d2h_bytes: u64,
    },
    /// Panel-required negative #5. Residency receipt has
    /// empty per-densor H2D AND empty per-densor D2H lists
    /// (no byte accounting at all). A Layer-A receipt may
    /// legitimately have zero-valued entries (baseline), but
    /// the per-densor lists must be populated.
    MissingH2dD2hByteAccounting,
    /// Panel-required negative #6. The inner
    /// `DeviceTrafficReceiptV1`'s `timing_method` (carried
    /// as the `inner_timing_method_wire_name` field) is not
    /// one of `CudaEvent` or `CudaStreamSync`. Layer-A claims
    /// require device-resident timing per S-PERF.1.
    CudaTimingMethodNotAllowedBySPerf1 {
        /// The (insufficient) timing method wire name.
        observed_timing_method_wire_name: &'static str,
    },
    /// Panel-required negative #7. Layer-A traffic receipt
    /// has `device_traffic_receipt_hash_v1 == [0; 32]`
    /// (claim made without reference to an S-PERF.1 receipt).
    LayerAClaimWithoutDeviceTrafficReceipt,
    /// Panel-required negative #8. Pipeline declares
    /// `mutates_court_authority_hashes == true` (the timed
    /// window would silently mutate a court anchor).
    PipelineThatMutatesCourtAuthorityHashes,
    /// Structural defect: pipeline `pipeline_id` is empty.
    PipelineIdEmpty,
    /// Structural defect: pipeline `stage_count` does not
    /// equal `stage_names.len()`.
    StageCountMismatch {
        /// What the pipeline claimed.
        claimed: u32,
        /// What `stage_names.len()` actually is.
        actual: u32,
    },
    /// Structural defect: pipeline `stage_names` is empty
    /// (zero stages would mean the Layer-A pipeline is
    /// trivial).
    StageNamesEmpty,
    /// Structural defect: per-densor declaration list
    /// contains the same densor kind twice.
    DuplicateDensorKindInPipeline {
        /// The densor wire name that appears twice.
        densor_kind_wire_name: &'static str,
    },
    /// Structural defect: per-densor declaration class is
    /// `HostMaterialized` (forbidden for Layer-A).
    HostMaterializedDensorInLayerAPipeline {
        /// The densor wire name with the forbidden class.
        densor_kind_wire_name: &'static str,
    },
    /// Structural defect: residency receipt
    /// `pipeline_hash` does not equal
    /// `pipeline.layer_a_resident_pipeline_hash_v1`.
    ResidencyReceiptPipelineHashMismatch {
        /// What the receipt cited.
        claimed: [u8; 32],
        /// What the pipeline's hash actually is.
        actual: [u8; 32],
    },
    /// Structural defect: residency receipt's per-densor D2H
    /// bytes exceed the corresponding declaration's
    /// `expected_max_d2h_bytes_per_catalog`.
    D2hBytesExceedDeclaredCap {
        /// The densor wire name.
        densor_kind_wire_name: &'static str,
        /// Observed D2H bytes.
        observed: u64,
        /// Declared cap.
        declared_cap: u64,
    },
    /// Structural defect: residency receipt's
    /// `total_h2d_bytes` does not equal the sum of per-densor
    /// H2D bytes.
    TotalH2dMismatchSumOfPerDensorBytes {
        /// What the receipt claimed.
        claimed: u64,
        /// What the per-densor sum is.
        actual_sum: u64,
    },
    /// Structural defect: residency receipt's
    /// `total_d2h_bytes` does not equal the sum of per-densor
    /// D2H bytes.
    TotalD2hMismatchSumOfPerDensorBytes {
        /// What the receipt claimed.
        claimed: u64,
        /// What the per-densor sum is.
        actual_sum: u64,
    },
    /// Structural defect: court-authority anchor list does
    /// not contain `corpus_hash_v1` (the minimum required
    /// anchor that every Layer-A pipeline must promise to
    /// keep stable).
    CourtAuthorityAnchorListMissingCorpusHashV1,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf2VerifyError {
    /// Error kind (see [`SPerf2VerifyErrorKind`]).
    pub kind: SPerf2VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build a [`LayerAResidentPipelineV1`] and populate
/// `layer_a_resident_pipeline_hash_v1` from the canonical-
/// byte projection of every field.
#[must_use]
#[allow(clippy::too_many_arguments)]
// The five booleans mirror the panel-locked Layer-A
// forbidden-host-activity set verbatim (negatives #1, #2,
// host_transcript, semantic_admission, #8). Collapsing them
// into a struct would obscure the panel-locked enumeration
// and force every caller to construct an extra type.
#[allow(clippy::fn_params_excessive_bools)]
pub fn build_layer_a_resident_pipeline(
    pipeline_id: &'static str,
    stage_names: Vec<&'static str>,
    residency_declarations: Vec<LayerADensorResidencyDeclaration>,
    casefile_materialization_present: bool,
    host_transcript_present: bool,
    host_json_emission_present: bool,
    semantic_admission_present: bool,
    mutates_court_authority_hashes: bool,
) -> LayerAResidentPipelineV1 {
    let stage_count = u32::try_from(stage_names.len()).unwrap_or(u32::MAX);
    let mut p = LayerAResidentPipelineV1 {
        pipeline_id,
        stage_count,
        stage_names,
        residency_declarations,
        casefile_materialization_present,
        host_transcript_present,
        host_json_emission_present,
        semantic_admission_present,
        mutates_court_authority_hashes,
        layer_a_resident_pipeline_hash_v1: [0u8; 32],
    };
    p.layer_a_resident_pipeline_hash_v1 = compute_layer_a_resident_pipeline_hash(&p);
    p
}

/// Build a [`LayerADeviceResidencyReceiptV1`] and populate
/// `layer_a_device_residency_receipt_hash_v1`. The caller is
/// responsible for ensuring `per_densor_h2d_bytes` and
/// `per_densor_d2h_bytes` are sorted ascending by densor
/// wire name; this builder sorts them defensively so the
/// hash is canonical regardless of caller order.
#[must_use]
pub fn build_layer_a_device_residency_receipt(
    pipeline_hash: [u8; 32],
    mut per_densor_h2d_bytes: Vec<(LayerADensorKind, u64)>,
    mut per_densor_d2h_bytes: Vec<(LayerADensorKind, u64)>,
) -> LayerADeviceResidencyReceiptV1 {
    per_densor_h2d_bytes.sort_by_key(|(k, _)| *k);
    per_densor_d2h_bytes.sort_by_key(|(k, _)| *k);
    let total_h2d_bytes: u64 = per_densor_h2d_bytes
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    let total_d2h_bytes: u64 = per_densor_d2h_bytes
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    let mut r = LayerADeviceResidencyReceiptV1 {
        pipeline_hash,
        per_densor_h2d_bytes,
        per_densor_d2h_bytes,
        total_h2d_bytes,
        total_d2h_bytes,
        layer_a_device_residency_receipt_hash_v1: [0u8; 32],
    };
    r.layer_a_device_residency_receipt_hash_v1 = compute_layer_a_device_residency_receipt_hash(&r);
    r
}

/// Build a [`LayerATrafficReceiptV1`] and populate
/// `layer_a_traffic_receipt_hash_v1`.
#[must_use]
pub fn build_layer_a_traffic_receipt(
    pipeline: LayerAResidentPipelineV1,
    residency_receipt: LayerADeviceResidencyReceiptV1,
    device_traffic_receipt_hash_v1: [u8; 32],
    inner_timing_method_wire_name: &'static str,
    court_authority_hash_anchors: Vec<[u8; 32]>,
) -> LayerATrafficReceiptV1 {
    let mut r = LayerATrafficReceiptV1 {
        pipeline,
        residency_receipt,
        device_traffic_receipt_hash_v1,
        inner_timing_method_wire_name,
        court_authority_hash_anchors,
        layer_a_traffic_receipt_hash_v1: [0u8; 32],
    };
    r.layer_a_traffic_receipt_hash_v1 = compute_layer_a_traffic_receipt_hash(&r);
    r
}

// ---------------------------------------------------------------
// Seed (panel-locked baseline)
// ---------------------------------------------------------------

/// Build the panel-locked baseline Layer-A pipeline schema.
/// Declares the five canonical stages, all five densor
/// residency declarations (Evidence / Witness / Fusion =
/// `DeviceResidentOnly`; Candidate / StageDigest =
/// `DeviceResidentWithCompactD2H` with caps 2 048 / 160 bytes
/// per catalog respectively), and every forbidden-host-
/// activity flag set to `false`.
///
/// Suitable as a known-good schema reference: two builds
/// produce byte-identical
/// `layer_a_resident_pipeline_hash_v1`.
#[must_use]
pub fn seed_baseline_layer_a_pipeline() -> LayerAResidentPipelineV1 {
    let residency_declarations = vec![
        LayerADensorResidencyDeclaration {
            densor_kind: LayerADensorKind::Evidence,
            residency_class: DeviceResidencyClass::DeviceResidentOnly,
            expected_max_d2h_bytes_per_catalog: 0,
        },
        LayerADensorResidencyDeclaration {
            densor_kind: LayerADensorKind::Witness,
            residency_class: DeviceResidencyClass::DeviceResidentOnly,
            expected_max_d2h_bytes_per_catalog: 0,
        },
        LayerADensorResidencyDeclaration {
            densor_kind: LayerADensorKind::Fusion,
            residency_class: DeviceResidencyClass::DeviceResidentOnly,
            expected_max_d2h_bytes_per_catalog: 0,
        },
        LayerADensorResidencyDeclaration {
            densor_kind: LayerADensorKind::Candidate,
            residency_class: DeviceResidencyClass::DeviceResidentWithCompactD2H,
            // Panel-locked candidate-summary cap (2 048 bytes
            // per catalog); future S-PERF.* commits MAY raise
            // this as the candidate-collapse compaction
            // improves.
            expected_max_d2h_bytes_per_catalog: 2_048,
        },
        LayerADensorResidencyDeclaration {
            densor_kind: LayerADensorKind::StageDigest,
            residency_class: DeviceResidencyClass::DeviceResidentWithCompactD2H,
            // Panel-locked stage-digest cap: 5 stages ×
            // 32 bytes (SHA-256 leaf) = 160 bytes per catalog.
            expected_max_d2h_bytes_per_catalog: 160,
        },
    ];
    build_layer_a_resident_pipeline(
        "layer_a_resident_pipeline_v1_baseline",
        LAYER_A_CANONICAL_STAGE_NAMES.to_vec(),
        residency_declarations,
        false, // casefile_materialization_present
        false, // host_transcript_present
        false, // host_json_emission_present
        false, // semantic_admission_present
        false, // mutates_court_authority_hashes
    )
}

/// Build the panel-locked baseline Layer-A residency receipt.
/// References the baseline pipeline by hash; declares per-
/// densor H2D + D2H accounting with all values zero (no
/// measurement taken yet).
#[must_use]
pub fn seed_baseline_layer_a_residency_receipt() -> LayerADeviceResidencyReceiptV1 {
    let pipeline = seed_baseline_layer_a_pipeline();
    let kinds = [
        LayerADensorKind::Evidence,
        LayerADensorKind::Witness,
        LayerADensorKind::Fusion,
        LayerADensorKind::Candidate,
        LayerADensorKind::StageDigest,
    ];
    let per_densor_h2d_bytes: Vec<(LayerADensorKind, u64)> =
        kinds.iter().map(|k| (*k, 0u64)).collect();
    let per_densor_d2h_bytes: Vec<(LayerADensorKind, u64)> =
        kinds.iter().map(|k| (*k, 0u64)).collect();
    build_layer_a_device_residency_receipt(
        pipeline.layer_a_resident_pipeline_hash_v1,
        per_densor_h2d_bytes,
        per_densor_d2h_bytes,
    )
}

/// Build the panel-locked baseline Layer-A traffic receipt.
/// Composes the baseline pipeline + baseline residency
/// receipt + the S-PERF.1 baseline `DeviceTrafficReceiptV1`
/// (referenced by hash). Court-authority anchors include
/// `corpus_hash_v1`.
#[must_use]
pub fn seed_baseline_layer_a_traffic_receipt() -> LayerATrafficReceiptV1 {
    let pipeline = seed_baseline_layer_a_pipeline();
    let residency_receipt = seed_baseline_layer_a_residency_receipt();
    let s_perf_1_baseline = seed_baseline_uninstrumented_receipt();
    let corpus_anchor = compute_corpus_hash_v1().bytes;
    build_layer_a_traffic_receipt(
        pipeline,
        residency_receipt,
        s_perf_1_baseline.device_traffic_receipt_hash_v1,
        s_perf_1_baseline.timing_method.as_str(),
        vec![corpus_anchor],
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_layer_a_resident_pipeline_hash(p: &LayerAResidentPipelineV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(LAYER_A_RESIDENT_PIPELINE_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, p.pipeline_id.as_bytes());
    buf.extend_from_slice(&p.stage_count.to_be_bytes());
    let n_stages = u32::try_from(p.stage_names.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n_stages.to_be_bytes());
    for name in &p.stage_names {
        push_len_prefixed(&mut buf, name.as_bytes());
    }
    let n_decls = u32::try_from(p.residency_declarations.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n_decls.to_be_bytes());
    for d in &p.residency_declarations {
        push_len_prefixed(&mut buf, d.densor_kind.as_str().as_bytes());
        push_len_prefixed(&mut buf, d.residency_class.as_str().as_bytes());
        buf.extend_from_slice(&d.expected_max_d2h_bytes_per_catalog.to_be_bytes());
    }
    buf.push(u8::from(p.casefile_materialization_present));
    buf.push(u8::from(p.host_transcript_present));
    buf.push(u8::from(p.host_json_emission_present));
    buf.push(u8::from(p.semantic_admission_present));
    buf.push(u8::from(p.mutates_court_authority_hashes));
    sha256(&buf)
}

fn compute_layer_a_device_residency_receipt_hash(r: &LayerADeviceResidencyReceiptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(LAYER_A_DEVICE_RESIDENCY_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&r.pipeline_hash);
    let nh = u32::try_from(r.per_densor_h2d_bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&nh.to_be_bytes());
    for (k, v) in &r.per_densor_h2d_bytes {
        push_len_prefixed(&mut buf, k.as_str().as_bytes());
        buf.extend_from_slice(&v.to_be_bytes());
    }
    let nd = u32::try_from(r.per_densor_d2h_bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&nd.to_be_bytes());
    for (k, v) in &r.per_densor_d2h_bytes {
        push_len_prefixed(&mut buf, k.as_str().as_bytes());
        buf.extend_from_slice(&v.to_be_bytes());
    }
    buf.extend_from_slice(&r.total_h2d_bytes.to_be_bytes());
    buf.extend_from_slice(&r.total_d2h_bytes.to_be_bytes());
    sha256(&buf)
}

fn compute_layer_a_traffic_receipt_hash(r: &LayerATrafficReceiptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(LAYER_A_TRAFFIC_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&r.pipeline.layer_a_resident_pipeline_hash_v1);
    buf.extend_from_slice(&r.residency_receipt.layer_a_device_residency_receipt_hash_v1);
    buf.extend_from_slice(&r.device_traffic_receipt_hash_v1);
    push_len_prefixed(&mut buf, r.inner_timing_method_wire_name.as_bytes());
    push_hash_list(&mut buf, &r.court_authority_hash_anchors);
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
// Verifier --- single pipeline
// ---------------------------------------------------------------

/// Verify a Layer-A pipeline against the panel-locked
/// pipeline-side rules (negatives #1, #2, #3, #8) plus
/// structural defects (empty pipeline_id, stage-count
/// mismatch, duplicate densor kind, HostMaterialized class
/// in a Layer-A declaration).
#[must_use]
pub fn verify_layer_a_resident_pipeline(
    pipeline: &LayerAResidentPipelineV1,
) -> Vec<SPerf2VerifyError> {
    let mut errors: Vec<SPerf2VerifyError> = Vec::new();

    // Panel-required negative #1.
    if pipeline.host_json_emission_present {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::LayerAReceiptWithHostJsonTime,
        });
    }
    // Panel-required negative #2.
    if pipeline.casefile_materialization_present {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::LayerAReceiptWithCasefileMaterializationTime,
        });
    }
    // Panel-required negative #3.
    if pipeline.residency_declarations.is_empty() {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::PipelineWithoutDeviceResidencyDeclaration,
        });
    }
    // Panel-required negative #8.
    if pipeline.mutates_court_authority_hashes {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::PipelineThatMutatesCourtAuthorityHashes,
        });
    }

    // Structural defects.
    if pipeline.pipeline_id.is_empty() {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::PipelineIdEmpty,
        });
    }
    if pipeline.stage_names.is_empty() {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::StageNamesEmpty,
        });
    }
    let claimed_stage_count = pipeline.stage_count;
    let actual_stage_count = u32::try_from(pipeline.stage_names.len()).unwrap_or(u32::MAX);
    if claimed_stage_count != actual_stage_count {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::StageCountMismatch {
                claimed: claimed_stage_count,
                actual: actual_stage_count,
            },
        });
    }
    // Duplicate densor-kind check.
    let mut seen: BTreeSet<LayerADensorKind> = BTreeSet::new();
    for d in &pipeline.residency_declarations {
        if !seen.insert(d.densor_kind) {
            errors.push(SPerf2VerifyError {
                kind: SPerf2VerifyErrorKind::DuplicateDensorKindInPipeline {
                    densor_kind_wire_name: d.densor_kind.as_str(),
                },
            });
        }
        if matches!(d.residency_class, DeviceResidencyClass::HostMaterialized) {
            errors.push(SPerf2VerifyError {
                kind: SPerf2VerifyErrorKind::HostMaterializedDensorInLayerAPipeline {
                    densor_kind_wire_name: d.densor_kind.as_str(),
                },
            });
        }
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- residency receipt
// ---------------------------------------------------------------

/// Verify a Layer-A residency receipt against a referenced
/// pipeline. Surfaces panel-required negatives #4 (full
/// witness D2H dump) and #5 (missing byte accounting) plus
/// structural defects (pipeline-hash mismatch, total-sum
/// mismatch, D2H cap exceeded).
#[must_use]
pub fn verify_layer_a_device_residency_receipt(
    receipt: &LayerADeviceResidencyReceiptV1,
    pipeline: &LayerAResidentPipelineV1,
) -> Vec<SPerf2VerifyError> {
    let mut errors: Vec<SPerf2VerifyError> = Vec::new();

    // Panel-required negative #5: missing byte accounting.
    // The baseline declares zero-valued per-densor entries, so
    // the rule fires only when BOTH per-densor lists are
    // empty (the receipt did not even declare which densors
    // it accounted for).
    if receipt.per_densor_h2d_bytes.is_empty() && receipt.per_densor_d2h_bytes.is_empty() {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::MissingH2dD2hByteAccounting,
        });
    }

    // Structural: pipeline-hash mismatch.
    if receipt.pipeline_hash != pipeline.layer_a_resident_pipeline_hash_v1 {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::ResidencyReceiptPipelineHashMismatch {
                claimed: receipt.pipeline_hash,
                actual: pipeline.layer_a_resident_pipeline_hash_v1,
            },
        });
    }

    // Panel-required negative #4: full witness / fusion /
    // evidence D2H dump. For every per-densor D2H entry, look
    // up the pipeline's residency class for the matching
    // densor kind. If the class is DeviceResidentOnly and the
    // entry's bytes > 0, reject.
    for (kind, observed) in &receipt.per_densor_d2h_bytes {
        let decl = pipeline
            .residency_declarations
            .iter()
            .find(|d| d.densor_kind == *kind);
        if let Some(d) = decl {
            if matches!(d.residency_class, DeviceResidencyClass::DeviceResidentOnly)
                && *observed > 0
            {
                errors.push(SPerf2VerifyError {
                    kind: SPerf2VerifyErrorKind::D2hFullWitnessDumpWhenSummaryOnlyDeclared {
                        densor_kind_wire_name: kind.as_str(),
                        observed_d2h_bytes: *observed,
                    },
                });
            }
            // Structural: D2H exceeds declared cap.
            if d.expected_max_d2h_bytes_per_catalog > 0
                && *observed > d.expected_max_d2h_bytes_per_catalog
            {
                errors.push(SPerf2VerifyError {
                    kind: SPerf2VerifyErrorKind::D2hBytesExceedDeclaredCap {
                        densor_kind_wire_name: kind.as_str(),
                        observed: *observed,
                        declared_cap: d.expected_max_d2h_bytes_per_catalog,
                    },
                });
            }
        }
    }

    // Structural: total H2D sum mismatch.
    let actual_h2d_sum: u64 = receipt
        .per_densor_h2d_bytes
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    if actual_h2d_sum != receipt.total_h2d_bytes {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::TotalH2dMismatchSumOfPerDensorBytes {
                claimed: receipt.total_h2d_bytes,
                actual_sum: actual_h2d_sum,
            },
        });
    }
    // Structural: total D2H sum mismatch.
    let actual_d2h_sum: u64 = receipt
        .per_densor_d2h_bytes
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    if actual_d2h_sum != receipt.total_d2h_bytes {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::TotalD2hMismatchSumOfPerDensorBytes {
                claimed: receipt.total_d2h_bytes,
                actual_sum: actual_d2h_sum,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- traffic receipt
// ---------------------------------------------------------------

/// Verify a Layer-A traffic receipt against the panel-locked
/// policy. Surfaces panel-required negatives #6
/// (insufficient timing method), #7 (missing device-traffic-
/// receipt reference) plus a structural rule that the court-
/// authority anchor list MUST include `corpus_hash_v1`.
///
/// Also re-runs `verify_layer_a_resident_pipeline` on the
/// embedded pipeline and `verify_layer_a_device_residency_receipt`
/// on the embedded receipt so a single
/// `verify_layer_a_traffic_receipt` call returns every
/// applicable error.
#[must_use]
pub fn verify_layer_a_traffic_receipt(receipt: &LayerATrafficReceiptV1) -> Vec<SPerf2VerifyError> {
    let mut errors: Vec<SPerf2VerifyError> = Vec::new();

    // Panel-required negative #6: timing method must satisfy
    // S-PERF.1's Layer-A rule (CudaEvent or CudaStreamSync).
    let cuda_event = TimingMethod::CudaEvent.as_str();
    let cuda_stream_sync = TimingMethod::CudaStreamSync.as_str();
    if receipt.inner_timing_method_wire_name != cuda_event
        && receipt.inner_timing_method_wire_name != cuda_stream_sync
    {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::CudaTimingMethodNotAllowedBySPerf1 {
                observed_timing_method_wire_name: receipt.inner_timing_method_wire_name,
            },
        });
    }

    // Panel-required negative #7: device-traffic-receipt
    // reference must be non-zero.
    if receipt.device_traffic_receipt_hash_v1 == [0u8; 32] {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::LayerAClaimWithoutDeviceTrafficReceipt,
        });
    }

    // Structural: court-authority anchor list MUST include
    // corpus_hash_v1.
    let corpus_anchor = compute_corpus_hash_v1().bytes;
    if !receipt
        .court_authority_hash_anchors
        .contains(&corpus_anchor)
    {
        errors.push(SPerf2VerifyError {
            kind: SPerf2VerifyErrorKind::CourtAuthorityAnchorListMissingCorpusHashV1,
        });
    }

    // Re-run the pipeline + residency receipt verifiers so a
    // single verify call surfaces every error at once.
    errors.extend(verify_layer_a_resident_pipeline(&receipt.pipeline));
    errors.extend(verify_layer_a_device_residency_receipt(
        &receipt.residency_receipt,
        &receipt.pipeline,
    ));

    errors
}

// ---------------------------------------------------------------
// Renderers --- text
// ---------------------------------------------------------------

/// Render the Layer-A pipeline as deterministic text.
#[must_use]
pub fn render_layer_a_resident_pipeline_text(p: &LayerAResidentPipelineV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.2 LayerAResidentPipelineV1");
    let _ = writeln!(s, "=================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Identity");
    let _ = writeln!(s, "  pipeline_id : {}", p.pipeline_id);
    let _ = writeln!(s, "  stage_count : {}", p.stage_count);
    let _ = writeln!(s);
    let _ = writeln!(s, "Stages ({})", p.stage_names.len());
    for (i, name) in p.stage_names.iter().enumerate() {
        let _ = writeln!(s, "  {}. {name}", i + 1);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Per-densor residency declarations ({})",
        p.residency_declarations.len()
    );
    for d in &p.residency_declarations {
        let _ = writeln!(
            s,
            "  {} : {} (max_d2h={} bytes/catalog)",
            d.densor_kind.as_str(),
            d.residency_class.as_str(),
            d.expected_max_d2h_bytes_per_catalog
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Forbidden host activities (all must be false)");
    let _ = writeln!(
        s,
        "  casefile_materialization_present  : {}",
        p.casefile_materialization_present
    );
    let _ = writeln!(
        s,
        "  host_transcript_present           : {}",
        p.host_transcript_present
    );
    let _ = writeln!(
        s,
        "  host_json_emission_present        : {}",
        p.host_json_emission_present
    );
    let _ = writeln!(
        s,
        "  semantic_admission_present        : {}",
        p.semantic_admission_present
    );
    let _ = writeln!(
        s,
        "  mutates_court_authority_hashes    : {}",
        p.mutates_court_authority_hashes
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "layer_a_resident_pipeline_hash_v1 : {}",
        hex32(&p.layer_a_resident_pipeline_hash_v1)
    );
    s
}

/// Render the Layer-A residency receipt as deterministic text.
#[must_use]
pub fn render_layer_a_device_residency_receipt_text(r: &LayerADeviceResidencyReceiptV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.2 LayerADeviceResidencyReceiptV1");
    let _ = writeln!(s, "=======================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pipeline reference");
    let _ = writeln!(s, "  pipeline_hash : {}", hex32(&r.pipeline_hash));
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-densor H2D bytes ({})", r.per_densor_h2d_bytes.len());
    for (k, v) in &r.per_densor_h2d_bytes {
        let _ = writeln!(s, "  {} : {}", k.as_str(), v);
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-densor D2H bytes ({})", r.per_densor_d2h_bytes.len());
    for (k, v) in &r.per_densor_d2h_bytes {
        let _ = writeln!(s, "  {} : {}", k.as_str(), v);
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Totals");
    let _ = writeln!(s, "  total_h2d_bytes : {}", r.total_h2d_bytes);
    let _ = writeln!(s, "  total_d2h_bytes : {}", r.total_d2h_bytes);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "layer_a_device_residency_receipt_hash_v1 : {}",
        hex32(&r.layer_a_device_residency_receipt_hash_v1)
    );
    s
}

/// Render the Layer-A traffic receipt as deterministic text.
#[must_use]
pub fn render_layer_a_traffic_receipt_text(r: &LayerATrafficReceiptV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.2 LayerATrafficReceiptV1");
    let _ = writeln!(s, "===============================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pipeline + residency");
    let _ = writeln!(
        s,
        "  pipeline_hash                   : {}",
        hex32(&r.pipeline.layer_a_resident_pipeline_hash_v1)
    );
    let _ = writeln!(
        s,
        "  residency_receipt_hash          : {}",
        hex32(&r.residency_receipt.layer_a_device_residency_receipt_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "S-PERF.1 reference");
    let _ = writeln!(
        s,
        "  device_traffic_receipt_hash_v1  : {}",
        hex32(&r.device_traffic_receipt_hash_v1)
    );
    let _ = writeln!(
        s,
        "  inner_timing_method             : {}",
        r.inner_timing_method_wire_name
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Court-authority anchors (must include corpus_hash_v1) ({})",
        r.court_authority_hash_anchors.len()
    );
    for h in &r.court_authority_hash_anchors {
        let _ = writeln!(s, "  {}", hex32(h));
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "layer_a_traffic_receipt_hash_v1 : {}",
        hex32(&r.layer_a_traffic_receipt_hash_v1)
    );
    s
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render the Layer-A pipeline as canonical JSON.
#[must_use]
pub fn render_layer_a_resident_pipeline_json(p: &LayerAResidentPipelineV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", LAYER_A_RESIDENT_PIPELINE_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "pipeline_id", p.pipeline_id);
    s.push(',');
    let _ = write!(s, "\"stage_count\":{}", p.stage_count);
    s.push(',');
    s.push_str("\"stage_names\":[");
    for (i, name) in p.stage_names.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        json_quoted(&mut s, name);
    }
    s.push(']');
    s.push(',');
    s.push_str("\"residency_declarations\":[");
    for (i, d) in p.residency_declarations.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_string(&mut s, "densor_kind", d.densor_kind.as_str());
        s.push(',');
        json_string(&mut s, "residency_class", d.residency_class.as_str());
        s.push(',');
        let _ = write!(
            s,
            "\"expected_max_d2h_bytes_per_catalog\":{}",
            d.expected_max_d2h_bytes_per_catalog
        );
        s.push('}');
    }
    s.push(']');
    s.push(',');
    let _ = write!(
        s,
        "\"casefile_materialization_present\":{}",
        p.casefile_materialization_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_transcript_present\":{}",
        p.host_transcript_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"host_json_emission_present\":{}",
        p.host_json_emission_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"semantic_admission_present\":{}",
        p.semantic_admission_present
    );
    s.push(',');
    let _ = write!(
        s,
        "\"mutates_court_authority_hashes\":{}",
        p.mutates_court_authority_hashes
    );
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_resident_pipeline_hash_v1",
        &p.layer_a_resident_pipeline_hash_v1,
    );
    s.push('}');
    s
}

/// Render the Layer-A residency receipt as canonical JSON.
#[must_use]
pub fn render_layer_a_device_residency_receipt_json(r: &LayerADeviceResidencyReceiptV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        LAYER_A_DEVICE_RESIDENCY_RECEIPT_SCHEMA_V1,
    );
    s.push(',');
    json_hex(&mut s, "pipeline_hash", &r.pipeline_hash);
    s.push(',');
    s.push_str("\"per_densor_h2d_bytes\":[");
    for (i, (k, v)) in r.per_densor_h2d_bytes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_string(&mut s, "densor_kind", k.as_str());
        s.push(',');
        let _ = write!(s, "\"bytes\":{v}");
        s.push('}');
    }
    s.push(']');
    s.push(',');
    s.push_str("\"per_densor_d2h_bytes\":[");
    for (i, (k, v)) in r.per_densor_d2h_bytes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_string(&mut s, "densor_kind", k.as_str());
        s.push(',');
        let _ = write!(s, "\"bytes\":{v}");
        s.push('}');
    }
    s.push(']');
    s.push(',');
    let _ = write!(s, "\"total_h2d_bytes\":{}", r.total_h2d_bytes);
    s.push(',');
    let _ = write!(s, "\"total_d2h_bytes\":{}", r.total_d2h_bytes);
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_device_residency_receipt_hash_v1",
        &r.layer_a_device_residency_receipt_hash_v1,
    );
    s.push('}');
    s
}

/// Render the Layer-A traffic receipt as canonical JSON.
#[must_use]
pub fn render_layer_a_traffic_receipt_json(r: &LayerATrafficReceiptV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", LAYER_A_TRAFFIC_RECEIPT_SCHEMA_V1);
    s.push(',');
    json_hex(
        &mut s,
        "pipeline_hash",
        &r.pipeline.layer_a_resident_pipeline_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "residency_receipt_hash",
        &r.residency_receipt.layer_a_device_residency_receipt_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "device_traffic_receipt_hash_v1",
        &r.device_traffic_receipt_hash_v1,
    );
    s.push(',');
    json_string(
        &mut s,
        "inner_timing_method_wire_name",
        r.inner_timing_method_wire_name,
    );
    s.push(',');
    json_hex_list(
        &mut s,
        "court_authority_hash_anchors",
        &r.court_authority_hash_anchors,
    );
    s.push(',');
    json_hex(
        &mut s,
        "layer_a_traffic_receipt_hash_v1",
        &r.layer_a_traffic_receipt_hash_v1,
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

/// Test-only read-only access to the panel-locked canonical
/// stage-name list (used to cross-check that the constant
/// has not drifted from the in-memory baseline pipeline's
/// stage list).
#[doc(hidden)]
#[must_use]
pub fn panel_locked_layer_a_canonical_stage_names() -> &'static [&'static str] {
    LAYER_A_CANONICAL_STAGE_NAMES
}

/// Test-only helper that re-exports the device-identity hash
/// helper from S-PERF.1 so the S-PERF.2 acceptance suite can
/// construct mutated receipts without depending on S-PERF.1
/// imports directly (the suite uses this as a sanity check
/// the two crates remain wired together).
#[doc(hidden)]
#[must_use]
pub fn panel_locked_device_identity_hash_helper(name: &str, sm_arch: u32) -> [u8; 32] {
    compute_device_identity_hash(name, sm_arch)
}
