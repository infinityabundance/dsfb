# Phosphoric — Language Reference

Phosphoric is a deliberately narrow systems language for ultra-narrow, ultra-safety usage. It is not a general-purpose language and does not aspire to be one. The surface, the semantics, the type system, and the four profiles that follow exist to support a single thesis: **a small, auditable, capability-oriented stack on $5-class microcontrollers, with hardware protection as belt-and-suspenders rather than the primary trust mechanism**.

This document is the canonical reference for the language from an application-author perspective. For the formal specification of v0 (frozen surface, grammar productions, diagnostic codes), see [docs/language/V0_FREEZE.md](language/V0_FREEZE.md) and the per-profile manifests under [docs/language/](language/).

---

## 1. Design Principles

Five constraints shape every line of Phosphoric source:

1. **`no_std`** — No standard library. No standard collections, no allocator, no string formatting, no I/O abstractions. Programs build everything they need from primitives.
2. **`no_alloc`** — No heap allocation. All collections are fixed-capacity arrays declared at compile time.
3. **`no_unsafe`** — No `unsafe` keyword, no raw pointer dereferences, no aliasing tricks. The only path to machine-dangerous operations is through the trusted profile's `trusted!` blocks, and every such block carries a razor-rationale annotation that names why the operation is irreducible.
4. **Capability-oriented authority** — Resources (tasks, channels, windows, framebuffers) are accessed through generation-tagged handles. There is no ambient access; revocation increments the generation and stale handles return typed errors.
5. **Bounded everything** — Every `for` loop carries a `#[bound = N]` annotation. Recursion is forbidden in application functions (the compiler tracks call graphs and rejects cycles). Every collection has a fixed compile-time capacity. Worst-case stack usage is computable from the call graph.

These constraints are not optional. They are how the language proves what it claims.

---

## 2. Lexical Structure

A Phosphoric source file is UTF-8 ASCII text. The lexer is whitespace-aware (statements end at `;`, blocks delimit by `{` `}`).

### Tokens

| Class | Examples |
|---|---|
| Keywords | `module`, `profile`, `capability`, `struct`, `enum`, `fn`, `effects`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `return`, `true`, `false`, `Some`, `None`, `Ok`, `Err`, `trusted` |
| Punctuation | `( ) { } [ ]` `: , ; .` `..` `->` `=>` `=` `#` `!` |
| Operators | `+ - * /` `== != < <= > >=` |
| Identifiers | `[A-Za-z_][A-Za-z0-9_]*` |
| Integer literals | base-10 only; `42`, `0`, `1024`. No hex, no octal, no underscores. |
| Boolean literals | `true`, `false` |

### Forbidden Tokens (v0 rejects)

| Form | Diagnostic | Reason |
|---|---|---|
| String literal `"hello"` | L-006 | No string allocator. Byte sequences are written as integer arrays. |
| Float literal `3.14` | L-007 | No floating-point in v0. |
| `unsafe` keyword | P-009 | The trusted profile's `trusted!` blocks are the only path to machine-dangerous operations. |
| `async` keyword | P-009 | No async runtime. |
| `trait`, `impl` keywords | P-009 | No traits, no inheritance. |
| `macro` keyword | P-009 | No metaprogramming. |
| Generic type parameter `<T>` | P-011 | No generics. The four parametric type forms (`[T; N]`, `Slice[T, N]`, `Result[T, E]`, `Option[T]`) are hardcoded. |
| Closure `\|x\| x + 1` | P-012 | No first-class functions. |
| `while`, `loop` | P-013 | All iteration is bounded `for` loops. |
| Pattern guard `match x { y if ... => ... }` | P-016 | Match arms are flat. |
| Method receiver `fn foo(self, ...)` | P-017 | No method dispatch. Functions take their data as parameters. |
| Lifetime annotation `&'a` | P-018 | No references in v0. |
| `import`, `extern`, `use` | P-019 | No module imports. Cross-module references use fully-qualified paths. |

The lexer rejects each forbidden form with its diagnostic code. The diagnostic codes are part of the language's public surface — UI corpus tests pin them.

---

## 3. Module Structure

Every `.phos` file is one module. The first non-comment lines are:

```phos
module pcc.foo.bar;
profile host;
```

- `module pcc.foo.bar;` — fully-qualified module path. The path becomes a name prefix for everything the module declares.
- `profile <name>;` — declares which of the four profiles this module compiles under. One of `boot`, `host`, `trusted`, `runtime`. Default is `boot`.

### Profiles

| Profile | Purpose | Effects allowed | Lives in |
|---|---|---|---|
| `boot` | UEFI boot path; the demo's bootable image. | `draw`, `ipc`, `sched`, `time`, `mmio` | [apps/demo/](../apps/demo/) |
| `host` | Verifier, attester, conformance runner programs that run on the developer machine. | `host-fs-read`, `host-fs-write`, `host-stdout`, `host-stderr`, `host-time-mono`, `host-hash` | [tools/phosphoric-host/](../tools/phosphoric-host/) |
| `trusted` | The Ember nucleus. Only profile that can emit `trusted!` blocks. | `cpu_op`, `mmio_*`, `port_*` | [ember/](../ember/) |
| `runtime` | The PhosphorOS kernel. Builds on the four boot profile effects + `cap-issue`, `cap-revoke`. | runtime profile alphabet | [kernel/](../kernel/) |

A program cannot mix profiles. The compiler rejects cross-profile imports (diagnostic E-006). Each profile has its own TOML manifest pinning the effect alphabet, capacity ceilings, and reserved diagnostic codes.

---

## 4. Items

A module declares zero or more items. Each item is one of:

### 4.1 Capability declaration

```phos
capability gpio;
```

A capability is a name. Programs that use the capability declare it as a parameter; the compiler tracks affine ownership.

### 4.2 Struct declaration

```phos
struct Point {
    x: u32,
    y: u32,
}

struct Buffer {
    bytes: [u8; 256],
    length: u16,
}
```

Structs have named fields. Each field has a fixed type. Capacity 64 fields per struct.

### 4.3 Enum declaration

```phos
enum Direction {
    North,
    South,
    East,
    West,
}

enum InputEvent {
    Key(KeyEvent),
    MouseClick(MouseClickEvent),
    MouseMove(MouseMoveEvent),
}
```

Enums have named variants. Each variant may carry a payload (a tuple of types, capacity 8). Capacity 64 variants per enum.

Pattern matching against an enum is exhaustive by language rule — the compiler rejects non-exhaustive matches (diagnostic K-006).

### 4.4 Function declaration

```phos
fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn write_pixel(
    base: u64,
    stride: u32,
    x: u32,
    y: u32,
    pixel: u32,
) effects(mmio) {
    // body
}
```

A function declares parameters (capacity 32), an optional return type, an optional `effects(...)` clause naming the side effects it performs, and a body block.

The trailing expression of the body block is the return value. An explicit `return` is permitted but optional.

---

## 5. Types

Phosphoric has six type classes:

### 5.1 Primitive types

| Type | Width | Range |
|---|---|---|
| `u8` | 8 bits | 0..256 |
| `u16` | 16 bits | 0..65536 |
| `u32` | 32 bits | 0..2³² |
| `u64` | 64 bits | 0..2⁶⁴ |
| `i8` | 8 bits | -128..128 |
| `i16` | 16 bits | -32768..32768 |
| `i32` | 32 bits | -2³¹..2³¹ |
| `i64` | 64 bits | -2⁶³..2⁶³ |
| `bool` | 1 bit | `true`, `false` |

No floating-point. No `usize` / `isize` (sizes are arch-specific; v0 chooses `u32` or `u64` explicitly per call site).

### 5.2 Array types

```phos
let buffer: [u8; 256] = [0; 256];
let triangle: [Point; 3] = [Point { x: 0, y: 0 }; 3];
```

Fixed-capacity. The size N is part of the type. `[T; N]` and `[T; M]` are different types when N != M.

### 5.3 Slice types

```phos
fn write(out: Slice[u8, 4096], cursor: u32, b: u8) -> u32 {
    out[cursor] = b;
    cursor + 1
}
```

`Slice[T, N]` is a fixed-capacity reference to an `[T; M]` where M ≤ N. The slice carries its capacity in the type; bounds-checking happens at compile time when possible, at runtime when the index is dynamic.

### 5.4 Result and Option

```phos
fn divide(a: u32, b: u32) -> Result[u32, DivError] {
    match b == 0 {
        true  => Err(DivError::DivideByZero),
        false => Ok(a / b),
    }
}

fn first_set_bit(mask: u32) -> Option[u8] {
    match mask == 0 {
        true  => None,
        false => Some(0),
    }
}
```

`Result[T, E]` and `Option[T]` are the only error/optional types in v0. They are pattern-matched exhaustively.

### 5.4a Type acyclicity (structural property)

Phosphoric has no pointer types. Every struct field and enum payload is a by-value type reference. As a consequence, no type may transitively reference itself — the type-reference graph must be a directed acyclic graph (DAG).

Acyclicity is **enforced** by [compiler/hir_wf.phos](../compiler/hir_wf.phos)'s `check_type_acyclicity` pass. The pass uses an iterative DFS (no recursion in v0) with an explicit work stack; on detecting a back-edge to a type already on the path, it emits `W-010` (type acyclicity violated).

This is the property that keeps stack analysis (S-prefix) sound: the worst-case stack budget computation depends on every type's size being computable in finite time, which requires the type-reference graph to be a DAG. With the acyclicity pass, the size calculation always terminates.

### 5.5 Application-declared structs and enums

Same name resolution as primitives but referenced by their fully-qualified module path:

```phos
let p: pcc.geometry.Point = pcc.geometry.Point { x: 0, y: 0 };
```

### 5.6 Capability types

A capability declared in a module becomes a type at use sites:

```phos
fn read_pin(gpio: pcc.hardware.gpio, pin: u8) -> bool {
    // body
}
```

Capabilities are affine: the compiler tracks each move and rejects use-after-move (K-008).

---

## 6. Expressions

### 6.1 Literals

```phos
let n: u32 = 42;
let yes: bool = true;
let nothing: () = ();
```

### 6.2 Path expressions

```phos
let p = pcc.geometry.Point { x: 0, y: 0 };  // path resolves to a type
let v = state.value;                         // path resolves through field access
```

### 6.3 Binary operators

| Precedence | Operators |
|---|---|
| 4 | `==` `!=` `<` `<=` `>` `>=` |
| 5 | `+` `-` |
| 6 | `*` `/` |

All binary operators are left-associative. v0 has no parenthesized expression grouping; operator precedence is the only way to control evaluation order.

### 6.4 Field access

```phos
let x = point.x;
let bytes = buffer.bytes;
```

### 6.5 Indexing

```phos
let first = buffer[0];
buffer[count] = next;
```

Array and slice indexing. Out-of-range accesses are diagnosed at compile time when the index is constant; runtime-detected when the index is dynamic.

### 6.6 Match expressions

```phos
let label: u32 = match direction {
    Direction::North => 1,
    Direction::South => 2,
    Direction::East  => 3,
    Direction::West  => 4,
};
```

Match arms are pattern → expression. The arm bodies must produce the same type (K-016 on divergence). The set of arms must cover the matched type's domain; non-exhaustive matches are K-006.

### 6.7 Block expressions

```phos
let total = {
    let a = compute_a();
    let b = compute_b();
    a + b           // trailing expression is the block's value
};
```

A block is a sequence of statements terminated by an optional trailing expression. The trailing expression's type is the block's type. With no trailing expression, the block's type is `()` (unit).

### 6.8 Unit literal

```phos
let nothing: () = ();
```

`()` is the unit value. Used as the value of statements that produce no meaningful result.

---

## 7. Statements

### 7.1 Let binding

```phos
let n: u32 = 42;
let mut count: u32 = 0;
```

`let mut` introduces a mutable binding. The compiler enforces single-assignment for non-mut bindings and rejects use-of-uninitialized.

### 7.2 Assignment

```phos
count = count + 1;
buffer[i] = next;
```

Assignment requires the place expression be mutable (`mut` binding or array element of a mut binding). Non-mut assignments are diagnosed.

### 7.3 Expression statement

```phos
compute_side_effect();
```

An expression followed by `;` whose value is discarded.

### 7.4 For loop

```phos
#[bound = 256]
for i in 0..count {
    process(i);
}
```

Every `for` loop carries a `#[bound = N]` annotation. The bound `N` is an upper bound on iteration count; the compiler verifies `(end - start) <= N` either statically (when both endpoints are constant) or symbolically (when an endpoint resolves to a profile-declared capacity). A `for` without `#[bound = N]` is rejected with diagnostic B-001.

### 7.5 Return statement

```phos
fn maybe_value(n: u32) -> Option[u32] {
    match n == 0 {
        true  => return None,
        false => (),
    };
    Some(n)
}
```

`return` exits the function early. Bare `return ;` is allowed when the return type is `()`; otherwise an expression is required.

### 7.6 Block statement

A standalone `{ ... }` block whose value is unit.

---

## 8. Effects

Functions declare which side effects they may perform via the optional `effects(...)` clause:

```phos
fn write_serial(byte: u8) effects(mmio) {
    // body
}

fn validate_handle(h: TaskHandle) -> Result[u32, KernelError] effects(cap-issue) {
    // body
}
```

The set of allowed effect names depends on the module's profile. The boot profile alphabet is `{draw, ipc, mmio, sched, time}`. The host profile alphabet is `{host-fs-read, host-fs-write, host-stdout, host-stderr, host-time-mono, host-hash}`. The trusted and runtime profiles have their own alphabets.

The compiler computes the **transitive effect closure**: a function that calls another inherits the callee's declared effects. Calling a function with an effect not declared at the caller's site is rejected (E-001). The closure axioms (idempotence, commutativity, associativity, monotonicity) are checked by the host program `phosphoric_effect_check.phos`.

A function with no `effects(...)` clause has the empty effect set — it may not call any function with declared effects.

---

## 9. Capabilities

A capability is an affine resource. The compiler tracks every use:

```phos
capability gpio;

fn configure_pin(g: gpio, pin: u8, mode: u8) -> gpio {
    // ... uses g ...
    g                        // returns the capability for the caller to continue using
}

fn use_capability_twice(g: gpio) {
    let _g1 = consume(g);    // moves g
    let _g2 = consume(g);    // K-008: use-after-move
}
```

Capabilities cannot be duplicated (K-009). They cannot be used after a move (K-008). They cannot be moved twice on the same path (K-007). Branches and match arms join conservatively: a capability moved on any branch is treated as moved on the merge.

A future precision pass (path-sensitive capability tracking) lifts the conservative join into a per-arm tracker; the conservative form is the v0 baseline.

---

## 10. Diagnostics

Phosphoric emits stable diagnostic codes from a per-pass prefix family:

| Prefix | Pass | Range |
|---|---|---|
| `L-` | Lexer | L-001 .. L-014 |
| `P-` | Parser | P-001 .. P-021 |
| `H-` | Host profile | H-001 .. H-008 |
| `T-` | Trusted profile | T-001 .. T-008 |
| `R-` | Runtime profile | R-001 .. R-008 |
| `E-` | Effects | E-001 .. E-007 |
| `B-` | Bounded loops | B-001 .. B-006 |
| `C-` | Capability prover | C-001 .. C-006 |
| `K-` | Type checker | K-001 .. K-020 |
| `S-` | Stack analysis | S-001 .. S-006 |
| `X-` | Layout | X-001 .. X-006 |
| `W-` | HIR well-formedness | W-001 .. W-010 |
| `Z-` | Razor discipline | Z-001 .. Z-002 |

Codes are pinned. The conformance corpus (74 cases across boot/host/trusted/runtime profiles) tests that each declared fail case produces the documented code on the documented span. A new diagnostic is a coordinated change: corpus entry, code reservation in the relevant pass, message catalog entry.

---

## 11. Worked Example

A complete boot-profile program:

```phos
module pcc.demo.counter;
profile boot;

struct Counter {
    value: u32,
    max: u32,
}

enum CounterStep {
    Continue(Counter),
    Done,
}

fn empty_counter(max: u32) -> Counter {
    Counter { value: 0, max: max }
}

fn step(c: Counter) -> CounterStep {
    match c.value >= c.max {
        true  => CounterStep::Done,
        false => CounterStep::Continue(Counter {
            value: c.value + 1,
            max: c.max,
        }),
    }
}

fn run_to_completion(initial_max: u32) -> u32 {
    let mut current: Counter = empty_counter(initial_max);
    #[bound = 65536]
    for _ in 0..initial_max {
        match step(current) {
            CounterStep::Done => return current.value,
            CounterStep::Continue(next) => current = next,
        };
    }
    current.value
}
```

Compiled by the project compiler (`pcc`) under the boot profile, this lowers to `boot_ir_v1` and then to `boot_asm_v1` — a narrow x86_64 assembly subset that the project's PE/COFF writer turns into a `BOOTX64.EFI` image directly, with no external assembler or linker.

---

## 12. What This Language Is Not

Phosphoric is deliberately not:

- A general-purpose systems language. The four profiles + frozen v0 grammar reject features that would expand the review surface without serving the narrow target.
- A replacement for any other systems language. Phosphoric occupies a specific niche: language-enforced affine capabilities + deterministic memory + ultra-narrow trusted surface, on $5-class chips.
- A language for cloud applications, web services, scientific computing, or anything that needs allocation, networking, or general I/O. Those are explicit non-goals.
- An attempt to compete with formal-methods systems. Phosphoric leans on language enforcement plus per-block razor-rationale annotations; full machine-checked correctness proofs are out of scope.

The narrowness is not a limitation. It is the project's defensible position.

---

## See Also

- [docs/language/V0_FREEZE.md](language/V0_FREEZE.md) — formal v0 surface freeze
- [docs/language/grammar.md](language/grammar.md) — full BNF grammar
- [docs/language/effects.md](language/effects.md) — effect lattice formal definition
- [docs/language/memory_model.md](language/memory_model.md) — memory and aliasing rules
- [docs/language/type_system.md](language/type_system.md) — type system formal definition
- [docs/language/HOST_PROFILE.md](language/HOST_PROFILE.md), [docs/language/TRUSTED_PROFILE.md](language/TRUSTED_PROFILE.md), [docs/language/RUNTIME_PROFILE.md](language/RUNTIME_PROFILE.md) — per-profile manifests
- [docs/language/BOUNDED_LOOPS.md](language/BOUNDED_LOOPS.md) — bound-attribute discipline
- [docs/EMBER.md](EMBER.md) — the trusted nucleus
- [docs/PHOSPHOROS.md](PHOSPHOROS.md) — the OS layer
- [docs/COMPILER.md](COMPILER.md) — the compiler
