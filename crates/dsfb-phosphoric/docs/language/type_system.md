# Phosphoric v0.1 Type System

This document defines the initial type-system profile for Phosphoric v0.1.

The design target is a type system that is narrow enough to audit and implement directly, while still enforcing explicit ownership, bounded memory representation, and capability-scoped authority.

## Design Objectives

The v0.1 type system exists to enforce:

- fixed-size data representation
- explicit ownership transfer
- affine capability handling
- bounded aggregate data
- explicit fallibility

It does not attempt to maximize abstraction power.

## Primitive Types

Phosphoric v0.1 includes these primitive scalar types:

- unsigned integers: `u8`, `u16`, `u32`, `u64`
- signed integers: `i8`, `i16`, `i32`, `i64`
- boolean: `bool`

Current guarantees:

- integer widths are explicit in source
- there is no implicit widening or narrowing between integer types
- `bool` is distinct from integer types

Forbidden in v0.1:

- floating-point types
- machine-sized implicit integer types
- pointer types in ordinary source code

## Compound Types

### Arrays

Syntax:

- `[T; N]`

Rules:

- `N` is a compile-time constant literal in v0.1
- array length is part of the type
- arrays have fixed layout and fixed element count
- array indexing semantics will remain bounded by the language profile rather than exposing raw pointer arithmetic

### Bounded Slices

Syntax:

- `Slice[T, N]`

Rules:

- a bounded slice is a view type with an element type and a compile-time maximum length
- the maximum capacity `N` is part of the type
- operations on a slice must preserve the declared bound
- bounded slices do not imply heap allocation or resizable storage

### Structs

Rules:

- structs are nominal product types
- field layout is fixed for a given compilation target and ABI profile
- fields are typed explicitly
- no hidden vtable or runtime object header is introduced

### Enums

Rules:

- enums are nominal tagged unions
- each variant may be empty or tuple-like
- pattern matching is the intended consumer of enum values
- `Option`-like and `Result`-like modeling should use explicit enums or the built-in forms described below

## Built-In Sum Types

### Option

Syntax:

- `Option[T]`

Variants:

- `Some(T)`
- `None`

Purpose:

- represent presence or absence without null pointers

### Result

Syntax:

- `Result[T, E]`

Variants:

- `Ok(T)`
- `Err(E)`

Purpose:

- represent explicit fallibility
- make error propagation part of the typed interface

## Capability Types

Capabilities are opaque nominal types that represent authority.

Examples:

- `Window`
- `Framebuffer`
- `Channel`
- `Timer`
- `DevicePort`

Rules:

- capabilities are affine by default
- copying a capability is forbidden unless the capability type is explicitly declared shareable by a later rule
- capabilities are passed explicitly as function parameters or returned explicitly as values
- capability values must not be forged from integers, raw addresses, or untyped byte sequences

Current guarantee:

- authority-bearing values are modeled as distinct types, not as ambient globals or untyped handles

## Ownership And Moves

The ownership model is move-oriented by default.

Rules:

- binding a value creates one owner unless the type is explicitly copyable by rule
- passing an owned value by move invalidates the previous binding
- returning an owned value moves it to the caller
- using a moved value is a type error

In v0.1, ordinary data types should be treated conservatively:

- affine capability values are always move-only
- fixed-size plain data may later gain copy semantics, but that is not the default assumption in the initial type checker design

## Borrowing Is Deferred

Borrow syntax is not part of the frozen v0 surface.

Current rules:

- `&x`, `&mut x`, and borrowed parameters are rejected as out-of-profile syntax
- the current checker enforces move-oriented ownership only
- future borrow support, if added at all, must update `docs/language/V0_FREEZE.md` first

Current non-guarantee:

- v0 does not currently claim source-level borrow checking or lifetime reasoning

## Mutability

Rules:

- mutability is explicit at bindings
- immutable bindings cannot be reassigned
- mutable access currently requires ownership of a mutable binding

The type system does not treat mutability as ambient or inferred authority.

## Type Equality And Compatibility

Rules:

- type equality is nominal for capabilities, structs, and enums
- array lengths participate in type identity
- slice capacity bounds participate in type identity
- `Result[T, E]` and `Option[T]` are invariant in v0.1
- implicit coercions are minimized

Forbidden in v0.1:

- subtyping hierarchies
- inheritance-based compatibility
- implicit conversion chains

## Forbidden Constructs

The initial type system forbids:

- unrestricted raw pointers
- nullable pointer-like values outside `Option`
- unbounded container types
- hidden reference counting
- trait-object style erased dynamic dispatch
- lifetime-erasing escape hatches
- type-level reflection
- exceptions as an alternative to typed `Result`

## Current Guarantees

Phosphoric v0.1 currently guarantees this type-system intent:

- bounded data has bounded type-level representation
- authority-bearing values are typed as capabilities
- moved-value reuse should be rejected, including conservative branch and loop joins
- capability duplication should be rejected, including conservative branch joins
- borrow-like syntax is rejected because it is outside the frozen v0 surface

These guarantees describe the required compiler direction for v0.1, not a formal proof.

## Future Work That Is Not Yet Promised

The following areas are intentionally deferred:

- source-level borrow syntax and lifetime rules
- richer generic polymorphism
- shareable capability classes with more than one ownership mode
- stronger type-state or protocol-state reasoning
- proof-carrying or refinement-style types
