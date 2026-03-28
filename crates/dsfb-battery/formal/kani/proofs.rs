#[cfg(kani)]
mod proofs {
    use core::mem::discriminant;

    use crate::detection::{evaluate_grammar_state, next_persistence_count};
    use crate::types::{EnvelopeParams, GrammarState, PipelineConfig};

    fn bounded_f64(seed: i16, scale: f64) -> f64 {
        seed as f64 * scale
    }

    #[kani::proof]
    fn persistence_counter_resets_or_increments() {
        let current: usize = kani::any();
        let condition_met: bool = kani::any();
        kani::assume(current < 16);

        let next = next_persistence_count(current, condition_met);
        if condition_met {
            assert!(next == current + 1);
        } else {
            assert!(next == 0);
        }
    }

    #[kani::proof]
    fn grammar_state_always_declared_variant() {
        let residual_seed: i16 = kani::any();
        let drift_seed: i16 = kani::any();
        let slew_seed: i16 = kani::any();
        let drift_counter: usize = kani::any();
        let slew_counter: usize = kani::any();
        kani::assume(drift_counter < 16);
        kani::assume(slew_counter < 16);

        let envelope = EnvelopeParams {
            mu: 0.0,
            sigma: 0.01,
            rho: 0.03,
        };
        let config = PipelineConfig {
            drift_persistence: 3,
            slew_persistence: 3,
            ..PipelineConfig::default()
        };
        let state = evaluate_grammar_state(
            bounded_f64(residual_seed, 0.001),
            &envelope,
            bounded_f64(drift_seed, 0.001),
            bounded_f64(slew_seed, 0.001),
            drift_counter,
            slew_counter,
            &config,
        );

        assert!(matches!(
            state,
            GrammarState::Admissible | GrammarState::Boundary | GrammarState::Violation
        ));
    }

    #[kani::proof]
    fn grammar_state_deterministic_for_same_inputs() {
        let residual_seed: i16 = kani::any();
        let drift_seed: i16 = kani::any();
        let slew_seed: i16 = kani::any();
        let drift_counter: usize = kani::any();
        let slew_counter: usize = kani::any();
        kani::assume(drift_counter < 8);
        kani::assume(slew_counter < 8);

        let envelope = EnvelopeParams {
            mu: 0.0,
            sigma: 0.02,
            rho: 0.06,
        };
        let config = PipelineConfig::default();
        let residual = bounded_f64(residual_seed, 0.001);
        let drift = bounded_f64(drift_seed, 0.001);
        let slew = bounded_f64(slew_seed, 0.001);

        let a = evaluate_grammar_state(
            residual,
            &envelope,
            drift,
            slew,
            drift_counter,
            slew_counter,
            &config,
        );
        let b = evaluate_grammar_state(
            residual,
            &envelope,
            drift,
            slew,
            drift_counter,
            slew_counter,
            &config,
        );

        assert!(discriminant(&a) == discriminant(&b));
    }
}
