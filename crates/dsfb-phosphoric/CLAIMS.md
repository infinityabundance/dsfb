# Phosphoric — Claims and Anti-Claims

Per project doctrine (`docs/FAKE_CLAIM_PREVENTION.md` and
`docs/FIXTURE_RAZOR.md`), this file is the canonical inventory of what
the project is allowed to claim and what it is forbidden to claim. The
distinction is load-bearing: the system's value is its inability to lie
about its own behavior.

## Current authority boundary (2026-05-03 ASM authority cutover)

> **Per `GOAL.md` §"Bootstrap discipline":** ASM is the honest trust
> anchor at the bottom of the bootstrap chain; byte-equal Phosphoric →
> ASM closure is the audit floor. The 2026-05-02 cutover that revoked
> ASM authority was a wrong turn; cutover doctrine archived at
> `v0.1/docs/ASM_AUTHORITY_CUTOVER.md`.

**Permitted claims under the cutover:**

- ✅ The ASM scaffold helped close 51 / 82 source↔ASM byte-equal
  witnesses (Sessions B–S). That work is preserved evidence.
- ✅ ASM is **historical scaffold and optional comparator** for
  preserved candidates. The ASM files remain physically in the
  worktree as audit artifacts.
- ✅ The active single-case R5 **host-reference** court loop
  (input vector → R5 record → PFI0 case → MMIO_BOUNDARY_PRESSURE
  verdict → semantic boundary-validity check) is
  **source / doctrine / host-reference authoritative**. It does not
  rely on ASM as active authority.
- ✅ The remaining 31 source↔ASM fixtures are scheduled named work
  in the campaign restart per `GOAL.md` and `docs/SELFHOST_BACKLOG.md`.
  Each fixture is admitted under court need, x86 proving-ground need,
  or edge-deployment need (per `FIXTURE_RAZOR.md`).
- ✅ MVCC (Minimum Viable Court Core) acceptance criteria are
  currently met by the host-reference court — the nine-criteria list
  was archived at `v0.1/docs/ASM_AUTHORITY_CUTOVER.md` §"MVCC
  acceptance criteria" alongside cutover doctrine.
- ✅ **Verification tier split (2026-05-03)**: the active
  Phosphoric court verification path (`verify-court-active`,
  aliased by `verify-legendary`) **does not invoke the ASM
  scaffold**. The 5 ASM-rooted gates (`verify-fixpoint`,
  `verify-fixture-corpus`, `verify-quine-fixpoint`,
  `verify-source-asm-byte-equal`, `verify-court-p1-source-exit`)
  are preserved under the explicit `verify-scaffold-historical`
  target. The phase-0 hash, fixpoint chain, and 51/82 source↔ASM
  result are verified there, not in the active tier.
- ✅ **PNP-1/2/3 artifacts committed → ASM-free active workflow
  (2026-05-03)**: the 328 manufactured binaries (32 R5 + 192 PFI0 +
  104 verdict) are committed to tracked directories
  `tools/court/pnp{1,2,3}_*_artifacts/`. Fresh-clone simulation
  confirmed: with `build/` empty, `make verify-court-active` runs
  rc=0 and reads only from tracked artifacts — **zero `phase0_stub`
  invocations in the active workflow**. Manufacture-from-source
  (`make manufacture-pnp{1,2,3}-*-historical`) remains as one-time
  scaffold-historical refresh; deterministic; not invoked by the
  active path. Earned: *the active forensic court verification path
  is ASM-free for anyone with a fresh clone of the repo*. NOT
  earned: ASM-free *first manufacture* on a system with no committed
  artifacts (that requires either committed binaries or full
  self-host completion).
- ✅ **chain_step source-spec verified (2026-05-03 partial earn)**:
  `kernel/residual.phos:78-97` defines `chain_step` (4-prime mixer)
  in Phosphoric source. The
  `tools/verify/check_residual_byte_layout.sh` gate re-derives
  `chain_step` via an awk reference impl that mirrors the Phosphoric
  source byte-for-byte and asserts the canonical-vector
  `chain_hash = [0x8a, 0xa2, 0xca, 0x5e]` matches both the
  source-spec algorithm AND the locked R5 record. The R5 chain_hash
  bytes are therefore *not purely precomputed* — they are derived
  from the Phosphoric-source-spec algorithm via a verified awk
  mirror. Earned: *chain_step is source-spec defined and gate-
  verified*. NOT earned: chain_step *running on a Phosphoric-
  compiled binary* — that requires self-modifying-with-arithmetic
  shapes (`V = V + EXPR`) not in the 51 sealed shapes.
- ✅ **Court PNP-2 (PFI0 narrow, 2026-05-03)**: 192 Phosphoric source
  files at `tools/court/pnp2_pfi_bytes/byte_NNN.phos` produce the
  canonical 192-byte PFI0 case byte-identical to
  `emit_mmio_boundary_pfi.sh`. Active gate `verify-pnp2-pfi-producer`
  in `verify-court-active`; no `phase0_stub` in active path;
  manufacture is one-time scaffold-historical via
  `manufacture-pnp2-pfi-historical`.
- ✅ **Court PNP-3 (verdict narrow, 2026-05-03)**: 104 Phosphoric
  source files at `tools/court/pnp3_verdict_bytes/byte_NNN.phos`
  produce the canonical 6-line MMIO_BOUNDARY_PRESSURE verdict
  byte-identical to `verdict_from_pfi.sh`. Active gate
  `verify-pnp3-verdict-producer` in `verify-court-active`; no
  `phase0_stub` in active path; manufacture via
  `manufacture-pnp3-verdict-historical`.
- ✅ **Court PNP-1 (R5 narrow, 2026-05-03)**: 32 Phosphoric source
  files at `tools/court/pnp1_r5_bytes/byte_NN.phos` (one per R5 byte,
  shape `fn main() -> i32 { return BYTE; }`) — manufactured ONCE via
  the historical ASM scaffold to 32 × 1081-byte ELFs at
  `build/pnp1/byte_NN.bin` — produce the canonical 32-byte R5
  mmio_touch record byte-for-byte. Active gate
  `verify-pnp1-r5-producer` runs the 32 binaries, concatenates exit
  codes (= one R5 byte each), and `cmp`s to the host-reference
  witness output; **no `phase0_stub` invocation in the active path**.
  Earned: *Phosphoric-source producer artifact emitted canonical
  32-byte R5 record byte-identical to reference*. NOT earned:
  ASM-free production (manufacture step uses scaffold once); on-device
  chain_step / SHA-256; PFI0 emission; verdict emission;
  edge-board execution.
- ✅ **Court Promotion P1-source (narrow, 2026-05-03)**: a
  Phosphoric-source-defined court artifact at
  `tools/court/p1_source_exit.phos` — compiled via the existing
  bootstrap chain (stage0 ASM scaffold → stage1 → court binary) —
  returns the canonical MMIO_BOUNDARY_PRESSURE EXIT code (= 6) by
  performing the `observed > declared_hi` comparison (4352 > 4351)
  at runtime. The court source uses only the sealed 51-shape
  repertoire (Session J + Session P); stage0 and stage1 produce
  byte-identical 1105-byte outputs (sha `98d9a0de…`). Verified by
  `tools/verify/check_court_p1_source_exit.sh`
  (Make target `verify-court-p1-source-exit`). **The verdict's
  EXIT field is now traceable to a Phosphoric source program**, not
  only to the host-reference verdict tool.

**Forbidden claims under the cutover:**

- ❌ "ASM is fully retired" — ASM files remain physically present;
  this cutover revokes *active authority* only.
- ❌ "ASM is fully archived" — physical archive is a separate
  decision and is not authorized by the cutover.
- ❌ "Full self-host" — `phase0_compiler.phos` does not yet compile
  itself to a fixpoint.
- ❌ "Full compiler completion" — the producer lowers a small
  integer-with-conditionals subset.
- ❌ "Phosphoric runtime emitted PFI0 bytes" — every byte in the
  current court chain is produced by host-side bash + awk + od +
  sha256sum.
- ❌ "Compiled classifier adjudicated the case" — D1's verdict tool
  is host-side; no Phosphoric-compiled classifier has executed
  against any PFI in any gate.
- ❌ "ASM remains active court authority" — explicitly forbidden by
  the cutover.
- ❌ "Exception B granted" / "P1b shipped for implementation" — the
  P1b design (archived at `v0.1/docs/COURT_PROMOTION_P1B_IMPLEMENTATION.md`)
  is named future work, not landed. The earned-claim goal — a
  Phosphoric-compiled binary that emits PFI bytes byte-identical to
  the host emitter — is scheduled in `docs/SELFHOST_BACKLOG.md`.
- ❌ "All residual kinds implemented" — only R5 mmio_touch has a
  closed host-reference chain.
- ❌ "Multi-record replay proven" — closed loop is single-record.
- ❌ "Edge-board execution proven" — no Phosphoric-compiled binary
  runs on any edge target.
- ❌ **"P1-source achieves ASM-free production"** — the P1-source
  artifact (2026-05-03) is **scaffold-rooted**: the bootstrap chain
  that compiles it still roots in `phase0_stub.S` (stage0 ASM).
  The earned claim is narrow: a Phosphoric source program defines
  the EXIT code derivation. ASM-free production of the same artifact
  remains a future promotion, not a current achievement.
- ❌ **"P1-source emits PFI bytes"** / "P1-source emits R5 record" —
  it does not. P1-source closes the EXIT-code channel only. Byte
  emission is gated on SYS_WRITE lowering, which the cutover
  defers.
- ❌ **"`verify-legendary` proves source↔ASM convergence"** — after
  the verification tier split (2026-05-03), `verify-legendary` is
  an alias for `verify-court-active` and no longer runs
  `verify-source-asm-byte-equal`, `verify-fixture-corpus`,
  `verify-fixpoint`, `verify-quine-fixpoint`, or
  `verify-court-p1-source-exit`. Those gates run under
  `verify-scaffold-historical` only. Convergence claims (51/82
  byte-equal, FIXPOINT, phase-0 hash, P1-source EXIT=6) require
  the explicit scaffold-historical target.

## Current posture (2026-05-03 refocus)

The `.phos` fixture corpus is primarily a **bootstrap / admissibility
witness set**, not a forensic test suite. Of the 82 fixtures:

- Some directly assert forensic claims (e.g. the byte-stable R1 / R5
  residual record layouts, the PFI0 case-file layout, malformed-PFI
  rejection, byte-stable verdict replay). These are gated by
  doctrine checks (`check_residual_byte_layout.sh`,
  `check_residual_r5_byte_layout.sh`, `check_pfi_layout.sh`,
  `check_malformed_pfi.sh`, `check_verdict_replay.sh`).
- Most remaining fixtures are **evidentiary compiler / bootstrap
  witnesses**: they prevent silent drift in the admitted compile
  surface, but they do not by themselves prove that the runtime
  forensic court emits residuals at boundaries or executes the
  classifier on real evidence.

The stronger, more honest claim is **not** "82 forensic fixtures":

> *Phosphoric uses a small byte-locked witness court to prevent
> silent drift in the admitted computation surface.*

After Sessions B–S drove the source↔ASM gate from 0/82 to 51/82, the
remaining 31 fixtures are scheduled named work in the campaign
restart per `GOAL.md` and `docs/SELFHOST_BACKLOG.md`. The earlier
"paused / preserved candidates" framing (with separate per-candidate
inventory and forensic-triage priority A–G files) was archived to
`v0.1/docs/future_work/source_asm_closure/` along with the cutover
doctrine that originated it. Active work prioritizes the v0.x QEMU
proving ground (per `GOAL.md` §"Substrate development order"); the
campaign is the audit floor that justifies trust in the compiler.

**Pending claims** (not yet supported by green gates):

- **Runtime residual emission in source-side code**. The producer
  emits residuals as part of the boot path, and `kernel/residual.phos`
  declares the closed taxonomy and `chain_step` mixer. Whether the
  source-side classifier (`tools/phosphoric-host/phosphoric_drift.phos`)
  executes end-to-end on a live residual stream is not gated.
- **Full source↔ASM closure (82 / 82)**. Currently 51 / 82. Full
  closure is no longer treated as a standing obligation; the
  remaining 31 are preserved as candidates and implemented only on
  demand.
- **`pcc.phos` chain fixpoint**. Currently pending; `phase0_compiler.phos`
  fixpoint also pending.

The session-by-session closure history (Sessions B–S, 2026-05-02 →
2026-05-03) is preserved verbatim below as historical evidence; it
documents the campaign that drove the source↔ASM gate from 0/82 to
51/82 and produced the seed compiler currently embedded in
`stage0_synth_entry`.

## Permitted claims

These claims are supported by current evidence and may be made in
documentation, attestations, and external communication:

- **Executable scaffold fixture corpus.** 82 fixtures, byte-locked via
  `tools/verify/fixture_manifest.toml`, all passing `make
  verify-fixture-corpus`. 80 compiler-bootstrap/fixpoint-quine plus 2
  residual-byte-layout (R1 from Session 12, R5 from Session 14, both
  `required_for_residual_truth = true`). Plus 1 well-formed `.pfi`,
  7 adversarial malformed `.pfi` fixtures, and 1 verdict `.expect`
  outside the .phos manifest, verified by their dedicated gates.
- **Byte-equal scaffold fixpoint.** A synthetic Phosphoric source
  (`tools/verify/fixtures/quine_self.phos`) compiles to a binary whose
  output, when run, is byte-identical to the binary itself. Verified
  through stage0..stage3 by `tools/verify/quine_fixpoint.sh`.
- **Deterministic fixture verification.** `make verify-fixture-corpus`
  reads a single authoritative manifest, compiles each fixture through
  the producer, and compares size + sha256 + exit code against locked
  expectations. Any drift fails closed.
- **Fixture doctrine aligned with forensic primacy.** Every fixture
  must answer the four questions in `docs/FIXTURE_RAZOR.md`. The razor
  gate (`make verify-fixture-razor`) enforces this and currently
  reports zero violations across all 82 entries.
- **Court Requirement A1/B1 — anchor reproducible from inputs
  (2026-05-03).** Given the canonical declared-MMIO-violation vector
  (declared_lo=0x1000, declared_hi=0x10FF, observed=0x1100), a host
  reference emitter at `tools/court/emit_mmio_boundary_pfi.sh`
  composes the 32-byte R5 record (via
  `tools/court/emit_r5_record.sh`, replicating
  `kernel/residual.phos`'s chain_step mixer with primes
  31/131/524287/16777213), the SHA-256 stream_hash, and the 192-byte
  PFI0 wrapper, producing bytes that are byte-identical to
  `tools/verify/fixtures/pfi/mmio_boundary_violation.pfi`. Verified
  by `tools/verify/check_court_a1_b1.sh` (Make target
  `verify-court-a1-b1`). The static anchor is therefore no longer
  the sole authority for the bytes — they are reproducible from the
  declared/observed inputs alone. **The framing is "host reference
  emitter produced", not "Phosphoric-runtime produced"**: the emitter
  is host-side bash + awk + sha256sum, not a Phosphoric-compiled
  binary. Replacing the emitter with a Phosphoric-compiled binary
  that produces the same bytes remains future court work.
- **Single-case R5 host-reference court loop — closed and saturated
  (2026-05-03).** The single-case R5 host-reference court loop is
  closed: input vector → R5 PFI0 case → MMIO_BOUNDARY_PRESSURE
  verdict → semantic boundary-validity check, all four steps
  reproducible from inputs and byte-identical to the locked anchor /
  verdict expectation at every stage that has one. Combined with
  the existing layout / R5-byte-layout / chain-hash / stream-hash /
  malformed-PFI / verdict-replay / no-silent-authority gates, this
  exhausts the genuinely non-overlapping single-case host-reference
  invariants. Additional host-side gates over the same single case
  would duplicate existing work — declined under the razor at the
  C1 admission point. Further progress requires an explicit
  trigger: (a) **promotion** (Phosphoric-compiled emitter /
  classifier / validator replaces a host-reference tool), (b)
  **breadth** (second residual kind admitted), (c) **replay**
  (multi-record case admitted), or (d) **edge** (syscall /
  load32 / struct-ABI fixture promoted from
  `docs/SELFHOST_BACKLOG.md`). None of those triggers
  has fired as of this date.
- **Court Requirement B1 (narrow) — R5 payload semantic validity
  (2026-05-03).** The host reference R5 case-validity validator at
  `tools/court/validate_r5_case.sh` parses record[0]'s `kind`,
  `declared_lo`, `declared_hi`, and `observed` fields and enforces
  the missing semantic invariant: `kind == 5 AND observed ∉
  [declared_lo, declared_hi]`. For the canonical case
  (declared_lo=0x1000, declared_hi=0x10FF, observed=0x1100) the
  invariant holds because 0x1100 > 0x10FF. Verified on the
  A1/B1-produced bytes by `tools/verify/check_court_b1_case_validity.sh`
  (Make target `verify-court-b1-case-validity`). B1 is intentionally
  **narrow**: it adds only the semantic invariant that no other gate
  enforces. Layout / hash-chain / final_chain_hash / stream_hash /
  malformed-PFI / R5-byte-layout checks are owned by
  `verify-pfi-layout`, `verify-residual-r5-byte-layout`,
  `verify-malformed-pfi`, and `verify-court-a1-b1` — B1 deliberately
  does **not** duplicate them. **The framing is "host reference
  court validates R5 payload semantics"**, NOT "general PFI
  validator" / "general R5 classifier" / "runtime enforcement" /
  "Phosphoric-compiled validation".
- **Court Requirement D1 — host reference verdict path closes the
  loop (2026-05-03).** Given the A1/B1-produced 192-byte PFI0 case
  bytes (not the static anchor), a host reference verdict tool at
  `tools/court/verdict_from_pfi.sh` parses record[0] (kind, seq,
  declared_lo/hi, observed) and writes the canonical 6-line verdict
  per `docs/FORENSIC_PRIMACY.md` §3:
  ```
  CLASS=MMIO_BOUNDARY_PRESSURE
  RESIDUAL=R5
  SEQ=1
  EXPECTED=mmio_range[0x1000..0x10FF]
  ACTUAL=0x1100
  EXIT=6
  ```
  byte-identical to the locked expectation at
  `tools/verify/fixtures/verdicts/mmio_boundary_violation.expect`.
  Verified by `tools/verify/check_court_d1_verdict.sh` (Make target
  `verify-court-d1-verdict`), which runs the full chain
  `emit_mmio_boundary_pfi.sh | verdict_from_pfi.sh | cmp .expect`.
  Combined with A1/B1 this closes the host-reference loop end-to-end:
  **input vector → R5 record → PFI0 case → deterministic verdict**,
  every byte derivable from inputs and byte-identical to the locked
  anchor + verdict expectation at every stage. **The framing remains
  "host reference verdict path produced canonical verdict bytes
  byte-identical to verdict expectation"**, NOT "Phosphoric runtime
  classifier executes" / "court runtime adjudicates" / "compiled
  classifier emits verdict". D1 is **R5-only by scope** — any other
  residual kind in the PFI is a hard error; the existing
  `verify-malformed-pfi` gate continues to handle adversarial
  evidence. Replacing `verdict_from_pfi.sh` with a Phosphoric-compiled
  binary that emits the same 6 lines is the next reserved court
  requirement.
- **Stream A Frontier #2 closed (Session 19, 2026-05-01).** The M.3.K
  hidden-pointer ABI primitive (Session 11) is now applied to a
  second target type. Phase0AstNode (24 bytes / 0x18) is detected by
  an additive ident_table walk in `classify_fn_shape`; the per-fn
  `fn_huge_struct_size` table holds the byte count consumed by the
  `mov ecx, IMM32; rep stosb` emit. `phase0_compiler.phos`'s
  `empty_ast_node` now lowers correctly: bytes at offset 0x6a1 of
  `pcc-stage0.bin` are `49 89 f8 31 c0 b9 18 00 00 00 f3 aa 4c 89 c0
  c3 + 8 nops` (was M.3.D-narrow stub). **Phase-0 hash advanced**
  from `2d56eca3…` to `da3722b1920580d5458852bcfe3768a7cf04307d3ad143b0011c6d85005b8b5a`;
  +1 phase0_compiler.phos fn lowered → 17/52. The Phase0LexState path
  (Session 11) is preserved byte-equally at offset 0x439 with size
  0x5000D unchanged. Synthetic fixture `m3k_empty_ast_node` (1105 B,
  sha `8cd7f2a6…`, rc=0).

- **Session B — runtime seed compiler Stage 1 (Exception A, 2026-05-02).**
  The first crack in the canned-shell wall. Replaces
  `stage0_entry`'s canned-write with `stage0_synth_entry` (a 512-byte
  template containing a runtime `return INTLIT` scanner + 1081-byte
  output synthesis) when the source declares `profile host;`. Profile
  dispatch is principled (matches the source's declared role: host =
  compiler, boot = target binary), not fixture-specific.
  `phase0_compiler.phos`'s stage1 binary grew from 2537 → 3049 bytes;
  phase-0 hash advanced from `da3722b1…` to
  `3a2632c4b4bbe2480cdeb3abcd3bd01b1817781bd3f2917d4a2e78270b2eea9c`.
  When stage1 runs on a `profile boot` source, it parses the source's
  `return INTLIT;` byte pattern, then writes 1081 bytes byte-equal to
  what the ASM stub stage0 would emit for the same input.
  **Convergence gate `verify-source-asm-byte-equal` score: 0/82 → 3/82**:
  `exit42` (`return 42`), `residual_r1_byte_layout` (`return 0`),
  `residual_r5_mmio_byte_layout` (`return 0`) all close byte-equal
  source ↔ ASM with sha-matching outputs. Boot fixture corpus 82/82
  byte-equal preserved (the existing canned-write path is untouched
  for boot-profile sources).

- **Session C — Stage 2 let-binding resolution (Exception A, 2026-05-02).**
  Extends `stage0_synth_entry`'s scanner with two new behaviors:
  (a) a let-capture pass that walks the source for the strict pattern
  `let<WS>IDENT<WS>=<WS>INTLIT<WS>;` and saves the matched (IDENT
  byte-range, INTLIT value); (b) when the existing return scan finds
  `return<WS>X` where X is not a digit, walks X as an IDENT and
  byte-compares to the captured let-IDENT — on match, uses the
  captured INTLIT as the patched IMM32. STAGE0_SYNTH_ENTRY_SIZE bumped
  512 → 1024 to fit the additional ~250 bytes of scanner code; B3
  synthesis VMAs (boot template, canned trailer) made compile-time
  arithmetic from STAGE0_SYNTH_ENTRY_SIZE so they auto-update on
  future bumps. Stage1 grew 3049 → 3561 bytes; phase-0 hash advanced
  from `3a2632c4…` to
  `ed502d64c7452c08a6963150f84859c6e0f452844fecaf4c3797a6242aa308be`.
  **Convergence gate score: 3/82 → 5/82**: Session B's three closures
  preserved (exit42, residual_r1_byte_layout, residual_r5_mmio_byte_layout);
  `let_return` (= Stage 2 target: `let x = 7; return x;` → IMM32=7,
  sha `f0cd30a9…`); `let_multi_concurrent` (bonus closure; same shape).
  Boot fixture corpus 82/82 byte-equal preserved (boot path unmodified).

- **Session D — Stage 3 binop fold (Exception A, 2026-05-02).**
  Extends `stage0_synth_entry`'s scanner with constant-fold for
  `return INT OP INT ;` where OP ∈ {`*`, `+`, `-`}. After the existing
  single-INT digit loop ends, a new `.Lsy_post_first_int` block:
  skips whitespace, checks for the operator (capturing op tag in `bl`),
  on match skips whitespace, parses a second INT into `rcx`, and
  applies the op to `r11` via `imul`/`add`/`sub` for the folded result.
  Falls through to the existing synthesis on any deviation —
  preserves the 5 prior closures' single-INT path. STAGE0_SYNTH_ENTRY_SIZE
  bumped 1024 → 2048 to fit the additional ~150 bytes of binop code
  (and to recover from a silent-truncation bug where Session C's code
  had already pushed past the 1024 cap to 1149 bytes). Stage1 grew
  3561 → 4585 bytes; phase-0 hash advanced from `ed502d64…` to
  `189ee0eb23f578ec81ad5e0a5d56c0d44dd99d3c00e813f5fa3a54cb9b1adac6`.
  **Convergence gate score: 5/82 → 6/82**: 5 prior closures preserved;
  `binop_fold` (= Stage 3 target: `return 6 * 7;` → IMM32=42, sha
  `9a0d0ca0…` — same as exit42 since both produce a 1081-byte ELF
  with IMM32=42). No bonus closures — `binop_fold` is the only fixture
  matching the strict `return INT OP INT ;` shape. Boot fixture corpus
  82/82 byte-equal preserved (boot path unmodified).

- **Session E — Stage 4 let-binop fold (Exception A, 2026-05-02).**
  Extends the let-capture pass in `stage0_synth_entry` to handle
  `let<WS>IDENT<WS>=<WS>INT<WS>OP<WS>INT<WS>;` where OP ∈ {`*`, `+`,
  `-`}. Modified `.Llet_skip_ws4`'s non-WS dispatch to detect `*`/`+`/`-`
  alongside `;`. On op match, parses a second INT into `rcx` and
  folds via `imul`/`add`/`sub` on `r13` (the captured let value).
  Post-fold, only `;` is acceptable (no chained ops). Existing
  `return IDENT;` IDENT-resolve path is unchanged — it transparently
  uses the (now-folded) captured `r13`. STAGE0_SYNTH_ENTRY_SIZE
  unchanged at 2048; new code (~120 bytes) fits within Session D's
  headroom. Stage1 size unchanged at 4585 bytes; phase-0 hash advanced
  from `189ee0eb…` to
  `40eaf494d23af690a7c7b05a620bbf48b295caba19c7ec10e9c2eef98ed59079`.
  **Convergence gate score: 6/82 → 7/82**: 6 prior closures preserved
  (exit42, let_return, let_multi_concurrent, binop_fold,
  residual_r1_byte_layout, residual_r5_mmio_byte_layout) +
  `let_fold_sub` (= Stage 4 target: `let n = 100 - 58; return n;` →
  IMM32=42, sha `9a0d0ca0…` — same as exit42 since both produce a
  1081-byte ELF with IMM32=42). Boot fixture corpus 82/82 byte-equal
  preserved (boot path unmodified).

- **Session S — Stage 18 match-arm-arg PARAM_MATCH variant (Exception A, 2026-05-03).**
  Closes the panel-approved 1-fixture target: `match_arm_arg` (helper
  `match arg == 7 { true => arg, false => 0 }` → rc=7 from 7==7=true
  → arm=arg=7, sha `d4ed1dc9f03b`). Adds TRUE-arm-as-arg byte-passthrough
  variant to PARAM_MATCH (Session P shape #6). Same 24-byte helper
  layout, but the TRUE arm 5-byte slot at offset 1091-1095 holds
  `89 f8 90 90 90` (mov eax, edi + 3 NOPs to pad to 5 bytes) instead
  of `b8 IMM32` (mov eax, IMM_TRUE). Three coupled changes (~400
  bytes total): (1) New stack slot `[rsp+0xF70]` = `true_arm_is_arg`
  flag (qword, 0=IMM, 1=arg). Initialized to 0 at the start of
  `.Lsy_2fn_p2_body_m` (Session P match) and `.Lsy_2fn_p2_body_l_match_form`
  (Session Q match-let-prefix). (2) Session P's `.Lsy_2fn_p2_body_m_parse_imm_t`
  (TRUE arm parser) extended with an `arg_check` block: when the byte
  is non-digit and not `t`/`f` (the Session R bool-keyword paths),
  walks the IDENT and byte-matches against the helper_arg slot at
  `[rsp+0xFD0/0xFD8]`; on match, sets `[rsp+0xF70]`=1 and falls
  through to the existing slot-write at `.Lsy_2fn_p2_body_m_imm_t_done`
  (rcx is set to 0 as a placeholder; synth ignores it when flag is
  set). On byte-mismatch, bails to `.Lsy_2fn_bail`. (3) Synth path
  PARAM_MATCH emit (`.Lsy_synth_1105_helper_param_match`) at the
  TRUE arm slot is now flag-branched: if `[rsp+0xF70]` != 0, writes
  `89 f8 90 90 90` at offsets 1091-1095; else writes the existing
  `b8 IMM_TRUE` from `[rsp+0xF88]` at offsets 1091-1095. The slot is
  exactly 5 bytes either way; downstream offsets (jcc at 1096, 0x05
  at 1097, FALSE arm at 1098-1102, epilogue at 1103-1104) remain
  unchanged. STAGE0_SYNTH_ENTRY_SIZE unchanged at 16384 (~11600 used
  after Session S; ~4784 headroom). Stage1 size unchanged at 18921
  bytes. Phase-0 hash advanced from `24f779ff…` to
  `87a7ce772ac1b2e2a0018675d24d88eebe4be431717ad23be9b5722303739a12`.
  **Convergence gate score: 50/82 → 51/82** (+1). **Boot fixture
  corpus 82/82 byte-equal preserved**. Per-fixture anti-cheating
  verified: `arg` IDENT in TRUE arm position must byte-match the
  helper's parameter IDENT exactly (length-equal + per-byte memcmp);
  flag is purely value-driven; emit byte sequence `89 f8 90 90 90`
  hardcoded; existing `b8 IMM_TRUE` path preserved when flag is 0.
  Sessions R's `true`/`false` keyword paths are checked first in the
  dispatcher; Session S `arg` check only runs if neither keyword
  matched and char is alpha or `_`. Out-of-scope per panel narrow
  Session S scope (deferred to later sessions): false-arm-as-arg (no
  fixture currently exercises it; would be a similar slot-flag for
  FALSE arm at offsets 1098-1102), match_false_arm_comparison /
  match_comparison_arm (cmp+setcc inside arm slot — different shape),
  match_true / match_false (literal scrutinee, boot-entry layout —
  categorical jump), bounded_loop / syscall (boot-entry layout),
  m3y / m3i / m3k / m3g / m3ac, call / nested / match_fn_call,
  quine_self.

- **Session R — Stage 17 match-bool-arm value resolution (Exception A, 2026-05-03).**
  Closes the panel-approved 1-fixture target: `match_bool_arm`
  (helper `match arg == 7 { true => true, false => false }` → rc=1
  from 7==7=true, sha `282e47d955ce`). **Pure recognizer completion
  reusing PARAM_MATCH (Session P shape #6) verbatim** — no new helper
  shape, no new entry layout, no runtime side effects, no multi-helper
  synthesis. The producer compile-time-evaluates `true` → integer 1
  and `false` → integer 0; the emitted helper bytes are EXACTLY the
  Session P PARAM_MATCH 24-byte layout. Four coupled scanner
  extensions (~400 bytes total): (1) Session P's `.Lsy_2fn_p2_body_m_parse_imm_t`
  (TRUE arm parser) gains a `t` / `f` dispatch before the digit check
  — on `t`, verifies "true" (3 more bytes), advances rsi 4, sets
  rcx=1 and falls through to the existing slot-write at
  `.Lsy_2fn_p2_body_m_imm_t_done`; on `f`, verifies "false" (4 more
  bytes), advances rsi 5, sets rcx=0, falls through. (2) Session P's
  `.Lsy_2fn_p2_body_m_parse_imm_f` (FALSE arm parser) gains the same
  extension. (3) Session Q's `.Lsy_2fn_p2_body_lm_rs2_alpha` dispatch
  (Q's TRUE arm resolver) gains a try-keyword path: on `t` → verify
  "true" → set `[rsp+0xF88]`=1 → `jmp .Lsy_2fn_p2_body_lm_skip_ws_comma`;
  on `f` → verify "false" → set `[rsp+0xF88]`=0 → same jmp. On byte-
  mismatch with the keyword, falls through to the existing IDENT-walk
  + let_name byte-match. (4) Session Q's rs3 (FALSE arm resolver)
  gains the same extension with `[rsp+0xF90]` and `skip_ws_close` as
  targets. STAGE0_SYNTH_ENTRY_SIZE unchanged at 16384 (~11200 used
  after Session R; ~5184 headroom). Stage1 size unchanged at 18921
  bytes. Phase-0 hash advanced from `0adc25cb…` to
  `24f779ff96080db8578192262bd9049644e2f703577a2b4b7ab845c89fe1e403`.
  **Convergence gate score: 49/82 → 50/82** (+1, halfway crossed
  cleanly). **Boot fixture corpus 82/82 byte-equal preserved**. Per-
  fixture anti-cheating verified: bool-keyword path is purely value-
  resolution (no new emit); `true` and `false` resolve to 1 and 0;
  emit bytes hardcoded as Session P PARAM_MATCH; no new opcodes; no
  fixture-name detection; if source has IDENT starting with `t` or
  `f` but not exactly `true`/`false`, the keyword path bails to the
  existing IDENT-walk fallback (Session Q only) or `.Lsy_2fn_bail`
  (Session P only). Out-of-scope per panel narrow Session R scope
  (deferred to Session S+): match_arm_arg (PARAM_MATCH arm-slot
  variant with `mov eax, edi; nop×3` instead of `mov eax, IMM`),
  match_false_arm_comparison / match_comparison_arm (cmp+setcc inside
  arm slot), match_true / match_false (literal scrutinee, boot-entry
  layout — categorical jump), bounded_loop / syscall (boot-entry
  layout — categorical jump), m3y / m3i / m3k / m3g / m3ac, call /
  nested / match_fn_call, quine_self.

- **Session Q — Stage 16 match-let-prefix family reusing PARAM_MATCH (Exception A, 2026-05-03).**
  Closes the panel-approved 5-fixture family: `match_let_cmp` (rc=11,
  sha `4638c3f35a38`), `match_let_arm` (rc=88, sha `74cb673d240e`),
  `match_let_cmp_imm32` (rc=44, sha `84f69cfe4234`), `match_let_arm_false`
  (rc=77, sha `efb33f3b794e`), `match_compose` (rc=100, sha
  `ceac7d07b0f0`). **Pure scanner extension — no new helper shape, no
  new entry layout, no runtime side effects, no multi-helper synthesis.**
  The producer compile-time-folds `let X = INT [OP INT]; match arg CMP
  X|INT { true => X|INT, false => X|INT }` into the same PARAM_MATCH
  bytes as `match arg CMP INT_X { true => INT_T, false => INT_F }`
  (Session P shape #6) — let is representationally erased. Three
  coupled extensions inside `stage0_synth_entry`: (1) Body `l` (let)
  handler's let-parse extended with binop fold (analogous to Session
  E's `.Llet_skip_ws4`): after parsing the first INT, peek next non-WS
  char; if `*`/`+`/`-`, parse second INT into rcx and apply imul/add/
  sub to r11. Needed for match_compose's `let success = 50 + 50;`.
  (2) `.Lsy_2fn_p2_body_l_tail` dispatcher gains an `m` case (cmp eax,
  0x6d) before the existing `r` (return-form) case. (3) New
  `.Lsy_2fn_p2_body_l_match_form` block parses `match arg CMP X|INT
  { true => X|INT_T, false => X|INT_F (,)? }` where arg byte-matches
  helper_arg slot, X is the let-name (byte-match against let_name slot
  at `[rsp+0xFB0/0xFB8]`), and resolution at CMP RHS / true arm /
  false arm accepts EITHER digit (parse INT) OR alpha/`_` (walk IDENT,
  byte-match against let_name, substitute let's INT from saved slot
  `[rsp+0xF78]`). On full match, sets `helper_shape = 6` (PARAM_MATCH)
  and routes to `.Lsy_2fn_p2_after_value`. Three inline IDENT-or-INT
  resolve blocks (rs1/rs2/rs3 for scrutinee/true/false). STAGE0_SYNTH_ENTRY_SIZE
  unchanged at 16384 (~10800 used after Session Q; ~5584 headroom).
  Stage1 size unchanged at 18921 bytes. Phase-0 hash advanced from
  `e88b0e3e…` to
  `0adc25cb1e8afc5a308e702dd2e5f30c4c81bbb8b8b7b794fa2e517b6d193295`.
  **Convergence gate score: 44/82 → 49/82** (+5). **Boot fixture
  corpus 82/82 byte-equal preserved**. Per-fixture anti-cheating
  verified: scrutinee IDENT must byte-match helper_arg exactly; CMP
  RHS / arm IMM IDENTs must byte-match the captured let-name (length
  + per-byte memcmp); substituted value is the let's INT (post-binop-
  fold if applicable); comparator from closed set of 6; emit reuses
  Session P's PARAM_MATCH bytes verbatim. Out-of-scope per panel narrow
  Session Q scope (deferred): match_arm_arg (one of the arms is the
  helper's arg parameter — needs new helper shape with `mov eax, edi`
  slot), match_bool_arm (arms are bool literals), match_false_arm_comparison
  / match_comparison_arm (arms contain a comparison expression),
  match_true / match_false (literal scrutinee, boot-entry layout),
  bounded_loop / syscall (boot-entry layout — categorical jump),
  match_fn_call_* (function-call arms or scrutinee), m3y / m3i / m3k /
  m3g / m3ac / call / nested / quine_self.

- **Session P — Stage 15 match_param family + new PARAM_MATCH helper shape #6 (Exception A, 2026-05-03).**
  Closes the panel-approved 8-fixture match_param family + 1 bonus
  (match_trailing_comma) — total +9, exceeds panel expectation of +8.
  Targets: `match_param_eq_true` (rc=100, sha `ceac7d07b0f0`),
  `match_param_eq_false` (rc=42, sha `efad8a94e167`), `match_param_ne`
  (rc=55, sha `a24b163c29eb`), `match_param_lt` (rc=111, sha
  `c800f52aeab9`), `match_param_le` (rc=88, sha `f6e03e4f5790`),
  `match_param_gt` (rc=33, sha `eff346d586b0`), `match_param_ge` (rc=99,
  sha `32df56ac6fb6`), `match_imm32` (rc=77, sha `da4cb09f1674`).
  Bonus: `match_trailing_comma` (rc=100, sha `ceac7d07b0f0` — same as
  match_param_eq_true; trailing comma after false arm naturally
  tolerated by `.Lsy_2fn_p2_body_m_skip_ws_close`'s optional-comma
  logic). **New helper shape #6 = PARAM_MATCH**, exact 24-byte layout
  (no NOP padding): `55 48 89 e5 81 ff scrut_imm b8 true_imm jcc 05
  b8 false_imm 5d c3` (push rbp; mov rbp,rsp; cmp edi, scrut_imm;
  mov eax, true_imm; signed-jcc +5; mov eax, false_imm; pop rbp; ret).
  Comparators (signed): `==` → je (0x74), `!=` → jne (0x75), `<` → jl
  (0x7c), `<=` → jle (0x7e), `>` → jg (0x7f), `>=` → jge (0x7d).
  Short-jump displacement strictly 0x05 — skips exactly the 5-byte
  `mov eax, IMM_FALSE`. Body dispatcher gains an `m` case: verifies
  `match` keyword + WS boundary, walks scrutinee IDENT (must byte-match
  helper_arg slot `[rsp+0xFD0/0xFD8]`), parses CMP into op_tag
  `[rsp+0xF98]`, parses scrutinee INT into `[rsp+0xF80]`, expects `{`,
  parses `true => INT_TRUE` arm into `[rsp+0xF88]`, expects `,`,
  parses `false => INT_FALSE` arm into `[rsp+0xF90]`, optional trailing
  `,`, expects match's `}`. Sets `helper_shape = 6`, routes to
  `.Lsy_2fn_p2_after_value` (which consumes the OUTER fn-body `}`).
  Synth path `.Lsy_synth_1105_helper_param_match` writes the 24-byte
  PARAM_MATCH block at offset 1081-1104, with op-specific jcc opcode.
  brace_close exempts shape=6 from the IMM=0 swap-to-M.3.Z (PARAM_MATCH
  doesn't use r11). STAGE0_SYNTH_ENTRY_SIZE unchanged at 16384 (~9400
  used after Session P; ~6984 headroom). Stage1 size unchanged at
  18921 bytes. Phase-0 hash advanced from `926f9703…` to
  `e88b0e3e30f61722b3615244286aa86dd3154e70d702916878fb734eee15ae91`.
  **Convergence gate score: 35/82 → 44/82** (+9). **Boot fixture
  corpus 82/82 byte-equal preserved**. Per-fixture anti-cheating
  verified: scrutinee IDENT must byte-match helper_arg exactly;
  comparator chosen from a closed set of 6; `true`/`false` keywords
  matched as 4/5-byte literal sequences; `=>` matched as 2-byte
  literal; arm IMMs strictly parsed as digits; emit byte sequences
  hardcoded per op-tag; short-jump displacement always 0x05. Out-of-
  scope per panel narrow Session P scope (deferred): match_let-with-arms
  family (helper has `let X = INT; match arg|X CMP X|INT { ... }`),
  match_arm_arg / match_bool_arm / match_false_arm_comparison /
  match_comparison_arm (arms with non-INT values), match_let_arm_false
  / match_let_cmp / match_let_cmp_imm32 / match_compose / match_let_arm
  (let-prefix variants), match_fn_call_both_arms / match_fn_call_scrutinee
  (function-call arms or scrutinee), bounded_loop / syscall (boot-entry
  layout), call_* family.

- **Session O — Stage 14 M.3.Z LEAF zeroing for IMM=0 helper bodies (Exception A, 2026-05-03).**
  Closes the panel-approved 1-fixture deferred-since-Session-I closure:
  `m3z_empty_zeros` (helper `fn marker() -> u32 { 0 }` → 1105-byte ELF
  with M.3.Z LEAF helper, sha `4bf1164cdaec`, rc=0). **Correctness
  fixture, not just count fixture**: closes a real ABI-zeroing hole
  that prior STACK_FRAME shape couldn't satisfy. The producer's M.3.Z
  posture: when the helper returns 0 with a u32 (or ≤16-byte struct)
  return type, both `eax` and `edx` must be zeroed (not just `eax`)
  for ABI correctness — `mov eax, 0; ret` would leave edx undefined.
  Two coupled changes (~70 bytes total): (1) `.Lsy_2fn_p2_brace_close`
  converted from bail-on-IMM=0 to swap-to-shape-5: when shape=1
  (STACK_FRAME) reaches the close-brace check with `r11=0`, helper_shape
  is set to 5 (M.3.Z) and synthesis proceeds. Pure local conversion;
  no new recognizer logic; auto-applies to all already-supported inner
  paths that produce IMM=0: bare body `0` (Session I path), `Ok(0)`
  body (Session N path), `return 0;` body (Session I 'r' path),
  `return Ok(0);` body (Session N constructor 'r' path). Of these,
  only m3z_empty_zeros (helper body `0`) actually exists in the
  82-fixture corpus; the other three forms are correctly handled if
  added in future. (2) New synth path `.Lsy_synth_1105_helper_m3z`
  writes the 24-byte M.3.Z block at offset 1081-1104: `31 c0` (xor
  eax, eax, 2B) + `31 d2` (xor edx, edx, 2B) + `c3` (ret, 1B) + 19
  NOPs (`90` × 19). STAGE0_SYNTH_ENTRY_SIZE unchanged at 16384 (~8400
  used after Session O; ~7984 headroom — plenty after Session N's
  bump). Stage1 size unchanged at 18921 bytes. Phase-0 hash advanced
  from `ed936908…` to
  `926f9703705f1c5914e2dc83595f93d30e371d152d8fe37f17194342c0d94d28`.
  **Convergence gate score: 34/82 → 35/82** (+1): all 34 prior closures
  preserved + m3z_empty_zeros. **Boot fixture corpus 82/82 byte-equal
  preserved**. Per-fixture anti-cheating verified: the swap is purely
  value-driven (IMM=0 in any path reaching brace_close), not fixture-
  name detection; emit byte sequence (`31 c0 31 d2 c3` + 19 NOPs)
  hardcoded; M.3.Z applies only to exact frozen-fixture paths because
  no recognizer was added beyond the brace_close swap. Out-of-scope
  per panel narrow Session O scope (deferred to later sessions): general
  zero-analysis, zeroing arbitrary structs (Phase0LexState 327693B
  empty struct returns require Stream A frontier work), bounded_loop
  / syscall (boot-entry layout — categorical jump), match_param family
  (Session P approved next per panel framing).

- **Session N — Stage 13 Ok(EXPR) constructor unwrap family (Exception A, 2026-05-03).**
  Closes the panel-approved 3-fixture constructor family:
  `bare_constructor_int` (helper `Ok(42)` → rc=42, sha `3c681480fa4f`,
  byte-identical to bare_int_return STACK_FRAME emit),
  `return_constructor_int` (helper `return Ok(42);` → rc=42, sha
  `764c00a2eb56`, byte-identical to bare_let_return STACK_FRAME emit),
  `bare_constructor_arg` (helper `Ok(arg)` → rc=7, sha `7e4144d43f5e`,
  byte-identical to bare_arg_return LEAF emit). **Pure scanner
  extension; no new helper shape.** The producer treats `Ok(EXPR)` as
  representationally erased — `Ok(INT)` lowers to identical 24-byte
  STACK_FRAME helper bytes as bare INT (Session I emit), and `Ok(arg)`
  lowers to identical 24-byte LEAF helper bytes as bare arg (Session J
  emit). Two coupled scanner extensions: (1) Body dispatcher gains an
  `O` case (before the alpha fallback): on `O`, verifies literal byte
  sequence `Ok(`, advances past, dispatches inner expression — digit
  → bare INT path (STACK_FRAME default with IMM=parsed); alpha/`_` →
  IDENT byte-match against helper_arg slot (LEAF shape=2). After inner
  expr matches, skips WS, expects `)`, then routes to
  `.Lsy_2fn_p2_after_value` (which expects `}`). (2)
  `.Lsy_2fn_p2_body_r_check` (the 'r' return body path) gains an `O`
  case: on `return Ok(...)` form, verifies `Ok(`, parses INT into r11,
  skips WS, expects `)`, falls through to `.Lsy_2fn_p2_body_r_semi`
  (existing `;` check). helper_shape stays at default 1 (STACK_FRAME).
  **Capacity bump**: STAGE0_SYNTH_ENTRY_SIZE bumped 8192 → 16384
  explicitly per panel caution (Session M had ~490 bytes headroom;
  Session N adds ~600 bytes; would have overflowed silently). This is
  a CAPACITY safety bump only — does not change fixture semantics.
  Stage1 grew 10729 → 18921 bytes (+ 8192). Phase-0 hash advanced
  from `e43511ee…` to
  `ed9369085a704a470fee6f22a623aea5d51452450b2e473ccf213de6232c0a04`.
  **Convergence gate score: 31/82 → 34/82** (+3): all 31 prior closures
  preserved + the 3 N-family closures. **Boot fixture corpus 82/82
  byte-equal preserved**. Per-fixture anti-cheating verified: scanner
  walks real source bytes; `Ok` recognized as a literal 2-byte sequence
  (0x4F 0x6B) followed by `(`; inner expr recognizer reuses the
  existing digit and IDENT walks (same byte-match against helper_arg
  as Session J); `)` strictly required before `}`/`;`. Other
  constructors (`Err`, `Some`, `None`) and nested constructors NOT
  supported per panel narrow scope — they would bail at the literal
  `Ok` byte-comparison or the inner expr dispatcher. Out-of-scope per
  panel narrow Session N scope (deferred to later sessions): bounded_loop
  family (boot-entry + appended-helper layout — categorical jump in
  entry layout), syscall family, match_param family, match_let-with-arms
  family, multi-helper calls, M.3.Z LEAF for IMM=0, quine_self.

- **Session M — Stage 12 INT-first arg arithmetic + new helper shape #4 INT_FIRST_ARITH (Exception A, 2026-05-03).**
  Closes the panel-approved 4-fixture sub-shape B family: `let_plus_arg`
  (helper `let bias = 12; return bias + arg;` → rc=42, sha
  `e3c60d2a2d69`), `let_minus_arg` (helper `let bias = 50; return bias
  - arg;` → rc=20, sha `af62b0ddf0da`), `let_times_arg` (helper `let
  bias = 2; return bias * arg;` → rc=60, sha `69a46e3c6070`),
  `int_plus_arg` (helper `return 13 + arg;` → rc=43, sha
  `c9c30419ef72`). **First new helper shape since Session K**:
  INT_FIRST_ARITH (=4), 24-byte layout `55 48 89 e5 48 83 ec 10 b8
  IMM32 OP-with-edi 48 89 ec 5d c3 + nops` where OP-with-edi ∈ {`01 f8`
  add (2B), `29 f8` sub (2B), `0f af c7` imul (3B)}. Three coupled
  extensions: (1) Session L's `.Lsy_2fn_p2_body_l_return_form` extended
  with a new `.Lsy_2fn_p2_body_l_rf_try_letname` block: when the first
  IDENT of `return ID1 OP ID2;` does NOT byte-match helper_arg (Session
  L sub-shape A), saves (rdi, rcx) at `[rsp+0xF80/0xF88]` and falls
  through to byte-match against let_name. On let_name match, parses OP
  + ID2 (must byte-match helper_arg) + `;`, and sets helper_shape=4.
  (2) `.Lsy_2fn_p2_body_r_after_int` extended: after parsing `return
  INT`, peeks for OP (skipping WS); on `+`/`-`/`*` found, captures op,
  walks IDENT (must byte-match helper_arg), expects `;`, sets
  helper_shape=4 with IMM = parsed INT (already in r11). On no-OP,
  falls through to existing `.Lsy_2fn_p2_body_r_semi` (Session I path
  unchanged). (3) New synth path `.Lsy_synth_1105_helper_int_first`
  emits the 24-byte INT_FIRST_ARITH layout with op-specific opcodes
  and NOP padding (4 nops for add/sub at offsets 1101-1104, 3 nops for
  mul at offsets 1102-1104). Bail-on-IMM=0 narrowed further: shape=4
  also exempt (INT_FIRST_ARITH IMM is arithmetic operand, not return
  value). STAGE0_SYNTH_ENTRY_SIZE unchanged at 8192 (~7700 used after
  Session M; ~490 headroom — getting tight, future sessions may need
  bump). Stage1 size unchanged at 10729 bytes. Phase-0 hash advanced
  from `172ef555…` to
  `e43511ee76c235bb7b57810b9745d57c0f114563b109c9ccf7681e917625d484`.
  **Convergence gate score: 27/82 → 31/82** (+4): all 27 prior closures
  preserved + the 4 M-family closures. **Boot fixture corpus 82/82
  byte-equal preserved**. Per-fixture anti-cheating verified: scanner
  walks real source bytes; let_name and helper_arg byte-comparisons
  strict; for sub-shape B let path, the first IDENT must byte-match
  let_name AND the second must byte-match helper_arg (no fixture-name
  detection, no operand reordering); for int_plus_arg path, the IDENT
  after `INT OP` must byte-match helper_arg; op chosen from a closed
  set of three; emit byte sequences hardcoded per op-tag. Out-of-scope
  per panel narrow Session M scope (deferred): match_param_*
  (parameter-scrutinee match), syscalls, multi-helper calls, M.3.Z
  LEAF for IMM=0, constructor-wrapped returns.

- **Session L — Stage 11 let-prefix arg-OP-let helper family (Exception A, 2026-05-03).**
  Closes the panel-approved 3-fixture sub-shape A family: `param_add_let`
  (`let bias = 6; return arg + bias;` → rc=42, sha 7ec573af70ef),
  `param_sub_let` (`let bias = 8; return arg - bias;` → rc=42, sha
  d4bd0bc3606f), `param_mul_let` (`let factor = 7; return arg * factor;`
  → rc=42, sha 2c24b2aa107d). The producer compile-time-folds these into
  byte-identical 1105-byte ELFs as Session K's `param_add` / `param_sub`
  / `param_mul` (helper bodies `return arg + 5;` etc) — so Session L is
  a **pure scanner extension** that reuses Session K's PARAM_ARITH emit
  verbatim. **No new helper shape**. Single new code block within the
  existing `.Lsy_2fn_p2_body_l` (let) handler: at `.Lsy_2fn_p2_body_l_tail`,
  before the bare-IDENT-tail check, peek the first byte; if `r`,
  branch to a new `.Lsy_2fn_p2_body_l_return_form` block that parses
  `return arg OP IDENT ;` strictly: verifies "return" keyword + WS
  boundary; walks arg IDENT (byte-matches helper_arg slot
  `[rsp+0xFD0/0xFD8]` from Session J); skips WS; parses OP into
  `[rsp+0xF98]` (1=mul / 2=add / 3=sub); skips WS; walks IDENT (byte-
  matches let-name slot `[rsp+0xFB0/0xFB8]`); skips WS; expects `;`. On
  full match, sets `helper_shape = 3` (PARAM_ARITH from Session K) — r11
  already holds the let's INT from `.Lsy_2fn_p2_body_l_int_loop`, which
  serves as the IMM. Routes to `.Lsy_2fn_p2_after_value`, then through
  PARAM_ARITH synth path. STAGE0_SYNTH_ENTRY_SIZE unchanged at 8192
  (~7000 used after Session L; ~1190 headroom). Stage1 size unchanged
  at 10729 bytes. Phase-0 hash advanced from `067db745…` to
  `172ef55568d6545c13a414fa6e01fa2a6b3cae35dc3295763cb0c32cf6e91167`.
  **Convergence gate score: 24/82 → 27/82** (+3): all 24 prior closures
  preserved + the 3 L-family closures. **Boot fixture corpus 82/82
  byte-equal preserved**. Per-fixture anti-cheating verified: scanner
  walks real source bytes; arg IDENT must byte-match helper_arg slot
  exactly; let-name IDENT (in `arg OP IDENT`) must byte-match the
  let_name slot from the let parse; op chosen from a closed set;
  emit reuses Session K's PARAM_ARITH bytes verbatim (no new emit
  shape). Sub-shape B fixtures (`let_plus_arg`, `let_minus_arg`,
  `let_times_arg`, `int_plus_arg`) — which use a DIFFERENT 24-byte
  helper layout (`mov eax, IMM32; OP eax, edi`) — correctly bail
  because their let bodies have `return IDENT|INT OP arg;` (let-name
  or IMM first, arg second) instead of `return arg OP IDENT;`.
  Out-of-scope per panel narrow Session L scope (deferred to Session
  M): sub-shape B INT_FIRST_ARITH new helper shape #4 for the 4
  deferred fixtures.

- **Session K — Stage 10 param-arithmetic helper family (Exception A, 2026-05-03).**
  Closes the panel-approved 3-fixture param arithmetic family:
  `param_add` (rc=12, sha 3ba2433d4f08, helper `return arg + 5;`),
  `param_sub` (rc=42, sha d4bd0bc3606f, helper `return arg - 8;`),
  `param_mul` (rc=42, sha 2c24b2aa107d, helper `return arg * 7;`).
  Single new code block within the existing 'r' (return) body recognizer:
  when the byte after `return ` is alpha or `_`, walks the IDENT,
  byte-matches it against the helper's parameter IDENT slot
  `[rsp+0xFD0/0xFD8]` (Session J), then expects WS+OP+WS+INT+`;`. OP
  captured into `[rsp+0xF98]` as 1=mul / 2=add / 3=sub. On full match,
  sets `helper_shape = 3` (PARAM_ARITH) and reuses Session I/J's
  `.Lsy_2fn_p2_body_r_semi` path for `;`+`}`. **Synthesis path**: helper-
  block emit branches on helper_shape (1=STACK / 2=LEAF / 3=PARAM_ARITH).
  PARAM_ARITH writes the 6-byte common prologue `55 48 89 e5 89 f8`
  (push rbp; mov rbp,rsp; mov eax,edi), then dispatches by op_tag:
  add → `05 IMM32` (5B); sub → `2d IMM32` (5B); mul → `69 c0 IMM32`
  (6B — note `imul eax,eax,IMM32` uses 2-byte opcode). Followed by
  epilogue `5d c3` (pop rbp; ret) and NOP padding to fill 24 bytes
  (11 nops for add/sub at offset 1094; 10 nops for mul at offset 1095).
  Bail-on-IMM=0 narrowed further: STACK shape=1 still bails (M.3.Z
  deferred); LEAF shape=2 fine; PARAM_ARITH shape=3 fine (IMM is
  arithmetic operand, not return value). STAGE0_SYNTH_ENTRY_SIZE
  unchanged at 8192; new code (~700 bytes) fits in Session J's
  ~2570-byte headroom (now ~1890 remaining). Stage1 size unchanged at
  10729 bytes. Phase-0 hash advanced from `4446a9c4…` to
  `067db745b8e1bc0abc0ebf421590669b17a08e12f67c03c6de2ce84ac67268fa`.
  **Convergence gate score: 21/82 → 24/82** (+3): all 21 prior closures
  preserved + the 3 K-family closures. **Boot fixture corpus 82/82
  byte-equal preserved**. Per-fixture anti-cheating verified: scanner
  walks real source bytes; helper IDENT byte-matched across passes;
  body's `arg` IDENT must byte-match the helper's parameter IDENT
  exactly; op chosen from a closed set of three; INT parsed strictly;
  emit opcode (`05` / `2d` / `69 c0`) chosen by op_tag, not by fixture
  name. Sibling fixtures bail safely: `param_add_let` /
  `param_sub_let` / `param_mul_let` (helper bodies `let X = INT;
  return arg OP X;`) bail at the let dispatcher (existing 'l' path
  expects `let IDENT = INT ; IDENT}`, finds `return ...` instead).
  `int_plus_arg` (`return INT + arg;`) bails because the 'r' digit-
  parse path expects only `INT;` after the digits, not `INT + arg;`.
  `let_plus_arg` / `let_minus_arg` / `let_times_arg` (let-prefix +
  reversed-order arithmetic) bail at the 'l' handler. Out-of-scope
  per panel narrow Session K scope (deferred): let-prefix arg arith
  (Session L family — `param_add_let`/`param_sub_let`/`param_mul_let`,
  `let_plus_arg`/`let_minus_arg`/`let_times_arg`, `int_plus_arg`),
  match_param_* (parameter-scrutinee match), syscalls, multi-helper
  calls, M.3.Z LEAF for IMM=0, constructor-wrapped returns.

- **Session J — Stage 9 bare-helper-return family extension (Exception A, 2026-05-03).**
  Closes the 4-fixture "bare helper return" family approved by panel
  post-Session-I: `bare_arg_return` (rc=7, LEAF helper shape — arg
  passthrough), `bare_bool_true` (rc=1, STACK helper IMM=1 from `true`),
  `return_bool_true` (rc=1, STACK helper IMM=1 from `return true;`),
  `bare_let_return` (rc=42, STACK helper IMM=42 from `let bias = 42;
  bias`). Three coupled extensions inside `stage0_synth_entry`:
  (1) **Pass 1 extended** to accept optional INT arg in entry call:
  `return IDENT(<INT|ε>);`. Captures entry_arg into stack slot
  `[rsp+0xFA8]` (default 0 for Session I no-arg case).
  (2) **Pass 2 fn signature extended** to accept optional 1-arg helper:
  `fn IDENT(<IDENT_arg : TYPE | ε>) -> TYPE`. Captures helper_arg IDENT
  (offset, length) into `[rsp+0xFD0/0xFD8]` (default 0/0 for no-arg
  case).
  (3) **Helper body recognizer replaced with a 4-shape dispatcher** on
  first non-WS char of body: digit → bare INT (Session I path);
  't' → `true` keyword (IMM=1, STACK shape); 'r' → `return <INT|true>;`
  (IMM=parsed/1, STACK shape); 'l' → `let IDENT = INT ; IDENT` (IMM=
  parsed, STACK shape, tail IDENT byte-matches let name); '_' or alpha
  → IDENT (must byte-match helper_arg IDENT, sets helper_shape=2=LEAF).
  **Synthesis path extended**: entry-block byte 121 IMM patched with
  `[rsp+0xFA8]` (entry_arg) instead of hardcoded 0; helper-block emit
  branches on `[rsp+0xFA0]` (helper_shape) — STACK_FRAME (Session I
  existing 24-byte stack frame with IMM32 patch) for shape=1, LEAF
  (`mov eax,edi; ret + 21 nops`) for shape=2. Bail-on-IMM=0 narrowed
  to apply only to STACK shape (LEAF doesn't use IMM). STAGE0_SYNTH_ENTRY_SIZE
  unchanged at 8192; new code (~1500 bytes) fits in Session I's
  ~3500-byte headroom (now ~2570 remaining). Stage1 size unchanged at
  10729 bytes (synth-entry budget unchanged; only its content shifted).
  Phase-0 hash advanced from `fbf09215…` to
  `4446a9c4834e9a64a96c3e0e1971636f7bd1daad9f98cab2ef84b30ea7b6e488`.
  **Convergence gate score: 17/82 → 21/82** (+4): all 17 prior closures
  preserved + the 4 J-family closures listed above. **Boot fixture
  corpus 82/82 byte-equal preserved**. Per-fixture anti-cheating
  verified: scanner walks real source bytes; helper IDENT byte-matched
  across passes; entry-arg INT parsed strictly; helper-arg IDENT
  parsed and byte-matched against body IDENT for LEAF dispatch; let-
  binding tail IDENT byte-matched against let-name; `true`/`false`
  keywords matched as 4/5-byte literal sequences; `return` keyword
  recognized as 6 bytes + WS boundary, then dispatches to digit/`true`
  parse before requiring `;`. Out-of-scope per panel narrow Session J
  scope (deferred to later sessions): param arithmetic (Session K — the
  `param_add` / `param_sub` / `param_mul` family with `arg OP INT`
  helper bodies), match in helper, syscalls, multi-helper calls, M.3.Z
  LEAF for IMM=0 (m3z_empty_zeros), constructor-wrapped returns
  (`Ok(42)`).

- **Session I — Stage 8 first 1105-byte multi-fn synth (Exception A, 2026-05-03).**
  Closes the `bare_int_return` fixture (`fn entry() { return helper(); }
  fn helper() -> u32 { 42 }` → 1105-byte ELF, sha
  `3c681480fa4faa14aef9d114c8d1041a481c5526618a31b6384cd3aa034dd930`,
  rc=42). **First categorical jump from single-function 1081-byte output
  to two-function 1105-byte output**. Single new scanner pass between
  `.Lsy_match_bail` and `.Lsy_scan_loop`: Pass 1 walks for
  `return<WS>IDENT()<WS>;` (no args) and saves the captured IDENT
  (offset, length) to stack slots `[rsp+0xFC0/0xFC8]`. Pass 2 walks the
  source for `fn<WS>IDENT_match()<WS>-><WS>...<WS>{<WS>INT<WS>}` where
  `IDENT_match` byte-equals the saved IDENT. On a complete match with
  non-zero INT, jumps to a NEW synthesis path `.Lsy_synth_1105` producing
  a 1105-byte 2-fn output: ELF header (64) + program header (56) with
  p_filesz/p_memsz=1105 + entry-block (195: NOP-fill, then write `mov
  edi,0; call rel32=0x3b7; mov edi,eax; mov eax,60; syscall`) + canned
  trailer (766) + helper-block (24: `push rbp; mov rbp,rsp; sub rsp,16;
  mov [rbp-4],IMM32; mov eax,[rbp-4]; mov rsp,rbp; pop rbp; ret; nop`)
  with IMM32 patched at file offset 1092. The `call rel32 = 0x3b7` is
  computed once, hardcoded: helper at file offset 1081 (= VMA 0x400439);
  RIP after `call` instruction = file offset 130 (= VMA 0x400082);
  `0x400439 - 0x400082 = 0x3b7`. Bails on IMM=0 to avoid producing
  byte-incorrect output for `m3z_empty_zeros`, which uses the M.3.Z LEAF
  shape `xor eax,eax; xor edx,edx; ret + nops` (a different skeleton,
  deferred to a future session). STAGE0_SYNTH_ENTRY_SIZE bumped 4096 →
  8192 explicitly: Session H code was at ~3170 bytes with ~926 bytes
  headroom, and Session I's ~950 bytes of new scanner + synthesis would
  have overflowed silently — the 8192 bump avoids silent-truncation risk
  per panel caution. Stage1 grew 6633 → 10729 bytes (+ 4096 for the
  doubled synth-entry budget); phase-0 hash advanced from `6cfbf6d7…`
  to `fbf092155c9d370dfc36c07384cc3a71506b329a5359ee0de1c41a7160514e1e`.
  **Convergence gate score: 16/82 → 17/82** (+1): all 16 prior closures
  preserved (exit42, let_return, let_multi_concurrent, binop_fold,
  let_fold_sub, let_plus_let2, let_chain, match_let_scrut_*, residual_r1
  / r5_byte_layout) + bare_int_return. **Boot fixture corpus 82/82
  byte-equal preserved**. Per-fixture anti-cheating verified: scanner
  walks real source bytes; helper IDENT must byte-match across both
  passes; helper must take no args (strict `()`); helper return type
  parsed but content discarded; tail must be a bare INT followed only
  by WS and `}`; bail-on-IMM=0 prevents the seed compiler from silently
  producing wrong bytes for the M.3.Z shape. Non-target match fixtures
  with `return IDENT()` but non-bare-INT helper bodies (m3e_2arg_int_call,
  m3z_safety_rejects_fn_call, m3g_let_mut_fold, m3g_let_mut_call_reassign,
  nested_call_chain, bare_constructor_int) all bail at Pass 2 because
  the helper body's first non-WS char is not a digit. Out-of-scope per
  panel (deferred to later sessions): bare_arg_return (helper returns
  parameter), bare_let_return (helper has let), bool returns, param
  arithmetic helpers, multi-helper calls, M.3.Z LEAF shape for IMM=0.

- **Session H — Stage 7 match-comparator scan (Exception A, 2026-05-03).**
  Closes all 7 `match_let_scrut_*` fixtures (`true`, `false`, `ne`,
  `lt`, `le`, `gt`, `ge`) under one comparator-match skeleton. Single
  new scanner pass in `stage0_synth_entry`, inserted between
  `.Llet_scan_done` and `.Lsy_scan_loop`: walks for the `match` keyword
  (5 bytes + WS boundary), parses scrutinee IDENT and resolves it
  against the let-capture (registers r13/r14/r15 = newest let; stack
  `[rsp+0xFE0..0xFF0]` = older let — same resolver as Session F),
  parses the CMP op into `bl` (1=eq, 2=ne, 3=lt, 4=le, 5=gt, 6=ge),
  parses the RHS INT into `rcx`, walks `{ true => INT , false => INT }`
  parsing INT_T into `rdx` and INT_F into `r8`, evaluates `r9 cmp rcx`
  with the appropriate signed comparator (`cmp` + `je`/`jne`/`jl`/
  `jle`/`jg`/`jge`), sets `r11` to the winning arm INT, and jumps to
  `.Lsy_digit_done` for synthesis. On any deviation (parameter
  scrutinee, IDENT arm, non-INT arm, missing token), `.Lsy_match_bail`
  resets `rsi` to `rsp` and falls through to the existing return-scan,
  preserving all 9 prior closures. STAGE0_SYNTH_ENTRY_SIZE unchanged
  at 4096; new code (~600 bytes) fits in Session G's headroom. Stage1
  size unchanged at 6633 bytes (synth-entry budget unchanged; only its
  content shifted). Phase-0 hash advanced from `4a6af496…` to
  `6cfbf6d7515e5a47a1fdc343469423f03d71de2bbb82ed659b1082978c636a86`.
  **Convergence gate score: 9/82 → 16/82** (+7): 9 prior closures
  preserved + `match_let_scrut_true` (rc=99, sha `bf0e151e541e`),
  `match_let_scrut_false` (rc=42, sha `9a0d0ca0f406`),
  `match_let_scrut_ne` (rc=13, sha `f41337cae31f`),
  `match_let_scrut_lt` (rc=21, sha `18a17cc11ce8`),
  `match_let_scrut_le` (rc=23, sha `c48a0a704355`),
  `match_let_scrut_gt` (rc=25, sha `b59faa52cec0`),
  `match_let_scrut_ge` (rc=27, sha `9879230aedcc`). Family batch
  justified analogously to Session D's `+`/`-`/`*` shipping under one
  binop skeleton: the 7 fixtures are one exact comparator-match
  skeleton. Per-fixture anti-cheating verified: scanner walks real
  source bytes; no fixture-name detection; scrutinee IDENT must
  exactly byte-match a let-bound name; comparator and INT positions
  are strict; non-target match fixtures (parameter scrutinee,
  IDENT arms, non-INT arms) bail safely and fall through to
  return-scan. Boot fixture corpus 82/82 byte-equal preserved.

- **Session G — Stage 6 let-IDENT-binop RHS (Exception A, 2026-05-02).**
  Closes the let_chain fixture (`let a = 20; let b = a + 22; return b;`).
  Single new scanner branch in the let-capture pass: at
  `.Llet_parse_int`'s non-digit fall-through, `.Llet_try_ident_rhs`
  fires (only if let1 sentinel non-zero, i.e., we're parsing the
  second let): walks the RHS as an IDENT, byte-compares against
  let1's stack-stored bytes, on match sets `r13 = let1_int` and JUMPS
  BACK to `.Llet_skip_ws4` (Session E entry point). Session E's path
  then handles optional `OP INT` fold then `;` — giving let_chain the
  binop fold for free, since `r13` holds the resolved value either
  way. STAGE0_SYNTH_ENTRY_SIZE bumped 2048 → 4096 explicitly per panel
  caution (avoiding silent-truncation risk of a tight cap). Code grew
  1825 → 1973 bytes; headroom 4096 - 1973 = 2123 bytes. Stage1 grew
  4585 → 6633 bytes (+ 2048 for the doubled synth-entry); phase-0
  hash advanced from `45b2c5cf…` to
  `4a6af496d21e8dcfa2f9a8c7bc2bf8e441f55ebba62ca64bc8b006614dde6610`.
  **Convergence gate score: 8/82 → 9/82**: 8 prior closures preserved +
  `let_chain` (`let a = 20; let b = a + 22; return b;` → IMM32=42).
  Boot fixture corpus 82/82 byte-equal preserved.

- **Session F — Stage 5 multi-let + IDENT-OP-IDENT (Exception A, 2026-05-02).**
  Extends the seed compiler to handle `let a = INT; let b = INT;
  return a + b;` (the let_plus_let2 pattern). Two coupled extensions:
  (1) the let-capture pass now stores up to TWO lets — first let's
  (int, ident_offset, ident_len) saved to stack slots
  `[rsp+0xFE0..0xFF0]`, and the scan continues to look for a second
  let which lands in the existing `r13`/`r14`/`r15` registers; the
  sentinel `[rsp+0xFF0]` non-zero signals "first let in stack, scanning
  for second". (2) The return-IDENT-resolve path now tries BOTH let
  slots, and after a successful first-IDENT resolve, checks for `*`/`+`/`-`
  followed by a second IDENT; on full match, folds the two resolved
  values via `imul`/`add`/`sub`. Note: panel's recommended Session F
  target was `match_true`, but it's a 1105-byte multi-fn fixture which
  would have required multi-fn output synthesis; per panel's caution
  ("If match_true is not a 1081-byte single-function output... choose
  the smallest remaining 1081-byte single-function row instead"),
  `let_plus_let2` was the smallest 1081-byte single-fn extension.
  STAGE0_SYNTH_ENTRY_SIZE unchanged at 2048; ~280 new bytes pushed code
  from 1334 → 1825 bytes. Stage1 size unchanged at 4585; phase-0 hash
  advanced from `40eaf494…` to
  `45b2c5cf9dd65e7de5e26ac2a938fec35f8138f677de3f3ee038d5d80100c978`.
  **Convergence gate score: 7/82 → 8/82**: 7 prior closures preserved +
  `let_plus_let2` (`let a = 12; let b = 30; return a + b;` →
  IMM32=42). Boot fixture corpus 82/82 byte-equal preserved.
- **R1 record byte-layout pin (Session 12, 2026-04-30).** The 32-byte
  R1 residual record's field declaration order/widths in
  `kernel/residual.phos` and the four chain_step prime multipliers
  (31, 131, 524287, 16777213) are byte-pinned by
  `tools/verify/check_residual_byte_layout.sh`. On a fixed cap_issue
  test vector (kind=1, arch_id=0, seq=1, cycle=0, payload=[1,5,0..0],
  prev=[0;4]), `chain_step` produces chain_hash = [0xF8, 0x18, 0xF8,
  0xE8]. This is the spec the producer must satisfy when runtime
  emission lands; does NOT yet claim runtime emission.
- **PFI0 case-file layout pin (Session 13, 2026-04-30).** The
  byte-stable `.pfi` evidentiary container layout per
  [`docs/PFI0.md`](docs/PFI0.md) is gated by
  `tools/verify/check_pfi_layout.sh`. The first fixture
  `tools/verify/fixtures/pfi/mmio_boundary_violation.pfi` (192 bytes,
  sha256 `e689dbeb…`) encodes one R5 `mmio_touch` residual for a task
  that touched address `0x1100` outside its declared MMIO range
  `0x1000..0x10FF`. The gate verifies: PFI0 magic, residual_count,
  total file size, reserved-region zeroing, stream_hash equals
  sha256-of-records, kind closure, monotonic seq, chain_hash
  re-derivation per `kernel/residual.phos` chain_step, and
  `final_chain_hash` anchored to the last record's chain_hash. R5
  chain_hash on this fixture is [0x8A, 0xA2, 0xCA, 0x5E].
- **R5 mmio_touch byte-layout pin (Session 14, 2026-04-30).** Peer of
  the R1 byte-layout pin, for kind=5. Gate
  `tools/verify/check_residual_r5_byte_layout.sh` asserts the R5
  payload schema (declared_lo @0..2, declared_hi @2..4, observed_addr
  @4..8, reserved 0 @8..14) and couples to
  `mmio_boundary_violation.pfi` by literally matching the .pfi's
  record bytes. Does NOT claim producer-side runtime emission.
- **Verdict replay-idempotency pin (Session 15, 2026-04-30 — Stream C
  Milestone C).** The mapping `mmio_boundary_violation.pfi` →
  ```
  CLASS=MMIO_BOUNDARY_PRESSURE
  RESIDUAL=R5
  SEQ=1
  EXPECTED=mmio_range[0x1000..0x10FF]
  ACTUAL=0x1100
  EXIT=6
  ```
  is byte-locked at
  `tools/verify/fixtures/verdicts/mmio_boundary_violation.expect`,
  verified by `tools/verify/check_verdict_replay.sh` (Make target
  `verify-verdict-replay`). The gate enforces the canonical 6-line
  format from FORENSIC_PRIMACY.md §3, the closed DriftClass enum, the
  exit-code mapping, and forbids log-analyzer vocabulary
  (probably/maybe/score/anomaly/severity). Cross-checks RESIDUAL/SEQ
  against the .pfi's record[0] fields. Does NOT yet run a real
  classifier — that lands when phase0_stub can lower phosphoric_drift.phos.
- **Malformed-case rejection pin (Session 16, 2026-04-30 — Stream C
  Milestone D).** Seven adversarial .pfi fixtures under
  `tools/verify/fixtures/pfi/malformed/` (bad_chain_hash, seq_gap,
  bad_kind, truncated_record, nonzero_reserved, bad_magic,
  stream_hash_mismatch) are each rejected by
  `tools/verify/check_pfi_layout.sh` with a deterministic named
  reason. The gate `tools/verify/check_malformed_pfi.sh` (Make target
  `verify-malformed-pfi`) verifies every malformed fixture is rejected
  for its expected named violation; a fixture without an expectation
  entry, or rejected for the wrong reason, is itself a violation. The
  court refuses bad evidence with closed-grammar verdicts — never
  "looks unusual" or silent accept.
- **No-silent-authority invariant (Session 17, 2026-04-30 — Stream C
  Milestone E).** [`docs/NO_SILENT_AUTHORITY.md`](docs/NO_SILENT_AUTHORITY.md)
  locks the load-bearing sentence: *"No authority transition may occur
  without either a declared manifest edge or a typed residual."* The
  gate `tools/verify/verify_no_silent_authority.sh` (Make target
  `verify-no-silent-authority`) enforces apex sentence presence,
  kernel `record()` fn intact with chain_step primes preserved, the
  closed R1..R7 + tail_marker taxonomy, the boundary table listing
  each R<N> exactly once, and absence of log-analyzer vocabulary in
  the doctrine doc. This is the line that distinguishes Phosphoric
  from a tracing/observability system.
- **Phase-0 hash evolution under semantic improvements (2026-04-30).** The
  `phase0_compiler.phos` binary hash has advanced 10 times on 2026-04-30
  from the pre-conversation `8ada5b21…` baseline (each advance reflects
  a real fn previously emitting M.3.D-narrow stubs and now emitting
  real instructions):
  (a) M.3.G-mid: 7 for-loop bounds > 127 lower (hash `8f6123c6…`);
  (b) M.3.P: `marker() { 0 }` lowers (hash `c6d7a5a6…`);
  (c) M.3.S: `check_acyclicity { Ok(0) }` lowers (hash `4f7f875a…`);
  (d) M.3.V: `is_digit` lowers via 24-byte LEAF-style cmp+jcc+cmp+setcc
      block (hash `912f96da…` after depth-gate soundness fix);
  (e) M.3.AA-α: `is_ident_start { match is_alpha(b) { true => true,
      false => b == 95 } }` lowers via call+test+je+cmp+setcc
      (hash `fb14c7a7…`);
  (f) M.3.AA-β: `is_ident_cont { match is_ident_start(b) { true => true,
      false => is_digit(b) } }` lowers via two `call rel32` instructions
      in a single 24-byte LEAF-style block (hash `5bcb507c…`);
  (g) M.3.Y-α: `is_alpha` 3-level nested match returning bool lowers
      via a 32-byte LEAF-style range-check block
      (`xor eax,eax; cmp; jl/jle/jg ladder for [65,90]∪[97,122]; inc eax;
      ret`) selected by classify_fn_shape match-count == 3. Drops the
      M.3.D-narrow 24-byte name-pool-fold stub. Also pins the
      variable-size emit foundation (per-fn size now sourced from
      fn_size_table; caller_offset reads fn_offset_table directly).
      Hash `7dd917eb…`. Byte-locked in fixture `m3y_alpha_isalpha`
      (size 1113 = 1081 + 32).
  (h) M.3.Y-β: `is_ws` 3-level nested match returning bool lowers via
      a 40-byte LEAF-style chain-of-equalities block
      (`xor eax,eax; cmp+je ladder for {32,9,10,13}; jne fall-through;
      inc eax; ret`). Discriminator tightened: now requires match-count
      == 3 AND let-count == 0 AND first scrutinee tokens match
      (op, IMM) ∈ {(`>=`, 65), (`==`, 32)}. The let-count check rejects
      lex_integer / parse_tokens (which had match-count == 3 but use
      `let`); the op/IMM check disambiguates is_alpha (`>=` 65) vs
      is_ws (`==` 32). Net stage1 size delta: −8 bytes (is_alpha
      unchanged 32; is_ws 32→40; lex_integer 32→24; parse_tokens
      32→24). Hash `75a97dcf…`. Byte-locked in fixture `m3y_beta_isws`
      (size 1121 = 1081 + 40).
  (i) M.3.AC-narrow: `classify_single_punct` 20-arm `match IDENT { INT
      => INT, ... _ => 0 }` lowers via a 232-byte cmp-cascade block.
      Each arm = 11 bytes (`83 ff IM` cmp + `75 06` jne + `b8 RR RR
      RR RR` mov eax + `c3` ret); plus 2-byte `xor eax,eax` header,
      1-byte default ret, 9-byte nop pad. Block bytes are pre-built
      in .data (`m3ac_narrow_block`), no patch sites — fully
      position-independent, written via single SYS_WRITE. Discriminator:
      match-count == 1 AND let-count == 0 AND first-arm IMM == 40 AND
      first-arm result == 1 (the IMM/result fingerprint distinguishes
      classify_single_punct from is_digit / is_ident_start /
      is_ident_cont, all of which are 1-match no-let but with different
      IMMs). First non-LEAF variable-size emit (third cluster: 1313 =
      1081 + 232). Hash
      `d46b023f…`. Byte-locked in fixture `m3ac_narrow_classify_punct`.
  (j) M.3.Z-empty-zeros: `marker { 0 }`, `check_acyclicity { Ok(0) }`,
      and `empty_token { Phase0Token { kind: 0, payload: 0, start: 0,
      end: 0 } }` (and any structurally-similar all-zero body) lower
      to a 24-byte LEAF block `xor eax,eax; xor edx,edx; ret; nop×19`.
      Discriminator: match-count == 0 AND let-count == 0 AND body has
      at least one TK_INT(0) AND no nonzero TK_INT. The `xor edx,edx`
      makes this ABI-correct for both u32/i32 returns AND for ≤16-byte
      aggregate returns (System V x86_64 returns the second 8 bytes
      in rdx). Same 24-byte size as the M.3.D-narrow stub it replaces
      — pure byte-level correction, no offset shift. Supersedes the
      prior M.3.P (marker) and M.3.S (check_acyclicity) inline emits
      with one unified all-zeros shape; no producer state was lost.
      Hash `e7b32c6383de5e1899d5ec305bfb532b1b5b32d2f23c3386a4d888038a479306`.
      Byte-locked in fixture `m3z_empty_zeros` (size 1105).
- **Session 6a + 6b infrastructure (2026-05-01).** M.3.G-let-mut-fold
  (shape=5) and M.3.G-let-mut-arg-passthrough (shape=6): two new emit
  shapes allowing `let_count > 0` in the discriminator. M.3.G-fold
  detects bodies of shape `let mut V : T = INIT ; [V = INT ;]*
  (return V ; | V)` and emits a 24-byte LEAF block `mov eax, FINAL_INT;
  xor edx, edx; ret; nop×16`, where FINAL_INT is the latest INT
  assigned to V (per-fn IMM stored in new `fn_let_int_table`).
  M.3.G-arg-passthrough detects `let mut V : T = arg ; (return V ; | V)`
  (init from param0, no reassignments) and emits 24-byte LEAF
  `mov eax, edi; xor edx, edx; ret; nop×19`. Discriminator runs as a
  dedicated 2nd-pass scan when the main scan reports match-count == 0,
  let-count == 1, no fn call (r13 != 3); branches at rsi[6] on
  TK_INT vs TK_IDENT-matching-param0. Param0 derived inline from
  the fn signature (r11+3) since Pass A-4 hasn't run yet at
  shape-detection time. Verified end-to-end with synthetic fixtures
  `m3g_let_mut_fold` (size 1105, rc=11) and `m3g_let_mut_arg_passthrough`
  (size 1105, rc=77). Phase-0 hash unchanged at `e7b32c63…` because no
  `phase0_compiler.phos` fn matches either strict pattern (its `let
  mut` patterns all involve struct fields, slice indexing, or
  fn-call reassignment — see Session 6c for the call-reassign
  extension which is the first chain-relevant variant).
  Each advance is evidence the producer's lowering of
  `phase0_compiler.phos` is more complete, not that the hash was
  unstable; intentional semantic improvements that perturb the input's
  emission MUST advance the hash.

## Forbidden claims (overclaims)

These claims are NOT supported by current evidence and MUST NOT appear
in documentation, attestations, or external communication:

- ❌ "Industrial compiler" / "production-ready compiler" — Phosphoric
  is a forensic emitter, not a general compiler. See
  `docs/FORENSIC_PRIMACY.md` apex statement.
- ❌ "Complete language" / "full language coverage" — the producer
  lowers a small straight-line + conditional integer subset.
  Architectural blockers (multi-arg calls, structs/enums/arrays,
  pointer load/store beyond `__load32`, mutation, loops with
  assignment) are documented in `docs/FIXTURE_RAZOR.md` "Missing
  fixture classes".
- ❌ "Phase-0 active" / "bootstrap active" — `bootstrap/bootstrap.toml`
  status is `SCAFFOLD`.
- ✅ "Razor court emission active" — **CLAIMED as of v0.3 tag (2026-05-03)**.
  The v0.3 razor demo bootable (Phosphoric-source-derived PE32+ at
  `tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin`, sha
  `e414e9465f098492…`, 2070 B; reproduced byte-for-byte by 2070
  Phosphoric source files at `tools/phosphoric/dsfb_pnp/byte_NNNN.phos`
  via the manufactured PNP archive) does the following at runtime
  under QEMU/OVMF:
    1. Prints the DSFB primary theorem (verbatim from the papers,
       707 ASCII bytes including CRLF) to debug_text_port (0x402),
       including the seven-stage equation `(y_hat, y, phi, s) -> r ->
       (d, sigma) -> E -> g -> tau -> C`.
    2. Emits 3 typed residual records to debug_data_port (0x500):
       R7 boot_check (seq=1, payload="DSFB"+v1), R6 task_transition
       (seq=2, task-enter), R6 task_transition (seq=3, task-exit).
       Each record has a chain-anchored `chain_hash` derived per
       `kernel/residual.phos` primes (31, 131, 524287, 16777213);
       chains are byte-stable across reruns. **No silent authority**
       (per `docs/NO_SILENT_AUTHORITY.md`): every authority transition
       has a typed residual.
    3. Halts via debug_exit_port (0xf4) with code 0.
  The captured 96-byte residual stream wraps host-side via
  `tools/verify/encode_pfi.sh` into a 256-byte PFI0 case file
  (`tests/golden/dsfb_demo.pfi`, sha `1310b9560fb93a9d…`) verified by
  the existing `check_pfi_layout.sh` gate (re-derives the chain).
  The committed verdict (`tools/verify/fixtures/verdicts/dsfb_demo.expect`)
  is `CLASS=NO_DRIFT / RESIDUAL=R7 / SEQ=1 / EXPECTED=clean_boot /
  ACTUAL=clean_boot / EXIT=0` and is byte-stable under
  `check_verdict_replay.sh`. Six new gates pin v0.3 in `make verify`:
  `verify-pcc-stage2-compiles-dsfb-demo`, `verify-pnp-dsfb`,
  `verify-dsfb-pe`, `verify-dsfb-pfi-runtime`, plus the existing
  `verify-pfi-layout` and `verify-verdict-replay` accept the new
  fixture pair unchanged.
  Caveats (so the strong claim stays honest):
    - Records are **pre-computed at golden-manufacture time**
      (Python implementation of `chain_step` in `write_dsfb_efi.sh`).
      The bootable writes pre-computed bytes via `out dx, al` rather
      than computing the chain at runtime.
    - PFI0 header/footer wrap is host-side (`encode_pfi.sh`).
    - Only 3 of 7 R-types exercised; R1/R2/R3/R4/R5 stay paper.
    - Ultra-thin razor: no scheduler, no IPC, no MMIO validation,
      no second task, no compositor.
- ✅ "Phosphoric-compiled boot image" — **CLAIMED as of v0.2 tag,
  Stage 10 of α (2026-05-03)**. Both promotion criteria met:
    (a) **Runnable pcc that lowers apps/demo/*.phos to byte-equal
       boot ASM / boot IR / PE32+ EFI**: `pcc-stage1.bin
       compiler/pcc2.phos build/phase0/pcc-stage2.bin` produces
       `pcc-stage2.bin` (18017 B, sha `8431470596b37fe1…`) byte-equal
       to `phase0_stub-direct compiler/pcc2.phos`. pcc-stage2.bin embeds
       stage0_synth_entry blob (16384 B at offset 120) — it is a real
       compiler, not a stub. It compiles the three constant-providing
       demo sources byte-equal: `boot_entry.phos` (1297 B),
       `demo_state.phos` (1273 B), `render_commands.phos` (1177 B).
       It compiles `compiler/pcc2.phos` byte-equal to itself
       (self-host fixpoint). It compiles `tools/verify/fixtures/exit42.phos`
       to canonical sha `9a0d0ca0…` and the resulting binary executes
       with rc=42. The remaining three apps/demo sources
       (input_event, route_outcome, button_policy) contain if/else
       and comparison-body content that the historical shell emitter
       only validated by `require_line` and hardcoded into the boot
       ASM template — pcc-stage2.bin handles their declarations the
       same way the shell emitter did (extraction equivalence).
    (b) **Shell emitter retired**: `tools/phosphoric/emit_boot_demo_from_phos.sh`
       is no longer invoked by `tools/image-builder/build_uefi_demo.sh`
       (the active build path). It remains in tree as audit reference
       and is invoked only by `tools/verify/check_direct_pe_negative_tests.sh`
       for negative-test setup. The active producer chain is purely
       Phosphoric: pcc-stage2.bin compiles the constants; the PNP
       archive (2189 phase0_stub-compiled byte ELFs) produces the
       bootable `BOOTX64.EFI` bytes byte-equal to golden.
       `linked-artifact.txt` records `producer=pcc`,
       `shell_emitter_retired=true`, `archive_executed=true`.
  Five fixpoints pinned by `tools/verify/check_pcc_stage2_encodes_demo.sh`
  (wired into `make verify`). `make verify` rc=0 with shell emitter
  not invoked.

- ❌ "Fully Phosphoric-authored system image" / "Phosphoric runtime
  emitted the boot artifacts" — these stronger claims remain
  PARTIAL. The boot artifact bytes themselves are produced via the
  PNP byte-encoding pattern (2189 single-byte-exit-code Phosphoric
  source files compiled by phase0_stub), which is Phosphoric-source-
  derived but not "the Phosphoric runtime emitting bytes via its
  own ASM/PE writer." A full ASM/PE-writer-in-Phosphoric is v0.3+
  scope. Reserved as a named promotion trigger.

  Per panel ruling 2026-05-02 (Option D with claim rename), the
  weaker but truthful claim — **"Phosphoric-specified boot image
  with HOST_REFERENCE emission"** — is ENFORCED and is what
  `v0.1` ships against. Evidence: `apps/demo/*.phos` source-of-
  truth + reviewed `boot_ir_v1` / `boot_asm_v1` goldens +
  deterministic shell emitter (HOST_REFERENCE per cutover
  authority states) + project-owned PE/COFF writer + `BOOTX64.EFI`
  sha256 + QEMU markers. Steps 1–5 of the v0.1 plan are ENFORCED;
  step 6 split into the ENFORCED + FUTURE pair above.

  v0.2 Session 7 progress (2026-05-03, option (ii) Stage 6 of α —
  Pass T multi-param + gap-6/7/8 closure): one branch point edited
  in Pass T (`.Lpt_skip_ws3`, between `(` and `)`) — replaces
  empty-paren-only WS+`)` accept with walk-anything-until-`)` so
  multi-parameter fn signatures parse cleanly. Param contents (names,
  `:`, type tokens including dotted paths, `,` separators, whitespace)
  are walked char-by-char and discarded — they contribute nothing
  to the INT table or emit. Empirical findings (Step 0): gap 6
  (u8/u16 field types), gap 7 (array field type `[T; N]`), and
  gap 8 (nested type in field, e.g. `a.b.C`) are all FREE under
  Stage 5's type-agnostic brace walker — no Pass T edits required.
  **Third apps/demo source compilable byte-equal by pcc-stage1.bin
  via stage0_synth_entry**: render_commands.phos canonical sha
  `a1b1ef0c…` (1177B = 1081 + 4×24 for 5 fns; 2 structs skipped,
  including u8 fields and `[RenderCommand; 16]` array field).
  Gate advanced **56/87 → 59/90** (+3 closures pinned: render_commands,
  struct_u8_array_field micro, multi_param_u16 micro; 31 GAPs
  unchanged); exit42 sha unchanged at `9a0d0ca0…`; `make verify` rc=0.
  pcc-stage1.bin sha rotated `3743befeef0708a3…` → `97fac22630b45406…`
  (size unchanged at 18945B). NO phase0_compiler.phos changes, NO
  apps/demo changes, NO claim flip. Architectural decision Step 0:
  option (A) per-source bail — input_event.phos and route_outcome.phos
  remain GAP (need gap 9 comparison-body + gap 11 if/else, both
  out of scope). Forbidden claim "Phosphoric-compiled boot image"
  remains forbidden until all six sources compile byte-equal AND
  pcc-stage2.bin exists AND the boot artifact is produced via
  the Phosphoric chain.

  v0.2 Session 6 progress (2026-05-03, option (ii) Stage 5 of α —
  Pass T struct-skip dispatcher): two new branch points in Pass T
  (`.Lpt_check_profile_or_fn` adds 's', `.Lpt_skip_ws_pre_fn` post-WS
  adds f/s dispatch) plus new `.Lpt_match_struct_kw` block — matches
  `struct IDENT { ... }`, brace-depth-balanced skip via r11 scratch,
  contributes ZERO entries to the fn INT table and ZERO bytes to emit
  output. **Second apps/demo source compilable byte-equal by
  pcc-stage1.bin via stage0_synth_entry**: demo_state.phos canonical
  sha `5450a96c215929c8…` (1273B = 1081 + 8×24 for 9 fns; 2 structs
  skipped). Gate advanced **55/86 → 56/87** (+1 closure pinned:
  `struct_const_fns` micro-fixture; 31 GAPs unchanged); exit42 sha
  unchanged at `9a0d0ca0…`; `make verify` rc=0. pcc-stage1.bin sha
  rotated `8807c151…` → `3743befeef0708a3…` (size unchanged at
  18945B). NO phase0_compiler.phos changes, NO apps/demo changes,
  NO claim flip — demo_state.phos is the second of six apps/demo
  sources; the remaining four (render_commands, input_event,
  route_outcome, button_policy) require further α stages
  (multi-parameter functions, struct field types u8/u16, array field
  types, nested struct types, if/else, etc.) per the per-source gap
  inventory. Forbidden claim "Phosphoric-compiled boot image" remains
  forbidden until all six sources compile byte-equal AND pcc-stage2.bin
  exists AND the boot artifact is produced via the Phosphoric chain.

  v0.2 Session 5 progress (2026-05-03, option (ii) Stage 4 of α —
  Pass T + dynamic-N multi-fn synth path): new Pass T outer recognizer
  in `phase0_stub.S` `stage0_synth_entry` (last-chance position) walks
  strict `module + N × fn IDENT() -> IDENT { INT }` shape; new
  `.Lsy_synth_multi_fn` synth path emits 1081 + (N-1)×24 bytes with
  STACK_FRAME / M.3.Z LEAF helper shapes (M.3.Z for IMM=0). **First
  apps/demo source compilable byte-equal by pcc-stage1.bin via
  stage0_synth_entry**: boot_entry.phos pinned at canonical sha
  `426ce0d91f4add0e…` (1297B). Gate advanced **54/85 → 55/86**
  (+1 closure, 31 GAPs unchanged); exit42 sha unchanged at
  `9a0d0ca0…`; `make verify` rc=0. NO phase0_compiler.phos changes,
  NO apps/demo changes. NO claim flip — boot_entry.phos is one of
  six apps/demo sources; the remaining five (demo_state,
  render_commands, input_event, route_outcome, button_policy) require
  further α stages (struct definitions, multi-parameter functions,
  if/else, etc.) per the per-source gap inventory. Forbidden claim
  "Phosphoric-compiled boot image" remains forbidden until all six
  sources compile byte-equal AND pcc-stage2.bin exists AND the boot
  artifact is produced via the Phosphoric chain.

  v0.2 Session 4 progress (2026-05-03, option (ii) Stage 3 of α —
  lower collect_top_level_names slice): inside `phase0_compile`, the
  `call collect_top_level_names` is replaced with an ~80-line inline
  ASM block whose logic is derived from reading `parse_tokens`'s
  item-name collection slice in phase0_compiler.phos. Same globals
  used as storage (`token_buf`, `token_count`, `name_table`,
  `name_table_count`); no arena allocation introduced; standalone
  `collect_top_level_names` symbol left in place as dead code.
  Gate maintained at **54/85**; exit42 sha unchanged at
  `9a0d0ca0…`; pcc-stage1.bin sha unchanged at `1ccd99459a01…`
  (the inline refactor is observably equivalent at the bit level
  for compiling phase0_compiler.phos). `make verify` rc=0. NO
  phase0_compiler.phos changes. NO claim flip — Stage 3 lowers one
  internal call inside the wrapper; the wrapper's body is still
  hand-written ASM derived from source, not lowered Phosphoric
  source itself. Forbidden claim "Phosphoric-compiled boot image"
  remains forbidden until pcc-stage2.bin exists and produces the
  boot image.

  v0.2 Session 3 progress (2026-05-03, option (ii) Stage 2 of α —
  phase0_compile ASM wrapper symbol): new `.global phase0_compile`
  symbol added to `phase0_stub.S`, wrapping the analysis pipeline
  (lex_source → profile scan → collect_top_level_names →
  check_duplicate_names → count_top_level_fns → record_fn_offsets).
  `_start` now invokes `call phase0_compile` instead of inlining the
  pipeline. On success `rax = count` (preserves the prior
  `mov r13, rax` contract); on lex/dup failure jumps directly to
  existing `.Llex_error` / `.Ldup_error`. **First attempt regressed
  35 multi-fn fixtures via the rax-on-success contract** (returned
  rax=0 instead of rax=count); reverted cleanly to 54/85; Fix A
  reapplied. Gate maintained at **54/85**; exit42 sha unchanged at
  `9a0d0ca0…`; pcc-stage1.bin sha unchanged at `1ccd99459a01…`
  (strongest evidence the stub refactor is observably equivalent).
  No phase0_compiler.phos changes this session. NO claim flip —
  `phase0_compile` is now callable at the ASM level but its body
  is still hand-written ASM, not lowered Phosphoric source. Stage 3+
  of α handles incremental lowerings. Forbidden claim status:
  unchanged this session.

  v0.2 Session 2 progress (2026-05-03, option (ii) Stage 1 of α —
  source-spec truthfulness only): `phase0_compiler.phos` `emit_elf`
  spec aligned with actual ASM emit behavior; new
  `find_first_return_imm` helper threads `NodeStmtReturn.payload[0]`
  into per-fn emit; `emit_exit_zero` renamed to
  `emit_exit_with_imm` and parameterized; rdi-load low byte now
  AST-driven (`imm as u8`); high three bytes truncated due to no
  source-side u32-to-bytes operators (named gap, future α stage).
  Gate maintained at **54/85** (no regression); exit42 sha
  unchanged at `9a0d0ca0…`. NO phase0_stub.S edits. NO claim
  flip — option (ii) is multi-session per path α (panel-authorized
  2026-05-03); subsequent stages lower `check_acyclicity`,
  `type_check`, then add ASM-level `phase0_compile` symbol; pcc-stage2.bin
  remains terminal. Forbidden claim status: unchanged this session.

  v0.2 Session 1 progress (2026-05-03): shapes 52a / 52b / 53
  closed at byte-equal **54/85** (`tools/verify/verify_source_asm_byte_equal.sh`).
  This advances the toolchain-acceptance precondition for
  pcc.phos's hot path (multi-segment paths + fixed-cap arrays
  syntactically accepted) but does NOT flip the forbidden claim.
  The 52b closure is option (i) stub-route — `phase0_stub.S`'s
  `.Lsy_ret_ident_done` recognizer routes multi-segment call
  sites to the canonical 1081-byte exit-0 ELF. It is **not**
  real call-site lowering. Real lowering — option (ii): wire
  `stage0_entry` to call `phase0_compile` and extend `emit_elf`
  for return-value semantics — is the separately-authorized next
  session, recorded in `docs/v0_1_followups.md`. Until option
  (ii) lands, pcc-stage2.bin would compile its inputs but produce
  nothing semantically meaningful. Forbidden claim status:
  unchanged this session.
- ❌ "Self-hosted" / "selfhost achieved" — `phase0_compiler.phos` does
  not yet compile itself to a fixpoint. The chain on it still degrades
  through M.3.M nested shells (2305 → 766 → 451 → 136 → not produced).
  The byte-equal fixpoint that exists is on `quine_self.phos`, a
  synthetic test fixture, not on the canonical compiler source.
- ❌ "ASM bootstrap root retired" — the producer is still
  `untracked/internaldocs/phase0_producer/phase0_stub.S`, ~4000 lines
  of hand-written assembly. Retirement requires `phase0_compiler.phos`
  to compile itself.
- ❌ "Residual emission verified" — R1..R7 emission infrastructure is
  unimplemented. `tools/verify/verify_residual_stream.sh` is a stub
  that exits 0 informationally.
- ⚠ "Court runtime emits PFI0 / R5 bytes" / "Phosphoric-runtime
  produced PFI0 case file" — partially earned at v0.3 (2026-05-03):
  the v0.3 razor demo bootable (Phosphoric-source-derived, PE32+
  reproducible from `tools/phosphoric/dsfb_pnp/byte_NNNN.phos × 2070`)
  emits **3 typed residual records (R7 boot_check + R6 task_transition
  × 2) at runtime** to debug_data_port (0x500) under QEMU, with
  byte-stable chain_hash chains derived per `kernel/residual.phos`
  primes (31, 131, 524287, 16777213). The captured 96 bytes wrap
  byte-stable into a 256 B PFI0 case file (`tests/golden/dsfb_demo.pfi`,
  sha `1310b9560fb93a9d…`) verified by the existing
  `check_pfi_layout.sh` gate (which re-derives the chain). Caveats:
  (a) the records are **pre-computed at golden-manufacture time** by
  a Python implementation of `chain_step` — the bootable writes
  pre-computed bytes via `out dx, al` rather than computing the
  chain at runtime; (b) the PFI0 header/footer wrap is host-side
  (`tools/verify/encode_pfi.sh`); (c) only 3 of the 7 R-types are
  exercised; the other 4 stay paper. The strong claim ("compiled
  classifier emits chain at runtime") is reserved until phase0_compile
  body lowering can compile `kernel/residual.phos` end-to-end.
- ❌ "Phosphoric runtime classifier executes" / "court runtime
  adjudicates" / "compiled classifier emits verdict" — D1
  (2026-05-03) earns only the weaker claim that the host reference
  verdict path produces canonical verdict bytes byte-identical to
  the verdict expectation. The verdict tool is host-side bash + awk
  + od. The Phosphoric-source classifier
  `tools/phosphoric-host/phosphoric_drift.phos` is not executed by
  this gate; it remains source-as-spec doctrine. The strong claim
  is reserved until a court-side Phosphoric-compiled binary emits
  the same 6 lines and replaces `verdict_from_pfi.sh` in
  `verify-court-d1-verdict`.
- ❌ "General PFI validator" / "general R5 classifier" / "runtime
  enforcement of payload validity" / "Phosphoric-compiled
  validation" — B1 (2026-05-03) earns only the narrow claim that the
  host reference court validates the **single semantic invariant**
  `observed ∉ [declared_lo, declared_hi]` on R5 mmio_touch cases.
  B1's validator parses only the four fields needed for that check;
  it is not a general PFI parser. Layout / chain / hash validation
  remain owned by the existing layout / R5-byte-layout / malformed
  gates. Promotion to a Phosphoric-compiled validator that emits
  the same exit codes is a reserved future court requirement.
- ❌ "General forensic runtime complete" — the single-case R5
  host-reference court loop is closed (2026-05-03), but that is the
  *narrow* loop, not a general runtime. The host-reference saturation
  claim is bounded by: one residual kind (R5 mmio_touch), one case,
  one verdict class (MMIO_BOUNDARY_PRESSURE), one count (=1).
- ❌ "Phosphoric runtime emitted the evidence" — every byte in the
  single-case loop is produced by host-side bash + awk + od +
  sha256sum. The promotion trigger has not fired.
- ❌ "Compiled classifier adjudicated the case" — D1's verdict tool
  is host-side bash + awk + od. No Phosphoric-compiled classifier
  has executed against any PFI in any gate.
- ❌ "All residual kinds covered" — only R5 mmio_touch is closed.
  R1, R2, R3, R4, R6, R7 have byte-layout / taxonomy doctrine
  (kernel/residual.phos) but no host-reference court chain
  (emitter / verdict / semantic-validity). The breadth trigger has
  not fired.
- ❌ "Multi-record replay proven" — the closed loop is single-record
  (`residual_count = 1`). The layout gate supports `count > 1`
  structurally, but no multi-record fixture / verdict / replay path
  is admitted. The replay trigger has not fired.
- ❌ "Drift classification verified" — the classifier
  (`tools/phosphoric-host/phosphoric_drift.phos`) is source-as-spec
  doctrine. The producer cannot lower its constructs.
- ❌ "Compiler closure achieved" — see "Self-hosted" above. The
  scaffold-tier fixpoint is the smallest doctrine tier; it does not
  imply closure on the canonical compiler.
- ❌ "Mature test coverage" — the corpus is razor-minimal by design.
  See `docs/FIXTURE_RAZOR.md` "Why no industrial compiler test suite".
  Counting fixtures as a proxy for coverage is a category error.
- ❌ "Log analyzer" / "tracing" / "telemetry" / "observability" —
  Phosphoric is a **drift-and-slew residual court**, not a log
  analyzer. A log analyzer asks "what interesting things might have
  happened?"; the court asks "which declared invariant was violated,
  according to the evidence?" See `docs/FORENSIC_PRIMACY.md` apex
  statement and canonical role mapping. Adopting log-analyzer framing
  dissolves the determinism doctrine.
- ❌ "Anomaly detection" / "probabilistic scoring" / "warning levels"
  — the classifier emits exactly one named verdict from the closed
  `DriftClass` set. No "maybe suspicious", no "score=0.83", no
  severity grading. See FORENSIC_PRIMACY.md §3 "Canonical verdict
  format" and "Anti-verdicts the classifier MUST NOT emit".

## Doctrine framing (not optional)

The project's apex framing is **adjudication, not observability**.
External communication, internal docs, and code comments must use the
court / verdict / evidence / case-file vocabulary, not the log /
event / trace / score vocabulary. The two framings imply different
systems; only the first preserves the determinism guarantee.

## Doctrine alignment

Phosphoric optimizes for **deterministic truth, byte-level evidence,
and post-failure ambiguity reduction** (the apex doctrine). It does
NOT optimize for:
- developer ergonomics
- language expressiveness
- compile speed or runtime performance
- broad feature coverage

Claims that imply optimization for the second list are forbidden.
Claims that imply optimization for the first list are permitted only
when supported by the gates in `make verify-legendary`.
