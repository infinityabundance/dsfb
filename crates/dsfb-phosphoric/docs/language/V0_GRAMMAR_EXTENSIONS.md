# V0 Grammar Extensions

This document records every extension applied to the v0 grammar after the [V0_FREEZE.md](V0_FREEZE.md) was originally pinned. Each extension has a rationale, the implementation site that forced it, and the date it landed.

The bar for landing an extension is high. Each must:

1. Be **forced by an existing implementation**, not added on speculation.
2. Be the **smallest change** that resolves the gap.
3. Pass the **same scrutiny** the original v0 freeze got: a non-goal section in [LANGUAGE_NON_GOALS.md](../../LANGUAGE_NON_GOALS.md), a [grammar.md](grammar.md) update, and a conformance-corpus pair (positive + negative test).

A "would be nicer" addition that doesn't satisfy all three is rejected.

## Extension log

### Extension 2026-04-27a — Unit type and unit expression

- **Form added to grammar:**
  - `PrimaryExpr` gains `UnitExpr`
  - `UnitExpr ::= "(" ")"`
- **Implementation site that forced it:** [compiler/lexer.phos](../../compiler/lexer.phos) lines 645–736 use `false => ()` in single-branch dispatch matches as a no-op fall-through. Without a unit literal, every such arm requires synthesizing a sentinel value of the arm's value type, which makes the lexer's keyword-dispatch tables verbose and error-prone.
- **Why this is minimal:** The unit type `()` is already implied by functions declared without a `ReturnType`. Making it spellable as a `PrimaryExpr` is the smallest addition that lets `match` arms produce unit explicitly. No new types, no new keyword.
- **Constraints:**
  - The unit expression has type `()` (the unit type, with one inhabitant).
  - It is permitted wherever `PrimaryExpr` is permitted but is only useful in unit-typed contexts.
  - Pointless unit construction (e.g. `let x: () = ();` followed by no use) may be flagged by the type checker with diagnostic `K-017` ("unit literal in a non-unit context"). Reserved; not yet emitted.
- **Conformance:**
  - pass: `tests/conformance/boot/pass/unit_literal_in_match.phos`
  - fail: `tests/conformance/boot/fail/unit_literal_type_mismatch.phos` (assigning `()` to a `u32` binding)
- **Diagnostic codes reserved:** `K-017` (unit literal type mismatch).

### Extension 2026-04-27b — Array indexing on the LHS of assignment

- **Form added to grammar:**
  - `Place ::= Ident ("." Ident)* PlaceIndex?`
  - `PlaceIndex ::= "[" Expr "]"`
- **Implementation site that forced it:** [compiler/lexer.phos](../../compiler/lexer.phos) lines 761 and 827 write `out[count] = token;` to fill a caller-supplied fixed-capacity buffer. Phosphoric's primary collection type *is* the fixed-capacity buffer (`[T; N]` and `Slice[T, N]`), and indexed assignment is the natural form for filling one. The original `Place ::= Ident ("." Ident)*` permitted only dotted-path stores, which forced helper functions like `set_at(buf, i, v)` everywhere.
- **Why this is minimal:** The grammar already permits array indexing as an *r-value* expression (via `ArrayExpr`-on-RHS plus `PostfixExpr` chains). Symmetry on the *l-value* side is the smallest change. No new operators, no new precedence rules.
- **Constraints:**
  - `PlaceIndex` is allowed at most once at the tail of a `Place`. Multi-level indexing (`arr[i][j] = x;`) is rejected with `P-021` ("nested indexed assignment not supported in v0"). If multi-level indexing is needed, it lands as a separate extension with its own rationale.
  - The index `Expr` must produce an unsigned integer type.
  - The compiler enforces a static or symbolic upper bound on the index against the target's declared capacity. An index that cannot be statically proved within bounds is rejected with `K-018` ("array LHS index out of bounds at compile time"). Runtime bounds checking is *not* added; v0's principle is that bounds are proved at compile time.
- **Conformance:**
  - pass: `tests/conformance/boot/pass/array_index_lhs.phos`
  - fail: `tests/conformance/boot/fail/array_index_lhs_unbounded.phos` (index that cannot be statically bounded), `tests/conformance/boot/fail/array_index_lhs_nested.phos` (multi-level index)
- **Diagnostic codes reserved:** `K-018` (array LHS index out of bounds at compile time), `P-021` (nested indexed assignment).

### Extension 2026-04-27c — Block trailing-expression-as-return semantics

- **Form added to grammar:** None. The shape `Block ::= "{" Statement* Expr? "}"` is unchanged; the *semantics* of the trailing `Expr` is now declared explicitly in [grammar.md](grammar.md) under "Block Semantics."
- **Implementation site that forced it:** [compiler/lexer.phos](../../compiler/lexer.phos) lines 561 and 568 (`scan_ident`, `scan_integer`) terminate functions by producing a value as the trailing block expression, with no explicit `return`. This pattern is heavily used across the lexer and parser and is the natural form for one-expression-bodied helpers.
- **Why this is a clarification, not an extension:** The grammar already lists `Expr?` in `Block`. What was missing was the *meaning* — does the trailing expression evaluate to the block's value, become the function's return, or get discarded? The grammar didn't say. The clarification states: the trailing expression is the block's value, and when the block is a function body, the function's return value.
- **Constraints:**
  - The trailing expression's type must match the function's declared `ReturnType` (or be `()` if the function has no `ReturnType`).
  - A function whose body is a `Block` with neither a trailing `Expr` nor any `return` statement implicitly returns `()`. If the function declared a non-unit return type, this is `K-019` ("function body produces no value matching declared return type"). Reserved.
  - A `return Expr;` statement *and* a trailing `Expr` in the same block is a redundant-return error (`K-020`, reserved).
- **Conformance:**
  - pass: `tests/conformance/boot/pass/trailing_block_expr.phos`
  - fail: `tests/conformance/boot/fail/trailing_block_expr_type_mismatch.phos`
- **Diagnostic codes reserved:** `K-019` (missing return value), `K-020` (redundant trailing-expr-and-return).

## Audit running list

After every implementation milestone, every active `.phos` file is re-validated against the current grammar. Any new construct the implementation requires that is not in the grammar is added to the table below — first as a row, then resolved by either amending the grammar (a new entry above) or refactoring the implementation to comply.

| Surfaced on | Construct | File:Line | Resolution |
|---|---|---|---|
| 2026-04-27 | unit expression `()` in match arms | lexer.phos:645–736 | Extension 2026-04-27a (above) |
| 2026-04-27 | array index on LHS of assignment | lexer.phos:761, 827 | Extension 2026-04-27b (above) |
| 2026-04-27 | trailing-expression-as-return semantics undefined | lexer.phos:561, 568 + many sites | Extension 2026-04-27c (above) |

## Non-goals

The following are explicitly *not* extensions on the v0 surface and will be rejected:

- multi-level indexed assignment (`arr[i][j] = x;`) — see P-021 above
- pattern guards in `match`
- `let` patterns (destructuring)
- `if let` / `while let`
- early-break in `for` (no `break` keyword)
- `loop` keyword (use bounded `for`)
- `&` / `&mut` borrow operators
- closures or function-as-value
- generic parameters
- string literals (only byte-array constants via integer literals)
- floating-point types or literals
- async / await
- macros / proc-macros / `paste!`
- inheritance / `impl` blocks / traits

These are documented in [LANGUAGE_NON_GOALS.md](../../LANGUAGE_NON_GOALS.md) and [docs/non_goals.md](../non_goals.md).
