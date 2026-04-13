use crate::DsfbDomainFrame;

#[derive(Debug, Clone, Copy)]
pub struct SubseaFrame {
    pub timestamp: f64,
    pub expected_actuation_pressure: f64,
    pub observed_actuation_pressure: f64,
    pub valve_command: f64,
}

impl DsfbDomainFrame for SubseaFrame {
    fn timestamp(&self) -> f64 { self.timestamp }
    fn expected(&self) -> f64 { self.expected_actuation_pressure }
    fn observed(&self) -> f64 { self.observed_actuation_pressure }
    /// Override default to provide a descriptive channel name for episode logs.
    fn channel_name(&self) -> &str { "subsea_actuation_pressure" }
}
