//! Sensor-health context witnesses (Wave-4 historian layer): `SensorTrustDegradationLedgerV1` and
//! `CalibrationEventWitnessV1`.
//!
//! A residual is only as trustworthy as the sensor behind it. A drifting, noisy, or stuck transmitter
//! manufactures "anomalies" that are really instrument problems; and a recalibration deliberately steps a
//! signal, which a naive monitor reads as a fault. These two sealed records carry sensor-health context into
//! the Court Record:
//!
//!   * [`SensorTrustDegradationLedgerV1`] — a per-sensor trust score (0..1) derived from declared degradation
//!     indicators (drift, excess noise, flatlining, out-of-range frequency, missingness), so a weak signal is
//!     visibly down-weighted rather than silently trusted.
//!   * [`CalibrationEventWitnessV1`] — a recalibration event (time, pre/post offset + span), so residuals
//!     across the event are read with the calibration step in mind rather than as a process fault.
//!
//! Bounded (non-claims): a trust score is **advisory sensor-health guidance, not a calibration verdict or a
//! declaration that the sensor is faulty**; a calibration witness **records the declared event, it is not a
//! metrology certificate**. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── SensorTrustDegradationLedgerV1 ──────────────────────────────────────────────────────────────────

/// Declared degradation indicators for one sensor, each a unit-interval severity in `[0, 1]` (0 = healthy,
/// 1 = fully degraded on that axis). Values outside `[0,1]` are clamped on build.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DegradationIndicators {
    pub drift: f64,
    pub excess_noise: f64,
    pub flatlining: f64,
    pub out_of_range_frequency: f64,
    pub missingness: f64,
}

impl DegradationIndicators {
    fn clamped(self) -> Self {
        let c = |x: f64| {
            if x.is_finite() {
                x.clamp(0.0, 1.0)
            } else {
                1.0
            }
        };
        DegradationIndicators {
            drift: c(self.drift),
            excess_noise: c(self.excess_noise),
            flatlining: c(self.flatlining),
            out_of_range_frequency: c(self.out_of_range_frequency),
            missingness: c(self.missingness),
        }
    }
    /// Worst single indicator (the conservative summary: trust is capped by the worst axis).
    fn worst(self) -> f64 {
        [
            self.drift,
            self.excess_noise,
            self.flatlining,
            self.out_of_range_frequency,
            self.missingness,
        ]
        .into_iter()
        .fold(0.0f64, f64::max)
    }
}

/// One sensor's trust entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorTrustEntry {
    pub sensor_id: String,
    pub indicators: DegradationIndicators,
    /// `1 − worst(indicators)` — trust is conservatively capped by the worst degradation axis.
    pub trust_score: f64,
}

/// A hash-sealed per-sensor trust ledger (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorTrustDegradationLedgerV1 {
    pub entries: Vec<SensorTrustEntry>,
    /// Trust at/below which a sensor is flagged low-trust (its residuals should be weighed cautiously).
    pub low_trust_threshold: f64,
    pub n_low_trust: usize,
    pub non_claim: String,
    pub ledger_hash: String,
}

impl SensorTrustDegradationLedgerV1 {
    const NON_CLAIM: &'static str =
        "advisory sensor-health guidance; a low trust score down-weights a signal, it is NOT a calibration verdict or a declaration that the sensor is faulty";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"sensor_trust_degradation_ledger_v1");
        for e in &self.entries {
            h.field("sensor_id", e.sensor_id.as_bytes());
            h.f64q("drift", e.indicators.drift);
            h.f64q("excess_noise", e.indicators.excess_noise);
            h.f64q("flatlining", e.indicators.flatlining);
            h.f64q(
                "out_of_range_frequency",
                e.indicators.out_of_range_frequency,
            );
            h.f64q("missingness", e.indicators.missingness);
            h.f64q("trust_score", e.trust_score);
        }
        h.f64q("low_trust_threshold", self.low_trust_threshold);
        h.u64("n_low_trust", self.n_low_trust as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Build the ledger from `(sensor_id, indicators)` pairs. `trust_score = 1 − worst(indicators)`.
    pub fn build(raw: &[(String, DegradationIndicators)], low_trust_threshold: f64) -> Self {
        let entries: Vec<SensorTrustEntry> = raw
            .iter()
            .map(|(id, ind)| {
                let indicators = ind.clamped();
                SensorTrustEntry {
                    sensor_id: id.clone(),
                    indicators,
                    trust_score: 1.0 - indicators.worst(),
                }
            })
            .collect();
        let n_low_trust = entries
            .iter()
            .filter(|e| e.trust_score <= low_trust_threshold)
            .count();
        let mut l = SensorTrustDegradationLedgerV1 {
            entries,
            low_trust_threshold,
            n_low_trust,
            non_claim: Self::NON_CLAIM.to_string(),
            ledger_hash: String::new(),
        };
        l.ledger_hash = l.seal();
        l
    }

    /// The trust score for a sensor (None if not in the ledger).
    pub fn trust_of(&self, sensor_id: &str) -> Option<f64> {
        self.entries
            .iter()
            .find(|e| e.sensor_id == sensor_id)
            .map(|e| e.trust_score)
    }

    pub fn verify(&self) -> bool {
        let recomputed_low = self
            .entries
            .iter()
            .filter(|e| e.trust_score <= self.low_trust_threshold)
            .count();
        let trust_ok = self
            .entries
            .iter()
            .all(|e| (e.trust_score - (1.0 - e.indicators.worst())).abs() <= 1e-12);
        trust_ok
            && recomputed_low == self.n_low_trust
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.ledger_hash
    }
}

// ── CalibrationEventWitnessV1 ───────────────────────────────────────────────────────────────────────

/// A hash-sealed calibration-event witness (schema v1): a recalibration of one sensor at a sample index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationEventWitnessV1 {
    pub sensor_id: String,
    /// Sample index at which the recalibration took effect.
    pub event_index: usize,
    pub pre_offset: f64,
    pub post_offset: f64,
    pub pre_span: f64,
    pub post_span: f64,
    /// `post_offset − pre_offset` — the deliberate signal step a monitor should attribute to calibration.
    pub offset_step: f64,
    pub non_claim: String,
    pub witness_hash: String,
}

impl CalibrationEventWitnessV1 {
    const NON_CLAIM: &'static str =
        "records the declared calibration event; NOT a metrology certificate";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"calibration_event_witness_v1");
        h.field("sensor_id", self.sensor_id.as_bytes());
        h.u64("event_index", self.event_index as u64);
        h.f64q("pre_offset", self.pre_offset);
        h.f64q("post_offset", self.post_offset);
        h.f64q("pre_span", self.pre_span);
        h.f64q("post_span", self.post_span);
        h.f64q("offset_step", self.offset_step);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    pub fn build(
        sensor_id: impl Into<String>,
        event_index: usize,
        pre_offset: f64,
        post_offset: f64,
        pre_span: f64,
        post_span: f64,
    ) -> Self {
        let mut w = CalibrationEventWitnessV1 {
            sensor_id: sensor_id.into(),
            event_index,
            pre_offset,
            post_offset,
            pre_span,
            post_span,
            offset_step: post_offset - pre_offset,
            non_claim: Self::NON_CLAIM.to_string(),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// True iff a sample index sits within `window` samples after the calibration (where a step is expected).
    pub fn explains_step_at(&self, index: usize, window: usize) -> bool {
        index >= self.event_index && index <= self.event_index + window
    }

    pub fn verify(&self) -> bool {
        (self.offset_step - (self.post_offset - self.pre_offset)).abs() <= 1e-12
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ind(drift: f64, noise: f64) -> DegradationIndicators {
        DegradationIndicators {
            drift,
            excess_noise: noise,
            flatlining: 0.0,
            out_of_range_frequency: 0.0,
            missingness: 0.0,
        }
    }

    #[test]
    fn trust_is_capped_by_worst_indicator_and_flags_low_trust() {
        let raw = vec![
            ("TI-101".to_string(), ind(0.05, 0.10)), // worst 0.10 → trust 0.90
            ("FT-204".to_string(), ind(0.70, 0.20)), // worst 0.70 → trust 0.30 (low)
        ];
        let l = SensorTrustDegradationLedgerV1::build(&raw, 0.5);
        assert!((l.trust_of("TI-101").unwrap() - 0.90).abs() < 1e-12);
        assert!((l.trust_of("FT-204").unwrap() - 0.30).abs() < 1e-12);
        assert_eq!(l.n_low_trust, 1);
        assert!(l.non_claim.contains("NOT a calibration verdict"));
        assert!(l.ledger_hash.len() == 64 && l.verify());
        let mut t = l.clone();
        t.entries[0].trust_score = 0.0; // forge a trust score inconsistent with its indicators
        assert!(!t.verify());
    }

    #[test]
    fn out_of_range_indicator_is_clamped() {
        let raw = vec![("X".to_string(), ind(1.5, -0.2))]; // clamp → drift 1.0, noise 0.0
        let l = SensorTrustDegradationLedgerV1::build(&raw, 0.5);
        assert!((l.trust_of("X").unwrap() - 0.0).abs() < 1e-12); // worst 1.0 → trust 0
        assert!(l.verify());
    }

    #[test]
    fn calibration_event_records_step_and_window() {
        let w = CalibrationEventWitnessV1::build("TI-101", 500, 0.2, 0.5, 1.0, 1.02);
        assert!((w.offset_step - 0.3).abs() < 1e-12);
        assert!(
            w.explains_step_at(503, 5)
                && !w.explains_step_at(600, 5)
                && !w.explains_step_at(499, 5)
        );
        assert!(w.non_claim.contains("NOT a metrology certificate"));
        assert!(w.verify());
        let mut t = w.clone();
        t.offset_step = 0.0; // forge away the step
        assert!(!t.verify());
    }
}
