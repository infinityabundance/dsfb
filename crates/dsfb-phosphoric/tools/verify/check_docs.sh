#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$repo_root"

# Public-facing required files. Internal planning artifacts are tracked
# in a separate, untracked location and are not part of the public verify.
required_files=(
  "README.md"
  "LICENSE"
  "NOTICE"
  "Makefile"
  "config/x86_64_qemu_budget.toml"
  "docs/abi.md"
  "docs/BOOT_ABI_V1.md"
  "docs/ir.md"
  "docs/language/V0_FREEZE.md"
  "docs/language/grammar.md"
  "docs/language/effects.md"
  "docs/language/effect_lattice.toml"
  "docs/language/HOST_PROFILE.md"
  "docs/language/host_profile.toml"
  "docs/language/TRUSTED_PROFILE.md"
  "docs/language/trusted_profile.toml"
  "docs/language/RUNTIME_PROFILE.md"
  "docs/language/runtime_profile.toml"
  "docs/language/BOUNDED_LOOPS.md"
  "docs/language/memory_model.md"
  "docs/language/type_system.md"
  ".github/workflows/verify.yml"
  "tools/verify/check_archive_inert.sh"
  "tools/verify/check_boot_phosphoric_only.sh"
  "tools/verify/check_direct_pe_negative_tests.sh"
  "tools/phosphoric/emit_boot_demo_from_phos.sh"
  "tools/phosphoric/write_boot_efi_from_ir.sh"
  "tools/phosphoric/verify_pe_efi_image.sh"
  "tools/phosphoric/verify_boot_manifest.sh"
  "apps/demo/button_policy.phos"
  "apps/demo/boot_entry.phos"
  "apps/demo/demo_state.phos"
  "apps/demo/input_event.phos"
  "apps/demo/render_commands.phos"
  "apps/demo/route_outcome.phos"
  "ember/docs/EMBER_TRUST_AUDIT.md"
  "ember/docs/EMBER_MINIMALITY.md"
)

for path in "${required_files[@]}"; do
  if [ ! -f "$path" ]; then
    printf 'missing required file: %s\n' "$path" >&2
    exit 1
  fi
done

removed_bootstrap_paths=(
  "ember/boot/uefi_demo.c"
  "ember/boot/uefi_min.h"
  "kernel/include/phosphoros_kernel.h"
)

for path in "${removed_bootstrap_paths[@]}"; do
  if [ -e "$path" ]; then
    printf 'legacy bootstrap path should not exist: %s\n' "$path" >&2
    exit 1
  fi
done

# Public README must reference the demo entry banner.
grep -q 'phosphoric: entering generated boot-asm demo' README.md || {
  printf 'README.md missing demo entry banner reference\n' >&2
  exit 1
}

printf '%s\n' "== phosphoric: docs verification complete =="
