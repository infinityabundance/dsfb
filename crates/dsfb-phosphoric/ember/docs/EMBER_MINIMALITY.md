# Ember Minimality

This document records the irreducible trusted operations that belong in `Ember`. Everything else is rejected from the trust boundary.

## The Reduction Rule

Reject any addition to `Ember` unless it fits one of these categories:

- Firmware entry — the single entrypoint the firmware calls before higher-level isolation exists.
- Privileged CPU or trap boundary work — interrupt-flag toggling, descriptor table loads, CSR access, privilege-level transitions.
- Raw MMIO or raw port I/O — direct memory-mapped or port-mapped register access against architecturally-named regions.
- Validated hardware handoff — early-boot data structures whose shape must be checked before use.
- Typed boundary descriptors — fixed-data shapes that prevent raw address or width mistakes from leaking upward.

If an operation is pure policy, GUI logic, message routing, or ordinary rendering, it does not belong in `Ember`.

## Irreducible Trusted Operations

| Operation | Why it exists | Why it must remain trusted |
|---|---|---|
| Firmware entry | The firmware must call exactly one entrypoint. | Control transfers into the system here before any isolation can be enforced. |
| Framebuffer handoff validation | The boot path must reject invalid or unsupported framebuffer metadata before any pixel is written. | Incorrect framebuffer metadata can corrupt memory or produce undefined device behaviour. |
| Serial / debug output | The diagnostics channel runs before any console service exists. | Raw port I/O is machine-dangerous and cannot be exposed as a convenience API above the boundary. |
| Reset and bring-up sequencing | The hart must install vector tables, MPU/PMP regions, signed-boot verification, and protection-unit configuration before the safe surface is reachable. | Each step depends on hardware-specific control register writes that have no equivalent above the boundary. |
| Trap and IDT installation | Interrupt vectors must be installed in architecturally-mandated formats. | The CPU's own state machine reads these structures; misformatting causes hard faults. |
| Critical-section primitives | Context switching and trap handlers need to disable and re-enable interrupts atomically. | The interrupt-enable bit lives in CPU flags outside the safe-surface type system. |
| MMIO shape descriptors | Hardware register blocks need typed descriptions so the safe surface can refer to them by name. | Register-shape descriptions stop raw address and width mistakes from leaking upward. |
| Per-arch security primitives | Each named arch (x86_64, cortex_m33, cortex_m0plus, riscv_rv32imc, esp32_c3) has its own privileged operation set. | The privileged operations are architecturally distinct; abstracting them would either leak unsafe details or hide capabilities the safe surface needs to reason about. |
| Demo-path debug exit | The QEMU smoke test needs deterministic process exit to prove success and failure states. | The exit mechanism uses a raw I/O port and is only valid in the trusted machine layer; restricted to the demo path by `// razor-rationale:` annotation. |

## Explicitly Not in Ember

These responsibilities live above the trust boundary, in the runtime kernel under [kernel/](../../kernel/):

- Capability table allocation, validation, revocation
- Task scheduling decisions and IPC routing
- Window damage tracking and render-command composition
- Input event dispatch
- Demo state machines

The runtime kernel runs under the runtime profile (see [docs/language/RUNTIME_PROFILE.md](../../docs/language/RUNTIME_PROFILE.md)). It cannot emit `trusted!` blocks; the compiler rejects them with diagnostic Z-001.

## Per-Block Audit Coverage

Every `trusted!` block in `ember/` is enumerated in [EMBER_TRUST_AUDIT.md](EMBER_TRUST_AUDIT.md) with its operation class, razor-rationale, and caller-side invariant. The host program `check_trusted_blocks.phos` fails CI on any block missing a row.

## Discipline Statement

Ember is small by construction, not by accident. Every byte added to `ember/` increases the trust surface a reviewer must read. The reduction rule is the forcing function that keeps the trust surface bounded; the per-block audit is the evidence that the rule was followed.
