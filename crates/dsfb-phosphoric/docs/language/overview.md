# Phosphoric Language Overview

Phosphoric is a deliberately tiny systems language for a narrow domain:

- kernel components above the `Ember` machine layer
- fixed-capacity drivers built on typed MMIO or port boundaries
- GUI compositor logic
- capability-scoped services
- message-passing applications

It is not a general-purpose language. Its design goal is to make authority, memory growth, and failure behavior explicit enough that the resulting runtime is easier to audit than a broader feature-rich stack.

## Design Goals

Phosphoric v0.1 is designed around these goals:

- tiny core syntax and semantics
- fixed capacities instead of unbounded runtime growth
- capability-oriented authority
- explicit effects
- deterministic failure behavior
- explicit ownership
- no heap-backed runtime path

## Core Principles

### 1. Static Memory First

- runtime objects have compile-time known layout
- arrays and slices are bounded
- runtime capacities are fixed by configuration, not grown on demand
- exhaustion is handled as an explicit error

### 2. Capability-Oriented Authority

- access to windows, framebuffers, channels, timers, and devices must flow through typed capabilities
- capabilities are affine by default
- a function must receive the authority it uses
- ambient global access is rejected

### 3. Explicit Effects

- functions declare the side-effect classes they require
- effect sets are part of interface review
- calling a function with undeclared effects is a compile-time error

### 4. Explicit Failure

- fallible operations return explicit result values
- failure is propagated by language constructs, not hidden runtime traps
- panic-driven control flow is outside the intended profile

### 5. Narrow Control Surface

- the language should be small enough to read and implement directly
- features that mainly add abstraction power without strengthening safety are excluded from v0

## Current v0.1 Feature Profile

Phosphoric v0.1 supports:

- modules
- `struct`
- `enum`
- function declarations
- `match`
- fixed arrays
- bounded slices
- `Option`
- `Result`
- bounded `for` loops
- explicit move semantics
- explicit `effects(...)` annotations

The language is expression-oriented where practical, but the v0.1 subset prioritizes clarity over cleverness.

## Current Guarantees

Phosphoric v0.1 currently guarantees this design intent:

- source programs are intended to be `no_std`, `no_alloc`, and `no_unsafe`
- ordinary language code has no raw pointer arithmetic surface
- effectful operations must be declared
- runtime data structures are fixed-capacity by design
- capability-carrying code is reviewed as authority-carrying code

These are language-profile guarantees, not proof that every implementation bug is eliminated.

## Forbidden In v0.1

The following constructs are forbidden in the initial language profile:

- classes and inheritance
- broad trait systems
- `async` / `await`
- arbitrary closure capture
- heap-backed strings and collections
- unrestricted recursion
- hidden dynamic dispatch
- exceptions
- reflection
- JIT execution
- macro systems that behave like a second language
- arbitrary FFI in normal code

## Relationship To Ember

Phosphoric does not replace the machine-dangerous boundary.

- `Ember` owns privileged and architecture-specific operations
- Phosphoric code uses typed interfaces exported by `Ember`
- if a low-level operation cannot honestly fit inside the Phosphoric safety profile, it belongs below the language boundary

## Relationship To PhosphorOS

Phosphoric is the language used to implement most kernel services, compositor logic, IPC-facing services, and applications above `Ember`.

It should make these patterns natural:

- explicit message passing
- explicit authority transfer
- bounded data representation
- small, reviewable service interfaces

## Future Work That Is Not Yet Promised

The following ideas are not part of the v0.1 contract:

- source-level borrow syntax
- richer generic systems beyond what the fixed-capacity profile requires
- stronger compile-time state-machine proofs
- verified lowering or verified backend behavior
- additional targets or runtime profiles
- broader library ecosystems

Any future expansion must prove that it does not invalidate the small-surface design goal.
