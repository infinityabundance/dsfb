# Stage 0 — Production Status

This document is the single source of truth for the stage 0 bootstrap binary's production status. It complements [STAGE0.md](STAGE0.md) (the conceptual narrative) and [STAGE0_BUILD.md](STAGE0_BUILD.md) (historical generic runbook; the active runbook is [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md)).

> Per `GOAL.md` §"Bootstrap discipline": stage 0 / ASM is the honest trust anchor; the source↔ASM closure campaign (Sessions B–S, gate 51 / 82) is the audit-of-record. The 2026-05-02 cutover that revoked ASM authority was a wrong turn and is no longer doctrine; the campaign is the active discipline going forward.

---

## Current Status

**Active entry:** `phase0-x86_64-linux-2026-04`
**Status:** `SCAFFOLD`

The phase 0 stage 0 binary `pcc-stage0.bin` exists at `build/phase0/pcc-stage0.bin` (18921 bytes, x86_64 Linux ELF). SHA-256: `87a7ce772ac1b2e2a0018675d24d88eebe4be431717ad23be9b5722303739a12`. Real chain progression via M.3.M-recursion-1: stage0(phase0_compiler.phos) → stage1 (766 B; runnable real-syscall ELF) → stage2 (451 B; runnable real-syscall ELF) → stage3 (136 B; terminal exit-0 ELF) → stage4 NOT PRODUCED (terminal exit-0 has no write logic). Non-entry fn blocks now span four size clusters under the variable-size emit foundation: 24-byte M.3.D-narrow / M.3.Z LEAF, 32-byte M.3.Y-α range-check LEAF, 40-byte M.3.Y-β chain-of-eq LEAF, and 232-byte M.3.AC-narrow cmp-cascade. `objdump` confirms real x86_64 instructions across the entire binary including real `call rel32` (M.3.E-full), real `add/sub/imul eax, IMM32` (M.3.F-narrow), real `add/sub/imul eax, edi` reg-RHS (Pass L-IDENT-RHS), real bounded loops in two emit shapes — IMM8 (`xor ecx,ecx; cmp ecx,N; jge; inc; jmp`, M.3.G-narrow) and IMM32 (`mov ecx,N; test; jz; loop`, M.3.G-mid) — and real shape-discriminated emits for `is_alpha`/`is_ws`/`classify_single_punct`/`marker`/`check_acyclicity`/`empty_token` (M.3.Y-α / M.3.Y-β / M.3.AC-narrow / M.3.Z).

**Convergence gate (stage2 == stage3) is structurally unreachable via canned_elf shells alone.** Each shell layer adds 315 B (header + entry) and pushes the break point one step deeper, but the deepest stage is always the 136 B terminal exit-0 which has no write logic. Closing the gate truthfully requires either (a) a quine (ASM-stub-emitted: forbidden per doctrine; pure-Phosphoric: requires full selfhost — see below), or (b) real lowering of phase0_compiler.phos's compilation logic into stage1's emitted bytes such that stage1(pcc.phos) is itself a Phosphoric compiler, and the chain converges at the fixpoint of compiling pcc.phos.

**Pure-Phosphoric quine path (per directive):** to land a quine in pure Phosphoric (no ASM-stub-emitted quine), the producer must lower a self-replicating Phosphoric source — `phase0_compiler.phos`'s `emit_text_section` plus syscall primitives — into emitted x86_64 instructions. This requires:
1. Extending `phase0_stub.S` to lower direct-syscall Phosphoric constructs (currently the producer lowers `return INTLIT`, `let-return-IDENT`, `A op B`, bounded `for`, fn calls — but NOT syscalls).
2. Writing a minimal self-replicating `phase0_compiler.phos` source that compiles itself byte-equal.
3. Verifying the loop: producer compiles phase0_compiler.phos → stage1; stage1 compiles phase0_compiler.phos → stage2; stage2 == stage1.

**Real-lowering path (after the pure-Phosphoric quine):** extend `phase0_compiler.phos` to compile arbitrary Phosphoric source (including `compiler/pcc.phos`). When stage_N compiles pcc.phos to stage_{N+1}, and stage_{N+1} compiles pcc.phos to stage_{N+2} byte-equal — that's gate 2 met for the real chain. ASM stub retires when this holds reliably for both phase0_compiler.phos and pcc.phos.

Both paths together = months of focused solo work per the project roadmap. Cannot be compressed into a single conversation. Hash evolution: `a71f8b0d…` (pre-Pass-J, 952 B) → `04f4c4fd…` (Pass J interning, 952 B) → `790e5317…` (Pass O.1, 952 B) → `54a6a380…` (Pass M.3.A real-syscall entry, 1001 B) → `e854f90d…` (Pass M.3.B read+write entry, 1127 B) → `ba6b98b3…` (Pass M.3.C canned_minimal_elf appended, stage1 is a runnable 136 B ELF, 1267 B). The producer is the deterministic hand-coded x86_64 ASM stub `phase0_stub.S` invoked via `untracked/internaldocs/phase0_producer/produce_stage0.sh`. Attester reproduction harness: `untracked/internaldocs/phase0_producer/attest_repro.sh`.

**Bootstrap chain progression (verifiable via `make verify-fixpoint`):**

| Pass | stage0 emits | stage1 status | chain breaks at |
|---|---|---|---|
| M.3.A | exit-0 with real argv[1] open/close validation | stage1 not produced (no write logic) | stage0→stage1 |
| M.3.B | input bytes verbatim (cat mode) | stage1 = .phos source bytes (not an ELF) | stage1→stage2 (not runnable) |
| M.3.C | 136-byte canned exit-0 ELF | stage1 = runnable ELF, exits 0 | stage1→stage2 (stage1 doesn't write argv[2]) |
| M.3.D | (multi-session) stage1 with real read+write logic | requires real codegen of stage0_entry-equivalent into stage1 (= quine UNLESS we have real lowering of arbitrary `phase0_compiler.phos` logic into instructions) | — |

This is a **scaffold-tier attestation** — a single self-signed entry. The `verify-bootstrap-status` Makefile target prints an informational warning whenever the manifest is below the active state.

| Field | Value |
|---|---|
| `[[stage0]] status` | `SCAFFOLD` |
| `binary_url` | `build/phase0/pcc-stage0.bin` (local; HTTPS hosting on first independent attester) |
| `binary_sha256` | `ba6b98b3e8e4821587ec4a3d0912b1878ffacd8dc42e3126b7111397cd7f9fa1` |
| `binary_size` | 1267 (= 120 header + 195-byte stage0_entry + 136-byte canned_minimal_elf + 51 × 16 non-entry blocks) |
| `created_on` | `2026-04-29T03:22:10Z` |
| `created_by` | `scaffold-attester` (single-author session) |
| `producer` | hand-coded x86_64 ASM stub (Pass C+D+E+G++H+J+K+L+M.3.A+M.3.B+M.3.C: prior constant-fold passes plus REAL syscall codegen in produced binary's entry block — argc check, open(argv[1]), read into 4096-byte stack buffer, close, argc≥3 branch, open(argv[2]), WRITE 136-byte canned_minimal_elf from absolute virtual address 0x40013B, close, exit with Pass-H-captured value) plus phase0_compiler.phos Pass O.1 (parse_return_stmt captures INTLIT into AST payload — source-as-spec parity for Pass H) |
| `producer location` | `untracked/internaldocs/phase0_producer/phase0_stub.S` (out-of-tree, ~1900 LOC) |
| `attestation count` | 9 (scaffold-self, …-pass-g+, …-pass-h, …-pass-jk, …-pass-l, …-pass-o1, …-pass-m3a, …-pass-m3b, …-pass-m3c) |
| `attestation diversity` | all nine entries are single-author and same session; attestations record the producer's evolution (Pass G+ → H → J+K → L → O.1 → M.3.A → M.3.B → M.3.C). |
| `reproduction harness` | `untracked/internaldocs/phase0_producer/attest_repro.sh` (Pass R.1; verifies the pinned source/output hashes and prints a paste-ready `[[stage0.attestation]]` block on match) |

---

## What's Landed (the architecture is complete)

1. ✅ **Phase 0 source with real pass bodies** at [phase0/phase0_compiler.phos](../phase0/phase0_compiler.phos). 1709 LOC of Phosphoric. Real lexer (19 keywords, 30 punct, idents, integers, line comments). Real parser walking module + profile + item list (struct/enum/fn/capability), with non-recursive iterative descent per v0 grammar. **Real function-body parsing (Pass G, 2026-04-29):** `parse_function_body` dispatches on token kind to per-statement parsers — `parse_let_stmt`, `parse_return_stmt`, `parse_expr_stmt`, `parse_for_with_bound`, `parse_for_stmt_no_bound`. Each statement appends a `NodeStmt*` AST node; the function node's `payload[0..2]` records the stmt-id range. Real type-check rejecting duplicate top-level names. Real `emit_elf` producing valid x86_64-unknown-linux-gnu executables. Per-instruction codegen lowering (let initializer → mov + arithmetic → instructions, etc.) is the next deliverable per the elevation roadmap.
   - **Pass G+ ASM producer codegen distinguishability (2026-04-29):** `phase0_stub.S` `emit_per_fn_codes` now emits a 16-byte block per fn keyed on `name_pool_idx` (push rbp / mov rbp,rsp / mov eax,name_pool_idx / mov rsp,rbp / pop rbp / ret / nop nop), so two sources with identical fn count but different fn names produce different binaries. Includes the r10/r11 syscall-clobber fix: emit_per_fn_codes restructured to use only callee-saved registers (rbx/r12-r15) plus stack scratch at [rbp-8] (depth) and [rbp-24] (16-byte code-block buffer). Determinism verified: two consecutive runs against `phase0/phase0_compiler.phos` produce byte-identical 952-byte ELFs (hash `a71f8b0d…`). Cross-source check: `compiler/pcc.phos` produces a 360-byte ELF with a different hash, scaling correctly.
   - **Pass H ASM producer real `return INTLIT;` lowering (2026-04-29):** emit_per_fn_codes now performs a non-recursive forward token-buffer scan within each fn body. When the `return INTLIT ;` pattern is found, the entry fn emits a 16-byte syscall-exit block with the literal as exit code (`mov rax,60 / mov rdi,IMM32 / syscall`), and non-entry fns emit a 16-byte block ending in `mov eax,IMM32 / ret`. When not found, the entry fn keeps the canned exit-0 and non-entry fns keep the Pass G+ name_pool_idx fallback. Pass H verified end-to-end on synthetic sources: `fn main() { return 42; }` produces a 136-byte ELF that **exits 42** (not 0); the same source plus `fn helper() { return 7; }` produces a 152-byte ELF that exits 42 with a non-entry helper block byte-distinguishable from a baseline source whose fns have no return literals. Three consecutive runs reproduce byte-identical output for every input.
   - **Pass J ASM producer lexer interning (2026-04-29):** `lex_ident_or_keyword` now walks an `ident_table` (.lcomm 1024 × 8) before appending to `name_pool`; on byte-equality match against `name_pool[+offset, +length]`, it reuses the existing offset rather than appending a duplicate copy. The TK_IDENT payload is now a real interned name index — two occurrences of the same source string carry equal payload. This is the prerequisite for any name-based codegen. Phase 0 hash CHANGED from `a71f8b0d…` to `04f4c4fd…` because Pass J collapses duplicate `name_pool` entries, shifting offsets for everything after the first duplicate. Size remains 952 bytes (52 fns × 16 + 120).
   - **Pass K ASM producer real `let IDENT = INTLIT ; return IDENT ;` lowering (2026-04-29, blocked-on-J before this revision):** emit_per_fn_codes scan recognises the `let IDENT = INTLIT ;` pattern (no mut, no type annotation), saves `(let_name_offset, let_value)` to per-fn stack slots `[rbp-32]` / `[rbp-28]`, and on a subsequent `return IDENT ;` matches the IDENT payload against the saved offset (Pass J's interning makes payload-equality a real name match), capturing the let value as the lowered immediate. Pass K verified end-to-end on six synthetic sources: `let x = 7; return x;` → exit 7; `let alpha = 9; return alpha;` → exit 9; `let y = 99; return 5;` → exit 5 (Pass H wins direct return); `let x = 7; return q;` → exit 0 (no name match → fallback); main `let x = 42; return x;` + helper `let q = 17; return q;` → exit 42 with helper non-entry block carrying mov eax,0x11 (= 17); baseline `return 42;` → exit 42 (Pass H direct path still works). Three consecutive runs reproduce byte-identical output.
   - **Pass L ASM producer binary-expr constant folding (2026-04-29):** scan extended to recognise `return A op B ;` (op in {+, -, *, /}; A is INTLIT or matched-let IDENT; B is INTLIT). On match, the producer evaluates op(A, B) at producer-time (signed 32-bit; idiv for division; divide-by-zero falls back to capturing operand_a alone) and writes the result as the lowered immediate. Block size unchanged (16 B); ELF header layout unchanged. Phase-0 hash UNCHANGED at `04f4c4fd…` because no top-level fn in `phase0_compiler.phos` uses binary expressions. Frame extended 32 → 48 B to hold op_tag and operand_b across the +3/+4 peeks (since r8 is the local depth counter). Verified: `return 5+7` → 12; `return 50-8` → 42; `return 6*7` → 42; `return 100/2` → 50; `let x=10; return x+32` → 42; `return 5/0` → 5 (divide-by-zero fallback). Determinism preserved.
2. ✅ **Phase 0 subset spec** at [phase0/phase0_subset.md](../phase0/phase0_subset.md). The pinned contract for what attesters' stubs must accept.
3. ✅ **Hand-bootstrap runbook** at [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md). The active attester procedure.
4. ✅ **Producer script** at `untracked/internaldocs/phase0_producer/produce_stage0.sh` (OUT-OF-TREE per doctrine). Emits the deterministic minimal ELF that matches what phase0_compiler.phos's emit_elf produces for the trivial empty-AST case.
5. ✅ **Real binary** at `build/phase0/pcc-stage0.bin`. 136 bytes, x86_64 Linux ELF, exits 0, hash-pinned in `bootstrap.toml`. The binary's byte composition (64-byte ELF header + 56-byte program header + 16-byte exit-0 sequence) corresponds exactly to phase0_compiler.phos's output for an empty input — the producer is a Python-based reference implementation of the phase 0 emit_elf path.
6. ✅ **`bootstrap.toml`** populated: phase 0 entry has real hash, size, timestamp, scaffold attestation. Historical externally-built entry preserved with `superseded_by` pointing at phase 0.

---

## What's Pending (the elevation roadmap)

1. ⏳ **Per-instruction codegen lowering.** Pass H (return INTLIT), Pass J (interning), Pass K (let-return-IDENT), and Pass L (binary-expr constant folding) all landed 2026-04-29. **Pass M.1 + M.2 are already in the producer** — `count_top_level_fns` runs in `_start`, then `filesize = 120 + 16 × fn_count` is computed and patched into `elf_image` at `p_filesz` (offset 96) and `p_memsz` (offset 104) BEFORE the header write. Dynamic-size ELF infrastructure is therefore in place; the only fixed-size assumption is the literal `shl r13, 4` (×16) that derives total text from fn_count alone. **Pass M.3 is the real codegen frontier.** Replace the constant-multiply with a sum of per-fn sizes computed by walking each fn's body, then extend `emit_per_fn_codes` to emit blocks whose actual byte length matches the size pass. This is where the producer starts emitting real instruction sequences for runtime data flow — loop counters / `jcc` for `#[bound=N] for`, stack-slot allocation for `let mut`, real `mov / add / sub / imul / idiv` over runtime values, function-call ABI. Constant folding (Pass H/K/L) was the limit at fixed 16 B; pcc.phos's data flow is dynamic and no constant-fold trick captures that. Pass M.3's real codegen is the prerequisite for Pass N (match), Pass O (source-spec parity), and Pass P (fixpoint). Per the elevation roadbook, weeks 2–4 = J/K/L (done in this session); weeks 5–8 = M.3/N/O; weeks 9–10 = P + Q + Pass R prep; weeks 11–12 = full regression + status flip.
2. ⏳ **`verify-fixpoint.phos` becomes a live gate.** Once the binary is real-attested, the stage1→stage2→stage3 byte-equality check runs.
3. ⏳ **Long-tail E15.** Replace the scaffold producer with a true bare-metal hand-bootstrap that has no python-3 ancestry.

---

## What Doesn't Change

- **The active source remains unchanged.** No `.phos` file needs modification for the attestation cycle.
- **The demo doesn't need to migrate.** [tools/phosphoric/emit_boot_demo_from_phos.sh](../tools/phosphoric/emit_boot_demo_from_phos.sh) continues serving the QEMU UEFI smoke test.
- **`make verify` stays green.** The bootstrap chain is informational, not gating.

---

## Observability gates wired (formerly scaffold no-ops)

Two `verify-legendary` sub-targets that were `[scaffold] not yet built` no-ops now run real shell harnesses:

- **`verify-fixpoint`** → [tools/verify/check_fixpoint_chain.sh](../tools/verify/check_fixpoint_chain.sh).
  Exercises the chain stage0 → stage1 → stage2 → stage3, reports the precise break point with hashes, and exits 0 informationally while `status = "SCAFFOLD"`. When the manifest flips to `active` and the chain still breaks, this target fails with exit 1. Currently reports: stage0 hash matches manifest; `stage0 → stage1: NOT PRODUCED (current stage0 exits 0 without consuming argv)`; bottleneck is producer Pass M.3.
- **`verify-bootstrap-manifest`** → [tools/verify/check_bootstrap_manifest.sh](../tools/verify/check_bootstrap_manifest.sh).
  Real TOML invariant gate. Verifies the active `[[stage0]]` entry has all required fields, the on-disk binary's SHA-256 + size match the pinned values, no two attestations share both `attestor` and `signed_at`, and the append-only `superseded_by` discipline is preserved. Currently passes.

Both harnesses are drop-in shell prototypes for the eventual `*.phos` host implementations referenced in [tools/phosphoric-host/verify_fixpoint.phos](../tools/phosphoric-host/verify_fixpoint.phos). They satisfy doctrine gate 5 ("verify_fixpoint.phos and verify-bootstrap-manifest.phos run green as part of make verify-legendary") at the shell-harness level; the .phos versions take over once phase0_compiler.phos becomes executable.

---

## Long-Tail: Reflections-on-Trusting-Trust

The phase 0 model assumes residual trust in the producer (currently the python-3 emit-ELF script; eventually attesters' hand-coded ASM stubs). The project's long-tail elevation item E15 names the harder problem: replace the scaffold with a bare-metal hand-authored stub that has no host-language ancestry.

E15 is not v0 work. It is documented honestly because the project does not pretend the trusted-trust gap is closed — only made explicit and small.

When E15 lands (years out), this document gets a section describing the bare-metal stub and the scaffold becomes purely historical.

---

## Pointers

- [bootstrap.toml](bootstrap.toml) — manifest with phase 0 entry populated
- [STAGE0.md](STAGE0.md) — conceptual narrative: what stage 0 is and what trust in it means
- [STAGE0_BUILD.md](STAGE0_BUILD.md) — historical generic runbook (now superseded by phase 0 path)
- [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md) — active attester procedure
- `untracked/internaldocs/phase0_producer/produce_stage0.sh` — scaffold producer (out-of-tree per doctrine)
- [verify_fixpoint.phos](../tools/phosphoric-host/verify_fixpoint.phos) — host program; live gate once status flips to `active`
- [Makefile](../Makefile) `verify-bootstrap-status` target — informational warning while below `active`
