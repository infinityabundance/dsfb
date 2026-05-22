//! S1.1 acceptance tests: domain tags + axis bindings.
//!
//! Panel-required tests in this file:
//!
//! - `domain_tagset_roundtrips_bits`
//! - `axis_binding_requires_at_least_one_axis`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_registry::{AxisBinding, DomainTag, DomainTagSet};

#[test]
fn domain_tagset_roundtrips_bits() {
    // Build a tag set explicitly via DomainTag::bit() and round-
    // trip through DomainTagSet::from_raw / to_raw.
    let bits = DomainTag::Debug.bit() | DomainTag::TimeSeries.bit() | DomainTag::Industrial.bit();
    let s = DomainTagSet::from_raw(bits);
    assert_eq!(s.to_raw(), bits, "DomainTagSet must round-trip raw bits");
    assert!(s.contains(DomainTag::Debug));
    assert!(s.contains(DomainTag::TimeSeries));
    assert!(s.contains(DomainTag::Industrial));
    assert!(!s.contains(DomainTag::Medical));
    // EMPTY must round-trip.
    let empty = DomainTagSet::EMPTY;
    assert_eq!(empty.to_raw(), 0);
    assert!(empty.is_empty());
}

#[test]
fn domain_tagset_bit_positions_match_corpus_byte_for_byte() {
    // The registry's DomainTag bit positions must equal the
    // corpus crate's DomainTagSet bit positions byte-for-byte
    // so an `origin_domains` field from the corpus can be
    // passed through to a `DetectorSpec::domain_tags` field
    // without translation.
    use dsfb_gpu_atlas_corpus::types::DomainTagSet as CorpusDomainTagSet;
    assert_eq!(DomainTag::Debug.bit(), CorpusDomainTagSet::DEBUG);
    assert_eq!(DomainTag::Telemetry.bit(), CorpusDomainTagSet::TELEMETRY);
    assert_eq!(DomainTag::Tabular.bit(), CorpusDomainTagSet::TABULAR);
    assert_eq!(DomainTag::TimeSeries.bit(), CorpusDomainTagSet::TIME_SERIES);
    assert_eq!(DomainTag::Graph.bit(), CorpusDomainTagSet::GRAPH);
    assert_eq!(DomainTag::Industrial.bit(), CorpusDomainTagSet::INDUSTRIAL);
    assert_eq!(
        DomainTag::Categorical.bit(),
        CorpusDomainTagSet::CATEGORICAL
    );
    assert_eq!(
        DomainTag::Missingness.bit(),
        CorpusDomainTagSet::MISSINGNESS
    );
    assert_eq!(
        DomainTag::EventStream.bit(),
        CorpusDomainTagSet::EVENT_STREAM
    );
    assert_eq!(DomainTag::Medical.bit(), CorpusDomainTagSet::MEDICAL);
    assert_eq!(DomainTag::RfComms.bit(), CorpusDomainTagSet::RF_COMMS);
    assert_eq!(
        DomainTag::Chemometrics.bit(),
        CorpusDomainTagSet::CHEMOMETRICS
    );
    assert_eq!(DomainTag::Database.bit(), CorpusDomainTagSet::DATABASE);
}

#[test]
fn axis_binding_requires_at_least_one_axis() {
    let empty = AxisBinding(0);
    assert!(empty.is_empty(), "AxisBinding(0) must be empty");
    let one = AxisBinding::single(AxisBinding::AXIS_1_RESIDUAL_MAGNITUDE);
    assert!(!one.is_empty(), "single-bit AxisBinding must not be empty");
}
