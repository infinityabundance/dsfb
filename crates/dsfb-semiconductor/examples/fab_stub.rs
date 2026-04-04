//! fab_stub — minimal read-only integration example.
//!
//! Demonstrates the DSFB observer contract:
//!   - input:  &[f64] residual slice from any upstream monitoring system
//!   - output: advisory Episode list — no write-back, no upstream coupling
//!
//! The upstream system (SPC / EWMA / FDC) is NOT modified.
//! DSFB can be removed without changing any upstream behavior.

fn main() {
    // Residuals produced by an existing upstream monitoring system.
    // DSFB receives a read-only slice — no mutable reference, no ownership.
    let residuals: &[f64] = &[
        0.10, 0.12, 0.15, 0.18, 0.21,   // nominal range
        0.35, 0.52, 0.74, 1.05, 1.40,   // drift onset
        1.80, 2.20, 2.65, 3.10, 3.60,   // boundary / violation
    ];

    let episodes = dsfb_semiconductor::observe(residuals);

    // Advisory output only — no write-back, no coupling to upstream.
    println!("tool=etch-chamber-12  features=S059  [advisory, read-only]");
    for e in &episodes {
        if e.decision != "Silent" {
            println!(
                "  index={:>2}  grammar={:<12}  decision={}",
                e.index, e.grammar, e.decision
            );
        }
    }

    println!("\n[no write-back]  [no upstream coupling]  [deterministic under replay]");
}
