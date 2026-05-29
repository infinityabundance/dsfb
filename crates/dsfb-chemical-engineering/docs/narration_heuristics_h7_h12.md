# Narration-failure heuristics H7–H12 (narrator-generator extension)

The chemical process-heuristic bank **H1–H6** (`crates/dsfb-chemical-engineering-atlas/src/heuristics.rs`) catalogues
how a *plant* fails. This bank catalogues how the project's own **narration generator** fails — the residual-semiotics
/ evidence-court discipline turned on the narrator itself.

The narration layer lets a constrained narrator re-present a sealed Chemical Court Record as prose under a 6-rule
binding contract (cite one anchor per sentence; never upgrade a claim tier, assert a root cause / causal link, relabel
an `unknown`, infer across episodes, or alter the record — see `docs/constrained_narration_extension.md` and
`crate::narration_context`). The hallucination gate (`NoNarrativeHallucinationGateV1`) already *mechanically* enforces
one rule — every sentence must cite a known anchor (here **H8**). H7–H12 catalogue the remaining failure modes and a
deterministic detector (`crate::narration_heuristics::detect`) flags each, so the contract is **enforced, not merely
stated**. The bank is sealed by its own `narration_heuristics_hash_v1` (separate from `atlas_hash_v1`) and a
feature-gated synthetic demonstrator (`--features narration-heuristic-demo`, subcommand `narration-heuristic-demo`)
exhibits the detector catching one deliberately-malformed narrative per heuristic while a faithful template narrative
trips none. It names no consumer, creates no evidence, and is off the replay path.

The detector is **lexical + structural, not semantic** — its markers are chosen to be disjoint from the narrator's
fixed safe templates, so a faithful narrative never trips them. Each heuristic honestly lists its known
false-positive and false-negative modes (the markers can miss a paraphrase, and conservatively flag a hedged mention).

| ID | Name | Narration failure it catches | Detection condition (deterministic) | Contract basis |
|----|------|------------------------------|-------------------------------------|----------------|
| **H7** | Claim-tier breach | A sentence asserts stronger certainty than its cited anchor's claim tier permits (a Tier-2 interpretation stated as a confirmed fact). | Anchor `claim_tier` is below `SealedFact` **and** the text carries a definitive-certainty marker (`confirmed` / `definitively` / `certain`). | Rule 5: no assertion above the anchor's tier (SealedFact > EvidenceInterpretation > SpeculativeImplication; NonClaim never). |
| **H8** | Unanchored sentence | A sentence cites no sealed evidence object, or one outside the record (a free-floating assertion / hallucination). | The sentence's `anchor_hash` is not in `known_anchor_set(context)` — the existing `NoNarrativeHallucinationGateV1` rule, catalogued. | Rules 2 & 4: every sentence cites one anchor from the set; the gate rejects any that does not. |
| **H9** | Forbidden-claim phrasing | A sentence asserts a standing non-claim — physical root cause / causal link, accuracy-or-speed superiority, control/safety authority, or regulatory compliance. | The text contains a forbidden phrase (`caused by` / `is the root cause` / `proven` / `accuracy superior` / `control action authority` / `regulatory compliance`). | Rule 3 + the standing `non_claims` (never asserted). |
| **H10** | Unknown-relabeling | A sentence assigns a named fault to an episode the record deliberately left `UNKNOWN`. | The cited anchor's `label` is `UNKNOWN` **and** the text names a fault (`fault`) without preserving the unknown disposition (no `unknown`). | Rule 3: may not relabel an `unknown` as a named fault. |
| **H11** | Cross-episode inference | A sentence asserts a relationship/causation *between* episodes (propagation, one causing another) absent from the record. | The text contains a cross-episode connective (`propagated to episode` / `led to episode` / `caused episode`). | Rule 3: may not merge/split/infer episodes beyond the record. |
| **H12** | Anchor-coverage drift | The narrative's coverage of the anchor set drifts — one anchor cited by multiple sentences (over-weighting an episode). | Some `anchor_hash` appears in more than one sentence. | Rules 1 & 6: re-present faithfully; coverage tracks the sealed anchors, not a subset. |

## Reproduce

```sh
# Exhibit the detector catching one malformed narrative per H7–H12 (faithful baseline trips none):
cargo run --release -p dsfb-chemical-engineering-edge --features narration-heuristic-demo -- narration-heuristic-demo

# The catalogue + detector are always compiled and tested on a faithful narrative:
cargo test  -p dsfb-chemical-engineering-edge --lib narration_heuristics
# The adversarial audit (each case trips exactly its target) runs under the feature:
cargo test  -p dsfb-chemical-engineering-edge --features narration-heuristic-demo --test narration_heuristic_audit
```

## Honesty boundary

This monitors *this project's* narration generator — it is not a claim about any external text producer, and it names
none. It is a deterministic, lexical+structural gate: it catches the catalogued, marker-expressible failures and
honestly does not catch paraphrases outside its markers (each heuristic's false-negative modes say so). It adds a
governed, inspectable enforcement layer to the constrained-narration contract; it creates no evidence and changes no
sealed hash.
