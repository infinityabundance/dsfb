#!/usr/bin/env bash
# verify_source_asm_byte_equal.sh — STATE ENFORCEMENT gate.
#
# Directive 2026-05-02 (HARD ENFORCEMENT, all-caps in original):
#
#   "100% PHOSPHORIC SELFHOST COMPILER MUST WORK WITH THE FIXTURES.
#    NOT A SINGLE OTHER EXPANSION HAPPENS UNTIL ALL FIXTURES ARE ON
#    THE SAME LEVEL AS THE ASM. HARD ENFORCEMENT."
#
# This gate is the freeze. Phosphoric expansion is forbidden until
# stage1 = stage0(phase0_compiler.phos) compiles every fixture in
# tools/verify/fixture_manifest.toml to byte-identical output as the
# ASM stub stage0 = phase0_stub. When this gate is green (N == total),
# byte-equal source ↔ ASM convergence is demonstrated and ASM
# retirement eligibility begins.
#
# For each fixture F:
#   stage0_out = stage0(F.phos)        # ASM stub on fixture
#   stage1_out = stage1(F.phos)        # Phosphoric source compiler on fixture
#   Assert sha256(stage0_out) == sha256(stage1_out)
#
# Exit 0: every fixture byte-equal. Freeze can lift.
# Exit 1: any fixture diverges. Freeze remains.
#
# Currently (2026-05-02) this gate is EXPECTED to fail for ~all
# fixtures because phase0_compiler.phos is mostly un-lowered.
# That failure IS the gap-exposure mechanism. Each session closes
# one fixture's gap; gate's N/total score is the convergence metric.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

stage0="untracked/internaldocs/phase0_producer/phase0_stub"
manifest="tools/verify/fixture_manifest.toml"
work_dir="build/phase0"

if [ ! -x "$stage0" ]; then
    echo "[source-asm] FAIL: stage0 (ASM stub) not built at $stage0" >&2
    exit 1
fi
if [ ! -r "$manifest" ]; then
    echo "[source-asm] FAIL: fixture manifest not found at $manifest" >&2
    exit 1
fi

mkdir -p "$work_dir"
stage1="$work_dir/pcc-stage1.bin"

# -------------------------------------------------------------------
# Build stage1 = stage0(phase0_compiler.phos).
# This is the Phosphoric source-side compiler binary; it was emitted
# by the ASM stub from phase0/phase0_compiler.phos.
# -------------------------------------------------------------------
if ! "$stage0" phase0/phase0_compiler.phos "$stage1" 2>/dev/null; then
    echo "[source-asm] FAIL: stage0 could not compile phase0_compiler.phos to stage1" >&2
    exit 1
fi
chmod +x "$stage1"

stage1_size="$(stat -c '%s' "$stage1")"
stage1_sha="$(sha256sum "$stage1" | awk '{print $1}')"

# -------------------------------------------------------------------
# Parse fixture manifest. Same parser shape as fixture_corpus.sh.
# -------------------------------------------------------------------
parsed="$(awk '
    BEGIN { in_block = 0 }
    /^\[\[fixture\]\]/ {
        if (in_block && name != "") {
            print name "|" source
        }
        in_block = 1
        name = ""; source = ""
        next
    }
    /^\[\[/ {
        if (in_block && name != "") {
            print name "|" source
        }
        in_block = 0
        next
    }
    in_block {
        if ($1 == "name")   { gsub(/[" \t]/, "", $3); name = $3 }
        if ($1 == "source") { gsub(/[" \t]/, "", $3); source = $3 }
    }
    END {
        if (in_block && name != "") {
            print name "|" source
        }
    }
' "$manifest")"

if [ -z "$parsed" ]; then
    echo "[source-asm] FAIL: manifest parsed to zero entries" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Per-fixture byte-equality check.
# -------------------------------------------------------------------
total=0
byte_equal=0
diverged=0
declare -a diverged_names=()

run_one() {
    local name="$1" source="$2"

    if [ ! -r "$source" ]; then
        printf '  %-30s SKIP  source missing\n' "$name"
        return
    fi

    local out0 out1
    out0="$(mktemp --suffix=.bin)"
    out1="$(mktemp --suffix=.bin)"
    local cleanup="rm -f \"$out0\" \"$out1\""

    # stage0 (ASM stub) on fixture
    if ! "$stage0" "$source" "$out0" 2>/dev/null; then
        printf '  %-30s SKIP  stage0 failed on source\n' "$name"
        eval "$cleanup"
        return
    fi

    # stage1 (Phosphoric source compiler) on fixture
    if ! "$stage1" "$source" "$out1" 2>/dev/null; then
        printf '  %-30s GAP   stage1 failed on source\n' "$name"
        diverged=$((diverged + 1))
        diverged_names+=("$name (stage1 reject)")
        eval "$cleanup"
        return
    fi

    local sha0 sha1
    sha0="$(sha256sum "$out0" | awk '{print $1}')"
    sha1="$(sha256sum "$out1" | awk '{print $1}')"

    if [ "$sha0" = "$sha1" ]; then
        byte_equal=$((byte_equal + 1))
        printf '  %-30s OK    byte-equal sha=%s\n' "$name" "${sha0:0:12}"
    else
        diverged=$((diverged + 1))
        local size0 size1
        size0="$(stat -c '%s' "$out0")"
        size1="$(stat -c '%s' "$out1")"
        diverged_names+=("$name (stage0=${size0}B/${sha0:0:8}, stage1=${size1}B/${sha1:0:8})")
        printf '  %-30s GAP   stage0=%sB sha=%s vs stage1=%sB sha=%s\n' \
            "$name" "$size0" "${sha0:0:12}" "$size1" "${sha1:0:12}"
    fi

    eval "$cleanup"
}

echo "============================================================"
echo "  Phosphoric STATE-ENFORCEMENT gate"
echo "  source ↔ ASM byte-equal convergence"
echo "  rule: docs/SOURCE_ASM_CONVERGENCE.md"
echo "============================================================"
echo "  stage0: $stage0"
echo "  stage1: $stage1 (size=$stage1_size sha=${stage1_sha:0:12})"
echo "------------------------------------------------------------"

while IFS='|' read -r name source; do
    [ -z "$name" ] && continue
    total=$((total + 1))
    run_one "$name" "$source"
done <<< "$parsed"

echo "------------------------------------------------------------"
printf '  %-30s %s / %s\n' "byte-equal source ↔ ASM:" "$byte_equal" "$total"
printf '  %-30s %s\n'      "diverged (gap remaining):" "$diverged"

if [ "$byte_equal" -eq "$total" ]; then
    echo "  [source-asm] OK — convergence achieved across $total fixtures"
    echo "  ASM retirement eligibility: ENGINEERING claim makeable"
    exit 0
fi

echo
echo "  [source-asm] FREEZE ACTIVE — $diverged of $total fixtures gap remaining"
echo
echo "  Per memory feedback_state_enforcement_byte_equal.md:"
echo "  no scope expansion (no new fixtures, shapes, gates, passes,"
echo "  Stream-C extensions, design docs) until this gate is green."
echo "  Only allowed direction: close one fixture's gap per session."
exit 1
