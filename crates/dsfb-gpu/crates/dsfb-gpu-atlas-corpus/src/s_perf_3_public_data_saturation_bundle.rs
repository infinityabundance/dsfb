//! S-PERF.3 --- Public-data saturation bundle.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S-PERF.3 defines the byte-pinned public artifact bundle
//! > used for future Layer-A saturation measurements. It does
//! > not claim saturation, does not benchmark throughput, and
//! > does not change kernels. It creates the reproducible
//! > public-data workload surface that S-PERF.4 / S-PERF.5
//! > will measure.**
//!
//! Core rule (panel-locked):
//!
//! > No saturation benchmark without a byte-pinned public-data
//! > bundle. No dataset claim without source, license / access
//! > status, hash policy, and fixed materialization recipe.
//!
//! ## Why
//!
//! S-PERF.1 supplied the byte-accounting receipt. S-PERF.2
//! supplied the Layer-A pipeline shape the receipt is taken
//! against. S-PERF.3 supplies the *input workload* the
//! pipeline is run on: a byte-pinned manifest of named public
//! datasets with declared source, license / access status,
//! hash policy, materialization recipe, and Layer-A role
//! mapping. Without this bundle, any "Layer-A saturation"
//! claim could silently substitute one dataset for another
//! (or invent a synthetic workload) and the saturation number
//! would mean nothing. With it, the next S-PERF.4 / S-PERF.5
//! commits can cite a hash that pins the entire workload
//! surface.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes (none folded upstream):
//!
//! - `public_artifact_manifest_hash_v1` under
//!   `DSFB-GPU-ATLAS:PUBLIC-ARTIFACT-MANIFEST:v1\0`. Pins the
//!   bytes of one public-dataset manifest record.
//! - `dataset_materialization_policy_hash_v1` under
//!   `DSFB-GPU-ATLAS:DATASET-MATERIALIZATION-POLICY:v1\0`.
//!   Pins the panel-locked 8-line policy doctrine for how
//!   public datasets are materialised reproducibly.
//! - `public_data_saturation_bundle_hash_v1` under
//!   `DSFB-GPU-ATLAS:PUBLIC-DATA-SATURATION-BUNDLE:v1\0`.
//!   Top-level META-hash binding every manifest + the
//!   materialization policy + the bundle identity.
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.3 does NOT:
//!
//! - claim that any DSFB-GPU kernel has saturated memory
//!   bandwidth (that is a separate S-PERF.4 / S-PERF.5
//!   commit gated on the device-resident pipeline running
//!   on real hardware);
//! - benchmark throughput;
//! - emit any timing receipt;
//! - change any CUDA kernel;
//! - change any court decision (S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e / S1.3f / S1.3g);
//! - mutate any upstream hash anchor (`corpus_hash_v1`,
//!   `corpus_hash_v2`, every T.11.* / T.12.* / FF.* /
//!   S1.3.* / T.12.PROV / S-PERF.1 / S-PERF.2 hash
//!   byte-identical);
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs, witness records, fusion
//!   tensors, candidate intervals, or episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate;
//! - download or fetch any dataset bytes (the baseline
//!   ships citation-only manifests; live-remote fetches are
//!   forbidden by panel-required negative #6).
//!
//! S-PERF.3 ships ONLY the bundle schema + materialization
//! policy + verifier + builder + panel-locked baseline of
//! five citation-only manifests covering the five panel-named
//! dataset classes + renderers.
//!
//! ## Panel-locked one-line verdict
//!
//! > S-PERF.2 isolated the evidence-factory path; S-PERF.3
//! > gives that path a reproducible public workload to run
//! > on.

use core::fmt::Write;
use std::collections::BTreeSet;

use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `public_artifact_manifest_hash_v1`.
/// The trailing `\0` byte mirrors the S-PERF.1 / S-PERF.2
/// discipline: it ensures the manifest hash cannot be
/// silently absorbed into a sibling domain by careless
/// concatenation.
pub const PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:PUBLIC-ARTIFACT-MANIFEST:v1\0";

/// Schema identifier for `public_artifact_manifest_hash_v1`.
pub const PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:PUBLIC-ARTIFACT-MANIFEST:v1";

/// Domain separator for
/// `dataset_materialization_policy_hash_v1`.
pub const DATASET_MATERIALIZATION_POLICY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:DATASET-MATERIALIZATION-POLICY:v1\0";

/// Schema identifier for
/// `dataset_materialization_policy_hash_v1`.
pub const DATASET_MATERIALIZATION_POLICY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:DATASET-MATERIALIZATION-POLICY:v1";

/// Domain separator for
/// `public_data_saturation_bundle_hash_v1`.
pub const PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:PUBLIC-DATA-SATURATION-BUNDLE:v1\0";

/// Schema identifier for
/// `public_data_saturation_bundle_hash_v1`.
pub const PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:PUBLIC-DATA-SATURATION-BUNDLE:v1";

// ---------------------------------------------------------------
// Panel-locked materialization policy lines
// ---------------------------------------------------------------

/// The eight-line panel-locked dataset-materialization
/// policy. The verifier enforces one rule per panel-required
/// negative; the renderer prints these verbatim so the
/// contract is human-readable AND machine-hashable.
///
/// **Do not reorder, edit, or extend without rebaselining**
/// `dataset_materialization_policy_hash_v1`. The hash is
/// canonical-byte over these lines exactly.
pub const S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES: &[&str] = &[
    "Every dataset MUST declare a source URL or DOI.",
    "Every dataset MUST declare an access note (acquisition route).",
    "Every dataset MUST declare a license or access status.",
    "Every dataset MUST declare a hash policy (Sha256OfSourceArchive / Sha256PerFileManifest / UpstreamProvidedChecksum).",
    "Every dataset MUST declare a deterministic materialization recipe.",
    "Live-remote fetches at measurement time are forbidden; bundles must reference pinned local archives only.",
    "Every dataset MUST declare a Layer-A role mapping.",
    "No bundle, manifest, or recipe may contain a benchmark claim, saturation claim, or peak-percentage claim.",
];

// ---------------------------------------------------------------
// Forbidden benchmark-claim substrings
// ---------------------------------------------------------------

/// Substrings that MUST NOT appear in any manifest free-text
/// field (display_name, access_note, materialization recipe
/// step strings) or in the bundle's identifier. The S-PERF.3
/// bundle defines the workload surface; saturation /
/// throughput / peak-percentage CLAIMS belong to S-PERF.4 /
/// S-PERF.5+ measurement commits, never the bundle
/// definition.
///
/// The scanner is case-insensitive (see
/// `contains_ascii_case_insensitive`) so phrasing variants
/// like "PEAK%" or "SaTuRaTeS" are all caught.
const S_PERF_3_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS: &[&str] = &[
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
// DatasetClass
// ---------------------------------------------------------------

/// Which panel-named workload family the dataset belongs to.
/// Layer-A measurement campaigns cover all five categories so
/// the saturation surface is representative across the
/// detector taxonomy.
///
/// Wire names are stable for the hash buffer; do not rename
/// without rebaselining `public_artifact_manifest_hash_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DatasetClass {
    /// Distributed-system trace / span / metric / log
    /// datasets (e.g. TADBench, AIOps KPI, DeepTraLog,
    /// Jaeger traces, LO2 OAuth2).
    DebugObservabilityTrace,
    /// Software-defect-mining tables (e.g. Defects4J,
    /// BugsInPy, PROMISE).
    SoftwareDefectTable,
    /// Data-science / tabular anomaly datasets (e.g.
    /// ADBench subset, missingness / category / correlation
    /// drift fixtures).
    DataScienceTabular,
    /// Time-series anomaly datasets (e.g. TSB-UAD,
    /// TimeSeriesBench).
    TimeSeriesAnomaly,
    /// Industrial / engineering-physics public fixtures
    /// (e.g. NASA PCoE, C-MAPSS, SECOM).
    IndustrialPublicFixture,
}

impl DatasetClass {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DebugObservabilityTrace => "DebugObservabilityTrace",
            Self::SoftwareDefectTable => "SoftwareDefectTable",
            Self::DataScienceTabular => "DataScienceTabular",
            Self::TimeSeriesAnomaly => "TimeSeriesAnomaly",
            Self::IndustrialPublicFixture => "IndustrialPublicFixture",
        }
    }
}

// ---------------------------------------------------------------
// HashPolicyKind
// ---------------------------------------------------------------

/// How a dataset's bytes will be hashed when the manifest is
/// materialised. The policy is declared at bundle-definition
/// time even when the bytes have not yet been downloaded
/// (citation-only manifests still declare the kind); later
/// S-PERF.* commits that actually download bytes populate
/// the corresponding hash fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashPolicyKind {
    /// One SHA-256 over the source archive (e.g. a single
    /// `.tar.gz`). Appropriate for upstream datasets shipped
    /// as a single archive.
    Sha256OfSourceArchive,
    /// One SHA-256 per file in a manifest list (the bundle
    /// declares `per_artifact_sha256_count > 0`).
    Sha256PerFileManifest,
    /// The dataset upstream publishes its own checksum file
    /// (e.g. `SHA256SUMS`); the manifest pins which checksum
    /// file is canonical.
    UpstreamProvidedChecksum,
    /// Hash policy is unknown or unspecified. Always
    /// rejected by the verifier (panel-required negative #2).
    Unknown,
}

impl HashPolicyKind {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha256OfSourceArchive => "Sha256OfSourceArchive",
            Self::Sha256PerFileManifest => "Sha256PerFileManifest",
            Self::UpstreamProvidedChecksum => "UpstreamProvidedChecksum",
            Self::Unknown => "Unknown",
        }
    }
}

// ---------------------------------------------------------------
// LicenseOrAccessStatus
// ---------------------------------------------------------------

/// The dataset's license or access posture. Many academic
/// datasets ship under permissive licenses (BSD / MIT /
/// Apache-2 / CC-BY) but some require registration or are
/// academic-research-only; the manifest declares the posture
/// so the bundle cannot silently mix license terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LicenseOrAccessStatus {
    /// Released into the public domain (no license required).
    PublicDomain,
    /// BSD 2-clause license.
    Bsd2Clause,
    /// BSD 3-clause license.
    Bsd3Clause,
    /// MIT license.
    MitLicense,
    /// Apache 2.0 license.
    Apache2,
    /// Creative Commons CC-BY (attribution required).
    CcBy,
    /// Creative Commons CC-BY-SA (share-alike).
    CcBySa,
    /// Creative Commons CC0 (no rights reserved).
    CcZero,
    /// Academic-research-only access (no commercial use).
    AcademicResearchOnly,
    /// Free download but requires registration / form
    /// submission to upstream.
    RegisteredAccess,
    /// License or access status not yet declared. Always
    /// rejected by the verifier (panel-required negative
    /// #5).
    UnknownLicense,
}

impl LicenseOrAccessStatus {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PublicDomain => "PublicDomain",
            Self::Bsd2Clause => "Bsd2Clause",
            Self::Bsd3Clause => "Bsd3Clause",
            Self::MitLicense => "MitLicense",
            Self::Apache2 => "Apache2",
            Self::CcBy => "CcBy",
            Self::CcBySa => "CcBySa",
            Self::CcZero => "CcZero",
            Self::AcademicResearchOnly => "AcademicResearchOnly",
            Self::RegisteredAccess => "RegisteredAccess",
            Self::UnknownLicense => "UnknownLicense",
        }
    }
}

// ---------------------------------------------------------------
// DatasetUsageMode
// ---------------------------------------------------------------

/// How the dataset will be used by Layer-A. Citation-only
/// datasets ship the manifest but do not move bytes through
/// the device-resident pipeline (the manifest's role is to
/// pin the future workload). MeasuredFixture datasets are
/// the bytes the pipeline actually reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatasetUsageMode {
    /// The manifest pins the dataset's identity for citation
    /// and future-work scheduling; no bytes flow through
    /// Layer-A at this bundle version.
    CitationOnly,
    /// The dataset bytes are used as a measured Layer-A
    /// fixture (the device-resident pipeline reads them).
    MeasuredFixture,
}

impl DatasetUsageMode {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CitationOnly => "CitationOnly",
            Self::MeasuredFixture => "MeasuredFixture",
        }
    }
}

// ---------------------------------------------------------------
// LayerARoleMapping
// ---------------------------------------------------------------

/// Which Layer-A densor kind the dataset feeds. Required for
/// every manifest (panel-required negative #7); the
/// `Unmapped` variant is the catch-all the verifier rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerARoleMapping {
    /// Dataset bytes become EvidenceDensor input (raw input
    /// projected into the device-resident evidence field).
    EvidenceDensorSource,
    /// Dataset is the witness-output reference (ground-truth
    /// labels Layer-A's WitnessDensor output is verified
    /// against).
    WitnessDensorReference,
    /// Dataset is the fusion-output reference.
    FusionDensorReference,
    /// Dataset is the candidate-collapse reference.
    CandidateDensorReference,
    /// Dataset is the stage-digest reference.
    StageDigestReference,
    /// Role mapping not yet declared. Always rejected by the
    /// verifier (panel-required negative #7).
    Unmapped,
}

impl LayerARoleMapping {
    /// Canonical wire name for the hash buffer + renderers.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceDensorSource => "EvidenceDensorSource",
            Self::WitnessDensorReference => "WitnessDensorReference",
            Self::FusionDensorReference => "FusionDensorReference",
            Self::CandidateDensorReference => "CandidateDensorReference",
            Self::StageDigestReference => "StageDigestReference",
            Self::Unmapped => "Unmapped",
        }
    }
}

// ---------------------------------------------------------------
// DatasetMaterializationRecipe
// ---------------------------------------------------------------

/// How the dataset is materialised reproducibly. Pins the
/// upstream source, the local path template, the ordered
/// materialization steps, the expected post-materialization
/// byte count, and a flag asserting the postprocess is
/// deterministic.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `public_artifact_manifest_hash_v1`.
#[derive(Debug, Clone)]
pub struct DatasetMaterializationRecipe {
    /// Upstream source URL or DOI (non-empty).
    pub source_url_or_doi: &'static str,
    /// Local path template the materialised bytes land at
    /// (e.g. `"fixtures/tadbench/{trace_id}.json"`).
    /// Non-empty.
    pub local_path_template: &'static str,
    /// Ordered prose steps that turn the upstream source
    /// into the local files. Non-empty.
    pub materialization_steps: Vec<&'static str>,
    /// Expected total bytes after materialization. May be
    /// `0` for `CitationOnly` manifests; MUST be non-zero
    /// for `MeasuredFixture` manifests.
    pub expected_bytes_after_materialization: u64,
    /// True iff the postprocess (decompression, parsing,
    /// canonicalisation) is deterministic. MUST be `true`
    /// for any S-PERF.3-admissible recipe; non-deterministic
    /// post-processing would invalidate replay.
    pub deterministic_postprocess: bool,
    /// True iff the recipe requires a live-remote fetch at
    /// measurement time. MUST be `false` for any S-PERF.3-
    /// admissible recipe (panel-required negative #6).
    /// Future S-PERF.* commits flip this to `true` only when
    /// the fetch is itself a pinned, deterministic
    /// operation.
    pub requires_live_remote_fetch: bool,
}

// ---------------------------------------------------------------
// PublicArtifactManifestV1
// ---------------------------------------------------------------

/// One public-dataset manifest. Carries the dataset's
/// identity, class, Layer-A role mapping, source, access /
/// license posture, hash policy, materialization recipe, and
/// the manifest hash.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `public_artifact_manifest_hash_v1`.
#[derive(Debug, Clone)]
pub struct PublicArtifactManifestV1 {
    /// Stable dataset id (e.g. `"tadbench_v1"`). Non-empty.
    pub dataset_id: &'static str,
    /// Operator-readable display name. Non-empty.
    pub display_name: &'static str,
    /// Which workload family this dataset belongs to.
    pub dataset_class: DatasetClass,
    /// Which Layer-A densor kind this dataset feeds.
    pub layer_a_role_mapping: LayerARoleMapping,
    /// Acquisition route note (e.g. `"clone from
    /// https://example.org/tadbench at tag v1.0"`).
    /// Non-empty (panel-required negative #1).
    pub access_note: &'static str,
    /// License or access posture (panel-required negative
    /// #5).
    pub license_or_access_status: LicenseOrAccessStatus,
    /// How the dataset is used by Layer-A.
    pub usage_mode: DatasetUsageMode,
    /// How the dataset will be hashed when materialised
    /// (panel-required negative #2).
    pub hash_policy_kind: HashPolicyKind,
    /// Number of per-file SHA-256 entries pinned by this
    /// manifest. Zero for `Sha256OfSourceArchive` or
    /// citation-only manifests; non-zero when the manifest
    /// ships a per-file hash list.
    pub per_artifact_sha256_count: u32,
    /// SHA-256 of the source archive (zero when not
    /// applicable or not yet materialised).
    pub source_archive_sha256: [u8; 32],
    /// True iff the dataset is synthetic (generated by code,
    /// not collected from a real-world source). Panel-
    /// required negative #3 rejects a bundle whose manifests
    /// are ALL synthetic; mixed bundles with some synthetic
    /// fixtures are fine.
    pub is_synthetic: bool,
    /// How the dataset is materialised reproducibly
    /// (panel-required negative #4).
    pub materialization_recipe: DatasetMaterializationRecipe,
    /// `public_artifact_manifest_hash_v1`. Populated by
    /// [`build_public_artifact_manifest`].
    pub public_artifact_manifest_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// DatasetMaterializationPolicyV1
// ---------------------------------------------------------------

/// The panel-locked materialization-policy doctrine. Pins
/// the 8 rules the verifier enforces and gives the policy
/// its own `dataset_materialization_policy_hash_v1` so the
/// contract is citable as a hash, not just as prose.
///
/// At S-PERF.3 baseline the policy lines equal
/// [`S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES`]
/// verbatim.
#[derive(Debug, Clone)]
pub struct DatasetMaterializationPolicyV1 {
    /// The 8 panel-locked policy lines.
    pub policy_lines: Vec<&'static str>,
    /// `dataset_materialization_policy_hash_v1`.
    pub dataset_materialization_policy_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// PublicDataSaturationBundleV1
// ---------------------------------------------------------------

/// The top-level S-PERF.3 bundle. Wraps every per-dataset
/// manifest + the materialization policy + the bundle
/// identity. One hash pins the entire workload surface.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `public_data_saturation_bundle_hash_v1`.
#[derive(Debug, Clone)]
pub struct PublicDataSaturationBundleV1 {
    /// Stable bundle identifier (e.g.
    /// `"s_perf_3_baseline_bundle_v1"`). Non-empty.
    pub bundle_id: &'static str,
    /// Per-dataset manifests (sorted ascending by
    /// `dataset_id` for canonical-byte determinism).
    pub manifests: Vec<PublicArtifactManifestV1>,
    /// The materialization policy this bundle carries.
    pub materialization_policy: DatasetMaterializationPolicyV1,
    /// `public_data_saturation_bundle_hash_v1`.
    pub public_data_saturation_bundle_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.3 rejected a manifest, materialization policy,
/// or bundle. Eight panel-required load-bearing negatives
/// plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf3VerifyErrorKind {
    /// Panel-required negative #1. Manifest declares an
    /// empty `source_url_or_doi` (on the recipe) OR an empty
    /// `access_note`.
    DatasetWithoutSourceOrAccessNote {
        /// The dataset id with the missing source / access.
        dataset_id: &'static str,
    },
    /// Panel-required negative #2. Manifest declares
    /// `hash_policy_kind == HashPolicyKind::Unknown`.
    ArtifactWithoutHashPolicy {
        /// The dataset id with the missing hash policy.
        dataset_id: &'static str,
    },
    /// Panel-required negative #3. Bundle has zero manifests
    /// OR every manifest's `is_synthetic` flag is `true`.
    BundleWithSyntheticOnlyData,
    /// Panel-required negative #4. Manifest's materialization
    /// recipe is missing required fields (empty
    /// `source_url_or_doi`, empty `local_path_template`,
    /// empty `materialization_steps`, or
    /// `expected_bytes_after_materialization == 0` when
    /// `usage_mode == MeasuredFixture`).
    DatasetWithoutMaterializationRecipe {
        /// The dataset id with the defective recipe.
        dataset_id: &'static str,
    },
    /// Panel-required negative #5. Manifest declares
    /// `license_or_access_status ==
    /// LicenseOrAccessStatus::UnknownLicense`.
    LicenseOrAccessStatusMissing {
        /// The dataset id with the missing license.
        dataset_id: &'static str,
    },
    /// Panel-required negative #6. Manifest's recipe
    /// declares `requires_live_remote_fetch == true`.
    UnpinnedDownloadOrLiveRemoteDependency {
        /// The dataset id with the live-remote fetch.
        dataset_id: &'static str,
    },
    /// Panel-required negative #7. Manifest declares
    /// `layer_a_role_mapping == LayerARoleMapping::Unmapped`.
    DatasetRoleWithoutLayerAMapping {
        /// The dataset id with the missing role mapping.
        dataset_id: &'static str,
    },
    /// Panel-required negative #8. A manifest free-text
    /// field (display_name, access_note, recipe step text)
    /// or the bundle identifier contains a forbidden
    /// benchmark-claim substring (case-insensitive scan
    /// over [`S_PERF_3_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS`]).
    BenchmarkClaimInsideBundleDefinition {
        /// Dataset id (or `"<bundle>"` if the violation is
        /// in the bundle identifier or policy lines).
        location: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Structural defect: `dataset_id` is empty.
    DatasetIdEmpty,
    /// Structural defect: `display_name` is empty.
    DisplayNameEmpty {
        /// The dataset id with the empty display name.
        dataset_id: &'static str,
    },
    /// Structural defect: `bundle_id` is empty.
    BundleIdEmpty,
    /// Structural defect: `manifests` list contains the
    /// same `dataset_id` twice.
    DuplicateDatasetIdInBundle {
        /// The dataset id that appears twice.
        dataset_id: &'static str,
    },
    /// Structural defect: `materialization_recipe`
    /// `deterministic_postprocess` is `false`. The S-PERF.3
    /// schema admits only deterministic recipes; non-
    /// deterministic post-processing would invalidate
    /// replay.
    NonDeterministicMaterializationRecipe {
        /// The dataset id with the non-deterministic recipe.
        dataset_id: &'static str,
    },
    /// Structural defect: `per_artifact_sha256_count > 0`
    /// declared but the hash-policy kind is
    /// `Sha256OfSourceArchive`. The two declarations are
    /// inconsistent.
    PerArtifactSha256CountInconsistentWithHashPolicyKind {
        /// The dataset id with the inconsistency.
        dataset_id: &'static str,
    },
    /// Structural defect: bundle's `manifests` list is not
    /// sorted ascending by `dataset_id`. Canonical-byte
    /// determinism requires sorted order.
    ManifestsNotSortedAscendingByDatasetId,
    /// Structural defect: bundle's materialization policy
    /// is empty or does not equal the panel-locked
    /// `S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES`.
    MaterializationPolicyNotPanelLocked,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf3VerifyError {
    /// Error kind (see [`SPerf3VerifyErrorKind`]).
    pub kind: SPerf3VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build a [`PublicArtifactManifestV1`] and populate
/// `public_artifact_manifest_hash_v1` from the canonical-
/// byte projection of every field.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_public_artifact_manifest(
    dataset_id: &'static str,
    display_name: &'static str,
    dataset_class: DatasetClass,
    layer_a_role_mapping: LayerARoleMapping,
    access_note: &'static str,
    license_or_access_status: LicenseOrAccessStatus,
    usage_mode: DatasetUsageMode,
    hash_policy_kind: HashPolicyKind,
    per_artifact_sha256_count: u32,
    source_archive_sha256: [u8; 32],
    is_synthetic: bool,
    materialization_recipe: DatasetMaterializationRecipe,
) -> PublicArtifactManifestV1 {
    let mut m = PublicArtifactManifestV1 {
        dataset_id,
        display_name,
        dataset_class,
        layer_a_role_mapping,
        access_note,
        license_or_access_status,
        usage_mode,
        hash_policy_kind,
        per_artifact_sha256_count,
        source_archive_sha256,
        is_synthetic,
        materialization_recipe,
        public_artifact_manifest_hash_v1: [0u8; 32],
    };
    m.public_artifact_manifest_hash_v1 = compute_public_artifact_manifest_hash(&m);
    m
}

/// Build the panel-locked
/// [`DatasetMaterializationPolicyV1`]. Always returns the
/// 8-line constant policy with its hash populated.
#[must_use]
pub fn build_panel_locked_dataset_materialization_policy() -> DatasetMaterializationPolicyV1 {
    let mut p = DatasetMaterializationPolicyV1 {
        policy_lines: S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES.to_vec(),
        dataset_materialization_policy_hash_v1: [0u8; 32],
    };
    p.dataset_materialization_policy_hash_v1 = compute_dataset_materialization_policy_hash(&p);
    p
}

/// Build a [`PublicDataSaturationBundleV1`] and populate
/// `public_data_saturation_bundle_hash_v1`. The builder
/// sorts the manifest list defensively so the canonical hash
/// is identical regardless of caller order.
#[must_use]
pub fn build_public_data_saturation_bundle(
    bundle_id: &'static str,
    mut manifests: Vec<PublicArtifactManifestV1>,
    materialization_policy: DatasetMaterializationPolicyV1,
) -> PublicDataSaturationBundleV1 {
    manifests.sort_by_key(|m| m.dataset_id);
    let mut b = PublicDataSaturationBundleV1 {
        bundle_id,
        manifests,
        materialization_policy,
        public_data_saturation_bundle_hash_v1: [0u8; 32],
    };
    b.public_data_saturation_bundle_hash_v1 = compute_public_data_saturation_bundle_hash(&b);
    b
}

// ---------------------------------------------------------------
// Seed (panel-locked baseline bundle)
// ---------------------------------------------------------------

/// Build the panel-locked baseline S-PERF.3 bundle: five
/// citation-only manifests covering the five panel-named
/// dataset classes (TADBench / Defects4J / ADBench subset /
/// TSB-UAD / NASA C-MAPSS). All manifests are
/// `usage_mode = CitationOnly` at S-PERF.3 baseline; later
/// S-PERF.* commits that actually download bytes flip the
/// mode to `MeasuredFixture` and populate per-file hashes.
///
/// Suitable as a known-good reference: two builds produce
/// byte-identical `public_data_saturation_bundle_hash_v1`.
#[must_use]
pub fn seed_baseline_public_data_saturation_bundle() -> PublicDataSaturationBundleV1 {
    let manifests = vec![
        seed_tadbench_manifest(),
        seed_defects4j_manifest(),
        seed_adbench_subset_manifest(),
        seed_tsb_uad_manifest(),
        seed_nasa_cmapss_manifest(),
    ];
    let policy = build_panel_locked_dataset_materialization_policy();
    build_public_data_saturation_bundle("s_perf_3_baseline_bundle_v1", manifests, policy)
}

/// Build the TADBench citation-only manifest. TADBench is a
/// public distributed-system trace anomaly benchmark
/// referenced in the S-PERF roadmap.
#[must_use]
pub fn seed_tadbench_manifest() -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        "tadbench_v1",
        "TADBench (distributed-system trace anomaly benchmark)",
        DatasetClass::DebugObservabilityTrace,
        LayerARoleMapping::EvidenceDensorSource,
        "Clone the TADBench repository at the pinned commit recorded in the recipe; copy the trace JSON files to the local fixture directory.",
        LicenseOrAccessStatus::Apache2,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256PerFileManifest,
        0,
        [0u8; 32],
        false,
        DatasetMaterializationRecipe {
            source_url_or_doi: "https://github.com/tadbench/tadbench",
            local_path_template: "fixtures/tadbench/{trace_id}.json",
            materialization_steps: vec![
                "git clone the tadbench repository at the pinned tag",
                "copy data/traces/*.json into fixtures/tadbench/",
                "verify per-file SHA-256 against the recipe manifest",
            ],
            expected_bytes_after_materialization: 0,
            deterministic_postprocess: true,
            requires_live_remote_fetch: false,
        },
    )
}

/// Build the Defects4J citation-only manifest. Defects4J is
/// a public Java defect dataset referenced in the S-PERF
/// roadmap (software-defect table class).
#[must_use]
pub fn seed_defects4j_manifest() -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        "defects4j_v2",
        "Defects4J v2 (Java defect dataset)",
        DatasetClass::SoftwareDefectTable,
        LayerARoleMapping::EvidenceDensorSource,
        "Clone the Defects4J repository at the pinned tag and run its bootstrap script to populate the bug database; the bootstrap is deterministic given the pinned tag.",
        LicenseOrAccessStatus::MitLicense,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256PerFileManifest,
        0,
        [0u8; 32],
        false,
        DatasetMaterializationRecipe {
            source_url_or_doi: "https://github.com/rjust/defects4j",
            local_path_template: "fixtures/defects4j/{project}_{bug_id}.csv",
            materialization_steps: vec![
                "git clone the defects4j repository at the pinned tag",
                "run the framework's bootstrap script (deterministic)",
                "export the per-bug metadata as CSV tables",
                "verify per-file SHA-256 against the recipe manifest",
            ],
            expected_bytes_after_materialization: 0,
            deterministic_postprocess: true,
            requires_live_remote_fetch: false,
        },
    )
}

/// Build the ADBench-subset citation-only manifest. ADBench
/// is the NeurIPS 2022 anomaly-detection benchmark with 57
/// tabular datasets under BSD-2 (data-science tabular
/// class).
#[must_use]
pub fn seed_adbench_subset_manifest() -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        "adbench_subset_v1",
        "ADBench subset (NeurIPS 2022 tabular anomaly benchmark)",
        DatasetClass::DataScienceTabular,
        LayerARoleMapping::EvidenceDensorSource,
        "Clone the ADBench repository at the pinned tag; select the panel-locked subset of datasets per the recipe; copy the tabular files to the local fixture directory.",
        LicenseOrAccessStatus::Bsd2Clause,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256PerFileManifest,
        0,
        [0u8; 32],
        false,
        DatasetMaterializationRecipe {
            source_url_or_doi: "https://github.com/Minqi824/ADBench",
            local_path_template: "fixtures/adbench/{dataset_name}.csv",
            materialization_steps: vec![
                "git clone the ADBench repository at the pinned tag",
                "copy the panel-locked subset of datasets to fixtures/adbench/",
                "verify per-file SHA-256 against the recipe manifest",
            ],
            expected_bytes_after_materialization: 0,
            deterministic_postprocess: true,
            requires_live_remote_fetch: false,
        },
    )
}

/// Build the TSB-UAD citation-only manifest. TSB-UAD is a
/// public time-series anomaly benchmark (time-series
/// anomaly class).
#[must_use]
pub fn seed_tsb_uad_manifest() -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        "tsb_uad_v1",
        "TSB-UAD (time-series anomaly benchmark)",
        DatasetClass::TimeSeriesAnomaly,
        LayerARoleMapping::EvidenceDensorSource,
        "Clone the TSB-UAD repository at the pinned tag; download the dataset bundle from the pinned mirror (recorded in the recipe); verify per-file SHA-256.",
        LicenseOrAccessStatus::Apache2,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256PerFileManifest,
        0,
        [0u8; 32],
        false,
        DatasetMaterializationRecipe {
            source_url_or_doi: "https://github.com/TheDatumOrg/TSB-UAD",
            local_path_template: "fixtures/tsb_uad/{series_id}.csv",
            materialization_steps: vec![
                "git clone the TSB-UAD repository at the pinned tag",
                "download the dataset bundle from the pinned mirror",
                "decompress the bundle into fixtures/tsb_uad/",
                "verify per-file SHA-256 against the recipe manifest",
            ],
            expected_bytes_after_materialization: 0,
            deterministic_postprocess: true,
            requires_live_remote_fetch: false,
        },
    )
}

/// Build the NASA C-MAPSS citation-only manifest. C-MAPSS is
/// a NASA Prognostics Center of Excellence (PCoE) public
/// industrial fixture for turbofan engine degradation
/// (industrial public fixture class).
#[must_use]
pub fn seed_nasa_cmapss_manifest() -> PublicArtifactManifestV1 {
    build_public_artifact_manifest(
        "nasa_cmapss_v1",
        "NASA PCoE C-MAPSS (turbofan engine degradation simulator)",
        DatasetClass::IndustrialPublicFixture,
        LayerARoleMapping::EvidenceDensorSource,
        "Download the C-MAPSS dataset bundle from the NASA PCoE data repository at the pinned mirror (recorded in the recipe); the bundle is public domain.",
        LicenseOrAccessStatus::PublicDomain,
        DatasetUsageMode::CitationOnly,
        HashPolicyKind::Sha256OfSourceArchive,
        0,
        [0u8; 32],
        false,
        DatasetMaterializationRecipe {
            source_url_or_doi: "https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/",
            local_path_template: "fixtures/nasa_cmapss/CMAPSSData.zip",
            materialization_steps: vec![
                "download the CMAPSSData.zip archive from the pinned NASA PCoE mirror",
                "verify the archive SHA-256 against the recipe manifest",
                "decompress into fixtures/nasa_cmapss/",
            ],
            expected_bytes_after_materialization: 0,
            deterministic_postprocess: true,
            requires_live_remote_fetch: false,
        },
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_public_artifact_manifest_hash(m: &PublicArtifactManifestV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(PUBLIC_ARTIFACT_MANIFEST_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, m.dataset_id.as_bytes());
    push_len_prefixed(&mut buf, m.display_name.as_bytes());
    push_len_prefixed(&mut buf, m.dataset_class.as_str().as_bytes());
    push_len_prefixed(&mut buf, m.layer_a_role_mapping.as_str().as_bytes());
    push_len_prefixed(&mut buf, m.access_note.as_bytes());
    push_len_prefixed(&mut buf, m.license_or_access_status.as_str().as_bytes());
    push_len_prefixed(&mut buf, m.usage_mode.as_str().as_bytes());
    push_len_prefixed(&mut buf, m.hash_policy_kind.as_str().as_bytes());
    buf.extend_from_slice(&m.per_artifact_sha256_count.to_be_bytes());
    buf.extend_from_slice(&m.source_archive_sha256);
    buf.push(u8::from(m.is_synthetic));
    push_recipe(&mut buf, &m.materialization_recipe);
    sha256(&buf)
}

fn push_recipe(buf: &mut Vec<u8>, r: &DatasetMaterializationRecipe) {
    push_len_prefixed(buf, r.source_url_or_doi.as_bytes());
    push_len_prefixed(buf, r.local_path_template.as_bytes());
    let n = u32::try_from(r.materialization_steps.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for step in &r.materialization_steps {
        push_len_prefixed(buf, step.as_bytes());
    }
    buf.extend_from_slice(&r.expected_bytes_after_materialization.to_be_bytes());
    buf.push(u8::from(r.deterministic_postprocess));
    buf.push(u8::from(r.requires_live_remote_fetch));
}

fn compute_dataset_materialization_policy_hash(p: &DatasetMaterializationPolicyV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(DATASET_MATERIALIZATION_POLICY_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(DATASET_MATERIALIZATION_POLICY_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    let n = u32::try_from(p.policy_lines.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for line in &p.policy_lines {
        push_len_prefixed(&mut buf, line.as_bytes());
    }
    sha256(&buf)
}

fn compute_public_data_saturation_bundle_hash(b: &PublicDataSaturationBundleV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(PUBLIC_DATA_SATURATION_BUNDLE_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, b.bundle_id.as_bytes());
    let n = u32::try_from(b.manifests.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for m in &b.manifests {
        buf.extend_from_slice(&m.public_artifact_manifest_hash_v1);
    }
    buf.extend_from_slice(
        &b.materialization_policy
            .dataset_materialization_policy_hash_v1,
    );
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier --- single manifest
// ---------------------------------------------------------------

/// Verify a single public-artifact manifest against the
/// panel-locked rules (negatives #1, #2, #4, #5, #6, #7, and
/// the per-manifest part of #8) plus structural defects.
#[must_use]
#[allow(clippy::too_many_lines)] // 8 panel-required negatives + structural rules
pub fn verify_public_artifact_manifest(
    manifest: &PublicArtifactManifestV1,
) -> Vec<SPerf3VerifyError> {
    let mut errors: Vec<SPerf3VerifyError> = Vec::new();

    // Panel-required negative #1.
    if manifest.materialization_recipe.source_url_or_doi.is_empty()
        || manifest.access_note.is_empty()
    {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::DatasetWithoutSourceOrAccessNote {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #2.
    if matches!(manifest.hash_policy_kind, HashPolicyKind::Unknown) {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::ArtifactWithoutHashPolicy {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #4.
    let r = &manifest.materialization_recipe;
    let recipe_defective = r.local_path_template.is_empty()
        || r.materialization_steps.is_empty()
        || (matches!(manifest.usage_mode, DatasetUsageMode::MeasuredFixture)
            && r.expected_bytes_after_materialization == 0);
    if recipe_defective {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::DatasetWithoutMaterializationRecipe {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #5.
    if matches!(
        manifest.license_or_access_status,
        LicenseOrAccessStatus::UnknownLicense
    ) {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::LicenseOrAccessStatusMissing {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #6.
    if r.requires_live_remote_fetch {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::UnpinnedDownloadOrLiveRemoteDependency {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #7.
    if matches!(manifest.layer_a_role_mapping, LayerARoleMapping::Unmapped) {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::DatasetRoleWithoutLayerAMapping {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    // Panel-required negative #8 (per-manifest scan).
    for &forbidden in S_PERF_3_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS {
        let mut scan = |text: &str| {
            if contains_ascii_case_insensitive(text, forbidden) {
                errors.push(SPerf3VerifyError {
                    kind: SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition {
                        location: manifest.dataset_id,
                        forbidden_substring: forbidden,
                    },
                });
            }
        };
        scan(manifest.display_name);
        scan(manifest.access_note);
        for step in &manifest.materialization_recipe.materialization_steps {
            scan(step);
        }
    }

    // Structural defects.
    if manifest.dataset_id.is_empty() {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::DatasetIdEmpty,
        });
    }
    if manifest.display_name.is_empty() {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::DisplayNameEmpty {
                dataset_id: manifest.dataset_id,
            },
        });
    }
    if !manifest.materialization_recipe.deterministic_postprocess {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::NonDeterministicMaterializationRecipe {
                dataset_id: manifest.dataset_id,
            },
        });
    }
    if manifest.per_artifact_sha256_count > 0
        && matches!(
            manifest.hash_policy_kind,
            HashPolicyKind::Sha256OfSourceArchive
        )
    {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::PerArtifactSha256CountInconsistentWithHashPolicyKind {
                dataset_id: manifest.dataset_id,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- whole bundle
// ---------------------------------------------------------------

/// Verify a whole [`PublicDataSaturationBundleV1`]: walks
/// every manifest through `verify_public_artifact_manifest`,
/// then applies the bundle-level rules (panel-required
/// negative #3, plus the bundle-identifier part of #8, plus
/// structural defects: empty bundle_id, duplicate dataset
/// ids, non-sorted manifests, non-panel-locked policy).
#[must_use]
#[allow(clippy::too_many_lines)] // panel-required negatives + structural; splitting obscures policy
pub fn verify_public_data_saturation_bundle(
    bundle: &PublicDataSaturationBundleV1,
) -> Vec<SPerf3VerifyError> {
    let mut errors: Vec<SPerf3VerifyError> = Vec::new();

    // Walk every manifest.
    for m in &bundle.manifests {
        errors.extend(verify_public_artifact_manifest(m));
    }

    // Panel-required negative #3.
    let all_synthetic =
        !bundle.manifests.is_empty() && bundle.manifests.iter().all(|m| m.is_synthetic);
    if bundle.manifests.is_empty() || all_synthetic {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::BundleWithSyntheticOnlyData,
        });
    }

    // Panel-required negative #8 (bundle-level scan over
    // bundle_id + policy lines).
    for &forbidden in S_PERF_3_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS {
        if contains_ascii_case_insensitive(bundle.bundle_id, forbidden) {
            errors.push(SPerf3VerifyError {
                kind: SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition {
                    location: "<bundle_id>",
                    forbidden_substring: forbidden,
                },
            });
        }
        for line in &bundle.materialization_policy.policy_lines {
            if contains_ascii_case_insensitive(line, forbidden) {
                errors.push(SPerf3VerifyError {
                    kind: SPerf3VerifyErrorKind::BenchmarkClaimInsideBundleDefinition {
                        location: "<policy_line>",
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
    }

    // Structural defects.
    if bundle.bundle_id.is_empty() {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::BundleIdEmpty,
        });
    }
    // Duplicate dataset_id detection.
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for m in &bundle.manifests {
        if !seen.insert(m.dataset_id) {
            errors.push(SPerf3VerifyError {
                kind: SPerf3VerifyErrorKind::DuplicateDatasetIdInBundle {
                    dataset_id: m.dataset_id,
                },
            });
        }
    }
    // Sorted-ascending invariant.
    for w in bundle.manifests.windows(2) {
        if w[0].dataset_id > w[1].dataset_id {
            errors.push(SPerf3VerifyError {
                kind: SPerf3VerifyErrorKind::ManifestsNotSortedAscendingByDatasetId,
            });
            break;
        }
    }
    // Panel-locked policy match.
    if bundle.materialization_policy.policy_lines != S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES {
        errors.push(SPerf3VerifyError {
            kind: SPerf3VerifyErrorKind::MaterializationPolicyNotPanelLocked,
        });
    }

    errors
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Case-insensitive substring scan. Matches the helper used
/// by T.12.PROV's `DsfbInventionClaimForPriorDetector`
/// scanner; identical implementation kept local so this
/// module has no cross-module helper dependency for hot
/// code.
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

/// Render a single manifest as deterministic text.
#[must_use]
pub fn render_public_artifact_manifest_text(m: &PublicArtifactManifestV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.3 PublicArtifactManifestV1");
    let _ = writeln!(s, "=================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Identity");
    let _ = writeln!(s, "  dataset_id          : {}", m.dataset_id);
    let _ = writeln!(s, "  display_name        : {}", m.display_name);
    let _ = writeln!(s, "  dataset_class       : {}", m.dataset_class.as_str());
    let _ = writeln!(
        s,
        "  layer_a_role        : {}",
        m.layer_a_role_mapping.as_str()
    );
    let _ = writeln!(s, "  usage_mode          : {}", m.usage_mode.as_str());
    let _ = writeln!(s, "  is_synthetic        : {}", m.is_synthetic);
    let _ = writeln!(s);
    let _ = writeln!(s, "Access + license");
    let _ = writeln!(s, "  access_note         : {}", m.access_note);
    let _ = writeln!(
        s,
        "  license_or_access   : {}",
        m.license_or_access_status.as_str()
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Hash policy");
    let _ = writeln!(
        s,
        "  hash_policy_kind             : {}",
        m.hash_policy_kind.as_str()
    );
    let _ = writeln!(
        s,
        "  per_artifact_sha256_count    : {}",
        m.per_artifact_sha256_count
    );
    let _ = writeln!(
        s,
        "  source_archive_sha256        : {}",
        hex32(&m.source_archive_sha256)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Materialization recipe");
    let _ = writeln!(
        s,
        "  source_url_or_doi              : {}",
        m.materialization_recipe.source_url_or_doi
    );
    let _ = writeln!(
        s,
        "  local_path_template            : {}",
        m.materialization_recipe.local_path_template
    );
    let _ = writeln!(
        s,
        "  expected_bytes_after_materialization : {}",
        m.materialization_recipe
            .expected_bytes_after_materialization
    );
    let _ = writeln!(
        s,
        "  deterministic_postprocess      : {}",
        m.materialization_recipe.deterministic_postprocess
    );
    let _ = writeln!(
        s,
        "  requires_live_remote_fetch     : {}",
        m.materialization_recipe.requires_live_remote_fetch
    );
    let _ = writeln!(
        s,
        "  materialization_steps ({})",
        m.materialization_recipe.materialization_steps.len()
    );
    for (i, step) in m
        .materialization_recipe
        .materialization_steps
        .iter()
        .enumerate()
    {
        let _ = writeln!(s, "    {}. {step}", i + 1);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "public_artifact_manifest_hash_v1 : {}",
        hex32(&m.public_artifact_manifest_hash_v1)
    );
    s
}

/// Render the materialization policy as deterministic text.
#[must_use]
pub fn render_dataset_materialization_policy_text(p: &DatasetMaterializationPolicyV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.3 DatasetMaterializationPolicyV1");
    let _ = writeln!(s, "=======================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Panel-locked policy lines ({})", p.policy_lines.len());
    for (i, line) in p.policy_lines.iter().enumerate() {
        let _ = writeln!(s, "  {}. {line}", i + 1);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "dataset_materialization_policy_hash_v1 : {}",
        hex32(&p.dataset_materialization_policy_hash_v1)
    );
    s
}

/// Render the bundle as deterministic text (header + one
/// per-manifest summary line).
#[must_use]
pub fn render_public_data_saturation_bundle_text(b: &PublicDataSaturationBundleV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.3 PublicDataSaturationBundleV1");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Bundle identity");
    let _ = writeln!(s, "  bundle_id : {}", b.bundle_id);
    let _ = writeln!(s);
    let _ = writeln!(s, "Manifests ({})", b.manifests.len());
    for m in &b.manifests {
        let _ = writeln!(
            s,
            "  {} : class={} layer_a={} usage={} license={}",
            m.dataset_id,
            m.dataset_class.as_str(),
            m.layer_a_role_mapping.as_str(),
            m.usage_mode.as_str(),
            m.license_or_access_status.as_str()
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "materialization_policy_hash : {}",
        hex32(
            &b.materialization_policy
                .dataset_materialization_policy_hash_v1
        )
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "public_data_saturation_bundle_hash_v1 : {}",
        hex32(&b.public_data_saturation_bundle_hash_v1)
    );
    s
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render a single manifest as canonical JSON.
#[must_use]
pub fn render_public_artifact_manifest_json(m: &PublicArtifactManifestV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", PUBLIC_ARTIFACT_MANIFEST_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "dataset_id", m.dataset_id);
    s.push(',');
    json_string(&mut s, "display_name", m.display_name);
    s.push(',');
    json_string(&mut s, "dataset_class", m.dataset_class.as_str());
    s.push(',');
    json_string(
        &mut s,
        "layer_a_role_mapping",
        m.layer_a_role_mapping.as_str(),
    );
    s.push(',');
    json_string(&mut s, "access_note", m.access_note);
    s.push(',');
    json_string(
        &mut s,
        "license_or_access_status",
        m.license_or_access_status.as_str(),
    );
    s.push(',');
    json_string(&mut s, "usage_mode", m.usage_mode.as_str());
    s.push(',');
    json_string(&mut s, "hash_policy_kind", m.hash_policy_kind.as_str());
    s.push(',');
    let _ = write!(
        s,
        "\"per_artifact_sha256_count\":{}",
        m.per_artifact_sha256_count
    );
    s.push(',');
    json_hex(&mut s, "source_archive_sha256", &m.source_archive_sha256);
    s.push(',');
    let _ = write!(s, "\"is_synthetic\":{}", m.is_synthetic);
    s.push(',');
    s.push_str("\"materialization_recipe\":{");
    json_string(
        &mut s,
        "source_url_or_doi",
        m.materialization_recipe.source_url_or_doi,
    );
    s.push(',');
    json_string(
        &mut s,
        "local_path_template",
        m.materialization_recipe.local_path_template,
    );
    s.push(',');
    s.push_str("\"materialization_steps\":[");
    for (i, step) in m
        .materialization_recipe
        .materialization_steps
        .iter()
        .enumerate()
    {
        if i > 0 {
            s.push(',');
        }
        json_quoted(&mut s, step);
    }
    s.push(']');
    s.push(',');
    let _ = write!(
        s,
        "\"expected_bytes_after_materialization\":{}",
        m.materialization_recipe
            .expected_bytes_after_materialization
    );
    s.push(',');
    let _ = write!(
        s,
        "\"deterministic_postprocess\":{}",
        m.materialization_recipe.deterministic_postprocess
    );
    s.push(',');
    let _ = write!(
        s,
        "\"requires_live_remote_fetch\":{}",
        m.materialization_recipe.requires_live_remote_fetch
    );
    s.push('}');
    s.push(',');
    json_hex(
        &mut s,
        "public_artifact_manifest_hash_v1",
        &m.public_artifact_manifest_hash_v1,
    );
    s.push('}');
    s
}

/// Render the materialization policy as canonical JSON.
#[must_use]
pub fn render_dataset_materialization_policy_json(p: &DatasetMaterializationPolicyV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        DATASET_MATERIALIZATION_POLICY_SCHEMA_V1,
    );
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
        "dataset_materialization_policy_hash_v1",
        &p.dataset_materialization_policy_hash_v1,
    );
    s.push('}');
    s
}

/// Render the bundle as canonical JSON (header + per-
/// manifest summary array).
#[must_use]
pub fn render_public_data_saturation_bundle_json(b: &PublicDataSaturationBundleV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", PUBLIC_DATA_SATURATION_BUNDLE_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "bundle_id", b.bundle_id);
    s.push(',');
    let _ = write!(s, "\"manifest_count\":{}", b.manifests.len());
    s.push(',');
    s.push_str("\"manifest_dataset_ids\":[");
    for (i, m) in b.manifests.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        json_quoted(&mut s, m.dataset_id);
    }
    s.push(']');
    s.push(',');
    json_hex(
        &mut s,
        "materialization_policy_hash",
        &b.materialization_policy
            .dataset_materialization_policy_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "public_data_saturation_bundle_hash_v1",
        &b.public_data_saturation_bundle_hash_v1,
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

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Test-only read-only access to the panel-locked policy
/// lines for cross-checking that the constant has not
/// drifted from the in-memory builder output.
#[doc(hidden)]
#[must_use]
pub fn panel_locked_dataset_materialization_policy_lines() -> &'static [&'static str] {
    S_PERF_3_DATASET_MATERIALIZATION_POLICY_LINES
}

/// Test-only read-only access to the forbidden benchmark-
/// claim substring set.
#[doc(hidden)]
#[must_use]
pub fn forbidden_benchmark_claim_substrings() -> &'static [&'static str] {
    S_PERF_3_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS
}
