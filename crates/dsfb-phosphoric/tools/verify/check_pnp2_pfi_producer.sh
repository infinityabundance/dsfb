#!/usr/bin/env bash
# check_pnp2_pfi_producer.sh — Court PNP-2 (PFI0 narrow) verify gate.
#
# ACTIVE PATH: runs 192 already-manufactured Phosphoric-source-compiled
# binaries from build/pnp2/byte_NNN.bin, captures each binary's exit
# code, concatenates to a 192-byte buffer, and compares to the
# canonical PFI0 case (declared_lo=0x1000, declared_hi=0x10FF,
# observed=0x1100) produced by tools/court/emit_mmio_boundary_pfi.sh.
#
# Same shape as PNP-1: each .phos source `fn main() -> i32 { return
# BYTE; }`, exit code is the channel. 192 binaries × 1 byte = 192
# bytes. Manufacture is one-time scaffold-historical (uses phase0_stub
# once per source); active gate is byte-comparison only.
#
# Doctrine anchors: tools/court/pnp2_pfi_bytes/byte_*.phos (source),
# tools/court/emit_mmio_boundary_pfi.sh (host-reference witness),
# docs/PFI0.md (PFI0 layout).
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bin_dir="tools/court/pnp2_pfi_artifacts"
emit_pfi="tools/court/emit_mmio_boundary_pfi.sh"

missing=0
for n in $(seq -f "%03g" 0 191); do
    if [ ! -x "$bin_dir/byte_$n.bin" ]; then
        missing=1; break
    fi
done
if [ "$missing" = "1" ]; then
    echo "[pnp2-pfi-producer] FAIL: tracked artifacts missing in $bin_dir/" >&2
    echo "[pnp2-pfi-producer] hint: run 'make manufacture-pnp2-pfi-historical' to rebuild" >&2
    exit 2
fi
[ -x "$emit_pfi" ] || { echo "[pnp2-pfi-producer] FAIL: $emit_pfi missing" >&2; exit 2; }

echo "============================================================"
echo "  Phosphoric Court PNP-2 (PFI0 narrow) — Phosphoric-source"
echo "  producer byte-equality gate"
echo "  doctrine: docs/PFI0.md + docs/ASM_AUTHORITY_CUTOVER.md"
echo "============================================================"
echo "  source files       : tools/court/pnp2_pfi_bytes/byte_NNN.phos (192)"
echo "  artifacts (tracked): $bin_dir/byte_NNN.bin (192 × 1081 B)"
echo "  channel            : exit code (192 binaries × 1 byte)"
echo "  active path uses phase0_stub: NO (committed artifacts)"

produced=""
for n in $(seq -f "%03g" 0 191); do
    set +e
    "$bin_dir/byte_$n.bin"
    rc=$?
    set -e
    [ "$rc" -ge 0 ] && [ "$rc" -le 255 ] || { echo "[pnp2-pfi-producer] FAIL: byte_$n returned rc=$rc" >&2; exit 1; }
    produced="${produced}$(printf '%02x' "$rc")"
done

expected="$(bash "$emit_pfi" | xxd -p | tr -d '\n')"

echo "  produced bytes     : 192 (sha256 ${produced:0:12}...)"
echo "  expected bytes     : 192 (sha256 ${expected:0:12}...)"

if [ "$produced" != "$expected" ]; then
    echo "  [pnp2-pfi-producer] FAIL — Phosphoric-source-produced PFI0 bytes differ from canonical" >&2
    exit 1
fi

echo "  [pnp2-pfi-producer] OK — Phosphoric-source producer artifact emitted canonical 192-byte PFI0 case byte-identical to reference"
exit 0
