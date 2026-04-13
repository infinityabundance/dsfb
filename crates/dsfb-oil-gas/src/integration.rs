/// DSFB Oil & Gas — Integration Contract Module
///
/// This module formalises and enforces the non-intrusive integration contract:
/// - DSFB is read-only: it accepts sensor data but never writes upstream.
/// - No feedback path exists between DSFB output and any control register.
/// - The framework is removable with zero upstream impact.
/// - Deterministic replay is guaranteed for any logged residual sequence.
///
/// Tests in this module verify that DSFB cannot mutate upstream state.

// ─────────────────────────────────────────────────────────────────────────────
// Integration contract marker types
// ─────────────────────────────────────────────────────────────────────────────

/// Wrapper that enforces read-only access to an upstream data source.
///
/// `T` is any type representing upstream state (e.g., a SCADA tag snapshot).
/// `ReadOnlySlice<T>` can be shared with the DSFB engine but provides no
/// mutable access.  The only operation is `as_slice()`.
#[cfg(feature = "alloc")]
pub struct ReadOnlySlice<T> {
    inner: Vec<T>,
}

#[cfg(feature = "alloc")]
impl<T> ReadOnlySlice<T> {
    /// Wrap upstream data.  The wrapping operation moves the data, ensuring
    /// the original binding cannot be used to mutate after wrapping.
    pub fn wrap(data: Vec<T>) -> Self {
        ReadOnlySlice { inner: data }
    }

    /// Immutable view.
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    pub fn len(&self) -> usize { self.inner.len() }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
}

// ─────────────────────────────────────────────────────────────────────────────
// No write-back guarantee: demonstrated by API design
// ─────────────────────────────────────────────────────────────────────────────

/// Consuming-sink adapter: processes a ReadOnlySlice through an arbitrary
/// consumer function without exposing a mutable reference to the source data.
///
/// This is a type-level enforcement of the no-write-back property:
/// `f` receives only a shared reference; it cannot write back to the source.
#[cfg(feature = "alloc")]
pub fn process_read_only<T, R, F>(source: &ReadOnlySlice<T>, f: F) -> R
where
    F: FnOnce(&[T]) -> R,
{
    f(source.as_slice())
}

// ─────────────────────────────────────────────────────────────────────────────
// Deterministic replay utility
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use crate::types::{AdmissibilityEnvelope, GrammarState, ResidualSample};
#[cfg(feature = "alloc")]
use crate::grammar::{GrammarClassifier, DeterministicDsfb};

/// Replay a recorded residual sequence through DSFB and return the state log.
///
/// Given identical inputs and identical engine parameters, the output is
/// guaranteed to be identical to the original run.  This supports forensic
/// post-incident analysis.
#[cfg(feature = "alloc")]
pub fn deterministic_replay(
    samples: &[ResidualSample],
    envelope: AdmissibilityEnvelope,
    drift_window: usize,
) -> Vec<GrammarState> {
    let mut engine = DeterministicDsfb::with_window(
        envelope,
        GrammarClassifier::new(),
        drift_window,
        "replay",
    );
    samples.iter()
        .map(|s| engine.ingest_sample(s).state)
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Removability: zero-impact architecture comment
// ─────────────────────────────────────────────────────────────────────────────

/// Documentation-only marker.
///
/// DSFB integration is removable with zero upstream impact.  Removing the
/// DSFB process terminates the annotation output stream only.  No upstream
/// SCADA tag, alarm limit, historian archive, control variable, or setpoint
/// is modified by DSFB; therefore, removing DSFB restores the pre-deployment
/// operational state identically.
///
/// This is enforced at the API level: the DSFB engine only receives data
/// through `ReadOnlySlice` or equivalent read-only borrows.
pub struct NonIntrusiveGuarantee;

impl NonIntrusiveGuarantee {
    pub const DESCRIPTION: &'static str =
        "DSFB is a read-only observer. It writes to no upstream register, \
         setpoint, alarm limit, historian tag, or control variable. \
         Its removal restores the pre-deployment operational baseline exactly.";
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn read_only_slice_has_no_mut_access() {
        let data = vec![1.0f64, 2.0, 3.0];
        let ro = ReadOnlySlice::wrap(data);
        // We can read
        assert_eq!(ro.as_slice().len(), 3);
        // We cannot call ro.inner.push(...) because inner is private.
        // ReadOnlySlice<T> exposes no &mut T or Vec<T>.
    }

    #[test]
    fn process_read_only_cannot_modify_source() {
        let source = ReadOnlySlice::wrap(vec![10.0f64, 20.0, 30.0]);
        let sum = process_read_only(&source, |sl| sl.iter().sum::<f64>());
        assert!((sum - 60.0).abs() < 1e-12);
        // source is unchanged (shared reference only was passed)
        assert_eq!(source.len(), 3);
    }

    #[test]
    fn replay_is_deterministic() {
        let samples: Vec<ResidualSample> = (0..20)
            .map(|i| ResidualSample::new(i as f64, (i as f64).sin() * 3.0, 0.0, "replay_test"))
            .collect();
        let env = AdmissibilityEnvelope::default_pipeline();
        let run1 = deterministic_replay(&samples, env, 5);
        let run2 = deterministic_replay(&samples, env, 5);
        assert_eq!(run1, run2, "replay results must be identical");
    }
}
