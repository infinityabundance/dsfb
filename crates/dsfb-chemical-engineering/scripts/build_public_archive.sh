#!/usr/bin/env bash
# build_public_archive.sh (P97.2 / P94) — produce + prove a public-release archive.
#
# Materialises the public archive the way a downloader receives it (`git archive HEAD`, which honours the
# `.gitattributes` export-ignore rules — so the untracked SESSION_* backups and the research/ quarantine never
# ship), runs the archive-mode release-scrub over it, and prints the deterministic per-file SHA-256 manifest hash
# (the git-version-independent fingerprint). This is the executable form of the chain-of-custody recipe in
# docs/public_archive_proof.md — see that doc for the full commit -> archive -> manifest -> deposit reasoning.
# Depositing the archive is USER-ONLY (see docs/release_checklist.md SS D).
#
# Usage:   bash scripts/build_public_archive.sh            # uses HEAD
#          bash scripts/build_public_archive.sh <commit>   # any committable ref
# Exit:    0 iff release-scrub reports RELEASE-CLEAN over the materialised archive.
set -euo pipefail

REF="${1:-HEAD}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

COMMIT="$(git rev-parse "$REF")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "== DSFB-Chemical public-archive build =="
echo "commit:      $COMMIT"
echo "archive dir: $TMP"

# 1. Materialise exactly the tracked, export-ignore-filtered tree of the commit.
git archive "$REF" | tar -x -C "$TMP"

# 2. Archive-mode release-scrub: fails hard on a smuggled SESSION_* backup, a controlled-access row, a controlled
#    roles sidecar missing its no-rows flags, or a missing .gitignore/.gitattributes. (The scan is over the
#    materialised tree, not this script.) Capture the findings so they can be recorded in
#    the generated proof (§4) as well as echoed.
echo "== release-scrub --archive-dir =="
if SCRUB_OUT="$(cargo run -q -p dsfb-chemical-engineering-edge -- release-scrub --archive-dir "$TMP" 2>&1)"; then
  SCRUB_OK=1
else
  SCRUB_OK=0
fi
printf '%s\n' "$SCRUB_OUT"

# 3. The deterministic, git-version-independent fingerprint: sorted per-file SHA-256, then one digest over those
#    lines (NOT the tarball byte-hash, which varies across git/tar versions — see public_archive_proof.md).
echo "== archive content fingerprint (sorted per-file SHA-256 -> one digest) =="
MANIFEST_HASH="$(cd "$TMP" && find . -type f -exec sha256sum {} \; | LC_ALL=C sort | sha256sum | awk '{print $1}')"
VERDICT="$( [ "$SCRUB_OK" -eq 1 ] && echo 'RELEASE-CLEAN' || echo 'RELEASE-DIRTY' )"
echo "commit:                 $COMMIT"
echo "archive_manifest_sha256: $MANIFEST_HASH"
echo "(record the commit + this manifest hash in the deposit notes; a verifier re-runs this script and compares.)"

# 4. Emit a GENERATED proof report. It is a deposit-time provenance snapshot for THIS commit — git-ignored, so it
#    never ships inside the archive it fingerprints (a tree-fingerprint cannot live in the tree it hashes without
#    invalidating itself). Regenerate it at deposit and record it alongside the DOI. Method + reasoning prose lives
#    in docs/public_archive_proof.md; this file is the machine-emitted instance for one commit.
PROOF="$REPO_ROOT/reports/public_archive_proof.md"
mkdir -p "$REPO_ROOT/reports"
{
  echo "# Public-archive proof — GENERATED (regenerate at deposit)"
  echo
  echo "> Emitted by \`scripts/build_public_archive.sh\`. A deposit-time provenance snapshot for the commit named"
  echo "> below — **not** a per-commit-synced file (the commit SHA dates it). It is git-ignored so it never ships"
  echo "> inside the archive it fingerprints. The method + chain-of-custody reasoning are in"
  echo "> \`docs/public_archive_proof.md\`; depositing is USER-ONLY (\`docs/release_checklist.md\` §D)."
  echo
  echo "| field | value |"
  echo "|---|---|"
  echo "| commit | \`$COMMIT\` |"
  echo "| archive_manifest_sha256 | \`$MANIFEST_HASH\` |"
  echo "| verdict | **$VERDICT** |"
  echo
  echo "The manifest hash is a sorted per-file SHA-256 over the export-filtered tree (\`git archive\` honours the"
  echo "\`.gitattributes\` export-ignore rules), then one digest over those lines — git/tar-version-independent."
  echo
  echo "## release-scrub --archive-dir findings"
  echo
  # Embed per-check PASS/FAIL + the verdict, but STRIP each finding's detail (everything after ' — '):
  # a check's detail text can quote marker strings (e.g. a placeholder-DOI token), and embedding those
  # verbatim could make this very proof trip a future scan. The
  # check name + pass/fail + verdict is the substance; full detail is in the live `release-scrub` output.
  echo '```'
  printf '%s\n' "$SCRUB_OUT" | sed 's/ — .*$//'
  echo '```'
  echo
  echo "## What this proves about the public archive"
  echo "- **No private backups:** the \`SESSION_*\` working-memory files are export-ignored + the scrub fails on any."
  echo "- **No controlled-access data:** no SWaT/WADI/BATADAL rows, attack lists, or reconstructable windows ship;"
  echo "  the \`research/\` quarantine is export-ignored and every controlled roles sidecar declares its no-rows flags."
  echo "- **No placeholder DOI:** the scrub scans the materialised tree for placeholder-DOI markers."
  echo "- **Hygiene config present:** \`.gitignore\` + \`.gitattributes\` ship and carry the export-ignore rules."
  echo "- **Reproducible identity:** a verifier re-runs this script on the same commit and gets the same manifest hash."
} > "$PROOF"
echo "wrote proof: $PROOF"

if [ "$SCRUB_OK" -eq 1 ]; then
  echo "RESULT: RELEASE-CLEAN — archive is safe to deposit (deposit is USER-ONLY)."
  exit 0
else
  echo "RESULT: RELEASE-DIRTY — do NOT deposit; fix the findings above." >&2
  exit 1
fi
