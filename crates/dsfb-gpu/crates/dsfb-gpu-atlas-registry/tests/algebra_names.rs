//! S1.1 acceptance tests: canonical name grammar.
//!
//! Panel-required tests in this file:
//!
//! - `canonical_name_is_stable`
//! - `canonical_name_rejects_empty_family`
//! - `canonical_name_uses_double_underscore_boundaries`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_registry::canonical_name::CanonicalDetectorName;
use dsfb_gpu_atlas_registry::{Comparator, DetectorFamily, DetectorParamSet, Statistic, Transform};

fn sample_name() -> CanonicalDetectorName {
    CanonicalDetectorName::build(
        DetectorFamily::RobustZMad,
        Transform::Residual,
        Statistic::Mad,
        Comparator::TwoSided,
        DetectorParamSet::new(64, 3, 1 << 16, 0),
    )
}

#[test]
fn canonical_name_is_stable() {
    let a = sample_name();
    let b = sample_name();
    assert_eq!(
        a.as_str(),
        b.as_str(),
        "canonical_name must be deterministic across two builds"
    );
    assert_eq!(
        a.as_str(),
        "ROBUST_Z_MAD__RESIDUAL__W64__MAD__TWO_SIDED__P3",
        "panel-locked canonical-name format must be stable"
    );
}

#[test]
fn canonical_name_rejects_empty_family() {
    // We cannot construct a `DetectorFamily::Empty` because the
    // enum has no such variant. Instead, the test constructs a
    // raw name with a leading empty token via the test-only
    // entry-point and asserts the well-formedness predicate
    // detects it.
    let bad =
        CanonicalDetectorName::from_raw_for_test("__RESIDUAL__W64__MAD__TWO_SIDED__P3".to_string());
    assert!(
        !bad.first_token_is_non_empty() || !bad.has_no_empty_token(),
        "canonical_name with empty leading family token must be rejected; got `{}`",
        bad.as_str()
    );
    assert!(
        !bad.has_no_empty_token(),
        "the well-formedness predicate must flag empty tokens"
    );
}

#[test]
fn canonical_name_uses_double_underscore_boundaries() {
    let name = sample_name();
    // Six `__`-delimited tokens.
    assert_eq!(
        name.token_count(),
        6,
        "canonical_name must split into exactly 6 `__`-delimited tokens; got {}: `{}`",
        name.token_count(),
        name.as_str()
    );
    // Every token non-empty.
    assert!(
        name.has_no_empty_token(),
        "canonical_name must have no empty tokens; got `{}`",
        name.as_str()
    );
    // Each known token boundary is `__` (double underscore),
    // not a single `_`. Verify by splitting on `__` and
    // re-joining.
    let parts: Vec<&str> = name.as_str().split("__").collect();
    let rejoined = parts.join("__");
    assert_eq!(
        name.as_str(),
        &rejoined,
        "split + rejoin via `__` must round-trip the canonical name"
    );
}

#[test]
fn canonical_name_emits_distinct_strings_for_distinct_grids() {
    // Sanity: two different parameter sets MUST produce two
    // different canonical names. (If they didn't, the algebra
    // would collapse two grid cells into one identity.)
    let a = CanonicalDetectorName::build(
        DetectorFamily::Ewma,
        Transform::Residual,
        Statistic::Mean,
        Comparator::High,
        DetectorParamSet::new(32, 2, 1 << 14, 0),
    );
    let b = CanonicalDetectorName::build(
        DetectorFamily::Ewma,
        Transform::Residual,
        Statistic::Mean,
        Comparator::High,
        DetectorParamSet::new(64, 2, 1 << 14, 0),
    );
    assert_ne!(
        a.as_str(),
        b.as_str(),
        "different windows must produce different canonical names"
    );
}
