#!/usr/bin/env bash
set -euo pipefail
#
# run_dsfb_demo.sh — v0.3 QEMU runner for the razor DSFB-theorem demo.
#
# Boots tools/image-builder/build_dsfb_demo.sh's BOOTX64.EFI under
# QEMU/OVMF, captures the debug-port output, asserts the seven trace
# markers (4 phosphoric: ... boundary lines + 3 theorem anchors), and
# exits 0 on a clean halt via debug_exit_port=0xf4 with code 0.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="$repo_root/build/uefi-demo/dsfb"
esp_dir="$build_dir/esp"
esp_image="$build_dir/esp.img"
ovmf_vars="$build_dir/OVMF_VARS.4m.fd"
debug_log="$build_dir/qemu-debug.log"
records_bin="$build_dir/dsfb_demo_records_runtime.bin"
pfi_out="$build_dir/dsfb_demo.pfi"
qemu_timeout="${PHOSPHORIC_QEMU_TIMEOUT:-30s}"

find_ovmf_file() {
    local configured="$1"
    shift

    if [ -n "$configured" ] && [ -f "$configured" ]; then
        printf '%s\n' "$configured"
        return 0
    fi

    local candidate
    for candidate in "$@"; do
        if [ -f "$candidate" ]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    return 1
}

bash "$repo_root/tools/image-builder/build_dsfb_demo.sh" >/dev/null

ovmf_mode=""
ovmf_code=""
ovmf_vars_src=""
ovmf_legacy=""

if ovmf_code="$(find_ovmf_file "${PHOSPHORIC_OVMF_CODE:-}" \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/OVMF/OVMF_CODE.fd)" && \
   ovmf_vars_src="$(find_ovmf_file "${PHOSPHORIC_OVMF_VARS:-}" \
    /usr/share/edk2/x64/OVMF_VARS.4m.fd \
    /usr/share/OVMF/OVMF_VARS_4M.fd \
    /usr/share/OVMF/OVMF_VARS.fd)"; then
    ovmf_mode="split"
elif ovmf_legacy="$(find_ovmf_file "${PHOSPHORIC_OVMF:-}" \
    /usr/share/edk2/x64/OVMF.4m.fd \
    /usr/share/OVMF/OVMF.fd)"; then
    ovmf_mode="legacy"
else
    printf 'run_dsfb_demo.sh: missing OVMF firmware\n' >&2
    printf '  set PHOSPHORIC_OVMF_CODE and PHOSPHORIC_OVMF_VARS, or set PHOSPHORIC_OVMF=/path/to/OVMF.fd\n' >&2
    exit 1
fi

rm -f "$debug_log" "$records_bin" "$pfi_out" "$esp_image" "$ovmf_vars"
truncate -s 64M "$esp_image"
mkfs.vfat "$esp_image" >/dev/null
mmd -i "$esp_image" ::/EFI ::/EFI/BOOT
mcopy -i "$esp_image" "$esp_dir/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI

qemu_firmware_args=()
case "$ovmf_mode" in
    split)
        cp "$ovmf_vars_src" "$ovmf_vars"
        qemu_firmware_args=(
            -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code"
            -drive "if=pflash,format=raw,file=$ovmf_vars"
        )
        ;;
    legacy)
        qemu_firmware_args=(
            -drive "if=pflash,format=raw,readonly=on,file=$ovmf_legacy"
        )
        ;;
esac

set +e
TMPDIR=/tmp timeout "$qemu_timeout" qemu-system-x86_64 \
    "${qemu_firmware_args[@]}" \
    -drive format=raw,file="$esp_image" \
    -nographic \
    -no-reboot \
    -chardev "file,id=cdtext,path=$debug_log" \
    -device isa-debugcon,chardev=cdtext,iobase=0x402 \
    -chardev "file,id=cddata,path=$records_bin" \
    -device isa-debugcon,chardev=cddata,iobase=0x500 \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    < /dev/null > "$build_dir/qemu-stdout.log" 2>&1
qemu_rc=$?
set -e

# QEMU isa-debug-exit returns (port_value << 1) | 1. A clean halt with
# code 0 yields rc=1. Anything else is failure.
if [ "$qemu_rc" -ne 1 ]; then
    printf 'run_dsfb_demo.sh: QEMU rc=%s (expected 1 = clean debug-exit code 0)\n' "$qemu_rc" >&2
    exit 1
fi

[ -f "$debug_log" ] || {
    printf 'run_dsfb_demo.sh: missing debug log %s\n' "$debug_log" >&2
    exit 1
}

required_markers=(
    'phosphoric: dsfb demo entry'
    'DSFB (Drift-Slew Fusion Bootstrap)'
    'Endoduction is the 4th mode of Inference.'
    'byte-deterministic certificate.'
    'phosphoric: task accepted'
    'phosphoric: residuals emitted'
    'phosphoric: dsfb demo halt'
)

missing=0
for marker in "${required_markers[@]}"; do
    if ! grep -F -q "$marker" "$debug_log"; then
        printf 'run_dsfb_demo.sh: missing marker %q in %s\n' "$marker" "$debug_log" >&2
        missing=$((missing + 1))
    fi
done
[ "$missing" -eq 0 ] || exit 1

# Records check: exactly 96 bytes (3 × 32B residuals) on debug_data_port.
[ -f "$records_bin" ] || {
    printf 'run_dsfb_demo.sh: missing records capture %s\n' "$records_bin" >&2
    exit 1
}
records_size=$(wc -c < "$records_bin")
[ "$records_size" -eq 96 ] || {
    printf 'run_dsfb_demo.sh: residual records size %s != 96 (3 × 32B)\n' "$records_size" >&2
    exit 1
}

# Encode the PFI0 case file from runtime-captured records.
bash "$repo_root/tools/verify/encode_pfi.sh" \
    "$records_bin" \
    "$build_dir/linked-artifact.txt" \
    "$esp_dir/EFI/BOOT/BOOTX64.EFI" \
    "$pfi_out" \
    >/dev/null

# Existing PFI layout gate must accept the runtime-emitted file
# unchanged (chain_hash chain re-derivation, magic, count, size, etc.).
bash "$repo_root/tools/verify/check_pfi_layout.sh" "$pfi_out" >/dev/null

# If a committed golden exists, runtime PFI must byte-equal it.
golden_pfi="$repo_root/tests/golden/dsfb_demo.pfi"
if [ -f "$golden_pfi" ]; then
    if ! cmp -s "$pfi_out" "$golden_pfi"; then
        printf 'run_dsfb_demo.sh: runtime PFI %s does not match golden %s\n' "$pfi_out" "$golden_pfi" >&2
        cmp -l "$pfi_out" "$golden_pfi" 2>&1 | head -5 >&2
        exit 1
    fi
fi

printf 'phosphoric: dsfb v0.3 demo booted; theorem printed; 3 typed residuals emitted; pfi case file produced (%s); halted clean (qemu rc=%d)\n' \
    "$(sha256sum "$pfi_out" | awk '{print substr($1,1,16)}')..." "$qemu_rc"
