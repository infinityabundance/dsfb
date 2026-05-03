#!/usr/bin/env bash
# fixture_corpus.sh — manifest-driven byte-level fixture verifier.
#
# Reads tools/verify/fixture_manifest.toml as authoritative metadata
# (see docs/FIXTURE_RAZOR.md for the doctrine). For each [[fixture]]
# entry: compile the source through phase0_stub, verify size +
# sha256 + rc against the manifest expectations.
#
# Fail closed if:
#   - a manifest entry references a missing source file
#   - a fixture's compiled output drifts from any locked field
#   - a .phos file exists in the fixtures dir with no manifest entry
#     (orphan detection — keeps the manifest authoritative)
#
# Usage:
#   tools/verify/fixture_corpus.sh           # run all manifest entries
#   tools/verify/fixture_corpus.sh <name>    # run one named fixture
#
# Exit 0: all fixtures pass + no orphans.
# Exit 1: any fixture fails or any orphan found.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

producer="untracked/internaldocs/phase0_producer/phase0_stub"
manifest="tools/verify/fixture_manifest.toml"
fixtures_dir="tools/verify/fixtures"

if [ ! -r "$manifest" ]; then
    echo "[fixture_corpus] FAIL: manifest not found at $manifest" >&2
    exit 1
fi

if [ ! -x "$producer" ]; then
    echo "[fixture_corpus] producer not built; building..."
    bash untracked/internaldocs/phase0_producer/build.sh >/dev/null
fi

# -------------------------------------------------------------------
# Parse manifest: extract per-fixture (name, source, size, sha, rc).
# AWK state machine reads block-by-block; each [[fixture]] starts a
# new record, blank line / next [[ resets.
# -------------------------------------------------------------------
parsed="$(awk '
    BEGIN { in_block = 0 }
    /^\[\[fixture\]\]/ {
        if (in_block && name != "") {
            print name "|" source "|" expect_size "|" expect_sha "|" expect_rc
        }
        in_block = 1
        name = ""; source = ""; expect_size = ""; expect_sha = ""; expect_rc = ""
        next
    }
    /^\[\[/ {
        if (in_block && name != "") {
            print name "|" source "|" expect_size "|" expect_sha "|" expect_rc
        }
        in_block = 0
        next
    }
    in_block {
        if ($1 == "name")           { gsub(/[" \t]/, "", $3); name = $3 }
        if ($1 == "source")         { gsub(/[" \t]/, "", $3); source = $3 }
        if ($1 == "expect_size")    { gsub(/[ \t]/, "", $3);  expect_size = $3 }
        if ($1 == "expect_sha256")  { gsub(/[" \t]/, "", $3); expect_sha = $3 }
        if ($1 == "expect_rc")      { gsub(/[ \t]/, "", $3);  expect_rc = $3 }
    }
    END {
        if (in_block && name != "") {
            print name "|" source "|" expect_size "|" expect_sha "|" expect_rc
        }
    }
' "$manifest")"

if [ -z "$parsed" ]; then
    echo "[fixture_corpus] FAIL: manifest parsed to zero entries" >&2
    exit 1
fi

# -------------------------------------------------------------------
# Run a single fixture.
# -------------------------------------------------------------------
run_one() {
    local name="$1" source="$2" expect_size="$3" expect_sha="$4" expect_rc="$5"

    if [ ! -r "$source" ]; then
        printf '  %-30s FAIL  source missing: %s\n' "$name" "$source"
        return 1
    fi

    local out
    out="$(mktemp --suffix=.bin)"
    trap 'rm -f "$out"' RETURN

    if ! "$producer" "$source" "$out" 2>/dev/null; then
        printf '  %-30s FAIL  producer error\n' "$name"
        return 1
    fi
    chmod +x "$out"

    local actual_size actual_sha actual_rc=0
    actual_size="$(stat -c '%s' "$out")"
    actual_sha="$(sha256sum "$out" | awk '{print $1}')"
    "$out" </dev/null >/dev/null 2>&1 || actual_rc="$?"

    local pass=1 notes=""
    if [ -n "$expect_size" ] && [ "$expect_size" != "$actual_size" ]; then
        pass=0; notes+=" size:expect=$expect_size,got=$actual_size"
    fi
    if [ -n "$expect_sha" ] && [ "$expect_sha" != "$actual_sha" ]; then
        pass=0; notes+=" sha:mismatch"
    fi
    if [ -n "$expect_rc" ] && [ "$expect_rc" != "$actual_rc" ]; then
        pass=0; notes+=" rc:expect=$expect_rc,got=$actual_rc"
    fi

    if [ "$pass" -eq 1 ]; then
        printf '  %-30s PASS  size=%s rc=%s\n' "$name" "$actual_size" "$actual_rc"
        return 0
    else
        printf '  %-30s FAIL %s\n' "$name" "$notes"
        printf '       got: size=%s sha=%s rc=%s\n' "$actual_size" "$actual_sha" "$actual_rc"
        return 1
    fi
}

# -------------------------------------------------------------------
# Main loop.
# -------------------------------------------------------------------
target="${1:-}"
fails=0
total=0

echo "============================================================"
echo "  Phosphoric byte-level fixture corpus (manifest-driven)"
echo "============================================================"

# Track which sources are referenced by the manifest (for orphan check).
declare -A manifest_sources

while IFS='|' read -r name source size sha rc; do
    [ -z "$name" ] && continue
    manifest_sources["$source"]=1
    if [ -n "$target" ] && [ "$target" != "$name" ]; then
        continue
    fi
    total=$((total + 1))
    if ! run_one "$name" "$source" "$size" "$sha" "$rc"; then
        fails=$((fails + 1))
    fi
done <<< "$parsed"

# -------------------------------------------------------------------
# Orphan check: any *.phos in fixtures/ that isn't referenced by the
# manifest is a doctrine violation (the manifest is authoritative).
# Skip when running a single named fixture.
# -------------------------------------------------------------------
orphans=0
if [ -z "$target" ] && [ -d "$fixtures_dir" ]; then
    while IFS= read -r src; do
        if [ -z "${manifest_sources[$src]:-}" ]; then
            printf '  %-30s ORPHAN %s\n' "(no name)" "$src"
            orphans=$((orphans + 1))
        fi
    done < <(find "$fixtures_dir" -maxdepth 1 -name '*.phos' | sort)
fi

echo "============================================================"
echo "$((total - fails)) / $total passed"
if [ "$orphans" -gt 0 ]; then
    echo "$orphans orphan fixture(s) found (in fixtures dir but not in manifest)"
fi

if [ "$fails" -eq 0 ] && [ "$orphans" -eq 0 ]; then
    exit 0
else
    exit 1
fi
