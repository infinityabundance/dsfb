//! Structural schema for the chemical-engineering soft-sensor data corpus.
//!
//! Mirrors the DSFB-GPU-Atlas canonicalisation-court discipline: `&'static` records in a `const`
//! table (byte-identical builds), and a mandatory [`SourceRef`] on every entry — a provenance-bearing
//! catalogue, not a dataset zoo. Each record describes a *public* soft-sensor dataset (cheap sensors
//! inferring a hard-to-measure target); **no dataset bytes are vendored** and access/licence is flagged
//! per entry.

/// Citation / provenance for a public dataset. Every record MUST carry one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SourceRef {
    /// Short citation key (e.g. `fortuna2007soft`).
    pub citation_key: &'static str,
    /// Originating work / depositor.
    pub authors: &'static str,
    /// Year (0 = "engineering benchmark, no dated primary source").
    pub year: u16,
    /// Venue / repository descriptor.
    pub venue_or_source: &'static str,
    /// Direct access URL (repository, Kaggle, UCI, Zenodo, GitHub).
    pub url: &'static str,
    /// Licence / usage terms (or "research-use; no explicit licence").
    pub license: &'static str,
}

/// Coarse process domain. New domains are added by commit, not mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ProcessDomain {
    RefineryDistillation,
    MineralProcessing,
    Power,
    Wastewater,
    Bioprocess,
    MultiphaseFlow,
    Semiconductor,
    GasSensing,
    PulpPaper,
    Emissions,
    Food,
    GeneralProcess,
}

/// What the hard-to-measure target is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TargetKind {
    Composition,
    Concentration,
    Quality,
    Emissions,
    Energy,
    RemovalEfficiency,
    FaultLabel,
}

/// How obtainable the dataset is (honesty flag; we never redistribute the bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AccessStatus {
    /// Directly downloadable, open licence.
    OpenDownload,
    /// Public but needs a Kaggle account.
    KaggleAccount,
    /// Gated behind a data-use agreement / request form.
    Gated,
    /// Code/simulator is open; the user generates the data by running it.
    CodeGeneratesData,
}

/// Whether DSFB has actually *run* a deterministic witness on this dataset, or it is catalogued only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ImplementationStatus {
    Executed,
    Catalogued,
}

// ---------------------------------------------------------------------------
// Provenance classification tiers (schema v1, added P53).
//
// Four orthogonal, hash-sealed *disclosure* axes over each record's licence and source. They are
// honest confidence/policy statements derived from the cited `license` string + `url` host + venue —
// NOT legal opinions and NOT data-quality judgements. They exist so a downstream user can see, at a
// glance and reproducibly, how verifiable a dataset's terms are and what they must respect before
// reusing it. `validation::census()` counts every tier; a gate asserts each axis partitions all
// records (no gaps). Folding them into `corpus_hash_v1` seals the classification as part of the
// authority. New tiers are added by commit + deliberate re-freeze, never by mutation.
// ---------------------------------------------------------------------------

/// How confident the catalogue is in a dataset's *stated* licence terms.
///
/// Honest disclosure, not a legal opinion: it records how verifiable the licence is from the cited
/// source, so a downstream user knows where to look before relying on it. The crate vendors no bytes
/// regardless of this value (see [`RedistributionPolicy`]) — a permissive tier never implies this
/// crate redistributes anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LicenseConfidence {
    /// A named, individually-verifiable open licence on the source page (e.g. CC0, CC BY 4.0).
    ExplicitOpen,
    /// A named copyleft licence (e.g. GPL via a packaged distribution) — reuse carries obligations.
    ExplicitCopyleft,
    /// A licence/term is stated but the exact variant should be confirmed on the page ("verify variant").
    StatedNeedsVerification,
    /// No explicit licence; research / academic use is the customary, stated posture.
    ResearchUseCustomary,
    /// Governed by a data-use agreement the recipient signs (terms set by the provider).
    AgreementGoverned,
}

/// How robust the *access route* to the bytes is (longevity / friction), independent of licence.
///
/// Honest disclosure: a strong tier does not assert the bytes were re-fetched in this artifact, only
/// that the cited route is the kind that is normally stable and obtainable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AccessConfidence {
    /// Open download from a curated repository or DOI archive with a stable identifier.
    OpenConfirmed,
    /// Open, but via a single-party mirror / vendor host whose longevity is less certain.
    OpenMirrorUnverified,
    /// Public but requires a platform account (e.g. Kaggle).
    AccountRequired,
    /// No bytes to fetch: the user runs reference simulator code to generate the data.
    GeneratedByCode,
    /// Gated behind a signed data-use agreement.
    AgreementRequired,
}

/// What a *downstream* recipient must respect before redistributing the dataset bytes.
///
/// Orthogonal to [`SoftSensorDatasetRecordV1::redistributed`], which is always `false` — **this crate
/// never ships dataset bytes**. This axis documents the *upstream* permission so a downstream user who
/// obtains the data knows the constraint; it is disclosure, not an action this crate takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RedistributionPolicy {
    /// Upstream licence permits redistribution with attribution (CC0 / CC BY family).
    UpstreamPermitsAttribution,
    /// Copyleft / share-alike: redistribution must carry the same licence (e.g. GPL).
    UpstreamCopyleftShareAlike,
    /// Terms unclear or unverified — do not redistribute without confirming upstream permission.
    UpstreamVerifyBeforeRedistribution,
    /// A data-use agreement prohibits redistribution outright.
    ProhibitedByAgreement,
}

/// What kind of authority vouches for the bytes — a provenance-robustness axis, **not** a data-quality
/// judgement.
///
/// Listed strongest-provenance first (immutable DOI) to weakest-longevity last (single-party host); the
/// ordering describes how durable / traceable the source is, nothing about the science on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SourceAuthorityKind {
    /// DOI-bearing research archive (Harvard Dataverse, Zenodo, Mendeley Data) — immutable identifier.
    DoiArchive,
    /// Curated institutional ML dataset repository with stable IDs (UCI, OpenML).
    CuratedMlRepository,
    /// Shipped inside a maintained software-package distribution (e.g. a CRAN package).
    PackageDistribution,
    /// Reference simulator / benchmark codebase the user runs to regenerate the data.
    SimulatorCodebase,
    /// Institutional testbed released under governance / a data-use agreement.
    GovernedTestbed,
    /// Community upload platform (e.g. Kaggle).
    CommunityUpload,
    /// Author, vendor, or community mirror host — weakest longevity guarantee.
    AuthorOrVendorHost,
}

/// One curated soft-sensor dataset record (schema v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SoftSensorDatasetRecordV1 {
    /// Stable short key (e.g. `"swat"`); the canonical identity/sort key folded into `corpus_hash_v1`.
    pub dataset_id: &'static str,
    /// Human-readable dataset name for reports and the datasets appendix.
    pub name: &'static str,
    /// The process domain this dataset belongs to (`ProcessDomain`) — a breadth/coverage axis.
    pub domain: ProcessDomain,
    /// The cheap, ubiquitous input sensor channels (no spectrometer assumed unless noted).
    pub cheap_sensors: &'static [&'static str],
    /// Count of `cheap_sensors` — pinned explicitly (and folded into the hash) so a miscount can't slip through.
    pub n_cheap_inputs: u32,
    /// True if the inputs are cheap ubiquitous sensors (the DSFB thesis); false if the dataset relies
    /// on spectroscopy (NIR/Raman) inputs — included for contrast, not as the cheap-sensor target.
    pub cheap_sensor: bool,
    /// The hard-to-measure target variable.
    pub target: &'static str,
    /// What kind of quantity the target is (`TargetKind`) — concentration, quality index, efficiency, etc.
    pub target_kind: TargetKind,
    /// How the target is normally obtained (lab assay / online analyzer) — the cost the soft sensor avoids.
    pub target_measurement: &'static str,
    /// Whether the dataset carries labelled faults (for the fault-signature side).
    pub has_fault_labels: bool,
    /// Input -> target dead time / multi-rate note (a first-class soft-sensor feature).
    pub input_target_lag: &'static str,
    /// Sampling cadence / rate note for the inputs (e.g. `"1 Hz"`, `"per-batch"`).
    pub sampling: &'static str,
    /// How a DSFB *deterministic* witness would read this (documented relationship / signature),
    /// as opposed to a probabilistic regressor.
    pub deterministic_inference: &'static str,
    /// Coarse access tier (`AccessStatus`) — open vs gated; the human-facing summary of the four axes below.
    pub access: AccessStatus,
    /// Always false: this crate never redistributes dataset bytes; it catalogues + seals provenance.
    pub redistributed: bool,
    /// `Executed` (used by a project witness/eval) vs `Catalogued` (provenance-sealed, not yet run here).
    pub implementation_status: ImplementationStatus,
    /// Confidence in the stated licence (honest disclosure; see [`LicenseConfidence`]).
    pub license_confidence: LicenseConfidence,
    /// Robustness of the access route, independent of licence (see [`AccessConfidence`]).
    pub access_confidence: AccessConfidence,
    /// Upstream redistribution constraint a downstream user must respect (see [`RedistributionPolicy`];
    /// orthogonal to `redistributed`, which is always `false` here — this crate ships no bytes).
    pub redistribution_policy: RedistributionPolicy,
    /// What kind of authority vouches for the bytes (see [`SourceAuthorityKind`]).
    pub source_authority: SourceAuthorityKind,
    /// Citation + canonical URL for the dataset (`SourceRef`) — the provenance pointer (no bytes shipped).
    pub source: SourceRef,
}
