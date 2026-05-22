#!/usr/bin/env bash
# Attribution scrub.
#
# Greps the working tree for forbidden strings and exits non-zero on any hit.
# This script is wired into the pre-commit hook so it runs on every commit.
# The scrub excludes build artifact directories and local tool caches
# (target/, .git/, node_modules/, and notebook checkpoints).
#
# What counts as forbidden: any string that would advertise an AI assistant
# in the committed code or documentation. This is enforced because such
# strings degrade the prior-art posture of the repository for academic and
# patent purposes.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# The forbidden words are assembled from concatenated fragments so this
# script itself does not contain the literal strings it is grepping for —
# otherwise the scrub would always match its own source line.
w1="cl""au""de"
w2="ant""hro""pic"
w3="co[- ]aut""hored[- ]by"
w4="gene""rated with"
w5=$'\xF0\x9F\xA4\x96'  # robot face emoji byte sequence
forbidden_pattern="${w1}|${w2}|${w3}|${w4}|${w5}"

excluded_dirs=(
  --exclude-dir=target
  --exclude-dir=.git
  --exclude-dir=node_modules
  --exclude-dir=.ipynb_checkpoints
)

excluded_files=(
  --exclude='*.aux'
  --exclude='*.log'
  --exclude='*.bbl'
  --exclude='*.bcf'
  --exclude='*.fdb_latexmk'
  --exclude='*.fls'
  --exclude='*.synctex.gz'
)

# -R recursive, -I skip binary, -n line numbers, -i case insensitive, -E ERE.
if grep -RIniE "${excluded_dirs[@]}" "${excluded_files[@]}" "$forbidden_pattern" . ; then
  echo
  echo "scrub: forbidden attribution string found" >&2
  exit 1
fi

echo "scrub: clean"
