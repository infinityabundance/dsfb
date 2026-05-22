//! T.3 — Five-hash detector identity.
//!
//! Every literature primitive in the corpus carries five domain-
//! separated SHA-256 hashes plus one composite `detector_identity_hash`:
//!
//! - `source_hash` — over the provenance (`source_refs`). Records
//!   citations; NEVER enters the identity hash.
//! - `formula_hash` — over the canonical math form (primitive family,
//!   mathematical form, decision functional, output witness, and
//!   input contract). The "what does this witness compute" axis of
//!   identity.
//! - `parameter_hash` — over the parameter-grid descriptor
//!   (axis_count + description). The "with which parameters" axis.
//! - `implementation_hash` — over the implementation-side fields
//!   (L-band status, GPU family kernel, deterministic status). Records
//!   how the witness is implemented; NEVER enters the identity hash.
//! - `semantic_role_hash` — over the courtroom role (witness role,
//!   negative-witness kind, fusion-axis bitset, confuser profile).
//!   The "what role does this witness play in fusion" axis.
//! - `detector_identity_hash` — the canonical primitive identity:
//!   `SHA256(DETECTOR_IDENTITY_DOMAIN || formula_hash ||
//!   parameter_hash || semantic_role_hash)` where
//!   `DETECTOR_IDENTITY_DOMAIN` is the versioned byte string
//!   `"DSFB-GPU-ATLAS:DETECTOR-IDENTITY:v1\0"`.
//!
//! **Panel-locked invariants** (T.3 acceptance tests pin these):
//!
//! - `source_hash` is provenance only. Improving citations / DOIs /
//!   notes must change `source_hash` and **must not** change
//!   `detector_identity_hash`.
//! - `implementation_hash` is implementation only. Upgrading the
//!   L-band (L1 → L6) or porting to a new GPU family must change
//!   `implementation_hash` and **must not** change
//!   `detector_identity_hash`. This is what lets the L-band ladder
//!   evolve without breaking equivalence classes.
//! - Two detectors with the same formula + parameter + semantic
//!   role MUST share `detector_identity_hash`. The dedup court
//!   (T.4) will use this property to detect equivalence classes.
//!
//! Canonical serialisation: length-prefixed UTF-8 strings (u32 LE
//! length) and little-endian integers. Each hash carries a versioned
//! domain separator (`...:v1\0`) so the schema can be migrated
//! without silently colliding with v1 hashes.
//!
//! NOT in T.3: `corpus_hash_v1` (T.10), the `CaseFileV2` integration
//! (T.11), and the dedup-court machinery itself (T.4). T.3 only
//! computes per-record hashes.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use dsfb_gpu_debug_core::hash::sha256;

use crate::loader::{LoadedLiteratureDetector, LoadedSourceRef};
use crate::types::{LiteratureDetector, SourceRef};

/// Length of every hash component (SHA-256 → 32 bytes).
pub const HASH_LEN: usize = 32;

/// Versioned domain separator for the source hash.
pub const SOURCE_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:SOURCE:v1\0";
/// Versioned domain separator for the formula hash.
pub const FORMULA_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:FORMULA:v1\0";
/// Versioned domain separator for the parameter hash.
pub const PARAMETER_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:PARAMETER:v1\0";
/// Versioned domain separator for the implementation hash.
pub const IMPLEMENTATION_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:IMPLEMENTATION:v1\0";
/// Versioned domain separator for the semantic-role hash.
pub const SEMANTIC_ROLE_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:SEMANTIC-ROLE:v1\0";
/// Versioned domain separator for the composite detector-identity hash.
pub const DETECTOR_IDENTITY_DOMAIN: &[u8] = b"DSFB-GPU-ATLAS:DETECTOR-IDENTITY:v1\0";

/// Five canonical hashes plus the composite identity hash.
///
/// The composite is intentionally NOT a function of `source_hash` or
/// `implementation_hash`: the corpus can fix citations or upgrade the
/// L-band without changing the canonical mathematical identity. The
/// philosophical claim that the T.3 acceptance tests pin: the
/// literature source is provenance, not identity; the implementation
/// is execution, not identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetectorIdentityHashes {
    /// Provenance hash (over `source_refs`). NEVER enters identity.
    pub source_hash: [u8; HASH_LEN],
    /// Mathematical form hash (formula identity axis).
    pub formula_hash: [u8; HASH_LEN],
    /// Parameter-grid hash (parameter identity axis).
    pub parameter_hash: [u8; HASH_LEN],
    /// Implementation hash (L-band, GPU family, deterministic
    /// status). NEVER enters identity.
    pub implementation_hash: [u8; HASH_LEN],
    /// Semantic-role hash (witness role + fusion bindings).
    pub semantic_role_hash: [u8; HASH_LEN],
    /// Composite canonical identity:
    /// `SHA256(domain || formula || parameter || semantic_role)`.
    pub detector_identity_hash: [u8; HASH_LEN],
}

impl DetectorIdentityHashes {
    /// Format `detector_identity_hash` as a lowercase hex string
    /// (64 characters). Useful for reports + paper rendering.
    #[must_use]
    pub fn detector_identity_hex(&self) -> String {
        hex_lower(&self.detector_identity_hash)
    }

    /// Format `source_hash` as a lowercase hex string.
    #[must_use]
    pub fn source_hex(&self) -> String {
        hex_lower(&self.source_hash)
    }
}

/// Compute every hash for a static-seed record.
#[must_use]
pub fn compute_identity_hashes(record: &LiteratureDetector) -> DetectorIdentityHashes {
    compute_from_fields(
        record.primitive_family.as_str(),
        record.mathematical_form.as_str(),
        record.decision_functional.as_str(),
        record.output_witness.as_str(),
        record.input_requirements.0,
        record.parameter_bounds.axis_count,
        record.parameter_bounds.description,
        record.implementation_status.as_str(),
        record.gpu_family.as_str(),
        record.deterministic_status.as_str(),
        record.witness_role.as_str(),
        record.negative_witness_kind.as_str(),
        record.fusion_axes.0,
        record.confuser_profile.as_str(),
        record.source_refs.iter().map(static_source_view),
    )
}

/// Compute every hash for a loaded (owned-data) record. The result
/// is byte-identical to [`compute_identity_hashes`] on the
/// equivalent static record (pinned by T.3 acceptance tests).
#[must_use]
pub fn compute_identity_hashes_loaded(record: &LoadedLiteratureDetector) -> DetectorIdentityHashes {
    compute_from_fields(
        record.primitive_family.as_str(),
        record.mathematical_form.as_str(),
        record.decision_functional.as_str(),
        record.output_witness.as_str(),
        record.input_requirements.0,
        record.parameter_bounds.axis_count,
        record.parameter_bounds.description.as_str(),
        record.implementation_status.as_str(),
        record.gpu_family.as_str(),
        record.deterministic_status.as_str(),
        record.witness_role.as_str(),
        record.negative_witness_kind.as_str(),
        record.fusion_axes.0,
        record.confuser_profile.as_str(),
        record.source_refs.iter().map(loaded_source_view),
    )
}

#[derive(Clone, Copy)]
struct SourceView<'a> {
    citation_key: &'a str,
    title: &'a str,
    authors: &'a str,
    year: u16,
    venue_or_source: &'a str,
    doi_or_url: Option<&'a str>,
    notes: &'a str,
}

fn static_source_view(s: &'static SourceRef) -> SourceView<'static> {
    SourceView {
        citation_key: s.citation_key,
        title: s.title,
        authors: s.authors,
        year: s.year,
        venue_or_source: s.venue_or_source,
        doi_or_url: s.doi_or_url,
        notes: s.notes,
    }
}

fn loaded_source_view(s: &LoadedSourceRef) -> SourceView<'_> {
    SourceView {
        citation_key: s.citation_key.as_str(),
        title: s.title.as_str(),
        authors: s.authors.as_str(),
        year: s.year,
        venue_or_source: s.venue_or_source.as_str(),
        doi_or_url: s.doi_or_url.as_deref(),
        notes: s.notes.as_str(),
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_from_fields<'a, I>(
    primitive_family: &str,
    mathematical_form: &str,
    decision_functional: &str,
    output_witness: &str,
    input_requirements_bits: u32,
    parameter_axis_count: u8,
    parameter_description: &str,
    implementation_status: &str,
    gpu_family: &str,
    deterministic_status: &str,
    witness_role: &str,
    negative_witness_kind: &str,
    fusion_axes_bits: u16,
    confuser_profile: &str,
    source_refs: I,
) -> DetectorIdentityHashes
where
    I: IntoIterator<Item = SourceView<'a>>,
{
    let formula_hash = hash_formula(
        primitive_family,
        mathematical_form,
        decision_functional,
        output_witness,
        input_requirements_bits,
    );
    let parameter_hash = hash_parameter(parameter_axis_count, parameter_description);
    let source_hash = hash_source(source_refs);
    let implementation_hash =
        hash_implementation(implementation_status, gpu_family, deterministic_status);
    let semantic_role_hash = hash_semantic_role(
        witness_role,
        negative_witness_kind,
        fusion_axes_bits,
        confuser_profile,
    );
    let detector_identity_hash =
        hash_detector_identity(&formula_hash, &parameter_hash, &semantic_role_hash);

    DetectorIdentityHashes {
        source_hash,
        formula_hash,
        parameter_hash,
        implementation_hash,
        semantic_role_hash,
        detector_identity_hash,
    }
}

fn hash_formula(
    primitive_family: &str,
    mathematical_form: &str,
    decision_functional: &str,
    output_witness: &str,
    input_requirements_bits: u32,
) -> [u8; HASH_LEN] {
    let mut buf = Vec::new();
    buf.extend_from_slice(FORMULA_DOMAIN);
    write_string(&mut buf, primitive_family);
    write_string(&mut buf, mathematical_form);
    write_string(&mut buf, decision_functional);
    write_string(&mut buf, output_witness);
    buf.extend_from_slice(&input_requirements_bits.to_le_bytes());
    sha256(&buf)
}

fn hash_parameter(axis_count: u8, description: &str) -> [u8; HASH_LEN] {
    let mut buf = Vec::new();
    buf.extend_from_slice(PARAMETER_DOMAIN);
    buf.push(axis_count);
    write_string(&mut buf, description);
    sha256(&buf)
}

fn hash_source<'a, I>(source_refs: I) -> [u8; HASH_LEN]
where
    I: IntoIterator<Item = SourceView<'a>>,
{
    let refs: Vec<SourceView<'a>> = source_refs.into_iter().collect();
    let mut buf = Vec::new();
    buf.extend_from_slice(SOURCE_DOMAIN);
    let count = u32::try_from(refs.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&count.to_le_bytes());
    for r in &refs {
        write_string(&mut buf, r.citation_key);
        write_string(&mut buf, r.title);
        write_string(&mut buf, r.authors);
        buf.extend_from_slice(&r.year.to_le_bytes());
        write_string(&mut buf, r.venue_or_source);
        write_string(&mut buf, r.doi_or_url.unwrap_or(""));
        write_string(&mut buf, r.notes);
    }
    sha256(&buf)
}

fn hash_implementation(
    implementation_status: &str,
    gpu_family: &str,
    deterministic_status: &str,
) -> [u8; HASH_LEN] {
    let mut buf = Vec::new();
    buf.extend_from_slice(IMPLEMENTATION_DOMAIN);
    write_string(&mut buf, implementation_status);
    write_string(&mut buf, gpu_family);
    write_string(&mut buf, deterministic_status);
    sha256(&buf)
}

fn hash_semantic_role(
    witness_role: &str,
    negative_witness_kind: &str,
    fusion_axes_bits: u16,
    confuser_profile: &str,
) -> [u8; HASH_LEN] {
    let mut buf = Vec::new();
    buf.extend_from_slice(SEMANTIC_ROLE_DOMAIN);
    write_string(&mut buf, witness_role);
    write_string(&mut buf, negative_witness_kind);
    buf.extend_from_slice(&fusion_axes_bits.to_le_bytes());
    write_string(&mut buf, confuser_profile);
    sha256(&buf)
}

fn hash_detector_identity(
    formula_hash: &[u8; HASH_LEN],
    parameter_hash: &[u8; HASH_LEN],
    semantic_role_hash: &[u8; HASH_LEN],
) -> [u8; HASH_LEN] {
    let mut buf = Vec::new();
    buf.extend_from_slice(DETECTOR_IDENTITY_DOMAIN);
    buf.extend_from_slice(formula_hash);
    buf.extend_from_slice(parameter_hash);
    buf.extend_from_slice(semantic_role_hash);
    sha256(&buf)
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let len = u32::try_from(s.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn hex_lower(bytes: &[u8; HASH_LEN]) -> String {
    let mut s = String::with_capacity(HASH_LEN * 2);
    for byte in bytes {
        let hi = (byte >> 4) & 0x0F;
        let lo = byte & 0x0F;
        s.push(hex_digit(hi));
        s.push(hex_digit(lo));
    }
    s
}

fn hex_digit(nibble: u8) -> char {
    if nibble < 10 {
        (b'0' + nibble) as char
    } else {
        (b'a' + (nibble - 10)) as char
    }
}
