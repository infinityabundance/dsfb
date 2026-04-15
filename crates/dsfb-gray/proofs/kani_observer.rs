#[cfg(kani)]
mod proofs {
    use dsfb_gray::{AdmissibilityEnvelope, ObserverConfig, WorkloadPhase};

    #[kani::proof]
    fn fast_response_window_is_never_zero() {
        let config = ObserverConfig {
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.1,
                0.05,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };

        assert!(config.persistence_window > 0);
        assert!(config.hysteresis_count > 0);
    }
}
