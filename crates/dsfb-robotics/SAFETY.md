# Safety posture

## Summary

`dsfb-robotics` is a **read-only advisory observer**. It does not
participate in any robot control loop, safety function, or actuation
decision. Removing it from the system changes nothing about the
robot's control or safety behaviour.

## Memory safety

- `#![forbid(unsafe_code)]` at the crate root. No `unsafe` blocks,
  no `UnsafeCell`, no `RefCell`, no FFI.
- No `build.rs`, no proc-macros, no raw pointers.
- No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or
  `unimplemented!()` in `src/` — the core returns structured results
  or saturates, it does not panic.
- Default build is `no_std` + `no_alloc`: no dynamic allocation,
  bounded stack usage, fixed-capacity internal structures.

## Functional safety

DSFB is explicitly **not** a safety-rated component. The crate is
out of scope for:

- ISO 10218-1:2025 / ISO 10218-2:2025 (industrial robot safety)
- ISO 13849 (safety of machinery — safety-related parts of control systems)
- IEC 61508 (functional safety of E/E/PE safety-related systems)
- ISO 13482 (personal-care robot safety)
- EN 61131-6 (programmable controllers — functional safety)

If the upstream robot controller is a safety-rated component, DSFB
must be deployed on a **separate, non-safety-rated companion
processor** (e.g. a second MCU in the same enclosure, or a non-safety
partition of a mixed-criticality SoC) receiving residuals through a
read-only unidirectional interface (UART, SPI mirror, or shared-memory
read-only view). DSFB outputs **must not** be connected back to the
safety-rated controller's command path.

## Non-intrusion contract

| Guarantee | Enforcement |
|---|---|
| No mutation of upstream residual data | Public API takes `&[f64]`, not `&mut [f64]` |
| No write path into upstream controller state | DSFB does not link against or depend on any controller API |
| Deterministic under identical inputs | Pure-function core; no global mutable state |
| Bounded output | `observe` writes at most `out.len()` episodes |
| Bounded stack usage | Fixed-capacity internal structures; no recursion |
| Observer-only, removable | Advisory outputs; zero coupling to control path |

## Review posture

Every DSFB output is **advisory only**. Operator workflows must treat
DSFB episodes as:

- A review-surface compression tool, not an automation signal.
- A structural explanation aid, not a fault classifier.
- A human-in-the-loop input, not a controller setpoint.

Automation built on top of DSFB outputs is the responsibility of the
integrator and must be independently safety-assessed against the
standards applicable to the integrator's deployment.

## Deployment matrix

| Deployment | Supported? | Notes |
|---|---|---|
| Host-side analytics dashboard reading controller logs | Yes | Recommended first deployment |
| Companion MCU reading residuals over read-only interface | Yes | `no_std` + `no_alloc` core designed for this |
| Inside a safety-rated controller | **No** | DSFB is not safety-rated |
| Driving actuation decisions | **No** | Violates the observer contract |
| Replacing an incumbent observer or estimator | **No** | DSFB augments, it does not replace |

## Reporting concerns

Safety concerns that do not rise to the level of a security
vulnerability (see `SECURITY.md`) can be opened as regular GitHub
issues with the `safety` label. For deployment-specific safety
questions, contact `safety@invariantforge.net`.
