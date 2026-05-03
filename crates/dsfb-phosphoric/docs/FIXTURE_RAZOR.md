# Fixture Razor

## Apex statement

Phosphoric is a forensic emitter, not an industrial compiler.

The fixture corpus exists to prove the system **cannot lie about its own
behavior**. It does not exist to certify language maturity, ergonomic
quality, performance, or feature breadth.

## Active razor (2026-05-03 refocus)

Every fixture must be *evidentiary*. Not every fixture must be
*directly forensic*. A fixture belongs in the **core** corpus only if
it supports one or more of the following seven admission criteria:

1. **Minimal court compilation** — the fixture pins a source-shape →
   emit-bytes lowering that the court runtime actually compiles or
   executes (or that the bootstrap path producing the court binary
   actually compiles).
2. **Authority sealing** — the fixture proves manifest-edge declared
   authority is enforced (no silent authority transitions; declared
   capabilities match emitted boundaries).
3. **Typed residual emission** — the fixture asserts byte-stable
   layout of an R1..R7 residual record and the `chain_step`
   determinism for that record.
4. **PFI / replay / verdict determinism** — the fixture asserts
   byte-stable PFI0 case-file layout, byte-identical verdict bytes,
   and replay idempotency.
5. **Byte-stable bootstrap for the court** — the fixture protects
   the boot path (boot stage0_entry, canned trailer, M.3.B-style
   exit logic) from silent drift.
6. **Fail-closed rejection of real forbidden evidence** — the
   fixture asserts that a malformed PFI / corrupted residual /
   broken chain_hash is rejected with a named verdict, not
   silently accepted.
7. **Architecture / ABI requirement actually used by the edge
   court** — the fixture pins a struct-field load/store, hidden-
   pointer return, syscall instruction sequence, or other ABI
   detail that the court runtime depends on.

A fixture that does **not** answer at least one of those seven is
**not** a court-required fixture. It may still be a *bootstrap /
admissibility witness* (preventing silent drift in the admitted
compile surface) — but its forensic priority is lower, and its
admission must be justified.

> **Unused expressivity is not neutral on constrained edge chips.**

Every accepted token, emitted opcode, and recognized shape adds
parser paths, flash/RAM pressure, verification burden, and possible
drift states on a $5 industrial edge chip. Closing a fixture that
does not support the court's runtime, residual evidence, or
admissibility surface widens the admitted behavior surface that the
court must police.

The same razor applies to **gates**, not just fixtures. After the
A1/B1 + D1 + B1-narrow host-reference court loop landed
(2026-05-03), an overlap analysis at the proposed C1 (bounded
fixed-buffer scan) admission point found that the bounded-walk
invariants — count-driven iteration, fixed 32-byte record stride,
no file-length / sentinel / unbounded scanning — are **already
structurally enforced** by `verify-pfi-layout` (size pinned to
`192 + 32 * residual_count`; `for i in 0..residual_count` walk;
footer immediately abuts the records region). C1 was therefore
**declined** under the razor; a separate gate would duplicate
existing verifier work. The single-case R5 host-reference court
loop is now **closed and saturated**; further host-side gates over
the same single case are rejected unless they enforce a genuinely
new non-overlapping invariant. Further progress requires an explicit
trigger (promotion / breadth / replay / edge); see the saturation
section in [`FORENSIC_PRIMACY.md`](FORENSIC_PRIMACY.md).

## ASM and the razor

Per `GOAL.md` §"Bootstrap discipline", ASM is the honest trust
anchor. The razor governs the **admitted fixture corpus** — what
gets implemented and proven byte-equal against the ASM spec —
but does not exclude ASM from the audit floor.

- Source↔ASM byte-equal witnesses are the **proof of the compiler**.
  The 51 / 82 closures are landed evidence; the 31 remaining
  fixtures are scheduled named work in the campaign restart (per
  `GOAL.md` and `SELFHOST_BACKLOG.md`).
- A fixture is admitted to the corpus if it satisfies one or more
  of the seven admission criteria above AND directly supports
  either (a) the deterministic forensic court, (b) the v0.x QEMU
  proving ground, or (c) the v1.x $5-MCU port plan.
- **No fixture should be implemented merely because it exists.**
  "Because it would close another fixture is not sufficient
  justification." Court need, x86 proving-ground need, or
  edge-deployment need is required.
- Adding compiler surface (`phase0_stub.S` extensions) to close a
  fixture is in-scope under the campaign restart — analogous to
  Sessions B–S Exception A work. Each extension is named work,
  not blanket authorization.

The razor's purpose is to bound the admitted court surface on
constrained edge chips, while preserving the byte-equal evidence
discipline that justifies trust in the compiler at all.

The remaining 31 source↔ASM fixtures (after Sessions B–S, score
51 / 82) are tracked in
[`SELFHOST_BACKLOG.md`](SELFHOST_BACKLOG.md) — each annotated with
admission status and implementation trigger. They are scheduled
work, not preserved candidates.

Every fixture must answer four questions:

1. **What ambiguity does this eliminate?**
2. **What behavior class does this prove?**
3. **What future regression would this catch?**
4. **Is it required for one of the four doctrine goals?**

A fixture that cannot answer all four is rejected.

## The four doctrine goals

A fixture exists only if it eliminates one of these ambiguities:

### 1. Compiler closure
Can Phosphoric compile the Phosphoric subset required to preserve itself?
Each compiler-closure fixture pins a specific producer lowering (a single
shape of source → emitted instructions) so a regression in that lowering
is caught at byte level.

### 2. Task-seal closure
Can source exceed manifest authority? It MUST fail deterministically.
Each task-seal fixture is a *negative* — source that violates a manifest
boundary, expected to fail at compile or load time with a specific,
named error (no probabilistic gating, no warnings).

### 3. Residual truth
Does every authority boundary emit the correct typed residual?
Each residual fixture exercises one of R1..R7 (cap_graph_delta,
ipc_route_delta, budget_pressure, effect_trace, mmio_touch,
task_transition, boot_check) and verifies the emitted bytes match the
declared shape.

### 4. Forensic classification determinism
Does the same incident artifact always produce the same byte-identical
verdict? Each classification fixture takes a residual stream and a
manifest, runs `classify`, and checks the output bytes match a locked
expectation. Idempotent replay is the test.

## Admitted fixture classes

A fixture is admitted if and only if it falls into one of:

| Class | Doctrine goal | Count target |
|-------|---------------|--------------|
| Compiler / bootstrap | Compiler closure | 30–50 |
| Task-seal negative   | Task-seal closure | 10–20 |
| Residual / incident  | Residual truth | 10–20 |
| Fixpoint / quine     | Compiler closure (deepest) | 5–10 |

The count targets are upper bounds. The actual corpus shrinks when
fixtures become redundant under the razor (same behavior_class without
explicit justification).

## Rejected fixture classes

The following classes are NOT admitted, regardless of how natural they
might seem in an industrial test suite:

- **Mature-language coverage**: e.g. fixtures that exist only to test
  every keyword, every operator, every type. The corpus is not a
  language conformance suite.
- **Ergonomic regressions**: e.g. fixtures that test diagnostic message
  wording, syntax-error formatting, or developer-friendliness. These
  belong in a separate diagnostics test suite if at all.
- **Performance benchmarks**: timing, throughput, allocation counts.
  Phosphoric optimizes for deterministic truth, not speed.
- **Heuristic / probabilistic checks**: fixtures that pass "most of the
  time" or that tolerate minor output drift. The system either is
  deterministic or it isn't; there is no middle.
- **Coverage-driven duplicates**: two fixtures with the same
  `behavior_class` and no `justified_duplicate` annotation. The razor
  gate rejects these.

## Current inventory (compiler-closure tier)

79 of the 81 currently-locked .phos fixtures fall under the compiler-
bootstrap or fixpoint-quine class (closure goal #1). They prove that the
producer correctly lowers specific source shapes. The remaining 2 are
goal-#3 (residual-truth) — see the next section.

| Fixture | Behavior class | Ambiguity removed |
|---------|----------------|-------------------|
| exit42 | return_intlit_lowering | Pass H entry block exits with captured INTLIT |
| let_return | let_then_return | Pass K let-tracker captures INTLIT and feeds the return path |
| binop_fold | constant_fold_return | Pass L folds `INT op INT` at producer time |
| bounded_loop | bounded_for_then_return | Pass M.3.G-narrow emits a real bounded loop and returns |
| match_true | match_literal_bool | Pass M.3.H-mini selects arm based on literal scrutinee |
| sys1_exit42 | syscall_1arg_lowering | `__sys1(NR, ARG)` lowers to mov eax,NR; mov edi,ARG; syscall |
| sys3_write | syscall_4arg_lowering | `__sys3(NR, A, B, C)` lowers to 4-reg syscall sequence |
| load32_filesz | load_u32_from_vma | `__load32(ADDR)` lowers to mov edi, ADDR; mov eax, [rdi] |
| quine_self | byte_equal_self_emit | `__quine()` produces a binary that emits its own bytes |
| call_one_arg | one_arg_call_emit | `return helper(INTLIT)` emits mov edi, IMM; call rel32 |
| call_entry_one_arg | entry_call_runtime | stage0_call_entry passes IMM to helper, exits with helper's return |
| call_param_name | param_identity_from_signature | callee captures param name from fn signature, returns it |
| call_let_arg | let_bound_as_call_arg | `helper(LET)` substitutes let_value as IMM |
| param_add | param_arith_add | `return arg + INT` emits mov eax, edi; add eax, IMM |
| param_sub | param_arith_sub | `return arg - INT` emits mov eax, edi; sub eax, IMM |
| param_mul | param_arith_mul | `return arg * INT` emits mov eax, edi; imul eax, eax, IMM |
| param_add_let | param_arith_let_rhs | `return arg + LET` substitutes let_value as second operand |
| call_arg_fold_int | call_arg_int_op_int_fold | `helper(20 + 22)` folds expr at producer time |
| call_arg_fold_let | call_arg_let_op_int_fold | `helper(LET op INT)` folds expr at producer time |
| match_param_eq_true | match_runtime_cmp_eq_true | runtime cmp + je on PARAM == INT, TRUE arm fires |
| match_param_eq_false | match_runtime_cmp_eq_false | runtime cmp + je, FALSE arm fires |
| match_param_lt | match_runtime_cmp_lt | jl variant |
| match_param_gt | match_runtime_cmp_gt | jg variant |
| match_param_ne | match_runtime_cmp_ne | jne variant |
| match_imm32 | match_cmp_imm32 | IMM_32 cmp encoding (lifts IMM_8 constraint) |
| match_let_cmp | match_cmp_let_rhs | cmp value sourced from let_tracker |
| match_let_arm | match_arm_let | match arms accept let-bound IDENT |
| match_compose | match_compose_let_fold_arm | A-8 + A-17 composition (regression check) |
| let_fold_sub | let_rhs_fold_sub | `let n = INT - INT` folds at scan time |
| let_chain | let_chain_prev | `let b = a + INT` reads prior let_value |
| match_let_scrut_false | match_scrut_let_fold_false | scan-time fold of match w/ let scrutinee, FALSE arm |
| match_let_scrut_true | match_scrut_let_fold_true | scan-time fold, TRUE arm |
| nested_call_chain | three_fn_call_chain | entry → middle() → leaf(), three-deep call chain |
| match_false | match_literal_bool_false_arm | M.3.H-mini's false-arm path emits the rcx+9 IMM (distinct from match_true's rcx+5) |
| param_sub_let | param_arith_let_rhs_sub | A-10 op-tag dispatch emits `sub eax, IMM` (not add) for `arg - LET` |
| param_mul_let | param_arith_let_rhs_mul | A-10 op-tag dispatch emits `imul eax, eax, IMM` for `arg * LET` |
| let_plus_arg | pass_l_ident_rhs_arg_add_let_first | Pass L-IDENT-RHS reg-form `mov eax, IMM_let; add eax, edi` (let-cache operand_a) |
| int_plus_arg | pass_l_ident_rhs_arg_add_int_first | Pass L-IDENT-RHS with INT operand_a (distinct from let-cache entry) |
| let_times_arg | pass_l_ident_rhs_arg_mul | Pass L-IDENT-RHS op tag 24 emits `imul eax, edi` (3-byte op encoding) |
| let_minus_arg | pass_l_ident_rhs_arg_sub | Pass L-IDENT-RHS op tag 23 emits `sub eax, edi` (2-byte op encoding) |
| return_bool_true | m3q_return_bool_lit | Pass M.3.Q: `return true ;` via Pass H's TK_KEYWORD dispatch (true → IMM 1) |
| bare_bool_true | m3q_bare_bool_lit | Pass M.3.Q-bare-bool: `{ true }` at depth=1 with peek+1 = `}`, KW tag dispatch |
| bare_arg_return | m3q_bare_arg_passthrough | Pass M.3.Q-bare-ident: bare-IDENT matching param0 sets arg-flag, emits `mov eax, edi; ret` |
| bare_let_return | m3q_bare_let_name | Pass M.3.Q-bare-ident: bare-IDENT matching let_name captures let_value as IMM |
| bare_constructor_int | m3s_bare_constructor | Pass M.3.S: `IDENT(INT)}` constructor at depth=1; inner INT becomes return IMM |
| return_constructor_int | m3t_return_constructor | Pass M.3.T-safe: `return IDENT(INNER);` post-fn-table-lookup-fail fallback (distinguishes constructor from fn call) |
| bare_constructor_arg | m3s_constructor_with_ident_arg | Pass M.3.S ext: `IDENT(IDENT)}` where inner IDENT is param0 (arg-passthrough emit) |
| match_fn_call_both_arms | m3aab_match_fn_call_both_arms | Pass M.3.AA-β: match scrutinee fn1() AND FALSE-arm fn2(); two `call rel32` in 24-byte LEAF |
| match_fn_call_scrutinee | m3aa_match_fn_call_scrutinee | Pass M.3.AA-α: match scrutinee is fn call; `call rel32` is the first instruction (24-byte LEAF) |
| match_false_arm_comparison | m3x_match_false_arm_comparison | Pass M.3.X: mirror of M.3.V using xor-init + inc trick (eax=1 in 4 bytes vs 5-byte mov) |
| match_comparison_arm | m3v_match_comparison_arm | Pass M.3.V: 24-byte LEAF block (no prologue) starting with `cmp` not `push rbp` |
| match_trailing_comma | m3w_match_trailing_comma | Pass M.3.W: optional trailing comma after FALSE arm in match-cmp expressions |
| match_arm_arg | m3u_match_arm_param0 | Pass M.3.U: match arm dispatch accepts TK_IDENT-matching-param0 (arg-passthrough emit) |
| match_bool_arm | m3r_match_arm_kw_true_false | Pass M.3.R: match arm dispatch accepts TK_KEYWORD true/false (→ IMM 1/0) |
| bare_int_return | m3p_bare_int_return | Pass M.3.P: bare-INT return (no `return` keyword, no trailing `;`) at depth=1, peek+1 = `}` |
| bounded_loop_imm32 | m3g_mid_imm32_loop_emit | Pass M.3.G-mid: bounds > 127 use `mov ecx, IMM32` + `loop` instruction (distinct from M.3.G-narrow IMM8 shape) |
| let_plus_let2 | pass_l_ident_rhs_let_join_fold | Pass L-IDENT-RHS: let-name as operand_b joins INT-fold path (yields single mov eax, IMM emit) |
| param_le | match_runtime_cmp_le | Pass A-13 chooses jg opcode 0x7F for `<=` (distinct from `<`'s jge) |
| param_ge | match_runtime_cmp_ge | Pass A-13 chooses jl opcode 0x7C for `>=` (distinct from `>`'s jle) |
| match_let_cmp_imm32 | match_cmp_let_rhs_imm32 | A-15 + A-14 composition: let_value > 127 feeds IMM32 cmp encoding |
| match_let_arm_false | match_arm_b_let | A-17's arm-B let path substitutes let_value when FALSE arm is the let-bound IDENT |
| match_let_scrut_ne | match_scrut_let_fold_ne | A-19 op-tag dispatch wires setne for `!=` (distinct from setl/setg/sete) |
| match_let_scrut_lt | match_scrut_let_fold_lt | A-19 op-tag dispatch wires setl for `<` |
| match_let_scrut_le | match_scrut_let_fold_le | A-19 op-tag dispatch wires setle for `<=` (boundary case: 7 <= 7 → true) |
| match_let_scrut_gt | match_scrut_let_fold_gt | A-19 op-tag dispatch wires setg for `>` |
| match_let_scrut_ge | match_scrut_let_fold_ge | A-19 op-tag dispatch wires setge for `>=` (boundary case: 7 >= 7 → true) |
| let_multi_concurrent | multi_let_concurrent_visible | Multi-let table preserves all bindings; second `let` does NOT evict the first |
| m3y_alpha_isalpha | m3y_alpha_isalpha_lowering | M.3.Y-α: 3-level nested-match `is_alpha`-shape → 32-byte LEAF range-check (variable-size emit foundation) |
| m3y_beta_isws | m3y_beta_isws_lowering | M.3.Y-β: 3-level nested-match `is_ws`-shape → 40-byte LEAF chain-of-eq, IMMs 32/9/10/13 |
| m3ac_narrow_classify_punct | m3ac_narrow_classify_punct_lowering | M.3.AC-narrow: 20-arm match → 232-byte cmp-cascade (pre-built in .data, single SYS_WRITE) |
| m3z_empty_zeros | m3z_empty_zeros_lowering | M.3.Z: all-zero-literal body → 24-byte `xor eax,eax; xor edx,edx; ret` (ABI-correct ≤16-byte aggregate return) |
| m3z_safety_rejects_fn_call | m3z_safety_gate_rejects_bare_fn_call | Locks M.3.Z's fn-table-walk safety gate: `caller{helper(0)}` MUST fall back to M.3.D-narrow, not misfire as M.3.Z |
| m3g_let_mut_fold | m3g_let_mut_fold_lowering | Session 6a M.3.G-let-mut-fold: `let mut V = INIT; [V = INT;]* (return V; \| V)` lowers to 24-byte LEAF `mov eax, FINAL_INT; xor edx, edx; ret`. First emit shape with let_count > 0 and per-fn IMM metadata (`fn_let_int_table`) |
| m3g_let_mut_arg_passthrough | m3g_let_mut_arg_passthrough_lowering | Session 6b M.3.G-let-mut-arg-passthrough: `let mut V = arg; (return V; \| V)` (init from param0, no reassignments) lowers to 24-byte LEAF `mov eax, edi; xor edx, edx; ret`. Param0 derived inline from fn signature (Pass A-4 hasn't run yet at shape-detection time) |
| m3e_2arg_int_call | m3e_2arg_int_call_lowering | Session 7-narrow M.3.E-2arg-int-call: `return IDENT(INT, INT);` lowers to 24-byte LEAF `mov edi, ARG1; mov esi, ARG2; call rel32; xor edx, edx; ret`. First multi-arg call shape; per-fn metadata in `fn_call_meta` (12B/fn). Bug-fix landing: int_handler in main scan was overwriting r13=3 sentinel — made saw-fn-call state sticky |
| m3i_load_start | m3i_load_start_lowering | Session 8 M.3.I-load-narrow: `param0 . start }` (bare-form struct field read) lowers to 24-byte LEAF `mov eax, esi; ret; nop×21`. First struct-field accessor shape. Sound discriminator: ident_table + name_pool walk verifies field IDENT bytes literally match "start" — `s.kind`/`s.payload`/`s.end` correctly fall back to M.3.D-narrow. ABI: ≤16-byte struct in (rdi, rsi); s.start at offset 8 = low 32 of rsi |
| m3g_let_mut_call_reassign | m3g_let_mut_call_reassign_lowering | Session 6c M.3.G-let-mut-call-reassign: `let mut V = INT_INIT; V = FN(); V }` lowers to 32-byte non-LEAF block with real `sub rsp, 16` frame, mov [rbp-4] INIT, call rel32, mov [rbp-4] eax, mov eax [rbp-4], epilogue. First 32-byte non-LEAF cluster. Per-fn metadata: target_offset + INT_INIT (fn_call_meta) |
| m3i_store_kind_imm | m3i_store_kind_imm_lowering | Session 10 M.3.I-store-narrow-kind-imm: `param0 . kind = IMM ; param0 }` lowers to 24-byte LEAF `mov rax, rdi; mov al, IMM; mov rdx, rsi; ret`. First struct-field STORE shape. Sound: ident_table walk verifies field is literally `kind`. ABI: ≤16-byte struct in (rdi, rsi); kind = low byte of rdi |
| m3k_empty_huge_struct | m3k_empty_huge_struct_lowering | Session 11 M.3.K-empty-huge-struct: `Phase0LexState { ... }` (all-zero huge-struct constructor) lowers to 24-byte LEAF `mov r8, rdi; xor eax, eax; mov ecx, 0x5000D; rep stosb; mov rax, r8; ret`. First hidden-pointer struct-return ABI emit. Sound: ident_table walk verifies type is literally `Phase0LexState` (14 bytes). Chain-advancing: lowers phase0_compiler.phos's `empty_lex_state` correctly; phase-0 hash advanced from `e7b32c63…` to `2d56eca3…` |
| m3k_empty_ast_node | m3k_empty_ast_node_lowering | Session 19 Stream A Frontier #2: extends Session 11's M.3.K primitive to a second hidden-ptr target type — Phase0AstNode (24 B / 0x18). Same 24-byte LEAF emit shape; the byte count `0x18` is read from the new per-fn `fn_huge_struct_size` table populated by an additive `Phase0AstNode` ident_table walk in `.Lrfo_shape_check_m3i_store` (r13=2 dispatch path). Sound: ident_table walk verifies type is literally `Phase0AstNode` (13 bytes). Chain-advancing: lowers phase0_compiler.phos's `empty_ast_node` correctly; phase-0 hash advanced from `2d56eca3…` to `da3722b1…` |

## Current inventory (residual-truth tier)

The first goal-#3 fixture landed in Session 12 (2026-04-30). This is a
spec-pinning fixture: it locks the R1 record ABI and chain_hash math
that all future R1..R7 runtime-emission fixtures depend on, but does
NOT yet assert producer-side emission (that is Session 14+ per Stream C
Milestone B).

| Fixture | Behavior class | Ambiguity removed |
|---------|----------------|-------------------|
| residual_r1_byte_layout | residual_r1_record_byte_stable | Session 12 first residual-truth fixture: pins the 32-byte R1 record's field declaration order/widths in `kernel/residual.phos` (kind:u8, arch_id:u8, seq:u16, cycle:u64, payload:[u8;14], chain_hash:[u8;4]) and the four chain_step prime constants (31, 131, 524287, 16777213). On a fixed cap_issue test vector (kind=1, arch_id=0, seq=1, cycle=0, payload=[1,5,0..0], prev=[0;4]) the chain_hash output is byte-locked to [0xF8, 0x18, 0xF8, 0xE8] via `tools/verify/check_residual_byte_layout.sh`. Per FORENSIC_PRIMACY.md §1+§2 |
| residual_r5_mmio_byte_layout | residual_r5_mmio_record_byte_stable | Session 14 (Stream C Milestone B): pins the R5 mmio_touch payload schema (declared_lo @0..2, declared_hi @2..4, observed_addr @4..8, reserved @8..14) on the canonical MMIO boundary test vector (declared 0x1000..0x10FF, observed 0x1100); chain_hash byte-locked to [0x8A, 0xA2, 0xCA, 0x5E]. Couples to mmio_boundary_violation.pfi by literal record-byte match. Per FORENSIC_PRIMACY.md §1+§2 |

## Current inventory (PFI0 case-file tier — Stream C Milestone A)

`.pfi` evidentiary case-file fixtures live under `tools/verify/fixtures/pfi/`
(separate from `tools/verify/fixture_manifest.toml`, which is for `.phos`
sources). Each `.pfi` is byte-stable per [`docs/PFI0.md`](PFI0.md) and is
verified by `tools/verify/check_pfi_layout.sh` (Make target
`verify-pfi-layout`, wired into `verify-legendary`).

| Fixture | Doctrine role | Ambiguity removed |
|---------|---------------|-------------------|
| mmio_boundary_violation.pfi | residual-truth (goal #3) end-to-end starter case | Session 13 Stream C Milestone A: 192-byte PFI0 container encoding one R5 mmio_touch residual for a task that touched address 0x1100 outside its declared MMIO range 0x1000..0x10FF. Pins PFI0 magic, residual_count u32 LE, manifest/image/stream hashes, R5 record (kind=5, seq=1, payload encoding declared range + observed addr), chain_hash byte-locked to [0x8A, 0xA2, 0xCA, 0x5E], final_chain_hash anchored to last record. sha256 e689dbeb…; verified by check_pfi_layout.sh. **Court Requirement A1/B1 (2026-05-03)**: this anchor is also reproducible from the (declared_lo, declared_hi, observed) input vector via the host reference emitters at `tools/court/emit_r5_record.sh` + `tools/court/emit_mmio_boundary_pfi.sh`; the replay gate `tools/verify/check_court_a1_b1.sh` (Make target `verify-court-a1-b1`) asserts byte-equality between emitter output and this fixture, so the fixture is no longer the sole authority for the bytes. **Court Requirement B1 narrow (2026-05-03)**: the semantic invariant `observed ∉ [declared_lo, declared_hi]` is enforced on the produced bytes by `tools/court/validate_r5_case.sh` via gate `tools/verify/check_court_b1_case_validity.sh` (Make target `verify-court-b1-case-validity`). The fixture's payload encodes a real boundary violation (0x1100 > 0x10FF), not an in-range touch mislabeled R5. |
| mmio_boundary_violation.expect | forensic-classification (goal #4) verdict spec pin | Session 15 Stream C Milestone C: byte-locks the verdict bytes the classifier MUST produce when adjudicating mmio_boundary_violation.pfi: CLASS=MMIO_BOUNDARY_PRESSURE, RESIDUAL=R5, SEQ=1, EXPECTED=mmio_range[0x1000..0x10FF], ACTUAL=0x1100, EXIT=6. Verified by check_verdict_replay.sh — enforces canonical 6-line format from FORENSIC_PRIMACY.md §3, closed DriftClass enum, exit-code mapping, no log-analyzer vocabulary, and cross-checks RESIDUAL/SEQ against the .pfi's record[0]. **Court Requirement D1 (2026-05-03)**: this expectation is also derivable from the A1/B1-produced PFI0 bytes via the host reference verdict tool at `tools/court/verdict_from_pfi.sh`; the gate `tools/verify/check_court_d1_verdict.sh` (Make target `verify-court-d1-verdict`) chains `emit_mmio_boundary_pfi.sh | verdict_from_pfi.sh | cmp .expect`, closing the host-reference loop input vector → R5 → PFI0 → verdict. Host reference verdict path — not a Phosphoric-compiled classifier. |
| pfi/malformed/* (7 fixtures) | residual-truth (goal #3) adversarial rejection | Session 16 Stream C Milestone D: seven hand-encoded malformed .pfi fixtures (bad_chain_hash, seq_gap, bad_kind, truncated_record, nonzero_reserved, bad_magic, stream_hash_mismatch) each rejected by check_pfi_layout.sh with a deterministic named violation. Verified by check_malformed_pfi.sh — each malformed fixture has a closed expected-reason entry; rejection for the wrong reason is itself a violation. The court refuses bad evidence with named verdicts, never "looks unusual" |

## Missing fixture classes (architectural blockers)

These fixture classes are NOT in the corpus because the producer cannot
yet lower the source shapes they require. They are documented here as
explicit gaps. None will be added to the manifest until the architectural
blocker is removed.

### Compiler-closure gaps
- `two_arg_call` — needs >24-byte fn block (current 24-byte invariant
  doesn't fit `mov edi/esi/edx + call` plus prologue/epilogue). Variable-
  size emit foundation (Sessions 1–5-mini) supports >24-byte blocks now,
  but multi-arg call sequencing is still a separate session.
- `assignment_update` — needs variable frame_size + per-fn `let mut`
  slot tracking. (Panel-revised 2026-04-30: the original "writable VMA"
  framing was wrong. PT_LOAD's `0x5 = PF_R | PF_X` flag is irrelevant
  because `let mut` reassignment uses the STACK, which is RW by default
  and not part of PT_LOAD. The real blocker is per-fn frame layout in
  the producer. Session 6a/6b/6c will close this incrementally.)
- `bounded_loop_assignment` — same blocker as assignment_update
  (variable frame_size + slot tracking, not writable VMA).
- `struct_field_load`, `enum_tag_match`, `array_index_load`,
  `slice_len_check` — need struct/enum/array layout, name resolution,
  and address arithmetic. Multi-session work (Session 8 onward starts
  with `struct_field_load` for one struct).
- `syscall_write_buffer` — needs writable buffer and length tracking
  beyond the current `__sys3` literal-args primitive.

### Recently-closed gaps (2026-04-30)

These compiler-closure gaps were architecturally blocked at this doc's
prior revision and are now closed by specific producer extensions.
Each closing extension has a byte-locked fixture pinning its emit:

- `is_alpha` — closed by **M.3.Y-α** (3-level nested-match returning
  bool with range-check IMMs 65/90/97/122). 32-byte LEAF block. Locked
  by fixture `m3y_alpha_isalpha`.
- `is_ws` — closed by **M.3.Y-β** (3-level nested-match returning bool
  with chain-of-equalities IMMs 32/9/10/13). 40-byte LEAF block.
  Locked by fixture `m3y_beta_isws`.
- `classify_single_punct` — closed by **M.3.AC-narrow** (single 20-arm
  match `IDENT { INT => INT, ... _ => 0 }`). 232-byte cmp-cascade block,
  pre-built in .data, single SYS_WRITE per emit. Locked by fixture
  `m3ac_narrow_classify_punct`.
- `marker { 0 }`, `check_acyclicity { Ok(0) }`, and `empty_token { ... }`
  — closed by **M.3.Z-empty-zeros** (body initializes return value
  entirely from zero literals; ABI-correct for ≤16-byte aggregate
  returns via `xor edx, edx`). 24-byte LEAF block. Locked by fixture
  `m3z_empty_zeros`. Companion fixture `m3z_safety_rejects_fn_call`
  pins the discriminator's safety gate (rejects bare-fn-call patterns
  from misclassifying as M.3.Z).

### Task-seal gaps (entire goal #2 unimplemented)
- `manifest_effect_exceed_negative`
- `manifest_cap_exceed_negative`
- `manifest_mmio_range_negative`
- `manifest_ipc_undeclared_negative`
- `manifest_budget_exceed_negative`

These need a manifest-aware producer: source → compile-time check
against declared authority. The producer currently lacks any manifest
reader. Multi-session.

### Residual / forensic gaps (entire goals #3 and #4 unimplemented)
- `residual_cap_issue`, `residual_mmio_touch`, `residual_boot_check_*` —
  need R1..R7 emission infrastructure. Producer must insert emission
  sites at boundary points and guarantee monotonic seq + chain_hash.
- `incident_classification_idempotent` — needs the `dsfb-gray`
  classifier (`tools/phosphoric-host/phosphoric_drift.phos`) which is
  currently a doctrine stub. Multi-session.

### Fixpoint gaps
- `phase0_chain_regression_visible` — `tools/verify/run_chain.sh` already
  traces the chain; will be reframed as a fixture-tier check (separate
  category in the manifest) tracking the chain depth and break point.
- `phase0_chain_non_regression_when_ready` — pending true selfhost
  convergence on `phase0_compiler.phos`. Currently the chain degrades
  through M.3.M shells. Adding this fixture before the chain converges
  would falsely claim convergence.

## Why no industrial compiler test suite

A normal compiler test suite tests *features*. Phosphoric's corpus
tests *evidence*: every fixture is a witness that the producer cannot
silently mislower a specific shape, that a sealed task cannot exceed
its authority, that a residual stream cannot drift, that a verdict
cannot vary.

If we tested features, we would inflate the corpus to thousands of
fixtures and each one would carry less evidence per byte. The razor
exists to keep the corpus minimal-but-load-bearing.

## Razor gate

`make verify-fixture-razor` enforces the doctrine. It fails closed if any
fixture entry in `tools/verify/fixture_manifest.toml`:
- has no `behavior_class`
- has no `ambiguity_removed`
- maps to none of the four doctrine flags
- duplicates another fixture's `behavior_class` without
  `justified_duplicate = "<reason>"`
- references a missing source file

The corpus is correct only if both `verify-fixture-corpus` and
`verify-fixture-razor` pass.
