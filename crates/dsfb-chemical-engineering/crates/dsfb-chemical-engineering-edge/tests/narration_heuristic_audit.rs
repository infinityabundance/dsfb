//! Audit for the H7–H12 narration-failure detector (panel narrator-generator extension).
//!
//! Proves the catalogued contract is mechanically enforceable: each demonstrator case trips EXACTLY its injected
//! heuristic, the faithful baseline (safe templates) trips none, the detector report is byte-verifiable + deterministic,
//! and the emitted report names no consumer. Feature-gated because it uses the synthetic adversarial constructors.
#![cfg(feature = "narration-heuristic-demo")]

use dsfb_chemical_engineering_edge::narration_heuristic_demo::{demo_cases, render_report};
use dsfb_chemical_engineering_edge::narration_heuristics::detect;
use std::collections::BTreeSet;

#[test]
fn each_case_trips_exactly_its_target_heuristic() {
    for case in demo_cases() {
        let r = detect(&case.narrative, &case.ctx);
        assert!(
            r.verify(),
            "detector report must be byte-verifiable for '{}'",
            case.name
        );
        let fired: BTreeSet<String> = r.hits.iter().map(|h| h.heuristic_id.clone()).collect();
        match case.injected {
            None => assert!(
                r.clean && fired.is_empty(),
                "the faithful baseline must trip no heuristic, got {fired:?}"
            ),
            Some(id) => {
                let expected: BTreeSet<String> = [id.to_string()].into_iter().collect();
                assert_eq!(
                    fired, expected,
                    "case '{}' must trip exactly {id} (and nothing else), got {fired:?}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn detector_is_deterministic() {
    for case in demo_cases() {
        let a = detect(&case.narrative, &case.ctx);
        let b = detect(&case.narrative, &case.ctx);
        assert_eq!(
            a.detector_hash, b.detector_hash,
            "detector_hash must be deterministic"
        );
        assert_eq!(a.detector_hash.len(), 64);
    }
}

#[test]
fn demonstrator_report_names_no_consumer() {
    let low = render_report().to_lowercase();
    assert!(!low.contains("llm"), "report must not name LLM");
    assert!(
        !low.contains("language model"),
        "report must not name a language model"
    );
    assert!(
        !low.split(|c: char| !c.is_ascii_alphabetic())
            .any(|w| w == "ai"),
        "report must not name AI"
    );
}
