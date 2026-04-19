#!/usr/bin/env bash
# Full pre-PR gate for dsfb-database.
#
# This script is the canonical CI entry point for the crate. It is kept
# as a self-contained bash script (rather than a GitHub Actions YAML)
# because the Pass-2 work is scoped strictly to the crate directory; the
# user can wire this into root-level `.github/workflows/` whenever they
# choose by adding one workflow that runs `bash crates/dsfb-database/ci/gate.sh`.
#
# Order matters: the fingerprint gate is job #1. Any drift in
# stream / episode / non-claim / allow-list SHAs is a fast-fail tripwire.
# Build / lint / supply-chain follow only if the locks pass.
#
# Tools required (script will skip-and-warn if a tool is missing rather
# than fail, so a contributor without cargo-deny / cargo-audit installed
# can still run the rest of the gate locally):
#
#   cargo, rustc, cargo-fmt, cargo-clippy            (rustup default)
#   cargo-deny                                       (cargo install cargo-deny)
#   cargo-audit                                      (cargo install cargo-audit)
#
# See `/home/one/.claude/plans/only-focus-on-dsfb-database-curious-scroll.md`
# Pass-2 Track M1 / M2 for the rationale.

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
CRATE="$(cd "${HERE}/.." && pwd)"
cd "${CRATE}"

log() { echo "[gate] $*"; }
have() { command -v "$1" >/dev/null 2>&1; }

log "==> 1/8  Fingerprint gate (locks must hold before any other job)"
bash scripts/check_locks.sh

log "==> 2/8  cargo build --release across feature combinations"
for feats in "cli report" "cli report live-postgres" "cli report live-mysql" "cli report otel" "full"; do
    log "    features: ${feats}"
    cargo build --release --features "${feats}" --locked --quiet
done

log "==> 3/8  cargo test --release across feature combinations"
for feats in "cli report" "cli report live-postgres" "cli report live-mysql" "full"; do
    log "    features: ${feats}"
    cargo test --release --features "${feats}" --locked --quiet
done

log "==> 4/8  cargo fmt --check"
cargo fmt --all -- --check

log "==> 5/8  cargo clippy -- -D warnings"
cargo clippy --release --features full --locked --all-targets -- -D warnings

log "==> 6/8  Trybuild compile-fail proofs (live ReadOnlyConn surface)"
cargo test --release --features full --locked --test live_readonly_conn_surface

if have cargo-deny; then
    log "==> 7/8  cargo-deny check"
    cargo deny --all-features check
else
    log "==> 7/8  cargo-deny not installed; skipping (install: cargo install cargo-deny)"
fi

if have cargo-audit; then
    log "==> 8/8  cargo-audit"
    cargo audit
else
    log "==> 8/8  cargo-audit not installed; skipping (install: cargo install cargo-audit)"
fi

log "done. dsfb-database gate passed."
