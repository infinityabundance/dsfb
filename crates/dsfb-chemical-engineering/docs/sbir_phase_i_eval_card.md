# SBIR Phase-I evaluation card — DSFB-Chemical-Engineering

> A one-page, generic Phase-I-style evaluation card (no agency, program, or vendor named). It states what a
> Phase-I evaluation of DSFB would attempt, the **replay-checkable go/no-go gates**, the operator-relevant
> success criteria, and — prominently — **what DSFB refuses to claim**. It mirrors `SBIRTransitionPackV1`
> (`transition_pack.rs`) and the paper's operator-evaluation protocol. The month-by-month operationalisation
> (deliverables + gates per 30/60/90-day window) is [`phase_i_workplan_30_60_90.md`](phase_i_workplan_30_60_90.md).
> Advisory; read-only; no control or safety-instrumented-function authority.

## Objective
Evaluate DSFB as a **read-only, deterministic forensic-augmentation layer** over the residuals an existing
chemometric / process-monitoring stack already emits: does it (a) reduce alarm/triage load, (b) produce a
**replayable, byte-exact** evidence case file per incident, and (c) honestly mark what it cannot resolve —
*without* touching control, asserting root cause, or requiring plant data to leave the operator's control?

## Milestone gates (replay-checkable; mirror the paper's M0–M3)
| Gate | Objective | Go/no-go (machine-checkable) |
|---|---|---|
| **M0** | Deterministic replay of the pipeline on the operator's historian export | `verify-replay` byte-identical; `evidence_root` reproduces on a second machine |
| **M1** | A balance/sensor-integrity witness fires on a known labelled event | closure/grammar fires within the labelled window; stays quiet on a known-good window |
| **M2** | Alarm-flood compression on a real upset | raw breach-steps → a small set of fused episodes, `lost_evidence = 0`, recoverable |
| **M3** | Confidential evaluation without raw-data egress | operator runs locally; only a redacted, hash-linked evidence bundle is shared (see `sbir_operator_data_request.md`) |

Each gate is **go/no-go on a sealed artifact**, not on a subjective metric. M3 is the transition-defining gate.

## Success criteria (operator-relevant, NOT accuracy)
- **Triage reduction:** alarm activations → fused episodes (report the ratio; the public TEP IDV(1) figure is 1674× over breach-steps, 17× over alarms).
- **Forensic reproducibility:** every episode → a sealed `evidence_root`; re-run reproduces it byte-for-byte.
- **Honest unknowns:** ambiguous structure is emitted as *unknown (evidence preserved)*, routed by the unknown taxonomy — not forced into a confident label.
- **Data sovereignty:** the operator never has to ship raw time-series (the redacted evidence bundle suffices for review).

## What DSFB refuses to claim (read this first)
No proven physical **root cause**; no **causality**; no **accuracy superiority** over the incumbent; **no control
signal** and **no safety-instrumented-function authority**; **no regulatory-compliance certification**. Every
label is a *candidate*; the public benchmarks (TEP, BATADAL, SWaT, BSM1, CSTR, penicillin) are valuable but
**cannot substitute** for validation on the operator's own historian.

## How to verify replay (do it yourself)
`cargo run -p dsfb-chemical-engineering-edge -- verify-replay` (synthetic suite, 6/6 byte-identical) and
`cargo run -p dsfb-chemical-engineering-edge -- casefile <dataset>` then re-run and confirm the `bundle_root` +
`evidence_root` match `data/EXPECTED_BUNDLE_ROOTS.toml`. See `reports/verification_report.md`.

## Minimum data to attempt M0–M2
`timestamp, tag, value` long-format historian export + a baseline (known-good) window. M1 additionally needs a
closeable, fully-metered control volume (a balance) or a labelled sensor event; M3 needs only the redacted
bundle. The full input contract is `docs/sbir_operator_data_request.md`.
