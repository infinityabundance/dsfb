//! Detector, family, and parameterization identifiers.
//!
//! At S1.1 these are simple newtypes around `u32`. The
//! authoritative `DetectorCanonicalId` for corpus literature
//! primitives lives in [`dsfb_gpu_atlas_corpus::types`]; this
//! crate's `DetectorId` is a separate handle for the Atlas
//! registry's expanded detector instantiations (16 motifs × N
//! variants × M parameter grids ≈ 2,000+ detectors), not the
//! literature primitives themselves. Linkage between Atlas
//! `DetectorId` and corpus `DetectorCanonicalId` is via the
//! `primitive_id` field on `DetectorSpec`.
//!
//! Why two separate id spaces: the corpus is small (54 records
//! at T.9) and stable. The Atlas registry is large (thousands)
//! and parameter-grid-driven. Sharing a single id namespace would
//! either force the corpus to renumber when the registry expands
//! or force the registry to inherit corpus's tiny range.

/// Atlas registry-side detector handle. Distinct from
/// `dsfb_gpu_atlas_corpus::types::DetectorCanonicalId` (the
/// corpus literature-primitive handle). At S1.2+ the registry
/// generator assigns these deterministically over the cartesian
/// product of family × parameter grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectorId(pub u32);

/// Stable identifier for a detector family. Used as a join key
/// between `DetectorTemplate` and `DetectorSpec`. The value is
/// derived from the position of the variant in [`crate::DetectorFamily`]'s
/// canonical ordering (`DetectorFamily::all()`); changing that
/// ordering would change every `FamilyId` value, so the order is
/// pinned by the `detector_family_order_is_stable` acceptance
/// test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FamilyId(pub u32);

/// Stable identifier for a concrete parameterization within a
/// family. Two `DetectorSpec`s sharing a `FamilyId` differ by
/// their `ParameterizationId`. The numbering scheme is opaque at
/// S1.1 — S1.2+ pins it via the registry-generation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterizationId(pub u32);
