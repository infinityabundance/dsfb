//! Synthetic narration-failure demonstrator (panel narrator-generator extension) — behind the
//! `narration-heuristic-demo` feature; off the default build + replay path.
//!
//! The H7–H12 bank + detector (`crate::narration_heuristics`) are always compiled and tested on a faithful narrative.
//! This module *exhibits* the detector: it builds the deliberately-malformed narratives an **untrusted** narrator
//! might emit — one per catalogued failure — and shows the detector flags exactly that failure, while the faithful
//! baseline (built from the safe fixed templates) trips nothing. This proves the constrained-narration contract is
//! mechanically enforceable, not merely stated. It is synthetic, deterministic, sealed, names no consumer, creates no
//! evidence, and touches no frozen hash. The adversarial narratives are constructed via the feature-gated
//! `ProcessNarrativeCompilerV1::from_sentences` (the production narrator has no such free-text path).

use crate::narration_context::{
    narration_context_from_anchors, AnchoredEvidence, NarrationContextV1,
};
use crate::narration_heuristics::detect;
use crate::narrative::{NarrativeSentence, ProcessNarrativeCompilerV1};

fn rep(c: char) -> String {
    c.to_string().repeat(64)
}

/// A synthetic citable anchor with a chosen claim tier + label (other fields are fixed, deterministic placeholders).
fn syn_anchor(hash: &str, episode_ref: &str, label: &str, claim_tier: &str) -> AnchoredEvidence {
    AnchoredEvidence {
        anchor: hash.to_string(),
        episode_ref: episode_ref.to_string(),
        dominant_motif: "DriftAccum".to_string(),
        families: vec!["trend".to_string()],
        consensus_strength: 0.5,
        disagreement_entropy: 0.1,
        peak_drift: 1.0,
        peak_slew: 0.2,
        witness_strength: "DetectorFamilyQuorum".to_string(),
        evidence_kind: "chemometric_detector".to_string(),
        claim_tier: claim_tier.to_string(),
        label: label.to_string(),
    }
}

/// One demonstrator case: a narrative + its narration context, built either faithfully or to trip one heuristic.
pub struct DemoCase {
    pub name: &'static str,
    /// The heuristic this case is built to trip (`None` = the faithful baseline, which must trip none).
    pub injected: Option<&'static str>,
    pub narrative: ProcessNarrativeCompilerV1,
    pub ctx: NarrationContextV1,
}

/// The faithful baseline + one adversarial narrative per H7–H12, each built to trip exactly its target heuristic.
pub fn demo_cases() -> Vec<DemoCase> {
    let (a0, a1, a2) = (rep('a'), rep('b'), rep('c'));
    let outside = rep('f'); // a citation NOT in the context anchor set (for H8)
    let ctx = narration_context_from_anchors(
        "synth_narration_demo",
        &rep('e'),
        vec![
            // A0: an interpretation-tier (Tier-2) episode with a candidate label.
            syn_anchor(
                &a0,
                "10-20",
                "reactor thermal excursion candidate",
                "EvidenceInterpretation",
            ),
            // A1: a Tier-2 episode the record left UNKNOWN.
            syn_anchor(&a1, "30-40", "UNKNOWN", "EvidenceInterpretation"),
            // A2: a SealedFact-tier (Tier-1) anchor — definitive assertion here is legitimate (must NOT trip H7).
            syn_anchor(&a2, "50-60", "historian import receipt", "SealedFact"),
        ],
    );
    let nm = |text: &str, anchor: &str| NarrativeSentence {
        text: text.to_string(),
        evidence_kind: "episode".to_string(),
        anchor_hash: anchor.to_string(),
    };
    let narr = |sents: Vec<NarrativeSentence>| {
        ProcessNarrativeCompilerV1::from_sentences("synth_narration_demo", sents)
    };

    vec![
        DemoCase {
            name: "faithful baseline (safe templates)",
            injected: None,
            narrative: narr(vec![
                nm(
                    "Episode 10-20: the dominant structural motif is DriftAccum.",
                    &a0,
                ),
                nm(
                    "Episode 30-40: the dominant structural motif is BoundaryGrazing.",
                    &a1,
                ),
                nm(
                    "Episode 50-60: the dominant structural motif is EnvViolation.",
                    &a2,
                ),
            ]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H7 claim-tier breach (definitive over a Tier-2 anchor)",
            injected: Some("H7"),
            narrative: narr(vec![nm("Episode 10-20: the deviation is confirmed.", &a0)]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H8 unanchored sentence (citation outside the record)",
            injected: Some("H8"),
            narrative: narr(vec![nm(
                "Episode 99-100: the dominant structural motif is DriftAccum.",
                &outside,
            )]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H9 forbidden-claim phrasing (asserts causation)",
            injected: Some("H9"),
            narrative: narr(vec![nm(
                "Episode 10-20: the deviation was caused by a valve leak.",
                &a0,
            )]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H10 unknown-relabeling (names a fault for an UNKNOWN episode)",
            injected: Some("H10"),
            narrative: narr(vec![nm(
                "Episode 30-40 is a reactor thermal excursion fault.",
                &a1,
            )]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H11 cross-episode inference (asserts propagation)",
            injected: Some("H11"),
            narrative: narr(vec![nm("Episode 10-20 propagated to episode 30-40.", &a0)]),
            ctx: ctx.clone(),
        },
        DemoCase {
            name: "H12 anchor-coverage drift (one anchor cited twice)",
            injected: Some("H12"),
            narrative: narr(vec![
                nm(
                    "Episode 10-20: the dominant structural motif is DriftAccum.",
                    &a0,
                ),
                nm(
                    "Episode 10-20: the dominant structural motif is DriftAccum.",
                    &a0,
                ),
            ]),
            ctx: ctx.clone(),
        },
    ]
}

/// Render the demonstrator report (Markdown). Deterministic; names no consumer.
pub fn render_report() -> String {
    let mut s = String::new();
    s.push_str("# Narration-failure demonstrator (H7\u{2013}H12)\n\n");
    s.push_str("> Runs the deterministic H7\u{2013}H12 detector over deliberately-malformed narratives a constrained\n");
    s.push_str("> narrator must never emit, proving each binding-contract rule of the narration context is mechanically\n");
    s.push_str("> enforceable. Synthetic, sealed, off the replay path. The faithful baseline (safe templates) trips none.\n\n");
    s.push_str("| Case | Built to trip | Verdict | Heuristics fired | detector_hash |\n");
    s.push_str("|---|---|---|---|---|\n");
    for case in demo_cases() {
        let r = detect(&case.narrative, &case.ctx);
        let mut fired: Vec<String> = r.hits.iter().map(|h| h.heuristic_id.clone()).collect();
        fired.sort();
        fired.dedup();
        s.push_str(&format!(
            "| {} | {} | {} | {} | `{}\u{2026}` |\n",
            case.name,
            case.injected.unwrap_or("\u{2014} (baseline)"),
            if r.clean { "CLEAN" } else { "FLAGGED" },
            if fired.is_empty() {
                "\u{2014}".to_string()
            } else {
                fired.join(", ")
            },
            &r.detector_hash[..12],
        ));
    }
    s.push_str("\nEach malformed narrative trips exactly its target heuristic; the faithful baseline trips none. The\n");
    s.push_str("catalogue + detector live in `crate::narration_heuristics` (own seal `narration_heuristics_hash_v1`).\n");
    s
}

/// CLI entry (feature-gated): print the demonstrator report. Returns 0.
pub fn run() -> i32 {
    print!("{}", render_report());
    0
}
