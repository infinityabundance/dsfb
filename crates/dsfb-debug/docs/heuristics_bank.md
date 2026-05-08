# DSFB-Debug — Heuristics Bank Reference

Authoritative per-motif reference for the 32 entries in
[`src/heuristics_bank.rs`](../src/heuristics_bank.rs)
(`HeuristicsBank::with_canonical_motifs`). Each entry is a hand-curated
rule that maps a structural signature (reason code + drift + slew +
boundary density + signal correlation + duration) to a named
`MotifClass` interpretation, with explicit provenance, IEEE 24765 /
Avizienis–Laprie–Randell taxonomy anchor, dataset-DOI evidence base,
and a one-line dashboard hint for the production debug engineer
reading the matched motif.

**Authoring policy.** Every motif decomposes into established software-
engineering vocabulary (no ad-hoc names). Every motif has a populated
`taxonomy_ref`, `dashboard_hint`, and non-empty `evidence_dataset`.
DatasetObserved entries also carry a non-empty
`evidence_dataset_doi` cross-referenced in
[`../data/MANIFEST.toml`](../data/MANIFEST.toml). LO2-style API-semantic
anomalies are deliberately NOT in the bank — those validate the
`SemanticDisposition::Unknown` (endoductive) branch.

**Scoring (episode-level, `match_episode`).** Per-motif weighted sum
across drift × slew × boundary-density × signal-correlation × duration,
gated by reason-code match plus min/max range checks on correlation
count and duration windows. Tie-breakers: higher provenance rank
(`FieldValidated > DatasetObserved > FrameworkDesign`), then lower
index. Fully deterministic.

The 32 entries are grouped into seven tiers:

- **Tier 1** — original framework motifs (10): `MemoryLeakDrift`,
  `CascadingTimeoutSlew`, `DeploymentRegressionSlew`,
  `CacheDegradationGrazing`, `ConnectionPoolExhaustionDrift`,
  `GcPressureOscillation`, `ErrorRateEscalation`, `DependencySlowdown`,
  `ResourceSaturation`, `QueueBackpressure`.
- **Tier 2** — TADBench / TrainTicket fault cases (5):
  `RetryStormCascade`, `CircuitBreakerOpenShift`,
  `DatabaseLockContention`, `AuthenticationFailureSpike`,
  `ConfigDriftRegression`.
- **Tier 3** — AIOps Challenge categories (6):
  `PacketLossErrorEscalation`, `NetworkDelayDependencyInflation`,
  `DiskIoSaturation`, `CpuSaturation`, `JvmHeapPressure`, `JvmGcPause`.
- **Tier 4** — MultiDim-Localization patterns (3):
  `ServiceGraphDriftPropagation`, `HighDimAnomalyCluster`,
  `MetricCorrelationCollapse`.
- **Tier 5** — DeepTraLog log + trace fusion (3):
  `LogVolumeAnomaly`, `LogTraceTemporalDecorrelation`,
  `LogSeverityEscalation`.
- **Tier 6** — cross-cutting structural motifs (3):
  `SaturationTrending`, `EpisodicTransientSpike`,
  `RegressiveDriftWithRecovery`.
- **Tier 7** — orphan reason-code closure (2; added Phase 5.5):
  `EnvelopeBoundaryApproach` (catches `ReasonCode::BoundaryApproach`),
  `EnvelopeBreach` (catches `ReasonCode::EnvelopeViolation`). Validated
  by `tests::no_orphan_reason_codes_in_canonical_bank`.

Total 32; `MAX_MOTIFS = 64` allows 32 free slots for v0.3 / v0.4
expansion.

---

## Tier 1 — Original framework motifs

### `MotifClass::MemoryLeakDrift`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Min signals | 1 (any) |
| Max signals | unbounded |
| Min duration | 5 windows |
| Max duration | unbounded |
| Drift threshold | 0.6 |
| Slew threshold | 0 |
| Boundary-density threshold | 0.4 |
| Recommended action | `Review` |
| Dashboard hint | Inspect process RSS / heap-used and `gc.duration` over the past hour |
| Taxonomy | IEEE 24765 "memory leak"; A-L-R: latent fault → error |
| Differentiator | `JvmHeapPressure` is a JVM-specific refinement |

### `MotifClass::CascadingTimeoutSlew`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench fault case F-04 (`tadbench_trainticket_F04`) |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Min signals | **3** (multi-service signature) |
| Max signals | unbounded |
| Min duration | 2 windows |
| Max duration | 30 |
| Slew threshold | 0.5 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect the upstream service in the dependency graph emitting the first slew |
| Taxonomy | IEEE 24765 "fault propagation"; A-L-R: error → service-failure |
| Differentiator | `DeploymentRegressionSlew` (single-service, max_correlation=2); `CircuitBreakerOpenShift` (similar slew, but different reason via FT mechanism) |

### `MotifClass::DeploymentRegressionSlew`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench fault case F-11 (`tadbench_trainticket_F11`) |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Min signals | 1 |
| Max signals | **2** (single-service step) |
| Min duration | 1 |
| Max duration | unbounded (sustained at new baseline) |
| Slew threshold | 0.8 |
| Recommended action | `Escalate` |
| Dashboard hint | Correlate the timestamp with the deployment log; consider rollback |
| Taxonomy | IEEE 24765 "regression"; A-L-R: design fault → error |
| Differentiator | `CascadingTimeoutSlew` (multi-service ≥ 3); `EpisodicTransientSpike` (max duration 4) |

### `MotifClass::CacheDegradationGrazing`

| Field | Value |
|-------|-------|
| Reason code | `RecurrentBoundaryGrazing` |
| Provenance | `FrameworkDesign` |
| Boundary-density threshold | 0.5 |
| Drift threshold | 0.3 |
| Min duration | 4 windows |
| Recommended action | `Watch` |
| Dashboard hint | Inspect cache hit-rate and eviction rate of the affected service |
| Taxonomy | IEEE 24765 "performance degradation"; A-L-R: marginal-state error |

### `MotifClass::ConnectionPoolExhaustionDrift`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench fault case F-19 (`tadbench_trainticket_F19`) |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Drift threshold | 0.5 |
| Boundary-density threshold | 0.4 |
| Min duration | 5 windows |
| Recommended action | `Review` |
| Dashboard hint | Inspect connection pool waiting queue + active connections + idle timeout |
| Taxonomy | IEEE 24765 "resource exhaustion"; A-L-R: error build-up |

### `MotifClass::GcPressureOscillation`

| Field | Value |
|-------|-------|
| Reason code | `RecurrentBoundaryGrazing` |
| Provenance | `FrameworkDesign` |
| Slew threshold | 0.2 |
| Boundary-density threshold | 0.4 |
| Max signals | 3 (typically a single JVM or small JVM cluster) |
| Recommended action | `Watch` |
| Dashboard hint | Inspect `gc.collection.count` + `gc.duration` histograms |
| Taxonomy | IEEE 24765 "stop-the-world pause"; A-L-R: transient error |
| Differentiator | `JvmGcPause` is a JVM-specific refinement |

### `MotifClass::ErrorRateEscalation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.7 |
| Min duration | 3 windows |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect HTTP 5xx rate by endpoint and recent deploys / config changes |
| Taxonomy | IEEE 24765 "error escalation"; A-L-R: error → multi-failure regime |

### `MotifClass::DependencySlowdown`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.4 |
| Min duration | 5 windows |
| Recommended action | `Review` |
| Dashboard hint | Inspect the upstream service's latency distribution; check its dependencies |
| Taxonomy | IEEE 24765 "performance degradation upstream"; A-L-R: external fault |
| Differentiator | `NetworkDelayDependencyInflation` is the AIOps-network-delay refinement |

### `MotifClass::ResourceSaturation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.5 |
| Boundary-density threshold | 0.5 |
| Min duration | 5 windows |
| Recommended action | `Review` |
| Dashboard hint | Inspect system resource gauges (CPU%, memory%, disk%) over the past hour |
| Taxonomy | IEEE 24765 "resource saturation"; A-L-R: latent → manifest fault |
| Differentiator | `CpuSaturation`, `DiskIoSaturation` are subsystem-specific refinements; `SaturationTrending` is the asymptotic generalisation |

### `MotifClass::QueueBackpressure`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.6 |
| Min duration | 4 windows |
| Recommended action | `Review` |
| Dashboard hint | Inspect message-queue depth and consumer lag metrics |
| Taxonomy | IEEE 24765 "back-pressure accumulation"; A-L-R: error build-up |

---

## Tier 2 — TADBench / TrainTicket fault cases

### `MotifClass::RetryStormCascade`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench retry-storm fault case |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Min signals | **2** (client + upstream simultaneously) |
| Drift threshold | 0.6 |
| Slew threshold | 0.3 |
| Min duration | 3 |
| Max duration | 60 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect retry-policy parameters and client-side request rate vs upstream success rate |
| Taxonomy | IEEE 24765 "retry-induced amplification"; A-L-R: cascading error |

### `MotifClass::CircuitBreakerOpenShift`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench circuit-breaker fault case |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Slew threshold | 0.6 |
| Min duration | 2 |
| Max signals | 4 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect circuit-breaker state metrics and the underlying service's health |
| Taxonomy | IEEE 24765 "fault tolerance mechanism state change" |

### `MotifClass::DatabaseLockContention`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench db-lock fault case |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Min signals | **2** |
| Drift threshold | 0.5 |
| Slew threshold | 0.2 |
| Min duration | 5 |
| Recommended action | `Review` |
| Dashboard hint | Inspect database lock-wait stats and slow-query log for the active transactions |
| Taxonomy | IEEE 24765 "concurrency fault"; A-L-R: synchronisation error |

### `MotifClass::AuthenticationFailureSpike`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | TADBench auth-fail fault case |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Slew threshold | 0.5 |
| Min duration | 1 |
| Max duration | 30 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect auth-service health, token-issuance rate, and 401/403 distribution |
| Taxonomy | IEEE 24765 "authentication subsystem failure" |

### `MotifClass::ConfigDriftRegression`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | TrainTicket-Anomaly version-config fault class |
| Evidence DOI | `10.5281/zenodo.6979726` |
| Slew threshold | 0.6 |
| Max signals | 3 |
| Recommended action | `Escalate` |
| Dashboard hint | Diff config artefacts between the last good window and the current; consider rollback |
| Taxonomy | IEEE 24765 "configuration regression"; A-L-R: design-time fault |

---

## Tier 3 — AIOps Challenge categories

### `MotifClass::PacketLossErrorEscalation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `packet_loss` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Min signals | **2** (multi-service signature) |
| Drift threshold | 0.6 |
| Min duration | 3 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect TCP retransmits / packet-loss counters at the network layer |
| Taxonomy | IEEE 24765 "communication failure (lower layer)" |

### `MotifClass::NetworkDelayDependencyInflation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `network_delay` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Min signals | **2** |
| Drift threshold | 0.4 |
| Min duration | 5 |
| Recommended action | `Review` |
| Dashboard hint | Inspect inter-service RTT histograms and link-level latency gauges |
| Taxonomy | IEEE 24765 "communication-path performance fault" |

### `MotifClass::DiskIoSaturation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `disk_exhaustion` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Drift threshold | 0.5 |
| Boundary-density threshold | 0.5 |
| Min duration | 4 |
| Recommended action | `Review` |
| Dashboard hint | Inspect disk IOPS / await / queue depth; check for runaway log writes |
| Taxonomy | IEEE 24765 "storage subsystem saturation" |

### `MotifClass::CpuSaturation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `cpu_exhaustion` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Drift threshold | 0.5 |
| Boundary-density threshold | 0.5 |
| Min duration | 4 |
| Recommended action | `Review` |
| Dashboard hint | Inspect CPU utilisation, run-queue length, and thread-level scheduling latency |
| Taxonomy | IEEE 24765 "compute resource saturation" |

### `MotifClass::JvmHeapPressure`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `memory_exhaustion` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Drift threshold | 0.6 |
| Min duration | 5 |
| Recommended action | `Review` |
| Dashboard hint | Inspect `jvm.memory.heap.used` + `gc.collection.count` + minor/major GC ratio |
| Taxonomy | IEEE 24765 "memory leak (JVM-specific)"; refines `MemoryLeakDrift` |

### `MotifClass::JvmGcPause`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | AIOps Challenge `jvm_resource_exhaustion` category |
| Evidence DOI | `AIOps-Challenge-2020-2021` |
| Slew threshold | 0.4 |
| Boundary-density threshold | 0.3 |
| Max signals | 3 |
| Max duration | 10 |
| Recommended action | `Watch` |
| Dashboard hint | Inspect `gc.duration` percentiles + STW pause histograms; consider GC tuning |
| Taxonomy | IEEE 24765 "stop-the-world pause (JVM-specific)"; refines `GcPressureOscillation` |

---

## Tier 4 — MultiDim-Localization patterns

### `MotifClass::ServiceGraphDriftPropagation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | MultiDim-Localization root-cause-graph cases |
| Evidence DOI | `MultiDimension-Localization-NetManAIOps` |
| Min signals | **4** |
| Drift threshold | 0.5 |
| Slew threshold | 0.2 |
| Min duration | 5 |
| Recommended action | `Escalate` |
| Dashboard hint | Walk the service-call graph from the originating service; identify the propagation root |
| Taxonomy | IEEE 24765 "graph-structured fault propagation" |

### `MotifClass::HighDimAnomalyCluster`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | MultiDim-Localization high-dim cluster cases |
| Evidence DOI | `MultiDimension-Localization-NetManAIOps` |
| Min signals | **6** (compound fault signature) |
| Drift threshold | 0.4 |
| Min duration | 4 |
| Recommended action | `Review` |
| Dashboard hint | Inspect the multi-metric anomaly cluster as a unit; no single metric is dominant |
| Taxonomy | IEEE 24765 "compound fault signature" |

### `MotifClass::MetricCorrelationCollapse`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | MultiDim-Localization correlation-collapse cases |
| Evidence DOI | `MultiDimension-Localization-NetManAIOps` |
| Min signals | **2** |
| Slew threshold | 0.4 |
| Min duration | 3 |
| Recommended action | `Review` |
| Dashboard hint | Compare current pairwise metric correlations against the baseline correlation matrix |
| Taxonomy | IEEE 24765 "structural model invalidation" |

---

## Tier 5 — DeepTraLog log + trace fusion

### `MotifClass::LogVolumeAnomaly`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | DeepTraLog log-volume anomalies |
| Evidence DOI | `DeepTraLog-ICSE-2022` |
| Drift threshold | 0.5 |
| Min duration | 3 |
| Recommended action | `Review` |
| Dashboard hint | Inspect log-volume gauges per severity per service over the past hour |
| Taxonomy | IEEE 24765 "diagnostic-output anomaly" |

### `MotifClass::LogTraceTemporalDecorrelation`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `DatasetObserved` |
| Evidence | DeepTraLog log-trace temporal-mismatch cases |
| Evidence DOI | `DeepTraLog-ICSE-2022` |
| Slew threshold | 0.3 |
| Min duration | 3 |
| Recommended action | `Review` |
| Dashboard hint | Inspect log-event timestamps vs trace-span timestamps for the same request IDs |
| Taxonomy | IEEE 24765 "instrumentation-temporal divergence" |

### `MotifClass::LogSeverityEscalation`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `DatasetObserved` |
| Evidence | DeepTraLog severity-shift cases |
| Evidence DOI | `DeepTraLog-ICSE-2022` |
| Drift threshold | 0.6 |
| Min duration | 3 |
| Recommended action | `Escalate` |
| Dashboard hint | Inspect log-severity distribution histograms; compare against the healthy-window baseline |
| Taxonomy | IEEE 24765 "diagnostic severity escalation" |

---

## Tier 6 — Cross-cutting structural motifs

### `MotifClass::SaturationTrending`

| Field | Value |
|-------|-------|
| Reason code | `SustainedOutwardDrift` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.5 |
| Slew threshold | 0.1 |
| Boundary-density threshold | 0.5 |
| Min duration | 6 |
| Recommended action | `Watch` |
| Dashboard hint | Project the current drift forward to estimate time-to-ceiling; consider scaling |
| Taxonomy | IEEE 24765 "asymptotic resource saturation" |
| Differentiator | Generalises `ResourceSaturation`, `DiskIoSaturation`, `CpuSaturation` |

### `MotifClass::EpisodicTransientSpike`

| Field | Value |
|-------|-------|
| Reason code | `AbruptSlewViolation` |
| Provenance | `FrameworkDesign` |
| Slew threshold | 0.5 |
| Min duration | 1 |
| Max duration | **4** (transient-only) |
| Recommended action | `Watch` |
| Dashboard hint | Note the timestamp; correlate with cron / scheduled jobs / external triggers |
| Taxonomy | IEEE 24765 "transient-only error"; A-L-R: transient fault |

### `MotifClass::RegressiveDriftWithRecovery`

| Field | Value |
|-------|-------|
| Reason code | `DriftWithRecovery` |
| Provenance | `FrameworkDesign` |
| Drift threshold | 0.4 |
| Min duration | 4 |
| Max duration | 30 |
| Recommended action | `Watch` |
| Dashboard hint | No action required if recovery confirmed; record as a near-miss for trend analysis |
| Taxonomy | IEEE 24765 "self-healing transient drift" |

---

## Tier 7 — Orphan reason-code closure (added Phase 5.5)

These two motifs were added in Phase 5.5 to close the orphan reason-code
coverage gap. Before Phase 5.5, `ReasonCode::BoundaryApproach` and
`ReasonCode::EnvelopeViolation` had no canonical bank entry — episodes
matching them silently fell through to `SemanticDisposition::Unknown`.
The property test `tests::no_orphan_reason_codes_in_canonical_bank`
caught the gap; these two motifs close it.

### `MotifClass::EnvelopeBoundaryApproach`

| Field | Value |
|-------|-------|
| Reason code | `BoundaryApproach` |
| Provenance | `FrameworkDesign` |
| Recommended action | `Watch` |
| Affinity tiers | A, I, J, X, Y |
| Primary witness tiers | A, I, J |
| Named witness detectors | `scalar_threshold_3sigma`, `mann_kendall`, `theil_sen_residual` |
| Confuser | `EnvelopeBreach` (margin gate 0.10) |
| Dashboard hint | Note the timestamp; if recurrence is observed in subsequent windows escalate to CacheDegradationGrazing |
| Taxonomy | IEEE 24765 "marginal-state transient"; A-L-R: dormant fault |

### `MotifClass::EnvelopeBreach`

| Field | Value |
|-------|-------|
| Reason code | `EnvelopeViolation` |
| Provenance | `FrameworkDesign` |
| Recommended action | `Escalate` |
| Affinity tiers | A, B, R, E, X, Y |
| Primary witness tiers | A, B, R |
| Named witness detectors | `scalar_threshold_3sigma`, `cusum`, `page_hinkley` |
| Confuser | `EnvelopeBoundaryApproach` (margin gate 0.10) |
| Dashboard hint | Inspect SLO/SLA threshold + the affected service's value distribution; threshold may be too tight |
| Taxonomy | IEEE 24765 "threshold breach (smooth)"; A-L-R: error → manifest |

---

## Cross-motif discriminator notes

- `CascadingTimeoutSlew` vs `DeploymentRegressionSlew`: same reason
  code (`AbruptSlewViolation`); discriminated by `min_correlation_count`
  (3 vs 1) and `max_correlation_count` (unbounded vs 2). Verified by
  `tests::cascading_timeout_requires_multi_service_correlation` and
  `tests::deployment_regression_requires_single_service_step` in
  [`src/heuristics_bank.rs`](../src/heuristics_bank.rs).
- `MemoryLeakDrift` vs `JvmHeapPressure`: same reason and similar
  weights; the JVM-specific entry is `DatasetObserved` (AIOps Challenge)
  and wins the provenance tie-breaker on JVM evidence.
- `GcPressureOscillation` vs `JvmGcPause`: GcPressureOscillation has
  reason `RecurrentBoundaryGrazing` (sustained); JvmGcPause has reason
  `AbruptSlewViolation` (single-shot pause). Discriminated by reason
  code, not by gates.
- `ResourceSaturation` vs `CpuSaturation`/`DiskIoSaturation`: identical
  thresholds; the AIOps refinements (`DatasetObserved`) win the
  provenance tie-breaker on AIOps evidence; on generic-evidence
  episodes, `ResourceSaturation` (`FrameworkDesign`) wins by lower
  index.
- `EpisodicTransientSpike` vs `DeploymentRegressionSlew`: same reason
  (`AbruptSlewViolation`); discriminated by `max_duration_windows`
  (4 vs unbounded). Verified by
  `tests::transient_spike_excludes_long_duration`.

## Reproducibility

Every entry's `evidence_dataset` field is grep-able against
`data/MANIFEST.toml` block names. Every `evidence_dataset_doi` field
either matches a vendored manifest's `upstream_doi` or is
`"FrameworkDesign"`/empty for hand-coded entries. To extend the bank:

1. Add the `MotifClass` variant in `src/types.rs` with a doc-comment
   citing IEEE 24765 / A-L-R terminology.
2. Append a `HeuristicEntry` literal to
   `HeuristicsBank::with_canonical_motifs` in
   `src/heuristics_bank.rs`. Populate every field; do not leave
   placeholders.
3. Add a unit test asserting a literal episode-feature vector lands on
   the new `MotifClass`.
4. Add a corresponding section to this document with the same
   schema as the entries above.
5. If the new entry is `DatasetObserved`, ensure the cited fixture is
   manifest-tracked in `data/MANIFEST.toml`.
