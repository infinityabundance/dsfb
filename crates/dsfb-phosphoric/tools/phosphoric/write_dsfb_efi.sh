#!/usr/bin/env bash
set -euo pipefail
#
# write_dsfb_efi.sh — build-time PE32+ writer for the v0.3 razor demo.
#
# Pure-shell PE32+ image generator. Runs ONCE at golden-manufacture
# time; the active boot path uses the PNP archive of the resulting
# bytes, not this script. Mirrors the v0.2-era write_boot_efi_from_ir.sh
# pattern — no external assembler, no external linker, no host C
# compiler.
#
# Output: $1 (default build/uefi-demo/dsfb_demo.efi) — a UEFI PE32+
# image that, on boot:
#   1. Writes "phosphoric: dsfb demo entry\r\n" to debug_text_port (0x402)
#   2. Writes the DSFB primary theorem text (692 bytes ASCII) to
#      debug_text_port
#   3. Writes "phosphoric: task accepted\r\n"
#   4. (Session 2) Emits 3 typed residual records to debug_data_port
#      (0x500). Session 1: emits a placeholder marker only.
#   5. Writes "phosphoric: residuals emitted\r\n"
#   6. Writes "phosphoric: dsfb demo halt\r\n"
#   7. Halts via debug_exit_port (0xf4) with code 0.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
out_dir="${1:-$repo_root/build/uefi-demo/dsfb}"
efi_out="${2:-$out_dir/dsfb_demo.efi}"

boot_entry="$repo_root/apps/dsfb_demo/boot_entry.phos"

# Pre-compute the 3 v0.3 residual records using kernel/residual.phos's
# chain_step (primes 31, 131, 524287, 16777213). Output: 96 raw bytes
# (3 × 32B records) emitted at $out_dir/dsfb_records.bin.
records_bin="$out_dir/dsfb_records.bin"
mkdir -p "$out_dir"
python3 - "$records_bin" <<'PY'
import struct, sys

# Three v0.3 razor-demo authority transitions, in source order:
#   record 0: R7 boot_check  — kernel init complete
#   record 1: R6 task_transition — task-enter
#   record 2: R6 task_transition — task-exit
# Closed taxonomy per kernel/residual.phos §1; payload schemas are
# v0.3-specific:
#   R7 boot_check    payload = [0x44 0x53 0x46 0x42 0x01 0x00 ...]  ("DSFB" + v1)
#   R6 task-enter    payload = [0x01, 0, ..., 0]                   (kind=enter)
#   R6 task-exit     payload = [0x02, 0, ..., 0]                   (kind=exit)
records = [
    {"kind": 7, "arch_id": 0, "seq": 1, "cycle": 0,
     "payload": bytes([0x44, 0x53, 0x46, 0x42, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0])},
    {"kind": 6, "arch_id": 0, "seq": 2, "cycle": 1,
     "payload": bytes([0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])},
    {"kind": 6, "arch_id": 0, "seq": 3, "cycle": 2,
     "payload": bytes([0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])},
]

PRIMES = (31, 131, 524287, 16777213)

def chain_step(prev: bytes, event: bytes) -> bytes:
    s = list(prev)
    for b in event:
        for n in range(4):
            s[n] = s[n] + b * PRIMES[n]
    return bytes(x & 0xFF for x in s)

prev = bytes(4)
out = bytearray()
for r in records:
    body = (
        bytes([r["kind"], r["arch_id"]])
        + struct.pack("<H", r["seq"])
        + struct.pack("<Q", r["cycle"])
        + r["payload"]
    )
    assert len(body) == 26
    event = body + bytes(2)
    assert len(event) == 28
    chash = chain_step(prev, event)
    record_bytes = body + chash + bytes(2)
    assert len(record_bytes) == 32
    out += record_bytes
    prev = chash

assert len(out) == 96
with open(sys.argv[1], "wb") as f:
    f.write(out)
PY

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

debug_text_port="$(extract_const "$boot_entry" debug_text_port)"
debug_exit_port="$(extract_const "$boot_entry" debug_exit_port)"
debug_data_port="$(extract_const "$boot_entry" debug_data_port)"

if [ "$debug_text_port" -gt 65535 ] || [ "$debug_exit_port" -gt 65535 ] || [ "$debug_data_port" -gt 65535 ]; then
    printf '%s\n' "debug ports must fit in u16" >&2
    exit 1
fi

mkdir -p "$out_dir"
code_file="$out_dir/dsfb_text.bin"
rdata_file="$out_dir/dsfb_rdata.bin"
symbols_file="$out_dir/dsfb_symbols.bin"
: > "$code_file"
: > "$rdata_file"
: > "$symbols_file"

align_to() {
    local value="$1"
    local alignment="$2"
    printf '%d\n' $(( ((value + alignment - 1) / alignment) * alignment ))
}

emit_u8() {
    local value=$(( $1 & 255 ))
    printf '%b' "\\x$(printf '%02x' "$value")" >> "$2"
}

emit_u16le() {
    local value=$(( $1 & 65535 ))
    emit_u8 "$value" "$2"
    emit_u8 "$(( value >> 8 ))" "$2"
}

emit_u32le() {
    local value=$1
    if [ "$value" -lt 0 ]; then
        value=$(( value + 4294967296 ))
    fi
    emit_u8 "$value" "$2"
    emit_u8 "$(( value >> 8 ))" "$2"
    emit_u8 "$(( value >> 16 ))" "$2"
    emit_u8 "$(( value >> 24 ))" "$2"
}

emit_u64le() {
    local value=$1
    emit_u32le "$(( value & 4294967295 ))" "$2"
    emit_u32le "$(( value >> 32 ))" "$2"
}

emit_zeros() {
    local count="$1"
    local file="$2"
    local i
    for ((i = 0; i < count; i += 1)); do
        emit_u8 0 "$file"
    done
}

emit_ascii() {
    local text="$1"
    local file="$2"
    local i byte
    LC_ALL=C
    for ((i = 0; i < ${#text}; i += 1)); do
        byte="$(printf '%d' "'${text:i:1}")"
        emit_u8 "$byte" "$file"
    done
}

emit_cstr() {
    emit_ascii "$1" "$2"
    emit_u8 0 "$2"
}

emit_bytes() {
    local file="$1"
    shift
    local byte
    for byte in "$@"; do
        emit_u8 "0x$byte" "$file"
    done
}

emit_mov_dx_imm16() {
    local value="$1"
    local file="$2"
    emit_bytes "$file" 66 ba
    emit_u16le "$value" "$file"
}

text_va=4096
rdata_va=8192
file_alignment=512
section_alignment=4096
headers_size=512

# DSFB primary theorem text — verbatim from the papers, ASCII-rendered,
# CRLF line endings. Length pinned by apps/dsfb_demo/theorem_text.phos
# theorem_text_length() == 692.
theorem_lines=(
    "DSFB (Drift-Slew Fusion Bootstrap) creates human-readable structure from usually-discarded residuals. Information from the signatures or fingerprints of noise. Endoduction is the 4th mode of Inference."
    ""
    "DSFB factors operator-legible deterministic residual inference into seven typed deterministic stages,"
    "  (y_hat, y, phi, s) -> r -> (d, sigma) -> E -> g -> tau -> C,"
    "i.e. observation+prediction+phase+source -> residual -> drift+slew -> admissibility envelope -> grammar/motif state -> trust state -> byte-deterministic certificate. Every stage is a total deterministic function on its typed input; no stage admits randomness, learned weights, or non-deterministic parallel reductions."
)

# Boundary markers — what tools/qemu-run/run_uefi_demo.sh greps for.
messages=(
    "phosphoric: dsfb demo entry"$'\r\n'
    "phosphoric: task accepted"$'\r\n'
    "phosphoric: residuals emitted"$'\r\n'
    "phosphoric: dsfb demo halt"$'\r\n'
)

# .rdata layout:
#   - msg_offsets[0..3]: 4 boundary markers (nul-terminated)
#   - theorem_offset: 692 bytes of theorem text (nul-terminated, so 693 raw)
msg_offsets=()
for message in "${messages[@]}"; do
    msg_offsets+=("$(stat -c '%s' "$rdata_file")")
    emit_cstr "$message" "$rdata_file"
done

theorem_offset="$(stat -c '%s' "$rdata_file")"
for ((idx = 0; idx < ${#theorem_lines[@]}; idx += 1)); do
    line="${theorem_lines[$idx]}"
    emit_ascii "$line" "$rdata_file"
    emit_u8 13 "$rdata_file"   # CR
    emit_u8 10 "$rdata_file"   # LF
done
emit_u8 0 "$rdata_file"        # NUL terminator for debug_puts

theorem_size=$(( $(stat -c '%s' "$rdata_file") - theorem_offset - 1 ))

# Append the 3 pre-computed residual records (96 bytes total) to .rdata.
# The .text emit_residuals block reads from this offset and writes 96
# bytes byte-for-byte to debug_data_port.
records_offset="$(stat -c '%s' "$rdata_file")"
cat "$records_bin" >> "$rdata_file"
records_size=96

# .text layout:
#   debug_puts(rcx) : write nul-terminated string at rcx to debug_text_port
#   efi_main       : print marker[0]; print theorem; print marker[1..3]; halt

debug_puts_offset=0

# phosphoric_debug_puts(rcx = nul-terminated debug string)
emit_bytes "$code_file" 55 48 89 e5 49 89 c8
emit_mov_dx_imm16 "$debug_text_port" "$code_file"
emit_bytes "$code_file" 41 8a 00 84 c0 74 06 ee 49 ff c0 eb f3 5d c3

entry_offset="$(stat -c '%s' "$code_file")"

emit_bytes "$code_file" 48 83 ec 28               # sub rsp, 0x28

print_msg_at_rdata_offset() {
    local rdata_off="$1"
    local lea_offset
    lea_offset="$(stat -c '%s' "$code_file")"
    emit_bytes "$code_file" 48 8d 0d
    emit_u32le "$(( rdata_va + rdata_off - (text_va + lea_offset + 7) ))" "$code_file"
    local call_offset
    call_offset="$(stat -c '%s' "$code_file")"
    emit_bytes "$code_file" e8
    emit_u32le "$(( debug_puts_offset - (call_offset + 5) ))" "$code_file"
}

print_msg_at_rdata_offset "${msg_offsets[0]}"   # phosphoric: dsfb demo entry
print_msg_at_rdata_offset "$theorem_offset"     # DSFB theorem
print_msg_at_rdata_offset "${msg_offsets[1]}"   # phosphoric: task accepted

# Emit 96 bytes of pre-computed residual records (3 × 32B) to
# debug_data_port (0x500). One byte per `out dx, al`. Records have
# correct chain_hash chains per kernel/residual.phos chain_step.
emit_residuals_block() {
    # lea rsi, [rip + records_block]   (48 8d 35 <disp32 LE>)
    local lea_offset
    lea_offset="$(stat -c '%s' "$code_file")"
    emit_bytes "$code_file" 48 8d 35
    emit_u32le "$(( rdata_va + records_offset - (text_va + lea_offset + 7) ))" "$code_file"
    # mov ecx, 96                       (b9 60 00 00 00)
    emit_bytes "$code_file" b9 60 00 00 00
    # mov dx, debug_data_port           (66 ba <imm16>)
    emit_mov_dx_imm16 "$debug_data_port" "$code_file"
    # .loop:
    #   lodsb                           (ac)
    #   out dx, al                      (ee)
    #   dec ecx                         (ff c9)
    #   jne .loop  (-6)                 (75 fa)
    emit_bytes "$code_file" ac ee ff c9 75 fa
}
emit_residuals_block

print_msg_at_rdata_offset "${msg_offsets[2]}"   # phosphoric: residuals emitted
print_msg_at_rdata_offset "${msg_offsets[3]}"   # phosphoric: dsfb demo halt

# Halt via debug_exit_port: out dx=port, eax=0; halt loop.
emit_mov_dx_imm16 "$debug_exit_port" "$code_file"
emit_bytes "$code_file" 31 c0 ef f4 eb fd

text_size="$(stat -c '%s' "$code_file")"
rdata_size="$(stat -c '%s' "$rdata_file")"
text_raw_size="$(align_to "$text_size" "$file_alignment")"
rdata_raw_size="$(align_to "$rdata_size" "$file_alignment")"
rdata_raw_ptr=$(( headers_size + text_raw_size ))
symbol_table_ptr=$(( rdata_raw_ptr + rdata_raw_size ))
size_of_image=$(( rdata_va + section_alignment ))

write_symbol_short() {
    local name="$1"
    local value="$2"
    local padded="$name"
    while [ "${#padded}" -lt 8 ]; do
        padded="${padded}"$'\0'
    done
    emit_ascii "${padded:0:8}" "$symbols_file"
    emit_u32le "$value" "$symbols_file"
    emit_u16le 1 "$symbols_file"
    emit_u16le 32 "$symbols_file"
    emit_u8 2 "$symbols_file"
    emit_u8 0 "$symbols_file"
}

write_symbol_short "efi_main" "$entry_offset"

: > "$efi_out"

# DOS header.
emit_ascii "MZ" "$efi_out"
emit_zeros 58 "$efi_out"
emit_u32le 128 "$efi_out"
emit_zeros 64 "$efi_out"

# PE signature and COFF header.
emit_ascii "PE" "$efi_out"
emit_u8 0 "$efi_out"
emit_u8 0 "$efi_out"
emit_u16le 34404 "$efi_out"
emit_u16le 2 "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le "$symbol_table_ptr" "$efi_out"
emit_u32le 1 "$efi_out"
emit_u16le 240 "$efi_out"
emit_u16le 546 "$efi_out"

# PE32+ optional header.
emit_u16le 523 "$efi_out"
emit_u8 0 "$efi_out"
emit_u8 1 "$efi_out"
emit_u32le "$text_raw_size" "$efi_out"
emit_u32le "$rdata_raw_size" "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le "$(( text_va + entry_offset ))" "$efi_out"
emit_u32le "$text_va" "$efi_out"
emit_u64le 4194304 "$efi_out"
emit_u32le "$section_alignment" "$efi_out"
emit_u32le "$file_alignment" "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 5 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le "$size_of_image" "$efi_out"
emit_u32le "$headers_size" "$efi_out"
emit_u32le 0 "$efi_out"
emit_u16le 10 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u64le 1048576 "$efi_out"
emit_u64le 4096 "$efi_out"
emit_u64le 1048576 "$efi_out"
emit_u64le 4096 "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le 16 "$efi_out"
emit_zeros 128 "$efi_out"

# Section table.
emit_ascii ".text" "$efi_out"
emit_zeros 3 "$efi_out"
emit_u32le "$text_size" "$efi_out"
emit_u32le "$text_va" "$efi_out"
emit_u32le "$text_raw_size" "$efi_out"
emit_u32le "$headers_size" "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u32le 1610612768 "$efi_out"

emit_ascii ".rdata" "$efi_out"
emit_zeros 2 "$efi_out"
emit_u32le "$rdata_size" "$efi_out"
emit_u32le "$rdata_va" "$efi_out"
emit_u32le "$rdata_raw_size" "$efi_out"
emit_u32le "$rdata_raw_ptr" "$efi_out"
emit_u32le 0 "$efi_out"
emit_u32le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u16le 0 "$efi_out"
emit_u32le 1073741888 "$efi_out"

current_size="$(stat -c '%s' "$efi_out")"
emit_zeros "$(( headers_size - current_size ))" "$efi_out"
cat "$code_file" >> "$efi_out"
emit_zeros "$(( text_raw_size - text_size ))" "$efi_out"
cat "$rdata_file" >> "$efi_out"
emit_zeros "$(( rdata_raw_size - rdata_size ))" "$efi_out"
cat "$symbols_file" >> "$efi_out"
emit_u32le 4 "$efi_out"

machine_image_hash="$(sha256sum "$efi_out" | awk '{print $1}')"
machine_image_size="$(stat -c '%s' "$efi_out")"

printf '%s\n' "wrote $efi_out"
printf '  size : %s bytes\n' "$machine_image_size"
printf '  sha  : %s\n' "$machine_image_hash"
printf '  text : code=%s raw=%s\n' "$text_size" "$text_raw_size"
printf '  rdata: data=%s raw=%s theorem=%s records=%s\n' "$rdata_size" "$rdata_raw_size" "$theorem_size" "$records_size"
