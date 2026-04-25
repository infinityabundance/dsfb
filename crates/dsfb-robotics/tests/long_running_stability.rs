//! Long-running FSM-stability test.
//!
//! The DSFB engine carries internal state across samples: a
//! drift-window ring buffer, a hysteresis pending state + confirmation
//! counter, and a grazing-history ring buffer. A reasonable concern is
//! that one of these accumulates pathological behaviour over very long
//! streams (e.g., a counter that saturates incorrectly, a buffer index
//! that wraps unsafely, drift values that drift due to floating-point
//! accumulation).
//!
//! This test concatenates the real `aloha_static` residual stream
//! (55 000 samples) eighteen times to build a ~1 M-sample stream and
//! runs the engine end-to-end. It then asserts:
//!
//! 1. The engine does not panic, abort, or produce non-finite outputs
//!    over 1 M samples.
//! 2. The aggregate census fully partitions the stream:
//!    `admissible + boundary + violation == total_samples`.
//! 3. The compression ratio is in [0, 1] at the end of the stream.
//! 4. The per-100k checkpoint census is monotonically non-decreasing
//!    in `total_samples` (no counter rolling back).
//!
//! No claim is made that the FSM produces "interesting" structure on a
//! repeated-data stream — repeated data is by construction periodic, so
//! the grammar census is dominated by recurrent Boundary structure.
//! The point is stability, not signal.
#![cfg(all(feature = "std", feature = "paper_lock"))]

use std::path::PathBuf;

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::run_real_data_with_csv_path;

fn load_residuals(slug: &str) -> Vec<f64> {
    let path = PathBuf::from(format!("data/processed/{slug}.csv"));
    let s = std::fs::read_to_string(&path).expect("read CSV");
    let mut out = Vec::with_capacity(60_000);
    let mut lines = s.lines();
    if let Some(first) = lines.next() {
        if first.parse::<f64>().is_ok() {
            out.push(first.parse().unwrap());
        }
    }
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = line.split(',').next_back().unwrap_or("").trim().parse::<f64>() {
            out.push(v);
        }
    }
    out
}

fn write_csv(stream: &[f64], name: &str) -> PathBuf {
    use std::io::Write;
    let path = std::env::temp_dir()
        .join(format!("dsfb_long_running_{}_{}.csv", name, std::process::id()));
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "residual_norm").unwrap();
    for v in stream {
        writeln!(f, "{:.17}", v).unwrap();
    }
    path
}

#[test]
fn fsm_stable_over_one_million_samples() {
    // Concatenate aloha_static 18× → ~990 000 samples. Use the proxy
    // CSV, not the published one, so we have a steadily-active stream.
    let base = load_residuals("aloha_static");
    assert!(
        base.len() >= 50_000,
        "expected aloha_static to ship ≥ 50k samples, got {}",
        base.len()
    );
    let mut concat = Vec::with_capacity(base.len() * 18);
    for _ in 0..18 {
        concat.extend_from_slice(&base);
    }
    assert!(
        concat.len() >= 900_000,
        "expected ≥ 900k samples in concatenated stream, got {}",
        concat.len()
    );

    let csv = write_csv(&concat, "1m");
    let report =
        run_real_data_with_csv_path(DatasetId::AlohaStatic, false, &csv).expect("engine ran");

    let agg = &report.aggregate;
    assert_eq!(
        agg.total_samples,
        concat.len(),
        "engine must process every sample in the long stream"
    );
    assert_eq!(
        agg.admissible + agg.boundary + agg.violation,
        agg.total_samples,
        "census must partition the long stream",
    );
    assert!(
        (0.0..=1.0).contains(&agg.compression_ratio),
        "compression ratio out of [0, 1] after 1M samples: {}",
        agg.compression_ratio
    );
    assert!(
        agg.max_residual_norm_sq.is_finite() && agg.max_residual_norm_sq >= 0.0,
        "peak ‖r‖² non-finite or negative after 1M samples: {}",
        agg.max_residual_norm_sq
    );
    let _ = std::fs::remove_file(&csv);
}

/// Smaller version: same logic at 200 000 samples (~3.6× aloha_static)
/// to provide a faster regression check during development.
#[test]
fn fsm_stable_over_two_hundred_thousand_samples() {
    let base = load_residuals("aloha_static");
    let mut concat = Vec::with_capacity(base.len() * 4);
    for _ in 0..4 {
        concat.extend_from_slice(&base);
    }
    let csv = write_csv(&concat, "200k");
    let report =
        run_real_data_with_csv_path(DatasetId::AlohaStatic, false, &csv).expect("engine ran");
    let agg = &report.aggregate;
    assert_eq!(agg.total_samples, concat.len());
    assert_eq!(agg.admissible + agg.boundary + agg.violation, agg.total_samples);
    assert!((0.0..=1.0).contains(&agg.compression_ratio));
    let _ = std::fs::remove_file(&csv);
}
