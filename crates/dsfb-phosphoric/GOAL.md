# Phosphoric — Goal

Anchor for Stage 2 alignment evaluation. Subsequent file-by-file decisions
("does this v0.1/ artifact align with the goal?") use this document as
the criterion.

## What Phosphoric is

A small, deployable runtime for tiny edge microcontrollers, built so
every privileged operation is line-precision auditable, every authority
is explicit and capability-typed, every memory and control-flow
behavior is deterministic, and every runtime drift produces a
byte-stable residual record that proves what happened.

## What Phosphoric is for

Industrial edge devices on $5-class hardware where:
- A vendor cannot ship undisclosed code paths (every byte traceable to
  source)
- A field failure must produce evidence that survives scrutiny (not a
  log; a court-grade residual)
- Deterministic memory and timing behavior is required (no heap
  allocation, no dynamic dispatch, no ambient authority)
- Capability-based authority replaces process-based isolation (because
  the substrate is too small for a conventional kernel)

Phosphoric is **not** a general-purpose systems language and is not
trying to be one. It is a substrate for one job, done with discipline.

## Substrate development order (current → eventual)

| Stage | Target | Status |
|---|---|---|
| v0.x | QEMU x86_64 + UEFI + the demo loop | **current proving ground** — v0.1 ships against this |
| v1.x | $5-class MCU deployment: RP2040, RP2350, ESP32-C3, low-end Cortex-M, simple RISC-V with PMP | next rung once x86 path is end-to-end honest |
| v2.x+ | additional hardware variants | follows v1.x |

QEMU x86 is not the product. It is the proving ground where the audit
discipline gets exercised end-to-end before the discipline costs
money on real silicon. The $5-MCU rung is the product. The discipline
is identical at both rungs; only the substrate target differs.

## The five disciplines

These are non-negotiable. A file that violates one is not Phosphoric.

1. **Auditable line-by-line.** Every privileged operation justified
   by name + line. No hidden control flow. No ambient authority. No
   surprise allocations.
2. **Capability-typed.** Authority is explicit, affine, and declared.
   The kernel cannot perform an operation it does not hold a cap for.
   Capabilities are not strings or ints; they are types.
3. **Deterministic.** Fixed-capacity tables. Bounded loops by language
   rule. No heap. No dynamic dispatch. No syscall the manifest does
   not declare.
4. **Forensic.** When something drifts at runtime — MMIO out of
   declared range, slew exceeded, residual produced — the runtime
   emits a byte-stable residual record. The court is a residual
   court, not a log analyzer. "No silent authority": every authority
   transition has either a declared manifest edge or a typed
   residual; no third path.
5. **Honestly layered.** Three layers, no more, no less:
   - **Ember**: the machine nucleus. Owns boot, traps, page tables,
     context switch, and the small set of hardware-dangerous
     operations.
   - **Phosphoric**: the constrained safe surface above Ember.
     `no_std`, `no_alloc`, `no_unsafe`, capability-typed, fixed
     capacity, declared effects.
   - **PhosphorOS**: the OS layer built on Ember + Phosphoric.
     Fixed-capacity tables, message-passing IPC, capability-based
     authority, single-window GUI demo today.

## Bootstrap discipline

The "fully-Phosphoric-no-ASM-ever" framing was a fantasy. Every
language has a bootstrap problem: C originally bootstrapped from
PDP-11 ASM, Rust started in OCaml, Go started in C. The only path
out of that problem is custom silicon that decodes a high-level IR
directly — and even then the silicon definition itself is HDL source
compiled by something else. The trust always anchors somewhere
outside the language.

**ASM is the honest trust anchor.** It is a thin, inspectable,
byte-level layer that maps directly to CPU instructions a human can
audit. The discipline is not "no ASM ever." The discipline is:

> For every Phosphoric source fixture, the compiler emits ASM that is
> byte-equal to a reviewed-and-pinned spec. The proof of the compiler
> *is* the byte-equal evidence.

This is what the 82-fixture source↔ASM closure campaign was. **It
was right.** It reached 51/82 before being paused. The pause was a
wrong turn (the cutover revoked the trust anchor without replacing
it, leaving a doctrine vacuum). The campaign restarts as the
audit-of-record under v0.x, treated as named work, not as an
"explicitly-resumed retirement campaign trigger."

**HOST_REFERENCE emitters** (bash + awk + sha256sum tools) are
admitted **only** as transitional scaffolding while the
Phosphoric-compiled emitters are still being grown. They are not a
permanent state. v0.1 ships with a HOST_REFERENCE boot emitter
honestly labeled as such; promotion to a Phosphoric-compiled emitter
is named future work.

## What is in scope

- The three layers (Ember, Phosphoric, PhosphorOS).
- The bootstrap chain (ASM stub → phase0 compiler → pcc → byte-equal
  Phosphoric).
- The 82-fixture source↔ASM closure campaign as the audit floor.
- The forensic court (residual ABI, PFI0 case file, verdict format,
  no-silent-authority invariant).
- The v0.x QEMU x86 demo as proving ground.
- The v1.x $5-MCU port plan, designed but not built.

## What is out of scope

- General-purpose language features.
- A second Rust/C/Go-style compiler frontend.
- Cosmetic source rewrites that do not change byte-equality.
- "No ASM ever" as a doctrine.
- Scope expansions justified by frustration or by `do X` instructions
  that don't include `do not Y` boundaries.

## How this document gets used

Stage 2 of the v0.1 freeze + curated restart uses this document as
the alignment criterion. For each file in `v0.1/`, the question is:

> Does this file embody one of the five disciplines, or does it
> describe in-scope work? If yes, evaluate for retention. If no,
> leave in `v0.1/` as historical reference and do not copy out.

If a file is retained, it must be retained as-is or with a single
identified delta (one cut, one fix, one rewrite — not bundled).

If multiple files describe the same topic, exactly one is retained
as the single source of truth. Others stay in `v0.1/`.

## Authority resolution

When two `v0.1/` documents contradict, this document is authoritative
as the goal definition. The `v0.1/` documents are historical record;
the goal here is the present statement.

If this document is itself wrong, it is corrected here, in this file,
not by writing a parallel doc.
