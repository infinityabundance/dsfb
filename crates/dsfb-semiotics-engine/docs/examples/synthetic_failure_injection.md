# Synthetic Failure Injection Example

The example `examples/synthetic_failure_injection.rs` demonstrates a minimal deterministic interpretation trace.

## Run

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example synthetic_failure_injection
```

## What It Does

- generates a nominal sine-like signal
- injects a linear drift term after a nominal period
- pushes the resulting residual stream through the bounded online engine
- prints time-stamped syntax, grammar, trust, and semantic changes

Representative output is intentionally plain-English and operator-readable. The exact heuristic wording depends on the currently selected bank configuration.

## Why It Exists

This example is meant to show the live deterministic interpretation path without requiring the full artifact bundle or notebook workflow. It is illustrative only.
