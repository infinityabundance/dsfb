//! Drift / slew envelope semantics.
//!
//! The DSFB-native interpretation of *drift* is the EMA-smoothed residual
//! magnitude `s_k = ρ s_k + (1−ρ)|r_k|`; *slew* is the instantaneous
//! residual magnitude `|r_k|`. An envelope is a deterministic threshold band
//! over both: when `s_k > drift_threshold` we are in the drift phase; when
//! additionally `|r_k| > slew_threshold` we are at a boundary breach.
//!
//! These are the same definitions used in the `dsfb-semiconductor` and
//! `dsfb-oil-gas` companion crates; we restate them here for clarity rather
//! than re-import to avoid a cross-crate coupling that would make
//! `dsfb-database` harder to publish independently.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Envelope {
    /// Below drift threshold; residual is within the noise envelope.
    Stable,
    /// EMA over drift threshold; persistent low-amplitude excursion.
    Drift,
    /// Slew over slew threshold; an instantaneous boundary breach.
    Boundary,
}

pub fn classify(ema: f64, instant: f64, drift_threshold: f64, slew_threshold: f64) -> Envelope {
    let abs_e = ema.abs();
    let abs_i = instant.abs();
    if abs_i >= slew_threshold {
        Envelope::Boundary
    } else if abs_e >= drift_threshold {
        Envelope::Drift
    } else {
        Envelope::Stable
    }
}
