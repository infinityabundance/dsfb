# dsfb-database

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![MSRV: 1.74](https://img.shields.io/badge/MSRV-1.74-orange.svg)](Cargo.toml)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-database/colab/dsfb_database_repro.ipynb)

A deterministic, read-only structural observer for residual trajectories in
SQL database telemetry. Built on the
[`dsfb`](https://crates.io/crates/dsfb) (Drift–Slew Fusion Bootstrap)
core; companion to the application crates at
[github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb).

`dsfb-database` consumes residuals that production SQL engines already
emit — `pg_stat_statements`, `pg_stat_io`, `pg_stat_activity`,
SQL Server Query Store + DMVs, MySQL Performance Schema, Oracle
ASH/AWR — and produces a small, auditable grammar of operator-legible
**motifs**: plan-regression onset, cardinality-mismatch regime,
contention ramp, cache/buffer collapse, workload phase transition.

It is positioned as a **read-only sidecar**, complementary to existing
engine observability surfaces. It does not optimise queries, replace the
optimiser, or modify execution plans.

**One-line operator pitch:** point it at a `pg_stat_statements` CSV
export, get back a deterministic episode stream telling you *which* of
the five motifs fired, *when*, and on *which* channel — replayable to
the byte, with five non-claims pinned by the test suite.

---

## Operator quickstart (PostgreSQL → episode stream in ten lines)

```bash
# 1. Build (release) — first time only, ~30 s on a modern laptop.
cargo build --release

# 2. On your prod box, export pg_stat_statements (PG 14+) once:
psql -d <yourdb> -c "\copy (SELECT queryid, calls, total_exec_time, \
  mean_exec_time, rows FROM pg_stat_statements) TO STDOUT WITH CSV HEADER" \
  > snap_$(date +%s).csv
# Repeat every minute or so for ~10 minutes; concatenate; ship to the analyst box.

# 3. Ingest. The bundled redacted sample under examples/data/ shows the schema:
./target/release/dsfb-database ingest --engine postgres \
  --csv examples/data/pg_stat_statements_sample.csv --out out/pgss

# 4. Inspect: out/pgss/episodes.csv lists each motif fired, with t_start / t_end.
column -ts, out/pgss/episodes.csv | head
```

Schema: the adapter expects the `pg_stat_statements` v14+ columns
verbatim (`queryid, calls, total_exec_time, mean_exec_time, rows`).
Older PostgreSQL versions need a column-mapping flag — open an issue
with your version + column list.

## Non-claims (read these first)

1. DSFB-Database does not optimise queries, replace the query optimiser, or modify execution plans.
2. DSFB-Database does not claim causal correctness; motifs represent structural consistency given observed signals, not root causes.
3. DSFB-Database does not provide a forecasting or predictive guarantee; precursor structure is structural, not probabilistic.
4. DSFB-Database does not provide ground-truth-validated detection on real workloads; we evaluate via injected perturbations, plan-hash concordance, and replay determinism.
5. DSFB-Database does not claim a universal SQL grammar; motifs are engine-aware, telemetry-aware, and workload-aware.

These five strings are pinned by `tests/non_claim_lock.rs` and
reproduced verbatim in §10 of the paper.

---

## Quick start

```bash
# Build
cargo build --release

# Print non-claim charter
./target/release/dsfb-database non-claims

# End-to-end controlled-perturbation pipeline (TPC-DS-shaped exemplar)
./target/release/dsfb-database reproduce --seed 42 --out out

# Single-dataset exemplar (snowset / sqlshare / ceb / job / tpcds)
./target/release/dsfb-database exemplar --dataset ceb --out out

# Real CSV (after fetching subsets via scripts/)
./target/release/dsfb-database run --dataset ceb --path data/ceb_subset.csv --out out

# Replay-determinism check
./target/release/dsfb-database replay-check --seed 42

# Threshold elasticity (+- 20 %)
./target/release/dsfb-database elasticity --seed 42 --out out
```

The `reproduce` subcommand emits CSV (`tpcds.metrics.csv`,
`tpcds.episodes.csv`), JSON (`tpcds.windows.json`,
`tpcds.grammar.json`), per-motif residual-overlay PNGs, and the
provenance manifest. Every figure in the paper is regenerated from this
output.

---

## Datasets

| Dataset | Tier | License | Role |
|---|---|---|---|
| [Snowset](https://github.com/resource-disaggregation/snowset) | Real | CC-BY 4.0 | Workload phase + resource motifs |
| [SQLShare](https://uwescience.github.io/sqlshare/data_release.html) | Real | CC-BY 4.0 | Real ad-hoc human SQL |
| [CEB](https://github.com/learnedsystems/CEB) | Real (ground-truth cardinalities) | MIT | Cardinality mismatch with `EXPLAIN ANALYZE` truths |
| [JOB](https://github.com/gregrahn/join-order-benchmark) | Controlled | MIT | Plan-regression stress on real correlated data |
| TPC-DS (DuckDB extension) | Controlled | TPC EULA | Reproducible perturbation harness |

Subsetting and checksum-verified download scripts live in `scripts/`.
The TPC-DS harness ships fully deterministic in-process — no external
download required for the paper figures.

---

## Architecture

```
                    +-----------------------------+
                    |  dsfb (crate, observer math)|
                    +--------------+--------------+
                                   |
       +-----------------+         |        +---------------------+
       | adapters/*.rs   |---residuals----->| residual/*.rs        |
       | (csv/parquet)   |                  | (5 typed classes)    |
       +-----------------+                  +----------+----------+
                                                       |
                                              +--------v---------+
                                              | grammar/motifs.rs|
                                              | (5 state machines)|
                                              +--------+---------+
                                                       |
                            +--------------------------v-------------------+
                            | report/ (CSV, JSON, PNG); metrics.rs (F1...) |
                            +----------------------------------------------+
```

The DSFB observer math (`dsfb::DsfbObserver`) is consumed unchanged. We
add only the residual *construction* layer (per-engine surfaces) and the
motif *grammar* layer (state machines + episode emission).

---

## Reproducibility

```bash
cargo test --release            # full suite, including replay determinism
                                #   and the non-claim lock
./scripts/reproduce_paper.sh    # regenerates every figure + table in the paper
                                #   (paper PDF build step is skipped if the
                                #    companion paper/ directory is not present
                                #    alongside the crate)
```

Two replay invariants are pinned:

* `paper_fingerprint_is_pinned` — SHA256 of the canonical residual
  stream at seed=42 must match the value cited in §8 of the paper.
* `paper_episode_fingerprint_is_pinned` — same, for the emitted episode
  stream.

If either changes (intentional or otherwise), the test fails until the
paper is updated. This is the technical mechanism behind the
"deterministic, replayable" claim.

---

## Colab

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-database/colab/dsfb_database_repro.ipynb)

Tier-1 reproduction with no local install: the notebook at
`colab/dsfb_database_repro.ipynb` clones the repo, builds the crate, runs
the controlled-perturbation pipeline, and surfaces the same CSV / JSON /
PNG artefacts that `scripts/reproduce_paper.sh` produces locally.

---

## Citing

See `CITATION.cff`. The companion paper (`paper/dsfb-database.tex` +
`paper/dsfb-database.pdf`) is maintained alongside the crate source; when
present it is rebuilt by `scripts/reproduce_paper.sh`. The upstream DSFB
stack is at
[github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb)
with Zenodo DOIs listed in the core crate's README.

Reproducible Colab notebook (Tier-1 reproduction, no local install
required): `colab/dsfb_database_repro.ipynb`.

Changelog: see [`CHANGELOG.md`](CHANGELOG.md).

---

## License

Apache-2.0. See `LICENSE`.

## IP Notice

The theoretical framework, formal constructions, and supervisory methods
described herein constitute proprietary Background IP of Invariant Forge LLC
(Delaware LLC No. 10529072), with prior art established by this publication and
earlier Zenodo DOI publications by the same author. Commercial deployment
requires a separate written license. Reference implementations are released
under Apache 2.0. Licensing: licensing@invariantforge.net
