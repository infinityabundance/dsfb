//! `ProcessNarrativeCompilerV1` + `NoNarrativeHallucinationGateV1` (P66) — the flagship
//! anti-hallucination disclosure.
//!
//! Operators want prose. The danger of prose is hallucination — sentences that read fluently but assert
//! more than the evidence supports. DSFB's answer: a narrative compiler that is **not an LLM**. Every
//! sentence is produced by a **fixed deterministic template** filled from structured fields, and every
//! sentence carries the **hash of the specific evidence object it was derived from** (`anchor_hash`).
//! There is no free-text path. [`NoNarrativeHallucinationGateV1`] then mechanically proves the property
//! that matters: **every sentence is anchored to a real, sealed evidence object** — if any sentence's
//! anchor is not in the case's evidence set, the gate fails. A narrative that passes the gate *cannot*
//! contain a claim that is not backed by sealed evidence.
//!
//! Additive + off the replay path. Deterministic templating only — no model, no sampling, no generation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One sentence of the narrative: its rendered text, the kind of evidence it came from, and the hash of
/// the specific evidence object that anchors it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarrativeSentence {
    pub text: String,
    /// The evidence-object kind that produced this sentence (e.g. `"episode"`, `"label"`, `"witness"`).
    pub evidence_kind: String,
    /// SHA-256 of the evidence object this sentence is derived from — the anchor the gate checks.
    pub anchor_hash: String,
}

/// A deterministic, evidence-anchored process narrative (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessNarrativeCompilerV1 {
    pub dataset: String,
    pub sentences: Vec<NarrativeSentence>,
    pub narrative_hash: String,
}

/// Builder for a narrative. Each method appends one fixed-template sentence bound to an evidence hash;
/// there is deliberately no method that takes free-form sentence text.
#[derive(Debug, Clone, Default)]
pub struct NarrativeBuilder {
    dataset: String,
    sentences: Vec<NarrativeSentence>,
}

impl NarrativeBuilder {
    pub fn new(dataset: impl Into<String>) -> Self {
        NarrativeBuilder {
            dataset: dataset.into(),
            sentences: Vec::new(),
        }
    }

    fn push(&mut self, evidence_kind: &str, text: String, anchor_hash: impl Into<String>) {
        self.sentences.push(NarrativeSentence {
            text,
            evidence_kind: evidence_kind.into(),
            anchor_hash: anchor_hash.into(),
        });
    }

    /// Episode sentence (template) — anchored to the episode/forensics evidence hash.
    pub fn episode(
        &mut self,
        episode_ref: &str,
        dominant_motif: &str,
        anchor_hash: impl Into<String>,
    ) -> &mut Self {
        self.push(
            "episode",
            format!("Episode {episode_ref}: the dominant structural motif is {dominant_motif}."),
            anchor_hash,
        );
        self
    }

    /// Operator-label sentence (template) — anchored to the heuristic/label evidence hash. Always marked
    /// advisory (the templates cannot emit a root-cause claim).
    pub fn label(&mut self, label: &str, anchor_hash: impl Into<String>) -> &mut Self {
        self.push(
            "label",
            format!("Candidate label (advisory, not root cause): {label}."),
            anchor_hash,
        );
        self
    }

    /// Witness sentence (template) — anchored to the detector/passport evidence hash.
    pub fn witness(
        &mut self,
        detector_id: &str,
        role: &str,
        anchor_hash: impl Into<String>,
    ) -> &mut Self {
        self.push(
            "witness",
            format!("Detector {detector_id} testified as a {role} witness."),
            anchor_hash,
        );
        self
    }

    /// Ruled-out sentence (template) — anchored to the confuser-docket evidence hash.
    pub fn ruled_out(&mut self, confuser: &str, anchor_hash: impl Into<String>) -> &mut Self {
        self.push(
            "ruled_out",
            format!("Ruled out (confuser considered): {confuser}."),
            anchor_hash,
        );
        self
    }

    /// Balance-witness sentence (template) — anchored to the balance-witness evidence hash.
    pub fn balance(
        &mut self,
        kind: &str,
        peak_residual: f64,
        units: &str,
        anchor_hash: impl Into<String>,
    ) -> &mut Self {
        self.push(
            "balance",
            format!("Balance witness ({kind}): peak closure residual {peak_residual} {units}."),
            anchor_hash,
        );
        self
    }

    /// Finalise + seal the narrative.
    pub fn seal(self) -> ProcessNarrativeCompilerV1 {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"process_narrative_compiler_v1");
        h.field("dataset", self.dataset.as_bytes());
        for s in &self.sentences {
            h.field("text", s.text.as_bytes());
            h.field("evidence_kind", s.evidence_kind.as_bytes());
            h.field("anchor_hash", s.anchor_hash.as_bytes());
        }
        ProcessNarrativeCompilerV1 {
            dataset: self.dataset,
            sentences: self.sentences,
            narrative_hash: h.finalize_hex(),
        }
    }
}

impl ProcessNarrativeCompilerV1 {
    pub fn verify(&self) -> bool {
        let mut b = NarrativeBuilder::new(self.dataset.clone());
        b.sentences = self.sentences.clone();
        b.seal().narrative_hash == self.narrative_hash
    }

    /// Render the narrative as plain text (one sentence per line, each with its anchor).
    pub fn to_text(&self) -> String {
        let mut out = format!("Process narrative — {}\n", self.dataset);
        for s in &self.sentences {
            out.push_str(&format!(
                "- {} [{}:{}…]\n",
                s.text,
                s.evidence_kind,
                &s.anchor_hash[..8.min(s.anchor_hash.len())]
            ));
        }
        out
    }

    /// Assemble a narrative from explicit sentences. This is an **adversarial / synthetic** constructor for the
    /// narration-failure demonstrator (`crate::narration_heuristic_demo`) — it lets a test build the deliberately
    /// malformed narratives an *untrusted* narrator might emit, so the H7–H12 detector can be shown to catch them.
    /// It is deliberately NOT part of the production narrator (the safe `NarrativeBuilder` templates remain the only
    /// emission path), and is compiled only behind the `narration-heuristic-demo` feature. Seals identically.
    #[cfg(feature = "narration-heuristic-demo")]
    pub fn from_sentences(dataset: impl Into<String>, sentences: Vec<NarrativeSentence>) -> Self {
        let mut b = NarrativeBuilder::new(dataset);
        b.sentences = sentences;
        b.seal()
    }
}

/// The result of gating a narrative for hallucination (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoNarrativeHallucinationGateV1 {
    pub narrative_hash: String,
    pub n_sentences: usize,
    pub n_anchored: usize,
    /// True iff every sentence's `anchor_hash` is present in the case's known evidence-object hashes.
    pub all_anchored: bool,
    /// Texts of any sentences whose anchor was *not* found (a hallucination would appear here).
    pub unanchored: Vec<String>,
    pub gate_hash: String,
}

impl NoNarrativeHallucinationGateV1 {
    /// Check the narrative against the set of real evidence-object hashes for the case. A sentence is
    /// "anchored" iff its `anchor_hash` is in `known_anchors`. The gate passes (`all_anchored`) iff every
    /// sentence is anchored — i.e. no sentence asserts anything not backed by a sealed evidence object.
    pub fn check(narrative: &ProcessNarrativeCompilerV1, known_anchors: &BTreeSet<String>) -> Self {
        let mut unanchored: Vec<String> = Vec::new();
        let mut n_anchored = 0usize;
        for s in &narrative.sentences {
            if known_anchors.contains(&s.anchor_hash) {
                n_anchored += 1;
            } else {
                unanchored.push(s.text.clone());
            }
        }
        let n_sentences = narrative.sentences.len();
        let all_anchored = unanchored.is_empty();
        let gate_hash = Self::seal(
            &narrative.narrative_hash,
            n_sentences,
            n_anchored,
            all_anchored,
            &unanchored,
        );
        NoNarrativeHallucinationGateV1 {
            narrative_hash: narrative.narrative_hash.clone(),
            n_sentences,
            n_anchored,
            all_anchored,
            unanchored,
            gate_hash,
        }
    }

    fn seal(
        narrative_hash: &str,
        n: usize,
        n_anchored: usize,
        all: bool,
        unanchored: &[String],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"no_narrative_hallucination_gate_v1");
        h.field("narrative_hash", narrative_hash.as_bytes());
        h.u64("n_sentences", n as u64);
        h.u64("n_anchored", n_anchored as u64);
        h.u64("all_anchored", all as u64);
        for u in unanchored {
            h.field("unanchored", u.as_bytes());
        }
        h.finalize_hex()
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.narrative_hash,
            self.n_sentences,
            self.n_anchored,
            self.all_anchored,
            &self.unanchored,
        ) == self.gate_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sentence_is_anchored_and_gate_passes() {
        // Three evidence objects with known hashes.
        let (h_ep, h_lbl, h_conf) = ("a".repeat(64), "b".repeat(64), "c".repeat(64));
        let mut b = NarrativeBuilder::new("cstr_reactor");
        b.episode("120..168", "EnvViolation/Compound", h_ep.clone())
            .label("reactor thermal excursion candidate", h_lbl.clone())
            .ruled_out("ambient/utility temperature drift", h_conf.clone());
        let narrative = b.seal();
        assert_eq!(narrative.sentences.len(), 3);
        assert!(narrative.verify());
        // Templates cannot emit a root-cause claim — the label sentence is marked advisory.
        assert!(narrative.sentences[1]
            .text
            .contains("advisory, not root cause"));

        let known: BTreeSet<String> = [h_ep, h_lbl, h_conf].into_iter().collect();
        let gate = NoNarrativeHallucinationGateV1::check(&narrative, &known);
        assert!(gate.all_anchored && gate.n_anchored == 3 && gate.unanchored.is_empty());
        assert!(gate.verify());
    }

    #[test]
    fn an_unanchored_sentence_fails_the_gate() {
        let mut b = NarrativeBuilder::new("ds");
        b.episode("0..10", "DriftAccum", "d".repeat(64));
        let narrative = b.seal();
        // The case's evidence set does NOT contain the sentence's anchor → hallucination caught.
        let known: BTreeSet<String> = ["z".repeat(64)].into_iter().collect();
        let gate = NoNarrativeHallucinationGateV1::check(&narrative, &known);
        assert!(!gate.all_anchored);
        assert_eq!(gate.unanchored.len(), 1);
        assert!(gate.verify());
    }
}
