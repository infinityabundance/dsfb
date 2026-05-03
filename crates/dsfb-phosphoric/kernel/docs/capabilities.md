# Capability Model

This document defines the v0 capability model for PhosphorOS kernel services.

Capabilities are the system's explicit authority mechanism. They do not create strong hostile-task isolation by themselves in the single address-space prototype, but they do prevent the design from depending on ambient global authority.

## Core Rules

The v0 capability model is built on these rules:

- every authority-bearing kernel object is referenced through a typed capability
- capabilities are affine by default
- authority does not arise from task identity alone
- capability possession is separate from raw machine privilege
- capability use must still respect the fixed-capacity runtime model

## Scope

The first capability model covers handles for:

- windows
- framebuffers
- channels
- timer access
- device access

It does not yet attempt:

- formal capability revocation proofs
- hardware-enforced isolation between hostile tasks
- arbitrary application-defined capability classes

## Capability Classes

The initial kernel capability families are:

### Window

Represents authority to:

- create or own a window through approved kernel interfaces
- request redraw or state changes for that window
- receive input or lifecycle events for that window when granted

### Framebuffer

Represents authority to:

- draw to an approved framebuffer or compositor target
- access rendering operations through bounded kernel APIs

It does not imply unrestricted raw framebuffer memory access.

### Channel Endpoint

Represents authority to:

- send on a channel
- receive from a channel
- or do both, depending on endpoint rights

### Timer (v0.2-deferred)

**Status: not in v0.1.** [docs/language/runtime_profile.toml](../../docs/language/runtime_profile.toml) does not declare a `Timer` handle kind. Time access in v0.1 is via the `time` effect (a read-only monotonic timer query); it is not authority-bearing. Timer-as-capability re-enters scope in v0.2 if and when bounded timer waits become a real workload need. The original v0 design intent — represent authority to read time and request bounded timer waits or deadlines — is preserved here as a future contract.

### Device (v0.2-deferred)

**Status: not in v0.1.** Devices in v0.1 (keyboard, mouse) are accessed exclusively by Ember (trusted profile) and routed to the kernel through Ember's typed input-handoff entrypoint. The kernel does not mint or hold Device capabilities. Device-as-capability re-enters scope in v0.2 if and when a real driver story lands. The original v0 design intent — authority to use a specific typed device interface approved by the kernel, without raw MMIO — is preserved here as a future contract.

The Timer and Device deferrals are pinned in [docs/PHOSPHOROS_DESIGN.md](../../docs/PHOSPHOROS_DESIGN.md) under "Contradictions resolved".

## Handle Shape

Capabilities should appear in the language and kernel interfaces as opaque typed handles.

Required properties:

- handle type identifies the capability family
- handle identity is not forgeable from ordinary integers or byte buffers
- internal kernel object lookup stays inside fixed-capacity tables
- handle use must be validated against kernel-owned state before acting on the authority

Forbidden:

- raw pointer-as-capability designs
- ambient global singleton authority
- integer-only untyped handle APIs in normal code

## Affine Ownership

Capabilities are affine by default.

Rules:

- a capability handle may be moved
- a capability handle may not be duplicated implicitly
- duplication is forbidden unless a later documented capability class is explicitly shareable
- borrowing a capability for inspection or call use does not create a second owned capability

Why this matters:

- it keeps authority transfer explicit
- it prevents silent privilege cloning in normal code

## Capability Tables

Tasks reference capabilities through fixed-capacity kernel-owned tables.

Rules:

- capability table storage is bounded
- entries are created through explicit kernel operations
- a task cannot hold an unbounded number of capabilities
- capability-table exhaustion is an explicit error

The exact per-task table capacity may be fixed by later kernel layout planning, but it must remain bounded and reviewable.

## Creation And Transfer

Capability creation and transfer are trusted kernel actions.

Rules:

- tasks do not mint arbitrary new capabilities for kernel objects
- transfer between tasks must be explicit and validated
- transfer through IPC must not bypass endpoint-right checks or object-type validation
- creation of a capability must be tied to creation or discovery of a real kernel-managed object

## Validation Rules

Every capability use must validate:

- that the handle family matches the expected operation
- that the referenced kernel object still exists
- that the rights requested by the operation are actually present
- that the task is not attempting to use a revoked or invalid handle

Forbidden:

- trusting handle type names without runtime validation of the kernel object state
- assuming possession of one capability implies possession of unrelated capabilities

## Revocation And Invalidation

The v0 model must support explicit invalidation semantics even if the policy remains simple.

Rules:

- when a kernel object is destroyed, related capabilities become invalid
- using an invalid capability returns an explicit error
- stale handles must not resurrect kernel objects

The revocation policy may be conservative in v0, but invalid-handle behavior must be explicit.

## Rights Discipline

Some capability families require rights refinement.

Examples:

- a channel endpoint may be send-only, receive-only, or bidirectional
- a window handle may distinguish ownership from limited event subscription in later phases
- a timer handle may distinguish read-only time queries from deadline-setting authority in later phases

The v0 rule is:

- if rights distinctions matter for correctness or authority control, they must be modeled explicitly instead of being inferred from call site conventions

## Current Guarantees

This capability model currently guarantees:

- windows, framebuffers, channels, timers, and devices are all authority-bearing kernel objects
- capability possession is explicit rather than ambient
- capabilities are affine by default
- invalid or stale handle use is expected to fail explicitly

These guarantees define the required authority model, not proof of formal non-interference.

## Forbidden Drift

The following changes are forbidden unless this document and the threat model are updated together:

- ambient access to kernel services that should require capabilities
- untyped integer handle APIs replacing typed capability interfaces
- implicit capability duplication
- granting raw device or framebuffer authority where a narrower typed interface is possible

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- formal revocation guarantees
- delegation policies richer than the initial kernel model requires
- shareable capability families with audited duplication rules
- stronger process-isolated capability enforcement
