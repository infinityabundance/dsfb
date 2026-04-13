/// Volve depth-indexed drilling frame sourced from Equinor Volve WITSML data.
///
/// Primary DSFB channel: surface torque TQA.
/// The "timestamp" coordinate is measured depth (m MD) — the native index
/// of WITSML depth-indexed logs — resampled to 0.5-m depth steps.
///
/// Source: Equinor Volve Data Village, well 15/9-F-15.
///         Original data licensed under the Equinor Volve Data Licence V1.0.
///         <https://data.equinor.com/dataset/Volve>
///         Channels: TQA (kNm), SWOB (kN), RPM (rpm), HKLD (kN), SPPA (kPa).
///         Depth range extracted: 1 200 – 4 065 m MD.
///         SWOB and HKLD converted from source kkgf (metric tonne-force × 9.80665).
use crate::DsfbDomainFrame;

#[derive(Debug, Clone, Copy)]
pub struct VolveFrame {
    /// Measured depth (m MD) — serves as the monotone frame index.
    pub depth_m: f64,

    // ── Primary DSFB channel: surface torque ─────────────────────────────────
    /// Median baseline of TQA over a 20-sample (10-m) window [kNm].
    pub baseline_tqa_knm: f64,
    /// Observed surface torque TQA [kNm].
    pub observed_tqa_knm: f64,

    // ── Auxiliary operational channels ───────────────────────────────────────
    /// Weight on bit SWOB [kN].  Converted from source kkgf ×9.80665.
    pub swob_kn: f64,
    /// Rotary RPM [rpm].
    pub rpm: f64,
    /// Hook load HKLD [kN].  Converted from source kkgf ×9.80665.
    pub hkld_kn: f64,
    /// Standpipe pressure SPPA [kPa].
    pub sppa_kpa: f64,
}

impl DsfbDomainFrame for VolveFrame {
    /// Returns measured depth (m MD) as the monotone index.
    fn timestamp(&self) -> f64 { self.depth_m }
    fn expected(&self)  -> f64 { self.baseline_tqa_knm }
    fn observed(&self)  -> f64 { self.observed_tqa_knm }
    fn channel_name(&self) -> &str { "volve_surface_torque_knm" }
}
