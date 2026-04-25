#!/usr/bin/env bash
# scripts/run_audit.sh — reproduce the full dsfb-robotics audit surface.
#
# Runs (in order):
#   1. cargo check across every feature matrix
#   2. cargo clippy --all-features -D warnings
#   3. cargo test --no-default-features --lib
#   4. cargo test --features std,paper_lock (lib + integration)
#   5. cargo test --features std --test proptest_invariants
#   6. Miri × 3 configurations (nightly required)
#   7. Kani (if installed)
#   8. cargo-fuzz (if installed) — short runs only; full runs stay in CI
#
# All outputs tee'd into `audit/run_audit.log`. Tool-missing is
# reported but non-fatal: a reviewer with only stable rustc can still
# confirm the stock-cargo-test surface.

set -o pipefail
set -u

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
M="${CRATE_DIR}/Cargo.toml"
LOG="${CRATE_DIR}/audit/run_audit.log"
mkdir -p "${CRATE_DIR}/audit"
: >"${LOG}"

log() { echo "$@" | tee -a "${LOG}"; }
heading() { log; log "=============================================================="; log "$@"; log "=============================================================="; }

missing_tool() { log "  (tool missing: $1) — skipping"; }

heading "1. cargo check across feature matrix"
for feat in "" "alloc" "std" "std,serde" "std,paper_lock" "std,serde,paper_lock,real_figures"; do
    label="${feat:-no-default-features}"
    log "-- cargo check --features ${label}"
    if [ -z "${feat}" ]; then
        cargo check --manifest-path "${M}" --no-default-features 2>&1 | tail -2 | tee -a "${LOG}"
    else
        cargo check --manifest-path "${M}" --features "${feat}" 2>&1 | tail -2 | tee -a "${LOG}"
    fi
done

heading "2. cargo clippy --all-features -D warnings"
cargo clippy --manifest-path "${M}" --all-features -- -D warnings 2>&1 | tail -6 | tee -a "${LOG}"

heading "3. cargo test --no-default-features --lib"
cargo test --manifest-path "${M}" --no-default-features --lib 2>&1 | tail -2 | tee -a "${LOG}"

heading "4. cargo test --features std,paper_lock"
cargo test --manifest-path "${M}" --features std,paper_lock --lib 2>&1 | grep "test result" | tee -a "${LOG}"
cargo test --manifest-path "${M}" --features std,paper_lock --tests 2>&1 | grep "test result" | tee -a "${LOG}"

heading "5. cargo test --test proptest_invariants"
cargo test --manifest-path "${M}" --features std --test proptest_invariants 2>&1 | tail -4 | tee -a "${LOG}"

heading "6. Miri × 3 configurations"
if command -v cargo-miri >/dev/null 2>&1; then
    log "-- Miri 1/3: no_std + strict provenance"
    MIRIFLAGS="-Zmiri-strict-provenance" \
        cargo +nightly miri test --manifest-path "${M}" --no-default-features --lib \
        2>&1 | tee "${CRATE_DIR}/audit/miri/miri_nostd_strict.txt" | tail -4 | tee -a "${LOG}"

    log "-- Miri 2/3: std+serde stacked borrows"
    MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
        cargo +nightly miri test --manifest-path "${M}" --features std,serde --lib \
        2>&1 | tee "${CRATE_DIR}/audit/miri/miri_std_stacked.txt" | tail -4 | tee -a "${LOG}"

    log "-- Miri 3/3: std+serde tree borrows"
    MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation -Zmiri-tree-borrows" \
        cargo +nightly miri test --manifest-path "${M}" --features std,serde --lib \
        2>&1 | tee "${CRATE_DIR}/audit/miri/miri_std_tree.txt" | tail -4 | tee -a "${LOG}"
else
    missing_tool "cargo-miri (install via: rustup component add miri --toolchain nightly)"
fi

heading "7. Kani"
if command -v cargo-kani >/dev/null 2>&1; then
    # Kani verifies the `#[kani::proof]` harnesses in `src/kani_proofs.rs`.
    # Scope to the library to avoid building the feature-gated
    # paper-lock binary, which Kani's model-checking runtime cannot
    # consume (serde_json is outside its feature subset).
    cargo kani --manifest-path "${M}" --no-default-features --lib 2>&1 | tail -30 | tee -a "${LOG}"
else
    missing_tool "cargo-kani (install via: cargo install --locked kani-verifier && cargo kani-setup)"
fi

heading "8. cargo-fuzz (short runs — 30s each)"
if command -v cargo-fuzz >/dev/null 2>&1; then
    FUZZ_DIR="${CRATE_DIR}/fuzz"
    for target in engine_roundtrip grammar_fsm; do
        log "-- cargo fuzz run ${target} (30s)"
        (cd "${FUZZ_DIR}" && timeout 30 cargo +nightly fuzz run "${target}" -- -max_total_time=25 2>&1 | tail -4) | tee -a "${LOG}" || true
    done
else
    missing_tool "cargo-fuzz (install via: cargo install cargo-fuzz)"
fi

heading "DONE — full log at ${LOG}"
