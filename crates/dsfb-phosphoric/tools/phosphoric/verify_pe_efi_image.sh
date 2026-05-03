#!/usr/bin/env bash
set -euo pipefail

image="${1:?usage: verify_pe_efi_image.sh BOOTX64.EFI}"

if [ ! -f "$image" ]; then
  printf 'missing EFI image: %s\n' "$image" >&2
  exit 1
fi

read_u8() {
  od -An -j "$1" -N 1 -tu1 "$image" | awk '{print $1}'
}

read_u16le() {
  local b0 b1
  b0="$(read_u8 "$1")"
  b1="$(read_u8 "$(( $1 + 1 ))")"
  printf '%d\n' $(( b0 | (b1 << 8) ))
}

read_u32le() {
  local b0 b1 b2 b3
  b0="$(read_u8 "$1")"
  b1="$(read_u8 "$(( $1 + 1 ))")"
  b2="$(read_u8 "$(( $1 + 2 ))")"
  b3="$(read_u8 "$(( $1 + 3 ))")"
  printf '%d\n' $(( b0 | (b1 << 8) | (b2 << 16) | (b3 << 24) ))
}

read_hex() {
  od -An -j "$1" -N "$2" -tx1 -v "$image" | tr -d ' \n'
}

fail() {
  printf 'invalid PE/EFI image %s: %s\n' "$image" "$1" >&2
  exit 1
}

[ "$(read_hex 0 2)" = "4d5a" ] || fail "missing DOS MZ header"
pe_offset="$(read_u32le 60)"
[ "$pe_offset" -eq 128 ] || fail "unexpected PE header offset"
[ "$(read_hex "$pe_offset" 4)" = "50450000" ] || fail "missing PE signature"

coff_offset=$(( pe_offset + 4 ))
machine="$(read_u16le "$coff_offset")"
sections="$(read_u16le "$(( coff_offset + 2 ))")"
symbol_ptr="$(read_u32le "$(( coff_offset + 8 ))")"
symbol_count="$(read_u32le "$(( coff_offset + 12 ))")"
optional_size="$(read_u16le "$(( coff_offset + 16 ))")"

[ "$machine" -eq 34404 ] || fail "machine is not x86_64"
[ "$sections" -eq 2 ] || fail "expected exactly two sections"
[ "$symbol_count" -eq 4 ] || fail "expected four COFF symbols"
[ "$optional_size" -eq 240 ] || fail "unexpected optional-header size"

optional_offset=$(( coff_offset + 20 ))
[ "$(read_u16le "$optional_offset")" -eq 523 ] || fail "not PE32+"

entry_rva="$(read_u32le "$(( optional_offset + 16 ))")"
section_alignment="$(read_u32le "$(( optional_offset + 32 ))")"
file_alignment="$(read_u32le "$(( optional_offset + 36 ))")"
size_of_image="$(read_u32le "$(( optional_offset + 56 ))")"
size_of_headers="$(read_u32le "$(( optional_offset + 60 ))")"
subsystem="$(read_u16le "$(( optional_offset + 68 ))")"
directory_count="$(read_u32le "$(( optional_offset + 108 ))")"

[ "$section_alignment" -eq 4096 ] || fail "unexpected section alignment"
[ "$file_alignment" -eq 512 ] || fail "unexpected file alignment"
[ "$size_of_headers" -eq 512 ] || fail "unexpected header size"
[ "$size_of_image" -eq 12288 ] || fail "unexpected image size"
[ "$subsystem" -eq 10 ] || fail "subsystem is not EFI application"
[ "$directory_count" -eq 16 ] || fail "unexpected data-directory count"
[ "$entry_rva" -ge 4096 ] && [ "$entry_rva" -lt 8192 ] || fail "entrypoint outside .text"

section_offset=$(( optional_offset + optional_size ))
[ "$(read_hex "$section_offset" 5)" = "2e74657874" ] || fail "missing .text section"
[ "$(read_hex "$(( section_offset + 40 ))" 6)" = "2e7264617461" ] || fail "missing .rdata section"

text_va="$(read_u32le "$(( section_offset + 12 ))")"
text_raw_size="$(read_u32le "$(( section_offset + 16 ))")"
text_raw_ptr="$(read_u32le "$(( section_offset + 20 ))")"
rdata_va="$(read_u32le "$(( section_offset + 52 ))")"
rdata_raw_size="$(read_u32le "$(( section_offset + 56 ))")"
rdata_raw_ptr="$(read_u32le "$(( section_offset + 60 ))")"

[ "$text_va" -eq 4096 ] || fail "unexpected .text RVA"
[ "$rdata_va" -eq 8192 ] || fail "unexpected .rdata RVA"
[ "$text_raw_ptr" -eq 512 ] || fail "unexpected .text file offset"
[ "$rdata_raw_ptr" -eq $(( text_raw_ptr + text_raw_size )) ] || fail "unexpected .rdata file offset"
[ "$symbol_ptr" -eq $(( rdata_raw_ptr + rdata_raw_size )) ] || fail "symbol table is not after sections"

image_size="$(stat -c '%s' "$image")"
[ "$image_size" -gt "$symbol_ptr" ] || fail "symbol table outside file"
[ "$image_size" -ge "$(( symbol_ptr + symbol_count * 18 + 4 ))" ] || fail "truncated symbol table"

if [ "$(read_hex "$symbol_ptr" 8)" != "6566695f6d61696e" ]; then
  fail "missing efi_main symbol"
fi
for symbol in phosphoric_demo_init phosphoric_demo_step phosphoric_demo_render; do
  if ! grep -a -F -q "$symbol" "$image"; then
    fail "missing symbol: $symbol"
  fi
done

for marker in \
  'phosphoric: entering generated boot-asm demo' \
  'phosphoric: generated boot-asm demo runtime active' \
  'phosphoric: event routed' \
  'phosphoric: redraw complete' \
  'phosphoric: demo complete'
do
  if ! grep -a -F -q "$marker" "$image"; then
    fail "missing trace marker: $marker"
  fi
done

printf '%s\n' "== phosphoric: direct PE/EFI structure verification complete =="
