//! Canonical detector-name generator and parser.
//!
//! Panel-locked grammar:
//!
//! ```text
//!   {FAMILY}__{TRANSFORM}__W{WINDOW}__{STATISTIC}__{COMPARATOR}__P{PERSISTENCE}
//! ```
//!
//! Examples:
//!
//! - `ROBUST_Z_MAD__RESIDUAL__W64__MAD__TWO_SIDED__P3`
//! - `EWMA__RESIDUAL__W32__MEAN__HIGH__P2`
//! - `CUSUM__DRIFT__W64__SUM__HIGH__P4`
//!
//! Tokens are uppercase snake-case. Token boundary is the exact
//! string `__` (double underscore). Window and persistence carry
//! their numeric suffix; family and other tokens use their
//! variant's [`canonical_wire_name`].
//!
//! [`canonical_wire_name`]: crate::DetectorFamily::canonical_wire_name

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::{Comparator, DetectorFamily, DetectorParamSet, Statistic, Transform};

/// Canonical wire name for one detector. Two calls with the
/// same inputs produce the same string. Wraps an owned
/// `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalDetectorName(String);

impl CanonicalDetectorName {
    /// Build a canonical name from the algebra coordinates.
    /// **Does not validate**: callers should run the spec
    /// through the verifier in production.
    #[must_use]
    pub fn build(
        family: DetectorFamily,
        transform: Transform,
        statistic: Statistic,
        comparator: Comparator,
        params: DetectorParamSet,
    ) -> Self {
        Self(format!(
            "{family}__{transform}__W{window}__{statistic}__{comparator}__P{persistence}",
            family = family.canonical_wire_name(),
            transform = transform.canonical_wire_name(),
            window = params.window_cells,
            statistic = statistic.canonical_wire_name(),
            comparator = comparator.canonical_wire_name(),
            persistence = params.persistence_windows,
        ))
    }

    /// Inspect the rendered string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Token-count check: a well-formed canonical name has
    /// exactly six `__`-delimited tokens. Used by the verifier
    /// and by the `canonical_name_uses_double_underscore_boundaries`
    /// test.
    #[must_use]
    pub fn token_count(&self) -> usize {
        // Counting via `split("__")` — the canonical separator.
        self.0.split("__").count()
    }

    /// True if every token after splitting on `__` is non-empty
    /// (no `__FOO__` empty leading token, no `FOO____BAR`
    /// double-double, etc.). The verifier requires this.
    #[must_use]
    pub fn has_no_empty_token(&self) -> bool {
        !self.0.split("__").any(str::is_empty)
    }

    /// Smoke-check: leading family token MUST be non-empty.
    /// Used by `canonical_name_rejects_empty_family`.
    #[must_use]
    pub fn first_token_is_non_empty(&self) -> bool {
        self.0.split("__").next().is_some_and(|tok| !tok.is_empty())
    }

    /// Construct from a raw string for diagnostic / test paths.
    /// Production code should use [`Self::build`].
    #[must_use]
    pub fn from_raw_for_test(raw: String) -> Self {
        Self(raw)
    }
}
