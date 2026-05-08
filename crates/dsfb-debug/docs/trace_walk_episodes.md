# DSFB-Debug — Trace-Walk Episodes (worked examples)

This document walks three TADBench / TrainTicket fault-directory slices
through the full DSFB-Debug pipeline at the level a production debug
engineer can reproduce step-by-step on the vendored fixtures. The pattern
follows the SECOM §10.7 trace-walk format from the dsfb-semiconductor
paper: input spans → per-service residual → sign tuple → grammar state →
motif lookup → episode aggregation → developer-facing output.

## Status

These walks are **schema-faithful and fixture-faithful**: every per-
fixture statement maps directly to a real upstream span aggregation as
documented in [`../data/MANIFEST.toml`](../data/MANIFEST.toml). The
**numerical values** are the verbatim output of `cargo test --features
"std paper-lock" -- --nocapture` against the populated real-bytes
fixtures. No values are hand-fabricated.

## Notation

Each per-window per-signal table row uses the column order:

| `w` | `s` | `service` | `metric` | `obs` | `r` | `‖r‖` | `ṙ` | `r̈` | `state` | `reason` | `motif` | `policy` |

Where:
- `w` — window index
- `s` — signal index
- `service` — TrainTicket microservice the metric was projected from
- `metric` — `latency_p50_ms` or `error_rate`
- `obs` — observed value
- `r = obs − baseline_mean[s]` — residual (per `src/residual.rs`)
- `‖r‖` — residual norm (per `src/residual.rs:residual_norm`)
- `ṙ`, `r̈` — finite-difference drift and slew (per `src/sign.rs:compute_sign_tuple`)
- `state` — confirmed grammar state (per `src/grammar.rs`)
- `reason` — reason code (per `src/types.rs:ReasonCode`)
- `motif` — heuristics-bank match (per `src/heuristics_bank.rs`)
- `policy` — developer-facing policy state (per `src/policy.rs`)

---

## Episode 1 — F-04 (admin-service springstarter version-config regression)

**Manifest:** `tadbench_trainticket_F04` (DOI `10.5281/zenodo.6979726`)
**Upstream fault directory:** `ts-admin-basic-info-service-sprintstarterweb_1.5.22`
**Fixture properties:** 25 316 real Jaeger spans projected to 30 windows × 12 signals (top-6 services × `{latency_p50_ms, error_rate}`); 15-second windows; first 12 windows are the healthy baseline.
**Top services:** `ts-travel-service`, `ts-basic-service`, `ts-ticketinfo-service`, `ts-station-service`, `ts-seat-service`, `ts-route-service`.

### Engine output (verbatim)

```
"manifest_name": "tadbench_trainticket_F04",
"deterministic_replay_holds": true,
"episode_count": 0,
"raw_anomaly_count": 0,
"dsfb_episode_count": 0,
"rscr": 0,
"fault_recall": 1,
"clean_window_false_episode_rate": 0
```

### Reading

The engine produces **zero typed structural episodes** on this fixture.
This is the engine working as designed: the projected upstream fault
directory is a *steady-state regressed-config run* — the entire run is
the regression relative to a different config — not a fault-injection
log with per-window transitions. With no per-window fault transitions
in the residual matrix, the grammar pipeline never escalates beyond
`Admissible`, the bank is never invoked at episode close, and the
operator surface remains empty.

This is the bank's anti-hallucination behaviour on a steady-state slice:
when the residuals do not contain a structural transition the engine
declines to type. `fault_recall = 1.0` is *vacuously* satisfied (the
fixture carries no per-window fault labels, so there are no faults to
miss); `clean_window_false_episode_rate = 0` is the operator-conservative
result. Theorem 9 deterministic replay holds across the empty episode
set.

### Developer-facing message

> No typed structural episodes on this fixture. The slice is a steady-
> state regressed-config run; the bank correctly emits no signal. To
> see DSFB-Debug type a deployment-regression motif, exercise the
> F-11 fixture (Episode 2 below).

---

## Episode 2 — F-11 (order-service mongodb 4.2.2 deployment regression)

**Manifest:** `tadbench_trainticket_F11` (DOI `10.5281/zenodo.6979726`)
**Upstream fault directory:** `ts-order-service_mongodb_4.2.2_2022-07-12`
**Fixture properties:** 35 604 real Jaeger spans projected to 431 windows × 16 signals (top-8 services × `{latency_p50_ms, error_rate}`); 15-second windows; first 172 windows are the healthy baseline.
**Top services:** `ts-travel-service`, `ts-basic-service`, `ts-ticketinfo-service`, `ts-station-service`, `ts-seat-service`, `ts-route-service`, `ts-train-service`, `ts-order-service`.

### Engine output — three closed structural episodes (verbatim)

```
"manifest_name": "tadbench_trainticket_F11",
"deterministic_replay_holds": true,
"episode_count": 3,
"raw_anomaly_count": 11,
"dsfb_episode_count": 3,
"rscr": 3.6666666666666665,
"fault_recall": 1,
"clean_window_false_episode_rate": 0.0069605568445475635
```

### Per-episode evidence packet at default 9-axis fusion config (N≥3)

Verbatim from `tests/fusion_compare.rs` Phase-2 tier-affinity diagnostic
output:

| Episode | Top motif | Top score | Runner-up motif (score) | Margin | tier_consensus_factor | Confuser motif (score) | margin_vs_confuser | Disposition |
|--------:|-----------|----------:|--------------------------|-------:|----------------------:|------------------------|-------------------:|:-----------:|
| 0 | `DeploymentRegressionSlew` | 12 920.22 | `CircuitBreakerOpenShift` (9 261.07) | 0.283 | 0.750 | `CircuitBreakerOpenShift` (9 261.07) | 0.283 | typed |
| 1 | `AuthenticationFailureSpike` | 2.29 | `EpisodicTransientSpike` (1.85) | 0.192 | 0.500 | `EpisodicTransientSpike` (1.85) | 0.192 | typed |
| 2 | `AuthenticationFailureSpike` | 2.70 | `EpisodicTransientSpike` (2.53) | 0.065 | 0.333 | `EpisodicTransientSpike` (2.53) | 0.065 | confuser_ambiguous |

### Reading

- **Episode 0** is the production-relevant typed result: the order-
  service deployment regression surfaces as `DeploymentRegressionSlew`
  with a confident margin (0.283) against its declared confuser
  (`CircuitBreakerOpenShift`, also a step-shaped single-service motif
  sharing the `AbruptSlewViolation` reason code). The
  `tier_consensus_factor` 0.750 indicates that 3 of the 4 affinity-mask
  tiers fired in support of the typing — the Routed Evidence Principle
  in action.
- **Episode 1** types as `AuthenticationFailureSpike` confidently
  (margin 0.192, above the default 0.10 confuser-margin gate).
- **Episode 2** is the anti-hallucination ladder catching a weak
  typing: margin against the declared confuser is only 0.065 (below
  the 0.10 default gate), so the harness emits `confuser_ambiguous`
  rather than committing to the top motif. The operator surface
  carries both motifs and their scores; the operator decides.

The fusion sweep at consensus N≥7 reduces this to 1 typed-confirmed
episode at Layer-2 false-positive rate 0.0023 — 3× lower than DSFB-
structural alone, ~12× lower than EWMA, ~6× lower than scalar-3σ.

### Developer-facing message

> Deployment-regression pattern detected on the order-service mongodb
> 4.2.2 redeployment. Three structural episodes; episode 0 confidently
> typed `DeploymentRegressionSlew`; episode 2 declined to type and
> surfaced both `AuthenticationFailureSpike` and its confuser
> `EpisodicTransientSpike`. Recommended action: correlate episode 0
> with the deployment timestamp; investigate episode 2 manually given
> the ambiguous typing.

---

## Episode 3 — F-19 (mongodb-driver-3.0.4 version-config regression)

**Manifest:** `tadbench_trainticket_F19` (DOI `10.5281/zenodo.6979726`)
**Upstream fault directory:** `ts-order-service_3.0.4-mongodb-driver_2022-07-13`
**Fixture properties:** 19 281 real Jaeger spans projected to 30 windows × 12 signals (top-6 services × `{latency_p50_ms, error_rate}`); 15-second windows; first 12 windows are the healthy baseline.
**Top services:** `ts-basic-service`, `ts-travel-service`, `ts-ticketinfo-service`, `ts-station-service`, `ts-seat-service`, `ts-travel2-service`.

### Engine output (verbatim)

```
"manifest_name": "tadbench_trainticket_F19",
"deterministic_replay_holds": true,
"episode_count": 0,
"raw_anomaly_count": 0,
"dsfb_episode_count": 0,
"rscr": 0,
"fault_recall": 1,
"clean_window_false_episode_rate": 0
```

### Reading

Same disposition as Episode 1 (F-04): the engine produces zero typed
structural episodes because the projected upstream fault directory is a
steady-state regressed-config run rather than a fault-injection log
with per-window transitions. The `mongodb-driver-3.0.4` version is a
distinct fault profile from F-11's `mongodb 4.2.2`, but as a steady-
state slice it looks structurally identical: a flat residual matrix
producing no grammar transitions.

The two steady-state fixtures (F-04, F-19) and the typed-episode
fixture (F-11) together exercise both branches of the engine's
operator-conservative discipline:

- **Typed branch (F-11)**: residuals contain real structural transitions
  → engine produces typed episodes with full evidence packets.
- **Silent branch (F-04, F-19)**: residuals are steady-state → engine
  declines to type, returning zero episodes rather than synthesising a
  structural narrative that the data does not support.

### Developer-facing message

> No typed structural episodes on this fixture. The slice is a steady-
> state regressed-config run for mongodb-driver-3.0.4; the bank
> correctly emits no signal. The fixture is included in the harness
> as a cross-fixture invariant validator — Theorem 9 holds, no
> hallucinated typing.

---

## Cross-fixture summary

| Fixture | Upstream fault dir | Spans | Episodes | Disposition |
|---------|-------------------|------:|---------:|-------------|
| F-04 | admin-service springstarter 1.5.22 | 25 316 | 0 | silent (steady-state) |
| F-11 | order-service mongodb 4.2.2 | 35 604 | 3 | typed (1 typed, 1 typed, 1 confuser_ambiguous) |
| F-19 | order-service mongodb-driver-3.0.4 | 19 281 | 0 | silent (steady-state) |

Three slices, three fixture-bound behaviours; one engine, one bank, one
config. The walk-throughs above are reproducible by running:

```
cargo test --features "std paper-lock" --test eval_tadbench -- --nocapture
cargo test --features "std paper-lock" --test fusion_compare -- --nocapture
```

The verbatim test stdout is the source of every number in this document.
