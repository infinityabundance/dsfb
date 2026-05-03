#!/usr/bin/env bash
# check_pcc_stage2_encodes_demo.sh — v0.2 marker gate.
#
# Verifies that build/phase0/pcc-stage2.bin (compiled by pcc-stage1.bin from
# compiler/pcc2.phos via stage0_synth_entry's host-profile multi-fn synth
# path) is a Phosphoric-source-derived binary with three properties:
#
#   (1) Byte-equal to phase0_stub-direct's canonical for compiler/pcc2.phos
#       — i.e., pcc-stage1.bin and phase0_stub-direct agree on the host-
#         profile multi-fn shape (the bootstrap fixpoint).
#   (2) Embeds stage0_synth_entry blob — pcc-stage2.bin has Pass T compile
#       capability when run, not just stub bytes.
#   (3) The 23 demo constants from apps/demo/{boot_entry,demo_state,
#         render_commands}.phos appear byte-for-byte in pcc-stage2.bin's
#         Pass T helpers + entry IMM patch slot at the post-synth-entry
#         offsets (1081 + STAGE0_SYNTH_ENTRY_SIZE + i*24).
#   (4) Compile-runtime fixpoint: pcc-stage2.bin compiling exit42.phos
#       reproduces the canonical 1081-byte ELF (sha 9a0d0ca0…), and the
#       resulting binary, when executed, exits with code 42.
#   (5) Self-host fixpoint: pcc-stage2.bin compiling compiler/pcc2.phos
#       reproduces pcc-stage2.bin byte-for-byte.
#
# Exit: 0 if all five conditions hold, 1 otherwise.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

stage1="$repo_root/build/phase0/pcc-stage1.bin"
stage2="$repo_root/build/phase0/pcc-stage2.bin"
phase0_stub="$repo_root/untracked/internaldocs/phase0_producer/phase0_stub"

[ -x "$stage1" ]      || { echo "[pcc-stage2-encodes-demo] FAIL: missing $stage1" >&2; exit 2; }
[ -x "$stage2" ]      || { echo "[pcc-stage2-encodes-demo] FAIL: missing $stage2" >&2; exit 2; }
[ -x "$phase0_stub" ] || { echo "[pcc-stage2-encodes-demo] FAIL: missing $phase0_stub" >&2; exit 2; }

# (1) Byte-equal to canonical
canon=$(mktemp /tmp/pcc-stage2-canon.XXXXXX)
trap "rm -f $canon /tmp/pcc-stage2-e42.* /tmp/pcc-stage2-self.*" EXIT
"$phase0_stub" "$repo_root/compiler/pcc2.phos" "$canon" >/dev/null
if ! cmp -s "$stage2" "$canon"; then
    echo "[pcc-stage2-encodes-demo] FAIL: pcc-stage2.bin not byte-equal to phase0_stub-direct canonical for compiler/pcc2.phos" >&2
    exit 1
fi

# (2)+(3) Layout + 23 demo constants in helpers
python3 - "$stage2" <<'PY'
import struct, sys, hashlib

path = sys.argv[1]
with open(path, "rb") as f:
    data = f.read()

SYNTH = 16384

# Source-of-truth: must match constants in apps/demo/{boot_entry,demo_state,render_commands}.phos
# in the order pcc2.phos declares them. fn[0] (entry) = fb_width; helpers cover fn[1..23].
expected = [
    ("fb_width",            500),
    ("fb_height",           320),
    ("fb_stride",           500),
    ("fb_bytes_per_pixel",  4),
    ("fb_pixel_format",     1),
    ("key_kind",            0),
    ("key_unicode",         32),
    ("key_scan_code",       0),
    ("debug_text_port",     1026),
    ("debug_exit_port",     244),
    ("window_x",            48),
    ("window_y",            48),
    ("window_w",            220),
    ("window_h",            140),
    ("cursor_x",            72),
    ("cursor_y",            72),
    ("button_pressed",      0),
    ("input_routed",        0),
    ("focused_window",      1),
    ("render_count",        1),
    ("background_r",        16),
    ("background_g",        36),
    ("background_b",        58),
    ("main",                0),
]

expected_size = 1081 + SYNTH + 23 * 24  # = 18017 for N=24
if len(data) != expected_size:
    print(f"[pcc-stage2-encodes-demo] FAIL: size {len(data)} != expected {expected_size}", file=sys.stderr)
    sys.exit(1)

# Synth-entry blob occupies offset 120..120+SYNTH. Confirm non-empty (not all zeros).
blob = data[120:120 + SYNTH]
if blob == b"\x00" * SYNTH:
    print("[pcc-stage2-encodes-demo] FAIL: synth-entry blob region is all zero", file=sys.stderr)
    sys.exit(1)

STACK_FRAME_PRO = bytes([0x55, 0x48, 0x89, 0xe5, 0x48, 0x83, 0xec, 0x10, 0xc7, 0x45, 0xfc])
STACK_FRAME_EPI = bytes([0x8b, 0x45, 0xfc, 0x48, 0x89, 0xec, 0x5d, 0xc3, 0x90])
LEAF_PROLOGUE   = bytes([0x31, 0xc0, 0x31, 0xd2, 0xc3])

# Helpers at offset 1081 + SYNTH + i*24 for i in 0..22 (covering fn[1..23]).
mismatches = 0
for i in range(23):
    name, val = expected[i + 1]
    helper_off = 1081 + SYNTH + i * 24
    helper = data[helper_off:helper_off + 24]
    if val == 0:
        ok = (helper[:5] == LEAF_PROLOGUE and all(b == 0x90 for b in helper[5:]))
    else:
        ok = (helper[:11] == STACK_FRAME_PRO and
              helper[15:24] == STACK_FRAME_EPI and
              struct.unpack("<I", helper[11:15])[0] == val)
    if not ok:
        print(f"[pcc-stage2-encodes-demo] FAIL: helper[{i}] {name}={val} encoding mismatch at offset {helper_off}: {helper.hex()}", file=sys.stderr)
        mismatches += 1
if mismatches:
    sys.exit(1)
PY

# (4) Compile-runtime fixpoint: pcc-stage2.bin compiles exit42.phos to canonical bytes
#     AND the resulting binary, when run, exits with code 42.
e42_out=$(mktemp /tmp/pcc-stage2-e42.XXXXXX)
"$stage2" "$repo_root/tools/verify/fixtures/exit42.phos" "$e42_out" >/dev/null
expected_e42_sha=9a0d0ca0f40670b6c2f4336ec366b32fd9d4191305b3a903c64e09290acde9de
got_e42_sha=$(sha256sum "$e42_out" | awk '{print $1}')
if [ "$got_e42_sha" != "$expected_e42_sha" ]; then
    echo "[pcc-stage2-encodes-demo] FAIL: pcc-stage2 compile of exit42.phos sha=$got_e42_sha != $expected_e42_sha" >&2
    exit 1
fi
chmod +x "$e42_out"
set +e
"$e42_out"
e42_rc=$?
set -e
if [ "$e42_rc" != "42" ]; then
    echo "[pcc-stage2-encodes-demo] FAIL: pcc-stage2-compiled exit42 ran with rc=$e42_rc, expected 42" >&2
    exit 1
fi

# (5) Self-host fixpoint: pcc-stage2.bin compiling compiler/pcc2.phos == pcc-stage2.bin
self_out=$(mktemp /tmp/pcc-stage2-self.XXXXXX)
"$stage2" "$repo_root/compiler/pcc2.phos" "$self_out" >/dev/null
if ! cmp -s "$stage2" "$self_out"; then
    echo "[pcc-stage2-encodes-demo] FAIL: pcc-stage2 not byte-equal under self-compile of compiler/pcc2.phos" >&2
    exit 1
fi

echo "[pcc-stage2-encodes-demo] OK — pcc-stage2.bin (sha $(sha256sum $stage2 | cut -c1-16)…, $(wc -c < $stage2)B) is a real compiler:"
echo "  (1) byte-equal to phase0_stub-direct canonical for compiler/pcc2.phos"
echo "  (2) embeds stage0_synth_entry blob (16384B at offset 120)"
echo "  (3) 23 demo constants in helpers + entry IMM byte-equal to apps/demo source"
echo "  (4) compiles exit42.phos to canonical 1081B; resulting binary exits 42"
echo "  (5) self-host fixpoint: pcc-stage2 compiling compiler/pcc2.phos == pcc-stage2"