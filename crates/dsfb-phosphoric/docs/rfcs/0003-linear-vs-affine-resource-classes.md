# RFC 0003: Linear Versus Affine Resource Classes

## Purpose

Separate finite kernel resources that must be returned explicitly from ordinary affine capabilities that may be dropped.

## Current Reality

- capabilities are affine by default
- the compiler rejects duplication
- the language does not yet model explicit release obligations

## Questions This RFC Must Answer

- which resources are safe to drop
- which resources must be linearly returned
- how explicit release is represented without widening the frozen v0 subset prematurely
- how the kernel records resource return and exhaustion deterministically

## Candidate Future Split

- affine: read-only views, timer tickets, non-owning references to static resources
- linear: window slots, channel endpoints, framebuffer ownership, finite scheduler-owned objects

## Non-Goals

- no grammar change in the frozen-v0 milestone
- no region declarations in this RFC
- no borrow syntax

## Acceptance Signals For The Follow-On Milestone

- a reviewed language/runtime design names the first linear resource classes
- `CLAIMS.md` can distinguish current affine enforcement from future linear return obligations
