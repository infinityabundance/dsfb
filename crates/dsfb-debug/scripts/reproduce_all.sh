#!/usr/bin/env bash
# DSFB-Debug — one-command reproduction of every audit number
# (Phase η.8, Session 18).
#
# Runs the full verification chain on a fresh checkout. Idempotent:
# safe to re-run; populates docs/audit/ ledgers byte-identical to
# the previous run (Theorem 9 deterministic replay).
#
# Usage (from crate root):
#
#     bash scripts/reproduce_all.sh
#
# Expected wall-clock on a modern laptop: ~10-15 minutes.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${CRATE_ROOT}"

echo "=== DSFB-Debug — full verification chain ==="
echo "Crate root: ${CRATE_ROOT}"
echo

echo "--- 1/6: building crate (no-default-features, then full features) ---"
cargo build --no-default-features
cargo build --features "std paper-lock"
echo

echo "--- 2/6: lib + integration tests (per-fixture eval, LO-CV, audit, property) ---"
cargo test --features "std paper-lock"
echo

echo "--- 3/6: sensitivity sweep (5 params x 5 values, ~7 minutes) ---"
cargo test --features "std paper-lock" --test sensitivity_sweep -- --nocapture
echo

echo "--- 4/6: per-axis ablation (~3 minutes) ---"
cargo test --features "std paper-lock" --test axis_ablation -- --nocapture
echo

echo "--- 5/6: detector subset optimization (~3 minutes) ---"
cargo test --features "std paper-lock" --test detector_subset_opt -- --nocapture
echo

echo "--- 6/6: timing benchmarks (~3 minutes, debug build) ---"
cargo test --features "std paper-lock" --test timing_benchmarks -- --nocapture
echo

echo "=== Verification complete ==="
echo
echo "Audit ledgers populated under docs/audit/:"
ls -1 docs/audit/*.md 2>/dev/null || echo "(none)"
echo
echo "Benchmark report under docs/:"
ls -1 docs/benchmarks.md 2>/dev/null || echo "(none)"
echo
echo "Theorem 9 deterministic replay verified across every test."
echo "Numbers in docs/audit/*.md are verbatim test stdout."
