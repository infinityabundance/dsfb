#!/usr/bin/env bash
# check_classifier_doctrine.sh — classifier doctrine immutability gate.
#
# The apex doctrine (recorded in docs/FORENSIC_PRIMACY.md):
#
#   "Phosphoric is a drift-and-slew residual court, not a log analyzer."
#
# The classifier source at tools/phosphoric-host/phosphoric_drift.phos
# carries the load-bearing doctrine:
#   - closed DriftClass enum with exactly 8 named verdicts
#   - canonical exit-code table
#   - no probabilistic / heuristic / log-analyzer vocabulary
#
# This gate fails closed if any of those properties drift. It does NOT
# verify the classifier is runnable (it is currently source-as-spec
# pending architectural work); it ONLY verifies the framing.
#
# Exit 0: doctrine intact.
# Exit 1: doctrine violation (named).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

src="tools/phosphoric-host/phosphoric_drift.phos"

if [ ! -r "$src" ]; then
    echo "[classifier-doctrine] FAIL: source not found at $src" >&2
    exit 1
fi

fails=0
note_fail() {
    fails=$((fails + 1))
    echo "[classifier-doctrine] FAIL: $*" >&2
}

# -------------------------------------------------------------------
# 1. Closed DriftClass enum: every canonical name must appear.
#    The 8 names are doctrine; missing any one means the verdict
#    registry has been corrupted.
# -------------------------------------------------------------------
required_names=(
    "NoDrift"
    "AuthorityExpansion"
    "SilentNarrowing"
    "IpcRouteDivergence"
    "MmioBoundaryPressure"
    "StackBudgetPressure"
    "TaskStateSlew"
    "BootAttestationMismatch"
)

for name in "${required_names[@]}"; do
    if ! grep -q -E "\\b$name\\b" "$src"; then
        note_fail "DriftClass name '$name' missing from $src"
    fi
done

# -------------------------------------------------------------------
# 2. No log-analyzer vocabulary. Each of these terms is a doctrine
#    violation if it appears anywhere in the classifier source —
#    including comments, except where the comment explicitly marks
#    the term as forbidden (lines starting with `// FORBIDDEN:` or
#    similar).
# -------------------------------------------------------------------
forbidden_terms=(
    "score"
    "maybe"
    "probably"
    "suspicious"
    "unusual"
    "anomaly"
    "heuristic"
    "probabilistic"
    "histogram"
    "telemetry"
)

for term in "${forbidden_terms[@]}"; do
    # Match case-insensitively as a whole word, but skip lines that
    # explicitly mark the term as forbidden (so the doctrine block at
    # the top of phosphoric_drift.phos can name what it bans).
    # Process substitution (not pipe) to keep `fails` updates in
    # the same shell — pipes spawn a subshell.
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        note_fail "forbidden term '$term' in $src: $line"
    done < <(grep -n -i -E "\\b$term\\b" "$src" 2>/dev/null | grep -v -E "FORBIDDEN|MUST NOT|never (use|emit)|anti-pattern" || true)
done

# -------------------------------------------------------------------
# 3. Exit-code table must match docs/FORENSIC_PRIMACY.md §3.
#    The classifier source's comment block lists numeric codes
#    paired with verdict-class names; any drift here unmoors the
#    classifier from the doctrine table.
#
#    Table (canonical):
#      0 → NO_DRIFT
#      1 → tool internal error (NOT a verdict)
#      2 → AUTHORITY_EXPANSION | IPC_ROUTE_DIVERGENCE | TASK_STATE_SLEW
#      3 → BOOT_ATTESTATION_MISMATCH
#      4 → chain corruption
#      5 → malformed container
#      6 → SILENT_NARROWING | *_PRESSURE
# -------------------------------------------------------------------
exit_table_pairs=(
    "0:NO_DRIFT"
    "2:AUTHORITY_EXPANSION"
    "2:IPC_ROUTE_DIVERGENCE"
    "2:TASK_STATE_SLEW"
    "3:BOOT_ATTESTATION_MISMATCH"
    "4:chain"
    "5:malformed"
    "6:SILENT_NARROWING"
    "6:.*PRESSURE"
)

for pair in "${exit_table_pairs[@]}"; do
    code="${pair%%:*}"
    expected="${pair#*:}"
    # Look for the numeric code AND the expected token on the same
    # line (the comment table format `//   N  TOKEN`).
    if ! grep -q -E "^\s*//\s*$code\b.*$expected" "$src"; then
        note_fail "exit-code table missing pairing: code=$code, expected token=$expected"
    fi
done

# -------------------------------------------------------------------
# 4. Apex framing reminder. The source (or its module-level comment)
#    must reference its judicial role at least once. The lexicon
#    "classifier" and "drift class" / "drift classes" are required.
# -------------------------------------------------------------------
if ! grep -q -i "classifier" "$src"; then
    note_fail "source does not name itself a 'classifier' (judicial role)"
fi
if ! grep -q -i -E "drift class(es)?" "$src"; then
    note_fail "source does not reference 'drift class(es)' as the verdict registry"
fi

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric classifier doctrine gate"
echo "  source: $src"
echo "  framing: 'drift-and-slew residual court, not log analyzer'"
echo "============================================================"

if [ "$fails" -eq 0 ]; then
    echo "  [classifier-doctrine] OK — verdict registry, vocabulary, and"
    echo "                          exit-code table all match doctrine"
    exit 0
fi

echo "  [classifier-doctrine] $fails violation(s) — see FAIL lines above"
exit 1
