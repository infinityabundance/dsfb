# Null pgbench harness (Pass-2 N3)

Measures the *no-fault floor* — every detector's false-alarm-per-hour
on a tape captured during pgbench under the exact same shape as the
multi-fault matrix, but with no fault planted at any point.

## Why

§13 names DSFB's contention_ramp false-positives during the
DROP CONSTRAINT cascade. That number is interpretable only relative
to a known-quiet baseline: if all detectors emit some background FAR
under pgbench load alone, "DSFB emits 7-9 contention_ramp episodes
under DROP CONSTRAINT" needs to subtract whatever DSFB emits under
pgbench-only. R2 / R6 mentorship: publish the no-fault floor, do not
gesture at it.

This is also the bound that the §44 (adversarial workload)
limitation cites: under a benign workload, what's the operator's
expected alert rate?

## What

Identical container, role, pgbench shape, scrape interval, and tape
format as `experiments/real_pg_eval/`, with the fault-injection step
deleted. Per replication:

1. Start pgbench `-c 16 -j 4 -T 90`.
2. Concurrently start `dsfb-database live --interval-ms 500
   --max-duration-sec 90`.
3. After the run completes, write a ground-truth JSON with
   `windows: []` (every detector emission is, by construction, a
   false alarm).
4. Invoke `replay_tape_baselines` on `(tape, empty-windows GT)`.
5. Aggregate across N replications: per-detector mean / min / max /
   sample-stddev FAR/hr.

## How

```sh
bash run.sh
```

≈ 8 minutes wall-clock at defaults (5 reps × 90 s pgbench + container
spinup + bake-off). Override knobs via env vars: `N_REPS`, `DURATION_S`,
`PGBENCH_CLIENTS`, `PGBENCH_JOBS`, `PG_PORT`, `OUT_DIR`.

## Pre-requisites

* podman (rootless OK) with the pinned `postgres:17` digest available.
* Python 3.
* Cargo workspace with `cli`, `report`, `live-postgres` features.

## Output

* `out/r{NN}/{live.tape.jsonl, live.tape.jsonl.hash, ground_truth.json,
  bakeoff.csv}` per replication.
* `out/summary_far.csv` — per-detector aggregate.
* `out/provenance.txt` — container digest, crate SHA, pgbench shape.

## Determinism

Live capture is non-deterministic by design (per non-claim #7). The
captured tape is then byte-pinned by its `.hash` sidecar and the
bake-off is a pure function of (tape, ground-truth); the replay
itself is reproducible. The CI on FAR/hr in `summary_far.csv`
therefore comes from across-replication variance, not from re-running
the same tape.

## Indicative result

Under defaults, expect all four detectors to emit non-zero FAR/hr
even with no fault — pgbench at scale 10, c=16 has enough
micro-jitter that an aggressive change-point detector will fire a
few times per 90-second tape. The §44 paragraph names DSFB's
no-fault FAR/hr explicitly so a reader knows whether the
fault-class FAR/hr in Table 2 is "fault signal + background" or
"fault signal alone".
