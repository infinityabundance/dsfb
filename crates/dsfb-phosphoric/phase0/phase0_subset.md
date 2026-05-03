# Phosphoric Subset for Phase 0

This document pins the minimal Phosphoric subset that [phase0_compiler.phos](phase0_compiler.phos) accepts. It is strictly smaller than v0; it exists only to compile [compiler/pcc.phos](../compiler/pcc.phos) into stage 1. From stage 1 onward, the full v0 grammar is available.

The subset is closed. Adding any production here is a doctrine change requiring a new phase 0 binary build and re-attestation.

## Accepted surface

### Module declaration

```
module dotted.path.identifier;
profile {boot|host|trusted|runtime};
```

Exactly one `module` line per file. Optional `profile` line follows.

### Type declarations

- **Primitive types:** `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `bool`, `()`.
- **Fixed arrays:** `[T; N]` where `T` is a primitive, struct, or fixed array; `N` is a literal `u32`.
- **Slices:** `Slice[T, N]` (capacity-bounded, fixed-size).
- **Result / Option:** `Result[T, E]`, `Option[T]`.
- **Structs:** `struct Name { field: T, ... }`. Fields are primitive, fixed array, slice, struct, enum, Result, Option, or other declared struct.
- **Enums:** `enum Name { Variant, Variant(payload, ...), ... }` where each payload is a primitive, struct, or fixed array.
- **Capability declarations:** `capability Name;` (opaque to phase 0 — recorded as a name; full move-state tracking happens in stage 1+).

### Function declarations

```
fn name(param: Type, ...) -> ReturnType {
    body
}
```

- No closures, no trait methods, no generics.
- Effect annotations (`effects(...)`) are parsed and recorded but not enforced; phase 0 trusts the in-source declaration; pcc.phos does the enforcement.
- No recursion: a function may not call itself directly. Mutual recursion is rejected by the call-graph check that phase 0 must run.

### Statements

- `let [mut] name [: Type] = expr;`
- `name = expr;` (assignment to a `let mut` binding only).
- `match scrutinee { pat => arm, ... }`
- `for var in 0..N { ... }` always wrapped by `#[bound = N_LITERAL]` immediately preceding. The literal must equal the loop's static upper bound.
- Expression statements (`expr;`) for expressions whose result is `()`.
- `return expr;` (only as the final statement in a block; otherwise the block's tail expression is the return value).

### Expressions

- Path: dotted module-prefixed identifier (e.g. `pcc.lexer.Token::Ident`).
- Literal: integer, bool (`true`/`false`), unit (`()`), char-as-byte (`b'x'`).
- Binary ops: `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`. Phase 0 lacks bitwise operators (a long-tail v0 grammar extension is `&`, `|`, `^`, `<<`, `>>` per the panel; phase 0 stays before that extension because pcc.phos does not use them in v0).
- Field access: `expr.name`.
- Array indexing: `expr[expr]`.
- Function call: `expr(args)`.
- Match expression (same shape as match statement).
- Block expression `{ ... }`.
- Result/Option construction: `Ok(expr)`, `Err(expr)`, `Some(expr)`, `None`.
- Struct/enum-variant construction: `Path { field: expr, ... }` or `Path::Variant(args)` or `Path::Variant { field: expr, ... }`.

### Match patterns

- Wildcards: `_`.
- Literal patterns: integer, bool, unit, char-as-byte.
- Binding patterns: `name`.
- Variant patterns: `Path::Variant`, `Path::Variant(pat, ...)`, `Path::Variant { field: pat, ... }`.

### Attribute support

Only `#[bound = N]` is recognized, only as the immediate preceding attribute on a `for` loop. Any other attribute is rejected.

## Rejected surface

Phase 0 explicitly rejects (these are valid v0 but not in phase 0):

- Profile-specific effect labels beyond the parser-recorded form (effect closure is pcc.phos's job).
- Trusted blocks (`trusted!{ ... }`). Phase 0 itself runs in host profile; it parses but does not lower trusted blocks.
- Custom attributes beyond `#[bound = N]`.
- Path-sensitive capability tracking (parsed; emitted to HIR with `Held` state; full tracking is pcc.phos's job).
- Diagnostic message catalog beyond the minimal set below.

## Diagnostic codes (phase 0)

Phase 0 uses the prefix `P0-###` for its own diagnostics so they are distinguishable from pcc.phos's L-/P-/M-/etc. codes.

| Code | Trigger |
|---|---|
| P0-001 | Unrecognized token in source |
| P0-002 | Unexpected token in parse |
| P0-003 | Construct outside the phase 0 subset |
| P0-004 | Unbounded for-loop |
| P0-005 | Recursive call detected |
| P0-006 | Unknown name reference |
| P0-007 | Type arity mismatch |
| P0-008 | Match pattern not exhaustive |

Phase 0 does not register pcc.phos's M-### / K-### / W-### / E-### / S-### codes; those land when stage 1 takes over.

## Output

Phase 0 emits one ELF executable: stage 1, a self-hosted `pcc.phos` binary.

The ELF target is `x86_64-unknown-linux-gnu`. No other targets in phase 0 (cross-compilation to other architectures lands when stage 1 is the compiler). Stage 1 produces stage 2 produces stage 3; `verify_fixpoint.phos` asserts byte-equality from stage 2 onward.

## Determinism

Phase 0's output is a pure function of (phase 0 source, pcc.phos source, build flags). Two attesters running phase 0 against the same `pcc.phos` source MUST produce byte-identical stage 1 binaries. Any divergence is either a phase 0 nondeterminism bug (broken doctrine; phase 0 must be re-spec'd) or an attester error (re-build under pinned conditions).
