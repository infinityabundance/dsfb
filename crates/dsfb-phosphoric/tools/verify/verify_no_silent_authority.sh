#!/usr/bin/env bash
# verify_no_silent_authority.sh — Stream C Milestone E doctrine gate
# (Session 17).
#
# Forensic role: lock the load-bearing invariant from
# docs/NO_SILENT_AUTHORITY.md:
#
#   "No authority transition may occur without either a declared
#    manifest edge or a typed residual."
#
# This is the line that distinguishes Phosphoric from a
# tracing/observability system. The current gate enforces what's
# enforceable today (doctrine sentence presence, kernel emission entry
# point intact, closed taxonomy preserved, boundary table coherent).
# Future tightening lands when runtime emission infrastructure exists.
#
# Anchors: docs/NO_SILENT_AUTHORITY.md, docs/FORENSIC_PRIMACY.md §1,
# kernel/residual.phos.
#
# Exit: 0 invariant intact, 1 doctrine drift detected, 2 missing tools.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

doctrine_doc="docs/NO_SILENT_AUTHORITY.md"
residual_kernel="kernel/residual.phos"

fail=0
note_fail() { echo "[no-silent-authority] FAIL: $*" >&2; fail=1; }

if [ ! -r "$doctrine_doc" ]; then
    note_fail "doctrine doc missing at $doctrine_doc"
    echo "[no-silent-authority] DOCTRINE VIOLATION" >&2
    exit 1
fi
if [ ! -r "$residual_kernel" ]; then
    note_fail "kernel residual source missing at $residual_kernel"
    echo "[no-silent-authority] DOCTRINE VIOLATION" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Check 1: doctrine apex sentence verbatim.
# -------------------------------------------------------------------
apex='No authority transition may occur without either a declared'
if ! grep -F -q "$apex" "$doctrine_doc"; then
    note_fail "apex doctrine sentence drift in $doctrine_doc"
fi

# Also require the manifest-edge OR typed-residual closure phrase.
closure='manifest edge or a typed residual'
if ! grep -F -q "$closure" "$doctrine_doc"; then
    note_fail "closure phrase '$closure' missing in $doctrine_doc"
fi

# -------------------------------------------------------------------
# Check 2: kernel/residual.phos record() fn intact.
# -------------------------------------------------------------------
if ! grep -qE '^fn record\($' "$residual_kernel" && \
   ! grep -qE '^fn record\(' "$residual_kernel"; then
    note_fail "kernel residual emission entry 'fn record' missing in $residual_kernel"
fi

# Verify the chain_step prime constants are still present (residual
# emission's load-bearing math).
for prime in 31 131 524287 16777213; do
    if ! grep -qE "\* $prime\b|\* $prime;" "$residual_kernel"; then
        note_fail "chain_step prime '$prime' missing in $residual_kernel — emission math drifted"
    fi
done

# Verify the byte-composition writes are still present (kind, arch_id,
# seq_lo, seq_hi, cycle, payload).
for marker in 'event_bytes\[0\] = kind' \
              'event_bytes\[1\] = arch_id' \
              'event_bytes\[12 \+ i\] = payload\[i\]'; do
    if ! grep -qE "$marker" "$residual_kernel"; then
        note_fail "kernel residual byte-composition marker '$marker' missing in $residual_kernel"
    fi
done

# -------------------------------------------------------------------
# Check 3: closed taxonomy R1..R7 + tail_marker preserved.
# -------------------------------------------------------------------
declare -a expected_kinds=(
    "1  cap_graph_delta"
    "2  ipc_route_delta"
    "3  budget_pressure"
    "4  effect_trace"
    "5  mmio_touch"
    "6  task_transition"
    "7  boot_check"
    "0xFF tail_marker"
)
for k in "${expected_kinds[@]}"; do
    # The taxonomy is in kernel/residual.phos comments.
    pattern="${k//[\\.]/ }"   # neutralize regex meta in keys
    if ! grep -F -q "$k" "$residual_kernel"; then
        note_fail "closed taxonomy entry '$k' drifted in $residual_kernel"
    fi
done

# -------------------------------------------------------------------
# Check 4: boundary table in NO_SILENT_AUTHORITY.md lists R1..R7 each
# exactly once.
# -------------------------------------------------------------------
for r in R1 R2 R3 R4 R5 R6 R7; do
    cnt="$(grep -c "| $r " "$doctrine_doc" || true)"
    if [ "$cnt" -ne 1 ]; then
        note_fail "$r appears $cnt times in boundary table of $doctrine_doc, expected 1"
    fi
done

# -------------------------------------------------------------------
# Check 5: forbidden vocabulary in doctrine doc (must not drift toward
# log-analyzer framing).
# -------------------------------------------------------------------
forbidden='\b(log[ -]analy[sz]er|dashboard|telemetry|metric|score|severity|debug-level|info-level|warn-level)\b'
# Allow these terms only in negative phrasings ("NOT a logger", "no
# severity levels", etc.). Reject standalone uses by checking that
# every match line includes "no", "not", or similar negation.
violators="$(grep -nEi "$forbidden" "$doctrine_doc" || true)"
if [ -n "$violators" ]; then
    while IFS= read -r vline; do
        if ! echo "$vline" | grep -qiE 'no |not |never |without |dissolves |toward them'; then
            note_fail "forbidden vocabulary in $doctrine_doc: $vline"
        fi
    done <<< "$violators"
fi

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric no-silent-authority invariant gate"
echo "  doctrine: docs/NO_SILENT_AUTHORITY.md"
echo "============================================================"
echo "  apex sentence : intact in $doctrine_doc"
echo "  emission path : kernel/residual.phos record() + chain_step intact"
echo "  taxonomy      : R1..R7 + tail_marker preserved"
echo "  boundary table: R1..R7 each listed exactly once"

if [ "$fail" -eq 0 ]; then
    echo "  [no-silent-authority] OK — invariant intact at all enforced surfaces"
    exit 0
fi

echo "  [no-silent-authority] DOCTRINE VIOLATION — see FAIL lines above" >&2
exit 1
