//! T.1a acceptance tests for the literature detector corpus.
//!
//! These tests pin the schema invariants that the verifier enforces,
//! plus a few cross-record invariants the verifier doesn't yet check
//! (T.1a-stable canonical-ID density, alias absence, origin-record
//! provenance, lifecycle baseline). Future T.2..T.9 commits add more
//! invariants here as the court's responsibilities grow.

use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    ConstitutionFlags, DetectorCanonicalId, LifecycleState, LiteratureDetector,
    NegativeWitnessKind, WitnessRole,
};
use dsfb_gpu_atlas_corpus::verify::{verify_corpus, verify_record, VerifyErrorKind};

#[test]
fn t1a_seed_is_non_empty() {
    assert!(!SEED.is_empty(), "T.1a seed must have at least one record");
    // The panel-locked T.1a target is "modest and sharp" - 15 entries.
    // Below this, the schema-dimension coverage is not exercised.
    assert!(
        SEED.len() >= 15,
        "T.1a seed must have at least 15 primitives (current: {})",
        SEED.len()
    );
}

#[test]
fn t1a_seed_passes_verify_clean() {
    let report = verify_corpus(SEED);
    assert!(
        report.is_clean(),
        "verify reported {} errors:\n{}",
        report.errors.len(),
        report
            .errors
            .iter()
            .map(|e| format!("  [{:>3}] {}", e.canonical_id.0, e.kind.describe()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn t1a_canonical_ids_are_dense_starting_at_1() {
    // ID 0 is the null sentinel. T.1a's seed should occupy 1..=N with
    // no gaps so downstream consumers can index into a Vec lookup.
    for (i, record) in SEED.iter().enumerate() {
        let expected = (i + 1) as u32;
        assert_eq!(
            record.canonical_id.0, expected,
            "record at index {i} has canonical_id {} (expected {expected})",
            record.canonical_id.0
        );
    }
}

#[test]
fn t1a_canonical_ids_are_unique() {
    let mut ids: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        before,
        "duplicate canonical_id values in the seed"
    );
}

#[test]
fn t1a_every_record_has_all_constitution_flags_true() {
    for record in SEED {
        let flags = record.constitution_compliance;
        assert!(
            all_flags_true(flags),
            "record `{}` has at least one constitution flag set to false",
            record.display_name
        );
    }
}

#[test]
fn t1a_every_record_is_canonical_head() {
    // T.1a invariant: every entry is a canonical record (no aliases
    // yet). T.1b+ introduces aliases that collapse into these
    // canonicals via the dedup court; the test will need to relax
    // when that lands.
    for record in SEED {
        assert_eq!(
            record.duplicate_group.0, record.canonical_id.0,
            "record `{}` has duplicate_group != canonical_id; T.1a entries must all be canonical",
            record.display_name
        );
    }
}

#[test]
fn t1a_every_record_has_provenance() {
    for record in SEED {
        assert!(
            !record.source_refs.is_empty(),
            "record `{}` has no source_refs",
            record.display_name
        );
        for s in record.source_refs {
            assert!(
                !s.citation_key.is_empty(),
                "record `{}` has SourceRef with empty citation_key",
                record.display_name
            );
            assert!(
                !s.title.is_empty(),
                "record `{}` has SourceRef with empty title",
                record.display_name
            );
            assert!(
                !s.authors.is_empty(),
                "record `{}` has SourceRef with empty authors",
                record.display_name
            );
            if s.year == 0 {
                assert!(
                    !s.notes.is_empty(),
                    "record `{}` has engineering-practice SourceRef `{}` without notes",
                    record.display_name,
                    s.citation_key
                );
            }
        }
    }
}

#[test]
fn t1a_origin_records_have_at_least_one_source_ref() {
    for record in SEED {
        if record.genealogy.is_origin {
            assert!(
                !record.source_refs.is_empty(),
                "origin record `{}` must have at least one SourceRef",
                record.display_name
            );
        }
    }
}

#[test]
fn t1a_non_origin_records_reference_valid_ancestors() {
    let known: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for record in SEED {
        if record.genealogy.is_origin {
            continue;
        }
        assert!(
            !record.genealogy.derived_from.is_empty()
                || !record.genealogy.special_case_of.is_empty()
                || !record.genealogy.generalizes.is_empty(),
            "record `{}` is non-origin but has no genealogy edges",
            record.display_name
        );
        for ancestor in record.genealogy.derived_from {
            assert!(
                known.contains(&ancestor.0),
                "record `{}` references unknown ancestor id {}",
                record.display_name,
                ancestor.0
            );
        }
    }
}

#[test]
fn t1a_negative_witnesses_use_confuser_role() {
    for record in SEED {
        if record.negative_witness_kind == NegativeWitnessKind::NotANegativeWitness {
            continue;
        }
        assert_eq!(
            record.witness_role,
            WitnessRole::Confuser,
            "record `{}` declares a NegativeWitnessKind but its WitnessRole is not Confuser",
            record.display_name
        );
    }
}

#[test]
fn t1a_at_least_one_negative_witness_in_seed() {
    let has = SEED
        .iter()
        .any(|r| r.negative_witness_kind != NegativeWitnessKind::NotANegativeWitness);
    assert!(
        has,
        "T.1a seed must exercise the NegativeWitness lane with at least one entry"
    );
}

#[test]
fn t1a_seed_lifecycle_baseline_is_active() {
    for record in SEED {
        assert_eq!(
            record.lifecycle_state,
            LifecycleState::Active,
            "T.1a seed records start Active; record `{}` has state {:?}",
            record.display_name,
            record.lifecycle_state
        );
    }
}

#[test]
fn t1a_verify_detects_missing_constitution_flag() {
    // Sanity check: the verifier actually catches a missing flag.
    // Synthesise a doctored copy of seed[0] with one flag flipped.
    let mut broken = SEED[0];
    broken.constitution_compliance = ConstitutionFlags {
        declared_provenance: false,
        ..broken.constitution_compliance
    };
    let errs = verify_record(&broken);
    let saw_missing_flag = errs.iter().any(|e| {
        matches!(
            e.kind,
            VerifyErrorKind::MissingConstitutionFlag("declared_provenance")
        )
    });
    assert!(
        saw_missing_flag,
        "verifier did not flag the synthesised missing constitution flag"
    );
}

#[test]
fn t1a_verify_detects_reserved_id_zero() {
    let mut broken = SEED[0];
    broken.canonical_id = DetectorCanonicalId(0);
    broken.duplicate_group = dsfb_gpu_atlas_corpus::types::DuplicateGroupId(0);
    let errs = verify_record(&broken);
    let saw = errs
        .iter()
        .any(|e| matches!(e.kind, VerifyErrorKind::ReservedCanonicalIdZero));
    assert!(saw, "verifier did not flag canonical_id = 0");
}

#[test]
fn t1a_seed_exercises_multiple_primitive_families() {
    let mut families: Vec<_> = SEED.iter().map(|r| r.primitive_family).collect();
    families.sort();
    families.dedup();
    assert!(
        families.len() >= 6,
        "T.1a seed should exercise at least 6 distinct PrimitiveFamily variants (got {})",
        families.len()
    );
}

#[test]
fn t1a_seed_covers_multiple_witness_roles() {
    let mut roles: Vec<_> = SEED.iter().map(|r| r.witness_role).collect();
    roles.sort();
    roles.dedup();
    assert!(
        roles.len() >= 3,
        "T.1a seed should cover at least 3 distinct WitnessRole variants (got {})",
        roles.len()
    );
}

#[test]
fn t1a_seed_covers_multiple_decision_functionals() {
    let mut dfs: Vec<_> = SEED.iter().map(|r| r.decision_functional).collect();
    dfs.sort();
    dfs.dedup();
    assert!(
        dfs.len() >= 4,
        "T.1a seed should cover at least 4 distinct DecisionFunctional variants (got {})",
        dfs.len()
    );
}

#[test]
fn t1a_verify_is_deterministic_across_runs() {
    let a = verify_corpus(SEED);
    let b = verify_corpus(SEED);
    assert_eq!(a.records_inspected, b.records_inspected);
    assert_eq!(a.errors.len(), b.errors.len());
}

// =============================================================
// T.1b acceptance tests: the larger seed proves the schema can
// hold the breadth the panel verdict named (SPC + distance + change-
// point + spectral + data-quality + debug + medical + RF) under one
// deterministic court without losing per-record provenance.
// =============================================================

#[test]
fn t1b_seed_total_is_at_least_45() {
    assert!(
        SEED.len() >= 45,
        "T.1b target is >= 45 primitives (current: {})",
        SEED.len()
    );
}

#[test]
fn t1b_seed_covers_panel_enumerated_families() {
    // Panel verdict named these primitive families as load-bearing
    // for the T.1b breadth claim. Each must appear at least once.
    let panel_required = [
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::ScalarThreshold,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::WindowStatistic,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::SequentialRecurrence,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::DistributionDistance,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::RankStatistic,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::Spectral,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::Wavelet,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::TabularConstraint,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::CategoricalHistogram,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::Missingness,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::ResidualObserver,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::ProjectionResidual,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::MultivariateHypothesis,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::OperabilityDiagnostic,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::DebugObservability,
        dsfb_gpu_atlas_corpus::types::PrimitiveFamily::NegativeWitness,
    ];
    for family in panel_required {
        let count = SEED.iter().filter(|r| r.primitive_family == family).count();
        assert!(
            count >= 1,
            "T.1b seed must include at least one primitive of family {family:?} (got {count})"
        );
    }
}

#[test]
fn t1b_seed_distance_measures_meet_panel_target() {
    // Panel verdict explicitly named distance / distribution measures
    // as a load-bearing breadth dimension: KS, Anderson-Darling, CvM,
    // KL, JS, Hellinger, MMD, Wasserstein, energy distance, total
    // variation, PSI. Target: >= 8 distinct DistributionDistance
    // primitives.
    let count = SEED
        .iter()
        .filter(|r| {
            r.primitive_family
                == dsfb_gpu_atlas_corpus::types::PrimitiveFamily::DistributionDistance
        })
        .count();
    assert!(
        count >= 8,
        "T.1b distance bucket target is >= 8 primitives (got {count})"
    );
}

#[test]
fn t1b_seed_industrial_domain_breadth() {
    // The panel verdict expects significant industrial / FDD coverage
    // (Shewhart, EWMA, CUSUM, Page-Hinkley, WE rules, Nelson, Tukey,
    // Hotelling, PCA T2, PCA SPE, PLS, envelope, sensor bias,
    // stiction, valve hunting, Mann-Kendall, SNHT, MOSUM, Buishand,
    // Pettitt). Industrial-bit count target: >= 14.
    let count = SEED
        .iter()
        .filter(|r| {
            (r.origin_domains.0 & dsfb_gpu_atlas_corpus::types::DomainTagSet::INDUSTRIAL) != 0
        })
        .count();
    assert!(
        count >= 14,
        "T.1b industrial-domain breadth target is >= 14 records (got {count})"
    );
}

#[test]
fn t1b_seed_medical_domain_present() {
    // The panel verdict required at least one medical primitive.
    // R-peak / HRV / QRS / ST-segment should all carry the MEDICAL
    // domain tag.
    let count = SEED
        .iter()
        .filter(|r| (r.origin_domains.0 & dsfb_gpu_atlas_corpus::types::DomainTagSet::MEDICAL) != 0)
        .count();
    assert!(
        count >= 4,
        "T.1b medical-domain target is >= 4 records (got {count})"
    );
}

#[test]
fn t1b_seed_rf_domain_present() {
    let count = SEED
        .iter()
        .filter(|r| {
            (r.origin_domains.0 & dsfb_gpu_atlas_corpus::types::DomainTagSet::RF_COMMS) != 0
        })
        .count();
    assert!(
        count >= 2,
        "T.1b RF-domain target is >= 2 records (got {count})"
    );
}

#[test]
fn t1b_seed_data_quality_breadth() {
    // Data quality bucket: missingness spike, missingness coupling,
    // schema drift, cardinality drift, uniqueness violation, FD
    // violation. Database-domain target: >= 4.
    let count = SEED
        .iter()
        .filter(|r| {
            (r.origin_domains.0 & dsfb_gpu_atlas_corpus::types::DomainTagSet::DATABASE) != 0
        })
        .count();
    assert!(
        count >= 4,
        "T.1b database-domain target is >= 4 records (got {count})"
    );
}

#[test]
fn t1b_engineering_practice_records_are_clearly_labelled() {
    // Any record whose venue STARTS with "engineering practice" (the
    // canonical marker the seed uses for non-academic provenance, as
    // opposed to journal names that merely contain the phrase like
    // `Control Engineering Practice`) MUST carry a non-empty notes
    // field and MUST NOT carry a doi_or_url. This guards against fake
    // DOI placeholders for the DSFB-Debug-derived debug-bank motifs.
    for record in SEED {
        for s in record.source_refs {
            if s.venue_or_source
                .to_lowercase()
                .starts_with("engineering practice")
            {
                assert!(
                    !s.notes.is_empty(),
                    "engineering-practice SourceRef `{}` on record `{}` must carry a non-empty notes field",
                    s.citation_key,
                    record.display_name
                );
                assert!(
                    s.doi_or_url.is_none(),
                    "engineering-practice SourceRef `{}` on record `{}` should NOT carry a doi_or_url (would imply false academic provenance)",
                    s.citation_key,
                    record.display_name
                );
            }
        }
    }
}

#[test]
fn t1b_report_still_renders_for_expanded_seed() {
    // Sanity: the report writer handles the expanded seed without
    // truncating or panicking, and the totals row matches the
    // seed length.
    let body = dsfb_gpu_atlas_corpus::report::render_report(SEED);
    assert!(body.contains("(1) Totals"));
    assert!(body.contains("(2) Per-primitive-family histogram"));
    assert!(body.contains("(6) Witness-role histogram"));
    let expected_total_line = format!("total records              : {}", SEED.len());
    assert!(
        body.contains(&expected_total_line),
        "report does not show the expected total-records line `{expected_total_line}`"
    );
}

#[test]
fn t1b_genealogy_stub_still_renders_for_expanded_seed() {
    let body = dsfb_gpu_atlas_corpus::report::render_genealogy_summary(SEED);
    let expected_total_line = format!("total records              : {}", SEED.len());
    assert!(
        body.contains(&expected_total_line),
        "genealogy stub does not show the expected total-records line"
    );
}

fn all_flags_true(f: ConstitutionFlags) -> bool {
    f.declared_input_contract
        && f.declared_output_type
        && f.declared_deterministic_form
        && f.declared_provenance
        && f.declared_equivalence_status
        && f.declared_witness_role
        && f.declared_activation_conditions
        && f.declared_failure_confuser_modes
}

#[allow(dead_code)]
fn _all_canonical_ids(records: &[LiteratureDetector]) -> Vec<u32> {
    records.iter().map(|r| r.canonical_id.0).collect()
}
