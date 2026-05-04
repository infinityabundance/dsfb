# Phosphoric

[![Open in Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-phosphoric/notebooks/dsfb_phosphoric_colab.ipynb)
[![DSFB Gray Audit: 73.3%](https://img.shields.io/badge/DSFB%20Gray%20Audit-73.3%25-green)](audit/dsfb-gray-2026-05-03T23-48-09Z/DSFB_PHOSPHORIC_GRAY_AUDIT_REPORT.md)

Phosphoric v0.3 explores a new forensic-primacy computing category: computation as typed residual evidence, bound into replayable case files and adjudicated by deterministic verdict logic. Adjacent fields include verified compilers, verified kernels, provenance systems, forensic specification languages, and deterministic replay; Phosphoric’s contribution is the integration of those concerns into a bootable, razor-scoped evidence-first substrate.


Phosphoric is a deliberately constrained stack for building tiny, auditable, capability-oriented systems for ultra-low-cost edge hardware.

> **The substrate's job is to make residual truth legible.**

The project is split into three layers:

- **Ember** — the minimal trusted machine nucleus. Owns boot, traps, page tables, context switch, and the small set of hardware-dangerous operations. Every privileged operation is justified at line precision.
- **Phosphoric** — the language surface above Ember. `no_std`, `no_alloc`, `no_unsafe`, with affine capabilities, fixed-capacity collections, and explicit declared effects on every function.
- **PhosphorOS** — the operating system layer built on Ember and Phosphoric. Fixed-capacity tables, message-passing IPC, capability-based authority, single-window GUI demo today.

## Goal

A small, defensible computing stack with hard constraints:

- `no_std`, `no_alloc`, `no_unsafe` in language programs
- Capability-oriented authority, no ambient access
- Deterministic memory behaviour and bounded loops by language rule
- Software-first enforcement on $5-class microcontrollers, with hardware protection (TrustZone-M, MPU, PMP, OTP, signed boot) as belt-and-suspenders
- Honest verification culture: every claim is traceable to a verifier or a labelled non-goal

The project occupies a narrow, defensible niche: language-enforced affine capabilities + deterministic memory + ultra-narrow trusted surface, deployed by a small team on a multi-year horizon. It is not trying to be a general-purpose systems language.

## Architectural Stance

- Ember owns the unavoidable machine-dangerous operations.
- Phosphoric owns the constrained safe surface above Ember.
- PhosphorOS owns the GUI, IPC, scheduling, and application model built on those layers.
- Capability handles are explicit and affine by default.
- Runtime memory use is fixed-capacity and deterministic.
- Failures are explicit results, not hidden traps or panic-driven control flow.

## v0 Constraints

- Single prototype target: `x86_64` on `UEFI`, booted in `QEMU`. Used as the reproducible smoke-test loop.
- Real deployment targets: $5-class MCUs — RP2040, RP2350, ESP32-C3, low-end Cortex-M, simple RISC-V with PMP.
- Software rendering first, using a linear framebuffer.
- Fixed-capacity tables, queues, buffers, and GUI objects.
- Message passing preferred over shared mutable state.
- No heap-backed runtime path.
- No hidden dynamic dispatch.
- No ambient authority.
- No unrestricted pointer arithmetic in normal language code.

## Current Development Rule

The repository is framing-first. The project starts with specification documents that define the threat model, trusted computing base, non-goals, language semantics, and safety boundary before implementation expands.

The current golden booted vertical slice consumes active `.phos` sources under `apps/demo/`, emits reviewed IR/ASM evidence artifacts, verifies them byte-for-byte against golden fixtures, and writes `BOOTX64.EFI` with a narrow project-owned PE/COFF image writer. The direct EFI image owns the UEFI entrypoint, routes one synthetic input event, renders a bounded command list, and exits deterministically under QEMU. Active verification records no external assembler/linker use and no non-Phosphoric runtime objects.

The QEMU run emits the marker line `phosphoric: entering generated boot-asm demo`, followed by per-stage progress markers and the closing `phosphoric: demo complete`.

## Reproducible Run Paths

### Google Colab

The primary external smoke/repro path is:

```text
notebooks/dsfb_phosphoric_colab.ipynb
```

The notebook starts from a fresh Colab runtime, clones
`https://github.com/infinityabundance/dsfb.git`, installs QEMU/OVMF host
dependencies, runs `make -k verify-court-active`, prints the DSFB QEMU log,
PFI hashes, verdict fixtures, release checksums, and release archive contents,
then exposes the evidence files for download.

The notebook's claim is intentionally narrow: it records that the checked
commands pass on that Colab runtime at the printed commit and QEMU/OVMF package
versions. It is not a claim of bit-identical output across arbitrary hosts.

### Local Ubuntu / Debian QEMU

From a clean host:

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  git make coreutils dosfstools mtools qemu-system-x86 ovmf

git clone https://github.com/infinityabundance/dsfb.git
cd dsfb/crates/dsfb-phosphoric

make -k verify
make -k verify-court-active
bash tools/qemu-run/run_dsfb_demo.sh
tools/release/package_release.sh
```

Important generated evidence paths:

```text
build/uefi-demo/dsfb/qemu-debug.log
build/uefi-demo/dsfb/linked-artifact.txt
build/uefi-demo/dsfb/dsfb_demo.pfi
build/release/SHA256SUMS
```

If QEMU reports a temporary file failure under read-only `/var/tmp`, see
[docs/QEMU_VVFAT_TMPDIR.md](docs/QEMU_VVFAT_TMPDIR.md). The active DSFB runner
uses a build-local FAT image and avoids QEMU `fat:rw:` vvfat mode.

## Audit Reports

The current DSFB Gray static audit artifacts are in
[audit/dsfb-gray-2026-05-03T23-48-09Z/](audit/dsfb-gray-2026-05-03T23-48-09Z/).

- [DSFB_PHOSPHORIC_GRAY_AUDIT_REPORT.md](audit/dsfb-gray-2026-05-03T23-48-09Z/DSFB_PHOSPHORIC_GRAY_AUDIT_REPORT.md) — human-readable audit summary
- [dsfb_phosphoric_scan.txt](audit/dsfb-gray-2026-05-03T23-48-09Z/dsfb_phosphoric_scan.txt) — raw `dsfb-gray` text report
- [dsfb_phosphoric_scan.sarif.json](audit/dsfb-gray-2026-05-03T23-48-09Z/dsfb_phosphoric_scan.sarif.json), [dsfb_phosphoric_scan.intoto.json](audit/dsfb-gray-2026-05-03T23-48-09Z/dsfb_phosphoric_scan.intoto.json), and [dsfb_phosphoric_scan.dsse.json](audit/dsfb-gray-2026-05-03T23-48-09Z/dsfb_phosphoric_scan.dsse.json) — machine-readable audit outputs

The badge score is an advisory source-visible review-readiness score from
`dsfb-gray`, not a certification result, runtime correctness proof, or universal
reproducibility claim. This scan reported `0` Rust source files scanned and
`26399` artifact files inspected, so function-level Rust checks should be read
as scanner coverage limits for this artifact-heavy package.

## Documentation

Public documentation lives under [docs/](docs/):

- [docs/PHOSPHORIC.md](docs/PHOSPHORIC.md) — the language: surface, semantics, type system, effects, capabilities, profiles
- [docs/EMBER.md](docs/EMBER.md) — the trusted nucleus: minimality contract, trust audit discipline, per-arch primitives
- [docs/PHOSPHOROS.md](docs/PHOSPHOROS.md) — the OS layer: capability tables, IPC, windows, framebuffer, demo loop
- [docs/COMPILER.md](docs/COMPILER.md) — the compiler: lexer, parser, type checker, codegen, host/trusted/boot/runtime profile lowering
- [docs/QEMU_VVFAT_TMPDIR.md](docs/QEMU_VVFAT_TMPDIR.md) — QEMU `fat:rw:` `/var/tmp` failure and the build-local FAT-image fix
- [docs/language/](docs/language/) — formal specifications: v0 freeze, grammar, type system, effect lattice, memory model, profile manifests
- [docs/abi.md](docs/abi.md), [docs/BOOT_ABI_V1.md](docs/BOOT_ABI_V1.md), [docs/ir.md](docs/ir.md) — boot ABI and IR specifications
- [ember/docs/](ember/docs/) — the Ember nucleus's per-module trust audit and minimality contracts
- [kernel/docs/](kernel/docs/) — the runtime kernel's per-module specifications

## Repository Layout

```
apps/demo/             — bootable demo programs (Phosphoric source)
bootstrap/             — bootstrap chain manifest and runbook
compiler/              — Phosphoric self-hosted compiler (lexer, parser, typeck, codegen)
config/                — build configuration (target budgets, profile manifests)
docs/                  — public documentation
ember/                 — Ember trusted nucleus (per-arch trusted! primitives)
kernel/                — PhosphorOS runtime kernel (capability tables, IPC, windows)
notebooks/             — external reproducibility notebooks
tests/                 — conformance suite, golden fixtures
tools/                 — verification scripts, host-profile verifiers, image builders
LICENSE                — Apache License 2.0
NOTICE                 — copyright and attribution
README.md              — this file
Makefile               — verification targets
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright 2026 Invariant Forge LLC.
