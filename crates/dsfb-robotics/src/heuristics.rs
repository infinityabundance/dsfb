//! Heuristics bank — typed motif library for robotics-specific
//! structural patterns.
//!
//! Each heuristic is a **named, typed motif** that an operator can
//! recognise: `stribeck_gap`, `backlash_ring`, `bpfi_growth`,
//! `grf_desync`, `mpc_stance_lag`, `com_drift`. The heuristics bank is
//! fixed-capacity and known-ahead: DSFB does not learn new motifs
//! online — it classifies observed sign-tuple sequences into one of a
//! published library, with `Unknown` as a first-class fallback.
//!
//! Phase 2 provides the typed enumeration and a minimal `Unknown`
//! classifier so downstream modules (syntax, engine emission) can
//! reference the type. Full pattern recognition for each named motif
//! lands in Phase 3 alongside the dataset adapters that produce the
//! relevant residual streams.

/// A named robotics-specific structural motif.
///
/// The bank is deliberately small and explicit. Anything the classifier
/// cannot recognise as one of the named motifs becomes `Unknown`,
/// surfacing the residual trajectory to the operator without a
/// spurious classification claim.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoboticsMotif {
    /// Stribeck-effect signature: low-velocity friction non-linearity
    /// producing a characteristic torque-residual plateau-then-drop.
    /// Observed during slow joint motion in manipulators without
    /// explicit friction compensation.
    StribeckGap,

    /// Backlash ring: oscillatory residual caused by gear-train
    /// backlash at velocity reversals. Observed in industrial robots
    /// with harmonic drives or cycloidal gearboxes.
    BacklashRing,

    /// Bearing ball-pass-frequency-inner (BPFI) growth: envelope-
    /// spectrum amplitude at the inner-race BPFI harmonics trending
    /// upward. Standard bearing-health indicator.
    BpfiGrowth,

    /// Ground-reaction-force desynchronisation: commanded vs.
    /// measured GRF diverging on a legged platform, typically the
    /// first structural sign of slip or unexpected contact geometry.
    GrfDesync,

    /// MPC stance-phase lag: the whole-body MPC's planned contact
    /// force arriving systematically late relative to measured foot
    /// contact, symptomatic of controller-bandwidth mismatch.
    MpcStanceLag,

    /// Centre-of-mass drift: centroidal-momentum observer estimate
    /// drifting relative to the model prediction, often the earliest
    /// balancing-residual signature before an envelope violation.
    CoMDrift,

    /// None of the above — hand to the operator unclassified.
    #[default]
    Unknown,
}

impl RoboticsMotif {
    /// Stable human-readable label for JSON / logging emission.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StribeckGap => "StribeckGap",
            Self::BacklashRing => "BacklashRing",
            Self::BpfiGrowth => "BpfiGrowth",
            Self::GrfDesync => "GrfDesync",
            Self::MpcStanceLag => "MpcStanceLag",
            Self::CoMDrift => "CoMDrift",
            Self::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unknown() {
        assert_eq!(RoboticsMotif::default(), RoboticsMotif::Unknown);
    }

    #[test]
    fn labels_are_unique() {
        let all = [
            RoboticsMotif::StribeckGap,
            RoboticsMotif::BacklashRing,
            RoboticsMotif::BpfiGrowth,
            RoboticsMotif::GrfDesync,
            RoboticsMotif::MpcStanceLag,
            RoboticsMotif::CoMDrift,
            RoboticsMotif::Unknown,
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a.label(), b.label(), "duplicate label");
                }
            }
        }
    }
}
