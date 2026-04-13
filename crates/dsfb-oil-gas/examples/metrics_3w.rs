/// metrics_3w.rs — Compute DSFB metrics on the real Petrobras 3W dataset.
///
/// Prints ECC, NCR, EDR, episode table, and grammar token distribution for the
/// paper's real-data empirical results table.
use dsfb_oil_gas::{
    aggregate_episodes, load_oilwell_csv, noise_compression_ratio,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frames = load_oilwell_csv("data/oilwell_real.csv")?;
    let total_steps = frames.len();

    let env = AdmissibilityEnvelope::default_oilwell();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_choke_pa - f.expected_choke_pa;
        let sample = ResidualSample::new(f.timestamp, residual, 0.0, "choke_pa");
        engine.ingest_sample(&sample);
    }

    let history = engine.history();
    let episodes = aggregate_episodes(history);
    let episode_count = episodes.len();
    let ncr = noise_compression_ratio(total_steps, episode_count);
    let ecc = total_steps as f64 / episode_count as f64;

    let nominal_steps = history.iter().filter(|s| s.state == GrammarState::Nominal).count();
    let edr = nominal_steps as f64 / total_steps as f64;

    // Token distribution
    let mut token_counts: HashMap<String, usize> = HashMap::new();
    for s in history {
        *token_counts.entry(format!("{:?}", s.state)).or_insert(0) += 1;
    }

    println!("========== DSFB REAL-DATA METRICS — Petrobras 3W ==========");
    println!("Dataset : oilwell_real.csv (12 raw instances; 11 choke-populated after filtering)");
    println!("Channel : P-MON-CKP (choke pressure, Pa)");
    println!("Baseline: 30-min rolling median; 60-s resampling");
    println!();
    println!("Timesteps (K)   : {}", total_steps);
    println!("Grammar episodes: {}", episode_count);
    println!("ECC             : {:.2}", ecc);
    println!("NCR             : {:.2}", ncr);
    println!("EDR             : {:.4}  ({:.1}% nominal)", edr, 100.0 * edr);
    println!();
    println!("--- Grammar token distribution ---");
    let mut token_vec: Vec<_> = token_counts.iter().collect();
    token_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (tok, cnt) in &token_vec {
        println!(
            "  {:20} {:>6} steps  ({:.1}%)",
            tok,
            cnt,
            100.0 * (**cnt) as f64 / total_steps as f64
        );
    }

    println!();
    println!("--- Episode table (first 20) ---");
    println!("{:>5}  {:>10}  {:>10}  {:50}", "Ep#", "Start-ts", "Steps", "State");
    for (i, ep) in episodes.iter().take(20).enumerate() {
        println!(
            "{:>5}  {:>10.0}  {:>10}  {:50}",
            i + 1,
            ep.start_ts,
            ep.step_count,
            format!("{:?}", ep.state)
        );
    }
    if episodes.len() > 20 {
        println!("  ... ({} more episodes)", episodes.len() - 20);
    }

    Ok(())
}
