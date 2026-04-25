//! CWRU Bearing Data Center adapter.
//!
//! **Provenance.** Case Western Reserve University Bearing Data Center,
//! <https://engineering.case.edu/bearingdatacenter>. A 2 HP Reliance
//! Electric motor rig with seeded faults on the drive-end and
//! fan-end bearings at four fault diameters (0.007, 0.014, 0.021,
//! 0.028 in) and four loads (0–3 HP). Vibration data captured at 12
//! kHz and 48 kHz.
//!
//! **Residual DSFB structures.** The adapter assumes the caller has
//! already extracted the **BPFI (ball-pass frequency, inner race)
//! envelope-spectrum amplitude** at each timestep — this is the
//! standard bearing-PHM scalar that threshold alarms compare against
//! a fixed cutoff. DSFB reads the *trajectory* of this amplitude:
//!
//! ```text
//! r(k) = |E_{BPFI}(k) − μ_healthy|
//! ```
//!
//! where `μ_healthy` is the mean BPFI amplitude over a healthy-window
//! calibration slice. The incumbent threshold alarm discards this
//! trajectory — it keeps only a boolean "above / below cutoff" each
//! sample. DSFB structures the residual trajectory into a grammar.

use crate::math;

/// Per-timestep envelope-spectrum amplitude sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Envelope-spectrum amplitude at the BPFI harmonics at time k.
    ///
    /// Caller computes this via standard bearing-envelope-analysis
    /// signal processing (Hilbert-envelope demodulation → band-pass
    /// around BPFI harmonics → RMS). This adapter does not care how
    /// the amplitude was computed — only that it is a scalar
    /// representative of bearing-inner-race fault energy.
    pub bpfi_amplitude: f64,
}

/// Calibrated BPFI baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    /// Mean BPFI amplitude over the healthy calibration window.
    pub mu_healthy: f64,
}

impl Baseline {
    /// Calibrate the baseline from a healthy-window slice.
    ///
    /// Returns `None` if the slice has no finite samples.
    #[must_use]
    pub fn from_healthy(healthy: &[f64]) -> Option<Self> {
        debug_assert!(healthy.len() <= 1_000_000, "healthy window unreasonably large");
        let mu = math::finite_mean(healthy)?;
        debug_assert!(mu.is_finite(), "finite_mean returns Some only for finite values");
        Some(Self { mu_healthy: mu })
    }

    /// `|amplitude − μ_healthy|` for one sample.
    #[inline]
    #[must_use]
    pub fn residual(&self, sample: Sample) -> f64 {
        debug_assert!(self.mu_healthy.is_finite(), "calibrated baseline must be finite");
        let r = math::abs_f64(sample.bpfi_amplitude - self.mu_healthy);
        debug_assert!(r >= 0.0 || !r.is_finite(), "residual is non-negative or non-finite");
        r
    }
}

/// Stream a per-sample amplitude slice into a residual buffer given a
/// baseline. Returns the number of residuals written.
pub fn residual_stream(samples: &[Sample], baseline: Baseline, out: &mut [f64]) -> usize {
    debug_assert!(baseline.mu_healthy.is_finite(), "baseline must be calibrated");
    let n = samples.len().min(out.len());
    debug_assert!(n <= out.len() && n <= samples.len(), "n respects both bounds");
    let mut i = 0_usize;
    while i < n {
        out[i] = baseline.residual(samples[i]);
        i += 1;
    }
    debug_assert_eq!(i, n, "loop must run exactly n iterations");
    n
}

/// Healthy-window calibration slice for smoke-test reproductions.
pub const HEALTHY_FIXTURE: [f64; 6] = [0.10, 0.11, 0.09, 0.10, 0.10, 0.11];

/// Faulted-sample trajectory for smoke-test reproductions.
pub const FAULTED_FIXTURE: [Sample; 5] = [
    Sample { bpfi_amplitude: 0.10 },
    Sample { bpfi_amplitude: 0.12 },
    Sample { bpfi_amplitude: 0.20 },
    Sample { bpfi_amplitude: 0.35 },
    Sample { bpfi_amplitude: 0.15 },
];

/// Calibrate from [`HEALTHY_FIXTURE`] and stream [`FAULTED_FIXTURE`]
/// residuals into `out`. Returns the number written. Used by
/// `paper-lock --fixture`.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    let Some(baseline) = Baseline::from_healthy(&HEALTHY_FIXTURE) else {
        debug_assert!(false, "HEALTHY_FIXTURE is non-empty + finite — calibration must succeed");
        return 0;
    };
    residual_stream(&FAULTED_FIXTURE, baseline, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_from_empty_is_none() {
        assert!(Baseline::from_healthy(&[]).is_none());
    }

    #[test]
    fn healthy_window_gives_near_zero_residual_for_nominal_sample() {
        let baseline = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let r = baseline.residual(Sample { bpfi_amplitude: 0.10 });
        assert!(r < 0.02);
    }

    #[test]
    fn faulted_sample_has_elevated_residual() {
        let baseline = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let r = baseline.residual(Sample { bpfi_amplitude: 0.35 });
        assert!(r > 0.20);
    }

    #[test]
    fn stream_trajectory_peaks_at_fault_sample() {
        let baseline = Baseline::from_healthy(&HEALTHY_FIXTURE).expect("finite");
        let mut out = [0.0_f64; 5];
        let n = residual_stream(&FAULTED_FIXTURE, baseline, &mut out);
        assert_eq!(n, 5);
        let peak = out.iter().copied().fold(0.0_f64, f64::max);
        assert!((peak - out[3]).abs() < 1e-12, "peak should be index 3");
    }
}
