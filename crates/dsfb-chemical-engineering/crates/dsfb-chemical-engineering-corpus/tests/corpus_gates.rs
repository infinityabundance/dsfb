//! Validation-gate, provenance-discipline, and hash-determinism tests for the soft-sensor corpus.

use corpus::seed::SOFT_SENSOR_DATASETS;
use dsfb_chemical_engineering_corpus as corpus;

#[test]
fn validate_admits_the_catalogue() {
    let r = corpus::validate().expect("soft-sensor catalogue validates");
    assert!(r.n_datasets >= 20, "a broad catalogue");
    assert!(
        r.n_cheap_sensor >= 15,
        "cheap-sensor datasets dominate the catalogue"
    );
    assert!(r.n_domains >= 6, "broad process-domain spread");
    assert!(r.n_with_fault_labels >= 3);
}

#[test]
fn every_dataset_is_sourced_and_never_redistributed() {
    for d in SOFT_SENSOR_DATASETS {
        assert!(
            !d.source.citation_key.is_empty(),
            "{} unsourced",
            d.dataset_id
        );
        assert!(d.source.url.starts_with("http"), "{} bad url", d.dataset_id);
        assert!(!d.source.license.is_empty(), "{} no licence", d.dataset_id);
        assert!(
            !d.redistributed,
            "{} must not redistribute bytes",
            d.dataset_id
        );
    }
}

#[test]
fn canonical_benchmarks_present_and_ids_unique() {
    for id in ["sru", "debutanizer", "tep", "mining_flotation", "ccpp"] {
        assert!(
            SOFT_SENSOR_DATASETS.iter().any(|d| d.dataset_id == id),
            "missing {id}"
        );
    }
    for (i, d) in SOFT_SENSOR_DATASETS.iter().enumerate() {
        for e in &SOFT_SENSOR_DATASETS[i + 1..] {
            assert_ne!(d.dataset_id, e.dataset_id, "duplicate dataset_id");
        }
    }
}

#[test]
fn spectroscopy_inputs_are_flagged_not_cheap() {
    // The NIR/Raman-input datasets must be honestly flagged as NOT cheap-sensor.
    for id in ["tecator_nir", "corn_nir"] {
        let d = SOFT_SENSOR_DATASETS
            .iter()
            .find(|d| d.dataset_id == id)
            .unwrap();
        assert!(
            !d.cheap_sensor,
            "{id} is a spectroscopy input; must be flagged cheap_sensor=false"
        );
    }
}

#[test]
fn corpus_hash_v1_is_deterministic_and_frozen() {
    let a = corpus::corpus_hash_v1();
    let b = corpus::corpus_hash_v1();
    assert_eq!(a, b, "deterministic");
    assert_eq!(a.len(), 64);
    assert_eq!(a, CORPUS_HASH_V1_EXPECTED);
}

// Re-frozen in P53 (was 1a5f4c31…) when the four provenance-classification tiers were folded into the
// canonical preimage. A deliberate, documented re-freeze — see hashes.rs / PROJECT_PLAN P53.
const CORPUS_HASH_V1_EXPECTED: &str =
    "7ce33a2e1ce1a11893b0651d448f59247f2f954d3d43288500ab6f62d9ddb748";

#[test]
fn classification_census_partitions_every_record() {
    // Each of the four P53 disclosure axes must classify EVERY record exactly once (no gaps, no
    // double-count): every per-axis subtotal equals n_datasets. This is the exhaustiveness gate that
    // keeps the README/paper census honest as the catalogue grows.
    let c = corpus::census();
    assert_eq!(c.n_datasets, SOFT_SENSOR_DATASETS.len());
    assert_eq!(
        c.license_total(),
        c.n_datasets,
        "license tiers must partition all records"
    );
    assert_eq!(
        c.access_total(),
        c.n_datasets,
        "access tiers must partition all records"
    );
    assert_eq!(
        c.redistribution_total(),
        c.n_datasets,
        "redistribution tiers must partition all records"
    );
    assert_eq!(
        c.authority_total(),
        c.n_datasets,
        "authority tiers must partition all records"
    );
}

#[test]
fn gated_datasets_are_prohibited_from_redistribution() {
    // Honesty invariant: any dataset behind a data-use agreement (SWaT/WADI) must be marked both
    // AgreementGoverned + AgreementRequired + ProhibitedByAgreement, and never redistributed.
    use corpus::{AccessConfidence, LicenseConfidence, RedistributionPolicy};
    for id in ["swat", "wadi"] {
        let d = SOFT_SENSOR_DATASETS
            .iter()
            .find(|d| d.dataset_id == id)
            .unwrap();
        assert!(!d.redistributed, "{id} must not redistribute bytes");
        assert_eq!(
            d.license_confidence,
            LicenseConfidence::AgreementGoverned,
            "{id} licence tier"
        );
        assert_eq!(
            d.access_confidence,
            AccessConfidence::AgreementRequired,
            "{id} access tier"
        );
        assert_eq!(
            d.redistribution_policy,
            RedistributionPolicy::ProhibitedByAgreement,
            "{id} redistribution tier"
        );
    }
}
