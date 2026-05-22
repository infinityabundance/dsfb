//! Verifier for `DetectorTemplate` and `DetectorSpec`
//! (S1.1.1; post-T.10 cross-field rule).
//!
//! Panel-locked rules:
//!
//! - **Template**: `primitive_id` is `Some`. `default_window.cells
//!   >= 1`. `default_axis_binding` is non-empty.
//!   `domain_tags` is non-empty.
//! - **Spec**: `parameter_hash` is not the all-zero sentinel.
//!   The **post-T.10 cross-field rule** holds:
//!
//!   ```text
//!     HashFrozenT10        ⇔  source_corpus_hash != [0; 32]
//!     PreHashT9InternalAudit ⇔  source_corpus_hash == [0; 32]
//!   ```
//!
//!   `axis_binding` is non-empty. `domain_tags` is non-empty.
//!   `window.cells >= 1`. `persistence_windows >= 1`.
//!   `canonical_name` is well-formed (six `__`-delimited tokens,
//!   no empty tokens).
//! - **Registry-level** (`verify_registry_spec`, S1.2+ callers):
//!   the spec MUST be `HashFrozenT10` with `source_corpus_hash`
//!   equal to the caller's `expected_corpus_hash`, AND the spec's
//!   `primitive_id` MUST resolve to a known corpus canonical id.
//!
//! The verifier is deterministic; two runs on the same spec
//! produce the same error sequence.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::{CorpusBindingStatus, DetectorSpec, DetectorTemplate};

/// One verification failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyError {
    /// Structured error kind.
    pub kind: VerifyErrorKind,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Structured error category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyErrorKind {
    /// Template's `primitive_id` was `None`. Every template MUST
    /// link to a corpus literature primitive.
    TemplateMissingPrimitiveId,
    /// Spec's `parameter_hash` is the all-zero sentinel.
    SpecMissingParameterHash,
    /// S1.1 era rule. Spec with `corpus_binding_status =
    /// HashFrozenT10` MUST carry a non-zero `source_corpus_hash`.
    /// The reverse holds too: `PreHashT9InternalAudit` MUST carry
    /// the all-zero sentinel. The variant name is preserved from
    /// the S1.1 gate so the test that pinned it continues to
    /// reference the same enum.
    SpecHashFrozenWithoutT10,
    /// Spec's `corpus_binding_status` is some other future
    /// variant not yet admissible at S1.2.
    SpecCorpusBindingStatusNotAdmissibleAtS11,
    /// Empty `axis_binding` (no fusion axis declared).
    EmptyAxisBinding,
    /// Empty `domain_tags` (no applicability domain declared).
    EmptyDomainTagSet,
    /// `window.cells == 0`. Windows MUST be at least one cell.
    InvalidWindowCells,
    /// `persistence_windows == 0`. Persistence MUST be at least
    /// one window.
    InvalidPersistenceWindows,
    /// Canonical name has the wrong token count
    /// (panel-locked: exactly six tokens separated by `__`).
    CanonicalNameWrongTokenCount,
    /// Canonical name has at least one empty token (e.g. an
    /// `__FOO__` leading empty or a `FOO____BAR` double-double).
    CanonicalNameHasEmptyToken,
    /// S1.2 registry-level error: `source_corpus_hash` does not
    /// match the live `compute_corpus_hash_v1` passed to the
    /// registry verifier. The spec is bound to a stale corpus
    /// snapshot.
    SpecSourceCorpusHashStale,
    /// S1.2 registry-level error: spec's `primitive_id` is `None`
    /// or points at an unknown corpus canonical id.
    SpecPrimitiveIdUnknown,
    /// S1.2 registry-level error: at S1.2 every spec MUST
    /// declare `corpus_binding_status = HashFrozenT10` because
    /// T.10 has landed and `corpus_hash_v1` is canonical.
    SpecMustBeHashFrozenAtS12,
}

/// Verify a template. Returns the list of errors (empty if clean).
#[must_use]
pub fn verify_template(template: &DetectorTemplate) -> alloc::vec::Vec<VerifyError> {
    let mut errors = alloc::vec::Vec::new();
    if template.primitive_id.is_none() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::TemplateMissingPrimitiveId,
            message: format!(
                "template for family {:?} has primitive_id = None; every template must link to a corpus literature primitive",
                template.family
            ),
        });
    }
    if template.default_window.cells == 0 {
        errors.push(VerifyError {
            kind: VerifyErrorKind::InvalidWindowCells,
            message: format!(
                "template for family {:?} declares default_window.cells = 0; must be >= 1",
                template.family
            ),
        });
    }
    if template.default_axis_binding.is_empty() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::EmptyAxisBinding,
            message: format!(
                "template for family {:?} declares an empty axis binding; must bind to >= 1 fusion axis",
                template.family
            ),
        });
    }
    if template.domain_tags.is_empty() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::EmptyDomainTagSet,
            message: format!(
                "template for family {:?} declares an empty domain_tags set; must apply to >= 1 domain",
                template.family
            ),
        });
    }
    errors
}

/// Verify a spec. Returns the list of errors (empty if clean).
#[must_use]
pub fn verify_spec(spec: &DetectorSpec) -> alloc::vec::Vec<VerifyError> {
    let mut errors = alloc::vec::Vec::new();
    if spec.parameter_hash == [0u8; 32] {
        errors.push(VerifyError {
            kind: VerifyErrorKind::SpecMissingParameterHash,
            message: format!(
                "spec for family {:?} has all-zero parameter_hash; must be computed before construction",
                spec.family
            ),
        });
    }
    // Post-T.10 cross-field rule:
    //
    // - `PreHashT9InternalAudit` MUST carry `source_corpus_hash =
    //   [0; 32]`. Pre-freeze specs cannot make a corpus-binding
    //   claim.
    // - `HashFrozenT10` MUST carry `source_corpus_hash != [0; 32]`.
    //   Post-freeze specs MUST identify which corpus they bind to.
    //
    // Both directions emit `SpecHashFrozenWithoutT10` so the
    // verifier surface stays small; the message distinguishes
    // the two failure modes.
    match spec.corpus_binding_status {
        CorpusBindingStatus::PreHashT9InternalAudit => {
            if spec.source_corpus_hash != [0u8; 32] {
                errors.push(VerifyError {
                    kind: VerifyErrorKind::SpecHashFrozenWithoutT10,
                    message: format!(
                        "spec for family {:?} carries `PreHashT9InternalAudit` but `source_corpus_hash` is non-zero; pre-freeze specs MUST keep `source_corpus_hash = [0; 32]`",
                        spec.family
                    ),
                });
            }
        }
        CorpusBindingStatus::HashFrozenT10 => {
            if spec.source_corpus_hash == [0u8; 32] {
                errors.push(VerifyError {
                    kind: VerifyErrorKind::SpecHashFrozenWithoutT10,
                    message: format!(
                        "spec for family {:?} claims `HashFrozenT10` but `source_corpus_hash` is the all-zero sentinel; T.10 specs MUST carry a concrete `corpus_hash_v1`",
                        spec.family
                    ),
                });
            }
        }
    }
    if spec.axis_binding.is_empty() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::EmptyAxisBinding,
            message: format!(
                "spec for family {:?} declares an empty axis_binding; must bind to >= 1 fusion axis",
                spec.family
            ),
        });
    }
    if spec.domain_tags.is_empty() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::EmptyDomainTagSet,
            message: format!(
                "spec for family {:?} declares an empty domain_tags set; must apply to >= 1 domain",
                spec.family
            ),
        });
    }
    if spec.window.cells == 0 {
        errors.push(VerifyError {
            kind: VerifyErrorKind::InvalidWindowCells,
            message: format!(
                "spec for family {:?} declares window.cells = 0; must be >= 1",
                spec.family
            ),
        });
    }
    if spec.persistence_windows == 0 {
        errors.push(VerifyError {
            kind: VerifyErrorKind::InvalidPersistenceWindows,
            message: format!(
                "spec for family {:?} declares persistence_windows = 0; must be >= 1",
                spec.family
            ),
        });
    }
    if spec.canonical_name.token_count() != 6 {
        errors.push(VerifyError {
            kind: VerifyErrorKind::CanonicalNameWrongTokenCount,
            message: format!(
                "spec for family {:?} has canonical_name `{}` with {} `__`-delimited tokens; must be 6",
                spec.family,
                spec.canonical_name.as_str(),
                spec.canonical_name.token_count()
            ),
        });
    }
    if !spec.canonical_name.has_no_empty_token() {
        errors.push(VerifyError {
            kind: VerifyErrorKind::CanonicalNameHasEmptyToken,
            message: format!(
                "spec for family {:?} has canonical_name `{}` with at least one empty `__`-delimited token",
                spec.family,
                spec.canonical_name.as_str()
            ),
        });
    }
    errors
}

/// S1.2 registry-level spec verifier.
///
/// Adds the following invariants on top of [`verify_spec`]:
///
/// - The spec MUST declare `corpus_binding_status = HashFrozenT10`
///   (the S1.2 generator always produces frozen specs; an
///   "unfrozen" spec has no place in a generated registry).
/// - The spec's `source_corpus_hash` MUST equal
///   `expected_corpus_hash` byte-for-byte. A spec bound to a
///   stale corpus snapshot is rejected.
/// - The spec's `primitive_id` MUST be `Some(_)` and MUST resolve
///   to a record in `corpus_canonical_ids` (the live SEED
///   canonical-id set).
///
/// The caller passes the canonical
/// `compute_corpus_hash_v1().bytes` and the set of admissible
/// `DetectorCanonicalId`s. This keeps the registry crate free of
/// hard-coded corpus snapshots — the verifier just enforces
/// reference integrity against whichever corpus the caller is
/// binding to.
#[must_use]
pub fn verify_registry_spec(
    spec: &crate::DetectorSpec,
    expected_corpus_hash: &[u8; 32],
    corpus_canonical_ids: &[dsfb_gpu_atlas_corpus::types::DetectorCanonicalId],
) -> alloc::vec::Vec<VerifyError> {
    let mut errors = verify_spec(spec);
    if spec.corpus_binding_status != crate::CorpusBindingStatus::HashFrozenT10 {
        errors.push(VerifyError {
            kind: VerifyErrorKind::SpecMustBeHashFrozenAtS12,
            message: format!(
                "registry spec for family {:?} declares corpus_binding_status = {:?}; S1.2 generated specs MUST be HashFrozenT10",
                spec.family, spec.corpus_binding_status
            ),
        });
    }
    if &spec.source_corpus_hash != expected_corpus_hash {
        errors.push(VerifyError {
            kind: VerifyErrorKind::SpecSourceCorpusHashStale,
            message: format!(
                "registry spec for family {:?} carries a stale source_corpus_hash; expected the live compute_corpus_hash_v1 bytes",
                spec.family
            ),
        });
    }
    match spec.primitive_id {
        None => errors.push(VerifyError {
            kind: VerifyErrorKind::SpecPrimitiveIdUnknown,
            message: format!(
                "registry spec for family {:?} has primitive_id = None; S1.2 specs MUST point at a corpus literature primitive",
                spec.family
            ),
        }),
        Some(id) => {
            if !corpus_canonical_ids.contains(&id) {
                errors.push(VerifyError {
                    kind: VerifyErrorKind::SpecPrimitiveIdUnknown,
                    message: format!(
                        "registry spec for family {:?} has primitive_id = DetectorCanonicalId({}); not a known corpus canonical id",
                        spec.family, id.0
                    ),
                });
            }
        }
    }
    errors
}
