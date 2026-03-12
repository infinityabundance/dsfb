use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RegimeLabel {
    Baseline,
    Degradation,
    Shock,
    Recovery,
}

impl RegimeLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Degradation => "degradation",
            Self::Shock => "shock",
            Self::Recovery => "recovery",
        }
    }
}

impl fmt::Display for RegimeLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

#[derive(Clone, Debug)]
pub struct StructuralEvent {
    pub event_id: usize,
    pub time_index: usize,
    pub channel_id: usize,
    pub latent_state: f64,
    pub predicted_value: f64,
    pub observed_value: f64,
    pub residual: f64,
    pub envelope: f64,
    pub trust: f64,
    pub regime_label: RegimeLabel,
}
