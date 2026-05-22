//! `DetectorSpec` — the concrete output of one algebra-grid
//! evaluation.
//!
//! A spec is a fully-resolved detector identity: family,
//! transform, statistic, comparator, gate, window, persistence,
//! axis binding, domain tags, cost / numeric / implementation
//! tags, parameter hash, corpus binding status, corpus binding
//! hash, and canonical name.
//!
//! Specs are CONSTRUCTED here but NOT generated — S1.2's
//! registry generator (separate commit) cartesian-products
//! templates into specs. The post-T.10 verifier rejects:
//!
//! - `parameter_hash == [0; 32]` (the all-zero hash is a
//!   sentinel for "not yet computed").
//! - cross-field rule violation:
//!   `HashFrozenT10 ⇔ source_corpus_hash != [0; 32]`.
//! - empty `axis_binding`.
//! - empty `domain_tags`.
//! - `window.cells == 0`.
//! - `persistence_windows == 0`.
//! - malformed `canonical_name`.

use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;

use crate::{
    AxisBinding, CanonicalDetectorName, Comparator, CorpusBindingStatus, CostClass, DetectorFamily,
    DetectorId, DomainTagSet, Gate, ImplementationKind, NumericMode, ParameterizationId, Statistic,
    Transform, WindowSpec,
};

/// One fully-resolved detector identity.
#[derive(Debug, Clone)]
pub struct DetectorSpec {
    /// Atlas-side detector handle.
    pub detector_id: DetectorId,
    /// Parameterisation handle within the family.
    pub parameterization_id: ParameterizationId,
    /// Detector family.
    pub family: DetectorFamily,
    /// Pre-statistic transform.
    pub transform: Transform,
    /// Window size.
    pub window: WindowSpec,
    /// Per-window statistic.
    pub statistic: Statistic,
    /// Comparator.
    pub comparator: Comparator,
    /// Firing gate.
    pub gate: Gate,
    /// Persistence (P{N}).
    pub persistence_windows: u32,
    /// Fusion-axis binding.
    pub axis_binding: AxisBinding,
    /// Applicability domain tags.
    pub domain_tags: DomainTagSet,
    /// Cost class.
    pub cost_class: CostClass,
    /// Numeric mode.
    pub numeric_mode: NumericMode,
    /// GPU-kernel implementation kind.
    pub implementation_kind: ImplementationKind,
    /// SHA-256 over the canonical parameter tuple. Sentinel
    /// `[0; 32]` is rejected by the verifier.
    pub parameter_hash: [u8; 32],
    /// Corpus literature primitive this spec links to.
    /// `verify_registry_spec` (S1.2) rejects `None`; the base
    /// `verify_spec` allows it for algebra-only fixtures that
    /// pre-date a real registry.
    pub primitive_id: Option<DetectorCanonicalId>,
    /// Corpus binding intent. Post-T.10, `HashFrozenT10`
    /// declares the spec is receipt-bound to the canonical
    /// `corpus_hash_v1`; `PreHashT9InternalAudit` declares it is
    /// not. The verifier enforces the cross-field rule against
    /// the `source_corpus_hash` field below.
    pub corpus_binding_status: CorpusBindingStatus,
    /// Corpus binding hash. Carries the `corpus_hash_v1` bytes
    /// for `HashFrozenT10` specs; MUST be the all-zero sentinel
    /// for `PreHashT9InternalAudit` specs. The base verifier
    /// enforces the cross-field invariant
    /// `HashFrozenT10 ⇔ source_corpus_hash != [0; 32]`.
    ///
    /// This field is what makes a generated `DetectorSpec`
    /// receipt-bound to the T.10-frozen corpus identity. A spec
    /// with a stale `source_corpus_hash` (one that does not
    /// match the live `compute_corpus_hash_v1`) is rejected by
    /// the registry verifier in
    /// [`crate::verify::verify_registry_spec`].
    pub source_corpus_hash: [u8; 32],
    /// Canonical wire name.
    pub canonical_name: CanonicalDetectorName,
}

impl DetectorSpec {
    /// True if the detector_id and parameterization_id pair
    /// matches the family (i.e. the family_id derived from the
    /// `DetectorFamily` value matches the family-id half of the
    /// `DetectorId` encoding). The base crate does not enforce a
    /// strict encoding (the S1.2 registry generator's job), but
    /// the helper exists so future tests can pin it.
    #[must_use]
    pub const fn family_id_matches_family(&self) -> bool {
        // Registry generator is S1.2+; this hook stays for
        // symmetry but accepts any encoding.
        true
    }
}
