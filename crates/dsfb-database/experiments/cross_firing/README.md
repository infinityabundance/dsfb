# Cross-firing matrix (Pass-2 N1)

Promotes the §13 ¶2 prose-only observation in `paper/dsfb-database.tex`
— "DSFB over-fires on `contention_ramp` during the `drop_constraint`
cascade" — into a **measured matrix** so the §36 face-on limitation
paragraph can quote numbers rather than gestures.

## Why

The Pass-2 panel (R1, R5) flagged that the contention-ramp cross-firing
is the single most obvious empirical gap in the paper as shipped:
DSFB's recall and TTD on `plan_regression_onset` are competitive with
the change-point baselines, but its FAR/hr on the *adjacent* motif
`contention_ramp` during the same fault is an order of magnitude
higher. R5's mentorship: name the gap explicitly with measurement,
not just prose.

## What

For every (planted_fault × emitted_motif × detector) triple on the
existing multi-fault tape corpus already produced by
`experiments/real_pg_eval/run.sh`, compute the mean episode count per
replication.

| Fault planted     | Ground-truth motif (diagonal)        |
|-------------------|--------------------------------------|
| `drop_constraint` | `plan_regression_onset`              |
| `stats_stale`     | `cardinality_mismatch_regime`        |
| `lock_hold`       | `contention_ramp`                    |
| `cache_evict`     | `cache_collapse`                     |

Off-diagonal cells quantify cross-firing. The bake-off CSV records
per-motif `tp` and `fp` columns; this aggregator counts the GT cell as
TP and every other cell as FP, so each row of the matrix reads as
"when fault X is planted, how many episodes does detector D emit on
motif M?".

## How

```sh
bash run.sh
```

Replay-only — never captures a new tape, never touches the engine.
≈ 1 second wall-clock (Python aggregator over O(160) CSVs).

The script reads `PG_OUT=experiments/real_pg_eval/out/` by default;
override with `PG_OUT=...` to point at a sibling sweep.

## Output

* `out/cross_firing.csv` — one row per (detector, fault, motif) cell
  with mean / max / min count, sample stddev, and an `is_gt` flag.
* `paper/tables/cross_firing.tex` — LaTeX matrix with bold cells on the
  ground-truth diagonal. Cited by paper §36.

## Indicative result

DSFB's `(drop_constraint, contention_ramp)` cell is in the high single
digits per replication while ADWIN's is zero and PELT's is two — the
quantitative version of the §13 prose. The §36 paragraph promotes this
from an aside into a named limitation.

## Determinism

Pure function of the input CSVs. Two independent runs over the same
`PG_OUT` produce byte-equal `cross_firing.csv` and `cross_firing.tex`.
