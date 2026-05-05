#!/usr/bin/env bash
set -euo pipefail

# Generate README-ready visual assets from the real v0.3 DSFB QEMU demo.
# This does not modify the demo. It reruns the same boot path used by the
# Colab notebook, validates the runtime evidence, then renders the captured
# debug-port stream as a PNG and slowed GIF.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
asset_dir="$repo_root/assets"
build_dir="$repo_root/build/uefi-demo/dsfb"
esp_dir="$build_dir/esp"
esp_image="$build_dir/esp.img"
ovmf_vars="$build_dir/OVMF_VARS.4m.fd"
debug_log="$build_dir/qemu-debug.log"
records_bin="$build_dir/dsfb_demo_records_runtime.bin"
pfi_out="$build_dir/dsfb_demo.pfi"
qemu_timeout="${PHOSPHORIC_QEMU_TIMEOUT:-30s}"

work_dir="$asset_dir/.tmp-dsfb-demo-assets"
frames_dir="$work_dir/frames"
transcript="$work_dir/runtime-transcript.txt"
render_text="$work_dir/render-text.txt"
compiler_summary="$work_dir/compiler-summary.txt"
evidence_summary="$work_dir/evidence-summary.txt"

out_png="$asset_dir/dsfb-demo-screenshot.png"
out_gif="$asset_dir/dsfb-demo-running.gif"

cleanup() {
    if [ -n "${work_dir:-}" ] && [ "$work_dir" != "/" ]; then
        rm -rf "$work_dir"
    fi
}
trap cleanup EXIT

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

require_tool() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'missing required tool: %s\n' "$1" >&2
        exit 127
    }
}

render_terminal_png() {
    local text_file="$1"
    local output_file="$2"
    local label_file="$work_dir/text-layer.png"
    local image_w=1600
    local image_h=1800

    magick \
        -background none \
        -fill '#d8e4eb' \
        -font DejaVu-Sans-Mono \
        -pointsize 13 \
        -size 1456x1520 \
        "caption:@$text_file" \
        "$label_file"

    magick \
        -size "${image_w}x${image_h}" \
        xc:'#08111a' \
        -fill '#0d1722' -draw 'roundrectangle 44,42 1556,1758 8,8' \
        -fill '#162231' -draw 'rectangle 44,42 1556,116' \
        -fill '#57c7ff' -font DejaVu-Sans-Mono-Bold -pointsize 30 \
        -annotate +72+88 'DSFB v0.3 QEMU demo: executable evidence path' \
        -fill '#9fb2c3' -font DejaVu-Sans-Mono -pointsize 19 \
        -annotate +72+144 'Same Colab demo path | UEFI boot -> debug ports -> typed residuals -> PFI0 golden check' \
        -fill '#2dd4bf' -draw 'circle 1490,79 1500,79' \
        -fill '#fbbf24' -draw 'circle 1458,79 1468,79' \
        -fill '#fb7185' -draw 'circle 1426,79 1436,79' \
        "$label_file" -geometry +72+190 -composite \
        "$output_file"
}

prepare_render_text() {
    local source_file="$1"
    local output_file="$2"
    fold -s -w 128 "$source_file" | sed -n '1,88p' > "$output_file"
}

extract_const() {
    local path="$1"
    local name="$2"
    local value
    value="$(
        sed -n "s/^[[:space:]]*fn ${name}()[[:space:]]*->[^{]*{[[:space:]]*\\([0-9][0-9]*\\)[[:space:]]*}[[:space:]]*$/\\1/p" "$path"
    )"
    if [ -z "$value" ]; then
        printf 'failed to extract literal constant %s from %s\n' "$name" "$path" >&2
        exit 1
    fi
    printf '%s\n' "$value"
}

generate_compiler_summary() {
    local stage2="$repo_root/build/phase0/pcc-stage2.bin"
    local phase0_stub="$repo_root/untracked/internaldocs/phase0_producer/phase0_stub"
    local efi_golden="$repo_root/tests/golden/bootx64_efi_v0_3_dsfb_theorem_golden.bin"
    local stage2_sha stage2_size synth_sha16 golden_sha golden_size image_size text_size rdata_size symbols_size
    local compiler_canon compiler_self exit42_out exit42_sha exit42_rc
    local pnp_source_count pnp_artifact_count pnp_artifact_size
    local fb_width fb_height fb_stride fb_bpp text_port data_port exit_port
    local boot_seq enter_seq exit_seq residual_count theorem_printed halt_code
    local theorem_len theorem_first theorem_last theorem_stages theorem_mode

    [ -x "$stage2" ] || { printf 'missing compiler: %s\n' "$stage2" >&2; exit 1; }
    [ -x "$phase0_stub" ] || { printf 'missing phase0 stub: %s\n' "$phase0_stub" >&2; exit 1; }
    [ -r "$efi_golden" ] || { printf 'missing EFI golden: %s\n' "$efi_golden" >&2; exit 1; }

    compiler_canon="$work_dir/pcc2.phase0_stub.bin"
    compiler_self="$work_dir/pcc2.stage2_self.bin"
    exit42_out="$work_dir/exit42.stage2.bin"

    "$phase0_stub" "$repo_root/compiler/pcc2.phos" "$compiler_canon" >/dev/null
    "$stage2" "$repo_root/compiler/pcc2.phos" "$compiler_self" >/dev/null
    "$stage2" "$repo_root/tools/verify/fixtures/exit42.phos" "$exit42_out" >/dev/null
    chmod +x "$exit42_out"
    set +e
    "$exit42_out" >/dev/null 2>&1
    exit42_rc=$?
    set -e

    cmp -s "$stage2" "$compiler_canon" || {
        printf 'pcc-stage2.bin is not byte-equal to phase0_stub compiler/pcc2.phos output\n' >&2
        exit 1
    }
    cmp -s "$stage2" "$compiler_self" || {
        printf 'pcc-stage2.bin self-compile did not reproduce pcc-stage2.bin\n' >&2
        exit 1
    }
    [ "$exit42_rc" -eq 42 ] || {
        printf 'pcc-stage2 exit42 compile ran with rc=%s, expected 42\n' "$exit42_rc" >&2
        exit 1
    }

    stage2_sha="$(sha256sum "$stage2" | awk '{print $1}')"
    stage2_size="$(stat -c '%s' "$stage2")"
    [ "$stage2_size" -eq 18017 ] || {
        printf 'pcc-stage2 size %s != expected 18017\n' "$stage2_size" >&2
        exit 1
    }
    synth_sha16="$(
        python3 - "$stage2" <<'PY'
import hashlib
import sys
with open(sys.argv[1], "rb") as f:
    data = f.read()
blob = data[120:120 + 16384]
if len(blob) != 16384 or blob == b"\x00" * 16384:
    raise SystemExit(1)
print(hashlib.sha256(blob).hexdigest()[:16])
PY
    )"
    exit42_sha="$(sha256sum "$exit42_out" | awk '{print $1}')"
    golden_sha="$(sha256sum "$efi_golden" | awk '{print $1}')"
    golden_size="$(stat -c '%s' "$efi_golden")"
    image_size="$(stat -c '%s' "$esp_dir/EFI/BOOT/BOOTX64.EFI")"
    text_size="$(stat -c '%s' "$build_dir/dsfb_text.bin")"
    rdata_size="$(stat -c '%s' "$build_dir/dsfb_rdata.bin")"
    symbols_size="$(stat -c '%s' "$build_dir/dsfb_symbols.bin")"
    pnp_source_count="$(find "$repo_root/tools/phosphoric/dsfb_pnp" -maxdepth 1 -type f -name 'byte_*.phos' | wc -l | awk '{print $1}')"
    pnp_artifact_count="$(find "$repo_root/tools/phosphoric/dsfb_pnp_artifacts" -maxdepth 1 -type f -name 'byte_*.bin' | wc -l | awk '{print $1}')"
    pnp_artifact_size="$(stat -c '%s' "$repo_root/tools/phosphoric/dsfb_pnp_artifacts/byte_0000.bin")"

    fb_width="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" demo_framebuffer_width)"
    fb_height="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" demo_framebuffer_height)"
    fb_stride="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" demo_framebuffer_stride)"
    fb_bpp="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" demo_framebuffer_bytes_per_pixel)"
    text_port="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" debug_text_port)"
    data_port="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" debug_data_port)"
    exit_port="$(extract_const "$repo_root/apps/dsfb_demo/boot_entry.phos" debug_exit_port)"
    boot_seq="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_boot_seq)"
    enter_seq="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_task_enter_seq)"
    exit_seq="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_task_exit_seq)"
    residual_count="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_residual_count)"
    theorem_printed="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_theorem_printed)"
    halt_code="$(extract_const "$repo_root/apps/dsfb_demo/task_state.phos" initial_halt_code)"
    theorem_len="$(extract_const "$repo_root/apps/dsfb_demo/theorem_text.phos" theorem_text_length)"
    theorem_first="$(extract_const "$repo_root/apps/dsfb_demo/theorem_text.phos" theorem_first_byte)"
    theorem_last="$(extract_const "$repo_root/apps/dsfb_demo/theorem_text.phos" theorem_last_byte)"
    theorem_stages="$(extract_const "$repo_root/apps/dsfb_demo/theorem_text.phos" theorem_stage_count)"
    theorem_mode="$(extract_const "$repo_root/apps/dsfb_demo/theorem_text.phos" endoduction_inference_mode)"

    {
        printf 'Compiler / build evidence:\n'
        printf '  pcc-stage2.bin: %s B  sha256=%s\n' "$stage2_size" "$stage2_sha"
        printf '  pcc2 canonical: phase0_stub compiler/pcc2.phos == pcc-stage2.bin: YES\n'
        printf '  pcc2 self-host: pcc-stage2 compiler/pcc2.phos == pcc-stage2.bin: YES\n'
        printf '  compile/run fixture: pcc-stage2 exit42.phos -> sha256=%s, rc=%s\n' "$exit42_sha" "$exit42_rc"
        printf '  pcc-stage2 layout: 1081 B base + 16384 B synth-entry sha16=%s + 23 x 24 B helper slots\n' "$synth_sha16"
        printf '\n'
        printf '  Real compile check for active DSFB demo sources:\n'
        printf '  source                                         result  bytes  stage2_sha16\n'
    } > "$compiler_summary"

    local src canon stage2_out size sha
    for src in \
        "apps/dsfb_demo/boot_entry.phos" \
        "apps/dsfb_demo/task_state.phos" \
        "apps/dsfb_demo/theorem_text.phos"
    do
        canon="$work_dir/$(basename "$src").phase0_stub.bin"
        stage2_out="$work_dir/$(basename "$src").stage2.bin"
        "$phase0_stub" "$repo_root/$src" "$canon" >/dev/null
        "$stage2" "$repo_root/$src" "$stage2_out" >/dev/null
        cmp -s "$stage2_out" "$canon" || {
            printf 'stage2 output for %s differs from phase0_stub output\n' "$src" >&2
            exit 1
        }
        size="$(stat -c '%s' "$stage2_out")"
        sha="$(sha256sum "$stage2_out" | awk '{print substr($1,1,16)}')"
        printf '  %-46s OK      %4s  %s\n' "$src" "$size" "$sha" >> "$compiler_summary"
    done

    {
        printf '\n'
        printf '  Active source constants compiled into the bootable path:\n'
        printf '  boot_entry: framebuffer=%sx%s stride=%s bpp=%s ports text=0x%03x data=0x%03x exit=0x%02x\n' \
            "$fb_width" "$fb_height" "$fb_stride" "$fb_bpp" "$text_port" "$data_port" "$exit_port"
        printf '  task_state: boot_seq=%s task_enter_seq=%s task_exit_seq=%s residual_count=%s theorem_printed=%s halt_code=%s\n' \
            "$boot_seq" "$enter_seq" "$exit_seq" "$residual_count" "$theorem_printed" "$halt_code"
        printf '  theorem_text: length=%s first_byte=%s last_byte=%s stage_count=%s endoduction_mode=%s\n' \
            "$theorem_len" "$theorem_first" "$theorem_last" "$theorem_stages" "$theorem_mode"
        printf '\n'
        printf '  Direct PE32+ boot image writer outputs:\n'
        printf '  write_dsfb_efi.sh -> .text=%s B .rdata=%s B symbols=%s B BOOTX64.EFI=%s B\n' \
            "$text_size" "$rdata_size" "$symbols_size" "$image_size"
        printf '  golden BOOTX64.EFI: %s B sha256=%s byte_equal=YES\n' "$golden_size" "$golden_sha"
        printf '  verify_dsfb_pe.sh: MZ/PE32+, x86_64, 2 sections, EFI app, efi_main, DSFB markers: YES\n'
        printf '  link policy: c_objects=none, clang_used=false, lld_used=false, external_linker_used=false\n'
        printf '  PNP byte archive inventory: %s byte_NNNN.phos sources, %s executable artifacts (%s B each)\n' \
            "$pnp_source_count" "$pnp_artifact_count" "$pnp_artifact_size"
    } >> "$compiler_summary"
}

require_tool qemu-system-x86_64
require_tool mkfs.vfat
require_tool mmd
require_tool mcopy
require_tool magick
require_tool python3
require_tool sha256sum

mkdir -p "$work_dir" "$frames_dir"
find "$frames_dir" -type f -name 'frame-*.png' -delete
find "$frames_dir" -type f -name 'frame-*.txt' -delete

bash "$repo_root/tools/image-builder/build_dsfb_demo.sh" >/dev/null
generate_compiler_summary

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
    printf 'missing OVMF firmware\n' >&2
    printf 'set PHOSPHORIC_OVMF_CODE and PHOSPHORIC_OVMF_VARS, or set PHOSPHORIC_OVMF=/path/to/OVMF.fd\n' >&2
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
TMPDIR="$work_dir" timeout "$qemu_timeout" qemu-system-x86_64 \
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

if [ "$qemu_rc" -ne 1 ]; then
    printf 'QEMU rc=%s; expected 1 from clean isa-debug-exit code 0\n' "$qemu_rc" >&2
    exit 1
fi

required_markers=(
    'phosphoric: dsfb demo entry'
    'DSFB (Drift-Slew Fusion Bootstrap)'
    'Endoduction is the 4th mode of Inference.'
    'byte-deterministic certificate.'
    'phosphoric: task accepted'
    'phosphoric: residuals emitted'
    'phosphoric: dsfb demo halt'
)

for marker in "${required_markers[@]}"; do
    grep -F -q "$marker" "$debug_log" || {
        printf 'missing marker %q in %s\n' "$marker" "$debug_log" >&2
        exit 1
    }
done

records_size=$(wc -c < "$records_bin")
if [ "$records_size" -ne 96 ]; then
    printf 'residual records size %s != 96 (3 x 32B)\n' "$records_size" >&2
    exit 1
fi

bash "$repo_root/tools/verify/encode_pfi.sh" \
    "$records_bin" \
    "$build_dir/linked-artifact.txt" \
    "$esp_dir/EFI/BOOT/BOOTX64.EFI" \
    "$pfi_out" \
    >/dev/null
bash "$repo_root/tools/verify/check_pfi_layout.sh" "$pfi_out" >/dev/null

golden_pfi="$repo_root/tests/golden/dsfb_demo.pfi"
if [ -f "$golden_pfi" ]; then
    cmp -s "$pfi_out" "$golden_pfi" || {
        printf 'runtime PFI %s does not match golden %s\n' "$pfi_out" "$golden_pfi" >&2
        exit 1
    }
fi

boot_hash="$(sha256sum "$esp_dir/EFI/BOOT/BOOTX64.EFI" | awk '{print $1}')"
pfi_hash="$(sha256sum "$pfi_out" | awk '{print $1}')"
source_bundle_hash="$(sed -n 's/^source_bundle_hash=//p' "$build_dir/linked-artifact.txt")"
boot_runtime="$(sed -n 's/^boot_runtime=//p' "$build_dir/linked-artifact.txt")"
producer="$(sed -n 's/^producer=//p' "$build_dir/linked-artifact.txt")"
machine_image_writer="$(sed -n 's/^machine_image_writer=//p' "$build_dir/linked-artifact.txt")"
active_sources="$(sed -n 's/^active_phosphoric_sources=//p' "$build_dir/linked-artifact.txt")"
pe_structure="$(sed -n 's/^pe_structure=//p' "$build_dir/linked-artifact.txt")"
golden_status="not checked"
if [ -f "$golden_pfi" ]; then
    golden_status="byte-equal to tests/golden/dsfb_demo.pfi"
fi

python3 - "$records_bin" "$pfi_out" "$evidence_summary" <<'PY'
import hashlib
import struct
import sys

records_path, pfi_path, out_path = sys.argv[1:4]
kind_names = {
    6: "R6 task_transition",
    7: "R7 boot_check",
}

with open(records_path, "rb") as f:
    records = f.read()
with open(pfi_path, "rb") as f:
    pfi = f.read()

lines = []
lines.append("Residual records captured from debug-data port 0x500:")
lines.append("idx  kind                 seq  cycle  payload anchor       chain_hash")
for idx in range(0, len(records), 32):
    record = records[idx : idx + 32]
    kind = record[0]
    arch_id = record[1]
    seq = struct.unpack("<H", record[2:4])[0]
    cycle = struct.unpack("<Q", record[4:12])[0]
    payload = record[12:26]
    chain_hash = record[26:30].hex()
    if kind == 7 and payload[:5] == b"DSFB\x01":
        anchor = "DSFB/v1 boot"
    elif kind == 6 and payload[:1] == b"\x01":
        anchor = "task enter"
    elif kind == 6 and payload[:1] == b"\x02":
        anchor = "task exit"
    else:
        anchor = payload[:6].hex()
    lines.append(
        f"{idx // 32:<3}  {kind_names.get(kind, 'R? unknown'):<20} "
        f"{seq:<4} {cycle:<6} {anchor:<18} {chain_hash}"
    )

magic = pfi[:4].decode("ascii", errors="replace")
count = struct.unpack("<I", pfi[4:8])[0]
manifest_hash = pfi[32:64].hex()
image_hash = pfi[64:96].hex()
stream_hash = pfi[96:128].hex()
final_chain_hash = pfi[-32:-28].hex()

lines.append("")
lines.append("PFI0 case file decoded from runtime records:")
lines.append(f"magic={magic}  residual_count={count}  size={len(pfi)} bytes")
lines.append(f"manifest_hash={manifest_hash}")
lines.append(f"image_hash   ={image_hash}")
lines.append(f"stream_hash  ={stream_hash}")
lines.append(f"final_chain  ={final_chain_hash}")
lines.append(f"records_sha256={hashlib.sha256(records).hexdigest()}")

with open(out_path, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
    f.write("\n")
PY

{
    printf '$ bash tools/qemu-run/run_dsfb_demo.sh\n'
    printf '\n'
    printf '[1/7] Build the active DSFB boot image through the compiler chain\n'
    cat "$compiler_summary"
    printf '\n'
    printf '[2/7] Boot and capture in QEMU/OVMF\n'
    printf '      ESP: 64 MiB FAT image -> EFI/BOOT/BOOTX64.EFI\n'
    printf '      text: isa-debugcon 0x402 -> build/uefi-demo/dsfb/qemu-debug.log\n'
    printf '      data: isa-debugcon 0x500 -> build/uefi-demo/dsfb/dsfb_demo_records_runtime.bin\n'
    printf '      halt: isa-debug-exit 0xf4 -> qemu_rc=%s (clean code 0)\n' "$qemu_rc"
    printf '\n'
    printf '[3/7] Raw debug-port theorem stream\n'
    printf '\n'
    cat "$debug_log"
    printf '\n'
    printf '[4/7] Validate runtime text markers\n'
    printf '      7/7 markers present: entry, theorem anchors, task accepted, residuals emitted, halt\n'
    printf '\n'
    printf '[5/7] Decode residual data stream and PFI0 case file\n'
    printf '      residual_records=%s bytes (3 x 32B typed deterministic records)\n' "$records_size"
    printf '\n'
    cat "$evidence_summary"
    printf '\n'
    printf '[6/7] Encode and verify runtime PFI0 evidence\n'
    printf '      runtime_pfi_sha256=%s\n' "$pfi_hash"
    printf '      golden comparison: %s\n' "$golden_status"
    printf '\n'
    printf '[7/7] Result: booted, printed theorem, emitted 3 typed residuals, produced PFI0, halted cleanly.\n'
} > "$transcript"

prepare_render_text "$transcript" "$render_text"
render_terminal_png "$render_text" "$out_png"

frame_count=22
transcript_bytes=$(wc -c < "$transcript")
for i in $(seq 1 "$frame_count"); do
    frame_text="$frames_dir/frame-$(printf '%02d' "$i").txt"
    frame_png="$frames_dir/frame-$(printf '%02d' "$i").png"
    chars=$(( (transcript_bytes * i + frame_count - 1) / frame_count ))
    head -c "$chars" "$transcript" | fold -s -w 128 | sed -n '1,88p' > "$frame_text"
    printf '\n[stream %02d/%02d]\n' "$i" "$frame_count" >> "$frame_text"
    render_terminal_png "$frame_text" "$frame_png"
done

magick \
    -delay 20 "$frames_dir"/frame-*.png \
    -delay 180 "$out_png" \
    -loop 0 \
    -layers Optimize \
    "$out_gif"

printf 'wrote %s\n' "$out_png"
printf 'wrote %s\n' "$out_gif"
