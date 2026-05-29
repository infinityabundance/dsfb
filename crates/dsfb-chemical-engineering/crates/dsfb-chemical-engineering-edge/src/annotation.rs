//! Context overlays + append-only human-review chains (P68): `RecipeTransitionGuardV1`,
//! `MaintenanceEventOverlayV1`, `OperatorAnnotationLedgerV1`, `EvidenceAmendmentChainV1`.
//!
//! DSFB's evidence is read-only and immutable. Real operations need to layer *context* (a batch recipe
//! changed phase; a unit was under maintenance) and *human review* (an operator note; a correction) on
//! top of that evidence — without ever mutating the sealed record. These four objects do exactly that:
//! - [`RecipeTransitionGuardV1`] / [`MaintenanceEventOverlayV1`] — read-only overlays marking windows
//!   where residuals are *expected* (a recipe phase change, a maintenance outage), so an episode in such
//!   a window is contextualised, not blindly alarmed. They annotate; they never change the evidence.
//! - [`OperatorAnnotationLedgerV1`] / [`EvidenceAmendmentChainV1`] — **append-only, hash-chained** logs.
//!   Each entry hashes the previous entry's hash, so the log is tamper-evident: a past entry cannot be
//!   altered or removed without breaking the chain. The amendment chain's genesis link is the *original
//!   sealed evidence hash*, which is never mutated — amendments are appended, the original stands.
//!
//! Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Genesis previous-hash for an annotation ledger (no prior evidence to chain from).
const ANNOTATION_GENESIS: &str = "annotation_ledger_genesis";

// ── Read-only context overlays ─────────────────────────────────────────────────────────────────────

/// A batch-recipe phase transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeTransition {
    pub at_index: usize,
    pub from_phase: String,
    pub to_phase: String,
}

/// A read-only recipe-transition guard (schema v1): residuals within `window` samples of a transition
/// are recipe-driven context, not necessarily faults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeTransitionGuardV1 {
    pub dataset: String,
    pub transitions: Vec<RecipeTransition>,
    pub guard_hash: String,
}

impl RecipeTransitionGuardV1 {
    pub fn build(dataset: impl Into<String>, transitions: Vec<RecipeTransition>) -> Self {
        let dataset = dataset.into();
        let mut h = CanonicalHasher::new();
        h.field("schema", b"recipe_transition_guard_v1");
        h.field("dataset", dataset.as_bytes());
        for t in &transitions {
            h.u64("at_index", t.at_index as u64);
            h.field("from_phase", t.from_phase.as_bytes());
            h.field("to_phase", t.to_phase.as_bytes());
        }
        RecipeTransitionGuardV1 {
            dataset,
            transitions,
            guard_hash: h.finalize_hex(),
        }
    }

    /// True iff `index` is within `window` samples of any recipe transition (a transition window).
    pub fn in_transition_window(&self, index: usize, window: usize) -> bool {
        self.transitions
            .iter()
            .any(|t| index.abs_diff(t.at_index) <= window)
    }

    pub fn verify(&self) -> bool {
        Self::build(self.dataset.clone(), self.transitions.clone()).guard_hash == self.guard_hash
    }
}

/// A maintenance event window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceEvent {
    pub start_index: usize,
    pub end_index: usize,
    pub description: String,
}

/// A read-only maintenance-event overlay (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceEventOverlayV1 {
    pub dataset: String,
    pub events: Vec<MaintenanceEvent>,
    pub overlay_hash: String,
}

impl MaintenanceEventOverlayV1 {
    pub fn build(dataset: impl Into<String>, events: Vec<MaintenanceEvent>) -> Self {
        let dataset = dataset.into();
        let mut h = CanonicalHasher::new();
        h.field("schema", b"maintenance_event_overlay_v1");
        h.field("dataset", dataset.as_bytes());
        for e in &events {
            h.u64("start_index", e.start_index as u64);
            h.u64("end_index", e.end_index as u64);
            h.field("description", e.description.as_bytes());
        }
        MaintenanceEventOverlayV1 {
            dataset,
            events,
            overlay_hash: h.finalize_hex(),
        }
    }

    /// True iff `index` falls within any maintenance window `[start, end]`.
    pub fn covers(&self, index: usize) -> bool {
        self.events
            .iter()
            .any(|e| index >= e.start_index && index <= e.end_index)
    }

    pub fn verify(&self) -> bool {
        Self::build(self.dataset.clone(), self.events.clone()).overlay_hash == self.overlay_hash
    }
}

// ── Append-only, hash-chained logs ──────────────────────────────────────────────────────────────────

/// One operator-annotation entry (a link in the hash chain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationEntry {
    pub seq: usize,
    pub timestamp_utc: String,
    pub episode_ref: String,
    pub author: String,
    pub note: String,
    /// Hash of the previous entry (or the genesis for `seq == 0`).
    pub prev_hash: String,
    /// `H(seq, timestamp, episode_ref, author, note, prev_hash)`.
    pub entry_hash: String,
}

/// An append-only, hash-chained operator-annotation ledger (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorAnnotationLedgerV1 {
    pub dataset: String,
    pub entries: Vec<AnnotationEntry>,
    /// Hash of the last entry (the chain head), or the genesis if empty.
    pub head_hash: String,
}

impl OperatorAnnotationLedgerV1 {
    fn entry_hash(seq: usize, ts: &str, ep: &str, author: &str, note: &str, prev: &str) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"annotation_entry_v1");
        h.u64("seq", seq as u64);
        h.field("timestamp_utc", ts.as_bytes());
        h.field("episode_ref", ep.as_bytes());
        h.field("author", author.as_bytes());
        h.field("note", note.as_bytes());
        h.field("prev_hash", prev.as_bytes());
        h.finalize_hex()
    }

    pub fn new(dataset: impl Into<String>) -> Self {
        OperatorAnnotationLedgerV1 {
            dataset: dataset.into(),
            entries: Vec::new(),
            head_hash: ANNOTATION_GENESIS.to_string(),
        }
    }

    /// Append an annotation, chaining it to the current head. Returns the new entry's hash.
    pub fn append(
        &mut self,
        timestamp_utc: impl Into<String>,
        episode_ref: impl Into<String>,
        author: impl Into<String>,
        note: impl Into<String>,
    ) -> String {
        let seq = self.entries.len();
        let prev_hash = self.head_hash.clone();
        let (ts, ep, author, note) = (
            timestamp_utc.into(),
            episode_ref.into(),
            author.into(),
            note.into(),
        );
        let entry_hash = Self::entry_hash(seq, &ts, &ep, &author, &note, &prev_hash);
        self.entries.push(AnnotationEntry {
            seq,
            timestamp_utc: ts,
            episode_ref: ep,
            author,
            note,
            prev_hash,
            entry_hash: entry_hash.clone(),
        });
        self.head_hash = entry_hash.clone();
        entry_hash
    }

    /// Verify the chain: each entry's hash re-derives, and its `prev_hash` links the prior entry
    /// (genesis for the first); the head equals the last entry's hash. Detects any tamper/removal.
    pub fn verify_chain(&self) -> bool {
        let mut prev = ANNOTATION_GENESIS.to_string();
        for (i, e) in self.entries.iter().enumerate() {
            if e.seq != i || e.prev_hash != prev {
                return false;
            }
            let h = Self::entry_hash(
                e.seq,
                &e.timestamp_utc,
                &e.episode_ref,
                &e.author,
                &e.note,
                &e.prev_hash,
            );
            if h != e.entry_hash {
                return false;
            }
            prev = e.entry_hash.clone();
        }
        self.head_hash == prev
    }
}

/// One amendment entry (a link in the chain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmendmentEntry {
    pub seq: usize,
    pub timestamp_utc: String,
    /// What kind of amendment (e.g. `"correction"`, `"clarification"`, `"retraction-note"`).
    pub amendment_kind: String,
    pub amendment_text: String,
    pub prev_hash: String,
    pub entry_hash: String,
}

/// An append-only, hash-chained evidence-amendment chain (schema v1). The **original sealed evidence is
/// never mutated**: it is the genesis link, and all amendments are appended after it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceAmendmentChainV1 {
    /// The hash of the original sealed evidence (e.g. a Court Record `evidence_root`). Immutable genesis.
    pub original_evidence_hash: String,
    pub amendments: Vec<AmendmentEntry>,
    pub head_hash: String,
}

impl EvidenceAmendmentChainV1 {
    fn entry_hash(seq: usize, ts: &str, kind: &str, text: &str, prev: &str) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"amendment_entry_v1");
        h.u64("seq", seq as u64);
        h.field("timestamp_utc", ts.as_bytes());
        h.field("amendment_kind", kind.as_bytes());
        h.field("amendment_text", text.as_bytes());
        h.field("prev_hash", prev.as_bytes());
        h.finalize_hex()
    }

    /// Start a chain anchored to the original sealed evidence (the genesis = the original's hash).
    pub fn new(original_evidence_hash: impl Into<String>) -> Self {
        let original_evidence_hash = original_evidence_hash.into();
        let head_hash = original_evidence_hash.clone();
        EvidenceAmendmentChainV1 {
            original_evidence_hash,
            amendments: Vec::new(),
            head_hash,
        }
    }

    /// Append an amendment (never mutates the original). Returns the new entry's hash.
    pub fn amend(
        &mut self,
        timestamp_utc: impl Into<String>,
        amendment_kind: impl Into<String>,
        amendment_text: impl Into<String>,
    ) -> String {
        let seq = self.amendments.len();
        let prev_hash = self.head_hash.clone();
        let (ts, kind, text) = (
            timestamp_utc.into(),
            amendment_kind.into(),
            amendment_text.into(),
        );
        let entry_hash = Self::entry_hash(seq, &ts, &kind, &text, &prev_hash);
        self.amendments.push(AmendmentEntry {
            seq,
            timestamp_utc: ts,
            amendment_kind: kind,
            amendment_text: text,
            prev_hash,
            entry_hash: entry_hash.clone(),
        });
        self.head_hash = entry_hash.clone();
        entry_hash
    }

    /// Verify the chain links back to the immutable original evidence hash and is untampered.
    pub fn verify_chain(&self) -> bool {
        let mut prev = self.original_evidence_hash.clone();
        for (i, e) in self.amendments.iter().enumerate() {
            if e.seq != i || e.prev_hash != prev {
                return false;
            }
            let h = Self::entry_hash(
                e.seq,
                &e.timestamp_utc,
                &e.amendment_kind,
                &e.amendment_text,
                &e.prev_hash,
            );
            if h != e.entry_hash {
                return false;
            }
            prev = e.entry_hash.clone();
        }
        self.head_hash == prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlays_contextualise_windows_and_self_verify() {
        let guard = RecipeTransitionGuardV1::build(
            "penicillin_fedbatch",
            vec![RecipeTransition {
                at_index: 600,
                from_phase: "growth".into(),
                to_phase: "production".into(),
            }],
        );
        assert!(guard.in_transition_window(605, 10)); // within 10 of the transition
        assert!(!guard.in_transition_window(700, 10));
        assert!(guard.verify());

        let maint = MaintenanceEventOverlayV1::build(
            "swat",
            vec![MaintenanceEvent {
                start_index: 100,
                end_index: 150,
                description: "pump P-101 service".into(),
            }],
        );
        assert!(maint.covers(120) && !maint.covers(200) && maint.verify());
    }

    #[test]
    fn annotation_ledger_is_append_only_and_tamper_evident() {
        let mut led = OperatorAnnotationLedgerV1::new("cstr_reactor");
        led.append(
            "2026-05-25T10:00:00Z",
            "idx=120..168",
            "operator_A",
            "checked TC-3, looks like drift",
        );
        led.append(
            "2026-05-25T10:05:00Z",
            "idx=120..168",
            "operator_B",
            "scheduled recalibration",
        );
        assert_eq!(led.entries.len(), 2);
        assert!(led.verify_chain());
        // Tamper a past note → the chain breaks.
        let mut t = led.clone();
        t.entries[0].note = "nothing to see".into();
        assert!(!t.verify_chain());
    }

    #[test]
    fn amendment_chain_anchors_to_immutable_original() {
        let original = "e".repeat(64);
        let mut chain = EvidenceAmendmentChainV1::new(original.clone());
        chain.amend(
            "2026-05-25T11:00:00Z",
            "clarification",
            "onset re-dated to sample 162 after review",
        );
        assert!(chain.verify_chain());
        // The original evidence hash is never mutated.
        assert_eq!(chain.original_evidence_hash, original);
        // Re-anchoring to a different original (rewriting history) breaks the chain.
        let mut t = chain.clone();
        t.original_evidence_hash = "f".repeat(64);
        assert!(!t.verify_chain());
    }
}
