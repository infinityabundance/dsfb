#!/usr/bin/env bash
# audit/scripts/run_all.sh — run dsfb-gray, Miri, Kani, cargo-fuzz against dsfb-atlas.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

echo "=== [1/4] dsfb-gray threat-surface scan ==="
./dsfb_gray.sh

echo "=== [2/4] Miri undefined-behaviour checker ==="
./miri.sh

echo "=== [3/4] Kani bounded model checker ==="
./kani.sh

echo "=== [4/4] cargo-fuzz YAML parser smoke run ==="
./fuzz.sh

echo
echo "All four audits completed. Reports under audit/reports/."
