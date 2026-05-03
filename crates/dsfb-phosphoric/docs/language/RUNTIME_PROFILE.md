# Phosphoric Runtime Profile

This document defines the **runtime profile**: the variant of the frozen v0 surface used for [PhosphorOS](../../kernel/) — the active runtime kernel. Runtime-profile source implements task scheduling, capability-affine handles, IPC, window management, and framebuffer composition for the bootable demo. It runs on top of [Ember](../../ember/) (trusted profile) and below applications (boot profile).

The runtime profile exists for one reason: the project rule is **only Phosphoric code in active verification**, and the kernel must be Phosphoric source. This profile is the smallest extension to v0 that lets the kernel express scheduling primitives, capability tables, and message queues — without inflating the language toward a general OS surface.

## Relationship To v0

The runtime profile inherits all of v0 unchanged ([V0_FREEZE.md](V0_FREEZE.md)). It adds two narrow capabilities that boot-profile code does not need:

- **Generation-tagged handles**: capability handles carry an explicit generation counter, so a handle to a freed slot is rejected when the slot is reused. Generation discipline is a runtime-profile compile-time feature; the type system tracks generation in the handle type.
- **Bounded queue primitives**: a closed family of fixed-capacity FIFO queue types with explicit overflow semantics (drop-newest, drop-oldest, fail-closed). Queue capacity is a compile-time constant; runtime overflow returns `Result::Err(QueueFull)`.

`no_alloc`, `no_unsafe`, `no_std` continue to apply. There is no heap. There is no inline assembly. Machine-dangerous operations remain Ember's exclusive responsibility — runtime-profile code calls into Ember through narrow trusted-profile entrypoints whose signatures are part of [BOOT_ABI_V1.md](../BOOT_ABI_V1.md).

## File Path Allowlist

Runtime-profile source is permitted only under:

- `kernel/` (and subdirectories)

A `.phos` file under any other path that declares `profile = "runtime"` is rejected with diagnostic `R-001`.

## Effect Alphabet

Runtime profile may declare a subset of the v0 effect alphabet, plus two runtime-specific labels:

| Label | Source | Permits |
|---|---|---|
| `draw` | inherited from v0 | issue render commands to a window |
| `ipc` | inherited from v0 | send or receive on an IPC channel |
| `sched` | inherited from v0 | yield, suspend, or wake a task |
| `time` | inherited from v0 | read the monotonic timer |
| `mmio` | NOT permitted in runtime; Ember-only | — |
| `cap-issue` | runtime-only | mint a new capability handle (generation 0) |
| `cap-revoke` | runtime-only | revoke an existing capability handle (generation increments) |

Runtime programs may not declare host-profile effects (no file I/O, no stdout) and may not declare trusted-profile primitives (no `trusted!`).

## Generation-Tagged Handles

A capability handle in runtime profile is a struct of:

```phos
struct Handle[T] {
    slot: u16,
    generation: u16,
    kind: HandleKind,
}
```

Where `kind` is a closed enum: `Task`, `Channel`, `Window`, `Framebuffer`, `Capability`. The runtime tracks generation per slot in a fixed-capacity table. On revocation, generation increments; subsequent lookups with the old generation return `Result::Err(StaleHandle)`. The compiler rejects handle field access except through the typed lookup function declared in `kernel/handles.phos`.

A handle's generation is *not* path-sensitive; the compiler does not statically prove which generation a handle has. The check is runtime: a stale handle returns an error result, never a panic.

## Bounded Queue Primitives

Three closed queue types:

| Type | Overflow semantics |
|---|---|
| `QueueDropNewest[T; N]` | new write returns `Result::Err(QueueFull)`; queue contents preserved |
| `QueueDropOldest[T; N]` | new write evicts the oldest entry and succeeds |
| `QueueFailClosed[T; N]` | new write past capacity is a hard error returned to the caller; the queue is sealed and rejects further writes until explicitly drained |

Capacity `N` is a compile-time constant. The compiler refuses an `N` larger than the runtime-profile capacity ceiling (default 4 096 entries; language max 65 536; per-program override allowed only downward).

## Forbidden In Runtime Profile

Beyond v0 forbidden items:

- direct MMIO (must call into Ember)
- direct port I/O
- direct CPU control-register access
- inline assembly
- file I/O (no host effects)
- subprocess / signal / thread primitives
- dynamic queue resizing
- queue types other than the three closed family
- handle field access outside the typed lookup function
- recursive scheduling (a task cannot synchronously wait on its own continuation)

## Diagnostic Codes

Runtime-profile codes use the `R-` prefix:

- `R-001` — runtime profile declared outside file-path allowlist
- `R-002` — handle field accessed outside typed lookup
- `R-003` — direct MMIO / port / cpu_op in runtime profile
- `R-004` — queue capacity exceeds language maximum
- `R-005` — host-profile effect declared in runtime profile
- `R-006` — trusted-profile primitive used in runtime profile
- `R-007` — recursive scheduling pattern
- `R-008` — dynamic queue resize attempt

## Enforcement Rule

- A `.phos` file is runtime-profile if and only if it declares `profile = "runtime"` and lives under `kernel/`.
- A runtime program may issue `cap-issue` / `cap-revoke` only on handles whose kind it owns.
- A runtime program may not import host-profile or trusted-profile modules; it may import boot-profile modules only as data definitions (no executable boot code).
- Calls into Ember go through the typed entrypoints declared in [BOOT_ABI_V1.md](../BOOT_ABI_V1.md); the entrypoints are statically resolved at link time, no dynamic dispatch.

## Review Rule

Reject any change that:

- adds an effect label to runtime profile without a corresponding entry in [runtime_profile.toml](runtime_profile.toml)
- adds a queue type beyond the three closed family
- relaxes the file-path allowlist
- introduces dynamic memory, unbounded buffers, or runtime feature flags
