#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$repo_root"

scan_paths=(
  "Makefile"
  ".github/workflows"
  "tools/image-builder"
  "tools/phosphoric"
  "tools/qemu-run"
  "tools/release"
)

# Active verification, build, and release paths must not invoke any
# external assembler, linker, or foreign-language toolchain. The list
# below is the closed set of forbidden invocation patterns.
for forbidden in \
  '(^|[^[:alnum:]_./-])clang([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])lld([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])lld-link([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])llvm-nm([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])objcopy([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])ld([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])gcc([[:space:]]|$)' \
  '(^|[^[:alnum:]_./-])cc([[:space:]]|$)'
do
  if rg -n "$forbidden" "${scan_paths[@]}" 2>/dev/null; then
    printf 'active verification or release path invokes a forbidden external toolchain pattern: %s\n' "$forbidden" >&2
    exit 1
  fi
done

printf '%s\n' "== phosphoric: archive inertness verification complete =="
