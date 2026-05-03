# Ember MMIO Model

This document defines how memory-mapped device access is represented at the `Ember` boundary in the Phosphoric prototype.

The purpose is to keep raw MMIO inside the trusted machine layer while still allowing higher layers to interact with devices through narrow, typed interfaces.

## Core Rule

Raw MMIO is not a general programming primitive.

Rules:

- higher layers do not read or write arbitrary physical addresses
- raw register addresses stay inside `Ember`
- device interaction crosses the boundary through typed register blocks or typed device functions
- MMIO access does not imply general pointer arithmetic authority

## MMIO Boundary Shape

The boundary is split into two layers:

- raw MMIO layer inside `Ember`
- typed device-facing layer exposed upward

The raw layer owns:

- physical base addresses
- volatile or architecture-specific access semantics
- width-correct load and store primitives
- memory-ordering or fence requirements tied to the device model

The typed layer owns:

- named register blocks
- register field meanings
- allowed operations over those registers
- narrow helper functions that express device intent instead of arbitrary address arithmetic

## Register Block Representation

Each device-visible MMIO region should be represented as a fixed register block specification.

Required properties:

- device name
- base address source
- total span or block size
- register offsets
- register widths
- access mode per register

Access modes in v0:

- read-only
- write-only
- read-write
- write-one-to-clear

If a register has side effects on read or write, the typed specification must say so.

## Register Access Rules

Each typed register access must define:

- which register is being accessed
- the required access width
- whether the access is read, write, or read-modify-write
- what preconditions hold before the access
- what postconditions or state changes are expected afterward

Forbidden:

- generic “read at offset” or “write at offset” interfaces exposed to higher layers
- byte-slice reinterpretation of device regions in ordinary code
- untyped arithmetic over MMIO base addresses outside the raw layer

## Typed Device Handles

Higher layers should interact with devices through typed handles such as:

- `Ps2Controller`
- `FramebufferRegs`
- `TimerRegs`

Rules:

- a typed handle represents access to one device model or one register family
- the handle does not expose the raw base address as ordinary data
- constructing the handle is a trusted action inside `Ember`
- possession of the handle does not bypass higher-level capability policy

## Read-Modify-Write Discipline

Read-modify-write operations are especially risky for device state.

Rules:

- use explicit typed helper operations where possible
- preserve reserved bits unless the register specification says otherwise
- do not expose generic bit-twiddling helpers upward when the device semantics are register-specific
- if a register read has side effects, avoid read-modify-write patterns entirely unless the device contract requires them

## Width And Alignment Rules

Rules:

- register width must match the documented device access width
- misaligned access must be treated as invalid unless the architecture and device contract explicitly allow it
- mixed-width aliasing of the same register is forbidden unless documented for that device

Why this matters:

- the wrong width can trigger undefined or device-specific destructive behavior
- alignment mistakes can fault or produce hardware-dependent corruption

## Ordering And Visibility

MMIO access may require ordering guarantees that normal memory does not.

Rules:

- ordering-sensitive accesses stay encapsulated inside the raw MMIO layer or the typed device helper that owns that protocol
- fence or barrier requirements must be local to the boundary code that knows the device contract
- higher layers should not be responsible for remembering hidden ordering requirements

## Handoff To Higher Layers

Higher layers may receive:

- typed device handles
- typed register-block wrappers
- narrow functions such as `ack_interrupt`, `read_scancode`, `set_timer_deadline`

Higher layers may not receive:

- raw physical addresses
- unrestricted register-index arithmetic
- opaque “unsafe device pointer” style escape hatches

## Error Handling

The MMIO model must integrate with the explicit error model.

Rules:

- unsupported or unavailable device states should surface as explicit errors where the interface can fail meaningfully
- misconfigured hardware metadata discovered at boot should be rejected at the trusted boundary
- MMIO access does not justify panic-driven control flow in ordinary runtime code

## Current Guarantees

This MMIO model currently guarantees:

- raw device access remains inside `Ember`
- higher layers consume typed device interfaces rather than raw addresses
- register width and access mode are part of the reviewed device contract
- MMIO is treated as an authority-bearing operation, not a convenience primitive

## Forbidden Drift

The following changes are forbidden unless this document and the safety-boundary document are updated together:

- exposing raw address arithmetic for device regions above `Ember`
- exporting generic untyped offset read or write APIs
- treating MMIO handles as ordinary integers
- moving ordering-sensitive device protocol code into unrelated higher layers

## Future Work That Is Not Yet Promised

The following are deferred:

- code-generated register definitions
- formal register-spec verification
- DMA-capable device modeling
- IOMMU-aware MMIO policy
- a generalized cross-device driver framework
