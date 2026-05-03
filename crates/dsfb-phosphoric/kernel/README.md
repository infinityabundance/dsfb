# PhosphorOS Runtime Kernel (Phosphoric)

This directory holds the active runtime kernel — the Phosphoric port of a historical pre-Phosphoric kernel that was retired and removed from the active repo (see [docs/RETIREMENT.md](../docs/RETIREMENT.md)).

**Profile:** runtime (declared per file). See [docs/language/RUNTIME_PROFILE.md](../docs/language/RUNTIME_PROFILE.md).

## Layout

| File | Module | Role |
| --- | --- | --- |
| [error.phos](error.phos) | `pcc.kernel.error` | KernelError enum + status mapping |
| [caps.phos](caps.phos) | `pcc.kernel.caps` | CapabilityTable + handle types |
| [tasks.phos](tasks.phos) | `pcc.kernel.tasks` | TaskTable, TaskState |
| [ipc.phos](ipc.phos) | `pcc.kernel.ipc` | ChannelTable, MessagePayload |
| [windows.phos](windows.phos) | `pcc.kernel.windows` | WindowTable, focus (recessed: damage cut) |
| [framebuffer.phos](framebuffer.phos) | `pcc.kernel.framebuffer` | FramebufferTable, PixelFormat (recessed: format negotiation cut) |
| [manifest.phos](manifest.phos) | `pcc.kernel.manifest` | Runtime manifest table + cap/channel containment predicates |
| [residual.phos](residual.phos) | `pcc.kernel.residual` | Residual ring + chain step + record API |
| [abi.phos](abi.phos) | `pcc.kernel.abi` | Kernel ↔ Ember signatures, InputEvent, RenderCommand |
| [kernel.phos](kernel.phos) | `pcc.kernel.kernel` | Top-level loop, GlobalTables |

## Status

Structural types are committed; function bodies are TODO-marked stubs. Real bodies land with elevation item E0f. The kernel is not yet executable — it compiles to a typed signature-only artifact pending `pcc.phos` self-hosting.

## The unwrap discipline

The historical pre-Phosphoric kernel had 30 implicit-panic sites — each one assumed an invariant without proving it. The pre-Phosphoric language had a panic-on-failure idiom; Phosphoric has none.

Phosphoric has no panic construct at all. Every former implicit-panic site translates by language rule into a typed `Result[T, KernelError]` return. The translation table is recorded per-module in the `// TODO(E0f)` comments — each comment names the original site and the new error variant.

This is the load-bearing claim of E0f: the kernel's failure-mode discipline is upgraded from "panic on bad state" to "return typed error to caller", enforced by language design rather than by lint.

## Capacities (from runtime_profile.toml)

| Object | Capacity | Notes |
| --- | --- | --- |
| Tasks | 64 | Per-task slot in TaskTable |
| Channels | 128 | Per-channel slot; 8 messages per queue |
| Windows | 16 | Per-window slot; one focused window |
| Framebuffers | 4 | Demo uses 1 |
| Capabilities total | 449 | Sum across kinds |
| Global IPC pool | 1024 messages | Bounded across all channels |
| IPC payload | 256 bytes | Per-message ceiling |

All capacities are compile-time constants. A program that requests more is rejected by the type system.

## Tests

Test corpus lives at [tests/kernel/](../tests/kernel/) (TBD; ~30 tests with E4). The runner is [tools/phosphoric-host/phosphoric_test_runner.phos](../tools/phosphoric-host/phosphoric_test_runner.phos). Tests are runtime-profile programs that exercise typed error paths against a deterministic Ember stub.

## Audit

[docs/KERNEL_AUDIT.md](docs/KERNEL_AUDIT.md) (TBD with E0f) records the per-module audit at the same precision as Ember's. Each function names its caller-side invariants and post-conditions.
