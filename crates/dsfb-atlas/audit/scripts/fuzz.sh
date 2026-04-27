#!/usr/bin/env bash
# audit/scripts/fuzz.sh — cargo-fuzz YAML parser smoke run (30 minutes).
# For release-gating use, raise -max_total_time to 86400 (24 h).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
REPORT="$HERE/../reports/fuzz.txt"
mkdir -p "$HERE/../reports"

cd "$ROOT/crates/dsfb-atlas/audit/fuzz"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "cargo-fuzz not present; install with \`cargo install --locked cargo-fuzz\`." | tee "$REPORT"
    echo "SKIP: see audit/AUDIT.md for installation instructions." | tee -a "$REPORT"
    exit 0
fi

# Smoke time bound (30 min). Override DSFB_FUZZ_SECS for longer runs.
SECS="${DSFB_FUZZ_SECS:-1800}"

cargo +nightly fuzz run yaml_part -- \
    -max_total_time="$SECS" \
    -max_len=65536 \
    2>&1 | tee "$REPORT"

CRASHES=$(ls artifacts/yaml_part 2>/dev/null | grep -c '^crash-' || true)
if [ "$CRASHES" -ne 0 ]; then
    echo "FAIL: $CRASHES crash artifact(s) in audit/fuzz/artifacts/yaml_part/." | tee -a "$REPORT"
    exit 2
fi
echo "PASS: 0 crashes after ${SECS}s." | tee -a "$REPORT"
