//! FEMTO-ST PRONOSTIA accelerated bearing degradation (IEEE PHM 2012 Challenge).
//!
//! **Provenance.** Nectoux, Gouriveau, Medjaher, Ramasso, Chebel-Morello,
//! Zerhouni, Varnier, *"PRONOSTIA: An experimental platform for bearings
//! accelerated degradation tests"*, IEEE PHM 2012 Conference. Seventeen
//! bearings run to failure under accelerated load, 25.6 kHz vibration
//! and 10 Hz temperature sampling.
//!
//! **Residual DSFB structures.** Same form as [`super::ims`]: the
//! caller supplies a per-snapshot vibration-derived health index (RMS,
//! kurtosis, crest-factor, spectral-kurtosis) and the adapter emits
//! the `|HI(k) − HI_calib|` residual trajectory. The PHM 2012 solvers
//! collapsed this trajectory to an RUL estimate; DSFB recovers the
//! trajectory's structure.

/// Per-snapshot vibration-derived health-index sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Caller-computed vibration health index.
    pub vib_hi: f64,
}

/// Calibrated HI baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    /// Calibrated nominal HI value.
    pub hi_calib: f64,
}

impl Baseline {
    /// Calibrate from a healthy-window slice.
    #[must_use]
    pub fn from_healthy(healthy: &[f64]) -> Option<Self> {
        debug_assert!(healthy.len() <= 1_000_000, "calibration window unreasonably large");
        let mu = crate::math::finite_mean(healthy)?;
        debug_assert!(mu.is_finite(), "finite_mean returns Some only for finite values");
        Some(Self { hi_calib: mu })
    }

    /// `|HI(k) − HI_calib|` for one sample.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self, sample: Sample) -> f64 {
        debug_assert!(self.hi_calib.is_finite(), "calibrated HI must be finite");
        let r = crate::math::abs_f64(sample.vib_hi - self.hi_calib);
        debug_assert!(r >= 0.0 || !r.is_finite(), "norm is non-negative or non-finite");
        r
    }
}

/// Stream samples into a residual-norm buffer.
pub fn residual_stream(samples: &[Sample], baseline: Baseline, out: &mut [f64]) -> usize {
    debug_assert!(baseline.hi_calib.is_finite(), "baseline must be calibrated");
    let n = samples.len().min(out.len());
    debug_assert!(n <= out.len() && n <= samples.len(), "n bounded by both buffers");
    let mut i = 0_usize;
    while i < n {
        out[i] = baseline.residual_norm(samples[i]);
        i += 1;
    }
    debug_assert_eq!(i, n, "loop must run exactly n iterations");
    n
}

/// Healthy-window calibration slice for smoke-test reproductions.
pub const HEALTHY_FIXTURE: [f64; 5] = [0.02, 0.02, 0.03, 0.02, 0.02];

/// Accelerated-aging trajectory for smoke-test reproductions.
pub const ACCELERATED_FIXTURE: [Sample; 5] = [
    Sample { vib_hi: 0.02 },
    Sample { vib_hi: 0.05 },
    Sample { vib_hi: 0.12 },
    Sample { vib_hi: 0.30 },
    Sample { vib_hi: 0.80 },
];

/// Calibrate from [`HEALTHY_FIXTURE`] and stream
/// [`ACCELERATED_FIXTURE`] residuals into `out`. Returns the number
/// written.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    let Some(baseline) = Baseline::from_healthy(&HEALTHY_FIXTURE) else {
        debug_assert!(false, "HEALTHY_FIXTURE is non-empty + finite — calibration must succeed");
        return 0;
    };
    residual_stream(&ACCELERATED_FIXTURE, baseline, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residual_increases_through_accelerated_aging() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let mut out = [0.0_f64; 5];
        let n = residual_stream(&ACCELERATED_FIXTURE, b, &mut out);
        assert_eq!(n, 5);
        for i in 1..n {
            assert!(out[i] >= out[i - 1], "accelerated aging → monotone residual, got {out:?}");
        }
    }

    #[test]
    fn nominal_healthy_calibration_matches_baseline_mean() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        // Calibration mean should be ≈ 0.022.
        assert!((b.hi_calib - 0.022).abs() < 1e-3);
    }

    #[test]
    fn residual_norm_is_non_negative() {
        let b = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        for h in [-1.0, 0.0, 0.5, 1.0] {
            let r = b.residual_norm(Sample { vib_hi: h });
            assert!(r >= 0.0);
        }
    }
}
