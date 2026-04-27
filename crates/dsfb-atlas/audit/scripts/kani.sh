#!/usr/bin/env bash
# audit/scripts/kani.sh — Rust bounded model checker.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
REPORT="$HERE/../reports/kani.txt"
mkdir -p "$HERE/../reports"

cd "$ROOT/crates/dsfb-atlas"

if ! command -v cargo-kani >/dev/null 2>&1; then
    echo "Kani not present; install with \`cargo install --locked kani-verifier && cargo kani setup\`." | tee "$REPORT"
    echo "SKIP: see audit/README.md for installation instructions." | tee -a "$REPORT"
    exit 0
fi

cargo kani --harness dedup_collision_iff_repeated_body 2>&1 | tee "$REPORT"

if grep -q "VERIFICATION:- SUCCESSFUL" "$REPORT"; then
    echo "PASS: dedup_collision_iff_repeated_body verified." | tee -a "$REPORT"
    exit 0
fi
echo "FAIL: Kani did not report VERIFICATION:- SUCCESSFUL." | tee -a "$REPORT"
exit 2
