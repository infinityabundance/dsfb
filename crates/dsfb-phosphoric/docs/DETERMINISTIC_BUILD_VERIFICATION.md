# Deterministic Build Verification

This document defines the current reproducibility envelope for the repository.

It does not claim bit-for-bit deterministic output across arbitrary hosts. It records the exact inputs, commands, expected outputs, artifact paths, and known nondeterministic dependencies that reviewers should use today.

## Fixed Inputs

Current verification assumes these tracked source inputs:

- `compiler/`
- `apps/demo/`
- `apps/dsfb_demo/`
- `ember/`
- `kernel/`
- `tools/`
- `tests/golden/`
- `STATUS.md`
- `CLAIMS.md`
- `docs/`

Current verification assumes these host inputs:

- Linux host
- shell and standard file/hash/byte utilities used by the bootstrap scripts
- `mkfs.vfat`
- `mmd`
- `mcopy`
- `qemu-system-x86_64`
- split OVMF firmware at either common Ubuntu paths
  `/usr/share/OVMF/OVMF_CODE_4M.fd` and
  `/usr/share/OVMF/OVMF_VARS_4M.fd`, or edk2 paths
  `/usr/share/edk2/x64/OVMF_CODE.4m.fd` and
  `/usr/share/edk2/x64/OVMF_VARS.4m.fd`

## Required Commands

Root verification:

```bash
make verify
```

Active DSFB court verification:

```bash
make -k verify-court-active
```

Equivalent lower-level commands:

```bash
tools/verify/check_compiler.sh
tools/verify/check_archive_inert.sh
tools/verify/check_docs.sh
tools/verify/check_all.sh
tools/qemu-run/run_uefi_demo.sh
bash tools/qemu-run/run_dsfb_demo.sh
```

Release packaging:

```bash
tools/release/package_release.sh
```

## Expected Logs

The current demo verification expects these log lines in `build/uefi-demo/qemu-debug.log`:

- `phosphoric: entering generated boot-asm demo`
- `phosphoric: generated boot-asm demo runtime active`
- `phosphoric: event routed`
- `phosphoric: redraw complete`
- `phosphoric: demo complete`

The DSFB v0.3 runtime verification expects these strings in
`build/uefi-demo/dsfb/qemu-debug.log`:

- `phosphoric: dsfb demo entry`
- `DSFB (Drift-Slew Fusion Bootstrap)`
- `Endoduction is the 4th mode of Inference.`
- `byte-deterministic certificate.`
- `phosphoric: task accepted`
- `phosphoric: residuals emitted`
- `phosphoric: dsfb demo halt`

It also expects a 96-byte residual capture, a valid generated PFI0 case file,
and byte equality against `tests/golden/dsfb_demo.pfi`.

The current docs verification expects:

- no missing required hardening docs
- no invalid status labels in `STATUS.md`
- enforced invariant-manifest rows point at existing spec, enforcement, and evidence paths
- active verification and release paths do not execute archived non-Phosphoric bootstrap code

## Artifact Paths

Current important artifact paths are:

- demo EFI artifact: `build/uefi-demo/esp/EFI/BOOT/BOOTX64.EFI`
- generated boot IR artifact: `build/uefi-demo/generated/boot_ir_v1_button_policy.json`
- generated native policy evidence artifact: `build/uefi-demo/generated/phosphoric_boot_asm_v1.s`
- generated direct machine image: `build/uefi-demo/esp/EFI/BOOT/BOOTX64.EFI`
- boot provenance manifest: `build/uefi-demo/linked-artifact.txt`
- demo debug log: `build/uefi-demo/qemu-debug.log`
- DSFB EFI artifact: `build/uefi-demo/dsfb/esp/EFI/BOOT/BOOTX64.EFI`
- DSFB boot provenance manifest: `build/uefi-demo/dsfb/linked-artifact.txt`
- DSFB debug log: `build/uefi-demo/dsfb/qemu-debug.log`
- DSFB runtime residual capture: `build/uefi-demo/dsfb/dsfb_demo_records_runtime.bin`
- DSFB runtime PFI: `build/uefi-demo/dsfb/dsfb_demo.pfi`
- DSFB golden PFI: `tests/golden/dsfb_demo.pfi`
- verdict fixtures: `tools/verify/fixtures/verdicts/*.expect`
- release tarball: `build/release/phosphoric-demo-<version>.tar.gz`
- release checksums: `build/release/SHA256SUMS`

These artifacts are intentionally untracked and are recreated from source.

## Known Non-Bit-Reproducible Dependencies

The current workflow is not bit-for-bit deterministic because it still depends on:

- shell and standard utility behavior used by the narrow bootstrap scripts
- `QEMU` and `OVMF` versions
- filesystem and FAT-image tooling behavior
- tarball metadata and gzip output in the packaging step

The Colab notebook at `notebooks/dsfb_phosphoric_colab.ipynb` records the exact
git commit, QEMU version, OVMF candidates, package versions, key hashes,
verdict text, release contents, and release checksums for a fresh-runtime run.
That notebook is an external smoke/repro path, not a universal bit-for-bit
reproducibility proof.

## Review Rule

- Treat matching commands, artifact paths, and expected logs as the current reproducibility claim.
- Do not claim stronger reproducibility than this document states.
- If any command, artifact path, or required tool changes, update this document in the same change.
