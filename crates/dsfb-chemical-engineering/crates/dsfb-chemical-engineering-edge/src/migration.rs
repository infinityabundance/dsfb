//! Governed authority-drift objects (P67): `SemanticDiffReportV1`, `DetectorRegistryCompatibilityV1`,
//! `HeuristicMigrationReceiptV1`.
//!
//! When the authority changes (a detector added, a heuristic edited, `atlas_hash_v1` re-frozen — as in
//! P60), the change must be *governed*, not silent. Three hash-sealed objects record exactly what moved:
//! - [`SemanticDiffReportV1`] — a structural diff between two record snapshots (id → content-hash):
//!   what was added, removed, changed, unchanged.
//! - [`DetectorRegistryCompatibilityV1`] — the same diff specialised to the detector registry, with a
//!   `backward_compatible` verdict (true iff nothing a downstream caller relied on was *removed*).
//! - [`HeuristicMigrationReceiptV1`] — a per-heuristic migration receipt (from/to version + hashes +
//!   the disclosed changed fields).
//!
//! A snapshot is just `id → content_hash`, so these work on any versioned record set. Additive + off the
//! replay path.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// A versioned record snapshot: stable id → content hash. The unit these objects diff over.
pub type RecordSnapshot = BTreeMap<String, String>;

/// Compute (added, removed, changed, unchanged) between two snapshots, deterministically (sorted).
fn diff_snapshots(
    from: &RecordSnapshot,
    to: &RecordSnapshot,
) -> (Vec<String>, Vec<String>, Vec<String>, usize) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = 0usize;
    for (id, to_hash) in to {
        match from.get(id) {
            None => added.push(id.clone()),
            Some(from_hash) if from_hash != to_hash => changed.push(id.clone()),
            Some(_) => unchanged += 1,
        }
    }
    for id in from.keys() {
        if !to.contains_key(id) {
            removed.push(id.clone());
        }
    }
    added.sort();
    removed.sort();
    changed.sort();
    (added, removed, changed, unchanged)
}

/// A hash-sealed semantic diff between two record snapshots (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiffReportV1 {
    pub from_label: String,
    pub to_label: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub unchanged: usize,
    pub diff_hash: String,
}

impl SemanticDiffReportV1 {
    fn seal(fl: &str, tl: &str, a: &[String], r: &[String], c: &[String], u: usize) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"semantic_diff_report_v1");
        h.field("from_label", fl.as_bytes());
        h.field("to_label", tl.as_bytes());
        for x in a {
            h.field("added", x.as_bytes());
        }
        for x in r {
            h.field("removed", x.as_bytes());
        }
        for x in c {
            h.field("changed", x.as_bytes());
        }
        h.u64("unchanged", u as u64);
        h.finalize_hex()
    }

    pub fn diff(
        from_label: impl Into<String>,
        to_label: impl Into<String>,
        from: &RecordSnapshot,
        to: &RecordSnapshot,
    ) -> Self {
        let (added, removed, changed, unchanged) = diff_snapshots(from, to);
        let from_label = from_label.into();
        let to_label = to_label.into();
        let diff_hash = Self::seal(
            &from_label,
            &to_label,
            &added,
            &removed,
            &changed,
            unchanged,
        );
        SemanticDiffReportV1 {
            from_label,
            to_label,
            added,
            removed,
            changed,
            unchanged,
            diff_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.from_label,
            &self.to_label,
            &self.added,
            &self.removed,
            &self.changed,
            self.unchanged,
        ) == self.diff_hash
    }
}

/// Detector-registry compatibility verdict between two versions (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectorRegistryCompatibilityV1 {
    pub from_version: String,
    pub to_version: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    /// True iff no detector a downstream caller relied on was **removed** (added/changed are allowed).
    pub backward_compatible: bool,
    pub report_hash: String,
}

impl DetectorRegistryCompatibilityV1 {
    fn seal(fv: &str, tv: &str, a: &[String], r: &[String], c: &[String], compat: bool) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"detector_registry_compatibility_v1");
        h.field("from_version", fv.as_bytes());
        h.field("to_version", tv.as_bytes());
        for x in a {
            h.field("added", x.as_bytes());
        }
        for x in r {
            h.field("removed", x.as_bytes());
        }
        for x in c {
            h.field("changed", x.as_bytes());
        }
        h.u64("backward_compatible", compat as u64);
        h.finalize_hex()
    }

    pub fn check(
        from_version: impl Into<String>,
        to_version: impl Into<String>,
        from: &RecordSnapshot,
        to: &RecordSnapshot,
    ) -> Self {
        let (added, removed, changed, _unchanged) = diff_snapshots(from, to);
        let backward_compatible = removed.is_empty();
        let from_version = from_version.into();
        let to_version = to_version.into();
        let report_hash = Self::seal(
            &from_version,
            &to_version,
            &added,
            &removed,
            &changed,
            backward_compatible,
        );
        DetectorRegistryCompatibilityV1 {
            from_version,
            to_version,
            added,
            removed,
            changed,
            backward_compatible,
            report_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.from_version,
            &self.to_version,
            &self.added,
            &self.removed,
            &self.changed,
            self.backward_compatible,
        ) == self.report_hash
    }
}

/// A per-heuristic migration receipt (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeuristicMigrationReceiptV1 {
    pub heuristic_id: String,
    pub from_version: String,
    pub to_version: String,
    pub from_hash: String,
    pub to_hash: String,
    /// True iff `from_hash != to_hash`.
    pub changed: bool,
    /// Disclosed list of which fields changed (caller-supplied; empty if unchanged).
    pub changed_fields: Vec<String>,
    pub receipt_hash: String,
}

impl HeuristicMigrationReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    fn seal(
        id: &str,
        fv: &str,
        tv: &str,
        fh: &str,
        th: &str,
        changed: bool,
        fields: &[String],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"heuristic_migration_receipt_v1");
        h.field("heuristic_id", id.as_bytes());
        h.field("from_version", fv.as_bytes());
        h.field("to_version", tv.as_bytes());
        h.field("from_hash", fh.as_bytes());
        h.field("to_hash", th.as_bytes());
        h.u64("changed", changed as u64);
        for f in fields {
            h.field("changed_field", f.as_bytes());
        }
        h.finalize_hex()
    }

    pub fn build(
        heuristic_id: impl Into<String>,
        from_version: impl Into<String>,
        to_version: impl Into<String>,
        from_hash: impl Into<String>,
        to_hash: impl Into<String>,
        changed_fields: Vec<String>,
    ) -> Self {
        let heuristic_id = heuristic_id.into();
        let from_version = from_version.into();
        let to_version = to_version.into();
        let from_hash = from_hash.into();
        let to_hash = to_hash.into();
        let changed = from_hash != to_hash;
        let receipt_hash = Self::seal(
            &heuristic_id,
            &from_version,
            &to_version,
            &from_hash,
            &to_hash,
            changed,
            &changed_fields,
        );
        HeuristicMigrationReceiptV1 {
            heuristic_id,
            from_version,
            to_version,
            from_hash,
            to_hash,
            changed,
            changed_fields,
            receipt_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.heuristic_id,
            &self.from_version,
            &self.to_version,
            &self.from_hash,
            &self.to_hash,
            self.changed,
            &self.changed_fields,
        ) == self.receipt_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(pairs: &[(&str, &str)]) -> RecordSnapshot {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn semantic_diff_classifies_added_removed_changed_unchanged() {
        let from = snap(&[("a", "h1"), ("b", "h2"), ("c", "h3")]);
        let to = snap(&[("a", "h1"), ("b", "h2x"), ("d", "h4")]); // a unchanged, b changed, c removed, d added
        let d = SemanticDiffReportV1::diff("v1", "v2", &from, &to);
        assert_eq!(d.added, vec!["d"]);
        assert_eq!(d.removed, vec!["c"]);
        assert_eq!(d.changed, vec!["b"]);
        assert_eq!(d.unchanged, 1);
        assert!(d.verify());
    }

    #[test]
    fn registry_compat_is_false_iff_something_removed() {
        let from = snap(&[("pca_t2", "h"), ("ewma_spe", "h")]);
        let added_only = snap(&[("pca_t2", "h"), ("ewma_spe", "h"), ("psi", "h")]);
        let c1 = DetectorRegistryCompatibilityV1::check("v1", "v2", &from, &added_only);
        assert!(c1.backward_compatible && c1.added == vec!["psi"] && c1.verify());
        let removed = snap(&[("pca_t2", "h")]); // ewma_spe removed
        let c2 = DetectorRegistryCompatibilityV1::check("v1", "v3", &from, &removed);
        assert!(!c2.backward_compatible && c2.removed == vec!["ewma_spe"] && c2.verify());
    }

    #[test]
    fn heuristic_migration_receipt_detects_change_and_self_verifies() {
        let r = HeuristicMigrationReceiptV1::build(
            "H6",
            "v1",
            "v2",
            "oldhash",
            "newhash",
            vec!["slew_condition".into()],
        );
        assert!(r.changed && r.verify());
        let same = HeuristicMigrationReceiptV1::build("H1", "v1", "v2", "h", "h", vec![]);
        assert!(!same.changed && same.verify());
    }
}
