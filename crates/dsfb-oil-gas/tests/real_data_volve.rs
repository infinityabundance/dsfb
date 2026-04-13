/// real_data_volve.rs — Integration tests against real Equinor Volve drilling data.
///
/// Source: Equinor Volve Data Village, well 15/9-F-15 (WITSML 1.4.1 depth-indexed logs).
///         Equinor Volve Data Licence V1.0.
///         <https://data.equinor.com/dataset/Volve>
///
/// The CSV at data/drilling_real.csv was derived as follows:
///   - WITSML depth-indexed logs extracted from well 15/9-F-15 zip archive.
///   - Target channels: TQA (kNm), SWOB (kN), RPM (rpm), HKLD (kN), SPPA (kPa).
///   - SWOB and HKLD converted from source kkgf × 9.80665 → kN.
///   - Resampled to 0.5-m measured-depth steps over 1 200 – 4 065 m MD.
///   - 20-sample (10-m) rolling-median baseline per channel.

use dsfb_oil_gas::{
    load_volve_csv,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample, aggregate_episodes, noise_compression_ratio,
};

const VOLVE_CSV: &str = "data/drilling_real.csv";

/// CSV loads without error and has the expected row count.
#[test]
fn volve_csv_loads() {
    let frames = load_volve_csv(VOLVE_CSV)
        .expect("load_volve_csv should succeed for Volve drilling data");
    // Expect ~5 000 rows at 0.5-m spacing over ~1 800 m drill interval
    assert!(frames.len() >= 3_000,
        "expected ≥3 000 rows from Volve CSV, got {}", frames.len());
    assert!(frames.len() <= 8_000,
        "row count suspiciously high: {}", frames.len());
}

/// Depths must be monotone and within the expected F-15 drill interval.
#[test]
fn volve_depth_range_valid() {
    let frames = load_volve_csv(VOLVE_CSV).unwrap();
    assert!(!frames.is_empty());

    let mut prev_depth = f64::NEG_INFINITY;
    for (i, f) in frames.iter().enumerate() {
        assert!(f.depth_m.is_finite() && f.depth_m > 0.0,
            "non-positive depth at row {}: {}", i, f.depth_m);
        assert!(f.depth_m >= prev_depth - 1e-6,
            "depth not monotone at row {}: {} < {}", i, f.depth_m, prev_depth);
        assert!(f.depth_m >= 1_000.0 && f.depth_m <= 4_500.0,
            "depth outside expected F-15 range at row {}: {}", i, f.depth_m);
        prev_depth = f.depth_m;
    }
}

/// All TQA observations must be finite and in a plausible torque range.
#[test]
fn volve_torque_values_plausible() {
    let frames = load_volve_csv(VOLVE_CSV).unwrap();

    let mut n_valid = 0usize;
    for (i, f) in frames.iter().enumerate() {
        assert!(f.observed_tqa_knm.is_finite(),
            "non-finite TQA at row {}: {}", i, f.observed_tqa_knm);
        assert!(f.baseline_tqa_knm.is_finite(),
            "non-finite baseline TQA at row {}: {}", i, f.baseline_tqa_knm);
        // Physically plausible drilling torque: 0–120 kNm for this well size
        assert!(f.observed_tqa_knm >= 0.0 && f.observed_tqa_knm <= 120.0,
            "TQA out of physical range at row {}: {} kNm", i, f.observed_tqa_knm);
        if f.observed_tqa_knm > 0.5 { n_valid += 1; }
    }
    // At least 60 % of rows should have non-trivial torque (bit on bottom)
    let threshold = frames.len() * 60 / 100;
    assert!(n_valid >= threshold,
        "too few rows with TQA > 0.5 kNm: {} / {} (threshold {})",
        n_valid, frames.len(), threshold);
}

/// DSFB engine must produce no NaN/Inf on real Volve TQA residuals.
#[test]
fn dsfb_on_volve_no_nan() {
    let frames = load_volve_csv(VOLVE_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_volve_drilling();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_tqa_knm - f.baseline_tqa_knm;
        let sample = ResidualSample::new(f.depth_m, residual, 0.0, "tqa_knm");
        engine.ingest_sample(&sample);
    }

    for step in engine.history() {
        assert!(step.triple.r.is_finite(),
            "r is NaN/Inf at depth={}", step.triple.timestamp);
        assert!(step.triple.delta.is_finite(), "delta is NaN/Inf");
        assert!(step.triple.sigma.is_finite(), "sigma is NaN/Inf");
    }
}

/// Noise compression ratio ≥ 1 on Volve data.
#[test]
fn volve_dsfb_noise_compression() {
    let frames = load_volve_csv(VOLVE_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_volve_drilling();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_tqa_knm - f.baseline_tqa_knm;
        let sample = ResidualSample::new(f.depth_m, residual, 0.0, "tqa_knm");
        engine.ingest_sample(&sample);
    }

    let episodes = aggregate_episodes(engine.history());
    let ncr = noise_compression_ratio(frames.len(), episodes.len());
    assert!(ncr >= 1.0,
        "NCR < 1: {} depth-steps → {} episodes (NCR={:.2})",
        frames.len(), episodes.len(), ncr);
}

/// DSFB must produce a mix of grammar states including at least one non-Nominal state.
///
/// Real drilling has weight-on-bit transitions, RPM changes, and pipe connections
/// that produce torque excursions exceeding the 1σ envelope threshold.
#[test]
fn volve_dsfb_emits_non_nominal_states() {
    let frames = load_volve_csv(VOLVE_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_volve_drilling();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.observed_tqa_knm - f.baseline_tqa_knm;
        let sample = ResidualSample::new(f.depth_m, residual, 0.0, "tqa_knm");
        engine.ingest_sample(&sample);
    }

    let non_nominal = engine.history().iter()
        .filter(|s| s.state != GrammarState::Nominal)
        .count();

    assert!(non_nominal > 10,
        "DSFB produced fewer than 10 non-Nominal states on Volve data — \
         envelope may be too wide (got {})", non_nominal);
}
