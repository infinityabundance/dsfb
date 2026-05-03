#!/usr/bin/env bash
# check_court_b1_case_validity.sh — Court Requirement B1 (narrow) host
# reference R5 case-validity gate.
#
# Forensic role: enforce the missing semantic invariant that the
# layout / chain / verdict gates do not — that an R5 mmio_touch
# payload encodes an actual MMIO boundary violation, not an in-range
# touch mislabeled R5. Concretely:
#
#   for record[0] of an R5 case:
#     kind == 5
#     observed < declared_lo OR observed > declared_hi
#
# The gate runs the A1/B1 emitter to produce the canonical 192-byte
# PFI0 case, pipes it through tools/court/validate_r5_case.sh, and
# requires exit 0. For the canonical vector (declared_lo=0x1000,
# declared_hi=0x10FF, observed=0x1100), 0x1100 > 0x10FF so the case
# is a real boundary violation and the validator exits 0.
#
# B1-narrow scope: this gate adds *only* the semantic invariant.
# Format / layout / hash-chain validation belong to:
#
#   - check_pfi_layout.sh           (PFI0 magic, residual_count,
#                                    stream_hash, final_chain_hash)
#   - check_residual_r5_byte_layout.sh
#                                   (R5 record byte layout + chain_step
#                                    mixer math against canonical vector)
#   - check_malformed_pfi.sh        (adversarial / malformed rejection)
#   - check_court_a1_b1.sh          (A1/B1 anchor reproducibility)
#   - check_court_d1_verdict.sh     (D1 verdict path)
#
# B1 deliberately does NOT re-implement any of those.
#
# Forensic claim earned: the host reference court not only reproduces
# the R5/PFI0/verdict bytes, but also validates that the R5 payload
# semantically represents a real boundary violation — observed lies
# outside the declared MMIO range.
#
# Forensic claim NOT earned: general PFI validator, general R5
# classifier, runtime enforcement, Phosphoric-compiled validation —
# the chain is host-side bash + awk + od + sha256sum.
#
# Doctrine: docs/PFI0.md, docs/FORENSIC_PRIMACY.md §1,
# kernel/residual.phos.
#
# Exit: 0 pass, 1 semantic violation, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

emitter="tools/court/emit_mmio_boundary_pfi.sh"
validator="tools/court/validate_r5_case.sh"

for path in "$emitter" "$validator"; do
    if [ ! -x "$path" ]; then
        echo "[court-b1-case-validity] missing or not executable: $path" >&2
        exit 2
    fi
done

pfi_tmp="$(mktemp)"
trap 'rm -f "$pfi_tmp"' EXIT

bash "$emitter" > "$pfi_tmp"

echo "============================================================"
echo "  Phosphoric Court Requirement B1 (narrow) — R5 case-validity gate"
echo "  doctrine: docs/PFI0.md + docs/FORENSIC_PRIMACY.md §1"
echo "============================================================"
echo "  invariant          : kind == 5 AND observed ∉ [declared_lo, declared_hi]"
echo "  emitter            : $emitter"
echo "  validator          : $validator"
echo "  produced PFI0      : $(wc -c < "$pfi_tmp") bytes"

if ! bash "$validator" "$pfi_tmp"; then
    echo "  [court-b1-case-validity] FAIL — R5 payload does not encode an actual MMIO boundary violation" >&2
    exit 1
fi

echo "  [court-b1-case-validity] OK — R5 payload encodes an actual MMIO boundary violation"
exit 0
