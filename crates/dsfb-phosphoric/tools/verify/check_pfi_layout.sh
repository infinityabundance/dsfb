#!/usr/bin/env bash
# check_pfi_layout.sh — PFI0 case-file layout gate (Session 13, Stream C
# Milestone A).
#
# Forensic role: pin the byte-stable layout of `.pfi` evidentiary
# containers per docs/PFI0.md. Verifies:
#
#   1. PFI0 magic at offset 0..3.
#   2. residual_count u32 LE at offset 4..7 matches the actual record
#      count derivable from file size.
#   3. Total file size == 192 + 32 * (residual_count - 1) for
#      residual_count >= 1.
#   4. Reserved byte regions are zero.
#   5. stream_hash at offset 96..127 == SHA-256 of concatenated record
#      bytes.
#   6. Each record's kind is in the closed taxonomy {1..7, 0xFF}.
#   7. Each record's seq is strictly monotonic starting at 1.
#   8. Each record's chain_hash matches `chain_step(prev_chain,
#      event_bytes)` re-derived in awk per kernel/residual.phos.
#   9. final_chain_hash at footer == record[N-1].chain_hash.
#
# Anchors: docs/PFI0.md, kernel/residual.phos.
#
# Iterates every `.pfi` under tools/verify/fixtures/pfi/. Adversarial
# fixtures (Session 16 Milestone D) will be opted-out by directory or
# manifest entry.
#
# Exit: 0 pass, 1 layout drift on any fixture, 2 awk/python/od
# unavailable.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if ! command -v od >/dev/null; then echo "[pfi-layout] od required" >&2; exit 2; fi
if ! command -v awk >/dev/null; then echo "[pfi-layout] awk required" >&2; exit 2; fi
if ! command -v sha256sum >/dev/null; then echo "[pfi-layout] sha256sum required" >&2; exit 2; fi

fixtures_dir="tools/verify/fixtures/pfi"
if [ ! -d "$fixtures_dir" ]; then
    echo "[pfi-layout] no fixtures directory at $fixtures_dir" >&2
    exit 1
fi

fail=0
total=0

note_fail() {
    fail=$((fail + 1))
    echo "[pfi-layout] FAIL: $*" >&2
}

# Read N bytes at offset O from file F as space-separated decimal bytes.
# Uses od -An -tu1 -j O -N N.
read_bytes_dec() {
    local f="$1" o="$2" n="$3"
    od -An -tu1 -j "$o" -N "$n" "$f" | tr -s ' \n' ' ' | sed 's/^ //; s/ $//'
}

read_bytes_hex() {
    local f="$1" o="$2" n="$3"
    od -An -tx1 -j "$o" -N "$n" "$f" | tr -d ' \n'
}

# Verify all bytes in a range are zero.
assert_zero_range() {
    local f="$1" o="$2" n="$3" label="$4"
    local got
    got="$(read_bytes_hex "$f" "$o" "$n")"
    local zeros
    zeros="$(printf '00%.0s' $(seq 1 "$n"))"
    if [ "$got" != "$zeros" ]; then
        note_fail "$f: $label not all-zero at offset $o length $n"
        return 1
    fi
    return 0
}

# Re-derive chain_step in awk: 28-byte event, 4-byte prev → 4-byte hash.
chain_step_awk() {
    local prev_hex="$1" event_hex="$2"
    awk -v prev="$prev_hex" -v event="$event_hex" 'BEGIN {
        p[0]=31; p[1]=131; p[2]=524287; p[3]=16777213;
        # Parse 4-byte prev
        for (i=0; i<4; i++) {
            s[i] = strtonum("0x" substr(prev, i*2 + 1, 2));
        }
        # Parse 28-byte event
        for (k=0; k<28; k++) {
            ev[k] = strtonum("0x" substr(event, k*2 + 1, 2));
        }
        for (k=0; k<28; k++) {
            for (n=0; n<4; n++) s[n] = s[n] + ev[k] * p[n];
        }
        printf "%02x%02x%02x%02x", s[0]%256, s[1]%256, s[2]%256, s[3]%256;
    }'
}

verify_one_pfi() {
    local f="$1"
    total=$((total + 1))

    local size
    size="$(stat -c '%s' "$f")"
    if [ "$size" -lt 192 ]; then
        note_fail "$f: size $size < minimum 192"
        return 1
    fi

    # Magic
    local magic_hex magic_expect
    magic_hex="$(read_bytes_hex "$f" 0 4)"
    magic_expect="50464930"  # "PFI0"
    if [ "$magic_hex" != "$magic_expect" ]; then
        note_fail "$f: magic mismatch (got $magic_hex, expected $magic_expect = 'PFI0')"
        return 1
    fi

    # residual_count u32 LE at offset 4..7
    local b4 b5 b6 b7 count
    set -- $(read_bytes_dec "$f" 4 4)
    b4="$1"; b5="$2"; b6="$3"; b7="$4"
    count=$(( b4 | (b5 << 8) | (b6 << 16) | (b7 << 24) ))

    local expected_size
    if [ "$count" -ge 1 ]; then
        expected_size=$(( 192 + 32 * (count - 1) ))
    else
        expected_size=192
    fi
    if [ "$size" -ne "$expected_size" ]; then
        note_fail "$f: size $size != expected $expected_size (residual_count=$count)"
        return 1
    fi

    # Header reserved (offset 8..32, 24 bytes)
    assert_zero_range "$f" 8 24 "header reserved" || return 1

    # Records start at offset 128
    local records_start=128
    local records_end=$(( records_start + 32 * count ))
    local footer_start=$records_end

    # Stream hash check
    local stream_hash_expected
    if [ "$count" -ge 1 ]; then
        stream_hash_expected="$(dd if="$f" bs=1 skip="$records_start" count=$((32 * count)) status=none | sha256sum | awk '{print $1}')"
    else
        # SHA-256 of empty input
        stream_hash_expected="e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    fi
    local stream_hash_got
    stream_hash_got="$(read_bytes_hex "$f" 96 32)"
    if [ "$stream_hash_got" != "$stream_hash_expected" ]; then
        note_fail "$f: stream_hash mismatch (got $stream_hash_got, expected $stream_hash_expected)"
        return 1
    fi

    # Per-record checks
    local i prev_chain_hex="00000000"
    for ((i = 0; i < count; i++)); do
        local off=$(( records_start + 32 * i ))

        # Kind (offset off+0): closed taxonomy
        local kind
        kind="$(od -An -tu1 -j "$off" -N 1 "$f" | tr -d ' \n')"
        case "$kind" in
            1|2|3|4|5|6|7|255) ;;
            *) note_fail "$f: record[$i] kind=$kind not in closed taxonomy {1..7, 0xFF}"; return 1 ;;
        esac

        # Seq (offset off+2..3): u16 LE; must equal i+1
        local sq_lo sq_hi seq
        set -- $(read_bytes_dec "$f" $((off + 2)) 2)
        sq_lo="$1"; sq_hi="$2"
        seq=$(( sq_lo | (sq_hi << 8) ))
        if [ "$seq" -ne $((i + 1)) ]; then
            note_fail "$f: record[$i] seq=$seq != expected $((i + 1))"
            return 1
        fi

        # Reconstruct event_bytes (28 bytes = record bytes 0..26 + 2 zero pad).
        # Per kernel/residual.phos `record` fn: ev[0..26] = record fields
        # except chain_hash; ev[26..28] = 0 padding.
        local event_hex
        event_hex="$(read_bytes_hex "$f" "$off" 26)0000"

        # Re-derived chain_hash
        local chain_expected chain_got
        chain_expected="$(chain_step_awk "$prev_chain_hex" "$event_hex")"
        chain_got="$(read_bytes_hex "$f" $((off + 26)) 4)"
        if [ "$chain_got" != "$chain_expected" ]; then
            note_fail "$f: record[$i] chain_hash mismatch (got $chain_got, expected $chain_expected)"
            return 1
        fi

        # Per-record padding (offset off+30..32)
        assert_zero_range "$f" $((off + 30)) 2 "record[$i] pad" || return 1

        prev_chain_hex="$chain_got"
    done

    # final_chain_hash at footer
    local final_chain_got final_chain_expected
    final_chain_got="$(read_bytes_hex "$f" "$footer_start" 4)"
    if [ "$count" -ge 1 ]; then
        final_chain_expected="$prev_chain_hex"
    else
        final_chain_expected="00000000"
    fi
    if [ "$final_chain_got" != "$final_chain_expected" ]; then
        note_fail "$f: final_chain_hash mismatch (got $final_chain_got, expected $final_chain_expected)"
        return 1
    fi

    # Footer reserved (28 bytes after final_chain_hash)
    assert_zero_range "$f" $((footer_start + 4)) 28 "footer reserved" || return 1

    printf '  %-50s PASS  size=%s count=%s\n' "$(basename "$f")" "$size" "$count"
    return 0
}

# -------------------------------------------------------------------
# Main loop.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric PFI0 case-file layout gate"
echo "  doctrine: docs/PFI0.md"
echo "============================================================"

shopt -s nullglob
# Adversarial fixtures live under fixtures/pfi/malformed/ and are
# routed to check_malformed_pfi.sh (Stream C Milestone D). Only
# well-formed fixtures at the top level are verified here.
fixtures=("$fixtures_dir"/*.pfi)
shopt -u nullglob

if [ "${#fixtures[@]}" -eq 0 ]; then
    echo "[pfi-layout] FAIL: no .pfi fixtures present in $fixtures_dir" >&2
    exit 1
fi

for f in "${fixtures[@]}"; do
    verify_one_pfi "$f" || true
done

echo "------------------------------------------------------------"
echo "  total .pfi fixtures examined : $total"
echo "  layout violations            : $fail"

if [ "$fail" -eq 0 ]; then
    echo "  [pfi-layout] OK — all .pfi fixtures byte-stable per PFI0.md"
    exit 0
fi

echo "  [pfi-layout] DOCTRINE VIOLATION — see FAIL lines above" >&2
exit 1
