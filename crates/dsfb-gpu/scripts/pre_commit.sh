#!/usr/bin/env bash
# Pre-commit pipeline.
#
# Run with no arguments. Wired by `just hooks` which sets
# `git config core.hooksPath .githooks` and ensures the hook symlink is
# executable. Each check is independently runnable from `scripts/`.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "pre-commit: cargo fmt --check"
cargo fmt --all --check

echo "pre-commit: cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "pre-commit: attribution scrub"
bash scripts/scrub.sh

echo "pre-commit: docs freshness"
bash scripts/docs_freshness.sh

echo "pre-commit: all checks passed"
