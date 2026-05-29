//! `StartupShutdownEnvelopeV1` (Wave-4 historian layer) — dedicated admissibility envelopes for the
//! canonical **transient operating phases** (cold-start, hot-start, shutdown-ramp, purge, inerting), so a
//! steady-state baseline does not false-alarm every time the plant is *legitimately* far from steady state.
//!
//! Real plants spend meaningful time outside steady operation, and the residual statistics of a cold start
//! look nothing like steady running. A steady-state envelope flags the whole startup as anomalous. This
//! object computes a **per-phase** admissibility bound (relaxed, never tighter than the steady reference, the
//! same relax-not-tighten discipline as [`crate::regime_envelope`]) and re-judges each sample against *its
//! own phase's* envelope. Its headline metric is `transient_alarms_reclassified`: the samples that exceed the
//! steady bound (a steady-state monitor would alarm) but sit inside their transient-phase envelope — i.e. the
//! false alarms a phase-aware monitor avoids — versus the samples that exceed even their phase envelope (a
//! genuine excursion *during* that phase, which is still flagged).
//!
//! Bounded (non-claims): a phase envelope is *relaxed for documented transient variability* — being inside it
//! marks the deviation as within the expected envelope for that phase, **not** that the transient is safe or
//! correct, and carries no control/safety authority. Phase labels are a supplied input. Additive + off the
//! replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The canonical operating phases. `Steady` is the reference; the rest are transients with naturally wider
/// residual statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingPhase {
    Steady,
    ColdStart,
    HotStart,
    ShutdownRamp,
    Purge,
    Inerting,
}

impl OperatingPhase {
    /// Stable tag for hashing / rendering.
    pub fn tag(self) -> &'static str {
        match self {
            OperatingPhase::Steady => "steady",
            OperatingPhase::ColdStart => "cold_start",
            OperatingPhase::HotStart => "hot_start",
            OperatingPhase::ShutdownRamp => "shutdown_ramp",
            OperatingPhase::Purge => "purge",
            OperatingPhase::Inerting => "inerting",
        }
    }
    /// True iff this is a transient (non-steady) phase.
    pub fn is_transient(self) -> bool {
        !matches!(self, OperatingPhase::Steady)
    }
}

/// One phase's calibrated envelope and how its samples fell relative to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseEnvelope {
    pub phase: String,
    pub n_samples: usize,
    /// Admissibility bound for this phase: `max(per-phase score quantile, steady_bound)` (relaxed, never tighter).
    pub bound: f64,
    pub n_within: usize,
    pub n_exceed: usize,
}

/// A hash-sealed startup/shutdown phase-envelope record (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartupShutdownEnvelopeV1 {
    /// Upper quantile used to set each phase's bound from its own scores (e.g. 0.99).
    pub quantile: f64,
    /// The global steady-state reference bound; every phase bound is floored at this value.
    pub steady_bound: f64,
    /// Per-phase envelopes, sorted by phase tag for determinism.
    pub phase_envelopes: Vec<PhaseEnvelope>,
    /// Samples exceeding the steady bound but within their (transient) phase envelope — the false alarms a
    /// phase-aware monitor avoids.
    pub transient_alarms_reclassified: usize,
    /// Samples exceeding even their own phase envelope (genuine excursions during that phase; still flagged).
    pub total_exceed: usize,
    pub series_hash: String,
    pub envelope_hash: String,
}

/// Upper-quantile of a finite slice (rank = ceil(q·n), clamped to [1,n]); 0 if empty.
fn quantile(values: &[f64], q: f64) -> f64 {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((q * v.len() as f64).ceil() as usize).clamp(1, v.len());
    v[rank - 1]
}

impl StartupShutdownEnvelopeV1 {
    fn hash_scores(scores: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"startup_shutdown_scores_v1");
        for &s in scores {
            h.f64q("s", s);
        }
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"startup_shutdown_envelope_v1");
        h.f64q("quantile", self.quantile);
        h.f64q("steady_bound", self.steady_bound);
        for e in &self.phase_envelopes {
            h.field("phase", e.phase.as_bytes());
            h.u64("n_samples", e.n_samples as u64);
            h.f64q("bound", e.bound);
            h.u64("n_within", e.n_within as u64);
            h.u64("n_exceed", e.n_exceed as u64);
        }
        h.u64(
            "transient_alarms_reclassified",
            self.transient_alarms_reclassified as u64,
        );
        h.u64("total_exceed", self.total_exceed as u64);
        h.field("series_hash", self.series_hash.as_bytes());
        h.finalize_hex()
    }

    /// Compute per-phase envelopes from the phase labels + score series and re-judge each sample against its
    /// own phase envelope. `steady_bound` is the global steady reference (floor for every phase bound).
    pub fn build(
        phases: &[OperatingPhase],
        scores: &[f64],
        quantile_q: f64,
        steady_bound: f64,
    ) -> Self {
        let n = phases.len().min(scores.len());
        // Distinct phases present, sorted by tag for determinism.
        let mut present: Vec<OperatingPhase> = Vec::new();
        for &p in &phases[..n] {
            if !present.contains(&p) {
                present.push(p);
            }
        }
        present.sort_by_key(|p| p.tag());

        // Per-phase bound = max(per-phase score quantile, steady_bound).
        let mut phase_envelopes = Vec::with_capacity(present.len());
        let bound_of = |target: OperatingPhase| -> f64 {
            let s: Vec<f64> = (0..n)
                .filter(|&k| phases[k] == target)
                .map(|k| scores[k])
                .collect();
            quantile(&s, quantile_q).max(steady_bound)
        };
        let bounds: Vec<(OperatingPhase, f64)> =
            present.iter().map(|&p| (p, bound_of(p))).collect();

        let (mut transient_alarms_reclassified, mut total_exceed) = (0usize, 0usize);
        for (phase, bound) in &bounds {
            let (mut n_samples, mut n_within, mut n_exceed) = (0usize, 0usize, 0usize);
            for k in 0..n {
                if phases[k] != *phase {
                    continue;
                }
                n_samples += 1;
                let s = scores[k];
                if s.is_finite() && s > *bound {
                    n_exceed += 1;
                    total_exceed += 1;
                } else {
                    n_within += 1;
                    // Reclassified: a transient sample a steady monitor would alarm, now within its phase envelope.
                    if phase.is_transient() && s.is_finite() && s > steady_bound {
                        transient_alarms_reclassified += 1;
                    }
                }
            }
            phase_envelopes.push(PhaseEnvelope {
                phase: phase.tag().into(),
                n_samples,
                bound: *bound,
                n_within,
                n_exceed,
            });
        }
        let mut e = StartupShutdownEnvelopeV1 {
            quantile: quantile_q,
            steady_bound,
            phase_envelopes,
            transient_alarms_reclassified,
            total_exceed,
            series_hash: Self::hash_scores(&scores[..n]),
            envelope_hash: String::new(),
        };
        e.envelope_hash = e.seal();
        e
    }

    pub fn verify(&self, phases: &[OperatingPhase], scores: &[f64]) -> bool {
        let n = phases.len().min(scores.len());
        Self::hash_scores(&scores[..n]) == self.series_hash && self.seal() == self.envelope_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use OperatingPhase::*;

    #[test]
    fn cold_start_transient_is_reclassified_not_alarmed() {
        // Steady bound 1.0. During a cold start the score legitimately rides to ~3; under steady rules every
        // such sample alarms, but the cold-start envelope (quantile of its own scores) admits them.
        let phases = vec![Steady, Steady, ColdStart, ColdStart, ColdStart, Steady];
        let scores = vec![0.5, 0.8, 3.0, 3.2, 2.9, 0.7];
        let e = StartupShutdownEnvelopeV1::build(&phases, &scores, 0.99, 1.0);
        // Cold-start bound ≥ ~3.2; its 3 samples are within → 3 reclassified (each > steady 1.0).
        assert_eq!(e.transient_alarms_reclassified, 3);
        assert_eq!(e.total_exceed, 0);
        let cold = e
            .phase_envelopes
            .iter()
            .find(|p| p.phase == "cold_start")
            .unwrap();
        assert!(cold.bound >= 3.2 - 1e-9 && cold.n_exceed == 0 && cold.n_samples == 3);
        assert!(e.envelope_hash.len() == 64 && e.verify(&phases, &scores));
    }

    #[test]
    fn excursion_beyond_the_phase_envelope_is_still_flagged() {
        // A cold start that mostly sits ~3 but spikes to 50 once → that spike exceeds even the cold-start
        // envelope and is flagged (a genuine excursion during startup, not just transient variability).
        let phases = vec![ColdStart, ColdStart, ColdStart, ColdStart, ColdStart];
        let scores = vec![3.0, 3.1, 3.0, 50.0, 3.2];
        let e = StartupShutdownEnvelopeV1::build(&phases, &scores, 0.80, 1.0);
        assert!(
            e.total_exceed >= 1,
            "the 50 spike must exceed the cold-start envelope"
        );
        assert!(e.verify(&phases, &scores));
    }

    #[test]
    fn phase_bounds_are_never_tighter_than_steady() {
        // A purge phase that is actually very quiet still gets a bound floored at the steady reference.
        let phases = vec![Purge, Purge, Purge];
        let scores = vec![0.01, 0.02, 0.01];
        let e = StartupShutdownEnvelopeV1::build(&phases, &scores, 0.99, 1.0);
        let purge = e
            .phase_envelopes
            .iter()
            .find(|p| p.phase == "purge")
            .unwrap();
        assert!(
            (purge.bound - 1.0).abs() < 1e-12,
            "bound floored at steady_bound, got {}",
            purge.bound
        );
    }

    #[test]
    fn tampering_a_phase_bound_breaks_the_seal() {
        let phases = vec![Steady, ColdStart, ColdStart];
        let scores = vec![0.5, 3.0, 3.1];
        let mut e = StartupShutdownEnvelopeV1::build(&phases, &scores, 0.99, 1.0);
        assert!(e.verify(&phases, &scores));
        e.phase_envelopes[0].bound = 999.0; // forge a looser bound
        assert!(!e.verify(&phases, &scores));
    }
}
