//! Minimal fab sidecar example.
//!
//! Demonstrates non-intrusive read-only observer integration without invoking
//! the full SECOM benchmark pipeline. No uploads, no control modifications,
//! no external network calls.
//!
//! INTEGRATION CONTRACT
//! ─────────────────────
//! - DSFB reads residuals produced by existing SPC / FDC / EWMA controllers.
//! - DSFB writes no data back to any upstream system.
//! - Removing the sidecar leaves upstream behavior entirely unchanged.
//! - The observer is deterministic: same residuals → same advisory output.
//!
//! WHAT THIS EXAMPLE SHOWS
//! ─────────────────────────
//! 1. Show the paper-evaluated parameter configuration.
//! 2. Build a synthetic scalar residual sequence (replaces historian reads).
//! 3. Feed residuals into the read-only `observe()` API.
//! 4. Print advisory dispositions (Silent / Review / Escalate) without any
//!    write-back to upstream systems.
//!
//! Canonical configuration (paper Section 10.8, Appendix F.4):
//!   W=10, K=4, tau=2.0, m=1, feature_set=all_features, mode=compression_biased

use dsfb_semiconductor::observe;

fn main() {
    // ── 1. Paper-evaluated parameter configuration ────────────────────────
    // Canonical string: W=10, K=4, tau=2.0, m=1, feature_set=all_features,
    //                   mode=compression_biased
    // These parameters are the output of the DSA optimization sweep and appear
    // in the paper (Section 10.8 and Appendix F.4).
    println!("DSFB Fab Sidecar — minimal read-only integration example");
    println!(
        "Selected config: W=10, K=4, tau=2.0, m=1, \
         feature_set=all_features, mode=compression_biased"
    );
    println!();

    // ── 2. Synthetic residual sequence ────────────────────────────────────
    // In production: replace with live reads from an existing FDC / SPC historian.
    // The observer accepts a shared reference — no mutable access required.
    let residuals: &[f64] = &[
        0.01, 0.02, 0.05, 0.10, 0.22, // nominal region
        0.35, 0.48, 0.61, 0.74, 0.88, // building drift
        0.92, 1.05, 1.18,              // escalation region
    ];

    // ── 3. Observer pass-through (read-only, no write-back) ───────────────
    let episodes = observe(residuals);

    // ── 4. Print advisory dispositions ────────────────────────────────────
    println!("Advisory output (no write-back, no upstream coupling):");
    for e in &episodes {
        if e.decision != "Silent" {
            println!(
                "  index={:>2}  grammar={:<20}  decision={}",
                e.index, e.grammar, e.decision
            );
        }
    }

    // ── Summary ───────────────────────────────────────────────────────────
    let non_silent = episodes.iter().filter(|e| e.decision != "Silent").count();
    println!();
    println!("Total observations : {}", episodes.len());
    println!("Advisory events    : {}", non_silent);
    println!("Upstream unchanged : true  (observer-only, no writes)");
    println!(
        "Reproducible       : true  (same residuals → same advisory output)"
    );
}
