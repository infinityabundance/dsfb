# dsfb-database

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![MSRV: 1.74](https://img.shields.io/badge/MSRV-1.74-orange.svg)](Cargo.toml)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-database/colab/dsfb_database_repro.ipynb)
[![DSFB Gray Audit: 58.9% mixed assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-58.9%25-yellowgreen)](audit/dsfb_database_scan.txt)

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
the byte, with seven non-claims pinned by the test suite.

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
6. DSFB-Database does not validate that an operator-supplied grammar is appropriate for a non-SQL residual stream; the generic CSV adapter is a worked example, not a universality claim.
7. DSFB-Database's live PostgreSQL adapter emits residuals at a cadence bounded by its polling interval, the engine's response latency, and the operator-configured CPU budget; it does not provide hard-real-time guarantees. Determinism holds only given a persisted tape — two live invocations against the same engine workload will produce different tapes.

These seven strings are pinned by `tests/non_claim_lock.rs` and
reproduced verbatim in §10 of the paper.

---

## Quick start

```bash
# Build
cargo build --release --features report

# Print non-claim charter
./target/release/dsfb-database non-claims

# One-command reproduction (every bundled artefact + deterministic zip)
./target/release/dsfb-database reproduce-all --seed 42 --out out

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

# Generic CSV — apply the grammar to any residual stream (operator responsibility)
./target/release/dsfb-database generic --csv examples/data/generic_sample.csv --out out
```

### One-command reproduction

`reproduce-all` composes the canonical TPC-DS pipeline with all bundled
dataset exemplars, the DSFB-vs-baselines comparison figure, the
null-trace refusal figure, and the cross-signal / stability metrics,
then packs everything into `out/dsfb_database_artifacts.zip`. The zip is
byte-stable across runs (same seed ⇒ same SHA-256); the
`tests/reproduce_all_zip_is_deterministic.rs` test pins this invariant.
A `MANIFEST.md` in the same directory enumerates every file, grouped by
paper section.

### Generic CSV mode

The `generic` subcommand runs the motif grammar over an
operator-supplied CSV. The adapter auto-detects a timestamp column
(header contains `t`/`time`/`timestamp`/`ts`), a numeric value column,
and an optional `channel`/`qclass`/`group`/`series` column; CLI flags
`--time-col`, `--value-col`, and `--channel-col` override the
auto-detection. By default the adapter builds residuals the same way as
the PostgreSQL adapter (subtract the mean of the first three samples
per channel, normalise by `max(|baseline|, ε)`); `--pre-residualized`
skips the subtraction for inputs that are already `(actual − expected)`
residuals. `--grammar <path.json>` loads an alternate grammar (the
JSON schema matches `out/tpcds.grammar.json`); the default is the
crate's pinned grammar. This is a **worked example**, not a
universality claim — the operator is responsible for confirming the
grammar is appropriate for the input signal (see non-claim #6).

The `reproduce` subcommand alone emits CSV (`tpcds.metrics.csv`,
`tpcds.episodes.csv`), JSON (`tpcds.windows.json`,
`tpcds.grammar.json`), per-motif residual-overlay PNGs, and the
provenance manifest. Every figure in the paper is regenerated from this
output.

---

## Datasets

| Dataset | Tier | License | Role |
|---|---|---|---|
| Dataset | Tier | License | Bundled sample | Full-corpus fetch |
|---|---|---|---|---|
| [Snowset](https://github.com/resource-disaggregation/snowset) | Real | CC-BY 4.0 | `examples/data/snowset_sample.csv` — 5 k real rows from the Cornell mirror | `scripts/fetch_snowset_subset.sh` (~7.5 GB gzipped, slices first 200 k rows) |
| [SQLShare](https://uwescience.github.io/sqlshare/data_release.html) | Real (text-only) | CC-BY 4.0 | `examples/data/queries.txt` — full 11,136-query release | `scripts/fetch_sqlshare.sh` (S3 bucket decommissioned — falls back to the bundled file or a user-provided zip) |
| [CEB](https://github.com/learnedsystems/CEB) | Real (ground-truth cardinalities) | MIT | `examples/data/ceb_sample.csv` — 5 k real `(actual, expected)` rows | `scripts/fetch_ceb.sh` (clones repo + pulls the CEB-IMDb pickle tarball) |
| [JOB](https://github.com/gregrahn/join-order-benchmark) | Real (EXPLAIN ANALYZE trace) | MIT | `examples/data/job_trace_sample.csv` — all 113 JOB queries × 3 replays, real DuckDB+IMDb EXPLAIN ANALYZE | `scripts/fetch_job.sh` (downloads IMDb 1.17 GB, loads into DuckDB, runs every query) |
| TPC-DS (DuckDB extension) | Controlled | TPC EULA | — | `scripts/build_tpcds.sh` (deterministic in-process, no download required for paper figures) |

All bundled samples are **real data**, sliced directly from the authoritative
public dumps; they are large enough to exercise every motif each dataset can
support and small enough that `cargo run --release -- run --dataset <ds>
--path examples/data/<sample>` completes in a few seconds. The
`scripts/fetch_*.sh` scripts pull the full corpora for anyone who wants to
rerun at production scale — they are heavy and are not required to
reproduce the paper's fingerprints.

**SQLShare — text-only mode disclosure.** The 2015 SIGMOD release's
richer CSV (`QueriesWithPlan.csv`, carrying `user_id` / `runtime_seconds`
/ `submitted_at`) lived on the S3 bucket `shrquerylogs`, which has been
decommissioned (`NoSuchBucket` as of 2026-04). The only remaining public
artefact is the UW eScience `sqlshare_data_release1.zip`, whose
`queries.txt` contains raw SQL texts separated by 40-underscore
dividers — **no timing, no user id, no submission timestamp**. We
therefore ship a narrower adapter (`--dataset sqlshare-text`) that emits
**only** the `WorkloadPhase` motif, using Jensen-Shannon divergence
between skeleton-histograms of consecutive 200-query ordinal buckets.
This is **structural-ordering JSD, not temporal drift**: the `t` axis is
ordinal-bucket-index (scaled for plot consistency), not wall-clock. The
`PlanRegression`, `Cardinality`, `Contention`, and `CacheIo` classes are
absent for this dataset — the public release does not carry the fields
required to construct them, and fabricating those fields would be a
category error. Streams from this adapter carry the
`sqlshare-text@<file>` source tag so downstream reports cannot confuse
them with wall-clock-indexed SQLShare runs.

---

## Operator runbook (production deployment)

The ten-line quickstart above gets one CSV through the pipe. This section
covers the rest: feature flags, the OpenTelemetry adapter, the streaming
ingestor, Prometheus exposition, per-motif threshold tuning, and alerting
hooks. Everything here is additive to the batch path — the four pinned
fingerprint locks continue to construct via the canonical batch
`ResidualStream::push` → `sort` sequence.

### Feature profiles

The crate is feature-gated so library consumers can opt out of CLI,
plotting, JSON, and OTel payload surface they do not need. `cargo tree
--depth 1` reports ≤10 direct dependencies on the default profile.

| Feature | Pulls | When to enable |
|---|---|---|
| `cli` *(default)* | `clap` | The bundled `dsfb-database` binary and the nine auxiliary binaries (`variance_sweep`, `pr_sweep`, `null_trace`, `baseline_bake_off`, `inject_over_real`, `ingest_throughput`, `ablation_sweep`, `tpc_c_generalization`). |
| `report` | `plotters`, `serde_json` | PNG figure emission and JSON sidecar artefacts. Required by the main binary and `pr_sweep`. |
| `otel` | `serde_json` | The OpenTelemetry DB-spans adapter at [src/adapters/otel.rs](src/adapters/otel.rs). |
| `full` | all of the above | Convenience superset: `cargo install dsfb-database --features full`. |

Library-mode consumers who only want the motif grammar and the batch
adapters should depend on `dsfb-database` with `default-features = false`.

### OpenTelemetry DB-spans adapter (`--features otel`)

The OTel adapter consumes a JSON array of simplified DB spans and emits
`PlanRegression` residuals from per-`statement_hash` duration drift. The
shape is forward-compatible with `otel-collector`'s OTLP/JSON DB-span
export:

```json
[
  {"t_start_ns": 1700000000000000000, "t_end_ns": 1700000000050000000,
   "statement_hash": "a1b2c3", "db_system": "postgresql"},
  {"t_start_ns": 1700000001000000000, "t_end_ns": 1700000001048000000,
   "statement_hash": "a1b2c3", "db_system": "postgresql"}
]
```

```rust
use dsfb_database::adapters::otel::load_otel_db_spans;
let stream = load_otel_db_spans(std::path::Path::new("spans.json"))?;
```

Baseline is the first `BASELINE_WINDOW = 3` observed durations per
`statement_hash`; subsequent samples emit a `log(duration / baseline)`
residual. Emission order is deterministic (hashes sorted; ties broken on
`t_start_ns`). Non-positive durations are dropped. A production
`otel-collector` pipeline that writes a rolling JSON file consumed by
this adapter is the lowest-friction way to get live OTel signal into the
motif grammar today; a native OTLP/gRPC ingestor is deferred to a pilot
LOI.

### Streaming ingestor and reorder-window tuning

`StreamingIngestor` at [src/streaming.rs](src/streaming.rs) accepts
samples one at a time and flushes a correctly-ordered prefix as the
reorder window slides forward:

```rust
use dsfb_database::streaming::StreamingIngestor;
let mut ing = StreamingIngestor::new("pg_stat_statements@prod-01");
ing.push(sample);
// ... later ...
let (stream, dropped_out_of_window) = ing.finish();
```

The default window is 10 s (`DEFAULT_REORDER_WINDOW_S`). Sizing guidance:

| Telemetry source | Typical cadence | Suggested window |
|---|---|---|
| `pg_stat_statements` poll | 60 s | 10 s (≈6× margin — default) |
| `pg_stat_io` poll | 10 s | 2 s |
| OTel DB spans (batched) | sub-second | 1 s |
| Log-tail adapter (file-based) | async bursty | 30 s |

Any sample whose `t` falls more than `reorder_window_s` behind the
already-flushed frontier is **dropped** and counted in
`dropped_out_of_window`. The streaming path is **parallel** to batch; it
is not expected to produce the batch path's pinned fingerprint on
jitter-bearing inputs (that is why batch is the reproducibility baseline).

### Prometheus / OpenMetrics exposition

[src/metrics_exporter.rs](src/metrics_exporter.rs) renders a
deterministic OpenMetrics 1.0 text blob from a `MetricsSnapshot`:

```rust
use dsfb_database::metrics_exporter::{MetricsSnapshot, render_openmetrics};
let snap = MetricsSnapshot::from_episodes(&closed_episodes)
    .with_streaming(&ingestor);
let body = render_openmetrics(&snap);
```

No HTTP runtime is pulled in. Three wiring options, each under 100
lines, cover the realistic deployment shapes:

1. **Plain `std::net::TcpListener` loop** — one accept loop, read until
   `\r\n\r\n`, respond with `HTTP/1.1 200 OK\r\nContent-Type: application/openmetrics-text; version=1.0.0; charset=utf-8\r\n\r\n{body}`.
   Zero extra dependencies.
2. **`tiny_http` crate** — add `tiny_http = "0.12"`, bind `:9184`, respond
   to every request with the body. One page of code; survives 100 rps
   scraping easily.
3. **Textfile collector (recommended for hardened environments)** —
   write the body atomically to `/var/lib/node_exporter/dsfb.prom` on a
   timer; `node_exporter`'s `textfile` collector picks it up. No extra
   listener, no extra port, no extra CVE surface.

Exposed metrics:

- `dsfb_episodes_total{motif}` — counter, per-motif episodes emitted.
- `dsfb_episode_peak_last{motif}` — gauge, `|peak|` of the most recently
  closed episode.
- `dsfb_episode_trust_sum_last{motif}` — gauge, observer trust-sum at
  the episode boundary. **Should stay in [0.99, 1.01]**; deviations
  indicate an observer bug.
- `dsfb_streaming_residuals_staged` — gauge, current reorder-buffer
  occupancy.
- `dsfb_streaming_residuals_flushed_total` — counter.
- `dsfb_streaming_residuals_dropped_out_of_window_total` — counter.
  **Any non-zero value is alertable**: it means the telemetry pipeline's
  jitter exceeds the configured reorder window; raise
  `reorder_window_s` or fix the upstream cadence.

### Per-motif envelope tuning

The default `MotifParams` (`drift_threshold=1.0`, `slew_threshold=1.0`,
`min_dwell_seconds=30`, `rho=0.90`, `sigma0=0.10`) are the values used
for the pinned fingerprints and the published F1 results. The
`ablation_sweep` binary reports which knobs each motif is most sensitive
to:

- `plan_regression`, `contention`, `cache_collapse` are stable under
  ±50% envelope sweeps (F1 = 1.0 across the tested range).
- `cardinality_mismatch` and `workload_phase_transition` recall collapse
  above `slew_threshold ≥ 1.5×`. For workloads where these two motifs
  matter, keep the envelope within the pre-registered band.

Start with defaults. If false-alarm rate on a quiet null trace exceeds
the per-motif bounds cited in the paper's §6.3 (`null_trace` calibration),
raise `drift_threshold` by 0.1 at a time. If detection latency is too
high on a known incident, lower `min_dwell_seconds` — never below 10 s
for `pg_stat_statements`-class telemetry (you will see poll-jitter
episodes).

### Alerting

A minimal PagerDuty / Slack integration reads the CSV episode stream
(`out/episodes.csv`) and fires one event per row. A more integrated
deployment scrapes `dsfb_episodes_total{motif}` and alerts on a non-zero
delta within a 5-minute window — this de-duplicates naturally and leaves
the full episode record (peak, `trust_sum`, channel, `t_start`/`t_end`)
in the Prometheus history for post-incident review. Always alert on
`dsfb_streaming_residuals_dropped_out_of_window_total > 0`; it is the
telemetry-pipeline health signal, not a motif signal.

### What is intentionally not shipped yet

- A live `pg_stat_statements` daemon (polling a running Postgres on a
  configurable interval) is deferred to a pilot LOI. The offline adapter
  at [src/adapters/postgres.rs](src/adapters/postgres.rs) handles the
  same CSV schema a psql `\copy` cron produces, so the near-term
  deployment path is `cron` + the offline adapter.
- Helm chart and Docker image are deferred for the same reason: the
  right ergonomics (values-file layout, sidecar vs. daemonset, secret
  handling) are deployment-shape-specific and are best drawn from a
  real pilot environment rather than synthesised up front.

---

## Live read-only mode (PostgreSQL)

The default build reads residuals from files. Behind the optional
`live-postgres` feature, `dsfb-database` can also connect directly to
a running PostgreSQL instance, scrape `pg_stat_*` views at a
configurable cadence, distill the cumulative-counter deltas into
residual samples on the fly, and emit motif episodes as they close.

The live path is a **read-only sidecar**, enforced by three layered
controls documented in [`src/live/mod.rs`](src/live/mod.rs):

1. **Type-level.** The public API on `ReadOnlyPgConn` is a single
   method that accepts a variant of a closed allow-list enum. The
   underlying `tokio_postgres::Client` is private; `execute`,
   `prepare`, `transaction`, `copy_in`, `copy_out`, and
   `batch_execute` are not re-exported. A compile-fail test
   ([tests/live_readonly_conn_surface.rs](tests/live_readonly_conn_surface.rs))
   pins this surface.
2. **Session-level.** On connect we issue
   `SET default_transaction_read_only = on` and verify via
   `current_setting(...)` that the setting took effect.
3. **Statement-level.** The SHA-256 of the concatenated allow-list
   SQL texts is pinned by
   [tests/live_query_allowlist_lock.rs](tests/live_query_allowlist_lock.rs).
   Any edit to a live query forces an intentional lock bump.

Together these are a **code-audit contract, not a cryptographic
proof**. The crate's 7th non-claim is explicit about that distinction.

### Operator quickstart

```bash
# 1. Provision the minimum-privilege observer role.
#    (Reference: spec/permissions.postgres.sql; also dumped by --print-permissions-manifest.)
./target/release/dsfb-database live --print-permissions-manifest | psql -U postgres

# 2. Run a live session for 10 minutes at 1 Hz polling, 10 % CPU budget,
#    writing an audit tape + an incremental episode CSV.
./target/release/dsfb-database live \
    --conn "host=/var/run/postgresql user=dsfb_observer dbname=app" \
    --interval-ms 1000 \
    --cpu-budget-pct 0.1 \
    --max-duration-sec 600 \
    --tape out/live.tape.jsonl \
    --out out/live

# 3. Replay the tape offline to produce a byte-deterministic episode stream.
./target/release/dsfb-database replay-tape \
    --tape out/live.tape.jsonl \
    --out out/replay
```

### Determinism boundary

The live path does **not** inherit the byte-determinism guarantee
that offline adapters enjoy:

* **engine → tape**: not deterministic. Two live sessions against the
  same workload will produce different tapes because of sampling
  jitter, counter-advance timing, and concurrent load.
* **tape → episodes**: deterministic. Given a tape, the replayed
  episode stream is byte-stable. This is pinned by
  [tests/live_tape_replay_is_deterministic.rs](tests/live_tape_replay_is_deterministic.rs)
  and stated verbatim in the 7th non-claim.

### Backpressure (measurement-based, not contractual)

The scraper maintains a rolling 16-poll window of wall-clock poll
duration and self-time / wall-clock CPU ratio. If either signal
crosses the operator-configured budget, the next inter-poll sleep
doubles (bounded at 60 s). After three consecutive within-budget
polls the sleep halves back toward the nominal interval. This is a
**governor, not a contract**: the paper's §Live section and 7th
non-claim explicitly disclaim a hard real-time guarantee. Every poll
writes a telemetry-of-the-telemetry row to `out/live/poll_log.csv`
so the operator can see what the sidecar is costing.

### Cardinality is refused on the live path

`pg_stat_statements` does not expose estimated-vs-actual row counts,
so the live adapter **does not** emit a cardinality residual. This
matches the `\pmark` in the paper's Table 10 for PostgreSQL ×
Cardinality. Operators who need the cardinality channel must use
`auto_explain` + JSON parsing, which is out of scope for this
adapter.

### Real-engine evaluation

The paper's §Live Evaluation reports an empirical bake-off against
a planted structural fault:

* Container: `postgres:17` (pinned by SHA-256 digest; see
  `experiments/real_pg_eval/run.sh`).
* Workload: `pgbench -c 16 -j 4 -T 70` at scale-10
  (TPC-B-like).
* Fault: `ALTER TABLE pgbench_accounts DROP CONSTRAINT
  pgbench_accounts_pkey` at `t = 30 s`, forcing sequential
  scans on the two pgbench account qids.
* Detectors: DSFB, ADWIN, BOCPD, PELT — all scored by the
  identical `metrics::evaluate` path on the identical live-captured
  tape, under the identical ground-truth window.
* Aggregation: `n = 10` replications; bootstrap 95 % CI at
  `B = 1000`.

To reproduce the n=10 summary table (this runs locally, not in CI;
requires `podman` and ~12 min of wall-clock per fault class):

```
cargo build --release --features "cli report live-postgres"
# single fault class (default: drop_constraint):
FAULT=drop_constraint bash experiments/real_pg_eval/run.sh
# full multi-fault matrix (4 classes x 10 reps, ~50 min):
for f in drop_constraint stats_stale lock_hold cache_evict; do
  FAULT=$f bash experiments/real_pg_eval/run.sh
done
# writes experiments/real_pg_eval/out/<fault>/r*/bakeoff.csv +
# experiments/real_pg_eval/out/summary.csv aggregated across faults
```

To reproduce the engine-version-compat row (PG16 vs PG17,
~50 min) writing to an isolated directory that does not clobber
the PG17 headline:

```
for f in drop_constraint stats_stale lock_hold cache_evict; do
  OUT_DIR="$PWD/experiments/real_pg_eval/out_pg16" \
  PG_IMAGE=docker.io/library/postgres:16 \
  PG_IMAGE_DIGEST=sha256:01710bb7d42744d53c02d61d7b265b2901aa5fc21ed3f7e35e726b29af5deeb6 \
  FAULT=$f \
  bash experiments/real_pg_eval/run.sh
done
python3 experiments/real_pg_eval/compat_to_tex.py \
  experiments/real_pg_eval/out/summary.csv \
  experiments/real_pg_eval/out_pg16/summary.csv \
  sha256:7ad98329d513dd497293b951c195ca354274a77f12ddbbbbf85e68a811823d72 \
  sha256:01710bb7d42744d53c02d61d7b265b2901aa5fc21ed3f7e35e726b29af5deeb6 \
  paper/tables/pg_version_compat.tex
```

### Multi-engine, multi-fault evaluation

Beyond the single-fault PG17 protocol the paper reports four
auxiliary harnesses. All are local-only, all are `podman`-based,
and all share the identical `metrics::evaluate` scoring path:

* **Observer self-load** (pgbench latency CDF with vs. without
  the scraper) — `experiments/observer_load/run.sh`. Writes
  `paper/tables/observer_self_load.tex` +
  `paper/figs/observer_self_load_cdf.png`.
* **Held-out baseline tuning** (ADWIN / BOCPD / PELT swept on
  replication 01 of every fault class, frozen, and evaluated on
  replications 02–10) — `experiments/baseline_tune/run.sh`.
  Writes `paper/tables/baseline_tuned.tex`.
* **Public-trace FAR bake-off** (DSFB + 3 baselines on Snowset,
  SQLShare, CEB, JOB, TPC-DS — workload-stress FAR upper bound,
  not detection quality) — `experiments/public_trace/run.sh`.
  Writes `paper/tables/public_trace_far.tex`.
* **MySQL live adapter contract** (contract-layer `podman mysql:8`
  harness; provisions the verbatim `dsfb_observer` role from
  `spec/permissions.mysql.sql` and prints the SHA-256-pinned
  allow-list) — `experiments/real_mysql_eval/run.sh`. The
  end-to-end tape capture is future work; the three-layer
  code-audit contract (see `src/live_mysql/` and
  `tests/live_query_allowlist_lock_mysql.rs`) is reviewable
  today.

The paper's two figures are byte-deterministic functions of the
two **pinned** tapes at `paper/fixtures/live_pg_real/`, which are
not regenerated by `run.sh` (engine→tape is non-deterministic
per the 7th non-claim). The n=10 summary table **is** regenerated
up to CI overlap.

To re-render the figures:

```
./target/release/render_live_eval_figures \
    --fixtures-dir paper/fixtures/live_pg_real \
    --figs-dir paper/figs
```

Per-run byte-determinism of the bake-off CSV is pinned by
[`tests/live_replay_baselines_reproducibility.rs`](tests/live_replay_baselines_reproducibility.rs).

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
./scripts/reproduce_paper.sh    # fast path (~minutes, no podman): regenerates
                                #   every synthetic-data figure + table.
                                #   Paper PDF build is skipped if the companion
                                #   paper/ directory is absent.
./experiments/reproduce_all.sh  # heavy path (~3.5 hours, requires podman):
                                #   chains every live experiment behind the
                                #   paper's auxiliary tables (real_pg_eval x
                                #   {PG17, PG16}, observer_load, baseline_tune,
                                #   public_trace, real_mysql_eval) and rebuilds
                                #   the PDF.
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
`colab/dsfb_database_repro.ipynb` clones the repo, builds the crate, and
runs the controlled-perturbation pipeline, producing the CSV / JSON / PNG
artefacts.

---

## Citing

If you use this crate, please cite:

> de Beer, R. (2026). *DSFB-Database: A Deterministic, Read-Only Structural
> Observer for Residual Trajectories in SQL Database Telemetry: An Empirical
> Prior-Art Demonstration on Five Public Workloads* (v1.0). Zenodo.
> [https://doi.org/10.5281/zenodo.19656368](https://doi.org/10.5281/zenodo.19656368)

See `CITATION.cff` for the machine-readable entry. The companion paper
(`paper/dsfb-database.tex` + `paper/dsfb-database.pdf`) is maintained
alongside the crate source; when present it is rebuilt by
`scripts/reproduce_paper.sh`. The upstream DSFB stack is at
[github.com/infinityabundance/dsfb](https://github.com/infinityabundance/dsfb)
with Zenodo DOIs listed in the core crate's README.

Reproducible Colab notebook (Tier-1 reproduction, no local install
required): `colab/dsfb_database_repro.ipynb`.

Changelog: see [`CHANGELOG.md`](CHANGELOG.md).

---

## Audit

This crate is audited by
[`dsfb-gray`](https://crates.io/crates/dsfb-gray), a deterministic,
read-only static auditor that emits assurance scores and
machine-verifiable attestation artifacts (SARIF, in-toto, DSSE). The
current scan of `dsfb-database v0.1.0` reports an overall score of
**58.9 %** ("mixed assurance posture"), against the
`dsfb-assurance-score-v1` rubric.

The scan is **not a compliance or certification claim** — it is a
source-visible structural audit against the DSFB rubric, intended as
a review-readiness and code-improvement signal.

Full artefacts live under [`audit/`](audit/):

| File | Purpose |
| --- | --- |
| [`audit/dsfb_database_scan.txt`](audit/dsfb_database_scan.txt) | Human-readable report with scoring, findings, and conclusion lenses |
| [`audit/dsfb_database_scan.sarif.json`](audit/dsfb_database_scan.sarif.json) | SARIF 2.1.0 export for CI / code-review ingestion |
| [`audit/dsfb_database_scan.intoto.json`](audit/dsfb_database_scan.intoto.json) | in-toto v1 attestation statement |
| [`audit/dsfb_database_scan.dsse.json`](audit/dsfb_database_scan.dsse.json) | DSSE envelope wrapping the in-toto statement (unsigned by default) |

To regenerate the audit against the current crate source:

```bash
cargo install dsfb-gray
dsfb-scan-crate dsfb-database   # scans the latest crates.io release
# or, against this working tree:
dsfb-scan-crate crates/dsfb-database
```

The source SHA-256 recorded in each artefact pins the exact tarball
that was scanned.

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
