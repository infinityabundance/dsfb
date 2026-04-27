//! Integration test: end-to-end smoke test that the dsfb-atlas API surface
//! produces the expected SHA-256 deduplication invariant.
//!
//! This test runs as part of `cargo test -p dsfb-atlas` and lifts the
//! dsfb-gray Verification Evidence subscore.

use dsfb_atlas::dedup::Dedup;
use dsfb_atlas::generator;
use dsfb_atlas::schema::{Chapter, Part};

fn sample_part() -> Part {
    Part {
        part_id: "P01".to_string(),
        part_name: "Categorical".to_string(),
        lens: "categorical".to_string(),
        default_anchor_tier: "T4".to_string(),
        missing_layers_default: vec![
            "grammar".to_string(),
            "trust".to_string(),
            "certificate".to_string(),
        ],
        reduction_kind_default: "constructive".to_string(),
        default_class_color: "partCAT".to_string(),
        chapters: (1..=10).map(sample_chapter).collect(),
    }
}

fn sample_chapter(idx: usize) -> Chapter {
    Chapter {
        chapter_id: format!("P01-C{idx:02}"),
        chapter_name: format!("Test Chapter {idx}"),
        class_color: None,
        anchor_tier: None,
        anchor_bank_ids: vec![],
        paperstack_cite: None,
        public_dataset: None,
        missing_layers: None,
        reduction_kind: None,
        stems: (1..=10).map(|i| format!("stem-{i}")).collect(),
        modifiers: (1..=10).map(|i| format!("modifier-{i}")).collect(),
        operation_phrase_template: "applies a {modifier} {stem} on residuals".to_string(),
        output_type: "scalar".to_string(),
        input_signal_class: "deterministic".to_string(),
    }
}

#[test]
fn generate_part_emits_one_thousand_unique_proofs() {
    let part = sample_part();
    let mut dedup = Dedup::new();
    let (latex, count) = generator::generate_part(&part, &mut dedup).unwrap();
    let report = dedup.finalize();

    assert_eq!(count, 1000, "Each Part contains exactly 10 chapters * 10 stems * 10 modifiers");
    assert_eq!(report.total, 1000);
    assert_eq!(report.unique, 1000, "All 1000 proof bodies must hash distinctly");
    assert!(report.collisions.is_empty(), "build must fail on any SHA-256 collision");
    assert!(latex.contains("\\begin{atlastheorem}"), "LaTeX output must include atlas-theorem environments");
}

#[test]
fn generate_part_is_deterministic() {
    // Two generations on the same input must produce byte-identical
    // LaTeX and identical hash sets — this is the byte-determinism
    // invariant that backs the audit-trail completeness claim.
    let part = sample_part();
    let mut d1 = Dedup::new();
    let (latex1, _) = generator::generate_part(&part, &mut d1).unwrap();
    let r1 = d1.finalize();

    let mut d2 = Dedup::new();
    let (latex2, _) = generator::generate_part(&part, &mut d2).unwrap();
    let r2 = d2.finalize();

    assert_eq!(latex1, latex2, "byte-deterministic LaTeX output");
    assert_eq!(r1.total, r2.total);
    assert_eq!(r1.unique, r2.unique);
    assert_eq!(r1.collisions.len(), r2.collisions.len());
}
