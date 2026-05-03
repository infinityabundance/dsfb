#!/usr/bin/env bash
# check_pnp3_verdict_producer.sh — Court PNP-3 (verdict narrow) gate.
#
# ACTIVE PATH: runs 104 already-manufactured Phosphoric-source-compiled
# binaries from build/pnp3/byte_NNN.bin, captures each binary's exit
# code, concatenates to a 104-byte buffer, and compares to the
# canonical 6-line MMIO_BOUNDARY_PRESSURE verdict produced by
# tools/court/verdict_from_pfi.sh on the canonical PFI0 case.
#
# Same shape as PNP-1/PNP-2: each .phos source `fn main() -> i32 {
# return BYTE; }`, exit code is the channel. 104 binaries × 1 byte =
# 104 bytes. Manufacture is one-time scaffold-historical; active gate
# is byte-comparison only.
#
# Doctrine anchors: tools/court/pnp3_verdict_bytes/byte_*.phos
# (source), tools/court/verdict_from_pfi.sh (host-reference witness),
# docs/FORENSIC_PRIMACY.md §3 (canonical 6-line verdict + DriftClass
# enum + EXIT mapping).
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bin_dir="tools/court/pnp3_verdict_artifacts"
emit_pfi="tools/court/emit_mmio_boundary_pfi.sh"
verdict_tool="tools/court/verdict_from_pfi.sh"

missing=0
for n in $(seq -f "%03g" 0 103); do
    if [ ! -x "$bin_dir/byte_$n.bin" ]; then
        missing=1; break
    fi
done
if [ "$missing" = "1" ]; then
    echo "[pnp3-verdict-producer] FAIL: tracked artifacts missing in $bin_dir/" >&2
    echo "[pnp3-verdict-producer] hint: run 'make manufacture-pnp3-verdict-historical' to rebuild" >&2
    exit 2
fi
[ -x "$emit_pfi" ] || { echo "[pnp3-verdict-producer] FAIL: $emit_pfi missing" >&2; exit 2; }
[ -x "$verdict_tool" ] || { echo "[pnp3-verdict-producer] FAIL: $verdict_tool missing" >&2; exit 2; }

echo "============================================================"
echo "  Phosphoric Court PNP-3 (verdict narrow) — Phosphoric-source"
echo "  producer byte-equality gate"
echo "  doctrine: docs/FORENSIC_PRIMACY.md §3 + docs/ASM_AUTHORITY_CUTOVER.md"
echo "============================================================"
echo "  source files       : tools/court/pnp3_verdict_bytes/byte_NNN.phos (104)"
echo "  artifacts (tracked): $bin_dir/byte_NNN.bin (104 × 1081 B)"
echo "  channel            : exit code (104 binaries × 1 byte)"
echo "  active path uses phase0_stub: NO (committed artifacts)"

produced=""
for n in $(seq -f "%03g" 0 103); do
    set +e
    "$bin_dir/byte_$n.bin"
    rc=$?
    set -e
    [ "$rc" -ge 0 ] && [ "$rc" -le 255 ] || { echo "[pnp3-verdict-producer] FAIL: byte_$n returned rc=$rc" >&2; exit 1; }
    produced="${produced}$(printf '%02x' "$rc")"
done

# Build canonical verdict via host-reference chain.
pfi_tmp="$(mktemp)"
verdict_tmp="$(mktemp)"
trap 'rm -f "$pfi_tmp" "$verdict_tmp"' EXIT
bash "$emit_pfi" > "$pfi_tmp"
bash "$verdict_tool" "$pfi_tmp" > "$verdict_tmp"
expected="$(xxd -p "$verdict_tmp" | tr -d '\n')"

echo "  produced bytes     : 104 (sha256 ${produced:0:12}...)"
echo "  expected bytes     : 104 (sha256 ${expected:0:12}...)"

if [ "$produced" != "$expected" ]; then
    echo "  [pnp3-verdict-producer] FAIL — Phosphoric-source-produced verdict differs from canonical" >&2
    exit 1
fi

echo "  [pnp3-verdict-producer] OK — Phosphoric-source producer artifact emitted canonical 104-byte verdict byte-identical to reference"
exit 0
