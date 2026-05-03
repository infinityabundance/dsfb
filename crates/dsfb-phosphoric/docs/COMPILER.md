# pcc — Phosphoric Compiler Reference

`pcc` is the Phosphoric compiler. It is itself written in Phosphoric (under the host profile) and compiles Phosphoric source under any of the four profiles (boot, host, trusted, runtime) into the target artifact for that profile.

This document describes the compiler's pipeline, each pass, and the contracts each pass produces and consumes.

---

## 1. Pipeline Overview

```
.phos source
    ↓
Lexer            (compiler/lexer.phos)        — tokenise UTF-8 ASCII into a token stream
    ↓
Parser           (compiler/parser.phos)       — build AST (items + arena-interned exprs/stmts/blocks)
    ↓
AST → HIR        (compiler/hir.phos)          — lower AstExpr/AstStmt/AstBlock into HirArenas
    ↓
Type checker     (compiler/typeck.phos)       — assign types, track capabilities, enforce K-### codes
    ↓
HIR well-formed  (compiler/hir_wf.phos)       — name uniqueness, structural invariants
    ↓
Effects          (compiler/effects.phos)      — transitive closure, profile alphabet check
    ↓
Layout           (compiler/layout.phos)       — ABI-backed type sizing, field offsets
    ↓
Call graph       (compiler/call_graph.phos)   — Tarjan SCC; reject cycles (no recursion in application code)
    ↓
Stack analysis   (compiler/stack_analysis.phos) — worst-case per-entrypoint frame, against budget
    ↓
Codegen          (compiler/codegen_*.phos)    — boot/host/trusted/runtime per profile
    ↓
Output artifact (boot_ir_v1 + boot_asm_v1, ELF, Ember-shape blob, runtime image)
```

Each pass is a separate `.phos` module. Each module is a flat call graph (no recursion); when traversal of a tree-shaped data structure is required, the pass uses an explicit work stack with a hard depth bound.

---

## 2. Driver

[compiler/pcc.phos](../compiler/pcc.phos) is the CLI driver. Expected invocation:

```
pcc <source> <output> [--target=<name>|--assure] [--budget=<path>]
```

| Flag | Meaning |
|---|---|
| `--target=boot` | Emit `boot_ir_v1` + `boot_asm_v1` for the UEFI boot profile. |
| `--target=host` | Emit a Linux x86_64 ELF for the host profile. |
| `--target=trusted` | Emit an Ember-shape blob for the trusted profile. |
| `--target=runtime` | Emit a runtime kernel object for the runtime profile. |
| `--assure` | Run analyses, emit assurance report; produce no output artifact. |
| `--budget=<path>` | Optional stack budget TOML path. |

Exit codes:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Bad CLI args |
| 2 | Source IO failure |
| 3 | Compile failure (a diagnostic is written to stderr) |
| 4 | Output write failure |

---

## 3. Lexer

[compiler/lexer.phos](../compiler/lexer.phos) (~850 LOC, real implementation, zero panics).

### 3.1 Token classes

```phos
enum TokenKind {
    Ident,
    Integer,
    KeywordTok(Keyword),
    PunctTok(Punct),
    OpTok(Operator),
    Eof,
}
```

### 3.2 Length-bucketed keyword recognition

The lexer dispatches on lexeme length — `kw_len2`, `kw_len3`, ..., `kw_len10` — each branch does a direct byte-by-byte comparison against the keyword's expected bytes. No hash table, no trie. Constant-time per keyword.

### 3.3 Diagnostic codes

| Code | Reason |
|---|---|
| L-001 | Unexpected byte |
| L-006 | String literal rejected |
| L-007 | Float literal rejected |
| L-014 | Token buffer overflow |

### 3.4 What the lexer rejects

- String literals — programs use byte-array literals instead.
- Float literals — no floating-point in v0.
- Non-ASCII bytes outside identifiers — UTF-8 not yet supported.

---

## 4. Parser

[compiler/parser.phos](../compiler/parser.phos) (~2,200 LOC).

### 4.1 Two-pass design

The v0 grammar has cycles: statements contain expressions which contain blocks which contain statements. v0 forbids unbounded recursion. The parser breaks the cycle by separating boundary recording from tree construction:

- **Boundary pass** — walks the token stream with paren/bracket/brace balancing. Records each statement's token span, classifies by leading token (let/return/for/expr-or-assign).
- **Tree-construction pass** — builds the actual `AstExpr` / `AstStmt` / `AstBlock` arenas. This pass uses iterative algorithms with explicit work stacks for naturally-recursive forms (type expressions, binary operator precedence climbing, match arm patterns).

### 4.2 Pratt expression parser

Binary operators are handled by an iterative Pratt parser with precedence climbing:

```phos
struct PrattState {
    value_stack: [u16; 32],
    value_top: u32,
    op_tag_stack: [u8; 32],
    prec_stack: [u8; 32],
    op_top: u32,
    arenas: pcc.ast.AstArenas,
}
```

Three precedence levels (4: comparisons, 5: additive, 6: multiplicative). Left-associative. Bounded loop with done-flag idiom (no `while` / `break` in v0).

### 4.3 Diagnostic codes

| Code | Reason |
|---|---|
| P-001 | Unexpected token |
| P-002 | Expected `module` keyword at file start |
| P-003 | Expected dotted module path |
| P-004 | Expected `;` after module declaration |
| P-005 | Expected struct/enum/fn/capability keyword |
| P-006 | Malformed type expression |
| P-007 | Malformed effect declaration |
| P-008 | Unexpected EOF mid-production |
| P-009 .. P-019 | v0-forbidden constructs (each kind has a dedicated code) |
| P-020 | Invalid `profile` clause |
| P-021 | Nested indexed assignment |

### 4.4 v2 entry

`parse_module_v2(tokens, token_count, arenas)` is the canonical top-level entry. It returns a `ParseModuleStep { ctx, arenas }` carrying the populated `AstArenas`. Every function decl threads arenas through `parse_block_v2` → `parse_statement_v2` → per-statement v2 parsers, so the resulting AstModule has real arena entries (not just stub spans).

---

## 5. AST → HIR

[compiler/hir.phos](../compiler/hir.phos) lowers AST node forms to their HIR equivalents.

### 5.1 HirArenas

Mirrors `AstArenas` in shape:

```phos
struct HirArenas {
    exprs: [HirExpr; 4096],
    expr_count: u16,
    stmts: [HirStmt; 2048],
    stmt_count: u16,
    blocks: [HirBlock; 256],
    block_count: u16,
}
```

### 5.2 Lowering

`lower_arenas(ast: AstArenas) -> HirArenas` walks every interned AST node and produces its HIR counterpart. The lowering is per-variant dispatch:

| AST | HIR |
|---|---|
| `AstExpr::IntegerLit(v)` | `HirExpr::IntLit(v)` |
| `AstExpr::BoolLit(b)` | `HirExpr::BoolLit(b)` |
| `AstExpr::UnitLit` | `HirExpr::Unit` |
| `AstExpr::PathExpr(...)` | `HirExpr::Path(0)` (path-id resolved by typeck) |
| `AstExpr::BinaryOp(op, lhs, rhs)` | `HirExpr::Binary(op, lhs, rhs)` |
| `AstStmt::LetStmt(name, ty, rhs)` | `HirStmt::Let(name, ty, rhs)` |
| `AstStmt::ForStmt(var, bound, start, end, body)` | `HirStmt::For(var, bound, start, end, body)` |

The lowering preserves arena ids — AstExpr id N corresponds to HirExpr id N.

---

## 6. Type Checker

[compiler/typeck.phos](../compiler/typeck.phos) (~830 LOC).

### 6.1 Five passes

| Pass | Purpose |
|---|---|
| 1 | Populate type table (struct/enum/capability declarations → entries). |
| 2 | Resolve type ids — every type_id must point at a known entry. |
| 3 | Per-function signature checks (parameter types known, return type known). |
| 4 | Struct/enum field-type resolution. |
| 5 | Per-statement and per-expression type constraints. |

### 6.2 Top-level v2 entry

`check_module_v2(ast_module: AstModule) -> Result[u32, Diagnostic]`:

1. Lowers `ast_module.arenas` → HirArenas via `pcc.hir.lower_arenas`.
2. Walks every interned `HirStmt` through `check_stmt_with_arenas`.
3. Walks every interned `HirExpr` through `check_binary_operand_types` (K-014).
4. Walks every block to confirm `stmt_ids` are in range.

First failure short-circuits.

### 6.3 Diagnostic codes (K-prefix)

Reachable from check_module_v2 today: K-001 (type mismatch), K-002 (arity), K-003 (field access on non-struct), K-004 (variant access on non-enum), K-005 (match arm pattern type mismatch), K-006 (non-exhaustive match), K-007 (move-after-move), K-008 (use-after-move), K-009 (capability duplication), K-010 (unknown name), K-011 (unsupported type form), K-012 (return type mismatch), K-013 (let-binding type mismatch), K-014 (binary operator operand types incompatible), K-016 (match arms divergent), K-019 (function body produces no value).

Reserved for later integration: K-015 (if-condition not bool — possibly unreachable in v0 since match replaces if), K-017/K-018/K-020 (E16 grammar extension precision codes).

### Warning class: K-W-### (typeck warnings)

Some properties the language contract requires cannot be statically verified across cross-module call boundaries without a path-sensitive prover. Rejecting the program would be a false positive; accepting it without comment would hide a precision gap. The compiler emits a non-fatal **K-W-### warning** and continues compilation:

- **K-W-001** — capability handle (`WindowHandle`, `ChannelHandle`) used at a call site the typecker accepts but cannot fully prove path-sensitively. Wired with E19 (Kernel Capability Typing). The full path-sensitive prover lands as a follow-up; until then, K-W-001 surfaces the precision gap so reviewers can audit the call graph manually.

K-W-### diagnostics use the format `K-W-<NNN> <span> <message_id>`. They do not fail compilation; they appear on stderr and are recorded in the assurance report.

### 6.4 Capability tracking

Sequential, branch-merge-conservative:

```phos
fn record_move(ctxt: TypeCtxt, capability_idx: u8) -> Result[TypeCtxt, Diagnostic] {
    // K-007 on move-after-move
    // K-008 on use-after-move
}
```

Branches and match arms join via `merge_capability_states`: a capability is `Moved` in the merge if it is moved in any branch. A future precision pass lifts this into per-arm tracking.

---

## 7. HIR Well-Formedness

[compiler/hir_wf.phos](../compiler/hir_wf.phos) checks structural invariants HIR must satisfy beyond what the parser verifies:

- Struct field names are unique (W-006).
- Enum variant names are unique (W-007).
- Capability names are unique (W-008).
- Function name resolution is consistent (W-001 .. W-005, integrated with typeck's symbol table).

---

## 8. Effects

[compiler/effects.phos](../compiler/effects.phos) computes the transitive effect closure over the call graph.

### 8.1 Effect alphabet

Per-profile alphabets are pinned in [docs/language/effect_lattice.toml](language/effect_lattice.toml):

| Profile | Alphabet |
|---|---|
| Boot | `draw`, `ipc`, `mmio`, `sched`, `time` |
| Host | `host-fs-read`, `host-fs-write`, `host-stdout`, `host-stderr`, `host-time-mono`, `host-hash` |
| Trusted | `cpu_op`, `mmio_*`, `port_*` |
| Runtime | boot alphabet + `cap-issue`, `cap-revoke` |

### 8.2 Closure axioms

Bitset OR over the per-profile alphabet:

- Idempotence: `join(A, A) == A`
- Commutativity: `join(A, B) == join(B, A)`
- Associativity: `join(join(A, B), C) == join(A, join(B, C))`
- Monotonicity: subsets are preserved through join.
- Alphabet closure: no join produces an out-of-alphabet bit.

These are checked by the host program `phosphoric_effect_check.phos`.

### 8.3 Diagnostic codes (E-prefix)

| Code | Reason |
|---|---|
| E-001 | Function calls a function with effects not declared at the call site |
| E-002 | Effect declared but unused |
| E-003 | Effect not in the profile's alphabet |
| E-006 | Cross-profile effect mix |

---

## 9. Layout

[compiler/layout.phos](../compiler/layout.phos) computes ABI-backed type sizes and field offsets per the rules in [docs/abi.md](abi.md).

### 9.1 Per-arch ABI

| Arch | ABI overhead per call frame |
|---|---|
| x86_64 SysV | 8 bytes (return address) |
| x86_64 MS | 32 bytes (shadow space + return address) |
| ARMv8-M | 0 bytes (link register in r14) |
| RISC-V | 0 bytes (return in ra) |

### 9.2 Struct layout

Fields are laid out in declaration order with natural alignment. Padding is inserted to bring each field to its alignment boundary. The layout is deterministic — no field reordering optimization.

---

## 10. Call Graph

[compiler/call_graph.phos](../compiler/call_graph.phos) builds the same-module call graph and runs Tarjan's strongly-connected-components algorithm to detect cycles.

### 10.1 Iterative Tarjan

Standard recursive Tarjan uses DFS with discovery numbers, low-link values, and an "on-stack" set. The iterative form uses an explicit `TarjanFrame` stack:

```phos
struct TarjanFrame {
    node_id: u16,
    next_edge_idx: u32,
    low: u32,
    on_stack: bool,
}

struct TarjanState {
    disc: [u32; 4096],
    low:  [u32; 4096],
    on_stack_set: [bool; 4096],
    component_stack: [u16; 4096],
    work_stack: [TarjanFrame; 4096],
    // ...
}
```

Bounded depth 4096. Any SCC of size > 1 is a recursion cycle and rejected.

### 10.2 Diagnostic codes (G-prefix)

| Code | Reason |
|---|---|
| G-001 | Self-recursion |
| G-002 | Mutual recursion (cycle of size > 1) |
| G-003 | Cross-module call (v0 forbids cross-module function calls) |

---

## 11. Stack Analysis

[compiler/stack_analysis.phos](../compiler/stack_analysis.phos) computes the worst-case stack depth per entrypoint.

The algorithm is an iterative fixpoint over the (acyclic) call graph: each function's frame size is the maximum of its own frame plus the maximum frame size of any function it calls. The fixpoint converges because the graph is a DAG (cycles were rejected by call_graph).

If the optional `--budget=<path>` flag was supplied, the per-entrypoint worst case is compared against the budget; overrun is S-001.

---

## 12. Codegen

The compiler has four codegen modules, one per profile:

### 12.1 Boot profile

[compiler/codegen_boot.phos](../compiler/codegen_boot.phos):

```
HIR → BootIrV1 → BootAsmV1 (text) → PE/COFF EFI image
```

`BootIrV1` is a small instruction set pinned in [docs/BOOT_ABI_V1.md](BOOT_ABI_V1.md):

```phos
enum BootIrInstr {
    LoadConst(u32, u32),
    LoadLocal(u32, u32),
    StoreLocal(u32, u32),
    BinaryOp(u8, u32, u32, u32),
    Branch(u32),
    BranchIfZero(u32, u32),
    Call(u32),
    Return,
    LabelDef(u32),
    Halt,
}
```

`emit_boot_asm` produces an 8-byte header (`"BV"` magic + LE schema_version + LE function_count + reserved zeros) followed by per-instruction byte streams. Each instruction's byte length is fixed by [docs/BOOT_ABI_V1.md](BOOT_ABI_V1.md).

### 12.2 Host profile

[compiler/codegen_host.phos](../compiler/codegen_host.phos):

```
HIR → ELF64 + 6-syscall stubs
```

Each declared host effect lowers to a 7-byte syscall stub:

```
B8 imm32                         # mov eax, syscall_no
0F 05                            # syscall
```

Syscall numbers per the Linux ABI: sys_read=0, sys_write=1, sys_clock_gettime=228. The stubs are emitted at the start of `.text` in stable order (bit 0 first, bit 5 last).

### 12.3 Trusted profile

[compiler/codegen_trusted.phos](../compiler/codegen_trusted.phos):

```
HIR + trusted! blocks → Ember-shape blob
```

Each `trusted!` block lowers to direct CPU instruction encoding. The lookup table `cpu_op_byte_length(op_id)` returns the canonical length per op_id (CLI=1, STI=1, LGDT=6, LIDT=6, etc.).

### 12.4 Runtime profile

The runtime profile lowers to a kernel object using the boot-profile machinery plus the runtime-specific effect alphabet (cap-issue, cap-revoke).

---

## 13. PE/COFF Writer

[tools/phosphoric/write_boot_efi_from_ir.sh](../tools/phosphoric/write_boot_efi_from_ir.sh) consumes the boot-asm output and writes a `BOOTX64.EFI` PE32+ image directly. No external linker, no `objcopy`. The writer's audit doc is at `tools/phosphoric/PE_WRITER_TRUST_AUDIT.md`.

---

## 14. Diagnostics

Every diagnostic is a `(prefix, number)` pair plus an optional source span:

```phos
struct Diagnostic {
    code_prefix: u8,        // 'L', 'P', 'K', etc. as ASCII byte
    code_number: u16,
    span: Option[Span],
    message_id: u16,
}
```

The format is `<PREFIX>-<NNN> <start>..<end> <message_id>\n`, e.g. `L-001 12..15 0\n`.

Codes are pinned. The conformance corpus tests that each declared fail case produces the documented code on the documented span.

---

## 15. Bootstrap

The compiler is self-hosted. Compiling the compiler from source requires a stage 0 binary, produced once externally per [bootstrap/STAGE0.md](../bootstrap/STAGE0.md). After stage 0, the chain is self-perpetuating:

```
stage0 + pcc.phos → stage1
stage1 + pcc.phos → stage2
stage2 + pcc.phos → stage3
verify_fixpoint:  stage2 == stage3 byte-equal
```

The fixpoint test proves that, given a trusted stage 0, stage 1 onward is reproducible source-to-binary.

---

## 16. The Conformance Corpus

[tests/conformance/](../tests/conformance/) contains 74 test cases organized by profile (boot/host/trusted/runtime) and outcome (pass/fail). Each fail case has a `// fail-expected: <code>` annotation pinning the expected diagnostic code.

The corpus is the canonical definition of v0 — when the grammar prose disagrees with what the corpus accepts or rejects, the corpus wins.

---

## 17. Verification Pipeline

| Gate | What it checks |
|---|---|
| `check_repo_hygiene.sh` | No non-Phosphoric source files in the active tree; required `.gitignore` patterns present |
| `check_archive_inert.sh` | Active build/release paths invoke no external assembler/linker |
| `check_docs.sh` | Required public docs exist and are well-formed |
| `check_boot_phosphoric_only.sh` | Boot path links no foreign objects; manifest fields match |
| `check_direct_pe_negative_tests.sh` | PE writer rejects malformed inputs |
| `run_uefi_demo.sh` | QEMU UEFI demo runs and emits all five marker lines |

The `verify-legendary` aggregate target adds 14 host-profile gates: phosphoric_invariant_check, phosphoric_conform, phosphoric_test_runner, phosphoric_lower_eq, phosphoric_repro_diff, phosphoric_attest, phosphoric_effect_check, phosphoric_fuzz, phosphoric_bound_check, check_tcb_budget, check_retirement_dates, check_phosphoric_only, check_host_profile_separation, check_trusted_blocks. Each is a Phosphoric host program.

---

## See Also

- [compiler/](../compiler/) — compiler source
- [compiler/README.md](../compiler/README.md) — compiler workspace overview
- [docs/PHOSPHORIC.md](PHOSPHORIC.md) — language reference
- [docs/EMBER.md](EMBER.md) — trusted nucleus
- [docs/PHOSPHOROS.md](PHOSPHOROS.md) — OS layer
- [docs/abi.md](abi.md) — ABI specification
- [docs/BOOT_ABI_V1.md](BOOT_ABI_V1.md) — boot IR specification
- [docs/ir.md](ir.md) — IR specification
- [docs/language/V0_FREEZE.md](language/V0_FREEZE.md) — v0 surface freeze
