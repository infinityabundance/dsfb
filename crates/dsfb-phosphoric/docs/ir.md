# Phosphoric v0 Compiler IR

This document defines the initial intermediate representation for the Phosphoric compiler prototype.

The IR is intentionally small, typed, and non-SSA. It exists to give the frontend, type/effect checks, and code generation a common lowered form that is easy to inspect and audit.

## IR Goals

The v0 IR is designed to preserve:

- explicit value types
- explicit moves
- explicit effectful calls
- explicit control-flow edges
- explicit failure returns
- bounded, structured lowering targets

The IR is not designed for optimization first.

## Design Constraints

The initial IR must remain:

- typed
- non-SSA
- block-structured
- easy to print and debug
- easy to lower to the prototype `x86_64` ABI

The initial IR does not attempt:

- global optimization
- speculative transforms
- alias-heavy memory rewriting
- implicit exception edges

## Unit Structure

The IR hierarchy is:

```text
Program
  Module*
    Function*
      Block*
        Instruction*
        Terminator
```

Definitions:

- `Program`: complete compiler unit
- `Module`: named source module after parsing and name resolution
- `Function`: typed function body with explicit parameters, return type, and effect set
- `Block`: ordered list of instructions ending in one terminator

## Function Shape

Each function records:

- symbol name
- parameter list with types
- return type
- declared effect set
- local slot declarations
- entry block identifier

This keeps the source-level interface visible after lowering.

## Value Model

The IR distinguishes:

- local slots
- temporary values
- immediate constants

Rules:

- local slots represent named storage locations after lowering from source bindings
- temporaries represent intermediate results
- moves are explicit instructions, not implied by assignment
- non-owning views such as `Slice[T, N]` stay explicit typed values rather than implicit aliasing side effects

## Type Model

IR types mirror the source profile closely:

- integer widths
- `bool`
- fixed arrays
- bounded slices
- structs
- enums
- capabilities
- `Option`
- `Result`

The IR must not invent heap-only or dynamically sized runtime types that do not exist in the language profile.

## Core Instructions

The initial instruction set is intentionally small:

```text
const        ; create an immediate scalar value
move         ; move from a local slot or temporary
store        ; write to a local slot or field place
load_field   ; read a field from a struct-like place
make_struct  ; construct a struct value
make_enum    ; construct an enum variant
match_enum   ; inspect enum tag and expose payload bindings
make_array   ; construct a fixed array value
slice_view   ; create a bounded slice view
call         ; call a named function with explicit effect set
cast_int     ; explicit integer conversion where allowed
cmp          ; comparison producing `bool`
binop        ; arithmetic or logical scalar operation
```

Every instruction has typed operands and a typed result when applicable.

## Terminators

Every block ends with one of:

```text
goto         ; unconditional branch
branch       ; branch on a boolean value
return       ; explicit return with optional value
fail         ; explicit lowering hook for trusted irrecoverable boundary cases only
```

Rules:

- ordinary recoverable failure is represented as `Result`, not `fail`
- `fail` is reserved for trusted lowering points that correspond to irrecoverable runtime boundaries and should remain rare
- no implicit fallthrough exists between blocks

## Effect Representation

Each function and call site records explicit effects.

Rules:

- the function header records its declared effect set
- each `call` instruction references a callee with a known effect set
- effect checking happens before code generation and must reject missing effects
- the IR never treats effects as comments or debug metadata

## Control Flow

The v0 IR supports only bounded, explicit control flow:

- straight-line blocks
- conditional branches
- lowering of bounded `for` loops into explicit blocks and branches
- `match` lowered into explicit discriminant tests and branches

Forbidden in the IR:

- implicit exceptions
- coroutine suspension
- hidden loop growth constructs
- recursive control operators

## Ownership And Borrow Lowering

The IR must preserve ownership semantics explicitly enough for simple checking:

- `move` consumes the source binding for move-only values
- capability values remain opaque typed values
- lowering must not duplicate affine capability values silently

Borrow-specific IR instructions are deferred because borrow syntax is outside the frozen v0 language surface.

The first compiler may perform only simple move-state analysis, but the IR must not erase ownership intent.

## Error Lowering

Recoverable failure is lowered as ordinary data and control flow.

Rules:

- `Result` and `Option` remain explicit typed values in the IR
- propagation lowers to explicit branch and return paths
- no exception tables or unwinding edges are introduced

## Current Guarantees

The v0 IR currently guarantees this design intent:

- typed lowering remains close to source intent
- control flow is explicit
- effect requirements remain visible
- ordinary failure remains data-driven
- ownership-sensitive operations remain explicit enough for later checking

## Forbidden In v0

The initial IR forbids:

- SSA phi machinery
- optimizer-only pseudo-operations
- heap allocation instructions
- exception or unwinding instructions
- hidden dynamic dispatch instructions
- backend-specific opcodes mixed into the general IR

## Future Work That Is Not Yet Promised

The following are intentionally deferred:

- separate MIR and LIR layers
- SSA conversion
- aggressive dataflow optimization
- backend-independent register allocation IR
- verified lowering passes
