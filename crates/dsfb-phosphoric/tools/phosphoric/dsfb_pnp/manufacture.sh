#!/usr/bin/env bash
# manufacture.sh — DSFB-PNP one-time manufacture for v0.3.
#
# Reads the canonical BOOTX64.EFI bytes from
#   tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin
# (2070 bytes), generates 2070 Phosphoric source files at
# tools/phosphoric/dsfb_pnp/byte_NNNN.phos, each
# `fn main() -> i32 { return BYTE; }`, then compiles each via
# phase0_stub into a 1081-byte Linux x86_64 ELF at
# tools/phosphoric/dsfb_pnp_artifacts/byte_NNNN.bin.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

canonical="tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin"
sources_dir="tools/phosphoric/dsfb_pnp"
artifacts_dir="tools/phosphoric/dsfb_pnp_artifacts"
producer="untracked/internaldocs/phase0_producer/produce_stage0.sh"

[ -r "$canonical" ] || { echo "[dsfb-pnp-mfr] missing canonical: $canonical" >&2; exit 2; }
[ -x "$producer" ] || { echo "[dsfb-pnp-mfr] missing or non-executable: $producer" >&2; exit 2; }

mkdir -p "$sources_dir" "$artifacts_dir"

total_bytes=$(wc -c < "$canonical")
echo "============================================================"
echo "  DSFB-PNP manufacture (v0.3 razor demo)"
echo "============================================================"
echo "  canonical          : $canonical"
echo "  canonical sha256   : $(sha256sum "$canonical" | awk '{print $1}')"
echo "  canonical bytes    : $total_bytes"
echo ""

mapfile -t byte_values < <(od -v -An -t u1 -w1 "$canonical" | awk '{print $1}')
if [ "${#byte_values[@]}" -ne "$total_bytes" ]; then
    echo "[dsfb-pnp-mfr] FAIL: byte_values count ${#byte_values[@]} != canonical $total_bytes" >&2
    exit 2
fi
for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    src="$sources_dir/byte_${nnnn}.phos"
    cat > "$src" <<EOF
module fixture.dsfb.byte_${nnnn};
profile boot;
fn main() -> i32 {
    return ${byte_values[$i]};
}
EOF
done
echo "[dsfb-pnp-mfr] Phase A: $total_bytes source files written"

mkdir -p build/phase0
fail=0
sample_indices=(0 1 100 500 1000 1500 2000 2069)

for ((i=0; i<total_bytes; i++)); do
    nnnn=$(printf '%04d' "$i")
    src="$sources_dir/byte_${nnnn}.phos"
    out="$artifacts_dir/byte_${nnnn}.bin"
    bash "$producer" "$src" >/dev/null 2>&1 || {
        echo "[dsfb-pnp-mfr] producer failed for $src (i=$i)" >&2
        fail=1
        break
    }
    cp -f build/phase0/pcc-stage0.bin "$out"
    chmod +x "$out"

    for s in "${sample_indices[@]}"; do
        if [ "$i" = "$s" ]; then
            expected="${byte_values[$i]}"
            actual=0
            "./$out" || actual=$?
            if [ "$actual" -ne "$expected" ]; then
                echo "[dsfb-pnp-mfr] FAIL spot-check at byte_${nnnn}: expected $expected, got $actual" >&2
                fail=1
            else
                echo "  spot-check byte_${nnnn}: OK (expected $expected, got $actual)"
            fi
        fi
    done

    if [ $((i % 250)) -eq 0 ] && [ "$i" -gt 0 ]; then
        echo "  ...compiled $i / $total_bytes"
    fi
done

[ "$fail" = "0" ] || exit 1

echo "[dsfb-pnp-mfr] OK — $total_bytes binaries manufactured"
