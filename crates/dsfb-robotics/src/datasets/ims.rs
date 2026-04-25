//! IMS Run-to-Failure bearing adapter.
//!
//! **Provenance.** NASA Prognostics Data Repository, *"Bearing Data
//! Set"* provided by the Center for Intelligent Maintenance Systems
//! (IMS), University of Cincinnati (Lee, Qiu, Yu, Lin et al. 2007).
//! Three test-to-failure experiments, four bearings per shaft,
//! 20 kHz sampling, 10-minute snapshots over ≈35 days total run time.
//!
//! **Residual DSFB structures.** The adapter consumes a per-snapshot
//! **health-index (HI)** trajectory — typically a vibration RMS, kurtosis,
//! or PCA-derived scalar that a Rainflow / RUL estimator collapses to
//! a single remaining-useful-life number and discards. DSFB reads the
//! residual between the HI trajectory and the nominal HI calibrated
//! from an early-life healthy window:
//!
//! ```text
//! r(k) = HI(k) − HI_nominal
//! ```
//!
//! The sign is preserved (not absolute-valued) so DSFB can distinguish
//! monotonic degradation (positive residual trajectory) from transient
//! operational excursions (bi-directional residual). The grammar FSM
//! treats both symmetrically via the `norm = |r|` view.

/// Per-snapshot health-index sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Caller-computed bearing health index. The exact HI formula
    /// (RMS, kurtosis, PCA-K1, MF-DFA, etc.) is up to the caller —
    /// DSFB treats this as an opaque scalar residual source.
    pub health_index: f64,
}

/// Calibrated HI baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    /// Nominal HI value calibrated from an early-life healthy
    /// snapshot window.
    pub hi_nominal: f64,
}

impl Baseline {
    /// Calibrate from a healthy snapshot window.
    #[must_use]
    pub fn from_healthy(healthy: &[f64]) -> Option<Self> {
        debug_assert!(healthy.len() <= 1_000_000, "calibration window unreasonably large");
        let mu = crate::math::finite_mean(healthy)?;
        debug_assert!(mu.is_finite(), "finite_mean returns Some only for finite values");
        Some(Self { hi_nominal: mu })
    }

    /// Signed residual `HI(k) − HI_nominal` for one sample.
    #[inline]
    #[must_use]
    pub fn residual(&self, sample: Sample) -> f64 {
        debug_assert!(self.hi_nominal.is_finite(), "calibrated nominal must be finite");
        sample.health_index - self.hi_nominal
    }

    /// Magnitude residual `|HI(k) − HI_nominal|` for one sample. This
    /// is the form the engine's `‖r‖` field consumes.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self, sample: Sample) -> f64 {
        debug_assert!(self.hi_nominal.is_finite(), "calibrated nominal must be finite");
        let r = crate::math::abs_f64(self.residual(sample));
        debug_assert!(r >= 0.0 || !r.is_finite(), "norm is non-negative or non-finite");
        r
    }
}

/// Stream a per-snapshot HI slice into a residual-norm buffer.
pub fn residual_stream(samples: &[Sample], baseline: Baseline, out: &mut [f64]) -> usize {
    debug_assert!(baseline.hi_nominal.is_finite(), "baseline must be calibrated");
    let n = samples.len().min(out.len());
    debug_assert!(n <= out.len() && n <= samples.len(), "n respects both bounds");
    let mut i = 0_usize;
    while i < n {
        out[i] = baseline.residual_norm(samples[i]);
        i += 1;
    }
    debug_assert_eq!(i, n, "loop must run exactly n iterations");
    n
}

/// Healthy-window calibration slice for smoke-test reproductions.
pub const HEALTHY_FIXTURE: [f64; 5] = [0.05, 0.06, 0.05, 0.05, 0.06];

/// Run-to-failure trajectory for smoke-test reproductions.
pub const RUN_TO_FAILURE_FIXTURE: [Sample; 6] = [
    Sample { health_index: 0.05 },
    Sample { health_index: 0.06 },
    Sample { health_index: 0.08 },
    Sample { health_index: 0.12 },
    Sample { health_index: 0.20 },
    Sample { health_index: 0.35 },
];

/// Calibrate from [`HEALTHY_FIXTURE`] and stream
/// [`RUN_TO_FAILURE_FIXTURE`] residuals into `out`. Returns the
/// number written.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    let Some(baseline) = Baseline::from_healthy(&HEALTHY_FIXTURE) else {
        debug_assert!(false, "HEALTHY_FIXTURE is non-empty + finite — calibration must succeed");
        return 0;
    };
    residual_stream(&RUN_TO_FAILURE_FIXTURE, baseline, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_residual_preserves_direction() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let above = Sample { health_index: 0.10 };
        let below = Sample { health_index: 0.01 };
        assert!(b.residual(above) > 0.0);
        assert!(b.residual(below) < 0.0);
    }

    #[test]
    fn magnitude_residual_is_non_negative() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        for h in [0.01, 0.05, 0.10, 0.35] {
            assert!(b.residual_norm(Sample { health_index: h }) >= 0.0);
        }
    }

    #[test]
    fn run_to_failure_residuals_are_monotone() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let mut out = [0.0_f64; 6];
        let n = residual_stream(&RUN_TO_FAILURE_FIXTURE, b, &mut out);
        assert_eq!(n, 6);
        // Fixture is monotonically increasing — residual-norm must also be.
        for i in 1..n {
            assert!(out[i] >= out[i - 1], "expected non-decreasing, got {out:?}");
        }
    }
}
