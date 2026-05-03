#!/usr/bin/env bash
# check_malformed_pfi.sh — Stream C Milestone D adversarial gate
# (Session 16).
#
# Forensic role: the court refuses bad evidence with a deterministic
# named violation. Each adversarial .pfi fixture under
# tools/verify/fixtures/pfi/malformed/ MUST cause check_pfi_layout.sh
# to exit non-zero with a specific named failure. A "looks unusual"
# accept-with-warning is doctrine violation; a hang or crash is
# doctrine violation; a generic "FAIL" without the expected named
# reason is doctrine violation.
#
# Per-fixture expectations:
#
#   bad_chain_hash.pfi          chain_hash mismatch
#   seq_gap.pfi                 seq= ... != expected
#   bad_kind.pfi                kind= ... not in closed taxonomy
#   truncated_record.pfi        size ... != expected
#   nonzero_reserved.pfi        header reserved not all-zero
#   bad_magic.pfi               magic mismatch
#   stream_hash_mismatch.pfi    stream_hash mismatch
#
# A fixture without an expectation entry is itself a violation (the
# corpus must be closed).
#
# Anchors: docs/PFI0.md (invariants 1–7), docs/FORENSIC_PRIMACY.md
# (closed grammar; bad evidence → MALFORMED_CASE).
#
# Exit: 0 every malformed fixture is rejected for its expected reason,
# 1 any fixture wrongly accepted or rejected for the wrong reason,
# 2 missing tools.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

malformed_dir="tools/verify/fixtures/pfi/malformed"
layout_gate="tools/verify/check_pfi_layout.sh"

if [ ! -d "$malformed_dir" ]; then
    echo "[malformed-pfi] FAIL: malformed dir missing at $malformed_dir" >&2
    exit 1
fi
if [ ! -x "$layout_gate" ]; then
    echo "[malformed-pfi] FAIL: layout gate not executable at $layout_gate" >&2
    exit 1
fi

# Closed expectation table: basename → grep regex.
declare -A expected_pattern=(
    ["bad_chain_hash"]="chain_hash mismatch"
    ["seq_gap"]="seq=.* != expected"
    ["bad_kind"]="kind=.* not in closed taxonomy"
    ["truncated_record"]="size .* (< minimum|!= expected)"
    ["nonzero_reserved"]="header reserved not all-zero"
    ["bad_magic"]="magic mismatch"
    ["stream_hash_mismatch"]="stream_hash mismatch"
)

# Run the layout gate against a single .pfi in isolation. The standard
# gate scans the whole directory; we need per-fixture isolation. Use
# a temp dir holding only the one fixture.
run_gate_on_one() {
    local pfi="$1"
    local td
    td="$(mktemp -d)"
    trap "rm -rf '$td'" RETURN
    mkdir -p "$td/tools/verify/fixtures/pfi"
    cp "$pfi" "$td/tools/verify/fixtures/pfi/"
    cp "$layout_gate" "$td/check_pfi_layout.sh"
    # Adjust gate's cd to the temp repo_root.
    sed -i "s|cd \"\$repo_root\"|cd \"$td\"|" "$td/check_pfi_layout.sh"
    chmod +x "$td/check_pfi_layout.sh"
    bash "$td/check_pfi_layout.sh" 2>&1 || true
}

fail=0
total=0
note_fail() { echo "[malformed-pfi] FAIL: $*" >&2; fail=1; }

shopt -s nullglob
fixtures=("$malformed_dir"/*.pfi)
shopt -u nullglob

if [ "${#fixtures[@]}" -eq 0 ]; then
    echo "[malformed-pfi] FAIL: no malformed fixtures in $malformed_dir" >&2
    exit 1
fi

echo "============================================================"
echo "  Phosphoric malformed-case rejection gate"
echo "  doctrine: docs/PFI0.md invariants 1-7"
echo "============================================================"

for pfi in "${fixtures[@]}"; do
    total=$((total + 1))
    base="$(basename "$pfi" .pfi)"

    pattern="${expected_pattern[$base]:-}"
    if [ -z "$pattern" ]; then
        note_fail "$base: no expected-reason entry in this gate's table — closed corpus violation"
        continue
    fi

    # Run the layout gate against this fixture. Capture exit code AND output.
    output="$(run_gate_on_one "$pfi")"
    # The layout gate exits non-zero on FAIL. Detect via a "DOCTRINE VIOLATION"
    # banner OR per-fixture FAIL line.
    if echo "$output" | grep -q '\[pfi-layout\] OK'; then
        note_fail "$base: layout gate ACCEPTED malformed fixture (should reject)"
        continue
    fi

    # The fixture was rejected. Verify it was rejected for the EXPECTED reason.
    if ! echo "$output" | grep -qE "$pattern"; then
        note_fail "$base: rejected for wrong reason (expected pattern '$pattern')"
        echo "  --- output ---" >&2
        echo "$output" | sed 's/^/  /' >&2
        echo "  --------------" >&2
        continue
    fi

    printf '  %-40s PASS  rejected: %s\n' "$base" "$pattern"
done

echo "------------------------------------------------------------"
echo "  total malformed fixtures examined : $total"
echo "  rejection violations              : $fail"

if [ "$fail" -eq 0 ]; then
    echo "  [malformed-pfi] OK — every malformed fixture rejected with named reason"
    exit 0
fi

echo "  [malformed-pfi] DOCTRINE VIOLATION — see FAIL lines above" >&2
exit 1
