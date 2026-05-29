//! H7–H12 narration-failure heuristic bank + a deterministic detector (P-panel narrator-generator extension).
//!
//! The project's narration layer (`crate::narrative::ProcessNarrativeCompilerV1` +
//! `NoNarrativeHallucinationGateV1` + `crate::narration_context::NarrationContextV1`) lets a constrained narrator
//! re-present a sealed Chemical Court Record as prose under a 6-rule binding contract (cite one anchor per sentence;
//! never upgrade a claim tier, assert a root cause, relabel an `unknown`, or infer across episodes). The hallucination
//! gate already *mechanically* enforces ONE of those rules — every sentence must cite a known anchor (the failure we
//! catalogue here as **H8**). This module applies the same DSFB residual-semiotics / evidence-court discipline to the
//! REMAINING contract rules: it catalogues the narration-failure heuristics **H7–H12** (the analogue, for the narrator,
//! of the chemical process-heuristic bank H1–H6 in the atlas) and ships a deterministic [`detect`] that mechanically
//! flags each failure in a produced narrative. This turns the constrained-narration contract from *stated* into
//! *enforced*.
//!
//! Scope discipline: this monitors THIS project's own narration generator. It names no consumer (no "LLM"/"AI"/
//! "language model" — a test pins that), creates no evidence, and is OFF the replay path (a pure structural check over
//! already-sealed objects); its own catalogue carries [`narration_heuristics_hash_v1`], separate from the atlas hash.
//! Like the chemical bank, every heuristic honestly documents its known false-positive and false-negative modes.

use crate::hashing::CanonicalHasher;
use crate::narration_context::{known_anchor_set, AnchoredEvidence, NarrationContextV1};
use crate::narrative::ProcessNarrativeCompilerV1;
use std::collections::BTreeMap;

/// One catalogued narration-failure heuristic (schema v1) — mirrors the shape + honesty fields of the chemical
/// `ChemicalProcessHeuristicRecordV1`, but for failures of the narrator rather than faults of the plant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NarrationHeuristicRecordV1 {
    /// `"H7"`..=`"H12"`.
    pub heuristic_id: &'static str,
    pub name: &'static str,
    /// The narration mistake this heuristic names (what a hallucinating/over-reaching narrator would do).
    pub narration_failure_pattern: &'static str,
    /// The deterministic, structural condition [`detect`] uses to flag it (no NLP, no model).
    pub detection_condition: &'static str,
    /// Which binding-contract rule / standing non-claim it enforces (see `narration_context::CONTRACT`).
    pub contract_basis: &'static str,
    pub severity: &'static str,
    pub known_false_positive_modes: &'static [&'static str],
    pub known_false_negative_modes: &'static [&'static str],
    pub basis: &'static str,
}

/// The H7–H12 narration-failure heuristic bank (in id order). Catalogue authority — a `const` table, like the
/// chemical `HEURISTIC_RECORDS`.
pub const NARRATION_HEURISTIC_RECORDS: [NarrationHeuristicRecordV1; 6] = [
    NarrationHeuristicRecordV1 {
        heuristic_id: "H7",
        name: "Claim-tier breach",
        narration_failure_pattern: "a sentence asserts a stronger certainty than its cited anchor's claim tier permits (e.g. states a Tier-2 interpretation as a confirmed fact)",
        detection_condition: "the cited anchor's claim_tier is below SealedFact AND the sentence text carries a definitive-certainty marker (confirmed / definitively / certain)",
        contract_basis: "contract rule 5: a sentence may not assert more than its anchor's claim tier (SealedFact > EvidenceInterpretation > SpeculativeImplication; NonClaim never)",
        severity: "high — silently upgrades an advisory candidate into an asserted fact",
        known_false_positive_modes: &[
            "a sentence that quotes the word 'confirmed' as part of a verbatim anchor label rather than asserting it",
            "a SealedFact anchor (Tier 1) legitimately stated definitively is NOT flagged (correct), but a marker list can miss synonymous certainty phrasing",
        ],
        known_false_negative_modes: &[
            "over-assertion phrased without a lexical certainty marker (implicit confidence) is not caught — the gate is lexical, not semantic",
        ],
        basis: "the claim-strength tier ladder (EvidenceKind -> ClaimStrength) that the operator report and narration context already carry per anchor",
    },
    NarrationHeuristicRecordV1 {
        heuristic_id: "H8",
        name: "Unanchored sentence",
        narration_failure_pattern: "a sentence cites no sealed evidence object, or cites one outside the record's anchor set (a free-floating assertion / hallucination)",
        detection_condition: "the sentence's anchor_hash is not in known_anchor_set(context) — exactly the existing NoNarrativeHallucinationGateV1 rule, catalogued",
        contract_basis: "contract rules 2 & 4: every sentence must cite exactly one anchor from the set; the gate rejects any that does not",
        severity: "critical — an unanchored sentence is unbacked by the sealed record",
        known_false_positive_modes: &[
            "none structurally — anchor-set membership is exact; a mismatch is always a real out-of-record citation",
        ],
        known_false_negative_modes: &[
            "a sentence that cites a VALID anchor but mis-states what that anchor stands for is anchored (passes H8) and must be caught by H7/H9/H10/H11 instead",
        ],
        basis: "NoNarrativeHallucinationGateV1::check (crate::narrative) — the load-bearing anchor-membership gate",
    },
    NarrationHeuristicRecordV1 {
        heuristic_id: "H9",
        name: "Forbidden-claim phrasing",
        narration_failure_pattern: "a sentence asserts one of the standing non-claims — a physical root cause / causal link, accuracy-or-speed superiority, control/safety authority, or regulatory compliance",
        detection_condition: "the sentence text contains a forbidden non-claim phrase (caused by / is the root cause / proven / accuracy superior / control action authority / regulatory compliance)",
        contract_basis: "contract rule 3 + the standing non_claims list (never asserted, by narrator or court)",
        severity: "critical — asserts a claim the record explicitly never makes",
        known_false_positive_modes: &[
            "a sentence that explicitly DISCLAIMS a non-claim (e.g. 'not the root cause') is written so the disclaimer phrasing is distinct from the asserting phrasing the markers target",
        ],
        known_false_negative_modes: &[
            "a forbidden claim paraphrased outside the marker list (e.g. an unusual synonym for causation) is not caught — lexical, not semantic",
        ],
        basis: "the standing non_claims vocabulary in crate::narration_context (NON_CLAIMS) + non_claims.md",
    },
    NarrationHeuristicRecordV1 {
        heuristic_id: "H10",
        name: "Unknown-relabeling",
        narration_failure_pattern: "a sentence assigns a named fault to an episode the record deliberately left UNKNOWN (forcing an unclassified episode into a diagnosis)",
        detection_condition: "the cited anchor's label is UNKNOWN AND the sentence names a fault (contains 'fault') without preserving the unknown disposition (no 'unknown')",
        contract_basis: "contract rule 3: may not relabel an `unknown` episode as a named fault",
        severity: "high — fabricates a classification the record withheld on purpose",
        known_false_positive_modes: &[
            "a sentence about an UNKNOWN episode that legitimately discusses a 'fault' in the abstract while still naming the episode unknown is written to retain the 'unknown' token",
        ],
        known_false_negative_modes: &[
            "relabeling that avoids the word 'fault' (names a specific mechanism directly) is not caught by the lexical marker",
        ],
        basis: "the deterministic unknown-taxonomy disposition the record seals per episode (anchor.label == UNKNOWN)",
    },
    NarrationHeuristicRecordV1 {
        heuristic_id: "H11",
        name: "Cross-episode inference",
        narration_failure_pattern: "a sentence asserts a relationship or causation BETWEEN episodes (propagation, one episode causing another) that the sealed record does not contain",
        detection_condition: "the sentence text contains a cross-episode connective (propagated to episode / led to episode / caused episode)",
        contract_basis: "contract rule 3: may not merge/split/infer episodes beyond the record",
        severity: "high — invents inter-episode structure the record never sealed",
        known_false_positive_modes: &[
            "the propagation FIGURE in the paper is a temporal-precedence + topology candidate, not a causal claim; narrator prose must not restate it causally — a legitimately hedged mention could match the connective and be flagged for review (conservative)",
        ],
        known_false_negative_modes: &[
            "an inter-episode inference phrased without one of the catalogued connectives is not caught",
        ],
        basis: "episodes are sealed independently; the record asserts no inter-episode causality (only the hedged propagation-candidate figure)",
    },
    NarrationHeuristicRecordV1 {
        heuristic_id: "H12",
        name: "Anchor-coverage drift",
        narration_failure_pattern: "the narrative's coverage of the anchor set drifts — the same anchor is cited by more than one sentence (duplicated emphasis that can over-weight one episode)",
        detection_condition: "some anchor_hash appears in more than one sentence of the narrative",
        contract_basis: "contract rules 1 & 6: the narrator re-presents the record faithfully; coverage must track the sealed anchors, not amplify a subset",
        severity: "medium — a faithfulness/coverage drift rather than a false assertion",
        known_false_positive_modes: &[
            "a narrative that deliberately cites one anchor twice for two genuinely distinct facts is flagged (conservative); the demonstrator treats one anchor = one sentence as the faithful baseline",
        ],
        known_false_negative_modes: &[
            "under-coverage (a sealed anchor that no sentence mentions) is NOT flagged here — omission is a separate, lower-severity concern left to review",
        ],
        basis: "residual-semiotics coverage discipline applied to narration: the sentence->anchor map should not over-concentrate",
    },
];

// Lexical markers (lowercase) used by the structural detector. Chosen to be DISJOINT from the fixed narrator
// templates (`crate::narrative::NarrativeBuilder`) so a faithful narrative never trips them: the safe templates say
// "the dominant structural motif is …", "Candidate label (advisory, not root cause): …", "Detector X testified as a Y
// witness.", "Ruled out (confuser considered): …", "Balance witness (…): peak closure residual …".
const H7_CERTAINTY_MARKERS: [&str; 4] =
    ["confirmed", "definitively", "is certain", "with certainty"];
const H9_FORBIDDEN_MARKERS: [&str; 6] = [
    "caused by",
    "is the root cause",
    "proven",
    "accuracy superior",
    "control action authority",
    "regulatory compliance",
];
const H11_CROSS_EPISODE_MARKERS: [&str; 3] =
    ["propagated to episode", "led to episode", "caused episode"];

/// SHA-256 over the H7–H12 narration heuristic bank (fixed field order, id order) — the catalogue's own seal,
/// independent of the atlas. Deterministic + re-runnable; pinned by a frozen-hex test.
pub fn narration_heuristics_hash_v1() -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"narration_heuristics_v1");
    for r in &NARRATION_HEURISTIC_RECORDS {
        h.field("heuristic_id", r.heuristic_id.as_bytes());
        h.field("name", r.name.as_bytes());
        h.field(
            "narration_failure_pattern",
            r.narration_failure_pattern.as_bytes(),
        );
        h.field("detection_condition", r.detection_condition.as_bytes());
        h.field("contract_basis", r.contract_basis.as_bytes());
        h.field("severity", r.severity.as_bytes());
        for m in r.known_false_positive_modes {
            h.field("fp", m.as_bytes());
        }
        for m in r.known_false_negative_modes {
            h.field("fn", m.as_bytes());
        }
        h.field("basis", r.basis.as_bytes());
    }
    h.finalize_hex()
}

/// Gate the bank: H7–H12 each present exactly once; no honesty field empty. Mirrors `atlas::validation::validate`.
pub fn validate() -> Result<(), String> {
    for id in ["H7", "H8", "H9", "H10", "H11", "H12"] {
        let n = NARRATION_HEURISTIC_RECORDS
            .iter()
            .filter(|r| r.heuristic_id == id)
            .count();
        if n != 1 {
            return Err(format!(
                "{id} must appear exactly once in the narration bank, found {n}"
            ));
        }
    }
    for r in &NARRATION_HEURISTIC_RECORDS {
        if r.detection_condition.is_empty()
            || r.contract_basis.is_empty()
            || r.basis.is_empty()
            || r.known_false_positive_modes.is_empty()
            || r.known_false_negative_modes.is_empty()
        {
            return Err(format!(
                "{} has an empty required/honesty field",
                r.heuristic_id
            ));
        }
    }
    Ok(())
}

/// One flagged narration failure: which heuristic fired, on which sentence, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrationFailureHit {
    pub heuristic_id: String,
    pub sentence_index: usize,
    pub detail: String,
}

/// The result of running the H7–H12 detector over a narrative against its narration context (schema v1).
/// `clean` iff `hits` is empty. Deterministically sealed by `detector_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrationFailureDetectorV1 {
    pub narrative_hash: String,
    pub context_root: String,
    pub n_sentences: usize,
    pub clean: bool,
    pub hits: Vec<NarrationFailureHit>,
    pub detector_hash: String,
}

fn hit(id: &str, i: usize, detail: impl Into<String>) -> NarrationFailureHit {
    NarrationFailureHit {
        heuristic_id: id.to_string(),
        sentence_index: i,
        detail: detail.into(),
    }
}

/// Run the H7–H12 narration-failure detector. Pure + deterministic: for each sentence it applies the catalogued
/// structural conditions against the narration context's sealed anchors. H8 is the existing anchor-membership gate;
/// H7/H9/H10/H11 are lexical+anchor checks; H12 is a whole-narrative coverage check. Off the replay path.
pub fn detect(
    narrative: &ProcessNarrativeCompilerV1,
    ctx: &NarrationContextV1,
) -> NarrationFailureDetectorV1 {
    let known = known_anchor_set(ctx);
    let by_anchor: BTreeMap<&str, &AnchoredEvidence> =
        ctx.anchors.iter().map(|a| (a.anchor.as_str(), a)).collect();

    // H12 coverage: how many sentences cite each anchor.
    let mut anchor_count: BTreeMap<&str, usize> = BTreeMap::new();
    for s in &narrative.sentences {
        *anchor_count.entry(s.anchor_hash.as_str()).or_insert(0) += 1;
    }

    let mut hits: Vec<NarrationFailureHit> = Vec::new();
    for (i, s) in narrative.sentences.iter().enumerate() {
        let low = s.text.to_lowercase();

        // H8 — unanchored: anchor not in the record's set. (Anchor-dependent checks are skipped for it.)
        if !known.contains(&s.anchor_hash) {
            hits.push(hit(
                "H8",
                i,
                "cited anchor is not in the record's anchor set",
            ));
            continue;
        }
        let ev = by_anchor.get(s.anchor_hash.as_str()).copied();

        // H7 — claim-tier breach: definitive certainty over a sub-SealedFact anchor.
        if let Some(ev) = ev {
            if ev.claim_tier != "SealedFact" && H7_CERTAINTY_MARKERS.iter().any(|m| low.contains(m))
            {
                hits.push(hit(
                    "H7",
                    i,
                    format!("definitive assertion over a {} anchor", ev.claim_tier),
                ));
            }
            // H10 — unknown-relabeling: names a fault for an UNKNOWN-label anchor without keeping it unknown.
            if ev.label.eq_ignore_ascii_case("UNKNOWN")
                && low.contains("fault")
                && !low.contains("unknown")
            {
                hits.push(hit(
                    "H10",
                    i,
                    "names a fault for an episode the record marks UNKNOWN",
                ));
            }
        }

        // H9 — forbidden non-claim phrasing.
        if let Some(m) = H9_FORBIDDEN_MARKERS.iter().find(|m| low.contains(**m)) {
            hits.push(hit("H9", i, format!("forbidden non-claim phrase: \"{m}\"")));
        }

        // H11 — cross-episode inference.
        if let Some(m) = H11_CROSS_EPISODE_MARKERS.iter().find(|m| low.contains(**m)) {
            hits.push(hit("H11", i, format!("cross-episode inference: \"{m}\"")));
        }

        // H12 — coverage drift: this anchor is cited by more than one sentence.
        if anchor_count
            .get(s.anchor_hash.as_str())
            .copied()
            .unwrap_or(0)
            > 1
        {
            hits.push(hit(
                "H12",
                i,
                "anchor cited by more than one sentence (coverage drift)",
            ));
        }
    }

    let detector_hash = seal_detector(&narrative.narrative_hash, &ctx.context_root, &hits);
    NarrationFailureDetectorV1 {
        narrative_hash: narrative.narrative_hash.clone(),
        context_root: ctx.context_root.clone(),
        n_sentences: narrative.sentences.len(),
        clean: hits.is_empty(),
        hits,
        detector_hash,
    }
}

fn seal_detector(narrative_hash: &str, context_root: &str, hits: &[NarrationFailureHit]) -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"narration_failure_detector_v1");
    h.field("narrative_hash", narrative_hash.as_bytes());
    h.field("context_root", context_root.as_bytes());
    for hit in hits {
        h.field("heuristic_id", hit.heuristic_id.as_bytes());
        h.u64("sentence_index", hit.sentence_index as u64);
        h.field("detail", hit.detail.as_bytes());
    }
    h.finalize_hex()
}

impl NarrationFailureDetectorV1 {
    /// Re-derive the seal — the detector report is itself byte-verifiable.
    pub fn verify(&self) -> bool {
        seal_detector(&self.narrative_hash, &self.context_root, &self.hits) == self.detector_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narrative::NarrativeBuilder;

    // A faithful narrative built only from the fixed templates, over a real context, trips no heuristic.
    fn ctx_synth() -> NarrationContextV1 {
        use crate::pipeline::{analyze, PipelineConfig};
        let d = crate::cli::synthetic_suite()
            .into_iter()
            .find(|d| d.name == "synth_wide_step")
            .expect("suite");
        let res = analyze(
            &d.name,
            &d.kind,
            &d.matrix,
            d.n_base,
            PipelineConfig::default(),
        );
        crate::narration_context::build_narration_context(&d.name, &res)
    }

    #[test]
    fn bank_validates_and_hash_is_deterministic() {
        validate().expect("H7-H12 bank must validate");
        assert_eq!(NARRATION_HEURISTIC_RECORDS.len(), 6);
        let a = narration_heuristics_hash_v1();
        let b = narration_heuristics_hash_v1();
        assert_eq!(a, b, "narration_heuristics_hash_v1 must be deterministic");
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn faithful_template_narrative_is_clean() {
        let ctx = ctx_synth();
        assert!(!ctx.anchors.is_empty());
        // One sentence per anchor, via the fixed safe templates → must trip no H7–H12.
        let mut b = NarrativeBuilder::new(&ctx.dataset);
        for a in &ctx.anchors {
            b.episode(&a.episode_ref, &a.dominant_motif, a.anchor.clone());
        }
        let narr = b.seal();
        let report = detect(&narr, &ctx);
        assert!(
            report.clean,
            "a faithful template narrative must be clean, got hits: {:?}",
            report.hits
        );
        assert!(report.verify());
    }

    #[test]
    fn the_no_consumer_rule_holds_for_the_catalogue() {
        // The catalogue itself names no consumer.
        for r in &NARRATION_HEURISTIC_RECORDS {
            for field in [
                r.name,
                r.narration_failure_pattern,
                r.detection_condition,
                r.contract_basis,
                r.basis,
            ] {
                let low = field.to_lowercase();
                assert!(!low.contains("llm") && !low.contains("language model"));
                assert!(!low
                    .split(|c: char| !c.is_ascii_alphabetic())
                    .any(|w| w == "ai"));
            }
        }
    }
}
