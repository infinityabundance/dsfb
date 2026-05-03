#!/usr/bin/env bash
set -euo pipefail
#
# encode_pfi.sh — wrap raw 32-byte residual records in a PFI0 case file.
#
# Usage: encode_pfi.sh RECORDS_BIN MANIFEST_PATH IMAGE_PATH PFI_OUT
#
# Layout (per docs/PFI0.md):
#   0..32     header        magic="PFI0" + residual_count u32 LE + reserved [u8;24]
#   32..64    manifest_hash sha256 of $MANIFEST_PATH
#   64..96    image_hash    sha256 of $IMAGE_PATH
#   96..128   stream_hash   sha256 of records bytes
#   128..128+32*N  records[0..N-1]
#   final_chain_hash[4] + reserved[28]   (footer; chain_hash from last record)
#
# Total: 192 + 32*(N-1) bytes for N>=1.

records_bin="${1:?usage: encode_pfi.sh RECORDS_BIN MANIFEST_PATH IMAGE_PATH PFI_OUT}"
manifest_path="${2:?missing MANIFEST_PATH}"
image_path="${3:?missing IMAGE_PATH}"
pfi_out="${4:?missing PFI_OUT}"

[ -r "$records_bin" ] || { echo "encode_pfi: missing $records_bin" >&2; exit 2; }
[ -r "$manifest_path" ] || { echo "encode_pfi: missing $manifest_path" >&2; exit 2; }
[ -r "$image_path" ] || { echo "encode_pfi: missing $image_path" >&2; exit 2; }

records_size=$(wc -c < "$records_bin")
if [ $((records_size % 32)) -ne 0 ] || [ "$records_size" -lt 32 ]; then
    echo "encode_pfi: records size $records_size is not a positive multiple of 32" >&2
    exit 1
fi
N=$((records_size / 32))

python3 - "$records_bin" "$manifest_path" "$image_path" "$pfi_out" <<'PY'
import hashlib, struct, sys

records_bin, manifest_path, image_path, pfi_out = sys.argv[1:5]
with open(records_bin, "rb") as f:
    records = f.read()
with open(manifest_path, "rb") as f:
    manifest_hash = hashlib.sha256(f.read()).digest()
with open(image_path, "rb") as f:
    image_hash = hashlib.sha256(f.read()).digest()
stream_hash = hashlib.sha256(records).digest()
N = len(records) // 32

# Header (32 bytes)
header = b"PFI0" + struct.pack("<I", N) + b"\x00" * 24

# Footer: final_chain_hash (last record's chain_hash, 4 bytes) + 28 zeros.
last_record = records[(N - 1) * 32 : N * 32]
final_chain_hash = last_record[26:30]
footer = final_chain_hash + b"\x00" * 28

pfi = header + manifest_hash + image_hash + stream_hash + records + footer
expected = 192 + 32 * (N - 1)
assert len(pfi) == expected, f"size {len(pfi)} != expected {expected}"
with open(pfi_out, "wb") as f:
    f.write(pfi)
PY

printf 'wrote %s (%s bytes; %s records)\n' "$pfi_out" "$(stat -c '%s' "$pfi_out")" "$N"
