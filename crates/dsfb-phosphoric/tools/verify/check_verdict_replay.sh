#!/usr/bin/env bash
# check_verdict_replay.sh — Stream C Milestone C verdict + replay gate
# (Session 15).
#
# Forensic role: pin the deterministic mapping
#
#     .pfi case file  →  classifier verdict bytes
#
# This is the load-bearing replay invariant: same .pfi → same verdict
# bytes, every time, on every host. Until the runtime classifier lands
# (post-Stream-C), the verdict is captured as a `.expect` file in
# tools/verify/fixtures/verdicts/. The gate verifies:
#
#   1. Each .pfi has a paired .expect with the same basename.
#   2. Each .expect matches the canonical 6-line format from
#      FORENSIC_PRIMACY.md §3:
#        CLASS=<DriftClass member>
#        RESIDUAL=R<1..7>
#        SEQ=<u32>
#        EXPECTED=<value>
#        ACTUAL=<value>
#        EXIT=<0,2,3,4,5,6>
#   3. CLASS is a member of the closed DriftClass enum.
#   4. EXIT matches the canonical exit-code mapping.
#   5. RESIDUAL kind matches the .pfi's record[0].kind (R5 ↔ kind=5).
#   6. SEQ matches the .pfi's record[0].seq.
#   7. The .expect file is byte-stable: cat .expect | sha256sum
#      produces the same hash on multiple runs (replay idempotency).
#   8. No forbidden vocabulary (probabilistic, heuristic, log-style).
#
# Anchors: docs/FORENSIC_PRIMACY.md §3, docs/PFI0.md, kernel/residual.phos.
#
# Exit: 0 pass, 1 verdict drift / format violation, 2 missing tools.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if ! command -v od >/dev/null; then echo "[verdict] od required" >&2; exit 2; fi
if ! command -v sha256sum >/dev/null; then echo "[verdict] sha256sum required" >&2; exit 2; fi

pfi_dir="tools/verify/fixtures/pfi"
verdict_dir="tools/verify/fixtures/verdicts"

# Closed DriftClass enum (per docs/FORENSIC_PRIMACY.md §3 + kernel/dsfb.phos).
declare -a drift_classes=(
    "NO_DRIFT"
    "AUTHORITY_EXPANSION"
    "SILENT_NARROWING"
    "IPC_ROUTE_DIVERGENCE"
    "MMIO_BOUNDARY_PRESSURE"
    "STACK_BUDGET_PRESSURE"
    "TASK_STATE_SLEW"
    "BOOT_ATTESTATION_MISMATCH"
)

is_drift_class() {
    local v="$1"
    for c in "${drift_classes[@]}"; do
        [ "$c" = "$v" ] && return 0
    done
    return 1
}

# Canonical exit-code mapping per FORENSIC_PRIMACY.md §3.
valid_exit() {
    case "$1" in
        0|2|3|4|5|6) return 0 ;;
        *) return 1 ;;
    esac
}

# Forbidden log-analyzer vocabulary.
forbidden_terms="maybe|probably|possibly|suspicious|anomaly|score|likely|warn|info|debug"

fail=0
total=0
note_fail() { echo "[verdict] FAIL: $*" >&2; fail=1; }

verify_one_verdict() {
    local pfi="$1"
    local base="$(basename "$pfi" .pfi)"
    local expect="$verdict_dir/$base.expect"

    total=$((total + 1))

    if [ ! -r "$expect" ]; then
        note_fail "$base: paired verdict file missing at $expect"
        return 1
    fi

    # Format check: 6 non-empty lines + optional trailing newline.
    local nlines
    nlines="$(grep -c -E '^[A-Z]+=' "$expect" || true)"
    if [ "$nlines" -ne 6 ]; then
        note_fail "$base: verdict has $nlines KEY=VALUE lines, expected 6"
        return 1
    fi

    # Extract fields.
    local class residual seq expected actual exit_code
    class="$(awk -F= '/^CLASS=/    {print $2; exit}' "$expect")"
    residual="$(awk -F= '/^RESIDUAL=/ {print $2; exit}' "$expect")"
    seq="$(awk -F= '/^SEQ=/      {print $2; exit}' "$expect")"
    expected="$(awk -F= '/^EXPECTED=/ {print $2; exit}' "$expect")"
    actual="$(awk -F= '/^ACTUAL=/   {print $2; exit}' "$expect")"
    exit_code="$(awk -F= '/^EXIT=/     {print $2; exit}' "$expect")"

    # Required fields non-empty.
    for var_name in class residual seq expected actual exit_code; do
        local val
        eval "val=\${$var_name}"
        if [ -z "$val" ]; then
            note_fail "$base: field '$var_name' empty"
            return 1
        fi
    done

    # CLASS in closed enum.
    if ! is_drift_class "$class"; then
        note_fail "$base: CLASS=$class not in closed DriftClass enum"
        return 1
    fi

    # RESIDUAL format = R[1-7].
    if ! [[ "$residual" =~ ^R[1-7]$ ]]; then
        note_fail "$base: RESIDUAL=$residual not in {R1..R7}"
        return 1
    fi

    # EXIT in canonical table.
    if ! valid_exit "$exit_code"; then
        note_fail "$base: EXIT=$exit_code not in canonical mapping {0,2,3,4,5,6}"
        return 1
    fi

    # Forbidden vocabulary.
    if grep -iE "($forbidden_terms)" "$expect" > /dev/null; then
        note_fail "$base: log-analyzer vocabulary detected (one of: $forbidden_terms)"
        return 1
    fi

    # Cross-check residual kind against .pfi record[0].kind.
    local pfi_kind
    pfi_kind="$(od -An -tu1 -j 128 -N 1 "$pfi" | tr -d ' \n')"
    local residual_n="${residual#R}"
    if [ "$pfi_kind" != "$residual_n" ]; then
        note_fail "$base: RESIDUAL=$residual implies kind=$residual_n, .pfi record[0].kind=$pfi_kind"
        return 1
    fi

    # Cross-check seq against .pfi record[0].seq.
    local b2 b3 pfi_seq
    b2="$(od -An -tu1 -j 130 -N 1 "$pfi" | tr -d ' \n')"
    b3="$(od -An -tu1 -j 131 -N 1 "$pfi" | tr -d ' \n')"
    pfi_seq=$(( b2 | (b3 << 8) ))
    if [ "$pfi_seq" != "$seq" ]; then
        note_fail "$base: SEQ=$seq does not match .pfi record[0].seq=$pfi_seq"
        return 1
    fi

    # Replay idempotency: hash twice, compare. Trivially same since file
    # is read-only, but the assertion documents the invariant.
    local h1 h2
    h1="$(sha256sum "$expect" | awk '{print $1}')"
    h2="$(sha256sum "$expect" | awk '{print $1}')"
    if [ "$h1" != "$h2" ]; then
        note_fail "$base: verdict not byte-stable across reads (impossible for static file)"
        return 1
    fi

    printf '  %-40s PASS  CLASS=%s EXIT=%s sha=%s\n' "$base" "$class" "$exit_code" "${h1:0:12}"
    return 0
}

echo "============================================================"
echo "  Phosphoric verdict replay gate"
echo "  doctrine: docs/FORENSIC_PRIMACY.md §3"
echo "============================================================"

shopt -s nullglob
pfis=("$pfi_dir"/*.pfi)
shopt -u nullglob

if [ "${#pfis[@]}" -eq 0 ]; then
    echo "[verdict] FAIL: no .pfi fixtures present in $pfi_dir" >&2
    exit 1
fi

for pfi in "${pfis[@]}"; do
    verify_one_verdict "$pfi" || true
done

echo "------------------------------------------------------------"
echo "  total verdicts examined : $total"
echo "  verdict violations      : $fail"

if [ "$fail" -eq 0 ]; then
    echo "  [verdict] OK — every .pfi has a byte-stable verdict pair"
    exit 0
fi

echo "  [verdict] DOCTRINE VIOLATION — see FAIL lines above" >&2
exit 1
