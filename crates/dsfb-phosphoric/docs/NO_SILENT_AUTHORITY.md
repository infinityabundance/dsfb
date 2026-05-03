# No Silent Authority

## The invariant

> **No authority transition may occur without either a declared
> manifest edge or a typed residual.**

This is the load-bearing sentence that distinguishes Phosphoric from a
tracing or observability system. A logger passively records what
*might* be interesting; a court closes the universe — every authority
crossing is either pre-authorized (manifest edge) or is itself
evidence (typed residual). There is no third option, no
"informational" event, no "warn-level" trace, no "for debugging only"
escape hatch.

## What counts as an authority transition

Any operation that crosses a forensic boundary declared in
`docs/FORENSIC_PRIMACY.md` §1:

| Boundary                  | Manifest field           | Residual on violation |
|---------------------------|--------------------------|-----------------------|
| CAP ISSUE / CAP REVOKE    | manifest.capabilities    | R1 cap_graph_delta    |
| IPC SEND / IPC RECV       | manifest.ipc.routes      | R2 ipc_route_delta    |
| BUDGET USE / LOOP EXIT    | manifest.budgets         | R3 budget_pressure    |
| EFFECT ENTRY              | manifest.effects         | R4 effect_trace       |
| MMIO TOUCH                | manifest.mmio.ranges     | R5 mmio_touch         |
| TASK TRANSITION           | manifest.task_state      | R6 task_transition    |
| BOOT / ATTEST             | manifest.boot.attest     | R7 boot_check         |

For each boundary, the producer either:
1. Verifies the operation is within a manifest declaration at compile
   time (M-001..M-012 named diagnostic if not), OR
2. Emits the corresponding `R<N>` residual record at runtime
   (kernel/residual.phos `record` fn).

There is no third path. A raw authority primitive used outside an
authority wrapper file is a doctrine violation.

## Why this matters

Without this invariant, Phosphoric becomes one of these failure modes:

- **A logger.** Authority operations leak into logs that may or may
  not be checked, with formatting that drifts, severities that need
  human interpretation, and ambiguous "events" that never reach the
  classifier.
- **A best-effort tracer.** Instrumentation is "should be there" but
  not enforced, so determinism breaks silently when a hot path skips
  emission for performance.
- **A guess engine.** Without typed residuals at every boundary, the
  classifier cannot return one named verdict per incident — it must
  guess from incomplete evidence, and guess-engines have probabilistic
  scores instead of named verdicts.

All three failure modes dissolve the residual-court framing. The
no-silent-authority invariant is the line that prevents drift toward
them.

## Enforcement (current scope)

`tools/verify/verify_no_silent_authority.sh` (Make target
`verify-no-silent-authority`, wired into `verify-legendary`) enforces
the invariant at the surfaces that exist today:

1. **Doctrine sentence presence.** This file must contain the apex
   invariant verbatim. Drift in wording is rejected.
2. **`kernel/residual.phos` `record` fn intact.** The canonical
   residual emission entry point with its 14-line byte composition
   logic (per `kernel/residual.phos` `record` fn) must remain in the
   kernel — it is the load-bearing path every R<N> emission flows
   through.
3. **Closed taxonomy preserved.** `kernel/residual.phos` declares R1
   through R7 plus tail_marker; no eighth kind without a doctrine
   edit.
4. **Boundary table coherence.** This file's boundary table (above)
   must list each of R1..R7 exactly once.

## Future tightening (post-runtime-emit)

When runtime residual emission lands (Stream A reaches caller-side
hidden-pointer ABI + multi-arg calls + struct field stores composed
inside producer-emit paths), this gate tightens to:

5. **Wrapper exclusivity.** Raw authority primitives (`__sys1` for
   open/read/write, `__mmio_*`, capability allocation/revocation
   intrinsics) are banned outside designated wrapper files
   (`compiler/`, `kernel/`, host code). A `grep` for these primitives
   in any non-wrapper `.phos` file is a doctrine violation.
6. **Wrapper coverage.** Each wrapper must call `residual_emit_*`
   with the kind matching its boundary. A wrapper that exits without
   emitting is a doctrine violation.
7. **Manifest edge OR residual emit, never both omitted.** Static
   analysis on producer-emit paths confirms each authority operation
   reaches one of the two paths.

The current gate is the foundation; tightening lands when the
runtime exists.

## Status as of 2026-04-30

This document and the corresponding gate landed in **Session 17,
Stream C Milestone E**. The invariant is now doctrine-locked at the
sentence level and gate-enforced at the surfaces that exist. Producer-
side emission infrastructure does not yet exist; once it does, the
tightening rules above become enforceable and will be wired in.

This file is the contract that future producer / runtime / tooling
work must satisfy. Any change that allows a third path (silent
authority transition) violates the framing regardless of local
benefit.
