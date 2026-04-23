//! Platform context: waveform transitions, SNR floor, regime suppression.
//!
//! This module provides the context information that allows the grammar layer
//! to suppress escalation during known-good transient windows (waveform
//! transitions, frequency hops, calibration periods) — analogous to the
//! `MaintenanceHysteresis` guard in the semiconductor domain.
//!
//! ## Failure Mode Mitigation (paper §XIV-C)
//!
//! Deliberate waveform transitions produce residual signatures that are
//! structurally indistinguishable from interference onset. The correct
//! integration contract includes a waveform-schedule context channel that
//! suppresses grammar escalation during flagged transition windows.
//!
//! Setting `WaveformState::Transition` returns `f32::INFINITY` from
//! `admissibility_multiplier()`, making envelope violations structurally
//! impossible during the transition window.

/// SNR floor marker. Observations below this floor are flagged as
/// sub-threshold and have drift/slew forced to zero.
///
/// Default: −10 dB (paper §L10, §IX-C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnrFloor {
    /// SNR floor in dB. Default −10.0.
    pub db: f32,
}

impl SnrFloor {
    /// Construct an SNR floor at `db` dB.
    pub const fn new(db: f32) -> Self {
        Self { db }
    }

    /// Returns true if the given SNR (dB) is below the floor.
    #[inline]
    pub fn is_sub_threshold(&self, snr_db: f32) -> bool {
        snr_db < self.db
    }
}

impl Default for SnrFloor {
    fn default() -> Self {
        Self { db: -10.0 }
    }
}

/// Current waveform/signal regime state.
///
/// Used to suppress grammar escalation during planned transitions,
/// preventing false episodes at every scheduled frequency hop,
/// modulation change, or burst boundary (paper §XIV-C, §IX-E).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformState {
    /// Normal signal operation. Grammar evaluation proceeds.
    Operational,
    /// Deliberate waveform transition window (hop, burst boundary,
    /// modulation change). Grammar escalation suppressed.
    /// Admissibility multiplier → +∞.
    Transition,
    /// Post-transition hysteresis guard window. Grammar accumulation
    /// suppressed for `guard_runs` remaining observations.
    /// Post-transition guard: grammar suppressed for `remaining` more observations.
    PostTransitionGuard {
        /// Observations remaining in guard window before returning to Operational.
        remaining: u16
    },
    /// Calibration window. Grammar evaluation suppressed.
    Calibration,
    /// Known co-site transmit-inhibit window.  The platform's own transmitter
    /// is active; structural violations during this period are expected
    /// artefacts of co-site interference, not external threats.
    ///
    /// The integration layer must set this state for the duration of every
    /// local transmit burst (see paper §L, item 17: Aperture Co-site
    /// Interference defence).  Grammar escalation is suppressed while this
    /// state is active; the platform context transition back to
    /// [`WaveformState::PostTransitionGuard`] automatically when the operator
    /// clears the inhibit.
    ///
    /// Admissibility multiplier → `+∞` (no violation possible).
    TransmitInhibit,
}

impl WaveformState {
    /// Returns the admissibility multiplier for this state.
    ///
    /// - `Operational`: 1.0 (normal envelope)
    /// - `Transition` / `Calibration`: f32::INFINITY (no violation possible)
    /// - `PostTransitionGuard`: f32::INFINITY until guard expires
    #[inline]
    pub fn admissibility_multiplier(&self) -> f32 {
        match self {
            WaveformState::Operational => 1.0,
            WaveformState::Transition => f32::INFINITY,
            WaveformState::PostTransitionGuard { .. } => f32::INFINITY,
            WaveformState::Calibration => f32::INFINITY,
            WaveformState::TransmitInhibit => f32::INFINITY,
        }
    }

    /// Returns true if grammar state assignment is suppressed.
    #[inline]
    pub fn is_suppressed(&self) -> bool {
        matches!(
            self,
            WaveformState::Transition
                | WaveformState::PostTransitionGuard { .. }
                | WaveformState::Calibration
                | WaveformState::TransmitInhibit
        )
    }

    /// Advance the state by one observation tick.
    /// `PostTransitionGuard { remaining: 0 }` transitions to `Operational`.
    #[must_use]
    pub fn tick(self) -> Self {
        match self {
            WaveformState::PostTransitionGuard { remaining: 0 } => WaveformState::Operational,
            WaveformState::PostTransitionGuard { remaining } => {
                WaveformState::PostTransitionGuard { remaining: remaining - 1 }
            }
            // TransmitInhibit persists until the integration layer explicitly
            // clears it (transitions to Operational or PostTransitionGuard).
            other => other,
        }
    }
}

/// Complete platform context passed to the engine on each observe() call.
///
/// This is the read-only context channel described in paper §XIV-C.
/// It is populated by the integration layer (e.g., the GNU Radio sink block)
/// and consumed by the engine. DSFB does not write to this struct.
#[derive(Debug, Clone, Copy)]
pub struct PlatformContext {
    /// Current SNR estimate in dB. Use `f32::NAN` if unknown.
    pub snr_db: f32,
    /// Current waveform state (transition suppression context).
    pub waveform_state: WaveformState,
    /// Post-transition guard duration (observations). Default: 5.
    pub post_transition_guard: u16,
}

impl PlatformContext {
    /// Create a nominal operational context (no suppression, SNR = +20 dB).
    pub const fn operational() -> Self {
        Self {
            snr_db: 20.0,
            waveform_state: WaveformState::Operational,
            post_transition_guard: 5,
        }
    }

    /// Create a context with specified SNR and operational state.
    pub const fn with_snr(snr_db: f32) -> Self {
        Self {
            snr_db,
            waveform_state: WaveformState::Operational,
            post_transition_guard: 5,
        }
    }

    /// Create a transition-suppressed context.
    pub const fn transition() -> Self {
        Self {
            snr_db: f32::NAN,
            waveform_state: WaveformState::Transition,
            post_transition_guard: 5,
        }
    }
}

impl Default for PlatformContext {
    fn default() -> Self {
        Self::operational()
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snr_floor_sub_threshold() {
        let floor = SnrFloor::new(-10.0);
        assert!(floor.is_sub_threshold(-15.0));
        assert!(!floor.is_sub_threshold(-5.0));
        assert!(!floor.is_sub_threshold(0.0));
    }

    #[test]
    fn transition_multiplier_is_infinite() {
        assert!(WaveformState::Transition.admissibility_multiplier().is_infinite());
        assert!(WaveformState::Calibration.admissibility_multiplier().is_infinite());
        assert_eq!(WaveformState::Operational.admissibility_multiplier(), 1.0);
    }

    #[test]
    fn guard_ticks_to_operational() {
        let s = WaveformState::PostTransitionGuard { remaining: 2 };
        let s1 = s.tick();
        assert_eq!(s1, WaveformState::PostTransitionGuard { remaining: 1 });
        let s2 = s1.tick();
        assert_eq!(s2, WaveformState::PostTransitionGuard { remaining: 0 });
        let s3 = s2.tick();
        assert_eq!(s3, WaveformState::Operational);
    }

    #[test]
    fn suppressed_states() {
        assert!(WaveformState::Transition.is_suppressed());
        assert!(WaveformState::Calibration.is_suppressed());
        assert!(WaveformState::PostTransitionGuard { remaining: 3 }.is_suppressed());
        assert!(!WaveformState::Operational.is_suppressed());
    }
}
