#!/usr/bin/env bash
# run_chain.sh — bootstrap chain runner.
#
# Reproduces the full stage0 → stage1 → stage2 → stage3 → stage4 chain
# on a given input source, recording each stage's hash + size + ELF
# header. Exits 0 on success regardless of whether the chain
# converges; the caller is responsible for interpreting the chain
# state. This is the OBSERVABILITY harness; gate enforcement lives
# in check_fixpoint_chain.sh.
#
# Usage:
#   tools/verify/run_chain.sh                    # default: phase0/phase0_compiler.phos
#   tools/verify/run_chain.sh <path.phos>        # any input source
#   tools/verify/run_chain.sh <path.phos> 5      # max chain depth (default 5)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

input="${1:-phase0/phase0_compiler.phos}"
max_depth="${2:-5}"

if [ ! -r "$input" ]; then
    echo "[run_chain] input not readable: $input" >&2
    exit 2
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

stage0="build/phase0/pcc-stage0.bin"
if [ ! -x "$stage0" ]; then
    echo "[run_chain] stage0 not built; running produce_stage0.sh"
    bash untracked/internaldocs/phase0_producer/produce_stage0.sh >/dev/null
fi

# Ensure stage0 is invocable
chmod +x "$stage0"

print_stage() {
    local idx="$1"
    local path="$2"
    if [ ! -s "$path" ]; then
        printf '  stage%d:  NOT PRODUCED\n' "$idx"
        return 1
    fi
    chmod +x "$path" 2>/dev/null || true
    local sz hash hdr
    sz="$(stat -c '%s' "$path")"
    hash="$(sha256sum "$path" | awk '{print $1}')"
    hdr="$(xxd -l 32 "$path" | head -2 | tr '\n' '|')"
    printf '  stage%d:  %s  (%5d B)\n' "$idx" "$hash" "$sz"
    return 0
}

echo "============================================================"
echo "  Phosphoric bootstrap chain trace"
echo "  input: $input"
echo "  max depth: $max_depth"
echo "============================================================"
echo

# Print stage0 (always present)
print_stage 0 "$stage0" || true

prev="$stage0"
for ((i=1; i<=max_depth; i++)); do
    out="$work/stage${i}.bin"
    "$prev" "$input" "$out" 2>"$work/stage${i}.err" || true

    if ! print_stage "$i" "$out"; then
        echo
        echo "[chain break] stage$((i-1)) did not produce stage${i}"
        if [ -s "$work/stage${i}.err" ]; then
            echo "[chain break] stderr:"
            sed 's/^/  /' "$work/stage${i}.err" >&2
        fi
        echo
        echo "============================================================"
        echo "Chain ended at depth $((i-1)) (max requested: $max_depth)"
        echo "============================================================"
        exit 0
    fi
    prev="$out"
done

echo
echo "============================================================"
echo "Chain reached max depth $max_depth without break"
echo "============================================================"
