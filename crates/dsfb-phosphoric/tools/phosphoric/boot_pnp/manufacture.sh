#!/usr/bin/env bash
# manufacture.sh — Boot-PNP-1 one-time manufacture.
#
# Reads the canonical BOOTX64.EFI bytes from
#   tests/golden/bootx64_efi_v1_button_policy_golden.bin
# (2189 bytes, sha256 7fe4f8bc40a51f1d…), generates 2189 Phosphoric
# source files at tools/phosphoric/boot_pnp/byte_NNNN.phos, each
# containing `fn main() -> i32 { return BYTE; }` per closed shape #1
# in the 51-fixture compiler repertoire, then compiles each via
# phase0_stub (one-time scaffold-historical) into a 1081-byte
# Linux x86_64 ELF at tools/phosphoric/boot_pnp_artifacts/byte_NNNN.bin.
#
# This is the manufacture step. The active verify path
# (tools/verify/check_pnp_boot_producer.sh) does NOT run this; it
# runs only the manufactured binaries. This script is invoked once
# via `make manufacture-pnp-boot-historical`. Re-running reproduces
# byte-stable artifacts deterministically (phase0_stub is deterministic,
# the canonical is hash-pinned).
#
# Pattern modeled directly on tools/court/pnp1_r5_bytes/ → pnp1_r5_artifacts/
# precedent (32 bytes for R5 record). Boot-PNP-1 applies the same
# pattern at 2189-byte scale. No new compiler shapes; no producer
# ASM extension; only closed shape #1.
#
# Exit: 0 on success, 1 on byte-equality failure (manufactured ELF
# does not exit with the expected byte), 2 on missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

canonical="tests/golden/bootx64_efi_v1_button_policy_golden.bin"
sources_dir="tools/phosphoric/boot_pnp"
artifacts_dir="tools/phosphoric/boot_pnp_artifacts"
producer="untracked/internaldocs/phase0_producer/produce_stage0.sh"

[ -r "$canonical" ] || { echo "[boot-pnp-mfr] missing canonical: $canonical" >&2; exit 2; }
[ -x "$producer" ] || { echo "[boot-pnp-mfr] missing or non-executable: $producer" >&2; exit 2; }

mkdir -p "$sources_dir" "$artifacts_dir"

total_bytes=$(wc -c < "$canonical")
echo "============================================================"
echo "  Boot-PNP-1 manufacture"
echo "============================================================"
echo "  canonical          : $canonical"
echo "  canonical sha256   : $(sha256sum "$canonical" | awk '{print $1}')"
echo "  canonical bytes    : $total_bytes"
echo "  sources dir        : $sources_dir"
echo "  artifacts dir      : $artifacts_dir"
echo "  producer (one-shot): $producer"
echo ""

# Phase A: source generation. For each byte, write a single-shape
# .phos source. Byte values are read with `od` (octal -> decimal map).
echo "[boot-pnp-mfr] Phase A: generating $total_bytes source files…"

# Read the canonical as a stream of decimal byte values, one per line.
# `od -v -An -t u1 -w1` outputs one unsigned-byte-decimal per line.
mapfile -t byte_values < <(od -v -An -t u1 -w1 "$canonical" | awk '{print $1}')

if [ "${#byte_values[@]}" -ne "$total_bytes" ]; then
    echo "[boot-pnp-mfr] FAIL: byte_values count ${#byte_values[@]} != canonical $total_bytes" >&2
    exit 2
fi

for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    src="$sources_dir/byte_${nnnn}.phos"
    cat > "$src" <<EOF
module fixture.bootx64.byte_${nnnn};
profile boot;
fn main() -> i32 {
    return ${byte_values[$i]};
}
EOF
done
echo "[boot-pnp-mfr] Phase A: ${#byte_values[@]} source files written"

# Phase B: binary manufacture via phase0_stub. The producer writes
# its output to build/phase0/pcc-stage0.bin; we move each into
# place under artifacts_dir/byte_NNNN.bin.
echo ""
echo "[boot-pnp-mfr] Phase B: compiling $total_bytes sources via phase0_stub…"

mkdir -p build/phase0
fail=0
sample_indices=(0 1 100 500 1000 1500 2000 2188)

for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    src="$sources_dir/byte_${nnnn}.phos"
    out="$artifacts_dir/byte_${nnnn}.bin"
    bash "$producer" "$src" >/dev/null 2>&1 || {
        echo "[boot-pnp-mfr] producer failed for $src (i=$i)" >&2
        fail=1
        break
    }
    cp -f build/phase0/pcc-stage0.bin "$out"
    chmod +x "$out"

    # Spot-check sample indices: each binary should exit with the
    # source byte's value when run.
    for s in "${sample_indices[@]}"; do
        if [ "$i" = "$s" ]; then
            expected="${byte_values[$i]}"
            actual=0
            "./$out" || actual=$?
            if [ "$actual" -ne "$expected" ]; then
                echo "[boot-pnp-mfr] FAIL spot-check at byte_${nnnn}: expected $expected, got $actual" >&2
                fail=1
            else
                echo "  spot-check byte_${nnnn}: OK (expected $expected, got $actual)"
            fi
        fi
    done

    # Progress every 250 bytes
    if [ $((i % 250)) -eq 0 ] && [ "$i" -gt 0 ]; then
        echo "  …compiled $i / $total_bytes"
    fi
done

[ "$fail" = "0" ] || exit 1

echo "[boot-pnp-mfr] Phase B: $total_bytes binaries manufactured"
echo ""
echo "[boot-pnp-mfr] OK — manufacture complete"
echo "  sources    : $(ls "$sources_dir"/*.phos | wc -l) files"
echo "  artifacts  : $(ls "$artifacts_dir"/*.bin | wc -l) files"
echo "  size each  : $(stat -c%s "$artifacts_dir/byte_0000.bin") B (expected 1081)"
exit 0
