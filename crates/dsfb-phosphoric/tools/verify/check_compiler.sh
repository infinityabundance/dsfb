#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$repo_root"

# The active compiler workspace lives under compiler/ and is authored
# entirely in Phosphoric. This gate confirms the directory is present
# and that no foreign-language source files leaked into it.
if [ ! -d compiler ]; then
  printf '%s\n' "compiler/ directory is missing" >&2
  exit 1
fi

foreign_in_compiler="$(
  find compiler \
    \( -name '*.rs' -o -name '*.c' -o -name '*.cpp' -o -name '*.h' -o -name '*.go' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) \
    -print 2>/dev/null
)"

if [ -n "$foreign_in_compiler" ]; then
  printf 'foreign-language sources found in compiler/:\n%s\n' "$foreign_in_compiler" >&2
  exit 1
fi

printf '%s\n' "== phosphoric: compiler workspace contains only Phosphoric source =="
