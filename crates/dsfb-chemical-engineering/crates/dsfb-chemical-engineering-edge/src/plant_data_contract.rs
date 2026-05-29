//! `PlantDataContractV1` + `HistorianImportReceiptV1` (Wave-6 confidential-evaluation chain) — the documented
//! **industrial input contract** and the **per-file import receipt** that open the confidential-evaluation path.
//!
//! Before any analysis an operator needs to know *exactly* what to supply, and DSFB needs to record *exactly*
//! what arrived — both without the raw data leaving the operator's control. These two sealed objects make that
//! concrete:
//!
//!   * [`PlantDataContractV1`] — the machine form of `docs/sbir_operator_data_request.md`: a fixed list of
//!     contract items (tags / units / controllers / setpoints / alarms / maintenance / batches / lab samples /
//!     topology / data dictionary), each `required` or optional and naming what it `enables`. Checking a
//!     supplied file manifest against the contract yields `Satisfied` / `MissingRequired` plus the punch-list.
//!   * [`HistorianImportReceiptV1`] — a per-file receipt of *what was ingested*: a content hash, row/tag
//!     counts, time range, sampling + missingness profile, unit coverage, and duplicate-timestamp / clock-drift
//!     warnings, with an `Accepted` / `AcceptedWithWarnings` / `Rejected` verdict.
//!
//! Bounded (non-claims): the contract states what is *needed*, not that supplied data is *correct*; the import
//! receipt records *declared/derived ingestion facts*, it is not a data-validation certificate and does not
//! vouch for the values. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── PlantDataContractV1 ─────────────────────────────────────────────────────────────────────────────

/// One item of the input contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractItem {
    /// The file/artifact key the operator supplies (e.g. `"tags.csv"`, `"units.csv"`, `"topology.json"`).
    pub key: String,
    pub required: bool,
    /// What supplying this item unlocks.
    pub enables: String,
    /// True iff the operator's manifest provided this key.
    pub provided: bool,
}

/// A hash-sealed plant-data input-contract verdict (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlantDataContractV1 {
    pub items: Vec<ContractItem>,
    pub n_required_missing: usize,
    pub n_optional_missing: usize,
    /// `"Satisfied"` iff every required item was provided, else `"MissingRequired"`.
    pub verdict: String,
    pub contract_hash: String,
}

impl PlantDataContractV1 {
    /// The fixed contract definition: `(key, required, enables)`. The minimum (M0–M2) inputs are required;
    /// everything else is optional and unlocks more witness families.
    fn definition() -> &'static [(&'static str, bool, &'static str)] {
        &[
            (
                "tags.csv",
                true,
                "the core long-format time-series (timestamp, tag, value)",
            ),
            (
                "baseline_window",
                true,
                "a known-good span to fit the baseline",
            ),
            (
                "units.csv",
                false,
                "unit-consistency court + physical balances",
            ),
            (
                "controllers.csv",
                false,
                "control-loop context; setpoint-vs-process separation; mode guarding",
            ),
            (
                "topology.json",
                false,
                "process-topology + fault-propagation candidates",
            ),
            (
                "alarms.csv",
                false,
                "alarm-flood compression vs the existing alarm system",
            ),
            (
                "maintenance.csv",
                false,
                "maintenance-window overlays; calibration-event witnesses",
            ),
            (
                "batches.csv",
                false,
                "batch-phase envelopes; genealogy recurrence",
            ),
            (
                "lab_samples.csv",
                false,
                "manual-sample bridge; soft-sensor witnesses",
            ),
            (
                "data_dictionary.toml",
                false,
                "observability map (which fault classes the instrumentation can see)",
            ),
        ]
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"plant_data_contract_v1");
        for it in &self.items {
            h.field("key", it.key.as_bytes());
            h.u64("required", it.required as u64);
            h.u64("provided", it.provided as u64);
        }
        h.u64("n_required_missing", self.n_required_missing as u64);
        h.u64("n_optional_missing", self.n_optional_missing as u64);
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    /// Check a supplied manifest (the keys the operator provided) against the fixed contract.
    pub fn check(provided_keys: &[&str]) -> Self {
        let items: Vec<ContractItem> = Self::definition()
            .iter()
            .map(|&(key, required, enables)| ContractItem {
                key: key.into(),
                required,
                enables: enables.into(),
                provided: provided_keys.contains(&key),
            })
            .collect();
        let n_required_missing = items.iter().filter(|i| i.required && !i.provided).count();
        let n_optional_missing = items.iter().filter(|i| !i.required && !i.provided).count();
        let verdict = if n_required_missing == 0 {
            "Satisfied"
        } else {
            "MissingRequired"
        };
        let mut c = PlantDataContractV1 {
            items,
            n_required_missing,
            n_optional_missing,
            verdict: verdict.into(),
            contract_hash: String::new(),
        };
        c.contract_hash = c.seal();
        c
    }

    pub fn satisfied(&self) -> bool {
        self.n_required_missing == 0
    }

    pub fn verify(&self) -> bool {
        let n_required_missing = self
            .items
            .iter()
            .filter(|i| i.required && !i.provided)
            .count();
        let n_optional_missing = self
            .items
            .iter()
            .filter(|i| !i.required && !i.provided)
            .count();
        let verdict = if n_required_missing == 0 {
            "Satisfied"
        } else {
            "MissingRequired"
        };
        n_required_missing == self.n_required_missing
            && n_optional_missing == self.n_optional_missing
            && verdict == self.verdict
            && self.seal() == self.contract_hash
    }
}

// ── HistorianImportReceiptV1 ────────────────────────────────────────────────────────────────────────

/// Per-file import verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportVerdict {
    Accepted,
    AcceptedWithWarnings,
    Rejected,
}

impl ImportVerdict {
    fn tag(self) -> &'static str {
        match self {
            ImportVerdict::Accepted => "Accepted",
            ImportVerdict::AcceptedWithWarnings => "AcceptedWithWarnings",
            ImportVerdict::Rejected => "Rejected",
        }
    }
}

/// A hash-sealed per-file historian-import receipt (schema v1): what was ingested + the warnings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistorianImportReceiptV1 {
    pub file_name: String,
    /// SHA-256 of the file's content (so the receipt pins the exact bytes ingested).
    pub file_hash: String,
    pub n_rows: usize,
    pub n_tags: usize,
    /// Time range as `[t_start, t_end]` in the file's own time unit (0,0 if no timestamps).
    pub t_start: f64,
    pub t_end: f64,
    pub missingness_fraction: f64,
    pub unit_coverage_fraction: f64,
    pub duplicate_timestamp_fraction: f64,
    /// Estimated clock drift (e.g. seconds vs a reference clock); 0 if not assessed.
    pub clock_drift: f64,
    /// Human-readable warning lines (empty ⇒ none).
    pub warnings: Vec<String>,
    pub verdict: String,
    pub receipt_hash: String,
}

impl HistorianImportReceiptV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"historian_import_receipt_v1");
        h.field("file_name", self.file_name.as_bytes());
        h.field("file_hash", self.file_hash.as_bytes());
        h.u64("n_rows", self.n_rows as u64);
        h.u64("n_tags", self.n_tags as u64);
        h.f64q("t_start", self.t_start);
        h.f64q("t_end", self.t_end);
        h.f64q("missingness_fraction", self.missingness_fraction);
        h.f64q("unit_coverage_fraction", self.unit_coverage_fraction);
        h.f64q(
            "duplicate_timestamp_fraction",
            self.duplicate_timestamp_fraction,
        );
        h.f64q("clock_drift", self.clock_drift);
        for w in &self.warnings {
            h.field("warning", w.as_bytes());
        }
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    /// Build the receipt from the ingested file's facts. Warnings + verdict are derived: a row/tag-empty file
    /// is `Rejected`; otherwise high missingness (>50%), any duplicate timestamps, or non-zero clock drift
    /// raise warnings → `AcceptedWithWarnings`; a clean file is `Accepted`.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        file_name: impl Into<String>,
        file_hash: impl Into<String>,
        n_rows: usize,
        n_tags: usize,
        t_start: f64,
        t_end: f64,
        missingness_fraction: f64,
        unit_coverage_fraction: f64,
        duplicate_timestamp_fraction: f64,
        clock_drift: f64,
    ) -> Self {
        let mut warnings = Vec::new();
        if missingness_fraction > 0.5 {
            warnings.push(format!(
                "high missingness: {:.0}%",
                100.0 * missingness_fraction
            ));
        }
        if duplicate_timestamp_fraction > 0.0 {
            warnings.push(format!(
                "duplicate timestamps: {:.2}%",
                100.0 * duplicate_timestamp_fraction
            ));
        }
        if clock_drift.abs() > 0.0 {
            warnings.push(format!("clock drift: {clock_drift}"));
        }
        let verdict = if n_rows == 0 || n_tags == 0 {
            ImportVerdict::Rejected
        } else if warnings.is_empty() {
            ImportVerdict::Accepted
        } else {
            ImportVerdict::AcceptedWithWarnings
        };
        let mut r = HistorianImportReceiptV1 {
            file_name: file_name.into(),
            file_hash: file_hash.into(),
            n_rows,
            n_tags,
            t_start,
            t_end,
            missingness_fraction,
            unit_coverage_fraction,
            duplicate_timestamp_fraction,
            clock_drift,
            warnings,
            verdict: verdict.tag().into(),
            receipt_hash: String::new(),
        };
        r.receipt_hash = r.seal();
        r
    }

    pub fn accepted(&self) -> bool {
        self.verdict != ImportVerdict::Rejected.tag()
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.receipt_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_satisfied_when_required_present() {
        let c = PlantDataContractV1::check(&["tags.csv", "baseline_window", "units.csv"]);
        assert!(c.satisfied() && c.verdict == "Satisfied");
        assert_eq!(c.n_required_missing, 0);
        // units provided, the rest optional missing.
        assert!(c.n_optional_missing >= 6);
        assert!(c.contract_hash.len() == 64 && c.verify());
    }

    #[test]
    fn contract_missing_required_is_flagged() {
        let c = PlantDataContractV1::check(&["units.csv"]); // no tags / baseline
        assert!(!c.satisfied() && c.verdict == "MissingRequired");
        assert_eq!(c.n_required_missing, 2);
        assert!(c.verify());
        let mut t = c.clone();
        t.verdict = "Satisfied".into(); // forge a pass
        assert!(!t.verify());
    }

    #[test]
    fn import_receipt_verdicts_and_warnings() {
        // Clean file → Accepted.
        let clean = HistorianImportReceiptV1::build(
            "tags.csv",
            "ab".repeat(32),
            5000,
            40,
            0.0,
            72.0,
            0.01,
            1.0,
            0.0,
            0.0,
        );
        assert!(clean.accepted() && clean.verdict == "Accepted" && clean.warnings.is_empty());
        assert!(clean.receipt_hash.len() == 64 && clean.verify());
        // Dupes + drift → AcceptedWithWarnings.
        let warned = HistorianImportReceiptV1::build(
            "t.csv",
            "cd".repeat(32),
            100,
            5,
            0.0,
            1.0,
            0.0,
            1.0,
            0.02,
            3.0,
        );
        assert_eq!(warned.verdict, "AcceptedWithWarnings");
        assert_eq!(warned.warnings.len(), 2);
        // Empty → Rejected.
        let empty = HistorianImportReceiptV1::build(
            "e.csv",
            "00".repeat(32),
            0,
            0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!empty.accepted() && empty.verdict == "Rejected");
    }

    #[test]
    fn import_receipt_is_tamper_evident() {
        let mut r = HistorianImportReceiptV1::build(
            "t.csv",
            "ab".repeat(32),
            100,
            5,
            0.0,
            1.0,
            0.6,
            1.0,
            0.0,
            0.0,
        );
        assert!(r.verify());
        r.missingness_fraction = 0.0; // forge away the high-missingness fact
        assert!(!r.verify());
    }
}
