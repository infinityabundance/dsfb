/// real_data_esp.rs — Integration tests against real RPDBCS ESPset vibration data.
///
/// Source: RPDBCS ESPset (Real-world Pump Bearing Dataset with Classification
///         Support).  MIT License.
///         11 ESP units; 6 032 labeled vibration snapshots;
///         5 operating conditions: Normal, Unbalance, Rubbing, Misalignment,
///         Faulty sensor.
///
/// The CSV at data/rotating_real.csv was derived as follows:
///   - features.csv extracted from RPDBCS ESPset zip.
///   - Semicolon-delimited columns parsed; rows grouped per esp_id.
///   - 15-sample causal rolling-median baseline computed per ESP unit.
///   - Channels: rms_broadband, peak1x, peak2x, median_8_13hz, coeff_a, coeff_b.

use dsfb_oil_gas::{
    load_esp_csv,
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample, aggregate_episodes, noise_compression_ratio,
};

const ESP_CSV: &str = "data/rotating_real.csv";

/// CSV loads without error and has the expected row count.
#[test]
fn esp_csv_loads() {
    let frames = load_esp_csv(ESP_CSV)
        .expect("load_esp_csv should succeed for RPDBCS ESPset data");
    // 6 032 rows across 11 ESP units
    assert!(frames.len() >= 5_500,
        "expected ≥5 500 rows from ESP CSV, got {}", frames.len());
    assert!(frames.len() <= 7_000,
        "row count suspiciously high: {}", frames.len());
}

/// All esp_id values must be in 0–10.
#[test]
fn esp_ids_valid() {
    let frames = load_esp_csv(ESP_CSV).unwrap();
    for (i, f) in frames.iter().enumerate() {
        assert!(f.esp_id <= 10,
            "esp_id out of range at row {}: {}", i, f.esp_id);
    }
    // Confirm multiple units are present
    let unique_ids: std::collections::HashSet<u8> = frames.iter().map(|f| f.esp_id).collect();
    assert!(unique_ids.len() >= 5,
        "expected ≥5 distinct ESP units, got {}", unique_ids.len());
}

/// All RMS values must be finite and non-negative.
#[test]
fn esp_rms_values_valid() {
    let frames = load_esp_csv(ESP_CSV).unwrap();
    for (i, f) in frames.iter().enumerate() {
        assert!(f.rms_broadband.is_finite(),
            "non-finite rms_broadband at row {}: {}", i, f.rms_broadband);
        assert!(f.rms_broadband >= 0.0,
            "negative rms_broadband at row {}: {}", i, f.rms_broadband);
        assert!(f.baseline_rms.is_finite(),
            "non-finite baseline_rms at row {}: {}", i, f.baseline_rms);
    }
}

/// DSFB engine must produce no NaN/Inf on real ESP RMS residuals.
#[test]
fn dsfb_on_esp_no_nan() {
    let frames = load_esp_csv(ESP_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_esp_rotating();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.rms_broadband - f.baseline_rms;
        let sample = ResidualSample::new(f.step as f64, residual, 0.0, "esp_rms");
        engine.ingest_sample(&sample);
    }

    for step in engine.history() {
        assert!(step.triple.r.is_finite(),
            "r is NaN/Inf at step={}", step.triple.timestamp);
        assert!(step.triple.delta.is_finite(), "delta is NaN/Inf");
        assert!(step.triple.sigma.is_finite(), "sigma is NaN/Inf");
    }
}

/// Noise compression ratio ≥ 1 on ESP data.
#[test]
fn esp_dsfb_noise_compression() {
    let frames = load_esp_csv(ESP_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_esp_rotating();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.rms_broadband - f.baseline_rms;
        let sample = ResidualSample::new(f.step as f64, residual, 0.0, "esp_rms");
        engine.ingest_sample(&sample);
    }

    let episodes = aggregate_episodes(engine.history());
    let ncr = noise_compression_ratio(frames.len(), episodes.len());
    assert!(ncr >= 1.0,
        "NCR < 1: {} samples → {} episodes (NCR={:.2})",
        frames.len(), episodes.len(), ncr);
}

/// DSFB must detect non-Nominal states in the fault samples
/// (Unbalance, Rubbing, Misalignment typically raise RMS significantly).
#[test]
fn esp_dsfb_detects_non_nominal() {
    let frames = load_esp_csv(ESP_CSV).unwrap();
    let env = AdmissibilityEnvelope::default_esp_rotating();
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for f in &frames {
        let residual = f.rms_broadband - f.baseline_rms;
        let sample = ResidualSample::new(f.step as f64, residual, 0.0, "esp_rms");
        engine.ingest_sample(&sample);
    }

    let non_nominal = engine.history().iter()
        .filter(|s| s.state != GrammarState::Nominal)
        .count();

    // 1 231 fault samples out of 6 032; expect a meaningful fraction to be flagged
    assert!(non_nominal > 50,
        "DSFB produced < 50 non-Nominal states on ESP fault data — \
         envelope may be too wide (got {})", non_nominal);
}
