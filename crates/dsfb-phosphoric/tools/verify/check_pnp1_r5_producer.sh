#!/usr/bin/env bash
# check_pnp1_r5_producer.sh — Court PNP-1 (R5 narrow) verify gate.
#
# ACTIVE PATH: this gate does NOT invoke phase0_stub. It runs the 32
# already-manufactured Phosphoric-source-compiled binaries from
# tools/court/pnp1_r5_artifacts/byte_NN.bin (committed scaffold-
# manufactured artifacts), captures each binary's exit code, and
# concatenates them into a 32-byte buffer. The buffer is then
# compared to the canonical R5 mmio_touch record (4096, 4351, 4352)
# produced by tools/court/emit_r5_record.sh (host-reference witness).
# The committed artifacts mean the active workflow is ASM-free even
# on a fresh clone — no manufacture step required for verification.
#
# Manufacture mechanism (out of scope for this gate; see Makefile
# target manufacture-pnp1-r5-historical):
# - 32 Phosphoric source files at tools/court/pnp1_r5_bytes/byte_NN.phos
#   each declare `fn main() -> i32 { return BYTE_VALUE; }`. Each is
#   compiled by phase0_stub ONCE to produce a 1081-byte ELF whose exit
#   code equals BYTE_VALUE. The compilation transits the historical
#   ASM scaffold; this gate does not. After manufacture the binaries
#   are byte-stable scaffold-historical artifacts.
#
# Forensic claim earned: 32 Phosphoric-source files define the
# canonical 32-byte R5 mmio_touch record, byte-for-byte. The
# Phosphoric-toolchain-produced binaries reproduce the host
# reference emitter's output.
#
# Forensic claim NOT earned: ASM-free production (the binaries were
# manufactured via phase0_stub once); on-device chain_step or SHA-256
# computation; full self-host; PFI0 emission; verdict emission;
# edge execution. Byte emission via SYS_WRITE is still not lowered
# in the 51-shape repertoire; the channel here is exit code, 32
# binaries × 1 byte each.
#
# Doctrine anchors: tools/court/pnp1_r5_bytes/byte_*.phos (source),
# tools/court/emit_r5_record.sh (host-reference witness),
# kernel/residual.phos (R5 spec + chain_step),
# docs/ASM_AUTHORITY_CUTOVER.md (cutover policy + manufacture
# boundary).
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bin_dir="tools/court/pnp1_r5_artifacts"
emit_r5="tools/court/emit_r5_record.sh"

# Verify all 32 manufactured binaries exist.
missing=0
for n in $(seq -f "%02g" 0 31); do
    if [ ! -x "$bin_dir/byte_$n.bin" ]; then
        echo "[pnp1-r5-producer] FAIL: missing manufactured artifact $bin_dir/byte_$n.bin" >&2
        missing=1
    fi
done
if [ "$missing" = "1" ]; then
    echo "[pnp1-r5-producer] tracked artifacts missing — repository may be incomplete" >&2
    echo "[pnp1-r5-producer] hint: run 'make manufacture-pnp1-r5-historical' to rebuild from source" >&2
    exit 2
fi

if [ ! -x "$emit_r5" ]; then
    echo "[pnp1-r5-producer] FAIL: host-reference witness missing: $emit_r5" >&2
    exit 2
fi

echo "============================================================"
echo "  Phosphoric Court PNP-1 (R5 narrow) — Phosphoric-source"
echo "  producer byte-equality gate"
echo "  doctrine: docs/ASM_AUTHORITY_CUTOVER.md + kernel/residual.phos"
echo "============================================================"
echo "  source files       : tools/court/pnp1_r5_bytes/byte_NN.phos (32)"
echo "  artifacts (tracked): $bin_dir/byte_NN.bin (32 × 1081 B)"
echo "  channel            : exit code (32 binaries × 1 byte)"
echo "  active path uses phase0_stub: NO (committed artifacts)"

# Run all 32 binaries, capture exit codes, build produced hex.
produced_hex=""
for n in $(seq -f "%02g" 0 31); do
    set +e
    "$bin_dir/byte_$n.bin"
    rc=$?
    set -e
    if [ "$rc" -lt 0 ] || [ "$rc" -gt 255 ]; then
        echo "[pnp1-r5-producer] FAIL: byte_$n.bin returned rc=$rc out of [0,255]" >&2
        exit 1
    fi
    produced_hex="${produced_hex}$(printf '%02x' "$rc")"
done

# Get canonical R5 bytes from host-reference witness.
expected_hex="$(bash "$emit_r5" 4096 4351 4352)"

echo "  produced (hex)     : $produced_hex"
echo "  expected (hex)     : $expected_hex"

if [ "$produced_hex" != "$expected_hex" ]; then
    echo "  [pnp1-r5-producer] FAIL — Phosphoric-source-produced bytes differ from canonical R5" >&2
    exit 1
fi

echo "  [pnp1-r5-producer] OK — Phosphoric-source producer artifact emitted canonical 32-byte R5 record byte-identical to reference"
exit 0
