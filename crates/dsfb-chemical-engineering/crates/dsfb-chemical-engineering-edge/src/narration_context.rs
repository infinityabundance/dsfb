//! `NarrationContextV1` — the deterministic **narration context** emitted from a sealed Chemical Court Record.
//!
//! A constrained external narrator may re-present a Court Record as prose, but it has **no authority over the
//! record**: every sentence it emits must cite a sealed evidence object, and a sentence with no such anchor (or one
//! that asserts more than its anchor's claim tier permits) is rejected by [`crate::narrative::NoNarrativeHallucination
//! GateV1`]. For that to be checkable, the narrator needs, up front, the *exact* set of things it is allowed to say
//! about — this module emits that set as a document.
//!
//! The emitted context lists, for one analysed dataset:
//! - the **binding contract** the narrator must obey (cite one anchor per sentence; never create evidence, upgrade a
//!   claim tier, invent/assert a root cause, relabel an `unknown`, or alter the case file);
//! - the complete vocabulary of **citable evidence anchors** — one per fused episode, each carrying its episode
//!   reference, dominant motif, detector families, consensus/entropy, witness rung, evidence kind, and the
//!   **claim tier** that anchor may be asserted at (derived from the evidence kind via the single source of truth);
//! - the standing **non-claims** that may never be said;
//! - the dataset `evidence_root` it pertains to, and a deterministic `context_root` seal over all of the above.
//!
//! The anchors are produced by the SAME [`crate::report::episode_evidence_anchor`] the operator report uses, so a
//! sentence citing either is checkable against one shared anchor. This is a *navigation/contract* layer over sealed
//! evidence — it creates no evidence and is off the replay path (the `context_root` is not part of `evidence_root`).
//! It carries no chemical claim beyond what the record already sealed, and states nothing about the consumer's nature.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::evidence_kind::EvidenceKind;
use crate::hashing::CanonicalHasher;
use crate::pipeline::AnalysisResult;
use crate::report::{episode_evidence_anchor, episode_witness_strength};

/// Schema tag — folded into the `context_root` preimage (so a different schema can never collide with this one)
/// and emitted in the JSON, the way every other `…V1` object in the crate stamps its format.
pub const NARRATION_CONTEXT_FORMAT: &str = "dsfb_chemical_engineering_narration_context_v1";

/// The binding rules a narrator must obey. Purpose-neutral: it names no consumer and states no use.
const CONTRACT: [&str; 6] = [
    "A constrained external narrator may re-present this sealed record as prose; it has NO authority over the record.",
    "Every emitted sentence MUST cite exactly one evidence anchor from the set below (by its full anchor hash).",
    "It may NOT create evidence, assign or upgrade a claim tier, assert or invent a root cause / causal link / diagnosis, relabel an `unknown` episode as a named fault, merge/split/infer episodes beyond the record, or alter the sealed case file.",
    "Each sentence is validated by NoNarrativeHallucinationGateV1: a sentence whose anchor is not in this set is rejected.",
    "A sentence may not assert more than the claim tier its cited anchor carries (SealedFact > EvidenceInterpretation > SpeculativeImplication; NonClaim is never asserted).",
    "Everything said must already be true in the sealed record. If it is not anchored here, it may not be said.",
];

/// The standing non-claims — never asserted, by narrator or court.
const NON_CLAIMS: [&str; 6] = [
    "no proven physical root cause",
    "no causal link asserted",
    "no accuracy or detection-speed superiority over established methods",
    "no control signal and no safety-instrumented-function authority",
    "no regulatory-compliance certification",
    "every label is a CANDIDATE for operator review, never a confirmed diagnosis",
];

/// One citable evidence anchor: the handle a narrator references, plus the sealed facts it stands for and the
/// claim tier it may be asserted at.
#[derive(Debug, Clone, Serialize)]
pub struct AnchoredEvidence {
    /// The full 64-hex anchor hash — the citable handle the hallucination gate checks.
    pub anchor: String,
    pub episode_ref: String,
    pub dominant_motif: String,
    pub families: Vec<String>,
    pub consensus_strength: f64,
    pub disagreement_entropy: f64,
    pub peak_drift: f64,
    pub peak_slew: f64,
    pub witness_strength: String,
    pub evidence_kind: String,
    /// The claim tier this anchor may be asserted at (`SealedFact` / `EvidenceInterpretation` / … / `NonClaim`).
    pub claim_tier: String,
    /// The matched candidate label, or the deterministic unknown-taxonomy subtype.
    pub label: String,
}

/// The emitted narration context (schema v1), sealed by `context_root`.
#[derive(Debug, Clone, Serialize)]
pub struct NarrationContextV1 {
    pub format: &'static str,
    pub dataset: String,
    /// The dataset's `evidence_root` (= replay hash) this context pertains to.
    pub evidence_root: String,
    pub contract: Vec<&'static str>,
    pub non_claims: Vec<&'static str>,
    pub anchors: Vec<AnchoredEvidence>,
    /// SHA-256 (via [`CanonicalHasher`]) over the canonical context — deterministic + re-runnable.
    pub context_root: String,
}

/// Build the narration context for an analysed dataset. Pure + deterministic (no timestamps/paths). Reads exactly
/// what the operator report reads (`res.fused_episodes` + `res.heuristic_labels`) and reuses the shared anchor fn,
/// so the emitted anchors are byte-identical to the operator-report anchors for the same episodes.
pub fn build_narration_context(dataset: &str, res: &AnalysisResult) -> NarrationContextV1 {
    let mut anchors: Vec<AnchoredEvidence> = Vec::with_capacity(res.fused_episodes.len());
    for (i, e) in res.fused_episodes.iter().enumerate() {
        let l = res.heuristic_labels.get(i);
        let matched = l.map(|l| l.matched).unwrap_or(false);
        let label = l
            .map(|l| {
                if l.matched {
                    l.episode_label.clone()
                } else {
                    l.unknown_subtype
                        .clone()
                        .unwrap_or_else(|| "UNKNOWN".into())
                }
            })
            .unwrap_or_else(|| "UNKNOWN".into());
        let witness = episode_witness_strength(e, matched);
        let kind = EvidenceKind::from_witness_strength(witness);
        let evidence_kind = kind.tag();
        let claim_tier = kind.claim_strength().tag();
        let anchor = episode_evidence_anchor(dataset, e, witness, evidence_kind, &label);
        anchors.push(AnchoredEvidence {
            anchor,
            episode_ref: format!("{}-{}", e.start_index, e.end_index),
            dominant_motif: e.dominant_motif.token().to_string(),
            families: e.families.clone(),
            consensus_strength: e.consensus_strength,
            disagreement_entropy: e.disagreement_entropy,
            peak_drift: e.peak_drift,
            peak_slew: e.peak_slew,
            witness_strength: witness.tag().to_string(),
            evidence_kind: evidence_kind.to_string(),
            claim_tier: claim_tier.to_string(),
            label,
        });
    }
    let mut ctx = NarrationContextV1 {
        format: NARRATION_CONTEXT_FORMAT,
        dataset: dataset.to_string(),
        evidence_root: res.replay_hash.clone(),
        contract: CONTRACT.to_vec(),
        non_claims: NON_CLAIMS.to_vec(),
        anchors,
        context_root: String::new(),
    };
    ctx.context_root = seal(&ctx);
    ctx
}

/// Build a narration context directly from explicit anchors — an **adversarial / synthetic** constructor for the
/// narration-failure demonstrator, letting it choose each anchor's claim tier / label so every H7–H12 failure can be
/// exhibited deterministically. Carries the same binding contract + non-claims and seals identically to
/// [`build_narration_context`]; it is NOT a path the production emitter uses (compiled only behind the
/// `narration-heuristic-demo` feature).
#[cfg(feature = "narration-heuristic-demo")]
pub fn narration_context_from_anchors(
    dataset: &str,
    evidence_root: &str,
    anchors: Vec<AnchoredEvidence>,
) -> NarrationContextV1 {
    let mut ctx = NarrationContextV1 {
        format: NARRATION_CONTEXT_FORMAT,
        dataset: dataset.to_string(),
        evidence_root: evidence_root.to_string(),
        contract: CONTRACT.to_vec(),
        non_claims: NON_CLAIMS.to_vec(),
        anchors,
        context_root: String::new(),
    };
    ctx.context_root = seal(&ctx);
    ctx
}

/// Seal the canonical context — fixed field order, every collection already ordered → re-runnable byte-for-byte.
fn seal(c: &NarrationContextV1) -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", NARRATION_CONTEXT_FORMAT.as_bytes());
    h.field("dataset", c.dataset.as_bytes());
    h.field("evidence_root", c.evidence_root.as_bytes());
    for line in &c.contract {
        h.field("contract", line.as_bytes());
    }
    for nc in &c.non_claims {
        h.field("non_claim", nc.as_bytes());
    }
    for a in &c.anchors {
        h.field("anchor", a.anchor.as_bytes());
        h.field("episode_ref", a.episode_ref.as_bytes());
        h.field("motif", a.dominant_motif.as_bytes());
        h.field("families", a.families.join(",").as_bytes());
        h.f64q("consensus", a.consensus_strength);
        h.f64q("entropy", a.disagreement_entropy);
        h.f64q("peak_drift", a.peak_drift);
        h.f64q("peak_slew", a.peak_slew);
        h.field("witness", a.witness_strength.as_bytes());
        h.field("evidence_kind", a.evidence_kind.as_bytes());
        h.field("claim_tier", a.claim_tier.as_bytes());
        h.field("label", a.label.as_bytes());
    }
    h.finalize_hex()
}

/// The set of full anchor hashes a narrator may cite — exactly what `NoNarrativeHallucinationGateV1::check`
/// validates each sentence against.
pub fn known_anchor_set(c: &NarrationContextV1) -> BTreeSet<String> {
    c.anchors.iter().map(|a| a.anchor.clone()).collect()
}

fn esc_md(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Render the human-readable narration-context document (Markdown). Self-contained; no consumer named.
pub fn render_markdown(c: &NarrationContextV1) -> String {
    let mut s = String::new();
    s.push_str(&format!("# Narration context — {}\n\n", c.dataset));
    s.push_str("> The complete, sealed evidence vocabulary a constrained narrator may re-present as prose, with the\n");
    s.push_str("> rules it must obey. A navigation/contract layer over an already-sealed Chemical Court Record — it\n");
    s.push_str("> creates no evidence and asserts nothing beyond what the record sealed.\n\n");
    s.push_str(&format!("- evidence_root: `{}`\n", c.evidence_root));
    s.push_str(&format!("- context_root: `{}`\n", c.context_root));
    s.push_str(&format!("- citable anchors: {}\n\n", c.anchors.len()));

    s.push_str("## Binding contract\n\n");
    for line in &c.contract {
        s.push_str(&format!("- {line}\n"));
    }
    s.push('\n');

    s.push_str("## Citable evidence anchors\n\n");
    s.push_str("Each row is one fused episode. A sentence may cite an anchor only, and may assert no more than its claim tier.\n\n");
    s.push_str("| Anchor (full hash) | Episode | Motif | Witness strength | Evidence kind | Claim tier | Candidate label / unknown |\n");
    s.push_str("|---|---|---|---|---|---|---|\n");
    for a in &c.anchors {
        s.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |\n",
            a.anchor,
            esc_md(&a.episode_ref),
            esc_md(&a.dominant_motif),
            esc_md(&a.witness_strength),
            esc_md(&a.evidence_kind),
            esc_md(&a.claim_tier),
            esc_md(&a.label),
        ));
    }
    s.push('\n');

    s.push_str("## Non-claims (never asserted)\n\n");
    for nc in &c.non_claims {
        s.push_str(&format!("- {nc}\n"));
    }
    s.push_str("\nValidation: every emitted sentence is checked by `NoNarrativeHallucinationGateV1` against the anchor set above; an unanchored or over-tier sentence is rejected, never shipped.\n");
    s
}

/// Render the machine-readable context (JSON): the known-anchor set + per-anchor claim tier, for a programmatic gate.
pub fn render_json(c: &NarrationContextV1) -> String {
    serde_json::to_string_pretty(c).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::synthetic_suite;
    use crate::narrative::{NarrativeBuilder, NoNarrativeHallucinationGateV1};
    use crate::pipeline::{analyze, PipelineConfig};

    fn ctx_for(name: &str) -> NarrationContextV1 {
        let d = synthetic_suite()
            .into_iter()
            .find(|d| d.name == name)
            .expect("dataset in suite");
        let res = analyze(
            &d.name,
            &d.kind,
            &d.matrix,
            d.n_base,
            PipelineConfig::default(),
        );
        build_narration_context(&d.name, &res)
    }

    #[test]
    fn context_is_deterministic_and_populated() {
        let a = ctx_for("synth_wide_step");
        let b = ctx_for("synth_wide_step");
        assert_eq!(
            a.context_root, b.context_root,
            "context_root must be deterministic"
        );
        assert_eq!(a.context_root.len(), 64);
        assert!(
            !a.anchors.is_empty(),
            "synth_wide_step must produce citable anchors"
        );
        for an in &a.anchors {
            assert_eq!(an.anchor.len(), 64, "each anchor is a full 64-hex hash");
        }
    }

    /// The load-bearing test: a sentence anchored to a context anchor passes the hallucination gate; an
    /// unanchored sentence fails — proving the emitted anchor set IS the gate's permitted vocabulary.
    #[test]
    fn gate_accepts_an_anchored_sentence_and_rejects_an_unanchored_one() {
        let ctx = ctx_for("synth_wide_step");
        let known = known_anchor_set(&ctx);
        let a0 = &ctx.anchors[0];

        let mut b = NarrativeBuilder::new(&ctx.dataset);
        b.episode(&a0.episode_ref, &a0.dominant_motif, a0.anchor.clone());
        let narr = b.seal();
        let gate = NoNarrativeHallucinationGateV1::check(&narr, &known);
        assert!(
            gate.all_anchored && gate.n_anchored == 1 && gate.unanchored.is_empty(),
            "a sentence citing a context anchor must pass the gate"
        );

        let mut b2 = NarrativeBuilder::new(&ctx.dataset);
        b2.episode("0-1", "DriftAccum", "f".repeat(64)); // an anchor NOT in the context set
        let narr2 = b2.seal();
        let gate2 = NoNarrativeHallucinationGateV1::check(&narr2, &known);
        assert!(
            !gate2.all_anchored && gate2.unanchored.len() == 1,
            "a sentence citing an anchor outside the context set must be rejected"
        );
    }

    /// The committed `reports/narration_context_sample.{md,json}` must equal a fresh render — so a format
    /// change can't go stale silently. Regenerate via `narration-context synth_wide_step` then copy the two
    /// output files into `reports/` (the index.html-determinism discipline; the A1 stale-artifact lesson).
    #[test]
    fn committed_sample_is_current() {
        let ctx = ctx_for("synth_wide_step");
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let md = std::fs::read_to_string(repo.join("reports/narration_context_sample.md"))
            .expect("sample md present");
        let json = std::fs::read_to_string(repo.join("reports/narration_context_sample.json"))
            .expect("sample json present");
        assert_eq!(
            md,
            render_markdown(&ctx),
            "reports/narration_context_sample.md is stale — regenerate it"
        );
        assert_eq!(
            json,
            render_json(&ctx),
            "reports/narration_context_sample.json is stale — regenerate it"
        );
    }

    /// The emitted document (md + json) names no consumer: zero "LLM" / "AI" / "language model".
    #[test]
    fn emitted_document_names_no_consumer() {
        let ctx = ctx_for("synth_wide_step");
        for doc in [render_markdown(&ctx), render_json(&ctx)] {
            let low = doc.to_lowercase();
            assert!(!low.contains("llm"), "must not name LLM");
            assert!(
                !low.contains("language model"),
                "must not name a language model"
            );
            // match " ai " / "ai-" style tokens, not substrings like "obtain"/"domain"
            assert!(
                !low.split(|c: char| !c.is_ascii_alphabetic())
                    .any(|w| w == "ai"),
                "must not name AI"
            );
        }
    }
}
