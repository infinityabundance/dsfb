# QEMU vvfat Temporary File Failure

This note documents a host-environment failure seen while moving the
`dsfb-phosphoric` folder into a standalone subtree.

It is a QEMU drive-mode issue, not evidence that the DSFB runtime or PFI
records drifted.

## Symptom

The DSFB QEMU runner failed before the EFI application could boot:

```text
qemu-system-x86_64: -drive format=raw,file=fat:rw:/path/to/esp: Could not open temporary file '/var/tmp/vl.XXXXXX': Read-only file system
```

The debug log then had none of the expected DSFB markers because the guest did
not start.

## Cause

QEMU's vvfat folder drive mode:

```bash
-drive format=raw,file=fat:rw:"$esp_dir"
```

uses host-side temporary files. In this environment QEMU tried to create those
files under `/var/tmp`, which was read-only. Setting `TMPDIR=/tmp` was not
sufficient for this QEMU/vvfat path.

## Fixed Pattern

Use a build-local FAT disk image instead of `fat:rw:`:

```bash
esp_image="build/uefi-demo/dsfb/esp.img"
esp_dir="build/uefi-demo/dsfb/esp"

rm -f "$esp_image"
truncate -s 64M "$esp_image"
mkfs.vfat "$esp_image"
mmd -i "$esp_image" ::/EFI ::/EFI/BOOT
mcopy -i "$esp_image" "$esp_dir/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI

TMPDIR=/tmp timeout 30s qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
  -drive if=pflash,format=raw,file="$ovmf_vars" \
  -drive format=raw,file="$esp_image" \
  -nographic \
  -no-reboot \
  -chardev "file,id=cdtext,path=build/uefi-demo/dsfb/qemu-debug.log" \
  -device isa-debugcon,chardev=cdtext,iobase=0x402 \
  -chardev "file,id=cddata,path=build/uefi-demo/dsfb/dsfb_demo_records_runtime.bin" \
  -device isa-debugcon,chardev=cddata,iobase=0x500 \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04
```

The project runner does this directly:

```bash
bash tools/qemu-run/run_dsfb_demo.sh
```

## Expected DSFB Markers

A successful DSFB run writes these marker strings into
`build/uefi-demo/dsfb/qemu-debug.log`:

```text
phosphoric: dsfb demo entry
DSFB (Drift-Slew Fusion Bootstrap)
Endoduction is the 4th mode of Inference.
byte-deterministic certificate.
phosphoric: task accepted
phosphoric: residuals emitted
phosphoric: dsfb demo halt
```

The runner also checks that exactly 96 residual bytes were captured on the
debug data port, encodes them into a PFI0 case file, validates the PFI layout,
and compares the runtime PFI to `tests/golden/dsfb_demo.pfi`.

## Reproducibility Claim

This fix removes a host filesystem dependency from the QEMU launch path. It
does not prove bit-identical behavior across all QEMU, OVMF, filesystem-tool,
or host-kernel versions. The supported claim is narrower: on a host with the
listed tools installed, the checked runner boots the DSFB EFI artifact, captures
the expected debug markers and residual records, and verifies the generated PFI
against the committed golden.
