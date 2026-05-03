#!/usr/bin/env bash
# check_fixture_razor.sh — fixture razor gate.
#
# Enforces the doctrine in docs/FIXTURE_RAZOR.md on every entry of
# tools/verify/fixture_manifest.toml. Every fixture must have:
#   - behavior_class
#   - ambiguity_removed
#   - at least one of the four required_for_* doctrine flags = true
#   - source file present on disk
#   - either unique behavior_class OR justified_duplicate explanation
#
# Exit 0: all entries satisfy the razor.
# Exit 1: any entry violates the doctrine.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

manifest="tools/verify/fixture_manifest.toml"

if [ ! -r "$manifest" ]; then
    echo "[razor] FAIL: manifest not found at $manifest" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Parse: per [[fixture]] block, collect every razor-relevant field.
# Output one record per fixture, '|' separated:
#   name | source | behavior_class | ambiguity_removed | self | seal | residual | forensic | justified
# -------------------------------------------------------------------
parsed="$(awk '
    function emit() {
        if (!in_block || name == "") return
        print name "|" source "|" behavior_class "|" ambiguity_removed "|" \
              req_self "|" req_seal "|" req_residual "|" req_forensic "|" justified
    }
    function strip_quoted(v) {
        gsub(/^[ \t]*"|"[ \t]*$/, "", v)
        return v
    }
    function strip_unquoted(v) {
        gsub(/[ \t]/, "", v)
        return v
    }
    BEGIN {
        in_block = 0
    }
    /^\[\[fixture\]\]/ {
        emit()
        in_block = 1
        name = ""; source = ""; behavior_class = ""; ambiguity_removed = ""
        req_self = "false"; req_seal = "false"; req_residual = "false"; req_forensic = "false"
        justified = ""
        next
    }
    /^\[\[/ {
        emit()
        in_block = 0
        next
    }
    in_block {
        # Match `key = value` lines. Value may be quoted-string or bareword.
        if (match($0, /^[ \t]*([a-z_][a-z_0-9]*)[ \t]*=[ \t]*(.*)$/, m)) {
            key = m[1]
            val = m[2]
            # Strip trailing comment + whitespace
            sub(/[ \t]*#.*$/, "", val)
            sub(/[ \t]+$/, "", val)
            # Strip surrounding quotes if present
            if (substr(val, 1, 1) == "\"") {
                val = substr(val, 2)
                idx = index(val, "\"")
                if (idx > 0) val = substr(val, 1, idx - 1)
            }
            if (key == "name")                                  name = val
            else if (key == "source")                           source = val
            else if (key == "behavior_class")                   behavior_class = val
            else if (key == "ambiguity_removed")                ambiguity_removed = val
            else if (key == "required_for_self_host")           req_self = val
            else if (key == "required_for_task_seal")           req_seal = val
            else if (key == "required_for_residual_truth")      req_residual = val
            else if (key == "required_for_forensic_classification") req_forensic = val
            else if (key == "justified_duplicate")              justified = val
        }
    }
    END {
        emit()
    }
' "$manifest")"

if [ -z "$parsed" ]; then
    echo "[razor] FAIL: manifest parsed to zero entries" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Per-entry razor checks.
# -------------------------------------------------------------------
fails=0
total=0
declare -A behavior_seen   # behavior_class → first fixture name
declare -A behavior_just   # behavior_class → first fixture's justified value

note_fail() {
    fails=$((fails + 1))
    echo "[razor] FAIL: $*" >&2
}

while IFS='|' read -r name source behavior_class ambig self seal residual forensic justified; do
    [ -z "$name" ] && continue
    total=$((total + 1))

    if [ -z "$behavior_class" ]; then
        note_fail "$name: missing behavior_class"
        continue
    fi
    if [ -z "$ambig" ]; then
        note_fail "$name: missing ambiguity_removed"
        continue
    fi

    # At least one doctrine flag must be true.
    if [ "$self" != "true" ] && [ "$seal" != "true" ] && \
       [ "$residual" != "true" ] && [ "$forensic" != "true" ]; then
        note_fail "$name: maps to no doctrine goal (all required_for_* are false)"
        continue
    fi

    # Source file must exist.
    if [ -z "$source" ] || [ ! -r "$source" ]; then
        note_fail "$name: source file missing or unreadable: $source"
        continue
    fi

    # Duplicate behavior_class detection.
    if [ -n "${behavior_seen[$behavior_class]:-}" ]; then
        first="${behavior_seen[$behavior_class]}"
        if [ -z "$justified" ]; then
            note_fail "$name: duplicate behavior_class '$behavior_class' (first seen on '$first') without justified_duplicate"
            continue
        fi
    else
        behavior_seen["$behavior_class"]="$name"
        behavior_just["$behavior_class"]="$justified"
    fi
done <<< "$parsed"

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric fixture razor gate"
echo "  doctrine: docs/FIXTURE_RAZOR.md"
echo "============================================================"
echo "  total fixtures examined : $total"
echo "  razor violations        : $fails"

if [ "$fails" -eq 0 ]; then
    echo "  [razor] OK — every fixture is doctrine-justified"
    exit 0
fi

echo "  [razor] DOCTRINE VIOLATION — see FAIL lines above"
exit 1
