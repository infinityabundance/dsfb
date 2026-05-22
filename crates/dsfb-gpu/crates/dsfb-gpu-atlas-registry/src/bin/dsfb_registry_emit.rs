//! S1.2 — emit canonical registry artifacts under `out/`.
//!
//! Writes two files into `crates/dsfb-gpu-atlas-registry/out/`:
//!
//! - `detector_registry_v2.bin` — the canonical-byte material
//!   fed to `compute_registry_hash_v2` (domain separator +
//!   schema id + spec count + sorted per-spec bytes).
//! - `detector_registry_v2.json` — a human-readable mirror of
//!   the same data: spec count, registry hash, source corpus
//!   hash, and one JSON object per spec listing every wire
//!   field in canonical-name order.
//!
//! Two invocations against the same corpus + family mapping +
//! parameter grid produce byte-identical files. The pinned
//! artifacts in the repo's `out/` directory MUST match this
//! binary's output on a fresh build.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::PathBuf;

use dsfb_gpu_atlas_registry::{
    compute_registry_hash_v2, write_registry_hash_v2_material, DetectorRegistryV2,
    REGISTRY_HASH_V2_DOMAIN,
};

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

fn out_dir() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.join("out")
}

fn append_hex(out: &mut String, bytes: &[u8]) {
    for b in bytes {
        out.push(HEX_DIGITS[usize::from(b >> 4)] as char);
        out.push(HEX_DIGITS[usize::from(b & 0x0F)] as char);
    }
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

fn render_bin(out: &mut Vec<u8>) {
    let registry = DetectorRegistryV2::build();
    out.extend_from_slice(REGISTRY_HASH_V2_DOMAIN.as_bytes());
    write_registry_hash_v2_material(out, &registry.specs);
}

#[allow(clippy::too_many_lines)]
fn render_json(out: &mut String) {
    let registry = DetectorRegistryV2::build();
    out.push_str("{\n");
    out.push_str("  \"schema\": \"DSFB-GPU-ATLAS:DETECTOR-REGISTRY:v2\",\n");
    let _ = writeln!(
        out,
        "  \"literature_primitives\": {},",
        registry.counts.literature_primitives
    );
    let _ = writeln!(
        out,
        "  \"parameterized_specs\": {},",
        registry.counts.parameterized_specs
    );
    let _ = writeln!(
        out,
        "  \"active_detectors\": {},",
        registry.counts.active_detectors
    );
    let _ = writeln!(
        out,
        "  \"admitted_episodes\": {},",
        registry.counts.admitted_episodes
    );
    out.push_str("  \"source_corpus_hash\": \"");
    append_hex(out, &registry.source_corpus_hash);
    out.push_str("\",\n");
    let h = compute_registry_hash_v2(&registry.specs);
    out.push_str("  \"registry_hash_v2\": \"");
    append_hex(out, &h);
    out.push_str("\",\n");
    out.push_str("  \"specs\": [\n");
    // Specs in the JSON dump are sorted by (detector_id,
    // parameterization_id) — the same canonical order used by
    // the hash. The clone-and-sort avoids mutating
    // `registry.specs` in case callers care.
    let mut sorted: Vec<_> = registry.specs.iter().collect();
    sorted.sort_by_key(|s| (s.detector_id.0, s.parameterization_id.0));
    for (i, spec) in sorted.iter().enumerate() {
        out.push_str("    {\n");
        let _ = writeln!(out, "      \"detector_id\": {},", spec.detector_id.0);
        let _ = writeln!(
            out,
            "      \"parameterization_id\": {},",
            spec.parameterization_id.0
        );
        out.push_str("      \"family\": ");
        json_quote(out, spec.family.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"transform\": ");
        json_quote(out, spec.transform.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"statistic\": ");
        json_quote(out, spec.statistic.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"comparator\": ");
        json_quote(out, spec.comparator.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"gate\": ");
        json_quote(out, spec.gate.canonical_wire_name());
        out.push_str(",\n");
        let _ = writeln!(out, "      \"window_cells\": {},", spec.window.cells);
        let _ = writeln!(
            out,
            "      \"persistence_windows\": {},",
            spec.persistence_windows
        );
        let _ = writeln!(out, "      \"axis_binding\": {},", spec.axis_binding.0);
        let _ = writeln!(out, "      \"domain_tags\": {},", spec.domain_tags.0);
        out.push_str("      \"cost_class\": ");
        json_quote(out, spec.cost_class.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"numeric_mode\": ");
        json_quote(out, spec.numeric_mode.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"implementation_kind\": ");
        json_quote(out, spec.implementation_kind.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"parameter_hash\": \"");
        append_hex(out, &spec.parameter_hash);
        out.push_str("\",\n");
        match spec.primitive_id {
            Some(id) => {
                let _ = writeln!(out, "      \"primitive_id\": {},", id.0);
            }
            None => out.push_str("      \"primitive_id\": null,\n"),
        }
        out.push_str("      \"corpus_binding_status\": ");
        json_quote(out, spec.corpus_binding_status.canonical_wire_name());
        out.push_str(",\n");
        out.push_str("      \"source_corpus_hash\": \"");
        append_hex(out, &spec.source_corpus_hash);
        out.push_str("\",\n");
        out.push_str("      \"canonical_name\": ");
        json_quote(out, spec.canonical_name.as_str());
        out.push('\n');
        if i + 1 < sorted.len() {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
}

fn main() -> io::Result<()> {
    let dir = out_dir();
    fs::create_dir_all(&dir)?;

    let mut bin_buf: Vec<u8> = Vec::new();
    render_bin(&mut bin_buf);
    let bin_path = dir.join("detector_registry_v2.bin");
    fs::write(&bin_path, &bin_buf)?;

    let mut json_buf = String::new();
    render_json(&mut json_buf);
    let json_path = dir.join("detector_registry_v2.json");
    fs::write(&json_path, json_buf.as_bytes())?;

    println!("wrote {} ({} bytes)", bin_path.display(), bin_buf.len());
    println!("wrote {} ({} bytes)", json_path.display(), json_buf.len());
    Ok(())
}
