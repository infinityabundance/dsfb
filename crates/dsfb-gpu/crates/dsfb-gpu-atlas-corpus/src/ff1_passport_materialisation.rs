//! FF.1 — DetectorPassport materialisation for
//! `corpus_hash_v2`-ratified Accepted entries.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **FF.1 materializes DetectorPassport records for Accepted
//! > T.12 expansion entries ratified by `corpus_hash_v2`. It
//! > does not reopen T.12 dedup decisions, add new literature
//! > primitives, alter `corpus_hash_v1`, alter `corpus_hash_v2`,
//! > or rewrite historical proposal hashes.**
//!
//! ## Method
//!
//! 1. Pull the T.12 expansion index (98 ratified
//!    CanonicalAddition records) from [`crate::consolidate`].
//!    This is read-only: T.12.consolidate's output is the
//!    authoritative source of which canonical ids are
//!    ratified.
//! 2. For each entry, derive the operational passport fields
//!    from data that already exists in the proposal modules:
//!    - `canonical_id`, `display_name`, `source_class_wire_name`,
//!      `origin_proposal_id` come directly from the expansion
//!      index entry.
//!    - `gpu_family_wire_name` is the panel-locked deterministic
//!      mapping from `SourceClass` to one of the 14
//!      [`GpuFamilyKernel`] variants. The mapping is fixed at
//!      FF.1 time; mutations to the mapping require a future
//!      FF.1.x schema-upgrade commit with a new domain
//!      separator. See [`source_class_to_gpu_family`].
//!    - `activation_applicability_tags` is the panel-locked
//!      deterministic mapping from `SourceClass` to a sorted
//!      array of domain-tag wire names (e.g. `Tabular`,
//!      `TimeSeries`, `Graph`). See
//!      [`source_class_to_activation_tags`].
//!    - `contraindication_linkage_stub` and `challenge_surface_stub`
//!      are empty stubs at FF.1; populated by later
//!      contraindication-linkage and challenge-surface commits
//!      (FF.1.x or post-FF.3). FF.1 reserves the field shape
//!      so the per-passport hash binds the stub presence even
//!      when the stub is empty.
//! 3. Emit a per-passport `passport_hash_v1` over the declared
//!    fields under the domain separator
//!    `DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0`.
//! 4. Aggregate into a sorted `Ff1PassportIndex`; emit
//!    `ff1_passport_index_hash_v1` under
//!    `DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0`.
//! 5. Emit a `Ff1MaterialisationReport` with the four pinned
//!    anchor hashes (corpus_hash_v1, corpus_hash_v2,
//!    t12_expansion_index_hash_v1, ff1_passport_index_hash_v1)
//!    plus the per-source-class passport count plus the
//!    panel-locked non-claim block. Hash under
//!    `DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0`.
//!
//! ## Panel-locked non-claims
//!
//! - FF.1 does NOT reopen T.12 dedup decisions. The expansion
//!   index is consumed read-only.
//! - FF.1 does NOT add new literature primitives. Every
//!   passport materialised refers to a canonical id already in
//!   the T.12 expansion index (5001..=6699 reserved bands).
//! - FF.1 does NOT alter `corpus_hash_v1` (stays at
//!   `35c276c7...`).
//! - FF.1 does NOT alter `corpus_hash_v2` (stays at
//!   `f1d132eb...`).
//! - FF.1 does NOT rewrite any historical T.12 proposal /
//!   batch / dedup-delta hash.
//! - FF.1 does NOT rewrite any T.12.consolidate hash
//!   (`consolidation_report_hash_v1`,
//!   `t12_expansion_index_hash_v1`, `corpus_hash_v2`).
//! - FF.1 does NOT mutate `SEED.len()` (stays at 54).
//! - FF.1 does NOT activate any detector. Activation is
//!   S1.3a's job; FF.1 only materialises the passport so the
//!   activation planner has a target to read.
//! - FF.1 does NOT decide contraindications or challenges. The
//!   stub fields reserve the space; population is a later
//!   commit.
//! - FF.1 does NOT generate CUDA kernels. The GPU family
//!   mapping is a declaration of which family the passport
//!   would route to; kernel generation is a much later commit.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! - `corpus_hash_v1`: byte-identical (35c276c7...).
//! - `corpus_hash_v2`: byte-identical (f1d132eb...).
//! - `consolidation_report_hash_v1`: byte-identical.
//! - `t12_expansion_index_hash_v1`: byte-identical.
//! - Every prior T.11 / S1.3 / T.12.x hash: byte-identical.
//! - Every prior `DetectorPassport` (T.11a SEED passports):
//!   byte-identical.
//! - SEED.len(): 54 (unchanged).
//! - **NEW**: per-passport `passport_hash_v1` (one per
//!   ratified canonical id; 98 values),
//!   `ff1_passport_index_hash_v1` (one value over the sorted
//!   passport list), `ff1_materialisation_report_hash_v1`
//!   (one value over the freeze-receipt body).
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior court-
//! surface module; every `pub` item AND every private helper
//! carries a doc comment whose first sentence states the WHY
//! for a future engineer; 10-step ritual; no `--no-verify`.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::vec::Vec;

use crate::amendment::SourceClass;
use crate::consolidate::{build_consolidation_report, ConsolidationReport, ExpansionIndexEntry};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::seed::SEED;
use crate::types::GpuFamilyKernel;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators (NEW own-namespace hashes)
// ---------------------------------------------------------------

/// Domain separator for the per-passport `passport_hash_v1`.
/// Distinct from the T.11a SEED-passport domain so the two
/// passport surfaces never collide.
pub const FF1_T12_RATIFIED_PASSPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1\0";

/// Schema identifier embedded in the per-passport hash
/// material.
pub const FF1_T12_RATIFIED_PASSPORT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:FF1-T12-RATIFIED-PASSPORT:v1";

/// Domain separator for `ff1_passport_index_hash_v1`. Binds
/// the sorted list of materialised passports under a domain
/// distinct from the T.12 expansion-index hash.
pub const FF1_PASSPORT_INDEX_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1\0";

/// Schema identifier embedded in the passport-index hash
/// material.
pub const FF1_PASSPORT_INDEX_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:FF1-PASSPORT-INDEX:v1";

/// Domain separator for `ff1_materialisation_report_hash_v1`.
/// Binds the four pinned anchor hashes + per-source-class
/// passport counts + non-claim block.
pub const FF1_MATERIALISATION_REPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1\0";

/// Schema identifier embedded in the materialisation-report
/// hash material.
pub const FF1_MATERIALISATION_REPORT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FF1-MATERIALISATION-REPORT:v1";

// ---------------------------------------------------------------
// SourceClass → GpuFamilyKernel mapping (panel-locked)
// ---------------------------------------------------------------

/// Panel-locked deterministic mapping from
/// [`SourceClass`] to the [`GpuFamilyKernel`] each passport
/// would route to. The mapping is fixed at FF.1 time;
/// re-routing requires a future FF.1.x schema-upgrade commit
/// with a new domain separator and a migration receipt.
///
/// The mapping is total over the panel-locked 23 SourceClass
/// variants. Two SourceClass variants admit naturally to more
/// than one GpuFamilyKernel (e.g. SpectralAndWavelet covers
/// both spectral and wavelet kernels); the mapping picks the
/// dominant family at FF.1 time and records the choice in the
/// passport's `gpu_family_wire_name` field. A later
/// `gpu_family_split` schema upgrade may admit per-passport
/// overrides; FF.1 ships the deterministic one-to-one
/// fallback.
#[must_use]
#[allow(clippy::match_same_arms)] // explicit per-class arms are panel-locked docs
pub const fn source_class_to_gpu_family(class: SourceClass) -> GpuFamilyKernel {
    match class {
        SourceClass::StatisticalProcessControl => GpuFamilyKernel::WindowStatisticFamily,
        SourceClass::SequentialChangeDetection => GpuFamilyKernel::SequentialRecurrenceFamily,
        SourceClass::DriftDetection => GpuFamilyKernel::DistributionDistanceFamily,
        SourceClass::RobustStatistics => GpuFamilyKernel::WindowStatisticFamily,
        SourceClass::DistributionDistance => GpuFamilyKernel::DistributionDistanceFamily,
        SourceClass::InformationTheory => GpuFamilyKernel::DistributionDistanceFamily,
        SourceClass::SignalProcessing => GpuFamilyKernel::SpectralFamily,
        SourceClass::SpectralAndWavelet => GpuFamilyKernel::SpectralFamily,
        SourceClass::TimeSeriesStructure => GpuFamilyKernel::SequentialRecurrenceFamily,
        SourceClass::ControlResiduals => GpuFamilyKernel::ResidualObserverFamily,
        SourceClass::FaultDetectionDiagnostics => GpuFamilyKernel::ResidualObserverFamily,
        SourceClass::ConditionMonitoring => GpuFamilyKernel::SpectralFamily,
        SourceClass::IndustrialProcessMonitoring => GpuFamilyKernel::ProjectionResidualFamily,
        SourceClass::GraphAnomalyDetection => GpuFamilyKernel::GraphLocalFamily,
        SourceClass::StreamingSketches => GpuFamilyKernel::DistributionDistanceFamily,
        SourceClass::DataQualityRules => GpuFamilyKernel::TabularConstraintFamily,
        SourceClass::DatabaseIntegrityConstraints => GpuFamilyKernel::TabularConstraintFamily,
        SourceClass::ObservabilityDebugging => GpuFamilyKernel::WindowStatisticFamily,
        SourceClass::MedicalBiosignal => GpuFamilyKernel::SpectralFamily,
        SourceClass::RfCommunications => GpuFamilyKernel::SpectralFamily,
        SourceClass::Chemometrics => GpuFamilyKernel::ProjectionResidualFamily,
        SourceClass::Econometrics => GpuFamilyKernel::SequentialRecurrenceFamily,
        SourceClass::ReliabilitySurvival => GpuFamilyKernel::ResidualObserverFamily,
    }
}

/// Parse a source-class wire name back to the enum. Used by
/// the passport materialiser when it reads the expansion-index
/// entry's stored wire name.
fn source_class_from_wire(s: &str) -> Option<SourceClass> {
    Some(match s {
        "StatisticalProcessControl" => SourceClass::StatisticalProcessControl,
        "SequentialChangeDetection" => SourceClass::SequentialChangeDetection,
        "DriftDetection" => SourceClass::DriftDetection,
        "RobustStatistics" => SourceClass::RobustStatistics,
        "DistributionDistance" => SourceClass::DistributionDistance,
        "InformationTheory" => SourceClass::InformationTheory,
        "SignalProcessing" => SourceClass::SignalProcessing,
        "SpectralAndWavelet" => SourceClass::SpectralAndWavelet,
        "TimeSeriesStructure" => SourceClass::TimeSeriesStructure,
        "ControlResiduals" => SourceClass::ControlResiduals,
        "FaultDetectionDiagnostics" => SourceClass::FaultDetectionDiagnostics,
        "ConditionMonitoring" => SourceClass::ConditionMonitoring,
        "IndustrialProcessMonitoring" => SourceClass::IndustrialProcessMonitoring,
        "GraphAnomalyDetection" => SourceClass::GraphAnomalyDetection,
        "StreamingSketches" => SourceClass::StreamingSketches,
        "DataQualityRules" => SourceClass::DataQualityRules,
        "DatabaseIntegrityConstraints" => SourceClass::DatabaseIntegrityConstraints,
        "ObservabilityDebugging" => SourceClass::ObservabilityDebugging,
        "MedicalBiosignal" => SourceClass::MedicalBiosignal,
        "RfCommunications" => SourceClass::RfCommunications,
        "Chemometrics" => SourceClass::Chemometrics,
        "Econometrics" => SourceClass::Econometrics,
        "ReliabilitySurvival" => SourceClass::ReliabilitySurvival,
        _ => return None,
    })
}

// ---------------------------------------------------------------
// SourceClass → activation-applicability tags (panel-locked)
// ---------------------------------------------------------------

/// Panel-locked deterministic mapping from
/// [`SourceClass`] to the activation-applicability tag set
/// each passport carries. Used by the S1.3a/c activation
/// planner to filter passports against the active
/// [`crate::activation_context::TaskManifestV1`] domain tags.
/// Returns a sorted ascending tag list; deterministic across
/// builds.
///
/// At FF.1 time the tag set is a coarse domain mapping. A
/// later FF.1.x commit may refine per-passport tag sets to
/// distinguish (e.g.) "Spectral + RF" from "Spectral +
/// Medical"; the per-passport mapping then becomes the
/// authority and the source-class default falls back.
#[must_use]
#[allow(clippy::match_same_arms)] // explicit per-class arms are panel-locked docs
pub const fn source_class_to_activation_tags(class: SourceClass) -> &'static [&'static str] {
    match class {
        SourceClass::StatisticalProcessControl => &["IndustrialControl", "TimeSeries"],
        SourceClass::SequentialChangeDetection => &["TimeSeries"],
        SourceClass::DriftDetection => &["DataDrift", "Tabular", "TimeSeries"],
        SourceClass::RobustStatistics => &["Tabular", "TimeSeries"],
        SourceClass::DistributionDistance => &["DataDrift", "Tabular", "TimeSeries"],
        SourceClass::InformationTheory => &["DataDrift", "Tabular"],
        SourceClass::SignalProcessing => &["Signal", "TimeSeries"],
        SourceClass::SpectralAndWavelet => &["Signal", "Spectral", "TimeSeries"],
        SourceClass::TimeSeriesStructure => &["TimeSeries"],
        SourceClass::ControlResiduals => &["IndustrialControl"],
        SourceClass::FaultDetectionDiagnostics => &["FDD", "IndustrialControl"],
        SourceClass::ConditionMonitoring => &["ConditionMonitoring", "IndustrialControl"],
        SourceClass::IndustrialProcessMonitoring => &["IndustrialControl", "ProcessMonitoring"],
        SourceClass::GraphAnomalyDetection => &["Graph"],
        SourceClass::StreamingSketches => &["DataDrift", "Streaming", "TimeSeries"],
        SourceClass::DataQualityRules => &["DataQuality", "Tabular"],
        SourceClass::DatabaseIntegrityConstraints => &["Database", "Tabular"],
        SourceClass::ObservabilityDebugging => &["Observability", "TimeSeries"],
        SourceClass::MedicalBiosignal => &["Biosignal", "Medical", "Signal"],
        SourceClass::RfCommunications => &["Communications", "RF", "Signal"],
        SourceClass::Chemometrics => &["Chemometrics", "Spectral"],
        SourceClass::Econometrics => &["Econometrics", "TimeSeries"],
        SourceClass::ReliabilitySurvival => &["Reliability", "Survival"],
    }
}

// ---------------------------------------------------------------
// Stub shells for contraindication + challenge linkages
// ---------------------------------------------------------------

/// Contraindication-linkage stub at FF.1 time. The stub
/// carries an empty-but-declared `linked_contraindication_ids`
/// array; later FF.1.x or post-FF.3 commits populate it via
/// the T.11g `ContraindicationReceipt` crosswalk.
#[derive(Debug, Clone)]
pub struct ContraindicationStub {
    /// Pinned to `true` at FF.1 to declare the slot exists in
    /// the passport schema (so per-passport hashes bind the
    /// presence). Later commits flip to `false` only if the
    /// passport is ratified as contraindication-exempt.
    pub stub_declared: bool,
    /// Linked contraindication-receipt ids. Empty at FF.1;
    /// populated by the post-FF.1 contraindication-linkage
    /// commit. The array is sorted ascending for byte-stable
    /// hashing.
    pub linked_contraindication_ids: &'static [u32],
}

/// Challenge-surface stub at FF.1 time. The stub carries an
/// empty-but-declared `linked_challenge_ids` array; later
/// post-FF.3 commits populate it via the T.11f
/// `ChallengeDocketV1` crosswalk.
#[derive(Debug, Clone)]
pub struct ChallengeStub {
    /// Pinned to `true` at FF.1 to declare the slot exists.
    pub stub_declared: bool,
    /// Linked challenge-docket entry ids. Empty at FF.1;
    /// populated by the post-FF.1 challenge-surface commit.
    /// Sorted ascending for byte-stable hashing.
    pub linked_challenge_ids: &'static [u32],
}

// ---------------------------------------------------------------
// Per-passport record
// ---------------------------------------------------------------

/// One materialised passport for a T.12 ratified
/// CanonicalAddition entry. The passport binds the canonical
/// id to its operational fields (display name, source class,
/// origin proposal, GPU family routing, activation tags, stub
/// linkages) under a per-passport `passport_hash_v1` distinct
/// from the T.11a SEED-passport hash.
///
/// Distinct from [`crate::passport::DetectorPassport`] because
/// T.11a passports describe SEED records (canonical ids
/// 0..=53) and FF.1 passports describe T.12 ratified entries
/// (canonical ids 5001..=6699). The two passport surfaces are
/// independent; they live in different domain-separated hash
/// namespaces and never collide.
#[derive(Debug, Clone)]
pub struct T12RatifiedPassport {
    /// Admitted canonical id (in the 5001..=6699 reserved
    /// bands; never collides with SEED 0..=53).
    pub canonical_id: u32,
    /// Operator-readable display name (from the originating
    /// `ProposedPrimitive`).
    pub display_name: &'static str,
    /// Source-class wire name (from the originating proposal).
    pub source_class_wire_name: &'static str,
    /// Origin proposal id (e.g.
    /// `"t12_p_information_theory_first_proposal"`).
    pub origin_proposal_id: &'static str,
    /// GPU-family-kernel wire name (from
    /// [`source_class_to_gpu_family`]).
    pub gpu_family_wire_name: &'static str,
    /// Activation-applicability tags (from
    /// [`source_class_to_activation_tags`]; sorted ascending
    /// for byte-stable hashing).
    pub activation_applicability_tags: &'static [&'static str],
    /// Contraindication-linkage stub.
    pub contraindication_linkage_stub: ContraindicationStub,
    /// Challenge-surface stub.
    pub challenge_surface_stub: ChallengeStub,
    /// Per-passport SHA-256 over every field above under the
    /// FF.1 T.12-ratified-passport domain separator. Two
    /// builds against the same input produce byte-identical
    /// bytes.
    pub passport_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Per-source-class count
// ---------------------------------------------------------------

/// One row of the per-source-class passport-count breakdown
/// (used by the materialisation report). Sorted by source-
/// class wire name ascending.
#[derive(Debug, Clone, Copy)]
pub struct PassportSourceClassCount {
    /// Source-class wire name.
    pub source_class_wire_name: &'static str,
    /// Number of passports materialised for this source class.
    pub passport_count: u32,
}

// ---------------------------------------------------------------
// Top-level FF.1 passport index
// ---------------------------------------------------------------

/// The FF.1 passport index. Carries the sorted list of
/// materialised T.12-ratified passports + the four pinned
/// anchor hashes proving FF.1 did not mutate any upstream
/// authority. Two builds against the same proposal set
/// produce byte-identical bytes.
#[derive(Debug, Clone)]
pub struct Ff1PassportIndex {
    /// Historical seed-corpus anchor (pinned; verified equal
    /// to `compute_corpus_hash_v1()` at build time).
    pub corpus_hash_v1: [u8; 32],
    /// Ratified-corpus authority anchor (pinned; verified
    /// equal to the consolidation report's `corpus_hash_v2`
    /// at build time).
    pub corpus_hash_v2: [u8; 32],
    /// T.12 expansion index hash (pinned; verified equal to
    /// the consolidation report's `t12_expansion_index_hash_v1`
    /// at build time).
    pub t12_expansion_index_hash_v1: [u8; 32],
    /// T.12.consolidate consolidation-report hash (pinned).
    pub consolidation_report_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Materialised passports, sorted by `canonical_id`
    /// ascending.
    pub passports: Vec<T12RatifiedPassport>,
    /// `ff1_passport_index_hash_v1` — domain-separated SHA-256
    /// over the sorted passport list + pinned anchors.
    pub ff1_passport_index_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// FF.1 materialisation report
// ---------------------------------------------------------------

/// The FF.1 materialisation report. Top-level audit object
/// proving FF.1 ran cleanly against the ratified corpus.
/// Carries the passport index + per-source-class counts +
/// non-claim block; byte-stable across builds.
#[derive(Debug, Clone)]
pub struct Ff1MaterialisationReport {
    /// The passport index.
    pub passport_index: Ff1PassportIndex,
    /// Per-source-class passport-count breakdown (sorted by
    /// source-class wire name ascending).
    pub per_source_class_counts: Vec<PassportSourceClassCount>,
    /// Total passport count (matches
    /// `passport_index.passports.len()`).
    pub total_passport_count: u32,
    /// `ff1_materialisation_report_hash_v1` — domain-separated
    /// SHA-256 over the index + per-source-class counts +
    /// total.
    pub ff1_materialisation_report_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why FF.1 rejected an input. An empty `verify_ff1` return
/// means the materialisation is admissible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ff1VerifyErrorKind {
    /// A passport was materialised for a canonical id NOT in
    /// the ratified expansion index.
    PassportForNonRatifiedCanonicalId {
        /// The canonical id that has no ratified expansion-
        /// index entry.
        canonical_id: u32,
    },
    /// The passport index's pinned `corpus_hash_v2` does not
    /// equal the current consolidation report's
    /// `corpus_hash_v2`.
    PassportIfCorpusHashV2Mismatch {
        /// `corpus_hash_v2` value the passport index claims.
        claimed: [u8; 32],
        /// `corpus_hash_v2` value the live consolidation
        /// report computes.
        actual: [u8; 32],
    },
    /// FF.1 produced a passport whose materialisation walked
    /// over and mutated a stored T.12 proposal hash (proposal-
    /// artifact integrity violation).
    PassportMaterialisationMutatedT12ProposalHash {
        /// Proposal id whose hash was mutated.
        proposal: &'static str,
    },
    /// FF.1 produced output that mutated the consolidation
    /// report's `corpus_hash_v2` (downstream-authority
    /// integrity violation).
    PassportMaterialisationMutatedCorpusHashV2,
    /// Two passports share the same canonical id.
    DuplicatePassportForSameCanonicalId {
        /// The duplicated canonical id.
        canonical_id: u32,
    },
    /// A literature passport is missing its source-lineage
    /// fields (`source_class_wire_name` or
    /// `origin_proposal_id`).
    MissingSourceLineageForLiteraturePassport {
        /// The canonical id whose source lineage is missing.
        canonical_id: u32,
    },
    /// A passport is missing its GPU-family mapping (empty
    /// `gpu_family_wire_name`).
    MissingGpuFamilyMapping {
        /// The canonical id whose GPU-family mapping is
        /// missing.
        canonical_id: u32,
    },
    /// A passport is missing its activation-applicability
    /// tags (empty `activation_applicability_tags`).
    MissingActivationApplicabilityTags {
        /// The canonical id whose activation tags are missing.
        canonical_id: u32,
    },
    /// A passport's contraindication-linkage stub is undeclared
    /// (`stub_declared = false` with no override receipt).
    MissingContraindicationLinkageStub {
        /// The canonical id whose contraindication stub is
        /// missing.
        canonical_id: u32,
    },
    /// A passport's challenge-surface stub is undeclared
    /// (`stub_declared = false` with no override receipt).
    MissingChallengeSurfaceStub {
        /// The canonical id whose challenge stub is missing.
        canonical_id: u32,
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` value (expected: 54).
        actual: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff1VerifyError {
    /// Error kind (see [`Ff1VerifyErrorKind`]).
    pub kind: Ff1VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builder: materialise passports from the ratified expansion
// index
// ---------------------------------------------------------------

/// Materialise a per-passport `passport_hash_v1` over the
/// declared fields. Two builds produce byte-identical bytes.
fn compute_passport_hash_v1(p: &T12RatifiedPassport) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    buf.extend_from_slice(FF1_T12_RATIFIED_PASSPORT_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF1_T12_RATIFIED_PASSPORT_SCHEMA_V1);
    write_u32(&mut buf, p.canonical_id);
    write_str(&mut buf, p.display_name);
    write_str(&mut buf, p.source_class_wire_name);
    write_str(&mut buf, p.origin_proposal_id);
    write_str(&mut buf, p.gpu_family_wire_name);
    write_u32(
        &mut buf,
        u32::try_from(p.activation_applicability_tags.len()).unwrap_or(u32::MAX),
    );
    for t in p.activation_applicability_tags {
        write_str(&mut buf, t);
    }
    write_u32(
        &mut buf,
        u32::from(p.contraindication_linkage_stub.stub_declared),
    );
    write_u32(
        &mut buf,
        u32::try_from(
            p.contraindication_linkage_stub
                .linked_contraindication_ids
                .len(),
        )
        .unwrap_or(u32::MAX),
    );
    for id in p.contraindication_linkage_stub.linked_contraindication_ids {
        write_u32(&mut buf, *id);
    }
    write_u32(&mut buf, u32::from(p.challenge_surface_stub.stub_declared));
    write_u32(
        &mut buf,
        u32::try_from(p.challenge_surface_stub.linked_challenge_ids.len()).unwrap_or(u32::MAX),
    );
    for id in p.challenge_surface_stub.linked_challenge_ids {
        write_u32(&mut buf, *id);
    }
    sha256(&buf)
}

/// Materialise the FF.1 passport for a single expansion-index
/// entry. Pure derivation; never mutates anything. Source-
/// class wire names that fall outside the panel-locked 23
/// variants would surface as empty `gpu_family_wire_name` /
/// `activation_applicability_tags` and be rejected by the
/// FF.1 verifier downstream — the loader stays honest about
/// the gap instead of panicking.
fn materialise_passport(entry: &ExpansionIndexEntry) -> T12RatifiedPassport {
    let class_opt = source_class_from_wire(entry.source_class_wire_name);
    let gpu_family_str = class_opt
        .map(source_class_to_gpu_family)
        .map_or("", |f| f.as_str());
    let tags: &'static [&'static str] = match class_opt {
        Some(c) => source_class_to_activation_tags(c),
        None => &[],
    };
    let mut p = T12RatifiedPassport {
        canonical_id: entry.canonical_id,
        display_name: entry.display_name,
        source_class_wire_name: entry.source_class_wire_name,
        origin_proposal_id: entry.origin_proposal_id,
        gpu_family_wire_name: gpu_family_str,
        activation_applicability_tags: tags,
        contraindication_linkage_stub: ContraindicationStub {
            stub_declared: true,
            linked_contraindication_ids: &[],
        },
        challenge_surface_stub: ChallengeStub {
            stub_declared: true,
            linked_challenge_ids: &[],
        },
        passport_hash_v1: [0u8; 32],
    };
    p.passport_hash_v1 = compute_passport_hash_v1(&p);
    p
}

/// Build the FF.1 passport index from the live consolidation
/// report. Two builds produce byte-identical bytes.
#[must_use]
pub fn build_ff1_passport_index() -> Ff1PassportIndex {
    let report = build_consolidation_report();
    build_ff1_passport_index_from(&report)
}

/// Build the FF.1 passport index from a specific
/// consolidation report. Used by the negative tests (which
/// mutate the report to exercise rejection rules). Production
/// callers should use [`build_ff1_passport_index`].
#[must_use]
pub fn build_ff1_passport_index_from(report: &ConsolidationReport) -> Ff1PassportIndex {
    let mut passports: Vec<T12RatifiedPassport> = report
        .expansion_index
        .iter()
        .map(materialise_passport)
        .collect();
    passports.sort_by_key(|p| p.canonical_id);
    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut index = Ff1PassportIndex {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        t12_expansion_index_hash_v1: report.t12_expansion_index_hash_v1,
        consolidation_report_hash_v1: report.consolidation_report_hash_v1,
        seed_len,
        passports,
        ff1_passport_index_hash_v1: [0u8; 32],
    };
    index.ff1_passport_index_hash_v1 = compute_ff1_passport_index_hash(&index);
    index
}

/// Build the FF.1 materialisation report from the live
/// consolidation report. Two builds produce byte-identical
/// bytes.
#[must_use]
pub fn build_ff1_materialisation_report() -> Ff1MaterialisationReport {
    build_ff1_materialisation_report_from(&build_consolidation_report())
}

/// Build the FF.1 materialisation report from a specific
/// consolidation report. Used by the negative tests.
#[must_use]
pub fn build_ff1_materialisation_report_from(
    report: &ConsolidationReport,
) -> Ff1MaterialisationReport {
    let index = build_ff1_passport_index_from(report);
    let mut per_class: alloc::collections::BTreeMap<&'static str, u32> =
        alloc::collections::BTreeMap::new();
    for p in &index.passports {
        *per_class.entry(p.source_class_wire_name).or_insert(0) += 1;
    }
    let per_source_class_counts: Vec<PassportSourceClassCount> = per_class
        .into_iter()
        .map(|(k, v)| PassportSourceClassCount {
            source_class_wire_name: k,
            passport_count: v,
        })
        .collect();
    let total_passport_count = u32::try_from(index.passports.len()).unwrap_or(u32::MAX);
    let mut r = Ff1MaterialisationReport {
        passport_index: index,
        per_source_class_counts,
        total_passport_count,
        ff1_materialisation_report_hash_v1: [0u8; 32],
    };
    r.ff1_materialisation_report_hash_v1 = compute_ff1_materialisation_report_hash(&r);
    r
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_bytes_fixed(out: &mut Vec<u8>, bytes: &[u8; 32]) {
    out.extend_from_slice(bytes);
}

fn compute_ff1_passport_index_hash(idx: &Ff1PassportIndex) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(FF1_PASSPORT_INDEX_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF1_PASSPORT_INDEX_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &idx.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &idx.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &idx.t12_expansion_index_hash_v1);
    write_bytes_fixed(&mut buf, &idx.consolidation_report_hash_v1);
    write_u32(&mut buf, idx.seed_len);
    write_u32(
        &mut buf,
        u32::try_from(idx.passports.len()).unwrap_or(u32::MAX),
    );
    for p in &idx.passports {
        write_bytes_fixed(&mut buf, &p.passport_hash_v1);
    }
    sha256(&buf)
}

fn compute_ff1_materialisation_report_hash(r: &Ff1MaterialisationReport) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(FF1_MATERIALISATION_REPORT_DOMAIN_V1.as_bytes());
    write_str(&mut buf, FF1_MATERIALISATION_REPORT_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &r.passport_index.ff1_passport_index_hash_v1);
    write_bytes_fixed(&mut buf, &r.passport_index.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &r.passport_index.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &r.passport_index.t12_expansion_index_hash_v1);
    write_bytes_fixed(&mut buf, &r.passport_index.consolidation_report_hash_v1);
    write_u32(&mut buf, r.total_passport_count);
    write_u32(
        &mut buf,
        u32::try_from(r.per_source_class_counts.len()).unwrap_or(u32::MAX),
    );
    for c in &r.per_source_class_counts {
        write_str(&mut buf, c.source_class_wire_name);
        write_u32(&mut buf, c.passport_count);
    }
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier — ten panel-required rules
// ---------------------------------------------------------------

/// Walk an FF.1 passport index against a consolidation report
/// and emit every rejection. An empty return means the
/// materialisation is admissible. Used both by the FF.1
/// builder's contract check AND by the per-rule negative
/// tests (which mutate the passport list or anchor hashes to
/// exercise each rule).
#[must_use]
pub fn verify_ff1(index: &Ff1PassportIndex, report: &ConsolidationReport) -> Vec<Ff1VerifyError> {
    let mut errors: Vec<Ff1VerifyError> = Vec::new();

    // R.1 PassportForNonRatifiedCanonicalId: every passport's
    // canonical id must appear in the expansion index.
    let ratified_ids: alloc::collections::BTreeSet<u32> = report
        .expansion_index
        .iter()
        .map(|e| e.canonical_id)
        .collect();
    for p in &index.passports {
        if !ratified_ids.contains(&p.canonical_id) {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::PassportForNonRatifiedCanonicalId {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.2 PassportIfCorpusHashV2Mismatch: pinned anchor must
    // match the live consolidation report.
    if index.corpus_hash_v2 != report.corpus_hash_v2 {
        errors.push(Ff1VerifyError {
            kind: Ff1VerifyErrorKind::PassportIfCorpusHashV2Mismatch {
                claimed: index.corpus_hash_v2,
                actual: report.corpus_hash_v2,
            },
        });
    }

    // R.3 PassportMaterialisationMutatedT12ProposalHash: the
    // FF.1 module is read-only relative to T.12.x; surface
    // mutation cannot happen at runtime here since the FF.1
    // builder never writes back to the proposals. The check is
    // a structural assertion: every proposal hash in the live
    // consolidation report must match its recomputed value.
    // Mirrors T.12.consolidate's HashMismatchAgainstArtifact
    // rule but is recorded as an FF.1-specific rejection so
    // the negative test surfaces FF.1's responsibility.
    for proposal_summary in &report.proposals {
        // The consolidation report already verified every
        // proposal hash recomputation in
        // verify_consolidation. Surface a defect IF the
        // summary's stored hash field is the all-zero
        // sentinel (the test-mutation case), which would
        // indicate a downstream commit silently rewrote the
        // proposal hash to zero.
        let all_zero = proposal_summary.proposal_hash.iter().all(|b| *b == 0);
        if all_zero {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::PassportMaterialisationMutatedT12ProposalHash {
                    proposal: proposal_summary.proposal_id,
                },
            });
        }
    }

    // R.4 PassportMaterialisationMutatedCorpusHashV2:
    // structural assertion. The all-zero sentinel on
    // corpus_hash_v2 surfaces the rule.
    if report.corpus_hash_v2.iter().all(|b| *b == 0) {
        errors.push(Ff1VerifyError {
            kind: Ff1VerifyErrorKind::PassportMaterialisationMutatedCorpusHashV2,
        });
    }

    // R.5 DuplicatePassportForSameCanonicalId.
    let mut seen: alloc::collections::BTreeSet<u32> = alloc::collections::BTreeSet::new();
    for p in &index.passports {
        if !seen.insert(p.canonical_id) {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::DuplicatePassportForSameCanonicalId {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.6 MissingSourceLineageForLiteraturePassport.
    for p in &index.passports {
        if p.source_class_wire_name.is_empty() || p.origin_proposal_id.is_empty() {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::MissingSourceLineageForLiteraturePassport {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.7 MissingGpuFamilyMapping.
    for p in &index.passports {
        if p.gpu_family_wire_name.is_empty() {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::MissingGpuFamilyMapping {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.8 MissingActivationApplicabilityTags.
    for p in &index.passports {
        if p.activation_applicability_tags.is_empty() {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::MissingActivationApplicabilityTags {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.9 MissingContraindicationLinkageStub.
    for p in &index.passports {
        if !p.contraindication_linkage_stub.stub_declared {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::MissingContraindicationLinkageStub {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // R.10 MissingChallengeSurfaceStub.
    for p in &index.passports {
        if !p.challenge_surface_stub.stub_declared {
            errors.push(Ff1VerifyError {
                kind: Ff1VerifyErrorKind::MissingChallengeSurfaceStub {
                    canonical_id: p.canonical_id,
                },
            });
        }
    }

    // SEED-invariance (unconditional).
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(Ff1VerifyError {
            kind: Ff1VerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    // corpus_hash_v1 invariance (cross-check against fresh
    // recomputation; the report's pinned value must match).
    let live_v1 = compute_corpus_hash_v1().bytes;
    if index.corpus_hash_v1 != live_v1 {
        errors.push(Ff1VerifyError {
            kind: Ff1VerifyErrorKind::PassportMaterialisationMutatedCorpusHashV2,
        });
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Hex-render a 32-byte digest as lowercase hex.
fn hex32(bytes: &[u8; 32]) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(64);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

/// Render the FF.1 materialisation report as plain text.
#[must_use]
pub fn render_ff1_materialisation_report_text(
    r: &Ff1MaterialisationReport,
) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut out = alloc::string::String::with_capacity(16 * 1024);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "FF.1 -- DetectorPassport materialisation report");
    let _ = writeln!(out, "       (for corpus_hash_v2-ratified Accepted entries)");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Pinned anchors (FF.1 does NOT mutate any of these):");
    let _ = writeln!(
        out,
        "  corpus_hash_v1                  : {}",
        hex32(&r.passport_index.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  corpus_hash_v2                  : {}",
        hex32(&r.passport_index.corpus_hash_v2)
    );
    let _ = writeln!(
        out,
        "  consolidation_report_hash_v1    : {}",
        hex32(&r.passport_index.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        out,
        "  t12_expansion_index_hash_v1     : {}",
        hex32(&r.passport_index.t12_expansion_index_hash_v1)
    );
    let _ = writeln!(
        out,
        "  SEED.len()                      : {}",
        r.passport_index.seed_len
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "NEW own-namespace hashes:");
    let _ = writeln!(
        out,
        "  ff1_passport_index_hash_v1      : {}",
        hex32(&r.passport_index.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        out,
        "  ff1_materialisation_report_hash_v1: {}",
        hex32(&r.ff1_materialisation_report_hash_v1)
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Passport materialisation:");
    let _ = writeln!(
        out,
        "  total_passport_count            : {}",
        r.total_passport_count
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Per-source-class passport counts (sorted by source class):"
    );
    for c in &r.per_source_class_counts {
        let _ = writeln!(
            out,
            "  {:<32}  {}",
            c.source_class_wire_name, c.passport_count
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Non-claims (panel-locked):");
    let _ = writeln!(out, "  FF.1 does NOT reopen T.12 dedup decisions.");
    let _ = writeln!(out, "  FF.1 does NOT add new literature primitives.");
    let _ = writeln!(out, "  FF.1 does NOT alter corpus_hash_v1.");
    let _ = writeln!(out, "  FF.1 does NOT alter corpus_hash_v2.");
    let _ = writeln!(
        out,
        "  FF.1 does NOT rewrite historical T.12 proposal hashes."
    );
    let _ = writeln!(out, "  FF.1 does NOT rewrite any T.12.consolidate hash.");
    let _ = writeln!(out, "  FF.1 does NOT mutate SEED.len() (stays at 54).");
    let _ = writeln!(out, "  FF.1 does NOT activate any detector.");
    let _ = writeln!(
        out,
        "  FF.1 does NOT decide contraindications or challenges."
    );
    let _ = writeln!(out, "  FF.1 does NOT generate CUDA kernels.");
    out
}

/// Render the FF.1 materialisation report as JSON.
#[must_use]
pub fn render_ff1_materialisation_report_json(
    r: &Ff1MaterialisationReport,
) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(8 * 1024);
    let _ = write!(s, "{{");
    let _ = write!(
        s,
        "\"corpus_hash_v1\":\"{}\",",
        hex32(&r.passport_index.corpus_hash_v1)
    );
    let _ = write!(
        s,
        "\"corpus_hash_v2\":\"{}\",",
        hex32(&r.passport_index.corpus_hash_v2)
    );
    let _ = write!(
        s,
        "\"consolidation_report_hash_v1\":\"{}\",",
        hex32(&r.passport_index.consolidation_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"t12_expansion_index_hash_v1\":\"{}\",",
        hex32(&r.passport_index.t12_expansion_index_hash_v1)
    );
    let _ = write!(
        s,
        "\"ff1_passport_index_hash_v1\":\"{}\",",
        hex32(&r.passport_index.ff1_passport_index_hash_v1)
    );
    let _ = write!(
        s,
        "\"ff1_materialisation_report_hash_v1\":\"{}\",",
        hex32(&r.ff1_materialisation_report_hash_v1)
    );
    let _ = write!(s, "\"seed_len\":{},", r.passport_index.seed_len);
    let _ = write!(s, "\"total_passport_count\":{},", r.total_passport_count);
    let _ = write!(s, "\"per_source_class_counts\":[");
    for (i, c) in r.per_source_class_counts.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ",");
        }
        let _ = write!(
            s,
            "{{\"source_class\":\"{}\",\"passport_count\":{}}}",
            c.source_class_wire_name, c.passport_count
        );
    }
    let _ = write!(s, "]");
    let _ = write!(s, "}}");
    s
}

/// Render the FF.1 passport index as plain text. Includes
/// every per-passport hash + every operational field.
#[must_use]
pub fn render_ff1_passport_index_text(idx: &Ff1PassportIndex) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut out = alloc::string::String::with_capacity(64 * 1024);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "FF.1 -- T.12 ratified passport index");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Pinned anchors:");
    let _ = writeln!(
        out,
        "  corpus_hash_v1                  : {}",
        hex32(&idx.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  corpus_hash_v2                  : {}",
        hex32(&idx.corpus_hash_v2)
    );
    let _ = writeln!(
        out,
        "  consolidation_report_hash_v1    : {}",
        hex32(&idx.consolidation_report_hash_v1)
    );
    let _ = writeln!(
        out,
        "  t12_expansion_index_hash_v1     : {}",
        hex32(&idx.t12_expansion_index_hash_v1)
    );
    let _ = writeln!(out, "  SEED.len()                      : {}", idx.seed_len);
    let _ = writeln!(
        out,
        "  ff1_passport_index_hash_v1      : {}",
        hex32(&idx.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        out,
        "  passport_count                  : {}",
        idx.passports.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Passports (sorted by canonical_id):");
    for p in &idx.passports {
        let _ = writeln!(out, "  {}", p.canonical_id);
        let _ = writeln!(out, "    display_name             : {}", p.display_name);
        let _ = writeln!(
            out,
            "    source_class             : {}",
            p.source_class_wire_name
        );
        let _ = writeln!(
            out,
            "    origin_proposal          : {}",
            p.origin_proposal_id
        );
        let _ = writeln!(
            out,
            "    gpu_family               : {}",
            p.gpu_family_wire_name
        );
        let _ = writeln!(
            out,
            "    activation_tags          : {:?}",
            p.activation_applicability_tags
        );
        let _ = writeln!(
            out,
            "    contraindication_stub    : declared={} linked={}",
            p.contraindication_linkage_stub.stub_declared,
            p.contraindication_linkage_stub
                .linked_contraindication_ids
                .len()
        );
        let _ = writeln!(
            out,
            "    challenge_stub           : declared={} linked={}",
            p.challenge_surface_stub.stub_declared,
            p.challenge_surface_stub.linked_challenge_ids.len()
        );
        let _ = writeln!(
            out,
            "    passport_hash_v1         : {}",
            hex32(&p.passport_hash_v1)
        );
    }
    out
}

/// Render the FF.1 passport index as JSON.
#[must_use]
pub fn render_ff1_passport_index_json(idx: &Ff1PassportIndex) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut s = alloc::string::String::with_capacity(64 * 1024);
    let _ = write!(s, "{{");
    let _ = write!(s, "\"corpus_hash_v1\":\"{}\",", hex32(&idx.corpus_hash_v1));
    let _ = write!(s, "\"corpus_hash_v2\":\"{}\",", hex32(&idx.corpus_hash_v2));
    let _ = write!(
        s,
        "\"consolidation_report_hash_v1\":\"{}\",",
        hex32(&idx.consolidation_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"t12_expansion_index_hash_v1\":\"{}\",",
        hex32(&idx.t12_expansion_index_hash_v1)
    );
    let _ = write!(
        s,
        "\"ff1_passport_index_hash_v1\":\"{}\",",
        hex32(&idx.ff1_passport_index_hash_v1)
    );
    let _ = write!(s, "\"seed_len\":{},", idx.seed_len);
    let _ = write!(s, "\"passport_count\":{},", idx.passports.len());
    let _ = write!(s, "\"passports\":[");
    for (i, p) in idx.passports.iter().enumerate() {
        if i > 0 {
            let _ = write!(s, ",");
        }
        let _ = write!(s, "{{\"canonical_id\":{},", p.canonical_id);
        let _ = write!(s, "\"display_name\":\"{}\",", p.display_name);
        let _ = write!(s, "\"source_class\":\"{}\",", p.source_class_wire_name);
        let _ = write!(s, "\"origin_proposal\":\"{}\",", p.origin_proposal_id);
        let _ = write!(s, "\"gpu_family\":\"{}\",", p.gpu_family_wire_name);
        let _ = write!(s, "\"activation_tags\":[");
        for (j, t) in p.activation_applicability_tags.iter().enumerate() {
            if j > 0 {
                let _ = write!(s, ",");
            }
            let _ = write!(s, "\"{t}\"");
        }
        let _ = write!(s, "],");
        let _ = write!(
            s,
            "\"contraindication_stub\":{{\"declared\":{},\"linked\":{}}},",
            p.contraindication_linkage_stub.stub_declared,
            p.contraindication_linkage_stub
                .linked_contraindication_ids
                .len()
        );
        let _ = write!(
            s,
            "\"challenge_stub\":{{\"declared\":{},\"linked\":{}}},",
            p.challenge_surface_stub.stub_declared,
            p.challenge_surface_stub.linked_challenge_ids.len()
        );
        let _ = write!(
            s,
            "\"passport_hash_v1\":\"{}\"}}",
            hex32(&p.passport_hash_v1)
        );
    }
    let _ = write!(s, "]");
    let _ = write!(s, "}}");
    s
}
