#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$repo_root"

required_ignore_patterns=(
  "/build/"
  "/target/"
  "/untracked/"
  ".codex"
  "*.log"
  "*.efi"
  "*.img"
  "*.zip"
  "*.tar"
  "*.tgz"
  "*.tar.gz"
  "*.7z"
  "*.rar"
)

for pattern in "${required_ignore_patterns[@]}"; do
  if ! grep -Fqx "$pattern" .gitignore; then
    printf 'missing required .gitignore pattern: %s\n' "$pattern" >&2
    exit 1
  fi
done

tracked_archives="$(
  git ls-files | grep -E '\.(zip|tar|tgz|tar\.gz|7z|rar)$' || true
)"
if [ -n "$tracked_archives" ]; then
  printf '%s\n%s\n' "tracked archive artifacts are not allowed:" "$tracked_archives" >&2
  exit 1
fi

tracked_generated="$(
  git ls-files build target untracked 2>/dev/null || true
)"
if [ -n "$tracked_generated" ]; then
  printf '%s\n%s\n' "tracked generated paths are not allowed:" "$tracked_generated" >&2
  exit 1
fi

# Active source must be Phosphoric only. The verifier rejects any
# non-Phosphoric source extension found in the active tree.
foreign_sources="$(
  find . \
    -path './.git' -prune -o \
    -path './target' -prune -o \
    -path './build' -prune -o \
    -path './untracked' -prune -o \
    -type f \( -name '*.rs' -o -name '*.c' -o -name '*.cc' -o -name '*.cpp' -o -name '*.h' -o -name '*.hpp' -o -name '*.go' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print
)"

if [ -n "$foreign_sources" ]; then
  printf '%s\n%s\n' "non-Phosphoric source files are not permitted in the active tree:" "$foreign_sources" >&2
  exit 1
fi

printf '%s\n' "== phosphoric: repo hygiene verification complete =="
