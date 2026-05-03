# Ember Architecture

This document defines the architecture of `Ember`, the trusted machine layer beneath Phosphoric and PhosphorOS.

`Ember` exists to contain the operations that cannot honestly fit inside the Phosphoric language contract. It is intentionally small, architecture-aware, and policy-light.

## Purpose

`Ember` is responsible for:

- entering the system from the firmware or boot environment
- establishing the first trusted execution path
- handling privileged CPU transitions and trap entry/exit
- setting up static memory mappings required for the prototype
- exposing typed hardware access boundaries to higher layers
- performing low-level context-switch work required by the scheduler

`Ember` is not the operating system. It is the narrow machine substrate that lets the rest of the system avoid direct privileged hardware control.

## Design Rules

`Ember` must remain:

- small enough to review directly
- architecture-specific where necessary, but isolated behind explicit modules
- policy-light rather than feature-rich
- explicit about every hardware-dangerous operation it performs
- separate from higher-level authority, scheduling, IPC, compositor, and application policy

## Layer Boundary

The stack is split this way:

- `Ember`: machine-dangerous boundary
- `Phosphoric`: constrained language surface above the boundary
- `PhosphorOS`: kernel services, compositor, IPC, tasks, and applications above the boundary

Rules:

- higher layers may not execute privileged instructions directly
- higher layers may not manipulate page tables or trap frames directly
- higher layers may not perform raw MMIO or port I/O directly
- `Ember` may export typed interfaces, but not ambient authority

## Allowed Responsibilities

`Ember` is allowed to implement only the following responsibility classes in v0:

### Boot And Early Bring-Up

- `UEFI` entry
- early CPU mode and environment setup
- framebuffer and serial discovery handoff
- transfer from firmware context into the internal runtime boundary

### Trap And Interrupt Machinery

- IDT and trap entry/exit setup
- interrupt frame save and restore
- interrupt masking and acknowledgement hooks required by the architecture
- low-level fault dispatch into trusted higher handlers

### Memory Mapping Primitives

- static page-table setup
- activation of initial virtual-memory mappings
- typed mapping helpers needed by trusted kernel code

### Context Switch Primitives

- save and restore of machine register state
- transition between task execution contexts
- low-level entry and return glue used by the scheduler core

### Typed Hardware Boundaries

- typed MMIO register-block access shims
- typed port I/O shims where the prototype hardware requires them
- timer, framebuffer, and input-device boundary glue needed for the first target

### Fatal Fault Containment

- irrecoverable halt path
- architecture-level failure stop path when trusted invariants have already been lost

## Explicit Non-Responsibilities

`Ember` must not grow into:

- a full kernel scheduler policy layer
- an IPC implementation
- a capability-minting policy engine
- a compositor
- a window manager
- a filesystem layer
- a driver framework for broad hardware support
- an application runtime

If a feature can be expressed without direct privileged hardware manipulation, it does not belong in `Ember`.

## Export Surface

`Ember` should export only narrow, typed interfaces such as:

- boot handoff structures
- trap registration hooks
- page-table and mapping primitives
- context-switch primitives
- typed register or device access wrappers
- halt or fatal-stop interfaces

The export surface must remain smaller than the combined surface of the higher layers that consume it.

## Expected Module Layout

The v0 layout is:

- `ember/boot/`
- `ember/arch/x86_64/`
- `ember/traps/`
- `ember/mmio/`
- `ember/sched/`
- `ember/docs/`

Rules:

- architecture-specific code belongs under `ember/arch/x86_64/`
- shared trusted glue belongs only where it cannot stay above the boundary
- documentation must be updated before adding a new trusted sub-area

## Interaction With Higher Layers

Higher layers interact with `Ember` by consuming typed functions and data, not by sharing unrestricted control.

Rules:

- `Ember` does not grant authority by global ambient state
- higher layers still perform capability checks for policy decisions
- `Ember` enforces machine-level preconditions, while higher layers enforce system policy

## Current Guarantees

This architecture document currently guarantees:

- `Ember` is the only project layer allowed to own machine-dangerous operations
- the boundary between `Ember` and the rest of the stack is explicit
- `Ember` is intentionally small and policy-light
- architecture-specific code is expected to stay isolated

## Forbidden Drift

The following changes are forbidden unless the TCB and threat-model documents are updated together:

- moving GUI, IPC, or scheduler policy into `Ember`
- placing raw MMIO or port I/O into higher layers
- using `Ember` as a convenience bucket for code that is merely performance-sensitive
- widening the export surface without documenting why the boundary still remains reviewable

## Future Work That Is Not Yet Promised

The following are not part of the v0 `Ember` contract:

- multi-architecture support
- formal verification
- SMP support
- DMA isolation
- generalized driver frameworks
- dynamic loadable machine-layer extensions
