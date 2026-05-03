#!/usr/bin/env bash
# verdict_from_pfi.sh — host reference verdict tool for Court
# Requirement D1.
#
# Forensic role: given a 192-byte PFI0 case file containing one R5
# mmio_touch residual, derive the canonical 6-line verdict bytes
# deterministically and write them to stdout. Closes the host
# reference loop alongside emit_r5_record.sh and emit_mmio_boundary_pfi.sh:
#
#   input vector
#     → R5 32-byte record   (emit_r5_record.sh)
#     → PFI0 192-byte case  (emit_mmio_boundary_pfi.sh)
#     → 6-line verdict      (this tool)
#
# This is a host reference verdict path — NOT a Phosphoric-compiled
# classifier. It is bash + awk + od. A subsequent court-side
# requirement may replace it with a Phosphoric-compiled binary that
# emits the same bytes; until then the framing is "host reference
# verdict path produced", not "Phosphoric runtime classifier
# executed" / "court runtime adjudicates" / "compiled classifier
# emits verdict".
#
# Scope (D1): R5 mmio_touch only. Any other residual kind, bad
# magic, or non-192-byte size is a hard error — this tool is not a
# general classifier. The existing tools/verify/check_malformed_pfi.sh
# gate continues to handle adversarial / malformed evidence.
#
# Output (stdout): exactly 6 lines per FORENSIC_PRIMACY.md §3:
#   CLASS=<DriftClass member>
#   RESIDUAL=R<1..7>
#   SEQ=<u32>
#   EXPECTED=<value>
#   ACTUAL=<value>
#   EXIT=<0,2,3,4,5,6>
#
# For the canonical R5 vector (declared_lo=0x1000, declared_hi=0x10FF,
# observed=0x1100), output is byte-identical to
# tools/verify/fixtures/verdicts/mmio_boundary_violation.expect.
#
# Doctrine: docs/PFI0.md (PFI0 layout), kernel/residual.phos (R5
# struct + payload encoding), docs/FORENSIC_PRIMACY.md §3 (canonical
# verdict format + DriftClass enum + EXIT mapping).
#
# Exit: 0 ok, 2 usage / IO / unsupported-kind / bad-size / bad-magic.

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <pfi_path>" >&2
    exit 2
fi

pfi="$1"

if [ ! -r "$pfi" ]; then
    echo "$0: cannot read $pfi" >&2
    exit 2
fi

if ! command -v od >/dev/null; then
    echo "$0: od required" >&2
    exit 2
fi

size="$(wc -c < "$pfi")"
if [ "$size" != "192" ]; then
    echo "$0: PFI0 size $size != 192 (D1 scope: 1-record canonical case)" >&2
    exit 2
fi

# Magic check (offset 0..4 == "PFI0").
magic="$(od -An -c -N 4 "$pfi" | tr -d ' \n')"
if [ "$magic" != "PFI0" ]; then
    echo "$0: bad magic '$magic' (expected PFI0)" >&2
    exit 2
fi

# record[0] starts at offset 128 (= 0x80) per docs/PFI0.md / emit_mmio_boundary_pfi.sh.
# Within the 32-byte record, byte layout per kernel/residual.phos and
# tools/court/emit_r5_record.sh:
#   off 128       : kind        (u8)
#   off 129       : arch_id     (u8)
#   off 130..131  : seq         (LE u16)
#   off 132..139  : cycle       (LE u64)              — unused for mmio_touch
#   off 140..141  : declared_lo (LE u16)              — payload[0..2]
#   off 142..143  : declared_hi (LE u16)              — payload[2..4]
#   off 144..147  : observed    (LE u32)              — payload[4..8]
#   off 148..153  : reserved    (6 bytes, payload[8..14])
#   off 154..157  : chain_hash  (4 bytes)
#   off 158..159  : pad         (2 bytes)

read_u8() {
    od -An -tu1 -j "$1" -N 1 "$pfi" | tr -d ' \n'
}
read_u16_le() {
    local lo hi
    lo="$(od -An -tu1 -j "$1"          -N 1 "$pfi" | tr -d ' \n')"
    hi="$(od -An -tu1 -j "$(( $1 + 1 ))" -N 1 "$pfi" | tr -d ' \n')"
    echo $(( lo | (hi << 8) ))
}
read_u32_le() {
    local b0 b1 b2 b3
    b0="$(od -An -tu1 -j "$1"          -N 1 "$pfi" | tr -d ' \n')"
    b1="$(od -An -tu1 -j "$(( $1 + 1 ))" -N 1 "$pfi" | tr -d ' \n')"
    b2="$(od -An -tu1 -j "$(( $1 + 2 ))" -N 1 "$pfi" | tr -d ' \n')"
    b3="$(od -An -tu1 -j "$(( $1 + 3 ))" -N 1 "$pfi" | tr -d ' \n')"
    echo $(( b0 | (b1 << 8) | (b2 << 16) | (b3 << 24) ))
}

kind="$(read_u8 128)"
seq="$(read_u16_le 130)"

if [ "$kind" != "5" ]; then
    echo "$0: D1 scope is R5 only; record[0].kind=$kind not supported" >&2
    exit 2
fi

declared_lo="$(read_u16_le 140)"
declared_hi="$(read_u16_le 142)"
observed="$(read_u32_le 144)"

# Canonical 6-line verdict per docs/FORENSIC_PRIMACY.md §3.
# CLASS / RESIDUAL / EXIT for R5 mmio_touch are fixed by doctrine:
#   CLASS = MMIO_BOUNDARY_PRESSURE  (closed DriftClass enum)
#   RESIDUAL = R5
#   EXIT = 6                        (canonical exit-code mapping)
# SEQ / EXPECTED / ACTUAL are derived from the record bytes.
printf 'CLASS=MMIO_BOUNDARY_PRESSURE\n'
printf 'RESIDUAL=R5\n'
printf 'SEQ=%d\n' "$seq"
printf 'EXPECTED=mmio_range[0x%X..0x%X]\n' "$declared_lo" "$declared_hi"
printf 'ACTUAL=0x%X\n' "$observed"
printf 'EXIT=6\n'
