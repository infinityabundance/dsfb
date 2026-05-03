# Phosphoric Trusted Profile

This document defines the **trusted profile**: the variant of the frozen v0 surface used for [Ember](../../ember/), the named hardware trust boundary. Trusted-profile source is the *only* place in the project where machine-dangerous operations are permitted — MMIO, control-register access, GDT/IDT loading, port I/O, MSR access. Every such operation is an explicit `trusted!` block over a small, named enum of operations. There is no inline assembly, no string-built opcode, and no general escape hatch.

The trusted profile exists for one reason: the project rule is **only Phosphoric code in active verification**, but a kernel must talk to the machine. The trusted profile is the smallest extension to v0 that makes this possible while keeping the trust boundary line-by-line auditable.

## Relationship To v0

The trusted profile is a strict superset of [V0_FREEZE.md](V0_FREEZE.md) for *language form*. It adds exactly two things: the `trusted!` block and the `cpu_op` / `mmio_op` / `port_op` primitives inside it.

Inheriting unchanged from v0:

- top-level forms, types, control flow, ownership, capability handling, result-style failures
- everything in the v0 forbidden list ([V0_FREEZE.md](V0_FREEZE.md) §Forbidden In v0)
- everything in v0 explicitly-not-in-the-frozen-surface

`no_alloc` continues to apply. `no_std` continues to apply. `no_unsafe` is replaced by the narrower rule: **no unsafe outside `trusted!` blocks; every `trusted!` block is line-audited**.

## File Path Allowlist

Trusted-profile source is permitted only under the path:

- `ember/` (and subdirectories)

A `.phos` file under any other path that declares `profile = "trusted"` is rejected by the compiler. The path check is part of the front-end, before any other pass. A file that declares `profile = "trusted"` and lives outside the allowlist produces diagnostic `T-001`.

The bootstrap chain (`pcc.phos` and stage 0) is not permitted to compile trusted-profile source from outside the allowlist even if a developer hand-edits the path table. The allowlist lives in [trusted_profile.toml](trusted_profile.toml) and the `pcc.phos` parser reads it once and refuses to run if the file is missing or malformed.

## The `trusted!` Block

A `trusted!` block is the only construct in Phosphoric that can issue a machine-dangerous primitive. Syntax:

```phos
trusted! {
    cpu_op(CpuOp::Cli);
    let val: u8 = mmio_read8(MmioRegion::SerialPort, 0x3FD);
    mmio_write32(MmioRegion::ApicBase, 0xF0, 0x000001FF);
}
```

Rules:

- A `trusted!` block contains *only* calls to the three primitive families (`cpu_op`, `mmio_*`, `port_*`). It does not contain general expressions, function calls, or control flow.
- Every primitive argument is an enum constant (for the operation selector) or a constant or local of the declared integer type (for offsets and values). No string-built opcodes; no runtime-computed operation selectors.
- A `trusted!` block does not return a value. To produce a value from a `trusted!` block, declare a local before the block, write into it inside the block, and read it after.
- A function may contain at most one `trusted!` block. A function that needs two operations issues them in a single block.
- A `trusted!` block must be preceded by a `// trust-audit:` comment that names the audit doc section it corresponds to. The cross-reference checker rejects unaudited blocks.

## Multi-arch trusted profile (Ultra-Razor extension)

The trusted profile is **arch-parametric**. The five supported architectures (x86_64 dev, cortex_m33 RP2350, cortex_m0plus RP2040, riscv_rv32imc, esp32_c3) each have their own primitive tables in [trusted_profile.toml](trusted_profile.toml) under `[arch.<name>.cpu_op]` and `[arch.<name>.mmio_region]`. The compiler reads the build-config `arch` field and refuses to emit a primitive whose variant is not in the target arch's table:

- `T-009` — cpu_op variant not in target arch's table (e.g., trying `Cli` on cortex_m33)
- `T-010` — primitive uses arch feature absent from [hardware_features.toml](hardware_features.toml)

The five arches are a closed set. Adding a sixth requires coordinated edits to `trusted_profile.toml`, `hardware_features.toml`, `tcb_budget.toml`, and the conformance corpus per the elevation plan's E21 contract.

## Permitted Operations

Three operation families. Each is a closed enum; new entries require coordinated edits to [trusted_profile.toml](trusted_profile.toml), `pcc.phos`, and the per-arch trust audit doc named in `[audit.per_arch]`.

### `cpu_op`

Whitelisted CPU control-register and flag operations:

| Variant | Effect |
|---|---|
| `Cli` | Clear interrupt flag |
| `Sti` | Set interrupt flag |
| `Hlt` | Halt until next interrupt |
| `Lgdt(addr: u64)` | Load global descriptor table |
| `Lidt(addr: u64)` | Load interrupt descriptor table |
| `LtrSelector(sel: u16)` | Load task register |
| `WriteCr0(val: u64)` / `ReadCr0` | CR0 access |
| `WriteCr3(val: u64)` / `ReadCr3` | CR3 (page-table base) access |
| `WriteCr4(val: u64)` / `ReadCr4` | CR4 access |
| `Wrmsr(idx: u32, val: u64)` / `Rdmsr(idx: u32)` | MSR access — index restricted to a closed set in [trusted_profile.toml](trusted_profile.toml) |
| `Invlpg(addr: u64)` | Invalidate TLB entry |
| `Wbinvd` | Write back and invalidate cache |

### `mmio_*`

Memory-mapped I/O reads and writes against named regions:

| Primitive | Signature |
|---|---|
| `mmio_read8` / `mmio_read16` / `mmio_read32` / `mmio_read64` | `(region: MmioRegion, offset: u32) -> uN` |
| `mmio_write8` / `mmio_write16` / `mmio_write32` / `mmio_write64` | `(region: MmioRegion, offset: u32, val: uN)` |

`MmioRegion` is a closed enum of named hardware regions: `ApicBase`, `IoApic`, `SerialPort`, `Framebuffer`, etc. Each region's physical base address is declared in [trusted_profile.toml](trusted_profile.toml) and validated at boot time against firmware-provided memory maps. The compiler refuses to emit MMIO against a region not in the manifest.

### `port_*`

Legacy x86 port I/O:

| Primitive | Signature |
|---|---|
| `port_read8` / `port_read16` / `port_read32` | `(port: u16) -> uN` |
| `port_write8` / `port_write16` / `port_write32` | `(port: u16, val: uN)` |

Port numbers are not enum-restricted, but the compiler emits a warning `T-W-001` for any port outside the closed set in [trusted_profile.toml](trusted_profile.toml). Warnings are gated to errors in CI.

## Forbidden Even In Trusted Profile

- inline assembly text
- raw pointer arithmetic
- raw memory reads outside `mmio_*`
- string-built opcodes
- runtime-computed `cpu_op` / `MmioRegion` selectors
- recursive functions
- heap operations
- interior mutability
- atomic operations (Ember is a single-CPU boot path; SMP is not in scope)
- floating-point operations

## Audit Discipline

Every `trusted!` block is named in [ember/docs/EMBER_TRUST_AUDIT.md](../../ember/docs/EMBER_TRUST_AUDIT.md). The audit doc records, per block:

- file path and line range
- the block's purpose in one sentence
- the caller-side invariant the block assumes
- the post-condition the block establishes
- the aliasing claim (what physical memory or register state this block touches and what it does *not* touch)

`check_trusted_blocks.phos` (host program, lands with E0e) walks every `trusted!` block in the trusted-profile source tree and asserts each has a matching audit-doc section. Unmatched blocks fail `make verify-legendary`.

## Diagnostic Codes

Trusted-profile-specific codes use the `T-` prefix:

- `T-001` — trusted profile declared outside file-path allowlist
- `T-002` — `trusted!` block contains a non-primitive expression
- `T-003` — runtime-computed operation selector in `trusted!`
- `T-004` — MMIO region not in trusted_profile.toml manifest
- `T-005` — MSR index outside closed set
- `T-006` — `trusted!` block missing trust-audit comment
- `T-007` — `trusted!` block not cross-referenced in audit doc
- `T-008` — multiple `trusted!` blocks in one function
- `T-W-001` — port number outside whitelisted set (warning, gated to error in CI)

## Enforcement Rule

- A `.phos` file is trusted-profile if and only if it declares `profile = "trusted"` and lives under `ember/`.
- A trusted-profile file may issue machine-dangerous primitives only inside a `trusted!` block with a matching trust-audit comment.
- Every operation is a closed enum value; no runtime-built operations.
- Every `trusted!` block is named in the audit doc.
- Adding a `cpu_op`, `MmioRegion`, or MSR index requires coordinated edits to [trusted_profile.toml](trusted_profile.toml), `pcc.phos`, and the audit doc — in the same commit.
