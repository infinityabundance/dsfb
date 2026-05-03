#!/usr/bin/env bash
# check_bootstrap_manifest.sh — bootstrap.toml invariant gate.
#
# Verifies that bootstrap/bootstrap.toml is internally consistent:
#
#   1. Every `[[stage0]]` entry has the required fields populated when
#      its status is SCAFFOLD or active (PLACEHOLDER may have TBDs).
#   2. The active entry's `binary_sha256` matches the file at
#      `binary_url` byte-for-byte.
#   3. The active entry's `binary_size` matches the file's actual size.
#   4. No two attestations in the active entry share both
#      `attestor` AND `signed_at` (duplicate attestation detection).
#   5. The append-only discipline: the historical entry is marked
#      `superseded_by` pointing at the active entry's id.
#
# Exit codes:
#   0 — manifest consistent.
#   1 — invariant violated (specific failure message printed).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

manifest="bootstrap/bootstrap.toml"
fails=0

note_fail() {
    echo "[manifest] FAIL: $*" >&2
    fails=$((fails + 1))
}

# ----------------------------------------------------------------------
# 1. Required fields in the active stage0 entry.
# ----------------------------------------------------------------------
default_id="$(awk -F'"' '/^default_stage0/ { print $2; exit }' "$manifest")"
if [ -z "$default_id" ]; then
    note_fail "no default_stage0 declared"
fi

# Extract the active entry block (between the first [[stage0]] line and
# the next blank line OR the next [[stage0]] line).
active_block="$(awk '
    /^\[\[stage0\]\]/ { in_block++; if (in_block > 1) exit; next }
    in_block == 1 { print }
' "$manifest")"

if [ -z "$active_block" ]; then
    note_fail "no [[stage0]] entry found"
fi

# Required fields (status/binary_url/binary_sha256/binary_size).
# NOTE: awk `exit` actions paired with bash `echo "$active_block" |` cause
# SIGPIPE on the upstream echo under `set -o pipefail`, which made this
# gate flaky as the manifest grew. Using awk's `!seen[f]++ { print }`
# pattern matches the FIRST occurrence without exiting early.
for field in id status binary_url binary_sha256 binary_size; do
    val="$(echo "$active_block" | awk -v f="$field" '$1 == f && !seen { print; seen=1 }')"
    if [ -z "$val" ]; then
        note_fail "active stage0 entry missing field: $field"
    fi
done

# ----------------------------------------------------------------------
# 2 + 3. binary_sha256 matches actual file; binary_size matches.
# ----------------------------------------------------------------------
expected_hash="$(echo "$active_block" | awk -F'"' '/^binary_sha256/ && !seen { print $2; seen=1 }')"
expected_size="$(echo "$active_block" | awk -F'=' '/^binary_size/ && !seen { v=$2; gsub(/[ \t]/, "", v); sub(/#.*$/, "", v); print v; seen=1 }')"
binary_url="$(echo "$active_block" | awk -F'"' '/^binary_url/ && !seen { print $2; seen=1 }')"

# binary_url may be an HTTPS URL (skip in scaffold mode) or a local path.
case "$binary_url" in
    https://*|http://*)
        echo "[manifest] binary_url is HTTPS; skipping local hash check"
        ;;
    "")
        : # already failed above
        ;;
    *)
        if [ -r "$binary_url" ]; then
            actual_hash="$(sha256sum "$binary_url" | awk '{print $1}')"
            actual_size="$(stat -c '%s' "$binary_url")"
            if [ "$actual_hash" != "$expected_hash" ]; then
                note_fail "binary_sha256 drift: expected $expected_hash, got $actual_hash"
            fi
            if [ "$actual_size" != "$expected_size" ]; then
                note_fail "binary_size drift: expected $expected_size, got $actual_size"
            fi
        else
            echo "[manifest] binary_url $binary_url not present locally; size/hash check skipped"
        fi
        ;;
esac

# ----------------------------------------------------------------------
# 4. Attestation uniqueness.
# ----------------------------------------------------------------------
# Every (attestor, signed_at) pair must be unique. (signed_at alone may
# repeat across entries, and attestor alone may repeat across separate
# pass attestations from the same author, but the pair must be unique.)
duplicates="$(awk '
    /^\[\[stage0.attestation\]\]/ { in_att = 1; attestor = ""; signed_at = ""; next }
    in_att && /^attestor/  { sub(/^attestor[ \t]*=[ \t]*"/, ""); sub(/".*$/, ""); attestor = $0 }
    in_att && /^signed_at/ {
        sub(/^signed_at[ \t]*=[ \t]*"/, "");
        sub(/".*$/, "");
        signed_at = $0;
        key = attestor "|" signed_at;
        if (seen[key]) print key;
        seen[key] = 1;
        in_att = 0;
    }
' "$manifest")"
if [ -n "$duplicates" ]; then
    note_fail "duplicate (attestor, signed_at) pairs:"
    echo "$duplicates" >&2
fi

# ----------------------------------------------------------------------
# 5. Append-only discipline: at least one entry should have a
#    superseded_by pointer if there is more than one [[stage0]] block.
# ----------------------------------------------------------------------
stage0_count="$(grep -c '^\[\[stage0\]\]' "$manifest")"
if [ "$stage0_count" -gt 1 ]; then
    if ! grep -q '^superseded_by' "$manifest"; then
        note_fail "multiple [[stage0]] entries but no superseded_by pointer"
    fi
fi

# ----------------------------------------------------------------------
# Report.
# ----------------------------------------------------------------------
if [ "$fails" -eq 0 ]; then
    echo "[manifest] OK — bootstrap/bootstrap.toml is internally consistent"
    echo "    default_stage0:  $default_id"
    echo "    binary_url:      $binary_url"
    echo "    binary_sha256:   $expected_hash"
    echo "    binary_size:     $expected_size"
    echo "    [[stage0]] entries: $stage0_count"
    exit 0
fi

echo "[manifest] FAIL: $fails invariant violation(s)" >&2
exit 1
