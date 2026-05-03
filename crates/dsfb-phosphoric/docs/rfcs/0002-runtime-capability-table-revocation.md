# RFC 0002: Runtime Capability Table And Stale-Handle Revocation

## Purpose

Define the first real runtime capability table for PhosphorOS after the compiler-assurance milestone lands cleanly.

## Current Reality

- capability rules are strongly specified
- compiler checks enforce affine use in source
- the running demo does not yet contain a real kernel capability table

## Proposal

- introduce a fixed-capacity kernel capability table
- represent handles as typed indices plus generation counters
- reject stale handles by generation mismatch on every validation path
- keep revocation deterministic and bounded

## Required Runtime Properties

- fixed table capacity
- explicit slot state
- generation increment on slot reuse
- no ambient global authority bypass
- no heap allocation in the fast path

## Non-Goals

- no process isolation claim upgrade in this RFC
- no application-visible borrow syntax
- no hardware capability extensions such as CHERI in this milestone

## Acceptance Signals For The Follow-On Milestone

- real kernel-side handle validation exists
- stale-handle tests fail before generation bump and pass after it
- `CLAIMS.md` can move capability-scoped runtime access from `SPECIFIED` to `ENFORCED`
