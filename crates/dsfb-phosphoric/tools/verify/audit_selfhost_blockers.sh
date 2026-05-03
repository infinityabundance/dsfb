#!/usr/bin/env bash
# audit_selfhost_blockers.sh — selfhost backlog data extractor.
#
# Scans phase0/phase0_compiler.phos and tallies, per producer
# extension, how many fn bodies contain a blocker construct that
# would prevent the producer from lowering the body. Re-derives the
# numbers in docs/SELFHOST_BACKLOG.md.
#
# Output is informational; this is NOT a fail-closed gate. The
# selfhost backlog is a priority record, not a doctrine record.
#
# Usage:
#   tools/verify/audit_selfhost_blockers.sh
#   tools/verify/audit_selfhost_blockers.sh <input.phos>
#
# Exit 0 always (informational). Use the doctrine gates for
# fail-closed properties.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

input="${1:-phase0/phase0_compiler.phos}"

if [ ! -r "$input" ]; then
    echo "[selfhost-audit] FAIL: input not readable: $input" >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# Extract each fn body into its own file. Brace counter advances on
# `{` and retreats on `}`; body starts at the opening brace of the
# fn signature, ends when the counter returns to 0.
mapfile -t fn_names < <(grep -oE '^fn [a-zA-Z_][a-zA-Z_0-9]*' "$input" | awk '{print $2}')

for fn in "${fn_names[@]}"; do
    awk -v target="$fn" '
        $0 ~ "^fn "target"\\(" { in_fn=1; depth=0; next }
        in_fn && /\{/ { depth++ }
        in_fn && /\}/ { depth--; if (depth<=0) { in_fn=0; exit } }
        in_fn { print }
    ' "$input" > "$work/$fn.body"
done

total_fns="${#fn_names[@]}"

echo "============================================================"
echo "  Phosphoric selfhost-blocker audit"
echo "  input: $input"
echo "  total fns: $total_fns"
echo "============================================================"
echo ""

# Each entry: extension-name '|' grep-pattern.
# Patterns are POSIX ERE.
declare -a checks=(
    'M.3.G-full|let mut'
    'M.3.H-full|\bmatch\b'
    'M.3.I-store|\.[a-zA-Z_][a-zA-Z_0-9]*\s*=[^=]'
    'M.3.I-load|[a-z][a-zA-Z_0-9]*\.[a-z]'
    'M.3.J|Slice\['
    'M.3.K|\bResult\[|\bOption\[|\bunwrap\b'
    'if-expr|\bif\b'
    'boolop|&&|\|\|'
)

printf "  %-15s %-10s %s\n" "extension" "blocked" "% of total"
echo "  --------------- ---------- ----------"

for entry in "${checks[@]}"; do
    name="${entry%%|*}"
    pattern="${entry#*|}"
    count=0
    for body in "$work"/*.body; do
        if grep -qE "$pattern" "$body" 2>/dev/null; then
            count=$((count + 1))
        fi
    done
    pct=$((count * 100 / total_fns))
    printf "  %-15s %4d / %2d  %2d%%\n" "$name" "$count" "$total_fns" "$pct"
done

echo ""

# Bodies with NO keyword-detected blocker. These are candidates for
# lowering today; manual inspection is needed because the scanner
# misses struct literals, Ok(...) / Err(...) constructors, and
# bare-expression returns (no `return` keyword).
echo "  Bodies with no keyword-detected blocker (manual-inspect):"
unblocked=0
for body in "$work"/*.body; do
    if ! grep -qE 'let mut|\bmatch\b|\.[a-zA-Z_][a-zA-Z_0-9]*\s*=[^=]|[a-z][a-zA-Z_0-9]*\.[a-z]|Slice\[|\bResult\[|\bOption\[|\bunwrap\b|\bif\b|&&|\|\|' "$body" 2>/dev/null; then
        fn=$(basename "$body" .body)
        printf "    %s\n" "$fn"
        unblocked=$((unblocked + 1))
    fi
done

if [ "$unblocked" -eq 0 ]; then
    echo "    (none)"
fi

echo ""
echo "  Note: a fn appearing in the no-blocker list does NOT mean its"
echo "  body lowers correctly — the scanner only checks keyword constructs"
echo "  (let mut, match, etc.) and misses struct literals, constructors,"
echo "  and bare-expression returns. See docs/SELFHOST_BACKLOG.md for"
echo "  the manual-inspection follow-up."

echo "============================================================"
