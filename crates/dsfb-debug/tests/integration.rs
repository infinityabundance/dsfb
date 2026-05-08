// DSFB-Debug: Integration tests
//
// Policy (per crate authoring rules): NO synthetic signal generators.
// Tests in this file exercise pure functions on hand-built `&[f64]`
// literals (constant or single-shape inputs that exercise specific
// invariants). End-to-end runs against real upstream trace bytes live
// in `tests/eval_tadbench.rs` / `tests/eval_illinois.rs`, gated behind
// the `std` + `paper-lock` features.

#[cfg(test)]
mod tests {
    use dsfb_debug::*;
    use dsfb_debug::types::*;
    use dsfb_debug::config::PAPER_LOCK_CONFIG;

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

    #[test]
    fn test_observer_only_contract() {
        let engine = DsfbDebugEngine::<256, 64>::paper_lock()
            .expect("engine creation should succeed");

        let data: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let labels: [bool; 2] = [false, true];

        let mut evals = [blank_eval(); 8];
        let mut episodes = [blank_episode(); 4];

        // The following would NOT compile — proves the contract:
        // engine.modify_upstream(&mut data);  // No such method exists

        let _ = engine.run_evaluation(
            &data, 4, 2, &labels, 1,
            &mut evals, &mut episodes, "contract_test",
        );

        // Original data unchanged — but we don't even need to check because
        // the type system made mutation impossible.
        assert_eq!(data[0], 1.0);
    }

    #[test]
    fn test_missingness_produces_zero_drift() {
        let engine = DsfbDebugEngine::<256, 64>::paper_lock()
            .expect("engine creation should succeed");

        let norms = [0.0_f64; 10];
        let recent = [GrammarState::Admissible; 2];

        let eval = engine.evaluate_signal(
            &norms, 5, 3.0, 0, 5, true, &recent, 0,
        );

        assert_eq!(eval.sign_tuple.drift, 0.0);
        assert_eq!(eval.sign_tuple.slew, 0.0);
        assert_eq!(eval.confirmed_grammar_state, GrammarState::Admissible);
        assert!(eval.was_imputed);
    }

    #[test]
    fn test_violation_produces_escalate() {
        let engine = DsfbDebugEngine::<256, 64>::paper_lock()
            .expect("engine creation should succeed");

        let norms = [0.0, 0.5, 1.0, 2.0, 5.0, 10.0];
        let recent = [GrammarState::Boundary, GrammarState::Violation];

        let eval = engine.evaluate_signal(
            &norms, 5, 3.0, 0, 5, false, &recent, 4,
        );

        assert_eq!(eval.confirmed_grammar_state, GrammarState::Violation);
        assert_eq!(eval.policy_state, PolicyState::Escalate);
    }

    #[test]
    fn test_config_validation() {
        let mut config = PAPER_LOCK_CONFIG;
        config.drift_window = 0;
        assert!(DsfbDebugEngine::<256, 64>::new(config).is_err());
    }

    #[test]
    fn test_clean_constant_produces_no_episodes() {
        // All-clean (constant) data is an invariant test: zero variance →
        // baseline envelope at floor → no signals can drift outward → zero
        // episodes. Constant input is not a synthetic *generator*; it is a
        // single literal value tested against an algebraic invariant.
        let engine = DsfbDebugEngine::<256, 64>::paper_lock()
            .expect("engine creation should succeed");

        let data = [50.0_f64; 200]; // 100 windows × 2 signals
        let labels = [false; 100];

        let mut evals = [blank_eval(); 200];
        let mut episodes = [blank_episode(); 16];

        let (ep_count, metrics) = engine.run_evaluation(
            &data, 2, 100, &labels, 50,
            &mut evals, &mut episodes, "constant_invariant_test",
        ).expect("should succeed");

        assert_eq!(ep_count, 0, "constant data must produce zero episodes");
        assert_eq!(metrics.dsfb_episode_count, 0);
    }

    #[test]
    fn test_buffer_too_small_returns_error() {
        // Theorem 9 guard: refuse rather than silently truncate when the
        // requested num_signals * num_windows exceeds the internal flat
        // aggregation cap (8192). Replaces the prior silent-truncation
        // behaviour at lib.rs:289-292.
        let engine = DsfbDebugEngine::<256, 64>::paper_lock()
            .expect("engine creation should succeed");

        // 16 signals * 1024 windows = 16384 > 8192
        let data = [0.0_f64; 16 * 1024];
        let labels = [false; 1024];
        let mut evals = [blank_eval(); 1];
        let mut episodes = [blank_episode(); 1];

        let r = engine.run_evaluation(
            &data, 16, 1024, &labels, 100,
            &mut evals, &mut episodes, "overflow_test",
        );
        assert!(matches!(r, Err(error::DsfbError::BufferTooSmall { .. })),
                "overflow must surface BufferTooSmall, not silent truncation");
    }
}
