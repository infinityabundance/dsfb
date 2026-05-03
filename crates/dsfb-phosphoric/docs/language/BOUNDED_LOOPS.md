# Bounded Loops

Every loop in active Phosphoric source carries an explicit upper bound. The bound is enforced at compile time. A loop without a declared bound is rejected. This is the same rule across boot, host, trusted, and runtime profiles.

The bound discipline is the project's answer to NASA P10 rule 2 ("all loops must have a fixed upper bound"), expressed as a *language feature* rather than a *post-hoc lint*.

## Annotation Form

Phosphoric extends v0 grammar with a `#[bound = N]` annotation that prefixes any `for` loop:

```ebnf
ForStmt ::= BoundAttr? "for" Ident "in" RangeExpr Block
BoundAttr ::= "#[" "bound" "=" IntegerLiteral "]"
```

Example:

```phos
#[bound = 256]
for i in 0..argc {
    // ...
}
```

The annotation is required. A `for` statement without `#[bound = N]` is a compile error (`B-001`).

## Bound Semantics

The bound `N` is an upper bound on the number of iterations. The compiler verifies that the actual `RangeExpr` cannot exceed `N` iterations:

- If both endpoints are constant, the compiler computes `(end − start)` and asserts it is ≤ `N`.
- If either endpoint is a variable, the compiler requires that the variable's declared maximum (from a struct field's type, an array length, a profile-declared capacity, etc.) makes `(end − start) ≤ N` provable.
- If the bound cannot be statically proved, the loop is rejected (`B-002`).

The bound is intentionally an *upper* bound. A loop that runs zero times is fine. A loop that always runs exactly `N` times is fine. A loop that *might* run more than `N` times is rejected.

## Symbolic Bounds

For loops over caller-supplied capacities, `N` may reference a `const` or a profile-declared capacity from a TOML manifest:

```phos
// host-profile capacity ceiling, declared in host_profile.toml
const ARGV_COUNT_MAX: u32 = 256;

#[bound = ARGV_COUNT_MAX]
for i in 0..argc {
    // ...
}
```

The compiler resolves the symbolic bound at compile time. v0 has no `const` declaration today; the bound checker therefore reads its symbol table from the relevant `*_profile.toml` capacity blocks. Adding language-level `const` is a future grammar extension.

## Bounded `match` Recursion (Implicit)

`match` arms do not iterate, so they need no bound. However, recursive function calls are forbidden across the language (per `V0_FREEZE.md`), so the only iterations in a Phosphoric program are `for` loops — and every one is bounded.

## Relationship To Stack Analysis

The bound on a `for` loop multiplies into the worst-case frame analysis run by [compiler/stack_analysis.phos](../../compiler/stack_analysis.phos) (TBD). A loop body that reserves stack space inside the loop has its frame counted at `N × inner_frame_size`. The stack budget rejects programs whose total exceeds the entrypoint stack budget.

## Diagnostic Codes

Loop-bound checker codes use the `B-` prefix:

- `B-001` — `for` loop missing required `#[bound = N]` annotation
- `B-002` — declared bound `N` cannot be proved as an upper bound for the loop range
- `B-003` — bound expression contains a non-constant, non-symbol-table reference
- `B-004` — bound is zero or negative
- `B-005` — symbolic bound references an unknown capacity manifest entry
- `B-006` — `for` annotation has malformed shape

## Implementation Pointers

- **Parser:** [compiler/parser.phos](../../compiler/parser.phos) (TBD) parses `#[bound = N]` as part of `ForStmt`.
- **Resolver:** [compiler/bound_resolver.phos](../../compiler/bound_resolver.phos) (TBD) resolves symbolic bounds from `host_profile.toml`, `runtime_profile.toml`, etc.
- **Checker:** [compiler/bound_check.phos](../../compiler/bound_check.phos) (TBD) verifies the static-bound property.
- **Verifier:** [tools/phosphoric-host/phosphoric-bound-check.phos](../../tools/phosphoric-host/phosphoric-bound-check.phos) (TBD; elevation item E14) runs the checker over every active `.phos` file and fails CI on any violation.

## Migration

Existing Phosphoric source under `apps/demo/`, `ember/`, `kernel/`, and `compiler/` does not yet carry `#[bound = N]` annotations because the syntax is being added with elevation item E14. The migration plan: every loop that lands in an active `.phos` file must be annotated; the bound-check tool runs in CI from the moment it lands; a one-time PR adds annotations to existing loops with reviewer-pinned bounds.

## Non-Goals

- This is not a WCET (worst-case execution time) annotation. The bound is iteration count, not microseconds.
- This is not a termination proof. A loop that terminates in `<<` `N` iterations still counts the bound as the worst case.
- This is not a generalized refinement-type system. The bound is a single integer; no dependent types.
