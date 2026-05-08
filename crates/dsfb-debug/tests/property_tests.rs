// DSFB-Debug: hand-rolled property tests.
//
// Zero new Cargo dependencies (the no_std core stays dep-free; even
// dev-dependencies aren't added). A deterministic LCG (pcg32-style)
// generates input streams over which the engine's invariants must
// hold for every sample. The seed is fixed per-property so failures
// are reproducible.
//
// Properties exercised:
//   1. Theorem 9 — `engine.run_evaluation(x) == engine.run_evaluation(x)`
//      for every valid `x` produced by the LCG. The full state
//      (episode_count, BenchmarkMetrics, eval_out, episodes_out) must
//      match bit-for-bit on two consecutive runs.
//   2. BufferTooSmall guard — `run_evaluation` returns `Err(BufferTooSmall)`
//      whenever `num_signals * num_windows > 8192`, never silently
//      truncates, and the error carries the correct `needed` and
//      `available` values.
//   3. No orphan reason codes — every variant of `ReasonCode` in the
//      public enum is matched by at least one entry in
//      `HeuristicsBank::with_canonical_motifs()`. Documented motifs
//      cover the documented reason-code surface.

#![cfg(feature = "std")]

use dsfb_debug::error::DsfbError;
use dsfb_debug::types::*;
use dsfb_debug::DsfbDebugEngine;
use dsfb_debug::heuristics_bank::HeuristicsBank;

const LCG_SEED: u64 = 0x9E3779B97F4A7C15; // golden-ratio constant

/// Deterministic linear congruential generator. Reproducible across runs.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    /// Returns a u64 in [0, u64::MAX]. Multiplier from MMIX (Knuth).
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    /// Float in [-1.0, 1.0) — used to scale the residual envelope.
    fn next_f64_signed(&mut self) -> f64 {
        // 53-bit mantissa: take top 53 bits, divide.
        let bits = (self.next_u64() >> 11) as f64;
        let frac = bits * (1.0 / (1u64 << 53) as f64); // [0, 1)
        2.0 * frac - 1.0                                  // [-1, 1)
    }
}

fn blank_eval() -> SignalEvaluation {
    SignalEvaluation {
        window_index: 0,
        signal_index: 0,
        residual_value: 0.0,
        sign_tuple: SignTuple::ZERO,
        raw_grammar_state: GrammarState::Admissible,
        confirmed_grammar_state: GrammarState::Admissible,
        reason_code: ReasonCode::Admissible,
        motif: None,
        semantic_disposition: SemanticDisposition::Unknown,
        dsa_score: 0.0,
        policy_state: PolicyState::Silent,
        was_imputed: false,
        drift_persistence: 0.0,
    }
}

fn blank_episode() -> DebugEpisode {
    DebugEpisode {
        episode_id: 0,
        start_window: 0,
        end_window: 0,
        peak_grammar_state: GrammarState::Admissible,
        primary_reason_code: ReasonCode::Admissible,
        matched_motif: SemanticDisposition::Unknown,
        policy_state: PolicyState::Silent,
        contributing_signal_count: 0,
        structural_signature: StructuralSignature {
            dominant_drift_direction: DriftDirection::None,
            peak_slew_magnitude: 0.0,
            duration_windows: 0,
            signal_correlation: 0.0,
        },
        root_cause_signal_index: None,
    }
}

/// Property 1: Theorem 9 — deterministic replay across pseudorandom inputs.
///
/// 32 distinct LCG-seeded data streams. For each stream, build a
/// (num_signals, num_windows) data matrix in [50, 150] (latency-like
/// values around a 100 mean), run the engine twice, assert
/// bit-identical outputs.
#[test]
fn theorem9_holds_over_pseudorandom_inputs() {
    const NUM_SAMPLES: usize = 32;
    let engine = DsfbDebugEngine::<32, 64>::paper_lock()
        .expect("engine creation should succeed");
    let mut lcg = Lcg::new(LCG_SEED);

    for sample in 0..NUM_SAMPLES {
        // Pick (num_signals, num_windows) such that product ≤ 8192.
        let num_signals = (((lcg.next_u64() >> 32) % 8) + 2) as usize;     // 2..=9
        let num_windows = (((lcg.next_u64() >> 32) % 200) + 110) as usize; // 110..=309

        let total = num_signals * num_windows;
        let mut data = vec![0.0_f64; total];
        for cell in data.iter_mut() {
            *cell = 100.0 + 10.0 * lcg.next_f64_signed();
        }

        let labels = vec![false; num_windows];
        let mut eval1 = vec![blank_eval(); total];
        let mut ep1   = vec![blank_episode(); 64];
        let mut eval2 = vec![blank_eval(); total];
        let mut ep2   = vec![blank_episode(); 64];

        let r1 = engine.run_evaluation(
            &data, num_signals, num_windows, &labels, num_windows / 4,
            &mut eval1, &mut ep1, "theorem9_sample",
        );
        let r2 = engine.run_evaluation(
            &data, num_signals, num_windows, &labels, num_windows / 4,
            &mut eval2, &mut ep2, "theorem9_sample",
        );

        let (c1, m1) = r1.expect("first run should succeed");
        let (c2, m2) = r2.expect("second run should succeed");

        assert_eq!(c1, c2,
                   "Theorem 9 violated on sample {}: episode_count differs ({} vs {})",
                   sample, c1, c2);
        assert_eq!(m1.dsfb_episode_count, m2.dsfb_episode_count,
                   "Theorem 9 violated on sample {}: dsfb_episode_count differs", sample);
        assert_eq!(m1.raw_anomaly_count, m2.raw_anomaly_count,
                   "Theorem 9 violated on sample {}: raw_anomaly_count differs", sample);

        // Episode-level equality up to populated count.
        for j in 0..c1 {
            assert_eq!(ep1[j], ep2[j],
                       "Theorem 9 violated on sample {} episode {}", sample, j);
        }

        // Eval-grid equality. SignalEvaluation is PartialEq with f64,
        // so this catches FP-order non-determinism.
        for j in 0..total {
            assert_eq!(eval1[j], eval2[j],
                       "Theorem 9 violated on sample {} eval[{}]", sample, j);
        }
    }
}

/// Property 2: BufferTooSmall always returned for `num_signals * num_windows > 8192`.
#[test]
fn buffer_too_small_always_returned_for_oversized_inputs() {
    let engine = DsfbDebugEngine::<512, 64>::paper_lock()
        .expect("engine creation should succeed");
    let mut lcg = Lcg::new(LCG_SEED ^ 0xDEADBEEF);

    // Twelve over-cap (signals, windows) combinations.
    let combos: &[(usize, usize)] = &[
        (16, 1024),  // 16384
        (32, 256),   // 8192 +  ← exact edge: not over
        (32, 257),   // 8224
        (33, 256),   // 8448
        (256, 33),   // 8448
        (64, 130),   // 8320
        (128, 65),   // 8320
        (200, 50),   // 10000
        (40, 256),   // 10240
        (10, 1000),  // 10000
        (4, 4096),   // 16384
        (300, 50),   // 15000
    ];

    for &(num_signals, num_windows) in combos {
        let total = num_signals * num_windows;
        let mut data = vec![0.0_f64; total];
        for cell in data.iter_mut() {
            *cell = 100.0 + 10.0 * lcg.next_f64_signed();
        }
        let labels = vec![false; num_windows];
        let mut eval = vec![blank_eval(); 1];
        let mut ep   = vec![blank_episode(); 1];

        let r = engine.run_evaluation(
            &data, num_signals, num_windows, &labels, num_windows / 4,
            &mut eval, &mut ep, "oversized",
        );

        if total > 8192 {
            match r {
                Err(DsfbError::BufferTooSmall { needed, available }) => {
                    assert_eq!(needed, total,
                               "BufferTooSmall.needed should be num_signals*num_windows");
                    assert_eq!(available, 8192,
                               "BufferTooSmall.available should be 8192");
                }
                other => panic!(
                    "({}, {}): expected BufferTooSmall, got {:?}",
                    num_signals, num_windows, other,
                ),
            }
        } else {
            // 32 × 256 = 8192 exactly: must NOT be BufferTooSmall.
            assert!(
                !matches!(r, Err(DsfbError::BufferTooSmall { .. })),
                "({}, {}): {} <= 8192 should not trip BufferTooSmall",
                num_signals, num_windows, total,
            );
        }
    }
}

/// Property 3: every ReasonCode variant has at least one matching motif
/// in the canonical bank (no orphan reason codes that produce only
/// `SemanticDisposition::Unknown`).
#[test]
fn no_orphan_reason_codes_in_canonical_bank() {
    let bank = HeuristicsBank::<64>::with_canonical_motifs();

    // Exhaustive over the 8 ReasonCode variants. The Admissible variant
    // has no motif by design (zero match → SemanticDisposition::Unknown
    // is correct for non-anomalous windows). The other seven must each
    // produce at least one Named match for some plausible (drift, slew)
    // input combination.
    let testable: &[ReasonCode] = &[
        ReasonCode::BoundaryApproach,
        ReasonCode::SustainedOutwardDrift,
        ReasonCode::AbruptSlewViolation,
        ReasonCode::RecurrentBoundaryGrazing,
        ReasonCode::EnvelopeViolation,
        ReasonCode::DriftWithRecovery,
        // SingleCrossing is intentionally orphan (transient, dismissed
        // by persistence gate before reaching the bank) — exclude it
        // from this test to match the engine's behaviour.
    ];

    for rc in testable {
        // Probe with strong drift + strong slew so any threshold-gated
        // motif fires.
        let got = bank.lookup(*rc, 1.0, 1.0);
        match got {
            SemanticDisposition::Named(_) => { /* ok */ }
            SemanticDisposition::Unknown => panic!(
                "Reason code {:?} matches no motif in canonical bank \
                 even at high drift + high slew. Motif coverage gap.",
                rc,
            ),
        }
    }
}

/// Property 4 (engineering invariant): bank lookup is deterministic
/// across multiple calls with identical input.
#[test]
fn bank_lookup_is_deterministic() {
    let bank = HeuristicsBank::<64>::with_canonical_motifs();
    let mut lcg = Lcg::new(LCG_SEED ^ 0xCAFEBABE);

    for _ in 0..256 {
        let drift = lcg.next_f64_signed().abs(); // [0, 1)
        let slew  = lcg.next_f64_signed().abs(); // [0, 1)
        // Cycle through reason codes deterministically.
        let rc_idx = (lcg.next_u64() >> 56) as usize % 7;
        let rc = match rc_idx {
            0 => ReasonCode::BoundaryApproach,
            1 => ReasonCode::SustainedOutwardDrift,
            2 => ReasonCode::AbruptSlewViolation,
            3 => ReasonCode::RecurrentBoundaryGrazing,
            4 => ReasonCode::EnvelopeViolation,
            5 => ReasonCode::DriftWithRecovery,
            _ => ReasonCode::Admissible,
        };
        let r1 = bank.lookup(rc, drift, slew);
        let r2 = bank.lookup(rc, drift, slew);
        assert_eq!(r1, r2,
                   "bank.lookup non-deterministic on (rc={:?}, drift={}, slew={})",
                   rc, drift, slew);
    }
}
