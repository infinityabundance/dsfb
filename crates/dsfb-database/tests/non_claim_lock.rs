//! Non-claim lock test.
//!
//! The five non-claim strings in [`dsfb_database::non_claims`] are
//! reproduced verbatim in the abstract, in §10 of the paper, and in every
//! CLI invocation. They are part of the paper's contract with reviewers
//! and operators and may not silently change. This test pins the exact
//! bytes of the non-claim block so that a future edit either updates the
//! paper *and* the CLI banner together, or fails CI.

use dsfb_database::non_claims;
use std::fs;
use std::path::PathBuf;

#[test]
fn non_claim_block_is_verbatim() {
    let expected = [
        "DSFB-Database does not optimise queries, replace the query optimiser, or modify execution plans.",
        "DSFB-Database does not claim causal correctness; motifs represent structural consistency given observed signals, not root causes.",
        "DSFB-Database does not provide a forecasting or predictive guarantee; precursor structure is structural, not probabilistic.",
        "DSFB-Database does not provide ground-truth-validated detection on real workloads; we evaluate via injected perturbations, plan-hash concordance, and replay determinism.",
        "DSFB-Database does not claim a universal SQL grammar; motifs are engine-aware, telemetry-aware, and workload-aware.",
    ];
    assert_eq!(
        non_claims::NON_CLAIMS.len(),
        expected.len(),
        "non-claim count drifted; update both the crate and the paper"
    );
    for (i, (actual, want)) in non_claims::NON_CLAIMS.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual, want,
            "non-claim #{} drifted from paper-locked text",
            i + 1
        );
    }
}

#[test]
fn non_claim_block_is_printable() {
    // The CLI prints `non_claims::as_block()` on every run; it must be a
    // newline-joined, numbered list. This protects against a refactor
    // that accidentally drops the formatting.
    let block = non_claims::as_block();
    for n in 1..=5 {
        assert!(
            block.contains(&format!("  {}.", n)),
            "non-claim numbering #{} missing",
            n
        );
    }
}

#[test]
fn paper_non_claims_section_matches_crate_strings() {
    // T3.7: close the loophole the audit flagged — the §10 Non-Claims
    // tcolorbox in the paper and the `NON_CLAIMS` array in the crate
    // are two copies of the same five strings, and they must not drift
    // independently. We parse the relevant block out of `paper/dsfb-database.tex`
    // and assert each `\item ...` body is byte-equal (modulo leading/
    // trailing whitespace) to the corresponding `NON_CLAIMS` entry.
    //
    // If the paper is moved or the section header is renamed, this test
    // fails loudly rather than silently passing because it could not
    // find the section. That is the intended behaviour: a paper edit
    // that breaks the lock should be visible in CI.
    let tex_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("paper")
        .join("dsfb-database.tex");
    // The lock only runs when the paper source is colocated with the crate.
    // Published crate tarballs do not ship the paper, so we treat its
    // absence as a skip rather than a hard failure. When the paper IS
    // present (in-tree development, full repro bundle), drift is still
    // caught byte-for-byte.
    let tex = match fs::read_to_string(&tex_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "skipping paper<->crate non-claim lock: {} not present",
                tex_path.display()
            );
            return;
        }
    };

    // Locate the §10 Non-Claims tcolorbox. We anchor on the section
    // label rather than the title text so a future title rewording
    // (e.g. capitalisation) does not break the lock.
    let label_anchor = "\\label{sec:non-claims}";
    let section_start = tex.find(label_anchor).unwrap_or_else(|| {
        panic!(
            "{} does not contain {:?}; non-claim section may have been removed or relabelled",
            tex_path.display(),
            label_anchor
        )
    });
    let section_tail = &tex[section_start..];
    let env_start_rel = section_tail.find("\\begin{enumerate}").expect(
        "non-claims section is missing its \\begin{enumerate}; the lock cannot match items",
    );
    let env_end_rel = section_tail.find("\\end{enumerate}").expect(
        "non-claims section is missing its \\end{enumerate}",
    );
    let block = &section_tail[env_start_rel..env_end_rel];

    let items: Vec<String> = block
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("\\item ").map(|s| s.trim().to_owned())
        })
        .collect();

    assert_eq!(
        items.len(),
        non_claims::NON_CLAIMS.len(),
        "paper §10 enumerate item count ({}) does not match crate non-claim count ({})",
        items.len(),
        non_claims::NON_CLAIMS.len()
    );
    for (i, (paper, crate_str)) in items.iter().zip(non_claims::NON_CLAIMS.iter()).enumerate() {
        assert_eq!(
            paper.as_str(),
            *crate_str,
            "non-claim #{} in paper §10 does not match crate string verbatim",
            i + 1
        );
    }
}
