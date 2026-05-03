# Security Assumptions

This document lists the assumptions the current project still relies on.

## Build And Toolchain Assumptions

- The generated BootAsm evidence artifact correctly corresponds to the reviewed Phosphoric demo source for the current narrow boot profile.
- The narrow direct PE/COFF writer and parser correctly encode and validate the current `BOOTX64.EFI` demo image.
- The verification scripts run on a Linux host with the documented tools installed.

## Firmware And Emulator Assumptions

- `OVMF` calls the generated `efi_main` entrypoint as expected in the current QEMU flow.
- `QEMU` correctly emulates the debug console and debug-exit device used by the demo.
- CPU behavior, port-I/O behavior, and UEFI boot flow are trusted underneath the current repo.

## Compiler Assumptions

- The archived compiler frontend rejected the documented invalid cases before archival.
- The project does not yet provide a self-hosted verified backend or verified semantics-preserving lowering.
- Conservative ownership checking is treated as safer than under-enforcement, but it is not equivalent to full path-sensitive ownership proof.

## Runtime Assumptions

- The current executable artifact is a demo path, not a real isolated kernel/runtime.
- The current prototype remains single-address-space and does not claim hostile-task isolation.
- The running demo executes generated BootAsm semantics only and no non-Phosphoric runtime object or external linker output; `tools/verify/check_all.sh` enforces that boundary.

## Documentation And Review Assumptions

- `STATUS.md`, `CLAIMS.md`, and `docs/invariant_trace.md` are kept current when guarantees change.
- Reviewers treat `specified`, `demo only`, and `not implemented` as real limits, not as soft wording.
- No stronger security or isolation claim is made than the threat model and current proof artifacts support.
