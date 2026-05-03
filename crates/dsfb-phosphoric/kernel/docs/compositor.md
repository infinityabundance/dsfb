# Compositor Model

This document defines the v0 compositor model for PhosphorOS.

The compositor is a small software layer that combines fixed-capacity windows into the validated framebuffer target. It is deliberately simpler than a general desktop compositor.

## Core Rules

The v0 compositor is built on these rules:

- software compositing only
- fixed-capacity window set
- explicit z-order
- rectangular clipping only
- no hidden allocation in composition

## Scope

The first compositor is responsible for:

- maintaining the visible window list
- compositing windows into the framebuffer
- drawing the cursor
- redrawing affected regions through bounded software operations

Out of scope in v0:

- GPU-accelerated composition
- animated scene graphs
- arbitrary transform stacks
- translucency-heavy desktop effects

## Surface Model

The compositor operates over:

- the validated framebuffer target
- a fixed-capacity window table
- bounded damage or redraw regions

Rules:

- every visible region is represented by rectangles
- redraw work must remain bounded by the fixed-capacity window set and clip rectangles
- the compositor must not allocate while processing a redraw path

## Z-Order

Windows are ordered explicitly from back to front.

Rules:

- z-order is stored in bounded kernel-owned state
- bringing a window forward or sending it backward is an explicit compositor operation
- the compositor must never rely on implicit creation order as a substitute for tracked z-order state

## Composition Rules

The v0 composition pipeline is:

1. clear background or redraw root surface
2. composite windows from back to front
3. draw cursor last

Rules:

- fully obscured window regions should not require redundant final writes if the compositor can avoid them with bounded logic
- partially visible windows must be clipped against visible rectangles
- compositor logic must not draw outside the validated framebuffer clip

## Damage Handling

The compositor may use bounded damage tracking, but it must remain fixed-capacity.

Rules:

- damage representation must remain rectangular and bounded
- if fine-grained damage tracking is not yet implemented, whole-screen redraw is acceptable for v0
- damage bookkeeping must not allocate

The first vertical slice may use conservative redraw behavior so long as it stays explicit and bounded.

## Cursor Rules

The cursor is a compositor-managed overlay.

Rules:

- cursor position is clipped to the visible screen bounds
- cursor drawing occurs after windows are composited
- cursor updates must not require unbounded backing-store logic in v0

## Authority Model

The compositor mediates draw authority.

Rules:

- applications do not own arbitrary global drawing rights
- application rendering flows through window or compositor capabilities
- the compositor decides final visibility order and clip enforcement

## Error Handling

Expected compositor error cases include:

- invalid window reference
- invalid z-order operation
- missing draw authority
- redraw request against a destroyed or hidden window state

Forbidden:

- panic as the normal redraw failure path
- hidden allocation to recover from damage or z-order complexity

## Current Guarantees

This compositor model currently guarantees:

- software-only composition
- explicit z-order
- rectangular clipping
- bounded redraw logic suitable for the first milestone

These guarantees define the intended graphics composition model, not proof of flicker-free or fully optimized rendering.

## Forbidden Drift

The following changes are forbidden unless this document and the framebuffer or window model are updated together:

- GPU-first composition assumptions
- implicit global draw authority outside compositor mediation
- unbounded scene or damage structures
- desktop effects that meaningfully widen the review surface without necessity

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- hardware composition
- alpha-heavy desktop effects
- non-rectangular windows
- sub-surface trees with deep retained composition semantics
