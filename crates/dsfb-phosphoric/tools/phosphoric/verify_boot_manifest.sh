#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="${1:?usage: verify_boot_manifest.sh build/uefi-demo/linked-artifact.txt}"

if [ ! -f "$manifest" ]; then
  printf 'missing boot manifest: %s\n' "$manifest" >&2
  exit 1
fi

field() {
  sed -n "s/^$1=//p" "$manifest"
}

require_equal() {
  local name="$1"
  local expected="$2"
  local actual
  actual="$(field "$name")"
  if [ "$actual" != "$expected" ]; then
    printf 'boot manifest field %s expected %s, got %s\n' "$name" "$expected" "${actual:-<missing>}" >&2
    exit 1
  fi
}

require_hash_field() {
  local name="$1"
  local actual
  actual="$(field "$name")"
  if ! printf '%s\n' "$actual" | grep -Eq '^[0-9a-f]{64}$'; then
    printf 'boot manifest field %s is not a sha256 hash: %s\n' "$name" "${actual:-<missing>}" >&2
    exit 1
  fi
}

check_file_hash() {
  local path="$1"
  local expected="$2"
  local label="$3"
  local actual
  if [ ! -f "$path" ]; then
    printf 'boot manifest %s path does not exist: %s\n' "$label" "$path" >&2
    exit 1
  fi
  actual="$(sha256sum "$path" | awk '{print $1}')"
  if [ "$actual" != "$expected" ]; then
    printf 'boot manifest %s hash mismatch: expected %s, got %s\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
}

require_equal boot_runtime boot-asm-v1-demo-runtime
require_equal c_objects none
require_equal non_phosphoric_runtime_objects none
require_equal clang_used false
require_equal lld_used false
require_equal external_linker_used false
require_equal machine_image_writer phosphoric-direct-pe32plus-v1
require_equal pe_structure verified
require_equal generated_symbols efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render

require_hash_field source_bundle_hash
require_hash_field generated_ir_hash
require_hash_field generated_asm_hash
require_hash_field generated_machine_image_hash
require_hash_field boot_artifact_sha256

boot_artifact="$(field boot_artifact)"
generated_ir="$(field generated_ir)"
generated_asm="$(field generated_asm)"
generated_machine_image="$(field generated_machine_image)"
active_phosphoric_sources="$(field active_phosphoric_sources)"

if [ -z "$active_phosphoric_sources" ]; then
  printf '%s\n' "boot manifest is missing active_phosphoric_sources" >&2
  exit 1
fi

# Current source paths are repository-local and contain no spaces. If that changes,
# the manifest format must change before this verifier is widened.
read -r -a source_paths <<< "$active_phosphoric_sources"
for source in "${source_paths[@]}"; do
  if [ ! -f "$source" ]; then
    printf 'boot manifest source path does not exist: %s\n' "$source" >&2
    exit 1
  fi
done

actual_source_hash="$(sha256sum "${source_paths[@]}" | sha256sum | awk '{print $1}')"
if [ "$actual_source_hash" != "$(field source_bundle_hash)" ]; then
  printf 'boot manifest source bundle hash mismatch: expected %s, got %s\n' "$(field source_bundle_hash)" "$actual_source_hash" >&2
  exit 1
fi

check_file_hash "$generated_ir" "$(field generated_ir_hash)" "generated_ir"
check_file_hash "$generated_asm" "$(field generated_asm_hash)" "generated_asm"
check_file_hash "$generated_machine_image" "$(field generated_machine_image_hash)" "generated_machine_image"
check_file_hash "$boot_artifact" "$(field boot_artifact_sha256)" "boot_artifact"

if [ "$generated_machine_image" != "$boot_artifact" ]; then
  printf 'boot manifest machine image and boot artifact differ: %s vs %s\n' "$generated_machine_image" "$boot_artifact" >&2
  exit 1
fi

"$repo_root/tools/phosphoric/verify_pe_efi_image.sh" "$boot_artifact" >/dev/null

printf '%s\n' "== phosphoric: boot manifest verification complete =="
