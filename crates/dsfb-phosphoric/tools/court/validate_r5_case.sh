#!/usr/bin/env bash
# validate_r5_case.sh — host reference R5 case-validity validator for
# Court Requirement B1 (semantic-payload narrow).
#
# Forensic role: enforce the missing semantic invariant for an R5
# mmio_touch case — that the recorded payload encodes a real MMIO
# boundary violation, i.e. the observed address lies *outside* the
# declared MMIO range. This complements (it does NOT duplicate):
#
#   - tools/verify/check_pfi_layout.sh        (PFI0 magic, residual_count,
#                                              stream_hash, final_chain_hash,
#                                              footer reserved zeros)
#   - tools/verify/check_residual_r5_byte_layout.sh
#                                             (R5 record byte layout +
#                                              chain_step mixer math against
#                                              the canonical test vector)
#   - tools/verify/check_malformed_pfi.sh     (adversarial / malformed PFI
#                                              rejection)
#   - tools/court/verdict_from_pfi.sh         (D1 host reference verdict
#                                              path)
#
# Scope (B1-narrow): R5 mmio_touch only, single-record canonical PFI0
# case. Parses only the four fields needed for the semantic check —
# kind, declared_lo, declared_hi, observed — and enforces:
#
#   kind == 5
#   observed < declared_lo OR observed > declared_hi
#
# Does NOT validate magic, residual_count, stream_hash,
# final_chain_hash, layout — those are owned by check_pfi_layout.sh.
# Does NOT handle malformed PFIs — that is owned by
# check_malformed_pfi.sh. Does NOT classify or render a verdict —
# that is owned by verdict_from_pfi.sh (D1).
#
# This is a host reference R5 case-validity validator. NOT a general
# PFI validator, NOT a general R5 classifier, NOT runtime enforcement,
# NOT Phosphoric-compiled validation. The framing is "host reference
# court validates R5 payload semantics", not "Phosphoric runtime
# court validates" / "compiled court reads its own evidence".
#
# Doctrine: docs/PFI0.md (PFI0 layout — for record[0] offset),
# kernel/residual.phos (R5 struct + payload encoding),
# docs/FORENSIC_PRIMACY.md §1 (R5 mmio_touch boundary semantics).
#
# Exit:
#   0  payload encodes an actual MMIO boundary violation
#   1  semantic violation (kind != 5, OR observed inside declared range)
#   2  usage / IO error

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
if [ "$size" -lt 160 ]; then
    echo "$0: PFI0 too small ($size B); need at least 160 B to read record[0] payload" >&2
    exit 2
fi

# record[0] starts at offset 128 (= 0x80) per docs/PFI0.md. Within
# the 32-byte record, byte layout per kernel/residual.phos:
#   off 128       : kind        (u8)
#   off 140..141  : declared_lo (LE u16)   — payload[0..2]
#   off 142..143  : declared_hi (LE u16)   — payload[2..4]
#   off 144..147  : observed    (LE u32)   — payload[4..8]

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
if [ "$kind" != "5" ]; then
    echo "$0: kind=$kind, expected 5 (R5 mmio_touch); B1 scope is R5-only" >&2
    exit 1
fi

declared_lo="$(read_u16_le 140)"
declared_hi="$(read_u16_le 142)"
observed="$(read_u32_le 144)"

if [ "$declared_lo" -gt "$declared_hi" ]; then
    printf '%s: malformed range: declared_lo=0x%X > declared_hi=0x%X\n' \
        "$0" "$declared_lo" "$declared_hi" >&2
    exit 1
fi

# The semantic invariant: observed must lie OUTSIDE [declared_lo, declared_hi].
# An R5 mmio_touch with observed inside the declared range is not a
# boundary violation — it is an in-range touch mislabeled R5, which is
# semantically invalid evidence even if the layout passes.
if [ "$observed" -ge "$declared_lo" ] && [ "$observed" -le "$declared_hi" ]; then
    printf '%s: observed=0x%X lies INSIDE declared range [0x%X..0x%X] — payload does not encode a boundary violation\n' \
        "$0" "$observed" "$declared_lo" "$declared_hi" >&2
    exit 1
fi

# Echo the parsed semantic fields for the gate to display; the
# validator's correctness contract is its exit code, not this output.
printf 'kind=R5 declared_range=[0x%X..0x%X] observed=0x%X (outside)\n' \
    "$declared_lo" "$declared_hi" "$observed"
exit 0
