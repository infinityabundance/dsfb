# Forensic Playback Manual for Post-Mission Analysts

## 1. Purpose

This manual explains how a human analyst should use the DSFB Semiotics Engine outputs during post-mission or post-test analysis.

The engine does not merely produce a binary alarm.
It produces a layered interpretation surface:
- residuals,
- signs,
- syntax,
- grammar,
- semantics,
- comparator context,
- and trust-like outputs.

This manual explains how to read that surface responsibly.

---

## 2. What the Analyst Actually Looks At

In a typical forensic session, the analyst should inspect:

1. residual trajectory
2. drift and slew summaries
3. sign trajectory / projected sign coordinates
4. syntax label
5. grammar status and grammar reason
6. semantic disposition and candidate set
7. comparator triggers
8. trust scalar if present

These layers are intended to be read together, not in isolation.

---

## 3. How to Read a Sign Trajectory

### 3.1 General principle
A sign trajectory is not a latent state estimate.
It is a structured representation of residual evolution.

### 3.2 Common visual intuitions
These are heuristic reading rules, not universal truths.

| Sign Pattern | Typical Structural Reading | Caution |
|---|---|---|
| bounded compact cluster | low structural deviation / baseline-like | may still hide slow drift if window is short |
| sustained radial movement | persistent outward drift | check grammar boundary and envelope calibration |
| looping / cyclic motion | oscillatory structure | may reflect real oscillation or filtered noise |
| isolated sharp spike | abrupt event / slew-rich transition | verify against sampling and smoothing settings |

### 3.3 Important warning
A sign trajectory is evidence of structure, not proof of cause.

---

## 4. How to Read Syntax

Syntax organizes local residual evolution into deterministic classes.

Examples documented in the crate include labels such as:
- `persistent-outward-drift`
- `discrete-event-like`
- `curvature-rich-transition`
- `inward-compatible-containment`
- `bounded-oscillatory-structured`
- `structured-noisy-admissible`
- `weakly-structured-baseline-like`
- `mixed-structured` as a conservative fallback. :contentReference[oaicite:11]{index=11}

### Analyst rule
Treat syntax as:
- a structural summary,
- not a diagnosis.

If syntax remains `mixed-structured`, that is a valid non-commitment, not a failure.

---

## 5. How to Read Grammar

Grammar determines whether the observed structural evolution remains admissible relative to the configured envelope.

### 5.1 Grammar statuses
Typical statuses include:
- Admissible
- Boundary
- Violation

The crate also documents typed reason codes such as:
- `RecurrentBoundaryGrazing`
- `SustainedOutwardDrift`
- `AbruptSlewViolation`
- `EnvelopeViolation`. :contentReference[oaicite:12]{index=12}

### 5.2 Analyst rule
Grammar is the main bridge from structure to seriousness.

A syntax label without a grammar transition may still be interesting.
A grammar transition without semantic uniqueness may still be operationally important.

---

## 6. How to Read Semantics

Semantics is a constrained retrieval layer, not an oracle.

Possible outcomes include:
- Match
- CompatibleSet
- Ambiguous
- Unknown

### 6.1 If the engine says Match
Read:
- the matched motif,
- the grammar status,
- the scenario context,
- and the trust scalar.

Do not read “Match” as guaranteed root cause.

### 6.2 If the engine says CompatibleSet or Ambiguous
This is not failure.
It means:
- the bank contains multiple structurally compatible candidates,
- or the available evidence does not justify narrower commitment.

### 6.3 Heuristic overlap resolution
When two or more interpretations overlap:
1. check grammar reason
2. check trust scalar trajectory
3. inspect comparator context
4. inspect whether the ambiguity is regime-dependent
5. consult domain-specific context outside the engine

The engine is explainable precisely because it allows ambiguity instead of forcing false certainty.

---

## 7. Comparator Context

Comparators provide supporting context, not the final interpretation.

Use them to answer:
- did classical triggers alarm?
- when did they alarm?
- did multiple comparators collapse distinct structures into similar scalar warnings?

This is particularly useful for communicating to colleagues accustomed to threshold or CUSUM logic.

---

## 8. Recommended Analyst Workflow

1. Open the report or dashboard replay.
2. Inspect residual and sign trajectory evolution.
3. Note syntax transition points.
4. Check grammar boundary interactions and reason codes.
5. Review semantic disposition and candidate set.
6. Compare with baseline comparator triggers.
7. Record final analyst judgment in layered form:
   - residual pattern
   - syntax summary
   - grammar reason
   - semantic disposition
   - confidence / unresolved ambiguity

---

## 9. What Not to Do

Do not:
- treat a semantic match as proof of root cause
- ignore `Unknown` or `Ambiguous`
- interpret sign visuals without checking grammar
- ignore envelope calibration assumptions
- present comparator alarms as equivalent to semantic interpretation

---

## 10. Conclusion

The forensic value of the DSFB engine is not just that it detects.
It is that it produces an interpretable, layered audit trail that a human analyst can replay, inspect, and explain.
