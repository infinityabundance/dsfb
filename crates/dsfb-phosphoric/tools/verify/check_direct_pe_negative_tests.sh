#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp_dir="$(mktemp -d /tmp/phosphoric-direct-pe-negative.XXXXXX)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

expect_fail() {
  local name="$1"
  shift
  if "$@" >/dev/null 2>"$tmp_dir/$name.err"; then
    printf 'negative direct-PE test unexpectedly passed: %s\n' "$name" >&2
    exit 1
  fi
}

write_bytes() {
  local path="$1"
  local offset="$2"
  local bytes="$3"
  printf '%b' "$bytes" | dd of="$path" bs=1 seek="$offset" conv=notrunc status=none
}

build_dir="$repo_root/build/uefi-demo"
manifest="$build_dir/linked-artifact.txt"
valid_image="$tmp_dir/BOOTX64.EFI"
generated_dir="$tmp_dir/generated"

"$repo_root/tools/phosphoric/emit_boot_demo_from_phos.sh" "$generated_dir" >/dev/null
"$repo_root/tools/phosphoric/write_boot_efi_from_ir.sh" "$generated_dir" "$valid_image" >/dev/null
"$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$valid_image" >/dev/null

bad_pe="$tmp_dir/bad-pe-signature.efi"
cp "$valid_image" "$bad_pe"
write_bytes "$bad_pe" 128 '\\x00'
expect_fail bad_pe_signature "$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$bad_pe"

bad_entry="$tmp_dir/bad-entrypoint.efi"
cp "$valid_image" "$bad_entry"
write_bytes "$bad_entry" 168 '\\x00\\x00\\x00\\x00'
expect_fail bad_entrypoint "$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$bad_entry"

bad_section="$tmp_dir/bad-section-offset.efi"
cp "$valid_image" "$bad_section"
write_bytes "$bad_section" 452 '\\x10\\x00\\x00\\x00'
expect_fail bad_section_offset "$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$bad_section"

"$repo_root/tools/image-builder/build_uefi_demo.sh" >/dev/null
"$repo_root/tools/phosphoric/verify_boot_manifest.sh" "$manifest" >/dev/null

bad_ir_manifest="$tmp_dir/bad-ir-hash.env"
sed 's/^generated_ir_hash=.*/generated_ir_hash=0000000000000000000000000000000000000000000000000000000000000000/' "$manifest" > "$bad_ir_manifest"
expect_fail bad_ir_hash "$repo_root/tools/phosphoric/verify_boot_manifest.sh" "$bad_ir_manifest"

bad_source_manifest="$tmp_dir/bad-source-hash.env"
sed 's/^source_bundle_hash=.*/source_bundle_hash=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/' "$manifest" > "$bad_source_manifest"
expect_fail bad_source_hash "$repo_root/tools/phosphoric/verify_boot_manifest.sh" "$bad_source_manifest"

printf '%s\n' "== phosphoric: direct PE negative tests complete =="
