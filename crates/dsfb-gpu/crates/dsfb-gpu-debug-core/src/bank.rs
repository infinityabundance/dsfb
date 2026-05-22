//! Heuristics bank and the Semantic Non-Bypass enforcement point.
//!
//! This module is the *only* place in the workspace that mints an
//! admissible `Episode`. The Semantic Non-Bypass Axiom from the paper
//! requires that no final semantic verdict be derived directly from GPU
//! detector evidence; the type system here enforces that property: an
//! `Episode` carries a `BankAdmissionToken` whose constructor is
//! private to this module, and the case-file emitter (Section G) will
//! reject any `Episode` produced without a token.
//!
//! The bank itself is a tiny compile-time const array of eight
//! `HeuristicEntry` records. Each entry describes:
//!
//! * Which detector bits the candidate's `union_mask` must cover.
//! * The Q16.16 minimums each of the nine fusion axes must exceed for
//!   the candidate to be admitted as this motif.
//! * A "confuser bit": if that detector is present in the candidate,
//!   admission requires the candidate's axes to clear an *extra*
//!   margin. This is axis 9 (confuser suppression) at work.
//! * A reason code and a tie-break priority used to pick a single
//!   `BankMotif` when more than one entry would otherwise admit.
//!
//! The bank's canonical bytes (motif IDs concatenated with comma
//! separators) are hashed to produce `BANK_HASH`, which the contract
//! pins.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::candidate::CandidateInterval;
use crate::consensus::ConsensusCell;
use crate::fixed::Q16;
use crate::grammar::{GrammarState, ReasonCode};
use crate::hash::{format_digest, sha256};
use crate::motif::MotifClass;

/// The eight canonical bank motifs.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum BankMotif {
    /// Sustained latency increase over many windows.
    LatencyRamp = 0,
    /// Concentrated error-code burst.
    ErrorBurst = 1,
    /// Single-window latency spike followed by a recovery edge.
    SlewShockRecovery = 2,
    /// Long-baseline drift without abrupt steps.
    SustainedDegradation = 3,
    /// Repeated sign-alternations in slew.
    OscillationInstability = 4,
    /// Anomaly localized to a particular route within the entity.
    LocalizedRouteFault = 5,
    /// Latency drift coupled with non-zero error rate — pre-cascade.
    FanoutCascadeCandidate = 6,
    /// Confuser-like transient that does not justify any other motif.
    ConfuserTransient = 7,
}

impl BankMotif {
    /// Number of motifs in the canonical bank.
    pub const COUNT: usize = 8;

    /// Stable lowercase-snake-case name. Part of the bank's canonical
    /// bytes; renaming changes [`bank_hash`] and is a contract breach.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LatencyRamp => "latency_ramp",
            Self::ErrorBurst => "error_burst",
            Self::SlewShockRecovery => "slew_shock_recovery",
            Self::SustainedDegradation => "sustained_degradation",
            Self::OscillationInstability => "oscillation_instability",
            Self::LocalizedRouteFault => "localized_route_fault",
            Self::FanoutCascadeCandidate => "fanout_cascade_candidate",
            Self::ConfuserTransient => "confuser_transient",
        }
    }
}

/// One bank-entry record. All thresholds are Q16.16 raw integers stored
/// alongside the motif identifier so the type can sit in a `const`
/// array.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct HeuristicEntry {
    /// Which canonical motif this entry admits.
    pub motif: BankMotif,
    /// Required detector bits. The candidate's `union_mask` must cover
    /// every bit in this value for admission to be considered.
    pub required_detector_bits: u32,
    /// Minimum peak detector count any single cell must have hit for
    /// admission. Defends against thin-cover candidates whose union is
    /// rich only because many cells contributed one bit each.
    pub min_peak_consensus_q_raw: i32,
    /// Minimum candidate length (in windows). Length-1 intervals are
    /// admissible only by the `ConfuserTransient` entry.
    pub min_length_windows: u32,
    /// Maximum candidate length (in windows). Motifs that are bounded
    /// in duration (e.g. a slew-shock-recovery is at most a single
    /// shock window plus a few recovery windows) set this so a longer
    /// candidate cannot accidentally admit as them. `u32::MAX` means
    /// no upper bound.
    pub max_length_windows: u32,
    /// Per-axis gates. Indexed 0..9 for axes 1..9; index 0 is unused
    /// (axes are 1-based by paper convention) so the gate values can be
    /// read off the same axis numbering used in the paper.
    pub axis_gates_q_raw: [i32; 10],
    /// Detector bit that, if present in the candidate's `union_mask`,
    /// triggers the confuser-suppression margin. `0` means "no
    /// confuser bit".
    pub confuser_bit: u32,
    /// Additional Q16 margin the residual axis must clear when the
    /// confuser bit is present.
    pub confuser_extra_margin_q_raw: i32,
    /// Tie-break priority when two entries both admit. Higher wins.
    pub tie_break_priority: u8,
    /// Grammar state attached to admitted episodes from this entry.
    pub peak_grammar_state: GrammarState,
    /// Reason code attached to admitted episodes.
    pub reason_code: ReasonCode,
}

/// The canonical 8-entry bank. Indexed by `BankMotif as usize`.
///
/// Axis gates use the index convention `axis_gates_q_raw[i]` = gate for
/// axis i, with `i` in `1..=9`. Index 0 is intentionally unused so the
/// paper's axis numbering is preserved verbatim in the source.
pub const CANONICAL_BANK: [HeuristicEntry; BankMotif::COUNT] = [
    // Motif 0 — LatencyRamp: sustained drift on the latency axis.
    HeuristicEntry {
        motif: BankMotif::LatencyRamp,
        required_detector_bits: MotifClass::ResidualSpike.bit_mask()
            | MotifClass::DriftRamp.bit_mask()
            | MotifClass::SustainedResidualElevation.bit_mask(),
        min_peak_consensus_q_raw: 0x2000, // ≥ ⅛ of detector budget
        min_length_windows: 3,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [
            0,           // axis 0 (unused)
            10 * 65_536, // axis 1 — residual magnitude ≥ 10 ms
            5 * 65_536,  // axis 2 — drift persistence ≥ 5 ms
            0,           // axis 3 — no slew floor for a ramp
            0x2000,      // axis 4 — temporal locality ≥ ⅛
            0,           // axis 5 — entity locality, CPU-only
            0,           // axis 6 — adjacency, CPU-only
            0x2000,      // axis 7 — consensus ≥ ⅛
            1,           // axis 8 — admissibility marker (≥ 1 raw)
            0,           // axis 9 — confuser handled separately
        ],
        confuser_bit: MotifClass::ConfuserLikeTransient.bit_mask(),
        confuser_extra_margin_q_raw: 5 * 65_536,
        tie_break_priority: 5,
        peak_grammar_state: GrammarState::Violation,
        reason_code: ReasonCode::SustainedOutwardDrift,
    },
    // Motif 1 — ErrorBurst.
    HeuristicEntry {
        motif: BankMotif::ErrorBurst,
        required_detector_bits: MotifClass::ErrorRateBurst.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 2,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [0, 0, 0, 0, 0, 0, 0, 0x1000, 1, 0],
        confuser_bit: MotifClass::ConfuserLikeTransient.bit_mask(),
        confuser_extra_margin_q_raw: 0x2000,
        tie_break_priority: 4,
        peak_grammar_state: GrammarState::Violation,
        reason_code: ReasonCode::EnvelopeViolation,
    },
    // Motif 2 — SlewShockRecovery. Bounded length so a long ramp
    // whose tail-edge cell crosses the slew threshold cannot admit as
    // a slew-shock candidate.
    HeuristicEntry {
        motif: BankMotif::SlewShockRecovery,
        required_detector_bits: MotifClass::SlewShock.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 1,
        max_length_windows: 8,
        axis_gates_q_raw: [0, 0, 0, 20 * 65_536, 0, 0, 0, 0x1000, 1, 0],
        confuser_bit: 0,
        confuser_extra_margin_q_raw: 0,
        tie_break_priority: 6,
        peak_grammar_state: GrammarState::Recovery,
        reason_code: ReasonCode::AbruptSlewViolation,
    },
    // Motif 3 — SustainedDegradation: drift without ramp.
    HeuristicEntry {
        motif: BankMotif::SustainedDegradation,
        required_detector_bits: MotifClass::SustainedResidualElevation.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 5,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [0, 5 * 65_536, 5 * 65_536, 0, 0x2000, 0, 0, 0x1000, 1, 0],
        confuser_bit: MotifClass::ConfuserLikeTransient.bit_mask(),
        confuser_extra_margin_q_raw: 5 * 65_536,
        tie_break_priority: 3,
        peak_grammar_state: GrammarState::Violation,
        reason_code: ReasonCode::SustainedOutwardDrift,
    },
    // Motif 4 — OscillationInstability.
    HeuristicEntry {
        motif: BankMotif::OscillationInstability,
        required_detector_bits: MotifClass::Oscillation.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 3,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [0, 0, 0, 0, 0x1000, 0, 0, 0x1000, 1, 0],
        confuser_bit: 0,
        confuser_extra_margin_q_raw: 0,
        tie_break_priority: 3,
        peak_grammar_state: GrammarState::Boundary,
        reason_code: ReasonCode::RecurrentBoundaryGrazing,
    },
    // Motif 5 — LocalizedRouteFault.
    HeuristicEntry {
        motif: BankMotif::LocalizedRouteFault,
        required_detector_bits: MotifClass::RouteLocalAnomaly.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 2,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [0, 5 * 65_536, 0, 0, 0, 0, 0, 0x1000, 1, 0],
        confuser_bit: 0,
        confuser_extra_margin_q_raw: 0,
        tie_break_priority: 2,
        peak_grammar_state: GrammarState::Boundary,
        reason_code: ReasonCode::BoundaryApproach,
    },
    // Motif 6 — FanoutCascadeCandidate.
    HeuristicEntry {
        motif: BankMotif::FanoutCascadeCandidate,
        required_detector_bits: MotifClass::FanoutPrecursor.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 2,
        max_length_windows: u32::MAX,
        axis_gates_q_raw: [0, 0, 3 * 65_536, 0, 0, 0, 0, 0x1000, 1, 0],
        confuser_bit: 0,
        confuser_extra_margin_q_raw: 0,
        tie_break_priority: 2,
        peak_grammar_state: GrammarState::Boundary,
        reason_code: ReasonCode::BoundaryApproach,
    },
    // Motif 7 — ConfuserTransient.
    HeuristicEntry {
        motif: BankMotif::ConfuserTransient,
        required_detector_bits: MotifClass::ConfuserLikeTransient.bit_mask(),
        min_peak_consensus_q_raw: 0x1000,
        min_length_windows: 1,
        max_length_windows: 2,
        axis_gates_q_raw: [0, 5 * 65_536, 0, 0, 0, 0, 0, 0x1000, 1, 0],
        confuser_bit: 0,
        confuser_extra_margin_q_raw: 0,
        tie_break_priority: 1,
        peak_grammar_state: GrammarState::Boundary,
        reason_code: ReasonCode::SingleCrossing,
    },
];

/// Canonical bytes for the bank: comma-joined motif names, no
/// whitespace. Reordering motifs or renaming any of them changes
/// `BANK_HASH`.
fn bank_canonical_bytes() -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);
    for (i, entry) in CANONICAL_BANK.iter().enumerate() {
        if i > 0 {
            buf.push(b',');
        }
        buf.extend_from_slice(entry.motif.name().as_bytes());
    }
    buf
}

/// SHA-256 digest of the canonical bank bytes. The contract's
/// `bank_hash` field pins this value.
#[must_use]
pub fn bank_hash() -> [u8; 32] {
    sha256(&bank_canonical_bytes())
}

/// Hex-encoded `sha256:`-prefixed bank hash, ready to drop into
/// `contract.toml`.
#[must_use]
pub fn bank_hash_string() -> [u8; 71] {
    format_digest(&bank_hash())
}

/// Zero-sized authorization token. Carried by every admissible
/// `Episode` to prove that the bank stage minted it. The constructor is
/// private to this module: nothing outside `bank` can produce a token,
/// which is what enforces the Semantic Non-Bypass Axiom at the type
/// level.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct BankAdmissionToken {
    _private: (),
}

impl BankAdmissionToken {
    /// Constructor visible only to `bank::collapse`.
    fn fresh() -> Self {
        Self { _private: () }
    }
}

/// One bank-admitted episode. Constructed only inside `collapse`; the
/// rest of the workspace receives `Episode` values without any ability
/// to fabricate one.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Episode {
    /// Entity this episode belongs to.
    pub entity_id: u32,
    /// Inclusive start window.
    pub start_window: u32,
    /// Exclusive end window.
    pub end_window: u32,
    /// Bank motif this episode was admitted as.
    pub motif: BankMotif,
    /// Reason code at the peak window.
    pub reason: ReasonCode,
    /// Peak grammar state during the episode.
    pub peak_state: GrammarState,
    /// Peak residual magnitude observed during the episode.
    pub peak_residual_q: Q16,
    /// Peak drift observed during the episode.
    pub peak_drift_q: Q16,
    /// Peak slew observed during the episode.
    pub peak_slew_q: Q16,
    /// Number of detector bits represented in the candidate's union
    /// mask. Carried for audit.
    pub detector_bit_count: u32,
    /// Bank admission token. The case-file emitter checks that this
    /// field is `Some(_)` on every episode it sees; the `None` variant
    /// exists solely for the bypass-attempt test and is never produced
    /// by `collapse`.
    pub admission: Option<BankAdmissionToken>,
}

impl Episode {
    /// Constructor used by tests that *intentionally* bypass the bank
    /// stage so the case-file emitter can reject the result. Outside
    /// tests this is never the right thing to call.
    #[must_use]
    pub const fn bypass_for_testing(
        entity_id: u32,
        start_window: u32,
        end_window: u32,
        motif: BankMotif,
    ) -> Self {
        Self {
            entity_id,
            start_window,
            end_window,
            motif,
            reason: ReasonCode::Admissible,
            peak_state: GrammarState::Admissible,
            peak_residual_q: Q16::ZERO,
            peak_drift_q: Q16::ZERO,
            peak_slew_q: Q16::ZERO,
            detector_bit_count: 0,
            admission: None,
        }
    }

    /// Was this episode admitted by the bank? Equivalent to
    /// `self.admission.is_some()`, named so the case-file emitter reads
    /// like the policy statement it implements.
    #[must_use]
    pub const fn is_bank_admitted(&self) -> bool {
        self.admission.is_some()
    }
}

/// Apply the 8-entry bank to the candidate intervals, returning the
/// admitted `Episode` list.
///
/// `consensus` is consulted for axis 5 (entity locality) which compares
/// the candidate entity's activity against the rest of the grid at the
/// candidate's window range.
#[must_use]
pub fn collapse(
    candidates: &[CandidateInterval],
    consensus: &[ConsensusCell],
    n_windows: u32,
    n_entities: u32,
) -> Vec<Episode> {
    let mut out: Vec<Episode> = Vec::new();
    for cand in candidates {
        if let Some(episode) = match_candidate(cand, consensus, n_windows, n_entities) {
            out.push(episode);
        }
    }
    out
}

fn match_candidate(
    cand: &CandidateInterval,
    consensus: &[ConsensusCell],
    n_windows: u32,
    n_entities: u32,
) -> Option<Episode> {
    // Highest declared tie_break_priority wins. Length-based selection
    // is enforced earlier via each entry's `min_length_windows` and
    // `max_length_windows`, so two entries that both pass admission are
    // genuinely competing on semantic priority by the time they reach
    // this comparison.
    let mut best: Option<(u8, Episode)> = None;
    for entry in &CANONICAL_BANK {
        if let Some(ep) = try_admit(entry, cand, consensus, n_windows, n_entities) {
            match best {
                None => best = Some((entry.tie_break_priority, ep)),
                Some((p, _)) if entry.tie_break_priority > p => {
                    best = Some((entry.tie_break_priority, ep));
                }
                _ => {}
            }
        }
    }
    best.map(|(_, ep)| ep)
}

fn try_admit(
    entry: &HeuristicEntry,
    cand: &CandidateInterval,
    consensus: &[ConsensusCell],
    n_windows: u32,
    n_entities: u32,
) -> Option<Episode> {
    // 1. Coverage gate: union mask must cover every required detector
    //    bit.
    if (cand.union_mask & entry.required_detector_bits) != entry.required_detector_bits {
        return None;
    }

    // 2. Length gate (both sides).
    if cand.length_windows < entry.min_length_windows
        || cand.length_windows > entry.max_length_windows
    {
        return None;
    }

    // 3. Per-axis Q16 gates (axes 1..=4 and 7 — the GPU-side axes).
    let peaks = [
        Q16::ZERO,             // axis 0 — unused
        cand.peak_residual_q,  // axis 1
        cand.peak_drift_q,     // axis 2
        cand.peak_slew_q,      // axis 3
        cand.peak_temporal_q,  // axis 4
        Q16::ZERO,             // axis 5 — filled below
        Q16::ZERO,             // axis 6 — CPU placeholder
        cand.peak_consensus_q, // axis 7
        Q16::ONE,              // axis 8 — admissibility marker
        Q16::ZERO,             // axis 9 — handled via confuser
    ];

    for axis in 1..=4 {
        if peaks[axis].raw() < entry.axis_gates_q_raw[axis] {
            return None;
        }
    }
    if peaks[7].raw() < entry.axis_gates_q_raw[7] {
        return None;
    }
    // Axis 8 is always Q16::ONE for an admissibility candidate; the gate
    // is `1` raw, so this is always true for well-formed candidates and
    // false for anyone trying to inject a zero-evidence episode.
    if peaks[8].raw() < entry.axis_gates_q_raw[8] {
        return None;
    }

    // 4. Axis 5 — entity locality. R.5 moved this computation into the
    //    candidate stage (CPU `prepare_with_detectors` second pass and
    //    CUDA `candidate_collapse_kernel` axis-5 sum), so the bank now
    //    reads the precomputed Q16 averages directly. The predicate
    //    "entity is exceptionally hot relative to grid" is unchanged
    //    (entity_avg > grid_avg); only where it is computed moved.
    //
    //    Same comparison as the pre-R.5 inline loop: integer-i32 Q16
    //    raw comparison, no fast-math, no floating-point.
    //
    //    Note: `consensus`, `n_windows`, `n_entities` are still in the
    //    function signature because v0 path callers may pass them.
    //    After R.3b strips the consensus D2H entirely they become
    //    unused and can drop, but R.5's scope is the byte change only.
    let _ = (consensus, n_windows, n_entities);
    let entity_avg = i64::from(cand.entity_avg_q.raw());
    let grid_avg = i64::from(cand.grid_avg_q.raw());
    if entity_avg <= grid_avg {
        // Entity is not exceptionally hot relative to the grid. Reject
        // anything other than the confuser entry which intentionally
        // does not require entity locality.
        if !matches!(entry.motif, BankMotif::ConfuserTransient) {
            return None;
        }
    }

    // 5. Confuser-suppression (axis 9). If the candidate carries the
    //    confuser bit, require the configured extra margin on the
    //    residual axis. Without that margin the bank rejects so the
    //    confuser transient does not masquerade as a real motif.
    if entry.confuser_bit != 0 && (cand.union_mask & entry.confuser_bit) != 0 {
        let needed = entry.axis_gates_q_raw[1].saturating_add(entry.confuser_extra_margin_q_raw);
        if cand.peak_residual_q.raw() < needed {
            return None;
        }
    }

    Some(Episode {
        entity_id: cand.entity_id,
        start_window: cand.start_window,
        end_window: cand.end_window,
        motif: entry.motif,
        reason: entry.reason_code,
        peak_state: entry.peak_grammar_state,
        peak_residual_q: cand.peak_residual_q,
        peak_drift_q: cand.peak_drift_q,
        peak_slew_q: cand.peak_slew_q,
        detector_bit_count: cand.union_mask.count_ones(),
        admission: Some(BankAdmissionToken::fresh()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{prepare_with_detectors, CandidateConfig};
    use crate::consensus::form as consensus_form;
    use crate::detector::{evaluate as detector_evaluate, DetectorThresholds};
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
    use crate::residual::{compute as residual_compute, Baseline};
    use crate::sign::compute as sign_compute;
    use crate::window::compute_features;

    const ALPHA: Q16 = Q16::from_raw(0x2000);

    fn run_pipeline() -> Vec<Episode> {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let detectors = detector_evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let consensus = consensus_form(&signs, &detectors, N_WINDOWS, N_ENTITIES);
        let masks: Vec<u32> = detectors.iter().map(|d| d.detector_mask).collect();
        let candidates = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        collapse(&candidates, &consensus, N_WINDOWS, N_ENTITIES)
    }

    #[test]
    fn bank_count_is_eight() {
        assert_eq!(BankMotif::COUNT, 8);
        assert_eq!(CANONICAL_BANK.len(), 8);
    }

    #[test]
    fn bank_hash_is_stable() {
        let a = bank_hash();
        let b = bank_hash();
        assert_eq!(a, b);
        assert_ne!(a, [0u8; 32]);
    }

    #[test]
    fn bank_hash_string_carries_sha256_prefix() {
        let s = bank_hash_string();
        assert!(s.starts_with(b"sha256:"));
        assert_eq!(s.len(), 71);
    }

    #[test]
    fn collapse_is_deterministic() {
        let a = run_pipeline();
        let b = run_pipeline();
        assert_eq!(a, b);
    }

    #[test]
    fn ramp_episode_admits_to_latency_ramp() {
        let episodes = run_pipeline();
        let ramp = episodes
            .iter()
            .find(|ep| ep.entity_id == 3 && ep.start_window <= 25 && ep.end_window >= 30);
        assert!(ramp.is_some(), "no ramp episode admitted");
        let ramp = ramp.unwrap();
        assert_eq!(ramp.motif, BankMotif::LatencyRamp);
        assert!(ramp.is_bank_admitted());
    }

    #[test]
    fn burst_episode_admits_to_error_burst() {
        let episodes = run_pipeline();
        let burst = episodes
            .iter()
            .find(|ep| ep.entity_id == 7 && ep.start_window <= 62 && ep.end_window >= 65);
        assert!(burst.is_some(), "no burst episode admitted");
        let burst = burst.unwrap();
        assert_eq!(burst.motif, BankMotif::ErrorBurst);
    }

    #[test]
    fn shock_episode_admits_to_slew_shock_recovery_motif() {
        let episodes = run_pipeline();
        let shock = episodes
            .iter()
            .find(|ep| ep.entity_id == 11 && ep.start_window <= 90 && ep.end_window >= 91);
        assert!(shock.is_some(), "no shock episode admitted");
        let shock = shock.unwrap();
        assert_eq!(shock.motif, BankMotif::SlewShockRecovery);
    }

    #[test]
    fn every_admitted_episode_carries_a_token() {
        let episodes = run_pipeline();
        assert!(!episodes.is_empty());
        for ep in &episodes {
            assert!(
                ep.is_bank_admitted(),
                "episode without admission token: {ep:?}"
            );
        }
    }

    #[test]
    fn bypass_constructor_produces_unadmitted_episode() {
        let ep = Episode::bypass_for_testing(0, 0, 1, BankMotif::ConfuserTransient);
        assert!(!ep.is_bank_admitted());
        assert!(ep.admission.is_none());
    }
}
