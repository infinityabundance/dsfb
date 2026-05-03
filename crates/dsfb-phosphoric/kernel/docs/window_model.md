# Window Model

This document defines the v0 window model for PhosphorOS.

The first window system is fixed-capacity, rectangular, and intentionally conservative. It exists to support the first software-composited desktop slice without introducing a large GUI object system.

## Core Rules

The v0 window model is built on these rules:

- the system supports at most `WINDOW_MAX = 16` windows in v0.1 (per [docs/language/runtime_profile.toml](../../docs/language/runtime_profile.toml) `[handle_capacity]`; the original aspirational value of 256 is deferred to v0.2 and the reconciliation is recorded in [docs/PHOSPHOROS_DESIGN.md](../../docs/PHOSPHOROS_DESIGN.md))
- every window occupies one fixed-capacity table slot
- windows are rectangular
- window visibility and z-order are explicit kernel state
- window operations fail explicitly when capacity or state rules are violated

## Scope

The first window model covers:

- top-level rectangular windows
- ownership and visibility state
- z-order participation
- event-routing eligibility
- redraw participation

Out of scope in v0:

- nested window hierarchies
- non-rectangular windows
- transparent scene-graph shells
- arbitrary widget-tree retention in the kernel

## Window Descriptor

Each window entry must contain fixed-size metadata for:

- window identifier
- owner task reference
- geometry rectangle
- visibility state
- z-order membership
- event-routing eligibility
- redraw or damage flags

The descriptor must not contain:

- heap-owned decoration trees
- unbounded child lists
- arbitrary script state

## Fixed Capacity

The kernel owns a fixed window table with `WINDOW_MAX` slots.

Rules:

- window creation beyond `WINDOW_MAX` returns an explicit error
- each slot is either free or bound to one live window descriptor
- destruction returns the slot to the free pool after cleanup of related kernel state

## Geometry Model

Windows use integer rectangular geometry:

- `x`
- `y`
- `width`
- `height`

Rules:

- geometry updates are explicit kernel operations
- negative or invalid geometry must be rejected or clipped according to the interface contract
- visible drawing and hit-testing use the validated window rectangle

## Visibility States

The first visibility states are:

- `Hidden`
- `Visible`
- `Destroyed`

Rules:

- hidden windows are not routed pointer or keyboard events as visible targets
- visible windows participate in composition according to z-order
- destroyed windows are invalid for further UI operations

## Z-Order Participation

Window order is explicit.

Rules:

- every visible window may occupy one position in the bounded z-order model
- raising or lowering a window is an explicit operation
- z-order changes must not allocate or rebuild unbounded structures

## Ownership And Authority

Windows are authority-bearing kernel objects.

Rules:

- each window has an owning task or service identity in kernel state
- window operations require the appropriate window capability
- ownership does not imply unrestricted framebuffer access outside compositor mediation

## Event Routing Eligibility

Window descriptors participate in input routing.

Rules:

- only visible, eligible windows may receive pointer hit-tested events
- keyboard focus points to at most one eligible window in the first prototype
- destroyed or hidden windows must be removed from normal routing paths

## Redraw Participation

The window model must support bounded redraw state.

Rules:

- a window may be marked dirty or needing redraw through fixed descriptor flags
- redraw participation must remain compatible with the compositor's bounded damage strategy
- windows do not own unbounded backing-store growth in v0

## Error Handling

Expected error cases include:

- window table full
- invalid window handle
- operation on destroyed window
- geometry update rejected by validation rules
- z-order operation on a non-participating window

Forbidden:

- panic as the normal window-capacity failure path
- hidden allocation to support extra windows
- continuing to route events to invalid windows

## Current Guarantees

This window model currently guarantees:

- fixed-capacity window management
- rectangular geometry only
- explicit visibility and z-order state
- capability-scoped window authority

These guarantees define the required desktop-surface model, not proof that the future shell policy is complete.

## Forbidden Drift

The following changes are forbidden unless this document and the compositor or input docs are updated together:

- unbounded window lists
- hidden scene-graph retention in kernel window descriptors
- non-rectangular window behavior presented as part of v0
- routing or redraw behavior that bypasses explicit window state

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- nested window trees
- translucency-heavy surface models
- advanced decoration systems
- multiple desktops or virtual workspaces
