//! `DetectorTemplate` — the algebra-grid expansion point.
//!
//! A template names a detector family + the variable parameter
//! axes the algebra walks; the generator (S1.2+) cartesian-products
//! the template's parameter spans to emit one `DetectorSpec` per
//! grid cell.
//!
//! At S1.1 the template is purely a schema declaration — no
//! generator runs. The verifier checks that:
//!
//! - `primitive_id` is `Some`. Every template must link back to
//!   a corpus literature primitive
//!   ([`dsfb_gpu_atlas_corpus::types::DetectorCanonicalId`])
//!   so the algebra cannot mint detectors with no provenance.
//! - `default_window` is positive.
//! - `default_axis_binding` is non-empty.

use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;

use crate::{
    AxisBinding, Comparator, CostClass, DetectorFamily, DomainTagSet, Gate, ImplementationKind,
    NumericMode, Statistic, Transform, WindowSpec,
};

/// Algebra-grid declaration for one detector family.
#[derive(Debug, Clone, Copy)]
pub struct DetectorTemplate {
    /// Family this template generates detectors for.
    pub family: DetectorFamily,
    /// Corpus literature primitive this template binds to. **MUST
    /// be `Some`**; the verifier rejects `None`. The hard link
    /// to corpus provenance is what prevents arbitrary detector
    /// fabrication.
    pub primitive_id: Option<DetectorCanonicalId>,
    /// Default signal transform the family operates on (e.g.
    /// `Residual` for `RobustZMad`).
    pub default_transform: Transform,
    /// Default per-window statistic.
    pub default_statistic: Statistic,
    /// Default comparator.
    pub default_comparator: Comparator,
    /// Default firing gate.
    pub default_gate: Gate,
    /// Default window size.
    pub default_window: WindowSpec,
    /// Default persistence (P{N} value in canonical name).
    pub default_persistence: u32,
    /// Default axis binding.
    pub default_axis_binding: AxisBinding,
    /// Domain tags this template's detectors apply to.
    pub domain_tags: DomainTagSet,
    /// Cost class for this template's detectors.
    pub cost_class: CostClass,
    /// Numeric mode for this template's detectors.
    pub numeric_mode: NumericMode,
    /// Default implementation kind.
    pub implementation_kind: ImplementationKind,
}

impl DetectorTemplate {
    /// Convenience: a minimal template with audit-default
    /// numeric mode and scalar-CPU implementation. Useful for
    /// tests; production templates declare every field.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn minimal(
        family: DetectorFamily,
        primitive_id: DetectorCanonicalId,
        default_transform: Transform,
        default_statistic: Statistic,
        default_comparator: Comparator,
        default_window: WindowSpec,
        default_persistence: u32,
        default_axis_binding: AxisBinding,
        domain_tags: DomainTagSet,
    ) -> Self {
        Self {
            family,
            primitive_id: Some(primitive_id),
            default_transform,
            default_statistic,
            default_comparator,
            default_gate: Gate::None,
            default_window,
            default_persistence,
            default_axis_binding,
            domain_tags,
            cost_class: CostClass::Light,
            numeric_mode: NumericMode::AUDIT_DEFAULT,
            implementation_kind: ImplementationKind::DEFAULT,
        }
    }
}
