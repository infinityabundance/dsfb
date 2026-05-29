# Constrained-narration extension contract

This document specifies the boundary for any **external narration consumer** — a downstream component that turns a
sealed DSFB Chemical Court Record into human-readable prose. It defines what such a consumer may and may not do.
DSFB remains the sole authority; the narrator is a constrained re-presenter of already-sealed evidence, never a
source of it.

> It is not a witness, not a judge, and not a detector — only a constrained narrator over already-sealed evidence,
> and every emitted sentence must cite an evidence anchor.

## 1. Purpose
Operators and reviewers sometimes want a plain-language summary of a case file. This contract lets a narration
consumer produce that summary **without contaminating the deterministic court**: the evidence, the claim strengths,
and the verdicts are fixed by DSFB before the narrator ever runs, and the narrator may only restate them.

## 2. Non-authority rule (the load-bearing constraint)
The narrator has **no authority**. It must not:
- create evidence (no new residuals, episodes, witnesses, or signatures);
- assign or upgrade a claim strength or an evidence kind;
- invent or assert a root cause, a causal link, or a diagnosis;
- alter, reorder, or re-weight anything in the sealed case file;
- bypass, soften, or re-interpret the `NoNarrativeHallucinationGateV1`.

Everything the narrator says must already be true *in the sealed record*. If it is not in the record, it may not be
said.

## 3. Evidence-anchor requirement
**Every emitted sentence must cite an evidence anchor** — a reference to a specific sealed item (an episode index, a
witness, a badge, a metric, a hash) in the case file. A sentence with no anchor is rejected. This makes each claim
traceable back to sealed evidence and makes hallucinated or embellished statements detectable by construction.

## 4. Allowed uses
- Summarising the sealed episodes, their badges, evidence kinds, and claim-strength tiers — in plain language.
- Restating the case file's non-claims and boundaries verbatim or faithfully paraphrased.
- Triage ordering (e.g. surfacing the strongest-witness episodes first) **as a presentation order only**, never as
  a re-ranking of the sealed claims.
- Producing an operator-facing draft that a human then reviews and signs off.

## 5. Forbidden uses
- Filling gaps with plausible-sounding but unsealed content.
- Converting an `unknown` episode into a named fault.
- Strengthening a candidate label into a confirmed one.
- Merging, splitting, or inferring episodes beyond what the record contains.
- Emitting any sentence that the gate (§7) cannot map to an evidence anchor.

## 6. Input contract
The narrator is given: (a) the sealed case file (manifest + the ten content files + the evidence/bundle roots), and
(b) an allowed-claim set — the list of sentences-with-anchors the record actually supports. Its task is to render a
subset of that allowed set into prose. It is never given raw plant time-series (those never leave the operator; see
the confidential-evaluation chain), and it is never asked to *decide* anything.

## 7. Output validation
Every draft passes through `NoNarrativeHallucinationGateV1` (the existing deterministic, non-generative gate in
`crates/dsfb-chemical-engineering-edge/src/narrative.rs`) before acceptance. The gate checks that each sentence maps
to a permitted evidence anchor and that no forbidden assertion (root cause, upgraded claim, invented episode) is
present. The outcome is **accepted / rejected / redlined**: a rejected or redlined draft is returned for correction,
never shipped. The gate, not the narrator, is the authority on what may be said.

## 8. Redaction boundary
When the case file is a redacted confidential bundle (`ConfidentialEvaluationBundleV1`), the narrator sees only the
redacted aliases and aggregate metrics — never real tag names or raw values. Narration therefore inherits the
no-data-egress property: the prose can be shared exactly when the bundle can.

## 9. Confidential-evaluation mode
In the confidential-evaluation chain, the narrator runs **inside the operator's boundary** over the local sealed
record, and only the validated, anchor-checked, redacted summary is shared — alongside the hash-linked bundle, never
instead of it. The evidence root and the verdicts remain the authoritative artifact; the narration is an attachment.

## 10. Future interactive operator-narration path
A future interactive mode could let an operator ask follow-up questions over a sealed case file. The same contract
holds unchanged: answers must cite evidence anchors, must pass the hallucination gate, and may not create evidence,
assign claim strength, or assert causation. The interaction is a lens over the sealed court — the court is never
re-opened by it.

## 11. The emitted narration context (executable)
The context this contract describes is **emitted automatically** from a sealed Court Record, so the narrator is
handed exactly what it may say about, up front:

- `dsfb-chem-edge narration-context <dataset>` writes `narration_context.{md,json}`, and a `casefile` run
  auto-emits the same next to its bundle. It is a separate standalone artifact — not one of the ten hashed
  CONTENT_FILES — so it moves no `bundle_root`.
- The document lists the **binding contract** above, the **non-claims**, and the complete vocabulary of **citable
  evidence anchors** — one per fused episode, each with its episode reference, dominant motif, detector families,
  consensus/entropy, witness rung, evidence kind, and the **claim tier** that anchor may be asserted at — sealed by
  a deterministic `context_root`. The anchors are the SAME `report::episode_evidence_anchor` digests the operator
  report shows, so a sentence citing either is checkable against one anchor.
- Each anchor is exactly an entry in the set `NoNarrativeHallucinationGateV1::check` validates against: a sentence
  whose anchor is not in the set, or that asserts above its anchor's claim tier, is rejected. A committed reference
  is at `reports/narration_context_sample.md` (regenerated + byte-checked by a test so it cannot drift).

## Mechanical enforcement: the H7–H12 narration-failure bank

The 6-rule contract above is not only stated — it is **mechanically enforceable**. The hallucination gate already
enforces one rule (every sentence must cite a known anchor). The narration-failure heuristic bank **H7–H12**
(`crate::narration_heuristics`, catalogued in `docs/narration_heuristics_h7_h12.md`) catalogues the rest as named,
honest failure modes and ships a deterministic detector (`detect`) that flags each:

- **H7** claim-tier breach · **H8** unanchored sentence (the existing gate) · **H9** forbidden-claim phrasing ·
  **H10** unknown-relabeling · **H11** cross-episode inference · **H12** anchor-coverage drift.

The bank carries its own seal `narration_heuristics_hash_v1` (separate from `atlas_hash_v1`). A feature-gated
synthetic demonstrator (`--features narration-heuristic-demo`, subcommand `narration-heuristic-demo`) exhibits the
detector catching one deliberately-malformed narrative per heuristic while a faithful template narrative trips none —
proving each rule is enforced, not merely asserted. The bank + detector are always compiled and off the replay path;
the demonstrator and its adversarial constructors are behind the feature, so the production narrator keeps its
no-free-text path. The whole layer names no consumer.

---
**Summary.** The court is deterministic and sealed first; any narrator is a constrained, gated, anchor-citing
re-presenter second. Substrate and structure stay with DSFB; the narrator only narrates what is already proven and
preserved.
