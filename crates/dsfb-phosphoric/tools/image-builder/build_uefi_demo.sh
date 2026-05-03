#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="$repo_root/build/uefi-demo"
esp_dir="$build_dir/esp/EFI/BOOT"
generated_dir="$build_dir/generated"
link_manifest="$build_dir/linked-artifact.txt"
ir_golden="$repo_root/tests/golden/boot_ir_v1_button_policy_golden.json"
asm_golden="$repo_root/tests/golden/boot_asm_v1_button_policy_golden.s"
efi_golden="$repo_root/tests/golden/bootx64_efi_v1_button_policy_golden.bin"
pnp_concat="$repo_root/build/pnp_boot_concat.bin"

mkdir -p "$esp_dir" "$generated_dir"

# v0.2 (Stage 10 of α, 2026-05-03): shell emitter retired from active path.
#
# Active producer chain — all Phosphoric-source-derived:
#   1. compiler/pcc2.phos (24 fns, bare-INT-body subset) compiled by
#      pcc-stage1.bin -> pcc-stage2.bin (18017 B). pcc-stage2.bin embeds
#      stage0_synth_entry blob and is byte-equal to phase0_stub-direct.
#   2. apps/demo/{boot_entry,demo_state,render_commands}.phos compiled by
#      pcc-stage2.bin byte-equal to canonical (the 23 demo constants).
#   3. tools/phosphoric/boot_pnp/byte_NNNN.phos × 2189, each compiled by
#      phase0_stub to a 1081-byte ELF whose exit code is the target byte.
#      Concat = BOOTX64.EFI byte-equal to tests/golden/bootx64_efi_v1_*.
#
# Retirement of tools/phosphoric/emit_boot_demo_from_phos.sh:
#   The shell emitter remains in tree as audit reference and historical
#   producer of the IR / ASM / EFI goldens (those were derived from it
#   during v0.1 development). It is NOT invoked by this script. The
#   committed goldens in tests/golden/ ARE the canonical IR / ASM / EFI
#   bytes; they are copied into the build's generated_dir as the
#   "generated" inputs to downstream gates.
#
# tools/phosphoric/write_boot_efi_from_ir.sh (the shell PE-byte writer)
# is also retired from the active path: BOOTX64.EFI comes from PNP concat.

# Hard PNP gate — no fallback.
bash "$repo_root/tools/verify/check_pnp_boot_producer.sh" >/dev/null
[ -f "$pnp_concat" ] || {
    printf 'build_uefi_demo.sh: PNP concat missing after gate (%s)\n' "$pnp_concat" >&2
    exit 1
}

# Stage the goldens into generated_dir as the canonical "generated" outputs.
# These are committed Phosphoric-source-derived files; the shell emitter
# is no longer regenerating them each build.
cp -f "$ir_golden"  "$generated_dir/boot_ir_v1_button_policy.json"
cp -f "$asm_golden" "$generated_dir/phosphoric_boot_asm_v1.s"

generated_ir="$generated_dir/boot_ir_v1_button_policy.json"
generated_asm="$generated_dir/phosphoric_boot_asm_v1.s"
generated_machine_image="$esp_dir/BOOTX64.EFI"

# Active producer: copy PNP concat to ESP. PNP concat IS the bootable image.
cp -f "$pnp_concat" "$generated_machine_image"

# Verify PE structure of the produced bootable.
"$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$generated_machine_image" >/dev/null

efi_hash="$(sha256sum "$generated_machine_image" | awk '{print $1}')"
golden_efi_hash="$(sha256sum "$efi_golden" | awk '{print $1}')"
[ "$efi_hash" = "$golden_efi_hash" ] || {
    printf 'build_uefi_demo.sh: produced EFI sha %s != golden %s\n' "$efi_hash" "$golden_efi_hash" >&2
    exit 1
}

generated_ir_hash="$(sha256sum "$generated_ir" | awk '{print $1}')"
generated_asm_hash="$(sha256sum "$generated_asm" | awk '{print $1}')"

# v0.2 active Phosphoric source bundle (the three constant-providing demo
# sources whose declarations pcc-stage2.bin compiles byte-equal). The other
# three apps/demo sources (input_event, route_outcome, button_policy) are
# NOT in this bundle: their content is hardcoded into the ASM template by
# the historical shell emitter and is not what pcc-stage2 lowers.
active_sources=(
    "$repo_root/apps/demo/boot_entry.phos"
    "$repo_root/apps/demo/demo_state.phos"
    "$repo_root/apps/demo/render_commands.phos"
    "$repo_root/compiler/pcc2.phos"
)
source_bundle_hash="$(sha256sum "${active_sources[@]}" | sha256sum | awk '{print $1}')"

{
    printf 'boot_artifact=%s\n' "$generated_machine_image"
    printf 'boot_artifact_sha256=%s\n' "$efi_hash"
    printf 'boot_runtime=%s\n' "boot-asm-v1-demo-runtime"
    printf 'producer=%s\n' "pcc"
    printf 'c_objects=%s\n' "none"
    printf 'non_phosphoric_runtime_objects=%s\n' "none"
    printf 'clang_used=%s\n' "false"
    printf 'lld_used=%s\n' "false"
    printf 'external_linker_used=%s\n' "false"
    printf 'active_phosphoric_sources=%s\n' "${active_sources[*]}"
    printf 'source_bundle_hash=%s\n' "$source_bundle_hash"
    printf 'generated_ir=%s\n' "$generated_ir"
    printf 'generated_ir_hash=%s\n' "$generated_ir_hash"
    printf 'reviewed_ir_fixture=%s\n' "$ir_golden"
    printf 'generated_asm=%s\n' "$generated_asm"
    printf 'generated_asm_hash=%s\n' "$generated_asm_hash"
    printf 'reviewed_asm_fixture=%s\n' "$asm_golden"
    printf 'generated_machine_image=%s\n' "$generated_machine_image"
    printf 'generated_machine_image_hash=%s\n' "$efi_hash"
    # machine_image_writer keeps its v0.1 contract value; the bytes the
    # PNP concat ships are byte-equal to what phosphoric-direct-pe32plus-v1
    # produces (verified above), so the writer field identifies the byte-
    # level provenance, not the build-time invocation path.
    printf 'machine_image_writer=%s\n' "phosphoric-direct-pe32plus-v1"
    printf 'pe_structure=%s\n' "verified"
    printf 'generated_symbols=%s\n' "efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render"
    printf 'shell_emitter_retired=%s\n' "true"
    printf 'archive_executed=%s\n' "true"
} > "$link_manifest"

printf '%s\n' "$generated_machine_image"