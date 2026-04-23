# Security Policy — `dsfb-rf`

## Scope

`dsfb-rf` is a `#![no_std]` / `#![forbid(unsafe_code)]` observer library for
residual streams produced by upstream RF signal-processing chains (matched
filters, AGC loops, channel estimators, tracking loops, beamformers,
scheduler telemetry). The crate computes structural semiotics on residuals
that its caller has already produced; it does **not** open sockets, spawn
threads, read untrusted input, interact with hardware, or execute dynamic
code. Its attack surface is scoped to the typed data its caller hands it.

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x: (pre-release)  |

Security fixes land on the `main` branch and ship as a patch release.

## Reporting a Vulnerability

Report privately to **riaan@invariantforge.net**. Please include:

1. Affected version and commit SHA.
2. Minimal reproducer (feature flags, input shape, observed output).
3. The specific behaviour you believe is unsafe or wrong.

We acknowledge receipt within 7 days and publish a coordinated advisory once
a patched release is out.

## Hardening Commitments

- `#![forbid(unsafe_code)]` at the crate root; no `unsafe` blocks.
- `#![no_std]` by default; `alloc` and `std` are behind feature flags.
- No filesystem, network, or process boundaries inside the library.
- Deterministic, panic-free core path (Q16.16 fixed-point, `saturating_*`
  arithmetic, bounded FSM); Kani harnesses gate the panic-freedom claim on
  the grammar / envelope / DSA / fixed-point paths.
- No dynamic code loading. Optional `hdf5_loader` feature pulls
  `hdf5-metno` (system `libhdf5` required); that dependency's FFI surface
  is the only native boundary in the crate and is exercised exclusively
  by the `paper-lock` calibration binary.
- Dev and example binaries may call `.unwrap()`/`.expect()` on
  calibration preconditions — these live outside the library surface and
  are not linked into `dsfb-rf` library consumers.

## Out of Scope

- Claims about runtime side-channels of the upstream producer (AGC, PLL,
  beamformer, etc.) — those are the producer's responsibility.
- Claims about the correctness of user-supplied calibration windows.
- Modulation-recognition, device-fingerprinting, link-budget, spoofing,
  beam-selection, or scheduling benchmarks — none are reproduced here
  (see `paper/dsfb_rf_v2.tex` §L13–L22 for the full honesty disclosure).
