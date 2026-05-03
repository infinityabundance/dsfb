#!/usr/bin/env bash
# check_court_p1_source_exit.sh — Court Promotion P1-source (narrow):
# first Phosphoric-toolchain-produced court artifact gate.
#
# Forensic role: assert that a Phosphoric-toolchain-produced binary —
# compiled from tools/court/p1_source_exit.phos via the bootstrap chain
# (phase0_stub → stage1 = stage0(phase0_compiler.phos) → court binary) —
# returns the canonical MMIO_BOUNDARY_PRESSURE EXIT code (= 6) by
# performing the observed > declared_hi comparison (4352 > 4351) at
# runtime. The exit code is the channel; bytes are not emitted.
#
# This is the smallest concrete advance the existing compiler chain
# can produce under the ASM authority cutover
# (docs/ASM_AUTHORITY_CUTOVER.md): no phase0_stub.S edits, no new
# producer shapes, no new candidate promotions, no fixture additions.
# The court source uses only sealed 51-shape repertoire (Session J
# entry-call-with-INT-arg + Session P PARAM_MATCH).
#
# Earned claim: a Phosphoric-toolchain-produced binary returns the
# canonical MMIO_BOUNDARY_PRESSURE EXIT code (= 6) from the canonical
# observed / declared_hi inputs by runtime comparison.
#
# NOT earned (still): a Phosphoric-compiled binary emits the 32-byte
# R5 record / 192-byte PFI0 case / 6-line verdict bytes. Byte
# emission is gated on SYS_WRITE lowering, which the cutover defers.
#
# Sanity cross-check: the gate also compiles the source via stage0
# (the ASM stub producer) and asserts the two outputs are
# byte-identical. This mirrors the source↔ASM byte-equal property
# the 51 sealed fixtures already prove, applied to a court artifact
# rather than a fixture witness.
#
# Doctrine: docs/ASM_AUTHORITY_CUTOVER.md, docs/FORENSIC_PRIMACY.md §3.
#
# Exit: 0 pass, 1 EXIT-code drift / byte drift, 2 missing dependency.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

source_path="tools/court/p1_source_exit.phos"
stage0_bin="untracked/internaldocs/phase0_producer/phase0_stub"
stage1_bin="build/phase0/pcc-stage1.bin"

if [ ! -r "$source_path" ]; then
    echo "[court-p1-source-exit] missing court source: $source_path" >&2
    exit 2
fi
if [ ! -x "$stage0_bin" ]; then
    echo "[court-p1-source-exit] stage0 ASM producer missing: $stage0_bin" >&2
    exit 2
fi
if [ ! -x "$stage1_bin" ]; then
    echo "[court-p1-source-exit] stage1 Phosphoric-toolchain compiler missing: $stage1_bin" >&2
    echo "[court-p1-source-exit] hint: run 'make verify-source-asm-byte-equal' once to materialize it" >&2
    exit 2
fi

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

stage0_out="$work_dir/stage0_court.bin"
stage1_out="$work_dir/stage1_court.bin"

# Compile the court source via both producers.
"$stage0_bin" "$source_path" "$stage0_out" 2>/dev/null
"$stage1_bin" "$source_path" "$stage1_out" 2>/dev/null
chmod +x "$stage0_out" "$stage1_out"

stage0_size="$(wc -c < "$stage0_out")"
stage1_size="$(wc -c < "$stage1_out")"
stage0_sha="$(sha256sum "$stage0_out" | awk '{print $1}')"
stage1_sha="$(sha256sum "$stage1_out" | awk '{print $1}')"

echo "============================================================"
echo "  Phosphoric Court Promotion P1-source — EXIT-code gate"
echo "  doctrine: docs/ASM_AUTHORITY_CUTOVER.md + docs/FORENSIC_PRIMACY.md §3"
echo "============================================================"
echo "  source             : $source_path"
echo "  stage0 (ASM)       : $stage0_bin"
echo "  stage1 (Phos-toolchain): $stage1_bin"
echo "  stage0 output      : $stage0_size B sha=${stage0_sha:0:12}"
echo "  stage1 output      : $stage1_size B sha=${stage1_sha:0:12}"

# 1. Byte-equal cross-check: stage1 reproduces stage0 for this source.
if ! cmp -s "$stage0_out" "$stage1_out"; then
    echo "  [court-p1-source-exit] FAIL — stage1 output diverges from stage0 (court source must use only sealed 51-shape repertoire)" >&2
    exit 1
fi

# 2. Exit-code check: Phosphoric-toolchain-produced binary returns 6.
set +e
"$stage1_out"
phos_exit=$?
set -e

echo "  Phos-toolchain binary exit code : $phos_exit"

if [ "$phos_exit" != "6" ]; then
    echo "  [court-p1-source-exit] FAIL — Phosphoric-toolchain-produced binary returned exit=$phos_exit, expected 6 (MMIO_BOUNDARY_PRESSURE)" >&2
    exit 1
fi

echo "  [court-p1-source-exit] OK — Phosphoric-toolchain-produced binary returned canonical MMIO_BOUNDARY_PRESSURE EXIT=6 from runtime observed > declared_hi comparison"
exit 0
