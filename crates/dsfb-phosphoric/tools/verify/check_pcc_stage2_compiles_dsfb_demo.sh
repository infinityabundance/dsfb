#!/usr/bin/env bash
# check_pcc_stage2_compiles_dsfb_demo.sh — v0.3 real-compile gate.
#
# Pins that pcc-stage2.bin (the v0.2-tagged Phosphoric compiler) compiles
# every apps/dsfb_demo/*.phos source byte-equal to phase0_stub-direct.
# This is the v0.3 analog of verify-pcc-stage2-encodes-demo: it proves
# the dsfb demo's source bundle is REAL-COMPILED through the bootstrap
# chain end-to-end, not just byte-encoded via PNP.
#
# The PNP archive (tools/phosphoric/dsfb_pnp/) handles the BOOTX64.EFI
# bootable bytes; this gate handles the source-level demo manifest
# (boot_entry / task_state / theorem_text) compiled to the canonical
# Pass-T-emit shape.
#
# Exit: 0 if every source matches; 1 on any divergence; 2 on missing
# dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

stage2="$repo_root/build/phase0/pcc-stage2.bin"
phase0_stub="$repo_root/untracked/internaldocs/phase0_producer/phase0_stub"

[ -x "$stage2" ]      || { echo "[pcc-stage2-compiles-dsfb] FAIL: missing $stage2" >&2; exit 2; }
[ -x "$phase0_stub" ] || { echo "[pcc-stage2-compiles-dsfb] FAIL: missing $phase0_stub" >&2; exit 2; }

sources=(
    "apps/dsfb_demo/boot_entry.phos"
    "apps/dsfb_demo/task_state.phos"
    "apps/dsfb_demo/theorem_text.phos"
)

trap 'rm -f /tmp/dsfb_real_compile_*.bin' EXIT

mismatch=0
total=0
echo "============================================================"
echo "  v0.3 real-compile gate — pcc-stage2.bin on apps/dsfb_demo/"
echo "============================================================"
for src in "${sources[@]}"; do
    [ -r "$src" ] || { echo "[pcc-stage2-compiles-dsfb] FAIL: missing $src" >&2; exit 2; }
    canon=$(mktemp /tmp/dsfb_real_compile_canon.XXXXXX)
    stage2_out=$(mktemp /tmp/dsfb_real_compile_stage2.XXXXXX)
    "$phase0_stub" "$src" "$canon" >/dev/null
    "$stage2"      "$src" "$stage2_out" >/dev/null
    sz=$(wc -c < "$stage2_out")
    sha=$(sha256sum "$stage2_out" | awk '{print $1}')
    if cmp -s "$stage2_out" "$canon"; then
        printf "  %-44s OK  (%4dB sha=%.16s)\n" "$src" "$sz" "$sha"
    else
        canon_sha=$(sha256sum "$canon" | awk '{print $1}')
        printf "  %-44s DIFFER stage2=%.16s phase0_stub=%.16s\n" \
            "$src" "$sha" "$canon_sha" >&2
        mismatch=$((mismatch + 1))
    fi
    rm -f "$canon" "$stage2_out"
    total=$((total + 1))
done

if [ "$mismatch" -ne 0 ]; then
    echo "[pcc-stage2-compiles-dsfb] FAIL: $mismatch / $total sources diverged" >&2
    exit 1
fi

echo "[pcc-stage2-compiles-dsfb] OK — pcc-stage2.bin compiles all $total apps/dsfb_demo/*.phos byte-equal to phase0_stub-direct"
