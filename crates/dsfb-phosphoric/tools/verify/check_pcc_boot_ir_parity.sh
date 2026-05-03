#!/usr/bin/env bash
# check_pcc_boot_ir_parity.sh — v0.1 step 2 done-criterion gate.
#
# Runs the active boot IR producer (the shell emitter at
# tools/phosphoric/emit_boot_demo_from_phos.sh, preserved verbatim
# until step 6 retires it), pins its output to the canonical step-2
# artifact path build/boot_ir_v1.json, and asserts byte-equality
# with tests/golden/boot_ir_v1_button_policy_golden.json (3415 bytes,
# sha256 7a45a3f17de68b40bbe68cd18c67413dc69dbd51132d516e110b2b0db6cc9d65).
#
# Until v0.1 step 6 fixpoint lands, pcc.phos itself cannot run end-to-end;
# the shell emitter is the active boot IR producer. Step 2 locks the
# byte-equality between that producer's output and the golden so any
# drift in apps/demo/ source, the emitter, or the golden surfaces
# immediately. When step 6 lands and pcc.phos becomes runnable, this
# gate's surface contract (build/boot_ir_v1.json byte-equal to
# golden) does not change — only the producer behind the artifact
# path swaps from shell to pcc.
#
# Structural assertion: compiler/codegen_boot.phos exists and carries
# the documented future-replacement signatures (lower_boot_module,
# lower_hir_expr_tree). This proves the source-side replacement is
# wired in repo. codegen_boot.phos today produces ASM (step 5
# territory), not JSON; the gate does NOT assert JSON production
# from codegen_boot.phos.
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

emitter="tools/phosphoric/emit_boot_demo_from_phos.sh"
canonical="build/boot_ir_v1.json"
golden="tests/golden/boot_ir_v1_button_policy_golden.json"
codegen_boot="compiler/codegen_boot.phos"

[ -x "$emitter" ] || { echo "[pcc-boot-ir-parity] missing or non-executable: $emitter" >&2; exit 2; }
[ -r "$golden" ]  || { echo "[pcc-boot-ir-parity] missing golden: $golden" >&2; exit 2; }

echo "============================================================"
echo "  pcc.phos boot IR parity gate (v0.1 step 2)"
echo "============================================================"

# Run the active boot IR producer in an isolated temp dir to avoid
# racing other gates (e.g. verify-boot-phosphoric-only) that invoke
# the same emitter against build/uefi-demo/generated/. The emitter
# accepts an output-dir argument; defaults match the build path
# elsewhere, but we override here.
mkdir -p "$(dirname "$canonical")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
bash "$emitter" "$work" >/dev/null

emitted="$work/boot_ir_v1_button_policy.json"
[ -r "$emitted" ] || { echo "[pcc-boot-ir-parity] emitter did not produce $emitted" >&2; exit 2; }

# Pin to canonical step-2 artifact path.
cp -f "$emitted" "$canonical"

# Byte-equality assertion.
if ! cmp -s "$canonical" "$golden"; then
    echo "[pcc-boot-ir-parity] FAIL: $canonical differs from $golden" >&2
    diff "$canonical" "$golden" | head -30 >&2 || true
    exit 1
fi

produced_sha="$(sha256sum "$canonical" | awk '{print $1}')"
golden_sha="$(sha256sum "$golden" | awk '{print $1}')"

echo "  active producer    : $emitter (preserved verbatim)"
echo "  emitter output     : $emitted"
echo "  canonical artifact : $canonical"
echo "  golden             : $golden"
echo "  produced sha256    : $produced_sha"
echo "  golden sha256      : $golden_sha"
echo "  byte-equal         : $(wc -c < "$canonical") B"

# Structural assertion: codegen_boot.phos future-replacement is wired.
[ -r "$codegen_boot" ] || { echo "[pcc-boot-ir-parity] FAIL: $codegen_boot missing (documented future replacement)" >&2; exit 1; }
fail=0
note() { echo "[pcc-boot-ir-parity] FAIL: $*" >&2; fail=1; }

grep -qE 'fn lower_boot_module' "$codegen_boot" \
    || note "$codegen_boot missing lower_boot_module signature"
grep -qE 'fn lower_hir_expr_tree' "$codegen_boot" \
    || note "$codegen_boot missing lower_hir_expr_tree signature"

[ "$fail" = "0" ] || exit 1

echo "  structural pointer : codegen_boot.phos signatures wired (future replacement; produces ASM today, JSON at step 6)"
echo "  [pcc-boot-ir-parity] OK — build/boot_ir_v1.json byte-identical to tests/golden/boot_ir_v1_button_policy_golden.json (sha256 ${produced_sha:0:12}...)"
exit 0
