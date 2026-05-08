# Confuser-pair utilisation audit

For each declared {motif, confuser} pair in the canonical bank,
the table reports: observed competition count (runner-up matched
the declared confuser), confuser-margin-gate firings (margin fell
below `min_margin_vs_confuser`, surfacing as `ConfuserAmbiguous`),
and the count of fixtures where this motif typed at all.

Source: Phase ζ.7 audit harness (`src/audit/confuser_audit.rs`).
Data: aggregated `EpisodeTypingObservation` records over all 12
vendored fixtures.

Sorted by observed competition (most-utilised first).
Pairs with 0 observed competitions are theatre on the current
12-fixture surface; partner-data may activate them.

| Motif | Confuser | Observed competitions | Confuser-ambig events | Fixtures typed |
|-------|----------|---------------------:|---------------------:|---------------:|
| `AuthenticationFailureSpike` | `EpisodicTransientSpike` | 2 | 1 | 1 |
| `DeploymentRegressionSlew` | `CircuitBreakerOpenShift` | 1 | 0 | 2 |
| `ConfigDriftRegression` | `DeploymentRegressionSlew` | 1 | 1 | 1 |
| `MemoryLeakDrift` | `ConnectionPoolExhaustionDrift` | 0 | 0 | 0 |
| `CascadingTimeoutSlew` | `DependencySlowdown` | 0 | 0 | 1 |
| `CacheDegradationGrazing` | `GcPressureOscillation` | 0 | 0 | 0 |
| `ConnectionPoolExhaustionDrift` | `MemoryLeakDrift` | 0 | 0 | 0 |
| `GcPressureOscillation` | `CacheDegradationGrazing` | 0 | 0 | 0 |
| `ErrorRateEscalation` | `PacketLossErrorEscalation` | 0 | 0 | 0 |
| `DependencySlowdown` | `CascadingTimeoutSlew` | 0 | 0 | 0 |
| `ResourceSaturation` | `ConnectionPoolExhaustionDrift` | 0 | 0 | 0 |
| `QueueBackpressure` | `ConnectionPoolExhaustionDrift` | 0 | 0 | 0 |
| `RetryStormCascade` | `CascadingTimeoutSlew` | 0 | 0 | 0 |
| `CircuitBreakerOpenShift` | `DeploymentRegressionSlew` | 0 | 0 | 0 |
| `DatabaseLockContention` | `DependencySlowdown` | 0 | 0 | 0 |
| `PacketLossErrorEscalation` | `ErrorRateEscalation` | 0 | 0 | 0 |
| `NetworkDelayDependencyInflation` | `DependencySlowdown` | 0 | 0 | 0 |
| `DiskIoSaturation` | `CpuSaturation` | 0 | 0 | 0 |
| `CpuSaturation` | `DiskIoSaturation` | 0 | 0 | 0 |
| `JvmHeapPressure` | `MemoryLeakDrift` | 0 | 0 | 0 |
| `JvmGcPause` | `AuthenticationFailureSpike` | 0 | 0 | 0 |
| `ServiceGraphDriftPropagation` | `DependencySlowdown` | 0 | 0 | 0 |
| `HighDimAnomalyCluster` | `MetricCorrelationCollapse` | 0 | 0 | 0 |
| `MetricCorrelationCollapse` | `HighDimAnomalyCluster` | 0 | 0 | 0 |
| `LogVolumeAnomaly` | `LogSeverityEscalation` | 0 | 0 | 0 |
| `LogTraceTemporalDecorrelation` | `ServiceGraphDriftPropagation` | 0 | 0 | 0 |
| `LogSeverityEscalation` | `LogVolumeAnomaly` | 0 | 0 | 0 |
| `SaturationTrending` | `ConnectionPoolExhaustionDrift` | 0 | 0 | 0 |
| `EpisodicTransientSpike` | `AuthenticationFailureSpike` | 0 | 0 | 0 |
| `RegressiveDriftWithRecovery` | `MemoryLeakDrift` | 0 | 0 | 0 |
| `EnvelopeBoundaryApproach` | `EnvelopeBreach` | 0 | 0 | 0 |
| `EnvelopeBreach` | `EnvelopeBoundaryApproach` | 0 | 0 | 3 |
