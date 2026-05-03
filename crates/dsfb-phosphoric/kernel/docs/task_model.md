# Task Model

This document defines the v0 task model for PhosphorOS.

The model is intentionally small and fixed-capacity. It is designed for the single address-space prototype described in the threat model and does not claim strong hostile-task isolation.

## Core Rules

The v0 task model is built on these rules:

- the system supports at most `TASK_MAX = 64` task descriptors
- every task has a fixed-size descriptor and fixed-size mutable state region
- task creation beyond capacity returns an explicit error
- task scheduling and state transitions are explicit kernel operations
- task control state is separate from capability policy, IPC policy, and GUI policy

## Scope

The v0 task model assumes:

- single address-space execution
- one CPU core in the first prototype
- fixed-capacity scheduler state
- fixed-capacity inbox and outbox channels attached through the IPC layer
- no demand-driven memory growth for task state

Out of scope in v0:

- hostile multi-tenant process isolation
- SMP task migration
- dynamic task stacks
- dynamic task priorities with arbitrary policy plugins

## Task Descriptor

Each task descriptor must contain fixed-size metadata for:

- task identifier
- lifecycle state
- scheduling state
- entry point or resume point reference
- fixed mutable state block reference
- capability table reference
- inbox and outbox references
- fault status

The descriptor must not contain:

- heap-owned runtime state
- unbounded message queues
- arbitrary application-managed raw pointers
- hidden ownership of global ambient authority

## Fixed Capacity

The kernel owns a fixed task table with `TASK_MAX` slots.

Rules:

- each slot is either free or bound to exactly one task descriptor
- task identifiers must not require heap allocation or global growth structures
- task creation fails explicitly when the task table is full
- task destruction returns the slot to the free pool only after the task's owned kernel state has been reclaimed through fixed-capacity cleanup paths

## Task State Machine

The v0 lifecycle states are:

- `Empty`
- `Created`
- `Ready`
- `Running`
- `Blocked`
- `Faulted`
- `Terminated`

Rules:

- `Empty` represents an unused descriptor slot
- `Created` means the descriptor exists but is not yet runnable
- `Ready` means the task may be scheduled
- `Running` means the task currently owns the CPU
- `Blocked` means the task is waiting on a bounded kernel event
- `Faulted` means the task hit a local failure state requiring trusted handling
- `Terminated` means execution has ended and the slot may be reclaimed after cleanup

Forbidden transitions:

- `Empty -> Running`
- `Blocked -> Running` without an explicit wake event
- `Faulted -> Ready` without an explicit recovery policy
- `Terminated -> Running`

## Scheduling Model

The first scheduler profile is intentionally narrow:

- cooperative yield is the baseline behavior
- timer integration is prepared for later bounded preemption work
- runnable task selection uses the fixed task table only
- the scheduler does not allocate while selecting the next task

Required scheduler invariants:

- at most one task is `Running` on the single-core prototype
- a task in `Running` state has a valid saved or live machine context
- a blocked task cannot be selected until an explicit wake condition occurs
- a terminated task cannot retain a runnable scheduling state

## Task Context

Task execution context is split across layers:

- low-level register save and restore belong to `Ember`
- task lifecycle and scheduling policy belong to the kernel task model

The task model therefore owns:

- which task should run next
- whether a task may be resumed
- which kernel-visible reason caused a block or wake

The task model does not own:

- raw machine register manipulation
- trap frame layout
- privileged context-switch instructions

## Fault Handling

The task model must support explicit local failure without turning ordinary errors into kernel-wide panics.

Rules:

- recoverable task-level failures should surface as local error results where possible
- unrecoverable task faults move the task into `Faulted` or `Terminated`
- fault status must be recorded in fixed descriptor state
- fault handling must preserve scheduler invariants and descriptor integrity

## Current Guarantees

This task model currently guarantees:

- fixed-capacity task management
- explicit lifecycle states
- explicit scheduling states in a single-core prototype
- deterministic failure on task-capacity exhaustion

These guarantees define the intended kernel model, not proof that every later implementation is race-free or verified.

## Forbidden Drift

The following changes are forbidden unless this document and the memory model are updated together:

- introducing heap-backed task metadata
- introducing unbounded runnable queues
- treating task creation as implicitly fallible through panic instead of explicit error
- assuming hostile-task isolation that the single address-space prototype does not provide

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- preemptive scheduling as a required default
- priority inheritance or advanced realtime policy
- process-isolated tasks with separate address spaces
- SMP scheduling
- application-extensible scheduler plugins
