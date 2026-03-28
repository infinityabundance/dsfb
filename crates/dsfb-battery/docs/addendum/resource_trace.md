# Resource Integrity & Audit

The `dsfb-battery` crate now includes an opt-in Resource Integrity & Audit helper path for host-side resource inspection. This path is additive and disabled by default. It does not modify the existing mono-cell production pipeline, production Stage II JSON artifact, or production figure generation path.

## What It Measures

The helper emits a separate `resource_trace.json` artifact describing:

- input and configuration hashes
- fixed window and persistence settings used for the traced run
- logical sample/update rate for the current dataset path
- host-side timing summaries for the current batch evaluation helper
- heuristics-bank lookup cost and bank size
- key runtime struct sizes via `size_of`
- hot-loop state footprint estimates
- a validation hash for the emitted resource trace

## Measurement Modes

The resource trace distinguishes between:

- `measured`: direct host timing or exact file-size values
- `estimated`: values derived from conservative footprint formulas
- `asserted`: values declared from fixed configuration or known execution shape
- `inferred`: values computed from measured or exact counts
- `not_measured`: values intentionally left unavailable rather than fabricated

Examples:

- host timing is `measured`
- hot-loop state bytes are `estimated`
- logical sample rate is `asserted`
- average evaluated heuristics per cycle is `inferred`
- exact heap allocation count and exact stack usage are `not_measured`

## Enablement

The helper is opt-in through a separate binary:

```bash
cargo run --bin dsfb-battery-resource-trace
```

It can also be pointed at a specific CSV and configuration:

```bash
cargo run --bin dsfb-battery-resource-trace -- \
  --data data/nasa_b0005_capacity.csv \
  --timing-repeats 5 \
  --drift-window 5 \
  --drift-persistence 12 \
  --slew-persistence 8
```

By default, outputs are written to a timestamped directory under:

```text
outputs/resource_trace/...
```

## Integrity Scope

The emitted hashes support artifact integrity and reproducibility checks for the traced run. They do not prove physical correctness, target-MCU timing, or certified WCET.

## Production Path Protection

- the helper is separate from `dsfb-battery-demo`
- resource tracing is not part of the default hot path
- production Stage II artifact filenames are not reused
- production figure filenames are not reused
