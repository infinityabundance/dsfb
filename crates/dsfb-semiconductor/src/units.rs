//! Type-safe physical quantity wrappers for semiconductor process variables.
//!
//! # Design Rationale
//! Every value entering the public DSFB observer API must carry an explicit
//! unit context.  Raw `f64` scalars are prohibited in externally-facing
//! function signatures; this module provides lightweight newtypes that make
//! dimensional intent part of the type system.
//!
//! # No-std Compatibility
//! This module is `no_std`-compatible.  All traits are from `core`, and the
//! `Display` implementation uses `core::fmt`.
//!
//! | Physical quantity               | Newtype      | Typical semiconductor use                |
//! |---------------------------------|--------------|------------------------------------------|
//! | Gas mass-flow                   | [`Sccm`]     | MFC setpoint / feedback residual         |
//! | Chamber pressure                | [`MilliTorr`]| Manometer / throttle valve residual      |
//! | RF generator power              | [`Watts`]    | Plasma source / bias power residual      |
//!
//! # Examples
//! ```
//! use dsfb_semiconductor::units::{PhysicalValue, Sccm, MilliTorr, Watts};
//!
//! let flow  = PhysicalValue::GasFlow(Sccm(120.5));
//! let press = PhysicalValue::Pressure(MilliTorr(35.2));
//! let power = PhysicalValue::RfPower(Watts(450.0));
//!
//! assert_eq!(flow.dimension(), "sccm");
//! assert!((press.raw_scalar() - 35.2).abs() < 1e-9);
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};

// ─── Newtypes ─────────────────────────────────────────────────────────────────

/// Standard cubic centimetres per minute — the SI-derived unit for gas
/// mass-flow controllers (MFCs) in semiconductor process chambers.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Sccm(pub f64);

/// Chamber pressure in milli-Torr — the practical unit for high-vacuum
/// semiconductor process chambers (typical range: 1–500 mTorr).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MilliTorr(pub f64);

/// RF generator / bias-power in Watts — used for plasma source and substrate
/// bias power in etch, deposition, and clean processes.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Watts(pub f64);

// ─── Tagged union ────────────────────────────────────────────────────────────

/// A tagged physical observation: a scalar value together with its dimensional
/// classification.  The DSFB public observer API accepts `PhysicalValue` rather
/// than bare `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PhysicalValue {
    /// Gas mass-flow controller value.
    GasFlow(Sccm),
    /// Chamber pressure sensor value.
    Pressure(MilliTorr),
    /// RF generator power value.
    RfPower(Watts),
    /// Dimensionless normalised residual (z-score or fractional deviation).
    Dimensionless(f64),
}

impl PhysicalValue {
    /// Extract the raw scalar.
    ///
    /// Callers that depend on the physical meaning of the value must preserve
    /// [`PhysicalValue::dimension`] alongside any downstream use of this scalar.
    #[must_use]
    pub fn raw_scalar(self) -> f64 {
        match self {
            Self::GasFlow(Sccm(v)) => v,
            Self::Pressure(MilliTorr(v)) => v,
            Self::RfPower(Watts(v)) => v,
            Self::Dimensionless(v) => v,
        }
    }

    /// The dimension tag as a canonical lowercase string, suitable for
    /// embedding in traceability manifests and JSON signature files.
    #[must_use]
    pub fn dimension(self) -> &'static str {
        match self {
            Self::GasFlow(_) => "sccm",
            Self::Pressure(_) => "milli_torr",
            Self::RfPower(_) => "watts",
            Self::Dimensionless(_) => "dimensionless",
        }
    }
}

impl fmt::Display for PhysicalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GasFlow(Sccm(v)) => write!(f, "{v:.4} sccm"),
            Self::Pressure(MilliTorr(v)) => write!(f, "{v:.4} mTorr"),
            Self::RfPower(Watts(v)) => write!(f, "{v:.4} W"),
            Self::Dimensionless(v) => write!(f, "{v:.6} (dimensionless)"),
        }
    }
}

// ─── Unit scale manifest ──────────────────────────────────────────────────────

/// Describes the physical unit conventions and normalisation strategy used
/// during a DSFB run.  Emitted verbatim in every `dsfb_run_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UomScales {
    /// Unit for gas mass-flow observables.
    pub gas_flow_unit: &'static str,
    /// Unit for chamber pressure observables.
    pub pressure_unit: &'static str,
    /// Unit for RF power observables.
    pub rf_power_unit: &'static str,
    /// Strategy used to normalise raw sensor values before DSFB ingestion.
    pub normalisation: &'static str,
    /// Scale convention note for future interoperability.
    pub interoperability_note: &'static str,
}

impl Default for UomScales {
    fn default() -> Self {
        Self {
            gas_flow_unit: "sccm",
            pressure_unit: "milli_torr",
            rf_power_unit: "watts",
            normalisation: "z-score relative to healthy-phase empirical mean and sigma",
            interoperability_note:
                "All DSFB residuals are dimensionless after normalisation; \
                 the original unit tags are preserved in the traceability manifest \
                 for reverse-engineering and physical interpretation.",
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_scalar_round_trips() {
        let v = PhysicalValue::GasFlow(Sccm(120.5));
        assert!((v.raw_scalar() - 120.5).abs() < 1e-12);

        let v2 = PhysicalValue::Pressure(MilliTorr(35.2));
        assert!((v2.raw_scalar() - 35.2).abs() < 1e-12);

        let v3 = PhysicalValue::RfPower(Watts(450.0));
        assert!((v3.raw_scalar() - 450.0).abs() < 1e-12);
    }

    #[test]
    fn dimension_tags_are_canonical() {
        assert_eq!(PhysicalValue::GasFlow(Sccm(1.0)).dimension(), "sccm");
        assert_eq!(
            PhysicalValue::Pressure(MilliTorr(1.0)).dimension(),
            "milli_torr"
        );
        assert_eq!(PhysicalValue::RfPower(Watts(1.0)).dimension(), "watts");
        assert_eq!(PhysicalValue::Dimensionless(1.0).dimension(), "dimensionless");
    }

    #[test]
    fn display_contains_unit_suffix() {
        let s = format!("{}", PhysicalValue::GasFlow(Sccm(10.0)));
        assert!(s.contains("sccm"), "expected 'sccm' in '{s}'");

        let s2 = format!("{}", PhysicalValue::Pressure(MilliTorr(5.0)));
        assert!(s2.contains("mTorr"), "expected 'mTorr' in '{s2}'");
    }

    #[test]
    fn uom_scales_default_is_deterministic() {
        let a = UomScales::default();
        let b = UomScales::default();
        assert_eq!(a.gas_flow_unit, b.gas_flow_unit);
        assert_eq!(a.normalisation, b.normalisation);
    }
}
