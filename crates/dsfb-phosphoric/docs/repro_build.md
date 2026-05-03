# Reproducible Build Notes

This document records the current rebuild path for the repository artifacts that exist today.

It does not claim bit-for-bit reproducibility across arbitrary host environments. It records the host tools, paths, and commands currently required to rebuild and rerun the `UEFI` vertical slice from repository sources.

## Host Assumptions

The current workflow assumes:

- Linux host environment
- shell and standard file/hash/byte utilities
- `mkfs.vfat`
- `mmd`
- `mcopy`
- `qemu-system-x86_64`
- split OVMF firmware such as `/usr/share/OVMF/OVMF_CODE_4M.fd` and
  `/usr/share/OVMF/OVMF_VARS_4M.fd`, or the edk2 paths
  `/usr/share/edk2/x64/OVMF_CODE.4m.fd` and
  `/usr/share/edk2/x64/OVMF_VARS.4m.fd`

If any of those tools or firmware paths differ, the documented commands may fail until the local environment is adjusted.

## Source Inputs

Current rebuilds are expected to start from tracked repository files only.

Generated directories are not source inputs:

- `build/`
- `target/`

The documented scripts recreate the needed build outputs from the tracked sources under:

- `apps/`
- `apps/dsfb_demo/`
- `compiler/phosphoric-compiler/tests/fixtures/`
- `ember/`
- `kernel/`
- `tools/`

## Compiler Verification Path

Archive-only compiler material lives under `archive/` and is not executed by active verification.

`tools/verify/check_compiler.sh` is a non-executing sentinel that records this boundary.

## Full Verification Path

Preferred root entrypoint:

```bash
make verify
```

The active DSFB court entrypoint is:

```bash
make -k verify-court-active
```

Equivalent script entrypoint:

```bash
tools/verify/check_all.sh
```

The script currently runs:

- `tools/verify/check_repo_hygiene.sh`
- `tools/verify/check_archive_inert.sh`
- `tools/verify/check_docs.sh`
- boot provenance, direct PE/EFI, and no non-Phosphoric runtime gates
- `tools/qemu-run/run_uefi_demo.sh`

Rules:

- the script must run from tracked source files
- the script must not require committed `build/` or `target/` artifacts
- new verification steps must be added here when they become required for local confidence

## Compiler Verification Path

Run the compiler-only regression suite with:

```bash
tools/verify/check_compiler.sh
```

The script does not run compiler tests. Those tests are archive-only material until a permitted active compiler path exists.

## Assurance Report Path

The frozen-v0 assurance driver is archive-only material. Active evidence currently consists of the reviewed JSON fixtures under `compiler/phosphoric-compiler/tests/fixtures/` and the invariant manifest gate.

## Documentation Verification Path

Run the documentation integrity gate with:

```bash
tools/verify/check_docs.sh
```

The script currently verifies:

- required governance, hardening, and trust-boundary docs exist
- `STATUS.md` uses only the allowed labels: `ENFORCED`, `SPECIFIED`, `PARTIAL`, `DEMO ONLY`, `NOT IMPLEMENTED`
- `docs/invariant_manifest.toml` exists and every enforced row points at existing spec, enforcement, and evidence paths

This is the docs assurance entrypoint used by `make verify-docs`.

## UEFI Demo Build Path

Build the vertical-slice boot artifact with:

```bash
tools/image-builder/build_uefi_demo.sh
```

The script currently:

- consumes active `.phos` sources under `apps/demo/`
- emits `boot_ir_v1_button_policy.json` and `phosphoric_boot_asm_v1.s` under `build/uefi-demo/generated/`
- compares generated IR/ASM byte-for-byte against reviewed golden fixtures
- writes a PE32+ `UEFI` application directly with `tools/phosphoric/write_boot_efi_from_ir.sh`
- validates the direct image with `tools/phosphoric/verify_pe_efi_image.sh`
- writes `build/uefi-demo/linked-artifact.txt` with source, IR, ASM evidence, machine-image, and EFI hashes plus manifest fields recording no non-Phosphoric runtime objects, no C objects, no archive execution, and no external assembler/linker use
- writes `BOOTX64.EFI` under `build/uefi-demo/esp/EFI/BOOT/`

The script prints the path to the generated `BOOTX64.EFI` artifact on success.

## UEFI Demo Run Path

Run the current vertical slice with:

```bash
tools/qemu-run/run_uefi_demo.sh
```

The runner currently:

- rebuilds the `UEFI` demo
- verifies the golden boot manifest rejects non-Phosphoric runtime objects, archived execution, and external linker participation
- creates a FAT ESP image under `build/uefi-demo/`
- copies the `BOOTX64.EFI` artifact into that image
- copies `OVMF_VARS.4m.fd` into the build directory
- launches `QEMU` against the local ESP image and `OVMF`
- prints the debug log and requires the demo to finish successfully

The runner now supports:

- `PHOSPHORIC_OVMF_CODE`
- `PHOSPHORIC_OVMF_VARS`

If those variables are unset, it auto-detects common `edk2` and Ubuntu `OVMF` install paths.

## DSFB v0.3 QEMU Run Path

Run the DSFB theorem demo and PFI capture path with:

```bash
bash tools/qemu-run/run_dsfb_demo.sh
```

The runner currently:

- rebuilds the DSFB `UEFI` demo with `tools/image-builder/build_dsfb_demo.sh`
- checks the generated PE/EFI image against committed golden bytes
- creates a build-local FAT ESP image at `build/uefi-demo/dsfb/esp.img`
- copies `BOOTX64.EFI` into that image with `mtools`
- launches `QEMU` against the FAT image and OVMF
- captures the text debug log at `build/uefi-demo/dsfb/qemu-debug.log`
- captures 96 residual bytes at `build/uefi-demo/dsfb/dsfb_demo_records_runtime.bin`
- encodes the runtime records into `build/uefi-demo/dsfb/dsfb_demo.pfi`
- compares that PFI byte-for-byte against `tests/golden/dsfb_demo.pfi`

The runner supports:

- `PHOSPHORIC_OVMF_CODE`
- `PHOSPHORIC_OVMF_VARS`
- `PHOSPHORIC_OVMF` for legacy combined firmware
- `PHOSPHORIC_QEMU_TIMEOUT`, default `30s`

It deliberately does not use QEMU `fat:rw:` vvfat folder mode. See
[QEMU_VVFAT_TMPDIR.md](QEMU_VVFAT_TMPDIR.md) for the `/var/tmp` failure mode
that motivated this.

## Expected DSFB Log

The DSFB success signal is the presence of these strings in
`build/uefi-demo/dsfb/qemu-debug.log`:

- `phosphoric: dsfb demo entry`
- `DSFB (Drift-Slew Fusion Bootstrap)`
- `Endoduction is the 4th mode of Inference.`
- `byte-deterministic certificate.`
- `phosphoric: task accepted`
- `phosphoric: residuals emitted`
- `phosphoric: dsfb demo halt`

The runner checks all of these markers, the 96-byte residual capture size, PFI
layout validity, and byte equality against the committed DSFB PFI golden.

## Colab Reproducibility Path

The fresh-runtime notebook is:

```text
notebooks/dsfb_phosphoric_colab.ipynb
```

By default it clones `https://github.com/infinityabundance/dsfb.git`, checks
out `main`, enters `crates/dsfb-phosphoric`, installs the Linux dependencies,
runs `make -k verify-court-active`, builds the release bundle, and displays:

- resolved git commit
- QEMU version and OVMF candidates
- installed package versions
- DSFB QEMU debug log
- `linked-artifact.txt`
- PFI and verdict SHA256 values
- `tools/verify/fixtures/verdicts/*.expect`
- `build/release/SHA256SUMS`
- release archive contents

This is the recommended external smoke/repro path. It is not a claim that every
host will produce bit-identical release archives.

## Expected Demo Log

The current success signal is the presence of these lines in the `QEMU` debug log:

- `phosphoric: entering generated boot-asm demo`
- `phosphoric: generated boot-asm demo runtime active`
- `phosphoric: event routed`
- `phosphoric: redraw complete`
- `phosphoric: demo complete`

The `tools/qemu-run/run_uefi_demo.sh` script already checks for the final success line.

## Release Packaging Path

Create a reviewable release bundle with:

```bash
tools/release/package_release.sh
```

The packaging script currently:

- rebuilds the UEFI demo
- stages the current release evidence set under `build/release/`
- copies the generated `BOOTX64.EFI`
- copies the key proof and review documents
- writes `SHA256SUMS`
- emits an untracked tarball under `build/release/`

## CI Verification Path

The required Linux workflow lives at:

```text
.github/workflows/verify.yml
```

The workflow currently:

- checks out the repository
- installs `dosfstools`, `mtools`, `qemu-system-x86`, and `ovmf`
- runs `make verify`

## Review Requirements For Rebuild Changes

- If a new host tool becomes required, add it here.
- If a firmware path changes, add the exact new path here.
- If the runner starts depending on checked-in generated artifacts, reject the change.
- If the vertical slice changes its expected success log, update the expected log lines here.
- If the DSFB runner changes its expected markers, residual byte count, or PFI
  artifact path, update this document and the Colab notebook in the same change.
- If compiler verification is reintroduced, update this document and the archive-inertness gate together.
- If `make verify` stops being the root entrypoint, update this document and the root `Makefile` together.

## Known Limits

- The document records command and evidence reproducibility, not a universal
  bit-for-bit reproducibility proof.
- The build still depends on shell/coreutils behavior, filesystem/FAT-image tooling, `QEMU`, and `OVMF` versions.
- The current demo path is a bootstrap artifact, not a fully packaged Phosphoric-native system image.
