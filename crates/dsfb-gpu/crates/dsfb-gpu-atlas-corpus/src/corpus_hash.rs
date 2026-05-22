//! T.10 — `corpus_hash_v1` definition and canonical-bytes
//! material writer.
//!
//! **Panel-locked scope (T.10 is freeze of T.1-T.9 only)**:
//!
//! T.10 defines `corpus_hash_v1` (a 32-byte SHA-256 over a
//! canonical byte projection of the live T.1-T.9 corpus
//! material) and exposes it via [`compute_corpus_hash_v1`]. It
//! does NOT:
//!
//! - Generate `registry_hash_v2` (S1.2 work).
//! - Emit the 2,000-detector Atlas registry (S1.2 work).
//! - Run Atlas family kernels on GPU (S1.4+ work).
//! - Publish or deposit anything externally.
//! - Change the R.13 D64 headline.
//! - Change any D16 / D64 / D128 / D205 golden hash.
//!
//! **The corpus_hash_v1 is hashed over canonical bytes, NOT over
//! rendered TXT or JSON.** The material writer below produces a
//! length-prefixed big-endian byte stream that depends only on
//! the structural T.1-T.9 content (records, court decisions,
//! genealogy edges, witness roles, fusion axes, L-band states,
//! lifecycle states, usefulness ledger). Re-rendering the public
//! reports does NOT change the hash; mutating any T.1-T.9
//! payload byte WILL change the hash.
//!
//! Material order (panel-locked):
//!
//! 1. Schema + version strings.
//! 2. Per-record canonical fields, sorted by `canonical_id`.
//! 3. Dedup-court decisions, sorted by subject.
//! 4. Source references, sorted by `citation_key`.
//! 5. Genealogy edges (per-record, in canonical edge-kind order).
//! 6. Witness-role + fusion-axis bindings (per-record).
//! 7. L-band states (per-record).
//! 8. Lifecycle states (per-record, derived from snapshot).
//! 9. Usefulness-ledger rows, sorted by triple
//!    `(canonical_id, task_id, dataset_id)`.
//!
//! Every string uses a 4-byte big-endian length prefix; every
//! integer field is written in network byte order. Enum values
//! use their declared `as_str()` wire names (NOT `Debug` output)
//! so a future Rust-version rename of a variant cannot silently
//! shift the hash.

extern crate alloc;
use alloc::vec::Vec;

use crate::claims::CLAIMS;
use crate::court::classify_all;
use crate::seed::SEED;
use crate::types::{
    CanonicalisationDecision, DedupReason, DedupSubject, GenealogyEdges, LiteratureDetector,
    SourceRef, UsefulnessLedgerSnapshot,
};
use crate::usefulness::USEFULNESS_LEDGER;
use dsfb_gpu_debug_core::sha256;

/// Domain separator prefix for the corpus hash. **Panel-locked**;
/// changing it produces a different hash on the same corpus.
pub const CORPUS_HASH_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0";

/// Schema identifier carried in the hash material. Identifies
/// which projection of T.1-T.9 the hash is over.
pub const CORPUS_HASH_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:CORPUS-HASH-SCHEMA:v1";

/// `corpus_hash_v1` — a 32-byte SHA-256 commitment to the live
/// T.1-T.9 corpus material. Two builds on different machines
/// produce the same value because the material writer depends
/// only on the canonical schema bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorpusHashV1 {
    /// Raw SHA-256 digest bytes.
    pub bytes: [u8; 32],
}

impl CorpusHashV1 {
    /// True if the hash is the all-zero sentinel. Callers
    /// should never construct this in production; the
    /// `CaseFileV2Header` verifier rejects zero corpus hashes.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.bytes[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Render as a 64-character lowercase hex string.
    #[must_use]
    pub fn to_hex(&self) -> alloc::string::String {
        use core::fmt::Write;
        let mut out = alloc::string::String::with_capacity(64);
        for b in &self.bytes {
            let _ = write!(out, "{b:02x}");
        }
        out
    }
}

// =====================================================================
// Canonical byte-projection helpers. Length-prefixed strings, big-endian
// integers, no Debug output, no rendered report bytes.
// =====================================================================

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn write_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_section_label(out: &mut Vec<u8>, label: &str) {
    // Section labels are 4-byte length-prefixed strings; they
    // exist to make the byte stream auditable but they do not
    // need their own framing beyond the length prefix.
    write_str(out, "===SECTION===");
    write_str(out, label);
}

fn write_source_ref(out: &mut Vec<u8>, sr: &SourceRef) {
    write_str(out, sr.citation_key);
    write_str(out, sr.title);
    write_str(out, sr.authors);
    write_u16(out, sr.year);
    write_str(out, sr.venue_or_source);
    match sr.doi_or_url {
        Some(s) => {
            write_u8(out, 1);
            write_str(out, s);
        }
        None => write_u8(out, 0),
    }
    write_str(out, sr.notes);
}

fn write_genealogy(out: &mut Vec<u8>, g: &GenealogyEdges) {
    let label = b"derived_from";
    write_str(out, core::str::from_utf8(label).unwrap_or(""));
    write_u32(out, g.derived_from.len() as u32);
    for id in g.derived_from {
        write_u32(out, id.0);
    }
    write_str(out, "generalizes");
    write_u32(out, g.generalizes.len() as u32);
    for id in g.generalizes {
        write_u32(out, id.0);
    }
    write_str(out, "special_case_of");
    write_u32(out, g.special_case_of.len() as u32);
    for id in g.special_case_of {
        write_u32(out, id.0);
    }
    write_u8(out, u8::from(g.is_origin));
}

fn write_snapshot(out: &mut Vec<u8>, s: &UsefulnessLedgerSnapshot) {
    write_i64(out, s.unique_episode_gain);
    write_i64(out, s.clean_window_false_positive_cost);
    write_i64(out, s.confuser_reduction_gain);
    write_u32(out, s.runtime_cost_us_p50);
    write_u64(out, s.memory_cost_bytes);
    write_u8(out, s.operator_readability_score);
    write_u64(out, s.sample_count);
}

fn write_record(out: &mut Vec<u8>, r: &LiteratureDetector) {
    write_u32(out, r.canonical_id.0);
    write_str(out, r.display_name);
    write_u32(out, r.aliases.len() as u32);
    for alias in r.aliases {
        write_str(out, alias);
    }
    write_u32(out, r.source_refs.len() as u32);
    for sr in r.source_refs {
        write_source_ref(out, sr);
    }
    write_u16(out, r.origin_domains.0);
    write_str(out, r.primitive_family.as_str());
    write_str(out, r.mathematical_form.as_str());
    write_str(out, r.decision_functional.as_str());
    // Input requirements as raw bit set (no Debug). u32 width.
    write_u32(out, r.input_requirements.0);
    write_str(out, r.output_witness.as_str());
    write_str(out, r.witness_role.as_str());
    write_str(out, r.negative_witness_kind.as_str());
    write_u16(out, r.fusion_axes.0);
    write_str(out, r.confuser_profile.as_str());
    write_str(out, r.deterministic_status.as_str());
    write_str(out, r.implementation_status.as_str());
    write_str(out, r.gpu_family.as_str());
    // Parameter bounds carries a textual description; include it
    // for completeness but tagged as "parameter_bounds".
    write_str(out, "parameter_bounds");
    write_str(out, r.parameter_bounds.description);
    write_u32(out, r.duplicate_group.0);
    write_str(out, "genealogy");
    write_genealogy(out, &r.genealogy);
    write_str(out, "snapshot");
    write_snapshot(out, &r.usefulness);
    write_str(out, r.lifecycle_state.as_str());
    // Constitution flags as 8 booleans.
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_input_contract),
    );
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_output_type),
    );
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_deterministic_form),
    );
    write_u8(out, u8::from(r.constitution_compliance.declared_provenance));
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_equivalence_status),
    );
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_witness_role),
    );
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_activation_conditions),
    );
    write_u8(
        out,
        u8::from(r.constitution_compliance.declared_failure_confuser_modes),
    );
}

fn write_court_decision(out: &mut Vec<u8>, d: &CanonicalisationDecision) {
    match d {
        CanonicalisationDecision::Canonical => write_str(out, "Canonical"),
        CanonicalisationDecision::AliasOf(id) => {
            write_str(out, "AliasOf");
            write_u32(out, id.0);
        }
        CanonicalisationDecision::ParameterisationOf(id) => {
            write_str(out, "ParameterisationOf");
            write_u32(out, id.0);
        }
        CanonicalisationDecision::CompositionOf(ids) => {
            write_str(out, "CompositionOf");
            write_u32(out, ids.len() as u32);
            for id in *ids {
                write_u32(out, id.0);
            }
        }
        CanonicalisationDecision::StochasticOriginalDeterministicReduction(id) => {
            write_str(out, "StochasticOriginalDeterministicReduction");
            write_u32(out, id.0);
        }
        CanonicalisationDecision::RejectedNotDeterministic => {
            write_str(out, "RejectedNotDeterministic");
        }
        CanonicalisationDecision::RejectedNotDetector => {
            write_str(out, "RejectedNotDetector");
        }
        CanonicalisationDecision::DeferredNeedsReview => {
            write_str(out, "DeferredNeedsReview");
        }
    }
}

fn dedup_reason_wire_name(r: DedupReason) -> &'static str {
    // Hand-pinned wire names so a future variant rename cannot
    // silently shift the corpus_hash_v1.
    match r {
        DedupReason::SameFormulaSameParametersSameContract => {
            "SameFormulaSameParametersSameContract"
        }
        DedupReason::SameFormulaDifferentParameters => "SameFormulaDifferentParameters",
        DedupReason::DifferentFormulaSameDomain => "DifferentFormulaSameDomain",
        DedupReason::SameFormulaDifferentInputContract => "SameFormulaDifferentInputContract",
        DedupReason::SameFormulaDifferentWitnessRole => "SameFormulaDifferentWitnessRole",
        DedupReason::DifferentDecisionFunctional => "DifferentDecisionFunctional",
        DedupReason::DeterministicReductionOfStochastic => "DeterministicReductionOfStochastic",
        DedupReason::CompositionOfCanonicals => "CompositionOfCanonicals",
        DedupReason::OriginRecord => "OriginRecord",
    }
}

fn write_subject(out: &mut Vec<u8>, subject: DedupSubject) {
    match subject {
        DedupSubject::Canonical(c) => {
            write_str(out, "Canonical");
            write_u32(out, c.0);
        }
        DedupSubject::AliasClaim(a) => {
            write_str(out, "AliasClaim");
            write_u32(out, a.0);
        }
    }
}

// =====================================================================
// Main material writer.
// =====================================================================

/// Write the canonical T.1-T.9 corpus hash material into `out`.
/// Two calls produce byte-identical output. The output is hashed
/// (with the panel-locked domain-separator prefix) to produce
/// [`compute_corpus_hash_v1`].
#[allow(clippy::too_many_lines)]
pub fn write_corpus_hash_material_v1(out: &mut Vec<u8>) {
    // Section A: schema + version strings.
    write_section_label(out, "SCHEMA");
    write_str(out, CORPUS_HASH_SCHEMA_V1);
    write_str(out, "T.1-T.9 frozen");

    // Section B: canonical detector records (sorted by canonical_id).
    write_section_label(out, "RECORDS");
    let mut sorted_records: Vec<&LiteratureDetector> = SEED.iter().collect();
    sorted_records.sort_by_key(|r| r.canonical_id.0);
    write_u32(out, sorted_records.len() as u32);
    for r in &sorted_records {
        write_record(out, r);
    }

    // Section C: dedup-court decisions, sorted by subject id.
    write_section_label(out, "COURT");
    let mut court_records = classify_all();
    court_records.sort_by_key(|cr| match cr.subject {
        DedupSubject::Canonical(c) => (0u8, c.0),
        DedupSubject::AliasClaim(a) => (1u8, a.0),
    });
    write_u32(out, court_records.len() as u32);
    for cr in &court_records {
        write_subject(out, cr.subject);
        write_str(out, cr.literature_name);
        write_court_decision(out, &cr.decision);
        write_str(out, dedup_reason_wire_name(cr.reason_code));
        write_str(out, cr.notes);
    }

    // Section D: alias-claim provenance (the raw CLAIMS table,
    // sorted by alias_id). The fields hashed are panel-locked
    // at the T.4 first-batch shape: alias_id, literature_name,
    // decision, reason_code, notes.
    write_section_label(out, "ALIAS_CLAIMS");
    let mut sorted_claims: Vec<_> = CLAIMS.iter().collect();
    sorted_claims.sort_by_key(|c| c.alias_id.0);
    write_u32(out, sorted_claims.len() as u32);
    for c in &sorted_claims {
        write_u32(out, c.alias_id.0);
        write_str(out, c.literature_name);
        write_court_decision(out, &c.decision);
        write_str(out, dedup_reason_wire_name(c.reason_code));
        write_str(out, c.notes);
    }

    // Section E: usefulness-ledger rows, sorted by triple
    // (canonical_id, task_id, dataset_id).
    write_section_label(out, "USEFULNESS");
    let mut sorted_rows: Vec<_> = USEFULNESS_LEDGER.iter().collect();
    sorted_rows.sort_by(|a, b| {
        (a.canonical_id.0, a.task_id.0, a.dataset_id.0).cmp(&(
            b.canonical_id.0,
            b.task_id.0,
            b.dataset_id.0,
        ))
    });
    write_u32(out, sorted_rows.len() as u32);
    for r in &sorted_rows {
        write_u32(out, r.canonical_id.0);
        write_str(out, r.task_id.0);
        write_u16(out, r.domain.0);
        write_str(out, r.dataset_id.0);
        write_str(out, r.evidence_level.as_str());
        write_str(out, r.lifecycle_state.as_str());
        write_str(out, r.score_kind.as_str());
        write_i64(out, r.unique_episode_gain);
        write_u32(out, r.redundant_with_count);
        write_i64(out, r.clean_window_false_positive_cost);
        write_i64(out, r.confuser_reduction_gain);
        write_u32(out, r.runtime_cost_us_p50);
        write_u64(out, r.memory_cost_bytes);
        write_i32(out, r.casefile_explanation_value);
        write_i32(out, r.operator_readability_score);
        write_u64(out, r.sample_count);
        write_str(out, r.ledger_source.as_str());
        write_str(out, r.reason_code.as_str());
    }

    // Section F: terminator label.
    write_section_label(out, "END");
}

/// Compute `corpus_hash_v1` over the canonical-byte projection
/// of the live T.1-T.9 corpus. Deterministic across two builds.
#[must_use]
pub fn compute_corpus_hash_v1() -> CorpusHashV1 {
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    buf.extend_from_slice(CORPUS_HASH_DOMAIN_V1.as_bytes());
    write_corpus_hash_material_v1(&mut buf);
    CorpusHashV1 {
        bytes: sha256(&buf),
    }
}
