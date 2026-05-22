#!/usr/bin/env bash
# pack_for_colab.sh — produce the source tarball used by the
# Colab notebook (notebooks/dsfb_gpu_debug_colab.ipynb).
#
# WHY THIS EXISTS (for the future engineer reading cold):
#
# The Colab notebook is the COLAB.S-REAL.1 public-replay surface (sealed
# operational flow post-S-REAL.3.1.1). It downloads the checked-in
# `notebooks/dsfb-gpu.tar.gz` bundle from GitHub raw content, extracts
# the tarball, builds the CUDA binary, runs `s-real-audit --dataset all`
# against the 20 panel-locked vendored fixtures, and packages the
# results via `scripts/package_s_real_colab_outputs.sh`.
#
# The tarball MUST include:
#   - the full source tree (crates/, cuda/, Cargo.toml, Cargo.lock,
#     rust-toolchain.toml) so the notebook can rebuild from scratch
#   - data/fixtures/*.tsv — the 20 SHA-pinned audit input TSVs
#   - data/fixtures/MANIFEST.toml — the SHA-256 pin table the notebook
#     verifies against (B-gate)
#   - notebooks/ — for self-reference (so an operator who unpacks the
#     tarball outside Colab can find the README)
#   - scripts/ — including package_s_real_colab_outputs.sh which the
#     notebook calls at section 5
# The tarball MUST exclude:
#   - target/                   — build artifacts; Colab rebuilds from
#                                 source
#   - .git/                     — version control metadata; the notebook
#                                 reads the sealed commit SHA from the
#                                 tarball's anchors inside Cargo.lock
#                                 plus the embedded receipt, not from git
#   - notebooks/dsfb-gpu.tar.gz — the checked-in bundle itself, to avoid
#                                 self-nesting on every refresh
#
# The implementation below uses an explicit allowlist-by-exclusion
# (every top-level entry except target/ and .git/) which
# naturally covers all of the above. `shopt -s dotglob` also picks up
# dotfiles like `.gitignore` and `.githooks/` for reproducibility.
#
# The archive lands in both `target/colab/dsfb-gpu.tar.gz` and the
# checked-in `notebooks/dsfb-gpu.tar.gz`. The tarball is rooted at a
# single top-level directory `dsfb-gpu/` so the notebook's `tar -xzf`
# lands the source at a known path.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mkdir -p target/colab notebooks
out=target/colab/dsfb-gpu.tar.gz
builtin_out=notebooks/dsfb-gpu.tar.gz

# Stage the repo into a tmpdir under target/ so the tarball is rooted at
# `dsfb-gpu/` rather than at the user's absolute path. We exclude target/
# and .git/ from the staged source tree.
stage="target/colab/_stage"
rm -rf "$stage"
mkdir -p "$stage/dsfb-gpu"

# Use rsync-like semantics via cp -a + careful excludes. cp is universal,
# rsync is not.
shopt -s extglob dotglob
for path in *; do
    case "$path" in
        target|.git)
            continue
            ;;
        *)
            cp -a "$path" "$stage/dsfb-gpu/"
            ;;
    esac
done

# Slim down the staged tree to the audit-essential payload. The audit
# binary needs only `data/fixtures/<id>.tsv` (20 small TSVs ~ 250 KB
# total) plus `data/fixtures/MANIFEST.toml`. The large saturation-class
# fixtures (`*_1024x1024.tsv`, ~9 MB each × 10, gitignored on the dev
# machine but physically present after a sweep run) and the upstream-
# archive directory (`data/upstream/` + the top-level `CMAPSSData.zip`,
# reconstruction-only) are NOT needed for the Colab audit and would
# add ~100 MB of dead weight to the upload. Excise them.
if [[ -d "$stage/dsfb-gpu/data/upstream" ]]; then
    rm -rf "$stage/dsfb-gpu/data/upstream"
fi
if [[ -f "$stage/dsfb-gpu/data/CMAPSSData.zip" ]]; then
    rm -f "$stage/dsfb-gpu/data/CMAPSSData.zip"
fi
# Drop saturation-class fixtures (file glob: any TSV whose stem ends
# in `_<N>x1024` for any N — covers 1024x1024 standard variants,
# imdb_tgz's 1020x1024, and deepsense6g's 512x1024). The audit DOES
# NOT use these; the saturation sweep is dev-machine hardware-
# anchored and not replayed on Colab.
find "$stage/dsfb-gpu/data/fixtures" -maxdepth 1 -type f -name '*x1024.tsv' -delete

# If an older built-in bundle exists in the working tree, it was copied
# into the stage by the top-level copy loop. Remove it before creating
# the refreshed tarball so the archive never contains itself.
rm -f "$stage/dsfb-gpu/$builtin_out"

tar -C "$stage" -czf "$out" dsfb-gpu
rm -rf "$stage"
cp "$out" "$builtin_out"

bytes=$(stat -c %s "$out")
echo "pack_for_colab: wrote $out (${bytes} bytes)"
builtin_bytes=$(stat -c %s "$builtin_out")
echo "pack_for_colab: refreshed $builtin_out (${builtin_bytes} bytes)"
