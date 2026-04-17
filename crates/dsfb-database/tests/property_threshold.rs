//! Threshold-elasticity property test.
//!
//! Reviewer attack #4 from `paperstack/limitations.txt`: "your numbers are
//! a single threshold tune away from collapsing." This test pins the
//! property that a ±20 % perturbation of every motif's drift and slew
//! threshold leaves the per-motif F1 within an acceptable band on the
//! perturbation harness — i.e., the grammar is *not* fragilely tuned.

use dsfb_database::grammar::{MotifEngine, MotifGrammar};
use dsfb_database::metrics::evaluate;
use dsfb_database::perturbation::tpcds_with_perturbations;
use std::collections::HashMap;

fn scale_grammar(g: &MotifGrammar, factor: f64) -> MotifGrammar {
    let mut g = g.clone();
    for p in [
        &mut g.plan_regression_onset,
        &mut g.cardinality_mismatch_regime,
        &mut g.contention_ramp,
        &mut g.cache_collapse,
        &mut g.workload_phase_transition,
    ] {
        p.drift_threshold *= factor;
        p.slew_threshold *= factor;
    }
    g
}

#[test]
fn f1_survives_plus_minus_twenty_percent_threshold_perturbation() {
    let (stream, windows) = tpcds_with_perturbations(42);
    let baseline = MotifGrammar::default();
    let mut samples: HashMap<dsfb_database::grammar::MotifClass, usize> = HashMap::new();
    for m in dsfb_database::grammar::MotifClass::ALL {
        samples.insert(m, stream.iter_class(m.residual_class()).count());
    }
    let dur = stream.duration();

    let baseline_eps = MotifEngine::new(baseline.clone()).run(&stream);
    let baseline_metrics = evaluate(&baseline_eps, &windows, &samples, dur);

    for factor in [0.80, 1.20] {
        let scaled = scale_grammar(&baseline, factor);
        let scaled_eps = MotifEngine::new(scaled).run(&stream);
        let scaled_metrics = evaluate(&scaled_eps, &windows, &samples, dur);
        for (b, s) in baseline_metrics.iter().zip(scaled_metrics.iter()) {
            // Each motif must keep F1 within 0.5 of baseline at +-20 %.
            // The published baseline is F1=1.0 across all motifs; this
            // band gives operators room to retune for their workload
            // without falling off a cliff.
            assert!(
                (b.f1 - s.f1).abs() <= 0.5,
                "{} F1 collapsed under factor {}: baseline {} -> scaled {}",
                b.motif,
                factor,
                b.f1,
                s.f1
            );
        }
    }
}
