//! `SpectralGrammarTokenV1` (Wave-7 semiotics) — frequency-domain grammar tokens, so a rhythmic fault
//! (a limit cycle, a hunting valve, a resonance) is named by its **spectral shape**, not just its amplitude.
//!
//! The time-domain DSFB grammar (drift / slew / envelope) is blind to *periodic* structure: a steady
//! oscillation can sit inside the amplitude envelope yet be a clear fault. This object runs the existing FFT
//! power spectrum ([`crate::fft::power_spectrum`], DC-removed) over a window and emits a token from the
//! spectral shape:
//!   * `LimitCycleResonance` — one bin holds a dominant share of the fluctuation power (a tonal limit cycle).
//!   * `Broadband` — power spread across bins (turbulence / cavitation-like) with no single tone.
//!   * `NoSpectralStructure` — the fluctuation power is below the floor (quiet / flat).
//!
//! Bounded (non-claims): a spectral token names the *frequency shape* of the residual, not its cause — a
//! `LimitCycleResonance` is a tonal oscillation to investigate, not a diagnosis of which loop or mechanism.
//! Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The spectral-shape token assigned to a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpectralToken {
    LimitCycleResonance,
    Broadband,
    NoSpectralStructure,
}

impl SpectralToken {
    pub fn tag(self) -> &'static str {
        match self {
            SpectralToken::LimitCycleResonance => "limit_cycle_resonance",
            SpectralToken::Broadband => "broadband",
            SpectralToken::NoSpectralStructure => "no_spectral_structure",
        }
    }
}

/// A hash-sealed spectral grammar token (schema v1) for one residual window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralGrammarTokenV1 {
    pub signal_len: usize,
    /// The dominant (highest-power) frequency bin index in the DC-removed half-spectrum.
    pub dominant_bin: usize,
    /// `dominant_bin power / total fluctuation power` in `[0, 1]` (0 when there is no fluctuation power).
    pub dominant_power_fraction: f64,
    /// Total fluctuation (AC) power, after DC removal.
    pub total_power: f64,
    /// Fraction at/above which a single bin is called a `LimitCycleResonance` (disclosed threshold).
    pub tonal_threshold: f64,
    pub token: String,
    pub token_hash: String,
}

impl SpectralGrammarTokenV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"spectral_grammar_token_v1");
        h.u64("signal_len", self.signal_len as u64);
        h.u64("dominant_bin", self.dominant_bin as u64);
        h.f64q("dominant_power_fraction", self.dominant_power_fraction);
        h.f64q("total_power", self.total_power);
        h.f64q("tonal_threshold", self.tonal_threshold);
        h.field("token", self.token.as_bytes());
        h.finalize_hex()
    }

    /// Classify a window's spectral shape. `power_floor` is the minimum total fluctuation power below which
    /// the window is `NoSpectralStructure`; `tonal_threshold` is the dominant-bin fraction at/above which it
    /// is a `LimitCycleResonance` (else `Broadband`).
    pub fn build(signal: &[f64], power_floor: f64, tonal_threshold: f64) -> Self {
        let ps = crate::fft::power_spectrum(signal);
        let total_power: f64 = ps.iter().sum();
        let (mut dominant_bin, mut peak) = (0usize, 0.0f64);
        for (k, &p) in ps.iter().enumerate() {
            if p > peak {
                peak = p;
                dominant_bin = k;
            }
        }
        let dominant_power_fraction = if total_power > 0.0 {
            peak / total_power
        } else {
            0.0
        };
        let token = if total_power < power_floor {
            SpectralToken::NoSpectralStructure
        } else if dominant_power_fraction >= tonal_threshold {
            SpectralToken::LimitCycleResonance
        } else {
            SpectralToken::Broadband
        };
        let mut t = SpectralGrammarTokenV1 {
            signal_len: signal.len(),
            dominant_bin,
            dominant_power_fraction,
            total_power,
            tonal_threshold,
            token: token.tag().into(),
            token_hash: String::new(),
        };
        t.token_hash = t.seal();
        t
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.token_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    /// A pure sinusoid of `cycles` periods over `n` samples.
    fn sine(n: usize, cycles: f64) -> Vec<f64> {
        (0..n)
            .map(|i| (2.0 * PI * cycles * i as f64 / n as f64).sin())
            .collect()
    }

    #[test]
    fn pure_tone_is_a_limit_cycle_resonance() {
        // A single sinusoid concentrates power into one bin → LimitCycleResonance.
        let sig = sine(64, 8.0);
        let t = SpectralGrammarTokenV1::build(&sig, 1e-6, 0.5);
        assert_eq!(t.token, "limit_cycle_resonance");
        assert!(t.dominant_power_fraction >= 0.5);
        assert!(t.token_hash.len() == 64 && t.verify());
    }

    #[test]
    fn flat_signal_has_no_spectral_structure() {
        let flat = vec![3.0; 64]; // DC removed → ~0 fluctuation power
        let t = SpectralGrammarTokenV1::build(&flat, 1e-6, 0.5);
        assert_eq!(t.token, "no_spectral_structure");
        assert!(t.verify());
    }

    #[test]
    fn mixed_many_tones_read_as_broadband() {
        // Sum of many incommensurate-ish tones spreads power → no single bin dominates → Broadband.
        let n = 128;
        let sig: Vec<f64> = (0..n)
            .map(|i| {
                let x = i as f64;
                (0..12)
                    .map(|k| (2.0 * PI * (3 + 7 * k) as f64 * x / n as f64).sin())
                    .sum::<f64>()
            })
            .collect();
        let t = SpectralGrammarTokenV1::build(&sig, 1e-6, 0.5);
        assert_eq!(t.token, "broadband");
        assert!(t.dominant_power_fraction < 0.5);
        assert!(t.verify());
        let mut tampered = t.clone();
        tampered.token = "limit_cycle_resonance".into();
        assert!(!tampered.verify());
    }
}
