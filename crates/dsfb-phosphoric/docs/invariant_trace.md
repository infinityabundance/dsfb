# Invariant Traceability Matrix

This document maps the current invariants to their evidence.

It groups related invariants from `docs/invariant_checklist.md` when they share the same enforcement point. A row marked `not yet enforced` is an explicit gap, not an implied guarantee.

## Status Labels

- `enforced`: checked by active executable verification or smoke test
- `archive-only`: material kept under `archive/` that active verification must not execute and must not use as active proof
- `specified`: documented, but not mechanically enforced yet
- `review-gated`: enforced by review policy and traceability requirements
- `not yet enforced`: known gap that must not be marketed as complete

## Global And Trust Boundary Invariants

| Invariant | Spec source | Evidence | Status |
| --- | --- | --- | --- |
| `Ember` is the only machine-dangerous layer | `docs/tcb.md`, `ember/docs/safety_boundary.md`, `ember/docs/EMBER_MINIMALITY.md` | `docs/security_review_checklist.md` rejects raw MMIO, port I/O, page tables, and privileged operations outside `Ember`; `ember/docs/EMBER_TRUST_AUDIT.md` inventories every current trusted function | review-gated |
| `Phosphoric` is the constrained language surface above `Ember` | `README.md`, `docs/language/overview.md` | compiler frontend forbids unsupported syntax in UI fail corpus | enforced |
| `PhosphorOS` is the capability GUI OS layer above `Ember` | `README.md`, `kernel/docs/*.md` | kernel docs define task, IPC, capability, compositor, input, and window boundaries | specified |
| The project goal is minimized trust, not categorical security superiority | `README.md`, `docs/threat_model.md` | `docs/security_review_checklist.md` rejects stronger claims | review-gated |
| Prototype target is `x86_64` + `UEFI` + `QEMU` | `README.md`, `docs/repro_build.md` | `tools/qemu-run/run_uefi_demo.sh` boots the current vertical slice | enforced |
| Golden boot path links no active non-Phosphoric runtime object | `docs/BOOTSTRAP_TCB.md`, `docs/repro_build.md` | `tools/verify/check_all.sh` rebuilds the image, requires no non-Phosphoric runtime objects, requires `c_objects=none` and `archive_executed=false`, rejects active non-Phosphoric/C files outside archive, compares generated IR/ASM to reviewed fixtures, and checks generated `efi_main` | enforced |
| Archived non-Phosphoric reference code remains inert | `docs/BOOTSTRAP_STRATEGY.md`, `docs/BOOTSTRAP_TCB.md` | `tools/verify/check_archive_inert.sh` rejects active verification and release paths that execute archived bootstrap tooling or reference archived bootstrap trees | enforced |
| Compiler correctness remains part of the v0 build TCB | `docs/tcb.md` | explicitly listed as build TCB and traced here | specified |
| Single address-space prototype is not strong hostile-task isolation | `docs/threat_model.md`, `docs/tcb.md` | explicitly listed as out of scope and not yet guaranteed | specified |

## Language And Compiler Invariants

| Invariant | Spec source | Evidence | Status |
| --- | --- | --- | --- |
| Language contract is `no_std`, `no_alloc`, and `no_unsafe` | `README.md`, `docs/language/overview.md` | current compiler surface has no syntax for std, alloc, unsafe, or FFI | specified |
| Supported surface remains modules, capabilities, structs, enums, functions, pattern matching, fixed arrays, bounded slices, `Option`, `Result`, bounded `for`, moves, and effects | `docs/language/grammar.md`, `docs/language/V0_FREEZE.md`, `docs/language/type_system.md` | UI pass corpus covers the current parsed and checked subset | archive-only |
| Unsupported constructs are rejected, not emulated | `docs/non_goals.md`, `docs/language/grammar.md` | UI fail corpus covers trait, async, generic function, and heap-string syntax | archive-only |
| Borrow-like syntax is outside the frozen v0 surface and is rejected | `docs/language/V0_FREEZE.md`, `docs/language/type_system.md` | UI fail corpus rejects `&x`, `&mut x`, and borrowed parameters | archive-only |
| Lexer token set stays aligned with grammar | `docs/language/grammar.md` | lexer unit test covers documented token classes | archive-only |
| Parser rejects unsupported constructs | `docs/language/grammar.md` | parser unit tests and UI fail corpus | archive-only |
| Compiler rejections have stable diagnostic codes | `docs/quality_bar.md` | `check_source_diagnostic`, UI fail corpus `// expect:` comments, and UI runner assertions | archive-only |
| Every emitted frontend diagnostic code has one direct span-bearing test and one UI fail case | `docs/quality_bar.md` | `tests/diagnostic_coverage.rs` and the UI fail corpus | archive-only |
| Current enforced frontend rejections preserve source spans | `docs/quality_bar.md` | AST/HIR span threading, `check_source_diagnostic`, and `tests/diagnostic_spans.rs` | archive-only |
| `None` spans are reserved for future diagnostics without a recoverable source site; current enforced frontend rejections do not rely on them | `docs/quality_bar.md` | current lexer, parser, typechecker, and effect checker emit source spans for the enforced rejection set | archive-only |
| HIR is well-formed before type, effect, layout, and stack analysis | `docs/ir.md`, `docs/abi.md`, `docs/language/type_system.md` | `hir_wf`, `tests/hir_wf.rs`, and pass ordering in `lib.rs` | archive-only |
| HIR lowering preserves capability typing | `docs/language/type_system.md` | typechecker capability tests and UI corpus capability cases | archive-only |
| Move-after-move is rejected, including conservative branch and loop joins | `docs/language/type_system.md` | `move_after_move.phos`, `branch_move_after_if.phos`, `loop_move_after_for.phos`, and unit tests | archive-only |
| Reachable-continuation ownership is enforced for `if` and `match` joins; returning branches do not poison later use | `docs/language/type_system.md`, `docs/language/memory_model.md` | returning-branch unit tests in `typeck.rs` and compiler regression suites | archive-only |
| Affine capability duplication is rejected, including conservative branch joins | `docs/language/type_system.md`, `kernel/docs/capabilities.md` | `capability_duplication.phos`, `branch_capability_after_match.phos`, and unit tests | archive-only |
| Undeclared effect use is rejected | `docs/language/effects.md` | `missing_declared_effect.phos`, effect checker tests | archive-only |
| Same-module transitive effect closure is enforced for legal local calls | `docs/language/effects.md` | effect checker closure tests, `frontend_pipeline.rs`, and assurance report summaries | archive-only |
| Unnecessary declared effects are rejected where same-module closure makes them provably redundant | `docs/language/effects.md`, `docs/quality_bar.md` | `unnecessary_effect.phos`, direct diagnostic span test, effect checker test, and assurance report summaries | archive-only |
| Effect set is only `draw`, `ipc`, `mmio`, `sched`, `time` | `docs/language/effects.md` | `unknown_effect_label.phos`, `duplicate_effect_label.phos`, `all_effect_labels.phos` | archive-only |
| Return shapes match declared function signatures | `docs/language/error_model.md` | return mismatch, missing return, bare return, and value-without-type UI fail cases | archive-only |
| Local function calls match arity and argument types | `docs/language/type_system.md` | wrong argument count/type UI fail cases and local-call pass case | archive-only |
| Only same-module named functions are legal call targets | `docs/language/V0_FREEZE.md`, `docs/language/effects.md` | `illegal_external_call_unresolved.phos`, `illegal_external_call_field.phos`, direct diagnostic span tests, and typechecker tests | archive-only |
| Recursive local calls are forbidden in the v0 kernel profile | `docs/language/type_system.md` | direct and indirect recursion UI fail cases | archive-only |
| `if` conditions are boolean and known branch types match | `docs/language/type_system.md` | non-bool condition and incompatible branch UI fail cases | archive-only |
| Known `match` arm types match | `docs/language/type_system.md` | incompatible match arm UI fail case | archive-only |
| Exhaustive `match` checking exists for `bool`, `Option`, `Result`, and same-module enums; unknown domains require a catch-all | `docs/language/type_system.md`, `docs/language/V0_FREEZE.md` | fail corpus covers missing `false`, `None`, `Err`, enum variants, and wildcard requirements; pass corpus covers exhaustive finite-domain matches | archive-only |
| ABI-backed type layout for the frozen subset is deterministic | `docs/abi.md`, `docs/language/memory_model.md` | `layout::tests::computes_deterministic_layouts_for_representative_types`, assurance analysis tests, and `assurance_report_golden.json` | archive-only |
| Configured entrypoints can be rejected for exceeding worst-case stack budgets | `docs/language/memory_model.md`, `config/x86_64_qemu_budget.toml` | `STACK_EXHAUSTION`, `stack_exhaustion.phos`, `tests/assurance_analysis.rs`, and `phosphoric-assure` | archive-only |
| Assurance report schema v1 is stable and deterministic | `docs/repro_build.md`, `docs/quality_bar.md` | `phosphoric-assure`, three golden reports, and the determinism test in `tests/assurance_driver.rs` | archive-only |
| `boot_ir_v1` is the shared fixed-capacity backend contract for the first native boot profile | `docs/BOOTSTRAP_STRATEGY.md`, `docs/BOOT_ABI_V1.md` | `boot_ir_v1`, `tests/boot_ir_v1.rs`, and `boot_ir_v1_button_policy_golden.json` | archive-only |
| The `boot-asm-v1` backend deterministically emits x86_64 assembly for the reviewed boot-policy profile and rejects unsupported profile features with stable diagnostics | `docs/BOOT_ABI_V1.md`, `docs/quality_bar.md` | `phosphoric-emit-asm`, `tests/emit_asm.rs`, `tests/backend_asm_corpus.rs`, and the generated assembly golden fixture | archive-only |
| The legacy `demo-v1` C backend remains oracle-only and no longer owns the booted build path | `docs/BOOTSTRAP_STRATEGY.md`, `docs/rfcs/0001-c-erasure-demo-path.md` | `phosphoric-emit-c`, `tests/emit_c.rs`, and `tests/backend_corpus.rs` | archive-only |
| Borrow lifetime rules | `docs/language/type_system.md` | borrow syntax is frozen out rather than implemented | not yet enforced |
| Full iteration-sensitive or path-complete ownership reasoning | `docs/language/type_system.md` | current checker reasons precisely for reachable `if`/`match` continuations but still treats loops conservatively | not yet enforced |
| Explicit external module/import resolution | `docs/language/V0_FREEZE.md`, `docs/language/effects.md` | frozen out of v0; only same-module named calls are legal today | not yet enforced |

## Memory, Capability, Kernel, And GUI Invariants

| Invariant | Spec source | Evidence | Status |
| --- | --- | --- | --- |
| Runtime path does not depend on heap allocation | `docs/language/memory_model.md`, `kernel/docs/*.md` | specs and review checklist forbid heap-backed runtime growth | specified |
| Runtime storage remains static or fixed-capacity | `docs/language/memory_model.md` | capacity constants documented in memory, task, IPC, and window docs | specified |
| Exhaustion is explicit and local | `docs/language/memory_model.md`, `docs/language/error_model.md` | specified for tasks, windows, channels, messages, and IPC payloads | specified |
| Hidden allocation and hidden dynamic growth are forbidden | `docs/language/memory_model.md`, `docs/security_review_checklist.md` | review checklist rejects hidden allocation | review-gated |
| Prototype capacities remain `TASK_MAX`, `WINDOW_MAX`, `CHANNEL_MAX`, `GLOBAL_MSG_MAX`, `IPC_PAYLOAD_MAX`, `WIDGET_TREE_DEPTH`, and `FILENAME_MAX` | `docs/language/memory_model.md`, `kernel/docs/*.md` | grouped capacity docs plus the current runtime task, channel, and window tables | specified |
| Authority-bearing objects use typed capabilities | `kernel/docs/capabilities.md` | language capability declarations, compiler affine checks, and archive-only runtime capability code | partially enforced |
| New authority does not depend on ambient globals | `docs/threat_model.md`, `kernel/docs/capabilities.md` | review checklist rejects ambient authority | review-gated |
| Fixed-capacity task slots are archive-only | `kernel/docs/task_model.md` | `archive/kernel/src/tasks.rs` and archived runtime tests | archive-only |
| Fixed-capacity IPC remains archive-only | `kernel/docs/ipc.md` | `archive/kernel/src/ipc.rs` and archived runtime tests | archive-only |
| Window management remains archive-only | `kernel/docs/window_model.md` | `archive/kernel/src/windows.rs` and archived runtime tests | archive-only |
| Capability-checked window, channel, and framebuffer access is archive-only | `kernel/docs/capabilities.md`, `kernel/docs/framebuffer.md` | `archive/kernel/src/kernel.rs`, `archive/kernel/src/framebuffer.rs`, and archived runtime tests | archive-only |
| Generation-based stale-handle rejection is archive-only | `kernel/docs/capabilities.md`, `kernel/docs/task_model.md`, `kernel/docs/ipc.md`, `kernel/docs/window_model.md` | archived generation-bumped release paths and stale-handle tests | archive-only |
| Booted demo input routing reaches the generated step path | `kernel/docs/input.md`, `kernel/docs/capabilities.md` | generated BootAsm QEMU log markers for routed input and redraw | enforced |
| Firmware and framebuffer metadata validation | `ember/docs/safety_boundary.md` | current BootAsm smoke path uses synthetic framebuffer metadata; real firmware validation is archived/reference or future work | not yet enforced |
| `Ember` does not become a higher-level policy bucket | `ember/docs/architecture.md` | review checklist and TCB document constrain additions | review-gated |
| Root verification gate is one command from the repository root | `docs/repro_build.md`, `docs/quality_bar.md` | `make verify`, `tools/verify/check_all.sh`, archive inertness, docs, boot provenance, no-non-Phosphoric link, and QEMU smoke gates | enforced |
| Enforced claims have machine-readable evidence links | `docs/quality_bar.md`, `docs/repro_build.md` | `docs/invariant_manifest.toml` and `tools/verify/check_invariants.sh` | enforced |

## Vertical Slice Invariants

| Invariant | Spec source | Evidence | Status |
| --- | --- | --- | --- |
| Demo boots under `QEMU` with `UEFI` | `docs/repro_build.md` | `tools/qemu-run/run_uefi_demo.sh` | enforced |
| Demo emits a bounded render command and routes one synthetic input event | `docs/repro_build.md` | QEMU debug log and UEFI demo smoke path | enforced |
| Generated BootAsm demo runtime owns one button and one explicit redraw path | `docs/repro_build.md`, `docs/BOOT_ABI_V1.md` | `phosphoric_demo_render`, generated assembly golden fixture, symbol gate, and QEMU log marker for redraw completion | enforced |
| Boot path is generated BootAsm only, with no linked non-Phosphoric runtime object | `README.md`, `docs/repro_build.md`, `docs/BOOTSTRAP_TCB.md` | `tools/verify/check_all.sh`, `tools/qemu-run/run_uefi_demo.sh`, manifest checks for no non-Phosphoric runtime objects, `c_objects=none`, `archive_executed=false`, source/artifact hashes, and QEMU generated-runtime markers | enforced |
| One boot-path behavior is generated from frozen-v0 Phosphoric source | `docs/repro_build.md`, `docs/rfcs/0001-c-erasure-demo-path.md` | active `apps/demo/*.phos`, `tools/phosphoric/emit_boot_demo_from_phos.sh`, byte-clean IR/ASM fixture comparison, symbol checks, and QEMU generated-runtime markers | enforced |
| Golden boot path links no non-Phosphoric runtime objects and no C objects | `docs/BOOTSTRAP_TCB.md`, `docs/rfcs/0001-c-erasure-demo-path.md` | `tools/verify/check_all.sh` and `build/uefi-demo/linked-artifact.txt` | enforced |
| Demo exits deterministically after the single-event flow | `tools/qemu-run/run_uefi_demo.sh` | runner waits for input state, injects one key, and requires completion log | enforced |
| Fully Phosphoric-authored system image | `docs/ir.md`, `docs/abi.md` | golden boot image is generated from active `.phos` boot-profile sources, but the emitter is still a narrow bootstrap tool rather than a Phosphoric-native compiler | partially enforced |

## Not Yet Guaranteed

| Gap | Why it matters | Required future evidence |
| --- | --- | --- |
| Formal verification | Current assurance is tests and review discipline, not proof | machine-checked model or proof artifact |
| Full static region / global-table memory budgeting | Frozen v0 compiler can report type and stack layouts, but the runtime tables are not source-visible yet | real runtime source objects or accepted v0.2 static-region design plus budget checks |
| Hardware TCB proof | `UEFI`, `QEMU`, CPU behavior, and boot path remain trusted assumptions | reduced boot boundary plus explicit proof or audit trail |
| Process isolation | single address-space prototype cannot enforce hostile-task isolation | address-space or MPU/MMU isolation design and tests |
| SMP safety | current model is single-core | scheduler and memory-ordering model |
| Writable filesystem | current resource strategy is read-only / packed assets first | fixed-capacity FS spec and tests |
| GPU acceleration | software rendering only | separate GPU threat model and typed device boundary |
| Self-hosting | compiler implementation is archive-only code | staged bootstrap plan and equivalence tests |
| Broader driver support | first target is intentionally narrow | per-driver MMIO model and capability contract |
