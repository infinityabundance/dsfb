# Head-to-head worked example — Tennessee Eastman IDV(1)

> **Prior-art disclosure / worked example.** This document is a *demonstration of the recorded artifact*,
> **not** a detection-superiority claim. Incumbent practice and the DSFB Chemical Court Record see the
> *same* residuals from the *same* committed input slice; what differs is the **output object** an
> operator is handed. Nothing here claims DSFB detects the fault earlier or more accurately.

## Why this example

The paper positions DSFB-Chemical-Engineering as an observer-side augmentation whose value is measured in
operator-workflow terms, not in displacing an estimator. The natural challenge is *"so what does an
operator actually get that they did not already have?"* This walkthrough answers it on one published
benchmark fault, end to end, from committed inputs.

- **Input:** `crates/dsfb-chemical-engineering-edge/data/slices/tennessee_eastman_idv01.csv` — 960
  samples × 52 variables, `kind = simulation/slice` (TEP is a simulation; the label is now stamped
  correctly at runtime, see the provenance gate).
- **Fault:** IDV(1), a feed-ratio (A/C) disturbance — a sustained, controller-visible upset.

## Reproduce (deterministic)

```sh
cargo run --release -p dsfb-chemical-engineering-edge -- demo
cargo run --release -p dsfb-chemical-engineering-edge -- casefile tennessee_eastman_idv01
python3 crates/dsfb-chemical-engineering-edge/scripts/head_to_head_tep_idv1.py
```

`replay_deterministic = true`: the run is byte-reproducible, so every number below is regenerable. The
run outputs are gitignored (regenerable); the analysis recipe `head_to_head_tep_idv1.py` is committed.

## The comparison

| | **Incumbent: raw detector bank + MSPC contribution plot** | **DSFB Chemical Court Record** |
|---|---|---|
| Triage volume | **10,041** raw breach-steps from 14 detectors (avg **10.5** detectors simultaneously in alarm per sample) | **6** fused episodes |
| | **102** ISA-18.2 alarm activations (false→true rising edges, summed over detectors) | **1,674×** fewer than breach-steps; **17×** fewer than activations |
| Disposition | Contribution plot at the sustained fault ranks `xmeas1`, `xmv3`, `xmv9`, `xmeas19`, `xmeas18` — a ranked list, no disposition, no admissibility | 1 `CANDIDATE_FAULT` + 5 `STRUCTURE_ONLY` claim-boundary badges; global `ROOT_CAUSE_NOT_ADMITTED` |
| Recorded silence | The intervals that *did not* rise to an alarm are unrecorded | **15** rejected candidates, **each** logged with a `QUORUM_NOT_MET` reason + an evidence hash |
| Persistence | Transient: scores + a plot; nothing persisted or replayable | `evidence_root` + `bundle_root`, `REPLAY_VERIFIED`, byte-exact |

### The two numbers that matter

1. **Triage reduction (the obvious one).** 10,041 raw breach-steps → 6 episodes is a 1,674× collapse in
   the count of things an operator must look at. Even counted charitably as ISA-18.2 *annunciations*
   (102, not 10,041), the fusion is still a 17× reduction. This is the same `tab:compression` ratio
   reported for `TEP-idv01` in the paper.

2. **The auditability delta (the one that is the actual contribution).** The incumbent emits a transient
   score and a contribution plot: when the upset clears, there is no artifact to cite. DSFB emits a
   hash-linked bundle in which even the **15 candidate intervals it refused to admit** are recorded —
   each with the reason it failed quorum and a SHA-256 evidence hash. The recorded *silence* is the
   difference: a post-incident reviewer can replay exactly why an alarm-like structure was, or was not,
   admitted.

## What this does *not* show

- **Not** a detection-rate or accuracy comparison — both pipelines key on the same residuals, and the
  contribution-plot variables (`xmv3`, `xmv9`) agree with DSFB's control-action fingerprint for IDV(1).
- **Not** a root-cause claim — the `CANDIDATE_FAULT` badge is a structural candidate; `xmeas1`/`xmv3`/…
  are *witness* variables, not a diagnosed cause (the bundle carries `ROOT_CAUSE_NOT_ADMITTED`).
- **Not** field-validated — TEP is a simulation; this is computational validation (≈ TRL 3).

## Where this lives

Paper: `\subsection{Incumbent comparison: a worked example on Tennessee Eastman IDV(1)}`
(`sec:headtohead`, `tab:headtohead`) under *Operator value*. Recipe:
`crates/dsfb-chemical-engineering-edge/scripts/head_to_head_tep_idv1.py`. The compression figure
cross-checks `tab:compression`.
