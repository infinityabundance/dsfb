//! Channel definitions for gas turbine sensor mapping.
//!
//! Maps C-MAPSS sensor indices to physically meaningful channel identifiers.
//! All types are `Copy` and stack-allocated.

/// Identifies a sensor channel in the C-MAPSS / engine health dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelId {
    /// s2: Total temperature at LPC outlet (T24)
    TempLpcOutlet,
    /// s3: Total temperature at HPC outlet (T30)
    TempHpcOutlet,
    /// s4: Total temperature at LPT outlet (T50) — EGT proxy
    TempLptOutlet,
    /// s7: Total pressure at HPC outlet (P30)
    PressureHpcOutlet,
    /// s8: Physical fan speed (Nf)
    FanSpeed,
    /// s9: Physical core speed (Nc)
    CoreSpeed,
    /// s11: Static pressure at HPC outlet (Ps30)
    StaticPressureHpc,
    /// s12: Fuel flow / Ps30 ratio (phi) — efficiency proxy
    FuelFlowRatio,
    /// s13: Corrected fan speed (NRf)
    CorrectedFanSpeed,
    /// s14: Corrected core speed (NRc)
    CorrectedCoreSpeed,
    /// s15: Bypass ratio (BPR)
    BypassRatio,
    /// s17: Bleed enthalpy
    BleedEnthalpy,
    /// s20: HPT coolant bleed
    HptCoolantBleed,
    /// s21: LPT coolant bleed
    LptCoolantBleed,
}

impl ChannelId {
    /// Returns the 0-based sensor index in C-MAPSS data (columns 5..26 → indices 0..20).
    #[must_use]
    pub const fn cmapss_sensor_index(self) -> usize {
        match self {
            Self::TempLpcOutlet => 1,      // s2
            Self::TempHpcOutlet => 2,      // s3
            Self::TempLptOutlet => 3,      // s4
            Self::PressureHpcOutlet => 6,  // s7
            Self::FanSpeed => 7,           // s8
            Self::CoreSpeed => 8,          // s9
            Self::StaticPressureHpc => 10, // s11
            Self::FuelFlowRatio => 11,     // s12
            Self::CorrectedFanSpeed => 12, // s13
            Self::CorrectedCoreSpeed => 13,// s14
            Self::BypassRatio => 14,       // s15
            Self::BleedEnthalpy => 16,     // s17
            Self::HptCoolantBleed => 19,   // s20
            Self::LptCoolantBleed => 20,   // s21
        }
    }

    /// Human-readable label for this channel.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TempLpcOutlet => "T24 (LPC outlet temp)",
            Self::TempHpcOutlet => "T30 (HPC outlet temp)",
            Self::TempLptOutlet => "T50 (LPT outlet temp / EGT proxy)",
            Self::PressureHpcOutlet => "P30 (HPC outlet pressure)",
            Self::FanSpeed => "Nf (fan speed)",
            Self::CoreSpeed => "Nc (core speed)",
            Self::StaticPressureHpc => "Ps30 (HPC static pressure)",
            Self::FuelFlowRatio => "phi (fuel flow / Ps30)",
            Self::CorrectedFanSpeed => "NRf (corrected fan speed)",
            Self::CorrectedCoreSpeed => "NRc (corrected core speed)",
            Self::BypassRatio => "BPR (bypass ratio)",
            Self::BleedEnthalpy => "Bleed enthalpy",
            Self::HptCoolantBleed => "HPT coolant bleed",
            Self::LptCoolantBleed => "LPT coolant bleed",
        }
    }
}

/// The set of informative channels for HPC degradation detection on FD001.
/// These are the sensors that show meaningful degradation trends.
/// Sensors s1, s5, s6, s10, s16, s18, s19 are excluded (near-constant in FD001).
pub const INFORMATIVE_CHANNELS_FD001: &[ChannelId] = &[
    ChannelId::TempLpcOutlet,
    ChannelId::TempHpcOutlet,
    ChannelId::TempLptOutlet,
    ChannelId::PressureHpcOutlet,
    ChannelId::FanSpeed,
    ChannelId::CoreSpeed,
    ChannelId::StaticPressureHpc,
    ChannelId::FuelFlowRatio,
    ChannelId::CorrectedFanSpeed,
    ChannelId::CorrectedCoreSpeed,
    ChannelId::BypassRatio,
    ChannelId::BleedEnthalpy,
    ChannelId::HptCoolantBleed,
    ChannelId::LptCoolantBleed,
];

/// A single sensor reading at one cycle, with regime metadata.
#[derive(Debug, Clone, Copy)]
pub struct SensorReading {
    /// Engine unit number.
    pub unit: u16,
    /// Cycle index (1-based, as in C-MAPSS).
    pub cycle: u32,
    /// Operational setting 1 (altitude proxy).
    pub op_setting_1: f64,
    /// Operational setting 2 (Mach number proxy).
    pub op_setting_2: f64,
    /// Operational setting 3 (throttle resolver angle proxy).
    pub op_setting_3: f64,
    /// Sensor values (21 channels, 0-indexed).
    pub sensors: [f64; 21],
}
