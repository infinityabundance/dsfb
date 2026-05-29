//! `IndustrialDataReadinessCourtV1` (Wave-4 historian layer) — grade a plant-historian export **before**
//! analysis, so DSFB tells an operator honestly whether their data is fit to evaluate.
//!
//! The single biggest reason a monitoring evaluation disappoints on real plant data is not the algorithm —
//! it is the data: no known-good baseline window, tags with no engineering units, ragged sampling, large
//! missingness, duplicate timestamps, or no licence to analyse it at all. This court reads a structured
//! profile of a historian export and returns one of three verdicts —
//! `Ready` / `ReadyWithCaveats` / `NotReadyMissingCriticalWitnesses` — with a per-dimension finding list, so
//! the operator gets an honest go/no-go *and* a punch-list before any analysis runs. It is the machine
//! counterpart to `docs/sbir_operator_data_request.md`.
//!
//! The grading is deterministic and conservative: a small set of **critical** requirements (some tags, some
//! rows, a baseline window, licence clearance) gate readiness outright; the rest are **caveats** that
//! downgrade `Ready → ReadyWithCaveats` but never block. Thresholds are explicit and disclosed.
//!
//! Bounded (non-claims): a grade is **advisory data-quality guidance, not a guarantee of analysis success**,
//! not a data-validation certificate, and not a statement about the truthfulness of the supplied profile
//! (DSFB grades what is declared). Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Disclosed grading thresholds (advisory; conservative). Caveat-level limits for the soft dimensions.
const MAX_MISSINGNESS: f64 = 0.10; // > 10 % missing ⇒ caveat
const MIN_UNIT_COVERAGE: f64 = 0.90; // < 90 % of tags carry a unit ⇒ caveat (blocks the unit court)
const MAX_DUPLICATE_TS: f64 = 0.01; // > 1 % duplicate timestamps ⇒ caveat
const MIN_SPAN_HOURS: f64 = 24.0; // < 24 h of data ⇒ caveat (too short to see diurnal structure)
const MIN_CRITICAL_ROWS: usize = 100; // fewer rows ⇒ critically thin
const MIN_CRITICAL_TAGS: usize = 1;

/// A structured profile of a historian export — what an operator (or an importer) can state about their data
/// without sharing it. Every field is declared, not measured by this court.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorianExportProfile {
    pub n_tags: usize,
    pub n_rows: usize,
    /// A span of known-good / normal operation is present (even approximate) — required to fit a baseline.
    pub baseline_window_present: bool,
    /// The data is the operator's to analyse (no licence / export-control blocker).
    pub license_cleared: bool,
    /// Fraction of cells missing, in `[0, 1]`.
    pub missingness_fraction: f64,
    /// Fraction of tags carrying a declared engineering unit, in `[0, 1]`.
    pub unit_coverage_fraction: f64,
    /// Fraction of duplicate timestamps, in `[0, 1]`.
    pub duplicate_timestamp_fraction: f64,
    /// Total time span of the export, in hours.
    pub time_span_hours: f64,
    /// Sampling is (approximately) regular; irregular/ragged sampling is handled but flagged.
    pub sampling_regular: bool,
    /// Controller context (PV/MV/SP, controller mode) is available — unlocks control-loop witnesses.
    pub has_controllers: bool,
    /// Maintenance log available — unlocks maintenance-window overlays / calibration witnesses.
    pub has_maintenance: bool,
    /// Batch / campaign genealogy available — unlocks batch-phase envelopes.
    pub has_batches: bool,
    /// Sparse lab / manual-sample results available — unlocks the manual-sample bridge.
    pub has_lab_samples: bool,
}

impl Default for HistorianExportProfile {
    /// A neutral profile (no data) — every critical requirement unmet, so the default grades `NotReady`.
    fn default() -> Self {
        HistorianExportProfile {
            n_tags: 0,
            n_rows: 0,
            baseline_window_present: false,
            license_cleared: false,
            missingness_fraction: 0.0,
            unit_coverage_fraction: 0.0,
            duplicate_timestamp_fraction: 0.0,
            time_span_hours: 0.0,
            sampling_regular: true,
            has_controllers: false,
            has_maintenance: false,
            has_batches: false,
            has_lab_samples: false,
        }
    }
}

/// The severity of one readiness finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessLevel {
    /// The dimension is satisfied.
    Ok,
    /// A non-blocking degradation: analysis can proceed but with a stated limitation.
    Caveat,
    /// A blocking gap: a critical witness/requirement is missing.
    CriticalMissing,
}

impl ReadinessLevel {
    fn tag(self) -> &'static str {
        match self {
            ReadinessLevel::Ok => "ok",
            ReadinessLevel::Caveat => "caveat",
            ReadinessLevel::CriticalMissing => "critical_missing",
        }
    }
}

/// One graded dimension of the export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessFinding {
    pub dimension: String,
    pub level: String,
    pub detail: String,
}

/// The overall readiness verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessVerdict {
    Ready,
    ReadyWithCaveats,
    NotReadyMissingCriticalWitnesses,
}

impl ReadinessVerdict {
    fn tag(self) -> &'static str {
        match self {
            ReadinessVerdict::Ready => "Ready",
            ReadinessVerdict::ReadyWithCaveats => "ReadyWithCaveats",
            ReadinessVerdict::NotReadyMissingCriticalWitnesses => {
                "NotReadyMissingCriticalWitnesses"
            }
        }
    }
}

/// A hash-sealed data-readiness grade (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndustrialDataReadinessCourtV1 {
    pub findings: Vec<ReadinessFinding>,
    pub n_critical_missing: usize,
    pub n_caveat: usize,
    pub verdict: String,
    pub court_hash: String,
}

impl IndustrialDataReadinessCourtV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"industrial_data_readiness_court_v1");
        for f in &self.findings {
            h.field("dimension", f.dimension.as_bytes());
            h.field("level", f.level.as_bytes());
        }
        h.u64("n_critical_missing", self.n_critical_missing as u64);
        h.u64("n_caveat", self.n_caveat as u64);
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    /// Grade a historian-export profile against the disclosed thresholds and seal the verdict.
    pub fn grade(p: &HistorianExportProfile) -> Self {
        let mut findings = Vec::new();
        let mut add = |dimension: &str, level: ReadinessLevel, detail: String| {
            findings.push(ReadinessFinding {
                dimension: dimension.into(),
                level: level.tag().into(),
                detail,
            });
        };

        // ── Critical requirements (any failure ⇒ NotReady) ──────────────────────────────────────────
        add(
            "tags_present",
            if p.n_tags >= MIN_CRITICAL_TAGS {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::CriticalMissing
            },
            format!("{} tags (need ≥ {MIN_CRITICAL_TAGS})", p.n_tags),
        );
        add(
            "rows_sufficient",
            if p.n_rows >= MIN_CRITICAL_ROWS {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::CriticalMissing
            },
            format!("{} rows (need ≥ {MIN_CRITICAL_ROWS})", p.n_rows),
        );
        add(
            "baseline_window",
            if p.baseline_window_present {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::CriticalMissing
            },
            "a known-good / normal-operation window is required to fit the baseline".into(),
        );
        add(
            "license_cleared",
            if p.license_cleared {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::CriticalMissing
            },
            "the data must be the operator's to analyse (no licence / export-control blocker)"
                .into(),
        );

        // ── Caveat dimensions (degrade Ready → ReadyWithCaveats, never block) ────────────────────────
        add(
            "missingness",
            if p.missingness_fraction <= MAX_MISSINGNESS {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::Caveat
            },
            format!(
                "{:.1}% missing (caveat above {:.0}%)",
                100.0 * p.missingness_fraction,
                100.0 * MAX_MISSINGNESS
            ),
        );
        add(
            "unit_coverage",
            if p.unit_coverage_fraction >= MIN_UNIT_COVERAGE { ReadinessLevel::Ok } else { ReadinessLevel::Caveat },
            format!(
                "{:.0}% of tags carry a unit (caveat below {:.0}%; limits the unit-consistency court + physical balances)",
                100.0 * p.unit_coverage_fraction, 100.0 * MIN_UNIT_COVERAGE
            ),
        );
        add(
            "duplicate_timestamps",
            if p.duplicate_timestamp_fraction <= MAX_DUPLICATE_TS {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::Caveat
            },
            format!(
                "{:.2}% duplicate timestamps (caveat above {:.0}%)",
                100.0 * p.duplicate_timestamp_fraction,
                100.0 * MAX_DUPLICATE_TS
            ),
        );
        add(
            "time_span",
            if p.time_span_hours >= MIN_SPAN_HOURS {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::Caveat
            },
            if p.time_span_hours.is_finite() {
                format!(
                    "{:.1} h of data (caveat below {:.0} h; too short for diurnal structure)",
                    p.time_span_hours, MIN_SPAN_HOURS
                )
            } else {
                format!("span not assessed (caveat; need ≥ {:.0} h of timestamped data for diurnal structure)", MIN_SPAN_HOURS)
            },
        );
        add(
            "sampling_regularity",
            if p.sampling_regular {
                ReadinessLevel::Ok
            } else {
                ReadinessLevel::Caveat
            },
            "irregular/ragged sampling is handled (multi-rate alignment) but flagged".into(),
        );
        // Optional-context dimensions: their absence is a caveat (fewer witness families available).
        for (present, dim, unlocks) in [
            (
                p.has_controllers,
                "controller_context",
                "control-loop interaction map + setpoint separation",
            ),
            (
                p.has_maintenance,
                "maintenance_log",
                "maintenance-window overlays + calibration witnesses",
            ),
            (
                p.has_batches,
                "batch_genealogy",
                "batch-phase envelopes + genealogy recurrence",
            ),
            (
                p.has_lab_samples,
                "lab_samples",
                "manual-sample bridge + soft-sensor witnesses",
            ),
        ] {
            add(
                dim,
                if present {
                    ReadinessLevel::Ok
                } else {
                    ReadinessLevel::Caveat
                },
                if present {
                    format!("available — unlocks {unlocks}")
                } else {
                    format!("absent — {unlocks} unavailable")
                },
            );
        }

        let n_critical_missing = findings
            .iter()
            .filter(|f| f.level == "critical_missing")
            .count();
        let n_caveat = findings.iter().filter(|f| f.level == "caveat").count();
        let verdict = if n_critical_missing > 0 {
            ReadinessVerdict::NotReadyMissingCriticalWitnesses
        } else if n_caveat > 0 {
            ReadinessVerdict::ReadyWithCaveats
        } else {
            ReadinessVerdict::Ready
        };
        let mut court = IndustrialDataReadinessCourtV1 {
            findings,
            n_critical_missing,
            n_caveat,
            verdict: verdict.tag().into(),
            court_hash: String::new(),
        };
        court.court_hash = court.seal();
        court
    }

    /// True iff analysis may proceed (verdict is not `NotReady…`).
    pub fn is_evaluable(&self) -> bool {
        self.verdict != ReadinessVerdict::NotReadyMissingCriticalWitnesses.tag()
    }

    /// Re-derive the tallies + seal from the findings and compare the whole record (catches tampering of a
    /// level, a tally, the verdict, or the hash — all derived from the findings).
    pub fn verify(&self) -> bool {
        let n_critical_missing = self
            .findings
            .iter()
            .filter(|f| f.level == "critical_missing")
            .count();
        let n_caveat = self.findings.iter().filter(|f| f.level == "caveat").count();
        let verdict = if n_critical_missing > 0 {
            ReadinessVerdict::NotReadyMissingCriticalWitnesses
        } else if n_caveat > 0 {
            ReadinessVerdict::ReadyWithCaveats
        } else {
            ReadinessVerdict::Ready
        };
        n_critical_missing == self.n_critical_missing
            && n_caveat == self.n_caveat
            && verdict.tag() == self.verdict
            && self.seal() == self.court_hash
    }

    /// One-line-per-finding render + the verdict, for the CLI and the operator report.
    pub fn render(&self) -> String {
        let mut s = String::new();
        for f in &self.findings {
            let mark = match f.level.as_str() {
                "ok" => "  OK      ",
                "caveat" => "  CAVEAT  ",
                _ => "  MISSING ",
            };
            s.push_str(mark);
            s.push_str(&f.dimension);
            s.push_str(&format!(" — {}\n", f.detail));
        }
        s.push_str(&format!(
            "verdict: {} ({} critical-missing, {} caveats)\ncourt_hash: {}\n",
            self.verdict, self.n_critical_missing, self.n_caveat, self.court_hash
        ));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clean, fully-equipped export (everything present, all soft dimensions good).
    fn ready_profile() -> HistorianExportProfile {
        HistorianExportProfile {
            n_tags: 40,
            n_rows: 5000,
            baseline_window_present: true,
            license_cleared: true,
            missingness_fraction: 0.01,
            unit_coverage_fraction: 1.0,
            duplicate_timestamp_fraction: 0.0,
            time_span_hours: 72.0,
            sampling_regular: true,
            has_controllers: true,
            has_maintenance: true,
            has_batches: true,
            has_lab_samples: true,
        }
    }

    #[test]
    fn fully_equipped_export_grades_ready() {
        let c = IndustrialDataReadinessCourtV1::grade(&ready_profile());
        assert_eq!(c.verdict, "Ready");
        assert_eq!(c.n_critical_missing, 0);
        assert_eq!(c.n_caveat, 0);
        assert!(c.is_evaluable() && c.verify() && c.court_hash.len() == 64);
    }

    #[test]
    fn soft_gaps_downgrade_to_ready_with_caveats() {
        let mut p = ready_profile();
        p.unit_coverage_fraction = 0.5; // half the tags lack units
        p.has_lab_samples = false; // no lab linkage
        p.time_span_hours = 6.0; // short span
        let c = IndustrialDataReadinessCourtV1::grade(&p);
        assert_eq!(c.verdict, "ReadyWithCaveats");
        assert_eq!(c.n_critical_missing, 0);
        assert!(c.n_caveat >= 3);
        assert!(c.is_evaluable() && c.verify());
    }

    #[test]
    fn missing_baseline_or_license_is_not_ready() {
        let mut p = ready_profile();
        p.baseline_window_present = false;
        let c = IndustrialDataReadinessCourtV1::grade(&p);
        assert_eq!(c.verdict, "NotReadyMissingCriticalWitnesses");
        assert!(c.n_critical_missing >= 1);
        assert!(!c.is_evaluable() && c.verify());
        // The default (no data) is also NotReady.
        let d = IndustrialDataReadinessCourtV1::grade(&HistorianExportProfile::default());
        assert_eq!(d.verdict, "NotReadyMissingCriticalWitnesses");
        assert!(!d.is_evaluable());
    }

    #[test]
    fn tampering_a_level_or_verdict_breaks_the_seal() {
        let mut c = IndustrialDataReadinessCourtV1::grade(&ready_profile());
        assert!(c.verify());
        c.verdict = "Ready".into(); // already Ready; force a critical gap and check the verdict no longer matches
        c.findings[2].level = "critical_missing".into(); // forge a gap while leaving verdict "Ready"
        assert!(!c.verify());
    }
}
