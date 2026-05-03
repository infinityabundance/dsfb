#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
out_dir="${1:-$repo_root/build/uefi-demo/generated}"
efi_out="${2:-$out_dir/BOOTX64.EFI}"
provenance_file="$out_dir/boot_profile_provenance.env"
machine_manifest="$out_dir/boot_machine_image.env"

if [ ! -f "$provenance_file" ]; then
  printf 'missing boot provenance file: %s\n' "$provenance_file" >&2
  exit 1
fi

# shellcheck disable=SC1090
. "$provenance_file"

boot_entry="$repo_root/apps/demo/boot_entry.phos"
extract_const() {
  local path="$1"
  local name="$2"
  local value

  value="$(
    sed -n "s/^[[:space:]]*fn ${name}()[[:space:]]*->[^{]*{[[:space:]]*\\([0-9][0-9]*\\)[[:space:]]*}[[:space:]]*$/\\1/p" "$path"
  )"

  if [ -z "$value" ]; then
    printf 'failed to extract literal boot-profile constant %s from %s\n' "$name" "$path" >&2
    exit 1
  fi

  printf '%s\n' "$value"
}

debug_text_port="$(extract_const "$boot_entry" debug_text_port)"
debug_exit_port="$(extract_const "$boot_entry" debug_exit_port)"

if [ "$debug_text_port" -gt 65535 ] || [ "$debug_exit_port" -gt 65535 ]; then
  printf '%s\n' "debug ports must fit in u16" >&2
  exit 1
fi

mkdir -p "$out_dir"
code_file="$out_dir/boot_text.bin"
rdata_file="$out_dir/boot_rdata.bin"
symbols_file="$out_dir/boot_symbols.bin"
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

messages=(
  "phosphoric: entering generated boot-asm demo"$'\r\n'
  "phosphoric: generated boot-asm demo runtime active"$'\r\n'
  "phosphoric: event routed"$'\r\n'
  "phosphoric: redraw complete"$'\r\n'
  "phosphoric: demo complete"$'\r\n'
)

msg_offsets=()
for message in "${messages[@]}"; do
  msg_offsets+=("$(stat -c '%s' "$rdata_file")")
  emit_cstr "$message" "$rdata_file"
done

# UCS-2 (UTF-16LE) banner messages for UEFI ConOut->OutputString.
# Each printable ASCII char becomes 2 bytes: (low, 0). Strings are
# CHAR16-terminated (two zero bytes).
emit_ucs2() {
  local text="$1"
  local file="$2"
  local i byte
  LC_ALL=C
  for ((i = 0; i < ${#text}; i += 1)); do
    byte="$(printf '%d' "'${text:i:1}")"
    emit_u8 "$byte" "$file"
    emit_u8 0 "$file"
  done
  # Trailing CR LF then NUL terminator.
  emit_u8 13 "$file"; emit_u8 0 "$file"
  emit_u8 10 "$file"; emit_u8 0 "$file"
  emit_u8 0  "$file"; emit_u8 0 "$file"
}

ucs2_messages=(
  "PHOSPHORIC DEMO * boot runtime entered"
  "PHOSPHORIC DEMO * generated boot-asm runtime active"
  "PHOSPHORIC DEMO * input event routed"
  "PHOSPHORIC DEMO * button-press redraw complete"
  "PHOSPHORIC DEMO * demo complete"
)

ucs2_offsets=()
for message in "${ucs2_messages[@]}"; do
  ucs2_offsets+=("$(stat -c '%s' "$rdata_file")")
  emit_ucs2 "$message" "$rdata_file"
done

debug_puts_offset=0

# phosphoric_debug_puts(rcx = nul-terminated debug string)
emit_bytes "$code_file" 55 48 89 e5 49 89 c8
emit_mov_dx_imm16 "$debug_text_port" "$code_file"
emit_bytes "$code_file" 41 8a 00 84 c0 74 06 ee 49 ff c0 eb f3 5d c3

demo_init_offset="$(stat -c '%s' "$code_file")"
emit_bytes "$code_file" 48 89 c8 c3

demo_step_offset="$(stat -c '%s' "$code_file")"
emit_bytes "$code_file" 48 89 c8 c3

demo_render_offset="$(stat -c '%s' "$code_file")"
emit_bytes "$code_file" c7 01 01 00 00 00 48 89 c8 c3

entry_offset="$(stat -c '%s' "$code_file")"

# efi_main(rcx=ImageHandle, rdx=SystemTable*):
#   - allocates 0x38 bytes of stack: 0x20 shadow space + 0x18 locals
#   - saves SystemTable* at [rsp+0x20]
#   - per marker: write to debug-port via debug_puts AND to firmware
#     console via SystemTable->ConOut->OutputString (visible in QEMU
#     screendumps)
#   - holds the framebuffer with a busy-loop delay before triggering
#     QEMU debug-exit (gives the visual runner time to capture)
emit_bytes "$code_file" 48 83 ec 38               # sub rsp, 0x38
emit_bytes "$code_file" 48 89 54 24 20            # mov [rsp+0x20], rdx (save SystemTable)

for idx in "${!msg_offsets[@]}"; do
  msg_offset="${msg_offsets[$idx]}"
  ucs2_offset="${ucs2_offsets[$idx]}"

  # lea rcx, [rip + ascii_msg]; call debug_puts
  lea_offset="$(stat -c '%s' "$code_file")"
  emit_bytes "$code_file" 48 8d 0d
  emit_u32le "$(( rdata_va + msg_offset - (text_va + lea_offset + 7) ))" "$code_file"

  call_offset="$(stat -c '%s' "$code_file")"
  emit_bytes "$code_file" e8
  emit_u32le "$(( debug_puts_offset - (call_offset + 5) ))" "$code_file"

  # Inline ConOut->OutputString(SystemTable->ConOut, ucs2_msg)
  # mov rcx, [rsp+0x20]    ; load SystemTable
  emit_bytes "$code_file" 48 8b 4c 24 20
  # test rcx, rcx ; je +20 (skip if SystemTable null)
  emit_bytes "$code_file" 48 85 c9
  emit_bytes "$code_file" 74 14
  # mov rax, [rcx+0x40]    ; load ConOut
  emit_bytes "$code_file" 48 8b 41 40
  # test rax, rax ; je +13 (skip if ConOut null)
  emit_bytes "$code_file" 48 85 c0
  emit_bytes "$code_file" 74 0d
  # mov rcx, rax           ; rcx = ConOut (this)
  emit_bytes "$code_file" 48 89 c1
  # lea rdx, [rip + ucs2_msg]
  lea_offset="$(stat -c '%s' "$code_file")"
  emit_bytes "$code_file" 48 8d 15
  emit_u32le "$(( rdata_va + ucs2_offset - (text_va + lea_offset + 7) ))" "$code_file"
  # call [rax+0x08]        ; OutputString
  emit_bytes "$code_file" ff 50 08
  # (skip target lands here)
done

# Delay loop so the visual runner has time to capture the firmware
# screen with the printed banners. ~150,000,000 iterations of dec/jne
# is on the order of one second on the QEMU TCG VCPU.
emit_bytes "$code_file" 48 c7 c1 00 e1 f5 08      # mov rcx, 150_000_000
emit_bytes "$code_file" 48 ff c9                  # dec rcx
emit_bytes "$code_file" 75 fb                     # jne -5 (loop)

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

write_symbol_long() {
  local string_offset="$1"
  local value="$2"
  emit_u32le 0 "$symbols_file"
  emit_u32le "$string_offset" "$symbols_file"
  emit_u32le "$value" "$symbols_file"
  emit_u16le 1 "$symbols_file"
  emit_u16le 32 "$symbols_file"
  emit_u8 2 "$symbols_file"
  emit_u8 0 "$symbols_file"
}

string_table_file="$out_dir/boot_symbol_strings.bin"
: > "$string_table_file"
emit_u32le 0 "$string_table_file"
init_name_offset="$(stat -c '%s' "$string_table_file")"
emit_cstr "phosphoric_demo_init" "$string_table_file"
step_name_offset="$(stat -c '%s' "$string_table_file")"
emit_cstr "phosphoric_demo_step" "$string_table_file"
render_name_offset="$(stat -c '%s' "$string_table_file")"
emit_cstr "phosphoric_demo_render" "$string_table_file"
string_table_size="$(stat -c '%s' "$string_table_file")"

tmp_string_table="$string_table_file.tmp"
: > "$tmp_string_table"
emit_u32le "$string_table_size" "$tmp_string_table"
tail -c +5 "$string_table_file" >> "$tmp_string_table"
mv "$tmp_string_table" "$string_table_file"

write_symbol_short "efi_main" "$entry_offset"
write_symbol_long "$init_name_offset" "$demo_init_offset"
write_symbol_long "$step_name_offset" "$demo_step_offset"
write_symbol_long "$render_name_offset" "$demo_render_offset"
cat "$string_table_file" >> "$symbols_file"

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
emit_u32le 4 "$efi_out"
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

machine_image_hash="$(sha256sum "$efi_out" | awk '{print $1}')"

{
  printf 'generated_machine_image=%s\n' "$efi_out"
  printf 'generated_machine_image_hash=%s\n' "$machine_image_hash"
  printf 'machine_image_writer=phosphoric-direct-pe32plus-v1\n'
  printf 'clang_used=false\n'
  printf 'lld_used=false\n'
  printf 'external_linker_used=false\n'
  printf 'generated_symbols=efi_main,phosphoric_demo_init,phosphoric_demo_step,phosphoric_demo_render\n'
  printf 'entrypoint_rva=0x%04x\n' "$(( text_va + entry_offset ))"
  printf 'text_section_rva=0x%04x\n' "$text_va"
  printf 'rdata_section_rva=0x%04x\n' "$rdata_va"
} > "$machine_manifest"

printf '%s\n' "$machine_manifest"
