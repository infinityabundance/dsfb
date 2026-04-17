//! Non-claim charter for DSFB-Database.
//!
//! These strings are reproduced verbatim in the CLI banner, in every report
//! header, and in the abstract of the paper. The test
//! `tests/non_claim_lock.rs` pins them so that a reviewer can verify
//! that the operator pitch and the published paper agree byte-for-byte.

/// Five things this work explicitly does not claim.
pub const NON_CLAIMS: [&str; 5] = [
    "DSFB-Database does not optimise queries, replace the query optimiser, or modify execution plans.",
    "DSFB-Database does not claim causal correctness; motifs represent structural consistency given observed signals, not root causes.",
    "DSFB-Database does not provide a forecasting or predictive guarantee; precursor structure is structural, not probabilistic.",
    "DSFB-Database does not provide ground-truth-validated detection on real workloads; we evaluate via injected perturbations, plan-hash concordance, and replay determinism.",
    "DSFB-Database does not claim a universal SQL grammar; motifs are engine-aware, telemetry-aware, and workload-aware.",
];

/// Print the non-claim block (used by CLI and embedded in every report).
pub fn print() {
    eprintln!("DSFB-Database non-claims (read these before interpreting any output):");
    for (i, c) in NON_CLAIMS.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, c);
    }
}

/// Render the non-claim block as a single newline-joined string for embedding
/// in CSV / JSON report headers and in the LaTeX paper.
pub fn as_block() -> String {
    NON_CLAIMS
        .iter()
        .enumerate()
        .map(|(i, c)| format!("  {}. {}", i + 1, c))
        .collect::<Vec<_>>()
        .join("\n")
}
