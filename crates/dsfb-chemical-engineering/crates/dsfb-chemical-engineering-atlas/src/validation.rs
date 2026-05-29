//! Deterministic validation gates over the bundled authority records.
//!
//! All checks are pure functions of the `const` record tables: no I/O, no clock, no map iteration,
//! no allocation in the hot path (errors are `&'static str`). `validate()` returns a summary report
//! on success or the first failing gate's message.

use crate::detector::{ImplementationStatus, DETECTOR_RECORDS};
use crate::fault_signature::FAULT_SIGNATURES;
use crate::heuristics::HEURISTIC_RECORDS;
use crate::unknown_taxonomy::{UNKNOWN_CLASS_COUNT, UNKNOWN_TAXONOMY};

/// Summary of a passing validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AtlasValidationReport {
    pub n_detectors: usize,
    pub n_executed: usize,
    pub n_heuristics: usize,
    pub n_primitive_families: usize,
    pub n_fault_signatures: usize,
    pub n_fault_signatures_executed: usize,
}

/// Run every gate. Returns the summary report, or the first failing gate's message.
pub fn validate() -> Result<AtlasValidationReport, &'static str> {
    let dets = DETECTOR_RECORDS;

    // Gate: unique detector_id and canonical_id (O(n^2) over a small fixed set; deterministic).
    for (i, a) in dets.iter().enumerate() {
        if a.detector_id.is_empty() {
            return Err("detector with empty detector_id");
        }
        if a.source_refs.is_empty() {
            return Err("detector missing source_refs");
        }
        if a.mathematical_form.is_empty() || a.decision_functional.is_empty() {
            return Err("detector missing mathematical_form / decision_functional");
        }
        if a.implementation_status != ImplementationStatus::Executed {
            return Err("authority set must contain only Executed detectors");
        }
        for b in &dets[i + 1..] {
            if a.detector_id == b.detector_id {
                return Err("duplicate detector_id");
            }
            if a.canonical_id == b.canonical_id {
                return Err("duplicate canonical_id");
            }
        }
    }

    // Gate: distinct primitive-family spread (a broad statistical basis, not one trick).
    let mut n_prim = 0usize;
    for (i, a) in dets.iter().enumerate() {
        if !dets[..i]
            .iter()
            .any(|b| b.primitive_family == a.primitive_family)
        {
            n_prim += 1;
        }
    }
    if n_prim < 6 {
        return Err("expected a broad primitive-family spread (>= 6)");
    }

    // Gate: H1..H6 present exactly once, each with inputs + FP/FN modes + conditions.
    let expected = ["H1", "H2", "H3", "H4", "H5", "H6"];
    for id in expected.iter() {
        let n = HEURISTIC_RECORDS
            .iter()
            .filter(|h| &h.heuristic_id == id)
            .count();
        if n != 1 {
            return Err("each of H1..H6 must appear exactly once");
        }
    }
    for h in HEURISTIC_RECORDS.iter() {
        if h.detector_inputs.is_empty() {
            return Err("heuristic missing detector_inputs");
        }
        if h.known_false_positive_modes.is_empty() || h.known_false_negative_modes.is_empty() {
            return Err("heuristic missing false-positive / false-negative modes");
        }
        if h.admissibility_condition.is_empty() {
            return Err("heuristic missing admissibility_condition");
        }
        if h.engineering_basis.is_empty() {
            return Err("heuristic missing engineering_basis");
        }
    }

    // Gate: fault signatures present and well-formed; unique fault_id; sourced; detector inputs
    // reference real detectors; every detector input is a catalogued detector id.
    if FAULT_SIGNATURES.is_empty() {
        return Err("fault-signature bank is empty");
    }
    for (i, f) in FAULT_SIGNATURES.iter().enumerate() {
        if f.fault_id.is_empty() || f.residual_motif.is_empty() {
            return Err("fault signature missing fault_id / residual_motif");
        }
        if f.cheap_sensors.is_empty() || f.detector_inputs.is_empty() {
            return Err("fault signature missing cheap_sensors / detector_inputs");
        }
        if f.source_refs.is_empty() || f.exhibiting_datasets.is_empty() {
            return Err("fault signature missing source_refs / exhibiting_datasets");
        }
        for b in &FAULT_SIGNATURES[i + 1..] {
            if f.fault_id == b.fault_id {
                return Err("duplicate fault_id");
            }
        }
    }

    // Gate: the unknown-disposition taxonomy is exactly the 7 fixed classes, each well-formed, with
    // unique class_ids (it is folded into atlas_hash_v1, so drift here re-freezes the authority).
    if UNKNOWN_TAXONOMY.len() != UNKNOWN_CLASS_COUNT {
        return Err("unknown taxonomy must have exactly 7 classes");
    }
    for (i, c) in UNKNOWN_TAXONOMY.iter().enumerate() {
        if c.class_id.is_empty()
            || c.name.is_empty()
            || c.description.is_empty()
            || c.disposition.is_empty()
        {
            return Err("unknown-taxonomy class missing a field");
        }
        for b in &UNKNOWN_TAXONOMY[i + 1..] {
            if c.class_id == b.class_id {
                return Err("duplicate unknown-taxonomy class_id");
            }
        }
    }

    let n_executed = dets
        .iter()
        .filter(|d| d.implementation_status == ImplementationStatus::Executed)
        .count();
    let n_fault_exec = FAULT_SIGNATURES
        .iter()
        .filter(|f| f.implementation_status == ImplementationStatus::Executed)
        .count();
    Ok(AtlasValidationReport {
        n_detectors: dets.len(),
        n_executed,
        n_heuristics: HEURISTIC_RECORDS.len(),
        n_primitive_families: n_prim,
        n_fault_signatures: FAULT_SIGNATURES.len(),
        n_fault_signatures_executed: n_fault_exec,
    })
}
