/// Rotating equipment frame sourced from the RPDBCS ESPset real vibration dataset.
///
/// Primary DSFB channel: broadband vibration RMS in the 98–102 Hz band.
/// Each frame is one vibration snapshot from one measurement instant on a
/// specific ESP unit.  The "timestamp" coordinate is the sequential sample
/// index within that unit's measurement series.
///
/// Source: RPDBCS ESPset (Real-world Pump Bearing Dataset with Classification
///         Support).  11 ESP units; 6 032 labeled vibration snapshots;
///         5 operating conditions: Normal, Unbalance, Rubbing, Misalignment,
///         Faulty sensor.
///         MIT License.
///         Primary reference: Gaboardi et al., "RPDBCS: A Real-World Pump
///         Bearing Dataset with Classification Support", 2024.
use crate::DsfbDomainFrame;

#[derive(Debug, Clone, Copy)]
pub struct EspFrame {
    /// Sequential sample index within this ESP unit's series (0-based).
    pub step: u32,
    /// ESP unit identifier (0–10).
    pub esp_id: u8,

    // ── Primary DSFB channel: broadband RMS ──────────────────────────────────
    /// Rolling-median baseline (15-sample causal window) of broadband RMS.
    pub baseline_rms: f64,
    /// Observed broadband vibration RMS at 98–102 Hz [dimensionless, g scale].
    pub rms_broadband: f64,

    // ── Auxiliary channels ───────────────────────────────────────────────────
    /// Vibration peak at fundamental (1×) running frequency.
    pub peak1x: f64,
    /// Vibration peak at 2× running frequency.
    pub peak2x: f64,
    /// Rolling-median baseline of peak1x.
    pub baseline_peak1x: f64,
    /// Low-frequency spectral median (8–13 Hz).
    pub median_8_13hz: f64,
    /// Spectrum regression slope coefficient.
    pub coeff_a: f64,
    /// Spectrum regression intercept coefficient.
    pub coeff_b: f64,
}

impl DsfbDomainFrame for EspFrame {
    /// Returns the sequential sample index as the frame timestamp.
    fn timestamp(&self) -> f64 { self.step as f64 }
    fn expected(&self)  -> f64 { self.baseline_rms }
    fn observed(&self)  -> f64 { self.rms_broadband }
    fn channel_name(&self) -> &str { "esp_rms_broadband" }
}
