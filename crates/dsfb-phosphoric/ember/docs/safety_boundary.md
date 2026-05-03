# Ember Safety Boundary

This document lists the operations that are inherently hardware-dangerous in the Phosphoric prototype and therefore belong inside `Ember` or another explicitly trusted boundary.

The purpose is not to claim these operations become safe by naming them. The purpose is to prevent them from leaking upward into ordinary Phosphoric code.

## Boundary Rule

An operation belongs to the `Ember` safety boundary if a mistake in that operation can directly:

- corrupt privileged machine state
- redirect control flow unpredictably
- map or expose memory incorrectly
- issue invalid hardware transactions
- violate interrupt or task-switch invariants

If an operation meets that threshold, it must not be available as unrestricted surface syntax in ordinary Phosphoric programs.

## Inherently Hardware-Dangerous Operations

### Firmware Entry And Exit

Operations:

- entering from the `UEFI` boot environment
- calling firmware services during bring-up
- leaving firmware-managed execution context

Why dangerous:

- ABI mismatch, bad pointers, or invalid service use can corrupt the earliest trusted state before the kernel boundary exists

Required placement:

- `Ember` boot and architecture code only

### Privileged CPU State Changes

Operations:

- enabling or disabling interrupts
- loading control registers
- changing page-table base registers
- changing privilege-transition related machine state

Why dangerous:

- mistakes can break interrupt delivery, corrupt memory translation, or leave the CPU in an undefined control state

Required placement:

- `Ember` architecture-specific code only

### Descriptor Table And Trap Setup

Operations:

- building and loading GDT, IDT, or equivalent descriptor tables
- setting trap and interrupt entry points
- constructing trap-return state

Why dangerous:

- malformed descriptors or entry stubs can redirect control flow, corrupt stacks, or trap recursively without recovery

Required placement:

- `Ember` trap and architecture code only

### Trap Entry And Exit

Operations:

- saving machine context on entry
- restoring machine context on return
- acknowledging architecture-specific fault or interrupt state

Why dangerous:

- a single register-save or return-path mistake can destroy task state, leak privileged data, or jump to an invalid address

Required placement:

- `Ember` trap entry/exit code only

### Page-Table Manipulation

Operations:

- creating page tables
- changing mappings
- changing mapping permissions
- activating a mapping root

Why dangerous:

- incorrect mappings can expose privileged memory, create aliasing hazards, or break control flow and data integrity

Required placement:

- `Ember` memory-mapping primitives only

### Raw MMIO Access

Operations:

- reading from device register addresses
- writing to device register addresses
- relying on ordering rules for device-visible memory

Why dangerous:

- the wrong address, width, or ordering can corrupt device state, hang the machine, or violate hardware protocol

Required placement:

- raw access inside `Ember`; only typed wrappers may escape upward

### Raw Port I/O

Operations:

- issuing `in` or `out` style port operations
- writing controller commands directly to device ports

Why dangerous:

- invalid port transactions can break device state or destabilize the machine

Required placement:

- `Ember` architecture or device-boundary code only

### Context Switching

Operations:

- saving task register sets
- restoring another task register set
- changing stacks or return targets during task handoff

Why dangerous:

- mistakes can cross-contaminate task state, lose execution context, or jump to invalid control targets

Required placement:

- low-level switch primitives inside `Ember`

### Framebuffer Memory Boundary Setup

Operations:

- accepting framebuffer metadata from firmware
- translating framebuffer metadata into trusted runtime structures
- mapping or exposing the framebuffer region to higher layers

Why dangerous:

- malformed dimensions, stride, pixel format, or base address interpretation can corrupt memory outside the real framebuffer

Required placement:

- `Ember` boot and typed handoff boundary

### Timer And Interrupt Controller Boundary Setup

Operations:

- configuring timer hardware state required for scheduling
- acknowledging interrupt controller state
- installing the first timer interrupt path

Why dangerous:

- incorrect setup can stall scheduling, lose interrupts, or trigger interrupt storms

Required placement:

- `Ember` machine-boundary code only

### Fatal Halt Path

Operations:

- entering an irrecoverable halt state
- stopping execution after trusted invariants are already lost

Why dangerous:

- once this path is taken, recovery assumptions are gone; the implementation must avoid making machine state worse

Required placement:

- `Ember` fatal-stop code or explicitly trusted kernel boundary glue

## Operations That Are Sensitive But Not Automatically Ember Responsibilities

The following are sensitive, but they are not automatically machine-boundary operations:

- capability checks
- scheduler policy
- IPC policy
- compositor policy
- window ownership policy
- application-level input handling

These belong above `Ember` unless they require direct privileged hardware manipulation.

## Export Rule

When a hardware-dangerous operation must be usable by higher layers, `Ember` may expose only:

- typed handles
- typed register abstractions
- fixed-purpose functions with explicit preconditions
- narrow handoff structs

`Ember` must not export:

- raw addresses as ordinary capabilities
- unrestricted read or write primitives over arbitrary machine memory
- convenience wrappers that hide which dangerous operation is actually occurring

## Review Checklist

Any new low-level code must answer:

- which dangerous operation category it belongs to
- why it cannot stay above `Ember`
- what typed interface, if any, is exposed upward
- what machine invariant would fail if the code were wrong

## Forbidden Drift

The following changes are forbidden unless the safety boundary is revised explicitly:

- adding raw MMIO or port I/O helpers to higher layers
- exposing page-table writes outside `Ember`
- hiding privileged instruction use behind general utility helpers
- treating performance-sensitive code as justification for moving it into the machine boundary
