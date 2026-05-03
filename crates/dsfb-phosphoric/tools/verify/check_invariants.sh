#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="$repo_root/untracked/internaldocs/docs/invariant_manifest.toml"

cd "$repo_root"

if [ ! -f "$manifest" ]; then
  # The invariant manifest is internal-only. When it is not present
  # in the working tree this gate is a no-op.
  printf '%s\n' "== phosphoric: invariant manifest not present (internal-only); skipping =="
  exit 0
fi

awk '
  BEGIN {
    block = 0
    id = status = spec_source = enforcement = evidence = status_ref = claim_ref = ""
  }
  function emit() {
    if (block == 0) {
      return
    }
    print id "\t" status "\t" spec_source "\t" enforcement "\t" evidence "\t" status_ref "\t" claim_ref
  }
  /^\[\[invariant\]\]$/ {
    emit()
    block = 1
    id = status = spec_source = enforcement = evidence = status_ref = claim_ref = ""
    next
  }
  /^[[:space:]]*[a-z_]+[[:space:]]*=/ {
    split($0, parts, "=")
    key = parts[1]
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
    value = substr($0, index($0, "=") + 1)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
    gsub(/^"/, "", value)
    gsub(/"$/, "", value)
    if (key == "id") id = value
    else if (key == "status") status = value
    else if (key == "spec_source") spec_source = value
    else if (key == "enforcement") enforcement = value
    else if (key == "evidence") evidence = value
    else if (key == "status_ref") status_ref = value
    else if (key == "claim_ref") claim_ref = value
  }
  END {
    emit()
  }
' "$manifest" | while IFS=$'\t' read -r id status spec_source enforcement evidence status_ref claim_ref; do
  if [ -z "$id" ]; then
    continue
  fi

  case "$status" in
    enforced|partial|specified|review-gated|not-yet-enforced)
      ;;
    *)
      printf 'invariant %s has unknown status: %s\n' "$id" "$status" >&2
      exit 1
      ;;
  esac

  if [ "$status" = "enforced" ]; then
    for field_name in spec_source enforcement evidence status_ref claim_ref; do
      field_value="${!field_name}"
      if [ -z "$field_value" ]; then
        printf 'invariant %s is missing required field %s\n' "$id" "$field_name" >&2
        exit 1
      fi
    done

    for field_name in spec_source enforcement evidence; do
      field_value="${!field_name}"
      if [ ! -e "$field_value" ]; then
        printf 'invariant %s references missing %s path: %s\n' "$id" "$field_name" "$field_value" >&2
        exit 1
      fi
    done

    case "$enforcement" in
      archive/*)
        printf 'invariant %s uses archive path as active enforcement: %s\n' "$id" "$enforcement" >&2
        exit 1
        ;;
    esac

    case "$evidence" in
      archive/*)
        if ! printf '%s %s\n' "$status_ref" "$claim_ref" | grep -Fq "archived reference"; then
          printf 'invariant %s uses archive evidence without an archived-reference claim: %s\n' "$id" "$evidence" >&2
          exit 1
        fi
        ;;
    esac

    if [ ! -e "$evidence" ]; then
      printf 'invariant %s references missing evidence path: %s\n' "$id" "$evidence" >&2
      exit 1
    fi

    if ! grep -Fq "$status_ref" untracked/internaldocs/root/STATUS.md; then
      printf 'invariant %s status_ref missing from internal STATUS.md: %s\n' "$id" "$status_ref" >&2
      exit 1
    fi

    if ! grep -Fq "$claim_ref" untracked/internaldocs/root/CLAIMS.md; then
      printf 'invariant %s claim_ref missing from internal CLAIMS.md: %s\n' "$id" "$claim_ref" >&2
      exit 1
    fi
  fi
done

printf '%s\n' "== phosphoric: invariant manifest verification complete =="
