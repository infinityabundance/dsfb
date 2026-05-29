//! Behavioural (empirical-property) tests — Wave-1 B1.
//!
//! These are deliberately **not** seal-checks. The golden gates (`golden_replay.rs`, `court_record.rs`,
//! `atlas_gates.rs`) and the per-object `verify()` methods prove *determinism + integrity* (a hash equals a
//! frozen hash); they would still pass if the pipeline computed something useless, as long as it did so
//! reproducibly. The panel-v2 review flagged exactly this: most tests were tautological. These tests instead
//! assert that DSFB **does the thing** — on the deterministic synthetic suite (a clear labelled fault at
//! sample 400 over a 300-sample clean baseline, six fault archetypes), every fault is actually *detected*
//! within its fault window, the clean baseline is *not* mostly false positives, and the residual structure
//! genuinely *compresses*. The bounds are loose enough to never flake on the seeded data but tight enough to
//! fail if detection, baseline behaviour, or compression regressed.

use dsfb_chemical_engineering_edge::cli::synthetic_suite;
use dsfb_chemical_engineering_edge::{analyze, PipelineConfig};

/// Every labelled synthetic fault must be DETECTED, and detected within its fault window (onset = 400,
/// window 400..600) — not "never" (`i64::MAX`) and not absurdly early in the baseline.
#[test]
fn every_synthetic_fault_is_detected_within_its_window() {
    let cfg = PipelineConfig::default();
    for d in synthetic_suite() {
        let res = analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let delay = res
            .metrics
            .detection_delay
            .expect("synthetic datasets carry a labelled onset");
        assert_ne!(
            delay,
            i64::MAX,
            "{}: the labelled fault was never detected",
            d.name
        );
        // A genuine detection lands at/after the onset and inside the 200-sample fault window
        // (a small negative tolerance allows detection on the last pre-onset sample).
        assert!(
            (-5..=200).contains(&delay),
            "{}: detection delay {} samples is outside the fault window (regression?)",
            d.name,
            delay
        );
        assert!(
            res.metrics.replay_deterministic,
            "{}: must replay deterministically",
            d.name
        );
    }
}

/// The clean 300-sample baseline must not be dominated by false positives, and the pipeline must
/// genuinely compress (fused episodes never exceed the raw detector breaches that produced them).
#[test]
fn synthetic_baseline_is_clean_and_residual_structure_compresses() {
    let cfg = PipelineConfig::default();
    for d in synthetic_suite() {
        let res = analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let m = &res.metrics;
        assert!(
            (0.0..=1.0).contains(&m.unknown_rate),
            "{}: unknown_rate {} out of [0,1]",
            d.name,
            m.unknown_rate
        );
        assert!(
            m.baseline_false_positive_rate <= 0.5,
            "{}: baseline false-positive rate {:.2} is too high for a clean synthetic baseline",
            d.name,
            m.baseline_false_positive_rate
        );
        assert!(
            m.episode_compression_ratio >= 1.0,
            "{}: compression ratio {:.2} < 1 — fused episodes should never exceed raw breaches",
            d.name,
            m.episode_compression_ratio
        );
        assert!(
            m.fused_episodes >= 1,
            "{}: a clear injected fault should yield ≥1 fused episode",
            d.name
        );
    }
}

/// Determinism as a *behavioural* property (not a frozen-hash check): two independent analyses of the same
/// dataset must agree on the headline metrics, not merely on the replay hash.
#[test]
fn analysis_metrics_are_run_to_run_stable() {
    let cfg = PipelineConfig::default();
    for d in synthetic_suite() {
        let a = analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let b = analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        assert_eq!(
            a.metrics.fused_episodes, b.metrics.fused_episodes,
            "{}: episode count not stable",
            d.name
        );
        assert_eq!(
            a.metrics.detection_delay, b.metrics.detection_delay,
            "{}: detection delay not stable",
            d.name
        );
        assert_eq!(
            a.replay_hash, b.replay_hash,
            "{}: replay hash not stable",
            d.name
        );
    }
}
