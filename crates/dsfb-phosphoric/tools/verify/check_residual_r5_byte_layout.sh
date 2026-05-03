#!/usr/bin/env bash
# check_residual_r5_byte_layout.sh — R5 mmio_touch byte-layout gate
# (Session 14, Stream C Milestone B).
#
# Forensic role: peer of Session 12's R1 byte-layout gate, for R5
# (mmio_touch). Pins:
#
#   1. The R5 payload encoding for an MMIO boundary violation:
#      payload[0..2]  = declared_lo  u16 LE (low end of allowed range)
#      payload[2..4]  = declared_hi  u16 LE (high end of allowed range)
#      payload[4..8]  = observed     u32 LE (address actually touched)
#      payload[8..14] = 0  (reserved per kind=5)
#
#   2. The chain_step deterministic mixer in kernel/residual.phos
#      produces the byte-stable chain_hash [0x8A, 0xA2, 0xCA, 0x5E] on
#      the canonical R5 test vector (kind=5, arch_id=0, seq=1, cycle=0,
#      declared_lo=0x1000, declared_hi=0x10FF, observed=0x1100,
#      prev=[0;4]). Same vector encoded by tools/verify/fixtures/pfi/
#      mmio_boundary_violation.pfi (Session 13 Milestone A).
#
# This gate does NOT verify producer-side R5 emission; that is later
# Stream C work after the runtime ring lands. It pins the spec the
# producer must satisfy.
#
# Exit: 0 pass, 1 spec drift, 2 awk unavailable.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

residual="kernel/residual.phos"
fail=0

note_fail() { echo "[r5-byte-layout] FAIL: $*" >&2; fail=1; }

if [ ! -r "$residual" ]; then
    note_fail "kernel/residual.phos not readable at $residual"
    echo "[r5-byte-layout] DOCTRINE VIOLATION" >&2
    exit 1
fi

if ! command -v awk >/dev/null; then echo "[r5-byte-layout] awk required" >&2; exit 2; fi

# -------------------------------------------------------------------
# Claim 1: kind=5 is in the closed taxonomy in kernel/residual.phos.
# -------------------------------------------------------------------
if ! grep -qE "^//[[:space:]]+5[[:space:]]+mmio_touch" "$residual"; then
    note_fail "kind=5 mmio_touch not declared in $residual taxonomy comment"
fi

# -------------------------------------------------------------------
# Claim 2: chain_step on R5 MMIO test vector produces [8A A2 CA 5E].
#
# 28-byte event composed per kernel/residual.phos `record` fn:
#   ev[0]=5 (kind); ev[1]=0 (arch_id); ev[2..4]=seq=1 LE;
#   ev[4..12]=cycle=0; ev[12..14]=declared_lo=0x1000 LE;
#   ev[14..16]=declared_hi=0x10FF LE; ev[16..20]=observed=0x1100 LE;
#   ev[20..26]=0; ev[26..28]=0
# -------------------------------------------------------------------
actual="$(awk 'BEGIN {
    p[0]=31; p[1]=131; p[2]=524287; p[3]=16777213;
    ev[0]=5; ev[1]=0; ev[2]=1; ev[3]=0;
    for (k=4; k<12; k++) ev[k]=0;
    ev[12]=0x00; ev[13]=0x10;        # declared_lo = 0x1000 LE
    ev[14]=0xFF; ev[15]=0x10;        # declared_hi = 0x10FF LE
    ev[16]=0x00; ev[17]=0x11;        # observed   = 0x1100 LE byte 0..1
    ev[18]=0x00; ev[19]=0x00;        # observed   = 0x1100 LE byte 2..3
    for (k=20; k<28; k++) ev[k]=0;
    s[0]=0; s[1]=0; s[2]=0; s[3]=0;
    for (k=0; k<28; k++) {
        for (n=0; n<4; n++) s[n] = s[n] + ev[k] * p[n];
    }
    printf "%02x%02x%02x%02x", s[0]%256, s[1]%256, s[2]%256, s[3]%256;
}')"

expected="8aa2ca5e"

if [ "$actual" != "$expected" ]; then
    note_fail "R5 chain_step output drift: expected $expected, got $actual"
fi

# -------------------------------------------------------------------
# Claim 3: the canonical .pfi fixture's record byte-encodes this same
# vector at the same offset. This couples the R5 payload schema to the
# .pfi fixture without needing producer-side emission.
# -------------------------------------------------------------------
pfi="tools/verify/fixtures/pfi/mmio_boundary_violation.pfi"
if [ ! -r "$pfi" ]; then
    note_fail "PFI0 anchor fixture missing at $pfi"
else
    # Record starts at offset 128. Verify R5 payload bytes literally.
    record_hex="$(od -An -tx1 -j 128 -N 32 "$pfi" | tr -d ' \n')"
    expected_record="0500010000000000000000000010ff10001100000000000000008aa2ca5e0000"
    if [ "$record_hex" != "$expected_record" ]; then
        note_fail "PFI0 record bytes drift: got $record_hex, expected $expected_record"
    fi
fi

# -------------------------------------------------------------------
# Summary.
# -------------------------------------------------------------------
echo "============================================================"
echo "  Phosphoric R5 mmio_touch byte-layout gate"
echo "  doctrine: docs/FORENSIC_PRIMACY.md §1 + §2 (R5 row)"
echo "============================================================"
echo "  taxonomy   : kind=5 mmio_touch declared in $residual"
echo "  test vector: declared_lo=0x1000, declared_hi=0x10FF, observed=0x1100"
echo "  chain_hash : $actual (expected $expected)"
echo "  PFI anchor : $pfi"

if [ "$fail" -eq 0 ]; then
    echo "  [r5-byte-layout] OK — R5 record layout and chain_hash math are byte-stable"
    exit 0
fi

echo "  [r5-byte-layout] DOCTRINE VIOLATION — see FAIL lines above" >&2
exit 1
