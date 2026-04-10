//! Heuristics bank: typed degradation motifs for gas turbine engines.
//!
//! The heuristics bank is a finite, versioned repository of candidate
//! interpretive motifs. Each entry maps a structural pattern (channel
//! signature, drift pattern, slew pattern) to a candidate interpretation.
//!
//! The bank does NOT function as a classifier. It functions as a
//! structured retrieval system that preserves ambiguity when multiple
//! entries are compatible with observed evidence.

use crate::core::grammar::GrammarState;

/// Engine-specific reason codes.
///
/// Each code is a typed structural interpretation, NOT a mechanistic
/// diagnosis. "SustainedCompressorFouling" means "the residual pattern
/// is consistent with compressor fouling under stated conditions" —
/// it does NOT mean "compressor fouling has been confirmed."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineReasonCode {
    /// Persistent negative efficiency drift, stable flow capacity.
    SustainedCompressorFouling,
    /// Persistent negative HPT efficiency drift + positive flow capacity drift.
    TurbineErosionOnset,
    /// Positive slew in EGT deviation channel (accelerating margin loss).
    AcceleratingEgtMarginLoss,
    /// Coupled efficiency and flow capacity drift across multiple components.
    SealClearanceGrowth,
    /// Abrupt efficiency step with subsequent stabilization.
    ForeignObjectDamageSignature,
    /// Intermittent boundary excursions coupled with pressure-ratio anomaly.
    CompressorStallPrecursor,
    /// Vibration residual drift coupled with oil-temperature deviation.
    BearingWearCoupling,
    /// HPT efficiency step followed by accelerating EGT margin loss.
    ThermalBarrierCoatingSpallation,
    /// Post-wash residual does not return to pre-fouling baseline.
    WashRecoveryIncomplete,
    /// Single-cycle deviation with no structural continuation.
    TransientExcursionNotPersistent,
    /// Structural anomaly not matching any bank entry.
    UnclassifiedStructuralAnomaly,
    /// No anomaly detected.
    NoAnomaly,
    /// Multiple degradation mechanisms act simultaneously; ambiguity set.
    MultiFaultSuperposition,
    /// Fan degradation signature (relevant for FD003 / FD004).
    FanDegradationOnset,
    /// Combined HPC + fan degradation.
    CombinedHpcFanDegradation,
}

impl EngineReasonCode {
    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SustainedCompressorFouling => "SustainedCompressorFouling",
            Self::TurbineErosionOnset => "TurbineErosionOnset",
            Self::AcceleratingEgtMarginLoss => "AcceleratingEGTMarginLoss",
            Self::SealClearanceGrowth => "SealClearanceGrowth",
            Self::ForeignObjectDamageSignature => "ForeignObjectDamageSignature",
            Self::CompressorStallPrecursor => "CompressorStallPrecursor",
            Self::BearingWearCoupling => "BearingWearCoupling",
            Self::ThermalBarrierCoatingSpallation => "ThermalBarrierCoatingSpallation",
            Self::WashRecoveryIncomplete => "WashRecoveryIncomplete",
            Self::TransientExcursionNotPersistent => "TransientExcursionNotPersistent",
            Self::UnclassifiedStructuralAnomaly => "UnclassifiedStructuralAnomaly",
            Self::NoAnomaly => "NoAnomaly",
            Self::MultiFaultSuperposition => "MultiFaultSuperposition",
            Self::FanDegradationOnset => "FanDegradationOnset",
            Self::CombinedHpcFanDegradation => "CombinedHPCFanDegradation",
        }
    }

    /// Whether this code represents a structural anomaly.
    #[must_use]
    pub const fn is_anomalous(self) -> bool {
        !matches!(self, Self::NoAnomaly | Self::TransientExcursionNotPersistent)
    }
}

/// A typed heuristic bank entry.
#[derive(Debug, Clone, Copy)]
pub struct HeuristicEntry {
    /// The reason code this entry matches.
    pub code: EngineReasonCode,
    /// Minimum sustained drift magnitude (absolute) required.
    pub min_drift: f64,
    /// Required drift sign: -1.0 = negative, 1.0 = positive, 0.0 = any.
    pub drift_sign: f64,
    /// Minimum sustained slew magnitude (absolute) required.
    pub min_slew: f64,
    /// Required slew sign: -1.0 = negative, 1.0 = positive, 0.0 = any.
    pub slew_sign: f64,
    /// Whether this motif requires envelope approach or breach.
    pub requires_envelope_stress: bool,
    /// Minimum grammar state required for this motif to be considered.
    pub min_grammar_state: GrammarState,
}

/// The heuristics bank. Fixed-size array of entries, no heap.
pub struct HeuristicsBank {
    entries: [HeuristicEntry; 16],
    count: usize,
}

impl HeuristicsBank {
    /// Constructs the default gas turbine heuristics bank.
    ///
    /// This is the versioned bank for C-MAPSS evaluation.
    /// Each entry is documented with its structural rationale.
    #[must_use]
    pub const fn default_gas_turbine() -> Self {
        Self {
            entries: [
                // H-1: Compressor fouling — persistent negative drift, no slew
                HeuristicEntry {
                    code: EngineReasonCode::SustainedCompressorFouling,
                    min_drift: 0.001,
                    drift_sign: -1.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-2: Turbine erosion — negative efficiency drift
                HeuristicEntry {
                    code: EngineReasonCode::TurbineErosionOnset,
                    min_drift: 0.002,
                    drift_sign: -1.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-3: Accelerating EGT margin loss — positive slew
                HeuristicEntry {
                    code: EngineReasonCode::AcceleratingEgtMarginLoss,
                    min_drift: 0.001,
                    drift_sign: 0.0,
                    min_slew: 0.0005,
                    slew_sign: 1.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-4: FOD — large drift, low slew (step function)
                HeuristicEntry {
                    code: EngineReasonCode::ForeignObjectDamageSignature,
                    min_drift: 0.01,
                    drift_sign: 0.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: true,
                    min_grammar_state: GrammarState::Violation,
                },
                // H-5: Fan degradation onset
                HeuristicEntry {
                    code: EngineReasonCode::FanDegradationOnset,
                    min_drift: 0.001,
                    drift_sign: -1.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-6: Combined HPC + fan
                HeuristicEntry {
                    code: EngineReasonCode::CombinedHpcFanDegradation,
                    min_drift: 0.001,
                    drift_sign: -1.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-7: Seal clearance growth
                HeuristicEntry {
                    code: EngineReasonCode::SealClearanceGrowth,
                    min_drift: 0.001,
                    drift_sign: 0.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-8: TBC spallation
                HeuristicEntry {
                    code: EngineReasonCode::ThermalBarrierCoatingSpallation,
                    min_drift: 0.005,
                    drift_sign: -1.0,
                    min_slew: 0.001,
                    slew_sign: 1.0,
                    requires_envelope_stress: true,
                    min_grammar_state: GrammarState::Violation,
                },
                // H-9: Transient excursion — requires envelope stress but NO persistence
                HeuristicEntry {
                    code: EngineReasonCode::TransientExcursionNotPersistent,
                    min_drift: 0.0,
                    drift_sign: 0.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: true,
                    min_grammar_state: GrammarState::Boundary,
                },
                // H-10: Multi-fault superposition
                HeuristicEntry {
                    code: EngineReasonCode::MultiFaultSuperposition,
                    min_drift: 0.001,
                    drift_sign: 0.0,
                    min_slew: 0.0,
                    slew_sign: 0.0,
                    requires_envelope_stress: false,
                    min_grammar_state: GrammarState::Boundary,
                },
                // Padding entries (unused)
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
                HeuristicEntry { code: EngineReasonCode::NoAnomaly, min_drift: 0.0, drift_sign: 0.0, min_slew: 0.0, slew_sign: 0.0, requires_envelope_stress: false, min_grammar_state: GrammarState::Admissible },
            ],
            count: 10,
        }
    }

    /// Matches the current residual sign and grammar state against the bank.
    ///
    /// Returns the best-matching reason code. If multiple entries match,
    /// returns the one with the highest minimum grammar state requirement
    /// (most specific match). If none match, returns `NoAnomaly`.
    #[must_use]
    pub fn match_motif(
        &self,
        drift: f64,
        slew: f64,
        grammar_state: GrammarState,
        envelope_stressed: bool,
    ) -> EngineReasonCode {
        let mut best_code = EngineReasonCode::NoAnomaly;
        let mut best_severity: u8 = 0;

        let mut i = 0;
        while i < self.count {
            let entry = &self.entries[i];

            // Check grammar state prerequisite
            if grammar_state.severity() < entry.min_grammar_state.severity() {
                i += 1;
                continue;
            }

            // Check envelope stress prerequisite
            if entry.requires_envelope_stress && !envelope_stressed {
                i += 1;
                continue;
            }

            // Check drift magnitude
            if drift.abs() < entry.min_drift {
                i += 1;
                continue;
            }

            // Check drift sign
            if entry.drift_sign < 0.0 && drift >= 0.0 {
                i += 1;
                continue;
            }
            if entry.drift_sign > 0.0 && drift <= 0.0 {
                i += 1;
                continue;
            }

            // Check slew magnitude
            if slew.abs() < entry.min_slew {
                i += 1;
                continue;
            }

            // Check slew sign
            if entry.slew_sign < 0.0 && slew >= 0.0 {
                i += 1;
                continue;
            }
            if entry.slew_sign > 0.0 && slew <= 0.0 {
                i += 1;
                continue;
            }

            // Match found. Keep the most specific (highest severity requirement).
            let sev = entry.min_grammar_state.severity();
            if sev >= best_severity {
                best_severity = sev;
                best_code = entry.code;
            }

            i += 1;
        }

        // If grammar state is Boundary or Violation but no specific match,
        // return UnclassifiedStructuralAnomaly
        if best_code == EngineReasonCode::NoAnomaly
            && grammar_state.severity() >= GrammarState::Boundary.severity()
        {
            return EngineReasonCode::UnclassifiedStructuralAnomaly;
        }

        best_code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_anomaly_in_admissible() {
        let bank = HeuristicsBank::default_gas_turbine();
        let code = bank.match_motif(0.0, 0.0, GrammarState::Admissible, false);
        assert_eq!(code, EngineReasonCode::NoAnomaly);
    }

    #[test]
    fn test_fouling_match() {
        let bank = HeuristicsBank::default_gas_turbine();
        let code = bank.match_motif(-0.005, 0.0, GrammarState::Boundary, false);
        // Should match compressor fouling or turbine erosion (negative drift, Boundary)
        assert!(code.is_anomalous());
    }

    #[test]
    fn test_unclassified_in_boundary() {
        let bank = HeuristicsBank::default_gas_turbine();
        // Very small drift, but in Boundary state
        let code = bank.match_motif(0.0001, 0.0, GrammarState::Boundary, false);
        assert_eq!(code, EngineReasonCode::UnclassifiedStructuralAnomaly);
    }
}
