//! Dataset-provenance consistency gate.
//!
//! The running tool stamps each dataset's `data_kind` from MANIFEST.toml (P34). This gate fails CI if any
//! discovered dataset's kind diverges from its MANIFEST `kind` — the artifact provenance an evaluator sees
//! (`metrics.csv`, every `casefile.json`, the operator report) must match the manifest and the paper's
//! dataset table. It is the sim≠real discipline made a mechanical check, alongside the SHA-256 and
//! atlas-authority gates. A blanket "measured/slice" mislabel (the prior bug) would fail this test.

use dsfb_chemical_engineering_edge::{cli, datasets};
use std::path::PathBuf;

#[test]
fn every_discovered_dataset_kind_matches_manifest() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let kinds = datasets::manifest_kinds(&dir);
    assert!(
        !kinds.is_empty(),
        "MANIFEST.toml must declare per-dataset `kind` values"
    );

    let found = cli::discover_datasets(&dir);
    // The committed slices exist, so discovery returns them (not the synthetic fallback).
    assert!(
        found.iter().any(|d| kinds.contains_key(&d.name)),
        "expected committed slices to be discovered and present in the manifest"
    );

    for d in &found {
        if let Some(manifest_kind) = kinds.get(&d.name) {
            let want = datasets::data_kind_tag(manifest_kind);
            assert_eq!(
                d.kind, want,
                "dataset '{}' stamped data_kind '{}' but MANIFEST kind '{}' maps to '{}' \
                 — the artifact provenance must match the manifest",
                d.name, d.kind, manifest_kind, want
            );
        }
    }
}

#[test]
fn the_nine_simulations_are_not_stamped_measured() {
    // Regression guard for the exact prior bug: the 9 simulation slices must NOT carry a "measured" tag.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let found = cli::discover_datasets(&dir);
    for sim in [
        "tennessee_eastman_idv01",
        "tennessee_eastman_idv04",
        "tennessee_eastman_idv06",
        "tennessee_eastman_idv13",
        "tennessee_eastman_idv14",
        "cstr_reactor",
        "three_tank",
        "bsm1_wastewater",
        "penicillin_fedbatch",
    ] {
        if let Some(d) = found.iter().find(|d| d.name == sim) {
            assert!(
                !d.kind.starts_with("measured"),
                "simulation '{sim}' must not be stamped '{}' (sim≠real)",
                d.kind
            );
            assert!(
                d.kind.starts_with("simulation"),
                "simulation '{sim}' should be 'simulation/...' got '{}'",
                d.kind
            );
        }
    }
}
