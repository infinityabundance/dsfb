# Narration context — synth_wide_step

> The complete, sealed evidence vocabulary a constrained narrator may re-present as prose, with the
> rules it must obey. A navigation/contract layer over an already-sealed Chemical Court Record — it
> creates no evidence and asserts nothing beyond what the record sealed.

- evidence_root: `5f0fab5e432d0628421f6165c73b1d7a4c0b8438713277e3abd60ddd36b4e790`
- context_root: `de2a6202e5c98caf639f10000b0b8606e87afc9d42254e7af58bfeea12458e8d`
- citable anchors: 10

## Binding contract

- A constrained external narrator may re-present this sealed record as prose; it has NO authority over the record.
- Every emitted sentence MUST cite exactly one evidence anchor from the set below (by its full anchor hash).
- It may NOT create evidence, assign or upgrade a claim tier, assert or invent a root cause / causal link / diagnosis, relabel an `unknown` episode as a named fault, merge/split/infer episodes beyond the record, or alter the sealed case file.
- Each sentence is validated by NoNarrativeHallucinationGateV1: a sentence whose anchor is not in this set is rejected.
- A sentence may not assert more than the claim tier its cited anchor carries (SealedFact > EvidenceInterpretation > SpeculativeImplication; NonClaim is never asserted).
- Everything said must already be true in the sealed record. If it is not anchored here, it may not be said.

## Citable evidence anchors

Each row is one fused episode. A sentence may cite an anchor only, and may assert no more than its claim tier.

| Anchor (full hash) | Episode | Motif | Witness strength | Evidence kind | Claim tier | Candidate label / unknown |
|---|---|---|---|---|---|---|
| `97e965e66f64094c48a1b530797a2e1e5100c602469f80ce035f8c2ad006949c` | 134-138 | EV | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_WEAK_QUORUM |
| `ac00fc091bcca2aa8ff67851c0f8e23d2178217213fd4b47db425c60f1c8792f` | 244-249 | EV | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_WEAK_QUORUM |
| `6f2dd6a5a426cae36626c4330e837f46397e3e5fa9e47c04e969dff5e2a243fd` | 259-264 | BG | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_WEAK_QUORUM |
| `a601b2c3d65a26102c0f437b393b14e881e126c5412526b587df0bcb8ace414f` | 281-282 | EV | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_SHORT_TRANSIENT |
| `66ca28f114cedcada9b1aebfe85c79fad5150459a0a0f4d6fb054a7f4d298319` | 322-330 | DA | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_WEAK_QUORUM |
| `90035482cfd5f934940a54d4ffa4cdbe3174be06bcd2d8ee78a6c9c89e0d3136` | 332-334 | EV | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_SHORT_TRANSIENT |
| `a98fccdafbef068afe125febbf19a9c6575e8389dd692c013f71a9cda3c26e5e` | 364-365 | CP | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_SHORT_TRANSIENT |
| `3f9b8ee317048023df81c1e445e6efeb62c961b5d76baa81e90ae430951a45df` | 372-374 | DA | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_SHORT_TRANSIENT |
| `242eaff4da1447a16d0991efb3a148cc7b7de7271fc60a862c1aa3e5f1e4179f` | 393-394 | RC | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | UNKNOWN_SHORT_TRANSIENT |
| `55c0a97e6c901aef36e8af59cb2746acb3c0dfb809e5555f72ab1417bdd5102b` | 400-599 | EV | DetectorFamilyQuorum | chemometric_detector | EvidenceInterpretation | candidate actuator stiction / valve stick-slip |

## Non-claims (never asserted)

- no proven physical root cause
- no causal link asserted
- no accuracy or detection-speed superiority over established methods
- no control signal and no safety-instrumented-function authority
- no regulatory-compliance certification
- every label is a CANDIDATE for operator review, never a confirmed diagnosis

Validation: every emitted sentence is checked by `NoNarrativeHallucinationGateV1` against the anchor set above; an unanchored or over-tier sentence is rejected, never shipped.
