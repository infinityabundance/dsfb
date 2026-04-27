#!/usr/bin/env bash
# audit/scripts/miri.sh — undefined-behaviour checker.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
REPORT="$HERE/../reports/miri.txt"
mkdir -p "$HERE/../reports"

cd "$ROOT"

if ! command -v rustup >/dev/null 2>&1; then
    echo "Miri requires rustup + nightly toolchain; not present in this environment." | tee "$REPORT"
    echo "SKIP: install rustup and run \`rustup +nightly component add miri\`." | tee -a "$REPORT"
    exit 0
fi

rustup +nightly component add miri --quiet 2>/dev/null || true

# Run only the unit tests under Miri (CI-tractable). Full-binary Miri is
# documented in AUDIT.md.
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows" \
cargo +nightly miri test -p dsfb-atlas --release 2>&1 | tee "$REPORT"

if grep -q "Undefined Behavior" "$REPORT"; then
    echo "FAIL: Miri reported UB." | tee -a "$REPORT"
    exit 2
fi

echo "PASS: no UB diagnostics." | tee -a "$REPORT"
