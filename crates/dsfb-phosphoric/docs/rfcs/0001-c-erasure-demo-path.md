# RFC 0001: Bootstrap Closure Of The Demo Path

## Purpose

Define the staged removal of handwritten C from the booted demo path without widening the frozen v0 language surface.

## Current State

- `apps/demo/button_app.c` is gone.
- `ember/boot/uefi_demo.c` is gone from the booted path.
- The current EFI image now enters through generated `boot-asm-v1`.
- Post-firmware demo state, routing, and redraw for the smoke path live in the generated BootAsm artifact.
- Active `.phos` files under `apps/demo/` are lowered by the narrow active boot-profile emitter into reviewed IR/ASM artifacts before linking.
- The legacy `demo-v1` C backend remains archive-only material.

## No-non-Phosphoric Boot-Layer Milestone

This milestone is complete when:

- the EFI image enters through generated BootAsm
- the booted path no longer depends on handwritten C
- the booted path links no non-Phosphoric runtime object
- active verification records no non-Phosphoric runtime objects, `c_objects=none`, and `archive_executed=false`
- generated IR/ASM are byte-clean against reviewed fixtures before assembly
- the generated BootAsm markers remain visible in the QEMU log

## Remaining Gap After C Removal

- The boot image at v0.1 was **Phosphoric-specified with HOST_REFERENCE emission**. At v0.2 (2026-05-03) the claim was promoted to **Phosphoric-compiled boot image**: `pcc-stage2.bin` lowers the constant-providing apps/demo sources byte-equal, and the shell emitter has been retired from the active build path.
- The boot layer is still trusted generated BootAsm plus the narrow direct PE/COFF image writer/parser.
- The active Phosphoric slice is still narrow and profile-driven rather than a general runtime generator.
- Self-hosting is still outside v0 and remains a post-v0 strategy target.

## Non-Goals

- no change to the frozen v0 grammar
- no general FFI surface in application code
- no borrow or region syntax
- no verified-linker work in this RFC

## Current Acceptance Signals

- active `.phos` files under `apps/demo/` emit reviewed IR/ASM via `tools/phosphoric/emit_boot_demo_from_phos.sh`
- the QEMU log includes `phosphoric: entering generated boot-asm demo`
- the QEMU log includes `phosphoric: generated boot-asm demo runtime active`
- the demo still passes `tools/qemu-run/run_uefi_demo.sh`
- `STATUS.md`, `CLAIMS.md`, and `docs/invariant_manifest.toml` reflect the removal of handwritten C from the booted path
