#!/usr/bin/env bash
# check_producer_gaps_doctrine.sh — producer soundness gap registry gate.
#
# The producer at untracked/internaldocs/phase0_producer/phase0_stub.S
# has a small set of known soundness gaps: source patterns it accepts
# without diagnostic but lowers to bytes that do not match the source's
# semantic meaning. Each gap is enumerated in
# tools/verify/producer_soundness_gaps.toml with:
#   - a minimal repro source
#   - the rc the producer emits today (the frozen wrong-bytes outcome)
#   - the rc the source semantically means
#   - the producer label / pass marker that owns the mis-lowering
#   - the planned producer extension that would close the gap
#
# This gate is the symmetric counterpart of fixture_corpus.sh:
#   - fixture_corpus.sh locks CORRECT producer paths byte-equally
#   - this gate locks INCORRECT producer paths semantically (rc)
#
# Fail-closed semantics. For each registered gap, compile the source
# through phase0_stub and inspect the runtime exit code:
#
#   rc == current_rc_buggy   → PASS  (gap still reproduces; visible + gated)
#   rc == correct_rc_expected → FAIL (gap closed → registry stale; remove
#                                       this entry and add a fixture)
#   rc == anything else      → FAIL  (gap shifted to a new wrong outcome;
#                                       investigate before re-pinning)
#
# Doctrine: the producer's known wrong-bytes outcomes must remain
# enumerated. A silently-changed gap is a doctrine violation regardless
# of whether the change improves or worsens the outcome.
#
# Apex framing this gate protects: producer soundness gaps are tracked,
# not silent. Registry entries are evidence; the gate is adjudication.
#
# Exit 0: all registered gaps reproduce their recorded rc.
# Exit 1: any gap closed, shifted, or registry malformed.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

producer="untracked/internaldocs/phase0_producer/phase0_stub"
registry="tools/verify/producer_soundness_gaps.toml"

if [ ! -r "$registry" ]; then
    echo "[producer-gaps] FAIL: registry not found at $registry" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Parse: per [[gap]] block, collect the four required fields.
# Output one record per gap, '|' separated:
#   name | source | current_rc_buggy | correct_rc_expected | producer_label
# -------------------------------------------------------------------
parsed="$(awk '
    function emit() {
        if (!in_block || name == "") return
        print name "|" source "|" current_rc "|" correct_rc "|" producer_label
    }
    BEGIN { in_block = 0 }
    /^\[\[gap\]\]/ {
        emit()
        in_block = 1
        name = ""; source = ""; current_rc = ""; correct_rc = ""; producer_label = ""
        next
    }
    /^\[\[/ {
        emit()
        in_block = 0
        next
    }
    in_block {
        if ($0 ~ /^[ \t]*[a-z_][a-z_0-9]*[ \t]*=/) {
            key = $0
            sub(/^[ \t]*/, "", key)
            sub(/[ \t]*=.*/, "", key)
            val = $0
            sub(/^[ \t]*[a-z_][a-z_0-9]*[ \t]*=[ \t]*/, "", val)
            sub(/[ \t]*#.*$/, "", val)
            sub(/[ \t]+$/, "", val)
            if (substr(val, 1, 1) == "\"") {
                val = substr(val, 2)
                idx = index(val, "\"")
                if (idx > 0) val = substr(val, 1, idx - 1)
            }
            if (key == "name")                  name = val
            else if (key == "source")           source = val
            else if (key == "current_rc_buggy") current_rc = val
            else if (key == "correct_rc_expected") correct_rc = val
            else if (key == "producer_label")   producer_label = val
        }
    }
    END { emit() }
' "$registry")"

# Distinguish empty-registry (a valid celebration: producer has no
# tracked gaps) from schema-drift (parser couldn't extract a gap block
# despite the file containing [[gap]] markers).
gap_marker_count="$(grep -c '^\[\[gap\]\]' "$registry" || true)"
if [ -z "$parsed" ] && [ "${gap_marker_count:-0}" -gt 0 ]; then
    echo "[producer-gaps] FAIL: registry has $gap_marker_count [[gap]] markers but parsed zero entries — schema drift" >&2
    exit 1
fi

if [ -n "$parsed" ] && [ ! -x "$producer" ]; then
    echo "[producer-gaps] producer not built; building..."
    bash untracked/internaldocs/phase0_producer/build.sh >/dev/null
fi

echo "============================================================"
echo "  Phosphoric producer soundness gap doctrine gate"
echo "  registry: $registry"
echo "============================================================"

fails=0
total=0
declare -A registered_sources

while IFS='|' read -r name source current_rc correct_rc producer_label; do
    [ -z "$name" ] && continue
    total=$((total + 1))
    registered_sources["$source"]=1

    if [ -z "$source" ] || [ -z "$current_rc" ] || [ -z "$correct_rc" ] || [ -z "$producer_label" ]; then
        printf '  %-22s SCHEMA  missing required field(s)\n' "$name"
        fails=$((fails + 1))
        continue
    fi

    if [ ! -r "$source" ]; then
        printf '  %-22s FAIL    source missing: %s\n' "$name" "$source"
        fails=$((fails + 1))
        continue
    fi

    if [ "$current_rc" = "$correct_rc" ]; then
        printf '  %-22s SCHEMA  current_rc_buggy == correct_rc_expected (degenerate gap)\n' "$name"
        fails=$((fails + 1))
        continue
    fi

    out="$(mktemp --suffix=.bin)"
    if ! "$producer" "$source" "$out" 2>/dev/null; then
        printf '  %-22s FAIL    producer rejected source (was previously accepted)\n' "$name"
        fails=$((fails + 1))
        rm -f "$out"
        continue
    fi
    chmod +x "$out"

    actual_rc=0
    "$out" </dev/null >/dev/null 2>&1 || actual_rc="$?"
    rm -f "$out"

    if [ "$actual_rc" = "$current_rc" ]; then
        printf '  %-22s GAP     rc=%s (locked; label=%s)\n' "$name" "$actual_rc" "$producer_label"
    elif [ "$actual_rc" = "$correct_rc" ]; then
        printf '  %-22s CLOSED  rc=%s (gap remediated by producer; remove registry entry, add fixture)\n' "$name" "$actual_rc"
        fails=$((fails + 1))
    else
        printf '  %-22s SHIFTED rc=%s (expected buggy=%s or correct=%s; investigate before re-pinning)\n' \
            "$name" "$actual_rc" "$current_rc" "$correct_rc"
        fails=$((fails + 1))
    fi
done <<< "$parsed"

# Orphan check: any *.phos in producer_gaps/ that isn't referenced
# by the registry is a doctrine violation (registry is authoritative).
gaps_dir="tools/verify/producer_gaps"
orphans=0
if [ -d "$gaps_dir" ]; then
    while IFS= read -r src; do
        if [ -z "${registered_sources[$src]:-}" ]; then
            printf '  %-22s ORPHAN  %s\n' "(no name)" "$src"
            orphans=$((orphans + 1))
        fi
    done < <(find "$gaps_dir" -maxdepth 1 -name '*.phos' | sort)
fi

echo "============================================================"
if [ "$fails" -eq 0 ] && [ "$orphans" -eq 0 ]; then
    echo "  total gaps examined : $total"
    echo "  doctrine violations : 0"
    if [ "$total" -eq 0 ]; then
        echo "  [producer-gaps] OK — registry is empty; producer has no tracked soundness gaps"
    else
        echo "  [producer-gaps] OK — every registered gap reproduces its recorded rc"
    fi
    exit 0
else
    echo "  total gaps examined : $total"
    echo "  doctrine violations : $fails"
    if [ "$orphans" -gt 0 ]; then
        echo "  orphan gap sources  : $orphans (in $gaps_dir but not in registry)"
    fi
    echo "  [producer-gaps] FAIL — registry drift; see lines marked CLOSED / SHIFTED / SCHEMA / FAIL / ORPHAN above" >&2
    exit 1
fi
