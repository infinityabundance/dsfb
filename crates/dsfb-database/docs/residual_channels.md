# Residual Channels

Five structural residual classes, one file each under `src/residual/`. Each
file defines a construction formula, a default admissibility envelope, and
the channel-naming convention used by the adapters.

## `PlanRegression` ([src/residual/plan_regression.rs](../src/residual/plan_regression.rs))

Query-level plan-time regression as a fraction of the per-query baseline.
Positive values mean the current plan runs slower than the query's
baseline; negative values mean faster.

- Construction: `(mean_latency_t - baseline) / baseline`
- Channel: query identifier (adapter-specific, opaque to the grammar)
- Bounded range: no — can spike on cold caches

## `Cardinality` ([src/residual/cardinality.rs](../src/residual/cardinality.rs))

Ratio of observed row count to optimiser-estimated row count, signed so
that under-estimation and over-estimation both surface as large-magnitude
residuals.

- Construction: `log2((obs_rows + 1) / (est_rows + 1))`
- Channel: plan operator fingerprint
- Bounded range: no

## `Contention` ([src/residual/contention.rs](../src/residual/contention.rs))

Lock-wait share of elapsed time. Quasi-bounded in `[0, 1]` but motif code
does not assume that bound because some telemetry exposes wait shares that
exceed 1.0 after attribution.

- Construction: `lock_wait_ms / elapsed_ms`
- Channel: lock class or wait-event type
- Bounded range: effectively `[0, ~1.5]`

## `CacheIo` ([src/residual/cache_io.rs](../src/residual/cache_io.rs))

Miss-rate deviation from the per-query baseline.

- Construction: `(miss_t - baseline) / (hit_t + miss_t + 1)`
- Channel: buffer pool or storage tier
- Bounded range: `[-1, 1]`

## `WorkloadPhase` ([src/residual/workload_phase.rs](../src/residual/workload_phase.rs))

Jensen-Shannon divergence between the current SQL-skeleton histogram and
the rolling baseline histogram.

- Construction: `jsd(histogram_t, baseline_histogram)`
- Channel: SQL-skeleton bucket identifier
- Bounded range: `[0, 1]` by definition of JSD (base-2 logarithm).
- Note: on Snowset, the real workload's skeleton histogram is frequently
  fully disjoint from baseline, producing JSD pinned at 1.0. The display
  layer recognises saturation and emits per-episode ticks rather than
  solid-red merged bands.

## Channel Label Convention

Channel strings are adapter-specific. The grammar treats them as opaque
identifiers and hashes them directly into the episode fingerprint. To
prevent warehouseId-style 20-digit identifiers from appearing in figures,
the display layer passes channel labels through
`humanize_channel_label` which rewrites pure-digit or long labels to a
stable `id@XXXXXX` form. **The adapter-side channel string remains
unchanged**; only the display label is rewritten.
