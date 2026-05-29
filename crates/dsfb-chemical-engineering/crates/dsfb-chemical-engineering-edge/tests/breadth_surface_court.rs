//! Self-check court for the top-level `breadth_surface.toml` navigation index (panel "Immediate" item).
//!
//! Joins the court/gate test family. It keeps the hand-curated index honest WITHOUT network: every tier
//! resolves to a real `ClaimStrength`; every `cli_subcommand` reproduction names one of the real
//! `dsfb-chem-edge` subcommands (an independent copy of the list, completeness-court "oracle" style); every
//! governing artifact exists on disk (or is a known frozen-hash name); the category counts equal what
//! `atlas::validate()` + the committed manifests actually report (re-derived, not re-typed); every Tier-3
//! claim states a non-claim boundary; ids are unique + categories are in the fixed set; and the parsed index
//! hashes to a pinned canonical digest. If the index drifts from reality, exactly one of these fails loudly.

use dsfb_chemical_engineering_edge::breadth_surface::BreadthSurfaceV1;
use std::path::{Path, PathBuf};

/// The real `dsfb-chem-edge` subcommands (an independent second copy of the dispatch in `main.rs`; the
/// length tripwire catches a subcommand added there without updating the index discipline).
const SUBCOMMANDS: [&str; 22] = [
    "demo",
    "figures",
    "analyze",
    "atlas",
    "corpus",
    "verify-replay",
    "completeness-court",
    "release-scrub",
    "unit-consistency",
    "data-readiness",
    "authority-diff",
    "regime-eval",
    "historian",
    "balance-witness",
    "control-action",
    "casefile",
    "confidential-demo",
    // added since the original 17: P97 generate-index, P98 verify-index, the narration-context emitter, the
    // feature-gated P101 densor-runtime-demo, and the feature-gated narration-heuristic-demo (each a subcommand even
    // when its feature is off — it prints a rebuild hint).
    "generate-index",
    "verify-index",
    "narration-context",
    "densor-runtime-demo",
    "narration-heuristic-demo",
];

/// The fixed category set (breadth only widens — adding a category is a deliberate edit here + the index).
const CATEGORIES: [&str; 15] = [
    "determinism_replay",
    "authority_atlas",
    "authority_corpus",
    "datasets_provenance",
    "physics_balance_witness",
    "unit_consistency",
    "historian_layer",
    "confidential_eval_chain",
    "formal_proofs",
    "figures_paper",
    "embedded_wasm",
    "completeness_court",
    "release_hygiene",
    "regime_envelope",
    "casefile_court",
];

const FROZEN_HASHES: [&str; 2] = ["atlas_hash_v1", "corpus_hash_v1"];

// Pinned canonical digest of the parsed index — re-freeze deliberately when a claim/tier/count changes
// (the same mint-then-freeze discipline as golden_replay.rs / atlas_gates.rs). Breadth only widens.
// Re-freeze (2026-05-28): added claim CASE-02 (the narration-context emitter) — a deliberate widening, so the
// canonical hash moved 848d100f… → e4110fc5… (the prior value covered the index before CASE-02).
// Re-freeze (2026-05-28, panel narrator-generator extension): added claim CASE-03 (the H7–H12 narration-failure
// heuristic bank + feature-gated demonstrator) and the feature-gated `narration-heuristic-demo` subcommand
// (SUBCOMMANDS 21 → 22) — a deliberate widening, so the canonical hash moved e4110fc5… → eb89cec8….
// Re-freeze (2026-05-29): the REL-01 release-hygiene claim was narrowed to the checks release-scrub performs
// (placeholder DOI + private-backup leak), so the canonical hash moved eb89cec8… → 97dbc4bc….
// Re-freeze (2026-05-29): FIG-01 was corrected from a bundled 60-figure claim to an explicit zero-figure
// non-claim for this crate snapshot, so the canonical hash moved 97dbc4bc… → a31c99a9….
const BREADTH_SURFACE_HASH_V1: &str =
    "a31c99a94e3995b66b8e85a6ca9b495a53f81d586cdbe58068544240cf19a69b";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}

fn load() -> BreadthSurfaceV1 {
    BreadthSurfaceV1::load(&repo_root()).expect("breadth_surface.toml loads + parses")
}

fn count_lines_matching(path: &Path, pred: impl Fn(&str) -> bool) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.lines().filter(|l| pred(l)).count())
        .unwrap_or(0)
}

/// (1) Every claim's tier resolves to a real `ClaimStrength`.
#[test]
fn every_tier_resolves() {
    for c in &load().claims {
        assert!(
            BreadthSurfaceV1::tier_of(&c.tier).is_some(),
            "{}: unknown tier '{}'",
            c.id,
            c.tier
        );
    }
}

/// (2) Every `cli_subcommand` reproduction names a real subcommand; the list has the expected length.
#[test]
fn every_cli_subcommand_is_real() {
    assert_eq!(
        SUBCOMMANDS.len(),
        22,
        "subcommand-count tripwire (update the index discipline if main.rs grew)"
    );
    for c in load()
        .claims
        .iter()
        .filter(|c| c.reproduce_kind == "cli_subcommand")
    {
        let sub = BreadthSurfaceV1::cli_subcommand(&c.reproduce)
            .unwrap_or_else(|| panic!("{}: reproduce must be 'dsfb-chem-edge <sub> …'", c.id));
        assert!(
            SUBCOMMANDS.contains(&sub),
            "{}: '{}' is not a real subcommand",
            c.id,
            sub
        );
    }
}

/// (3) Every governing artifact exists on disk (path kinds) or is a known frozen-hash name.
#[test]
fn every_governing_artifact_resolves() {
    let root = repo_root();
    for c in &load().claims {
        if c.artifact_kind == "frozen_hash" {
            assert!(
                FROZEN_HASHES.contains(&c.governing_artifact.as_str()),
                "{}: unknown frozen hash '{}'",
                c.id,
                c.governing_artifact
            );
        } else {
            assert!(
                root.join(&c.governing_artifact).exists(),
                "{}: artifact '{}' does not exist",
                c.id,
                c.governing_artifact
            );
        }
    }
}

/// (4) The category counts equal what the authority + the committed manifests actually report — re-derived,
/// not re-typed, so the index cannot silently drift (promote a 19th detector and this fails until updated).
#[test]
fn counts_match_reality() {
    let s = load();
    let e = &s.expected_counts;
    let root = repo_root();
    let edge = root.join("crates/dsfb-chemical-engineering-edge");

    let v = dsfb_chemical_engineering_atlas::validate().expect("atlas validates");
    assert_eq!(e.executed_detectors, v.n_executed, "executed_detectors");
    assert_eq!(
        e.executed_fault_signatures, v.n_fault_signatures_executed,
        "executed_fault_signatures"
    );
    assert_eq!(
        e.fault_signatures, v.n_fault_signatures,
        "fault_signatures (total bank)"
    );
    assert_eq!(e.heuristics, v.n_heuristics, "heuristics");

    let datasets = count_lines_matching(&edge.join("data/MANIFEST.toml"), |l| {
        l.trim_start().starts_with("[[dataset]]")
    });
    assert_eq!(
        e.datasets, datasets,
        "datasets (MANIFEST [[dataset]] count)"
    );

    let golden = std::fs::read_to_string(edge.join("tests/golden_replay.rs"))
        .map(|s| s.matches("\"synth_").count())
        .unwrap_or(0);
    assert_eq!(
        e.golden_replays, golden,
        "golden_replays (GOLDEN synth_ entries)"
    );

    let witnesses =
        count_lines_matching(&edge.join("data/instrumented/EXPECTED_DIGESTS.toml"), |l| {
            let t = l.trim_start();
            !t.starts_with('#') && t.contains('=')
        });
    assert_eq!(
        e.balance_witnesses, witnesses,
        "balance_witnesses (EXPECTED_DIGESTS entries)"
    );

    let fig = std::fs::read_to_string(root.join("paper/figures/figure_manifest.json"))
        .unwrap_or_default();
    let figures = fig
        .split("\"n_figures\"")
        .nth(1)
        .and_then(|t| t.split([',', '}']).next())
        .and_then(|t| {
            t.trim_start_matches([':', ' '])
                .trim()
                .parse::<usize>()
                .ok()
        })
        .unwrap_or(0);
    assert_eq!(e.figures, figures, "figures (figure_manifest n_figures)");
}

/// (5) Every Tier-3 (`SpeculativeImplication`) claim states a non-claim boundary (the no-overclaim discipline,
/// enforced structurally).
#[test]
fn tier3_claims_state_a_boundary() {
    for c in load()
        .claims
        .iter()
        .filter(|c| c.tier == "SpeculativeImplication")
    {
        assert!(
            !c.non_claim_boundary.trim().is_empty(),
            "{}: a Tier-3 claim must state its non-claim boundary",
            c.id
        );
    }
}

/// (6) Ids are unique and every category is in the fixed set; reproduce/artifact kinds are known.
#[test]
fn ids_unique_categories_and_kinds_known() {
    let s = load();
    let mut seen = std::collections::BTreeSet::new();
    for c in &s.claims {
        assert!(seen.insert(c.id.clone()), "duplicate claim id {}", c.id);
        assert!(
            CATEGORIES.contains(&c.category.as_str()),
            "{}: unknown category '{}'",
            c.id,
            c.category
        );
        assert!(
            ["cli_subcommand", "cargo_test", "script", "external"]
                .contains(&c.reproduce_kind.as_str()),
            "{}: unknown reproduce_kind '{}'",
            c.id,
            c.reproduce_kind
        );
        assert!(
            [
                "file",
                "test",
                "manifest",
                "figure_manifest",
                "script",
                "frozen_hash"
            ]
            .contains(&c.artifact_kind.as_str()),
            "{}: unknown artifact_kind '{}'",
            c.id,
            c.artifact_kind
        );
    }
    assert!(
        s.claims.len() >= 15,
        "the index should cover the breadth surface (got {})",
        s.claims.len()
    );
}

/// (7) The parsed index hashes to the pinned canonical digest (joins the governed-artifact discipline).
#[test]
fn canonical_hash_is_frozen() {
    let got = load().canonical_hash();
    assert_eq!(
        got, BREADTH_SURFACE_HASH_V1,
        "breadth_surface canonical hash drift — re-freeze deliberately if the index changed"
    );
}
