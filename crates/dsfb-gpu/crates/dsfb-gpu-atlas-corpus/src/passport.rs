//! T.11a — `DetectorPassport`: per-detector legal-identity packet.
//!
//! A passport gathers every T.1–T.10 fact about a single canonical
//! detector into one inspectable, hashable record. The panel
//! framing:
//!
//! > Turn the corpus from "internally correct" into "inspectable."
//!
//! Each passport carries:
//!
//! - identity: canonical id, display name, aliases, source refs,
//!   primitive family, mathematical form, decision functional;
//! - the five T.3 identity hashes plus the composite
//!   `detector_identity_hash`;
//! - dedup-court decision (T.4) + reason code;
//! - genealogy edges (T.5);
//! - witness role (T.6) and the T.6 fusion-plane bitset derived
//!   from the record's axis bindings via
//!   [`crate::fusion::axes_to_planes`];
//! - implementation-status band (T.7);
//! - lifecycle state + usefulness evidence level (T.8);
//! - constitution flags;
//! - a 32-byte `passport_hash` over the canonical-byte projection
//!   of every field above.
//!
//! **Panel-locked non-claims (T.11a)**:
//!
//! - The passport does NOT include S1.2 registry-spec linkage.
//!   The corpus crate stays free of a dependency on the registry
//!   crate; a follow-on commit (post-T.11a) may add a
//!   registry-aware view in a higher-level crate.
//! - The passport does NOT include the T.11d episode transcript
//!   or T.11b court precedents or T.11c admissibility grammar.
//! - The passport hash is DSFB-native; it does NOT claim
//!   in-toto / SLSA / SPDX / CycloneDX compatibility.
//!
//! Two builds against the same corpus produce byte-identical
//! passports + passport hashes. Acceptance tests in
//! `tests/passport_invariants.rs` pin determinism + sensitivity
//! to every passport field.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::court::classify_all;
use crate::fusion::{axes_to_planes, FusionPlaneSet};
use crate::identity::{compute_identity_hashes, DetectorIdentityHashes};
use crate::precedent::{
    collect_court_precedents, precedents_for_canonical, PrecedentId, PrecedentSet,
};
use crate::seed::SEED;
use crate::types::{
    CanonicalisationDecision, ConstitutionFlags, DecisionFunctional, DedupReason, DedupRecord,
    DedupSubject, DetectorCanonicalId, GenealogyEdges, ImplementationLevel, LifecycleState,
    LiteratureDetector, MathFormId, PrimitiveFamily, SourceRef, WitnessRole,
};
use crate::usefulness::{UsefulnessEvidenceLevel, USEFULNESS_LEDGER};

/// Domain separator prefix for the passport hash.
/// **Panel-locked**; changing it changes every passport hash.
pub const PASSPORT_DOMAIN: &str = "DSFB-GPU-ATLAS:DETECTOR-PASSPORT:v1\0";

/// Per-detector legal-identity packet.
///
/// Built deterministically from the static corpus by
/// [`passport_for`] / [`all_passports`]. Two builds produce
/// byte-identical passports and byte-identical passport hashes.
///
/// The struct is `Copy`-free intentionally — it carries `&'static`
/// slices (aliases / source_refs) so cloning is cheap but `Copy`
/// would be misleading.
#[derive(Debug, Clone)]
pub struct DetectorPassport {
    /// Corpus canonical handle (T.1).
    pub canonical_id: DetectorCanonicalId,
    /// Human-readable display name.
    pub display_name: &'static str,
    /// Literature aliases for this primitive.
    pub aliases: &'static [&'static str],
    /// Provenance references.
    pub source_refs: &'static [SourceRef],
    /// Primitive-family classification.
    pub primitive_family: PrimitiveFamily,
    /// Coarse mathematical-form classification.
    pub mathematical_form: MathFormId,
    /// Decision-functional shape.
    pub decision_functional: DecisionFunctional,
    /// Five T.3 identity hashes (source / formula / parameter /
    /// implementation / semantic-role) plus the composite
    /// `detector_identity_hash`.
    pub identity_hashes: DetectorIdentityHashes,
    /// Convenience copy of the composite identity hash. Equal to
    /// `identity_hashes.detector_identity_hash`; surfaced as a
    /// top-level field so renderers + readers can grab it without
    /// drilling.
    pub detector_identity_hash: [u8; 32],
    /// T.4 dedup-court decision for this canonical record.
    pub dedup_decision: CanonicalisationDecision,
    /// T.4 reason code attached to the dedup decision.
    pub dedup_reason: DedupReason,
    /// T.5 genealogy edges for this primitive.
    pub genealogy_edges: GenealogyEdges,
    /// T.6 witness role in the fusion layer.
    pub witness_role: WitnessRole,
    /// T.6 fusion-plane bitset derived from the record's
    /// `fusion_axes` via [`axes_to_planes`].
    pub fusion_planes: FusionPlaneSet,
    /// T.7 implementation-status band (L0..L8).
    pub implementation_level: ImplementationLevel,
    /// T.8 lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// T.8 usefulness evidence level (Unmeasured /
    /// LiteraturePrior / RoleSeeded / ... / RetiredByEvidence).
    /// Pulled from the matching `USEFULNESS_LEDGER` row when one
    /// exists; defaults to `Unmeasured` otherwise.
    pub usefulness_evidence_level: UsefulnessEvidenceLevel,
    /// Eight constitution flags — every flag MUST be `true` for
    /// the corpus verifier to pass.
    pub constitution_flags: ConstitutionFlags,
    /// T.11b court precedents linked to this passport, in
    /// canonical-sorted ascending order. Includes every global
    /// law (witness, L-band, usefulness, corpus-hash, registry-
    /// binding, constitution, deferred-gate) plus any per-record
    /// dedup / alias / composition / parameterisation /
    /// semantic-role precedent that touches this canonical id.
    pub linked_precedent_ids: Vec<PrecedentId>,
    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of every other field including
    /// `linked_precedent_ids`. Computed by
    /// [`compute_passport_hash`].
    pub passport_hash: [u8; 32],
}

/// Build a passport for the given canonical id, or `None` if the
/// id is not in the corpus SEED.
#[must_use]
pub fn passport_for(canonical_id: DetectorCanonicalId) -> Option<DetectorPassport> {
    let precedents = collect_court_precedents();
    SEED.iter()
        .find(|r| r.canonical_id == canonical_id)
        .map(|r| build_passport(r, &classify_all(), &precedents))
}

/// Build passports for every SEED canonical record, in
/// `canonical_id` ascending order.
#[must_use]
pub fn all_passports() -> Vec<DetectorPassport> {
    let dedup_records = classify_all();
    let precedents = collect_court_precedents();
    let mut out: Vec<DetectorPassport> = SEED
        .iter()
        .map(|r| build_passport(r, &dedup_records, &precedents))
        .collect();
    out.sort_by_key(|p| p.canonical_id.0);
    out
}

fn build_passport(
    record: &LiteratureDetector,
    dedup_records: &[DedupRecord],
    precedents: &PrecedentSet,
) -> DetectorPassport {
    let identity_hashes = compute_identity_hashes(record);
    let detector_identity_hash = identity_hashes.detector_identity_hash;
    let (dedup_decision, dedup_reason) = lookup_dedup(record.canonical_id, dedup_records);
    let usefulness_evidence_level = lookup_evidence_level(record.canonical_id);
    let fusion_planes = axes_to_planes(record.fusion_axes);
    let linked_precedent_ids = precedents_for_canonical(precedents, record.canonical_id);
    let mut p = DetectorPassport {
        canonical_id: record.canonical_id,
        display_name: record.display_name,
        aliases: record.aliases,
        source_refs: record.source_refs,
        primitive_family: record.primitive_family,
        mathematical_form: record.mathematical_form,
        decision_functional: record.decision_functional,
        identity_hashes,
        detector_identity_hash,
        dedup_decision,
        dedup_reason,
        genealogy_edges: record.genealogy,
        witness_role: record.witness_role,
        fusion_planes,
        implementation_level: record.implementation_status,
        lifecycle_state: record.lifecycle_state,
        usefulness_evidence_level,
        constitution_flags: record.constitution_compliance,
        linked_precedent_ids,
        passport_hash: [0u8; 32],
    };
    p.passport_hash = compute_passport_hash(&p);
    p
}

fn lookup_dedup(
    canonical_id: DetectorCanonicalId,
    dedup_records: &[DedupRecord],
) -> (CanonicalisationDecision, DedupReason) {
    for rec in dedup_records {
        if let DedupSubject::Canonical(id) = rec.subject {
            if id == canonical_id {
                return (rec.decision, rec.reason_code);
            }
        }
    }
    // Every SEED canonical record has a court record by the T.4
    // invariant. If this branch ever runs, the corpus is
    // structurally broken; fall back to the most conservative
    // decision so the caller can still inspect the rest of the
    // passport.
    (
        CanonicalisationDecision::Canonical,
        DedupReason::OriginRecord,
    )
}

/// Wire name for a [`DedupReason`]. Mirrors the private helper in
/// [`crate::corpus_hash`] so the passport hash is byte-stable
/// without making the corpus-hash helper public.
fn dedup_reason_wire_name(r: DedupReason) -> &'static str {
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

fn lookup_evidence_level(canonical_id: DetectorCanonicalId) -> UsefulnessEvidenceLevel {
    for row in USEFULNESS_LEDGER {
        if row.canonical_id == canonical_id {
            return row.evidence_level;
        }
    }
    UsefulnessEvidenceLevel::Unmeasured
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_ids(out: &mut Vec<u8>, ids: &[DetectorCanonicalId]) {
    // Sort a copy so canonical-byte material is order-independent.
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    write_u32(out, u32::try_from(sorted.len()).unwrap_or(u32::MAX));
    for id in sorted {
        write_u32(out, id);
    }
}

fn write_dedup_decision(out: &mut Vec<u8>, d: CanonicalisationDecision) {
    match d {
        CanonicalisationDecision::Canonical => {
            write_str(out, "Canonical");
            write_u32(out, 0);
        }
        CanonicalisationDecision::AliasOf(id) => {
            write_str(out, "AliasOf");
            write_u32(out, 1);
            write_u32(out, id.0);
        }
        CanonicalisationDecision::ParameterisationOf(id) => {
            write_str(out, "ParameterisationOf");
            write_u32(out, 1);
            write_u32(out, id.0);
        }
        CanonicalisationDecision::CompositionOf(ids) => {
            write_str(out, "CompositionOf");
            write_ids(out, ids);
        }
        CanonicalisationDecision::StochasticOriginalDeterministicReduction(id) => {
            write_str(out, "StochasticOriginalDeterministicReduction");
            write_u32(out, 1);
            write_u32(out, id.0);
        }
        CanonicalisationDecision::RejectedNotDeterministic => {
            write_str(out, "RejectedNotDeterministic");
            write_u32(out, 0);
        }
        CanonicalisationDecision::RejectedNotDetector => {
            write_str(out, "RejectedNotDetector");
            write_u32(out, 0);
        }
        CanonicalisationDecision::DeferredNeedsReview => {
            write_str(out, "DeferredNeedsReview");
            write_u32(out, 0);
        }
    }
}

fn write_genealogy_edges(out: &mut Vec<u8>, e: &GenealogyEdges) {
    write_ids(out, e.derived_from);
    write_ids(out, e.generalizes);
    write_ids(out, e.special_case_of);
    out.push(u8::from(e.is_origin));
}

fn write_constitution_flags(out: &mut Vec<u8>, f: ConstitutionFlags) {
    out.push(u8::from(f.declared_input_contract));
    out.push(u8::from(f.declared_output_type));
    out.push(u8::from(f.declared_deterministic_form));
    out.push(u8::from(f.declared_provenance));
    out.push(u8::from(f.declared_equivalence_status));
    out.push(u8::from(f.declared_witness_role));
    out.push(u8::from(f.declared_activation_conditions));
    out.push(u8::from(f.declared_failure_confuser_modes));
}

fn write_source_refs(out: &mut Vec<u8>, refs: &[SourceRef]) {
    write_u32(out, u32::try_from(refs.len()).unwrap_or(u32::MAX));
    for r in refs {
        write_str(out, r.citation_key);
        write_str(out, r.title);
        write_str(out, r.authors);
        write_u16(out, r.year);
        write_str(out, r.venue_or_source);
        match r.doi_or_url {
            Some(d) => {
                out.push(1);
                write_str(out, d);
            }
            None => out.push(0),
        }
        write_str(out, r.notes);
    }
}

fn write_aliases(out: &mut Vec<u8>, aliases: &[&str]) {
    write_u32(out, u32::try_from(aliases.len()).unwrap_or(u32::MAX));
    for a in aliases {
        write_str(out, a);
    }
}

/// Compute the canonical-byte passport hash. Deterministic across
/// two builds. The hash includes every passport field except the
/// `passport_hash` slot itself (which is set after computation).
#[must_use]
pub fn compute_passport_hash(p: &DetectorPassport) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    buf.extend_from_slice(PASSPORT_DOMAIN.as_bytes());
    write_u32(&mut buf, p.canonical_id.0);
    write_str(&mut buf, p.display_name);
    write_aliases(&mut buf, p.aliases);
    write_source_refs(&mut buf, p.source_refs);
    write_str(&mut buf, p.primitive_family.as_str());
    write_str(&mut buf, p.mathematical_form.as_str());
    write_str(&mut buf, p.decision_functional.as_str());
    // Identity hashes — the five T.3 axes + the composite.
    buf.extend_from_slice(&p.identity_hashes.source_hash);
    buf.extend_from_slice(&p.identity_hashes.formula_hash);
    buf.extend_from_slice(&p.identity_hashes.parameter_hash);
    buf.extend_from_slice(&p.identity_hashes.implementation_hash);
    buf.extend_from_slice(&p.identity_hashes.semantic_role_hash);
    buf.extend_from_slice(&p.detector_identity_hash);
    write_dedup_decision(&mut buf, p.dedup_decision);
    write_str(&mut buf, dedup_reason_wire_name(p.dedup_reason));
    write_genealogy_edges(&mut buf, &p.genealogy_edges);
    write_str(&mut buf, p.witness_role.as_str());
    buf.push(p.fusion_planes.0);
    write_str(&mut buf, p.implementation_level.as_str());
    write_str(&mut buf, p.lifecycle_state.as_str());
    write_str(&mut buf, p.usefulness_evidence_level.as_str());
    write_constitution_flags(&mut buf, p.constitution_flags);
    // Linked precedent ids (T.11b). Already in canonical-sorted
    // ascending order by `precedents_for_canonical`; we still
    // write the count + ids explicitly so the hash material
    // includes the count framing.
    write_u32(
        &mut buf,
        u32::try_from(p.linked_precedent_ids.len()).unwrap_or(u32::MAX),
    );
    for pid in &p.linked_precedent_ids {
        write_u32(&mut buf, pid.0);
    }
    sha256(&buf)
}

/// Render a passport as a human-readable text packet. Two calls
/// on the same passport produce byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_passport_text(p: &DetectorPassport) -> String {
    let mut out = String::with_capacity(2048);
    let _ = writeln!(
        out,
        "DetectorPassport (canonical_id = {})",
        p.canonical_id.0
    );
    let _ = writeln!(out, "  display_name              : {}", p.display_name);
    out.push_str("  aliases                   : ");
    if p.aliases.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push('\n');
        for a in p.aliases {
            let _ = writeln!(out, "                                - {a}");
        }
    }
    out.push_str("  source_refs               : ");
    if p.source_refs.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push('\n');
        for r in p.source_refs {
            let _ = writeln!(
                out,
                "                                - {} ({}) [{}]",
                r.citation_key, r.year, r.venue_or_source
            );
        }
    }
    let _ = writeln!(
        out,
        "  primitive_family          : {}",
        p.primitive_family.as_str()
    );
    let _ = writeln!(
        out,
        "  mathematical_form         : {}",
        p.mathematical_form.as_str()
    );
    let _ = writeln!(
        out,
        "  decision_functional       : {}",
        p.decision_functional.as_str()
    );
    let _ = writeln!(
        out,
        "  formula_hash              : {}",
        hex_lower(&p.identity_hashes.formula_hash)
    );
    let _ = writeln!(
        out,
        "  parameter_hash            : {}",
        hex_lower(&p.identity_hashes.parameter_hash)
    );
    let _ = writeln!(
        out,
        "  implementation_hash       : {}",
        hex_lower(&p.identity_hashes.implementation_hash)
    );
    let _ = writeln!(
        out,
        "  semantic_role_hash        : {}",
        hex_lower(&p.identity_hashes.semantic_role_hash)
    );
    let _ = writeln!(
        out,
        "  detector_identity_hash    : {}",
        hex_lower(&p.detector_identity_hash)
    );
    let _ = writeln!(
        out,
        "  dedup_decision            : {}",
        format_dedup_decision(p.dedup_decision)
    );
    let _ = writeln!(
        out,
        "  dedup_reason              : {}",
        dedup_reason_wire_name(p.dedup_reason)
    );
    let _ = writeln!(
        out,
        "  genealogy.derived_from    : {}",
        format_ids(p.genealogy_edges.derived_from)
    );
    let _ = writeln!(
        out,
        "  genealogy.generalizes     : {}",
        format_ids(p.genealogy_edges.generalizes)
    );
    let _ = writeln!(
        out,
        "  genealogy.special_case_of : {}",
        format_ids(p.genealogy_edges.special_case_of)
    );
    let _ = writeln!(
        out,
        "  genealogy.is_origin       : {}",
        p.genealogy_edges.is_origin
    );
    let _ = writeln!(
        out,
        "  witness_role              : {}",
        p.witness_role.as_str()
    );
    out.push_str("  fusion_planes             : ");
    let planes = p.fusion_planes.planes();
    if planes.is_empty() {
        out.push_str("(none)\n");
    } else {
        out.push('\n');
        for plane in planes {
            let _ = writeln!(out, "                                - {}", plane.as_str());
        }
    }
    let _ = writeln!(
        out,
        "  implementation_level      : {}",
        p.implementation_level.as_str()
    );
    let _ = writeln!(
        out,
        "  lifecycle_state           : {}",
        p.lifecycle_state.as_str()
    );
    let _ = writeln!(
        out,
        "  usefulness_evidence_level : {}",
        p.usefulness_evidence_level.as_str()
    );
    out.push_str("  constitution_flags        :\n");
    let flags = [
        (
            "input_contract",
            p.constitution_flags.declared_input_contract,
        ),
        ("output_type", p.constitution_flags.declared_output_type),
        (
            "deterministic_form",
            p.constitution_flags.declared_deterministic_form,
        ),
        ("provenance", p.constitution_flags.declared_provenance),
        (
            "equivalence_status",
            p.constitution_flags.declared_equivalence_status,
        ),
        ("witness_role", p.constitution_flags.declared_witness_role),
        (
            "activation_conditions",
            p.constitution_flags.declared_activation_conditions,
        ),
        (
            "failure_confuser_modes",
            p.constitution_flags.declared_failure_confuser_modes,
        ),
    ];
    for (name, value) in flags {
        let _ = writeln!(out, "                                {name:<24} = {value}");
    }
    out.push_str("  linked_precedent_ids      : ");
    if p.linked_precedent_ids.is_empty() {
        out.push_str("(none)\n");
    } else {
        let mut ids: Vec<u32> = p.linked_precedent_ids.iter().map(|i| i.0).collect();
        ids.sort_unstable();
        for (i, id) in ids.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{id}");
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "  passport_hash             : {}",
        hex_lower(&p.passport_hash)
    );
    out
}

/// Render a passport as a deterministic JSON object. Two calls on
/// the same passport produce byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_passport_json(p: &DetectorPassport) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("{\n");
    let _ = writeln!(out, "  \"canonical_id\": {},", p.canonical_id.0);
    out.push_str("  \"display_name\": ");
    json_quote(&mut out, p.display_name);
    out.push_str(",\n");
    out.push_str("  \"aliases\": [");
    for (i, a) in p.aliases.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_quote(&mut out, a);
    }
    out.push_str("],\n");
    out.push_str("  \"source_refs\": [");
    for (i, r) in p.source_refs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"citation_key\": ");
        json_quote(&mut out, r.citation_key);
        let _ = write!(out, ", \"year\": {}", r.year);
        out.push_str(", \"venue_or_source\": ");
        json_quote(&mut out, r.venue_or_source);
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"primitive_family\": ");
    json_quote(&mut out, p.primitive_family.as_str());
    out.push_str(",\n");
    out.push_str("  \"mathematical_form\": ");
    json_quote(&mut out, p.mathematical_form.as_str());
    out.push_str(",\n");
    out.push_str("  \"decision_functional\": ");
    json_quote(&mut out, p.decision_functional.as_str());
    out.push_str(",\n");
    let _ = writeln!(
        out,
        "  \"formula_hash\": \"{}\",",
        hex_lower(&p.identity_hashes.formula_hash)
    );
    let _ = writeln!(
        out,
        "  \"parameter_hash\": \"{}\",",
        hex_lower(&p.identity_hashes.parameter_hash)
    );
    let _ = writeln!(
        out,
        "  \"implementation_hash\": \"{}\",",
        hex_lower(&p.identity_hashes.implementation_hash)
    );
    let _ = writeln!(
        out,
        "  \"semantic_role_hash\": \"{}\",",
        hex_lower(&p.identity_hashes.semantic_role_hash)
    );
    let _ = writeln!(
        out,
        "  \"detector_identity_hash\": \"{}\",",
        hex_lower(&p.detector_identity_hash)
    );
    out.push_str("  \"dedup_decision\": ");
    json_quote(&mut out, &format_dedup_decision(p.dedup_decision));
    out.push_str(",\n");
    out.push_str("  \"dedup_reason\": ");
    json_quote(&mut out, dedup_reason_wire_name(p.dedup_reason));
    out.push_str(",\n");
    let _ = writeln!(
        out,
        "  \"genealogy_derived_from\": {},",
        format_ids_json(p.genealogy_edges.derived_from)
    );
    let _ = writeln!(
        out,
        "  \"genealogy_generalizes\": {},",
        format_ids_json(p.genealogy_edges.generalizes)
    );
    let _ = writeln!(
        out,
        "  \"genealogy_special_case_of\": {},",
        format_ids_json(p.genealogy_edges.special_case_of)
    );
    let _ = writeln!(
        out,
        "  \"genealogy_is_origin\": {},",
        p.genealogy_edges.is_origin
    );
    out.push_str("  \"witness_role\": ");
    json_quote(&mut out, p.witness_role.as_str());
    out.push_str(",\n");
    out.push_str("  \"fusion_planes\": [");
    for (i, plane) in p.fusion_planes.planes().iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_quote(&mut out, plane.as_str());
    }
    out.push_str("],\n");
    out.push_str("  \"implementation_level\": ");
    json_quote(&mut out, p.implementation_level.as_str());
    out.push_str(",\n");
    out.push_str("  \"lifecycle_state\": ");
    json_quote(&mut out, p.lifecycle_state.as_str());
    out.push_str(",\n");
    out.push_str("  \"usefulness_evidence_level\": ");
    json_quote(&mut out, p.usefulness_evidence_level.as_str());
    out.push_str(",\n");
    out.push_str("  \"constitution_flags\": {\n");
    let flags = [
        (
            "declared_input_contract",
            p.constitution_flags.declared_input_contract,
        ),
        (
            "declared_output_type",
            p.constitution_flags.declared_output_type,
        ),
        (
            "declared_deterministic_form",
            p.constitution_flags.declared_deterministic_form,
        ),
        (
            "declared_provenance",
            p.constitution_flags.declared_provenance,
        ),
        (
            "declared_equivalence_status",
            p.constitution_flags.declared_equivalence_status,
        ),
        (
            "declared_witness_role",
            p.constitution_flags.declared_witness_role,
        ),
        (
            "declared_activation_conditions",
            p.constitution_flags.declared_activation_conditions,
        ),
        (
            "declared_failure_confuser_modes",
            p.constitution_flags.declared_failure_confuser_modes,
        ),
    ];
    for (i, (name, value)) in flags.iter().enumerate() {
        let _ = write!(out, "    \"{name}\": {value}");
        if i + 1 < flags.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  },\n");
    out.push_str("  \"linked_precedent_ids\": [");
    let mut ids: Vec<u32> = p.linked_precedent_ids.iter().map(|i| i.0).collect();
    ids.sort_unstable();
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{id}");
    }
    out.push_str("],\n");
    let _ = writeln!(
        out,
        "  \"passport_hash\": \"{}\"",
        hex_lower(&p.passport_hash)
    );
    out.push_str("}\n");
    out
}

fn json_quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn format_dedup_decision(d: CanonicalisationDecision) -> String {
    match d {
        CanonicalisationDecision::Canonical => "Canonical".to_string(),
        CanonicalisationDecision::AliasOf(id) => format!("AliasOf({})", id.0),
        CanonicalisationDecision::ParameterisationOf(id) => format!("ParameterisationOf({})", id.0),
        CanonicalisationDecision::CompositionOf(ids) => {
            let mut s = String::from("CompositionOf([");
            let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
            sorted.sort_unstable();
            for (i, id) in sorted.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                let _ = write!(s, "{id}");
            }
            s.push_str("])");
            s
        }
        CanonicalisationDecision::StochasticOriginalDeterministicReduction(id) => {
            format!("StochasticOriginalDeterministicReduction({})", id.0)
        }
        CanonicalisationDecision::RejectedNotDeterministic => {
            "RejectedNotDeterministic".to_string()
        }
        CanonicalisationDecision::RejectedNotDetector => "RejectedNotDetector".to_string(),
        CanonicalisationDecision::DeferredNeedsReview => "DeferredNeedsReview".to_string(),
    }
}

fn format_ids(ids: &[DetectorCanonicalId]) -> String {
    if ids.is_empty() {
        return "(none)".to_string();
    }
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::new();
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    s
}

fn format_ids_json(ids: &[DetectorCanonicalId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::from("[");
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    s.push(']');
    s
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[usize::from(b >> 4)] as char);
        s.push(HEX[usize::from(b & 0x0F)] as char);
    }
    s
}
