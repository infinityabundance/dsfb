//! metrics_volve — Print real DSFB metrics for Equinor Volve 15/9-F-15 drilling data.
//!
//! Source: Equinor Volve Data Village, WITSML 1.4.1 depth-indexed logs.
//!         Equinor Volve Data Licence V1.0.
//!         Primary channel: TQA surface torque [kNm], depth-indexed, 0.5-m steps.
//!
//! Run with:
//!   cargo run --example metrics_volve

use dsfb_oil_gas::{
    load_volve_csv,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample, aggregate_episodes, noise_compression_ratio,
};
use std::collections::HashMap;

fn main() {
    let path = "data/drilling_real.csv";
    let frames = load_volve_csv(path).expect("failed to load drilling_real.csv");

    let env = AdmissibilityEnvelope::default_volve_drilling();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_tqa_knm - f.baseline_tqa_knm;
        let sample = ResidualSample::new(f.depth_m, residual, 0.0, "tqa_knm");
        engine.ingest_sample(&sample);
    }

    let history = engine.history();
    let episodes = aggregate_episodes(history);

    let k = frames.len();
    let n_ep = episodes.len();
    let ncr = noise_compression_ratio(k, n_ep);

    // Episode count per grammar state
    let mut state_counts: HashMap<GrammarState, usize> = HashMap::new();
    for ep in &episodes {
        *state_counts.entry(ep.state).or_insert(0usize) += 1;
    }

    // Depth coverage
    let d_min = frames.first().map(|f| f.depth_m).unwrap_or(0.0);
    let d_max = frames.last().map(|f| f.depth_m).unwrap_or(0.0);

    // TQA residual stats
    let residuals: Vec<f64> = frames.iter()
        .map(|f| f.observed_tqa_knm - f.baseline_tqa_knm)
        .collect();
    let mean_r: f64 = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let var_r: f64 = residuals.iter().map(|&r| (r - mean_r).powi(2)).sum::<f64>()
        / residuals.len() as f64;
    let sigma_r = var_r.sqrt();

    // Non-nominal fraction
    let non_nom = history.iter()
        .filter(|s| s.state != GrammarState::Nominal)
        .count();
    let event_frac = 100.0 * non_nom as f64 / k as f64;

    println!("=== Volve 15/9-F-15 DSFB Metrics (TQA, real WITSML data) ===");
    println!("Depth: {:.1} – {:.1} m MD  |  Step: 0.5 m", d_min, d_max);
    println!("K = {}  depth-steps processed", k);
    println!("TQA residual: μ = {:.3} kNm,  σ = {:.3} kNm", mean_r, sigma_r);
    println!();
    println!("Grammar episodes: {}", n_ep);
    println!("Noise compression ratio (NCR): {:.1}", ncr);
    println!("Non-Nominal depth-steps: {} ({:.1} %)", non_nom, event_frac);
    println!();
    println!("Episode distribution by dominant state:");
    let mut sorted_states: Vec<_> = state_counts.iter().collect();
    sorted_states.sort_by_key(|(s, _)| format!("{:?}", s));
    for (state, count) in &sorted_states {
        let frac = 100.0 * **count as f64 / n_ep as f64;
        println!("  {:20?}  {:4}  ({:.1} %)", state, count, frac);
    }
}
