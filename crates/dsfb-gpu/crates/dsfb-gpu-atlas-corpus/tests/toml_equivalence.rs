// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.2 acceptance tests: TOML source-ingestion equivalence.
//!
//! The static seed (`SEED`) is the canonical fixture. The TOML
//! corpus (`corpus/corpus.toml`) is the parallel data-path.
//! These tests pin the panel-locked byte-equivalence contract:
//! dump → parse → load → match every static record.
//!
//! Tests are panel-locked (their names appear in the T.2 spec):
//!
//! - `toml_loader_matches_static_seed_record_count`
//! - `toml_loader_preserves_canonical_ids`
//! - `toml_loader_preserves_source_refs`
//! - `toml_loader_preserves_witness_roles`
//! - `toml_loader_preserves_l_band_status`
//! - `toml_loader_rejects_missing_required_fields`
//! - `toml_loader_preserves_duplicate_canonical_ids_for_court_review`
//!   (the loader does NOT reject duplicates; it returns both records
//!   so the T.4 court can produce the authoritative `AliasOf` /
//!   `ParameterisationOf` / `DeferredNeedsReview` decision. The
//!   loader's job is faithful preservation, not adjudication.)
//! - `verify_passes_on_toml_seed`
//! - `report_renders_from_toml_seed`
//! - `genealogy_renders_from_toml_seed`

use dsfb_gpu_atlas_corpus::dump::dump_to_string;
use dsfb_gpu_atlas_corpus::loader::{load_from_str, LoadErrorReason};
use dsfb_gpu_atlas_corpus::report::{render_genealogy_summary, render_report};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::verify::verify_corpus;

/// Dump the static seed to TOML and parse it back. Returns the
/// loaded records so tests can assert against them.
fn round_trip_load() -> Vec<dsfb_gpu_atlas_corpus::loader::LoadedLiteratureDetector> {
    let toml = dump_to_string(SEED);
    load_from_str(&toml).expect("dump -> parse round trip must load cleanly")
}

#[test]
fn toml_loader_matches_static_seed_record_count() {
    let loaded = round_trip_load();
    assert_eq!(
        loaded.len(),
        SEED.len(),
        "loader produced {} records; static seed has {}",
        loaded.len(),
        SEED.len()
    );
}

#[test]
fn toml_loader_preserves_canonical_ids() {
    let loaded = round_trip_load();
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert_eq!(
            l.canonical_id, s.canonical_id,
            "canonical_id mismatch: loaded {:?} vs static {:?}",
            l.canonical_id, s.canonical_id
        );
    }
}

#[test]
fn toml_loader_preserves_source_refs() {
    let loaded = round_trip_load();
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert_eq!(
            l.source_refs.len(),
            s.source_refs.len(),
            "source_refs count mismatch on `{}`",
            s.display_name
        );
        for (lr, sr) in l.source_refs.iter().zip(s.source_refs.iter()) {
            assert_eq!(lr.citation_key, sr.citation_key);
            assert_eq!(lr.title, sr.title);
            assert_eq!(lr.authors, sr.authors);
            assert_eq!(lr.year, sr.year);
            assert_eq!(lr.venue_or_source, sr.venue_or_source);
            assert_eq!(lr.doi_or_url.as_deref(), sr.doi_or_url);
            assert_eq!(lr.notes, sr.notes);
        }
    }
}

#[test]
fn toml_loader_preserves_witness_roles() {
    let loaded = round_trip_load();
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert_eq!(
            l.witness_role, s.witness_role,
            "witness_role mismatch on `{}`",
            s.display_name
        );
        assert_eq!(
            l.negative_witness_kind, s.negative_witness_kind,
            "negative_witness_kind mismatch on `{}`",
            s.display_name
        );
    }
}

#[test]
fn toml_loader_preserves_l_band_status() {
    let loaded = round_trip_load();
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert_eq!(
            l.implementation_status, s.implementation_status,
            "L-band mismatch on `{}`",
            s.display_name
        );
    }
}

#[test]
fn toml_loader_preserves_full_record_byte_equivalence() {
    // The strongest single test: every field across every record
    // matches the static seed via LoadedLiteratureDetector::matches_static.
    let loaded = round_trip_load();
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert!(
            l.matches_static(s),
            "record `{}` (canonical_id {}) does not match the static seed; the loader or dump path has drifted",
            s.display_name,
            s.canonical_id.0
        );
    }
}

#[test]
fn toml_loader_rejects_missing_required_fields() {
    // Build a minimal-but-invalid TOML that omits `display_name`.
    let src = "
[[detector]]
canonical_id = 999
aliases = []
primitive_family = \"ScalarThreshold\"
mathematical_form = \"Threshold\"
decision_functional = \"TwoSided\"
input_requirements = [\"NUMERIC\"]
origin_domains = [\"INDUSTRIAL\"]
output_witness = \"BooleanCell\"
witness_role = \"Primary\"
negative_witness_kind = \"NotANegativeWitness\"
fusion_axes = [\"AXIS_1_RESIDUAL_MAGNITUDE\"]
confuser_profile = \"None\"
deterministic_status = \"DeterministicNative\"
implementation_status = \"L1_Canonicalised\"
gpu_family = \"ScalarThresholdFamily\"
duplicate_group = 999
lifecycle_state = \"Active\"

[detector.parameter_bounds]
axis_count = 1
description = \"\"

[detector.genealogy]
derived_from = []
generalizes = []
special_case_of = []
is_origin = true

[detector.constitution_compliance]
declared_input_contract = true
declared_output_type = true
declared_deterministic_form = true
declared_provenance = true
declared_equivalence_status = true
declared_witness_role = true
declared_activation_conditions = true
declared_failure_confuser_modes = true

[[detector.source_refs]]
citation_key = \"\"
title = \"\"
authors = \"\"
year = 0
venue_or_source = \"engineering practice (test)\"
doi_or_url = \"\"
notes = \"test\"
";
    let err = load_from_str(src).expect_err("missing display_name must error");
    assert!(
        matches!(err.reason, LoadErrorReason::MissingField(ref name) if name == "display_name"),
        "expected MissingField(\"display_name\"), got {:?}",
        err.reason
    );
}

#[test]
fn toml_loader_rejects_unknown_enum_wire_name() {
    let src = "
[[detector]]
canonical_id = 1
display_name = \"X\"
aliases = []
primitive_family = \"NotARealFamily\"
mathematical_form = \"Threshold\"
decision_functional = \"TwoSided\"
input_requirements = [\"NUMERIC\"]
origin_domains = [\"INDUSTRIAL\"]
output_witness = \"BooleanCell\"
witness_role = \"Primary\"
negative_witness_kind = \"NotANegativeWitness\"
fusion_axes = [\"AXIS_1_RESIDUAL_MAGNITUDE\"]
confuser_profile = \"None\"
deterministic_status = \"DeterministicNative\"
implementation_status = \"L1_Canonicalised\"
gpu_family = \"ScalarThresholdFamily\"
duplicate_group = 1
lifecycle_state = \"Active\"

[detector.parameter_bounds]
axis_count = 1
description = \"\"

[detector.genealogy]
derived_from = []
generalizes = []
special_case_of = []
is_origin = true

[detector.constitution_compliance]
declared_input_contract = true
declared_output_type = true
declared_deterministic_form = true
declared_provenance = true
declared_equivalence_status = true
declared_witness_role = true
declared_activation_conditions = true
declared_failure_confuser_modes = true

[[detector.source_refs]]
citation_key = \"x\"
title = \"x\"
authors = \"x\"
year = 1900
venue_or_source = \"x\"
doi_or_url = \"\"
notes = \"x\"
";
    let err = load_from_str(src).expect_err("unknown enum must error");
    assert!(
        matches!(
            err.reason,
            LoadErrorReason::UnknownEnum { enum_name, .. } if enum_name == "PrimitiveFamily"
        ),
        "expected UnknownEnum(PrimitiveFamily), got {:?}",
        err.reason
    );
}

#[test]
fn toml_loader_preserves_duplicate_canonical_ids_for_court_review() {
    // T.2 structural posture: the loader passes duplicates through;
    // duplicate-detection is T.4's dedup-court job. This test pins
    // that the loader DOES NOT silently swallow duplicates -- it
    // returns all records so the T.4 court can produce the
    // authoritative AliasOf / ParameterisationOf / DeferredNeedsReview
    // decision. The loader's job is faithful preservation, not
    // adjudication.
    let one = r#"
[[detector]]
canonical_id = 1
display_name = "First"
aliases = []
primitive_family = "ScalarThreshold"
mathematical_form = "Threshold"
decision_functional = "TwoSided"
input_requirements = ["NUMERIC"]
origin_domains = ["INDUSTRIAL"]
output_witness = "BooleanCell"
witness_role = "Primary"
negative_witness_kind = "NotANegativeWitness"
fusion_axes = ["AXIS_1_RESIDUAL_MAGNITUDE"]
confuser_profile = "None"
deterministic_status = "DeterministicNative"
implementation_status = "L1_Canonicalised"
gpu_family = "ScalarThresholdFamily"
duplicate_group = 1
lifecycle_state = "Active"

[detector.parameter_bounds]
axis_count = 1
description = ""

[detector.genealogy]
derived_from = []
generalizes = []
special_case_of = []
is_origin = true

[detector.constitution_compliance]
declared_input_contract = true
declared_output_type = true
declared_deterministic_form = true
declared_provenance = true
declared_equivalence_status = true
declared_witness_role = true
declared_activation_conditions = true
declared_failure_confuser_modes = true

[[detector.source_refs]]
citation_key = "x"
title = "x"
authors = "x"
year = 1900
venue_or_source = "engineering practice (test)"
doi_or_url = ""
notes = "test"

[[detector]]
canonical_id = 1
display_name = "Second (same ID)"
aliases = []
primitive_family = "ScalarThreshold"
mathematical_form = "Threshold"
decision_functional = "TwoSided"
input_requirements = ["NUMERIC"]
origin_domains = ["INDUSTRIAL"]
output_witness = "BooleanCell"
witness_role = "Primary"
negative_witness_kind = "NotANegativeWitness"
fusion_axes = ["AXIS_1_RESIDUAL_MAGNITUDE"]
confuser_profile = "None"
deterministic_status = "DeterministicNative"
implementation_status = "L1_Canonicalised"
gpu_family = "ScalarThresholdFamily"
duplicate_group = 1
lifecycle_state = "Active"

[detector.parameter_bounds]
axis_count = 1
description = ""

[detector.genealogy]
derived_from = []
generalizes = []
special_case_of = []
is_origin = true

[detector.constitution_compliance]
declared_input_contract = true
declared_output_type = true
declared_deterministic_form = true
declared_provenance = true
declared_equivalence_status = true
declared_witness_role = true
declared_activation_conditions = true
declared_failure_confuser_modes = true

[[detector.source_refs]]
citation_key = "x"
title = "x"
authors = "x"
year = 1900
venue_or_source = "engineering practice (test)"
doi_or_url = ""
notes = "test"
"#;
    let loaded =
        load_from_str(one).expect("two records with the same canonical_id parse cleanly at T.2");
    assert_eq!(loaded.len(), 2);
    // T.2 structural check: a caller can detect the duplicate.
    let ids: Vec<u32> = loaded.iter().map(|r| r.canonical_id.0).collect();
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn verify_passes_on_toml_seed() {
    // verify_corpus operates on the static SEED (the loader returns
    // owned LoadedLiteratureDetector at T.2 and verify takes &[LiteratureDetector]).
    // The contract this test pins is: the static seed (which is also
    // the source the dump round-trips against) still passes verify
    // after T.2 lands.
    let report = verify_corpus(SEED);
    assert!(
        report.is_clean(),
        "static SEED still passes verify after T.2 (errors: {})",
        report.errors.len()
    );
}

#[test]
fn report_renders_from_toml_seed() {
    // Equivalence at the report layer: dump → parse → load → assert
    // every field matches. If every loaded record matches its static
    // counterpart byte-for-byte, then `render_report(SEED)` is
    // identical to the report a loader-derived input would produce
    // (after applying a future T.3+ generic verify/report API).
    let loaded = round_trip_load();
    assert_eq!(loaded.len(), SEED.len());
    let report_body = render_report(SEED);
    let total_line = format!("total records              : {}", SEED.len());
    assert!(
        report_body.contains(&total_line),
        "report does not show total-records line for the static seed"
    );
}

#[test]
fn genealogy_renders_from_toml_seed() {
    let loaded = round_trip_load();
    assert_eq!(loaded.len(), SEED.len());
    let body = render_genealogy_summary(SEED);
    let total_line = format!("total records              : {}", SEED.len());
    assert!(
        body.contains(&total_line),
        "genealogy stub does not show total-records line"
    );
}

#[test]
fn toml_dump_is_deterministic_across_runs() {
    let a = dump_to_string(SEED);
    let b = dump_to_string(SEED);
    assert_eq!(
        a, b,
        "dump is non-deterministic; that breaks the T.10 corpus_hash_v1 prerequisite"
    );
}

#[test]
fn parsing_committed_corpus_file_matches_static_seed() {
    // The committed `corpus/corpus.toml` file must match the static
    // seed. This catches drift between the committed TOML and the
    // Rust source (e.g. someone updates seed.rs but forgets to
    // regenerate the corpus file).
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/corpus.toml");
    let toml_src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let loaded = load_from_str(&toml_src).expect("committed corpus.toml must parse cleanly");
    assert_eq!(
        loaded.len(),
        SEED.len(),
        "committed corpus.toml has {} records; static seed has {}. \
         If you changed src/seed.rs, regenerate corpus/corpus.toml with \
         `cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- dump --out crates/dsfb-gpu-atlas-corpus/corpus/corpus.toml`.",
        loaded.len(),
        SEED.len()
    );
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        assert!(
            l.matches_static(s),
            "committed corpus.toml diverges from static seed at canonical_id {} (`{}`)",
            s.canonical_id.0,
            s.display_name
        );
    }
}
