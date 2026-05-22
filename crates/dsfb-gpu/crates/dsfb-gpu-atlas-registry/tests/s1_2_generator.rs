//! S1.2 acceptance tests — literature-bound registry generator
//! + `registry_hash_v2` over the T.10-frozen corpus.
//!
//! Panel-required invariants (12 tests):
//!
//! - `s1_2_generates_one_spec_per_corpus_record_times_grid_point`
//! - `s1_2_specs_have_distinct_detector_ids`
//! - `s1_2_specs_carry_source_corpus_hash_equal_to_corpus_hash_v1`
//! - `s1_2_specs_are_hash_frozen_t10`
//! - `s1_2_specs_have_primitive_id_pointing_at_corpus_canonical_id`
//! - `s1_2_specs_implementation_kind_is_scalar_cpu`
//! - `s1_2_registry_hash_v2_is_deterministic_across_two_builds`
//! - `s1_2_registry_hash_v2_changes_when_a_spec_byte_changes`
//! - `s1_2_registry_hash_v2_changes_when_family_mapping_changes`
//! - `s1_2_registry_hash_v2_changes_when_grid_window_changes`
//! - `verify_registry_spec_admits_every_generated_s1_2_spec_against_live_corpus_hash`
//! - `registry_counts_match_panel_locked_four_tier_definition`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_registry::{
    compute_registry_hash_v2, generate_s1_2_specs, verify_registry_spec, CorpusBindingStatus,
    DetectorFamily, DetectorRegistryV2, ImplementationKind, WindowSpec,
    S1_2_GRID_POINTS_PER_RECORD,
};

#[test]
fn s1_2_generates_one_spec_per_corpus_record_times_grid_point() {
    let specs = generate_s1_2_specs();
    assert_eq!(
        specs.len(),
        SEED.len() * S1_2_GRID_POINTS_PER_RECORD,
        "generator must produce SEED.len() ({}) × {} grid points = {}",
        SEED.len(),
        S1_2_GRID_POINTS_PER_RECORD,
        SEED.len() * S1_2_GRID_POINTS_PER_RECORD
    );
    assert_eq!(specs.len(), 162, "panel-locked count at S1.2 is 162");
}

#[test]
fn s1_2_specs_have_distinct_detector_ids() {
    let specs = generate_s1_2_specs();
    let ids: BTreeSet<u32> = specs.iter().map(|s| s.detector_id.0).collect();
    assert_eq!(
        ids.len(),
        specs.len(),
        "detector_id collisions detected; expected {} distinct ids",
        specs.len()
    );
}

#[test]
fn s1_2_specs_carry_source_corpus_hash_equal_to_corpus_hash_v1() {
    let specs = generate_s1_2_specs();
    let live = compute_corpus_hash_v1();
    for spec in &specs {
        assert_eq!(
            spec.source_corpus_hash, live.bytes,
            "spec {:?} carries source_corpus_hash != live compute_corpus_hash_v1",
            spec.detector_id
        );
    }
}

#[test]
fn s1_2_specs_are_hash_frozen_t10() {
    let specs = generate_s1_2_specs();
    for spec in &specs {
        assert_eq!(
            spec.corpus_binding_status,
            CorpusBindingStatus::HashFrozenT10,
            "spec {:?} must declare HashFrozenT10 at S1.2",
            spec.detector_id
        );
    }
}

#[test]
fn s1_2_specs_have_primitive_id_pointing_at_corpus_canonical_id() {
    let specs = generate_s1_2_specs();
    let canonical_ids: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for spec in &specs {
        let pid = spec
            .primitive_id
            .expect("S1.2 specs MUST carry primitive_id = Some(_)");
        assert!(
            canonical_ids.contains(&pid.0),
            "spec {:?} primitive_id {:?} is not in the corpus SEED canonical-id set",
            spec.detector_id,
            pid
        );
    }
}

#[test]
fn s1_2_specs_implementation_kind_is_scalar_cpu() {
    // Honesty rule: the S1.2 registry generator has no GPU
    // dispatch layer. Every spec MUST declare ScalarCpu even
    // for the five L6 corpus records whose underlying primitives
    // do execute on GPU in dsfb-gpu-debug-core. The GPU surface
    // is claimed by a later commit; the spec does not borrow it.
    let specs = generate_s1_2_specs();
    for spec in &specs {
        assert_eq!(
            spec.implementation_kind,
            ImplementationKind::ScalarCpu,
            "spec {:?} must declare ImplementationKind::ScalarCpu at S1.2 (no GPU claim)",
            spec.detector_id
        );
    }
}

#[test]
fn s1_2_registry_hash_v2_is_deterministic_across_two_builds() {
    let a = generate_s1_2_specs();
    let b = generate_s1_2_specs();
    let ha = compute_registry_hash_v2(&a);
    let hb = compute_registry_hash_v2(&b);
    assert_eq!(
        ha, hb,
        "registry_hash_v2 must be byte-identical across two builds"
    );
}

#[test]
fn s1_2_registry_hash_v2_changes_when_a_spec_byte_changes() {
    // Sensitivity: mutate one byte of one spec's
    // source_corpus_hash → the registry hash must change. This
    // is the load-bearing "stale corpus snapshot can't sneak
    // past the receipt" check.
    let mut specs = generate_s1_2_specs();
    let baseline = compute_registry_hash_v2(&specs);
    specs[0].source_corpus_hash[0] ^= 0xFF;
    let mutated = compute_registry_hash_v2(&specs);
    assert_ne!(
        baseline, mutated,
        "registry_hash_v2 must change when any spec byte changes"
    );
}

#[test]
fn s1_2_registry_hash_v2_changes_when_family_mapping_changes() {
    // Mirror the "family mapping changes" panel invariant by
    // mutating one generated spec's family field. The
    // corpus_to_registry_family mapping is a pure function; its
    // output appears in every spec via spec.family, so swapping
    // a spec's family is equivalent to changing the mapping for
    // that one corpus record.
    let mut specs = generate_s1_2_specs();
    let baseline = compute_registry_hash_v2(&specs);
    // Swap the first spec's family to Shewhart (the
    // corpus_to_registry_family mapping never picks Shewhart
    // for any primitive family in the seed). A deliberate
    // panel-locked perturbation.
    specs[0].family = DetectorFamily::Shewhart;
    let mutated = compute_registry_hash_v2(&specs);
    assert_ne!(
        baseline, mutated,
        "registry_hash_v2 must change when the family mapping changes"
    );
}

#[test]
fn s1_2_registry_hash_v2_changes_when_grid_window_changes() {
    // Mirror the "grid changes" panel invariant by mutating one
    // generated spec's window. The S1.2 grid is panel-locked to
    // (W32, W64, W128) — swapping any grid point to a different
    // window must change the hash on every affected spec.
    let mut specs = generate_s1_2_specs();
    let baseline = compute_registry_hash_v2(&specs);
    // The first generated spec carries WindowSpec::W32 (grid
    // point 0). Swap to W8 — never in the S1.2 grid — so the
    // hash must change.
    specs[0].window = WindowSpec::W8;
    let mutated = compute_registry_hash_v2(&specs);
    assert_ne!(
        baseline, mutated,
        "registry_hash_v2 must change when the parameter grid changes"
    );
}

#[test]
fn verify_registry_spec_admits_every_generated_s1_2_spec_against_live_corpus_hash() {
    let specs = generate_s1_2_specs();
    let live = compute_corpus_hash_v1();
    let canonical_ids: Vec<_> = SEED.iter().map(|r| r.canonical_id).collect();
    for spec in &specs {
        let errors = verify_registry_spec(spec, &live.bytes, &canonical_ids);
        assert!(
            errors.is_empty(),
            "spec {:?} failed registry-level verification: {errors:?}",
            spec.detector_id
        );
    }
}

#[test]
fn registry_counts_match_panel_locked_four_tier_definition() {
    let registry = DetectorRegistryV2::build();
    assert_eq!(
        registry.counts.literature_primitives, 54,
        "literature_primitives count must equal SEED.len() == 54"
    );
    assert_eq!(
        registry.counts.parameterized_specs, 162,
        "parameterized_specs count must equal 54 × 3 = 162"
    );
    assert_eq!(
        registry.counts.active_detectors, 0,
        "active_detectors must be 0 at S1.2 (no activation planner)"
    );
    assert_eq!(
        registry.counts.admitted_episodes, 0,
        "admitted_episodes must be 0 at S1.2 (no GPU execution)"
    );
    let live = compute_corpus_hash_v1();
    assert_eq!(
        registry.source_corpus_hash, live.bytes,
        "registry.source_corpus_hash must equal compute_corpus_hash_v1"
    );
}
