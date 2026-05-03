# Framebuffer Model

This document defines the v0 framebuffer contract for PhosphorOS.

The first graphics target is deliberately narrow: a software-rendered linear framebuffer obtained through the trusted `Ember` handoff boundary.

## Core Rules

The v0 framebuffer model is built on these rules:

- the framebuffer is a linear pixel surface
- framebuffer metadata is validated before use
- rendering is software-only
- drawing authority is capability-scoped
- clipping is rectangular only

## Scope

The first framebuffer path assumes:

- one active display target
- `UEFI`-provided linear framebuffer metadata
- no GPU acceleration
- no hardware compositing
- no dynamic mode switching in the first milestone

Out of scope in v0:

- multiple displays
- GPU command submission
- complex pixel shaders
- non-rectangular clipping regions

## Required Metadata

The framebuffer boundary must validate:

- width
- height
- stride or pitch
- bytes per pixel or pixel format description
- base address or trusted surface reference

Rules:

- invalid or inconsistent metadata is rejected at the trusted boundary
- higher layers must not reinterpret unvalidated framebuffer metadata
- framebuffer state must not be inferred from ambient globals

## Pixel Format Assumptions

The first implementation must lock to one software-renderable pixel format for the active target.

Rules:

- the pixel format must be named explicitly in kernel code or trusted handoff data
- pixel writes must respect the validated bytes-per-pixel and stride values
- pixel format conversion complexity should be avoided in the first milestone

The initial goal is correctness and auditability, not broad format support.

## Drawing Surface

The framebuffer is treated as a bounded surface with:

- origin at the top-left corner
- integer pixel coordinates
- width and height bounds from validated metadata

Rules:

- every drawing primitive must remain inside the validated surface bounds after clipping
- out-of-bounds drawing must be clipped or rejected explicitly
- drawing must not depend on unchecked pointer arithmetic outside the trusted boundary

## Clipping Rules

The v0 clipping model is rectangular only.

Rules:

- every draw operation is clipped against a rectangular target region
- partially visible primitives are reduced to the visible rectangle
- fully out-of-bounds primitives become no-ops or explicit rejected operations depending on the interface

Forbidden:

- arbitrary polygon clipping
- non-rectangular window masks
- hidden fallback drawing outside the declared clip

## Required Primitives

The initial framebuffer contract must support these software primitives:

- clear surface
- fill rectangle
- draw line
- draw glyph from the built-in bitmap font
- blit a bounded bitmap region

The first vertical slice needs at minimum:

- full-screen clear
- window rectangle fill
- cursor drawing

## Authority Model

Framebuffer use is capability-scoped.

Rules:

- code may draw only if it holds the appropriate framebuffer or compositor draw capability
- framebuffer authority does not imply unrestricted raw memory authority
- direct framebuffer writes must remain behind reviewed drawing interfaces where possible

## Error Handling

Framebuffer operations must fail explicitly when validation or authority checks fail.

Expected error cases:

- invalid surface metadata
- invalid clip rectangle
- unsupported pixel format for the active software path
- missing draw authority

Forbidden:

- using panic as the normal clipping or bounds-failure strategy
- silently drawing past the end of the validated framebuffer

## Current Guarantees

This framebuffer model currently guarantees:

- one validated linear framebuffer target
- software-only rendering
- rectangular clipping
- capability-scoped draw authority

These guarantees define the required graphics surface contract, not proof of rendering correctness.

## Forbidden Drift

The following changes are forbidden unless this document and the MMIO or bring-up docs are updated together:

- bypassing metadata validation before drawing
- assuming GPU acceleration exists in the first milestone
- exposing unrestricted raw framebuffer access as a convenience API
- adding non-rectangular clipping without an explicit reviewable design

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- multi-display support
- hardware acceleration
- multiple runtime pixel formats
- alpha-composited scene graphs
- dynamic mode switching
