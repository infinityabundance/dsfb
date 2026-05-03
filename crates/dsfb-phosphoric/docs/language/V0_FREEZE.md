# Phosphoric v0 Freeze

This document defines the legal v0 language boundary.

Anything not listed as allowed here is out of profile for v0 and must be rejected, deferred, or documented as future work. This file is intended to stop “small” additions from turning into silent language expansion.

## Allowed In v0

Top-level forms:

- modules
- capabilities
- structs
- enums
- functions

Types:

- explicit integer types
- `bool`
- fixed arrays
- bounded slices
- `Option[T]`
- `Result[T, E]`
- same-module nominal type paths already documented in the grammar

Control flow:

- `if`
- `match`
- bounded `for` ranges
- explicit `return`
- block expressions already present in the grammar

Authority and effects:

- explicit capability parameters and returns
- explicit `effects(...)` annotations
- the current fixed effect set only: `draw`, `ipc`, `mmio`, `sched`, `time`

Semantics already in scope:

- move-oriented ownership
- simple affine capability handling
- explicit result-style failure modeling

## Forbidden In v0

- traits
- impl blocks
- macros
- `async`
- recursion beyond the currently rejected kernel-profile subset
- heap objects
- heap-backed strings
- classes
- inheritance
- closures
- generic expansion beyond the current rejected generic-function syntax
- unrestricted dynamic dispatch
- exceptions
- reflection
- plugin or JIT surfaces
- floating-point dependence in kernel-core logic

## Explicitly Not In The Frozen Surface

These are not part of v0, even if they are mentioned elsewhere as design intent:

- borrow syntax
- mutable-borrow syntax
- FFI syntax
- import syntax
- external-call syntax
- method receiver syntax
- pattern guards
- lifetime syntax
- package or module dependency syntax beyond the current single-module source shape

## Enforcement Rule

- parser acceptance is not enough to claim a feature exists
- if a construct is not listed in `Allowed In v0`, it is out
- if a construct is documented as design intent but has no parser/checker support, it remains out
- future additions must update this file before they are treated as part of the language

## Review Rule

Reject any change that:

- widens the grammar without updating this file
- claims a v0 feature that only exists in prose
- introduces a new abstraction family without a corresponding enforcement story
- treats “nice to have” syntax as harmless creep
