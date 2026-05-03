#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_script="$repo_root/tools/image-builder/build_uefi_demo.sh"
build_dir="$repo_root/build/uefi-demo"
esp_root="$build_dir/esp"
esp_image="$build_dir/esp.img"
ovmf_vars="$build_dir/OVMF_VARS.4m.fd"
debug_log="$build_dir/qemu-debug.log"
link_manifest="$build_dir/linked-artifact.txt"

find_ovmf_file() {
  local configured="$1"
  shift

  if [ -n "$configured" ] && [ -f "$configured" ]; then
    printf '%s\n' "$configured"
    return 0
  fi

  local candidate
  for candidate in "$@"; do
    if [ -f "$candidate" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  return 1
}

if ! ovmf_code="$(find_ovmf_file "${PHOSPHORIC_OVMF_CODE:-}" \
  /usr/share/edk2/x64/OVMF_CODE.4m.fd \
  /usr/share/OVMF/OVMF_CODE_4M.fd \
  /usr/share/OVMF/OVMF_CODE.fd)"; then
  echo "failed to locate OVMF code image; set PHOSPHORIC_OVMF_CODE" >&2
  exit 1
fi

if ! ovmf_vars_src="$(find_ovmf_file "${PHOSPHORIC_OVMF_VARS:-}" \
  /usr/share/edk2/x64/OVMF_VARS.4m.fd \
  /usr/share/OVMF/OVMF_VARS_4M.fd \
  /usr/share/OVMF/OVMF_VARS.fd)"; then
  echo "failed to locate OVMF vars image; set PHOSPHORIC_OVMF_VARS" >&2
  exit 1
fi

if [ -z "$ovmf_code" ] || [ -z "$ovmf_vars_src" ]; then
  echo "failed to locate OVMF firmware; set PHOSPHORIC_OVMF_CODE and PHOSPHORIC_OVMF_VARS" >&2
  exit 1
fi

"$build_script" >/dev/null

grep -q '^boot_runtime=boot-asm-v1-demo-runtime$' "$link_manifest"
grep -q '^c_objects=none$' "$link_manifest"
grep -q '^non_phosphoric_runtime_objects=none$' "$link_manifest"
grep -q '^clang_used=false$' "$link_manifest"
grep -q '^lld_used=false$' "$link_manifest"
grep -q '^external_linker_used=false$' "$link_manifest"
grep -q '^source_bundle_hash=[0-9a-f]\{64\}$' "$link_manifest"
grep -q '^generated_ir_hash=[0-9a-f]\{64\}$' "$link_manifest"
grep -q '^generated_asm_hash=[0-9a-f]\{64\}$' "$link_manifest"
grep -q '^generated_machine_image_hash=[0-9a-f]\{64\}$' "$link_manifest"
grep -q '^machine_image_writer=phosphoric-direct-pe32plus-v1$' "$link_manifest"
grep -q '^pe_structure=verified$' "$link_manifest"
grep -q '^generated_symbols=efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render$' "$link_manifest"
"$repo_root/tools/phosphoric/verify_boot_manifest.sh" "$link_manifest" >/dev/null

mkdir -p "$build_dir"
cp "$ovmf_vars_src" "$ovmf_vars"
rm -f "$debug_log" "$esp_image"
truncate -s 64M "$esp_image"
mkfs.vfat "$esp_image" >/dev/null
mmd -i "$esp_image" ::/EFI ::/EFI/BOOT
mcopy -i "$esp_image" "$esp_root/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI

qemu_status=0
if ! TMPDIR=/tmp timeout 20s qemu-system-x86_64 \
  -machine q35,accel=tcg \
  -cpu max \
  -m 256M \
  -display none \
  -monitor none \
  -serial none \
  -debugcon "file:$debug_log" \
  -global isa-debugcon.iobase=0x402 \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
  -drive if=pflash,format=raw,file="$ovmf_vars" \
  -drive format=raw,file="$esp_image" \
  -vga std \
  -boot menu=off; then
  qemu_status=$?
fi

cat "$debug_log"

grep -q "phosphoric: entering generated boot-asm demo" "$debug_log"
grep -q "phosphoric: generated boot-asm demo runtime active" "$debug_log"
grep -q "phosphoric: event routed" "$debug_log"
grep -q "phosphoric: redraw complete" "$debug_log"
grep -q "phosphoric: demo complete" "$debug_log"

if [ "$qemu_status" -ne 0 ] && [ "$qemu_status" -ne 1 ]; then
  exit "$qemu_status"
fi
