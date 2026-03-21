# Theorem-to-Code Traceability Matrix

## 1. Purpose

This document links formal statements in the paper to the concrete implementation surface of `dsfb-semiotics-engine`.

Its purpose is auditability:
- to show where each theorem or definition is realized in code,
- to expose where implementation assumptions enter,
- and to make verification discussions concrete.

This document is not a proof of correctness by itself. It is a traceability aid.

---

## 2. Traceability Policy

Each entry contains:
- Paper item
- Informal role
- Primary implementation file
- Primary function/type
- Verification path
- Source line(s)

**Important:** source lines should be generated from the current checked-out source and updated automatically or semi-automatically when the crate changes.

---

## 3. Matrix

| Paper Item | Role in Theory | Primary Code Location | Function / Type | Verification Path | Source Lines |
|---|---|---|---|---|---|
| Definition: Residual construction | defines discrepancy object | `src/...` | TBD | unit tests + CSV fixtures | TBD |
| Definition: Drift and slew | deterministic temporal descriptors | `src/...` | TBD | property tests on constant/affine signals | TBD |
| Definition: Sign | structured residual object | `src/...` | TBD | sign projection tests | TBD |
| Syntax layer definitions | temporal motif construction | `src/...` | TBD | syntax alignment tests | TBD |
| Grammar / admissibility envelope | admissibility status and reason codes | `src/...` | TBD | grammar tests + trust scalar checks | TBD |
| Semantics / heuristics bank | constrained semantic retrieval | `src/...` | TBD | retrieval tests + bank validation | TBD |
| Theorem: Deterministic Interpretability | identical inputs -> identical outputs | `src/...`, `tests/...` | TBD | reproducibility + property tests | TBD |
| Theorem: Certifiable Early-Warning Inference | deterministic detectability + auditability | `src/...`, `tests/...` | TBD | scenario bundle + reproducibility reports | TBD |
| Theorem: Layer Separation | no collapse of residual/sign/syntax/grammar/semantics | `src/...` | TBD | pipeline decomposition + export audit | TBD |

---

## 4. Verification Notes

### 4.1 Deterministic interpretability
The paper’s determinism claims are reflected in:
- fixed layered transforms,
- explicit intermediate exports,
- reproducibility checks,
- and fixed-bank evaluation assumptions. 

### 4.2 Layer separation
The paper explicitly treats residuals, signs, syntax, grammar, and semantics as distinct inferential stages, and the crate README mirrors this layered separation directly. :contentReference[oaicite:6]{index=6}

### 4.3 Audit trail expectation
This matrix should be paired with:
- schema documentation,
- reproducibility reports,
- bank validation output,
- and integration tests.

---

## 5. Maintenance Rule

Whenever:
- theorem numbering changes,
- files are split/renamed,
- or functions move,

this matrix must be regenerated.

Recommended future improvement:
- add a script that extracts line numbers automatically from tagged code markers.
