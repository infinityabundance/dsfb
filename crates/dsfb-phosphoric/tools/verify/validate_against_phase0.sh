#!/usr/bin/env bash
# Phosphoric Stream B phase-0 validation harness.
#
# Per the internal phase-0 roadmap, every session that ships a new
# producer shape MUST run this harness as part of its gate set. The harness:
#
#   1. Builds + runs phase0_stub on phase0/phase0_compiler.phos.
#   2. Captures the output sha (build/phase0/pcc-stage0.bin).
#   3. Compares against the baseline in
#      tools/verify/phase0_baseline.toml.
#   4. Reports UNCHANGED (synthetic-only session) or ADVANCE (some
#      phase0_compiler.phos fn now lowers differently — investigate).
#
# Sessions that intentionally advance the hash MUST update
# tools/verify/phase0_baseline.toml in the same commit as the
# corresponding bootstrap/bootstrap.toml attestation note.
#
# Exit codes:
#   0 — UNCHANGED or ADVANCE (both are valid outcomes; doctrine layer
#       separately ensures baseline + attestation stay aligned).
#   1 — build failure or missing baseline file.

set -e

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

baseline_file="tools/verify/phase0_baseline.toml"
out="build/phase0/pcc-stage0.bin"

if [ ! -r "$baseline_file" ]; then
    echo "[phase-0] FAIL: baseline file not found at $baseline_file" >&2
    exit 1
fi

# Build (idempotent — produce_stage0.sh re-runs the ASM stub on
# phase0_compiler.phos and writes pcc-stage0.bin).
if ! bash untracked/internaldocs/phase0_producer/produce_stage0.sh > /dev/null 2>&1; then
    echo "[phase-0] FAIL: produce_stage0.sh failed" >&2
    exit 1
fi

if [ ! -r "$out" ]; then
    echo "[phase-0] FAIL: output not found at $out" >&2
    exit 1
fi

current_sha="$(sha256sum "$out" | awk '{print $1}')"
current_size="$(stat -c %s "$out")"

baseline_sha="$(awk -F\" '/^sha256/ {print $2; exit}' "$baseline_file")"
baseline_size="$(awk '/^size_bytes/ {print $3; exit}' "$baseline_file")"

echo "============================================================"
echo "  Phase-0 validation — pcc-stage0.bin (Stream B harness)"
echo "============================================================"
echo "  baseline: sha=${baseline_sha}"
echo "            size=${baseline_size}"
echo "  current:  sha=${current_sha}"
echo "            size=${current_size}"
echo "------------------------------------------------------------"

if [ "$current_sha" = "$baseline_sha" ]; then
    echo "  [phase-0] UNCHANGED"
    echo
    echo "  This session ships infrastructure-only — no fn body in"
    echo "  phase0/phase0_compiler.phos matches the new shape's"
    echo "  strict pattern. The new shape is verifiable via its"
    echo "  synthetic fixture in tools/verify/fixture_manifest.toml."
    echo
    exit 0
fi

echo "  [phase-0] ADVANCE"
echo
echo "  pcc-stage0.bin's bytes have changed. Some phase0_compiler.phos"
echo "  fn now lowers differently."
echo
echo "  Required follow-up before commit:"
echo "    1. Verify which phase0_compiler.phos fn(s) changed (compare"
echo "       fn_offset_table-derived offsets against bytes at those"
echo "       offsets)."
echo "    2. Confirm the new bytes match the discriminator's intended"
echo "       emit shape (not an accidental over-fire)."
echo "    3. Update $baseline_file:"
echo "       sha256 = \"${current_sha}\""
echo "       size_bytes = ${current_size}"
echo "       last_advanced_session = \"<session id>\""
echo "       last_advanced_date = \"<YYYY-MM-DD>\""
echo "    4. Append [[stage0.attestation]] to bootstrap/bootstrap.toml"
echo "       documenting the advance."
echo
exit 0
