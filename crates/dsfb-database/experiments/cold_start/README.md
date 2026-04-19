# Cold-start ablation (Pass-2 N2)

Quantifies the §43 cold-start motif tuning gap: how much TTD does each
detector lose when the observer is started near the fault, instead of
after a full warmup window?

## Why

Every motif's grammar uses an EMA-style residual envelope. Warmup
under-fills the envelope and the motif emits on a wider tolerance
band than at steady state. The §43 limitation paragraph names this
honestly. R1's mentorship was: name the gap with measurement, not
prose. This experiment is the measurement.

## What

For each `(fault, replication, detector, warmup_seconds)` tuple, we
take the original tape, drop residual samples with `t < warmup_s`,
recompute the SHA-256 sidecar manifest, patch a matching
ground-truth JSON whose `tape_sha256` references the truncated tape,
and invoke `replay_tape_baselines` on the pair. The bake-off CSV's
`ttd_mean_s`, `recall`, `tp`, `fp` columns for the ground-truth motif
land in `out/cold_start.csv`.

Default warmup grid: `{0, 10, 20, 30}` seconds. The fault is planted
at `t = 30s`, so `warmup_seconds = 30` corresponds to the operator
starting the observer the instant before the fault — the worst case
— and `warmup_seconds = 0` is the original tape.

## How

```sh
bash run.sh
```

Override knobs via env vars:

```sh
WARMUPS=0,5,15,25,30 MAX_REPS=3 PG_OUT=/path/to/sweep bash run.sh
```

≈ 5 minutes wall-clock at defaults (4 faults × 10 reps × 4 warmups ×
< 1 s per replay invocation, sequential).

Replay-only — never captures a new tape, never touches the engine.
The truncation lives entirely in tempdirs and is discarded after each
invocation; the original tapes under `experiments/real_pg_eval/out/`
are read-only.

## Output

* `out/cold_start.csv` — one row per `(fault, rep, detector, warmup_s)`
  on the ground-truth motif.

## Indicative result

DSFB and the change-point baselines all lose recall as warmup
shortens; the loss is largest on `cardinality_mismatch_regime` and
smallest on `contention_ramp` (the contention envelope warms fastest
because its residual channel is high-rate). At
`warmup_seconds = 30`, expect DSFB recall to drop by ≥ 0.2 vs the
`warmup_seconds = 0` baseline on at least one fault class.

## Determinism

The truncation, hash recomputation, and ground-truth patch are pure
functions of (original tape, warmup). `replay_tape_baselines` is a
pure function of (tape, ground_truth). Two independent runs over the
same `PG_OUT` produce byte-equal `cold_start.csv`.
