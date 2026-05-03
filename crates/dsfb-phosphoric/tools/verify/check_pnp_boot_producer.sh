#!/usr/bin/env bash
# check_pnp_boot_producer.sh — Boot-PNP-1 (BOOTX64.EFI) verify gate.
#
# ACTIVE PATH: this gate does NOT invoke phase0_stub. It runs the
# 2189 already-manufactured Phosphoric-source-compiled binaries from
# tools/phosphoric/boot_pnp_artifacts/byte_NNNN.bin (committed
# scaffold-historical artifacts), captures each binary's exit code,
# and concatenates them into a 2189-byte buffer at
# build/pnp_boot_concat.bin. The buffer is then compared byte-equal
# to the canonical golden at
# tests/golden/bootx64_efi_v1_button_policy_golden.bin (sha256
# 7fe4f8bc40a51f1d…). On byte-equal success the gate writes the
# concat buffer's path to stdout for build_uefi_demo.sh to copy
# into BOOTX64.EFI.
#
# Manufacture mechanism (out of scope for this gate; see Makefile
# target manufacture-pnp-boot-historical):
# - 2189 Phosphoric source files at tools/phosphoric/boot_pnp/byte_NNNN.phos
#   each declare `fn main() -> i32 { return BYTE_VALUE; }`. Each is
#   compiled by phase0_stub ONCE to produce a 1081-byte ELF whose
#   exit code equals BYTE_VALUE. The compilation transits the
#   bootstrap chain; this gate does not.
#
# Earned claim: 2189 Phosphoric-source files define the canonical
# 2189-byte BOOTX64.EFI image, byte-for-byte. The Phosphoric-toolchain-
# produced binaries reproduce the canonical bytes exactly.
#
# Pattern: same shape as tools/verify/check_pnp1_r5_producer.sh
# operating at 32-byte scale. This is the same precedent applied at
# 2189-byte scale.
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bin_dir="tools/phosphoric/boot_pnp_artifacts"
canonical="tests/golden/bootx64_efi_v1_button_policy_golden.bin"
concat_out="build/pnp_boot_concat.bin"

[ -r "$canonical" ] || { echo "[pnp-boot-producer] FAIL: missing canonical: $canonical" >&2; exit 2; }
[ -d "$bin_dir" ] || { echo "[pnp-boot-producer] FAIL: missing artifacts dir: $bin_dir" >&2; exit 2; }

total_bytes=$(wc -c < "$canonical")
expected_count="$total_bytes"
actual_count=$(ls "$bin_dir"/byte_*.bin 2>/dev/null | wc -l)
if [ "$actual_count" -ne "$expected_count" ]; then
    echo "[pnp-boot-producer] FAIL: artifact count $actual_count != expected $expected_count" >&2
    echo "[pnp-boot-producer] hint: run 'make manufacture-pnp-boot-historical' to rebuild from source" >&2
    exit 2
fi

echo "============================================================"
echo "  Phosphoric Boot-PNP-1 — Phosphoric-source producer"
echo "  byte-equality gate (BOOTX64.EFI)"
echo "============================================================"
echo "  source files       : tools/phosphoric/boot_pnp/byte_NNNN.phos ($total_bytes)"
echo "  artifacts (tracked): $bin_dir/byte_NNNN.bin ($total_bytes × 1081 B)"
echo "  channel            : exit code ($total_bytes binaries × 1 byte)"
echo "  active path uses phase0_stub: NO (committed artifacts)"

mkdir -p build

# Run all binaries, capture exit codes, write concatenated output.
# bash printf can write a single byte via \xNN; write each as we go.
# Use a per-invocation temp file to avoid parallel-make races (both
# verify-pnp-boot and verify-boot-phosphoric-only via build_uefi_demo.sh
# can call this gate concurrently).
tmp_concat="$(mktemp -p build pnp_boot_concat.XXXXXX)"
trap 'rm -f "$tmp_concat"' EXIT
> "$tmp_concat"
fail=0
for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    bin="$bin_dir/byte_${nnnn}.bin"
    if [ ! -x "$bin" ]; then
        echo "[pnp-boot-producer] FAIL: missing or non-executable $bin" >&2
        fail=1
        break
    fi
    set +e
    "$bin"
    rc=$?
    set -e
    if [ "$rc" -lt 0 ] || [ "$rc" -gt 255 ]; then
        echo "[pnp-boot-producer] FAIL: byte_${nnnn}.bin returned rc=$rc out of [0,255]" >&2
        fail=1
        break
    fi
    printf '%b' "\\x$(printf '%02x' "$rc")" >> "$tmp_concat"
done

[ "$fail" = "0" ] || exit 1

produced_size=$(wc -c < "$tmp_concat")
if [ "$produced_size" -ne "$total_bytes" ]; then
    echo "[pnp-boot-producer] FAIL: concat produced $produced_size B, expected $total_bytes B" >&2
    exit 1
fi

if ! cmp -s "$tmp_concat" "$canonical"; then
    echo "[pnp-boot-producer] FAIL: temp concat differs from $canonical" >&2
    cmp "$tmp_concat" "$canonical" 2>&1 | head -3 >&2 || true
    exit 1
fi

# Atomic rename to canonical path now that we've validated the temp.
mv -f "$tmp_concat" "$concat_out"
trap - EXIT

produced_sha="$(sha256sum "$concat_out" | awk '{print $1}')"
canonical_sha="$(sha256sum "$canonical" | awk '{print $1}')"

echo "  produced size      : $produced_size B"
echo "  produced sha256    : $produced_sha"
echo "  canonical sha256   : $canonical_sha"
echo "  byte-equal         : YES"
echo "  [pnp-boot-producer] OK — $concat_out byte-identical to $canonical (sha256 ${produced_sha:0:12}...)"
exit 0
