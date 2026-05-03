#!/usr/bin/env bash
# check_court_d1_verdict.sh — Court Requirement D1 host reference
# verdict path replay gate.
#
# Closes the host-reference loop:
#
#   input vector
#     → R5 32-byte record   (tools/court/emit_r5_record.sh)
#     → PFI0 192-byte case  (tools/court/emit_mmio_boundary_pfi.sh)
#     → 6-line verdict      (tools/court/verdict_from_pfi.sh)
#
# Asserts the final 6-line verdict bytes are byte-identical to the
# locked expectation at
# tools/verify/fixtures/verdicts/mmio_boundary_violation.expect.
#
# Input is the A1/B1 emitter's *produced* PFI0 bytes — not the static
# anchor. A1/B1 (check_court_a1_b1.sh) already proved
# produced PFI0 == anchor PFI0 byte-for-byte; this gate proves
# produced PFI0 → verdict == anchor → verdict == .expect, completing
# the chain from input vector to canonical verdict.
#
# Forensic claim earned: given the canonical MMIO violation vector,
# the host reference court deterministically produces the R5 PFI0
# case bytes and derives the canonical MMIO_BOUNDARY_PRESSURE
# verdict bytes, byte-identical to the locked expectation.
#
# Forensic claim NOT earned: Phosphoric runtime classifier executes;
# court runtime adjudicates; compiled classifier emits verdict — the
# tool chain is host-side bash + awk + od + sha256sum.
#
# Doctrine: docs/PFI0.md (PFI0 layout), docs/FORENSIC_PRIMACY.md §3
# (canonical 6-line verdict + DriftClass enum + EXIT mapping),
# kernel/residual.phos (R5 struct + chain_step mixer).
#
# Exit: 0 pass, 1 verdict drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

emitter="tools/court/emit_mmio_boundary_pfi.sh"
verdict_tool="tools/court/verdict_from_pfi.sh"
expect="tools/verify/fixtures/verdicts/mmio_boundary_violation.expect"

for path in "$emitter" "$verdict_tool"; do
    if [ ! -x "$path" ]; then
        echo "[court-d1-verdict] missing or not executable: $path" >&2
        exit 2
    fi
done
if [ ! -r "$expect" ]; then
    echo "[court-d1-verdict] expected verdict missing: $expect" >&2
    exit 2
fi

pfi_tmp="$(mktemp)"
verdict_tmp="$(mktemp)"
trap 'rm -f "$pfi_tmp" "$verdict_tmp"' EXIT

bash "$emitter" > "$pfi_tmp"
bash "$verdict_tool" "$pfi_tmp" > "$verdict_tmp"

actual_size="$(wc -c < "$verdict_tmp")"
expected_size="$(wc -c < "$expect")"

echo "============================================================"
echo "  Phosphoric Court Requirement D1 — host reference verdict path"
echo "  doctrine: docs/PFI0.md + docs/FORENSIC_PRIMACY.md §3"
echo "============================================================"
echo "  pipeline           : input vector → R5 → PFI0 → verdict"
echo "  emitter            : $emitter"
echo "  verdict tool       : $verdict_tool"
echo "  expectation        : $expect"
echo "  produced PFI0      : $(wc -c < "$pfi_tmp") bytes"
echo "  produced verdict   : $actual_size bytes"
echo "  expected verdict   : $expected_size bytes"

if ! cmp -s "$verdict_tmp" "$expect"; then
    echo "  [court-d1-verdict] FAIL — verdict bytes differ from expectation:" >&2
    diff "$verdict_tmp" "$expect" >&2 || true
    exit 1
fi

echo "  [court-d1-verdict] OK — host reference verdict path produced canonical verdict bytes byte-identical to verdict expectation"
exit 0
