//! Heuristics bank H: fixed-capacity typed RF motif library.
//!
//! The heuristics bank is what makes DSFB diagnostically superior to a
//! Luenberger observer for operator-facing applications. A Luenberger
//! observer has no memory of what structural pattern it is observing —
//! it sees only "error." The heuristics bank accumulates typed,
//! provenance-aware motif entries that distinguish classes of structural
//! behavior (paper §V-F, Table I).
//!
//! ## Design
//!
//! Fixed-capacity array `[MotifEntry; M]` — no heap, no alloc.
//! Generic parameter M = max entries (default 32 in the engine).
//! Linear scan for lookup: O(M) per observe() call, M ≤ 32 → negligible.
//!
//! ## Non-Attribution Policy (paper §V-F)
//!
//! Motif entries carry *candidate mechanism hypotheses*, not attributions.
//! No physical mechanism is attributed from public datasets (RadioML, ORACLE).
//! The `provenance` field records whether the entry is framework-designed,
//! empirically observed, or field-validated.

use crate::grammar::GrammarState;
use crate::syntax::MotifClass;

/// Provenance of a heuristics bank entry.
///
/// Records the epistemic status of a motif entry — how it was derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Provenance {
    /// Derived from framework design principles.
    FrameworkDesign,
    /// Observed in public dataset (RadioML or ORACLE). No physical attribution.
    PublicDataObserved,
    /// Observed in deployment; field-validated by domain expert.
    FieldValidated,
}

/// Semantic disposition returned by heuristics bank lookup.
///
/// This is what the operator sees as the final semantic label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SemanticDisposition {
    /// Structurally confirmed pre-transition cluster.
    PreTransitionCluster,
    /// Corroborating drift — secondary support for escalation.
    CorroboratingDrift,
    /// Transient noise spike — no escalation warranted.
    TransientNoise,
    /// Recurrent structural pattern — watch and monitor.
    RecurrentPattern,
    /// Abrupt onset event — immediate escalation warranted.
    AbruptOnsetEvent,
    /// Spectral mask approach — proactive monitoring warranted.
    MaskApproach,
    /// Phase noise degradation — link quality monitoring.
    PhaseNoiseDegradation,
    /// Endoductive: no named interpretation. Returns full σ(k) to operator.
    Unknown,
    /// LNA gain instability: progressive gain collapse detected.
    /// Signature: linear norm ramp below 30% ρ, near-zero r̈.
    /// Recommended action: flag node telemetry for operator review.
    /// Do NOT abort mission or reset radio — this is read-only observer output.
    LnaGainInstability,
    /// LO oscillator instability precursor detected.
    /// Signature: `RecurrentBoundaryGrazing` with oscillatory slew.
    /// Consistent with phase-noise excursion (OcxoWarmup or FreeRunXtal class).
    /// Recommended action: tag geolocation/timing data with advisory.
    LoInstabilityPrecursor,
}

/// A single entry in the heuristics bank.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MotifEntry {
    /// The motif class this entry matches.
    pub motif_class: MotifClass,
    /// Minimum grammar severity required to activate this entry.
    /// 0 = any, 1 = Boundary or above, 2 = Violation only.
    pub min_severity: u8,
    /// Semantic disposition returned when this entry matches.
    pub disposition: SemanticDisposition,
    /// Provenance of this entry.
    pub provenance: Provenance,
    /// Human-readable description (fixed-length str for no_std).
    pub description: &'static str,
}

impl MotifEntry {
    /// Returns true if this entry matches the given motif class and grammar severity.
    #[inline]
    pub fn matches(&self, motif: MotifClass, grammar: GrammarState) -> bool {
        self.motif_class == motif && grammar.severity() >= self.min_severity
    }
}

/// Fixed-capacity heuristics bank.
///
/// M = maximum number of entries. Populated at construction with the
/// framework-designed RF motif library. Additional entries can be
/// registered at runtime (up to M capacity) for endoductive learning.
pub struct HeuristicsBank<const M: usize> {
    entries: [Option<MotifEntry>; M],
    count: usize,
}

impl<const M: usize> HeuristicsBank<M> {
    /// Create an empty heuristics bank.
    pub const fn empty() -> Self {
        Self {
            entries: [None; M],
            count: 0,
        }
    }

    /// Create a heuristics bank pre-populated with the RF framework motif library.
    ///
    /// Populates with the 7 canonical RF motifs from paper §V-F.
    /// These are framework-designed entries (Provenance::FrameworkDesign).
    pub fn default_rf() -> Self {
        const MOTIFS: [MotifEntry; 9] = [
            MotifEntry { motif_class: MotifClass::PreFailureSlowDrift,       min_severity: 1, disposition: SemanticDisposition::PreTransitionCluster,    provenance: Provenance::FrameworkDesign, description: "Persistent outward drift toward boundary" },
            MotifEntry { motif_class: MotifClass::TransientExcursion,        min_severity: 2, disposition: SemanticDisposition::TransientNoise,          provenance: Provenance::FrameworkDesign, description: "Brief violation with rapid recovery" },
            MotifEntry { motif_class: MotifClass::RecurrentBoundaryApproach, min_severity: 1, disposition: SemanticDisposition::RecurrentPattern,        provenance: Provenance::FrameworkDesign, description: "Repeated near-boundary excursions" },
            MotifEntry { motif_class: MotifClass::AbruptOnset,               min_severity: 2, disposition: SemanticDisposition::AbruptOnsetEvent,        provenance: Provenance::FrameworkDesign, description: "Abrupt large slew" },
            MotifEntry { motif_class: MotifClass::SpectralMaskApproach,      min_severity: 1, disposition: SemanticDisposition::MaskApproach,            provenance: Provenance::FrameworkDesign, description: "Monotone outward drift toward mask edge" },
            MotifEntry { motif_class: MotifClass::PhaseNoiseExcursion,       min_severity: 1, disposition: SemanticDisposition::PhaseNoiseDegradation,   provenance: Provenance::FrameworkDesign, description: "Oscillatory slew with growing amplitude" },
            MotifEntry { motif_class: MotifClass::FreqHopTransition,         min_severity: 1, disposition: SemanticDisposition::TransientNoise,          provenance: Provenance::FrameworkDesign, description: "FHSS waveform transition (suppressible)" },
            MotifEntry { motif_class: MotifClass::LnaGainInstability,        min_severity: 1, disposition: SemanticDisposition::LnaGainInstability,      provenance: Provenance::FrameworkDesign, description: "Linear gain ramp, near-zero second derivative" },
            MotifEntry { motif_class: MotifClass::LoInstabilityPrecursor,    min_severity: 1, disposition: SemanticDisposition::LoInstabilityPrecursor,  provenance: Provenance::FrameworkDesign, description: "Recurrent boundary grazing with oscillatory slew" },
        ];
        let mut bank = Self::empty();
        for entry in MOTIFS {
            bank.register(entry);
        }
        bank
    }

    /// Register a new motif entry. Returns false if bank is full.
    pub fn register(&mut self, entry: MotifEntry) -> bool {
        if self.count >= M {
            return false;
        }
        self.entries[self.count] = Some(entry);
        self.count += 1;
        true
    }

    /// Look up a motif in the bank. Returns `Unknown` if no entry matches.
    ///
    /// Linear scan O(M). For M≤32 this is negligible per-observation cost.
    pub fn lookup(&self, motif: MotifClass, grammar: GrammarState) -> SemanticDisposition {
        for i in 0..self.count {
            if let Some(ref entry) = self.entries[i] {
                if entry.matches(motif, grammar) {
                    return entry.disposition;
                }
            }
        }
        // No match: endoductive Unknown
        SemanticDisposition::Unknown
    }

    /// Number of registered entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the bank is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over populated entries.
    pub fn entries(&self) -> impl Iterator<Item = &MotifEntry> {
        self.entries[..self.count]
            .iter()
            .filter_map(|e| e.as_ref())
    }
}

// ---------------------------------------------------------------
// Clock Stability Library
// ---------------------------------------------------------------
// Allan deviation σ_y(τ) slope classification for oscillator-based
// internal-cause heuristics (paper §10.3 "Clock and LO Instability").
//
// When the grammar trips at Boundary/Violation AND the Physics model
// returns ArrheniusModel with low activation energy, a parallel check
// of the Allan deviation signature can distinguish:
//   – TCXO first-warmup (τ^{-1} slope → white FM, settling within ~60 s)
//   – OCXO warmup (τ^{-3/2} slope → flicker FM, oven thermal lag)
//   – Free-run crystal (τ^{+1/2} slope → random walk, no oven)
//   – PLL acquisition (transient oscillation in σ_y at τ ≈ 1/f_bw)
//
// These are classified by fitting a log-log slope α to σ_y(τ) ∝ τ^α
// and matching against the IEEE Std 1139-2008 Table 1 canonical slopes.
//
// Reference:
//   IEEE Std 1139-2008, "Characterization of Clocks and Oscillators".
//   Allan (1966), Proc. IEEE 54(2):221.

/// Canonical clock / oscillator instability classes matched by Allan slope.
///
/// Classified by fitting log-log slope α to σ_y(τ) ∝ τ^α across the
/// provided τ window.  α ≈ –1 → white FM; α ≈ –3/2 → flicker FM (OCXO);
/// α ≈ +1/2 → random walk FM; α ≈ 0 → flicker phase noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownClockClass {
    /// White frequency modulation (α ≈ –1.0).
    /// Typical of a TCXO in steady-state or immediately post-warmup.
    TcxoSteadyState,
    /// Flicker frequency modulation (α ≈ –0.5, strong τ^0 floor).
    /// Typical of OCXO during oven thermal equilibration (~1–10 min).
    OcxoWarmup,
    /// Random walk frequency modulation (α ≈ +0.5).
    /// Typical of a free-run crystal without temperature compensation.
    FreeRunXtal,
    /// Transient oscillation around a fast-crossing τ value.
    /// Indicates an active PLL mid-acquisition or a loop bandwidth mismatch.
    PllAcquisition,
    /// Flicker phase noise (α ≈ –1.5), adjacent to carrier.
    /// Typical of a low-noise OCXO or Rb oscillator in steady-state.
    LowNoiseOcxo,
    /// Slope could not be determined (too few τ points or noisy data).
    Unknown,
}

impl KnownClockClass {
    /// Human-readable label for SigMF annotation or log emission.
    pub const fn label(self) -> &'static str {
        match self {
            KnownClockClass::TcxoSteadyState  => "TcxoSteadyState",
            KnownClockClass::OcxoWarmup        => "OcxoWarmup",
            KnownClockClass::FreeRunXtal       => "FreeRunXtal",
            KnownClockClass::PllAcquisition    => "PllAcquisition",
            KnownClockClass::LowNoiseOcxo      => "LowNoiseOcxo",
            KnownClockClass::Unknown           => "Unknown",
        }
    }

    /// Whether this class indicates an *internal* cause (clock/LO issue)
    /// rather than an *external* cause (channel interference).
    pub const fn is_internal_cause(self) -> bool {
        matches!(
            self,
            KnownClockClass::TcxoSteadyState
            | KnownClockClass::OcxoWarmup
            | KnownClockClass::FreeRunXtal
            | KnownClockClass::PllAcquisition
            | KnownClockClass::LowNoiseOcxo
        )
    }
}

/// Classify an oscillator's instability class from its Allan deviation curve.
///
/// # Arguments
/// - `sigma_y`  — array of Allan deviation values, one per τ point
/// - `taus`     — corresponding integration times τ (seconds), monotone increasing
///
/// Both slices must have the same length ≥ 3.  Shorter inputs return
/// `KnownClockClass::Unknown`.
///
/// ## Algorithm
///
/// Fits log-log slope α = Δ log(σ_y) / Δ log(τ) via least-squares over
/// all provided τ points, then maps the slope to a canonical class:
///
/// | α range          | Class              |
/// |------------------|--------------------|
/// | (−∞, −1.2)       | `LowNoiseOcxo`     |
/// | [−1.2, −0.7]     | `TcxoSteadyState`  |
/// | (−0.7, −0.1)     | `OcxoWarmup`       |
/// | [−0.1, +0.2)     | `PllAcquisition`   |
/// | [+0.2, ∞)        | `FreeRunXtal`      |
pub fn classify_clock_instability(sigma_y: &[f32], taus: &[f32]) -> KnownClockClass {
    if sigma_y.len() < 3 || taus.len() < 3 || sigma_y.len() != taus.len() {
        return KnownClockClass::Unknown;
    }
    let (sum_x, sum_y, sum_xx, sum_xy, m) = accumulate_log_sums(sigma_y, taus);
    if m < 3 {
        return KnownClockClass::Unknown;
    }
    let mf = m as f32;
    let denom = mf * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-9 {
        return KnownClockClass::Unknown;
    }
    let alpha = (mf * sum_xy - sum_x * sum_y) / denom;
    classify_slope(alpha)
}

fn accumulate_log_sums(sigma_y: &[f32], taus: &[f32]) -> (f32, f32, f32, f32, u32) {
    let log = |v: f32| -> f32 { crate::math::ln_f32(v.max(1e-38)) };
    let n = sigma_y.len().min(taus.len());
    let mut sum_x  = 0.0_f32;
    let mut sum_y  = 0.0_f32;
    let mut sum_xx = 0.0_f32;
    let mut sum_xy = 0.0_f32;
    let mut m = 0u32;
    for i in 0..n {
        if taus[i] > 0.0 && sigma_y[i] > 0.0 {
            let lx = log(taus[i]);
            let ly = log(sigma_y[i]);
            sum_x  += lx;
            sum_y  += ly;
            sum_xx += lx * lx;
            sum_xy += lx * ly;
            m += 1;
        }
    }
    (sum_x, sum_y, sum_xx, sum_xy, m)
}

fn classify_slope(alpha: f32) -> KnownClockClass {
    if alpha < -1.2 {
        KnownClockClass::LowNoiseOcxo
    } else if alpha <= -0.7 {
        KnownClockClass::TcxoSteadyState
    } else if alpha < -0.1 {
        KnownClockClass::OcxoWarmup
    } else if alpha < 0.2 {
        KnownClockClass::PllAcquisition
    } else {
        KnownClockClass::FreeRunXtal
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::GrammarState;

    #[test]
    fn default_bank_has_nine_entries() {
        let bank = HeuristicsBank::<32>::default_rf();
        assert_eq!(bank.len(), 9);
    }

    #[test]
    fn slow_drift_lookup_returns_pre_transition() {
        let bank = HeuristicsBank::<32>::default_rf();
        let disp = bank.lookup(
            MotifClass::PreFailureSlowDrift,
            GrammarState::Boundary(crate::grammar::ReasonCode::SustainedOutwardDrift),
        );
        assert_eq!(disp, SemanticDisposition::PreTransitionCluster);
    }

    #[test]
    fn unknown_motif_returns_unknown() {
        let bank = HeuristicsBank::<32>::default_rf();
        let disp = bank.lookup(MotifClass::Unknown, GrammarState::Admissible);
        assert_eq!(disp, SemanticDisposition::Unknown);
    }

    #[test]
    fn abrupt_onset_lookup() {
        let bank = HeuristicsBank::<32>::default_rf();
        let disp = bank.lookup(
            MotifClass::AbruptOnset,
            GrammarState::Violation,
        );
        assert_eq!(disp, SemanticDisposition::AbruptOnsetEvent);
    }

    #[test]
    fn bank_register_beyond_capacity_returns_false() {
        let mut bank = HeuristicsBank::<2>::empty();
        let entry = MotifEntry {
            motif_class: MotifClass::Unknown,
            min_severity: 0,
            disposition: SemanticDisposition::Unknown,
            provenance: Provenance::FrameworkDesign,
            description: "test",
        };
        assert!(bank.register(entry));
        assert!(bank.register(entry));
        assert!(!bank.register(entry), "should be full at M=2");
    }

    #[test]
    fn transient_excursion_requires_violation_severity() {
        let bank = HeuristicsBank::<32>::default_rf();
        // min_severity=2 (Violation), so Boundary should return Unknown
        let disp = bank.lookup(
            MotifClass::TransientExcursion,
            GrammarState::Boundary(crate::grammar::ReasonCode::SustainedOutwardDrift),
        );
        assert_eq!(disp, SemanticDisposition::Unknown,
            "TransientExcursion requires Violation severity");
    }

    // ── Clock stability library tests ────────────────────────────────────

    #[test]
    fn clock_labels_are_correct() {
        assert_eq!(KnownClockClass::TcxoSteadyState.label(), "TcxoSteadyState");
        assert_eq!(KnownClockClass::OcxoWarmup.label(),       "OcxoWarmup");
        assert_eq!(KnownClockClass::FreeRunXtal.label(),      "FreeRunXtal");
        assert_eq!(KnownClockClass::PllAcquisition.label(),   "PllAcquisition");
        assert_eq!(KnownClockClass::LowNoiseOcxo.label(),     "LowNoiseOcxo");
    }

    #[test]
    fn clock_all_are_internal() {
        for &cls in &[
            KnownClockClass::TcxoSteadyState,
            KnownClockClass::OcxoWarmup,
            KnownClockClass::FreeRunXtal,
            KnownClockClass::PllAcquisition,
            KnownClockClass::LowNoiseOcxo,
        ] {
            assert!(cls.is_internal_cause(), "{:?} should be internal", cls);
        }
        assert!(!KnownClockClass::Unknown.is_internal_cause());
    }

    #[test]
    fn classify_tcxo_steady_state_slope_minus_one() {
        // σ_y ∝ τ^{-1}: σ_y(τ) = 1e-11 / τ
        let taus:    [f32; 5] = [1.0, 2.0, 4.0, 8.0, 16.0];
        let sigma_y: [f32; 5] = [1e-11, 0.5e-11, 0.25e-11, 0.125e-11, 0.0625e-11];
        let cls = classify_clock_instability(&sigma_y, &taus);
        assert_eq!(cls, KnownClockClass::TcxoSteadyState, "α≈-1 slope: {:?}", cls);
    }

    #[test]
    fn classify_freerun_xtal_slope_plus_half() {
        // σ_y ∝ τ^{+0.5}: σ_y(τ) = 1e-11 * sqrt(τ)
        let taus:    [f32; 5] = [1.0, 4.0, 9.0, 16.0, 25.0];
        let sigma_y: [f32; 5] = [1e-11, 2e-11, 3e-11, 4e-11, 5e-11];
        let cls = classify_clock_instability(&sigma_y, &taus);
        assert_eq!(cls, KnownClockClass::FreeRunXtal, "α≈+0.5 slope: {:?}", cls);
    }

    #[test]
    fn classify_too_few_points_returns_unknown() {
        let taus:    [f32; 2] = [1.0, 2.0];
        let sigma_y: [f32; 2] = [1e-11, 0.5e-11];
        assert_eq!(classify_clock_instability(&sigma_y, &taus), KnownClockClass::Unknown);
    }
}
