//! Confidential export primitives (Wave-6 confidential-evaluation chain): `CaseFileRedactionMapV1`,
//! `TamperEvidenceSealV1`, and `AuditTrailExportV1`.
//!
//! The mechanisms that let an operator share *evidence* without sharing *data*: a redaction map that
//! preserves proof links while stripping real names/values, an append-only tamper-evident hash-chain seal,
//! and a manifest of the compliance-friendly export formats — each sealed.
//!
//!   * [`CaseFileRedactionMapV1`] — per-tag `(alias, tag_name_hash, unit_class, variable_group)`; **the
//!     shared map contains no real tag names and no raw values** (the operator keeps the real↔alias mapping
//!     private), yet the `tag_name_hash` lets the operator later prove which alias is which tag.
//!   * [`TamperEvidenceSealV1`] — a bundle's `bundle_root` over its file hashes **chained to the previous
//!     bundle's root**, so a tampered or reordered history is detectable (append-only, git-like).
//!   * [`AuditTrailExportV1`] — a manifest of the formats a case file was exported to (JSONL / CSV / HTML /
//!     Merkle-proof / WORM manifest) with each export's content hash.
//!
//! Bounded (non-claims): redaction preserves *structure + proof links*, not semantics a reviewer might infer;
//! the tamper seal is a hash chain (optional crypto signing is a field, not provided here) — it detects
//! tampering, it does not by itself authenticate the author. Additive + off the replay path; deterministic,
//! hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::{sha256_hex, CanonicalHasher};

// ── CaseFileRedactionMapV1 ──────────────────────────────────────────────────────────────────────────

/// One redacted tag entry — what is safe to share. No real name, no raw value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedactionEntry {
    /// The neutral alias shown to a reviewer (e.g. `"TAG_0007"`).
    pub alias: String,
    /// SHA-256 of the real tag name (lets the operator later prove alias↔tag without revealing the name).
    pub tag_name_hash: String,
    /// Coarse unit class (e.g. `"temperature"`, `"pressure"`) — structure, not the exact unit/value.
    pub unit_class: String,
    /// Variable group (e.g. `"reactor"`, `"utility"`) — topology context, not the raw signal.
    pub variable_group: String,
}

/// A hash-sealed redaction map (schema v1) — the shareable alias↔structure mapping, no raw data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseFileRedactionMapV1 {
    pub entries: Vec<RedactionEntry>,
    pub map_hash: String,
}

impl CaseFileRedactionMapV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"case_file_redaction_map_v1");
        for e in &self.entries {
            h.field("alias", e.alias.as_bytes());
            h.field("tag_name_hash", e.tag_name_hash.as_bytes());
            h.field("unit_class", e.unit_class.as_bytes());
            h.field("variable_group", e.variable_group.as_bytes());
        }
        h.finalize_hex()
    }

    /// Build the shareable map from the operator's real `(tag_name, unit_class, variable_group)` rows. Aliases
    /// are assigned positionally (`TAG_0000…`); the real name is hashed, never stored — so the returned map is
    /// safe to share. (The operator retains the real↔alias correspondence privately.)
    pub fn build(rows: &[(String, String, String)]) -> Self {
        let entries: Vec<RedactionEntry> = rows
            .iter()
            .enumerate()
            .map(
                |(i, (real_name, unit_class, variable_group))| RedactionEntry {
                    alias: format!("TAG_{i:04}"),
                    tag_name_hash: sha256_hex(real_name.as_bytes()),
                    unit_class: unit_class.clone(),
                    variable_group: variable_group.clone(),
                },
            )
            .collect();
        let mut m = CaseFileRedactionMapV1 {
            entries,
            map_hash: String::new(),
        };
        m.map_hash = m.seal();
        m
    }

    /// True iff no entry leaks a real tag name (every `tag_name_hash` is a 64-hex digest, never plaintext).
    /// A defensive check that the shareable map carries only hashes.
    pub fn is_safe_to_share(&self) -> bool {
        self.entries.iter().all(|e| {
            e.tag_name_hash.len() == 64 && e.tag_name_hash.bytes().all(|b| b.is_ascii_hexdigit())
        })
    }

    /// Prove (privately) that an alias corresponds to a given real tag name, by re-hashing the name.
    pub fn alias_matches(&self, alias: &str, real_name: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.alias == alias && e.tag_name_hash == sha256_hex(real_name.as_bytes()))
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.map_hash
    }
}

// ── TamperEvidenceSealV1 ────────────────────────────────────────────────────────────────────────────

/// A hash-sealed, append-only tamper-evidence seal (schema v1) for a bundle, chained to the previous bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperEvidenceSealV1 {
    /// `(file_name, content_hash)` for every file in the bundle, in order.
    pub file_hashes: Vec<(String, String)>,
    /// The previous bundle's `bundle_root` (None for the first link) — this is what makes the chain append-only.
    pub previous_bundle_root: Option<String>,
    /// Optional signing-key id (crypto signing is out of scope here; hash-chaining is the default).
    pub signing_key_id: Option<String>,
    /// Optional timestamp (seconds); recorded but not trusted as authoritative.
    pub timestamp: Option<u64>,
    /// SHA-256 over the file hashes + previous root — the chain link for this bundle.
    pub bundle_root: String,
    pub seal_hash: String,
}

impl TamperEvidenceSealV1 {
    fn compute_bundle_root(
        file_hashes: &[(String, String)],
        previous_bundle_root: &Option<String>,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"tamper_evidence_bundle_root_v1");
        for (name, hash) in file_hashes {
            h.field("file_name", name.as_bytes());
            h.field("content_hash", hash.as_bytes());
        }
        h.field(
            "previous_bundle_root",
            previous_bundle_root.as_deref().unwrap_or("").as_bytes(),
        );
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"tamper_evidence_seal_v1");
        h.field("bundle_root", self.bundle_root.as_bytes());
        h.field(
            "previous_bundle_root",
            self.previous_bundle_root
                .as_deref()
                .unwrap_or("")
                .as_bytes(),
        );
        h.field(
            "signing_key_id",
            self.signing_key_id.as_deref().unwrap_or("").as_bytes(),
        );
        h.u64("has_timestamp", self.timestamp.is_some() as u64);
        h.u64("timestamp", self.timestamp.unwrap_or(0));
        h.finalize_hex()
    }

    /// Build a seal for a bundle, chaining to `previous_bundle_root`.
    pub fn build(
        file_hashes: Vec<(String, String)>,
        previous_bundle_root: Option<String>,
        signing_key_id: Option<String>,
        timestamp: Option<u64>,
    ) -> Self {
        let bundle_root = Self::compute_bundle_root(&file_hashes, &previous_bundle_root);
        let mut s = TamperEvidenceSealV1 {
            file_hashes,
            previous_bundle_root,
            signing_key_id,
            timestamp,
            bundle_root,
            seal_hash: String::new(),
        };
        s.seal_hash = s.seal();
        s
    }

    /// True iff `next` correctly chains onto `self` (its `previous_bundle_root` equals `self.bundle_root`).
    pub fn chains_to(&self, next: &TamperEvidenceSealV1) -> bool {
        next.previous_bundle_root.as_deref() == Some(self.bundle_root.as_str())
    }

    pub fn verify(&self) -> bool {
        Self::compute_bundle_root(&self.file_hashes, &self.previous_bundle_root) == self.bundle_root
            && self.seal() == self.seal_hash
    }
}

// ── AuditTrailExportV1 ──────────────────────────────────────────────────────────────────────────────

/// One exported audit-trail artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditExport {
    /// Format id (`"jsonl"` / `"csv"` / `"html"` / `"merkle_proof"` / `"worm_manifest"`).
    pub format: String,
    pub file_name: String,
    pub content_hash: String,
}

/// A hash-sealed audit-trail export manifest (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditTrailExportV1 {
    pub case_ref: String,
    pub exports: Vec<AuditExport>,
    pub manifest_hash: String,
}

impl AuditTrailExportV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"audit_trail_export_v1");
        h.field("case_ref", self.case_ref.as_bytes());
        for e in &self.exports {
            h.field("format", e.format.as_bytes());
            h.field("file_name", e.file_name.as_bytes());
            h.field("content_hash", e.content_hash.as_bytes());
        }
        h.finalize_hex()
    }

    pub fn build(case_ref: impl Into<String>, exports: Vec<AuditExport>) -> Self {
        let mut m = AuditTrailExportV1 {
            case_ref: case_ref.into(),
            exports,
            manifest_hash: String::new(),
        };
        m.manifest_hash = m.seal();
        m
    }

    /// True iff the manifest includes the named format.
    pub fn has_format(&self, format: &str) -> bool {
        self.exports.iter().any(|e| e.format == format)
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.manifest_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn redaction_map_hides_names_but_keeps_proof_links() {
        let rows = vec![
            (s("TIC-101.PV"), s("temperature"), s("reactor")),
            (s("FIC-204.PV"), s("flow"), s("utility")),
        ];
        let m = CaseFileRedactionMapV1::build(&rows);
        assert!(m.is_safe_to_share()); // only hashes, no plaintext names
        assert_eq!(m.entries[0].alias, "TAG_0000");
        assert_eq!(m.entries[0].unit_class, "temperature");
        // The shared map carries no real name, yet the operator can prove the alias↔tag link.
        assert!(m.alias_matches("TAG_0000", "TIC-101.PV"));
        assert!(!m.alias_matches("TAG_0000", "WRONG.PV"));
        assert!(m.map_hash.len() == 64 && m.verify());
    }

    #[test]
    fn tamper_seal_chains_append_only() {
        let b1 = TamperEvidenceSealV1::build(
            vec![(s("casefile.json"), "aa".repeat(32))],
            None,
            None,
            Some(1700000000),
        );
        let b2 = TamperEvidenceSealV1::build(
            vec![(s("casefile.json"), "bb".repeat(32))],
            Some(b1.bundle_root.clone()),
            None,
            Some(1700000100),
        );
        assert!(b1.verify() && b2.verify());
        assert!(b1.chains_to(&b2)); // b2 references b1's root
        assert!(!b2.chains_to(&b1)); // not the other way
                                     // Tamper a file hash → the bundle_root no longer matches → verify fails.
        let mut t = b2.clone();
        t.file_hashes[0].1 = "cc".repeat(32);
        assert!(!t.verify());
    }

    #[test]
    fn audit_trail_lists_formats_and_self_verifies() {
        let m = AuditTrailExportV1::build(
            "case-001",
            vec![
                AuditExport {
                    format: s("jsonl"),
                    file_name: s("case.jsonl"),
                    content_hash: "ab".repeat(32),
                },
                AuditExport {
                    format: s("worm_manifest"),
                    file_name: s("WORM.toml"),
                    content_hash: "cd".repeat(32),
                },
            ],
        );
        assert!(m.has_format("jsonl") && m.has_format("worm_manifest") && !m.has_format("pdf"));
        assert!(m.manifest_hash.len() == 64 && m.verify());
        let mut t = m.clone();
        t.exports[0].content_hash = "00".repeat(32);
        assert!(!t.verify());
    }
}
