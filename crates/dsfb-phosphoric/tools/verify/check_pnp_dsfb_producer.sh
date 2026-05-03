#!/usr/bin/env bash
# check_pnp_dsfb_producer.sh — DSFB-PNP (v0.3 BOOTX64.EFI) verify gate.
#
# ACTIVE PATH: this gate does NOT invoke phase0_stub. It runs the
# 2070 already-manufactured Phosphoric-source-compiled binaries from
# tools/phosphoric/dsfb_pnp_artifacts/byte_NNNN.bin (committed
# scaffold-historical artifacts), captures each binary's exit code,
# and concatenates them into a 2070-byte buffer at
# build/pnp_dsfb_concat.bin. The buffer is then compared byte-equal
# to the canonical golden at
# tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin (sha256
# ec07ced3c205b767…). On byte-equal success the gate writes the
# concat buffer's path to stdout for build_uefi_demo.sh to copy
# into BOOTX64.EFI.
#
# Manufacture mechanism (out of scope for this gate; see Makefile
# target manufacture-pnp-dsfb-historical):
# - 2070 Phosphoric source files at tools/phosphoric/dsfb_pnp/byte_NNNN.phos
#   each declare `fn main() -> i32 { return BYTE_VALUE; }`. Each is
#   compiled by phase0_stub ONCE to produce a 1081-byte ELF whose
#   exit code equals BYTE_VALUE.
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bin_dir="tools/phosphoric/dsfb_pnp_artifacts"
canonical="tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin"
concat_out="build/pnp_dsfb_concat.bin"

[ -r "$canonical" ] || { echo "[pnp-dsfb-producer] FAIL: missing canonical: $canonical" >&2; exit 2; }
[ -d "$bin_dir" ]   || { echo "[pnp-dsfb-producer] FAIL: missing artifacts dir: $bin_dir" >&2; exit 2; }

total_bytes=$(wc -c < "$canonical")
expected_count="$total_bytes"
actual_count=$(ls "$bin_dir"/byte_*.bin 2>/dev/null | wc -l)
if [ "$actual_count" -ne "$expected_count" ]; then
    echo "[pnp-dsfb-producer] FAIL: artifact count $actual_count != expected $expected_count" >&2
    echo "[pnp-dsfb-producer] hint: run 'make manufacture-pnp-dsfb-historical' to rebuild from source" >&2
    exit 2
fi

echo "============================================================"
echo "  Phosphoric DSFB-PNP — v0.3 razor demo producer"
echo "  byte-equality gate (BOOTX64.EFI)"
echo "============================================================"
echo "  source files       : tools/phosphoric/dsfb_pnp/byte_NNNN.phos ($total_bytes)"
echo "  artifacts (tracked): $bin_dir/byte_NNNN.bin ($total_bytes × 1081 B)"
echo "  channel            : exit code ($total_bytes binaries × 1 byte)"
echo "  active path uses phase0_stub: NO (committed artifacts)"

mkdir -p build

tmp_concat="$(mktemp -p build pnp_dsfb_concat.XXXXXX)"
trap 'rm -f "$tmp_concat"' EXIT
> "$tmp_concat"
fail=0
for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    bin="$bin_dir/byte_${nnnn}.bin"
    if [ ! -x "$bin" ]; then
        echo "[pnp-dsfb-producer] FAIL: missing or non-executable $bin" >&2
        fail=1
        break
    fi
    rc=0
    "./$bin" || rc=$?
    if [ "$rc" -gt 255 ]; then
        echo "[pnp-dsfb-producer] FAIL: byte_${nnnn} exited rc=$rc (must be 0..255)" >&2
        fail=1
        break
    fi
    printf '%b' "\\x$(printf '%02x' "$rc")" >> "$tmp_concat"
done

[ "$fail" = "0" ] || exit 1

actual=$(stat -c '%s' "$tmp_concat")
[ "$actual" -eq "$total_bytes" ] || {
    echo "[pnp-dsfb-producer] FAIL: concat size $actual != expected $total_bytes" >&2
    exit 1
}

if ! cmp -s "$tmp_concat" "$canonical"; then
    echo "[pnp-dsfb-producer] FAIL: concat differs from canonical" >&2
    cmp -l "$tmp_concat" "$canonical" 2>&1 | head -3 >&2
    exit 1
fi

mv -f "$tmp_concat" "$concat_out"
trap - EXIT

produced_sha=$(sha256sum "$concat_out" | awk '{print $1}')
canonical_sha=$(sha256sum "$canonical"  | awk '{print $1}')

echo "  produced size      : $total_bytes B"
echo "  produced sha256    : $produced_sha"
echo "  canonical sha256   : $canonical_sha"
echo "  byte-equal         : YES"
echo "[pnp-dsfb-producer] OK — build/pnp_dsfb_concat.bin byte-identical to $canonical (sha256 $produced_sha)"
echo "$concat_out"
