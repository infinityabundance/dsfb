# Non-intrusion contract

DSFB is a **read-only advisory observer**. It runs alongside an
incumbent robotics control / prognostics stack and reads the
residuals that stack already computes. Removing DSFB from the system
changes nothing about the robot's control, actuation, or safety
behaviour. This document enumerates the contract and the mechanisms
that enforce it.

## The five clauses

### 1. No mutation of upstream data

The public API of this crate accepts residual streams as `&[f64]`
— an immutable shared reference. The Rust borrow checker enforces
at compile time that the observer cannot mutate the incumbent's
residual buffer. Any attempt to do so is a compile error.

Enforcement: type signature + `#![forbid(unsafe_code)]`.

### 2. No write path into upstream state

DSFB does not link against, depend on, or import any crate belonging
to an incumbent robot controller (ROS 2 nodes, OPC UA Robotics
clients, manufacturer SDKs). Its output is advisory `Episode` records
emitted into a caller-owned `&mut [Episode]` buffer. The caller
decides what (if anything) to do with them.

Enforcement: dependency graph. The crate's direct dependencies are
`serde`, `serde_json`, and `csv` — all optional and feature-gated.
No controller or actuation dependency.

### 3. Deterministic under identical inputs

Identical ordered residual streams produce identical ordered episode
sequences. This is the cornerstone of paper-lock's bit-exact
reproducibility gate: three consecutive runs of
`paper-lock <dataset> --fixture` produce byte-identical JSON output.

Enforcement:

- Pure-function composition: no global mutable state, no `RefCell`,
  no atomics, no interior mutability.
- `tests/proptest_invariants.rs::observe_is_deterministic` verifies
  with 256 randomised inputs per invocation.
- `tests/paper_lock_binary.rs::fixture_output_is_bit_exact_across_repeat_invocations`
  verifies at the binary boundary for all ten datasets.

### 4. Bounded output

The observer writes at most `out.len()` episodes into the caller's
output buffer. It never allocates, never grows the buffer, never
overflows it.

Enforcement:

- Type signature: `observe(residuals: &[f64], out: &mut [Episode]) -> usize`.
  The returned count is the number of valid episodes; the caller
  knows the capacity.
- Kani proof `proof_engine_observe_bounded` verifies the property
  symbolically with 585 checks passing.
- `tests/proptest_invariants.rs::observe_never_writes_past_output`
  verifies stochastically.

### 5. Observer-only, removable

DSFB outputs are **advisory only**. Downstream automation built on
top of DSFB episodes is the integrator's responsibility and must be
independently safety-assessed. A correct deployment:

- Routes DSFB outputs to an operator review surface (dashboard,
  audit log, alert channel).
- **Does not** route DSFB outputs back into the safety-rated
  controller's command path.
- Deploys the observer on a separate, non-safety-rated companion
  processor if the upstream controller is safety-rated (see
  [`SAFETY.md`](../SAFETY.md) §"Deployment matrix").

## Adversarial posture

DSFB does not defend against malicious inputs — it is not an
intrusion-detection system. If an attacker controls the residual
stream fed into `observe()`, they control DSFB's output in the
obvious way (residuals → residual norms → grammar transitions).
DSFB's read-only posture means the attacker **cannot** escalate from
controlling DSFB's output to modifying the upstream robot state
through DSFB — that escalation path does not exist by construction.

## Compile-time summary of enforcement

| Clause | Enforcement surface |
|---|---|
| 1. No mutation of upstream data | `&[f64]` type signature + `#![forbid(unsafe_code)]` |
| 2. No write path into upstream state | Dependency graph (no controller deps) |
| 3. Determinism | Pure-function core + 256 proptest inputs per property + paper-lock tolerance gate |
| 4. Bounded output | Kani proof + proptest invariant + type signature |
| 5. Removable | Cargo feature gating; default build has zero runtime deps |

Reviewers who need to see the contract spelled out at the source
level: the observer-contract guarantees are restated at
[`src/lib.rs::observe`](../src/lib.rs) crate-level docs and
[`ARCHITECTURE.md`](../ARCHITECTURE.md) §"Canonical API".
