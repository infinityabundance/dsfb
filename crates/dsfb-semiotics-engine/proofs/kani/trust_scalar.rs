//! Kani proof harnesses for bounded DSFB invariants.
//!
//! These harnesses deliberately cover small, explicit properties rather than implying whole-engine
//! formal verification.

#![cfg(kani)]

use dsfb_semiotics_engine::engine::types::{GrammarReasonCode, TrustScalar};

#[kani::proof]
fn proof_trust_scalar_in_unit_interval() {
    let value: f64 = kani::any();
    let trust = TrustScalar::new(value).value();
    assert!((0.0..=1.0).contains(&trust));
}

#[kani::proof]
fn proof_trust_scalar_not_nan() {
    let value: f64 = kani::any();
    let trust = TrustScalar::new(value).value();
    assert!(!trust.is_nan());
}

#[kani::proof]
fn proof_grammar_reason_enum_is_closed() {
    let codes = [
        GrammarReasonCode::Admissible,
        GrammarReasonCode::Boundary,
        GrammarReasonCode::RecurrentBoundaryGrazing,
        GrammarReasonCode::SustainedOutwardDrift,
        GrammarReasonCode::AbruptSlewViolation,
        GrammarReasonCode::EnvelopeViolation,
    ];
    for code in codes {
        match code {
            GrammarReasonCode::Admissible
            | GrammarReasonCode::Boundary
            | GrammarReasonCode::RecurrentBoundaryGrazing
            | GrammarReasonCode::SustainedOutwardDrift
            | GrammarReasonCode::AbruptSlewViolation
            | GrammarReasonCode::EnvelopeViolation => {}
        }
    }
}
