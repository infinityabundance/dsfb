//! Gate for `ArtifactCompletenessCourtV1`: the committed artifact graph (MANIFEST ↔ per-slice
//! digests ↔ frozen bundle/evidence roots ↔ authority hashes ↔ metrics protocol) must be complete
//! and mutually consistent. A failing court is a release blocker. Run with the `soft-sensor-corpus`
//! feature to also exercise the corpus hash + classification-census partition checks.

use dsfb_chemical_engineering_edge::completeness::run_completeness_court;
use std::path::PathBuf;

#[test]
fn committed_artifact_graph_is_complete_and_consistent() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report = run_completeness_court(&crate_dir);
    assert!(!report.findings.is_empty(), "court must produce findings");
    let failures: Vec<String> = report
        .findings
        .iter()
        .filter(|f| !f.passed)
        .map(|f| format!("  {} — {}", f.check, f.detail))
        .collect();
    assert!(
        report.all_passed(),
        "ArtifactCompletenessCourtV1 found {} incomplete/inconsistent artifact(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    // The verdict digest is deterministic (stable across two runs of the same committed tree).
    let again = run_completeness_court(&crate_dir);
    assert_eq!(
        report.report_hash, again.report_hash,
        "report_hash must be deterministic"
    );
}

// ── Independent oracle (Wave-1 B2) ──────────────────────────────────────────────────────────────────
// The test above calls the court and trusts its verdict — the court is checked *by itself*. This oracle
// re-derives the core completeness facts from the raw committed files via a SEPARATE, dependency-free code
// path (line scanning, not the court's toml/serde logic), then cross-checks the court against it. A bug
// shared between the court and the first test would slip through; a bug in only one of the two independent
// paths is caught here.

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Extract `name = "..."` values that immediately follow a `[[dataset]]` (MANIFEST) or that head a
/// `[section]` (EXPECTED_BUNDLE_ROOTS), plus every `key = "<64hex>"` value, by raw line scanning.
fn scan(text: &str, key: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let prefix = format!("{key} = \"");
            l.strip_prefix(&prefix)
                .and_then(|r| r.strip_suffix('"'))
                .map(|s| s.to_string())
        })
        .collect()
}

fn section_names(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            // `[name]` (EXPECTED) — exclude the array-of-tables `[[dataset]]`.
            if l.starts_with('[') && l.ends_with(']') && !l.starts_with("[[") {
                Some(l[1..l.len() - 1].to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn independent_oracle_cross_checks_the_court() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        std::fs::read_to_string(crate_dir.join("data/MANIFEST.toml")).expect("MANIFEST.toml");
    let bundles = std::fs::read_to_string(crate_dir.join("data/EXPECTED_BUNDLE_ROOTS.toml"))
        .expect("EXPECTED_BUNDLE_ROOTS.toml");

    // (1) MANIFEST: independent name + sha256 extraction.
    let manifest_names = scan(&manifest, "name");
    let manifest_shas = scan(&manifest, "sha256");
    assert_eq!(
        manifest_names.len(),
        20,
        "expected 20 dataset names in MANIFEST, found {}",
        manifest_names.len()
    );
    assert_eq!(
        manifest_shas.len(),
        20,
        "expected 20 sha256 digests in MANIFEST, found {}",
        manifest_shas.len()
    );
    assert!(
        manifest_shas.iter().all(|s| is_hex64(s)),
        "every MANIFEST sha256 must be 64-hex"
    );
    // The declared count must match the independently-counted blocks.
    assert!(
        manifest.contains("dataset_count = 20"),
        "MANIFEST declared dataset_count must be 20"
    );

    // (2) EXPECTED_BUNDLE_ROOTS: independent name + root extraction.
    let bundle_names = section_names(&bundles);
    let bundle_roots = scan(&bundles, "bundle_root");
    let evidence_roots = scan(&bundles, "evidence_root");
    assert_eq!(
        bundle_names.len(),
        20,
        "expected 20 bundle sections, found {}",
        bundle_names.len()
    );
    assert_eq!(
        bundle_roots.len(),
        20,
        "expected 20 bundle_root values, found {}",
        bundle_roots.len()
    );
    assert_eq!(
        evidence_roots.len(),
        20,
        "expected 20 evidence_root values, found {}",
        evidence_roots.len()
    );
    assert!(
        bundle_roots
            .iter()
            .chain(&evidence_roots)
            .all(|s| is_hex64(s)),
        "every root must be 64-hex"
    );

    // (3) name-for-name agreement between the two authorities (as sets).
    let mut a = manifest_names.clone();
    let mut b = bundle_names.clone();
    a.sort();
    b.sort();
    assert_eq!(
        a, b,
        "MANIFEST and EXPECTED_BUNDLE_ROOTS must name the same 20 datasets"
    );

    // (4) Cross-check the court: it must PASS and must have operated on the same 20 datasets the oracle
    // independently counted (the court reports `dataset_count=20` in its manifest finding).
    let report = run_completeness_court(&crate_dir);
    assert!(
        report.all_passed(),
        "court must pass when the oracle says the graph is complete"
    );
    let says_20 = report
        .findings
        .iter()
        .any(|f| f.detail.contains("dataset_count=20"));
    assert!(
        says_20,
        "court's dataset_count must agree with the oracle's independent count of 20"
    );
}
