// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.6 acceptance tests: witness-role and fusion-axis semantics.
//!
//! Panel-locked tests (each name documents an invariant):
//!
//! - `every_detector_has_witness_role` (structurally guaranteed by
//!   the enum-typed schema; the test asserts the seed exercises
//!   multiple roles, not a default fallback).
//! - `every_detector_has_at_least_one_axis_binding`
//! - `confuser_roles_have_negative_witness_kind`
//! - `primary_witnesses_cannot_be_negative_only`
//! - `clean_window_witnesses_cannot_admit_episode_alone`
//! - `semantic_role_hash_changes_when_role_changes` (cross-reference
//!   to T.3's identity tests).
//! - `axis_binding_hash_changes_when_axis_changes`
//! - `axis_bindings_are_deterministically_sorted`
//! - `report_contains_witness_role_histogram`
//! - `report_contains_fusion_plane_histogram`
//! - `axes_to_planes_is_deterministic`
//! - `every_axis_maps_to_at_least_one_plane`

use dsfb_gpu_atlas_corpus::fusion::{
    axes_to_planes, clean_window_can_admit_episode_alone, lookup_verdict,
    primary_may_carry_negative_witness_kind, CompatibilityVerdict, FusionPlane, FusionPlaneSet,
    COMPATIBILITY_RULES,
};
use dsfb_gpu_atlas_corpus::identity::compute_identity_hashes;
use dsfb_gpu_atlas_corpus::report::render_report;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{AxisBindingSet, NegativeWitnessKind, WitnessRole};

#[test]
fn every_detector_has_witness_role() {
    // The enum schema makes a missing role structurally impossible,
    // but the seed must exercise *real* roles (not all be Primary).
    // T.6 expects at least 3 distinct roles in the canonical seed.
    let mut roles: Vec<WitnessRole> = SEED.iter().map(|r| r.witness_role).collect();
    roles.sort_unstable();
    roles.dedup();
    assert!(
        roles.len() >= 3,
        "expected at least 3 distinct witness roles in the seed; got {}",
        roles.len()
    );
}

#[test]
fn every_detector_has_at_least_one_axis_binding() {
    for r in SEED {
        assert!(
            !r.fusion_axes.is_empty(),
            "detector `{}` has empty fusion_axes — every record must bind to at least one fusion axis",
            r.display_name
        );
    }
}

#[test]
fn confuser_roles_have_negative_witness_kind() {
    for r in SEED {
        if r.witness_role != WitnessRole::Confuser {
            continue;
        }
        assert_ne!(
            r.negative_witness_kind,
            NegativeWitnessKind::NotANegativeWitness,
            "Confuser record `{}` must carry a concrete NegativeWitnessKind (not NotANegativeWitness)",
            r.display_name
        );
    }
}

#[test]
fn primary_witnesses_cannot_be_negative_only() {
    // T.6 invariant: a Primary witness fires to ADMIT an episode;
    // it cannot simultaneously carry a non-NotANegativeWitness
    // negative kind. The static seed currently has zero violators;
    // the test pins that property.
    for r in SEED {
        if r.witness_role != WitnessRole::Primary {
            continue;
        }
        assert!(
            primary_may_carry_negative_witness_kind(r.negative_witness_kind),
            "Primary record `{}` carries a non-NotANegativeWitness negative_witness_kind ({:?}); primary + negative is incoherent",
            r.display_name,
            r.negative_witness_kind
        );
    }
}

#[test]
fn clean_window_witnesses_cannot_admit_episode_alone() {
    // T.6 declarative rule: CleanWindow witnesses do not admit
    // episodes. The fusion engine (Section S Phase 1+) enforces
    // this at runtime; T.6 pins it at the rule layer.
    assert!(!clean_window_can_admit_episode_alone());
}

#[test]
fn semantic_role_hash_changes_when_role_changes() {
    // Cross-reference to T.3: if witness_role differs, the
    // detector_identity_hash MUST differ. T.6 ensures this still
    // holds after the new fusion-plane layer.
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.witness_role = WitnessRole::Confuser;
    altered.negative_witness_kind = NegativeWitnessKind::SingleWindowSpikeConfuser;
    let h_baseline = compute_identity_hashes(&baseline);
    let h_altered = compute_identity_hashes(&altered);
    assert_ne!(
        h_baseline.semantic_role_hash, h_altered.semantic_role_hash,
        "semantic_role_hash must change when witness_role changes"
    );
    assert_ne!(
        h_baseline.detector_identity_hash, h_altered.detector_identity_hash,
        "detector_identity_hash must change when witness_role changes"
    );
}

#[test]
fn axis_binding_hash_changes_when_axis_changes() {
    // T.6 invariant: changing the AxisBindingSet shifts the
    // semantic_role_hash (since fusion_axes is part of that hash).
    let baseline = SEED[0];
    let mut altered = baseline;
    altered.fusion_axes = AxisBindingSet(AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY);
    let h_baseline = compute_identity_hashes(&baseline);
    let h_altered = compute_identity_hashes(&altered);
    assert_ne!(
        h_baseline.semantic_role_hash, h_altered.semantic_role_hash,
        "semantic_role_hash must change when fusion_axes change"
    );
}

#[test]
fn axis_bindings_are_deterministically_sorted() {
    // The bit_names() table is the canonical ordering. Two builds
    // must see the same order.
    let a = AxisBindingSet::bit_names();
    let b = AxisBindingSet::bit_names();
    assert_eq!(a.len(), b.len());
    for (ax, bx) in a.iter().zip(b.iter()) {
        assert_eq!(ax.0, bx.0);
        assert_eq!(ax.1, bx.1);
    }
    // The bits must be strictly increasing (no overlap, no
    // accidental reuse).
    let mut last_bit = 0u16;
    for (bit, _) in a {
        assert!(*bit > last_bit, "axis bits must be strictly increasing");
        last_bit = *bit;
    }
}

#[test]
fn axes_to_planes_is_deterministic() {
    for r in SEED {
        let p1 = axes_to_planes(r.fusion_axes);
        let p2 = axes_to_planes(r.fusion_axes);
        assert_eq!(
            p1, p2,
            "axes_to_planes must be deterministic for `{}`",
            r.display_name
        );
    }
}

#[test]
fn every_axis_maps_to_at_least_one_plane() {
    // Each of the 9 v1 axes individually must map to at least
    // one plane; otherwise it's an orphan axis.
    for (bit, name) in AxisBindingSet::bit_names() {
        let single_axis = AxisBindingSet(*bit);
        let planes = axes_to_planes(single_axis);
        assert!(
            !planes.is_empty(),
            "axis `{name}` (bit 0x{bit:04x}) maps to no fusion plane"
        );
    }
}

#[test]
fn every_seed_record_has_non_empty_plane_binding() {
    for r in SEED {
        let planes = axes_to_planes(r.fusion_axes);
        assert!(
            !planes.is_empty(),
            "record `{}` produces an empty fusion-plane set; check fusion_axes",
            r.display_name
        );
    }
}

#[test]
fn fusion_planes_iter_in_canonical_order() {
    let all = FusionPlane::all();
    assert_eq!(all.len(), 8);
    assert_eq!(all[0], FusionPlane::ProvenanceAdmissibility);
    assert_eq!(all[7], FusionPlane::TaskUtility);
}

#[test]
fn fusion_plane_wire_names_round_trip() {
    for p in FusionPlane::all() {
        let s = p.as_str();
        let parsed = FusionPlane::from_wire(s);
        assert_eq!(parsed, Some(*p));
    }
    assert!(FusionPlane::from_wire("NotAPlane").is_none());
}

#[test]
fn fusion_plane_set_contains_is_consistent_with_planes_list() {
    let set = FusionPlaneSet(FusionPlaneSet::NUMERIC_STRENGTH | FusionPlaneSet::TEMPORAL_STRUCTURE);
    let planes = set.planes();
    assert_eq!(planes.len(), 2);
    assert!(planes.contains(&FusionPlane::NumericStrength));
    assert!(planes.contains(&FusionPlane::TemporalStructure));
    assert!(set.contains(FusionPlane::NumericStrength));
    assert!(set.contains(FusionPlane::TemporalStructure));
    assert!(!set.contains(FusionPlane::DistributionStructure));
}

#[test]
fn compatibility_rules_include_confuser_suppresses_primary() {
    let v = lookup_verdict(WitnessRole::Confuser, WitnessRole::Primary);
    assert_eq!(v, Some(CompatibilityVerdict::Suppresses));
}

#[test]
fn compatibility_rules_include_clean_window_incompatible_with_primary() {
    let v = lookup_verdict(WitnessRole::CleanWindow, WitnessRole::Primary);
    assert_eq!(v, Some(CompatibilityVerdict::Incompatible));
}

#[test]
fn compatibility_rules_include_primary_compatible_with_corroborating() {
    let v = lookup_verdict(WitnessRole::Primary, WitnessRole::Corroborating);
    assert_eq!(v, Some(CompatibilityVerdict::Compatible));
}

#[test]
fn compatibility_rules_have_no_self_loop_suppressions() {
    // A role suppressing itself would be nonsensical. T.6 pins
    // that no rule has role_a == role_b.
    for rule in COMPATIBILITY_RULES {
        assert_ne!(
            rule.role_a, rule.role_b,
            "compatibility rule self-loop on {:?}",
            rule.role_a
        );
    }
}

#[test]
fn report_contains_witness_role_histogram() {
    let body = render_report(SEED);
    assert!(body.contains("(6) Witness-role histogram"));
    assert!(body.contains("Primary"));
    assert!(body.contains("Confuser"));
}

#[test]
fn report_contains_negative_witness_histogram() {
    let body = render_report(SEED);
    assert!(body.contains("(7) Negative-witness histogram"));
    assert!(body.contains("SingleWindowSpikeConfuser"));
}

#[test]
fn report_contains_fusion_plane_histogram() {
    let body = render_report(SEED);
    assert!(body.contains("(8) Fusion-plane histogram"));
    for plane in FusionPlane::all() {
        assert!(
            body.contains(plane.as_str()),
            "report must list plane `{}` in the histogram",
            plane.as_str()
        );
    }
}

#[test]
fn report_contains_role_axis_matrix() {
    let body = render_report(SEED);
    assert!(body.contains("(9) Witness-role x fusion-plane coverage matrix"));
}

#[test]
fn report_contains_witness_law_coverage_block() {
    let body = render_report(SEED);
    assert!(body.contains("(10) Witness-law coverage"));
    assert!(body.contains("detectors_missing_axis_bindings          : 0"));
    assert!(body.contains("primary_witnesses_with_negative_kind     : 0"));
    assert!(body.contains("confuser_witnesses_without_negative_kind : 0"));
}

#[test]
fn report_contains_primary_and_confuser_lists() {
    let body = render_report(SEED);
    assert!(body.contains("(11) Primary-witness detector list"));
    assert!(body.contains("(12) Confuser detector list"));
}
