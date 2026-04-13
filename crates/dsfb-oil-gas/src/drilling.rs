/// DSFB Oil & Gas — Drilling Domain Module
///
/// Maps rotary drilling telemetry into DSFB residual samples.
///
/// Physical context: drillstring torsion wave equation (PDE) governs torque
/// propagation.  Surface torque estimators use WOB, RPM, and formation type
/// as proxies.  Residuals on the torque channel capture stick-slip signatures,
/// bit wear trends, and formation transition events.
/// DSFB augments existing drilling advisory systems; it does not replace
/// torsion-wave simulation or managed-pressure-drilling controllers.

use crate::types::DsfbDomainFrame;

/// One telemetry frame from a rotary drilling system.
///
/// expected_torque: estimator output [kNm] from surface drilling model.
/// observed_torque: surface top-drive torque measurement [kNm].
/// wob: weight on bit [kN].
/// rpm: surface rotary speed [rpm].
#[derive(Debug, Clone, Copy)]
pub struct DrillingFrame {
    pub timestamp: f64,
    pub expected_torque: f64,
    pub observed_torque: f64,
    pub wob: f64,
    pub rpm: f64,
}

impl DsfbDomainFrame for DrillingFrame {
    fn timestamp(&self) -> f64 { self.timestamp }
    fn expected(&self) -> f64 { self.expected_torque }
    fn observed(&self) -> f64 { self.observed_torque }
    fn channel_name(&self) -> &str { "drilling_torque_kNm" }
}

/// Named reason codes specific to drilling grammar events.
pub mod drilling_reasons {
    pub const STICK_SLIP: &str = "Oscillatory slew consistent with stick–slip torsional dynamics; review RPM/WOB combination";
    pub const BIT_WEAR:   &str = "Monotone positive drift in torque residual; consistent with increasing mechanical specific energy";
    pub const FORMATION:  &str = "Step-like residual; correlate with lithologic marker depth";
}
