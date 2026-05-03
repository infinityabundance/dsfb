# Phosphoric v0.1 Effect System

This document defines the initial effect system for Phosphoric v0.1.

The effect system exists to make side-effect authority visible in function signatures. It is not a replacement for capability checks. A function must both hold the required capability values and declare the relevant effect classes.

## Effect Set

The initial effect set is fixed to:

- `draw`
- `ipc`
- `mmio`
- `sched`
- `time`

No additional effect labels are part of v0.1.

## Effect Declaration

Functions declare effects with an `effects(...)` clause:

```text
fn redraw(win: Window, fb: Framebuffer) -> Result[Unit, DrawError]
    effects(draw)
{
    ...
}
```

Rules:

- the effect clause is optional only for effect-free functions
- an empty effect set is represented by the absence of an effect clause in v0.1
- effect labels are explicit names, not inferred ambient properties
- effect declarations are part of the function interface

## Meaning Of Each Effect

### `draw`

`draw` covers:

- issuing drawing operations to a framebuffer or compositor surface
- mutating visible UI state through drawing primitives
- invoking helper routines that perform drawing work

`draw` does not by itself grant access to every framebuffer. The code still needs the appropriate drawing capability.

### `ipc`

`ipc` covers:

- sending messages
- receiving or dequeuing messages
- manipulating channel endpoints through approved IPC interfaces

`ipc` does not by itself grant access to every channel. The code still needs the appropriate channel capability.

### `mmio`

`mmio` covers:

- interacting with typed memory-mapped device interfaces exposed above `Ember`
- calling wrappers whose purpose is direct device register access

In practice, most raw MMIO remains inside `Ember`. This effect exists because higher layers may still call typed boundary functions that represent MMIO activity.

### `sched`

`sched` covers:

- operations that alter task scheduling state
- yielding, blocking, waking, or otherwise interacting with scheduler-visible task transitions
- calling into scheduling primitives exposed by the kernel

### `time`

`time` covers:

- reading timers or clocks through approved interfaces
- registering time-based waits or deadlines through time services

## High-Level Checking Rules

The compiler should enforce these high-level rules in v0.1:

- if a function performs an operation requiring an effect, that effect must appear in the function's declared effect set
- if a function calls another legal same-module function, the caller's effect set must include every effect required by the callee
- if a function declares an effect that is not required by the same-module effect closure, that declaration is rejected where the current checker can prove it unnecessary
- effect-free functions may call only other effect-free functions
- effect declarations are checked statically and never granted implicitly by module location or naming convention
- qualified, unresolved, and FFI-like call targets are rejected before effect propagation because they are outside the frozen v0 call surface

The effect checker is intentionally conservative in v0.1. If the compiler cannot prove that an operation is effect-free, it should require the explicit effect.

## Relationship To Capabilities

Effects and capabilities solve different problems:

- effects describe what classes of side effects a function may cause
- capabilities describe what specific authorities the code holds

Both are required for sensitive actions:

- a draw routine needs `effects(draw)` and a drawing capability
- an IPC routine needs `effects(ipc)` and a channel capability
- an MMIO-facing routine needs `effects(mmio)` and a device capability or typed boundary handle

An effect declaration never replaces a capability check.

## Transitive Use

Effect requirements are transitive across legal same-module calls.

Example:

- if `fill_rect` requires `draw`
- and `paint_button` calls `fill_rect`
- then `paint_button` must also declare `draw`

This rule is what keeps effectful behavior visible in review.

Current closure boundary:

- only same-module named calls participate in v0 closure analysis
- unresolved, qualified, field-based, and FFI-like call targets are rejected before effect solving
- leaf functions currently act as local effect roots because the frozen subset has no other effectful primitive surface

## Current Guarantees

The v0.1 effect system currently guarantees this design intent:

- the effect vocabulary is small and fixed
- effectful operations are visible in signatures
- undeclared effect use is rejected for legal same-module call chains
- unnecessary declared effects are rejected where the current same-module closure can prove they are redundant
- effect-free code cannot silently call effectful helpers
- effect propagation currently applies only to legal same-module named calls

These guarantees describe the intended compiler behavior, not a formal proof of non-interference.

## Forbidden In v0.1

The initial effect system forbids:

- implicit effect inference from ambient global state
- unchecked calls from effect-free code into effectful code
- application-defined ad hoc effect taxonomies
- treating effect declarations as a substitute for capability possession
- effect-polymorphic interfaces

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- explicit external-call or FFI effect policy
- additional effects such as `fs` or `portio`
- effect subsets or effect aliases
- effect polymorphism
- proof of unnecessary-effect rejection beyond same-module closure
- richer region-sensitive or state-sensitive effect reasoning
- proof that a function is pure beyond the conservative checker
