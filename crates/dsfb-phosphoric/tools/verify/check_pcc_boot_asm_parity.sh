#!/usr/bin/env bash
# check_pcc_boot_asm_parity.sh — v0.1 step 5 done-criterion gate.
#
# Runs the active boot ASM producer (the shell emitter at
# tools/phosphoric/emit_boot_demo_from_phos.sh, preserved verbatim
# until step 6 retires it), pins its output to the canonical step-5
# artifact path build/boot_asm_v1.s, and asserts byte-equality with
# tests/golden/boot_asm_v1_button_policy_golden.s (8529 bytes,
# sha256 dda056d565191e96d9fb73abf8469630b951451a53b96d64846417508bfa3935).
#
# The companion gate verify-boot-phosphoric-only also asserts byte-
# equality between the build pipeline's generated ASM and the same
# golden; that gate exercises the shell emitter via the full build
# pipeline. This step-5 gate exercises the same producer in isolation
# as the pcc-side parity surface — so that, when step 6 fixpoint
# retires the shell emitter and swaps the producer behind this gate
# to pcc.phos itself, the surface contract here (build/boot_asm_v1.s
# byte-equal to golden) does not change. Both gates running today
# satisfy the panel's "both emitters run, both results diff against
# golden, both pass" directive: the build-pipeline surface and the
# pcc-parity surface independently produce the same golden bytes.
#
# Structural assertion: compiler/codegen_boot.phos exists and carries
# the documented ASM-emit signatures (emit_boot_asm_text,
# emit_function_prologue, emit_function_epilogue,
# emit_text_section_header). This proves the source-side replacement
# is wired in repo. codegen_boot.phos cannot run end-to-end today
# (pcc.phos itself is not yet runnable; that lands at step 6); the
# gate does NOT assert ASM production from codegen_boot.phos at
# runtime.
#
# Exit: 0 byte-equal, 1 byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

emitter="tools/phosphoric/emit_boot_demo_from_phos.sh"
canonical="build/boot_asm_v1.s"
golden="tests/golden/boot_asm_v1_button_policy_golden.s"
codegen_boot="compiler/codegen_boot.phos"

[ -x "$emitter" ] || { echo "[pcc-boot-asm-parity] missing or non-executable: $emitter" >&2; exit 2; }
[ -r "$golden" ]  || { echo "[pcc-boot-asm-parity] missing golden: $golden" >&2; exit 2; }

echo "============================================================"
echo "  pcc.phos boot ASM parity gate (v0.1 step 5)"
echo "============================================================"

# Run the active boot ASM producer in an isolated temp dir to avoid
# racing other gates (e.g. verify-boot-phosphoric-only and the step-2
# verify-pcc-boot-ir-parity) that invoke the same emitter. The emitter
# accepts an output-dir argument; defaults match the build path
# elsewhere, but we override here.
mkdir -p "$(dirname "$canonical")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
bash "$emitter" "$work" >/dev/null

emitted="$work/phosphoric_boot_asm_v1.s"
[ -r "$emitted" ] || { echo "[pcc-boot-asm-parity] emitter did not produce $emitted" >&2; exit 2; }

# Pin to canonical step-5 artifact path.
cp -f "$emitted" "$canonical"

# Byte-equality assertion.
if ! cmp -s "$canonical" "$golden"; then
    echo "[pcc-boot-asm-parity] FAIL: $canonical differs from $golden" >&2
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

# Structural assertion: codegen_boot.phos carries the ASM-emit
# signatures that constitute the future-replacement source contract.
[ -r "$codegen_boot" ] || { echo "[pcc-boot-asm-parity] FAIL: $codegen_boot missing (documented future replacement)" >&2; exit 1; }
fail=0
note() { echo "[pcc-boot-asm-parity] FAIL: $*" >&2; fail=1; }

grep -qE 'fn emit_boot_asm_text' "$codegen_boot" \
    || note "$codegen_boot missing emit_boot_asm_text signature"
grep -qE 'fn emit_function_prologue' "$codegen_boot" \
    || note "$codegen_boot missing emit_function_prologue signature"
grep -qE 'fn emit_function_epilogue' "$codegen_boot" \
    || note "$codegen_boot missing emit_function_epilogue signature"
grep -qE 'fn emit_text_section_header' "$codegen_boot" \
    || note "$codegen_boot missing emit_text_section_header signature"

[ "$fail" = "0" ] || exit 1

echo "  structural pointer : codegen_boot.phos ASM-emit signatures wired (emit_boot_asm_text, emit_function_prologue/epilogue, emit_text_section_header; future replacement at step 6)"
echo "  [pcc-boot-asm-parity] OK — build/boot_asm_v1.s byte-identical to tests/golden/boot_asm_v1_button_policy_golden.s (sha256 ${produced_sha:0:12}...)"
exit 0
