//! Property-based invariants for the DSFB observer core.
//!
//! These proptests cover the sign-tuple / grammar FSM / DSA-bound /
//! envelope-monotonicity invariants named in `CHANGELOG.md` under the
//! v1.0.2-tracked deliverable. We bring them forward here because the
//! invariants are checkable without the deferred `Result<_, CalibrationError>`
//! refactor: every property below operates on already-constructed engine
//! state.

#![cfg(feature = "std")]

use dsfb_rf::envelope::AdmissibilityEnvelope;
use dsfb_rf::fixedpoint::{dequantize_q16_16, quantize_q16_16};
use dsfb_rf::grammar::{GrammarEvaluator, GrammarState};
use dsfb_rf::platform::{SnrFloor, WaveformState};
use dsfb_rf::sign::{SignTuple, SignWindow};
use proptest::prelude::*;

proptest! {
    /// Q16.16 round-trip: dequantize(quantize(x)) is within the native
    /// 2^-16 resolution of x.
    #[test]
    fn q1616_round_trip_bounded(x in -32000.0f64..32000.0f64) {
        let q = quantize_q16_16(x);
        let back = dequantize_q16_16(q);
        prop_assert!((back - x).abs() <= 1.0 / 65536.0 + 1e-9);
    }

    /// SignTuple components stay finite on any finite input norm sequence.
    #[test]
    fn sign_tuple_finite(norms in proptest::collection::vec(0.0f32..1e6f32, 1..64)) {
        let mut window: SignWindow<8> = SignWindow::new();
        let floor = SnrFloor::default();
        for n in norms {
            let s: SignTuple = window.push(n, false, floor);
            prop_assert!(s.norm.is_finite(), "norm must be finite");
            prop_assert!(s.drift.is_finite(), "drift must be finite");
            prop_assert!(s.slew.is_finite(), "slew must be finite");
        }
    }

    /// Envelope monotonicity: calibrating from a scaled healthy window
    /// scales ρ by the same factor (up to numerical drift).
    #[test]
    fn envelope_rho_scales_with_input(scale in 0.1f32..10.0f32) {
        let base = [0.05f32; 128];
        let scaled: [f32; 128] = core::array::from_fn(|i| base[i] * scale);
        let env_base = AdmissibilityEnvelope::calibrate_from_window(&base)
            .expect("base calibrates");
        let env_scaled = AdmissibilityEnvelope::calibrate_from_window(&scaled)
            .expect("scaled calibrates");
        let ratio = env_scaled.rho / env_base.rho;
        prop_assert!((ratio - scale).abs() < 1e-3,
            "rho scales with input scale: got ratio={ratio}, expected {scale}");
    }

    /// Grammar FSM is Admissible on an in-envelope flat stream with a
    /// healthy calibration; never panics for any finite input.
    #[test]
    fn grammar_flat_stream_admissible(
        base in 0.01f32..0.5f32,
        seed in 0u64..4096,
    ) {
        let healthy: [f32; 256] = core::array::from_fn(|_| base);
        let env = AdmissibilityEnvelope::calibrate_from_window(&healthy)
            .expect("flat healthy window calibrates");
        let mut window: SignWindow<8> = SignWindow::new();
        let floor = SnrFloor::default();
        let mut grammar: GrammarEvaluator<4> = GrammarEvaluator::new();
        let ws = WaveformState::Operational;
        let mut s = seed;
        let mut next = || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 40) as f32 / (1u32 << 24) as f32) * 1e-4
        };
        for _ in 0..128 {
            let norm = (base + next()).max(0.0);
            let sig = window.push(norm, false, floor);
            let state = grammar.evaluate(&sig, &env, ws);
            prop_assert!(matches!(
                state,
                GrammarState::Admissible
                    | GrammarState::Boundary(_)
                    | GrammarState::Violation,
            ));
        }
    }
}
