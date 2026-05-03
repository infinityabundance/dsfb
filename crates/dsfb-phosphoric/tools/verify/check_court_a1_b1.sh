#!/usr/bin/env bash
# check_court_a1_b1.sh — Court Requirement A1/B1 anchor replay gate.
#
# Runs the host reference emitter at
# tools/court/emit_mmio_boundary_pfi.sh and asserts its 192-byte PFI0
# output is byte-identical to the canonical anchor at
# tools/verify/fixtures/pfi/mmio_boundary_violation.pfi.
#
# What this gate proves: the hand-encoded anchor is no longer only a
# static fixture; given the canonical declared-MMIO-violation vector
# (declared_lo=0x1000, declared_hi=0x10FF, observed=0x1100), the court
# reference emitter reproduces it byte-for-byte from inputs. The R5
# record bytes, the chain_step mixer, the SHA-256 stream_hash over the
# record, and the PFI0 wrapper are all reproducible by deterministic
# composition from the inputs.
#
# What this gate does NOT prove yet: the bytes were emitted by a
# Phosphoric-compiled binary. The emitter is host-side bash + awk +
# sha256sum. A subsequent court-side step may replace the emitter with
# a Phosphoric-compiled binary that produces the same bytes; until
# then, the claim is "host reference emitter produced", not
# "Phosphoric-runtime produced".
#
# Doctrine: docs/PFI0.md (PFI0 layout), kernel/residual.phos (R5 +
# chain_step), tools/verify/check_residual_r5_byte_layout.sh (R5
# canonical vector), docs/FORENSIC_PRIMACY.md.
#
# Exit: 0 pass, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

emitter="tools/court/emit_mmio_boundary_pfi.sh"
anchor="tools/verify/fixtures/pfi/mmio_boundary_violation.pfi"

if [ ! -x "$emitter" ]; then
    echo "[court-a1-b1] emitter missing or not executable: $emitter" >&2
    exit 2
fi
if [ ! -r "$anchor" ]; then
    echo "[court-a1-b1] anchor missing: $anchor" >&2
    exit 2
fi

actual_tmp="$(mktemp)"
trap 'rm -f "$actual_tmp"' EXIT

bash "$emitter" > "$actual_tmp"

actual_size="$(wc -c < "$actual_tmp")"
expected_size="$(wc -c < "$anchor")"

echo "============================================================"
echo "  Phosphoric Court Requirement A1/B1 — anchor replay gate"
echo "  doctrine: docs/PFI0.md + docs/FORENSIC_PRIMACY.md"
echo "============================================================"
echo "  declared MMIO range: 0x1000..0x10FF"
echo "  observed touch     : 0x1100 (one byte above declared_hi)"
echo "  emitter            : $emitter"
echo "  anchor             : $anchor"
echo "  emitter output     : $actual_size bytes"
echo "  anchor             : $expected_size bytes"

if [ "$actual_size" != "192" ] || [ "$expected_size" != "192" ]; then
    echo "  [court-a1-b1] FAIL — size mismatch" >&2
    exit 1
fi

if ! cmp -s "$actual_tmp" "$anchor"; then
    echo "  [court-a1-b1] FAIL — bytes differ from anchor:" >&2
    diff <(xxd "$actual_tmp") <(xxd "$anchor") | head -30 >&2
    exit 1
fi

echo "  [court-a1-b1] OK — host reference emitter produced PFI0 bytes byte-identical to anchor"
exit 0
