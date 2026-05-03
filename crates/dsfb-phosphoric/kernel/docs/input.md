# Input Model

This document defines the v0 input model for PhosphorOS.

The first input system is intentionally narrow and built around PS/2 keyboard and PS/2 mouse events routed through explicit kernel-owned queues and window focus state.

## Core Rules

The v0 input model is built on these rules:

- keyboard and mouse are the first supported input sources
- input events are bounded messages, not unbounded streams
- event delivery is routed through explicit focus and hit-testing rules
- input parsing and delivery must not allocate
- malformed device input is treated as untrusted data until validated

## Scope

The first input model covers:

- PS/2 keyboard input
- PS/2 mouse movement and button events
- routing to the shell, compositor, or focused window

Out of scope in v0:

- multitouch
- IME or advanced text composition
- game-controller support
- arbitrary hotplug device management

## Event Families

The first kernel-visible input events should support:

- key press
- key release
- mouse move
- mouse button press
- mouse button release
- redraw or focus-related delivery triggers where needed by the UI path

Rules:

- event payloads must remain bounded and fixed-format
- event types must be validated before routing
- input event queues must use fixed-capacity storage

## Device Boundary

Raw device interaction stays below the kernel input policy layer.

Rules:

- `Ember` owns machine-dangerous device access
- the kernel input layer consumes typed input data after trusted device-boundary parsing
- malformed device bytes or sequences must be rejected before they become ordinary UI events where possible

## Routing Rules

Input routing is explicit.

The first routing order is:

1. update kernel-owned cursor state from mouse movement
2. determine focused or hit-tested target window
3. deliver bounded event message to the appropriate task or shell component

Rules:

- pointer events are routed by cursor position and visible window hit-testing
- keyboard events are routed to the focused window or focused shell target
- event routing must honor window visibility and z-order
- destroyed or hidden windows must not continue receiving input

## Focus Model

The input system requires explicit focus state.

Rules:

- at most one window is keyboard-focused at a time in the first prototype
- mouse interaction may change focus through explicit policy
- focus changes must be reflected in kernel-owned fixed state

The focus policy may remain simple in v0, but it must not be ambient or inferred from unstored UI assumptions.

## Queueing And Delivery

Input events are delivered through bounded queues and IPC messages.

Rules:

- event queues must stay within the fixed-capacity IPC and task model
- overflow must become an explicit error or documented drop policy
- delivery to a blocked task must interact with explicit wake rules

## Cursor Model

The kernel owns cursor position and button-state tracking needed for routing.

Rules:

- cursor position is clamped to screen bounds
- mouse movement updates are applied before pointer-event routing
- cursor state remains bounded fixed data, not a dynamically growing history

## Error Handling

Expected input error cases include:

- malformed device packet
- invalid focus target
- event queue full
- unsupported input event type

Forbidden:

- treating malformed device input as trusted UI events
- panic as the normal queue-overflow strategy
- unbounded event accumulation

## Current Guarantees

This input model currently guarantees:

- PS/2 keyboard and mouse are the initial supported inputs
- routing depends on explicit focus and hit-testing state
- event delivery is bounded and queue-based
- malformed input remains a local validation problem, not an excuse for ambient trust

These guarantees define the intended input-routing model, not proof that every future device edge case is handled.

## Forbidden Drift

The following changes are forbidden unless this document and the IPC or window model are updated together:

- bypassing bounded event queues for convenience delivery paths
- routing keyboard or pointer events by ambient global assumptions instead of tracked focus and hit state
- introducing unsupported device families as if they were part of the v0 promise

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- richer text input systems
- gesture recognition
- hotplug device management
- complex accessibility event layers
