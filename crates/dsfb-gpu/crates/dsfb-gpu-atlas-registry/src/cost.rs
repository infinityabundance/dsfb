//! Cost class — coarse compute-cost tag on a detector.
//!
//! The activation planner (Section S1.3+) uses this to gate
//! detectors against a wall-time budget without re-measuring
//! per-detector cost on every plan. The class is set at
//! schema-declaration time based on the family's known compute
//! profile (e.g. wavelet ≫ scalar threshold) and is NOT
//! empirically refined at S1.1 — refinement is T.8 ledger work.

/// Coarse compute-cost class for a detector spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CostClass {
    /// Cheap (scalar threshold, simple window mean, etc.).
    Light,
    /// Moderate (rank statistics, distribution-distance under a
    /// fixed window).
    Medium,
    /// Expensive (FFT / wavelet / sequential change detector with
    /// long lookback).
    Heavy,
}

impl CostClass {
    /// Canonical ordering — Light < Medium < Heavy. Pinned by
    /// the `cost_class_order_is_stable` acceptance test.
    #[must_use]
    pub const fn all() -> &'static [CostClass] {
        &[CostClass::Light, CostClass::Medium, CostClass::Heavy]
    }

    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Light => "LIGHT",
            Self::Medium => "MEDIUM",
            Self::Heavy => "HEAVY",
        }
    }
}
