# Phosphoric v0.1 Error Model

This document defines the initial failure semantics for Phosphoric v0.1.

The error model is built around explicit, typed failure. Operations that can fail must surface that fact in their interface rather than relying on exceptions, ambient traps, or panic-driven control flow.

## Core Rules

The v0.1 error model follows these rules:

- fallibility is explicit in function signatures
- recoverable failure is represented with `Result[T, E]`
- optional absence is represented with `Option[T]`
- failure propagation is explicit in the language and compiler IR
- exhaustion, validation failure, and capability denial are local error cases unless explicitly escalated by trusted policy

## Result As The Default Fallible Interface

`Result[T, E]` is the standard representation for recoverable failure.

Rules:

- if an operation can fail in ordinary operation, its interface should return `Result`
- the error type `E` must be explicit
- callers must either handle or explicitly propagate the error
- APIs should prefer small, named error enums instead of untyped numeric codes where practical

Examples of `Result`-style failures:

- queue full
- capability missing
- invalid state transition
- malformed input
- unsupported operation
- draw target unavailable

## Option As Absence, Not Failure Hiding

`Option[T]` is used only when absence is expected and not itself diagnostic.

Allowed uses:

- lookup that may or may not find an item
- optional cached state
- optional event payload field

Forbidden use:

- replacing a meaningful error with `None` when the caller needs to know why the operation failed

## Propagation

The language may support concise propagation syntax in source or IR, but propagation must remain explicit in meaning.

Rules:

- propagation may only forward typed failures already present in the function signature
- propagation cannot erase or invent effect requirements
- propagation cannot turn recoverable failure into process abort implicitly

## Failure Categories

The v0 profile expects these broad failure categories:

### Input Validation Failure

Examples:

- malformed messages
- out-of-range coordinates
- invalid enum tag in decoded external data

Required handling:

- reject locally with an explicit error
- do not trust malformed input after detection

### Capacity Exhaustion

Examples:

- task table full
- channel full
- message ring full
- payload too large for `IPC_PAYLOAD_MAX`

Required handling:

- return an explicit error or apply a documented bounded policy
- never allocate around the failure

### Capability Denial

Examples:

- missing window authority
- missing framebuffer authority
- attempt to send on an endpoint without send rights

Required handling:

- reject explicitly
- do not widen authority as a fallback

### State Violation

Examples:

- using a handle after revocation
- invalid scheduler transition
- GUI operation against a destroyed window

Required handling:

- reject explicitly
- preserve global invariants even if the local operation fails

## Panic And Trap Policy

Phosphoric v0.1 does not treat panic as a normal error-handling tool.

Rules:

- panic-driven control flow is outside the intended language profile
- recoverable failures must not be modeled as panic paths
- trusted low-level code may still need irrecoverable halt or fault handling below the language boundary, but that belongs to `Ember` or explicitly trusted kernel policy
- application-facing or service-facing code should return explicit failures instead of crashing upward

## Compiler Obligations

The compiler direction for v0.1 should enforce:

- functions returning `Result` cannot ignore mismatched return shapes
- propagation is checked against the declared result type
- unsupported implicit failure paths are rejected rather than silently lowered
- error handling remains visible in the typed interface

## Error Type Design Guidance

Error types should be:

- specific to a subsystem
- finite and enumerable
- free of heap-backed payload requirements
- representable with fixed-size data

Preferred style:

- small `enum`-based error families such as `DrawError`, `IpcError`, `SchedError`, `InputError`

Avoid:

- stringly typed errors
- dynamically allocated backtraces
- opaque catch-all status codes when a narrower enum is possible

## Current Guarantees

The v0.1 error model currently guarantees this design intent:

- ordinary failure is typed and explicit
- absence and failure are distinguished
- local exhaustion does not justify hidden allocation
- explicit failure handling is part of interface review

These guarantees define the intended profile, not proof that every later implementation obeys it automatically.

## Forbidden In v0.1

The initial error model forbids:

- exceptions
- unwinding as a normal control-flow tool
- hidden panic-based recovery
- silently ignored failure results
- translating detailed recoverable failure into generic process termination
- dynamic error objects that require heap allocation

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- richer typed diagnostics for development-only builds
- source-level sugar for propagation beyond the minimum explicit model
- machine-checked proof that all trusted interfaces preserve failure containment
- multiple error profiles for debug versus release beyond what later docs define explicitly
