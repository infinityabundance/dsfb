# Ember — Trusted Machine Nucleus Reference

Ember is the trusted machine nucleus underneath Phosphoric. It owns the small, precisely-named set of operations that interact with hardware directly, and nothing else. Every line of `ember/` source is justified at the line of the operation it performs.

This document describes Ember's architecture, its discipline, the per-arch primitives it provides, and the contract it presents to the layers above.

---

## 1. The Reduction Rule

Ember's defining principle is the **reduction rule**: an operation belongs in `ember/` if and only if it fits one of these categories:

- Firmware entry — the single entrypoint the firmware calls before higher-level isolation can be enforced.
- Privileged CPU or trap boundary work — interrupt-flag toggling, descriptor table loads, CSR access, privilege-level transitions.
- Raw MMIO or raw port I/O — direct memory-mapped or port-mapped register access against architecturally-named regions.
- Validated hardware handoff — early-boot data structures whose shape must be checked before use.
- Typed boundary descriptors — fixed-data shapes that prevent raw address or width mistakes from leaking upward.

Anything that is pure policy, GUI logic, message routing, or ordinary rendering does not belong in Ember.

The reduction rule is enforced by `check_razor_justification.phos` (host program): every `trusted!` block in `ember/` must carry one of two annotations:

```
// razor-rationale: irreducible-hw
```

or

```
// razor-rationale: language-not-expressible: <reason>
```

Blocks lacking the annotation are rejected with diagnostic Z-001.

---

## 2. The Trusted Profile

Ember source compiles under the **trusted profile** (see [docs/language/TRUSTED_PROFILE.md](language/TRUSTED_PROFILE.md)). The profile permits exactly three operation classes inside a `trusted!` block:

| Class | Operations |
|---|---|
| `cpu_op` | Architecturally-named CPU instructions: GDT/IDT load, interrupt-flag toggle, CSR access, MSR write, port I/O. The operand is a closed enum constant; raw assembly text is forbidden. |
| `mmio_*` | `mmio_read_u8/16/32/64` and `mmio_write_u8/16/32/64` against named `MmioRegion` constants from `trusted_profile.toml`. |
| `port_*` | `port_read_u8/16/32` and `port_write_u8/16/32` for legacy x86 port I/O. |

Anything else inside a `trusted!` block — function calls, arithmetic, control flow — is the safe-surface language. The privileged operations sit at named call-site boundaries.

Trusted-profile source can use all of v0's primitive types, structs, enums, match expressions, bounded loops. It cannot allocate, cannot loop unboundedly, cannot recurse.

The compiler refuses to emit `trusted!` from non-trusted-profile source (Z-001). The runtime kernel and host tools cannot bypass Ember; they call into Ember through typed signatures.

---

## 3. The Trust Audit

Every `trusted!` block in `ember/` has a row in [ember/docs/EMBER_TRUST_AUDIT.md](../ember/docs/EMBER_TRUST_AUDIT.md). Each row names:

- The block's location (file + label).
- The operation class.
- The razor-rationale category.
- The caller-side invariant the block assumes — what the calling code must guarantee for the block to be safe.

The host program `check_trusted_blocks.phos` walks every `trusted!` block and asserts a matching audit row exists. CI fails on any block lacking an audit entry.

This is the discipline that keeps Ember small. Each audit row is read by reviewers; reviewers can refuse to merge a new block if its rationale is weak. The audit doc is the project's single source of truth for what privileged operations exist and why.

---

## 4. Per-Arch Architecture

Ember supports five named hardware targets:

| Target | Class | Why included |
|---|---|---|
| `x86_64` | Development & QEMU smoke test | Reproducible CI loop. UEFI firmware-supplied reset; Ember provides post-reset operations only. |
| `cortex_m33` (RP2350) | Embedded — full security primitives | TrustZone-M, OTP, signed boot, SHA-256 accel, TRNG, MPU, SAU. The richest privileged surface. |
| `cortex_m0plus` (RP2040) | Embedded — bare-MPU silicon | Proves the razor scales down to the canonical $5 MCU without TrustZone. MPU is the only protection unit. |
| `riscv_rv32imc` | Open-ISA RISC-V with PMP | CSR-based privileged access; PMP is the protection unit. |
| `esp32_c3` | Most exposed — eFuse trust root only | No MPU, no PMP. Protection rests entirely on language enforcement plus eFuse-burned trust roots. |

Ember source tree:

```
ember/
├── lib.phos                        — top-level scaffold + per-frame primitives
├── boot.phos                       — generic boot scaffold types
├── mmio.phos                       — register-block descriptors
├── sched.phos                      — context-switch boundary + critical sections
├── traps.phos                      — IDT install scaffolding
└── arch/
    ├── x86_64/
    │   ├── descriptor_tables.phos  — GDT/IDT load, CLI/STI
    │   ├── serial.phos             — UART 16550 byte writes
    │   └── framebuffer.phos        — linear-framebuffer pixel writes
    ├── cortex_m33/
    │   ├── boot.phos               — RP2350 reset handler
    │   ├── otp.phos                — OTP-burned trust root
    │   ├── sha256.phos             — hardware SHA-256 engine
    │   ├── trng.phos               — hardware true RNG
    │   ├── sau.phos                — TrustZone-M region configuration
    │   └── mpu.phos                — ARMv8-M MPU configuration
    ├── cortex_m0plus/
    │   ├── boot.phos               — RP2040 reset handler
    │   └── mpu.phos                — ARMv6-M MPU configuration
    ├── riscv_rv32imc/
    │   ├── boot.phos               — low-end RV32 reset handler
    │   └── pmp.phos                — Physical Memory Protection
    └── esp32_c3/
        └── boot.phos               — ESP32-C3 reset + eFuse readout
```

Each per-arch subtree is self-contained. The compiler refuses to mix architectures (T-001).

---

## 5. The Reset Path

Each named hardware target has a `reset_handler` function in its `boot.phos`. The handler is the first Phosphoric code that runs after firmware/ROM hands off control. Its responsibilities:

| Stage | x86_64 | cortex_m33 (RP2350) | cortex_m0plus (RP2040) | riscv_rv32imc | esp32_c3 |
|---|---|---|---|---|---|
| 1 | UEFI firmware-supplied | Read OTP signed-boot key hash | Install vector table (VTOR) | Read mhartid | Install trap vector (mtvec) |
| 2 | (firmware checks signature) | Verify image signature | Configure MPU stack guard | Install mtvec | Verify secure-boot enable |
| 3 | (firmware sets up GDT/IDT) | Configure SAU regions | Configure flash RO MPU region | Configure mstatus | Read trust-root key digest |
| 4 | (firmware enables interrupts) | Configure MPU regions | Enable MPU | Configure PMP entry 0 | Hand off `Esp32c3BootInfo` |
| 5 | Hand off to runtime | Hand off `Rp2350BootInfo` | Hand off `Rp2040BootInfo` | Hand off `RiscvBootInfo` | — |

Each stage's `trusted!` block has a row in [ember/docs/EMBER_TRUST_AUDIT.md](../ember/docs/EMBER_TRUST_AUDIT.md). Each block carries `// razor-rationale:` annotation. The handoff structure is a typed `BootInfo` shape that the runtime kernel consumes.

---

## 6. Per-Frame Primitives

After reset, Ember exposes a small set of per-frame primitives that the runtime kernel calls into during the boot loop:

| Primitive | Signature | Purpose |
|---|---|---|
| `input_poll` | `() -> Option[InputEvent]` | Poll the input controller for one queued event. None when no event ready. |
| `present_frame` | `(commands: RenderCommandList)` | Apply a bounded list of render commands to the framebuffer. |
| `debug_log` | `(message: Slice[u8, 256], len: u16)` | Emit a debug message via the serial console. |
| `qemu_exit` | `(status: u8)` | Demo-only: terminate QEMU via the debug-exit port. |

Each primitive has a `trusted!` block (or a composition of them) at its leaves. The runtime kernel calls these by typed signature; the `trusted!` blocks are statically resolved by the linker.

---

## 7. The Trust Boundary Crossing

The kernel calls into Ember through the typed signatures declared in [kernel/abi.phos](../kernel/abi.phos). The five crossings in the v0 surface:

| Operation | Direction | Notes |
|---|---|---|
| `BootHandoff` | Ember → Kernel | Single typed `BootInfo` parameter at startup |
| `PresentFrame` | Kernel → Ember | Bounded `RenderCommandList` |
| `DebugLog` | Kernel → Ember | Bounded byte slice |
| `InputPoll` | Kernel → Ember | Returns `Option[InputEvent]` |
| `QemuExit` | Kernel → Ember | Demo-only path |

This enumeration is the auditable contract; the typed ABI in `kernel/abi.phos` is the single source of truth. (The previous `kernel/boundary.phos` descriptive enum was recessed in Phase 7 Tranche B — orphaned descriptive code is doc, and lives in markdown.)

---

## 8. The Minimality Discipline

[ember/docs/EMBER_MINIMALITY.md](../ember/docs/EMBER_MINIMALITY.md) is the canonical statement of what belongs and does not belong in Ember. The discipline is enforced by:

- The reduction rule (above).
- Per-block razor-rationale annotation (Z-001 on missing).
- Per-block trust-audit row (CI fail on missing).
- TCB LOC budget (per-arch ceiling enforced by `check_tcb_budget.phos`).

When a new privileged operation is added:

1. The `trusted!` block carries a `// razor-rationale:` annotation.
2. A row is added to `EMBER_TRUST_AUDIT.md` describing the block's caller-side invariant.
3. `check_trusted_blocks.phos` and `check_razor_justification.phos` still pass.
4. The per-arch LOC budget is not exceeded.

A reviewer reads the audit row and decides whether the operation truly belongs at the trust boundary, or whether it can move upward into the safe surface.

---

## 8a. SHA-256 hardware acceleration on cortex_m33

The RP2350 carries an on-chip SHA-256 engine. [ember/arch/cortex_m33/sha256.phos](../ember/arch/cortex_m33/sha256.phos) wraps the engine with a public hash entrypoint:

- `sha256_hash_buffer(buf: Slice[u8, 65536], len: u32) -> Sha256Digest` — walks the input in 4-byte big-endian groups, applies FIPS 180-4 §5.1.1 padding (0x80, zeros to mod 64 == 56, 64-bit big-endian length), then reads the engine's 8-word digest.
- `sha256_hash_matches(buf, len, expected) -> bool` — convenience comparator for golden-fixture verification.

**Performance:** the RP2350 engine completes a 256-byte block in ~80 cycles at 150 MHz (~0.5 µs). Software SHA-256 over the same input takes >50 µs. Golden-fixture verification (E5 corpus comparison) becomes millisecond-class on the MCU instead of multi-second.

The `host-hash` effect lowering dispatches arch-specifically: software SHA on x86_64 ELF; hardware engine via `sha256_hash_buffer` on cortex_m33. Same API surface; dispatch chosen by target arch. Other arches (cortex_m0plus, riscv_rv32imc) currently fall back to software SHA — adding hardware acceleration is a future arch-specific advance.

## 9. Hardware-Features Manifest

[docs/language/hardware_features.toml](language/hardware_features.toml) catalogues per-arch capabilities — which arch has MPU, PMP, SAU, OTP, signed-boot, SHA-256 accel, TRNG, watchdog, RTC, etc.

The compiler reads this manifest and refuses to emit a `trusted!` primitive for a feature unavailable on the target arch (Z-002). Attempting `verify_image_signature` for cortex_m0plus, for example, fails compilation because RP2040 has no signed-boot.

---

## 10. The Stage 0 Bootstrap

Ember source is itself compiled by `pcc`. The first time `pcc` is built, it requires a stage 0 binary — a one-time-only externally-pinned bootstrap binary. The full bootstrap chain is documented in [bootstrap/STAGE0.md](../bootstrap/STAGE0.md) and the runbook is [bootstrap/STAGE0_BUILD.md](../bootstrap/STAGE0_BUILD.md).

The stage 0 binary's hash is pinned in [bootstrap/bootstrap.toml](../bootstrap/bootstrap.toml). Multiple independent attesters reproduce the build per the runbook, sign the resulting hash with detached GPG signatures, and submit attestations to the manifest. Trust in stage 0 is residual but explicit.

After stage 0, the compiler is self-perpetuating: stage1 compiles `pcc.phos` to produce stage2; stage2 compiles to produce stage3; `verify_fixpoint.phos` asserts byte-equality at the fixpoint.

---

## 11. What Ember Is Not

- Not a microkernel. Ember does not schedule tasks, route messages, or own runtime state. The runtime kernel ([kernel/](../kernel/)) does that under the runtime profile.
- Not a hardware abstraction layer. Ember does not present a uniform interface across architectures; it exposes each architecture's privileged operations as they are.
- Not a driver framework. Drivers, if they exist, live above Ember in the runtime profile or in application code.
- Not a runtime library. Ember has no `printf`, no allocator, no collections beyond fixed-capacity arrays.

The narrowness is the point.

---

## See Also

- [ember/docs/EMBER_TRUST_AUDIT.md](../ember/docs/EMBER_TRUST_AUDIT.md) — per-block trust audit
- [ember/docs/EMBER_MINIMALITY.md](../ember/docs/EMBER_MINIMALITY.md) — minimality discipline
- [ember/docs/architecture.md](../ember/docs/architecture.md) — internal architecture
- [ember/docs/safety_boundary.md](../ember/docs/safety_boundary.md) — safety boundary contract
- [ember/docs/mmio_model.md](../ember/docs/mmio_model.md) — MMIO descriptor model
- [docs/language/TRUSTED_PROFILE.md](language/TRUSTED_PROFILE.md) — trusted profile manifest
- [docs/language/hardware_features.toml](language/hardware_features.toml) — per-arch feature matrix
- [docs/PHOSPHORIC.md](PHOSPHORIC.md) — the language reference
- [docs/PHOSPHOROS.md](PHOSPHOROS.md) — the OS layer
