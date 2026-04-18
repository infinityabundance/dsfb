//! Adapter / motif round-trip invariants.
//!
//! For each dataset adapter, the deterministic exemplar must:
//!   1. Produce a non-empty residual stream (or, in the case of TPC-DS
//!      "clean baseline", a non-empty stream with no perturbations).
//!   2. Produce a residual stream whose fingerprint is bytewise stable.
//!   3. Produce a residual stream whose channels are non-empty strings
//!      (no `_anonymous_`-only adapters).
//!
//! The CEB exemplar is the only one with a deliberate cardinality drift
//! and is asserted to emit at least one cardinality episode. The TPC-DS
//! exemplar is a clean baseline (the perturbation pipeline lives in
//! `crate::perturbation`) and is asserted to emit *zero* episodes — the
//! negative-control demonstration.

use dsfb_database::adapters::{
    ceb::Ceb, job::Job, snowset::Snowset, sqlshare::SqlShare, tpcds::TpcDs, DatasetAdapter,
};
use dsfb_database::grammar::{MotifClass, MotifEngine, MotifGrammar};

fn assert_stream_well_formed<A: DatasetAdapter>(adapter: &A) {
    let s1 = adapter.exemplar(7);
    let s2 = adapter.exemplar(7);
    assert_eq!(
        s1.fingerprint(),
        s2.fingerprint(),
        "{} exemplar must be deterministic under fixed seed",
        adapter.name()
    );
    assert!(
        !s1.is_empty(),
        "{} exemplar must produce a non-empty residual stream",
        adapter.name()
    );
    for sample in s1.samples.iter() {
        if let Some(ch) = &sample.channel {
            assert!(
                !ch.is_empty(),
                "{} produced an empty-string channel name",
                adapter.name()
            );
        }
    }
}

#[test]
fn snowset_exemplar_well_formed() {
    assert_stream_well_formed(&Snowset);
}

#[test]
fn sqlshare_exemplar_well_formed() {
    assert_stream_well_formed(&SqlShare);
}

#[test]
fn ceb_exemplar_well_formed() {
    assert_stream_well_formed(&Ceb);
}

#[test]
fn job_exemplar_well_formed() {
    assert_stream_well_formed(&Job);
}

#[test]
fn tpcds_exemplar_well_formed() {
    assert_stream_well_formed(&TpcDs);
}

#[test]
fn ceb_exemplar_emits_cardinality_episode() {
    let s = Ceb.exemplar(7);
    let episodes = MotifEngine::new(MotifGrammar::default()).run(&s);
    let cardinality_eps: Vec<_> = episodes
        .iter()
        .filter(|e| e.motif == MotifClass::CardinalityMismatchRegime)
        .collect();
    assert!(
        !cardinality_eps.is_empty(),
        "CEB exemplar (sp7 30x mismatch) must produce at least one cardinality episode"
    );
}

#[test]
fn tpcds_clean_exemplar_emits_no_episodes() {
    // The TPC-DS exemplar is a *clean baseline* — perturbation injection
    // lives in `crate::perturbation::tpcds_with_perturbations`. A clean
    // baseline must produce zero episodes; any false alarm here is a
    // grammar-thresholding regression.
    let s = TpcDs.exemplar(7);
    let episodes = MotifEngine::new(MotifGrammar::default()).run(&s);
    assert!(
        episodes.is_empty(),
        "clean TPC-DS exemplar must produce zero episodes; got {} (false-alarm regression)",
        episodes.len()
    );
}
