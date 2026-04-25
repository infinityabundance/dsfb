#!/usr/bin/env bash
# Embedded-target build verification for `dsfb-robotics`.
#
# Builds the no_std + no_alloc core for two representative bare-metal
# targets: Cortex-M4F (thumbv7em-none-eabihf) and 32-bit RISC-V
# (riscv32imac-unknown-none-elf). Both must succeed with the empty
# `default = []` feature set — i.e. zero `alloc`/`std`/`serde`
# dependencies pulled in.
#
# Usage:
#   bash scripts/build_embedded.sh
#
# Pre-reqs (one-time, off-tree):
#   rustup target add thumbv7em-none-eabihf riscv32imac-unknown-none-elf
#
# Exit code: 0 if both targets build clean, non-zero otherwise.

set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$CRATE_ROOT/Cargo.toml"

TARGETS=(
    "thumbv7em-none-eabihf"
    "riscv32imac-unknown-none-elf"
)

echo "==> dsfb-robotics embedded-target build check"
echo "    manifest: $MANIFEST"

for tgt in "${TARGETS[@]}"; do
    echo
    echo "--- Building for target: $tgt"
    if ! rustup target list --installed | grep -qx "$tgt"; then
        echo "ERROR: target $tgt is not installed."
        echo "       Run: rustup target add $tgt"
        exit 1
    fi
    cargo build \
        --manifest-path "$MANIFEST" \
        --no-default-features \
        --target "$tgt"
    echo "OK: $tgt"
done

echo
echo "==> All embedded-target builds succeeded."
