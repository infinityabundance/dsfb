# Task Seal v0

This document is the canonical specification of the Phosphoric task seal — the manifest-driven authority discipline that turns a Phosphoric image into a *task-sealed computing artifact*.

A task-sealed image is one where every effect, capability, MMIO range, IPC channel, and bounded-loop ceiling is named in a manifest, the manifest is hashed into the boot image, and every layer of the stack refuses to admit anything outside the manifest. The image does not ask for permission at runtime. It has already proven, before boot, that no other permission can exist.

---

## 0. Category — DSFB-Native Deterministic Forensic Compute Model

Phosphoric is not a programming language, a kernel, or an OS. Those are implementation details. Phosphoric is a *DSFB-native deterministic forensic compute model* — an execution substrate built to emit structurally meaningful residuals at every authority boundary, so that an external structural court (dsfb-gray) can read execution as grammar rather than infer structure from noisy telemetry.

**Apex thesis.** A deterministic forensic operating system should not merely log what happened. It should emit *structurally meaningful residuals at every authority boundary.* The host system is built to speak DSFB; DSFB does not have to fight the host system.

**Core innovation — five-stage accountability chain:**

```
declared behavior
  → compiled behavior
  → boot behavior
  → runtime behavior
  → forensic replay
```

Every transition is cryptographically and structurally accountable. The combination does not meaningfully exist today in a unified stack: seL4 has formal kernel proofs, MirageOS has single-purpose deployment, Tock OS has embedded isolation, proof-carrying code has proof artifacts, Qubes has isolation. Phosphoric merges these fragments into one shape.

**Four-layer substrate:**

- **Phosphoric (the language)** prevents undeclared behavior from being expressible.
- **Ember (the trusted nucleus)** keeps the trusted machine boundary tiny.
- **PhosphorOS (the runtime)** enforces the task seal at runtime.
- **dsfb-gray (the auditor)** audits structural drift between declared, compiled, booted, and observed behavior.

**Killer sentence.** The device is not only constrained while running; it is built so that after failure, the exact boundary between permitted, observed, and impossible behavior remains reconstructable.

### 0.1 The flagship capability — forensic replay

```
phosphoric replay incident_<id>
```

When a deployed device fails, the operator extracts the manifest, the signed image, the runtime trace, the capability transitions, the IPC graph, and the dsfb-gray report. `phosphoric replay` reproduces the failure deterministically, off-device, against the sealed authority graph. This is the load-bearing demonstration of the forensic claim.

### 0.2 Terminology discipline

The doctrine does not use "most secure." It uses, in order of strength:

- **most constrained** — provable structural foreclosure of behaviors
- **most deterministic** — every execution reconstructable from inputs + manifest
- **most forensic** — post-failure boundary-reconstructability
- **most auditable** — every transition independently checkable

Each is provable. "Most secure" is not.

### 0.3 Scope — the no-list

The model is foreclosed against networking, filesystems, multi-tenant operation, package managers, cloud integration, browser support, and "developer ergonomics." Going broad on any of these would yield a weaker version of an existing system. The model targets device-class deployments where the seal is the value: defense subsystems, industrial control, aerospace, medical devices, air-gapped terminals, critical infrastructure controllers, voting systems, HSM-class devices, classified field hardware, forensic chain-of-custody environments.

### 0.4 DSFB-native residuals — the substrate emits structural grammar

A residual is the typed difference between *declared* and *observed* at one authority boundary. The substrate computes seven such residuals; each one is a fixed-shape, named, hash-pinnable object — not a log line.

| Residual | Boundary |
|---|---|
| `declared_cap_graph - observed_cap_graph` | capabilities |
| `declared_ipc_routes - observed_ipc_routes` | IPC channels |
| `budget_limit - measured_budget_use` | stack / loop / kernel-init budgets |
| `allowed_effect_closure - actual_effect_trace` | effect alphabet |
| `declared_mmio_ranges - observed_mmio_touches` | physical memory |
| `manifest_hash - image_hash` | image identity |
| `expected_task_transition - observed_task_transition` | scheduler state |

dsfb-gray reads the residual stream and emits one of eight drift classifications:

| Class | Meaning |
|---|---|
| `NO_DRIFT` | every residual = 0 |
| `AUTHORITY_EXPANSION` | observed cap/effect/route/mmio is a strict superset of declared |
| `SILENT_NARROWING` | declared > observed; the manifest grants more than the image uses |
| `IPC_ROUTE_DIVERGENCE` | observed channel id outside `[[ipc.channel]]` set |
| `MMIO_BOUNDARY_PRESSURE` | observed touch within manifest range but at the high-water edge |
| `STACK_BUDGET_PRESSURE` | measured budget use within bound but above a configurable warning fraction |
| `TASK_STATE_SLEW` | observed task transition not in the expected scheduler graph |
| `BOOT_ATTESTATION_MISMATCH` | `manifest_hash != image_hash`, or any of the eight `.pmanifest` certificates fails |

The eight `.pmanifest` certificates (§6.1) are the *static* residuals — the build-time difference between manifest and compiled image, pinned to zero. The runtime trace (§9-bound, see plan files) is the *dynamic* residual stream — the per-event difference between declared and observed. Together they make the substrate's behavior legible as DSFB grammar.

### 0.5 Trust-zone partition — the compiler is not the apex

The biggest internal risk is a too-trusted compiler. The model partitions trust across three zones; only the inner two are load-bearing for the forensic claim.

| Zone | Components | Why |
|---|---|---|
| **Untrusted** | lexer, parser, AST transforms, optimization passes | a bug here cannot violate the seal — it can only refuse a valid program or emit a wrong-but-still-boundary-checked image |
| **Trusted** | effect closure verifier, capability verifier, manifest verifier (parser + predicates), boot certificate generation | a bug here can issue a wrong certificate; this is what dsfb-gray independently re-derives |
| **Ultra trusted** | Ember primitives (boot, traps, MMIO, context entry) | a bug here breaks the substrate the seal sits on |

Formal methods are applied selectively — only to the trusted and ultra-trusted zones (capability issuance, manifest parser, MMIO bounds checking, boot hash validation). Trying to formally verify every pass is suicide; verifying the small load-bearing surface is leverage.

The task seal is the substrate that makes the forensic claim true. The manifest is the declared boundary. The `.pmanifest` certificates (§6) are the compiled evidence. The kernel runtime checks (§5) are the observation log. The five hard gates (§9) are the boundary-reconstruction harness.

---

## 1. Purpose

Phosphoric does not become safe by handling everything. It becomes safe by making almost everything impossible.

A general-purpose OS asks "what is this program allowed to do?" at runtime. A task-sealed artifact answers that question once, before boot, and forecloses every other answer.

The task seal carries this answer. Source code declares effects, capabilities, MMIO ranges, IPC channels, and budgets. The manifest pins them. The compiler, kernel, boot loader, and external auditor each refuse anything outside what the manifest names.

> **Slogan.** Not a general-purpose OS. A task-sealed computing artifact.
>
> **North star.** One task, one image, one manifest, one authority graph, one sealed device envelope.

### 1.1 The safety ladder

Runtime checks are weaker than non-existence. The doctrine drives every concern as high up the ladder as the language permits:

1. **Not in grammar** — the construct cannot be lexed or parsed.
2. **Not in type system** — the construct has no inferable type.
3. **Not in effect alphabet** — no effect label exists for the operation.
4. **Not in manifest** — even alphabet-allowed behavior is excluded by this task's seal.
5. **Not in generated image** — the compiler emits no bytes that could perform the operation.
6. **Not accepted by loader** — boot loader hashes manifest into image; mismatch aborts.
7. **Not issuable by kernel** — runtime refuses any cap/route/resource the manifest does not name.
8. **Not accepted by dsfb-gray** — external structural court refuses to sign drifted envelopes.

A property defended at rung 1 needs no defense at rung 8.

---

## 2. Forbidden by Default

The following are not features Phosphoric refuses to implement. They are features that cannot exist in a Phosphoric image.

**Heap, allocator.** No `box`, no `vec`, no `alloc` syntax. v0 has no allocator. The runtime never asks the system for memory; every collection is a fixed-capacity array sized at compile time.

**Filesystem.** No filesystem effect, no path types, no `open`/`read`/`write` against named files. The runtime alphabet has no `fs-*` label.

**Network.** No socket types, no protocol stacks, no driver framework. The runtime alphabet has no `net-*` label.

**Dynamic loading.** No `extern`, no `use`, no `import`, no runtime symbol lookup. Cross-module references are fully-qualified paths the compiler resolves once at link.

**Ambient MMIO.** No register access at an arbitrary physical address. Every memory-mapped operation passes through a trusted-profile primitive whose target falls within an MMIO range the manifest enumerates.

**Reflection.** No `typeof`, no `sizeof_value`, no runtime type introspection. Type information is compile-time only.

**Pointer types.** No raw pointers, no borrow operator, no lifetime annotations. v0 grammar rejects these tokens.

**Hidden syscalls.** Every host effect is in the closed alphabet declared by the profile manifest. The image's actual closure must be a subset of the manifest's `[effects].allowed` list.

**Undeclared IPC.** Every channel is a manifest-declared resource with a typed payload schema. Send and receive on an unnamed channel is a compile-time error.

**Undeclared effects.** Every function declares the effects its body transitively performs. The compiler verifies the closure; functions whose closure exceeds the manifest's bound are rejected.

Any feature added to this list is a doctrine change requiring a coordinated manifest schema update. The list is closed.

---

## 3. Manifest Authority

The manifest is the root of allowed existence for a Phosphoric image. It lives at `apps/<task>/task.manifest.toml`. The compiler reads it before any pass that walks declared effects or capabilities.

### 3.1 Schema

```toml
manifest_version = "phosphoric-task-seal-v0"
task_id          = "demo"
task_image_hash  = "TBD"
profile          = "boot"

[effects]
allowed = ["draw", "ipc", "sched", "time", "mmio"]

[capabilities]

[[capabilities.entry]]
kind     = "Task"
slot     = 0
purpose  = "demo task root"

[mmio]

[[mmio.range]]
name   = "framebuffer"
base   = 2684354560
length = 2097152
mode   = "rw"

[ipc]

[[ipc.channel]]
id       = "input_event_route"
sender   = "demo.boot_entry"
receiver = "demo.button_policy"
payload  = "input_event"
capacity = 8

[budgets]
task_stack_limit  = 8192
kernel_init_limit = 4096
loop_bound_max    = 65536

[forbidden]
heap                = false
filesystem          = false
network             = false
dynamic_loading     = false
ambient_mmio        = false
reflection          = false
pointer_types       = false
hidden_syscalls     = false
undeclared_ipc      = false
undeclared_effects  = false

[bootstrap]
compiler_image_hash = "TBD"
manifest_self_hash  = "TBD"
```

### 3.2 Authority chain

```
   source.phos  +  task.manifest.toml
         │              │
         ▼              ▼
   compiler reads both, rejects source declaring more than manifest permits
         │
         ▼
   boot image embeds manifest_self_hash + compiler_image_hash + budgets
         │
         ▼
   loader verifies hashes; mismatch → boot abort
         │
         ▼
   runtime refuses any cap/route/resource not in manifest
         │
         ▼
   dsfb-gray reads boot image and manifest; signs the envelope
       or refuses (post-boot drift detected)
```

The image is sealed when source ⊆ manifest ⊆ alphabet ⊆ image-hash ⊆ loader-check ⊆ runtime-check ⊆ external-attestation, and any inequality is a fail-closed boot abort.

### 3.3 Hash discipline

`manifest_self_hash` is the SHA-256 of the manifest file with the `manifest_self_hash` field itself zeroed. The compiler computes this hash and embeds it in the boot image. The loader recomputes it from the loaded image's `.pmanifest` section and refuses to launch on mismatch.

`compiler_image_hash` is the SHA-256 of the `pcc` binary used to build this image. It pins the bootstrap chain into the manifest, so an image built by a different compiler (potentially with different effect-closure rules) is detectable.

`task_image_hash` is populated at build time and is the SHA-256 of the resulting `BOOTX64.EFI`. Cross-checks against the boot artifact's manifest record.

---

## 4. Compiler Enforcement

The compiler reads the manifest before any pass that walks effects or capabilities. Diagnostic codes, all M-prefix:

| Code | Trigger |
|---|---|
| M-001 | manifest file missing for the named task |
| M-002 | manifest schema invalid (missing required key, malformed value) |
| M-003 | function declares an effect not in `[effects].allowed` |
| M-004 | function transitive closure exceeds manifest bound |
| M-005 | `trusted!` block touches MMIO range outside `[[mmio.range]]` list |
| M-006 | IPC send/recv on channel id not in `[[ipc.channel]]` |
| M-007 | manifest `[forbidden]` key absent or set non-false |
| M-008 | computed worst-case stack usage exceeds `[budgets].task_stack_limit` |
| M-009 | computed loop bound exceeds `[budgets].loop_bound_max` |
| M-010 | manifest references module path not present in source set |
| M-011 | capability slot collides with another `[[capabilities.entry]]` |
| M-012 | profile mismatch (manifest says `boot`, source declares `profile runtime;`) |

The compiler driver requires `--manifest <path>` for any `boot` or `runtime` profile compilation. Compilation without a manifest is a hard error (M-001), not a warning.

### 4.1 Pass ordering

The manifest reader runs as a new pass in `compiler/manifest.phos`. The pass order becomes:

```
lexer → parser → AST → HIR → manifest (NEW) → typeck → effects → layout → call_graph → stack_analysis → codegen
```

Manifest comes between HIR and typeck because typeck needs the manifest's capability list to validate handle issue sites (M-011), and effects needs the manifest's `[effects].allowed` list to bound the closure (M-003, M-004).

### 4.2 Conformance corpus

For every M-### code, the corpus carries one positive case (manifest accepts) and one negative case (manifest rejects with the documented code). Cases live under `tests/conformance/manifest/{positive,negative}/`.

---

## 5. Kernel Enforcement

The runtime kernel reads the manifest once at `kernel_init` from a fixed-capacity table the compiler mirrored into the image. The kernel does not allocate. The manifest table is consulted by `cap-issue`, `ipc.send`, and `ipc.recv` thereafter.

### 5.1 New `KernelError` variants

Two integers added to `kernel/error.phos` (gaps in the stable status sequence preserved per ABI discipline):

- `UndeclaredChannel = 12` — emitted by `kernel/ipc.phos` when send/recv targets a channel id not in the manifest table.
- `ManifestMismatch = 13` — emitted by `kernel/kernel.phos` `kernel_init` when the embedded `.pmanifest` section's hash does not match the recomputed hash. Fatal; halts the kernel.

### 5.2 Manifest-table data structure

```phos
struct ManifestEntry {
    kind: pcc.kernel.caps.CapabilityKind,
    slot: u16,
}

struct ChannelDecl {
    id_hash: [u8; 32],
    sender_hash: [u8; 32],
    receiver_hash: [u8; 32],
    capacity: u16,
}

struct ManifestTable {
    capabilities: [ManifestEntry; 64],
    capability_count: u16,
    channels: [ChannelDecl; 16],
    channel_count: u16,
    effects_bitset: u32,
    manifest_hash: [u8; 32],
}
```

`id_hash`, `sender_hash`, `receiver_hash` are SHA-256 of the named strings — fixed-size representation so the runtime side has no string handling. The compiler computes them from the manifest TOML at build time.

### 5.3 Per-call cross-check

Every `caps.allocate_entry` consults the manifest table; an issued kind+slot must match an entry. Every `ipc.send_message` and `ipc.recv_message` matches the channel's id_hash against the table; mismatch returns `UndeclaredChannel`.

---

## 6. Boot Enforcement

The boot image carries the manifest hash and the measured budget table in a dedicated PE section.

### 6.1 `.pmanifest` section format — proof-carrying artifact

The `.pmanifest` section is a fixed 268-byte certificate bundle. Each certificate is the SHA-256 of a deterministically-ordered serialization of the corresponding manifest field plus the compiler's computed witness. The loader recomputes each from the loaded image and compares; dsfb-gray recomputes from the published manifest and compares against the image. The image is not trusted because the compiler said so — it carries independently checkable structural evidence.

```
Offset 0x000..0x00C  header                          (12 B: magic "PMNFv1\0\0" + total_len u16 + reserved u16)
Offset 0x00C..0x02C  manifest_self_hash              (32 B, SHA-256 of manifest.toml with this field zeroed)
Offset 0x02C..0x04C  compiler_image_hash             (32 B, SHA-256 of pcc binary)
Offset 0x04C..0x06C  effect_closure_certificate      (32 B, SHA-256 of sorted effects bitset bytes)
Offset 0x06C..0x08C  capability_graph_certificate    (32 B, SHA-256 of sorted [[capabilities.entry]])
Offset 0x08C..0x0AC  mmio_range_certificate          (32 B, SHA-256 of sorted [[mmio.range]])
Offset 0x0AC..0x0CC  ipc_route_certificate           (32 B, SHA-256 of sorted [[ipc.channel]])
Offset 0x0CC..0x0EC  stack_bound_certificate         (32 B, SHA-256 of computed WCSU per entrypoint)
Offset 0x0EC..0x10C  loop_bound_certificate          (32 B, SHA-256 of computed loop bounds per for-stmt)
```

Total: 268 bytes. The PE writer at `tools/phosphoric/write_boot_efi_from_ir.sh` allocates this section after `.text` and `.data`. The section is read-only; modification post-build is detectable via certificate mismatch.

The eight certificates collectively reconstruct the full authority graph of the image. Any single-byte change to the manifest or to any compiled witness propagates to exactly one certificate, so the failure isolates to which boundary moved.

### 6.2 Loader preamble

`apps/demo/boot_entry.phos`'s entrypoint does, before any draw call:

1. Read `.pmanifest` from the loaded image's known offset.
2. Recompute `manifest_self_hash` from the manifest file embedded as a `.rodata` blob.
3. Compare against `.pmanifest` byte 0x00..0x20.
4. On mismatch: `qemu_exit(2)` (or arch-equivalent halt).
5. Verify the measured budget table against compile-time WCSU markers in the image. Mismatch: halt.
6. Proceed to `kernel_main`.

### 6.3 PE writer audit

The PE writer's section emit is line-audited in `tools/phosphoric/PE_WRITER_TRUST_AUDIT.md`. The new `.pmanifest` emit gets one row per byte-region (manifest_self_hash, compiler_image_hash, budget table). Without the audit row, `make verify` fails.

---

## 7. dsfb-gray Enforcement

dsfb-gray is the external structural auditor. It is not part of the Phosphoric build path; it consumes Phosphoric's published artifacts.

### 7.1 Phosphoric publishes

For every shipped image:

1. `task.manifest.toml` — the manifest as built.
2. The boot image (`BOOTX64.EFI` or arch-equivalent).
3. A per-checkpoint attestation file emitted by `tools/phosphoric-host/phosphoric_attest.phos`. Each checkpoint carries one of four reason labels:
   - `enforced` — a verifier ran and passed
   - `feature-absent` — the feature is forbidden by manifest, so the checkpoint is trivially satisfied
   - `not-applicable-v0` — the checkpoint exercises a v0 non-goal
   - `project-practice` — governance discipline; not a language attribute

### 7.2 dsfb-gray asserts

- The manifest's `[effects].allowed` is a subset of the v0 closed alphabet.
- The image's `.pmanifest` section's `manifest_self_hash` equals the recomputed hash of the published manifest.
- No symbol exported by the image references an effect, capability, channel, or MMIO range outside the manifest.
- Every per-checkpoint attestation reason is honestly labeled — feature-absent claims must correspond to manifest entries marked `false` in `[forbidden]`, not to features the image silently omits without declaration.

### 7.3 Refusal to sign

dsfb-gray refuses to sign the envelope if any check fails. Refusal is silent at the dsfb-gray layer (it does not produce a signature); the project's downstream consumers detect the unsigned envelope.

---

## 8. Claim Boundary

Phosphoric does **not** claim to be more formally proven than any system with a machine-checked refinement proof. The well-known systems with such proofs (seL4 most notably) compete on an axis Phosphoric does not enter.

Phosphoric claims to be **more radically task-constrained by construction** than systems on this list (best of the author's knowledge):

| Property | Other systems | Phosphoric |
|---|---|---|
| Boot-time-fixed authority graph | partial or absent | yes (manifest-pinned, hash-verified) |
| Single-language stack from boot to app | mixed languages typical | yes (`.phos` only) |
| Manifest-hashed image with loader verify | rare or absent | yes (`.pmanifest` section) |
| Closed effect alphabet per profile | open or implicit | yes (4 profiles, closed) |
| Trusted nucleus LOC | thousands | hundreds |

These are different axes from formal refinement. Both are honest claims when stated separately.

The single load-bearing weakness this framing acknowledges: Phosphoric does not have a machine-checked theorem prover behind any of its claims. The chain is honest but it is not refinement. dsfb-gray's external structural check is the closest the project comes to outside attestation, and it is itself a single-author tool.

This is a real limit. The doctrine does not paper over it; it competes on a different axis instead.

---

## 9. Five Hard Gates — the forensic-boundary harness

The doctrine requires `make verify` to pass five gates. Each is a forensic check answering the question "does the observed boundary equal the declared boundary at this layer?" Failure of any one is a hard error.

| Gate | Asserts | Forensic role |
|---|---|---|
| `verify-no-excess-grammar` | the lexer + parser accept no token sequence outside the v0 grammar | nothing was *expressible* outside v0 |
| `verify-effect-closure-subset-of-manifest` | every function's transitive effect closure ⊆ `[effects].allowed` | nothing was *compileable* outside the manifest's effect alphabet |
| `verify-capability-graph-exact` | declared authority graph **==** compiled graph **==** boot graph **==** observed graph | declared and compiled graphs are identical (equality, not subset) |
| `verify-boot-image-manifest-hash` | the eight `.pmanifest` certificates round-trip-reproduce from the manifest source | image's compiled evidence equals manifest-derived evidence |
| `verify-runtime-trace-subset-of-seal` | observed runtime trace (cap-issues, ipc.send/recv, mmio touches) ⊆ manifest authority graph | observed graph ⊆ declared graph (the post-mortem invariant) |

The strongest is `verify-capability-graph-exact`: it demands equality at every transition; subset would permit silent narrowing the manifest does not sanction.

The forensic claim is testable: deliberately corrupt one byte of `apps/<task>/task.manifest.toml`'s effect set; the gates that fail isolate which layer the corruption hit. `verify-effect-closure-subset-of-manifest` catches the declared-boundary move; `verify-boot-image-manifest-hash` catches the certificate mismatch (compiled evidence still on the old manifest); `verify-runtime-trace-subset-of-seal` catches the runtime-trace divergence (observed boundary unchanged but declared boundary moved). The set of failing gates *is* the post-mortem reconstruction.

---

## See Also

- [docs/PHOSPHORIC.md](PHOSPHORIC.md) — language reference
- [docs/EMBER.md](EMBER.md) — trusted nucleus
- [docs/PHOSPHOROS.md](PHOSPHOROS.md) — runtime layer
- [docs/COMPILER.md](COMPILER.md) — compiler pipeline
- [docs/language/V0_FREEZE.md](language/V0_FREEZE.md) — frozen v0 surface
- [apps/demo/task.manifest.toml](../apps/demo/task.manifest.toml) — first manifest
- [bootstrap/STAGE0.md](../bootstrap/STAGE0.md) — bootstrap chain
