# SBIR Phase-I 30/60/90-day workplan — DSFB-Chemical-Engineering

> A generic (no agency/program/vendor named) month-by-month Phase-I workplan with explicit deliverables and
> **replay-checkable** go/no-go gates. It operationalises the one-page [`sbir_phase_i_eval_card.md`](sbir_phase_i_eval_card.md),
> uses the input contract in [`sbir_operator_data_request.md`](sbir_operator_data_request.md), and tracks the
> candid [`sbir_risk_register.md`](sbir_risk_register.md). Advisory; read-only; **no control or
> safety-instrumented-function authority**, and **no plant data ever leaves the operator's control**.

## Premise (honest scope)

Most of the read-only evidence court **already exists and is replay-checkable today** on synthetic + public
data (295 workspace tests, `verify-replay` 6/6, 20/20 Court Record bundle roots, the confidential-evaluation
chain, the embedded core). Phase I is therefore **not** a build-from-scratch; it is a focused, low-risk
evaluation answering three questions on a *real* plant export, **without that export ever leaving the
operator**:

1. Does the read-only court reduce alarm/triage load on the operator's own incident history?
2. Does it produce a **byte-exact, replayable** evidence case file per incident that a reviewer can confirm
   from hash roots alone?
3. Does it **honestly mark what it cannot resolve** (unknown-taxonomy route), never asserting root cause,
   control action, or accuracy superiority?

The single data-dependent gap (risk-register **R1**) — public benchmarks ≠ a real plant — is what the Phase-I
historian run closes; everything else is demonstrated pre-award.

## The data-egress posture (unchanged every month)

The operator runs the read-only binary/container **locally** on their historian export. The only artifact
shared for review is the **redacted, hash-linked evidence bundle** (`dsfb-chem-edge confidential-demo` →
`ConfidentialEvaluationBundleV1`: episode structure + hash roots + metrics + non-claims; **no raw time-series,
no real tag names, no raw values**). A reviewer re-derives the hash roots to confirm reproducibility without
ever seeing the data (`PartnerDataEscrowProtocolV1`). This holds for all three months.

---

## Month 1 (days 0–30) — Onboarding + data readiness  →  gate M0

**Objective.** Stand the read-only court up on the operator's environment and grade their historian export
*before* any interpretation, so data trust precedes evidence.

**Activities.**
- Operator supplies the minimum input contract (`tags.csv` long-format + optional `units` / `controllers` /
  `maintenance` / `lab` sidecars) per `sbir_operator_data_request.md` — locally, never transmitted.
- Run `dsfb-chem-edge data-readiness <historian.csv>` → `Ready | ReadyWithCaveats | NotReadyMissingCriticalWitnesses`
  + a `HistorianImportReceiptV1` (file hash, row/tag counts, time range, sampling/missingness profile,
  clock-skew/duplicate warnings).
- Build the `InstrumentationCoverageMapV1` / `ResidualWitnessCoverageScoreV1`: per fault class, *can the
  supplied tags even see it?* — set expectations honestly up front.
- Confirm the toolchain reproduces the shipped guarantees on the operator's machine: `verify-replay` 6/6 +
  `completeness-court` COMPLETE.

**Deliverables.** Local readiness report + import receipt + coverage map; a confirmed local reproduction of
the determinism gates.

**Gate M0 (machine-checkable).** `verify-replay` 6/6 byte-identical on the operator's host **and** the
historian export grades at least `ReadyWithCaveats`. Go/no-go: if `NotReady`, the missing witnesses are named
(no analysis is forced on inadequate data — that is itself a successful, honest M0 outcome).

## Month 2 (days 30–60) — Evidence on real incidents  →  gates M1–M2

**Objective.** Run the court over the operator's history and measure triage-load reduction against their own
incident log.

**Activities.**
- `dsfb-chem-edge historian <export.csv>` → a sealed **Chemical Court Record** per window: fused episodes,
  NE 107 status, detector-family quorum, the universal claim legend (tier · witness strength · evidence kind ·
  unknown route), evidence/bundle roots.
- Quantify alarm/triage compression (raw breach-steps → fused episodes) on the operator's real data, and the
  unknown rate (episodes honestly preserved, not forced).
- Cross-check detected episodes against the operator's **known** incident log (operator-held; only the
  agreement summary is shared) — a within-scope recall/specificity read, framed like the SWaT
  scope-stratified analysis, never an accuracy-superiority claim.
- Emit the **redacted confidential-evaluation bundle** for each reviewed incident (`confidential-demo`).

**Deliverables.** Per-incident Court Records (local) + redacted evidence bundles (shareable) + a
triage-load-reduction measurement on real data.

**Gates M1–M2 (machine-checkable).** M1: every emitted bundle's `bundle_root` + `evidence_root` re-derive
identically on a second host (reproducibility). M2: ≥1 real incident produces a fused episode with a
defensible evidence grade **and** the detection delay vs the operator-labelled onset is reported (signed,
honest — early/late both shown).

## Month 3 (days 60–90) — Review, honest scorecard, transition readiness  →  gate M3

**Objective.** Independent review of the redacted bundles, a candid coverage scorecard, and a go/no-go for a
Phase-II scope — with the non-claims stated as prominently as the results.

**Activities.**
- Reviewer re-derives the hash roots from the redacted bundles alone (no raw data) and confirms reproducibility
  (`PartnerDataEscrowProtocolV1` valid: raw never egressed, only evidence reviewed, reproducible via hashes).
- Produce the honest coverage scorecard: executed-vs-catalogued (18/57 detectors, 7/12 fault signatures today),
  which fault classes the operator's instrumentation can/can't observe, and the residual-witness coverage.
- Map each finding to the risk register; state what a Phase II would *add* (more executed detectors/signatures
  on the operator's classes; a longer historian run) — breadth widened by adding executions, never trimmed.
- Final operator-facing report carrying the claim legend + the full **non-claims** footer.

**Deliverables.** Independent reproducibility confirmation; coverage scorecard; risk-mapped findings; a
Phase-II scope recommendation (go/no-go) with explicit boundaries.

**Gate M3 (machine-checkable).** A reviewer, holding **only** the redacted bundles, reproduces every hash root
and the `release-scrub --archive-dir` on the shared artifact is RELEASE-CLEAN (no raw data, no
leaked backup). Go/no-go for Phase II is recorded against the coverage scorecard.

---

## Deliverables at a glance

| Month | Gate | Primary deliverable | Reproduced by (CLI / artifact) |
|---|---|---|---|
| 1 (0–30) | M0 | Local readiness + import receipt + coverage map | `data-readiness` · `verify-replay` · `completeness-court` |
| 2 (30–60) | M1–M2 | Court Records + redacted bundles + triage-load measurement | `historian` · `casefile` · `confidential-demo` |
| 3 (60–90) | M3 | Independent reproducibility + honest coverage scorecard + Phase-II go/no-go | hash-root re-derivation · `release-scrub --archive-dir` |

## What Phase I deliberately does NOT do (bounded)

- **No control / no SIS authority** — read-only throughout; emits no actuation and no safety function.
- **No root-cause / causality / accuracy-superiority claim** — every label is a CANDIDATE; episodes that meet
  no heuristic are preserved as unknown, never forced.
- **No plant data egress** — the operator keeps all raw data; only redacted, hash-linked evidence is shared.
- **No plant modification, no retraining of the existing monitoring stack** — DSFB augments the residuals an
  operator's stack already emits; removing it restores the pre-deployment baseline exactly.
- **No real-time / WCET certification** — the embedded path is a `no_std` smoke profile (risk-register R8),
  not a certified controller.
