# IPC Model

This document defines the v0 inter-process communication model for PhosphorOS.

The IPC design is fixed-capacity and message-oriented. It avoids heap growth, shared mutable message pools with unbounded structure, and ambient communication authority.

## Core Rules

The v0 IPC model is built on these rules:

- communication uses fixed-capacity channels only
- messages are bounded values, not heap-owned graphs
- the system supports at most `CHANNEL_MAX = 128` channels
- total queued messages across the runtime are bounded by `GLOBAL_MSG_MAX = 1024`
- each message payload is bounded by `IPC_PAYLOAD_MAX = 256` bytes

## Scope

The first IPC model is for:

- task-to-task messaging inside the single address-space prototype
- kernel services and applications communicating through explicit channel handles
- GUI event delivery through bounded message forms

Out of scope in v0:

- distributed IPC
- shared-memory fast paths that bypass message validation
- dynamically resized channels
- stream abstractions with unbounded buffering

## Channel Model

A channel is a kernel-managed communication object with fixed-capacity state.

Each channel has:

- a channel identifier
- endpoint rights metadata
- a bounded ring buffer
- head and tail indices
- occupancy state
- sender and receiver attachment metadata

Rules:

- channel descriptors are kernel-owned fixed entries
- channel creation beyond `CHANNEL_MAX` returns an explicit error
- channel state changes must not allocate
- channel handles are capability-governed and must not be ambient

## Message Model

Every message has fixed, reviewable structure.

Required message parts:

- message kind or tag
- sender-visible payload bytes or typed fields
- payload length bounded by `IPC_PAYLOAD_MAX`

Rules:

- payloads larger than `IPC_PAYLOAD_MAX` are rejected before enqueue
- message bodies must be copyable or moveable within fixed storage
- messages must not depend on heap-backed buffers
- external or peer-provided payloads are untrusted until validated by the receiver

## Ring Buffer Discipline

Each channel uses a bounded ring buffer.

Rules:

- enqueue writes to the tail slot and advances the tail index
- dequeue reads from the head slot and advances the head index
- full and empty states must be distinguishable without allocation
- a channel must never overwrite unread messages implicitly

Required invariants:

- ring indices remain within the channel's fixed capacity
- enqueue on a full ring is an explicit error unless a later documented drop policy exists
- dequeue from an empty ring is an explicit error or empty-result case

## Global Capacity

The runtime must also respect the global queued-message bound:

- `GLOBAL_MSG_MAX = 1024`

Rules:

- the kernel tracks total queued message occupancy
- enqueue must fail explicitly if accepting the message would exceed the global bound
- global accounting must remain fixed-size and deterministic

This prevents the system from hiding unbounded growth behind many small per-channel queues.

## Endpoint Rights

Channel use is capability-scoped.

The initial rights model distinguishes:

- send
- receive
- both send and receive

Rules:

- a task may send only if it holds a send-capable endpoint
- a task may receive only if it holds a receive-capable endpoint
- endpoint rights do not arise from task identity alone
- duplicating endpoint authority follows the capability model rather than ad hoc task-global rules

## Blocking And Wake Semantics

The IPC model interacts with the task model through explicit bounded wait states.

Rules:

- send may block only through an explicit kernel policy for full channels
- receive may block only through an explicit kernel policy for empty channels
- blocked IPC waits must map to explicit task states
- wakeup must occur through explicit enqueue or dequeue events, not ambient polling assumptions

The default v0 design should prefer simple explicit error returns over complex hidden waiting behavior unless a later document narrows the policy more precisely.

## Validation Rules

Receivers must treat incoming messages as untrusted data.

Required receiver checks:

- validate message kind
- validate payload length
- validate typed field ranges before acting on them
- reject malformed or unsupported messages locally

Forbidden:

- trusting peer payload layout without validation
- reading beyond the declared payload size
- treating IPC input as proof of capability possession

## Error Handling

IPC failure must be explicit.

Expected error cases include:

- channel full
- channel empty
- channel table full
- payload too large
- unsupported message kind
- missing send or receive rights

Forbidden:

- allocating around IPC exhaustion
- panic as the normal full-queue behavior
- silently dropping messages unless the interface explicitly documents that policy

## Current Guarantees

This IPC model currently guarantees:

- channel and message storage are fixed-capacity
- send and receive authority are explicit
- queue exhaustion is handled as an explicit failure
- total queued message growth is globally bounded

These guarantees describe the required kernel design, not proof that every later implementation preserves fairness or liveness.

## Forbidden Drift

The following changes are forbidden unless this document and the memory model are updated together:

- heap-backed message queues
- implicit shared-memory IPC paths that bypass validation
- channel growth past declared capacity
- hiding payload overflows behind silent truncation without an explicit interface contract

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- zero-copy IPC protocols
- cross-address-space IPC
- priority-aware message scheduling
- richer capability transfer semantics
- typed protocol verification beyond the initial bounded message model
