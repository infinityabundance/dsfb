/// DSFB Oil & Gas — Rotating Equipment Domain Module
///
/// Maps ESP and centrifugal compressor telemetry into DSFB residual samples.
///
/// Physical context: ESP head–flow performance curve H(Q) = a0 − a1Q − a2Q².
/// Expected head is from the calibrated factory or commissioning curve at the
/// measured flow and speed.  Residuals capture scale deposition (slow positive
/// drift), gas interference (negative slew spikes), and worn-stage degradation
/// (monotone negative efficiency residual).
/// DSFB augments plant historian trending; it does not replace API 670 vibration
/// monitoring or OEM performance curves.

use crate::types::{DsfbDomainFrame, ResidualSample};

/// One telemetry frame from a rotating equipment system (ESP or compressor).
///
/// expected_head: model or curve-fit head [m] at current flow/speed.
/// observed_head: computed from measured discharge minus intake pressure.
/// vibration_rms: RMS vibration amplitude [mm/s or g], for auxiliary channel.
/// flow_rate: volumetric flow [m³/h].
#[derive(Debug, Clone, Copy)]
pub struct RotatingFrame {
    pub timestamp: f64,
    pub expected_head: f64,
    pub observed_head: f64,
    pub vibration_rms: f64,
    pub flow_rate: f64,
}

impl DsfbDomainFrame for RotatingFrame {
    fn timestamp(&self) -> f64 { self.timestamp }
    fn expected(&self) -> f64 { self.expected_head }
    fn observed(&self) -> f64 { self.observed_head }
    fn channel_name(&self) -> &str { "rotating_head_m" }
}

impl RotatingFrame {
    /// Auxiliary vibration residual (deviation from baseline at this operating point).
    /// Caller supplies the expected_vibration for the current speed/flow quadrant.
    pub fn vibration_residual(&self, expected_vibration: f64) -> ResidualSample {
        ResidualSample::new(
            self.timestamp,
            self.vibration_rms,
            expected_vibration,
            "rotating_vibration_rms",
        )
    }
}

/// Named reason codes specific to rotating equipment grammar events.
pub mod rotating_reasons {
    pub const SCALE_DEPOSIT: &str = "Sustained positive head drift; consistent with scale deposition reducing pump efficiency";
    pub const GAS_INTERFER:  &str = "Negative slew spike; consistent with gas pocket displacement in ESP stages";
    pub const SURGE_APPROACH: &str = "Positive slew in vibration residual near design point; review compressor surge proximity";
}
