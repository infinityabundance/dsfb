/// Oil-well production frame sourced from the Petrobras 3W Dataset.
///
/// Primary DSFB channel: upstream production-choke pressure (P-MON-CKP).
/// Auxiliary channels retained for multi-sensor correlation.
///
/// Source: Petrobras 3W Dataset v2.0.0 — real WELL-* instances only.
///         <https://github.com/petrobras/3W>  CC BY 4.0.
use crate::DsfbDomainFrame;

#[derive(Debug, Clone, Copy)]
pub struct OilwellFrame {
    /// Seconds from episode start (60-s resolution).
    pub timestamp: f64,

    // ── Primary DSFB channel: production-choke upstream pressure ─────────────
    /// 30-min rolling-median baseline of P-MON-CKP [Pa].
    pub expected_choke_pa: f64,
    /// 60-s median observation of P-MON-CKP [Pa].
    pub observed_choke_pa: f64,

    // ── Auxiliary channels (NaN when sensor unavailable in source file) ───────
    /// Well annulus pressure P-ANULAR [Pa].
    pub observed_annulus_pa: f64,
    /// Subsea Xmas-tree pressure P-TPT [Pa].
    pub observed_xmas_pa: f64,
    /// Subsea Xmas-tree temperature T-TPT [°C].
    pub observed_xmas_degc: f64,

    /// 3W majority event class label (0 = normal; 2,3,4,7,8,9 = events;
    /// 1xx = transient phase preceding event).
    pub event_class: i16,
}

impl DsfbDomainFrame for OilwellFrame {
    fn timestamp(&self) -> f64 { self.timestamp }
    fn expected(&self) -> f64  { self.expected_choke_pa }
    fn observed(&self) -> f64  { self.observed_choke_pa }
    fn channel_name(&self) -> &str { "oilwell_choke_pressure_pa" }
}
