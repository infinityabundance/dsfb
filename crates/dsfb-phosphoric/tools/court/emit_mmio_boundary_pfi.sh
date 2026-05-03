#!/usr/bin/env bash
# emit_mmio_boundary_pfi.sh — host reference emitter for the canonical
# Court Requirement A1/B1 PFI0 case file.
#
# Forensic role: converts the hand-encoded mmio_boundary_violation
# anchor into a deterministic reference-emitted output. Given the
# canonical declared-range/observed-touch vector, this emitter
# produces 192 bytes byte-identical to
# tools/verify/fixtures/pfi/mmio_boundary_violation.pfi.
#
# This is a host-side reference emitter — NOT a Phosphoric-compiled
# binary. Its purpose is to prove that the byte stability already
# pinned at the spec level (R5 record layout, chain_step mixer, PFI0
# layout, stream_hash via SHA-256) is reproducible from the inputs
# alone.
#
# Output: exactly 192 bytes on stdout.
#
# Doctrine: docs/PFI0.md (PFI0 layout), kernel/residual.phos (R5
# struct + chain_step mixer), docs/FORENSIC_PRIMACY.md.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
emit_r5="$repo_root/tools/court/emit_r5_record.sh"

if [ ! -x "$emit_r5" ]; then
    echo "$0: missing or non-executable: $emit_r5" >&2
    exit 2
fi
if ! command -v sha256sum >/dev/null; then
    echo "$0: sha256sum required" >&2
    exit 2
fi
if ! command -v xxd >/dev/null; then
    echo "$0: xxd required" >&2
    exit 2
fi

# Anchor inputs (the canonical R5 test vector pinned by
# tools/verify/check_residual_r5_byte_layout.sh).
declared_lo=$((0x1000))
declared_hi=$((0x10FF))
observed=$((0x1100))

# 1. R5 record (32 bytes, hex).
record_hex="$(bash "$emit_r5" "$declared_lo" "$declared_hi" "$observed")"
if [ "${#record_hex}" -ne 64 ]; then
    echo "$0: emit_r5 produced ${#record_hex} hex chars, expected 64" >&2
    exit 2
fi

# 2. stream_hash = SHA-256(record_bytes), 32 bytes (hex).
record_tmp="$(mktemp)"
trap 'rm -f "$record_tmp"' EXIT
printf '%s' "$record_hex" | xxd -r -p > "$record_tmp"
stream_hash_hex="$(sha256sum "$record_tmp" | awk '{print $1}')"
if [ "${#stream_hash_hex}" -ne 64 ]; then
    echo "$0: sha256sum produced ${#stream_hash_hex} hex chars, expected 64" >&2
    exit 2
fi

# 3. final_chain_hash = chain_hash from the (single) record =
# bytes 26..29 of the record = hex chars 52..59.
final_chain_hex="${record_hex:52:8}"

# 4. Build the 192-byte PFI0 hex stream:
#   0x00..0x04   magic "PFI0"
#   0x04..0x08   residual_count = 1 (LE u32)
#   0x08..0x20   24 reserved bytes (zero)
#   0x20..0x40   manifest_hash (32 bytes; sentinel 0xAA per anchor)
#   0x40..0x60   image_hash    (32 bytes; sentinel 0xBB per anchor)
#   0x60..0x80   stream_hash   (32 bytes; SHA-256 of record_bytes)
#   0x80..0xA0   record[0]     (32 bytes)
#   0xA0..0xA4   final_chain_hash (4 bytes)
#   0xA4..0xC0   28 reserved bytes (zero)
hex=""
hex+="50464930"                                           # magic
hex+="01000000"                                           # residual_count = 1 LE u32
hex+="$(printf '00%.0s' $(seq 1 24))"                     # 24 reserved
hex+="$(printf 'aa%.0s' $(seq 1 32))"                     # manifest_hash sentinel
hex+="$(printf 'bb%.0s' $(seq 1 32))"                     # image_hash sentinel
hex+="$stream_hash_hex"                                   # stream_hash
hex+="$record_hex"                                        # record[0]
hex+="$final_chain_hex"                                   # final_chain_hash
hex+="$(printf '00%.0s' $(seq 1 28))"                     # 28 reserved

if [ "${#hex}" -ne 384 ]; then
    echo "$0: hex stream is ${#hex} chars, expected 384" >&2
    exit 2
fi

printf '%s' "$hex" | xxd -r -p
