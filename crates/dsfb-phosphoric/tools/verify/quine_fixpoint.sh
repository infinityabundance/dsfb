#!/usr/bin/env bash
# quine_fixpoint.sh — assert the byte-equal fixpoint property for the
# quine_self fixture. Compiles quine_self.phos to stage0, then iterates
# stage_N(any-input, stage_{N+1}) for N=0..2 and asserts
# stage_N == stage_{N+1} byte-equal at each step. Exit 0 if the chain
# converges, exit 1 if it diverges at any step.
#
# This is the byte-equal fixpoint gate that Path B has been driving
# toward (B-1 sys1, B-2 sys3, B-3 load32, B-4 quine).
# Closing this gate means stage_N's bytes equal stage_N's emitted output.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

producer="untracked/internaldocs/phase0_producer/phase0_stub"
src="tools/verify/fixtures/quine_self.phos"

if [ ! -x "$producer" ]; then
    echo "[quine_fixpoint] producer not built; building..."
    bash untracked/internaldocs/phase0_producer/build.sh >/dev/null
fi

if [ ! -r "$src" ]; then
    echo "[quine_fixpoint] missing fixture: $src" >&2; exit 2
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "============================================================"
echo "  Phosphoric byte-equal fixpoint gate (Path B-4)"
echo "============================================================"

stages=("$work/stage0.bin" "$work/stage1.bin" "$work/stage2.bin" "$work/stage3.bin")

# stage0 = producer(quine_self.phos)
"$producer" "$src" "${stages[0]}"
chmod +x "${stages[0]}"

# Iterate stage0 → stage1 → stage2 → stage3
for i in 0 1 2; do
    in="${stages[i]}"
    out="${stages[i + 1]}"
    "$in" /dev/null "$out"
    chmod +x "$out"
done

prev_sha=""
ok=1
for i in 0 1 2 3; do
    f="${stages[i]}"
    size=$(stat -c '%s' "$f")
    sha=$(sha256sum "$f" | awk '{print $1}')
    printf "  stage%d:  %s  (%5d B)\n" "$i" "$sha" "$size"
    if [ -n "$prev_sha" ] && [ "$prev_sha" != "$sha" ]; then
        echo "  [DIVERGED at stage$((i - 1)) → stage$i]"
        ok=0
    fi
    prev_sha="$sha"
done

echo "============================================================"
if [ "$ok" -eq 1 ]; then
    echo "  FIXPOINT: stage0 == stage1 == stage2 == stage3 byte-equal"
    exit 0
else
    echo "  REGRESSION: chain diverged"
    exit 1
fi
