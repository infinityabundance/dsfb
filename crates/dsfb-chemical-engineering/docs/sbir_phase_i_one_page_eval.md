# DSFB-Chemical-Engineering — Phase-I one-page evaluation

> A single-page entry route for a Phase-I-style evaluation (no agency/program/vendor named). It states, in one
> place, what DSFB **reads**, what it **never writes**, what it **emits**, the 30/60/90 plan, and the success /
> **failure** metrics. Deeper detail lives in the sibling docs (cross-referenced, not duplicated):
> [`sbir_phase_i_eval_card.md`](sbir_phase_i_eval_card.md) (M0–M3 go/no-go gates),
> [`sbir_operator_data_request.md`](sbir_operator_data_request.md) (full input contract),
> [`sbir_risk_register.md`](sbir_risk_register.md) (risks + kill condition),
> [`phase_i_workplan_30_60_90.md`](phase_i_workplan_30_60_90.md) (month-by-month). Advisory; read-only; **no
> control or safety-instrumented-function authority.**

## Objective
Evaluate DSFB as a **read-only, deterministic forensic-augmentation layer** over the residuals an existing
chemometric / process-monitoring stack already emits — reducing alarm/triage load and producing a **replayable,
byte-exact** evidence case file per incident, while honestly marking what it cannot resolve, **without** touching
control, asserting root cause, or requiring plant data to leave the operator's control.

## Data required (minimum → recommended)
- **Minimum:** a `timestamp, tag, value` long-format historian export + one **baseline (known-good) window**.
- **For a physical witness (M1):** a closeable, fully-metered control volume (a balance) **or** a labelled sensor event.
- **Recommended (richer context):** units, ranges, controllers/setpoints, alarms, maintenance events, batch/campaign
  phases, lab samples, material lots, topology, a data dictionary. Full contract: `sbir_operator_data_request.md`.

## What DSFB reads (read-only)
The historian export and the optional context files above — interpreted against a **process-context contract**
(tag identity, engineering unit, sampling rate, variable role, regime/phase, controller state, declared observability
limits). It reads the residuals an existing detector stack emits; it does not require raw setpoints to act.

## What DSFB NEVER writes
No control signal, no setpoint, no actuator command; **nothing to the DCS / historian / PLC / SIS**. It emits only
its own sealed evidence artifacts to its own output directory. Removing DSFB restores the pre-deployment baseline
exactly (non-interference is Kani-checked).

## Artifacts emitted
A hash-sealed **Chemical Court Record** per run: fused episodes + per-detector passports, balance/topology/control
witnesses with a **PhysicalWitnessStrength** rung, candidate labels (or honest `unknown` routed by the 7-class
taxonomy), confuser dockets, a NAMUR NE 107 status + a 9-question operator one-pager, and a byte-exact `evidence_root`
a GPU reproduces identically. Every claim wears a **ClaimStrength** tier (sealed fact / interpretation / bounded
implication / non-claim).

## 30 / 60 / 90-day plan (replay-checkable gates M0–M3)
- **30 (M0):** deterministic replay on the operator's historian export — `verify-replay` byte-identical; `evidence_root` reproduces on a second machine.
- **60 (M1+M2):** a balance/sensor-integrity witness fires on a known labelled event (quiet on a known-good window); alarm-flood compression on a real upset with `lost_evidence = 0`.
- **90 (M3, transition-defining):** confidential evaluation with **no raw-data egress** — the operator runs locally and shares only a redacted, hash-linked evidence bundle.

## Success metrics (operator-relevant, NOT accuracy)
- **Triage reduction:** alarm activations → fused episodes (report the ratio; public TEP IDV(1) reference: 1674× over breach-steps, 17× over alarms).
- **Forensic reproducibility:** every episode → a sealed `evidence_root` that re-runs byte-for-byte.
- **Honest unknowns:** ambiguous structure emitted as *unknown (evidence preserved)*, not forced into a label.
- **Data sovereignty:** the operator never ships raw time-series; the redacted bundle suffices for review.

## Failure metrics (honest go/no-go — when DSFB should be rejected)
- **Kill condition:** it does **not** reduce triage load on the operator's own ungated historian (the redacted bundle adds review burden instead of removing it).
- Replay is **not** byte-identical across machines (M0 fails) → the forensic-reproducibility claim is void.
- No physical witness can fire on a labelled event because the required instrumentation is absent (observability gap) → M1 not attemptable; reported as an `ObservabilityNonClaim`, not a silent miss.
- The unknown rate is so high that few episodes carry an admitted signature → the heuristics/signature bank does not yet cover the operator's process (a backlog item, reported honestly, not hidden).

## Non-claims (read first)
No proven physical **root cause**; no **causality**; no **accuracy superiority** over the incumbent; **no control
signal**; **no SIS authority**; **no regulatory-compliance certification**. Every label is a *candidate*; the public
benchmarks (TEP, BATADAL, SWaT, BSM1, CSTR, penicillin) cannot substitute for validation on the operator's own historian.

## Licensing / data boundary
DSFB ships under its repository license; **the operator's plant data stays under the operator's control** (confidential
evaluation, no egress). Controlled-access benchmark datasets (e.g. iTrust SWaT/WADI) are **not redistributed** — the
repo ships only scripts, manifests, and aggregate metrics; the user obtains such datasets directly from their custodian
under that custodian's agreement.
