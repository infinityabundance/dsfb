//! Robot operating context for envelope scaling and violation suppression.
//!
//! Residuals observed during commissioning, calibration, or planned
//! maintenance are **not** violations — they are expected. The
//! [`RobotContext`] encodes which operating regime the residual stream
//! is in, and the [`RobotContext::admissibility_multiplier`] method
//! returns the scaling factor DSFB applies to the envelope radius ρ.
//!
//! During commissioning and maintenance the multiplier is `+∞`, which
//! makes envelope violations structurally impossible — the grammar FSM
//! is forced to `Admissible` regardless of residual magnitude.

/// The robot's current operating regime.
///
/// Set by the caller from whatever state source is authoritative for
/// the deployment (ROS 2 `mode` topic, OPC UA `OperationalStatus`
/// node, programmable-logic flag, manual operator switch). DSFB never
/// transitions between contexts on its own — it is an observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RobotContext {
    /// Arm is being commissioned: dynamic parameter identification in
    /// progress, friction tables being populated, excitation
    /// trajectories being executed. Residuals are expected to be large
    /// and non-stationary. Violations are suppressed.
    ArmCommissioning,

    /// Arm is in normal operation: trajectory tracking, force control,
    /// teleoperation. Residuals are expected to be small and stationary
    /// within the calibrated envelope. Full violation enforcement.
    ArmOperating,

    /// Legged platform is in a stance phase: at least one end effector
    /// in contact with the ground, contact forces actively controlled
    /// by the whole-body controller. Residuals include MPC force
    /// tracking error and centroidal-momentum estimator discrepancy.
    /// Full violation enforcement.
    LeggedStance,

    /// Legged platform is in a swing phase: at least one end effector
    /// off the ground, swing-foot trajectory tracking active. Contact
    /// residuals are not applicable; only joint-level kinematic
    /// residuals are enforced. Full violation enforcement with a
    /// relaxed envelope (swing-phase residuals are typically looser
    /// than stance-phase).
    LeggedSwing,

    /// Planned maintenance: operator-initiated diagnostics or
    /// mechanical service. Residuals may be very large as the robot is
    /// deliberately moved through unusual configurations. Violations
    /// are suppressed.
    Maintenance,
}

impl RobotContext {
    /// Returns `true` if violations are suppressed in this context.
    ///
    /// Commissioning and maintenance periods both produce residual
    /// patterns that look like faults to a naive observer but are
    /// expected by design. DSFB recognises these contexts and holds
    /// the grammar FSM in `Admissible`.
    #[inline]
    #[must_use]
    pub const fn is_suppressed(self) -> bool {
        matches!(self, Self::ArmCommissioning | Self::Maintenance)
    }

    /// Multiplier applied to the envelope radius ρ in this context.
    ///
    /// - `ArmOperating`, `LeggedStance`: `1.0` (baseline envelope).
    /// - `LeggedSwing`: `1.5` (swing residuals run wider than stance).
    /// - `ArmCommissioning`, `Maintenance`: `f64::INFINITY` (no
    ///   violation possible — residuals are expected to be arbitrary).
    ///
    /// The choice of `1.5` for swing is intentionally modest; the
    /// caller may override with a custom envelope per-phase if a
    /// tighter bound is known from the controller.
    #[inline]
    #[must_use]
    pub fn admissibility_multiplier(self) -> f64 {
        match self {
            Self::ArmOperating | Self::LeggedStance => 1.0,
            Self::LeggedSwing => 1.5,
            Self::ArmCommissioning | Self::Maintenance => f64::INFINITY,
        }
    }

    /// Short stable string label, for logging and JSON emission.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ArmCommissioning => "ArmCommissioning",
            Self::ArmOperating => "ArmOperating",
            Self::LeggedStance => "LeggedStance",
            Self::LeggedSwing => "LeggedSwing",
            Self::Maintenance => "Maintenance",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppressed_contexts_are_commissioning_and_maintenance() {
        assert!(RobotContext::ArmCommissioning.is_suppressed());
        assert!(RobotContext::Maintenance.is_suppressed());
        assert!(!RobotContext::ArmOperating.is_suppressed());
        assert!(!RobotContext::LeggedStance.is_suppressed());
        assert!(!RobotContext::LeggedSwing.is_suppressed());
    }

    #[test]
    fn multiplier_matches_contract() {
        assert_eq!(RobotContext::ArmOperating.admissibility_multiplier(), 1.0);
        assert_eq!(RobotContext::LeggedStance.admissibility_multiplier(), 1.0);
        assert_eq!(RobotContext::LeggedSwing.admissibility_multiplier(), 1.5);
        assert!(RobotContext::ArmCommissioning.admissibility_multiplier().is_infinite());
        assert!(RobotContext::Maintenance.admissibility_multiplier().is_infinite());
    }

    #[test]
    fn labels_are_stable_and_unique() {
        let labels = [
            RobotContext::ArmCommissioning.label(),
            RobotContext::ArmOperating.label(),
            RobotContext::LeggedStance.label(),
            RobotContext::LeggedSwing.label(),
            RobotContext::Maintenance.label(),
        ];
        for (i, a) in labels.iter().enumerate() {
            for (j, b) in labels.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "labels must be unique");
                }
            }
        }
    }
}
