//! Concurrency exploration harness for the DSFB observer.
//!
//! Rationale: the `DsfbRfEngine` is owned by a single observer thread per
//! residual stream, and all typed state updates happen behind `&mut self`.
//! Still, the `paper-lock` toolchain hands engine state across threads in
//! its replay harness (see `src/pipeline.rs`), so we want an explicit
//! exploration that `Send`/`Sync` bounds are respected on the public
//! surface and that the shared calibration snapshot is race-free.
//!
//! This harness uses `loom` to model the calibration-snapshot publish path.
//! Build with:
//!
//!     RUSTFLAGS="--cfg loom" cargo test --test concurrency_observer
//!
//! When `loom` is not selected, the file still compiles as a compile-time
//! trait-bound check over the observer types.

#![cfg(feature = "std")]

#[cfg(not(loom))]
mod compile_time_bounds {
    use dsfb_rf::envelope::AdmissibilityEnvelope;
    use dsfb_rf::grammar::GrammarEvaluator;
    use dsfb_rf::sign::SignWindow;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn public_engine_types_are_send_and_sync() {
        assert_send::<AdmissibilityEnvelope>();
        assert_sync::<AdmissibilityEnvelope>();
        assert_send::<SignWindow<8>>();
        assert_sync::<SignWindow<8>>();
        assert_send::<GrammarEvaluator<4>>();
        assert_sync::<GrammarEvaluator<4>>();
    }
}

#[cfg(loom)]
mod loom_explore {
    use dsfb_rf::envelope::AdmissibilityEnvelope;
    use loom::sync::Arc;
    use loom::thread;

    /// Loom-model: a calibration snapshot is produced once and shared by
    /// N reader threads. The publisher never mutates after handing over
    /// the `Arc`; every reader observes the same `rho`.
    #[test]
    fn envelope_snapshot_publish_is_race_free() {
        loom::model(|| {
            let env = Arc::new(
                AdmissibilityEnvelope::calibrate_from_window(&[0.05f32; 32])
                    .expect("calibrates"),
            );
            let env_a = env.clone();
            let env_b = env.clone();
            let a = thread::spawn(move || env_a.rho);
            let b = thread::spawn(move || env_b.rho);
            let ra = a.join().unwrap();
            let rb = b.join().unwrap();
            assert_eq!(ra, rb);
        });
    }
}
