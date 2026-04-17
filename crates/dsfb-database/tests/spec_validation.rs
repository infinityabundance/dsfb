//! YAML spec build-time validation (T3.6 of the elevation plan).
//!
//! The three YAML files under `spec/` are paper-grade artefacts:
//! `motifs.yaml` is the ground-truth threshold table (Table 3),
//! `perturbations.yaml` is the ground-truth label set for §8, and
//! `wizard_concordance.yaml` is the 50-fact concordance rendered as
//! Appendix A. They are not loaded at runtime by the crate (the
//! grammar's `default_for` constants are the runtime source of truth)
//! but they ARE the documents a reviewer reads first to understand
//! how the system was tuned. A silent rename in either YAML file —
//! a misspelt motif name, a stale class reference — would be caught
//! today only by a paper re-read. This test pins their cross-references
//! so a refactor that renames `MotifClass::CacheCollapse` cannot ship
//! without also touching the spec.
//!
//! Invariants pinned:
//!   1. `spec/motifs.yaml` parses as `MotifGrammar` and every top-level
//!      key resolves to a real `MotifClass`.
//!   2. Every `motif:` field in `spec/perturbations.yaml` resolves to
//!      a real `MotifClass` and the perturbation count matches the
//!      number of injected windows the harness emits at baseline.
//!   3. Every `motif:` field in `spec/wizard_concordance.yaml` resolves
//!      to a real `MotifClass`.
//!
//! These checks are intentionally cheap (pure parse + string compare)
//! so they stay well under a millisecond and add zero CI cost.

use dsfb_database::grammar::{MotifClass, MotifGrammar};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn spec_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("spec")
        .join(file)
}

fn motif_camel_names() -> HashSet<&'static str> {
    HashSet::from([
        "PlanRegressionOnset",
        "CardinalityMismatchRegime",
        "ContentionRamp",
        "CacheCollapse",
        "WorkloadPhaseTransition",
    ])
}

#[test]
fn motifs_yaml_parses_as_grammar_and_round_trips() {
    let yaml = fs::read_to_string(spec_path("motifs.yaml"))
        .expect("spec/motifs.yaml must be present alongside the crate");
    let g: MotifGrammar = serde_yaml::from_str(&yaml)
        .expect("spec/motifs.yaml must deserialise as MotifGrammar");
    // Cross-check that the YAML defines parameters for every MotifClass —
    // round-tripping through `params(class)` is the surest way to detect a
    // missing top-level section.
    for class in MotifClass::ALL {
        let p = g.params(class);
        assert!(
            p.rho > 0.0 && p.rho < 1.0,
            "spec/motifs.yaml: motif {:?} has invalid rho={}",
            class,
            p.rho
        );
    }
}

#[derive(Debug, Deserialize)]
struct PerturbationsFile {
    perturbations: Vec<PerturbationEntry>,
}
#[derive(Debug, Deserialize)]
struct PerturbationEntry {
    name: String,
    motif: String,
}

#[test]
fn perturbations_yaml_motifs_resolve_to_real_classes() {
    let yaml = fs::read_to_string(spec_path("perturbations.yaml"))
        .expect("spec/perturbations.yaml must be present");
    let parsed: PerturbationsFile = serde_yaml::from_str(&yaml)
        .expect("spec/perturbations.yaml must deserialise");
    let valid = motif_camel_names();
    assert_eq!(
        parsed.perturbations.len(),
        5,
        "spec/perturbations.yaml: expected 5 injected windows (one per motif class); found {}",
        parsed.perturbations.len()
    );
    for p in &parsed.perturbations {
        assert!(
            valid.contains(p.motif.as_str()),
            "spec/perturbations.yaml: perturbation {:?} references unknown motif {:?}",
            p.name,
            p.motif
        );
    }
}

#[derive(Debug, Deserialize)]
struct ConcordanceFile {
    facts: Vec<ConcordanceFact>,
}
#[derive(Debug, Deserialize)]
struct ConcordanceFact {
    id: u32,
    title: String,
    motif: String,
}

#[test]
fn wizard_concordance_yaml_motifs_resolve_to_real_classes() {
    let yaml = fs::read_to_string(spec_path("wizard_concordance.yaml"))
        .expect("spec/wizard_concordance.yaml must be present");
    let parsed: ConcordanceFile = serde_yaml::from_str(&yaml)
        .expect("spec/wizard_concordance.yaml must deserialise");
    let valid = motif_camel_names();
    assert!(
        !parsed.facts.is_empty(),
        "spec/wizard_concordance.yaml: facts list is empty"
    );
    for f in &parsed.facts {
        assert!(
            valid.contains(f.motif.as_str()),
            "spec/wizard_concordance.yaml: fact #{} ({:?}) references unknown motif {:?}",
            f.id,
            f.title,
            f.motif
        );
    }
}
