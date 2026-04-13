/// real_data_3w.rs — Integration tests against real Petrobras 3W sensor data.
///
/// Source: Petrobras 3W Dataset v2.0.0, CC BY 4.0
///         <https://github.com/petrobras/3W>
///
/// Only real WELL-* instances are used; SIMULATED_* and DRAWN_* are excluded.
/// The CSV at data/oilwell_real.csv was derived as follows:
///   - Resampled to 60-s medians.
///   - 30-min rolling-median baseline per channel.
///   - 12 raw instances in the bundled CSV, but one DHSV-closure instance has
///     no usable choke-populated rows after filtering, so load_oilwell_csv()
///     yields 11 processed choke episodes / 9,087 rows.

use dsfb_oil_gas::{
    load_oilwell_csv,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample, aggregate_episodes, noise_compression_ratio,
};

const REAL_CSV: &str = "data/oilwell_real.csv";

/// Confirm the real CSV loads without error and has a plausible row count.
#[test]
fn oilwell_real_csv_loads() {
    let frames = load_oilwell_csv(REAL_CSV)
        .expect("load_oilwell_csv should succeed for real 3W data");
    // 11 episodes × avg ~800 rows (only choke-populated rows kept)
    assert!(frames.len() > 1_000,
        "expected >1000 rows from real CSV, got {}", frames.len());
    assert!(frames.len() <= 9_500,
        "row count suspiciously high: {}", frames.len());
}

/// All timestamps must be non-negative and finite.
#[test]
fn oilwell_timestamps_valid() {
    let frames = load_oilwell_csv(REAL_CSV).unwrap();
    for (i, f) in frames.iter().enumerate() {
        assert!(f.timestamp.is_finite() && f.timestamp >= 0.0,
            "bad timestamp at row {}: {}", i, f.timestamp);
    }
}

/// All pressure observations must be positive (subsea pressures are >>0 Pa).
#[test]
fn oilwell_choke_pressures_positive() {
    let frames = load_oilwell_csv(REAL_CSV).unwrap();
    let mut zero_count = 0usize;
    for f in &frames {
        if f.observed_choke_pa <= 0.0 { zero_count += 1; }
    }
    // Allow up to 5 % zero-padded rows (loader maps NaN → 0.0)
    let threshold = frames.len() / 20;
    assert!(zero_count <= threshold,
        "too many zero choke-pressure rows: {} / {}", zero_count, frames.len());
}

/// DSFB engine must produce no NaN/Inf on real 3W residuals.
#[test]
fn dsfb_on_real_data_no_nan() {
    let frames = load_oilwell_csv(REAL_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_oilwell();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_choke_pa - f.expected_choke_pa;
        let sample = ResidualSample::new(f.timestamp, residual, 0.0, "choke_pa");
        engine.ingest_sample(&sample);
    }

    for step in engine.history() {
        assert!(step.triple.r.is_finite(),     "r is NaN/Inf at t={}", step.triple.r);
        assert!(step.triple.delta.is_finite(), "delta is NaN/Inf");
        assert!(step.triple.sigma.is_finite(), "sigma is NaN/Inf");
    }
}

/// Noise compression ratio ≥ 1 on real data (episodes ≤ samples processed).
#[test]
fn dsfb_real_data_noise_compression() {
    let frames = load_oilwell_csv(REAL_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_oilwell();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_choke_pa - f.expected_choke_pa;
        let sample = ResidualSample::new(f.timestamp, residual, 0.0, "choke_pa");
        engine.ingest_sample(&sample);
    }

    let episodes = aggregate_episodes(engine.history());
    let ncr = noise_compression_ratio(frames.len(), episodes.len());
    assert!(ncr >= 1.0,
        "NCR < 1 on real data: {} samples -> {} episodes (NCR={:.2})",
        frames.len(), episodes.len(), ncr);
}

/// DSFB must emit at least one non-Nominal state on the real event episodes
/// (flow instability and hydrate events have large pressure excursions).
#[test]
fn dsfb_real_data_detects_events() {
    let frames = load_oilwell_csv(REAL_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_oilwell();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_choke_pa - f.expected_choke_pa;
        let sample = ResidualSample::new(f.timestamp, residual, 0.0, "choke_pa");
        engine.ingest_sample(&sample);
    }

    let non_nominal = engine.history().iter()
        .filter(|s| s.state != GrammarState::Nominal)
        .count();

    assert!(non_nominal > 0,
        "DSFB produced only Nominal states on real event data — \
         envelope may be too wide or residuals not reaching threshold");
}
