//! Authority gate: prove the edge crate's *executed* detectors and *runtime* heuristics are all
//! catalogued in the `dsfb-chemical-engineering-atlas` authority crate.
//!
//! The check is **subset-membership, not order** — `build_bank()` order intentionally differs from
//! the corpus `canonical_id` order, so an order-equality assertion would be wrong. We assert that
//! every executed detector id and every heuristic id/input is present in the authority records, and
//! that shared records agree on family / primitive_family.

use dsfb_chemical_engineering_atlas as authority;
use dsfb_chemical_engineering_edge::data::DataMatrix;
use dsfb_chemical_engineering_edge::detectors::build_bank;
use dsfb_chemical_engineering_edge::heuristics::canonical_bank;
use dsfb_chemical_engineering_edge::Corpus;

fn atlas_detector_ids() -> Vec<&'static str> {
    authority::DETECTOR_RECORDS
        .iter()
        .map(|d| d.detector_id)
        .collect()
}

#[test]
fn executed_detectors_are_catalogued_in_atlas() {
    // A small synthetic matrix is enough to instantiate build_bank (RobustZMad::fit needs samples).
    let rows: Vec<Vec<f64>> = (0..16)
        .map(|i| vec![i as f64, (i % 3) as f64, (i % 5) as f64])
        .collect();
    let m = DataMatrix::new(vec!["a".into(), "b".into(), "c".into()], rows);
    let bank = build_bank(&m, 8);
    let atlas_ids = atlas_detector_ids();
    for d in &bank {
        assert!(
            atlas_ids.contains(&d.id()),
            "executed detector '{}' is not catalogued in the atlas authority crate",
            d.id()
        );
    }
}

#[test]
fn edge_corpus_executed_agrees_with_atlas() {
    // For every detector id present in BOTH the edge corpus (executed) and the atlas, family and
    // primitive_family must agree — catches drift between the two record sources.
    let corpus = Corpus::load().expect("edge corpus loads");
    for cd in corpus.executed() {
        let rec = authority::DETECTOR_RECORDS
            .iter()
            .find(|r| r.detector_id == cd.detector_id)
            .unwrap_or_else(|| {
                panic!(
                    "executed corpus detector '{}' missing from atlas",
                    cd.detector_id
                )
            });
        assert_eq!(
            rec.family, cd.family,
            "family mismatch for {}",
            cd.detector_id
        );
        assert_eq!(
            format!("{:?}", rec.primitive_family),
            cd.primitive_family,
            "primitive_family mismatch for {}",
            cd.detector_id
        );
    }
}

#[test]
fn atlas_executed_records_are_all_built_by_the_bank() {
    // Converse of `executed_detectors_are_catalogued_in_atlas`: every detector the atlas ships as an
    // *executed authority* record must actually be instantiated by `build_bank()`. Without this direction
    // the atlas could claim a `detector_id` as executed authority that the edge pipeline never runs — a
    // phantom authority record. With both directions the executed set is a proven **bijection**
    // (atlas executed authority == edge runtime bank), not merely a one-way subset.
    let rows: Vec<Vec<f64>> = (0..16)
        .map(|i| vec![i as f64, (i % 3) as f64, (i % 5) as f64])
        .collect();
    let m = DataMatrix::new(vec!["a".into(), "b".into(), "c".into()], rows);
    let bank_ids: Vec<&'static str> = build_bank(&m, 8).iter().map(|d| d.id()).collect();
    for id in atlas_detector_ids() {
        assert!(
            bank_ids.contains(&id),
            "atlas catalogues '{id}' as an executed detector, but build_bank() never instantiates it",
        );
    }
    // Exact bijection: no duplicate ids in the bank, and the two sets have the same cardinality, so the
    // forward gate (bank ⊆ atlas) + this gate (atlas ⊆ bank) pin the executed set on the nose.
    let mut uniq = bank_ids.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(
        uniq.len(),
        bank_ids.len(),
        "build_bank() must not build duplicate detector ids"
    );
    assert_eq!(
        bank_ids.len(),
        atlas_detector_ids().len(),
        "executed-set size mismatch: bank has {} detectors, atlas catalogues {}",
        bank_ids.len(),
        atlas_detector_ids().len()
    );
}

#[test]
fn heuristics_and_their_inputs_are_catalogued() {
    let atlas_ids = atlas_detector_ids();
    let bank = canonical_bank();
    for h in &bank {
        // heuristic id present in the atlas
        let rec = authority::HEURISTIC_RECORDS
            .iter()
            .find(|r| r.heuristic_id == h.heuristic_id)
            .unwrap_or_else(|| panic!("runtime heuristic '{}' missing from atlas", h.heuristic_id));
        // every detector input references a catalogued detector
        for input in &h.detector_inputs {
            assert!(
                atlas_ids.contains(&input.as_str()),
                "heuristic {} input '{}' is not a catalogued detector",
                h.heuristic_id,
                input
            );
        }
        // the atlas record mirrors edge's runtime detector_inputs exactly
        let atlas_inputs: Vec<&str> = rec.detector_inputs.to_vec();
        let edge_inputs: Vec<&str> = h.detector_inputs.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            atlas_inputs, edge_inputs,
            "detector_inputs drift for {}",
            h.heuristic_id
        );
    }
}
