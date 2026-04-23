//! Paper-lock: headline metric enforcement for reproducibility.
//!
//! Implements the paper-lock discipline described in paper §F (Reproducibility
//! Statement): a CI-gated check that crate evolution has not silently altered
//! the published evaluation results.
//!
//! # Scope
//!
//! Only the **ORACLE (USRP B200)** scenario is enforced here. The ORACLE
//! scenario uses a fully self-contained synthetic observation stream that
//! is generated deterministically from fixed parameters and does not depend
//! on any external dataset or pre-trained receiver chain.
//!
//! ## Why RadioML 2018.01a is NOT paper-locked here
//!
//! The RadioML Table IV metrics require:
//! 1. A pre-trained demodulator or neural-network receiver that produces a
//!    per-symbol error residual (not raw IQ amplitude).
//! 2. That receiver's residual normalised to the healthy calibration window.
//!
//! The `hdf5_loader` module uses an **amplitude-template residual** -- the
//! paper's r(k) = x(k) - x_hat projected to the amplitude domain to avoid
//! carrier-phase sensitivity.  This captures the same structural phenomenon
//! (amplitude shape collapse at the demodulation threshold) but differs from
//! the carrier-synchronised decoder residual used in the original work.
//! Results are NOT directly comparable to Table IV.  Locking metrics from
//! this approximation would constitute an overclaim.  See `src/hdf5_loader.rs`.
//!
//! The RadioML row in paper Table IV is reproducible only when:
//! - The same pre-trained receiver is available (as used in the original work),
//!   OR
//! - A coherent analytical expression for the receiver residual is derived and
//!   the metrics are proved from the expression rather than from a run.
//!
//! ## Paper Headline Metrics (paper Table IV)
//!
//! ### ORACLE (USRP B200 field capture — fully self-contained)
//! - Episode count: 52
//! - Episode precision: 71.2% (±0.5% tolerance)
//! - Recall: 96/102 (≥96 minimum)
//!
//! ### RadioML 2018.01a (receiver-residual evaluation — NOT locked here)
//! - Episode count: 87           (requires correct receiver chain)
//! - Episode precision: 73.6%    (requires correct receiver chain)
//! - Recall: 97/102              (requires correct receiver chain)
//!
//! ## Usage
//!
//! ```text
//! cargo test --features paper_lock
//! cargo run --features paper_lock -- paper-lock
//! ```

extern crate std;

use crate::pipeline::{EvaluationResult, PaperLockExpected};

/// Headline metric expectations from paper Table IV — ORACLE scenario only.
///
/// The RadioML row is intentionally absent. See module-level documentation
/// for the reason (raw IQ amplitude is not a valid DSFB input feature without
/// a pre-trained receiver chain to provide the actual decoder residual).
pub struct PaperLockConfig {
    /// Locked expected metrics for the ORACLE USRP B200 field capture.
    /// This is fully self-contained and reproducible without external data.
    pub oracle: PaperLockExpected,

    /// Reference (NOT enforced) metrics for RadioML 2018.01a, reported in
    /// Table IV. Requires a pre-trained receiver chain to reproduce.
    /// Stored here for display/documentation purposes only.
    pub radioml_reference: PaperLockExpected,
}

impl PaperLockConfig {
    /// Build the locked configuration from paper Table IV.
    pub fn from_paper() -> Self {
        Self {
            oracle: PaperLockExpected {
                episode_count: 52,
                precision: 0.712,
                recall_min: 96,
            },
            radioml_reference: PaperLockExpected {
                episode_count: 87,
                precision: 0.736,
                recall_min: 97,
            },
        }
    }
}

/// Run the paper-lock verification against the ORACLE evaluation result.
///
/// Only the ORACLE scenario is enforced (see module documentation).
/// Returns `Ok(())` if all ORACLE metrics match, `Err(messages)` otherwise.
pub fn verify(
    oracle_result: &EvaluationResult,
) -> Result<(), std::vec::Vec<std::string::String>> {
    let config = PaperLockConfig::from_paper();
    let mut errors: std::vec::Vec<std::string::String> = std::vec::Vec::new();

    if let Err(e) = oracle_result.check_paper_lock(&config.oracle) {
        errors.push(e);
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Run a partial advisory check against the RadioML reference metrics.
///
/// Does NOT fail CI. Prints a note with the measured vs reference metrics.
pub fn advisory_check_radioml(result: &EvaluationResult) {
    advisory_check_radioml_aggregate(
        result.dsfb_episode_count,
        result.episode_precision,
        result.recall_numerator,
        result.recall_denominator,
    );
}

/// Advisory comparison for per-class aggregated results.
///
/// Does NOT fail CI. Prints an informational comparison of the aggregated
/// per-class DSFB metrics against the Table IV reference row.
/// The amplitude-template residual used here differs from the
/// carrier-synchronised decoder residual used in the paper.
pub fn advisory_check_radioml_aggregate(
    episodes: usize,
    precision: f32,
    recall_num: usize,
    recall_den: usize,
) {
    let config = PaperLockConfig::from_paper();
    let ref_cfg = &config.radioml_reference;
    std::println!("┌──────────────────────────────────────────────────────────────────┐");
    std::println!("│  DSFB Augmentation Summary  (NOT a detection benchmark)           │");
    std::println!("└──────────────────────────────────────────────────────────────────┘");
    std::println!("  DSFB augments an existing receiver chain — it does not replace it.");
    std::println!("  The amplitude-template Wasserstein residual is the \"usually");
    std::println!("  discarded residual\" from an amplitude-domain receiver.  DSFB");
    std::println!("  compresses raw threshold alarms into structured episodes,");
    std::println!("  providing human-readable regime characterisation.");
    std::println!();
    std::println!("  Structural information extracted:");
    std::println!("    DSFB episodes       : {:<6}  (compressed from raw boundary alarms)", episodes);
    std::println!("    Episode precision   : {:>5.1}%  (fraction covering a GT bin boundary)",
        precision * 100.0);
    std::println!("    Recall              : {}/{}   (GT bin boundaries covered)",
        recall_num, recall_den);
    std::println!();
    std::println!("  Paper Table IV reference (different residual source — NOT comparable):");
    std::println!("    Episodes: {}  Precision: {:.1}%  Recall: ≥{}/102",
        ref_cfg.episode_count, ref_cfg.precision * 100.0, ref_cfg.recall_min);
    std::println!("    (uses carrier-synchronised decoder residual, 128-sample variant)");
    std::println!("  See src/hdf5_loader.rs module documentation.");
}

/// Print a paper-lock verification report (ORACLE only).
pub fn print_report(
    oracle: &EvaluationResult,
) {
    std::println!("╔══════════════════════════════════════════════════════╗");
    std::println!("║      DSFB-RF Paper-Lock Verification (ORACLE)        ║");
    std::println!("╚══════════════════════════════════════════════════════╝");
    std::println!();
    std::println!(" Paper: de Beer (2026) DSFB Structural Semiotics Engine");
    std::println!("        for RF Signal Monitoring, Table IV");
    std::println!(" Note:  RadioML paper-lock removed — see src/paper_lock.rs §Scope.");
    std::println!();

    let config = PaperLockConfig::from_paper();

    print_dataset_check("ORACLE (USRP B200)", oracle, &config.oracle);
    std::println!();

    match verify(oracle) {
        Ok(()) => {
            std::println!("✓ ORACLE METRICS MATCH — paper-lock PASSED");
            std::println!("  Crate evolution has not changed published results.");
        }
        Err(errs) => {
            std::println!("✗ PAPER-LOCK FAILED — {} violation(s):", errs.len());
            for e in &errs {
                std::println!("  • {}", e);
            }
        }
    }
}

fn print_dataset_check(label: &str, result: &EvaluationResult, expected: &PaperLockExpected) {
    let ep_ok = (result.episode_precision - expected.precision).abs() <= 0.005;
    let rc_ok = result.recall_numerator >= expected.recall_min;
    let cnt_ok = result.dsfb_episode_count == expected.episode_count;

    std::println!(" Dataset: {}", label);
    std::println!("   Episodes:  {} (expected {}) {}",
        result.dsfb_episode_count, expected.episode_count,
        if cnt_ok { "✓" } else { "✗" });
    std::println!("   Precision: {:.1}% (expected {:.1}% ±0.5%) {}",
        result.episode_precision * 100.0, expected.precision * 100.0,
        if ep_ok { "✓" } else { "✗" });
    std::println!("   Recall:    {}/{} (min {}) {}",
        result.recall_numerator, result.recall_denominator,
        expected.recall_min,
        if rc_ok { "✓" } else { "✗" });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{synthetic_radioml_stream, run_stage_iii, EvaluationResult};

    fn make_synthetic_result(label: &'static str, n: usize, events_at: &[usize])
        -> EvaluationResult
    {
        let (obs, events) = synthetic_radioml_stream(n, events_at, 15.0);
        run_stage_iii(label, &obs, &events)
    }

    #[test]
    fn paper_lock_config_matches_paper_table_iv() {
        let config = PaperLockConfig::from_paper();
        // ORACLE — enforced
        assert_eq!(config.oracle.episode_count, 52);
        assert!((config.oracle.precision - 0.712).abs() < 1e-4);
        assert_eq!(config.oracle.recall_min, 96);
        // RadioML reference row — stored but not enforced
        assert_eq!(config.radioml_reference.episode_count, 87);
        assert!((config.radioml_reference.precision - 0.736).abs() < 1e-4);
        assert_eq!(config.radioml_reference.recall_min, 97);
    }

    #[test]
    fn verify_oracle_error_messages_are_informative() {
        // Synthetic result will NOT match paper metrics by design.
        // Verify the error message logic.
        let r = make_synthetic_result("fake_oracle", 400, &[150, 250, 350]);
        match verify(&r) {
            Ok(()) => { /* synthetic might accidentally match — acceptable */ }
            Err(errs) => {
                for e in &errs {
                    assert!(!e.is_empty(), "error messages must not be empty");
                    assert!(e.contains("expected") || e.contains("below"),
                        "error message should be informative: {}", e);
                }
            }
        }
    }

    #[test]
    fn print_report_does_not_panic() {
        let r = make_synthetic_result("oracle_print", 400, &[180, 300]);
        // Must not panic regardless of metric values
        print_report(&r);
    }

    #[test]
    fn advisory_check_does_not_panic() {
        let r = make_synthetic_result("radioml_advisory", 500, &[200, 350]);
        advisory_check_radioml(&r);
    }
}
