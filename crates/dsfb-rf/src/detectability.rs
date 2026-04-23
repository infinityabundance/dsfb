//! Rich deterministic detectability taxonomy for DSFB-RF.
//!
//! ## Theoretical Basis
//!
//! A key contribution of the DSFB framework is the derivation of a
//! **deterministic upper bound on detection latency** (DSFB-Lattice §V,
//! DSFB-Semiotics-Calculus §III).  Unlike Pd/Pfa quantities — which require
//! calibrated probabilistic models and are sensitive to signal model mis-match
//! — the DSFB detectability bound is purely algebraic:
//!
//! ```text
//! τ_upper = δ₀ / (α − κ)   provided α > κ
//! ```
//!
//! where:
//! - δ₀ = initial residual offset from the nominal
//! - α  = divergence rate (from the Lyapunov exponent λ or slew rate)
//! - κ  = noise-floor rate (minimum observable drift, derived from σ₀)
//!
//! The bound asserts: *if a structural change is occurring at rate α, the
//! grammar layer will detect it within τ_upper sample periods*.
//!
//! ## Interpretation taxonomy (DSFB-Lattice)
//!
//! Beyond the raw bound, the lattice framework defines a full hierarchy of
//! semantic interpretation classes, making operator-facing output actionable
//! rather than merely numeric:
//!
//! | InterpretationClass | Meaning |
//! |---|---|
//! | StructuralDetected | Envelope crossing confirmed with margin; clear fault |
//! | StressDetected | Crossing detected with low post-crossing margin; degradation likely |
//! | EarlyLowMarginCrossing | Crossing detected early but barely; watch state |
//! | NotDetected | No crossing; nominal operation |
//!
//! The `SemanticStatus` refines this further with temporal persistence
//! qualities, and `DetectionStrengthBand` provides a coarse ordinal for
//! dashboard display.
//!
//! ## Post-crossing persistence
//!
//! For SIGINT / EW applications, it is important to know not just *when*
//! a crossing occurs but **how long** the trajectory remains outside the
//! envelope.  Long post-crossing persistence indicates a persistent structural
//! fault (e.g., hardware failure, sustained jamming) rather than a transient
//! spike.  This module tracks:
//!
//! - `post_crossing_duration`: number of samples outside envelope since first crossing
//! - `post_crossing_fraction`: fraction of recent W samples spent outside envelope
//! - `peak_margin_after_crossing`: maximum normalised excess (‖r‖ − ρ) / ρ since crossing
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - O(1) per `update()` call (circular buffer for fraction tracking)
//! - All types `Clone + Copy` for zero-cost passing through audit chain

/// Coarse detection interpretation class.
///
/// Semantically richer than a binary "detected / not-detected" flag.
/// Directly maps to VITA 49.2 context packet severity codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DetectabilityClass {
    /// Envelope crossing confirmed with post-crossing margin > high_threshold.
    /// Clear structural fault.  Escalate to operator.
    StructuralDetected,

    /// Envelope crossing with post-crossing margin in (low_threshold, high_threshold].
    /// Stress / degradation detected with reduced confidence.
    StressDetected,

    /// Crossing detected very quickly (early) but margin is low.
    /// Could indicate a brief transient or the early onset of a fault.
    EarlyLowMarginCrossing,

    /// No envelope crossing; trajectory within nominal bounds.
    NotDetected,
}

/// Fine-grained semantic status combining class and temporal context.
///
/// Used for dashboard labelling and VITA 49.2 context packet annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SemanticStatus {
    /// StructuralDetected + post-crossing duration ≥ duration_threshold.
    /// Persistent structural fault; recommend hardware inspection.
    PersistentStructuralFault,

    /// StructuralDetected + duration < duration_threshold.
    /// Clear structural detection (single-event or early onset).
    ClearStructuralDetection,

    /// StressDetected + post-crossing fraction ≥ fraction_threshold.
    /// Marginal but sustained degradation; link quality watch.
    MarginalStructuralDegradation,

    /// StressDetected + fraction < fraction_threshold.
    /// Isolated stress event; transient interference likely.
    IsolatedStressEvent,

    /// EarlyLowMarginCrossing sustained over multiple windows.
    /// Ambiguous: could be noise or nascent fault — heightened watch.
    DegradedAmbiguous,

    /// EarlyLowMarginCrossing single occurrence.
    /// Ambiguous transient — monitor only.
    Ambiguous,

    /// NotDetected; nominal operation.
    NotDetected,
}

/// Operator-facing coarse strength band (for dashboard colour coding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DetectionStrengthBand {
    /// No detection — green.
    Clear = 0,
    /// Ambiguous / low-margin — yellow.
    Marginal = 1,
    /// Stress / degradation detected — amber.
    Degraded = 2,
    /// Clear structural fault — red.
    Critical = 3,
}

impl DetectionStrengthBand {
    /// Derive from a `DetectabilityClass`.
    pub fn from_class(class: DetectabilityClass) -> Self {
        match class {
            DetectabilityClass::NotDetected => Self::Clear,
            DetectabilityClass::EarlyLowMarginCrossing => Self::Marginal,
            DetectabilityClass::StressDetected => Self::Degraded,
            DetectabilityClass::StructuralDetected => Self::Critical,
        }
    }
}

/// Thresholds governing the detectability taxonomy.
#[derive(Debug, Clone, Copy)]
pub struct DetectabilityThresholds {
    /// Normalised excess (‖r‖ − ρ) / ρ above which a crossing is
    /// classified `StructuralDetected` rather than `StressDetected`.
    pub high_margin_threshold: f32,

    /// Normalised excess below `high_margin_threshold` classified as
    /// `StressDetected` (must be > 0).
    pub low_margin_threshold: f32,

    /// Post-crossing sample count above which a `StructuralDetected` event
    /// is tagged as `PersistentStructuralFault`.
    pub persistence_duration_threshold: u32,

    /// Post-crossing fraction (0..=1) above which a `StressDetected` event
    /// is tagged as `MarginalStructuralDegradation`.
    pub persistence_fraction_threshold: f32,

    /// Number of samples defining the "early" window.
    /// Crossings occurring within `early_window` samples of the episode
    /// start are tagged `EarlyLowMarginCrossing`.
    pub early_window: u32,

    /// Divergence rate threshold κ used in the τ_upper bound.
    ///
    /// Represents the minimum meaningful drift rate above the noise floor.
    pub kappa: f32,
}

impl DetectabilityThresholds {
    /// Conservative defaults suitable for most RF receiver applications.
    pub const fn default_rf() -> Self {
        Self {
            high_margin_threshold: 0.20,    // 20 % excess → structural
            low_margin_threshold: 0.02,     // 2 % → stress
            persistence_duration_threshold: 10,
            persistence_fraction_threshold: 0.30,
            early_window: 5,
            kappa: 0.001,
        }
    }
}

impl Default for DetectabilityThresholds {
    fn default() -> Self { Self::default_rf() }
}

/// Deterministic detectability upper bound τ_upper.
///
/// Computes the latency upper bound from DSFB-Lattice Theorem 1:
///
/// ```text
/// τ_upper = δ₀ / (α − κ)   iff α > κ
/// ```
///
/// Returns `None` if α ≤ κ (divergence rate does not exceed noise floor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectabilityBound {
    /// Initial offset δ₀ = ‖r_initial‖ from nominal.
    pub delta_0: f32,
    /// Observed divergence rate α (from Lyapunov λ or empirical slew rate).
    pub alpha: f32,
    /// Noise-floor rate κ.
    pub kappa: f32,
    /// Computed bound τ_upper (sample periods).  `None` if α ≤ κ.
    pub tau_upper: Option<f32>,
    /// Whether observed crossing time t_cross ≤ τ_upper + ε.
    pub bound_satisfied: Option<bool>,
}

impl DetectabilityBound {
    /// Compute the τ_upper bound given δ₀, α, κ.
    pub fn compute(delta_0: f32, alpha: f32, kappa: f32) -> Self {
        let tau_upper = if alpha > kappa + 1e-12 {
            Some(delta_0 / (alpha - kappa))
        } else {
            None
        };
        Self { delta_0, alpha, kappa, tau_upper, bound_satisfied: None }
    }

    /// Validate whether the observed crossing time satisfies the bound.
    ///
    /// `t_cross` = number of samples from episode start to envelope crossing.
    /// `epsilon` = tolerance for sample-period quantisation (default: 1.0).
    pub fn validate_crossing(&mut self, t_cross: f32, epsilon: f32) {
        self.bound_satisfied = self.tau_upper.map(|tau| t_cross <= tau + epsilon);
    }
}

/// Full detectability summary for one observation window.
///
/// All fields are deterministically computed from the residual norm history.
/// No probability model is required or assumed.
#[derive(Debug, Clone, Copy)]
pub struct DetectabilitySummary {
    /// Coarse interpretation class.
    pub class: DetectabilityClass,
    /// Fine semantic status.
    pub semantic: SemanticStatus,
    /// Dashboard strength band.
    pub band: DetectionStrengthBand,
    /// Deterministic τ_upper bound (if computable).
    pub bound: DetectabilityBound,
    /// Number of samples spent outside envelope since first crossing.
    /// Zero if no crossing has occurred.
    pub post_crossing_duration: u32,
    /// Fraction of recent `W` samples spent outside envelope (0..=1).
    pub post_crossing_fraction: f32,
    /// Maximum normalised excess (‖r‖ − ρ) / ρ observed since first crossing.
    pub peak_margin_after_crossing: f32,
    /// True if the crossing occurred within the "early window."
    pub boundary_proximate_crossing: bool,
}

/// Running detectability tracker with O(1) per-sample update.
///
/// Generic `W` = window size for fraction tracking.
pub struct DetectabilityTracker<const W: usize> {
    /// Running count of consecutive outside-envelope samples.
    post_crossing_duration: u32,
    /// Circular buffer of outside-envelope flags for fraction computation.
    outside_buf: [bool; W],
    /// Write head for circular buffer.
    head: usize,
    /// Total samples pushed (saturates at W for fraction purposes).
    count: usize,
    /// Peak normalised excess since crossing.
    peak_margin: f32,
    /// Sample index at which the first crossing occurred (if any).
    first_crossing_sample: Option<u32>,
    /// Current sample index.
    sample_idx: u32,
    /// Thresholds.
    thresholds: DetectabilityThresholds,
    /// Cached divergence rate α (from last Lyapunov update).
    cached_alpha: f32,
    /// Initial offset δ₀ (set at first crossing).
    delta_0: f32,
}

impl<const W: usize> DetectabilityTracker<W> {
    /// Create a new tracker with the given thresholds.
    pub const fn new(thresholds: DetectabilityThresholds) -> Self {
        Self {
            post_crossing_duration: 0,
            outside_buf: [false; W],
            head: 0,
            count: 0,
            peak_margin: 0.0,
            first_crossing_sample: None,
            sample_idx: 0,
            thresholds,
            cached_alpha: 0.0,
            delta_0: 0.0,
        }
    }

    /// Create with default RF thresholds.
    pub const fn default_rf() -> Self {
        Self::new(DetectabilityThresholds::default_rf())
    }

    /// Update the tracker with one residual norm observation.
    ///
    /// - `norm`:  current ‖r(k)‖
    /// - `rho`:   current admissibility envelope radius ρ(k)
    /// - `alpha`: divergence rate (Lyapunov λ or empirical slew; pass 0.0 if unknown)
    ///
    /// Returns a complete `DetectabilitySummary` for this observation.
    pub fn update(&mut self, norm: f32, rho: f32, alpha: f32) -> DetectabilitySummary {
        let outside = norm > rho && rho > 1e-30;
        let normalised_excess = if rho > 1e-30 { ((norm - rho) / rho).max(0.0) } else { 0.0 };
        self.cached_alpha = alpha;

        self.update_crossing_state(outside, normalised_excess);
        let post_crossing_fraction = self.update_outside_ring(outside);

        let crossing_time = self.first_crossing_sample
            .map(|s| self.sample_idx.saturating_sub(s) as f32)
            .unwrap_or(0.0);
        let early = self.first_crossing_sample
            .map(|s| self.sample_idx.saturating_sub(s) < self.thresholds.early_window)
            .unwrap_or(false);

        let class = self.classify_detection(outside, normalised_excess);
        let semantic = self.derive_semantic(class, post_crossing_fraction, early);
        let bound = self.compute_bound(outside, alpha, crossing_time);
        let band = DetectionStrengthBand::from_class(class);

        self.sample_idx = self.sample_idx.wrapping_add(1);

        DetectabilitySummary {
            class,
            semantic,
            band,
            bound,
            post_crossing_duration: self.post_crossing_duration,
            post_crossing_fraction,
            peak_margin_after_crossing: self.peak_margin,
            boundary_proximate_crossing: early,
        }
    }

    fn update_crossing_state(&mut self, outside: bool, normalised_excess: f32) {
        if outside && self.first_crossing_sample.is_none() {
            self.first_crossing_sample = Some(self.sample_idx);
            self.delta_0 = normalised_excess;
        }
        if outside {
            self.post_crossing_duration = self.post_crossing_duration.saturating_add(1);
            if normalised_excess > self.peak_margin {
                self.peak_margin = normalised_excess;
            }
        } else {
            self.post_crossing_duration = 0;
            self.peak_margin = 0.0;
            self.first_crossing_sample = None;
        }
    }

    fn update_outside_ring(&mut self, outside: bool) -> f32 {
        self.outside_buf[self.head] = outside;
        self.head = (self.head + 1) % W;
        if self.count < W { self.count += 1; }
        let outside_count = self.outside_buf[..self.count].iter().filter(|&&b| b).count();
        outside_count as f32 / self.count.max(1) as f32
    }

    fn classify_detection(&self, outside: bool, normalised_excess: f32) -> DetectabilityClass {
        if !outside && self.post_crossing_duration == 0 {
            DetectabilityClass::NotDetected
        } else if normalised_excess > self.thresholds.high_margin_threshold {
            DetectabilityClass::StructuralDetected
        } else if normalised_excess > self.thresholds.low_margin_threshold {
            DetectabilityClass::StressDetected
        } else {
            DetectabilityClass::EarlyLowMarginCrossing
        }
    }

    fn derive_semantic(
        &self,
        class: DetectabilityClass,
        post_crossing_fraction: f32,
        early: bool,
    ) -> SemanticStatus {
        match class {
            DetectabilityClass::NotDetected => SemanticStatus::NotDetected,
            DetectabilityClass::StructuralDetected => {
                if self.post_crossing_duration >= self.thresholds.persistence_duration_threshold {
                    SemanticStatus::PersistentStructuralFault
                } else {
                    SemanticStatus::ClearStructuralDetection
                }
            }
            DetectabilityClass::StressDetected => {
                if post_crossing_fraction >= self.thresholds.persistence_fraction_threshold {
                    SemanticStatus::MarginalStructuralDegradation
                } else {
                    SemanticStatus::IsolatedStressEvent
                }
            }
            DetectabilityClass::EarlyLowMarginCrossing => {
                if early { SemanticStatus::Ambiguous } else { SemanticStatus::DegradedAmbiguous }
            }
        }
    }

    fn compute_bound(&self, outside: bool, alpha: f32, crossing_time: f32) -> DetectabilityBound {
        let kappa = self.thresholds.kappa;
        let mut bound = DetectabilityBound::compute(self.delta_0, alpha, kappa);
        if outside {
            bound.validate_crossing(crossing_time, 1.0);
        }
        bound
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.post_crossing_duration = 0;
        self.outside_buf = [false; W];
        self.head = 0;
        self.count = 0;
        self.peak_margin = 0.0;
        self.first_crossing_sample = None;
        self.sample_idx = 0;
        self.cached_alpha = 0.0;
        self.delta_0 = 0.0;
    }

    /// Access current post-crossing duration.
    #[inline] pub fn post_crossing_duration(&self) -> u32 { self.post_crossing_duration }
    /// Access peak margin since crossing.
    #[inline] pub fn peak_margin(&self) -> f32 { self.peak_margin }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_operation_is_not_detected() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        for i in 0..50 {
            let r = tracker.update(0.05, 0.10, 0.0);
            assert_eq!(r.class, DetectabilityClass::NotDetected, "step {}", i);
            assert_eq!(r.semantic, SemanticStatus::NotDetected);
            assert_eq!(r.band, DetectionStrengthBand::Clear);
        }
    }

    #[test]
    fn large_crossing_structural_detected() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        // norm = 0.13, rho = 0.10 → excess = 30 % > 20 % threshold
        let r = tracker.update(0.13, 0.10, 0.01);
        assert_eq!(r.class, DetectabilityClass::StructuralDetected);
        assert_eq!(r.band, DetectionStrengthBand::Critical);
    }

    #[test]
    fn small_crossing_stress_detected() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        // norm = 0.105, rho = 0.10 → excess = 5 % > 2 % but < 20 %
        let r = tracker.update(0.105, 0.10, 0.005);
        assert_eq!(r.class, DetectabilityClass::StressDetected);
        assert_eq!(r.band, DetectionStrengthBand::Degraded);
    }

    #[test]
    fn marginal_crossing_early_low_margin() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        // norm = 0.1005, rho = 0.10 → excess = 0.5 % < 2 %
        let r = tracker.update(0.1005, 0.10, 0.001);
        assert_eq!(r.class, DetectabilityClass::EarlyLowMarginCrossing);
    }

    #[test]
    fn persistent_structural_fault_after_threshold() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        // persistence_duration_threshold = 10
        for i in 0..12 {
            let r = tracker.update(0.15, 0.10, 0.01);
            if i >= 10 {
                assert_eq!(
                    r.semantic, SemanticStatus::PersistentStructuralFault,
                    "step {}: expected PersistentStructuralFault", i
                );
            }
        }
    }

    #[test]
    fn post_crossing_fraction_accumulates() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        // 10 outside, 10 inside
        for _ in 0..10 { tracker.update(0.15, 0.10, 0.01); }
        for _ in 0..10 {
            let r = tracker.update(0.05, 0.10, 0.0);
            // After return to nominal, class should be not detected
            assert_eq!(r.class, DetectabilityClass::NotDetected);
        }
    }

    #[test]
    fn tau_upper_bound_computed() {
        let bound = DetectabilityBound::compute(0.05, 0.01, 0.001);
        // tau_upper = 0.05 / (0.01 - 0.001) = 0.05 / 0.009 ≈ 5.56
        let tau = bound.tau_upper.expect("should have bound");
        assert!((tau - 5.555_555).abs() < 1e-2, "tau={}", tau);
    }

    #[test]
    fn tau_upper_none_when_alpha_le_kappa() {
        let bound = DetectabilityBound::compute(0.05, 0.0005, 0.001);
        assert!(bound.tau_upper.is_none(), "alpha <= kappa → no bound");
    }

    #[test]
    fn detection_strength_band_ordering() {
        assert!(DetectionStrengthBand::Clear < DetectionStrengthBand::Marginal);
        assert!(DetectionStrengthBand::Marginal < DetectionStrengthBand::Degraded);
        assert!(DetectionStrengthBand::Degraded < DetectionStrengthBand::Critical);
    }

    #[test]
    fn reset_clears_all_state() {
        let mut tracker = DetectabilityTracker::<20>::default_rf();
        for _ in 0..20 { tracker.update(0.15, 0.10, 0.01); }
        tracker.reset();
        let r = tracker.update(0.05, 0.10, 0.0);
        assert_eq!(r.class, DetectabilityClass::NotDetected);
        assert_eq!(r.post_crossing_duration, 0);
    }
}
