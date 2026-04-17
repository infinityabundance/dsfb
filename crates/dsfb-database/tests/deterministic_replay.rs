//! Deterministic-replay invariant.
//!
//! The DSFB-Database paper's reproducibility claim is that, for any fixed
//! seed and grammar, the SHA256 fingerprint of (a) the residual stream and
//! (b) the emitted episode stream is bytewise identical across runs and
//! across machines (modulo IEEE-754 rounding parity, which we do not
//! perturb). This file pins that invariant so a future refactor cannot
//! quietly break replay.

use dsfb_database::adapters::{ceb::Ceb, DatasetAdapter};
use dsfb_database::grammar::{replay, MotifEngine, MotifGrammar};
use dsfb_database::perturbation::tpcds_with_perturbations;

#[test]
fn residual_stream_is_deterministic() {
    let (s1, _) = tpcds_with_perturbations(42);
    let (s2, _) = tpcds_with_perturbations(42);
    assert_eq!(
        s1.fingerprint(),
        s2.fingerprint(),
        "residual stream must be bytewise identical under fixed seed"
    );
}

#[test]
fn residual_stream_differs_under_different_seeds() {
    let (s1, _) = tpcds_with_perturbations(42);
    let (s2, _) = tpcds_with_perturbations(43);
    assert_ne!(
        s1.fingerprint(),
        s2.fingerprint(),
        "different seeds must produce different residual streams"
    );
}

#[test]
fn episode_stream_is_deterministic() {
    let (s, _) = tpcds_with_perturbations(42);
    let g = MotifGrammar::default();
    let e1 = MotifEngine::new(g.clone()).run(&s);
    let e2 = MotifEngine::new(g).run(&s);
    assert_eq!(
        replay::fingerprint(&e1),
        replay::fingerprint(&e2),
        "episode stream must be bytewise identical for the same input"
    );
}

#[test]
fn paper_fingerprint_is_pinned() {
    // This is the SHA256 of the canonical TPC-DS perturbed residual stream
    // at seed=42. It is the value cited verbatim in §8 of the paper. If
    // you change the perturbation harness, the residual schema, or the
    // serialisation of `ResidualSample`, this test will fail and you must
    // (a) re-derive the value, (b) update the paper, (c) explain the
    // change in the changelog.
    let (s, _) = tpcds_with_perturbations(42);
    let hex = s
        .fingerprint()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let expected = "c1b64dac658acf66e673743c27ff0ce08f020013b741efbffa6e6f3fea2af72a";
    assert_eq!(
        hex, expected,
        "stream fingerprint changed; update paper §8 if intended"
    );
}

#[test]
fn paper_episode_fingerprint_is_pinned() {
    let (s, _) = tpcds_with_perturbations(42);
    let g = MotifGrammar::default();
    let e = MotifEngine::new(g).run(&s);
    let hex = replay::fingerprint_hex(&e);
    let expected = "ac28aeed0f54ed6d62e676b64034e3e06403858282210afc867a3ee4e494bdde";
    assert_eq!(
        hex, expected,
        "episode fingerprint changed; update paper §8 if intended"
    );
}

#[test]
fn paper_ceb_episode_fingerprint_is_pinned() {
    // Mirrors `paper_episode_fingerprint_is_pinned` for the CEB
    // (Cardinality Estimation Benchmark) exemplar, which exercises a
    // *different* motif (CardinalityMismatchRegime) on a *different*
    // residual generator. Without this pin a silent drift in the CEB
    // adapter or the cardinality state machine would be caught only by
    // visual inspection of the per-dataset PNG. Same protocol as the
    // TPC-DS pin: if it fails, re-derive, update the paper, log the
    // change.
    let stream = Ceb.exemplar(42);
    let g = MotifGrammar::default();
    let e = MotifEngine::new(g).run(&stream);
    let hex = replay::fingerprint_hex(&e);
    let expected = "0dd77cd7cdd489eadd4fd6344e8dd81f98b24b8eb3b54a65f304816a4d9865db";
    assert_eq!(
        hex, expected,
        "CEB episode fingerprint changed; update paper §8 if intended"
    );
}
