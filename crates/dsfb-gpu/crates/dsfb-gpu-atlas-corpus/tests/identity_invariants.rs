// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.3 acceptance tests: five-hash detector identity invariants.
//!
//! Panel-locked invariants (every one is a test in this file):
//!
//! - `source_hash_does_not_define_detector_identity` (philosophical
//!   load-bearing claim).
//! - Provenance edits change `source_hash` only.
//! - Implementation edits change `implementation_hash` only.
//! - Formula edits change `formula_hash` AND `detector_identity_hash`.
//! - Parameter edits change `parameter_hash` AND
//!   `detector_identity_hash`.
//! - Semantic-role edits change `semantic_role_hash` AND
//!   `detector_identity_hash`.
//! - Two records with the same formula + parameter + semantic role
//!   share `detector_identity_hash` (the dedup court will use this
//!   in T.4).
//! - Static-seed hashes and TOML-loaded hashes are byte-identical.
//! - Hashes are deterministic across two runs.
//! - Every domain separator is versioned (`...:v1\0`) so a future
//!   v2 cannot silently collide with a v1 hash.

use dsfb_gpu_atlas_corpus::dump::dump_to_string;
use dsfb_gpu_atlas_corpus::identity::{
    compute_identity_hashes, compute_identity_hashes_loaded, DetectorIdentityHashes,
    DETECTOR_IDENTITY_DOMAIN, FORMULA_DOMAIN, IMPLEMENTATION_DOMAIN, PARAMETER_DOMAIN,
    SEMANTIC_ROLE_DOMAIN, SOURCE_DOMAIN,
};
use dsfb_gpu_atlas_corpus::loader::load_from_str;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    ConfuserProfile, DecisionFunctional, DeterministicStatus, GpuFamilyKernel, ImplementationLevel,
    LifecycleState, LiteratureDetector, MathFormId, NegativeWitnessKind, PrimitiveFamily,
    SourceRef, WitnessKind, WitnessRole,
};

// =========================================================
// Domain separators are versioned. T.3 invariant: anyone who
// changes the canonical-bytes layout MUST bump the domain
// separator suffix. A future v2 collision with a v1 hash would
// be silently catastrophic; this test makes that impossible.
// =========================================================

#[test]
fn domain_separators_are_versioned() {
    for (name, sep) in [
        ("SOURCE", SOURCE_DOMAIN),
        ("FORMULA", FORMULA_DOMAIN),
        ("PARAMETER", PARAMETER_DOMAIN),
        ("IMPLEMENTATION", IMPLEMENTATION_DOMAIN),
        ("SEMANTIC_ROLE", SEMANTIC_ROLE_DOMAIN),
        ("DETECTOR_IDENTITY", DETECTOR_IDENTITY_DOMAIN),
    ] {
        let text = core::str::from_utf8(sep).expect("domain separator is UTF-8");
        assert!(
            text.starts_with("DSFB-GPU-ATLAS:"),
            "{name} domain separator must carry the architecture prefix"
        );
        assert!(
            text.contains(":v1\0"),
            "{name} domain separator must be versioned (`:v1\\0`) so a future v2 cannot collide"
        );
    }
}

// =========================================================
// Determinism: two runs over the same seed produce identical
// hashes. This pins the load-bearing reproducibility property.
// =========================================================

#[test]
fn identity_hashes_are_deterministic_across_two_runs() {
    for record in SEED {
        let a = compute_identity_hashes(record);
        let b = compute_identity_hashes(record);
        assert_eq!(a, b, "hashes diverged on `{}`", record.display_name);
    }
}

#[test]
fn identity_hashes_are_unique_per_record() {
    // No two records in the T.1b seed should collide on
    // detector_identity_hash. T.4 will detect aliases that
    // SHOULD collide; T.3's seed is canonical (no aliases yet),
    // so all 54 hashes must be distinct.
    let mut hashes: Vec<[u8; 32]> = SEED
        .iter()
        .map(|r| compute_identity_hashes(r).detector_identity_hash)
        .collect();
    let before = hashes.len();
    hashes.sort_unstable();
    hashes.dedup();
    assert_eq!(
        hashes.len(),
        before,
        "duplicate detector_identity_hash in the canonical T.1b seed"
    );
}

// =========================================================
// Philosophical: source_hash and implementation_hash do NOT
// define detector identity.
// =========================================================

#[test]
fn source_hash_does_not_define_detector_identity() {
    // Mutate a record's source_refs (different DOI, different
    // notes, different citation key). The detector_identity_hash
    // MUST be unchanged. This is the load-bearing T.3 claim that
    // the corpus can fix citations without breaking equivalence
    // classes.
    let baseline = SEED[0];
    let altered_source = SourceRef {
        citation_key: "TOTALLY_DIFFERENT_KEY",
        title: "Completely Different Title",
        authors: "Someone Else, Nobody Real",
        year: 9999,
        venue_or_source: "engineering practice (T.3 test fixture)",
        doi_or_url: Some("https://example.invalid/different"),
        notes: "this should not change identity",
    };
    let mut altered = baseline;
    // SAFETY-LIKE: we cannot mutate the `&'static [SourceRef]` of
    // `LiteratureDetector` directly. We synthesise a new record
    // with a different source_refs slice.
    altered.source_refs = Box::leak(Box::new([altered_source])) as &'static [SourceRef];

    let h_baseline = compute_identity_hashes(&baseline);
    let h_altered = compute_identity_hashes(&altered);

    assert_ne!(
        h_baseline.source_hash, h_altered.source_hash,
        "source_hash must change when source_refs change"
    );
    assert_eq!(
        h_baseline.detector_identity_hash, h_altered.detector_identity_hash,
        "detector_identity_hash MUST NOT change when only source_refs change \
         (philosophical T.3 invariant: source is provenance, not identity)"
    );
}

#[test]
fn implementation_hash_does_not_define_detector_identity() {
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.implementation_status = ImplementationLevel::L6_CpuGpuByteEquivalent;
    altered.gpu_family = GpuFamilyKernel::NegativeWitnessFamily;
    altered.deterministic_status = DeterministicStatus::DeterministicConditional;

    let h_baseline = compute_identity_hashes(&baseline);
    let h_altered = compute_identity_hashes(&altered);

    assert_ne!(
        h_baseline.implementation_hash, h_altered.implementation_hash,
        "implementation_hash must change when L-band / gpu_family / \
         deterministic_status change"
    );
    assert_eq!(
        h_baseline.detector_identity_hash, h_altered.detector_identity_hash,
        "detector_identity_hash MUST NOT change when only implementation \
         changes (philosophical T.3 invariant: implementation is execution, \
         not identity)"
    );
}

// =========================================================
// Change-localisation: each component hash isolates its own
// fields.
// =========================================================

#[test]
fn provenance_edits_change_source_hash_only() {
    let baseline = SEED[0];
    let mut altered = baseline;
    let altered_source = SourceRef {
        notes: "different notes only",
        ..baseline.source_refs[0]
    };
    altered.source_refs = Box::leak(Box::new([altered_source])) as &'static [SourceRef];

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_ne!(h_b.source_hash, h_a.source_hash);
    assert_eq!(h_b.formula_hash, h_a.formula_hash);
    assert_eq!(h_b.parameter_hash, h_a.parameter_hash);
    assert_eq!(h_b.implementation_hash, h_a.implementation_hash);
    assert_eq!(h_b.semantic_role_hash, h_a.semantic_role_hash);
    assert_eq!(h_b.detector_identity_hash, h_a.detector_identity_hash);
}

#[test]
fn implementation_edits_change_implementation_hash_only() {
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.implementation_status = ImplementationLevel::L6_CpuGpuByteEquivalent;

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_ne!(h_b.implementation_hash, h_a.implementation_hash);
    assert_eq!(h_b.formula_hash, h_a.formula_hash);
    assert_eq!(h_b.parameter_hash, h_a.parameter_hash);
    assert_eq!(h_b.source_hash, h_a.source_hash);
    assert_eq!(h_b.semantic_role_hash, h_a.semantic_role_hash);
    assert_eq!(h_b.detector_identity_hash, h_a.detector_identity_hash);
}

#[test]
fn formula_edits_change_formula_hash_and_identity() {
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.mathematical_form = MathFormId::CumulativeSum;

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_ne!(h_b.formula_hash, h_a.formula_hash);
    assert_ne!(h_b.detector_identity_hash, h_a.detector_identity_hash);
    assert_eq!(h_b.parameter_hash, h_a.parameter_hash);
    assert_eq!(h_b.source_hash, h_a.source_hash);
    assert_eq!(h_b.implementation_hash, h_a.implementation_hash);
    assert_eq!(h_b.semantic_role_hash, h_a.semantic_role_hash);
}

#[test]
fn parameter_edits_change_parameter_hash_and_identity() {
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.parameter_bounds = dsfb_gpu_atlas_corpus::types::ParameterBounds {
        axis_count: 99,
        description: "totally different parameter grid for the T.3 test",
    };

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_ne!(h_b.parameter_hash, h_a.parameter_hash);
    assert_ne!(h_b.detector_identity_hash, h_a.detector_identity_hash);
    assert_eq!(h_b.formula_hash, h_a.formula_hash);
    assert_eq!(h_b.source_hash, h_a.source_hash);
    assert_eq!(h_b.implementation_hash, h_a.implementation_hash);
    assert_eq!(h_b.semantic_role_hash, h_a.semantic_role_hash);
}

#[test]
fn semantic_role_edits_change_semantic_role_hash_and_identity() {
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.witness_role = WitnessRole::Confuser;
    altered.negative_witness_kind = NegativeWitnessKind::SingleWindowSpikeConfuser;

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_ne!(h_b.semantic_role_hash, h_a.semantic_role_hash);
    assert_ne!(h_b.detector_identity_hash, h_a.detector_identity_hash);
    assert_eq!(h_b.formula_hash, h_a.formula_hash);
    assert_eq!(h_b.parameter_hash, h_a.parameter_hash);
    assert_eq!(h_b.source_hash, h_a.source_hash);
    assert_eq!(h_b.implementation_hash, h_a.implementation_hash);
}

// =========================================================
// Equivalence: two records with the same formula + parameter +
// semantic role share detector_identity_hash. T.4 will use
// this to detect aliases.
// =========================================================

#[test]
fn two_records_with_same_math_share_detector_identity_hash() {
    let template = SEED[0];
    let mut a = template;
    let mut b = template;

    // Diverge only on source + implementation + lifecycle (i.e.
    // everything that is NOT in detector_identity_hash). The two
    // records remain canonically equivalent.
    let alt_source = SourceRef {
        citation_key: "alt1",
        title: "Alternative source A",
        authors: "Someone Different",
        year: 2026,
        venue_or_source: "engineering practice (T.3 test fixture)",
        doi_or_url: None,
        notes: "alt source A",
    };
    a.source_refs = Box::leak(Box::new([alt_source])) as &'static [SourceRef];
    b.source_refs = Box::leak(Box::new([SourceRef {
        citation_key: "alt2",
        title: "Alternative source B",
        ..alt_source
    }])) as &'static [SourceRef];
    a.implementation_status = ImplementationLevel::L3_CpuImplemented;
    b.implementation_status = ImplementationLevel::L6_CpuGpuByteEquivalent;
    a.lifecycle_state = LifecycleState::Dormant;
    b.lifecycle_state = LifecycleState::Active;

    let h_a = compute_identity_hashes(&a);
    let h_b = compute_identity_hashes(&b);

    assert_ne!(h_a.source_hash, h_b.source_hash);
    assert_ne!(h_a.implementation_hash, h_b.implementation_hash);
    assert_eq!(
        h_a.detector_identity_hash, h_b.detector_identity_hash,
        "two records that differ ONLY in source + implementation + \
         lifecycle MUST share detector_identity_hash; this is the \
         identity claim the T.4 dedup court will rely on"
    );
}

// =========================================================
// Cross-source equivalence: hashes from the static seed and
// from the TOML loader are byte-identical.
// =========================================================

#[test]
fn hashes_from_static_seed_match_hashes_from_toml() {
    let toml = dump_to_string(SEED);
    let loaded = load_from_str(&toml).expect("dump -> parse round trip must succeed");
    assert_eq!(loaded.len(), SEED.len());
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        let h_static = compute_identity_hashes(s);
        let h_loaded = compute_identity_hashes_loaded(l);
        assert_eq!(
            h_static, h_loaded,
            "hashes diverge between static seed and TOML loader at canonical_id {} (`{}`)",
            s.canonical_id.0, s.display_name
        );
    }
}

#[test]
fn hashes_from_committed_corpus_file_match_static_seed() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/corpus.toml");
    let toml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let loaded = load_from_str(&toml).expect("committed corpus.toml must parse cleanly");
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        let h_static = compute_identity_hashes(s);
        let h_loaded = compute_identity_hashes_loaded(l);
        assert_eq!(
            h_static.detector_identity_hash, h_loaded.detector_identity_hash,
            "committed corpus.toml hash diverges from static seed at canonical_id {} (`{}`); \
             regenerate corpus.toml with the dump CLI if you changed src/seed.rs",
            s.canonical_id.0, s.display_name
        );
    }
}

// =========================================================
// Hex formatting (used by reports + paper rendering).
// =========================================================

#[test]
fn detector_identity_hex_is_64_lowercase_chars() {
    let h = compute_identity_hashes(&SEED[0]);
    let hex = h.detector_identity_hex();
    assert_eq!(hex.len(), 64, "hex of a 32-byte hash must be 64 chars");
    assert!(
        hex.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "hex must be lowercase hexadecimal"
    );
}

// =========================================================
// Counter-test: changing a non-identity field NEVER touches the
// composite identity hash. Pick a few fields the schema knows
// about but T.3 deliberately omits from identity.
// =========================================================

#[test]
fn lifecycle_edits_do_not_change_any_hash() {
    // lifecycle_state is metadata about court membership, not
    // about identity or implementation. T.3 omits it from every
    // hash. T.4+ may use it independently of identity.
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.lifecycle_state = LifecycleState::Dormant;

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_eq!(h_b, h_a, "lifecycle_state must not affect any T.3 hash");
}

#[test]
fn display_name_and_aliases_do_not_change_any_hash() {
    // display_name / aliases are UI text. They MUST NOT define
    // identity (panel-locked: human display names should never
    // define identity).
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.display_name = "Completely Different Display Name";
    altered.aliases = &["alpha", "beta"];

    let h_b = compute_identity_hashes(&baseline);
    let h_a = compute_identity_hashes(&altered);

    assert_eq!(
        h_b, h_a,
        "display_name and aliases must not affect any T.3 hash"
    );
}

// Sanity: at least one Hash trait implementation exists so the
// DetectorIdentityHashes struct can be used as a HashMap key in
// T.4's dedup court. This is also a smoke test that the derive
// is wired correctly.
#[test]
fn detector_identity_hashes_can_be_used_in_hashmap() {
    use std::collections::HashMap;
    let mut map: HashMap<DetectorIdentityHashes, u32> = HashMap::new();
    for r in SEED.iter().take(5) {
        let h = compute_identity_hashes(r);
        map.insert(h, r.canonical_id.0);
    }
    assert_eq!(map.len(), 5);
}

// Sanity: every enum variant used in identity has a unique
// `as_str` so two different variants cannot collide on
// `formula_hash`.
#[test]
fn enum_wire_names_are_distinct() {
    // Spot-check the families the T.3 hash hits.
    let families = [
        PrimitiveFamily::ScalarThreshold.as_str(),
        PrimitiveFamily::SequentialRecurrence.as_str(),
        PrimitiveFamily::DistributionDistance.as_str(),
    ];
    let mut sorted = families.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), families.len());

    let math_forms = [
        MathFormId::Threshold.as_str(),
        MathFormId::CumulativeSum.as_str(),
        MathFormId::AndersonDarling.as_str(),
    ];
    let mut sorted = math_forms.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), math_forms.len());

    let dfs = [
        DecisionFunctional::TwoSided.as_str(),
        DecisionFunctional::OneSidedUpper.as_str(),
        DecisionFunctional::SequentialStopping.as_str(),
    ];
    let mut sorted = dfs.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), dfs.len());

    let outs = [
        WitnessKind::BooleanCell.as_str(),
        WitnessKind::ScalarMargin.as_str(),
        WitnessKind::Interval.as_str(),
    ];
    let mut sorted = outs.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), outs.len());

    let confusers = [
        ConfuserProfile::None.as_str(),
        ConfuserProfile::SmallSample.as_str(),
        ConfuserProfile::SchemaChange.as_str(),
    ];
    let mut sorted = confusers.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), confusers.len());

    // Reference parameter to anchor LiteratureDetector usage.
    let _ = SEED[0].canonical_id;
}

// Silence unused-import warnings for types referenced only by
// the leak-shaped test fixtures above. Kept as a constant for
// the future T.4 commit to consume.
#[allow(dead_code)]
const _LITERATURE_TYPE_HANDLE: Option<&LiteratureDetector> = None;
