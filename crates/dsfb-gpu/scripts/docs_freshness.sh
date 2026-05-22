#!/usr/bin/env bash
# Documentation freshness gate.
#
# This script enforces the "full code commentary" rule: every public Rust
# item must have a doc comment, and stale TODO/FIXME/XXX markers must be
# resolved (or annotated with an explicit context) before they accumulate.
#
# Two checks:
#
#   1. `cargo doc --no-deps --document-private-items` is run with
#      `RUSTDOCFLAGS=-D missing_docs` so any undocumented `pub` item fails
#      the build. This is the load-bearing check.
#
#   2. A grep for TODO/FIXME/XXX/HACK markers in source files. These are
#      tolerated only when annotated with a follow-up note in the form
#      `TODO(reason): ...` so future readers see the rationale. Bare markers
#      fail the gate.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "docs_freshness: running cargo doc with missing_docs as deny"
RUSTDOCFLAGS="-D missing_docs -D rustdoc::broken_intra_doc_links" \
  cargo doc --workspace --no-deps --document-private-items >/dev/null

echo "docs_freshness: scanning for stale TODO/FIXME/XXX markers"
unannotated_pattern='\b(TODO|FIXME|XXX|HACK)\b([^(]|$)'
excluded_dirs=(
  --exclude-dir=target
  --exclude-dir=.git
  --exclude-dir=research
  --exclude-dir=node_modules
)

if grep -RIniE "${excluded_dirs[@]}" "$unannotated_pattern" \
      --include='*.rs' --include='*.cu' --include='*.cuh' . ; then
  echo
  echo "docs_freshness: bare TODO/FIXME/XXX/HACK found — annotate with TODO(context): ..." >&2
  exit 1
fi

echo "docs_freshness: clean"
