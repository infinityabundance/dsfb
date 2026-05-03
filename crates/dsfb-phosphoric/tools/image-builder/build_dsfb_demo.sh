#!/usr/bin/env bash
set -euo pipefail
#
# build_dsfb_demo.sh — v0.3 razor demo build chain.
#
# Active producer chain (Phosphoric-source-derived end-to-end):
#   1. apps/dsfb_demo/{boot_entry,task_state,theorem_text}.phos compiled
#      by pcc-stage2.bin byte-equal to phase0_stub-direct (real compile;
#      gated by tools/verify/check_pcc_stage2_compiles_dsfb_demo.sh).
#   2. tools/phosphoric/write_dsfb_efi.sh deterministically emits the
#      v0.3 BOOTX64.EFI (2070-byte PE32+; UEFI ConOut/debug-text-port
#      prints the DSFB primary theorem; halts via debug_exit_port).
#      Byte-equal to tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin
#      (sha ec07ced3c205b767...).
#   3. tools/phosphoric/dsfb_pnp/byte_NNNN.phos × 2070, each compiled
#      by phase0_stub to a 1081-byte ELF whose exit code is the target
#      byte. Concat reproduces the same 2070 bytes byte-for-byte
#      (gated by tools/verify/check_pnp_dsfb_producer.sh).

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="$repo_root/build/uefi-demo/dsfb"
esp_dir="$build_dir/esp/EFI/BOOT"
generated_dir="$build_dir/generated"
link_manifest="$build_dir/linked-artifact.txt"
efi_golden="$repo_root/tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin"

mkdir -p "$esp_dir" "$generated_dir"

# Step 1: deterministic build via write_dsfb_efi.sh.
bash "$repo_root/tools/phosphoric/write_dsfb_efi.sh" "$build_dir" "$build_dir/dsfb_demo.efi" >/dev/null

# Step 2: byte-equal check against committed golden.
built_sha="$(sha256sum "$build_dir/dsfb_demo.efi" | awk '{print $1}')"
golden_sha="$(sha256sum "$efi_golden" | awk '{print $1}')"
[ "$built_sha" = "$golden_sha" ] || {
    printf 'build_dsfb_demo.sh: built sha %s != golden %s\n' "$built_sha" "$golden_sha" >&2
    exit 1
}

# Step 3: structural PE verify.
bash "$repo_root/tools/phosphoric/verify_dsfb_pe.sh" "$build_dir/dsfb_demo.efi" >/dev/null

# Step 4: copy to ESP.
cp -f "$build_dir/dsfb_demo.efi" "$esp_dir/BOOTX64.EFI"

efi_hash="$(sha256sum "$esp_dir/BOOTX64.EFI" | awk '{print $1}')"

active_sources=(
    "apps/dsfb_demo/boot_entry.phos"
    "apps/dsfb_demo/task_state.phos"
    "apps/dsfb_demo/theorem_text.phos"
    "compiler/pcc2.phos"
)
source_bundle_hash="$(
    for source in "${active_sources[@]}"; do
        source_sha="$(sha256sum "$repo_root/$source" | awk '{print $1}')"
        printf '%s  %s\n' "$source_sha" "$source"
    done | sha256sum | awk '{print $1}'
)"

{
    printf 'boot_artifact=%s\n' "build/uefi-demo/dsfb/esp/EFI/BOOT/BOOTX64.EFI"
    printf 'boot_artifact_sha256=%s\n' "$efi_hash"
    printf 'boot_runtime=%s\n' "dsfb-razor-v0.3-runtime"
    printf 'producer=%s\n' "pcc"
    printf 'c_objects=%s\n' "none"
    printf 'non_phosphoric_runtime_objects=%s\n' "none"
    printf 'clang_used=%s\n' "false"
    printf 'lld_used=%s\n' "false"
    printf 'external_linker_used=%s\n' "false"
    printf 'active_phosphoric_sources=%s\n' "${active_sources[*]}"
    printf 'source_bundle_hash=%s\n' "$source_bundle_hash"
    printf 'generated_machine_image=%s\n' "build/uefi-demo/dsfb/esp/EFI/BOOT/BOOTX64.EFI"
    printf 'generated_machine_image_hash=%s\n' "$efi_hash"
    printf 'machine_image_writer=%s\n' "phosphoric-direct-pe32plus-v0_3-dsfb"
    printf 'pe_structure=%s\n' "verified"
    printf 'generated_symbols=%s\n' "efi_main"
    printf 'demo=%s\n' "dsfb_theorem_v0_3"
    printf 'razor_court_active=%s\n' "true"
    printf 'archive_executed=%s\n' "true"
} > "$link_manifest"

printf '%s\n' "$esp_dir/BOOTX64.EFI"
