//! Corpus binding status — post-T.10 honest enum tracking how a
//! `DetectorSpec` binds to the literature corpus.
//!
//! T.10 froze `corpus_hash_v1`. Every `DetectorSpec` carries one
//! of these variants plus a separate `source_corpus_hash: [u8;
//! 32]` field on the spec; the verifier in [`super::verify`]
//! enforces the cross-field rule
//!
//! ```text
//!   HashFrozenT10  ⇔  source_corpus_hash != [0; 32]
//! ```
//!
//! `verify_registry_spec` (S1.2+) further requires that the hash
//! equal the live `compute_corpus_hash_v1()` of the corpus crate
//! and that the spec's `primitive_id` resolve to a known
//! canonical corpus record. The base verifier here only checks
//! cross-field consistency; it does not load the corpus.
//!
//! Why the enum still exists post-T.10: the corpus-binding
//! intent is a separate axis from the hash bytes. A spec might
//! legitimately carry `PreHashT9InternalAudit` (zero hash) during
//! pre-S1.2 testing of the algebra surface itself, and S1.2
//! specs all carry `HashFrozenT10` with a non-zero hash. Keeping
//! both axes explicit means we can never silently flip between
//! the two without the verifier noticing.

/// Tracks how a `DetectorSpec` binds to the corpus crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CorpusBindingStatus {
    /// Pre-hash phase. The spec does NOT carry a frozen corpus
    /// hash; its `source_corpus_hash` must be `[0; 32]`. Used by
    /// algebra-only fixtures and the pre-S1.2 verifier tests
    /// that exercise the type surface without binding to a
    /// specific corpus snapshot.
    PreHashT9InternalAudit,
    /// Post-hash phase. `corpus_hash_v1` is frozen and the spec
    /// binds to a specific corpus snapshot by hash; its
    /// `source_corpus_hash` MUST be non-zero. S1.2-generated
    /// specs all use this variant with the live
    /// `compute_corpus_hash_v1()` bytes.
    HashFrozenT10,
}

impl CorpusBindingStatus {
    /// Algebra-only default carried by pre-S1.2 fixtures. Used
    /// only by tests that exercise the type surface without
    /// binding to a corpus snapshot. S1.2-generated specs use
    /// `HashFrozenT10`.
    pub const S1_1_DEFAULT: CorpusBindingStatus = CorpusBindingStatus::PreHashT9InternalAudit;

    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::PreHashT9InternalAudit => "PRE_HASH_T9_INTERNAL_AUDIT",
            Self::HashFrozenT10 => "HASH_FROZEN_T10",
        }
    }

    /// True if this status was admissible under the pre-T.10
    /// (S1.1) policy where only the pre-hash variant was
    /// accepted. Retained for historical tests; this is **not**
    /// the post-T.10 verifier rule — see
    /// [`super::verify::verify_spec`] for the live cross-field
    /// rule.
    #[must_use]
    pub const fn admissible_at_s1_1(self) -> bool {
        matches!(self, Self::PreHashT9InternalAudit)
    }
}
