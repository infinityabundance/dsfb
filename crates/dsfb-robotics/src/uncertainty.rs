//! Uncertainty budget per GUM JCGM 100:2008.
//!
//! The calibration `ρ = μ + 3σ` is a point estimate. Reporting it
//! without an uncertainty budget is incompatible with metrological
//! honesty. This module captures the two GUM uncertainty dimensions
//! and provides a simple combination rule suitable for the companion
//! paper's uncertainty-budget table.
//!
//! Phase 2 provides the types and the quadrature combiner; Phase 7
//! populates the budget tables in `docs/uncertainty_budget_gum.md`
//! with per-dataset values sourced from each oracle-protocol document.

use crate::math;

/// A single line in the uncertainty budget.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UncertaintyComponent {
    /// Stable identifier (e.g. `"calibration_sample_variance"`).
    pub name: &'static str,
    /// GUM Type — `A` for statistical, `B` for non-statistical (datasheet, heuristic).
    pub ty: GumType,
    /// Standard uncertainty `u_i` expressed in the same units as the
    /// residual norm (typically newton-metres or newtons, depending
    /// on the dataset).
    pub standard_uncertainty: f64,
}

/// GUM JCGM 100:2008 uncertainty category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GumType {
    /// Type A: evaluated from statistical analysis of observations.
    A,
    /// Type B: evaluated from scientific judgement (datasheet, prior,
    /// calibration certificate).
    B,
}

impl GumType {
    /// Stable label.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::A => "TypeA",
            Self::B => "TypeB",
        }
    }
}

/// Combine a slice of uncertainty components into a combined standard
/// uncertainty `u_c = sqrt(Σ u_i²)`.
///
/// Assumes components are **uncorrelated**. Returns `None` if any
/// component is non-finite or negative, or the slice is empty.
#[must_use]
pub fn combined_standard_uncertainty(components: &[UncertaintyComponent]) -> Option<f64> {
    if components.is_empty() {
        return None;
    }
    let mut ssq = 0.0_f64;
    for c in components {
        let u = c.standard_uncertainty;
        if !u.is_finite() || u < 0.0 {
            return None;
        }
        ssq += u * u;
    }
    math::sqrt_f64(ssq)
}

/// Expanded uncertainty `U = k · u_c` for a chosen coverage factor.
///
/// `k = 2` corresponds to ≈ 95 % coverage for a Normal distribution.
/// `k = 3` corresponds to ≈ 99.7 %. Negative or non-finite `k` is
/// rejected.
#[must_use]
pub fn expanded_uncertainty(
    components: &[UncertaintyComponent],
    coverage_factor: f64,
) -> Option<f64> {
    if !coverage_factor.is_finite() || coverage_factor < 0.0 {
        return None;
    }
    let u_c = combined_standard_uncertainty(components)?;
    Some(coverage_factor * u_c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(u: f64, ty: GumType) -> UncertaintyComponent {
        UncertaintyComponent { name: "test", ty, standard_uncertainty: u }
    }

    #[test]
    fn empty_components_is_none() {
        assert!(combined_standard_uncertainty(&[]).is_none());
    }

    #[test]
    fn single_component_passthrough() {
        let c = [mk(0.5, GumType::A)];
        let u = combined_standard_uncertainty(&c).expect("finite");
        assert!((u - 0.5).abs() < 1e-12);
    }

    #[test]
    fn quadrature_sum() {
        // 3-4-5 triangle in uncertainty space.
        let c = [mk(3.0, GumType::A), mk(4.0, GumType::B)];
        let u = combined_standard_uncertainty(&c).expect("finite");
        assert!((u - 5.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_negative_or_non_finite() {
        assert!(combined_standard_uncertainty(&[mk(-0.1, GumType::A)]).is_none());
        assert!(combined_standard_uncertainty(&[mk(f64::NAN, GumType::A)]).is_none());
    }

    #[test]
    fn expanded_is_k_times_combined() {
        let c = [mk(3.0, GumType::A), mk(4.0, GumType::B)];
        let u95 = expanded_uncertainty(&c, 2.0).expect("finite");
        assert!((u95 - 10.0).abs() < 1e-12);
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(GumType::A.label(), "TypeA");
        assert_eq!(GumType::B.label(), "TypeB");
    }
}
