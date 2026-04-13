/// DSFB Oil & Gas — Pipeline Domain Module
///
/// Maps pipeline flow–pressure sensor frames into DSFB residual samples.
///
/// Physical context: Darcy–Weisbach friction governs steady-state
/// pressure drop; RTTM systems track transient line-pack.  The residual
/// presented here is (observed_flow_balance − expected_flow_balance) on the
/// primary metered channel, plus inlet/outlet pressure auxiliary channels.
/// DSFB augments the RTTM output; it does not replicate RTTM logic.

use crate::types::{DsfbDomainFrame, ResidualSample};

/// One telemetry frame from a pipeline monitoring system.
///
/// expected_flow_balance: RTTM or steady-state model estimate [m³/h or kg/s].
/// observed_flow_balance: measured metered balance on the same channel.
#[derive(Debug, Clone, Copy)]
pub struct PipelineFrame {
    pub timestamp: f64,
    pub expected_flow_balance: f64,
    pub observed_flow_balance: f64,
    pub inlet_pressure: f64,
    pub outlet_pressure: f64,
}

impl DsfbDomainFrame for PipelineFrame {
    fn timestamp(&self) -> f64 { self.timestamp }
    fn expected(&self) -> f64 { self.expected_flow_balance }
    fn observed(&self) -> f64 { self.observed_flow_balance }
    fn channel_name(&self) -> &str { "pipeline_flow_balance" }
}

impl PipelineFrame {
    /// Derive a pressure-differential residual sample.
    ///
    /// Expected differential is a domain-calibrated constant (or model output).
    /// Caller supplies the reference dP; DSFB does not solve Darcy–Weisbach.
    pub fn pressure_residual(&self, expected_dp: f64) -> ResidualSample {
        let observed_dp = self.inlet_pressure - self.outlet_pressure;
        ResidualSample::new(
            self.timestamp,
            observed_dp,
            expected_dp,
            "pipeline_delta_pressure",
        )
    }
}

/// Named reason codes specific to pipeline grammar events.
/// These are illustrative; operators map tokens to operational hypotheses.
pub mod pipeline_reasons {
    pub const RTTM_DRIFT: &str = "Sustained flow-balance drift; review RTTM model accuracy or metering calibration";
    pub const PUMP_SLEW: &str = "Rapid flow transient; correlate with pump start/stop or valve event log";
    pub const PRESSDROP_DRIFT: &str = "Sustained ΔP drift; review fouling, pigging schedule, or density variation";
}
