# SBIR readiness

This document tracks the Technology Readiness Level (TRL) and
SBIR-relevant deliverables of the `dsfb-robotics` crate and its
companion paper. DSFB's robotics application is targeted at SBIR
Phase I / II opportunities in robot-health-monitoring, predictive
maintenance, and safety-rated operator-review-surface compression.

## Technology Readiness Level

Per the NASA / DoD TRL definitions:

| TRL | Description | Crate status |
|---|---|---|
| **TRL 3** | Analytical and experimental proof-of-concept | **Achieved** — companion paper `dsfb_robotics.tex` §6 Formal Results: Law 1 Structural Detectability, Theorem 1 Finite-Time Envelope Exit, Theorem 9 Deterministic Interpretability. |
| **TRL 4** | Component validation in laboratory environment | **Achieved (public-data)** — ten real-world-dataset evaluation via `paper-lock`, ≥ 95% dsfb-gray audit target, Miri ×3 clean, Kani-verified safety properties. |
| **TRL 5** | Validation in relevant environment | **Pending** — requires on-platform integration with a real robot controller in a non-safety-rated companion-processor configuration. |
| **TRL 6** | Prototype in relevant environment | **Pending Phase II** — site-data validation against a deployed PHM system. |
| **TRL 7** | Prototype in operational environment | **Out of scope** for this crate. |

## SBIR Phase I deliverables

This crate satisfies the Phase I deliverables typically required for
robot-PHM SBIR opportunities:

- **Read-only non-intrusive integration** — documented in
  [`non_intrusion_contract.md`](non_intrusion_contract.md), enforced
  by type signature and `#![forbid(unsafe_code)]`.
- **Public-data validation** — ten real-world datasets exercised via
  `paper-lock`, aggregated via `PaperLockReport`, and documented in
  per-dataset oracle-protocol files.
- **A / B comparison capability** — paper-lock emits per-dataset
  aggregate statistics (review-surface compression ratio, episode
  precision, peak residual) that a Phase I report can compare
  against an incumbent threshold-alarm chain.
- **Structural episode outputs** — canonical `Episode` records with
  traceable provenance (index, residual norm, drift, grammar,
  decision) suitable for operator review-surface integration.
- **Reproducibility package** — pinned toolchain, bit-exact
  paper-lock tolerance gate, Miri + Kani audit reports, Colab
  notebook, figure regeneration script.
- **Audit trail** — every decision in the episode stream is
  reproducible from the input residuals via the pure-function
  pipeline; `paper-lock --emit-episodes` exposes the per-sample
  trace for forensic review.

## SBIR Phase II path

Phase II typically advances from public-data validation to **on-robot,
non-intrusive companion-processor deployment**. The crate is engineered
for this path:

- `no_std` + `no_alloc` + zero-unsafe core compiles for Cortex-M4F
  and RISC-V 32-bit targets via the pinned `rust-toolchain.toml`.
- The observer's per-sample API (`observe_one`) is bounded-latency:
  a single call executes in O(W + K) constant-stack time without
  heap access.
- The non-intrusion contract guarantees deployment on a separate
  processor via a read-only interface (UART / SPI mirror / shared-
  memory read-only view) does not require any change to the
  safety-rated controller.

Phase II candidate datasets and platforms (real-world, on-robot):

- Production KUKA LWR-IV deployment in a research lab or
  manufacturing cell.
- Franka Panda in a teleoperation / force-control research setting.
- UR10 in a real manufacturing or warehouse environment.
- ANYmal, Spot, or Cassie for legged-platform balance monitoring
  (per-vendor data-use terms).

## Licensing posture for SBIR deliverables

Per [`NOTICE`](../NOTICE):

- The **theoretical framework and supervisory methods** are
  proprietary Background IP of Invariant Forge LLC (Delaware LLC
  No. 10529072). Commercial deployment requires a separate written
  licence.
- The **reference implementation** is released under Apache-2.0.
- Dataset attributions per each `docs/<slug>_oracle_protocol.md`.

Licensing enquiries: `licensing@invariantforge.net`.
