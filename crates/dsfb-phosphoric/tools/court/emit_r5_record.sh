#!/usr/bin/env bash
# emit_r5_record.sh — host reference emitter for one R5 mmio_touch
# residual record. Produces the canonical 32-byte byte-stable record
# from a declared-range/observed-address vector, mirroring
# kernel/residual.phos's chain_step mixer (primes 31, 131, 524287,
# 16777213) on the 28-byte event composed per the R5 payload encoding
# documented in tools/verify/check_residual_r5_byte_layout.sh.
#
# This is a host-side reference emitter — it is NOT a Phosphoric-
# compiled binary. Its purpose is to convert the hand-encoded R5 anchor
# into a deterministic reference-emitted record so the spec's byte
# stability is reproducible from inputs.
#
# Inputs (decimal integers):
#   $1 = declared_lo (u16)
#   $2 = declared_hi (u16)
#   $3 = observed    (u32)
#
# Output: 64 hex characters on stdout = 32 bytes of R5 record.
#
# Exit: 0 ok, 2 usage/dependency error.

set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "usage: $0 declared_lo declared_hi observed (decimal integers)" >&2
    exit 2
fi

declared_lo=$1
declared_hi=$2
observed=$3

if ! command -v awk >/dev/null; then
    echo "$0: awk required" >&2
    exit 2
fi

# Compute the 32 record bytes; print as 64 lowercase hex chars on one line.
awk -v dl="$declared_lo" -v dh="$declared_hi" -v ob="$observed" '
BEGIN {
    # Record bytes (32 total), 0-indexed. See kernel/residual.phos
    # struct Residual {kind, arch_id, seq, cycle, payload[14], chain_hash[4], pad[2]}.
    rec[0] = 5            # kind = 5 (mmio_touch)
    rec[1] = 0            # arch_id = 0
    rec[2] = 1            # seq lo
    rec[3] = 0            # seq hi  (seq = 1, LE u16)
    for (k = 4; k <= 11; k++) rec[k] = 0   # cycle = 0 (LE u64)

    # payload[0..2] = declared_lo LE u16 (record bytes 12..13)
    rec[12] = dl % 256
    rec[13] = int(dl / 256) % 256
    # payload[2..4] = declared_hi LE u16 (record bytes 14..15)
    rec[14] = dh % 256
    rec[15] = int(dh / 256) % 256
    # payload[4..8] = observed LE u32 (record bytes 16..19)
    rec[16] = ob % 256
    rec[17] = int(ob / 256) % 256
    rec[18] = int(ob / 65536) % 256
    rec[19] = int(ob / 16777216) % 256
    # payload[8..14] = reserved 6 bytes (record bytes 20..25)
    for (k = 20; k <= 25; k++) rec[k] = 0

    # 28-byte event for chain_step input:
    #   ev[0..26]  = record bytes 0..25  (kind+arch_id+seq+cycle+payload, 26 bytes)
    #   ev[26..28] = 2 zero pad bytes
    # (matches the awk vector in tools/verify/check_residual_r5_byte_layout.sh.)
    for (k = 0; k <= 25; k++) ev[k] = rec[k]
    ev[26] = 0
    ev[27] = 0

    # chain_step mixer with prev = [0;4]: primes from kernel/residual.phos.
    p[0] = 31; p[1] = 131; p[2] = 524287; p[3] = 16777213
    s[0] = 0; s[1] = 0; s[2] = 0; s[3] = 0
    for (k = 0; k < 28; k++) {
        for (n = 0; n < 4; n++) {
            s[n] = s[n] + ev[k] * p[n]
        }
    }
    # chain_hash[0..4] = (s[n] & 255) for n in 0..4 (record bytes 26..29)
    rec[26] = s[0] % 256
    rec[27] = s[1] % 256
    rec[28] = s[2] % 256
    rec[29] = s[3] % 256
    # 2-byte alignment pad (record bytes 30..31)
    rec[30] = 0
    rec[31] = 0

    # Emit 32 bytes as 64 hex chars on stdout.
    out = ""
    for (k = 0; k < 32; k++) {
        out = out sprintf("%02x", rec[k])
    }
    print out
}'
