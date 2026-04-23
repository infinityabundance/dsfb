//! RF disturbance taxonomy: classification and envelope compatibility.
//!
//! ## Theoretical basis
//!
//! The DSFB-DDMF framework (de Beer 2026, Deterministic Disturbance
//! Modelling Framework) establishes a taxonomy of disturbance types that
//! affect residual norms in a predictable, classifiable way.  This module
//! adapts that taxonomy to the specific context of RF receivers, mapping
//! each DDMF class to its RF physical mechanism.
//!
//! The taxonomy is **not probabilistic**: the parameters describe
//! *worst-case bounds* on disturbance magnitude and rate, not distribution
//! parameters.  This is what makes the resulting envelope bounds GUM-traceable
//! and deterministic under the DSFB framework.
//!
//! ## Taxonomy overview
//!
//! | Disturbance class | DDMF bound type | RF physical mechanism |
//! |---|---|---|
//! | PointwiseBounded | ‖d(k)‖ ≤ d_max all k | Thermal/Johnson–Nyquist noise, ADC dither |
//! | Drift | ‖d(k)‖ ≤ b + s_max·k | LO frequency drift, PA thermal drift |
//! | SlewRateBounded | ‖Δd(k)‖ ≤ s_max | Slow AGC transient, temperature ramp |
//! | Impulsive | spike at specific time, bounded amplitude | Jamming onset, ESD, lightning near-field |
//! | PersistentElevated | step change to sustained high level | CW interference, in-band carrier, co-site blocker |
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - All types `Clone + Copy`
//! - Optional `serde` feature for JSON serialisation into SigMF annotations
//! - The `classify()` function provides a heuristic assignment from observed
//!   residual statistics (from the DSA score / grammar outputs)

/// DDMF disturbance class with RF-specific parameters.
///
/// Each variant holds the worst-case bound parameters for that class.
/// The parameter names mirror the DSFB-DDMF notation exactly for
/// cross-reference traceability.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RfDisturbance {
    /// Pointwise-bounded disturbance: ‖d(k)‖ ≤ d_max for all k.
    ///
    /// RF interpretation: thermal noise floor, ADC quantisation dither.
    /// DDMF parameter: `d_max` in normalised residual norm units.
    ///
    /// The admissibility envelope radius is valid as calibrated when this
    /// is the only active disturbance class (no additional envelope expansion
    /// is required beyond the 3σ nominal margin).
    PointwiseBounded {
        /// Maximum instantaneous disturbance magnitude d_max.
        d_max: f32,
    },

    /// Drift disturbance: ‖d(k)‖ ≤ b + s_max · k.
    ///
    /// RF interpretation: LO frequency drift (±n Hz / s), PA thermal drift,
    /// slow aging of calibration state.
    ///
    /// DDMF parameters: b = initial offset, s_max = maximum drift slope.
    /// Envelope action: envelope must be widened at rate s_max per sample
    /// to remain a valid bound. Recommend `EnvelopeMode::Widening` from
    /// `regime` module.
    Drift {
        /// Initial bias b (offset at k=0, normalised).
        b: f32,
        /// Maximum drift slope s_max (normalised units per sample).
        s_max: f32,
    },

    /// Slew-rate-bounded disturbance: ‖d(k) − d(k−1)‖ ≤ s_max.
    ///
    /// RF interpretation: slow automatic gain control (AGC) transient,
    /// temperature-driven gain variation, antenna pattern scan.
    /// Bounds the *rate of change* rather than the absolute magnitude.
    ///
    /// Note: a SlewRateBounded disturbance can still have large accumulated
    /// magnitude if sustained long enough; pair with a Drift bound for
    /// long-duration validity.
    SlewRateBounded {
        /// Maximum per-sample change s_max (normalised).
        s_max: f32,
    },

    /// Impulsive disturbance: a single large spike over a bounded window.
    ///
    /// RF interpretation: jamming onset pulse, near-field EMP, ESD event,
    /// radar pulse (non-self) cross-coupling, lightning discharge.
    ///
    /// The DSFB grammar layer naturally handles these via the
    /// `AbruptSlewViolation` reason code.  The `Impulsive` class provides
    /// the adversary model that bounds the spike amplitude and duration.
    Impulsive {
        /// Peak amplitude A (normalised residual norm units).
        amplitude: f32,
        /// Onset sample index (samples since epoch, wrapping).
        start_sample: u32,
        /// Duration in samples (window during which amplitude ≤ A).
        duration_samples: u32,
    },

    /// Persistent elevated disturbance: step to sustained elevated residual.
    ///
    /// RF interpretation: continuous-wave (CW) in-band interference,
    /// broadband noise jammer, co-site RF blocker, transmitter failure
    /// in radiating mode.
    ///
    /// This is the most operationally significant class for SIGINT / EW
    /// applications because a persistent elevated residual is often
    /// indistinguishable from a modulation change without the DSFB framework.
    PersistentElevated {
        /// Nominal (pre-step) residual norm level r_nom.
        r_nominal: f32,
        /// Elevated (post-step) residual norm level r_high.
        r_elevated: f32,
        /// Sample at which the step occurred.
        step_sample: u32,
    },
}

impl RfDisturbance {
    /// Return the DDMF class label string for provenance annotation.
    pub fn class_label(&self) -> &'static str {
        match self {
            Self::PointwiseBounded { .. }  => "PointwiseBounded",
            Self::Drift { .. }             => "Drift",
            Self::SlewRateBounded { .. }   => "SlewRateBounded",
            Self::Impulsive { .. }         => "Impulsive",
            Self::PersistentElevated { .. } => "PersistentElevated",
        }
    }

    /// Upper bound on the instantaneous disturbance magnitude at sample k.
    ///
    /// Returns `Some(bound)` for classes where a finite bound exists.
    /// Returns `None` for `Impulsive` outside its active window (no bound
    /// outside the window, and inside the window it is `amplitude`).
    pub fn magnitude_bound(&self, k: u32) -> Option<f32> {
        match self {
            Self::PointwiseBounded { d_max } => Some(*d_max),
            Self::Drift { b, s_max } => Some(b + s_max * k as f32),
            Self::SlewRateBounded { .. } => None, // bounds rate, not magnitude
            Self::Impulsive { amplitude, start_sample, duration_samples } => {
                let end = start_sample.wrapping_add(*duration_samples);
                if k >= *start_sample && k < end {
                    Some(*amplitude)
                } else {
                    Some(0.0) // outside window: negligible
                }
            }
            Self::PersistentElevated { r_elevated, step_sample, .. } => {
                if k >= *step_sample {
                    Some(*r_elevated)
                } else {
                    None
                }
            }
        }
    }

    /// Returns true if this disturbance requires envelope adaptation.
    ///
    /// `PointwiseBounded` and `SlewRateBounded` (bounded change rate) do
    /// not require the envelope to widen over time.  `Drift` and
    /// `PersistentElevated` do.
    pub fn requires_envelope_adaptation(&self) -> bool {
        matches!(
            self,
            Self::Drift { .. } | Self::PersistentElevated { .. }
        )
    }

    /// Recommended `EnvelopeMode` from the `regime` module for this disturbance.
    pub fn recommended_envelope_mode_label(&self) -> &'static str {
        match self {
            Self::PointwiseBounded { .. } => "Fixed",
            Self::Drift { .. }            => "Widening",
            Self::SlewRateBounded { .. }  => "Fixed",      // bounded-rate, no net trend
            Self::Impulsive { .. }        => "Fixed",      // brief; grammar handles it
            Self::PersistentElevated { .. } => "RegimeSwitched", // snap to new level
        }
    }
}

/// A fixed-capacity log of active disturbance hypotheses.
///
/// The DSFB observer does not create disturbances; it classifies the
/// residual trajectories it observes into candidate disturbance types.
/// This log accumulates those hypotheses for the operator advisory.
///
/// `N` = maximum number of simultaneous hypotheses (default: 4).
/// Older entries are overwritten when the log is full (oldest-first ring).
#[derive(Debug, Clone)]
pub struct DisturbanceLog<const N: usize> {
    entries: [Option<DisturbanceHypothesis>; N],
    head: usize,
    count: usize,
}

/// A single disturbance hypothesis entry.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DisturbanceHypothesis {
    /// Classified disturbance type.
    pub disturbance: RfDisturbance,
    /// Sample index at which this hypothesis was created.
    pub created_at: u32,
    /// Confidence score [0, 1] — heuristic, not probabilistic.
    ///
    /// Derived from how well the observed residual trajectory matches the
    /// predicted trajectory under this disturbance model.
    pub confidence: f32,
    /// Whether this hypothesis has been corroborated by the DSA score.
    pub dsa_corroborated: bool,
}

impl<const N: usize> DisturbanceLog<N> {
    /// Create an empty log.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            head: 0,
            count: 0,
        }
    }

    /// Record a new disturbance hypothesis.
    pub fn push(&mut self, hyp: DisturbanceHypothesis) {
        self.entries[self.head] = Some(hyp);
        self.head = (self.head + 1) % N;
        if self.count < N { self.count += 1; }
    }

    /// Iterate over all current hypotheses (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &DisturbanceHypothesis> {
        self.entries.iter().filter_map(|e| e.as_ref())
    }

    /// Number of recorded hypotheses.
    pub fn len(&self) -> usize { self.count }

    /// True if the log is empty.
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Most confident hypothesis.
    pub fn most_confident(&self) -> Option<&DisturbanceHypothesis> {
        self.iter().max_by(|a, b| {
            a.confidence.partial_cmp(&b.confidence).unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries = [None; N];
        self.head = 0;
        self.count = 0;
    }
}

impl<const N: usize> Default for DisturbanceLog<N> {
    fn default() -> Self { Self::new() }
}

/// Heuristic disturbance classifier.
///
/// Given observable quantities from the grammar/DSA/Lyapunov pipeline,
/// produces a candidate `RfDisturbance` hypothesis with a confidence score.
///
/// This is a **structural** classifier — it operates on the *shape* of the
/// residual trajectory, not on modulation features.  It is therefore
/// modulation-agnostic by construction.
///
/// ## Decision rules
///
/// The rules are derived from the DDMF disturbance model signatures:
///
/// | Observation | Likely disturbance |
/// |---|---|
/// | Large λ + sustained outward drift | Drift |
/// | Abrupt step in ‖r‖ with sustained elevation | PersistentElevated |
/// | Single spike above ρ then return | Impulsive |
/// | Slowly increasing ‖r̈‖ trend | SlewRateBounded |
/// | Stationary bounded noise | PointwiseBounded |
pub struct DisturbanceClassifier {
    /// Threshold: normalised-excess (‖r‖−ρ)/ρ above which a sample is "notably outside."
    pub excess_threshold: f32,
    /// Minimum consecutive samples above threshold to classify as PersistentElevated.
    pub persistence_min: u32,
    /// Lyapunov λ threshold below which Drift is not inferred.
    pub drift_lambda_min: f32,
    /// Running consecutive outside count.
    consecutive_outside: u32,
    /// Previous norm (for slew estimation).
    prev_norm: f32,
    /// Whether a previous norm has been observed (skips slew check on first call).
    has_prev: bool,
    /// Current sample index.
    sample_idx: u32,
}

impl DisturbanceClassifier {
    /// Construct with default RF thresholds.
    pub const fn default_rf() -> Self {
        Self {
            excess_threshold: 0.05,
            persistence_min: 8,
            drift_lambda_min: 0.005,
            consecutive_outside: 0,
            prev_norm: 0.0,
            has_prev: false,
            sample_idx: 0,
        }
    }

    /// Classify one observation.
    ///
    /// - `norm`:    current ‖r(k)‖
    /// - `rho`:     admissibility envelope radius
    /// - `lambda`:  Lyapunov exponent from `lyapunov` module (pass 0.0 if unknown)
    /// - `dsa_fired`: whether the DSA motif-fired flag is active
    ///
    /// Returns `Some(DisturbanceHypothesis)` when a classification is made;
    /// `None` during nominal operation.
    pub fn classify(
        &mut self,
        norm: f32,
        rho: f32,
        lambda: f32,
        dsa_fired: bool,
    ) -> Option<DisturbanceHypothesis> {
        let k = self.sample_idx;
        self.sample_idx = self.sample_idx.wrapping_add(1);

        let normalised_excess = if rho > 1e-30 { (norm - rho) / rho } else { 0.0 };
        let outside = normalised_excess > 0.0;
        let delta_norm = if self.has_prev { (norm - self.prev_norm).abs() } else { 0.0 };
        self.prev_norm = norm;
        self.has_prev = true;

        self.update_persistence(outside, normalised_excess);

        let disturbance = self.select_disturbance(norm, rho, lambda, normalised_excess, outside, delta_norm, k)?;
        let confidence = self.compute_confidence(&disturbance, lambda);

        Some(DisturbanceHypothesis {
            disturbance,
            created_at: k,
            confidence,
            dsa_corroborated: dsa_fired,
        })
    }

    fn update_persistence(&mut self, outside: bool, normalised_excess: f32) {
        if outside && normalised_excess > self.excess_threshold {
            self.consecutive_outside = self.consecutive_outside.saturating_add(1);
        } else {
            self.consecutive_outside = 0;
        }
    }

    fn select_disturbance(
        &self,
        norm: f32,
        rho: f32,
        lambda: f32,
        normalised_excess: f32,
        outside: bool,
        delta_norm: f32,
        k: u32,
    ) -> Option<RfDisturbance> {
        if outside && self.consecutive_outside >= self.persistence_min && normalised_excess < 0.5 {
            return Some(RfDisturbance::PersistentElevated {
                r_nominal: rho,
                r_elevated: norm,
                step_sample: k.saturating_sub(self.consecutive_outside),
            });
        }
        if lambda > self.drift_lambda_min && outside {
            return Some(RfDisturbance::Drift {
                b: normalised_excess * rho,
                s_max: lambda * rho,
            });
        }
        if outside && self.consecutive_outside == 1 && normalised_excess > 0.20 {
            return Some(RfDisturbance::Impulsive {
                amplitude: norm,
                start_sample: k,
                duration_samples: 1,
            });
        }
        if delta_norm > 0.02 * rho && !outside {
            return Some(RfDisturbance::SlewRateBounded { s_max: delta_norm });
        }
        if !outside { return None; }
        Some(RfDisturbance::PointwiseBounded { d_max: norm })
    }

    fn compute_confidence(&self, disturbance: &RfDisturbance, lambda: f32) -> f32 {
        match disturbance {
            RfDisturbance::PersistentElevated { .. } => {
                (self.consecutive_outside as f32 / self.persistence_min as f32).min(1.0)
            }
            RfDisturbance::Drift { .. } => (lambda / (self.drift_lambda_min * 5.0)).min(1.0),
            RfDisturbance::Impulsive { .. } => 0.5,
            RfDisturbance::SlewRateBounded { .. } => 0.3,
            RfDisturbance::PointwiseBounded { .. } => 0.4,
        }
    }

    /// Reset internal state.
    pub fn reset(&mut self) {
        self.consecutive_outside = 0;
        self.prev_norm = 0.0;
        self.has_prev = false;
        self.sample_idx = 0;
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_labels_canonical() {
        assert_eq!(
            RfDisturbance::PointwiseBounded { d_max: 0.1 }.class_label(),
            "PointwiseBounded"
        );
        assert_eq!(
            RfDisturbance::Drift { b: 0.0, s_max: 0.001 }.class_label(),
            "Drift"
        );
        assert_eq!(
            RfDisturbance::SlewRateBounded { s_max: 0.005 }.class_label(),
            "SlewRateBounded"
        );
        assert_eq!(
            RfDisturbance::Impulsive { amplitude: 0.5, start_sample: 10, duration_samples: 3 }.class_label(),
            "Impulsive"
        );
        assert_eq!(
            RfDisturbance::PersistentElevated { r_nominal: 0.05, r_elevated: 0.20, step_sample: 50 }.class_label(),
            "PersistentElevated"
        );
    }

    #[test]
    fn drift_magnitude_bound_grows() {
        let d = RfDisturbance::Drift { b: 0.01, s_max: 0.001 };
        let bound0 = d.magnitude_bound(0).unwrap();
        let bound100 = d.magnitude_bound(100).unwrap();
        assert!(bound100 > bound0, "drift bound must grow with k");
        assert!((bound100 - 0.11).abs() < 1e-5, "bound100={}", bound100);
    }

    #[test]
    fn impulsive_bound_outside_window_zero() {
        let d = RfDisturbance::Impulsive { amplitude: 2.0, start_sample: 10, duration_samples: 5 };
        // Outside window
        let before = d.magnitude_bound(9).unwrap();
        let after = d.magnitude_bound(15).unwrap();
        assert_eq!(before, 0.0);
        assert_eq!(after, 0.0);
        // Inside window
        let inside = d.magnitude_bound(12).unwrap();
        assert_eq!(inside, 2.0);
    }

    #[test]
    fn persistent_elevated_bound_after_step() {
        let d = RfDisturbance::PersistentElevated { r_nominal: 0.05, r_elevated: 0.20, step_sample: 20 };
        assert!(d.magnitude_bound(19).is_none(), "before step: no bound");
        let after = d.magnitude_bound(20).unwrap();
        assert!((after - 0.20).abs() < 1e-6);
    }

    #[test]
    fn envelope_adaptation_flags() {
        assert!(!RfDisturbance::PointwiseBounded { d_max: 0.1 }.requires_envelope_adaptation());
        assert!(RfDisturbance::Drift { b: 0.0, s_max: 0.001 }.requires_envelope_adaptation());
        assert!(!RfDisturbance::SlewRateBounded { s_max: 0.005 }.requires_envelope_adaptation());
        assert!(RfDisturbance::PersistentElevated {
            r_nominal: 0.05, r_elevated: 0.20, step_sample: 0
        }.requires_envelope_adaptation());
    }

    #[test]
    fn disturbance_log_push_and_most_confident() {
        let mut log = DisturbanceLog::<4>::new();
        assert!(log.is_empty());

        log.push(DisturbanceHypothesis {
            disturbance: RfDisturbance::PointwiseBounded { d_max: 0.1 },
            created_at: 0,
            confidence: 0.4,
            dsa_corroborated: false,
        });
        log.push(DisturbanceHypothesis {
            disturbance: RfDisturbance::Drift { b: 0.01, s_max: 0.001 },
            created_at: 5,
            confidence: 0.8,
            dsa_corroborated: true,
        });

        assert_eq!(log.len(), 2);
        let best = log.most_confident().unwrap();
        assert!(
            (best.confidence - 0.8).abs() < 1e-6,
            "most confident should be the Drift entry"
        );
    }

    #[test]
    fn disturbance_log_ring_behaviour() {
        let mut log = DisturbanceLog::<2>::new();
        for i in 0..5_u32 {
            log.push(DisturbanceHypothesis {
                disturbance: RfDisturbance::PointwiseBounded { d_max: i as f32 * 0.01 },
                created_at: i,
                confidence: 0.5,
                dsa_corroborated: false,
            });
        }
        // Ring size 2: only 2 entries should be visible
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn classifier_nominal_returns_none() {
        let mut clf = DisturbanceClassifier::default_rf();
        // Deep inside envelope — should return None
        for _ in 0..20 {
            let h = clf.classify(0.05, 0.10, 0.0, false);
            assert!(h.is_none(), "nominal operation should produce no hypothesis");
        }
    }

    #[test]
    fn classifier_detects_persistent() {
        let mut clf = DisturbanceClassifier::default_rf();
        // 10 samples consistently outside envelope
        let mut got_persistent = false;
        for i in 0..15 {
            if let Some(h) = clf.classify(0.12, 0.10, 0.002, false) {
                if matches!(h.disturbance, RfDisturbance::PersistentElevated { .. }) {
                    got_persistent = true;
                    let _ = i;
                    break;
                }
            }
        }
        assert!(got_persistent, "persistent elevated disturbance not detected");
    }

    #[test]
    fn classifier_detects_impulsive() {
        let mut clf = DisturbanceClassifier::default_rf();
        // Single large spike
        let h = clf.classify(0.50, 0.10, 0.0, false);
        assert!(h.is_some(), "large spike should produce a hypothesis");
        if let Some(hyp) = h {
            assert!(
                matches!(hyp.disturbance, RfDisturbance::Impulsive { .. }),
                "large spike should be Impulsive, got {}", hyp.disturbance.class_label()
            );
        }
    }
}
