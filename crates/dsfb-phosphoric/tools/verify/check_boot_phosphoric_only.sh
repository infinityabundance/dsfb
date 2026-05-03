#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_script="$repo_root/tools/image-builder/build_uefi_demo.sh"
build_dir="$repo_root/build/uefi-demo"
manifest="$build_dir/linked-artifact.txt"

cd "$repo_root"

"$build_script" >/dev/null

grep -q '^boot_runtime=boot-asm-v1-demo-runtime$' "$manifest"
grep -q '^c_objects=none$' "$manifest"
grep -q '^non_phosphoric_runtime_objects=none$' "$manifest"
grep -q '^source_bundle_hash=[0-9a-f]\{64\}$' "$manifest"
grep -q '^generated_ir_hash=[0-9a-f]\{64\}$' "$manifest"
grep -q '^generated_asm_hash=[0-9a-f]\{64\}$' "$manifest"
grep -q '^generated_machine_image_hash=[0-9a-f]\{64\}$' "$manifest"
grep -q '^boot_artifact_sha256=[0-9a-f]\{64\}$' "$manifest"
grep -q '^clang_used=false$' "$manifest"
grep -q '^lld_used=false$' "$manifest"
grep -q '^external_linker_used=false$' "$manifest"
grep -q '^machine_image_writer=phosphoric-direct-pe32plus-v1$' "$manifest"
grep -q '^pe_structure=verified$' "$manifest"
grep -q '^reviewed_ir_fixture=.*/tests/golden/boot_ir_v1_button_policy_golden.json$' "$manifest"
grep -q '^reviewed_asm_fixture=.*/tests/golden/boot_asm_v1_button_policy_golden.s$' "$manifest"
grep -q '^generated_symbols=efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render$' "$manifest"

generated_ir="$(sed -n 's/^generated_ir=//p' "$manifest")"
generated_asm="$(sed -n 's/^generated_asm=//p' "$manifest")"
reviewed_ir_fixture="$(sed -n 's/^reviewed_ir_fixture=//p' "$manifest")"
reviewed_asm_fixture="$(sed -n 's/^reviewed_asm_fixture=//p' "$manifest")"

"$repo_root/tools/phosphoric/verify_boot_manifest.sh" "$manifest" >/dev/null

if ! cmp -s "$generated_ir" "$reviewed_ir_fixture"; then
  printf 'generated boot IR is not diff-clean against reviewed fixture\n' >&2
  exit 1
fi

if ! cmp -s "$generated_asm" "$reviewed_asm_fixture"; then
  printf 'generated boot assembly is not diff-clean against reviewed fixture\n' >&2
  exit 1
fi

# Active source must be Phosphoric only.
foreign_in_active="$(
  find . \
    -path './.git' -prune -o \
    -path './target' -prune -o \
    -path './build' -prune -o \
    -path './untracked' -prune -o \
    -type f \( -name '*.rs' -o -name '*.c' -o -name '*.cc' -o -name '*.cpp' -o -name '*.h' -o -name '*.hpp' -o -name '*.go' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print
)"

if [ -n "$foreign_in_active" ]; then
  printf '%s\n%s\n' "non-Phosphoric source files are not permitted in the active tree:" "$foreign_in_active" >&2
  exit 1
fi

active_build_paths=(
  "Makefile"
  ".github/workflows"
  "tools/image-builder"
  "tools/phosphoric"
  "tools/qemu-run"
  "tools/release"
)

for forbidden_tool in \
  '(^|[^[:alnum:]_./-])clang([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])lld([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])lld-link([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])llvm-nm([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])objcopy([[:space:]]|$)'
do
  if rg -n "$forbidden_tool" "${active_build_paths[@]}" 2>/dev/null; then
    printf 'active golden boot path invokes forbidden external binary tool pattern: %s\n' "$forbidden_tool" >&2
    exit 1
  fi
done

printf '%s\n' "== phosphoric: golden boot path has no linked non-Phosphoric runtime objects or external linker =="
