# Motif Grammar

The grammar emits `Episode` objects when a residual stream's structural
behaviour matches one of five named patterns. Each pattern is a finite-state
machine over a single residual channel.

## Five Motif Classes

| `MotifClass`                      | Residual class it consumes | State machine file                                          |
|-----------------------------------|----------------------------|-------------------------------------------------------------|
| `PlanRegressionOnset`             | `PlanRegression`           | [src/grammar/motifs.rs](../src/grammar/motifs.rs)           |
| `CardinalityMismatchRegime`       | `Cardinality`              | [src/grammar/motifs.rs](../src/grammar/motifs.rs)           |
| `ContentionRamp`                  | `Contention`               | [src/grammar/motifs.rs](../src/grammar/motifs.rs)           |
| `CacheCollapse`                   | `CacheIo`                  | [src/grammar/motifs.rs](../src/grammar/motifs.rs)           |
| `WorkloadPhaseTransition`         | `WorkloadPhase`            | [src/grammar/motifs.rs](../src/grammar/motifs.rs)           |

## Admissibility Envelope

Each motif defines a `(drift, slew)` pair in
[src/grammar/envelope.rs](../src/grammar/envelope.rs).

- `drift` is the long-horizon EMA deviation threshold.
- `slew` is the short-horizon slope threshold (residual per unit time).

A sample is `Admissible` if both `|ema - 0| ≤ drift` and
`|slope| ≤ slew`. Otherwise it is `DriftViolating`, `SlewViolating`, or
both.

## State Machine

```
                     ┌───────────┐
      admissible ──▶ │  Resting  │ ──── drift or slew violation ──┐
                     └───────────┘                                 │
                          ▲                                         ▼
                          │                              ┌──────────────────┐
                          │                              │  Observing       │
      back admissible ────┘  ◀──── below minimum dwell ──┤  (collecting)    │
                                                         └──────────────────┘
                                                                 │
                                            dwell ≥ minimum + peak found
                                                                 ▼
                                                         ┌──────────────────┐
                                                         │  Emit Episode    │
                                                         └──────────────────┘
```

`advance(&mut self, sample) -> Option<Transition>` is the only public
method. Transitions are returned by value — the engine observes them and
emits episodes when a `(start, peak, end)` triple is complete.

## Minimum-Dwell Rule

An episode is only emitted if the observing state persisted for at least
`min_dwell` samples. The dwell rule exists to suppress one-sample spikes
that would inflate episode counts on noisy telemetry. Default dwell values
per motif are listed in the `envelope.rs` table.

## Episode Payload

```rust
pub struct Episode {
    pub motif: MotifClass,
    pub channel: String,          // opaque adapter identifier
    pub t_start: f64,             // seconds since trace origin
    pub t_end: f64,
    pub t_peak: f64,
    pub peak: f64,                // signed residual at t_peak
    pub ema_at_boundary: f64,     // EMA value at transition into Observing
}
```

The `Episode` tuple is what `paper_episode_fingerprint_is_pinned` hashes.

## Why No Recursion, No Global State

State machines are implemented as explicit `enum` variants with a
dispatch table in `advance`. There is no recursion, no `Box<dyn State>`,
no interior mutability. A reviewer can read `motifs.rs` top-to-bottom
and see every transition.
