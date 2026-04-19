#!/usr/bin/env bash
# Contributor pre-commit gate for dsfb-database.
#
# Runs the four byte-level fingerprint locks that pin the paper's claims
# to executable artefacts. A passing run means: the seed-42 stream and
# episode SHAs have not drifted, the seven non-claim strings still match
# the paper §10 tcolorbox verbatim, and the PG / MySQL allow-list query
# concatenation hashes still match the values pinned in the trybuild
# tests.
#
# This is a fast tripwire (~5 s release-build cached). For the full
# pre-PR check including clippy, fmt, deny, and audit, use
# `ci/gate.sh` instead.
#
# Exit codes:
#   0  - all four locks intact
#   1  - one or more locks broke (test name printed by cargo)
#   2  - cargo not on PATH
#
# Invariants enforced (see /home/one/.claude/plans/only-focus-on-dsfb-database-curious-scroll.md
# Pass-2 guardrails):
#   tests/deterministic_replay.rs               stream + episode SHA-256
#   tests/non_claim_lock.rs                     paper §10 ↔ NON_CLAIMS
#   tests/live_query_allowlist_lock.rs          PG allow-list SHA-256
#   tests/live_query_allowlist_lock_mysql.rs    MySQL allow-list SHA-256

set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not on PATH; install Rust toolchain to run the gate" >&2
    exit 2
fi

HERE="$(cd "$(dirname "$0")" && pwd)"
CRATE="$(cd "${HERE}/.." && pwd)"

cd "${CRATE}"

echo "[check_locks] running fingerprint gate against ${CRATE}"
cargo test --release --features full --locked \
    --test deterministic_replay \
    --test non_claim_lock \
    --test live_query_allowlist_lock \
    --test live_query_allowlist_lock_mysql

echo "[check_locks] all four locks intact"
