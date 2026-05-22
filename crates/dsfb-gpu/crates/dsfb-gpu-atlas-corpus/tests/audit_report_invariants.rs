// Tests legitimately panic on assertion failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.9 acceptance tests: internal corpus audit report invariants.
//!
//! Panel-locked tests (each name documents an invariant). T.9
//! produces an INTERNAL audit bundle (TXT + JSON + DOT +
//! genealogy-JSON); these tests pin its honesty and cross-source
//! consistency. None of these tests claim usefulness, GPU
//! execution, or measured ranking — they only pin that the
//! bundle is deterministic, internally consistent, and
//! publication-free.
//!
//! Bundle shape / determinism:
//! - `bundle_emits_four_artifacts`
//! - `bundle_is_byte_deterministic_across_two_calls`
//! - `json_schema_field_is_corpus_audit_report_v1`
//! - `txt_carries_internal_audit_header`
//! - `txt_and_json_counts_agree`
//!
//! Cross-source consistency (counts in bundle match T.1–T.8 sources):
//! - `counts_literature_primitives_matches_seed_length`
//! - `counts_alias_claims_matches_claims_length`
//! - `counts_court_subjects_matches_classify_all_total`
//! - `court_summary_matches_classify_all`
//! - `genealogy_summary_matches_graph_builder`
//! - `lband_histogram_matches_lband_compute_histogram`
//! - `usefulness_summary_matches_ledger_helpers`
//!
//! Honesty invariants:
//! - `aliases_not_counted_as_unique_primitives`
//! - `l0_to_l4_not_described_as_gpu_ready`
//! - `l7_l8_counts_are_zero_at_t9`
//! - `l7_l8_marked_forbidden_in_txt`
//! - `usefulness_ledger_remains_notscored_at_t9`
//! - `no_fabricated_claims_at_t9`
//!
//! Publication-language exclusion:
//! - `txt_contains_no_publication_language`
//! - `json_contains_no_publication_language`
//! - `txt_contains_no_zenodo_marker`
//! - `txt_contains_no_doi_marker`
//! - `txt_explicitly_says_internal_audit_not_release`
//!
//! Limitations / deferred-gates content:
//! - `limitations_section_mentions_t10_t11_deferral`
//! - `deferred_gates_lists_t10_corpus_hash_v1`
//! - `deferred_gates_lists_section_s_phase_1`
//!
//! Genealogy bundle:
//! - `genealogy_dot_matches_t5_export`
//! - `genealogy_json_matches_t5_export`

use dsfb_gpu_atlas_corpus::audit_report::{
    collect_audit_report_data, contains_publication_language, generate_audit_report_bundle,
    render_audit_report_json, render_audit_report_txt, AUDIT_REPORT_SCHEMA,
};
use dsfb_gpu_atlas_corpus::claims::CLAIMS;
use dsfb_gpu_atlas_corpus::court::{classify_all, count_decisions};
use dsfb_gpu_atlas_corpus::genealogy::{build_genealogy, export_dot, export_json};
use dsfb_gpu_atlas_corpus::lband::compute_histogram as compute_lband_histogram;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::usefulness::{
    compute_evidence_histogram, compute_lifecycle_histogram, USEFULNESS_LEDGER,
};

// ---------------------------------------------------------------
// Bundle shape / determinism.
// ---------------------------------------------------------------

#[test]
fn bundle_emits_four_artifacts() {
    let bundle = generate_audit_report_bundle();
    assert!(
        !bundle.audit_report_txt.is_empty(),
        "TXT artifact must not be empty"
    );
    assert!(
        !bundle.audit_report_json.is_empty(),
        "JSON artifact must not be empty"
    );
    assert!(
        !bundle.genealogy_dot.is_empty(),
        "genealogy DOT must not be empty"
    );
    assert!(
        !bundle.genealogy_json.is_empty(),
        "genealogy JSON must not be empty"
    );
}

#[test]
fn bundle_is_byte_deterministic_across_two_calls() {
    let a = generate_audit_report_bundle();
    let b = generate_audit_report_bundle();
    assert_eq!(a.audit_report_txt, b.audit_report_txt);
    assert_eq!(a.audit_report_json, b.audit_report_json);
    assert_eq!(a.genealogy_dot, b.genealogy_dot);
    assert_eq!(a.genealogy_json, b.genealogy_json);
}

#[test]
fn json_schema_field_is_corpus_audit_report_v1() {
    let bundle = generate_audit_report_bundle();
    let expected = format!("\"schema\": \"{AUDIT_REPORT_SCHEMA}\"");
    assert!(
        bundle.audit_report_json.contains(&expected),
        "JSON must declare schema={AUDIT_REPORT_SCHEMA}"
    );
    assert_eq!(
        AUDIT_REPORT_SCHEMA, "DSFB-GPU-ATLAS:CORPUS-AUDIT-REPORT:v1",
        "schema constant must be the panel-locked CORPUS-AUDIT-REPORT:v1"
    );
}

#[test]
fn txt_carries_internal_audit_header() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle
            .audit_report_txt
            .contains("DSFB-GPU-Atlas Corpus Internal Audit Report"),
        "TXT header must explicitly say 'Internal Audit Report'"
    );
}

#[test]
fn txt_and_json_counts_agree() {
    let data = collect_audit_report_data();
    let txt = render_audit_report_txt(&data);
    let json = render_audit_report_json(&data);
    // Sample five integers and assert both renders carry them.
    let samples = [
        data.counts.literature_primitives,
        data.counts.alias_claims,
        data.counts.court_subjects,
        data.court.counts.canonical,
        data.usefulness.rows_loaded,
    ];
    for s in samples {
        let needle = s.to_string();
        assert!(txt.contains(&needle), "TXT must contain count {s}");
        assert!(json.contains(&needle), "JSON must contain count {s}");
    }
}

// ---------------------------------------------------------------
// Cross-source consistency.
// ---------------------------------------------------------------

#[test]
fn counts_literature_primitives_matches_seed_length() {
    let data = collect_audit_report_data();
    assert_eq!(data.counts.literature_primitives, SEED.len());
}

#[test]
fn counts_alias_claims_matches_claims_length() {
    let data = collect_audit_report_data();
    assert_eq!(data.counts.alias_claims, CLAIMS.len());
}

#[test]
fn counts_court_subjects_matches_classify_all_total() {
    let data = collect_audit_report_data();
    let records = classify_all();
    let counts = count_decisions(&records);
    assert_eq!(data.counts.court_subjects, counts.total());
}

#[test]
fn court_summary_matches_classify_all() {
    let data = collect_audit_report_data();
    let records = classify_all();
    let counts = count_decisions(&records);
    assert_eq!(data.court.counts.canonical, counts.canonical);
    assert_eq!(data.court.counts.aliases, counts.aliases);
    assert_eq!(
        data.court.counts.parameterisations,
        counts.parameterisations
    );
    assert_eq!(data.court.counts.compositions, counts.compositions);
    assert_eq!(
        data.court.counts.stochastic_reductions,
        counts.stochastic_reductions
    );
    assert_eq!(data.court.counts.rejected, counts.rejected);
    assert_eq!(data.court.counts.deferred, counts.deferred);
}

#[test]
fn genealogy_summary_matches_graph_builder() {
    let data = collect_audit_report_data();
    let graph = build_genealogy();
    assert_eq!(data.genealogy.nodes, graph.nodes.len());
    assert_eq!(data.genealogy.edges, graph.edges.len());
}

#[test]
fn lband_histogram_matches_lband_compute_histogram() {
    let data = collect_audit_report_data();
    let h = compute_lband_histogram(SEED);
    assert_eq!(data.lband.counts[0], h.l0);
    assert_eq!(data.lband.counts[1], h.l1);
    assert_eq!(data.lband.counts[2], h.l2);
    assert_eq!(data.lband.counts[3], h.l3);
    assert_eq!(data.lband.counts[4], h.l4);
    assert_eq!(data.lband.counts[5], h.l5);
    assert_eq!(data.lband.counts[6], h.l6);
    assert_eq!(data.lband.counts[7], h.l7);
    assert_eq!(data.lband.counts[8], h.l8);
}

#[test]
fn usefulness_summary_matches_ledger_helpers() {
    let data = collect_audit_report_data();
    let ev = compute_evidence_histogram(USEFULNESS_LEDGER);
    let lc = compute_lifecycle_histogram(USEFULNESS_LEDGER);
    assert_eq!(data.usefulness.rows_loaded, USEFULNESS_LEDGER.len());
    assert_eq!(data.usefulness.evidence_level_counts[0], ev.unmeasured);
    assert_eq!(
        data.usefulness.evidence_level_counts[1],
        ev.literature_prior
    );
    assert_eq!(data.usefulness.evidence_level_counts[2], ev.role_seeded);
    assert_eq!(data.usefulness.lifecycle_counts[0], lc.active);
    assert_eq!(data.usefulness.lifecycle_counts[1], lc.dormant);
}

// ---------------------------------------------------------------
// Honesty invariants.
// ---------------------------------------------------------------

#[test]
fn aliases_not_counted_as_unique_primitives() {
    let data = collect_audit_report_data();
    // The literature_primitives count is SEED only (54). The
    // alias_claims count is CLAIMS only (12). They are NEVER
    // summed into the literature_primitives headline.
    assert_eq!(data.counts.literature_primitives, SEED.len());
    assert_eq!(data.counts.alias_claims, CLAIMS.len());
    assert_ne!(
        data.counts.literature_primitives,
        SEED.len() + CLAIMS.len(),
        "literature_primitives must NOT include alias claims"
    );
}

#[test]
fn l0_to_l4_not_described_as_gpu_ready() {
    let bundle = generate_audit_report_bundle();
    // The phrase 'GPU-implemented whitelist size' must appear and
    // refer to the L5/L6 surface only. Verify no line of the TXT
    // describes L0/L1/L2/L3/L4 as GPU-ready by looking for
    // 'L0 ... GPU-ready' or 'L4 ... GPU-ready' patterns.
    let txt = bundle.audit_report_txt.to_lowercase();
    for forbidden in [
        "l0_citedonly : gpu",
        "l1_canonicalised : gpu",
        "l2_deterministicformula : gpu",
        "l3_cpuimplemented : gpu",
        "l4_cpuverified : gpu",
        "l0-l4 are gpu",
        "l1-l4 are gpu",
        "l0..l4 are gpu",
    ] {
        assert!(
            !txt.contains(forbidden),
            "TXT must not describe L0-L4 as GPU-ready: hit `{forbidden}`"
        );
    }
}

#[test]
fn l7_l8_counts_are_zero_at_t9() {
    let data = collect_audit_report_data();
    assert_eq!(
        data.lband.counts[7], 0,
        "T.9 must report zero L7_BenchmarkCharacterised records"
    );
    assert_eq!(
        data.lband.counts[8], 0,
        "T.9 must report zero L8_LedgerCharacterised records"
    );
}

#[test]
fn l7_l8_marked_forbidden_in_txt() {
    let bundle = generate_audit_report_bundle();
    let lband_block = bundle
        .audit_report_txt
        .split("(9) Usefulness ledger")
        .next()
        .expect("L-band section precedes usefulness section");
    assert!(
        lband_block.contains("L7_BenchmarkCharacterised")
            && lband_block.contains("forbidden at T.9"),
        "TXT L-band block must mark L7 as forbidden at T.9"
    );
    assert!(
        lband_block.contains("L8_LedgerCharacterised") && lband_block.contains("forbidden at T.9"),
        "TXT L-band block must mark L8 as forbidden at T.9"
    );
}

#[test]
fn usefulness_ledger_remains_notscored_at_t9() {
    // At T.9, every USEFULNESS_LEDGER row must be NotScored.
    // Section 9 of the TXT explicitly says so.
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle
            .audit_report_txt
            .contains("At T.9 every row is NotScored."),
        "TXT Section 9 must declare the NotScored invariant"
    );
}

#[test]
fn no_fabricated_claims_at_t9() {
    let data = collect_audit_report_data();
    assert_eq!(
        data.usefulness.fabricated_claims, 0,
        "T.9 must report zero rows with fabricated empirical claims; got {}",
        data.usefulness.fabricated_claims
    );
}

// ---------------------------------------------------------------
// Publication-language exclusion.
// ---------------------------------------------------------------

#[test]
fn txt_contains_no_publication_language() {
    let bundle = generate_audit_report_bundle();
    assert!(
        !contains_publication_language(&bundle.audit_report_txt),
        "TXT must not contain any of zenodo / doi / deposit / publication-grade markers"
    );
}

#[test]
fn json_contains_no_publication_language() {
    let bundle = generate_audit_report_bundle();
    assert!(
        !contains_publication_language(&bundle.audit_report_json),
        "JSON must not contain any of zenodo / doi / deposit / publication-grade markers"
    );
}

#[test]
fn txt_contains_no_zenodo_marker() {
    let bundle = generate_audit_report_bundle();
    assert!(
        !bundle.audit_report_txt.to_lowercase().contains("zenodo"),
        "TXT must not mention Zenodo"
    );
}

#[test]
fn txt_contains_no_doi_marker() {
    let bundle = generate_audit_report_bundle();
    assert!(
        !bundle.audit_report_txt.to_lowercase().contains("doi"),
        "TXT must not mention DOI"
    );
}

#[test]
fn txt_explicitly_says_internal_audit_not_release() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle.audit_report_txt.contains("internal audit"),
        "TXT must explicitly say 'internal audit'"
    );
    assert!(
        bundle.audit_report_txt.contains("NOT a release"),
        "TXT must explicitly say 'NOT a release artifact'"
    );
}

// ---------------------------------------------------------------
// Limitations / deferred-gates content.
// ---------------------------------------------------------------

#[test]
fn limitations_section_mentions_t10_t11_deferral() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle.audit_report_txt.contains("deferred to T.10/T.11")
            || bundle.audit_report_txt.contains("deferred to T.10"),
        "Limitations section must name T.10/T.11 deferral"
    );
}

#[test]
fn deferred_gates_lists_t10_corpus_hash_v1() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle.audit_report_txt.contains("T.10 corpus_hash_v1"),
        "Deferred-gates section must list T.10 corpus_hash_v1"
    );
    assert!(
        bundle.audit_report_json.contains("T.10 corpus_hash_v1"),
        "Deferred-gates JSON array must include T.10 corpus_hash_v1"
    );
}

#[test]
fn deferred_gates_lists_section_s_phase_1() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle.audit_report_txt.contains("Section S Phase 1"),
        "Deferred-gates section must list Section S Phase 1"
    );
}

// ---------------------------------------------------------------
// Genealogy bundle.
// ---------------------------------------------------------------

#[test]
fn genealogy_dot_matches_t5_export() {
    let bundle = generate_audit_report_bundle();
    let graph = build_genealogy();
    let direct = export_dot(&graph);
    assert_eq!(
        bundle.genealogy_dot, direct,
        "T.9 genealogy DOT must be byte-identical to the T.5 export"
    );
}

#[test]
fn genealogy_json_matches_t5_export() {
    let bundle = generate_audit_report_bundle();
    let graph = build_genealogy();
    let direct = export_json(&graph);
    assert_eq!(
        bundle.genealogy_json, direct,
        "T.9 genealogy JSON must be byte-identical to the T.5 export"
    );
}

#[test]
fn txt_lists_ten_top_level_sections() {
    let bundle = generate_audit_report_bundle();
    for n in 1..=10 {
        let header = format!("({n}) ");
        assert!(
            bundle.audit_report_txt.contains(&header),
            "TXT must contain section header `{header}`"
        );
    }
}

#[test]
fn json_contains_each_top_level_block() {
    let bundle = generate_audit_report_bundle();
    for key in [
        "\"schema\"",
        "\"generated_by_section\"",
        "\"corpus_release_state\"",
        "\"counts\"",
        "\"primitive_family_histogram\"",
        "\"court_summary\"",
        "\"identity_summary\"",
        "\"genealogy_summary\"",
        "\"witness_summary\"",
        "\"lband_summary\"",
        "\"usefulness_summary\"",
        "\"limitations\"",
        "\"deferred_gates\"",
    ] {
        assert!(
            bundle.audit_report_json.contains(key),
            "JSON must contain top-level key {key}"
        );
    }
}

#[test]
fn release_stage_says_internal_audit_pre_freeze() {
    let bundle = generate_audit_report_bundle();
    assert!(
        bundle
            .audit_report_txt
            .contains("internal-audit-pre-freeze"),
        "TXT must declare release stage = internal-audit-pre-freeze"
    );
    assert!(
        bundle
            .audit_report_json
            .contains("\"stage\": \"internal-audit-pre-freeze\""),
        "JSON must declare stage = internal-audit-pre-freeze"
    );
}
