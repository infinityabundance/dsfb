# pcc — Phosphoric Self-Hosted Compiler

`pcc` is the Phosphoric compiler, written in Phosphoric host profile. It replaces a historical pre-Phosphoric compiler that was retired and removed from the active repo (see [docs/RETIREMENT.md](../docs/RETIREMENT.md)).

## Status

This directory is the active home of the canonical compiler. As of 2026-04-27 it contains scaffolding plus the Phase A–E manifest authority and residual primitive work. Full self-hosting (E0b) requires Stage 0 to produce a runnable `pcc-stage1.bin`.

## Layout

The compiler is organized as a flat list of `.phos` files, one module per file, all under `module pcc.<name>`. There is no rollup-file-style aggregation — v0 grammar has no `use` statement, so callers reference functions by their full dotted path.

| File | Module | Role |
| --- | --- | --- |
| [pcc.phos](pcc.phos) | `pcc.driver` | CLI entrypoint; reads `.phos` source, runs passes, writes output. |
| [lexer.phos](lexer.phos) | `pcc.lexer` | Source bytes → token stream. |
| [parser.phos](parser.phos) | `pcc.parser` | Token stream → AST. (TBD) |
| [hir.phos](hir.phos) | `pcc.hir` | AST → HIR data structures. (TBD) |
| [hir_wf.phos](hir_wf.phos) | `pcc.hir_wf` | HIR well-formedness gate. (TBD) |
| [typeck.phos](typeck.phos) | `pcc.typeck` | Type checking + ownership joins. (TBD) |
| [effects.phos](effects.phos) | `pcc.effects` | Effect closure pass. (TBD) |
| [layout.phos](layout.phos) | `pcc.layout` | Type layout analysis. (TBD) |
| [stack_analysis.phos](stack_analysis.phos) | `pcc.stack_analysis` | Worst-case stack depth. (TBD) |
| [frame_layout.phos](frame_layout.phos) | `pcc.frame_layout` | Function frame sizing. (TBD) |
| [call_graph.phos](call_graph.phos) | `pcc.call_graph` | Same-module call cycles. (TBD) |
| [budget.phos](budget.phos) | `pcc.budget` | TOML budget manifest reader. (TBD) |
| [diagnostic.phos](diagnostic.phos) | `pcc.diagnostic` | Stable diagnostic codes + spans. (TBD) |
| [assurance.phos](assurance.phos) | `pcc.assurance` | Assurance report v1 emitter. (TBD) |
| [codegen_boot.phos](codegen_boot.phos) | `pcc.codegen.boot` | HIR → boot_ir_v1 → boot_asm_v1. (TBD) |
| [codegen_host.phos](codegen_host.phos) | `pcc.codegen.host` | HIR → Linux x86_64 ELF. (TBD) |
| [codegen_trusted.phos](codegen_trusted.phos) | `pcc.codegen.trusted` | HIR → Ember-shape boot binary. (TBD) |

Per-pass LOC ceilings are recorded in [docs/tcb_budget.toml](../docs/tcb_budget.toml) under `[component.pcc_total.subceilings]`. Aggregate ceiling: 6 500 LOC.

## Profile

`pcc` is a host-profile program. It declares the host effects it needs — `host-fs-read` for source input, `host-fs-write` for output artifacts, `host-stdout`/`host-stderr` for diagnostics, `host-time-mono` for timing, and `host-hash` for golden-fixture comparison. See [docs/language/HOST_PROFILE.md](../docs/language/HOST_PROFILE.md).

A subset of `pcc` runs as a build-side library inside other host tools (e.g. `phosphoric-conform`, `phosphoric-fuzz`). For library use the CLI driver is bypassed and individual passes are invoked directly.

## Diagnostic Stability

Every diagnostic code emitted by `pcc` is stable. The UI corpus in [tests/ui/](../tests/ui/) is the ground truth: a passing test must produce a documented code and span attribution. The pre-Phosphoric historical compiler's diagnostic codes are recorded in the project history for archaeological reference but are not consulted by the active build path.

## Bootstrap

`pcc` is compiled by the bootstrap chain documented in [bootstrap/STAGE0.md](../bootstrap/STAGE0.md) and [bootstrap/bootstrap.toml](../bootstrap/bootstrap.toml). The active doctrine path is **phase 0** — Phosphoric-source bootstrap built externally by attesters with hand-coded x86_64 ASM. Phase 0 source is in [phase0/phase0_compiler.phos](../phase0/phase0_compiler.phos); the runbook for attesters is [phase0/HANDBOOTSTRAP.md](../phase0/HANDBOOTSTRAP.md). The current chain state is `SCAFFOLD`. Stage N for N ≥ 1 is `pcc.phos` compiled by stage N−1; the fixpoint `stage{N+1} == stage{N+2}` byte-for-byte is asserted by `verify_fixpoint.phos`.

## Non-Goals

- No optimization passes beyond what the boot profile requires.
- No new language features beyond v0 plus the host/trusted/runtime profile additions documented in [docs/language/](../docs/language/).
- No general-purpose assembler or linker — the `boot_asm_v1` and ELF backends emit only the narrow subset needed for the project's three profiles.
- No incremental compilation, no parallel compilation, no LSP, no language server.
