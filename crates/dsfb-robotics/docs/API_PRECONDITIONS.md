# API preconditions

This document enumerates the **preconditions** a caller must satisfy
when using the `dsfb-robotics` public API. DSFB's observer contract
guarantees that violating a precondition will not cause undefined
behaviour (the crate is `#![forbid(unsafe_code)]`) but **may** cause
the observer to produce less useful output — zero episodes, all-
Admissible episodes, or silently-ignored samples — so callers need
to know when these degraded modes apply.

## Core API

### `fn observe(residuals: &[f64], out: &mut [Episode]) -> usize`

Top-level convenience wrapper. Stage III calibrates from the first
20 % of the input.

| Precondition | What happens if violated |
|---|---|
| `residuals.len() <= usize::MAX / 2` | Debug-assert fails; release truncates at `isize::MAX / 2`. |
| `out.len() <= usize::MAX / 2` | Debug-assert fails; release truncates. |
| Calibration window contains ≥ 1 finite sample | Falls back to `AdmissibilityEnvelope::new(f64::INFINITY)` — all episodes Admissible/Silent. |
| `residuals[k].is_finite()` for structural-claim samples | Non-finite samples are below-floor: drift/slew forced to 0, grammar forced to Admissible. |

### `DsfbRoboticsEngine::<W, K>::new(rho: f64)`

| Precondition | What happens if violated |
|---|---|
| `rho >= 0.0` | `is_violation(norm, _)` effectively always true for `norm > 0`. |
| `rho.is_finite()` (recommended) | `rho = ±∞` disables violations for the positive-infinity case; NaN produces NaN-tainted envelopes. |
| `W >= 2` (for meaningful drift) | `W = 1` or `0` produces zero drift/slew. |
| `K >= 1` | `K = 0` disables recurrent-grazing detection. |

### `DsfbRoboticsEngine::observe(residuals, out, context)`

| Precondition | What happens if violated |
|---|---|
| `out.len() <= residuals.len()` is not required | Extra output capacity is left as `Episode::empty()`. |
| `out.len() >= residuals.len()` is not required | Samples past `out.len()` are dropped; the method returns `out.len()`. |
| Single `RobotContext` for the whole call | Callers who need per-sample context switching should use `observe_one` in a loop. |

### `AdmissibilityEnvelope::calibrate_from_window(norms)`

| Precondition | What happens if violated |
|---|---|
| `norms` non-empty | Returns `None`. |
| `norms` contains ≥ 1 finite sample | Returns `None`. |
| `std(norms) < ∞` | Returns `None` via `sqrt_f64(variance)`'s finite guard. |

## Policy-layer invariants (guaranteed by construction)

- Every `GrammarState` maps to a valid `PolicyDecision` via
  `PolicyDecision::from_grammar`. Kani-verified:
  `proof_policy_from_grammar_is_total`.
- `Admissible` → `Silent`; every `Boundary[_]` → `Review`;
  `Violation` → `Escalate`. Pure function; no side effects.

## Determinism guarantee

- `observe(residuals, out)` is a pure function: identical ordered
  inputs produce identical ordered outputs. Verified by 12 proptest
  invariants and the paper-lock bit-exact tolerance gate. See
  [`audit/kani/KANI_AUDIT.md`](../audit/kani/KANI_AUDIT.md)
  §"Observer-purity coverage path" for the full coverage trace.

## What DSFB never does

- No interior mutability; no `RefCell`, `Cell`, `UnsafeCell`.
- No atomics; no `Arc`, `Mutex`, `RwLock`.
- No I/O in the core; no filesystem reads, no logging, no syscalls.
- No panics in production code (`.unwrap` / `.expect` / `panic!` /
  `todo!` / `unimplemented!` forbidden in `src/` outside test modules).

See also [`SAFETY.md`](../SAFETY.md) for the full safety posture and
[`non_intrusion_contract.md`](non_intrusion_contract.md) for the
read-only observer contract.
