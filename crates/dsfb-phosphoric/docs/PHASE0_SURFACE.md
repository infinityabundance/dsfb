# Phase-0 producer surface — current state

Apex framing: **the producer surface is what `phase0_stub.S` can lower today, sub-pass by sub-pass.** This doc is the single canonical "current state" inventory of the source↔ASM closure campaign (Sessions B–S, gate 51 / 82). The campaign is the audit-of-record per `GOAL.md` §"Bootstrap discipline".

Producer source: `untracked/internaldocs/phase0_producer/phase0_stub.S` (4584 lines x86_64 ASM).

Phase-0 binary hash: `da3722b1920580d5458852bcfe3768a7cf04307d3ad143b0011c6d85005b8b5a` (size 2537 B). Confirmed via `sha256sum build/phase0/pcc-stage0.bin` against this doc's date. Advanced 12 times: 10× on 2026-04-30 + 2× on 2026-05-01 (Session 11 M.3.K-empty-huge-struct lowers `empty_lex_state`; **Session 19 M.3.K-extension lowers `empty_ast_node` via the same hidden-ptr ABI primitive with size 0x18**). Prior 11 advances: — M.3.G-mid; M.3.P (`marker { 0 }`); M.3.S (`check_acyclicity { Ok(0) }`); M.3.V (`is_digit`); M.3.AA-α (`is_ident_start`); M.3.AA-β (`is_ident_cont`); M.3.Y-α (`is_alpha` 32-byte LEAF range-check + variable-size emit foundation); M.3.Y-β (`is_ws` 40-byte chain-of-equalities + tightened discriminator); M.3.AC-narrow (`classify_single_punct` 232-byte cmp-cascade); M.3.Z-empty-zeros (`marker / check_acyclicity / empty_token` 24-byte LEAF `xor eax,eax; xor edx,edx; ret`, ABI-correct for u32 and ≤16-byte aggregate returns).

## Landed sub-passes

The producer accumulates sub-pass markers as `# Pass M.3.X-tag:` and `# Pass A-N:` in source comments. The current set, grouped by family:

### M.3 spine — coarse-grained selfhost ladder

| Sub-pass | Status | What it lowers |
|---|---|---|
| M.3.A | sealed | `open(argv[1])` / `close` / `exit` argv validation |
| M.3.B | sealed | real `read` / `write` cat-mode → stage1 produced |
| M.3.C | sealed | stage0 emits 136-byte canned exit-0 ELF (the entry-block template anchor at line 121) |
| M.3.D-narrow | sealed | 24-byte real prologue / store / load / epilogue per non-entry fn |
| M.3.E-tiny | sealed | `return PREV_FN();` resolves to a real `call rel32 = -37` (only the immediately previous fn) |
| M.3.E-full | sealed | `record_fn_offsets` + `fn_offset_table` (1024 entries, 8 B each) populates a symbol table at scan time; any `return IDENT();` resolves via lookup, not just the previous fn |
| M.3.F-narrow | sealed | real `mov eax, IMM; add\|sub\|imul eax, IMM` for `return A op B;` (24-byte block; idiv still folded by Pass L) |
| M.3.G-narrow | sealed | bounded `for i in 0..N { s = s + i; }` lowered to `cmp/jge/inc/jmp` byte sequence (24-byte IMM8 block; N ≤ 127) |
| M.3.G-mid | sealed | bounded `for i in 0..N` for N > 127 lowered to a distinct 24-byte block using `mov ecx, IMM32` + `test+jz` N=0 guard + `loop` self-jump. Co-exists with M.3.G-narrow's IMM8 block (dispatched at scan time on a per-fn flag) so the IMM8 path stays byte-equal. |
| M.3.P-bare-int | sealed | bare-expression return at end of fn body: `fn name() -> T { N }` (no `return` keyword, no trailing `;`) lowers to the same `mov eax, N; ret` block as `return N;` would. Scan-time pattern: TK_INT at depth 1 with peek+1 = TK_PUNCT `}`. |
| M.3.Q-simple-returns | sealed | extends M.3.P with TK_KEYWORD true/false bare returns (`{ true }` / `{ false }` → IMM 1/0), TK_IDENT bare returns (param0 IDENT → arg-passthrough `mov eax, edi; ret`; let-name IDENT → IMM let_value), AND `return true ;` / `return false ;` via Pass H's KW dispatch. |
| M.3.R-match-arm-bool | sealed | A-17's match-arm dispatch (both TRUE-branch at +7 and FALSE-branch at +11) now accepts TK_KEYWORD true/false → IMM 1/0 in addition to TK_INT and TK_IDENT-matching-let. Closes `match arg cmp INT { true => true, false => false }` (effectively returns the boolean of the comparison). |
| M.3.S-bare-constructor | sealed (with safety fix 2026-04-30) | bare-expression return of the form `IDENT ( INNER ) }` at depth=1 — Phosphoric's Result/Option constructors `Ok(N)` / `Err(N)` / `Some(N)` are treated transparently (Result/Option are u32-equivalent at phase 0). INNER may be TK_INT (literal value), TK_IDENT-matching-param0 (arg-passthrough emit shape), or TK_IDENT-matching-let_name (let_value as IMM). Gated on fn_offset_table lookup — if IDENT is a known fn, this is a bare-form fn call and falls through (NOT a constructor); the gate was added after a soundness gap was discovered: bare `helper(7)` was miscompiled as if helper were a constructor. Closes `check_acyclicity { Ok(0) }` and unlocks bare `Ok(s)` / `Some(arg)` patterns. |
| M.3.T-safe-return-constructor | sealed | symmetric to M.3.S but for return-form: `return IDENT ( INNER ) ;`. **Critical safety**: fires AFTER the fn_offset_table lookup confirms the IDENT is NOT a known fn — this distinguishes `return Ok(N);` (constructor) from `return helper(N);` (fn call, taken by the existing call dispatch). Without this gating, a token-shape ambiguity miscompiles all return-form fn calls. INNER may be TK_INT, TK_IDENT-matching-param0, or TK_IDENT-matching-let_name. |
| M.3.U-match-arm-param0 | sealed | match-arm returning param0 IDENT (`match arg cmp INT { true => arg, false => N }` and FALSE-arm symmetric). The 5-byte `mov eax, IMM` slot in A-12/13's emit block becomes `mov eax, edi; nop nop nop` (`89 f8 90 90 90`) — same 5-byte slot, different opcode bytes. Both arm-A and arm-B paths support arg-passthrough independently via flags at [rbp-140] / [rbp-144]. **First M.3.H-full-class extension** — runtime-expression arms instead of compile-time IMMs. |
| M.3.V-comparison-arm | sealed | match comparison-arm: `match SCRUT op1 IMM_X { true => SCRUT op2 IMM_Y, false => false }` (with optional trailing comma) lowers to a **24-byte LEAF-style block** (no prologue, no epilogue) with two cmp instructions plus setcc + movzx. Distinct emit shape from all prior 24-byte blocks: starts with `81 ff IM_X×4` (cmp edi, IMM32) at offset 0 instead of `55` (push rbp). Closes `is_digit { match b >= 48 { true => b <= 57, false => false, } }` and similar range-check fns in phase0_compiler.phos. Frame extended 208 → 224 to fit 3 new state slots ([rbp-204] flag, [rbp-208] inner op, [rbp-212] IMM_Y). |
| M.3.W-match-trailing-comma | sealed | optional trailing comma after FALSE arm in match-cmp expressions: `match SCRUT cmp INT { true => N, false => N, }`. Scan-time +12 check now accepts either `}` (existing) or `,` followed by `}` at +13. Defensive surface — phase0_compiler.phos uses trailing commas universally in match arms, so future extensions that close more match patterns benefit from this. |
| M.3.X-false-arm-comparison | sealed (with depth gate) | mirror of M.3.V: `match SCRUT op1 IMM_X { true => true, false => SCRUT op2 IMM_Y }` lowers to a 24-byte LEAF-style block using the **xor-init + inc trick** to fit `eax=1` in 4 bytes instead of a 5-byte mov. Depth-gated (r8 == 1). |
| M.3.AA-α-fn-call-scrutinee | sealed (with depth gate) | match scrutinee is a fn call: `match fn(arg) { true => true, false => SCRUT op2 IMM_Y }`. Lowers to a 24-byte LEAF-style block starting with `call rel32` followed by `test eax,eax; je +1; ret` (TRUE arm = "ret eax as-is from call") then `cmp; setcc; movzx; ret` (FALSE arm comparison). Closes `is_ident_start { match is_alpha(b) { true => true, false => b == 95 } }` in phase0_compiler.phos. |
| M.3.AA-β-fn-call-both-arms | sealed (with depth gate) | match scrutinee AND FALSE arm both fn calls: `match fn1(arg) { true => true, false => fn2(arg) }`. Lowers to 24-byte LEAF-style with **two call rel32 instructions**: call fn1 → test+je → ret (TRUE) → call fn2 → ret (FALSE). Two distinct rel32 offsets computed from the per-fn caller_offset. Closes `is_ident_cont { match is_ident_start(b) { true => true, false => is_digit(b) } }`. Assumes fn1 preserves edi (caller-saved per ABI but true for simple leaf-style callees). New state slot [rbp-220] for fn2 target_offset. |
| variable-size-emit foundation | sealed (preparatory, no surface advance) | infrastructure step toward M.3.Y / M.3.AC: per-fn `fn_size_table` (1024 × 4 bytes in .bss) parallel to `fn_offset_table`, populated by `record_fn_offsets` alongside the offset table. record_fn_offsets's offset computation refactored from hardcoded `1081 + 24*(rbx-1)` to `1081 + sum(fn_size_table[1..rbx-1])`. |
| Session 1: caller_offset + filesize lift | sealed (refactor, no surface advance) | closes the load-bearing gap that the panel review identified: four emit-time `caller_offset = 1081 + 24*(rbx-1)` formulas and `_start`'s filesize patch (`1057 + 24*count`) were still hardcoded after the foundation landed. Migrated all 5 sites to read fn_offset_table[rbx].file_offset / sum(fn_size_table). Output byte-identical (sha256 verified `5bcb507c…`). The variable-size emit shapes (M.3.Y, M.3.AC, etc.) can now ship without breaking call rel32 for downstream fns. |
| M.3.Y-α-isalpha-32B-leaf | sealed | First variable-size emit shape (32 bytes, +8 over the M.3.D-narrow stub). Closes `is_alpha { match b >= 65 { true => match b <= 90 { true => true, false => match b >= 97 { true => b <= 122, false => false } }, false => false } }` (3-level nested match returning bool, range-check IMMs 65/90/97/122). Emit shape: `xor eax,eax; cmp edi,65; jl .ret_zero; cmp edi,90; jle .ret_one; cmp edi,97; jl .ret_zero; cmp edi,122; jg .ret_zero; .ret_one: inc eax; ret; .ret_zero: ret; nop×6`. Discriminator (Sessions 2-3): match-count == 3 AND let-count == 0 AND first scrutinee `IDENT >= 65`. Byte-locked in fixture `m3y_alpha_isalpha` (size 1113 = 1081 + 32). |
| M.3.Y-β-isws-40B-leaf | sealed | Second variable-size emit shape (40 bytes, +16 over the M.3.D-narrow stub). Closes `is_ws { match b == 32 { ... false => match b == 9 { ... false => match b == 10 { true => true, false => b == 13 } } } }` (3-level nested match returning bool, chain-of-equalities IMMs 32/9/10/13). Emit shape: `xor eax,eax; cmp+je ladder for {32,9,10,13}; jne .ret_zero; inc eax; ret; ret; nop×14`. Discriminator (Session 3): match-count == 3 AND let-count == 0 AND first scrutinee `IDENT == 32`. The Session-3 discriminator tightening (let-count check + first-IMM check) closes the Session-2 soundness gap on `lex_integer` and `parse_tokens` (both have match-count == 3 but use `let` bindings; both now correctly fall back to M.3.D-narrow). Net stage1 size delta from Session 2: −8 bytes. Byte-locked in fixture `m3y_beta_isws` (size 1121 = 1081 + 40). |
| M.3.AC-narrow-classify-punct-232B | sealed | Third variable-size emit shape (232 bytes, +208 over M.3.D-narrow). First non-LEAF cluster — uses pre-built data block (`m3ac_narrow_block` in .data) and a single SYS_WRITE per emit, no patch sites. Closes `classify_single_punct { match b { 40 => 1, 41 => 2, 123 => 3, ..., 33 => 30, _ => 0 } }` — a 20-arm cmp-cascade. Per-arm shape (11 bytes): `cmp edi, IMM (3); jne +6 (2); mov eax, RESULT (5); ret (1)`. Block layout: 2-byte `xor eax,eax` header + 20×11 = 220 bytes of arms + 1-byte default ret + 9-byte nop pad → 232. Discriminator (Session 4): match-count == 1 AND let-count == 0 AND first-arm IMM == 40 AND first-arm result == 1. The IMM/result fingerprint distinguishes classify_single_punct from is_digit / is_ident_start / is_ident_cont (all 1-match, no-let, but with different IMMs). Byte-locked in fixture `m3ac_narrow_classify_punct` (size 1313 = 1081 + 232). |
| M.3.Z-empty-zeros-24B-leaf | sealed | Same-size byte-level correction (24 bytes, no offset shift). Closes `marker { 0 }`, `check_acyclicity { Ok(0) }`, `empty_token { Phase0Token { kind: 0, payload: 0, start: 0, end: 0 } }`, and any structurally-similar fn whose body emits only zero literals. Emit shape: `xor eax,eax; xor edx,edx; ret; nop×19` (5 instruction bytes + 19 nops = 24). The `xor edx,edx` makes this ABI-correct for both u32/i32 returns AND for ≤16-byte aggregate returns (System V x86_64 returns the second 8 bytes in rdx). Discriminator (Session 5 mini): match-count == 0 AND let-count == 0 AND body has at least one TK_INT(0) AND no nonzero TK_INT. Supersedes the prior M.3.P / M.3.S inline emits — those were per-fn flag-driven; this is a body-shape-driven shape-table classification, with the same producer outcome but unified into the new shape architecture. Byte-locked in fixture `m3z_empty_zeros` (size 1105 = 1081 + 24). |
| M.3.G-let-mut-fold-24B-leaf | sealed (Session 6a, 2026-05-01) | First emit shape allowing `let_count > 0`. Closes `let mut V : T = INIT ; [V = INT ;]* (return V ; | V)` (scalar fold of let-mut reassignment chain). Emit shape (24 bytes): `mov eax, FINAL_INT; xor edx, edx; ret; nop×16`, where FINAL_INT is patched at emit time from per-fn metadata. **First shape with per-fn IMM metadata** — new `.lcomm fn_let_int_table, 1024 * 4` parallel to `fn_shape_table` / `fn_size_table` stores FINAL_INT during scan. Discriminator runs as a dedicated 2nd-pass scan when the main scan reports `match_count == 0 AND let_count == 1 AND r13 != 3 (no fn call)`. The 2nd-pass walks body tokens through a strict state machine: 8-token init pattern (`let mut V : T = INIT ;`), zero or more 4-token reassignments (`V = INT ;`), terminal (bare `V }` or `return V ; }`). Any deviation rejects → falls back to M.3.D-narrow. **Note**: phase0_compiler.phos's `let mut` patterns all involve struct fields (`s.cursor = c`), slice indexing (`out[c] = byte`), or fn-call reassignment (`c = emit_*()`); none match this strict scalar-fold pattern. Byte-locked in fixture `m3g_let_mut_fold` (size 1105 = 1081 + 24, rc=11 from `let mut x = 5; x = 7; x = 11; x`). Producer hash unchanged at `e7b32c63…` (synthetic-only path through producer this session — Session 6c extends to call-reassign, the first chain-relevant variant). |
| M.3.G-let-mut-arg-passthrough-24B-leaf | sealed (Session 6b, 2026-05-01) | Variant of M.3.G-let-mut-fold for arg init (instead of INT init). Closes `let mut V : T = arg ; (return V ; | V)` where arg is param0 and there are zero reassignments. Emit shape (24 bytes): `mov eax, edi; xor edx, edx; ret; nop×19`. Discriminator: extends 6a's by branching at rsi[6] of the init pattern — TK_INT → shape=5 (M.3.G-fold), TK_IDENT-matching-param0 → shape=6 (M.3.G-arg-passthrough). Param0 derived inline from the fn signature (r11+3) since Pass A-4 runs after shape detection in record_fn_offsets. The arg-passthrough path forbids reassignments — any `V = ...` rejects → falls back to M.3.D-narrow. **Note**: phase0_compiler.phos's `finalize_skip { let mut s = state; s.cursor = c; s }` doesn't match because of the intervening `s.cursor = c` field assign (which requires M.3.I-store). Byte-locked in fixture `m3g_let_mut_arg_passthrough` (size 1105, rc=77 from `let mut x = arg; x`). Producer hash unchanged at `e7b32c63…`. |
| M.3.K-empty-huge-struct-24B-hidden-ptr | sealed (Session 11, 2026-05-01) | **First hidden-pointer struct-return ABI emit**. **First chain advance in 6 sessions.** Closes `Phase0LexState { ... }` (the all-zero huge-struct constructor pattern, where the IDENT is verified literally as `Phase0LexState` via ident_table walk). Emit (24 bytes): `mov r8, rdi; xor eax, eax; mov ecx, 0x5000D; rep stosb; mov rax, r8; ret; nop×8`. ABI: System V x86_64 caller passes hidden return-slot ptr in rdi for >16-byte aggregate return; callee zeros that slot for sizeof(Phase0LexState)=327693 bytes via `rep stosb` and returns the original ptr. **Stream-B validation outcome**: phase-0 hash advanced from `e7b32c63…` to `2d56eca30d521ce516318a12272517a5554dc9195bed36fd6b5b94a441bfd6a1` — phase0_compiler.phos's `empty_lex_state` (line 85) now lowers correctly. Foundation primitive for future huge-struct constructors (`empty_ast_node`, `empty_ast`) and the receiving caller-side ABI for `let mut s = empty_lex_state()`. Byte-locked in fixture `m3k_empty_huge_struct` (size 1129, rc=0). |
| M.3.K-extension-24B-hidden-ptr-AstNode | sealed (Session 19, 2026-05-01) | **Stream A Frontier #2.** Extends Session 11's M.3.K primitive to a second hidden-ptr target type — Phase0AstNode (24 bytes / 0x18). New per-fn `fn_huge_struct_size` table holds the size; emit reads from it instead of hardcoding `0x5000D`. Discriminator: an additive `Phase0AstNode` ident_table walk inserted at the front of `.Lrfo_shape_check_m3i_store` (the r13=2 dispatch path; empty_ast_node's `payload: [0;4]` triggers r13=2). On match: shape=11, size=24, fn_huge_struct_size=0x18. Same 24-byte LEAF emit shape; only the IMM32 in `mov ecx, IMM32` changes from `0d 00 05 00` to `18 00 00 00`. **Stream-B validation outcome**: phase-0 hash advanced from `2d56eca3…` to `da3722b1920580d5458852bcfe3768a7cf04307d3ad143b0011c6d85005b8b5a` — phase0_compiler.phos's `empty_ast_node` (line 634) now lowers correctly at offset 0x6a1. Phase0LexState path preserved byte-equally at offset 0x439. Byte-locked in fixture `m3k_empty_ast_node` (size 1105, sha `8cd7f2a6…`, rc=0). |
| M.3.I-store-narrow-kind-imm-24B-leaf | sealed (Session 10, 2026-05-01) | **First struct-field STORE shape**. Closes `param0 . kind = IMM ; param0 }` (8-token bare-form, where the field IDENT is verified to be exactly the literal "kind" via ident_table walk against name_pool bytes — sound, like Sessions 8/9). Emit (24 bytes): `mov rax, rdi; mov al, IMM; mov rdx, rsi; ret; nop×15`. ABI assumption: ≤16-byte struct passed in (rdi, rsi); kind is byte 0 of struct = low byte of rdi. Per-fn metadata: `fn_field_store_imm` (4B/fn). Pre-conditions: match=0, let=0, r13=2 (has nonzero INT — the IMM value). **Stream-B validation outcome**: phase-0 hash unchanged at `e7b32c63…` (no `phase0_compiler.phos` fn matches the 8-token strict pattern; the actual chain-relevant patterns like `s.cursor = c; s` use Phase0LexState which is ≫16 bytes and requires hidden-pointer struct-pass ABI not currently lowered). Synthetic-only this session; foundation for Sessions 11+ compositions. Byte-locked in fixture `m3i_store_kind_imm` (size 1105, rc=0). |
| M.3.G-let-mut-call-reassign-32B | sealed (Session 6c, 2026-05-01) | **First 32-byte non-LEAF block** with real per-fn `sub rsp, 16` frame. Closes `let mut V : T = INT_INIT ; V = FN() ; V }` (16-token strict pattern, FN must be in fn_offset_table). Emit (32 bytes): `push rbp; mov rbp,rsp; sub rsp,16; mov DWORD [rbp-4], INT_INIT; call rel32; mov [rbp-4], eax; mov eax, [rbp-4]; mov rsp,rbp; pop rbp; ret; nop`. The `mov [rbp-4], INT_INIT` is preserved despite being immediately overwritten by the call result — preserves source-level let-mut semantics step-by-step. Discriminator pre-conditions: match=0, let=1, r13=3 (saw fn call). Per-fn metadata: target_offset + INT_INIT (in fn_call_meta — reused 12B/fn slot from Session 7). rel32 = target_offset - caller_offset - 20 (call inst at byte 15, end at byte 20). Byte-locked in fixture `m3g_let_mut_call_reassign` (size 1137 = 1081 + 24 (target) + 32 (caller), rc=77). Producer hash unchanged at `e7b32c63…` (no phase0_compiler.phos fn matches the strict 16-token pattern; chain-relevant variants need composing this with multi-arg calls or struct-field stores). |
| M.3.I-load-narrow-start-24B-leaf | sealed (Session 8, 2026-05-01) | **First struct-field accessor shape**. Closes `fn NAME(s: TYPE) -> RET { s.start }` (bare-form, 4-token body `IDENT . IDENT }` where IDENT[0]==param0 and IDENT[2] is verified to be exactly the string "start" via ident_table walk against name_pool bytes). Emit (24 bytes): `mov eax, esi; ret; nop×21`. ABI assumption: ≤16-byte struct passed in (rdi, rsi); `s.start` at offset 8 = low 32 bits of rsi for the canonical `{u8, u32, u32, u32}` Phase0Token-shape layout. Sound discriminator (rejects `s.kind`/`s.payload`/`s.end` → M.3.D-narrow fallback): the ident_table walk requires entry length == 5 AND name_pool bytes match `s/t/a/r/t` literally before allowing the field's name_pool offset to match. Pre-conditions: match=0, let=0, r13=0 (no INT seen, no fn call). Byte-locked in fixture `m3i_load_start` (size 1105, rc=0; entry returns 0). Producer hash unchanged at `e7b32c63…` (no phase0_compiler.phos fn has the strict bare-form `s.start }` pattern outside match arms or as part of a larger expression). |
| M.3.E-2arg-int-call-24B-leaf | sealed (Session 7-narrow, 2026-05-01) | **First multi-arg call shape**. Closes `return IDENT(INT, INT) ;` (where IDENT resolves via fn_offset_table walk to a known fn). Emit shape (24 bytes): `mov edi, ARG1; mov esi, ARG2; call rel32; xor edx, edx; ret; nop×6`. Reuses fn_offset_table for rel32 computation: `rel32 = target_offset - caller_offset - 15` (call_inst_end at caller_offset + 15 = 5 mov edi + 5 mov esi + 5 call). Per-fn metadata in new `.lcomm fn_call_meta, 1024 * 12` (target_offset + arg1 + arg2). Discriminator pre-conditions: match=0, let=0, r13=3 (saw fn call). 9-token state machine: `return IDENT ( INT , INT ) ; }`. **Bug caught and fixed during landing**: int_handler in main scan was overwriting r13=3 (saw-fn-call sentinel) when seeing IMM args inside the call expression — fixed by making the saw-fn-call state sticky (r13==3 short-circuits int_handler). Byte-locked in fixture `m3e_2arg_int_call` (size 1129, rc=13 from `helper(13, 99)` returning first arg). Producer hash unchanged at `e7b32c63…` (no phase0_compiler.phos fn has the strict pattern outside match arms). |
| M.3.H-mini | sealed | `match BOOL_LITERAL { true => INTLIT, false => INTLIT }` |
| M.3.M-foundation | sealed | canned_minimal_elf grown from 136 B → 451 B; doctrine-anchor for the recursion-1 layout (120 header + 195 entry + 766 canned) |
| M.3.M-recursion-1 | sealed | filesize formula = 120 + entry_size + 24·(count-1) for the recursive stage layout |

### A-pass micro-ladder — fixture-driven feature accumulation

A-1 through A-19 land in source. A-9 and A-16 are not present (skipped or absorbed). Feature surface by pass:

| Pass | Surface |
|---|---|
| A-1 | one-arg call `helper(INTLIT)` |
| A-2 | `__a0` magic IDENT (callee param0 placeholder) |
| A-3 | `stage0_call_entry` 195-byte custom entry block |
| A-4 | callee param0 name capture from fn signature |
| A-5 | (let-lookup site, deferred-migration target) |
| A-6 / A-7 | param-arith block (24-byte): `arg + INT`, `arg - INT`, `arg * INT` |
| A-8 | let-fold of `INTLIT op INTLIT` at scan time |
| A-10 | (let-lookup site, deferred-migration target) |
| A-11 | call-arg fold: `helper((IMM op INT))` folds expression at producer time |
| A-12 / A-13 | match-compare block (24-byte): `match arg op INT { true => N, false => N }` for `==`, `!=`, `<`, `>` |
| A-14 | match IMM32 cmp encoding (lifts the IMM8 ceiling) |
| A-15 | `match arg op LET { ... }` (let-lookup site, deferred-migration target) |
| A-17 | `match cond { true => let_name, false => N }` (let-lookup site, deferred) |
| A-18 | `let b = a + INT;` reads prior let_value (let chain, deferred site) |
| A-19 | scan-time match-cmp fold for let-bound scrutinee |
| Pass L-IDENT-RHS | `let_value op IDENT(arg)` and `INT op IDENT(arg)` lower to a reg-RHS 24-byte block (`mov eax, IMM_a; ADD/SUB/MUL eax, edi`); IDENT-matching-let resolves to let_value as IMM and joins fold path. Closed the let_plus_arg / int_plus_arg / let_times_arg soundness gaps. |

The "deferred-migration" annotation refers to the planned consolidation of A-5/A-10/A-15/A-17/A-18 onto a shared multi-let table walker. The migration is **deferred until a fixture requires it** per the fixture-razor doctrine.

## Fixture corpus mapping

79 fixtures live in `tools/verify/fixture_manifest.toml`. Each pins one producer path. Mapping fixture → sub-pass that emits its bytes:

```
exit42, let_return, binop_fold, bounded_loop, load32_filesz, quine_self
                                  ← Pass A / B / B-3 / B-4 (foundation)
call_one_arg, call_entry_one_arg ← A-1, A-3
call_param_name                   ← A-4
call_let_arg                      ← A-5 site
param_add, param_sub, param_mul   ← A-6 / A-7
param_add_let, param_sub_let, param_mul_let ← A-7 + A-10 site (add/sub/mul let-RHS dispatch)
let_plus_arg, int_plus_arg, let_times_arg ← Pass L-IDENT-RHS (let-cache+arg, INT+arg, mul-reg variants)
call_arg_fold_int, call_arg_fold_let ← A-11
match_param_eq_true, match_param_eq_false, match_param_lt,
match_param_gt, match_param_ne, match_param_le, match_param_ge ← A-12 / A-13 (eq/ne/lt/gt/le/ge jcc emission)
match_imm32                       ← A-14
match_let_cmp, match_let_cmp_imm32 ← A-15 site (+ A-15·A-14 IMM32 composition)
match_let_arm, match_let_arm_false ← A-17 site (arm-A and arm-B let paths)
match_compose                     ← A-8 + A-17 composition
match_true, match_false           ← M.3.H-mini (true-arm and false-arm dispatch)
let_fold_sub                      ← A-8 (sub variant; add/mul collapse byte-equal — input-driven fold)
let_chain                         ← A-18 site
let_multi_concurrent              ← A-18 (multi-let walker, partial)
match_let_scrut_false, match_let_scrut_true,
match_let_scrut_ne, match_let_scrut_lt, match_let_scrut_le,
match_let_scrut_gt, match_let_scrut_ge ← A-19 (6-of-6 set-cc paths in scan-time fold)
nested_call_chain                 ← A-3 + M.3.E-full chain (3-deep call)
```

All 82 pass `bash tools/verify/fixture_corpus.sh` deterministically (80 compiler-bootstrap/fixpoint-quine + 2 residual-byte-layout via Session 12 R1 + Session 14 R5). Sizes cluster at 1081 / 1105 / 1113 / 1121 / 1129 / 1177 / 1313 bytes per the file-size formula above (1113 is M.3.Y-α's variable-size cluster: 1081 + 32; 1121 is M.3.Y-β's: 1081 + 40; 1313 is M.3.AC-narrow's: 1081 + 232; M.3.Z-empty-zeros stays at 1105 = 1081 + 24). New fixtures added 2026-04-30 (in approximate landing order): `let_minus_arg`, `let_plus_let2` (Pass L-IDENT-RHS); `bounded_loop_imm32` (M.3.G-mid); `bare_int_return` (M.3.P); `return_bool_true`, `bare_bool_true`, `bare_arg_return`, `bare_let_return` (M.3.Q); `match_bool_arm` (M.3.R); `bare_constructor_int`, `bare_constructor_arg` (M.3.S); `return_constructor_int` (M.3.T-safe); `match_arm_arg` (M.3.U); `match_comparison_arm` (M.3.V); `match_trailing_comma` (M.3.W); `match_false_arm_comparison` (M.3.X); `match_fn_call_scrutinee` (M.3.AA-α); `match_fn_call_both_arms` (M.3.AA-β); `m3y_alpha_isalpha` (M.3.Y-α); `m3y_beta_isws` (M.3.Y-β); `m3ac_narrow_classify_punct` (M.3.AC-narrow); `m3z_empty_zeros` (M.3.Z-empty-zeros); `m3z_safety_rejects_fn_call` (M.3.Z safety gate lock); `m3g_let_mut_fold` (M.3.G-let-mut-fold, Session 6a — synthetic); `m3g_let_mut_arg_passthrough` (M.3.G-let-mut-arg-passthrough, Session 6b — synthetic); `m3e_2arg_int_call` (M.3.E-2arg-int-call, Session 7-narrow — synthetic); `m3i_load_start` (M.3.I-load-narrow-start, Session 8 — synthetic); `m3g_let_mut_call_reassign` (M.3.G-let-mut-call-reassign, Session 6c — synthetic, first 32-byte non-LEAF); `m3i_store_kind_imm` (M.3.I-store-narrow-kind-imm, Session 10 — synthetic, first struct-field-store shape); `m3k_empty_huge_struct` (M.3.K-empty-huge-struct, Session 11 — chain-advancing, first hidden-pointer struct-return ABI; lowers phase0_compiler.phos's `empty_lex_state`); `residual_r1_byte_layout` (Session 12 — Stream C kickoff, first `required_for_residual_truth = true` fixture; pins R1 record ABI + chain_hash determinism via `tools/verify/check_residual_byte_layout.sh`; producer codegen unchanged this session).

## Producer soundness gaps

Gaps are source patterns the producer accepts without diagnostic but lowers to bytes that do not match the source's semantic meaning. They are tracked in `tools/verify/producer_soundness_gaps.toml` and gated by `tools/verify/check_producer_gaps_doctrine.sh` (Make target `verify-producer-gaps-doctrine`). Each registry entry pins the current buggy rc, the correct rc the source means, and the producer label that owns the mis-lowering. The gate fails closed if any gap closes (producer extension fixed it; the entry must be removed and replaced with a fixture) or shifts to a different wrong outcome.

**Current state (2026-04-30): registry empty — producer has no tracked soundness gaps.** The let_plus_arg / int_plus_arg / let_times_arg gaps registered earlier on 2026-04-30 (Pass L flag-leak family, all three sharing the root cause that Pass L's `.Lefc_l_op_ok` set the M.3.F-narrow flag before validating that token +3 is TK_INT) were closed by Pass L-IDENT-RHS. The three repro sources have been promoted to fixtures of the same names in `tools/verify/fixture_manifest.toml`, byte-locking the now-correct emissions.

**Audit scope notes (binary-expression surface, 2026-04-30).** A systematic audit of binary-expression orderings revealed seven mis-lowering patterns sharing the (now-closed) Pass L flag-leak root cause. The three doctrine-distinct surfaces (covering ADD vs MUL × let-first vs INT-first × distinct visible-rc outcomes) were registered, gated, and then closed via Pass L-IDENT-RHS. The other four are clones — they were not separately registered because closing the root cause closed all of them. Audits of match-expression, function-call, for-loop, and parenthesized-expression surfaces showed *parser permissiveness* (producer accepts unsupported features and falls back to a default name_pool_idx emission) rather than soundness gaps; that is a separate class — frontend-rejection scope, not lowering scope — and is not tracked in this registry.

## Chain degradation

Self-compile chain on `phase0/phase0_compiler.phos` (per CLAIMS.md):

```
stage0 (ASM-built):  2305 B
stage1:               766 B  ← producer can only lower a strict subset of phase0_compiler.phos
stage2:               451 B  ← canned_minimal_elf shape; producer can lower less still
stage3:               136 B  ← minimal exit-0 ELF; producer cannot recognise its input shapes
stage4:               not produced
```

The chain degrades because each stage's producer surface is strictly smaller than its input requires. Convergence requires producer surface ≥ source surface for every stage. The byte-equal fixpoint is reached on `quine_self.phos` (sha `ea55ecd7…`, stage0..stage3 byte-equal) only because `quine_self.phos` is engineered to use exactly the surface the producer offers.

## Source surface required (`phase0/phase0_compiler.phos`, 1730 lines)

| Construct | Count | Producer status | Blocking sub-pass |
|---|---:|---|---|
| multi-arg fn calls | 57 | not lowered (24 B fn-block invariant) | producer fn-block-size lift (≈M.3.E-full's bigger sibling) |
| `let mut` reassignment | 54 | reads only; no writable VMA | M.3.G (full, not narrow) |
| `for` loops | 40 | narrow path only (≤ 127 IMM8) | M.3.G full |
| `match` arms (structured) | 171 | bool literal + int cmp variants only | M.3.H-full + M.3.I |
| `struct` types | 8 | no field load | M.3.I |
| `enum` types | 6 | no tag match | M.3.I |
| `Slice[...]` | 16 | none | M.3.J |
| `Result[...]` / `Option[...]` | 8 | partial (constructor only) | M.3.K |

`compiler/pcc.phos` (580 lines, the "real" compiler the producer must eventually compile) uses MORE diverse constructs (82 match arms, 15 Slice usages, module-path qualified types) but fewer total. M.3.N is its dedicated pass.

## Next concrete sub-passes

The named extensions in dependency order are listed below. **The data-driven priority order based on per-fn body impact is in [SELFHOST_BACKLOG.md](SELFHOST_BACKLOG.md)**, regenerated by `make audit-selfhost-blockers`.

1. **M.3.G-full** — full `for` loops (bound > IMM8) and `let mut` reassignment. Per the blocker audit: blocks 71% of `phase0_compiler.phos` fn bodies — co-equal top blocker with M.3.H-full. Estimated 1 plan session ≈ 4–6 micro-passes.
2. **M.3.H-full** — `match` arms over IDENT scrutinees (vs A-12/13's bool-cmp arms) and arms returning IDENT (vs A-17's bool-arm IDENT). Per the blocker audit: blocks 71% of bodies. Estimated 1 plan session ≈ 3–5 micro-passes.
3. **M.3.I-narrow** — single struct, single field-load + assign: `s.x` and `s.x = expr;` lower via a layout table global. Per the blocker audit: blocks 44–54% of bodies (load + store branches). Estimated 1 plan session ≈ 4–6 micro-passes.
4. **boolop** — `&&` / `\|\|` short-circuit. Small but present (4 bodies). May fold into M.3.H-full's match-guard handling.
5. **M.3.J-tiny** — Slice byte-index load in `phase0_compile`'s body. Plan-stated as a major pass; per blocker audit only 1 body uses it (the rest are type signatures the producer just needs to accept syntactically). Estimated 1–2 sessions.
6. **M.3.K-tiny** — single Result/Option constructor in `phase0_compile`'s body. Same calibration as M.3.J: most usage is in type signatures. Estimated 1–2 sessions.
7. **M.3.L** — first selfhost attempt: run producer on `phase0/phase0_compiler.phos`, debug each failure, iterate. Open-ended; multiple sessions.
8. **M.3.M** — selfhost fixpoint: stage_N == stage_{N+1} byte-equal on the full compiler source. Cannot be planned in advance; depends on M.3.L surface state.
9. **M.3.N** — same fixpoint on `compiler/pcc.phos`.
10. **M.3.O** — razor sweep + ASM retirement decision.

Plan-stated budget: 8 named sub-passes, ≈ 1 session each. Realistic budget given A-history and the 2026-04-30 blocker re-calibration: **15–25 sessions of producer work** before the chain converges (revised down from prior 20–35 estimate; M.3.J / M.3.K are smaller than the plan suggested, but M.3.G-full was added to the head of the list as a co-equal top blocker).

## Doctrine status (parallel to producer surface)

Independent of the M.3 ladder, the project's four doctrine goals each have their first executable lint gate as of 2026-04-30:

| Goal | Source-as-spec | Doctrine gate |
|---|---|---|
| #1 compiler-bootstrap | `phase0/phase0_compiler.phos`, `compiler/pcc.phos` | n/a (byte-equal fixpoint is the doctrine) |
| #2 task-seal | `compiler/manifest.phos`, `kernel/manifest.phos` | `verify-manifest-schema-doctrine` + `verify-effect-lattice-doctrine` |
| #3 residual truth | `kernel/residual.phos` | `verify-residual-doctrine` |
| #4 forensic classification | `tools/phosphoric-host/phosphoric_drift.phos` | `verify-classifier-doctrine` |

The doctrine layer protects framing from drift; it does not advance the producer. The producer surface advances independently via M.3.X passes.
