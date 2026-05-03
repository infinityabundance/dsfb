# Ember Trust Audit

This file audits every function under `ember/` that participates in the trust boundary.

The goal is not to claim every function is irreducible. The goal is to state precisely why each function exists, why it is trusted today, and whether its responsibilities can move upward into the safe surface later.

## Discipline

Every `trusted!` block in `ember/` carries two annotations:

- `// razor-rationale: irreducible-hw` — there is no equivalent above the trust boundary; the operation is a single CPU instruction or a memory-mapped register access whose semantics cannot be expressed in the safe surface.
- `// razor-rationale: language-not-expressible: <reason>` — the operation enforces something the safe-surface type system cannot describe (e.g., interrupt-flag transitions, privilege-level changes).

`check_trusted_blocks.phos` (host program) fails CI on any `trusted!` block lacking a row in this audit and a matching annotation in source.

## Trusted Operation Classes

Per [docs/language/TRUSTED_PROFILE.md](../../docs/language/TRUSTED_PROFILE.md), the trusted profile permits exactly three operation classes inside a `trusted!` block. Anything else in trusted-profile source is rejected by the compiler.

| Class | Operation | Audit row required |
|---|---|---|
| `cpu_op` | Architecturally-named CPU instructions: GDT/IDT load, interrupt-flag toggle, CSR access, MSR write, port I/O. The operand is a closed enum constant; raw assembly text is forbidden. | Yes — one per call site. |
| `mmio_*` | `mmio_read_u8/16/32/64` and `mmio_write_u8/16/32/64` against named `MmioRegion` constants from `trusted_profile.toml`. | Yes — one per call site. |
| `port_*` | `port_read_u8/16/32` and `port_write_u8/16/32` for legacy x86 port I/O. | Yes — one per call site. |

## Per-Module Trusted Surface

| Module | Purpose | Trusted operation classes used |
|---|---|---|
| [`lib.phos`](../lib.phos) | Boot scaffold types + per-frame primitives (`input_poll`, `present_frame`, `debug_log`, `qemu_exit`). | `port_*` (qemu_exit), via composition `mmio_*` (present_frame, debug_log). |
| [`mmio.phos`](../mmio.phos) | Register block descriptors. | None — pure data. |
| [`sched.phos`](../sched.phos) | Saved-context shape + critical-section enter/exit. | `cpu_op` (Cli, Sti). |
| [`traps.phos`](../traps.phos) | IDT descriptor build + install. | Composition of `arch/x86_64/descriptor_tables.load_idt`. |
| [`boot.phos`](../boot.phos) | Boot scaffold structures. | None — pure data. |
| [`arch/x86_64/descriptor_tables.phos`](../arch/x86_64/descriptor_tables.phos) | GDT/IDT load + interrupt-flag toggle. | `cpu_op` (Lgdt, Lidt, Cli, Sti). |
| [`arch/x86_64/serial.phos`](../arch/x86_64/serial.phos) | UART 16550 byte writes + status reads. | `port_*` (read_u8, write_u8). |
| [`arch/x86_64/framebuffer.phos`](../arch/x86_64/framebuffer.phos) | Linear-framebuffer pixel writes. | `mmio_write_u32`. |
| [`arch/cortex_m33/boot.phos`](../arch/cortex_m33/boot.phos) | RP2350 reset handler — OTP read, signed-boot verify, SAU configure, MPU configure. | Composition of below. |
| [`arch/cortex_m33/otp.phos`](../arch/cortex_m33/otp.phos) | OTP-burned trust-root row reads. | `mmio_read_u32`. |
| [`arch/cortex_m33/sha256.phos`](../arch/cortex_m33/sha256.phos) | RP2350 SHA-256 hardware accelerator wrapper. | `mmio_read_u32`, `mmio_write_u32`. |
| [`arch/cortex_m33/trng.phos`](../arch/cortex_m33/trng.phos) | RP2350 hardware TRNG status + word read. | `mmio_read_u32`. |
| [`arch/cortex_m33/sau.phos`](../arch/cortex_m33/sau.phos) | TrustZone-M SAU region configuration. | `mmio_write_u32`. |
| [`arch/cortex_m33/mpu.phos`](../arch/cortex_m33/mpu.phos) | ARMv8-M MPU region configuration. | `mmio_write_u32`. |
| [`arch/cortex_m0plus/boot.phos`](../arch/cortex_m0plus/boot.phos) | RP2040 reset handler — VTOR install, MPU regions. | Composition of below. |
| [`arch/cortex_m0plus/mpu.phos`](../arch/cortex_m0plus/mpu.phos) | ARMv6-M MPU region configuration. | `mmio_write_u32`. |
| [`arch/riscv_rv32imc/boot.phos`](../arch/riscv_rv32imc/boot.phos) | Low-end RISC-V reset handler — mtvec install, mstatus, PMP. | Composition of below. |
| [`arch/riscv_rv32imc/pmp.phos`](../arch/riscv_rv32imc/pmp.phos) | Physical Memory Protection entry configuration. | `cpu_op(Csrrw)`. |
| [`arch/esp32_c3/boot.phos`](../arch/esp32_c3/boot.phos) | ESP32-C3 reset handler — eFuse trust root readout. | `cpu_op(Csrrw)` (mtvec), `mmio_read_u32` (eFuse). |

## Audit Rows

Each row corresponds to one `trusted!` block in source. Format: file path → block label → operation class → razor-rationale → caller-side invariant.

### x86_64 / `arch/x86_64/descriptor_tables.phos`

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| LGDT | `cpu_op(Lgdt)` | irreducible-hw | The descriptor address must point at a 6-byte (size, base) tuple in valid memory. |
| LIDT | `cpu_op(Lidt)` | irreducible-hw | The descriptor address must point at a 6-byte (size, base) tuple in valid memory. |
| CLI | `cpu_op(Cli)` | language-not-expressible | Caller is in machine mode and is preparing to mutate state that requires interrupts disabled. |
| STI | `cpu_op(Sti)` | language-not-expressible | The state machine is in a recoverable point; interrupt re-enable is safe. |

### x86_64 / `arch/x86_64/serial.phos`

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| UART data write | `port_write_u8(base, byte)` | irreducible-hw | The base port is 0x3F8 (COM1). |
| UART LSR read | `port_read_u8(base+5)` | irreducible-hw | The base port is 0x3F8 (COM1); the line-status register is at offset +5. |

### x86_64 / `arch/x86_64/framebuffer.phos`

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| framebuffer pixel write | `mmio_write_u32(addr, pixel)` | irreducible-hw | The address falls within the linear framebuffer region whose base+stride were validated at handoff. |

### x86_64 / `sched.phos`

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| context switch CLI | `cpu_op(Cli)` | language-not-expressible: IF flag manipulation | Caller is entering a critical section and saving register state. |
| context switch STI | `cpu_op(Sti)` | language-not-expressible: IF flag manipulation | The next task's register state has been fully restored. |

### cortex_m33 / RP2350

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| OTP read | `mmio_read_u32(otp_base + offset)` | irreducible-hw | The row id is in the closed `OtpRow` enum; the offset is within the OTP controller's MMIO window. |
| SHA-256 reset | `mmio_write_u32(csr, 1)` | language-not-expressible: reset clears engine state | Caller has not begun feeding input. |
| SHA-256 feed word | `mmio_write_u32(wdata, word)` | irreducible-hw | The engine is in feed state; padded final block is observed by caller. |
| SHA-256 digest read | `mmio_read_u32(dig0 + i*4)` | irreducible-hw | The engine has signalled finalization. |
| TRNG status read | `mmio_read_u32(status)` | irreducible-hw | Engine is powered. |
| TRNG output read | `mmio_read_u32(output)` | irreducible-hw | Status read indicated ready and healthy. |
| SAU region select (RNR) | `mmio_write_u32(rnr, idx)` | irreducible-hw | The index is in `[0, 8)`. |
| SAU region base (RBAR) | `mmio_write_u32(rbar, base)` | irreducible-hw | The selected region was set via the prior RNR write. |
| SAU region limit + attr (RLAR) | `mmio_write_u32(rlar, encoding)` | irreducible-hw | Encoding combines limit and attribute bits per ARMv8-M spec. |
| MPU region select (RNR) | `mmio_write_u32(rnr, idx)` | irreducible-hw | The index is in `[0, 8)`. |
| MPU region base (RBAR) | `mmio_write_u32(rbar, base)` | irreducible-hw | The selected region was set via the prior RNR write. |
| MPU region limit + attr (RLAR) | `mmio_write_u32(rlar, encoding)` | irreducible-hw | Encoding combines limit and MAIR attribute index. |

### cortex_m0plus / RP2040

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| VTOR install | `mmio_write_u32(vtor, table)` | irreducible-hw | The vector table address is 256-byte aligned. |
| MPU region select (RNR, ARMv6-M) | `mmio_write_u32(rnr, idx)` | irreducible-hw | The index is in `[0, 8)`. |
| MPU region base (RBAR, ARMv6-M) | `mmio_write_u32(rbar, base)` | irreducible-hw | The selected region was set via the prior RNR write. |
| MPU region attr (RASR, ARMv6-M) | `mmio_write_u32(rasr, encoding)` | irreducible-hw | Encoding packs ENABLE/SIZE/AP bits per ARMv6-M spec. |

### riscv_rv32imc / low-end RV32

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| mhartid CSR read | `cpu_op(Csrrs(mhartid, 0))` | irreducible-hw | Hart is in machine mode. |
| mtvec CSR write | `cpu_op(Csrrw(mtvec, addr))` | irreducible-hw | The handler address is 4-byte aligned and direct-mode is intended. |
| mstatus CSR write | `cpu_op(Csrrw(mstatus, value))` | language-not-expressible: privilege-level transition | Hart is in machine mode and the caller is finalizing pre-runtime configuration. |
| PMP address (csrrw pmpaddr) | `cpu_op(Csrrw(pmpaddr<i>, encoded))` | irreducible-hw | The encoding matches the addressing mode declared in the matching pmpcfg byte. |
| PMP config (csrrw pmpcfg) | `cpu_op(Csrrw(pmpcfg<i/4>, word))` | irreducible-hw | The config byte position matches the entry index. |

### esp32_c3

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| eFuse read | `mmio_read_u32(efuse_base + offset)` | irreducible-hw | The block id is in the closed `EfuseBlock` enum. |
| mtvec write (ESP32-C3) | `cpu_op(Csrrw(mtvec, addr))` | irreducible-hw | Handler address is 4-byte aligned. |

### lib.phos / per-frame primitives

| Block | Operation | Rationale | Caller invariant |
|---|---|---|---|
| qemu_exit | `port_write_u8(0xF4, status)` | irreducible-hw | The QEMU debug-exit device is at port 0xF4; only invoked on the demo path. |

## Audit Result

The trusted core is intentionally narrow. Every block above is either:

- An architecturally-irreducible CPU operation (interrupt-flag toggle, descriptor table install, CSR access, port I/O), or
- A direct memory-mapped register access whose semantics cannot be expressed in the safe-surface type system.

Reducible work — input parsing, command-list composition, message routing, capability validation — lives above the trust boundary in the runtime kernel ([kernel/](../../kernel/)) and runs under the runtime profile, not the trusted profile. Adding any new `trusted!` block requires a row in this document and a matching `// razor-rationale:` annotation in source.
