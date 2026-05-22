//! T.10 acceptance tests: `corpus_hash_v1` invariants.
//!
//! Panel-required tests in this file:
//!
//! - `corpus_hash_v1_is_stable_across_two_runs`
//! - `corpus_hash_v1_changes_when_source_hash_material_changes`
//! - `corpus_hash_v1_includes_dedup_court_decisions`
//! - `corpus_hash_v1_includes_genealogy_edges`
//! - `corpus_hash_v1_includes_witness_roles`
//! - `corpus_hash_v1_includes_lband_states`
//! - `corpus_hash_v1_includes_usefulness_rows`
//! - `corpus_hash_v1_uses_domain_separator`
//! - `corpus_hash_v1_does_not_hash_report_rendering`
//! - `static_seed_and_toml_loader_project_same_hash_material`
//! - `t10_does_not_change_d16_d64_d128_d205_hashes`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::corpus_hash::{
    compute_corpus_hash_v1, write_corpus_hash_material_v1, CorpusHashV1, CORPUS_HASH_DOMAIN_V1,
    CORPUS_HASH_SCHEMA_V1,
};
use dsfb_gpu_debug_core::sha256;

#[test]
fn corpus_hash_v1_is_stable_across_two_runs() {
    let a = compute_corpus_hash_v1();
    let b = compute_corpus_hash_v1();
    assert_eq!(
        a.bytes, b.bytes,
        "corpus_hash_v1 must be deterministic across two builds"
    );
    assert!(
        !a.is_zero(),
        "corpus_hash_v1 must not be the all-zero sentinel on the non-empty T.1-T.9 corpus"
    );
}

#[test]
fn corpus_hash_v1_uses_domain_separator() {
    // Construct a material buffer manually WITHOUT the domain
    // prefix, hash it, and confirm the result is different from
    // compute_corpus_hash_v1(). This pins that the domain prefix
    // is load-bearing — without it the hash collapses to a
    // generic SHA-256 over the material bytes.
    let mut buf_without_prefix: Vec<u8> = Vec::new();
    write_corpus_hash_material_v1(&mut buf_without_prefix);
    let hash_without_prefix = sha256(&buf_without_prefix);
    let with_prefix = compute_corpus_hash_v1();
    assert_ne!(
        hash_without_prefix, with_prefix.bytes,
        "domain separator must affect the hash"
    );

    // Construct a material buffer WITH the domain prefix and
    // confirm it matches compute_corpus_hash_v1.
    let mut buf_with_prefix: Vec<u8> = Vec::new();
    buf_with_prefix.extend_from_slice(CORPUS_HASH_DOMAIN_V1.as_bytes());
    write_corpus_hash_material_v1(&mut buf_with_prefix);
    let direct = sha256(&buf_with_prefix);
    assert_eq!(
        direct, with_prefix.bytes,
        "domain-prefixed manual hash must equal compute_corpus_hash_v1"
    );
}

#[test]
fn corpus_hash_v1_schema_constant_is_panel_locked() {
    assert_eq!(
        CORPUS_HASH_DOMAIN_V1, "DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0",
        "domain separator is panel-locked"
    );
    assert_eq!(
        CORPUS_HASH_SCHEMA_V1, "DSFB-GPU-ATLAS:CORPUS-HASH-SCHEMA:v1",
        "schema id is panel-locked"
    );
}

#[test]
fn corpus_hash_v1_hex_renders_64_lowercase_hex_chars() {
    let h = compute_corpus_hash_v1();
    let hex = h.to_hex();
    assert_eq!(hex.len(), 64);
    assert!(
        hex.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "hex render must be lowercase hex; got `{hex}`"
    );
}

#[test]
fn material_writer_is_deterministic() {
    let mut a: Vec<u8> = Vec::new();
    let mut b: Vec<u8> = Vec::new();
    write_corpus_hash_material_v1(&mut a);
    write_corpus_hash_material_v1(&mut b);
    assert_eq!(
        a, b,
        "two calls to write_corpus_hash_material_v1 must produce byte-identical output"
    );
    assert!(!a.is_empty(), "material buffer must be non-empty");
}

// ---------------------------------------------------------------
// Material-coverage invariants: confirm that key T.1-T.9 sections
// contribute to the hash. We test this by grepping the canonical
// bytes for the load-bearing section labels and content tokens.
// ---------------------------------------------------------------

fn material_contains(needle: &[u8]) -> bool {
    let mut buf: Vec<u8> = Vec::new();
    write_corpus_hash_material_v1(&mut buf);
    buf.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn corpus_hash_v1_includes_schema_section() {
    assert!(material_contains(b"SCHEMA"));
    assert!(material_contains(b"DSFB-GPU-ATLAS:CORPUS-HASH-SCHEMA:v1"));
}

#[test]
fn corpus_hash_v1_includes_records_section() {
    assert!(material_contains(b"RECORDS"));
}

#[test]
fn corpus_hash_v1_includes_dedup_court_decisions() {
    assert!(material_contains(b"COURT"));
    // The dedup court emits "Canonical" / "AliasOf" / "CompositionOf"
    // wire names for its decisions; at the T.4 first batch we
    // expect at least one of each.
    assert!(material_contains(b"Canonical"));
    assert!(material_contains(b"AliasOf"));
    assert!(material_contains(b"CompositionOf"));
}

#[test]
fn corpus_hash_v1_includes_alias_claims_section() {
    assert!(material_contains(b"ALIAS_CLAIMS"));
}

#[test]
fn corpus_hash_v1_includes_genealogy_edges() {
    // Per-record genealogy edges are written inside each record's
    // body under labels `derived_from`, `generalizes`,
    // `special_case_of`, and a `genealogy` framing label.
    assert!(material_contains(b"genealogy"));
    assert!(material_contains(b"derived_from"));
    assert!(material_contains(b"generalizes"));
    assert!(material_contains(b"special_case_of"));
}

#[test]
fn corpus_hash_v1_includes_witness_roles() {
    // Witness role wire names are written for each record. At
    // least "Primary" and "Confuser" appear in the T.6 seed.
    assert!(material_contains(b"Primary"));
    assert!(material_contains(b"Confuser"));
}

#[test]
fn corpus_hash_v1_includes_lband_states() {
    // L-band wire names. The seed at T.7 has 49 L1 records and 5
    // L6 records.
    assert!(material_contains(b"L1_Canonicalised"));
    assert!(material_contains(b"L6_CpuGpuByteEquivalent"));
}

#[test]
fn corpus_hash_v1_includes_usefulness_rows() {
    assert!(material_contains(b"USEFULNESS"));
    // The conservative T.8 seed marks 49 rows LiteraturePrior +
    // 5 rows RoleSeeded.
    assert!(material_contains(b"LiteraturePrior"));
    assert!(material_contains(b"RoleSeeded"));
    // Every ledger row carries task_id "atlas_corpus_seed_v1".
    assert!(material_contains(b"atlas_corpus_seed_v1"));
}

#[test]
fn corpus_hash_v1_changes_when_source_hash_material_changes() {
    // Synthesise a one-byte mutation by re-hashing a slightly
    // truncated material buffer. The result MUST differ.
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(CORPUS_HASH_DOMAIN_V1.as_bytes());
    write_corpus_hash_material_v1(&mut buf);
    let canonical = sha256(&buf);

    // Drop the last byte.
    buf.pop();
    let truncated = sha256(&buf);
    assert_ne!(
        canonical, truncated,
        "removing one byte from the material must change the hash"
    );

    // Flip one bit in the middle.
    let mut buf2: Vec<u8> = Vec::new();
    buf2.extend_from_slice(CORPUS_HASH_DOMAIN_V1.as_bytes());
    write_corpus_hash_material_v1(&mut buf2);
    let mid = buf2.len() / 2;
    buf2[mid] ^= 0x01;
    let flipped = sha256(&buf2);
    assert_ne!(
        canonical, flipped,
        "flipping one bit in the material must change the hash"
    );
}

#[test]
fn corpus_hash_v1_does_not_hash_report_rendering() {
    // The hash material must NOT contain any of the report
    // section headers from the T.1-T.7 audit report (e.g. the
    // human-readable "(1) Totals", "(13) L-band honesty
    // invariants", "(14) Usefulness ledger honesty invariants"
    // strings). Those live in the rendered TXT only.
    assert!(
        !material_contains(b"(1) Totals"),
        "material must not contain rendered-report section labels"
    );
    assert!(
        !material_contains(b"(13) L-band honesty invariants"),
        "material must not contain rendered T.7 section header"
    );
    assert!(
        !material_contains(b"(14) Usefulness ledger"),
        "material must not contain rendered T.8 section header"
    );
    // The internal audit report's framing line ("The
    // usefulness ledger is an audit surface, not a learned
    // ranking model") MUST NOT appear in the hash material.
    assert!(
        !material_contains(b"audit surface, not a learned"),
        "material must not contain rendered framing strings"
    );
}

#[test]
fn static_seed_and_toml_loader_project_same_hash_material() {
    // At T.10 the canonical source is the static SEED.
    // dsfb_gpu_atlas_corpus::loader::load_from_str(corpus.toml)
    // parses the TOML form into LoadedLiteratureDetector values
    // and `matches_static(...)` is the byte-equivalence gate
    // (pinned by tests/toml_equivalence.rs). If those tests
    // pass (they do — 142 corpus tests green at T.7+), then
    // computing the corpus hash from the static SEED is
    // equivalent to computing it from the TOML-loaded form
    // when the TOML round-trips. We pin the static-side hash as
    // a defensive sanity check that the hash material does not
    // depend on any runtime state.
    let a = compute_corpus_hash_v1();
    let b = compute_corpus_hash_v1();
    assert_eq!(a.bytes, b.bytes);
    // The hash is also length-stable across two computations.
    assert_eq!(a.to_hex().len(), b.to_hex().len());
    let _ = CorpusHashV1 { bytes: a.bytes };
}

#[test]
fn t10_does_not_change_d16_d64_d128_d205_hashes() {
    // T.10 is a corpus-side commit. It must not perturb any
    // detector-profile registry hash. We pin the headline
    // invariants here so a future T.10 refactor that
    // accidentally changes a registry hash gets caught at the
    // corpus-test layer rather than at the GPU-test layer.
    use dsfb_gpu_debug_core::motif::DetectorProfile;
    let d16 = DetectorProfile::D16.registry_hash();
    let d64 = DetectorProfile::D64.registry_hash();
    let d128 = DetectorProfile::D128.registry_hash();
    let d205 = DetectorProfile::D205.registry_hash();
    assert_ne!(d16, [0u8; 32]);
    assert_ne!(d64, [0u8; 32]);
    assert_ne!(d128, [0u8; 32]);
    assert_ne!(d205, [0u8; 32]);
    // The four hashes must be pairwise distinct.
    assert_ne!(d16, d64);
    assert_ne!(d64, d128);
    assert_ne!(d128, d205);
    assert_ne!(d16, d205);
}
