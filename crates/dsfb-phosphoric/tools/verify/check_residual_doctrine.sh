#!/usr/bin/env bash
# check_residual_doctrine.sh — residual primitive doctrine immutability gate.
#
# Parallel to check_classifier_doctrine.sh, but for goal #3 (residual
# truth) instead of goal #4 (forensic classification). The classifier
# gate locks the verdict-side doctrine; this gate locks the
# evidence-side doctrine.
#
# Apex framing (FORENSIC_PRIMACY.md, memory/feedback_residual_court.md):
#
#   "Phosphoric is a drift-and-slew residual court, not a log analyzer."
#
# `kernel/residual.phos` is the source-as-spec for the residual
# primitive — the typed delta between declared and observed at each
# authority boundary. It carries:
#   - the closed 8-kind taxonomy (R1..R7 + tail_marker)
#   - the numeric kind→name mapping that anchors FORENSIC_PRIMACY.md §2
#   - the 32-byte fixed-record layout
#   - the chain_hash discipline (deterministic 4-byte mixer)
#
# This gate fails closed if any of those properties drift. It does NOT
# verify residual emission is wired up at runtime (that's gated on
# verify_residual_stream.sh, currently a stub pending producer infra).
# It ONLY verifies the framing in source.
#
# Exit 0: doctrine intact.
# Exit 1: doctrine violation (named).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

src="kernel/residual.phos"
doc="docs/FORENSIC_PRIMACY.md"

if [ ! -r "$src" ]; then
    echo "[residual-doctrine] FAIL: source not found at $src" >&2
    exit 1
fi
if [ ! -r "$doc" ]; then
    echo "[residual-doctrine] FAIL: doctrine doc not found at $doc" >&2
    exit 1
fi

fails=0
note_fail() {
    fails=$((fails + 1))
    echo "[residual-doctrine] FAIL: $*" >&2
}

# -------------------------------------------------------------------
# 1. Closed kind taxonomy: every canonical name must appear in source.
#    Missing any one means the residual registry has been corrupted.
#    The 8 canonical kinds (R1..R7 plus the tail_marker sentinel) are
#    doctrine; an eighth-or-ninth kind requires a doctrine change per
#    docs/TASK_SEAL_V0.md §0.4.
# -------------------------------------------------------------------
required_kinds=(
    "cap_graph_delta"
    "ipc_route_delta"
    "budget_pressure"
    "effect_trace"
    "mmio_touch"
    "task_transition"
    "boot_check"
    "tail_marker"
)

for name in "${required_kinds[@]}"; do
    if ! grep -q -E "\\b$name\\b" "$src"; then
        note_fail "residual kind '$name' missing from $src"
    fi
done

# -------------------------------------------------------------------
# 2. Numeric kind→name pairings must appear in the source's comment
#    table at the top of the file. The expected format (from the
#    existing source) is `//   N  name   ...` for kinds 1..7 and
#    `//   0xFF name   ...` for the tail marker.
# -------------------------------------------------------------------
kind_pairs=(
    "1:cap_graph_delta"
    "2:ipc_route_delta"
    "3:budget_pressure"
    "4:effect_trace"
    "5:mmio_touch"
    "6:task_transition"
    "7:boot_check"
    "0xFF:tail_marker"
)

for pair in "${kind_pairs[@]}"; do
    code="${pair%%:*}"
    expected="${pair#*:}"
    if ! grep -q -E "^\s*//\s*$code\s+$expected\b" "$src"; then
        note_fail "kind table missing pairing: code=$code, name=$expected"
    fi
done

# -------------------------------------------------------------------
# 3. Residual struct layout discipline. The struct must declare
#    exactly the canonical fields with exactly the canonical types.
#    Any rename, retype, or extra field is a doctrine violation:
#    R1..R7 record shape is part of the public ABI.
#
#    Canonical:
#      kind: u8
#      arch_id: u8
#      seq: u16
#      cycle: u64
#      payload: [u8; 14]
#      chain_hash: [u8; 4]
# -------------------------------------------------------------------
struct_fields=(
    "kind:\\s*u8"
    "arch_id:\\s*u8"
    "seq:\\s*u16"
    "cycle:\\s*u64"
    "payload:\\s*\\[u8;\\s*14\\]"
    "chain_hash:\\s*\\[u8;\\s*4\\]"
)
struct_field_names=(
    "kind: u8"
    "arch_id: u8"
    "seq: u16"
    "cycle: u64"
    "payload: [u8; 14]"
    "chain_hash: [u8; 4]"
)

for i in "${!struct_fields[@]}"; do
    pat="${struct_fields[$i]}"
    name="${struct_field_names[$i]}"
    if ! grep -q -E "$pat" "$src"; then
        note_fail "Residual struct missing field declaration: $name"
    fi
done

# -------------------------------------------------------------------
# 4. No log-analyzer vocabulary. Each of these terms is a doctrine
#    violation if it appears in the source — parallel to the
#    classifier gate. Lines that explicitly mark the term as
#    forbidden are exempt (FORBIDDEN, MUST NOT, never use, etc.).
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
    "log_event"
    "log_record"
    "observability"
    "severity"
)

#
# Boundary discipline: \b treats `_` as a word character, so \banomaly\b
# would NOT match inside `anomaly_score`. That is exactly the
# compound-name pattern most likely to smuggle log-analyzer framing
# into a residual primitive (e.g. severity_score, drift_anomaly).
# Bound the term by non-alphanumeric chars on both sides so the
# match fires on `_term_`, `=term`, `term=`, ` term `, etc.
for term in "${forbidden_terms[@]}"; do
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        note_fail "forbidden term '$term' in $src: $line"
    done < <(grep -n -i -E "(^|[^A-Za-z0-9])$term([^A-Za-z0-9]|$)" "$src" 2>/dev/null | grep -v -E "FORBIDDEN|MUST NOT|never (use|emit)|anti-pattern" || true)
done

# -------------------------------------------------------------------
# 5. Apex framing. The source (or its module-level comment) must
#    name itself a 'residual' primitive and reference the doctrine
#    'declared' / 'observed' delta. Without this language the source
#    has slipped from court framing toward log-analyzer framing.
# -------------------------------------------------------------------
if ! grep -q -i -E "\\bresidual\\b" "$src"; then
    note_fail "source does not reference 'residual' as the primitive"
fi
if ! grep -q -i "declared" "$src" || ! grep -q -i "observed" "$src"; then
    note_fail "source does not frame the primitive as a delta between declared and observed"
fi

# -------------------------------------------------------------------
# 6. Cross-doc consistency: the R1..R7 mapping in the source's kind
#    table must match the verdict→residual table in FORENSIC_PRIMACY.md.
#    A drift in either direction unmoors the source-as-spec from the
#    doctrine document.
# -------------------------------------------------------------------
doc_pairs=(
    "cap_graph_delta:R1"
    "ipc_route_delta:R2"
    "budget_pressure:R3"
    "effect_trace:R4"
    "mmio_touch:R5"
    "task_transition:R6"
    "boot_check:R7"
)

for pair in "${doc_pairs[@]}"; do
    name="${pair%%:*}"
    rcode="${pair#*:}"
    # The doc's table format is `| ... | NAME | RN |`.
    if ! grep -q -E "\\b$name\\b.*\\b$rcode\\b" "$doc"; then
        note_fail "FORENSIC_PRIMACY.md missing canonical row: $name → $rcode"
    fi
done

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric residual doctrine gate"
echo "  source: $src"
echo "  doctrine: $doc"
echo "  framing: 'drift-and-slew residual court, not log analyzer'"
echo "============================================================"

if [ "$fails" -eq 0 ]; then
    echo "  [residual-doctrine] OK — closed taxonomy, struct layout,"
    echo "                       kind table, vocabulary, and FORENSIC_PRIMACY"
    echo "                       cross-references all match doctrine"
    exit 0
fi

echo "  [residual-doctrine] $fails violation(s) — see FAIL lines above"
exit 1
