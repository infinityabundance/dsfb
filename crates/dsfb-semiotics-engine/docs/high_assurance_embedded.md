# High-Assurance Embedded Notes

This note records architectural hardening relevant to embedded and high-assurance review. It does not claim certification, hard-real-time guarantees, or full `no_std` readiness.

## Fixed-Point Scope

The crate now exposes an experimental `numeric-fixed` backend for the bounded live path.

- Backend label: `fixed_q16_16`
- Quantization: deterministic `q16.16`
- Overflow policy: saturating ingress quantization
- Current covered scope:
  - bounded live residual ingress
  - bounded live residual storage
  - deterministic live-path replay and snapshot/replay tooling
  - downstream sign/syntax/grammar/semantics evaluation over the quantized residual values

What this does **not** mean:

- it is not a full fixed-point rewrite of the offline report pipeline
- it is not a claim of FPGA or ASIC deployment readiness by itself

Equivalence framing:

- the tested claim is classification consistency within documented scope
- fixed-point comparisons should be read as "same syntax / grammar / semantic disposition under the
  tested fixtures within quantization tolerance," not as universal numerical identity
- precision bounds and quantization tradeoffs must therefore remain documented alongside any
  fixed-point demo or timing result

## Safety-First Smoothing

The named `safety_first` profile is a conservative two-stage causal preconditioner for derivative estimation: a bounded moving average followed by a low-alpha exponential smoother.

- intent: attenuate jitter before finite differencing
- causal window: exported in run metadata
- estimated centroid lag: exported in report metadata as samples
- maximum settling horizon: exported in report metadata as samples

Guidance note:

- use the lag numbers as integration aids for downstream guidance or gating logic
- do not treat them as closed-loop stability guarantees

## Allocation Policy

Current allocation policy for the bounded live path:

- initialization-time heap allocation: allowed
- fixed-capacity ring buffer allocation: occurs during initialization only
- offline history accumulation: optional and explicitly separate from the bounded online path
- interface wrappers such as owned status strings and artifact exports remain allocation-bearing

This is a stronger deployment-oriented split, not a claim of whole-crate zero-allocation runtime.

## Timing Determinism Reporting

Use:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-timing-determinism
```

The generated report records:

- median
- p95
- p99
- p99.9
- maximum observed time
- jitter summary
- measured platform metadata

These are observed host-side measurements only.

## State-Exact Replay

The bounded live engine can serialize a versioned snapshot, reload it, and replay exactly one more sample under the same numeric backend and settings.

Support workflow:

1. capture a live snapshot from the bounded engine
2. save the binary snapshot
3. reproduce the next sample transition with `dsfb-state-replay`
4. inspect the resulting syntax / grammar / semantics / trust output

This is a state-exact replay aid under documented build and numeric conditions. It is not a claim of bit-exact equality across every platform and toolchain.

## Formal Verification Scope

Kani harnesses currently target:

- trust scalar range clamping
- trust scalar non-NaN behavior
- closed grammar-reason enum coverage

That is a bounded proof scope only. The full engine is not formally verified.

## Supply-Chain Review

The crate also carries lightweight audit-readiness tooling for dependency review:

- `deny.toml` for `cargo deny`
- documented `cargo audit` usage in the crate-local workflow mirrors
- [`docs/REQUIREMENTS.md`](REQUIREMENTS.md) for claim-to-code and claim-to-test mapping

This is intended to improve technical-data-package readiness. It is not a formal compliance claim.
