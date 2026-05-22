//! S1.2 — `registry_hash_v2` definition.
//!
//! Hashes the canonical-byte projection of an S1.2-generated
//! detector-spec list. Two builds produce byte-identical hashes
//! given the same `corpus_hash_v1` + generator + family mapping.
//!
//! Material order (panel-locked):
//!
//! 1. Domain separator + schema id.
//! 2. Spec count (u32 BE).
//! 3. Per-spec canonical bytes, sorted by
//!    `(detector_id, parameterization_id)`:
//!    - detector_id (u32 BE)
//!    - parameterization_id (u32 BE)
//!    - family wire name (length-prefixed string)
//!    - transform / statistic / comparator / gate wire names
//!    - window cells / persistence_windows (u32 BE)
//!    - axis_binding.0 / domain_tags.0 (u16 BE)
//!    - cost_class / numeric_mode / implementation_kind wire names
//!    - parameter_hash (32 bytes)
//!    - primitive_id (u32 BE; `u32::MAX` for `None`)
//!    - corpus_binding_status wire name
//!    - source_corpus_hash (32 bytes)
//!    - canonical_name (length-prefixed string)
//!
//! Domain separator: `DSFB-GPU-ATLAS:DETECTOR-REGISTRY:v2\0`.
//! Schema id: `DSFB-GPU-ATLAS:REGISTRY-HASH-SCHEMA:v2`.
//!
//! Sensitivity: changing any byte of any spec — or the spec
//! count, the family-mapping table, the parameter grid, or the
//! `source_corpus_hash` — changes `registry_hash_v2`. Pinned by
//! S1.2 acceptance tests.

extern crate alloc;
use alloc::vec::Vec;

use dsfb_gpu_debug_core::sha256;

use crate::DetectorSpec;

/// Domain separator prefix for the registry hash.
pub const REGISTRY_HASH_V2_DOMAIN: &str = "DSFB-GPU-ATLAS:DETECTOR-REGISTRY:v2\0";

/// Schema identifier carried in the hash material.
pub const REGISTRY_HASH_V2_SCHEMA: &str = "DSFB-GPU-ATLAS:REGISTRY-HASH-SCHEMA:v2";

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn write_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_spec(out: &mut Vec<u8>, spec: &DetectorSpec) {
    write_u32(out, spec.detector_id.0);
    write_u32(out, spec.parameterization_id.0);
    write_str(out, spec.family.canonical_wire_name());
    write_str(out, spec.transform.canonical_wire_name());
    write_str(out, spec.statistic.canonical_wire_name());
    write_str(out, spec.comparator.canonical_wire_name());
    write_str(out, spec.gate.canonical_wire_name());
    write_u32(out, spec.window.cells);
    write_u32(out, spec.persistence_windows);
    write_u16(out, spec.axis_binding.0);
    write_u16(out, spec.domain_tags.0);
    write_str(out, spec.cost_class.canonical_wire_name());
    write_str(out, spec.numeric_mode.canonical_wire_name());
    write_str(out, spec.implementation_kind.canonical_wire_name());
    out.extend_from_slice(&spec.parameter_hash);
    let prim_id = spec.primitive_id.map_or(u32::MAX, |c| c.0);
    write_u32(out, prim_id);
    write_str(out, spec.corpus_binding_status.canonical_wire_name());
    out.extend_from_slice(&spec.source_corpus_hash);
    write_str(out, spec.canonical_name.as_str());
}

/// Write the canonical-byte registry-hash material into `out`.
/// Specs are sorted by `(detector_id, parameterization_id)`
/// before writing.
pub fn write_registry_hash_v2_material(out: &mut Vec<u8>, specs: &[DetectorSpec]) {
    write_str(out, REGISTRY_HASH_V2_SCHEMA);
    write_u32(out, specs.len() as u32);
    // Canonical sort. specs.to_vec().sort() avoids mutating the
    // caller's slice.
    let mut sorted: Vec<&DetectorSpec> = specs.iter().collect();
    sorted.sort_by_key(|s| (s.detector_id.0, s.parameterization_id.0));
    for s in sorted {
        write_spec(out, s);
    }
}

/// Compute `registry_hash_v2` over a slice of detector specs.
/// Deterministic across two calls.
#[must_use]
pub fn compute_registry_hash_v2(specs: &[DetectorSpec]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    buf.extend_from_slice(REGISTRY_HASH_V2_DOMAIN.as_bytes());
    write_registry_hash_v2_material(&mut buf, specs);
    sha256(&buf)
}
