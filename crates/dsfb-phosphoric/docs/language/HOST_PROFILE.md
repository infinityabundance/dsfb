# Phosphoric Host Profile

This document defines the **host profile**: the variant of the frozen v0 surface used to write Phosphoric programs that run on a developer machine, not on the bare-metal target. Host programs are the tools that compile, audit, attest, and verify Phosphoric source. They are not part of any boot image, not loaded by Ember, and not callable from the runtime kernel.

The host profile exists for one reason: the project rule is **only Phosphoric code in active verification**. Verification tooling, conformance runners, attestation emitters, and similar host programs must be written in Phosphoric. The host profile is the smallest extension to v0 that makes this possible without inflating the boot or runtime surface.

## Relationship To v0

The host profile is a strict superset of [V0_FREEZE.md](V0_FREEZE.md) for *language form* and a strict, narrow extension for *effects* and *codegen target*.

Inheriting unchanged from v0:

- top-level forms (modules, capabilities, structs, enums, functions)
- explicit integer types, `bool`, fixed arrays, bounded slices, `Option[T]`, `Result[T, E]`
- control flow: `if`, `match`, bounded `for`, explicit `return`
- semantics: move-oriented ownership, affine capability handling, result-style failures
- everything in the v0 forbidden list ([V0_FREEZE.md](V0_FREEZE.md) §Forbidden In v0)
- everything in v0 explicitly-not-in-the-frozen-surface ([V0_FREEZE.md](V0_FREEZE.md) §Explicitly Not In The Frozen Surface)

`no_std`, `no_alloc`, `no_unsafe` continue to apply. A host program has zero dynamic allocation and zero unsafe blocks. Buffers are caller-allocated stack arrays of fixed declared capacity. No `String`, no `Vec`, no `Box`.

## Added Effect Labels

A host program may declare exactly six new effect labels. Each is required-explicit; default is denied. Combined with the v0 set, the legal effect alphabet for any single program is one of:

- boot profile: subset of `{draw, ipc, mmio, sched, time}`
- host profile: subset of `{host-fs-read, host-fs-write, host-stdout, host-stderr, host-time-mono, host-hash}`
- trusted profile (Ember only): defined in [TRUSTED_PROFILE.md](TRUSTED_PROFILE.md)
- runtime profile (kernel only): defined in [RUNTIME_PROFILE.md](RUNTIME_PROFILE.md)

A program may declare effects from exactly one profile alphabet. Cross-profile mixing is a hard compile error. The check belongs in the same effect closure pass that exists today.

The six host labels:

| Label | Permits |
|---|---|
| `host-fs-read` | Open and read from a regular file under the project tree, into a caller-supplied stack buffer of declared capacity. |
| `host-fs-write` | Open, truncate, and write to a regular file under the project tree, from a caller-supplied buffer. |
| `host-stdout` | Write a caller-supplied buffer to file descriptor 1. |
| `host-stderr` | Write a caller-supplied buffer to file descriptor 2. |
| `host-time-mono` | Read the monotonic clock as a `u64` of nanoseconds since process start. |
| `host-hash` | Compute SHA-256 over a caller-supplied buffer. |

## Capacity Ceilings

A host program declares its buffer ceilings as compile-time constants. The defaults below are language-level maxima; an individual program may declare smaller ceilings. The compiler rejects any declaration that exceeds the language ceiling.

| Buffer kind | Default ceiling | Language maximum |
|---|---|---|
| File path length | 256 bytes | 4096 bytes |
| File contents (single read or write) | 1 MiB | 16 MiB |
| stdout/stderr message | 4 KiB | 64 KiB |
| Hash input | 16 MiB | 256 MiB |
| Argv entry length | 256 bytes | 4096 bytes |
| Argv count | 16 | 256 |
| Working-set buffer (per program) | 32 MiB | 256 MiB |

A program that exceeds its declared working-set ceiling at compile time is rejected. The ceiling is checked by summing the static sizes of every fixed array reachable from the entrypoint.

## Codegen Target

The host profile compiles to a statically-linked Linux x86_64 ELF executable. The generated binary uses the Linux kernel ABI directly via the same named-syscall mechanism Ember uses for its UEFI boundary. There is no libc, no dynamic linker, no startup runtime beyond the entrypoint.

Permitted Linux syscalls, one per host effect label:

| Effect label | Linux syscall(s) |
|---|---|
| `host-fs-read` | `openat`, `read`, `close` |
| `host-fs-write` | `openat`, `write`, `close`, `fsync` |
| `host-stdout` | `write` (fd=1) |
| `host-stderr` | `write` (fd=2) |
| `host-time-mono` | `clock_gettime` (CLOCK_MONOTONIC) |
| `host-hash` | none — pure computation |

The codegen pass refuses to emit any syscall not in this table. Adding a syscall requires an explicit edit to [host_profile.toml](host_profile.toml) and a new effect label.

The entrypoint signature is:

```phos
fn main(argv: [BoundedSlice[u8; 256]; 256], argc: u32) -> i32 effects(host-stdout, host-stderr)
```

`argc` is bounded to 256 by the loader. `argv[0]` is the program name. There is no environment variable access, no stdin, no signal handling, no current-working-directory query. A program that needs the current directory must be invoked with the directory as an explicit argument.

## Cross-Profile Separation

The compiler enforces these rules:

- A `.phos` file declares its profile via `profile = "boot" | "host" | "trusted" | "runtime"` in its module header. Default is `boot`.
- A program may import only modules of its own profile.
- A program may declare only effects from its own profile alphabet.
- The same source file cannot be compiled in two profiles. If a function genuinely belongs to two profiles, the source must be duplicated, intentionally.

The cross-profile check is part of the effect closure pass. UI corpus must include positive tests (each profile compiles cleanly) and negative tests (mixing host and boot effects is rejected with a stable diagnostic code).

## Forbidden In Host Profile

Beyond the v0 forbidden list, the host profile additionally forbids:

- network access (no socket syscalls)
- subprocess creation (no `fork`, `clone`, `execve`, `vfork`, `posix_spawn`)
- environment variable access (no `getenv`-equivalent)
- signal handlers (no `sigaction`, `signal`)
- shared memory (no `mmap` of named files, no `shm_open`)
- threads (no `clone` with thread flags)
- IPC (no pipes, sockets, message queues, futexes)
- file metadata mutation (no `chmod`, `chown`, `utimes`, `rename` outside narrow build paths — and the build paths whitelist is part of `host_profile.toml`)
- recursive directory walks at language level — a host tool walks at most one directory level per call

A host tool that needs to traverse a tree expresses the traversal as bounded iteration over an explicitly-passed list of paths.

## Bounded Iteration

Every loop in a host program must carry a `#[bound = N]` annotation. The bound may be a constant or a symbolic expression in compile-time constants and a budget pulled from `host_profile.toml`. A loop without a bound is rejected. This is the same rule as boot profile, applied consistently.

## Diagnostic Codes

Host-profile-specific diagnostic codes are reserved in the `H-` prefix range:

- `H-001` — undeclared host effect
- `H-002` — host effect declared in non-host profile
- `H-003` — boot effect declared in host profile (cross-profile contamination)
- `H-004` — buffer ceiling exceeded at declaration site
- `H-005` — disallowed Linux syscall in codegen
- `H-006` — recursive directory walk at language level
- `H-007` — argv access beyond declared capacity
- `H-008` — host profile module imported from non-host profile

Codes are stable; renumbering is a breaking change to the diagnostic catalog.

## Enforcement Rule

- A program is host-profile if and only if its module declares `profile = "host"`.
- A host program may declare only effects from the host alphabet.
- A host program may import only host modules.
- A host program may emit only the syscalls listed in this document.
- A host program is rejected if it would violate any v0 rule.
- Any change to this file's effect alphabet, syscall table, or capacity ceilings is a breaking change and requires a coordinated update to [host_profile.toml](host_profile.toml), the conformance corpus, and the diagnostic catalog.

## Review Rule

Reject any change that:

- widens the host effect alphabet without a corresponding entry in `host_profile.toml`
- adds a syscall to the codegen table without a corresponding effect label
- relaxes a capacity ceiling without an explicit motivating program in the conformance suite
- introduces network, subprocess, signal, thread, or shared-memory primitives in any form
- treats the host profile as a place to add language features that boot profile does not need
