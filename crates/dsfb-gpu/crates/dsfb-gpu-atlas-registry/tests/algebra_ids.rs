//! S1.1 acceptance tests: id system + family ordering.
//!
//! Panel-required tests in this file:
//!
//! - `detector_family_order_is_stable`
//! - `cost_class_order_is_stable`
//! - `numeric_mode_defaults_to_q16_16_for_audit`
//! - `implementation_kind_is_not_gpu_claim_by_default`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_registry::{CostClass, DetectorFamily, ImplementationKind, NumericMode};

#[test]
fn detector_family_order_is_stable() {
    let all = DetectorFamily::all();
    // The panel-recommended seed has 43 families.
    assert_eq!(
        all.len(),
        43,
        "DetectorFamily::all() must return exactly 43 panel-seed families; got {}",
        all.len()
    );
    // First and last are pinned by the panel order.
    assert_eq!(all[0], DetectorFamily::Shewhart);
    assert_eq!(all[all.len() - 1], DetectorFamily::EvmAnomaly);
    // Family ids derive from position; the position of LatencyRamp
    // (a known dsfb-gpu-debug bank surface entry) is pinned.
    let latency_ramp_id = DetectorFamily::LatencyRamp.family_id();
    assert_eq!(
        latency_ramp_id.0, 33,
        "LatencyRamp must sit at family_id 33 in the canonical order; got {}",
        latency_ramp_id.0
    );
    // Two calls must return the same slice — `all()` is a const
    // slice so this is trivially deterministic, but the test
    // pins the contract.
    let a = DetectorFamily::all();
    let b = DetectorFamily::all();
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x, y);
    }
}

#[test]
fn cost_class_order_is_stable() {
    let all = CostClass::all();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0], CostClass::Light);
    assert_eq!(all[1], CostClass::Medium);
    assert_eq!(all[2], CostClass::Heavy);
    // Ord on the enum follows the declaration order.
    assert!(CostClass::Light < CostClass::Medium);
    assert!(CostClass::Medium < CostClass::Heavy);
}

#[test]
fn numeric_mode_defaults_to_q16_16_for_audit() {
    assert_eq!(
        NumericMode::AUDIT_DEFAULT,
        NumericMode::Q16_16,
        "Atlas audit-mode default numeric must be Q16_16 (matches v0 dsfb-gpu-debug baseline)"
    );
    assert_eq!(NumericMode::AUDIT_DEFAULT.canonical_wire_name(), "Q16_16");
}

#[test]
fn implementation_kind_is_not_gpu_claim_by_default() {
    // The default implementation kind for a newly-declared spec
    // is ScalarCpu — explicitly NOT a GPU claim. An upgrade to a
    // GPU surface requires explicit attestation via the L-band
    // ladder on the corpus side.
    let default = ImplementationKind::DEFAULT;
    assert_eq!(default, ImplementationKind::ScalarCpu);
    assert!(
        !default.is_gpu_claim(),
        "default ImplementationKind must NOT be a GPU claim"
    );
    // The other kinds ARE GPU claims.
    assert!(ImplementationKind::CellParallel.is_gpu_claim());
    assert!(ImplementationKind::SegmentScan.is_gpu_claim());
    assert!(ImplementationKind::FamilyKernel.is_gpu_claim());
}

#[test]
fn family_canonical_wire_names_are_uppercase_snake_case() {
    for family in DetectorFamily::all() {
        let name = family.canonical_wire_name();
        assert!(!name.is_empty(), "family {family:?} has empty wire name");
        assert!(
            name.bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'),
            "family {family:?} wire name `{name}` contains non-uppercase-snake-case bytes"
        );
        assert!(
            !name.starts_with('_') && !name.ends_with('_'),
            "family {family:?} wire name `{name}` must not start/end with underscore"
        );
    }
}
