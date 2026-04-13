//! metrics_esp — Print real DSFB metrics for RPDBCS ESPset rotating equipment data.
//!
//! Source: RPDBCS ESPset — 11 ESP units, 6 032 vibration snapshots.
//!         Primary channel: broadband vibration RMS at 98–102 Hz.
//!         MIT License.
//!
//! Run with:
//!   cargo run --example metrics_esp

use std::collections::HashMap;
use dsfb_oil_gas::{
    load_esp_csv,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample, aggregate_episodes, noise_compression_ratio,
};

fn main() {
    let frames = load_esp_csv("data/rotating_real.csv")
        .expect("failed to load rotating_real.csv");

    let env = AdmissibilityEnvelope::default_esp_rotating();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.rms_broadband - f.baseline_rms;
        let sample = ResidualSample::new(f.step as f64, residual, 0.0, "esp_rms");
        engine.ingest_sample(&sample);
    }

    let history = engine.history();
    let episodes = aggregate_episodes(history);

    let k = frames.len();
    let n_ep = episodes.len();
    let ncr = noise_compression_ratio(k, n_ep);

    // State distribution per episode
    let mut ep_state_counts: HashMap<GrammarState, usize> = HashMap::new();
    for ep in &episodes {
        *ep_state_counts.entry(ep.state).or_insert(0) += 1;
    }

    // Step distribution
    let mut step_state_counts: HashMap<GrammarState, usize> = HashMap::new();
    for s in history {
        *step_state_counts.entry(s.state).or_insert(0) += 1;
    }

    // Residual stats
    let residuals: Vec<f64> = frames.iter()
        .map(|f| f.rms_broadband - f.baseline_rms)
        .collect();
    let mean_r = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let var_r = residuals.iter().map(|&r| (r - mean_r).powi(2)).sum::<f64>()
        / residuals.len() as f64;
    let sigma_r = var_r.sqrt();

    let non_nom = history.iter()
        .filter(|s| s.state != GrammarState::Nominal)
        .count();

    println!("=== RPDBCS ESPset DSFB Metrics (rms_broadband, real vibration data) ===");
    println!("Units: 11 ESP pumps  |  Samples: {}", k);
    println!("rms_broadband residual: μ = {:.5},  σ = {:.5}", mean_r, sigma_r);
    println!();
    println!("Grammar episodes: {}", n_ep);
    println!("Noise compression ratio (NCR): {:.1}", ncr);
    println!("Non-Nominal steps: {} ({:.1} %)", non_nom, 100.0 * non_nom as f64 / k as f64);
    println!();

    println!("Step distribution by grammar state:");
    let mut sv: Vec<_> = step_state_counts.iter().collect();
    sv.sort_by_key(|(s, _)| format!("{:?}", s));
    for (state, cnt) in &sv {
        println!("  {:20?}  {:5} steps  ({:.1} %)",
            state, cnt, 100.0 * **cnt as f64 / k as f64);
    }

    println!();
    println!("Episode distribution by dominant state:");
    let mut ev: Vec<_> = ep_state_counts.iter().collect();
    ev.sort_by_key(|(s, _)| format!("{:?}", s));
    for (state, cnt) in &ev {
        println!("  {:20?}  {:4} episodes  ({:.1} %)",
            state, cnt, 100.0 * **cnt as f64 / n_ep as f64);
    }
}
