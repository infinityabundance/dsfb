//! Industry standards integration: VITA 49.2, SigMF, SOSA/MORA.
//!
//! ## VITA 49.2 (VITA Radio Transport — VRT)
//!
//! VITA 49.2 defines the packet format for digitized IF/RF data transport,
//! including sub-nanosecond timestamping and hardware context packets
//! (gain, temperature, frequency, sample rate). DSFB consumes VRT context
//! to enrich its platform context with hardware-level metadata.
//!
//! The `Vrt49Context` struct captures the fields DSFB needs from VRT
//! context packets — gain, temperature, center frequency, and timestamp.
//! This enables the heuristics bank to distinguish gain-drift from thermal
//! drift from frequency offset drift at the hardware metadata level.
//!
//! ## SigMF (Signal Metadata Format)
//!
//! SigMF provides a standardized JSON schema for annotating IQ recordings.
//! DSFB episodes are exported as SigMF annotations, enabling instant
//! visualization in tools like IQEngine, inspectrum, and Universal Radio Hacker.
//!
//! Each Review/Escalate episode maps to a SigMF annotation with:
//! - `core:sample_start` / `core:sample_count`
//! - `core:label` = grammar state (e.g., "Boundary[SustainedOutwardDrift]")
//! - `dsfb:motif`, `dsfb:dsa_score`, `dsfb:lyapunov_lambda`
//!
//! ## SOSA / MORA Alignment
//!
//! The Sensor Open Systems Architecture (SOSA™) and Modular Open RF
//! Architecture (MORA) mandate software-defined, vendor-neutral RF
//! processing components. DSFB is positioned as a MORA-compliant
//! Software Resource:
//! - Stateless observer with well-defined input/output interfaces
//! - No vendor-specific hardware dependencies in the core engine
//! - Deployable as a SOSA-aligned processing element alongside
//!   existing signal processing chains
//!
//! ## Design
//!
//! - Core structs: `no_std`, `no_alloc`, zero `unsafe`
//! - SigMF export: requires `serde` feature (JSON serialization)
//! - VRT context consumption: `no_std` compatible (struct population only)

// ── VITA 49.2 VRT Context ──────────────────────────────────────────────────

/// Hardware context from a VITA 49.2 (VRT) context packet.
///
/// Populated by the integration layer from VRT context extension packets.
/// The DSFB engine reads these fields but never writes VRT packets.
///
/// ## VRT Field Mapping
///
/// | VRT Field           | DSFB Usage                                      |
/// |---------------------|-------------------------------------------------|
/// | Reference Level     | Maps to admissibility envelope scaling           |
/// | Gain                | Distinguishes AGC drift from signal-level drift  |
/// | Temperature         | Correlates thermal drift with PA thermal motif   |
/// | RF Reference Freq   | Detects LO offset / frequency drift              |
/// | Timestamp (picosec) | Sub-nanosecond event timestamping                |
/// | Bandwidth           | Contextualizes spectral mask width               |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vrt49Context {
    /// Receiver gain in dB (from VRT Gain field, CIF 0 word).
    pub gain_db: f32,
    /// Device temperature in °C (from VRT Temperature field).
    /// `f32::NAN` if not available.
    pub temperature_c: f32,
    /// RF reference frequency in Hz (from VRT RF Reference Frequency field).
    pub rf_ref_freq_hz: f64,
    /// Integer-seconds timestamp (from VRT Integer-Seconds Timestamp).
    pub timestamp_int_sec: u32,
    /// Fractional-seconds timestamp in picoseconds (from VRT Fractional Timestamp).
    pub timestamp_frac_ps: u64,
    /// Bandwidth in Hz (from VRT Bandwidth field).
    pub bandwidth_hz: f32,
    /// Sample rate in samples/sec (from VRT Sample Rate field).
    pub sample_rate_sps: f64,
}

impl Vrt49Context {
    /// Create a context with unknown/default values.
    pub const fn unknown() -> Self {
        Self {
            gain_db: 0.0,
            temperature_c: f32::NAN,
            rf_ref_freq_hz: 0.0,
            timestamp_int_sec: 0,
            timestamp_frac_ps: 0,
            bandwidth_hz: 0.0,
            sample_rate_sps: 0.0,
        }
    }

    /// Returns true if a valid temperature reading is available.
    #[inline]
    pub fn has_temperature(&self) -> bool {
        !self.temperature_c.is_nan()
    }

    /// Returns true if a valid RF reference frequency is set.
    #[inline]
    pub fn has_rf_freq(&self) -> bool {
        self.rf_ref_freq_hz > 0.0
    }
}

impl Default for Vrt49Context {
    fn default() -> Self { Self::unknown() }
}

// ── SigMF Annotation ──────────────────────────────────────────────────────

/// A DSFB episode exported as a SigMF-compatible annotation.
///
/// Conforms to the SigMF `core` namespace plus DSFB extension fields.
/// Serializable to JSON via `serde` for direct insertion into a
/// `.sigmf-meta` file's `annotations` array.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SigmfAnnotation {
    /// `core:sample_start` — first sample index of the episode.
    #[cfg_attr(feature = "serde", serde(rename = "core:sample_start"))]
    pub sample_start: u64,
    /// `core:sample_count` — duration of the episode in samples.
    #[cfg_attr(feature = "serde", serde(rename = "core:sample_count"))]
    pub sample_count: u64,
    /// `core:label` — grammar state label.
    #[cfg_attr(feature = "serde", serde(rename = "core:label"))]
    pub label: &'static str,
    /// `core:comment` — human-readable episode summary.
    #[cfg_attr(feature = "serde", serde(rename = "core:comment"))]
    pub comment: &'static str,
    /// `dsfb:motif_class` — named temporal motif.
    #[cfg_attr(feature = "serde", serde(rename = "dsfb:motif_class"))]
    pub motif_class: &'static str,
    /// `dsfb:dsa_score` — Deterministic Structural Accumulator score.
    #[cfg_attr(feature = "serde", serde(rename = "dsfb:dsa_score"))]
    pub dsa_score: f32,
    /// `dsfb:lyapunov_lambda` — finite-time Lyapunov exponent.
    #[cfg_attr(feature = "serde", serde(rename = "dsfb:lyapunov_lambda"))]
    pub lyapunov_lambda: f32,
    /// `dsfb:policy_decision` — Silent/Watch/Review/Escalate.
    #[cfg_attr(feature = "serde", serde(rename = "dsfb:policy_decision"))]
    pub policy_decision: &'static str,
}

// ── MIL-STD-461G Spectral Mask Envelope ────────────────────────────────────

/// A spectral emission mask point for MIL-STD-461G RE102/CE102 or
/// ITU-R SM.1048-5 §4.3 mask-deviation tracking.
///
/// The DSFB spectral mask deviation residual uses these points as
/// the outer admissibility boundary. Structural monitoring tracks
/// whether measured PSD is drifting toward the mask boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectralMaskPoint {
    /// Frequency in Hz.
    pub freq_hz: f64,
    /// Maximum allowable power spectral density in dBm/MHz (or dBμV/m for RE102).
    pub limit_db: f32,
}

/// A piecewise-linear spectral emission mask.
///
/// Fixed-capacity array of mask points, sorted by frequency.
/// Supports MIL-STD-461G RE102 (2 MHz – 18 GHz), CE102 (10 kHz – 10 MHz),
/// 3GPP TS 36.141 §6.3 ACLR, and ITU-R SM.1048-5 masks.
pub struct SpectralMask<const N: usize> {
    /// Mask points sorted by frequency.
    points: [SpectralMaskPoint; N],
    /// Number of valid points.
    count: usize,
    /// Mask identifier (e.g., "RE102_ground", "CE102", "ACLR_E-UTRA").
    pub name: &'static str,
}

impl<const N: usize> SpectralMask<N> {
    /// Create an empty mask.
    pub const fn empty(name: &'static str) -> Self {
        Self {
            points: [SpectralMaskPoint { freq_hz: 0.0, limit_db: 0.0 }; N],
            count: 0,
            name,
        }
    }

    /// Add a point. Returns false if mask is full.
    pub fn add_point(&mut self, freq_hz: f64, limit_db: f32) -> bool {
        if self.count >= N { return false; }
        self.points[self.count] = SpectralMaskPoint { freq_hz, limit_db };
        self.count += 1;
        true
    }

    /// Interpolate the mask limit at a given frequency.
    ///
    /// Returns `None` if the frequency is outside the mask range.
    /// Uses linear interpolation between adjacent points.
    pub fn limit_at(&self, freq_hz: f64) -> Option<f32> {
        if self.count < 2 { return None; }
        let pts = &self.points[..self.count];

        if freq_hz < pts[0].freq_hz || freq_hz > pts[self.count - 1].freq_hz {
            return None;
        }

        for i in 0..self.count - 1 {
            if freq_hz >= pts[i].freq_hz && freq_hz <= pts[i + 1].freq_hz {
                let frac = ((freq_hz - pts[i].freq_hz) / (pts[i + 1].freq_hz - pts[i].freq_hz)) as f32;
                return Some(pts[i].limit_db + frac * (pts[i + 1].limit_db - pts[i].limit_db));
            }
        }
        None
    }

    /// Number of mask points.
    #[inline]
    pub fn len(&self) -> usize { self.count }

    /// Whether the mask is empty.
    #[inline]
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Compute the mask deviation residual: measured_db − limit_db.
    ///
    /// Positive values indicate the measurement exceeds the mask (violation).
    /// Negative values indicate margin remains.
    #[inline]
    pub fn deviation(&self, freq_hz: f64, measured_db: f32) -> Option<f32> {
        self.limit_at(freq_hz).map(|limit| measured_db - limit)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vrt_context_default() {
        let ctx = Vrt49Context::unknown();
        assert!(!ctx.has_temperature());
        assert!(!ctx.has_rf_freq());
    }

    #[test]
    fn vrt_context_with_values() {
        let ctx = Vrt49Context {
            gain_db: 30.0,
            temperature_c: 45.5,
            rf_ref_freq_hz: 2.4e9,
            timestamp_int_sec: 1700000000,
            timestamp_frac_ps: 500_000_000_000,
            bandwidth_hz: 20e6,
            sample_rate_sps: 61.44e6,
        };
        assert!(ctx.has_temperature());
        assert!(ctx.has_rf_freq());
    }

    #[test]
    fn spectral_mask_interpolation() {
        let mut mask = SpectralMask::<4>::empty("test_mask");
        mask.add_point(100e6, -40.0);
        mask.add_point(200e6, -30.0);
        mask.add_point(300e6, -50.0);

        let limit = mask.limit_at(150e6).unwrap();
        assert!((limit - (-35.0)).abs() < 0.1, "midpoint interpolation: {}", limit);

        assert!(mask.limit_at(50e6).is_none(), "below range");
        assert!(mask.limit_at(400e6).is_none(), "above range");
    }

    #[test]
    fn spectral_mask_deviation() {
        let mut mask = SpectralMask::<4>::empty("test");
        mask.add_point(100e6, -30.0);
        mask.add_point(200e6, -30.0); // flat mask at -30 dBm

        // Measurement below limit
        let dev = mask.deviation(150e6, -40.0).unwrap();
        assert!(dev < 0.0, "below mask must be negative deviation: {}", dev);

        // Measurement above limit
        let dev2 = mask.deviation(150e6, -20.0).unwrap();
        assert!(dev2 > 0.0, "above mask must be positive deviation: {}", dev2);
    }

    #[test]
    fn mask_capacity_enforced() {
        let mut mask = SpectralMask::<2>::empty("tiny");
        assert!(mask.add_point(100.0, -10.0));
        assert!(mask.add_point(200.0, -20.0));
        assert!(!mask.add_point(300.0, -30.0), "must reject when full");
        assert_eq!(mask.len(), 2);
    }

    #[test]
    fn sigmf_annotation_fields() {
        let ann = SigmfAnnotation {
            sample_start: 1000,
            sample_count: 500,
            label: "Boundary[SustainedOutwardDrift]",
            comment: "PA thermal drift detected",
            motif_class: "PreFailureSlowDrift",
            dsa_score: 2.5,
            lyapunov_lambda: 0.015,
            policy_decision: "Review",
        };
        assert_eq!(ann.sample_start, 1000);
        assert_eq!(ann.label, "Boundary[SustainedOutwardDrift]");
    }
}
