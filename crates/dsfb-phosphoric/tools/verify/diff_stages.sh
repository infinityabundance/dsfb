#!/usr/bin/env bash
# diff_stages.sh — objdump-aligned diff between two stage binaries.
#
# Disassembles each stage's loaded segment via objdump (binary mode,
# i386:x86-64, intel syntax, vma 0x400000) and emits a side-by-side
# diff that respects instruction boundaries. Intended for the chain
# fixpoint debugging loop: stage_N != stage_{N+1} happens; this tells
# you exactly which instructions differ.
#
# Usage:
#   tools/verify/diff_stages.sh <bin_a> <bin_b>
#   tools/verify/diff_stages.sh                     # default: stage1 vs stage2
#   tools/verify/diff_stages.sh stage2 stage3       # named-shorthand
#
# Named shorthand: "stage<N>" resolves to the path the chain runner
# would have produced. The chain is rebuilt fresh each invocation
# (no caching) so this script is its own oracle.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

resolve_stage() {
    local arg="$1"
    case "$arg" in
        stage0|0)  echo "build/phase0/pcc-stage0.bin"; return ;;
        stage*)    echo "$arg" ;;  # caller supplied the resolved path or a literal name
        *)         echo "$arg" ;;
    esac
}

if [ $# -eq 0 ]; then
    a="stage1"
    b="stage2"
elif [ $# -eq 2 ]; then
    a="$1"
    b="$2"
else
    echo "usage: $0 [<a> <b>]" >&2
    exit 2
fi

# If either arg is a stage<N> name, rebuild the chain to materialize them.
if [[ "$a" == stage* ]] || [[ "$b" == stage* ]]; then
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT

    if [ ! -x build/phase0/pcc-stage0.bin ]; then
        bash untracked/internaldocs/phase0_producer/produce_stage0.sh >/dev/null
    fi
    chmod +x build/phase0/pcc-stage0.bin

    # Build chain up to the deepest stage requested
    deepest=0
    for name in "$a" "$b"; do
        [[ "$name" =~ ^stage([0-9]+)$ ]] || continue
        n="${BASH_REMATCH[1]}"
        [ "$n" -gt "$deepest" ] && deepest="$n"
    done

    cp build/phase0/pcc-stage0.bin "$work/stage0.bin"
    chmod +x "$work/stage0.bin"
    for ((i=1; i<=deepest; i++)); do
        prev="$work/stage$((i-1)).bin"
        next="$work/stage${i}.bin"
        "$prev" phase0/phase0_compiler.phos "$next" 2>/dev/null || true
        if [ ! -s "$next" ]; then
            echo "[diff_stages] chain ended before stage${i} — cannot compare" >&2
            exit 2
        fi
        chmod +x "$next"
    done

    [[ "$a" == stage* ]] && a="$work/${a}.bin"
    [[ "$b" == stage* ]] && b="$work/${b}.bin"
fi

if [ ! -r "$a" ]; then echo "[diff_stages] not readable: $a" >&2; exit 2; fi
if [ ! -r "$b" ]; then echo "[diff_stages] not readable: $b" >&2; exit 2; fi

a_hash="$(sha256sum "$a" | awk '{print $1}')"
b_hash="$(sha256sum "$b" | awk '{print $1}')"
a_size="$(stat -c '%s' "$a")"
b_size="$(stat -c '%s' "$b")"

echo "==================================================="
echo "A: $a"
echo "   sha256: $a_hash"
echo "   size:   $a_size B"
echo "B: $b"
echo "   sha256: $b_hash"
echo "   size:   $b_size B"
echo "==================================================="

if [ "$a_hash" = "$b_hash" ]; then
    echo "[diff_stages] IDENTICAL — sha256 matches"
    exit 0
fi

# Disassemble both
asm_a="$(mktemp)"
asm_b="$(mktemp)"
trap 'rm -f "$asm_a" "$asm_b"' EXIT

objdump -D -b binary -m i386:x86-64 -M intel "$a" --adjust-vma=0x400000 2>/dev/null \
    | sed -E 's/^ */ /' \
    > "$asm_a"
objdump -D -b binary -m i386:x86-64 -M intel "$b" --adjust-vma=0x400000 2>/dev/null \
    | sed -E 's/^ */ /' \
    > "$asm_b"

echo
echo "------ Unified diff (truncated to first 200 differing lines) ------"
echo
diff -u "$asm_a" "$asm_b" | head -200 || true
echo
echo "------ Byte diff summary ------"
cmp -l "$a" "$b" 2>/dev/null | head -20 || true
diff_count="$(cmp -l "$a" "$b" 2>/dev/null | wc -l || echo 0)"
echo "Total differing bytes: $diff_count"
