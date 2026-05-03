# Phosphoric v0.1 Memory Model

This document defines the initial memory model for Phosphoric v0.1 and the runtime profile expected by PhosphorOS v0.

The memory model is designed around fixed-size layouts, fixed capacities, and explicit failure on exhaustion. The v0 profile rejects heap allocation in the runtime path.

## Core Rules

The v0.1 memory model is built on these rules:

- all runtime storage must have a statically known layout
- the runtime path must not depend on heap allocation
- fixed-capacity tables, queues, buffers, and state blocks are the default design
- growth beyond declared capacity must become an explicit error or a rejected operation
- no language feature may hide allocation behind ordinary syntax

## Storage Classes

Phosphoric v0.1 assumes these storage classes:

### Read-Only Static Data

Examples:

- code
- constant tables
- packed assets
- compile-time configuration values

Rules:

- read-only static data is placed in immutable memory regions
- static assets are loaded from the boot image or packed resource blob
- mutation of read-only static data is forbidden

### Mutable Static Regions

Examples:

- kernel-global fixed tables
- device state blocks
- compositor state
- scheduler state

Rules:

- mutable static regions must have compile-time known maximum size
- the number of entries in a global table is fixed by configuration
- mutable statics are restricted to well-defined runtime subsystems and must not act as ambient authority

### Stack Storage

Rules:

- stack-allocated values must have compile-time known size
- stack frames must stay analyzable under the bounded-control-flow profile
- unbounded recursion is forbidden because it defeats predictable stack use
- configured entrypoints may be checked against explicit worst-case stack budgets
- large data should remain in fixed regions rather than being copied onto stacks opportunistically

### Borrowed Views

Rules:

- borrowed views such as `Slice[T, N]` do not allocate
- a borrowed view carries bounds information as part of its type-level contract
- creating a view does not create a new backing allocation

## No Heap Rule

The runtime profile forbids:

- general-purpose heap allocators
- per-object dynamic allocation
- hidden allocation in string, collection, or closure machinery
- runtime growth of tables, queues, or buffers past declared capacity

Bootstrap note:

- a bootstrap-only bump allocator may be tolerated below the Phosphoric runtime path if it is isolated inside trusted bring-up code and removed from steady-state execution
- such usage belongs to `Ember` or boot-specific code, not the ordinary language/runtime contract

## Fixed-Capacity Design

The first prototype locks these system capacities:

- `TASK_MAX = 64`
- `WINDOW_MAX = 256`
- `CHANNEL_MAX = 128`
- `GLOBAL_MSG_MAX = 1024`
- `IPC_PAYLOAD_MAX = 256`
- `WIDGET_TREE_DEPTH = 32`
- `FILENAME_MAX = 64`

These capacities are part of the v0 design, not placeholders to be replaced with heap growth.

## Object Layout

Rules:

- every value has a compile-time known representation for the active ABI profile
- arrays include element count in the type, not in hidden metadata
- bounded slices carry a bounded-view contract rather than implying ownership of resizable storage
- structs and enums must not acquire hidden heap headers, reference counts, or virtual dispatch tables

The exact ABI is defined separately in `docs/abi.md`, but the memory model requires the representation to stay explicit and fixed-size where values are stored.

## Region Discipline

The runtime should be understood as a set of fixed regions:

- code region
- read-only constant region
- fixed mutable kernel region
- per-task fixed state blocks
- fixed inbox and outbox ring buffers
- optional framebuffer region

Rules:

- region sizes are set by build configuration or static layout planning
- tasks do not request arbitrary new regions at runtime
- message queues and window tables consume preallocated capacity from fixed regions

## Exhaustion Behavior

Capacity exhaustion must never silently widen memory use.

Required behavior:

- task creation beyond `TASK_MAX` returns an explicit error
- window creation beyond `WINDOW_MAX` returns an explicit error
- channel creation beyond `CHANNEL_MAX` returns an explicit error
- message enqueue beyond ring-buffer capacity returns an explicit error or drops according to a documented policy
- oversized IPC payloads are rejected before write

Forbidden behavior:

- automatic reallocation
- panic as the normal exhaustion strategy
- implicit fallback to host allocation
- silent truncation unless the interface explicitly documents that policy

## Current Guarantees

The v0.1 memory model currently guarantees this design intent:

- runtime memory use is meant to be deterministic and bounded by configuration
- the ordinary Phosphoric surface does not expose heap allocation
- fixed capacities are part of interface design, not an implementation accident
- exhaustion is handled as an explicit local failure
- the current compiler can report deterministic type layouts, per-function frame sizes, and worst-case stack depth for configured entrypoints

These guarantees describe the required system profile, not proof that every implementation obeys it yet.

## Forbidden Constructs

The memory model forbids:

- garbage collection
- hidden allocator calls
- dynamically resized arrays
- heap-backed strings
- recursive data structures that require unbounded allocation
- ambient global mutable state without a documented region role

## Future Work That Is Not Yet Promised

The following remain outside the v0.1 contract:

- full static region or global-table memory budgeting
- stack analysis for future runtime objects that are not yet represented in the frozen source model
- multiple memory profiles for different deployment classes
- process-isolated address spaces for hostile tasks
- writable filesystems with dynamic metadata growth
- any runtime model that depends on transparent background allocation
