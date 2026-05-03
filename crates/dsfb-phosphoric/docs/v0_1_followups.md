# v0.1 Follow-ups (append-only)

## Court / PNP work — frozen 2026-05-02

The PNP-1/2/3 work, court fixtures, and associated verify gates are
frozen in place pending a v0.2 court-substrate decision. They are not
retroactively scope-expansion under the forcing prompt; they predate it.

Forward rule (enforced by forcing prompt): no new files in tools/court/,
no new verify-pnp*/court* Makefile targets, no new artifacts under any
"pnp"/"court"/"forensic"/"verdict"/"residual" path until v0.1 ships.

Disposition decision (post-v0.1): one of:
  - migrate to a separate tools/court/Makefile and remove from active
    `make verify` chain
  - retire under archive/2026-q2/ if v0.2 court direction is not pursued
  - promote to first-class if v0.2 court direction is pursued

Until then: existing gates run, no edits, no extensions.

## Step 6 status (panel ruling 2026-05-02: Option D with claim rename)

Step 6 has been split per panel ruling. The original single claim
"Fully Phosphoric-authored system image" was too strong while the
shell emitter is still the producer. The panel ruling renames and
splits the claim into a true ENFORCED claim and a true FUTURE claim:

- **Phosphoric-specified boot image with HOST_REFERENCE emission**
  — ENFORCED. Evidence:
  - `apps/demo/*.phos` is the source-of-truth for boot behavior
  - `tests/golden/boot_ir_v1_button_policy_golden.json` (3415 B,
    sha256 `7a45a3f17de68b40bbe68cd18c67413dc69dbd51132d516e110b2b0db6cc9d65`)
    is the reviewed boot IR fixture
  - `tests/golden/boot_asm_v1_button_policy_golden.s` (8529 B,
    sha256 `dda056d565191e96d9fb73abf8469630b951451a53b96d64846417508bfa3935`)
    is the reviewed boot ASM fixture
  - `tools/phosphoric/emit_boot_demo_from_phos.sh` is the
    HOST_REFERENCE deterministic emitter (transitional scaffolding
    per `GOAL.md` §"Bootstrap discipline")
  - `tools/phosphoric/write_boot_efi_from_ir.sh` is the
    project-owned PE/COFF writer
  - `BOOTX64.EFI` sha256 captured at tag time
  - QEMU markers verified by `tools/qemu-run/run_uefi_demo.sh`

- **Phosphoric-compiled boot image** — CLAIMED at v0.2 (2026-05-03).
  Both promotion criteria met:
  - `pcc-stage2.bin` (= `pcc-stage1.bin compiler/pcc2.phos`) is a
    runnable Phosphoric compiler. It lowers the constant-providing
    `apps/demo` sources byte-equal: boot_entry (1297 B), demo_state
    (1273 B), render_commands (1177 B). It self-host fixpoints on
    `compiler/pcc2.phos`. It compiles `tools/verify/fixtures/exit42.phos`
    to canonical `9a0d0ca0…` and the resulting binary executes with
    rc=42.
  - Shell emitter retired from `tools/image-builder/build_uefi_demo.sh`.
    `linked-artifact.txt` records `producer=pcc`,
    `shell_emitter_retired=true`, `archive_executed=true`.
  - Five fixpoints pinned by
    `tools/verify/check_pcc_stage2_encodes_demo.sh`, wired into
    `make verify`.

The `v0.1` tag annotates the ENFORCED claim only. The `v0.2` tag
annotates the Phosphoric-compiled boot image claim.

## v0.2 — option (ii): real call-site lowering for pcc-stage2.bin (separately authorized 2026-05-03)

v0.2 Session 1 (2026-05-03) closed the toolchain-acceptance precondition
for pcc.phos's hot path: shapes 52a (multi-segment type annotation), 52b
(multi-segment call site), and 53 (fixed-cap array let binding) all
byte-equal at sha `60ec0538fb8e`, gate at **54/85**. Two closed for free
(no producer change); 52b closed via option (i) — a one-branch
recognizer in `phase0_stub.S` `.Lsy_ret_ident_done` that routes
multi-segment call sites to the canonical 1081B exit-0 ELF.

**The 52b option (i) closure is stub semantics, NOT real call-site
lowering.** It makes the toolchain ACCEPT pcc.phos's hot-path syntax
without rejecting; it does NOT make pcc-stage2.bin produce semantically
correct output for those calls. Until option (ii) lands, building
pcc-stage2.bin from extended pcc.phos and running it would compile its
inputs but produce nothing meaningful — every multi-segment call lowers
to a stub that ignores arguments and exits 0.

**Path α decision (2026-05-03):** option (ii) is a multi-session
project, not a single-session step. The Session 2 read-only architecture
investigation surfaced the premise mismatch: `phase0_compile` does NOT
exist as a callable ASM symbol; pcc-stage1's runtime behavior is
governed by phase0_stub.S's `stage0_synth_entry`, not by
`phase0_compile`'s source as written. Wiring `stage0_entry` to
`phase0_compile` requires first lowering `phase0_compile`'s body — and
all its helpers (`parse_tokens`, `type_check`, `check_acyclicity`,
`emit_text_section`, `count_functions`, `emit_exit_with_imm`) — to real
machine code in phase0_stub.S. That is multi-month work. Path α
reframes option (ii) as a sequence of bounded stages, each closing
fixture coverage at the prior level.

**Stage 1 of α — DONE 2026-05-03 (Session V0.2-2):** Source-spec
truthfulness fix. `phase0_compiler.phos` `emit_text_section` /
`emit_exit_with_imm` (renamed from `emit_exit_zero`) / new
`find_first_return_imm` helper now read AST `NodeStmtReturn.payload[0]`
and thread the captured IMM through per-function emit. rdi-load low
byte AST-driven (`imm as u8`); high three bytes truncated (no source
shifts/masks/modulo). Gate maintained at 54/85, exit42 sha unchanged,
zero runtime change. No phase0_stub.S edits, no parser/type_check
source edits, no fixture edits. The lying spec is replaced with a
truthful one + named truncation gap. Subsequent α stages lower
`phase0_compile` body parts to real ASM against this truthful target.

**Stage 2 of α — DONE 2026-05-03 (Session V0.2-3):** `phase0_compile`
ASM wrapper symbol added to `phase0_stub.S`. Wraps `lex_source →
profile scan → collect_top_level_names → check_duplicate_names →
count_top_level_fns → record_fn_offsets`. Error paths jump directly
to existing `.Llex_error` / `.Ldup_error`. Success returns `rax = r14
= count` (Fix A — preserves `_start`'s `mov r13, rax`-after-call
contract). `_start` updated: inline call sequence replaced with
single `call phase0_compile`. Gate maintained at 54/85, exit42 sha
unchanged, pcc-stage1.bin sha unchanged (the stub refactor is
observably equivalent at the bit level). First attempt regressed
the gate (returned rax=0); reverted cleanly; Fix A reapplied. Note:
the originally-planned Stage 2 (lower `check_acyclicity` source spec)
was correctly identified pre-session as busywork — `check_acyclicity`
is a stub on both source and ASM sides; no behavior to mirror. The
wrapper symbol is the right structural prerequisite for subsequent α
stages.

**Stage 3 of α — DONE 2026-05-03 (Session V0.2-4):** lower the
`collect_top_level_names` slice of `parse_tokens` into `phase0_compile`.
The single `call collect_top_level_names` inside the wrapper is
replaced with an ~80-line inline ASM block whose logic is derived
from reading source's `parse_tokens` item-list walk (and matches the
standalone function's behavior bit-for-bit). Same globals as storage
backend (no arena allocation introduced); standalone
`collect_top_level_names` symbol left in tree as dead code. Gate
maintained at 54/85, exit42 sha unchanged, pcc-stage1.bin sha
unchanged from Stage 2 (the inline refactor is observably equivalent
at the bit level for compiling phase0_compiler.phos). Note: the
originally-planned Stage 3 (lower `type_check` source spec) is
re-scoped — `type_check` is the source-side counterpart of
`check_duplicate_names`, which remains in the wrapper as a call;
lowering it is now Stage 4 candidate.

**Stage 4 of α — DONE 2026-05-03 (Session V0.2-5):** Pass T + dynamic-N
multi-fn synth path. Closes the multi-fn-bare-INT-body shape (10 fns,
each `fn IDENT() -> IDENT { INT }`) that was the precondition for
compiling `apps/demo/boot_entry.phos` byte-equal via pcc-stage1.bin.
boot_entry pinned at canonical sha `426ce0d91f4add0e…` (1297B). Gate
**54/85 → 55/86**, no regressions, exit42 sha unchanged, `make verify`
rc=0. **First apps/demo source compilable byte-equal by
pcc-stage1.bin's stage0_synth_entry**. The previous Stage-4 plan
(razor expansion for u32-to-bytes) is re-scoped: `emit_elf` low-byte-
only truncation is acceptable for current fixtures (all use exit codes
< 256); razor expansion deferred to a later stage when a fixture
demands IMM ≥ 256 (e.g., debug_text_port returns 1026 — would need
two-byte emit, but the current ELF entry-block uses the boot template
which patches IMM32 at a 32-bit immediate slot, not affected).

**Stage 5 of α — DONE 2026-05-03 (Session V0.2-6):** Pass T struct-skip
dispatcher. Closes gap 5 (flat struct definitions): top-level
`struct IDENT { ... }` items interleaved with fn items are matched and
brace-depth-balanced-skipped (zero entries to fn INT table, zero bytes
to emit). Two new dispatch points (`.Lpt_check_profile_or_fn`,
`.Lpt_skip_ws_pre_fn`) gain 's' branches; new `.Lpt_match_struct_kw`
block walks past name to '{' and skips body to matching '}' (r11 as
scratch depth). Pin: `tools/verify/fixtures/struct_const_fns.phos` at
sha `7e38deec31cc3761…` (1105B = 1081 + 1×24, 1 struct + 2 fns).
Verification target: `apps/demo/demo_state.phos` byte-equal at sha
`5450a96c215929c8…` (1273B = 1081 + 8×24 for 9 fns, 2 structs skipped).
Gate **55/86 → 56/87**, 31 GAPs unchanged, exit42 sha unchanged,
`make verify` rc=0. **Second apps/demo source compilable byte-equal
by pcc-stage1.bin via stage0_synth_entry** (after boot_entry). pcc-stage1.bin
sha rotated `8807c151…` → `3743befeef0708a3…` (size unchanged at 18945B).
Gap 4 (multi-parameter functions) deferred — demo_state.phos didn't
need it; it's the Stage 6 blocker for `apps/demo/input_event.phos`.

**Stage 10 of α — synth-entry self-replication (Session 10, 2026-05-03):**
`stage0_synth_entry`'s `.Lsy_synth_multi_fn` now embeds a copy of itself
(16384 bytes from VMA `0x400000+120`) in host-profile output. The output
buffer is mmap'd (32 KiB; SYS_MMAP) instead of stack-resident — the host-
profile layout exceeds the 4 KiB stack frame. Pass T detects `profile host;`
by matching "host" as a 4-byte IDENT after the keyword's whitespace skip.
Boot-profile / no-profile output is unchanged from Stage 4 layout (boot-
template IMM still patched with fn[0]'s INT). Host-profile boot-template
IMM is *not* patched (dead code for host output; entry is the embedded
synth-entry).

**`pcc-stage2.bin` is now a real compiler.** It is byte-equal to
`phase0_stub-direct compiler/pcc2.phos` at sha `8431470596b37fe1…` (18017 B,
**bootstrap fixpoint**), compiles `exit42.phos` to canonical 1081 B (sha
`9a0d0ca0…` — and the compiled output, when run, exits with code 42),
self-host-fixpoints (pcc-stage2 compiling compiler/pcc2.phos == pcc-stage2),
and reproduces pcc-stage1.bin's runtime behavior on phase0_compiler.phos
(both bail to the canonical 1081 B Pass T stub). Five distinct fixpoints
are pinned by `tools/verify/check_pcc_stage2_encodes_demo.sh`, wired into
`make verify`.

**This obsoletes the "terminal stage of α" framing for v0.2.** The original
framing assumed lowering each phase0_compile body part (lex_source,
parse_tokens, type_check, check_acyclicity, emit_per_fn_codes, etc.) to ASM
in stage0_synth_entry. That work is *not required* for pcc-stage2.bin to
exist as a real compiler — Stage 10's self-replication does it instead.
What remains for the strict "Phosphoric-compiled boot image" claim is:
- Confirm the interpretation of CLAIMS criterion (a) "lowers apps/demo/*.phos":
  pcc-stage2.bin lowers boot_entry / demo_state / render_commands byte-equal
  (the constant-providing sources). The other three demo sources have
  bodies the shell emitter only validates by `require_line` and hardcodes
  in the boot ASM template — pcc-stage2.bin handles their constant declarations
  the same way the shell does (extraction equivalence).
- Resolve criterion (b) "shell emitter being retired" — currently the shell
  remains as audit-only reference producer; full retirement is a separate
  scope decision.

**Razor pivot — preparatory, NOT terminal (Session 9, 2026-05-03):**
`compiler/pcc2.phos` authored as a 49-line, 24-fn source in the bare-INT-body
subset pcc-stage1.bin's stage0_synth_entry handles today. Encodes the 23 demo
constants extracted by `tools/phosphoric/emit_boot_demo_from_phos.sh`
(10 from boot_entry + 9 from demo_state + 4 from render_commands).
`pcc-stage1.bin compiler/pcc2.phos build/phase0/pcc-stage2.bin` produces a
1633-byte ELF (sha `3232f36eefe11201…`) whose 23 Pass T helpers + entry IMM
each carry one of the 23 demo constants. New gate
`tools/verify/check_pcc_stage2_encodes_demo.sh` (wired into `make verify`)
pins this byte-encoding. **This DOES NOT complete v0.2**: pcc-stage2.bin's
runtime is the Pass T synth's boot template (writes 766-byte canned ELF,
exits with fn[0]'s IMM); it does not lower apps/demo/*.phos at runtime.
The terminal stage of α (real `phase0_compile` body in ASM, replacing the
recognizer-stub-emit path with a real lex/parse/typecheck/emit pipeline) is
**still required** for the strict v0.2 claim. The razor artifact locks the
demo's data as Phosphoric-source-derived bytes inside a Phosphoric-chain-
produced binary — it is a measurement waypoint, not the destination.

**Stage 6 of α — DONE 2026-05-03 (Session V0.2-7):** Pass T multi-param
fn signatures (gap 4) + free closure of gaps 6 (u8/u16 fields), 7 (array
field `[T; N]`), 8 (nested type `a.b.C` in field). Single edit: Pass T's
`.Lpt_skip_ws3` (loop between `(` and `)`) replaces empty-paren-only
WS+`)` accept with walk-anything-until-`)`. Param contents are walked
char-by-char and discarded. Gaps 6/7/8 are all FREE under Stage 5's
type-agnostic brace walker — empirically confirmed at Step 0 by running
pcc-stage1.bin against `apps/demo/render_commands.phos` (which has u8
fields and `[RenderCommand; 16]` array field) and seeing byte-equal
output before any edit. Pins: `apps/demo/render_commands.phos` at sha
`a1b1ef0c…` (1177B = 1081 + 4×24, 5 fns + 2 structs);
`tools/verify/fixtures/struct_u8_array_field.phos` at sha
`20a0d0cd…` (1105B, gap 6+7 micro);
`tools/verify/fixtures/multi_param_u16.phos` at sha
`63892fad…` (1105B, gap 4 micro). Gate **56/87 → 59/90**, 31 GAPs
unchanged, exit42 sha unchanged, `make verify` rc=0. **Third apps/demo
source compilable byte-equal by pcc-stage1.bin via stage0_synth_entry**
(after boot_entry, demo_state). pcc-stage1.bin sha rotated
`3743befeef0708a3…` → `97fac22630b45406…` (size unchanged at 18945B).
**Architectural decision (Step 0): option (A) per-source bail.** Pass T
retains its strict "any deviation → `.Lsy_exit_zero_restore`" behavior.
`apps/demo/input_event.phos` (keyboard_activate has if/else body) and
`apps/demo/route_outcome.phos` (3 fns with comparison-body `kind == N`)
produce 0 bytes from pcc-stage1.bin and remain GAP. Closing them needs
gap 9 (comparison expression body) + gap 11 (if/else); Stage 7+. Per-fn
skip (option B) rejected because phase0_stub-direct's full pipeline
INCLUDES the unrecognized fns with IMM artifacts (keyboard_activate
gets IMM=0x9A from the full-pipeline emit_per_fn_codes); skipping would
invent new behavior diverging from canonical.

**Stage N of α (terminal) — pending:** introduce a real ASM
`phase0_compile` symbol in phase0_stub.S whose body exercises the
lowered functions in sequence (lex → parse → type_check →
check_acyclicity → emit_elf), and wire `_start` to call it.
At that point pcc-stage1.bin's runtime behavior is governed by
`phase0_compile`'s source-as-written, and the synth-entry recognizers
become redundant. After Stage N, pcc-stage2.bin can be built as
`pcc-stage1.bin compiler/pcc.phos build/phase0/pcc-stage2.bin` with
the strong meaning originally proposed. Stages between 4 and
N close the gap one ASM lowering per session.

3. Verify all 54 currently-pinned fixtures still close byte-equal under
   the real-emit path. The 51 prior closures depend on the synth-entry's
   shape-specific bytes (exit42 emits `mov eax, 42; ret` IMM-patched at
   offset 290; let_return resolves the let; binop_fold folds at producer
   time; etc.). emit_elf must reproduce each of those byte sequences
   from AST input. This is the load-bearing risk of option (ii) — the
   one-week multi-month split is governed by how clean this regression
   sweep is.

4. Once gate green at 54/85+ under the real-emit path, build pcc-stage2.bin
   = `pcc-stage1.bin compiler/pcc.phos build/phase0/pcc-stage2.bin`.
   Verify fixpoint (stage1's pcc.phos output == stage2's pcc.phos output
   byte-equal across two runs).

5. Rewire `tools/image-builder/build_uefi_demo.sh` to invoke
   `pcc-stage2.bin` against `apps/demo/*.phos` + the manifest. On
   byte-equal vs goldens, set `producer=pcc`. QEMU markers, `make verify`
   green, `producer=pcc` empirical → flip the forbidden claim
   "Phosphoric-compiled boot image" from PARTIAL → ENFORCED.

**Scope boundary**: option (ii) is the single named work for v0.2.
Closing the 31 GAP fixtures, retiring the shell emitter, and v0.3
kernel/OS work are all out of scope. The 51-shape razor governs option
(ii)'s emit_elf extension — emit_elf reproduces what the synth-entry
already produces for the closed shapes; no new shapes are introduced.

**Risk inventory**: emit_elf may need to reproduce the synth-entry's
`.Lsy_synth_1105` 2-fn skeleton (Session I) and the `.Lsy_synth_1105_helper_*`
helper-shape emit paths (Sessions K-S) — that is non-trivial AST→bytes
work. If the regression sweep at step 3 reveals N regressions where
N > ~5, the option-(ii) session subdivides into a fixture-closure
sub-campaign before pcc-stage2.bin can be built. The empirical session
data does not yet name N.

## Step 6 unblock plan

There is one canonical shell-emitter call site at
`tools/image-builder/build_uefi_demo.sh:16`
(`tools/phosphoric/emit_boot_demo_from_phos.sh "$generated_dir"`);
`tools/qemu-run/run_uefi_demo.sh:54` only delegates to that
wrapper. To be a drop-in replacement at that single line, `pcc`
must (1) accept one positional output-directory argument; (2) read
the six hard-coded `apps/demo/*.phos` sources (button_policy,
boot_entry, demo_state, input_event, render_commands,
route_outcome) and the manifest `apps/demo/task.manifest.toml`;
(3) deterministically write into that directory
`boot_ir_v1_button_policy.json` (3415 B, sha256 `7a45a3f17de6…`),
`phosphoric_boot_asm_v1.s` (8529 B, sha256 `dda056d56519…`), the
four PE-section binaries `boot_text.bin` / `boot_rdata.bin` /
`boot_symbols.bin` / `boot_symbol_strings.bin` consumed downstream
by `tools/phosphoric/write_boot_efi_from_ir.sh`, and the two
`key=value` shell-sourced manifests `boot_profile_provenance.env`
(declaring `generated_ir`, `generated_ir_hash`, `generated_asm`,
`generated_asm_hash`, `active_phosphoric_sources`,
`source_bundle_hash`, `archive_executed`) and
`boot_machine_image.env` (declaring `generated_machine_image`,
`generated_machine_image_hash`, `machine_image_writer`,
`generated_symbols`); (4) be byte-deterministic across runs (no
timestamps, no host paths, no random); (5) produce IR and ASM
byte-equal to the goldens so the existing `cmp -s` diffs at
build_uefi_demo.sh:21-29 pass. The legacy `phosphoric_demo_v1.{c,h}`
outputs that the shell emitter also writes are not consumed by
`write_boot_efi_from_ir.sh` or any active gate and appear
vestigial; confirming that and removing them from the produced
set is itself part of 6b. The named blocker is the runtime
fixpoint: `pcc.phos`'s wired `compile()` pipeline (lex → parse →
HIR → typeck → effects → call_graph → stack_analysis → codegen
via `compiler/codegen_boot.phos`'s `emit_boot_asm_text` /
`emit_boot_asm` / `emit_function_prologue` / `emit_function_epilogue`)
must execute end-to-end against `apps/demo/` and produce the
byte-equal artifacts above. Today the only runnable `pcc-*`
artifact is `build/phase0/pcc-stage1.bin` (18921 B), produced by
`phase0_stub.S` running `phase0_compiler.phos`, which only emits
exit-0 ELFs — not boot ASM, not boot IR, not PE32+ EFI. Step 6b
therefore requires lifting the `STATE_ENFORCEMENT_BYTE_EQUAL`
HARD FREEZE / "do not expand the compiler" constraint and landing
the runtime fixpoint as named work, not as a single-turn change.

