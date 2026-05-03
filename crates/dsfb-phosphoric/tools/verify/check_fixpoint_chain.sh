#!/usr/bin/env bash
# check_fixpoint_chain.sh — bootstrap chain observability gate.
#
# At doctrine-active state (`bootstrap.toml` `[[stage0]] status = "active"`
# AND the producer can compile `compiler/pcc.phos` into a working
# self-hosted compiler), this script runs the full convergence test:
#
#   stage0 (attested binary) -> compiles pcc.phos -> stage1.bin
#   stage1                    -> compiles pcc.phos -> stage2.bin
#   stage2                    -> compiles pcc.phos -> stage3.bin
#   ASSERT sha256(stage2) == sha256(stage3)
#
# Until the producer reaches Pass M.3+ codegen (real per-fn variable-size
# instruction emission), stage1 from the current ASM stub is a degenerate
# 360-byte ELF that exits 0 without consuming argv — running it on
# pcc.phos produces no stage2, so the chain breaks at stage1->stage2.
# This script reports that honestly and exits 0 as INFORMATIONAL while
# the manifest is below SCAFFOLD-active. Once the producer can actually
# compile pcc.phos, the script transitions to fail-close on divergence.
#
# Exit codes:
#   0 — chain ran to convergence (stage2 == stage3) OR the manifest is
#       still SCAFFOLD-tier and the chain breaks at the documented gate
#       (informational while below active state).
#   1 — chain ran but stage2 != stage3 (divergence after active state).
#   2 — manifest inconsistency (no stage0 binary, or pinned hash mismatch).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

manifest="bootstrap/bootstrap.toml"
stage0_bin="build/phase0/pcc-stage0.bin"
pcc_src="compiler/pcc.phos"
producer="untracked/internaldocs/phase0_producer/phase0_stub"

# ----------------------------------------------------------------------
# 1. Manifest sanity: the active stage0 entry has a pinned hash and the
#    on-disk binary matches.
# ----------------------------------------------------------------------
expected_hash="$(awk -F'"' '/^binary_sha256/ { print $2; exit }' "$manifest")"
if [ -z "$expected_hash" ]; then
    echo "[fixpoint-chain] manifest has no binary_sha256 entry (skipping; exit 2)"
    exit 2
fi

if [ ! -x "$stage0_bin" ]; then
    echo "[fixpoint-chain] stage0 binary missing at $stage0_bin (skipping; exit 2)"
    exit 2
fi

actual_hash="$(sha256sum "$stage0_bin" | awk '{print $1}')"
if [ "$actual_hash" != "$expected_hash" ]; then
    echo "[fixpoint-chain] stage0 hash drift: expected $expected_hash got $actual_hash (exit 2)" >&2
    exit 2
fi

# ----------------------------------------------------------------------
# 2. Status check: is the manifest active or scaffold?
# ----------------------------------------------------------------------
status="$(awk -F'"' '/^status[[:space:]]*=/ { print $2; exit }' "$manifest")"

# ----------------------------------------------------------------------
# 3. Try the chain. stage1 = stage0 on pcc.phos.
# ----------------------------------------------------------------------
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

stage1="$work/stage1.bin"
"$stage0_bin" "$pcc_src" "$stage1" 2>"$work/stage1.err" || true

if [ ! -s "$stage1" ]; then
    # Chain breaks at stage0 -> stage1: the current stage0 binary
    # exits 0 without consuming argv. This is the documented
    # SCAFFOLD-tier state — the producer does not yet emit a real
    # compiler from phase0_compiler.phos.
    case "$status" in
        active)
            echo "[fixpoint-chain] stage0 did not produce stage1 — chain BROKEN at active state (exit 1)" >&2
            cat "$work/stage1.err" >&2 || true
            exit 1
            ;;
        SCAFFOLD|*)
            echo "[fixpoint-chain] manifest status=$status (informational; chain breaks below active-tier)"
            echo "    stage0 hash:    $actual_hash (matches manifest)"
            echo "    stage0->stage1: NOT PRODUCED (current stage0 exits 0 without consuming argv)."
            echo "    bottleneck:     producer Pass M.3 (real per-fn variable-size codegen) is the prerequisite for"
            echo "                    stage0 to actually compile compiler/pcc.phos. Until then the chain is"
            echo "                    structurally broken at the first step. See bootstrap/STAGE0_STATUS.md."
            echo "    convergence gate: stage2 == stage3 (gates open until Pass M.3 + N + O + P land)."
            exit 0
            ;;
    esac
fi
stage1_hash="$(sha256sum "$stage1" | awk '{print $1}')"
stage1_size="$(stat -c '%s' "$stage1")"

# ----------------------------------------------------------------------
# 4. stage2 = stage1 on pcc.phos.
# ----------------------------------------------------------------------
chmod +x "$stage1"
stage2="$work/stage2.bin"
"$stage1" "$pcc_src" "$stage2" 2>"$work/stage2.err" || true

if [ ! -s "$stage2" ]; then
    case "$status" in
        active)
            echo "[fixpoint-chain] stage1 did not produce stage2 — chain BROKEN at active state (exit 1)" >&2
            cat "$work/stage2.err" >&2 || true
            exit 1
            ;;
        SCAFFOLD|*)
            echo "[fixpoint-chain] manifest status=$status (informational)"
            echo "    stage0 hash:    $actual_hash"
            echo "    stage1 hash:    $stage1_hash (size $stage1_size)"
            echo "    stage1->stage2: NOT PRODUCED."
            echo "    convergence gate: stage2 == stage3."
            exit 0
            ;;
    esac
fi

# ----------------------------------------------------------------------
# 5. stage3 = stage2 on pcc.phos. Convergence test.
# ----------------------------------------------------------------------
chmod +x "$stage2"
stage3="$work/stage3.bin"
"$stage2" "$pcc_src" "$stage3" 2>"$work/stage3.err" || true

stage2_hash="$(sha256sum "$stage2" | awk '{print $1}')"

if [ ! -s "$stage3" ]; then
    case "$status" in
        active)
            echo "[fixpoint-chain] stage2 did not produce stage3 — chain BROKEN at active state (exit 1)" >&2
            cat "$work/stage3.err" >&2 || true
            exit 1
            ;;
        SCAFFOLD|*)
            echo "[fixpoint-chain] manifest status=$status (informational; chain advanced 2 steps but break point at stage2->stage3)"
            echo "    stage0 hash:    $actual_hash"
            echo "    stage1 hash:    $stage1_hash (size $stage1_size)"
            echo "    stage2 hash:    $stage2_hash"
            echo "    stage2->stage3: NOT PRODUCED."
            echo "    bottleneck:     stage2's embedded canned_elf (the terminal exit-0 in M.3.M) has no write logic. Closing this requires recursion of canned_elf shells (each adds ~315 B; bounded by binary size) OR real lowering of phase0_compiler.phos into stage_n's bytes."
            exit 0
            ;;
    esac
fi

stage3_hash="$(sha256sum "$stage3" | awk '{print $1}')"

if [ "$stage2_hash" = "$stage3_hash" ]; then
    echo "[fixpoint-chain] CONVERGED — stage2 == stage3"
    echo "    stage0 hash:  $actual_hash"
    echo "    stage1 hash:  $stage1_hash"
    echo "    stage2 hash:  $stage2_hash"
    echo "    stage3 hash:  $stage3_hash"
    exit 0
fi

case "$status" in
    active)
        echo "[fixpoint-chain] DIVERGED — stage2 ($stage2_hash) != stage3 ($stage3_hash)" >&2
        exit 1
        ;;
    SCAFFOLD|*)
        echo "[fixpoint-chain] manifest status=$status (informational; chain runs but diverges)"
        echo "    stage0 hash:  $actual_hash"
        echo "    stage1 hash:  $stage1_hash"
        echo "    stage2 hash:  $stage2_hash"
        echo "    stage3 hash:  $stage3_hash"
        echo "    convergence gate: stage2 == stage3 (currently diverged at SCAFFOLD; informational only)"
        exit 0
        ;;
esac
